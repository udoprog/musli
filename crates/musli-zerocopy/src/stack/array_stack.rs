use core::mem::MaybeUninit;
use core::ptr;
use core::slice;

use super::Stack;

pub(crate) struct ArrayStack<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> ArrayStack<T, N> {
    const fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn clear(&mut self) {
        self.truncate(0);
    }

    pub(crate) fn try_push(&mut self, value: T) -> bool {
        if self.len == N {
            return false;
        }

        unsafe {
            self.as_mut_ptr().add(self.len).write(value);
            self.len += 1;
        }

        true
    }

    pub(crate) fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        unsafe {
            let new_len = self.len - 1;
            self.len = new_len;
            Some(ptr::read(self.as_ptr().add(new_len)))
        }
    }

    fn truncate(&mut self, new_len: usize) {
        // SAFETY: `len` defines the initialized length of the array vector.
        unsafe {
            let len = self.len();

            if new_len < len {
                self.len = new_len;
                let tail = slice::from_raw_parts_mut(self.as_mut_ptr().add(new_len), len - new_len);
                ptr::drop_in_place(tail);
            }
        }
    }

    fn as_ptr(&self) -> *const T {
        self.data.as_ptr() as _
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as _
    }
}

impl<T, const N: usize> Drop for ArrayStack<T, N> {
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, const N: usize> Stack<T> for ArrayStack<T, N> {
    const CAPACITY: usize = N;

    #[inline]
    fn new() -> Self {
        ArrayStack::new()
    }

    #[inline]
    fn try_push(&mut self, value: T) -> bool {
        ArrayStack::try_push(self, value)
    }

    #[inline]
    fn pop(&mut self) -> Option<T> {
        ArrayStack::pop(self)
    }
}
