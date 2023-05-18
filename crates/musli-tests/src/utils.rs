mod full;
pub use self::full::*;

mod extra;
pub use self::extra::*;

#[cfg(feature = "musli-json")]
pub mod musli_json {
    use alloc::vec::Vec;

    use ::musli_json::error::BufferError;
    use ::musli_json::{Encoding, ParseError};
    use musli::{Decode, Encode};

    const ENCODING: Encoding = Encoding::new();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    #[inline(always)]
    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], BufferError>
    where
        T: Encode,
    {
        ENCODING.encode(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    #[inline(always)]
    pub fn decode<'de, T>(data: &'de [u8]) -> Result<T, ParseError>
    where
        T: Decode<'de>,
    {
        ENCODING.from_slice(data)
    }
}

#[cfg(feature = "musli-storage")]
pub mod musli_storage_packed {
    use alloc::vec::Vec;

    use ::musli_storage::error::BufferError;
    use ::musli_storage::int::{Fixed, Variable};
    use ::musli_storage::Encoding;
    use musli::{Decode, Encode};

    use crate::mode::Packed;

    const ENCODING: Encoding<Packed, Fixed, Variable> =
        Encoding::new().with_fixed_integers().with_mode::<Packed>();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    #[inline(always)]
    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], BufferError>
    where
        T: Encode<Packed>,
    {
        ENCODING.encode(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    #[inline(always)]
    pub fn decode<'de, T>(data: &'de [u8]) -> Result<T, BufferError>
    where
        T: Decode<'de, Packed>,
    {
        ENCODING.from_slice(data)
    }
}

#[cfg(feature = "musli-storage")]
pub mod musli_storage {
    use alloc::vec::Vec;

    use ::musli_storage::error::BufferError;
    use ::musli_storage::int::{Fixed, Variable};
    use ::musli_storage::Encoding;
    use musli::mode::DefaultMode;
    use musli::{Decode, Encode};

    const ENCODING: Encoding<DefaultMode, Fixed, Variable> = Encoding::new().with_fixed_integers();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    #[inline(always)]
    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], BufferError>
    where
        T: Encode,
    {
        ENCODING.encode(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    #[inline(always)]
    pub fn decode<'de, T>(data: &'de [u8]) -> Result<T, BufferError>
    where
        T: Decode<'de>,
    {
        ENCODING.from_slice(data)
    }
}

#[cfg(feature = "musli-wire")]
pub mod musli_wire {
    use alloc::vec::Vec;

    use ::musli_wire::error::BufferError;
    use ::musli_wire::int::Variable;
    use ::musli_wire::Encoding;
    use musli::mode::DefaultMode;
    use musli::{Decode, Encode};

    const ENCODING: Encoding<DefaultMode, Variable, Variable> = Encoding::new();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    #[inline(always)]
    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], BufferError>
    where
        T: Encode,
    {
        ENCODING.encode(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    #[inline(always)]
    pub fn decode<'de, T>(mut data: &'de [u8]) -> Result<T, BufferError>
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
    use musli_descriptive::error::BufferError;

    const ENCODING: Encoding<DefaultMode> = Encoding::new();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    #[inline(always)]
    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], BufferError>
    where
        T: Encode,
    {
        ENCODING.encode(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    #[inline(always)]
    pub fn decode<'de, T>(data: &'de [u8]) -> Result<T, BufferError>
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
    pub fn reset(_: &mut ()) {}

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
