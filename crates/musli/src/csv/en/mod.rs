//! Encoding of CSV.
//!
//! The encoder is layered:
//!
//! * [`CsvEncoder`] encodes a whole document. A sequence at this level is a
//!   sequence of records, anything else is a document with a single record.
//! * [`RecordEncoder`] encodes a single record. Structs and sequences are
//!   encoded as one field per element, anything else as a record with a
//!   single field.
//! * [`FieldEncoder`] encodes a single field, which has to be a scalar.
//!
//! A header row is written by encoding the first record twice: once in
//! [`Pass::Header`] where only field names are written, and once in
//! [`Pass::Value`] where only values are written. This means that headers can
//! be written without having to buffer the record.

mod field_encoder;
pub(crate) use self::field_encoder::{EndRecord, FieldEncoder, FieldVariantEncoder};

mod fields_encoder;
pub(crate) use self::fields_encoder::FieldsEncoder;

mod sink;
pub(crate) use self::sink::Sink;

use core::borrow::BorrowMut;
use core::fmt;
use core::marker::PhantomData;

use crate::en::{Encode, Encoder, SequenceEncoder};
use crate::hint::{MapHint, SequenceHint};
use crate::{Context, Writer};

use super::encoding::Config;

/// The state of the encoding of a sequence of records.
pub(crate) struct EncState {
    /// Whether the header row has been handled.
    header_done: bool,
    /// The number of fields of the first record or the header row.
    fields: Option<usize>,
    /// The number of records written.
    records: usize,
}

impl EncState {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            header_done: false,
            fields: None,
            records: 0,
        }
    }
}

/// Which pass of encoding a record we are in.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pass {
    /// Only field names are written.
    Header,
    /// Only field values are written.
    Value,
}

/// Test if the given field needs to be quoted.
#[inline]
fn needs_quotes(cfg: &Config, bytes: &[u8]) -> bool {
    let term = cfg.terminator.read_byte();

    bytes.iter().any(|&b| {
        b == cfg.delimiter || b == cfg.quote || matches!(b, b'\r' | b'\n') || Some(b) == term
    })
}

/// Write the content of a single field, quoting it if necessary.
pub(crate) fn write_field<W, C>(cx: C, w: &mut W, cfg: &Config, bytes: &[u8]) -> Result<(), C::Error>
where
    W: ?Sized + Writer,
    C: Context,
{
    if !needs_quotes(cfg, bytes) {
        return w.write_bytes(cx, bytes);
    }

    w.write_byte(cx, cfg.quote)?;

    let mut start = 0;

    for (i, &b) in bytes.iter().enumerate() {
        if b == cfg.quote {
            // Write up to and including the quote, so that it is doubled up
            // when the next chunk starts with it.
            w.write_bytes(cx, &bytes[start..=i])?;
            start = i;
        }
    }

    w.write_bytes(cx, &bytes[start..])?;
    w.write_byte(cx, cfg.quote)?;
    Ok(())
}

/// Finish a record with `count` fields.
pub(crate) fn finish_record<W, C>(
    cx: C,
    w: &mut W,
    cfg: &Config,
    state: &mut EncState,
    pass: Pass,
    count: usize,
    last_empty: bool,
) -> Result<(), C::Error>
where
    W: ?Sized + Writer,
    C: Context,
{
    if count == 0 {
        return Err(cx.message("CSV records must contain at least one field"));
    }

    // A record with a single empty field would be indistinguishable from an
    // empty line which is skipped when reading, so it is written as an empty
    // quoted field.
    if count == 1 && last_empty {
        w.write_bytes(cx, &[cfg.quote, cfg.quote])?;
    }

    w.write_bytes(cx, cfg.terminator.write_bytes())?;

    match pass {
        Pass::Header => {
            state.fields = Some(count);
        }
        Pass::Value => {
            state.records += 1;

            match state.fields {
                None => {
                    state.fields = Some(count);
                }
                Some(expected) if expected != count && !cfg.flexible => {
                    return Err(cx.message(format_args!(
                        "Record {} has {count} fields, but expected {expected}",
                        state.records
                    )));
                }
                Some(..) => {}
            }
        }
    }

    Ok(())
}

/// The encoder for a whole CSV document.
pub(crate) struct CsvEncoder<'v, W, C, M, V>
where
    V: ?Sized,
{
    cx: C,
    writer: W,
    cfg: Config,
    value: &'v V,
    _marker: PhantomData<M>,
}

impl<'v, W, C, M, V> CsvEncoder<'v, W, C, M, V>
where
    W: Writer,
    C: Context,
    M: 'static,
    V: ?Sized + Encode<M>,
{
    /// Construct a new document encoder for the given value.
    #[inline]
    pub(crate) fn new(cx: C, writer: W, cfg: Config, value: &'v V) -> Self {
        Self {
            cx,
            writer,
            cfg,
            value,
            _marker: PhantomData,
        }
    }

    /// Encode the document as a single record.
    #[inline]
    fn record(self) -> Result<RecordEncoder<'v, W, C, M, V, EncState>, C::Error> {
        Ok(RecordEncoder::new(
            self.cx,
            self.writer,
            self.cfg,
            EncState::new(),
            Some(self.value),
            Pass::Value,
        ))
    }
}

#[crate::trait_defaults(crate)]
impl<'v, W, C, M, V> Encoder for CsvEncoder<'v, W, C, M, V>
where
    W: Writer,
    C: Context,
    M: 'static,
    V: ?Sized + Encode<M>,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodePack = DocumentEncoder<W, C, M>;
    type EncodeSome = Self;
    type EncodeSequence = DocumentEncoder<W, C, M>;
    type EncodeMap = FieldsEncoder<W, C, M, EncState>;
    type EncodeMapEntries = FieldsEncoder<W, C, M, EncState>;
    type EncodeVariant = FieldVariantEncoder<Sink<W>, C, M, EndRecord<EncState>>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be encoded to CSV")
    }

    #[inline]
    fn encode<T>(self, value: T) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        let value = value.as_encode();
        value.encode(CsvEncoder::new(self.cx, self.writer, self.cfg, value))
    }

    #[inline]
    fn encode_bool(self, value: bool) -> Result<(), Self::Error> {
        self.record()?.encode_bool(value)
    }

    #[inline]
    fn encode_char(self, value: char) -> Result<(), Self::Error> {
        self.record()?.encode_char(value)
    }

    #[inline]
    fn encode_u8(self, value: u8) -> Result<(), Self::Error> {
        self.record()?.encode_u8(value)
    }

    #[inline]
    fn encode_u16(self, value: u16) -> Result<(), Self::Error> {
        self.record()?.encode_u16(value)
    }

    #[inline]
    fn encode_u32(self, value: u32) -> Result<(), Self::Error> {
        self.record()?.encode_u32(value)
    }

    #[inline]
    fn encode_u64(self, value: u64) -> Result<(), Self::Error> {
        self.record()?.encode_u64(value)
    }

    #[inline]
    fn encode_u128(self, value: u128) -> Result<(), Self::Error> {
        self.record()?.encode_u128(value)
    }

    #[inline]
    fn encode_i8(self, value: i8) -> Result<(), Self::Error> {
        self.record()?.encode_i8(value)
    }

    #[inline]
    fn encode_i16(self, value: i16) -> Result<(), Self::Error> {
        self.record()?.encode_i16(value)
    }

    #[inline]
    fn encode_i32(self, value: i32) -> Result<(), Self::Error> {
        self.record()?.encode_i32(value)
    }

    #[inline]
    fn encode_i64(self, value: i64) -> Result<(), Self::Error> {
        self.record()?.encode_i64(value)
    }

    #[inline]
    fn encode_i128(self, value: i128) -> Result<(), Self::Error> {
        self.record()?.encode_i128(value)
    }

    #[inline]
    fn encode_usize(self, value: usize) -> Result<(), Self::Error> {
        self.record()?.encode_usize(value)
    }

    #[inline]
    fn encode_isize(self, value: isize) -> Result<(), Self::Error> {
        self.record()?.encode_isize(value)
    }

    #[inline]
    fn encode_f32(self, value: f32) -> Result<(), Self::Error> {
        self.record()?.encode_f32(value)
    }

    #[inline]
    fn encode_f64(self, value: f64) -> Result<(), Self::Error> {
        self.record()?.encode_f64(value)
    }

    #[inline]
    fn encode_string(self, value: &str) -> Result<(), Self::Error> {
        self.record()?.encode_string(value)
    }

    #[inline]
    fn encode_empty(self) -> Result<(), Self::Error> {
        self.record()?.encode_empty()
    }

    #[inline]
    fn encode_bytes(self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.record()?.encode_bytes(bytes)
    }

    #[inline]
    fn encode_variant(self) -> Result<Self::EncodeVariant, Self::Error> {
        self.record()?.encode_variant()
    }

    #[inline]
    fn encode_some(self) -> Result<Self::EncodeSome, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_none(self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::EncodePack, Self::Error> {
        Ok(DocumentEncoder::new(self.cx, self.writer, self.cfg))
    }

    #[inline]
    fn encode_sequence(self, _: impl SequenceHint) -> Result<Self::EncodeSequence, Self::Error> {
        Ok(DocumentEncoder::new(self.cx, self.writer, self.cfg))
    }

    #[inline]
    fn encode_map(self, hint: impl MapHint) -> Result<Self::EncodeMap, Self::Error> {
        self.record()?.encode_map(hint)
    }

    #[inline]
    fn encode_map_entries(self, hint: impl MapHint) -> Result<Self::EncodeMapEntries, Self::Error> {
        self.record()?.encode_map_entries(hint)
    }
}

/// Encoder for a sequence of records.
pub(crate) struct DocumentEncoder<W, C, M> {
    cx: C,
    writer: W,
    cfg: Config,
    state: EncState,
    _marker: PhantomData<M>,
}

impl<W, C, M> DocumentEncoder<W, C, M> {
    #[inline]
    fn new(cx: C, writer: W, cfg: Config) -> Self {
        Self {
            cx,
            writer,
            cfg,
            state: EncState::new(),
            _marker: PhantomData,
        }
    }
}

impl<W, C, M> SequenceEncoder for DocumentEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeNext<'this>
        = RecordEncoder<'this, W::Mut<'this>, C, M, (), &'this mut EncState>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, Self::Error> {
        Ok(RecordEncoder::new(
            self.cx,
            self.writer.borrow_mut(),
            self.cfg,
            &mut self.state,
            None,
            Pass::Value,
        ))
    }

    #[inline]
    fn finish_sequence(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Encoder for a single record.
///
/// If `value` is set, it refers to the value being encoded which is used to
/// write the header row if one is needed.
pub(crate) struct RecordEncoder<'v, W, C, M, V, S>
where
    V: ?Sized,
{
    cx: C,
    writer: W,
    cfg: Config,
    state: S,
    value: Option<&'v V>,
    pass: Pass,
    _marker: PhantomData<M>,
}

impl<'v, W, C, M, V, S> RecordEncoder<'v, W, C, M, V, S>
where
    W: Writer,
    C: Context,
    M: 'static,
    V: ?Sized + Encode<M>,
    S: BorrowMut<EncState>,
{
    /// Construct a new record encoder.
    #[inline]
    pub(crate) fn new(
        cx: C,
        writer: W,
        cfg: Config,
        state: S,
        value: Option<&'v V>,
        pass: Pass,
    ) -> Self {
        Self {
            cx,
            writer,
            cfg,
            state,
            value,
            pass,
            _marker: PhantomData,
        }
    }

    /// Begin encoding the record, writing a header row first if needed.
    #[inline]
    fn begin(&mut self) -> Result<(), C::Error> {
        if self.pass == Pass::Header || !self.cfg.has_headers {
            return Ok(());
        }

        let state = self.state.borrow_mut();

        if state.header_done {
            return Ok(());
        }

        state.header_done = true;

        let Some(value) = self.value else {
            return Err(self.cx.message(
                "Cannot write a header row since the record is not encoded through Encoder::encode",
            ));
        };

        value.encode(RecordEncoder::<_, _, M, V, _>::new(
            self.cx,
            self.writer.borrow_mut(),
            self.cfg,
            &mut *state,
            None,
            Pass::Header,
        ))
    }

    /// Encode the record as a single field.
    #[inline]
    fn single(mut self) -> Result<FieldEncoder<Sink<W>, C, M, EndRecord<S>>, C::Error> {
        self.begin()?;

        match self.pass {
            Pass::Header => {
                // The header of a single field record is its position.
                write_field(self.cx, &mut self.writer, &self.cfg, b"0")?;
                finish_record(
                    self.cx,
                    &mut self.writer,
                    &self.cfg,
                    self.state.borrow_mut(),
                    Pass::Header,
                    1,
                    false,
                )?;

                Ok(FieldEncoder::new(
                    self.cx,
                    Sink::Discard,
                    self.cfg,
                    EndRecord::new(self.state, Pass::Header, false),
                ))
            }
            Pass::Value => Ok(FieldEncoder::new(
                self.cx,
                Sink::Real(self.writer),
                self.cfg,
                EndRecord::new(self.state, Pass::Value, true),
            )),
        }
    }

    /// Encode the record as a collection of fields.
    #[inline]
    fn fields(mut self) -> Result<FieldsEncoder<W, C, M, S>, C::Error> {
        self.begin()?;
        Ok(FieldsEncoder::new(
            self.cx,
            self.writer,
            self.cfg,
            self.state,
            self.pass,
        ))
    }
}

#[crate::trait_defaults(crate)]
impl<'v, W, C, M, V, S> Encoder for RecordEncoder<'v, W, C, M, V, S>
where
    W: Writer,
    C: Context,
    M: 'static,
    V: ?Sized + Encode<M>,
    S: BorrowMut<EncState>,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodePack = FieldsEncoder<W, C, M, S>;
    type EncodeSome = Self;
    type EncodeSequence = FieldsEncoder<W, C, M, S>;
    type EncodeMap = FieldsEncoder<W, C, M, S>;
    type EncodeMapEntries = FieldsEncoder<W, C, M, S>;
    type EncodeVariant = FieldVariantEncoder<Sink<W>, C, M, EndRecord<S>>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be encoded as a CSV record")
    }

    #[inline]
    fn encode<T>(self, value: T) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        let value = value.as_encode();

        value.encode(RecordEncoder::new(
            self.cx,
            self.writer,
            self.cfg,
            self.state,
            Some(value),
            self.pass,
        ))
    }

    #[inline]
    fn encode_bool(self, value: bool) -> Result<(), Self::Error> {
        self.single()?.encode_bool(value)
    }

    #[inline]
    fn encode_char(self, value: char) -> Result<(), Self::Error> {
        self.single()?.encode_char(value)
    }

    #[inline]
    fn encode_u8(self, value: u8) -> Result<(), Self::Error> {
        self.single()?.encode_u8(value)
    }

    #[inline]
    fn encode_u16(self, value: u16) -> Result<(), Self::Error> {
        self.single()?.encode_u16(value)
    }

    #[inline]
    fn encode_u32(self, value: u32) -> Result<(), Self::Error> {
        self.single()?.encode_u32(value)
    }

    #[inline]
    fn encode_u64(self, value: u64) -> Result<(), Self::Error> {
        self.single()?.encode_u64(value)
    }

    #[inline]
    fn encode_u128(self, value: u128) -> Result<(), Self::Error> {
        self.single()?.encode_u128(value)
    }

    #[inline]
    fn encode_i8(self, value: i8) -> Result<(), Self::Error> {
        self.single()?.encode_i8(value)
    }

    #[inline]
    fn encode_i16(self, value: i16) -> Result<(), Self::Error> {
        self.single()?.encode_i16(value)
    }

    #[inline]
    fn encode_i32(self, value: i32) -> Result<(), Self::Error> {
        self.single()?.encode_i32(value)
    }

    #[inline]
    fn encode_i64(self, value: i64) -> Result<(), Self::Error> {
        self.single()?.encode_i64(value)
    }

    #[inline]
    fn encode_i128(self, value: i128) -> Result<(), Self::Error> {
        self.single()?.encode_i128(value)
    }

    #[inline]
    fn encode_usize(self, value: usize) -> Result<(), Self::Error> {
        self.single()?.encode_usize(value)
    }

    #[inline]
    fn encode_isize(self, value: isize) -> Result<(), Self::Error> {
        self.single()?.encode_isize(value)
    }

    #[inline]
    fn encode_f32(self, value: f32) -> Result<(), Self::Error> {
        self.single()?.encode_f32(value)
    }

    #[inline]
    fn encode_f64(self, value: f64) -> Result<(), Self::Error> {
        self.single()?.encode_f64(value)
    }

    #[inline]
    fn encode_string(self, value: &str) -> Result<(), Self::Error> {
        self.single()?.encode_string(value)
    }

    #[inline]
    fn encode_empty(self) -> Result<(), Self::Error> {
        self.single()?.encode_empty()
    }

    #[inline]
    fn encode_bytes(self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.single()?.encode_bytes(bytes)
    }

    #[inline]
    fn encode_variant(self) -> Result<Self::EncodeVariant, Self::Error> {
        self.single()?.encode_variant()
    }

    #[inline]
    fn encode_some(self) -> Result<Self::EncodeSome, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_none(mut self) -> Result<(), Self::Error> {
        if self.pass == Pass::Header {
            return self.single()?.encode_none();
        }

        self.begin()?;

        // A missing record is written as a single empty field, which is exempt
        // from the field count check since it has no fields.
        self.writer
            .write_bytes(self.cx, &[self.cfg.quote, self.cfg.quote])?;
        self.writer
            .write_bytes(self.cx, self.cfg.terminator.write_bytes())?;
        self.state.borrow_mut().records += 1;
        Ok(())
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::EncodePack, Self::Error> {
        self.fields()
    }

    #[inline]
    fn encode_sequence(self, _: impl SequenceHint) -> Result<Self::EncodeSequence, Self::Error> {
        self.fields()
    }

    #[inline]
    fn encode_map(self, _: impl MapHint) -> Result<Self::EncodeMap, Self::Error> {
        self.fields()
    }

    #[inline]
    fn encode_map_entries(self, _: impl MapHint) -> Result<Self::EncodeMapEntries, Self::Error> {
        self.fields()
    }
}
