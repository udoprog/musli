use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::punctuated::Punctuated;
use syn::Token;

use crate::expander::{NameMethod, StructKind};
use crate::internals::apply;
use crate::internals::attr::{EnumTagging, Packing};
use crate::internals::build::{Body, Build, BuildData, Enum, Field, Variant};
use crate::internals::{Import, Result, Tokens};

struct Ctxt<'a> {
    ctx_var: &'a Ident,
    decoder_var: &'a Ident,
    name_var: &'a Ident,
    trace: bool,
    trace_body: bool,
}

pub(crate) fn expand_decode_entry(e: Build<'_, '_>) -> Result<TokenStream> {
    e.validate_decode()?;
    e.cx.reset();

    let ctx_var = e.cx.ident("ctx");
    let decoder_var = e.cx.ident("decoder");
    let tag_var = e.cx.ident("tag");
    let d_param = e.cx.type_with_span("D", Span::call_site());

    let cx = Ctxt {
        ctx_var: &ctx_var,
        decoder_var: &decoder_var,
        name_var: &tag_var,
        trace: true,
        trace_body: true,
    };

    let packed;

    let body = match &e.data {
        BuildData::Struct(st) => {
            packed = crate::internals::packed(&e, st);
            decode_struct(&cx, &e, st)?
        }
        BuildData::Enum(en) => {
            packed = syn::parse_quote!(false);
            decode_enum(&cx, &e, en)?
        }
    };

    if e.cx.has_errors() {
        return Err(());
    }

    // Figure out which lifetime to use for what. We use the first lifetime in
    // the type (if any is available) as the decoder lifetime. Else we generate
    // a new anonymous lifetime `'de` to use for the `Decode` impl.
    let mut generics = e.input.generics.clone();
    let type_ident = &e.input.ident;

    let Tokens {
        allocator_t,
        context_t,
        result,
        decode_t,
        decoder_t,
        try_fast_decode,
        ..
    } = e.tokens;

    let lt = &e.p.lt;

    if !e.p.lt_exists {
        generics
            .params
            .push(syn::GenericParam::Lifetime(syn::LifetimeParam {
                attrs: Vec::new(),
                lifetime: lt.clone(),
                colon_token: None,
                bounds: Punctuated::new(),
            }));
    }

    let allocator_ident = &e.p.allocator_ident;

    if !e.p.allocator_exists {
        generics
            .params
            .push(syn::GenericParam::Type(allocator_ident.clone().into()));

        generics
            .make_where_clause()
            .predicates
            .push(syn::parse_quote!(#allocator_ident: #allocator_t));
    }

    if !e.bounds.is_empty() && !e.decode_bounds.is_empty() {
        generics.make_where_clause().predicates.extend(
            e.bounds
                .iter()
                .chain(e.decode_bounds.iter())
                .map(|(_, v)| v.clone()),
        );
    }

    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let (_, type_generics, _) = e.input.generics.split_for_impl();

    let mut attributes = Vec::<syn::Attribute>::new();

    if cfg!(not(feature = "verbose")) {
        attributes.push(syn::parse_quote!(#[allow(clippy::just_underscores_and_digits)]));
    }

    let mode_ident = e.expansion.mode_path(e.tokens);

    Ok(quote! {
        const _: () = {
            #[automatically_derived]
            #(#attributes)*
            impl #impl_generics #decode_t<#lt, #mode_ident, #allocator_ident> for #type_ident #type_generics
            #where_clause
            {
                const IS_BITWISE_DECODE: bool = #packed;

                #[inline]
                fn decode<#d_param>(#decoder_var: #d_param) -> #result<Self, <#d_param as #decoder_t<#lt>>::Error>
                where
                    #d_param: #decoder_t<#lt, Mode = #mode_ident, Allocator = #allocator_ident>,
                {
                    let #ctx_var = #decoder_t::cx(&#decoder_var);

                    let #decoder_var = match #decoder_t::try_fast_decode(#decoder_var)? {
                        #try_fast_decode::Ok(value) => return #result::Ok(value),
                        #try_fast_decode::Unsupported(#decoder_var) => #decoder_var,
                        _ => return #result::Err(#context_t::message(#ctx_var, "Fast decoding failed")),
                    };

                    #body
                }
            }
        };
    })
}

fn decode_struct(cx: &Ctxt<'_>, b: &Build<'_, '_>, st: &Body<'_>) -> Result<TokenStream> {
    let Tokens { result, .. } = b.tokens;

    let body = match (st.kind, st.packing) {
        (_, Packing::Transparent) => decode_transparent(cx, b, st)?,
        (_, Packing::Packed) => decode_packed(cx, b, st)?,
        (StructKind::Empty, _) => decode_empty(cx, b, st)?,
        (_, Packing::Tagged) => decode_tagged(cx, b, st, None)?,
    };

    Ok(quote!(#result::Ok({ #body })))
}

fn decode_enum(cx: &Ctxt<'_>, b: &Build<'_, '_>, en: &Enum) -> Result<TokenStream> {
    let Ctxt {
        ctx_var,
        name_var,
        decoder_var,
        ..
    } = *cx;

    let Tokens {
        as_decoder_t,
        context_t,
        decoder_t,
        fmt,
        option,
        result,
        skip_field,
        skip,
        map_decoder_t,
        struct_field_decoder_t,
        map_hint,
        variant_decoder_t,
        ..
    } = b.tokens;

    if let Some(&(span, Packing::Packed)) = en.packing_span {
        b.decode_packed_enum_diagnostics(span);
        return Err(());
    }

    let type_name = en.name;

    // Trying to decode an uninhabitable type.
    if en.variants.is_empty() {
        return Ok(quote!(#result::Err(#context_t::uninhabitable(#ctx_var, #type_name))));
    }

    let binding_var = b.cx.ident("binding");
    let body_decoder_var = b.cx.ident("body_decoder");
    let buffer_decoder_var = b.cx.ident("buffer_decoder");
    let buffer_var = b.cx.ident("buffer");
    let entry_var = b.cx.ident("entry");
    let field_name = b.cx.ident("field_name");
    let field_name_var = b.cx.ident("field_name");
    let field_var = b.cx.ident("field");
    let outcome_type = b.cx.type_with_span("Outcome", Span::call_site());
    let buf_type = b.cx.type_with_span("B", Span::call_site());
    let outcome_var = b.cx.ident("outcome");
    let output_var = b.cx.ident("output");
    let struct_decoder_var = b.cx.ident("struct_decoder");
    let struct_hint_static = b.cx.ident("STRUCT_HINT");
    let struct_var = b.cx.ident("st");
    let value_var = b.cx.ident("value");
    let variant_decoder_var = b.cx.ident("variant_decoder");
    let variant_tag_var = b.cx.ident("variant_tag");
    let tag_static = b.cx.ident("TAG");
    let content_static = b.cx.ident("CONTENT");

    let mut output_arms = Vec::new();

    let mut fallback = match en.fallback {
        Some(ident) => {
            quote! {{
                if #skip(#variant_decoder_t::decode_value(#variant_decoder_var)?)? {
                    return #result::Err(#context_t::invalid_variant_tag(#ctx_var, #type_name, &#variant_tag_var));
                }

                Self::#ident {}
            }}
        }
        None => quote! {
            return #result::Err(#context_t::invalid_variant_tag(#ctx_var, #type_name, &#variant_tag_var))
        },
    };

    let decode_name;
    let output_enum;
    let name_type;

    match en.name_type.method {
        NameMethod::Sized => {
            for v in &en.variants {
                let arm = output_arm(v.pattern, &v.name, &binding_var);
                output_arms.push((v, arm, &v.name));
            }

            let decode_t_decode = &b.decode_t_decode;

            decode_name = quote!(#decode_t_decode(#variant_decoder_var));
            output_enum = None;
            fallback = quote!(_ => #fallback);
            name_type = en.name_type.ty.clone();
        }
        NameMethod::Unsized(method) => {
            let mut variants = Vec::new();
            let output_type = b.cx.type_with_span("VariantTag", en.span);

            for v in &en.variants {
                let (pat, variant) =
                    unsized_arm(b, v.span, v.index, &v.name, v.pattern, &output_type);

                output_arms.push((v, OutputArm { pat, cond: None }, &v.name));
                variants.push(variant);
            }

            let arms = variants.iter().map(|o| o.as_arm(&binding_var, option));

            let visit_type = &en.name_type.ty;
            let method = method.as_method_name();

            decode_name = quote! {
                #decoder_t::#method(#variant_decoder_var, |#value_var: &#visit_type| {
                    #result::Ok(match #value_var {
                        #(#arms,)*
                        _ => #option::None,
                    })
                })
            };

            let fmt_patterns = variants.iter().map(|o| {
                let variant = &o.variant;
                let name = o.name;
                quote!(#output_type::#variant => #fmt::Debug::fmt(&#name, f))
            });

            let fmt_patterns2 = variants.iter().map(|o| {
                let variant = &o.variant;
                let name = o.name;
                quote!(#output_type::#variant => #fmt::Display::fmt(&#name, f))
            });

            let variants = variants.iter().map(|o| &o.variant);

            output_enum = Some(quote! {
                enum #output_type { #(#variants,)* }

                impl #fmt::Debug for #output_type {
                    #[inline]
                    fn fmt(&self, f: &mut #fmt::Formatter<'_>) -> #fmt::Result {
                        match *self { #(#fmt_patterns,)* }
                    }
                }

                impl #fmt::Display for #output_type {
                    #[inline]
                    fn fmt(&self, f: &mut #fmt::Formatter<'_>) -> #fmt::Result {
                        match *self { #(#fmt_patterns2,)* }
                    }
                }
            });

            fallback = quote!(#option::None => { #fallback });
            name_type = syn::parse_quote!(#option<#output_type>);
        }
    }

    match &en.enum_tagging {
        EnumTagging::Empty => {
            let mut arms = Vec::new();

            for v in &en.variants {
                let path = &v.st.path;
                let pat = output_arm(v.pattern, &v.name, &binding_var);
                arms.push(quote!(#pat => #result::Ok(#path {})));
            }

            match en.fallback {
                Some(ident) => {
                    arms.push(quote!(_ => #result::Ok(Self::#ident {})));
                }
                None => {
                    arms.push(quote!(#value_var => #result::Err(#context_t::invalid_variant_tag(#ctx_var, #type_name, &#value_var))));
                }
            }

            match en.name_type.method {
                NameMethod::Sized => {
                    let decode_t_decode = &b.decode_t_decode;
                    let name_type = &en.name_type.ty;

                    Ok(quote! {{
                        let #value_var: #name_type = #decode_t_decode(#decoder_var)?;

                        match #value_var { #(#arms,)* }
                    }})
                }
                NameMethod::Unsized(method) => {
                    let method = method.as_method_name();
                    let visit_type = &en.name_type.ty;

                    Ok(quote! {
                        #decoder_t::#method(#decoder_var, |#value_var: &#visit_type| {
                            match #value_var { #(#arms,)* }
                        })
                    })
                }
            }
        }
        EnumTagging::Default => {
            let arms = output_arms.iter().flat_map(|(v, pat, tag_value)| {
                let name = &v.st.name;

                let decode = decode_variant(cx, b, v, &body_decoder_var, &variant_tag_var).ok()?;

                let enter = cx.trace.then(|| {
                    let formatted_tag = en.name_type.name_format(&tag_static);
                    let tag_type = en.name_type.ty();

                    quote! {
                        static #tag_static: #tag_type = #tag_value;
                        #context_t::enter_variant(#ctx_var, #name, #formatted_tag);
                    }
                });

                let leave = cx.trace.then(|| quote! {
                    #context_t::leave_variant(#ctx_var);
                });

                Some(quote! {
                    #pat => {
                        #enter

                        let #body_decoder_var = #variant_decoder_t::decode_value(#variant_decoder_var)?;
                        let #output_var = #decode;

                        #leave
                        #output_var
                    }
                })
            });

            let enter = cx.trace.then(|| {
                quote! {
                    #context_t::enter_enum(#ctx_var, #type_name);
                }
            });

            let leave = cx.trace.then(|| {
                quote! {
                    #context_t::leave_enum(#ctx_var);
                }
            });

            Ok(quote! {{
                #output_enum
                #enter

                let #output_var = #decoder_t::decode_variant(#decoder_var, move |#variant_decoder_var| {
                    let #variant_tag_var: #name_type = {
                        let mut #variant_decoder_var = #variant_decoder_t::decode_tag(#variant_decoder_var)?;
                        #decode_name?
                    };

                    let #output_var = match #variant_tag_var {
                        #(#arms,)*
                        #fallback
                    };

                    #result::Ok(#output_var)
                })?;

                #leave
                Ok(#output_var)
            }})
        }
        EnumTagging::Internal {
            tag_value,
            tag_type,
        } => {
            let arms = output_arms.iter().flat_map(|(v, pat, tag_value)| {
                let name = &v.st.name;

                let decode =
                    decode_variant(cx, b, v, &buffer_decoder_var, &variant_tag_var).ok()?;

                let enter = cx.trace.then(|| {
                    let formatted_tag = en.name_type.name_format(&tag_static);
                    let tag_type = en.name_type.ty();

                    quote! {
                        static #tag_static: #tag_type = #tag_value;
                        #context_t::enter_variant(#ctx_var, #name, #formatted_tag);
                    }
                });

                let leave = cx.trace.then(|| {
                    quote! {
                        #context_t::leave_variant(#ctx_var);
                    }
                });

                Some(quote! {
                    #pat => {
                        #enter

                        let #buffer_decoder_var = #as_decoder_t::as_decoder(&#buffer_var)?;
                        let #output_var = #decode;

                        #leave
                        #output_var
                    }
                })
            });

            let outcome_enum;
            let decode_match;

            match tag_type.method {
                NameMethod::Sized => {
                    let decode_t_decode = &b.decode_t_decode;

                    outcome_enum = None;

                    let tag_type = &tag_type.ty;
                    let tag_arm = output_arm(None, tag_value, &binding_var);

                    decode_match = quote! {
                        let #value_var: #tag_type = #decode_t_decode(#field_name_var)?;

                        match #value_var {
                            #tag_arm => {
                                break #struct_field_decoder_t::decode_value(#entry_var)?;
                            }
                            #field_var => {
                                if #skip_field(#entry_var)? {
                                    return #result::Err(#context_t::invalid_field_tag(#ctx_var, #type_name, &#field_var));
                                }
                            }
                        }
                    };
                }
                NameMethod::Unsized(method) => {
                    outcome_enum = Some(quote! {
                        enum #outcome_type<#buf_type> { Tag, Skip(#buf_type) }
                    });

                    let visit_type = &tag_type.ty;
                    let method = method.as_method_name();

                    let tag_arm = output_arm(None, tag_value, &binding_var);

                    let format_value_var = tag_type.name_format(&value_var);

                    let decode_outcome = quote! {
                        #decoder_t::#method(#field_name_var, |#value_var: &#visit_type| {
                            #result::Ok(match #value_var {
                                #tag_arm => #outcome_type::Tag,
                                #value_var => {
                                    #outcome_type::Skip(#context_t::collect_string(#ctx_var, #format_value_var)?)
                                }
                            })
                        })?
                    };

                    decode_match = quote! {{
                        let #field_name_var = #decode_outcome;

                        match #field_name_var {
                            #outcome_type::Tag => {
                                break #struct_field_decoder_t::decode_value(#entry_var)?;
                            }
                            #outcome_type::Skip(#field_name) => {
                                if #skip_field(#entry_var)? {
                                    return #result::Err(#context_t::invalid_field_string_tag(#ctx_var, #type_name, #field_name));
                                }
                            }
                        }
                    }};
                }
            };

            let enter = cx.trace.then(|| {
                quote! {
                    #context_t::enter_enum(#ctx_var, #type_name);
                }
            });

            let leave = cx.trace.then(|| {
                quote! {
                    #context_t::leave_enum(#ctx_var);
                }
            });

            let tag_static_value = tag_type.expr(tag_static.clone());
            let tag_type = tag_type.ty();

            Ok(quote! {{
                static #tag_static: #tag_type = #tag_value;

                #output_enum
                #outcome_enum

                #enter
                let #buffer_var = #decoder_t::decode_buffer(#decoder_var)?;
                let #struct_var = #as_decoder_t::as_decoder(&#buffer_var)?;

                let #variant_tag_var: #name_type = #decoder_t::decode_map(#struct_var, |#struct_var| {
                    let #variant_decoder_var = loop {
                        let #option::Some(mut #entry_var) = #map_decoder_t::decode_entry(#struct_var)? else {
                            return #result::Err(#context_t::missing_variant_field(#ctx_var, #type_name, #tag_static_value));
                        };

                        let #field_name_var = #struct_field_decoder_t::decode_key(&mut #entry_var)?;

                        #decode_match
                    };

                    #decode_name
                })?;

                let #output_var = match #variant_tag_var {
                    #(#arms,)*
                    #fallback
                };

                #leave
                #result::Ok(#output_var)
            }})
        }
        EnumTagging::Adjacent {
            tag_value,
            tag_type,
            content_value,
            content_type,
        } => {
            let arms = output_arms.iter().flat_map(|(v, pat, tag_value)| {
                let name = &v.st.name;

                let decode = decode_variant(cx, b, v, &body_decoder_var, &variant_tag_var).ok()?;

                let enter = cx.trace.then(|| {
                    let formatted_tag = en.name_type.name_format(&tag_static);
                    let tag_type = en.name_type.ty();

                    quote! {
                        static #tag_static: #tag_type = #tag_value;
                        #context_t::enter_variant(#ctx_var, #name, #formatted_tag);
                    }
                });

                let leave = cx.trace.then(|| {
                    quote! {
                        #context_t::leave_variant(#ctx_var);
                    }
                });

                Some(quote! {
                    #pat => {
                        #enter
                        let #output_var = #decode;
                        #leave
                        #output_var
                    }
                })
            });

            let decode_t_decode = &b.decode_t_decode;

            let outcome_enum;
            let decode_match;

            match tag_type.method {
                NameMethod::Sized => {
                    outcome_enum = None;

                    let value_type = &tag_type.ty;
                    let tag_arm = output_arm(None, tag_value, &binding_var);
                    let content_arm = output_arm(None, content_value, &binding_var);

                    decode_match = quote! {
                        let #value_var: #value_type = #decode_t_decode(#field_name_var)?;

                        match #value_var {
                            #tag_arm => {
                                let #variant_decoder_var = #struct_field_decoder_t::decode_value(#entry_var)?;
                                let #variant_tag_var: #name_type = #decode_name?;
                                #name_var = #option::Some(#variant_tag_var);
                            }
                            #content_arm => {
                                let #option::Some(#variant_tag_var) = #name_var else {
                                    return #result::Err(#context_t::missing_adjacent_tag(#ctx_var, #type_name, &#content_value));
                                };

                                let #body_decoder_var = #struct_field_decoder_t::decode_value(#entry_var)?;

                                break #result::Ok(match #variant_tag_var {
                                    #(#arms,)*
                                    #fallback
                                });
                            }
                            #field_var => {
                                if #skip_field(#entry_var)? {
                                    return #result::Err(#context_t::invalid_field_tag(#ctx_var, #type_name, &#field_var));
                                }
                            }
                        }
                    };
                }
                NameMethod::Unsized(method) => {
                    outcome_enum = Some(quote! {
                        enum #outcome_type<#buf_type> { Tag, Content, Skip(#buf_type) }
                    });

                    let visit_type = &tag_type.ty;
                    let format_value_var = tag_type.name_format(&value_var);
                    let method = method.as_method_name();
                    let tag_arm = output_arm(None, tag_value, &binding_var);
                    let content_arm = output_arm(None, content_value, &binding_var);

                    decode_match = quote! {
                        let #outcome_var = #decoder_t::#method(#field_name_var, |#value_var: &#visit_type| {
                            #result::Ok(match #value_var {
                                #tag_arm => #outcome_type::Tag,
                                #content_arm => #outcome_type::Content,
                                #value_var => {
                                    #outcome_type::Skip(#context_t::collect_string(#ctx_var, #format_value_var)?)
                                }
                            })
                        })?;

                        match #outcome_var {
                            #outcome_type::Tag => {
                                let #variant_decoder_var = #struct_field_decoder_t::decode_value(#entry_var)?;
                                #name_var = #option::Some(#decode_name?);
                            }
                            #outcome_type::Content => {
                                let #option::Some(#variant_tag_var) = #name_var else {
                                    return #result::Err(#context_t::invalid_field_tag(#ctx_var, #type_name, &#tag_value));
                                };

                                let #body_decoder_var = #struct_field_decoder_t::decode_value(#entry_var)?;

                                break #result::Ok(match #variant_tag_var {
                                    #(#arms,)*
                                    #fallback
                                });
                            }
                            #outcome_type::Skip(#field_name) => {
                                if #skip_field(#entry_var)? {
                                    return #result::Err(#context_t::invalid_field_string_tag(#ctx_var, #type_name, #field_name));
                                }
                            }
                        }
                    };
                }
            };

            let enter = cx.trace.then(|| {
                quote! {
                    #context_t::enter_enum(#ctx_var, #type_name);
                }
            });

            let leave = cx.trace.then(|| {
                quote! {
                    #context_t::leave_enum(#ctx_var);
                }
            });

            let tag_value_type = tag_type.ty();
            let content_value_type = content_type.ty();

            Ok(quote! {{
                static #tag_static: #tag_value_type = #tag_value;
                static #content_static: #content_value_type = #content_value;

                #output_enum
                #outcome_enum

                static #struct_hint_static: #map_hint = #map_hint::with_size(2);

                #enter

                #decoder_t::decode_map_hint(#decoder_var, &#struct_hint_static, move |#struct_decoder_var| {
                    let mut #name_var = #option::None;

                    let #output_var = loop {
                        let #option::Some(mut #entry_var) = #map_decoder_t::decode_entry(#struct_decoder_var)? else {
                            return #result::Err(#context_t::expected_field_adjacent(#ctx_var, #type_name, &#tag_static, &#content_static));
                        };

                        let #field_name_var = #struct_field_decoder_t::decode_key(&mut #entry_var)?;

                        #decode_match
                    };

                    #leave
                    #result::Ok(#output_var)
                })?
            }})
        }
    }
}

fn decode_variant(
    cx: &Ctxt<'_>,
    b: &Build,
    v: &Variant<'_>,
    decoder_var: &Ident,
    variant_tag: &Ident,
) -> Result<TokenStream, ()> {
    let cx = Ctxt {
        decoder_var,
        trace_body: false,
        ..*cx
    };

    Ok(match (v.st.kind, v.st.packing) {
        (_, Packing::Transparent) => decode_transparent(&cx, b, &v.st)?,
        (_, Packing::Packed) => decode_packed(&cx, b, &v.st)?,
        (StructKind::Empty, _) => decode_empty(&cx, b, &v.st)?,
        (_, Packing::Tagged) => decode_tagged(&cx, b, &v.st, Some(variant_tag))?,
    })
}

/// Decode something empty.
fn decode_empty(cx: &Ctxt, b: &Build<'_, '_>, st: &Body<'_>) -> Result<TokenStream> {
    let Ctxt {
        ctx_var,
        decoder_var,
        ..
    } = *cx;

    let Tokens {
        context_t,
        decoder_t,
        result,
        map_hint,
        ..
    } = b.tokens;

    let Body { path, name, .. } = st;

    let output_var = b.cx.ident("output");
    let struct_hint_static = b.cx.ident("STRUCT_HINT");

    let enter = (cx.trace && cx.trace_body).then(|| {
        quote! {
            #context_t::enter_struct(#ctx_var, #name);
        }
    });

    let leave = (cx.trace && cx.trace_body).then(|| {
        quote! {
            #context_t::leave_struct(#ctx_var);
        }
    });

    Ok(quote! {{
        #enter
        static #struct_hint_static: #map_hint = #map_hint::with_size(0);
        let #output_var = #decoder_t::decode_map_hint(#decoder_var, &#struct_hint_static, |_| #result::Ok(()))?;
        #leave
        #path
    }})
}

/// Decode something tagged.
///
/// If `variant_name` is specified it implies that a tagged enum is being
/// decoded.
fn decode_tagged(
    cx: &Ctxt,
    b: &Build<'_, '_>,
    st: &Body<'_>,
    variant_tag: Option<&Ident>,
) -> Result<TokenStream> {
    let Ctxt {
        ctx_var,
        decoder_var,
        name_var,
        ..
    } = *cx;

    let Tokens {
        context_t,
        decoder_t,
        default_function,
        fmt,
        option,
        result,
        skip_field,
        map_decoder_t,
        struct_field_decoder_t,
        map_hint,
        ..
    } = b.tokens;

    let struct_decoder_var = b.cx.ident("struct_decoder");
    let struct_hint_static = b.cx.ident("STRUCT_HINT");
    let type_decoder_var = b.cx.ident("type_decoder");
    let value_var = b.cx.ident("value");
    let binding_var = b.cx.ident("binding");
    let static_name_var = b.cx.ident("FIELD_NAME");
    let static_name_type = st.name_type.ty();

    let type_name = &st.name;

    let mut assigns = Punctuated::<_, Token![,]>::new();

    let mut fields_with = Vec::new();

    for f in &st.all_fields {
        let tag = &f.name;
        let var = &f.var;
        let decode_path = &f.decode_path.1;

        let expr = match &f.skip {
            Some(span) => {
                let ty = f.ty;

                match &f.default_attr {
                    Some((_, Some(path))) => syn::Expr::Verbatim(quote_spanned!(*span => #path())),
                    _ => syn::Expr::Verbatim(quote_spanned!(*span => #default_function::<#ty>())),
                }
            }
            None => {
                let formatted_tag = st.name_type.name_format(&static_name_var);

                let enter = cx.trace.then(|| {
                    let (name, enter) = match &f.member {
                        syn::Member::Named(name) => (
                            syn::Lit::Str(syn::LitStr::new(&name.to_string(), name.span())),
                            Ident::new("enter_named_field", Span::call_site()),
                        ),
                        syn::Member::Unnamed(index) => (
                            syn::Lit::Int(syn::LitInt::from(Literal::u32_suffixed(index.index))),
                            Ident::new("enter_unnamed_field", Span::call_site()),
                        ),
                    };

                    quote! {
                        #context_t::#enter(#ctx_var, #name, #formatted_tag);
                    }
                });

                let leave = cx.trace.then(|| {
                    quote! {
                        #context_t::leave_field(#ctx_var);
                    }
                });

                let decode = quote! {
                    #var = #option::Some(#decode_path(#struct_decoder_var)?);
                };

                fields_with.push((tag, f, decode, (enter, leave)));

                let fallback = match f.default_attr {
                    Some((span, None)) => quote_spanned!(span => #default_function()),
                    Some((_, Some(path))) => quote!(#path()),
                    None => quote! {{
                        static #static_name_var: #static_name_type = #tag;
                        return #result::Err(#context_t::expected_tag(#ctx_var, #type_name, #formatted_tag))
                    }},
                };

                let var = &f.var;

                syn::Expr::Verbatim(quote! {
                    match #var {
                        #option::Some(#var) => #var,
                        #option::None => #fallback,
                    }
                })
            }
        };

        assigns.push(syn::FieldValue {
            attrs: Vec::new(),
            member: f.member.clone(),
            colon_token: Some(<Token![:]>::default()),
            expr,
        });
    }

    let decode_tag;
    let mut output_enum = quote!();

    let unsupported = match variant_tag {
        Some(variant_tag) => quote! {
            #context_t::invalid_variant_field_tag(#ctx_var, #type_name, &#variant_tag, &#name_var)
        },
        None => quote! {
            #context_t::invalid_field_tag(#ctx_var, #type_name, &#name_var)
        },
    };

    let skip_field = quote! {
        if #skip_field(#struct_decoder_var)? {
            return #result::Err(#unsupported);
        }
    };

    let body;
    let name_type: syn::Type;

    match st.name_type.method {
        NameMethod::Sized => {
            let mut arms = Vec::with_capacity(fields_with.len());

            for (tag, f, decode, (enter, leave)) in fields_with {
                let arm = output_arm(f.pattern, &f.name, &binding_var);

                arms.push(quote! {
                    #arm => {
                        static #static_name_var: #static_name_type = #tag;
                        #enter
                        let #struct_decoder_var = #struct_field_decoder_t::decode_value(#struct_decoder_var)?;
                        #decode
                        #leave
                    }
                });
            }

            body = quote!(match #name_var { #(#arms,)* _ => { #skip_field } });

            let decode_t_decode = &b.decode_t_decode;

            decode_tag = quote! {
                #decode_t_decode(#struct_decoder_var)?
            };

            name_type = st.name_type.ty.clone();
        }
        NameMethod::Unsized(method) => {
            let output_type =
                b.cx.type_with_span("TagVisitorOutput", b.input.ident.span());

            let mut outputs = Vec::with_capacity(fields_with.len());
            let mut name_arms = Vec::with_capacity(fields_with.len());

            for (tag, f, decode, trace) in fields_with {
                let (name_pat, name_variant) =
                    unsized_arm(b, f.span, f.index, &f.name, f.pattern, &output_type);

                outputs.push(name_variant);
                name_arms.push((tag, name_pat, decode, trace));
            }

            if !name_arms.is_empty() {
                let arms = name_arms
                    .into_iter()
                    .map(|(tag, name_pat, decode, (enter, leave))| {
                        quote! {
                            #name_pat => {
                                static #static_name_var: #static_name_type = #tag;
                                #enter
                                let #struct_decoder_var = #struct_field_decoder_t::decode_value(#struct_decoder_var)?;
                                #decode
                                #leave
                            }
                        }
                    });

                body = quote! {
                    match #name_var { #(#arms,)* #name_var => { #skip_field } }
                }
            } else {
                body = skip_field;
            }

            let arms = outputs.iter().map(|o| o.as_arm(&binding_var, option));

            let visit_type = &st.name_type.ty;
            let method = method.as_method_name();

            decode_tag = quote! {
                #decoder_t::#method(#struct_decoder_var, |#value_var: &#visit_type| {
                    #result::Ok(match #value_var {
                        #(#arms,)*
                        #value_var => {
                            #option::None
                        }
                    })
                })?
            };

            let variants = outputs.iter().map(|o| &o.variant);

            let fmt_patterns = outputs.iter().map(|o| {
                let variant = &o.variant;
                let tag = o.name;
                quote!(#output_type::#variant => #fmt::Debug::fmt(&#tag, f))
            });

            output_enum = quote! {
                enum #output_type {
                    #(#variants,)*
                }

                impl #fmt::Debug for #output_type {
                    #[inline]
                    fn fmt(&self, f: &mut #fmt::Formatter<'_>) -> #fmt::Result {
                        match *self { #(#fmt_patterns,)* }
                    }
                }
            };

            name_type = syn::parse_quote!(#option<#output_type>);
        }
    }

    let path = &st.path;
    let fields_len = st.unskipped_fields.len();

    let decls = st
        .unskipped_fields
        .iter()
        .map(|f| &**f)
        .map(|Field { var, ty, .. }| quote!(let mut #var: #option<#ty> = #option::None));

    let enter = (cx.trace && cx.trace_body).then(|| {
        quote! {
            #context_t::enter_struct(#ctx_var, #type_name);
        }
    });

    let leave = (cx.trace && cx.trace_body).then(|| {
        quote! {
            #context_t::leave_struct(#ctx_var);
        }
    });

    Ok(quote! {{
        #output_enum
        #(#decls;)*

        #enter

        static #struct_hint_static: #map_hint = #map_hint::with_size(#fields_len);

        #decoder_t::decode_map_hint(#decoder_var, &#struct_hint_static, move |#type_decoder_var| {
            while let #option::Some(mut #struct_decoder_var) = #map_decoder_t::decode_entry(#type_decoder_var)? {
                let #name_var: #name_type = {
                    let #struct_decoder_var = #struct_field_decoder_t::decode_key(&mut #struct_decoder_var)?;
                    #decode_tag
                };

                #body
            }

            #leave
            #result::Ok(#path { #assigns })
        })?
    }})
}

/// Decode a transparent value.
fn decode_transparent(cx: &Ctxt<'_>, b: &Build<'_, '_>, st: &Body<'_>) -> Result<TokenStream> {
    let Ctxt {
        decoder_var,
        ctx_var,
        ..
    } = *cx;

    let output_var = b.cx.ident("output");

    let Tokens { context_t, .. } = b.tokens;

    let f = &st.unskipped_fields[0];

    let type_name = &st.name;
    let path = &st.path;
    let decode_path = &f.decode_path.1;
    let member = &f.member;

    let enter = (cx.trace && cx.trace_body).then(|| {
        quote! {
            #context_t::enter_struct(#ctx_var, #type_name);
        }
    });

    let leave = (cx.trace && cx.trace_body).then(|| {
        quote! {
            #context_t::leave_struct(#ctx_var);
        }
    });

    Ok(quote! {{
        #enter

        let #output_var = #path {
            #member: #decode_path(#decoder_var)?
        };

        #leave
        #output_var
    }})
}

/// Decode something packed.
fn decode_packed(cx: &Ctxt<'_>, b: &Build<'_, '_>, st_: &Body<'_>) -> Result<TokenStream> {
    let Ctxt {
        decoder_var,
        ctx_var,
        ..
    } = *cx;

    let Tokens {
        context_t,
        decoder_t,
        pack_decoder_t,
        ..
    } = b.tokens;

    let type_name = &st_.name;
    let output_var = b.cx.ident("output");
    let field_decoder = b.cx.ident("field_decoder");

    let mut assign = Vec::new();

    for f in &st_.unskipped_fields {
        if let Some((span, _)) = f.default_attr {
            b.packed_default_diagnostics(span);
        }

        let (_, decode_path) = &f.decode_path;
        let member = &f.member;
        let field_decoder = &field_decoder;

        assign.push(move |ident: &syn::Ident, tokens: &mut TokenStream| {
            tokens.extend(quote! {
                #member: {
                    let #field_decoder = #pack_decoder_t::decode_next(#ident)?;
                    #decode_path(#field_decoder)?
                }
            })
        });
    }

    let enter = (cx.trace && cx.trace_body).then(|| {
        quote! {
            #context_t::enter_struct(#ctx_var, #type_name);
        }
    });

    let leave = (cx.trace && cx.trace_body).then(|| {
        quote! {
            #context_t::leave_struct(#ctx_var);
        }
    });

    let pack = b.cx.ident("pack");
    let assign = apply::iter(assign, &pack);
    let path = &st_.path;

    Ok(quote! {{
        #enter

        let #output_var = #decoder_t::decode_pack(#decoder_var, move |#pack| {
            Ok(#path { #(#assign),* })
        })?;

        #leave
        #output_var
    }})
}

/// Output type used when indirectly encoding a variant or field as type which
/// might require special handling. Like a string.
pub(crate) struct NameVariant<'a> {
    /// The path of the variant this output should generate.
    path: syn::Path,
    /// The identified of the variant this path generates.
    variant: Ident,
    /// The tag this variant corresponds to.
    name: &'a syn::Expr,
    /// The pattern being matched.
    pattern: Option<&'a syn::Pat>,
}

impl NameVariant<'_> {
    /// Generate the pattern for this output.
    pub(crate) fn as_arm(&self, binding_var: &syn::Ident, option: &Import<'_>) -> syn::Arm {
        let path = &self.path;
        let arm = output_arm(self.pattern, self.name, binding_var);

        syn::Arm {
            attrs: Vec::new(),
            pat: arm.pat,
            guard: arm.cond.map(|_| {
                let name = self.name;

                (
                    <syn::Token![if]>::default(),
                    syn::parse_quote!(*#binding_var == #name),
                )
            }),
            fat_arrow_token: <Token![=>]>::default(),
            body: Box::new(syn::parse_quote!(#option::Some(#path))),
            comma: None,
        }
    }
}

fn unsized_arm<'a>(
    b: &Build<'_, '_>,
    span: Span,
    index: usize,
    name: &'a syn::Expr,
    pattern: Option<&'a syn::Pat>,
    output: &Ident,
) -> (syn::Pat, NameVariant<'a>) {
    let variant = b.cx.type_with_span(format_args!("Variant{}", index), span);

    let mut path = syn::Path::from(output.clone());
    path.segments.push(syn::PathSegment::from(variant.clone()));

    let output = NameVariant {
        path: path.clone(),
        variant,
        name,
        pattern,
    };

    let option = &b.tokens.option;
    (syn::parse_quote!(#option::Some(#path)), output)
}

struct Condition<'a> {
    if_: syn::Token![if],
    star: syn::Token![*],
    ident: &'a syn::Ident,
    equals: syn::Token![==],
    expr: &'a syn::Expr,
}

impl ToTokens for Condition<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.if_.to_tokens(tokens);
        self.star.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.equals.to_tokens(tokens);
        self.expr.to_tokens(tokens);
    }
}

fn condition<'a>(ident: &'a syn::Ident, expr: &'a syn::Expr) -> Condition<'a> {
    Condition {
        if_: <syn::Token![if]>::default(),
        star: <syn::Token![*]>::default(),
        ident,
        equals: <syn::Token![==]>::default(),
        expr,
    }
}

fn ref_pattern(ident: &syn::Ident) -> syn::Pat {
    syn::Pat::Ident(syn::PatIdent {
        attrs: Vec::new(),
        by_ref: Some(<syn::Token![ref]>::default()),
        mutability: None,
        ident: ident.clone(),
        subpat: None,
    })
}

fn output_arm<'a>(
    pat: Option<&'a syn::Pat>,
    name: &'a syn::Expr,
    binding: &'a syn::Ident,
) -> OutputArm<'a> {
    if let Some(pat) = pat {
        return OutputArm {
            pat: pat.clone(),
            cond: None,
        };
    }

    if let Some(pat) = expr_to_pat(name) {
        return OutputArm { pat, cond: None };
    }

    OutputArm {
        pat: ref_pattern(binding),
        cond: Some(condition(binding, name)),
    }
}

fn expr_to_pat(expr: &syn::Expr) -> Option<syn::Pat> {
    match expr {
        syn::Expr::Lit(lit) => {
            let pat = syn::Pat::Lit(syn::PatLit {
                attrs: Vec::new(),
                lit: lit.lit.clone(),
            });

            Some(pat)
        }
        syn::Expr::Array(expr) => {
            let mut elems = Punctuated::new();

            for e in &expr.elems {
                elems.push(expr_to_pat(e)?);
            }

            Some(syn::Pat::Slice(syn::PatSlice {
                attrs: Vec::new(),
                bracket_token: expr.bracket_token,
                elems,
            }))
        }
        _ => None,
    }
}

struct OutputArm<'a> {
    pat: syn::Pat,
    cond: Option<Condition<'a>>,
}

impl ToTokens for OutputArm<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.pat.to_tokens(tokens);

        if let Some(cond) = &self.cond {
            cond.to_tokens(tokens);
        }
    }
}
