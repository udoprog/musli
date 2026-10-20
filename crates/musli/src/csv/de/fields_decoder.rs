use core::borrow::BorrowMut;
use core::marker::PhantomData;

use crate::Context;
use crate::de::{EntriesDecoder, EntryDecoder, MapDecoder, SequenceDecoder, SizeHint};

use super::super::encoding::Config;
use super::super::parser::{self, Parser};
use super::{DocState, FieldDecoder, Key, KeyDecoder};

/// A decoder for the fields of a record.
pub(crate) struct FieldsDecoder<P, C, M, S> {
    cx: C,
    parser: P,
    cfg: Config,
    doc: S,
    /// The number of fields which have been handed out or skipped.
    count: usize,
    /// The number of fields parsed when the last field decoder was handed
    /// out, which is used to detect if it was used or not.
    handed: Option<u64>,
    _marker: PhantomData<M>,
}

impl<'de, P, C, M, S> FieldsDecoder<P, C, M, S>
where
    P: Parser<'de>,
    C: Context,
    M: 'static,
    S: BorrowMut<DocState<C::Allocator>>,
{
    #[inline]
    pub(super) fn new(cx: C, parser: P, cfg: Config, doc: S) -> Self {
        Self {
            cx,
            parser,
            cfg,
            doc,
            count: 0,
            handed: None,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub(super) fn cx(&self) -> C {
        self.cx
    }

    /// Skip over the last field handed out if it wasn't used, and test if
    /// there are more fields in the record.
    #[inline]
    fn has_next(&mut self) -> Result<bool, C::Error> {
        if let Some(fields) = self.handed.take() {
            if self.parser.state().fields == fields {
                parser::skip_field(self.cx, &mut self.parser, &self.cfg)?;
            }
        }

        Ok(!self.parser.state().record_end)
    }

    /// Hand out a decoder for the next field, assuming that there is one.
    #[inline]
    fn next_field(&mut self) -> FieldDecoder<P::Mut<'_>, C, M> {
        self.count += 1;
        self.handed = Some(self.parser.state().fields);
        FieldDecoder::new(self.cx, self.parser.borrow_mut(), self.cfg)
    }

    /// Get a decoder for the next field, or error if there are no more fields
    /// in the record.
    #[inline]
    pub(super) fn field(&mut self) -> Result<FieldDecoder<P::Mut<'_>, C, M>, C::Error> {
        if !self.has_next()? {
            let number = self.doc.borrow_mut().record_number();

            return Err(self.cx.message(format_args!(
                "Record {number} has {} fields, but more were expected",
                self.count
            )));
        }

        Ok(self.next_field())
    }

    /// Get the key of the next field, if there is one.
    #[inline]
    fn next_key(&mut self) -> Result<Option<Key<'_>>, C::Error> {
        if !self.has_next()? {
            let doc = self.doc.borrow_mut();

            // Report short records before the fields are found to be missing.
            if let Some(expected) = doc.expected {
                if self.count < expected && !self.cfg.flexible {
                    return Err(self.cx.message(format_args!(
                        "Record {} has {} fields, but expected {expected}",
                        doc.record_number(),
                        self.count
                    )));
                }
            }

            return Ok(None);
        }

        let doc = self.doc.borrow_mut();

        let Some(headers) = doc.headers.as_ref() else {
            return Ok(Some(Key::Index(self.count)));
        };

        if let Some(name) = headers.get(self.count) {
            return Ok(Some(Key::Name(name)));
        }

        if self.cfg.flexible {
            // Extra fields without a header are skipped.
            self.count += parser::skip_record(self.cx, &mut self.parser, &self.cfg)?;
            return Ok(None);
        }

        let number = doc.record_number();

        Err(self.cx.message(format_args!(
            "Record {number} has more fields than the {} in the header row",
            headers.len()
        )))
    }

    /// Finish the record, which is expected to consist of a single field.
    #[inline]
    pub(super) fn finish_single(self) -> Result<(), C::Error> {
        self.finish_sequence(Some(1))
    }

    /// Finish decoding a sequence, where `expected` is the number of fields
    /// expected if known.
    #[inline]
    pub(super) fn finish_sequence(mut self, expected: Option<usize>) -> Result<(), C::Error> {
        self.has_next()?;
        let remaining = parser::skip_record(self.cx, &mut self.parser, &self.cfg)?;
        let count = self.count + remaining;

        if let (Some(expected), true) = (expected, remaining > 0 && !self.cfg.flexible) {
            let number = self.doc.borrow_mut().record_number();

            return Err(self.cx.message(format_args!(
                "Record {number} has {count} fields, but expected {expected}"
            )));
        }

        self.doc
            .borrow_mut()
            .finish_record(self.cx, &self.cfg, count)
    }

    /// Finish decoding a map.
    #[inline]
    pub(super) fn finish_map(self) -> Result<(), C::Error> {
        self.finish_sequence(None)
    }
}

impl<'de, P, C, M, S> SequenceDecoder<'de> for FieldsDecoder<P, C, M, S>
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
    type DecodeNext<'this>
        = FieldDecoder<P::Mut<'this>, C, M>
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
        if !self.has_next()? {
            return Ok(None);
        }

        Ok(Some(self.next_field()))
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, Self::Error> {
        self.field()
    }
}

impl<'de, P, C, M, S> MapDecoder<'de> for FieldsDecoder<P, C, M, S>
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
    type DecodeEntry<'this>
        = FieldsEntryDecoder<'this, P, C, M, S>
    where
        Self: 'this;
    type DecodeRemainingEntries<'this>
        = &'this mut Self
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
    fn decode_entry(&mut self) -> Result<Option<Self::DecodeEntry<'_>>, Self::Error> {
        if self.next_key()?.is_none() {
            return Ok(None);
        }

        Ok(Some(FieldsEntryDecoder { fields: self }))
    }

    #[inline]
    fn decode_remaining_entries(
        &mut self,
    ) -> Result<Self::DecodeRemainingEntries<'_>, Self::Error> {
        Ok(self)
    }
}

/// Decoder for a single entry of a struct or map.
pub(crate) struct FieldsEntryDecoder<'a, P, C, M, S> {
    fields: &'a mut FieldsDecoder<P, C, M, S>,
}

impl<'a, 'de, P, C, M, S> EntryDecoder<'de> for FieldsEntryDecoder<'a, P, C, M, S>
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
    type DecodeKey<'this>
        = KeyDecoder<'this, C, M>
    where
        Self: 'this;
    type DecodeValue = FieldDecoder<P::Mut<'a>, C, M>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.fields.cx
    }

    #[inline]
    fn decode_key(&mut self) -> Result<Self::DecodeKey<'_>, Self::Error> {
        let cx = self.fields.cx;

        match self.fields.next_key()? {
            Some(key) => Ok(KeyDecoder::new(cx, key)),
            None => Err(cx.message("No more fields in record")),
        }
    }

    #[inline]
    fn decode_value(self) -> Result<Self::DecodeValue, Self::Error> {
        self.fields.field()
    }
}

impl<'de, P, C, M, S> EntriesDecoder<'de> for FieldsDecoder<P, C, M, S>
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
    type DecodeEntryKey<'this>
        = KeyDecoder<'this, C, M>
    where
        Self: 'this;
    type DecodeEntryValue<'this>
        = FieldDecoder<P::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_entry_key(&mut self) -> Result<Option<Self::DecodeEntryKey<'_>>, Self::Error> {
        let cx = self.cx;
        Ok(self.next_key()?.map(|key| KeyDecoder::new(cx, key)))
    }

    #[inline]
    fn decode_entry_value(&mut self) -> Result<Self::DecodeEntryValue<'_>, Self::Error> {
        self.field()
    }

    #[inline]
    fn end_entries(self) -> Result<(), Self::Error> {
        // The record is finished by its owner.
        Ok(())
    }
}


impl<'de, P, C, M, S> EntriesDecoder<'de> for &mut FieldsDecoder<P, C, M, S>
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
    type DecodeEntryKey<'this>
        = KeyDecoder<'this, C, M>
    where
        Self: 'this;
    type DecodeEntryValue<'this>
        = FieldDecoder<P::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_entry_key(&mut self) -> Result<Option<Self::DecodeEntryKey<'_>>, Self::Error> {
        EntriesDecoder::decode_entry_key(&mut **self)
    }

    #[inline]
    fn decode_entry_value(&mut self) -> Result<Self::DecodeEntryValue<'_>, Self::Error> {
        EntriesDecoder::decode_entry_value(&mut **self)
    }

    #[inline]
    fn end_entries(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
