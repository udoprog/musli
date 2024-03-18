use core::alloc::Layout;
use core::any::type_name;
use core::fmt;
use core::ops::{Range, RangeFrom};
use core::str::Utf8Error;

mod sealed {
    pub trait Sealed {}
    impl Sealed for () {}
}

/// Helper trait to convert any type into a type-erased [`Repr`] used for diagnostics.
pub trait IntoRepr: self::sealed::Sealed {
    #[doc(hidden)]
    fn into_repr(self) -> Repr;
}

impl IntoRepr for () {
    fn into_repr(self) -> Repr {
        Repr {
            kind: ReprKind::Unit,
        }
    }
}

macro_rules! impl_into_repr {
    ($($ty:ty, $repr:ident),* $(,)?) => {
        $(
        impl self::sealed::Sealed for $ty {
        }

        impl IntoRepr for $ty {
            fn into_repr(self) -> Repr {
                Repr {
                    kind: ReprKind::$repr(self),
                }
            }
        }
        )*
    }
}

impl_into_repr! {
    u8, U8,
    u16, U16,
    u32, U32,
    u64, U64,
    u128, U128,
    usize, Usize,
    i8, I8,
    i16, I16,
    i32, I32,
    i64, I64,
    i128, I128,
    isize, Isize,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
enum ReprKind {
    Unit,
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

/// Indicates the representation that was tried to coerce into an enum.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[non_exhaustive]
pub struct Repr {
    kind: ReprKind,
}

impl fmt::Display for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ReprKind::Unit => write!(f, "()"),
            ReprKind::U8(value) => write!(f, "{value}u8"),
            ReprKind::U16(value) => write!(f, "{value}u16"),
            ReprKind::U32(value) => write!(f, "{value}u32"),
            ReprKind::U64(value) => write!(f, "{value}u64"),
            ReprKind::U128(value) => write!(f, "{value}u128"),
            ReprKind::I8(value) => write!(f, "{value}i8"),
            ReprKind::I16(value) => write!(f, "{value}i16"),
            ReprKind::I32(value) => write!(f, "{value}i32"),
            ReprKind::I64(value) => write!(f, "{value}i64"),
            ReprKind::I128(value) => write!(f, "{value}i128"),
            ReprKind::Usize(value) => write!(f, "{value}usize"),
            ReprKind::Isize(value) => write!(f, "{value}isize"),
        }
    }
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

    #[inline(always)]
    #[doc(hidden)]
    pub fn __illegal_enum_discriminant<T>(discriminant: impl IntoRepr) -> Self {
        Self::new(ErrorKind::IllegalDiscriminant {
            name: type_name::<T>(),
            discriminant: discriminant.into_repr(),
        })
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
#[cfg_attr(test, derive(PartialEq))]
#[non_exhaustive]
pub(crate) enum ErrorKind {
    InvalidOffset {
        ty: &'static str,
    },
    InvalidOffsetRange {
        offset: Repr,
        max: Repr,
    },
    InvalidMetadata {
        ty: &'static str,
        packed: &'static str,
    },
    InvalidMetadataRange {
        metadata: Repr,
        max: Repr,
    },
    LengthOverflow {
        len: usize,
        size: usize,
    },
    AlignmentRangeMismatch {
        addr: usize,
        range: Range<usize>,
        align: usize,
    },
    AlignmentRangeFromMismatch {
        range: RangeFrom<usize>,
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
    OutOfRangeFromBounds {
        range: RangeFrom<usize>,
        len: usize,
    },
    NonZeroZeroed {
        range: Range<usize>,
    },
    IndexOutOfBounds {
        index: usize,
        len: usize,
    },
    ControlRangeOutOfBounds {
        range: Range<usize>,
        len: usize,
    },
    StrideOutOfBounds {
        index: usize,
        len: usize,
    },
    IllegalDiscriminant {
        name: &'static str,
        discriminant: Repr,
    },
    IllegalChar {
        repr: u32,
    },
    IllegalBool {
        repr: u8,
    },
    Utf8Error {
        error: Utf8Error,
    },
    Underflow {
        at: usize,
        len: usize,
    },
    Overflow {
        at: usize,
        len: usize,
    },
    StackOverflow {
        capacity: usize,
    },
    #[cfg(feature = "alloc")]
    CapacityError,
    #[cfg(feature = "alloc")]
    FailedPhf,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::InvalidOffset { ty } => {
                write!(
                    f,
                    "Offset `{ty}` cannot be byte-ordered since it would not inhabit valid types",
                )
            }
            ErrorKind::InvalidOffsetRange { offset, max } => {
                write!(f, "Offset {offset} not in legal range 0-{max}",)
            }
            ErrorKind::InvalidMetadata { ty, packed } => {
                write!(
                    f,
                    "Metadata `{ty}` once packed as `{packed}` cannot be byte-ordered since it would not inhabit valid types",
                )
            }
            ErrorKind::InvalidMetadataRange { metadata, max } => {
                write!(f, "Metadata {metadata} not in legal range 0-{max}")
            }
            ErrorKind::LengthOverflow { len, size } => {
                write!(
                    f,
                    "Length overflowed when trying to take {len} elements of size {size}"
                )
            }
            ErrorKind::AlignmentRangeMismatch { addr, range, align } => {
                write!(
                    f,
                    "Alignment mismatch, expected alignment {align} for range {range:?} at address {addr:x}"
                )
            }
            ErrorKind::AlignmentRangeFromMismatch { range, align } => {
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
            ErrorKind::OutOfRangeFromBounds { range, len } => {
                write!(f, "Range {range:?} out of bound 0-{len}")
            }
            ErrorKind::NonZeroZeroed { range } => {
                write!(f, "Expected non-zero range at {range:?}")
            }
            ErrorKind::IndexOutOfBounds { index, len } => {
                write!(f, "Index {index} out of bound 0-{len}")
            }
            ErrorKind::ControlRangeOutOfBounds { range, len } => {
                write!(f, "Control range {range:?} out of bound 0-{len}")
            }
            ErrorKind::StrideOutOfBounds { index, len } => {
                write!(f, "Stride at index {index} out of bound 0-{len}")
            }
            ErrorKind::IllegalDiscriminant { name, discriminant } => {
                write!(f, "Illegal discriminant {discriminant} for enum {name}")
            }
            ErrorKind::IllegalChar { repr } => {
                write!(f, "Illegal char representation {repr}")
            }
            ErrorKind::IllegalBool { repr } => {
                write!(f, "Illegal bool representation {repr}")
            }
            ErrorKind::Underflow { at, len } => {
                write!(f, "Arithmetic underflow calculating {at} - {len}")
            }
            ErrorKind::Overflow { at, len } => {
                write!(f, "Arithmetic overflow calculating {at} + {len}")
            }
            ErrorKind::StackOverflow { capacity } => {
                write!(f, "Stack with capacity {capacity} overflowed")
            }
            ErrorKind::Utf8Error { error } => error.fmt(f),
            #[cfg(feature = "alloc")]
            ErrorKind::CapacityError => {
                write!(f, "Out of capacity")
            }
            #[cfg(feature = "alloc")]
            ErrorKind::FailedPhf => {
                write!(f, "Failed to construct perfect hash for map")
            }
        }
    }
}
