use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::spanned::Spanned;

use crate::expander::{Result, TagMethod};
use crate::internals::attr::{EnumTagging, Packing};
use crate::internals::build::{Build, BuildData, EnumBuild, FieldBuild, StructBuild};

pub(crate) fn expand_decode_entry(e: Build<'_>) -> Result<TokenStream> {
    let span = e.input.ident.span();

    let root_decoder_var = syn::Ident::new("root_decoder", Span::call_site());

    let body = match &e.data {
        BuildData::Struct(data) => decode_struct(&e, &root_decoder_var, data)?,
        BuildData::Enum(data) => decode_enum(&e, &root_decoder_var, data)?,
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
        let lt = syn::LifetimeDef::new(syn::Lifetime::new("'de", e.input.span()));
        (lt, false)
    };

    if !exists {
        impl_generics.params.push(lt.clone().into());
    }

    let decode_t = &e.tokens.decode_t;
    let decoder_t = &e.tokens.decoder_t;
    let original_generics = &e.input.generics;

    let (impl_generics, mode_ident, where_clause) =
        e.expansion.as_impl_generics(impl_generics, &e.tokens);

    Ok(quote_spanned! {
        span =>
        #[automatically_derived]
        impl #impl_generics #decode_t<#lt, #mode_ident> for #type_ident #original_generics #where_clause {
            #[inline]
            fn decode<D>(#root_decoder_var: D) -> Result<Self, D::Error>
            where
                D: #decoder_t<#lt>
            {
                #body
            }
        }
    })
}

fn decode_struct(e: &Build<'_>, decoder_var: &syn::Ident, st: &StructBuild) -> Result<TokenStream> {
    let body = match st.packing {
        Packing::Tagged => decode_tagged(
            e,
            st.span,
            decoder_var,
            &e.type_name,
            st.tag_type,
            &st.path,
            &st.fields,
            None,
            st.field_tag_method,
        )?,
        Packing::Packed => decode_packed(e, st.span, decoder_var, &st.path, &st.fields)?,
        Packing::Transparent => decode_transparent(e, st.span, decoder_var, &st.path, &st.fields)?,
    };

    Ok(quote_spanned! {
        st.span =>
        Ok({ #body })
    })
}

fn decode_enum(
    e: &Build<'_>,
    root_decoder_var: &syn::Ident,
    en: &EnumBuild,
) -> Result<TokenStream> {
    if let Some(&(span, Packing::Packed)) = en.packing_span {
        e.decode_packed_enum_diagnostics(span);
        return Err(());
    }

    let error_t = &e.tokens.error_t;
    let type_name = &e.type_name;

    // Trying to decode an uninhabitable type.
    if en.variants.is_empty() {
        return Ok(quote_spanned! {
            en.span =>
            Err(<D::Error as #error_t>::uninhabitable(#type_name))
        });
    }

    if let Some(..) = en.enum_tagging {}

    let decoder_t = &e.tokens.decoder_t;
    let variant_tag = syn::Ident::new("variant_tag", en.span);
    let variant_decoder_var = syn::Ident::new("variant_decoder", en.span);
    let body_decoder_var = syn::Ident::new("body_decoder", en.span);

    let tag_visitor_output = syn::Ident::new("VariantTagVisitorOutput", en.span);
    let mut outputs = Vec::with_capacity(en.variants.len());
    let mut patterns = Vec::with_capacity(en.variants.len());

    for v in en.variants.iter() {
        // Default variants are specially picked when decoding.
        if v.is_default {
            continue;
        }

        let decode = match v.packing {
            Packing::Tagged => decode_tagged(
                e,
                v.span,
                &body_decoder_var,
                &v.name,
                v.tag_type,
                &v.path,
                &v.fields,
                Some(&variant_tag),
                v.field_tag_method,
            )?,
            Packing::Packed => decode_packed(e, v.span, &body_decoder_var, &v.path, &v.fields)?,
            Packing::Transparent => {
                decode_transparent(e, v.span, &body_decoder_var, &v.path, &v.fields)?
            }
        };

        let (output_tag, output) = handle_indirect_output_tag(
            v.span,
            v.index,
            en.variant_tag_method,
            &v.tag,
            &tag_visitor_output,
        );

        outputs.extend(output);
        patterns.push((v.span, output_tag, decode));
    }

    let fallback = match en.fallback {
        Some(ident) => {
            let variant_decoder_t = &e.tokens.variant_decoder_t;

            quote_spanned! {
                en.span =>
                if !#variant_decoder_t::skip_variant(&mut #variant_decoder_var)? {
                    return Err(<D::Error as #error_t>::invalid_variant_tag(#type_name, tag));
                }

                #variant_decoder_t::end(#variant_decoder_var)?;
                Self::#ident {}
            }
        }
        None => quote_spanned! {
            en.span =>
            return Err(<D::Error as #error_t>::invalid_variant_tag(#type_name, tag));
        },
    };

    let (decode_tag, unsupported_pattern, output_enum) = handle_tag_decode(
        e,
        en.span,
        &variant_decoder_var,
        en.variant_tag_method,
        &tag_visitor_output,
        &outputs,
    )?;

    let tag_type = en
        .tag_type
        .as_ref()
        .map(|(_, ty)| quote_spanned!(ty.span() => : #ty));

    if let Some(EnumTagging::Internal(field_tag)) = en.enum_tagging {
        let mode_ident = &e.mode_ident;
        let pair_decoder_t = &e.tokens.pair_decoder_t;
        let pairs_decoder_t = &e.tokens.pairs_decoder_t;
        let as_decoder_t = &e.tokens.as_decoder_t;

        let patterns = patterns.into_iter().map(|(span, tag, output)| {
            quote_spanned! {
                span =>
                #tag => {
                    let #body_decoder_var = #as_decoder_t::as_decoder(&buffer)?;
                    #output
                }
            }
        });

        return Ok(quote_spanned! {
            en.span => {
                let buffer = #decoder_t::decode_buffer::<#mode_ident>(#root_decoder_var)?;

                let decoder = #as_decoder_t::as_decoder(&buffer)?;
                let mut st = #decoder_t::decode_map(decoder)?;

                let discriminator = loop {
                    let mut entry = match #pairs_decoder_t::next(&mut st)? {
                        Some(entry) => entry,
                        None => break None,
                    };

                    let decoder = #pair_decoder_t::first(&mut entry)?;
                    let found = #decoder_t::decode_string(decoder, musli::utils::visit_string_fn(|string| {
                        Ok(string == #field_tag)
                    }))?;

                    if found {
                        break Some(#pair_decoder_t::second(entry)?);
                    }

                    #pair_decoder_t::skip_second(entry)?;
                };

                let #variant_decoder_var = match discriminator {
                    Some(decoder) => decoder,
                    None => return Err(<D::Error as #error_t>::missing_variant_field(#type_name, #field_tag)),
                };

                #output_enum
                let #variant_tag #tag_type = #decode_tag;

                Ok(match #variant_tag {
                    #(#patterns,)*
                    #unsupported_pattern => {
                        #fallback
                    }
                })
            }
        });
    }

    let patterns = patterns.into_iter().map(|(span, tag, output)| {
        let variant_decoder_t = &e.tokens.variant_decoder_t;

        quote_spanned! {
            span =>
            #tag => {
                let output = {
                    let #body_decoder_var = #variant_decoder_t::variant(&mut #variant_decoder_var)?;
                    #output
                };

                #variant_decoder_t::end(#variant_decoder_var)?;
                output
            }
        }
    });

    let decode_path = &e.tokens.variant_decoder_t_tag;

    Ok(quote_spanned! {
        en.span =>
        let mut #variant_decoder_var = #decoder_t::decode_variant(#root_decoder_var)?;
        #output_enum

        let #variant_tag #tag_type = {
            let mut #variant_decoder_var = #decode_path(&mut #variant_decoder_var)?;
            #decode_tag
        };

        Ok(match #variant_tag {
            #(#patterns,)*
            #unsupported_pattern => {
                #fallback
            }
        })
    })
}

/// Decode something tagged.
///
/// If `variant_name` is specified it implies that a tagged enum is being
/// decoded.
fn decode_tagged(
    e: &Build<'_>,
    span: Span,
    parent_decoder_var: &syn::Ident,
    type_name: &syn::LitStr,
    tag_type: Option<&(Span, syn::Type)>,
    path: &syn::Path,
    fields: &[FieldBuild],
    variant_tag: Option<&syn::Ident>,
    field_tag_method: TagMethod,
) -> Result<TokenStream> {
    let struct_decoder_var = syn::Ident::new("struct_decoder", span);

    let decoder_t = &e.tokens.decoder_t;
    let error_t = &e.tokens.error_t;
    let default_function = &e.tokens.default_function;
    let pairs_decoder_t = &e.tokens.pairs_decoder_t;

    let fields_len = fields.len();
    let mut decls = Vec::with_capacity(fields_len);
    let mut patterns = Vec::with_capacity(fields_len);
    let mut assigns = Vec::with_capacity(fields_len);

    let tag_visitor_output = syn::Ident::new("TagVisitorOutput", e.input.ident.span());
    let mut outputs = Vec::with_capacity(fields_len);

    for f in fields {
        let tag = &f.tag;
        let access = &f.field_access;
        let (span, decode_path) = &f.decode_path;
        let var = syn::Ident::new(&format!("v{}", f.index), *span);

        decls.push(quote_spanned!(*span => let mut #var = None;));
        let decode = quote_spanned!(*span => #var = Some(#decode_path(#struct_decoder_var)?));

        let (output_tag, output) =
            handle_indirect_output_tag(*span, f.index, field_tag_method, &tag, &tag_visitor_output);

        outputs.extend(output);
        patterns.push((f.span, output_tag, decode));

        let fallback = if let Some(span) = f.default_attr {
            quote_spanned!(span => #default_function())
        } else {
            quote_spanned! {
                f.span =>
                return Err(<D::Error as #error_t>::expected_tag(#type_name, #tag))
            }
        };

        assigns.push(quote_spanned! {
            f.span =>
            #access: match #var {
                Some(#var) => #var,
                None => #fallback,
            }
        });
    }

    let (decode_tag, unsupported_pattern, output_enum) = handle_tag_decode(
        e,
        span,
        &struct_decoder_var,
        field_tag_method,
        &tag_visitor_output,
        &outputs,
    )?;

    let pair_decoder_t = &e.tokens.pair_decoder_t;

    let patterns = patterns
        .into_iter()
        .map(|(span, tag, decode)| {
            quote_spanned! {
                span =>
                #tag => {
                    let #struct_decoder_var = #pair_decoder_t::second(#struct_decoder_var)?;
                    #decode;
                }
            }
        })
        .collect::<Vec<_>>();

    let skip_field = quote_spanned! {
        span =>
        #pair_decoder_t::skip_second(#struct_decoder_var)?
    };

    let unsupported = match variant_tag {
        Some(variant_tag) => quote_spanned! {
            span =>
            <D::Error as #error_t>::invalid_variant_field_tag(#type_name, #variant_tag, tag)
        },
        None => quote_spanned! {
            span =>
            <D::Error as #error_t>::invalid_field_tag(#type_name, tag)
        },
    };

    let tag_type = tag_type
        .as_ref()
        .map(|(_, ty)| quote_spanned!(span => : #ty));

    let body = if patterns.is_empty() {
        quote_spanned! {
            span =>
            if !#skip_field {
                return Err(#unsupported);
            }
        }
    } else {
        quote_spanned! {
            span =>
            match tag {
                #(#patterns,)*
                #unsupported_pattern => {
                    if !#skip_field {
                        return Err(#unsupported);
                    }
                },
            }
        }
    };

    let decode_path = &e.tokens.pair_decoder_t_first;

    Ok(quote_spanned! {
        span =>
        #(#decls)*
        #output_enum
        let mut type_decoder = #decoder_t::decode_struct(#parent_decoder_var, #fields_len)?;

        while let Some(mut #struct_decoder_var) = #pairs_decoder_t::next(&mut type_decoder)? {
            let tag #tag_type = {
                let #struct_decoder_var = #decode_path(&mut #struct_decoder_var)?;
                #decode_tag
            };

            #body
        }

        #pairs_decoder_t::end(type_decoder)?;
        #path { #(#assigns),* }
    })
}

/// Decode a transparent value.
fn decode_transparent(
    e: &Build<'_>,
    span: Span,
    decoder_var: &syn::Ident,
    path: &syn::Path,
    fields: &[FieldBuild],
) -> Result<TokenStream> {
    let f = match fields {
        [f] => f,
        _ => {
            e.transparent_diagnostics(span, fields);
            return Err(());
        }
    };

    let (span, decode_path) = &f.decode_path;
    let access = &f.field_access;

    Ok(quote_spanned! {
        *span =>
        #path {
            #access: #decode_path(#decoder_var)?
        }
    })
}

/// Decode something packed.
fn decode_packed(
    e: &Build<'_>,
    span: Span,
    decoder_var: &syn::Ident,
    path: &syn::Path,
    fields: &[FieldBuild],
) -> Result<TokenStream> {
    let decoder_t = &e.tokens.decoder_t;
    let pack_decoder_t = &e.tokens.pack_decoder_t;

    let mut assign = Vec::new();

    for f in fields {
        if let Some(span) = f.default_attr {
            e.packed_default_diagnostics(span);
        }

        let (_, decode_path) = &f.decode_path;
        let access = &f.field_access;

        assign.push(quote_spanned! {
            f.span => #access: {
                let field_decoder = #pack_decoder_t::next(&mut unpack)?;
                #decode_path(field_decoder)?
            }
        });
    }

    if assign.is_empty() {
        Ok(quote_spanned!(span => #path {}))
    } else {
        Ok(quote_spanned! {
            span =>
            let mut unpack = #decoder_t::decode_pack(#decoder_var)?;
            let output = #path { #(#assign),* };
            #pack_decoder_t::end(unpack)?;
            output
        })
    }
}

/// Handle tag decoding.
fn handle_tag_decode(
    e: &Build<'_>,
    span: Span,
    thing_decoder_var: &syn::Ident,
    tag_method: TagMethod,
    output: &syn::Ident,
    outputs: &[IndirectOutput],
) -> Result<(TokenStream, TokenStream, Option<TokenStream>)> {
    match tag_method {
        TagMethod::String => {
            let (decode_tag, output_enum) =
                string_variant_tag_decode(e, span, thing_decoder_var, output, outputs)?;

            Ok((
                decode_tag,
                quote_spanned!(span => #output::Err(tag)),
                Some(output_enum),
            ))
        }
        TagMethod::Index => {
            let decode_t_decode = &e.decode_t_decode;

            let decode_tag = quote_spanned! {
                span => #decode_t_decode(#thing_decoder_var)?
            };

            Ok((decode_tag, quote_spanned!(span => tag), None))
        }
    }
}

/// Output type used when indirectly encoding a variant or field as type which
/// might require special handling. Like a string.
pub(crate) struct IndirectOutput<'a> {
    span: Span,
    /// The path of the variant this output should generate.
    path: syn::Path,
    /// The identified of the variant this path generates.
    variant: syn::Ident,
    /// The tag this variant corresponds to.
    tag: &'a syn::Expr,
}

impl IndirectOutput<'_> {
    /// Generate the pattern for this output.
    pub(crate) fn pattern(&self) -> TokenStream {
        let tag = self.tag;
        let path = &self.path;
        quote_spanned!(self.span => #tag => #path)
    }
}

fn handle_indirect_output_tag<'a>(
    span: Span,
    index: usize,
    tag_method: TagMethod,
    tag: &'a syn::Expr,
    tag_visitor_output: &syn::Ident,
) -> (syn::Expr, Option<IndirectOutput<'a>>) {
    match tag_method {
        TagMethod::String => {
            let variant = syn::Ident::new(&format!("Variant{}", index), span);
            let mut path = syn::Path::from(tag_visitor_output.clone());
            path.segments.push(syn::PathSegment::from(variant.clone()));

            let output = IndirectOutput {
                span,
                path: path.clone(),
                variant,
                tag,
            };

            let expr = syn::Expr::Path(syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path,
            });

            (expr, Some(output))
        }
        TagMethod::Index => (tag.clone(), None),
    }
}

fn string_variant_tag_decode(
    e: &Build<'_>,
    span: Span,
    var: &syn::Ident,
    output: &syn::Ident,
    outputs: &[IndirectOutput],
) -> Result<(TokenStream, TokenStream)> {
    let decoder_t = &e.tokens.decoder_t;
    let visit_string_fn = &e.tokens.visit_string_fn;
    let fmt = &e.tokens.fmt;
    let patterns = outputs.iter().map(|o| o.pattern());

    // Declare a tag visitor, allowing string tags to be decoded by
    // decoders that owns the string.
    let decode_tag = quote_spanned! {
        span =>
        #decoder_t::decode_string(#var, #visit_string_fn(|string| {
            Ok::<#output, D::Error>(match string { #(#patterns,)* _ => #output::Err(string.into())})
        }))?
    };

    let variants = outputs.iter().map(|o| &o.variant);

    let patterns = outputs.iter().map(|o| {
        let variant = &o.variant;
        let tag = o.tag;
        quote_spanned!(o.span => #output::#variant => #fmt::Debug::fmt(&#tag, f))
    });

    let output_enum = quote_spanned! {
        span =>
        enum #output {
            #(#variants,)*
            Err(String),
        }

        impl #fmt::Debug for #output {
            #[inline]
            fn fmt(&self, f: &mut #fmt::Formatter<'_>) -> #fmt::Result {
                match self { #(#patterns,)* #output::Err(field) => field.fmt(f) }
            }
        }
    };

    Ok((decode_tag, output_enum))
}
