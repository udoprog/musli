/// This one lives here because it doesn't support serialization of maps with
/// other than string keys, and 128-bit numerical types.
#[cfg(feature = "dlhn")]
#[crate::benchmarker]
pub mod serde_dlhn {
    use alloc::vec::Vec;

    use dlhn::de::Deserializer;
    use dlhn::ser::Serializer;
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
    ) -> Result<&'buf [u8], dlhn::ser::Error>
    where
        T: Serialize,
    {
        let mut serializer = Serializer::new(&mut *buf);
        value.serialize(&mut serializer)?;
        Ok(buf)
    }

    pub fn decode<T>(mut buf: &[u8]) -> Result<T, dlhn::de::Error>
    where
        for<'de> T: Deserialize<'de>,
    {
        let mut deserializer = Deserializer::new(&mut buf);
        T::deserialize(&mut deserializer)
    }
}

/// rkyv lives here for a whole variety of reasons.
///
/// It has limited type support, so comparing it with full serialization methods
/// would not be fair.
#[cfg(feature = "rkyv")]
#[crate::benchmarker]
pub mod rkyv {
    use rkyv::api::high::{to_bytes_in, HighDeserializer, HighSerializer, HighValidator};
    use rkyv::bytecheck::CheckBytes;
    use rkyv::rancor::Failure;
    use rkyv::ser::allocator::ArenaHandle;
    use rkyv::util::AlignedVec;
    use rkyv::{Archive, Serialize};

    pub type Serializer<'buf, 'arena> =
        HighSerializer<&'buf mut AlignedVec, ArenaHandle<'arena>, Failure>;
    pub type Deserializer = HighDeserializer<Failure>;
    pub type Validator<'a> = HighValidator<'a, Failure>;

    const BUFFER_LEN: usize = 10_000_000;

    struct Buffers {
        serializer: AlignedVec,
    }

    pub fn buffer() -> Buffers {
        Buffers {
            serializer: AlignedVec::with_capacity(BUFFER_LEN),
        }
    }

    pub fn encode<'buf, T>(buf: &'buf mut Buffers, value: &T) -> Result<&'buf [u8], Failure>
    where
        T: for<'a, 'b> Serialize<Serializer<'a, 'b>>,
    {
        to_bytes_in(value, &mut buf.serializer)?;
        Ok(buf.serializer.as_slice())
    }

    pub fn decode<T>(buf: &[u8]) -> Result<&T::Archived, Failure>
    where
        T: Archive<Archived: for<'a> CheckBytes<Validator<'a>>>,
    {
        rkyv::access(buf)
    }
}

/// RMP lacks support for certain numerical types.
#[cfg(feature = "rmp-serde")]
#[crate::benchmarker]
pub mod serde_rmp {
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
    ) -> Result<&'buf [u8], rmp_serde::encode::Error>
    where
        T: Serialize,
    {
        rmp_serde::encode::write(&mut *buf, value)?;
        Ok(buf)
    }

    pub fn decode<'buf, T>(buf: &'buf [u8]) -> Result<T, rmp_serde::decode::Error>
    where
        T: Deserialize<'buf>,
    {
        rmp_serde::from_slice(buf)
    }
}

#[cfg(feature = "zerocopy")]
#[crate::benchmarker]
pub mod zerocopy {
    use core::fmt;

    use alloc::vec::Vec;

    use anyhow::Result;
    use zerocopy::{FromBytes, Immutable, IntoBytes};

    #[derive(Debug)]
    pub struct Error;

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Error during decoding")
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
        T: Immutable + IntoBytes,
    {
        buf.extend_from_slice(value.as_bytes());
        Ok(buf.as_slice())
    }

    pub fn decode<T>(buf: &[u8]) -> Result<T, Error>
    where
        T: FromBytes,
    {
        T::read_from_bytes(buf).map_err(|_| Error)
    }
}

#[cfg(feature = "miniserde")]
#[crate::benchmarker]
pub mod miniserde {
    use alloc::vec::Vec;
    use miniserde::{json, Deserialize, Serialize};

    pub fn encode<T>(value: &T) -> Result<Vec<u8>, miniserde::Error>
    where
        T: Serialize,
    {
        Ok(json::to_string(value).into_bytes())
    }

    pub fn decode<T>(buf: &[u8]) -> Result<T, miniserde::Error>
    where
        T: Deserialize,
    {
        let string = std::str::from_utf8(buf).map_err(|_| miniserde::Error)?;
        json::from_str(string)
    }
}
