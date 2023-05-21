use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Token;

use crate::expander::{Result, TagMethod};
use crate::internals::attr::{EnumTag, EnumTagging, Packing};
use crate::internals::build::{Build, BuildData, EnumBuild, StructBuild, VariantBuild};

pub(crate) fn expand_decode_entry(e: Build<'_>) -> Result<TokenStream> {
    e.validate_decode()?;

    let ctx_var = syn::Ident::new("__ctx", Span::call_site());
    let root_decoder_var = syn::Ident::new("__decoder", Span::call_site());

    let body = match &e.data {
        BuildData::Struct(st) => decode_struct(&e, st, &ctx_var, &root_decoder_var)?,
        BuildData::Enum(en) => decode_enum(&e, en, &ctx_var, &root_decoder_var)?,
    };

    if e.cx.has_errors() {
        return Err(());
    }

    // Figure out which lifetime to use for what. We use the first lifetime in
    // the type (if any is available) as the decoder lifetime. Else we generate
    // a new anonymous lifetime `'de` to use for the `Decode` impl.
    let mut impl_generics = e.input.generics.clone();
    let type_ident = &e.input.ident;

    let (lt, exists) = if let Some(existing) = impl_generics.lifetimes().next() {
        (existing.clone(), true)
    } else {
        let lt = syn::LifetimeParam::new(syn::Lifetime::new("'de", e.input.span()));
        (lt, false)
    };

    if !exists {
        impl_generics.params.push(lt.clone().into());
    }

    let decode_t = &e.tokens.decode_t;
    let context_t = &e.tokens.context_t;
    let core_result: &syn::Path = &e.tokens.core_result;
    let decoder_t = &e.tokens.decoder_t;
    let original_generics = &e.input.generics;

    let (impl_generics, mode_ident, mut where_clause) =
        e.expansion.as_impl_generics(impl_generics, e.tokens);

    if !e.bounds.is_empty() && !e.decode_bounds.is_empty() {
        let where_clause = where_clause.get_or_insert_with(|| syn::WhereClause {
            where_token: <Token![where]>::default(),
            predicates: Default::default(),
        });

        where_clause.predicates.extend(
            e.bounds
                .iter()
                .chain(e.decode_bounds.iter())
                .map(|(_, v)| v.clone()),
        );
    }

    let buf_lt = e.cx.lifetime("'buf");
    let c_param = e.cx.ident("C");
    let d_param = e.cx.ident("D");

    Ok(quote! {
        #[automatically_derived]
        #[allow(clippy::init_numbered_fields)]
        #[allow(clippy::let_unit_value)]
        impl #impl_generics #decode_t<#lt, #mode_ident> for #type_ident #original_generics #where_clause {
            #[inline]
            fn decode<#buf_lt, #c_param, #d_param>(#ctx_var: &mut #c_param, #root_decoder_var: #d_param) -> #core_result<Self, <#c_param as #context_t<#buf_lt>>::Error>
            where
                #c_param: #context_t<#buf_lt, Input = <#d_param as #decoder_t<#lt>>::Error>,
                #d_param: #decoder_t<#lt>
            {
                #body
            }
        }
    })
}

fn decode_struct(
    e: &Build<'_>,
    st: &StructBuild<'_>,
    ctx_var: &syn::Ident,
    decoder_var: &syn::Ident,
) -> Result<TokenStream> {
    let body = match st.packing {
        Packing::Tagged => decode_tagged(e, st.span, ctx_var, decoder_var, None, e.type_name, st)?,
        Packing::Packed => decode_packed(e, ctx_var, decoder_var, st)?,
        Packing::Transparent => decode_transparent(e, ctx_var, decoder_var, st)?,
    };

    let result_ok = &e.tokens.result_ok;
    Ok(quote!(#result_ok({ #body })))
}

fn decode_enum(
    e: &Build<'_>,
    en: &EnumBuild,
    ctx_var: &syn::Ident,
    decoder_var: &syn::Ident,
) -> Result<TokenStream> {
    if let Some(&(span, Packing::Packed)) = en.packing_span {
        e.decode_packed_enum_diagnostics(span);
        return Err(());
    }

    let as_decoder_t = &e.tokens.as_decoder_t;
    let context_t = &e.tokens.context_t;
    let decoder_t = &e.tokens.decoder_t;
    let fmt = &e.tokens.fmt;
    let option_none = &e.tokens.option_none;
    let option_some = &e.tokens.option_some;
    let pair_decoder_t = &e.tokens.pair_decoder_t;
    let pairs_decoder_t = &e.tokens.pairs_decoder_t;
    let result_err = &e.tokens.result_err;
    let result_ok = &e.tokens.result_ok;
    let type_name = &e.type_name;
    let variant_decoder_t_tag = &e.tokens.variant_decoder_t_tag;
    let visit_owned_fn = &e.tokens.visit_owned_fn;
    let variant_decoder_t = &e.tokens.variant_decoder_t;

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
    let tag_var = e.cx.ident("tag");
    let variant_decoder_var = e.cx.ident("variant_decoder");
    let variant_marker_var = e.cx.ident("variant_marker");
    let variant_tag_var = e.cx.ident("variant_tag");
    let variant_output_var = e.cx.ident("variant_output");

    let mut variant_output_tags = Vec::new();

    let mut fallback = match en.fallback {
        Some(ident) => {
            quote! {{
                if !#variant_decoder_t::skip_variant(&mut #variant_decoder_var, #ctx_var)? {
                    return #result_err(#context_t::invalid_variant_tag(#ctx_var, #type_name, #variant_tag_var));
                }

                #variant_decoder_t::end(#variant_decoder_var, #ctx_var)?;
                Self::#ident {}
            }}
        }
        None => quote! {
            return #result_err(#context_t::invalid_variant_tag(#ctx_var, #type_name, #variant_tag_var))
        },
    };

    let decode_tag;
    let mut output_enum = quote!();

    match en.variant_tag_method {
        TagMethod::String => {
            let mut tag_variants = Vec::new();
            let output = syn::Ident::new("VariantTagVisitorOutput", en.span);

            for v in en.variants.iter().filter(|v| !v.is_default) {
                let (output_tag, tag_variant) =
                    build_tag_variant(e, v.span, v.index, &v.tag, &output);

                tag_variants.push(tag_variant);
                variant_output_tags.push((v, output_tag));
            }

            let arms = tag_variants.iter().map(|o| o.as_arm(option_some));

            decode_tag = quote! {
                #decoder_t::decode_string(#variant_decoder_var, #ctx_var, #visit_owned_fn("a string variant tag", |#ctx_var, string: &str| {
                    #result_ok(match string {
                        #(#arms,)*
                        string => {
                            #context_t::store_string(#ctx_var, string);
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

            fallback = quote! {
                #option_none => {
                    #fallback
                }
            };
        }
        TagMethod::Any => {
            for v in en.variants.iter().filter(|v| !v.is_default) {
                variant_output_tags.push((v, v.tag.clone()));
            }

            let decode_t_decode = &e.decode_t_decode;
            decode_tag = quote!(#decode_t_decode(#ctx_var, #variant_decoder_var)?);
            fallback = quote!(_ => #fallback);
        }
    }

    let name_type = en.name_type.as_ref().map(|(_, ty)| quote!(: #ty));

    let Some(enum_tagging) = en.enum_tagging else {
        let patterns = variant_output_tags.iter().flat_map(|(v, tag)| {
            let name = &v.name;

            let decode = decode_variant(e, v, ctx_var, &body_decoder_var, &variant_tag_var).ok()?;

            let formatted_tag = match en.name_format_with {
                Some((_, path)) => quote!(&#path(&#tag)),
                None => quote!(&#tag),
            };

            Some(quote! {
                #tag => {
                    let #variant_marker_var = #context_t::trace_enter_variant(#ctx_var, #name, #formatted_tag);

                    let #output_var = {
                        let #body_decoder_var = #variant_decoder_t::variant(&mut #variant_decoder_var, #ctx_var)?;
                        #decode
                    };

                    #variant_decoder_t::end(#variant_decoder_var, #ctx_var)?;
                    #context_t::trace_leave_variant(#ctx_var, #variant_marker_var);
                    #output_var
                }
            })
        });


        return Ok(quote! {{
            #output_enum

            let mut #variant_decoder_var = #decoder_t::decode_variant(#decoder_var, #ctx_var)?;

            let #variant_tag_var #name_type = {
                let mut #variant_decoder_var = #variant_decoder_t_tag(&mut #variant_decoder_var, #ctx_var)?;
                #decode_tag
            };

            #result_ok(match #variant_tag_var {
                #(#patterns,)*
                #fallback
            })
        }});
    };

    let mode_ident = e.mode_ident.as_path();

    match enum_tagging {
        EnumTagging::Internal {
            tag:
                EnumTag {
                    value: field_tag,
                    method: field_tag_method,
                },
        } => {
            let patterns = variant_output_tags.iter().flat_map(|(v, tag)| {
                let name = &v.name;

                let formatted_tag = match en.name_format_with {
                    Some((_, path)) => quote!(&#path(&#tag)),
                    None => quote!(&#tag),
                };

                let decode = decode_variant(e, v, ctx_var, &buffer_decoder_var, &variant_tag_var).ok()?;

                Some(quote! {
                    #tag => {
                        let #variant_marker_var = #context_t::trace_enter_variant(#ctx_var, #name, #formatted_tag);
                        let #buffer_decoder_var = #as_decoder_t::as_decoder(&#buffer_var, #ctx_var)?;
                        let #variant_output_var = #decode;
                        #context_t::trace_leave_variant(#ctx_var, #variant_marker_var);
                        #variant_output_var
                    }
                })
            });

            let decode_t_decode = &e.decode_t_decode;

            let mut outcome_enum = quote!();
            let decode_match;

            match field_tag_method {
                Some(TagMethod::String) => {
                    outcome_enum = quote! {
                        enum Outcome {
                            Tag,
                            Err,
                        }
                    };

                    let decode_outcome = quote! {
                        #decoder_t::decode_string(decoder, #ctx_var, #visit_owned_fn("a string field tag", |#ctx_var, string: &str| {
                            #result_ok(match string {
                                #field_tag => Outcome::Tag,
                                string => {
                                    #context_t::store_string(#ctx_var, string);
                                    Outcome::Err
                                }
                            })
                        }))?
                    };

                    decode_match = quote! {
                        match #decode_outcome {
                            Outcome::Tag => {
                                break #pair_decoder_t::second(#entry_var, #ctx_var)?;
                            }
                            Outcome::Err => {
                                if !#pair_decoder_t::skip_second(#entry_var, #ctx_var)? {
                                    return #result_err(#context_t::invalid_field_string_tag(#ctx_var, #type_name));
                                }
                            }
                        }
                    };
                }
                _ => {
                    decode_match = quote! {
                        match #decode_t_decode(#ctx_var, decoder)? {
                            #field_tag => {
                                break #pair_decoder_t::second(#entry_var, #ctx_var)?;
                            }
                            field => {
                                if !#pair_decoder_t::skip_second(#entry_var, #ctx_var)? {
                                    return #result_err(#context_t::invalid_field_tag(#ctx_var, #type_name, field));
                                }
                            }
                        }
                    };
                }
            };

            Ok(quote! {{
                #output_enum
                #outcome_enum

                let #buffer_var = #decoder_t::decode_buffer::<#mode_ident, _>(#decoder_var, #ctx_var)?;
                let st = #as_decoder_t::as_decoder(&#buffer_var, #ctx_var)?;
                let mut st = #decoder_t::decode_map(st, #ctx_var)?;

                let #variant_tag_var #name_type = {
                    let #variant_decoder_var = loop {
                        let #option_some(mut #entry_var) = #pairs_decoder_t::next(&mut st, #ctx_var)? else {
                            return #result_err(#context_t::missing_variant_field(#ctx_var, #type_name, #field_tag));
                        };

                        let decoder = #pair_decoder_t::first(&mut #entry_var, #ctx_var)?;
                        #decode_match
                    };

                    #decode_tag
                };

                #pairs_decoder_t::end(st, #ctx_var)?;

                #result_ok(match #variant_tag_var {
                    #(#patterns,)*
                    #fallback
                })
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
            let patterns = variant_output_tags.iter().flat_map(|(v, tag)| {
                let name = &v.name;

                let formatted_tag = match en.name_format_with {
                    Some((_, path)) => quote!(&#path(&#tag)),
                    None => quote!(&#tag),
                };

                let decode = decode_variant(e, v, ctx_var, &body_decoder_var, &variant_tag_var).ok()?;

                Some(quote! {
                    #tag => {
                        let #variant_marker_var = #context_t::trace_enter_variant(#ctx_var, #name, #formatted_tag);
                        let #variant_output_var = #decode;
                        #context_t::trace_leave_variant(#ctx_var, #variant_marker_var);
                        #variant_output_var
                    }
                })
            });

            let decode_t_decode = &e.decode_t_decode;

            let mut outcome_enum = quote!();
            let decode_match;

            match tag_method {
                Some(TagMethod::String) => {
                    outcome_enum = quote! {
                        enum Outcome { Tag, Content, Err }
                    };

                    decode_match = quote! {
                        let outcome = #decoder_t::decode_string(decoder, #ctx_var, #visit_owned_fn("a string field tag", |#ctx_var, string: &str| {
                            #result_ok(match string {
                                #tag => Outcome::Tag,
                                #content => Outcome::Content,
                                string => {
                                    #context_t::store_string(#ctx_var, string);
                                    Outcome::Err
                                }
                            })
                        }))?;

                        match outcome {
                            Outcome::Tag => {
                                let #variant_decoder_var = #pair_decoder_t::second(#entry_var, #ctx_var)?;
                                #tag_var = #option_some(#decode_tag);
                            }
                            Outcome::Content => {
                                let #option_some(#variant_tag_var #name_type) = #tag_var else {
                                    return #result_err(#context_t::invalid_field_tag(#ctx_var, #type_name, #tag));
                                };

                                let #body_decoder_var = #pair_decoder_t::second(#entry_var, #ctx_var)?;

                                break #result_ok(match #variant_tag_var {
                                    #(#patterns,)*
                                    #fallback
                                });
                            }
                            Outcome::Err => {
                                if !#pair_decoder_t::skip_second(#entry_var, #ctx_var)? {
                                    return #result_err(#context_t::invalid_field_string_tag(#ctx_var, #type_name));
                                }
                            }
                        }
                    };
                }
                _ => {
                    decode_match = quote! {
                        match #decode_t_decode(#ctx_var, decoder)? {
                            #tag => {
                                let #variant_decoder_var = #pair_decoder_t::second(#entry_var, #ctx_var)?;
                                #tag_var = #option_some(#decode_tag);
                            }
                            #content => {
                                let #option_some(#variant_tag_var #name_type) = #tag_var else {
                                    return #result_err(#context_t::invalid_field_tag(#ctx_var, #type_name, #tag));
                                };

                                let #body_decoder_var = #pair_decoder_t::second(#entry_var, #ctx_var)?;

                                break #result_ok(match #variant_tag_var {
                                    #(#patterns,)*
                                    #fallback
                                });
                            }
                            field => {
                                if !#pair_decoder_t::skip_second(#entry_var, #ctx_var)? {
                                    return #result_err(#context_t::invalid_field_tag(#ctx_var, #type_name, field));
                                }
                            }
                        }
                    };
                }
            };

            Ok(quote! {{
                #output_enum
                #outcome_enum

                let mut #struct_decoder_var = #decoder_t::decode_map(#decoder_var, #ctx_var)?;
                let mut #tag_var = #option_none;

                let #output_var = loop {
                    let #option_some(mut #entry_var) = #pairs_decoder_t::next(&mut #struct_decoder_var, #ctx_var)? else {
                        return #result_err(#context_t::invalid_field_tag(#ctx_var, #type_name, "other"));
                    };

                    let decoder = #pair_decoder_t::first(&mut #entry_var, #ctx_var)?;
                    #decode_match
                };

                #pairs_decoder_t::end(#struct_decoder_var, #ctx_var)?;
                #output_var
            }})
        }
    }
}

fn decode_variant(
    e: &Build,
    v: &VariantBuild<'_>,
    ctx_var: &Ident,
    body_decoder_var: &Ident,
    variant_tag: &Ident,
) -> Result<TokenStream, ()> {
    Ok(match v.st_.packing {
        Packing::Tagged => decode_tagged(
            e,
            v.span,
            ctx_var,
            body_decoder_var,
            Some(variant_tag),
            v.name,
            &v.st_,
        )?,
        Packing::Packed => decode_packed(e, ctx_var, body_decoder_var, &v.st_)?,
        Packing::Transparent => decode_transparent(e, ctx_var, body_decoder_var, &v.st_)?,
    })
}

/// Decode something tagged.
///
/// If `variant_name` is specified it implies that a tagged enum is being
/// decoded.
fn decode_tagged(
    e: &Build<'_>,
    span: Span,
    ctx_var: &syn::Ident,
    parent_decoder_var: &syn::Ident,
    variant_tag: Option<&syn::Ident>,
    type_name: &syn::LitStr,
    st_: &StructBuild<'_>,
) -> Result<TokenStream> {
    let struct_decoder_var = syn::Ident::new("struct_decoder", span);

    let context_t = &e.tokens.context_t;
    let decoder_t = &e.tokens.decoder_t;
    let default_function = &e.tokens.default_function;
    let fmt = &e.tokens.fmt;
    let option_none = &e.tokens.option_none;
    let option_some = &e.tokens.option_some;
    let pair_decoder_t = &e.tokens.pair_decoder_t;
    let pairs_decoder_t = &e.tokens.pairs_decoder_t;
    let result_err = &e.tokens.result_err;
    let result_ok = &e.tokens.result_ok;
    let visit_owned_fn = &e.tokens.visit_owned_fn;

    let mut decls = Vec::with_capacity(st_.fields.len());
    let mut patterns = Vec::with_capacity(st_.fields.len());
    let mut assigns = Punctuated::<_, Token![,]>::new();

    let mut fields_with = Vec::new();

    for f in &st_.fields {
        let tag = &f.tag;
        let (span, decode_path) = &f.decode_path;
        let var = syn::Ident::new(&format!("v{}", f.index), *span);

        decls.push(quote!(let mut #var = #option_none;));

        let (name, enter) = match &f.member {
            syn::Member::Named(name) => (
                syn::Lit::Str(syn::LitStr::new(&name.to_string(), name.span())),
                syn::Ident::new("trace_enter_named_field", Span::call_site()),
            ),
            syn::Member::Unnamed(index) => (
                syn::Lit::Int(syn::LitInt::from(Literal::u32_suffixed(index.index))),
                syn::Ident::new("trace_enter_unnamed_field", Span::call_site()),
            ),
        };

        let formatted_tag = match &st_.name_format_with {
            Some((_, path)) => quote!(&#path(&#tag)),
            None => quote!(&#tag),
        };

        let decode = quote! {
            #var = {
                let trace_marker = #context_t::#enter(#ctx_var, #name, #formatted_tag);
                let field_value = #decode_path(#ctx_var, #struct_decoder_var)?;
                #context_t::trace_leave_field(#ctx_var, trace_marker);
                #option_some(field_value)
            }
        };

        fields_with.push((f, decode));

        let fallback = if f.default_attr.is_some() {
            quote!(#default_function())
        } else {
            quote! {
                return #result_err(#context_t::expected_tag(#ctx_var, #type_name, #tag))
            }
        };

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

    let decode_tag;
    let mut output_enum = quote!();

    match st_.field_tag_method {
        TagMethod::String => {
            let mut outputs = Vec::new();
            let output = syn::Ident::new("TagVisitorOutput", e.input.ident.span());

            for (f, decode) in fields_with {
                let (output_tag, output) = build_tag_variant(e, f.span, f.index, &f.tag, &output);

                outputs.push(output);
                patterns.push((output_tag, decode));
            }

            let patterns = outputs.iter().map(|o| o.as_arm(option_some));

            decode_tag = quote! {
                #decoder_t::decode_string(#struct_decoder_var, #ctx_var, #visit_owned_fn("a string variant tag", |#ctx_var, string: &str| {
                    #result_ok(match string {
                        #(#patterns,)*
                        string => {
                            #context_t::store_string(#ctx_var, string);
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
            for (f, decode) in fields_with {
                patterns.push((f.tag.clone(), decode));
            }

            let decode_t_decode = &e.decode_t_decode;

            decode_tag = quote! {
                #decode_t_decode(#ctx_var, #struct_decoder_var)?
            };
        }
    }

    let skip_field = quote! {
        #pair_decoder_t::skip_second(#struct_decoder_var, #ctx_var)?
    };

    let unsupported = match variant_tag {
        Some(variant_tag) => quote! {
            #context_t::invalid_variant_field_tag(#ctx_var, #type_name, #variant_tag, tag)
        },
        None => quote! {
            #context_t::invalid_field_tag(#ctx_var, #type_name, tag)
        },
    };

    let tag = syn::Ident::new("tag", Span::call_site());

    let mut tag_stmt = syn::Local {
        attrs: Vec::new(),
        let_token: <Token![let]>::default(),
        pat: syn::Pat::Ident(syn::PatIdent {
            attrs: Vec::new(),
            by_ref: None,
            mutability: None,
            ident: tag.clone(),
            subpat: None,
        }),
        init: None,
        semi_token: <Token![;]>::default(),
    };

    if let Some((_, name_type)) = st_.name_type {
        tag_stmt.pat = syn::Pat::Type(syn::PatType {
            attrs: Vec::new(),
            pat: Box::new(tag_stmt.pat),
            colon_token: <Token![:]>::default(),
            ty: Box::new(name_type.clone()),
        });
    }

    let mut body = quote! {
        if !#skip_field {
            return #result_err(#unsupported);
        }
    };

    if !patterns.is_empty() {
        let patterns = patterns
            .into_iter()
            .map(|(tag, decode)| {
                quote! {
                    #tag => {
                        let #struct_decoder_var = #pair_decoder_t::second(#struct_decoder_var, #ctx_var)?;
                        #decode;
                    }
                }
            });

        body = quote! {
            match tag { #(#patterns,)* #tag => { #body } }
        }
    };

    tag_stmt.init = Some(syn::LocalInit {
        eq_token: <Token![=]>::default(),
        expr: Box::new(syn::Expr::Verbatim(quote! {{
            let #struct_decoder_var = #pair_decoder_t::first(&mut #struct_decoder_var, #ctx_var)?;
            #decode_tag
        }})),
        diverge: None,
    });

    let path = &st_.path;
    let fields_len = st_.fields.len();

    Ok(quote! {{
        #(#decls)*
        #output_enum
        let mut type_decoder = #decoder_t::decode_struct(#parent_decoder_var, #ctx_var, #fields_len)?;

        while let #option_some(mut #struct_decoder_var) = #pairs_decoder_t::next(&mut type_decoder, #ctx_var)? {
            #tag_stmt
            #body
        }

        #pairs_decoder_t::end(type_decoder, #ctx_var)?;
        #path { #assigns }
    }})
}

/// Decode a transparent value.
fn decode_transparent(
    e: &Build<'_>,
    ctx_var: &syn::Ident,
    decoder_var: &syn::Ident,
    st_: &StructBuild<'_>,
) -> Result<TokenStream> {
    let [f] = &st_.fields[..] else {
        e.transparent_diagnostics(st_.span, &st_.fields);
        return Err(());
    };

    let path = &st_.path;
    let decode_path = &f.decode_path.1;
    let member = &f.member;

    Ok(quote! {
        #path {
            #member: #decode_path(#ctx_var, #decoder_var)?
        }
    })
}

/// Decode something packed.
fn decode_packed(
    e: &Build<'_>,
    ctx_var: &syn::Ident,
    decoder_var: &syn::Ident,
    st_: &StructBuild<'_>,
) -> Result<TokenStream> {
    let decoder_t = &e.tokens.decoder_t;
    let pack_decoder_t = &e.tokens.pack_decoder_t;
    let output_var = e.cx.ident("output");

    let mut assign = Vec::new();

    for f in &st_.fields {
        if let Some(span) = f.default_attr {
            e.packed_default_diagnostics(span);
        }

        let (_, decode_path) = &f.decode_path;
        let member = &f.member;

        assign.push(quote! {
            #member: {
                let field_decoder = #pack_decoder_t::next(&mut unpack, #ctx_var)?;
                #decode_path(#ctx_var, field_decoder)?
            }
        });
    }

    let path = &st_.path;

    if assign.is_empty() {
        return Ok(quote!(#path {}));
    }

    Ok(quote! {{
        let mut unpack = #decoder_t::decode_pack(#decoder_var, #ctx_var)?;
        let #output_var = #path { #(#assign),* };
        #pack_decoder_t::end(unpack, #ctx_var)?;
        #output_var
    }})
}

/// Output type used when indirectly encoding a variant or field as type which
/// might require special handling. Like a string.
pub(crate) struct TagVariant<'a> {
    /// The path of the variant this output should generate.
    path: syn::Path,
    /// The identified of the variant this path generates.
    variant: syn::Ident,
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

        let mut args = Punctuated::default();
        args.push(body);

        syn::Arm {
            attrs: Vec::new(),
            pat: syn::Pat::Verbatim(self.tag.to_token_stream()),
            guard: None,
            fat_arrow_token: <Token![=>]>::default(),
            body: Box::new(build_call(option_some, args)),
            comma: None,
        }
    }
}

fn build_call(path: &syn::Path, args: Punctuated<syn::Expr, Token![,]>) -> syn::Expr {
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

fn build_tag_variant<'a>(
    e: &Build<'_>,
    span: Span,
    index: usize,
    tag: &'a syn::Expr,
    output: &syn::Ident,
) -> (syn::Expr, TagVariant<'a>) {
    let variant = syn::Ident::new(&format!("Variant{}", index), span);
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

    let mut args = Punctuated::new();
    args.push(expr);

    (build_call(&e.tokens.option_some, args), output)
}
