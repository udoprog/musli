use core::fmt;

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
pub struct Error;

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error during encoding (see context)")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl crate::error::Error for Error {
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
