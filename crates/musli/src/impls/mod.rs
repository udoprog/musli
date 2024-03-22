#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
mod alloc;
#[cfg(feature = "std")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
mod net;
mod range;
mod tuples;

use core::ffi::CStr;
use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize, Wrapping,
};
use core::{fmt, marker};

use crate::de::{Decode, DecodeBytes, Decoder, ValueVisitor, VariantDecoder};
use crate::en::{Encode, EncodeBytes, Encoder, SequenceEncoder, VariantEncoder};
use crate::Context;

impl<M> Encode<M> for () {
    #[inline]
    fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_unit(cx)
    }
}

impl<'de, M> Decode<'de, M> for () {
    #[inline]
    fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_unit(cx)
    }
}

impl<T, M> Encode<M> for marker::PhantomData<T> {
    #[inline]
    fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_unit(cx)
    }
}

impl<'de, M, T> Decode<'de, M> for marker::PhantomData<T> {
    fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
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
            impl<'de, M> Decode<'de, M> for core::sync::atomic::$ty {
                fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
                where
                    D: Decoder<'de>,
                {
                    cx.decode(decoder).map(Self::new)
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
        impl<M> Encode<M> for $ty {
            #[inline]
            fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder,
            {
                self.get().encode(cx, encoder)
            }
        }

        impl<'de, M> Decode<'de, M> for $ty {
            fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                let value = cx.decode(decoder)?;

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

impl<M, T, const N: usize> Encode<M> for [T; N]
where
    T: Encode<M>,
{
    #[inline]
    fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        let mut seq = encoder.encode_sequence(cx, N)?;

        for value in self.iter() {
            value.encode(cx, seq.encode_next(cx)?)?;
        }

        seq.end(cx)
    }
}

impl<'de, M, const N: usize> Decode<'de, M> for [u8; N] {
    #[inline]
    fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_array(cx)
    }
}

macro_rules! impl_number {
    ($ty:ty, $read:ident, $write:ident) => {
        impl<M> Encode<M> for $ty {
            #[inline]
            fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder,
            {
                encoder.$write(cx, *self)
            }
        }

        impl<'de, M> Decode<'de, M> for $ty {
            #[inline]
            fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                decoder.$read(cx)
            }
        }
    };
}

impl<M> Encode<M> for bool {
    #[inline]
    fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_bool(cx, *self)
    }
}

impl<'de, M> Decode<'de, M> for bool {
    #[inline]
    fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_bool(cx)
    }
}

impl<M> Encode<M> for char {
    #[inline]
    fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_char(cx, *self)
    }
}

impl<'de, M> Decode<'de, M> for char {
    #[inline]
    fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
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

impl<M> Encode<M> for str {
    #[inline]
    fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_string(cx, self)
    }
}

impl<'de, M> Decode<'de, M> for &'de str {
    #[inline]
    fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        struct Visitor;

        impl<'de, C> ValueVisitor<'de, C, str> for Visitor
        where
            C: ?Sized + Context,
        {
            type Ok = &'de str;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "string borrowed from source")
            }

            #[inline]
            fn visit_borrowed(self, _: &C, string: &'de str) -> Result<Self::Ok, C::Error> {
                Ok(string)
            }
        }

        decoder.decode_string(cx, Visitor)
    }
}

impl<M, T> Encode<M> for [T]
where
    T: Encode<M>,
{
    fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        let mut seq = encoder.encode_sequence(cx, self.len())?;

        let mut index = 0;

        for value in self {
            cx.enter_sequence_index(index);
            let encoder = seq.encode_next(cx)?;
            value.encode(cx, encoder)?;
            cx.leave_sequence_index();
            index = index.wrapping_add(index);
        }

        seq.end(cx)
    }
}

impl<'de, M> Decode<'de, M> for &'de [u8] {
    #[inline]
    fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        struct Visitor;

        impl<'de, C> ValueVisitor<'de, C, [u8]> for Visitor
        where
            C: ?Sized + Context,
        {
            type Ok = &'de [u8];

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes borrowed from source")
            }

            #[inline]
            fn visit_borrowed(self, _: &C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                Ok(bytes)
            }
        }

        decoder.decode_bytes(cx, Visitor)
    }
}

impl<T, M> Encode<M> for Option<T>
where
    T: Encode<M>,
{
    #[inline]
    fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        match self {
            Some(value) => value.encode(cx, encoder.encode_some(cx)?),
            None => encoder.encode_none(cx),
        }
    }
}

impl<'de, M, T> Decode<'de, M> for Option<T>
where
    T: Decode<'de, M>,
{
    #[inline]
    fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>,
    {
        if let Some(decoder) = decoder.decode_option(cx)? {
            Ok(Some(cx.decode(decoder)?))
        } else {
            Ok(None)
        }
    }
}

#[derive(Encode, Decode)]
#[musli(crate)]
enum ResultTag {
    Ok,
    Err,
}

impl<T, U, M> Encode<M> for Result<T, U>
where
    T: Encode<M>,
    U: Encode<M>,
{
    #[inline]
    fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        let variant = encoder.encode_variant(cx)?;

        match self {
            Ok(ok) => variant.insert_variant(cx, ResultTag::Ok, ok),
            Err(err) => variant.insert_variant(cx, ResultTag::Err, err),
        }
    }
}

impl<'de, M, T, U> Decode<'de, M> for Result<T, U>
where
    T: Decode<'de, M>,
    U: Decode<'de, M>,
{
    #[inline]
    fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>,
    {
        let mut variant = decoder.decode_variant(cx)?;

        let tag = cx.decode(variant.decode_tag(cx)?)?;

        let this = match tag {
            ResultTag::Ok => Ok(cx.decode(variant.decode_value(cx)?)?),
            ResultTag::Err => Err(cx.decode(variant.decode_value(cx)?)?),
        };

        variant.end(cx)?;
        Ok(this)
    }
}

impl<T, M> Encode<M> for Wrapping<T>
where
    T: Encode<M>,
{
    #[inline]
    fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        self.0.encode(cx, encoder)
    }
}

impl<'de, M, T> Decode<'de, M> for Wrapping<T>
where
    T: Decode<'de, M>,
{
    #[inline]
    fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>,
    {
        Ok(Wrapping(cx.decode(decoder)?))
    }
}

impl<M> Encode<M> for CStr {
    #[inline]
    fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_bytes(cx, self.to_bytes_with_nul())
    }
}

impl<'de, M> Decode<'de, M> for &'de CStr {
    #[inline]
    fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let bytes = cx.decode(decoder)?;
        CStr::from_bytes_with_nul(bytes).map_err(cx.map())
    }
}

impl<M> EncodeBytes<M> for [u8] {
    #[inline]
    fn encode_bytes<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        encoder.encode_bytes(cx, self)
    }
}

impl<const N: usize, M> EncodeBytes<M> for [u8; N] {
    #[inline]
    fn encode_bytes<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        encoder.encode_array(cx, self)
    }
}

impl<'de, M> DecodeBytes<'de, M> for &'de [u8] {
    #[inline]
    fn decode_bytes<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Decode::decode(cx, decoder)
    }
}

impl<'de, M, const N: usize> DecodeBytes<'de, M> for [u8; N] {
    #[inline]
    fn decode_bytes<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_array(cx)
    }
}
