#![no_std]
#![allow(internal_features)]
#![feature(start, core_intrinsics, lang_items, link_cfg)]

mod prelude;

use musli::allocator::{Stack, StackBuffer};
use musli::context::StackContext;
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Serde {
    field: u32,
}

#[derive(Debug, Encode, Decode)]
struct Value<'a> {
    name: &'a str,
    age: u32,
    #[musli(with = musli::serde)]
    serde: Serde,
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let mut buf = StackBuffer::<1024>::new();
    let alloc = Stack::new(&mut buf);
    let cx = StackContext::new(&alloc);

    let encoding = musli::json::Encoding::new();

    let mut buf = [0u8; 1024];

    let value = Value {
        name: "Aristotle",
        age: 61,
        serde: Serde { field: 42 },
    };

    let mut w = &mut buf[..];

    let Ok(..) = encoding.encode_with(&cx, &mut w, &value) else {
        for _error in cx.errors() {
            // report error
        }

        return 1;
    };

    let written = 1024 - w.len();

    let Ok(value): Result<Value, _> = encoding.from_slice_with(&cx, &buf[..written]) else {
        for _error in cx.errors() {
            // report error
        }

        return 2;
    };

    if value.name != "Aristotle" {
        return 3;
    }

    if value.age != 61 {
        return 4;
    }

    if value.serde.field != 42 {
        return 5;
    }

    0
}
