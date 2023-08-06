// Map internals copied from rust-phf under the MIT license.
//
// See: https://github.com/rust-phf/rust-phf/tree/b7116ff519415d302c070aa313831cd473b1a911

#[cfg(test)]
mod tests;

mod generator;
mod hashing;
mod map;
#[cfg(feature = "build")]
mod map_builder;

pub use self::map::{Map, MapRef};
#[cfg(feature = "build")]
pub use self::map_builder::MapBuilder;
