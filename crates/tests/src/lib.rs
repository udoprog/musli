//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/tests.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/tests)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-tests-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/tests)
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg_attr(feature = "alloc", path = "alloc.rs")]
#[cfg_attr(not(feature = "alloc"), path = "no_alloc.rs")]
mod no_alloc;

/// Default random seed to use.
pub const RNG_SEED: u64 = 2718281828459045235;

pub use musli_macros::benchmarker;

#[macro_export]
macro_rules! miri {
    ($($(#[$($meta:meta)*])* $vis:vis const $ident:ident: $value_ty:ty = $range:expr, $miri:expr;)*) => {
        $(
            $(#[$($meta)*])*
            #[cfg(miri)]
            $vis const $ident: $value_ty = $miri;
            $(#[$($meta)*])*
            #[cfg(not(miri))]
            $vis const $ident: $value_ty = $range;
        )*
    }
}

pub mod generate;
mod mode;
pub mod models;
pub mod utils;

pub use self::aligned_buf::AlignedBuf;
mod aligned_buf;

#[cfg(feature = "musli-storage")]
pub mod storage {
    pub use musli_storage::*;
}

#[cfg(feature = "musli-wire")]
pub mod wire {
    pub use musli_wire::*;
}

#[cfg(feature = "musli-descriptive")]
pub mod desc {
    pub use musli_descriptive::*;
}

#[cfg(feature = "musli-json")]
pub mod json {
    pub use musli_json::*;
}

/// Roundtrip self-descriptive formats.
#[macro_export]
#[doc(hidden)]
macro_rules! rt_self {
    ($expr:expr $(, json = $json_expected:expr)?) => {{
        let expected = $expr;

        #[cfg(feature = "musli-descriptive")]
        {
            let descriptive = ::musli_descriptive::test::rt($expr);
            assert_eq!(expected, descriptive);
        }

        #[cfg(feature = "musli-json")]
        {
            let json = ::musli_json::test::rt($expr);
            assert_eq!(expected, json);
        }

        #[cfg(feature = "musli-json")]
        {
            let json = ::musli_json::test::to_vec($expr);
            let string = ::std::string::String::from_utf8(json).expect("encoded JSON is not valid utf-8");

            $(
                assert_eq!(
                    string, $json_expected,
                    "json: encoded json does not match expected value"
                );
            )*
        }

        expected
    }};
}

/// Roundtrip the given expression through all supported formats.
#[macro_export]
#[doc(hidden)]
macro_rules! rt {
    ($expr:expr $(, json = $json_expected:expr)?) => {{
        let expected = $crate::rt_no_json!($expr);

        #[cfg(feature = "musli-json")]
        {
            let json = ::musli_json::test::rt($expr);
            assert_eq!(expected, json);
        }

        #[cfg(feature = "musli-json")]
        {
            let json = ::musli_json::test::to_vec($expr);
            let string = ::std::string::String::from_utf8(json).expect("Encoded JSON is not valid utf-8");

            $(
                assert_eq!(
                    string, $json_expected,
                    "json: encoded json does not match expected value"
                );
            )*
        }

        expected
    }};
}

/// Roundtrip the given expression through all supported formats except JSON.
#[macro_export]
#[doc(hidden)]
macro_rules! rt_no_json {
    ($expr:expr) => {{
        let expected = $expr;

        #[cfg(feature = "musli-storage")]
        {
            let storage = ::musli_storage::test::rt($expr);
            assert_eq!(expected, storage);
        }

        #[cfg(feature = "musli-wire")]
        {
            let wire = ::musli_wire::test::rt($expr);
            assert_eq!(expected, wire);
        }

        #[cfg(feature = "musli-descriptive")]
        {
            let descriptive = ::musli_descriptive::test::rt($expr);
            assert_eq!(expected, descriptive);
        }

        expected
    }};
}

/// This is used to test when there is a decode assymmetry, such as the decoded
/// value does not match the encoded one due to things such as skipped fields.
#[macro_export]
#[doc(hidden)]
macro_rules! assert_decode_eq {
    ($expr:expr, $expected:expr) => {{
        #[cfg(feature = "musli-storage")]
        {
            let storage = ::musli_storage::test::decode($expr);
            assert_eq!(
                storage, $expected,
                "storage: decoded value does not match expected"
            );
        }

        #[cfg(feature = "musli-wire")]
        {
            let wire = ::musli_wire::test::decode($expr);
            assert_eq!(
                wire, $expected,
                "wire: decoded value does not match expected"
            );
        }

        #[cfg(feature = "musli-descriptive")]
        {
            let descriptive = ::musli_descriptive::test::decode($expr);
            assert_eq!(
                descriptive, $expected,
                "descriptive: decoded value does not match expected"
            );
        }

        #[cfg(feature = "musli-json")]
        {
            let json = ::musli_json::test::decode($expr);
            assert_eq!(
                json, $expected,
                "json: decoded value does not match expected"
            );
        }
    }};
}

/// Call the given macro with the existing feature matrix.
#[macro_export]
macro_rules! feature_matrix {
    ($call:path $(, $($tt:tt)*)?) => {
        #[cfg(feature = "serde_json")]
        $call!(serde_json $(, $($tt)*)*);
        #[cfg(feature = "bincode")]
        $call!(serde_bincode $(, $($tt)*)*);
        #[cfg(feature = "rmp-serde")]
        $call!(serde_rmp $(, $($tt)*)*);
        #[cfg(feature = "musli-json")]
        $call!(musli_json $(, $($tt)*)*);
        #[cfg(feature = "musli-wire")]
        $call!(musli_wire $(, $($tt)*)*);
        #[cfg(feature = "musli-descriptive")]
        $call!(musli_descriptive $(, $($tt)*)*);
        #[cfg(feature = "musli-storage")]
        $call!(musli_storage $(, $($tt)*)*);
        #[cfg(feature = "musli-storage")]
        $call!(musli_storage_packed $(, $($tt)*)*);
        #[cfg(feature = "musli-value")]
        $call!(musli_value $(, $($tt)*)*);
        #[cfg(feature = "musli-zerocopy")]
        $call!(musli_zerocopy $(, $($tt)*)*);
        #[cfg(feature = "zerocopy")]
        $call!(zerocopy $(, $($tt)*)*);
        #[cfg(feature = "dlhn")]
        $call!(serde_dlhn $(, $($tt)*)*);
        #[cfg(feature = "serde_cbor")]
        $call!(serde_cbor $(, $($tt)*)*);
        #[cfg(all(feature = "bitcode", feature = "serde"))]
        $call!(serde_bitcode $(, $($tt)*)*);
        #[cfg(feature = "bitcode-derive")]
        $call!(derive_bitcode $(, $($tt)*)*);
        #[cfg(feature = "rkyv")]
        $call!(rkyv $(, $($tt)*)*);
        #[cfg(feature = "postcard")]
        $call!(postcard $(, $($tt)*)*);
        #[cfg(feature = "bson")]
        $call!(bson $(, $($tt)*)*);
    };
}

/// Only expand `$block` if the given test is supported by this framework.
#[macro_export]
macro_rules! if_supported {
    (musli_zerocopy, large, $($tt:tt)*) => {};
    (musli_zerocopy, allocated, $($tt:tt)*) => {};
    (musli_zerocopy, medium_enum, $($tt:tt)*) => {};

    (zerocopy, primitives, $($tt:tt)*) => {};
    (zerocopy, large, $($tt:tt)*) => {};
    (zerocopy, allocated, $($tt:tt)*) => {};
    (zerocopy, medium_enum, $($tt:tt)*) => {};

    ($framework:ident, $test:ident, $($tt:tt)*) => { $($tt)* };
}

#[macro_export]
macro_rules! types {
    ($call:path $(, $($tt:tt)*)?) => {
        $call! {
            $($($tt)*,)?
            primitives, Primitives, PRIMITIVES, 1000,
            primpacked, PrimitivesPacked, PRIMITIVES_PACKED, 1000,
            large, LargeStruct, LARGE_STRUCTS, 10000,
            allocated, Allocated, ALLOCATED, 5000,
            medium_enum, MediumEnum, MEDIUM_ENUMS, 1000
        }
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
