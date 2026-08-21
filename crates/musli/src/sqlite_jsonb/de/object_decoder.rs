use core::marker::PhantomData;

use crate::Context;
use crate::de::{EntriesDecoder, MapDecoder, SizeHint};

use super::super::cursor::Cursor;
use super::{JsonbDecoder, JsonbKeyDecoder, JsonbObjectPairDecoder};

/// A decoder over the payload of a JSONB object, which is the keys and values
/// of its entries written after one another.
pub(crate) struct JsonbObjectDecoder<P, C, M> {
    cx: C,
    cursor: P,
    _marker: PhantomData<M>,
}

impl<'de, P, C, M> JsonbObjectDecoder<P, C, M>
where
    P: Cursor<'de>,
    C: Context,
    M: 'static,
{
    #[inline]
    pub(super) fn new(cx: C, cursor: P) -> Self {
        Self {
            cx,
            cursor,
            _marker: PhantomData,
        }
    }

    /// Finish decoding the object.
    ///
    /// Any entries which have not been decoded are contained in the payload
    /// which the enclosing decoder has already moved past.
    #[inline]
    pub(super) fn end(self) -> Result<(), C::Error> {
        Ok(())
    }
}

impl<'de, P, C, M> MapDecoder<'de> for JsonbObjectDecoder<P, C, M>
where
    P: Cursor<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeEntry<'this>
        = JsonbObjectPairDecoder<P::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeRemainingEntries<'this>
        = JsonbObjectDecoder<P::Mut<'this>, C, M>
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
        if self.cursor.remaining() == 0 {
            return Ok(None);
        }

        Ok(Some(JsonbObjectPairDecoder::new(
            self.cx,
            self.cursor.borrow_mut(),
        )))
    }

    #[inline]
    fn decode_remaining_entries(
        &mut self,
    ) -> Result<Self::DecodeRemainingEntries<'_>, Self::Error> {
        Ok(JsonbObjectDecoder::new(self.cx, self.cursor.borrow_mut()))
    }
}

impl<'de, P, C, M> EntriesDecoder<'de> for JsonbObjectDecoder<P, C, M>
where
    P: Cursor<'de>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;
    type DecodeEntryKey<'this>
        = JsonbKeyDecoder<P::Mut<'this>, C, M>
    where
        Self: 'this;
    type DecodeEntryValue<'this>
        = JsonbDecoder<P::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_entry_key(&mut self) -> Result<Option<Self::DecodeEntryKey<'_>>, Self::Error> {
        if self.cursor.remaining() == 0 {
            return Ok(None);
        }

        Ok(Some(JsonbKeyDecoder::new(
            self.cx,
            self.cursor.borrow_mut(),
        )))
    }

    #[inline]
    fn decode_entry_value(&mut self) -> Result<Self::DecodeEntryValue<'_>, Self::Error> {
        Ok(JsonbDecoder::new(self.cx, self.cursor.borrow_mut()))
    }

    #[inline]
    fn end_entries(self) -> Result<(), Self::Error> {
        self.end()
    }
}
