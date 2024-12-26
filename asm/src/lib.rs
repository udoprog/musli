#[cfg(feature = "musli-zerocopy")]
pub mod musli_zerocopy_store {
    use musli_zerocopy::endian;
    use musli_zerocopy::OwnedBuf;
    use tests::models::*;

    macro_rules! build {
        ($id:ident, $ty:ty, $constant:ident, $number:literal) => {
            tests::if_supported! {
                musli_zerocopy, $id,
                #[inline(never)]
                pub fn $id(buf: &mut OwnedBuf<endian::Native, usize>, primitives: &$ty) {
                    buf.store(primitives);
                }
            }
        };
    }

    tests::types!(build);
}

#[cfg(feature = "musli-zerocopy")]
pub mod musli_zerocopy_store_unchecked {
    use musli_zerocopy::endian;
    use musli_zerocopy::OwnedBuf;
    use tests::models::*;

    macro_rules! build {
        ($id:ident, $ty:ty, $constant:ident, $number:literal) => {
            tests::if_supported! {
                musli_zerocopy, $id,
                #[inline(never)]
                pub fn $id(buf: &mut OwnedBuf<endian::Native, usize>, primitives: &$ty) {
                    unsafe {
                        buf.store_unchecked(primitives);
                    }
                }
            }
        };
    }

    tests::types!(build);
}

#[cfg(feature = "musli-zerocopy")]
pub mod musli_zerocopy_load {
    use musli_zerocopy::{Buf, Error, Ref};
    use tests::models::*;

    macro_rules! build {
        ($id:ident, $ty:ty, $constant:ident, $number:literal) => {
            tests::if_supported! {
                musli_zerocopy, $id,
                #[inline(never)]
                pub fn $id(
                    buf: &Buf,
                    reference: Ref<$ty>,
                ) -> Result<&$ty, Error> {
                    buf.load(reference)
                }
            }
        };
    }

    tests::types!(build);
}

#[cfg(feature = "zerocopy")]
pub mod zerocopy_load {
    use tests::models::*;
    use zerocopy::FromBytes;

    macro_rules! build {
        ($id:ident, $ty:ty, $constant:ident, $number:literal) => {
            tests::if_supported! {
                zerocopy, $id,
                #[inline(never)]
                pub fn $id(buf: &[u8]) -> Option<$ty> {
                    <$ty>::read_from_bytes(buf).ok()
                }
            }
        };
    }

    tests::types!(build);
}

#[cfg(feature = "zerocopy")]
pub mod zerocopy_store {
    use core::mem::size_of;
    use tests::models::*;
    use zerocopy::IntoBytes;

    macro_rules! build {
        ($id:ident, $ty:ty, $constant:ident, $number:literal) => {
            tests::if_supported! {
                zerocopy, $id,
                #[inline(never)]
                pub fn $id<'buf>(out: &'buf mut [u8; size_of::<$ty>()], value: &$ty) -> &'buf [u8] {
                    out.copy_from_slice(value.as_bytes());
                    out
                }
            }
        };
    }

    tests::types!(build);
}

#[cfg(feature = "musli-zerocopy")]
pub mod musli_zerocopy_swap {
    use musli_zerocopy::endian;
    use musli_zerocopy::{Ref, ZeroCopy};

    #[inline(never)]
    pub fn array_le(value: [u32; 8]) -> [u32; 8] {
        value.swap_bytes::<endian::Little>()
    }

    #[inline(never)]
    pub fn array_be(value: [u32; 8]) -> [u32; 8] {
        value.swap_bytes::<endian::Big>()
    }

    #[inline(never)]
    pub fn reference_noop(value: Ref<u32>) -> Ref<u32> {
        value.swap_bytes::<endian::Big>()
    }

    #[derive(ZeroCopy)]
    #[repr(C)]
    pub struct Struct {
        bits32: u32,
        bits64: u64,
        bits128: u128,
        array: [u8; 16],
    }

    #[inline(never)]
    pub fn ensure_struct_ne(st: Struct) -> Struct {
        st.swap_bytes::<endian::Native>()
    }

    #[inline(never)]
    pub fn ensure_struct_le(st: Struct) -> Struct {
        st.swap_bytes::<endian::Little>()
    }

    #[inline(never)]
    pub fn ensure_struct_be(st: Struct) -> Struct {
        st.swap_bytes::<endian::Big>()
    }

    #[derive(ZeroCopy)]
    #[repr(u32)]
    pub enum Enum {
        Variant1,
        Variant2 { bits32: u32, bits64: u64 },
    }

    #[inline(never)]
    pub fn ensure_enum_ne(st: Enum) -> Enum {
        st.swap_bytes::<endian::Native>()
    }

    #[inline(never)]
    pub fn ensure_enum_le(st: Enum) -> Enum {
        st.swap_bytes::<endian::Little>()
    }

    #[inline(never)]
    pub fn ensure_enum_be(st: Enum) -> Enum {
        st.swap_bytes::<endian::Big>()
    }
}

pub mod generic {
    pub enum Enum {
        Empty,
        One(u32),
        Two(u32, u64),
    }

    pub fn do_nothing(en: Enum) -> Enum {
        en
    }
}

#[cfg(feature = "musli")]
pub mod musli {
    use musli::options::{self, Float, Integer, Options};
    use musli::storage::{Encoding, Error};
    use musli::{Decode, Encode};

    use tests::models::Primitives;
    use tests::Packed;

    pub fn encode_primitives<'buf>(
        buffer: &'buf mut Vec<u8>,
        value: &Primitives,
    ) -> Result<&'buf [u8], Error> {
        encode::<Primitives>(buffer, value)
    }

    pub fn decode_primitives<'buf>(buf: &'buf [u8]) -> Result<Primitives, Error> {
        decode::<Primitives>(buf)
    }

    const OPTIONS: Options = options::new()
        .with_length(Integer::Fixed)
        .with_integer(Integer::Fixed)
        .with_float(Float::Fixed)
        .build();

    const ENCODING: Encoding<OPTIONS, Packed> = Encoding::new().with_options().with_mode();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], Error>
    where
        T: Encode<Packed>,
    {
        ENCODING.encode(&mut *buf, value)?;
        Ok(buf)
    }

    pub fn decode<'buf, T>(buf: &'buf [u8]) -> Result<T, Error>
    where
        T: Decode<'buf, Packed>,
    {
        ENCODING.from_slice(buf)
    }
}

#[cfg(feature = "speedy")]
pub mod speedy {
    use speedy::{Readable, Writable};
    use tests::models::Primitives;

    pub fn encode_primitives<'buf>(
        buffer: &'buf mut Vec<u8>,
        value: &Primitives,
    ) -> Result<&'buf [u8], speedy::Error> {
        encode::<Primitives>(buffer, value)
    }

    pub fn decode_primitives<'buf>(buf: &'buf [u8]) -> Result<Primitives, speedy::Error> {
        decode::<Primitives>(buf)
    }

    #[inline(always)]
    pub fn encode<'buf, T>(
        buffer: &'buf mut Vec<u8>,
        value: &T,
    ) -> Result<&'buf [u8], speedy::Error>
    where
        T: Writable<speedy::LittleEndian>,
    {
        let len = value.bytes_needed()?;
        // See https://github.com/koute/speedy/issues/78
        buffer.resize(len, 0);
        value.write_to_buffer(buffer.as_mut_slice())?;
        Ok(buffer.as_slice())
    }

    #[inline(always)]
    pub fn decode<'buf, T>(buf: &'buf [u8]) -> Result<T, speedy::Error>
    where
        T: Readable<'buf, speedy::LittleEndian>,
    {
        T::read_from_buffer(buf)
    }
}
