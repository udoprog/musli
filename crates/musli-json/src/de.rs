use musli::{de::Decoder, never::Never};
use musli_common::reader::Reader;

/// A JSON decoder for Müsli.
pub struct JsonDecoder<'de, R> {
    _reader: &'de mut R,
}

impl<'de, R> JsonDecoder<'de, R> {
    /// Construct a new fixed width message encoder.
    #[inline]
    pub(crate) fn new(reader: &'de mut R) -> Self {
        Self { _reader: reader }
    }
}

impl<'de, 'a, R> Decoder<'de> for JsonDecoder<'a, R>
where
    R: Reader<'de>,
{
    type Error = R::Error;
    type Pack = Never<Self>;
    type Sequence = Never<Self>;
    type Tuple = Never<Self>;
    type Map = Never<Self>;
    type Some = Never<Self>;
    type Struct = Never<Self>;
    type TupleStruct = Never<Self>;
    type Variant = Never<Self>;

    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "value that can be decoded from JSON")
    }
}
