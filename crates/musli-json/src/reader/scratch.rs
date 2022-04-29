#[cfg(not(feature = "std"))]
use musli_common::fixed_bytes::FixedBytes;

/// Provides the necessary scratch buffer used when decoding JSON.
#[doc(hidden)]
pub struct Scratch {
    #[cfg(feature = "std")]
    pub(crate) bytes: Vec<u8>,
    #[cfg(not(feature = "std"))]
    pub(crate) bytes: FixedBytes<128>,
}

impl Scratch {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            bytes: Default::default(),
        }
    }

    #[inline]
    pub fn push(&mut self, value: u8) -> bool {
        #[cfg(feature = "std")]
        {
            self.bytes.push(value);
            true
        }

        #[cfg(not(feature = "std"))]
        {
            self.bytes.push(value)
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    #[inline]
    pub(crate) fn extend_from_slice(&mut self, slice: &[u8]) -> bool {
        #[cfg(feature = "std")]
        {
            self.bytes.extend_from_slice(slice);
            true
        }

        #[cfg(not(feature = "std"))]
        {
            self.bytes.extend_from_slice(slice)
        }
    }

    #[inline]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}
