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
pub mod rkyv {
    use rkyv::ser::serializers::{AlignedSerializer, BufferScratch, CompositeSerializer};
    use rkyv::ser::Serializer;
    use rkyv::validation::validators::DefaultValidator;
    use rkyv::validation::CheckTypeError;
    use rkyv::{AlignedVec, Archive, CheckBytes, Fallible, Infallible, Serialize};

    const BUFFER_LEN: usize = 10_000_000;
    const SCRATCH_LEN: usize = 512_000;

    pub struct Benchmarker {
        serialize_buffer: AlignedVec,
        serialize_scratch: AlignedVec,
    }

    pub fn new() -> Benchmarker {
        let serialize_buffer = AlignedVec::with_capacity(BUFFER_LEN);
        let mut serialize_scratch = AlignedVec::with_capacity(SCRATCH_LEN);

        // SAFETY: I don't know why this is OK.
        unsafe {
            serialize_scratch.set_len(SCRATCH_LEN);
        }

        Benchmarker {
            serialize_buffer,
            serialize_scratch,
        }
    }

    type S<'buf> = CompositeSerializer<
        AlignedSerializer<&'buf mut AlignedVec>,
        BufferScratch<&'buf mut AlignedVec>,
        Infallible,
    >;

    pub struct State<'buf, 'scratch> {
        serialize_buffer: &'buf mut AlignedVec,
        serialize_scratch: &'scratch mut AlignedVec,
    }

    impl Benchmarker {
        pub fn state(&mut self) -> State<'_, '_> {
            State {
                serialize_buffer: &mut self.serialize_buffer,
                serialize_scratch: &mut self.serialize_scratch,
            }
        }
    }

    pub struct DecodeState<'buf> {
        bytes: &'buf [u8],
    }

    #[inline(always)]
    pub fn decode<'de, T>(
        bytes: &'de [u8],
    ) -> Result<&'de T::Archived, CheckTypeError<T::Archived, DefaultValidator<'de>>>
    where
        T: Archive,
        T::Archived: CheckBytes<DefaultValidator<'de>>,
    {
        rkyv::check_archived_root::<T>(bytes)
    }

    impl<'de> DecodeState<'de> {
        #[inline(always)]
        pub fn len(&self) -> usize {
            self.bytes.len()
        }

        pub fn as_bytes(&self) -> Option<&'de [u8]> {
            Some(self.bytes)
        }

        #[inline(always)]
        pub fn decode<T>(
            &self,
        ) -> Result<&'de T::Archived, CheckTypeError<T::Archived, DefaultValidator<'de>>>
        where
            T: Archive,
            T::Archived: CheckBytes<DefaultValidator<'de>>,
        {
            self::decode::<T>(self.bytes)
        }
    }

    impl<'buf, 'scratch> State<'buf, 'scratch> {
        pub fn reset<T>(&mut self, _: usize, _: &T) {
            self.serialize_buffer.clear();
        }

        #[inline(always)]
        pub fn encode<T>(
            &mut self,
            value: &T,
        ) -> Result<DecodeState<'_>, <S<'buf> as Fallible>::Error>
        where
            T: for<'value> Serialize<S<'value>>,
        {
            let mut serializer = CompositeSerializer::new(
                AlignedSerializer::new(&mut *self.serialize_buffer),
                BufferScratch::new(&mut *self.serialize_scratch),
                Infallible,
            );

            serializer.serialize_value(value)?;
            let bytes = serializer.into_serializer().into_inner();
            Ok(DecodeState { bytes })
        }
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
    use zerocopy::{AsBytes, FromBytes};

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
        T: AsBytes,
    {
        buf.extend_from_slice(value.as_bytes());
        Ok(buf.as_slice())
    }

    pub fn decode<'buf, T>(buf: &[u8]) -> Result<T, Error>
    where
        T: FromBytes,
    {
        T::read_from(buf).ok_or(Error)
    }
}
