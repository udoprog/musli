/// This one lives here because it doesn't support serialization of maps with
/// other than string keys, and 128-bit numerical types.
#[cfg(feature = "dlhn")]
pub mod serde_dlhn {
    use alloc::vec::Vec;

    use dlhn::de::Deserializer;
    use dlhn::ser::Serializer;
    use serde::{Deserialize, Serialize};

    benchmarker! {
        'buf

        pub fn buffer() -> Vec<u8> {
            Vec::with_capacity(4096)
        }

        pub fn reset<T>(&mut self, _: usize, _: &T) {
            self.buffer.clear();
        }

        pub fn encode<T>(&mut self, value: &T) -> Result<&'buf [u8], dlhn::ser::Error>
        where
            T: Serialize,
        {
            let mut serializer = Serializer::new(&mut *self.buffer);
            value.serialize(&mut serializer)?;
            Ok(self.buffer.as_slice())
        }

        pub fn decode<T>(&self) -> Result<T, dlhn::de::Error>
        where
            for<'de> T: Deserialize<'de>,
        {
            let mut buffer = &self.buffer[..];
            let mut deserializer = Deserializer::new(&mut buffer);
            T::deserialize(&mut deserializer)
        }
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
        pub fn with<T, O>(&mut self, runner: T) -> O
        where
            T: FnOnce(State<'_, '_>) -> O,
        {
            runner(State {
                serialize_buffer: &mut self.serialize_buffer,
                serialize_scratch: &mut self.serialize_scratch,
            })
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
pub mod serde_rmp {
    use alloc::vec::Vec;

    use serde::{Deserialize, Serialize};

    benchmarker! {
        'buf

        pub fn buffer() -> Vec<u8> {
            Vec::with_capacity(4096)
        }

        pub fn reset<T>(&mut self, _: usize, _: &T) {
            self.buffer.clear();
        }

        pub fn encode<T>(&mut self, value: &T) -> Result<&'buf [u8], rmp_serde::encode::Error>
        where
            T: Serialize,
        {
            rmp_serde::encode::write(&mut *self.buffer, value)?;
            Ok(self.buffer.as_slice())
        }

        pub fn decode<T>(&self) -> Result<T, rmp_serde::decode::Error>
        where
            T: Deserialize<'buf>,
        {
            rmp_serde::from_slice(self.buffer)
        }
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

    benchmarker! {
        'buf

        pub fn buffer() -> Buffer {
            Buffer::with_capacity(4096)
        }

        pub fn reset<T>(&mut self, _: usize, _: &T) {}

        pub fn encode<T>(&mut self, value: &T) -> Result<&'buf [u8], bitcode::Error>
        where
            T: Serialize,
        {
            self.buffer.serialize(value)
        }

        pub fn decode<T>(&self) -> Result<T, bitcode::Error>
        where
            for<'de> T: Deserialize<'de>,
        {
            bitcode::deserialize(self.buffer)
        }
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

    benchmarker! {
        'buf

        pub fn buffer() -> Buffer {
            Buffer::with_capacity(4096)
        }

        pub fn reset<T>(&mut self, _: usize, _: &T) {}

        pub fn encode<T>(&mut self, value: &T) -> Result<&'buf [u8], bitcode::Error>
        where
            T: Encode,
        {
            self.buffer.encode(value)
        }

        pub fn decode<T>(&self) -> Result<T, bitcode::Error>
        where
            T: Decode,
        {
            bitcode::decode(self.buffer)
        }
    }
}

#[cfg(feature = "zerocopy")]
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

    benchmarker! {
        'buf

        pub fn buffer() -> Vec<u8> {
            Vec::with_capacity(4096)
        }

        pub fn reset<T>(&mut self, _: usize, _: &T) {
            self.buffer.clear();
        }

        pub fn encode<T>(&mut self, value: &T) -> Result<&'buf [u8], Error>
        where
            T: AsBytes,
        {
            self.buffer.extend_from_slice(value.as_bytes());
            Ok(self.buffer.as_slice())
        }

        pub fn decode<T>(&self) -> Result<T, Error>
        where
            T: FromBytes,
        {
            T::read_from(&self.buffer[..]).ok_or(Error)
        }
    }
}
