use super::{Allocator, Vec};

fn basic_allocations<A>(alloc: A)
where
    A: Copy + Allocator,
{
    let mut a = Vec::new_in(alloc);
    let mut b = Vec::new_in(alloc);

    _ = b.extend_from_slice(b"He11o");

    assert_eq!(b.as_slice(), b"He11o");
    assert_eq!(b.len(), 5);

    _ = a.extend_from_slice(b.as_slice());

    assert_eq!(a.as_slice(), b"He11o");
    assert_eq!(a.len(), 5);

    _ = a.extend_from_slice(b" W0rld");

    assert_eq!(a.as_slice(), b"He11o W0rld");
    assert_eq!(a.len(), 11);

    let mut c = Vec::new_in(alloc);
    _ = c.extend_from_slice(b"!");
    assert_eq!(c.len(), 1);

    _ = a.extend_from_slice(c.as_slice());
    assert_eq!(a.as_slice(), b"He11o W0rld!");
    assert_eq!(a.len(), 12);
}

fn grow_allocations<A>(alloc: A)
where
    A: Copy + Allocator,
{
    const BYTES: &[u8] = b"abcd";

    let mut a = Vec::new_in(alloc);
    let mut b = Vec::new_in(alloc);

    for _ in 0..1024 {
        assert!(a.extend_from_slice(BYTES).is_ok());
        assert!(b.extend_from_slice(BYTES).is_ok());
    }

    assert_eq!(a.len(), 1024 * 4);
    assert_eq!(b.len(), 1024 * 4);

    assert_eq!(a.as_slice(), b.as_slice());

    for n in 0..1024 {
        assert_eq!(&a.as_slice()[n * 4..n * 4 + 4], BYTES);
        assert_eq!(&b.as_slice()[n * 4..n * 4 + 4], BYTES);
    }

    drop(a);
    let mut c = Vec::new_in(alloc);

    for _ in 0..1024 {
        assert!(c.extend_from_slice(BYTES).is_ok());
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
    basic_allocations(alloc);
}

#[test]
fn nostd_basic() {
    let mut buf = super::ArrayBuffer::<4096>::new();
    let alloc = super::Slice::new(&mut buf);
    basic_allocations(&alloc);
}

#[test]
fn system_grow() {
    let alloc = super::System::new();
    grow_allocations(alloc);
}

fn zst_allocations<A>(alloc: A)
where
    A: Copy + Allocator,
{
    let mut a = Vec::new_in(alloc);
    let mut b = Vec::new_in(alloc);

    assert!(a.extend_from_slice(&[(); 100]).is_ok());
    assert!(b.extend_from_slice(&[(); 100]).is_ok());

    assert_eq!(a.len(), 100);
    assert_eq!(b.len(), 100);

    assert_eq!(a.as_slice(), b.as_slice());
}

#[test]
fn system_zst() {
    let alloc = super::System::new();
    zst_allocations(alloc);
}

#[test]
fn stack_zst() {
    let mut buf = super::ArrayBuffer::<4096>::new();
    let alloc = super::Slice::new(&mut buf);
    zst_allocations(&alloc);
}
