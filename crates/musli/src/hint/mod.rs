//! Types used to provide encoding hints to methods which needs it.

mod map_hint;
pub use self::map_hint::MapHint;

mod unsized_map_hint;
pub use self::unsized_map_hint::UnsizedMapHint;

mod sequence_hint;
pub use self::sequence_hint::SequenceHint;
