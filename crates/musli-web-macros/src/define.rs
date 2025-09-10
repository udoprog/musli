use std::cell::RefCell;
use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::parse::{ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{Attribute, Generics, Ident, LitInt, Path, Token, Type, Visibility};

pub(super) fn cx(base: &Path) -> Context<'_> {
    let errors = Vec::new();

    macro_rules! path {
        ($first:ident $(:: $remaining:ident)*) => {
            TraitPath {
                base,
                segments: vec![
                    Ident::new(stringify!($first), Span::call_site()),
                    $(Ident::new(stringify!($remaining), Span::call_site()),)*
                ],
            }
        }
    }

    Context {
        t: Tokens {
            api: path!(api),
            message_id: path!(api::MessageId),
            fmt: path!(__macros::fmt),
            brace: syn::token::Brace::default(),
            colon_colon: <Token![::]>::default(),
            enum_: <Token![enum]>::default(),
            eq: <Token![=]>::default(),
            impl_: <Token![impl]>::default(),
            semi: <Token![;]>::default(),
            type_: <Token![type]>::default(),
        },
        errors: RefCell::new(errors),
    }
}

struct AssocType {
    attrs: Vec<Attribute>,
    type_: Token![type],
    what: Ident,
    generics: Generics,
    eq: Token![=],
    ty: Type,
    semi: Token![;],
}

impl AssocType {
    #[inline]
    fn parse(
        attrs: Vec<Attribute>,
        type_: Token![type],
        what: Ident,
        input: ParseStream,
    ) -> syn::Result<Self> {
        Ok(Self {
            attrs,
            type_,
            what,
            generics: input.parse::<Generics>()?,
            eq: input.parse()?,
            ty: input.parse::<Type>()?,
            semi: input.parse()?,
        })
    }

    #[inline]
    fn span(&self) -> Span {
        let end = self.ty.span();
        self.what.span().join(end).unwrap_or(end)
    }
}

struct ImplType {
    attrs: Vec<Attribute>,
    impl_: Token![impl],
    generics: Generics,
    what: Ident,
    for_: Token![for],
    ty: Type,
    semi: Token![;],
}

impl ImplType {
    #[inline]
    fn parse(
        attrs: Vec<Attribute>,
        impl_: Token![impl],
        mut generics: Generics,
        what: Ident,
        input: ParseStream,
    ) -> syn::Result<Self> {
        let for_ = input.parse()?;
        let ty = input.parse::<Type>()?;
        generics.where_clause = input.parse()?;
        let semi = input.parse()?;

        Ok(Self {
            attrs,
            impl_,
            generics,
            what,
            for_,
            ty,
            semi,
        })
    }
}

#[derive(Default)]
struct ParsedAttrs {
    id: Option<(u16, Span)>,
}

impl ParsedAttrs {
    fn deny(self, cx: &Context<'_>) {
        if let Some((_, span)) = &self.id {
            cx.errors.borrow_mut().push(syn::Error::new(
                *span,
                "The `#[musli(kind)]` attribute cannot be specified here",
            ));
        }
    }
}

struct TypeDeclBuilder {
    id: Option<(u16, Span)>,
    attrs: Vec<Attribute>,
    #[allow(dead_code)]
    vis: Visibility,
    #[allow(dead_code)]
    type_: Token![type],
    name: Ident,
    #[allow(dead_code)]
    semi: Token![;],
}

impl TypeDeclBuilder {
    fn parse(
        parsed_attrs: ParsedAttrs,
        attrs: Vec<Attribute>,
        vis: Visibility,
        type_: Token![type],
        input: ParseStream,
    ) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        let semi = input.parse::<Token![;]>()?;

        Ok(Self {
            id: parsed_attrs.id,
            attrs,
            vis,
            type_,
            name,
            semi,
        })
    }
}

struct TypeDecl {
    id: (u16, Span),
    attrs: Vec<Attribute>,
    #[allow(dead_code)]
    vis: Visibility,
    #[allow(dead_code)]
    type_: Token![type],
    name: Ident,
    #[allow(dead_code)]
    semi: Token![;],
    endpoint: bool,
    broadcast: bool,
}

impl TypeDecl {
    fn implement(&self, cx: &Context, t: &mut TokenStream) {
        for attr in &self.attrs {
            attr.to_tokens(t);
        }

        self.vis.to_tokens(t);
        cx.t.enum_.to_tokens(t);
        self.name.to_tokens(t);
        cx.t.brace.surround(t, |_| {});

        cx.t.impl_.to_tokens(t);
        self.name.to_tokens(t);
        cx.t.brace.surround(t, |t| {
            self.vis.to_tokens(t);
            cx.define_id("ID", self.id, t);
        });
    }
}

struct Endpoint {
    attrs: Vec<Attribute>,
    impl_: Token![impl],
    generics: Generics,
    what: Ident,
    for_: Token![for],
    name: Ident,
    brace: syn::token::Brace,
    requests: Vec<ImplType>,
    res: AssocType,
}

impl Endpoint {
    fn parse(
        cx: &Context,
        parsed_attrs: ParsedAttrs,
        attrs: Vec<Attribute>,
        impl_: Token![impl],
        mut generics: Generics,
        what: Ident,
        input: ParseStream,
    ) -> syn::Result<Self> {
        parsed_attrs.deny(cx);

        let for_ = input.parse()?;
        let name = input.parse::<Ident>()?;
        generics.where_clause = input.parse()?;

        let content;
        let brace = syn::braced!(content in input);

        let mut requests = Vec::new();
        let mut response = None::<AssocType>;

        while !content.is_empty() {
            let attrs = content.call(Attribute::parse_outer)?;

            if let Some(impl_) = content.parse::<Option<Token![impl]>>()? {
                let generics = content.parse::<Generics>()?;
                let what = content.parse::<Ident>()?;

                if what == "Request" {
                    requests.push(ImplType::parse(attrs, impl_, generics, what, &content)?);
                    continue;
                }

                return Err(syn::Error::new(
                    what.span(),
                    "Unsupported impl type, expected `Request`",
                ));
            }

            if let Some(type_) = content.parse::<Option<Token![type]>>()? {
                let what = content.parse::<Ident>()?;

                if what == "Response" {
                    if let Some(response) = response.take() {
                        cx.errors.borrow_mut().push(syn::Error::new(
                            response.span(),
                            "Expected at most one `response`",
                        ));
                    }

                    response = Some(AssocType::parse(attrs, type_, what, &content)?);
                    continue;
                }

                return Err(syn::Error::new(
                    what.span(),
                    "Unsupported associated type, expected `Response`",
                ));
            }

            return Err(syn::Error::new(content.span(), "Expected `impl` or `type`"));
        }

        let Some(res) = response.take() else {
            return Err(syn::Error::new(
                name.span(),
                "Expected at least one `Request`",
            ));
        };

        Ok(Endpoint {
            attrs,
            impl_,
            generics,
            what,
            for_,
            name,
            brace,
            requests,
            res,
        })
    }

    fn implement(&self, cx: &Context, types: &mut [TypeDecl], t: &mut TokenStream) {
        let Some(ty) = types.iter_mut().find(|ty| ty.name == self.name) else {
            cx.errors.borrow_mut().push(syn::Error::new(
                self.name.span(),
                format_args!(
                    "Expected corresponding `type` declaration for `{}`",
                    self.name
                ),
            ));
            return;
        };

        ty.endpoint = true;

        let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();

        {
            for attr in &self.attrs {
                attr.to_tokens(t);
            }

            self.impl_.to_tokens(t);
            impl_generics.to_tokens(t);
            cx.t.api.to_tokens(t);
            cx.t.colon_colon.to_tokens(t);
            Ident::new("Endpoint", self.what.span()).to_tokens(t);
            self.for_.to_tokens(t);
            self.name.to_tokens(t);
            type_generics.to_tokens(t);
            where_clause.to_tokens(t);

            self.brace.surround(t, |t| {
                cx.define_id("ID", ty.id, t);

                for attr in &self.res.attrs {
                    attr.to_tokens(t);
                }

                self.res.type_.to_tokens(t);
                self.res.what.to_tokens(t);
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

            let (impl_generics, _, where_clause) = req.generics.split_for_impl();

            req.impl_.to_tokens(t);
            impl_generics.to_tokens(t);
            cx.t.api.to_tokens(t);
            cx.t.colon_colon.to_tokens(t);
            req.what.to_tokens(t);
            req.for_.to_tokens(t);
            req.ty.to_tokens(t);
            where_clause.to_tokens(t);

            cx.t.brace.surround(t, |t| {
                cx.t.type_.to_tokens(t);
                Ident::new("Endpoint", Span::call_site()).to_tokens(t);
                cx.t.eq.to_tokens(t);
                self.name.to_tokens(t);
                req.semi.to_tokens(t);

                cx.do_not_implement("__do_not_implement_request", t);
            });
        }
    }
}

struct Broadcast {
    attrs: Vec<Attribute>,
    impl_: Token![impl],
    generics: Generics,
    what: Ident,
    for_: Token![for],
    name: Ident,
    brace: syn::token::Brace,
    first: ImplType,
    remaining: Vec<ImplType>,
}

impl Broadcast {
    fn parse(
        cx: &Context,
        parsed_attrs: ParsedAttrs,
        attrs: Vec<Attribute>,
        impl_: Token![impl],
        generics: Generics,
        what: Ident,
        input: ParseStream,
    ) -> syn::Result<Self> {
        parsed_attrs.deny(cx);

        let for_ = input.parse()?;
        let name = input.parse::<Ident>()?;

        let content;
        let brace = syn::braced!(content in input);

        let mut first = None;
        let mut remaining = Vec::new();

        while !content.is_empty() {
            let attrs = content.call(Attribute::parse_outer)?;

            if let Some(impl_) = content.parse::<Option<Token![impl]>>()? {
                let generics = content.parse::<Generics>()?;
                let what = content.parse::<Ident>()?;

                if what == "Event" {
                    let impl_type = ImplType::parse(attrs, impl_, generics, what, &content)?;

                    if first.is_none() {
                        first = Some(impl_type);
                    } else {
                        remaining.push(impl_type);
                    }

                    continue;
                }

                return Err(syn::Error::new(what.span(), "Expected `Event`"));
            }

            return Err(syn::Error::new(content.span(), "Expected `impl`"));
        }

        let Some(first) = first.take() else {
            return Err(syn::Error::new(
                name.span(),
                "Expected at least one `event`",
            ));
        };

        Ok(Self {
            attrs,
            impl_,
            generics,
            what,
            for_,
            name,
            brace,
            first,
            remaining,
        })
    }

    fn implement(&self, cx: &Context, types: &mut [TypeDecl], t: &mut TokenStream) {
        let Some(ty) = types.iter_mut().find(|ty| ty.name == self.name) else {
            cx.errors.borrow_mut().push(syn::Error::new(
                self.name.span(),
                format_args!(
                    "Expected corresponding `type` declaration for `{}`",
                    self.name
                ),
            ));
            return;
        };

        ty.broadcast = true;

        let (impl_generics, type_generics, where_clause) = self.generics.split_for_impl();

        {
            for attr in &self.attrs {
                attr.to_tokens(t);
            }

            self.impl_.to_tokens(t);
            impl_generics.to_tokens(t);
            cx.t.api.to_tokens(t);
            cx.t.colon_colon.to_tokens(t);
            Ident::new("Broadcast", self.what.span()).to_tokens(t);
            self.for_.to_tokens(t);
            self.name.to_tokens(t);
            type_generics.to_tokens(t);
            where_clause.to_tokens(t);

            self.brace.surround(t, |t| {
                cx.define_id("ID", ty.id, t);

                cx.t.type_.to_tokens(t);
                Ident::new("Event", Span::call_site()).to_tokens(t);
                self.first.generics.to_tokens(t);
                cx.t.eq.to_tokens(t);
                self.first.ty.to_tokens(t);
                cx.t.semi.to_tokens(t);

                cx.do_not_implement("__do_not_implement_broadcast", t);
            });
        }

        for ev in [&self.first].into_iter().chain(&self.remaining) {
            for attr in &ev.attrs {
                attr.to_tokens(t);
            }

            let (impl_generics, _, where_clause) = ev.generics.split_for_impl();

            ev.impl_.to_tokens(t);
            impl_generics.to_tokens(t);
            cx.t.api.to_tokens(t);
            cx.t.colon_colon.to_tokens(t);
            Ident::new("Event", ev.what.span()).to_tokens(t);
            ev.for_.to_tokens(t);
            ev.ty.to_tokens(t);
            where_clause.to_tokens(t);

            cx.t.brace.surround(t, |t| {
                cx.t.type_.to_tokens(t);
                Ident::new("Broadcast", Span::call_site()).to_tokens(t);
                cx.t.eq.to_tokens(t);
                self.name.to_tokens(t);
                cx.t.semi.to_tokens(t);

                cx.do_not_implement("__do_not_implement_event", t);
            });
        }
    }
}

pub(super) fn expand(cx: &Context, input: TokenStream) -> TokenStream {
    let mut builders = Vec::new();
    let mut endpoints = Vec::new();
    let mut broadcasts = Vec::new();

    let parser = |input: ParseStream| {
        while !input.is_empty() {
            let mut attrs = Vec::new();
            let mut parsed_attrs = ParsedAttrs::default();

            for attr in input.call(Attribute::parse_outer)? {
                if !attr.path().is_ident("musli") {
                    attrs.push(attr);
                    continue;
                }

                let result = attr.parse_args_with(|input: ParseStream| {
                    let key = input.parse::<Ident>()?;

                    if key == "id" {
                        input.parse::<Token![=]>()?;
                        let lit = input.parse::<LitInt>()?;
                        let id = lit.base10_parse()?;
                        parsed_attrs.id = Some((id, lit.span()));
                        return Ok(());
                    }

                    Err(syn::Error::new(
                        key.span(),
                        "Unsupported attribute, expected `id`",
                    ))
                });

                if let Err(error) = result {
                    cx.errors.borrow_mut().push(error);
                }
            }

            let vis = input.parse::<Visibility>()?;

            if let Some(type_) = input.parse::<Option<Token![type]>>()? {
                builders.push(TypeDeclBuilder::parse(
                    parsed_attrs,
                    attrs,
                    vis,
                    type_,
                    input,
                )?);
                continue;
            };

            if let Some(impl_) = input.parse::<Option<Token![impl]>>()? {
                if !matches!(vis, Visibility::Inherited) {
                    return Err(syn::Error::new(
                        vis.span(),
                        "`impl` cannot be preceded by visibility",
                    ));
                }

                let generics = input.parse::<Generics>()?;
                let what = input.parse::<Ident>()?;

                if what == "Endpoint" {
                    endpoints.push(Endpoint::parse(
                        cx,
                        parsed_attrs,
                        attrs,
                        impl_,
                        generics,
                        what,
                        input,
                    )?);

                    continue;
                }

                if what == "Broadcast" {
                    broadcasts.push(Broadcast::parse(
                        cx,
                        parsed_attrs,
                        attrs,
                        impl_,
                        generics,
                        what,
                        input,
                    )?);

                    continue;
                }

                return Err(syn::Error::new(
                    what.span(),
                    "Expected `Endpoint` or `Broadcast`",
                ));
            }

            return Err(syn::Error::new(input.span(), "Expected `type` or `impl`"));
        }

        Ok(())
    };

    let result = parser.parse2(input);

    if let Err(error) = result {
        cx.errors.borrow_mut().push(error);
    }

    let mut alloc = KindAlloc::new();

    for ty in &builders {
        if let Some((id, span)) = ty.id {
            if !alloc.used.insert(id) {
                cx.errors.borrow_mut().push(syn::Error::new(
                    span,
                    format_args!("Message id `{id}` has already been used"),
                ));
            }
        }
    }

    let mut types = Vec::with_capacity(builders.len());

    for ty in builders {
        let id = match ty.id {
            Some((id, span)) => (id, span),
            None => (alloc.allocate(), ty.name.span()),
        };

        types.push(TypeDecl {
            id,
            attrs: ty.attrs,
            vis: ty.vis,
            type_: ty.type_,
            name: ty.name,
            semi: ty.semi,
            endpoint: false,
            broadcast: false,
        });
    }

    let mut tokens = TokenStream::new();

    for ty in &types {
        ty.implement(cx, &mut tokens);
    }

    for endpoint in endpoints {
        endpoint.implement(cx, &mut types, &mut tokens);
    }

    for broadcast in broadcasts {
        broadcast.implement(cx, &mut types, &mut tokens);
    }

    let message_id = &cx.t.message_id;
    let fmt = &cx.t.fmt;
    let api = &cx.t.api;

    let idents = types.iter().map(|ty| &ty.name).collect::<Vec<_>>();

    let requests = types
        .iter()
        .filter(|ty| ty.endpoint)
        .map(|ty| &ty.name)
        .collect::<Vec<_>>();

    let request_values = types
        .iter()
        .filter(|ty| ty.endpoint)
        .map(|ty| ty.id.0)
        .collect::<Vec<_>>();

    if requests.is_empty() {
        tokens.extend(quote! {
            /// Enum of request types used in this protocol.
            pub enum Request {}

            impl #api::Id for Request {
                #[inline]
                fn from_raw(_id: u16) -> Option<Self> {
                    None
                }
            }

            impl #fmt::Debug for Request {
                fn fmt(&self, f: &mut #fmt::Formatter<'_>) -> #fmt::Result {
                    match *self {}
                }
            }
        });
    } else {
        tokens.extend(quote! {
            /// Enum of request types used in this protocol.
            #[repr(u16)]
            pub enum Request {
                #(#requests = #request_values,)*
            }

            impl #api::Id for Request {
                #[inline]
                fn from_raw(id: u16) -> Option<Self> {
                    match id {
                        #(#request_values => Some(Self::#requests),)*
                        _ => None,
                    }
                }
            }

            impl #fmt::Debug for Request {
                fn fmt(&self, f: &mut #fmt::Formatter<'_>) -> #fmt::Result {
                    match *self {
                        #(Self::#requests => f.write_str(stringify!(#requests)),)*
                    }
                }
            }
        });
    }

    tokens.extend(quote! {
        /// Debug a message id.
        ///
        /// This coerced the debug message into a type which implements
        /// `fmt::Debug` that can be used for to visualize the name of the
        /// message being received from a message identifier.
        pub fn debug_id(id: #message_id) -> impl #fmt::Debug {
            enum Debug {
                Known(&'static str),
                Unknown(#message_id),
            }

            impl #fmt::Debug for Debug {
                fn fmt(&self, f: &mut #fmt::Formatter<'_>) -> #fmt::Result {
                    match *self {
                        Debug::Known(name) => f.write_str(name),
                        Debug::Unknown(id) => f.debug_tuple("Unknown").field(&id.get()).finish(),
                    }
                }
            }

            match id {
                #(#idents::ID => Debug::Known(stringify!(#idents)),)*
                id => Debug::Unknown(id),
            }
        }
    });

    tokens
}

struct Tokens<'a> {
    api: TraitPath<'a>,
    message_id: TraitPath<'a>,
    fmt: TraitPath<'a>,
    brace: syn::token::Brace,
    colon_colon: Token![::],
    enum_: Token![enum],
    eq: Token![=],
    impl_: Token![impl],
    semi: Token![;],
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
        let name = Ident::new(name, Span::call_site());
        t.extend(quote!(fn #name() {}));
    }

    fn define_id(&self, name: &str, (value, span): (u16, Span), t: &mut TokenStream) {
        if value == 0 || value >= i16::MAX as u16 {
            self.errors.borrow_mut().push(syn::Error::new(
                span,
                format_args!("Message id `{value}` not in range 1-{}", i16::MAX as u16),
            ));

            return;
        }

        let name = Ident::new(name, Span::call_site());
        let message_id = &self.t.message_id;

        t.extend(quote! {
            const #name: #message_id = unsafe { #message_id::new_unchecked(#value) };
        });
    }
}

struct TraitPath<'a> {
    base: &'a Path,
    segments: Vec<Ident>,
}

impl ToTokens for TraitPath<'_> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let colon_colon = <Token![::]>::default();

        self.base.to_tokens(tokens);

        for segment in &self.segments {
            colon_colon.to_tokens(tokens);
            segment.to_tokens(tokens);
        }
    }
}

struct KindAlloc {
    used: HashSet<u16>,
    next: u16,
}

impl KindAlloc {
    fn new() -> Self {
        Self {
            used: HashSet::new(),
            next: 1,
        }
    }

    fn allocate(&mut self) -> u16 {
        while self.used.contains(&self.next) {
            self.next = self.next.saturating_add(1);
        }

        let id = self.next;
        self.used.insert(id);
        self.next = self.next.saturating_add(1);
        id
    }
}
