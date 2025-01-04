//! Internal helpers for generating nice expectation messages.

use core::fmt::{self, Display};

use crate::de::SizeHint;

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
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt(f)
    }
}

impl fmt::Display for &dyn Expecting {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.expecting(f)
    }
}

struct FormatFn<T>(T);

impl<T> fmt::Display for FormatFn<T>
where
    T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.0)(f)
    }
}

#[inline]
fn format_fn<T>(function: T) -> FormatFn<T>
where
    T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    FormatFn(function)
}

/// Format an invalid type message.
pub(crate) fn unsupported_type<'a>(
    actual: &'a dyn fmt::Display,
    expected: &'a dyn Expecting,
) -> impl fmt::Display + 'a {
    format_fn(move |f| {
        write! {
            f,
            "Got unsupported type `{actual}`, but expected {expected}"
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
            "Bad reference type {actual}, expected {expected}",
        }
    })
}

macro_rules! expect_with {
    ($($vis:vis $ident:ident($string:literal, $ty:ty);)*) => {
        $(
            $vis struct $ident($vis $ty);

            impl fmt::Display for $ident {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, $string, self.0)
                }
            }
        )*
    }
}

macro_rules! expect {
    ($($vis:vis $ident:ident($string:literal);)*) => {
        $(
            $vis struct $ident;

            impl fmt::Display for $ident {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, $string)
                }
            }
        )*
    }
}

expect_with! {
    pub(crate) SequenceWith("sequence with {0}", SizeHint);
    pub(crate) MapWith("map with {0}", SizeHint);
    pub(crate) BytesWith("bytes with {0}", SizeHint);
    pub(crate) StringWith("string with {0}", SizeHint);
}

expect! {
    pub(crate) Any("a dynamic value");
    pub(crate) Empty("empty");
    pub(crate) Option("option");
    pub(crate) Pack("pack");
    pub(crate) Bool("boolean");
    pub(crate) Char("character");
    pub(crate) Number("arbitrary precision number");
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
    pub(crate) MapEntries("map entries");
    pub(crate) UnsizedMap("unsized map");
    pub(crate) MapVariant("map variant");
    pub(crate) UnsizedSequence("unsized sequence");
    pub(crate) SequenceVariant("sequence variant");
    pub(crate) Variant("variant");
    pub(crate) AnyValue("a value");
}
