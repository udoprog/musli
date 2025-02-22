use core::cell::UnsafeCell;
use core::error::Error;
use core::fmt;
use core::marker::PhantomData;

use super::{ContextError, ErrorMarker};

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Ignore {}
    impl<E> Sealed for super::Capture<E> {}
    impl<E> Sealed for super::Emit<E> {}
}

/// The trait governing how error capture is implemented.
///
/// See [`DefaultContext::with_capture`] or [`DefaultContext::with_error`] for
/// more information.
///
/// [`DefaultContext::with_capture`]: super::DefaultContext::with_capture
/// [`DefaultContext::with_error`]: super::DefaultContext::with_error
pub trait ErrorMode<A>: self::sealed::Sealed {
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
///
/// The error produced will be an [`ErrorMarker`] which is a zero-sized
/// placeholder type.
///
/// To capture an error, use [`with_capture::<E>`]. To produce an error see
/// [`with_error::<E>`].
///
/// This is the default behavior you get when calling [`new`] or [`new_in`].
///
/// [`with_capture::<E>`]: super::DefaultContext::with_capture
/// [`with_error::<E>`]: super::DefaultContext::with_error
///
/// [`new`]: super::new
/// [`new_in`]: super::new_in
#[non_exhaustive]
pub struct Ignore;

impl<A> ErrorMode<A> for Ignore {
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

/// Emit an error of the specified type `E`.
///
/// See [`DefaultContext::with_error`] for more information.
///
/// [`DefaultContext::with_error`]: super::DefaultContext::with_error
pub struct Emit<E> {
    _marker: PhantomData<E>,
}

impl<E> Emit<E> {
    #[inline]
    pub(super) fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<E, A> ErrorMode<A> for Emit<E>
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

/// Capture an error of the specified type `E`.
///
/// See [`DefaultContext::with_capture`] for more information.
///
/// [`DefaultContext::with_capture`]: super::DefaultContext::with_capture
pub struct Capture<E> {
    error: UnsafeCell<Option<E>>,
}

impl<E> Capture<E> {
    #[inline]
    pub(super) fn new() -> Self {
        Self {
            error: UnsafeCell::new(None),
        }
    }
}

impl<E> Capture<E> {
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

impl<E, A> ErrorMode<A> for Capture<E>
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
