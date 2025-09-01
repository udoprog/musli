use alloc::string::ToString;

use musli_zerocopy_macros::ZeroCopy;

use crate::endian::{Big, ByteOrder, Little, Native};
use crate::mem::PackedMaybeUninit;
use crate::traits::ZeroCopy;
use crate::{Buf, Error, Ref};

const MAX: usize = isize::MAX as usize;

macro_rules! each_byte_order {
    ($macro:path $(, $($tt:tt)*)?) => {
        $macro!(Native $(, $($tt:tt)*)?);
        $macro!(Little $(, $($tt:tt)*)?);
        $macro!(Big $(, $($tt:tt)*)?);
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

            let e = Ref::<PackedMaybeUninit<u8>, $order, usize>::try_with_metadata(usize::MAX, ())
                .unwrap_err();

            assert_eq!(
                e.to_string(),
                "Offset 18446744073709551615 not in valid range 0-18446744073709551614"
            );
        };
    }

    each_byte_order!(test);
}

#[test]
fn test_bit_pattern() -> Result<(), Error> {
    #[derive(Debug, ZeroCopy)]
    #[repr(C)]
    #[zero_copy(crate)]
    struct Empty;

    #[derive(Debug, ZeroCopy)]
    #[repr(C)]
    #[zero_copy(crate)]
    struct SizedValue {
        a: u64,
        b: u64,
    }

    macro_rules! each_size {
        ($macro:path $(, $($tt:tt)*)?) => {
            #[cfg(target_pointer_width = "32")]
            $macro!(u32, swap_u32 $(, $($tt)*)*);
            #[cfg(target_pointer_width = "64")]
            $macro!(u64, swap_u64 $(, $($tt)*)*);
            $macro!(usize, swap_usize $(, $($tt)*)*);
        };
    }

    macro_rules! test {
        ($size:ty, $swap:ident, $order:path) => {{
            // A raw representation of a Ref used for testing invalid bit patterns.
            #[derive(ZeroCopy)]
            #[repr(C)]
            #[zero_copy(crate)]
            struct RawRef {
                offset: $size,
                metadata: $size,
            }

            let mut invalid_ref = RawRef {
                offset: <$size>::MAX,
                metadata: <$order as ByteOrder>::$swap(<$size>::MAX / 2),
            };

            let buf = Buf::new(invalid_ref.to_bytes());

            assert!(buf.load(Ref::<Ref<Empty, $order, $size>>::zero()).is_ok());
            let slice = buf.load(Ref::<Ref<[Empty], $order, $size>>::zero())?;

            assert_eq!(
                slice.len() as $size,
                <$size>::MAX / 2,
                "{}: {}: Slice length should match metadata",
                stringify!($size),
                stringify!($order),
            );

            // Loading a sized value should fail, since it has a layout requirements
            // which cannot be satisfied by the given offset (`<$size>::MAX`).
            let e = buf
                .load(Ref::<Ref<SizedValue, $order, $size>>::zero())
                .unwrap_err();

            assert_eq!(
                e.to_string(),
                "Offset 18446744073709551615 not in valid range 0-18446744073709551599",
                "{}: {}: Error should match",
                stringify!($size),
                stringify!($order),
            );

            let e = buf
                .load(Ref::<Ref<[SizedValue], $order, $size>>::zero())
                .unwrap_err();

            assert_eq!(
                e.to_string(),
                "Invalid layout for overflowing size and alignment 8",
                "{}: {}: Error should match",
                stringify!($size),
                stringify!($order),
            );

            let e = Ref::<SizedValue, $order, $size>::try_with_metadata(invalid_ref.offset, ())
                .unwrap_err();

            assert_eq!(
                e.to_string(),
                "Offset 18446744073709551615 not in valid range 0-18446744073709551599",
                "{}: {}: Error should match",
                stringify!($size),
                stringify!($order),
            );

            let e = Ref::<[SizedValue], $order, $size>::try_with_metadata(
                invalid_ref.offset,
                invalid_ref.metadata as usize,
            )
            .unwrap_err();

            assert_eq!(
                e.to_string(),
                "Invalid layout for overflowing size and alignment 8",
                "{}: {}: Error should match",
                stringify!($size),
                stringify!($order),
            );

            let mut valid_ref = RawRef {
                offset: <$order as ByteOrder>::$swap(
                    <$size>::MAX - size_of::<SizedValue>() as $size,
                ),
                metadata: <$order as ByteOrder>::$swap(<$size>::MAX / 2),
            };

            let buf = Buf::new(valid_ref.to_bytes());

            // Loading a sized value should fail, since it has a layout requirements
            // which cannot be satisfied by the given offset (`usize::MAX`).
            let r = buf.load(Ref::<Ref<SizedValue, $order, $size>>::zero())?;

            assert_eq!(r.offset(), <$size>::MAX as usize - size_of::<SizedValue>());
        }};
    }

    macro_rules! inner {
        ($order:path) => {
            each_size!(test, $order);
        };
    }

    each_byte_order!(inner);
    Ok(())
}
