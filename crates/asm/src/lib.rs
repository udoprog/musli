#[cfg(feature = "musli-zerocopy")]
pub mod musli_zerocopy_store {
    use musli_zerocopy::endian;
    use musli_zerocopy::OwnedBuf;
    use tests::models::*;

    macro_rules! build {
        ($id:ident, $ty:ty, $constant:ident, $number:literal) => {
            tests::if_supported! {
                musli_zerocopy, $id,
                #[inline(never)]
                pub fn $id(buf: &mut OwnedBuf<endian::Native, usize>, primitives: &$ty) {
                    buf.store(primitives);
                }
            }
        };
    }

    tests::types!(build);
}

#[cfg(feature = "musli-zerocopy")]
pub mod musli_zerocopy_store_unchecked {
    use musli_zerocopy::endian;
    use musli_zerocopy::OwnedBuf;
    use tests::models::*;

    macro_rules! build {
        ($id:ident, $ty:ty, $constant:ident, $number:literal) => {
            tests::if_supported! {
                musli_zerocopy, $id,
                #[inline(never)]
                pub fn $id(buf: &mut OwnedBuf<endian::Native, usize>, primitives: &$ty) {
                    unsafe {
                        buf.store_unchecked(primitives);
                    }
                }
            }
        };
    }

    tests::types!(build);
}

#[cfg(feature = "musli-zerocopy")]
pub mod musli_zerocopy_load {
    use musli_zerocopy::{Buf, Error, Ref};
    use tests::models::*;

    macro_rules! build {
        ($id:ident, $ty:ty, $constant:ident, $number:literal) => {
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
        };
    }

    tests::types!(build);
}

#[cfg(feature = "zerocopy")]
pub mod zerocopy_load {
    use tests::models::*;
    use zerocopy::FromBytes;

    macro_rules! build {
        ($id:ident, $ty:ty, $constant:ident, $number:literal) => {
            tests::if_supported! {
                zerocopy, $id,
                #[inline(never)]
                pub fn $id(buf: &[u8]) -> Option<$ty> {
                    <$ty>::read_from(buf)
                }
            }
        };
    }

    tests::types!(build);
}

#[cfg(feature = "zerocopy")]
pub mod zerocopy_store {
    use core::mem::size_of;
    use tests::models::*;
    use zerocopy::AsBytes;

    macro_rules! build {
        ($id:ident, $ty:ty, $constant:ident, $number:literal) => {
            tests::if_supported! {
                zerocopy, $id,
                #[inline(never)]
                pub fn $id<'buf>(out: &'buf mut [u8; size_of::<$ty>()], value: &$ty) -> &'buf [u8] {
                    out.copy_from_slice(value.as_bytes());
                    out
                }
            }
        };
    }

    tests::types!(build);
}

#[cfg(feature = "musli-zerocopy")]
pub mod musli_zerocopy_swap {
    use musli_zerocopy::endian;
    use musli_zerocopy::{Ref, ZeroCopy};

    #[inline(never)]
    pub fn array_le(value: [u32; 8]) -> [u32; 8] {
        value.swap_bytes::<endian::Little>()
    }

    #[inline(never)]
    pub fn array_be(value: [u32; 8]) -> [u32; 8] {
        value.swap_bytes::<endian::Big>()
    }

    #[inline(never)]
    pub fn reference_noop(value: Ref<u32>) -> Ref<u32> {
        value.swap_bytes::<endian::Big>()
    }

    #[derive(ZeroCopy)]
    #[repr(C)]
    pub struct Struct {
        bits32: u32,
        bits64: u64,
        bits128: u128,
        array: [u8; 16],
    }

    #[inline(never)]
    pub fn ensure_struct_ne(st: Struct) -> Struct {
        st.swap_bytes::<endian::Native>()
    }

    #[inline(never)]
    pub fn ensure_struct_le(st: Struct) -> Struct {
        st.swap_bytes::<endian::Little>()
    }

    #[inline(never)]
    pub fn ensure_struct_be(st: Struct) -> Struct {
        st.swap_bytes::<endian::Big>()
    }

    #[derive(ZeroCopy)]
    #[repr(u32)]
    pub enum Enum {
        Variant1,
        Variant2 { bits32: u32, bits64: u64 },
    }

    #[inline(never)]
    pub fn ensure_enum_ne(st: Enum) -> Enum {
        st.swap_bytes::<endian::Native>()
    }

    #[inline(never)]
    pub fn ensure_enum_le(st: Enum) -> Enum {
        st.swap_bytes::<endian::Little>()
    }

    #[inline(never)]
    pub fn ensure_enum_be(st: Enum) -> Enum {
        st.swap_bytes::<endian::Big>()
    }
}

pub mod generic {
    pub enum Enum {
        Empty,
        One(u32),
        Two(u32, u64),
    }

    pub fn do_nothing(en: Enum) -> Enum {
        en
    }
}
