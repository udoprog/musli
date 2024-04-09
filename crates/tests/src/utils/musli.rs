#[cfg(feature = "musli-json")]
#[crate::benchmarker]
pub mod musli_json {
    use alloc::vec::Vec;

    use musli::{Decode, Encode};
    use musli_json::Encoding;
    use musli_json::Error;

    const ENCODING: Encoding = Encoding::new();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buffer: &mut Vec<u8>) {
        buffer.clear();
    }

    pub fn encode<'buf, T>(buffer: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], Error>
    where
        T: Encode,
    {
        ENCODING.encode(&mut *buffer, value)?;
        Ok(buffer)
    }

    pub fn decode<'buf, T>(buffer: &'buf [u8]) -> Result<T, Error>
    where
        T: Decode<'buf>,
    {
        ENCODING.from_slice(buffer)
    }
}

#[cfg(feature = "musli-storage")]
#[crate::benchmarker]
pub mod musli_storage_packed {
    use alloc::vec::Vec;

    use musli::{Decode, Encode};
    use musli_storage::{Encoding, Error};
    use musli_utils::options::{self, Integer, Options};

    use crate::mode::Packed;

    const OPTIONS: Options = options::new().with_integer(Integer::Fixed).build();
    const ENCODING: Encoding<Packed, OPTIONS> = Encoding::new().with_options().with_mode();

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

#[cfg(feature = "musli-storage")]
#[crate::benchmarker]
pub mod musli_storage {
    use alloc::vec::Vec;

    use musli::mode::DefaultMode;
    use musli::{Decode, Encode};
    use musli_storage::{Encoding, Error};
    use musli_utils::options::{self, Integer, Options};

    const OPTIONS: Options = options::new().with_integer(Integer::Fixed).build();
    const ENCODING: Encoding<DefaultMode, OPTIONS> = Encoding::new().with_options();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], Error>
    where
        T: Encode,
    {
        ENCODING.encode(&mut *buf, value)?;
        Ok(buf)
    }

    pub fn decode<'buf, T>(buf: &'buf [u8]) -> Result<T, Error>
    where
        T: Decode<'buf>,
    {
        ENCODING.from_slice(buf)
    }
}

#[cfg(feature = "musli-wire")]
#[crate::benchmarker]
pub mod musli_wire {
    use alloc::vec::Vec;

    use musli::mode::DefaultMode;
    use musli::{Decode, Encode};
    use musli_wire::Encoding;
    use musli_wire::Error;

    const ENCODING: Encoding<DefaultMode> = Encoding::new();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], Error>
    where
        T: Encode,
    {
        ENCODING.encode(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    pub fn decode<'buf, T>(buf: &'buf [u8]) -> Result<T, Error>
    where
        T: Decode<'buf>,
    {
        ENCODING.from_slice(buf)
    }
}

#[cfg(feature = "musli-descriptive")]
#[crate::benchmarker]
pub mod musli_descriptive {
    use crate::no_alloc::Bytes;

    use musli::mode::DefaultMode;
    use musli::{Decode, Encode};
    use musli_descriptive::Encoding;
    use musli_descriptive::Error;

    const ENCODING: Encoding<DefaultMode> = Encoding::new();

    pub fn buffer() -> Bytes {
        Bytes::with_capacity(4096)
    }

    pub fn reset(buf: &mut Bytes) {
        buf.clear();
    }

    pub fn encode<'buf, T>(buf: &'buf mut Bytes, value: &T) -> Result<&'buf [u8], Error>
    where
        T: Encode,
    {
        ENCODING.encode(&mut *buf, value)?;
        Ok(buf)
    }

    pub fn decode<'buf, T>(buf: &'buf [u8]) -> Result<T, Error>
    where
        T: Decode<'buf>,
    {
        ENCODING.from_slice(buf)
    }
}

#[cfg(feature = "musli-value")]
#[crate::benchmarker(as_bytes_disabled)]
pub mod musli_value {
    use musli::{Decode, Encode};
    use musli_value::Value;

    pub fn encode<T>(value: &T) -> Result<Value, musli_value::Error>
    where
        T: Encode,
    {
        musli_value::encode(value)
    }

    pub fn decode<T>(buf: &Value) -> Result<T, musli_value::Error>
    where
        for<'a> T: Decode<'a>,
    {
        musli_value::decode(buf)
    }
}

#[cfg(feature = "musli-zerocopy")]
#[crate::benchmarker]
pub mod musli_zerocopy {
    use musli_zerocopy::endian;
    use musli_zerocopy::{Buf, Error, OwnedBuf, Ref, ZeroCopy};

    fn buffer() -> OwnedBuf<endian::Native, usize> {
        OwnedBuf::with_capacity(4096).with_size()
    }

    #[inline(always)]
    pub fn reset<T>(buf: &mut OwnedBuf<endian::Native, usize>, #[value] _: &T)
    where
        T: ZeroCopy,
    {
        buf.clear();
        buf.request_align::<T>();
        buf.align_in_place();
    }

    pub fn encode<'buf, T>(
        buf: &'buf mut OwnedBuf<endian::Native, usize>,
        value: &T,
    ) -> Result<&'buf [u8], Error>
    where
        T: ZeroCopy,
    {
        // SAFETY: We know we've allocated space for `T` in the `reset`
        // call, so this is safe.
        unsafe { buf.store_unchecked(value) };
        Ok(&buf[..])
    }

    pub fn decode<'buf, T>(buf: &'buf [u8]) -> Result<&'buf T, Error>
    where
        T: ZeroCopy,
    {
        Buf::new(buf).load(Ref::<T, endian::Native, usize>::zero())
    }
}
