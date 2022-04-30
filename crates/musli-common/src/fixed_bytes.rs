//! A container which can store up to a fixed number of uninitialized bytes on
//! the stack and read into and from it.

use core::fmt;
use core::marker;
use core::mem::MaybeUninit;
use core::ptr;

use musli::error::Error;

use crate::writer::Writer;

/// A fixed-size bytes storage which keeps track of how much has been initialized.
pub struct FixedBytes<const N: usize, E = FixedBytesWriterError> {
    /// Data storage.
    data: [MaybeUninit<u8>; N],
    /// How many bytes have been initialized.
    init: usize,
    /// Error type to raise when this is used as a `Writer` implementation.
    _marker: marker::PhantomData<E>,
}

impl<const N: usize, E> FixedBytes<N, E> {
    /// Construct a new fixed bytes array storage.
    pub const fn new() -> Self {
        Self {
            // SAFETY: MaybeUnint::uninit_array is not stable.
            data: unsafe { MaybeUninit::<[MaybeUninit<u8>; N]>::uninit().assume_init() },
            init: 0,
            _marker: marker::PhantomData,
        }
    }

    /// Get the length of the collection.
    pub const fn len(&self) -> usize {
        self.init
    }

    /// Test if the current container is empty.
    pub const fn is_empty(&self) -> bool {
        self.init == 0
    }

    /// Clear the [FixedBytes] container.
    pub fn clear(&mut self) {
        self.init = 0;
    }

    /// Get the remaining capacity of the [FixedBytes].
    pub const fn remaining(&self) -> usize {
        N.saturating_sub(self.init)
    }

    /// Coerce into the underlying bytes if all of them have been initialized.
    pub fn into_bytes(self) -> Option<[u8; N]> {
        if self.init == N {
            // SAFETY: All of the bytes in the sequence have been initialized
            // and can be safety transmuted.
            //
            // Method of transmuting comes from the implementation of
            // `MaybeUninit::array_assume_init` which is not yet stable.
            unsafe { Some((&self.data as *const _ as *const [u8; N]).read()) }
        } else {
            None
        }
    }

    /// Coerce into the slice of initialized memory which is present.
    pub fn as_slice(&self) -> &[u8] {
        if self.init == 0 {
            return &[];
        }

        // SAFETY: We've asserted that `initialized` accounts for the number of
        // bytes that have been initialized.
        unsafe { core::slice::from_raw_parts(self.data.as_ptr().cast(), self.init) }
    }

    /// Coerce into the mutable slice of initialized memory which is present.
    pub fn as_mut_slice(&mut self) -> &[u8] {
        if self.init == 0 {
            return &[];
        }

        // SAFETY: We've asserted that `initialized` accounts for the number of
        // bytes that have been initialized.
        unsafe { core::slice::from_raw_parts_mut(self.data.as_mut_ptr().cast(), self.init) }
    }

    /// Try and push a single byte.
    pub fn push(&mut self, value: u8) -> bool {
        if N.saturating_sub(self.init) == 0 {
            return false;
        }

        unsafe {
            self.data
                .as_mut_ptr()
                .cast::<u8>()
                .add(self.init)
                .write(value)
        }

        self.init += 1;
        true
    }

    /// Try and extend from the given slice.
    pub fn extend_from_slice(&mut self, source: &[u8]) -> bool {
        if source.len() > N.saturating_sub(self.init) {
            return false;
        }

        unsafe {
            let dst = (self.data.as_mut_ptr() as *mut u8).add(self.init);
            ptr::copy_nonoverlapping(source.as_ptr(), dst, source.len());
        }

        self.init += source.len();
        true
    }
}

impl<const N: usize, E> Default for FixedBytes<N, E> {
    fn default() -> Self {
        Self::new()
    }
}

decl_message_repr!(FixedBytesWriterErrorRepr, "failed to write to fixed bytes");

/// An error raised while decoding a slice.
#[derive(Debug)]
pub struct FixedBytesWriterError(FixedBytesWriterErrorRepr);

impl fmt::Display for FixedBytesWriterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for FixedBytesWriterError {
    #[inline]
    fn custom<T>(message: T) -> Self
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        Self(FixedBytesWriterErrorRepr::collect(message))
    }

    #[inline]
    fn message<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        Self(FixedBytesWriterErrorRepr::collect(message))
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FixedBytesWriterError {}

impl<const N: usize, E> Writer for FixedBytes<N, E>
where
    E: Error,
{
    type Error = E;
    type Mut<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        if !self.extend_from_slice(bytes) {
            return Err(E::message(format_args! {
                "Overflow when writing {additional} bytes at {at} with capacity {capacity}",
                at = self.init,
                additional = bytes.len(),
                capacity = N,
            }));
        }

        Ok(())
    }
}
