mod allocated;
pub use self::allocated::Allocated;

mod primpacked;
pub use self::primpacked::PrimitivesPacked;

mod primitives;
pub use self::primitives::Primitives;

mod mesh;
pub use self::mesh::Mesh;

mod tuples;
pub use self::tuples::Tuples;

mod large;
pub use self::large::LargeStruct;

mod medium_enum;
#[cfg(any(not(feature = "no-empty"), not(feature = "no-nonunit-variant")))]
pub use self::medium_enum::MediumEnum;

use core::ops::Range;

options! {
    pub(crate) unsafe fn init_ranges();
    pub(crate) fn enumerate_ranges();
    static PRIMITIVES_RANGE: Range<usize> = 10..100, 1..3;
    static MEDIUM_RANGE: Range<usize> = 10..100, 1..3;
    static SMALL_FIELDS: Range<usize> = 1..3, 1..2;
}
