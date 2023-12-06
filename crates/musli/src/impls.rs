#[cfg(feature = "alloc")]
mod alloc;
#[cfg(feature = "std")]
mod net;
mod tuples;

use core::ffi::CStr;
use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize, Wrapping,
};
use core::{fmt, marker};

use crate::de::{Decode, Decoder, ValueVisitor, VariantDecoder};
use crate::en::{Encode, Encoder, VariantEncoder};
use crate::mode::Mode;
use crate::Context;

impl<M> Encode<M> for ()
where
    M: Mode,
{
    #[inline]
    fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        encoder.encode_unit(cx)
    }
}

impl<'de, M> Decode<'de, M> for ()
where
    M: Mode,
{
    #[inline]
    fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        decoder.decode_unit(cx)
    }
}

impl<T, M> Encode<M> for marker::PhantomData<T>
where
    M: Mode,
{
    #[inline]
    fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        encoder.encode_unit(cx)
    }
}

impl<'de, M, T> Decode<'de, M> for marker::PhantomData<T>
where
    M: Mode,
{
    fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        decoder.decode_unit(cx)?;
        Ok(marker::PhantomData)
    }
}

macro_rules! atomic_impl {
    ($size:literal $(, $ty:ident)*) => {
        $(
            #[cfg(target_has_atomic = $size)]
            impl<'de, M> Decode<'de, M> for core::sync::atomic::$ty
            where
                M: Mode,
            {
                fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
                where
                    C: Context<Input = D::Error>,
                    D: Decoder<'de>,
                {
                    Decode::<M>::decode(cx, decoder).map(Self::new)
                }
            }
        )*
    };
}

atomic_impl!("8", AtomicBool, AtomicI8, AtomicU8);
atomic_impl!("16", AtomicI16, AtomicU16);
atomic_impl!("32", AtomicI32, AtomicU32);
atomic_impl!("64", AtomicI64, AtomicU64);
atomic_impl!("ptr", AtomicIsize, AtomicUsize);

macro_rules! non_zero {
    ($ty:ty) => {
        impl<M> Encode<M> for $ty
        where
            M: Mode,
        {
            #[inline]
            fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
            where
                C: Context<Input = E::Error>,
                E: Encoder,
            {
                Encode::<M>::encode(&self.get(), cx, encoder)
            }
        }

        impl<'de, M> Decode<'de, M> for $ty
        where
            M: Mode,
        {
            fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
            where
                C: Context<Input = D::Error>,
                D: Decoder<'de>,
            {
                let value = Decode::<M>::decode(cx, decoder)?;

                match Self::new(value) {
                    Some(value) => Ok(value),
                    None => Err(cx.message(NonZeroUnsupportedValue {
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

impl<M, const N: usize> Encode<M> for [u8; N]
where
    M: Mode,
{
    #[inline]
    fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        encoder.encode_array(cx, *self)
    }
}

impl<'de, M, const N: usize> Decode<'de, M> for [u8; N]
where
    M: Mode,
{
    #[inline]
    fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        decoder.decode_array(cx)
    }
}

macro_rules! impl_number {
    ($ty:ty, $read:ident, $write:ident) => {
        impl<M> Encode<M> for $ty
        where
            M: Mode,
        {
            #[inline]
            fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
            where
                C: Context<Input = E::Error>,
                E: Encoder,
            {
                encoder.$write(cx, *self)
            }
        }

        impl<'de, M> Decode<'de, M> for $ty
        where
            M: Mode,
        {
            #[inline]
            fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
            where
                C: Context<Input = D::Error>,
                D: Decoder<'de>,
            {
                decoder.$read(cx)
            }
        }
    };
}

impl<M> Encode<M> for bool
where
    M: Mode,
{
    #[inline]
    fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        encoder.encode_bool(cx, *self)
    }
}

impl<'de, M> Decode<'de, M> for bool
where
    M: Mode,
{
    #[inline]
    fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        decoder.decode_bool(cx)
    }
}

impl<M> Encode<M> for char
where
    M: Mode,
{
    #[inline]
    fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        encoder.encode_char(cx, *self)
    }
}

impl<'de, M> Decode<'de, M> for char
where
    M: Mode,
{
    #[inline]
    fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        decoder.decode_char(cx)
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

impl<M> Encode<M> for str
where
    M: Mode,
{
    #[inline]
    fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        encoder.encode_string(cx, self)
    }
}

impl<'de, M> Decode<'de, M> for &'de str
where
    M: Mode,
{
    #[inline]
    fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        struct Visitor;

        impl<'de, C> ValueVisitor<'de, C, str> for Visitor
        where
            C: Context,
        {
            type Ok = &'de str;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "string borrowed from source")
            }

            #[inline]
            fn visit_borrowed(self, _: &mut C, string: &'de str) -> Result<Self::Ok, C::Error> {
                Ok(string)
            }
        }

        decoder.decode_string(cx, Visitor)
    }
}

impl<M> Encode<M> for [u8]
where
    M: Mode,
{
    fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        encoder.encode_bytes(cx, self)
    }
}

impl<'de, M> Decode<'de, M> for &'de [u8]
where
    M: Mode,
{
    #[inline]
    fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        struct Visitor;

        impl<'de, C> ValueVisitor<'de, C, [u8]> for Visitor
        where
            C: Context,
        {
            type Ok = &'de [u8];

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes borrowed from source")
            }

            #[inline]
            fn visit_borrowed(self, _: &mut C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                Ok(bytes)
            }
        }

        decoder.decode_bytes(cx, Visitor)
    }
}

impl<T, M> Encode<M> for Option<T>
where
    M: Mode,
    T: Encode<M>,
{
    #[inline]
    fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        match self {
            Some(value) => encoder
                .encode_some(cx)
                .and_then(|encoder| value.encode(cx, encoder)),
            None => encoder.encode_none(cx),
        }
    }
}

impl<'de, M, T> Decode<'de, M> for Option<T>
where
    M: Mode,
    T: Decode<'de, M>,
{
    #[inline]
    fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        if let Some(decoder) = decoder.decode_option(cx)? {
            Ok(Some(T::decode(cx, decoder)?))
        } else {
            Ok(None)
        }
    }
}

impl<T, U, M> Encode<M> for Result<T, U>
where
    M: Mode,
    T: Encode<M>,
    U: Encode<M>,
{
    #[inline]
    fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        let variant = encoder.encode_variant(cx)?;

        match self {
            Ok(ok) => variant.insert::<M, _, _, _>(cx, 0usize, ok),
            Err(err) => variant.insert::<M, _, _, _>(cx, 1usize, err),
        }
    }
}

impl<'de, M, T, U> Decode<'de, M> for Result<T, U>
where
    M: Mode,
    T: Decode<'de, M>,
    U: Decode<'de, M>,
{
    #[inline]
    fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        let mut variant = decoder.decode_variant(cx)?;

        let this = match variant
            .tag(cx)
            .and_then(|v| <usize as Decode<M>>::decode(cx, v))?
        {
            0 => Ok(variant.variant(cx).and_then(|v| T::decode(cx, v))?),
            1 => Err(variant.variant(cx).and_then(|v| U::decode(cx, v))?),
            tag => return Err(cx.invalid_variant_tag("Result", tag)),
        };

        variant.end(cx)?;
        Ok(this)
    }
}

impl<T, M> Encode<M> for Wrapping<T>
where
    M: Mode,
    T: Encode<M>,
{
    #[inline]
    fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        Encode::encode(&self.0, cx, encoder)
    }
}

impl<'de, M, T> Decode<'de, M> for Wrapping<T>
where
    M: Mode,
    T: Decode<'de, M>,
{
    #[inline]
    fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        Ok(Wrapping(Decode::<M>::decode(cx, decoder)?))
    }
}

impl<M> Encode<M> for CStr
where
    M: Mode,
{
    #[inline]
    fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        encoder.encode_bytes(cx, self.to_bytes_with_nul())
    }
}

impl<'de, M> Decode<'de, M> for &'de CStr
where
    M: Mode,
{
    #[inline]
    fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        let bytes = <&[u8] as Decode<M>>::decode(cx, decoder)?;
        CStr::from_bytes_with_nul(bytes).map_err(|error| cx.custom(error))
    }
}
