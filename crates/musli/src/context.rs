use core::fmt;

/// Provides ergonomic access to the serialization context.
///
/// This is used to among other things report diagnostics.
pub trait Context {
    /// The error type which is collected by the context.
    type Input;
    /// Error produced by context.
    type Error;
    /// Returned marker for matching field traces.
    type FieldMarker: Default;
    /// Returned marker for matching variant traces.
    type VariantMarker: Default;

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

    /// Encountered an unsupported field tag.
    #[inline]
    fn invalid_field_tag<T>(&mut self, type_name: &'static str, tag: T) -> Self::Error
    where
        T: fmt::Debug,
    {
        self.message(format_args!("{type_name}: invalid field tag: {tag:?}"))
    }

    /// Encountered an unsupported field tag, where the tag has been stored
    /// using [`store_string`].
    ///
    /// [`store_string`]: Context::store_string
    #[inline]
    fn invalid_field_string_tag(&mut self, type_name: &'static str) -> Self::Error {
        if let Some(..) = self.get_string() {
            // self.message(format_args!("{type_name}: invalid field tag: {string}"))
        }

        self.message(format_args!("{type_name}: invalid field tag"))
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

    /// For named (string) variants, stores the tag string in the context.
    ///
    /// It should be possible to recall the string later using [`take_string`].
    ///
    /// [`take_string`]: Context::take_string
    #[allow(unused_variables)]
    fn store_string(&mut self, string: &str) {}

    /// Access the last string stored with [`store_string`].
    ///
    /// [`store_string`]: Context::store_string
    fn get_string(&self) -> Option<&str> {
        None
    }

    /// Trace that we've entered the given index of an array.
    #[inline(always)]
    fn trace_array(&mut self, _: usize) {}

    /// Indicate that we've entered a struct with the given `name`.
    ///
    /// The `name` variable corresponds to the identifiers of the struct.
    ///
    /// This will be matched with a corresponding call to
    /// [`trace_leave_struct`][Context::trace_leave_struct].
    ///
    /// [`trace_leave_struct`]: Context::trace_leave_struct
    #[allow(unused_variables)]
    fn trace_enter_struct(&mut self, name: &'static str) {}

    /// Trace that we've left the last struct that was entered.
    #[inline(always)]
    fn trace_leave_struct(&mut self) {}

    /// Trace that we've entered the given named field.
    ///
    /// A named field is part of a regular struct, where the literal field name
    /// is the `name` argument below, and the musli tag being used for the field
    /// is the second argument.
    ///
    /// This will be matched with a corresponding call to [`trace_leave_field`].
    ///
    /// Here `name` is `"field"` and `tag` is `"string"`.
    ///
    /// ```rust
    /// use musli::{Decode, Encode};
    ///
    /// #[derive(Decode, Encode)]
    /// struct Struct {
    ///     #[musli(rename = "string")]
    ///     field: String,
    /// }
    /// ```
    ///
    /// [`trace_leave_field`]: Context::trace_leave_field
    #[inline(always)]
    #[allow(unused_variables)]
    fn trace_enter_named_field<T>(&mut self, name: &'static str, tag: T) -> Self::FieldMarker
    where
        T: fmt::Display,
    {
        Self::FieldMarker::default()
    }

    /// Trace that we've entered the given unnamed field.
    ///
    /// An unnamed field is part of a tuple struct, where the field index is the
    /// `index` argument below, and the musli tag being used for the field is
    /// the second argument.
    ///
    /// This will be matched with a corresponding call to [`trace_leave_field`].
    ///
    /// Here `index` is `0` and `tag` is `"string"`.
    ///
    /// ```rust
    /// use musli::{Decode, Encode};
    ///
    /// #[derive(Decode, Encode)]
    /// struct Struct(#[musli(rename = "string")] String);
    /// ```
    ///
    /// [`trace_leave_field`]: Context::trace_leave_field
    #[inline(always)]
    #[allow(unused_variables)]
    fn trace_enter_unnamed_field<T>(&mut self, index: u32, _: T) -> Self::FieldMarker
    where
        T: fmt::Display,
    {
        Self::FieldMarker::default()
    }

    /// Trace that we've left the last field that was entered.
    ///
    /// The `marker` argument will be the same as the one returned from
    /// [`trace_enter_named_field`] or [`trace_enter_unnamed_field`].
    ///
    /// [`trace_enter_named_field`]: Context::trace_enter_named_field
    /// [`trace_enter_unnamed_field`]: Context::trace_enter_unnamed_field
    #[inline(always)]
    #[allow(unused_variables)]
    fn trace_leave_field(&mut self, marker: Self::FieldMarker) {}

    /// Trace that we've entered the given variant in an enum.
    ///
    /// A named variant is part of an enum, where the literal variant name is
    /// the `name` argument below, and the musli tag being used to decode the
    /// variant is the second argument.
    ///
    /// This will be matched with a corresponding call to
    /// [`trace_leave_variant`] with the same marker provided as an argument as
    /// the one returned here.
    ///
    /// Here `name` is `"field"` and `tag` is `"string"`.
    ///
    /// ```rust
    /// use musli::{Decode, Encode};
    ///
    /// #[derive(Decode, Encode)]
    /// struct Struct {
    ///     #[musli(rename = "string")]
    ///     field: String,
    /// }
    /// ```
    ///
    /// [`trace_leave_variant`]: Context::trace_leave_variant
    #[inline(always)]
    #[allow(unused_variables)]
    fn trace_enter_variant<T>(&mut self, name: &'static str, tag: T) -> Self::VariantMarker {
        Self::VariantMarker::default()
    }

    /// Trace that we've left the last variant that was entered.
    ///
    /// The `marker` argument will be the same as the one returned from
    /// [`trace_enter_variant`].
    ///
    /// [`trace_enter_variant`]: Context::trace_enter_variant
    #[inline(always)]
    #[allow(unused_variables)]
    fn trace_leave_variant(&mut self, marker: Self::VariantMarker) {}
}
