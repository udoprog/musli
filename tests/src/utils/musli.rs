#[cfg(feature = "musli-json")]
#[crate::benchmarker]
pub mod musli_json {
    use alloc::vec::Vec;

    use musli::json::Encoding;
    use musli::json::Error;
    use musli::mode::Text;
    use musli::{Decode, Encode};

    const ENCODING: Encoding = Encoding::new();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buffer: &mut Vec<u8>) {
        buffer.clear();
    }

    pub fn encode<'buf, T>(buffer: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], Error>
    where
        T: Encode<Text>,
    {
        ENCODING.encode(&mut *buffer, value)?;
        Ok(buffer)
    }

    pub fn decode<'buf, T>(buffer: &'buf [u8]) -> Result<T, Error>
    where
        T: Decode<'buf, Text>,
    {
        ENCODING.from_slice(buffer)
    }
}

#[cfg(feature = "musli-storage")]
#[crate::benchmarker]
pub mod musli_storage_packed {
    use alloc::vec::Vec;

    use musli::options::{self, Float, Integer, Options};
    use musli::storage::{Encoding, Error};
    use musli::{Decode, Encode};

    use crate::mode::Packed;

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

#[cfg(feature = "musli-storage")]
#[crate::benchmarker]
pub mod musli_storage {
    use alloc::vec::Vec;

    use musli::mode::Binary;
    use musli::options::{self, Options};
    use musli::storage::{Encoding, Error};
    use musli::{Decode, Encode};

    const OPTIONS: Options = options::new().build();
    const ENCODING: Encoding<OPTIONS> = Encoding::new().with_options();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], Error>
    where
        T: Encode<Binary>,
    {
        ENCODING.encode(&mut *buf, value)?;
        Ok(buf)
    }

    pub fn decode<'buf, T>(buf: &'buf [u8]) -> Result<T, Error>
    where
        T: Decode<'buf, Binary>,
    {
        ENCODING.from_slice(buf)
    }
}

#[cfg(feature = "musli-wire")]
#[crate::benchmarker]
pub mod musli_wire {
    use alloc::vec::Vec;

    use musli::mode::Binary;
    use musli::wire::Encoding;
    use musli::wire::Error;
    use musli::{Decode, Encode};

    const ENCODING: Encoding = Encoding::new();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], Error>
    where
        T: Encode<Binary>,
    {
        ENCODING.encode(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    pub fn decode<'buf, T>(buf: &'buf [u8]) -> Result<T, Error>
    where
        T: Decode<'buf, Binary>,
    {
        ENCODING.from_slice(buf)
    }
}

#[cfg(feature = "musli-descriptive")]
#[crate::benchmarker]
pub mod musli_descriptive {
    use alloc::vec::Vec;

    use musli::descriptive::Encoding;
    use musli::descriptive::Error;
    use musli::mode::Binary;
    use musli::{Decode, Encode};

    const ENCODING: Encoding = Encoding::new();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], Error>
    where
        T: Encode<Binary>,
    {
        ENCODING.encode(&mut *buf, value)?;
        Ok(buf)
    }

    pub fn decode<'buf, T>(buf: &'buf [u8]) -> Result<T, Error>
    where
        T: Decode<'buf, Binary>,
    {
        ENCODING.from_slice(buf)
    }
}

#[cfg(feature = "musli-value")]
#[crate::benchmarker(as_bytes_disabled)]
pub mod musli_value {
    use musli::mode::Binary;
    use musli::value::Value;
    use musli::{Decode, Encode};

    pub fn encode<T>(value: &T) -> Result<Value, musli::value::Error>
    where
        T: Encode<Binary>,
    {
        musli::value::encode(value)
    }

    pub fn decode<T>(buf: &Value) -> Result<T, musli::value::Error>
    where
        for<'a> T: Decode<'a, Binary>,
    {
        musli::value::decode(buf)
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

    pub fn decode<T>(buf: &[u8]) -> Result<&T, Error>
    where
        T: ZeroCopy,
    {
        Buf::new(buf).load(Ref::<T, endian::Native, usize>::zero())
    }
}
