//! `isize` is a signed type, so it has to be narrowed and sign extended to the
//! configured pointer width rather than reinterpreted as a `usize` at the width
//! of the host.
//!
//! Reinterpreting it meant that no negative value could be encoded with a
//! narrow pointer width, and that the encoding differed between a 32-bit and a
//! 64-bit host.

#![cfg(all(feature = "storage", feature = "wire"))]

use musli::options::{self, Options, Width};

const VARIABLE: Options = options::new().build();
const PTR8: Options = options::new().pointer(Width::U8).build();
const PTR16: Options = options::new().pointer(Width::U16).build();
const PTR32: Options = options::new().pointer(Width::U32).build();
const PTR64: Options = options::new().pointer(Width::U64).build();

macro_rules! round_trip {
    ($opt:ident, $values:expr) => {{
        const STORAGE: musli::storage::Encoding<$opt> =
            musli::storage::Encoding::new().with_options();
        const WIRE: musli::wire::Encoding<$opt> = musli::wire::Encoding::new().with_options();

        for value in $values {
            let bytes = STORAGE
                .to_vec(&value)
                .unwrap_or_else(|e| panic!("storage {} {value}: {e}", stringify!($opt)));

            assert_eq!(
                STORAGE.from_slice::<isize>(&bytes).unwrap(),
                value,
                "storage {}",
                stringify!($opt)
            );

            let bytes = WIRE
                .to_vec(&value)
                .unwrap_or_else(|e| panic!("wire {} {value}: {e}", stringify!($opt)));

            assert_eq!(
                WIRE.from_slice::<isize>(&bytes).unwrap(),
                value,
                "wire {}",
                stringify!($opt)
            );
        }
    }};
}

#[test]
fn round_trip_at_every_width() {
    round_trip!(VARIABLE, [0isize, 1, -1, -2, isize::MIN, isize::MAX]);
    round_trip!(PTR64, [0isize, 1, -1, -2, isize::MIN, isize::MAX]);
    round_trip!(
        PTR32,
        [0isize, 1, -1, -2, i32::MIN as isize, i32::MAX as isize]
    );
    round_trip!(
        PTR16,
        [0isize, 1, -1, -2, i16::MIN as isize, i16::MAX as isize]
    );
    round_trip!(PTR8, [0isize, 1, -1, -2, -128, 127]);
}

/// A value which does not fit the configured width is rejected rather than
/// silently truncated.
#[test]
fn out_of_range_is_rejected() {
    const STORAGE: musli::storage::Encoding<PTR8> = musli::storage::Encoding::new().with_options();
    const WIRE: musli::wire::Encoding<PTR8> = musli::wire::Encoding::new().with_options();

    for value in [128isize, -129, isize::MIN, isize::MAX] {
        assert!(STORAGE.to_vec(&value).is_err(), "storage {value}");
        assert!(WIRE.to_vec(&value).is_err(), "wire {value}");
    }
}

/// A fixed width encoding uses exactly that many bytes and does not depend on
/// the pointer width of the host, which is what makes it portable.
#[test]
fn fixed_width_is_host_independent() {
    const STORAGE32: musli::storage::Encoding<PTR32> =
        musli::storage::Encoding::new().with_options();

    assert_eq!(STORAGE32.to_vec(&(-1isize)).unwrap(), [0xFF; 4]);
    assert_eq!(STORAGE32.to_vec(&0isize).unwrap(), [0x00; 4]);

    const STORAGE8: musli::storage::Encoding<PTR8> = musli::storage::Encoding::new().with_options();

    assert_eq!(STORAGE8.to_vec(&(-1isize)).unwrap(), [0xFF]);
    assert_eq!(STORAGE8.to_vec(&(-128isize)).unwrap(), [0x80]);
    assert_eq!(STORAGE8.to_vec(&127isize).unwrap(), [0x7F]);
}

/// A small negative value is zig-zag encoded under a variable width, so it
/// takes one byte rather than the ten of its two's complement.
#[test]
fn variable_width_is_zig_zag_encoded() {
    const STORAGE: musli::storage::Encoding<VARIABLE> =
        musli::storage::Encoding::new().with_options();

    assert_eq!(STORAGE.to_vec(&(-1isize)).unwrap(), [1]);
    assert_eq!(STORAGE.to_vec(&0isize).unwrap(), [0]);
    assert_eq!(STORAGE.to_vec(&1isize).unwrap(), [2]);
    assert_eq!(STORAGE.to_vec(&(-2isize)).unwrap(), [3]);
}
