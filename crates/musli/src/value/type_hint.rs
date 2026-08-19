use core::fmt;

use crate::de::SizeHint;

/// A type hint.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub(crate) enum TypeHint {
    /// A unit type or an empty value.
    Empty,
    /// A boolean type.
    Bool,
    /// A character type.
    Char,
    /// The type as a number.
    Number(NumberHint),
    /// A byte array.
    Bytes(SizeHint),
    /// A string with the given length.
    String(SizeHint),
    /// A sequence with a length hint.
    Sequence(SizeHint),
    /// A map with a length hint.
    Map(SizeHint),
    /// A variant.
    Variant,
    /// An optional value.
    Option,
}

impl fmt::Display for TypeHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeHint::Empty => write!(f, "empty"),
            TypeHint::Bool => write!(f, "bool"),
            TypeHint::Char => write!(f, "char"),
            TypeHint::Number(number) => number.fmt(f),
            TypeHint::Bytes(size) => write!(f, "bytes with length {size}"),
            TypeHint::String(size) => write!(f, "string with length {size}"),
            TypeHint::Sequence(size) => write!(f, "sequence with length {size}"),
            TypeHint::Map(size) => write!(f, "map with length {size}"),
            TypeHint::Variant => write!(f, "variant"),
            TypeHint::Option => write!(f, "option"),
        }
    }
}

/// A number hint.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub(crate) enum NumberHint {
    /// An integer of the given kind.
    Integer(IntegerKind),
    /// A float of the given kind.
    Float(FloatKind),
}

impl fmt::Display for NumberHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberHint::Integer(kind) => kind.fmt(f),
            NumberHint::Float(kind) => kind.fmt(f),
        }
    }
}

/// The kind of an integer, which identifies its exact representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub(crate) enum IntegerKind {
    /// An unsigned 8-bit integer.
    U8,
    /// An unsigned 16-bit integer.
    U16,
    /// An unsigned 32-bit integer.
    U32,
    /// An unsigned 64-bit integer.
    U64,
    /// An unsigned 128-bit integer.
    U128,
    /// A signed 8-bit integer.
    I8,
    /// A signed 16-bit integer.
    I16,
    /// A signed 32-bit integer.
    I32,
    /// A signed 64-bit integer.
    I64,
    /// A signed 128-bit integer.
    I128,
}

impl IntegerKind {
    /// The kind a [usize] is represented as.
    #[cfg(target_pointer_width = "32")]
    pub(crate) const USIZE: Self = IntegerKind::U32;
    /// The kind a [usize] is represented as.
    #[cfg(target_pointer_width = "64")]
    pub(crate) const USIZE: Self = IntegerKind::U64;
    /// The kind an [isize] is represented as.
    #[cfg(target_pointer_width = "32")]
    pub(crate) const ISIZE: Self = IntegerKind::I32;
    /// The kind an [isize] is represented as.
    #[cfg(target_pointer_width = "64")]
    pub(crate) const ISIZE: Self = IntegerKind::I64;

    /// Test if the integer kind is signed.
    #[inline]
    pub(crate) const fn is_signed(self) -> bool {
        matches!(
            self,
            IntegerKind::I8
                | IntegerKind::I16
                | IntegerKind::I32
                | IntegerKind::I64
                | IntegerKind::I128
        )
    }
}

impl fmt::Display for IntegerKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntegerKind::U8 => write!(f, "u8"),
            IntegerKind::U16 => write!(f, "u16"),
            IntegerKind::U32 => write!(f, "u32"),
            IntegerKind::U64 => write!(f, "u64"),
            IntegerKind::U128 => write!(f, "u128"),
            IntegerKind::I8 => write!(f, "i8"),
            IntegerKind::I16 => write!(f, "i16"),
            IntegerKind::I32 => write!(f, "i32"),
            IntegerKind::I64 => write!(f, "i64"),
            IntegerKind::I128 => write!(f, "i128"),
        }
    }
}

/// The kind of a float, which identifies its exact representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub(crate) enum FloatKind {
    /// A 32-bit float.
    F32,
    /// A 64-bit float.
    F64,
}

impl fmt::Display for FloatKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FloatKind::F32 => write!(f, "f32"),
            FloatKind::F64 => write!(f, "f64"),
        }
    }
}
