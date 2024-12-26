use core::cell::Cell;
use core::fmt;
use core::marker::PhantomData;

#[cfg(feature = "alloc")]
use crate::alloc::System;
use crate::alloc::{self, Allocator, String};
#[cfg(test)]
use crate::mode::Binary;
use crate::Context;

use super::ErrorMarker;

/// A simple non-diagnostical capturing context which ignores the error and
/// loses all information about it (except that it happened).
pub struct Ignore<M, A> {
    alloc: A,
    error: Cell<bool>,
    _marker: PhantomData<M>,
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<M> Ignore<M, &'static System> {
    /// Construct a new ignoring context with the [`System`] allocator.
    #[inline]
    pub fn new() -> Self {
        Self::with_alloc(crate::alloc::system())
    }
}

impl<M, A> Ignore<M, A> {
    /// Construct a new ignoring context.
    pub fn with_alloc(alloc: A) -> Self {
        Self {
            alloc,
            error: Cell::new(false),
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
impl<A> Ignore<Binary, A> {
    /// Construct a new ignoring context which collects an error marker.
    pub(crate) fn with_marker(alloc: A) -> Self {
        Self::with_alloc(alloc)
    }
}

impl<M, A> Ignore<M, A> {
    /// Construct an error or panic.
    pub fn unwrap(self) -> ErrorMarker {
        if self.error.get() {
            return ErrorMarker;
        }

        panic!("did not error")
    }
}

impl<M, A> Context for Ignore<M, A>
where
    M: 'static,
    A: Allocator,
{
    type Mode = M;
    type Error = ErrorMarker;
    type Mark = ();
    type Allocator = A;
    type String<'this>
        = String<'this, A>
    where
        Self: 'this;

    #[inline]
    fn clear(&self) {}

    #[inline]
    fn mark(&self) -> Self::Mark {}

    #[inline]
    fn advance(&self, _: usize) {}

    #[inline]
    fn alloc(&self) -> &Self::Allocator {
        &self.alloc
    }

    #[inline]
    fn collect_string<T>(&self, value: &T) -> Result<Self::String<'_>, Self::Error>
    where
        T: ?Sized + fmt::Display,
    {
        alloc::collect_string(self, value)
    }

    #[inline]
    fn custom<T>(&self, _: T) -> ErrorMarker
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        self.error.set(true);
        ErrorMarker
    }

    #[inline]
    fn message<T>(&self, _: T) -> ErrorMarker
    where
        T: fmt::Display,
    {
        self.error.set(true);
        ErrorMarker
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<M> Default for Ignore<M, &'static System> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
