use core::error::Error;
use core::fmt;
use core::marker::PhantomData;

#[cfg(feature = "alloc")]
use crate::alloc::System;
use crate::alloc::{self, String};
#[cfg(test)]
use crate::mode::Binary;
use crate::{Allocator, Context};

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
    M: 'static,
    E: ContextError<A>,
    A: Allocator,
{
    alloc: A,
    _marker: PhantomData<(M, E)>,
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<M, E> Same<M, E, System>
where
    M: 'static,
    E: ContextError<System>,
{
    /// Construct a new same-error context with the [`System`] allocator.
    #[inline]
    pub fn new() -> Self {
        Self::with_alloc(crate::alloc::System::new())
    }
}

impl<M, E, A> Same<M, E, A>
where
    M: 'static,
    E: ContextError<A>,
    A: Allocator,
{
    /// Construct a new `Same` context with a custom allocator.
    #[inline]
    pub fn with_alloc(alloc: A) -> Self {
        Self {
            alloc,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
impl<A> Same<Binary, ErrorMarker, A>
where
    A: Allocator,
{
    /// Construct a new `Same` capturing context.
    #[inline]
    pub(crate) fn with_marker(alloc: A) -> Self {
        Self::with_alloc(alloc)
    }
}

impl<M, E, A> Context for &Same<M, E, A>
where
    M: 'static,
    E: ContextError<A>,
    A: Clone + Allocator,
{
    type Mode = M;
    type Error = E;
    type Mark = ();
    type Allocator = A;
    type String = String<A>;

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
    fn collect_string<T>(self, value: &T) -> Result<Self::String, Self::Error>
    where
        T: ?Sized + fmt::Display,
    {
        match alloc::collect_string(self.alloc(), value) {
            Ok(string) => Ok(string),
            Err(error) => Err(self.custom(error)),
        }
    }

    #[inline]
    fn custom<T>(self, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + Error,
    {
        E::custom(self.alloc(), message)
    }

    #[inline]
    fn message<T>(self, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        E::message(self.alloc(), message)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<M, E> Default for Same<M, E, System>
where
    M: 'static,
    E: ContextError<System>,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
