use super::Decoder;

/// A trait implemented for types which can be visited by reference.
///
/// This behaves the same as [`DecodeUnsized`], but implicitly hints that the
/// caller is after bytes.
///
/// This is used for types like `[u8]` which are unsized, and might require
/// internal allocating to properly decode. Simply using the `Decode`
/// implementation would restrict it to only be used through `&'de [u8]` which
/// would demand an exact reference to data from the decoded source.
///
/// [`DecodeUnsized`]: super::DecodeUnsized
pub trait DecodeUnsizedBytes<'de, M> {
    /// Decode the given input using a closure as visitor.
    fn decode_unsized_bytes<D, F, O>(decoder: D, f: F) -> Result<O, D::Error>
    where
        D: Decoder<'de, Mode = M>,
        F: FnOnce(&Self) -> Result<O, D::Error>;
}
