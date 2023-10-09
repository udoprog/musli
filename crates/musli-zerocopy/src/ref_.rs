use core::marker::PhantomData;

use crate::ptr::Ptr;

/// A sized reference.
#[repr(C)]
pub struct Ref<T> {
    ptr: Ptr,
    _marker: PhantomData<T>,
}

impl<T> Ref<T> {
    pub(crate) fn new(ptr: Ptr) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn ptr(&self) -> Ptr {
        self.ptr
    }
}
