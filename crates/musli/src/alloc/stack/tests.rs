use std::collections::{BTreeSet, HashMap};
use std::fmt::{self, Write};
use std::string::String;
use std::vec::Vec as StdVec;

use crate::alloc::{Allocator, ArrayBuffer, Vec};

use super::{Header, HeaderId, Range, Slice};

const A: HeaderId = unsafe { HeaderId::new_unchecked(1) };
const B: HeaderId = unsafe { HeaderId::new_unchecked(2) };
const C: HeaderId = unsafe { HeaderId::new_unchecked(3) };
const D: HeaderId = unsafe { HeaderId::new_unchecked(4) };

fn to_ident(id: HeaderId) -> &'static Ident {
    Ident::new(match id {
        A => "A",
        B => "B",
        C => "C",
        D => "D",
        _ => unimplemented!("Unknown header id: {id:?}"),
    })
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
) -> StdVec<HeaderId>
where
    E: IntoIterator<Item = (&'static str, HeaderId)>,
    N: FnMut(HeaderId) -> Option<HeaderId>,
{
    let mut actual = StdVec::new();
    let mut actual_idents = StdVec::new();
    let mut expected_idents = StdVec::new();
    let mut it = expected.into_iter();

    let mut errors = StdVec::new();

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

        actual.push(c);
        actual_idents.push(to_ident(c));
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
        let actual = collect("free", $i.free_head, expected.iter().copied(), |c| $i.header(c).next);
        assert_eq!(actual, [$($free),*], "Expected `free` list");

        let expected: &'static [HeaderId] = &[$($free),*];
        expected
    }};
}

macro_rules! assert_list {
    ($i:expr $(, $node:expr)* $(,)?) => {{
        let expected: &'static [(&str, HeaderId)] = &[$((stringify!($node), $node)),*];
        let backward = collect("backward", $i.tail, expected.iter().rev().copied(), |c| $i.header(c).prev);
        let forward = collect("forward", backward.last().copied(), expected.iter().copied(), |c| $i.header(c).next);
        assert!(forward.iter().eq(backward.iter().rev()), "The forward and backward lists should match");

        let expected: &'static [HeaderId] = &[$($node),*];
        expected
    }};
}

macro_rules! assert_structure {
    (
        $list:expr,
        occupied [$($occupied:expr),* $(,)?],
        free [$($free:expr),* $(,)?],
        list [$($node:expr),* $(,)?]
        $(, $region:expr => { $start:expr, $len:expr, $cap:expr })* $(,)?
    ) => {{
        let i = unsafe { &*$list.internal.get() };

        let occupied = [$($occupied)*];

        if i.occupied.as_slice() != occupied {
            let actual = i.occupied.map(to_ident);
            let actual = actual.as_slice();
            let expected: &[&'static Ident] = &[$(to_ident($occupied))*];
            panic!("Expected occupied node to be {expected:?} but was {actual:?}");
        }

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
                    range: Range::new(unsafe { i.full.start.add($start)..i.full.start.add($start + $cap) }),
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
                    range: i.full.head(),
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
    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);

    let a = Vec::<u8, _>::new_in(&alloc);

    let mut b = Vec::<u8, _>::new_in(&alloc);
    _ = b.extend_from_slice(&[1, 2, 3, 4, 5, 6]);
    _ = b.extend_from_slice(&[7, 8]);

    assert_structure! {
        alloc, occupied[], free[], list[A, B],
        A => { 0, 0, 0 },
        B => { 0, 8, 8 },
    };

    _ = b.extend_from_slice(&[9, 10]);

    assert_structure! {
        alloc, occupied[], free[], list[A, B],
        A => { 0, 0, 0 },
        B => { 0, 10, 10 },
    };

    drop(a);
    drop(b);

    assert_structure! {
        alloc, occupied[], free[A, B], list[]
    };
}

#[test]
fn realloc() {
    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);

    let mut a = Vec::<u8, _>::new_in(&alloc);
    _ = a.extend_from_slice(&[1, 2, 3, 4]);
    assert_eq!(a.raw().region, Some(A));

    let mut b = Vec::<u8, _>::new_in(&alloc);
    _ = b.extend_from_slice(&[1, 2, 3, 4]);
    assert_eq!(b.raw().region, Some(B));

    let mut c = Vec::<u8, _>::new_in(&alloc);
    _ = c.extend_from_slice(&[1, 2, 3, 4]);
    assert_eq!(c.raw().region, Some(C));

    assert_eq!(a.raw().region, Some(A));
    assert_eq!(b.raw().region, Some(B));
    assert_eq!(c.raw().region, Some(C));

    assert_structure! {
        alloc, occupied[], free[], list[A, B, C],
        A => { 0, 4, 4 },
        B => { 4, 4, 4 },
        C => { 8, 4, 4 },
    };

    drop(a);

    assert_structure! {
        alloc, occupied[A], free[], list[A, B, C],
        A => { 0, 0, 4 },
        B => { 4, 4, 4 },
        C => { 8, 4, 4 },
    };

    drop(b);

    assert_structure! {
        alloc, occupied[A], free[B], list[A, C],
        A => { 0, 0, 8 },
        C => { 8, 4, 4 },
    };

    let mut d = Vec::<u8, _>::new_in(&alloc);
    assert_eq!(d.raw().region, Some(A));

    assert_structure! {
        alloc, occupied[], free[B], list[A, C],
        A => { 0, 0, 8 },
        C => { 8, 4, 4 },
    };

    _ = d.extend_from_slice(&[1, 2]);
    assert_eq!(d.raw().region, Some(A));

    assert_structure! {
        alloc, occupied[], free[B], list[A, C],
        A => { 0, 2, 8 },
        C => { 8, 4, 4 },
    };

    _ = d.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    assert_eq!(d.raw().region, Some(B));

    assert_structure! {
        alloc, occupied[A], free[], list[A, C, B],
        A => { 0, 0, 8 },
        B => { 12, 18, 18 },
        C => { 8, 4, 4 },
    };
}

/// Empty regions will be automatically relinked to the end of the slab once
/// they're being written to.
#[test]
fn grow_empty_moved() {
    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);

    let mut a = Vec::<u8, _>::new_in(&alloc);
    let b = Vec::<u8, _>::new_in(&alloc);
    let mut c = Vec::<u8, _>::new_in(&alloc);

    _ = c.extend_from_slice(&[0]);
    _ = a.extend_from_slice(&[1, 2, 3, 4]);

    assert_structure! {
        alloc, occupied[], free[], list[B, C, A],
        A => { 1, 4, 4 },
        B => { 0, 0, 0 },
        C => { 0, 1, 1 },
    };

    drop(c);

    assert_structure! {
        alloc, occupied[], free[C], list[B, A],
        A => { 1, 4, 4 },
        B => { 0, 0, 1 },
        C => { 0, 0, 0 },
    };

    drop(b);

    assert_structure! {
        alloc, occupied[B], free[C], list[B, A],
        A => { 1, 4, 4 },
        B => { 0, 0, 1 },
        C => { 0, 0, 0 },
    };

    drop(a);

    assert_structure! {
        alloc, occupied[], free[B, A, C], list[],
        A => { 0, 0, 0 },
        B => { 0, 0, 0 },
        C => { 0, 0, 0 },
    };
}

/// Ensure that we support write buffer optimizations which allows for zero-copy
/// merging of buffers.
#[test]
fn extend() {
    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);

    let mut a = Vec::<u8, _>::new_in(&alloc);
    let mut b = Vec::<u8, _>::new_in(&alloc);

    _ = a.extend_from_slice(&[1, 2]);
    _ = b.extend_from_slice(&[1, 2, 3, 4]);

    assert_structure! {
        alloc, occupied[], free[], list[A, B],
        A => { 0, 2, 2 },
        B => { 2, 4, 4 },
    };

    _ = a.extend(b);

    assert_structure! {
        alloc, occupied[], free[B], list[A],
        A => { 0, 6, 6 },
        B => { 0, 0, 0 },
    };
}

/// Ensure that we support write buffer optimizations which allows for zero-copy
/// merging of buffers.
#[test]
fn extend_middle() {
    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);

    let mut a = Vec::<u8, _>::new_in(&alloc);
    let mut b = Vec::<u8, _>::new_in(&alloc);
    let mut c = Vec::<u8, _>::new_in(&alloc);

    _ = a.extend_from_slice(&[1, 2]);
    _ = b.extend_from_slice(&[1, 2, 3, 4]);
    _ = c.extend_from_slice(&[1, 2, 3, 4]);

    assert_structure! {
        alloc, occupied[], free[], list[A, B, C],
        A => { 0, 2, 2 },
        B => { 2, 4, 4 },
        C => { 6, 4, 4 },
    };

    _ = a.extend(b);

    assert_structure! {
        alloc, occupied[], free[B], list[A, C],
        A => { 0, 6, 6 },
        B => { 0, 0, 0 },
        C => { 6, 4, 4 },
    };
}

/// Ensure that we support write buffer optimizations which allows for zero-copy
/// merging of buffers.
#[test]
fn extend_gap() {
    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);

    let mut a = Vec::<u8, _>::new_in(&alloc);
    let mut b = Vec::<u8, _>::new_in(&alloc);
    let mut c = Vec::<u8, _>::new_in(&alloc);

    _ = a.extend_from_slice(&[1, 2]);
    _ = b.extend_from_slice(&[7, 8, 9, 10]);
    _ = c.extend_from_slice(&[3, 4, 5, 6]);

    assert_structure! {
        alloc, occupied[], free[], list[A, B, C],
        A => { 0, 2, 2 },
        B => { 2, 4, 4 },
        C => { 6, 4, 4 },
    };

    drop(b);
    _ = a.extend(c);

    assert_structure! {
        alloc, occupied[], free[C, B], list[A],
        A => { 0, 6, 10 },
        B => { 0, 0, 0 },
        C => { 0, 0, 0 },
    };

    assert_eq!(a.as_slice(), &[1, 2, 3, 4, 5, 6]);
}

/// Hold onto a slice while we grow another buffer to make sure MIRI doesn't get
/// unhappy about it.
#[test]
fn test_overlapping_slice_miri() {
    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);

    let mut a = Vec::<u8, _>::new_in(&alloc);
    _ = a.extend_from_slice(&[1, 2, 3, 4]);
    let a_slice = a.as_slice();

    let mut b = Vec::<u8, _>::new_in(&alloc);
    _ = b.extend_from_slice(&[5, 6, 7, 8]);
    let b_slice = b.as_slice();

    assert_eq!(a_slice, &[1, 2, 3, 4]);
    assert_eq!(b_slice, &[5, 6, 7, 8]);
}

/// Test when we have a prior allocation that has been freed and we can grow into it.
#[test]
fn grow_into_preceeding() {
    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);

    let mut a = Vec::<u8, _>::new_in(&alloc);
    _ = a.extend_from_slice(&[0]);

    let mut b = Vec::<u8, _>::new_in(&alloc);
    _ = b.extend_from_slice(&[1]);

    let mut c = Vec::<u8, _>::new_in(&alloc);
    _ = c.extend_from_slice(&[2]);

    let mut d = Vec::<u8, _>::new_in(&alloc);
    _ = d.extend_from_slice(&[3]);

    drop(a);

    assert_structure! {
        alloc, occupied[A], free[], list[A, B, C, D],
        A => { 0, 0, 1 },
        B => { 1, 1, 1 },
        C => { 2, 1, 1 },
        D => { 3, 1, 1 },
    };

    _ = b.extend_from_slice(&[2]);

    assert_structure! {
        alloc, occupied[], free[B], list[A, C, D],
        A => { 0, 2, 2 },
        C => { 2, 1, 1 },
        D => { 3, 1, 1 },
    };
}

/// Test when we have a prior allocation that has been freed and we can grow into it.
#[test]
fn flip_flop() {
    let mut buf = ArrayBuffer::new();
    let alloc = Slice::new(&mut buf);

    let mut a = Vec::<u8, _>::new_in(&alloc);
    let mut b = Vec::<u8, _>::new_in(&alloc);

    _ = a.extend_from_slice(&[0]);
    _ = b.extend_from_slice(&[0]);

    assert_structure! {
        alloc, occupied[], free[], list[A, B],
        A => { 0, 1, 1 },
        B => { 1, 1, 1 },
    };

    _ = a.extend_from_slice(&[1]);
    assert_eq!(a.raw().region, Some(C));

    assert_structure! {
        alloc, occupied[A], free[], list[A, B, C],
        A => { 0, 0, 1 },
        B => { 1, 1, 1 },
        C => { 2, 2, 2 },
    };

    _ = b.extend_from_slice(&[1]);
    assert_eq!(b.raw().region, Some(A));

    assert_structure! {
        alloc, occupied[], free[B], list[A, C],
        A => { 0, 2, 2 },
        C => { 2, 2, 2 },
    };

    _ = a.extend_from_slice(&[2]);
    assert_eq!(a.raw().region, Some(C));

    assert_structure! {
        alloc, occupied[], free[B], list[A, C],
        A => { 0, 2, 2 },
        C => { 2, 3, 3 },
    };

    _ = b.extend_from_slice(&[2]);
    assert_eq!(b.raw().region, Some(B));

    assert_structure! {
        alloc, occupied[A], free[], list[A, C, B],
        A => { 0, 0, 2 },
        C => { 2, 3, 3 },
        B => { 5, 3, 3 },
    };

    _ = a.extend_from_slice(&[3]);
    assert_eq!(a.raw().region, Some(A));

    assert_structure! {
        alloc, occupied[], free[C], list[A, B],
        A => { 0, 4, 5 },
        B => { 5, 3, 3 },
    };

    _ = b.extend_from_slice(&[3]);
    assert_eq!(b.raw().region, Some(B));

    assert_structure! {
        alloc, occupied[], free[C], list[A, B],
        A => { 0, 4, 5 },
        B => { 5, 4, 4 },
    };

    assert_eq!(a.as_slice(), &[0, 1, 2, 3]);
    assert_eq!(b.as_slice(), &[0, 1, 2, 3]);
}

/// Test when we have a prior allocation that has been freed and we can grow into it.
#[test]
fn limits() {
    let mut buf = ArrayBuffer::<8>::with_size();
    let alloc = Slice::new(&mut buf);
    assert!(alloc.alloc_slice::<u8>().region.is_none());

    let mut buf = ArrayBuffer::<32>::with_size();
    let alloc = Slice::new(&mut buf);

    let mut a = Vec::<u8, _>::new_in(&alloc);
    assert!(a.extend_from_slice(&[0, 1, 2, 3, 4, 5, 6, 7]).is_ok());

    assert_structure! {
        alloc, occupied[], free[], list[A],
        A => { 0, 8, 8 },
    };

    assert!(a.extend_from_slice(&[0]).is_err());
}
