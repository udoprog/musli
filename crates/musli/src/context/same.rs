use core::error::Error;
use core::fmt;
use core::marker::PhantomData;

use crate::alloc::{self, Allocator, String};
#[cfg(feature = "alloc")]
use crate::alloc::{System, SYSTEM};
#[cfg(test)]
use crate::mode::Binary;
use crate::Context;

use super::ContextError;
#[cfg(test)]
use super::ErrorMarker;

/// A simple non-diagnostical capturing context which simply emits the original
/// error.
///
/// Using this should result in code which essentially just uses the emitted
/// error type directly.
pub struct Same<M, E, A>
where
    E: ContextError,
{
    alloc: A,
    _marker: PhantomData<(M, E)>,
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<M, E> Same<M, E, &'static System>
where
    E: ContextError,
{
    /// Construct a new same-error context with a custom allocator.
    pub fn new() -> Self {
        Self::with_alloc(&SYSTEM)
    }
}

impl<M, E, A> Same<M, E, A>
where
    E: ContextError,
{
    /// Construct a new `Same` context with a custom allocator.
    pub fn with_alloc(alloc: A) -> Self {
        Self {
            alloc,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
impl<A> Same<Binary, ErrorMarker, A> {
    /// Construct a new `Same` capturing context.
    pub(crate) fn with_marker(alloc: A) -> Self {
        Self::with_alloc(alloc)
    }
}

impl<M, E, A> Context for Same<M, E, A>
where
    M: 'static,
    A: Allocator,
    E: ContextError,
{
    type Mode = M;
    type Error = E;
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
    fn custom<T>(&self, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + Error,
    {
        E::custom(message)
    }

    #[inline]
    fn message<T>(&self, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        E::message(message)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<M, E> Default for Same<M, E, &'static System>
where
    E: ContextError,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
