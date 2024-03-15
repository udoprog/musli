use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::{self, Write};

use proc_macro2::Span;

struct Inner {
    b1: String,
    b2: String,
    modes: HashSet<syn::Path>,
    errors: Vec<syn::Error>,
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
                b2: String::new(),
                modes: HashSet::new(),
                errors: Vec::new(),
            }),
        }
    }

    /// Register a new mode.
    pub(crate) fn register_mode(&self, mode: syn::Path) {
        self.inner.borrow_mut().modes.insert(mode);
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

    /// Get all "extra" modes specified.
    pub(crate) fn modes(&self) -> Vec<syn::Path> {
        self.inner.borrow().modes.iter().cloned().collect()
    }

    /// Build an identifier with the given name, escaped so it's harder to conflict with.
    pub(crate) fn ident(&self, name: &str) -> syn::Ident {
        let name = format!("i_{name}");
        syn::Ident::new(&name, Span::call_site())
    }

    /// Build an identifier with the given name, escaped so it's harder to conflict with.
    pub(crate) fn ident_with_span(&self, name: &str, span: Span) -> syn::Ident {
        let name = format!("i_{name}");
        syn::Ident::new(&name, span)
    }

    /// Build a type identifier with a span.
    pub(crate) fn type_with_span(&self, name: &str, span: Span) -> syn::Ident {
        let name = format!("T{name}");
        syn::Ident::new(&name, span)
    }

    /// Escape an ident so it's harder to conflict with, preserving idents span
    pub(crate) fn field_ident(&self, ident: &syn::Ident) -> syn::Ident {
        let mut inner = self.inner.borrow_mut();

        let Inner { b1, b2, .. } = &mut *inner;

        write!(b1, "{ident}").unwrap();

        if let Some(rest) = b1.strip_prefix('_') {
            write!(b2, "_f_{rest}").unwrap()
        } else {
            write!(b2, "f_{ident}").unwrap()
        }

        let ident = syn::Ident::new(b2, ident.span());

        b1.clear();
        b2.clear();

        ident
    }
}
