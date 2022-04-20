pub mod wire {
    pub use musli_wire::*;
}

pub mod storage {
    pub use musli_storage::*;
}

/// Roundtrip the given expression through all supported formats.
#[macro_export]
macro_rules! rt {
    ($expr:expr) => {{
        let a = $crate::wire::rt!($expr);
        let b = $crate::storage::rt!($expr);
        assert_eq!(a, b);
        a
    }};
}
