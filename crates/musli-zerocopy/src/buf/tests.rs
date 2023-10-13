#![allow(clippy::assertions_on_constants)]

use core::array;

use crate::buf::MaybeUninit;
use crate::error::Error;
use crate::pointer::{Ref, Unsized};
use crate::ZeroCopy;

use super::AlignedBuf;

#[derive(Debug, PartialEq, ZeroCopy)]
#[zero_copy(crate)]
#[repr(C)]
struct Inner {
    first: u8,
    second: u64,
}

#[test]
fn allocate_array_needing_padding() -> Result<(), Error> {
    #[derive(Debug, PartialEq, ZeroCopy)]
    #[zero_copy(crate)]
    #[repr(C)]
    struct Element {
        first: u8,
        second: u32,
        third: [Inner; 2],
    }

    assert!(Element::PADDED);

    let mut buf = AlignedBuf::new();

    let array = buf.store_uninit::<[Element; 10]>();

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

    buf.load_uninit_mut(array).write(&values);
    let array = array.assume_init();

    let buf = buf.as_aligned();

    assert_eq!(buf.load(array)?, &values);
    Ok(())
}

#[test]
fn allocate_array_not_needing_padding() -> Result<(), Error> {
    #[derive(Debug, PartialEq, ZeroCopy)]
    #[zero_copy(crate)]
    #[repr(C)]
    struct Element {
        first: u32,
        second: u32,
        third: u32,
    }

    assert!(!Element::PADDED);

    let mut buf = AlignedBuf::new();

    let array = buf.store_uninit::<[Element; 10]>();

    let values = array::from_fn(|index| Element {
        first: index as u32,
        second: 0x01020304u32,
        third: 0x05060708u32,
    });

    buf.load_uninit_mut(array).write(&values);
    let array = array.assume_init();

    let buf = buf.as_aligned();

    assert_eq!(buf.load(array)?, &values);
    Ok(())
}

#[test]
fn test_unaligned_write() -> Result<(), Error> {
    #[derive(ZeroCopy)]
    #[repr(C)]
    #[zero_copy(crate)]
    struct Custom {
        string: Unsized<str>,
    }

    let mut buf = AlignedBuf::with_capacity_and_alignment::<u8>(128);
    buf.extend_from_slice(&[1]);

    let reference: Ref<MaybeUninit<Custom>> = buf.store_uninit::<Custom>();

    let string = buf.store_unsized("Hello World!");

    buf.load_uninit_mut(reference).write(&Custom { string });

    let buf = buf.as_aligned();
    let reference = reference.assume_init();

    assert_eq!(reference.offset(), 4);

    let custom = buf.load(reference)?;
    assert_eq!(buf.load(custom.string)?, "Hello World!");
    Ok(())
}

#[test]
fn inner_padding() -> Result<(), Error> {
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

    assert!(Custom::PADDED);
    assert!(Inner::PADDED);
    assert!(Inner2::PADDED);

    let inner = Inner { field: 10 };
    let inner2 = Inner2 { field: 20 };
    let custom = Custom { inner, inner2 };

    let mut buf = AlignedBuf::with_capacity_and_alignment::<u8>(128);
    buf.extend_from_slice(&[1]);

    let reference: Ref<MaybeUninit<Custom>> = buf.store_uninit::<Custom>();

    buf.load_uninit_mut(reference).write(&custom);

    let buf = buf.as_aligned();
    let reference = reference.assume_init();

    assert_eq!(reference.offset(), 16);

    let custom = buf.load(reference)?;
    assert_eq!(&custom.inner, &inner);
    assert_eq!(&custom.inner2, &inner2);
    Ok(())
}
