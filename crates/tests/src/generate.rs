//! Module used to generate random structures.

use core::array;
#[cfg(feature = "std")]
use core::hash::Hash;
use core::ops::Range;

use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec::Vec;

use rand::distributions::Distribution;
use rand::distributions::Standard;

pub use musli_macros::Generate;

#[cfg(feature = "std")]
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// Random number generator.
pub struct Rng {
    rng: rand::rngs::StdRng,
}

impl Rng {
    pub(super) fn from_seed(seed: u64) -> Self {
        use rand::SeedableRng;

        Self {
            rng: rand::rngs::StdRng::seed_from_u64(seed),
        }
    }

    /// Get the next vector.
    pub fn next_vector<T>(&mut self, count: usize) -> Vec<T>
    where
        T: Generate,
    {
        let mut out = Vec::with_capacity(count);

        for _ in 0..count {
            out.push(self.next());
        }

        out
    }

    /// Get the next value.
    #[allow(clippy::should_implement_trait)]
    pub fn next<T>(&mut self) -> T
    where
        T: Generate,
    {
        T::generate(self)
    }
}

impl rand::RngCore for Rng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.rng.fill_bytes(dest);
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.rng.try_fill_bytes(dest)
    }
}

miri! {
    pub const STRING_RANGE: Range<usize> = 0..32, 0..16;
    #[cfg(feature = "std")]
    pub const MAP_RANGE: Range<usize> = 10..100, 1..3;
    pub const VEC_RANGE: Range<usize> = 10..100, 1..3;
}

pub trait Generate: Sized {
    /// Generate a value of the given type.
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng;

    /// Implement to receive a range parameters, by default it is simply ignored.
    fn generate_range<R>(rng: &mut R, _: Range<usize>) -> Self
    where
        R: rand::Rng,
    {
        Self::generate(rng)
    }
}

impl<T, const N: usize> Generate for [T; N]
where
    T: Generate,
{
    #[inline]
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        array::from_fn(|_| T::generate(rng))
    }
}

impl<T> Generate for Vec<T>
where
    T: Generate,
{
    #[inline]
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        <Vec<T> as Generate>::generate_range(rng, VEC_RANGE)
    }

    fn generate_range<R>(rng: &mut R, range: Range<usize>) -> Self
    where
        R: rand::Rng,
    {
        let cap = rng.gen_range(range);
        let mut vec = Vec::with_capacity(cap);

        for _ in 0..cap {
            vec.push(T::generate(rng));
        }

        vec
    }
}

#[cfg(feature = "std")]
impl<K, V> Generate for HashMap<K, V>
where
    K: Eq + Hash,
    K: Generate,
    V: Generate,
{
    #[inline]
    fn generate<T>(rng: &mut T) -> Self
    where
        T: rand::Rng,
    {
        Self::generate_range(rng, MAP_RANGE)
    }

    fn generate_range<T>(rng: &mut T, range: Range<usize>) -> Self
    where
        T: rand::Rng,
    {
        let cap = rng.gen_range(range);
        let mut map = HashMap::with_capacity(cap);

        for _ in 0..cap {
            map.insert(K::generate(rng), V::generate(rng));
        }

        map
    }
}

#[cfg(feature = "std")]
impl<K> Generate for HashSet<K>
where
    K: Eq + Hash,
    K: Generate,
{
    #[inline]
    fn generate<T>(rng: &mut T) -> Self
    where
        T: rand::Rng,
    {
        Self::generate_range(rng, MAP_RANGE)
    }

    fn generate_range<T>(rng: &mut T, range: Range<usize>) -> Self
    where
        T: rand::Rng,
    {
        let mut map = HashSet::new();

        for _ in 0..rng.gen_range(range) {
            map.insert(K::generate(rng));
        }

        map
    }
}

#[cfg(feature = "std")]
impl<K, V> Generate for BTreeMap<K, V>
where
    K: Eq + Ord,
    K: Generate,
    V: Generate,
{
    #[inline]
    fn generate<T>(rng: &mut T) -> Self
    where
        T: rand::Rng,
    {
        Self::generate_range(rng, MAP_RANGE)
    }

    fn generate_range<T>(rng: &mut T, range: Range<usize>) -> Self
    where
        T: rand::Rng,
    {
        let mut map = BTreeMap::new();

        for _ in 0..rng.gen_range(range) {
            map.insert(K::generate(rng), V::generate(rng));
        }

        map
    }
}

#[cfg(feature = "std")]
impl<K> Generate for BTreeSet<K>
where
    K: Ord,
    K: Generate,
{
    #[inline]
    fn generate<T>(rng: &mut T) -> Self
    where
        T: rand::Rng,
    {
        Self::generate_range(rng, MAP_RANGE)
    }

    fn generate_range<T>(rng: &mut T, range: Range<usize>) -> Self
    where
        T: rand::Rng,
    {
        let mut map = BTreeSet::new();

        for _ in 0..rng.gen_range(range) {
            map.insert(K::generate(rng));
        }

        map
    }
}

impl Generate for String {
    fn generate<T>(rng: &mut T) -> Self
    where
        T: rand::Rng,
    {
        let mut string = String::new();

        for _ in 0..rng.gen_range(STRING_RANGE) {
            string.push(rng.gen());
        }

        string
    }
}

impl Generate for CString {
    fn generate<T>(rng: &mut T) -> Self
    where
        T: rand::Rng,
    {
        let mut string = Vec::new();

        for _ in 0..rng.gen_range(STRING_RANGE) {
            string.push(rng.gen_range(1..=u8::MAX));
        }

        string.push(0);
        CString::from_vec_with_nul(string).unwrap()
    }
}

impl Generate for () {
    #[inline]
    fn generate<T>(_: &mut T) -> Self
    where
        T: rand::Rng,
    {
    }
}

macro_rules! tuple {
    ($($ty:ident),* $(,)?) => {
        impl<$($ty,)*> Generate for ($($ty,)*) where $($ty: Generate,)* {
            #[inline]
            fn generate<T>(rng: &mut T) -> Self where T: rand::Rng {
                ($(<$ty>::generate(rng),)*)
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
        impl Generate for $ty
        where
            Standard: Distribution<$ty>,
        {
            #[inline]
            fn generate<T>(rng: &mut T) -> Self
            where
                T: rand::Rng,
            {
                rng.gen()
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
