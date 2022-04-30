//! Type that describe a mode of encoding.

/// The trait for a mode.
pub trait Mode {
    /// Indicate if the current mode is human readable.
    fn is_human_readable() -> bool {
        false
    }
}

/// The default encoding mode.
#[derive(Clone, Copy)]
pub enum DefaultMode {}

impl Mode for DefaultMode {}
