use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::Token;

use crate::expander::Result;
use crate::internals::attr::{EnumTag, EnumTagging, Packing};
use crate::internals::build::{Build, BuildData, EnumBuild, StructBuild, VariantBuild};

pub(crate) fn expand_encode_entry(e: Build<'_>) -> Result<TokenStream> {
    e.validate_encode()?;

    let type_ident = &e.input.ident;

    let encoder_var = e.cx.ident("encoder");
    let ctx_var = e.cx.ident("ctx");
    let buf_lt = e.cx.lifetime("'buf");
    let c_param = e.cx.ident("C");
    let e_param = e.cx.ident("E");

    let body = match &e.data {
        BuildData::Struct(st) => encode_struct(&e, st, &ctx_var, &encoder_var, true)?,
        BuildData::Enum(en) => encode_enum(&e, en, &ctx_var, &encoder_var, true)?,
    };

    if e.cx.has_errors() {
        return Err(());
    }

    let encode_t = &e.tokens.encode_t;
    let context_t = &e.tokens.context_t;
    let encoder_t = &e.tokens.encoder_t;
    let core_result = &e.tokens.core_result;

    let (impl_generics, mode_ident, mut where_clause) = e
        .expansion
        .as_impl_generics(e.input.generics.clone(), e.tokens);

    if !e.bounds.is_empty() {
        let where_clause = where_clause.get_or_insert_with(|| syn::WhereClause {
            where_token: <Token![where]>::default(),
            predicates: Default::default(),
        });

        where_clause
            .predicates
            .extend(e.bounds.iter().map(|(_, v)| v.clone()));
    }

    let type_generics = &e.input.generics;

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #encode_t<#mode_ident> for #type_ident #type_generics #where_clause {
            #[inline]
            fn encode<#buf_lt, #c_param, #e_param>(&self, #ctx_var: &mut #c_param, #encoder_var: #e_param) -> #core_result<<#e_param as #encoder_t>::Ok, <#c_param as #context_t<#buf_lt>>::Error>
            where
                #c_param: #context_t<#buf_lt, Input = <#e_param as #encoder_t>::Error>,
                #e_param: #encoder_t
            {
                #body
            }
        }
    })
}

/// Encode a struct.
fn encode_struct(
    e: &Build<'_>,
    st: &StructBuild<'_>,
    ctx_var: &syn::Ident,
    encoder_var: &syn::Ident,
    trace: bool,
) -> Result<TokenStream> {
    let pack_var = e.cx.ident("pack");
    let output_var = e.cx.ident("output");

    let (encoders, tests) = encode_fields(e, st, ctx_var, encoder_var, &pack_var, trace)?;

    let context_t = &e.tokens.context_t;
    let result_ok = &e.tokens.result_ok;
    let type_name = &st.name;

    let enter = trace.then(|| quote!(#context_t::trace_enter_struct(#ctx_var, #type_name);));
    let leave = trace.then(|| quote!(#context_t::trace_leave_struct(#ctx_var);));

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

            let encoder_t = &e.tokens.encoder_t;
            let pairs_encoder_t = &e.tokens.pairs_encoder_t;

            encode = quote! {{
                #enter
                #(#decls)*
                let mut #encoder_var = #encoder_t::encode_struct(#encoder_var, #ctx_var, #len)?;
                #(#encoders)*
                let #output_var = #pairs_encoder_t::end(#encoder_var, #ctx_var)?;
                #leave
                #output_var
            }};
        }
        Packing::Packed => {
            let decls = tests.iter().map(|t| &t.decl);

            let encoder_t = &e.tokens.encoder_t;
            let sequence_encoder_t = &e.tokens.sequence_encoder_t;

            encode = quote! {{
                #enter
                #(#decls)*
                let mut #pack_var = #encoder_t::encode_pack(#encoder_var, #ctx_var)?;
                #(#encoders)*
                let #output_var = #sequence_encoder_t::end(#pack_var, #ctx_var)?;
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

fn encode_fields(
    e: &Build<'_>,
    st: &StructBuild<'_>,
    ctx_var: &syn::Ident,
    encoder_var: &syn::Ident,
    pack_var: &syn::Ident,
    trace: bool,
) -> Result<(Vec<TokenStream>, Vec<FieldTest>)> {
    let pair_encoder_t = &e.tokens.pair_encoder_t;
    let pairs_encoder_t = &e.tokens.pairs_encoder_t;
    let encode_t_encode = &e.encode_t_encode;
    let sequence_encoder_t = &e.tokens.sequence_encoder_t;
    let context_t = &e.tokens.context_t;

    let sequence_decoder_next_var = e.cx.ident("sequence_decoder_next");
    let pair_encoder_var = e.cx.ident("pair_encoder");
    let field_encoder_var = e.cx.ident("field_encoder");
    let value_encoder_var = e.cx.ident("value_encoder");
    let field_marker = e.cx.ident("field_marker");

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

                trace.then(|| {
                    let tag = st.name_format(tag);

                    quote! {
                        let #field_marker = #context_t::trace_enter_named_field(#ctx_var, #field_name, #tag);
                    }
                })
            }
            syn::Member::Unnamed(index) => {
                let index = index.index;
                trace.then(|| {
                    let tag = st.name_format(tag); quote! {
                    let #field_marker = #context_t::trace_enter_unnamed_field(#ctx_var, #index, #tag);
                }})
            }
        };

        let leave = trace.then(|| quote!(#context_t::trace_leave_field(#ctx_var, #field_marker);));

        match f.packing {
            Packing::Tagged | Packing::Transparent => {
                encode = quote! {
                    #enter
                    let mut #pair_encoder_var = #pairs_encoder_t::next(&mut #encoder_var, #ctx_var)?;
                    let #field_encoder_var = #pair_encoder_t::first(&mut #pair_encoder_var, #ctx_var)?;
                    #encode_t_encode(&#tag, #ctx_var, #field_encoder_var)?;
                    let #value_encoder_var = #pair_encoder_t::second(&mut #pair_encoder_var, #ctx_var)?;
                    #encode_path(#access, #ctx_var, #value_encoder_var)?;
                    #pair_encoder_t::end(#pair_encoder_var, #ctx_var)?;
                    #leave
                };
            }
            Packing::Packed => {
                encode = quote! {
                    #enter
                    let #sequence_decoder_next_var = #sequence_encoder_t::next(&mut #pack_var, #ctx_var)?;
                    #encode_path(#access, #ctx_var, #sequence_decoder_next_var)?;
                    #leave
                };
            }
        };

        if let Some((_, skip_encoding_if_path)) = f.skip_encoding_if.as_ref() {
            let var = syn::Ident::new(&format!("t{}", f.index), f.span);

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
fn encode_enum(
    e: &Build<'_>,
    en: &EnumBuild<'_>,
    ctx_var: &syn::Ident,
    encoder_var: &syn::Ident,
    trace: bool,
) -> Result<TokenStream> {
    let context_t = &e.tokens.context_t;
    let result_err = &e.tokens.result_err;
    let result_ok = &e.tokens.result_ok;
    let type_name = en.name;

    if let Some(&(span, Packing::Transparent)) = en.packing_span {
        e.encode_transparent_enum_diagnostics(span);
        return Err(());
    }

    let mut variants = Vec::with_capacity(en.variants.len());

    for v in &en.variants {
        let Ok((pattern, encode)) = encode_variant(e, en, v, ctx_var, encoder_var, trace) else {
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
    b: &Build<'_>,
    en: &EnumBuild,
    v: &VariantBuild,
    ctx_var: &syn::Ident,
    encoder_var: &syn::Ident,
    trace: bool,
) -> Result<(syn::PatStruct, TokenStream)> {
    let pack_var = b.cx.ident("pack");

    let (encoders, tests) = encode_fields(b, &v.st, ctx_var, encoder_var, &pack_var, true)?;

    let encoder_t = &b.tokens.encoder_t;
    let pair_encoder_t = &b.tokens.pair_encoder_t;
    let variant_encoder_t = &b.tokens.variant_encoder_t;
    let pairs_encoder_t = &b.tokens.pairs_encoder_t;
    let sequence_encoder_t = &b.tokens.sequence_encoder_t;
    let context_t = &b.tokens.context_t;
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
                        let mut #pack_var = #encoder_t::encode_pack(#encoder_var, #ctx_var)?;
                        #(#decls)*
                        #(#encoders)*
                        #sequence_encoder_t::end(#pack_var, #ctx_var)?
                    }};
                }
                Packing::Tagged => {
                    let decls = tests.iter().map(|t| &t.decl);
                    let len = length_test(v.st.fields.len(), &tests);

                    encode = quote! {{
                        let mut #encoder_var = #encoder_t::encode_struct(#encoder_var, #ctx_var, #len)?;
                        #(#decls)*
                        #(#encoders)*
                        #pairs_encoder_t::end(#encoder_var, #ctx_var)?
                    }};
                }
            }

            if let Packing::Tagged = en.enum_packing {
                let encode_t_encode = &b.encode_t_encode;
                let tag = &v.tag;
                let variant_encoder = b.cx.ident("variant_encoder");
                let tag_encoder = b.cx.ident("tag_encoder");

                encode = quote! {{
                    let mut #variant_encoder = #encoder_t::encode_variant(#encoder_var, #ctx_var)?;

                    let #tag_encoder = #variant_encoder_t::tag(&mut #variant_encoder, #ctx_var)?;
                    #encode_t_encode(&#tag, #ctx_var, #tag_encoder)?;

                    let #encoder_var = #variant_encoder_t::variant(&mut #variant_encoder, #ctx_var)?;
                    #encode;
                    #variant_encoder_t::end(#variant_encoder, #ctx_var)?
                }};
            }
        }
        Some(enum_tagging) => match enum_tagging {
            EnumTagging::Internal {
                tag: EnumTag {
                    value: field_tag, ..
                },
            } => {
                let mode_ident = b.mode_ident.as_path();
                let tag = &v.tag;
                let decls = tests.iter().map(|t| &t.decl);

                encode = quote! {{
                    let mut #encoder_var = #encoder_t::encode_struct(#encoder_var, #ctx_var, 0)?;
                    #pairs_encoder_t::insert::<#mode_ident, _, _, _>(&mut #encoder_var, #ctx_var, #field_tag, #tag)?;
                    #(#decls)*
                    #(#encoders)*
                    #pairs_encoder_t::end(#encoder_var, #ctx_var)?
                }};
            }
            EnumTagging::Adjacent {
                tag: EnumTag {
                    value: field_tag, ..
                },
                content,
            } => {
                let mode_ident = b.mode_ident.as_path();
                let encode_t_encode = &b.encode_t_encode;

                let tag = &v.tag;

                let decls = tests.iter().map(|t| &t.decl);

                let len = length_test(v.st.fields.len(), &tests);
                let struct_encoder = b.cx.ident("struct_encoder");
                let content_struct = b.cx.ident("content_struct");
                let pair = b.cx.ident("pair");
                let content_tag = b.cx.ident("content_tag");

                encode = quote! {{
                    let mut #struct_encoder = #encoder_t::encode_struct(#encoder_var, #ctx_var, 2)?;
                    #pairs_encoder_t::insert::<#mode_ident, _, _, _>(&mut #struct_encoder, #ctx_var, &#field_tag, #tag)?;
                    let mut #pair = #pairs_encoder_t::next(&mut #struct_encoder, #ctx_var)?;
                    let #content_tag = #pair_encoder_t::first(&mut #pair, #ctx_var)?;
                    #encode_t_encode(&#content, #ctx_var, #content_tag)?;

                    let #content_struct = #pair_encoder_t::second(&mut #pair, #ctx_var)?;
                    let mut #encoder_var = #encoder_t::encode_struct(#content_struct, #ctx_var, #len)?;
                    #(#decls)*
                    #(#encoders)*
                    #pairs_encoder_t::end(#encoder_var, #ctx_var)?;

                    #pair_encoder_t::end(#pair, #ctx_var)?;
                    #pairs_encoder_t::end(#struct_encoder, #ctx_var)?
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

    if trace {
        let output_var = b.cx.ident("output");
        let trace_variant_var = b.cx.ident("trace_variant");

        let tag = en.name_format(&v.tag);
        let enter = quote!(#context_t::trace_enter_variant(#ctx_var, #type_name, #tag));
        let leave = quote!(#context_t::trace_leave_variant(#ctx_var, #trace_variant_var));

        encode = quote! {{
            let #trace_variant_var = #enter;
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
