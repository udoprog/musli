use core::mem;
use core::num::NonZeroUsize;
use core::ops::Range;

use alloc::vec::Vec;
use musli::mode::Binary;
use musli::{Encode, storage};

/// A length-prefixed buffer which keeps track of the start of each frame and
/// allows them to be iterated over.
#[derive(Default)]
pub(crate) struct Buf {
    start: Option<NonZeroUsize>,
    buffer: Vec<u8>,
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
        if self.buffer.is_empty() {
            return true;
        }

        matches!(self.len_at(0), None | Some(0))
    }

    fn len_at(&self, at: usize) -> Option<u32> {
        match self.buffer.get(at..at + mem::size_of::<u32>())? {
            &[a, b, c, d] => Some(u32::from_ne_bytes([a, b, c, d])),
            _ => None,
        }
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
    }

    #[inline]
    pub(crate) fn shrink_to(&mut self, size: usize) {
        self.buffer.shrink_to(size);
    }

    /// Drain the current frames from the buffer.
    ///
    /// Note that this does not drain the buffer itself, to do this you must
    /// call [`Buf::clear`].
    #[inline]
    pub(crate) fn frames(&mut self) -> Frames<'_> {
        Frames {
            buffer: self.buffer.as_slice(),
        }
    }
}

/// An iterator over frames.
pub(crate) struct Frames<'a> {
    buffer: &'a [u8],
}

impl<'a> Iterator for Frames<'a> {
    type Item = Result<&'a [u8], (Range<usize>, usize)>;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! split_at {
            ($len:expr) => {{
                let Some((out, tail)) = self.buffer.split_at_checked($len) else {
                    let range = 0..$len;
                    let available = self.buffer.len();
                    self.buffer = &[];
                    return Some(Err((range, available)));
                };

                self.buffer = tail;
                out
            }};
        }

        if self.buffer.is_empty() {
            return None;
        }

        let len = split_at!(mem::size_of::<u32>());

        let len = match len {
            &[a, b, c, d] => usize::try_from(u32::from_ne_bytes([a, b, c, d])).ok()?,
            _ => return None,
        };

        let frame = split_at!(len);
        Some(Ok(frame))
    }
}
