use core::marker::PhantomData;

use crate::en::{EntriesEncoder, MapEncoder};
use crate::{Context, Writer};

use super::{JsonEncoder, JsonObjectKeyEncoder, JsonObjectPairEncoder};

/// An object encoder for JSON.
pub(crate) struct JsonObjectEncoder<W, C, M> {
    cx: C,
    len: usize,
    variant: bool,
    writer: W,
    _marker: PhantomData<M>,
}

impl<W, C, M> JsonObjectEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    #[inline]
    pub(super) fn new(cx: C, writer: W) -> Result<Self, C::Error> {
        Self::with_variant(cx, writer, false)
    }

    /// Construct an object encoder which, if `variant` is set, also closes an
    /// enclosing object once this object has been written. This is how an
    /// externally tagged map variant is encoded.
    #[inline]
    pub(super) fn with_variant(cx: C, mut writer: W, variant: bool) -> Result<Self, C::Error> {
        writer.begin_object(cx)?;
        writer.write_byte(cx, b'{')?;

        Ok(Self {
            cx,
            len: 0,
            variant,
            writer,
            _marker: PhantomData,
        })
    }

    #[inline]
    fn finish(mut self) -> Result<(), C::Error> {
        self.writer.end_object(self.cx, self.len == 0)?;
        self.writer.write_byte(self.cx, b'}')?;

        if self.variant {
            self.writer.end_object(self.cx, false)?;
            self.writer.write_byte(self.cx, b'}')?;
        }

        Ok(())
    }
}

impl<W, C, M> MapEncoder for JsonObjectEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeEntry<'this>
        = JsonObjectPairEncoder<W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, Self::Error> {
        self.len += 1;

        Ok(JsonObjectPairEncoder::new(
            self.cx,
            self.len == 1,
            self.writer.borrow_mut(),
        ))
    }

    #[inline]
    fn finish_map(self) -> Result<(), Self::Error> {
        self.finish()
    }
}

impl<W, C, M> EntriesEncoder for JsonObjectEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeEntryKey<'this>
        = JsonObjectKeyEncoder<W::Mut<'this>, C, M>
    where
        Self: 'this;
    type EncodeEntryValue<'this>
        = JsonEncoder<W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry_key(&mut self) -> Result<Self::EncodeEntryKey<'_>, Self::Error> {
        let first = self.len == 0;

        if !first {
            self.writer.write_byte(self.cx, b',')?;
        }

        self.len += 1;
        self.writer.begin_object_key(self.cx, first)?;
        Ok(JsonObjectKeyEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_entry_value(&mut self) -> Result<Self::EncodeEntryValue<'_>, Self::Error> {
        self.writer.write_byte(self.cx, b':')?;
        self.writer.begin_object_value(self.cx)?;
        Ok(JsonEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_entries(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
