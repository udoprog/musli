use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::Token;

use crate::expander::Result;
use crate::internals::attr::{EnumTag, EnumTagging, Packing};
use crate::internals::build::{Body, Build, BuildData, Enum, Variant};
use crate::internals::tokens::Tokens;

struct Ctxt<'a> {
    ctx_var: &'a syn::Ident,
    encoder_var: &'a syn::Ident,
    trace: bool,
}

pub(crate) fn expand_insert_entry(e: Build<'_>) -> Result<TokenStream> {
    e.validate_encode()?;

    let type_ident = &e.input.ident;

    let encoder_var = e.cx.ident("encoder");
    let ctx_var = e.cx.ident("ctx");
    let e_param = e.cx.type_with_span("E", Span::call_site());

    let cx = Ctxt {
        ctx_var: &ctx_var,
        encoder_var: &encoder_var,
        trace: true,
    };

    let Tokens {
        core_result,
        encode_t,
        encoder_t,
        ..
    } = e.tokens;

    let body = match &e.data {
        BuildData::Struct(st) => encode_struct(&cx, &e, st)?,
        BuildData::Enum(en) => encode_enum(&cx, &e, en)?,
    };

    if e.cx.has_errors() {
        return Err(());
    }

    let (mut impl_generics, mode_ident) = e
        .expansion
        .as_impl_generics(e.input.generics.clone(), e.tokens);

    if !e.bounds.is_empty() {
        let where_clause = impl_generics.make_where_clause();

        where_clause
            .predicates
            .extend(e.bounds.iter().map(|(_, v)| v.clone()));
    }

    let (impl_generics, _, where_clause) = impl_generics.split_for_impl();
    let (_, type_generics, _) = e.input.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #encode_t<#mode_ident> for #type_ident #type_generics #where_clause {
            #[inline]
            fn encode<#e_param>(&self, #ctx_var: &#e_param::Cx, #encoder_var: #e_param) -> #core_result<<#e_param as #encoder_t>::Ok, <#e_param as #encoder_t>::Error>
            where
                #e_param: #encoder_t<Mode = #mode_ident>,
            {
                #body
            }
        }
    })
}

/// Encode a struct.
fn encode_struct(cx: &Ctxt<'_>, e: &Build<'_>, st: &Body<'_>) -> Result<TokenStream> {
    let Ctxt {
        ctx_var,
        encoder_var,
        ..
    } = *cx;

    let Tokens {
        context_t,
        encoder_t,
        result_ok,
        ..
    } = e.tokens;

    let pack_var = e.cx.ident("pack");
    let output_var = e.cx.ident("output");

    let (encoders, tests) = insert_fields(cx, e, st, &pack_var)?;

    let type_name = &st.name;

    let enter = cx
        .trace
        .then(|| quote!(#context_t::enter_struct(#ctx_var, #type_name);));
    let leave = cx
        .trace
        .then(|| quote!(#context_t::leave_struct(#ctx_var);));

    let encode;

    match st.packing {
        Packing::Transparent => {
            let f = match &st.fields[..] {
                [f] => f,
                _ => {
                    e.transparent_diagnostics(st.span, &st.fields);
                    return Err(());
                }
            };

            let access = &f.self_access;
            let encode_path = &f.encode_path.1;

            encode = quote! {{
                #enter
                let #output_var = #encode_path(#access, #ctx_var, #encoder_var)?;
                #leave
                #output_var
            }};
        }
        Packing::Tagged => {
            let len = length_test(st.fields.len(), &tests);
            let decls = tests.iter().map(|t| &t.decl);

            encode = quote! {{
                #enter
                #(#decls)*
                let #output_var = #encoder_t::encode_struct_fn(#encoder_var, #len, |#encoder_var| {
                    #(#encoders)*
                    #result_ok(())
                })?;
                #leave
                #output_var
            }};
        }
        Packing::Packed => {
            let decls = tests.iter().map(|t| &t.decl);

            encode = quote! {{
                #enter
                let #output_var = #encoder_t::encode_pack_fn(#encoder_var, |#pack_var| {
                    #(#decls)*
                    #(#encoders)*
                    #result_ok(())
                })?;
                #leave
                #output_var
            }};
        }
    }

    Ok(quote!(#result_ok(#encode)))
}

struct FieldTest {
    decl: TokenStream,
    var: syn::Ident,
}

fn insert_fields(
    cx: &Ctxt<'_>,
    e: &Build<'_>,
    st: &Body<'_>,
    pack_var: &syn::Ident,
) -> Result<(Vec<TokenStream>, Vec<FieldTest>)> {
    let Ctxt {
        ctx_var,
        encoder_var,
        ..
    } = *cx;

    let Tokens {
        context_t,
        result_ok,
        sequence_encoder_t,
        struct_encoder_t,
        struct_field_encoder_t,
        ..
    } = e.tokens;

    let encode_t_encode = &e.encode_t_encode;

    let sequence_decoder_next_var = e.cx.ident("sequence_decoder_next");
    let pair_encoder_var = e.cx.ident("pair_encoder");
    let field_encoder_var = e.cx.ident("field_encoder");
    let value_encoder_var = e.cx.ident("value_encoder");

    let mut encoders = Vec::with_capacity(st.fields.len());
    let mut tests = Vec::with_capacity(st.fields.len());

    for f in &st.fields {
        let encode_path = &f.encode_path.1;
        let access = &f.self_access;
        let tag = &f.tag;

        let mut encode;

        let enter = match &f.member {
            syn::Member::Named(ident) => {
                let field_name = syn::LitStr::new(&ident.to_string(), ident.span());

                cx.trace.then(|| {
                    let tag = st.name_format(tag);

                    quote! {
                        #context_t::enter_named_field(#ctx_var, #field_name, #tag);
                    }
                })
            }
            syn::Member::Unnamed(index) => {
                let index = index.index;
                cx.trace.then(|| {
                    let tag = st.name_format(tag);
                    quote! {
                        #context_t::enter_unnamed_field(#ctx_var, #index, #tag);
                    }
                })
            }
        };

        let leave = cx.trace.then(|| quote!(#context_t::leave_field(#ctx_var);));

        match f.packing {
            Packing::Tagged | Packing::Transparent => {
                encode = quote! {
                    #enter

                    #struct_encoder_t::encode_field_fn(#encoder_var, |#pair_encoder_var| {
                        let #field_encoder_var = #struct_field_encoder_t::encode_field_name(#pair_encoder_var)?;
                        #encode_t_encode(&#tag, #ctx_var, #field_encoder_var)?;
                        let #value_encoder_var = #struct_field_encoder_t::encode_field_value(#pair_encoder_var)?;
                        #encode_path(#access, #ctx_var, #value_encoder_var)?;
                        #result_ok(())
                    })?;

                    #leave
                };
            }
            Packing::Packed => {
                encode = quote! {
                    #enter
                    let #sequence_decoder_next_var = #sequence_encoder_t::encode_next(#pack_var)?;
                    #encode_path(#access, #ctx_var, #sequence_decoder_next_var)?;
                    #leave
                };
            }
        };

        if let Some((_, skip_encoding_if_path)) = f.skip_encoding_if.as_ref() {
            let var = e.cx.ident_with_span(&format!("t{}", f.index), f.span);

            let decl = quote! {
                let #var = !#skip_encoding_if_path(#access);
            };

            encode = quote! {
                if #var {
                    #encode
                }
            };

            tests.push(FieldTest { decl, var })
        }

        encoders.push(encode);
    }

    Ok((encoders, tests))
}

/// Encode an internally tagged enum.
fn encode_enum(cx: &Ctxt<'_>, e: &Build<'_>, en: &Enum<'_>) -> Result<TokenStream> {
    let Ctxt { ctx_var, .. } = *cx;

    let Tokens {
        context_t,
        result_err,
        result_ok,
        ..
    } = e.tokens;

    let type_name = en.name;

    if let Some(&(span, Packing::Transparent)) = en.packing_span {
        e.encode_transparent_enum_diagnostics(span);
        return Err(());
    }

    let mut variants = Vec::with_capacity(en.variants.len());

    for v in &en.variants {
        let Ok((pattern, encode)) = encode_variant(cx, e, en, v) else {
            continue;
        };

        variants.push(quote!(#pattern => #encode));
    }

    // Special case: uninhabitable types.
    Ok(if variants.is_empty() {
        quote!(#result_err(#context_t::uninhabitable(#ctx_var, #type_name)))
    } else {
        quote! {
            #result_ok(match self {
                #(#variants),*
            })
        }
    })
}

/// Setup encoding for a single variant. that is externally tagged.
fn encode_variant(
    cx: &Ctxt<'_>,
    b: &Build<'_>,
    en: &Enum,
    v: &Variant,
) -> Result<(syn::PatStruct, TokenStream)> {
    let pack_var = b.cx.ident("pack");

    let (encoders, tests) = insert_fields(cx, b, &v.st, &pack_var)?;

    let Ctxt {
        ctx_var,
        encoder_var,
        ..
    } = *cx;

    let Tokens {
        context_t,
        encoder_t,
        result_ok,
        struct_encoder_t,
        struct_field_encoder_t,
        variant_encoder_t,
        ..
    } = b.tokens;

    let type_name = v.st.name;

    let mut encode;

    match en.enum_tagging {
        None => {
            match v.st.packing {
                Packing::Transparent => {
                    let [f] = &v.st.fields[..] else {
                        b.transparent_diagnostics(v.span, &v.st.fields);
                        return Err(());
                    };

                    let encode_path = &f.encode_path.1;
                    let var = &f.self_access;
                    encode = quote!(#encode_path(#var, #ctx_var, #encoder_var)?);
                }
                Packing::Packed => {
                    let decls = tests.iter().map(|t| &t.decl);

                    encode = quote! {{
                        #encoder_t::encode_pack_fn(#encoder_var, |#pack_var| {
                            #(#decls)*
                            #(#encoders)*
                            #result_ok(())
                        })?
                    }};
                }
                Packing::Tagged => {
                    let decls = tests.iter().map(|t| &t.decl);
                    let len = length_test(v.st.fields.len(), &tests);

                    encode = quote! {{
                        #encoder_t::encode_struct_fn(#encoder_var, #len, |#encoder_var| {
                            #(#decls)*
                            #(#encoders)*
                            #result_ok(())
                        })?
                    }};
                }
            }

            if let Packing::Tagged = en.enum_packing {
                let encode_t_encode = &b.encode_t_encode;
                let tag = &v.tag;
                let variant_encoder = b.cx.ident("variant_encoder");
                let tag_encoder = b.cx.ident("tag_encoder");

                encode = quote! {{
                    #encoder_t::encode_variant_fn(#encoder_var, |#variant_encoder| {
                        let #tag_encoder = #variant_encoder_t::encode_tag(#variant_encoder)?;
                        #encode_t_encode(&#tag, #ctx_var, #tag_encoder)?;

                        let #encoder_var = #variant_encoder_t::encode_value(#variant_encoder)?;
                        #encode;
                        #result_ok(())
                    })?
                }};
            }
        }
        Some(enum_tagging) => match enum_tagging {
            EnumTagging::Internal {
                tag: EnumTag {
                    value: field_tag, ..
                },
            } => {
                let tag = &v.tag;
                let decls = tests.iter().map(|t| &t.decl);

                encode = quote! {{
                    #encoder_t::encode_struct_fn(#encoder_var, 0, |#encoder_var| {
                        #struct_encoder_t::insert_field(#encoder_var, #field_tag, #tag)?;
                        #(#decls)*
                        #(#encoders)*
                        #result_ok(())
                    })?
                }};
            }
            EnumTagging::Adjacent {
                tag: EnumTag {
                    value: field_tag, ..
                },
                content,
            } => {
                let encode_t_encode = &b.encode_t_encode;

                let tag = &v.tag;

                let decls = tests.iter().map(|t| &t.decl);

                let len = length_test(v.st.fields.len(), &tests);
                let struct_encoder = b.cx.ident("struct_encoder");
                let content_struct = b.cx.ident("content_struct");
                let pair = b.cx.ident("pair");
                let content_tag = b.cx.ident("content_tag");

                encode = quote! {{
                    #encoder_t::encode_struct_fn(#encoder_var, 2, |#struct_encoder| {
                        #struct_encoder_t::insert_field(#struct_encoder, &#field_tag, #tag)?;

                        #struct_encoder_t::encode_field_fn(#struct_encoder, |#pair| {
                            let #content_tag = #struct_field_encoder_t::encode_field_name(#pair)?;
                            #encode_t_encode(&#content, #ctx_var, #content_tag)?;

                            let #content_struct = #struct_field_encoder_t::encode_field_value(#pair)?;

                            #encoder_t::encode_struct_fn(#content_struct, #len, |#encoder_var| {
                                #(#decls)*
                                #(#encoders)*
                                #result_ok(())
                            })?;

                            #result_ok(())
                        })?;

                        #result_ok(())
                    })?
                }};
            }
        },
    }

    let pattern = syn::PatStruct {
        attrs: Vec::new(),
        qself: None,
        path: v.st.path.clone(),
        brace_token: syn::token::Brace::default(),
        fields: v.patterns.clone(),
        rest: None,
    };

    if cx.trace {
        let output_var = b.cx.ident("output");

        let tag = en.name_format(&v.tag);
        let enter = quote!(#context_t::enter_variant(#ctx_var, #type_name, #tag));
        let leave = quote!(#context_t::leave_variant(#ctx_var));

        encode = quote! {{
            #enter;
            let #output_var = #encode;
            #leave;
            #output_var
        }};
    }

    Ok((pattern, encode))
}

fn length_test(count: usize, tests: &[FieldTest]) -> Punctuated<TokenStream, Token![+]> {
    let mut punctuated = Punctuated::<_, Token![+]>::new();
    let count = count.saturating_sub(tests.len());
    punctuated.push(quote!(#count));

    for FieldTest { var, .. } in tests {
        punctuated.push(quote!(if #var { 1 } else { 0 }))
    }

    punctuated
}
