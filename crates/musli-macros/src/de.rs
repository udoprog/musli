use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Token;

use crate::expander::{Result, TagMethod};
use crate::internals::attr::{EnumTag, EnumTagging, Packing};
use crate::internals::build::{Build, BuildData, EnumBuild, FieldBuild, StructBuild};

pub(crate) fn expand_decode_entry(e: Build<'_>) -> Result<TokenStream> {
    e.validate_decode()?;

    let ctx_var = syn::Ident::new("__ctx", Span::call_site());
    let root_decoder_var = syn::Ident::new("__decoder", Span::call_site());

    let body = match &e.data {
        BuildData::Struct(data) => decode_struct(&e, &ctx_var, &root_decoder_var, data)?,
        BuildData::Enum(data) => decode_enum(&e, &ctx_var, &root_decoder_var, data)?,
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

    Ok(quote! {
        #[automatically_derived]
        #[allow(clippy::init_numbered_fields)]
        #[allow(clippy::let_unit_value)]
        impl #impl_generics #decode_t<#lt, #mode_ident> for #type_ident #original_generics #where_clause {
            #[inline]
            fn decode<C, D>(#ctx_var: &mut C, #root_decoder_var: D) -> #core_result<Self, <C as #context_t<<D as #decoder_t<#lt>>::Error>>::Error>
            where
                C: #context_t<<D as #decoder_t<#lt>>::Error>,
                D: #decoder_t<#lt>
            {
                #body
            }
        }
    })
}

fn decode_struct(
    e: &Build<'_>,
    ctx_var: &syn::Ident,
    decoder_var: &syn::Ident,
    st: &StructBuild,
) -> Result<TokenStream> {
    let body = match st.packing {
        Packing::Tagged => decode_tagged(
            e,
            st.span,
            ctx_var,
            decoder_var,
            e.type_name,
            st.tag_type,
            &st.path,
            &st.fields,
            None,
            st.field_tag_method,
        )?,
        Packing::Packed => decode_packed(e, st.span, ctx_var, decoder_var, &st.path, &st.fields)?,
        Packing::Transparent => {
            decode_transparent(e, st.span, ctx_var, decoder_var, &st.path, &st.fields)?
        }
    };

    Ok(quote_spanned! {
        st.span =>
        Ok({ #body })
    })
}

fn decode_enum(
    e: &Build<'_>,
    ctx_var: &syn::Ident,
    root_decoder_var: &syn::Ident,
    en: &EnumBuild,
) -> Result<TokenStream> {
    if let Some(&(span, Packing::Packed)) = en.packing_span {
        e.decode_packed_enum_diagnostics(span);
        return Err(());
    }

    let type_name = &e.type_name;
    let context_t = &e.tokens.context_t;

    // Trying to decode an uninhabitable type.
    if en.variants.is_empty() {
        return Ok(quote_spanned! {
            en.span =>
            Err(#context_t::uninhabitable(#ctx_var, #type_name))
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
                ctx_var,
                &body_decoder_var,
                v.name,
                v.tag_type,
                &v.path,
                &v.fields,
                Some(&variant_tag),
                v.field_tag_method,
            )?,
            Packing::Packed => {
                decode_packed(e, v.span, ctx_var, &body_decoder_var, &v.path, &v.fields)?
            }
            Packing::Transparent => {
                decode_transparent(e, v.span, ctx_var, &body_decoder_var, &v.path, &v.fields)?
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
                if !#variant_decoder_t::skip_variant(&mut #variant_decoder_var, #ctx_var)? {
                    return Err(#context_t::invalid_variant_tag(#ctx_var, #type_name, tag));
                }

                #variant_decoder_t::end(#variant_decoder_var, #ctx_var)?;
                Self::#ident {}
            }
        }
        None => quote_spanned! {
            en.span =>
            return Err(#context_t::invalid_variant_tag(#ctx_var, #type_name, tag));
        },
    };

    let (decode_tag, unsupported_pattern, output_enum) = handle_tag_decode(
        e,
        en.span,
        ctx_var,
        &variant_decoder_var,
        en.variant_tag_method,
        &tag_visitor_output,
        &outputs,
    )?;

    let fallback = quote_spanned! {
        en.span => #unsupported_pattern => { #fallback }
    };

    let tag_type = en
        .tag_type
        .as_ref()
        .map(|(_, ty)| quote_spanned!(ty.span() => : #ty));

    if let Some(enum_tagging) = en.enum_tagging {
        let mode_ident = e.mode_ident.as_path();
        let pair_decoder_t = &e.tokens.pair_decoder_t;
        let pairs_decoder_t = &e.tokens.pairs_decoder_t;
        let as_decoder_t = &e.tokens.as_decoder_t;

        match enum_tagging {
            EnumTagging::Internal {
                tag:
                    EnumTag {
                        value: field_tag,
                        method: field_tag_method,
                    },
            } => {
                let patterns = patterns.into_iter().map(|(span, tag, output)| {
                    quote_spanned! {
                        span =>
                        #tag => {
                            let #body_decoder_var = #as_decoder_t::as_decoder(&content, #ctx_var)?;
                            #output
                        }
                    }
                });

                let decode_outcome = decode_outcome(
                    e,
                    en.span,
                    ctx_var,
                    field_tag_method,
                    [(field_tag, syn::Ident::new("Tag", en.span))],
                );
                let output_match = decode_output_match(en.span, &variant_tag, patterns, fallback);

                return Ok(quote_spanned! {
                    en.span => {
                        #output_enum

                        enum Outcome<T> {
                            Tag,
                            Other(T),
                        }

                        let content = #decoder_t::decode_buffer::<#mode_ident, _>(#root_decoder_var, #ctx_var)?;
                        let st = #as_decoder_t::as_decoder(&content, #ctx_var)?;
                        let mut st = #decoder_t::decode_map(st, #ctx_var)?;

                        let #variant_tag #tag_type = {
                            let field = loop {
                                let mut entry = match #pairs_decoder_t::next(&mut st, #ctx_var)? {
                                    Some(entry) => entry,
                                    None => break None,
                                };

                                let decoder = #pair_decoder_t::first(&mut entry, #ctx_var)?;
                                let outcome = #decode_outcome;

                                match outcome {
                                    Outcome::Tag => {
                                        break Some(#pair_decoder_t::second(entry, #ctx_var)?);
                                    }
                                    Outcome::Other(field) => {
                                        if !#pair_decoder_t::skip_second(entry, #ctx_var)? {
                                            return Err(#context_t::invalid_field_tag(#ctx_var, #type_name, field));
                                        }
                                    }
                                }
                            };

                            let #variant_decoder_var = match field {
                                Some(decoder) => decoder,
                                None => return Err(#context_t::missing_variant_field(#ctx_var, #type_name, #field_tag)),
                            };

                            #decode_tag
                        };

                        #pairs_decoder_t::end(st, #ctx_var)?;
                        Ok(#output_match)
                    }
                });
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
                let patterns = patterns.into_iter().map(|(span, tag, output)| {
                    quote_spanned! {
                        span =>
                        #tag => {
                            #output
                        }
                    }
                });

                let decode_outcome = decode_outcome(
                    e,
                    en.span,
                    ctx_var,
                    tag_method,
                    [
                        (tag, syn::Ident::new("Tag", en.span)),
                        (content, syn::Ident::new("Content", en.span)),
                    ],
                );

                let output_match = decode_output_match(en.span, &variant_tag, patterns, fallback);

                return Ok(quote_spanned! {
                    en.span => {
                        #output_enum

                        enum Outcome<T> {
                            Tag,
                            Content,
                            Other(T),
                        }

                        let mut st = #decoder_t::decode_map(#root_decoder_var, #ctx_var)?;
                        let mut tag = None;

                        let output = loop {
                            let mut entry = match #pairs_decoder_t::next(&mut st, #ctx_var)? {
                                Some(entry) => entry,
                                None => {
                                    return Err(#context_t::invalid_field_tag(#ctx_var, #type_name, "other"));
                                },
                            };

                            let decoder = #pair_decoder_t::first(&mut entry, #ctx_var)?;
                            let outcome = #decode_outcome;

                            match outcome {
                                Outcome::Tag => {
                                    let #variant_decoder_var = #pair_decoder_t::second(entry, #ctx_var)?;
                                    tag = Some(#decode_tag);
                                }
                                Outcome::Content => {
                                    let #variant_tag #tag_type = match tag {
                                        Some(tag) => tag,
                                        None => {
                                            return Err(#context_t::invalid_field_tag(#ctx_var, #type_name, #tag));
                                        }
                                    };

                                    let #body_decoder_var = #pair_decoder_t::second(entry, #ctx_var)?;
                                    break Ok(#output_match);
                                }
                                Outcome::Other(field) => {
                                    if !#pair_decoder_t::skip_second(entry, #ctx_var)? {
                                        return Err(#context_t::invalid_field_tag(#ctx_var, #type_name, field));
                                    }
                                }
                            }
                        };

                        #pairs_decoder_t::end(st, #ctx_var)?;
                        output
                    }
                });
            }
        }
    }

    let patterns = patterns.into_iter().map(|(span, tag, output)| {
        let variant_decoder_t = &e.tokens.variant_decoder_t;

        quote_spanned! {
            span =>
            #tag => {
                let output = {
                    let #body_decoder_var = #variant_decoder_t::variant(&mut #variant_decoder_var, #ctx_var)?;
                    #output
                };

                #variant_decoder_t::end(#variant_decoder_var, #ctx_var)?;
                output
            }
        }
    });

    let output_match = decode_output_match(en.span, &variant_tag, patterns, fallback);

    let variant_decoder_t_tag = &e.tokens.variant_decoder_t_tag;

    Ok(quote_spanned! {
        en.span =>
        #output_enum

        let mut #variant_decoder_var = #decoder_t::decode_variant(#root_decoder_var, #ctx_var)?;

        let #variant_tag #tag_type = {
            let mut #variant_decoder_var = #variant_decoder_t_tag(&mut #variant_decoder_var, #ctx_var)?;
            #decode_tag
        };

        Ok(#output_match)
    })
}

fn decode_outcome<P, T>(
    e: &Build<'_>,
    span: Span,
    ctx_var: &syn::Ident,
    tag_method: Option<TagMethod>,
    patterns: P,
) -> TokenStream
where
    P: IntoIterator<Item = (T, syn::Ident)>,
    T: ToTokens,
{
    let decode_t_decode = &e.decode_t_decode;
    let decoder_t = &e.tokens.decoder_t;
    let visit_string_fn = &e.tokens.visit_string_fn;

    let patterns = patterns
        .into_iter()
        .map(|(pat, outcome)| quote_spanned!(span => #pat => Outcome::#outcome));

    match tag_method {
        Some(TagMethod::String) => quote_spanned! {
            span =>
            #decoder_t::decode_string(decoder, #ctx_var, #visit_string_fn(|#ctx_var, string| {
                Ok(match string {
                    #(#patterns,)*
                    other => Outcome::Other(string.to_owned()),
                })
            }))?
        },
        _ => {
            quote_spanned! {
                span =>
                match #decode_t_decode(#ctx_var, decoder)? {
                    #(#patterns,)*
                    other => Outcome::Other(other),
                }
            }
        }
    }
}

fn decode_output_match<P>(
    span: Span,
    tag: &syn::Ident,
    patterns: P,
    fallback: TokenStream,
) -> TokenStream
where
    P: Iterator<Item = TokenStream>,
{
    quote_spanned! {
        span =>
        match #tag {
            #(#patterns,)*
            #fallback
        }
    }
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
    type_name: &syn::LitStr,
    tag_type: Option<&(Span, syn::Type)>,
    path: &syn::Path,
    fields: &[FieldBuild],
    variant_tag: Option<&syn::Ident>,
    field_tag_method: TagMethod,
) -> Result<TokenStream> {
    let struct_decoder_var = syn::Ident::new("struct_decoder", span);

    let decoder_t = &e.tokens.decoder_t;
    let default_function = &e.tokens.default_function;
    let pairs_decoder_t = &e.tokens.pairs_decoder_t;
    let context_t = &e.tokens.context_t;

    let fields_len = fields.len();
    let mut decls = Vec::with_capacity(fields_len);
    let mut patterns = Vec::with_capacity(fields_len);
    let mut assigns = Punctuated::<_, Token![,]>::new();

    let tag_visitor_output = syn::Ident::new("TagVisitorOutput", e.input.ident.span());
    let mut outputs = Vec::with_capacity(fields_len);

    for f in fields {
        let tag = &f.tag;
        let (span, decode_path) = &f.decode_path;
        let var = syn::Ident::new(&format!("v{}", f.index), *span);

        decls.push(quote_spanned!(*span => let mut #var = None;));
        let decode =
            quote_spanned!(*span => #var = Some(#decode_path(#ctx_var, #struct_decoder_var)?));

        let (output_tag, output) =
            handle_indirect_output_tag(*span, f.index, field_tag_method, tag, &tag_visitor_output);

        outputs.extend(output);
        patterns.push((f.span, output_tag, decode));

        let fallback = if let Some(span) = f.default_attr {
            quote_spanned!(span => #default_function())
        } else {
            quote! {
                return Err(#context_t::expected_tag(#ctx_var, #type_name, #tag))
            }
        };

        assigns.push(syn::FieldValue {
            attrs: Vec::new(),
            member: f.field_access.clone(),
            colon_token: Some(<Token![:]>::default()),
            expr: syn::Expr::Verbatim(quote! {
                match #var {
                    Some(#var) => #var,
                    None => #fallback,
                }
            }),
        });
    }

    let (decode_tag, unsupported_pattern, output_enum) = handle_tag_decode(
        e,
        span,
        ctx_var,
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
                    let #struct_decoder_var = #pair_decoder_t::second(#struct_decoder_var, #ctx_var)?;
                    #decode;
                }
            }
        })
        .collect::<Vec<_>>();

    let skip_field = quote_spanned! {
        span =>
        #pair_decoder_t::skip_second(#struct_decoder_var, #ctx_var)?
    };

    let unsupported = match variant_tag {
        Some(variant_tag) => quote_spanned! {
            span =>
            #context_t::invalid_variant_field_tag(#ctx_var, #type_name, #variant_tag, tag)
        },
        None => quote_spanned! {
            span =>
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

    if let Some((_, tag_type)) = tag_type {
        tag_stmt.pat = syn::Pat::Type(syn::PatType {
            attrs: Vec::new(),
            pat: Box::new(tag_stmt.pat),
            colon_token: <Token![:]>::default(),
            ty: Box::new(tag_type.clone()),
        });
    }

    let body = if patterns.is_empty() {
        quote! {
            if !#skip_field {
                return Err(#unsupported);
            }
        }
    } else {
        quote! {
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

    tag_stmt.init = Some(syn::LocalInit {
        eq_token: <Token![=]>::default(),
        expr: Box::new(syn::Expr::Verbatim(quote! {{
            let #struct_decoder_var = #pair_decoder_t::first(&mut #struct_decoder_var, #ctx_var)?;
            #decode_tag
        }})),
        diverge: None,
    });

    Ok(quote! {
        #(#decls)*
        #output_enum
        let mut type_decoder = #decoder_t::decode_struct(#parent_decoder_var, #ctx_var, #fields_len)?;

        while let Some(mut #struct_decoder_var) = #pairs_decoder_t::next(&mut type_decoder, #ctx_var)? {
            #tag_stmt
            #body
        }

        #pairs_decoder_t::end(type_decoder, #ctx_var)?;
        #path { #assigns }
    })
}

/// Decode a transparent value.
fn decode_transparent(
    e: &Build<'_>,
    span: Span,
    ctx_var: &syn::Ident,
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
            #access: #decode_path(#ctx_var, #decoder_var)?
        }
    })
}

/// Decode something packed.
fn decode_packed(
    e: &Build<'_>,
    span: Span,
    ctx_var: &syn::Ident,
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
                let field_decoder = #pack_decoder_t::next(&mut unpack, #ctx_var)?;
                #decode_path(#ctx_var, field_decoder)?
            }
        });
    }

    if assign.is_empty() {
        Ok(quote_spanned!(span => #path {}))
    } else {
        Ok(quote_spanned! {
            span =>
            let mut unpack = #decoder_t::decode_pack(#decoder_var, #ctx_var)?;
            let output = #path { #(#assign),* };
            #pack_decoder_t::end(unpack, #ctx_var)?;
            output
        })
    }
}

/// Handle tag decoding.
fn handle_tag_decode(
    e: &Build<'_>,
    span: Span,
    ctx_var: &syn::Ident,
    thing_decoder_var: &syn::Ident,
    tag_method: TagMethod,
    output: &syn::Ident,
    outputs: &[IndirectOutput],
) -> Result<(TokenStream, TokenStream, Option<TokenStream>)> {
    match tag_method {
        TagMethod::String => {
            let (decode_tag, output_enum) =
                string_variant_tag_decode(e, span, ctx_var, thing_decoder_var, output, outputs)?;

            Ok((
                decode_tag,
                quote_spanned!(span => #output::Err(tag)),
                Some(output_enum),
            ))
        }
        TagMethod::Index => {
            let decode_t_decode = &e.decode_t_decode;

            let decode_tag = quote! {{
                let tag = #decode_t_decode(#ctx_var, #thing_decoder_var)?;
                tag
            }};

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
    pub(crate) fn as_arm(&self) -> syn::Arm {
        syn::Arm {
            attrs: Vec::new(),
            pat: syn::Pat::Verbatim(self.tag.to_token_stream()),
            guard: None,
            fat_arrow_token: <Token![=>]>::default(),
            body: Box::new(syn::Expr::Path(syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: self.path.clone(),
            })),
            comma: None,
        }
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
    ctx_var: &syn::Ident,
    var: &syn::Ident,
    output: &syn::Ident,
    outputs: &[IndirectOutput],
) -> Result<(TokenStream, TokenStream)> {
    let decoder_t = &e.tokens.decoder_t;
    let visit_string_fn = &e.tokens.visit_string_fn;
    let fmt = &e.tokens.fmt;
    let patterns = outputs.iter().map(|o| o.as_arm());

    // Declare a tag visitor, allowing string tags to be decoded by
    // decoders that owns the string.
    let decode_tag = quote_spanned! {
        span =>
        #decoder_t::decode_string(#var, #ctx_var, #visit_string_fn(|#ctx_var, string| {
            Ok::<#output, _>(match string { #(#patterns,)* _ => #output::Err(string.into())})
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
