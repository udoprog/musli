use core::fmt;
use core::marker::PhantomData;
use core::mem::size_of_val;
use core::slice;

use crate::en::{
    utils, Encode, Encoder, EntriesEncoder, EntryEncoder, MapEncoder, SequenceEncoder,
    TryFastEncode, VariantEncoder,
};
use crate::hint::{MapHint, SequenceHint};
use crate::options::is_native_fixed;
use crate::{Context, Options, Writer};

/// Test if the current options and `$t` is suitable for bitwise slice encoding.
macro_rules! is_bitwise_slice {
    ($opt:ident, $t:ident) => {
        const {
            is_native_fixed::<$opt>()
                && <$t>::IS_BITWISE_ENCODE
                && size_of::<$t>() % align_of::<$t>() == 0
        }
    };
}

/// A very simple encoder suitable for storage encoding.
pub struct StorageEncoder<const OPT: Options, const PACK: bool, W, C, M>
where
    M: 'static,
{
    cx: C,
    writer: W,
    _marker: PhantomData<M>,
}

impl<const OPT: Options, const PACK: bool, W, C, M> StorageEncoder<OPT, PACK, W, C, M>
where
    M: 'static,
{
    /// Construct a new storage encoder.
    #[inline]
    pub(crate) fn new(cx: C, writer: W) -> Self {
        Self {
            cx,
            writer,
            _marker: PhantomData,
        }
    }
}

#[crate::encoder(crate)]
impl<const OPT: Options, const PACK: bool, W, C, M> Encoder for StorageEncoder<OPT, PACK, W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Ok = ();
    type Mode = M;
    type EncodePack = StorageEncoder<OPT, true, W, C, M>;
    type EncodeSome = Self;
    type EncodeSequence = Self;
    type EncodeMap = Self;
    type EncodeMapEntries = Self;
    type EncodeVariant = Self;
    type EncodeSequenceVariant = Self;
    type EncodeMapVariant = Self;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
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
    fn encode_empty(self) -> Result<Self::Ok, Self::Error> {
        static HINT: SequenceHint = SequenceHint::with_size(0);
        self.encode_sequence_fn(&HINT, |_| Ok(()))
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::EncodePack, Self::Error> {
        Ok(StorageEncoder::<OPT, true, _, _, M>::new(
            self.cx,
            self.writer,
        ))
    }

    #[inline]
    fn encode_array<const N: usize>(mut self, array: &[u8; N]) -> Result<Self::Ok, Self::Error> {
        self.writer.write_bytes(self.cx, array)
    }

    #[inline]
    fn encode_bytes(mut self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
        crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), bytes.len())?;
        self.writer.write_bytes(self.cx, bytes)?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored<I>(mut self, len: usize, vectors: I) -> Result<Self::Ok, Self::Error>
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
    fn encode_string(mut self, string: &str) -> Result<Self::Ok, Self::Error> {
        crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), string.len())?;
        self.writer.write_bytes(self.cx, string.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn encode_bool(mut self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(self.cx, if value { 1 } else { 0 })
    }

    #[inline]
    fn encode_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.encode_u32(value as u32)
    }

    #[inline]
    fn encode_u8(mut self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.writer.write_byte(self.cx, value)
    }

    #[inline]
    fn encode_u16(mut self, value: u16) -> Result<Self::Ok, Self::Error> {
        crate::int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u32(mut self, value: u32) -> Result<Self::Ok, Self::Error> {
        crate::int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u64(mut self, value: u64) -> Result<Self::Ok, Self::Error> {
        crate::int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_u128(mut self, value: u128) -> Result<Self::Ok, Self::Error> {
        crate::int::encode_unsigned::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.encode_u8(value as u8)
    }

    #[inline]
    fn encode_i16(mut self, value: i16) -> Result<Self::Ok, Self::Error> {
        crate::int::encode_signed::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i32(mut self, value: i32) -> Result<Self::Ok, Self::Error> {
        crate::int::encode_signed::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i64(mut self, value: i64) -> Result<Self::Ok, Self::Error> {
        crate::int::encode_signed::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_i128(mut self, value: i128) -> Result<Self::Ok, Self::Error> {
        crate::int::encode_signed::<_, _, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        self.encode_u32(value.to_bits())
    }

    #[inline]
    fn encode_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        self.encode_u64(value.to_bits())
    }

    #[inline]
    fn encode_usize(mut self, value: usize) -> Result<Self::Ok, Self::Error> {
        crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), value)
    }

    #[inline]
    fn encode_isize(self, value: isize) -> Result<Self::Ok, Self::Error> {
        self.encode_usize(value as usize)
    }

    #[inline]
    fn encode_some(mut self) -> Result<Self::EncodeSome, Self::Error> {
        if !PACK {
            self.writer.write_byte(self.cx, 1)?;
        }

        Ok(self)
    }

    #[inline]
    fn encode_none(mut self) -> Result<Self::Ok, Self::Error> {
        if !PACK {
            self.writer.write_byte(self.cx, 0)?;
        }

        Ok(())
    }

    #[inline]
    fn encode_slice<T>(mut self, slice: impl AsRef<[T]>) -> Result<Self::Ok, Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        // Check that the type is packed inside of the slice.
        if !is_bitwise_slice!(OPT, T) {
            return utils::default_encode_slice(self, slice);
        }

        // SAFETY: We've ensured the type is layout compatible with the current
        // serialization just above.
        unsafe { encode_packed_len_slice(self.writer.borrow_mut(), self.cx, slice) }
    }

    #[inline]
    fn encode_slices<T>(
        mut self,
        len: usize,
        slices: impl IntoIterator<Item: AsRef<[T]>>,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        // Check that the type is packed inside of the slice.
        if !is_bitwise_slice!(OPT, T) {
            return utils::default_encode_slices(self, len, slices);
        }

        // SAFETY: We've ensured the type is layout compatible with the current
        // serialization just above.
        unsafe { encode_packed_len_slices(self.writer.borrow_mut(), self.cx, len, slices) }
    }

    #[inline]
    fn encode_sequence(mut self, hint: &SequenceHint) -> Result<Self::EncodeSequence, Self::Error> {
        crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), hint.size)?;
        Ok(self)
    }

    #[inline]
    fn encode_map(mut self, hint: &MapHint) -> Result<Self::EncodeMap, Self::Error> {
        crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), hint.size)?;
        Ok(self)
    }

    #[inline]
    fn encode_map_entries(mut self, hint: &MapHint) -> Result<Self::EncodeMapEntries, Self::Error> {
        crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), hint.size)?;
        Ok(self)
    }

    #[inline]
    fn encode_variant(self) -> Result<Self::EncodeVariant, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_sequence_variant<T>(
        mut self,
        tag: &T,
        hint: &SequenceHint,
    ) -> Result<Self::EncodeSequenceVariant, Self::Error>
    where
        T: ?Sized + Encode<Self::Mode>,
    {
        StorageEncoder::<OPT, PACK, _, _, M>::new(self.cx, self.writer.borrow_mut()).encode(tag)?;
        crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), hint.size)?;
        Ok(self)
    }

    #[inline]
    fn encode_map_variant<T>(
        mut self,
        tag: &T,
        hint: &MapHint,
    ) -> Result<Self::EncodeMapVariant, Self::Error>
    where
        T: ?Sized + Encode<Self::Mode>,
    {
        StorageEncoder::<OPT, PACK, _, _, M>::new(self.cx, self.writer.borrow_mut()).encode(tag)?;
        crate::int::encode_usize::<_, _, OPT>(self.cx, self.writer.borrow_mut(), hint.size)?;
        Ok(self)
    }
}

impl<const OPT: Options, const PACK: bool, W, C, M> SequenceEncoder
    for StorageEncoder<OPT, PACK, W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Ok = ();
    type Error = C::Error;
    type Mode = M;
    type EncodeNext<'this>
        = StorageEncoder<OPT, PACK, W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_slice<T>(&mut self, slice: impl AsRef<[T]>) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        // Check that the type is packed inside of the slice.
        if !is_bitwise_slice!(OPT, T) {
            return utils::default_sequence_encode_slice(self, slice);
        }

        // SAFETY: We've ensured the type is layout compatible with the current
        // serialization just above.
        unsafe { encode_packed_slice(self.writer.borrow_mut(), self.cx, slice) }
    }

    #[inline]
    fn encode_slices<T>(
        &mut self,
        slices: impl IntoIterator<Item: AsRef<[T]>>,
    ) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        // Check that the type is packed inside of the slice.
        if !is_bitwise_slice!(OPT, T) {
            return utils::default_sequence_encode_slices(self, slices);
        }

        // SAFETY: We've ensured the type is layout compatible with the current
        // serialization just above.
        unsafe { encode_packed_slices(self.writer.borrow_mut(), self.cx, slices) }
    }

    #[inline]
    fn finish_sequence(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<const OPT: Options, const PACK: bool, W, C, M> MapEncoder
    for StorageEncoder<OPT, PACK, W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Ok = ();
    type Error = C::Error;
    type Mode = M;
    type EncodeEntry<'this>
        = StorageEncoder<OPT, PACK, W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_map(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<const OPT: Options, const PACK: bool, W, C, M> EntryEncoder
    for StorageEncoder<OPT, PACK, W, C, M>
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
        = StorageEncoder<OPT, PACK, W::Mut<'this>, C, M>
    where
        Self: 'this;
    type EncodeValue<'this>
        = StorageEncoder<OPT, PACK, W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_key(&mut self) -> Result<Self::EncodeKey<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_entry(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<const OPT: Options, const PACK: bool, W, C, M> EntriesEncoder
    for StorageEncoder<OPT, PACK, W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Ok = ();
    type Error = C::Error;
    type Mode = M;
    type EncodeEntryKey<'this>
        = StorageEncoder<OPT, PACK, W::Mut<'this>, C, M>
    where
        Self: 'this;
    type EncodeEntryValue<'this>
        = StorageEncoder<OPT, PACK, W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry_key(&mut self) -> Result<Self::EncodeEntryKey<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_entry_value(&mut self) -> Result<Self::EncodeEntryValue<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_entries(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<const OPT: Options, const PACK: bool, W, C, M> VariantEncoder
    for StorageEncoder<OPT, PACK, W, C, M>
where
    W: Writer,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Ok = ();
    type Error = C::Error;
    type Mode = M;
    type EncodeTag<'this>
        = StorageEncoder<OPT, PACK, W::Mut<'this>, C, M>
    where
        Self: 'this;
    type EncodeData<'this>
        = StorageEncoder<OPT, PACK, W::Mut<'this>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn encode_data(&mut self) -> Result<Self::EncodeData<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.cx, self.writer.borrow_mut()))
    }

    #[inline]
    fn finish_variant(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// Encode a packed length-prefixed slice.
///
/// # Safety
///
/// The caller must ensure that the format in use is compatible with the native
/// format of the slice.
#[inline]
unsafe fn encode_packed_len_slice<W, C, T, M>(
    mut writer: W,
    cx: C,
    slice: impl AsRef<[T]>,
) -> Result<(), C::Error>
where
    W: Writer,
    C: Context,
    T: Encode<M>,
{
    let slice = slice.as_ref();
    let len = slice.len().to_ne_bytes();
    writer.write_bytes(cx, &len)?;

    if size_of::<T>() > 0 {
        let slice = {
            let at = slice.as_ptr().cast::<u8>();
            let size = size_of_val(slice);
            slice::from_raw_parts(at, size)
        };

        writer.write_bytes(cx, slice)?;
    }

    Ok(())
}

/// Encode a packed length-prefixed slice from an iterator of slices.
///
/// # Safety
///
/// The caller must ensure that the format in use is compatible with the native
/// format of the slice.
#[inline]
unsafe fn encode_packed_len_slices<W, C, I, T, M>(
    mut writer: W,
    cx: C,
    len: usize,
    slices: I,
) -> Result<(), C::Error>
where
    W: Writer,
    C: Context,
    I: IntoIterator<Item: AsRef<[T]>>,
    T: Encode<M>,
{
    let len = len.to_ne_bytes();
    writer.write_bytes(cx, &len)?;

    if size_of::<T>() > 0 {
        for slice in slices {
            let slice = slice.as_ref();

            let slice = {
                let at = slice.as_ptr().cast::<u8>();
                let size = size_of_val(slice);
                slice::from_raw_parts(at, size)
            };

            writer.write_bytes(cx, slice)?;
        }
    }

    Ok(())
}

/// Encode a packed slice.
///
/// # Safety
///
/// The caller must ensure that the format in use is compatible with the native
/// format of the slice.
#[inline]
unsafe fn encode_packed_slice<W, C, T, M>(
    mut writer: W,
    cx: C,
    slice: impl AsRef<[T]>,
) -> Result<(), C::Error>
where
    W: Writer,
    C: Context,
    T: Encode<M>,
{
    if size_of::<T>() > 0 {
        let slice = slice.as_ref();

        let slice = {
            let at = slice.as_ptr().cast::<u8>();
            let size = size_of_val(slice);
            slice::from_raw_parts(at, size)
        };

        writer.write_bytes(cx, slice)?;
    }

    Ok(())
}

/// Encode a packed slice from an iterator of slices.
///
/// # Safety
///
/// The caller must ensure that the format in use is compatible with the native
/// format of the slice.
#[inline]
unsafe fn encode_packed_slices<W, C, I, T, M>(
    mut writer: W,
    cx: C,
    slices: I,
) -> Result<(), C::Error>
where
    W: Writer,
    C: Context,
    I: IntoIterator<Item: AsRef<[T]>>,
    T: Encode<M>,
{
    if size_of::<T>() > 0 {
        for slice in slices {
            let slice = slice.as_ref();

            let slice = {
                let at = slice.as_ptr().cast::<u8>();
                let size = size_of_val(slice);
                slice::from_raw_parts(at, size)
            };

            writer.write_bytes(cx, slice)?;
        }
    }

    Ok(())
}
