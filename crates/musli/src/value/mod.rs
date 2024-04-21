//! Transparent buffered values.
//!
//! The [`Value`] type can losslessly store and represent anything which is
//! supported in the [Müsli data model].
//!
//! [Müsli data model]: crate::help::data_model

#![cfg(feature = "value")]
#![cfg_attr(doc_cfg, doc(cfg(feature = "value")))]

mod de;
mod en;
mod error;
mod type_hint;
mod value;

/// Convenient result alias for use with `musli_value`.
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[doc(inline)]
pub use self::value::{AsValueDecoder, Value};
#[doc(inline)]
pub use error::Error;

use crate::Options;
use en::ValueEncoder;
use musli_core::mode::Binary;
use musli_core::{Decode, Encode};

const OPTIONS: Options = crate::options::new().build();

/// Encode something that implements [Encode] into a [Value].
pub fn encode<T>(value: T) -> Result<Value, Error>
where
    T: Encode<Binary>,
{
    use musli_core::en::Encoder;

    let mut output = Value::Unit;

    default_allocator!(|alloc| {
        let cx = crate::context::Same::<_, Binary, Error>::new(&alloc);
        ValueEncoder::<OPTIONS, _, _>::new(&cx, &mut output).encode(value)?;
        Ok(output)
    })
}

/// Decode a [Value] into a type which implements [Decode].
pub fn decode<'de, T>(value: &'de Value) -> Result<T, Error>
where
    T: Decode<'de, Binary>,
{
    use musli_core::de::Decoder;

    default_allocator!(|alloc| {
        let cx = crate::context::Same::<_, Binary, Error>::new(&alloc);
        value.decoder::<OPTIONS, _>(&cx).decode()
    })
}

/// Decode a [Value] into a type which implements [Decode] using a custom
/// context.
pub fn decode_with<'de, C, T>(cx: &C, value: &'de Value) -> Result<T, C::Error>
where
    C: ?Sized + musli_core::Context,
    T: Decode<'de, C::Mode>,
{
    use musli_core::de::Decoder;

    cx.clear();
    value.decoder::<OPTIONS, _>(cx).decode()
}
