use core::fmt;
use std::cell::RefCell;
use std::collections::HashSet;

use proc_macro2::Span;
use quote::ToTokens;

struct Inner {
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
    pub(crate) fn error_spanned_by<S, T>(&self, spanned: S, message: T)
    where
        S: ToTokens,
        T: fmt::Display,
    {
        self.inner
            .borrow_mut()
            .errors
            .push(syn::Error::new_spanned(spanned, message));
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
}
