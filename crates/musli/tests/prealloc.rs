//! Tests that a length prefix read out of the input cannot be used to make the
//! decoder pre-allocate an arbitrary amount of memory.

#![cfg(feature = "descriptive")]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use musli::alloc::Global;
use musli::value::Value;

static MAX_REQUEST: AtomicUsize = AtomicUsize::new(0);

struct Counting;

// SAFETY: Every operation is forwarded to `System`, only recording the size of
// the requested allocation on the way through.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        MAX_REQUEST.fetch_max(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        MAX_REQUEST.fetch_max(new_size, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOC: Counting = Counting;

/// The pre-allocation cap in `musli_core::internal::size_hint`.
const MAX_PREALLOC_BYTES: usize = 1024 * 1024;

fn varint(mut value: u64) -> Vec<u8> {
    let mut out = Vec::new();

    loop {
        let b = (value & 0x7F) as u8;
        value >>= 7;

        if value == 0 {
            out.push(b);
            return out;
        }

        out.push(b | 0x80);
    }
}

/// Build a descriptive buffer consisting of nothing but a container header
/// which claims to hold `len` elements.
fn absurd_container(kind: u8, len: u64) -> Vec<u8> {
    // Data set to all ones means the length follows as a prefix.
    let mut bytes = vec![kind | 0b000_11111];
    bytes.extend_from_slice(&varint(len));
    bytes
}

#[test]
fn sequence_length_does_not_pre_allocate() {
    let bytes = absurd_container(0b011_00000, 100_000_000);
    assert!(bytes.len() < 16);

    MAX_REQUEST.store(0, Ordering::Relaxed);
    let result = musli::descriptive::from_slice::<Value<Global>>(&bytes);
    let peak = MAX_REQUEST.load(Ordering::Relaxed);

    assert!(result.is_err(), "truncated sequence should not decode");
    assert!(
        peak <= MAX_PREALLOC_BYTES,
        "a {} byte input requested a {peak} byte allocation",
        bytes.len()
    );
}

#[test]
fn map_length_does_not_pre_allocate() {
    let bytes = absurd_container(0b100_00000, 100_000_000);
    assert!(bytes.len() < 16);

    MAX_REQUEST.store(0, Ordering::Relaxed);
    let result = musli::descriptive::from_slice::<Value<Global>>(&bytes);
    let peak = MAX_REQUEST.load(Ordering::Relaxed);

    assert!(result.is_err(), "truncated map should not decode");
    assert!(
        peak <= MAX_PREALLOC_BYTES,
        "a {} byte input requested a {peak} byte allocation",
        bytes.len()
    );
}
