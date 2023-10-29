#[cfg(feature = "serde_json")]
pub mod serde_json {
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

        pub fn encode<T>(&mut self, value: &T) -> Result<&'buf [u8], serde_json::Error>
        where
            T: Serialize,
        {
            serde_json::to_writer(&mut *self.buffer, value)?;
            Ok(self.buffer.as_slice())
        }

        pub fn decode<T>(&self) -> Result<T, serde_json::Error>
        where
            T: Deserialize<'buf>,
        {
            serde_json::from_slice(self.buffer)
        }
    }
}

#[cfg(feature = "bincode")]
pub mod serde_bincode {
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

        pub fn encode<T>(&mut self, value: &T) -> Result<&'buf [u8], bincode::Error>
        where
            T: Serialize,
        {
            bincode::serialize_into(&mut *self.buffer, value)?;
            Ok(self.buffer.as_slice())
        }

        pub fn decode<T>(&self) -> Result<T, bincode::Error>
        where
            T: Deserialize<'buf>,
        {
            bincode::deserialize(self.buffer)
        }
    }
}

#[cfg(feature = "serde_cbor")]
pub mod serde_cbor {
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

        pub fn encode<T>(&mut self, value: &T) -> Result<&'buf [u8], serde_cbor::Error>
        where
            T: Serialize,
        {
            serde_cbor::to_writer(&mut *self.buffer, value)?;
            Ok(self.buffer.as_slice())
        }

        pub fn decode<T>(&self) -> Result<T, serde_cbor::Error>
        where
            T: Deserialize<'buf>,
        {
            serde_cbor::from_slice(self.buffer)
        }
    }
}

#[cfg(feature = "postcard")]
pub mod postcard {
    use alloc::vec::Vec;

    use serde::{Deserialize, Serialize};

    benchmarker! {
        'buf

        pub fn buffer() -> Vec<u8> {
            Vec::new()
        }

        pub fn reset<T>(&mut self, size_hint: usize, value: &T)
        where
            T: Serialize,
        {
            if self.buffer.len() < size_hint {
                self.buffer.resize(size_hint, 0);
            }

            // Figure out the size of the buffer to use. Don't worry, anything we do
            // in `reset` doesn't count towards benchmarking.
            while let Err(error) = postcard::to_slice(value, self.buffer) {
                match error {
                    postcard::Error::SerializeBufferFull => {
                        let new_size = (self.buffer.len() as f32 * 1.5f32) as usize;
                        self.buffer.resize(new_size, 0);
                    }
                    error => {
                        panic!("{}", error)
                    }
                }
            }
        }

        pub fn encode<T>(&mut self, value: &T) -> Result<&'buf [u8], postcard::Error>
        where
            T: Serialize,
        {
            let buf = postcard::to_slice(value, self.buffer)?;
            Ok(buf)
        }

        pub fn decode<T>(&self) -> Result<T, postcard::Error>
        where
            T: Deserialize<'buf>,
        {
            postcard::from_bytes(self.buffer)
        }
    }
}

/// Bitcode lives in here with two variants, one using serde and another using
/// its own derives.
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
