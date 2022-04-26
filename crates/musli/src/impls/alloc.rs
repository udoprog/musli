use core::fmt;
use std::borrow::Cow;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet, VecDeque};
use std::ffi::{CStr, CString};
use std::hash::{BuildHasher, Hash};
use std::marker;

use crate::de::{Decode, Decoder, PairDecoder, PairsDecoder, SequenceDecoder, ValueVisitor};
use crate::en::{Encode, Encoder, PairEncoder, SequenceEncoder};
use crate::error::Error;
use crate::internal::size_hint;

impl Encode for String {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        self.as_str().encode(encoder)
    }
}

impl<'de> Decode<'de> for String {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        return decoder.decode_string(Visitor(marker::PhantomData));

        struct Visitor<E>(marker::PhantomData<E>);

        impl<'de, E> ValueVisitor<'de> for Visitor<E>
        where
            E: Error,
        {
            type Target = str;
            type Ok = String;
            type Error = E;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "a string")
            }

            #[inline]
            fn visit_borrowed(self, string: &'de str) -> Result<Self::Ok, Self::Error> {
                self.visit_any(string)
            }

            #[inline]
            fn visit_any(self, string: &str) -> Result<Self::Ok, Self::Error> {
                Ok(string.to_owned())
            }
        }
    }
}

impl Encode for Box<str> {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        self.as_ref().encode(encoder)
    }
}

impl<'de> Decode<'de> for Box<str> {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Ok(<&str>::decode(decoder)?.into())
    }
}

impl Encode for Cow<'_, str> {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        self.as_ref().encode(encoder)
    }
}

impl<'de> Decode<'de> for Cow<'de, str> {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        return decoder.decode_string(Visitor(marker::PhantomData));

        struct Visitor<E>(marker::PhantomData<E>);

        impl<'de, E> ValueVisitor<'de> for Visitor<E>
        where
            E: Error,
        {
            type Target = str;
            type Ok = Cow<'de, str>;
            type Error = E;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "a string")
            }

            #[inline]
            fn visit_borrowed(self, string: &'de str) -> Result<Self::Ok, Self::Error> {
                Ok(Cow::Borrowed(string))
            }

            #[inline]
            fn visit_any(self, string: &str) -> Result<Self::Ok, Self::Error> {
                Ok(Cow::Owned(string.to_owned()))
            }
        }
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
            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder,
            {
                let mut seq = encoder.encode_sequence(self.len())?;

                for value in self {
                    value.encode(seq.next()?)?;
                }

                seq.end()
            }
        }

        impl<'de, T $(, $extra)*> Decode<'de> for $ty<T $(, $extra)*>
        where
            T: Decode<'de> $(+ $trait0 $(+ $trait)*)*,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                let mut $access = decoder.decode_sequence()?;
                let mut out = $factory;

                while let Some(value) = $access.next()? {
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
    VecDeque<T>,
    push_back,
    seq,
    VecDeque::with_capacity(size_hint::cautious(seq.size_hint()))
);
sequence!(
    HashSet<T: Eq + Hash, S: BuildHasher + Default>,
    insert,
    seq,
    HashSet::with_capacity_and_hasher(size_hint::cautious(seq.size_hint()), S::default())
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
            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder,
            {
                let mut map = encoder.encode_map(self.len())?;

                for (k, v) in self {
                    let entry = map.first()?;
                    Encode::encode(k, entry)?;
                    let value = map.second()?;
                    Encode::encode(v, value)?;
                }

                map.end()
            }
        }

        impl<'de, K, V $(, $extra)*> Decode<'de> for $ty<K, V $(, $extra)*>
        where
            K: Decode<'de> $(+ $key_bound0 $(+ $key_bound)*)*,
            V: Decode<'de>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                let mut $access = decoder.decode_map()?;
                let mut out = $with_capacity;

                while let Some(mut entry) = $access.next()? {
                    let key = K::decode(entry.first()?)?;
                    let value = V::decode(entry.second()?)?;
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
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
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
        let bytes = <&[u8]>::decode(decoder)?;
        CStr::from_bytes_with_nul(bytes).map_err(D::Error::custom)
    }
}

impl Encode for CString {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
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
