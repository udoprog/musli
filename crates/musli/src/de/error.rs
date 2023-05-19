use core::fmt;

/// Indicates that an error occurred during decoding.
///
/// Error details are expected to be reported to the corresponding [`Context`].
#[derive(Debug)]
pub struct Error;

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error during decoding (see context)")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
