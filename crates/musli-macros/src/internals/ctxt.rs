use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::{self, Write};
#[cfg(not(feature = "verbose"))]
use std::collections::HashMap;

use proc_macro2::Span;

struct Inner {
    b1: String,
    modes: HashSet<syn::Path>,
    errors: Vec<syn::Error>,
    names: HashMap<String, usize>,
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
                modes: HashSet::new(),
                errors: Vec::new(),
                names: HashMap::new(),
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

    pub(crate) fn reset(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.names.clear();
    }

    /// Build an identifier with the given name, escaped so it's harder to conflict with.
    pub(crate) fn ident(&self, name: &str) -> syn::Ident {
        self.ident_with_span(name, Span::call_site(), "")
    }

    /// Build an identifier with the given name, escaped so it's harder to conflict with.
    pub(crate) fn ident_with_span(&self, name: &str, span: Span, extra: &str) -> syn::Ident {
        let mut inner = self.inner.borrow_mut();

        let index = if let Some(index) = inner.names.get(name) {
            *index
        } else {
            let index = inner.names.len();
            inner.names.insert(name.to_owned(), index);
            index
        };

        #[cfg(not(feature = "verbose"))]
        {
            _ = write!(inner.b1, "_{extra}{index}");
        }

        #[cfg(feature = "verbose")]
        {
            let name = name.strip_prefix("_").unwrap_or(name);
            _ = write!(inner.b1, "_{extra}{name}");
        }

        let ident = syn::Ident::new(&inner.b1, span);
        inner.b1.clear();
        ident
    }

    /// Build a type identifier with a span.
    pub(crate) fn type_with_span(&self, name: &str, span: Span) -> syn::Ident {
        let name = format!("T{name}");
        syn::Ident::new(&name, span)
    }
}
