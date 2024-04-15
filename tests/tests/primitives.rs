#![cfg(feature = "test")]

use musli::compat::Sequence;
use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Inner;

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Primitives {
    pub bool_field: bool,
    pub char_field: char,
    pub u8_field: u8,
    pub u16_field: u16,
    pub u32_field: u32,
    pub u64_field: u64,
    pub u128_field: u128,
    pub i8_field: i8,
    pub i16_field: i16,
    pub i32_field: i32,
    pub i64_field: i64,
    pub i128_field: i128,
    pub f32_field: f32,
    pub f64_field: f64,
    pub usize_field: usize,
    pub isize_field: isize,
    pub empty_tuple: (),
}

#[test]
fn primitives() {
    tests::rt!(
        full,
        Primitives {
            bool_field: true,
            char_field: char::MAX,
            u8_field: u8::MAX,
            u16_field: u16::MAX,
            u32_field: u32::MAX,
            u64_field: u64::MAX,
            u128_field: u128::MAX,
            i8_field: i8::MAX,
            i16_field: i16::MAX,
            i32_field: i32::MAX,
            i64_field: i64::MAX,
            i128_field: i128::MAX,
            f32_field: f32::MAX,
            f64_field: f64::MAX,
            usize_field: usize::MAX,
            isize_field: isize::MAX,
            empty_tuple: (),
        }
    );

    tests::rt!(
        full,
        Primitives {
            bool_field: false,
            char_field: '\u{0000}',
            u8_field: u8::MIN,
            u16_field: u16::MIN,
            u32_field: u32::MIN,
            u64_field: u64::MIN,
            u128_field: u128::MIN,
            i8_field: i8::MIN,
            i16_field: i16::MIN,
            i32_field: i32::MIN,
            i64_field: i64::MIN,
            i128_field: i128::MIN,
            f32_field: f32::MIN,
            f64_field: f64::MIN,
            usize_field: usize::MIN,
            isize_field: isize::MIN,
            empty_tuple: (),
        }
    );
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Arrays {
    pub empty_array_field: [u8; 0],
    pub ten_array: [i32; 10],
}

#[test]
fn arrays() {
    tests::rt!(
        full,
        Arrays {
            empty_array_field: [],
            ten_array: [i32::MIN; 10],
        }
    );

    tests::rt!(
        full,
        Arrays {
            empty_array_field: [],
            ten_array: [i32::MAX; 10],
        }
    );
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Sequences {
    pub empty_sequence: Sequence<()>,
}

#[test]
fn sequences() {
    tests::rt!(
        full,
        Sequences {
            empty_sequence: Sequence(()),
        }
    );

    tests::rt!(
        full,
        Sequences {
            empty_sequence: Sequence(()),
        }
    );
}
