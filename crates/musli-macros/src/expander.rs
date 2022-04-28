use std::collections::BTreeSet;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

use crate::internals::attr::{self, DefaultTag, Packing, TypeAttr};
use crate::internals::symbol::*;
use crate::internals::tokens::Tokens;
use crate::internals::{Ctxt, Mode, ModePath, Needs, NeedsKind};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

enum ExpansionMode<'a> {
    Generic { mode_ident: &'a syn::Ident },
    Default,
    Moded { mode_ident: &'a syn::ExprPath },
}

impl ExpansionMode<'_> {
    fn as_mode<'a>(&'a self, tokens: &'a Tokens) -> Mode<'a> {
        match *self {
            ExpansionMode::Generic { mode_ident } => Mode {
                ident: None,
                mode_path: ModePath::Ident(mode_ident),
                tokens,
            },
            ExpansionMode::Default => Mode {
                ident: None,
                mode_path: ModePath::Path(&tokens.default_mode),
                tokens,
            },
            ExpansionMode::Moded { mode_ident } => Mode {
                ident: Some(mode_ident),
                mode_path: ModePath::Path(mode_ident),
                tokens,
            },
        }
    }

    /// Coerce into impl generics.
    fn as_impl_generics(
        &self,
        generics: syn::Generics,
        tokens: &Tokens,
    ) -> (syn::Generics, syn::ExprPath) {
        match *self {
            ExpansionMode::Generic { mode_ident } => {
                let mut impl_generics = generics.clone();

                impl_generics
                    .params
                    .push(syn::TypeParam::from(mode_ident.clone()).into());

                let path = syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: syn::Path::from(mode_ident.clone()),
                };

                (impl_generics, path)
            }
            ExpansionMode::Default => (generics, tokens.default_mode.clone()),
            ExpansionMode::Moded { mode_ident } => (generics, mode_ident.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum StructKind {
    Named,
    Unnamed,
    Unit,
}

struct FieldData<'a> {
    span: Span,
    attr: attr::FieldAttr,
    ident: Option<&'a syn::Ident>,
}

struct StructData<'a> {
    fields: Vec<FieldData<'a>>,
    kind: StructKind,
}

struct VariantData<'a> {
    span: Span,
    attr: attr::VariantAttr,
    ident: &'a syn::Ident,
    kind: StructKind,
    fields: Vec<FieldData<'a>>,
}

struct EnumData<'a> {
    variants: Vec<VariantData<'a>>,
}

enum Data<'a> {
    Struct(StructData<'a>),
    Enum(EnumData<'a>),
    Union,
}

pub(crate) struct Expander<'a> {
    pub(crate) input: &'a syn::DeriveInput,
    cx: Ctxt,
    type_attr: TypeAttr,
    type_name: syn::LitStr,
    data: Data<'a>,
    tokens: Tokens,
}

impl<'a> Expander<'a> {
    pub(crate) fn new(input: &'a syn::DeriveInput) -> Self {
        let cx = Ctxt::new();
        let type_attr = attr::type_attrs(&cx, &input.attrs);
        let type_name = syn::LitStr::new(&input.ident.to_string(), input.ident.span());

        let data = match &input.data {
            syn::Data::Struct(st) => {
                let fields = st.fields.iter().map(|field| FieldData {
                    span: field.span(),
                    attr: attr::field_attrs(&cx, &field.attrs),
                    ident: field.ident.as_ref(),
                });

                Data::Struct(StructData {
                    fields: fields.collect(),
                    kind: match &st.fields {
                        syn::Fields::Named(_) => StructKind::Named,
                        syn::Fields::Unnamed(_) => StructKind::Unnamed,
                        syn::Fields::Unit => StructKind::Unit,
                    },
                })
            }
            syn::Data::Enum(en) => {
                let variants = en.variants.iter().map(|variant| {
                    let fields = variant.fields.iter().map(|field| FieldData {
                        span: field.span(),
                        attr: attr::field_attrs(&cx, &field.attrs),
                        ident: field.ident.as_ref(),
                    });

                    VariantData {
                        span: variant.span(),
                        attr: attr::variant_attrs(&cx, &variant.attrs),
                        ident: &variant.ident,
                        fields: fields.collect(),
                        kind: match &variant.fields {
                            syn::Fields::Named(_) => StructKind::Named,
                            syn::Fields::Unnamed(_) => StructKind::Unnamed,
                            syn::Fields::Unit => StructKind::Unit,
                        },
                    }
                });

                Data::Enum(EnumData {
                    variants: variants.collect(),
                })
            }
            syn::Data::Union(..) => Data::Union,
        };

        let prefix = type_attr.crate_or_default();

        Self {
            input,
            cx,
            type_attr,
            type_name,
            data,
            tokens: Tokens::new(input.ident.span(), &prefix),
        }
    }

    /// Coerce into errors.
    pub(crate) fn into_errors(self) -> Vec<syn::Error> {
        self.cx.into_errors()
    }

    /// Expand Encode implementation.
    pub(crate) fn expand_encode(&self) -> Option<TokenStream> {
        let modes = self.cx.modes();

        let mode_ident = syn::Ident::new("Mode", self.type_name.span());

        if modes.is_empty() {
            return self.expand_encode_moded(ExpansionMode::Generic {
                mode_ident: &mode_ident,
            });
        }

        let mut out = TokenStream::new();

        for mode in modes {
            out.extend(self.expand_encode_moded(ExpansionMode::Moded { mode_ident: &mode })?);
        }

        out.extend(self.expand_encode_moded(ExpansionMode::Default)?);
        Some(out)
    }

    fn expand_encode_moded(&self, expansion: ExpansionMode) -> Option<TokenStream> {
        let span = self.input.ident.span();

        let encoder_var = &self.tokens.encoder_var;

        let type_ident = &self.input.ident;

        let mut needs = Needs::default();

        let mode = expansion.as_mode(&self.tokens);

        let body = match &self.data {
            Data::Struct(data) => self.encode_struct(mode, data, &mut needs)?,
            Data::Enum(data) => self.encode_enum(mode, data, &mut needs)?,
            Data::Union => {
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

        let (impl_generics, mode_ident) =
            expansion.as_impl_generics(self.input.generics.clone(), &self.tokens);

        let type_generics = &self.input.generics;

        Some(quote_spanned! {
            span =>
            #[automatically_derived]
            impl #impl_generics #encode_t<#mode_ident> for #type_ident #type_generics {
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

    fn transparent_diagnostics(&self, span: Span, fields: &[FieldData]) {
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
        mode: Mode<'_>,
        span: Span,
        fields: &[FieldData],
        needs: &mut Needs,
    ) -> Option<TokenStream> {
        let first_field = fields.iter().next();

        let (span, accessor, field_attr) = match first_field {
            Some(field) if fields.len() == 1 => {
                let accessor = match &field.ident {
                    Some(ident) => quote_spanned!(field.span => &self.#ident),
                    None => quote_spanned!(field.span => &self.0),
                };

                (field.span, accessor, &field.attr)
            }
            _ => {
                self.transparent_diagnostics(span, fields);
                return None;
            }
        };

        let encoder_var = &self.tokens.encoder_var;

        needs.mark_used();

        let (span, encode_path) = field_attr.encode_path(mode, span);

        Some(quote_spanned! {
            span => #encode_path(#accessor, #encoder_var)
        })
    }

    /// Encode a transparent element.
    fn transparent_variant(
        &self,
        mode: Mode<'_>,
        span: Span,
        fields: &[FieldData],
        needs: &mut Needs,
    ) -> Option<(TokenStream, Vec<TokenStream>)> {
        let ident = fields.iter().next();

        let (span, pattern, field_attr) = match ident {
            Some(field) if fields.len() == 1 => {
                let accessor = match &field.ident {
                    Some(ident) => quote_spanned!(field.span => #ident: this),
                    None => quote_spanned!(field.span => 0: this),
                };

                (field.span, accessor, &field.attr)
            }
            _ => {
                self.transparent_diagnostics(span, fields);
                return None;
            }
        };

        let encoder_var = &self.tokens.encoder_var;

        needs.mark_used();

        let (span, encode_path) = field_attr.encode_path(mode, span);

        let encode = quote_spanned! {
            span => #encode_path(this, #encoder_var)
        };

        Some((encode, vec![pattern]))
    }

    /// Decode a transparent value.
    fn decode_transparent(
        &self,
        mode: Mode<'_>,
        span: Span,
        path: syn::Path,
        fields: &[FieldData],
        needs: &mut Needs,
    ) -> Option<TokenStream> {
        let ident = fields.iter().next();

        let (accessor, field) = match ident {
            Some(field) if fields.len() == 1 => {
                let accessor = match &field.ident {
                    Some(ident) => quote!(#ident),
                    None => quote!(0),
                };

                (accessor, field)
            }
            _ => {
                self.transparent_diagnostics(span, fields);
                return None;
            }
        };

        let decoder_var = &self.tokens.decoder_var;

        needs.mark_used();
        needs.mark_inline();

        let (span, decode_path) = field.attr.decode_path(mode, span);

        Some(quote_spanned! {
            span =>
            #path {
                #accessor: #decode_path(#decoder_var)?
            }
        })
    }

    /// Encode a struct.
    fn encode_struct(
        &self,
        mode: Mode<'_>,
        st: &StructData,
        needs: &mut Needs,
    ) -> Option<TokenStream> {
        let mut field_tests = Vec::with_capacity(st.fields.len());
        let mut encoders = Vec::with_capacity(st.fields.len());
        let mut test_variables = Vec::with_capacity(st.fields.len());
        let encoder_var = &self.tokens.encoder_var;
        let packing = self.type_attr.packing_or_default(mode);

        for (index, field) in st.fields.iter().enumerate() {
            needs.mark_used();

            let access = match &field.ident {
                Some(ident) => quote!(&self.#ident),
                None => {
                    let n = field_int(index, field.span);
                    quote!(&self.#n)
                }
            };

            let (encoder, skip) = self.encode_field(
                mode,
                index,
                field,
                &access,
                packing,
                self.type_attr.default_field_tag(mode),
            )?;

            encoders.push(encoder);

            if let Some((decl, test)) = skip {
                field_tests.push(decl);
                test_variables.push(test);
            }
        }

        let encode = match packing {
            Packing::Transparent => {
                self.encode_transparent(mode, self.input.ident.span(), &st.fields, needs)?
            }
            Packing::Tagged => {
                needs.mark_used();
                let encode = quote! { #(#encoders)* };
                self.encode_field_tag(&st.fields, st.kind, encode, &field_tests, &test_variables)
            }
            Packing::Packed => {
                needs.mark_used();

                let encoder_t = &self.tokens.encoder_t;
                let sequence_encoder_t = &self.tokens.sequence_encoder_t;

                quote! {{
                    let mut pack = #encoder_t::encode_pack(#encoder_var)?;
                    #(#field_tests)*
                    #(#encoders)*
                    #sequence_encoder_t::end(pack)
                }}
            }
        };

        Some(encode)
    }

    fn encode_enum(
        &self,
        mode: Mode<'_>,
        data: &EnumData,
        needs: &mut Needs,
    ) -> Option<TokenStream> {
        if let Some((span, Packing::Transparent)) = self.type_attr.packing(mode) {
            self.cx.error_span(
                span,
                format!("#[{}({})] cannot be used on enums", ATTR, TRANSPARENT),
            );
            return None;
        }

        let mut variants = Vec::with_capacity(data.variants.len());

        for (variant_index, variant) in data.variants.iter().enumerate() {
            if let Some(variant) = self.encode_variant(mode, variant_index, variant, needs) {
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
        let modes = self.cx.modes();

        let mode_ident = syn::Ident::new("Mode", self.type_name.span());

        if modes.is_empty() {
            return self.expand_decode_moded(ExpansionMode::Generic {
                mode_ident: &mode_ident,
            });
        }

        let mut out = TokenStream::new();

        for mode in modes {
            out.extend(self.expand_decode_moded(ExpansionMode::Moded { mode_ident: &mode })?);
        }

        out.extend(self.expand_decode_moded(ExpansionMode::Default)?);
        Some(out)
    }

    fn expand_decode_moded(&self, expansion: ExpansionMode) -> Option<TokenStream> {
        let span = self.input.ident.span();

        let decoder_var = &self.tokens.decoder_var;
        let mut impl_generics = self.input.generics.clone();
        let type_ident = &self.input.ident;

        let (lt, exists) = if let Some(existing) = impl_generics.lifetimes().next() {
            (existing.clone(), true)
        } else {
            let lt = syn::LifetimeDef::new(syn::Lifetime::new("'de", self.input.span()));
            (lt, false)
        };

        if !exists {
            impl_generics.params.push(lt.clone().into());
        }

        let mode = expansion.as_mode(&self.tokens);

        let mut needs = Needs::default();

        let body = match &self.data {
            Data::Struct(data) => self.decode_struct(mode, span, data, &mut needs)?,
            Data::Enum(data) => self.decode_enum(mode, data, &mut needs)?,
            Data::Union => {
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
        let original_generics = &self.input.generics;

        let (impl_generics, mode_ident) = expansion.as_impl_generics(impl_generics, &self.tokens);

        Some(quote_spanned! {
            span =>
            #[automatically_derived]
            impl #impl_generics #decode_t<#lt, #mode_ident> for #type_ident #original_generics {
                #inline
                fn decode<D>(#assignment: D) -> Result<Self, D::Error>
                where
                    D: #decoder_t<#lt>
                {
                    #body
                }
            }
        })
    }

    fn decode_struct(
        &self,
        mode: Mode<'_>,
        span: Span,
        data: &StructData,
        needs: &mut Needs,
    ) -> Option<TokenStream> {
        let path = syn::Path::from(syn::Ident::new("Self", self.input.ident.span()));
        let tag_type = self.type_attr.tag_type(mode);

        let body = match self.type_attr.packing_or_default(mode) {
            Packing::Tagged => {
                needs.mark_used();
                self.decode_tagged(
                    mode,
                    &self.type_name,
                    tag_type,
                    path,
                    &data.fields,
                    data.kind,
                    None,
                    self.type_attr.default_field_tag(mode),
                )?
            }
            Packing::Packed => self.decode_untagged(mode, path, &data.fields, needs)?,
            Packing::Transparent => {
                self.decode_transparent(mode, span, path, &data.fields, needs)?
            }
        };

        Some(quote! {
            Ok({
                #body
            })
        })
    }

    fn decode_enum(
        &self,
        mode: Mode<'_>,
        data: &EnumData,
        needs: &mut Needs,
    ) -> Option<TokenStream> {
        let decoder_t = &self.tokens.decoder_t;
        let decoder_var = &self.tokens.decoder_var;
        let error_t = &self.tokens.error_t;
        let type_ident = &self.input.ident;
        let type_name = &self.type_name;
        let pair_decoder_t = &self.tokens.pair_decoder_t;
        let variant_tag = syn::Ident::new("variant_tag", self.input.ident.span());

        if let Some((span, Packing::Packed)) = self.type_attr.packing(mode) {
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

        let type_packing = self.type_attr.packing_or_default(mode);

        let tag_visitor_output =
            syn::Ident::new("VariantTagVisitorOutput", self.input.ident.span());
        let mut string_patterns = Vec::with_capacity(data.variants.len());
        let mut output_variants = Vec::with_capacity(data.variants.len());
        // Collect variant names so that we can generate a debug implementation.
        let mut output_names = Vec::with_capacity(data.variants.len());

        let mut patterns = Vec::with_capacity(data.variants.len());
        let mut fallback = None;
        // Keep track of variant index manually since fallback variants do not
        // count.
        let mut variant_index = 0;
        let mut tag_methods = TagMethods::new(&self.cx);

        for variant in data.variants.iter() {
            let span = variant.span;

            if variant.attr.default_attr(mode).is_some() {
                if !variant.fields.is_empty() {
                    self.cx.error_span(
                        variant.span,
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

            let tag_type = variant.attr.tag_type(mode);

            let default_field_tag = variant
                .attr
                .default_field_tag(mode)
                .unwrap_or_else(|| self.type_attr.default_field_tag(mode));

            let decode = match variant.attr.packing(mode).unwrap_or(type_packing) {
                Packing::Tagged => self.decode_tagged(
                    mode,
                    &variant_name,
                    tag_type,
                    path,
                    &variant.fields,
                    variant.kind,
                    Some(&variant_tag),
                    default_field_tag,
                )?,
                Packing::Packed => self.decode_untagged(mode, path, &variant.fields, needs)?,
                Packing::Transparent => {
                    self.decode_transparent(mode, span, path, &variant.fields, needs)?
                }
            };

            let (tag, tag_method) = self.expand_tag(
                variant.span,
                variant.attr.rename(mode),
                self.type_attr.default_variant_tag(mode),
                variant_index,
                Some(&variant.ident),
            )?;

            tag_methods.insert(variant.span, tag_method);

            let output_tag = self.handle_output_tag(
                span,
                variant_index,
                tag_method,
                &tag,
                &tag_visitor_output,
                &mut string_patterns,
                &mut output_variants,
            );

            output_names.push(syn::LitStr::new(&variant.ident.to_string(), span));
            patterns.push((output_tag, decode));
            variant_index += 1;
        }

        let tag_type = self
            .type_attr
            .tag_type(mode)
            .as_ref()
            .map(|(_, ty)| quote!(: #ty));

        let fallback = match fallback {
            Some(ident) => {
                quote! {
                    if !#pair_decoder_t::skip_second(#decoder_var)? {
                        return Err(<D::Error as #error_t>::invalid_variant_tag(#type_name, tag));
                    }

                    Self::#ident {}
                }
            }
            None => quote! {
                return Err(<D::Error as #error_t>::invalid_variant_tag(#type_name, tag));
            },
        };

        let (decode_tag, unsupported_pattern, patterns, output_enum) = self.handle_tag_decode(
            mode,
            tag_methods.pick(),
            &tag_visitor_output,
            &output_variants,
            &string_patterns,
            &patterns,
        )?;

        // A `std::fmt::Debug` implementation is necessary for the output enum
        // since it is used to produce diagnostics.
        let output_enum_debug_impl = output_enum.is_some().then(|| {
            let fmt = &self.tokens.fmt;

            let mut patterns = Vec::new();

            for (name, variant) in output_names.iter().zip(output_variants.iter()) {
                patterns.push(quote!(#tag_visitor_output::#variant => #name.fmt(f)));
            }

            quote! {
                impl #fmt::Debug for #tag_visitor_output {
                    #[inline]
                    fn fmt(&self, f: &mut #fmt::Formatter<'_>) -> #fmt::Result {
                        match self { #(#patterns),* }
                    }
                }
            }
        });

        let patterns = patterns
            .into_iter()
            .map(|(tag, output)| {
                quote! {
                    #tag => {
                        let #decoder_var = #pair_decoder_t::second(#decoder_var)?;
                        #output
                    }
                }
            })
            .collect::<Vec<_>>();

        Some(quote! {
            let mut #decoder_var = #decoder_t::decode_variant(#decoder_var)?;
            #output_enum
            #output_enum_debug_impl

            let #variant_tag #tag_type = #decode_tag;

            Ok(match #variant_tag {
                #(#patterns,)*
                #unsupported_pattern => {
                    #fallback
                }
            })
        })
    }

    fn encode_field_tag(
        &self,
        fields: &[FieldData],
        kind: StructKind,
        encode: TokenStream,
        field_tests: &[TokenStream],
        test_variables: &[syn::Ident],
    ) -> TokenStream {
        let encoder_var = &self.tokens.encoder_var;
        let encoder_t = &self.tokens.encoder_t;
        let pairs_encoder_t = &self.tokens.pairs_encoder_t;

        match kind {
            StructKind::Named => {
                let len = calculate_tests(fields.len(), test_variables);
                quote! {{
                    #(#field_tests)*
                    let mut #encoder_var = #encoder_t::encode_struct(#encoder_var, #len)?;
                    #encode
                    #pairs_encoder_t::end(#encoder_var)
                }}
            }
            StructKind::Unnamed => {
                let len = calculate_tests(fields.len(), test_variables);
                quote! {{
                    #(#field_tests)*
                    let mut #encoder_var = #encoder_t::encode_tuple_struct(#encoder_var, #len)?;
                    #encode
                    #pairs_encoder_t::end(#encoder_var)
                }}
            }
            StructKind::Unit => {
                quote!(#encoder_t::encode_unit_struct(#encoder_var))
            }
        }
    }

    /// Setup encoding for a single variant.
    fn encode_variant(
        &self,
        mode: Mode<'_>,
        variant_index: usize,
        variant: &VariantData,
        needs: &mut Needs,
    ) -> Option<TokenStream> {
        let span = variant.span;

        let encoder_var = &self.tokens.encoder_var;
        let packing = variant
            .attr
            .packing(mode)
            .or_else(|| Some(self.type_attr.packing(mode)?.1))
            .unwrap_or_default();

        let default_field_tag = variant
            .attr
            .default_field_tag(mode)
            .unwrap_or_else(|| self.type_attr.default_field_tag(mode));

        let (mut encode, patterns) = match packing {
            Packing::Tagged => {
                let (encode, patterns, tests) = self.encode_variant_fields(
                    mode,
                    &variant.fields,
                    needs,
                    packing,
                    default_field_tag,
                )?;

                // Special stuff needed to encode the field if its tagged.
                let encode =
                    self.encode_field_tag(&variant.fields, variant.kind, encode, &[], &tests);
                (encode, patterns)
            }
            Packing::Packed => {
                let (encode, patterns, _) = self.encode_variant_fields(
                    mode,
                    &variant.fields,
                    needs,
                    packing,
                    default_field_tag,
                )?;
                (encode, patterns)
            }
            Packing::Transparent => self.transparent_variant(mode, span, &variant.fields, needs)?,
        };

        if let Packing::Tagged = self.type_attr.packing_or_default(mode) {
            needs.mark_used();

            let Tokens {
                encoder_t,
                pair_encoder_t,
                ..
            } = &self.tokens;

            let (tag, _) = self.expand_tag(
                variant.span,
                variant.attr.rename(mode),
                self.type_attr.default_variant_tag(mode),
                variant_index,
                Some(&variant.ident),
            )?;

            let encode_t_encode = mode.encode_t_encode();

            let body = quote! {
                let tag_encoder = #pair_encoder_t::first(&mut variant_encoder)?;
                #encode_t_encode(&#tag, tag_encoder)?;
                let #encoder_var = #pair_encoder_t::second(&mut variant_encoder)?;
                #encode?;
                #pair_encoder_t::end(variant_encoder)
            };

            encode = match &variant.kind {
                StructKind::Named => {
                    let len = variant.fields.len();

                    quote! {{
                        let mut variant_encoder = #encoder_t::encode_struct_variant(#encoder_var, #len)?;
                        #body
                    }}
                }
                StructKind::Unnamed => {
                    let len = variant.fields.len();

                    quote! {{
                        let mut variant_encoder = #encoder_t::encode_tuple_variant(#encoder_var, #len)?;
                        #body
                    }}
                }
                StructKind::Unit => {
                    quote! {{
                        let mut variant_encoder = #encoder_t::encode_unit_variant(#encoder_var)?;
                        #body
                    }}
                }
            };
        }

        let mut path = syn::Path::from(syn::Ident::new("Self", span));
        path.segments.push(variant.ident.clone().into());

        Some(quote! {
            #path { #(#patterns),* } => { #encode }
        })
    }

    fn encode_variant_fields(
        &self,
        mode: Mode<'_>,
        fields: &[FieldData],
        needs: &mut Needs,
        tagged: Packing,
        default_field_tag: DefaultTag,
    ) -> Option<(TokenStream, Vec<TokenStream>, Vec<syn::Ident>)> {
        let mut field_tests = Vec::with_capacity(fields.len());
        let mut encoders = Vec::with_capacity(fields.len());
        let mut patterns = Vec::with_capacity(fields.len());
        let mut test_variables = Vec::with_capacity(fields.len());

        for (index, field) in fields.iter().enumerate() {
            needs.mark_used();

            let access = match &field.ident {
                Some(ident) => {
                    patterns.push(quote!(#ident));
                    quote!(#ident)
                }
                None => {
                    let index = field_int(index, field.span);
                    let var = syn::Ident::new(&format!("v{}", index), field.span);
                    patterns.push(quote!(#index: #var));
                    quote!(#var)
                }
            };

            let (encoder, skip) =
                self.encode_field(mode, index, field, &access, tagged, default_field_tag)?;
            encoders.push(encoder);

            if let Some((decl, test)) = skip {
                field_tests.push(decl);
                test_variables.push(test);
            }
        }

        let encode = match tagged {
            Packing::Tagged => {
                quote! {
                    #(#field_tests)*
                    #(#encoders)*
                }
            }
            Packing::Packed => {
                needs.mark_used();

                let encoder_t = &self.tokens.encoder_t;
                let encoder_var = &self.tokens.encoder_var;
                let sequence_encoder_t = &self.tokens.sequence_encoder_t;

                quote! {{
                    let mut pack = #encoder_t::encode_pack(#encoder_var)?;
                    #(#field_tests)*
                    #(#encoders)*
                    #sequence_encoder_t::end(pack)
                }}
            }
            _ => quote!(),
        };

        Some((encode, patterns, test_variables))
    }

    /// Encode a field.
    fn encode_field(
        &self,
        mode: Mode<'_>,
        index: usize,
        field: &FieldData,
        access: &TokenStream,
        tagged: Packing,
        default_field_tag: DefaultTag,
    ) -> Option<(TokenStream, Option<(TokenStream, syn::Ident)>)> {
        let encoder_var = &self.tokens.encoder_var;

        let (span, encode_path) = field.attr.encode_path(mode, field.span);

        let body = match tagged {
            Packing::Tagged | Packing::Transparent => {
                let (tag, _) = self.expand_tag(
                    field.span,
                    field.attr.rename(mode),
                    default_field_tag,
                    index,
                    field.ident,
                )?;

                let pair_encoder_t = &self.tokens.pair_encoder_t;
                let pairs_encoder_t = &self.tokens.pairs_encoder_t;

                let encode_t_encode = mode.encode_t_encode();

                quote_spanned! {
                    span => {
                        let mut pair_encoder = #pairs_encoder_t::next(&mut #encoder_var)?;
                        let field_encoder = #pair_encoder_t::first(&mut pair_encoder)?;
                        #encode_t_encode(&#tag, field_encoder)?;
                        let value_encoder = #pair_encoder_t::second(&mut pair_encoder)?;
                        #encode_path(#access, value_encoder)?;
                        #pair_encoder_t::end(pair_encoder)?;
                    }
                }
            }
            Packing::Packed => {
                let sequence_encoder_t = &self.tokens.sequence_encoder_t;
                quote_spanned!(span => #encode_path(#access, #sequence_encoder_t::next(&mut pack)?)?;)
            }
        };

        // Add condition to encode a field if configured.
        if let Some((skip_span, skip_encoding_if_path)) = field.attr.skip_encoding_if(mode) {
            let test = syn::Ident::new(&format!("t{}", index), field.span);

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
        mode: Mode<'_>,
        type_name: &syn::LitStr,
        tag_type: Option<&(Span, syn::Type)>,
        path: syn::Path,
        fields: &[FieldData],
        kind: StructKind,
        variant_tag: Option<&syn::Ident>,
        default_field_tag: DefaultTag,
    ) -> Option<TokenStream> {
        let decoder_t = &self.tokens.decoder_t;
        let decoder_var = &self.tokens.decoder_var;
        let error_t = &self.tokens.error_t;
        let default_t = &self.tokens.default_t;
        let pairs_decoder_t = &self.tokens.pairs_decoder_t;

        let fields_len = fields.len();
        let mut decls = Vec::with_capacity(fields_len);
        let mut patterns = Vec::with_capacity(fields_len);
        let mut assigns = Vec::with_capacity(fields_len);

        let tag_visitor_output = syn::Ident::new("TagVisitorOutput", self.input.ident.span());
        let mut string_patterns = Vec::with_capacity(fields_len);
        let mut output_variants = Vec::with_capacity(fields_len);

        let mut tag_methods = TagMethods::new(&self.cx);

        for (index, field) in fields.iter().enumerate() {
            let (tag, tag_method) = self.expand_tag(
                field.span,
                field.attr.rename(mode),
                default_field_tag,
                index,
                field.ident,
            )?;

            tag_methods.insert(field.span, tag_method);

            let (span, decode_path) = field.attr.decode_path(mode, field.span);
            let var = syn::Ident::new(&format!("v{}", index), span);
            decls.push(quote_spanned!(span => let mut #var = None;));
            let decode = quote_spanned!(span => #var = Some(#decode_path(#decoder_var)?));

            let output_tag = self.handle_output_tag(
                span,
                index,
                tag_method,
                &tag,
                &tag_visitor_output,
                &mut string_patterns,
                &mut output_variants,
            );

            patterns.push((output_tag, decode));

            let field_ident = match &field.ident {
                Some(ident) => quote!(#ident),
                None => {
                    let field = field_int(index, field.span);
                    quote!(#field)
                }
            };

            let fallback = if let Some(span) = field.attr.default_attr(mode) {
                quote_spanned!(span => #default_t)
            } else {
                quote!(return Err(<D::Error as #error_t>::expected_tag(#type_name, #tag)))
            };

            assigns.push(quote!(#field_ident: match #var {
                Some(#var) => #var,
                None => #fallback,
            }));
        }

        let pair_decoder_t = &self.tokens.pair_decoder_t;

        let (decode_tag, unsupported_pattern, patterns, output_enum) = self.handle_tag_decode(
            mode,
            tag_methods.pick(),
            &tag_visitor_output,
            &output_variants,
            &string_patterns,
            &patterns,
        )?;

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

        let body = quote! {
            while let Some(mut #decoder_var) = #pairs_decoder_t::next(&mut type_decoder)? {
                let tag #tag_type = #decode_tag;
                #body
            }

            #path { #(#assigns),* }
        };

        let body = match kind {
            StructKind::Named => quote! {{
                let mut type_decoder = #decoder_t::decode_struct(#decoder_var, #fields_len)?;
                #body
            }},
            StructKind::Unnamed => quote! {
                let mut type_decoder = #decoder_t::decode_tuple_struct(#decoder_var, #fields_len)?;
                #body
            },
            StructKind::Unit => {
                quote! {
                    #decoder_t::decode_unit_struct(#decoder_var)?;
                    #path {}
                }
            }
        };

        Some(quote! {
            #(#decls;)*
            #output_enum
            #body
        })
    }

    fn handle_output_tag(
        &self,
        span: Span,
        index: usize,
        tag_method: TagMethod,
        tag: &syn::Expr,
        tag_visitor_output: &syn::Ident,
        string_patterns: &mut Vec<TokenStream>,
        output_variants: &mut Vec<syn::Ident>,
    ) -> syn::Expr {
        match tag_method {
            TagMethod::String => {
                let variant = syn::Ident::new(&format!("Variant{}", index), span);
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
        }
    }

    /// Handle tag decoding.
    fn handle_tag_decode(
        &self,
        mode: Mode<'_>,
        tag_method: TagMethod,
        tag_visitor_output: &syn::Ident,
        output_variants: &[syn::Ident],
        string_patterns: &[TokenStream],
        patterns: &[(syn::Expr, TokenStream)],
    ) -> Option<(
        TokenStream,
        TokenStream,
        Vec<(TokenStream, TokenStream)>,
        Option<TokenStream>,
    )> {
        let decoder_var = &self.tokens.decoder_var;
        let pair_decoder_t = &self.tokens.pair_decoder_t;

        match tag_method {
            TagMethod::String => {
                let (decode_tag, output_enum) = self.string_variant_tag_decode(
                    tag_visitor_output,
                    output_variants,
                    string_patterns,
                )?;

                let patterns = patterns
                    .iter()
                    .map(|(tag, decode)| (quote!(Ok(#tag)), decode.clone()))
                    .collect::<Vec<_>>();

                Some((decode_tag, quote!(Err(tag)), patterns, Some(output_enum)))
            }
            TagMethod::Default => {
                let decode_t_decode = mode.decode_t_decode();

                let decode_tag = quote! {{
                    let index_decoder = #pair_decoder_t::first(&mut #decoder_var)?;
                    #decode_t_decode(index_decoder)?
                }};

                let patterns = patterns
                    .iter()
                    .map(|(tag, decode)| (quote!(#tag), decode.clone()))
                    .collect::<Vec<_>>();

                Some((decode_tag, quote!(tag), patterns, None))
            }
        }
    }

    fn string_variant_tag_decode(
        &self,
        output: &syn::Ident,
        output_variants: &[syn::Ident],
        string_patterns: &[TokenStream],
    ) -> Option<(TokenStream, TokenStream)> {
        let value_visitor_t = &self.tokens.value_visitor_t;
        let phantom_data = &self.tokens.phantom_data;
        let fmt = &self.tokens.fmt;
        let error_t = &self.tokens.error_t;
        let decoder_t = &self.tokens.decoder_t;
        let decoder_var = &self.tokens.decoder_var;
        let pair_decoder_t = &self.tokens.pair_decoder_t;

        // Declare a tag visitor, allowing string tags to be decoded by
        // decoders that owns the string.
        let decode_tag = quote! {{
            struct TagVisitor<E>(#phantom_data<E>);

            impl<'de, E> #value_visitor_t<'de> for TagVisitor<E> where E: #error_t {
                type Target = str;
                type Ok = Result<#output, Box<str>>;
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

        let output_enum = quote! {
            enum #output {
                #(#output_variants,)*
            }
        };

        Some((decode_tag, output_enum))
    }

    /// Decode something packed.
    fn decode_untagged(
        &self,
        mode: Mode<'_>,
        path: syn::Path,
        fields: &[FieldData],
        needs: &mut Needs,
    ) -> Option<TokenStream> {
        let pack_decoder_t = &self.tokens.pack_decoder_t;
        let decoder_var = &self.tokens.decoder_var;
        let decoder_t = &self.tokens.decoder_t;

        let mut assign = Vec::new();

        for (index, field) in fields.iter().enumerate() {
            needs.mark_used();

            if let Some(span) = field.attr.default_attr(mode) {
                self.cx.error_span(
                    span,
                    format!(
                        "#[{}({})] fields cannot be used in an packed container",
                        ATTR, DEFAULT
                    ),
                );
            }

            let (span, decode_path) = field.attr.decode_path(mode, field.span);

            let decode = quote! {{
                let field_decoder = #pack_decoder_t::next(&mut unpack)?;
                #decode_path(field_decoder)?
            }};

            match field.ident {
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
                #path { #(#assign),* }
            }})
        }
    }

    /// Expand the given configuration to the appropriate tag expression and
    /// [TagMethod].
    fn expand_tag(
        &self,
        span: Span,
        rename: Option<&(Span, syn::Expr)>,
        default_field_tag: DefaultTag,
        index: usize,
        ident: Option<&syn::Ident>,
    ) -> Option<(syn::Expr, TagMethod)> {
        let (lit, tag_method) = match (rename, default_field_tag, ident) {
            (Some((_, rename)), _, _) => {
                return Some((self.rename_lit(rename), self.determine_tag_method(rename)))
            }
            (None, DefaultTag::Index, _) => (usize_int(index, span).into(), TagMethod::Default),
            (None, DefaultTag::Name, None) => {
                self.cx.error_span(
                    span,
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

/// Usize-suffixed integer.
fn usize_int(index: usize, span: Span) -> syn::LitInt {
    syn::LitInt::new(&format!("{}usize", index), span)
}

/// Integer used for tuple initialization.
fn field_int(index: usize, span: Span) -> syn::LitInt {
    syn::LitInt::new(&index.to_string(), span)
}

struct TagMethods<'a> {
    cx: &'a Ctxt,
    methods: BTreeSet<TagMethod>,
}

impl<'a> TagMethods<'a> {
    fn new(cx: &'a Ctxt) -> Self {
        Self {
            cx,
            methods: BTreeSet::new(),
        }
    }

    /// Insert a tag method and error in case it's invalid.
    fn insert(&mut self, span: Span, method: TagMethod) {
        let before = self.methods.len();
        self.methods.insert(method);

        if before == 1 && self.methods.len() > 1 {
            self.cx
                .error_span(span, format!("#[{}({})] conflicting tag kind", ATTR, TAG));
        }
    }

    /// Pick a tag method to use.
    fn pick(self) -> TagMethod {
        self.methods.into_iter().next().unwrap_or_default()
    }
}
