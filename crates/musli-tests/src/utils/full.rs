#[cfg(feature = "serde_json")]
pub mod serde_json {
    use alloc::vec::Vec;

    use serde::{Deserialize, Serialize};

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset<T>(buf: &mut Vec<u8>, _: usize, _: &T) {
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

    pub fn reset<T>(buf: &mut Vec<u8>, _: usize, _: &T) {
        buf.clear();
    }

    #[inline(always)]
    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> bincoResult<&'buf [u8]>
    where
        T: Serialize,
    {
        bincode::serialize_into(&mut *buf, value)?;
        Ok(buf.as_slice())
    }

    #[inline(always)]
    pub fn decode<'de, T>(data: &'de [u8]) -> bincoResult<T>
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

    pub fn reset<T>(buf: &mut Vec<u8>, _: usize, _: &T) {
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

#[cfg(feature = "postcard")]
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

    #[inline(always)]
    pub fn encode<'buf, T>(buf: &'buf mut Vec<u8>, value: &T) -> postcard::Result<&'buf [u8]>
    where
        T: Serialize,
    {
        let buf = postcard::to_slice(value, buf)?;
        Ok(buf)
    }

    #[inline(always)]
    pub fn decode<'de, T>(data: &'de [u8]) -> postcard::Result<T>
    where
        T: Deserialize<'de>,
    {
        postcard::from_bytes(data)
    }
}
