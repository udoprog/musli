use crate::{endian::Native, Ref};

#[test]
fn test_new() {
    Ref::<u8, Native, usize>::new(usize::MAX - 1);
}

#[test]
#[should_panic = "overflow"]
fn test_new_panic() {
    Ref::<u8, Native, usize>::new(usize::MAX);
}

#[test]
fn test_sized_with_metadata() {
    Ref::<u8, Native, usize>::try_with_metadata(usize::MAX, ()).unwrap_err();
    Ref::<u8, Native, usize>::try_with_metadata(usize::MAX - 1, ()).unwrap();

    Ref::<u16, Native, usize>::try_with_metadata(usize::MAX, ()).unwrap_err();
    Ref::<u16, Native, usize>::try_with_metadata(usize::MAX - 1, ()).unwrap_err();
    Ref::<u16, Native, usize>::try_with_metadata(usize::MAX - 2, ()).unwrap();
    Ref::<u16, Native, usize>::try_with_metadata(usize::MAX - 3, ()).unwrap();
}

#[test]
fn test_slice_with_metadata() {
    Ref::<[()], Native, usize>::try_with_metadata(usize::MAX, usize::MAX).unwrap();

    Ref::<[u8], Native, usize>::try_with_metadata(usize::MAX, 0).unwrap();
    Ref::<[u8], Native, usize>::try_with_metadata(usize::MAX - 1, 0).unwrap();
    Ref::<[u8], Native, usize>::try_with_metadata(usize::MAX - 1, 1).unwrap();
    Ref::<[u8], Native, usize>::try_with_metadata(usize::MAX - 1, 2).unwrap_err();
    Ref::<[u8], Native, usize>::try_with_metadata(0, usize::MAX).unwrap();
    Ref::<[u8], Native, usize>::try_with_metadata(1, usize::MAX).unwrap_err();
    Ref::<[u8], Native, usize>::try_with_metadata(1, usize::MAX - 1).unwrap();

    Ref::<[u16], Native, usize>::try_with_metadata(usize::MAX, 0).unwrap();
    Ref::<[u16], Native, usize>::try_with_metadata(usize::MAX - 2, 0).unwrap();
    Ref::<[u16], Native, usize>::try_with_metadata(usize::MAX - 2, 1).unwrap();
    Ref::<[u16], Native, usize>::try_with_metadata(usize::MAX - 2, 2).unwrap_err();
}
