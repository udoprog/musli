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
        impl<Mode, $ty0 $(, $ty)*> Encode<Mode> for ($ty0, $($ty),*) where $ty0: Encode<Mode>, $($ty: Encode<Mode>),* {
            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder,
            {
                let mut pack = encoder.encode_tuple(count!($ident0 $($ident)*))?;
                let ($ident0, $($ident),*) = self;
                <$ty0>::encode($ident0, pack.next()?)?;
                $(<$ty>::encode($ident, pack.next()?)?;)*
                pack.end()
            }
        }

        impl<'de, Mode, $ty0, $($ty,)*> Decode<'de, Mode> for ($ty0, $($ty),*) where $ty0: Decode<'de, Mode>, $($ty: Decode<'de, Mode>),* {
            #[inline]
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>
            {
                let mut unpack = decoder.decode_tuple(count!($ident0 $($ident)*))?;
                let $ident0 = unpack.next().and_then(<$ty0>::decode)?;
                $(let $ident = unpack.next().and_then(<$ty>::decode)?;)*
                Ok(($ident0, $($ident),*))
            }
        }

        impl<Mode, $ty0 $(,$ty)*> Encode<Mode> for Packed<($ty0, $($ty),*)> where $ty0: Encode<Mode>, $($ty: Encode<Mode>),* {
            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder,
            {
                let Packed(($ident0, $($ident),*)) = self;
                let mut pack = encoder.encode_pack()?;
                <$ty0>::encode($ident0, pack.next()?)?;
                $(<$ty>::encode($ident, pack.next()?)?;)*
                pack.end()
            }
        }

        impl<'de, Mode, $ty0, $($ty,)*> Decode<'de, Mode> for Packed<($ty0, $($ty),*)> where $ty0: Decode<'de, Mode>, $($ty: Decode<'de, Mode>),* {
            #[inline]
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>
            {
                let mut unpack = decoder.decode_pack()?;
                let $ident0 = unpack.next().and_then(<$ty0>::decode)?;
                $(let $ident = unpack.next().and_then(<$ty>::decode)?;)*
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
