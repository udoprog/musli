use core::any::TypeId;

use crate::mode::Text;
use crate::{Allocator, Context, Decode, Decoder, Encode, Encoder};

/// Platform tag used by certain platform-specific implementations.
#[repr(u8)]
pub(super) enum PlatformTag {
    Unix = 0,
    Windows = 1,
}

impl<M> Encode<M> for PlatformTag
where
    M: 'static,
{
    type Encode = Self;

    /// Can't be bitwise encoded since TypeId cannot be const compiled.
    const IS_BITWISE_ENCODE: bool = false;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder<Mode = M>,
    {
        if TypeId::of::<M>() == TypeId::of::<Text>() {
            match self {
                PlatformTag::Unix => encoder.encode("unix"),
                PlatformTag::Windows => encoder.encode("windows"),
            }
        } else {
            // For binary encoding, we use the tag as a single byte.
            let tag = match self {
                PlatformTag::Unix => 0,
                PlatformTag::Windows => 1,
            };

            encoder.encode_u8(tag)
        }
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M, A> Decode<'de, M, A> for PlatformTag
where
    M: 'static,
    A: Allocator,
{
    /// This will always be false since platform tag cannot inhabit all possible
    /// u8 bit patterns even when in binary mode.
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Allocator = A>,
    {
        let cx = decoder.cx();

        if TypeId::of::<M>() == TypeId::of::<Text>() {
            decoder.decode_unsized(|value: &str| match value {
                "unix" => Ok(PlatformTag::Unix),
                "windows" => Ok(PlatformTag::Windows),
                _ => Err(cx.message(format_args!("Unsupported platform tag `{value}`",))),
            })
        } else {
            match decoder.decode_u8()? {
                0 => Ok(PlatformTag::Unix),
                1 => Ok(PlatformTag::Windows),
                _ => Err(cx.message("Unsupported platform tag")),
            }
        }
    }
}
