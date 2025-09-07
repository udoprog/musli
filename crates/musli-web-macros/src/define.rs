use std::cell::RefCell;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    Generics, Ident, Lifetime, LitStr, Path, Token, Type,
    parse::{Parse, ParseStream, Parser},
};

pub(super) fn cx(base: &Path) -> Context<'_> {
    let errors = Vec::new();

    Context {
        t: Tokens {
            endpoint: TraitPath {
                base,
                segments: vec![
                    Ident::new("api", Span::call_site()),
                    Ident::new("Endpoint", Span::call_site()),
                ],
            },
            broadcast: TraitPath {
                base,
                segments: vec![
                    Ident::new("api", Span::call_site()),
                    Ident::new("Broadcast", Span::call_site()),
                ],
            },
            request: TraitPath {
                base,
                segments: vec![
                    Ident::new("api", Span::call_site()),
                    Ident::new("Request", Span::call_site()),
                ],
            },
            event: TraitPath {
                base,
                segments: vec![
                    Ident::new("api", Span::call_site()),
                    Ident::new("Event", Span::call_site()),
                ],
            },
            brace: syn::token::Brace::default(),
            const_: <Token![const]>::default(),
            fn_: <Token![fn]>::default(),
            for_: <Token![for]>::default(),
            impl_: <Token![impl]>::default(),
            paren: syn::token::Paren::default(),
            type_: <Token![type]>::default(),
        },
        errors: RefCell::new(errors),
    }
}

struct AssocType {
    attrs: Vec<syn::Attribute>,
    generics: Generics,
    eq: Token![=],
    ty: Type,
    semi: Token![;],
}

impl Parse for AssocType {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(AssocType {
            attrs: input.call(syn::Attribute::parse_outer)?,
            generics: input.parse::<Generics>()?,
            eq: input.parse::<Token![=]>()?,
            ty: input.parse::<Type>()?,
            semi: input.parse::<Token![;]>()?,
        })
    }
}

#[derive(Default)]
struct ParsedAttrs {
    kind: Option<LitStr>,
}

struct Endpoint {
    parsed_attrs: ParsedAttrs,
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    name: Ident,
    requests: Vec<AssocType>,
    res: AssocType,
}

impl Endpoint {
    fn implement(self, cx: &Context, t: &mut TokenStream) {
        let kind = match self.parsed_attrs.kind {
            Some(kind) => kind,
            None => LitStr::new(&self.name.to_string(), self.name.span()),
        };

        {
            for attr in &self.attrs {
                attr.to_tokens(t);
            }

            self.vis.to_tokens(t);
            <Token![enum]>::default().to_tokens(t);
            self.name.to_tokens(t);
            cx.t.brace.surround(t, |_| {});

            cx.t.impl_.to_tokens(t);
            self.name.to_tokens(t);
            cx.t.brace.surround(t, |t| {
                self.vis.to_tokens(t);
                cx.define_const("KIND", &kind, t);
            });
        }

        {
            cx.t.impl_.to_tokens(t);
            cx.t.endpoint.to_tokens(t);
            cx.t.for_.to_tokens(t);
            self.name.to_tokens(t);

            cx.t.brace.surround(t, |t| {
                cx.define_const("KIND", &kind, t);

                for attr in &self.res.attrs {
                    attr.to_tokens(t);
                }

                cx.t.type_.to_tokens(t);
                syn::Ident::new("Response", Span::call_site()).to_tokens(t);
                self.res.generics.to_tokens(t);
                self.res.eq.to_tokens(t);
                self.res.ty.to_tokens(t);
                self.res.semi.to_tokens(t);

                cx.do_not_implement("__do_not_implement_endpoint", t);
            });
        }

        for req in &self.requests {
            for attr in &req.attrs {
                attr.to_tokens(t);
            }

            cx.t.impl_.to_tokens(t);
            req.generics.to_tokens(t);
            cx.t.request.to_tokens(t);
            cx.t.for_.to_tokens(t);
            req.ty.to_tokens(t);

            cx.t.brace.surround(t, |t| {
                cx.t.type_.to_tokens(t);
                syn::Ident::new("Endpoint", Span::call_site()).to_tokens(t);
                req.eq.to_tokens(t);
                self.name.to_tokens(t);
                req.semi.to_tokens(t);

                cx.do_not_implement("__do_not_implement_request", t);
            });
        }
    }
}

struct Broadcast {
    parsed_attrs: ParsedAttrs,
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    name: Ident,
    first: AssocType,
    events: Vec<AssocType>,
}

impl Broadcast {
    fn implement(self, cx: &Context, t: &mut TokenStream) {
        let kind = match self.parsed_attrs.kind {
            Some(kind) => kind,
            None => LitStr::new(&self.name.to_string(), self.name.span()),
        };

        {
            for attr in &self.attrs {
                attr.to_tokens(t);
            }

            self.vis.to_tokens(t);
            <Token![enum]>::default().to_tokens(t);
            self.name.to_tokens(t);
            cx.t.brace.surround(t, |_| {});

            cx.t.impl_.to_tokens(t);
            self.name.to_tokens(t);
            cx.t.brace.surround(t, |t| {
                self.vis.to_tokens(t);
                cx.define_const("KIND", &kind, t);
            });
        }

        {
            cx.t.impl_.to_tokens(t);
            cx.t.broadcast.to_tokens(t);
            cx.t.for_.to_tokens(t);
            self.name.to_tokens(t);

            cx.t.brace.surround(t, |t| {
                cx.define_const("KIND", &kind, t);

                cx.t.type_.to_tokens(t);
                syn::Ident::new("Event", Span::call_site()).to_tokens(t);
                self.first.generics.to_tokens(t);
                <Token![=]>::default().to_tokens(t);
                self.first.ty.to_tokens(t);
                <Token![;]>::default().to_tokens(t);

                cx.do_not_implement("__do_not_implement_broadcast", t);
            });
        }

        for ev in [&self.first].into_iter().chain(&self.events) {
            for attr in &ev.attrs {
                attr.to_tokens(t);
            }

            cx.t.impl_.to_tokens(t);
            ev.generics.to_tokens(t);
            cx.t.event.to_tokens(t);

            cx.t.for_.to_tokens(t);
            ev.ty.to_tokens(t);

            cx.t.brace.surround(t, |t| {
                cx.t.type_.to_tokens(t);
                syn::Ident::new("Broadcast", Span::call_site()).to_tokens(t);
                <Token![=]>::default().to_tokens(t);
                self.name.to_tokens(t);
                <Token![;]>::default().to_tokens(t);

                cx.do_not_implement("__do_not_implement_event", t);
            });
        }
    }
}

pub(super) fn expand(cx: &Context, input: TokenStream) -> TokenStream {
    let mut endpoints = Vec::new();
    let mut broadcasts = Vec::new();

    let parser = |input: ParseStream| {
        while !input.is_empty() {
            let mut attrs = Vec::new();
            let mut parsed_attrs = ParsedAttrs::default();

            for attr in input.call(syn::Attribute::parse_outer)? {
                if !attr.path().is_ident("musli") {
                    attrs.push(attr);
                    continue;
                }

                let result = attr.parse_args_with(|input: ParseStream| {
                    let kind = input.parse::<Ident>()?;

                    if kind == "kind" {
                        input.parse::<Token![=]>()?;
                        parsed_attrs.kind = Some(input.parse::<LitStr>()?);
                        return Ok(());
                    }

                    Err(syn::Error::new(kind.span(), "Expected `kind` as attribute"))
                });

                if let Err(error) = result {
                    cx.errors.borrow_mut().push(error);
                }
            }

            let vis = input.parse::<syn::Visibility>()?;
            let what = input.parse::<Ident>()?;

            if what == "endpoint" {
                let name = input.parse::<Ident>()?;

                let content;
                syn::braced!(content in input);

                let mut requests = Vec::new();
                let mut response = None;

                while !content.is_empty() {
                    let ty = content.parse::<Ident>()?;

                    if ty == "request" {
                        requests.push(content.parse()?);
                        continue;
                    }

                    if ty == "response" {
                        response = Some(content.parse()?);
                        continue;
                    }

                    return Err(syn::Error::new(
                        ty.span(),
                        "Expected `request` or `response`",
                    ));
                }

                let Some(response) = response.take() else {
                    return Err(syn::Error::new(
                        name.span(),
                        "Expected at least one `request`",
                    ));
                };

                endpoints.push(Endpoint {
                    parsed_attrs,
                    attrs,
                    vis,
                    name,
                    requests,
                    res: response,
                });
                continue;
            }

            if what == "broadcast" {
                let name = input.parse::<Ident>()?;

                let content;
                syn::braced!(content in input);

                let mut first = None;
                let mut events = Vec::new();

                while !content.is_empty() {
                    let ty = content.parse::<Ident>()?;

                    if ty == "event" {
                        if first.is_none() {
                            first = Some(content.parse()?);
                        } else {
                            events.push(content.parse()?);
                        }

                        continue;
                    }

                    return Err(syn::Error::new(ty.span(), "Expected `event`"));
                }

                let Some(first) = first.take() else {
                    return Err(syn::Error::new(
                        name.span(),
                        "Expected at least one `event`",
                    ));
                };

                broadcasts.push(Broadcast {
                    parsed_attrs,
                    attrs,
                    vis,
                    name,
                    first,
                    events,
                });
                continue;
            }

            return Err(syn::Error::new(
                what.span(),
                "Expected `endpoint` or `broadcast`",
            ));
        }

        Ok(())
    };

    let result = parser.parse2(input);

    if let Err(error) = result {
        cx.errors.borrow_mut().push(error);
    }

    let mut tokens = TokenStream::new();

    for endpoint in endpoints {
        endpoint.implement(cx, &mut tokens);
    }

    for broadcast in broadcasts {
        broadcast.implement(cx, &mut tokens);
    }

    tokens
}

struct Tokens<'a> {
    endpoint: TraitPath<'a>,
    broadcast: TraitPath<'a>,
    request: TraitPath<'a>,
    event: TraitPath<'a>,
    brace: syn::token::Brace,
    const_: Token![const],
    fn_: Token![fn],
    for_: Token![for],
    impl_: Token![impl],
    paren: syn::token::Paren,
    type_: Token![type],
}

pub(super) struct Context<'a> {
    t: Tokens<'a>,
    errors: RefCell<Vec<syn::Error>>,
}

impl Context<'_> {
    /// Coerce context into compile errors.
    pub(super) fn into_compile_errors(self) -> Option<TokenStream> {
        let errors = self.errors.into_inner();

        if errors.is_empty() {
            return None;
        }

        let mut tokens = TokenStream::new();

        for error in errors {
            let compile_error = error.to_compile_error();
            tokens.extend(compile_error);
        }

        Some(tokens)
    }

    fn do_not_implement(&self, name: &str, t: &mut TokenStream) {
        self.t.fn_.to_tokens(t);
        syn::Ident::new(name, Span::call_site()).to_tokens(t);
        self.t.paren.surround(t, |_| {});
        self.t.brace.surround(t, |_| {});
    }

    fn define_const(&self, name: &str, value: impl ToTokens, t: &mut TokenStream) {
        self.t.const_.to_tokens(t);
        <Ident>::new(name, Span::call_site()).to_tokens(t);
        <Token![:]>::default().to_tokens(t);
        <Token![&]>::default().to_tokens(t);
        Lifetime::new("'static", Span::call_site()).to_tokens(t);
        Ident::new("str", Span::call_site()).to_tokens(t);
        <Token![=]>::default().to_tokens(t);
        value.to_tokens(t);
        <Token![;]>::default().to_tokens(t);
    }
}

struct TraitPath<'a> {
    base: &'a Path,
    segments: Vec<Ident>,
}

impl<'a> TraitPath<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let colon_colon = <Token![::]>::default();

        self.base.to_tokens(tokens);

        for segment in &self.segments {
            colon_colon.to_tokens(tokens);
            segment.to_tokens(tokens);
        }
    }
}
