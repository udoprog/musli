#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

/// Default random seed to use.
pub const RNG_SEED: u64 = 2718281828459045235;

pub use tests_macros::benchmarker;

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
#[doc(inline)]
pub use self::generate::{Generate, Rng};
#[cfg(feature = "musli")]
mod mode;
#[cfg(feature = "musli")]
pub use self::mode::Packed;
pub mod models;
pub mod utils;

pub use self::aligned_buf::AlignedBuf;
mod aligned_buf;

/// Call the given macro with the existing feature matrix.
#[macro_export]
macro_rules! feature_matrix {
    ($call:path $(, $($tt:tt)*)?) => {
        $call!(mock $(, $($tt)*)*);
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
        #[cfg(feature = "serde_json")]
        $call!(serde_json $(, $($tt)*)*);
        #[cfg(feature = "bincode")]
        $call!(serde_bincode $(, $($tt)*)*);
        #[cfg(feature = "rmp-serde")]
        $call!(serde_rmp $(, $($tt)*)*);
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
        #[cfg(feature = "miniserde")]
        $call!(miniserde $(, $($tt)*)*);
        #[cfg(feature = "speedy")]
        $call!(speedy $(, $($tt)*)*);
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
    ($call:path) => {
        $call!(primitives, Primitives, PRIMITIVES, 1000);
        $call!(primpacked, PrimitivesPacked, PRIMITIVES_PACKED, 1000);
        $call!(large, LargeStruct, LARGE_STRUCTS, 10000);
        $call!(allocated, Allocated, ALLOCATED, 5000);
        #[cfg(any(not(feature = "no-empty"), not(feature = "no-nonunit-variant")))]
        $call!(medium_enum, MediumEnum, MEDIUM_ENUMS, 1000);
    };
}

/// Build common RNG with custom seed.
pub fn rng_with_seed(seed: u64) -> generate::Rng {
    generate::Rng::from_seed(seed)
}

/// Build common RNG.
pub fn rng() -> generate::Rng {
    rng_with_seed(RNG_SEED)
}
