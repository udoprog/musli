use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;

use crate::expander::Result;
use crate::internals::attr::{EnumTag, EnumTagging, Packing};
use crate::internals::build::{Build, BuildData, EnumBuild, FieldBuild, StructBuild, VariantBuild};

pub(crate) fn expand_encode_entry(e: Build<'_>) -> Result<TokenStream> {
    e.validate_encode()?;

    let span = e.input.ident.span();

    let type_ident = &e.input.ident;

    let var = syn::Ident::new("encoder", span);

    let body = match &e.data {
        BuildData::Struct(data) => encode_struct(&e, &var, data)?,
        BuildData::Enum(data) => encode_enum(&e, &var, data)?,
    };

    if e.cx.has_errors() {
        return Err(());
    }

    let encode_t = &e.tokens.encode_t;
    let encoder_t = &e.tokens.encoder_t;

    let (impl_generics, mode_ident, mut where_clause) = e
        .expansion
        .as_impl_generics(e.input.generics.clone(), &e.tokens);

    if !e.bounds.is_empty() {
        let where_clause = where_clause.get_or_insert_with(|| syn::WhereClause {
            where_token: <syn::Token![where]>::default(),
            predicates: Default::default(),
        });

        where_clause.predicates.extend(e.bounds.iter().cloned());
    }

    let type_generics = &e.input.generics;

    Ok(quote_spanned! {
        span =>
        #[automatically_derived]
        impl #impl_generics #encode_t<#mode_ident> for #type_ident #type_generics #where_clause {
            #[inline]
            fn encode<E>(&self, #var: E) -> Result<E::Ok, E::Error>
            where
                E: #encoder_t
            {
                #body
            }
        }
    })
}

/// Encode a struct.
fn encode_struct(e: &Build<'_>, var: &syn::Ident, st: &StructBuild<'_>) -> Result<TokenStream> {
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
                *span => #encode_path(#access, #var)
            })
        }
        Packing::Tagged => {
            let fields = encode_fields(e, var, &st.fields)?;

            let len = length_test(st.span, st.fields.len(), &fields.tests);
            let decls = fields.test_decls();
            let encoders = &fields.encoders;

            let encoder_t = &e.tokens.encoder_t;
            let pairs_encoder_t = &e.tokens.pairs_encoder_t;

            Ok(quote_spanned! {
                st.span =>
                #(#decls)*
                let mut #var = #encoder_t::encode_struct(#var, #len)?;
                #(#encoders)*
                #pairs_encoder_t::end(#var)
            })
        }
        Packing::Packed => {
            let fields = encode_fields(e, var, &st.fields)?;

            let decls = fields.tests.iter().map(|t| &t.decl);
            let encoders = &fields.encoders;

            let encoder_t = &e.tokens.encoder_t;
            let sequence_encoder_t = &e.tokens.sequence_encoder_t;

            Ok(quote_spanned! {
                st.span => {
                    let mut pack = #encoder_t::encode_pack(#var)?;
                    #(#decls)*
                    #(#encoders)*
                    #sequence_encoder_t::end(pack)
                }
            })
        }
    }
}

struct FieldTest {
    span: Span,
    decl: TokenStream,
    var: syn::Ident,
}

struct EncodedFields {
    encoders: Vec<TokenStream>,
    tests: Vec<FieldTest>,
}

impl EncodedFields {
    fn test_decls<'a>(&'a self) -> impl Iterator<Item = &'a TokenStream> {
        self.tests.iter().map(|t| &t.decl)
    }
}

fn encode_fields(e: &Build<'_>, var: &syn::Ident, fields: &[FieldBuild]) -> Result<EncodedFields> {
    let mut encoders = Vec::with_capacity(fields.len());
    let mut tests = Vec::with_capacity(fields.len());

    for f in fields {
        let mut encode = encode_field(e, var, f)?;

        if let Some((decl, var)) = do_field_test(f) {
            encode = quote_spanned! {
                f.span =>
                if #var {
                    #encode
                }
            };

            tests.push(FieldTest {
                span: f.span,
                decl,
                var,
            })
        }

        encoders.push(encode);
    }

    Ok(EncodedFields { encoders, tests })
}

/// Encode an internally tagged enum.
fn encode_enum(e: &Build<'_>, var: &syn::Ident, en: &EnumBuild<'_>) -> Result<TokenStream> {
    if let Some((span, Packing::Transparent)) = en.packing_span {
        e.encode_transparent_enum_diagnostics(span);
        return Err(());
    }

    let mut variants = Vec::with_capacity(en.variants.len());

    for v in &en.variants {
        if let Ok((pattern, encode)) = encode_variant(e, var, en, v) {
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

        let (span, encode_path) = &f.encode_path;
        let access = &f.field_access;

        let encode = quote_spanned! {
            *span => #encode_path(this, #var)
        };

        let encode = encode_variant_container(e, var, v, encode)?;
        let path = &v.path;
        return Ok((quote_spanned!(v.span => #path { #access: this }), encode));
    }

    let fields = encode_fields(e, var, &v.fields)?;

    if let Packing::Packed = v.packing {
        let encoder_t = &e.tokens.encoder_t;
        let sequence_encoder_t = &e.tokens.sequence_encoder_t;

        let encode = if fields.encoders.is_empty() {
            quote_spanned! {
                v.span =>
                let pack = #encoder_t::encode_pack(#var)?;
                #sequence_encoder_t::end(pack)
            }
        } else {
            let decls = fields.test_decls();
            let encoders = &fields.encoders;

            quote_spanned! {
                v.span =>
                let mut pack = #encoder_t::encode_pack(#var)?;
                #(#decls)*
                #(#encoders)*
                #sequence_encoder_t::end(pack)
            }
        };

        let encode = encode_variant_container(e, var, v, encode)?;
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
                let mode_ident = e.mode_ident;

                let tag = &v.tag;

                let decls = fields.test_decls();
                let encoders = &fields.encoders;

                let encode = quote_spanned! {
                    v.span =>
                    let mut #var = #encoder_t::encode_struct(#var, 0)?;
                    #pairs_encoder_t::insert::<#mode_ident, _, _>(&mut #var, #field_tag, #tag)?;
                    #(#decls)*
                    #(#encoders)*
                    #pairs_encoder_t::end(#var)
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
                let mode_ident = e.mode_ident;

                let tag = &v.tag;

                let decls = fields.test_decls();
                let encoders = &fields.encoders;

                let encode_t_encode = &e.encode_t_encode;

                let len = length_test(v.span, v.fields.len(), &fields.tests);

                let encode = quote_spanned! {
                    v.span =>
                    let mut struct_encoder = #encoder_t::encode_struct(#var, 2)?;
                    #pairs_encoder_t::insert::<#mode_ident, _, _>(&mut struct_encoder, &#field_tag, #tag)?;
                    let mut pair = #pairs_encoder_t::next(&mut struct_encoder)?;
                    let content_tag = #pair_encoder_t::first(&mut pair)?;
                    #encode_t_encode(&#content_tag, content_tag)?;

                    {
                        let content_struct = #pair_encoder_t::second(&mut pair)?;
                        let mut #var = #encoder_t::encode_struct(content_struct, #len)?;
                        #(#decls)*
                        #(#encoders)*
                        #pairs_encoder_t::end(#var)?;
                    }

                    #pair_encoder_t::end(pair)?;
                    #pairs_encoder_t::end(struct_encoder)
                };

                return Ok((v.constructor(), encode));
            }
        }
    }

    let encoder_t = &e.tokens.encoder_t;
    let pairs_encoder_t = &e.tokens.pairs_encoder_t;

    let len = length_test(v.span, v.fields.len(), &fields.tests);

    let encode = if fields.encoders.is_empty() {
        quote_spanned! {
            v.span =>
            let #var = #encoder_t::encode_struct(#var, #len)?;
            #pairs_encoder_t::end(#var)
        }
    } else {
        let decls = fields.test_decls();
        let encoders = &fields.encoders;

        quote_spanned! {
            v.span =>
            #(#decls)*
            let mut #var = #encoder_t::encode_struct(#var, #len)?;
            #(#encoders)*
            #pairs_encoder_t::end(#var)
        }
    };

    let encode = encode_variant_container(e, var, v, encode)?;
    Ok((v.constructor(), encode))
}

fn encode_variant_container(
    e: &Build<'_>,
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
            let mut variant_encoder = #encoder_t::encode_variant(#var)?;

            let tag_encoder = #variant_encoder_t::tag(&mut variant_encoder)?;
            #encode_t_encode(&#tag, tag_encoder)?;

            let #var = #variant_encoder_t::variant(&mut variant_encoder)?;
            #body?;
            #variant_encoder_t::end(variant_encoder)
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
fn encode_field(e: &Build<'_>, var: &syn::Ident, f: &FieldBuild) -> Result<TokenStream> {
    let (span, encode_path) = &f.encode_path;
    let access = &f.self_access;

    match f.packing {
        Packing::Tagged | Packing::Transparent => {
            let pair_encoder_t = &e.tokens.pair_encoder_t;
            let pairs_encoder_t = &e.tokens.pairs_encoder_t;
            let encode_t_encode = &e.encode_t_encode;
            let tag = &f.tag;

            Ok(quote_spanned! {
                *span => {
                    let mut pair_encoder = #pairs_encoder_t::next(&mut #var)?;
                    let field_encoder = #pair_encoder_t::first(&mut pair_encoder)?;
                    #encode_t_encode(&#tag, field_encoder)?;
                    let value_encoder = #pair_encoder_t::second(&mut pair_encoder)?;
                    #encode_path(#access, value_encoder)?;
                    #pair_encoder_t::end(pair_encoder)?;
                }
            })
        }
        Packing::Packed => {
            let sequence_encoder_t = &e.tokens.sequence_encoder_t;

            Ok(quote_spanned! {
                *span =>
                #encode_path(#access, #sequence_encoder_t::next(&mut pack)?)?;
            })
        }
    }
}

fn length_test(span: Span, count: usize, tests: &[FieldTest]) -> TokenStream {
    if tests.is_empty() {
        quote_spanned!(span => #count)
    } else {
        let count = count.saturating_sub(tests.len());
        let tests = tests
            .iter()
            .map(|FieldTest { span, var, .. }| quote_spanned!(*span => if #var { 1 } else { 0 }));
        quote_spanned!(span => #count + #(#tests)+*)
    }
}
