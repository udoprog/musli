use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::Token;

use crate::internals::attr::{EnumTagging, Packing};
use crate::internals::build::{Body, Build, BuildData, Enum, Variant};
use crate::internals::tokens::Tokens;
use crate::internals::Result;

struct Ctxt<'a> {
    ctx_var: &'a syn::Ident,
    encoder_var: &'a syn::Ident,
    trace: bool,
}

pub(crate) fn expand_insert_entry(e: Build<'_>) -> Result<TokenStream> {
    e.validate_encode()?;
    e.cx.reset();

    let type_ident = &e.input.ident;

    let encoder_var = e.cx.ident("encoder");
    let ctx_var = e.cx.ident("ctx");
    let e_param = e.cx.type_with_span("E", Span::call_site());

    let cx = Ctxt {
        ctx_var: &ctx_var,
        encoder_var: &encoder_var,
        trace: true,
    };

    let Tokens {
        encode_t,
        encoder_t,
        result,
        try_fast_encode,
        context_t,
        ..
    } = e.tokens;

    let packed;

    let body = match &e.data {
        BuildData::Struct(st) => {
            packed = crate::packed::packed(&e, st, e.tokens.encode_t, "ENCODE_PACKED");
            encode_map(&cx, &e, st)?
        }
        BuildData::Enum(en) => {
            packed = syn::parse_quote!(false);
            encode_enum(&cx, &e, en)?
        }
    };

    if e.cx.has_errors() {
        return Err(());
    }

    let mut impl_generics = e.input.generics.clone();

    if !e.bounds.is_empty() {
        let where_clause = impl_generics.make_where_clause();

        where_clause
            .predicates
            .extend(e.bounds.iter().map(|(_, v)| v.clone()));
    }

    let (impl_generics, _, where_clause) = impl_generics.split_for_impl();
    let (_, type_generics, _) = e.input.generics.split_for_impl();

    let mut attributes = Vec::<syn::Attribute>::new();

    if cfg!(not(feature = "verbose")) {
        attributes.push(syn::parse_quote!(#[allow(clippy::just_underscores_and_digits)]));
    }

    let mode_ident = e.expansion.mode_path(&e.tokens);

    Ok(quote! {
        const _: () = {
            #[automatically_derived]
            #(#attributes)*
            impl #impl_generics #encode_t<#mode_ident> for #type_ident #type_generics
            #where_clause
            {
                const ENCODE_PACKED: bool = #packed;

                type Encode = Self;

                #[inline]
                fn encode<#e_param>(&self, #encoder_var: #e_param) -> #result<<#e_param as #encoder_t>::Ok, <#e_param as #encoder_t>::Error>
                where
                    #e_param: #encoder_t<Mode = #mode_ident>,
                {
                    #encoder_t::cx(#encoder_var, |#ctx_var, #encoder_var| {
                        let #encoder_var = match #encoder_t::try_fast_encode(#encoder_var, self)? {
                            #try_fast_encode::Ok(value) => return #result::Ok(value),
                            #try_fast_encode::Unsupported(_, #encoder_var) => #encoder_var,
                            _ => return #result::Err(#context_t::message(#ctx_var, "fast encoding failed")),
                        };

                        #body
                    })
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
fn encode_map(cx: &Ctxt<'_>, b: &Build<'_>, st: &Body<'_>) -> Result<TokenStream> {
    let Ctxt {
        ctx_var,
        encoder_var,
        ..
    } = *cx;

    let Tokens {
        context_t,
        encoder_t,
        result,
        ..
    } = b.tokens;

    let pack_var = b.cx.ident("pack");
    let output_var = b.cx.ident("output");

    let (encoders, tests) = insert_fields(cx, b, st, &pack_var)?;

    let type_name = &st.name;

    let enter = cx
        .trace
        .then(|| quote!(#context_t::enter_struct(#ctx_var, #type_name);));
    let leave = cx
        .trace
        .then(|| quote!(#context_t::leave_struct(#ctx_var);));

    let encode;

    match st.packing {
        Packing::Transparent => {
            let f = &st.unskipped_fields[0];

            let access = &f.self_access;
            let encode_path = &f.encode_path.1;

            encode = quote! {{
                #enter
                let #output_var = #encode_path(#access, #encoder_var)?;
                #leave
                #output_var
            }};
        }
        Packing::Tagged => {
            let decls = tests.iter().map(|t| &t.decl);
            let (build_hint, hint) = length_test(st.unskipped_fields.len(), &tests).build(b);

            encode = quote! {{
                #enter
                #(#decls)*
                #build_hint

                let #output_var = #encoder_t::encode_map_fn(#encoder_var, &#hint, move |#encoder_var| {
                    #(#encoders)*
                    #result::Ok(())
                })?;
                #leave
                #output_var
            }};
        }
        Packing::Packed(..) => {
            let decls = tests.iter().map(|t| &t.decl);

            encode = quote! {{
                #enter
                let #output_var = #encoder_t::encode_pack_fn(#encoder_var, move |#pack_var| {
                    #(#decls)*
                    #(#encoders)*
                    #result::Ok(())
                })?;
                #leave
                #output_var
            }};
        }
    }

    Ok(quote!(#result::Ok(#encode)))
}

struct FieldTest<'st> {
    decl: syn::Stmt,
    var: &'st syn::Ident,
}

fn insert_fields<'st>(
    cx: &Ctxt<'_>,
    b: &Build<'_>,
    st: &'st Body<'_>,
    pack_var: &syn::Ident,
) -> Result<(Vec<TokenStream>, Vec<FieldTest<'st>>)> {
    let Ctxt {
        ctx_var,
        encoder_var,
        ..
    } = *cx;

    let Tokens {
        context_t,
        sequence_encoder_t,
        result,

        map_encoder_t,
        map_entry_encoder_t,
        ..
    } = b.tokens;

    let encode_t_encode = &b.encode_t_encode;

    let sequence_decoder_next_var = b.cx.ident("sequence_decoder_next");
    let pair_encoder_var = b.cx.ident("pair_encoder");
    let field_encoder_var = b.cx.ident("field_encoder");
    let value_encoder_var = b.cx.ident("value_encoder");
    let field_name_static = b.cx.ident("FIELD_NAME");
    let field_name_expr = st.name_type.expr(field_name_static.clone());

    let mut encoders = Vec::with_capacity(st.all_fields.len());
    let mut tests = Vec::with_capacity(st.all_fields.len());

    for f in &st.unskipped_fields {
        let encode_path = &f.encode_path.1;
        let access = &f.self_access;
        let name = &f.name;
        let name_type = st.name_type.ty();

        let mut encode;

        let enter = match &f.member {
            syn::Member::Named(ident) => {
                let field_name = syn::LitStr::new(&ident.to_string(), ident.span());

                cx.trace.then(|| {
                    let formatted_name = st.name_type.name_format(&field_name_static);

                    quote! {
                        #context_t::enter_named_field(#ctx_var, #field_name, #formatted_name);
                    }
                })
            }
            syn::Member::Unnamed(index) => {
                let index = index.index;
                cx.trace.then(|| {
                    let formatted_name = st.name_type.name_format(&field_name_static);

                    quote! {
                        #context_t::enter_unnamed_field(#ctx_var, #index, #formatted_name);
                    }
                })
            }
        };

        let leave = cx.trace.then(|| quote!(#context_t::leave_field(#ctx_var);));

        match f.packing {
            Packing::Tagged | Packing::Transparent => {
                encode = quote! {{
                    static #field_name_static: #name_type = #name;

                    #enter

                    #map_encoder_t::encode_entry_fn(#encoder_var, move |#pair_encoder_var| {
                        let #field_encoder_var = #map_entry_encoder_t::encode_key(#pair_encoder_var)?;
                        #encode_t_encode(#field_name_expr, #field_encoder_var)?;
                        let #value_encoder_var = #map_entry_encoder_t::encode_value(#pair_encoder_var)?;
                        #encode_path(#access, #value_encoder_var)?;
                        #result::Ok(())
                    })?;

                    #leave
                }};
            }
            Packing::Packed(..) => {
                let decl = enter.is_some().then(|| {
                    quote! {
                        static #field_name_static: #name_type = #name;
                    }
                });

                encode = quote! {{
                    #decl
                    #enter
                    let #sequence_decoder_next_var = #sequence_encoder_t::encode_next(#pack_var)?;
                    #encode_path(#access, #sequence_decoder_next_var)?;
                    #leave
                }};
            }
        };

        if let Some((_, skip_encoding_if_path)) = f.skip_encoding_if.as_ref() {
            let var = &f.var;

            let decl = syn::parse_quote! {
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
fn encode_enum(cx: &Ctxt<'_>, b: &Build<'_>, en: &Enum<'_>) -> Result<TokenStream> {
    let Ctxt { ctx_var, .. } = *cx;

    let Tokens {
        context_t, result, ..
    } = b.tokens;

    let type_name = en.name;

    if let Some(&(span, Packing::Transparent)) = en.packing_span {
        b.encode_transparent_enum_diagnostics(span);
        return Err(());
    }

    let mut variants = Vec::with_capacity(en.variants.len());

    for v in &en.variants {
        let Ok((pattern, encode)) = encode_variant(cx, b, en, v) else {
            continue;
        };

        variants.push(quote!(#pattern => #encode));
    }

    // Special case: uninhabitable types.
    Ok(if variants.is_empty() {
        quote!(#result::Err(#context_t::uninhabitable(#ctx_var, #type_name)))
    } else {
        quote!(#result::Ok(match self { #(#variants),* }))
    })
}

/// Setup encoding for a single variant. that is externally tagged.
fn encode_variant(
    cx: &Ctxt<'_>,
    b: &Build<'_>,
    en: &Enum<'_>,
    v: &Variant<'_>,
) -> Result<(syn::PatStruct, TokenStream)> {
    let pack_var = b.cx.ident("pack");

    let (encoders, tests) = insert_fields(cx, b, &v.st, &pack_var)?;

    let Ctxt {
        ctx_var,
        encoder_var,
        ..
    } = *cx;

    let Tokens {
        context_t,
        encoder_t,
        result,
        map_encoder_t,
        map_entry_encoder_t,
        variant_encoder_t,
        map_hint,
        ..
    } = b.tokens;

    let content_static = b.cx.ident("CONTENT");
    let hint = b.cx.ident("STRUCT_HINT");
    let name_static = b.cx.ident("NAME");
    let name_expr = en.name_type.expr(name_static.clone());
    let tag_encoder = b.cx.ident("tag_encoder");
    let tag_static = b.cx.ident("TAG");
    let variant_encoder = b.cx.ident("variant_encoder");

    let type_name = v.st.name;

    let mut encode;

    match &en.enum_tagging {
        EnumTagging::Empty => {
            let name_type = en.name_type.ty();
            let encode_t_encode = &b.encode_t_encode;
            let name = &v.name;

            encode = quote! {{
                static #name_static: #name_type = #name;
                #encode_t_encode(#name_expr, #encoder_var)?
            }};
        }
        EnumTagging::Default => {
            match v.st.packing {
                Packing::Transparent => {
                    let f = &v.st.unskipped_fields[0];

                    let encode_path = &f.encode_path.1;
                    let var = &f.self_access;
                    encode = quote!(#encode_path(#var, #encoder_var)?);
                }
                Packing::Packed(..) => {
                    let decls = tests.iter().map(|t| &t.decl);

                    encode = quote! {{
                        #encoder_t::encode_pack_fn(#encoder_var, move |#pack_var| {
                            #(#decls)*
                            #(#encoders)*
                            #result::Ok(())
                        })?
                    }};
                }
                Packing::Tagged => {
                    let decls = tests.iter().map(|t| &t.decl);
                    let (build_hint, hint) =
                        length_test(v.st.unskipped_fields.len(), &tests).build(b);

                    encode = quote! {{
                        #build_hint

                        #encoder_t::encode_map_fn(#encoder_var, &#hint, move |#encoder_var| {
                            #(#decls)*
                            #(#encoders)*
                            #result::Ok(())
                        })?
                    }};
                }
            }

            if let Packing::Tagged = en.enum_packing {
                let encode_t_encode = &b.encode_t_encode;
                let name = &v.name;
                let name_type = en.name_type.ty();

                encode = quote! {{
                    #encoder_t::encode_variant_fn(#encoder_var, move |#variant_encoder| {
                        let #tag_encoder = #variant_encoder_t::encode_tag(#variant_encoder)?;
                        static #name_static: #name_type = #name;

                        #encode_t_encode(#name_expr, #tag_encoder)?;

                        let #encoder_var = #variant_encoder_t::encode_data(#variant_encoder)?;
                        #encode;
                        #result::Ok(())
                    })?
                }};
            }
        }
        EnumTagging::Internal {
            tag_value,
            tag_type,
        } => {
            let name = &v.name;

            let name_type = en.name_type.ty();

            let decls = tests.iter().map(|t| &t.decl);
            let mut len = length_test(v.st.unskipped_fields.len(), &tests);

            // Add one for the tag field.
            len.expressions.push(quote!(1));

            let (build_hint, hint) = len.build(b);

            let tag_type = tag_type.ty();

            encode = quote! {{
                #build_hint

                #encoder_t::encode_map_fn(#encoder_var, &#hint, move |#encoder_var| {
                    static #tag_static: #tag_type = #tag_value;
                    static #name_static: #name_type = #name;
                    #map_encoder_t::insert_entry(#encoder_var, #tag_static, #name_static)?;
                    #(#decls)*
                    #(#encoders)*
                    #result::Ok(())
                })?
            }};
        }
        EnumTagging::Adjacent {
            tag_value,
            tag_type,
            content_value,
            content_type,
        } => {
            let encode_t_encode = &b.encode_t_encode;

            let name = &v.name;
            let name_type = en.name_type.ty();

            let decls = tests.iter().map(|t| &t.decl);

            let (build_hint, inner_hint) =
                length_test(v.st.unskipped_fields.len(), &tests).build(b);
            let struct_encoder = b.cx.ident("struct_encoder");
            let content_struct = b.cx.ident("content_struct");
            let pair = b.cx.ident("pair");
            let content_tag = b.cx.ident("content_tag");

            let tag_type = tag_type.ty();
            let content_static_expr = content_type.expr(content_static.clone());
            let content_type = content_type.ty();

            encode = quote! {{
                static #hint: #map_hint = #map_hint::with_size(2);
                #build_hint

                #encoder_t::encode_map_fn(#encoder_var, &#hint, move |#struct_encoder| {
                    static #tag_static: #tag_type = #tag_value;
                    static #content_static: #content_type = #content_value;
                    static #name_static: #name_type = #name;

                    #map_encoder_t::insert_entry(#struct_encoder, #tag_static, #name_static)?;

                    #map_encoder_t::encode_entry_fn(#struct_encoder, move |#pair| {
                        let #content_tag = #map_entry_encoder_t::encode_key(#pair)?;
                        #encode_t_encode(#content_static_expr, #content_tag)?;

                        let #content_struct = #map_entry_encoder_t::encode_value(#pair)?;

                        #encoder_t::encode_map_fn(#content_struct, &#inner_hint, move |#encoder_var| {
                            #(#decls)*
                            #(#encoders)*
                            #result::Ok(())
                        })?;

                        #result::Ok(())
                    })?;

                    #result::Ok(())
                })?
            }};
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

    if cx.trace {
        let output_var = b.cx.ident("output");

        let formatted_tag = en.name_type.name_format(&name_static);
        let name_type = en.name_type.ty();
        let name_value = &v.name;

        encode = quote! {{
            static #name_static: #name_type = #name_value;
            #context_t::enter_variant(#ctx_var, #type_name, #formatted_tag);
            let #output_var = #encode;
            #context_t::leave_variant(#ctx_var);
            #output_var
        }};
    }

    Ok((pattern, encode))
}

struct LengthTest {
    kind: LengthTestKind,
    expressions: Punctuated<TokenStream, Token![+]>,
}

impl LengthTest {
    fn build(&self, b: &Build<'_>) -> (syn::Stmt, syn::Ident) {
        let Tokens { map_hint, .. } = b.tokens;

        match self.kind {
            LengthTestKind::Static => {
                let hint = b.cx.ident("HINT");
                let len = &self.expressions;
                let item = syn::parse_quote!(static #hint: #map_hint = #map_hint::with_size(#len););
                (item, hint)
            }
            LengthTestKind::Dynamic => {
                let hint = b.cx.ident("hint");
                let len = &self.expressions;
                let item = syn::parse_quote!(let #hint: #map_hint = #map_hint::with_size(#len););
                (item, hint)
            }
        }
    }
}

enum LengthTestKind {
    Static,
    Dynamic,
}

fn length_test(count: usize, tests: &[FieldTest<'_>]) -> LengthTest {
    let mut kind = LengthTestKind::Static;

    let mut expressions = Punctuated::<_, Token![+]>::new();
    let count = count.saturating_sub(tests.len());
    expressions.push(quote!(#count));

    for FieldTest { var, .. } in tests {
        kind = LengthTestKind::Dynamic;
        expressions.push(quote!(if #var { 1 } else { 0 }))
    }

    LengthTest { kind, expressions }
}
