//! Internal helpers for generating nice expectation messages.

use core::fmt;

pub trait Expecting {
    /// Generated the actual message of what we expected.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
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

pub(crate) struct InvalidType<'a, A> {
    actual: A,
    expecting: &'a dyn Expecting,
}

impl<'a, A> InvalidType<'a, A> {
    pub(crate) const fn new(actual: A, expecting: &'a dyn Expecting) -> Self {
        Self { actual, expecting }
    }
}

impl<'a, A> fmt::Display for InvalidType<'a, A>
where
    A: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid type: got {}, but expected {}",
            self.actual, self.expecting
        )
    }
}

pub(crate) struct BadVisitorType<'a, A> {
    actual: A,
    expecting: &'a dyn Expecting,
}

impl<'a, A> BadVisitorType<'a, A> {
    pub(crate) const fn new(actual: A, expecting: &'a dyn Expecting) -> Self {
        Self { actual, expecting }
    }
}

impl<'a, 'de, A> fmt::Display for BadVisitorType<'a, A>
where
    A: fmt::Display,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "bad reference type: got: {}, expected: {}",
            self.actual, self.expecting
        )
    }
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
    pub(crate) Sequence("sequence");
    pub(crate) Tuple("tuple");
    pub(crate) Unit("unit");
    pub(crate) Struct("struct");
    pub(crate) UnitStruct("unit struct");
    pub(crate) Variant("variant");
    pub(crate) StructVariant("struct variant");
    pub(crate) TupleVariant("tuple variant");
    pub(crate) UnitVariant("unit variant");
    pub(crate) AnyValue("a value");
}
