use core::fmt;
use core::mem::MaybeUninit;

#[cfg(feature = "alloc")]
use rust_alloc::vec::Vec;

use crate::de::{
    utils, DecodeSliceBuilder, Decoder, EntriesDecoder, EntryDecoder, MapDecoder, SequenceDecoder,
    SizeHint, TryFastDecode, UnsizedVisitor, VariantDecoder,
};
use crate::options::is_native_fixed;
use crate::{Context, Decode, Options, Reader};

/// Test if the current options and `$t` is suitable for bitwise slice decoding.
macro_rules! is_bitwise_slice {
    ($opt:ident, $t:ident) => {
        const {
            is_native_fixed::<$opt>()
                && <$t>::IS_BITWISE_DECODE
                && size_of::<$t>() % align_of::<$t>() == 0
        }
    };
}

/// A very simple decoder suitable for storage decoding.
pub struct StorageDecoder<const OPT: Options, const PACK: bool, R, C> {
    cx: C,
    reader: R,
}

impl<const OPT: Options, const PACK: bool, R, C> StorageDecoder<OPT, PACK, R, C> {
    /// Construct a new fixed width message decoder.
    #[inline]
    pub(crate) fn new(cx: C, reader: R) -> Self {
        Self { cx, reader }
    }
}

/// A length-prefixed decode wrapper.
///
/// This simplifies implementing decoders that do not have any special handling
/// for length-prefixed types.
#[doc(hidden)]
pub struct LimitedStorageDecoder<const OPT: Options, const PACK: bool, R, C> {
    remaining: usize,
    cx: C,
    reader: R,
}

#[crate::decoder(crate)]
impl<'de, const OPT: Options, const PACK: bool, R, C> Decoder<'de>
    for StorageDecoder<OPT, PACK, R, C>
where
    R: Reader<'de>,
    C: Context,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = C::Mode;
    type Allocator = C::Allocator;
    type WithContext<U>
        = StorageDecoder<OPT, PACK, R, U>
    where
        U: Context<Allocator = Self::Allocator>;
    type DecodePack = StorageDecoder<OPT, true, R, C>;
    type DecodeSome = Self;
    type DecodeSequence = LimitedStorageDecoder<OPT, PACK, R, C>;
    type DecodeMap = LimitedStorageDecoder<OPT, PACK, R, C>;
    type DecodeMapEntries = LimitedStorageDecoder<OPT, PACK, R, C>;
    type DecodeVariant = Self;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn with_context<U>(self, cx: U) -> Result<Self::WithContext<U>, C::Error>
    where
        U: Context<Allocator = Self::Allocator>,
    {
        Ok(StorageDecoder::new(cx, self.reader))
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type supported by the storage decoder")
    }

    #[inline]
    fn try_fast_decode<T>(mut self) -> Result<TryFastDecode<T, Self>, Self::Error>
    where
        T: Decode<'de, Self::Mode, Self::Allocator>,
    {
        if !const { is_native_fixed::<OPT>() && T::IS_BITWISE_DECODE } {
            return Ok(TryFastDecode::Unsupported(self));
        }

        let mut value = MaybeUninit::<T>::uninit();

        // SAFETY: We've ensured the type is layout compatible with the current
        // serialization just above.
        unsafe {
            let ptr = value.as_mut_ptr().cast::<u8>();
            let n = size_of::<T>();
            self.reader.read_bytes_uninit(self.cx, ptr, n)?;
            Ok(TryFastDecode::Ok(value.assume_init()))
        }
    }

    #[inline]
    fn decode_empty(mut self) -> Result<(), C::Error> {
        let mark = self.cx.mark();
        let count = crate::int::decode_usize::<_, _, OPT>(self.cx, self.reader.borrow_mut())?;

        if count != 0 {
            return Err(self
                .cx
                .marked_message(&mark, ExpectedEmptySequence { actual: count }));
        }

        Ok(())
    }

    #[inline]
    fn decode_pack<F, O>(self, f: F) -> Result<O, C::Error>
    where
        F: FnOnce(&mut Self::DecodePack) -> Result<O, C::Error>,
    {
        let mut this = StorageDecoder::new(self.cx, self.reader);
        f(&mut this)
    }

    #[inline]
    fn decode_array<const N: usize>(mut self) -> Result<[u8; N], C::Error> {
        self.reader.read_array(self.cx)
    }

    #[inline]
    fn decode_bytes<V>(mut self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: UnsizedVisitor<'de, C, [u8]>,
    {
        let len = crate::int::decode_usize::<_, _, OPT>(self.cx, self.reader.borrow_mut())?;
        self.reader.read_bytes(self.cx, len, visitor)
    }

    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, C::Error>
    where
        V: UnsizedVisitor<'de, C, str>,
    {
        struct Visitor<V>(V);

        impl<'de, C, V> UnsizedVisitor<'de, C, [u8]> for Visitor<V>
        where
            C: Context,
            V: UnsizedVisitor<'de, C, str>,
        {
            type Ok = V::Ok;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.expecting(f)
            }

            #[cfg(feature = "alloc")]
            #[inline]
            fn visit_owned(self, cx: C, bytes: Vec<u8>) -> Result<Self::Ok, C::Error> {
                let string = crate::str::from_utf8_owned(bytes).map_err(cx.map())?;
                self.0.visit_owned(cx, string)
            }

            #[inline]
            fn visit_borrowed(self, cx: C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                let string = crate::str::from_utf8(bytes).map_err(cx.map())?;
                self.0.visit_borrowed(cx, string)
            }

            #[inline]
            fn visit_ref(self, cx: C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                let string = crate::str::from_utf8(bytes).map_err(cx.map())?;
                self.0.visit_ref(cx, string)
            }
        }

        self.decode_bytes(Visitor(visitor))
    }

    #[inline]
    fn decode_bool(mut self) -> Result<bool, C::Error> {
        let mark = self.cx.mark();
        let byte = self.reader.read_byte(self.cx)?;

        match byte {
            0 => Ok(false),
            1 => Ok(true),
            b => Err(self.cx.marked_message(&mark, BadBoolean { actual: b })),
        }
    }

    #[inline]
    fn decode_char(self) -> Result<char, C::Error> {
        let cx = self.cx;
        let mark = self.cx.mark();
        let num = self.decode_u32()?;

        match char::from_u32(num) {
            Some(d) => Ok(d),
            None => Err(cx.marked_message(&mark, BadCharacter { actual: num })),
        }
    }

    #[inline]
    fn decode_u8(mut self) -> Result<u8, C::Error> {
        self.reader.read_byte(self.cx)
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, C::Error> {
        crate::int::decode_unsigned::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, C::Error> {
        crate::int::decode_unsigned::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, C::Error> {
        crate::int::decode_unsigned::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, C::Error> {
        crate::int::decode_unsigned::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, C::Error> {
        Ok(self.decode_u8()? as i8)
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, C::Error> {
        crate::int::decode_signed::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, C::Error> {
        crate::int::decode_signed::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, C::Error> {
        crate::int::decode_signed::<_, _, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, C::Error> {
        crate::int::decode_signed::<_, _, _, OPT>(self.cx, self.reader)
    }

    /// Decode a 32-bit floating point value by reading the 32-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f32(self) -> Result<f32, C::Error> {
        Ok(f32::from_bits(self.decode_u32()?))
    }

    /// Decode a 64-bit floating point value by reading the 64-bit in-memory
    /// IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f64(self) -> Result<f64, C::Error> {
        let bits = self.decode_u64()?;
        Ok(f64::from_bits(bits))
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, C::Error> {
        crate::int::decode_usize::<_, _, OPT>(self.cx, self.reader)
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, C::Error> {
        Ok(self.decode_usize()? as isize)
    }

    #[inline]
    fn decode_option(mut self) -> Result<Option<Self::DecodeSome>, C::Error> {
        if PACK {
            if self.reader.is_eof() {
                Ok(None)
            } else {
                Ok(Some(self))
            }
        } else {
            let b = self.reader.read_byte(self.cx)?;
            Ok(if b == 1 { Some(self) } else { None })
        }
    }

    /// Decode a sequence of values.
    #[inline]
    fn decode_slice<V, T>(mut self) -> Result<V, <Self::Cx as Context>::Error>
    where
        V: DecodeSliceBuilder<T>,
        T: Decode<'de, Self::Mode, Self::Allocator>,
    {
        // Check that the type is packed inside of the slice.
        if !is_bitwise_slice!(OPT, T) {
            return utils::default_decode_slice(self);
        }

        let len = self.reader.borrow_mut().read_array(self.cx)?;
        let len = usize::from_ne_bytes(len);

        let mut out = V::new(self.cx)?;

        if size_of::<T>() > 0 {
            // Calculate a max chunk size which takes the size of the current
            // element into account.
            //
            // We obey by a max chunk size since we desperately want to avoid
            // overallocating in case the data is garbage. The only reason data
            // could be garbage in this instance is if we don't have enough to
            // fill the buffer. But if we allocate too much space, we run the
            // risk of blowing up the allocator instead which at best causes an
            // error or more likely panics or aborts.
            let max_chunk: usize = const {
                // 64k is a reasonably good default max chunk size.
                let base = 1usize << 16;

                let size = match size_of::<T>() {
                    0 => 1,
                    size if size > base => base,
                    size => size,
                };

                base.div_ceil(size)
            };

            let mut at = 0;

            while at < len {
                self.cx.enter_sequence_index(at);

                // The size of the chunk to write.
                let chunk = (len - at).min(max_chunk);

                out.reserve(self.cx, chunk)?;

                let ptr = out.as_mut_ptr().wrapping_add(at).cast::<u8>();
                let n = chunk * size_of::<T>();

                // Read into allocated space and mark as initialized.
                unsafe {
                    self.reader.read_bytes_uninit(self.cx, ptr, n)?;
                    at += max_chunk;
                }

                self.cx.leave_sequence_index();
            }
        }

        // SAFETY: If the type is zero-sized, we don't need to copy anything and
        // can just set the length, otherwise setting the length here has no
        // drop implications since bitwise encoding in particular essentially
        // requires that the type is `Copy`.
        unsafe {
            out.set_len(len);
        }

        Ok(out)
    }

    #[inline]
    fn decode_sequence<F, O>(self, f: F) -> Result<O, C::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, C::Error>,
    {
        let cx = self.cx;
        let mut decoder = LimitedStorageDecoder::new(self.cx, self.reader)?;
        let output = f(&mut decoder)?;

        if decoder.remaining != 0 {
            return Err(cx.message("Caller did not decode all available map entries"));
        }

        Ok(output)
    }

    #[inline]
    fn decode_map<F, O>(self, f: F) -> Result<O, C::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, C::Error>,
    {
        let cx = self.cx;
        let mut decoder = LimitedStorageDecoder::new(self.cx, self.reader)?;
        let output = f(&mut decoder)?;

        if decoder.remaining != 0 {
            return Err(cx.message("Caller did not decode all available map entries"));
        }

        Ok(output)
    }

    #[inline]
    fn decode_map_entries<F, O>(self, f: F) -> Result<O, C::Error>
    where
        F: FnOnce(&mut Self::DecodeMapEntries) -> Result<O, C::Error>,
    {
        self.decode_map(f)
    }

    #[inline]
    fn decode_variant<F, O>(mut self, f: F) -> Result<O, C::Error>
    where
        F: FnOnce(&mut Self::DecodeVariant) -> Result<O, C::Error>,
    {
        f(&mut self)
    }
}

impl<'de, const OPT: Options, const PACK: bool, R, C> SequenceDecoder<'de>
    for StorageDecoder<OPT, PACK, R, C>
where
    R: Reader<'de>,
    C: Context,
{
    type Cx = C;
    type DecodeNext<'this>
        = StorageDecoder<OPT, PACK, R::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn try_decode_next(
        &mut self,
    ) -> Result<Option<Self::DecodeNext<'_>>, <Self::Cx as Context>::Error> {
        Ok(Some(self.decode_next()?))
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, C::Error> {
        Ok(StorageDecoder::new(self.cx, self.reader.borrow_mut()))
    }
}

impl<'de, const OPT: Options, const PACK: bool, R, C> LimitedStorageDecoder<OPT, PACK, R, C>
where
    C: Context,
    R: Reader<'de>,
{
    #[inline]
    fn new(cx: C, mut reader: R) -> Result<Self, C::Error> {
        let remaining = crate::int::decode_usize::<_, _, OPT>(cx, reader.borrow_mut())?;

        Ok(Self {
            cx,
            reader,
            remaining,
        })
    }

    #[inline]
    fn with_remaining(cx: C, reader: R, remaining: usize) -> Self {
        Self {
            cx,
            reader,
            remaining,
        }
    }
}

impl<'de, const OPT: Options, const PACK: bool, R, C> SequenceDecoder<'de>
    for LimitedStorageDecoder<OPT, PACK, R, C>
where
    R: Reader<'de>,
    C: Context,
{
    type Cx = C;
    type DecodeNext<'this>
        = StorageDecoder<OPT, PACK, R::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::exact(self.remaining)
    }

    #[inline]
    fn try_decode_next(&mut self) -> Result<Option<Self::DecodeNext<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(self.cx, self.reader.borrow_mut())))
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Self::DecodeNext<'_>, <Self::Cx as Context>::Error> {
        let cx = self.cx;

        let Some(decoder) = self.try_decode_next()? else {
            return Err(cx.message("No remaining elements"));
        };

        Ok(decoder)
    }
}

impl<'de, const OPT: Options, const PACK: bool, R, C> MapDecoder<'de>
    for LimitedStorageDecoder<OPT, PACK, R, C>
where
    R: Reader<'de>,
    C: Context,
{
    type Cx = C;
    type DecodeEntry<'this>
        = StorageDecoder<OPT, PACK, R::Mut<'this>, C>
    where
        Self: 'this;
    type DecodeRemainingEntries<'this>
        = LimitedStorageDecoder<OPT, PACK, R::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::exact(self.remaining)
    }

    #[inline]
    fn decode_entry(&mut self) -> Result<Option<Self::DecodeEntry<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(self.cx, self.reader.borrow_mut())))
    }

    #[inline]
    fn decode_remaining_entries(
        &mut self,
    ) -> Result<Self::DecodeRemainingEntries<'_>, <Self::Cx as Context>::Error> {
        Ok(LimitedStorageDecoder::with_remaining(
            self.cx,
            self.reader.borrow_mut(),
            self.remaining,
        ))
    }
}

impl<'de, const OPT: Options, const PACK: bool, R, C> EntryDecoder<'de>
    for StorageDecoder<OPT, PACK, R, C>
where
    R: Reader<'de>,
    C: Context,
{
    type Cx = C;
    type DecodeKey<'this>
        = StorageDecoder<OPT, PACK, R::Mut<'this>, C>
    where
        Self: 'this;
    type DecodeValue = Self;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_key(&mut self) -> Result<Self::DecodeKey<'_>, C::Error> {
        Ok(StorageDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn decode_value(self) -> Result<Self::DecodeValue, C::Error> {
        Ok(self)
    }
}

impl<'de, const OPT: Options, const PACK: bool, R, C> EntriesDecoder<'de>
    for LimitedStorageDecoder<OPT, PACK, R, C>
where
    R: Reader<'de>,
    C: Context,
{
    type Cx = C;
    type DecodeEntryKey<'this>
        = StorageDecoder<OPT, PACK, R::Mut<'this>, C>
    where
        Self: 'this;
    type DecodeEntryValue<'this>
        = StorageDecoder<OPT, PACK, R::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_entry_key(&mut self) -> Result<Option<Self::DecodeEntryKey<'_>>, C::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        Ok(Some(StorageDecoder::new(self.cx, self.reader.borrow_mut())))
    }

    #[inline]
    fn decode_entry_value(&mut self) -> Result<Self::DecodeEntryValue<'_>, C::Error> {
        Ok(StorageDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn end_entries(self) -> Result<(), C::Error> {
        if self.remaining != 0 {
            return Err(self
                .cx
                .message("Caller did not decode all available map entries"));
        }

        Ok(())
    }
}

impl<'de, const OPT: Options, const PACK: bool, R, C> VariantDecoder<'de>
    for StorageDecoder<OPT, PACK, R, C>
where
    R: Reader<'de>,
    C: Context,
{
    type Cx = C;
    type DecodeTag<'this>
        = StorageDecoder<OPT, PACK, R::Mut<'this>, C>
    where
        Self: 'this;
    type DecodeValue<'this>
        = StorageDecoder<OPT, PACK, R::Mut<'this>, C>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn decode_tag(&mut self) -> Result<Self::DecodeTag<'_>, C::Error> {
        Ok(StorageDecoder::new(self.cx, self.reader.borrow_mut()))
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::DecodeValue<'_>, C::Error> {
        Ok(StorageDecoder::new(self.cx, self.reader.borrow_mut()))
    }
}

struct ExpectedEmptySequence {
    actual: usize,
}

impl fmt::Display for ExpectedEmptySequence {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual } = *self;
        write!(f, "Expected empty sequence, but was {actual}",)
    }
}

struct BadBoolean {
    actual: u8,
}

impl fmt::Display for BadBoolean {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual } = *self;
        write!(f, "Bad boolean byte 0x{actual:02x}")
    }
}

struct BadCharacter {
    actual: u32,
}

impl fmt::Display for BadCharacter {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { actual } = *self;
        write!(f, "Bad character number {actual}")
    }
}
