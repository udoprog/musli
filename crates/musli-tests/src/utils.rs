#[cfg(feature = "dlhn")]
pub mod serde_dlhn {
    use alloc::vec::Vec;

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

#[cfg(feature = "serde_json")]
pub mod serde_json {
    use alloc::vec::Vec;

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

#[cfg(feature = "musli-json")]
pub mod musli_json {
    use alloc::vec::Vec;

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

#[cfg(feature = "rmp-serde")]
pub mod serde_rmp {
    use alloc::vec::Vec;

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

#[cfg(feature = "bincode")]
pub mod serde_bincode {
    use alloc::vec::Vec;

    use serde::{Deserialize, Serialize};

    pub fn encode<T>(value: &T) -> Vec<u8>
    where
        T: Serialize,
    {
        let mut data = Vec::new();
        bincode::serialize_into(&mut data, value).unwrap();
        data
    }

    pub fn decode<'de, T>(data: &'de [u8]) -> T
    where
        T: Deserialize<'de>,
    {
        bincode::deserialize(data).unwrap()
    }
}

#[cfg(feature = "serde_cbor")]
pub mod serde_cbor {
    use alloc::vec::Vec;

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

#[cfg(feature = "musli-storage")]
pub mod musli_storage_packed {
    use alloc::vec::Vec;

    use ::musli_storage::int::{Fixed, FixedUsize};
    use ::musli_storage::Encoding;
    use musli::{Decode, Encode};

    use crate::mode::Packed;

    const ENCODING: Encoding<Packed, Fixed, FixedUsize<u64>> = Encoding::new()
        .with_fixed_integers()
        .with_fixed_lengths64()
        .with_mode::<Packed>();

    pub fn encode<T>(value: &T) -> Vec<u8>
    where
        T: Encode<Packed>,
    {
        ENCODING.to_buffer(value).unwrap().into_vec()
    }

    pub fn decode<'de, T>(data: &'de [u8]) -> T
    where
        T: Decode<'de, Packed>,
    {
        ENCODING.from_slice(data).unwrap()
    }
}

#[cfg(feature = "musli-storage")]
pub mod musli_storage {
    use alloc::vec::Vec;

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

#[cfg(feature = "musli-wire")]
pub mod musli_wire {
    use alloc::vec::Vec;

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

#[cfg(feature = "musli-descriptive")]
pub mod musli_descriptive {
    use alloc::vec::Vec;

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

#[cfg(feature = "musli-value")]
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

#[cfg(feature = "bitcode")]
pub mod serde_bitcode {
    use alloc::vec::Vec;

    use serde::{Deserialize, Serialize};

    pub fn encode<T>(value: &T) -> Vec<u8>
    where
        T: Serialize,
    {
        bitcode::serialize(value).unwrap()
    }

    pub fn decode<T>(data: &[u8]) -> T
    where
        T: for<'de> Deserialize<'de>,
    {
        bitcode::deserialize(data).unwrap()
    }
}

#[cfg(feature = "bitcode")]
pub mod derive_bitcode {
    use alloc::vec::Vec;

    use bitcode::{Decode, Encode};

    pub fn encode<T>(value: &T) -> Vec<u8>
    where
        T: Encode,
    {
        bitcode::encode(value).unwrap()
    }

    pub fn decode<T>(data: &[u8]) -> T
    where
        T: Decode,
    {
        bitcode::decode(data).unwrap()
    }
}

#[cfg(feature = "rkyv")]
pub mod rkyv {
    use rkyv::ser::serializers::AllocSerializer;
    use rkyv::ser::Serializer;
    use rkyv::validation::validators::DefaultValidator;
    use rkyv::{Archive, CheckBytes, Serialize};

    pub fn encode<T>(value: &T) -> rkyv::AlignedVec
    where
        T: Serialize<AllocSerializer<0>>,
    {
        let mut serializer = AllocSerializer::<0>::default();
        serializer.serialize_value(&*value).unwrap();
        serializer.into_serializer().into_inner()
    }

    pub fn decode<'de, T>(data: &'de [u8]) -> &'de T::Archived
    where
        T: Archive,
        T::Archived: CheckBytes<DefaultValidator<'de>>,
    {
        rkyv::check_archived_root::<T>(data).unwrap()
    }
}
