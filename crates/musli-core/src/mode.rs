//! Type that describe a mode of encoding.

/// The binary encoding mode.
///
/// The key of fields and variants are encoded by their index, as if
/// `#[musli(name_type = usize)]` was specified.
///
/// See [modes] for more.
///
/// [modes]: https://docs.rs/musli/latest/musli/_help/derives/index.html#modes
pub enum Binary {}

/// The text encoding mode.
///
/// The key of fields and variants are encoded by their name, as if
/// `#[musli(name_type = str)]` was specified.
///
/// See [modes] for more.
///
/// [modes]: https://docs.rs/musli/latest/musli/_help/derives/index.html#modes
pub enum Text {}
