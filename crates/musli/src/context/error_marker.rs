use core::fmt;

/// Indicates that an error occurred during encoding. This is a placeholder
/// error that can be used by context implementations and is a ZST.
///
/// Error details are expected to be reported to the corresponding [`Context`].
///
/// [`Context`]: crate::Context
#[derive(Debug)]
#[non_exhaustive]
pub struct ErrorMarker;

impl fmt::Display for ErrorMarker {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error during encoding or decoding (see context)")
    }
}

impl core::error::Error for ErrorMarker {}

impl<A> crate::context::ContextError<A> for ErrorMarker
where
    A: crate::Allocator,
{
    #[inline]
    fn custom<T>(_: A, _: T) -> Self
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        ErrorMarker
    }

    #[inline]
    fn message<T>(_: A, _: T) -> Self
    where
        T: fmt::Display,
    {
        ErrorMarker
    }
}
