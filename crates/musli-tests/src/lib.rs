pub mod models;
pub mod utils;

pub mod wire {
    pub use musli_wire::*;
}

pub mod storage {
    pub use musli_storage::*;
}

pub mod s {
    pub use musli_descriptive::*;
}

/// Roundtrip the given expression through all supported formats.
#[macro_export]
macro_rules! rt {
    ($($tt:tt)*) => {{
        let a = $crate::wire::rt!($($tt)*);
        let b = $crate::storage::rt!($($tt)*);
        let c = $crate::s::rt!($($tt)*);
        assert_eq!(a, b);
        assert_eq!(a, c);
        a
    }};
}
