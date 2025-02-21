//! Transparent [`serde`] support for Müsli types.
//!
//! This conveniently and transparently allows Müsli to use fields which are
//! serde types by marking them with  `#[musli(with = musli::serde)]`. This can
//! be useful because there is a wide ecosystem of types which implements serde
//! traits.
//!
//! Note that the exact method that fields are serialized and deserialized will
//! not match what Müsli does, since serde requires the use of a fundamentally
//! different model and Müsli metadata such as `#[musli(name = ..)]` is not
//! available in [`serde`].
//!
//! [`serde`]: https://serde.rs
//!
//! <br>
//!
//! ## Examples
//!
//! ```
//! use serde::{Serialize, Deserialize};
//! use musli::{Encode, Decode};
//! use url::Url;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Address {
//!     street: String,
//!     city: String,
//!     zip: u32,
//! }
//!
//! #[derive(Encode, Decode)]
//! #[musli(name_all = "name")]
//! struct Person {
//!     name: String,
//!     #[musli(with = musli::serde)]
//!     address: Address,
//!     #[musli(with = musli::serde)]
//!     url: Url,
//! }
//! ```
//!
//! A compatible Müsli structure would look like this:
//!
//! ```
//! use musli::{Encode, Decode};
//! use url::Url;
//! # use serde::{Serialize, Deserialize};
//! # #[derive(Serialize, Deserialize)]
//! # struct Address { street: String, city: String, zip: u32 }
//! # #[derive(Encode, Decode)]
//! # #[musli(name_all = "name")]
//! # struct Person { name: String, #[musli(with = musli::serde)] address: Address, #[musli(with = musli::serde)] url: Url }
//!
//! #[derive(Encode, Decode)]
//! #[musli(name_all = "name")]
//! struct MusliAddress {
//!     street: String,
//!     city: String,
//!     zip: u32,
//! }
//!
//! #[derive(Encode, Decode)]
//! #[musli(name_all = "name")]
//! struct MusliPerson {
//!     name: String,
//!     address: MusliAddress,
//!     url: String,
//! }
//!
//! let json = musli::json::to_string(&Person {
//!     name: "John Doe".to_string(),
//!     address: Address {
//!         street: "Main St.".to_string(),
//!         city: "Springfield".to_string(),
//!         zip: 12345,
//!     },
//!     url: Url::parse("https://example.com")?,
//! })?;
//!
//! let musli = musli::json::from_str::<MusliPerson>(&json)?;
//!
//! assert_eq!(musli.name, "John Doe");
//! assert_eq!(musli.address.street, "Main St.");
//! assert_eq!(musli.address.city, "Springfield");
//! assert_eq!(musli.address.zip, 12345);
//! assert_eq!(musli.url, "https://example.com/");
//! # Ok::<_, Box<dyn core::error::Error>>(())
//! ```

#![cfg(feature = "serde")]
#![cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]

mod de;
mod error;
mod ser;

use serde::{Deserialize, Serialize};

use self::de::Deserializer;
use self::ser::Serializer;

use crate::{Decoder, Encoder};

/// Encode the given serde value `T` to the given [`Encoder`] using the serde
/// compatibility layer.
///
/// ## Examples
///
/// ```
/// use serde::{Serialize, Deserialize};
/// use musli::{Encode, Decode};
/// use url::Url;
///
/// #[derive(Serialize, Deserialize)]
/// struct Address {
///     street: String,
///     city: String,
///     zip: u32,
/// }
///
/// #[derive(Encode, Decode)]
/// #[musli(name_all = "name")]
/// struct Person {
///     name: String,
///     #[musli(with = musli::serde)]
///     address: Address,
///     #[musli(with = musli::serde)]
///     url: Url,
/// }
/// ```
#[inline]
pub fn encode<E, T>(value: &T, encoder: E) -> Result<E::Ok, E::Error>
where
    E: Encoder,
    T: Serialize,
{
    let cx = encoder.cx();
    let serializer = Serializer::new(encoder);
    value.serialize(serializer).map_err(error::err(cx))
}

/// Decode the given serde value `T` from the given [`Decoder`] using the serde
/// compatibility layer.
///
/// ## Examples
///
/// ```
/// use serde::{Serialize, Deserialize};
/// use musli::{Encode, Decode};
/// use url::Url;
///
/// #[derive(Serialize, Deserialize)]
/// struct Address {
///     street: String,
///     city: String,
///     zip: u32,
/// }
///
/// #[derive(Encode, Decode)]
/// #[musli(name_all = "name")]
/// struct Person {
///     name: String,
///     #[musli(with = musli::serde)]
///     address: Address,
///     #[musli(with = musli::serde)]
///     url: Url,
/// }
/// ```
#[inline]
pub fn decode<'de, D, T>(decoder: D) -> Result<T, D::Error>
where
    D: Decoder<'de>,
    T: Deserialize<'de>,
{
    let cx = decoder.cx();
    let deserializer = Deserializer::new(decoder);
    T::deserialize(deserializer).map_err(error::err(cx))
}
