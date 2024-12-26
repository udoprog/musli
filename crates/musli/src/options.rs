//! Serialization options.

/// [`Options`] builder.
pub struct Builder(Options);

const DEFAULT: Options = (ByteOrder::NATIVE as Options) << BYTEORDER_BIT;

/// Start building new options.
///
/// Call [`Builder::build`] to construct them.
pub const fn new() -> Builder {
    Builder(DEFAULT)
}

/// Type encapsulating a static options for an encoding.
///
/// Note: despite being made up of a primitive type, this cannot be serialized
/// and correctly re-used. This is simply the case because of restrictions in
/// constant evaluation.
///
/// Making assumptions about its layout might lead to unspecified behavior
/// during encoding. Only use this type through the provided [`options`] APIs.
///
/// [`options`]: crate::options
pub type Options = u128;

const BYTEORDER_BIT: Options = 0;
const INTEGER_BIT: Options = 1;
const LENGTH_BIT: Options = 2;
const MAP_KEYS_AS_NUMBERS_BIT: Options = 3;
const FLOAT_BIT: Options = 8;
const LENGTH_WIDTH_BIT: Options = 16;

impl Builder {
    /// Indicates if an integer serialization should be variable.
    #[inline(always)]
    pub const fn with_integer(self, integer: Integer) -> Self {
        const MASK: Options = 0b1 << INTEGER_BIT;
        Self((self.0 & !MASK) | ((integer as Options) << INTEGER_BIT))
    }

    /// Indicates the configuration of float serialization.
    #[inline(always)]
    pub const fn with_float(self, float: Float) -> Self {
        const MASK: Options = 0b11 << FLOAT_BIT;
        Self((self.0 & !MASK) | ((float as Options) << FLOAT_BIT))
    }

    /// Specify which byte order to use, if that's relevant.
    #[inline(always)]
    pub const fn with_byte_order(self, byte_order: ByteOrder) -> Self {
        const MASK: Options = 0b1 << BYTEORDER_BIT;
        Self((self.0 & !MASK) | ((byte_order as Options) << BYTEORDER_BIT))
    }

    /// Specify how lengths should be serialized.
    #[inline(always)]
    pub const fn with_length(self, length: Integer) -> Self {
        const MASK: Options = 0b1 << LENGTH_BIT;
        Self((self.0 & !MASK) | ((length as Options) << LENGTH_BIT))
    }

    /// Allows for treating string keys as numbers.
    #[inline(always)]
    pub const fn with_map_keys_as_numbers(self, value: bool) -> Self {
        const MASK: Options = 0b1 << MAP_KEYS_AS_NUMBERS_BIT;
        let value = if value { 1 } else { 0 };
        Self((self.0 & !MASK) | (value << MAP_KEYS_AS_NUMBERS_BIT))
    }

    /// If length is set to [`Integer::Fixed`], specify the width of the length.
    #[inline(always)]
    pub const fn with_length_width(self, width: Width) -> Self {
        const MASK: Options = 0b11 << LENGTH_WIDTH_BIT;
        let this = self.with_length(Integer::Fixed);
        Self((this.0 & !MASK) | ((width as Options) << LENGTH_WIDTH_BIT))
    }

    /// Build a flavor.
    #[inline(always)]
    pub const fn build(self) -> Options {
        self.0
    }
}

#[cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "json",
    feature = "value"
))]
#[inline(always)]
pub(crate) const fn integer<const OPT: Options>() -> Integer {
    match (OPT >> INTEGER_BIT) & 0b1 {
        0 => Integer::Variable,
        _ => Integer::Fixed,
    }
}

#[cfg(test)]
#[inline(always)]
pub(crate) const fn float<const OPT: Options>() -> Float {
    match (OPT >> FLOAT_BIT) & 0b11 {
        0 => Float::Integer,
        1 => Float::Variable,
        _ => Float::Fixed,
    }
}

#[cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "json",
    feature = "value"
))]
#[inline(always)]
pub(crate) const fn length<const OPT: Options>() -> Integer {
    match (OPT >> LENGTH_BIT) & 0b1 {
        0 => Integer::Variable,
        _ => Integer::Fixed,
    }
}

#[cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "json",
    feature = "value"
))]
#[inline(always)]
pub(crate) const fn length_width<const OPT: Options>() -> Width {
    match (OPT >> LENGTH_WIDTH_BIT) & 0b11 {
        0 => Width::U64,
        1 => Width::U32,
        2 => Width::U16,
        _ => Width::U8,
    }
}

#[cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "json",
    feature = "value"
))]
#[inline(always)]
pub(crate) const fn byteorder<const OPT: Options>() -> ByteOrder {
    match (OPT >> BYTEORDER_BIT) & 0b1 {
        0 => ByteOrder::Little,
        _ => ByteOrder::Big,
    }
}

#[cfg(all(feature = "alloc", feature = "value"))]
#[inline(always)]
pub(crate) const fn is_map_keys_as_numbers<const OPT: Options>() -> bool {
    ((OPT >> MAP_KEYS_AS_NUMBERS_BIT) & 0b1) == 1
}

/// Integer serialization mode.
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
#[non_exhaustive]
pub enum Integer {
    /// Variable number encoding.
    Variable = 0,
    /// Fixed number encoding.
    Fixed = 1,
}

/// Float serialization mode.
#[derive(Debug, PartialEq, Eq)]
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

/// Byte order to use when encoding numbers.
///
/// By default, this is the [`ByteOrder::NATIVE`] byte order of the target
/// platform.
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
#[non_exhaustive]
pub enum ByteOrder {
    /// Little endian byte order.
    Little = 0,
    /// Big endian byte order.
    Big = 1,
}

impl ByteOrder {
    /// The native byte order.
    ///
    /// [`Little`] for little and [`Big`] for big endian platforms.
    ///
    /// [`Little`]: ByteOrder::Little
    /// [`Big`]: ByteOrder::Big
    pub const NATIVE: Self = if cfg!(target_endian = "little") {
        Self::Little
    } else {
        Self::Big
    };

    /// The network byte order.
    ///
    /// This is the same as [`Big`].
    ///
    /// [`Big`]: ByteOrder::Big
    pub const NETWORK: Self = Self::Big;
}

#[doc(hidden)]
#[cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "value"
))]
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
            $crate::options::Width::U64 => {
                $macro!(u64)
            }
        }
    };
}

#[cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "value"
))]
pub(crate) use width_arm;

/// The width of a numerical type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[non_exhaustive]
pub enum Width {
    /// 64 bit width.
    U64 = 0,
    /// 32 bit width.
    U32 = 1,
    /// 16 bit width.
    U16 = 2,
    /// 8 bit width.
    U8 = 3,
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
            $(is_map_keys_as_numbers = $is_map_keys_as_numbers:expr,)?
        }) => {{
            const O: Options = $expr.build();
            assert_or_default!($expr, byteorder::<O>(), ByteOrder::NATIVE, ($($byteorder)?));
            assert_or_default!($expr, integer::<O>(), Integer::Variable, ($($integer)?));
            assert_or_default!($expr, float::<O>(), Float::Integer, ($($float)?));
            assert_or_default!($expr, length::<O>(), Integer::Variable, ($($length)?));
            assert_or_default!($expr, length_width::<O>(), Width::U64, ($($length_width)?));
            assert_or_default!($expr, is_map_keys_as_numbers::<O>(), false, ($($is_map_keys_as_numbers)?));
        }}
    }

    test_case! {
        self::new() => {}
    }

    test_case! {
        self::new().with_map_keys_as_numbers(true) => {
            is_map_keys_as_numbers = true,
        }
    }

    test_case! {
        self::new().with_integer(Integer::Fixed) => {
            integer = Integer::Fixed,
        }
    }

    test_case! {
        self::new().with_length(Integer::Fixed) => {
            length = Integer::Fixed,
            length_width = Width::U64,
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
        self::new().with_byte_order(ByteOrder::Big) => {
            byteorder = ByteOrder::Big,
        }
    }

    test_case! {
        self::new().with_byte_order(ByteOrder::Little) => {
            byteorder = ByteOrder::Little,
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
