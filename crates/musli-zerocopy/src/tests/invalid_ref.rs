use crate::{Ref, endian::Native};

const MAX: usize = isize::MAX as usize;

#[test]
fn test_slice_with_metadata() {
    Ref::<[()], Native, usize>::try_with_metadata(0, usize::MAX).unwrap();

    Ref::<[u8], Native, usize>::try_with_metadata(0, MAX).unwrap();
    Ref::<[u8], Native, usize>::try_with_metadata(0, MAX + 1).unwrap_err();

    Ref::<[u16], Native, usize>::try_with_metadata(0, MAX / 2).unwrap();
    Ref::<[u16], Native, usize>::try_with_metadata(0, (MAX / 2) + 1).unwrap_err();
}
