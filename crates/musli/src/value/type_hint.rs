use core::fmt;

#[cfg(feature = "alloc")]
use musli_core::de::SizeHint;

/// A type hint.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub(crate) enum TypeHint {
    /// A unit type or an empty value.
    Unit,
    /// A boolean type.
    Bool,
    /// A character type.
    Char,
    /// The type as a number.
    Number(NumberHint),
    /// A byte array.
    #[cfg(feature = "alloc")]
    Bytes(SizeHint),
    /// A string with the given length.
    #[cfg(feature = "alloc")]
    String(SizeHint),
    /// A sequence with a length hint.
    #[cfg(feature = "alloc")]
    Sequence(SizeHint),
    /// A map with a length hint.
    #[cfg(feature = "alloc")]
    Map(SizeHint),
    /// A variant.
    #[cfg(feature = "alloc")]
    Variant,
    /// An optional value.
    #[cfg(feature = "alloc")]
    Option,
}

impl fmt::Display for TypeHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeHint::Unit => write!(f, "unit"),
            TypeHint::Bool => write!(f, "bool"),
            TypeHint::Char => write!(f, "char"),
            TypeHint::Number(number) => number.fmt(f),
            #[cfg(feature = "alloc")]
            TypeHint::Bytes(size) => write!(f, "bytes with {size}"),
            #[cfg(feature = "alloc")]
            TypeHint::String(size) => write!(f, "string with {size}"),
            #[cfg(feature = "alloc")]
            TypeHint::Sequence(size) => write!(f, "sequence with {size}"),
            #[cfg(feature = "alloc")]
            TypeHint::Map(size) => write!(f, "map with {size}"),
            #[cfg(feature = "alloc")]
            TypeHint::Variant => write!(f, "variant"),
            #[cfg(feature = "alloc")]
            TypeHint::Option => write!(f, "option"),
        }
    }
}

/// A number hint.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub(crate) enum NumberHint {
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
    /// A [usize]-typed value.
    Usize,
    /// A [isize]-typed value.
    Isize,
    /// A 32-bit float.
    F32,
    /// A 64-bit float.
    F64,
}

impl fmt::Display for NumberHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberHint::U8 => write!(f, "u8"),
            NumberHint::U16 => write!(f, "u16"),
            NumberHint::U32 => write!(f, "u32"),
            NumberHint::U64 => write!(f, "u64"),
            NumberHint::U128 => write!(f, "u128"),
            NumberHint::I8 => write!(f, "i8"),
            NumberHint::I16 => write!(f, "i16"),
            NumberHint::I32 => write!(f, "i32"),
            NumberHint::I64 => write!(f, "i64"),
            NumberHint::I128 => write!(f, "i128"),
            NumberHint::Usize => write!(f, "usize"),
            NumberHint::Isize => write!(f, "isize"),
            NumberHint::F32 => write!(f, "f32"),
            NumberHint::F64 => write!(f, "f64"),
        }
    }
}
