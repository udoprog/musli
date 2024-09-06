#![allow(clippy::assertions_on_constants)]

use crate::endian::{Big, Little, Other};
use crate::{ByteOrder, Endian, ZeroCopy};

#[repr(C)]
struct Align<A, T: ?Sized>([A; 0], T);

#[test]
fn test_byte_order() {
    #[derive(ZeroCopy, PartialEq, Debug, Clone, Copy)]
    #[zero_copy(crate, swap_bytes)]
    #[repr(u32)]
    enum SomeEnum {
        A = 1,
        #[zero_copy(swap = A)]
        A_ = u32::swap_bytes(1),
        B = 2,
        #[zero_copy(swap = B)]
        B_ = u32::swap_bytes(2),
    }

    #[derive(ZeroCopy, Debug, Clone, Copy)]
    #[zero_copy(crate)]
    #[repr(C)]
    struct SomeStruct<T>
    where
        T: ByteOrder,
    {
        a: Endian<u32, T>,
        b: Endian<SomeEnum, T>,
    }

    let data: &[u8; 8] = &Align::<SomeStruct<Big>, _>([], [0, 0, 0, 1, 0, 0, 0, 2]).1;
    let st = SomeStruct::<Big>::from_bytes(data).unwrap();

    assert_eq!(st.a.to_ne(), 1);
    assert_eq!(st.b.to_ne(), SomeEnum::B);

    let data: &[u8; 8] = &Align::<SomeStruct<Little>, _>([], [1, 0, 0, 0, 2, 0, 0, 0]).1;
    let st = SomeStruct::<Little>::from_bytes(data).unwrap();

    assert_eq!(st.a.to_ne(), 1);
    assert_eq!(st.b.to_ne(), SomeEnum::B);
}

#[derive(ZeroCopy, PartialEq, Debug, Clone, Copy)]
#[zero_copy(crate)]
#[repr(u32)]
enum EnumNonSwapBytes {
    A = 1,
    A_ = u32::swap_bytes(1),
    B = 2,
    B_ = u32::swap_bytes(3),
}

const _: () = assert!(!EnumNonSwapBytes::CAN_SWAP_BYTES);

#[derive(ZeroCopy, PartialEq, Debug, Clone, Copy)]
#[zero_copy(crate, swap_bytes)]
#[repr(u32)]
enum EnumSwapBytesMismatch {
    A = 1,
    #[zero_copy(swap = A)]
    A_ = u32::swap_bytes(1),
    B = 2,
    #[zero_copy(swap = B)]
    B_ = u32::swap_bytes(3),
}

const _: () = assert!(!EnumSwapBytesMismatch::CAN_SWAP_BYTES);

#[test]
fn test_field_enum() {
    #[derive(ZeroCopy, PartialEq, Debug, Clone, Copy)]
    #[zero_copy(crate, swap_bytes)]
    #[repr(u32)]
    enum FieldEnum {
        A {
            field: u32,
        } = 1,
        #[zero_copy(swap = A)]
        OtherA {
            field: u32,
        } = u32::swap_bytes(1),
        B = 2,
        #[zero_copy(swap = B)]
        OtherB = u32::swap_bytes(2),
    }

    const _: () = assert!(FieldEnum::CAN_SWAP_BYTES);

    let a = FieldEnum::A { field: 42 };
    let a_ = a.swap_bytes::<Other>();
    assert_eq!(
        a_,
        FieldEnum::OtherA {
            field: u32::swap_bytes(42)
        }
    );

    let b = FieldEnum::B;
    let b_ = b.swap_bytes::<Other>();
    assert_eq!(b_, FieldEnum::OtherB);
}

#[test]
fn test_field_enum_implicit() {
    #[derive(ZeroCopy, PartialEq, Debug, Clone, Copy)]
    #[zero_copy(crate, swap_bytes)]
    #[repr(u32)]
    enum FieldEnumImplicit {
        A { field: u32 } = 1,
        A_ { field: u32 } = u32::swap_bytes(1),
        B = 2,
        B_ = u32::swap_bytes(2),
    }

    const _: () = assert!(FieldEnumImplicit::CAN_SWAP_BYTES);

    let a = FieldEnumImplicit::A { field: 42 };
    let a_ = a.swap_bytes::<Other>();

    let b = FieldEnumImplicit::B;
    let b_ = b.swap_bytes::<Other>();

    assert_eq!(
        a_,
        FieldEnumImplicit::A_ {
            field: u32::swap_bytes(42)
        }
    );

    assert_eq!(b_, FieldEnumImplicit::B_);
}
