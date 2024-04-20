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
pub mod descriptive {
    pub use musli_descriptive::*;
}

#[cfg(feature = "musli-json")]
pub mod json {
    pub use musli_json::*;
}

#[cfg(feature = "musli-value")]
#[track_caller]
pub fn musli_value_rt<T>(expected: T)
where
    T: musli::Encode<musli::mode::Binary> + for<'de> musli::Decode<'de, musli::mode::Binary>,
    T: PartialEq + core::fmt::Debug,
{
    let value: ::musli_value::Value =
        ::musli_value::encode(&expected).expect("value: Encoding should succeed");
    let actual: T = ::musli_value::decode(&value).expect("value: Decoding should succeed");
    assert_eq!(
        actual, expected,
        "value: roundtripped value does not match expected"
    );
}

/// Roundtrip the given expression through all supported formats.
#[macro_export]
#[doc(hidden)]
macro_rules! rt {
    ($what:ident, $expr:expr $(, $($extra:tt)*)?) => {{
        let expected = $expr;

        macro_rules! rt {
            ($name:ident) => {{
                assert_eq!(
                    ::$name::test::rt($expr), expected,
                    "{}: roundtripped value does not match expected",
                    stringify!($name),
                );
            }}
        }

        $crate::$what!(rt);

        #[cfg(feature = "musli-value")]
        {
            $crate::musli_value_rt($expr);
        }

        $crate::extra!($expr $(, $($extra)*)*);
        expected
    }};
}

/// This is used to test when there is a decode assymmetry, such as the decoded
/// value does not match the encoded one due to things such as skipped fields.
#[macro_export]
#[doc(hidden)]
macro_rules! assert_decode_eq {
    ($what:ident, $expr:expr, $expected:expr $(, $($extra:tt)*)?) => {{
        let mut bytes = Vec::new();

        macro_rules! decode {
            ($name:ident) => {{
                ::$name::test::decode($expr, &mut bytes, &$expected);
            }}
        }

        $crate::$what!(decode);
        $crate::extra!($expr $(, $($extra)*)*);
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! extra {
    ($expr:expr $(,)?) => {};

    ($expr:expr, json = $json_expected:expr $(, $($tt:tt)*)?) => {{
        #[cfg(feature = "musli-json")]
        {
            let json = ::musli_json::test::to_vec($expr);
            let string = ::std::string::String::from_utf8(json).expect("Encoded JSON is not valid utf-8");

            assert_eq!(
                string, $json_expected,
                "json: encoded json does not match expected value"
            );
        }

        $crate::extra!($expr $(, $($tt)*)*);
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! descriptive {
    ($call:path) => {
        #[cfg(feature = "musli-descriptive")]
        $call!(musli_descriptive);
        #[cfg(feature = "musli-json")]
        $call!(musli_json);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! upgrade_stable {
    ($call:path) => {
        #[cfg(feature = "musli-wire")]
        $call!(musli_wire);
        #[cfg(feature = "musli-descriptive")]
        $call!(musli_descriptive);
        #[cfg(feature = "musli-json")]
        $call!(musli_json);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! upgrade_stable_no_text {
    ($call:path) => {
        #[cfg(feature = "musli-wire")]
        $call!(musli_wire);
        #[cfg(feature = "musli-descriptive")]
        $call!(musli_descriptive);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! full {
    ($call:path) => {
        #[cfg(feature = "musli-storage")]
        $call!(musli_storage);
        #[cfg(feature = "musli-wire")]
        $call!(musli_wire);
        #[cfg(feature = "musli-descriptive")]
        $call!(musli_descriptive);
        #[cfg(feature = "musli-json")]
        $call!(musli_json);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! no_json {
    ($call:path) => {
        #[cfg(feature = "musli-storage")]
        $call!(musli_storage);
        #[cfg(feature = "musli-wire")]
        $call!(musli_wire);
        #[cfg(feature = "musli-descriptive")]
        $call!(musli_descriptive);
    };
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
        #[cfg(feature = "miniserde")]
        $call!(miniserde $(, $($tt)*)*);
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
