#![allow(clippy::assertions_on_constants)]

use core::num::NonZero;

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Decode, Encode)]
#[musli(packed)]
#[repr(C)]
struct BitwiseTuple(u32, u32, ());

const _: () = assert!(musli::is_bitwise_encode::<BitwiseTuple>());
const _: () = assert!(musli::is_bitwise_decode::<BitwiseTuple>());

#[derive(Debug, PartialEq, Decode, Encode)]
#[musli(packed)]
#[repr(C)]
struct Bitwise {
    a: u32,
    b: u32,
    pad: (),
}

const _: () = assert!(musli::is_bitwise_encode::<Bitwise>());
const _: () = assert!(musli::is_bitwise_decode::<Bitwise>());

#[derive(Debug, PartialEq, Decode, Encode)]
#[musli(packed)]
struct Zst {}

const _: () = assert!(musli::is_bitwise_encode::<Zst>());
const _: () = assert!(musli::is_bitwise_decode::<Zst>());

#[derive(Debug, PartialEq, Decode, Encode)]
#[musli(packed)]
struct Zst2 {
    a: (),
}

const _: () = assert!(musli::is_bitwise_encode::<Zst2>());
const _: () = assert!(musli::is_bitwise_decode::<Zst2>());

#[derive(Debug, PartialEq, Decode, Encode)]
#[musli(packed)]
#[repr(C)]
struct NotBitwise {
    a: u32,
    b: u16,
}

const _: () = assert!(!musli::is_bitwise_encode::<NotBitwise>());
const _: () = assert!(!musli::is_bitwise_decode::<NotBitwise>());

#[derive(Debug, PartialEq, Decode, Encode)]
#[musli(packed)]
#[repr(C)]
struct BitwiseChar {
    a: char,
    b: u32,
}

const _: () = assert!(musli::is_bitwise_encode::<BitwiseChar>());
const _: () = assert!(!musli::is_bitwise_decode::<BitwiseChar>());

#[derive(Debug, PartialEq, Decode, Encode)]
#[musli(packed)]
#[repr(C)]
struct BitwiseNonZero {
    a: NonZero<u32>,
    b: u32,
}

const _: () = assert!(musli::is_bitwise_encode::<BitwiseNonZero>());
const _: () = assert!(!musli::is_bitwise_decode::<BitwiseNonZero>());

#[cfg(target_has_atomic = "8")]
#[derive(Debug, Decode)]
#[musli(packed)]
#[repr(C)]
struct WithAtomicBool {
    a: core::sync::atomic::AtomicBool,
}

// An atomic boolean must not be bitwise decoded, since not every byte is a
// valid bool.
#[test]
#[cfg(target_has_atomic = "8")]
fn atomic_bool_is_not_bitwise_decode() {
    assert!(!musli::is_bitwise_decode::<core::sync::atomic::AtomicBool>());
    assert!(!musli::is_bitwise_decode::<WithAtomicBool>());
}

#[test]
#[cfg(target_has_atomic = "8")]
fn atomic_bool_rejects_invalid_byte() {
    use core::sync::atomic::{AtomicBool, Ordering};

    assert!(musli::packed::from_slice::<AtomicBool>(&[2]).is_err());
    assert!(musli::storage::from_slice::<AtomicBool>(&[2]).is_err());
    assert!(musli::packed::from_slice::<[AtomicBool; 2]>(&[1, 2]).is_err());
    assert!(musli::packed::from_slice::<WithAtomicBool>(&[2]).is_err());

    let value = musli::packed::from_slice::<AtomicBool>(&[1]).unwrap();
    assert!(value.load(Ordering::Relaxed));
    let value = musli::packed::from_slice::<AtomicBool>(&[0]).unwrap();
    assert!(!value.load(Ordering::Relaxed));
}
