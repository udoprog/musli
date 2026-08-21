use core::marker::PhantomData;

use crate::en::{EntriesEncoder, MapEncoder};
use crate::writer::BufWriter;
use crate::{Context, Writer};

use super::super::tag::OBJECT;
use super::{JsonbEncoder, JsonbObjectKeyEncoder, JsonbObjectPairEncoder, finish_container};

/// An object encoder for JSONB.
///
/// Just like for arrays the payload has to be buffered, since the header of an
/// object stores the size of its payload.
pub(crate) struct JsonbObjectEncoder<W, C, M>
where
    C: Context,
{
    cx: C,
    writer: W,
    variant: Option<BufWriter<C::Allocator>>,
    buffer: BufWriter<C::Allocator>,
    _marker: PhantomData<M>,
}

impl<W, C, M> JsonbObjectEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    #[inline]
    pub(super) fn new(cx: C, writer: W) -> Self {
        Self {
            cx,
            writer,
            variant: None,
            buffer: BufWriter::new(cx.alloc()),
            _marker: PhantomData,
        }
    }

    /// Construct an object encoder which wraps the object in the single entry
    /// object whose key `variant` already contains. This is how an externally
    /// tagged map variant is encoded.
    #[inline]
    pub(super) fn with_variant(cx: C, writer: W, variant: BufWriter<C::Allocator>) -> Self {
        Self {
            cx,
            writer,
            variant: Some(variant),
            buffer: BufWriter::new(cx.alloc()),
            _marker: PhantomData,
        }
    }

    #[inline]
    fn finish(self) -> Result<(), C::Error> {
        finish_container(self.cx, self.writer, self.variant, self.buffer, OBJECT)
    }
}

impl<W, C, M> MapEncoder for JsonbObjectEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeEntry<'this>
        = JsonbObjectPairEncoder<&'this mut BufWriter<C::Allocator>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, Self::Error> {
        Ok(JsonbObjectPairEncoder::new(self.cx, &mut self.buffer))
    }

    #[inline]
    fn finish_map(self) -> Result<(), Self::Error> {
        self.finish()
    }
}

impl<W, C, M> EntriesEncoder for JsonbObjectEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeEntryKey<'this>
        = JsonbObjectKeyEncoder<&'this mut BufWriter<C::Allocator>, C, M>
    where
        Self: 'this;
    type EncodeEntryValue<'this>
        = JsonbEncoder<&'this mut BufWriter<C::Allocator>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry_key(&mut self) -> Result<Self::EncodeEntryKey<'_>, Self::Error> {
        Ok(JsonbObjectKeyEncoder::new(self.cx, &mut self.buffer))
    }

    #[inline]
    fn encode_entry_value(&mut self) -> Result<Self::EncodeEntryValue<'_>, Self::Error> {
        Ok(JsonbEncoder::new(self.cx, &mut self.buffer))
    }

    #[inline]
    fn finish_entries(self) -> Result<(), Self::Error> {
        self.finish()
    }
}
