use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};

use crate::build::{Build, BuildData, EnumBuild, FieldBuild, StructBuild, VariantBuild};
use crate::expander::Result;
use crate::internals::attr::{EnumTagging, Packing};
use crate::internals::symbol::*;

pub(crate) fn expand_encode_entry(e: Build<'_>) -> Result<TokenStream> {
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

    let (impl_generics, mode_ident, where_clause) = e
        .expansion
        .as_impl_generics(e.input.generics.clone(), &e.tokens);

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

/// Encode a transparent element.
fn encode_transparent(
    e: &Build<'_>,
    var: &syn::Ident,
    span: Span,
    fields: &[FieldBuild],
) -> Result<TokenStream> {
    let f = match fields {
        [f] => f,
        _ => {
            e.transparent_diagnostics(span, fields);
            return Err(());
        }
    };

    let accessor = match &f.ident {
        Some(ident) => quote_spanned!(f.span => &self.#ident),
        None => quote_spanned!(f.span => &self.0),
    };

    let (span, encode_path) = &f.encode_path;

    Ok(quote_spanned! {
        *span => #encode_path(#accessor, #var)
    })
}

/// Encode a transparent element.
fn encode_transparent_variant(
    e: &Build<'_>,
    var: &syn::Ident,
    span: Span,
    fields: &[FieldBuild],
) -> Result<(TokenStream, Vec<TokenStream>)> {
    let f = match fields {
        [f] => f,
        _ => {
            e.transparent_diagnostics(span, fields);
            return Err(());
        }
    };

    let (span, encode_path) = &f.encode_path;

    let encode = quote_spanned! {
        *span => #encode_path(this, #var)
    };

    let accessor = match &f.ident {
        Some(ident) => quote_spanned!(f.span => #ident: this),
        None => quote_spanned!(f.span => 0: this),
    };

    Ok((encode, vec![accessor]))
}

/// Encode a struct.
fn encode_struct(e: &Build<'_>, var: &syn::Ident, st: &StructBuild<'_>) -> Result<TokenStream> {
    let fields = encode_fields(e, var, &st.fields)?;

    match st.packing {
        Packing::Transparent => encode_transparent(e, var, e.input.ident.span(), &st.fields),
        Packing::Tagged => {
            let encoder_t = &e.tokens.encoder_t;
            let pairs_encoder_t = &e.tokens.pairs_encoder_t;

            let len = length_test(st.fields.len(), &fields.tests);
            let decls = fields.test_decls();
            let encoders = &fields.encoders;

            Ok(quote! {
                #(#decls)*
                let mut #var = #encoder_t::encode_struct(#var, #len)?;
                #(#encoders)*
                #pairs_encoder_t::end(#var)
            })
        }
        Packing::Packed => {
            let encoder_t = &e.tokens.encoder_t;
            let sequence_encoder_t = &e.tokens.sequence_encoder_t;
            let decls = fields.tests.iter().map(|t| &t.decl);
            let encoders = &fields.encoders;

            Ok(quote! {{
                let mut pack = #encoder_t::encode_pack(#var)?;
                #(#decls)*
                #(#encoders)*
                #sequence_encoder_t::end(pack)
            }})
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

            tests.push(FieldTest { decl, var })
        }

        encoders.push(encode);
    }

    Ok(EncodedFields { encoders, tests })
}

/// Encode an internally tagged enum.
fn encode_enum(e: &Build<'_>, var: &syn::Ident, data: &EnumBuild<'_>) -> Result<TokenStream> {
    if let Some(&(span, Packing::Transparent)) = data.packing_span {
        e.cx.error_span(
            span,
            format!(
                "#[{}({})] cannot be used to encode enums",
                ATTR, TRANSPARENT
            ),
        );
        return Err(());
    }

    let mut variants = Vec::with_capacity(data.variants.len());

    for variant in &data.variants {
        if let Ok((pattern, encode)) = encode_variant(e, var, variant) {
            variants.push(quote!(#pattern => { #encode }));
        }
    }

    // Special case: uninhabitable types.
    Ok(if variants.is_empty() {
        quote! {
            match *self {}
        }
    } else {
        quote! {
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
    v: &VariantBuild,
) -> Result<(TokenStream, TokenStream)> {
    if let Packing::Transparent = v.packing {
        let (encode, patterns) = encode_transparent_variant(e, var, v.span, &v.fields)?;
        let encode = encode_variant_container(e, var, v, encode)?;
        let path = &v.path;
        return Ok((quote!(#path { #(#patterns),* }), encode));
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

    if let Some(EnumTagging::Internal(field_tag)) = v.enum_tagging {
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

    let encoder_t = &e.tokens.encoder_t;
    let pairs_encoder_t = &e.tokens.pairs_encoder_t;

    let len = length_test(v.fields.len(), &fields.tests);

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
    let access = &f.access;

    let decl = quote_spanned! {
        *skip_span =>
        let #test = !#skip_encoding_if_path(#access);
    };

    Some((decl, test))
}

/// Encode a field.
fn encode_field(e: &Build<'_>, var: &syn::Ident, f: &FieldBuild) -> Result<TokenStream> {
    let (span, encode_path) = &f.encode_path;
    let access = &f.access;

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

fn length_test(count: usize, tests: &[FieldTest]) -> TokenStream {
    if tests.is_empty() {
        quote!(#count)
    } else {
        let count = count.saturating_sub(tests.len());
        let tests = tests
            .iter()
            .map(|FieldTest { var, .. }| quote!(if #var { 1 } else { 0 }));
        quote!(#count + #(#tests)+*)
    }
}
