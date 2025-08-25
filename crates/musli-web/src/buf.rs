use core::ops::Range;

use alloc::vec::Vec;
use musli::mode::Binary;
use musli::{Encode, storage};

#[derive(Default)]
pub(crate) struct Buf {
    start: Option<usize>,
    buffer: Vec<u8>,
    frames: Vec<Range<usize>>,
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
            self.start = Some(self.buffer.len());
        }

        storage::to_writer(&mut self.buffer, &value)?;
        Ok(())
    }

    /// Check if the buffer is empty.
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Mark an outgoing frame as done from the previous start point.
    ///
    /// If no start point is recorded, calling this method does nothing.
    #[inline]
    pub(crate) fn done(&mut self) {
        if let Some(start) = self.start.take() {
            self.frames.push(start..self.buffer.len());
        }
    }

    /// Reset the buffer to the previous start point.
    ///
    /// If no start point is set, this method does nothing.
    #[inline]
    pub(crate) fn reset(&mut self) {
        if let Some(start) = self.start {
            self.buffer.truncate(start);
        }
    }

    #[inline]
    pub(crate) fn clear(&mut self) {
        self.start = None;
        self.buffer.clear();
        self.frames.clear();
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
    pub(crate) fn frames(
        &mut self,
    ) -> impl Iterator<Item = Result<&[u8], (Range<usize>, usize)>> + '_ {
        let buffer = &self.buffer;

        self.frames.drain(..).map(move |range| {
            let Some(slice) = buffer.get(range.clone()) else {
                return Err((range, buffer.len()));
            };

            Ok(slice)
        })
    }
}
