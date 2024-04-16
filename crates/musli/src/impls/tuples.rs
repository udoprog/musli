//! Implementations for variously lengthed tuples.

use crate::de::{Decode, DecodePacked, Decoder, PackDecoder, SequenceDecoder};
use crate::en::{Encode, EncodePacked, Encoder, PackEncoder, SequenceEncoder};
use crate::hint::SequenceHint;

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
        impl<M, $ty0 $(, $ty)*> Encode<M> for ($ty0, $($ty),*)
        where
            $ty0: Encode<M>,
            $($ty: Encode<M>),*
        {
            #[inline]
            fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder<Mode = M>,
            {
                static HINT: SequenceHint = SequenceHint::with_size(count!($ident0 $($ident)*));

                encoder.encode_sequence_fn(&HINT, |tuple| {
                    let ($ident0, $($ident),*) = self;
                    tuple.encode_element()?.encode($ident0)?;
                    $(tuple.encode_element()?.encode($ident)?;)*
                    Ok(())
                })
            }
        }

        impl<'de, M, $ty0, $($ty,)*> Decode<'de, M> for ($ty0, $($ty),*)
        where
            $ty0: Decode<'de, M>,
            $($ty: Decode<'de, M>),*
        {
            #[inline]
            fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de, Mode = M>,
            {
                static HINT: SequenceHint = SequenceHint::with_size(count!($ident0 $($ident)*));

                decoder.decode_sequence_hint(&HINT, |tuple| {
                    let $ident0 = tuple.required_next(cx)?;
                    $(let $ident = tuple.required_next(cx)?;)*
                    Ok(($ident0, $($ident),*))
                })
            }
        }

        impl<M, $ty0 $(,$ty)*> EncodePacked<M> for ($ty0, $($ty),*)
        where
            $ty0: Encode<M>,
            $($ty: Encode<M>),*
        {
            #[inline]
            fn encode_packed<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder<Mode = M>,
            {
                let ($ident0, $($ident),*) = self;
                encoder.encode_pack_fn(|pack| {
                    pack.encode_packed()?.encode($ident0)?;
                    $(pack.encode_packed()?.encode($ident)?;)*
                    Ok(())
                })
            }
        }

        impl<'de, M, $ty0, $($ty,)*> DecodePacked<'de, M> for ($ty0, $($ty),*)
        where
            $ty0: Decode<'de, M>,
            $($ty: Decode<'de, M>),*
        {
            #[inline]
            fn decode_packed<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de, Mode = M>,
            {
                decoder.decode_pack(|pack| {
                    let $ident0 = pack.next()?;
                    $(let $ident = pack.next()?;)*
                    Ok(($ident0, $($ident),*))
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
