pub mod wire {
    pub use musli_wire::*;
}

pub mod storage {
    pub use musli_storage::*;
}

/// Roundtrip the given expression through all supported formats.
#[macro_export]
macro_rules! rt {
    ($($tt:tt)*) => {{
        let a = $crate::wire::rt!($($tt)*);
        let b = $crate::storage::rt!($($tt)*);
        assert_eq!(a, b);
        a
    }};
}
