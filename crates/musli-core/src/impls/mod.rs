#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
mod alloc;
#[cfg(feature = "std")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
mod net;
mod range;
mod tuples;

#[cfg(feature = "std")]
use core::any::TypeId;
use core::ffi::CStr;
use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize, Saturating, Wrapping,
};
use core::{fmt, marker};

use crate::de::{
    Decode, DecodeBytes, DecodePacked, DecodeUnsized, DecodeUnsizedBytes, Decoder, SequenceDecoder,
    UnsizedVisitor, VariantDecoder,
};
use crate::en::{Encode, EncodeBytes, EncodePacked, Encoder, SequenceEncoder, VariantEncoder};
#[cfg(feature = "std")]
use crate::mode::Text;
use crate::{Allocator, Context};

/// Platform tag used by certain platform-specific implementations.
#[cfg(feature = "std")]
enum PlatformTag {
    Unix,
    Windows,
}

#[cfg(feature = "std")]
impl<M> Encode<M> for PlatformTag
where
    M: 'static,
{
    type Encode = Self;

    const IS_BITWISE_ENCODE: bool = false;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode = M>,
    {
        if TypeId::of::<M>() == TypeId::of::<Text>() {
            match self {
                PlatformTag::Unix => encoder.encode("unix"),
                PlatformTag::Windows => encoder.encode("windows"),
            }
        } else {
            // For binary encoding, we use the tag as a single byte.
            let tag = match self {
                PlatformTag::Unix => 0,
                PlatformTag::Windows => 1,
            };

            encoder.encode_u8(tag)
        }
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

#[cfg(feature = "std")]
impl<'de, M, A> Decode<'de, M, A> for PlatformTag
where
    M: 'static,
    A: Allocator,
{
    // Unit is always packed, since it is a ZST.
    const IS_BITWISE_DECODE: bool = true;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Allocator = A>,
    {
        let cx = decoder.cx();

        if TypeId::of::<M>() == TypeId::of::<Text>() {
            decoder.decode_unsized(|value: &str| match value {
                "unix" => Ok(PlatformTag::Unix),
                "windows" => Ok(PlatformTag::Windows),
                _ => Err(cx.message(format_args!("Unsupported platform tag `{value}`",))),
            })
        } else {
            match decoder.decode_u8()? {
                0 => Ok(PlatformTag::Unix),
                1 => Ok(PlatformTag::Windows),
                _ => Err(cx.message("Unsupported platform tag")),
            }
        }
    }
}

impl<M> Encode<M> for () {
    type Encode = Self;

    // Unit is always packed, since it is a ZST.
    const IS_BITWISE_ENCODE: bool = true;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.encode_empty()
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M, A> Decode<'de, M, A> for ()
where
    A: Allocator,
{
    // Unit is always packed, since it is a ZST.
    const IS_BITWISE_DECODE: bool = true;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Allocator = A>,
    {
        decoder.decode_empty()
    }
}

impl<T, M> Encode<M> for marker::PhantomData<T> {
    type Encode = Self;

    // PhantomData is always packed, since it is a ZST.
    const IS_BITWISE_ENCODE: bool = true;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.encode_empty()
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M, A, T> Decode<'de, M, A> for marker::PhantomData<T>
where
    A: Allocator,
{
    // PhantomData is always packed, since it is a ZST.
    const IS_BITWISE_DECODE: bool = true;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_empty()?;
        Ok(marker::PhantomData)
    }
}

macro_rules! atomic_impl {
    ($size:literal $(, $ty:ident)*) => {
        $(
            #[cfg(target_has_atomic = $size)]
            impl<'de, M, A> Decode<'de, M, A> for core::sync::atomic::$ty
            where
                A: Allocator
            {
                const IS_BITWISE_DECODE: bool = true;

                fn decode<D>(decoder: D) -> Result<Self, D::Error>
                where
                    D: Decoder<'de>,
                {
                    decoder.decode().map(Self::new)
                }
            }

            #[cfg(target_has_atomic = $size)]
            impl<M> Encode<M> for core::sync::atomic::$ty {
                const IS_BITWISE_ENCODE: bool = false;

                type Encode = Self;

                #[inline]
                fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
                where
                    E: Encoder,
                {
                    use core::sync::atomic::Ordering::Relaxed;

                    self.load(Relaxed).encode(encoder)
                }

                #[inline]
                fn as_encode(&self) -> &Self::Encode {
                    self
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
            const IS_BITWISE_ENCODE: bool = true;

            type Encode = Self;

            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
            where
                E: Encoder,
            {
                self.get().encode(encoder)
            }

            #[inline]
            fn as_encode(&self) -> &Self::Encode {
                self
            }
        }

        impl<'de, M, A> Decode<'de, M, A> for $ty
        where
            A: Allocator,
        {
            // Non zero types are not considered packed during decoding, because
            // they cannot inhabit the bit pattern of all zeros.
            const IS_BITWISE_DECODE: bool = false;

            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de, Allocator = A>,
            {
                let cx = decoder.cx();
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
    const IS_BITWISE_ENCODE: bool =
        T::IS_BITWISE_ENCODE && core::mem::size_of::<T>() % core::mem::align_of::<T>() == 0;

    type Encode = [T; N];

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode = M>,
    {
        encoder.encode_slice(self)
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M, T, A, const N: usize> Decode<'de, M, A> for [T; N]
where
    T: Decode<'de, M, A>,
    A: Allocator,
{
    const IS_BITWISE_DECODE: bool =
        T::IS_BITWISE_DECODE && core::mem::size_of::<T>() % core::mem::align_of::<T>() == 0;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        let cx = decoder.cx();
        let mark = cx.mark();

        decoder.decode_sequence(|seq| {
            let mut array = crate::internal::FixedVec::new();

            while let Some(item) = seq.try_decode_next()? {
                array.try_push(item.decode()?).map_err(cx.map())?;
            }

            if array.len() != N {
                return Err(cx.message_at(
                    &mark,
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
    fn encode_packed<E>(&self, encoder: E) -> Result<(), E::Error>
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

impl<'de, M, A, T, const N: usize> DecodePacked<'de, M, A> for [T; N]
where
    A: Allocator,
    T: Decode<'de, M, A>,
{
    #[inline]
    fn decode_packed<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        let cx = decoder.cx();

        decoder.decode_pack(|pack| {
            let mut array = crate::internal::FixedVec::new();

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
            const IS_BITWISE_ENCODE: bool = true;

            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
            where
                E: Encoder,
            {
                encoder.$write(*self)
            }

            type Encode = Self;

            #[inline]
            fn as_encode(&self) -> &Self::Encode {
                self
            }
        }

        impl<'de, M, A> Decode<'de, M, A> for $ty
        where
            A: Allocator,
        {
            const IS_BITWISE_DECODE: bool = true;

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

impl<M> Encode<M> for bool {
    type Encode = Self;

    // A boolean is bitwise encodeable since it's guaranteed to inhabit a valid
    // bit pattern of one byte without padding.
    const IS_BITWISE_ENCODE: bool = true;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.encode_bool(*self)
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M, A> Decode<'de, M, A> for bool
where
    A: Allocator,
{
    // A boolean is not packed during decoding since every bit pattern that
    // comes in is not necessarily valid.
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_bool()
    }
}

impl<M> Encode<M> for char {
    type Encode = Self;

    // A char bitwise encodeable since it's guaranteed to inhabit a valid bit
    // pattern of a u32 and can be bitwise copied when encoded.
    const IS_BITWISE_ENCODE: bool = true;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.encode_char(*self)
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M, A> Decode<'de, M, A> for char
where
    A: Allocator,
{
    // A char is not packed during decoding since it's not guaranteed to inhabit
    // a valid bit pattern of a u32 and can not be bitwise copied when encoded.
    const IS_BITWISE_DECODE: bool = false;

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

impl<M> Encode<M> for str {
    const IS_BITWISE_ENCODE: bool = false;
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.encode_string(self)
    }

    type Encode = Self;

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M, A> Decode<'de, M, A> for &'de str
where
    A: Allocator,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        struct Visitor;

        #[crate::unsized_visitor(crate)]
        impl<'de, C> UnsizedVisitor<'de, C, str> for Visitor
        where
            C: Context,
        {
            type Ok = &'de str;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "string borrowed from source")
            }

            #[inline]
            fn visit_borrowed(self, _: C, string: &'de str) -> Result<Self::Ok, Self::Error> {
                Ok(string)
            }
        }

        decoder.decode_string(Visitor)
    }
}

impl<'de, M> DecodeUnsized<'de, M> for str {
    #[inline]
    fn decode_unsized<D, F, O>(decoder: D, f: F) -> Result<O, D::Error>
    where
        D: Decoder<'de>,
        F: FnOnce(&Self) -> Result<O, D::Error>,
    {
        struct Visitor<F>(F);

        #[crate::unsized_visitor(crate)]
        impl<C, F, O> UnsizedVisitor<'_, C, str> for Visitor<F>
        where
            C: Context,
            F: FnOnce(&str) -> Result<O, C::Error>,
        {
            type Ok = O;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "string visited from source")
            }

            #[inline]
            fn visit_ref(self, _: C, string: &str) -> Result<Self::Ok, C::Error> {
                (self.0)(string)
            }
        }

        decoder.decode_string(Visitor(f))
    }
}

impl<'de, M> DecodeUnsized<'de, M> for [u8] {
    #[inline]
    fn decode_unsized<D, F, O>(decoder: D, f: F) -> Result<O, D::Error>
    where
        D: Decoder<'de>,
        F: FnOnce(&Self) -> Result<O, D::Error>,
    {
        struct Visitor<F>(F);

        #[crate::unsized_visitor(crate)]
        impl<C, F, O> UnsizedVisitor<'_, C, [u8]> for Visitor<F>
        where
            C: Context,
            F: FnOnce(&[u8]) -> Result<O, C::Error>,
        {
            type Ok = O;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes visited from source")
            }

            #[inline]
            fn visit_ref(self, _: C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                (self.0)(bytes)
            }
        }

        decoder.decode_bytes(Visitor(f))
    }
}

impl<M, T> Encode<M> for [T]
where
    T: Encode<M>,
{
    type Encode = Self;

    const IS_BITWISE_ENCODE: bool = false;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode = M>,
    {
        encoder.encode_slice(self)
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M, A> Decode<'de, M, A> for &'de [u8]
where
    A: Allocator,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        struct Visitor;

        #[crate::unsized_visitor(crate)]
        impl<'de, C> UnsizedVisitor<'de, C, [u8]> for Visitor
        where
            C: Context,
        {
            type Ok = &'de [u8];

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes borrowed from source")
            }

            #[inline]
            fn visit_borrowed(self, _: C, bytes: &'de [u8]) -> Result<Self::Ok, Self::Error> {
                Ok(bytes)
            }
        }

        decoder.decode_bytes(Visitor)
    }
}

impl<'de, M> DecodeUnsizedBytes<'de, M> for [u8] {
    #[inline]
    fn decode_unsized_bytes<D, F, O>(decoder: D, f: F) -> Result<O, D::Error>
    where
        D: Decoder<'de>,
        F: FnOnce(&Self) -> Result<O, D::Error>,
    {
        struct Visitor<F>(F);

        #[crate::unsized_visitor(crate)]
        impl<C, F, O> UnsizedVisitor<'_, C, [u8]> for Visitor<F>
        where
            C: Context,
            F: FnOnce(&[u8]) -> Result<O, C::Error>,
        {
            type Ok = O;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes visited from source")
            }

            #[inline]
            fn visit_ref(self, _: C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
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
    type Encode = Self;

    const IS_BITWISE_ENCODE: bool = false;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode = M>,
    {
        match self {
            Some(value) => encoder.encode_some()?.encode(value),
            None => encoder.encode_none(),
        }
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M, A, T> Decode<'de, M, A> for Option<T>
where
    A: Allocator,
    T: Decode<'de, M, A>,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
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
    ResultTag: Encode<M>,
{
    type Encode = Self;

    const IS_BITWISE_ENCODE: bool = false;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode = M>,
    {
        let variant = encoder.encode_variant()?;

        match self {
            Ok(ok) => variant.insert_variant(&ResultTag::Ok, ok),
            Err(err) => variant.insert_variant(&ResultTag::Err, err),
        }
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M, A, T, U> Decode<'de, M, A> for Result<T, U>
where
    A: Allocator,
    T: Decode<'de, M, A>,
    U: Decode<'de, M, A>,
    ResultTag: Decode<'de, M, A>,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
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
    const IS_BITWISE_ENCODE: bool = T::IS_BITWISE_ENCODE;

    type Encode = Self;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode = M>,
    {
        self.0.encode(encoder)
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M, T, A> Decode<'de, M, A> for Wrapping<T>
where
    T: Decode<'de, M, A>,
    A: Allocator,
{
    const IS_BITWISE_DECODE: bool = T::IS_BITWISE_DECODE;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        Ok(Wrapping(decoder.decode()?))
    }
}

impl<T, M> Encode<M> for Saturating<T>
where
    T: Encode<M>,
{
    const IS_BITWISE_ENCODE: bool = T::IS_BITWISE_ENCODE;

    type Encode = Self;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode = M>,
    {
        self.0.encode(encoder)
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M, T, A> Decode<'de, M, A> for Saturating<T>
where
    T: Decode<'de, M, A>,
    A: Allocator,
{
    const IS_BITWISE_DECODE: bool = T::IS_BITWISE_DECODE;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        Ok(Saturating(decoder.decode()?))
    }
}

impl<M> Encode<M> for CStr {
    type Encode = Self;

    const IS_BITWISE_ENCODE: bool = false;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.encode_bytes(self.to_bytes_with_nul())
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M, A> Decode<'de, M, A> for &'de CStr
where
    A: Allocator,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let cx = decoder.cx();
        let bytes = decoder.decode()?;
        CStr::from_bytes_with_nul(bytes).map_err(cx.map())
    }
}

impl<'de, M> DecodeUnsized<'de, M> for CStr {
    #[inline]
    fn decode_unsized<D, F, O>(decoder: D, f: F) -> Result<O, D::Error>
    where
        D: Decoder<'de, Mode = M>,
        F: FnOnce(&Self) -> Result<O, D::Error>,
    {
        let cx = decoder.cx();

        DecodeUnsizedBytes::decode_unsized_bytes(decoder, |bytes: &[u8]| {
            let cstr = CStr::from_bytes_with_nul(bytes).map_err(cx.map())?;
            f(cstr)
        })
    }
}

impl<M> EncodeBytes<M> for [u8] {
    const ENCODE_BYTES_PACKED: bool = false;

    type EncodeBytes = [u8];

    #[inline]
    fn encode_bytes<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode = M>,
    {
        encoder.encode_bytes(self)
    }

    #[inline]
    fn as_encode_bytes(&self) -> &Self::EncodeBytes {
        self
    }
}

impl<const N: usize, M> EncodeBytes<M> for [u8; N] {
    const ENCODE_BYTES_PACKED: bool = true;

    type EncodeBytes = [u8; N];

    #[inline]
    fn encode_bytes<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode = M>,
    {
        encoder.encode_array(self)
    }

    #[inline]
    fn as_encode_bytes(&self) -> &Self::EncodeBytes {
        self
    }
}

impl<'de, M, A> DecodeBytes<'de, M, A> for &'de [u8]
where
    A: Allocator,
{
    const DECODE_BYTES_PACKED: bool = false;

    #[inline]
    fn decode_bytes<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Allocator = A>,
    {
        Decode::decode(decoder)
    }
}

impl<'de, M, A, const N: usize> DecodeBytes<'de, M, A> for [u8; N]
where
    A: Allocator,
{
    const DECODE_BYTES_PACKED: bool = true;

    #[inline]
    fn decode_bytes<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Allocator = A>,
    {
        decoder.decode_array()
    }
}
