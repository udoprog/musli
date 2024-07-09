use crate::endian::{Big, Little};
use crate::{Endian, ZeroCopy};

#[derive(ZeroCopy, PartialEq, Debug, Clone, Copy)]
#[zero_copy(crate)]
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
    let data: [u8; 8] = [0, 0, 0, 1, 0, 0, 0, 2];

    let struct_be = SomeStructBE::from_bytes(&data).unwrap();

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
    let data: [u8; 8] = [1, 0, 0, 0, 2, 0, 0, 0];

    let struct_be = SomeStructLE::from_bytes(&data).unwrap();

    assert_eq!(struct_be.a.to_ne(), 1);
    assert_eq!(struct_be.b.to_ne(), SomeEnum::B);
}
