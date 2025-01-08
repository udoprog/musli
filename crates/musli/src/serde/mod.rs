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

mod deserializer;
mod error;
mod serializer;

use core::cell::RefCell;
use core::error::Error;
use core::fmt;

use serde::{Deserialize, Serialize};

use self::deserializer::Deserializer;
use self::serializer::Serializer;

use crate::{Context, Decoder, Encoder};

struct SerdeContext<C>
where
    C: Context,
{
    error: RefCell<Option<C::Error>>,
    inner: C,
}

impl<C> Context for &SerdeContext<C>
where
    C: Context,
{
    type Error = error::SerdeError;
    type Mark = C::Mark;
    type Allocator = C::Allocator;

    #[inline]
    fn clear(self) {
        self.inner.clear();
        *self.error.borrow_mut() = None;
    }

    #[inline]
    fn mark(self) -> Self::Mark {
        self.inner.mark()
    }

    #[inline]
    fn advance(self, n: usize) {
        self.inner.advance(n)
    }

    #[inline]
    fn alloc(self) -> Self::Allocator {
        self.inner.alloc()
    }

    #[inline]
    fn custom<E>(self, error: E) -> Self::Error
    where
        E: 'static + Send + Sync + Error,
    {
        *self.error.borrow_mut() = Some(self.inner.custom(error));
        error::SerdeError::Captured
    }

    #[inline]
    fn message<M>(self, message: M) -> Self::Error
    where
        M: fmt::Display,
    {
        *self.error.borrow_mut() = Some(self.inner.message(message));
        error::SerdeError::Captured
    }

    #[inline]
    fn enter_struct(self, type_name: &'static str) {
        self.inner.enter_struct(type_name);
    }

    #[inline]
    fn leave_struct(self) {
        self.inner.leave_struct();
    }

    #[inline]
    fn enter_enum(self, type_name: &'static str) {
        self.inner.enter_enum(type_name);
    }

    #[inline]
    fn leave_enum(self) {
        self.inner.leave_enum();
    }

    #[inline]
    fn enter_named_field<F>(self, type_name: &'static str, field: F)
    where
        F: fmt::Display,
    {
        self.inner.enter_named_field(type_name, field);
    }

    #[inline]
    fn enter_unnamed_field<F>(self, index: u32, name: F)
    where
        F: fmt::Display,
    {
        self.inner.enter_unnamed_field(index, name);
    }

    #[inline]
    fn leave_field(self) {
        self.inner.leave_field();
    }

    #[inline]
    fn enter_variant<V>(self, type_name: &'static str, tag: V)
    where
        V: fmt::Display,
    {
        self.inner.enter_variant(type_name, tag);
    }

    #[inline]
    fn leave_variant(self) {
        self.inner.leave_variant();
    }

    #[inline]
    fn enter_map_key<K>(self, field: K)
    where
        K: fmt::Display,
    {
        self.inner.enter_map_key(field);
    }

    #[inline]
    fn leave_map_key(self) {
        self.inner.leave_map_key();
    }

    #[inline]
    fn enter_sequence_index(self, index: usize) {
        self.inner.enter_sequence_index(index);
    }

    #[inline]
    fn leave_sequence_index(self) {
        self.inner.leave_sequence_index();
    }
}

/// Encode the given serde value `T` to the given [Encoder] using the serde
/// compatibility layer.
pub fn encode<E, T>(value: &T, encoder: E) -> Result<E::Ok, E::Error>
where
    E: Encoder,
    T: Serialize,
{
    let cx = encoder.cx();

    let cx = SerdeContext {
        error: RefCell::new(None),
        inner: cx,
    };

    let encoder = encoder.with_context(&cx)?;

    let serializer = Serializer::new(encoder);

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
pub fn decode<'de, D, T>(decoder: D) -> Result<T, D::Error>
where
    D: Decoder<'de>,
    T: Deserialize<'de>,
{
    let cx = decoder.cx();

    let cx = SerdeContext {
        error: RefCell::new(None),
        inner: cx,
    };

    let decoder = decoder.with_context(&cx)?;

    let deserializer = Deserializer::new(decoder);

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
