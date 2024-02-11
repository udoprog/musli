//! Serialization options.

/// Type encapsulating a static flavor of an encoding.
pub struct OptionsBuilder(Options);

const DEFAULT: Options = (ByteOrder::NATIVE as Options) << BYTEORDER_BIT;

/// Start building new options.
///
/// Call [`OptionsBuilder::build`] to construct them.
pub const fn new() -> OptionsBuilder {
    OptionsBuilder(DEFAULT)
}

/// Type encapsulating a static flavor of an encoding.
///
/// Note: despite being made up of a primitive type, this cannot be serialized
/// and correctly re-used.
pub type Options = u128;

const BYTEORDER_BIT: Options = 0;
const INTEGER_BIT: Options = 1;
const LENGTH_BIT: Options = 2;
const FLOAT_BIT: Options = 8;
const LENGTH_WIDTH_BIT: Options = 16;

impl OptionsBuilder {
    /// Indicates if an integer serialization should be variable.
    pub const fn with_integer(self, integer: Integer) -> Self {
        Self(self.0 | ((integer as Options) << INTEGER_BIT))
    }

    /// Indicates the configuration of float serialization.
    pub const fn with_float(self, float: Float) -> Self {
        Self(self.0 | ((float as Options) << FLOAT_BIT))
    }

    /// Specify which byte order to use, if that's relevant.
    pub const fn with_byte_order(self, byte_order: ByteOrder) -> Self {
        Self(self.0 | ((byte_order as Options) << BYTEORDER_BIT))
    }

    /// Specify how lengths should be serialized.
    pub const fn with_length(self, length: Integer) -> Self {
        Self(self.0 | ((length as Options) << LENGTH_BIT))
    }

    /// If length is set to [`Integer::Fixed`], specify the width of the length.
    pub const fn with_length_width(self, width: Width) -> Self {
        let this = self.with_length(Integer::Fixed);
        Self(this.0 | ((width as Options) << LENGTH_WIDTH_BIT))
    }

    /// Build a flavor.
    pub const fn build(self) -> Options {
        self.0
    }
}

#[doc(hidden)]
pub const fn integer<const F: Options>() -> Integer {
    match (F >> INTEGER_BIT) & 0b1 {
        0 => Integer::Variable,
        _ => Integer::Fixed,
    }
}

#[doc(hidden)]
pub const fn float<const F: Options>() -> Float {
    match (F >> FLOAT_BIT) & 0b11 {
        0 => Float::Integer,
        1 => Float::Variable,
        _ => Float::Fixed,
    }
}

#[doc(hidden)]
pub const fn length<const F: Options>() -> Integer {
    match (F >> LENGTH_BIT) & 0b1 {
        0 => Integer::Variable,
        _ => Integer::Fixed,
    }
}

#[doc(hidden)]
pub const fn length_width<const F: Options>() -> Width {
    match (F >> LENGTH_WIDTH_BIT) & 0b11 {
        0 => Width::U8,
        1 => Width::U16,
        2 => Width::U32,
        _ => Width::U64,
    }
}

#[doc(hidden)]
pub const fn byteorder<const F: Options>() -> ByteOrder {
    match (F >> BYTEORDER_BIT) & 0b1 {
        0 => ByteOrder::LittleEndian,
        _ => ByteOrder::BigEndian,
    }
}

/// Integer serialization mode.
#[cfg_attr(test, derive(Debug, PartialEq))]
#[repr(u8)]
#[non_exhaustive]
pub enum Integer {
    /// Variable number encoding.
    Variable = 0,
    /// Fixed number encoding.
    Fixed = 1,
}

/// Float serialization mode.
#[cfg_attr(test, derive(Debug, PartialEq))]
#[repr(u8)]
#[non_exhaustive]
pub enum Float {
    /// Use the same serialization as integers, after coercing the bits of a
    /// float into an unsigned integer.
    Integer = 0,
    /// Use variable float encoding.
    Variable = 1,
    /// Use fixed float encoding.
    Fixed = 2,
}

/// Byte order.
#[derive(PartialEq, Eq)]
#[cfg_attr(test, derive(Debug))]
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

#[doc(hidden)]
#[macro_export]
macro_rules! width_arm {
    ($width:expr, $macro:path) => {
        match $width {
            $crate::options::Width::U8 => {
                $macro!(u8)
            }
            $crate::options::Width::U16 => {
                $macro!(u16)
            }
            $crate::options::Width::U32 => {
                $macro!(u32)
            }
            _ => {
                $macro!(u64)
            }
        }
    };
}

/// The width of a numerical type.
#[derive(Clone, Copy)]
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
fn test_builds() {
    macro_rules! assert_or_default {
        ($expr:expr, $test:expr, $default:expr, ()) => {
            assert_eq!(
                $test,
                $default,
                "{}: Expected default value for {}",
                stringify!($expr),
                stringify!($test)
            );
        };

        ($expr:expr, $test:expr, $_default:expr, ($expected:expr)) => {
            assert_eq!(
                $test,
                $expected,
                "{}: Expected custom value for {}",
                stringify!($expr),
                stringify!($test)
            );
        };
    }

    macro_rules! test_case {
        ($expr:expr => {
            $(byteorder = $byteorder:expr,)?
            $(integer = $integer:expr,)?
            $(float = $float:expr,)?
            $(length = $length:expr,)?
            $(length_width = $length_width:expr,)?
        }) => {{
            const O: Options = $expr.build();
            assert_or_default!($expr, byteorder::<O>(), ByteOrder::NATIVE, ($($byteorder)?));
            assert_or_default!($expr, integer::<O>(), Integer::Variable, ($($integer)?));
            assert_or_default!($expr, float::<O>(), Float::Integer, ($($float)?));
            assert_or_default!($expr, length::<O>(), Integer::Variable, ($($length)?));
            assert_or_default!($expr, length_width::<O>(), Width::U8, ($($length_width)?));
        }}
    }

    test_case! {
        self::new() => {}
    }

    test_case! {
        self::new().with_integer(Integer::Fixed) => {
            integer = Integer::Fixed,
        }
    }

    test_case! {
        self::new().with_float(Float::Fixed) => {
            float = Float::Fixed,
        }
    }

    test_case! {
        self::new().with_float(Float::Variable) => {
            float = Float::Variable,
        }
    }

    test_case! {
        self::new().with_float(Float::Variable) => {
            float = Float::Variable,
        }
    }

    test_case! {
        self::new().with_byte_order(ByteOrder::BigEndian) => {
            byteorder = ByteOrder::BigEndian,
        }
    }

    test_case! {
        self::new().with_byte_order(ByteOrder::LittleEndian) => {
            byteorder = ByteOrder::LittleEndian,
        }
    }

    test_case! {
        self::new().with_length_width(Width::U16) => {
            length = Integer::Fixed,
            length_width = Width::U16,
        }
    }

    test_case! {
        self::new().with_length_width(Width::U32) => {
            length = Integer::Fixed,
            length_width = Width::U32,
        }
    }

    test_case! {
        self::new().with_length_width(Width::U64) => {
            length = Integer::Fixed,
            length_width = Width::U64,
        }
    }
}
