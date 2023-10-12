#[macro_use]
mod macros;

pub use self::full::*;
mod full;

pub use self::extra::*;
mod extra;

#[cfg(feature = "musli-json")]
pub mod musli_json {
    use alloc::vec::Vec;

    use ::musli_json::Encoding;
    use ::musli_json::Error;
    use musli::{Decode, Encode};

    const ENCODING: Encoding = Encoding::new();

    benchmarker! {
        'buf

        pub fn buffer() -> Vec<u8> {
            Vec::with_capacity(4096)
        }

        pub fn reset<T>(&mut self, _size_hint: usize, _value: &T) {
            self.buffer.clear();
        }

        pub fn encode<T>(&mut self, value: &T) -> Result<&'buf [u8], Error>
        where
            T: Encode,
        {
            ENCODING.encode(&mut *self.buffer, value)?;
            Ok(self.buffer.as_slice())
        }

        pub fn decode<T>(&self) -> Result<T, Error>
        where
            T: Decode<'buf>,
        {
            ENCODING.from_slice(self.buffer)
        }
    }
}

#[cfg(feature = "musli-storage")]
pub mod musli_storage_packed {
    use alloc::vec::Vec;

    use ::musli_storage::int::{Fixed, Variable};
    use ::musli_storage::Encoding;
    use ::musli_storage::Error;
    use musli::{Decode, Encode};

    use crate::mode::Packed;

    const ENCODING: Encoding<Packed, Fixed, Variable> =
        Encoding::new().with_fixed_integers().with_mode();

    benchmarker! {
        'buf

        pub fn buffer() -> Vec<u8> {
            Vec::with_capacity(4096)
        }

        pub fn reset<T>(&mut self, _size_hint: usize, _value: &T) {
            self.buffer.clear();
        }

        pub fn encode<T>(&mut self, value: &T) -> Result<&'buf [u8], Error>
        where
            T: Encode<Packed>,
        {
            ENCODING.encode(&mut *self.buffer, value)?;
            Ok(self.buffer.as_slice())
        }

        pub fn decode<T>(&self) -> Result<T, Error>
        where
            T: Decode<'buf, Packed>,
        {
            ENCODING.from_slice(self.buffer)
        }
    }
}

#[cfg(feature = "musli-storage")]
pub mod musli_storage {
    use alloc::vec::Vec;

    use ::musli_storage::int::{Fixed, Variable};
    use ::musli_storage::Encoding;
    use ::musli_storage::Error;
    use musli::mode::DefaultMode;
    use musli::{Decode, Encode};

    const ENCODING: Encoding<DefaultMode, Fixed, Variable> = Encoding::new().with_fixed_integers();

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
            T: Encode,
        {
            ENCODING.encode(&mut *self.buffer, value)?;
            Ok(self.buffer.as_slice())
        }

        pub fn decode<T>(&self) -> Result<T, Error>
        where
            T: Decode<'buf>,
        {
            ENCODING.from_slice(self.buffer)
        }
    }
}

#[cfg(feature = "musli-wire")]
pub mod musli_wire {
    use alloc::vec::Vec;

    use ::musli_wire::int::Variable;
    use ::musli_wire::Encoding;
    use ::musli_wire::Error;
    use musli::mode::DefaultMode;
    use musli::{Decode, Encode};

    const ENCODING: Encoding<DefaultMode, Variable, Variable> = Encoding::new();

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
            T: Encode,
        {
            ENCODING.encode(&mut *self.buffer, value)?;
            Ok(self.buffer.as_slice())
        }

        pub fn decode<T>(&self) -> Result<T, Error>
        where
            T: Decode<'buf>,
        {
            ENCODING.from_slice(self.buffer)
        }
    }
}

#[cfg(feature = "musli-descriptive")]
pub mod musli_descriptive {
    use alloc::vec::Vec;

    use ::musli_descriptive::Encoding;
    use musli::mode::DefaultMode;
    use musli::{Decode, Encode};
    use musli_descriptive::Error;

    const ENCODING: Encoding<DefaultMode> = Encoding::new();

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
            T: Encode,
        {
            ENCODING.encode(&mut *self.buffer, value)?;
            Ok(self.buffer.as_slice())
        }

        pub fn decode<T>(&self) -> Result<T, Error>
        where
            T: Decode<'buf>,
        {
            ENCODING.from_slice(self.buffer)
        }
    }
}

#[cfg(feature = "musli-value")]
pub mod musli_value {
    use ::musli_value::Value;
    use musli::{Decode, Encode};

    benchmarker! {
        'buf {@nolen}

        pub fn buffer() -> () {}

        pub fn reset<T>(&mut self, _: usize, _: &T) {}

        pub fn encode<T>(&mut self, value: &T) -> Result<Value, musli_value::Error>
        where
            T: Encode,
        {
            musli_value::encode(value)
        }

        pub fn decode<T>(&self) -> Result<T, musli_value::Error>
        where
            for<'a> T: Decode<'a>,
        {
            musli_value::decode(&self.buffer)
        }
    }
}

#[cfg(feature = "musli-zerocopy")]
pub mod musli_zerocopy {
    use musli_zerocopy::pointer::{DefaultSize, Ref};
    use musli_zerocopy::{AlignedBuf, Buf, Error, ZeroCopy};

    pub struct Benchmarker {
        buf: AlignedBuf,
    }

    #[inline(always)]
    pub fn new() -> Benchmarker {
        Benchmarker {
            buf: AlignedBuf::with_capacity(4096),
        }
    }

    impl Benchmarker {
        #[inline(always)]
        pub fn with<T, O>(&mut self, with: T) -> O
        where
            T: FnOnce(State) -> O,
        {
            with(State { buf: &mut self.buf })
        }
    }

    pub struct State<'buf> {
        buf: &'buf mut AlignedBuf,
    }

    pub struct DecodeState<'buf> {
        buf: &'buf Buf,
        offset: usize,
    }

    impl<'buf> DecodeState<'buf> {
        #[inline(always)]
        pub fn len(&self) -> usize {
            self.buf.len()
        }

        #[inline(always)]
        pub fn decode<T>(&self) -> Result<&'buf T, Error>
        where
            T: ZeroCopy,
        {
            self.buf.load(Ref::<T, DefaultSize>::new(self.offset))
        }
    }

    impl<'buf> State<'buf> {
        #[inline(always)]
        pub fn reset<T>(&mut self, reserve: usize, _: &T) {
            self.buf.clear();
            self.buf.reserve(reserve);
        }

        #[inline(always)]
        pub fn encode<T>(&mut self, value: &T) -> Result<DecodeState<'_>, Error>
        where
            T: ZeroCopy,
        {
            let offset = self.buf.store(value)?.offset();

            Ok(DecodeState {
                buf: self.buf.as_ref(),
                offset,
            })
        }
    }

    #[inline(always)]
    pub fn decode<'de, T>(bytes: &'de [u8]) -> Result<&'de T, Error>
    where
        T: ZeroCopy,
    {
        Buf::new(bytes).load(Ref::<T>::zero())
    }
}
