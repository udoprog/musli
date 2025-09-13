use core::error::Error;
use core::fmt;

use crate::alloc::ToOwned;
use crate::de::{
    AsDecoder, DecodeSliceBuilder, DecodeUnsized, DecodeUnsizedBytes, SizeHint, Skip,
    TryFastDecode, UnsizedVisitor, Visitor,
};
use crate::hint::SequenceHint;
use crate::{Allocator, Context, Decode, Decoder};

/// A type that has been modified to carry the specified allocator instead of
/// the one provided through the wrapped context.
///
/// See [`Context::with_allocator`] or [`Decoder::with_allocator`] for more
/// details.
///
/// [`Context::with_allocator`]: crate::context::Context::with_allocator
/// [`Decoder::with_allocator`]: crate::de::Decoder::with_allocator
pub struct WithAllocator<I, A> {
    inner: I,
    allocator: A,
}

impl<C, A> WithAllocator<C, A> {
    /// Create a new context wrapper.
    #[inline]
    pub(crate) fn new(inner: C, allocator: A) -> Self {
        Self { inner, allocator }
    }
}

impl<C, A> Clone for WithAllocator<C, A>
where
    C: Copy,
    A: Copy,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<C, A> Copy for WithAllocator<C, A>
where
    C: Copy,
    A: Copy,
{
}

impl<C, A> Context for WithAllocator<C, A>
where
    C: Context,
    A: Allocator,
{
    type Error = C::Error;
    type Mark = C::Mark;
    type Allocator = A;

    #[inline]
    fn clear(self) {
        self.inner.clear()
    }

    #[inline]
    fn advance(self, n: usize) {
        self.inner.advance(n)
    }

    #[inline]
    fn mark(self) -> Self::Mark {
        self.inner.mark()
    }

    #[inline]
    fn restore(self, mark: &Self::Mark) {
        self.inner.restore(mark)
    }

    #[inline]
    fn alloc(self) -> Self::Allocator {
        self.allocator
    }

    #[inline]
    fn custom<E>(self, error: E) -> Self::Error
    where
        E: 'static + Send + Sync + Error,
    {
        self.inner.custom(error)
    }

    #[inline]
    fn message<M>(self, message: M) -> Self::Error
    where
        M: fmt::Display,
    {
        self.inner.message(message)
    }

    #[inline]
    fn map<E>(self) -> impl FnOnce(E) -> Self::Error
    where
        E: 'static + Send + Sync + Error,
    {
        self.inner.map()
    }

    #[inline]
    fn message_at<M>(self, mark: &Self::Mark, message: M) -> Self::Error
    where
        M: fmt::Display,
    {
        self.inner.message_at(mark, message)
    }

    #[inline]
    fn custom_at<E>(self, mark: &Self::Mark, message: E) -> Self::Error
    where
        E: 'static + Send + Sync + Error,
    {
        self.inner.custom_at(mark, message)
    }

    #[inline]
    fn enter_struct(self, type_name: &'static str) {
        self.inner.enter_struct(type_name)
    }

    #[inline]
    fn leave_struct(self) {
        self.inner.leave_struct()
    }

    #[inline]
    fn enter_enum(self, type_name: &'static str) {
        self.inner.enter_enum(type_name)
    }

    #[inline]
    fn leave_enum(self) {
        self.inner.leave_enum()
    }

    #[inline]
    fn enter_named_field<F>(self, type_name: &'static str, field: F)
    where
        F: fmt::Display,
    {
        self.inner.enter_named_field(type_name, field)
    }

    #[inline]
    fn enter_unnamed_field<F>(self, index: u32, name: F)
    where
        F: fmt::Display,
    {
        self.inner.enter_unnamed_field(index, name)
    }

    #[inline]
    fn leave_field(self) {
        self.inner.leave_field()
    }

    #[inline]
    fn enter_variant<V>(self, type_name: &'static str, tag: V)
    where
        V: fmt::Display,
    {
        self.inner.enter_variant(type_name, tag)
    }

    #[inline]
    fn leave_variant(self) {
        self.inner.leave_variant()
    }

    #[inline]
    fn enter_map_key<K>(self, field: K)
    where
        K: fmt::Display,
    {
        self.inner.enter_map_key(field)
    }

    #[inline]
    fn leave_map_key(self) {
        self.inner.leave_map_key()
    }

    #[inline]
    fn enter_sequence_index(self, index: usize) {
        self.inner.enter_sequence_index(index)
    }

    #[inline]
    fn leave_sequence_index(self) {
        self.inner.leave_sequence_index()
    }
}

#[crate::trait_defaults(crate)]
impl<'de, I, U> Decoder<'de> for WithAllocator<I, U>
where
    I: Decoder<'de>,
    U: Allocator,
{
    type Cx = WithAllocator<I::Cx, U>;
    type Error = I::Error;
    type Allocator = U;
    type Mode = I::Mode;
    type TryClone = WithAllocator<I::TryClone, U>;
    type DecodeBuffer = WithAllocator<I::DecodeBuffer, U>;
    type DecodeSome = WithAllocator<I::DecodeSome, U>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.inner.cx().with_allocator(self.allocator)
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expecting(f)
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        Some(WithAllocator::new(self.inner.try_clone()?, self.allocator))
    }

    #[inline]
    fn try_fast_decode<T, A>(self) -> Result<TryFastDecode<T, Self>, Self::Error>
    where
        T: Decode<'de, Self::Mode, A>,
        A: Allocator,
    {
        match self.inner.try_fast_decode()? {
            TryFastDecode::Ok(value) => Ok(TryFastDecode::Ok(value)),
            TryFastDecode::Unsupported(decoder) => Ok(TryFastDecode::Unsupported(
                WithAllocator::new(decoder, self.allocator),
            )),
        }
    }

    #[inline]
    fn decode<T>(self) -> Result<T, Self::Error>
    where
        T: Decode<'de, Self::Mode, Self::Allocator>,
    {
        match self.try_fast_decode::<T, Self::Allocator>()? {
            TryFastDecode::Ok(value) => Ok(value),
            TryFastDecode::Unsupported(decoder) => T::decode(decoder),
        }
    }

    #[inline]
    fn decode_unsized<T, F, O>(self, f: F) -> Result<O, Self::Error>
    where
        T: ?Sized + DecodeUnsized<'de, Self::Mode>,
        F: FnOnce(&T) -> Result<O, Self::Error>,
    {
        self.inner.decode_unsized(f)
    }

    #[inline]
    fn decode_unsized_bytes<T, F, O>(self, f: F) -> Result<O, Self::Error>
    where
        T: ?Sized + DecodeUnsizedBytes<'de, Self::Mode>,
        F: FnOnce(&T) -> Result<O, Self::Error>,
    {
        self.inner.decode_unsized_bytes(f)
    }

    #[inline]
    fn skip(self) -> Result<(), Self::Error> {
        todo!()
    }

    #[inline]
    fn try_skip(self) -> Result<Skip, Self::Error> {
        todo!()
    }

    #[inline]
    fn decode_buffer(self) -> Result<Self::DecodeBuffer, Self::Error> {
        todo!()
    }

    #[inline]
    fn decode_empty(self) -> Result<(), Self::Error> {
        self.inner.decode_empty()
    }

    #[inline]
    fn decode_bool(self) -> Result<bool, Self::Error> {
        self.inner.decode_bool()
    }

    #[inline]
    fn decode_char(self) -> Result<char, Self::Error> {
        self.inner.decode_char()
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, Self::Error> {
        self.inner.decode_u8()
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        self.inner.decode_u16()
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        self.inner.decode_u32()
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        self.inner.decode_u64()
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, Self::Error> {
        self.inner.decode_u128()
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        self.inner.decode_i8()
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        self.inner.decode_i16()
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        self.inner.decode_i32()
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        self.inner.decode_i64()
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, Self::Error> {
        self.inner.decode_i128()
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, Self::Error> {
        self.inner.decode_usize()
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, Self::Error> {
        self.inner.decode_isize()
    }

    #[inline]
    fn decode_f32(self) -> Result<f32, Self::Error> {
        self.inner.decode_f32()
    }

    #[inline]
    fn decode_f64(self) -> Result<f64, Self::Error> {
        self.inner.decode_f64()
    }

    #[inline]
    fn decode_array<const N: usize>(self) -> Result<[u8; N], Self::Error> {
        self.inner.decode_array()
    }

    #[inline]
    fn decode_bytes<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, Self::Cx, [u8], Error = Self::Error, Allocator = Self::Allocator>,
    {
        let visitor = WithAllocator::new(visitor, self.inner.cx().alloc());
        self.inner.decode_bytes(visitor)
    }

    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, Self::Cx, str, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let visitor = WithAllocator::new(visitor, self.inner.cx().alloc());
        self.inner.decode_string(visitor)
    }

    #[inline]
    fn decode_option(self) -> Result<Option<Self::DecodeSome>, Self::Error> {
        Some(WithAllocator::new(
            self.inner.decode_option()?,
            self.allocator,
        ))
    }

    #[inline]
    fn decode_pack<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodePack) -> Result<O, Self::Error>,
    {
        self.inner.decode_pack(f)
    }

    #[inline]
    fn decode_slice<V, T>(self) -> Result<V, Self::Error>
    where
        V: DecodeSliceBuilder<T, Self::Allocator>,
        T: Decode<'de, Self::Mode, Self::Allocator>,
    {
        self.inner.decode_slice()
    }

    #[inline]
    fn decode_sequence<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        self.inner.decode_sequence(f)
    }

    #[inline]
    fn decode_sequence_hint<F, O>(self, hint: impl SequenceHint, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        self.inner.decode_sequence_hint(hint, f)
    }

    #[inline]
    fn decode_map<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        self.inner.decode_map(f)
    }

    #[inline]
    fn decode_map_hint<F, O>(self, hint: impl crate::hint::MapHint, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        self.inner.decode_map_hint(hint, f)
    }

    #[inline]
    fn decode_map_entries<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMapEntries) -> Result<O, Self::Error>,
    {
        self.inner.decode_map_entries(f)
    }

    #[inline]
    fn decode_variant<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeVariant) -> Result<O, Self::Error>,
    {
        self.inner.decode_variant(f)
    }

    #[inline]
    fn decode_number<V>(self, visitor: V) -> Result<V::Ok, Self::Error>
    where
        V: Visitor<'de, Self::Cx, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let visitor = WithAllocator::new(visitor, self.inner.cx().alloc());
        self.inner.decode_number(visitor)
    }

    #[inline]
    fn decode_any<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: Visitor<'de, Self::Cx, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let visitor = WithAllocator::new(visitor, self.inner.cx().alloc());
        self.inner.decode_any(visitor)
    }
}

impl<I, U> AsDecoder for WithAllocator<I, U>
where
    I: AsDecoder,
    U: Allocator,
{
    type Cx = WithAllocator<I::Cx, U>;
    type Error = I::Error;
    type Allocator = U;
    type Mode = I::Mode;
    type Decoder<'this>
        = WithAllocator<I::Decoder<'this>, U>
    where
        Self: 'this;

    #[inline]
    fn as_decoder(&self) -> Result<Self::Decoder<'_>, Self::Error> {
        let decoder = self.inner.as_decoder()?;
        Ok(WithAllocator::new(decoder, self.allocator))
    }
}

impl<'de, I, A, C> Visitor<'de, C> for WithAllocator<I, A>
where
    I: Visitor<'de, C>,
    C: Context<Error = Self::Error, Allocator = Self::Allocator>,
{
    type Ok = I::Ok;
    type Error = I::Error;
    type Allocator = A;
    type String = WithAllocator<Self::String, A>;
    type Bytes = WithAllocator<Self::Bytes, A>;

    type __UseMusliVisitorAttributeMacro = ();

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expecting(f)
    }

    #[inline]
    fn visit_empty(self, cx: C) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_empty(cx)
    }

    #[inline]
    fn visit_bool(self, cx: C, value: bool) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_bool(cx, value)
    }

    #[inline]
    fn visit_char(self, cx: C, value: char) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_char(cx, value)
    }

    #[inline]
    fn visit_u8(self, cx: C, value: u8) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_u8(cx, value)
    }

    #[inline]
    fn visit_u16(self, cx: C, value: u16) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_u16(cx, value)
    }

    #[inline]
    fn visit_u32(self, cx: C, value: u32) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_u32(cx, value)
    }

    #[inline]
    fn visit_u64(self, cx: C, value: u64) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_u64(cx, value)
    }

    #[inline]
    fn visit_u128(self, cx: C, value: u128) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_u128(cx, value)
    }

    #[inline]
    fn visit_i8(self, cx: C, value: i8) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_i8(cx, value)
    }

    #[inline]
    fn visit_i16(self, cx: C, value: i16) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_i16(cx, value)
    }

    #[inline]
    fn visit_i32(self, cx: C, value: i32) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_i32(cx, value)
    }

    #[inline]
    fn visit_i64(self, cx: C, value: i64) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_i64(cx, value)
    }

    #[inline]
    fn visit_i128(self, cx: C, value: i128) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_i128(cx, value)
    }

    #[inline]
    fn visit_usize(self, cx: C, value: usize) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_usize(cx, value)
    }

    #[inline]
    fn visit_isize(self, cx: C, value: isize) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_isize(cx, value)
    }

    #[inline]
    fn visit_f32(self, cx: C, value: f32) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_f32(cx, value)
    }

    #[inline]
    fn visit_f64(self, cx: C, value: f64) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_f64(cx, value)
    }

    #[inline]
    fn visit_none(self, cx: C) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_none(cx)
    }

    #[inline]
    fn visit_some<D>(self, decoder: D) -> Result<Self::Ok, Self::Error>
    where
        D: Decoder<'de, Cx = C, Error = C::Error, Allocator = C::Allocator>,
    {
        self.inner
            .visit_some(WithAllocator::new(decoder, self.allocator))
    }

    #[inline]
    fn visit_sequence<D>(self, decoder: &mut D) -> Result<Self::Ok, Self::Error>
    where
        D: ?Sized
            + crate::de::SequenceDecoder<
                'de,
                Cx = C,
                Error = Self::Error,
                Allocator = Self::Allocator,
            >,
    {
        self.inner
            .visit_sequence(WithAllocator::new(decoder, self.allocator))
    }

    #[inline]
    fn visit_map<D>(self, decoder: &mut D) -> Result<Self::Ok, Self::Error>
    where
        D: ?Sized
            + crate::de::MapDecoder<'de, Cx = C, Error = Self::Error, Allocator = Self::Allocator>,
    {
        self.inner
            .visit_map(WithAllocator::new(decoder, self.allocator))
    }

    #[inline]
    fn visit_string(self, cx: C, hint: SizeHint) -> Result<Self::String, Self::Error> {
        self.inner.visit_string(cx, hint)
    }

    #[inline]
    fn visit_bytes(self, cx: C, hint: SizeHint) -> Result<Self::Bytes, Self::Error> {
        self.inner.visit_bytes(cx, hint)
    }

    #[inline]
    fn visit_variant<D>(self, decoder: &mut D) -> Result<Self::Ok, Self::Error>
    where
        D: ?Sized
            + crate::de::VariantDecoder<'de, Cx = C, Error = C::Error, Allocator = C::Allocator>,
    {
        self.inner
            .visit_variant(WithAllocator::new(decoder, self.allocator))
    }

    #[inline]
    fn visit_unknown<D>(self, decoder: D) -> Result<Self::Ok, D::Error>
    where
        D: Decoder<'de, Cx = C, Error = C::Error, Allocator = C::Allocator>,
    {
        self.inner
            .visit_unknown(WithAllocator::new(decoder, self.allocator))
    }
}

impl<'de, I, A, C, T> UnsizedVisitor<'de, C, T> for WithAllocator<I, A>
where
    I: UnsizedVisitor<'de, C, T>,
    T: ?Sized + ToOwned,
{
    type Ok = I::Ok;
    type Error = I::Error;
    type Allocator = A;
    type __UseMusliUnsizedVisitorAttributeMacro = ();

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expecting(f)
    }

    #[inline]
    fn visit_owned(self, cx: C, value: T::Owned<Self::Allocator>) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_owned(cx, value)
    }

    #[inline]
    fn visit_borrowed(self, cx: C, value: &'de T) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_borrowed(cx, value)
    }

    #[inline]
    fn visit_ref(self, cx: C, value: &T) -> Result<Self::Ok, Self::Error> {
        self.inner.visit_ref(cx, value)
    }
}
