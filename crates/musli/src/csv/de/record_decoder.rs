use core::borrow::BorrowMut;
use core::fmt;
use core::marker::PhantomData;

use crate::Context;
use crate::context::nested;
use crate::de::{Decoder, Skip, UnsizedVisitor, VariantDecoder};
use crate::hint::{MapHint, SequenceHint};

use super::super::encoding::Config;
use super::super::parser::{self, Parser};
use super::{DocState, EmptyDecoder, FieldDecoder, FieldsDecoder};

/// A decoder for a single record.
pub(crate) struct RecordDecoder<P, C, M, S> {
    cx: C,
    parser: P,
    cfg: Config,
    doc: S,
    /// Whether the record has been started, which means that the header row
    /// has been handled and that a record is known to be present.
    started: bool,
    _marker: PhantomData<M>,
}

impl<'de, P, C, M, S> RecordDecoder<P, C, M, S>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
    S: BorrowMut<DocState<C::Allocator>>,
{
    #[inline]
    pub(crate) fn new(cx: C, parser: P, cfg: Config, doc: S, started: bool) -> Self {
        Self {
            cx,
            parser,
            cfg,
            doc,
            started,
            _marker: PhantomData,
        }
    }

    /// Make sure that the record is started.
    #[inline]
    fn begin(&mut self) -> Result<(), C::Error> {
        if self.started {
            return Ok(());
        }

        self.started = true;

        self.doc
            .borrow_mut()
            .ensure_header(self.cx, &mut self.parser, &self.cfg)?;

        if !parser::begin_record(self.cx, &mut self.parser, &self.cfg) {
            return Err(self
                .cx
                .message("Expected a record, but reached the end of the input"));
        }

        Ok(())
    }

    /// Decode the fields of the record.
    #[inline]
    fn fields(mut self) -> Result<FieldsDecoder<P, C, M, S>, C::Error> {
        self.begin()?;
        Ok(FieldsDecoder::new(self.cx, self.parser, self.cfg, self.doc))
    }

    /// Decode the record as a single field.
    #[inline]
    fn single(self) -> Result<Single<P, C, M, S>, C::Error> {
        Ok(Single {
            fields: self.fields()?,
        })
    }
}

/// Helper to decode a record which consists of a single field.
struct Single<P, C, M, S> {
    fields: FieldsDecoder<P, C, M, S>,
}

macro_rules! single_scalars {
    ($($method:ident($ty:ty)),* $(,)?) => {
        $(
            #[inline]
            fn $method(mut self) -> Result<$ty, C::Error> {
                let value = self.fields.field()?.$method()?;
                self.fields.finish_single()?;
                Ok(value)
            }
        )*
    };
}

impl<'de, P, C, M, S> Single<P, C, M, S>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
    S: BorrowMut<DocState<C::Allocator>>,
{
    single_scalars! {
        decode_empty(()),
        decode_bool(bool),
        decode_char(char),
        decode_u8(u8),
        decode_u16(u16),
        decode_u32(u32),
        decode_u64(u64),
        decode_u128(u128),
        decode_i8(i8),
        decode_i16(i16),
        decode_i32(i32),
        decode_i64(i64),
        decode_i128(i128),
        decode_usize(usize),
        decode_isize(isize),
        decode_f32(f32),
        decode_f64(f64),
    }

    #[inline]
    fn decode_string<V>(mut self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, C, str, Error = C::Error, Allocator = C::Allocator>,
    {
        let value = self.fields.field()?.decode_string(visitor)?;
        self.fields.finish_single()?;
        Ok(value)
    }
}

#[crate::trait_defaults(crate)]
impl<'de, P, C, M, S> Decoder<'de> for RecordDecoder<P, C, M, S>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
    S: BorrowMut<DocState<C::Allocator>>,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodePack = FieldsDecoder<P, C, M, S>;
    type DecodeSequence = FieldsDecoder<P, C, M, S>;
    type DecodeMap = FieldsDecoder<P, C, M, S>;
    type DecodeMapEntries = FieldsDecoder<P, C, M, S>;
    type DecodeSome = Self;
    type DecodeVariant = RecordVariantDecoder<P, C, M, S>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from a CSV record")
    }

    #[inline]
    fn skip(mut self) -> Result<(), Self::Error> {
        self.begin()?;
        parser::skip_record(self.cx, &mut self.parser, &self.cfg)?;
        self.doc.borrow_mut().records += 1;
        Ok(())
    }

    #[inline]
    fn try_skip(self) -> Result<Skip, Self::Error> {
        self.skip()?;
        Ok(Skip::Skipped)
    }

    #[inline]
    fn decode_empty(self) -> Result<(), Self::Error> {
        self.single()?.decode_empty()
    }

    #[inline]
    fn decode_bool(self) -> Result<bool, Self::Error> {
        self.single()?.decode_bool()
    }

    #[inline]
    fn decode_char(self) -> Result<char, Self::Error> {
        self.single()?.decode_char()
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, Self::Error> {
        self.single()?.decode_u8()
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        self.single()?.decode_u16()
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        self.single()?.decode_u32()
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        self.single()?.decode_u64()
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, Self::Error> {
        self.single()?.decode_u128()
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        self.single()?.decode_i8()
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        self.single()?.decode_i16()
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        self.single()?.decode_i32()
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        self.single()?.decode_i64()
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, Self::Error> {
        self.single()?.decode_i128()
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, Self::Error> {
        self.single()?.decode_usize()
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, Self::Error> {
        self.single()?.decode_isize()
    }

    #[inline]
    fn decode_f32(self) -> Result<f32, Self::Error> {
        self.single()?.decode_f32()
    }

    #[inline]
    fn decode_f64(self) -> Result<f64, Self::Error> {
        self.single()?.decode_f64()
    }

    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, C, str, Error = Self::Error, Allocator = Self::Allocator>,
    {
        self.single()?.decode_string(visitor)
    }

    #[inline]
    fn decode_option(mut self) -> Result<Option<Self::DecodeSome>, Self::Error> {
        self.begin()?;

        // A missing record is a single empty field, which is exempt from the
        // field count check since it has no fields.
        if parser::peek_empty(&mut self.parser, &self.cfg, true).is_some() {
            parser::skip_field(self.cx, &mut self.parser, &self.cfg)?;
            self.doc.borrow_mut().records += 1;
            return Ok(None);
        }

        Ok(Some(self))
    }

    #[inline]
    fn decode_pack<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodePack) -> Result<O, Self::Error>,
    {
        let cx = self.cx;
        let mut fields = self.fields()?;
        let output = nested(cx, || f(&mut fields))?;
        fields.finish_sequence(None)?;
        Ok(output)
    }

    #[inline]
    fn decode_sequence<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        let cx = self.cx;
        let mut fields = self.fields()?;
        let output = nested(cx, || f(&mut fields))?;
        fields.finish_sequence(None)?;
        Ok(output)
    }

    #[inline]
    fn decode_sequence_hint<F, O>(self, hint: impl SequenceHint, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        let cx = self.cx;
        let mut fields = self.fields()?;
        let output = nested(cx, || f(&mut fields))?;
        fields.finish_sequence(hint.get())?;
        Ok(output)
    }

    #[inline]
    fn decode_map<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        let cx = self.cx;
        let mut fields = self.fields()?;
        let output = nested(cx, || f(&mut fields))?;
        fields.finish_map()?;
        Ok(output)
    }

    #[inline]
    fn decode_map_hint<F, O>(self, _: impl MapHint, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        self.decode_map(f)
    }

    #[inline]
    fn decode_map_entries<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMapEntries) -> Result<O, Self::Error>,
    {
        self.decode_map(f)
    }

    #[inline]
    fn decode_variant<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeVariant) -> Result<O, Self::Error>,
    {
        let mut decoder = RecordVariantDecoder {
            fields: self.fields()?,
        };

        let output = f(&mut decoder)?;
        decoder.fields.finish_single()?;
        Ok(output)
    }
}

/// A decoder for a record which consists of a single unit variant.
pub(crate) struct RecordVariantDecoder<P, C, M, S> {
    fields: FieldsDecoder<P, C, M, S>,
}

impl<'de, P, C, M, S> VariantDecoder<'de> for RecordVariantDecoder<P, C, M, S>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
    S: BorrowMut<DocState<C::Allocator>>,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeTag<'this>
        = FieldDecoder<P::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeValue<'this>
        = EmptyDecoder<C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.fields.cx()
    }

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, Self::Error> {
        self.fields.field()
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeValue<'_>, Self::Error> {
        Ok(EmptyDecoder::new(self.fields.cx()))
    }
}
