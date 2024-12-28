#![allow(clippy::assertions_on_constants)]

use core::num::NonZero;

use musli::mode::Binary;
use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Decode, Encode)]
#[musli(bitwise)]
#[repr(C)]
struct BitwiseTuple(u32, u32, ());

const _: () = assert!(<BitwiseTuple as Encode<Binary>>::ENCODE_PACKED);
const _: () = assert!(<BitwiseTuple as Decode<Binary>>::DECODE_PACKED);

#[derive(Debug, PartialEq, Decode, Encode)]
#[musli(bitwise)]
#[repr(C)]
struct Bitwise {
    a: u32,
    b: u32,
    pad: (),
}

const _: () = assert!(<Bitwise as Encode<Binary>>::ENCODE_PACKED);
const _: () = assert!(<Bitwise as Decode<Binary>>::DECODE_PACKED);

#[derive(Debug, PartialEq, Decode, Encode)]
#[musli(bitwise)]
struct Zst {}

const _: () = assert!(<Zst as Encode<Binary>>::ENCODE_PACKED);
const _: () = assert!(<Zst as Decode<Binary>>::DECODE_PACKED);

#[derive(Debug, PartialEq, Decode, Encode)]
#[musli(bitwise)]
struct Zst2 {
    a: (),
}

const _: () = assert!(<Zst2 as Encode<Binary>>::ENCODE_PACKED);
const _: () = assert!(<Zst2 as Decode<Binary>>::DECODE_PACKED);

#[derive(Debug, PartialEq, Decode, Encode)]
#[musli(bitwise)]
#[repr(C)]
struct NotBitwise {
    a: u32,
    b: u16,
}

const _: () = assert!(!<NotBitwise as Encode<Binary>>::ENCODE_PACKED);
const _: () = assert!(!<NotBitwise as Decode<Binary>>::DECODE_PACKED);

#[derive(Debug, PartialEq, Decode, Encode)]
#[musli(bitwise)]
#[repr(C)]
struct BitwiseChar {
    a: char,
    b: u32,
}

const _: () = assert!(<BitwiseChar as Encode<Binary>>::ENCODE_PACKED);
const _: () = assert!(!<BitwiseChar as Decode<Binary>>::DECODE_PACKED);

#[derive(Debug, PartialEq, Decode, Encode)]
#[musli(bitwise)]
#[repr(C)]
struct BitwiseNonZero {
    a: NonZero<u32>,
    b: u32,
}

const _: () = assert!(<BitwiseNonZero as Encode<Binary>>::ENCODE_PACKED);
const _: () = assert!(!<BitwiseNonZero as Decode<Binary>>::DECODE_PACKED);
