//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/musli-serde.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-serde)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--serde-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-serde)
//!
//! Transparent shim to use [`serde`] types in Müsli.
//!
//! This conveniently and transparently allows Müsli to use fields which are
//! serde types by marking them with  `#[musli(with = musli_serde)]`. This can
//! be useful because there is a wide ecosystem of types which implements serde
//! traits.
//!
//! Note that the exact method that fields are serialized and deserialized will
//! not match what Müsli does, since serde requires the use of a fundamentally
//! different model and Müsli metadata such as `#[musli(rename = ..)]` is not
//! available in [`serde`].
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
//! #[musli(default_field = "name")]
//! struct Person {
//!     name: String,
//!     #[musli(with = musli_serde)]
//!     address: Address,
//!     #[musli(with = musli_serde)]
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
//! # #[musli(default_field = "name")]
//! # struct Person { name: String, #[musli(with = musli_serde)] address: Address, #[musli(with = musli_serde)] url: Url }
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_field = "name")]
//! struct MusliAddress {
//!     street: String,
//!     city: String,
//!     zip: u32,
//! }
//!
//! #[derive(Encode, Decode)]
//! #[musli(default_field = "name")]
//! struct MusliPerson {
//!     name: String,
//!     address: MusliAddress,
//!     url: String,
//! }
//!
//! let json = musli_json::to_string(&Person {
//!     name: "John Doe".to_string(),
//!     address: Address {
//!         street: "Main St.".to_string(),
//!         city: "Springfield".to_string(),
//!         zip: 12345,
//!     },
//!     url: Url::parse("https://example.com")?,
//! })?;
//!
//! let musli = musli_json::from_str::<MusliPerson>(&json)?;
//!
//! assert_eq!(musli.name, "John Doe");
//! assert_eq!(musli.address.street, "Main St.");
//! assert_eq!(musli.address.city, "Springfield");
//! assert_eq!(musli.address.zip, 12345);
//! assert_eq!(musli.url, "https://example.com/");
//! # Ok::<_, Box<dyn std::error::Error>>(())
//! ```
//!
//! [`serde`]: https://serde.rs

#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

mod deserializer;
mod error;
mod serializer;

use core::cell::RefCell;
use core::fmt;

use musli::context::StdError;
use musli::{Context, Decoder, Encoder};
use serde::{Deserialize, Serialize};

use self::deserializer::Deserializer;
use self::serializer::Serializer;

use musli_common::exports::buf::{self, BufString};

struct SerdeContext<'a, C>
where
    C: ?Sized + Context,
{
    error: RefCell<Option<C::Error>>,
    inner: &'a C,
}

impl<'a, C> Context for SerdeContext<'a, C>
where
    C: ?Sized + Context,
{
    type Mode = C::Mode;
    type Error = error::SerdeError;
    type Mark = C::Mark;
    type Buf<'this> = C::Buf<'this>
    where
        Self: 'this;
    type BufString<'this> = BufString<C::Buf<'this>>
    where
        Self: 'this;

    #[inline]
    fn mark(&self) -> Self::Mark {
        self.inner.mark()
    }

    #[inline]
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        self.inner.alloc()
    }

    #[inline]
    fn collect_string<T>(&self, value: &T) -> Result<Self::BufString<'_>, Self::Error>
    where
        T: ?Sized + fmt::Display,
    {
        buf::collect_string(self, value)
    }

    #[inline]
    fn custom<T>(&self, error: T) -> Self::Error
    where
        T: 'static + Send + Sync + StdError,
    {
        *self.error.borrow_mut() = Some(self.inner.custom(error));
        error::SerdeError::Captured
    }

    #[inline]
    fn message<T>(&self, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        *self.error.borrow_mut() = Some(self.inner.message(message));
        error::SerdeError::Captured
    }
}

/// Encode the given serde value `T` to the given [Encoder] using the serde
/// compatibility layer.
pub fn encode<E, T>(value: &T, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
where
    E: Encoder,
    T: Serialize,
{
    let cx = SerdeContext {
        error: RefCell::new(None),
        inner: cx,
    };

    let encoder = encoder.with_context(&cx)?;

    let serializer = Serializer::new(&cx, encoder);

    let error = match value.serialize(serializer) {
        Ok(value) => return Ok(value),
        Err(error) => error,
    };

    if let Some(error) = error.report(cx.inner) {
        return Err(error);
    }

    let Some(error) = cx.error.borrow_mut().take() else {
        return Err(cx.inner.message("error during encoding (no information)"));
    };

    Err(error)
}

/// Decode the given serde value `T` from the given [Decoder] using the serde
/// compatibility layer.
pub fn decode<'de, D, T>(cx: &D::Cx, decoder: D) -> Result<T, D::Error>
where
    D: Decoder<'de>,
    T: Deserialize<'de>,
{
    let cx = SerdeContext {
        error: RefCell::new(None),
        inner: cx,
    };

    let decoder = decoder.with_context(&cx)?;

    let deserializer = Deserializer::new(&cx, decoder);

    let error = match T::deserialize(deserializer) {
        Ok(value) => return Ok(value),
        Err(error) => error,
    };

    if let Some(error) = error.report(cx.inner) {
        return Err(error);
    }

    let Some(error) = cx.error.borrow_mut().take() else {
        return Err(cx.inner.message("error during encoding (no information)"));
    };

    Err(error)
}
