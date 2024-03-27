#[cfg(feature = "serde_json")]
#[crate::benchmarker]
pub mod serde_json {
    use alloc::vec::Vec;

    use serde::{Deserialize, Serialize};

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    pub fn encode<'buf, T>(
        buf: &'buf mut Vec<u8>,
        value: &T,
    ) -> Result<&'buf [u8], serde_json::Error>
    where
        T: Serialize,
    {
        serde_json::to_writer(&mut *buf, value)?;
        Ok(buf)
    }

    pub fn decode<'buf, T>(buf: &'buf [u8]) -> Result<T, serde_json::Error>
    where
        T: Deserialize<'buf>,
    {
        serde_json::from_slice(buf)
    }
}

#[cfg(feature = "bincode")]
#[crate::benchmarker]
pub mod serde_bincode {
    use alloc::vec::Vec;

    use serde::{Deserialize, Serialize};

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], bincode::Error>
    where
        T: Serialize,
    {
        bincode::serialize_into(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    pub fn decode<'buf, T>(buf: &'buf [u8]) -> Result<T, bincode::Error>
    where
        T: Deserialize<'buf>,
    {
        bincode::deserialize(buf)
    }
}

#[cfg(feature = "serde_cbor")]
#[crate::benchmarker]
pub mod serde_cbor {
    use alloc::vec::Vec;

    use serde::{Deserialize, Serialize};

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    pub fn encode<'buf, T>(
        buf: &'buf mut Vec<u8>,
        value: &T,
    ) -> Result<&'buf [u8], serde_cbor::Error>
    where
        T: Serialize,
    {
        serde_cbor::to_writer(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    pub fn decode<'buf, T>(buf: &'buf [u8]) -> Result<T, serde_cbor::Error>
    where
        T: Deserialize<'buf>,
    {
        serde_cbor::from_slice(buf)
    }
}

#[cfg(feature = "postcard")]
#[crate::benchmarker]
pub mod postcard {
    use alloc::vec::Vec;

    use serde::{Deserialize, Serialize};

    pub fn buffer() -> Vec<u8> {
        Vec::new()
    }

    pub fn reset<T>(buf: &mut Vec<u8>, size_hint: usize, value: &T)
    where
        T: Serialize,
    {
        if buf.len() < size_hint {
            buf.resize(size_hint, 0);
        }

        // Figure out the size of the buffer to use. Don't worry, anything we do
        // in `reset` doesn't count towards benchmarking.
        while let Err(error) = postcard::to_slice(value, buf) {
            match error {
                postcard::Error::SerializeBufferFull => {
                    let new_size = (buf.len() as f32 * 1.5f32) as usize;
                    buf.resize(new_size, 0);
                }
                error => {
                    panic!("{}", error)
                }
            }
        }
    }

    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], postcard::Error>
    where
        T: Serialize,
    {
        Ok(postcard::to_slice(value, buf)?)
    }

    pub fn decode<'buf, T>(buf: &'buf [u8]) -> Result<T, postcard::Error>
    where
        T: Deserialize<'buf>,
    {
        postcard::from_bytes(buf)
    }
}

#[cfg(all(feature = "bson", feature = "serde"))]
#[crate::benchmarker]
pub mod bson {
    use alloc::vec::Vec;

    use serde::{Deserialize, Serialize};

    pub fn encode<T>(value: &T) -> Result<Vec<u8>, bson::ser::Error>
    where
        T: Serialize,
    {
        bson::to_vec(value)
    }

    pub fn decode<T>(buf: &[u8]) -> Result<T, bson::de::Error>
    where
        for<'de> T: Deserialize<'de>,
    {
        bson::from_slice(buf)
    }
}

/// Bitcode lives in here with two variants, one using serde and another using
/// its own derives.
#[cfg(all(feature = "bitcode", feature = "serde"))]
#[crate::benchmarker]
pub mod serde_bitcode {
    use alloc::vec::Vec;

    use serde::{Deserialize, Serialize};

    pub fn encode<T>(value: &T) -> Result<Vec<u8>, bitcode::Error>
    where
        T: Serialize,
    {
        bitcode::serialize(value)
    }

    pub fn decode<T>(buf: &[u8]) -> Result<T, bitcode::Error>
    where
        for<'de> T: Deserialize<'de>,
    {
        bitcode::deserialize(buf)
    }
}

/// Bitcode lives in here with two variants, one using serde and another using
/// its own derives.
#[cfg(feature = "bitcode-derive")]
#[crate::benchmarker]
pub mod derive_bitcode {
    use bitcode::Buffer;
    use bitcode::{Decode, Encode};

    pub fn buffer() -> Buffer {
        Buffer::new()
    }

    #[provider]
    pub fn decode_buf() -> Buffer {
        Buffer::new()
    }

    pub fn reset<T>(buf: &mut Buffer, decode_buf: &mut Buffer, value: &T)
    where
        for<'de> T: Encode + Decode<'de>,
    {
        // Encode a value of the given type to "warm up" the buffer.
        let encoded = buf.encode(value);
        // Decode the same value to "warm up" the decode buffer.
        decode_buf.decode::<T>(encoded).unwrap();
    }

    pub fn encode<'buf, T>(buf: &'buf mut Buffer, value: &T) -> Result<&'buf [u8], bitcode::Error>
    where
        T: Encode,
    {
        Ok(buf.encode(value))
    }

    pub fn decode<'buf, T>(decode_buf: &mut Buffer, buf: &'buf [u8]) -> Result<T, bitcode::Error>
    where
        for<'de> T: Decode<'de>,
    {
        decode_buf.decode(buf)
    }
}
