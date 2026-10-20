//! Encode a value through [`Display`] and decode it through [`FromStr`].
//!
//! This module can be used as a field adapter with
//! [`#[musli(with = musli::display)]`][with], which makes a field a string on
//! the wire. Its [`option`] submodule does the same for an `Option<T>` field.
//!
//! For a type which should always be a string on the wire, the derives also
//! accept [`#[musli(display)]`][display] on the type itself.
//!
//! Decoding fails with the [`FromStr`] error, formatted through its
//! [`Display`] implementation, if the string can not be parsed.
//!
//! [`Display`]: core::fmt::Display
//! [with]: https://docs.rs/musli/latest/musli/_help/derives/index.html#musliwith--path
//! [display]: https://docs.rs/musli/latest/musli/_help/derives/index.html#muslidisplay
//!
//! # Examples
//!
//! ```
//! use std::net::Ipv4Addr;
//!
//! use musli::{Decode, Encode};
//!
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! struct Host {
//!     #[musli(with = musli::display)]
//!     address: Ipv4Addr,
//!     #[musli(with = musli::display::option)]
//!     gateway: Option<Ipv4Addr>,
//! }
//!
//! let host = Host {
//!     address: Ipv4Addr::new(10, 0, 0, 2),
//!     gateway: None,
//! };
//!
//! let json = musli::json::to_string(&host)?;
//! assert_eq!(json, r#"{"address":"10.0.0.2","gateway":null}"#);
//! assert_eq!(musli::json::from_str::<Host>(&json)?, host);
//!
//! let error = musli::json::from_str::<Host>(r#"{"address":"10.0.0","gateway":null}"#);
//! assert!(error.is_err());
//! # Ok::<_, musli::json::Error>(())
//! ```

use core::fmt;
use core::str::FromStr;

use crate::{Context, Decoder, Encoder};

/// Encode `value` as a string through its [`Display`] implementation.
///
/// [`Display`]: core::fmt::Display
///
/// # Examples
///
/// ```
/// use std::net::Ipv4Addr;
///
/// use musli::Encode;
///
/// #[derive(Encode)]
/// struct Host {
///     #[musli(encode_with = musli::display::encode)]
///     address: Ipv4Addr,
/// }
///
/// let host = Host { address: Ipv4Addr::LOCALHOST };
/// assert_eq!(musli::json::to_string(&host)?, r#"{"address":"127.0.0.1"}"#);
/// # Ok::<_, musli::json::Error>(())
/// ```
#[inline]
pub fn encode<E, T>(value: &T, encoder: E) -> Result<(), E::Error>
where
    E: Encoder,
    T: ?Sized + fmt::Display,
{
    encoder.collect_string(value)
}

/// Decode a string and parse it through [`FromStr`].
///
/// A parse error is reported through the decoder's context, formatted through
/// its [`Display`] implementation.
///
/// [`Display`]: core::fmt::Display
///
/// # Examples
///
/// ```
/// use std::net::Ipv4Addr;
///
/// use musli::Decode;
///
/// #[derive(Debug, PartialEq, Decode)]
/// struct Host {
///     #[musli(decode_with = musli::display::decode)]
///     address: Ipv4Addr,
/// }
///
/// let host: Host = musli::json::from_str(r#"{"address":"127.0.0.1"}"#)?;
/// assert_eq!(host, Host { address: Ipv4Addr::LOCALHOST });
/// # Ok::<_, musli::json::Error>(())
/// ```
#[inline]
pub fn decode<'de, D, T>(decoder: D) -> Result<T, D::Error>
where
    D: Decoder<'de>,
    T: FromStr<Err: fmt::Display>,
{
    let cx = decoder.cx();
    decoder.decode_unsized(|string: &str| string.parse().map_err(|error| cx.message(error)))
}

/// Encode an `Option<T>` through [`Display`] and decode it through [`FromStr`].
///
/// The value is encoded as an optional string, so `None` is encoded the same
/// way as it is for any other `Option<T>`.
///
/// [`Display`]: core::fmt::Display
///
/// # Examples
///
/// ```
/// use std::net::Ipv4Addr;
///
/// use musli::{Decode, Encode};
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Host {
///     #[musli(with = musli::display::option)]
///     gateway: Option<Ipv4Addr>,
/// }
///
/// let host = Host { gateway: Some(Ipv4Addr::new(10, 0, 0, 1)) };
/// let json = musli::json::to_string(&host)?;
/// assert_eq!(json, r#"{"gateway":"10.0.0.1"}"#);
/// assert_eq!(musli::json::from_str::<Host>(&json)?, host);
/// # Ok::<_, musli::json::Error>(())
/// ```
pub mod option {
    use core::fmt;
    use core::str::FromStr;

    use crate::{Decoder, Encoder};

    /// Encode `value` as an optional string through [`Display`].
    ///
    /// [`Display`]: core::fmt::Display
    #[inline]
    pub fn encode<E, T>(value: &Option<T>, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
        T: fmt::Display,
    {
        match value {
            Some(value) => super::encode(value, encoder.encode_some()?),
            None => encoder.encode_none(),
        }
    }

    /// Decode an optional string and parse it through [`FromStr`].
    #[inline]
    pub fn decode<'de, D, T>(decoder: D) -> Result<Option<T>, D::Error>
    where
        D: Decoder<'de>,
        T: FromStr<Err: fmt::Display>,
    {
        match decoder.decode_option()? {
            Some(decoder) => Ok(Some(super::decode(decoder)?)),
            None => Ok(None),
        }
    }
}
