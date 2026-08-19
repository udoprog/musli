use core::cell::Cell;
use core::cell::RefCell;
use core::fmt;
use core::mem;
use core::mem::ManuallyDrop;
use core::ops::Range;

use alloc::vec::Vec;
use musli::Encode;
use musli::mode::Binary;

use crate::api::{EncodeBody, Format};
use crate::format;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
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
#[cfg_attr(test, derive(PartialEq))]
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

#[must_use = "Writer must be consumed with Writer::flush to have an effect on the underlying buffer"]
pub(crate) struct Writer<'a> {
    start: usize,
    buf: &'a mut Buf,
}

impl Writer<'_> {
    /// Write the fixed envelope of a message to the current frame.
    ///
    /// The envelope never depends on the negotiated format, see the [wire
    /// format].
    ///
    /// [wire format]: crate::api#wire-format
    #[inline]
    pub(crate) fn envelope<T>(&mut self, value: &T) -> Result<(), format::Error>
    where
        T: ?Sized + Encode<Binary>,
    {
        format::encode_envelope(&mut self.buf.buffer, value)
    }

    /// Write the body of a message to the current frame using `format`.
    #[inline]
    pub(crate) fn body<T>(&mut self, format: Format, value: &T) -> Result<(), format::Error>
    where
        T: ?Sized + EncodeBody,
    {
        format.encode(&mut self.buf.buffer, value)
    }

    /// Finalize the current frame.
    #[inline]
    pub(crate) fn flush(self) {
        let mut this = ManuallyDrop::new(self);
        let start = this.start;
        this.buf.done(start);
    }
}

impl Drop for Writer<'_> {
    #[inline]
    fn drop(&mut self) {
        self.buf.reset(self.start);
    }
}

/// A length-prefixed buffer which keeps track of the start of each frame and
/// allows them to be iterated over.
#[derive(Default)]
pub(crate) struct Buf {
    buffer: Vec<u8>,
    read: Cell<usize>,
}

impl Buf {
    /// Start a write.
    pub(crate) fn writer(&mut self) -> Writer<'_> {
        if self.read.get() == self.buffer.len() {
            self.buffer.clear();
            self.read.set(0);
        }

        let start = self.buffer.len();
        self.buffer.extend_from_slice(&[0; mem::size_of::<u32>()]);
        Writer { start, buf: self }
    }

    #[inline]
    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        // NB: Read should never exceed the length of the buffer.
        debug_assert!(self.read.get() <= self.buffer.len());
        self.read.get() >= self.buffer.len()
    }

    fn len_at_mut(&mut self, at: usize) -> Option<&mut [u8; 4]> {
        let bytes = self.buffer.get_mut(at..at + mem::size_of::<u32>())?;
        Some(unsafe { &mut *bytes.as_mut_ptr().cast() })
    }

    /// Mark an outgoing frame as done from the previous start point.
    ///
    /// If no start point is recorded, calling this method does nothing.
    #[inline]
    fn done(&mut self, start: usize) {
        let delta = self
            .buffer
            .len()
            .saturating_sub(start)
            .saturating_sub(mem::size_of::<u32>());

        let l = u32::try_from(delta).unwrap_or(u32::MAX).to_le_bytes();

        let Some(len) = self.len_at_mut(start) else {
            return;
        };

        *len = l;
    }

    /// Reset the buffer to the previous start point.
    ///
    /// If no start point is set, this method does nothing.
    #[inline]
    fn reset(&mut self, start: usize) {
        self.buffer.truncate(start);
    }

    #[inline]
    pub(crate) fn clear(&mut self) {
        self.buffer.clear();
        self.read.set(0);
    }

    /// Release any allocation beyond `capacity`.
    ///
    /// A single large message would otherwise pin its allocation for as long as
    /// the buffer is pooled.
    #[inline]
    pub(crate) fn shrink_to(&mut self, capacity: usize) {
        self.buffer.shrink_to(capacity);
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

pub(crate) struct BufPool {
    pool: RefCell<Vec<Buf>>,
    /// Buffers are shrunk back down to this when they are returned, so a single
    /// large message does not pin its allocation for the lifetime of the
    /// connection.
    max_capacity: usize,
}

impl BufPool {
    /// Construct a pool which shrinks returned buffers to `max_capacity`.
    #[inline]
    pub(crate) fn new(max_capacity: usize) -> Self {
        Self {
            pool: RefCell::new(Vec::new()),
            max_capacity,
        }
    }

    /// Try to run the given closure with a pool from the buffer.
    ///
    /// If the closure errors, the pool is returned.
    #[inline]
    pub(crate) fn with<E>(&self, f: impl FnOnce(&mut Buf) -> Result<(), E>) -> Result<Buf, E> {
        let mut buf = self.get();
        let result = f(&mut buf);

        match result {
            Ok(()) => Ok(buf),
            Err(err) => {
                self.put(buf);
                Err(err)
            }
        }
    }

    #[inline]
    pub(crate) fn get(&self) -> Buf {
        self.pool.borrow_mut().pop().unwrap_or_default()
    }

    #[inline]
    pub(crate) fn put(&self, mut buf: Buf) {
        buf.clear();
        buf.shrink_to(self.max_capacity);
        self.pool.borrow_mut().push(buf);
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::{String, ToString};

    use musli::Encode;

    use super::{Buf, BufPool};
    use crate::api::Format;

    /// A pooled buffer which grew past the pool's capacity must give the
    /// allocation back when it is returned, or one large message would pin it
    /// for the lifetime of the connection.
    #[test]
    fn test_pool_shrinks_returned_buffers() {
        let pool = BufPool::new(16);

        let mut buf = pool.get();
        buf.buffer.extend_from_slice(&[0; 1024]);
        assert!(buf.buffer.capacity() >= 1024);

        pool.put(buf);

        let buf = pool.get();
        assert!(buf.buffer.is_empty());

        assert!(
            buf.buffer.capacity() <= 16,
            "Expected the allocation to be released, got {}",
            buf.buffer.capacity()
        );
    }

    #[test]
    fn test_empty_buf() {
        let buf = Buf::default();
        assert!(buf.is_empty());
        assert_eq!(buf.read(), Ok(None));
    }

    #[derive(Encode, musli::Decode)]
    struct Message {
        a: u32,
        b: String,
    }

    #[test]
    fn test_two_elements() {
        let mut buf = Buf::default();

        assert!(buf.is_empty());
        assert_eq!(buf.read(), Ok(None));

        // Buffer not consumed, so should leave empty.
        buf.writer()
            .body(
                Format::DEFAULT,
                &Message {
                    a: 42,
                    b: "hello".to_string(),
                },
            )
            .unwrap();

        assert!(buf.is_empty());
        assert_eq!(buf.read(), Ok(None));

        // Buffer consumed, so should be available for reading.
        let mut writer = buf.writer();
        writer
            .body(
                Format::DEFAULT,
                &Message {
                    a: 42,
                    b: "hello".to_string(),
                },
            )
            .unwrap();

        writer.flush();

        assert!(!buf.is_empty());
        assert!(matches!(buf.read(), Ok(Some(..))));

        assert!(buf.is_empty());
        assert_eq!(buf.read(), Ok(None));
    }
}
