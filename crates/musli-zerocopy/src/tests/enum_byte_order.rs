use crate::endian::{Big, Little, Other};
use crate::{Endian, ZeroCopy};

#[repr(C)]
struct Align<A, T: ?Sized>([A; 0], T);

#[derive(ZeroCopy, PartialEq, Debug, Clone, Copy)]
#[zero_copy(crate, swap_bytes)]
#[repr(u32)]
enum SomeEnum {
    A = 1,
    B = 2,
    A_ = 1u32.swap_bytes(),
    B_ = 2u32.swap_bytes(),
}

#[derive(ZeroCopy, Debug, Clone, Copy)]
#[zero_copy(crate)]
#[repr(C)]
struct SomeStructBE {
    a: Endian<u32, Big>,
    b: Endian<SomeEnum, Big>,
}

#[test]
fn test_from_bytes_be() {
    let data: &[u8; 8] = &Align::<SomeStructLE, _>([], [0, 0, 0, 1, 0, 0, 0, 2]).1;

    let struct_be = SomeStructBE::from_bytes(data).unwrap();

    assert_eq!(struct_be.a.to_ne(), 1);
    assert_eq!(struct_be.b.to_ne(), SomeEnum::B);
}

#[derive(ZeroCopy, Debug, Clone, Copy)]
#[zero_copy(crate)]
#[repr(C)]
struct SomeStructLE {
    a: Endian<u32, Little>,
    b: Endian<SomeEnum, Little>,
}

#[test]
fn test_from_bytes_le() {
    let data: &[u8; 8] = &Align::<SomeStructLE, _>([], [1, 0, 0, 0, 2, 0, 0, 0]).1;

    let struct_be = SomeStructLE::from_bytes(data).unwrap();

    assert_eq!(struct_be.a.to_ne(), 1);
    assert_eq!(struct_be.b.to_ne(), SomeEnum::B);
}

#[derive(ZeroCopy, PartialEq, Debug, Clone, Copy)]
#[zero_copy(crate)]
#[repr(u32)]
enum EnumNonSwapBytes {
    A = 1,
    B = 2,
    A_ = 1u32.swap_bytes(),
    B_ = 3u32.swap_bytes(),
}

#[derive(ZeroCopy, PartialEq, Debug, Clone, Copy)]
#[zero_copy(crate, swap_bytes)]
#[repr(u32)]
enum EnumSwapBytesMismatch {
    A = 1,
    B = 2,
    A_ = 1u32.swap_bytes(),
    B_ = 3u32.swap_bytes(),
}

const _: () = assert!(!EnumNonSwapBytes::CAN_SWAP_BYTES);
const _: () = assert!(!EnumSwapBytesMismatch::CAN_SWAP_BYTES);

#[test]
fn test_enum_unsupported_swap_bytes() {}

#[derive(ZeroCopy, PartialEq, Debug, Clone, Copy)]
#[zero_copy(crate, swap_bytes)]
#[repr(u32)]
enum FieldEnum {
    A { field: u32 } = 1,
    B = 2,
    A_ { field: u32 } = 1u32.swap_bytes(),
    B_ = 2u32.swap_bytes(),
}

const _: () = assert!(FieldEnum::CAN_SWAP_BYTES);

#[test]
fn test_field_enum() {
    let a = FieldEnum::A { field: 42 };
    let a_ = a.swap_bytes::<Other>();

    let b = FieldEnum::B;
    let b_ = b.swap_bytes::<Other>();

    assert_eq!(
        a_,
        FieldEnum::A_ {
            field: 42u32.swap_bytes()
        }
    );

    assert_eq!(b_, FieldEnum::B_);
}
