use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

use crate::internals::attr::{self, FieldAttr, Packing, Tag, TypeAttr};
use crate::internals::symbol::*;
use crate::internals::{Ctxt, Needs, NeedsKind};

struct Tokens {
    decode_t: TokenStream,
    decoder_t: TokenStream,
    default_t: TokenStream,
    encode_t: TokenStream,
    encoder_t: TokenStream,
    error_t: TokenStream,
    pack_encoder_t: TokenStream,
    struct_decoder_t: TokenStream,
    struct_encoder_t: TokenStream,
    struct_field_decoder_t: TokenStream,
    tuple_decoder_t: TokenStream,
    tuple_encoder_t: TokenStream,
    tuple_field_decoder_t: TokenStream,
    pack_decoder_t: TokenStream,
    variant_decoder_t: TokenStream,
    variant_encoder_t: TokenStream,
    decoder_var: syn::Ident,
    encoder_var: syn::Ident,
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
                default_t: quote!(::core::default::Default::default()),
                encode_t: quote!(#prefix::en::Encode),
                encoder_t: quote!(#prefix::en::Encoder),
                error_t: quote!(#prefix::error::Error),
                pack_encoder_t: quote!(#prefix::en::PackEncoder),
                struct_decoder_t: quote!(#prefix::de::StructDecoder),
                struct_encoder_t: quote!(#prefix::en::StructEncoder),
                struct_field_decoder_t: quote!(#prefix::de::StructFieldDecoder),
                tuple_decoder_t: quote!(#prefix::de::TupleDecoder),
                tuple_encoder_t: quote!(#prefix::en::TupleEncoder),
                tuple_field_decoder_t: quote!(#prefix::de::TupleFieldDecoder),
                pack_decoder_t: quote!(#prefix::de::PackDecoder),
                variant_decoder_t: quote!(#prefix::de::VariantDecoder),
                variant_encoder_t: quote!(#prefix::en::VariantEncoder),
                decoder_var: syn::Ident::new("decoder", input.ident.span()),
                encoder_var: syn::Ident::new("encoder", input.ident.span()),
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
                fn encode<E>(&self, #assignment: E) -> Result<(), E::Error>
                where
                    E: #encoder_t
                {
                    #body
                }
            }
        })
    }

    fn transparent_diagnostics(&self, span: Span, fields: &syn::Fields) {
        if fields.len() == 0 {
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
            span =>
            #encode_path(#accessor, #encoder_var)?;
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
            span =>
            #encode_path(this, #encoder_var)?;
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

        if let Some((field_kind, fields)) = FieldKind::new(fields) {
            for (index, field) in fields.enumerate() {
                needs.mark_used();

                let field_attr = attr::field_attrs(&self.cx, &field.attrs);

                let access = match &field.ident {
                    Some(ident) => quote!(&self.#ident),
                    None => {
                        let n = field_int(index, field.span());
                        quote!(&self.#n)
                    }
                };

                let (encoder, skip) =
                    self.encode_field(index, field, &field_attr, &access, field_kind, packing)?;
                encoders.push(encoder);

                if let Some((decl, test)) = skip {
                    decls.push(decl);
                    field_tests.push(test);
                }
            }
        }

        let encode = match packing {
            Packing::Transparent => {
                self.encode_transparent(self.input.ident.span(), &fields, needs)?
            }
            Packing::Tagged => {
                needs.mark_used();

                let (setup, finish) = self.encode_field_tag(fields, &field_tests)?;

                quote! {
                    #(#decls)*
                    let mut #encoder_var = #setup;
                    #(#encoders)*
                    #finish
                }
            }
            Packing::Packed if !encoders.is_empty() => {
                let encoder_t = &self.tokens.encoder_t;
                let pack_encoder_t = &self.tokens.pack_encoder_t;

                quote! {
                    #(#decls)*
                    let mut pack = #encoder_t::encode_pack(#encoder_var)?;
                    #(#encoders)*
                    #pack_encoder_t::finish(pack)?;
                }
            }
            _ => quote!(),
        };

        Some(quote! {
            #encode
            Ok(())
        })
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

                Ok(())
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

        let body = match self.type_attr.packing() {
            Packing::Tagged => {
                needs.mark_used();
                self.decode_tagged(&self.type_name, path, &data.fields)?
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
        let variant_decoder_t = &self.tokens.variant_decoder_t;
        let decode_t = &self.tokens.decode_t;

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

        for (variant_index, variant) in data.variants.iter().enumerate() {
            let span = variant.span();

            let variant_attr = attr::variant_attrs(&self.cx, &variant.attrs);
            let variant_name = syn::LitStr::new(&variant.ident.to_string(), variant.ident.span());

            let mut path = syn::Path::from(syn::Ident::new("Self", type_ident.span()));
            path.segments.push(variant.ident.clone().into());

            let output = match variant_attr.packing().unwrap_or(type_packing) {
                Packing::Tagged => self.decode_tagged(&variant_name, path, &variant.fields)?,
                Packing::Packed => self.decode_untagged(path, &variant.fields, needs)?,
                Packing::Transparent => {
                    self.decode_transparent(span, path, &variant.fields, needs)?
                }
            };

            let tag = self.expand_variant_tag(
                variant_attr.rename.as_ref(),
                self.type_attr.variant_tag,
                variant_index,
                &variant.ident,
            );

            variants.push(quote! {
                #tag => {
                    #output
                }
            });
        }

        Some(quote! {{
            let mut variant = #decoder_t::decode_variant(#decoder_var)?;
            let tag_decoder = #variant_decoder_t::decode_variant_tag(&mut variant)?;
            let tag = #decode_t::decode(tag_decoder)?;
            let #decoder_var = #variant_decoder_t::decode_variant_value(variant)?;

            Ok(match tag {
                #(#variants,)*
                tag => {
                    return Err(<D::Error as #error_t>::unsupported_variant(#type_name, tag));
                }
            })
        }})
    }

    fn encode_field_tag(
        &self,
        fields: &syn::Fields,
        tests: &[syn::Ident],
    ) -> Option<(TokenStream, Option<TokenStream>)> {
        let encoder_var = &self.tokens.encoder_var;
        let encoder_t = &self.tokens.encoder_t;

        match fields {
            syn::Fields::Named(..) => {
                let fields = calculate_tests(fields.len(), tests);

                let setup = quote! {
                    #encoder_t::encode_struct(#encoder_var, #fields)?
                };

                let struct_encoder_t = &self.tokens.struct_encoder_t;

                let finish = Some(quote! {
                    #struct_encoder_t::finish(#encoder_var)?;
                });

                Some((setup, finish))
            }
            syn::Fields::Unnamed(..) => {
                let fields = calculate_tests(fields.len(), tests);

                let setup = quote! {
                    #encoder_t::encode_tuple(#encoder_var, #fields)?
                };

                let tuple_encoder_t = &self.tokens.tuple_encoder_t;

                let finish = Some(quote! {
                    #tuple_encoder_t::finish(#encoder_var)?;
                });

                Some((setup, finish))
            }
            syn::Fields::Unit => {
                let setup = quote! {
                    #encoder_t::encode_unit_struct(#encoder_var)?
                };

                Some((setup, None))
            }
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
        let packing = variant_attr.packing().unwrap_or(self.type_attr.packing());

        let (mut encode, patterns) = match packing {
            Packing::Tagged => {
                let (encode, patterns, tests) =
                    self.encode_variant_fields(&variant.fields, needs, packing)?;

                // Special stuff needed to encode the field if its tagged.
                let (setup, finish) = self.encode_field_tag(&variant.fields, &tests)?;

                let encode = quote! {
                    let mut #encoder_var = #setup;
                    #encode
                    #finish
                };

                (encode, patterns)
            }
            Packing::Packed => {
                let (encode, patterns, _) =
                    self.encode_variant_fields(&variant.fields, needs, packing)?;
                (encode, patterns)
            }
            Packing::Transparent => self.transparent_variant(span, &variant.fields, needs)?,
        };

        if let Packing::Tagged = self.type_attr.packing() {
            needs.mark_used();

            let Tokens {
                encode_t,
                encoder_t,
                variant_encoder_t,
                ..
            } = &self.tokens;

            let tag = self.expand_variant_tag(
                variant_attr.rename.as_ref(),
                self.type_attr.variant_tag,
                variant_index,
                &variant.ident,
            );

            encode = quote! {{
                let mut variant_encoder = #encoder_t::encode_variant(#encoder_var)?;
                let tag_encoder = #variant_encoder_t::encode_variant_tag(&mut variant_encoder)?;
                #encode_t::encode(&#tag, tag_encoder)?;
                let #encoder_var = #variant_encoder_t::encode_variant_value(variant_encoder)?;
                #encode
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
    ) -> Option<(TokenStream, Vec<TokenStream>, Vec<syn::Ident>)> {
        let mut decls = Vec::with_capacity(fields.len());
        let mut encoders = Vec::with_capacity(fields.len());
        let mut patterns = Vec::with_capacity(fields.len());
        let mut field_tests = Vec::with_capacity(fields.len());

        if let Some((field_kind, fields)) = FieldKind::new(fields) {
            for (index, field) in fields.enumerate() {
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

                let (encoder, skip) =
                    self.encode_field(index, field, &field_attr, &access, field_kind, tagged)?;
                encoders.push(encoder);

                if let Some((decl, test)) = skip {
                    decls.push(decl);
                    field_tests.push(test);
                }
            }
        }

        let encode = match tagged {
            Packing::Tagged => {
                quote! {
                    #(#decls)*
                    #(#encoders)*
                }
            }
            Packing::Packed if !encoders.is_empty() => {
                let encoder_t = &self.tokens.encoder_t;
                let encoder_var = &self.tokens.encoder_var;
                let pack_encoder_t = &self.tokens.pack_encoder_t;

                quote! {
                    #(#decls)*
                    let mut pack = #encoder_t::encode_pack(#encoder_var)?;
                    #(#encoders)*
                    #pack_encoder_t::finish(pack)?;
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
        field_kind: FieldKind,
        tagged: Packing,
    ) -> Option<(TokenStream, Option<(TokenStream, syn::Ident)>)> {
        let encoder_var = &self.tokens.encoder_var;
        let encode_t = &self.tokens.encode_t;

        let (span, encode_path) = field_attr.encode_path(encode_t, field.span());

        let body = match tagged {
            Packing::Tagged | Packing::Transparent => {
                let tag = self.expand_field_tag(
                    field_attr.rename.as_ref(),
                    self.type_attr.field_tag,
                    index,
                    field,
                )?;

                match field_kind {
                    FieldKind::Struct => {
                        let struct_encoder_t = &self.tokens.struct_encoder_t;

                        quote_spanned! {
                            span => {
                                let field_encoder = #struct_encoder_t::encode_field_tag(&mut #encoder_var)?;
                                #encode_t::encode(&#tag, field_encoder)?;
                                let value_encoder = #struct_encoder_t::encode_field_value(&mut #encoder_var)?;
                                #encode_path(#access, value_encoder)?;
                            }
                        }
                    }
                    FieldKind::Tuple => {
                        let tuple_encoder_t = &self.tokens.tuple_encoder_t;

                        quote_spanned! {
                            span => {
                                let tag_encoder = #tuple_encoder_t::encode_field_tag(&mut #encoder_var)?;
                                #encode_t::encode(&#tag, tag_encoder)?;
                                let value_encoder = #tuple_encoder_t::encode_field_value(&mut #encoder_var)?;
                                #encode_path(#access, value_encoder)?;
                            }
                        }
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
        path: syn::Path,
        fields: &syn::Fields,
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

        for (index, field) in fields.enumerate() {
            let field_attr = attr::field_attrs(&self.cx, &field.attrs);

            let tag = self.expand_field_tag(
                field_attr.rename.as_ref(),
                self.type_attr.field_tag,
                index,
                field,
            )?;

            let (span, decode_path) = field_attr.decode_path(decode_t, field.span());
            let var = syn::Ident::new(&format!("v{}", index), span);
            decls.push(quote_spanned!(span => let mut #var = None;));
            let decode = quote_spanned!(span => #var = Some(#decode_path(#decoder_var)?));

            let prefix = match field_kind {
                FieldKind::Struct => {
                    let struct_field_decoder_t = &self.tokens.struct_field_decoder_t;

                    Some(quote! {
                        let #decoder_var = #struct_field_decoder_t::decode_field_value(&mut #decoder_var)?;
                    })
                }
                FieldKind::Tuple => {
                    let tuple_field_decoder_t = &self.tokens.tuple_field_decoder_t;

                    Some(quote! {
                        let #decoder_var = #tuple_field_decoder_t::decode_field_value(&mut #decoder_var)?;
                    })
                }
            };

            patterns.push(quote! {
                #tag => {
                    #prefix
                    #decode;
                }
            });

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
                quote!(return Err(<D::Error as #error_t>::missing_field(#type_name, #tag)))
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

        let decode_next = match field_kind {
            FieldKind::Struct => {
                let struct_decoder_t = &self.tokens.struct_decoder_t;
                quote!(#struct_decoder_t::decode_field)
            }
            FieldKind::Tuple => {
                let tuple_decoder_t = &self.tokens.tuple_decoder_t;
                quote!(#tuple_decoder_t::decode_field)
            }
        };

        let (decode_tag, skip_field) = match field_kind {
            FieldKind::Struct => {
                let struct_field_decoder_t = &self.tokens.struct_field_decoder_t;
                let decode_t = &self.tokens.decode_t;

                let decode_tag = quote! {{
                    let name_decoder = #struct_field_decoder_t::decode_field_tag(&mut #decoder_var)?;
                    #decode_t::decode(name_decoder)?
                }};

                let skip_field = quote! {
                    #struct_field_decoder_t::skip_field_value(&mut #decoder_var)?
                };

                (decode_tag, skip_field)
            }
            FieldKind::Tuple => {
                let tuple_field_decoder_t = &self.tokens.tuple_field_decoder_t;
                let decode_t = &self.tokens.decode_t;

                let decode_tag = quote! {{
                    let index_decoder = #tuple_field_decoder_t::decode_field_tag(&mut #decoder_var)?;
                    #decode_t::decode(index_decoder)?
                }};

                let skip_field = quote! {
                    #tuple_field_decoder_t::skip_field_value(&mut #decoder_var)?
                };

                (decode_tag, skip_field)
            }
        };

        if patterns.is_empty() {
            return Some(quote! {
                let mut #decoder_var = #declare;

                while let Some(mut #decoder_var) = #decode_next(&mut #decoder_var)? {
                    let tag = #decode_tag;

                    if !#skip_field {
                        return Err(<D::Error as #error_t>::unsupported_tag(#type_name, tag));
                    }
                }

                #path {}
            });
        }

        return Some(quote! {
            #(#decls;)*

            let mut #decoder_var = #declare;

            while let Some(mut #decoder_var) = #decode_next(&mut #decoder_var)? {
                match #decode_tag {
                    #(#patterns,)*
                    tag => {
                        if !#skip_field {
                            return Err(<D::Error as #error_t>::unsupported_tag(#type_name, tag));
                        }
                    },
                }
            }

            #path {
                #(#assigns),*
            }
        });
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
                #pack_decoder_t::finish(unpack)?;
                this
            }})
        }
    }

    /// Expand the variant tag depending on the given tag configuration.
    fn expand_variant_tag(
        &self,
        rename: Option<&(Span, attr::Rename)>,
        tag: Tag,
        index: usize,
        ident: &syn::Ident,
    ) -> syn::Lit {
        match (rename, tag) {
            (Some((_, rename)), _) => rename_lit(rename),
            (None, Tag::Index) => usize_int(index, ident.span()).into(),
            (None, Tag::Name) => syn::LitStr::new(&ident.to_string(), ident.span()).into(),
        }
    }

    /// Match out the field tag depending on the given tag configuration.
    fn expand_field_tag(
        &self,
        rename: Option<&(Span, attr::Rename)>,
        tag: Tag,
        index: usize,
        field: &syn::Field,
    ) -> Option<syn::Lit> {
        match (rename, tag, &field.ident) {
            (Some((_, rename)), _, _) => Some(rename_lit(rename)),
            (None, Tag::Index, _) => Some(usize_int(index, field.span()).into()),
            (None, Tag::Name, None) => {
                self.cx.error_spanned_by(
                    field,
                    format!(
                        "#[{}({} = \"name\")] is not supported with unnamed fields",
                        ATTR, TAG
                    ),
                );
                None
            }
            (None, Tag::Name, Some(ident)) => {
                Some(syn::LitStr::new(&ident.to_string(), ident.span()).into())
            }
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
    fn new<'a>(
        fields: &'a syn::Fields,
    ) -> Option<(
        Self,
        impl Iterator<Item = &'a syn::Field> + ExactSizeIterator,
    )> {
        match fields {
            syn::Fields::Named(named) => Some((Self::Struct, named.named.iter())),
            syn::Fields::Unnamed(unnamed) => Some((Self::Tuple, unnamed.unnamed.iter())),
            syn::Fields::Unit => None,
        }
    }
}

/// Process rename literal to ensure it's always typed.
fn rename_lit(rename: &attr::Rename) -> syn::Lit {
    match rename.as_lit() {
        syn::Lit::Int(int) if int.suffix().is_empty() => {
            syn::LitInt::new(&format!("{}usize", int.to_string()), int.span()).into()
        }
        lit => lit.clone(),
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
