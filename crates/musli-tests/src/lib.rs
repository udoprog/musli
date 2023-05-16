#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

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
    ($call:path) => {
        #[cfg(feature = "serde_json")]
        $call!(serde_json);
        #[cfg(feature = "bincode")]
        $call!(serde_bincode);
        #[cfg(feature = "rmp-serde")]
        $call!(serde_rmp);
        #[cfg(feature = "musli-json")]
        $call!(musli_json);
        #[cfg(feature = "musli-wire")]
        $call!(musli_wire);
        #[cfg(feature = "musli-descriptive")]
        $call!(musli_descriptive);
        #[cfg(feature = "musli-storage")]
        $call!(musli_storage);
        #[cfg(feature = "musli-storage")]
        $call!(musli_storage_packed);
        #[cfg(feature = "musli-value")]
        $call!(musli_value);
        #[cfg(all(feature = "dlhn", not(any(model_128, model_all))))]
        $call!(serde_dlhn);
        #[cfg(feature = "serde_cbor")]
        $call!(serde_cbor);
        #[cfg(feature = "bitcode")]
        $call!(serde_bitcode);
        #[cfg(feature = "bitcode")]
        $call!(derive_bitcode);
    };
}
