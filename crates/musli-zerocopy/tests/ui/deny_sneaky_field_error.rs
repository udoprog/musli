//! This test ensures that the derive can correctly see fields which were
//! sneakily introduced by an attribute macro.

use musli_zerocopy::ZeroCopy;
use musli_zerocopy_macros::sneaky_fields;

#[derive(ZeroCopy)]
#[repr(C)]
struct Sneaky(u64);

#[derive(ZeroCopy)]
#[repr(C)]
#[sneaky_fields(Sneaky)]
struct SneakyNamed {
    field: u32,
}

#[sneaky_fields(Sneaky)]
#[derive(ZeroCopy)]
#[repr(C)]
struct SneakyNamed2 {
    field: u32,
}

#[derive(ZeroCopy)]
#[repr(C)]
#[sneaky_fields(Sneaky)]
struct SneakyUnnamed(u32);

#[sneaky_fields(Sneaky)]
#[derive(ZeroCopy)]
#[repr(C)]
struct SneakyUnnamed2(u32);

#[derive(ZeroCopy)]
#[repr(u8)]
#[sneaky_fields(Sneaky)]
enum SneakyEnumNamed {
    Named {
        field: u32,
    },
}

#[derive(ZeroCopy)]
#[repr(u8)]
#[sneaky_fields(Sneaky)]
enum SneakyEnumUnnamed {
    Unnamed(u32)
}

#[derive(ZeroCopy)]
#[repr(u8)]
#[sneaky_fields(Sneaky)]
enum SneakyEnumUnit {
    Unit
}

fn main() {
}
