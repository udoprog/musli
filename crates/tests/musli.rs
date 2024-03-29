#![feature(prelude_import)]
//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/tests.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/tests)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-tests-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/tests)
#![no_std]
#[prelude_import]
use core::prelude::rust_2021::*;
#[macro_use]
extern crate core;
extern crate compiler_builtins as _;
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;
/// Default random seed to use.
pub const RNG_SEED: u64 = 2718281828459045235;
pub use musli_macros::benchmarker;
pub mod generate {
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
    #[cfg(not(miri))]
    pub const STRING_RANGE: Range<usize> = 4..32;
    #[cfg(feature = "std")]
    #[cfg(not(miri))]
    pub const MAP_RANGE: Range<usize> = 10..20;
    #[cfg(not(miri))]
    pub const VEC_RANGE: Range<usize> = 10..20;
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
                T::generate_in(self, &mut out);
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
        fn generate_in<R>(rng: &mut R, out: &mut Vec<Self>)
        where
            R: rand::Rng,
        {
            out.push(Self::generate(rng));
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
        {}
    }
    impl<A> Generate for (A,)
    where
        A: Generate,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            (<A>::generate(rng),)
        }
    }
    impl<A, B> Generate for (A, B)
    where
        A: Generate,
        B: Generate,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            (<A>::generate(rng), <B>::generate(rng))
        }
    }
    impl<A, B, C> Generate for (A, B, C)
    where
        A: Generate,
        B: Generate,
        C: Generate,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            (<A>::generate(rng), <B>::generate(rng), <C>::generate(rng))
        }
    }
    impl<A, B, C, D> Generate for (A, B, C, D)
    where
        A: Generate,
        B: Generate,
        C: Generate,
        D: Generate,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            (
                <A>::generate(rng),
                <B>::generate(rng),
                <C>::generate(rng),
                <D>::generate(rng),
            )
        }
    }
    impl<A, B, C, D, E> Generate for (A, B, C, D, E)
    where
        A: Generate,
        B: Generate,
        C: Generate,
        D: Generate,
        E: Generate,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            (
                <A>::generate(rng),
                <B>::generate(rng),
                <C>::generate(rng),
                <D>::generate(rng),
                <E>::generate(rng),
            )
        }
    }
    impl<A, B, C, D, E, F> Generate for (A, B, C, D, E, F)
    where
        A: Generate,
        B: Generate,
        C: Generate,
        D: Generate,
        E: Generate,
        F: Generate,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            (
                <A>::generate(rng),
                <B>::generate(rng),
                <C>::generate(rng),
                <D>::generate(rng),
                <E>::generate(rng),
                <F>::generate(rng),
            )
        }
    }
    impl<A, B, C, D, E, F, G> Generate for (A, B, C, D, E, F, G)
    where
        A: Generate,
        B: Generate,
        C: Generate,
        D: Generate,
        E: Generate,
        F: Generate,
        G: Generate,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            (
                <A>::generate(rng),
                <B>::generate(rng),
                <C>::generate(rng),
                <D>::generate(rng),
                <E>::generate(rng),
                <F>::generate(rng),
                <G>::generate(rng),
            )
        }
    }
    impl Generate for u8
    where
        Standard: Distribution<u8>,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            rng.gen()
        }
    }
    impl Generate for u16
    where
        Standard: Distribution<u16>,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            rng.gen()
        }
    }
    impl Generate for u32
    where
        Standard: Distribution<u32>,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            rng.gen()
        }
    }
    impl Generate for u64
    where
        Standard: Distribution<u64>,
    {
        #[inline]
        #[cfg(not(feature = "no-u64"))]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            rng.gen()
        }
    }
    impl Generate for u128
    where
        Standard: Distribution<u128>,
    {
        #[inline]
        #[cfg(not(feature = "no-u64"))]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            rng.gen()
        }
    }
    impl Generate for usize
    where
        Standard: Distribution<usize>,
    {
        #[inline]
        #[cfg(not(feature = "no-u64"))]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            rng.gen()
        }
    }
    impl Generate for i8
    where
        Standard: Distribution<i8>,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            rng.gen()
        }
    }
    impl Generate for i16
    where
        Standard: Distribution<i16>,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            rng.gen()
        }
    }
    impl Generate for i32
    where
        Standard: Distribution<i32>,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            rng.gen()
        }
    }
    impl Generate for i64
    where
        Standard: Distribution<i64>,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            rng.gen()
        }
    }
    impl Generate for i128
    where
        Standard: Distribution<i128>,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            rng.gen()
        }
    }
    impl Generate for isize
    where
        Standard: Distribution<isize>,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            rng.gen()
        }
    }
    impl Generate for f32
    where
        Standard: Distribution<f32>,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            rng.gen()
        }
    }
    impl Generate for f64
    where
        Standard: Distribution<f64>,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            rng.gen()
        }
    }
    impl Generate for char
    where
        Standard: Distribution<char>,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            rng.gen()
        }
    }
    impl Generate for bool
    where
        Standard: Distribution<bool>,
    {
        #[inline]
        fn generate<T>(rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            rng.gen()
        }
    }
}
mod mode {
    #[cfg(feature = "musli")]
    pub enum Packed {}
}
pub mod models {
    #[cfg(not(feature = "no-map"))]
    use std::collections::HashMap;
    use std::collections::HashSet;
    #[cfg(not(feature = "no-btree"))]
    use std::collections::{BTreeMap, BTreeSet};
    use core::ops::Range;
    #[cfg(not(feature = "no-cstring"))]
    use alloc::ffi::CString;
    use alloc::string::String;
    use alloc::vec::Vec;
    #[cfg(feature = "musli")]
    use musli::{Decode, Encode};
    #[cfg(feature = "musli")]
    use crate::mode::Packed;
    use crate::generate::Generate;
    pub use rand::prelude::*;
    #[cfg(not(miri))]
    pub const PRIMITIVES_RANGE: Range<usize> = 10..100;
    #[cfg(not(miri))]
    pub const MEDIUM_RANGE: Range<usize> = 10..100;
    #[cfg(not(miri))]
    pub const SMALL_FIELDS: Range<usize> = 1..3;
    pub struct PrimitivesPacked {
        unsigned8: u8,
        #[musli(bytes)]
        _pad0: [u8; 1],
        unsigned16: u16,
        unsigned32: u32,
        unsigned64: u64,
        #[cfg(not(feature = "no-128"))]
        unsigned128: u128,
        signed8: i8,
        #[musli(bytes)]
        _pad1: [u8; 1],
        signed16: i16,
        signed32: i32,
        signed64: i64,
        #[cfg(not(feature = "no-128"))]
        signed128: i128,
        #[cfg(not(feature = "no-usize"))]
        unsignedsize: usize,
        #[cfg(not(feature = "no-usize"))]
        signedsize: isize,
        float32: f32,
        #[musli(bytes)]
        _pad3: [u8; 4],
        float64: f64,
    }
    const _: () = {
        #[automatically_derived]
        impl<M> ::musli::en::Encode<M> for PrimitivesPacked {
            #[inline]
            fn encode<T0>(
                &self,
                _1: &T0::Cx,
                _0: T0,
            ) -> ::musli::__priv::Result<
                <T0 as ::musli::en::Encoder>::Ok,
                <T0 as ::musli::en::Encoder>::Error,
            >
            where
                T0: ::musli::en::Encoder<Mode = M>,
            {
                ::musli::__priv::Ok({
                    ::musli::Context::enter_struct(_1, "PrimitivesPacked");
                    let _3 = ::musli::en::Encoder::encode_struct_fn(
                        _0,
                        17usize,
                        move |_0| {
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsigned8",
                                &0usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&0usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&self.unsigned8, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "_pad0", &1usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&1usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::EncodeBytes::<
                                        M,
                                    >::encode_bytes(&self._pad0, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsigned16",
                                &2usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&2usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&self.unsigned16, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsigned32",
                                &3usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&3usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&self.unsigned32, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsigned64",
                                &4usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&4usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&self.unsigned64, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsigned128",
                                &5usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&5usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        M,
                                    >::encode(&self.unsigned128, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "signed8", &6usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&6usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&self.signed8, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "_pad1", &7usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&7usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::EncodeBytes::<
                                        M,
                                    >::encode_bytes(&self._pad1, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "signed16", &8usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&8usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&self.signed16, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "signed32", &9usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&9usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&self.signed32, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "signed64",
                                &10usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&10usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&self.signed64, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "signed128",
                                &11usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&11usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&self.signed128, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsignedsize",
                                &12usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&12usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        M,
                                    >::encode(&self.unsignedsize, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "signedsize",
                                &13usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&13usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&self.signedsize, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "float32", &14usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&14usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&self.float32, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "_pad3", &15usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&15usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::EncodeBytes::<
                                        M,
                                    >::encode_bytes(&self._pad3, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "float64", &16usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&16usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<M>::encode(&self.float64, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::__priv::Ok(())
                        },
                    )?;
                    ::musli::Context::leave_struct(_1);
                    _3
                })
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        #[allow(clippy::init_numbered_fields)]
        #[allow(clippy::let_unit_value)]
        impl<'de, M> ::musli::de::Decode<'de, M> for PrimitivesPacked {
            #[inline]
            fn decode<T0>(
                _0: &T0::Cx,
                _1: T0,
            ) -> ::musli::__priv::Result<Self, <T0::Cx as ::musli::Context>::Error>
            where
                T0: ::musli::de::Decoder<'de, Mode = M>,
            {
                ::musli::__priv::Ok({
                    {
                        let mut _0_f: ::musli::__priv::Option<u8> = ::musli::__priv::None;
                        let mut _1_f: ::musli::__priv::Option<[u8; 1]> = ::musli::__priv::None;
                        let mut _2_f: ::musli::__priv::Option<u16> = ::musli::__priv::None;
                        let mut _3_f: ::musli::__priv::Option<u32> = ::musli::__priv::None;
                        let mut _4_f: ::musli::__priv::Option<u64> = ::musli::__priv::None;
                        let mut _5_f: ::musli::__priv::Option<u128> = ::musli::__priv::None;
                        let mut _6_f: ::musli::__priv::Option<i8> = ::musli::__priv::None;
                        let mut _7_f: ::musli::__priv::Option<[u8; 1]> = ::musli::__priv::None;
                        let mut _8_f: ::musli::__priv::Option<i16> = ::musli::__priv::None;
                        let mut _9_f: ::musli::__priv::Option<i32> = ::musli::__priv::None;
                        let mut _10_f: ::musli::__priv::Option<i64> = ::musli::__priv::None;
                        let mut _11_f: ::musli::__priv::Option<i128> = ::musli::__priv::None;
                        let mut _12_f: ::musli::__priv::Option<usize> = ::musli::__priv::None;
                        let mut _13_f: ::musli::__priv::Option<isize> = ::musli::__priv::None;
                        let mut _14_f: ::musli::__priv::Option<f32> = ::musli::__priv::None;
                        let mut _15_f: ::musli::__priv::Option<[u8; 4]> = ::musli::__priv::None;
                        let mut _16_f: ::musli::__priv::Option<f64> = ::musli::__priv::None;
                        ::musli::Context::enter_struct(_0, "PrimitivesPacked");
                        ::musli::de::Decoder::decode_struct(
                            _1,
                            Some(17usize),
                            move |_5| {
                                while let ::musli::__priv::Some(mut _3) = ::musli::de::StructDecoder::decode_field(
                                    _5,
                                )? {
                                    let _2 = {
                                        let _3 = ::musli::de::StructFieldDecoder::decode_field_name(
                                            &mut _3,
                                        )?;
                                        ::musli::de::Decode::<M>::decode(_0, _3)?
                                    };
                                    match _2 {
                                        0usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "unsigned8",
                                                &0usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _0_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<M>::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        1usize => {
                                            ::musli::Context::enter_named_field(_0, "_pad0", &1usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _1_f = ::musli::__priv::Some(
                                                ::musli::de::DecodeBytes::<M>::decode_bytes(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        2usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "unsigned16",
                                                &2usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _2_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<M>::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        3usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "unsigned32",
                                                &3usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _3_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<M>::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        4usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "unsigned64",
                                                &4usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _4_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<M>::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        5usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "unsigned128",
                                                &5usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _5_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<M>::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        6usize => {
                                            ::musli::Context::enter_named_field(_0, "signed8", &6usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _6_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<M>::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        7usize => {
                                            ::musli::Context::enter_named_field(_0, "_pad1", &7usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _7_f = ::musli::__priv::Some(
                                                ::musli::de::DecodeBytes::<M>::decode_bytes(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        8usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "signed16",
                                                &8usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _8_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<M>::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        9usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "signed32",
                                                &9usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _9_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<M>::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        10usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "signed64",
                                                &10usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _10_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<M>::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        11usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "signed128",
                                                &11usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _11_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<M>::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        12usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "unsignedsize",
                                                &12usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _12_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<M>::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        13usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "signedsize",
                                                &13usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _13_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<M>::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        14usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "float32",
                                                &14usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _14_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<M>::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        15usize => {
                                            ::musli::Context::enter_named_field(_0, "_pad3", &15usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _15_f = ::musli::__priv::Some(
                                                ::musli::de::DecodeBytes::<M>::decode_bytes(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        16usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "float64",
                                                &16usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _16_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<M>::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        _2 => {
                                            if ::musli::__priv::skip_field(_3)? {
                                                return ::musli::__priv::Err(
                                                    ::musli::Context::invalid_field_tag(
                                                        _0,
                                                        "PrimitivesPacked",
                                                        &_2,
                                                    ),
                                                );
                                            }
                                        }
                                    }
                                }
                                ::musli::Context::leave_struct(_0);
                                ::musli::__priv::Ok(Self {
                                    unsigned8: match _0_f {
                                        ::musli::__priv::Some(_0_f) => _0_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(
                                                    _0,
                                                    "PrimitivesPacked",
                                                    &0usize,
                                                ),
                                            );
                                        }
                                    },
                                    _pad0: match _1_f {
                                        ::musli::__priv::Some(_1_f) => _1_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(
                                                    _0,
                                                    "PrimitivesPacked",
                                                    &1usize,
                                                ),
                                            );
                                        }
                                    },
                                    unsigned16: match _2_f {
                                        ::musli::__priv::Some(_2_f) => _2_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(
                                                    _0,
                                                    "PrimitivesPacked",
                                                    &2usize,
                                                ),
                                            );
                                        }
                                    },
                                    unsigned32: match _3_f {
                                        ::musli::__priv::Some(_3_f) => _3_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(
                                                    _0,
                                                    "PrimitivesPacked",
                                                    &3usize,
                                                ),
                                            );
                                        }
                                    },
                                    unsigned64: match _4_f {
                                        ::musli::__priv::Some(_4_f) => _4_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(
                                                    _0,
                                                    "PrimitivesPacked",
                                                    &4usize,
                                                ),
                                            );
                                        }
                                    },
                                    unsigned128: match _5_f {
                                        ::musli::__priv::Some(_5_f) => _5_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(
                                                    _0,
                                                    "PrimitivesPacked",
                                                    &5usize,
                                                ),
                                            );
                                        }
                                    },
                                    signed8: match _6_f {
                                        ::musli::__priv::Some(_6_f) => _6_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(
                                                    _0,
                                                    "PrimitivesPacked",
                                                    &6usize,
                                                ),
                                            );
                                        }
                                    },
                                    _pad1: match _7_f {
                                        ::musli::__priv::Some(_7_f) => _7_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(
                                                    _0,
                                                    "PrimitivesPacked",
                                                    &7usize,
                                                ),
                                            );
                                        }
                                    },
                                    signed16: match _8_f {
                                        ::musli::__priv::Some(_8_f) => _8_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(
                                                    _0,
                                                    "PrimitivesPacked",
                                                    &8usize,
                                                ),
                                            );
                                        }
                                    },
                                    signed32: match _9_f {
                                        ::musli::__priv::Some(_9_f) => _9_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(
                                                    _0,
                                                    "PrimitivesPacked",
                                                    &9usize,
                                                ),
                                            );
                                        }
                                    },
                                    signed64: match _10_f {
                                        ::musli::__priv::Some(_10_f) => _10_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(
                                                    _0,
                                                    "PrimitivesPacked",
                                                    &10usize,
                                                ),
                                            );
                                        }
                                    },
                                    signed128: match _11_f {
                                        ::musli::__priv::Some(_11_f) => _11_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(
                                                    _0,
                                                    "PrimitivesPacked",
                                                    &11usize,
                                                ),
                                            );
                                        }
                                    },
                                    unsignedsize: match _12_f {
                                        ::musli::__priv::Some(_12_f) => _12_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(
                                                    _0,
                                                    "PrimitivesPacked",
                                                    &12usize,
                                                ),
                                            );
                                        }
                                    },
                                    signedsize: match _13_f {
                                        ::musli::__priv::Some(_13_f) => _13_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(
                                                    _0,
                                                    "PrimitivesPacked",
                                                    &13usize,
                                                ),
                                            );
                                        }
                                    },
                                    float32: match _14_f {
                                        ::musli::__priv::Some(_14_f) => _14_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(
                                                    _0,
                                                    "PrimitivesPacked",
                                                    &14usize,
                                                ),
                                            );
                                        }
                                    },
                                    _pad3: match _15_f {
                                        ::musli::__priv::Some(_15_f) => _15_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(
                                                    _0,
                                                    "PrimitivesPacked",
                                                    &15usize,
                                                ),
                                            );
                                        }
                                    },
                                    float64: match _16_f {
                                        ::musli::__priv::Some(_16_f) => _16_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(
                                                    _0,
                                                    "PrimitivesPacked",
                                                    &16usize,
                                                ),
                                            );
                                        }
                                    },
                                })
                            },
                        )?
                    }
                })
            }
        }
    };
    #[automatically_derived]
    impl ::core::fmt::Debug for PrimitivesPacked {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "unsigned8",
                "_pad0",
                "unsigned16",
                "unsigned32",
                "unsigned64",
                "unsigned128",
                "signed8",
                "_pad1",
                "signed16",
                "signed32",
                "signed64",
                "signed128",
                "unsignedsize",
                "signedsize",
                "float32",
                "_pad3",
                "float64",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.unsigned8,
                &self._pad0,
                &self.unsigned16,
                &self.unsigned32,
                &self.unsigned64,
                &self.unsigned128,
                &self.signed8,
                &self._pad1,
                &self.signed16,
                &self.signed32,
                &self.signed64,
                &self.signed128,
                &self.unsignedsize,
                &self.signedsize,
                &self.float32,
                &self._pad3,
                &&self.float64,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "PrimitivesPacked",
                names,
                values,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for PrimitivesPacked {
        #[inline]
        fn clone(&self) -> PrimitivesPacked {
            let _: ::core::clone::AssertParamIsClone<u8>;
            let _: ::core::clone::AssertParamIsClone<[u8; 1]>;
            let _: ::core::clone::AssertParamIsClone<u16>;
            let _: ::core::clone::AssertParamIsClone<u32>;
            let _: ::core::clone::AssertParamIsClone<u64>;
            let _: ::core::clone::AssertParamIsClone<u128>;
            let _: ::core::clone::AssertParamIsClone<i8>;
            let _: ::core::clone::AssertParamIsClone<[u8; 1]>;
            let _: ::core::clone::AssertParamIsClone<i16>;
            let _: ::core::clone::AssertParamIsClone<i32>;
            let _: ::core::clone::AssertParamIsClone<i64>;
            let _: ::core::clone::AssertParamIsClone<i128>;
            let _: ::core::clone::AssertParamIsClone<usize>;
            let _: ::core::clone::AssertParamIsClone<isize>;
            let _: ::core::clone::AssertParamIsClone<f32>;
            let _: ::core::clone::AssertParamIsClone<[u8; 4]>;
            let _: ::core::clone::AssertParamIsClone<f64>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for PrimitivesPacked {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for PrimitivesPacked {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for PrimitivesPacked {
        #[inline]
        fn eq(&self, other: &PrimitivesPacked) -> bool {
            self.unsigned8 == other.unsigned8 && self._pad0 == other._pad0
                && self.unsigned16 == other.unsigned16
                && self.unsigned32 == other.unsigned32
                && self.unsigned64 == other.unsigned64
                && self.unsigned128 == other.unsigned128 && self.signed8 == other.signed8
                && self._pad1 == other._pad1 && self.signed16 == other.signed16
                && self.signed32 == other.signed32 && self.signed64 == other.signed64
                && self.signed128 == other.signed128
                && self.unsignedsize == other.unsignedsize
                && self.signedsize == other.signedsize && self.float32 == other.float32
                && self._pad3 == other._pad3 && self.float64 == other.float64
        }
    }
    impl Generate for PrimitivesPacked {
        fn generate<__R>(__rng: &mut __R) -> Self
        where
            __R: rand::Rng,
        {
            Self {
                unsigned8: <u8 as Generate>::generate(__rng),
                _pad0: <[u8; 1] as Generate>::generate(__rng),
                unsigned16: <u16 as Generate>::generate(__rng),
                unsigned32: <u32 as Generate>::generate(__rng),
                unsigned64: <u64 as Generate>::generate(__rng),
                #[cfg(not(feature = "no-128"))]
                unsigned128: <u128 as Generate>::generate(__rng),
                signed8: <i8 as Generate>::generate(__rng),
                _pad1: <[u8; 1] as Generate>::generate(__rng),
                signed16: <i16 as Generate>::generate(__rng),
                signed32: <i32 as Generate>::generate(__rng),
                signed64: <i64 as Generate>::generate(__rng),
                #[cfg(not(feature = "no-128"))]
                signed128: <i128 as Generate>::generate(__rng),
                #[cfg(not(feature = "no-usize"))]
                unsignedsize: <usize as Generate>::generate(__rng),
                #[cfg(not(feature = "no-usize"))]
                signedsize: <isize as Generate>::generate(__rng),
                float32: <f32 as Generate>::generate(__rng),
                _pad3: <[u8; 4] as Generate>::generate(__rng),
                float64: <f64 as Generate>::generate(__rng),
            }
        }
    }
    impl PartialEq<PrimitivesPacked> for &PrimitivesPacked {
        #[inline]
        fn eq(&self, other: &PrimitivesPacked) -> bool {
            *other == **self
        }
    }
    #[musli(mode = Packed, packed)]
    pub struct Primitives {
        boolean: bool,
        character: char,
        unsigned8: u8,
        unsigned16: u16,
        unsigned32: u32,
        unsigned64: u64,
        #[cfg(not(feature = "no-128"))]
        unsigned128: u128,
        signed8: i8,
        signed16: i16,
        signed32: i32,
        signed64: i64,
        #[cfg(not(feature = "no-128"))]
        signed128: i128,
        #[cfg(not(feature = "no-usize"))]
        unsignedsize: usize,
        #[cfg(not(feature = "no-usize"))]
        signedsize: isize,
        #[cfg(not(feature = "no-float"))]
        float32: f32,
        #[cfg(not(feature = "no-float"))]
        float64: f64,
    }
    const _: () = {
        #[automatically_derived]
        impl ::musli::en::Encode<Packed> for Primitives {
            #[inline]
            fn encode<T0>(
                &self,
                _1: &T0::Cx,
                _0: T0,
            ) -> ::musli::__priv::Result<
                <T0 as ::musli::en::Encoder>::Ok,
                <T0 as ::musli::en::Encoder>::Error,
            >
            where
                T0: ::musli::en::Encoder<Mode = Packed>,
            {
                ::musli::__priv::Ok({
                    ::musli::Context::enter_struct(_1, "Primitives");
                    let _3 = ::musli::en::Encoder::encode_pack_fn(
                        _0,
                        move |_2| {
                            ::musli::Context::enter_named_field(_1, "boolean", &0usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.boolean, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "character",
                                &1usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.character, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsigned8",
                                &2usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.unsigned8, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsigned16",
                                &3usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.unsigned16, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsigned32",
                                &4usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.unsigned32, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsigned64",
                                &5usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.unsigned64, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsigned128",
                                &6usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.unsigned128, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "signed8", &7usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.signed8, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "signed16", &8usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.signed16, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "signed32", &9usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.signed32, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "signed64",
                                &10usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.signed64, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "signed128",
                                &11usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.signed128, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsignedsize",
                                &12usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.unsignedsize, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "signedsize",
                                &13usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.signedsize, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "float32", &14usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.float32, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "float64", &15usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.float64, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::__priv::Ok(())
                        },
                    )?;
                    ::musli::Context::leave_struct(_1);
                    _3
                })
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        impl ::musli::en::Encode<::musli::mode::DefaultMode> for Primitives {
            #[inline]
            fn encode<T0>(
                &self,
                _1: &T0::Cx,
                _0: T0,
            ) -> ::musli::__priv::Result<
                <T0 as ::musli::en::Encoder>::Ok,
                <T0 as ::musli::en::Encoder>::Error,
            >
            where
                T0: ::musli::en::Encoder<Mode = ::musli::mode::DefaultMode>,
            {
                ::musli::__priv::Ok({
                    ::musli::Context::enter_struct(_1, "Primitives");
                    let _3 = ::musli::en::Encoder::encode_struct_fn(
                        _0,
                        16usize,
                        move |_0| {
                            ::musli::Context::enter_named_field(_1, "boolean", &0usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&0usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.boolean, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "character",
                                &1usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&1usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.character, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsigned8",
                                &2usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&2usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.unsigned8, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsigned16",
                                &3usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&3usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.unsigned16, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsigned32",
                                &4usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&4usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.unsigned32, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsigned64",
                                &5usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&5usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.unsigned64, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsigned128",
                                &6usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&6usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.unsigned128, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "signed8", &7usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&7usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.signed8, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "signed16", &8usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&8usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.signed16, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "signed32", &9usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&9usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.signed32, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "signed64",
                                &10usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&10usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.signed64, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "signed128",
                                &11usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&11usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.signed128, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "unsignedsize",
                                &12usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&12usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.unsignedsize, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "signedsize",
                                &13usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&13usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.signedsize, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "float32", &14usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&14usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.float32, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "float64", &15usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&15usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.float64, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::__priv::Ok(())
                        },
                    )?;
                    ::musli::Context::leave_struct(_1);
                    _3
                })
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        #[allow(clippy::init_numbered_fields)]
        #[allow(clippy::let_unit_value)]
        impl<'de> ::musli::de::Decode<'de, Packed> for Primitives {
            #[inline]
            fn decode<T0>(
                _0: &T0::Cx,
                _1: T0,
            ) -> ::musli::__priv::Result<Self, <T0::Cx as ::musli::Context>::Error>
            where
                T0: ::musli::de::Decoder<'de, Mode = Packed>,
            {
                ::musli::__priv::Ok({
                    {
                        ::musli::Context::enter_struct(_0, "Primitives");
                        let _3 = ::musli::de::Decoder::decode_pack(
                            _1,
                            move |_5| {
                                Ok(Self {
                                    boolean: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    character: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    unsigned8: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    unsigned16: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    unsigned32: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    unsigned64: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    unsigned128: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    signed8: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    signed16: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    signed32: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    signed64: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    signed128: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    unsignedsize: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    signedsize: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    float32: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    float64: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                })
                            },
                        )?;
                        ::musli::Context::leave_struct(_0);
                        _3
                    }
                })
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        #[allow(clippy::init_numbered_fields)]
        #[allow(clippy::let_unit_value)]
        impl<'de> ::musli::de::Decode<'de, ::musli::mode::DefaultMode> for Primitives {
            #[inline]
            fn decode<T0>(
                _0: &T0::Cx,
                _1: T0,
            ) -> ::musli::__priv::Result<Self, <T0::Cx as ::musli::Context>::Error>
            where
                T0: ::musli::de::Decoder<'de, Mode = ::musli::mode::DefaultMode>,
            {
                ::musli::__priv::Ok({
                    {
                        let mut _0_f: ::musli::__priv::Option<bool> = ::musli::__priv::None;
                        let mut _1_f: ::musli::__priv::Option<char> = ::musli::__priv::None;
                        let mut _2_f: ::musli::__priv::Option<u8> = ::musli::__priv::None;
                        let mut _3_f: ::musli::__priv::Option<u16> = ::musli::__priv::None;
                        let mut _4_f: ::musli::__priv::Option<u32> = ::musli::__priv::None;
                        let mut _5_f: ::musli::__priv::Option<u64> = ::musli::__priv::None;
                        let mut _6_f: ::musli::__priv::Option<u128> = ::musli::__priv::None;
                        let mut _7_f: ::musli::__priv::Option<i8> = ::musli::__priv::None;
                        let mut _8_f: ::musli::__priv::Option<i16> = ::musli::__priv::None;
                        let mut _9_f: ::musli::__priv::Option<i32> = ::musli::__priv::None;
                        let mut _10_f: ::musli::__priv::Option<i64> = ::musli::__priv::None;
                        let mut _11_f: ::musli::__priv::Option<i128> = ::musli::__priv::None;
                        let mut _12_f: ::musli::__priv::Option<usize> = ::musli::__priv::None;
                        let mut _13_f: ::musli::__priv::Option<isize> = ::musli::__priv::None;
                        let mut _14_f: ::musli::__priv::Option<f32> = ::musli::__priv::None;
                        let mut _15_f: ::musli::__priv::Option<f64> = ::musli::__priv::None;
                        ::musli::Context::enter_struct(_0, "Primitives");
                        ::musli::de::Decoder::decode_struct(
                            _1,
                            Some(16usize),
                            move |_5| {
                                while let ::musli::__priv::Some(mut _3) = ::musli::de::StructDecoder::decode_field(
                                    _5,
                                )? {
                                    let _2 = {
                                        let _3 = ::musli::de::StructFieldDecoder::decode_field_name(
                                            &mut _3,
                                        )?;
                                        ::musli::de::Decode::<
                                            ::musli::mode::DefaultMode,
                                        >::decode(_0, _3)?
                                    };
                                    match _2 {
                                        0usize => {
                                            ::musli::Context::enter_named_field(_0, "boolean", &0usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _0_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        1usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "character",
                                                &1usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _1_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        2usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "unsigned8",
                                                &2usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _2_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        3usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "unsigned16",
                                                &3usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _3_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        4usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "unsigned32",
                                                &4usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _4_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        5usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "unsigned64",
                                                &5usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _5_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        6usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "unsigned128",
                                                &6usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _6_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        7usize => {
                                            ::musli::Context::enter_named_field(_0, "signed8", &7usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _7_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        8usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "signed16",
                                                &8usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _8_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        9usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "signed32",
                                                &9usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _9_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        10usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "signed64",
                                                &10usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _10_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        11usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "signed128",
                                                &11usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _11_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        12usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "unsignedsize",
                                                &12usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _12_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        13usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "signedsize",
                                                &13usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _13_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        14usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "float32",
                                                &14usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _14_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        15usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "float64",
                                                &15usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _15_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        _2 => {
                                            if ::musli::__priv::skip_field(_3)? {
                                                return ::musli::__priv::Err(
                                                    ::musli::Context::invalid_field_tag(_0, "Primitives", &_2),
                                                );
                                            }
                                        }
                                    }
                                }
                                ::musli::Context::leave_struct(_0);
                                ::musli::__priv::Ok(Self {
                                    boolean: match _0_f {
                                        ::musli::__priv::Some(_0_f) => _0_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Primitives", &0usize),
                                            );
                                        }
                                    },
                                    character: match _1_f {
                                        ::musli::__priv::Some(_1_f) => _1_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Primitives", &1usize),
                                            );
                                        }
                                    },
                                    unsigned8: match _2_f {
                                        ::musli::__priv::Some(_2_f) => _2_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Primitives", &2usize),
                                            );
                                        }
                                    },
                                    unsigned16: match _3_f {
                                        ::musli::__priv::Some(_3_f) => _3_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Primitives", &3usize),
                                            );
                                        }
                                    },
                                    unsigned32: match _4_f {
                                        ::musli::__priv::Some(_4_f) => _4_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Primitives", &4usize),
                                            );
                                        }
                                    },
                                    unsigned64: match _5_f {
                                        ::musli::__priv::Some(_5_f) => _5_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Primitives", &5usize),
                                            );
                                        }
                                    },
                                    unsigned128: match _6_f {
                                        ::musli::__priv::Some(_6_f) => _6_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Primitives", &6usize),
                                            );
                                        }
                                    },
                                    signed8: match _7_f {
                                        ::musli::__priv::Some(_7_f) => _7_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Primitives", &7usize),
                                            );
                                        }
                                    },
                                    signed16: match _8_f {
                                        ::musli::__priv::Some(_8_f) => _8_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Primitives", &8usize),
                                            );
                                        }
                                    },
                                    signed32: match _9_f {
                                        ::musli::__priv::Some(_9_f) => _9_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Primitives", &9usize),
                                            );
                                        }
                                    },
                                    signed64: match _10_f {
                                        ::musli::__priv::Some(_10_f) => _10_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Primitives", &10usize),
                                            );
                                        }
                                    },
                                    signed128: match _11_f {
                                        ::musli::__priv::Some(_11_f) => _11_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Primitives", &11usize),
                                            );
                                        }
                                    },
                                    unsignedsize: match _12_f {
                                        ::musli::__priv::Some(_12_f) => _12_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Primitives", &12usize),
                                            );
                                        }
                                    },
                                    signedsize: match _13_f {
                                        ::musli::__priv::Some(_13_f) => _13_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Primitives", &13usize),
                                            );
                                        }
                                    },
                                    float32: match _14_f {
                                        ::musli::__priv::Some(_14_f) => _14_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Primitives", &14usize),
                                            );
                                        }
                                    },
                                    float64: match _15_f {
                                        ::musli::__priv::Some(_15_f) => _15_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Primitives", &15usize),
                                            );
                                        }
                                    },
                                })
                            },
                        )?
                    }
                })
            }
        }
    };
    #[automatically_derived]
    impl ::core::fmt::Debug for Primitives {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "boolean",
                "character",
                "unsigned8",
                "unsigned16",
                "unsigned32",
                "unsigned64",
                "unsigned128",
                "signed8",
                "signed16",
                "signed32",
                "signed64",
                "signed128",
                "unsignedsize",
                "signedsize",
                "float32",
                "float64",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.boolean,
                &self.character,
                &self.unsigned8,
                &self.unsigned16,
                &self.unsigned32,
                &self.unsigned64,
                &self.unsigned128,
                &self.signed8,
                &self.signed16,
                &self.signed32,
                &self.signed64,
                &self.signed128,
                &self.unsignedsize,
                &self.signedsize,
                &self.float32,
                &&self.float64,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "Primitives",
                names,
                values,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Primitives {
        #[inline]
        fn clone(&self) -> Primitives {
            Primitives {
                boolean: ::core::clone::Clone::clone(&self.boolean),
                character: ::core::clone::Clone::clone(&self.character),
                unsigned8: ::core::clone::Clone::clone(&self.unsigned8),
                unsigned16: ::core::clone::Clone::clone(&self.unsigned16),
                unsigned32: ::core::clone::Clone::clone(&self.unsigned32),
                unsigned64: ::core::clone::Clone::clone(&self.unsigned64),
                unsigned128: ::core::clone::Clone::clone(&self.unsigned128),
                signed8: ::core::clone::Clone::clone(&self.signed8),
                signed16: ::core::clone::Clone::clone(&self.signed16),
                signed32: ::core::clone::Clone::clone(&self.signed32),
                signed64: ::core::clone::Clone::clone(&self.signed64),
                signed128: ::core::clone::Clone::clone(&self.signed128),
                unsignedsize: ::core::clone::Clone::clone(&self.unsignedsize),
                signedsize: ::core::clone::Clone::clone(&self.signedsize),
                float32: ::core::clone::Clone::clone(&self.float32),
                float64: ::core::clone::Clone::clone(&self.float64),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Primitives {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Primitives {
        #[inline]
        fn eq(&self, other: &Primitives) -> bool {
            self.boolean == other.boolean && self.character == other.character
                && self.unsigned8 == other.unsigned8
                && self.unsigned16 == other.unsigned16
                && self.unsigned32 == other.unsigned32
                && self.unsigned64 == other.unsigned64
                && self.unsigned128 == other.unsigned128 && self.signed8 == other.signed8
                && self.signed16 == other.signed16 && self.signed32 == other.signed32
                && self.signed64 == other.signed64 && self.signed128 == other.signed128
                && self.unsignedsize == other.unsignedsize
                && self.signedsize == other.signedsize && self.float32 == other.float32
                && self.float64 == other.float64
        }
    }
    impl Generate for Primitives {
        fn generate<__R>(__rng: &mut __R) -> Self
        where
            __R: rand::Rng,
        {
            Self {
                boolean: <bool as Generate>::generate(__rng),
                character: <char as Generate>::generate(__rng),
                unsigned8: <u8 as Generate>::generate(__rng),
                unsigned16: <u16 as Generate>::generate(__rng),
                unsigned32: <u32 as Generate>::generate(__rng),
                unsigned64: <u64 as Generate>::generate(__rng),
                #[cfg(not(feature = "no-128"))]
                unsigned128: <u128 as Generate>::generate(__rng),
                signed8: <i8 as Generate>::generate(__rng),
                signed16: <i16 as Generate>::generate(__rng),
                signed32: <i32 as Generate>::generate(__rng),
                signed64: <i64 as Generate>::generate(__rng),
                #[cfg(not(feature = "no-128"))]
                signed128: <i128 as Generate>::generate(__rng),
                #[cfg(not(feature = "no-usize"))]
                unsignedsize: <usize as Generate>::generate(__rng),
                #[cfg(not(feature = "no-usize"))]
                signedsize: <isize as Generate>::generate(__rng),
                #[cfg(not(feature = "no-float"))]
                float32: <f32 as Generate>::generate(__rng),
                #[cfg(not(feature = "no-float"))]
                float64: <f64 as Generate>::generate(__rng),
            }
        }
    }
    impl PartialEq<Primitives> for &Primitives {
        #[inline]
        fn eq(&self, other: &Primitives) -> bool {
            *other == **self
        }
    }
    #[musli(mode = Packed, packed)]
    pub struct Allocated {
        string: String,
        #[musli(bytes)]
        #[generate(range = SMALL_FIELDS)]
        bytes: Vec<u8>,
        #[cfg(all(not(feature = "no-map"), not(feature = "no-number-key")))]
        #[generate(range = SMALL_FIELDS)]
        number_map: HashMap<u32, u64>,
        #[cfg(all(not(feature = "no-map"), not(feature = "no-string-key")))]
        #[generate(range = SMALL_FIELDS)]
        string_map: HashMap<String, u64>,
        #[generate(range = SMALL_FIELDS)]
        number_set: HashSet<u32>,
        #[generate(range = SMALL_FIELDS)]
        #[cfg(not(feature = "no-string-set"))]
        string_set: HashSet<String>,
        #[cfg(all(not(feature = "no-btree"), not(feature = "no-number-key")))]
        #[generate(range = SMALL_FIELDS)]
        number_btree: BTreeMap<u32, u64>,
        #[cfg(not(feature = "no-btree"))]
        #[generate(range = SMALL_FIELDS)]
        string_btree: BTreeMap<String, u64>,
        #[cfg(not(feature = "no-btree"))]
        #[generate(range = SMALL_FIELDS)]
        number_btree_set: BTreeSet<u32>,
        #[cfg(not(feature = "no-btree"))]
        #[generate(range = SMALL_FIELDS)]
        string_btree_set: BTreeSet<String>,
        #[cfg(not(feature = "no-cstring"))]
        c_string: CString,
    }
    const _: () = {
        #[automatically_derived]
        impl ::musli::en::Encode<Packed> for Allocated {
            #[inline]
            fn encode<T0>(
                &self,
                _1: &T0::Cx,
                _0: T0,
            ) -> ::musli::__priv::Result<
                <T0 as ::musli::en::Encoder>::Ok,
                <T0 as ::musli::en::Encoder>::Error,
            >
            where
                T0: ::musli::en::Encoder<Mode = Packed>,
            {
                ::musli::__priv::Ok({
                    ::musli::Context::enter_struct(_1, "Allocated");
                    let _3 = ::musli::en::Encoder::encode_pack_fn(
                        _0,
                        move |_2| {
                            ::musli::Context::enter_named_field(_1, "string", &0usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<Packed>::encode(&self.string, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "bytes", &1usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::EncodeBytes::<
                                Packed,
                            >::encode_bytes(&self.bytes, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "number_map",
                                &2usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.number_map, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "string_map",
                                &3usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.string_map, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "number_set",
                                &4usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.number_set, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "string_set",
                                &5usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.string_set, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "number_btree",
                                &6usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.number_btree, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "string_btree",
                                &7usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.string_btree, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "number_btree_set",
                                &8usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.number_btree_set, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "string_btree_set",
                                &9usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.string_btree_set, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "c_string",
                                &10usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.c_string, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::__priv::Ok(())
                        },
                    )?;
                    ::musli::Context::leave_struct(_1);
                    _3
                })
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        impl ::musli::en::Encode<::musli::mode::DefaultMode> for Allocated {
            #[inline]
            fn encode<T0>(
                &self,
                _1: &T0::Cx,
                _0: T0,
            ) -> ::musli::__priv::Result<
                <T0 as ::musli::en::Encoder>::Ok,
                <T0 as ::musli::en::Encoder>::Error,
            >
            where
                T0: ::musli::en::Encoder<Mode = ::musli::mode::DefaultMode>,
            {
                ::musli::__priv::Ok({
                    ::musli::Context::enter_struct(_1, "Allocated");
                    let _3 = ::musli::en::Encoder::encode_struct_fn(
                        _0,
                        11usize,
                        move |_0| {
                            ::musli::Context::enter_named_field(_1, "string", &0usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&0usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.string, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "bytes", &1usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&1usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::EncodeBytes::<
                                        ::musli::mode::DefaultMode,
                                    >::encode_bytes(&self.bytes, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "number_map",
                                &2usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&2usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.number_map, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "string_map",
                                &3usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&3usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.string_map, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "number_set",
                                &4usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&4usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.number_set, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "string_set",
                                &5usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&5usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.string_set, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "number_btree",
                                &6usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&6usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.number_btree, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "string_btree",
                                &7usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&7usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.string_btree, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "number_btree_set",
                                &8usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&8usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.number_btree_set, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "string_btree_set",
                                &9usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&9usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.string_btree_set, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "c_string",
                                &10usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&10usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.c_string, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::__priv::Ok(())
                        },
                    )?;
                    ::musli::Context::leave_struct(_1);
                    _3
                })
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        #[allow(clippy::init_numbered_fields)]
        #[allow(clippy::let_unit_value)]
        impl<'de> ::musli::de::Decode<'de, Packed> for Allocated {
            #[inline]
            fn decode<T0>(
                _0: &T0::Cx,
                _1: T0,
            ) -> ::musli::__priv::Result<Self, <T0::Cx as ::musli::Context>::Error>
            where
                T0: ::musli::de::Decoder<'de, Mode = Packed>,
            {
                ::musli::__priv::Ok({
                    {
                        ::musli::Context::enter_struct(_0, "Allocated");
                        let _3 = ::musli::de::Decoder::decode_pack(
                            _1,
                            move |_5| {
                                Ok(Self {
                                    string: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    bytes: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::DecodeBytes::<Packed>::decode_bytes(_0, _4)?
                                    },
                                    number_map: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    string_map: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    number_set: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    string_set: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    number_btree: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    string_btree: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    number_btree_set: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    string_btree_set: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    c_string: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                })
                            },
                        )?;
                        ::musli::Context::leave_struct(_0);
                        _3
                    }
                })
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        #[allow(clippy::init_numbered_fields)]
        #[allow(clippy::let_unit_value)]
        impl<'de> ::musli::de::Decode<'de, ::musli::mode::DefaultMode> for Allocated {
            #[inline]
            fn decode<T0>(
                _0: &T0::Cx,
                _1: T0,
            ) -> ::musli::__priv::Result<Self, <T0::Cx as ::musli::Context>::Error>
            where
                T0: ::musli::de::Decoder<'de, Mode = ::musli::mode::DefaultMode>,
            {
                ::musli::__priv::Ok({
                    {
                        let mut _0_f: ::musli::__priv::Option<String> = ::musli::__priv::None;
                        let mut _1_f: ::musli::__priv::Option<Vec<u8>> = ::musli::__priv::None;
                        let mut _2_f: ::musli::__priv::Option<HashMap<u32, u64>> = ::musli::__priv::None;
                        let mut _3_f: ::musli::__priv::Option<HashMap<String, u64>> = ::musli::__priv::None;
                        let mut _4_f: ::musli::__priv::Option<HashSet<u32>> = ::musli::__priv::None;
                        let mut _5_f: ::musli::__priv::Option<HashSet<String>> = ::musli::__priv::None;
                        let mut _6_f: ::musli::__priv::Option<BTreeMap<u32, u64>> = ::musli::__priv::None;
                        let mut _7_f: ::musli::__priv::Option<BTreeMap<String, u64>> = ::musli::__priv::None;
                        let mut _8_f: ::musli::__priv::Option<BTreeSet<u32>> = ::musli::__priv::None;
                        let mut _9_f: ::musli::__priv::Option<BTreeSet<String>> = ::musli::__priv::None;
                        let mut _10_f: ::musli::__priv::Option<CString> = ::musli::__priv::None;
                        ::musli::Context::enter_struct(_0, "Allocated");
                        ::musli::de::Decoder::decode_struct(
                            _1,
                            Some(11usize),
                            move |_5| {
                                while let ::musli::__priv::Some(mut _3) = ::musli::de::StructDecoder::decode_field(
                                    _5,
                                )? {
                                    let _2 = {
                                        let _3 = ::musli::de::StructFieldDecoder::decode_field_name(
                                            &mut _3,
                                        )?;
                                        ::musli::de::Decode::<
                                            ::musli::mode::DefaultMode,
                                        >::decode(_0, _3)?
                                    };
                                    match _2 {
                                        0usize => {
                                            ::musli::Context::enter_named_field(_0, "string", &0usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _0_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        1usize => {
                                            ::musli::Context::enter_named_field(_0, "bytes", &1usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _1_f = ::musli::__priv::Some(
                                                ::musli::de::DecodeBytes::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode_bytes(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        2usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "number_map",
                                                &2usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _2_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        3usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "string_map",
                                                &3usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _3_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        4usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "number_set",
                                                &4usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _4_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        5usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "string_set",
                                                &5usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _5_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        6usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "number_btree",
                                                &6usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _6_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        7usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "string_btree",
                                                &7usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _7_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        8usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "number_btree_set",
                                                &8usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _8_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        9usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "string_btree_set",
                                                &9usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _9_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        10usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "c_string",
                                                &10usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _10_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        _2 => {
                                            if ::musli::__priv::skip_field(_3)? {
                                                return ::musli::__priv::Err(
                                                    ::musli::Context::invalid_field_tag(_0, "Allocated", &_2),
                                                );
                                            }
                                        }
                                    }
                                }
                                ::musli::Context::leave_struct(_0);
                                ::musli::__priv::Ok(Self {
                                    string: match _0_f {
                                        ::musli::__priv::Some(_0_f) => _0_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Allocated", &0usize),
                                            );
                                        }
                                    },
                                    bytes: match _1_f {
                                        ::musli::__priv::Some(_1_f) => _1_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Allocated", &1usize),
                                            );
                                        }
                                    },
                                    number_map: match _2_f {
                                        ::musli::__priv::Some(_2_f) => _2_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Allocated", &2usize),
                                            );
                                        }
                                    },
                                    string_map: match _3_f {
                                        ::musli::__priv::Some(_3_f) => _3_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Allocated", &3usize),
                                            );
                                        }
                                    },
                                    number_set: match _4_f {
                                        ::musli::__priv::Some(_4_f) => _4_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Allocated", &4usize),
                                            );
                                        }
                                    },
                                    string_set: match _5_f {
                                        ::musli::__priv::Some(_5_f) => _5_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Allocated", &5usize),
                                            );
                                        }
                                    },
                                    number_btree: match _6_f {
                                        ::musli::__priv::Some(_6_f) => _6_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Allocated", &6usize),
                                            );
                                        }
                                    },
                                    string_btree: match _7_f {
                                        ::musli::__priv::Some(_7_f) => _7_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Allocated", &7usize),
                                            );
                                        }
                                    },
                                    number_btree_set: match _8_f {
                                        ::musli::__priv::Some(_8_f) => _8_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Allocated", &8usize),
                                            );
                                        }
                                    },
                                    string_btree_set: match _9_f {
                                        ::musli::__priv::Some(_9_f) => _9_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Allocated", &9usize),
                                            );
                                        }
                                    },
                                    c_string: match _10_f {
                                        ::musli::__priv::Some(_10_f) => _10_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Allocated", &10usize),
                                            );
                                        }
                                    },
                                })
                            },
                        )?
                    }
                })
            }
        }
    };
    #[automatically_derived]
    impl ::core::fmt::Debug for Allocated {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "string",
                "bytes",
                "number_map",
                "string_map",
                "number_set",
                "string_set",
                "number_btree",
                "string_btree",
                "number_btree_set",
                "string_btree_set",
                "c_string",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.string,
                &self.bytes,
                &self.number_map,
                &self.string_map,
                &self.number_set,
                &self.string_set,
                &self.number_btree,
                &self.string_btree,
                &self.number_btree_set,
                &self.string_btree_set,
                &&self.c_string,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "Allocated",
                names,
                values,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Allocated {
        #[inline]
        fn clone(&self) -> Allocated {
            Allocated {
                string: ::core::clone::Clone::clone(&self.string),
                bytes: ::core::clone::Clone::clone(&self.bytes),
                number_map: ::core::clone::Clone::clone(&self.number_map),
                string_map: ::core::clone::Clone::clone(&self.string_map),
                number_set: ::core::clone::Clone::clone(&self.number_set),
                string_set: ::core::clone::Clone::clone(&self.string_set),
                number_btree: ::core::clone::Clone::clone(&self.number_btree),
                string_btree: ::core::clone::Clone::clone(&self.string_btree),
                number_btree_set: ::core::clone::Clone::clone(&self.number_btree_set),
                string_btree_set: ::core::clone::Clone::clone(&self.string_btree_set),
                c_string: ::core::clone::Clone::clone(&self.c_string),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Allocated {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Allocated {
        #[inline]
        fn eq(&self, other: &Allocated) -> bool {
            self.string == other.string && self.bytes == other.bytes
                && self.number_map == other.number_map
                && self.string_map == other.string_map
                && self.number_set == other.number_set
                && self.string_set == other.string_set
                && self.number_btree == other.number_btree
                && self.string_btree == other.string_btree
                && self.number_btree_set == other.number_btree_set
                && self.string_btree_set == other.string_btree_set
                && self.c_string == other.c_string
        }
    }
    impl Generate for Allocated {
        fn generate<__R>(__rng: &mut __R) -> Self
        where
            __R: rand::Rng,
        {
            Self {
                string: <String as Generate>::generate(__rng),
                bytes: <Vec<u8> as Generate>::generate_range(__rng, SMALL_FIELDS),
                #[cfg(all(not(feature = "no-map"), not(feature = "no-number-key")))]
                number_map: <HashMap<
                    u32,
                    u64,
                > as Generate>::generate_range(__rng, SMALL_FIELDS),
                #[cfg(all(not(feature = "no-map"), not(feature = "no-string-key")))]
                string_map: <HashMap<
                    String,
                    u64,
                > as Generate>::generate_range(__rng, SMALL_FIELDS),
                number_set: <HashSet<
                    u32,
                > as Generate>::generate_range(__rng, SMALL_FIELDS),
                #[cfg(not(feature = "no-string-set"))]
                string_set: <HashSet<
                    String,
                > as Generate>::generate_range(__rng, SMALL_FIELDS),
                #[cfg(all(not(feature = "no-btree"), not(feature = "no-number-key")))]
                number_btree: <BTreeMap<
                    u32,
                    u64,
                > as Generate>::generate_range(__rng, SMALL_FIELDS),
                #[cfg(not(feature = "no-btree"))]
                string_btree: <BTreeMap<
                    String,
                    u64,
                > as Generate>::generate_range(__rng, SMALL_FIELDS),
                #[cfg(not(feature = "no-btree"))]
                number_btree_set: <BTreeSet<
                    u32,
                > as Generate>::generate_range(__rng, SMALL_FIELDS),
                #[cfg(not(feature = "no-btree"))]
                string_btree_set: <BTreeSet<
                    String,
                > as Generate>::generate_range(__rng, SMALL_FIELDS),
                #[cfg(not(feature = "no-cstring"))]
                c_string: <CString as Generate>::generate(__rng),
            }
        }
    }
    impl PartialEq<Allocated> for &Allocated {
        #[inline]
        fn eq(&self, other: &Allocated) -> bool {
            *other == **self
        }
    }
    #[musli(mode = Packed, packed)]
    pub struct Tuples {
        u0: (),
        u1: (bool,),
        u2: (bool, u8),
        u3: (bool, u8, u32),
        u4: (bool, u8, u32, u64),
        #[cfg(not(feature = "no-float"))]
        u5: (bool, u8, u32, u64, f32),
        #[cfg(not(feature = "no-float"))]
        u6: (bool, u8, u32, u64, f32, f64),
        i0: (),
        i1: (bool,),
        i2: (bool, i8),
        i3: (bool, i8, i32),
        i4: (bool, i8, i32, i64),
        #[cfg(not(feature = "no-float"))]
        i5: (bool, i8, i32, i64, f32),
        #[cfg(not(feature = "no-float"))]
        i6: (bool, i8, i32, i64, f32, f64),
    }
    const _: () = {
        #[automatically_derived]
        impl ::musli::en::Encode<Packed> for Tuples {
            #[inline]
            fn encode<T0>(
                &self,
                _1: &T0::Cx,
                _0: T0,
            ) -> ::musli::__priv::Result<
                <T0 as ::musli::en::Encoder>::Ok,
                <T0 as ::musli::en::Encoder>::Error,
            >
            where
                T0: ::musli::en::Encoder<Mode = Packed>,
            {
                ::musli::__priv::Ok({
                    ::musli::Context::enter_struct(_1, "Tuples");
                    let _3 = ::musli::en::Encoder::encode_pack_fn(
                        _0,
                        move |_2| {
                            ::musli::Context::enter_named_field(_1, "u0", &0usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<Packed>::encode(&self.u0, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "u1", &1usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<Packed>::encode(&self.u1, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "u2", &2usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<Packed>::encode(&self.u2, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "u3", &3usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<Packed>::encode(&self.u3, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "u4", &4usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<Packed>::encode(&self.u4, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "u5", &5usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<Packed>::encode(&self.u5, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "u6", &6usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<Packed>::encode(&self.u6, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "i0", &7usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<Packed>::encode(&self.i0, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "i1", &8usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<Packed>::encode(&self.i1, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "i2", &9usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<Packed>::encode(&self.i2, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "i3", &10usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<Packed>::encode(&self.i3, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "i4", &11usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<Packed>::encode(&self.i4, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "i5", &12usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<Packed>::encode(&self.i5, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "i6", &13usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<Packed>::encode(&self.i6, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::__priv::Ok(())
                        },
                    )?;
                    ::musli::Context::leave_struct(_1);
                    _3
                })
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        impl ::musli::en::Encode<::musli::mode::DefaultMode> for Tuples {
            #[inline]
            fn encode<T0>(
                &self,
                _1: &T0::Cx,
                _0: T0,
            ) -> ::musli::__priv::Result<
                <T0 as ::musli::en::Encoder>::Ok,
                <T0 as ::musli::en::Encoder>::Error,
            >
            where
                T0: ::musli::en::Encoder<Mode = ::musli::mode::DefaultMode>,
            {
                ::musli::__priv::Ok({
                    ::musli::Context::enter_struct(_1, "Tuples");
                    let _3 = ::musli::en::Encoder::encode_struct_fn(
                        _0,
                        14usize,
                        move |_0| {
                            ::musli::Context::enter_named_field(_1, "u0", &0usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&0usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.u0, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "u1", &1usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&1usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.u1, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "u2", &2usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&2usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.u2, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "u3", &3usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&3usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.u3, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "u4", &4usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&4usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.u4, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "u5", &5usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&5usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.u5, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "u6", &6usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&6usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.u6, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "i0", &7usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&7usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.i0, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "i1", &8usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&8usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.i1, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "i2", &9usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&9usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.i2, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "i3", &10usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&10usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.i3, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "i4", &11usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&11usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.i4, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "i5", &12usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&12usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.i5, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "i6", &13usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&13usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.i6, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::__priv::Ok(())
                        },
                    )?;
                    ::musli::Context::leave_struct(_1);
                    _3
                })
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        #[allow(clippy::init_numbered_fields)]
        #[allow(clippy::let_unit_value)]
        impl<'de> ::musli::de::Decode<'de, Packed> for Tuples {
            #[inline]
            fn decode<T0>(
                _0: &T0::Cx,
                _1: T0,
            ) -> ::musli::__priv::Result<Self, <T0::Cx as ::musli::Context>::Error>
            where
                T0: ::musli::de::Decoder<'de, Mode = Packed>,
            {
                ::musli::__priv::Ok({
                    {
                        ::musli::Context::enter_struct(_0, "Tuples");
                        let _3 = ::musli::de::Decoder::decode_pack(
                            _1,
                            move |_5| {
                                Ok(Self {
                                    u0: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    u1: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    u2: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    u3: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    u4: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    u5: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    u6: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    i0: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    i1: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    i2: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    i3: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    i4: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    i5: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    i6: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                })
                            },
                        )?;
                        ::musli::Context::leave_struct(_0);
                        _3
                    }
                })
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        #[allow(clippy::init_numbered_fields)]
        #[allow(clippy::let_unit_value)]
        impl<'de> ::musli::de::Decode<'de, ::musli::mode::DefaultMode> for Tuples {
            #[inline]
            fn decode<T0>(
                _0: &T0::Cx,
                _1: T0,
            ) -> ::musli::__priv::Result<Self, <T0::Cx as ::musli::Context>::Error>
            where
                T0: ::musli::de::Decoder<'de, Mode = ::musli::mode::DefaultMode>,
            {
                ::musli::__priv::Ok({
                    {
                        let mut _0_f: ::musli::__priv::Option<()> = ::musli::__priv::None;
                        let mut _1_f: ::musli::__priv::Option<(bool,)> = ::musli::__priv::None;
                        let mut _2_f: ::musli::__priv::Option<(bool, u8)> = ::musli::__priv::None;
                        let mut _3_f: ::musli::__priv::Option<(bool, u8, u32)> = ::musli::__priv::None;
                        let mut _4_f: ::musli::__priv::Option<(bool, u8, u32, u64)> = ::musli::__priv::None;
                        let mut _5_f: ::musli::__priv::Option<
                            (bool, u8, u32, u64, f32),
                        > = ::musli::__priv::None;
                        let mut _6_f: ::musli::__priv::Option<
                            (bool, u8, u32, u64, f32, f64),
                        > = ::musli::__priv::None;
                        let mut _7_f: ::musli::__priv::Option<()> = ::musli::__priv::None;
                        let mut _8_f: ::musli::__priv::Option<(bool,)> = ::musli::__priv::None;
                        let mut _9_f: ::musli::__priv::Option<(bool, i8)> = ::musli::__priv::None;
                        let mut _10_f: ::musli::__priv::Option<(bool, i8, i32)> = ::musli::__priv::None;
                        let mut _11_f: ::musli::__priv::Option<(bool, i8, i32, i64)> = ::musli::__priv::None;
                        let mut _12_f: ::musli::__priv::Option<
                            (bool, i8, i32, i64, f32),
                        > = ::musli::__priv::None;
                        let mut _13_f: ::musli::__priv::Option<
                            (bool, i8, i32, i64, f32, f64),
                        > = ::musli::__priv::None;
                        ::musli::Context::enter_struct(_0, "Tuples");
                        ::musli::de::Decoder::decode_struct(
                            _1,
                            Some(14usize),
                            move |_5| {
                                while let ::musli::__priv::Some(mut _3) = ::musli::de::StructDecoder::decode_field(
                                    _5,
                                )? {
                                    let _2 = {
                                        let _3 = ::musli::de::StructFieldDecoder::decode_field_name(
                                            &mut _3,
                                        )?;
                                        ::musli::de::Decode::<
                                            ::musli::mode::DefaultMode,
                                        >::decode(_0, _3)?
                                    };
                                    match _2 {
                                        0usize => {
                                            ::musli::Context::enter_named_field(_0, "u0", &0usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _0_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        1usize => {
                                            ::musli::Context::enter_named_field(_0, "u1", &1usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _1_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        2usize => {
                                            ::musli::Context::enter_named_field(_0, "u2", &2usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _2_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        3usize => {
                                            ::musli::Context::enter_named_field(_0, "u3", &3usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _3_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        4usize => {
                                            ::musli::Context::enter_named_field(_0, "u4", &4usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _4_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        5usize => {
                                            ::musli::Context::enter_named_field(_0, "u5", &5usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _5_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        6usize => {
                                            ::musli::Context::enter_named_field(_0, "u6", &6usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _6_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        7usize => {
                                            ::musli::Context::enter_named_field(_0, "i0", &7usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _7_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        8usize => {
                                            ::musli::Context::enter_named_field(_0, "i1", &8usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _8_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        9usize => {
                                            ::musli::Context::enter_named_field(_0, "i2", &9usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _9_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        10usize => {
                                            ::musli::Context::enter_named_field(_0, "i3", &10usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _10_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        11usize => {
                                            ::musli::Context::enter_named_field(_0, "i4", &11usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _11_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        12usize => {
                                            ::musli::Context::enter_named_field(_0, "i5", &12usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _12_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        13usize => {
                                            ::musli::Context::enter_named_field(_0, "i6", &13usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _13_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        _2 => {
                                            if ::musli::__priv::skip_field(_3)? {
                                                return ::musli::__priv::Err(
                                                    ::musli::Context::invalid_field_tag(_0, "Tuples", &_2),
                                                );
                                            }
                                        }
                                    }
                                }
                                ::musli::Context::leave_struct(_0);
                                ::musli::__priv::Ok(Self {
                                    u0: match _0_f {
                                        ::musli::__priv::Some(_0_f) => _0_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Tuples", &0usize),
                                            );
                                        }
                                    },
                                    u1: match _1_f {
                                        ::musli::__priv::Some(_1_f) => _1_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Tuples", &1usize),
                                            );
                                        }
                                    },
                                    u2: match _2_f {
                                        ::musli::__priv::Some(_2_f) => _2_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Tuples", &2usize),
                                            );
                                        }
                                    },
                                    u3: match _3_f {
                                        ::musli::__priv::Some(_3_f) => _3_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Tuples", &3usize),
                                            );
                                        }
                                    },
                                    u4: match _4_f {
                                        ::musli::__priv::Some(_4_f) => _4_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Tuples", &4usize),
                                            );
                                        }
                                    },
                                    u5: match _5_f {
                                        ::musli::__priv::Some(_5_f) => _5_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Tuples", &5usize),
                                            );
                                        }
                                    },
                                    u6: match _6_f {
                                        ::musli::__priv::Some(_6_f) => _6_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Tuples", &6usize),
                                            );
                                        }
                                    },
                                    i0: match _7_f {
                                        ::musli::__priv::Some(_7_f) => _7_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Tuples", &7usize),
                                            );
                                        }
                                    },
                                    i1: match _8_f {
                                        ::musli::__priv::Some(_8_f) => _8_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Tuples", &8usize),
                                            );
                                        }
                                    },
                                    i2: match _9_f {
                                        ::musli::__priv::Some(_9_f) => _9_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Tuples", &9usize),
                                            );
                                        }
                                    },
                                    i3: match _10_f {
                                        ::musli::__priv::Some(_10_f) => _10_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Tuples", &10usize),
                                            );
                                        }
                                    },
                                    i4: match _11_f {
                                        ::musli::__priv::Some(_11_f) => _11_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Tuples", &11usize),
                                            );
                                        }
                                    },
                                    i5: match _12_f {
                                        ::musli::__priv::Some(_12_f) => _12_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Tuples", &12usize),
                                            );
                                        }
                                    },
                                    i6: match _13_f {
                                        ::musli::__priv::Some(_13_f) => _13_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "Tuples", &13usize),
                                            );
                                        }
                                    },
                                })
                            },
                        )?
                    }
                })
            }
        }
    };
    #[automatically_derived]
    impl ::core::fmt::Debug for Tuples {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "u0",
                "u1",
                "u2",
                "u3",
                "u4",
                "u5",
                "u6",
                "i0",
                "i1",
                "i2",
                "i3",
                "i4",
                "i5",
                "i6",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.u0,
                &self.u1,
                &self.u2,
                &self.u3,
                &self.u4,
                &self.u5,
                &self.u6,
                &self.i0,
                &self.i1,
                &self.i2,
                &self.i3,
                &self.i4,
                &self.i5,
                &&self.i6,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "Tuples",
                names,
                values,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Tuples {
        #[inline]
        fn clone(&self) -> Tuples {
            Tuples {
                u0: ::core::clone::Clone::clone(&self.u0),
                u1: ::core::clone::Clone::clone(&self.u1),
                u2: ::core::clone::Clone::clone(&self.u2),
                u3: ::core::clone::Clone::clone(&self.u3),
                u4: ::core::clone::Clone::clone(&self.u4),
                u5: ::core::clone::Clone::clone(&self.u5),
                u6: ::core::clone::Clone::clone(&self.u6),
                i0: ::core::clone::Clone::clone(&self.i0),
                i1: ::core::clone::Clone::clone(&self.i1),
                i2: ::core::clone::Clone::clone(&self.i2),
                i3: ::core::clone::Clone::clone(&self.i3),
                i4: ::core::clone::Clone::clone(&self.i4),
                i5: ::core::clone::Clone::clone(&self.i5),
                i6: ::core::clone::Clone::clone(&self.i6),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Tuples {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Tuples {
        #[inline]
        fn eq(&self, other: &Tuples) -> bool {
            self.u0 == other.u0 && self.u1 == other.u1 && self.u2 == other.u2
                && self.u3 == other.u3 && self.u4 == other.u4 && self.u5 == other.u5
                && self.u6 == other.u6 && self.i0 == other.i0 && self.i1 == other.i1
                && self.i2 == other.i2 && self.i3 == other.i3 && self.i4 == other.i4
                && self.i5 == other.i5 && self.i6 == other.i6
        }
    }
    impl Generate for Tuples {
        fn generate<__R>(__rng: &mut __R) -> Self
        where
            __R: rand::Rng,
        {
            Self {
                u0: <() as Generate>::generate(__rng),
                u1: <(bool,) as Generate>::generate(__rng),
                u2: <(bool, u8) as Generate>::generate(__rng),
                u3: <(bool, u8, u32) as Generate>::generate(__rng),
                u4: <(bool, u8, u32, u64) as Generate>::generate(__rng),
                #[cfg(not(feature = "no-float"))]
                u5: <(bool, u8, u32, u64, f32) as Generate>::generate(__rng),
                #[cfg(not(feature = "no-float"))]
                u6: <(bool, u8, u32, u64, f32, f64) as Generate>::generate(__rng),
                i0: <() as Generate>::generate(__rng),
                i1: <(bool,) as Generate>::generate(__rng),
                i2: <(bool, i8) as Generate>::generate(__rng),
                i3: <(bool, i8, i32) as Generate>::generate(__rng),
                i4: <(bool, i8, i32, i64) as Generate>::generate(__rng),
                #[cfg(not(feature = "no-float"))]
                i5: <(bool, i8, i32, i64, f32) as Generate>::generate(__rng),
                #[cfg(not(feature = "no-float"))]
                i6: <(bool, i8, i32, i64, f32, f64) as Generate>::generate(__rng),
            }
        }
    }
    impl PartialEq<Tuples> for &Tuples {
        #[inline]
        fn eq(&self, other: &Tuples) -> bool {
            *other == **self
        }
    }
    #[musli(mode = Packed)]
    pub enum MediumEnum {
        #[cfg(not(feature = "no-empty"))]
        Empty,
        EmptyTuple(),
        #[musli(transparent)]
        #[cfg(not(feature = "no-newtype"))]
        NewType(u64),
        Tuple(u64, u64),
        #[musli(transparent)]
        #[cfg(not(feature = "no-newtype"))]
        NewTypeString(String),
        TupleString(String, Vec<u8>),
        Struct { a: u32, primitives: Primitives, b: u64 },
        EmptyStruct {},
    }
    const _: () = {
        #[automatically_derived]
        impl ::musli::en::Encode<Packed> for MediumEnum {
            #[inline]
            fn encode<T0>(
                &self,
                _1: &T0::Cx,
                _0: T0,
            ) -> ::musli::__priv::Result<
                <T0 as ::musli::en::Encoder>::Ok,
                <T0 as ::musli::en::Encoder>::Error,
            >
            where
                T0: ::musli::en::Encoder<Mode = Packed>,
            {
                ::musli::__priv::Ok(
                    match self {
                        Self::Empty {} => {
                            ::musli::Context::enter_variant(_1, "Empty", &0usize);
                            let _9 = {
                                ::musli::en::Encoder::encode_variant_fn(
                                    _0,
                                    move |_7| {
                                        let _8 = ::musli::en::VariantEncoder::encode_tag(_7)?;
                                        ::musli::en::Encode::<Packed>::encode(&0usize, _1, _8)?;
                                        let _0 = ::musli::en::VariantEncoder::encode_value(_7)?;
                                        {
                                            ::musli::en::Encoder::encode_struct_fn(
                                                _0,
                                                0usize,
                                                move |_0| { ::musli::__priv::Ok(()) },
                                            )?
                                        };
                                        ::musli::__priv::Ok(())
                                    },
                                )?
                            };
                            ::musli::Context::leave_variant(_1);
                            _9
                        }
                        Self::EmptyTuple {} => {
                            ::musli::Context::enter_variant(_1, "EmptyTuple", &1usize);
                            let _9 = {
                                ::musli::en::Encoder::encode_variant_fn(
                                    _0,
                                    move |_7| {
                                        let _8 = ::musli::en::VariantEncoder::encode_tag(_7)?;
                                        ::musli::en::Encode::<Packed>::encode(&1usize, _1, _8)?;
                                        let _0 = ::musli::en::VariantEncoder::encode_value(_7)?;
                                        {
                                            ::musli::en::Encoder::encode_struct_fn(
                                                _0,
                                                0usize,
                                                move |_0| { ::musli::__priv::Ok(()) },
                                            )?
                                        };
                                        ::musli::__priv::Ok(())
                                    },
                                )?
                            };
                            ::musli::Context::leave_variant(_1);
                            _9
                        }
                        Self::NewType { 0: v0 } => {
                            ::musli::Context::enter_variant(_1, "NewType", &2usize);
                            let _9 = {
                                ::musli::en::Encoder::encode_variant_fn(
                                    _0,
                                    move |_7| {
                                        let _8 = ::musli::en::VariantEncoder::encode_tag(_7)?;
                                        ::musli::en::Encode::<Packed>::encode(&2usize, _1, _8)?;
                                        let _0 = ::musli::en::VariantEncoder::encode_value(_7)?;
                                        ::musli::en::Encode::<Packed>::encode(v0, _1, _0)?;
                                        ::musli::__priv::Ok(())
                                    },
                                )?
                            };
                            ::musli::Context::leave_variant(_1);
                            _9
                        }
                        Self::Tuple { 0: v0, 1: v1 } => {
                            ::musli::Context::enter_variant(_1, "Tuple", &3usize);
                            let _9 = {
                                ::musli::en::Encoder::encode_variant_fn(
                                    _0,
                                    move |_7| {
                                        let _8 = ::musli::en::VariantEncoder::encode_tag(_7)?;
                                        ::musli::en::Encode::<Packed>::encode(&3usize, _1, _8)?;
                                        let _0 = ::musli::en::VariantEncoder::encode_value(_7)?;
                                        {
                                            ::musli::en::Encoder::encode_struct_fn(
                                                _0,
                                                2usize,
                                                move |_0| {
                                                    ::musli::Context::enter_unnamed_field(_1, 0u32, &0usize);
                                                    ::musli::en::StructEncoder::encode_struct_field_fn(
                                                        _0,
                                                        move |_4| {
                                                            let _5 = ::musli::en::StructFieldEncoder::encode_field_name(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<Packed>::encode(&0usize, _1, _5)?;
                                                            let _6 = ::musli::en::StructFieldEncoder::encode_field_value(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<Packed>::encode(v0, _1, _6)?;
                                                            ::musli::__priv::Ok(())
                                                        },
                                                    )?;
                                                    ::musli::Context::leave_field(_1);
                                                    ::musli::Context::enter_unnamed_field(_1, 1u32, &1usize);
                                                    ::musli::en::StructEncoder::encode_struct_field_fn(
                                                        _0,
                                                        move |_4| {
                                                            let _5 = ::musli::en::StructFieldEncoder::encode_field_name(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<Packed>::encode(&1usize, _1, _5)?;
                                                            let _6 = ::musli::en::StructFieldEncoder::encode_field_value(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<Packed>::encode(v1, _1, _6)?;
                                                            ::musli::__priv::Ok(())
                                                        },
                                                    )?;
                                                    ::musli::Context::leave_field(_1);
                                                    ::musli::__priv::Ok(())
                                                },
                                            )?
                                        };
                                        ::musli::__priv::Ok(())
                                    },
                                )?
                            };
                            ::musli::Context::leave_variant(_1);
                            _9
                        }
                        Self::NewTypeString { 0: v0 } => {
                            ::musli::Context::enter_variant(
                                _1,
                                "NewTypeString",
                                &4usize,
                            );
                            let _9 = {
                                ::musli::en::Encoder::encode_variant_fn(
                                    _0,
                                    move |_7| {
                                        let _8 = ::musli::en::VariantEncoder::encode_tag(_7)?;
                                        ::musli::en::Encode::<Packed>::encode(&4usize, _1, _8)?;
                                        let _0 = ::musli::en::VariantEncoder::encode_value(_7)?;
                                        ::musli::en::Encode::<Packed>::encode(v0, _1, _0)?;
                                        ::musli::__priv::Ok(())
                                    },
                                )?
                            };
                            ::musli::Context::leave_variant(_1);
                            _9
                        }
                        Self::TupleString { 0: v0, 1: v1 } => {
                            ::musli::Context::enter_variant(_1, "TupleString", &5usize);
                            let _9 = {
                                ::musli::en::Encoder::encode_variant_fn(
                                    _0,
                                    move |_7| {
                                        let _8 = ::musli::en::VariantEncoder::encode_tag(_7)?;
                                        ::musli::en::Encode::<Packed>::encode(&5usize, _1, _8)?;
                                        let _0 = ::musli::en::VariantEncoder::encode_value(_7)?;
                                        {
                                            ::musli::en::Encoder::encode_struct_fn(
                                                _0,
                                                2usize,
                                                move |_0| {
                                                    ::musli::Context::enter_unnamed_field(_1, 0u32, &0usize);
                                                    ::musli::en::StructEncoder::encode_struct_field_fn(
                                                        _0,
                                                        move |_4| {
                                                            let _5 = ::musli::en::StructFieldEncoder::encode_field_name(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<Packed>::encode(&0usize, _1, _5)?;
                                                            let _6 = ::musli::en::StructFieldEncoder::encode_field_value(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<Packed>::encode(v0, _1, _6)?;
                                                            ::musli::__priv::Ok(())
                                                        },
                                                    )?;
                                                    ::musli::Context::leave_field(_1);
                                                    ::musli::Context::enter_unnamed_field(_1, 1u32, &1usize);
                                                    ::musli::en::StructEncoder::encode_struct_field_fn(
                                                        _0,
                                                        move |_4| {
                                                            let _5 = ::musli::en::StructFieldEncoder::encode_field_name(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<Packed>::encode(&1usize, _1, _5)?;
                                                            let _6 = ::musli::en::StructFieldEncoder::encode_field_value(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<Packed>::encode(v1, _1, _6)?;
                                                            ::musli::__priv::Ok(())
                                                        },
                                                    )?;
                                                    ::musli::Context::leave_field(_1);
                                                    ::musli::__priv::Ok(())
                                                },
                                            )?
                                        };
                                        ::musli::__priv::Ok(())
                                    },
                                )?
                            };
                            ::musli::Context::leave_variant(_1);
                            _9
                        }
                        Self::Struct { a, primitives, b } => {
                            ::musli::Context::enter_variant(_1, "Struct", &6usize);
                            let _9 = {
                                ::musli::en::Encoder::encode_variant_fn(
                                    _0,
                                    move |_7| {
                                        let _8 = ::musli::en::VariantEncoder::encode_tag(_7)?;
                                        ::musli::en::Encode::<Packed>::encode(&6usize, _1, _8)?;
                                        let _0 = ::musli::en::VariantEncoder::encode_value(_7)?;
                                        {
                                            ::musli::en::Encoder::encode_struct_fn(
                                                _0,
                                                3usize,
                                                move |_0| {
                                                    ::musli::Context::enter_named_field(_1, "a", &0usize);
                                                    ::musli::en::StructEncoder::encode_struct_field_fn(
                                                        _0,
                                                        move |_4| {
                                                            let _5 = ::musli::en::StructFieldEncoder::encode_field_name(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<Packed>::encode(&0usize, _1, _5)?;
                                                            let _6 = ::musli::en::StructFieldEncoder::encode_field_value(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<Packed>::encode(a, _1, _6)?;
                                                            ::musli::__priv::Ok(())
                                                        },
                                                    )?;
                                                    ::musli::Context::leave_field(_1);
                                                    ::musli::Context::enter_named_field(
                                                        _1,
                                                        "primitives",
                                                        &1usize,
                                                    );
                                                    ::musli::en::StructEncoder::encode_struct_field_fn(
                                                        _0,
                                                        move |_4| {
                                                            let _5 = ::musli::en::StructFieldEncoder::encode_field_name(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<Packed>::encode(&1usize, _1, _5)?;
                                                            let _6 = ::musli::en::StructFieldEncoder::encode_field_value(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<Packed>::encode(primitives, _1, _6)?;
                                                            ::musli::__priv::Ok(())
                                                        },
                                                    )?;
                                                    ::musli::Context::leave_field(_1);
                                                    ::musli::Context::enter_named_field(_1, "b", &2usize);
                                                    ::musli::en::StructEncoder::encode_struct_field_fn(
                                                        _0,
                                                        move |_4| {
                                                            let _5 = ::musli::en::StructFieldEncoder::encode_field_name(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<Packed>::encode(&2usize, _1, _5)?;
                                                            let _6 = ::musli::en::StructFieldEncoder::encode_field_value(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<Packed>::encode(b, _1, _6)?;
                                                            ::musli::__priv::Ok(())
                                                        },
                                                    )?;
                                                    ::musli::Context::leave_field(_1);
                                                    ::musli::__priv::Ok(())
                                                },
                                            )?
                                        };
                                        ::musli::__priv::Ok(())
                                    },
                                )?
                            };
                            ::musli::Context::leave_variant(_1);
                            _9
                        }
                        Self::EmptyStruct {} => {
                            ::musli::Context::enter_variant(_1, "EmptyStruct", &7usize);
                            let _9 = {
                                ::musli::en::Encoder::encode_variant_fn(
                                    _0,
                                    move |_7| {
                                        let _8 = ::musli::en::VariantEncoder::encode_tag(_7)?;
                                        ::musli::en::Encode::<Packed>::encode(&7usize, _1, _8)?;
                                        let _0 = ::musli::en::VariantEncoder::encode_value(_7)?;
                                        {
                                            ::musli::en::Encoder::encode_struct_fn(
                                                _0,
                                                0usize,
                                                move |_0| { ::musli::__priv::Ok(()) },
                                            )?
                                        };
                                        ::musli::__priv::Ok(())
                                    },
                                )?
                            };
                            ::musli::Context::leave_variant(_1);
                            _9
                        }
                    },
                )
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        impl ::musli::en::Encode<::musli::mode::DefaultMode> for MediumEnum {
            #[inline]
            fn encode<T0>(
                &self,
                _1: &T0::Cx,
                _0: T0,
            ) -> ::musli::__priv::Result<
                <T0 as ::musli::en::Encoder>::Ok,
                <T0 as ::musli::en::Encoder>::Error,
            >
            where
                T0: ::musli::en::Encoder<Mode = ::musli::mode::DefaultMode>,
            {
                ::musli::__priv::Ok(
                    match self {
                        Self::Empty {} => {
                            ::musli::Context::enter_variant(_1, "Empty", &0usize);
                            let _9 = {
                                ::musli::en::Encoder::encode_variant_fn(
                                    _0,
                                    move |_7| {
                                        let _8 = ::musli::en::VariantEncoder::encode_tag(_7)?;
                                        ::musli::en::Encode::<
                                            ::musli::mode::DefaultMode,
                                        >::encode(&0usize, _1, _8)?;
                                        let _0 = ::musli::en::VariantEncoder::encode_value(_7)?;
                                        {
                                            ::musli::en::Encoder::encode_struct_fn(
                                                _0,
                                                0usize,
                                                move |_0| { ::musli::__priv::Ok(()) },
                                            )?
                                        };
                                        ::musli::__priv::Ok(())
                                    },
                                )?
                            };
                            ::musli::Context::leave_variant(_1);
                            _9
                        }
                        Self::EmptyTuple {} => {
                            ::musli::Context::enter_variant(_1, "EmptyTuple", &1usize);
                            let _9 = {
                                ::musli::en::Encoder::encode_variant_fn(
                                    _0,
                                    move |_7| {
                                        let _8 = ::musli::en::VariantEncoder::encode_tag(_7)?;
                                        ::musli::en::Encode::<
                                            ::musli::mode::DefaultMode,
                                        >::encode(&1usize, _1, _8)?;
                                        let _0 = ::musli::en::VariantEncoder::encode_value(_7)?;
                                        {
                                            ::musli::en::Encoder::encode_struct_fn(
                                                _0,
                                                0usize,
                                                move |_0| { ::musli::__priv::Ok(()) },
                                            )?
                                        };
                                        ::musli::__priv::Ok(())
                                    },
                                )?
                            };
                            ::musli::Context::leave_variant(_1);
                            _9
                        }
                        Self::NewType { 0: v0 } => {
                            ::musli::Context::enter_variant(_1, "NewType", &2usize);
                            let _9 = {
                                ::musli::en::Encoder::encode_variant_fn(
                                    _0,
                                    move |_7| {
                                        let _8 = ::musli::en::VariantEncoder::encode_tag(_7)?;
                                        ::musli::en::Encode::<
                                            ::musli::mode::DefaultMode,
                                        >::encode(&2usize, _1, _8)?;
                                        let _0 = ::musli::en::VariantEncoder::encode_value(_7)?;
                                        ::musli::en::Encode::<
                                            ::musli::mode::DefaultMode,
                                        >::encode(v0, _1, _0)?;
                                        ::musli::__priv::Ok(())
                                    },
                                )?
                            };
                            ::musli::Context::leave_variant(_1);
                            _9
                        }
                        Self::Tuple { 0: v0, 1: v1 } => {
                            ::musli::Context::enter_variant(_1, "Tuple", &3usize);
                            let _9 = {
                                ::musli::en::Encoder::encode_variant_fn(
                                    _0,
                                    move |_7| {
                                        let _8 = ::musli::en::VariantEncoder::encode_tag(_7)?;
                                        ::musli::en::Encode::<
                                            ::musli::mode::DefaultMode,
                                        >::encode(&3usize, _1, _8)?;
                                        let _0 = ::musli::en::VariantEncoder::encode_value(_7)?;
                                        {
                                            ::musli::en::Encoder::encode_struct_fn(
                                                _0,
                                                2usize,
                                                move |_0| {
                                                    ::musli::Context::enter_unnamed_field(_1, 0u32, &0usize);
                                                    ::musli::en::StructEncoder::encode_struct_field_fn(
                                                        _0,
                                                        move |_4| {
                                                            let _5 = ::musli::en::StructFieldEncoder::encode_field_name(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::encode(&0usize, _1, _5)?;
                                                            let _6 = ::musli::en::StructFieldEncoder::encode_field_value(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::encode(v0, _1, _6)?;
                                                            ::musli::__priv::Ok(())
                                                        },
                                                    )?;
                                                    ::musli::Context::leave_field(_1);
                                                    ::musli::Context::enter_unnamed_field(_1, 1u32, &1usize);
                                                    ::musli::en::StructEncoder::encode_struct_field_fn(
                                                        _0,
                                                        move |_4| {
                                                            let _5 = ::musli::en::StructFieldEncoder::encode_field_name(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::encode(&1usize, _1, _5)?;
                                                            let _6 = ::musli::en::StructFieldEncoder::encode_field_value(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::encode(v1, _1, _6)?;
                                                            ::musli::__priv::Ok(())
                                                        },
                                                    )?;
                                                    ::musli::Context::leave_field(_1);
                                                    ::musli::__priv::Ok(())
                                                },
                                            )?
                                        };
                                        ::musli::__priv::Ok(())
                                    },
                                )?
                            };
                            ::musli::Context::leave_variant(_1);
                            _9
                        }
                        Self::NewTypeString { 0: v0 } => {
                            ::musli::Context::enter_variant(
                                _1,
                                "NewTypeString",
                                &4usize,
                            );
                            let _9 = {
                                ::musli::en::Encoder::encode_variant_fn(
                                    _0,
                                    move |_7| {
                                        let _8 = ::musli::en::VariantEncoder::encode_tag(_7)?;
                                        ::musli::en::Encode::<
                                            ::musli::mode::DefaultMode,
                                        >::encode(&4usize, _1, _8)?;
                                        let _0 = ::musli::en::VariantEncoder::encode_value(_7)?;
                                        ::musli::en::Encode::<
                                            ::musli::mode::DefaultMode,
                                        >::encode(v0, _1, _0)?;
                                        ::musli::__priv::Ok(())
                                    },
                                )?
                            };
                            ::musli::Context::leave_variant(_1);
                            _9
                        }
                        Self::TupleString { 0: v0, 1: v1 } => {
                            ::musli::Context::enter_variant(_1, "TupleString", &5usize);
                            let _9 = {
                                ::musli::en::Encoder::encode_variant_fn(
                                    _0,
                                    move |_7| {
                                        let _8 = ::musli::en::VariantEncoder::encode_tag(_7)?;
                                        ::musli::en::Encode::<
                                            ::musli::mode::DefaultMode,
                                        >::encode(&5usize, _1, _8)?;
                                        let _0 = ::musli::en::VariantEncoder::encode_value(_7)?;
                                        {
                                            ::musli::en::Encoder::encode_struct_fn(
                                                _0,
                                                2usize,
                                                move |_0| {
                                                    ::musli::Context::enter_unnamed_field(_1, 0u32, &0usize);
                                                    ::musli::en::StructEncoder::encode_struct_field_fn(
                                                        _0,
                                                        move |_4| {
                                                            let _5 = ::musli::en::StructFieldEncoder::encode_field_name(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::encode(&0usize, _1, _5)?;
                                                            let _6 = ::musli::en::StructFieldEncoder::encode_field_value(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::encode(v0, _1, _6)?;
                                                            ::musli::__priv::Ok(())
                                                        },
                                                    )?;
                                                    ::musli::Context::leave_field(_1);
                                                    ::musli::Context::enter_unnamed_field(_1, 1u32, &1usize);
                                                    ::musli::en::StructEncoder::encode_struct_field_fn(
                                                        _0,
                                                        move |_4| {
                                                            let _5 = ::musli::en::StructFieldEncoder::encode_field_name(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::encode(&1usize, _1, _5)?;
                                                            let _6 = ::musli::en::StructFieldEncoder::encode_field_value(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::encode(v1, _1, _6)?;
                                                            ::musli::__priv::Ok(())
                                                        },
                                                    )?;
                                                    ::musli::Context::leave_field(_1);
                                                    ::musli::__priv::Ok(())
                                                },
                                            )?
                                        };
                                        ::musli::__priv::Ok(())
                                    },
                                )?
                            };
                            ::musli::Context::leave_variant(_1);
                            _9
                        }
                        Self::Struct { a, primitives, b } => {
                            ::musli::Context::enter_variant(_1, "Struct", &6usize);
                            let _9 = {
                                ::musli::en::Encoder::encode_variant_fn(
                                    _0,
                                    move |_7| {
                                        let _8 = ::musli::en::VariantEncoder::encode_tag(_7)?;
                                        ::musli::en::Encode::<
                                            ::musli::mode::DefaultMode,
                                        >::encode(&6usize, _1, _8)?;
                                        let _0 = ::musli::en::VariantEncoder::encode_value(_7)?;
                                        {
                                            ::musli::en::Encoder::encode_struct_fn(
                                                _0,
                                                3usize,
                                                move |_0| {
                                                    ::musli::Context::enter_named_field(_1, "a", &0usize);
                                                    ::musli::en::StructEncoder::encode_struct_field_fn(
                                                        _0,
                                                        move |_4| {
                                                            let _5 = ::musli::en::StructFieldEncoder::encode_field_name(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::encode(&0usize, _1, _5)?;
                                                            let _6 = ::musli::en::StructFieldEncoder::encode_field_value(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::encode(a, _1, _6)?;
                                                            ::musli::__priv::Ok(())
                                                        },
                                                    )?;
                                                    ::musli::Context::leave_field(_1);
                                                    ::musli::Context::enter_named_field(
                                                        _1,
                                                        "primitives",
                                                        &1usize,
                                                    );
                                                    ::musli::en::StructEncoder::encode_struct_field_fn(
                                                        _0,
                                                        move |_4| {
                                                            let _5 = ::musli::en::StructFieldEncoder::encode_field_name(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::encode(&1usize, _1, _5)?;
                                                            let _6 = ::musli::en::StructFieldEncoder::encode_field_value(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::encode(primitives, _1, _6)?;
                                                            ::musli::__priv::Ok(())
                                                        },
                                                    )?;
                                                    ::musli::Context::leave_field(_1);
                                                    ::musli::Context::enter_named_field(_1, "b", &2usize);
                                                    ::musli::en::StructEncoder::encode_struct_field_fn(
                                                        _0,
                                                        move |_4| {
                                                            let _5 = ::musli::en::StructFieldEncoder::encode_field_name(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::encode(&2usize, _1, _5)?;
                                                            let _6 = ::musli::en::StructFieldEncoder::encode_field_value(
                                                                _4,
                                                            )?;
                                                            ::musli::en::Encode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::encode(b, _1, _6)?;
                                                            ::musli::__priv::Ok(())
                                                        },
                                                    )?;
                                                    ::musli::Context::leave_field(_1);
                                                    ::musli::__priv::Ok(())
                                                },
                                            )?
                                        };
                                        ::musli::__priv::Ok(())
                                    },
                                )?
                            };
                            ::musli::Context::leave_variant(_1);
                            _9
                        }
                        Self::EmptyStruct {} => {
                            ::musli::Context::enter_variant(_1, "EmptyStruct", &7usize);
                            let _9 = {
                                ::musli::en::Encoder::encode_variant_fn(
                                    _0,
                                    move |_7| {
                                        let _8 = ::musli::en::VariantEncoder::encode_tag(_7)?;
                                        ::musli::en::Encode::<
                                            ::musli::mode::DefaultMode,
                                        >::encode(&7usize, _1, _8)?;
                                        let _0 = ::musli::en::VariantEncoder::encode_value(_7)?;
                                        {
                                            ::musli::en::Encoder::encode_struct_fn(
                                                _0,
                                                0usize,
                                                move |_0| { ::musli::__priv::Ok(()) },
                                            )?
                                        };
                                        ::musli::__priv::Ok(())
                                    },
                                )?
                            };
                            ::musli::Context::leave_variant(_1);
                            _9
                        }
                    },
                )
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        #[allow(clippy::init_numbered_fields)]
        #[allow(clippy::let_unit_value)]
        impl<'de> ::musli::de::Decode<'de, Packed> for MediumEnum {
            #[inline]
            fn decode<T0>(
                _0: &T0::Cx,
                _1: T0,
            ) -> ::musli::__priv::Result<Self, <T0::Cx as ::musli::Context>::Error>
            where
                T0: ::musli::de::Decoder<'de, Mode = Packed>,
            {
                {
                    ::musli::Context::enter_enum(_0, "MediumEnum");
                    let _7 = ::musli::de::Decoder::decode_variant(
                        _1,
                        move |_9| {
                            let _10 = {
                                let mut _9 = ::musli::de::VariantDecoder::decode_tag(_9)?;
                                ::musli::de::Decode::<Packed>::decode(_0, _9)?
                            };
                            let _7 = match _10 {
                                0usize => {
                                    ::musli::Context::enter_variant(_0, "Empty", &0usize);
                                    let _7 = {
                                        let _3 = ::musli::de::VariantDecoder::decode_value(_9)?;
                                        {
                                            ::musli::de::Decoder::decode_struct(
                                                _3,
                                                Some(0usize),
                                                move |_16| {
                                                    while let ::musli::__priv::Some(mut _8) = ::musli::de::StructDecoder::decode_field(
                                                        _16,
                                                    )? {
                                                        let _2 = {
                                                            let _8 = ::musli::de::StructFieldDecoder::decode_field_name(
                                                                &mut _8,
                                                            )?;
                                                            ::musli::de::Decode::<Packed>::decode(_0, _8)?
                                                        };
                                                        if ::musli::__priv::skip_field(_8)? {
                                                            return ::musli::__priv::Err(
                                                                ::musli::Context::invalid_variant_field_tag(
                                                                    _0,
                                                                    "Empty",
                                                                    &_10,
                                                                    &_2,
                                                                ),
                                                            );
                                                        }
                                                    }
                                                    ::musli::__priv::Ok(Self::Empty {})
                                                },
                                            )?
                                        }
                                    };
                                    ::musli::Context::leave_variant(_0);
                                    _7
                                }
                                1usize => {
                                    ::musli::Context::enter_variant(_0, "EmptyTuple", &1usize);
                                    let _7 = {
                                        let _3 = ::musli::de::VariantDecoder::decode_value(_9)?;
                                        {
                                            ::musli::de::Decoder::decode_struct(
                                                _3,
                                                Some(0usize),
                                                move |_16| {
                                                    while let ::musli::__priv::Some(mut _8) = ::musli::de::StructDecoder::decode_field(
                                                        _16,
                                                    )? {
                                                        let _2 = {
                                                            let _8 = ::musli::de::StructFieldDecoder::decode_field_name(
                                                                &mut _8,
                                                            )?;
                                                            ::musli::de::Decode::<Packed>::decode(_0, _8)?
                                                        };
                                                        if ::musli::__priv::skip_field(_8)? {
                                                            return ::musli::__priv::Err(
                                                                ::musli::Context::invalid_variant_field_tag(
                                                                    _0,
                                                                    "EmptyTuple",
                                                                    &_10,
                                                                    &_2,
                                                                ),
                                                            );
                                                        }
                                                    }
                                                    ::musli::__priv::Ok(Self::EmptyTuple {})
                                                },
                                            )?
                                        }
                                    };
                                    ::musli::Context::leave_variant(_0);
                                    _7
                                }
                                2usize => {
                                    ::musli::Context::enter_variant(_0, "NewType", &2usize);
                                    let _7 = {
                                        let _3 = ::musli::de::VariantDecoder::decode_value(_9)?;
                                        let _7 = Self::NewType {
                                            0: ::musli::de::Decode::<Packed>::decode(_0, _3)?,
                                        };
                                        _7
                                    };
                                    ::musli::Context::leave_variant(_0);
                                    _7
                                }
                                3usize => {
                                    ::musli::Context::enter_variant(_0, "Tuple", &3usize);
                                    let _7 = {
                                        let _3 = ::musli::de::VariantDecoder::decode_value(_9)?;
                                        {
                                            let mut _0_f: ::musli::__priv::Option<u64> = ::musli::__priv::None;
                                            let mut _1_f: ::musli::__priv::Option<u64> = ::musli::__priv::None;
                                            ::musli::de::Decoder::decode_struct(
                                                _3,
                                                Some(2usize),
                                                move |_16| {
                                                    while let ::musli::__priv::Some(mut _8) = ::musli::de::StructDecoder::decode_field(
                                                        _16,
                                                    )? {
                                                        let _2 = {
                                                            let _8 = ::musli::de::StructFieldDecoder::decode_field_name(
                                                                &mut _8,
                                                            )?;
                                                            ::musli::de::Decode::<Packed>::decode(_0, _8)?
                                                        };
                                                        match _2 {
                                                            0usize => {
                                                                ::musli::Context::enter_unnamed_field(_0, 0u32, &0usize);
                                                                let _8 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                                    _8,
                                                                )?;
                                                                _0_f = ::musli::__priv::Some(
                                                                    ::musli::de::Decode::<Packed>::decode(_0, _8)?,
                                                                );
                                                                ::musli::Context::leave_field(_0);
                                                            }
                                                            1usize => {
                                                                ::musli::Context::enter_unnamed_field(_0, 1u32, &1usize);
                                                                let _8 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                                    _8,
                                                                )?;
                                                                _1_f = ::musli::__priv::Some(
                                                                    ::musli::de::Decode::<Packed>::decode(_0, _8)?,
                                                                );
                                                                ::musli::Context::leave_field(_0);
                                                            }
                                                            _2 => {
                                                                if ::musli::__priv::skip_field(_8)? {
                                                                    return ::musli::__priv::Err(
                                                                        ::musli::Context::invalid_variant_field_tag(
                                                                            _0,
                                                                            "Tuple",
                                                                            &_10,
                                                                            &_2,
                                                                        ),
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }
                                                    ::musli::__priv::Ok(Self::Tuple {
                                                        0: match _0_f {
                                                            ::musli::__priv::Some(_0_f) => _0_f,
                                                            ::musli::__priv::None => {
                                                                return ::musli::__priv::Err(
                                                                    ::musli::Context::expected_tag(_0, "Tuple", &0usize),
                                                                );
                                                            }
                                                        },
                                                        1: match _1_f {
                                                            ::musli::__priv::Some(_1_f) => _1_f,
                                                            ::musli::__priv::None => {
                                                                return ::musli::__priv::Err(
                                                                    ::musli::Context::expected_tag(_0, "Tuple", &1usize),
                                                                );
                                                            }
                                                        },
                                                    })
                                                },
                                            )?
                                        }
                                    };
                                    ::musli::Context::leave_variant(_0);
                                    _7
                                }
                                4usize => {
                                    ::musli::Context::enter_variant(
                                        _0,
                                        "NewTypeString",
                                        &4usize,
                                    );
                                    let _7 = {
                                        let _3 = ::musli::de::VariantDecoder::decode_value(_9)?;
                                        let _7 = Self::NewTypeString {
                                            0: ::musli::de::Decode::<Packed>::decode(_0, _3)?,
                                        };
                                        _7
                                    };
                                    ::musli::Context::leave_variant(_0);
                                    _7
                                }
                                5usize => {
                                    ::musli::Context::enter_variant(_0, "TupleString", &5usize);
                                    let _7 = {
                                        let _3 = ::musli::de::VariantDecoder::decode_value(_9)?;
                                        {
                                            let mut _0_f: ::musli::__priv::Option<String> = ::musli::__priv::None;
                                            let mut _1_f: ::musli::__priv::Option<Vec<u8>> = ::musli::__priv::None;
                                            ::musli::de::Decoder::decode_struct(
                                                _3,
                                                Some(2usize),
                                                move |_16| {
                                                    while let ::musli::__priv::Some(mut _8) = ::musli::de::StructDecoder::decode_field(
                                                        _16,
                                                    )? {
                                                        let _2 = {
                                                            let _8 = ::musli::de::StructFieldDecoder::decode_field_name(
                                                                &mut _8,
                                                            )?;
                                                            ::musli::de::Decode::<Packed>::decode(_0, _8)?
                                                        };
                                                        match _2 {
                                                            0usize => {
                                                                ::musli::Context::enter_unnamed_field(_0, 0u32, &0usize);
                                                                let _8 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                                    _8,
                                                                )?;
                                                                _0_f = ::musli::__priv::Some(
                                                                    ::musli::de::Decode::<Packed>::decode(_0, _8)?,
                                                                );
                                                                ::musli::Context::leave_field(_0);
                                                            }
                                                            1usize => {
                                                                ::musli::Context::enter_unnamed_field(_0, 1u32, &1usize);
                                                                let _8 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                                    _8,
                                                                )?;
                                                                _1_f = ::musli::__priv::Some(
                                                                    ::musli::de::Decode::<Packed>::decode(_0, _8)?,
                                                                );
                                                                ::musli::Context::leave_field(_0);
                                                            }
                                                            _2 => {
                                                                if ::musli::__priv::skip_field(_8)? {
                                                                    return ::musli::__priv::Err(
                                                                        ::musli::Context::invalid_variant_field_tag(
                                                                            _0,
                                                                            "TupleString",
                                                                            &_10,
                                                                            &_2,
                                                                        ),
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }
                                                    ::musli::__priv::Ok(Self::TupleString {
                                                        0: match _0_f {
                                                            ::musli::__priv::Some(_0_f) => _0_f,
                                                            ::musli::__priv::None => {
                                                                return ::musli::__priv::Err(
                                                                    ::musli::Context::expected_tag(_0, "TupleString", &0usize),
                                                                );
                                                            }
                                                        },
                                                        1: match _1_f {
                                                            ::musli::__priv::Some(_1_f) => _1_f,
                                                            ::musli::__priv::None => {
                                                                return ::musli::__priv::Err(
                                                                    ::musli::Context::expected_tag(_0, "TupleString", &1usize),
                                                                );
                                                            }
                                                        },
                                                    })
                                                },
                                            )?
                                        }
                                    };
                                    ::musli::Context::leave_variant(_0);
                                    _7
                                }
                                6usize => {
                                    ::musli::Context::enter_variant(_0, "Struct", &6usize);
                                    let _7 = {
                                        let _3 = ::musli::de::VariantDecoder::decode_value(_9)?;
                                        {
                                            let mut _2_f: ::musli::__priv::Option<u32> = ::musli::__priv::None;
                                            let mut _3_f: ::musli::__priv::Option<Primitives> = ::musli::__priv::None;
                                            let mut _4_f: ::musli::__priv::Option<u64> = ::musli::__priv::None;
                                            ::musli::de::Decoder::decode_struct(
                                                _3,
                                                Some(3usize),
                                                move |_16| {
                                                    while let ::musli::__priv::Some(mut _8) = ::musli::de::StructDecoder::decode_field(
                                                        _16,
                                                    )? {
                                                        let _2 = {
                                                            let _8 = ::musli::de::StructFieldDecoder::decode_field_name(
                                                                &mut _8,
                                                            )?;
                                                            ::musli::de::Decode::<Packed>::decode(_0, _8)?
                                                        };
                                                        match _2 {
                                                            0usize => {
                                                                ::musli::Context::enter_named_field(_0, "a", &0usize);
                                                                let _8 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                                    _8,
                                                                )?;
                                                                _2_f = ::musli::__priv::Some(
                                                                    ::musli::de::Decode::<Packed>::decode(_0, _8)?,
                                                                );
                                                                ::musli::Context::leave_field(_0);
                                                            }
                                                            1usize => {
                                                                ::musli::Context::enter_named_field(
                                                                    _0,
                                                                    "primitives",
                                                                    &1usize,
                                                                );
                                                                let _8 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                                    _8,
                                                                )?;
                                                                _3_f = ::musli::__priv::Some(
                                                                    ::musli::de::Decode::<Packed>::decode(_0, _8)?,
                                                                );
                                                                ::musli::Context::leave_field(_0);
                                                            }
                                                            2usize => {
                                                                ::musli::Context::enter_named_field(_0, "b", &2usize);
                                                                let _8 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                                    _8,
                                                                )?;
                                                                _4_f = ::musli::__priv::Some(
                                                                    ::musli::de::Decode::<Packed>::decode(_0, _8)?,
                                                                );
                                                                ::musli::Context::leave_field(_0);
                                                            }
                                                            _2 => {
                                                                if ::musli::__priv::skip_field(_8)? {
                                                                    return ::musli::__priv::Err(
                                                                        ::musli::Context::invalid_variant_field_tag(
                                                                            _0,
                                                                            "Struct",
                                                                            &_10,
                                                                            &_2,
                                                                        ),
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }
                                                    ::musli::__priv::Ok(Self::Struct {
                                                        a: match _2_f {
                                                            ::musli::__priv::Some(_2_f) => _2_f,
                                                            ::musli::__priv::None => {
                                                                return ::musli::__priv::Err(
                                                                    ::musli::Context::expected_tag(_0, "Struct", &0usize),
                                                                );
                                                            }
                                                        },
                                                        primitives: match _3_f {
                                                            ::musli::__priv::Some(_3_f) => _3_f,
                                                            ::musli::__priv::None => {
                                                                return ::musli::__priv::Err(
                                                                    ::musli::Context::expected_tag(_0, "Struct", &1usize),
                                                                );
                                                            }
                                                        },
                                                        b: match _4_f {
                                                            ::musli::__priv::Some(_4_f) => _4_f,
                                                            ::musli::__priv::None => {
                                                                return ::musli::__priv::Err(
                                                                    ::musli::Context::expected_tag(_0, "Struct", &2usize),
                                                                );
                                                            }
                                                        },
                                                    })
                                                },
                                            )?
                                        }
                                    };
                                    ::musli::Context::leave_variant(_0);
                                    _7
                                }
                                7usize => {
                                    ::musli::Context::enter_variant(_0, "EmptyStruct", &7usize);
                                    let _7 = {
                                        let _3 = ::musli::de::VariantDecoder::decode_value(_9)?;
                                        {
                                            ::musli::de::Decoder::decode_struct(
                                                _3,
                                                Some(0usize),
                                                move |_16| {
                                                    while let ::musli::__priv::Some(mut _8) = ::musli::de::StructDecoder::decode_field(
                                                        _16,
                                                    )? {
                                                        let _2 = {
                                                            let _8 = ::musli::de::StructFieldDecoder::decode_field_name(
                                                                &mut _8,
                                                            )?;
                                                            ::musli::de::Decode::<Packed>::decode(_0, _8)?
                                                        };
                                                        if ::musli::__priv::skip_field(_8)? {
                                                            return ::musli::__priv::Err(
                                                                ::musli::Context::invalid_variant_field_tag(
                                                                    _0,
                                                                    "EmptyStruct",
                                                                    &_10,
                                                                    &_2,
                                                                ),
                                                            );
                                                        }
                                                    }
                                                    ::musli::__priv::Ok(Self::EmptyStruct {})
                                                },
                                            )?
                                        }
                                    };
                                    ::musli::Context::leave_variant(_0);
                                    _7
                                }
                                _ => {
                                    return ::musli::__priv::Err(
                                        ::musli::Context::invalid_variant_tag(
                                            _0,
                                            "MediumEnum",
                                            &_10,
                                        ),
                                    );
                                }
                            };
                            ::musli::__priv::Ok(_7)
                        },
                    )?;
                    ::musli::Context::leave_enum(_0);
                    Ok(_7)
                }
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        #[allow(clippy::init_numbered_fields)]
        #[allow(clippy::let_unit_value)]
        impl<'de> ::musli::de::Decode<'de, ::musli::mode::DefaultMode> for MediumEnum {
            #[inline]
            fn decode<T0>(
                _0: &T0::Cx,
                _1: T0,
            ) -> ::musli::__priv::Result<Self, <T0::Cx as ::musli::Context>::Error>
            where
                T0: ::musli::de::Decoder<'de, Mode = ::musli::mode::DefaultMode>,
            {
                {
                    ::musli::Context::enter_enum(_0, "MediumEnum");
                    let _7 = ::musli::de::Decoder::decode_variant(
                        _1,
                        move |_9| {
                            let _10 = {
                                let mut _9 = ::musli::de::VariantDecoder::decode_tag(_9)?;
                                ::musli::de::Decode::<
                                    ::musli::mode::DefaultMode,
                                >::decode(_0, _9)?
                            };
                            let _7 = match _10 {
                                0usize => {
                                    ::musli::Context::enter_variant(_0, "Empty", &0usize);
                                    let _7 = {
                                        let _3 = ::musli::de::VariantDecoder::decode_value(_9)?;
                                        {
                                            ::musli::de::Decoder::decode_struct(
                                                _3,
                                                Some(0usize),
                                                move |_16| {
                                                    while let ::musli::__priv::Some(mut _8) = ::musli::de::StructDecoder::decode_field(
                                                        _16,
                                                    )? {
                                                        let _2 = {
                                                            let _8 = ::musli::de::StructFieldDecoder::decode_field_name(
                                                                &mut _8,
                                                            )?;
                                                            ::musli::de::Decode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::decode(_0, _8)?
                                                        };
                                                        if ::musli::__priv::skip_field(_8)? {
                                                            return ::musli::__priv::Err(
                                                                ::musli::Context::invalid_variant_field_tag(
                                                                    _0,
                                                                    "Empty",
                                                                    &_10,
                                                                    &_2,
                                                                ),
                                                            );
                                                        }
                                                    }
                                                    ::musli::__priv::Ok(Self::Empty {})
                                                },
                                            )?
                                        }
                                    };
                                    ::musli::Context::leave_variant(_0);
                                    _7
                                }
                                1usize => {
                                    ::musli::Context::enter_variant(_0, "EmptyTuple", &1usize);
                                    let _7 = {
                                        let _3 = ::musli::de::VariantDecoder::decode_value(_9)?;
                                        {
                                            ::musli::de::Decoder::decode_struct(
                                                _3,
                                                Some(0usize),
                                                move |_16| {
                                                    while let ::musli::__priv::Some(mut _8) = ::musli::de::StructDecoder::decode_field(
                                                        _16,
                                                    )? {
                                                        let _2 = {
                                                            let _8 = ::musli::de::StructFieldDecoder::decode_field_name(
                                                                &mut _8,
                                                            )?;
                                                            ::musli::de::Decode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::decode(_0, _8)?
                                                        };
                                                        if ::musli::__priv::skip_field(_8)? {
                                                            return ::musli::__priv::Err(
                                                                ::musli::Context::invalid_variant_field_tag(
                                                                    _0,
                                                                    "EmptyTuple",
                                                                    &_10,
                                                                    &_2,
                                                                ),
                                                            );
                                                        }
                                                    }
                                                    ::musli::__priv::Ok(Self::EmptyTuple {})
                                                },
                                            )?
                                        }
                                    };
                                    ::musli::Context::leave_variant(_0);
                                    _7
                                }
                                2usize => {
                                    ::musli::Context::enter_variant(_0, "NewType", &2usize);
                                    let _7 = {
                                        let _3 = ::musli::de::VariantDecoder::decode_value(_9)?;
                                        let _7 = Self::NewType {
                                            0: ::musli::de::Decode::<
                                                ::musli::mode::DefaultMode,
                                            >::decode(_0, _3)?,
                                        };
                                        _7
                                    };
                                    ::musli::Context::leave_variant(_0);
                                    _7
                                }
                                3usize => {
                                    ::musli::Context::enter_variant(_0, "Tuple", &3usize);
                                    let _7 = {
                                        let _3 = ::musli::de::VariantDecoder::decode_value(_9)?;
                                        {
                                            let mut _0_f: ::musli::__priv::Option<u64> = ::musli::__priv::None;
                                            let mut _1_f: ::musli::__priv::Option<u64> = ::musli::__priv::None;
                                            ::musli::de::Decoder::decode_struct(
                                                _3,
                                                Some(2usize),
                                                move |_16| {
                                                    while let ::musli::__priv::Some(mut _8) = ::musli::de::StructDecoder::decode_field(
                                                        _16,
                                                    )? {
                                                        let _2 = {
                                                            let _8 = ::musli::de::StructFieldDecoder::decode_field_name(
                                                                &mut _8,
                                                            )?;
                                                            ::musli::de::Decode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::decode(_0, _8)?
                                                        };
                                                        match _2 {
                                                            0usize => {
                                                                ::musli::Context::enter_unnamed_field(_0, 0u32, &0usize);
                                                                let _8 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                                    _8,
                                                                )?;
                                                                _0_f = ::musli::__priv::Some(
                                                                    ::musli::de::Decode::<
                                                                        ::musli::mode::DefaultMode,
                                                                    >::decode(_0, _8)?,
                                                                );
                                                                ::musli::Context::leave_field(_0);
                                                            }
                                                            1usize => {
                                                                ::musli::Context::enter_unnamed_field(_0, 1u32, &1usize);
                                                                let _8 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                                    _8,
                                                                )?;
                                                                _1_f = ::musli::__priv::Some(
                                                                    ::musli::de::Decode::<
                                                                        ::musli::mode::DefaultMode,
                                                                    >::decode(_0, _8)?,
                                                                );
                                                                ::musli::Context::leave_field(_0);
                                                            }
                                                            _2 => {
                                                                if ::musli::__priv::skip_field(_8)? {
                                                                    return ::musli::__priv::Err(
                                                                        ::musli::Context::invalid_variant_field_tag(
                                                                            _0,
                                                                            "Tuple",
                                                                            &_10,
                                                                            &_2,
                                                                        ),
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }
                                                    ::musli::__priv::Ok(Self::Tuple {
                                                        0: match _0_f {
                                                            ::musli::__priv::Some(_0_f) => _0_f,
                                                            ::musli::__priv::None => {
                                                                return ::musli::__priv::Err(
                                                                    ::musli::Context::expected_tag(_0, "Tuple", &0usize),
                                                                );
                                                            }
                                                        },
                                                        1: match _1_f {
                                                            ::musli::__priv::Some(_1_f) => _1_f,
                                                            ::musli::__priv::None => {
                                                                return ::musli::__priv::Err(
                                                                    ::musli::Context::expected_tag(_0, "Tuple", &1usize),
                                                                );
                                                            }
                                                        },
                                                    })
                                                },
                                            )?
                                        }
                                    };
                                    ::musli::Context::leave_variant(_0);
                                    _7
                                }
                                4usize => {
                                    ::musli::Context::enter_variant(
                                        _0,
                                        "NewTypeString",
                                        &4usize,
                                    );
                                    let _7 = {
                                        let _3 = ::musli::de::VariantDecoder::decode_value(_9)?;
                                        let _7 = Self::NewTypeString {
                                            0: ::musli::de::Decode::<
                                                ::musli::mode::DefaultMode,
                                            >::decode(_0, _3)?,
                                        };
                                        _7
                                    };
                                    ::musli::Context::leave_variant(_0);
                                    _7
                                }
                                5usize => {
                                    ::musli::Context::enter_variant(_0, "TupleString", &5usize);
                                    let _7 = {
                                        let _3 = ::musli::de::VariantDecoder::decode_value(_9)?;
                                        {
                                            let mut _0_f: ::musli::__priv::Option<String> = ::musli::__priv::None;
                                            let mut _1_f: ::musli::__priv::Option<Vec<u8>> = ::musli::__priv::None;
                                            ::musli::de::Decoder::decode_struct(
                                                _3,
                                                Some(2usize),
                                                move |_16| {
                                                    while let ::musli::__priv::Some(mut _8) = ::musli::de::StructDecoder::decode_field(
                                                        _16,
                                                    )? {
                                                        let _2 = {
                                                            let _8 = ::musli::de::StructFieldDecoder::decode_field_name(
                                                                &mut _8,
                                                            )?;
                                                            ::musli::de::Decode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::decode(_0, _8)?
                                                        };
                                                        match _2 {
                                                            0usize => {
                                                                ::musli::Context::enter_unnamed_field(_0, 0u32, &0usize);
                                                                let _8 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                                    _8,
                                                                )?;
                                                                _0_f = ::musli::__priv::Some(
                                                                    ::musli::de::Decode::<
                                                                        ::musli::mode::DefaultMode,
                                                                    >::decode(_0, _8)?,
                                                                );
                                                                ::musli::Context::leave_field(_0);
                                                            }
                                                            1usize => {
                                                                ::musli::Context::enter_unnamed_field(_0, 1u32, &1usize);
                                                                let _8 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                                    _8,
                                                                )?;
                                                                _1_f = ::musli::__priv::Some(
                                                                    ::musli::de::Decode::<
                                                                        ::musli::mode::DefaultMode,
                                                                    >::decode(_0, _8)?,
                                                                );
                                                                ::musli::Context::leave_field(_0);
                                                            }
                                                            _2 => {
                                                                if ::musli::__priv::skip_field(_8)? {
                                                                    return ::musli::__priv::Err(
                                                                        ::musli::Context::invalid_variant_field_tag(
                                                                            _0,
                                                                            "TupleString",
                                                                            &_10,
                                                                            &_2,
                                                                        ),
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }
                                                    ::musli::__priv::Ok(Self::TupleString {
                                                        0: match _0_f {
                                                            ::musli::__priv::Some(_0_f) => _0_f,
                                                            ::musli::__priv::None => {
                                                                return ::musli::__priv::Err(
                                                                    ::musli::Context::expected_tag(_0, "TupleString", &0usize),
                                                                );
                                                            }
                                                        },
                                                        1: match _1_f {
                                                            ::musli::__priv::Some(_1_f) => _1_f,
                                                            ::musli::__priv::None => {
                                                                return ::musli::__priv::Err(
                                                                    ::musli::Context::expected_tag(_0, "TupleString", &1usize),
                                                                );
                                                            }
                                                        },
                                                    })
                                                },
                                            )?
                                        }
                                    };
                                    ::musli::Context::leave_variant(_0);
                                    _7
                                }
                                6usize => {
                                    ::musli::Context::enter_variant(_0, "Struct", &6usize);
                                    let _7 = {
                                        let _3 = ::musli::de::VariantDecoder::decode_value(_9)?;
                                        {
                                            let mut _2_f: ::musli::__priv::Option<u32> = ::musli::__priv::None;
                                            let mut _3_f: ::musli::__priv::Option<Primitives> = ::musli::__priv::None;
                                            let mut _4_f: ::musli::__priv::Option<u64> = ::musli::__priv::None;
                                            ::musli::de::Decoder::decode_struct(
                                                _3,
                                                Some(3usize),
                                                move |_16| {
                                                    while let ::musli::__priv::Some(mut _8) = ::musli::de::StructDecoder::decode_field(
                                                        _16,
                                                    )? {
                                                        let _2 = {
                                                            let _8 = ::musli::de::StructFieldDecoder::decode_field_name(
                                                                &mut _8,
                                                            )?;
                                                            ::musli::de::Decode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::decode(_0, _8)?
                                                        };
                                                        match _2 {
                                                            0usize => {
                                                                ::musli::Context::enter_named_field(_0, "a", &0usize);
                                                                let _8 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                                    _8,
                                                                )?;
                                                                _2_f = ::musli::__priv::Some(
                                                                    ::musli::de::Decode::<
                                                                        ::musli::mode::DefaultMode,
                                                                    >::decode(_0, _8)?,
                                                                );
                                                                ::musli::Context::leave_field(_0);
                                                            }
                                                            1usize => {
                                                                ::musli::Context::enter_named_field(
                                                                    _0,
                                                                    "primitives",
                                                                    &1usize,
                                                                );
                                                                let _8 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                                    _8,
                                                                )?;
                                                                _3_f = ::musli::__priv::Some(
                                                                    ::musli::de::Decode::<
                                                                        ::musli::mode::DefaultMode,
                                                                    >::decode(_0, _8)?,
                                                                );
                                                                ::musli::Context::leave_field(_0);
                                                            }
                                                            2usize => {
                                                                ::musli::Context::enter_named_field(_0, "b", &2usize);
                                                                let _8 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                                    _8,
                                                                )?;
                                                                _4_f = ::musli::__priv::Some(
                                                                    ::musli::de::Decode::<
                                                                        ::musli::mode::DefaultMode,
                                                                    >::decode(_0, _8)?,
                                                                );
                                                                ::musli::Context::leave_field(_0);
                                                            }
                                                            _2 => {
                                                                if ::musli::__priv::skip_field(_8)? {
                                                                    return ::musli::__priv::Err(
                                                                        ::musli::Context::invalid_variant_field_tag(
                                                                            _0,
                                                                            "Struct",
                                                                            &_10,
                                                                            &_2,
                                                                        ),
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }
                                                    ::musli::__priv::Ok(Self::Struct {
                                                        a: match _2_f {
                                                            ::musli::__priv::Some(_2_f) => _2_f,
                                                            ::musli::__priv::None => {
                                                                return ::musli::__priv::Err(
                                                                    ::musli::Context::expected_tag(_0, "Struct", &0usize),
                                                                );
                                                            }
                                                        },
                                                        primitives: match _3_f {
                                                            ::musli::__priv::Some(_3_f) => _3_f,
                                                            ::musli::__priv::None => {
                                                                return ::musli::__priv::Err(
                                                                    ::musli::Context::expected_tag(_0, "Struct", &1usize),
                                                                );
                                                            }
                                                        },
                                                        b: match _4_f {
                                                            ::musli::__priv::Some(_4_f) => _4_f,
                                                            ::musli::__priv::None => {
                                                                return ::musli::__priv::Err(
                                                                    ::musli::Context::expected_tag(_0, "Struct", &2usize),
                                                                );
                                                            }
                                                        },
                                                    })
                                                },
                                            )?
                                        }
                                    };
                                    ::musli::Context::leave_variant(_0);
                                    _7
                                }
                                7usize => {
                                    ::musli::Context::enter_variant(_0, "EmptyStruct", &7usize);
                                    let _7 = {
                                        let _3 = ::musli::de::VariantDecoder::decode_value(_9)?;
                                        {
                                            ::musli::de::Decoder::decode_struct(
                                                _3,
                                                Some(0usize),
                                                move |_16| {
                                                    while let ::musli::__priv::Some(mut _8) = ::musli::de::StructDecoder::decode_field(
                                                        _16,
                                                    )? {
                                                        let _2 = {
                                                            let _8 = ::musli::de::StructFieldDecoder::decode_field_name(
                                                                &mut _8,
                                                            )?;
                                                            ::musli::de::Decode::<
                                                                ::musli::mode::DefaultMode,
                                                            >::decode(_0, _8)?
                                                        };
                                                        if ::musli::__priv::skip_field(_8)? {
                                                            return ::musli::__priv::Err(
                                                                ::musli::Context::invalid_variant_field_tag(
                                                                    _0,
                                                                    "EmptyStruct",
                                                                    &_10,
                                                                    &_2,
                                                                ),
                                                            );
                                                        }
                                                    }
                                                    ::musli::__priv::Ok(Self::EmptyStruct {})
                                                },
                                            )?
                                        }
                                    };
                                    ::musli::Context::leave_variant(_0);
                                    _7
                                }
                                _ => {
                                    return ::musli::__priv::Err(
                                        ::musli::Context::invalid_variant_tag(
                                            _0,
                                            "MediumEnum",
                                            &_10,
                                        ),
                                    );
                                }
                            };
                            ::musli::__priv::Ok(_7)
                        },
                    )?;
                    ::musli::Context::leave_enum(_0);
                    Ok(_7)
                }
            }
        }
    };
    #[automatically_derived]
    impl ::core::fmt::Debug for MediumEnum {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                MediumEnum::Empty => ::core::fmt::Formatter::write_str(f, "Empty"),
                MediumEnum::EmptyTuple() => {
                    ::core::fmt::Formatter::write_str(f, "EmptyTuple")
                }
                MediumEnum::NewType(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "NewType",
                        &__self_0,
                    )
                }
                MediumEnum::Tuple(__self_0, __self_1) => {
                    ::core::fmt::Formatter::debug_tuple_field2_finish(
                        f,
                        "Tuple",
                        __self_0,
                        &__self_1,
                    )
                }
                MediumEnum::NewTypeString(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "NewTypeString",
                        &__self_0,
                    )
                }
                MediumEnum::TupleString(__self_0, __self_1) => {
                    ::core::fmt::Formatter::debug_tuple_field2_finish(
                        f,
                        "TupleString",
                        __self_0,
                        &__self_1,
                    )
                }
                MediumEnum::Struct { a: __self_0, primitives: __self_1, b: __self_2 } => {
                    ::core::fmt::Formatter::debug_struct_field3_finish(
                        f,
                        "Struct",
                        "a",
                        __self_0,
                        "primitives",
                        __self_1,
                        "b",
                        &__self_2,
                    )
                }
                MediumEnum::EmptyStruct {} => {
                    ::core::fmt::Formatter::write_str(f, "EmptyStruct")
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for MediumEnum {
        #[inline]
        fn clone(&self) -> MediumEnum {
            match self {
                MediumEnum::Empty => MediumEnum::Empty,
                MediumEnum::EmptyTuple() => MediumEnum::EmptyTuple(),
                MediumEnum::NewType(__self_0) => {
                    MediumEnum::NewType(::core::clone::Clone::clone(__self_0))
                }
                MediumEnum::Tuple(__self_0, __self_1) => {
                    MediumEnum::Tuple(
                        ::core::clone::Clone::clone(__self_0),
                        ::core::clone::Clone::clone(__self_1),
                    )
                }
                MediumEnum::NewTypeString(__self_0) => {
                    MediumEnum::NewTypeString(::core::clone::Clone::clone(__self_0))
                }
                MediumEnum::TupleString(__self_0, __self_1) => {
                    MediumEnum::TupleString(
                        ::core::clone::Clone::clone(__self_0),
                        ::core::clone::Clone::clone(__self_1),
                    )
                }
                MediumEnum::Struct { a: __self_0, primitives: __self_1, b: __self_2 } => {
                    MediumEnum::Struct {
                        a: ::core::clone::Clone::clone(__self_0),
                        primitives: ::core::clone::Clone::clone(__self_1),
                        b: ::core::clone::Clone::clone(__self_2),
                    }
                }
                MediumEnum::EmptyStruct {} => MediumEnum::EmptyStruct {},
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for MediumEnum {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for MediumEnum {
        #[inline]
        fn eq(&self, other: &MediumEnum) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
                && match (self, other) {
                    (MediumEnum::NewType(__self_0), MediumEnum::NewType(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (
                        MediumEnum::Tuple(__self_0, __self_1),
                        MediumEnum::Tuple(__arg1_0, __arg1_1),
                    ) => *__self_0 == *__arg1_0 && *__self_1 == *__arg1_1,
                    (
                        MediumEnum::NewTypeString(__self_0),
                        MediumEnum::NewTypeString(__arg1_0),
                    ) => *__self_0 == *__arg1_0,
                    (
                        MediumEnum::TupleString(__self_0, __self_1),
                        MediumEnum::TupleString(__arg1_0, __arg1_1),
                    ) => *__self_0 == *__arg1_0 && *__self_1 == *__arg1_1,
                    (
                        MediumEnum::Struct {
                            a: __self_0,
                            primitives: __self_1,
                            b: __self_2,
                        },
                        MediumEnum::Struct {
                            a: __arg1_0,
                            primitives: __arg1_1,
                            b: __arg1_2,
                        },
                    ) => {
                        *__self_0 == *__arg1_0 && *__self_1 == *__arg1_1
                            && *__self_2 == *__arg1_2
                    }
                    _ => true,
                }
        }
    }
    impl Generate for MediumEnum {
        fn generate<__R>(__rng: &mut __R) -> Self
        where
            __R: rand::Rng,
        {
            let mut total = 5usize;
            total += usize::from(true);
            total += usize::from(true);
            total += usize::from(true);
            match __rng.gen_range(0..total) {
                #[cfg(not(feature = "no-empty"))]
                0usize => MediumEnum::Empty {},
                1usize => MediumEnum::EmptyTuple {},
                #[cfg(not(feature = "no-newtype"))]
                2usize => {
                    MediumEnum::NewType {
                        0: <u64 as Generate>::generate(__rng),
                    }
                }
                3usize => {
                    MediumEnum::Tuple {
                        0: <u64 as Generate>::generate(__rng),
                        1: <u64 as Generate>::generate(__rng),
                    }
                }
                #[cfg(not(feature = "no-newtype"))]
                4usize => {
                    MediumEnum::NewTypeString {
                        0: <String as Generate>::generate(__rng),
                    }
                }
                5usize => {
                    MediumEnum::TupleString {
                        0: <String as Generate>::generate(__rng),
                        1: <Vec<u8> as Generate>::generate(__rng),
                    }
                }
                6usize => {
                    MediumEnum::Struct {
                        a: <u32 as Generate>::generate(__rng),
                        primitives: <Primitives as Generate>::generate(__rng),
                        b: <u64 as Generate>::generate(__rng),
                    }
                }
                7usize => MediumEnum::EmptyStruct {},
                _ => ::core::panicking::panic("internal error: entered unreachable code"),
            }
        }
        fn generate_in<__R>(__rng: &mut __R, __out: &mut Vec<Self>)
        where
            __R: rand::Rng,
        {
            #[cfg(not(feature = "no-empty"))]
            {
                __out.push(MediumEnum::Empty {});
            }
            {
                __out.push(MediumEnum::EmptyTuple {});
            }
            #[cfg(not(feature = "no-newtype"))]
            {
                __out
                    .push(MediumEnum::NewType {
                        0: <u64 as Generate>::generate(__rng),
                    });
            }
            {
                __out
                    .push(MediumEnum::Tuple {
                        0: <u64 as Generate>::generate(__rng),
                        1: <u64 as Generate>::generate(__rng),
                    });
            }
            #[cfg(not(feature = "no-newtype"))]
            {
                __out
                    .push(MediumEnum::NewTypeString {
                        0: <String as Generate>::generate(__rng),
                    });
            }
            {
                __out
                    .push(MediumEnum::TupleString {
                        0: <String as Generate>::generate(__rng),
                        1: <Vec<u8> as Generate>::generate(__rng),
                    });
            }
            {
                __out
                    .push(MediumEnum::Struct {
                        a: <u32 as Generate>::generate(__rng),
                        primitives: <Primitives as Generate>::generate(__rng),
                        b: <u64 as Generate>::generate(__rng),
                    });
            }
            {
                __out.push(MediumEnum::EmptyStruct {});
            }
        }
    }
    impl PartialEq<MediumEnum> for &MediumEnum {
        #[inline]
        fn eq(&self, other: &MediumEnum) -> bool {
            *other == **self
        }
    }
    #[musli(mode = Packed, packed)]
    pub struct LargeStruct {
        #[generate(range = PRIMITIVES_RANGE)]
        primitives: Vec<Primitives>,
        #[cfg(not(any(feature = "no-vec", feature = "no-tuple")))]
        #[generate(range = PRIMITIVES_RANGE)]
        tuples: Vec<(Tuples, Tuples)>,
        #[generate(range = MEDIUM_RANGE)]
        medium_vec: Vec<MediumEnum>,
        #[cfg(all(not(feature = "no-map"), not(feature = "no-string-key")))]
        #[generate(range = MEDIUM_RANGE)]
        medium_map: HashMap<String, MediumEnum>,
        #[cfg(all(not(feature = "no-map"), not(feature = "no-string-key")))]
        string_keys: HashMap<String, u64>,
        #[cfg(all(not(feature = "no-map"), not(feature = "no-number-key")))]
        number_map: HashMap<u32, u64>,
        #[cfg(not(feature = "no-tuple"))]
        number_vec: Vec<(u32, u64)>,
    }
    const _: () = {
        #[automatically_derived]
        impl ::musli::en::Encode<Packed> for LargeStruct {
            #[inline]
            fn encode<T0>(
                &self,
                _1: &T0::Cx,
                _0: T0,
            ) -> ::musli::__priv::Result<
                <T0 as ::musli::en::Encoder>::Ok,
                <T0 as ::musli::en::Encoder>::Error,
            >
            where
                T0: ::musli::en::Encoder<Mode = Packed>,
            {
                ::musli::__priv::Ok({
                    ::musli::Context::enter_struct(_1, "LargeStruct");
                    let _3 = ::musli::en::Encoder::encode_pack_fn(
                        _0,
                        move |_2| {
                            ::musli::Context::enter_named_field(
                                _1,
                                "primitives",
                                &0usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.primitives, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "tuples", &1usize);
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<Packed>::encode(&self.tuples, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "medium_vec",
                                &2usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.medium_vec, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "medium_map",
                                &3usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.medium_map, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "string_keys",
                                &4usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.string_keys, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "number_map",
                                &5usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.number_map, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "number_vec",
                                &6usize,
                            );
                            let _4 = ::musli::en::PackEncoder::encode_packed(_2)?;
                            ::musli::en::Encode::<
                                Packed,
                            >::encode(&self.number_vec, _1, _4)?;
                            ::musli::Context::leave_field(_1);
                            ::musli::__priv::Ok(())
                        },
                    )?;
                    ::musli::Context::leave_struct(_1);
                    _3
                })
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        impl ::musli::en::Encode<::musli::mode::DefaultMode> for LargeStruct {
            #[inline]
            fn encode<T0>(
                &self,
                _1: &T0::Cx,
                _0: T0,
            ) -> ::musli::__priv::Result<
                <T0 as ::musli::en::Encoder>::Ok,
                <T0 as ::musli::en::Encoder>::Error,
            >
            where
                T0: ::musli::en::Encoder<Mode = ::musli::mode::DefaultMode>,
            {
                ::musli::__priv::Ok({
                    ::musli::Context::enter_struct(_1, "LargeStruct");
                    let _3 = ::musli::en::Encoder::encode_struct_fn(
                        _0,
                        7usize,
                        move |_0| {
                            ::musli::Context::enter_named_field(
                                _1,
                                "primitives",
                                &0usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&0usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.primitives, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(_1, "tuples", &1usize);
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&1usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.tuples, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "medium_vec",
                                &2usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&2usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.medium_vec, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "medium_map",
                                &3usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&3usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.medium_map, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "string_keys",
                                &4usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&4usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.string_keys, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "number_map",
                                &5usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&5usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.number_map, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::Context::enter_named_field(
                                _1,
                                "number_vec",
                                &6usize,
                            );
                            ::musli::en::StructEncoder::encode_struct_field_fn(
                                _0,
                                move |_5| {
                                    let _6 = ::musli::en::StructFieldEncoder::encode_field_name(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&6usize, _1, _6)?;
                                    let _7 = ::musli::en::StructFieldEncoder::encode_field_value(
                                        _5,
                                    )?;
                                    ::musli::en::Encode::<
                                        ::musli::mode::DefaultMode,
                                    >::encode(&self.number_vec, _1, _7)?;
                                    ::musli::__priv::Ok(())
                                },
                            )?;
                            ::musli::Context::leave_field(_1);
                            ::musli::__priv::Ok(())
                        },
                    )?;
                    ::musli::Context::leave_struct(_1);
                    _3
                })
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        #[allow(clippy::init_numbered_fields)]
        #[allow(clippy::let_unit_value)]
        impl<'de> ::musli::de::Decode<'de, Packed> for LargeStruct {
            #[inline]
            fn decode<T0>(
                _0: &T0::Cx,
                _1: T0,
            ) -> ::musli::__priv::Result<Self, <T0::Cx as ::musli::Context>::Error>
            where
                T0: ::musli::de::Decoder<'de, Mode = Packed>,
            {
                ::musli::__priv::Ok({
                    {
                        ::musli::Context::enter_struct(_0, "LargeStruct");
                        let _3 = ::musli::de::Decoder::decode_pack(
                            _1,
                            move |_5| {
                                Ok(Self {
                                    primitives: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    tuples: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    medium_vec: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    medium_map: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    string_keys: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    number_map: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                    number_vec: {
                                        let _4 = ::musli::de::PackDecoder::decode_next(_5)?;
                                        ::musli::de::Decode::<Packed>::decode(_0, _4)?
                                    },
                                })
                            },
                        )?;
                        ::musli::Context::leave_struct(_0);
                        _3
                    }
                })
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        #[allow(clippy::init_numbered_fields)]
        #[allow(clippy::let_unit_value)]
        impl<'de> ::musli::de::Decode<'de, ::musli::mode::DefaultMode> for LargeStruct {
            #[inline]
            fn decode<T0>(
                _0: &T0::Cx,
                _1: T0,
            ) -> ::musli::__priv::Result<Self, <T0::Cx as ::musli::Context>::Error>
            where
                T0: ::musli::de::Decoder<'de, Mode = ::musli::mode::DefaultMode>,
            {
                ::musli::__priv::Ok({
                    {
                        let mut _0_f: ::musli::__priv::Option<Vec<Primitives>> = ::musli::__priv::None;
                        let mut _1_f: ::musli::__priv::Option<Vec<(Tuples, Tuples)>> = ::musli::__priv::None;
                        let mut _2_f: ::musli::__priv::Option<Vec<MediumEnum>> = ::musli::__priv::None;
                        let mut _3_f: ::musli::__priv::Option<
                            HashMap<String, MediumEnum>,
                        > = ::musli::__priv::None;
                        let mut _4_f: ::musli::__priv::Option<HashMap<String, u64>> = ::musli::__priv::None;
                        let mut _5_f: ::musli::__priv::Option<HashMap<u32, u64>> = ::musli::__priv::None;
                        let mut _6_f: ::musli::__priv::Option<Vec<(u32, u64)>> = ::musli::__priv::None;
                        ::musli::Context::enter_struct(_0, "LargeStruct");
                        ::musli::de::Decoder::decode_struct(
                            _1,
                            Some(7usize),
                            move |_5| {
                                while let ::musli::__priv::Some(mut _3) = ::musli::de::StructDecoder::decode_field(
                                    _5,
                                )? {
                                    let _2 = {
                                        let _3 = ::musli::de::StructFieldDecoder::decode_field_name(
                                            &mut _3,
                                        )?;
                                        ::musli::de::Decode::<
                                            ::musli::mode::DefaultMode,
                                        >::decode(_0, _3)?
                                    };
                                    match _2 {
                                        0usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "primitives",
                                                &0usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _0_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        1usize => {
                                            ::musli::Context::enter_named_field(_0, "tuples", &1usize);
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _1_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        2usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "medium_vec",
                                                &2usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _2_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        3usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "medium_map",
                                                &3usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _3_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        4usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "string_keys",
                                                &4usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _4_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        5usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "number_map",
                                                &5usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _5_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        6usize => {
                                            ::musli::Context::enter_named_field(
                                                _0,
                                                "number_vec",
                                                &6usize,
                                            );
                                            let _3 = ::musli::de::StructFieldDecoder::decode_field_value(
                                                _3,
                                            )?;
                                            _6_f = ::musli::__priv::Some(
                                                ::musli::de::Decode::<
                                                    ::musli::mode::DefaultMode,
                                                >::decode(_0, _3)?,
                                            );
                                            ::musli::Context::leave_field(_0);
                                        }
                                        _2 => {
                                            if ::musli::__priv::skip_field(_3)? {
                                                return ::musli::__priv::Err(
                                                    ::musli::Context::invalid_field_tag(_0, "LargeStruct", &_2),
                                                );
                                            }
                                        }
                                    }
                                }
                                ::musli::Context::leave_struct(_0);
                                ::musli::__priv::Ok(Self {
                                    primitives: match _0_f {
                                        ::musli::__priv::Some(_0_f) => _0_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "LargeStruct", &0usize),
                                            );
                                        }
                                    },
                                    tuples: match _1_f {
                                        ::musli::__priv::Some(_1_f) => _1_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "LargeStruct", &1usize),
                                            );
                                        }
                                    },
                                    medium_vec: match _2_f {
                                        ::musli::__priv::Some(_2_f) => _2_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "LargeStruct", &2usize),
                                            );
                                        }
                                    },
                                    medium_map: match _3_f {
                                        ::musli::__priv::Some(_3_f) => _3_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "LargeStruct", &3usize),
                                            );
                                        }
                                    },
                                    string_keys: match _4_f {
                                        ::musli::__priv::Some(_4_f) => _4_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "LargeStruct", &4usize),
                                            );
                                        }
                                    },
                                    number_map: match _5_f {
                                        ::musli::__priv::Some(_5_f) => _5_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "LargeStruct", &5usize),
                                            );
                                        }
                                    },
                                    number_vec: match _6_f {
                                        ::musli::__priv::Some(_6_f) => _6_f,
                                        ::musli::__priv::None => {
                                            return ::musli::__priv::Err(
                                                ::musli::Context::expected_tag(_0, "LargeStruct", &6usize),
                                            );
                                        }
                                    },
                                })
                            },
                        )?
                    }
                })
            }
        }
    };
    #[automatically_derived]
    impl ::core::fmt::Debug for LargeStruct {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "primitives",
                "tuples",
                "medium_vec",
                "medium_map",
                "string_keys",
                "number_map",
                "number_vec",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.primitives,
                &self.tuples,
                &self.medium_vec,
                &self.medium_map,
                &self.string_keys,
                &self.number_map,
                &&self.number_vec,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "LargeStruct",
                names,
                values,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for LargeStruct {
        #[inline]
        fn clone(&self) -> LargeStruct {
            LargeStruct {
                primitives: ::core::clone::Clone::clone(&self.primitives),
                tuples: ::core::clone::Clone::clone(&self.tuples),
                medium_vec: ::core::clone::Clone::clone(&self.medium_vec),
                medium_map: ::core::clone::Clone::clone(&self.medium_map),
                string_keys: ::core::clone::Clone::clone(&self.string_keys),
                number_map: ::core::clone::Clone::clone(&self.number_map),
                number_vec: ::core::clone::Clone::clone(&self.number_vec),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for LargeStruct {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for LargeStruct {
        #[inline]
        fn eq(&self, other: &LargeStruct) -> bool {
            self.primitives == other.primitives && self.tuples == other.tuples
                && self.medium_vec == other.medium_vec
                && self.medium_map == other.medium_map
                && self.string_keys == other.string_keys
                && self.number_map == other.number_map
                && self.number_vec == other.number_vec
        }
    }
    impl Generate for LargeStruct {
        fn generate<__R>(__rng: &mut __R) -> Self
        where
            __R: rand::Rng,
        {
            Self {
                primitives: <Vec<
                    Primitives,
                > as Generate>::generate_range(__rng, PRIMITIVES_RANGE),
                #[cfg(not(any(feature = "no-vec", feature = "no-tuple")))]
                tuples: <Vec<
                    (Tuples, Tuples),
                > as Generate>::generate_range(__rng, PRIMITIVES_RANGE),
                medium_vec: <Vec<
                    MediumEnum,
                > as Generate>::generate_range(__rng, MEDIUM_RANGE),
                #[cfg(all(not(feature = "no-map"), not(feature = "no-string-key")))]
                medium_map: <HashMap<
                    String,
                    MediumEnum,
                > as Generate>::generate_range(__rng, MEDIUM_RANGE),
                #[cfg(all(not(feature = "no-map"), not(feature = "no-string-key")))]
                string_keys: <HashMap<String, u64> as Generate>::generate(__rng),
                #[cfg(all(not(feature = "no-map"), not(feature = "no-number-key")))]
                number_map: <HashMap<u32, u64> as Generate>::generate(__rng),
                #[cfg(not(feature = "no-tuple"))]
                number_vec: <Vec<(u32, u64)> as Generate>::generate(__rng),
            }
        }
    }
    impl PartialEq<LargeStruct> for &LargeStruct {
        #[inline]
        fn eq(&self, other: &LargeStruct) -> bool {
            *other == **self
        }
    }
}
pub mod utils {
    #[allow(unused_imports)]
    pub use self::full::*;
    mod full {}
    #[allow(unused_imports)]
    pub use self::extra::*;
    mod extra {}
    #[allow(unused_imports)]
    pub use self::musli::*;
    mod musli {}
}
pub use self::aligned_buf::AlignedBuf;
mod aligned_buf {
    use alloc::alloc;
    use core::alloc::Layout;
    use core::mem::size_of_val;
    use core::ptr::NonNull;
    use core::slice;
    /// A bytes vector that can have a specific alignment.
    pub struct AlignedBuf {
        alignment: usize,
        len: usize,
        capacity: usize,
        data: NonNull<u8>,
    }
    impl AlignedBuf {
        /// Construct an alignable vec with the given alignment.
        #[inline]
        pub fn new(alignment: usize) -> Self {
            if !alignment.is_power_of_two() {
                ::core::panicking::panic("assertion failed: alignment.is_power_of_two()")
            }
            Self {
                alignment,
                len: 0,
                capacity: 0,
                data: unsafe { dangling(alignment) },
            }
        }
        #[inline]
        pub fn reserve(&mut self, capacity: usize) {
            let new_capacity = self.len + capacity;
            self.ensure_capacity(new_capacity);
        }
        #[inline]
        pub fn extend_from_slice(&mut self, bytes: &[u8]) {
            self.reserve(bytes.len());
            unsafe {
                self.store_bytes(bytes);
            }
        }
        #[inline]
        pub(crate) unsafe fn store_bytes<T>(&mut self, values: &[T]) {
            let dst = self.data.as_ptr().wrapping_add(self.len);
            dst.copy_from_nonoverlapping(values.as_ptr().cast(), size_of_val(values));
            self.len += size_of_val(values);
        }
        #[inline]
        pub fn as_slice(&self) -> &[u8] {
            unsafe { slice::from_raw_parts(self.data.as_ptr() as *const _, self.len) }
        }
        #[inline(never)]
        fn ensure_capacity(&mut self, new_capacity: usize) {
            if self.capacity >= new_capacity {
                return;
            }
            let new_capacity = new_capacity.max((self.capacity as f32 * 1.5) as usize);
            let (old_layout, new_layout) = self.layouts(new_capacity);
            if old_layout.size() == 0 {
                self.alloc_init(new_layout);
            } else {
                self.alloc_realloc(old_layout, new_layout);
            }
        }
        /// Perform the initial allocation with the given layout and capacity.
        fn alloc_init(&mut self, new_layout: Layout) {
            unsafe {
                let ptr = alloc::alloc(new_layout);
                if ptr.is_null() {
                    alloc::handle_alloc_error(new_layout);
                }
                self.data = NonNull::new_unchecked(ptr);
                self.capacity = new_layout.size();
            }
        }
        /// Reallocate, note that the alignment of the old layout must match the new
        /// one.
        fn alloc_realloc(&mut self, old_layout: Layout, new_layout: Layout) {
            if true {
                match (&old_layout.align(), &new_layout.align()) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            let kind = ::core::panicking::AssertKind::Eq;
                            ::core::panicking::assert_failed(
                                kind,
                                &*left_val,
                                &*right_val,
                                ::core::option::Option::None,
                            );
                        }
                    }
                };
            }
            unsafe {
                let ptr = alloc::realloc(
                    self.data.as_ptr(),
                    old_layout,
                    new_layout.size(),
                );
                if ptr.is_null() {
                    alloc::handle_alloc_error(old_layout);
                }
                self.data = NonNull::new_unchecked(ptr);
                self.capacity = new_layout.size();
            }
        }
        /// Return a pair of the currently allocated layout, and new layout that is
        /// requested with the given capacity.
        #[inline]
        fn layouts(&self, new_capacity: usize) -> (Layout, Layout) {
            let old_layout = unsafe {
                Layout::from_size_align_unchecked(self.capacity, self.alignment)
            };
            let layout = Layout::from_size_align(new_capacity, self.alignment)
                .expect("Proposed layout invalid");
            (old_layout, layout)
        }
    }
    impl Drop for AlignedBuf {
        fn drop(&mut self) {
            unsafe {
                if self.capacity != 0 {
                    let layout = Layout::from_size_align_unchecked(
                        self.capacity,
                        self.alignment,
                    );
                    alloc::dealloc(self.data.as_ptr(), layout);
                    self.capacity = 0;
                }
            }
        }
    }
    const unsafe fn dangling(align: usize) -> NonNull<u8> {
        NonNull::new_unchecked(invalid_mut(align))
    }
    #[allow(clippy::useless_transmute)]
    const fn invalid_mut<T>(addr: usize) -> *mut T {
        unsafe { core::mem::transmute(addr) }
    }
}
/// Build common RNG with custom seed.
pub fn rng_with_seed(seed: u64) -> generate::Rng {
    generate::Rng::from_seed(seed)
}
/// Build common RNG.
pub fn rng() -> generate::Rng {
    rng_with_seed(RNG_SEED)
}
