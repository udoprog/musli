#![allow(clippy::assertions_on_constants)]

use core::array;

use anyhow::Result;

use crate::mem::MaybeUninit;
use crate::{Ref, ZeroCopy};

use super::OwnedBuf;

#[derive(Debug, PartialEq, ZeroCopy)]
#[zero_copy(crate)]
#[repr(C)]
struct Inner {
    first: u8,
    second: u64,
}

#[test]
fn allocate_array_needing_padding() -> Result<()> {
    #[derive(Debug, PartialEq, ZeroCopy)]
    #[zero_copy(crate)]
    #[repr(C)]
    struct Element {
        first: u8,
        second: u32,
        third: [Inner; 2],
    }

    const _: () = assert!(Element::PADDED);

    let mut buf = OwnedBuf::new();

    let array = buf.store_uninit::<[Element; 10]>()?;

    let values = array::from_fn(|index| Element {
        first: index as u8,
        second: 0x01020304u32,
        third: [
            Inner {
                first: index as u8,
                second: 0x05060708090a0b0cu64,
            },
            Inner {
                first: index as u8,
                second: 0x05060708090a0b0cu64,
            },
        ],
    });

    buf.load_uninit_mut(array)?.write(&values);
    let array = array.assume_init();

    assert_eq!(buf.load(array)?, &values);
    Ok(())
}

#[test]
fn allocate_array_not_needing_padding() -> Result<()> {
    #[derive(Debug, PartialEq, ZeroCopy)]
    #[zero_copy(crate)]
    #[repr(C)]
    struct Element {
        first: u32,
        second: u32,
        third: u32,
    }

    const _: () = assert!(!Element::PADDED);

    let mut buf = OwnedBuf::new();

    let array = buf.store_uninit::<[Element; 10]>()?;

    let values = array::from_fn(|index| Element {
        first: index as u32,
        second: 0x01020304u32,
        third: 0x05060708u32,
    });

    buf.load_uninit_mut(array)?.write(&values);
    let array = array.assume_init();

    assert_eq!(buf.load(array)?, &values);
    Ok(())
}

#[test]
fn test_unaligned_write() -> Result<()> {
    #[derive(ZeroCopy)]
    #[repr(C)]
    #[zero_copy(crate)]
    struct Custom {
        string: Ref<str>,
    }

    let mut buf = OwnedBuf::with_capacity_and_alignment::<u8>(128)?;
    buf.extend_from_slice(&[1])?;

    let reference: Ref<MaybeUninit<Custom>> = buf.store_uninit::<Custom>()?;

    let string = buf.store_unsized("Hello World!")?;

    buf.load_uninit_mut(reference)?.write(&Custom { string });

    let reference = reference.assume_init();

    assert_eq!(reference.offset(), 4);

    buf.align_in_place()?;

    let custom = buf.load(reference)?;
    assert_eq!(buf.load(custom.string)?, "Hello World!");
    Ok(())
}

#[test]
fn inner_padding() -> Result<()> {
    #[derive(Debug, PartialEq, Clone, Copy, ZeroCopy)]
    #[repr(C, align(8))]
    #[zero_copy(crate)]
    struct Inner {
        field: u8,
    }

    #[derive(Debug, PartialEq, Clone, Copy, ZeroCopy)]
    #[repr(C, align(16))]
    #[zero_copy(crate)]
    struct Inner2 {
        field: u32,
    }

    #[derive(ZeroCopy)]
    #[repr(C)]
    #[zero_copy(crate)]
    struct Custom {
        inner: Inner,
        inner2: Inner2,
    }

    const _: () = assert!(Custom::PADDED);
    const _: () = assert!(Inner::PADDED);
    const _: () = assert!(Inner2::PADDED);

    let inner = Inner { field: 10 };
    let inner2 = Inner2 { field: 20 };
    let custom = Custom { inner, inner2 };

    let mut buf = OwnedBuf::with_capacity_and_alignment::<u8>(128)?;
    buf.extend_from_slice(&[1])?;

    let reference: Ref<MaybeUninit<Custom>> = buf.store_uninit::<Custom>()?;

    buf.load_uninit_mut(reference)?.write(&custom);

    let reference = reference.assume_init();

    assert_eq!(reference.offset(), 16);

    buf.align_in_place()?;

    let custom = buf.load(reference)?;
    assert_eq!(&custom.inner, &inner);
    assert_eq!(&custom.inner2, &inner2);
    Ok(())
}

#[test]
fn test_packing() {
    #[derive(ZeroCopy)]
    #[repr(C, packed)]
    #[zero_copy(crate)]
    struct Packed {
        inner: u8,
        inner2: u32,
    }

    const _: () = assert!(!Packed::PADDED);

    #[derive(ZeroCopy)]
    #[repr(C, packed(1))]
    #[zero_copy(crate)]
    struct Packed1 {
        inner: u8,
        inner2: u32,
    }

    const _: () = assert!(!Packed1::PADDED);
}
