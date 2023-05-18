#[cfg(feature = "serde_json")]
pub mod serde_json {
    use alloc::vec::Vec;

    use serde::{Deserialize, Serialize};

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    #[inline(always)]
    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> serde_json::Result<&'buf [u8]>
    where
        T: Serialize,
    {
        serde_json::to_writer(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    #[inline(always)]
    pub fn decode<'de, T>(data: &'de [u8]) -> serde_json::Result<T>
    where
        T: Deserialize<'de>,
    {
        serde_json::from_slice(data)
    }
}

#[cfg(feature = "bincode")]
pub mod serde_bincode {
    use alloc::vec::Vec;

    use serde::{Deserialize, Serialize};

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    #[inline(always)]
    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> bincode::Result<&'buf [u8]>
    where
        T: Serialize,
    {
        bincode::serialize_into(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    #[inline(always)]
    pub fn decode<'de, T>(data: &'de [u8]) -> bincode::Result<T>
    where
        T: Deserialize<'de>,
    {
        bincode::deserialize(data)
    }
}

#[cfg(feature = "serde_cbor")]
pub mod serde_cbor {
    use alloc::vec::Vec;

    use serde::{Deserialize, Serialize};

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    #[inline(always)]
    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> serde_cbor::Result<&'buf [u8]>
    where
        T: Serialize,
    {
        serde_cbor::to_writer(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    #[inline(always)]
    pub fn decode<'de, T>(data: &'de [u8]) -> serde_cbor::Result<T>
    where
        T: Deserialize<'de>,
    {
        serde_cbor::from_slice(data)
    }
}
