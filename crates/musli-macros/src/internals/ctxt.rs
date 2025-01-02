use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{self, Write};
use std::rc::Rc;

use proc_macro2::Span;

use super::attr::{ModeIdent, ModeKind};
use super::build::Field;
use super::ATTR;

struct Inner {
    b1: String,
    modes: HashMap<ModeKind, ModeIdent>,
    errors: Vec<syn::Error>,
    #[cfg(not(feature = "verbose"))]
    names: HashMap<String, usize>,
    #[cfg(not(feature = "verbose"))]
    types: usize,
}

pub(crate) struct Ctxt {
    inner: RefCell<Inner>,
}

impl Ctxt {
    /// Construct a new handling context.
    pub(crate) fn new() -> Self {
        Self {
            inner: RefCell::new(Inner {
                b1: String::new(),
                modes: HashMap::new(),
                errors: Vec::new(),
                #[cfg(not(feature = "verbose"))]
                names: HashMap::new(),
                #[cfg(not(feature = "verbose"))]
                types: 0,
            }),
        }
    }

    /// Emit diagnostics for a transparent encode / decode that failed because
    /// the wrong number of fields existed.
    pub(crate) fn transparent_diagnostics(&self, span: Span, fields: &[Rc<Field>]) {
        if fields.is_empty() {
            self.error_span(
                span,
                format_args!("#[{ATTR}(transparent)] types must have a single unskipped field"),
            );
        } else {
            self.error_span(
                span,
                format_args!(
                    "#[{ATTR}(transparent)] can only be used on types which have a single field",
                ),
            );
        }
    }

    /// Register a new mode.
    pub(crate) fn register_mode(&self, mode: ModeIdent) {
        self.inner
            .borrow_mut()
            .modes
            .insert(mode.kind.clone(), mode);
    }

    /// Test if context contains errors.
    pub(crate) fn has_errors(&self) -> bool {
        !self.inner.borrow().errors.is_empty()
    }

    /// Report an error with a span.
    pub(crate) fn error_span<T>(&self, span: Span, message: T)
    where
        T: fmt::Display,
    {
        self.inner
            .borrow_mut()
            .errors
            .push(syn::Error::new(span, message));
    }

    /// Error reported directly by syn.
    pub(crate) fn syn_error(&self, error: syn::Error) {
        self.inner.borrow_mut().errors.push(error);
    }

    /// Access interior errors.
    pub(crate) fn into_errors(self) -> Vec<syn::Error> {
        std::mem::take(&mut self.inner.borrow_mut().errors)
    }

    /// Get all extra modes specified.
    pub(crate) fn modes(&self) -> Vec<ModeIdent> {
        self.inner.borrow().modes.values().cloned().collect()
    }

    pub(crate) fn reset(&self) {
        #[cfg(not(feature = "verbose"))]
        {
            let mut inner = self.inner.borrow_mut();
            inner.names.clear();
            inner.types = 0;
        }
    }

    /// Build a lifetime.
    #[allow(unused)]
    pub(crate) fn lifetime(&self, name: &str) -> syn::Lifetime {
        self.with_string("'", name, "", |s| syn::Lifetime::new(s, Span::call_site()))
    }

    /// Build an identifier with the given name, escaped so it's harder to conflict with.
    pub(crate) fn ident(&self, name: &str) -> syn::Ident {
        self.ident_with_span(name, Span::call_site(), "")
    }

    /// Build an identifier with the given name, escaped so it's harder to conflict with.
    pub(crate) fn ident_with_span(&self, name: &str, span: Span, extra: &str) -> syn::Ident {
        self.with_string("", name, extra, |s| syn::Ident::new(s, span))
    }

    fn with_string<F, O>(&self, prefix: &str, name: &str, suffix: &str, f: F) -> O
    where
        F: FnOnce(&str) -> O,
    {
        let mut inner = self.inner.borrow_mut();

        #[cfg(not(feature = "verbose"))]
        {
            let index = if let Some(index) = inner.names.get(name) {
                *index
            } else {
                let index = inner.names.len();
                inner.names.insert(name.to_owned(), index);
                index
            };

            _ = write!(inner.b1, "{prefix}_{index}{suffix}");
        }

        #[cfg(feature = "verbose")]
        {
            let name = name.strip_prefix('_').unwrap_or(name);
            _ = write!(inner.b1, "{prefix}_{name}{suffix}");
        }

        let ident = f(&inner.b1);
        inner.b1.clear();
        ident
    }

    /// Build a type identifier with a span.
    pub(crate) fn type_with_span<N>(&self, name: N, span: Span) -> syn::Ident
    where
        N: fmt::Display,
    {
        self.type_with_span_inner(name, span, false)
    }

    /// Build a type identifier with a span.
    pub(crate) fn type_with_span_permanent<N>(&self, name: N, span: Span) -> syn::Ident
    where
        N: fmt::Display,
    {
        self.type_with_span_inner(name, span, true)
    }

    fn type_with_span_inner<N>(&self, name: N, span: Span, permanent: bool) -> syn::Ident
    where
        N: fmt::Display,
    {
        let mut inner = self.inner.borrow_mut();

        #[cfg(not(feature = "verbose"))]
        {
            _ = name;

            let index;

            if permanent {
                _ = write!(inner.b1, "{name}");
            } else {
                index = inner.types;
                inner.types += 1;
                _ = write!(inner.b1, "T{index}");
            }
        }

        #[cfg(feature = "verbose")]
        {
            _ = permanent;
            _ = write!(inner.b1, "{name}");
        }

        let ident = syn::Ident::new(&inner.b1, span);
        inner.b1.clear();
        ident
    }
}
