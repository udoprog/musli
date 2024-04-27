#![no_std]
#![allow(internal_features)]
#![feature(alloc_error_handler, start, core_intrinsics, lang_items, link_cfg)]

use musli::allocator::{Stack, StackBuffer};
use musli::context::StackContext;
use musli::{Decode, Encode};

#[cfg(all(windows, target_env = "msvc"))]
#[link(name = "msvcrt")]
extern "C" {}

#[cfg(unix)]
#[link(name = "c")]
extern "C" {}

#[alloc_error_handler]
fn err_handler(_: core::alloc::Layout) -> ! {
    core::intrinsics::abort();
}

#[panic_handler]
#[lang = "panic_impl"]
fn rust_begin_panic(_: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort();
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[cfg(unix)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() {}

// This needs to be implemented since core::intrinsics::abort is not stable.
#[no_mangle]
extern "C" fn __musli_abort() -> ! {
    core::intrinsics::abort();
}

#[derive(Debug, Encode, Decode)]
struct Value<'a> {
    name: &'a str,
    age: u32,
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
