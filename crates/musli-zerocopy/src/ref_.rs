use core::marker::PhantomData;

use crate::ptr::Ptr;
use crate::zero_copy::ZeroCopy;

/// A sized reference.
#[repr(C)]
pub struct Ref<T> {
    ptr: Ptr,
    _marker: PhantomData<T>,
}

impl<T> Ref<T>
where
    T: ZeroCopy,
{
    /// Construct a reference wrapping the given type at the specified address.
    pub fn new(ptr: Ptr) -> Self {
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
