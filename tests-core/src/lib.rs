//! Types deriving `Encode` and `Decode` through `musli_core` without a
//! `#[musli(crate = ..)]` attribute.
//!
//! This crate does not depend on `musli`, so it only builds if the derives
//! refer to `musli_core`.

#![no_std]

use musli_core::{Decode, Encode};

#[derive(Encode, Decode)]
pub struct Person<'a> {
    pub name: &'a str,
    pub age: u32,
}

#[derive(Encode, Decode)]
#[musli(packed)]
pub struct Packed {
    pub a: u8,
    pub b: u64,
}

#[derive(Encode, Decode)]
#[musli(transparent)]
pub struct Transparent(pub u32);

#[derive(Encode, Decode)]
pub struct Generic<T> {
    pub value: T,
}

#[derive(Encode, Decode)]
#[musli(Text, name_all = "kebab-case")]
pub enum Animal {
    Dog { age: u32 },
    Cat(u32),
    Fish,
}

#[derive(Encode, Decode)]
#[musli(Binary, name_all = "index")]
#[musli(Text, name_all = "name")]
pub struct Modes {
    #[musli(Text, name = "renamed")]
    pub field: u32,
    #[musli(default, skip_encoding_if = Option::is_none)]
    pub optional: Option<u32>,
}

#[cfg(test)]
mod tests {
    use musli_core::mode::{Binary, Text};
    use musli_core::{Decode, Encode};

    use super::*;

    fn encode<T: ?Sized + Encode<Binary> + Encode<Text>>() {}
    fn decode<'de, T: Decode<'de, Binary, A> + Decode<'de, Text, A>, A: musli_core::Allocator>() {}

    #[test]
    fn derives_through_musli_core() {
        fn check<A: musli_core::Allocator>() {
            encode::<Person<'_>>();
            decode::<Person<'_>, A>();
            encode::<Packed>();
            decode::<Packed, A>();
            encode::<Transparent>();
            decode::<Transparent, A>();
            encode::<Generic<u32>>();
            decode::<Generic<u32>, A>();
            encode::<Animal>();
            decode::<Animal, A>();
            encode::<Modes>();
            decode::<Modes, A>();
        }

        check::<musli_core::alloc::Global>();
    }
}
