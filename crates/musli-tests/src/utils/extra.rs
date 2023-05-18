/// This one lives here because it doesn't support serialization of maps with
/// other than string keys, and 128-bit numerical types.
#[cfg(feature = "dlhn")]
pub mod serde_dlhn {
    use alloc::vec::Vec;

    use dlhn::de::Deserializer;
    use dlhn::ser::Serializer;
    use serde::{Deserialize, Serialize};

    #[inline(always)]
    pub fn encode<T>(value: &T) -> Result<Vec<u8>, dlhn::ser::Error>
    where
        T: Serialize,
    {
        let mut buf = Vec::new();
        let mut serializer = Serializer::new(&mut buf);
        value.serialize(&mut serializer)?;
        Ok(buf)
    }

    #[inline(always)]
    pub fn decode<T>(mut data: &[u8]) -> Result<T, dlhn::de::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut deserializer = Deserializer::new(&mut data);
        T::deserialize(&mut deserializer)
    }
}

/// rkyv lives here for a whole variety of reasons.
///
/// It has limited type support, so comparing it with full serialization methods
/// would not be fair.
#[cfg(feature = "rkyv")]
pub mod rkyv {
    use rkyv::ser::serializers::AllocSerializer;
    use rkyv::ser::Serializer;
    use rkyv::validation::validators::DefaultValidator;
    use rkyv::validation::CheckTypeError;
    use rkyv::{Archive, CheckBytes, Fallible, Serialize};

    #[inline(always)]
    pub fn encode<T>(value: &T) -> Result<rkyv::AlignedVec, <AllocSerializer<0> as Fallible>::Error>
    where
        T: Serialize<AllocSerializer<0>>,
    {
        let mut serializer = AllocSerializer::<0>::default();
        serializer.serialize_value(value)?;
        Ok(serializer.into_serializer().into_inner())
    }

    #[inline(always)]
    pub fn decode<'de, T>(
        data: &'de [u8],
    ) -> Result<&'de T::Archived, CheckTypeError<T::Archived, DefaultValidator<'de>>>
    where
        T: Archive,
        T::Archived: CheckBytes<DefaultValidator<'de>>,
    {
        rkyv::check_archived_root::<T>(data)
    }
}

/// RMP lacks support for certain numerical types.
#[cfg(feature = "rmp-serde")]
pub mod serde_rmp {
    use alloc::vec::Vec;

    use serde::{Deserialize, Serialize};

    #[inline(always)]
    pub fn encode<T>(value: &T) -> Result<Vec<u8>, rmp_serde::encode::Error>
    where
        T: Serialize,
    {
        rmp_serde::to_vec(value)
    }

    #[inline(always)]
    pub fn decode<'de, T>(data: &'de [u8]) -> Result<T, rmp_serde::decode::Error>
    where
        T: Deserialize<'de>,
    {
        rmp_serde::from_slice(data)
    }
}

/// Bitcode lives in here with two variants, one using serde and another using
/// its own derives.
///
/// It lacks support for 128-bit numerical types.
#[cfg(feature = "bitcode")]
pub mod serde_bitcode {
    use alloc::vec::Vec;

    use serde::{Deserialize, Serialize};

    #[inline(always)]
    pub fn encode<T>(value: &T) -> Result<Vec<u8>, bitcode::Error>
    where
        T: Serialize,
    {
        bitcode::serialize(value)
    }

    #[inline(always)]
    pub fn decode<T>(data: &[u8]) -> Result<T, bitcode::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        bitcode::deserialize(data)
    }
}

/// Bitcode lives in here with two variants, one using serde and another using
/// its own derives.
///
/// It lacks support for 128-bit numerical types.
#[cfg(feature = "bitcode")]
pub mod derive_bitcode {
    use alloc::vec::Vec;

    use bitcode::{Decode, Encode};

    #[inline(always)]
    pub fn encode<T>(value: &T) -> Result<Vec<u8>, bitcode::Error>
    where
        T: Encode,
    {
        bitcode::encode(value)
    }

    #[inline(always)]
    pub fn decode<T>(data: &[u8]) -> Result<T, bitcode::Error>
    where
        T: Decode,
    {
        bitcode::decode(data)
    }
}
