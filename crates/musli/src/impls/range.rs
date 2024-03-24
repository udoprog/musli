use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

use crate::en::SequenceEncoder;
use crate::{Decode, Decoder, Encode, Encoder};

macro_rules! implement {
    ($ty:ident $(<$type:ident>)? { $($field:ident),* }, $count:expr) => {
        impl<M, $($type)*> Encode<M> for $ty $(<$type>)*
        where
            $($type: Encode<M>,)*
        {
            #[inline]
            #[allow(unused)]
            fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder<Mode = M>,
            {
                encoder.encode_tuple_fn($count, |tuple| {
                    $(tuple.encode_next()?.encode(&self.$field)?;)*
                    Ok(())
                })
            }
        }

        impl<'de, M, $($type)*> Decode<'de, M> for $ty $(<$type>)*
        where
            $($type: Decode<'de, M>,)*
        {
            #[inline]
            fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de, Mode = M>,
            {
                let ($($field,)*) = decoder.decode()?;
                Ok($ty { $($field,)* })
            }
        }
    }
}

macro_rules! implement_new {
    ($ty:ident { $($field:ident),* }, $count:expr) => {
        impl<M, T> Encode<M> for $ty<T>
        where
            T: Encode<M>,
        {
            #[inline]
            fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder<Mode = M>,
            {
                encoder.encode_tuple_fn($count, |tuple| {
                    $(tuple.encode_next()?.encode(self.$field())?;)*
                    Ok(())
                })
            }
        }

        impl<'de, M, T> Decode<'de, M> for $ty<T>
        where
            T: Decode<'de, M>,
        {
            #[inline]
            fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de, Mode = M>,
            {
                let ($($field,)*) = Decode::decode(cx, decoder)?;
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
