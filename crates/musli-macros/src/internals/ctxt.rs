use core::fmt;
use std::cell::RefCell;

use proc_macro2::Span;
use quote::ToTokens;

pub(crate) struct Ctxt {
    errors: RefCell<Vec<syn::Error>>,
}

impl Ctxt {
    /// Construct a new handling context.
    pub(crate) fn new() -> Self {
        Self {
            errors: RefCell::new(Vec::new()),
        }
    }

    /// Test if context contains errors.
    pub(crate) fn has_errors(&self) -> bool {
        !self.errors.borrow().is_empty()
    }

    /// Report an error with a span.
    pub(crate) fn error_spanned_by<S, T>(&self, spanned: S, message: T)
    where
        S: ToTokens,
        T: fmt::Display,
    {
        self.errors
            .borrow_mut()
            .push(syn::Error::new_spanned(spanned, message));
    }

    /// Report an error with a span.
    pub(crate) fn error_span<T>(&self, span: Span, message: T)
    where
        T: fmt::Display,
    {
        self.errors
            .borrow_mut()
            .push(syn::Error::new(span, message));
    }

    /// Error reported directly by syn.
    pub(crate) fn syn_error(&self, error: syn::Error) {
        self.errors.borrow_mut().push(error);
    }

    /// Access interior errors.
    pub(crate) fn into_errors(self) -> Vec<syn::Error> {
        std::mem::take(&mut *self.errors.borrow_mut())
    }
}
