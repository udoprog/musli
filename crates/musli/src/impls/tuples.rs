//! Implementations for variously lengthed tuples.

use crate::de::{Decode, Decoder, PackDecoder};
use crate::en::{Encode, Encoder, PackEncoder};

macro_rules! declare {
    () => {
    };

    ($ty0:ident, $ident0:ident $(,)? $($ty:ident, $ident:ident),*) => {
        impl<$ty0, $($ty),*> Encode for ($ty0, $($ty),*) where $ty0: Encode, $($ty: Encode),* {
            fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
            where
                E: Encoder
            {
                let ($ident0, $($ident),*) = self;
                let mut pack = encoder.encode_pack()?;
                <$ty0>::encode($ident0, pack.next()?)?;
                $(<$ty>::encode($ident, pack.next()?)?;)*
                Ok(())
            }
        }

        impl<'de, $ty0, $($ty,)*> Decode<'de> for ($ty0, $($ty),*) where $ty0: Decode<'de>, $($ty: Decode<'de>),* {
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>
            {
                let mut unpack = decoder.decode_pack()?;
                let $ident0 = unpack.next().and_then(<$ty0>::decode)?;
                $(let $ident = unpack.next().and_then(<$ty>::decode)?;)*
                Ok(($ident0, $($ident),*))
            }
        }

        declare!($($ty, $ident),*);
    };
}

declare!(T0, t0, T1, t1, T2, t2, T3, t3, T4, t4, T5, t5, T6, t6, T7, t7, T8, t8, T9, t9, T10, t10);
