use core::fmt;

use crate::hint::MapHint;
use crate::Context;

use super::{Encode, Encoder, EntryEncoder};

/// Encoder for a map.
pub trait MapEncoder {
    /// Context associated with the encoder.
    type Cx: Context<Error = Self::Error>;
    /// Error associated with encoding.
    type Error;
    /// The mode of the encoder.
    type Mode: 'static;
    /// Encode the next pair.
    type EncodeEntry<'this>: EntryEncoder<Cx = Self::Cx, Error = Self::Error, Mode = Self::Mode>
    where
        Self: 'this;

    /// Access the context associated with the encoder.
    fn cx(&self) -> Self::Cx;

    /// Encode the next map entry.
    #[must_use = "Encoders must be consumed"]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, Self::Error>;

    /// Simplified encoder for a map entry, which ensures that encoding is
    /// always finished.
    #[inline]
    fn encode_entry_fn<F>(&mut self, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self::EncodeEntry<'_>) -> Result<(), Self::Error>,
    {
        let mut encoder = self.encode_entry()?;
        f(&mut encoder)?;
        encoder.finish_entry()
    }

    /// Insert a map entry.
    #[inline]
    fn insert_entry<F, S>(&mut self, key: F, value: S) -> Result<(), Self::Error>
    where
        F: Encode<Self::Mode>,
        S: Encode<Self::Mode>,
    {
        self.encode_entry()?.insert_entry(key, value)?;
        Ok(())
    }

    /// Encode a map entry.
    #[inline]
    fn as_encoder(&mut self) -> AsEncoder<'_, Self>
    where
        Self: Sized,
    {
        AsEncoder { encoder: self }
    }

    /// Finish encoding a map.
    fn finish_map(self) -> Result<(), Self::Error>;
}

/// Encoder for a map.
pub struct AsEncoder<'a, E>
where
    E: ?Sized,
{
    encoder: &'a mut E,
}

#[crate::trait_defaults(crate)]
impl<'a, E> Encoder for AsEncoder<'a, E>
where
    E: MapEncoder,
{
    type Cx = E::Cx;
    type Error = E::Error;
    type Mode = E::Mode;
    type EncodeMap = AsEncoderEncodeMap<'a, E>;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a map")
    }

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.encoder.cx()
    }

    #[inline]
    fn encode_map(self, _: impl MapHint) -> Result<Self::EncodeMap, Self::Error> {
        Ok(AsEncoderEncodeMap {
            encoder: self.encoder,
        })
    }
}

pub struct AsEncoderEncodeMap<'a, E>
where
    E: ?Sized,
{
    encoder: &'a mut E,
}

impl<E> MapEncoder for AsEncoderEncodeMap<'_, E>
where
    E: ?Sized + MapEncoder,
{
    type Cx = E::Cx;
    type Error = E::Error;
    type Mode = E::Mode;
    type EncodeEntry<'this>
        = E::EncodeEntry<'this>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.encoder.cx()
    }

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, Self::Error> {
        self.encoder.encode_entry()
    }

    #[inline]
    fn finish_map(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
