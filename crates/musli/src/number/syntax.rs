/// The syntax a number is written in.
///
/// Everything here is an associated constant, so a parser instantiated for a
/// given syntax is specialized down to the forms that syntax actually accepts
/// and pays nothing for the ones it does not.
///
/// Note that a trailing point, as in `42.`, is accepted regardless of the
/// syntax. Canonical JSON does not permit it, but the JSON parser has always
/// accepted it and rejecting it now would be a breaking change.
///
/// This governs integers exactly. Floats are only steered by it as far as
/// deciding what to pick off before the digits reach the conversion in
/// [`dec2flt`], which accepts a superset of all of them.
///
/// [`dec2flt`]: crate::dec2flt
pub(crate) trait Syntax {
    /// Whether a number may carry a leading `+`.
    ///
    /// A leading `-` is always permitted.
    const PLUS: bool;

    /// Whether an integer may be written in hexadecimal, introduced by `0x` or
    /// `0X`.
    const HEX: bool;

    /// Whether the integral part may be left out entirely, as in `.5`.
    const LEADING_POINT: bool;

    /// Whether redundant leading zeros are permitted, as in `007`.
    const LEADING_ZEROS: bool;
}

/// Numbers as [RFC 8259] spells them, which is what the [`json`] module reads
/// and what SQLite stores in its `INT` and `FLOAT` elements.
///
/// [RFC 8259]: https://datatracker.ietf.org/doc/html/rfc8259#section-6
/// [`json`]: crate::json
#[derive(Clone, Copy)]
pub(crate) struct Json;

impl Syntax for Json {
    const PLUS: bool = false;
    const HEX: bool = false;
    const LEADING_POINT: bool = false;
    const LEADING_ZEROS: bool = false;
}

/// Numbers as [JSON5] spells them, which is what SQLite stores in its `INT5`
/// and `FLOAT5` elements.
///
/// This is a superset of [`Json`] which additionally permits a leading `+`,
/// hexadecimal integers, a number with no digits before its point, and, since
/// SQLite is the one writing these, a redundant leading zero.
///
/// [JSON5]: https://spec.json5.org/#numbers
#[derive(Clone, Copy)]
pub(crate) struct Json5;

impl Syntax for Json5 {
    const PLUS: bool = true;
    const HEX: bool = true;
    const LEADING_POINT: bool = true;
    const LEADING_ZEROS: bool = true;
}
