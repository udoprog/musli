use core::hash::Hash;
use core::ops::Range;

use alloc::string::String;
use alloc::vec::Vec;

use rand::distributions::Distribution;
use rand::distributions::Standard;
use rand::{rngs::StdRng, Rng};

#[cfg(feature = "std")]
use std::collections::HashMap;

miri! {
    const STRING_RANGE: Range<usize> = 0..256, 0..16;
    const MAP_RANGE: Range<usize> = 100..500, 1..3;
    const VEC_RANGE: Range<usize> = 100..500, 1..3;
}

pub trait Generate<T>: Sized {
    /// Generate a value of the given type.
    fn generate(&mut self) -> T;

    /// Implement to receive a range parameters, by default it is simply ignored.
    fn generate_range(&mut self, _: Range<usize>) -> T {
        self.generate()
    }
}

impl<T> Generate<Vec<T>> for StdRng
where
    Self: Generate<T>,
{
    #[inline]
    fn generate(&mut self) -> Vec<T> {
        Generate::<Vec<T>>::generate_range(self, VEC_RANGE)
    }

    fn generate_range(&mut self, range: Range<usize>) -> Vec<T> {
        let cap = self.gen_range(range);
        let mut vec = Vec::with_capacity(cap);

        for _ in 0..cap {
            vec.push(self.generate());
        }

        vec
    }
}

#[cfg(feature = "std")]
impl<K, V> Generate<HashMap<K, V>> for StdRng
where
    K: Eq + Hash,
    Self: Generate<K>,
    Self: Generate<V>,
{
    #[inline]
    fn generate(&mut self) -> HashMap<K, V> {
        self.generate_range(MAP_RANGE)
    }

    fn generate_range(&mut self, range: Range<usize>) -> HashMap<K, V> {
        let cap = self.gen_range(range);
        let mut map = HashMap::with_capacity(cap);

        for _ in 0..cap {
            map.insert(self.generate(), self.generate());
        }

        map
    }
}

impl Generate<String> for StdRng {
    fn generate(&mut self) -> String {
        let mut string = String::new();

        for _ in 0..self.gen_range(STRING_RANGE) {
            string.push(self.gen());
        }

        string
    }
}

impl Generate<()> for StdRng {
    #[inline]
    fn generate(&mut self) {}
}

macro_rules! tuple {
    ($($ty:ident),* $(,)?) => {
        impl<$($ty,)*> Generate<($($ty,)*)> for StdRng where $(Self: Generate<$ty>,)* {
            #[inline]
            fn generate(&mut self) -> ($($ty,)*) {
                macro_rules! generate {
                    ($_:ident) => {
                        self.generate()
                    }
                }

                ($(generate!($ty),)*)
            }
        }
    }
}

tuple!(A);
tuple!(A, B);
tuple!(A, B, C);
tuple!(A, B, C, D);
tuple!(A, B, C, D, E);
tuple!(A, B, C, D, E, F);
tuple!(A, B, C, D, E, F, G);

macro_rules! primitive {
    ($ty:ty) => {
        impl Generate<$ty> for StdRng
        where
            Standard: Distribution<$ty>,
        {
            #[inline]
            fn generate(&mut self) -> $ty {
                self.gen()
            }
        }
    };
}

primitive!(u8);
primitive!(u16);
primitive!(u32);
primitive!(u64);
primitive!(u128);
primitive!(i8);
primitive!(i16);
primitive!(i32);
primitive!(i64);
primitive!(i128);
primitive!(usize);
primitive!(isize);
primitive!(f32);
primitive!(f64);
primitive!(char);
primitive!(bool);
