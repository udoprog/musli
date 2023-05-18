#[cfg(feature = "serde_json")]
pub mod serde_json {
    use alloc::vec::Vec;

    use serde::{Deserialize, Serialize};

    #[inline(always)]
    pub fn encode<T>(value: &T) -> serde_json::Result<Vec<u8>>
    where
        T: Serialize,
    {
        serde_json::to_vec(value)
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

    #[inline(always)]
    pub fn encode<T>(value: &T) -> bincode::Result<Vec<u8>>
    where
        T: Serialize,
    {
        let mut data = Vec::new();
        bincode::serialize_into(&mut data, value)?;
        Ok(data)
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

    #[inline(always)]
    pub fn encode<T>(value: &T) -> serde_cbor::Result<Vec<u8>>
    where
        T: Serialize,
    {
        serde_cbor::to_vec(value)
    }

    #[inline(always)]
    pub fn decode<'de, T>(data: &'de [u8]) -> serde_cbor::Result<T>
    where
        T: Deserialize<'de>,
    {
        serde_cbor::from_slice(data)
    }
}
