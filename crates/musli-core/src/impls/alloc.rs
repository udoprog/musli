use core::ffi::CStr;
use core::fmt;
#[cfg(feature = "std")]
use core::hash::{BuildHasher, Hash};
use core::marker::PhantomData;

use rust_alloc::borrow::Cow;
use rust_alloc::boxed::Box;
use rust_alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque};
use rust_alloc::ffi::CString;
use rust_alloc::rc::Rc;
use rust_alloc::string::String;
use rust_alloc::sync::Arc;
use rust_alloc::vec::Vec;

#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet};
#[cfg(all(feature = "std", any(unix, windows)))]
use std::ffi::{OsStr, OsString};
#[cfg(all(feature = "std", any(unix, windows)))]
use std::path::{Path, PathBuf};

use crate::alloc::ToOwned;
use crate::de::{
    Decode, DecodeBytes, DecodeSliceBuilder, DecodeTrace, Decoder, EntryDecoder, MapDecoder,
    SequenceDecoder, UnsizedVisitor,
};
use crate::en::{
    Encode, EncodeBytes, EncodePacked, EncodeTrace, Encoder, EntryEncoder, MapEncoder,
    SequenceEncoder,
};
use crate::hint::{MapHint, SequenceHint};
use crate::internal::size_hint;
use crate::{Allocator, Context};

#[cfg(all(feature = "std", any(unix, windows)))]
use super::PlatformTag;

impl<M> Encode<M> for String {
    const IS_BITWISE_ENCODE: bool = false;

    type Encode = str;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        self.as_str().encode(encoder)
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M, A> Decode<'de, M, A> for String
where
    A: Allocator,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>,
    {
        struct Visitor;

        impl<'de, C> UnsizedVisitor<'de, C, str> for Visitor
        where
            C: Context,
        {
            type Ok = String;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "string")
            }

            #[inline]
            fn visit_borrowed(self, cx: C, string: &'de str) -> Result<Self::Ok, C::Error> {
                self.visit_ref(cx, string)
            }

            #[inline]
            fn visit_ref(self, _: C, string: &str) -> Result<Self::Ok, C::Error> {
                use rust_alloc::borrow::ToOwned;
                Ok(string.to_owned())
            }
        }

        decoder.decode_string(Visitor)
    }
}

impl<'de, M, A> Decode<'de, M, A> for Box<str>
where
    A: Allocator,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>,
    {
        Ok(decoder.decode::<String>()?.into())
    }
}

impl<'de, M, A, T> Decode<'de, M, A> for Box<[T]>
where
    A: Allocator,
    T: Decode<'de, M, A>,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        Ok(decoder.decode::<Vec<T>>()?.into())
    }
}

macro_rules! cow {
    (
        $encode:ident :: $encode_fn:ident,
        $as_encode:ident,
        $decode:ident :: $decode_fn:ident,
        $encode_packed:ident,
        $decode_packed:ident,
        $ty:ty, $source:ty,
        $decode_method:ident, $cx:pat,
        |$owned:ident| $owned_expr:expr,
        |$borrowed:ident| $borrowed_expr:expr,
        |$reference:ident| $reference_expr:expr $(,)?
    ) => {
        impl<M> $encode<M> for Cow<'_, $ty> {
            const $encode_packed: bool = false;

            type $encode = $ty;

            #[inline]
            fn $encode_fn<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder<Mode = M>,
            {
                self.as_ref().$encode_fn(encoder)
            }

            #[inline]
            fn $as_encode(&self) -> &Self::$encode {
                self
            }
        }

        impl<'de, M, A> $decode<'de, M, A> for Cow<'de, $ty>
        where
            A: Allocator,
        {
            const $decode_packed: bool = false;

            #[inline]
            fn $decode_fn<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de, Mode = M, Allocator = A>,
            {
                struct Visitor;

                impl<'de, C> UnsizedVisitor<'de, C, $source> for Visitor
                where
                    C: Context,
                {
                    type Ok = Cow<'de, $ty>;

                    #[inline]
                    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        write!(f, "a string")
                    }

                    #[inline]
                    fn visit_owned(
                        self,
                        $cx: C,
                        $owned: <$source as ToOwned<C::Allocator>>::Owned,
                    ) -> Result<Self::Ok, C::Error> {
                        Ok($owned_expr)
                    }

                    #[inline]
                    fn visit_borrowed(
                        self,
                        $cx: C,
                        $borrowed: &'de $source,
                    ) -> Result<Self::Ok, C::Error> {
                        Ok($borrowed_expr)
                    }

                    #[inline]
                    fn visit_ref(self, $cx: C, $reference: &$source) -> Result<Self::Ok, C::Error> {
                        Ok($reference_expr)
                    }
                }

                decoder.$decode_method(Visitor)
            }
        }
    };
}

cow! {
    Encode::encode,
    as_encode,
    Decode::decode,
    IS_BITWISE_ENCODE,
    IS_BITWISE_DECODE,
    str, str, decode_string, _,
    |owned| {
        match owned.into_std() {
            Ok(owned) => Cow::Owned(owned),
            Err(owned) => {
                Cow::Owned(rust_alloc::borrow::ToOwned::to_owned(owned.as_str()))
            }
        }
    },
    |borrowed| Cow::Borrowed(borrowed),
    |reference| Cow::Owned(rust_alloc::borrow::ToOwned::to_owned(reference))
}

cow! {
    Encode::encode,
    as_encode,
    Decode::decode,
    IS_BITWISE_ENCODE,
    IS_BITWISE_DECODE,
    CStr, [u8], decode_bytes, cx,
    |owned| {
        match owned.into_std() {
            Ok(owned) => Cow::Owned(CString::from_vec_with_nul(owned).map_err(cx.map())?),
            Err(reference) => {
                let value = CStr::from_bytes_with_nul(&reference).map_err(cx.map())?;
                Cow::Owned(rust_alloc::borrow::ToOwned::to_owned(value))
            }
        }
    },
    |borrowed| Cow::Borrowed(CStr::from_bytes_with_nul(borrowed).map_err(cx.map())?),
    |reference| {
        let value = CStr::from_bytes_with_nul(reference).map_err(cx.map())?;
        Cow::Owned(rust_alloc::borrow::ToOwned::to_owned(value))
    }
}

cow! {
    EncodeBytes::encode_bytes,
    as_encode_bytes,
    DecodeBytes::decode_bytes,
    ENCODE_BYTES_PACKED,
    DECODE_BYTES_PACKED,
    [u8], [u8], decode_bytes, _,
    |owned| {
        match owned.into_std() {
            Ok(owned) => Cow::Owned(owned),
            Err(owned) => Cow::Owned(rust_alloc::borrow::ToOwned::to_owned(owned.as_slice())),
        }
    },
    |borrowed| Cow::Borrowed(borrowed),
    |reference| Cow::Owned(rust_alloc::borrow::ToOwned::to_owned(reference)),
}

macro_rules! slice_sequence {
    (
        $(#[$($meta:meta)*])*
        $cx:ident,
        $ty:ident <T $(: $trait0:ident $(+ $trait:ident)*)? $(, $extra:ident: $extra_bound0:ident $(+ $extra_bound:ident)*)*>,
        $insert:ident,
        $access:ident,
    ) => {
        $(#[$($meta)*])*
        impl<M, T $(, $extra)*> Encode<M> for $ty<T $(, $extra)*>
        where
            T: Encode<M>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            const IS_BITWISE_ENCODE: bool = false;

            type Encode = Self;

            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder<Mode = M>,
            {
                encoder.encode_slice(self)
            }

            #[inline]
            fn as_encode(&self) -> &Self::Encode {
                self
            }
        }

        $(#[$($meta)*])*
        impl<'de, M, A, T $(, $extra)*> Decode<'de, M, A> for $ty<T $(, $extra)*>
        where
            A: Allocator,
            T: Decode<'de, M, A> $(+ $trait0 $(+ $trait)*)*,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            const IS_BITWISE_DECODE: bool = false;

            #[inline]
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de, Mode = M, Allocator = A>,
            {
                struct Builder<'de, M, A, T $(, $extra)*>($ty<T $(, $extra)*>, PhantomData<(M, A, &'de ())>);

                impl<'de, M, A, T $(, $extra)*> DecodeSliceBuilder<T> for Builder<'de, M, A, T $(, $extra)*>
                where
                    A: Allocator,
                    T: Decode<'de, M, A> $(+ $trait0 $(+ $trait)*)*,
                    $($extra: $extra_bound0 $(+ $extra_bound)*),*
                {
                    #[inline]
                    fn new<C>(_: C) -> Result<Self, C::Error>
                    where
                        C: Context,
                    {
                        Ok(Builder($ty::new(), PhantomData))
                    }

                    #[inline]
                    fn with_capacity<C>(_: C, size: usize) -> Result<Self, C::Error>
                    where
                        C: Context,
                    {
                        Ok(Builder($ty::with_capacity(size), PhantomData))
                    }

                    #[inline]
                    fn push<C>(&mut self, _: C, value: T) -> Result<(), C::Error>
                    where
                        C: Context,
                    {
                        self.0.$insert(value);
                        Ok(())
                    }

                    #[inline]
                    fn reserve<C>(&mut self, _: C, capacity: usize) -> Result<(), C::Error>
                    where
                        C: Context,
                    {
                        self.0.reserve(capacity);
                        Ok(())
                    }

                    #[inline]
                    unsafe fn set_len(&mut self, len: usize) {
                        self.0.set_len(len);
                    }

                    #[inline]
                    fn as_mut_ptr(&mut self) -> *mut T {
                        self.0.as_mut_ptr()
                    }
                }

                let Builder(value, PhantomData) = decoder.decode_slice()?;
                Ok(value)
            }
        }

        $(#[$($meta)*])*
        impl<M, T $(, $extra)*> EncodePacked<M> for $ty<T $(, $extra)*>
        where
            T: Encode<M>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn encode_packed<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder<Mode = M>,
            {
                encoder.encode_pack_fn(|pack| {
                    pack.encode_slice(self)
                })
            }
        }
    }
}

macro_rules! sequence {
    (
        $(#[$($meta:meta)*])*
        $cx:ident,
        $ty:ident <T $(: $trait0:ident $(+ $trait:ident)*)? $(, $extra:ident: $extra_bound0:ident $(+ $extra_bound:ident)*)*>,
        $insert:ident,
        $access:ident,
        $factory:expr
    ) => {
        $(#[$($meta)*])*
        impl<M, T $(, $extra)*> Encode<M> for $ty<T $(, $extra)*>
        where
            T: Encode<M>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            const IS_BITWISE_ENCODE: bool = false;

            type Encode = Self;

            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder<Mode = M>,
            {
                let hint = SequenceHint::with_size(self.len());

                let $cx = encoder.cx();

                encoder.encode_sequence_fn(&hint, |seq| {
                    let mut index = 0;

                    for value in self {
                        $cx.enter_sequence_index(index);
                        seq.push(value)?;
                        $cx.leave_sequence_index();
                        index = index.wrapping_add(1);
                    }

                    Ok(())
                })
            }

            #[inline]
            fn as_encode(&self) -> &Self::Encode {
                self
            }
        }

        $(#[$($meta)*])*
        impl<'de, M, A, T $(, $extra)*> Decode<'de, M, A> for $ty<T $(, $extra)*>
        where
            A: Allocator,
            T: Decode<'de, M, A> $(+ $trait0 $(+ $trait)*)*,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            const IS_BITWISE_DECODE: bool = false;

            #[inline]
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de, Mode = M, Allocator = A>,
            {
                let $cx = decoder.cx();

                decoder.decode_sequence(|$access| {
                    let mut out = $factory;

                    let mut index = 0;

                    while let Some(value) = $access.try_decode_next()? {
                        $cx.enter_sequence_index(index);
                        out.$insert(value.decode()?);
                        $cx.leave_sequence_index();
                        index = index.wrapping_add(1);
                    }

                    Ok(out)
                })
            }
        }

        $(#[$($meta)*])*
        impl<M, T $(, $extra)*> EncodePacked<M> for $ty<T $(, $extra)*>
        where
            T: Encode<M>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn encode_packed<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder<Mode = M>,
            {
                let $cx = encoder.cx();

                encoder.encode_pack_fn(|pack| {
                    let mut index = 0;

                    for value in self {
                        $cx.enter_sequence_index(index);
                        pack.push(value)?;
                        $cx.leave_sequence_index();
                        index = index.wrapping_add(1);
                    }

                    Ok(())
                })
            }
        }
    }
}

slice_sequence!(cx, Vec<T>, push, seq,);

impl<M, T> Encode<M> for VecDeque<T>
where
    T: Encode<M>,
{
    const IS_BITWISE_ENCODE: bool = false;

    type Encode = Self;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        let (a, b) = self.as_slices();
        encoder.encode_slices(self.len(), [a, b])
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M, A, T> Decode<'de, M, A> for VecDeque<T>
where
    A: Allocator,
    T: Decode<'de, M, A>,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        Ok(VecDeque::from(Vec::decode(decoder)?))
    }
}

impl<M, T> EncodePacked<M> for VecDeque<T>
where
    T: Encode<M>,
{
    #[inline]
    fn encode_packed<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        encoder.encode_pack_fn(|pack| {
            let (a, b) = self.as_slices();
            pack.encode_slices([a, b])
        })
    }
}

sequence!(cx, BTreeSet<T: Ord>, insert, seq, BTreeSet::new());
sequence!(
    #[cfg(feature = "std")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
    cx,
    HashSet<T: Eq + Hash, S: BuildHasher + Default>,
    insert,
    seq,
    HashSet::with_capacity_and_hasher(size_hint::cautious(seq.size_hint()), S::default())
);
sequence!(
    cx,
    BinaryHeap<T: Ord>,
    push,
    seq,
    BinaryHeap::with_capacity(size_hint::cautious(seq.size_hint()))
);

macro_rules! map {
    (
        $(#[$($meta:meta)*])*
        $cx:ident,
        $ty:ident<K $(: $key_bound0:ident $(+ $key_bound:ident)*)?, V $(, $extra:ident: $extra_bound0:ident $(+ $extra_bound:ident)*)*>,
        $access:ident,
        $with_capacity:expr
    ) => {
        $(#[$($meta)*])*
        impl<'de, M, K, V $(, $extra)*> Encode<M> for $ty<K, V $(, $extra)*>
        where
            K: Encode<M>,
            V: Encode<M>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            const IS_BITWISE_ENCODE: bool = false;

            type Encode = Self;

            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder<Mode = M>,
            {
                let hint = MapHint::with_size(self.len());

                encoder.encode_map_fn(&hint, |map| {
                    for (k, v) in self {
                        map.insert_entry(k, v)?;
                    }

                    Ok(())
                })
            }

            #[inline]
            fn as_encode(&self) -> &Self::Encode {
                self
            }
        }

        $(#[$($meta)*])*
        impl<'de, M, K, V $(, $extra)*> EncodeTrace<M> for $ty<K, V $(, $extra)*>
        where
            K: fmt::Display + Encode<M>,
            V: Encode<M>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn trace_encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder<Mode = M>,
            {
                let hint = MapHint::with_size(self.len());

                let $cx = encoder.cx();

                encoder.encode_map_fn(&hint, |map| {
                    for (k, v) in self {
                        $cx.enter_map_key(k);
                        map.encode_entry_fn(|entry| {
                            entry.encode_key()?.encode(k)?;
                            entry.encode_value()?.encode(v)?;
                            Ok(())
                        })?;
                        $cx.leave_map_key();
                    }

                    Ok(())
                })
            }
        }

        $(#[$($meta)*])*
        impl<'de, K, V, A, M $(, $extra)*> Decode<'de, M, A> for $ty<K, V $(, $extra)*>
        where
            A: Allocator,
            K: Decode<'de, M, A> $(+ $key_bound0 $(+ $key_bound)*)*,
            V: Decode<'de, M, A>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            const IS_BITWISE_DECODE: bool = false;

            #[inline]
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de, Mode = M, Allocator = A>,
            {
                decoder.decode_map(|$access| {
                    let mut out = $with_capacity;

                    while let Some((key, value)) = $access.entry()? {
                        out.insert(key, value);
                    }

                    Ok(out)
                })
            }
        }

        $(#[$($meta)*])*
        impl<'de, K, V, A, M $(, $extra)*> DecodeTrace<'de, M, A> for $ty<K, V $(, $extra)*>
        where
            A: Allocator,
            K: fmt::Display + Decode<'de, M, A> $(+ $key_bound0 $(+ $key_bound)*)*,
            V: Decode<'de, M, A>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn trace_decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de, Mode = M, Allocator = A>,
            {
                let $cx = decoder.cx();

                decoder.decode_map(|$access| {
                    let mut out = $with_capacity;

                    while let Some(mut entry) = $access.decode_entry()? {
                        let key = entry.decode_key()?.decode()?;
                        $cx.enter_map_key(&key);
                        let value = entry.decode_value()?.decode()?;
                        out.insert(key, value);
                        $cx.leave_map_key();
                    }

                    Ok(out)
                })
            }
        }
    }
}

map!(_cx, BTreeMap<K: Ord, V>, map, BTreeMap::new());

map!(
    #[cfg(feature = "std")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
    _cx,
    HashMap<K: Eq + Hash, V, S: BuildHasher + Default>,
    map,
    HashMap::with_capacity_and_hasher(size_hint::cautious(map.size_hint()), S::default())
);

impl<M> Encode<M> for CString {
    const IS_BITWISE_ENCODE: bool = false;

    type Encode = CStr;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_bytes(self.to_bytes_with_nul())
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M, A> Decode<'de, M, A> for CString
where
    A: Allocator,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Allocator = A>,
    {
        struct Visitor;

        impl<'de, C> UnsizedVisitor<'de, C, [u8]> for Visitor
        where
            C: Context,
        {
            type Ok = CString;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "a cstring")
            }

            #[inline]
            fn visit_owned(
                self,
                cx: C,
                value: crate::alloc::Vec<u8, C::Allocator>,
            ) -> Result<Self::Ok, C::Error> {
                match value.into_std() {
                    Ok(value) => CString::from_vec_with_nul(value).map_err(cx.map()),
                    Err(value) => self.visit_ref(cx, &value),
                }
            }

            #[inline]
            fn visit_borrowed(self, cx: C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                self.visit_ref(cx, bytes)
            }

            #[inline]
            fn visit_ref(self, cx: C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                let value = CStr::from_bytes_with_nul(bytes).map_err(cx.map())?;
                Ok(rust_alloc::borrow::ToOwned::to_owned(value))
            }
        }

        decoder.decode_bytes(Visitor)
    }
}

macro_rules! smart_pointer {
    ($($ty:ident),* $(,)?) => {
        $(
            impl<M, T> Encode<M> for $ty<T>
            where
                T: ?Sized + Encode<M>,
            {
                const IS_BITWISE_ENCODE: bool = false;

                type Encode = T;

                #[inline]
                fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
                where
                    E: Encoder<Mode = M>,
                {
                    self.as_ref().encode(encoder)
                }

                #[inline]
                fn as_encode(&self) -> &Self::Encode {
                    self
                }
            }

            impl<'de, M, A, T> Decode<'de, M, A> for $ty<T>
            where
                A: Allocator,
                T: Decode<'de, M, A>,
            {
                const IS_BITWISE_DECODE: bool = false;

                #[inline]
                fn decode<D>(decoder: D) -> Result<Self, D::Error>
                where
                    D: Decoder<'de, Mode = M, Allocator = A>,
                {
                    Ok($ty::new(decoder.decode()?))
                }
            }

            impl<'de, M, A> DecodeBytes<'de, M, A> for $ty<[u8]>
            where
                A: Allocator
            {
                const DECODE_BYTES_PACKED: bool = false;

                #[inline]
                fn decode_bytes<D>(decoder: D) -> Result<Self, D::Error>
                where
                    D: Decoder<'de, Mode = M, Allocator = A>,
                {
                    Ok($ty::from(<Vec<u8>>::decode_bytes(decoder)?))
                }
            }

            impl<'de, M, A> Decode<'de, M, A> for $ty<CStr>
            where
                A: Allocator,
            {
                const IS_BITWISE_DECODE: bool = false;

                #[inline]
                fn decode<D>(decoder: D) -> Result<Self, D::Error>
                where
                    D: Decoder<'de, Mode = M, Allocator = A>,
                {
                    Ok($ty::from(CString::decode(decoder)?))
                }
            }

            #[cfg(all(feature = "std", any(unix, windows)))]
            #[cfg_attr(doc_cfg, doc(cfg(all(feature = "std", any(unix, windows)))))]
            impl<'de, M, A> Decode<'de, M, A> for $ty<Path>
            where
                A: Allocator,
                PlatformTag: Decode<'de, M, A>,
            {
                const IS_BITWISE_DECODE: bool = false;

                #[inline]
                fn decode<D>(decoder: D) -> Result<Self, D::Error>
                where
                    D: Decoder<'de, Mode = M, Allocator = A>,
                {
                    Ok($ty::from(PathBuf::decode(decoder)?))
                }
            }

            #[cfg(all(feature = "std", any(unix, windows)))]
            #[cfg_attr(doc_cfg, doc(cfg(all(feature = "std", any(unix, windows)))))]
            impl<'de, M, A> Decode<'de, M, A> for $ty<OsStr>
            where
                A: Allocator,
                PlatformTag: Decode<'de, M, A>,
            {
                const IS_BITWISE_DECODE: bool = false;

                #[inline]
                fn decode<D>(decoder: D) -> Result<Self, D::Error>
                where
                    D: Decoder<'de, Mode = M, Allocator = A>,
                {
                    Ok($ty::from(OsString::decode(decoder)?))
                }
            }
        )*
    };
}

smart_pointer!(Box, Arc, Rc);

#[cfg(all(feature = "std", any(unix, windows)))]
#[cfg_attr(doc_cfg, doc(cfg(all(feature = "std", any(unix, windows)))))]
impl<M> Encode<M> for OsStr
where
    PlatformTag: Encode<M>,
{
    const IS_BITWISE_ENCODE: bool = false;

    type Encode = Self;

    #[cfg(unix)]
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        use std::os::unix::ffi::OsStrExt;

        use crate::en::VariantEncoder;

        encoder.encode_variant_fn(|variant| {
            variant.encode_tag()?.encode(PlatformTag::Unix)?;
            variant.encode_data()?.encode_bytes(self.as_bytes())?;
            Ok(())
        })
    }

    #[cfg(windows)]
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        use std::os::windows::ffi::OsStrExt;

        use crate::alloc::Alloc;
        use crate::en::VariantEncoder;

        let cx = encoder.cx();

        encoder.encode_variant_fn(|variant| {
            let mut buf = cx.alloc().alloc_empty::<u8>();
            let mut len = 0;

            for w in self.encode_wide() {
                let bytes = w.to_le_bytes();

                buf.resize(len, bytes.len()).map_err(cx.map())?;

                // SAFETY: We've just resized the above buffer.
                unsafe {
                    buf.as_mut_ptr()
                        .add(len)
                        .copy_from_nonoverlapping(bytes.as_ptr(), bytes.len());
                }

                len += bytes.len();
            }

            // SAFETY: Slice does not outlive the buffer it references.
            let bytes = unsafe { core::slice::from_raw_parts(buf.as_ptr(), len) };

            variant.encode_tag()?.encode(&PlatformTag::Windows)?;
            variant.encode_data()?.encode_bytes(bytes)?;
            Ok(())
        })
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

#[cfg(all(feature = "std", any(unix, windows)))]
#[cfg_attr(doc_cfg, doc(cfg(all(feature = "std", any(unix, windows)))))]
impl<M> Encode<M> for OsString
where
    PlatformTag: Encode<M>,
{
    const IS_BITWISE_ENCODE: bool = false;

    type Encode = OsStr;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        encoder.encode(self.as_os_str())
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

#[cfg(all(feature = "std", any(unix, windows)))]
#[cfg_attr(doc_cfg, doc(cfg(all(feature = "std", any(unix, windows)))))]
impl<'de, M, A> Decode<'de, M, A> for OsString
where
    A: Allocator,
    PlatformTag: Decode<'de, M, A>,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        use crate::de::VariantDecoder;

        let cx = decoder.cx();

        decoder.decode_variant(|variant| {
            let tag = variant.decode_tag()?.decode::<PlatformTag>()?;

            match tag {
                #[cfg(not(unix))]
                PlatformTag::Unix => Err(cx.message("Unsupported OsString::Unix variant")),
                #[cfg(unix)]
                PlatformTag::Unix => {
                    use std::os::unix::ffi::OsStringExt;
                    Ok(OsString::from_vec(variant.decode_value()?.decode()?))
                }
                #[cfg(not(windows))]
                PlatformTag::Windows => Err(cx.message("Unsupported OsString::Windows variant")),
                #[cfg(windows)]
                PlatformTag::Windows => {
                    use std::os::windows::ffi::OsStringExt;

                    struct Visitor;

                    impl<C> UnsizedVisitor<'_, C, [u8]> for Visitor
                    where
                        C: Context,
                    {
                        type Ok = OsString;

                        #[inline]
                        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                            write!(f, "a literal byte reference")
                        }

                        #[inline]
                        fn visit_ref(self, _: C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
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

                    variant.decode_value()?.decode_bytes(Visitor)
                }
            }
        })
    }
}

#[cfg(all(feature = "std", any(unix, windows)))]
#[cfg_attr(doc_cfg, doc(cfg(all(feature = "std", any(unix, windows)))))]
impl<M> Encode<M> for Path
where
    PlatformTag: Encode<M>,
{
    const IS_BITWISE_ENCODE: bool = false;

    type Encode = Self;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        self.as_os_str().encode(encoder)
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

#[cfg(all(feature = "std", any(unix, windows)))]
#[cfg_attr(doc_cfg, doc(cfg(all(feature = "std", any(unix, windows)))))]
impl<M> Encode<M> for PathBuf
where
    PlatformTag: Encode<M>,
{
    const IS_BITWISE_ENCODE: bool = false;

    type Encode = Self;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        self.as_path().encode(encoder)
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

#[cfg(all(feature = "std", any(unix, windows)))]
#[cfg_attr(doc_cfg, doc(cfg(all(feature = "std", any(unix, windows)))))]
impl<'de, M, A> Decode<'de, M, A> for PathBuf
where
    A: Allocator,
    PlatformTag: Decode<'de, M, A>,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        Ok(PathBuf::from(decoder.decode::<OsString>()?))
    }
}

impl<M> EncodeBytes<M> for Vec<u8> {
    const ENCODE_BYTES_PACKED: bool = false;

    type EncodeBytes = [u8];

    #[inline]
    fn encode_bytes<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        encoder.encode_bytes(self.as_slice())
    }

    #[inline]
    fn as_encode_bytes(&self) -> &Self::EncodeBytes {
        self
    }
}

impl<M> EncodeBytes<M> for Box<[u8]> {
    const ENCODE_BYTES_PACKED: bool = false;

    type EncodeBytes = [u8];

    #[inline]
    fn encode_bytes<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        encoder.encode_bytes(self.as_ref())
    }

    #[inline]
    fn as_encode_bytes(&self) -> &Self::EncodeBytes {
        self
    }
}

impl<'de, M, A> DecodeBytes<'de, M, A> for Vec<u8>
where
    A: Allocator,
{
    const DECODE_BYTES_PACKED: bool = false;

    #[inline]
    fn decode_bytes<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        struct Visitor;

        impl<'de, C> UnsizedVisitor<'de, C, [u8]> for Visitor
        where
            C: Context,
        {
            type Ok = Vec<u8>;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes")
            }

            #[inline]
            fn visit_borrowed(self, _: C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                Ok(bytes.to_vec())
            }

            #[inline]
            fn visit_ref(self, _: C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                Ok(bytes.to_vec())
            }
        }

        decoder.decode_bytes(Visitor)
    }
}

impl<M> EncodeBytes<M> for VecDeque<u8> {
    const ENCODE_BYTES_PACKED: bool = false;

    type EncodeBytes = VecDeque<u8>;

    #[inline]
    fn encode_bytes<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        let (first, second) = self.as_slices();
        encoder.encode_bytes_vectored(self.len(), &[first, second])
    }

    #[inline]
    fn as_encode_bytes(&self) -> &Self::EncodeBytes {
        self
    }
}

impl<'de, M, A> DecodeBytes<'de, M, A> for VecDeque<u8>
where
    A: Allocator,
{
    const DECODE_BYTES_PACKED: bool = false;

    #[inline]
    fn decode_bytes<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        Ok(VecDeque::from(<Vec<u8>>::decode_bytes(decoder)?))
    }
}
