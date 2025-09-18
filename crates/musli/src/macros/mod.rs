//! Helper macros for use with Musli.

#[cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "value"
))]
mod internal;
#[cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "value"
))]
pub(crate) use self::internal::{bare_encoding, encoding_impls, implement_error};

#[cfg(all(
    feature = "test",
    any(
        feature = "storage",
        feature = "wire",
        feature = "descriptive",
        feature = "value"
    )
))]
mod test;
#[cfg(all(feature = "test", feature = "alloc"))]
pub use self::test::support;
#[cfg(feature = "test")]
pub use self::test::{
    __test_extra, __test_matrix, FormatBytes, assert_decode_eq, assert_roundtrip_borrowed_eq,
    assert_roundtrip_eq,
};
#[cfg(feature = "test")]
pub(crate) use self::test::{test_fns, test_include_if};
