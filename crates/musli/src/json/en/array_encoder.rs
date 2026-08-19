use core::marker::PhantomData;
use core::mem::take;

use crate::en::SequenceEncoder;
use crate::{Context, Writer};

use super::JsonEncoder;

/// Encoder for a JSON array.
pub(crate) struct JsonArrayEncoder<W, C, M> {
    cx: C,
    first: bool,
    variant: bool,
    writer: W,
    _marker: PhantomData<M>,
}

impl<W, C, M> JsonArrayEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    #[inline]
    pub(super) fn new(cx: C, writer: W) -> Result<Self, C::Error> {
        Self::with_variant(cx, writer, false)
    }

    /// Construct an array encoder which, if `variant` is set, also closes an
    /// enclosing object once the array has been written. This is how an
    /// externally tagged sequence variant is encoded.
    #[inline]
    pub(super) fn with_variant(cx: C, mut writer: W, variant: bool) -> Result<Self, C::Error> {
        writer.begin_array(cx)?;
        writer.write_byte(cx, b'[')?;

        Ok(Self {
            cx,
            first: true,
            variant,
            writer,
            _marker: PhantomData,
        })
    }
}

impl<W, C, M> SequenceEncoder for JsonArrayEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeNext<'this>
        = JsonEncoder<W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        let first = take(&mut self.first);

        if !first {
            self.writer.write_byte(self.cx, b',')?;
        }

        self.writer.begin_array_element(self.cx, first)?;
        Ok(JsonEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_sequence(mut self) -> Result<(), C::Error> {
        self.writer.end_array(self.cx, self.first)?;
        self.writer.write_byte(self.cx, b']')?;

        if self.variant {
            self.writer.end_object(self.cx, false)?;
            self.writer.write_byte(self.cx, b'}')?;
        }

        Ok(())
    }
}
