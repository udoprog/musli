//! Helper types to set up a basic Müsli [`Context`].

mod access;
#[cfg(feature = "alloc")]
mod alloc_context;
mod no_std_context;
mod rich_error;

use core::cell::{Cell, UnsafeCell};
use core::fmt;
use core::marker::PhantomData;

use musli::context::Error;
use musli::Context;

use crate::allocator::Allocator;

#[cfg(feature = "alloc")]
pub use self::alloc_context::AllocContext;

pub use self::no_std_context::NoStdContext;

pub use self::rich_error::RichError;

/// A simple non-diagnostical capturing context which simply emits the original
/// error.
///
/// Using this should result in code which essentially just uses the emitted
/// error type directly.
pub struct Same<A, E> {
    alloc: A,
    _marker: PhantomData<E>,
}

impl<A, E> Same<A, E> {
    /// Construct a new `Same` capturing context.
    pub fn new(alloc: A) -> Self {
        Self {
            alloc,
            _marker: PhantomData,
        }
    }
}

impl<A, E> Default for Same<A, E>
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

impl<A, E> Context for Same<A, E>
where
    A: Allocator,
    E: musli::error::Error,
{
    type Input = E;
    type Error = E;
    type Mark = ();
    type Buf = A::Buf;

    #[inline(always)]
    fn alloc(&self) -> Self::Buf {
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
pub struct Ignore<A, E> {
    alloc: A,
    error: Cell<bool>,
    _marker: PhantomData<E>,
}

impl<A, E> Default for Ignore<A, E>
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

impl<A, E> Ignore<A, E>
where
    A: Allocator,
{
    /// Construct a new ignoring context.
    pub fn new(alloc: A) -> Self {
        Self {
            alloc,
            error: Cell::new(false),
            _marker: PhantomData,
        }
    }
}

impl<A, E> Ignore<A, E>
where
    A: Allocator,
    E: musli::error::Error,
{
    /// Construct an error or panic.
    pub fn unwrap(self) -> E {
        if self.error.get() {
            return E::custom("error");
        }

        panic!("did not error")
    }
}

impl<A, E> Context for Ignore<A, E>
where
    A: Allocator,
{
    type Input = E;
    type Error = Error;
    type Mark = ();
    type Buf = A::Buf;

    #[inline(always)]
    fn alloc(&self) -> Self::Buf {
        self.alloc.alloc()
    }

    #[inline(always)]
    fn report<T>(&self, _: T) -> Error
    where
        E: From<T>,
    {
        self.error.set(true);
        Error
    }

    #[inline(always)]
    fn custom<T>(&self, _: T) -> Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        self.error.set(true);
        Error
    }

    #[inline(always)]
    fn message<T>(&self, _: T) -> Error
    where
        T: fmt::Display,
    {
        self.error.set(true);
        Error
    }
}

/// A simple non-diagnostical capturing context.
pub struct Capture<A, E> {
    alloc: A,
    error: UnsafeCell<Option<E>>,
}

impl<A, E> Capture<A, E>
where
    E: musli::error::Error,
{
    /// Construct a new capturing allocator.
    pub fn new(alloc: A) -> Self {
        Self {
            alloc,
            error: UnsafeCell::new(None),
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

impl<A, E> Context for Capture<A, E>
where
    A: Allocator,
    E: musli::error::Error,
{
    type Input = E;
    type Error = Error;
    type Mark = ();
    type Buf = A::Buf;

    #[inline(always)]
    fn alloc(&self) -> Self::Buf {
        self.alloc.alloc()
    }

    #[inline(always)]
    fn report<T>(&self, error: T) -> Error
    where
        E: From<T>,
    {
        // SAFETY: We're restricting access to the context, so that this is
        // safe.
        unsafe {
            self.error.get().replace(Some(E::from(error)));
        }

        Error
    }

    #[inline(always)]
    fn custom<T>(&self, error: T) -> Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        // SAFETY: We're restricting access to the context, so that this is
        // safe.
        unsafe {
            self.error.get().replace(Some(E::custom(error)));
        }
        Error
    }

    #[inline(always)]
    fn message<T>(&self, message: T) -> Error
    where
        T: fmt::Display,
    {
        // SAFETY: We're restricting access to the context, so that this is
        // safe.
        unsafe {
            self.error.get().replace(Some(E::message(message)));
        }

        Error
    }
}

impl<A, E> Default for Capture<A, E>
where
    A: Default,
{
    #[inline(always)]
    fn default() -> Self {
        Self {
            alloc: A::default(),
            error: UnsafeCell::new(None),
        }
    }
}
