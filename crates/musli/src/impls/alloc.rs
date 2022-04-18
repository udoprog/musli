use std::borrow::Cow;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::hash::{BuildHasher, Hash};

use crate::de::{Decode, Decoder, MapDecoder, MapEntryDecoder, SequenceDecoder};
use crate::en::{Encode, Encoder, PairEncoder, SequenceEncoder};
use crate::error::Error;
use crate::internal::size_hint;

impl Encode for String {
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        self.as_str().encode(encoder)
    }
}

impl<'de> Decode<'de> for String {
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Ok(<&str>::decode(decoder)?.to_owned())
    }
}

impl Encode for Box<str> {
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        self.as_ref().encode(encoder)
    }
}

impl<'de> Decode<'de> for Box<str> {
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Ok(<&str>::decode(decoder)?.into())
    }
}

impl Encode for Cow<'_, str> {
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        self.as_ref().encode(encoder)
    }
}

impl<'de> Decode<'de> for Cow<'de, str> {
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let string = <&str>::decode(decoder)?;
        Ok(Cow::Borrowed(string))
    }
}

macro_rules! sequence {
    (
        $ty:ident <T $(: $trait0:ident $(+ $trait:ident)*)? $(, $extra:ident: $extra_bound0:ident $(+ $extra_bound:ident)*)*>,
        $insert:ident,
        $access:ident,
        $factory:expr
    ) => {
        impl<T $(, $extra)*> Encode for $ty<T $(, $extra)*>
        where
            T: Encode,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
            where
                E: Encoder,
            {
                let mut seq = encoder.encode_sequence(self.len())?;

                for value in self {
                    value.encode(seq.encode_next()?)?;
                }

                seq.finish()
            }
        }

        impl<'de, T $(, $extra)*> Decode<'de> for $ty<T $(, $extra)*>
        where
            T: Decode<'de> $(+ $trait0 $(+ $trait)*)*,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                let mut $access = decoder.decode_sequence()?;
                let mut out = $factory;

                while let Some(value) = $access.decode_next()? {
                    out.$insert(T::decode(value)?);
                }

                Ok(out)
            }
        }
    }
}

sequence!(
    Vec<T>,
    push,
    seq,
    Vec::with_capacity(size_hint::cautious(seq.size_hint()))
);
sequence!(
    HashSet<T: Eq + Hash, S: BuildHasher + Default>,
    insert,
    set,
    HashSet::with_capacity_and_hasher(size_hint::cautious(set.size_hint()), S::default())
);
sequence!(
    BinaryHeap<T: Ord>,
    push,
    seq,
    BinaryHeap::with_capacity(size_hint::cautious(seq.size_hint()))
);

macro_rules! map {
    (
        $ty:ident<K $(: $key_bound0:ident $(+ $key_bound:ident)*)?, V $(, $extra:ident: $extra_bound0:ident $(+ $extra_bound:ident)*)*>,
        $access:ident,
        $with_capacity:expr
    ) => {
        impl<'de, K, V $(, $extra)*> Encode for $ty<K, V $(, $extra)*>
        where
            K: Encode,
            V: Encode,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
            where
                E: Encoder,
            {
                let mut map = encoder.encode_map(self.len())?;

                for (k, v) in self {
                    let entry = map.encode_first()?;
                    Encode::encode(k, entry)?;
                    let value = map.encode_second()?;
                    Encode::encode(v, value)?;
                }

                map.finish()
            }
        }

        impl<'de, K, V $(, $extra)*> Decode<'de> for $ty<K, V $(, $extra)*>
        where
            K: Decode<'de> $(+ $key_bound0 $(+ $key_bound)*)*,
            V: Decode<'de>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                let mut $access = decoder.decode_map()?;
                let mut out = $with_capacity;

                while let Some(mut entry) = $access.decode_entry()? {
                    let key = K::decode(entry.decode_key()?)?;
                    let value = V::decode(entry.decode_value()?)?;
                    out.insert(key, value);
                }

                Ok(out)
            }
        }
    }
}

map!(
    BTreeMap<K: Ord, V>,
    map,
    BTreeMap::new()
);
map!(
    HashMap<K: Eq + Hash, V, S: BuildHasher + Default>,
    map,
    HashMap::with_capacity_and_hasher(size_hint::cautious(map.size_hint()), S::default())
);

impl Encode for CStr {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.encode_bytes(self.to_bytes_with_nul())
    }
}

impl<'de> Decode<'de> for &'de CStr {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let bytes = decoder.decode_bytes()?;
        CStr::from_bytes_with_nul(bytes).map_err(D::Error::custom)
    }
}

impl Encode for CString {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.encode_bytes(self.to_bytes_with_nul())
    }
}

impl<'de> Decode<'de> for CString {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Ok(<&CStr>::decode(decoder)?.to_owned())
    }
}
