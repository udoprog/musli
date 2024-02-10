//! Serialization options.

/// Type encapsulating a static flavor of an encoding.
pub struct OptionsBuilder(Options);

/// Start building new options.
///
/// Call [`OptionsBuilder::build`] to construct them.
pub const fn new() -> OptionsBuilder {
    OptionsBuilder(0)
}

/// Type encapsulating a static flavor of an encoding.
pub type Options = u128;

const INTEGER_BIT: Options = 1;
const FLOAT_BIT: Options = 2;
const USIZE_BIT: Options = 3;
const BYTEORDER_BIT: Options = 4;
const USIZE_TYPE_BIT: Options = 5;

impl OptionsBuilder {
    /// Indicates if an integer serialization should be variable.
    pub const fn with_integer(self, number: Integer) -> Self {
        Self(self.0 | (INTEGER_BIT << (number as Options)))
    }

    /// Indicates if an float serialization should be variable.
    pub const fn with_float(self, number: Integer) -> Self {
        Self(self.0 | (FLOAT_BIT << (number as Options)))
    }

    /// Use a native endian byte order.
    pub const fn with_byte_order(self, byte_order: ByteOrder) -> Self {
        Self(self.0 | (BYTEORDER_BIT << (byte_order as Options)))
    }

    /// Indicates if an usize serialization should be variable.
    pub const fn with_usize(self, number: Integer) -> Self {
        Self(self.0 | (USIZE_BIT << (number as Options)))
    }

    /// Build a flavor.
    pub const fn build(self) -> Options {
        self.0
    }
}

#[doc(hidden)]
pub const fn integer<const F: Options>() -> Integer {
    match (F >> INTEGER_BIT) & 1 {
        0 => Integer::Variable,
        _ => Integer::Fixed,
    }
}

#[doc(hidden)]
pub const fn float<const F: Options>() -> Integer {
    match (F >> FLOAT_BIT) & 1 {
        0 => Integer::Variable,
        _ => Integer::Fixed,
    }
}

#[doc(hidden)]
pub const fn usize<const F: Options>() -> Integer {
    match (F >> USIZE_BIT) & 1 {
        0 => Integer::Variable,
        _ => Integer::Fixed,
    }
}

#[doc(hidden)]
pub const fn usize_width<const F: Options>() -> Width {
    match (F >> USIZE_TYPE_BIT) & 2 {
        0 => Width::U8,
        1 => Width::U16,
        2 => Width::U32,
        _ => Width::U64,
    }
}

#[doc(hidden)]
pub const fn byteorder<const F: Options>() -> ByteOrder {
    match (F >> BYTEORDER_BIT) & 1 {
        0 => ByteOrder::LittleEndian,
        _ => ByteOrder::BigEndian,
    }
}

/// Number serialization mode.
#[cfg_attr(test, derive(Debug, PartialEq))]
#[repr(u8)]
#[non_exhaustive]
pub enum Integer {
    /// Variable number encoding.
    Variable = 0,
    /// Fixed number encoding.
    Fixed = 1,
}

/// Byte order.
#[cfg_attr(test, derive(Debug, PartialEq))]
#[repr(u8)]
#[non_exhaustive]
pub enum ByteOrder {
    /// Little endian byte order.
    LittleEndian = 0,
    /// Big endian byte order.
    BigEndian = 1,
}

impl ByteOrder {
    /// The native byte order.
    pub const NATIVE: Self = if cfg!(target_endian = "little") {
        Self::LittleEndian
    } else {
        Self::BigEndian
    };

    /// The network byte order.
    pub const NETWORK: Self = Self::BigEndian;
}

/// The width of a numerical type.
#[cfg_attr(test, derive(Debug, PartialEq))]
#[repr(u8)]
#[non_exhaustive]
pub enum Width {
    /// 8 bit width.
    U8 = 0,
    /// 16 bit width.
    U16 = 1,
    /// 32 bit width.
    U32 = 2,
    /// 64 bit width.
    U64 = 3,
}

#[test]
fn test_flavor() {
    const F1: Options = self::new().with_integer(Integer::Fixed).build();
    assert_eq!(integer::<F1>(), Integer::Fixed);

    const F2: Options = self::new().build();
    assert_eq!(integer::<F2>(), Integer::Variable);
}
