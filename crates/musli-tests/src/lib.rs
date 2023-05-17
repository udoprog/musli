#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[macro_export]
macro_rules! miri {
    ($($(#[$($meta:meta)*])* const $ident:ident: $value_ty:ty = $range:expr, $miri:expr;)*) => {
        $(
            $(#[$($meta)*])*
            #[cfg(miri)]
            const $ident: $value_ty = $miri;
            $(#[$($meta)*])*
            #[cfg(not(miri))]
            const $ident: $value_ty = $range;
        )*
    }
}

// Defines denies feature combinations.
//
// * Negative features are not supported in cargo, and feature blocking
//   everything is too complex. model_map for example also depends on std.
// * Benchmarks for these must be explicitly run, because they only include a
//   subset of available data, we wouldn't be doing an apples-to-apples
//   comparison if we allowed only a model subset to be compared against a
//   serialization which supports a superset. If you do want to make this
//   comparison, you can enable `model_minimal`.
macro_rules! deny {
    ($base:literal $(, $feature:literal)*) => {
        $(
            #[cfg(all(feature = $base, feature = $feature))]
            compile_error!(concat!($base, ": does not support feature: ", $feature));
        )*
    }
}

deny!("rkyv", "model_tuple", "model_map_string_key", "model_usize");
deny!("dlhn", "model_map", "model_128");
deny!("bitcode", "model_128");

#[cfg(feature = "musli")]
mod mode;
pub mod models;
pub mod utils;

#[cfg(feature = "musli-wire")]
pub mod wire {
    pub use musli_wire::*;
}

#[cfg(feature = "musli-storage")]
pub mod storage {
    pub use musli_storage::*;
}

#[cfg(feature = "musli-descriptive")]
pub mod s {
    pub use musli_descriptive::*;
}

/// Roundtrip the given expression through all supported formats.
#[cfg(all(
    feature = "musli-wire",
    feature = "musli-storage",
    feature = "musli-descriptive"
))]
#[macro_export]
macro_rules! rt {
    ($($tt:tt)*) => {{
        let a = ::musli_wire::rt!($($tt)*);
        let b = ::musli_storage::rt!($($tt)*);
        let c = ::musli_descriptive::rt!($($tt)*);
        assert_eq!(a, b);
        assert_eq!(a, c);
        a
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
        #[cfg(all(feature = "dlhn", not(any(model_128, model_all))))]
        $call!(serde_dlhn $(, $($tt)*)*);
        #[cfg(feature = "serde_cbor")]
        $call!(serde_cbor $(, $($tt)*)*);
        #[cfg(feature = "bitcode")]
        $call!(serde_bitcode $(, $($tt)*)*);
        #[cfg(feature = "bitcode")]
        $call!(derive_bitcode $(, $($tt)*)*);
        #[cfg(feature = "rkyv")]
        $call!(rkyv $(, $($tt)*)*);
    };
}

#[macro_export]
macro_rules! types {
    ($call:path $(, $($tt:tt)*)?) => {
        $call!($($($tt)*,)? prim, Primitives, PRIMITIVES, lg, LargeStruct, LARGE_STRUCTS, allocated, Allocated, ALLOCATED, medium_enum, MediumEnum, MEDIUM_ENUMS);
    }
}

const SEED: u64 = 2718281828459045235;

pub fn rng() -> rand::prelude::StdRng {
    use rand::prelude::*;
    rand::prelude::StdRng::seed_from_u64(SEED)
}
