//! Trait governing what error types associated with the encoding framework must
//! support.
//!
//! The most important component in here is `Error::custom` which allows custom
//! encoding implementations to raise custom errors based on types that
//! implement [Display][core::fmt::Display].

use core::fmt;

/// Trait governing errors raised during encodeing or decoding.
pub trait Error: 'static + Send + Sync + Sized {
    /// Construct a custom error.
    fn custom<T>(message: T) -> Self
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug;

    /// Collect an error from something that can be displayed.
    ///
    /// This is made available to format custom error messages in `no_std`
    /// environments. The error message is to be collected by formatting `T`.
    fn message<T>(message: T) -> Self
    where
        T: fmt::Display;

    /// Trying to decode an uninhabitable type.
    #[inline]
    fn uninhabitable(type_name: &'static str) -> Self {
        Self::message(format_args!(
            "{}: cannot decode uninhabitable types",
            type_name
        ))
    }

    /// The value for the given tag could not be collected.
    #[inline]
    fn expected_tag<T>(type_name: &'static str, tag: T) -> Self
    where
        T: fmt::Debug,
    {
        Self::message(format_args!("expected tag: {}: {:?}", type_name, tag))
    }

    /// Encountered an unsupported number tag.
    #[inline]
    fn invalid_field_tag<T>(type_name: &'static str, tag: T) -> Self
    where
        T: fmt::Debug,
    {
        Self::message(format_args!("invalid field tag: {}: {:?}", type_name, tag))
    }

    /// Indicate that a variant wasn't supported by tag.
    #[inline]
    fn invalid_variant_tag<T>(type_name: &'static str, tag: T) -> Self
    where
        T: fmt::Debug,
    {
        Self::message(format_args!(
            "invalid variant tag: {}: {:?}",
            type_name, tag
        ))
    }

    /// Encountered an unsupported variant field.
    #[inline]
    fn invalid_variant_field_tag<V, T>(type_name: &'static str, variant: V, tag: T) -> Self
    where
        V: fmt::Debug,
        T: fmt::Debug,
    {
        Self::message(format_args!(
            "invalid variant field tag: {}: variant: {:?}, field: {:?}",
            type_name, tag, variant
        ))
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

    fn message<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        std::io::Error::new(std::io::ErrorKind::Other, message.to_string())
    }
}

#[cfg(feature = "std")]
impl Error for String {
    fn custom<T>(message: T) -> Self
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        message.to_string()
    }

    fn message<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        message.to_string()
    }
}
