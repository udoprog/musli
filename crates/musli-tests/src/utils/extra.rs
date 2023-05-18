/// This one lives here because it doesn't support serialization of maps with
/// other than string keys, and 128-bit numerical types.
#[cfg(feature = "dlhn")]
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

    #[inline(always)]
    pub fn encode<'buf, T>(
        buf: &'buf mut Vec<u8>,
        value: &T,
    ) -> Result<&'buf [u8], dlhn::ser::Error>
    where
        T: Serialize,
    {
        let mut serializer = Serializer::new(&mut *buf);
        value.serialize(&mut serializer)?;
        Ok(buf.as_slice())
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
    use rkyv::ser::serializers::{AlignedSerializer, BufferScratch, CompositeSerializer};
    use rkyv::ser::Serializer;
    use rkyv::validation::validators::DefaultValidator;
    use rkyv::validation::CheckTypeError;
    use rkyv::{AlignedVec, Archive, CheckBytes, Fallible, Infallible, Serialize};

    const BUFFER_LEN: usize = 10_000_000;
    const SCRATCH_LEN: usize = 512_000;

    pub struct Buffers {
        serialize_buffer: AlignedVec,
        serialize_scratch: AlignedVec,
    }

    pub fn buffer() -> Buffers {
        let serialize_buffer = AlignedVec::with_capacity(BUFFER_LEN);
        let mut serialize_scratch = AlignedVec::with_capacity(SCRATCH_LEN);

        // SAFETY: I don't know why this is OK.
        unsafe {
            serialize_scratch.set_len(SCRATCH_LEN);
        }

        Buffers {
            serialize_buffer,
            serialize_scratch,
        }
    }

    pub fn reset(buf: &mut Buffers) {
        buf.serialize_buffer.clear();
    }

    type S<'buf> = CompositeSerializer<
        AlignedSerializer<&'buf mut AlignedVec>,
        BufferScratch<&'buf mut AlignedVec>,
        Infallible,
    >;

    #[inline(always)]
    pub fn encode<'buf, T>(
        buf: &'buf mut Buffers,
        value: &T,
    ) -> Result<&'buf [u8], <S<'buf> as Fallible>::Error>
    where
        T: for<'value> Serialize<S<'value>>,
    {
        let mut serializer = CompositeSerializer::new(
            AlignedSerializer::new(&mut buf.serialize_buffer),
            BufferScratch::new(&mut buf.serialize_scratch),
            Infallible,
        );

        serializer.serialize_value(value)?;
        let bytes = serializer.into_serializer().into_inner();
        Ok(bytes)
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

    pub fn buffer() -> Vec<u8> {
        Vec::with_capacity(4096)
    }

    pub fn reset(buf: &mut Vec<u8>) {
        buf.clear();
    }

    #[inline(always)]
    pub fn encode<'buf, T>(
        buf: &'buf mut Vec<u8>,
        value: &T,
    ) -> Result<&'buf [u8], rmp_serde::encode::Error>
    where
        T: Serialize,
    {
        rmp_serde::encode::write(&mut *buf, value)?;
        Ok(buf.as_slice())
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
    use bitcode::Buffer;
    use serde::{Deserialize, Serialize};

    pub fn buffer() -> Buffer {
        Buffer::with_capacity(4096)
    }

    pub fn reset(_: &mut Buffer) {}

    #[inline(always)]
    pub fn encode<'buf, T>(buf: &'buf mut Buffer, value: &T) -> Result<&'buf [u8], bitcode::Error>
    where
        T: Serialize,
    {
        buf.serialize(value)
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
    use bitcode::Buffer;
    use bitcode::{Decode, Encode};

    pub fn buffer() -> Buffer {
        Buffer::with_capacity(4096)
    }

    pub fn reset(_: &mut Buffer) {}

    #[inline(always)]
    pub fn encode<'buf, T>(buf: &'buf mut Buffer, value: &T) -> Result<&'buf [u8], bitcode::Error>
    where
        T: Encode,
    {
        buf.encode(value)
    }

    #[inline(always)]
    pub fn decode<T>(data: &[u8]) -> Result<T, bitcode::Error>
    where
        T: Decode,
    {
        bitcode::decode(data)
    }
}
