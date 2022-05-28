pub mod serde_dlhn {
    use dlhn::de::Deserializer;
    use dlhn::ser::Serializer;
    use serde::{Deserialize, Serialize};

    pub fn encode<T>(value: &T) -> Vec<u8>
    where
        T: Serialize,
    {
        let mut buf = Vec::new();
        let mut serializer = Serializer::new(&mut buf);
        value.serialize(&mut serializer).unwrap();
        buf
    }

    pub fn decode<T>(mut data: &[u8]) -> T
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut deserializer = Deserializer::new(&mut data);
        T::deserialize(&mut deserializer).unwrap()
    }
}

pub mod serde_json {
    use serde::{Deserialize, Serialize};

    pub fn encode<T>(value: &T) -> Vec<u8>
    where
        T: Serialize,
    {
        serde_json::to_vec(value).unwrap()
    }

    pub fn decode<'de, T>(data: &'de [u8]) -> T
    where
        T: Deserialize<'de>,
    {
        serde_json::from_slice(data).unwrap()
    }
}

pub mod musli_json {
    use ::musli_json::Encoding;
    use musli::{Decode, Encode};

    const ENCODING: Encoding = Encoding::new();

    pub fn encode<T>(value: &T) -> Vec<u8>
    where
        T: Encode,
    {
        ENCODING.to_buffer(value).unwrap().into_vec()
    }

    pub fn decode<'de, T>(data: &'de [u8]) -> T
    where
        T: Decode<'de>,
    {
        ENCODING.from_slice(data).unwrap()
    }
}

pub mod serde_rmp {
    use serde::{Deserialize, Serialize};

    #[allow(unused)]
    pub fn encode<T>(value: &T) -> Vec<u8>
    where
        T: Serialize,
    {
        rmp_serde::to_vec(value).unwrap()
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
    use std::io::Cursor;

    pub fn encode<T>(value: &T) -> Vec<u8>
    where
        T: Serialize,
    {
        let mut writer = Cursor::new(Vec::new());
        bincode::serialize_into(&mut writer, value).unwrap();
        writer.into_inner()
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

    pub fn encode<T>(value: &T) -> Vec<u8>
    where
        T: Serialize,
    {
        serde_cbor::to_vec(value).unwrap()
    }

    pub fn decode<'de, T>(data: &'de [u8]) -> T
    where
        T: Deserialize<'de>,
    {
        serde_cbor::from_slice(data).unwrap()
    }
}

pub mod musli_storage {
    use ::musli_storage::int::{Fixed, FixedUsize};
    use ::musli_storage::Encoding;
    use musli::mode::DefaultMode;
    use musli::{Decode, Encode};

    const ENCODING: Encoding<DefaultMode, Fixed, FixedUsize<u64>> =
        Encoding::new().with_fixed_integers().with_fixed_lengths64();

    pub fn encode<T>(value: &T) -> Vec<u8>
    where
        T: Encode,
    {
        ENCODING.to_buffer(value).unwrap().into_vec()
    }

    pub fn decode<'de, T>(data: &'de [u8]) -> T
    where
        T: Decode<'de>,
    {
        ENCODING.from_slice(data).unwrap()
    }
}

pub mod musli_wire {
    use ::musli_wire::int::{Fixed, FixedUsize};
    use ::musli_wire::Encoding;
    use musli::mode::DefaultMode;
    use musli::{Decode, Encode};

    const ENCODING: Encoding<DefaultMode, Fixed, FixedUsize<u64>> =
        Encoding::new().with_fixed_integers().with_fixed_lengths64();

    pub fn encode<T>(value: &T) -> Vec<u8>
    where
        T: Encode,
    {
        ENCODING.to_buffer(value).unwrap().into_vec()
    }

    pub fn decode<'de, T>(data: &'de [u8]) -> T
    where
        T: Decode<'de>,
    {
        ENCODING.decode(data).unwrap()
    }
}

pub mod musli_descriptive {
    use ::musli_descriptive::Encoding;
    use musli::mode::DefaultMode;
    use musli::{Decode, Encode};

    const ENCODING: Encoding<DefaultMode> = Encoding::new();

    pub fn encode<T>(value: &T) -> Vec<u8>
    where
        T: Encode,
    {
        ENCODING.to_buffer(value).unwrap().into_vec()
    }

    pub fn decode<'de, T>(data: &'de [u8]) -> T
    where
        T: Decode<'de>,
    {
        ENCODING.decode(data).unwrap()
    }
}

pub mod musli_value {
    use ::musli_value::Value;
    use musli::{Decode, Encode};

    pub fn encode<T>(value: &T) -> Value
    where
        T: Encode,
    {
        musli_value::encode(value).unwrap()
    }

    pub fn decode<'de, T>(data: &'de Value) -> T
    where
        T: Decode<'de>,
    {
        musli_value::decode(data).unwrap()
    }
}
