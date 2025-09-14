use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::Token;
use syn::punctuated::Punctuated;

use crate::expander::Name;
use crate::internals::attr::{EnumTagging, Packing};
use crate::internals::build::{self, Body, Build, BuildData, Enum, Field, Variant};
use crate::internals::{Result, Tokens};

#[derive(Clone, Copy)]
struct Ctxt<'a> {
    ctx_var: &'a syn::Ident,
    encoder_var: &'a syn::Ident,
    trace: bool,
}

impl<'a> Ctxt<'a> {
    fn with_encoder(self, encoder_var: &'a syn::Ident) -> Self {
        Self {
            encoder_var,
            ..self
        }
    }
}

pub(crate) fn expand_encode_entry(b: &Build<'_>) -> Result<TokenStream> {
    b.validate_encode()?;
    b.cx.reset();

    let type_ident = &b.input.ident;

    let encoder_var = b.cx.ident("encoder");
    let ctx_var = b.cx.ident("ctx");
    let e_param = b.cx.type_with_span("E", Span::call_site());

    let cx = Ctxt {
        ctx_var: &ctx_var,
        encoder_var: &encoder_var,
        trace: true,
    };

    let Tokens {
        context_t,
        encode_t,
        encoder_t,
        option,
        result,
        try_fast_encode,
        ..
    } = b.tokens;

    let packed;

    let (body, size_hint) = match &b.data {
        BuildData::Struct(st) => {
            packed = crate::internals::packed(b, st);
            encode_map(cx, b, st)?
        }
        BuildData::Enum(en) => {
            packed = syn::parse_quote!(false);
            encode_enum(cx, b, en)?
        }
    };

    if b.cx.has_errors() {
        return Err(());
    }

    let mode_ident = b.expansion.mode_path(b.tokens);
    let mut impl_generics = b.input.generics.clone();

    if !b.bounds.is_empty() {
        let where_clause = impl_generics.make_where_clause();

        where_clause
            .predicates
            .extend(b.bounds.iter().flat_map(|(_, v)| v.as_predicate()).cloned());
    }

    let existing_bounds = build::existing_bounds(b.bounds, b.p.extra_idents());

    for t in b.input.generics.type_params() {
        if existing_bounds.contains(&t.ident) {
            continue;
        }

        let mut bounds = Punctuated::new();
        bounds.push(syn::parse_quote!(#encode_t<#mode_ident>));

        impl_generics
            .make_where_clause()
            .predicates
            .push(syn::WherePredicate::Type(syn::PredicateType {
                lifetimes: None,
                bounded_ty: syn::Type::Path(syn::TypePath {
                    qself: None,
                    path: syn::Path::from(syn::PathSegment::from(t.ident.clone())),
                }),
                colon_token: <Token![:]>::default(),
                bounds,
            }));
    }

    let (impl_generics, _, where_clause) = impl_generics.split_for_impl();
    let (_, type_generics, _) = b.input.generics.split_for_impl();

    let mut attributes = Vec::<syn::Attribute>::new();

    if cfg!(not(feature = "verbose")) {
        attributes.push(syn::parse_quote!(#[allow(clippy::just_underscores_and_digits)]));
    }

    Ok(quote! {
        const _: () = {
            #[automatically_derived]
            #(#attributes)*
            impl #impl_generics #encode_t<#mode_ident> for #type_ident #type_generics
            #where_clause
            {
                type Encode = Self;

                const IS_BITWISE_ENCODE: bool = #packed;

                #[inline]
                fn encode<#e_param>(&self, #encoder_var: #e_param) -> #result<(), <#e_param as #encoder_t>::Error>
                where
                    #e_param: #encoder_t<Mode = #mode_ident>,
                {
                    let #ctx_var = #encoder_t::cx(&#encoder_var);

                    let #encoder_var = match #encoder_t::try_fast_encode(#encoder_var, self)? {
                        #try_fast_encode::Ok => return #result::Ok(()),
                        #try_fast_encode::Unsupported(_, #encoder_var) => #encoder_var,
                        _ => return #result::Err(#context_t::message(#ctx_var, "Fast encoding failed")),
                    };

                    #body
                }

                #[inline]
                fn size_hint(&self) -> #option<usize> {
                    #size_hint
                }

                #[inline]
                fn as_encode(&self) -> &Self::Encode {
                    self
                }
            }
        };
    })
}

/// Encode a struct.
fn encode_map(cx: Ctxt<'_>, b: &Build<'_>, st: &Body<'_>) -> Result<(TokenStream, TokenStream)> {
    let Ctxt { ctx_var, .. } = cx;

    let Tokens {
        context_t, result, ..
    } = b.tokens;

    let output_var = b.cx.ident("output");

    let type_name = st.name.value;

    let enter = cx
        .trace
        .then(|| quote!(#context_t::enter_struct(#ctx_var, #type_name);));

    let leave = cx
        .trace
        .then(|| quote!(#context_t::leave_struct(#ctx_var);));

    let encode = encode_default(cx, b, st)?;
    let size_hint = default_size_hint(b, st)?;

    let encode = quote! {
        #enter
        let #output_var = #encode;
        #leave
        #result::Ok(#output_var)
    };

    Ok((encode, size_hint))
}

fn encode_default(cx: Ctxt<'_>, b: &Build<'_>, st: &Body<'_>) -> Result<TokenStream> {
    let encoder_var = cx.encoder_var;

    let Tokens {
        encoder_t, result, ..
    } = b.tokens;

    let encode;

    match st.packing {
        (_, Packing::Transparent) => {
            let Field {
                encode_path: (_, encode_path),
                access,
                ..
            } = st.transparent_field()?;

            encode = quote!(#encode_path(#access, #encoder_var)?);
        }
        (_, Packing::Packed) => {
            let decls = st.field_tests();
            let encoders = make_encoders(cx, b, st)?;

            encode = quote! {
                #encoder_t::encode_pack_fn(#encoder_var, move |#encoder_var| {
                    #(#decls)*
                    #(#encoders)*
                    #result::Ok(())
                })?
            };
        }
        (_, Packing::Tagged) => {
            let decls = st.field_tests();
            let encoders = make_encoders(cx, b, st)?;

            let len = length_test(st.unskipped_fields());
            let (build_hint, hint) = len.build_hint(b);

            encode = quote! {{
                #build_hint

                #encoder_t::encode_map_fn(#encoder_var, #hint, move |#encoder_var| {
                    #(#decls)*
                    #(#encoders)*
                    #result::Ok(())
                })?
            }};
        }
        (_, Packing::Untagged) => {
            return Err(());
        }
    }

    Ok(encode)
}

fn default_size_hint(b: &Build<'_>, st: &Body<'_>) -> Result<TokenStream> {
    let Tokens { option, .. } = b.tokens;

    let size_hint;

    match st.packing {
        (_, Packing::Transparent) => {
            let Field {
                size_hint_path,
                access,
                ..
            } = st.transparent_field()?;

            size_hint = match size_hint_path {
                Some((_, path)) => quote!(#path(#access)),
                None => quote!(#option::None),
            };
        }
        (_, Packing::Packed) => {
            size_hint = quote!(#option::None);
        }
        (_, Packing::Tagged) => {
            let len = length_test(st.unskipped_fields());
            let decls = st.field_tests();

            size_hint = quote! {{
                #(#decls)*
                #option::Some(#len)
            }};
        }
        (_, Packing::Untagged) => {
            return Err(());
        }
    }

    Ok(size_hint)
}

fn make_encoders(cx: Ctxt<'_>, b: &Build<'_>, st: &Body<'_>) -> Result<Vec<TokenStream>, ()> {
    let Ctxt {
        ctx_var,
        encoder_var,
        ..
    } = cx;

    let Tokens {
        context_t,
        entry_encoder_t,
        map_encoder_t,
        result,
        sequence_encoder_t,
        ..
    } = b.tokens;

    let encode_t_encode = &b.encode_t_encode;

    let sequence_decoder_var = b.cx.ident("sequence_decoder");
    let pair_encoder_var = b.cx.ident("pair_encoder");
    let field_encoder_var = b.cx.ident("field_encoder");
    let value_encoder_var = b.cx.ident("value_encoder");
    let field_name_static = b.cx.ident("FIELD_NAME");
    let field_name_expr = st.name.expr(field_name_static.clone());

    let mut encoders = Vec::with_capacity(st.all_fields.len());

    for f in st.unskipped_fields() {
        let Field {
            name,
            member,
            encode_path: (_, encode_path),
            access,
            skip_encoding_if,
            var,
            ..
        } = f;

        let enter = cx.trace.then(|| {
            let name_type = st.name.ty();

            match member {
                syn::Member::Named(ident) => {
                    let ident = syn::LitStr::new(&ident.to_string(), ident.span());
                    let formatted_name = st.name.name_format(&field_name_static);

                    quote! {
                        static #field_name_static: #name_type = #name;
                        #context_t::enter_named_field(#ctx_var, #ident, #formatted_name);
                    }
                }
                syn::Member::Unnamed(index) => {
                    let index = index.index;
                    let formatted_name = st.name.name_format(&field_name_static);

                    quote! {
                        static #field_name_static: #name_type = #name;
                        #context_t::enter_unnamed_field(#ctx_var, #index, #formatted_name);
                    }
                }
            }
        });

        let leave = cx.trace.then(|| quote!(#context_t::leave_field(#ctx_var);));

        let mut encode;

        match st.packing {
            (_, Packing::Transparent) => {
                return Err(());
            }
            (_, Packing::Packed) => {
                encode = quote! {{
                    #enter

                    let #sequence_decoder_var = #sequence_encoder_t::encode_next(#encoder_var)?;
                    #encode_path(#access, #sequence_decoder_var)?;

                    #leave
                }};
            }
            (_, Packing::Tagged) => {
                encode = quote! {{
                    #enter

                    #map_encoder_t::encode_entry_fn(#encoder_var, move |#pair_encoder_var| {
                        let #field_encoder_var = #entry_encoder_t::encode_key(#pair_encoder_var)?;
                        #encode_t_encode(#field_name_expr, #field_encoder_var)?;
                        let #value_encoder_var = #entry_encoder_t::encode_value(#pair_encoder_var)?;
                        #encode_path(#access, #value_encoder_var)?;
                        #result::Ok(())
                    })?;

                    #leave
                }};
            }
            (_, Packing::Untagged) => {
                return Err(());
            }
        };

        if skip_encoding_if.is_some() {
            encode = quote! {
                if #var {
                    #encode
                }
            };
        }

        encoders.push(encode);
    }

    Ok(encoders)
}

/// Encode an internally tagged enum.
fn encode_enum(cx: Ctxt<'_>, b: &Build<'_>, en: &Enum<'_>) -> Result<(TokenStream, TokenStream)> {
    let mut encode_variants = Vec::with_capacity(en.variants.len());
    let mut size_hint_variants = Vec::with_capacity(en.variants.len());

    for v in &en.variants {
        let Ok((pattern, encode, size_hint)) = encode_variant(cx, b, en, v) else {
            continue;
        };

        encode_variants.push(quote!(#pattern => #encode));
        size_hint_variants.push(quote!(#pattern => #size_hint));
    }

    let encode;
    let size_hint;

    if encode_variants.is_empty() {
        encode = quote!(match *self {});
        size_hint = quote!(match *self {});
    } else {
        encode = quote! {
            match *self { #(#encode_variants),* }
        };

        size_hint = quote! {
            match *self { #(#size_hint_variants),* }
        };
    }

    Ok((encode, size_hint))
}

/// Generate encoding for a single variant.
fn encode_variant(
    cx: Ctxt<'_>,
    b: &Build<'_>,
    en: &Enum<'_>,
    v: &Variant<'_>,
) -> Result<(syn::PatStruct, TokenStream, TokenStream)> {
    let Ctxt { ctx_var, .. } = cx;

    let Tokens {
        context_t, result, ..
    } = b.tokens;

    let output_var = b.cx.ident("output");

    let encode;
    let size_hint;

    match &en.enum_tagging {
        EnumTagging::Empty => {
            (encode, size_hint) = encode_empty_variant(cx, b, en, v)?;
        }
        EnumTagging::Default => match en.packing {
            (_, Packing::Tagged) => {
                (encode, size_hint) = encode_tagged_variant(cx, b, en, v)?;
            }
            (_, Packing::Untagged) => {
                encode = encode_default(cx, b, &v.st)?;
                size_hint = default_size_hint(b, &v.st)?;
            }
            _ => {
                return Err(());
            }
        },
        EnumTagging::Internal { tag } => {
            (encode, size_hint) = encode_internal_variant(cx, b, en, v, tag)?;
        }
        EnumTagging::Adjacent { tag, content } => {
            (encode, size_hint) = encode_adjacent_variant(cx, b, en, v, tag, content)?;
        }
    }

    let pattern = syn::PatStruct {
        attrs: Vec::new(),
        qself: None,
        path: v.st.path.clone(),
        brace_token: syn::token::Brace::default(),
        fields: v.patterns.clone(),
        rest: None,
    };

    let enter = cx.trace.then(|| {
        let name_static = b.cx.ident("NAME");

        let formatted_tag = en.name.name_format(&name_static);
        let name_type = en.name.ty();
        let name_value = &v.name;
        let type_name = v.st.name.value;

        quote! {
            static #name_static: #name_type = #name_value;
            #context_t::enter_variant(#ctx_var, #type_name, #formatted_tag);
        }
    });

    let leave = cx.trace.then(|| {
        quote! {
            #context_t::leave_variant(#ctx_var);
        }
    });

    let encode = quote! {{
        #enter
        let #output_var = #encode;
        #leave
        #result::Ok(#output_var)
    }};

    Ok((pattern, encode, size_hint))
}

fn encode_empty_variant(
    cx: Ctxt<'_>,
    b: &Build<'_>,
    en: &Enum<'_>,
    v: &Variant<'_>,
) -> Result<(TokenStream, TokenStream)> {
    let encoder_var = cx.encoder_var;

    let Tokens { option, .. } = b.tokens;

    let name_type = en.name.ty();
    let encode_t_encode = &b.encode_t_encode;
    let name = &v.name;

    let name_static = b.cx.ident("NAME");

    let name_expr = en.name.expr(name_static.clone());

    let encode = quote! {{
        static #name_static: #name_type = #name;
        #encode_t_encode(#name_expr, #encoder_var)?
    }};

    let size_hint = quote!(#option::Some(1));

    Ok((encode, size_hint))
}

fn encode_tagged_variant(
    cx: Ctxt<'_>,
    b: &Build<'_>,
    en: &Enum<'_>,
    v: &Variant<'_>,
) -> Result<(TokenStream, TokenStream)> {
    let Tokens {
        encoder_t,
        option,
        result,
        variant_encoder_t,
        ..
    } = b.tokens;

    let encoder_var = cx.encoder_var;

    let tag_encoder = b.cx.ident("tag_encoder");
    let variant_encoder = b.cx.ident("variant_encoder");
    let name_static = b.cx.ident("NAME");
    let data_encoder = b.cx.ident("data_encoder");
    let name_expr = en.name.expr(name_static.clone());

    let encode = encode_default(cx.with_encoder(&data_encoder), b, &v.st)?;
    let size_hint = quote!(#option::Some(1));

    let encode_t_encode = &b.encode_t_encode;
    let name = &v.name;
    let name_type = en.name.ty();

    let encode = quote! {{
        #encoder_t::encode_variant_fn(#encoder_var, move |#variant_encoder| {
            let #tag_encoder = #variant_encoder_t::encode_tag(#variant_encoder)?;
            static #name_static: #name_type = #name;

            #encode_t_encode(#name_expr, #tag_encoder)?;

            let #data_encoder = #variant_encoder_t::encode_data(#variant_encoder)?;
            #encode;
            #result::Ok(())
        })?
    }};

    Ok((encode, size_hint))
}

fn encode_internal_variant(
    cx: Ctxt<'_>,
    b: &Build<'_>,
    en: &Enum<'_>,
    v: &Variant<'_>,
    tag: &Name<'_, syn::Expr>,
) -> Result<(TokenStream, TokenStream)> {
    let Ctxt {
        ctx_var,
        encoder_var,
        ..
    } = cx;

    let Tokens {
        context_t,
        encoder_t,
        map_encoder_t,
        option,
        result,
        ..
    } = b.tokens;

    let tag_static = b.cx.ident("TAG");
    let name_static = b.cx.ident("NAME");

    let encode;
    let size_hint;

    'done: {
        let inner_encode;
        let mut len;

        match v.st.packing {
            (_, Packing::Transparent) => {
                let Field {
                    access,
                    size_hint_path,
                    encode_path: (_, encode_path),
                    ..
                } = v.st.transparent_field()?;

                let Some((_, path)) = size_hint_path else {
                    encode = quote! {
                        return #result::Err(#context_t::message(
                            #ctx_var,
                            "Cannot encode transparent field with custom encoding",
                        ));
                    };

                    size_hint = quote!(#option::None);
                    break 'done;
                };

                len = LengthTest::default();
                len.expressions.push(quote!(1));
                len.expressions.push(quote!(#path(#access)?));

                inner_encode = quote! {
                    let #encoder_var = #map_encoder_t::as_encoder(#encoder_var);
                    #encode_path(#access, #encoder_var)?;
                };

                size_hint = quote!(#option::Some(#len));
            }
            (_, Packing::Packed) => {
                return Err(());
            }
            (_, Packing::Tagged) => {
                len = length_test(v.st.unskipped_fields());
                len.expressions.push(quote!(1));

                let decls = v.st.field_tests();
                let encoders = make_encoders(cx, b, &v.st)?;

                inner_encode = quote! {
                    #(#decls)*
                    #(#encoders)*
                };

                let decls = v.st.field_tests();

                size_hint = quote! {{
                    #(#decls)*
                    #option::Some(#len)
                }};
            }
            (_, Packing::Untagged) => {
                return Err(());
            }
        }

        let (build_hint, hint) = len.build_hint(b);

        let tag_value = tag.value;
        let tag_type = tag.ty();

        let name = &v.name;
        let name_type = en.name.ty();

        encode = quote! {{
            #build_hint

            #encoder_t::encode_map_fn(#encoder_var, #hint, move |#encoder_var| {
                static #tag_static: #tag_type = #tag_value;
                static #name_static: #name_type = #name;
                #map_encoder_t::insert_entry(#encoder_var, #tag_static, #name_static)?;
                #inner_encode
                #result::Ok(())
            })?
        }};
    };

    Ok((encode, size_hint))
}

fn encode_adjacent_variant(
    cx: Ctxt<'_>,
    b: &Build<'_>,
    en: &Enum<'_>,
    v: &Variant<'_>,
    tag: &Name<'_, syn::Expr>,
    content: &Name<'_, syn::Expr>,
) -> Result<(TokenStream, TokenStream)> {
    let encoder_var = cx.encoder_var;

    let Tokens {
        encoder_t,
        entry_encoder_t,
        map_encoder_t,
        option,
        result,
        ..
    } = b.tokens;

    let encode_t_encode = &b.encode_t_encode;

    let name = &v.name;
    let name_type = en.name.ty();

    let adjacent_encoder_var = b.cx.ident("adjacent_encoder");
    let content_static = b.cx.ident("CONTENT");
    let content_tag = b.cx.ident("content_tag");
    let hint_static = b.cx.ident("HINT");
    let name_static = b.cx.ident("NAME");
    let pair_encoder_var = b.cx.ident("pair_encoder");
    let tag_static = b.cx.ident("TAG");

    let tag_value = tag.value;
    let tag_type = tag.ty();
    let content_value = content.value;
    let content_static_expr = content.expr(content_static.clone());
    let content_type = content.ty();

    let inner_encode = encode_default(cx, b, &v.st)?;

    let encode = quote! {{
        static #hint_static: usize = 2;

        #encoder_t::encode_map_fn(#encoder_var, #hint_static, move |#adjacent_encoder_var| {
            static #tag_static: #tag_type = #tag_value;
            static #content_static: #content_type = #content_value;
            static #name_static: #name_type = #name;

            #map_encoder_t::insert_entry(#adjacent_encoder_var, #tag_static, #name_static)?;

            #map_encoder_t::encode_entry_fn(#adjacent_encoder_var, move |#pair_encoder_var| {
                let #content_tag = #entry_encoder_t::encode_key(#pair_encoder_var)?;
                #encode_t_encode(#content_static_expr, #content_tag)?;

                let #encoder_var = #entry_encoder_t::encode_value(#pair_encoder_var)?;
                #inner_encode;
                #result::Ok(())
            })?;

            #result::Ok(())
        })?
    }};

    let size_hint = quote!(#option::Some(2));
    Ok((encode, size_hint))
}

#[derive(Default)]
struct LengthTest {
    kind: LengthTestKind,
    expressions: Punctuated<TokenStream, Token![+]>,
}

impl ToTokens for LengthTest {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.expressions.to_tokens(tokens);
    }
}

impl LengthTest {
    fn build_hint(&self, b: &Build<'_>) -> (TokenStream, syn::Ident) {
        let Tokens { map_hint, .. } = b.tokens;

        let len = &self.expressions;

        match self.kind {
            LengthTestKind::Static => {
                let hint = b.cx.ident("HINT");
                let item = quote! {
                    static #hint: usize = #len;
                };

                (item, hint)
            }
            LengthTestKind::Dynamic => {
                let mode = &b.mode.mode_path;
                let hint = b.cx.ident("hint");
                let item = quote! {
                    let #hint = #map_hint::<#mode>(self);
                };

                (item, hint)
            }
        }
    }
}

#[derive(Default)]
enum LengthTestKind {
    Static,
    #[default]
    Dynamic,
}

fn length_test<'a>(fields: impl IntoIterator<Item = &'a Field<'a>>) -> LengthTest {
    let mut kind = LengthTestKind::Static;

    let mut expressions = Punctuated::<_, Token![+]>::new();
    let mut count = 0usize;

    for Field {
        skip_encoding_if,
        var,
        ..
    } in fields
    {
        if skip_encoding_if.is_some() {
            kind = LengthTestKind::Dynamic;
            expressions.push(quote!(if #var { 1 } else { 0 }))
        } else {
            count += 1;
        }
    }

    expressions.push(quote!(#count));
    LengthTest { kind, expressions }
}
