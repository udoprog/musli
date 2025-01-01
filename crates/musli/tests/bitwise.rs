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
