#![no_std]

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
        #[cfg(feature = "bitcode")]
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
pub fn rng_with_seed(seed: u64) -> rand::prelude::StdRng {
    use rand::prelude::*;
    rand::prelude::StdRng::seed_from_u64(seed)
}

/// Build common RNG.
pub fn rng() -> rand::prelude::StdRng {
    rng_with_seed(RNG_SEED)
}
