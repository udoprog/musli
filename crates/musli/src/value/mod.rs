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

/// Convenient result alias for use with `musli_value`.
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[doc(inline)]
pub use self::value::{AsValueDecoder, IntoValueDecoder, Value};
#[doc(inline)]
pub use error::Error;

use crate::alloc::Allocator;
#[cfg(feature = "alloc")]
use crate::alloc::Global;
#[cfg(feature = "alloc")]
use crate::mode::Binary;
#[cfg(feature = "alloc")]
use crate::value::en::ValueEncoder;
#[cfg(feature = "alloc")]
use crate::Encode;
use crate::{Decode, Options};

const OPTIONS: Options = crate::options::new().build();

/// Encode something that implements [Encode] into a [Value] in the [`Binary`]
/// mode.
///
/// # Examples
///
/// ```
/// use musli::{Encode, value};
///
/// #[derive(Encode)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let person = Person { name: "Alice".to_string(), age: 30 };
/// let value = value::encode(person)?;
/// # Ok::<_, value::Error>(())
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub fn encode<T>(value: T) -> Result<Value<Global>, Error>
where
    T: Encode<Binary>,
{
    use crate::en::Encoder;

    let mut output = Value::Unit;
    let cx = crate::context::new().with_error();
    ValueEncoder::<OPTIONS, _, _, Binary>::new(&cx, &mut output).encode(value)?;
    Ok(output)
}

/// Decode a [Value] into a type which implements [Decode] in the [`Binary`]
/// mode.
///
/// # Examples
///
/// ```
/// use musli::{Encode, Decode, value};
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let original = Person { name: "Alice".to_string(), age: 30 };
/// let encoded = value::encode(&original)?;
/// let decoded: Person = value::decode(&encoded)?;
/// assert_eq!(original, decoded);
/// # Ok::<_, value::Error>(())
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub fn decode<'de, T>(value: &'de Value<impl Allocator>) -> Result<T, Error>
where
    T: Decode<'de, Binary, Global>,
{
    use crate::de::Decoder;
    let cx = crate::context::new().with_error();
    value.decoder::<OPTIONS, _, Binary>(&cx).decode()
}

/// Decode a [Value] into a type which implements [Decode] using a custom
/// context in the [`Binary`] mode.
///
/// # Examples
///
/// ```
/// use musli::{context, Encode, Decode, value};
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let cx = context::new();
/// let original = Person { name: "Alice".to_string(), age: 30 };
/// let encoded = value::encode(&original).unwrap();
/// let decoded: Person = value::decode_with::<_, _, musli::mode::Binary>(&cx, &encoded)?;
/// assert_eq!(original, decoded);
/// # Ok::<_, musli::context::ErrorMarker>(())
/// ```
pub fn decode_with<'de, C, T, M>(cx: C, value: &'de Value<impl Allocator>) -> Result<T, C::Error>
where
    C: crate::Context,
    T: Decode<'de, M, C::Allocator>,
    M: 'static,
{
    use crate::de::Decoder;

    cx.clear();
    value.decoder::<OPTIONS, _, M>(cx).decode()
}
