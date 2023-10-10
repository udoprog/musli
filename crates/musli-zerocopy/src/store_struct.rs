use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ptr;

use crate::buf_mut::BufMut;
use crate::error::{Error, ErrorKind};
use crate::ptr::Ptr;
use crate::r#ref::Ref;
use crate::zero_copy::ZeroCopy;

/// A writer as returned from [AlignedBuf::writer].
#[must_use = "For the writer to have an effect on `AlignedBuf` you must call `StoreStruct::finish`"]
pub struct StoreStruct<B, T> {
    buf: B,
    len: usize,
    _marker: PhantomData<T>,
}

impl<B, T> StoreStruct<B, T>
where
    B: BufMut,
    T: ZeroCopy,
{
    pub(crate) fn new(buf: B, len: usize) -> Self {
        Self {
            buf,
            len,
            _marker: PhantomData,
        }
    }

    /// Pad around the given field with zeros.
    ///
    /// Note that this is necessary to do correctly in order to satisfy the
    /// requirements imposed by [`finish()`].
    ///
    /// [`finish()`]: Self::finish
    pub fn pad<F>(&mut self)
    where
        F: ZeroCopy,
    {
        self.zero_pad_align::<F>();
        self.len = self.len.wrapping_add(size_of::<F>());
    }

    /// Finish writing the current buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that they've called [`pad`] in order for every
    /// field in a struct being serialized. Otherwise we might not have written
    /// the necessary padding, and the [`AlignedBuf`] we're writing to might
    /// contain uninitialized data in the form of uninitialized padding.
    ///
    /// [`pad`]: Self::pad
    pub unsafe fn finish(mut self) -> Result<Ref<T>, Error> {
        self.zero_pad_align::<T>();

        let ptr = Ptr::new(self.buf.len());

        if self.len > self.buf.capacity() {
            return Err(Error::new(ErrorKind::BufferOverflow {
                offset: self.len,
                capacity: self.buf.capacity(),
            }));
        }

        self.buf.set_len(self.len);
        Ok(Ref::new(ptr))
    }

    /// Zero pad around a field with the given type `T`.
    ///
    /// # Safety
    ///
    /// This requires that the non-padding bytes of the given field have been
    /// initialized.
    fn zero_pad_align<F>(&mut self)
    where
        F: ZeroCopy,
    {
        let o = self.len.next_multiple_of(align_of::<F>());

        // zero out padding.
        if o > self.len {
            if o <= self.buf.capacity() {
                let start = self.buf.as_ptr_mut().wrapping_add(self.len);

                unsafe {
                    ptr::write_bytes(start, 0, o - self.len);
                }
            }

            self.len = o;
        }
    }
}
