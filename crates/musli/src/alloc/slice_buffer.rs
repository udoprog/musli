use core::mem::MaybeUninit;
use core::slice::from_raw_parts_mut;

use super::ArrayBuffer;

mod sealed {
    use core::mem::MaybeUninit;

    use super::super::ArrayBuffer;

    pub trait Sealed {}
    impl Sealed for [MaybeUninit<u8>] {}
    impl Sealed for [MaybeUninit<u16>] {}
    impl Sealed for [MaybeUninit<u32>] {}
    impl Sealed for [MaybeUninit<u64>] {}
    impl Sealed for [MaybeUninit<u128>] {}
    impl<const N: usize> Sealed for [MaybeUninit<u8>; N] {}
    impl<const N: usize> Sealed for [MaybeUninit<u16>; N] {}
    impl<const N: usize> Sealed for [MaybeUninit<u32>; N] {}
    impl<const N: usize> Sealed for [MaybeUninit<u64>; N] {}
    impl<const N: usize> Sealed for [MaybeUninit<u128>; N] {}
    impl<const N: usize> Sealed for ArrayBuffer<N> {}
}

/// The trait over anything that can be treated as a buffer by the [`Slice`]
/// allocator.
///
/// Only buffers of [`MaybeUninit`] and [`ArrayBuffer`] can be used. The
/// allocator writes arbitrary values into the buffer which might leave bytes
/// uninitialized, such as padding, so it would not be sound to read the buffer
/// back as initialized integers after the allocator has been dropped:
///
/// ```compile_fail,E0277
/// use musli::alloc::{Slice, Vec};
///
/// let mut buf = [0u8; 256];
///
/// {
///     let alloc = Slice::new(&mut buf);
///     let mut values = Vec::new_in(&alloc);
///     values.push((1u8, 2u32))?;
/// }
///
/// // Would read uninitialized padding bytes.
/// let sum = buf.iter().map(|&b| b as u32).sum::<u32>();
/// # Ok::<_, musli::alloc::AllocError>(())
/// ```
///
/// [`Slice`]: super::Slice
pub trait SliceBuffer: self::sealed::Sealed {
    #[doc(hidden)]
    fn as_uninit_bytes(&mut self) -> &mut [MaybeUninit<u8>];
}

/// The [`SliceBuffer`] implementation for `[MaybeUninit<u8>]`.
///
/// # Examples
///
/// ```
/// use core::mem::MaybeUninit;
///
/// use musli::alloc::Slice;
///
/// let mut bytes: [MaybeUninit<u8>; 128] = [const { MaybeUninit::uninit() }; 128];
/// let alloc = Slice::new(&mut bytes[..]);
/// ```
impl SliceBuffer for [MaybeUninit<u8>] {
    #[inline]
    fn as_uninit_bytes(&mut self) -> &mut [MaybeUninit<u8>] {
        self
    }
}

/// The [`SliceBuffer`] implementation for `[MaybeUninit<u8>; N]`.
///
/// # Examples
///
/// ```
/// use core::mem::MaybeUninit;
///
/// # use musli::alloc::SliceBuffer as _;
/// use musli::alloc::Slice;
///
/// let mut bytes: [MaybeUninit<u8>; 128] = [const { MaybeUninit::uninit() }; 128];
/// # assert_eq!(bytes.as_uninit_bytes().len(), 128);
/// let alloc = Slice::new(&mut bytes);
/// ```
impl<const N: usize> SliceBuffer for [MaybeUninit<u8>; N] {
    #[inline]
    fn as_uninit_bytes(&mut self) -> &mut [MaybeUninit<u8>] {
        self
    }
}

/// The [`SliceBuffer`] implementation for `ArrayBuffer<N>`.
///
/// # Examples
///
/// ```
/// use core::mem::MaybeUninit;
///
/// use musli::alloc::{ArrayBuffer, Slice};
///
/// let mut buffer = ArrayBuffer::new();
/// let alloc = Slice::new(&mut buffer);
/// ```
impl<const N: usize> SliceBuffer for ArrayBuffer<N> {
    #[inline]
    fn as_uninit_bytes(&mut self) -> &mut [MaybeUninit<u8>] {
        self
    }
}

macro_rules! primitive {
    ($($ty:ty, $len:expr),* $(,)?) => {
        $(
            #[doc = concat!(" The [`SliceBuffer`] implementation for `[MaybeUninit<", stringify!($ty), ">]`.")]
            ///
            /// # Examples
            ///
            /// ```
            /// use core::mem::MaybeUninit;
            ///
            /// use musli::alloc::Slice;
            /// # use musli::alloc::SliceBuffer as _;
            ///
            #[doc = concat!(" let mut bytes: [MaybeUninit<", stringify!($ty), ">; 128] = [const { MaybeUninit::uninit() }; 128];")]
            #[doc = concat!(" # assert_eq!(bytes.as_uninit_bytes().len(), ", stringify!($len), ");")]
            /// let alloc = Slice::new(&mut bytes[..]);
            /// ```
            impl SliceBuffer for [MaybeUninit<$ty>] {
                #[inline]
                fn as_uninit_bytes(&mut self) -> &mut [MaybeUninit<u8>] {
                    // SAFETY: An integer is made up of `BITS / 8` bytes without
                    // padding, so the region can be viewed as uninitialized bytes.
                    unsafe {
                        let len = <[_]>::len(self) * (<$ty>::BITS / 8u32) as usize;
                        from_raw_parts_mut(self.as_mut_ptr().cast(), len)
                    }
                }
            }

            #[doc = concat!(" The [`SliceBuffer`] implementation for `[MaybeUninit<", stringify!($ty), ">; N]`.")]
            ///
            /// # Examples
            ///
            /// ```
            /// use core::mem::MaybeUninit;
            ///
            /// use musli::alloc::Slice;
            /// # use musli::alloc::SliceBuffer as _;
            ///
            #[doc = concat!(" let mut bytes: [MaybeUninit<", stringify!($ty), ">; 128] = [const { MaybeUninit::uninit() }; 128];")]
            #[doc = concat!(" # assert_eq!(bytes.as_uninit_bytes().len(), ", stringify!($len), ");")]
            /// let alloc = Slice::new(&mut bytes);
            /// ```
            impl<const N: usize> SliceBuffer for [MaybeUninit<$ty>; N] {
                #[inline]
                fn as_uninit_bytes(&mut self) -> &mut [MaybeUninit<u8>] {
                    self.as_mut_slice().as_uninit_bytes()
                }
            }
        )*
    }
}

primitive! {
    u16, 128 * 2,
    u32, 128 * 4,
    u64, 128 * 8,
    u128, 128 * 16,
}
