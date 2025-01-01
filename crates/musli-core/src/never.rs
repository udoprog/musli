//! Module that provides a never type which conveniently implements all the
//! encoder and decoder traits so that it can be used as a placeholder.
//!
//! This is a private module of musli, and is not intended for use outside of
//! the implementation attributes:
//!
//! * [`#[musli::encoder]`][crate::encoder].
//! * [`#[musli::decoder]`][crate::decoder].

use core::fmt;
use core::marker;

use crate::no_std::ToOwned;

use crate::alloc::RawVec;
use crate::de::{
    AsDecoder, Decode, DecodeUnsized, DecodeUnsizedBytes, Decoder, EntriesDecoder, EntryDecoder,
    MapDecoder, SequenceDecoder, SizeHint, UnsizedVisitor, VariantDecoder,
};
use crate::en::{
    Encode, Encoder, EntriesEncoder, EntryEncoder, MapEncoder, SequenceEncoder, VariantEncoder,
};
use crate::Context;

/// Marker type used for the [`Never`] type.
#[doc(hidden)]
pub enum NeverMarker {}

/// An uninhabitable never type which implements all possible encoders and
/// decoders. This can be used if your [Encoder] implementation doesn't
/// implement a particular function.
///
/// ```
/// use std::fmt;
///
/// use musli::Context;
/// use musli::de::{Decoder, Decode};
///
/// struct MyDecoder<C> {
///     cx: C,
///     number: u32,
/// }
///
/// #[musli::decoder]
/// impl<'de, C> Decoder<'de> for MyDecoder<C>
/// where
///     C: Context,
/// {
///     type Cx = C;
///
///     #[inline]
///     fn cx(&self) -> Self::Cx {
///         self.cx
///     }
///
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "32-bit unsigned integers")
///     }
///
///     fn decode_u32(self) -> Result<u32, C::Error> {
///         if self.number == 42 {
///             return Ok(self.number);
///         }
///
///         Err(self.cx.message("I do not have the answer..."))
///     }
/// }
/// ```
pub struct Never<A = NeverMarker, B: ?Sized = NeverMarker> {
    // Field makes type uninhabitable.
    _never: NeverMarker,
    _marker: marker::PhantomData<(A, B)>,
}

impl<T> RawVec<T> for Never<T> {
    #[inline]
    fn resize(&mut self, _: usize, _: usize) -> bool {
        match self._never {}
    }

    #[inline]
    fn as_ptr(&self) -> *const T {
        match self._never {}
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T {
        match self._never {}
    }

    #[inline]
    fn try_merge<B>(&mut self, _: usize, _: B, _: usize) -> Result<(), B>
    where
        B: RawVec<T>,
    {
        match self._never {}
    }
}

impl<'de, C> Decoder<'de> for Never<(), C>
where
    C: Context,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = C::Mode;
    type WithContext<U>
        = Never<(), U>
    where
        U: Context;
    type DecodeBuffer = Self;
    type DecodePack = Self;
    type DecodeSequence = Self;
    type DecodeMapEntries = Self;
    type DecodeSome = Self;
    type DecodeMap = Self;
    type DecodeVariant = Self;
    type __UseMusliDecoderAttributeMacro = ();

    #[inline]
    fn cx(&self) -> Self::Cx {
        match self._never {}
    }

    #[inline]
    fn with_context<U>(self, _: U) -> Result<Self::WithContext<U>, C::Error>
    where
        U: Context,
    {
        match self._never {}
    }

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }

    #[inline]
    fn decode<T>(self) -> Result<T, Self::Error>
    where
        T: Decode<'de, Self::Mode>,
    {
        match self._never {}
    }

    #[inline]
    fn decode_unsized<T, F, O>(self, _: F) -> Result<O, Self::Error>
    where
        T: ?Sized + DecodeUnsized<'de, Self::Mode>,
        F: FnOnce(&T) -> Result<O, Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn decode_unsized_bytes<T, F, O>(self, _: F) -> Result<O, Self::Error>
    where
        T: ?Sized + DecodeUnsizedBytes<'de, Self::Mode>,
        F: FnOnce(&T) -> Result<O, Self::Error>,
    {
        match self._never {}
    }
}

impl<C> AsDecoder for Never<(), C>
where
    C: Context,
{
    type Cx = C;
    type Decoder<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn as_decoder(&self) -> Result<Self::Decoder<'_>, C::Error> {
        match self._never {}
    }
}

impl<C> EntriesDecoder<'_> for Never<(), C>
where
    C: Context,
{
    type Cx = C;
    type DecodeEntryKey<'this>
        = Self
    where
        Self: 'this;
    type DecodeEntryValue<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn decode_entry_key(&mut self) -> Result<Option<Self::DecodeEntryKey<'_>>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_entry_value(&mut self) -> Result<Self::DecodeEntryValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end_entries(self) -> Result<(), C::Error> {
        match self._never {}
    }
}

impl<C> VariantDecoder<'_> for Never<(), C>
where
    C: Context,
{
    type Cx = C;
    type DecodeTag<'this>
        = Self
    where
        Self: 'this;
    type DecodeValue<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeValue<'_>, C::Error> {
        match self._never {}
    }
}

impl<C> MapDecoder<'_> for Never<(), C>
where
    C: Context,
{
    type Cx = C;
    type DecodeEntry<'this>
        = Self
    where
        Self: 'this;
    type DecodeRemainingEntries<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        match self._never {}
    }

    #[inline]
    fn decode_entry(&mut self) -> Result<Option<Self::DecodeEntry<'_>>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_remaining_entries(
        &mut self,
    ) -> Result<Self::DecodeRemainingEntries<'_>, <Self::Cx as Context>::Error> {
        match self._never {}
    }
}

impl<C> EntryDecoder<'_> for Never<(), C>
where
    C: Context,
{
    type Cx = C;
    type DecodeKey<'this>
        = Self
    where
        Self: 'this;
    type DecodeValue = Self;

    #[inline]
    fn decode_key(&mut self) -> Result<Self::DecodeKey<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_value(self) -> Result<Self::DecodeValue, C::Error> {
        match self._never {}
    }
}

impl<C> SequenceDecoder<'_> for Never<(), C>
where
    C: Context,
{
    type Cx = C;
    type DecodeNext<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn try_decode_next(&mut self) -> Result<Option<Self::DecodeNext<'_>>, C::Error> {
        match self._never {}
    }
}

impl<C, A> Encoder for Never<A, C>
where
    C: Context,
    A: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Ok = A;
    type Mode = C::Mode;
    type WithContext<U>
        = Never<A, U>
    where
        U: Context;
    type EncodePack = Self;
    type EncodeSome = Self;
    type EncodeSequence = Self;
    type EncodeMap = Self;
    type EncodeMapEntries = Self;
    type EncodeVariant = Self;
    type EncodeSequenceVariant = Self;
    type EncodeMapVariant = Self;
    type __UseMusliEncoderAttributeMacro = ();

    #[inline]
    fn cx(&self) -> Self::Cx {
        match self._never {}
    }

    #[inline]
    fn with_context<U>(self, _: U) -> Result<Self::WithContext<U>, C::Error>
    where
        U: Context,
    {
        match self._never {}
    }

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }

    #[inline]
    fn encode<T>(self, _: T) -> Result<Self::Ok, C::Error>
    where
        T: Encode<Self::Mode>,
    {
        match self._never {}
    }
}

impl<C, A, T> UnsizedVisitor<'_, C, T> for Never<A, T>
where
    C: Context,
    T: ?Sized + ToOwned,
{
    type Ok = A;

    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<A, C> SequenceEncoder for Never<A, C>
where
    A: 'static,
    C: Context,
{
    type Cx = C;
    type Ok = A;
    type EncodeNext<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        match self._never {}
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn finish_sequence(self) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<A, C> MapEncoder for Never<A, C>
where
    A: 'static,
    C: Context,
{
    type Cx = C;
    type Ok = A;
    type EncodeEntry<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, C::Error> {
        match self._never {}
    }

    fn finish_map(self) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<A, C> EntryEncoder for Never<A, C>
where
    A: 'static,
    C: Context,
{
    type Cx = C;
    type Ok = A;
    type EncodeKey<'this>
        = Self
    where
        Self: 'this;
    type EncodeValue<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn encode_key(&mut self) -> Result<Self::EncodeKey<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn finish_entry(self) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<A, C> EntriesEncoder for Never<A, C>
where
    A: 'static,
    C: Context,
{
    type Cx = C;
    type Ok = A;
    type EncodeEntryKey<'this>
        = Self
    where
        Self: 'this;
    type EncodeEntryValue<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn encode_entry_key(&mut self) -> Result<Self::EncodeEntryKey<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn encode_entry_value(&mut self) -> Result<Self::EncodeEntryValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn finish_entries(self) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<A, C> VariantEncoder for Never<A, C>
where
    A: 'static,
    C: Context,
{
    type Cx = C;
    type Ok = A;
    type EncodeTag<'this>
        = Self
    where
        Self: 'this;
    type EncodeData<'this>
        = Self
    where
        Self: 'this;

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn encode_data(&mut self) -> Result<Self::EncodeData<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn finish_variant(self) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}
