use core::cell::Cell;
use core::fmt;
use core::mem;
use core::num::NonZeroUsize;
use core::ops::Range;

use alloc::vec::Vec;
use musli::mode::Binary;
use musli::{Encode, storage};

#[derive(Debug)]
enum InvalidFrameWhat {
    ReadPosition(usize),
    LengthPrefix,
    LengthPrefixOverflow(u32),
    InsufficientLength(usize),
    InsufficientFrame(usize),
}

impl fmt::Display for InvalidFrameWhat {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadPosition(pos) => write!(f, "read position ({pos}) out of bounds"),
            Self::LengthPrefix => write!(f, "4 byte length prefix out of bounds"),
            Self::LengthPrefixOverflow(len) => write!(f, "length prefix {len} overflowed usize"),
            Self::InsufficientLength(len) => {
                write!(f, "insufficient data for length (needed {len} bytes)")
            }
            Self::InsufficientFrame(len) => {
                write!(f, "insufficient data for frame (needed {len} bytes)")
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct InvalidFrame {
    what: InvalidFrameWhat,
    range: Range<usize>,
    size: usize,
}

impl fmt::Display for InvalidFrame {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}-{} (has {} bytes)",
            self.what, self.range.start, self.range.end, self.size
        )
    }
}

/// A length-prefixed buffer which keeps track of the start of each frame and
/// allows them to be iterated over.
#[derive(Default)]
pub(crate) struct Buf {
    start: Option<NonZeroUsize>,
    buffer: Vec<u8>,
    read: Cell<usize>,
}

impl Buf {
    /// Write data to the current frame, or start a new frame if no frame is
    /// being written.
    ///
    /// This needs to be paired with a call to [`Buf::done`] to complete an
    /// outgoing frame.
    ///
    /// If a new frame is started, a new start point is recorded.
    #[inline]
    pub(crate) fn write<T>(&mut self, value: T) -> Result<(), storage::Error>
    where
        T: Encode<Binary>,
    {
        if self.start.is_none() {
            let bytes = 0u32.to_le_bytes();
            self.buffer.extend_from_slice(&bytes);
            self.start = NonZeroUsize::new(self.buffer.len());
        }

        storage::to_writer(&mut self.buffer, &value)?;
        Ok(())
    }

    /// Check if the buffer is empty.
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        // NB: Read should never exceed the length of the buffer.
        debug_assert!(self.read.get() <= self.buffer.len());
        self.read.get() >= self.buffer.len()
    }

    fn len_at_mut(&mut self, at: usize) -> Option<&mut [u8; 4]> {
        match self.buffer.get_mut(at..at + mem::size_of::<u32>())? {
            bytes if bytes.len() == mem::size_of::<u32>() => {
                Some(unsafe { &mut *bytes.as_mut_ptr().cast() })
            }
            _ => None,
        }
    }

    /// Mark an outgoing frame as done from the previous start point.
    ///
    /// If no start point is recorded, calling this method does nothing.
    #[inline]
    pub(crate) fn done(&mut self) {
        if let Some(start) = self.start.take() {
            let l = u32::try_from(self.buffer.len().saturating_sub(start.get()))
                .unwrap_or(u32::MAX)
                .to_le_bytes();

            let Some(len) = self.len_at_mut(start.get().saturating_sub(mem::size_of::<u32>()))
            else {
                return;
            };

            *len = l;
        }
    }

    /// Reset the buffer to the previous start point.
    ///
    /// If no start point is set, this method does nothing.
    #[inline]
    pub(crate) fn reset(&mut self) {
        if let Some(start) = self.start {
            self.buffer.truncate(start.get());
        }
    }

    #[inline]
    pub(crate) fn clear(&mut self) {
        self.start = None;
        self.buffer.clear();
        self.read.set(0);
    }

    /// Get the next frame starting at the given location.
    #[inline]
    pub(crate) fn read(&self) -> Result<Option<&[u8]>, InvalidFrame> {
        let read = self.read.get();

        if self.buffer.len() == read {
            return Ok(None);
        }

        let Some(tail) = self.buffer.get(read..) else {
            return Err(InvalidFrame {
                what: InvalidFrameWhat::ReadPosition(read),
                range: 0..read,
                size: self.buffer.len(),
            });
        };

        let Some((head, tail)) = tail.split_at_checked(mem::size_of::<u32>()) else {
            return Err(InvalidFrame {
                what: InvalidFrameWhat::InsufficientLength(mem::size_of::<u32>()),
                range: 0..read,
                size: self.buffer.len(),
            });
        };

        let frame = read..read + mem::size_of::<u32>();

        let &[a, b, c, d] = head else {
            return Err(InvalidFrame {
                what: InvalidFrameWhat::LengthPrefix,
                range: frame.clone(),
                size: self.buffer.len(),
            });
        };

        let len = u32::from_le_bytes([a, b, c, d]);

        let Ok(len) = usize::try_from(len) else {
            return Err(InvalidFrame {
                what: InvalidFrameWhat::LengthPrefixOverflow(len),
                range: frame.clone(),
                size: self.buffer.len(),
            });
        };

        let Some(out) = tail.get(..len) else {
            return Err(InvalidFrame {
                what: InvalidFrameWhat::InsufficientFrame(len),
                range: frame.start..frame.end + len,
                size: self.buffer.len(),
            });
        };

        let next = read
            .saturating_add(mem::size_of::<u32>())
            .saturating_add(len);

        self.read.set(next);
        Ok(Some(out))
    }
}
