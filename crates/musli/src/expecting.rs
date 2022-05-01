//! Internal helpers for generating nice expectation messages.

use core::fmt;

pub trait Expecting {
    /// Generated the actual message of what we expected.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Return a type that can be formatted from `self`.
    #[doc(hidden)]
    fn format(&self) -> &dyn Expecting
    where
        Self: Sized,
    {
        self
    }
}

impl Expecting for str {
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use core::fmt::Display;
        self.fmt(f)
    }
}

impl fmt::Display for &dyn Expecting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.expecting(f)
    }
}

struct FormatFn<T>(T);

impl<T> fmt::Display for FormatFn<T>
where
    T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.0)(f)
    }
}

fn format_fn<T>(function: T) -> FormatFn<T>
where
    T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    FormatFn(function)
}

/// Format an invalid type message.
pub(crate) fn invalid_type<'a>(
    actual: &'a dyn fmt::Display,
    expected: &'a dyn Expecting,
) -> impl fmt::Display + 'a {
    format_fn(move |f| {
        write! {
            f,
            "invalid type: got {actual}, but expected {expected}"
        }
    })
}

/// Format a bad visitor type message.
pub(crate) fn bad_visitor_type<'a>(
    actual: &'a dyn fmt::Display,
    expected: &'a dyn Expecting,
) -> impl fmt::Display + 'a {
    format_fn(move |f| {
        write! {
            f,
            "bad reference type: got: {actual}, expected: {expected}",
        }
    })
}

macro_rules! expect {
    ($($vis:vis $ident:ident($string:expr);)*) => {
        $(
            $vis struct $ident;

            impl fmt::Display for $ident {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    $string.fmt(f)
                }
            }
        )*
    }
}

expect! {
    pub(crate) Pack("pack");
    pub(crate) Bool("boolean");
    pub(crate) Char("character");
    pub(crate) Number("arbitrary precision number");
    pub(crate) NumberComponents("number from 128-bit components");
    pub(crate) Unsigned8("8-bit unsigned integer");
    pub(crate) Unsigned16("16-bit unsigned integer");
    pub(crate) Unsigned32("32-bit unsigned integer");
    pub(crate) Unsigned64("64-bit unsigned integer");
    pub(crate) Unsigned128("128-bit unsigned integer");
    pub(crate) Signed8("8-bit signed integer");
    pub(crate) Signed16("16-bit signed integer");
    pub(crate) Signed32("32-bit signed integer");
    pub(crate) Signed64("64-bit signed integer");
    pub(crate) Signed128("128-bit signed integer");
    pub(crate) Float32("32-bit float");
    pub(crate) Float64("64-bit float");
    pub(crate) Isize("isize");
    pub(crate) Usize("usize");
    pub(crate) String("string");
    pub(crate) Bytes("bytes");
    pub(crate) Array("array");
    pub(crate) Map("map");
    pub(crate) Option("option");
    pub(crate) Tuple("tuple");
    pub(crate) Sequence("sequence");
    pub(crate) Unit("unit");
    pub(crate) Struct("struct");
    pub(crate) TupleStruct("tuple struct");
    pub(crate) UnitStruct("unit struct");
    pub(crate) Variant("variant");
    pub(crate) AnyValue("a value");
}
