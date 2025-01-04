use core::cell::Cell;
use core::fmt;

#[cfg(feature = "alloc")]
use crate::alloc::System;
use crate::{Allocator, Context};

use super::ErrorMarker;

/// A simple non-diagnostical capturing context which ignores the error and
/// loses all information about it (except that it happened).
pub struct Ignore<A> {
    alloc: A,
    error: Cell<bool>,
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl Ignore<System> {
    /// Construct a new ignoring context with the [`System`] allocator.
    #[inline]
    pub fn new() -> Self {
        Self::with_alloc(crate::alloc::System::new())
    }
}

impl<A> Ignore<A> {
    /// Construct a new ignoring context.
    #[inline]
    pub fn with_alloc(alloc: A) -> Self {
        Self {
            alloc,
            error: Cell::new(false),
        }
    }
}

#[cfg(test)]
impl<A> Ignore<A> {
    /// Construct a new ignoring context which collects an error marker.
    #[inline]
    pub(crate) fn with_marker(alloc: A) -> Self {
        Self::with_alloc(alloc)
    }
}

impl<A> Ignore<A> {
    /// Construct an error or panic.
    #[inline]
    pub fn unwrap(self) -> ErrorMarker {
        if self.error.get() {
            return ErrorMarker;
        }

        panic!("did not error")
    }
}

impl<A> Context for &Ignore<A>
where
    A: Clone + Allocator,
{
    type Error = ErrorMarker;
    type Mark = ();
    type Allocator = A;

    #[inline]
    fn clear(self) {}

    #[inline]
    fn mark(self) -> Self::Mark {}

    #[inline]
    fn advance(self, _: usize) {}

    #[inline]
    fn alloc(self) -> Self::Allocator {
        self.alloc.clone()
    }

    #[inline]
    fn custom<T>(self, _: T) -> ErrorMarker
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        self.error.set(true);
        ErrorMarker
    }

    #[inline]
    fn message<T>(self, _: T) -> ErrorMarker
    where
        T: fmt::Display,
    {
        self.error.set(true);
        ErrorMarker
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl Default for Ignore<System> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
