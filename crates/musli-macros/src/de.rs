use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Token;

use crate::expander::NameMethod;
use crate::internals::apply;
use crate::internals::attr::{EnumTag, EnumTagging, Packing};
use crate::internals::build::{Body, Build, BuildData, Enum, Field, Variant};
use crate::internals::tokens::Tokens;
use crate::internals::Result;

struct Ctxt<'a> {
    ctx_var: &'a Ident,
    decoder_var: &'a Ident,
    tag_var: &'a Ident,
    trace: bool,
    trace_body: bool,
}

pub(crate) fn expand_decode_entry(e: Build<'_>) -> Result<TokenStream> {
    e.validate_decode()?;
    e.cx.reset();

    let ctx_var = e.cx.ident("ctx");
    let root_decoder_var = e.cx.ident("decoder");
    let tag_var = e.cx.ident("tag");
    let d_param = e.cx.type_with_span("D", Span::call_site());

    let cx = Ctxt {
        ctx_var: &ctx_var,
        decoder_var: &root_decoder_var,
        tag_var: &tag_var,
        trace: true,
        trace_body: true,
    };

    let body = match &e.data {
        BuildData::Struct(st) => decode_struct(&cx, &e, st)?,
        BuildData::Enum(en) => decode_enum(&cx, &e, en)?,
    };

    if e.cx.has_errors() {
        return Err(());
    }

    // Figure out which lifetime to use for what. We use the first lifetime in
    // the type (if any is available) as the decoder lifetime. Else we generate
    // a new anonymous lifetime `'de` to use for the `Decode` impl.
    let mut generics = e.input.generics.clone();
    let type_ident = &e.input.ident;

    let (lt, exists) = if let Some(existing) = generics.lifetimes().next() {
        (existing.clone(), true)
    } else {
        let lt = syn::LifetimeParam::new(syn::Lifetime::new("'de", e.input.span()));
        (lt, false)
    };

    if !exists {
        generics.params.push(lt.clone().into());
    }

    let Tokens {
        context_t,
        result,
        decode_t,
        decoder_t,
        ..
    } = e.tokens;

    let (mut generics, mode_ident) = e.expansion.as_impl_generics(generics, e.tokens);

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

    Ok(quote! {
        const _: () = {
            #[automatically_derived]
            #[allow(clippy::init_numbered_fields)]
            #[allow(clippy::let_unit_value)]
            impl #impl_generics #decode_t<#lt, #mode_ident> for #type_ident #type_generics #where_clause {
                #[inline]
                fn decode<#d_param>(#ctx_var: &#d_param::Cx, #root_decoder_var: #d_param) -> #result<Self, <#d_param::Cx as #context_t>::Error>
                where
                    #d_param: #decoder_t<#lt, Mode = #mode_ident>,
                {
                    #body
                }
            }
        };
    })
}

fn decode_struct(cx: &Ctxt<'_>, e: &Build<'_>, st: &Body<'_>) -> Result<TokenStream> {
    let Tokens { result_ok, .. } = e.tokens;

    let body = match st.packing {
        Packing::Tagged => decode_tagged(cx, e, st, None)?,
        Packing::Packed => decode_packed(cx, e, st)?,
        Packing::Transparent => decode_transparent(cx, e, st)?,
    };

    Ok(quote!(#result_ok({ #body })))
}

fn decode_enum(cx: &Ctxt<'_>, e: &Build<'_>, en: &Enum) -> Result<TokenStream> {
    let Ctxt {
        ctx_var,
        tag_var,
        decoder_var,
        ..
    } = *cx;

    let Tokens {
        as_decoder_t,
        context_t,
        decoder_t,
        fmt,
        option_none,
        option_some,
        option,
        priv_write,
        result_err,
        result_ok,
        skip_field,
        skip,
        struct_decoder_t,
        struct_field_decoder_t,
        struct_hint,
        unsized_struct_hint,
        variant_decoder_t,
        ..
    } = e.tokens;

    if let Some(&(span, Packing::Packed)) = en.packing_span {
        e.decode_packed_enum_diagnostics(span);
        return Err(());
    }

    let type_name = &en.name;

    // Trying to decode an uninhabitable type.
    if en.variants.is_empty() {
        return Ok(quote!(#result_err(#context_t::uninhabitable(#ctx_var, #type_name))));
    }

    let body_decoder_var = e.cx.ident("body_decoder");
    let buffer_decoder_var = e.cx.ident("buffer_decoder");
    let buffer_var = e.cx.ident("buffer");
    let struct_var = e.cx.ident("st");
    let entry_var = e.cx.ident("entry");
    let field_alloc_var = e.cx.ident("field_alloc");
    let field_name_var = e.cx.ident("field_name");
    let field_var = e.cx.ident("field");
    let outcome_type = e.cx.type_with_span("Outcome", Span::call_site());
    let output_var = e.cx.ident("output");
    let struct_decoder_var = e.cx.ident("struct_decoder");
    let struct_hint_static = e.cx.ident("STRUCT_HINT");
    let variant_decoder_var = e.cx.ident("variant_decoder");
    let variant_output_var = e.cx.ident("variant_output");
    let variant_tag_var = e.cx.ident("variant_tag");
    let value_var = e.cx.ident("value");

    let mut variant_output_tags = Vec::new();

    let mut fallback = match en.fallback {
        Some(ident) => {
            quote! {{
                if #skip(#variant_decoder_t::decode_value(#variant_decoder_var)?)? {
                    return #result_err(#context_t::invalid_variant_tag(#ctx_var, #type_name, &#variant_tag_var));
                }

                Self::#ident {}
            }}
        }
        None => quote! {
            return #result_err(#context_t::invalid_variant_tag(#ctx_var, #type_name, &#variant_tag_var))
        },
    };

    let decode_tag;
    let output_enum;
    let name_type;

    match en.name_method {
        NameMethod::Value => {
            for v in &en.variants {
                variant_output_tags.push((v, v.name.clone(), v.name.clone()));
            }

            let decode_t_decode = &e.decode_t_decode;

            decode_tag = quote!(#decode_t_decode(#ctx_var, #variant_decoder_var)?);
            output_enum = None;
            fallback = quote!(_ => #fallback);
            name_type = en.name_type.clone();
        }
        NameMethod::Unsized(method) => {
            let mut tag_variants = Vec::new();
            let output_type = e.cx.type_with_span("VariantTag", en.span);

            for v in &en.variants {
                let (tag_pattern, tag_value, tag_variant) =
                    build_tag_variant(e, v.span, v.index, &v.name, &output_type);

                tag_variants.push(tag_variant);
                variant_output_tags.push((v, tag_pattern, tag_value));
            }

            let arms = tag_variants.iter().map(|o| o.as_arm(option_some));

            let visit_type = &en.name_type;
            let method = method.as_method_name();

            decode_tag = quote! {
                #decoder_t::#method(#variant_decoder_var, |#value_var: &#visit_type| {
                    #result_ok(match #value_var {
                        #(#arms,)*
                        _ => #option_none,
                    })
                })?
            };

            let variants = tag_variants.iter().map(|o| &o.variant);

            let fmt_patterns = tag_variants.iter().map(|o| {
                let variant = &o.variant;
                let tag = o.tag;
                quote!(#output_type::#variant => #fmt::Debug::fmt(&#tag, f))
            });

            let fmt_patterns2 = tag_variants.iter().map(|o| {
                let variant = &o.variant;
                let tag = o.tag;
                quote!(#output_type::#variant => #fmt::Display::fmt(&#tag, f))
            });

            output_enum = Some(quote! {
                enum #output_type {
                    #(#variants,)*
                }

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

            fallback = quote!(#option_none => { #fallback });
            name_type = syn::parse_quote!(#option<#output_type>);
        }
    }

    let Some(enum_tagging) = en.enum_tagging else {
        let patterns = variant_output_tags.iter().flat_map(|(v, tag_pattern, tag_value)| {
            let name = &v.st.name;

            let formatted_tag = en.name_format(tag_value);
            let decode = decode_variant(cx, e, v, &body_decoder_var, &variant_tag_var).ok()?;

            let enter = cx.trace.then(|| quote!{
                #context_t::enter_variant(#ctx_var, #name, #formatted_tag);
            });

            let leave = cx.trace.then(|| quote! {
                #context_t::leave_variant(#ctx_var);
            });

            Some(quote! {
                #tag_pattern => {
                    #enter

                    let #output_var = {
                        let #body_decoder_var = #variant_decoder_t::decode_value(#variant_decoder_var)?;
                        #decode
                    };

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

        return Ok(quote! {{
            #output_enum
            #enter

            let #output_var = #decoder_t::decode_variant(#decoder_var, move |#variant_decoder_var| {
                let #variant_tag_var: #name_type = {
                    let mut #variant_decoder_var = #variant_decoder_t::decode_tag(#variant_decoder_var)?;
                    #decode_tag
                };

                let #output_var = match #variant_tag_var {
                    #(#patterns,)*
                    #fallback
                };

                #result_ok(#output_var)
            })?;

            #leave
            Ok(#output_var)
        }});
    };

    match enum_tagging {
        EnumTagging::Internal {
            tag: EnumTag { value: field_tag },
        } => {
            let patterns = variant_output_tags
                .iter()
                .flat_map(|(v, tag_pattern, tag_value)| {
                    let name = &v.st.name;

                    let formatted_tag = en.name_format(tag_value);
                    let decode =
                        decode_variant(cx, e, v, &buffer_decoder_var, &variant_tag_var).ok()?;

                    let enter = cx.trace.then(|| {
                        quote! {
                            #context_t::enter_variant(#ctx_var, #name, #formatted_tag);
                        }
                    });

                    let leave = cx.trace.then(|| {
                        quote! {
                            #context_t::leave_variant(#ctx_var);
                        }
                    });

                    Some(quote! {
                        #tag_pattern => {
                            #enter
                            let #buffer_decoder_var = #as_decoder_t::as_decoder(&#buffer_var)?;
                            let #variant_output_var = #decode;
                            #leave
                            #variant_output_var
                        }
                    })
                });

            let field_alloc;
            let outcome_enum;
            let decode_match;

            match en.name_method {
                NameMethod::Value => {
                    let decode_t_decode = &e.decode_t_decode;

                    outcome_enum = None;
                    field_alloc = None;

                    decode_match = quote! {
                        match #decode_t_decode(#ctx_var, #field_name_var)? {
                            #field_tag => {
                                break #struct_field_decoder_t::decode_field_value(#entry_var)?;
                            }
                            #field_var => {
                                if #skip_field(#entry_var)? {
                                    return #result_err(#context_t::invalid_field_tag(#ctx_var, #type_name, &#field_var));
                                }
                            }
                        }
                    };
                }
                NameMethod::Unsized(method) => {
                    outcome_enum = Some(quote! {
                        enum #outcome_type { Tag, Skip }
                    });

                    field_alloc = Some(quote! {
                        let #option_some(mut #field_alloc_var) = #context_t::alloc(#ctx_var) else {
                            return #result_err(#context_t::alloc_failed(#ctx_var));
                        };
                    });

                    let visit_type = &en.name_type;
                    let method = method.as_method_name();

                    let decode_outcome = quote! {
                        #decoder_t::#method(#field_name_var, |#value_var: &#visit_type| {
                            #result_ok(match #value_var {
                                #field_tag => #outcome_type::Tag,
                                #value_var => {
                                    if #priv_write(&mut #field_alloc_var, #value_var).is_err() {
                                        return #result_err(#context_t::alloc_failed(#ctx_var));
                                    }

                                    #outcome_type::Skip
                                }
                            })
                        })?
                    };

                    decode_match = quote! {{
                        let #field_name_var = #decode_outcome;

                        match #field_name_var {
                            #outcome_type::Tag => {
                                break #struct_field_decoder_t::decode_field_value(#entry_var)?;
                            }
                            #outcome_type::Skip => {
                                if #skip_field(#entry_var)? {
                                    return #result_err(#context_t::invalid_field_string_tag(#ctx_var, #type_name, #field_alloc_var));
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

            Ok(quote! {{
                #output_enum
                #outcome_enum

                #enter
                let #buffer_var = #decoder_t::decode_buffer(#decoder_var)?;
                let #struct_var = #as_decoder_t::as_decoder(&#buffer_var)?;

                static #struct_hint_static: #unsized_struct_hint = #unsized_struct_hint::new();

                let #variant_tag_var = #decoder_t::decode_unsized_struct(#struct_var, &#struct_hint_static, |#struct_var| {
                    let #variant_tag_var = {
                        let #variant_decoder_var = loop {
                            let #option_some(mut #entry_var) = #struct_decoder_t::decode_field(#struct_var)? else {
                                return #result_err(#context_t::missing_variant_field(#ctx_var, #type_name, &#field_tag));
                            };

                            let #field_name_var = #struct_field_decoder_t::decode_field_name(&mut #entry_var)?;

                            #field_alloc
                            #decode_match
                        };

                        #decode_tag
                    };

                    #result_ok(#variant_tag_var)
                })?;

                let #output_var = match #variant_tag_var {
                    #(#patterns,)*
                    #fallback
                };

                #leave
                #result_ok(#output_var)
            }})
        }
        EnumTagging::Adjacent {
            tag: EnumTag { value: tag },
            content,
        } => {
            let patterns = variant_output_tags
                .iter()
                .flat_map(|(v, tag_pattern, tag_value)| {
                    let name = &v.st.name;

                    let formatted_tag = en.name_format(tag_value);
                    let decode =
                        decode_variant(cx, e, v, &body_decoder_var, &variant_tag_var).ok()?;

                    let enter = cx.trace.then(|| {
                        quote! {
                            #context_t::enter_variant(#ctx_var, #name, #formatted_tag);
                        }
                    });

                    let leave = cx.trace.then(|| {
                        quote! {
                            #context_t::leave_variant(#ctx_var);
                        }
                    });

                    Some(quote! {
                        #tag_pattern => {
                            #enter
                            let #variant_output_var = #decode;
                            #leave
                            #variant_output_var
                        }
                    })
                });

            let decode_t_decode = &e.decode_t_decode;

            let field_alloc;
            let mut outcome_enum = quote!();
            let decode_match;

            match en.name_method {
                NameMethod::Value => {
                    field_alloc = None;

                    decode_match = quote! {
                        match #decode_t_decode(#ctx_var, decoder)? {
                            #tag => {
                                let #variant_decoder_var = #struct_field_decoder_t::decode_field_value(#entry_var)?;
                                #tag_var = #option_some(#decode_tag);
                            }
                            #content => {
                                let #option_some(#variant_tag_var) = #tag_var else {
                                    return #result_err(#context_t::invalid_field_tag(#ctx_var, #type_name, &#tag));
                                };

                                let #body_decoder_var = #struct_field_decoder_t::decode_field_value(#entry_var)?;

                                break #result_ok(match #variant_tag_var {
                                    #(#patterns,)*
                                    #fallback
                                });
                            }
                            #field_var => {
                                if #skip_field(#entry_var)? {
                                    return #result_err(#context_t::invalid_field_tag(#ctx_var, #type_name, &#field_var));
                                }
                            }
                        }
                    };
                }
                NameMethod::Unsized(method) => {
                    outcome_enum = quote! {
                        enum #outcome_type { Tag, Content, Skip }
                    };

                    field_alloc = Some(quote! {
                        let #option_some(mut #field_alloc_var) = #context_t::alloc(#ctx_var) else {
                            return #result_err(#context_t::alloc_failed(#ctx_var));
                        };
                    });

                    let visit_type = &en.name_type;
                    let method = method.as_method_name();

                    decode_match = quote! {
                        let outcome = #decoder_t::#method(#field_name_var, |#value_var: &#visit_type| {
                            #result_ok(match #value_var {
                                #tag => #outcome_type::Tag,
                                #content => #outcome_type::Content,
                                #value_var => {
                                    if #priv_write(&mut #field_alloc_var, #value_var).is_err() {
                                        return #result_err(#context_t::alloc_failed(#ctx_var));
                                    }

                                    #outcome_type::Skip
                                }
                            })
                        })?;

                        match outcome {
                            #outcome_type::Tag => {
                                let #variant_decoder_var = #struct_field_decoder_t::decode_field_value(#entry_var)?;
                                #tag_var = #option_some(#decode_tag);
                            }
                            #outcome_type::Content => {
                                let #option_some(#variant_tag_var) = #tag_var else {
                                    return #result_err(#context_t::invalid_field_tag(#ctx_var, #type_name, &#tag));
                                };

                                let #body_decoder_var = #struct_field_decoder_t::decode_field_value(#entry_var)?;

                                break #result_ok(match #variant_tag_var {
                                    #(#patterns,)*
                                    #fallback
                                });
                            }
                            #outcome_type::Skip => {
                                if #skip_field(#entry_var)? {
                                    return #result_err(#context_t::invalid_field_string_tag(#ctx_var, #type_name, #field_alloc_var));
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

            Ok(quote! {{
                #output_enum
                #outcome_enum

                static #struct_hint_static: #struct_hint = #struct_hint::with_size(2);

                #enter

                #decoder_t::decode_struct(#decoder_var, &#struct_hint_static, move |#struct_decoder_var| {
                    let mut #tag_var = #option_none;

                    let #output_var = loop {
                        let #option_some(mut #entry_var) = #struct_decoder_t::decode_field(#struct_decoder_var)? else {
                            return #result_err(#context_t::invalid_field_tag(#ctx_var, #type_name, "other"));
                        };

                        let #field_name_var = #struct_field_decoder_t::decode_field_name(&mut #entry_var)?;

                        #field_alloc
                        #decode_match
                    };

                    #leave
                    #result_ok(#output_var)
                })?
            }})
        }
    }
}

fn decode_variant(
    cx: &Ctxt<'_>,
    e: &Build,
    v: &Variant<'_>,
    decoder_var: &Ident,
    variant_tag: &Ident,
) -> Result<TokenStream, ()> {
    let cx = Ctxt {
        decoder_var,
        trace_body: false,
        ..*cx
    };

    Ok(match v.st.packing {
        Packing::Tagged => decode_tagged(&cx, e, &v.st, Some(variant_tag))?,
        Packing::Packed => decode_packed(&cx, e, &v.st)?,
        Packing::Transparent => decode_transparent(&cx, e, &v.st)?,
    })
}

/// Decode something tagged.
///
/// If `variant_name` is specified it implies that a tagged enum is being
/// decoded.
fn decode_tagged(
    cx: &Ctxt,
    e: &Build<'_>,
    st: &Body<'_>,
    variant_tag: Option<&Ident>,
) -> Result<TokenStream> {
    let Ctxt {
        ctx_var,
        decoder_var,
        tag_var,
        ..
    } = *cx;

    let Tokens {
        context_t,
        decoder_t,
        default_function,
        fmt,
        option_none,
        option_some,
        option,
        priv_write,
        result_err,
        result_ok,
        skip_field,
        struct_decoder_t,
        struct_field_decoder_t,
        struct_hint,
        ..
    } = e.tokens;

    let field_alloc_var = e.cx.ident("field_alloc");
    let leave_label = e.cx.lifetime("leave");
    let struct_decoder_var = e.cx.ident("struct_decoder");
    let struct_hint_static = e.cx.ident("STRUCT_HINT");
    let type_decoder_var = e.cx.ident("type_decoder");
    let value_var = e.cx.ident("value");

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
                let formatted_tag = match &st.name_format_with {
                    Some((_, path)) => quote!(&#path(&#tag)),
                    None => quote!(&#tag),
                };

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
                    #var = #option_some(#decode_path(#ctx_var, #struct_decoder_var)?);
                };

                fields_with.push((f, decode, (enter, leave)));

                let fallback = match f.default_attr {
                    Some((span, None)) => quote_spanned!(span => #default_function()),
                    Some((_, Some(path))) => quote!(#path()),
                    None => quote! {
                        return #result_err(#context_t::expected_tag(#ctx_var, #type_name, &#tag))
                    },
                };

                let var = &f.var;

                syn::Expr::Verbatim(quote! {
                    match #var {
                        #option_some(#var) => #var,
                        #option_none => #fallback,
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

    let field_alloc;
    let decode_tag;
    let mut output_enum = quote!();

    let unsupported = match variant_tag {
        Some(variant_tag) => quote! {
            #context_t::invalid_variant_field_tag(#ctx_var, #type_name, &#variant_tag, &#tag_var)
        },
        None => quote! {
            #context_t::invalid_field_tag(#ctx_var, #type_name, &#tag_var)
        },
    };

    let skip_field = quote! {
        if #skip_field(#struct_decoder_var)? {
            return #result_err(#unsupported);
        }
    };

    let body;
    let name_type: syn::Type;

    match st.name_method {
        NameMethod::Value => {
            let mut statements = Vec::with_capacity(fields_with.len());

            for (f, decode, (enter, leave)) in fields_with {
                let tag = &f.name;

                statements.push(quote! {
                    if #tag_var == #tag {
                        #enter
                        let #struct_decoder_var = #struct_field_decoder_t::decode_field_value(#struct_decoder_var)?;
                        #decode
                        #leave
                        break #leave_label;
                    }
                });
            }

            body = quote! {{
                #leave_label: {
                    #(#statements)*
                    #skip_field
                }
            }};

            field_alloc = None;

            let decode_t_decode = &e.decode_t_decode;

            decode_tag = quote! {
                #decode_t_decode(#ctx_var, #struct_decoder_var)?
            };

            name_type = st.name_type.clone();
        }
        NameMethod::Unsized(method) => {
            let mut outputs = Vec::new();
            let output_type =
                e.cx.type_with_span("TagVisitorOutput", e.input.ident.span());

            let mut patterns = Vec::with_capacity(fields_with.len());

            for (f, decode, trace) in fields_with {
                let (output_pattern, _, output) =
                    build_tag_variant(e, f.span, f.index, &f.name, &output_type);

                outputs.push(output);
                patterns.push((output_pattern, decode, trace));
            }

            if !patterns.is_empty() {
                let patterns = patterns
                    .into_iter()
                    .map(|(pattern_var, decode, (enter, leave))| {
                        quote! {
                            #pattern_var => {
                                #enter
                                let #struct_decoder_var = #struct_field_decoder_t::decode_field_value(#struct_decoder_var)?;
                                #decode
                                #leave
                            }
                        }
                    });

                body = quote! {
                    match #tag_var { #(#patterns,)* #tag_var => { #skip_field } }
                }
            } else {
                body = skip_field;
            }

            let patterns = outputs.iter().map(|o| o.as_arm(option_some));

            field_alloc = Some(quote! {
                let #option_some(mut #field_alloc_var) = #context_t::alloc(#ctx_var) else {
                    return #result_err(#context_t::alloc_failed(#ctx_var));
                };
            });

            let visit_type = &st.name_type;
            let method = method.as_method_name();

            decode_tag = quote! {
                #decoder_t::#method(#struct_decoder_var, |#value_var: &#visit_type| {
                    #result_ok(match #value_var {
                        #(#patterns,)*
                        #value_var => {
                            if #priv_write(&mut #field_alloc_var, #value_var).is_err() {
                                return #result_err(#context_t::alloc_failed(#ctx_var));
                            }

                            #option_none
                        }
                    })
                })?
            };

            let variants = outputs.iter().map(|o| &o.variant);

            let fmt_patterns = outputs.iter().map(|o| {
                let variant = &o.variant;
                let tag = o.tag;
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
        .map(|Field { var, ty, .. }| quote!(let mut #var: #option<#ty> = #option_none;));

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
        #(#decls)*

        #enter

        static #struct_hint_static: #struct_hint = #struct_hint::with_size(#fields_len);

        #decoder_t::decode_struct(#decoder_var, &#struct_hint_static, move |#type_decoder_var| {
            while let #option_some(mut #struct_decoder_var) = #struct_decoder_t::decode_field(#type_decoder_var)? {
                #field_alloc

                let #tag_var: #name_type = {
                    let #struct_decoder_var = #struct_field_decoder_t::decode_field_name(&mut #struct_decoder_var)?;
                    #decode_tag
                };

                #body
            }

            #leave
            #result_ok(#path { #assigns })
        })?
    }})
}

/// Decode a transparent value.
fn decode_transparent(cx: &Ctxt<'_>, e: &Build<'_>, st_: &Body<'_>) -> Result<TokenStream> {
    let Ctxt {
        decoder_var,
        ctx_var,
        ..
    } = *cx;

    let [f] = &st_.unskipped_fields[..] else {
        e.transparent_diagnostics(st_.span, &st_.unskipped_fields);
        return Err(());
    };

    let output_var = e.cx.ident("output");

    let Tokens { context_t, .. } = e.tokens;

    let type_name = &st_.name;
    let path = &st_.path;
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

    Ok(quote! {
        #enter

        let #output_var = #path {
            #member: #decode_path(#ctx_var, #decoder_var)?
        };

        #leave
        #output_var
    })
}

/// Decode something packed.
fn decode_packed(cx: &Ctxt<'_>, e: &Build<'_>, st_: &Body<'_>) -> Result<TokenStream> {
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
    } = e.tokens;

    let type_name = &st_.name;
    let output_var = e.cx.ident("output");
    let field_decoder = e.cx.ident("field_decoder");

    let mut assign = Vec::new();

    for f in &st_.unskipped_fields {
        if let Some((span, _)) = f.default_attr {
            e.packed_default_diagnostics(span);
        }

        let (_, decode_path) = &f.decode_path;
        let member = &f.member;
        let field_decoder = &field_decoder;

        assign.push(move |ident: &syn::Ident, tokens: &mut TokenStream| {
            tokens.extend(quote! {
                #member: {
                    let #field_decoder = #pack_decoder_t::decode_next(#ident)?;
                    #decode_path(#ctx_var, #field_decoder)?
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

    let pack = e.cx.ident("pack");
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
pub(crate) struct TagVariant<'a> {
    /// The path of the variant this output should generate.
    path: syn::Path,
    /// The identified of the variant this path generates.
    variant: Ident,
    /// The tag this variant corresponds to.
    tag: &'a syn::Expr,
}

impl TagVariant<'_> {
    /// Generate the pattern for this output.
    pub(crate) fn as_arm(&self, option_some: &syn::Path) -> syn::Arm {
        let body = syn::Expr::Path(syn::ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: self.path.clone(),
        });

        syn::Arm {
            attrs: Vec::new(),
            pat: syn::Pat::Verbatim(self.tag.to_token_stream()),
            guard: None,
            fat_arrow_token: <Token![=>]>::default(),
            body: Box::new(build_call(option_some, [body])),
            comma: None,
        }
    }
}

pub(crate) fn build_call<A>(path: &syn::Path, it: A) -> syn::Expr
where
    A: IntoIterator<Item = syn::Expr>,
{
    let mut args = Punctuated::default();

    for arg in it {
        args.push(arg);
    }

    syn::Expr::Call(syn::ExprCall {
        attrs: Vec::new(),
        func: Box::new(syn::Expr::Path(syn::ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: path.clone(),
        })),
        paren_token: syn::token::Paren::default(),
        args,
    })
}

pub(crate) fn build_reference(expr: syn::Expr) -> syn::Expr {
    syn::Expr::Reference(syn::ExprReference {
        attrs: Vec::new(),
        and_token: <Token![&]>::default(),
        mutability: None,
        expr: Box::new(expr),
    })
}

fn build_tag_variant<'a>(
    e: &Build<'_>,
    span: Span,
    index: usize,
    tag: &'a syn::Expr,
    output: &Ident,
) -> (syn::Expr, syn::Expr, TagVariant<'a>) {
    let variant = e.cx.type_with_span(format_args!("Variant{}", index), span);

    let mut path = syn::Path::from(output.clone());
    path.segments.push(syn::PathSegment::from(variant.clone()));

    let output = TagVariant {
        path: path.clone(),
        variant,
        tag,
    };

    let expr = syn::Expr::Path(syn::ExprPath {
        attrs: Vec::new(),
        qself: None,
        path,
    });

    (
        build_call(&e.tokens.option_some, [expr.clone()]),
        expr,
        output,
    )
}
