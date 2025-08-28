use core::cell::Cell;
use core::fmt;
use core::mem;
use core::num::NonZeroUsize;
use core::ops::Range;

use alloc::vec::Vec;
use musli::mode::Binary;
use musli::{Encode, storage};

#[derive(Debug)]
pub(crate) struct InvalidFrame {
    frame: Range<usize>,
    size: usize,
}

impl fmt::Display for InvalidFrame {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid frame {}-{} (in {} bytes)",
            self.frame.start, self.frame.end, self.size
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
            let bytes = 0u32.to_ne_bytes();
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
                .to_ne_bytes();

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
                frame: 0..read,
                size: self.buffer.len(),
            });
        };

        let Some((head, tail)) = tail.split_at_checked(mem::size_of::<u32>()) else {
            return Err(InvalidFrame {
                frame: 0..read,
                size: self.buffer.len(),
            });
        };

        let frame = read..read + mem::size_of::<u32>();

        let &[a, b, c, d] = head else {
            return Err(InvalidFrame {
                frame: frame.clone(),
                size: self.buffer.len(),
            });
        };

        let Ok(len) = usize::try_from(u32::from_ne_bytes([a, b, c, d])) else {
            return Err(InvalidFrame {
                frame: frame.clone(),
                size: self.buffer.len(),
            });
        };

        let Some(out) = tail.get(..len) else {
            return Err(InvalidFrame {
                frame: frame.start..frame.end + len,
                size: self.buffer.len(),
            });
        };

        self.read.set(
            read.saturating_add(mem::size_of::<u32>())
                .saturating_add(len),
        );
        Ok(Some(out))
    }
}
