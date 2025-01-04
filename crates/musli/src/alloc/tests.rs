use super::{Allocator, ArrayBuffer, Slice, System, Vec};

macro_rules! test_for_each {
    ($system:ident, $stack:ident, $inner:ident) => {
        #[test]
        fn $system() {
            let alloc = System::new();
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

test_for_each!(system_basic, stack_basic, basic_allocations);
test_for_each!(system_grow, stack_grow, grow_allocations);
test_for_each!(system_zst, stack_zst, zst_allocations);
