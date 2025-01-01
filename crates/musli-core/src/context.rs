//! Things related to working with contexts.

use core::error::Error;
use core::fmt;
use core::str;

use crate::alloc::Allocator;

/// Provides ergonomic access to the serialization context.
///
/// This is used to among other things report diagnostics.
pub trait Context: Copy {
    /// Mode of the context.
    type Mode: 'static;
    /// Error produced by context.
    type Error;
    /// A mark during processing.
    type Mark;
    /// The allocator associated with the context.
    type Allocator: Allocator;
    /// An allocated buffer containing a valid string.
    type String: AsRef<str>;

    /// Clear the state of the context, allowing it to be re-used.
    fn clear(self);

    /// Advance the context by `n` bytes of input.
    ///
    /// This is typically used to move the mark forward as produced by
    /// [Context::mark].
    fn advance(self, n: usize);

    /// Return a mark which acts as a checkpoint at the current encoding state.
    ///
    /// The context is in a privileged state in that it sees everything, so a
    /// mark can be quite useful for determining the context of an error.
    ///
    /// This typically indicates a byte offset, and is used by
    /// [`marked_message`][Context::marked_message] to report a spanned error.
    fn mark(self) -> Self::Mark;

    /// Access the underlying allocator.
    fn alloc(self) -> Self::Allocator;

    /// Collect and allocate a string from a [`Display`] implementation.
    ///
    /// [`Display`]: fmt::Display
    fn collect_string<T>(self, value: &T) -> Result<Self::String, Self::Error>
    where
        T: ?Sized + fmt::Display;

    /// Generate a map function which maps an error using the `custom` function.
    #[inline]
    fn map<T>(self) -> impl FnOnce(T) -> Self::Error
    where
        T: 'static + Send + Sync + Error,
    {
        move |error| self.custom(error)
    }

    /// Report a custom error, which is not encapsulated by the error type
    /// expected by the context. This is essentially a type-erased way of
    /// reporting error-like things out from the context.
    fn custom<T>(self, error: T) -> Self::Error
    where
        T: 'static + Send + Sync + Error;

    /// Report a message as an error.
    ///
    /// This is made available to format custom error messages in `no_std`
    /// environments. The error message is to be collected by formatting `T`.
    fn message<T>(self, message: T) -> Self::Error
    where
        T: fmt::Display;

    /// Report an error based on a mark.
    ///
    /// A mark is generated using [Context::mark] and indicates a prior state.
    #[allow(unused_variables)]
    #[inline(always)]
    fn marked_message<T>(self, mark: &Self::Mark, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.message(message)
    }

    /// Report an error based on a mark.
    ///
    /// A mark is generated using [Context::mark] and indicates a prior state.
    #[allow(unused_variables)]
    #[inline(always)]
    fn marked_custom<T>(self, mark: &Self::Mark, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + Error,
    {
        self.custom(message)
    }

    /// Report that an invalid variant tag was encountered.
    #[inline(always)]
    fn invalid_variant_tag<T>(self, type_name: &'static str, tag: &T) -> Self::Error
    where
        T: ?Sized + fmt::Debug,
    {
        self.message(format_args!(
            "Type {type_name} received invalid variant tag {tag:?}"
        ))
    }

    /// The value for the given tag could not be collected.
    #[inline(always)]
    fn expected_tag<T>(self, type_name: &'static str, tag: &T) -> Self::Error
    where
        T: ?Sized + fmt::Debug,
    {
        self.message(format_args!("Type {type_name} expected tag {tag:?}"))
    }

    /// Trying to decode an uninhabitable type.
    #[inline(always)]
    fn uninhabitable(self, type_name: &'static str) -> Self::Error {
        self.message(format_args!(
            "Type {type_name} cannot be decoded since it's uninhabitable"
        ))
    }

    /// Encountered an unsupported field tag.
    #[inline(always)]
    fn invalid_field_tag<T>(self, type_name: &'static str, tag: &T) -> Self::Error
    where
        T: ?Sized + fmt::Debug,
    {
        self.message(format_args!(
            "Type {type_name} is missing invalid field tag {tag:?}"
        ))
    }

    /// Expected another field to decode.
    #[inline(always)]
    fn expected_field_adjacent<T, C>(
        self,
        type_name: &'static str,
        tag: &T,
        content: &C,
    ) -> Self::Error
    where
        T: ?Sized + fmt::Debug,
        C: ?Sized + fmt::Debug,
    {
        self.message(format_args!(
            "Type {type_name} expected adjacent field {tag:?} or {content:?}"
        ))
    }

    /// Missing adjacent tag when decoding.
    #[inline(always)]
    fn missing_adjacent_tag<T>(self, type_name: &'static str, tag: &T) -> Self::Error
    where
        T: ?Sized + fmt::Debug,
    {
        self.message(format_args!(
            "Type {type_name} is missing adjacent tag {tag:?}"
        ))
    }

    /// Encountered an unsupported field tag.
    #[inline(always)]
    fn invalid_field_string_tag(self, type_name: &'static str, field: Self::String) -> Self::Error {
        let field = field.as_ref();

        self.message(format_args!(
            "Type {type_name} received invalid field tag {field:?}"
        ))
    }

    /// Missing variant field required to decode.
    #[inline(always)]
    fn missing_variant_field<T>(self, type_name: &'static str, tag: &T) -> Self::Error
    where
        T: ?Sized + fmt::Debug,
    {
        self.message(format_args!(
            "Type {type_name} is missing variant field {tag:?}"
        ))
    }

    /// Indicate that a variant tag could not be determined.
    #[inline(always)]
    fn missing_variant_tag(self, type_name: &'static str) -> Self::Error {
        self.message(format_args!("Type {type_name} is missing variant tag"))
    }

    /// Encountered an unsupported variant field.
    #[inline(always)]
    fn invalid_variant_field_tag<V, T>(
        self,
        type_name: &'static str,
        variant: &V,
        tag: &T,
    ) -> Self::Error
    where
        V: ?Sized + fmt::Debug,
        T: ?Sized + fmt::Debug,
    {
        self.message(format_args!(
            "Type {type_name} received invalid variant field tag {tag:?} for variant {variant:?}",
        ))
    }

    /// Missing variant field required to decode.
    #[inline(always)]
    fn alloc_failed(self) -> Self::Error {
        self.message("Failed to allocate")
    }

    /// Indicate that we've entered a struct with the given `name`.
    ///
    /// The `name` variable corresponds to the identifiers of the struct.
    ///
    /// This will be matched with a corresponding call to [`leave_struct`].
    ///
    /// [`leave_struct`]: Context::leave_struct
    #[inline(always)]
    fn enter_struct(self, type_name: &'static str) {
        _ = type_name;
    }

    /// Trace that we've left the last struct that was entered.
    #[inline(always)]
    fn leave_struct(self) {}

    /// Indicate that we've entered an enum with the given `name`.
    ///
    /// The `name` variable corresponds to the identifiers of the enum.
    ///
    /// This will be matched with a corresponding call to [`leave_enum`].
    ///
    /// [`leave_enum`]: Context::leave_enum
    #[inline(always)]
    fn enter_enum(self, type_name: &'static str) {
        _ = type_name;
    }

    /// Trace that we've left the last enum that was entered.
    #[inline(always)]
    fn leave_enum(self) {}

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
    /// ```
    /// use musli::{Decode, Encode};
    ///
    /// #[derive(Decode, Encode)]
    /// #[musli(name_all = "name")]
    /// struct Struct {
    ///     #[musli(name = "string")]
    ///     field: String,
    /// }
    /// ```
    ///
    /// [`leave_field`]: Context::leave_field
    #[inline(always)]
    fn enter_named_field<T>(self, type_name: &'static str, tag: &T)
    where
        T: ?Sized + fmt::Display,
    {
        _ = type_name;
        _ = tag;
    }

    /// Trace that we've entered the given unnamed field.
    ///
    /// An unnamed field is part of a tuple struct, where the field index is the
    /// `index` argument below, and the musli tag being used for the field is
    /// the second argument.
    ///
    /// This will be matched with a corresponding call to [`leave_field`].
    ///
    /// Here `index` is `0` and `name` is `"string"`.
    ///
    /// ```
    /// use musli::{Decode, Encode};
    ///
    /// #[derive(Decode, Encode)]
    /// #[musli(name_all = "name")]
    /// struct Struct(#[musli(name = "string")] String);
    /// ```
    ///
    /// [`leave_field`]: Context::leave_field
    #[inline(always)]
    fn enter_unnamed_field<T>(self, index: u32, name: &T)
    where
        T: ?Sized + fmt::Display,
    {
        _ = index;
        _ = name;
    }

    /// Trace that we've left the last field that was entered.
    ///
    /// The `marker` argument will be the same as the one returned from
    /// [`enter_named_field`] or [`enter_unnamed_field`].
    ///
    /// [`enter_named_field`]: Context::enter_named_field
    /// [`enter_unnamed_field`]: Context::enter_unnamed_field
    #[inline(always)]
    fn leave_field(self) {}

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
    /// ```
    /// use musli::{Decode, Encode};
    ///
    /// #[derive(Decode, Encode)]
    /// #[musli(name_all = "name")]
    /// struct Struct {
    ///     #[musli(name = "string")]
    ///     field: String,
    /// }
    /// ```
    ///
    /// [`leave_variant`]: Context::leave_variant
    #[inline(always)]
    fn enter_variant<T>(self, type_name: &'static str, tag: T)
    where
        T: fmt::Display,
    {
        _ = type_name;
        _ = tag;
    }

    /// Trace that we've left the last variant that was entered.
    ///
    /// The `marker` argument will be the same as the one returned from
    /// [`enter_variant`].
    ///
    /// [`enter_variant`]: Context::enter_variant
    #[inline(always)]
    fn leave_variant(self) {}

    /// Trace a that a map key has been entered.
    #[inline(always)]
    fn enter_map_key<T>(self, field: T)
    where
        T: fmt::Display,
    {
        _ = field;
    }

    /// Trace that we've left the last map field that was entered.
    ///
    /// The `marker` argument will be the same as the one returned from
    /// [`enter_map_key`].
    ///
    /// [`enter_map_key`]: Context::enter_map_key
    #[allow(unused_variables)]
    #[inline(always)]
    fn leave_map_key(self) {}

    /// Trace a sequence field.
    #[allow(unused_variables)]
    #[inline(always)]
    fn enter_sequence_index(self, index: usize) {}

    /// Trace that we've left the last sequence index that was entered.
    ///
    /// The `marker` argument will be the same as the one returned from
    /// [`enter_sequence_index`].
    ///
    /// [`enter_sequence_index`]: Context::enter_sequence_index
    #[allow(unused_variables)]
    #[inline(always)]
    fn leave_sequence_index(self) {}
}
