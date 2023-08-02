#![no_std]
#![feature(alloc_error_handler, start, core_intrinsics, lang_items, link_cfg)]

use core::alloc::GlobalAlloc;

use musli::{Decode, Encode};
use musli_json::allocator::NoStd;
use musli_json::context::NoStdContext;

struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, _: core::alloc::Layout) -> *mut u8 {
        core::intrinsics::abort();
    }

    unsafe fn dealloc(&self, _: *mut u8, _: core::alloc::Layout) {
        core::intrinsics::abort();
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

#[cfg(all(windows, target_env = "msvc"))]
#[link(name = "msvcrt")]
extern "C" {}

#[alloc_error_handler]
fn err_handler(_: core::alloc::Layout) -> ! {
    core::intrinsics::abort();
}

#[panic_handler]
#[lang = "panic_impl"]
extern "C" fn rust_begin_panic(_: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort();
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[cfg(unix)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() {}

#[derive(Debug, Encode, Decode)]
struct Value<'a> {
    name: &'a str,
    age: u32,
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let alloc = NoStd::<1024>::new();
    let mut cx = NoStdContext::new(&alloc);

    let encoding = musli_json::Encoding::new();

    let mut buf = [0u8; 1024];

    let value = Value {
        name: "Aristotle",
        age: 61,
    };

    let mut w = &mut buf[..];

    let Ok(..) = encoding.encode_with(&mut cx, &mut w, &value) else {
        for _error in cx.iter() {
            // report error
        }

        return 1;
    };

    let written = 1024 - w.len();

    let Ok(value): Result<Value, _> = encoding.from_slice_with(&mut cx, &buf[..written]) else {
        for _error in cx.iter() {
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
