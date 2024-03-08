use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

use crate::en::SequenceEncoder;
use crate::{Context, Decode, Decoder, Encode, Encoder, Mode};

macro_rules! implement {
    ($ty:ident $(<$type:ident>)? { $($field:ident),* }, $count:expr) => {
        impl<M, $($type)*> Encode<M> for $ty $(<$type>)*
        where
            M: Mode,
            $($type: Encode<M>,)*
        {
            #[inline]
            #[allow(unused_mut)]
            fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
            where
                C: Context<Input = E::Error>,
                E: Encoder,
            {
                let mut tuple = encoder.encode_tuple(cx, $count)?;
                $(
                let $field = tuple.next(cx)?;
                Encode::<M>::encode(&self.$field, cx, $field)?;
                )*
                tuple.end(cx)
            }
        }

        impl<'de, M, $($type)*> Decode<'de, M> for $ty $(<$type>)*
        where
            M: Mode,
            $($type: Decode<'de, M>,)*
        {
            #[inline]
            fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
            where
                C: Context<Input = D::Error>,
                D: Decoder<'de>,
            {
                let ($($field,)*) = Decode::<'de, M>::decode(cx, decoder)?;
                Ok($ty { $($field,)* })
            }
        }
    }
}

macro_rules! implement_new {
    ($ty:ident { $($field:ident),* }, $count:expr) => {
        impl<M, T> Encode<M> for $ty<T>
        where
            M: Mode,
            T: Encode<M>,
        {
            #[inline]
            fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
            where
                C: Context<Input = E::Error>,
                E: Encoder,
            {
                let mut tuple = encoder.encode_tuple(cx, $count)?;
                $(
                let $field = tuple.next(cx)?;
                Encode::<M>::encode(&self.$field(), cx, $field)?;
                )*
                tuple.end(cx)
            }
        }

        impl<'de, M, T> Decode<'de, M> for $ty<T>
        where
            M: Mode,
            T: Decode<'de, M>,
        {
            #[inline]
            fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
            where
                C: Context<Input = D::Error>,
                D: Decoder<'de>,
            {
                let ($($field,)*) = Decode::<'de, M>::decode(cx, decoder)?;
                Ok($ty::new($($field,)*))
            }
        }
    }
}

implement!(RangeFull {}, 0);
implement!(Range<T> { start, end }, 2);
implement!(RangeFrom<T> { start }, 1);
implement!(RangeTo<T> { end }, 1);
implement!(RangeToInclusive<T> { end }, 1);
implement_new!(RangeInclusive { start, end }, 2);
