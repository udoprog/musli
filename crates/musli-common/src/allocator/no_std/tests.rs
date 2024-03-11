use crate::allocator::{Allocator, Buf};

use super::{Header, Region, State};

macro_rules! assert_free {
    (
        $i:expr $(, $kind:ident [$($free:expr),* $(,)?])* $(,)?
    ) => {{
        $(
            let mut free = alloc::vec::Vec::<Region>::new();
            let mut current = $i.$kind;

            while let Some(c) = current.take() {
                free.push(c);
                current = $i.header(c).next_free;
            }

            assert_eq!(free, [$($free),*], "Expected `{}`", stringify!($kind));
        )*
    }};
}

macro_rules! assert_list {
    (
        $i:expr, $($free:expr),* $(,)?
    ) => {
        let mut expected = [$($free),*];

        {
            let mut list = alloc::vec::Vec::<Region>::new();
            let mut current = $i.head;

            while let Some(c) = current.take() {
                list.push(c);
                current = $i.header(c).next;
            }

            assert_eq!(list, expected, "Expected forward list");
        }

        {
            let mut list = alloc::vec::Vec::<Region>::new();
            let mut current = $i.tail;

            while let Some(c) = current.take() {
                list.push(c);
                current = $i.header(c).prev;
            }

            expected.reverse();
            assert_eq!(list, expected, "Expected reverse list");
        }
    };
}

macro_rules! assert_structure {
    (
        $list:expr,
        free [$($free:expr),* $(,)?],
        list [$($node:expr),* $(,)?],
        $($region:expr => {
            $start:expr,
            $len:expr,
            $cap:expr,
            $state:expr,
            next_free: $next_free:expr,
            prev: $prev:expr,
            next: $next:expr $(,)?
        },)* $(,)?
    ) => {{
        let i = unsafe { &*$list.internal.get() };

        $(
            assert_eq! {
                *i.header($region),
                Header {
                    start: $start,
                    len: $len,
                    cap: $cap,
                    state: $state,
                    next_free: $next_free,
                    prev: $prev,
                    next: $next,
                },
                "Comparing region {}", stringify!($region)
            };
        )*

        assert_free!(i, free[$($free),*]);
        assert_list!(i, $($node),*);
    }};
}

const A: Region = unsafe { Region::new_unchecked(1) };
const B: Region = unsafe { Region::new_unchecked(2) };
const C: Region = unsafe { Region::new_unchecked(3) };

#[test]
fn nostd_grow_last() {
    let mut buf = crate::allocator::StackBuffer::<4096>::new();
    let alloc = crate::allocator::NoStd::new(&mut buf);

    let a = alloc.alloc().unwrap();

    let mut b = alloc.alloc().unwrap();
    b.write(&[1, 2, 3, 4, 5, 6]);
    b.write(&[7, 8]);

    assert_structure! {
        alloc,
        free[], list[A, B],
        A => { 0, 0, 0, State::Used, next_free: None, prev: None, next: Some(B) },
        B => { 0, 8, 8, State::Used, next_free: None, prev: Some(A), next: None },
    };

    b.write(&[9, 10]);

    assert_structure! {
        alloc,
        free[], list[A, B],
        A => { 0, 0, 0, State::Used, next_free: None, prev: None, next: Some(B) },
        B => { 0, 10, 10, State::Used, next_free: None, prev: Some(A), next: None },
    };

    drop(a);
    drop(b);

    assert_structure! {
        alloc,
        free[A, B], list[],
        A => { 0, 0, 0, State::Free, next_free: Some(B), prev: None, next: None },
        B => { 0, 0, 0, State::Free, next_free: None, prev: None, next: None },
    };
}

#[test]
fn nostd_realloc() {
    let mut buf = crate::allocator::StackBuffer::<4096>::new();
    let alloc = crate::allocator::NoStd::new(&mut buf);

    let mut a = alloc.alloc().unwrap();
    a.write(&[1, 2, 3, 4]);
    assert_eq!(a.region.get(), A);

    let mut b = alloc.alloc().unwrap();
    b.write(&[1, 2, 3, 4]);
    assert_eq!(b.region.get(), B);

    let mut c = alloc.alloc().unwrap();
    c.write(&[1, 2, 3, 4]);
    assert_eq!(c.region.get(), C);

    assert_eq!(a.region.get(), A);
    assert_eq!(b.region.get(), B);
    assert_eq!(c.region.get(), C);

    assert_structure! {
        alloc,
        free[], list[A, B, C],
        A => { 0, 4, 4, State::Used, next_free: None, prev: None, next: Some(B) },
        B => { 4, 4, 4, State::Used, next_free: None, prev: Some(A), next: Some(C) },
        C => { 8, 4, 4, State::Used, next_free: None, prev: Some(B), next: None },
    };

    drop(a);

    assert_structure! {
        alloc,
        free[], list[A, B, C],
        A => { 0, 0, 4, State::Occupy, next_free: None, prev: None, next: Some(B) },
        B => { 4, 4, 4, State::Used, next_free: None, prev: Some(A), next: Some(C) },
        C => { 8, 4, 4, State::Used, next_free: None, prev: Some(B), next: None },
    };

    drop(b);

    assert_structure! {
        alloc,
        free[B], list[A, C],
        A => { 0, 0, 8, State::Occupy, next_free: None, prev: None, next: Some(C) },
        B => { 0, 0, 0, State::Free, next_free: None, prev: None, next: None },
        C => { 8, 4, 4, State::Used, next_free: None, prev: Some(A), next: None },
    };

    let mut d = alloc.alloc().unwrap();
    assert_eq!(d.region.get(), A);

    assert_structure! {
        alloc,
        free[B], list[A, C],
        A => { 0, 0, 8, State::Used, next_free: None, prev: None, next: Some(C) },
        B => { 0, 0, 0, State::Free, next_free: None, prev: None, next: None },
        C => { 8, 4, 4, State::Used, next_free: None, prev: Some(A), next: None },
    };

    d.write(&[1, 2]);
    assert_eq!(d.region.get(), A);

    assert_structure! {
        alloc,
        free[B], list[A, C],
        A => { 0, 2, 8, State::Used, next_free: None, prev: None, next: Some(C) },
        B => { 0, 0, 0, State::Free, next_free: None, prev: None, next: None },
        C => { 8, 4, 4, State::Used, next_free: None, prev: Some(A), next: None },
    };

    d.write(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    assert_eq!(d.region.get(), B);

    assert_structure! {
        alloc,
        free[], list[A, C, B],
        A => { 0, 0, 8, State::Occupy, next_free: None, prev: None, next: Some(C) },
        B => { 12, 18, 18, State::Used, next_free: None, prev: Some(C), next: None },
        C => { 8, 4, 4, State::Used, next_free: None, prev: Some(A), next: Some(B) },
    };
}

/// Empty regions will be automatically relinked to the end of the slab once
/// they're being written to.
#[test]
fn nostd_grow_empty_moved() {
    let mut buf = crate::allocator::StackBuffer::<4096>::new();
    let alloc = crate::allocator::NoStd::new(&mut buf);

    let mut a = alloc.alloc().unwrap();
    let b = alloc.alloc().unwrap();
    let mut c = alloc.alloc().unwrap();

    c.write(&[0]);
    a.write(&[1, 2, 3, 4]);

    assert_structure! {
        alloc,
        free[], list[B, C, A],
        A => { 1, 4, 4, State::Used, next_free: None, prev: Some(C), next: None },
        B => { 0, 0, 0, State::Used, next_free: None, prev: None, next: Some(C) },
        C => { 0, 1, 1, State::Used, next_free: None, prev: Some(B), next: Some(A) },
    };

    drop(c);

    assert_structure! {
        alloc,
        free[C], list[B, A],
        A => { 1, 4, 4, State::Used, next_free: None, prev: Some(B), next: None },
        B => { 0, 0, 1, State::Used, next_free: None, prev: None, next: Some(A) },
        C => { 0, 0, 0, State::Free, next_free: None, prev: None, next: None },
    };

    drop(b);

    assert_structure! {
        alloc,
        free[C], list[B, A],
        A => { 1, 4, 4, State::Used, next_free: None, prev: Some(B), next: None },
        B => { 0, 0, 1, State::Occupy, next_free: None, prev: None, next: Some(A) },
        C => { 0, 0, 0, State::Free, next_free: None, prev: None, next: None },
    };

    drop(a);

    assert_structure! {
        alloc,
        free[B, A, C], list[],
        A => { 0, 0, 0, State::Free, next_free: Some(C), prev: None, next: None },
        B => { 0, 0, 0, State::Free, next_free: Some(A), prev: None, next: None },
        C => { 0, 0, 0, State::Free, next_free: None, prev: None, next: None },
    };
}

/// Ensure that we support write buffer optimizations which allows for zero-copy
/// merging of buffers.
#[test]
fn nostd_write_buffer() {
    let mut buf = crate::allocator::StackBuffer::<4096>::new();
    let alloc = crate::allocator::NoStd::new(&mut buf);

    let mut a = alloc.alloc().unwrap();
    let mut b = alloc.alloc().unwrap();

    a.write(&[1, 2]);
    b.write(&[1, 2, 3, 4]);

    assert_structure! {
        alloc,
        free[], list[A, B],
        A => { 0, 2, 2, State::Used, next_free: None, prev: None, next: Some(B) },
        B => { 2, 4, 4, State::Used, next_free: None, prev: Some(A), next: None },
    };

    a.write_buffer(b);

    assert_structure! {
        alloc,
        free[B], list[A],
        A => { 0, 6, 6, State::Used, next_free: None, prev: None, next: None },
        B => { 0, 0, 0, State::Free, next_free: None, prev: None, next: None },
    };
}

/// Ensure that we support write buffer optimizations which allows for zero-copy
/// merging of buffers.
#[test]
fn nostd_write_buffer_middle() {
    let mut buf = crate::allocator::StackBuffer::<4096>::new();
    let alloc = crate::allocator::NoStd::new(&mut buf);

    let mut a = alloc.alloc().unwrap();
    let mut b = alloc.alloc().unwrap();
    let mut c = alloc.alloc().unwrap();

    a.write(&[1, 2]);
    b.write(&[1, 2, 3, 4]);
    c.write(&[1, 2, 3, 4]);

    assert_structure! {
        alloc,
        free[], list[A, B, C],
        A => { 0, 2, 2, State::Used, next_free: None, prev: None, next: Some(B) },
        B => { 2, 4, 4, State::Used, next_free: None, prev: Some(A), next: Some(C) },
        C => { 6, 4, 4, State::Used, next_free: None, prev: Some(B), next: None },
    };

    a.write_buffer(b);

    assert_structure! {
        alloc,
        free[B], list[A, C],
        A => { 0, 6, 6, State::Used, next_free: None, prev: None, next: Some(C) },
        B => { 0, 0, 0, State::Free, next_free: None, prev: None, next: None },
        C => { 6, 4, 4, State::Used, next_free: None, prev: Some(A), next: None },
    };
}
