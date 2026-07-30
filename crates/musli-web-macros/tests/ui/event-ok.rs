use musli::{Encode, Decode};
use musli_web::api::EncodeBody;

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

    // NB: A generic body has to ask for `EncodeBody`, which covers every mode a
    // body can be encoded in and is what allows any `api::Format` to be used.
    impl Broadcast for Type2 {
        impl<T> Event for Types<T> where T: EncodeBody;
    }
}

fn main() {}
