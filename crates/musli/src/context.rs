use core::fmt;

/// Provides ergonomic access to the serialization context.
///
/// This is used to among other things report diagnostics.
pub trait Context<'buf> {
    /// The error type which is collected by the context.
    type Input;
    /// Error produced by context.
    type Error;
    /// A mark during processing.
    type Mark: Copy + Default;

    /// Report the given encoding error.
    fn report<T>(&mut self, error: T) -> Self::Error
    where
        Self::Input: From<T>;

    /// Report the given encoding error from the given mark.
    #[allow(unused_variables)]
    #[inline(always)]
    fn marked_report<T>(&mut self, mark: Self::Mark, error: T) -> Self::Error
    where
        Self::Input: From<T>,
    {
        self.report(error)
    }

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

    /// Report an error based on a mark.
    ///
    /// A mark is generated using [Context::mark] and indicates a prior state.
    #[allow(unused_variables)]
    #[inline(always)]
    fn marked_message<T>(&mut self, mark: Self::Mark, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.message(message)
    }

    /// Advance the context by `n` bytes of input.
    ///
    /// This is typically used to move the mark forward as produced by
    /// [Context::mark].
    #[allow(unused_variables)]
    #[inline(always)]
    fn advance(&mut self, n: usize) {}

    /// Return a mark which acts as a checkpoint at the current encoding state.
    ///
    /// The context is in a privileged state in that it sees everything, so a
    /// mark can be quite useful for determining the context of an error.
    ///
    /// This typically indicates a byte offset, and is used by
    /// [`marked_message`][Context::marked_message] to report a spanned error.
    #[inline(always)]
    fn mark(&mut self) -> Self::Mark {
        Self::Mark::default()
    }

    /// Report that an invalid variant tag was encountered.
    #[inline(always)]
    fn invalid_variant_tag<T>(&mut self, _: &'static str, tag: T) -> Self::Error
    where
        T: fmt::Debug,
    {
        self.message(format_args!("invalid variant tag: {tag:?}"))
    }

    /// The value for the given tag could not be collected.
    #[inline(always)]
    fn expected_tag<T>(&mut self, _: &'static str, tag: T) -> Self::Error
    where
        T: fmt::Debug,
    {
        self.message(format_args!("expected tag: {tag:?}"))
    }

    /// Trying to decode an uninhabitable type.
    #[inline(always)]
    fn uninhabitable(&mut self, _: &'static str) -> Self::Error {
        self.message(format_args!("cannot decode uninhabitable types",))
    }

    /// Encountered an unsupported field tag.
    #[inline(always)]
    fn invalid_field_tag<T>(&mut self, _: &'static str, tag: T) -> Self::Error
    where
        T: fmt::Debug,
    {
        self.message(format_args!("invalid field tag: {tag:?}"))
    }

    /// Encountered an unsupported field tag, where the tag has been stored
    /// using [`store_string`].
    ///
    /// [`store_string`]: Context::store_string
    #[inline(always)]
    fn invalid_field_string_tag(&mut self, _: &'static str) -> Self::Error {
        if let Some(string) = self.get_string() {
            self.message(format_args!("invalid field tag: {string}"))
        } else {
            self.message(format_args!("invalid field tag"))
        }
    }

    /// Missing variant field required to decode.
    #[inline(always)]
    fn missing_variant_field<T>(&mut self, _: &'static str, tag: T) -> Self::Error
    where
        T: fmt::Debug,
    {
        self.message(format_args!("missing variant field: {tag:?}"))
    }

    /// Indicate that a variant tag could not be determined.
    #[inline(always)]
    fn missing_variant_tag(&mut self, _: &'static str) -> Self::Error {
        self.message(format_args!("missing variant tag"))
    }

    /// Encountered an unsupported variant field.
    #[inline(always)]
    fn invalid_variant_field_tag<V, T>(
        &mut self,
        _: &'static str,
        variant: V,
        tag: T,
    ) -> Self::Error
    where
        V: fmt::Debug,
        T: fmt::Debug,
    {
        self.message(format_args!(
            "invalid variant field tag: variant: {variant:?}, tag: {tag:?}",
        ))
    }

    /// For named (string) variants, stores the tag string in the context.
    ///
    /// It should be possible to recall the string later using [`take_string`].
    ///
    /// [`take_string`]: Context::take_string
    #[allow(unused_variables)]
    #[inline(always)]
    fn store_string(&mut self, string: &str) {}

    /// Access the last string stored with [`store_string`], referenced from the
    /// internal buffer.
    ///
    /// [`store_string`]: Context::store_string
    #[inline(always)]
    fn get_string(&self) -> Option<&'buf str> {
        None
    }

    /// Indicate that we've entered a struct with the given `name`.
    ///
    /// The `name` variable corresponds to the identifiers of the struct.
    ///
    /// This will be matched with a corresponding call to [`leave_struct`].
    ///
    /// [`leave_struct`]: Context::leave_struct
    #[allow(unused_variables)]
    #[inline(always)]
    fn enter_struct(&mut self, name: &'static str) {}

    /// Trace that we've left the last struct that was entered.
    #[inline(always)]
    fn leave_struct(&mut self) {}

    /// Indicate that we've entered an enum with the given `name`.
    ///
    /// The `name` variable corresponds to the identifiers of the enum.
    ///
    /// This will be matched with a corresponding call to [`leave_enum`].
    ///
    /// [`leave_enum`]: Context::leave_enum
    #[allow(unused_variables)]
    #[inline(always)]
    fn enter_enum(&mut self, name: &'static str) {}

    /// Trace that we've left the last enum that was entered.
    #[inline(always)]
    fn leave_enum(&mut self) {}

    /// Trace that we've entered the given named field.
    ///
    /// A named field is part of a regular struct, where the literal field name
    /// is the `name` argument below, and the musli tag being used for the field
    /// is the second argument.
    ///
    /// This will be matched with a corresponding call to [`leave_field`].
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
    /// [`leave_field`]: Context::leave_field
    #[allow(unused_variables)]
    #[inline(always)]
    fn enter_named_field<T>(&mut self, name: &'static str, tag: T)
    where
        T: fmt::Display,
    {
    }

    /// Trace that we've entered the given unnamed field.
    ///
    /// An unnamed field is part of a tuple struct, where the field index is the
    /// `index` argument below, and the musli tag being used for the field is
    /// the second argument.
    ///
    /// This will be matched with a corresponding call to [`leave_field`].
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
    /// [`leave_field`]: Context::leave_field
    #[allow(unused_variables)]
    #[inline(always)]
    fn enter_unnamed_field<T>(&mut self, index: u32, _: T)
    where
        T: fmt::Display,
    {
    }

    /// Trace that we've left the last field that was entered.
    ///
    /// The `marker` argument will be the same as the one returned from
    /// [`enter_named_field`] or [`enter_unnamed_field`].
    ///
    /// [`enter_named_field`]: Context::enter_named_field
    /// [`enter_unnamed_field`]: Context::enter_unnamed_field
    #[allow(unused_variables)]
    #[inline(always)]
    fn leave_field(&mut self) {}

    /// Trace that we've entered the given variant in an enum.
    ///
    /// A named variant is part of an enum, where the literal variant name is
    /// the `name` argument below, and the musli tag being used to decode the
    /// variant is the second argument.
    ///
    /// This will be matched with a corresponding call to
    /// [`leave_variant`] with the same marker provided as an argument as
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
    /// [`leave_variant`]: Context::leave_variant
    #[allow(unused_variables)]
    #[inline(always)]
    fn enter_variant<T>(&mut self, name: &'static str, tag: T)
    where
        T: fmt::Display,
    {
    }

    /// Trace that we've left the last variant that was entered.
    ///
    /// The `marker` argument will be the same as the one returned from
    /// [`enter_variant`].
    ///
    /// [`enter_variant`]: Context::enter_variant
    #[allow(unused_variables)]
    #[inline(always)]
    fn leave_variant(&mut self) {}

    /// Trace a that a map key has been entered.
    #[allow(unused_variables)]
    #[inline(always)]
    fn enter_map_key<T>(&mut self, field: T)
    where
        T: fmt::Display,
    {
    }

    /// Trace that we've left the last map field that was entered.
    ///
    /// The `marker` argument will be the same as the one returned from
    /// [`enter_map_key`].
    ///
    /// [`enter_map_key`]: Context::enter_map_key
    #[allow(unused_variables)]
    #[inline(always)]
    fn leave_map_key(&mut self) {}

    /// Trace a sequence field.
    #[allow(unused_variables)]
    #[inline(always)]
    fn enter_sequence_index(&mut self, index: usize) {}

    /// Trace that we've left the last sequence index that was entered.
    ///
    /// The `marker` argument will be the same as the one returned from
    /// [`enter_sequence_index`].
    ///
    /// [`enter_sequence_index`]: Context::enter_sequence_index
    #[allow(unused_variables)]
    #[inline(always)]
    fn leave_sequence_index(&mut self) {}
}
