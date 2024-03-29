use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Token;

use crate::expander::{Result, TagMethod};
use crate::internals::apply;
use crate::internals::attr::{EnumTag, EnumTagging, Packing};
use crate::internals::build::{Body, Build, BuildData, Enum, Field, Variant};
use crate::internals::tokens::Tokens;

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
        buf_t,
        context_t,
        decoder_t,
        fmt,
        option_none,
        option_some,
        result_err,
        result_ok,
        skip,
        str_ty,
        struct_decoder_t,
        struct_field_decoder_t,
        variant_decoder_t,
        visit_owned_fn,
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
    let entry_var = e.cx.ident("entry");
    let output_var = e.cx.ident("output");
    let struct_decoder_var = e.cx.ident("struct_decoder");
    let variant_decoder_var = e.cx.ident("variant_decoder");
    let variant_tag_var = e.cx.ident("variant_tag");
    let variant_output_var = e.cx.ident("variant_output");
    let variant_alloc_var = e.cx.ident("variant_alloc");
    let field_alloc_var = e.cx.ident("field_alloc");

    let mut variant_output_tags = Vec::new();

    let mut fallback = match en.fallback {
        Some(ident) => {
            quote! {{
                if #skip::is_unsupported(#decoder_t::try_skip(#variant_decoder_t::decode_value(#variant_decoder_var)?)?) {
                    return #result_err(#context_t::invalid_variant_tag(#ctx_var, #type_name, #variant_tag_var));
                }

                Self::#ident {}
            }}
        }
        None => quote! {
            return #result_err(#context_t::invalid_variant_tag(#ctx_var, #type_name, #variant_tag_var))
        },
    };

    let variant_alloc;
    let decode_tag;
    let mut output_enum = quote!();

    match en.variant_tag_method {
        TagMethod::String => {
            let mut tag_variants = Vec::new();
            let output = e.cx.type_with_span("VariantTagVisitorOutput", en.span);

            for v in en.variants.iter().filter(|v| !v.is_default) {
                let (tag_pattern, tag_value, tag_variant) =
                    build_tag_variant(e, v.span, v.index, &v.tag, &output);

                tag_variants.push(tag_variant);
                variant_output_tags.push((v, tag_pattern, tag_value));
            }

            let arms = tag_variants.iter().map(|o| o.as_arm(option_some));

            variant_alloc = Some(quote! {
                let #option_some(mut #variant_alloc_var) = #context_t::alloc(#ctx_var) else {
                    return #result_err(#context_t::alloc_failed(#ctx_var));
                };
            });

            decode_tag = quote! {
                #decoder_t::decode_string(#variant_decoder_var, #visit_owned_fn("a string variant tag", |string: &str| {
                    #result_ok(match string {
                        #(#arms,)*
                        string => {
                            if !#buf_t::write(&mut #variant_alloc_var, #str_ty::as_bytes(string)) {
                                return #result_err(#context_t::alloc_failed(#ctx_var));
                            }

                            #option_none
                        }
                    })
                }))?
            };

            let variants = tag_variants.iter().map(|o| &o.variant);

            let fmt_patterns = tag_variants.iter().map(|o| {
                let variant = &o.variant;
                let tag = o.tag;
                quote!(#output::#variant => #fmt::Debug::fmt(&#tag, f))
            });

            let fmt_patterns2 = tag_variants.iter().map(|o| {
                let variant = &o.variant;
                let tag = o.tag;
                quote!(#output::#variant => #fmt::Display::fmt(&#tag, f))
            });

            output_enum = quote! {
                enum #output {
                    #(#variants,)*
                }

                impl #fmt::Debug for #output {
                    #[inline]
                    fn fmt(&self, f: &mut #fmt::Formatter<'_>) -> #fmt::Result {
                        match self { #(#fmt_patterns,)* }
                    }
                }

                impl #fmt::Display for #output {
                    #[inline]
                    fn fmt(&self, f: &mut #fmt::Formatter<'_>) -> #fmt::Result {
                        match self { #(#fmt_patterns2,)* }
                    }
                }
            };

            fallback = quote! {
                #option_none => {
                    #fallback
                }
            };
        }
        TagMethod::Any => {
            for v in en.variants.iter().filter(|v| !v.is_default) {
                variant_output_tags.push((v, v.tag.clone(), v.tag.clone()));
            }

            let decode_t_decode = &e.decode_t_decode;

            variant_alloc = None;
            decode_tag = quote!(#decode_t_decode(#ctx_var, #variant_decoder_var)?);
            fallback = quote!(_ => #fallback);
        }
    }

    let name_type = en.name_type.as_ref().map(|(_, ty)| quote!(: #ty));

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

            let #output_var = #decoder_t::decode_variant(#decoder_var, |#variant_decoder_var| {
                #variant_alloc

                let #variant_tag_var #name_type = {
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
            tag:
                EnumTag {
                    value: field_tag,
                    method: field_tag_method,
                },
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

            let decode_t_decode = &e.decode_t_decode;

            let field_alloc;
            let mut outcome_enum = quote!();
            let decode_match;

            match field_tag_method {
                Some(TagMethod::String) => {
                    outcome_enum = quote! {
                        enum Outcome {
                            Tag,
                            AllocErr,
                            Err,
                        }
                    };

                    field_alloc = Some(quote! {
                        let #option_some(mut #field_alloc_var) = #context_t::alloc(#ctx_var) else {
                            return #result_err(#context_t::alloc_failed(#ctx_var));
                        };
                    });

                    let decode_outcome = quote! {
                        #decoder_t::decode_string(decoder, #visit_owned_fn("a string field tag", |string: &str| {
                            #result_ok(match string {
                                #field_tag => Outcome::Tag,
                                string => {
                                    if !#buf_t::write(&mut #field_alloc_var, #str_ty::as_bytes(string)) {
                                        Outcome::AllocErr
                                    } else {
                                        Outcome::Err
                                    }
                                }
                            })
                        }))?
                    };

                    decode_match = quote! {
                        match #decode_outcome {
                            Outcome::Tag => {
                                break #struct_field_decoder_t::decode_field_value(#entry_var)?;
                            }
                            Outcome::AllocErr => {
                                return #result_err(#context_t::alloc_failed(#ctx_var));
                            }
                            Outcome::Err => {
                                if #skip::is_unsupported(#decoder_t::try_skip(#struct_field_decoder_t::decode_field_value(#entry_var)?)?) {
                                    return #result_err(#context_t::invalid_field_string_tag(#ctx_var, #type_name, #field_alloc_var));
                                }
                            }
                        }
                    };
                }
                _ => {
                    field_alloc = None;

                    decode_match = quote! {
                        match #decode_t_decode(#ctx_var, decoder)? {
                            #field_tag => {
                                break #struct_field_decoder_t::decode_field_value(#entry_var)?;
                            }
                            field => {
                                if #skip::is_unsupported(#decoder_t::try_skip(#struct_field_decoder_t::decode_field_value(#entry_var)?)?) {
                                    return #result_err(#context_t::invalid_field_tag(#ctx_var, #type_name, field));
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

                #enter
                let #buffer_var = #decoder_t::decode_buffer(#decoder_var)?;
                let st = #as_decoder_t::as_decoder(&#buffer_var)?;

                let #variant_tag_var = #decoder_t::decode_struct(st, None, |st| {
                    let #variant_tag_var #name_type = {
                        let #variant_decoder_var = loop {
                            let #option_some(mut #entry_var) = #struct_decoder_t::decode_field(st)? else {
                                return #result_err(#context_t::missing_variant_field(#ctx_var, #type_name, #field_tag));
                            };

                            let decoder = #struct_field_decoder_t::decode_field_name(&mut #entry_var)?;

                            #field_alloc
                            #decode_match
                        };

                        #variant_alloc
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
            tag:
                EnumTag {
                    value: tag,
                    method: tag_method,
                    ..
                },
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

            match tag_method {
                Some(TagMethod::String) => {
                    outcome_enum = quote! {
                        enum Outcome { Tag, Content, AllocErr, Err }
                    };

                    field_alloc = Some(quote! {
                        let #option_some(mut #field_alloc_var) = #context_t::alloc(#ctx_var) else {
                            return #result_err(#context_t::alloc_failed(#ctx_var));
                        };
                    });

                    decode_match = quote! {
                        let outcome = #decoder_t::decode_string(decoder, #visit_owned_fn("a string field tag", |string: &str| {
                            #result_ok(match string {
                                #tag => Outcome::Tag,
                                #content => Outcome::Content,
                                string => {
                                    if !#buf_t::write(&mut #field_alloc_var, #str_ty::as_bytes(string)) {
                                        Outcome::AllocErr
                                    } else {
                                        Outcome::Err
                                    }
                                }
                            })
                        }))?;

                        match outcome {
                            Outcome::Tag => {
                                let #variant_decoder_var = #struct_field_decoder_t::decode_field_value(#entry_var)?;
                                #variant_alloc
                                #tag_var = #option_some(#decode_tag);
                            }
                            Outcome::Content => {
                                let #option_some(#variant_tag_var #name_type) = #tag_var else {
                                    return #result_err(#context_t::invalid_field_tag(#ctx_var, #type_name, #tag));
                                };

                                let #body_decoder_var = #struct_field_decoder_t::decode_field_value(#entry_var)?;

                                break #result_ok(match #variant_tag_var {
                                    #(#patterns,)*
                                    #fallback
                                });
                            }
                            Outcome::AllocErr => {
                                return #result_err(#context_t::alloc_failed(#ctx_var));
                            }
                            Outcome::Err => {
                                if #skip::is_unsupported(#decoder_t::try_skip(#struct_field_decoder_t::decode_field_value(#entry_var)?)?) {
                                    return #result_err(#context_t::invalid_field_string_tag(#ctx_var, #type_name, #field_alloc_var));
                                }
                            }
                        }
                    };
                }
                _ => {
                    field_alloc = None;

                    decode_match = quote! {
                        match #decode_t_decode(#ctx_var, decoder)? {
                            #tag => {
                                let #variant_decoder_var = #struct_field_decoder_t::decode_field_value(#entry_var)?;
                                #variant_alloc
                                #tag_var = #option_some(#decode_tag);
                            }
                            #content => {
                                let #option_some(#variant_tag_var #name_type) = #tag_var else {
                                    return #result_err(#context_t::invalid_field_tag(#ctx_var, #type_name, #tag));
                                };

                                let #body_decoder_var = #struct_field_decoder_t::decode_field_value(#entry_var)?;

                                break #result_ok(match #variant_tag_var {
                                    #(#patterns,)*
                                    #fallback
                                });
                            }
                            field => {
                                if #skip::is_unsupported(#decoder_t::try_skip(#struct_field_decoder_t::decode_field_value(#entry_var)?)?) {
                                    return #result_err(#context_t::invalid_field_tag(#ctx_var, #type_name, field));
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

                #enter

                #decoder_t::decode_struct(#decoder_var, None, |#struct_decoder_var| {
                    let mut #tag_var = #option_none;

                    let #output_var = loop {
                        let #option_some(mut #entry_var) = #struct_decoder_t::decode_field(#struct_decoder_var)? else {
                            return #result_err(#context_t::invalid_field_tag(#ctx_var, #type_name, "other"));
                        };

                        let decoder = #struct_field_decoder_t::decode_field_name(&mut #entry_var)?;

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
        str_ty,
        buf_t,
        context_t,
        decoder_t,
        default_function,
        fmt,
        option_none,
        option_some,
        option,
        result_err,
        result_ok,
        skip,
        struct_decoder_t,
        struct_field_decoder_t,
        visit_owned_fn,
        ..
    } = e.tokens;

    let struct_decoder_var = e.cx.ident("struct_decoder");
    let field_alloc_var = e.cx.ident("field_alloc");

    let type_name = &st.name;

    let mut patterns = Vec::with_capacity(st.fields.len());
    let mut assigns = Punctuated::<_, Token![,]>::new();

    let mut fields_with = Vec::new();

    for f in &st.fields {
        let tag = &f.tag;
        let var = &f.var;
        let decode_path = &f.decode_path.1;

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

        let fallback = if f.default_attr.is_some() {
            quote!(#default_function())
        } else {
            quote! {
                return #result_err(#context_t::expected_tag(#ctx_var, #type_name, #tag))
            }
        };

        let var = &f.var;

        assigns.push(syn::FieldValue {
            attrs: Vec::new(),
            member: f.member.clone(),
            colon_token: Some(<Token![:]>::default()),
            expr: syn::Expr::Verbatim(quote! {
                match #var {
                    #option_some(#var) => #var,
                    #option_none => #fallback,
                }
            }),
        });
    }

    let field_alloc;
    let decode_tag;
    let mut output_enum = quote!();

    match st.field_tag_method {
        TagMethod::String => {
            let mut outputs = Vec::new();
            let output =
                e.cx.type_with_span("TagVisitorOutput", e.input.ident.span());

            for (f, decode, trace) in fields_with {
                let (output_pattern, output_tag, output) =
                    build_tag_variant(e, f.span, f.index, &f.tag, &output);

                outputs.push(output);
                patterns.push((output_pattern, output_tag, decode, trace));
            }

            let patterns = outputs.iter().map(|o| o.as_arm(option_some));

            field_alloc = Some(quote! {
                let #option_some(mut #field_alloc_var) = #context_t::alloc(#ctx_var) else {
                    return #result_err(#context_t::alloc_failed(#ctx_var));
                };
            });

            decode_tag = quote! {
                #decoder_t::decode_string(#struct_decoder_var, #visit_owned_fn("a string variant tag", |string: &str| {
                    #result_ok(match string {
                        #(#patterns,)*
                        string => {
                            if !#buf_t::write(&mut #field_alloc_var, #str_ty::as_bytes(string)) {
                                return #result_err(#context_t::alloc_failed(#ctx_var));
                            }

                            #option_none
                        }
                    })
                }))?
            };

            let variants = outputs.iter().map(|o| &o.variant);

            let fmt_patterns = outputs.iter().map(|o| {
                let variant = &o.variant;
                let tag = o.tag;
                quote!(#output::#variant => #fmt::Debug::fmt(&#tag, f))
            });

            output_enum = quote! {
                enum #output {
                    #(#variants,)*
                }

                impl #fmt::Debug for #output {
                    #[inline]
                    fn fmt(&self, f: &mut #fmt::Formatter<'_>) -> #fmt::Result {
                        match self { #(#fmt_patterns,)* }
                    }
                }
            };
        }
        TagMethod::Any => {
            for (f, decode, trace) in fields_with {
                patterns.push((f.tag.clone(), f.tag.clone(), decode, trace));
            }

            let decode_t_decode = &e.decode_t_decode;

            field_alloc = None;

            decode_tag = quote! {
                #decode_t_decode(#ctx_var, #struct_decoder_var)?
            };
        }
    }

    let mut tag_stmt = syn::Local {
        attrs: Vec::new(),
        let_token: <Token![let]>::default(),
        pat: syn::Pat::Ident(syn::PatIdent {
            attrs: Vec::new(),
            by_ref: None,
            mutability: None,
            ident: tag_var.clone(),
            subpat: None,
        }),
        init: None,
        semi_token: <Token![;]>::default(),
    };

    if let Some((_, name_type)) = st.name_type {
        tag_stmt.pat = syn::Pat::Type(syn::PatType {
            attrs: Vec::new(),
            pat: Box::new(tag_stmt.pat),
            colon_token: <Token![:]>::default(),
            ty: Box::new(name_type.clone()),
        });
    }

    let unsupported = match variant_tag {
        Some(variant_tag) => quote! {
            #context_t::invalid_variant_field_tag(#ctx_var, #type_name, #variant_tag, #tag_var)
        },
        None => quote! {
            #context_t::invalid_field_tag(#ctx_var, #type_name, #tag_var)
        },
    };

    let mut body = quote! {
        if #skip::is_unsupported(#decoder_t::try_skip(#struct_field_decoder_t::decode_field_value(#struct_decoder_var)?)?) {
            return #result_err(#unsupported);
        }
    };

    if !patterns.is_empty() {
        let patterns = patterns
            .into_iter()
            .map(|(pattern_var, _, decode, (enter, leave))| {
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
            match #tag_var { #(#patterns,)* #tag_var => { #body } }
        }
    };

    tag_stmt.init = Some(syn::LocalInit {
        eq_token: <Token![=]>::default(),
        expr: Box::new(syn::Expr::Verbatim(quote! {{
            let #struct_decoder_var = #struct_field_decoder_t::decode_field_name(&mut #struct_decoder_var)?;
            #decode_tag
        }})),
        diverge: None,
    });

    let path = &st.path;
    let fields_len = st.fields.len();

    let decls = st
        .fields
        .iter()
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
        #decoder_t::decode_struct(#decoder_var, Some(#fields_len), |type_decoder| {
            while let #option_some(mut #struct_decoder_var) = #struct_decoder_t::decode_field(type_decoder)? {
                #field_alloc
                #tag_stmt
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

    let [f] = &st_.fields[..] else {
        e.transparent_diagnostics(st_.span, &st_.fields);
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

    for f in &st_.fields {
        if let Some(span) = f.default_attr {
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

    let path = &st_.path;

    if assign.is_empty() {
        return Ok(quote!(#path {}));
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

    Ok(quote! {{
        #enter
        let #output_var = #decoder_t::decode_pack(#decoder_var, |#pack| {
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
    let variant = e.cx.type_with_span(&format!("Variant{}", index), span);
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
