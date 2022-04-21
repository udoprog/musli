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
    pub fn as_bytes(&self) -> &[u8] {
        if self.init == 0 {
            return &[];
        }

        // SAFETY: We've asserted that `initialized` accounts for the number of
        // bytes that have been initialized.
        unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const u8, self.init) }
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
    fn custom<T>(message: T) -> Self
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        Self(FixedBytesWriterErrorRepr::collect(message))
    }

    fn collect_from_display<T>(message: T) -> Self
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

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        if bytes.len() > N.saturating_sub(self.init) {
            return Err(E::custom("buffer overflow"));
        }

        unsafe {
            let dst = (self.data.as_mut_ptr() as *mut u8).add(self.init);
            ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len());
        }

        self.init += bytes.len();
        Ok(())
    }
}
