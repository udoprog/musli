//! Module used to generate random structures.

use core::array;
#[cfg(feature = "std")]
use core::hash::Hash;
use core::ops::Range;

#[cfg(feature = "alloc")]
use alloc::ffi::CString;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use rand::distr::{Distribution, StandardUniform};

pub use tests_macros::Generate;

#[cfg(feature = "alloc")]
use alloc::collections::{BTreeMap, BTreeSet};
#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet};

options! {
    pub(crate) unsafe fn init_ranges();
    pub(crate) fn enumerate_ranges();
    static STRING_RANGE: Range<usize> = 4..32, 2..16;
    #[cfg(any(feature = "std", feature = "alloc"))]
    static MAP_RANGE: Range<usize> = 10..20, 1..3;
    #[cfg(any(feature = "std", feature = "alloc"))]
    static SET_RANGE: Range<usize> = 10..20, 1..3;
    #[cfg(feature = "alloc")]
    static VEC_RANGE: Range<usize> = 10..20, 1..3;
}

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
    #[cfg(feature = "alloc")]
    pub fn next_vector<T>(&mut self, count: usize) -> Vec<T>
    where
        T: Generate,
    {
        let mut out = Vec::with_capacity(count);

        for _ in 0..count {
            T::generate_in(self, |value| out.push(value));
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

    /// Generate a value of the given type into the specified collections.
    fn generate_in<R, F>(rng: &mut R, mut out: F)
    where
        R: rand::Rng,
        F: FnMut(Self),
    {
        out(Self::generate(rng));
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

#[cfg(feature = "alloc")]
impl<T> Generate for Vec<T>
where
    T: Generate,
{
    #[inline]
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        <Vec<T> as Generate>::generate_range(rng, VEC_RANGE.get())
    }

    fn generate_range<R>(rng: &mut R, range: Range<usize>) -> Self
    where
        R: rand::Rng,
    {
        let cap = rng.random_range(range);
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
        Self::generate_range(rng, MAP_RANGE.get())
    }

    fn generate_range<T>(rng: &mut T, range: Range<usize>) -> Self
    where
        T: rand::Rng,
    {
        let cap = rng.random_range(range);
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
        Self::generate_range(rng, SET_RANGE.get())
    }

    fn generate_range<T>(rng: &mut T, range: Range<usize>) -> Self
    where
        T: rand::Rng,
    {
        let mut map = HashSet::new();

        for _ in 0..rng.random_range(range) {
            map.insert(K::generate(rng));
        }

        map
    }
}

#[cfg(feature = "alloc")]
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
        Self::generate_range(rng, MAP_RANGE.get())
    }

    fn generate_range<T>(rng: &mut T, range: Range<usize>) -> Self
    where
        T: rand::Rng,
    {
        let mut map = BTreeMap::new();

        for _ in 0..rng.random_range(range) {
            map.insert(K::generate(rng), V::generate(rng));
        }

        map
    }
}

#[cfg(feature = "alloc")]
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
        Self::generate_range(rng, SET_RANGE.get())
    }

    fn generate_range<T>(rng: &mut T, range: Range<usize>) -> Self
    where
        T: rand::Rng,
    {
        let mut map = BTreeSet::new();

        for _ in 0..rng.random_range(range) {
            map.insert(K::generate(rng));
        }

        map
    }
}

#[cfg(feature = "alloc")]
impl Generate for String {
    fn generate<T>(rng: &mut T) -> Self
    where
        T: rand::Rng,
    {
        let mut string = String::new();

        for _ in 0..rng.random_range(STRING_RANGE.get()) {
            string.push(rng.random());
        }

        string
    }
}

#[cfg(feature = "alloc")]
impl Generate for CString {
    fn generate<T>(rng: &mut T) -> Self
    where
        T: rand::Rng,
    {
        let mut string = Vec::new();

        for _ in 0..rng.random_range(STRING_RANGE.get()) {
            string.push(rng.random_range(1..=u8::MAX));
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

macro_rules! unsigned {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Generate for $ty
            where
                StandardUniform: Distribution<$ty>,
            {
                #[inline]
                #[cfg(feature = "no-u64")]
                fn generate<T>(rng: &mut T) -> Self
                where
                    T: rand::Rng,
                {
                    rng.random_range(0..(i64::MAX as $ty))
                }

                #[inline]
                #[cfg(not(feature = "no-u64"))]
                fn generate<T>(rng: &mut T) -> Self
                where
                    T: rand::Rng,
                {
                    rng.random()
                }
            }
        )*
    };
}

macro_rules! primitive {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Generate for $ty
            where
                StandardUniform: Distribution<$ty>,
            {
                #[inline]
                fn generate<T>(rng: &mut T) -> Self
                where
                    T: rand::Rng,
                {
                    rng.random()
                }
            }
        )*
    };
}

primitive!(u8, u16, u32, i8, i16, i32, i64, i128);
unsigned!(u64, u128);

impl Generate for usize {
    #[inline]
    #[cfg(feature = "no-u64")]
    fn generate<T>(rng: &mut T) -> Self
    where
        T: rand::Rng,
    {
        rng.random_range(0..(i64::MAX as usize))
    }

    #[inline]
    #[cfg(not(feature = "no-u64"))]
    fn generate<T>(rng: &mut T) -> Self
    where
        T: rand::Rng,
    {
        let mut bytes = usize::to_ne_bytes(0);
        rng.fill_bytes(&mut bytes);
        usize::from_ne_bytes(bytes)
    }
}

impl Generate for isize {
    #[inline]
    fn generate<T>(rng: &mut T) -> Self
    where
        T: rand::Rng,
    {
        rng.random::<i64>() as isize
    }
}

primitive!(f32);
primitive!(f64);
primitive!(char);
primitive!(bool);
