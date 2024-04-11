use crate::mode::DefaultMode;
use crate::Context;

use super::Decoder;

/// A trait implemented for types which can be visited by reference.
///
/// This is used for types like `str` which are unsized, and might require
/// internal allocating to properly decode. Simply using the `Decode`
/// implementation would restrict it to only be used through `&'de str` which
/// would demand an exact reference to data from the decoded source.
pub trait Visit<'de, M = DefaultMode> {
    /// Decode the given input using a closure as visitor.
    fn visit<D, F, O>(cx: &D::Cx, decoder: D, f: F) -> Result<O, <D::Cx as Context>::Error>
    where
        D: Decoder<'de, Mode = M>,
        F: FnOnce(&Self) -> Result<O, D::Error>;
}
