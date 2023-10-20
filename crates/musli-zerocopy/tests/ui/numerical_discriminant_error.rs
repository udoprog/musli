//! This ensure that the `ZeroCopy` macro produces decent diagnostics when
//! dealing with simple numerical discriminants.

use musli_zerocopy::ZeroCopy;

#[derive(ZeroCopy)]
#[repr(u8)]
enum UnsignedNegative {
    Variant = 1u8
}

#[derive(ZeroCopy)]
#[repr(u8)]
enum Overflow {
    Variant = 255,
    Variant2
}

fn main() {
}
