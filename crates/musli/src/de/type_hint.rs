/// A number hint.
#[derive(Debug, Clone, Copy)]
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

/// A length hint.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum LengthHint {
    /// The length isn't known.
    Any,
    /// The length is exactly known.
    Exact(usize),
}

/// A type hint.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum TypeHint {
    /// Any type.
    Any,
    /// A unit type.
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
    /// A sequence.
    Sequence,
    /// A tuple of a statically known length.
    Tuple(usize),
    /// A map.
    Map(LengthHint),
    /// A struct with an unknown length.
    Struct(LengthHint),
    /// A tuple struct with an unknown length.
    TupleStruct(LengthHint),
    /// A unit struct.
    UnitStruct,
    /// A variant.
    Variant,
}
