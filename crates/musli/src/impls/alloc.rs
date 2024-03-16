use core::ffi::CStr;
use core::fmt;
#[cfg(feature = "std")]
use core::hash::{BuildHasher, Hash};

use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BinaryHeap, VecDeque};
use alloc::ffi::CString;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet};
#[cfg(feature = "std")]
use std::ffi::{OsStr, OsString};
#[cfg(feature = "std")]
use std::path::{Path, PathBuf};

use crate::de::{
    Decode, Decoder, MapDecoder, MapEntryDecoder, SequenceDecoder, TraceDecode, ValueVisitor,
};
use crate::en::{Encode, Encoder, MapEncoder, MapEntryEncoder, SequenceEncoder, TraceEncode};
use crate::internal::size_hint;
use crate::Context;

impl<M> Encode<M> for String {
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Mode = M>,
        E: Encoder<C>,
    {
        self.as_str().encode(cx, encoder)
    }
}

impl<'de, M> Decode<'de, M> for String {
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Mode = M>,
        D: Decoder<'de, C>,
    {
        struct Visitor;

        impl<'de, C> ValueVisitor<'de, C, str> for Visitor
        where
            C: Context,
        {
            type Ok = String;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "string")
            }

            #[inline]
            fn visit_owned(self, _: &C, value: String) -> Result<Self::Ok, C::Error> {
                Ok(value)
            }

            #[inline]
            fn visit_borrowed(self, cx: &C, string: &'de str) -> Result<Self::Ok, C::Error> {
                self.visit_ref(cx, string)
            }

            #[inline]
            fn visit_ref(self, _: &C, string: &str) -> Result<Self::Ok, C::Error> {
                Ok(string.to_owned())
            }
        }

        decoder.decode_string(cx, Visitor)
    }
}

impl<M> Encode<M> for Box<str> {
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Mode = M>,
        E: Encoder<C>,
    {
        self.as_ref().encode(cx, encoder)
    }
}

impl<'de, M> Decode<'de, M> for Box<str> {
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Mode = M>,
        D: Decoder<'de, C>,
    {
        let string: String = cx.decode(decoder)?;
        Ok(string.into())
    }
}

macro_rules! cow {
    ($ty:ty, $source:ty, $decode:ident, $cx:pat, |$owned:ident| $owned_expr:expr, |$borrowed:ident| $borrowed_expr:expr, |$reference:ident| $reference_expr:expr) => {
        impl<M> Encode<M> for Cow<'_, $ty> {
            #[inline]
            fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
            where
                C: Context<Mode = M>,
                E: Encoder<C>,
            {
                self.as_ref().encode(cx, encoder)
            }
        }

        impl<'de, M> Decode<'de, M> for Cow<'de, $ty> {
            #[inline]
            fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
            where
                C: Context<Mode = M>,
                D: Decoder<'de, C>,
            {
                struct Visitor;

                impl<'de, C> ValueVisitor<'de, C, $source> for Visitor
                where
                    C: Context,
                {
                    type Ok = Cow<'de, $ty>;

                    #[inline]
                    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        write!(f, "string")
                    }

                    #[inline]
                    fn visit_owned(
                        self,
                        $cx: &C,
                        $owned: <$source as ToOwned>::Owned,
                    ) -> Result<Self::Ok, C::Error> {
                        Ok($owned_expr)
                    }

                    #[inline]
                    fn visit_borrowed(
                        self,
                        $cx: &C,
                        $borrowed: &'de $source,
                    ) -> Result<Self::Ok, C::Error> {
                        Ok($borrowed_expr)
                    }

                    #[inline]
                    fn visit_ref(
                        self,
                        $cx: &C,
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
        $cx:ident,
        $ty:ident <T $(: $trait0:ident $(+ $trait:ident)*)? $(, $extra:ident: $extra_bound0:ident $(+ $extra_bound:ident)*)*>,
        $insert:ident,
        $access:ident,
        $factory:expr
    ) => {
        impl<M, T $(, $extra)*> Encode<M> for $ty<T $(, $extra)*>
        where
            T: Encode<M>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn encode<C, E>(&self, $cx: &C, encoder: E) -> Result<E::Ok, C::Error>
            where
                C: Context<Mode = M>,
                E: Encoder<C>,
            {
                let mut seq = encoder.encode_sequence($cx, self.len())?;

                let mut index = 0;

                for value in self {
                    $cx.enter_sequence_index(index);
                    let encoder = seq.next($cx)?;
                    value.encode($cx, encoder)?;
                    $cx.leave_sequence_index();
                    index = index.wrapping_add(1);
                }

                seq.end($cx)
            }
        }

        impl<'de, M, T $(, $extra)*> Decode<'de, M> for $ty<T $(, $extra)*>
        where
            T: Decode<'de, M> $(+ $trait0 $(+ $trait)*)*,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn decode<C, D>($cx: &C, decoder: D) -> Result<Self, C::Error>
            where
                C: Context<Mode = M>,
                D: Decoder<'de, C>,
            {
                let mut $access = decoder.decode_sequence($cx)?;
                let mut out = $factory;

                let mut index = 0;

                while let Some(value) = $access.next($cx)? {
                    $cx.enter_sequence_index(index);
                    out.$insert(T::decode($cx, value)?);
                    $cx.leave_sequence_index();
                    index = index.wrapping_add(1);
                }

                $access.end($cx)?;
                Ok(out)
            }
        }
    }
}

sequence!(
    cx,
    Vec<T>,
    push,
    seq,
    Vec::with_capacity(size_hint::cautious(seq.size_hint(cx)))
);
sequence!(
    cx,
    VecDeque<T>,
    push_back,
    seq,
    VecDeque::with_capacity(size_hint::cautious(seq.size_hint(cx)))
);
#[cfg(feature = "std")]
sequence!(
    cx,
    HashSet<T: Eq + Hash, S: BuildHasher + Default>,
    insert,
    seq,
    HashSet::with_capacity_and_hasher(size_hint::cautious(seq.size_hint(cx)), S::default())
);
sequence!(
    cx,
    BinaryHeap<T: Ord>,
    push,
    seq,
    BinaryHeap::with_capacity(size_hint::cautious(seq.size_hint(cx)))
);

macro_rules! map {
    (
        $cx:ident,
        $ty:ident<K $(: $key_bound0:ident $(+ $key_bound:ident)*)?, V $(, $extra:ident: $extra_bound0:ident $(+ $extra_bound:ident)*)*>,
        $access:ident,
        $with_capacity:expr
    ) => {
        impl<'de, M, K, V $(, $extra)*> Encode<M> for $ty<K, V $(, $extra)*>
        where
            K: Encode<M>,
            V: Encode<M>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn encode<C, E>(&self, $cx: &C, encoder: E) -> Result<E::Ok, C::Error>
            where
                C: Context<Mode = M>,
                E: Encoder<C>,
            {
                let mut map = encoder.encode_map($cx, self.len())?;

                for (k, v) in self {
                    let mut entry = map.entry($cx)?;
                    k.encode($cx, entry.map_key($cx)?)?;
                    v.encode($cx, entry.map_value($cx)?)?;
                    entry.end($cx)?;
                }

                map.end($cx)
            }
        }

        impl<'de, M, K, V $(, $extra)*> TraceEncode<M> for $ty<K, V $(, $extra)*>
        where
            K: fmt::Display + Encode<M>,
            V: Encode<M>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn trace_encode<C, E>(&self, $cx: &C, encoder: E) -> Result<E::Ok, C::Error>
            where
                C: Context<Mode = M>,
                E: Encoder<C>,
            {
                let mut map = encoder.encode_map($cx, self.len())?;

                for (k, v) in self {
                    $cx.enter_map_key(k);
                    let mut entry = map.entry($cx)?;
                    k.encode($cx, entry.map_key($cx)?)?;
                    v.encode($cx, entry.map_value($cx)?)?;
                    entry.end($cx)?;
                    $cx.leave_map_key();
                }

                map.end($cx)
            }
        }

        impl<'de, K, V, M $(, $extra)*> Decode<'de, M> for $ty<K, V $(, $extra)*>
        where
            K: Decode<'de, M> $(+ $key_bound0 $(+ $key_bound)*)*,
            V: Decode<'de, M>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn decode<C, D>($cx: &C, decoder: D) -> Result<Self, C::Error>
            where
                C: Context<Mode = M>,
                D: Decoder<'de, C>,
            {
                let mut $access = decoder.decode_map($cx)?;
                let mut out = $with_capacity;

                while let Some(mut entry) = $access.entry($cx)? {
                    let key = entry.map_key($cx).and_then(|key| $cx.decode(key))?;
                    let value = entry.map_value($cx).and_then(|value| $cx.decode(value))?;
                    out.insert(key, value);
                }

                $access.end($cx)?;
                Ok(out)
            }
        }

        impl<'de, K, V, M $(, $extra)*> TraceDecode<'de, M> for $ty<K, V $(, $extra)*>
        where
            K: fmt::Display + Decode<'de, M> $(+ $key_bound0 $(+ $key_bound)*)*,
            V: Decode<'de, M>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn trace_decode<C, D>($cx: &C, decoder: D) -> Result<Self, C::Error>
            where
                C: Context<Mode = M>,
                D: Decoder<'de, C>,
            {
                let mut $access = decoder.decode_map($cx)?;
                let mut out = $with_capacity;

                while let Some(mut entry) = $access.entry($cx)? {
                    let key = entry.map_key($cx).and_then(|key| $cx.decode(key))?;
                    $cx.enter_map_key(&key);
                    let value = entry.map_value($cx).and_then(|value| $cx.decode(value))?;
                    out.insert(key, value);
                    $cx.leave_map_key();
                }

                $access.end($cx)?;
                Ok(out)
            }
        }
    }
}

map!(cx, BTreeMap<K: Ord, V>, map, BTreeMap::new());

#[cfg(feature = "std")]
map!(
    cx,
    HashMap<K: Eq + Hash, V, S: BuildHasher + Default>,
    map,
    HashMap::with_capacity_and_hasher(size_hint::cautious(map.size_hint(cx)), S::default())
);

impl<M> Encode<M> for CString {
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context,
        E: Encoder<C>,
    {
        encoder.encode_bytes(cx, self.to_bytes_with_nul())
    }
}

impl<'de, M> Decode<'de, M> for CString {
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context,
        D: Decoder<'de, C>,
    {
        struct Visitor;

        impl<'de, C> ValueVisitor<'de, C, [u8]> for Visitor
        where
            C: Context,
        {
            type Ok = CString;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "a cstring")
            }

            #[inline]
            fn visit_owned(self, cx: &C, value: Vec<u8>) -> Result<Self::Ok, C::Error> {
                CString::from_vec_with_nul(value).map_err(|error| cx.custom(error))
            }

            #[inline]
            fn visit_borrowed(self, cx: &C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                self.visit_ref(cx, bytes)
            }

            #[inline]
            fn visit_ref(self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                Ok(CStr::from_bytes_with_nul(bytes)
                    .map_err(|error| cx.custom(error))?
                    .to_owned())
            }
        }

        decoder.decode_bytes(cx, Visitor)
    }
}

macro_rules! smart_pointer {
    ($ty:ident) => {
        impl<M, T> Encode<M> for $ty<T>
        where
            T: Encode<M>,
        {
            #[inline]
            fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
            where
                C: Context<Mode = M>,
                E: Encoder<C>,
            {
                (**self).encode(cx, encoder)
            }
        }

        impl<'de, M, T> Decode<'de, M> for $ty<T>
        where
            T: Decode<'de, M>,
        {
            #[inline]
            fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
            where
                C: Context<Mode = M>,
                D: Decoder<'de, C>,
            {
                Ok($ty::new(cx.decode(decoder)?))
            }
        }
    };
}

smart_pointer!(Arc);
smart_pointer!(Rc);

#[cfg(feature = "std")]
#[derive(Encode, Decode)]
#[musli(crate)]
enum Tag {
    Unix,
    Windows,
}

#[cfg(all(feature = "std", any(unix, windows)))]
impl<M> Encode<M> for OsStr {
    #[cfg(unix)]
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context,
        E: Encoder<C>,
    {
        use std::os::unix::ffi::OsStrExt;

        use crate::en::VariantEncoder;

        let mut variant = encoder.encode_variant(cx)?;
        Tag::Unix.encode(cx, variant.tag(cx)?)?;
        self.as_bytes().encode(cx, variant.variant(cx)?)?;
        variant.end(cx)
    }

    #[cfg(windows)]
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context,
        E: Encoder<C>,
    {
        use crate::en::VariantEncoder;
        use crate::Buf;
        use std::os::windows::ffi::OsStrExt;

        let mut variant = encoder.encode_variant(cx)?;
        let tag = variant.tag(cx)?;

        Tag::Windows.encode(cx, tag)?;

        let Some(mut buf) = cx.alloc() else {
            return Err(cx.message("Failed to allocate buffer"));
        };

        for w in self.encode_wide() {
            if !buf.write(&w.to_le_bytes()) {
                return Err(cx.message("Failed to write to buffer"));
            }
        }

        buf.as_slice().encode(cx, variant.variant(cx)?)?;
        variant.end(cx)
    }
}

#[cfg(all(feature = "std", any(unix, windows)))]
impl<M> Encode<M> for OsString {
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context,
        E: Encoder<C>,
    {
        self.as_os_str().encode(cx, encoder)
    }
}

#[cfg(all(feature = "std", any(unix, windows)))]
impl<'de, M> Decode<'de, M> for OsString {
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context,
        D: Decoder<'de, C>,
    {
        use crate::de::VariantDecoder;

        let mut variant = decoder.decode_variant(cx)?;

        let tag = variant.tag(cx)?;
        let tag = cx.decode(tag)?;

        match tag {
            #[cfg(not(unix))]
            Tag::Unix => Err(cx.message("Unsupported OsString::Unix variant")),
            #[cfg(unix)]
            Tag::Unix => {
                use std::os::unix::ffi::OsStringExt;
                let bytes = cx.decode(variant.variant(cx)?)?;
                variant.end(cx)?;
                Ok(OsString::from_vec(bytes))
            }
            #[cfg(not(windows))]
            Tag::Windows => Err(cx.message("Unsupported OsString::Windows variant")),
            #[cfg(windows)]
            Tag::Windows => {
                use std::os::windows::ffi::OsStringExt;

                struct Visitor;

                impl<'de, C> ValueVisitor<'de, C, [u8]> for Visitor
                where
                    C: Context,
                {
                    type Ok = OsString;

                    #[inline]
                    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        write!(f, "a literal byte reference")
                    }

                    #[inline]
                    fn visit_ref(self, _: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                        let mut buf = Vec::with_capacity(bytes.len() / 2);

                        for pair in bytes.chunks_exact(2) {
                            let &[a, b] = pair else {
                                continue;
                            };

                            buf.push(u16::from_le_bytes([a, b]));
                        }

                        Ok(OsString::from_wide(&buf))
                    }
                }

                let value = variant.variant(cx)?;
                let os_string = value.decode_bytes(cx, Visitor)?;
                variant.end(cx)?;
                Ok(os_string)
            }
        }
    }
}

#[cfg(all(feature = "std", any(unix, windows)))]
impl<M> Encode<M> for PathBuf {
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context,
        E: Encoder<C>,
    {
        self.as_path().encode(cx, encoder)
    }
}

#[cfg(all(feature = "std", any(unix, windows)))]
impl<M> Encode<M> for Path {
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context,
        E: Encoder<C>,
    {
        self.as_os_str().encode(cx, encoder)
    }
}

#[cfg(all(feature = "std", any(unix, windows)))]
impl<'de, M> Decode<'de, M> for PathBuf {
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context,
        D: Decoder<'de, C>,
    {
        let string: OsString = cx.decode(decoder)?;
        Ok(PathBuf::from(string))
    }
}
