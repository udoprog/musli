//! This is a test that ensures that arbitrary packs of data can be successfully skipped over.

#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Inner;

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(packed)]
struct Packed<const N: usize> {
    header: u32,
    #[musli(bytes)]
    values: [u8; N],
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct PackedCompat<const N: usize, const L: usize> {
    prefix: u32,
    small: Packed<N>,
    large: Packed<L>,
    suffix: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct IgnoreLarge<const N: usize> {
    prefix: u32,
    #[musli(rename = 1)]
    small: Packed<N>,
    #[musli(rename = 3)]
    suffix: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct IgnoreSmall<const L: usize> {
    prefix: u32,
    #[musli(rename = 2)]
    large: Packed<L>,
    #[musli(rename = 3)]
    suffix: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct IgnoreBoth {
    prefix: u32,
    #[musli(rename = 3)]
    suffix: u32,
}

const fn array<const N: usize>() -> [u8; N] {
    let mut array = [0; N];
    let mut i = 0;

    while i < N {
        array[i] = i as u8;
        i += 1;
    }

    array
}

fn test_length<const N: usize, const L: usize>() {
    tests::rt! {
        upgrade_stable,
        PackedCompat {
            prefix: 42,
            small: Packed { header: 42, values: array::<N>() },
            large: Packed { header: 42, values: array::<L>() },
            suffix: 84,
        }
    };

    tests::assert_decode_eq! {
        upgrade_stable,
        PackedCompat {
            prefix: 42,
            small: Packed { header: 42, values: array::<N>() },
            large: Packed { header: 42, values: array::<L>() },
            suffix: 84,
        },
        IgnoreSmall {
            prefix: 42,
            large: Packed { header: 42, values: array::<L>() },
            suffix: 84
        }
    };

    tests::assert_decode_eq! {
        upgrade_stable,
        PackedCompat {
            prefix: 42,
            small: Packed { header: 42, values: array::<N>() },
            large: Packed { header: 42, values: array::<L>() },
            suffix: 84,
        },
        IgnoreLarge {
            prefix: 42,
            small: Packed { header: 42, values: array::<N>() },
            suffix: 84
        }
    };

    tests::assert_decode_eq! {
        upgrade_stable,
        PackedCompat {
            prefix: 42,
            small: Packed { header: 42, values: array::<N>() },
            large: Packed { header: 42, values: array::<L>() },
            suffix: 84,
        },
        IgnoreBoth {
            prefix: 42,
            suffix: 84
        }
    };
}

#[test]
fn test_lengths() {
    test_length::<{ tests::wire::tag::MAX_INLINE_LEN - 4 }, 256>();
    test_length::<{ tests::descriptive::tag::MAX_INLINE_LEN - 4 }, 256>();
}
