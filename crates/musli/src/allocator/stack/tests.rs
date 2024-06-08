use std::collections::{BTreeSet, HashMap};
use std::fmt::{self, Write};
use std::string::String;
use std::vec::Vec;

use crate::buf::BytesBuf;
use crate::Allocator;

use super::{Header, HeaderId, Stack, StackBuffer, State};

const A: HeaderId = unsafe { HeaderId::new_unchecked(1) };
const B: HeaderId = unsafe { HeaderId::new_unchecked(2) };
const C: HeaderId = unsafe { HeaderId::new_unchecked(3) };
const D: HeaderId = unsafe { HeaderId::new_unchecked(4) };

fn to_ident(id: HeaderId) -> Option<&'static Ident> {
    Some(Ident::new(match id {
        A => "A",
        B => "B",
        C => "C",
        D => "D",
        _ => return None,
    }))
}

#[repr(transparent)]
struct Ident(str);

impl Ident {
    const fn new(string: &str) -> &Self {
        // SAFETY: Ident is repr transparent.
        unsafe { &*(string as *const _ as *const Self) }
    }
}

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Collect actual nodes and assert that they match the provided structure.
#[track_caller]
fn collect<E, N>(
    what: &'static str,
    mut current: Option<HeaderId>,
    expected: E,
    mut next: N,
) -> Vec<HeaderId>
where
    E: IntoIterator<Item = (&'static str, HeaderId)>,
    N: FnMut(HeaderId) -> Option<HeaderId>,
{
    let mut actual = Vec::new();
    let mut actual_idents = Vec::new();
    let mut expected_idents = Vec::new();
    let mut it = expected.into_iter();

    let mut errors = Vec::new();

    loop {
        let expected = it.next();

        let expected_name = expected.map(|(n, _)| Ident::new(n));
        let expected_node = expected.map(|(_, n)| n);

        expected_idents.extend(expected_name);

        if current != expected_node {
            errors.push((expected_name, actual.len() + 1));
        }

        let Some(c) = current.take() else {
            break;
        };

        let Some(ident) = to_ident(c) else {
            panic!("Got unexpected header: {c:?}");
        };

        actual.push(c);
        actual_idents.push(ident);
        current = next(c);
    }

    if !errors.is_empty() {
        let mut s = String::new();

        writeln!(s, "Error in `{what}` list").unwrap();

        for (expected_name, at) in errors {
            writeln!(s, "Expected element #{at} {expected_name:?}` list").unwrap();
        }

        writeln!(s, "Actual list: {actual_idents:?}").unwrap();
        writeln!(s, "Expected list: {expected_idents:?}").unwrap();
        panic!("{s}")
    }

    actual
}

macro_rules! assert_free {
    ($i:expr $(, $free:expr)* $(,)?) => {{
        let expected: &'static [(&str, HeaderId)] = &[$((stringify!($free), $free)),*];
        let actual = collect("free", $i.free, expected.iter().copied(), |c| $i.header(c).next);
        assert_eq!(actual, [$($free),*], "Expected `free` list");

        let expected: &'static [HeaderId] = &[$($free),*];
        expected
    }};
}

macro_rules! assert_list {
    ($i:expr $(, $node:expr)* $(,)?) => {{
        let expected: &'static [(&str, HeaderId)] = &[$((stringify!($node), $node)),*];
        let forward = collect("forward", $i.head, expected.iter().copied(), |c| $i.header(c).next);
        let backward = collect("backward", $i.tail, expected.iter().rev().copied(), |c| $i.header(c).prev);
        assert!(forward.iter().eq(backward.iter().rev()), "The forward and backward lists should match");

        let expected: &'static [HeaderId] = &[$($node),*];
        expected
    }};
}

macro_rules! assert_structure {
    (
        $list:expr, free [$($free:expr),* $(,)?],
        list [$($node:expr),* $(,)?]
        $(, $region:expr => { $start:expr, $len:expr, $cap:expr, $state:ident $(,)? })* $(,)?
    ) => {{
        let i = unsafe { &*$list.internal.get() };

        let free = assert_free!(i $(, $free)*);
        let list = assert_list!(i $(, $node)*);

        let expected_bytes = (0usize $(+ (*i.header($region)).capacity())*);

        assert_eq!(i.bytes(), expected_bytes, "The number of bytes allocated should match");
        assert_eq!(i.headers(), free.len() + list.len(), "The number of headers should match");

        let mut forward = HashMap::new();
        let mut backward = HashMap::new();

        for pair in list.windows(2) {
            forward.insert(pair[0], pair[1]);
            backward.insert(pair[1], pair[0]);
        }

        for pair in free.windows(2) {
            assert!(forward.insert(pair[0], pair[1]).is_none());
        }

        $(
            assert_eq! {
                *i.header($region),
                Header {
                    start: unsafe { i.start.add($start) },
                    end: unsafe { i.start.add($start + $cap) },
                    state: State::$state,
                    next: forward.get(&$region).copied(),
                    prev: backward.get(&$region).copied(),
                },
                "Comparing region `{}`", stringify!($region)
            };
        )*

        let mut unused = BTreeSet::new();

        $(unused.insert($region);)*
        $(unused.remove(&$node);)*

        let _ = &mut unused;

        for node in unused {
            assert_eq! {
                *i.header(node),
                Header {
                    start: i.start,
                    end: i.start,
                    state: State::Free,
                    next: forward.get(&node).copied(),
                    prev: backward.get(&node).copied(),
                },
                "Expected region {:?} to be free", node
            };
        }
    }};
}

#[test]
fn grow_last() {
    let mut buf = StackBuffer::<4096>::new();
    let alloc = Stack::new(&mut buf);

    let a = BytesBuf::new(alloc.alloc().unwrap());

    let mut b = BytesBuf::new(alloc.alloc().unwrap());
    b.write(&[1, 2, 3, 4, 5, 6]);
    b.write(&[7, 8]);

    assert_structure! {
        alloc, free[], list[A, B],
        A => { 0, 0, 0, Used },
        B => { 0, 8, 8, Used },
    };

    b.write(&[9, 10]);

    assert_structure! {
        alloc, free[], list[A, B],
        A => { 0, 0, 0, Used },
        B => { 0, 10, 10, Used },
    };

    drop(a);
    drop(b);

    assert_structure! {
        alloc, free[A, B], list[]
    };
}

#[test]
fn realloc() {
    let mut buf = StackBuffer::<4096>::new();
    let alloc = Stack::new(&mut buf);

    let mut a = BytesBuf::new(alloc.alloc().unwrap());
    a.write(&[1, 2, 3, 4]);
    assert_eq!(a.region, A);

    let mut b = BytesBuf::new(alloc.alloc().unwrap());
    b.write(&[1, 2, 3, 4]);
    assert_eq!(b.region, B);

    let mut c = BytesBuf::new(alloc.alloc().unwrap());
    c.write(&[1, 2, 3, 4]);
    assert_eq!(c.region, C);

    assert_eq!(a.region, A);
    assert_eq!(b.region, B);
    assert_eq!(c.region, C);

    assert_structure! {
        alloc, free[], list[A, B, C],
        A => { 0, 4, 4, Used },
        B => { 4, 4, 4, Used },
        C => { 8, 4, 4, Used },
    };

    drop(a);

    assert_structure! {
        alloc, free[], list[A, B, C],
        A => { 0, 0, 4, Occupy },
        B => { 4, 4, 4, Used },
        C => { 8, 4, 4, Used },
    };

    drop(b);

    assert_structure! {
        alloc, free[B], list[A, C],
        A => { 0, 0, 8, Occupy },
        C => { 8, 4, 4, Used },
    };

    let mut d = BytesBuf::new(alloc.alloc().unwrap());
    assert_eq!(d.region, A);

    assert_structure! {
        alloc, free[B], list[A, C],
        A => { 0, 0, 8, Used },
        C => { 8, 4, 4, Used },
    };

    d.write(&[1, 2]);
    assert_eq!(d.region, A);

    assert_structure! {
        alloc, free[B], list[A, C],
        A => { 0, 2, 8, Used },
        C => { 8, 4, 4, Used },
    };

    d.write(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    assert_eq!(d.region, B);

    assert_structure! {
        alloc, free[], list[A, C, B],
        A => { 0, 0, 8, Occupy },
        B => { 12, 18, 18, Used },
        C => { 8, 4, 4, Used },
    };
}

/// Empty regions will be automatically relinked to the end of the slab once
/// they're being written to.
#[test]
fn grow_empty_moved() {
    let mut buf = StackBuffer::<4096>::new();
    let alloc = Stack::new(&mut buf);

    let mut a = BytesBuf::new(alloc.alloc().unwrap());
    let b = BytesBuf::new(alloc.alloc().unwrap());
    let mut c = BytesBuf::new(alloc.alloc().unwrap());

    c.write(&[0]);
    a.write(&[1, 2, 3, 4]);

    assert_structure! {
        alloc, free[], list[B, C, A],
        A => { 1, 4, 4, Used },
        B => { 0, 0, 0, Used },
        C => { 0, 1, 1, Used },
    };

    drop(c);

    assert_structure! {
        alloc, free[C], list[B, A],
        A => { 1, 4, 4, Used },
        B => { 0, 0, 1, Used },
        C => { 0, 0, 0, Free },
    };

    drop(b);

    assert_structure! {
        alloc, free[C], list[B, A],
        A => { 1, 4, 4, Used },
        B => { 0, 0, 1, Occupy },
        C => { 0, 0, 0, Free },
    };

    drop(a);

    assert_structure! {
        alloc, free[B, A, C], list[],
        A => { 0, 0, 0, Free },
        B => { 0, 0, 0, Free },
        C => { 0, 0, 0, Free },
    };
}

/// Ensure that we support write buffer optimizations which allows for zero-copy
/// merging of buffers.
#[test]
fn extend() {
    let mut buf = StackBuffer::<4096>::new();
    let alloc = Stack::new(&mut buf);

    let mut a = BytesBuf::new(alloc.alloc().unwrap());
    let mut b = BytesBuf::new(alloc.alloc().unwrap());

    a.write(&[1, 2]);
    b.write(&[1, 2, 3, 4]);

    assert_structure! {
        alloc, free[], list[A, B],
        A => { 0, 2, 2, Used },
        B => { 2, 4, 4, Used },
    };

    a.extend(b);

    assert_structure! {
        alloc, free[B], list[A],
        A => { 0, 6, 6, Used },
        B => { 0, 0, 0, Free },
    };
}

/// Ensure that we support write buffer optimizations which allows for zero-copy
/// merging of buffers.
#[test]
fn extend_middle() {
    let mut buf = StackBuffer::<4096>::new();
    let alloc = Stack::new(&mut buf);

    let mut a = BytesBuf::new(alloc.alloc().unwrap());
    let mut b = BytesBuf::new(alloc.alloc().unwrap());
    let mut c = BytesBuf::new(alloc.alloc().unwrap());

    a.write(&[1, 2]);
    b.write(&[1, 2, 3, 4]);
    c.write(&[1, 2, 3, 4]);

    assert_structure! {
        alloc, free[], list[A, B, C],
        A => { 0, 2, 2, Used },
        B => { 2, 4, 4, Used },
        C => { 6, 4, 4, Used },
    };

    a.extend(b);

    assert_structure! {
        alloc, free[B], list[A, C],
        A => { 0, 6, 6, Used },
        B => { 0, 0, 0, Free },
        C => { 6, 4, 4, Used },
    };
}

/// Ensure that we support write buffer optimizations which allows for zero-copy
/// merging of buffers.
#[test]
fn extend_gap() {
    let mut buf = StackBuffer::<4096>::new();
    let alloc = Stack::new(&mut buf);

    let mut a = BytesBuf::new(alloc.alloc().unwrap());
    let mut b = BytesBuf::new(alloc.alloc().unwrap());
    let mut c = BytesBuf::new(alloc.alloc().unwrap());

    a.write(&[1, 2]);
    b.write(&[7, 8, 9, 10]);
    c.write(&[3, 4, 5, 6]);

    assert_structure! {
        alloc, free[], list[A, B, C],
        A => { 0, 2, 2, Used },
        B => { 2, 4, 4, Used },
        C => { 6, 4, 4, Used },
    };

    drop(b);
    a.extend(c);

    assert_structure! {
        alloc, free[C, B], list[A],
        A => { 0, 6, 10, Used },
        B => { 0, 0, 0, Free },
        C => { 0, 0, 0, Free },
    };

    assert_eq!(a.as_slice(), &[1, 2, 3, 4, 5, 6]);
}

/// Hold onto a slice while we grow another buffer to make sure MIRI doesn't get
/// unhappy about it.
#[test]
fn test_overlapping_slice_miri() {
    let mut buf = StackBuffer::<4096>::new();
    let alloc = Stack::new(&mut buf);

    let mut a = BytesBuf::new(alloc.alloc().unwrap());
    a.write(&[1, 2, 3, 4]);
    let a_slice = a.as_slice();

    let mut b = BytesBuf::new(alloc.alloc().unwrap());
    b.write(&[5, 6, 7, 8]);
    let b_slice = b.as_slice();

    assert_eq!(a_slice, &[1, 2, 3, 4]);
    assert_eq!(b_slice, &[5, 6, 7, 8]);
}

/// Test when we have a prior allocation that has been freed and we can grow into it.
#[test]
fn grow_into_preceeding() {
    let mut buf = StackBuffer::<4096>::new();
    let alloc = Stack::new(&mut buf);

    let mut a = BytesBuf::new(alloc.alloc().unwrap());
    a.write(&[0]);

    let mut b = BytesBuf::new(alloc.alloc().unwrap());
    b.write(&[1]);

    let mut c = BytesBuf::new(alloc.alloc().unwrap());
    c.write(&[2]);

    let mut d = BytesBuf::new(alloc.alloc().unwrap());
    d.write(&[3]);

    drop(a);

    assert_structure! {
        alloc, free[], list[A, B, C, D],
        A => { 0, 0, 1, Occupy },
        B => { 1, 1, 1, Used },
        C => { 2, 1, 1, Used },
        D => { 3, 1, 1, Used },
    };

    b.write(&[2]);

    assert_structure! {
        alloc, free[B], list[A, C, D],
        A => { 0, 2, 2, Used },
        C => { 2, 1, 1, Used },
        D => { 3, 1, 1, Used },
    };
}

/// Test when we have a prior allocation that has been freed and we can grow into it.
#[test]
fn flip_flop() {
    let mut buf = StackBuffer::<4096>::new();
    let alloc = Stack::new(&mut buf);

    let mut a = BytesBuf::new(alloc.alloc().unwrap());
    let mut b = BytesBuf::new(alloc.alloc().unwrap());

    a.write(&[0]);
    b.write(&[0]);

    assert_structure! {
        alloc, free[], list[A, B],
        A => { 0, 1, 1, Used },
        B => { 1, 1, 1, Used },
    };

    a.write(&[1]);
    assert_eq!(a.region, C);

    assert_structure! {
        alloc, free[], list[A, B, C],
        A => { 0, 0, 1, Occupy },
        B => { 1, 1, 1, Used },
        C => { 2, 2, 2, Used },
    };

    b.write(&[1]);
    assert_eq!(b.region, A);

    assert_structure! {
        alloc, free[B], list[A, C],
        A => { 0, 2, 2, Used },
        C => { 2, 2, 2, Used },
    };

    a.write(&[2]);
    assert_eq!(a.region, C);

    assert_structure! {
        alloc, free[B], list[A, C],
        A => { 0, 2, 2, Used },
        C => { 2, 3, 3, Used },
    };

    b.write(&[2]);
    assert_eq!(b.region, B);

    assert_structure! {
        alloc, free[], list[A, C, B],
        A => { 0, 0, 2, Occupy },
        C => { 2, 3, 3, Used },
        B => { 5, 3, 3, Used },
    };

    a.write(&[3]);
    assert_eq!(a.region, A);

    assert_structure! {
        alloc, free[C], list[A, B],
        A => { 0, 4, 5, Used },
        B => { 5, 3, 3, Used },
    };

    b.write(&[3]);
    assert_eq!(b.region, B);

    assert_structure! {
        alloc, free[C], list[A, B],
        A => { 0, 4, 5, Used },
        B => { 5, 4, 4, Used },
    };

    assert_eq!(a.as_slice(), &[0, 1, 2, 3]);
    assert_eq!(b.as_slice(), &[0, 1, 2, 3]);
}

/// Test when we have a prior allocation that has been freed and we can grow into it.
#[test]
fn limits() {
    let mut buf = StackBuffer::<8>::new();
    let alloc = Stack::new(&mut buf);
    assert!(alloc.alloc().is_none());

    let mut buf = StackBuffer::<32>::new();
    let alloc = Stack::new(&mut buf);

    let mut a = BytesBuf::new(alloc.alloc().unwrap());
    assert!(a.write(&[0, 1, 2, 3, 4, 5, 6, 7]));

    assert_structure! {
        alloc, free[], list[A],
        A => { 0, 8, 8, Used },
    };

    assert!(!a.write(&[0]));
}
