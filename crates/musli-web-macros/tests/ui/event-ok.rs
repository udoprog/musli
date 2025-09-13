use musli::{Encode, Decode};
use musli::mode::Binary;
use musli::alloc::Global;

#[derive(Encode, Decode)]
struct Lifetime<'a> {
    a: &'a str,
}

#[derive(Encode, Decode)]
struct Types<T> {
    value: T,
}

musli_web_macros::define! {
    type Type1;

    impl Broadcast for Type1 {
        impl<'de> Event for Lifetime<'de>;
    }

    type Type2;

    impl Broadcast for Type2 {
        impl<'de, T> Event for Types<T> where T: Decode<'de, Binary, Global> + Encode<Binary>;
    }
}

fn main() {}
