use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

use crate::internals::attr::{self, DefaultTag, FieldAttr, Packing, TypeAttr};
use crate::internals::symbol::*;
use crate::internals::{Ctxt, Needs, NeedsKind};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum TagMethod {
    /// The default tag method.
    Default,
    /// Special method that requires generating a visitor.
    String,
}

impl Default for TagMethod {
    fn default() -> Self {
        Self::Default
    }
}

struct Tokens {
    decode_t: TokenStream,
    decoder_t: TokenStream,
    decoder_var: syn::Ident,
    default_t: TokenStream,
    encode_t: TokenStream,
    encoder_t: TokenStream,
    encoder_var: syn::Ident,
    error_t: TokenStream,
    pack_decoder_t: TokenStream,
    pack_encoder_t: TokenStream,
    pair_decoder_t: TokenStream,
    pair_encoder_t: TokenStream,
    struct_decoder_t: TokenStream,
    reference_visitor_t: TokenStream,
    phantom_data: TokenStream,
    fmt: TokenStream,
}

pub(crate) struct Expander<'a> {
    pub(crate) input: &'a syn::DeriveInput,
    cx: Ctxt,
    type_attr: TypeAttr,
    type_name: syn::LitStr,
    tokens: Tokens,
}

impl<'a> Expander<'a> {
    pub(crate) fn new(input: &'a syn::DeriveInput, prefix: &TokenStream) -> Self {
        let cx = Ctxt::new();
        let type_attr = attr::type_attrs(&cx, &input.attrs);
        let type_name = syn::LitStr::new(&input.ident.to_string(), input.ident.span());

        Self {
            input,
            cx,
            type_attr,
            type_name,
            tokens: Tokens {
                decode_t: quote!(#prefix::de::Decode),
                decoder_t: quote!(#prefix::de::Decoder),
                decoder_var: syn::Ident::new("decoder", input.ident.span()),
                default_t: quote!(::core::default::Default::default()),
                encode_t: quote!(#prefix::en::Encode),
                encoder_t: quote!(#prefix::en::Encoder),
                encoder_var: syn::Ident::new("encoder", input.ident.span()),
                error_t: quote!(#prefix::error::Error),
                pack_decoder_t: quote!(#prefix::de::PackDecoder),
                pack_encoder_t: quote!(#prefix::en::PackEncoder),
                pair_decoder_t: quote!(#prefix::de::PairDecoder),
                pair_encoder_t: quote!(#prefix::en::PairEncoder),
                struct_decoder_t: quote!(#prefix::de::PairsDecoder),
                reference_visitor_t: quote!(#prefix::de::ValueVisitor),
                phantom_data: quote!(core::marker::PhantomData),
                fmt: quote!(core::fmt),
            },
        }
    }

    /// Coerce into errors.
    pub(crate) fn into_errors(self) -> Vec<syn::Error> {
        self.cx.into_errors()
    }

    /// Expand Encode implementation.
    pub(crate) fn expand_encode(&self) -> Option<TokenStream> {
        let span = self.input.ident.span();

        let encoder_var = &self.tokens.encoder_var;

        let type_generics = &self.input.generics;
        let type_ident = &self.input.ident;

        let mut needs = Needs::default();

        let body = match &self.input.data {
            syn::Data::Struct(data) => self.encode_struct(&data.fields, &mut needs)?,
            syn::Data::Enum(data) => self.encode_enum(data, &mut needs)?,
            syn::Data::Union(..) => {
                self.cx.error_span(span, "Unions are not supported");
                return None;
            }
        };

        if self.cx.has_errors() {
            return None;
        }

        let assignment = match needs.kind {
            NeedsKind::Unused => quote!(_),
            NeedsKind::Used => quote!(#encoder_var),
        };

        let inline = if needs.inline {
            Some(quote!(#[inline]))
        } else {
            None
        };

        let encode_t = &self.tokens.encode_t;
        let encoder_t = &self.tokens.encoder_t;

        Some(quote_spanned! {
            span =>
            #[automatically_derived]
            impl #type_generics #encode_t for #type_ident #type_generics {
                #inline
                fn encode<E>(&self, #assignment: E) -> Result<E::Ok, E::Error>
                where
                    E: #encoder_t
                {
                    #body
                }
            }
        })
    }

    fn transparent_diagnostics(&self, span: Span, fields: &syn::Fields) {
        if fields.is_empty() {
            self.cx.error_span(
                span,
                format!(
                    "#[{}({})] types must have a single field",
                    ATTR, TRANSPARENT
                ),
            );
        } else {
            self.cx.error_span(
                span,
                format!(
                    "#[{}({})] can only be used on types which have a single field",
                    ATTR, TRANSPARENT
                ),
            );
        }
    }

    /// Encode a transparent element.
    fn encode_transparent(
        &self,
        span: Span,
        fields: &syn::Fields,
        needs: &mut Needs,
    ) -> Option<TokenStream> {
        let first_field = fields.iter().next();

        let (span, accessor, field_attr) = match first_field {
            Some(field) if fields.len() == 1 => {
                let field_attr = attr::field_attrs(&self.cx, &field.attrs);

                let accessor = match &field.ident {
                    Some(ident) => quote_spanned!(field.span() => &self.#ident),
                    None => quote_spanned!(field.span() => &self.0),
                };

                (field.span(), accessor, field_attr)
            }
            _ => {
                self.transparent_diagnostics(span, fields);
                return None;
            }
        };

        let encode_t = &self.tokens.encode_t;
        let encoder_var = &self.tokens.encoder_var;

        needs.mark_used();

        let (span, encode_path) = field_attr.encode_path(encode_t, span);

        Some(quote_spanned! {
            span => #encode_path(#accessor, #encoder_var)
        })
    }

    /// Encode a transparent element.
    fn transparent_variant(
        &self,
        span: Span,
        fields: &syn::Fields,
        needs: &mut Needs,
    ) -> Option<(TokenStream, Vec<TokenStream>)> {
        let ident = fields.iter().next();

        let (span, pattern, field_attr) = match ident {
            Some(field) if fields.len() == 1 => {
                let field_attr = attr::field_attrs(&self.cx, &field.attrs);

                let accessor = match &field.ident {
                    Some(ident) => quote_spanned!(field.span() => #ident: this),
                    None => quote_spanned!(field.span() => 0: this),
                };

                (field.span(), accessor, field_attr)
            }
            _ => {
                self.transparent_diagnostics(span, fields);
                return None;
            }
        };

        let encode_t = &self.tokens.encode_t;
        let encoder_var = &self.tokens.encoder_var;

        needs.mark_used();

        let (span, encode_path) = field_attr.encode_path(encode_t, span);

        let encode = quote_spanned! {
            span => #encode_path(this, #encoder_var)
        };

        Some((encode, vec![pattern]))
    }

    /// Decode a transparent value.
    fn decode_transparent(
        &self,
        span: Span,
        path: syn::Path,
        fields: &syn::Fields,
        needs: &mut Needs,
    ) -> Option<TokenStream> {
        let ident = fields.iter().next();

        let (accessor, field_attr) = match ident {
            Some(field) if fields.len() == 1 => {
                let field_attr = attr::field_attrs(&self.cx, &field.attrs);

                let accessor = match &field.ident {
                    Some(ident) => quote!(#ident),
                    None => quote!(0),
                };

                (accessor, field_attr)
            }
            _ => {
                self.transparent_diagnostics(span, fields);
                return None;
            }
        };

        let decode_t = &self.tokens.decode_t;
        let decoder_var = &self.tokens.decoder_var;

        needs.mark_used();
        needs.mark_inline();

        let (span, decode_path) = field_attr.decode_path(decode_t, span);

        Some(quote_spanned! {
            span =>
            #path {
                #accessor: #decode_path(#decoder_var)?
            }
        })
    }

    /// Encode a struct.
    fn encode_struct(&self, fields: &syn::Fields, needs: &mut Needs) -> Option<TokenStream> {
        let mut decls = Vec::with_capacity(fields.len());
        let mut encoders = Vec::with_capacity(fields.len());
        let mut field_tests = Vec::with_capacity(fields.len());
        let encoder_var = &self.tokens.encoder_var;
        let packing = self.type_attr.packing();

        for (index, field) in fields.iter().enumerate() {
            needs.mark_used();

            let field_attr = attr::field_attrs(&self.cx, &field.attrs);

            let access = match &field.ident {
                Some(ident) => quote!(&self.#ident),
                None => {
                    let n = field_int(index, field.span());
                    quote!(&self.#n)
                }
            };

            let (encoder, skip) = self.encode_field(
                index,
                field,
                &field_attr,
                &access,
                packing,
                self.type_attr.default_field_tag,
            )?;

            encoders.push(encoder);

            if let Some((decl, test)) = skip {
                decls.push(decl);
                field_tests.push(test);
            }
        }

        let encode = match packing {
            Packing::Transparent => {
                self.encode_transparent(self.input.ident.span(), fields, needs)?
            }
            Packing::Tagged => {
                needs.mark_used();
                let encode = quote! { #(#encoders)* };
                self.encode_field_tag(fields, &decls, encode, &field_tests)
            }
            Packing::Packed => {
                needs.mark_used();

                let encoder_t = &self.tokens.encoder_t;
                let pack_encoder_t = &self.tokens.pack_encoder_t;

                quote! {
                    #(#decls)*
                    let mut pack = #encoder_t::encode_pack(#encoder_var)?;
                    #(#encoders)*
                    #pack_encoder_t::end(pack)
                }
            }
        };

        Some(encode)
    }

    fn encode_enum(&self, data: &syn::DataEnum, needs: &mut Needs) -> Option<TokenStream> {
        if let Some((span, Packing::Transparent)) = self.type_attr.packing {
            self.cx.error_span(
                span,
                format!("#[{}({})] cannot be used on enums", ATTR, TRANSPARENT),
            );
            return None;
        }

        let mut variants = Vec::with_capacity(data.variants.len());

        for (variant_index, variant) in data.variants.iter().enumerate() {
            if let Some(variant) = self.encode_variant(variant_index, variant, needs) {
                variants.push(variant);
            }
        }

        // Special case: uninhabitable types.
        Some(if variants.is_empty() {
            quote! {
                match *self {}
            }
        } else {
            quote! {
                match self {
                    #(#variants)*
                }
            }
        })
    }

    /// Expand Decode implementation.
    pub(crate) fn expand_decode(&self) -> Option<TokenStream> {
        let span = self.input.ident.span();

        let decoder_var = &self.tokens.decoder_var;
        let type_generics = &self.input.generics;
        let type_ident = &self.input.ident;

        let no_lifetimes = type_generics.lifetimes().count() == 0;

        let lts = if no_lifetimes {
            quote!(<'de>)
        } else {
            let lts = type_generics.lifetimes();
            quote!(<#(#lts),*>)
        };

        let impl_clause = if type_generics.params.is_empty() {
            quote!(<'de>)
        } else if no_lifetimes {
            quote!(#type_generics)
        } else {
            let params = type_generics.params.iter();
            quote!(<#(#params),*, 'de>)
        };

        let decoder_lt = if let Some(lt) = type_generics.lifetimes().next() {
            quote!(#lt)
        } else {
            quote!('de)
        };

        let mut needs = Needs::default();

        let body = match &self.input.data {
            syn::Data::Struct(data) => self.decode_struct(span, data, &mut needs)?,
            syn::Data::Enum(data) => self.decode_enum(data, &mut needs)?,
            syn::Data::Union(..) => {
                self.cx.error_span(span, "Unions are not supported");
                return None;
            }
        };

        if self.cx.has_errors() {
            return None;
        }

        let assignment = match needs.kind {
            NeedsKind::Unused => quote!(_),
            NeedsKind::Used => quote!(#decoder_var),
        };

        let inline = if needs.inline {
            Some(quote!(#[inline]))
        } else {
            None
        };

        let decode_t = &self.tokens.decode_t;
        let decoder_t = &self.tokens.decoder_t;

        Some(quote_spanned! {
            span =>
            #[automatically_derived]
            impl #impl_clause #decode_t #lts for #type_ident #type_generics {
                #inline
                fn decode<D>(#assignment: D) -> Result<Self, D::Error>
                where
                    D: #decoder_t<#decoder_lt>
                {
                    #body
                }
            }
        })
    }

    fn decode_struct(
        &self,
        span: Span,
        data: &syn::DataStruct,
        needs: &mut Needs,
    ) -> Option<TokenStream> {
        let path = syn::Path::from(syn::Ident::new("Self", self.input.ident.span()));
        let tag_type = self.type_attr.tag_type.as_ref();

        let body = match self.type_attr.packing() {
            Packing::Tagged => {
                needs.mark_used();
                self.decode_tagged(
                    &self.type_name,
                    tag_type,
                    path,
                    &data.fields,
                    None,
                    self.type_attr.default_field_tag,
                )?
            }
            Packing::Packed => self.decode_untagged(path, &data.fields, needs)?,
            Packing::Transparent => self.decode_transparent(span, path, &data.fields, needs)?,
        };

        Some(quote! {
            Ok({
                #body
            })
        })
    }

    fn decode_enum(&self, data: &syn::DataEnum, needs: &mut Needs) -> Option<TokenStream> {
        let decoder_t = &self.tokens.decoder_t;
        let decoder_var = &self.tokens.decoder_var;
        let error_t = &self.tokens.error_t;
        let type_ident = &self.input.ident;
        let type_name = &self.type_name;
        let pair_decoder_t = &self.tokens.pair_decoder_t;
        let decode_t = &self.tokens.decode_t;
        let variant_tag = syn::Ident::new("variant_tag", self.input.ident.span());

        if let Some((span, Packing::Packed)) = self.type_attr.packing {
            self.cx.error_span(
                span,
                format!(
                    "`Decode` cannot be implemented on enums which are #[{}({})]",
                    ATTR, PACKED
                ),
            );
            return None;
        }

        if data.variants.is_empty() {
            // Special case: Uninhabitable type. Since this cannot be reached, generate the never type.
            return Some(quote! {
                Err(<D::Error as #error_t>::uninhabitable(#type_name))
            });
        }

        needs.mark_used();
        let mut variants = Vec::with_capacity(data.variants.len());

        let type_packing = self.type_attr.packing();

        let mut fallback = None;
        // Keep track of variant index manually since fallback variants do not
        // count.
        let mut variant_index = 0;

        for variant in data.variants.iter() {
            let span = variant.span();

            let variant_attr = attr::variant_attrs(&self.cx, &variant.attrs);

            if variant_attr.default.is_some() {
                if !variant.fields.is_empty() {
                    self.cx.error_span(
                        variant.fields.span(),
                        format!("#[{}({})] variant must be empty", ATTR, DEFAULT),
                    );
                    continue;
                }

                if fallback.is_some() {
                    self.cx.error_span(
                        variant.ident.span(),
                        format!(
                            "#[{}({})] only one fallback variant is supported",
                            ATTR, DEFAULT
                        ),
                    );
                    continue;
                }

                fallback = Some(&variant.ident);
                continue;
            }

            let variant_name = syn::LitStr::new(&variant.ident.to_string(), variant.ident.span());

            let mut path = syn::Path::from(syn::Ident::new("Self", type_ident.span()));
            path.segments.push(variant.ident.clone().into());

            let tag_type = variant_attr.tag_type.as_ref();

            let default_field_tag = variant_attr
                .default_field_tag
                .unwrap_or(self.type_attr.default_field_tag);

            let output = match variant_attr.packing().unwrap_or(type_packing) {
                Packing::Tagged => self.decode_tagged(
                    &variant_name,
                    tag_type,
                    path,
                    &variant.fields,
                    Some(&variant_tag),
                    default_field_tag,
                )?,
                Packing::Packed => self.decode_untagged(path, &variant.fields, needs)?,
                Packing::Transparent => {
                    self.decode_transparent(span, path, &variant.fields, needs)?
                }
            };

            let tag = self.expand_variant_tag(
                variant_attr.rename.as_ref(),
                self.type_attr.default_variant_tag,
                variant_index,
                &variant.ident,
            );

            variants.push(quote! {
                #tag => {
                    let #decoder_var = #pair_decoder_t::second(variant)?;
                    #output
                }
            });

            variant_index += 1;
        }

        let tag_type = self
            .type_attr
            .tag_type
            .as_ref()
            .map(|(_, ty)| quote!(: #ty));

        let fallback = match fallback {
            Some(ident) => {
                quote! {
                    if !#pair_decoder_t::skip_second(variant)? {
                        return Err(<D::Error as #error_t>::invalid_variant_tag(#type_name, tag));
                    }

                    Self::#ident {}
                }
            }
            None => quote! {
                return Err(<D::Error as #error_t>::invalid_variant_tag(#type_name, tag));
            },
        };

        Some(quote! {{
            let mut variant = #decoder_t::decode_variant(#decoder_var)?;

            let #variant_tag #tag_type = {
                let tag_decoder = #pair_decoder_t::first(&mut variant)?;
                #decode_t::decode(tag_decoder)?
            };

            Ok(match variant_tag {
                #(#variants,)*
                tag => {
                    #fallback
                }
            })
        }})
    }

    fn encode_field_tag(
        &self,
        fields: &syn::Fields,
        decls: &[TokenStream],
        encode: TokenStream,
        tests: &[syn::Ident],
    ) -> TokenStream {
        let encoder_var = &self.tokens.encoder_var;
        let encoder_t = &self.tokens.encoder_t;
        let pair_encoder_t = &self.tokens.pair_encoder_t;

        let setup = match fields {
            syn::Fields::Named(..) => {
                let fields = calculate_tests(fields.len(), tests);
                quote!(#encoder_t::encode_struct(#encoder_var, #fields)?)
            }
            syn::Fields::Unnamed(..) => {
                let fields = calculate_tests(fields.len(), tests);
                quote!(#encoder_t::encode_tuple(#encoder_var, #fields)?)
            }
            syn::Fields::Unit => {
                return quote!(#encoder_t::encode_unit_struct(#encoder_var));
            }
        };

        quote! {
            #(#decls)*
            let mut #encoder_var = #setup;
            #encode
            #pair_encoder_t::end(#encoder_var)
        }
    }

    /// Setup encoding for a single variant.
    fn encode_variant(
        &self,
        variant_index: usize,
        variant: &syn::Variant,
        needs: &mut Needs,
    ) -> Option<TokenStream> {
        let span = variant.span();

        let encoder_var = &self.tokens.encoder_var;
        let variant_attr = attr::variant_attrs(&self.cx, &variant.attrs);
        let packing = variant_attr
            .packing()
            .unwrap_or_else(|| self.type_attr.packing());

        let default_field_tag = variant_attr
            .default_field_tag
            .unwrap_or(self.type_attr.default_field_tag);

        let (mut encode, patterns) = match packing {
            Packing::Tagged => {
                let (encode, patterns, tests) =
                    self.encode_variant_fields(&variant.fields, needs, packing, default_field_tag)?;

                // Special stuff needed to encode the field if its tagged.
                let encode = self.encode_field_tag(&variant.fields, &[], encode, &tests);
                (encode, patterns)
            }
            Packing::Packed => {
                let (encode, patterns, _) =
                    self.encode_variant_fields(&variant.fields, needs, packing, default_field_tag)?;
                (encode, patterns)
            }
            Packing::Transparent => self.transparent_variant(span, &variant.fields, needs)?,
        };

        if let Packing::Tagged = self.type_attr.packing() {
            needs.mark_used();

            let Tokens {
                encode_t,
                encoder_t,
                pair_encoder_t,
                ..
            } = &self.tokens;

            let tag = self.expand_variant_tag(
                variant_attr.rename.as_ref(),
                self.type_attr.default_variant_tag,
                variant_index,
                &variant.ident,
            );

            let (encode_function, len) = match &variant.fields {
                syn::Fields::Named(_) => (
                    syn::Ident::new("encode_struct_variant", variant.span()),
                    Some(variant.fields.len()),
                ),
                syn::Fields::Unnamed(_) => (
                    syn::Ident::new("encode_tuple_variant", variant.span()),
                    Some(variant.fields.len()),
                ),
                syn::Fields::Unit => (syn::Ident::new("encode_unit_variant", variant.span()), None),
            };

            encode = quote! {{
                let mut variant_encoder = #encoder_t::#encode_function(#encoder_var, #len)?;
                let tag_encoder = #pair_encoder_t::first(&mut variant_encoder)?;
                #encode_t::encode(&#tag, tag_encoder)?;
                let _ = {
                    let #encoder_var = #pair_encoder_t::second(&mut variant_encoder)?;
                    #encode
                }?;
                #pair_encoder_t::end(variant_encoder)
            }};
        }

        let mut path = syn::Path::from(syn::Ident::new("Self", span));
        path.segments.push(variant.ident.clone().into());

        Some(quote! {
            #path { #(#patterns),* } => { #encode }
        })
    }

    fn encode_variant_fields(
        &self,
        fields: &syn::Fields,
        needs: &mut Needs,
        tagged: Packing,
        default_field_tag: DefaultTag,
    ) -> Option<(TokenStream, Vec<TokenStream>, Vec<syn::Ident>)> {
        let mut decls = Vec::with_capacity(fields.len());
        let mut encoders = Vec::with_capacity(fields.len());
        let mut patterns = Vec::with_capacity(fields.len());
        let mut field_tests = Vec::with_capacity(fields.len());

        for (index, field) in fields.iter().enumerate() {
            needs.mark_used();

            let field_attr = attr::field_attrs(&self.cx, &field.attrs);

            let access = match &field.ident {
                Some(ident) => {
                    patterns.push(quote!(#ident));
                    quote!(#ident)
                }
                None => {
                    let index = field_int(index, field.span());
                    let var = syn::Ident::new(&format!("v{}", index), field.span());
                    patterns.push(quote!(#index: #var));
                    quote!(#var)
                }
            };

            let (encoder, skip) = self.encode_field(
                index,
                field,
                &field_attr,
                &access,
                tagged,
                default_field_tag,
            )?;
            encoders.push(encoder);

            if let Some((decl, test)) = skip {
                decls.push(decl);
                field_tests.push(test);
            }
        }

        let encode = match tagged {
            Packing::Tagged => {
                quote! {
                    #(#decls)*
                    #(#encoders)*
                }
            }
            Packing::Packed => {
                needs.mark_used();

                let encoder_t = &self.tokens.encoder_t;
                let encoder_var = &self.tokens.encoder_var;
                let pack_encoder_t = &self.tokens.pack_encoder_t;

                quote! {
                    #(#decls)*
                    let mut pack = #encoder_t::encode_pack(#encoder_var)?;
                    #(#encoders)*
                    #pack_encoder_t::end(pack)
                }
            }
            _ => quote!(),
        };

        Some((encode, patterns, field_tests))
    }

    /// Encode a field.
    fn encode_field(
        &self,
        index: usize,
        field: &syn::Field,
        field_attr: &FieldAttr,
        access: &TokenStream,
        tagged: Packing,
        default_field_tag: DefaultTag,
    ) -> Option<(TokenStream, Option<(TokenStream, syn::Ident)>)> {
        let encoder_var = &self.tokens.encoder_var;
        let encode_t = &self.tokens.encode_t;

        let (span, encode_path) = field_attr.encode_path(encode_t, field.span());

        let body = match tagged {
            Packing::Tagged | Packing::Transparent => {
                let (tag, _) = self.expand_field_tag(
                    field_attr.rename.as_ref(),
                    default_field_tag,
                    index,
                    field,
                )?;

                let pair_encoder_t = &self.tokens.pair_encoder_t;

                quote_spanned! {
                    span => {
                        let field_encoder = #pair_encoder_t::first(&mut #encoder_var)?;
                        #encode_t::encode(&#tag, field_encoder)?;
                        let value_encoder = #pair_encoder_t::second(&mut #encoder_var)?;
                        #encode_path(#access, value_encoder)?;
                    }
                }
            }
            Packing::Packed => {
                let pack_encoder_t = &self.tokens.pack_encoder_t;
                quote_spanned!(span => #encode_path(#access, #pack_encoder_t::next(&mut pack)?)?;)
            }
        };

        // Add condition to encode a field if configured.
        if let Some((skip_span, skip_encoding_if_path)) = field_attr.skip_encoding_if() {
            let test = syn::Ident::new(&format!("t{}", index), field.span());

            let body = quote_spanned! {
                skip_span =>
                if #test {
                    #body
                }
            };

            let decl = quote_spanned! {
                skip_span =>
                let #test = !#skip_encoding_if_path(#access);
            };

            Some((body, Some((decl, test))))
        } else {
            Some((body, None))
        }
    }

    /// Decode something tagged.
    ///
    /// If `variant_name` is specified it implies that a tagged enum is being
    /// decoded.
    fn decode_tagged(
        &self,
        type_name: &syn::LitStr,
        tag_type: Option<&(Span, syn::Type)>,
        path: syn::Path,
        fields: &syn::Fields,
        variant_tag: Option<&syn::Ident>,
        default_field_tag: DefaultTag,
    ) -> Option<TokenStream> {
        let decode_t = &self.tokens.decode_t;
        let decoder_t = &self.tokens.decoder_t;
        let decoder_var = &self.tokens.decoder_var;
        let error_t = &self.tokens.error_t;
        let default_t = &self.tokens.default_t;

        let (field_kind, fields) = match FieldKind::new(fields) {
            Some((field_kind, fields)) => (field_kind, fields),
            None => {
                return Some(quote! {
                    #decoder_t::decode_unit_struct(#decoder_var)?;
                    #path {}
                });
            }
        };

        let fields_len = fields.len();
        let mut decls = Vec::with_capacity(fields_len);
        let mut patterns = Vec::with_capacity(fields_len);
        let mut assigns = Vec::with_capacity(fields_len);
        let mut string_patterns = Vec::with_capacity(fields_len);
        let mut output_variants = Vec::with_capacity(fields_len);

        let tag_visitor_output = syn::Ident::new("TagVisitorOutput", self.input.ident.span());

        let mut tag_methods = HashSet::new();

        for (index, field) in fields.enumerate() {
            let field_attr = attr::field_attrs(&self.cx, &field.attrs);

            let (tag, tag_method) =
                self.expand_field_tag(field_attr.rename.as_ref(), default_field_tag, index, field)?;

            let before = tag_methods.len();
            tag_methods.insert(tag_method);

            if before == 1 && tag_methods.len() > 1 {
                self.cx
                    .error_spanned_by(field, format!("#[{}({})] conflicting tag kind", ATTR, TAG));
            }

            let (span, decode_path) = field_attr.decode_path(decode_t, field.span());
            let var = syn::Ident::new(&format!("v{}", index), span);
            decls.push(quote_spanned!(span => let mut #var = None;));
            let decode = quote_spanned!(span => #var = Some(#decode_path(#decoder_var)?));

            let output_tag = match tag_method {
                TagMethod::String => {
                    let variant = syn::Ident::new(&format!("Variant{}", index), field.span());
                    let mut path = syn::Path::from(tag_visitor_output.clone());
                    path.segments.push(syn::PathSegment::from(variant.clone()));

                    string_patterns.push(quote!(#tag => Ok(#path)));
                    output_variants.push(variant.clone());

                    syn::Expr::Path(syn::ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path,
                    })
                }
                TagMethod::Default => tag.clone(),
            };

            patterns.push((output_tag, decode));

            let field = match &field.ident {
                Some(ident) => quote!(#ident),
                None => {
                    let field = field_int(index, field.span());
                    quote!(#field)
                }
            };

            let fallback = if let Some(span) = field_attr.default {
                quote_spanned!(span => #default_t)
            } else {
                quote!(return Err(<D::Error as #error_t>::expected_tag(#type_name, #tag)))
            };

            assigns.push(quote!(#field: match #var {
                Some(#var) => #var,
                None => #fallback,
            }));
        }

        let declare = match field_kind {
            FieldKind::Struct => quote! {
                #decoder_t::decode_struct(#decoder_var, #fields_len)?
            },
            FieldKind::Tuple => quote! {
                #decoder_t::decode_tuple(#decoder_var, #fields_len)?
            },
        };

        let next = match field_kind {
            FieldKind::Struct => {
                let struct_decoder_t = &self.tokens.struct_decoder_t;
                quote!(#struct_decoder_t::next)
            }
            FieldKind::Tuple => {
                let struct_decoder_t = &self.tokens.struct_decoder_t;
                quote!(#struct_decoder_t::next)
            }
        };

        let tag_method = tag_methods.into_iter().next().unwrap_or_default();

        let pair_decoder_t = &self.tokens.pair_decoder_t;
        let decode_t = &self.tokens.decode_t;

        let (decode_tag, unsupported_pattern, patterns, output_enum) = match tag_method {
            TagMethod::String => {
                let reference_visitor_t = &self.tokens.reference_visitor_t;
                let phantom_data = &self.tokens.phantom_data;
                let fmt = &self.tokens.fmt;
                let error_t = &self.tokens.error_t;

                // Declare a tag visitor, allowing string tags to be decoded by
                // decoders that owns the string.
                let decode_tag = quote! {{
                    struct TagVisitor<E>(#phantom_data<E>);

                    impl<'de, E> #reference_visitor_t<'de> for TagVisitor<E> where E: #error_t {
                        type Target = str;
                        type Ok = Result<#tag_visitor_output, Box<str>>;
                        type Error = E;

                        #[inline]
                        fn expecting(&self, f: &mut #fmt::Formatter<'_>) -> #fmt::Result {
                            write!(f, "string tag")
                        }

                        #[inline]
                        fn visit_borrowed(self, string: &'de Self::Target) -> Result<Self::Ok, Self::Error> {
                            self.visit_any(string)
                        }

                        #[inline]
                        fn visit_any(self, string: &Self::Target) -> Result<Self::Ok, Self::Error> {
                            Ok(match string {
                                #(#string_patterns,)*
                                _ => Err(string.into()),
                            })
                        }
                    }

                    let index_decoder = #pair_decoder_t::first(&mut #decoder_var)?;
                    #decoder_t::decode_string(index_decoder, TagVisitor::<D::Error>(#phantom_data))?
                }};

                let patterns = patterns
                    .into_iter()
                    .map(|(tag, decode)| (quote!(Ok(#tag)), decode))
                    .collect::<Vec<_>>();

                let output_enum = quote! {
                    enum #tag_visitor_output {
                        #(#output_variants,)*
                    }
                };

                (decode_tag, quote!(Err(tag)), patterns, Some(output_enum))
            }
            TagMethod::Default => {
                let decode_tag = quote! {{
                    let index_decoder = #pair_decoder_t::first(&mut #decoder_var)?;
                    #decode_t::decode(index_decoder)?
                }};

                let patterns = patterns
                    .into_iter()
                    .map(|(tag, decode)| (quote!(#tag), decode))
                    .collect::<Vec<_>>();

                (decode_tag, quote!(tag), patterns, None)
            }
        };

        let patterns = patterns
            .into_iter()
            .map(|(tag, decode)| {
                quote! {
                    #tag => {
                        let #decoder_var = #pair_decoder_t::second(#decoder_var)?;
                        #decode;
                    }
                }
            })
            .collect::<Vec<_>>();

        let skip_field = quote! {
            #pair_decoder_t::skip_second(#decoder_var)?
        };

        let unsupported = match variant_tag {
            Some(variant_tag) => quote! {
                <D::Error as #error_t>::invalid_variant_field_tag(#type_name, #variant_tag, tag)
            },
            None => quote! {
                <D::Error as #error_t>::invalid_field_tag(#type_name, tag)
            },
        };

        let tag_type = tag_type.as_ref().map(|(_, ty)| quote!(: #ty));

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

        Some(quote! {
            #(#decls;)*
            #output_enum

            let mut #decoder_var = #declare;

            while let Some(mut #decoder_var) = #next(&mut #decoder_var)? {
                let tag #tag_type = #decode_tag;
                #body
            }

            #path { #(#assigns),* }
        })
    }

    /// Decode something packed.
    fn decode_untagged(
        &self,
        path: syn::Path,
        fields: &syn::Fields,
        needs: &mut Needs,
    ) -> Option<TokenStream> {
        let decode_t = &self.tokens.decode_t;
        let pack_decoder_t = &self.tokens.pack_decoder_t;
        let decoder_var = &self.tokens.decoder_var;
        let decoder_t = &self.tokens.decoder_t;

        let mut assign = Vec::new();

        for (index, field) in fields.iter().enumerate() {
            needs.mark_used();

            let field_attr = attr::field_attrs(&self.cx, &field.attrs);

            if let Some(span) = field_attr.default {
                self.cx.error_span(
                    span,
                    format!(
                        "#[{}({})] fields cannot be used in an packed container",
                        ATTR, DEFAULT
                    ),
                );
            }

            let (span, decode_path) = field_attr.decode_path(decode_t, field.span());

            let decode = quote! {{
                let field_decoder = #pack_decoder_t::next(&mut unpack)?;
                #decode_path(field_decoder)?
            }};

            match &field.ident {
                Some(ident) => {
                    let mut ident = ident.clone();
                    ident.set_span(span);
                    assign.push(quote_spanned!(span => #ident: #decode));
                }
                None => {
                    let field = field_int(index, span);
                    assign.push(quote_spanned!(span => #field: #decode));
                }
            }
        }

        if assign.is_empty() {
            Some(quote!(#path {}))
        } else {
            Some(quote! {{
                let mut unpack = #decoder_t::decode_pack(#decoder_var)?;
                let this = #path { #(#assign),* };
                #pack_decoder_t::end(unpack)?;
                this
            }})
        }
    }

    /// Expand the variant tag depending on the given tag configuration.
    fn expand_variant_tag(
        &self,
        rename: Option<&(Span, syn::Expr)>,
        tag: DefaultTag,
        index: usize,
        ident: &syn::Ident,
    ) -> Option<syn::Expr> {
        let lit = match (rename, tag) {
            (Some((_, value)), _) => return Some(self.rename_lit(value)),
            (None, DefaultTag::Index) => usize_int(index, ident.span()).into(),
            (None, DefaultTag::Name) => syn::LitStr::new(&ident.to_string(), ident.span()).into(),
        };

        Some(syn::Expr::Lit(syn::ExprLit {
            attrs: Vec::new(),
            lit,
        }))
    }

    /// Match out the field tag depending on the given tag configuration.
    fn expand_field_tag(
        &self,
        rename: Option<&(Span, syn::Expr)>,
        default_field_tag: DefaultTag,
        index: usize,
        field: &syn::Field,
    ) -> Option<(syn::Expr, TagMethod)> {
        let (lit, tag_method) = match (rename, default_field_tag, &field.ident) {
            (Some((_, rename)), _, _) => {
                return Some((self.rename_lit(rename), self.determine_tag_method(rename)))
            }
            (None, DefaultTag::Index, _) => {
                (usize_int(index, field.span()).into(), TagMethod::Default)
            }
            (None, DefaultTag::Name, None) => {
                self.cx.error_spanned_by(
                    field,
                    format!(
                        "#[{}({} = \"name\")] is not supported with unnamed fields",
                        ATTR, TAG
                    ),
                );
                return None;
            }
            (None, DefaultTag::Name, Some(ident)) => (
                syn::LitStr::new(&ident.to_string(), ident.span()).into(),
                TagMethod::String,
            ),
        };

        let tag = syn::Expr::Lit(syn::ExprLit {
            attrs: Vec::new(),
            lit,
        });

        Some((tag, tag_method))
    }

    /// Process rename literal to ensure it's always typed.
    fn rename_lit(&self, expr: &syn::Expr) -> syn::Expr {
        match expr {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(int),
                ..
            }) if int.suffix().is_empty() => syn::Expr::Lit(syn::ExprLit {
                attrs: Vec::new(),
                lit: syn::LitInt::new(&format!("{}usize", int), int.span()).into(),
            }),
            expr => expr.clone(),
        }
    }

    /// Try and determine tag method from the given expression.
    fn determine_tag_method(&self, expr: &syn::Expr) -> TagMethod {
        let lit = match expr {
            syn::Expr::Lit(lit) => lit,
            _ => return TagMethod::Default,
        };

        match lit {
            syn::ExprLit {
                lit: syn::Lit::Str(..),
                ..
            } => TagMethod::String,
            _ => TagMethod::Default,
        }
    }
}

fn calculate_tests(count: usize, tests: &[syn::Ident]) -> TokenStream {
    if tests.is_empty() {
        quote!(#count)
    } else {
        let count = count.saturating_sub(tests.len());
        let tests = tests.iter().map(|v| quote!(if #v { 1 } else { 0 }));
        quote!(#count + #(#tests)+*)
    }
}

#[derive(Debug, Clone, Copy)]
enum FieldKind {
    Struct,
    Tuple,
}

impl FieldKind {
    fn new(
        fields: &syn::Fields,
    ) -> Option<(
        Self,
        impl Iterator<Item = &'_ syn::Field> + ExactSizeIterator,
    )> {
        match fields {
            syn::Fields::Named(named) => Some((Self::Struct, named.named.iter())),
            syn::Fields::Unnamed(unnamed) => Some((Self::Tuple, unnamed.unnamed.iter())),
            syn::Fields::Unit => None,
        }
    }
}

/// Usize-suffixed integer.
fn usize_int(index: usize, span: Span) -> syn::LitInt {
    syn::LitInt::new(&format!("{}usize", index), span)
}

/// Integer used for tuple initialization.
fn field_int(index: usize, span: Span) -> syn::LitInt {
    syn::LitInt::new(&index.to_string(), span)
}
