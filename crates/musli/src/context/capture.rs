use core::cell::UnsafeCell;
use core::fmt;
use core::marker::PhantomData;

use crate::alloc::{self, Allocator, String};
#[cfg(feature = "alloc")]
use crate::alloc::{System, SYSTEM};
use crate::no_std;
use crate::Context;

use super::{ContextError, ErrorMarker};

/// A simple non-diagnostical capturing context.
pub struct Capture<M, E, A>
where
    E: ContextError,
{
    alloc: A,
    error: UnsafeCell<Option<E>>,
    _marker: PhantomData<M>,
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<M, E> Capture<M, E, &'static System>
where
    E: ContextError,
{
    /// Construct a new capturing context using the [`System`] allocator.
    pub fn new() -> Self {
        Self::with_alloc(&SYSTEM)
    }
}

impl<M, E, A> Capture<M, E, A>
where
    E: ContextError,
{
    /// Construct a new capturing allocator.
    pub fn with_alloc(alloc: A) -> Self {
        Self {
            alloc,
            error: UnsafeCell::new(None),
            _marker: PhantomData,
        }
    }

    /// Construct an error or panic.
    pub fn unwrap(self) -> E {
        let Some(error) = self.error.into_inner() else {
            return E::message("no error captured");
        };

        error
    }
}

impl<M, E, A> Context for Capture<M, E, A>
where
    M: 'static,
    E: ContextError,
    A: Allocator,
{
    type Mode = M;
    type Error = ErrorMarker;
    type Mark = ();
    type Allocator = A;
    type String<'this> = String<'this, A> where Self: 'this;

    #[inline]
    fn clear(&self) {
        // SAFETY: We're restricting access to the context, so that this is
        // safe.
        unsafe {
            (*self.error.get()) = None;
        }
    }

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
    fn custom<T>(&self, error: T) -> ErrorMarker
    where
        T: 'static + Send + Sync + no_std::Error,
    {
        // SAFETY: We're restricting access to the context, so that this is
        // safe.
        unsafe {
            self.error.get().replace(Some(E::custom(error)));
        }

        ErrorMarker
    }

    #[inline]
    fn message<T>(&self, message: T) -> ErrorMarker
    where
        T: fmt::Display,
    {
        // SAFETY: We're restricting access to the context, so that this is
        // safe.
        unsafe {
            self.error.get().replace(Some(E::message(message)));
        }

        ErrorMarker
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<M, E> Default for Capture<M, E, &'static System>
where
    E: ContextError,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
