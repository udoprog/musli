use core::fmt;

use crate::Context;

#[cfg(feature = "alloc")]
use rust_alloc::boxed::Box;
#[cfg(feature = "alloc")]
use rust_alloc::format;

enum SerdeErrorKind<E> {
    Captured(E),
    #[cfg(not(feature = "alloc"))]
    Custom,
    #[cfg(feature = "alloc")]
    Custom(Box<str>),
}

/// The internal error type for serde operations.
pub(super) struct SerdeError<E> {
    kind: SerdeErrorKind<E>,
}

impl<E> From<E> for SerdeError<E> {
    #[inline]
    fn from(value: E) -> Self {
        SerdeError {
            kind: SerdeErrorKind::Captured(value),
        }
    }
}

impl<E> fmt::Debug for SerdeError<E> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            SerdeErrorKind::Captured(..) => write!(f, "Captured error in musli::serde"),
            #[cfg(feature = "alloc")]
            SerdeErrorKind::Custom(error) => error.fmt(f),
            #[cfg(not(feature = "alloc"))]
            SerdeErrorKind::Custom => write!(f, "Custom error in musli::serde"),
        }
    }
}

impl<E> fmt::Display for SerdeError<E> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            SerdeErrorKind::Captured(..) => write!(f, "Captured error in musli::serde"),
            #[cfg(feature = "alloc")]
            SerdeErrorKind::Custom(error) => error.fmt(f),
            #[cfg(not(feature = "alloc"))]
            SerdeErrorKind::Custom => write!(f, "Custom error in musli::serde"),
        }
    }
}

impl<E> core::error::Error for SerdeError<E> {}

impl<E> serde::ser::Error for SerdeError<E> {
    #[cfg(feature = "alloc")]
    #[inline]
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        SerdeError {
            kind: SerdeErrorKind::Custom(format!("{msg}").into()),
        }
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn custom<T>(_: T) -> Self
    where
        T: fmt::Display,
    {
        SerdeError {
            kind: SerdeErrorKind::Custom,
        }
    }
}

impl<E> serde::de::Error for SerdeError<E> {
    #[cfg(feature = "alloc")]
    #[inline]
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        SerdeError {
            kind: SerdeErrorKind::Custom(format!("{msg}").into()),
        }
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn custom<T>(_: T) -> Self
    where
        T: fmt::Display,
    {
        SerdeError {
            kind: SerdeErrorKind::Custom,
        }
    }
}

#[inline]
pub(super) fn err<C>(cx: C) -> impl FnOnce(SerdeError<C::Error>) -> C::Error + Copy
where
    C: Context,
{
    move |e| match e.kind {
        SerdeErrorKind::Captured(value) => value,
        #[cfg(not(feature = "alloc"))]
        SerdeErrorKind::Custom => cx.message("Custom error in musli::serde"),
        #[cfg(feature = "alloc")]
        SerdeErrorKind::Custom(value) => cx.message(value),
    }
}
