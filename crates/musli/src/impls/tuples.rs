//! Implementations for variously lengthed tuples.

use crate::compat::Packed;
use crate::de::{Decode, Decoder, PackDecoder};
use crate::en::{Encode, Encoder, SequenceEncoder};
use crate::mode::Mode;
use crate::Context;

macro_rules! count {
    (_) => { 1 };
    (_ _) => { 2 };
    (_ _ _) => { 3 };
    (_ _ _ _) => { 4 };
    (_ _ _ _ _) => { 5 };
    (_ _ _ _ _ _) => { 6 };
    (_ _ _ _ _ _ _) => { 7 };
    (_ _ _ _ _ _ _ _) => { 8 };
    (_ _ _ _ _ _ _ _ _) => { 9 };
    (_ _ _ _ _ _ _ _ _ _) => { 10 };
    (_ _ _ _ _ _ _ _ _ _ _) => { 11 };
    (_ _ _ _ _ _ _ _ _ _ _ _) => { 12 };
    (_ _ _ _ _ _ _ _ _ _ _ _ _) => { 13 };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _) => { 14 };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => { 15 };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => { 16 };

    (( $($s:tt)* ) $_:ident $($tail:tt)*) => {
        count!(( $($s)* _ ) $($tail)*)
    };

    (( $($s:tt)* )) => {
        count!( $($s)* )
    };

    ($($ident:ident)*) => {
        count!(() $($ident)*)
    };
}

macro_rules! declare {
    () => {
    };

    (($ty0:ident, $ident0:ident) $(, ($ty:ident, $ident:ident))* $(,)?) => {
        impl<M, $ty0 $(, $ty)*> Encode<M> for ($ty0, $($ty),*) where M: Mode, $ty0: Encode<M>, $($ty: Encode<M>),* {
            #[inline]
            fn encode<'buf, C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
            where
                C: Context<'buf, Input = E::Error>,
                E: Encoder,
            {
                let mut pack = encoder.encode_tuple(cx, count!($ident0 $($ident)*))?;
                let ($ident0, $($ident),*) = self;
                let value = pack.next(cx)?;
                <$ty0>::encode($ident0, cx, value)?;
                $(
                    let value = pack.next(cx)?;
                    <$ty>::encode($ident, cx, value)?;
                )*
                pack.end(cx)
            }
        }

        impl<'de, M, $ty0, $($ty,)*> Decode<'de, M> for ($ty0, $($ty),*) where M: Mode, $ty0: Decode<'de, M>, $($ty: Decode<'de, M>),* {
            #[inline]
            fn decode<'buf, C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
            where
                C: Context<'buf, Input = D::Error>,
                D: Decoder<'de>
            {
                let mut unpack = decoder.decode_tuple(cx, count!($ident0 $($ident)*))?;
                let $ident0 = unpack.next(cx).and_then(|v| <$ty0>::decode(cx, v))?;
                $(let $ident = unpack.next(cx).and_then(|v| <$ty>::decode(cx, v))?;)*
                unpack.end(cx)?;
                Ok(($ident0, $($ident),*))
            }
        }

        impl<M, $ty0 $(,$ty)*> Encode<M> for Packed<($ty0, $($ty),*)> where M: Mode, $ty0: Encode<M>, $($ty: Encode<M>),* {
            #[inline]
            fn encode<'buf, C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
            where
                C: Context<'buf, Input = E::Error>,
                E: Encoder,
            {
                let Packed(($ident0, $($ident),*)) = self;
                let mut pack = encoder.encode_pack(cx)?;
                let value = pack.next(cx)?;
                <$ty0>::encode($ident0, cx, value)?;
                $(
                    let value = pack.next(cx)?;
                    <$ty>::encode($ident, cx, value)?;
                )*
                pack.end(cx)
            }
        }

        impl<'de, M, $ty0, $($ty,)*> Decode<'de, M> for Packed<($ty0, $($ty),*)> where M: Mode, $ty0: Decode<'de, M>, $($ty: Decode<'de, M>),* {
            #[inline]
            fn decode<'buf, C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
            where
                C: Context<'buf, Input = D::Error>,
                D: Decoder<'de>
            {
                let mut unpack = decoder.decode_pack(cx)?;
                let $ident0 = unpack.next(cx).and_then(|v| <$ty0>::decode(cx, v))?;
                $(let $ident = unpack.next(cx).and_then(|v| <$ty>::decode(cx, v))?;)*
                unpack.end(cx)?;
                Ok(Packed(($ident0, $($ident),*)))
            }
        }

        declare!($(($ty, $ident)),*);
    };
}

declare! {
    (T0, t0),
    (T1, t1),
    (T2, t2),
    (T3, t3),
    (T4, t4),
    (T5, t5),
    (T6, t6),
    (T7, t7),
    (T8, t8),
    (T9, t9),
    (T10, t10),
    (T11, t11),
    (T12, t12),
    (T13, t13),
    (T14, t14),
    (T15, t15),
}
