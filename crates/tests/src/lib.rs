//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/tests.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/tests)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-tests-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/tests)
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

/// Default random seed to use.
pub const RNG_SEED: u64 = 2718281828459045235;

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

/// Roundtrip the given expression through all supported formats.
#[macro_export]
macro_rules! rt {
    ($enum:ident :: $variant:ident $($tt:tt)?) => {
        $crate::rt!($enum, $enum :: $variant $($tt)*)
    };

    ($struct:ident $($tt:tt)?) => {
        $crate::rt!($struct, $struct $($tt)*)
    };

    ($ty:ty, $expr:expr) => {{
        let expected = $expr;

        #[cfg(feature = "musli-storage")]
        {
            let storage = ::musli_storage::rt!($ty, $expr);
            assert_eq!(expected, storage);
        }

        #[cfg(feature = "musli-wire")]
        {
            let wire = ::musli_wire::rt!($ty, $expr);
            assert_eq!(expected, wire);
        }

        #[cfg(feature = "musli-descriptive")]
        {
            let descriptive = ::musli_descriptive::rt!($ty, $expr);
            assert_eq!(expected, descriptive);
        }

        expected
    }};
}

#[macro_export]
macro_rules! musli_zerocopy_call {
    ($call:path) => {
    };

    ($call:path, primitives, $ty:ty, $size_hint:expr $(, $tt:tt)*) => {
        $call!(musli_value, musli_value_buf, primitives, $ty, $size_hint);
        $crate::musli_zerocopy_call!($call $(, $tt)*);
    };

    // Ignore others.
    ($call:path, $name:ident, $ty:ty, $size_hint:expr $(, $tt:tt)*) => {
        $crate::musli_zerocopy_call!($call $(, $tt)*);
    };
}

/// Call the given macro with the existing feature matrix.
#[macro_export]
macro_rules! feature_matrix {
    ($call:path $(, $($tt:tt)*)?) => {
        #[cfg(feature = "serde_json")]
        $call!(serde_json, serde_json_buf $(, $($tt)*)*);
        #[cfg(feature = "bincode")]
        $call!(serde_bincode, serde_bincode_buf $(, $($tt)*)*);
        #[cfg(feature = "rmp-serde")]
        $call!(serde_rmp, serde_rmp_buf $(, $($tt)*)*);
        #[cfg(feature = "musli-json")]
        $call!(musli_json, musli_json_buf $(, $($tt)*)*);
        #[cfg(feature = "musli-wire")]
        $call!(musli_wire, musli_wire_buf $(, $($tt)*)*);
        #[cfg(feature = "musli-descriptive")]
        $call!(musli_descriptive, musli_descriptive_buf $(, $($tt)*)*);
        #[cfg(feature = "musli-storage")]
        $call!(musli_storage, musli_storage_buf $(, $($tt)*)*);
        #[cfg(feature = "musli-storage")]
        $call!(musli_storage_packed, musli_storage_packed_buf $(, $($tt)*)*);
        #[cfg(feature = "musli-value")]
        $call!(musli_value, musli_value_buf $(, $($tt)*)*);
        #[cfg(feature = "musli-zerocopy")]
        $call!(musli_zerocopy, musli_zerocopy_buf $(, $($tt)*)*);
        #[cfg(feature = "zerocopy")]
        $call!(zerocopy, zerocopy_buf $(, $($tt)*)*);
        #[cfg(feature = "dlhn")]
        $call!(serde_dlhn, serde_dlhn_buf $(, $($tt)*)*);
        #[cfg(feature = "serde_cbor")]
        $call!(serde_cbor, serde_cbor_buf $(, $($tt)*)*);
        #[cfg(all(feature = "bitcode", feature = "serde"))]
        $call!(serde_bitcode, serde_bitcode_buf $(, $($tt)*)*);
        #[cfg(feature = "bitcode-derive")]
        $call!(derive_bitcode, derive_bitcode_buf $(, $($tt)*)*);
        #[cfg(feature = "rkyv")]
        $call!(rkyv, rkyv_buf $(, $($tt)*)*);
        #[cfg(feature = "postcard")]
        $call!(postcard, postcard_buf $(, $($tt)*)*);
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
