use musli::compat::{Bytes, Sequence};
use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Inner;

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Numbers {
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
    pub usize_field: usize,
    pub isize_field: isize,
    pub empty_array_field: Bytes<[u8; 0]>,
    pub empty_tuple: (),
    pub empty_sequence: Sequence<()>,
}

#[test]
fn test_primitives_max() {
    musli_wire::test::rt(Numbers {
        bool_field: false,
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
        usize_field: usize::MAX,
        isize_field: isize::MAX,
        empty_array_field: Bytes([]),
        empty_tuple: (),
        empty_sequence: Sequence(()),
    });
}

#[test]
fn test_primitives_min() {
    musli_wire::test::rt(Numbers {
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
        usize_field: usize::MIN,
        isize_field: isize::MIN,
        empty_array_field: Bytes([]),
        empty_tuple: (),
        empty_sequence: Sequence(()),
    });
}
