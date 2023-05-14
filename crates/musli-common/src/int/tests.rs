use std::boxed::Box;
use std::fmt;
use std::vec::Vec;

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

use crate::int::continuation as c;
use crate::int::zigzag as zig;
use crate::int::{Signed, Unsigned};
use crate::writer::Buffer;

#[test]
fn test_continuation_encoding() -> Result<()> {
    use rand::prelude::*;

    fn rt<T>(expected: T) -> Result<()>
    where
        T: PartialEq<T> + fmt::Debug + Unsigned,
    {
        let mut out = Buffer::new();
        c::encode(&mut out, expected)?;
        c::encode(&mut out, expected)?;
        let mut data = out.as_slice();
        let a: T = c::decode(&mut data)?;
        let b: T = c::decode(&mut data)?;
        assert!(data.is_empty());
        assert_eq!(a, expected);
        assert_eq!(b, expected);
        Ok(())
    }

    fn encode<T>(value: T) -> Result<Vec<u8>>
    where
        T: Unsigned,
    {
        let mut out = Vec::new();
        c::encode(crate::wrap::wrap(&mut out), value)?;
        Ok(out)
    }

    macro_rules! test {
        ($ty:ty) => {{
            rt::<$ty>(0)?;
            rt::<$ty>(1)?;
            rt::<$ty>(42)?;
            rt::<$ty>(127)?;
            rt::<$ty>(128)?;
            rt::<$ty>(128 << 8)?;
            rt::<$ty>(<$ty>::MAX)?;

            let mut rng = StdRng::seed_from_u64(0xfd80fd80fd80fd80);

            for _ in 0..10000 {
                let value = rng.gen::<usize>();
                rt(value)?;
            }
        }};
    }

    test!(usize);
    test!(u16);
    test!(u32);
    test!(u64);
    test!(u128);

    assert_eq!(encode(1000u128)?, vec![232, 7]);
    Ok(())
}

#[test]
fn test_zigzag() {
    fn rt<T>(value: T, expected: T::Unsigned)
    where
        T: fmt::Debug + Signed + PartialEq,
        T::Unsigned: Unsigned<Signed = T> + fmt::Debug + PartialEq,
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
