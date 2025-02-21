use core::marker::PhantomData;

use crate::en::EntryEncoder;
use crate::{Context, Writer};

use super::{JsonEncoder, JsonObjectKeyEncoder};

/// Encoder for a JSON object pair.
pub(crate) struct JsonObjectPairEncoder<W, C, M> {
    cx: C,
    empty: bool,
    writer: W,
    _marker: PhantomData<M>,
}

impl<W, C, M> JsonObjectPairEncoder<W, C, M> {
    #[inline]
    pub(super) const fn new(cx: C, empty: bool, writer: W) -> Self {
        Self {
            cx,
            empty,
            writer,
            _marker: PhantomData,
        }
    }
}

impl<W, C, M> EntryEncoder for JsonObjectPairEncoder<W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Ok = ();
    type Error = C::Error;
    type Mode = M;
    type EncodeKey<'this>
        = JsonObjectKeyEncoder<W::Mut<'this>, C, M>
    where
        Self: 'this;
    type EncodeValue<'this>
        = JsonEncoder<W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_key(&mut self) -> Result<Self::EncodeKey<'_>, Self::Error> {
        if !self.empty {
            self.writer.write_byte(self.cx, b',')?;
        }

        Ok(JsonObjectKeyEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, Self::Error> {
        self.writer.write_byte(self.cx, b':')?;
        Ok(JsonEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_entry(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
