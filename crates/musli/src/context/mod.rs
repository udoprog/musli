//! [`Context`] implementations.
//!
//! [`Context`]: crate::Context

mod access;
mod error_marker;
mod rich_error;
mod stack_context;
#[cfg(feature = "alloc")]
mod system_context;
pub use self::error_marker::ErrorMarker;
mod error;
pub use self::error::Error;

use core::cell::{Cell, UnsafeCell};
use core::fmt;
use core::marker::PhantomData;

use musli_core::mode::Binary;
use musli_core::{Allocator, Context, StdError};

use crate::buf::{self, BufString};

#[cfg(feature = "alloc")]
pub use self::system_context::SystemContext;

pub use self::stack_context::StackContext;

pub use self::rich_error::RichError;

/// A simple non-diagnostical capturing context which simply emits the original
/// error.
///
/// Using this should result in code which essentially just uses the emitted
/// error type directly.
pub struct Same<A, M, E> {
    alloc: A,
    _marker: PhantomData<(M, E)>,
}

impl<A, M, E> Same<A, M, E> {
    /// Construct a new `Same` capturing context.
    pub fn new(alloc: A) -> Self {
        Self {
            alloc,
            _marker: PhantomData,
        }
    }
}

impl<A> Same<A, Binary, ErrorMarker> {
    /// Construct a new `Same` capturing context.
    #[inline]
    #[doc(hidden)]
    pub fn marker(alloc: A) -> Self {
        Self::new(alloc)
    }
}

impl<A, M, E> Default for Same<A, M, E>
where
    A: Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            alloc: A::default(),
            _marker: PhantomData,
        }
    }
}

impl<A, M, E> Context for Same<A, M, E>
where
    A: Allocator,
    E: Error,
{
    type Mode = M;
    type Error = E;
    type Mark = ();
    type Buf<'this> = A::Buf<'this> where Self: 'this;
    type BufString<'this> = BufString<A::Buf<'this>> where Self: 'this;

    #[inline]
    fn clear(&self) {}

    #[inline]
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        self.alloc.alloc()
    }

    #[inline]
    fn collect_string<T>(&self, value: &T) -> Result<Self::BufString<'_>, Self::Error>
    where
        T: ?Sized + fmt::Display,
    {
        buf::collect_string(self, value)
    }

    #[inline]
    fn custom<T>(&self, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + StdError,
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

/// A simple non-diagnostical capturing context which ignores the error and
/// loses all information about it (except that it happened).
pub struct Ignore<A, M, E> {
    alloc: A,
    error: Cell<bool>,
    _marker: PhantomData<(M, E)>,
}

impl<A, M, E> Default for Ignore<A, M, E>
where
    A: Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            alloc: A::default(),
            error: Cell::new(false),
            _marker: PhantomData,
        }
    }
}

impl<A, M, E> Ignore<A, M, E> {
    /// Construct a new ignoring context.
    pub fn new(alloc: A) -> Self {
        Self {
            alloc,
            error: Cell::new(false),
            _marker: PhantomData,
        }
    }
}

impl<A> Ignore<A, Binary, ErrorMarker> {
    /// Construct a new ignoring context which collects an error marker.
    #[doc(hidden)]
    pub fn marker(alloc: A) -> Self {
        Self::new(alloc)
    }
}

impl<A, M, E> Ignore<A, M, E>
where
    E: Error,
{
    /// Construct an error or panic.
    pub fn unwrap(self) -> E {
        if self.error.get() {
            return E::message("error");
        }

        panic!("did not error")
    }
}

impl<A, M, E: 'static> Context for Ignore<A, M, E>
where
    A: Allocator,
{
    type Mode = M;
    type Error = ErrorMarker;
    type Mark = ();
    type Buf<'this> = A::Buf<'this> where Self: 'this;
    type BufString<'this> = BufString<A::Buf<'this>> where Self: 'this;

    #[inline]
    fn clear(&self) {}

    #[inline]
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        self.alloc.alloc()
    }

    #[inline]
    fn collect_string<T>(&self, value: &T) -> Result<Self::BufString<'_>, Self::Error>
    where
        T: ?Sized + fmt::Display,
    {
        buf::collect_string(self, value)
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

/// A simple non-diagnostical capturing context.
pub struct Capture<A, M, E> {
    alloc: A,
    error: UnsafeCell<Option<E>>,
    _marker: PhantomData<M>,
}

impl<A, M, E> Capture<A, M, E> {
    /// Construct a new capturing allocator.
    pub fn new(alloc: A) -> Self {
        Self {
            alloc,
            error: UnsafeCell::new(None),
            _marker: PhantomData,
        }
    }

    /// Construct an error or panic.
    pub fn unwrap(self) -> E {
        if let Some(error) = self.error.into_inner() {
            return error;
        }

        panic!("no error captured")
    }
}

impl<A, M, E> Context for Capture<A, M, E>
where
    A: Allocator,
    E: Error,
{
    type Mode = M;
    type Error = ErrorMarker;
    type Mark = ();
    type Buf<'this> = A::Buf<'this> where Self: 'this;
    type BufString<'this> = BufString<A::Buf<'this>> where Self: 'this;

    #[inline]
    fn clear(&self) {
        // SAFETY: We're restricting access to the context, so that this is
        // safe.
        unsafe {
            (*self.error.get()) = None;
        }
    }

    #[inline]
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        self.alloc.alloc()
    }

    #[inline]
    fn collect_string<T>(&self, value: &T) -> Result<Self::BufString<'_>, Self::Error>
    where
        T: ?Sized + fmt::Display,
    {
        buf::collect_string(self, value)
    }

    #[inline]
    fn custom<T>(&self, error: T) -> ErrorMarker
    where
        T: 'static + Send + Sync + StdError,
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

impl<A, M, E> Default for Capture<A, M, E>
where
    A: Default,
{
    #[inline]
    fn default() -> Self {
        Self::new(A::default())
    }
}
