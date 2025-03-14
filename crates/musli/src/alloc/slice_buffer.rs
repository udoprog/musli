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
    impl Sealed for [u8] {}
    impl Sealed for [u16] {}
    impl Sealed for [u32] {}
    impl Sealed for [u64] {}
    impl Sealed for [u128] {}
    impl<const N: usize> Sealed for [u8; N] {}
    impl<const N: usize> Sealed for [u16; N] {}
    impl<const N: usize> Sealed for [u32; N] {}
    impl<const N: usize> Sealed for [u64; N] {}
    impl<const N: usize> Sealed for [u128; N] {}
    impl<const N: usize> Sealed for ArrayBuffer<N> {}
}

/// The trait over anything that can be treated as a buffer by the [`Slice`]
/// allocator.
///
/// [`Slice`]: super::Slice
pub trait SliceBuffer: self::sealed::Sealed {
    #[doc(hidden)]
    fn as_uninit_bytes(&mut self) -> &mut [MaybeUninit<u8>];
}

/// The [`SliceBuffer`] implementation for `[u8]`.
///
/// # Examples
///
/// ```rust
/// use musli::alloc::Slice;
///
/// let mut bytes = [0u8; 128];
/// let alloc = Slice::new(&mut bytes[..]);
/// ```
impl SliceBuffer for [u8] {
    #[inline]
    fn as_uninit_bytes(&mut self) -> &mut [MaybeUninit<u8>] {
        // SAFEYT: &mut [u8] has the same layout as &mut [MaybeUninit<u8>]
        unsafe { from_raw_parts_mut(self.as_mut_ptr().cast(), self.len()) }
    }
}

/// The [`SliceBuffer`] implementation for `[u8; N]`.
///
/// # Examples
///
/// ```rust
/// use musli::alloc::Slice;
///
/// let mut bytes = [0u8; 128];
/// let alloc = Slice::new(&mut bytes);
/// ```
impl<const N: usize> SliceBuffer for [u8; N] {
    #[inline]
    fn as_uninit_bytes(&mut self) -> &mut [MaybeUninit<u8>] {
        self.as_mut_slice().as_uninit_bytes()
    }
}

/// The [`SliceBuffer`] implementation for `[MaybeUninit<u8>]`.
///
/// # Examples
///
/// ```rust
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
/// ```rust
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
/// ```rust
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
            #[doc = concat!(" The [`SliceBuffer`] implementation for `[", stringify!($ty), "]`.")]
            ///
            /// # Examples
            ///
            /// ```rust
            /// use musli::alloc::Slice;
            /// # use musli::alloc::SliceBuffer as _;
            ///
            #[doc = concat!(" let mut bytes = [0", stringify!($ty), "; 128];")]
            #[doc = concat!(" # assert_eq!(bytes.as_uninit_bytes().len(), ", stringify!($len), ");")]
            /// let alloc = Slice::new(&mut bytes[..]);
            /// ```
            impl SliceBuffer for [$ty] {
                #[inline]
                fn as_uninit_bytes(&mut self) -> &mut [MaybeUninit<u8>] {
                    // SAFEYT: &mut [u8] has the same layout as &mut [MaybeUninit<u8>]
                    unsafe {
                        let len = <[_]>::len(self) * (<$ty>::BITS / 8u32) as usize;
                        from_raw_parts_mut(self.as_mut_ptr().cast(), len)
                    }
                }
            }

            #[doc = concat!(" The [`SliceBuffer`] implementation for `[MaybeUninit<", stringify!($ty), ">]`.")]
            ///
            /// # Examples
            ///
            /// ```rust
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
                    // SAFEYT: &mut [u8] has the same layout as &mut [MaybeUninit<u8>]
                    unsafe {
                        let len = <[_]>::len(self) * (<$ty>::BITS / 8u32) as usize;
                        from_raw_parts_mut(self.as_mut_ptr().cast(), len)
                    }
                }
            }

            #[doc = concat!(" The [`SliceBuffer`] implementation for `[", stringify!($ty), "]`.")]
            ///
            /// # Examples
            ///
            /// ```rust
            /// use core::mem::MaybeUninit;
            ///
            /// use musli::alloc::Slice;
            /// # use musli::alloc::SliceBuffer as _;
            ///
            #[doc = concat!(" let mut bytes = [0", stringify!($ty), "; 128];")]
            #[doc = concat!(" # assert_eq!(bytes.as_uninit_bytes().len(), ", stringify!($len), ");")]
            /// let alloc = Slice::new(&mut bytes);
            /// ```
            impl<const N: usize> SliceBuffer for [$ty; N] {
                #[inline]
                fn as_uninit_bytes(&mut self) -> &mut [MaybeUninit<u8>] {
                    self.as_mut_slice().as_uninit_bytes()
                }
            }

            #[doc = concat!(" The [`SliceBuffer`] implementation for `[MaybeUninit<", stringify!($ty), ">; N]`.")]
            ///
            /// # Examples
            ///
            /// ```rust
            /// use musli::alloc::Slice;
            /// # use musli::alloc::SliceBuffer as _;
            ///
            #[doc = concat!(" let mut bytes = [0", stringify!($ty), "; 128];")]
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
