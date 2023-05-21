use core::ffi::CStr;
use core::fmt;
#[cfg(feature = "std")]
use core::hash::{BuildHasher, Hash};

use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BinaryHeap, VecDeque};
use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet};

use crate::de::{Decode, Decoder, PairDecoder, PairsDecoder, SequenceDecoder, ValueVisitor};
use crate::en::{Encode, Encoder, PairEncoder, PairsEncoder, SequenceEncoder};
use crate::internal::size_hint;
use crate::mode::Mode;
use crate::Context;

impl<M> Encode<M> for String
where
    M: Mode,
{
    #[inline]
    fn encode<'buf, C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<'buf, Input = E::Error>,
        E: Encoder,
    {
        Encode::<M>::encode(self.as_str(), cx, encoder)
    }
}

impl<'de, M> Decode<'de, M> for String
where
    M: Mode,
{
    #[inline]
    fn decode<'buf, C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<'buf, Input = D::Error>,
        D: Decoder<'de>,
    {
        struct Visitor;

        impl<'de, 'buf, C> ValueVisitor<'de, 'buf, C, str> for Visitor
        where
            C: Context<'buf>,
        {
            type Ok = String;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "string")
            }

            #[inline]
            fn visit_owned(self, _: &mut C, value: String) -> Result<Self::Ok, C::Error> {
                Ok(value)
            }

            #[inline]
            fn visit_borrowed(self, cx: &mut C, string: &'de str) -> Result<Self::Ok, C::Error> {
                self.visit_ref(cx, string)
            }

            #[inline]
            fn visit_ref(self, _: &mut C, string: &str) -> Result<Self::Ok, C::Error> {
                Ok(string.to_owned())
            }
        }

        decoder.decode_string(cx, Visitor)
    }
}

impl<M> Encode<M> for Box<str>
where
    M: Mode,
{
    #[inline]
    fn encode<'buf, C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<'buf, Input = E::Error>,
        E: Encoder,
    {
        Encode::<M>::encode(self.as_ref(), cx, encoder)
    }
}

impl<'de, M> Decode<'de, M> for Box<str>
where
    M: Mode,
{
    #[inline]
    fn decode<'buf, C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<'buf, Input = D::Error>,
        D: Decoder<'de>,
    {
        Ok(<String as Decode<M>>::decode(cx, decoder)?.into())
    }
}

macro_rules! cow {
    ($ty:ty, $source:ty, $decode:ident, $cx:pat, |$owned:ident| $owned_expr:expr, |$borrowed:ident| $borrowed_expr:expr, |$reference:ident| $reference_expr:expr) => {
        impl<M> Encode<M> for Cow<'_, $ty>
        where
            M: Mode,
        {
            #[inline]
            fn encode<'buf, C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
            where
                C: Context<'buf, Input = E::Error>,
                E: Encoder,
            {
                Encode::<M>::encode(self.as_ref(), cx, encoder)
            }
        }

        impl<'de, M> Decode<'de, M> for Cow<'de, $ty>
        where
            M: Mode,
        {
            #[inline]
            fn decode<'buf, C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
            where
                C: Context<'buf, Input = D::Error>,
                D: Decoder<'de>,
            {
                struct Visitor;

                impl<'de, 'buf, C> ValueVisitor<'de, 'buf, C, $source> for Visitor
                where
                    C: Context<'buf>,
                {
                    type Ok = Cow<'de, $ty>;

                    #[inline]
                    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        write!(f, "string")
                    }

                    #[inline]
                    fn visit_owned(
                        self,
                        $cx: &mut C,
                        $owned: <$source as ToOwned>::Owned,
                    ) -> Result<Self::Ok, C::Error> {
                        Ok($owned_expr)
                    }

                    #[inline]
                    fn visit_borrowed(
                        self,
                        $cx: &mut C,
                        $borrowed: &'de $source,
                    ) -> Result<Self::Ok, C::Error> {
                        Ok($borrowed_expr)
                    }

                    #[inline]
                    fn visit_ref(
                        self,
                        $cx: &mut C,
                        $reference: &$source,
                    ) -> Result<Self::Ok, C::Error> {
                        Ok($reference_expr)
                    }
                }

                decoder.$decode(cx, Visitor)
            }
        }
    };
}

cow! {
    str, str, decode_string, _,
    |owned| Cow::Owned(owned),
    |borrowed| Cow::Borrowed(borrowed),
    |reference| Cow::Owned(reference.to_owned())
}

cow! {
    CStr, [u8], decode_bytes, cx,
    |owned| Cow::Owned(CString::from_vec_with_nul(owned).map_err(|error| cx.custom(error))?),
    |borrowed| Cow::Borrowed(CStr::from_bytes_with_nul(borrowed).map_err(|error| cx.custom(error))?),
    |reference| Cow::Owned(CStr::from_bytes_with_nul(reference).map_err(|error| cx.custom(error))?.to_owned())
}

macro_rules! sequence {
    (
        $ty:ident <T $(: $trait0:ident $(+ $trait:ident)*)? $(, $extra:ident: $extra_bound0:ident $(+ $extra_bound:ident)*)*>,
        $insert:ident,
        $access:ident,
        $factory:expr
    ) => {
        impl<M, T $(, $extra)*> Encode<M> for $ty<T $(, $extra)*>
        where
            M: Mode,
            T: Encode<M>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn encode<'buf, C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
            where
                C: Context<'buf, Input = E::Error>,
                E: Encoder,
            {
                let mut seq = encoder.encode_sequence(cx, self.len())?;

                for value in self {
                    let encoder = seq.next(cx)?;
                    value.encode(cx, encoder)?;
                }

                seq.end(cx)
            }
        }

        impl<'de, M, T $(, $extra)*> Decode<'de, M> for $ty<T $(, $extra)*>
        where
            M: Mode,
            T: Decode<'de, M> $(+ $trait0 $(+ $trait)*)*,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn decode<'buf, C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
            where
                C: Context<'buf, Input = D::Error>,
                D: Decoder<'de>,
            {
                let mut $access = decoder.decode_sequence(cx)?;
                let mut out = $factory;

                while let Some(value) = $access.next(cx)? {
                    out.$insert(T::decode(cx, value)?);
                }

                $access.end(cx)?;
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
#[cfg(feature = "std")]
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
        impl<'de, M, K, V $(, $extra)*> Encode<M> for $ty<K, V $(, $extra)*>
        where
            M: Mode,
            K: Encode<M>,
            V: Encode<M>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn encode<'buf, C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
            where
                C: Context<'buf, Input = E::Error>,
                E: Encoder,
            {
                let mut map = encoder.encode_map(cx, self.len())?;

                for (k, v) in self {
                    let mut entry = map.next(cx)?;
                    let first = entry.first(cx)?;
                    k.encode(cx, first)?;
                    let second = entry.second(cx)?;
                    v.encode(cx, second)?;
                    entry.end(cx)?;
                }

                map.end(cx)
            }
        }

        impl<'de, K, V, M $(, $extra)*> Decode<'de, M> for $ty<K, V $(, $extra)*>
        where
            M: Mode,
            K: Decode<'de, M> $(+ $key_bound0 $(+ $key_bound)*)*,
            V: Decode<'de, M>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn decode<'buf, C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
            where
                C: Context<'buf, Input = D::Error>,
                D: Decoder<'de>,
            {
                let mut $access = decoder.decode_map(cx)?;
                let mut out = $with_capacity;

                while let Some(mut entry) = $access.next(cx)? {
                    let key = entry.first(cx).and_then(|key| K::decode(cx, key))?;
                    let value = entry.second(cx).and_then(|value| V::decode(cx, value))?;
                    out.insert(key, value);
                }

                $access.end(cx)?;
                Ok(out)
            }
        }
    }
}

map!(BTreeMap<K: Ord, V>, map, BTreeMap::new());

#[cfg(feature = "std")]
map!(
    HashMap<K: Eq + Hash, V, S: BuildHasher + Default>,
    map,
    HashMap::with_capacity_and_hasher(size_hint::cautious(map.size_hint()), S::default())
);

impl<M> Encode<M> for CString
where
    M: Mode,
{
    #[inline]
    fn encode<'buf, C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<'buf, Input = E::Error>,
        E: Encoder,
    {
        encoder.encode_bytes(cx, self.to_bytes_with_nul())
    }
}

impl<'de, M> Decode<'de, M> for CString
where
    M: Mode,
{
    #[inline]
    fn decode<'buf, C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<'buf, Input = D::Error>,
        D: Decoder<'de>,
    {
        struct Visitor;

        impl<'de, 'buf, C> ValueVisitor<'de, 'buf, C, [u8]> for Visitor
        where
            C: Context<'buf>,
        {
            type Ok = CString;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "a cstring")
            }

            #[inline]
            fn visit_owned(self, cx: &mut C, value: Vec<u8>) -> Result<Self::Ok, C::Error> {
                CString::from_vec_with_nul(value).map_err(|error| cx.custom(error))
            }

            #[inline]
            fn visit_borrowed(self, cx: &mut C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                self.visit_ref(cx, bytes)
            }

            #[inline]
            fn visit_ref(self, cx: &mut C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                Ok(CStr::from_bytes_with_nul(bytes)
                    .map_err(|error| cx.custom(error))?
                    .to_owned())
            }
        }

        decoder.decode_bytes(cx, Visitor)
    }
}
