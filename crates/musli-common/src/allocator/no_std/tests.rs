use crate::allocator::{Allocator, Buf};

use super::{Header, NoStd, Region, State};

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
            $size:expr,
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
                    size: $size,
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

fn grow_last(alloc: &NoStd<'_>) {
    let a = alloc.alloc().unwrap();

    let mut b = alloc.alloc().unwrap();
    b.write(&[1, 2, 3, 4, 5, 6]);
    b.write(&[7, 8]);

    assert_structure! {
        alloc,
        free[], list[A, B],
        A => { 0, 0, State::Used, next_free: None, prev: None, next: Some(B) },
        B => { 0, 8, State::Used, next_free: None, prev: Some(A), next: None },
    };

    b.write(&[9, 10]);

    assert_structure! {
        alloc,
        free[], list[A, B],
        A => { 0, 0, State::Used, next_free: None, prev: None, next: Some(B) },
        B => { 0, 10, State::Used, next_free: None, prev: Some(A), next: None },
    };

    drop(a);
    drop(b);

    assert_structure! {
        alloc,
        free[A, B], list[],
        A => { 0, 0, State::Free, next_free: Some(B), prev: None, next: None },
        B => { 0, 0, State::Free, next_free: None, prev: None, next: None },
    };
}

#[test]
fn nostd_grow_last() {
    let mut buf = crate::allocator::StackBuffer::<4096>::new();
    let alloc = crate::allocator::NoStd::new(&mut buf);
    grow_last(&alloc);
}

fn realloc(alloc: &NoStd<'_>) {
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
        A => { 0, 4, State::Used, next_free: None, prev: None, next: Some(B) },
        B => { 4, 4, State::Used, next_free: None, prev: Some(A), next: Some(C) },
        C => { 8, 4, State::Used, next_free: None, prev: Some(B), next: None },
    };

    drop(a);

    assert_structure! {
        alloc,
        free[], list[A, B, C],
        A => { 0, 4, State::Occupy, next_free: None, prev: None, next: Some(B) },
        B => { 4, 4, State::Used, next_free: None, prev: Some(A), next: Some(C) },
        C => { 8, 4, State::Used, next_free: None, prev: Some(B), next: None },
    };

    drop(b);

    assert_structure! {
        alloc,
        free[B], list[A, C],
        A => { 0, 8, State::Occupy, next_free: None, prev: None, next: Some(C) },
        B => { 0, 0, State::Free, next_free: None, prev: None, next: None },
        C => { 8, 4, State::Used, next_free: None, prev: Some(A), next: None },
    };

    let mut d = alloc.alloc().unwrap();

    assert_structure! {
        alloc,
        free[B], list[A, C],
        A => { 0, 8, State::Used, next_free: None, prev: None, next: Some(C) },
        B => { 0, 0, State::Free, next_free: None, prev: None, next: None },
        C => { 8, 4, State::Used, next_free: None, prev: Some(A), next: None },
    };

    d.write(&[1, 2]);
    assert_eq!(d.region.get(), A);

    assert_structure! {
        alloc,
        free[B], list[A, C],
        A => { 0, 8, State::Used, next_free: None, prev: None, next: Some(C) },
        B => { 0, 0, State::Free, next_free: None, prev: None, next: None },
        C => { 8, 4, State::Used, next_free: None, prev: Some(A), next: None },
    };

    d.write(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    assert_eq!(d.region.get(), B);

    assert_structure! {
        alloc,
        free[], list[A, C, B],
        A => { 0, 8, State::Occupy, next_free: None, prev: None, next: Some(C) },
        B => { 12, 18, State::Used, next_free: None, prev: Some(C), next: None },
        C => { 8, 4, State::Used, next_free: None, prev: Some(A), next: Some(B) },
    };
}

#[test]
fn nostd_realloc() {
    let mut buf = crate::allocator::StackBuffer::<4096>::new();
    let alloc = crate::allocator::NoStd::new(&mut buf);
    realloc(&alloc);
}
