//! Module that provides a never type which conveniently implements all the
//! encoder and decoder traits so that it can be used as a placeholder.
//!
//! This is a private module of musli, and is not intended for use outside of
//! the implementation attributes:
//!
//! * [`#[musli::encoder]`][crate::encoder].
//! * [`#[musli::decoder]`][crate::decoder].

use core::convert::Infallible;
use core::fmt;
use core::marker;

use crate::no_std::ToOwned;

use crate::de::{
    AsDecoder, Decoder, NumberVisitor, PackDecoder, PairDecoder, PairsDecoder, SequenceDecoder,
    SizeHint, ValueVisitor, VariantDecoder,
};
use crate::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder, VariantEncoder};
use crate::error::Error;

enum NeverMarker {}

/// An uninhabitable never type which implements all possible encoders and
/// decoders. This can be used if your [Encoder] implementation doesn't
/// implement a particular function.
///
/// ```
/// use std::fmt;
///
/// use musli::de::Decoder;
///
/// struct MyDecoder;
///
/// #[musli::decoder]
/// impl Decoder<'_> for MyDecoder {
///     type Error = String;
///
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "32-bit unsigned integers")
///     }
///
///     fn decode_u32(self) -> Result<u32, Self::Error> {
///         Ok(42)
///     }
/// }
/// ```
pub struct Never<A, B = Infallible, C: ?Sized = Infallible> {
    // Field makes type uninhabitable.
    _never: NeverMarker,
    _marker: marker::PhantomData<(A, B, C)>,
}

impl<'de, E> Decoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;
    type Buffer = Self;
    type Pack = Self;
    type Sequence = Self;
    type Tuple = Self;
    type Map = Self;
    type Some = Self;
    type Struct = Self;
    type Variant = Self;
    type __UseMusliDecoderAttributeMacro = ();

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<E> AsDecoder for Never<E>
where
    E: Error,
{
    type Error = E;

    type Decoder<'this> = Never<E>
    where
        Self: 'this;

    #[inline]
    fn as_decoder(&self) -> Result<Self::Decoder<'_>, Self::Error> {
        match self._never {}
    }
}

impl<'de, E> PairDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type First<'this> = Self
    where
        Self: 'this;

    type Second = Self;

    #[inline]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn second(self) -> Result<Self::Second, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn skip_second(self) -> Result<bool, Self::Error> {
        match self._never {}
    }
}

impl<'de, E> VariantDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type Tag<'this> = Self
    where
        Self: 'this;

    type Variant<'this> = Self where Self: 'this;

    #[inline]
    fn tag(&mut self) -> Result<Self::Tag<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn variant(&mut self) -> Result<Self::Variant<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn skip_variant(&mut self) -> Result<bool, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}

impl<'de, E> PairsDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type Decoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        match self._never {}
    }

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Decoder<'_>>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}

impl<'de, E> SequenceDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type Decoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        match self._never {}
    }

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Decoder<'_>>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}

impl<'de, E> PackDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type Decoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Decoder<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}

impl<O, E> Encoder for Never<O, E>
where
    E: Error,
{
    type Ok = O;
    type Error = E;
    type Pack = Self;
    type Some = Self;
    type Sequence = Self;
    type Tuple = Self;
    type Map = Self;
    type Struct = Self;
    type Variant = Self;
    type __UseMusliEncoderAttributeMacro = ();

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<'de, O, E> NumberVisitor<'de> for Never<O, E>
where
    E: Error,
{
    type Ok = O;
    type Error = E;

    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<'de, O, E, T> ValueVisitor<'de> for Never<O, E, T>
where
    T: ?Sized + ToOwned,
    E: Error,
{
    type Target = T;
    type Ok = O;
    type Error = E;

    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<O, E> SequenceEncoder for Never<O, E>
where
    E: Error,
{
    type Ok = O;
    type Error = E;

    type Encoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self._never {}
    }
}

impl<O, E> PairsEncoder for Never<O, E>
where
    E: Error,
{
    type Ok = O;
    type Error = E;
    type Encoder<'this> = Self where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        match self._never {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self._never {}
    }
}

impl<O, E> PairEncoder for Never<O, E>
where
    E: Error,
{
    type Ok = O;
    type Error = E;
    type First<'this> = Self
    where
        Self: 'this;
    type Second<'this> = Self where Self: 'this;

    #[inline]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn second(&mut self) -> Result<Self::Second<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self._never {}
    }
}

impl<O, E> VariantEncoder for Never<O, E>
where
    E: Error,
{
    type Ok = O;
    type Error = E;
    type Tag<'this> = Self
    where
        Self: 'this;
    type Variant<'this> = Self where Self: 'this;

    #[inline]
    fn tag(&mut self) -> Result<Self::Tag<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn variant(&mut self) -> Result<Self::Variant<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self._never {}
    }
}
