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
mod mode {}
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
    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};
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
        _pad0: [u8; 1],
        unsigned16: u16,
        unsigned32: u32,
        unsigned64: u64,
        #[cfg(not(feature = "no-128"))]
        unsigned128: u128,
        signed8: i8,
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
        _pad3: [u8; 4],
        float64: f64,
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for PrimitivesPacked {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "PrimitivesPacked",
                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
                        + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "unsigned8",
                    &self.unsigned8,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "_pad0",
                    &self._pad0,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "unsigned16",
                    &self.unsigned16,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "unsigned32",
                    &self.unsigned32,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "unsigned64",
                    &self.unsigned64,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "unsigned128",
                    &self.unsigned128,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "signed8",
                    &self.signed8,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "_pad1",
                    &self._pad1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "signed16",
                    &self.signed16,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "signed32",
                    &self.signed32,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "signed64",
                    &self.signed64,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "signed128",
                    &self.signed128,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "unsignedsize",
                    &self.unsignedsize,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "signedsize",
                    &self.signedsize,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "float32",
                    &self.float32,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "_pad3",
                    &self._pad3,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "float64",
                    &self.float64,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for PrimitivesPacked {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __field6,
                    __field7,
                    __field8,
                    __field9,
                    __field10,
                    __field11,
                    __field12,
                    __field13,
                    __field14,
                    __field15,
                    __field16,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            4u64 => _serde::__private::Ok(__Field::__field4),
                            5u64 => _serde::__private::Ok(__Field::__field5),
                            6u64 => _serde::__private::Ok(__Field::__field6),
                            7u64 => _serde::__private::Ok(__Field::__field7),
                            8u64 => _serde::__private::Ok(__Field::__field8),
                            9u64 => _serde::__private::Ok(__Field::__field9),
                            10u64 => _serde::__private::Ok(__Field::__field10),
                            11u64 => _serde::__private::Ok(__Field::__field11),
                            12u64 => _serde::__private::Ok(__Field::__field12),
                            13u64 => _serde::__private::Ok(__Field::__field13),
                            14u64 => _serde::__private::Ok(__Field::__field14),
                            15u64 => _serde::__private::Ok(__Field::__field15),
                            16u64 => _serde::__private::Ok(__Field::__field16),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "unsigned8" => _serde::__private::Ok(__Field::__field0),
                            "_pad0" => _serde::__private::Ok(__Field::__field1),
                            "unsigned16" => _serde::__private::Ok(__Field::__field2),
                            "unsigned32" => _serde::__private::Ok(__Field::__field3),
                            "unsigned64" => _serde::__private::Ok(__Field::__field4),
                            "unsigned128" => _serde::__private::Ok(__Field::__field5),
                            "signed8" => _serde::__private::Ok(__Field::__field6),
                            "_pad1" => _serde::__private::Ok(__Field::__field7),
                            "signed16" => _serde::__private::Ok(__Field::__field8),
                            "signed32" => _serde::__private::Ok(__Field::__field9),
                            "signed64" => _serde::__private::Ok(__Field::__field10),
                            "signed128" => _serde::__private::Ok(__Field::__field11),
                            "unsignedsize" => _serde::__private::Ok(__Field::__field12),
                            "signedsize" => _serde::__private::Ok(__Field::__field13),
                            "float32" => _serde::__private::Ok(__Field::__field14),
                            "_pad3" => _serde::__private::Ok(__Field::__field15),
                            "float64" => _serde::__private::Ok(__Field::__field16),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"unsigned8" => _serde::__private::Ok(__Field::__field0),
                            b"_pad0" => _serde::__private::Ok(__Field::__field1),
                            b"unsigned16" => _serde::__private::Ok(__Field::__field2),
                            b"unsigned32" => _serde::__private::Ok(__Field::__field3),
                            b"unsigned64" => _serde::__private::Ok(__Field::__field4),
                            b"unsigned128" => _serde::__private::Ok(__Field::__field5),
                            b"signed8" => _serde::__private::Ok(__Field::__field6),
                            b"_pad1" => _serde::__private::Ok(__Field::__field7),
                            b"signed16" => _serde::__private::Ok(__Field::__field8),
                            b"signed32" => _serde::__private::Ok(__Field::__field9),
                            b"signed64" => _serde::__private::Ok(__Field::__field10),
                            b"signed128" => _serde::__private::Ok(__Field::__field11),
                            b"unsignedsize" => _serde::__private::Ok(__Field::__field12),
                            b"signedsize" => _serde::__private::Ok(__Field::__field13),
                            b"float32" => _serde::__private::Ok(__Field::__field14),
                            b"_pad3" => _serde::__private::Ok(__Field::__field15),
                            b"float64" => _serde::__private::Ok(__Field::__field16),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<PrimitivesPacked>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = PrimitivesPacked;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct PrimitivesPacked",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            u8,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct PrimitivesPacked with 17 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            [u8; 1],
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct PrimitivesPacked with 17 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            u16,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct PrimitivesPacked with 17 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            u32,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct PrimitivesPacked with 17 elements",
                                    ),
                                );
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            u64,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct PrimitivesPacked with 17 elements",
                                    ),
                                );
                            }
                        };
                        let __field5 = match _serde::de::SeqAccess::next_element::<
                            u128,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        5usize,
                                        &"struct PrimitivesPacked with 17 elements",
                                    ),
                                );
                            }
                        };
                        let __field6 = match _serde::de::SeqAccess::next_element::<
                            i8,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        6usize,
                                        &"struct PrimitivesPacked with 17 elements",
                                    ),
                                );
                            }
                        };
                        let __field7 = match _serde::de::SeqAccess::next_element::<
                            [u8; 1],
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        7usize,
                                        &"struct PrimitivesPacked with 17 elements",
                                    ),
                                );
                            }
                        };
                        let __field8 = match _serde::de::SeqAccess::next_element::<
                            i16,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        8usize,
                                        &"struct PrimitivesPacked with 17 elements",
                                    ),
                                );
                            }
                        };
                        let __field9 = match _serde::de::SeqAccess::next_element::<
                            i32,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        9usize,
                                        &"struct PrimitivesPacked with 17 elements",
                                    ),
                                );
                            }
                        };
                        let __field10 = match _serde::de::SeqAccess::next_element::<
                            i64,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        10usize,
                                        &"struct PrimitivesPacked with 17 elements",
                                    ),
                                );
                            }
                        };
                        let __field11 = match _serde::de::SeqAccess::next_element::<
                            i128,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        11usize,
                                        &"struct PrimitivesPacked with 17 elements",
                                    ),
                                );
                            }
                        };
                        let __field12 = match _serde::de::SeqAccess::next_element::<
                            usize,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        12usize,
                                        &"struct PrimitivesPacked with 17 elements",
                                    ),
                                );
                            }
                        };
                        let __field13 = match _serde::de::SeqAccess::next_element::<
                            isize,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        13usize,
                                        &"struct PrimitivesPacked with 17 elements",
                                    ),
                                );
                            }
                        };
                        let __field14 = match _serde::de::SeqAccess::next_element::<
                            f32,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        14usize,
                                        &"struct PrimitivesPacked with 17 elements",
                                    ),
                                );
                            }
                        };
                        let __field15 = match _serde::de::SeqAccess::next_element::<
                            [u8; 4],
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        15usize,
                                        &"struct PrimitivesPacked with 17 elements",
                                    ),
                                );
                            }
                        };
                        let __field16 = match _serde::de::SeqAccess::next_element::<
                            f64,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        16usize,
                                        &"struct PrimitivesPacked with 17 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(PrimitivesPacked {
                            unsigned8: __field0,
                            _pad0: __field1,
                            unsigned16: __field2,
                            unsigned32: __field3,
                            unsigned64: __field4,
                            unsigned128: __field5,
                            signed8: __field6,
                            _pad1: __field7,
                            signed16: __field8,
                            signed32: __field9,
                            signed64: __field10,
                            signed128: __field11,
                            unsignedsize: __field12,
                            signedsize: __field13,
                            float32: __field14,
                            _pad3: __field15,
                            float64: __field16,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<u8> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<[u8; 1]> = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<u16> = _serde::__private::None;
                        let mut __field3: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field4: _serde::__private::Option<u64> = _serde::__private::None;
                        let mut __field5: _serde::__private::Option<u128> = _serde::__private::None;
                        let mut __field6: _serde::__private::Option<i8> = _serde::__private::None;
                        let mut __field7: _serde::__private::Option<[u8; 1]> = _serde::__private::None;
                        let mut __field8: _serde::__private::Option<i16> = _serde::__private::None;
                        let mut __field9: _serde::__private::Option<i32> = _serde::__private::None;
                        let mut __field10: _serde::__private::Option<i64> = _serde::__private::None;
                        let mut __field11: _serde::__private::Option<i128> = _serde::__private::None;
                        let mut __field12: _serde::__private::Option<usize> = _serde::__private::None;
                        let mut __field13: _serde::__private::Option<isize> = _serde::__private::None;
                        let mut __field14: _serde::__private::Option<f32> = _serde::__private::None;
                        let mut __field15: _serde::__private::Option<[u8; 4]> = _serde::__private::None;
                        let mut __field16: _serde::__private::Option<f64> = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "unsigned8",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<u8>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("_pad0"),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<[u8; 1]>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "unsigned16",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<u16>(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private::Option::is_some(&__field3) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "unsigned32",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<u32>(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private::Option::is_some(&__field4) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "unsigned64",
                                            ),
                                        );
                                    }
                                    __field4 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<u64>(&mut __map)?,
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private::Option::is_some(&__field5) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "unsigned128",
                                            ),
                                        );
                                    }
                                    __field5 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<u128>(&mut __map)?,
                                    );
                                }
                                __Field::__field6 => {
                                    if _serde::__private::Option::is_some(&__field6) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "signed8",
                                            ),
                                        );
                                    }
                                    __field6 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<i8>(&mut __map)?,
                                    );
                                }
                                __Field::__field7 => {
                                    if _serde::__private::Option::is_some(&__field7) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("_pad1"),
                                        );
                                    }
                                    __field7 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<[u8; 1]>(&mut __map)?,
                                    );
                                }
                                __Field::__field8 => {
                                    if _serde::__private::Option::is_some(&__field8) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "signed16",
                                            ),
                                        );
                                    }
                                    __field8 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<i16>(&mut __map)?,
                                    );
                                }
                                __Field::__field9 => {
                                    if _serde::__private::Option::is_some(&__field9) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "signed32",
                                            ),
                                        );
                                    }
                                    __field9 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<i32>(&mut __map)?,
                                    );
                                }
                                __Field::__field10 => {
                                    if _serde::__private::Option::is_some(&__field10) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "signed64",
                                            ),
                                        );
                                    }
                                    __field10 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<i64>(&mut __map)?,
                                    );
                                }
                                __Field::__field11 => {
                                    if _serde::__private::Option::is_some(&__field11) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "signed128",
                                            ),
                                        );
                                    }
                                    __field11 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<i128>(&mut __map)?,
                                    );
                                }
                                __Field::__field12 => {
                                    if _serde::__private::Option::is_some(&__field12) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "unsignedsize",
                                            ),
                                        );
                                    }
                                    __field12 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<usize>(&mut __map)?,
                                    );
                                }
                                __Field::__field13 => {
                                    if _serde::__private::Option::is_some(&__field13) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "signedsize",
                                            ),
                                        );
                                    }
                                    __field13 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<isize>(&mut __map)?,
                                    );
                                }
                                __Field::__field14 => {
                                    if _serde::__private::Option::is_some(&__field14) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "float32",
                                            ),
                                        );
                                    }
                                    __field14 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<f32>(&mut __map)?,
                                    );
                                }
                                __Field::__field15 => {
                                    if _serde::__private::Option::is_some(&__field15) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("_pad3"),
                                        );
                                    }
                                    __field15 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<[u8; 4]>(&mut __map)?,
                                    );
                                }
                                __Field::__field16 => {
                                    if _serde::__private::Option::is_some(&__field16) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "float64",
                                            ),
                                        );
                                    }
                                    __field16 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<f64>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("unsigned8")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("_pad0")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("unsigned16")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private::Some(__field3) => __field3,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("unsigned32")?
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private::Some(__field4) => __field4,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("unsigned64")?
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private::Some(__field5) => __field5,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("unsigned128")?
                            }
                        };
                        let __field6 = match __field6 {
                            _serde::__private::Some(__field6) => __field6,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("signed8")?
                            }
                        };
                        let __field7 = match __field7 {
                            _serde::__private::Some(__field7) => __field7,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("_pad1")?
                            }
                        };
                        let __field8 = match __field8 {
                            _serde::__private::Some(__field8) => __field8,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("signed16")?
                            }
                        };
                        let __field9 = match __field9 {
                            _serde::__private::Some(__field9) => __field9,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("signed32")?
                            }
                        };
                        let __field10 = match __field10 {
                            _serde::__private::Some(__field10) => __field10,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("signed64")?
                            }
                        };
                        let __field11 = match __field11 {
                            _serde::__private::Some(__field11) => __field11,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("signed128")?
                            }
                        };
                        let __field12 = match __field12 {
                            _serde::__private::Some(__field12) => __field12,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("unsignedsize")?
                            }
                        };
                        let __field13 = match __field13 {
                            _serde::__private::Some(__field13) => __field13,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("signedsize")?
                            }
                        };
                        let __field14 = match __field14 {
                            _serde::__private::Some(__field14) => __field14,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("float32")?
                            }
                        };
                        let __field15 = match __field15 {
                            _serde::__private::Some(__field15) => __field15,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("_pad3")?
                            }
                        };
                        let __field16 = match __field16 {
                            _serde::__private::Some(__field16) => __field16,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("float64")?
                            }
                        };
                        _serde::__private::Ok(PrimitivesPacked {
                            unsigned8: __field0,
                            _pad0: __field1,
                            unsigned16: __field2,
                            unsigned32: __field3,
                            unsigned64: __field4,
                            unsigned128: __field5,
                            signed8: __field6,
                            _pad1: __field7,
                            signed16: __field8,
                            signed32: __field9,
                            signed64: __field10,
                            signed128: __field11,
                            unsignedsize: __field12,
                            signedsize: __field13,
                            float32: __field14,
                            _pad3: __field15,
                            float64: __field16,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
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
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "PrimitivesPacked",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<PrimitivesPacked>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
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
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for Primitives {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "Primitives",
                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
                        + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "boolean",
                    &self.boolean,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "character",
                    &self.character,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "unsigned8",
                    &self.unsigned8,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "unsigned16",
                    &self.unsigned16,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "unsigned32",
                    &self.unsigned32,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "unsigned64",
                    &self.unsigned64,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "unsigned128",
                    &self.unsigned128,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "signed8",
                    &self.signed8,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "signed16",
                    &self.signed16,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "signed32",
                    &self.signed32,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "signed64",
                    &self.signed64,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "signed128",
                    &self.signed128,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "unsignedsize",
                    &self.unsignedsize,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "signedsize",
                    &self.signedsize,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "float32",
                    &self.float32,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "float64",
                    &self.float64,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for Primitives {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __field6,
                    __field7,
                    __field8,
                    __field9,
                    __field10,
                    __field11,
                    __field12,
                    __field13,
                    __field14,
                    __field15,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            4u64 => _serde::__private::Ok(__Field::__field4),
                            5u64 => _serde::__private::Ok(__Field::__field5),
                            6u64 => _serde::__private::Ok(__Field::__field6),
                            7u64 => _serde::__private::Ok(__Field::__field7),
                            8u64 => _serde::__private::Ok(__Field::__field8),
                            9u64 => _serde::__private::Ok(__Field::__field9),
                            10u64 => _serde::__private::Ok(__Field::__field10),
                            11u64 => _serde::__private::Ok(__Field::__field11),
                            12u64 => _serde::__private::Ok(__Field::__field12),
                            13u64 => _serde::__private::Ok(__Field::__field13),
                            14u64 => _serde::__private::Ok(__Field::__field14),
                            15u64 => _serde::__private::Ok(__Field::__field15),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "boolean" => _serde::__private::Ok(__Field::__field0),
                            "character" => _serde::__private::Ok(__Field::__field1),
                            "unsigned8" => _serde::__private::Ok(__Field::__field2),
                            "unsigned16" => _serde::__private::Ok(__Field::__field3),
                            "unsigned32" => _serde::__private::Ok(__Field::__field4),
                            "unsigned64" => _serde::__private::Ok(__Field::__field5),
                            "unsigned128" => _serde::__private::Ok(__Field::__field6),
                            "signed8" => _serde::__private::Ok(__Field::__field7),
                            "signed16" => _serde::__private::Ok(__Field::__field8),
                            "signed32" => _serde::__private::Ok(__Field::__field9),
                            "signed64" => _serde::__private::Ok(__Field::__field10),
                            "signed128" => _serde::__private::Ok(__Field::__field11),
                            "unsignedsize" => _serde::__private::Ok(__Field::__field12),
                            "signedsize" => _serde::__private::Ok(__Field::__field13),
                            "float32" => _serde::__private::Ok(__Field::__field14),
                            "float64" => _serde::__private::Ok(__Field::__field15),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"boolean" => _serde::__private::Ok(__Field::__field0),
                            b"character" => _serde::__private::Ok(__Field::__field1),
                            b"unsigned8" => _serde::__private::Ok(__Field::__field2),
                            b"unsigned16" => _serde::__private::Ok(__Field::__field3),
                            b"unsigned32" => _serde::__private::Ok(__Field::__field4),
                            b"unsigned64" => _serde::__private::Ok(__Field::__field5),
                            b"unsigned128" => _serde::__private::Ok(__Field::__field6),
                            b"signed8" => _serde::__private::Ok(__Field::__field7),
                            b"signed16" => _serde::__private::Ok(__Field::__field8),
                            b"signed32" => _serde::__private::Ok(__Field::__field9),
                            b"signed64" => _serde::__private::Ok(__Field::__field10),
                            b"signed128" => _serde::__private::Ok(__Field::__field11),
                            b"unsignedsize" => _serde::__private::Ok(__Field::__field12),
                            b"signedsize" => _serde::__private::Ok(__Field::__field13),
                            b"float32" => _serde::__private::Ok(__Field::__field14),
                            b"float64" => _serde::__private::Ok(__Field::__field15),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<Primitives>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = Primitives;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct Primitives",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            bool,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct Primitives with 16 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            char,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct Primitives with 16 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            u8,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct Primitives with 16 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            u16,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct Primitives with 16 elements",
                                    ),
                                );
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            u32,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct Primitives with 16 elements",
                                    ),
                                );
                            }
                        };
                        let __field5 = match _serde::de::SeqAccess::next_element::<
                            u64,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        5usize,
                                        &"struct Primitives with 16 elements",
                                    ),
                                );
                            }
                        };
                        let __field6 = match _serde::de::SeqAccess::next_element::<
                            u128,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        6usize,
                                        &"struct Primitives with 16 elements",
                                    ),
                                );
                            }
                        };
                        let __field7 = match _serde::de::SeqAccess::next_element::<
                            i8,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        7usize,
                                        &"struct Primitives with 16 elements",
                                    ),
                                );
                            }
                        };
                        let __field8 = match _serde::de::SeqAccess::next_element::<
                            i16,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        8usize,
                                        &"struct Primitives with 16 elements",
                                    ),
                                );
                            }
                        };
                        let __field9 = match _serde::de::SeqAccess::next_element::<
                            i32,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        9usize,
                                        &"struct Primitives with 16 elements",
                                    ),
                                );
                            }
                        };
                        let __field10 = match _serde::de::SeqAccess::next_element::<
                            i64,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        10usize,
                                        &"struct Primitives with 16 elements",
                                    ),
                                );
                            }
                        };
                        let __field11 = match _serde::de::SeqAccess::next_element::<
                            i128,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        11usize,
                                        &"struct Primitives with 16 elements",
                                    ),
                                );
                            }
                        };
                        let __field12 = match _serde::de::SeqAccess::next_element::<
                            usize,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        12usize,
                                        &"struct Primitives with 16 elements",
                                    ),
                                );
                            }
                        };
                        let __field13 = match _serde::de::SeqAccess::next_element::<
                            isize,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        13usize,
                                        &"struct Primitives with 16 elements",
                                    ),
                                );
                            }
                        };
                        let __field14 = match _serde::de::SeqAccess::next_element::<
                            f32,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        14usize,
                                        &"struct Primitives with 16 elements",
                                    ),
                                );
                            }
                        };
                        let __field15 = match _serde::de::SeqAccess::next_element::<
                            f64,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        15usize,
                                        &"struct Primitives with 16 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(Primitives {
                            boolean: __field0,
                            character: __field1,
                            unsigned8: __field2,
                            unsigned16: __field3,
                            unsigned32: __field4,
                            unsigned64: __field5,
                            unsigned128: __field6,
                            signed8: __field7,
                            signed16: __field8,
                            signed32: __field9,
                            signed64: __field10,
                            signed128: __field11,
                            unsignedsize: __field12,
                            signedsize: __field13,
                            float32: __field14,
                            float64: __field15,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<bool> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<char> = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<u8> = _serde::__private::None;
                        let mut __field3: _serde::__private::Option<u16> = _serde::__private::None;
                        let mut __field4: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field5: _serde::__private::Option<u64> = _serde::__private::None;
                        let mut __field6: _serde::__private::Option<u128> = _serde::__private::None;
                        let mut __field7: _serde::__private::Option<i8> = _serde::__private::None;
                        let mut __field8: _serde::__private::Option<i16> = _serde::__private::None;
                        let mut __field9: _serde::__private::Option<i32> = _serde::__private::None;
                        let mut __field10: _serde::__private::Option<i64> = _serde::__private::None;
                        let mut __field11: _serde::__private::Option<i128> = _serde::__private::None;
                        let mut __field12: _serde::__private::Option<usize> = _serde::__private::None;
                        let mut __field13: _serde::__private::Option<isize> = _serde::__private::None;
                        let mut __field14: _serde::__private::Option<f32> = _serde::__private::None;
                        let mut __field15: _serde::__private::Option<f64> = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "boolean",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<bool>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "character",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<char>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "unsigned8",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<u8>(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private::Option::is_some(&__field3) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "unsigned16",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<u16>(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private::Option::is_some(&__field4) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "unsigned32",
                                            ),
                                        );
                                    }
                                    __field4 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<u32>(&mut __map)?,
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private::Option::is_some(&__field5) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "unsigned64",
                                            ),
                                        );
                                    }
                                    __field5 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<u64>(&mut __map)?,
                                    );
                                }
                                __Field::__field6 => {
                                    if _serde::__private::Option::is_some(&__field6) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "unsigned128",
                                            ),
                                        );
                                    }
                                    __field6 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<u128>(&mut __map)?,
                                    );
                                }
                                __Field::__field7 => {
                                    if _serde::__private::Option::is_some(&__field7) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "signed8",
                                            ),
                                        );
                                    }
                                    __field7 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<i8>(&mut __map)?,
                                    );
                                }
                                __Field::__field8 => {
                                    if _serde::__private::Option::is_some(&__field8) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "signed16",
                                            ),
                                        );
                                    }
                                    __field8 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<i16>(&mut __map)?,
                                    );
                                }
                                __Field::__field9 => {
                                    if _serde::__private::Option::is_some(&__field9) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "signed32",
                                            ),
                                        );
                                    }
                                    __field9 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<i32>(&mut __map)?,
                                    );
                                }
                                __Field::__field10 => {
                                    if _serde::__private::Option::is_some(&__field10) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "signed64",
                                            ),
                                        );
                                    }
                                    __field10 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<i64>(&mut __map)?,
                                    );
                                }
                                __Field::__field11 => {
                                    if _serde::__private::Option::is_some(&__field11) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "signed128",
                                            ),
                                        );
                                    }
                                    __field11 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<i128>(&mut __map)?,
                                    );
                                }
                                __Field::__field12 => {
                                    if _serde::__private::Option::is_some(&__field12) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "unsignedsize",
                                            ),
                                        );
                                    }
                                    __field12 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<usize>(&mut __map)?,
                                    );
                                }
                                __Field::__field13 => {
                                    if _serde::__private::Option::is_some(&__field13) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "signedsize",
                                            ),
                                        );
                                    }
                                    __field13 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<isize>(&mut __map)?,
                                    );
                                }
                                __Field::__field14 => {
                                    if _serde::__private::Option::is_some(&__field14) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "float32",
                                            ),
                                        );
                                    }
                                    __field14 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<f32>(&mut __map)?,
                                    );
                                }
                                __Field::__field15 => {
                                    if _serde::__private::Option::is_some(&__field15) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "float64",
                                            ),
                                        );
                                    }
                                    __field15 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<f64>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("boolean")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("character")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("unsigned8")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private::Some(__field3) => __field3,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("unsigned16")?
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private::Some(__field4) => __field4,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("unsigned32")?
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private::Some(__field5) => __field5,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("unsigned64")?
                            }
                        };
                        let __field6 = match __field6 {
                            _serde::__private::Some(__field6) => __field6,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("unsigned128")?
                            }
                        };
                        let __field7 = match __field7 {
                            _serde::__private::Some(__field7) => __field7,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("signed8")?
                            }
                        };
                        let __field8 = match __field8 {
                            _serde::__private::Some(__field8) => __field8,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("signed16")?
                            }
                        };
                        let __field9 = match __field9 {
                            _serde::__private::Some(__field9) => __field9,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("signed32")?
                            }
                        };
                        let __field10 = match __field10 {
                            _serde::__private::Some(__field10) => __field10,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("signed64")?
                            }
                        };
                        let __field11 = match __field11 {
                            _serde::__private::Some(__field11) => __field11,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("signed128")?
                            }
                        };
                        let __field12 = match __field12 {
                            _serde::__private::Some(__field12) => __field12,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("unsignedsize")?
                            }
                        };
                        let __field13 = match __field13 {
                            _serde::__private::Some(__field13) => __field13,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("signedsize")?
                            }
                        };
                        let __field14 = match __field14 {
                            _serde::__private::Some(__field14) => __field14,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("float32")?
                            }
                        };
                        let __field15 = match __field15 {
                            _serde::__private::Some(__field15) => __field15,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("float64")?
                            }
                        };
                        _serde::__private::Ok(Primitives {
                            boolean: __field0,
                            character: __field1,
                            unsigned8: __field2,
                            unsigned16: __field3,
                            unsigned32: __field4,
                            unsigned64: __field5,
                            unsigned128: __field6,
                            signed8: __field7,
                            signed16: __field8,
                            signed32: __field9,
                            signed64: __field10,
                            signed128: __field11,
                            unsignedsize: __field12,
                            signedsize: __field13,
                            float32: __field14,
                            float64: __field15,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
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
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "Primitives",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<Primitives>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
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
    pub struct Allocated {
        string: String,
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
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for Allocated {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "Allocated",
                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "string",
                    &self.string,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "bytes",
                    &self.bytes,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "number_map",
                    &self.number_map,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "string_map",
                    &self.string_map,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "number_set",
                    &self.number_set,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "string_set",
                    &self.string_set,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "number_btree",
                    &self.number_btree,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "string_btree",
                    &self.string_btree,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "number_btree_set",
                    &self.number_btree_set,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "string_btree_set",
                    &self.string_btree_set,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "c_string",
                    &self.c_string,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for Allocated {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __field6,
                    __field7,
                    __field8,
                    __field9,
                    __field10,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            4u64 => _serde::__private::Ok(__Field::__field4),
                            5u64 => _serde::__private::Ok(__Field::__field5),
                            6u64 => _serde::__private::Ok(__Field::__field6),
                            7u64 => _serde::__private::Ok(__Field::__field7),
                            8u64 => _serde::__private::Ok(__Field::__field8),
                            9u64 => _serde::__private::Ok(__Field::__field9),
                            10u64 => _serde::__private::Ok(__Field::__field10),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "string" => _serde::__private::Ok(__Field::__field0),
                            "bytes" => _serde::__private::Ok(__Field::__field1),
                            "number_map" => _serde::__private::Ok(__Field::__field2),
                            "string_map" => _serde::__private::Ok(__Field::__field3),
                            "number_set" => _serde::__private::Ok(__Field::__field4),
                            "string_set" => _serde::__private::Ok(__Field::__field5),
                            "number_btree" => _serde::__private::Ok(__Field::__field6),
                            "string_btree" => _serde::__private::Ok(__Field::__field7),
                            "number_btree_set" => {
                                _serde::__private::Ok(__Field::__field8)
                            }
                            "string_btree_set" => {
                                _serde::__private::Ok(__Field::__field9)
                            }
                            "c_string" => _serde::__private::Ok(__Field::__field10),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"string" => _serde::__private::Ok(__Field::__field0),
                            b"bytes" => _serde::__private::Ok(__Field::__field1),
                            b"number_map" => _serde::__private::Ok(__Field::__field2),
                            b"string_map" => _serde::__private::Ok(__Field::__field3),
                            b"number_set" => _serde::__private::Ok(__Field::__field4),
                            b"string_set" => _serde::__private::Ok(__Field::__field5),
                            b"number_btree" => _serde::__private::Ok(__Field::__field6),
                            b"string_btree" => _serde::__private::Ok(__Field::__field7),
                            b"number_btree_set" => {
                                _serde::__private::Ok(__Field::__field8)
                            }
                            b"string_btree_set" => {
                                _serde::__private::Ok(__Field::__field9)
                            }
                            b"c_string" => _serde::__private::Ok(__Field::__field10),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<Allocated>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = Allocated;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct Allocated",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct Allocated with 11 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            Vec<u8>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct Allocated with 11 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            HashMap<u32, u64>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct Allocated with 11 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            HashMap<String, u64>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct Allocated with 11 elements",
                                    ),
                                );
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            HashSet<u32>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct Allocated with 11 elements",
                                    ),
                                );
                            }
                        };
                        let __field5 = match _serde::de::SeqAccess::next_element::<
                            HashSet<String>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        5usize,
                                        &"struct Allocated with 11 elements",
                                    ),
                                );
                            }
                        };
                        let __field6 = match _serde::de::SeqAccess::next_element::<
                            BTreeMap<u32, u64>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        6usize,
                                        &"struct Allocated with 11 elements",
                                    ),
                                );
                            }
                        };
                        let __field7 = match _serde::de::SeqAccess::next_element::<
                            BTreeMap<String, u64>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        7usize,
                                        &"struct Allocated with 11 elements",
                                    ),
                                );
                            }
                        };
                        let __field8 = match _serde::de::SeqAccess::next_element::<
                            BTreeSet<u32>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        8usize,
                                        &"struct Allocated with 11 elements",
                                    ),
                                );
                            }
                        };
                        let __field9 = match _serde::de::SeqAccess::next_element::<
                            BTreeSet<String>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        9usize,
                                        &"struct Allocated with 11 elements",
                                    ),
                                );
                            }
                        };
                        let __field10 = match _serde::de::SeqAccess::next_element::<
                            CString,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        10usize,
                                        &"struct Allocated with 11 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(Allocated {
                            string: __field0,
                            bytes: __field1,
                            number_map: __field2,
                            string_map: __field3,
                            number_set: __field4,
                            string_set: __field5,
                            number_btree: __field6,
                            string_btree: __field7,
                            number_btree_set: __field8,
                            string_btree_set: __field9,
                            c_string: __field10,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<Vec<u8>> = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<HashMap<u32, u64>> = _serde::__private::None;
                        let mut __field3: _serde::__private::Option<
                            HashMap<String, u64>,
                        > = _serde::__private::None;
                        let mut __field4: _serde::__private::Option<HashSet<u32>> = _serde::__private::None;
                        let mut __field5: _serde::__private::Option<HashSet<String>> = _serde::__private::None;
                        let mut __field6: _serde::__private::Option<
                            BTreeMap<u32, u64>,
                        > = _serde::__private::None;
                        let mut __field7: _serde::__private::Option<
                            BTreeMap<String, u64>,
                        > = _serde::__private::None;
                        let mut __field8: _serde::__private::Option<BTreeSet<u32>> = _serde::__private::None;
                        let mut __field9: _serde::__private::Option<BTreeSet<String>> = _serde::__private::None;
                        let mut __field10: _serde::__private::Option<CString> = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("string"),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("bytes"),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<Vec<u8>>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "number_map",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            HashMap<u32, u64>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private::Option::is_some(&__field3) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "string_map",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            HashMap<String, u64>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private::Option::is_some(&__field4) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "number_set",
                                            ),
                                        );
                                    }
                                    __field4 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            HashSet<u32>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private::Option::is_some(&__field5) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "string_set",
                                            ),
                                        );
                                    }
                                    __field5 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            HashSet<String>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field6 => {
                                    if _serde::__private::Option::is_some(&__field6) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "number_btree",
                                            ),
                                        );
                                    }
                                    __field6 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            BTreeMap<u32, u64>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field7 => {
                                    if _serde::__private::Option::is_some(&__field7) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "string_btree",
                                            ),
                                        );
                                    }
                                    __field7 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            BTreeMap<String, u64>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field8 => {
                                    if _serde::__private::Option::is_some(&__field8) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "number_btree_set",
                                            ),
                                        );
                                    }
                                    __field8 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            BTreeSet<u32>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field9 => {
                                    if _serde::__private::Option::is_some(&__field9) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "string_btree_set",
                                            ),
                                        );
                                    }
                                    __field9 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            BTreeSet<String>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field10 => {
                                    if _serde::__private::Option::is_some(&__field10) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "c_string",
                                            ),
                                        );
                                    }
                                    __field10 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<CString>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("string")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("bytes")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("number_map")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private::Some(__field3) => __field3,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("string_map")?
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private::Some(__field4) => __field4,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("number_set")?
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private::Some(__field5) => __field5,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("string_set")?
                            }
                        };
                        let __field6 = match __field6 {
                            _serde::__private::Some(__field6) => __field6,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("number_btree")?
                            }
                        };
                        let __field7 = match __field7 {
                            _serde::__private::Some(__field7) => __field7,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("string_btree")?
                            }
                        };
                        let __field8 = match __field8 {
                            _serde::__private::Some(__field8) => __field8,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("number_btree_set")?
                            }
                        };
                        let __field9 = match __field9 {
                            _serde::__private::Some(__field9) => __field9,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("string_btree_set")?
                            }
                        };
                        let __field10 = match __field10 {
                            _serde::__private::Some(__field10) => __field10,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("c_string")?
                            }
                        };
                        _serde::__private::Ok(Allocated {
                            string: __field0,
                            bytes: __field1,
                            number_map: __field2,
                            string_map: __field3,
                            number_set: __field4,
                            string_set: __field5,
                            number_btree: __field6,
                            string_btree: __field7,
                            number_btree_set: __field8,
                            string_btree_set: __field9,
                            c_string: __field10,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
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
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "Allocated",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<Allocated>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
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
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for Tuples {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "Tuples",
                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
                        + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "u0",
                    &self.u0,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "u1",
                    &self.u1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "u2",
                    &self.u2,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "u3",
                    &self.u3,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "u4",
                    &self.u4,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "u5",
                    &self.u5,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "u6",
                    &self.u6,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i0",
                    &self.i0,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i1",
                    &self.i1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i2",
                    &self.i2,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i3",
                    &self.i3,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i4",
                    &self.i4,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i5",
                    &self.i5,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "i6",
                    &self.i6,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for Tuples {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __field6,
                    __field7,
                    __field8,
                    __field9,
                    __field10,
                    __field11,
                    __field12,
                    __field13,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            4u64 => _serde::__private::Ok(__Field::__field4),
                            5u64 => _serde::__private::Ok(__Field::__field5),
                            6u64 => _serde::__private::Ok(__Field::__field6),
                            7u64 => _serde::__private::Ok(__Field::__field7),
                            8u64 => _serde::__private::Ok(__Field::__field8),
                            9u64 => _serde::__private::Ok(__Field::__field9),
                            10u64 => _serde::__private::Ok(__Field::__field10),
                            11u64 => _serde::__private::Ok(__Field::__field11),
                            12u64 => _serde::__private::Ok(__Field::__field12),
                            13u64 => _serde::__private::Ok(__Field::__field13),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "u0" => _serde::__private::Ok(__Field::__field0),
                            "u1" => _serde::__private::Ok(__Field::__field1),
                            "u2" => _serde::__private::Ok(__Field::__field2),
                            "u3" => _serde::__private::Ok(__Field::__field3),
                            "u4" => _serde::__private::Ok(__Field::__field4),
                            "u5" => _serde::__private::Ok(__Field::__field5),
                            "u6" => _serde::__private::Ok(__Field::__field6),
                            "i0" => _serde::__private::Ok(__Field::__field7),
                            "i1" => _serde::__private::Ok(__Field::__field8),
                            "i2" => _serde::__private::Ok(__Field::__field9),
                            "i3" => _serde::__private::Ok(__Field::__field10),
                            "i4" => _serde::__private::Ok(__Field::__field11),
                            "i5" => _serde::__private::Ok(__Field::__field12),
                            "i6" => _serde::__private::Ok(__Field::__field13),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"u0" => _serde::__private::Ok(__Field::__field0),
                            b"u1" => _serde::__private::Ok(__Field::__field1),
                            b"u2" => _serde::__private::Ok(__Field::__field2),
                            b"u3" => _serde::__private::Ok(__Field::__field3),
                            b"u4" => _serde::__private::Ok(__Field::__field4),
                            b"u5" => _serde::__private::Ok(__Field::__field5),
                            b"u6" => _serde::__private::Ok(__Field::__field6),
                            b"i0" => _serde::__private::Ok(__Field::__field7),
                            b"i1" => _serde::__private::Ok(__Field::__field8),
                            b"i2" => _serde::__private::Ok(__Field::__field9),
                            b"i3" => _serde::__private::Ok(__Field::__field10),
                            b"i4" => _serde::__private::Ok(__Field::__field11),
                            b"i5" => _serde::__private::Ok(__Field::__field12),
                            b"i6" => _serde::__private::Ok(__Field::__field13),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<Tuples>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = Tuples;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct Tuples",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            (),
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct Tuples with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            (bool,),
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct Tuples with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            (bool, u8),
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct Tuples with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            (bool, u8, u32),
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct Tuples with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            (bool, u8, u32, u64),
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct Tuples with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field5 = match _serde::de::SeqAccess::next_element::<
                            (bool, u8, u32, u64, f32),
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        5usize,
                                        &"struct Tuples with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field6 = match _serde::de::SeqAccess::next_element::<
                            (bool, u8, u32, u64, f32, f64),
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        6usize,
                                        &"struct Tuples with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field7 = match _serde::de::SeqAccess::next_element::<
                            (),
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        7usize,
                                        &"struct Tuples with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field8 = match _serde::de::SeqAccess::next_element::<
                            (bool,),
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        8usize,
                                        &"struct Tuples with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field9 = match _serde::de::SeqAccess::next_element::<
                            (bool, i8),
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        9usize,
                                        &"struct Tuples with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field10 = match _serde::de::SeqAccess::next_element::<
                            (bool, i8, i32),
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        10usize,
                                        &"struct Tuples with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field11 = match _serde::de::SeqAccess::next_element::<
                            (bool, i8, i32, i64),
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        11usize,
                                        &"struct Tuples with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field12 = match _serde::de::SeqAccess::next_element::<
                            (bool, i8, i32, i64, f32),
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        12usize,
                                        &"struct Tuples with 14 elements",
                                    ),
                                );
                            }
                        };
                        let __field13 = match _serde::de::SeqAccess::next_element::<
                            (bool, i8, i32, i64, f32, f64),
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        13usize,
                                        &"struct Tuples with 14 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(Tuples {
                            u0: __field0,
                            u1: __field1,
                            u2: __field2,
                            u3: __field3,
                            u4: __field4,
                            u5: __field5,
                            u6: __field6,
                            i0: __field7,
                            i1: __field8,
                            i2: __field9,
                            i3: __field10,
                            i4: __field11,
                            i5: __field12,
                            i6: __field13,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<()> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<(bool,)> = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<(bool, u8)> = _serde::__private::None;
                        let mut __field3: _serde::__private::Option<(bool, u8, u32)> = _serde::__private::None;
                        let mut __field4: _serde::__private::Option<
                            (bool, u8, u32, u64),
                        > = _serde::__private::None;
                        let mut __field5: _serde::__private::Option<
                            (bool, u8, u32, u64, f32),
                        > = _serde::__private::None;
                        let mut __field6: _serde::__private::Option<
                            (bool, u8, u32, u64, f32, f64),
                        > = _serde::__private::None;
                        let mut __field7: _serde::__private::Option<()> = _serde::__private::None;
                        let mut __field8: _serde::__private::Option<(bool,)> = _serde::__private::None;
                        let mut __field9: _serde::__private::Option<(bool, i8)> = _serde::__private::None;
                        let mut __field10: _serde::__private::Option<(bool, i8, i32)> = _serde::__private::None;
                        let mut __field11: _serde::__private::Option<
                            (bool, i8, i32, i64),
                        > = _serde::__private::None;
                        let mut __field12: _serde::__private::Option<
                            (bool, i8, i32, i64, f32),
                        > = _serde::__private::None;
                        let mut __field13: _serde::__private::Option<
                            (bool, i8, i32, i64, f32, f64),
                        > = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("u0"),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<()>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("u1"),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<(bool,)>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("u2"),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<(bool, u8)>(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private::Option::is_some(&__field3) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("u3"),
                                        );
                                    }
                                    __field3 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            (bool, u8, u32),
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private::Option::is_some(&__field4) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("u4"),
                                        );
                                    }
                                    __field4 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            (bool, u8, u32, u64),
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private::Option::is_some(&__field5) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("u5"),
                                        );
                                    }
                                    __field5 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            (bool, u8, u32, u64, f32),
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field6 => {
                                    if _serde::__private::Option::is_some(&__field6) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("u6"),
                                        );
                                    }
                                    __field6 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            (bool, u8, u32, u64, f32, f64),
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field7 => {
                                    if _serde::__private::Option::is_some(&__field7) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("i0"),
                                        );
                                    }
                                    __field7 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<()>(&mut __map)?,
                                    );
                                }
                                __Field::__field8 => {
                                    if _serde::__private::Option::is_some(&__field8) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("i1"),
                                        );
                                    }
                                    __field8 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<(bool,)>(&mut __map)?,
                                    );
                                }
                                __Field::__field9 => {
                                    if _serde::__private::Option::is_some(&__field9) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("i2"),
                                        );
                                    }
                                    __field9 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<(bool, i8)>(&mut __map)?,
                                    );
                                }
                                __Field::__field10 => {
                                    if _serde::__private::Option::is_some(&__field10) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("i3"),
                                        );
                                    }
                                    __field10 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            (bool, i8, i32),
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field11 => {
                                    if _serde::__private::Option::is_some(&__field11) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("i4"),
                                        );
                                    }
                                    __field11 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            (bool, i8, i32, i64),
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field12 => {
                                    if _serde::__private::Option::is_some(&__field12) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("i5"),
                                        );
                                    }
                                    __field12 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            (bool, i8, i32, i64, f32),
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field13 => {
                                    if _serde::__private::Option::is_some(&__field13) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("i6"),
                                        );
                                    }
                                    __field13 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            (bool, i8, i32, i64, f32, f64),
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("u0")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("u1")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("u2")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private::Some(__field3) => __field3,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("u3")?
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private::Some(__field4) => __field4,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("u4")?
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private::Some(__field5) => __field5,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("u5")?
                            }
                        };
                        let __field6 = match __field6 {
                            _serde::__private::Some(__field6) => __field6,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("u6")?
                            }
                        };
                        let __field7 = match __field7 {
                            _serde::__private::Some(__field7) => __field7,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("i0")?
                            }
                        };
                        let __field8 = match __field8 {
                            _serde::__private::Some(__field8) => __field8,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("i1")?
                            }
                        };
                        let __field9 = match __field9 {
                            _serde::__private::Some(__field9) => __field9,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("i2")?
                            }
                        };
                        let __field10 = match __field10 {
                            _serde::__private::Some(__field10) => __field10,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("i3")?
                            }
                        };
                        let __field11 = match __field11 {
                            _serde::__private::Some(__field11) => __field11,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("i4")?
                            }
                        };
                        let __field12 = match __field12 {
                            _serde::__private::Some(__field12) => __field12,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("i5")?
                            }
                        };
                        let __field13 = match __field13 {
                            _serde::__private::Some(__field13) => __field13,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("i6")?
                            }
                        };
                        _serde::__private::Ok(Tuples {
                            u0: __field0,
                            u1: __field1,
                            u2: __field2,
                            u3: __field3,
                            u4: __field4,
                            u5: __field5,
                            u6: __field6,
                            i0: __field7,
                            i1: __field8,
                            i2: __field9,
                            i3: __field10,
                            i4: __field11,
                            i5: __field12,
                            i6: __field13,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
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
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "Tuples",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<Tuples>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
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
    pub enum MediumEnum {
        #[cfg(not(feature = "no-empty"))]
        Empty,
        EmptyTuple(),
        #[cfg(not(feature = "no-newtype"))]
        NewType(u64),
        Tuple(u64, u64),
        #[cfg(not(feature = "no-newtype"))]
        NewTypeString(String),
        TupleString(String, Vec<u8>),
        Struct { a: u32, primitives: Primitives, b: u64 },
        EmptyStruct {},
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for MediumEnum {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    MediumEnum::Empty => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "MediumEnum",
                            0u32,
                            "Empty",
                        )
                    }
                    MediumEnum::EmptyTuple() => {
                        let __serde_state = _serde::Serializer::serialize_tuple_variant(
                            __serializer,
                            "MediumEnum",
                            1u32,
                            "EmptyTuple",
                            0,
                        )?;
                        _serde::ser::SerializeTupleVariant::end(__serde_state)
                    }
                    MediumEnum::NewType(ref __field0) => {
                        _serde::Serializer::serialize_newtype_variant(
                            __serializer,
                            "MediumEnum",
                            2u32,
                            "NewType",
                            __field0,
                        )
                    }
                    MediumEnum::Tuple(ref __field0, ref __field1) => {
                        let mut __serde_state = _serde::Serializer::serialize_tuple_variant(
                            __serializer,
                            "MediumEnum",
                            3u32,
                            "Tuple",
                            0 + 1 + 1,
                        )?;
                        _serde::ser::SerializeTupleVariant::serialize_field(
                            &mut __serde_state,
                            __field0,
                        )?;
                        _serde::ser::SerializeTupleVariant::serialize_field(
                            &mut __serde_state,
                            __field1,
                        )?;
                        _serde::ser::SerializeTupleVariant::end(__serde_state)
                    }
                    MediumEnum::NewTypeString(ref __field0) => {
                        _serde::Serializer::serialize_newtype_variant(
                            __serializer,
                            "MediumEnum",
                            4u32,
                            "NewTypeString",
                            __field0,
                        )
                    }
                    MediumEnum::TupleString(ref __field0, ref __field1) => {
                        let mut __serde_state = _serde::Serializer::serialize_tuple_variant(
                            __serializer,
                            "MediumEnum",
                            5u32,
                            "TupleString",
                            0 + 1 + 1,
                        )?;
                        _serde::ser::SerializeTupleVariant::serialize_field(
                            &mut __serde_state,
                            __field0,
                        )?;
                        _serde::ser::SerializeTupleVariant::serialize_field(
                            &mut __serde_state,
                            __field1,
                        )?;
                        _serde::ser::SerializeTupleVariant::end(__serde_state)
                    }
                    MediumEnum::Struct { ref a, ref primitives, ref b } => {
                        let mut __serde_state = _serde::Serializer::serialize_struct_variant(
                            __serializer,
                            "MediumEnum",
                            6u32,
                            "Struct",
                            0 + 1 + 1 + 1,
                        )?;
                        _serde::ser::SerializeStructVariant::serialize_field(
                            &mut __serde_state,
                            "a",
                            a,
                        )?;
                        _serde::ser::SerializeStructVariant::serialize_field(
                            &mut __serde_state,
                            "primitives",
                            primitives,
                        )?;
                        _serde::ser::SerializeStructVariant::serialize_field(
                            &mut __serde_state,
                            "b",
                            b,
                        )?;
                        _serde::ser::SerializeStructVariant::end(__serde_state)
                    }
                    MediumEnum::EmptyStruct {} => {
                        let __serde_state = _serde::Serializer::serialize_struct_variant(
                            __serializer,
                            "MediumEnum",
                            7u32,
                            "EmptyStruct",
                            0,
                        )?;
                        _serde::ser::SerializeStructVariant::end(__serde_state)
                    }
                }
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for MediumEnum {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __field6,
                    __field7,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "variant identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            4u64 => _serde::__private::Ok(__Field::__field4),
                            5u64 => _serde::__private::Ok(__Field::__field5),
                            6u64 => _serde::__private::Ok(__Field::__field6),
                            7u64 => _serde::__private::Ok(__Field::__field7),
                            _ => {
                                _serde::__private::Err(
                                    _serde::de::Error::invalid_value(
                                        _serde::de::Unexpected::Unsigned(__value),
                                        &"variant index 0 <= i < 8",
                                    ),
                                )
                            }
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "Empty" => _serde::__private::Ok(__Field::__field0),
                            "EmptyTuple" => _serde::__private::Ok(__Field::__field1),
                            "NewType" => _serde::__private::Ok(__Field::__field2),
                            "Tuple" => _serde::__private::Ok(__Field::__field3),
                            "NewTypeString" => _serde::__private::Ok(__Field::__field4),
                            "TupleString" => _serde::__private::Ok(__Field::__field5),
                            "Struct" => _serde::__private::Ok(__Field::__field6),
                            "EmptyStruct" => _serde::__private::Ok(__Field::__field7),
                            _ => {
                                _serde::__private::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"Empty" => _serde::__private::Ok(__Field::__field0),
                            b"EmptyTuple" => _serde::__private::Ok(__Field::__field1),
                            b"NewType" => _serde::__private::Ok(__Field::__field2),
                            b"Tuple" => _serde::__private::Ok(__Field::__field3),
                            b"NewTypeString" => _serde::__private::Ok(__Field::__field4),
                            b"TupleString" => _serde::__private::Ok(__Field::__field5),
                            b"Struct" => _serde::__private::Ok(__Field::__field6),
                            b"EmptyStruct" => _serde::__private::Ok(__Field::__field7),
                            _ => {
                                let __value = &_serde::__private::from_utf8_lossy(__value);
                                _serde::__private::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<MediumEnum>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = MediumEnum;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "enum MediumEnum",
                        )
                    }
                    fn visit_enum<__A>(
                        self,
                        __data: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::EnumAccess<'de>,
                    {
                        match _serde::de::EnumAccess::variant(__data)? {
                            (__Field::__field0, __variant) => {
                                _serde::de::VariantAccess::unit_variant(__variant)?;
                                _serde::__private::Ok(MediumEnum::Empty)
                            }
                            (__Field::__field1, __variant) => {
                                #[doc(hidden)]
                                struct __Visitor<'de> {
                                    marker: _serde::__private::PhantomData<MediumEnum>,
                                    lifetime: _serde::__private::PhantomData<&'de ()>,
                                }
                                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                                    type Value = MediumEnum;
                                    fn expecting(
                                        &self,
                                        __formatter: &mut _serde::__private::Formatter,
                                    ) -> _serde::__private::fmt::Result {
                                        _serde::__private::Formatter::write_str(
                                            __formatter,
                                            "tuple variant MediumEnum::EmptyTuple",
                                        )
                                    }
                                    #[inline]
                                    fn visit_seq<__A>(
                                        self,
                                        _: __A,
                                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                                    where
                                        __A: _serde::de::SeqAccess<'de>,
                                    {
                                        _serde::__private::Ok(MediumEnum::EmptyTuple())
                                    }
                                }
                                _serde::de::VariantAccess::tuple_variant(
                                    __variant,
                                    0usize,
                                    __Visitor {
                                        marker: _serde::__private::PhantomData::<MediumEnum>,
                                        lifetime: _serde::__private::PhantomData,
                                    },
                                )
                            }
                            (__Field::__field2, __variant) => {
                                _serde::__private::Result::map(
                                    _serde::de::VariantAccess::newtype_variant::<
                                        u64,
                                    >(__variant),
                                    MediumEnum::NewType,
                                )
                            }
                            (__Field::__field3, __variant) => {
                                #[doc(hidden)]
                                struct __Visitor<'de> {
                                    marker: _serde::__private::PhantomData<MediumEnum>,
                                    lifetime: _serde::__private::PhantomData<&'de ()>,
                                }
                                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                                    type Value = MediumEnum;
                                    fn expecting(
                                        &self,
                                        __formatter: &mut _serde::__private::Formatter,
                                    ) -> _serde::__private::fmt::Result {
                                        _serde::__private::Formatter::write_str(
                                            __formatter,
                                            "tuple variant MediumEnum::Tuple",
                                        )
                                    }
                                    #[inline]
                                    fn visit_seq<__A>(
                                        self,
                                        mut __seq: __A,
                                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                                    where
                                        __A: _serde::de::SeqAccess<'de>,
                                    {
                                        let __field0 = match _serde::de::SeqAccess::next_element::<
                                            u64,
                                        >(&mut __seq)? {
                                            _serde::__private::Some(__value) => __value,
                                            _serde::__private::None => {
                                                return _serde::__private::Err(
                                                    _serde::de::Error::invalid_length(
                                                        0usize,
                                                        &"tuple variant MediumEnum::Tuple with 2 elements",
                                                    ),
                                                );
                                            }
                                        };
                                        let __field1 = match _serde::de::SeqAccess::next_element::<
                                            u64,
                                        >(&mut __seq)? {
                                            _serde::__private::Some(__value) => __value,
                                            _serde::__private::None => {
                                                return _serde::__private::Err(
                                                    _serde::de::Error::invalid_length(
                                                        1usize,
                                                        &"tuple variant MediumEnum::Tuple with 2 elements",
                                                    ),
                                                );
                                            }
                                        };
                                        _serde::__private::Ok(MediumEnum::Tuple(__field0, __field1))
                                    }
                                }
                                _serde::de::VariantAccess::tuple_variant(
                                    __variant,
                                    2usize,
                                    __Visitor {
                                        marker: _serde::__private::PhantomData::<MediumEnum>,
                                        lifetime: _serde::__private::PhantomData,
                                    },
                                )
                            }
                            (__Field::__field4, __variant) => {
                                _serde::__private::Result::map(
                                    _serde::de::VariantAccess::newtype_variant::<
                                        String,
                                    >(__variant),
                                    MediumEnum::NewTypeString,
                                )
                            }
                            (__Field::__field5, __variant) => {
                                #[doc(hidden)]
                                struct __Visitor<'de> {
                                    marker: _serde::__private::PhantomData<MediumEnum>,
                                    lifetime: _serde::__private::PhantomData<&'de ()>,
                                }
                                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                                    type Value = MediumEnum;
                                    fn expecting(
                                        &self,
                                        __formatter: &mut _serde::__private::Formatter,
                                    ) -> _serde::__private::fmt::Result {
                                        _serde::__private::Formatter::write_str(
                                            __formatter,
                                            "tuple variant MediumEnum::TupleString",
                                        )
                                    }
                                    #[inline]
                                    fn visit_seq<__A>(
                                        self,
                                        mut __seq: __A,
                                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                                    where
                                        __A: _serde::de::SeqAccess<'de>,
                                    {
                                        let __field0 = match _serde::de::SeqAccess::next_element::<
                                            String,
                                        >(&mut __seq)? {
                                            _serde::__private::Some(__value) => __value,
                                            _serde::__private::None => {
                                                return _serde::__private::Err(
                                                    _serde::de::Error::invalid_length(
                                                        0usize,
                                                        &"tuple variant MediumEnum::TupleString with 2 elements",
                                                    ),
                                                );
                                            }
                                        };
                                        let __field1 = match _serde::de::SeqAccess::next_element::<
                                            Vec<u8>,
                                        >(&mut __seq)? {
                                            _serde::__private::Some(__value) => __value,
                                            _serde::__private::None => {
                                                return _serde::__private::Err(
                                                    _serde::de::Error::invalid_length(
                                                        1usize,
                                                        &"tuple variant MediumEnum::TupleString with 2 elements",
                                                    ),
                                                );
                                            }
                                        };
                                        _serde::__private::Ok(
                                            MediumEnum::TupleString(__field0, __field1),
                                        )
                                    }
                                }
                                _serde::de::VariantAccess::tuple_variant(
                                    __variant,
                                    2usize,
                                    __Visitor {
                                        marker: _serde::__private::PhantomData::<MediumEnum>,
                                        lifetime: _serde::__private::PhantomData,
                                    },
                                )
                            }
                            (__Field::__field6, __variant) => {
                                #[allow(non_camel_case_types)]
                                #[doc(hidden)]
                                enum __Field {
                                    __field0,
                                    __field1,
                                    __field2,
                                    __ignore,
                                }
                                #[doc(hidden)]
                                struct __FieldVisitor;
                                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                                    type Value = __Field;
                                    fn expecting(
                                        &self,
                                        __formatter: &mut _serde::__private::Formatter,
                                    ) -> _serde::__private::fmt::Result {
                                        _serde::__private::Formatter::write_str(
                                            __formatter,
                                            "field identifier",
                                        )
                                    }
                                    fn visit_u64<__E>(
                                        self,
                                        __value: u64,
                                    ) -> _serde::__private::Result<Self::Value, __E>
                                    where
                                        __E: _serde::de::Error,
                                    {
                                        match __value {
                                            0u64 => _serde::__private::Ok(__Field::__field0),
                                            1u64 => _serde::__private::Ok(__Field::__field1),
                                            2u64 => _serde::__private::Ok(__Field::__field2),
                                            _ => _serde::__private::Ok(__Field::__ignore),
                                        }
                                    }
                                    fn visit_str<__E>(
                                        self,
                                        __value: &str,
                                    ) -> _serde::__private::Result<Self::Value, __E>
                                    where
                                        __E: _serde::de::Error,
                                    {
                                        match __value {
                                            "a" => _serde::__private::Ok(__Field::__field0),
                                            "primitives" => _serde::__private::Ok(__Field::__field1),
                                            "b" => _serde::__private::Ok(__Field::__field2),
                                            _ => _serde::__private::Ok(__Field::__ignore),
                                        }
                                    }
                                    fn visit_bytes<__E>(
                                        self,
                                        __value: &[u8],
                                    ) -> _serde::__private::Result<Self::Value, __E>
                                    where
                                        __E: _serde::de::Error,
                                    {
                                        match __value {
                                            b"a" => _serde::__private::Ok(__Field::__field0),
                                            b"primitives" => _serde::__private::Ok(__Field::__field1),
                                            b"b" => _serde::__private::Ok(__Field::__field2),
                                            _ => _serde::__private::Ok(__Field::__ignore),
                                        }
                                    }
                                }
                                impl<'de> _serde::Deserialize<'de> for __Field {
                                    #[inline]
                                    fn deserialize<__D>(
                                        __deserializer: __D,
                                    ) -> _serde::__private::Result<Self, __D::Error>
                                    where
                                        __D: _serde::Deserializer<'de>,
                                    {
                                        _serde::Deserializer::deserialize_identifier(
                                            __deserializer,
                                            __FieldVisitor,
                                        )
                                    }
                                }
                                #[doc(hidden)]
                                struct __Visitor<'de> {
                                    marker: _serde::__private::PhantomData<MediumEnum>,
                                    lifetime: _serde::__private::PhantomData<&'de ()>,
                                }
                                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                                    type Value = MediumEnum;
                                    fn expecting(
                                        &self,
                                        __formatter: &mut _serde::__private::Formatter,
                                    ) -> _serde::__private::fmt::Result {
                                        _serde::__private::Formatter::write_str(
                                            __formatter,
                                            "struct variant MediumEnum::Struct",
                                        )
                                    }
                                    #[inline]
                                    fn visit_seq<__A>(
                                        self,
                                        mut __seq: __A,
                                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                                    where
                                        __A: _serde::de::SeqAccess<'de>,
                                    {
                                        let __field0 = match _serde::de::SeqAccess::next_element::<
                                            u32,
                                        >(&mut __seq)? {
                                            _serde::__private::Some(__value) => __value,
                                            _serde::__private::None => {
                                                return _serde::__private::Err(
                                                    _serde::de::Error::invalid_length(
                                                        0usize,
                                                        &"struct variant MediumEnum::Struct with 3 elements",
                                                    ),
                                                );
                                            }
                                        };
                                        let __field1 = match _serde::de::SeqAccess::next_element::<
                                            Primitives,
                                        >(&mut __seq)? {
                                            _serde::__private::Some(__value) => __value,
                                            _serde::__private::None => {
                                                return _serde::__private::Err(
                                                    _serde::de::Error::invalid_length(
                                                        1usize,
                                                        &"struct variant MediumEnum::Struct with 3 elements",
                                                    ),
                                                );
                                            }
                                        };
                                        let __field2 = match _serde::de::SeqAccess::next_element::<
                                            u64,
                                        >(&mut __seq)? {
                                            _serde::__private::Some(__value) => __value,
                                            _serde::__private::None => {
                                                return _serde::__private::Err(
                                                    _serde::de::Error::invalid_length(
                                                        2usize,
                                                        &"struct variant MediumEnum::Struct with 3 elements",
                                                    ),
                                                );
                                            }
                                        };
                                        _serde::__private::Ok(MediumEnum::Struct {
                                            a: __field0,
                                            primitives: __field1,
                                            b: __field2,
                                        })
                                    }
                                    #[inline]
                                    fn visit_map<__A>(
                                        self,
                                        mut __map: __A,
                                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                                    where
                                        __A: _serde::de::MapAccess<'de>,
                                    {
                                        let mut __field0: _serde::__private::Option<u32> = _serde::__private::None;
                                        let mut __field1: _serde::__private::Option<Primitives> = _serde::__private::None;
                                        let mut __field2: _serde::__private::Option<u64> = _serde::__private::None;
                                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                                            __Field,
                                        >(&mut __map)? {
                                            match __key {
                                                __Field::__field0 => {
                                                    if _serde::__private::Option::is_some(&__field0) {
                                                        return _serde::__private::Err(
                                                            <__A::Error as _serde::de::Error>::duplicate_field("a"),
                                                        );
                                                    }
                                                    __field0 = _serde::__private::Some(
                                                        _serde::de::MapAccess::next_value::<u32>(&mut __map)?,
                                                    );
                                                }
                                                __Field::__field1 => {
                                                    if _serde::__private::Option::is_some(&__field1) {
                                                        return _serde::__private::Err(
                                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                                "primitives",
                                                            ),
                                                        );
                                                    }
                                                    __field1 = _serde::__private::Some(
                                                        _serde::de::MapAccess::next_value::<Primitives>(&mut __map)?,
                                                    );
                                                }
                                                __Field::__field2 => {
                                                    if _serde::__private::Option::is_some(&__field2) {
                                                        return _serde::__private::Err(
                                                            <__A::Error as _serde::de::Error>::duplicate_field("b"),
                                                        );
                                                    }
                                                    __field2 = _serde::__private::Some(
                                                        _serde::de::MapAccess::next_value::<u64>(&mut __map)?,
                                                    );
                                                }
                                                _ => {
                                                    let _ = _serde::de::MapAccess::next_value::<
                                                        _serde::de::IgnoredAny,
                                                    >(&mut __map)?;
                                                }
                                            }
                                        }
                                        let __field0 = match __field0 {
                                            _serde::__private::Some(__field0) => __field0,
                                            _serde::__private::None => {
                                                _serde::__private::de::missing_field("a")?
                                            }
                                        };
                                        let __field1 = match __field1 {
                                            _serde::__private::Some(__field1) => __field1,
                                            _serde::__private::None => {
                                                _serde::__private::de::missing_field("primitives")?
                                            }
                                        };
                                        let __field2 = match __field2 {
                                            _serde::__private::Some(__field2) => __field2,
                                            _serde::__private::None => {
                                                _serde::__private::de::missing_field("b")?
                                            }
                                        };
                                        _serde::__private::Ok(MediumEnum::Struct {
                                            a: __field0,
                                            primitives: __field1,
                                            b: __field2,
                                        })
                                    }
                                }
                                #[doc(hidden)]
                                const FIELDS: &'static [&'static str] = &[
                                    "a",
                                    "primitives",
                                    "b",
                                ];
                                _serde::de::VariantAccess::struct_variant(
                                    __variant,
                                    FIELDS,
                                    __Visitor {
                                        marker: _serde::__private::PhantomData::<MediumEnum>,
                                        lifetime: _serde::__private::PhantomData,
                                    },
                                )
                            }
                            (__Field::__field7, __variant) => {
                                #[allow(non_camel_case_types)]
                                #[doc(hidden)]
                                enum __Field {
                                    __ignore,
                                }
                                #[doc(hidden)]
                                struct __FieldVisitor;
                                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                                    type Value = __Field;
                                    fn expecting(
                                        &self,
                                        __formatter: &mut _serde::__private::Formatter,
                                    ) -> _serde::__private::fmt::Result {
                                        _serde::__private::Formatter::write_str(
                                            __formatter,
                                            "field identifier",
                                        )
                                    }
                                    fn visit_u64<__E>(
                                        self,
                                        __value: u64,
                                    ) -> _serde::__private::Result<Self::Value, __E>
                                    where
                                        __E: _serde::de::Error,
                                    {
                                        match __value {
                                            _ => _serde::__private::Ok(__Field::__ignore),
                                        }
                                    }
                                    fn visit_str<__E>(
                                        self,
                                        __value: &str,
                                    ) -> _serde::__private::Result<Self::Value, __E>
                                    where
                                        __E: _serde::de::Error,
                                    {
                                        match __value {
                                            _ => _serde::__private::Ok(__Field::__ignore),
                                        }
                                    }
                                    fn visit_bytes<__E>(
                                        self,
                                        __value: &[u8],
                                    ) -> _serde::__private::Result<Self::Value, __E>
                                    where
                                        __E: _serde::de::Error,
                                    {
                                        match __value {
                                            _ => _serde::__private::Ok(__Field::__ignore),
                                        }
                                    }
                                }
                                impl<'de> _serde::Deserialize<'de> for __Field {
                                    #[inline]
                                    fn deserialize<__D>(
                                        __deserializer: __D,
                                    ) -> _serde::__private::Result<Self, __D::Error>
                                    where
                                        __D: _serde::Deserializer<'de>,
                                    {
                                        _serde::Deserializer::deserialize_identifier(
                                            __deserializer,
                                            __FieldVisitor,
                                        )
                                    }
                                }
                                #[doc(hidden)]
                                struct __Visitor<'de> {
                                    marker: _serde::__private::PhantomData<MediumEnum>,
                                    lifetime: _serde::__private::PhantomData<&'de ()>,
                                }
                                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                                    type Value = MediumEnum;
                                    fn expecting(
                                        &self,
                                        __formatter: &mut _serde::__private::Formatter,
                                    ) -> _serde::__private::fmt::Result {
                                        _serde::__private::Formatter::write_str(
                                            __formatter,
                                            "struct variant MediumEnum::EmptyStruct",
                                        )
                                    }
                                    #[inline]
                                    fn visit_seq<__A>(
                                        self,
                                        _: __A,
                                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                                    where
                                        __A: _serde::de::SeqAccess<'de>,
                                    {
                                        _serde::__private::Ok(MediumEnum::EmptyStruct {})
                                    }
                                    #[inline]
                                    fn visit_map<__A>(
                                        self,
                                        mut __map: __A,
                                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                                    where
                                        __A: _serde::de::MapAccess<'de>,
                                    {
                                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                                            __Field,
                                        >(&mut __map)? {
                                            match __key {
                                                _ => {
                                                    let _ = _serde::de::MapAccess::next_value::<
                                                        _serde::de::IgnoredAny,
                                                    >(&mut __map)?;
                                                }
                                            }
                                        }
                                        _serde::__private::Ok(MediumEnum::EmptyStruct {})
                                    }
                                }
                                #[doc(hidden)]
                                const FIELDS: &'static [&'static str] = &[];
                                _serde::de::VariantAccess::struct_variant(
                                    __variant,
                                    FIELDS,
                                    __Visitor {
                                        marker: _serde::__private::PhantomData::<MediumEnum>,
                                        lifetime: _serde::__private::PhantomData,
                                    },
                                )
                            }
                        }
                    }
                }
                #[doc(hidden)]
                const VARIANTS: &'static [&'static str] = &[
                    "Empty",
                    "EmptyTuple",
                    "NewType",
                    "Tuple",
                    "NewTypeString",
                    "TupleString",
                    "Struct",
                    "EmptyStruct",
                ];
                _serde::Deserializer::deserialize_enum(
                    __deserializer,
                    "MediumEnum",
                    VARIANTS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<MediumEnum>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
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
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for LargeStruct {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "LargeStruct",
                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "primitives",
                    &self.primitives,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "tuples",
                    &self.tuples,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "medium_vec",
                    &self.medium_vec,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "medium_map",
                    &self.medium_map,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "string_keys",
                    &self.string_keys,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "number_map",
                    &self.number_map,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "number_vec",
                    &self.number_vec,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for LargeStruct {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __field6,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            4u64 => _serde::__private::Ok(__Field::__field4),
                            5u64 => _serde::__private::Ok(__Field::__field5),
                            6u64 => _serde::__private::Ok(__Field::__field6),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "primitives" => _serde::__private::Ok(__Field::__field0),
                            "tuples" => _serde::__private::Ok(__Field::__field1),
                            "medium_vec" => _serde::__private::Ok(__Field::__field2),
                            "medium_map" => _serde::__private::Ok(__Field::__field3),
                            "string_keys" => _serde::__private::Ok(__Field::__field4),
                            "number_map" => _serde::__private::Ok(__Field::__field5),
                            "number_vec" => _serde::__private::Ok(__Field::__field6),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"primitives" => _serde::__private::Ok(__Field::__field0),
                            b"tuples" => _serde::__private::Ok(__Field::__field1),
                            b"medium_vec" => _serde::__private::Ok(__Field::__field2),
                            b"medium_map" => _serde::__private::Ok(__Field::__field3),
                            b"string_keys" => _serde::__private::Ok(__Field::__field4),
                            b"number_map" => _serde::__private::Ok(__Field::__field5),
                            b"number_vec" => _serde::__private::Ok(__Field::__field6),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<LargeStruct>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = LargeStruct;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct LargeStruct",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            Vec<Primitives>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct LargeStruct with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            Vec<(Tuples, Tuples)>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct LargeStruct with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            Vec<MediumEnum>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct LargeStruct with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            HashMap<String, MediumEnum>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct LargeStruct with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            HashMap<String, u64>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct LargeStruct with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field5 = match _serde::de::SeqAccess::next_element::<
                            HashMap<u32, u64>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        5usize,
                                        &"struct LargeStruct with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field6 = match _serde::de::SeqAccess::next_element::<
                            Vec<(u32, u64)>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        6usize,
                                        &"struct LargeStruct with 7 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(LargeStruct {
                            primitives: __field0,
                            tuples: __field1,
                            medium_vec: __field2,
                            medium_map: __field3,
                            string_keys: __field4,
                            number_map: __field5,
                            number_vec: __field6,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<Vec<Primitives>> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<
                            Vec<(Tuples, Tuples)>,
                        > = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<Vec<MediumEnum>> = _serde::__private::None;
                        let mut __field3: _serde::__private::Option<
                            HashMap<String, MediumEnum>,
                        > = _serde::__private::None;
                        let mut __field4: _serde::__private::Option<
                            HashMap<String, u64>,
                        > = _serde::__private::None;
                        let mut __field5: _serde::__private::Option<HashMap<u32, u64>> = _serde::__private::None;
                        let mut __field6: _serde::__private::Option<Vec<(u32, u64)>> = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "primitives",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Vec<Primitives>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("tuples"),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Vec<(Tuples, Tuples)>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "medium_vec",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Vec<MediumEnum>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private::Option::is_some(&__field3) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "medium_map",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            HashMap<String, MediumEnum>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private::Option::is_some(&__field4) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "string_keys",
                                            ),
                                        );
                                    }
                                    __field4 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            HashMap<String, u64>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private::Option::is_some(&__field5) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "number_map",
                                            ),
                                        );
                                    }
                                    __field5 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            HashMap<u32, u64>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field6 => {
                                    if _serde::__private::Option::is_some(&__field6) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "number_vec",
                                            ),
                                        );
                                    }
                                    __field6 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Vec<(u32, u64)>,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("primitives")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("tuples")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("medium_vec")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private::Some(__field3) => __field3,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("medium_map")?
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private::Some(__field4) => __field4,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("string_keys")?
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private::Some(__field5) => __field5,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("number_map")?
                            }
                        };
                        let __field6 = match __field6 {
                            _serde::__private::Some(__field6) => __field6,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("number_vec")?
                            }
                        };
                        _serde::__private::Ok(LargeStruct {
                            primitives: __field0,
                            tuples: __field1,
                            medium_vec: __field2,
                            medium_map: __field3,
                            string_keys: __field4,
                            number_map: __field5,
                            number_vec: __field6,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "primitives",
                    "tuples",
                    "medium_vec",
                    "medium_map",
                    "string_keys",
                    "number_map",
                    "number_vec",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "LargeStruct",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<LargeStruct>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
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
