use core::marker::PhantomData;

use crate::en::VariantEncoder;
use crate::writer::BufWriter;
use crate::{Context, Writer};

use super::super::tag::OBJECT;
use super::{JsonbEncoder, JsonbObjectKeyEncoder, finish_container};

/// A JSONB variant encoder.
///
/// Variants are externally tagged, which means they are encoded as an object
/// with a single entry mapping the tag to the data of the variant.
pub(crate) struct JsonbVariantEncoder<W, C, M>
where
    C: Context,
{
    cx: C,
    writer: W,
    buffer: BufWriter<C::Allocator>,
    _marker: PhantomData<M>,
}

impl<W, C, M> JsonbVariantEncoder<W, C, M>
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
            buffer: BufWriter::new(cx.alloc()),
            _marker: PhantomData,
        }
    }
}

impl<W, C, M> VariantEncoder for JsonbVariantEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeTag<'this>
        = JsonbObjectKeyEncoder<&'this mut BufWriter<C::Allocator>, C, M>
    where
        Self: 'this;
    type EncodeData<'this>
        = JsonbEncoder<&'this mut BufWriter<C::Allocator>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, C::Error> {
        Ok(JsonbObjectKeyEncoder::new(self.cx, &mut self.buffer))
    }

    #[inline]
    fn encode_data(&mut self) -> Result<Self::EncodeData<'_>, C::Error> {
        Ok(JsonbEncoder::new(self.cx, &mut self.buffer))
    }

    #[inline]
    fn finish_variant(self) -> Result<(), C::Error> {
        finish_container(self.cx, self.writer, None, self.buffer, OBJECT)
    }
}
