//! Helper types to set up a basic Müsli [`Context`].

mod access;
#[cfg(feature = "alloc")]
mod alloc_context;
mod error_marker;
mod no_std_context;
mod rich_error;
#[doc(inline)]
pub use self::error_marker::ErrorMarker;
mod error;
pub use self::error::Error;

use core::cell::{Cell, UnsafeCell};
use core::fmt;
use core::marker::PhantomData;

use musli::mode::DefaultMode;
use musli::{Allocator, Context};

#[cfg(feature = "alloc")]
pub use self::alloc_context::AllocContext;

pub use self::no_std_context::NoStdContext;

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

impl<A> Same<A, DefaultMode, ErrorMarker> {
    /// Construct a new `Same` capturing context.
    #[inline]
    pub fn marker(alloc: A) -> Self {
        Self::new(alloc)
    }
}

impl<A, M, E> Default for Same<A, M, E>
where
    A: Default,
{
    #[inline(always)]
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
    type Input = E;
    type Error = E;
    type Mark = ();
    type Buf<'this> = A::Buf<'this> where Self: 'this;

    #[inline(always)]
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        self.alloc.alloc()
    }

    #[inline(always)]
    fn report<T>(&self, error: T) -> Self::Error
    where
        E: From<T>,
    {
        E::from(error)
    }

    #[inline(always)]
    fn custom<T>(&self, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        E::custom(message)
    }

    #[inline(always)]
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
    #[inline(always)]
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

impl<A> Ignore<A, DefaultMode, ErrorMarker> {
    /// Construct a new ignoring context which collects an error marker.
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
            return E::custom("error");
        }

        panic!("did not error")
    }
}

impl<A, M, E: 'static> Context for Ignore<A, M, E>
where
    A: Allocator,
{
    type Mode = M;
    type Input = E;
    type Error = ErrorMarker;
    type Mark = ();
    type Buf<'this> = A::Buf<'this> where Self: 'this;

    #[inline(always)]
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        self.alloc.alloc()
    }

    #[inline(always)]
    fn report<T>(&self, _: T) -> ErrorMarker
    where
        E: From<T>,
    {
        self.error.set(true);
        ErrorMarker
    }

    #[inline(always)]
    fn custom<T>(&self, _: T) -> ErrorMarker
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        self.error.set(true);
        ErrorMarker
    }

    #[inline(always)]
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
    type Input = E;
    type Error = ErrorMarker;
    type Mark = ();
    type Buf<'this> = A::Buf<'this> where Self: 'this;

    #[inline(always)]
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        self.alloc.alloc()
    }

    #[inline(always)]
    fn report<T>(&self, error: T) -> ErrorMarker
    where
        E: From<T>,
    {
        // SAFETY: We're restricting access to the context, so that this is
        // safe.
        unsafe {
            self.error.get().replace(Some(E::from(error)));
        }

        ErrorMarker
    }

    #[inline(always)]
    fn custom<T>(&self, error: T) -> ErrorMarker
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        // SAFETY: We're restricting access to the context, so that this is
        // safe.
        unsafe {
            self.error.get().replace(Some(E::custom(error)));
        }

        ErrorMarker
    }

    #[inline(always)]
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
    #[inline(always)]
    fn default() -> Self {
        Self::new(A::default())
    }
}
