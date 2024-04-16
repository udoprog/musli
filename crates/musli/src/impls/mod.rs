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

use crate::de::{
    Decode, DecodeBytes, DecodePacked, DecodeUnsized, DecodeUnsizedBytes, Decoder, PackDecoder,
    SequenceDecoder, ValueVisitor, VariantDecoder,
};
use crate::en::{
    Encode, EncodeBytes, EncodePacked, Encoder, PackEncoder, SequenceEncoder, VariantEncoder,
};
use crate::hint::SequenceHint;
use crate::Context;

/// Platform tag used by certain platform-specific implementations.
#[cfg(feature = "std")]
#[derive(Encode, Decode)]
#[musli(crate)]
enum PlatformTag {
    Unix,
    Windows,
}

impl<M> Encode<M> for () {
    #[inline]
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_unit()
    }
}

impl<'de, M> Decode<'de, M> for () {
    #[inline]
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_unit()
    }
}

impl<T, M> Encode<M> for marker::PhantomData<T> {
    #[inline]
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_unit()
    }
}

impl<'de, M, T> Decode<'de, M> for marker::PhantomData<T> {
    #[inline]
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_unit()?;
        Ok(marker::PhantomData)
    }
}

macro_rules! atomic_impl {
    ($size:literal $(, $ty:ident)*) => {
        $(
            #[cfg(target_has_atomic = $size)]
            impl<'de, M> Decode<'de, M> for core::sync::atomic::$ty {
                fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
                where
                    D: Decoder<'de>,
                {
                    decoder.decode().map(Self::new)
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
                let value = decoder.decode()?;

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
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        let hint = SequenceHint::with_size(N);

        encoder.encode_sequence_fn(&hint, |seq| {
            for value in self.iter() {
                seq.push(value)?;
            }

            Ok(())
        })
    }
}

impl<'de, M, T, const N: usize> Decode<'de, M> for [T; N]
where
    T: Decode<'de, M>,
{
    #[inline]
    fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>,
    {
        let mark = cx.mark();

        decoder.decode_sequence(|seq| {
            let mut array = crate::fixed::FixedVec::new();

            while let Some(item) = seq.decode_element()? {
                array.try_push(item.decode()?).map_err(cx.map())?;
            }

            if array.len() != N {
                return Err(cx.marked_message(
                    mark,
                    format_args!(
                        "Array with length {} does not have the expected {N} number of elements",
                        array.len()
                    ),
                ));
            }

            Ok(array.into_inner())
        })
    }
}

impl<M, T, const N: usize> EncodePacked<M> for [T; N]
where
    T: Encode<M>,
{
    #[inline]
    fn encode_packed<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        encoder.encode_pack_fn(|seq| {
            for value in self.iter() {
                seq.push(value)?;
            }

            Ok(())
        })
    }
}

impl<'de, M, T, const N: usize> DecodePacked<'de, M> for [T; N]
where
    T: Decode<'de, M>,
{
    #[inline]
    fn decode_packed<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>,
    {
        decoder.decode_pack(|pack| {
            let mut array = crate::fixed::FixedVec::new();

            while array.len() < N {
                let item = pack.decode_next()?;
                array.try_push(item.decode()?).map_err(cx.map())?;
            }

            Ok(array.into_inner())
        })
    }
}

macro_rules! impl_number {
    ($ty:ty, $read:ident, $write:ident) => {
        impl<M> Encode<M> for $ty {
            #[inline]
            fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder,
            {
                encoder.$write(*self)
            }
        }

        impl<'de, M> Decode<'de, M> for $ty {
            #[inline]
            fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                decoder.$read()
            }
        }
    };
}

impl<M> Encode<M> for bool {
    #[inline]
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_bool(*self)
    }
}

impl<'de, M> Decode<'de, M> for bool {
    #[inline]
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_bool()
    }
}

impl<M> Encode<M> for char {
    #[inline]
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_char(*self)
    }
}

impl<'de, M> Decode<'de, M> for char {
    #[inline]
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
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

impl<M> Encode<M> for str {
    #[inline]
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_string(self)
    }
}

impl<'de, M> Decode<'de, M> for &'de str {
    #[inline]
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
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

        decoder.decode_string(Visitor)
    }
}

impl<'de, M> DecodeUnsized<'de, M> for str {
    #[inline]
    fn decode_unsized<D, F, O>(_: &D::Cx, decoder: D, f: F) -> Result<O, D::Error>
    where
        D: Decoder<'de>,
        F: FnOnce(&Self) -> Result<O, D::Error>,
    {
        struct Visitor<F>(F);

        impl<'de, C, F, O> ValueVisitor<'de, C, str> for Visitor<F>
        where
            C: ?Sized + Context,
            F: FnOnce(&str) -> Result<O, C::Error>,
        {
            type Ok = O;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "string visited from source")
            }

            #[inline]
            fn visit_ref(self, _: &C, string: &str) -> Result<Self::Ok, C::Error> {
                (self.0)(string)
            }
        }

        decoder.decode_string(Visitor(f))
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
        let hint = SequenceHint::with_size(self.len());

        encoder.encode_sequence_fn(&hint, |seq| {
            let mut index = 0;

            for value in self {
                cx.enter_sequence_index(index);
                seq.encode_element()?.encode(value)?;
                cx.leave_sequence_index();
                index = index.wrapping_add(index);
            }

            Ok(())
        })
    }
}

impl<'de, M> Decode<'de, M> for &'de [u8] {
    #[inline]
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
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

        decoder.decode_bytes(Visitor)
    }
}

impl<'de, M> DecodeUnsizedBytes<'de, M> for [u8] {
    #[inline]
    fn decode_unsized_bytes<D, F, O>(_: &D::Cx, decoder: D, f: F) -> Result<O, D::Error>
    where
        D: Decoder<'de>,
        F: FnOnce(&Self) -> Result<O, D::Error>,
    {
        struct Visitor<F>(F);

        impl<'de, C, F, O> ValueVisitor<'de, C, [u8]> for Visitor<F>
        where
            C: ?Sized + Context,
            F: FnOnce(&[u8]) -> Result<O, C::Error>,
        {
            type Ok = O;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes visited from source")
            }

            #[inline]
            fn visit_ref(self, _: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                (self.0)(bytes)
            }
        }

        decoder.decode_bytes(Visitor(f))
    }
}

impl<T, M> Encode<M> for Option<T>
where
    T: Encode<M>,
{
    #[inline]
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        match self {
            Some(value) => encoder.encode_some()?.encode(value),
            None => encoder.encode_none(),
        }
    }
}

impl<'de, M, T> Decode<'de, M> for Option<T>
where
    T: Decode<'de, M>,
{
    #[inline]
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>,
    {
        if let Some(decoder) = decoder.decode_option()? {
            Ok(Some(decoder.decode()?))
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
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        let variant = encoder.encode_variant()?;

        match self {
            Ok(ok) => variant.insert_variant(ResultTag::Ok, ok),
            Err(err) => variant.insert_variant(ResultTag::Err, err),
        }
    }
}

impl<'de, M, T, U> Decode<'de, M> for Result<T, U>
where
    T: Decode<'de, M>,
    U: Decode<'de, M>,
{
    #[inline]
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>,
    {
        decoder.decode_variant(|variant| {
            let tag = variant.decode_tag()?.decode()?;

            Ok(match tag {
                ResultTag::Ok => Ok(variant.decode_value()?.decode()?),
                ResultTag::Err => Err(variant.decode_value()?.decode()?),
            })
        })
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
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>,
    {
        Ok(Wrapping(decoder.decode()?))
    }
}

impl<M> Encode<M> for CStr {
    #[inline]
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_bytes(self.to_bytes_with_nul())
    }
}

impl<'de, M> Decode<'de, M> for &'de CStr {
    #[inline]
    fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let bytes = decoder.decode()?;
        CStr::from_bytes_with_nul(bytes).map_err(cx.map())
    }
}

impl<'de, M> DecodeUnsized<'de, M> for CStr {
    #[inline(always)]
    fn decode_unsized<D, F, O>(cx: &D::Cx, decoder: D, f: F) -> Result<O, D::Error>
    where
        D: Decoder<'de, Mode = M>,
        F: FnOnce(&Self) -> Result<O, D::Error>,
    {
        cx.decode_unsized_bytes(decoder, |bytes: &[u8]| {
            let cstr = CStr::from_bytes_with_nul(bytes).map_err(cx.map())?;
            f(cstr)
        })
    }
}

impl<M> EncodeBytes<M> for [u8] {
    #[inline]
    fn encode_bytes<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        encoder.encode_bytes(self)
    }
}

impl<const N: usize, M> EncodeBytes<M> for [u8; N] {
    #[inline]
    fn encode_bytes<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        encoder.encode_array(self)
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
    fn decode_bytes<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_array()
    }
}
