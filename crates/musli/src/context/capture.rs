use core::cell::UnsafeCell;
use core::error::Error;
use core::fmt;

#[cfg(feature = "alloc")]
use crate::alloc::System;
use crate::{Allocator, Context};

use super::{ContextError, ErrorMarker};

/// A simple non-diagnostical capturing context.
pub struct Capture<E, A>
where
    E: ContextError<A>,
    A: Allocator,
{
    alloc: A,
    error: UnsafeCell<Option<E>>,
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<E> Capture<E, System>
where
    E: ContextError<System>,
{
    /// Construct a new capturing context using the [`System`] allocator.
    #[inline]
    pub fn new() -> Self {
        Self::with_alloc(System::new())
    }
}

impl<E, A> Capture<E, A>
where
    E: ContextError<A>,
    A: Clone + Allocator,
{
    /// Construct a new capturing allocator.
    pub fn with_alloc(alloc: A) -> Self {
        Self {
            alloc,
            error: UnsafeCell::new(None),
        }
    }

    /// Construct an error or panic.
    pub fn unwrap(self) -> E {
        let alloc = self.alloc();

        let Some(error) = self.error.into_inner() else {
            return E::message(alloc, "no error captured");
        };

        error
    }
}

impl<E, A> Context for &Capture<E, A>
where
    E: ContextError<A>,
    A: Clone + Allocator,
{
    type Error = ErrorMarker;
    type Mark = ();
    type Allocator = A;

    #[inline]
    fn clear(self) {
        // SAFETY: We're restricting access to the context, so that this is
        // safe.
        unsafe {
            (*self.error.get()) = None;
        }
    }

    #[inline]
    fn mark(self) -> Self::Mark {}

    #[inline]
    fn advance(self, _: usize) {}

    #[inline]
    fn alloc(self) -> Self::Allocator {
        self.alloc.clone()
    }

    #[inline]
    fn custom<T>(self, error: T) -> ErrorMarker
    where
        T: 'static + Send + Sync + Error,
    {
        let error = E::custom(self.alloc(), error);

        // SAFETY: We're restricting access to the context, so that this is
        // safe.
        unsafe {
            self.error.get().replace(Some(error));
        }

        ErrorMarker
    }

    #[inline]
    fn message<T>(self, message: T) -> ErrorMarker
    where
        T: fmt::Display,
    {
        let error = E::message(self.alloc(), message);

        // SAFETY: We're restricting access to the context, so that this is
        // safe.
        unsafe {
            self.error.get().replace(Some(error));
        }

        ErrorMarker
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<E> Default for Capture<E, System>
where
    E: ContextError<System>,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
