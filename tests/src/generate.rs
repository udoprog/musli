//! Module used to generate random structures.

use core::array;
#[cfg(all(feature = "alloc", feature = "std"))]
use core::ffi::CStr;
#[cfg(feature = "std")]
use core::hash::Hash;
use core::ops::Range;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::collections::binary_heap::BinaryHeap;
#[cfg(feature = "alloc")]
use alloc::collections::vec_deque::VecDeque;
#[cfg(feature = "alloc")]
use alloc::ffi::CString;
#[cfg(feature = "alloc")]
use alloc::rc::Rc;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::sync::Arc;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(all(feature = "alloc", feature = "std"))]
use std::ffi::OsStr;
#[cfg(feature = "std")]
use std::ffi::OsString;
#[cfg(all(feature = "alloc", feature = "std"))]
use std::path::Path;
#[cfg(feature = "std")]
use std::path::PathBuf;

use rand::distr::{Distribution, StandardUniform};

pub use tests_macros::Generate;

#[cfg(feature = "alloc")]
use alloc::collections::{BTreeMap, BTreeSet};
#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet};

options! {
    pub(crate) unsafe fn init_ranges();
    pub(crate) fn enumerate_ranges();
    static PATH_SEGMENT_RANGE: Range<usize> = 4..32, 2..4;
    static PATH_SEGMENTS_RANGE: Range<usize> = 4..32, 2..4;
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

#[cfg(feature = "alloc")]
impl<T> Generate for VecDeque<T>
where
    T: Generate,
{
    #[inline]
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        <VecDeque<T> as Generate>::generate_range(rng, VEC_RANGE.get())
    }

    fn generate_range<R>(rng: &mut R, range: Range<usize>) -> Self
    where
        R: rand::Rng,
    {
        let cap = rng.random_range(range);
        let mut vec = VecDeque::with_capacity(cap);

        for _ in 0..cap {
            if rng.random() {
                vec.push_front(T::generate(rng));
            } else {
                vec.push_back(T::generate(rng));
            }
        }

        vec
    }
}

#[cfg(feature = "alloc")]
impl<T> Generate for BinaryHeap<T>
where
    T: Ord + Generate,
{
    #[inline]
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        <BinaryHeap<T> as Generate>::generate_range(rng, VEC_RANGE.get())
    }

    fn generate_range<R>(rng: &mut R, range: Range<usize>) -> Self
    where
        R: rand::Rng,
    {
        let cap = rng.random_range(range);
        let mut vec = BinaryHeap::with_capacity(cap);

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
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        Self::generate_range(rng, MAP_RANGE.get())
    }

    fn generate_range<R>(rng: &mut R, range: Range<usize>) -> Self
    where
        R: rand::Rng,
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
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        Self::generate_range(rng, SET_RANGE.get())
    }

    fn generate_range<R>(rng: &mut R, range: Range<usize>) -> Self
    where
        R: rand::Rng,
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
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        Self::generate_range(rng, MAP_RANGE.get())
    }

    fn generate_range<R>(rng: &mut R, range: Range<usize>) -> Self
    where
        R: rand::Rng,
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
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        Self::generate_range(rng, SET_RANGE.get())
    }

    fn generate_range<R>(rng: &mut R, range: Range<usize>) -> Self
    where
        R: rand::Rng,
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
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        let mut string = String::new();

        for _ in 0..rng.random_range(STRING_RANGE.get()) {
            string.push(rng.random());
        }

        string
    }
}

#[cfg(feature = "alloc")]
impl Generate for Box<str> {
    #[inline]
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        Box::from(String::generate(rng))
    }
}

#[cfg(feature = "alloc")]
impl Generate for Rc<str> {
    #[inline]
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        Rc::from(String::generate(rng))
    }
}

#[cfg(feature = "alloc")]
impl Generate for Arc<str> {
    #[inline]
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        Arc::from(String::generate(rng))
    }
}

#[cfg(feature = "alloc")]
impl Generate for CString {
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        let mut string = Vec::new();

        for _ in 0..rng.random_range(STRING_RANGE.get()) {
            string.push(rng.random_range(1..=u8::MAX));
        }

        string.push(0);
        CString::from_vec_with_nul(string).unwrap()
    }
}

#[cfg(feature = "alloc")]
impl Generate for OsString {
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        let mut string = OsString::new();

        for _ in 0..rng.random_range(STRING_RANGE.get()) {
            string.push(rng.random::<char>().encode_utf8(&mut [0; 4]));
        }

        string
    }
}

#[cfg(feature = "std")]
impl Generate for PathBuf {
    #[inline]
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        use std::ffi::OsString;

        let mut path = OsString::new();

        const CORPUS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-%^&()[]{};:,.!?@#~`+=|\\\"'<>";

        for _ in 0..rng.random_range(PATH_SEGMENTS_RANGE.get()) {
            for _ in 0..rng.random_range(PATH_SEGMENT_RANGE.get()) {
                let c = CORPUS[rng.random_range(0..CORPUS.len())] as char;
                path.push(c.encode_utf8(&mut [0; 4]));
            }

            path.push(std::path::MAIN_SEPARATOR_STR);
        }

        PathBuf::from(path)
    }
}

macro_rules! container {
    ($from:ty, $inner:ty) => {
        impl Generate for Box<$inner> {
            #[inline]
            fn generate<R>(rng: &mut R) -> Self
            where
                R: rand::Rng,
            {
                Box::from(<$from>::generate(rng))
            }
        }

        impl Generate for Rc<$inner> {
            #[inline]
            fn generate<R>(rng: &mut R) -> Self
            where
                R: rand::Rng,
            {
                Rc::from(<$from>::generate(rng))
            }
        }

        impl Generate for Arc<$inner> {
            #[inline]
            fn generate<R>(rng: &mut R) -> Self
            where
                R: rand::Rng,
            {
                Arc::from(<$from>::generate(rng))
            }
        }
    };
}

#[cfg(all(feature = "alloc", feature = "std"))]
container!(PathBuf, Path);
#[cfg(all(feature = "alloc", feature = "std"))]
container!(CString, CStr);
#[cfg(all(feature = "alloc", feature = "std"))]
container!(OsString, OsStr);

impl Generate for () {
    #[inline]
    fn generate<R>(_: &mut R) -> Self
    where
        R: rand::Rng,
    {
    }
}

macro_rules! tuple {
    ($($ty:ident),* $(,)?) => {
        impl<$($ty,)*> Generate for ($($ty,)*) where $($ty: Generate,)* {
            #[inline]
            fn generate<R>(rng: &mut R) -> Self where R: rand::Rng {
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
                fn generate<R>(rng: &mut R) -> Self
                where
                    R: rand::Rng,
                {
                    rng.random_range(0..(i64::MAX as $ty))
                }

                #[inline]
                #[cfg(not(feature = "no-u64"))]
                fn generate<R>(rng: &mut R) -> Self
                where
                    R: rand::Rng,
                {
                    rng.random()
                }
            }
        )*
    }
}

macro_rules! primitive {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Generate for $ty
            where
                StandardUniform: Distribution<$ty>,
            {
                #[inline]
                fn generate<R>(rng: &mut R) -> Self
                where
                    R: rand::Rng,
                {
                    rng.random()
                }
            }
        )*
    };
}

macro_rules! atomic_impl {
    ($size:literal, $($atomic:ident),* $(,)?) => {
        $(
            #[cfg(target_has_atomic = $size)]
            impl Generate for core::sync::atomic::$atomic {
                #[inline]
                fn generate<R>(rng: &mut R) -> Self
                where
                    R: rand::Rng,
                {
                    core::sync::atomic::$atomic::new(Generate::generate(rng))
                }
            }
        )*
    };
}

primitive!(u8, u16, u32, i8, i16, i32, i64, i128);
unsigned!(u64, u128);
atomic_impl!("8", AtomicBool, AtomicU8, AtomicI8);
atomic_impl!("16", AtomicU16, AtomicI16);
atomic_impl!("32", AtomicU32, AtomicI32);
atomic_impl!("64", AtomicI64, AtomicU64);
atomic_impl!("ptr", AtomicIsize, AtomicUsize);

impl Generate for usize {
    #[inline]
    #[cfg(feature = "no-u64")]
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        rng.random_range(0..(i64::MAX as usize))
    }

    #[inline]
    #[cfg(not(feature = "no-u64"))]
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        let mut bytes = usize::to_ne_bytes(0);
        rng.fill_bytes(&mut bytes);
        usize::from_ne_bytes(bytes)
    }
}

impl Generate for isize {
    #[inline]
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        rng.random::<i64>() as isize
    }
}

primitive!(f32);
primitive!(f64);
primitive!(char);
primitive!(bool);

impl<T> Generate for core::num::Saturating<T>
where
    T: Generate,
{
    #[inline]
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        core::num::Saturating(T::generate(rng))
    }
}

impl<T> Generate for core::num::Wrapping<T>
where
    T: Generate,
{
    #[inline]
    fn generate<R>(rng: &mut R) -> Self
    where
        R: rand::Rng,
    {
        core::num::Wrapping(T::generate(rng))
    }
}

macro_rules! non_zero {
    ($($non_zero:ident, $signed_non_zero:ident, $ty:ty),* $(,)?) => {
        $(
            impl Generate for core::num::$non_zero {
                #[inline]
                fn generate<R>(rng: &mut R) -> Self
                where
                    R: rand::Rng,
                {
                    unsafe {
                        core::num::$non_zero::new_unchecked(rng.random_range(1..=<$ty>::MAX))
                    }
                }
            }

            impl Generate for core::num::$signed_non_zero {
                #[inline]
                fn generate<R>(rng: &mut R) -> Self
                where
                    R: rand::Rng,
                {
                    core::num::$non_zero::generate(rng).cast_signed()
                }
            }
        )*
    }
}

non_zero! {
    NonZeroU8, NonZeroI8, u8,
    NonZeroU16, NonZeroI16, u16,
    NonZeroU32, NonZeroI32, u32,
    NonZeroU64, NonZeroI64, u64,
    NonZeroU128, NonZeroI128, u128,
    NonZeroUsize, NonZeroIsize, usize,
}
