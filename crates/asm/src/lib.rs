#[cfg(feature = "musli-zerocopy")]
pub mod musli_zerocopy_store {
    use musli_zerocopy::OwnedBuf;
    use tests::models::*;

    macro_rules! build {
        ($($id:ident, $ty:ty, $constant:ident, $number:literal),* $(,)?) => {
            $(
                tests::if_supported! {
                    musli_zerocopy, $id,
                    #[inline(never)]
                    pub fn $id(buf: &mut OwnedBuf<usize>, primitives: &$ty) {
                        buf.store(primitives);
                    }
                }
            )*
        }
    }

    tests::types!(build);
}

#[cfg(feature = "musli-zerocopy")]
pub mod musli_zerocopy_store_unchecked {
    use musli_zerocopy::OwnedBuf;
    use tests::models::*;

    macro_rules! build {
        ($($id:ident, $ty:ty, $constant:ident, $number:literal),* $(,)?) => {
            $(
                tests::if_supported! {
                    musli_zerocopy, $id,
                    #[inline(never)]
                    pub fn $id(buf: &mut OwnedBuf<usize>, primitives: &$ty) {
                        unsafe {
                            buf.store_unchecked(primitives);
                        }
                    }
                }
            )*
        }
    }

    tests::types!(build);
}

#[cfg(feature = "musli-zerocopy")]
pub mod musli_zerocopy_load {
    use musli_zerocopy::{Buf, Error, Ref};
    use tests::models::*;

    macro_rules! build {
        ($($id:ident, $ty:ty, $constant:ident, $number:literal),* $(,)?) => {
            $(
                tests::if_supported! {
                    musli_zerocopy, $id,
                    #[inline(never)]
                    pub fn $id(
                        buf: &Buf,
                        reference: Ref<$ty>,
                    ) -> Result<&$ty, Error> {
                        buf.load(reference)
                    }
                }
            )*
        }
    }

    tests::types!(build);
}

#[cfg(feature = "zerocopy")]
pub mod zerocopy_load {
    use tests::models::*;
    use zerocopy::FromBytes;

    macro_rules! build {
        ($($id:ident, $ty:ty, $constant:ident, $number:literal),* $(,)?) => {
            $(
                tests::if_supported! {
                    zerocopy, $id,
                    #[inline(never)]
                    pub fn $id(buf: &[u8]) -> Option<$ty> {
                        <$ty>::read_from(buf)
                    }
                }
            )*
        }
    }

    tests::types!(build);
}

#[cfg(feature = "zerocopy")]
pub mod zerocopy_store {
    use core::mem::size_of;
    use tests::models::*;
    use zerocopy::AsBytes;

    macro_rules! build {
        ($($id:ident, $ty:ty, $constant:ident, $number:literal),* $(,)?) => {
            $(
                tests::if_supported! {
                    zerocopy, $id,
                    #[inline(never)]
                    pub fn $id<'buf>(out: &'buf mut [u8; size_of::<$ty>()], value: &$ty) -> &'buf [u8] {
                        let bytes = value.as_bytes();
                        out.copy_from_slice(value.as_bytes());
                        out
                    }
                }
            )*
        }
    }

    tests::types!(build);
}
