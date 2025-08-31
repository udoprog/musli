use musli_zerocopy_macros::ZeroCopy;

use crate::{
    Ref,
    endian::{Big, Little, Native},
};

const MAX: usize = isize::MAX as usize;

#[test]
fn test_slice_with_metadata() {
    Ref::<[()], Native, usize>::try_with_metadata(0, usize::MAX).unwrap();

    Ref::<[u8], Native, usize>::try_with_metadata(0, MAX).unwrap();
    Ref::<[u8], Native, usize>::try_with_metadata(0, MAX + 1).unwrap_err();

    Ref::<[u16], Native, usize>::try_with_metadata(0, MAX / 2).unwrap();
    Ref::<[u16], Native, usize>::try_with_metadata(0, (MAX / 2) + 1).unwrap_err();
}

#[test]
fn test_metadata_byte_order() {
    #[derive(ZeroCopy)]
    #[zero_copy(crate)]
    #[repr(C)]
    struct BigStruct([u8; 4096]);

    const BIG_MAX_LEN: usize = MAX / size_of::<BigStruct>();

    Ref::<[BigStruct], Native, usize>::try_with_metadata(0, BIG_MAX_LEN).unwrap();
    Ref::<[BigStruct], Native, usize>::try_with_metadata(0, BIG_MAX_LEN + 1).unwrap_err();

    Ref::<[BigStruct], Little, usize>::try_with_metadata(0, BIG_MAX_LEN).unwrap();
    Ref::<[BigStruct], Little, usize>::try_with_metadata(0, BIG_MAX_LEN + 1).unwrap_err();

    Ref::<[BigStruct], Big, usize>::try_with_metadata(0, BIG_MAX_LEN).unwrap();
    Ref::<[BigStruct], Big, usize>::try_with_metadata(0, BIG_MAX_LEN + 1).unwrap_err();
}
