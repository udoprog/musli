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

#[cfg(feature = "simd-json")]
#[crate::benchmarker]
pub mod simd_json {
    use alloc::vec::Vec;

    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use simd_json::{from_slice, to_writer, Error};

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], Error>
    where
        T: Serialize,
    {
        to_writer(&mut *buf, value)?;
        Ok(buf)
    }

    pub fn decode<T>(buf: &[u8]) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let mut buf = buf.to_vec();
        from_slice(buf.as_mut_slice())
    }
}

#[cfg(feature = "bincode1")]
#[crate::benchmarker]
pub mod bincode1 {
    use alloc::vec::Vec;

    use bincode1::{deserialize, serialize_into, Error};
    use serde::{Deserialize, Serialize};

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], Error>
    where
        T: Serialize,
    {
        serialize_into(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    pub fn decode<'buf, T>(buf: &'buf [u8]) -> Result<T, Error>
    where
        T: Deserialize<'buf>,
    {
        deserialize(buf)
    }
}

#[cfg(feature = "bincode-serde")]
#[crate::benchmarker]
pub mod bincode_serde {
    use alloc::vec::Vec;

    use serde::de::DeserializeOwned;
    use serde::Serialize;

    use bincode::config::Configuration;
    use bincode::error::{DecodeError, EncodeError};

    const CONFIG: Configuration = bincode::config::standard();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.resize(2 << 20, 0);
    }

    pub fn encode<'buf, T>(buf: &'buf mut [u8], value: &T) -> Result<&'buf [u8], EncodeError>
    where
        T: Serialize,
    {
        let len = bincode::serde::encode_into_slice(value, buf, CONFIG)?;
        Ok(&buf[..len])
    }

    pub fn decode<T>(buf: &[u8]) -> Result<T, DecodeError>
    where
        T: DeserializeOwned,
    {
        let (value, _) = bincode::serde::decode_from_slice(buf, CONFIG)?;
        Ok(value)
    }
}

#[cfg(feature = "bincode-derive")]
#[crate::benchmarker]
pub mod bincode_derive {
    use alloc::vec::Vec;

    use bincode::Decode;
    use bincode::Encode;

    use bincode::config::Configuration;
    use bincode::error::{DecodeError, EncodeError};

    const CONFIG: Configuration = bincode::config::standard();

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.resize(2 << 20, 0);
    }

    pub fn encode<'buf, T>(buf: &'buf mut [u8], value: &T) -> Result<&'buf [u8], EncodeError>
    where
        T: Encode,
    {
        let len = bincode::encode_into_slice(value, buf, CONFIG)?;
        Ok(&buf[..len])
    }

    pub fn decode<T>(buf: &[u8]) -> Result<T, DecodeError>
    where
        T: Decode<()>,
    {
        let (value, _) = bincode::decode_from_slice(buf, CONFIG)?;
        Ok(value)
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

    pub fn encode<'buf, T>(buf: &'buf mut [u8], value: &T) -> Result<&'buf [u8], postcard::Error>
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

    pub fn decode<T>(decode_buf: &mut Buffer, buf: &[u8]) -> Result<T, bitcode::Error>
    where
        for<'de> T: Decode<'de>,
    {
        decode_buf.decode(buf)
    }
}

#[cfg(feature = "facet-json")]
#[crate::benchmarker]
pub mod facet_json {
    use core::fmt;

    use alloc::vec::Vec;

    use facet::Facet;

    #[derive(Debug)]
    pub struct Error;

    impl fmt::Display for Error {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Serialization error")
        }
    }

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> Result<&'buf [u8], Error>
    where
        T: for<'facet> Facet<'facet>,
    {
        facet_json::to_writer(value, &mut *buf).map_err(|_| Error)?;
        Ok(buf)
    }

    pub fn decode<'buf: 'facet, 'facet, 'shape, T>(
        buf: &'buf [u8],
    ) -> Result<T, facet_json::DeserError<'buf, 'shape>>
    where
        T: Facet<'facet>,
    {
        facet_json::from_slice(buf)
    }
}

#[cfg(feature = "facet-msgpack")]
#[crate::benchmarker]
pub mod facet_msgpack {
    use core::fmt;

    use alloc::vec::Vec;

    use facet::Facet;

    #[derive(Debug)]
    pub struct Error;

    impl fmt::Display for Error {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Serialization error")
        }
    }

    pub fn encode<T>(value: &T) -> Result<Vec<u8>, Error>
    where
        T: for<'a> Facet<'a>,
    {
        let data = facet_msgpack::to_vec(value);
        Ok(data)
    }

    pub fn decode<T>(buf: &[u8]) -> Result<T, facet_msgpack::DecodeError<'_>>
    where
        T: Facet<'static>,
    {
        facet_msgpack::from_slice(buf)
    }
}
