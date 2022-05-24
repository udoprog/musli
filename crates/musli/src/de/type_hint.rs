use core::fmt;

/// A number hint.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum NumberHint {
    /// Any numerical type of unknown kind.
    Any,
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
            NumberHint::Any => write!(f, "any"),
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

/// A length hint.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum LengthHint {
    /// The length isn't known.
    Any,
    /// The length is exactly known.
    Exact(usize),
}

impl LengthHint {
    /// Coerce into a size hint.
    pub fn size_hint(self) -> usize {
        match self {
            LengthHint::Any => 0,
            LengthHint::Exact(len) => len,
        }
    }
}

impl Default for LengthHint {
    fn default() -> Self {
        LengthHint::Any
    }
}

/// A type hint.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum TypeHint {
    /// Any type.
    Any,
    /// A unit type or an empty value.
    Unit,
    /// A boolean type.
    Bool,
    /// A character type.
    Char,
    /// The type as a number.
    Number(NumberHint),
    /// A byte array.
    Bytes(LengthHint),
    /// A string with the given length.
    String(LengthHint),
    /// A sequence with a length hint.
    Sequence(LengthHint),
    /// A map with a length hint.
    Map(LengthHint),
    /// A variant.
    Variant,
    /// An optional value.
    Option,
}

impl fmt::Display for TypeHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeHint::Any => write!(f, "any"),
            TypeHint::Unit => write!(f, "unit"),
            TypeHint::Bool => write!(f, "bool"),
            TypeHint::Char => write!(f, "char"),
            TypeHint::Number(_) => write!(f, "number"),
            TypeHint::Bytes(_) => write!(f, "bytes"),
            TypeHint::String(_) => write!(f, "string"),
            TypeHint::Sequence(_) => write!(f, "sequence"),
            TypeHint::Map(_) => write!(f, "map"),
            TypeHint::Variant => write!(f, "variant"),
            TypeHint::Option => write!(f, "option"),
        }
    }
}
