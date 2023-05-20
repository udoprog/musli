use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::punctuated::Punctuated;
use syn::Token;

use crate::expander::Result;
use crate::internals::attr::{EnumTag, EnumTagging, Packing};
use crate::internals::build::{Build, BuildData, EnumBuild, FieldBuild, StructBuild, VariantBuild};

pub(crate) fn expand_encode_entry(e: Build<'_>) -> Result<TokenStream> {
    e.validate_encode()?;

    let span = e.input.ident.span();

    let type_ident = &e.input.ident;

    let var = syn::Ident::new("__encoder", span);
    let ctx_var = syn::Ident::new("__ctx", span);

    let body = match &e.data {
        BuildData::Struct(data) => encode_struct(&e, &ctx_var, &var, data)?,
        BuildData::Enum(data) => encode_enum(&e, &ctx_var, &var, data)?,
    };

    if e.cx.has_errors() {
        return Err(());
    }

    let encode_t = &e.tokens.encode_t;
    let context_t = &e.tokens.context_t;
    let encoder_t = &e.tokens.encoder_t;

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

    Ok(quote_spanned! {
        span =>
        #[automatically_derived]
        impl #impl_generics #encode_t<#mode_ident> for #type_ident #type_generics #where_clause {
            #[inline]
            fn encode<C, E>(&self, #ctx_var: &mut C, #var: E) -> Result<<E as #encoder_t>::Ok, <C as #context_t>::Error>
            where
                C: #context_t<Input = <E as #encoder_t>::Error>,
                E: #encoder_t
            {
                #body
            }
        }
    })
}

/// Encode a struct.
fn encode_struct(
    e: &Build<'_>,
    ctx_var: &syn::Ident,
    var: &syn::Ident,
    st: &StructBuild<'_>,
) -> Result<TokenStream> {
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
            let (span, encode_path) = &f.encode_path;

            Ok(quote_spanned! {
                *span => #encode_path(#access, #ctx_var, #var)
            })
        }
        Packing::Tagged => {
            let fields = encode_fields(e, ctx_var, var, &st.fields)?;

            let len = length_test(st.fields.len(), &fields.tests);
            let decls = fields.test_decls();
            let encoders = &fields.encoders;

            let encoder_t = &e.tokens.encoder_t;
            let pairs_encoder_t = &e.tokens.pairs_encoder_t;

            Ok(quote_spanned! {
                st.span =>
                #(#decls)*
                let mut #var = #encoder_t::encode_struct(#var, #ctx_var, #len)?;
                #(#encoders)*
                #pairs_encoder_t::end(#var, #ctx_var)
            })
        }
        Packing::Packed => {
            let fields = encode_fields(e, ctx_var, var, &st.fields)?;

            let decls = fields.tests.iter().map(|t| &t.decl);
            let encoders = &fields.encoders;

            let encoder_t = &e.tokens.encoder_t;
            let sequence_encoder_t = &e.tokens.sequence_encoder_t;

            Ok(quote_spanned! {
                st.span => {
                    let mut pack = #encoder_t::encode_pack(#var, #ctx_var)?;
                    #(#decls)*
                    #(#encoders)*
                    #sequence_encoder_t::end(pack, #ctx_var)
                }
            })
        }
    }
}

struct FieldTest {
    decl: TokenStream,
    var: syn::Ident,
}

struct EncodedFields {
    encoders: Vec<TokenStream>,
    tests: Vec<FieldTest>,
}

impl EncodedFields {
    fn test_decls(&self) -> impl Iterator<Item = &TokenStream> {
        self.tests.iter().map(|t| &t.decl)
    }
}

fn encode_fields(
    e: &Build<'_>,
    ctx_var: &syn::Ident,
    var: &syn::Ident,
    fields: &[FieldBuild],
) -> Result<EncodedFields> {
    let mut encoders = Vec::with_capacity(fields.len());
    let mut tests = Vec::with_capacity(fields.len());

    for f in fields {
        let mut encode = encode_field(e, ctx_var, var, f)?;

        if let Some((decl, var)) = do_field_test(f) {
            encode = quote! {
                if #var {
                    #encode
                }
            };

            tests.push(FieldTest { decl, var })
        }

        encoders.push(encode);
    }

    Ok(EncodedFields { encoders, tests })
}

/// Encode an internally tagged enum.
fn encode_enum(
    e: &Build<'_>,
    ctx_var: &syn::Ident,
    var: &syn::Ident,
    en: &EnumBuild<'_>,
) -> Result<TokenStream> {
    if let Some(&(span, Packing::Transparent)) = en.packing_span {
        e.encode_transparent_enum_diagnostics(span);
        return Err(());
    }

    let mut variants = Vec::with_capacity(en.variants.len());

    for v in &en.variants {
        if let Ok((pattern, encode)) = encode_variant(e, ctx_var, var, en, v) {
            variants.push(quote_spanned!(v.span => #pattern => { #encode }));
        }
    }

    // Special case: uninhabitable types.
    Ok(if variants.is_empty() {
        quote_spanned! {
            en.span =>
            match *self {}
        }
    } else {
        quote_spanned! {
            en.span =>
            match self {
                #(#variants),*
            }
        }
    })
}

/// Setup encoding for a single variant. that is externally tagged.
fn encode_variant(
    e: &Build<'_>,
    ctx_var: &syn::Ident,
    var: &syn::Ident,
    en: &EnumBuild,
    v: &VariantBuild,
) -> Result<(TokenStream, TokenStream)> {
    if let Packing::Transparent = v.packing {
        let f = match &v.fields[..] {
            [f] => f,
            _ => {
                e.transparent_diagnostics(v.span, &v.fields);
                return Err(());
            }
        };

        let encode_path = &f.encode_path.1;
        let encode = quote!(#encode_path(this, #ctx_var, #var));
        let encode = encode_variant_container(e, ctx_var, var, v, encode)?;
        let access = &f.field_access;
        let path = &v.path;
        return Ok((quote!(#path { #access: this }), encode));
    }

    let fields = encode_fields(e, ctx_var, var, &v.fields)?;

    if let Packing::Packed = v.packing {
        let encoder_t = &e.tokens.encoder_t;
        let sequence_encoder_t = &e.tokens.sequence_encoder_t;

        let encode = if fields.encoders.is_empty() {
            quote! {
                let pack = #encoder_t::encode_pack(#var, #ctx_var)?;
                #sequence_encoder_t::end(pack, #ctx_var)
            }
        } else {
            let decls = fields.test_decls();
            let encoders = &fields.encoders;

            quote! {
                let mut pack = #encoder_t::encode_pack(#var, #ctx_var)?;
                #(#decls)*
                #(#encoders)*
                #sequence_encoder_t::end(pack, #ctx_var)
            }
        };

        let encode = encode_variant_container(e, ctx_var, var, v, encode)?;
        return Ok((v.constructor(), encode));
    }

    if let Some(enum_tagging) = en.enum_tagging {
        match enum_tagging {
            EnumTagging::Internal {
                tag: EnumTag {
                    value: field_tag, ..
                },
            } => {
                let pairs_encoder_t = &e.tokens.pairs_encoder_t;
                let encoder_t = &e.tokens.encoder_t;
                let mode_ident = e.mode_ident.as_path();

                let tag = &v.tag;

                let decls = fields.test_decls();
                let encoders = &fields.encoders;

                let encode = quote_spanned! {
                    v.span =>
                    let mut #var = #encoder_t::encode_struct(#var, #ctx_var, 0)?;
                    #pairs_encoder_t::insert::<#mode_ident, _, _, _>(&mut #var, #ctx_var, #field_tag, #tag)?;
                    #(#decls)*
                    #(#encoders)*
                    #pairs_encoder_t::end(#var, #ctx_var)
                };

                return Ok((v.constructor(), encode));
            }
            EnumTagging::Adjacent {
                tag: EnumTag {
                    value: field_tag, ..
                },
                content: content_tag,
            } => {
                let pairs_encoder_t = &e.tokens.pairs_encoder_t;
                let pair_encoder_t = &e.tokens.pair_encoder_t;
                let encoder_t = &e.tokens.encoder_t;
                let mode_ident = e.mode_ident.as_path();

                let tag = &v.tag;

                let decls = fields.test_decls();
                let encoders = &fields.encoders;

                let encode_t_encode = &e.encode_t_encode;

                let len = length_test(v.fields.len(), &fields.tests);

                let encode = quote_spanned! {
                    v.span =>
                    let mut struct_encoder = #encoder_t::encode_struct(#var, #ctx_var, 2)?;
                    #pairs_encoder_t::insert::<#mode_ident, _, _, _>(&mut struct_encoder, #ctx_var, &#field_tag, #tag)?;
                    let mut pair = #pairs_encoder_t::next(&mut struct_encoder, #ctx_var)?;
                    let content_tag = #pair_encoder_t::first(&mut pair, #ctx_var)?;
                    #encode_t_encode(&#content_tag, #ctx_var, content_tag)?;

                    {
                        let content_struct = #pair_encoder_t::second(&mut pair, #ctx_var)?;
                        let mut #var = #encoder_t::encode_struct(content_struct, #ctx_var, #len)?;
                        #(#decls)*
                        #(#encoders)*
                        #pairs_encoder_t::end(#var, #ctx_var)?;
                    }

                    #pair_encoder_t::end(pair, #ctx_var)?;
                    #pairs_encoder_t::end(struct_encoder, #ctx_var)
                };

                return Ok((v.constructor(), encode));
            }
        }
    }

    let encoder_t = &e.tokens.encoder_t;
    let pairs_encoder_t = &e.tokens.pairs_encoder_t;

    let len = length_test(v.fields.len(), &fields.tests);

    let encode = if fields.encoders.is_empty() {
        quote_spanned! {
            v.span =>
            let #var = #encoder_t::encode_struct(#var, #ctx_var, #len)?;
            #pairs_encoder_t::end(#var, #ctx_var)
        }
    } else {
        let decls = fields.test_decls();
        let encoders = &fields.encoders;

        quote_spanned! {
            v.span =>
            #(#decls)*
            let mut #var = #encoder_t::encode_struct(#var, #ctx_var, #len)?;
            #(#encoders)*
            #pairs_encoder_t::end(#var, #ctx_var)
        }
    };

    let encode = encode_variant_container(e, ctx_var, var, v, encode)?;
    Ok((v.constructor(), encode))
}

fn encode_variant_container(
    e: &Build<'_>,
    ctx_var: &syn::Ident,
    var: &syn::Ident,
    v: &VariantBuild,
    body: TokenStream,
) -> Result<TokenStream> {
    if let Packing::Tagged = v.enum_packing {
        let encoder_t = &e.tokens.encoder_t;
        let variant_encoder_t = &e.tokens.variant_encoder_t;
        let encode_t_encode = &e.encode_t_encode;
        let tag = &v.tag;

        Ok(quote_spanned! {
            v.span =>
            let mut variant_encoder = #encoder_t::encode_variant(#var, #ctx_var)?;

            let tag_encoder = #variant_encoder_t::tag(&mut variant_encoder, #ctx_var)?;
            #encode_t_encode(&#tag, #ctx_var, tag_encoder)?;

            let #var = #variant_encoder_t::variant(&mut variant_encoder, #ctx_var)?;
            #body?;
            #variant_encoder_t::end(variant_encoder, #ctx_var)
        })
    } else {
        Ok(body)
    }
}

fn do_field_test(f: &FieldBuild) -> Option<(TokenStream, syn::Ident)> {
    let (skip_span, skip_encoding_if_path) = f.skip_encoding_if.as_ref()?;
    let test = syn::Ident::new(&format!("t{}", f.index), f.span);
    let access = &f.self_access;

    let decl = quote_spanned! {
        *skip_span =>
        let #test = !#skip_encoding_if_path(#access);
    };

    Some((decl, test))
}

/// Encode a field.
fn encode_field(
    e: &Build<'_>,
    ctx_var: &syn::Ident,
    var: &syn::Ident,
    f: &FieldBuild,
) -> Result<TokenStream> {
    let encode_path = &f.encode_path.1;
    let access = &f.self_access;

    match f.packing {
        Packing::Tagged | Packing::Transparent => {
            let pair_encoder_t = &e.tokens.pair_encoder_t;
            let pairs_encoder_t = &e.tokens.pairs_encoder_t;
            let encode_t_encode = &e.encode_t_encode;
            let tag = &f.tag;

            Ok(quote! {
                let mut pair_encoder = #pairs_encoder_t::next(&mut #var, #ctx_var)?;
                let field_encoder = #pair_encoder_t::first(&mut pair_encoder, #ctx_var)?;
                #encode_t_encode(&#tag, #ctx_var, field_encoder)?;
                let value_encoder = #pair_encoder_t::second(&mut pair_encoder, #ctx_var)?;
                #encode_path(#access, #ctx_var, value_encoder)?;
                #pair_encoder_t::end(pair_encoder, #ctx_var)?;
            })
        }
        Packing::Packed => {
            let sequence_encoder_t = &e.tokens.sequence_encoder_t;

            Ok(quote! {
                let __seq_next_decoder = #sequence_encoder_t::next(&mut pack, #ctx_var)?;
                #encode_path(#access, #ctx_var, __seq_next_decoder)?;
            })
        }
    }
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
