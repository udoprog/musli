//! Decoding of CSV.
//!
//! The decoder mirrors the encoder:
//!
//! * [`CsvDecoder`] decodes a whole document. A sequence at this level is a
//!   sequence of records, anything else is a document with a single record.
//! * [`RecordDecoder`] decodes a single record. Structs are decoded by
//!   matching fields against the header row, sequences one field at a time,
//!   and anything else as a record with a single field.
//! * [`FieldDecoder`] decodes a single field, which has to be a scalar.

mod field_decoder;
use self::field_decoder::{EmptyDecoder, FieldDecoder};

mod fields_decoder;
use self::fields_decoder::FieldsDecoder;

mod key_decoder;
use self::key_decoder::{Key, KeyDecoder};

mod record_decoder;
pub(crate) use self::record_decoder::{RecordDecoder, RecordVariantDecoder};

use core::fmt;
use core::marker::PhantomData;

use crate::Context;
use crate::alloc::{Allocator, Vec};
use crate::context::nested;
use crate::de::{Decoder, SequenceDecoder, SizeHint, Skip, UnsizedVisitor};
use crate::hint::{MapHint, SequenceHint};

use super::encoding::Config;
use super::parser::{self, Parser};

/// The header row of a CSV document.
///
/// This is returned by [`Reader::headers`].
///
/// [`Reader::headers`]: crate::csv::Reader::headers
pub struct Headers<A>
where
    A: Allocator,
{
    bytes: Vec<u8, A>,
    ends: Vec<usize, A>,
}

impl<A> Headers<A>
where
    A: Allocator,
{
    #[inline]
    fn new_in(alloc: A) -> Self {
        Self {
            bytes: Vec::new_in(alloc),
            ends: Vec::new_in(alloc),
        }
    }

    /// The number of fields in the header row.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// let mut reader = Encoding::new().reader("name,age\nAristotle,61\n");
    /// let headers = reader.headers()?.expect("missing header row");
    ///
    /// assert_eq!(headers.len(), 2);
    /// assert!(!headers.is_empty());
    /// # Ok::<_, Error>(())
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.ends.len()
    }

    /// Test if the header row has no fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// let mut reader = Encoding::new().reader("name\nAristotle\n");
    /// let headers = reader.headers()?.expect("missing header row");
    /// assert!(!headers.is_empty());
    /// # Ok::<_, Error>(())
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ends.is_empty()
    }

    /// Get the name of the field at the given position.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// let mut reader = Encoding::new().reader("name,\"a \"\"quoted\"\" age\"\nAristotle,61\n");
    /// let headers = reader.headers()?.expect("missing header row");
    ///
    /// assert_eq!(headers.get(0), Some("name"));
    /// assert_eq!(headers.get(1), Some("a \"quoted\" age"));
    /// assert_eq!(headers.get(2), None);
    /// # Ok::<_, Error>(())
    /// ```
    #[inline]
    pub fn get(&self, index: usize) -> Option<&str> {
        let ends = self.ends.as_slice();
        let end = *ends.get(index)?;

        let start = match index.checked_sub(1) {
            Some(prev) => ends[prev],
            None => 0,
        };

        let bytes = self.bytes.as_slice().get(start..end)?;
        // SAFETY: Every field is validated as UTF-8 before it is added, and
        // fields are only ever split at the boundaries they were added at.
        Some(unsafe { core::str::from_utf8_unchecked(bytes) })
    }

    /// Iterate over the names of the fields in the header row.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// let mut reader = Encoding::new().reader("name,age\nAristotle,61\n");
    /// let headers = reader.headers()?.expect("missing header row");
    ///
    /// assert!(headers.iter().eq(["name", "age"]));
    /// # Ok::<_, Error>(())
    /// ```
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        (0..self.len()).filter_map(|index| self.get(index))
    }

    #[inline]
    fn push(&mut self, name: &str) -> Result<(), crate::alloc::AllocError> {
        self.bytes.extend_from_slice(name.as_bytes())?;
        self.ends.push(self.bytes.len())?;
        Ok(())
    }
}

impl<A> fmt::Debug for Headers<A>
where
    A: Allocator,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// The state of decoding a sequence of records.
pub(crate) struct DocState<A>
where
    A: Allocator,
{
    /// Whether the header row has been handled.
    header_read: bool,
    /// The header row, if one has been read.
    headers: Option<Headers<A>>,
    /// The number of fields every record is expected to have.
    expected: Option<usize>,
    /// The number of records decoded so far.
    records: usize,
    /// The allocator used for the header row.
    alloc: A,
}

impl<A> DocState<A>
where
    A: Allocator,
{
    #[inline]
    pub(crate) fn new(alloc: A) -> Self {
        Self {
            header_read: false,
            headers: None,
            expected: None,
            records: 0,
            alloc,
        }
    }

    /// The header row, if one has been read.
    #[inline]
    pub(crate) fn headers(&self) -> Option<&Headers<A>> {
        self.headers.as_ref()
    }

    /// Read the header row if it hasn't been read yet and headers are enabled.
    pub(crate) fn ensure_header<'de, P, C>(
        &mut self,
        cx: C,
        parser: &mut P,
        cfg: &Config,
    ) -> Result<(), C::Error>
    where
        P: ?Sized + Parser<'de>,
        C: Context,
    {
        if !cfg.has_headers || self.header_read {
            return Ok(());
        }

        self.header_read = true;

        if !parser::begin_record(cx, parser, cfg) {
            return Ok(());
        }

        let mut headers = Headers::new_in(self.alloc);
        let mut scratch = Vec::new_in(cx.alloc());

        while !parser.state().record_end {
            let mark = cx.mark();
            let field = parser::parse_field(cx, parser, cfg, &mut scratch)?;
            let name = crate::str::from_utf8(field.as_bytes())
                .map_err(|error| cx.custom_at(&mark, error))?;
            headers.push(name).map_err(cx.map())?;
        }

        self.expected = Some(headers.len());
        self.headers = Some(headers);
        Ok(())
    }

    /// Check that a record with `count` fields is acceptable.
    #[inline]
    pub(crate) fn finish_record<C>(&mut self, cx: C, cfg: &Config, count: usize) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.records += 1;

        match self.expected {
            None => {
                self.expected = Some(count);
            }
            Some(expected) if expected != count && !cfg.flexible => {
                return Err(cx.message(format_args!(
                    "Record {} has {count} fields, but expected {expected}",
                    self.records
                )));
            }
            Some(..) => {}
        }

        Ok(())
    }

    /// The number of the record currently being decoded, starting at 1.
    #[inline]
    pub(crate) fn record_number(&self) -> usize {
        self.records + 1
    }
}

/// A CSV decoder for a whole document.
pub(crate) struct CsvDecoder<P, C, M> {
    cx: C,
    parser: P,
    cfg: Config,
    _marker: PhantomData<M>,
}

impl<'de, P, C, M> CsvDecoder<P, C, M>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
{
    #[inline]
    pub(crate) fn new(cx: C, parser: P, cfg: Config) -> Self {
        Self {
            cx,
            parser,
            cfg,
            _marker: PhantomData,
        }
    }

    /// Decode the document as a single record.
    #[inline]
    fn record(self) -> Result<RecordDecoder<P, C, M, DocState<C::Allocator>>, C::Error> {
        Ok(RecordDecoder::new(
            self.cx,
            self.parser,
            self.cfg,
            DocState::new(self.cx.alloc()),
            false,
        ))
    }

    /// Decode the document as a sequence of records.
    #[inline]
    fn document<F, O>(self, f: F) -> Result<O, C::Error>
    where
        F: FnOnce(&mut DocumentDecoder<P, C, M>) -> Result<O, C::Error>,
    {
        let cx = self.cx;
        let mut decoder = DocumentDecoder::new(cx, self.parser, self.cfg);
        let output = nested(cx, || f(&mut decoder))?;
        decoder.skip_remaining()?;
        Ok(output)
    }
}

#[crate::trait_defaults(crate)]
impl<'de, P, C, M> Decoder<'de> for CsvDecoder<P, C, M>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodePack = DocumentDecoder<P, C, M>;
    type DecodeSequence = DocumentDecoder<P, C, M>;
    type DecodeMap = FieldsDecoder<P, C, M, DocState<C::Allocator>>;
    type DecodeMapEntries = FieldsDecoder<P, C, M, DocState<C::Allocator>>;
    type DecodeSome = Self;
    type DecodeVariant = RecordVariantDecoder<P, C, M, DocState<C::Allocator>>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from CSV")
    }

    #[inline]
    fn skip(self) -> Result<(), Self::Error> {
        self.document(|_| Ok(()))
    }

    #[inline]
    fn try_skip(self) -> Result<Skip, Self::Error> {
        self.skip()?;
        Ok(Skip::Skipped)
    }

    #[inline]
    fn decode_empty(self) -> Result<(), Self::Error> {
        self.record()?.decode_empty()
    }

    #[inline]
    fn decode_bool(self) -> Result<bool, Self::Error> {
        self.record()?.decode_bool()
    }

    #[inline]
    fn decode_char(self) -> Result<char, Self::Error> {
        self.record()?.decode_char()
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, Self::Error> {
        self.record()?.decode_u8()
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        self.record()?.decode_u16()
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        self.record()?.decode_u32()
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        self.record()?.decode_u64()
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, Self::Error> {
        self.record()?.decode_u128()
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        self.record()?.decode_i8()
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        self.record()?.decode_i16()
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        self.record()?.decode_i32()
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        self.record()?.decode_i64()
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, Self::Error> {
        self.record()?.decode_i128()
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, Self::Error> {
        self.record()?.decode_usize()
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, Self::Error> {
        self.record()?.decode_isize()
    }

    #[inline]
    fn decode_f32(self) -> Result<f32, Self::Error> {
        self.record()?.decode_f32()
    }

    #[inline]
    fn decode_f64(self) -> Result<f64, Self::Error> {
        self.record()?.decode_f64()
    }

    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, C, str, Error = Self::Error, Allocator = Self::Allocator>,
    {
        self.record()?.decode_string(visitor)
    }

    #[inline]
    fn decode_option(mut self) -> Result<Option<Self::DecodeSome>, Self::Error> {
        parser::skip_blank_lines(self.cx, &mut self.parser, &self.cfg);

        if self.parser.remaining().is_empty() {
            return Ok(None);
        }

        Ok(Some(self))
    }

    #[inline]
    fn decode_pack<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodePack) -> Result<O, Self::Error>,
    {
        self.document(f)
    }

    #[inline]
    fn decode_sequence<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        self.document(f)
    }

    #[inline]
    fn decode_sequence_hint<F, O>(self, _: impl SequenceHint, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        self.document(f)
    }

    #[inline]
    fn decode_map<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        self.record()?.decode_map(f)
    }

    #[inline]
    fn decode_map_hint<F, O>(self, hint: impl MapHint, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        self.record()?.decode_map_hint(hint, f)
    }

    #[inline]
    fn decode_map_entries<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMapEntries) -> Result<O, Self::Error>,
    {
        self.record()?.decode_map_entries(f)
    }

    #[inline]
    fn decode_variant<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeVariant) -> Result<O, Self::Error>,
    {
        self.record()?.decode_variant(f)
    }
}

/// A decoder for a sequence of records.
pub(crate) struct DocumentDecoder<P, C, M>
where
    C: Context,
{
    cx: C,
    parser: P,
    cfg: Config,
    doc: DocState<C::Allocator>,
    _marker: PhantomData<M>,
}

impl<'de, P, C, M> DocumentDecoder<P, C, M>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
{
    #[inline]
    fn new(cx: C, parser: P, cfg: Config) -> Self {
        Self {
            cx,
            parser,
            cfg,
            doc: DocState::new(cx.alloc()),
            _marker: PhantomData,
        }
    }

    /// Prepare to decode the next record, returning `false` if there are no
    /// more records.
    #[inline]
    fn next_record(&mut self) -> Result<bool, C::Error> {
        self.doc
            .ensure_header(self.cx, &mut self.parser, &self.cfg)?;
        Ok(parser::begin_record(self.cx, &mut self.parser, &self.cfg))
    }

    /// Skip over any remaining records.
    #[inline]
    fn skip_remaining(mut self) -> Result<(), C::Error> {
        while let Some(record) = self.try_decode_next()? {
            record.skip()?;
        }

        Ok(())
    }
}

impl<'de, P, C, M> SequenceDecoder<'de> for DocumentDecoder<P, C, M>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeNext<'this>
        = RecordDecoder<P::Mut<'this>, C, M, &'this mut DocState<C::Allocator>>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::any()
    }

    #[inline]
    fn try_decode_next(&mut self) -> Result<Option<Self::DecodeNext<'_>>, Self::Error> {
        if !self.next_record()? {
            return Ok(None);
        }

        Ok(Some(RecordDecoder::new(
            self.cx,
            self.parser.borrow_mut(),
            self.cfg,
            &mut self.doc,
            true,
        )))
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, Self::Error> {
        if !self.next_record()? {
            return Err(self
                .cx
                .message("Expected another record, but reached the end of the input"));
        }

        Ok(RecordDecoder::new(
            self.cx,
            self.parser.borrow_mut(),
            self.cfg,
            &mut self.doc,
            true,
        ))
    }
}
