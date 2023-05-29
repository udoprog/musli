//! Helper types to set up a basic Müsli [`Context`].

#[cfg(feature = "alloc")]
mod alloc_context;
#[cfg(feature = "arrayvec")]
mod no_std_context;
#[cfg(any(feature = "alloc", feature = "arrayvec"))]
mod rich_error;

use core::fmt;
use core::marker::PhantomData;

use musli::context::Error;
use musli::Context;

#[cfg(feature = "alloc")]
pub use self::alloc_context::{AllocBuf, AllocContext};

#[cfg(feature = "arrayvec")]
pub use self::no_std_context::{NoStdBuf, NoStdContext};

#[cfg(any(feature = "alloc", feature = "arrayvec"))]
pub use self::rich_error::RichError;

/// A simple non-diagnostical capturing context which simply emits the original
/// error.
///
/// Using this should result in code which essentially just uses the emitted
/// error type directly.
pub struct Same<E>(PhantomData<E>);

impl<E> Default for Same<E> {
    #[inline(always)]
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<'buf, E> Context<'buf> for Same<E>
where
    E: musli::error::Error,
{
    type Input = E;
    type Error = E;
    type Mark = ();

    #[inline(always)]
    fn report<T>(&mut self, error: T) -> Self::Error
    where
        E: From<T>,
    {
        E::from(error)
    }

    #[inline(always)]
    fn custom<T>(&mut self, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        E::custom(message)
    }

    #[inline(always)]
    fn message<T>(&mut self, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        E::message(message)
    }
}

/// A simple non-diagnostical capturing context which ignores the error and
/// loses all information about it (except that it happened).
pub struct Ignore<E> {
    error: bool,
    _marker: PhantomData<E>,
}

impl<E> Default for Ignore<E> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            error: false,
            _marker: PhantomData,
        }
    }
}

impl<E> Ignore<E>
where
    E: musli::error::Error,
{
    /// Construct an error or panic.
    pub fn unwrap(self) -> E {
        if self.error {
            return E::custom("error");
        }

        panic!("did not error")
    }
}

impl<'buf, E> Context<'buf> for Ignore<E> {
    type Input = E;
    type Error = Error;
    type Mark = ();

    #[inline(always)]
    fn report<T>(&mut self, _: T) -> Error
    where
        E: From<T>,
    {
        self.error = true;
        Error
    }

    #[inline(always)]
    fn custom<T>(&mut self, _: T) -> Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        self.error = true;
        Error
    }

    #[inline(always)]
    fn message<T>(&mut self, _: T) -> Error
    where
        T: fmt::Display,
    {
        self.error = true;
        Error
    }
}

/// A simple non-diagnostical capturing context.
pub struct Capture<E> {
    error: Option<E>,
}

impl<E> Capture<E>
where
    E: musli::error::Error,
{
    /// Construct an error or panic.
    pub fn unwrap(self) -> E {
        if let Some(error) = self.error {
            return error;
        }

        panic!("no error captured")
    }
}

impl<'buf, E> Context<'buf> for Capture<E>
where
    E: musli::error::Error,
{
    type Input = E;
    type Error = Error;
    type Mark = ();

    #[inline(always)]
    fn report<T>(&mut self, error: T) -> Error
    where
        E: From<T>,
    {
        self.error = Some(E::from(error));
        Error
    }

    #[inline(always)]
    fn custom<T>(&mut self, error: T) -> Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        self.error = Some(E::custom(error));
        Error
    }

    #[inline(always)]
    fn message<T>(&mut self, message: T) -> Error
    where
        T: fmt::Display,
    {
        self.error = Some(E::message(message));
        Error
    }
}

impl<E> Default for Capture<E> {
    #[inline(always)]
    fn default() -> Self {
        Self { error: None }
    }
}
