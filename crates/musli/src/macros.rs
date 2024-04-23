/// Generate extensions assuming an encoding has implemented encode_with.
#[doc(hidden)]
#[macro_export]
macro_rules! encode_with_extensions {
    ($mode:ident) => {
        /// Encode the given value to the given [`Writer`] using the current
        /// configuration.
        #[inline]
        pub fn encode<W, T>(self, writer: W, value: &T) -> Result<(), Error>
        where
            W: Writer,
            T: ?Sized + Encode<$mode>,
        {
            $crate::default_allocator!(|alloc| {
                let cx = $crate::context::Same::new(alloc);
                self.encode_with(&cx, writer, value)
            })
        }

        /// Encode the given value to the given [Write][io::Write] using the current
        /// configuration.
        #[cfg(feature = "std")]
        #[inline]
        pub fn to_writer<W, T>(self, write: W, value: &T) -> Result<(), Error>
        where
            W: io::Write,
            T: ?Sized + Encode<$mode>,
        {
            let writer = $crate::wrap::wrap(write);
            self.encode(writer, value)
        }

        /// Encode the given value to the given [Write][io::Write] using the current
        /// configuration and context `C`.
        #[cfg(feature = "std")]
        #[inline]
        pub fn to_writer_with<C, W, T>(self, cx: &C, write: W, value: &T) -> Result<(), C::Error>
        where
            C: ?Sized + Context<Mode = $mode>,
            W: io::Write,
            T: ?Sized + Encode<$mode>,
        {
            let writer = $crate::wrap::wrap(write);
            self.encode_with(cx, writer, value)
        }

        /// Encode the given value to a [`Vec`] using the current configuration.
        #[cfg(feature = "alloc")]
        #[inline]
        pub fn to_vec<T>(self, value: &T) -> Result<Vec<u8>, Error>
        where
            T: ?Sized + Encode<$mode>,
        {
            let mut vec = Vec::new();
            self.encode(&mut vec, value)?;
            Ok(vec)
        }

        /// Encode the given value to a [`Vec`] using the current configuration.
        ///
        /// This is the same as [`Encoding::to_vec`], but allows for using a
        /// configurable [`Context`].
        #[cfg(feature = "alloc")]
        #[inline]
        pub fn to_vec_with<C, T>(self, cx: &C, value: &T) -> Result<Vec<u8>, C::Error>
        where
            C: ?Sized + Context<Mode = $mode>,
            T: ?Sized + Encode<$mode>,
        {
            let mut vec = Vec::new();
            self.encode_with(cx, &mut vec, value)?;
            Ok(vec)
        }

        /// Encode the given value to a fixed-size bytes using the current
        /// configuration.
        #[inline]
        pub fn to_fixed_bytes<const N: usize, T>(self, value: &T) -> Result<FixedBytes<N>, Error>
        where
            T: ?Sized + Encode<$mode>,
        {
            $crate::default_allocator!(|alloc| {
                let cx = $crate::context::Same::new(alloc);
                self.to_fixed_bytes_with(&cx, value)
            })
        }

        /// Encode the given value to a fixed-size bytes using the current
        /// configuration.
        #[inline]
        pub fn to_fixed_bytes_with<C, const N: usize, T>(
            self,
            cx: &C,
            value: &T,
        ) -> Result<FixedBytes<N>, C::Error>
        where
            C: ?Sized + Context<Mode = $mode>,
            T: ?Sized + Encode<$mode>,
        {
            let mut bytes = FixedBytes::new();
            self.encode_with(cx, &mut bytes, value)?;
            Ok(bytes)
        }
    };
}

/// Generate all public encoding helpers.
#[doc(hidden)]
#[macro_export]
macro_rules! encoding_from_slice_impls {
    ($mode:ident) => {
        /// Decode the given type `T` from the given slice using the current
        /// configuration.
        #[inline]
        pub fn from_slice<'de, T>(self, bytes: &'de [u8]) -> Result<T, Error>
        where
            T: Decode<'de, $mode>,
        {
            $crate::default_allocator!(|alloc| {
                let cx = $crate::context::Same::new(alloc);
                self.from_slice_with(&cx, bytes)
            })
        }

        /// Decode the given type `T` from the given slice using the current
        /// configuration.
        ///
        /// This is the same as [`Encoding::from_slice`], but allows for using a
        /// configurable [`Context`].
        #[inline]
        pub fn from_slice_with<'de, C, T>(self, cx: &C, bytes: &'de [u8]) -> Result<T, C::Error>
        where
            C: ?Sized + Context<Mode = $mode>,
            T: Decode<'de, $mode>,
        {
            let reader = $crate::reader::SliceReader::new(bytes);
            self.decode_with(cx, reader)
        }
    };
}

/// Generate all public encoding helpers.
#[doc(hidden)]
#[macro_export]
macro_rules! encoding_impls {
    ($mode:ident, $encoder_new:path, $decoder_new:path) => {
        /// Encode the given value to the given [`Writer`] using the current
        /// configuration.
        ///
        /// This is the same as [`Encoding::encode`] but allows for using a
        /// configurable [`Context`].
        #[inline]
        pub fn encode_with<C, W, T>(self, cx: &C, writer: W, value: &T) -> Result<(), C::Error>
        where
            C: ?Sized + Context<Mode = $mode>,
            W: Writer,
            T: ?Sized + Encode<$mode>,
        {
            cx.clear();
            T::encode(value, cx, $encoder_new(cx, writer))
        }

        /// Decode the given type `T` from the given [`Reader`] using the
        /// current configuration.
        ///
        /// This is the same as [`Encoding::decode`] but allows for using a
        /// configurable [`Context`].
        #[inline]
        pub fn decode_with<'de, C, R, T>(self, cx: &C, reader: R) -> Result<T, C::Error>
        where
            C: ?Sized + Context<Mode = $mode>,
            R: Reader<'de>,
            T: Decode<'de, $mode>,
        {
            cx.clear();
            T::decode(cx, $decoder_new(cx, reader))
        }

        /// Decode the given type `T` from the given [`Reader`] using the
        /// current configuration.
        #[inline]
        pub fn decode<'de, R, T>(self, reader: R) -> Result<T, Error>
        where
            R: Reader<'de>,
            T: Decode<'de, $mode>,
        {
            $crate::default_allocator!(|alloc| {
                let cx = $crate::context::Same::new(alloc);
                self.decode_with(&cx, reader)
            })
        }

        $crate::encode_with_extensions!($mode);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! test_include_if {
    (#[musli_value] => $($rest:tt)*) => { $($rest)* };
    (=> $($_:tt)*) => {};
}

/// Generate test functions which provides rich diagnostics when they fail.
#[doc(hidden)]
#[macro_export]
macro_rules! test_fns {
    ($what:expr, $mode:ty $(, $(#[$option:ident])*)?) => {
        /// Roundtrip encode the given value.
        #[doc(hidden)]
        #[track_caller]
        #[cfg(feature = "test")]
        pub fn rt<T>(value: T) -> T
        where
            T: ::musli_core::en::Encode<$mode> + ::musli_core::de::DecodeOwned<$mode>,
            T: ::core::fmt::Debug + ::core::cmp::PartialEq,
        {
            const WHAT: &str = $what;
            const ENCODING: super::Encoding = super::Encoding::new();

            use ::core::any::type_name;

            struct FormatBytes<'a>(&'a [u8]);

            impl ::core::fmt::Display for FormatBytes<'_> {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    write!(f, "b\"")?;

                    for b in self.0 {
                        if b.is_ascii_graphic() {
                            write!(f, "{}", *b as char)?;
                        } else {
                            write!(f, "\\x{b:02x}")?;
                        }
                    }

                    write!(f, "\" (0-{})", self.0.len())?;
                    Ok(())
                }
            }

            default_allocator!(|alloc| {
                let mut cx = $crate::context::SystemContext::new(&alloc);
                cx.include_type();

                let out = match ENCODING.to_vec_with(&cx, &value) {
                    Ok(out) => out,
                    Err(..) => {
                        let error = cx.report();
                        panic!("{WHAT}: {}: failed to encode:\n{error}", type_name::<T>())
                    }
                };

                let decoded: T = match ENCODING.from_slice_with(&cx, out.as_slice()) {
                    Ok(decoded) => decoded,
                    Err(..) => {
                        let out = FormatBytes(&out);
                        let error = cx.report();
                        panic!("{WHAT}: {}: failed to decode:\nValue: {value:?}\nBytes: {out}\n{error}", type_name::<T>())
                    }
                };

                assert_eq!(decoded, value, "{WHAT}: {}: roundtrip does not match\nValue: {value:?}", type_name::<T>());

                $crate::test_include_if! {
                    $($(#[$option])*)* =>
                    let value_decode: $crate::value::Value = match ENCODING.from_slice_with(&cx, out.as_slice()) {
                        Ok(decoded) => decoded,
                        Err(..) => {
                            let out = FormatBytes(&out);
                            let error = cx.report();
                            panic!("{WHAT}: {}: failed to decode to value type:\nValue: {value:?}\nBytes:{out}\n{error}", type_name::<T>())
                        }
                    };

                    let value_decoded: T = match $crate::value::decode_with(&cx, &value_decode) {
                        Ok(decoded) => decoded,
                        Err(..) => {
                            let out = FormatBytes(&out);
                            let error = cx.report();
                            panic!("{WHAT}: {}: failed to decode from value type:\nValue: {value:?}\nBytes: {out}\nBuffered value: {value_decode:?}\n{error}", type_name::<T>())
                        }
                    };

                    assert_eq!(value_decoded, value, "{WHAT}: {}: musli-value roundtrip does not match\nValue: {value:?}", type_name::<T>());
                }

                decoded
            })
        }

        /// Encode and then decode the given value once.
        #[doc(hidden)]
        #[track_caller]
        #[cfg(feature = "test")]
        pub fn decode<'de, T, U>(value: T, out: &'de mut ::alloc::vec::Vec<u8>, expected: &U) -> U
        where
            T: ::musli_core::en::Encode<$mode>,
            T: ::core::fmt::Debug + ::core::cmp::PartialEq,
            U: ::musli_core::de::Decode<'de, $mode>,
            U: ::core::fmt::Debug + ::core::cmp::PartialEq,
        {
            const WHAT: &str = $what;
            const ENCODING: super::Encoding = super::Encoding::new();

            use ::core::any::type_name;

            struct FormatBytes<'a>(&'a [u8]);

            impl ::core::fmt::Display for FormatBytes<'_> {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    write!(f, "b\"")?;

                    for b in self.0 {
                        if b.is_ascii_graphic() {
                            write!(f, "{}", *b as char)?;
                        } else {
                            write!(f, "\\x{b:02x}")?;
                        }
                    }

                    write!(f, "\" (0-{})", self.0.len())?;
                    Ok(())
                }
            }

            default_allocator!(|alloc| {
                let mut cx = $crate::context::SystemContext::new(&alloc);
                cx.include_type();

                out.clear();

                match ENCODING.to_writer_with(&cx, &mut *out, &value) {
                    Ok(()) => (),
                    Err(..) => {
                        let error = cx.report();
                        panic!("{WHAT}: {}: failed to encode:\n{error}", type_name::<T>())
                    }
                };

                let actual = match ENCODING.from_slice_with(&cx, &*out) {
                    Ok(decoded) => decoded,
                    Err(..) => {
                        let out = FormatBytes(&*out);
                        let error = cx.report();
                        panic!("{WHAT}: {}: failed to decode:\nValue: {value:?}\nBytes: {out}\n{error}", type_name::<U>())
                    }
                };

                assert_eq!(
                    actual,
                    *expected,
                    "{WHAT}: decoded value does not match expected\nBytes: {}",
                    FormatBytes(&*out),
                );

                actual
            })
        }

        /// Encode a value to bytes.
        #[doc(hidden)]
        #[track_caller]
        #[cfg(feature = "test")]
        pub fn to_vec<T>(value: T) -> ::alloc::vec::Vec<u8>
        where
            T: ::musli_core::en::Encode<$mode>,
        {
            const WHAT: &str = $what;
            const ENCODING: super::Encoding = super::Encoding::new();

            use ::core::any::type_name;

            default_allocator!(|alloc| {
                let mut cx = $crate::context::SystemContext::new(alloc);
                cx.include_type();

                match ENCODING.to_vec_with(&cx, &value) {
                    Ok(out) => out,
                    Err(..) => {
                        let error = cx.report();
                        panic!("{WHAT}: {}: failed to encode:\n{error}", type_name::<T>())
                    }
                }
            })
        }
    }
}

#[cfg(all(feature = "test", feature = "alloc"))]
#[doc(hidden)]
pub mod support {
    use crate::mode::Binary;
    use crate::value::{self, Value};
    use crate::{Decode, Encode};

    pub use alloc::vec::Vec;

    #[track_caller]
    pub fn musli_value_rt<T>(expected: T)
    where
        T: Encode<Binary> + for<'de> Decode<'de, Binary>,
        T: PartialEq + core::fmt::Debug,
    {
        let value: Value = value::encode(&expected).expect("value: Encoding should succeed");
        let actual: T = value::decode(&value).expect("value: Decoding should succeed");
        assert_eq!(
            actual, expected,
            "value: roundtripped value does not match expected"
        );
    }
}

/// Roundtrip the given expression through all supported formats.
#[cfg(feature = "test")]
#[macro_export]
#[doc(hidden)]
macro_rules! rt {
    ($what:ident, $expr:expr $(, $($extra:tt)*)?) => {{
        let expected = $expr;

        macro_rules! rt {
            ($name:ident) => {{
                assert_eq!(
                    $crate::$name::test::rt($expr), expected,
                    "{}: roundtripped value does not match expected",
                    stringify!($name),
                );
            }}
        }

        $crate::test_matrix!($what, rt);
        $crate::macros::support::musli_value_rt($expr);
        $crate::extra!($expr $(, $($extra)*)*);
        expected
    }};
}

/// This is used to test when there is a decode assymmetry, such as the decoded
/// value does not match the encoded one due to things such as skipped fields.
#[cfg(feature = "test")]
#[macro_export]
macro_rules! assert_decode_eq {
    ($what:ident, $expr:expr, $expected:expr $(, $($extra:tt)*)?) => {{
        let mut bytes = $crate::macros::support::Vec::<u8>::new();

        macro_rules! decode {
            ($name:ident) => {{
                $crate::$name::test::decode($expr, &mut bytes, &$expected);
            }}
        }

        $crate::test_matrix!($what, decode);
        $crate::extra!($expr $(, $($extra)*)*);
    }};
}

#[cfg(feature = "test")]
#[macro_export]
macro_rules! extra {
    ($expr:expr $(,)?) => {};

    ($expr:expr, json = $json_expected:expr $(, $($tt:tt)*)?) => {{
        let json = $crate::json::test::to_vec($expr);
        let string = ::std::string::String::from_utf8(json).expect("Encoded JSON is not valid utf-8");

        assert_eq!(
            string, $json_expected,
            "json: encoded json does not match expected value"
        );

        $crate::extra!($expr $(, $($tt)*)*);
    }};
}

#[cfg(feature = "test")]
#[macro_export]
macro_rules! test_matrix {
    (full, $call:path) => {
        $call!(storage);
        $call!(wire);
        $call!(descriptive);
        $call!(json);
    };

    (no_json, $call:path) => {
        $call!(storage);
        $call!(wire);
        $call!(descriptive);
    };

    (descriptive, $call:path) => {
        $call!(descriptive);
        $call!(json);
    };

    // TODO: Deprecate this in favor of only using `upgrade_stable`.
    (wire_only, $call:path) => {
        $call!(wire);
    };

    (json, $call:path) => {
        $call!(json);
    };

    (upgrade_stable, $call:path) => {
        $call!(wire);
        $call!(descriptive);
        $call!(json);
    };

    (upgrade_stable_no_text, $call:path) => {
        $call!(wire);
        $call!(descriptive);
    };
}
