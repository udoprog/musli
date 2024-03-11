//! Things related to working with contexts.

mod error;

use core::fmt;
use core::marker;

#[doc(inline)]
pub use self::error::Error;

/// A buffer allocated from a context.
///
/// Buffers are allocated from a context through [`Context::alloc`].
pub trait Buf {
    /// Write the given number of bytes.
    ///
    /// Returns `true` if the bytes could be successfully written. A `false`
    /// value indicates that we are out of buffer capacity.
    fn write(&mut self, bytes: &[u8]) -> bool;

    /// Write a buffer of the same type onto the current buffer.
    ///
    /// This allows allocators to provide more efficient means of extending the
    /// current buffer with one provided from the same allocator.
    fn write_buffer(&mut self, other: Self) -> bool
    where
        Self: Sized,
    {
        self.write(other.as_slice())
    }

    /// Write a single byte.
    ///
    /// Returns `true` if the bytes could be successfully written. A `false`
    /// value indicates that we are out of buffer capacity.
    #[inline(always)]
    fn push(&mut self, byte: u8) -> bool {
        self.write(&[byte])
    }

    /// Get the length of the buffer in bytes.
    fn len(&self) -> usize;

    /// Test if the buffer is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the buffer as its initialized slice.
    fn as_slice(&self) -> &[u8];
}

impl Buf for [u8] {
    #[inline(always)]
    fn write(&mut self, _: &[u8]) -> bool {
        false
    }

    #[inline(always)]
    fn len(&self) -> usize {
        <[_]>::len(self)
    }

    #[inline(always)]
    fn as_slice(&self) -> &[u8] {
        self
    }
}

/// Provides ergonomic access to the serialization context.
///
/// This is used to among other things report diagnostics.
pub trait Context {
    /// The error type which is collected by the context.
    type Input;
    /// Error produced by context.
    type Error: 'static;
    /// A mark during processing.
    type Mark: Copy + Default;
    /// A growable buffer.
    type Buf<'this>: Buf
    where
        Self: 'this;

    /// Allocate a buffer.
    fn alloc(&self) -> Option<Self::Buf<'_>>;

    /// Adapt the current context so that it can convert an error from a
    /// different type convertible to the current input
    ///
    /// This is necessary to support context error modules which consume an
    /// error kind that is not identical to the current [`Context::Input`], but
    /// can be converted to one.
    ///
    /// ```
    /// use musli::Context;
    ///
    /// struct Function1Error {
    ///     /* .. */
    /// }
    ///
    /// impl From<Function2Error> for Function1Error {
    ///     fn from(error: Function2Error) -> Self {
    ///         /* .. */
    ///         # Self {  }
    ///     }
    /// }
    ///
    /// fn function1<C>(cx: &C) -> Result<(), C::Error>
    /// where
    ///     C: Context<Input = Function1Error>
    /// {
    ///     function2(cx.adapt())
    /// }
    ///
    /// struct Function2Error {
    ///     /* .. */
    /// }
    ///
    /// // This function uses a different error as input.
    /// fn function2<C>(cx: &C) -> Result<(), C::Error>
    /// where
    ///     C: Context<Input = Function2Error>
    /// {
    ///     /* .. */
    ///     Ok(())
    /// }
    /// ```
    fn adapt<E>(&self) -> &Adapt<Self, E>
    where
        Self::Input: From<E>,
    {
        // SAFETY: adapter type is repr transparent.
        unsafe { &*(self as *const _ as *const _) }
    }

    /// Report the given context error.
    fn report<T>(&self, error: T) -> Self::Error
    where
        Self::Input: From<T>;

    /// Report a custom error, which is not encapsulated by the error type
    /// expected by the context. This is essentially a type-erased way of
    /// reporting error-like things out from the context.
    fn custom<T>(&self, error: T) -> Self::Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug;

    /// Report a message as an error.
    ///
    /// This is made available to format custom error messages in `no_std`
    /// environments. The error message is to be collected by formatting `T`.
    fn message<T>(&self, message: T) -> Self::Error
    where
        T: fmt::Display;

    /// Report the given encoding error from the given mark.
    #[allow(unused_variables)]
    #[inline(always)]
    fn marked_report<T>(&self, mark: Self::Mark, error: T) -> Self::Error
    where
        Self::Input: From<T>,
    {
        self.report(error)
    }

    /// Report an error based on a mark.
    ///
    /// A mark is generated using [Context::mark] and indicates a prior state.
    #[allow(unused_variables)]
    #[inline(always)]
    fn marked_message<T>(&self, mark: Self::Mark, message: T) -> Self::Error
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
    fn advance(&self, n: usize) {}

    /// Return a mark which acts as a checkpoint at the current encoding state.
    ///
    /// The context is in a privileged state in that it sees everything, so a
    /// mark can be quite useful for determining the context of an error.
    ///
    /// This typically indicates a byte offset, and is used by
    /// [`marked_message`][Context::marked_message] to report a spanned error.
    #[inline(always)]
    fn mark(&self) -> Self::Mark {
        Self::Mark::default()
    }

    /// Report that an invalid variant tag was encountered.
    #[inline(always)]
    fn invalid_variant_tag<T>(&self, _: &'static str, tag: T) -> Self::Error
    where
        T: fmt::Debug,
    {
        self.message(format_args!("Invalid variant tag: {tag:?}"))
    }

    /// The value for the given tag could not be collected.
    #[inline(always)]
    fn expected_tag<T>(&self, _: &'static str, tag: T) -> Self::Error
    where
        T: fmt::Debug,
    {
        self.message(format_args!("Expected tag: {tag:?}"))
    }

    /// Trying to decode an uninhabitable type.
    #[inline(always)]
    fn uninhabitable(&self, _: &'static str) -> Self::Error {
        self.message(format_args!("Cannot decode uninhabitable types"))
    }

    /// Encountered an unsupported field tag.
    #[inline(always)]
    fn invalid_field_tag<T>(&self, _: &'static str, tag: T) -> Self::Error
    where
        T: fmt::Debug,
    {
        self.message(format_args!("Invalid field tag: {tag:?}"))
    }

    /// Encountered an unsupported field tag.
    #[inline(always)]
    fn invalid_field_string_tag(&self, _: &'static str, field: Self::Buf<'_>) -> Self::Error {
        // SAFETY: Getting the slice does not overlap any interleaving operations.
        let bytes = field.as_slice();

        if let Ok(string) = core::str::from_utf8(bytes) {
            self.message(format_args!("Invalid field tag: {string}"))
        } else {
            self.message(format_args!("Invalid field tag"))
        }
    }

    /// Missing variant field required to decode.
    #[allow(unused_variables)]
    #[inline(always)]
    fn missing_variant_field<T>(&self, name: &'static str, tag: T) -> Self::Error
    where
        T: fmt::Debug,
    {
        self.message(format_args!("Missing variant field: {tag:?}"))
    }

    /// Indicate that a variant tag could not be determined.
    #[allow(unused_variables)]
    #[inline(always)]
    fn missing_variant_tag(&self, name: &'static str) -> Self::Error {
        self.message(format_args!("Missing variant tag"))
    }

    /// Encountered an unsupported variant field.
    #[allow(unused_variables)]
    #[inline(always)]
    fn invalid_variant_field_tag<V, T>(&self, name: &'static str, variant: V, tag: T) -> Self::Error
    where
        V: fmt::Debug,
        T: fmt::Debug,
    {
        self.message(format_args!(
            "invalid variant field tag: variant: {variant:?}, tag: {tag:?}",
        ))
    }

    /// Missing variant field required to decode.
    #[allow(unused_variables)]
    #[inline(always)]
    fn alloc_failed(&self) -> Self::Error {
        self.message("Failed to allocate")
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
    fn enter_struct(&self, name: &'static str) {}

    /// Trace that we've left the last struct that was entered.
    #[inline(always)]
    fn leave_struct(&self) {}

    /// Indicate that we've entered an enum with the given `name`.
    ///
    /// The `name` variable corresponds to the identifiers of the enum.
    ///
    /// This will be matched with a corresponding call to [`leave_enum`].
    ///
    /// [`leave_enum`]: Context::leave_enum
    #[allow(unused_variables)]
    #[inline(always)]
    fn enter_enum(&self, name: &'static str) {}

    /// Trace that we've left the last enum that was entered.
    #[inline(always)]
    fn leave_enum(&self) {}

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
    fn enter_named_field<T>(&self, name: &'static str, tag: T)
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
    fn enter_unnamed_field<T>(&self, index: u32, tag: T)
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
    fn leave_field(&self) {}

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
    fn enter_variant<T>(&self, name: &'static str, tag: T)
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
    fn leave_variant(&self) {}

    /// Trace a that a map key has been entered.
    #[allow(unused_variables)]
    #[inline(always)]
    fn enter_map_key<T>(&self, field: T)
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
    fn leave_map_key(&self) {}

    /// Trace a sequence field.
    #[allow(unused_variables)]
    #[inline(always)]
    fn enter_sequence_index(&self, index: usize) {}

    /// Trace that we've left the last sequence index that was entered.
    ///
    /// The `marker` argument will be the same as the one returned from
    /// [`enter_sequence_index`].
    ///
    /// [`enter_sequence_index`]: Context::enter_sequence_index
    #[allow(unused_variables)]
    #[inline(always)]
    fn leave_sequence_index(&self) {}
}

/// Context adaptor returned by [`Context::adapt`].
#[repr(transparent)]
pub struct Adapt<C, E>
where
    C: ?Sized,
{
    error: marker::PhantomData<E>,
    context: C,
}

impl<C, E> Context for Adapt<C, E>
where
    C: Context,
    C::Input: From<E>,
    E: 'static,
{
    type Input = E;
    type Error = C::Error;
    type Mark = C::Mark;
    type Buf<'this> = C::Buf<'this> where Self: 'this;

    #[inline(always)]
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        self.context.alloc()
    }

    #[inline(always)]
    fn report<T>(&self, error: T) -> Self::Error
    where
        Self::Input: From<T>,
    {
        self.context.report(Self::Input::from(error))
    }

    #[inline(always)]
    fn custom<T>(&self, error: T) -> Self::Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        self.context.custom(error)
    }

    #[inline(always)]
    fn message<T>(&self, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.context.message(message)
    }

    #[inline(always)]
    fn marked_report<T>(&self, mark: Self::Mark, error: T) -> Self::Error
    where
        Self::Input: From<T>,
    {
        self.context.marked_report(mark, E::from(error))
    }

    #[inline(always)]
    fn marked_message<T>(&self, mark: Self::Mark, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.context.marked_message(mark, message)
    }

    #[inline(always)]
    fn advance(&self, n: usize) {
        self.context.advance(n);
    }

    #[inline(always)]
    fn mark(&self) -> Self::Mark {
        self.context.mark()
    }

    #[inline(always)]
    fn invalid_variant_tag<T>(&self, name: &'static str, tag: T) -> Self::Error
    where
        T: fmt::Debug,
    {
        self.context.invalid_variant_tag(name, tag)
    }

    #[inline(always)]
    fn expected_tag<T>(&self, name: &'static str, tag: T) -> Self::Error
    where
        T: fmt::Debug,
    {
        self.context.expected_tag(name, tag)
    }

    #[inline(always)]
    fn uninhabitable(&self, name: &'static str) -> Self::Error {
        self.context.uninhabitable(name)
    }

    #[inline(always)]
    fn invalid_field_tag<T>(&self, name: &'static str, tag: T) -> Self::Error
    where
        T: fmt::Debug,
    {
        self.context.invalid_field_tag(name, tag)
    }

    #[inline(always)]
    fn invalid_field_string_tag(&self, name: &'static str, field: Self::Buf<'_>) -> Self::Error {
        self.context.invalid_field_string_tag(name, field)
    }

    #[inline(always)]
    fn missing_variant_field<T>(&self, name: &'static str, tag: T) -> Self::Error
    where
        T: fmt::Debug,
    {
        self.context.missing_variant_field(name, tag)
    }

    #[inline(always)]
    fn missing_variant_tag(&self, name: &'static str) -> Self::Error {
        self.context.missing_variant_tag(name)
    }

    #[inline(always)]
    fn invalid_variant_field_tag<V, T>(&self, name: &'static str, variant: V, tag: T) -> Self::Error
    where
        V: fmt::Debug,
        T: fmt::Debug,
    {
        self.context.invalid_variant_field_tag(name, variant, tag)
    }

    #[inline(always)]
    fn enter_struct(&self, name: &'static str) {
        self.context.enter_struct(name)
    }

    #[inline(always)]
    fn leave_struct(&self) {
        self.context.leave_struct()
    }

    #[inline(always)]
    fn enter_enum(&self, name: &'static str) {
        self.context.enter_enum(name)
    }

    #[inline(always)]
    fn leave_enum(&self) {
        self.context.leave_enum()
    }

    #[inline(always)]
    fn enter_named_field<T>(&self, name: &'static str, tag: T)
    where
        T: fmt::Display,
    {
        self.context.enter_named_field(name, tag)
    }

    #[inline(always)]
    fn enter_unnamed_field<T>(&self, index: u32, tag: T)
    where
        T: fmt::Display,
    {
        self.context.enter_unnamed_field(index, tag)
    }

    #[inline(always)]
    fn leave_field(&self) {
        self.context.leave_field()
    }

    #[inline(always)]
    fn enter_variant<T>(&self, name: &'static str, tag: T)
    where
        T: fmt::Display,
    {
        self.context.enter_variant(name, tag)
    }

    #[inline(always)]
    fn leave_variant(&self) {
        self.context.leave_variant()
    }

    #[inline(always)]
    fn enter_map_key<T>(&self, field: T)
    where
        T: fmt::Display,
    {
        self.context.enter_map_key(field)
    }

    #[inline(always)]
    fn leave_map_key(&self) {
        self.context.leave_map_key()
    }

    #[inline(always)]
    fn enter_sequence_index(&self, index: usize) {
        self.context.enter_sequence_index(index)
    }

    #[inline(always)]
    fn leave_sequence_index(&self) {
        self.context.leave_sequence_index()
    }
}
