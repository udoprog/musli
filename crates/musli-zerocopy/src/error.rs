use core::alloc::Layout;
use core::any::type_name;
use core::fmt;
use core::ops::Range;
use core::str::Utf8Error;

/// Indicates the representation that was tried to coerce into an enum.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[non_exhaustive]
pub(crate) enum EnumRepr {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Usize(usize),
    Isize(isize),
}

impl fmt::Display for EnumRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnumRepr::U8(value) => write!(f, "{value}u8"),
            EnumRepr::U16(value) => write!(f, "{value}u16"),
            EnumRepr::U32(value) => write!(f, "{value}u32"),
            EnumRepr::U64(value) => write!(f, "{value}u64"),
            EnumRepr::U128(value) => write!(f, "{value}u128"),
            EnumRepr::I8(value) => write!(f, "{value}i8"),
            EnumRepr::I16(value) => write!(f, "{value}i16"),
            EnumRepr::I32(value) => write!(f, "{value}i32"),
            EnumRepr::I64(value) => write!(f, "{value}i64"),
            EnumRepr::I128(value) => write!(f, "{value}i128"),
            EnumRepr::Usize(value) => write!(f, "{value}usize"),
            EnumRepr::Isize(value) => write!(f, "{value}isize"),
        }
    }
}

macro_rules! illegal_enum {
    ($name:ident, $repr:ident, $ty:ty) => {
        /// Private helper function to indicate that an illegal enum
        /// representation has been encountered.
        #[doc(hidden)]
        pub fn $name<T>(repr: $ty) -> Self {
            Self::new(ErrorKind::IllegalEnumRepr {
                name: type_name::<T>(),
                repr: EnumRepr::$repr(repr),
            })
        }
    };
}

/// Müsli's zero copy error type.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    #[inline]
    pub(crate) const fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }

    illegal_enum!(__illegal_enum_u8, U8, u8);
    illegal_enum!(__illegal_enum_u16, U16, u16);
    illegal_enum!(__illegal_enum_u32, U32, u32);
    illegal_enum!(__illegal_enum_u64, U64, u64);
    illegal_enum!(__illegal_enum_u128, U128, u128);
    illegal_enum!(__illegal_enum_i8, I8, i8);
    illegal_enum!(__illegal_enum_i16, I16, i16);
    illegal_enum!(__illegal_enum_i32, I32, i32);
    illegal_enum!(__illegal_enum_i64, I64, i64);
    illegal_enum!(__illegal_enum_i128, I128, i128);
    illegal_enum!(__illegal_enum_usize, Usize, usize);
    illegal_enum!(__illegal_enum_isize, Isize, isize);
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
#[cfg_attr(test, derive(PartialEq))]
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
    IllegalEnumRepr {
        name: &'static str,
        repr: EnumRepr,
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
            ErrorKind::IllegalEnumRepr { name, repr } => {
                write!(f, "Illegal enum representation {repr} for enum {name}")
            }
            ErrorKind::Utf8Error { error } => error.fmt(f),
            #[cfg(feature = "alloc")]
            ErrorKind::FailedPhf => {
                write!(f, "Failed to construct perfect hash for map")
            }
        }
    }
}
