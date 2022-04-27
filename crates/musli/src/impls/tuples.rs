//! Implementations for variously lengthed tuples.

use crate::compat::Packed;
use crate::de::{Decode, Decoder, PackDecoder};
use crate::en::{Encode, Encoder, SequenceEncoder};

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
        impl<$ty0, $($ty),*> Encode for ($ty0, $($ty),*) where $ty0: Encode, $($ty: Encode),* {
            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder
            {
                encoder.encode_tuple(count!($ident0 $($ident)*), |mut pack| {
                    let ($ident0, $($ident),*) = self;
                    <$ty0>::encode($ident0, pack.next()?)?;
                    $(<$ty>::encode($ident, pack.next()?)?;)*
                    pack.end()
                })
            }
        }

        impl<'de, $ty0, $($ty,)*> Decode<'de> for ($ty0, $($ty),*) where $ty0: Decode<'de>, $($ty: Decode<'de>),* {
            #[inline]
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>
            {
                decoder.decode_tuple(count!($ident0 $($ident)*), |mut unpack| {
                    let $ident0 = unpack.next().and_then(<$ty0>::decode)?;
                    $(let $ident = unpack.next().and_then(<$ty>::decode)?;)*
                    Ok(($ident0, $($ident),*))
                })
            }
        }

        impl<$ty0, $($ty),*> Encode for Packed<($ty0, $($ty),*)> where $ty0: Encode, $($ty: Encode),* {
            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder
            {
                encoder.encode_pack(|mut pack| {
                    let Packed(($ident0, $($ident),*)) = self;
                    <$ty0>::encode($ident0, pack.next()?)?;
                    $(<$ty>::encode($ident, pack.next()?)?;)*
                    pack.end()
                })
            }
        }

        impl<'de, $ty0, $($ty,)*> Decode<'de> for Packed<($ty0, $($ty),*)> where $ty0: Decode<'de>, $($ty: Decode<'de>),* {
            #[inline]
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>
            {
                decoder.decode_pack(|mut unpack| {
                    let $ident0 = unpack.next().and_then(<$ty0>::decode)?;
                    $(let $ident = unpack.next().and_then(<$ty>::decode)?;)*
                    Ok(Packed(($ident0, $($ident),*)))
                })
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
