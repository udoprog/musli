use core::cell::UnsafeCell;
use core::error::Error;
use core::fmt;
use core::marker::PhantomData;

use super::{ContextError, ErrorMarker};

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::NoCapture {}
    impl<E, A> Sealed for super::CaptureError<E, A> {}
    impl<E, A> Sealed for super::SameError<E, A> {}
}

/// The trait governing how error capture is implemented.
///
/// See [`DefaultContext::with_capture`] or [`DefaultContext::with_same`] for
/// more information.
///
/// [`DefaultContext::with_capture`]: super::DefaultContext::with_capture
/// [`DefaultContext::with_same`]: super::DefaultContext::with_same
pub trait Capture<A>
where
    Self: self::sealed::Sealed,
{
    #[doc(hidden)]
    type Error;

    #[doc(hidden)]
    fn clear(&self);

    #[doc(hidden)]
    fn message<T>(&self, alloc: A, message: T) -> Self::Error
    where
        T: fmt::Display;

    #[doc(hidden)]
    fn custom<T>(&self, alloc: A, error: T) -> Self::Error
    where
        T: 'static + Send + Sync + Error;
}

/// Disable error capture.
#[non_exhaustive]
pub struct NoCapture;

impl<A> Capture<A> for NoCapture {
    type Error = ErrorMarker;

    #[inline]
    fn clear(&self) {}

    #[inline]
    fn message<T>(&self, alloc: A, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        _ = alloc;
        _ = message;
        ErrorMarker
    }

    #[inline]
    fn custom<T>(&self, alloc: A, error: T) -> Self::Error
    where
        T: 'static + Send + Sync + Error,
    {
        _ = alloc;
        _ = error;
        ErrorMarker
    }
}

/// Capture an error of the specified type.
///
/// See [`DefaultContext::with_same`] for more information.
///
/// [`DefaultContext::with_same`]: super::DefaultContext::with_same
pub struct SameError<E, A> {
    _marker: PhantomData<(E, A)>,
}

impl<E, A> SameError<E, A> {
    #[inline]
    pub(super) fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<E, A> Capture<A> for SameError<E, A>
where
    E: ContextError<A>,
{
    type Error = E;

    #[inline]
    fn clear(&self) {}

    #[inline]
    fn message<T>(&self, alloc: A, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        E::message(alloc, message)
    }

    #[inline]
    fn custom<T>(&self, alloc: A, error: T) -> Self::Error
    where
        T: 'static + Send + Sync + Error,
    {
        E::custom(alloc, error)
    }
}

/// Capture an error of the specified type.
///
/// See [`DefaultContext::with_capture`] for more information.
///
/// [`DefaultContext::with_capture`]: super::DefaultContext::with_capture
pub struct CaptureError<E, A> {
    error: UnsafeCell<Option<E>>,
    _marker: PhantomData<A>,
}

impl<E, A> CaptureError<E, A> {
    #[inline]
    pub(super) fn new() -> Self {
        Self {
            error: UnsafeCell::new(None),
            _marker: PhantomData,
        }
    }
}

impl<E, A> CaptureError<E, A> {
    #[inline]
    pub(super) fn unwrap(&self) -> E {
        // SAFETY: We're restricting access to the context, so that this is
        // safe.
        unsafe {
            match (*self.error.get()).take() {
                Some(error) => error,
                None => panic!("no error captured"),
            }
        }
    }

    #[inline]
    pub(super) fn result(&self) -> Result<(), E> {
        // SAFETY: We're restricting access to the context, so that this is
        // safe.
        unsafe {
            match (*self.error.get()).take() {
                Some(error) => Err(error),
                None => Ok(()),
            }
        }
    }
}

impl<E, A> Capture<A> for CaptureError<E, A>
where
    E: ContextError<A>,
{
    type Error = ErrorMarker;

    #[inline]
    fn clear(&self) {
        // SAFETY: We're restricting access to the context, so that this is
        // safe.
        unsafe {
            (*self.error.get()) = None;
        }
    }

    #[inline]
    fn message<T>(&self, alloc: A, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        // SAFETY: We're restricting access to the context, so that this is
        // safe.
        unsafe {
            (*self.error.get()) = Some(E::message(alloc, message));
        }

        ErrorMarker
    }

    #[inline]
    fn custom<T>(&self, alloc: A, error: T) -> Self::Error
    where
        T: 'static + Send + Sync + Error,
    {
        // SAFETY: We're restricting access to the context, so that this is
        // safe.
        unsafe {
            (*self.error.get()) = Some(E::custom(alloc, error));
        }

        ErrorMarker
    }
}
