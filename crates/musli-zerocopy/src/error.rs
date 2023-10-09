use core::alloc::{Layout, LayoutError};
use core::fmt;
use core::ops::Range;
use core::str::Utf8Error;

/// Müsli's zero copy error type.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    #[inline]
    pub(crate) const fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Utf8Error { error } => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum ErrorKind {
    BadAlignment { ptr: usize, align: usize },
    LayoutMismatch { layout: Layout, buf: Range<usize> },
    OutOfBounds { len: usize },
    OutOfStartBound { start: usize, len: usize },
    IndexOutOfBounds { index: usize, len: usize },
    NonZeroZeroed { range: Range<usize> },
    FailedPhf,
    LayoutError { error: LayoutError },
    Utf8Error { error: Utf8Error },
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::BadAlignment { ptr, align } => {
                write!(f, "Bad alignment {align} for pointer {ptr}")
            }
            ErrorKind::LayoutMismatch { layout, buf } => {
                write!(
                    f,
                    "Layout mismatch, expected {layout:?}, but buffer is 0x{:x}-0x{:x}",
                    buf.start, buf.end
                )
            }
            ErrorKind::OutOfBounds { len } => {
                write!(f, "Out of bounds 0-{len}")
            }
            ErrorKind::OutOfStartBound { start, len } => {
                write!(f, "Out of start bound {start}, expected 0-{len}")
            }
            ErrorKind::IndexOutOfBounds { index, len } => {
                write!(f, "Index {index} out of bounds, expected 0-{len}")
            }
            ErrorKind::NonZeroZeroed { range } => {
                write!(f, "Expected non-zero range at {range:?}")
            }
            ErrorKind::FailedPhf => {
                write!(f, "Failed to construct perfect hash for map")
            }
            ErrorKind::LayoutError { error } => error.fmt(f),
            ErrorKind::Utf8Error { error } => error.fmt(f),
        }
    }
}
