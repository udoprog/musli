#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(core_intrinsics, lang_items, link_cfg)]

mod prelude;

use core::ffi::c_int;

use musli::alloc::{ArrayBuffer, Slice};
use musli::context;
use musli::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
struct Value<'a> {
    name: &'a str,
    age: u32,
}

#[no_mangle]
extern "C" fn main(_argc: c_int, _argv: *const *const u8) -> c_int {
    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);
    let cx = context::with_alloc(&alloc).with_trace();

    let encoding = musli::json::Encoding::new();

    let mut buf = [0u8; 1024];

    let value = Value {
        name: "Aristotle",
        age: 61,
    };

    let Ok(w) = encoding.to_slice_with(&cx, &mut buf[..], &value) else {
        for _error in cx.errors() {
            // report error
        }

        return 1;
    };

    let Ok(value): Result<Value, _> = encoding.from_slice_with(&cx, &buf[..w]) else {
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
