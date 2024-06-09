use crate::buf::BufVec;
use crate::Allocator;

fn basic_allocations<A: Allocator>(alloc: &A) {
    let mut a = BufVec::new_in(alloc);
    let mut b = BufVec::new_in(alloc);

    b.write(b"He11o");

    assert_eq!(b.as_slice(), b"He11o");
    assert_eq!(b.len(), 5);

    a.write(b.as_slice());

    assert_eq!(a.as_slice(), b"He11o");
    assert_eq!(a.len(), 5);

    a.write(b" W0rld");

    assert_eq!(a.as_slice(), b"He11o W0rld");
    assert_eq!(a.len(), 11);

    let mut c = BufVec::new_in(alloc);
    c.write(b"!");
    assert_eq!(c.len(), 1);

    a.write(c.as_slice());
    assert_eq!(a.as_slice(), b"He11o W0rld!");
    assert_eq!(a.len(), 12);
}

fn grow_allocations<A: Allocator>(alloc: &A) {
    const BYTES: &[u8] = b"abcd";

    let mut a = BufVec::new_in(alloc);
    let mut b = BufVec::new_in(alloc);

    for _ in 0..1024 {
        assert!(a.write(BYTES));
        assert!(b.write(BYTES));
    }

    assert_eq!(a.len(), 1024 * 4);
    assert_eq!(b.len(), 1024 * 4);

    assert_eq!(a.as_slice(), b.as_slice());

    for n in 0..1024 {
        assert_eq!(&a.as_slice()[n * 4..n * 4 + 4], BYTES);
        assert_eq!(&b.as_slice()[n * 4..n * 4 + 4], BYTES);
    }

    drop(a);
    let mut c = BufVec::new_in(alloc);

    for _ in 0..1024 {
        assert!(c.write(BYTES));
    }

    assert_eq!(c.as_slice(), b.as_slice());

    for n in 0..1024 {
        assert_eq!(&c.as_slice()[n * 4..n * 4 + 4], BYTES);
        assert_eq!(&b.as_slice()[n * 4..n * 4 + 4], BYTES);
    }
}

#[test]
fn system_basic() {
    let alloc = super::System::new();
    basic_allocations(&alloc);
}

#[test]
fn system_grow() {
    let alloc = super::System::new();
    grow_allocations(&alloc);
}

#[test]
fn nostd_basic() {
    let mut buf = super::StackBuffer::<4096>::new();
    let alloc = super::Stack::new(&mut buf);
    basic_allocations(&alloc);
}
