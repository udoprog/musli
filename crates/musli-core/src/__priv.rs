use core::marker::PhantomData;

pub use crate::alloc::Allocator;
use crate::alloc::String;
pub use crate::context::Context;
use crate::de::UnsizedVisitor;
pub use crate::de::{
    AsDecoder, Decode, DecodeBytes, DecodePacked, DecodeTrace, Decoder, EntryDecoder, MapDecoder,
    SequenceDecoder, TryFastDecode, VariantDecoder,
};
pub use crate::en::{
    Encode, EncodeBytes, EncodePacked, EncodeTrace, Encoder, EntryEncoder, MapEncoder,
    SequenceEncoder, TryFastEncode, VariantEncoder,
};
use crate::expecting;
pub use crate::hint::MapHint;
pub use crate::never::Never;

pub use ::core::fmt;
pub use ::core::mem::{needs_drop, offset_of, size_of};
pub use ::core::option::Option;
pub use ::core::result::Result;

#[inline]
pub fn default<T>() -> T
where
    T: ::core::default::Default,
{
    ::core::default::Default::default()
}

/// Note that this returns `true` if skipping was unsupported.
#[inline]
pub fn skip<'de, D>(decoder: D) -> Result<bool, D::Error>
where
    D: Decoder<'de>,
{
    Ok(decoder.try_skip()?.is_unsupported())
}

/// Note that this returns `true` if skipping was unsupported.
#[inline]
pub fn skip_field<'de, D>(decoder: D) -> Result<bool, D::Error>
where
    D: EntryDecoder<'de>,
{
    skip(decoder.decode_value()?)
}

/// Collect and allocate a string from a [`Display`] implementation.
///
/// [`Display`]: fmt::Display
#[inline]
pub fn collect_string<C>(cx: C, value: impl fmt::Display) -> Result<String<C::Allocator>, C::Error>
where
    C: Context,
{
    match crate::alloc::collect_string(cx.alloc(), value) {
        Ok(string) => Ok(string),
        Err(error) => Err(cx.message(error)),
    }
}

/// Construct a map hint from an `Encode` implementation.
#[inline]
pub fn map_hint<M>(encode: &(impl Encode<M> + ?Sized)) -> impl MapHint + '_
where
    M: 'static,
{
    EncodeMapHint {
        encode,
        _marker: PhantomData,
    }
}

pub(crate) struct EncodeMapHint<'a, T, M>
where
    T: ?Sized,
{
    encode: &'a T,
    _marker: PhantomData<M>,
}

impl<T, M> MapHint for EncodeMapHint<'_, T, M>
where
    T: ?Sized + Encode<M>,
{
    #[inline]
    fn get(self) -> Option<usize> {
        self.encode.size_hint()
    }
}

pub mod sealed {
    pub trait Sealed {}
    impl Sealed for str {}
    impl Sealed for [u8] {}
}

pub trait UnsizedValue: self::sealed::Sealed {
    #[inline]
    fn decode_bytes<'de, C, V>(&'de self, cx: C, visitor: V) -> Result<V::Ok, V::Error>
    where
        C: Context,
        V: UnsizedVisitor<'de, C, [u8], Error = C::Error, Allocator = C::Allocator>,
    {
        let _ = visitor;

        Err(cx.message(expecting::unsupported_type(
            &expecting::Bytes,
            &"an unsized value",
        )))
    }

    #[inline]
    fn decode_string<'de, C, V>(&'de self, cx: C, visitor: V) -> Result<V::Ok, V::Error>
    where
        C: Context,
        V: UnsizedVisitor<'de, C, str, Error = C::Error, Allocator = C::Allocator>,
    {
        let _ = visitor;

        Err(cx.message(expecting::unsupported_type(
            &expecting::String,
            &"an unsized value",
        )))
    }
}

impl UnsizedValue for str {
    #[inline]
    fn decode_string<'de, C, V>(&'de self, cx: C, visitor: V) -> Result<V::Ok, V::Error>
    where
        C: Context,
        V: UnsizedVisitor<'de, C, str, Error = C::Error, Allocator = C::Allocator>,
    {
        visitor.visit_borrowed(cx, self)
    }
}

impl UnsizedValue for [u8] {
    #[inline]
    fn decode_bytes<'de, C, V>(&'de self, cx: C, visitor: V) -> Result<V::Ok, V::Error>
    where
        C: Context,
        V: UnsizedVisitor<'de, C, [u8], Error = C::Error, Allocator = C::Allocator>,
    {
        visitor.visit_borrowed(cx, self)
    }
}

/// Provide an unsized value.
#[inline]
pub fn provide_unsized<'de, C, T, M>(cx: C, value: &'de T) -> ProvideUnsizedEncoder<'de, C, T, M>
where
    C: Context,
    T: ?Sized + UnsizedValue,
    M: 'static,
{
    ProvideUnsizedEncoder {
        cx,
        value,
        _marker: PhantomData,
    }
}

/// Provider of an unsized value.
pub struct ProvideUnsizedEncoder<'de, C, T, M>
where
    C: Context,
    T: ?Sized + UnsizedValue,
    M: 'static,
{
    cx: C,
    value: &'de T,
    _marker: PhantomData<M>,
}

#[crate::decoder(crate)]
impl<'de, C, T, M> Decoder<'de> for ProvideUnsizedEncoder<'de, C, T, M>
where
    C: Context,
    T: ?Sized + UnsizedValue,
    M: 'static,
{
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type TryClone = Self;
    type DecodeBuffer = Self;

    #[inline]
    fn try_clone(&self) -> Option<Self> {
        Some(Self {
            cx: self.cx,
            value: self.value,
            _marker: PhantomData,
        })
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "an unsized value")
    }

    #[inline]
    fn decode_buffer(self) -> Result<Self::DecodeBuffer, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn decode_bytes<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, Self::Cx, [u8], Error = Self::Error, Allocator = Self::Allocator>,
    {
        self.value.decode_bytes(self.cx, visitor)
    }

    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, Self::Cx, str, Error = Self::Error, Allocator = Self::Allocator>,
    {
        self.value.decode_string(self.cx, visitor)
    }
}

impl<'de, C, T, M> AsDecoder for ProvideUnsizedEncoder<'de, C, T, M>
where
    C: Context,
    T: ?Sized + UnsizedValue,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type Decoder<'this>
        = ProvideUnsizedEncoder<'this, C, T, M>
    where
        Self: 'this;

    #[inline]
    fn as_decoder(&self) -> Result<Self::Decoder<'_>, Self::Error> {
        Ok(ProvideUnsizedEncoder {
            cx: self.cx,
            value: self.value,
            _marker: PhantomData,
        })
    }
}

/// Helper methods to report errors.
pub mod m {
    use core::fmt;

    use crate::Context;

    /// Report that an invalid variant tag was encountered.
    #[inline]
    pub fn invalid_variant_tag<C>(cx: C, type_name: &'static str, tag: impl fmt::Debug) -> C::Error
    where
        C: Context,
    {
        cx.message(format_args!(
            "Type {type_name} received invalid variant tag {tag:?}"
        ))
    }

    /// The value for the given tag could not be collected.
    #[inline]
    pub fn expected_tag<C>(cx: C, type_name: &'static str, tag: impl fmt::Debug) -> C::Error
    where
        C: Context,
    {
        cx.message(format_args!("Type {type_name} expected tag {tag:?}"))
    }

    /// Trying to decode an uninhabitable type.
    #[inline]
    pub fn uninhabitable<C>(cx: C, type_name: &'static str) -> C::Error
    where
        C: Context,
    {
        cx.message(format_args!(
            "Type {type_name} cannot be decoded since it's uninhabitable"
        ))
    }

    /// Encountered an unsupported field tag.
    #[inline]
    pub fn invalid_field_tag<C>(cx: C, type_name: &'static str, tag: impl fmt::Debug) -> C::Error
    where
        C: Context,
    {
        cx.message(format_args!(
            "Type {type_name} is missing invalid field tag {tag:?}"
        ))
    }

    /// Expected another field to decode.
    #[inline]
    pub fn expected_field_adjacent<C>(
        cx: C,
        type_name: &'static str,
        tag: impl fmt::Debug,
        content: impl fmt::Debug,
    ) -> C::Error
    where
        C: Context,
    {
        cx.message(format_args!(
            "Type {type_name} expected adjacent field {tag:?} or {content:?}"
        ))
    }

    /// Missing adjacent tag when decoding.
    #[inline]
    pub fn missing_adjacent_tag<C>(cx: C, type_name: &'static str, tag: impl fmt::Debug) -> C::Error
    where
        C: Context,
    {
        cx.message(format_args!(
            "Type {type_name} is missing adjacent tag {tag:?}"
        ))
    }

    /// Encountered an unsupported field tag.
    #[inline]
    pub fn invalid_field_string_tag<C>(
        cx: C,
        type_name: &'static str,
        field: impl fmt::Debug,
    ) -> C::Error
    where
        C: Context,
    {
        cx.message(format_args!(
            "Type {type_name} received invalid field tag {field:?}"
        ))
    }

    /// Missing variant field required to decode.
    #[inline]
    pub fn missing_variant_field<C>(
        cx: C,
        type_name: &'static str,
        tag: impl fmt::Debug,
    ) -> C::Error
    where
        C: Context,
    {
        cx.message(format_args!(
            "Type {type_name} is missing variant field {tag:?}"
        ))
    }

    /// Encountered an unsupported variant field.
    #[inline]
    pub fn invalid_variant_field_tag<C>(
        cx: C,
        type_name: &'static str,
        variant: impl fmt::Debug,
        tag: impl fmt::Debug,
    ) -> C::Error
    where
        C: Context,
    {
        cx.message(format_args!(
            "Type {type_name} received invalid variant field tag {tag:?} for variant {variant:?}",
        ))
    }

    /// Untagged enum could not be decoded.
    #[inline]
    pub fn untagged_mismatch<C>(cx: C, type_name: &'static str) -> C::Error
    where
        C: Context,
    {
        cx.message(format_args!("No variant of {type_name} could be decoded"))
    }
}
