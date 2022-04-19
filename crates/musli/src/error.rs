//! Trait governing what error types associated with the encoding framework must
//! support.
//!
//! The most important component in here is `Error::custom` which allows custom
//! encoding implementations to raise custom errors based on types that
//! implement [Display][core::fmt::Display].

use core::fmt;

/// Trait governing errors raised during encodeing or decoding.
pub trait Error: Sized {
    /// Construct a custom error.
    fn custom<T>(message: T) -> Self
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug;

    /// Collect an error from something that can be displayed.
    ///
    /// This is made available to format custom error messages in `no_std`
    /// environments. The error message is to be collected by formatting `T`.
    fn collect_from_display<T>(message: T) -> Self
    where
        T: fmt::Display;

    /// The given value was unexpected.
    fn bad_value<T>(value: T) -> Self
    where
        T: fmt::Debug,
    {
        Self::collect_from_display(BadValue { value })
    }

    /// Trying to decode an uninhabitable type.
    #[inline]
    fn uninhabitable(type_name: &'static str) -> Self {
        Self::collect_from_display(Uninhabitable { type_name })
    }

    /// Indicate that a variant wasn't supported by tag.
    #[inline]
    fn unsupported_variant<T>(type_name: &'static str, tag: T) -> Self
    where
        T: fmt::Debug,
    {
        Self::collect_from_display(UnsupportedVariant { type_name, tag })
    }

    /// Encountered an unsupported number tag.
    #[inline]
    fn unsupported_field<T>(type_name: &'static str, tag: T) -> Self
    where
        T: fmt::Debug,
    {
        Self::collect_from_display(UnsupportedField { type_name, tag })
    }

    /// Encountered an unsupported variant field.
    #[inline]
    fn unsupported_variant_field<V, T>(type_name: &'static str, variant: V, tag: T) -> Self
    where
        V: fmt::Debug,
        T: fmt::Debug,
    {
        Self::collect_from_display(UnsupportedVariantField {
            type_name,
            variant,
            tag,
        })
    }

    /// Found an unexpected field.
    #[inline]
    fn unexpected_field(type_name: &'static str) -> Self {
        Self::collect_from_display(UnexpectedField { type_name })
    }

    /// The value for the given tag could not be collected.
    #[inline]
    fn expected_tag<T>(type_name: &'static str, tag: T) -> Self
    where
        T: fmt::Debug,
    {
        Self::collect_from_display(ExpectedTag { type_name, tag })
    }
}

#[cfg(feature = "std")]
impl Error for std::io::Error {
    fn custom<T>(message: T) -> Self
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        std::io::Error::new(std::io::ErrorKind::Other, message.to_string())
    }

    fn collect_from_display<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        std::io::Error::new(std::io::ErrorKind::Other, message.to_string())
    }
}

struct BadValue<T> {
    value: T,
}

impl<T> fmt::Display for BadValue<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bad value: {:?}", self.value)
    }
}

struct Uninhabitable {
    type_name: &'static str,
}

impl fmt::Display for Uninhabitable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: cannot decode uninhabitable types", self.type_name)
    }
}

struct UnsupportedVariant<T> {
    type_name: &'static str,
    tag: T,
}

impl<T> fmt::Display for UnsupportedVariant<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: unsupported variant {:?}", self.type_name, self.tag)
    }
}

struct ExpectedTag<T> {
    type_name: &'static str,
    tag: T,
}

impl<T> fmt::Display for ExpectedTag<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: missing field {:?}", self.type_name, self.tag)
    }
}

struct UnsupportedField<T> {
    type_name: &'static str,
    tag: T,
}

impl<T> fmt::Display for UnsupportedField<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: unsupported field {:?}", self.type_name, self.tag)
    }
}

struct UnsupportedVariantField<V, T> {
    type_name: &'static str,
    variant: V,
    tag: T,
}

impl<V, T> fmt::Display for UnsupportedVariantField<V, T>
where
    V: fmt::Debug,
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: unsupported field {:?} in variant {:?}",
            self.type_name, self.tag, self.variant
        )
    }
}

struct UnsupportedValue {
    type_name: &'static str,
}

impl fmt::Display for UnsupportedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: trying to construct from unsupported value",
            self.type_name
        )
    }
}

struct UnexpectedField {
    type_name: &'static str,
}

impl fmt::Display for UnexpectedField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: got a field but expected none", self.type_name)
    }
}
