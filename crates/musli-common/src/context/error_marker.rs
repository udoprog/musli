use core::fmt;

use super::Error;

/// Indicates that an error occurred during encoding. This is a placeholder
/// error that can be used by context implementations and is a ZST.
///
/// Using it directly as a musli [`Error`] is not a good idea, since it discards
/// any diagnostics provided to it.
///
/// Error details are expected to be reported to the corresponding [`Context`].
///
/// [`Context`]: crate::context::Context
#[derive(Debug)]
#[non_exhaustive]
pub struct ErrorMarker;

impl fmt::Display for ErrorMarker {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error during encoding (see context)")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ErrorMarker {}

impl Error for ErrorMarker {
    #[inline(always)]
    fn custom<T>(_: T) -> Self
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        Self
    }

    #[inline(always)]
    fn message<T>(_: T) -> Self
    where
        T: fmt::Display,
    {
        Self
    }
}
