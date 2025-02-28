use crate::Context;

use super::MapBuilder;

/// Decode something explicitly as a map.
pub trait DecodeMap<'de, M, C>
where
    M: 'static,
    C: Context,
{
    /// The builder.
    type MapBuilder: MapBuilder<'de, M, C, Output = Self>;

    /// Construct a decoder into a map.
    fn new_map_builder() -> Self::MapBuilder;
}
