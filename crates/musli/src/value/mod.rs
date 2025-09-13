//! Transparent buffered values.
//!
//! The [`Value`] type can losslessly store and represent anything which is
//! supported in the [Müsli data model].
//!
//! [Müsli data model]: crate::_help::data_model

#![cfg(any(feature = "json", feature = "descriptive", feature = "value"))]
#![cfg_attr(doc_cfg, doc(cfg(feature = "value")))]

mod de;
mod en;
mod error;
mod type_hint;
mod value;

use core::marker;

/// Convenient result alias for use with `musli_value`.
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[doc(inline)]
pub use self::value::{AsValueDecoder, IntoValueDecoder, Value};
use self::value::{Cow, Number, ValueKind};
#[doc(inline)]
pub use error::Error;

use crate::alloc::Allocator;
#[cfg(feature = "alloc")]
use crate::alloc::Global;
use crate::mode::Binary;
use crate::value::en::ValueEncoder;
use crate::{Context, Decode, Decoder, Encode, Encoder, Options};

const ENCODING: Encoding = Encoding::new();

/// The default options used in the value encoding and decoding.
pub const OPTIONS: Options = crate::options::new().build();

/// Encode something that implements [`Encode`] into a [`Value`] in the
/// [`Binary`] mode.
///
/// # Examples
///
/// ```
/// use musli::{Encode, Decode};
/// use musli::value;
/// use musli::mode::Binary;
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Person<'de> {
///     name: &'de str,
///     age: u32,
/// }
///
/// let person = Person { name: "Alice", age: 30 };
/// let value = value::encode(&person)?;
///
/// let decoded: Person<'_> = value::decode(&value)?;
/// assert_eq!(decoded, person);
/// # Ok::<_, value::Error>(())
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub fn encode(value: impl Encode<Binary>) -> Result<Value<'static, Global>, Error> {
    ENCODING.encode(value)
}

/// Encode something that implements [`Encode`] into a [`Value`] using a custom
/// [`Context`] `C` in the [`Binary`] mode.
///
/// A custom context allows [`Value`] encoding to be performed without an
/// allocator.
///
/// # Examples
///
/// ```
/// use musli::{Encode, Decode};
/// use musli::context;
/// use musli::value;
/// use musli::mode::Binary;
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Person<'de> {
///     name: &'de str,
///     age: u32,
/// }
///
/// let person = Person { name: "Alice", age: 30 };
///
/// let cx = context::new();
/// let value = value::encode_with(&cx, &person)?;
///
/// let decoded: Person<'_> = value::decode(&value)?;
/// assert_eq!(decoded, person);
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
pub fn encode_with<C>(
    cx: C,
    value: impl Encode<Binary>,
) -> Result<Value<'static, C::Allocator>, C::Error>
where
    C: Context,
{
    ENCODING.encode_with(cx, value)
}

/// Decode a [`Value`] into a type which implements [`Decode`] in the [`Binary`]
/// mode.
///
/// # Examples
///
/// ```
/// use musli::{Encode, Decode};
/// use musli::value;
/// use musli::mode::Binary;
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Person<'de> {
///     name: &'de str,
///     age: u32,
/// }
///
/// let person = Person { name: "Alice", age: 30 };
/// let encoded = value::encode(&person)?;
///
/// let decoded: Person<'_> = value::decode(&encoded)?;
/// assert_eq!(decoded, person);
/// # Ok::<_, value::Error>(())
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub fn decode<'a, 'de, T>(value: &'a Value<'de, impl Allocator>) -> Result<T, Error>
where
    T: Decode<'de, Binary, Global>,
{
    ENCODING.decode(value)
}

/// Decode a [`Value`] into a type which implements [`Decode`] using a custom
/// [`Context`] `C` in the [`Binary`] mode.
///
/// A custom context allows [`Value`] decoding to be performed without an
/// allocator.
///
/// # Examples
///
/// ```
/// use musli::{Encode, Decode};
/// use musli::context;
/// use musli::value;
/// use musli::mode::Binary;
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Person<'de> {
///     name: &'de str,
///     age: u32,
/// }
///
/// let person = Person { name: "Alice", age: 30 };
/// let value = value::encode(&person)?;
///
/// let cx = context::new();
/// let decoded: Person<'_> = value::decode_with(&cx, &value)?;
/// assert_eq!(decoded, person);
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
pub fn decode_with<'a, 'de, C, T>(
    cx: C,
    value: &'a Value<'de, impl Allocator>,
) -> Result<T, C::Error>
where
    C: Context,
    T: Decode<'de, Binary, C::Allocator>,
{
    ENCODING.decode_with(cx, value)
}

/// Setting up encoding with parameters.
pub struct Encoding<const OPT: Options = OPTIONS, M = Binary>
where
    M: 'static,
{
    _marker: marker::PhantomData<M>,
}

impl Default for Encoding<OPTIONS, Binary> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Encoding<OPTIONS, Binary> {
    /// Construct a new [`Encoding`].
    ///
    /// ```
    /// use musli::{Encode, Decode};
    /// use musli::value::Encoding;
    ///
    /// const ENCODING: Encoding = Encoding::new();
    ///
    /// #[derive(Debug, PartialEq, Encode, Decode)]
    /// struct Person<'a> {
    ///     name: &'a str,
    ///     age: u32,
    /// }
    ///
    /// let person = Person { name: "Alice", age: 30 };
    /// let value = ENCODING.encode(&person)?;
    ///
    /// let decoded: Person<'_> = ENCODING.decode(&value)?;
    /// assert_eq!(decoded, person);
    /// # Ok::<_, musli::value::Error>(())
    /// ```
    pub const fn new() -> Self {
        Encoding {
            _marker: marker::PhantomData,
        }
    }
}

impl<const OPT: Options, M> Encoding<OPT, M>
where
    M: 'static,
{
    /// Change the mode of the encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::value::{OPTIONS, Encoding};
    ///
    /// enum Custom {}
    ///
    /// const ENCODING: Encoding<OPTIONS, Custom> = Encoding::new().with_mode();
    /// ```
    pub const fn with_mode<T>(self) -> Encoding<OPT, T>
    where
        T: 'static,
    {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Change the options of the encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::options::{self, Options, Integer};
    /// use musli::value::Encoding;
    ///
    /// const OPTIONS: Options = options::new().integer(Integer::Fixed).build();
    /// const ENCODING: Encoding<OPTIONS> = Encoding::new().with_options();
    /// ```
    pub const fn with_options<const U: Options>(self) -> Encoding<U, M> {
        Encoding {
            _marker: marker::PhantomData,
        }
    }

    /// Encode something that implements [`Encode`] into a [`Value`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Decode};
    /// use musli::value;
    /// use musli::mode::Binary;
    ///
    /// const ENCODING: value::Encoding = value::Encoding::new();
    ///
    /// #[derive(Debug, PartialEq, Encode, Decode)]
    /// struct Person<'de> {
    ///     name: &'de str,
    ///     age: u32,
    /// }
    ///
    /// let person = Person { name: "Alice", age: 30 };
    /// let value = ENCODING.encode(&person)?;
    ///
    /// let decoded: Person<'_> = ENCODING.decode(&value)?;
    /// assert_eq!(decoded, person);
    /// # Ok::<_, value::Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
    pub fn encode(&self, value: impl Encode<M>) -> Result<Value<Global>, Error> {
        let mut output = Value::new(ValueKind::Unit);
        let cx = crate::context::new().with_error();
        ValueEncoder::<OPT, _, _, M>::new(&cx, &mut output).encode(value)?;
        Ok(output)
    }

    /// Encode something that implements [`Encode`] into a [`Value`] using a
    /// custom [`Context`] `C`.
    ///
    /// A custom context allows [`Value`] encoding to be performed without an
    /// allocator.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Decode};
    /// use musli::context;
    /// use musli::value;
    /// use musli::mode::Binary;
    ///
    /// const ENCODING: value::Encoding = value::Encoding::new();
    ///
    /// #[derive(Debug, PartialEq, Encode, Decode)]
    /// struct Person<'de> {
    ///     name: &'de str,
    ///     age: u32,
    /// }
    ///
    /// let person = Person { name: "Alice", age: 30 };
    ///
    /// let cx = context::new();
    /// let value = ENCODING.encode_with(&cx, &person)?;
    ///
    /// let decoded: Person<'_> = ENCODING.decode(&value)?;
    /// assert_eq!(decoded, person);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn encode_with<C>(
        &self,
        cx: C,
        value: impl Encode<M>,
    ) -> Result<Value<'static, C::Allocator>, C::Error>
    where
        C: Context,
    {
        let mut output = Value::new(ValueKind::Unit);
        ValueEncoder::<OPT, _, _, M>::new(cx, &mut output).encode(value)?;
        Ok(output)
    }

    /// Decode a [`Value`] into a type which implements [`Decode`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Decode};
    /// use musli::value;
    /// use musli::mode::Binary;
    ///
    /// const ENCODING: value::Encoding = value::Encoding::new();
    ///
    /// #[derive(Debug, PartialEq, Encode, Decode)]
    /// struct Person<'de> {
    ///     name: &'de str,
    ///     age: u32,
    /// }
    ///
    /// let person = Person { name: "Alice", age: 30 };
    /// let encoded = ENCODING.encode(&person)?;
    ///
    /// let decoded: Person<'_> = ENCODING.decode(&encoded)?;
    /// assert_eq!(decoded, person);
    /// # Ok::<_, value::Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
    pub fn decode<'a, 'de, T>(&self, value: &'a Value<'de, impl Allocator>) -> Result<T, Error>
    where
        T: Decode<'de, M, Global>,
    {
        let cx = crate::context::new().with_error();
        value.decoder::<OPT, _, M>(&cx).decode()
    }

    /// Decode a [`Value`] into a type which implements [`Decode`] using a
    /// custom [`Context`] `C`.
    ///
    /// A custom context allows [`Value`] decoding to be performed without an
    /// allocator.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Decode};
    /// use musli::context;
    /// use musli::value;
    /// use musli::mode::Binary;
    ///
    /// const ENCODING: value::Encoding = value::Encoding::new();
    ///
    /// #[derive(Debug, PartialEq, Encode, Decode)]
    /// struct Person<'de> {
    ///     name: &'de str,
    ///     age: u32,
    /// }
    ///
    /// let person = Person { name: "Alice", age: 30 };
    ///
    /// let value = ENCODING.encode(&person)?;
    ///
    /// let cx = context::new();
    /// let decoded: Person<'_> = ENCODING.decode_with(&cx, &value)?;
    /// assert_eq!(decoded, person);
    /// # Ok::<_, Box<dyn core::error::Error>>(())
    /// ```
    pub fn decode_with<'a, 'de: 'a, C, T>(
        &self,
        cx: C,
        value: &'a Value<'de, impl Allocator>,
    ) -> Result<T, C::Error>
    where
        C: Context,
        T: Decode<'de, M, C::Allocator>,
    {
        cx.clear();
        value.decoder::<OPT, _, M>(cx).decode()
    }
}
