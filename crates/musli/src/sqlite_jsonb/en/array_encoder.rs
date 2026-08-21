use core::marker::PhantomData;

use crate::en::SequenceEncoder;
use crate::writer::BufWriter;
use crate::{Context, Writer};

use super::super::tag::ARRAY;
use super::{JsonbEncoder, finish_container};

/// Encoder for a JSONB array.
///
/// The header of an array stores the size of its payload, which is only known
/// once every element has been written, so the payload is buffered.
pub(crate) struct JsonbArrayEncoder<W, C, M>
where
    C: Context,
{
    cx: C,
    writer: W,
    variant: Option<BufWriter<C::Allocator>>,
    buffer: BufWriter<C::Allocator>,
    _marker: PhantomData<M>,
}

impl<W, C, M> JsonbArrayEncoder<W, C, M>
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

    /// Construct an array encoder which wraps the array in the single entry
    /// object whose key `variant` already contains. This is how an externally
    /// tagged sequence variant is encoded.
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
}

impl<W, C, M> SequenceEncoder for JsonbArrayEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeNext<'this>
        = JsonbEncoder<&'this mut BufWriter<C::Allocator>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(JsonbEncoder::new(self.cx, &mut self.buffer))
    }

    #[inline]
    fn finish_sequence(self) -> Result<(), C::Error> {
        finish_container(self.cx, self.writer, self.variant, self.buffer, ARRAY)
    }
}
