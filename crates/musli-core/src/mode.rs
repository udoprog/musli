//! Type that describe a mode of encoding.

/// The binary encoding mode.
///
/// The key of fields and variants are encoded by their index, as if
/// `#[musli(name_type = usize)]` was specified.
///
/// See [modes] for more.
///
/// [modes]: https://docs.rs/musli/latest/musli/help/derives/index.html#modes
pub enum Binary {}

/// The text encoding mode.
///
/// The key of fields and variants are encoded by their name, as if
/// `#[musli(name_type = str)]` was specified.
///
/// See [modes] for more.
///
/// [modes]: https://docs.rs/musli/latest/musli/help/derives/index.html#modes
pub enum Text {}

/// Trait implemented for Modes to indicate human-readable formats.
pub trait Mode {
    /// Return whether the Mode is human-readable or not
    fn is_human_readable() -> bool;
}

impl Mode for Binary {
    fn is_human_readable() -> bool {
        false
    }
}

impl Mode for Text {
    fn is_human_readable() -> bool {
        true
    }
}
