//! Types used to provide encoding hints to methods which needs it.

mod map_hint;
pub use self::map_hint::MapHint;

mod struct_hint;
pub use self::struct_hint::StructHint;

mod tuple_hint;
pub use self::tuple_hint::TupleHint;

mod sequence_hint;
pub use self::sequence_hint::SequenceHint;

mod unsized_struct_hint;
pub use self::unsized_struct_hint::UnsizedStructHint;
