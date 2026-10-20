use core::borrow::BorrowMut;
use core::marker::PhantomData;

use crate::en::{EntriesEncoder, EntryEncoder, MapEncoder, SequenceEncoder};
use crate::{Context, Writer};

use super::super::encoding::Config;
use super::{EncState, FieldEncoder, Pass, Sink, finish_record, write_field};

/// Encoder for the fields of a record, which is either a struct, a map or a
/// sequence.
///
/// In [`Pass::Header`] the names of fields are written, which for sequences is
/// their position. In [`Pass::Value`] only the values are written.
pub(crate) struct FieldsEncoder<W, C, M, S> {
    cx: C,
    writer: W,
    cfg: Config,
    state: S,
    pass: Pass,
    count: usize,
    last_empty: bool,
    ignored: bool,
    _marker: PhantomData<M>,
}

impl<W, C, M, S> FieldsEncoder<W, C, M, S>
where
    W: Writer,
    C: Context,
    M: 'static,
    S: BorrowMut<EncState>,
{
    #[inline]
    pub(crate) fn new(cx: C, writer: W, cfg: Config, state: S, pass: Pass) -> Self {
        Self {
            cx,
            writer,
            cfg,
            state,
            pass,
            count: 0,
            last_empty: false,
            ignored: false,
            _marker: PhantomData,
        }
    }

    /// Begin the next field, writing a delimiter if needed.
    #[inline]
    fn next_field(&mut self) -> Result<(), C::Error> {
        if self.pass == Pass::Value && !self.cfg.flexible {
            let state = self.state.borrow_mut();

            if let Some(expected) = state.fields {
                if self.count >= expected {
                    return Err(self.cx.message(format_args!(
                        "Record {} has more than the expected {expected} fields",
                        state.records + 1
                    )));
                }
            }
        }

        if self.count > 0 {
            self.writer.write_byte(self.cx, self.cfg.delimiter)?;
        }

        self.count += 1;
        Ok(())
    }

    /// An encoder for a field which is written.
    #[inline]
    fn write(&mut self) -> Result<FieldEncoder<Sink<W::Mut<'_>>, C, M, &mut bool>, C::Error> {
        self.next_field()?;

        Ok(FieldEncoder::new(
            self.cx,
            Sink::Real(self.writer.borrow_mut()),
            self.cfg,
            &mut self.last_empty,
        ))
    }

    /// An encoder for a value which is ignored in the current pass.
    #[inline]
    fn ignore(&mut self) -> FieldEncoder<Sink<W::Mut<'_>>, C, M, &mut bool> {
        FieldEncoder::new(self.cx, Sink::Discard, self.cfg, &mut self.ignored)
    }

    /// Encode a map key.
    #[inline]
    fn key(&mut self) -> Result<FieldEncoder<Sink<W::Mut<'_>>, C, M, &mut bool>, C::Error> {
        match self.pass {
            Pass::Header => self.write(),
            Pass::Value => Ok(self.ignore()),
        }
    }

    /// Encode a map value.
    #[inline]
    fn value(&mut self) -> Result<FieldEncoder<Sink<W::Mut<'_>>, C, M, &mut bool>, C::Error> {
        match self.pass {
            Pass::Header => Ok(self.ignore()),
            Pass::Value => self.write(),
        }
    }

    #[inline]
    fn finish(mut self) -> Result<(), C::Error> {
        finish_record(
            self.cx,
            &mut self.writer,
            &self.cfg,
            self.state.borrow_mut(),
            self.pass,
            self.count,
            self.last_empty,
        )
    }
}

impl<W, C, M, S> SequenceEncoder for FieldsEncoder<W, C, M, S>
where
    W: Writer,
    C: Context,
    M: 'static,
    S: BorrowMut<EncState>,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeNext<'this>
        = FieldEncoder<Sink<W::Mut<'this>>, C, M, &'this mut bool>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, Self::Error> {
        match self.pass {
            Pass::Header => {
                let index = self.count;
                self.next_field()?;
                let mut buffer = itoa::Buffer::new();
                write_field(
                    self.cx,
                    &mut self.writer,
                    &self.cfg,
                    buffer.format(index).as_bytes(),
                )?;
                self.last_empty = false;
                Ok(self.ignore())
            }
            Pass::Value => self.write(),
        }
    }

    #[inline]
    fn finish_sequence(self) -> Result<(), Self::Error> {
        self.finish()
    }
}

impl<W, C, M, S> MapEncoder for FieldsEncoder<W, C, M, S>
where
    W: Writer,
    C: Context,
    M: 'static,
    S: BorrowMut<EncState>,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeEntry<'this>
        = FieldsEntryEncoder<'this, W, C, M, S>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, Self::Error> {
        Ok(FieldsEntryEncoder { fields: self })
    }

    #[inline]
    fn finish_map(self) -> Result<(), Self::Error> {
        self.finish()
    }
}

impl<W, C, M, S> EntriesEncoder for FieldsEncoder<W, C, M, S>
where
    W: Writer,
    C: Context,
    M: 'static,
    S: BorrowMut<EncState>,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeEntryKey<'this>
        = FieldEncoder<Sink<W::Mut<'this>>, C, M, &'this mut bool>
    where
        Self: 'this;
    type EncodeEntryValue<'this>
        = FieldEncoder<Sink<W::Mut<'this>>, C, M, &'this mut bool>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry_key(&mut self) -> Result<Self::EncodeEntryKey<'_>, Self::Error> {
        self.key()
    }

    #[inline]
    fn encode_entry_value(&mut self) -> Result<Self::EncodeEntryValue<'_>, Self::Error> {
        self.value()
    }

    #[inline]
    fn finish_entries(self) -> Result<(), Self::Error> {
        self.finish()
    }
}

/// Encoder for a single entry in a struct or map.
pub(crate) struct FieldsEntryEncoder<'a, W, C, M, S> {
    fields: &'a mut FieldsEncoder<W, C, M, S>,
}

impl<W, C, M, S> EntryEncoder for FieldsEntryEncoder<'_, W, C, M, S>
where
    W: Writer,
    C: Context,
    M: 'static,
    S: BorrowMut<EncState>,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeKey<'this>
        = FieldEncoder<Sink<W::Mut<'this>>, C, M, &'this mut bool>
    where
        Self: 'this;
    type EncodeValue<'this>
        = FieldEncoder<Sink<W::Mut<'this>>, C, M, &'this mut bool>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.fields.cx
    }

    #[inline]
    fn encode_key(&mut self) -> Result<Self::EncodeKey<'_>, Self::Error> {
        self.fields.key()
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, Self::Error> {
        self.fields.value()
    }

    #[inline]
    fn finish_entry(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
