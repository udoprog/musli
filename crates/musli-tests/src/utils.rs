pub mod serde_json {
    use serde::{Deserialize, Serialize};

    pub fn encode<T>(expected: &T) -> Vec<u8>
    where
        T: Serialize,
    {
        serde_json::to_vec(expected).unwrap()
    }

    pub fn decode<'de, T>(data: &'de [u8]) -> T
    where
        T: Deserialize<'de>,
    {
        serde_json::from_slice(data).unwrap()
    }
}

pub mod musli_json {
    use ::musli_json::JsonEncoding;
    use musli::{Decode, Encode};

    const JSON_ENCODING: JsonEncoding = JsonEncoding::new();

    pub fn encode<T>(expected: &T) -> Vec<u8>
    where
        T: Encode,
    {
        JSON_ENCODING.to_vec(expected).unwrap()
    }

    pub fn decode<'de, T>(data: &'de [u8]) -> T
    where
        T: Decode<'de>,
    {
        JSON_ENCODING.from_slice(data).unwrap()
    }
}

pub mod serde_rmp {
    use serde::{Deserialize, Serialize};

    #[allow(unused)]
    pub fn encode<T>(expected: &T) -> Vec<u8>
    where
        T: Serialize,
    {
        rmp_serde::to_vec(expected).unwrap()
    }

    #[allow(unused)]
    pub fn decode<'de, T>(data: &'de [u8]) -> T
    where
        T: Deserialize<'de>,
    {
        rmp_serde::from_slice(data).unwrap()
    }
}

pub mod serde_bincode {
    use serde::{Deserialize, Serialize};

    pub fn encode<T>(expected: &T) -> Vec<u8>
    where
        T: Serialize,
    {
        bincode::serialize(expected).unwrap()
    }

    pub fn decode<'de, T>(data: &'de [u8]) -> T
    where
        T: Deserialize<'de>,
    {
        bincode::deserialize(data).unwrap()
    }
}

pub mod serde_cbor {
    use serde::{Deserialize, Serialize};

    pub fn encode<T>(expected: &T) -> Vec<u8>
    where
        T: Serialize,
    {
        serde_cbor::to_vec(expected).unwrap()
    }

    pub fn decode<'de, T>(data: &'de [u8]) -> T
    where
        T: Deserialize<'de>,
    {
        serde_cbor::from_slice(data).unwrap()
    }
}

pub mod musli_wire {
    use ::musli_wire::{Fixed, FixedLength, WireEncoding};
    use musli::mode::DefaultMode;
    use musli::{Decode, Encode};

    const WIRE_ENCODING: WireEncoding<DefaultMode, Fixed, FixedLength> = WireEncoding::new()
        .with_fixed_integers()
        .with_fixed_lengths();

    pub fn encode<T>(expected: &T) -> Vec<u8>
    where
        T: Encode,
    {
        // NB: bincode uses a 128-byte pre-allocated vector.
        let mut data = Vec::with_capacity(128);
        WIRE_ENCODING.encode(&mut data, expected).unwrap();
        data
    }

    pub fn decode<'de, T>(data: &'de [u8]) -> T
    where
        T: Decode<'de>,
    {
        WIRE_ENCODING.decode(data).unwrap()
    }
}

pub mod musli_storage {
    use ::musli_storage::{Fixed, FixedLength, StorageEncoding};
    use musli::mode::DefaultMode;
    use musli::{Decode, Encode};

    const STORAGE_ENCODING: StorageEncoding<DefaultMode, Fixed, FixedLength> =
        StorageEncoding::new()
            .with_fixed_integers()
            .with_fixed_lengths();

    pub fn encode<T>(expected: &T) -> Vec<u8>
    where
        T: Encode,
    {
        // NB: bincode uses a 128-byte pre-allocated vector.
        let mut data = Vec::with_capacity(128);
        STORAGE_ENCODING.encode(&mut data, expected).unwrap();
        data
    }

    pub fn decode<'de, T>(data: &'de [u8]) -> T
    where
        T: Decode<'de>,
    {
        STORAGE_ENCODING.from_slice(data).unwrap()
    }
}
