use core::mem::size_of_val;
use core::{fmt, slice};

use crate::en::{
    utils, Encode, Encoder, EntriesEncoder, EntryEncoder, MapEncoder, SequenceEncoder,
    TryFastEncode, VariantEncoder,
};
use crate::hint::{MapHint, SequenceHint};
use crate::options::is_native_fixed;
use crate::{Context, Options, Writer};

/// A vaery simple encoder suitable for storage encoding.
pub struct StorageEncoder<'a, W, const OPT: Options, C: ?Sized> {
    cx: &'a C,
    writer: W,
}

impl<'a, W, const OPT: Options, C: ?Sized> StorageEncoder<'a, W, OPT, C> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub fn new(cx: &'a C, writer: W) -> Self {
        Self { cx, writer }
    }
}

#[crate::encoder(crate)]
impl<'a, W, const OPT: Options, C> Encoder for StorageEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Error = C::Error;
    type Ok = ();
    type Mode = C::Mode;
    type WithContext<'this, U>
        = StorageEncoder<'this, W, OPT, U>
    where
        U: 'this + Context;
    type EncodePack = StorageEncoder<'a, W, OPT, C>;
    type EncodeSome = Self;
    type EncodeSequence = Self;
    type EncodeMap = Self;
    type EncodeMapEntries = Self;
    type EncodeVariant = Self;
    type EncodeSequenceVariant = Self;
    type EncodeMapVariant = Self;

    #[inline]
    fn cx<F, O>(self, f: F) -> O
    where
        F: FnOnce(&Self::Cx, Self) -> O,
    {
        f(self.cx, self)
    }

    #[inline]
    fn with_context<U>(self, cx: &U) -> Result<Self::WithContext<'_, U>, C::Error>
    where
        U: Context,
    {
        Ok(StorageEncoder::new(cx, self.writer))
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the storage encoder")
    }

    #[inline]
    fn try_fast_encode<T>(mut self, value: T) -> Result<TryFastEncode<T, Self>, Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        if !const { is_native_fixed::<OPT>() && T::Encode::IS_BITWISE_ENCODE } {
            return Ok(TryFastEncode::Unsupported(value, self));
        }

        let value = value.as_encode();

        // SAFETY: We've ensured the type is layout compatible with the current
        // serialization just above.
        let slice = unsafe {
            let at = (value as *const T::Encode).cast::<u8>();
            slice::from_raw_parts(at, size_of_val(value))
        };

        self.writer.write_bytes(self.cx, slice)?;
        Ok(TryFastEncode::Ok(()))
    }

    #[inline]
    fn encode_empty(self) -> Result<Self::Ok, C::Error> {
        static HINT: SequenceHint = SequenceHint::with_size(0);
        self.encode_sequence_fn(&HINT, |_| Ok(()))
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::EncodePack, C::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_array<const N: usize>(mut self, array: &[u8; N]) -> Result<Self::Ok, C::Error> {
        self.writer.write_bytes(self.cx, array)
    }

    #[inline]
    fn encode_bytes(mut self, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), bytes.len())?;
        self.writer.write_bytes(self.cx, bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored<I>(mut self, len: usize, vectors: I) -> Result<Self::Ok, C::Error>
    where
        I: IntoIterator<Item: AsRef<[u8]>>,
    {
        crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), len)?;

        for bytes in vectors {
            self.writer.write_bytes(self.cx, bytes.as_ref())?;
        }

        Ok(())
    }

    #[inline]
    fn encode_string(mut self, string: &str) -> Result<Self::Ok, C::Error> {
        crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), string.len())?;
        self.writer.write_bytes(self.cx, string.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn collect_string<T>(self, value: &T) -> Result<Self::Ok, <Self::Cx as Context>::Error>
    where
        T: ?Sized + fmt::Display,
    {
        let buf = self.cx.collect_string(value)?;
        self.encode_string(buf.as_ref())
    }

    #[inline]
    fn encode_bool(mut self, value: bool) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(self.cx, if value { 1 } else { 0 })
    }

    #[inline]
    fn encode_char(self, value: char) -> Result<Self::Ok, C::Error> {
        self.encode_u32(value as u32)
    }

    #[inline]
    fn encode_u8(mut self, value: u8) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(self.cx, value)
    }

    #[inline]
    fn encode_u16(mut self, value: u16) -> Result<Self::Ok, C::Error> {
        crate::int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<Self::Ok, C::Error> {
        crate::int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<Self::Ok, C::Error> {
        crate::int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<Self::Ok, C::Error> {
        crate::int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i8(self, value: i8) -> Result<Self::Ok, C::Error> {
        self.encode_u8(value as u8)
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<Self::Ok, C::Error> {
        crate::int::encode_signed::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<Self::Ok, C::Error> {
        crate::int::encode_signed::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<Self::Ok, C::Error> {
        crate::int::encode_signed::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<Self::Ok, C::Error> {
        crate::int::encode_signed::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_f32(self, value: f32) -> Result<Self::Ok, C::Error> {
        self.encode_u32(value.to_bits())
    }

    #[inline]
    fn encode_f64(self, value: f64) -> Result<Self::Ok, C::Error> {
        self.encode_u64(value.to_bits())
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<Self::Ok, C::Error> {
        crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_isize(self, value: isize) -> Result<Self::Ok, C::Error> {
        self.encode_usize(value as usize)
    }

    #[inline]
    fn encode_some(mut self) -> Result<Self::EncodeSome, C::Error> {
        self.writer.write_byte(self.cx, 1)?;
        Ok(self)
    }

    #[inline]
    fn encode_none(mut self) -> Result<Self::Ok, C::Error> {
        self.writer.write_byte(self.cx, 0)?;
        Ok(())
    }

    #[inline]
    fn encode_slice<T>(mut self, slice: &[T]) -> Result<Self::Ok, C::Error>
    where
        T: Encode<Self::Mode>,
    {
        // Check that the type is packed inside of the slice.
        if !const {
            is_native_fixed::<OPT>()
                && T::IS_BITWISE_ENCODE
                && size_of::<T>() % align_of::<T>() == 0
        } {
            return utils::default_encode_slice(self, slice);
        }

        let len = slice.len().to_ne_bytes();
        self.writer.write_bytes(self.cx, &len)?;

        if size_of::<T>() > 0 {
            // SAFETY: We've ensured the type is layout compatible with the
            // current serialization just above.
            let slice = unsafe {
                let at = slice.as_ptr().cast::<u8>();
                let size = size_of_val(slice);
                slice::from_raw_parts(at, size)
            };

            self.writer.write_bytes(self.cx, slice)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_sequence(mut self, hint: &SequenceHint) -> Result<Self::EncodeSequence, C::Error> {
        crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), hint.size)?;
        Ok(self)
    }

    #[inline]
    fn encode_map(mut self, hint: &MapHint) -> Result<Self::EncodeMap, C::Error> {
        crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), hint.size)?;
        Ok(self)
    }

    #[inline]
    fn encode_map_entries(mut self, hint: &MapHint) -> Result<Self::EncodeMapEntries, C::Error> {
        crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), hint.size)?;
        Ok(self)
    }

    #[inline]
    fn encode_variant(self) -> Result<Self::EncodeVariant, C::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_sequence_variant<T>(
        mut self,
        tag: &T,
        hint: &SequenceHint,
    ) -> Result<Self::EncodeSequenceVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        StorageEncoder::<_, OPT, _>::new(self.cx, self.writer.borrow_mut()).encode(tag)?;
        crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), hint.size)?;
        Ok(self)
    }

    #[inline]
    fn encode_map_variant<T>(
        mut self,
        tag: &T,
        hint: &MapHint,
    ) -> Result<Self::EncodeMapVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        StorageEncoder::<_, OPT, _>::new(self.cx, self.writer.borrow_mut()).encode(tag)?;
        crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), hint.size)?;
        Ok(self)
    }
}

impl<'a, W, const OPT: Options, C> SequenceEncoder for StorageEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeNext<'this>
        = StorageEncoder<'a, W::Mut<'this>, OPT, C>
    where
        Self: 'this;

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_sequence(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> MapEncoder for StorageEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeEntry<'this>
        = StorageEncoder<'a, W::Mut<'this>, OPT, C>
    where
        Self: 'this;

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_map(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> EntryEncoder for StorageEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeKey<'this>
        = StorageEncoder<'a, W::Mut<'this>, OPT, C>
    where
        Self: 'this;
    type EncodeValue<'this>
        = StorageEncoder<'a, W::Mut<'this>, OPT, C>
    where
        Self: 'this;

    #[inline]
    fn encode_key(&mut self) -> Result<Self::EncodeKey<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_entry(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> EntriesEncoder for StorageEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeEntryKey<'this>
        = StorageEncoder<'a, W::Mut<'this>, OPT, C>
    where
        Self: 'this;
    type EncodeEntryValue<'this>
        = StorageEncoder<'a, W::Mut<'this>, OPT, C>
    where
        Self: 'this;

    #[inline]
    fn encode_entry_key(&mut self) -> Result<Self::EncodeEntryKey<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_entry_value(&mut self) -> Result<Self::EncodeEntryValue<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_entries(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}

impl<'a, W, const OPT: Options, C> VariantEncoder for StorageEncoder<'a, W, OPT, C>
where
    C: ?Sized + Context,
    W: Writer,
{
    type Cx = C;
    type Ok = ();
    type EncodeTag<'this>
        = StorageEncoder<'a, W::Mut<'this>, OPT, C>
    where
        Self: 'this;
    type EncodeData<'this>
        = StorageEncoder<'a, W::Mut<'this>, OPT, C>
    where
        Self: 'this;

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_data(&mut self) -> Result<Self::EncodeData<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_variant(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }
}
