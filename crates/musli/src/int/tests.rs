use std::fmt;
use std::vec::Vec;

use rust_alloc::vec;

use crate::context;
use crate::fixed::FixedBytes;
use crate::int::continuation as c;
use crate::int::zigzag as zig;
use crate::int::{Signed, Unsigned};

#[cfg(not(miri))]
const ITER: usize = 10000;
#[cfg(miri)]
const ITER: usize = 100;

#[test]
fn basic_continuation() {
    crate::alloc::default!(|alloc| {
        let cx = context::Ignore::with_marker(&alloc);
        let mut bytes = FixedBytes::<8>::new();
        c::encode(&cx, &mut bytes, 5000u32).unwrap();
        assert_eq!(bytes.as_slice(), &[0b1000_1000, 0b0010_0111]);

        let cx = context::Ignore::with_marker(&alloc);
        let number: u32 = c::decode(&cx, bytes.as_slice()).unwrap();
        assert_eq!(number, 5000u32);
    })
}

#[test]
fn test_continuation_encoding() {
    use rand::prelude::*;

    fn rt<T>(expected: T)
    where
        T: PartialEq<T> + fmt::Debug + Unsigned,
    {
        crate::alloc::default!(|alloc| {
            let mut out = Vec::new();
            let cx = crate::context::Ignore::with_marker(&alloc);
            c::encode(&cx, &mut out, expected).unwrap();
            c::encode(&cx, &mut out, expected).unwrap();
            let mut data = out.as_slice();
            let cx = context::Ignore::with_marker(&alloc);
            let a: T = c::decode(&cx, &mut data).unwrap();
            let b: T = c::decode(&cx, &mut data).unwrap();
            assert!(data.is_empty());
            assert_eq!(a, expected);
            assert_eq!(b, expected);
        })
    }

    fn encode<T>(value: T) -> Vec<u8>
    where
        T: Unsigned,
    {
        crate::alloc::default!(|alloc| {
            let mut out = Vec::new();
            let cx = crate::context::Same::with_marker(&alloc);
            c::encode(&cx, crate::wrap::wrap(&mut out), value).unwrap();
            out
        })
    }

    macro_rules! test {
        ($ty:ty) => {{
            rt::<$ty>(0);
            rt::<$ty>(1);
            rt::<$ty>(42);
            rt::<$ty>(127);
            rt::<$ty>(128);
            rt::<$ty>(128 << 8);
            rt::<$ty>(<$ty>::MAX);

            let mut rng = StdRng::seed_from_u64(0xfd80fd80fd80fd80);

            for _ in 0..ITER {
                let value = rng.gen::<usize>();
                rt(value);
            }
        }};
    }

    test!(usize);
    test!(u16);
    test!(u32);
    test!(u64);
    test!(u128);

    assert_eq!(encode(1000u128), vec![232, 7]);
}

#[test]
fn test_zigzag() {
    assert_eq!(super::zigzag::encode(-1i32), 1u32);
    assert_eq!(super::zigzag::encode(-2i32), 3u32);

    fn rt<T>(value: T, expected: T::Unsigned)
    where
        T: fmt::Debug + Signed + PartialEq,
        T::Unsigned: fmt::Debug + PartialEq,
    {
        assert_eq!(zig::encode(value), expected);
        assert_eq!(zig::decode(expected), value);
    }

    macro_rules! test {
        ($signed:ty, $unsigned:ty) => {
            rt::<$signed>(0, 0);
            rt::<$signed>(-1, 1);
            rt::<$signed>(1, 2);
            rt::<$signed>(-2, 3);
            rt::<$signed>(2, 4);
            rt::<$signed>(<$signed>::MAX, <$unsigned>::MAX - 1);
            rt::<$signed>(<$signed>::MIN, <$unsigned>::MAX);
        };
    }

    test!(isize, usize);
    test!(i16, u16);
    test!(i32, u32);
    test!(i64, u64);
    test!(i128, u128);
}
