#[cfg(feature = "alloc")]
mod allocated;
#[cfg(feature = "alloc")]
pub use self::allocated::Allocated;

mod packed;
pub use self::packed::Packed;

mod primitives;
pub use self::primitives::Primitives;

#[cfg(feature = "alloc")]
mod mesh;
#[cfg(feature = "alloc")]
pub use self::mesh::Mesh;

mod tuples;
pub use self::tuples::Tuples;

#[cfg(feature = "alloc")]
mod large;
#[cfg(feature = "alloc")]
pub use self::large::Large;

mod full_enum;
#[cfg(any(not(feature = "no-empty"), not(feature = "no-nonunit-variant")))]
pub use self::full_enum::FullEnum;

use core::ops::Range;

options! {
    pub(crate) unsafe fn init_ranges();
    pub(crate) fn enumerate_ranges();
    static PRIMITIVES_RANGE: Range<usize> = 10..100, 1..3;
    static MEDIUM_RANGE: Range<usize> = 10..100, 1..3;
    static SMALL_FIELDS: Range<usize> = 1..3, 1..2;
}
