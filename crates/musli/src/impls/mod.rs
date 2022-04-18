#[cfg(feature = "std")]
mod alloc;
#[cfg(feature = "std")]
mod net;
mod tuples;

use core::marker;
use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize, Wrapping,
};
use core::sync::atomic::{
    AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32,
    AtomicU64, AtomicU8, AtomicUsize,
};

use crate::de::{Decode, Decoder, VariantDecoder};
use crate::en::{Encode, Encoder, SequenceEncoder, VariantEncoder};
use crate::error::Error;

impl Encode for () {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.encode_unit()
    }
}

impl<'de> Decode<'de> for () {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_unit()
    }
}

impl<T> Encode for marker::PhantomData<T> {
    #[inline]
    fn encode<E>(&self, _: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        Ok(())
    }
}

impl<'de, T> Decode<'de> for marker::PhantomData<T> {
    fn decode<D>(_: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Ok(marker::PhantomData)
    }
}

macro_rules! atomic_impl {
    ($ty:ty) => {
        impl<'de> Decode<'de> for $ty {
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                Decode::decode(decoder).map(Self::new)
            }
        }
    };
}

atomic_impl!(AtomicBool);
atomic_impl!(AtomicI16);
atomic_impl!(AtomicI32);
atomic_impl!(AtomicI64);
atomic_impl!(AtomicI8);
atomic_impl!(AtomicIsize);
atomic_impl!(AtomicU16);
atomic_impl!(AtomicU32);
atomic_impl!(AtomicU64);
atomic_impl!(AtomicU8);
atomic_impl!(AtomicUsize);

macro_rules! non_zero {
    ($ty:ty) => {
        impl Encode for $ty {
            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
            where
                E: Encoder,
            {
                Encode::encode(&self.get(), encoder)
            }
        }

        impl<'de> Decode<'de> for $ty {
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                match Self::new(Decode::decode(decoder)?) {
                    Some(value) => Ok(value),
                    None => Err(D::Error::invalid_value(stringify!($ty))),
                }
            }
        }
    };
}

non_zero!(NonZeroI128);
non_zero!(NonZeroI16);
non_zero!(NonZeroI32);
non_zero!(NonZeroI64);
non_zero!(NonZeroI8);
non_zero!(NonZeroIsize);
non_zero!(NonZeroU128);
non_zero!(NonZeroU16);
non_zero!(NonZeroU32);
non_zero!(NonZeroU64);
non_zero!(NonZeroU8);
non_zero!(NonZeroUsize);

impl<const N: usize> Encode for [u8; N] {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.encode_array(*self)
    }
}

impl<'de, const N: usize> Decode<'de> for [u8; N] {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_array()
    }
}

macro_rules! impl_number {
    ($ty:ty, $read:ident, $write:ident) => {
        impl Encode for $ty {
            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
            where
                E: Encoder,
            {
                encoder.$write(*self)
            }
        }

        impl<'de> Decode<'de> for $ty {
            #[inline]
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                decoder.$read()
            }
        }
    };
}

impl Encode for bool {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.encode_bool(*self)
    }
}

impl<'de> Decode<'de> for bool {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_bool()
    }
}

impl Encode for char {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.encode_char(*self)
    }
}

impl<'de> Decode<'de> for char {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_char()
    }
}

impl_number!(usize, decode_usize, encode_usize);
impl_number!(isize, decode_isize, encode_isize);
impl_number!(u8, decode_u8, encode_u8);
impl_number!(u16, decode_u16, encode_u16);
impl_number!(u32, decode_u32, encode_u32);
impl_number!(u64, decode_u64, encode_u64);
impl_number!(u128, decode_u128, encode_u128);
impl_number!(i8, decode_i8, encode_i8);
impl_number!(i16, decode_i16, encode_i16);
impl_number!(i32, decode_i32, encode_i32);
impl_number!(i64, decode_i64, encode_i64);
impl_number!(i128, decode_i128, encode_i128);
impl_number!(f32, decode_f32, encode_f32);
impl_number!(f64, decode_f64, encode_f64);

impl Encode for str {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.encode_str(self)
    }
}

impl<'de> Decode<'de> for &'de str {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_str()
    }
}

impl<T> Encode for [T]
where
    T: Encode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        let mut seq = encoder.encode_sequence(self.len())?;

        for value in self {
            let encoder = seq.encode_next()?;
            T::encode(value, encoder)?;
        }

        seq.finish()
    }
}

impl<'de> Decode<'de> for &'de [u8] {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_bytes()
    }
}

impl<T> Encode for Option<T>
where
    T: Encode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        match self {
            Some(value) => {
                let encoder = encoder.encode_some()?;
                value.encode(encoder)
            }
            None => encoder.encode_none(),
        }
    }
}

impl<'de, T> Decode<'de> for Option<T>
where
    T: Decode<'de>,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        if let Some(decoder) = decoder.decode_option()? {
            Ok(Some(T::decode(decoder)?))
        } else {
            Ok(None)
        }
    }
}

impl<T, U> Encode for Result<T, U>
where
    T: Encode,
    U: Encode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        let mut variant = encoder.encode_variant()?;

        match self {
            Ok(ok) => {
                Encode::encode(&0usize, variant.encode_variant_tag()?)?;
                Encode::encode(ok, variant.encode_variant_value()?)?;
                Ok(())
            }
            Err(err) => {
                Encode::encode(&1usize, variant.encode_variant_tag()?)?;
                Encode::encode(err, variant.encode_variant_value()?)?;
                Ok(())
            }
        }
    }
}

impl<'de, T, U> Decode<'de> for Result<T, U>
where
    T: Decode<'de>,
    U: Decode<'de>,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut variant = decoder.decode_variant()?;

        match variant.decode_variant_tag().and_then(usize::decode)? {
            0 => variant.decode_variant_value().and_then(T::decode).map(Ok),
            1 => variant.decode_variant_value().and_then(U::decode).map(Err),
            tag => Err(D::Error::unsupported_variant("Result", tag)),
        }
    }
}

impl<T> Encode for Wrapping<T>
where
    T: Encode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        Encode::encode(&self.0, encoder)
    }
}

impl<'de, T> Decode<'de> for Wrapping<T>
where
    T: Decode<'de>,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Ok(Wrapping(Decode::decode(decoder)?))
    }
}
