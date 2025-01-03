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
use crate::alloc::System;
#[cfg(feature = "alloc")]
use crate::mode::Binary;
#[cfg(feature = "alloc")]
use crate::value::en::ValueEncoder;
#[cfg(feature = "alloc")]
use crate::Encode;
use crate::{Decode, Options};

const OPTIONS: Options = crate::options::new().build();

/// Encode something that implements [Encode] into a [Value].
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub fn encode<T>(value: T) -> Result<Value<System>, Error>
where
    T: Encode<Binary>,
{
    use crate::en::Encoder;

    let mut output = Value::Unit;
    let cx = crate::context::Same::<Binary, Error, _>::with_alloc(System::new());
    ValueEncoder::<OPTIONS, _, _>::new(&cx, &mut output).encode(value)?;
    Ok(output)
}

/// Decode a [Value] into a type which implements [Decode].
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub fn decode<'de, T>(value: &'de Value<impl Allocator>) -> Result<T, Error>
where
    T: Decode<'de, Binary, System>,
{
    use crate::de::Decoder;
    let cx = crate::context::Same::<Binary, Error, _>::with_alloc(System::new());
    value.decoder::<OPTIONS, _>(&cx).decode()
}

/// Decode a [Value] into a type which implements [Decode] using a custom
/// context.
pub fn decode_with<'de, C, T>(cx: C, value: &'de Value<impl Allocator>) -> Result<T, C::Error>
where
    C: crate::Context,
    T: Decode<'de, C::Mode, C::Allocator>,
{
    use crate::de::Decoder;

    cx.clear();
    value.decoder::<OPTIONS, _>(cx).decode()
}
