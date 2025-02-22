/// Test if the current options and `$t` is suitable for bitwise slice decoding
/// under the given options.
macro_rules! __is_bitwise_decode {
    // Test if the given slice can be bitwise decoded.
    ($opt:ident, [$t:ty]) => {
        const {
            $crate::options::is_native_fixed::<$opt>()
                && <$t as $crate::de::Decode<_, _>>::IS_BITWISE_DECODE
                && ::core::mem::size_of::<$t>() % ::core::mem::align_of::<$t>() == 0
        }
    };

    // Test if the given type can be bitwise decoded.
    ($opt:ident, $t:ty) => {
        const {
            $crate::options::is_native_fixed::<$opt>()
                && <$t as $crate::de::Decode<_, _>>::IS_BITWISE_DECODE
        }
    };
}

pub(super) use __is_bitwise_decode as is_bitwise_decode;

/// Test if the current options and `$t` is suitable for bitwise slice encoding
/// under the given options.
macro_rules! __is_bitwise_encode {
    // Test if the given slice can be bitwise encode.
    ($opt:ident, [$t:ty]) => {
        const {
            $crate::options::is_native_fixed::<$opt>()
                && <$t as $crate::en::Encode<_>>::IS_BITWISE_ENCODE
                && ::core::mem::size_of::<$t>() % ::core::mem::align_of::<$t>() == 0
        }
    };

    // Test if the given type can be bitwise encode.
    ($opt:ident, $t:ty) => {
        const {
            $crate::options::is_native_fixed::<$opt>()
                && <$t as $crate::en::Encode<_>>::IS_BITWISE_ENCODE
        }
    };
}

pub(super) use __is_bitwise_encode as is_bitwise_encode;
