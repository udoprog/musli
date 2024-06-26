#![no_std]
#![allow(internal_features)]
#![feature(start, core_intrinsics, lang_items, link_cfg)]

mod prelude;

use musli::alloc::{ArrayBuffer, Slice};
use musli::context;
use musli::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
struct Value<'a> {
    name: &'a str,
    age: u32,
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);
    let cx = context::with_alloc(&alloc);

    let encoding = musli::json::Encoding::new();

    let mut buf = [0u8; 1024];

    let value = Value {
        name: "Aristotle",
        age: 61,
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

    0
}
