use alloc::string::ToString;

use musli_zerocopy_macros::ZeroCopy;

use crate::Ref;
use crate::endian::{Big, Little, Native};
use crate::mem::MaybeUninit;

const MAX: usize = isize::MAX as usize;

macro_rules! each_byte_order {
    ($macro:path) => {
        $macro!(Native);
        $macro!(Little);
        $macro!(Big);
    };
}

#[test]
fn test_slice_with_metadata() {
    Ref::<[()], Native, usize>::try_with_metadata(0u32, usize::MAX).unwrap();

    macro_rules! test {
        ($order:path) => {
            Ref::<[u8], $order, usize>::try_with_metadata(0u32, MAX).unwrap();
            let e = Ref::<[u8], $order, usize>::try_with_metadata(0u32, MAX + 1).unwrap_err();
            assert_eq!(
                e.to_string(),
                "Invalid layout for size 9223372036854775808 and alignment 1"
            );

            Ref::<[u16], $order, usize>::try_with_metadata(0u32, MAX / 2).unwrap();
            let e =
                Ref::<[u16], $order, usize>::try_with_metadata(0u32, (MAX / 2) + 1).unwrap_err();
            assert_eq!(
                e.to_string(),
                "Invalid layout for size 9223372036854775808 and alignment 2"
            );
        };
    }

    each_byte_order!(test);
}

#[test]
fn test_metadata_byte_order() {
    #[derive(ZeroCopy)]
    #[zero_copy(crate)]
    #[repr(C)]
    struct BigStruct([u8; 4096]);

    const BIG_MAX_LEN: usize = MAX / size_of::<BigStruct>();

    macro_rules! test {
        ($order:path) => {
            Ref::<[BigStruct], $order, usize>::try_with_metadata(0u32, BIG_MAX_LEN).unwrap();
            let e = Ref::<[BigStruct], $order, usize>::try_with_metadata(0u32, BIG_MAX_LEN + 1)
                .unwrap_err();
            assert_eq!(
                e.to_string(),
                "Invalid layout for size 9223372036854775808 and alignment 1"
            );
        };
    }

    each_byte_order!(test);
}

#[test]
fn test_swap() {
    macro_rules! test {
        ($order:path) => {
            let e = Ref::<u8, $order, usize>::try_with_metadata(usize::MAX, ()).unwrap_err();
            assert_eq!(
                e.to_string(),
                "Offset 18446744073709551615 not in valid range 0-18446744073709551614"
            );

            let e = Ref::<MaybeUninit<u8>, $order, usize>::try_with_metadata(usize::MAX, ())
                .unwrap_err();
            assert_eq!(
                e.to_string(),
                "Offset 18446744073709551615 not in valid range 0-18446744073709551614"
            );
        };
    }

    each_byte_order!(test);
}
