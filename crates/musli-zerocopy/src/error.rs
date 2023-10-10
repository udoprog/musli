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
    AlignmentMismatch {
        range: Range<usize>,
        align: usize,
    },
    LayoutMismatch {
        range: Range<usize>,
        layout: Layout,
    },
    OutOfRangeBounds {
        range: Range<usize>,
        len: usize,
    },
    NonZeroZeroed {
        range: Range<usize>,
    },
    BufferUnderflow {
        range: Range<usize>,
        expected: usize,
    },
    #[cfg(feature = "alloc")]
    BufferOverflow {
        offset: usize,
        capacity: usize,
    },
    IndexOutOfBounds {
        index: usize,
        len: usize,
    },
    LayoutError {
        error: LayoutError,
    },
    Utf8Error {
        error: Utf8Error,
    },
    #[cfg(feature = "alloc")]
    FailedPhf,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::AlignmentMismatch { range, align } => {
                write!(
                    f,
                    "Alignment mismatch, expected alignment {align} for range {range:?}"
                )
            }
            ErrorKind::LayoutMismatch { range, layout } => {
                write!(
                    f,
                    "Layout mismatch, expected {layout:?} for range {range:?}"
                )
            }
            ErrorKind::OutOfRangeBounds { range, len } => {
                write!(f, "Range {range:?} out of bound 0-{len}")
            }
            ErrorKind::NonZeroZeroed { range } => {
                write!(f, "Expected non-zero range at {range:?}")
            }
            ErrorKind::BufferUnderflow { range, expected } => {
                let len = range.len();

                write!(
                    f,
                    "Expected end of buffer at {expected} in range {range:?} but was {len}"
                )
            }
            #[cfg(feature = "alloc")]
            ErrorKind::BufferOverflow { offset, capacity } => {
                write!(
                    f,
                    "Offset {offset} is not within the allocated buffer 0-{capacity}"
                )
            }
            ErrorKind::IndexOutOfBounds { index, len } => {
                write!(f, "Index {index} out of bound 0-{len}")
            }
            ErrorKind::LayoutError { error } => error.fmt(f),
            ErrorKind::Utf8Error { error } => error.fmt(f),
            #[cfg(feature = "alloc")]
            ErrorKind::FailedPhf => {
                write!(f, "Failed to construct perfect hash for map")
            }
        }
    }
}
