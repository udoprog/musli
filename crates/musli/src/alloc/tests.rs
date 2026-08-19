use crate::alloc::Disabled;

use super::{Allocator, ArrayBuffer, Box, Global, Slice, String, Vec};

macro_rules! test_for_each {
    ($global:ident, $stack:ident, $inner:ident) => {
        #[test]
        fn $global() {
            let alloc = Global::new();
            $inner(alloc);
        }

        #[test]
        fn $stack() {
            let mut buf = ArrayBuffer::<16384>::with_size();
            let alloc = Slice::new(&mut buf);
            $inner(&alloc);
        }
    };
}

fn basic_allocations<A>(alloc: A)
where
    A: Copy + Allocator,
{
    let mut a = Vec::new_in(alloc);
    let mut b = Vec::new_in(alloc);

    b.extend_from_slice(b"He11o").unwrap();

    assert_eq!(b.as_slice(), b"He11o");
    assert_eq!(b.len(), 5);

    a.extend_from_slice(b.as_slice()).unwrap();

    assert_eq!(a.as_slice(), b"He11o");
    assert_eq!(a.len(), 5);

    a.extend_from_slice(b" W0rld").unwrap();

    assert_eq!(a.as_slice(), b"He11o W0rld");
    assert_eq!(a.len(), 11);

    let mut c = Vec::new_in(alloc);
    c.extend_from_slice(b"!").unwrap();
    assert_eq!(c.len(), 1);

    a.extend_from_slice(c.as_slice()).unwrap();
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

fn zst_allocations<A>(alloc: A)
where
    A: Copy + Allocator,
{
    let mut a = Vec::new_in(alloc);
    let mut b = Vec::new_in(alloc);

    assert_eq!(b.capacity(), usize::MAX);

    assert!(a.extend_from_slice(&[(); 100]).is_ok());
    assert!(b.extend_from_slice(&[(); 100]).is_ok());

    assert_eq!(a.len(), 100);
    assert_eq!(b.len(), 100);

    assert_eq!(a.as_slice(), b.as_slice());
}

/// Cloning a zero-sized allocation must not try to allocate.
#[test]
fn zst_box_clone() {
    let a = Box::new_in((), Global::new()).unwrap();
    let b = a.clone();
    assert_eq!(*b, ());
}

test_for_each!(global_basic, stack_basic, basic_allocations);
test_for_each!(global_grow, stack_grow, grow_allocations);
test_for_each!(global_zst, stack_zst, zst_allocations);

const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<String<Disabled>>();
    assert_send_sync::<Vec<u8, Disabled>>();
};

/// The capacity of a vector is expressed in elements, not in bytes.
#[test]
fn capacity_is_in_elements() {
    fn test<A>(alloc: A)
    where
        A: Copy + Allocator,
    {
        let mut a = Vec::<u32, _>::with_capacity_in(4, alloc).unwrap();
        let capacity = a.capacity();
        assert!(capacity >= 4);

        // Filling the vector up to its reported capacity must not require more
        // memory than has already been handed out.
        for n in 0..capacity {
            assert!(a.push(n as u32).is_ok(), "push {n} of {capacity} failed");
        }

        assert_eq!(a.len(), capacity);
    }

    test(Global::new());

    let mut buf = ArrayBuffer::<64>::with_size();
    let alloc = Slice::new(&mut buf);
    test(&alloc);
}
