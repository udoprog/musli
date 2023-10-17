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
