//! Fixed capacity containers.

// Parts borrowed under the MIT license from
// https://github.com/bluss/arrayvec/tree/2c92a59bed0d1669cede3806000d2e61d5994c4e

use core::fmt;
use core::mem::{self, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::slice;
use core::str;

/// An error raised when we are at capacity.
#[non_exhaustive]
pub struct CapacityError;

/// A fixed capacity vector allocated on the stack.
pub struct FixedVec<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> FixedVec<T, N> {
    /// Construct a new empty fixed vector.
    pub const fn new() -> FixedVec<T, N> {
        unsafe {
            FixedVec {
                data: MaybeUninit::uninit().assume_init(),
                len: 0,
            }
        }
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const T {
        self.data.as_ptr() as *const T
    }

    #[inline]
    pub(crate) fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as *mut T
    }

    #[inline]
    pub(crate) fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
    }

    #[inline]
    pub(crate) fn as_uninit_slice(&self) -> &[MaybeUninit<T>] {
        unsafe { slice::from_raw_parts(self.data.as_ptr(), N) }
    }

    #[inline]
    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }

    #[inline]
    pub(crate) fn as_mut_uninit_slice(&mut self) -> &mut [MaybeUninit<T>] {
        unsafe { slice::from_raw_parts_mut(self.data.as_mut_ptr(), N) }
    }

    pub(crate) fn try_extend_from_slice(&mut self, other: &[T]) -> Result<(), CapacityError>
    where
        T: Copy,
    {
        if self.len + other.len() > N {
            return Err(CapacityError);
        }

        let self_len = self.len;
        let other_len = other.len();

        unsafe {
            let dst = self.as_mut_ptr().wrapping_add(self_len);
            ptr::copy_nonoverlapping(other.as_ptr(), dst, other_len);
            self.len = self_len + other_len;
        }

        Ok(())
    }

    /// Try to push an element onto the fixed vector.
    pub fn try_push(&mut self, element: T) -> Result<(), CapacityError> {
        if self.len >= N {
            return Err(CapacityError);
        }

        unsafe {
            ptr::write(self.as_mut_ptr().wrapping_add(self.len), element);
            self.len += 1;
        }

        Ok(())
    }

    /// Pop the last element in the fixed vector.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        unsafe {
            let new_len = self.len - 1;
            self.len = new_len;
            Some(ptr::read(self.as_ptr().wrapping_add(new_len)))
        }
    }

    pub(crate) fn clear(&mut self) {
        if self.len == 0 {
            return;
        }

        let len = mem::take(&mut self.len);

        if mem::needs_drop::<T>() {
            unsafe {
                let tail = slice::from_raw_parts_mut(self.as_mut_ptr(), len);
                ptr::drop_in_place(tail);
            }
        }
    }
}

impl<T, const N: usize> Deref for FixedVec<T, N> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for FixedVec<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, const N: usize> Drop for FixedVec<T, N> {
    #[inline]
    fn drop(&mut self) {
        self.clear()
    }
}

/// A fixed capacity string.
pub struct FixedString<const N: usize> {
    data: FixedVec<u8, N>,
}

impl<const N: usize> FixedString<N> {
    /// Construct a new fixed string.
    pub const fn new() -> FixedString<N> {
        FixedString {
            data: FixedVec::new(),
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        // SAFETY: Interactions ensure that data is valid utf-8.
        unsafe { str::from_utf8_unchecked(self.data.as_slice()) }
    }

    pub(crate) fn try_push(&mut self, c: char) -> Result<(), CapacityError> {
        let len = self.data.len;

        unsafe {
            let ptr = self.data.as_mut_ptr().wrapping_add(len);
            let remaining_cap = N - len;

            let Ok(n) = encode_utf8(c, ptr, remaining_cap) else {
                return Err(CapacityError);
            };

            self.data.len += n;
            Ok(())
        }
    }

    pub(crate) fn try_push_str(&mut self, s: &str) -> Result<(), CapacityError> {
        self.data.try_extend_from_slice(s.as_bytes())
    }
}

impl<const N: usize> fmt::Write for FixedString<N> {
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.try_push(c).map_err(|_| fmt::Error)
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.try_push_str(s).map_err(|_| fmt::Error)
    }
}

impl<const N: usize> Deref for FixedString<N> {
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.data.as_slice()) }
    }
}

impl<const N: usize> DerefMut for FixedString<N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut str {
        unsafe { str::from_utf8_unchecked_mut(self.data.as_mut_slice()) }
    }
}

impl<const N: usize> fmt::Display for FixedString<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<const N: usize> AsRef<str> for FixedString<N> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

const TAG_CONT: u8 = 0b1000_0000;
const TAG_TWO_B: u8 = 0b1100_0000;
const TAG_THREE_B: u8 = 0b1110_0000;
const TAG_FOUR_B: u8 = 0b1111_0000;
const MAX_ONE_B: u32 = 0x80;
const MAX_TWO_B: u32 = 0x800;
const MAX_THREE_B: u32 = 0x10000;

pub(crate) struct EncodeUtf8Error;

/// Encode a char into buf using UTF-8.
///
/// On success, return the byte length of the encoding (1, 2, 3 or 4).<br>
/// On error, return `EncodeUtf8Error` if the buffer was too short for the char.
///
/// Safety: `ptr` must be writable for `len` bytes.
#[inline]
pub(crate) unsafe fn encode_utf8(
    ch: char,
    ptr: *mut u8,
    len: usize,
) -> Result<usize, EncodeUtf8Error> {
    let code = ch as u32;
    if code < MAX_ONE_B && len >= 1 {
        ptr.wrapping_add(0).write(code as u8);
        return Ok(1);
    } else if code < MAX_TWO_B && len >= 2 {
        ptr.wrapping_add(0)
            .write((code >> 6 & 0x1F) as u8 | TAG_TWO_B);
        ptr.wrapping_add(1).write((code & 0x3F) as u8 | TAG_CONT);
        return Ok(2);
    } else if code < MAX_THREE_B && len >= 3 {
        ptr.wrapping_add(0)
            .write((code >> 12 & 0x0F) as u8 | TAG_THREE_B);
        ptr.wrapping_add(1)
            .write((code >> 6 & 0x3F) as u8 | TAG_CONT);
        ptr.wrapping_add(2).write((code & 0x3F) as u8 | TAG_CONT);
        return Ok(3);
    } else if len >= 4 {
        ptr.wrapping_add(0)
            .write((code >> 18 & 0x07) as u8 | TAG_FOUR_B);
        ptr.wrapping_add(1)
            .write((code >> 12 & 0x3F) as u8 | TAG_CONT);
        ptr.wrapping_add(2)
            .write((code >> 6 & 0x3F) as u8 | TAG_CONT);
        ptr.wrapping_add(3).write((code & 0x3F) as u8 | TAG_CONT);
        return Ok(4);
    };

    Err(EncodeUtf8Error)
}
