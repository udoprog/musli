#[cfg(feature = "std")]
mod alloc;
#[cfg(feature = "std")]
mod net;
mod tuples;

use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize, Wrapping,
};
use core::sync::atomic::{
    AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32,
    AtomicU64, AtomicU8, AtomicUsize,
};
use core::{fmt, marker};

use crate::de::{Decode, Decoder, ValueVisitor, VariantDecoder};
use crate::en::{Encode, Encoder, VariantEncoder};
use crate::error::Error;

impl<Mode> Encode<Mode> for () {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_unit()
    }
}

impl<'de, Mode> Decode<'de, Mode> for () {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_unit()
    }
}

impl<T, Mode> Encode<Mode> for marker::PhantomData<T> {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_unit()
    }
}

impl<'de, Mode, T> Decode<'de, Mode> for marker::PhantomData<T> {
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let () = decoder.decode_unit()?;
        Ok(marker::PhantomData)
    }
}

macro_rules! atomic_impl {
    ($ty:ty) => {
        impl<'de, Mode> Decode<'de, Mode> for $ty {
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                Decode::<Mode>::decode(decoder).map(Self::new)
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
        impl<Mode> Encode<Mode> for $ty {
            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder,
            {
                Encode::<Mode>::encode(&self.get(), encoder)
            }
        }

        impl<'de, Mode> Decode<'de, Mode> for $ty {
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                let value = Decode::<Mode>::decode(decoder)?;

                match Self::new(value) {
                    Some(value) => Ok(value),
                    None => Err(D::Error::message(NonZeroUnsupportedValue {
                        type_name: stringify!($ty),
                        value,
                    })),
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

struct NonZeroUnsupportedValue<T> {
    type_name: &'static str,
    value: T,
}

impl<T> fmt::Display for NonZeroUnsupportedValue<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: unsupported non-zero value `{}`",
            self.type_name, self.value
        )
    }
}

impl<Mode, const N: usize> Encode<Mode> for [u8; N] {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_array(*self)
    }
}

impl<'de, Mode, const N: usize> Decode<'de, Mode> for [u8; N] {
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
        impl<Mode> Encode<Mode> for $ty {
            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder,
            {
                encoder.$write(*self)
            }
        }

        impl<'de, Mode> Decode<'de, Mode> for $ty {
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

impl<Mode> Encode<Mode> for bool {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_bool(*self)
    }
}

impl<'de, Mode> Decode<'de, Mode> for bool {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_bool()
    }
}

impl<Mode> Encode<Mode> for char {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_char(*self)
    }
}

impl<'de, Mode> Decode<'de, Mode> for char {
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

impl<Mode> Encode<Mode> for str {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_string(self)
    }
}

impl<'de, Mode> Decode<'de, Mode> for &'de str {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        return decoder.decode_string(Visitor(marker::PhantomData));

        struct Visitor<E>(marker::PhantomData<E>);

        impl<'de, E> ValueVisitor<'de> for Visitor<E>
        where
            E: Error,
        {
            type Target = str;
            type Ok = &'de str;
            type Error = E;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "string borrowed from source")
            }

            #[inline]
            fn visit_borrowed(self, string: &'de str) -> Result<Self::Ok, Self::Error> {
                Ok(string)
            }
        }
    }
}

impl<Mode> Encode<Mode> for [u8] {
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_bytes(self)
    }
}

impl<'de, Mode> Decode<'de, Mode> for &'de [u8] {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        return decoder.decode_bytes(Visitor(marker::PhantomData));

        struct Visitor<E>(marker::PhantomData<E>);

        impl<'de, E> ValueVisitor<'de> for Visitor<E>
        where
            E: Error,
        {
            type Target = [u8];
            type Ok = &'de [u8];
            type Error = E;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes borrowed from source")
            }

            #[inline]
            fn visit_borrowed(self, bytes: &'de [u8]) -> Result<Self::Ok, Self::Error> {
                Ok(bytes)
            }
        }
    }
}

impl<T, Mode> Encode<Mode> for Option<T>
where
    T: Encode<Mode>,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        match self {
            Some(value) => encoder
                .encode_some()
                .and_then(|encoder| value.encode(encoder)),
            None => encoder.encode_none(),
        }
    }
}

impl<'de, Mode, T> Decode<'de, Mode> for Option<T>
where
    T: Decode<'de, Mode>,
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

impl<T, U, Mode> Encode<Mode> for Result<T, U>
where
    T: Encode<Mode>,
    U: Encode<Mode>,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        let variant = encoder.encode_variant()?;

        match self {
            Ok(ok) => variant.insert::<Mode, _, _>(0usize, ok),
            Err(err) => variant.insert::<Mode, _, _>(1usize, err),
        }
    }
}

impl<'de, Mode, T, U> Decode<'de, Mode> for Result<T, U>
where
    T: Decode<'de, Mode>,
    U: Decode<'de, Mode>,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut variant = decoder.decode_variant()?;

        let this = match variant.tag().and_then(<usize as Decode<Mode>>::decode)? {
            0 => Ok(variant.variant().and_then(T::decode)?),
            1 => Err(variant.variant().and_then(U::decode)?),
            tag => return Err(D::Error::invalid_variant_tag("Result", tag)),
        };

        variant.end()?;
        Ok(this)
    }
}

impl<T, Mode> Encode<Mode> for Wrapping<T>
where
    T: Encode<Mode>,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        Encode::encode(&self.0, encoder)
    }
}

impl<'de, Mode, T> Decode<'de, Mode> for Wrapping<T>
where
    T: Decode<'de, Mode>,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Ok(Wrapping(Decode::<Mode>::decode(decoder)?))
    }
}
