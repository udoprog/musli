mod full;
pub use self::full::*;

mod extra;
pub use self::extra::*;

#[cfg(feature = "musli-json")]
pub mod musli_json {
    use alloc::vec::Vec;

    use ::musli_json::Encoding;
    use ::musli_json::Error;
    use musli::{Decode, Encode};

    const ENCODING: Encoding = Encoding::new();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset<T>(buf: &mut Vec<u8>, _: usize, _: &T) {
        buf.clear();
    }

    #[inline(always)]
    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], Error>
    where
        T: Encode,
    {
        ENCODING.encode(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    #[inline(always)]
    pub fn decode<'de, T>(data: &'de [u8]) -> Result<T, Error>
    where
        T: Decode<'de>,
    {
        ENCODING.from_slice(data)
    }
}

#[cfg(feature = "musli-storage")]
pub mod musli_storage_packed {
    use alloc::vec::Vec;

    use ::musli_storage::int::{Fixed, Variable};
    use ::musli_storage::Encoding;
    use ::musli_storage::Error;
    use musli::{Decode, Encode};

    use crate::mode::Packed;

    const ENCODING: Encoding<Packed, Fixed, Variable> =
        Encoding::new().with_fixed_integers().with_mode();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset<T>(buf: &mut Vec<u8>, _: usize, _: &T) {
        buf.clear();
    }

    #[inline(always)]
    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], Error>
    where
        T: Encode<Packed>,
    {
        ENCODING.encode(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    #[inline(always)]
    pub fn decode<'de, T>(data: &'de [u8]) -> Result<T, Error>
    where
        T: Decode<'de, Packed>,
    {
        ENCODING.from_slice(data)
    }
}

#[cfg(feature = "musli-storage")]
pub mod musli_storage {
    use alloc::vec::Vec;

    use ::musli_storage::int::{Fixed, Variable};
    use ::musli_storage::Encoding;
    use ::musli_storage::Error;
    use musli::mode::DefaultMode;
    use musli::{Decode, Encode};

    const ENCODING: Encoding<DefaultMode, Fixed, Variable> = Encoding::new().with_fixed_integers();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset<T>(buf: &mut Vec<u8>, _: usize, _: &T) {
        buf.clear();
    }

    #[inline(always)]
    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], Error>
    where
        T: Encode,
    {
        ENCODING.encode(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    #[inline(always)]
    pub fn decode<'de, T>(data: &'de [u8]) -> Result<T, Error>
    where
        T: Decode<'de>,
    {
        ENCODING.from_slice(data)
    }
}

#[cfg(feature = "musli-wire")]
pub mod musli_wire {
    use alloc::vec::Vec;

    use ::musli_wire::int::Variable;
    use ::musli_wire::Encoding;
    use ::musli_wire::Error;
    use musli::mode::DefaultMode;
    use musli::{Decode, Encode};

    const ENCODING: Encoding<DefaultMode, Variable, Variable> = Encoding::new();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset<T>(buf: &mut Vec<u8>, _: usize, _: &T) {
        buf.clear();
    }

    #[inline(always)]
    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], Error>
    where
        T: Encode,
    {
        ENCODING.encode(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    #[inline(always)]
    pub fn decode<'de, T>(mut data: &'de [u8]) -> Result<T, Error>
    where
        T: Decode<'de>,
    {
        ENCODING.decode(&mut data)
    }
}

#[cfg(feature = "musli-descriptive")]
pub mod musli_descriptive {
    use alloc::vec::Vec;

    use ::musli_descriptive::Encoding;
    use musli::mode::DefaultMode;
    use musli::{Decode, Encode};
    use musli_descriptive::Error;

    const ENCODING: Encoding<DefaultMode> = Encoding::new();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset<T>(buf: &mut Vec<u8>, _: usize, _: &T) {
        buf.clear();
    }

    #[inline(always)]
    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], Error>
    where
        T: Encode,
    {
        ENCODING.encode(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    #[inline(always)]
    pub fn decode<'de, T>(data: &'de [u8]) -> Result<T, Error>
    where
        T: Decode<'de>,
    {
        ENCODING.decode(data)
    }
}

#[cfg(feature = "musli-value")]
pub mod musli_value {
    use ::musli_value::Value;
    use musli::{Decode, Encode};

    pub fn buffer() {}
    pub fn reset<T>(_: &mut (), _: usize, _: &T) {}

    #[inline(always)]
    pub fn encode<T>(_: &mut (), value: &T) -> musli_value::Result<Value>
    where
        T: Encode,
    {
        musli_value::encode(value)
    }

    #[inline(always)]
    pub fn decode<'de, T>(data: &'de Value) -> musli_value::Result<T>
    where
        T: Decode<'de>,
    {
        musli_value::decode(data)
    }
}

#[cfg(feature = "musli-zerocopy")]
pub mod musli_zerocopy {
    use musli_zerocopy::pointer::Ref;
    use musli_zerocopy::{AlignedBuf, Buf, Error, ZeroCopy};

    #[inline(always)]
    pub fn buffer() -> AlignedBuf {
        AlignedBuf::with_capacity(4096)
    }

    #[inline(always)]
    pub fn reset<T>(buf: &mut AlignedBuf, reserve: usize, _: &T) {
        buf.clear();
        buf.reserve(reserve);
    }

    #[inline(always)]
    pub fn encode<'de, T>(buf: &'de mut AlignedBuf, value: &T) -> Result<(&'de Buf, Ref<T>), Error>
    where
        T: ZeroCopy,
    {
        let pointer = buf.store(value)?;
        Ok((buf.as_ref(), pointer))
    }

    #[inline(always)]
    pub fn decode<'de, T>(data: &(&'de Buf, Ref<T>)) -> Result<&'de T, Error>
    where
        T: ZeroCopy,
    {
        let (buf, pointer) = *data;
        buf.load(pointer)
    }
}
