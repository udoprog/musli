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

    /// Missing a field of the given tag.
    #[inline]
    fn missing_field<T>(type_name: &'static str, tag: T) -> Self
    where
        T: fmt::Debug,
    {
        Self::collect_from_display(MissingField { type_name, tag })
    }

    /// Encountered an unsupported number tag.
    #[inline]
    fn unsupported_tag<T>(type_name: &'static str, tag: T) -> Self
    where
        T: fmt::Debug,
    {
        Self::collect_from_display(UnsupportedTag { type_name, tag })
    }

    /// Invalid value.
    #[inline]
    fn invalid_value(type_name: &'static str) -> Self {
        Self::collect_from_display(InvalidValue { type_name })
    }

    /// Found an unexpected field.
    #[inline]
    fn unexpected_field(type_name: &'static str) -> Self {
        Self::collect_from_display(UnexpectedField { type_name })
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

struct MissingField<T> {
    type_name: &'static str,
    tag: T,
}

impl<T> fmt::Display for MissingField<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: missing field {:?}", self.type_name, self.tag)
    }
}

struct UnsupportedTag<T> {
    type_name: &'static str,
    tag: T,
}

impl<T> fmt::Display for UnsupportedTag<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: unsupported tag {:?}", self.type_name, self.tag)
    }
}

struct InvalidValue {
    type_name: &'static str,
}

impl fmt::Display for InvalidValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: trying to construct from invalid value",
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
