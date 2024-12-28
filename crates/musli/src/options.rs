//! Serialization options.

use core::fmt;

/// [`Options`] builder.
pub struct Builder(Options);

const DEFAULT: Options = (ByteOrder::Little as Options) << BYTEORDER_BIT;

/// Start building new options.
///
/// Call [`Builder::build`] to construct them.
#[inline]
pub const fn new() -> Builder {
    Builder(DEFAULT)
}

/// Construct a [`Builder`] from the raw underlying value of an [`Options`].
///
/// This can be used to modify a value at compile time.
#[inline]
pub const fn from_raw(value: Options) -> Builder {
    Builder(value)
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
pub type Options = u32;

const BYTEORDER_BIT: Options = 0;
const INTEGER_BIT: Options = 4;
const FLOAT_BIT: Options = 8;
const LENGTH_BIT: Options = 12;
const MAP_KEYS_AS_NUMBERS_BIT: Options = 16;

impl Builder {
    /// Indicates if an integer serialization should be variable.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::options::{self, Integer, Options};
    ///
    /// const OPTIONS: Options = options::new().integer(Integer::Fixed).build();
    /// ```
    #[inline]
    pub const fn integer(self, integer: Integer) -> Self {
        const MASK: Options = Integer::MASK << INTEGER_BIT;
        Self((self.0 & !MASK) | ((integer as Options) << INTEGER_BIT))
    }

    /// Indicates the configuration of float serialization.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::options::{self, Float, Options};
    ///
    /// const OPTIONS: Options = options::new().float(Float::Fixed).build();
    /// ```
    #[inline]
    pub const fn float(self, float: Float) -> Self {
        const MASK: Options = Float::MASK << FLOAT_BIT;
        Self((self.0 & !MASK) | ((float as Options) << FLOAT_BIT))
    }

    /// Specify which byte order to use.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::options::{self, ByteOrder, Options};
    ///
    /// const OPTIONS: Options = options::new().byte_order(ByteOrder::Little).build();
    /// ```
    #[inline]
    pub const fn byte_order(self, byte_order: ByteOrder) -> Self {
        const MASK: Options = ByteOrder::MASK << BYTEORDER_BIT;
        Self((self.0 & !MASK) | ((byte_order as Options) << BYTEORDER_BIT))
    }

    /// Specify that the [`ByteOrder::NATIVE`] byte order should be used.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::options::{self, Width, Options};
    ///
    /// const OPTIONS: Options = options::new().native_byte_order().build();
    /// ```
    #[inline]
    pub const fn native_byte_order(self) -> Self {
        self.byte_order(ByteOrder::NATIVE)
    }

    /// Sets the way in which pointer-like `usize` and `isize` types are
    /// encoded.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::options::{self, Width, Options};
    ///
    /// const OPTIONS: Options = options::new().pointer(Width::U32).build();
    /// ```
    #[inline]
    pub const fn pointer(self, width: Width) -> Self {
        const MASK: Options = Width::MASK << LENGTH_BIT;
        Self((self.0 & !MASK) | ((width as Options) << LENGTH_BIT))
    }

    /// Configured a format to use numbers as map keys.
    ///
    /// This options is used for an encoding such as JSON to allow for storing
    /// numbers as string keys, since this would otherwise not be possible and
    /// cause an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::options::{self, Options};
    ///
    /// const OPTIONS: Options = options::new().map_keys_as_numbers().build();
    /// ```
    #[inline]
    pub const fn map_keys_as_numbers(self) -> Self {
        const MASK: Options = 0b1 << MAP_KEYS_AS_NUMBERS_BIT;
        Self((self.0 & !MASK) | (1 << MAP_KEYS_AS_NUMBERS_BIT))
    }

    /// Configure the options to use fixed serialization.
    ///
    /// This causes numerical types to use the default fixed-length
    /// serialization which is typically more efficient than variable-length
    /// through [`variable()`] but is less compact.
    ///
    /// This is the same as calling [`integer(Integer::Fixed)`],
    /// [`float(Float::Fixed)`], and [`pointer(Width::NATIVE)`].
    ///
    /// [`variable()`]: Builder::variable
    /// [`integer(Integer::Fixed)`]: Builder::integer
    /// [`float(Float::Fixed)`]: Builder::float
    /// [`pointer(Width::NATIVE)`]: Builder::pointer
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::options::{self, Options};
    ///
    /// const OPTIONS: Options = options::new().fixed().build();
    /// ```
    #[inline]
    pub const fn fixed(self) -> Self {
        self.integer(Integer::Fixed)
            .float(Float::Fixed)
            .pointer(Width::NATIVE)
    }

    /// Configure the options to use variable serialization.
    ///
    /// This causes numerical types to use the default variable-length
    /// serialization which is typically less efficient than fixed-length
    /// through [`fixed()`] but is more compact.
    ///
    /// This is the same as calling [`integer(Integer::Variable)`],
    /// [`float(Float::Variable)`], and [`pointer(Width::Variable)`].
    ///
    /// [`fixed()`]: Builder::fixed
    /// [`integer(Integer::Variable)`]: Builder::integer
    /// [`float(Float::Variable)`]: Builder::float
    /// [`pointer(Width::Variable)`]: Builder::pointer
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::options::{self, Options};
    ///
    /// const OPTIONS: Options = options::new().variable().build();
    /// ```
    #[inline]
    pub const fn variable(self) -> Self {
        self.integer(Integer::Variable)
            .float(Float::Variable)
            .pointer(Width::Variable)
    }

    /// Built an options builder into a constant.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::options::{self, Options};
    ///
    /// const OPTIONS: Options = options::new().variable().build();
    /// ```
    #[inline]
    pub const fn build(self) -> Options {
        self.0
    }
}

impl fmt::Debug for Builder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builder")
            .field("byteorder", &byteorder_value(self.0))
            .field("integer", &integer_value(self.0))
            .field("float", &float_value(self.0))
            .field("length", &length_value(self.0))
            .field(
                "is_map_keys_as_numbers",
                &is_map_keys_as_numbers_value(self.0),
            )
            .finish()
    }
}

#[cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "json",
    feature = "value"
))]
#[inline]
pub(crate) const fn integer<const OPT: Options>() -> Integer {
    integer_value(OPT)
}

#[inline]
const fn integer_value(opt: Options) -> Integer {
    match (opt >> INTEGER_BIT) & 0b1 {
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
#[inline]
pub(crate) const fn float<const OPT: Options>() -> Float {
    float_value(OPT)
}

#[inline]
const fn float_value(opt: Options) -> Float {
    match (opt >> FLOAT_BIT) & 0b11 {
        0b00 => Float::Integer,
        0b01 => Float::Variable,
        0b10 => Float::Fixed,
        _ => Float::Pad0,
    }
}

#[cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "json",
    feature = "value"
))]
#[inline]
pub(crate) const fn length<const OPT: Options>() -> Width {
    length_value(OPT)
}

#[inline]
const fn length_value(opt: Options) -> Width {
    match (opt >> LENGTH_BIT) & 0b111 {
        0b000 => Width::Variable,
        0b001 => Width::U8,
        0b010 => Width::U16,
        0b011 => Width::U32,
        0b100 => Width::U64,
        0b101 => Width::Pad0,
        0b110 => Width::Pad1,
        _ => Width::Pad2,
    }
}

#[cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "json",
    feature = "value"
))]
#[inline]
pub(crate) const fn byteorder<const OPT: Options>() -> ByteOrder {
    byteorder_value(OPT)
}

#[inline]
pub(crate) const fn byteorder_value(opt: Options) -> ByteOrder {
    match (opt >> BYTEORDER_BIT) & 0b1 {
        0 => ByteOrder::Little,
        _ => ByteOrder::Big,
    }
}

#[cfg(all(feature = "alloc", feature = "value"))]
#[inline]
pub(crate) const fn is_map_keys_as_numbers<const OPT: Options>() -> bool {
    is_map_keys_as_numbers_value(OPT)
}

const fn is_map_keys_as_numbers_value(opt: Options) -> bool {
    ((opt >> MAP_KEYS_AS_NUMBERS_BIT) & 0b1) == 1
}

#[cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "value"
))]
pub(crate) const fn is_native_fixed<const OPT: Options>() -> bool {
    matches!(
        (integer::<OPT>(), float::<OPT>(), length::<OPT>(),),
        (Integer::Fixed, Float::Fixed, Width::NATIVE)
    )
}

/// Integer serialization mode.
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
#[non_exhaustive]
pub enum Integer {
    /// Variable number encoding.
    Variable = 0b0,
    /// Fixed number encoding.
    Fixed = 0b1,
}

impl Integer {
    const MASK: Options = 0b1;
}

/// Float serialization mode.
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
#[non_exhaustive]
pub enum Float {
    /// Use the same serialization as integers, after coercing the bits of a
    /// float into an unsigned integer.
    Integer = 0b00,
    /// Use variable float encoding.
    Variable = 0b01,
    /// Use fixed float encoding.
    Fixed = 0b10,
    /// Padding.
    #[doc(hidden)]
    Pad0 = 0b11,
}

impl Float {
    const MASK: Options = 0b11;
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
    const MASK: Options = 0b1;

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
            _ => {
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
    /// Use a variable width encoding.
    Variable = 0b000,
    /// 8 bit width.
    U8 = 0b001,
    /// 16 bit width.
    U16 = 0b010,
    /// 32 bit width.
    U32 = 0b011,
    /// 64 bit width.
    U64 = 0b100,
    /// Padding.
    #[doc(hidden)]
    Pad0 = 0b101,
    /// Padding.
    #[doc(hidden)]
    Pad1 = 0b110,
    /// Padding.
    #[doc(hidden)]
    Pad2 = 0b111,
}

impl Width {
    const MASK: Options = 0b111;

    /// The native width.
    ///
    /// This is the width of the target platform's native integer type.
    pub const NATIVE: Self = const {
        if cfg!(target_pointer_width = "64") {
            Self::U64
        } else if cfg!(target_pointer_width = "32") {
            Self::U32
        } else if cfg!(target_pointer_width = "16") {
            Self::U16
        } else {
            panic!("Unsupported target pointer width")
        }
    };
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
            $(is_map_keys_as_numbers = $is_map_keys_as_numbers:expr,)?
        }) => {{
            const O: Options = $expr.build();
            assert_or_default!($expr, byteorder::<O>(), ByteOrder::Little, ($($byteorder)?));
            assert_or_default!($expr, integer::<O>(), Integer::Variable, ($($integer)?));
            assert_or_default!($expr, float::<O>(), Float::Integer, ($($float)?));
            assert_or_default!($expr, length::<O>(), Width::Variable, ($($length)?));
            assert_or_default!($expr, is_map_keys_as_numbers::<O>(), false, ($($is_map_keys_as_numbers)?));
        }}
    }

    test_case! {
        self::new() => {}
    }

    test_case! {
        self::new().map_keys_as_numbers() => {
            is_map_keys_as_numbers = true,
        }
    }

    test_case! {
        self::new().integer(Integer::Fixed) => {
            integer = Integer::Fixed,
        }
    }

    test_case! {
        self::new().float(Float::Fixed) => {
            float = Float::Fixed,
        }
    }

    test_case! {
        self::new().float(Float::Variable) => {
            float = Float::Variable,
        }
    }

    test_case! {
        self::new().float(Float::Variable) => {
            float = Float::Variable,
        }
    }

    test_case! {
        self::new().byte_order(ByteOrder::Big) => {
            byteorder = ByteOrder::Big,
        }
    }

    test_case! {
        self::new().byte_order(ByteOrder::Little) => {
            byteorder = ByteOrder::Little,
        }
    }

    test_case! {
        self::new().pointer(Width::Variable) => {
            length = Width::Variable,
        }
    }

    test_case! {
        self::new().pointer(Width::U8) => {
            length = Width::U8,
        }
    }

    test_case! {
        self::new().pointer(Width::U16) => {
            length = Width::U16,
        }
    }

    test_case! {
        self::new().pointer(Width::U32) => {
            length = Width::U32,
        }
    }

    test_case! {
        self::new().pointer(Width::U64) => {
            length = Width::U64,
        }
    }
}
