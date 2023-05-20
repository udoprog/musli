use core::fmt;

/// Provides ergonomic access to the serialization context.
///
/// This is used to among other things report diagnostics.
pub trait Context {
    /// The error type which is collected by the context.
    type Input;
    /// Error produced by context.
    type Error;

    /// Report the given encoding error.
    fn report<T>(&mut self, error: T) -> Self::Error
    where
        Self::Input: From<T>;

    /// Report a custom error.
    fn custom<T>(&mut self, error: T) -> Self::Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug;

    /// Report an error as a message.
    ///
    /// This is made available to format custom error messages in `no_std`
    /// environments. The error message is to be collected by formatting `T`.
    fn message<T>(&mut self, message: T) -> Self::Error
    where
        T: fmt::Display;

    /// Report that an invalid variant tag was encountered.
    #[inline]
    fn invalid_variant_tag<T>(&mut self, type_name: &'static str, tag: T) -> Self::Error
    where
        T: fmt::Debug,
    {
        self.message(format_args!("{type_name}: invalid variant tag: {tag:?}"))
    }

    /// The value for the given tag could not be collected.
    #[inline]
    fn expected_tag<T>(&mut self, type_name: &'static str, tag: T) -> Self::Error
    where
        T: fmt::Debug,
    {
        self.message(format_args!("{type_name}: expected tag: {tag:?}"))
    }

    /// Trying to decode an uninhabitable type.
    #[inline]
    fn uninhabitable(&mut self, type_name: &'static str) -> Self::Error {
        self.message(format_args!(
            "{type_name}: cannot decode uninhabitable types",
        ))
    }

    /// Encountered an unsupported number tag.
    #[inline]
    fn invalid_field_tag<T>(&mut self, type_name: &'static str, tag: T) -> Self::Error
    where
        T: fmt::Debug,
    {
        self.message(format_args!("{type_name}: invalid field tag: {tag:?}"))
    }

    /// Missing variant field required to decode.
    #[inline]
    fn missing_variant_field<T>(&mut self, type_name: &'static str, tag: T) -> Self::Error
    where
        T: fmt::Debug,
    {
        self.message(format_args!("{type_name}: missing variant field: {tag:?}"))
    }

    /// Indicate that a variant tag could not be determined.
    #[inline]
    fn missing_variant_tag(&mut self, type_name: &'static str) -> Self::Error {
        self.message(format_args!("{type_name}: missing variant tag"))
    }

    /// Encountered an unsupported variant field.
    #[inline]
    fn invalid_variant_field_tag<V, T>(
        &mut self,
        type_name: &'static str,
        variant: V,
        tag: T,
    ) -> Self::Error
    where
        V: fmt::Debug,
        T: fmt::Debug,
    {
        self.message(format_args!(
            "{type_name}: invalid variant field tag: variant: {variant:?}, tag: {tag:?}",
        ))
    }

    /// Trace that we've entered the given index of an array.
    #[inline(always)]
    fn trace_array(&mut self, _: usize) {}

    /// Trace that we've entered the given field.
    #[inline(always)]
    fn trace_field<T>(&mut self, _: &T)
    where
        T: ?Sized + fmt::Display,
    {
    }

    /// Trace that we've entered the given variant.
    #[inline(always)]
    fn trace_variant<T>(&mut self, _: &T)
    where
        T: ?Sized + fmt::Display,
    {
    }

    /// Trace that we've left the current context.
    #[inline(always)]
    fn trace_leave(&mut self) {}
}
