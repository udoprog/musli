//! Helper macros for use with Musli.

macro_rules! test_include_if {
    (#[musli_value] => $($rest:tt)*) => { $($rest)* };
    (=> $($_:tt)*) => {};
}

pub(crate) use test_include_if;

/// Generate test functions which provides rich diagnostics when they fail.
macro_rules! test_fns {
    ($mode:ident, $what:expr $(, $(#[$option:ident])*)?) => {
        /// Roundtrip encode the given value.
        #[doc(hidden)]
        #[track_caller]
        #[cfg(feature = "alloc")]
        pub fn rt<T, M>(value: T) -> T
        where
            T: $crate::en::Encode<M> + $crate::de::DecodeOwned<M, $crate::alloc::Global>,
            T: ::core::fmt::Debug + ::core::cmp::PartialEq,
            M: 'static,
        {
            const WHAT: &str = $what;

            let encoding = super::Encoding::new().with_mode::<M>();

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

            let cx = $crate::context::new().with_trace().with_type();

            let out = match encoding.to_vec_with(&cx, &value) {
                Ok(out) => out,
                Err(..) => {
                    let error = cx.report();
                    panic!("{WHAT}: {}: failed to encode:\n{error}", type_name::<T>())
                }
            };

            let decoded: T = match encoding.from_slice_with(&cx, out.as_slice()) {
                Ok(decoded) => decoded,
                Err(..) => {
                    let out = FormatBytes(&out);
                    let error = cx.report();
                    panic!("{WHAT}: {}: failed to decode:\nValue: {value:?}\nBytes: {out}\n{error}", type_name::<T>())
                }
            };

            assert_eq!(decoded, value, "{WHAT}: {}: roundtrip does not match\nValue: {value:?}", type_name::<T>());

            $crate::macros::test_include_if! {
                $($(#[$option])*)* =>
                let value_encoding = crate::value::Encoding::new().with_mode::<M>();

                let value_decode: $crate::value::Value<_> = match encoding.from_slice_with(&cx, out.as_slice()) {
                    Ok(decoded) => decoded,
                    Err(..) => {
                        let out = FormatBytes(&out);
                        let error = cx.report();
                        panic!("{WHAT}: {}: failed to decode to value type:\nValue: {value:?}\nBytes:{out}\n{error}", type_name::<T>())
                    }
                };

                let value_decoded: T = match value_encoding.decode_with(&cx, &value_decode) {
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
        }

        /// Encode and then decode the given value once.
        #[doc(hidden)]
        #[track_caller]
        #[cfg(feature = "alloc")]
        pub fn decode<'de, T, U, M>(value: T, out: &'de mut rust_alloc::vec::Vec<u8>, expected: &U) -> U
        where
            T: $crate::en::Encode<M>,
            T: ::core::fmt::Debug + ::core::cmp::PartialEq,
            U: $crate::de::Decode<'de, M, $crate::alloc::Global>,
            U: ::core::fmt::Debug + ::core::cmp::PartialEq,
            M: 'static,
        {
            const WHAT: &str = $what;

            let encoding = super::Encoding::new().with_mode::<M>();

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

            let cx = $crate::context::new().with_trace().with_type();

            out.clear();

            match encoding.to_writer_with(&cx, &mut *out, &value) {
                Ok(()) => (),
                Err(..) => {
                    let error = cx.report();
                    panic!("{WHAT}: {}: failed to encode:\n{error}", type_name::<T>())
                }
            };

            let actual = match encoding.from_slice_with(&cx, &*out) {
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
        }

        /// Encode a value to bytes.
        #[doc(hidden)]
        #[track_caller]
        #[cfg(feature = "alloc")]
        pub fn to_vec<T, M>(value: T) -> rust_alloc::vec::Vec<u8>
        where
            T: $crate::en::Encode<M>,
            M: 'static,
        {
            const WHAT: &str = $what;

            let encoding = super::Encoding::new().with_mode::<M>();

            use ::core::any::type_name;

            $crate::alloc::default(|alloc| {
                let cx = $crate::context::new_in(alloc).with_trace().with_type();

                match encoding.to_vec_with(&cx, &value) {
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

pub(crate) use test_fns;

/// Assert that expression `$expr` can be roundtrip encoded using the encodings
/// specified by `$support`.
///
/// This demands that encoding and subsequently decoding `$expr` through any
/// formats results in a value that can be compared using [`PartialEq`] to
/// `$expr`.
///
/// This can be used to test upgrade stable formats.
///
/// The `$support` parameter is one of:
/// * `full` - All formats are tested.
/// * `no_json` - All formats except JSON are tested.
/// * `descriptive` - All fully self-descriptive formats are tested.
/// * `json` - Only JSON is tested.
/// * `upgrade_stable` - Only upgrade-stable formats are tested.
///
/// Extra tests can be specified using the `$extra` parameter:
/// * `json = <expected>` - Assert that the JSON encoding of `$expr` matched
///   exactly `$expected`.
///
/// # Examples
///
/// ```
/// use musli::{Decode, Encode};
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Version1 {
///     name: String,
/// }
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Person {
///     name: String,
///     #[musli(default)]
///     age: Option<u32>,
/// }
///
/// musli::macros::assert_roundtrip_eq! {
///     full,
///     Person {
///         name: String::from("Aristotle"),
///         age: Some(61),
///     },
///     json = r#"{"name":"Aristotle","age":61}"#,
/// };
/// ```
#[cfg_attr(doc_cfg, doc(cfg(feature = "test")))]
#[macro_export]
macro_rules! assert_roundtrip_eq {
    ($support:ident, $expr:expr $(, $($extra:tt)*)?) => {{
        let expected = $expr;

        macro_rules! inner {
            ($name:ident, $mode:ident) => {{
                assert_eq!(
                    $crate::$name::test::rt::<_, $crate::mode::$mode>($expr),
                    expected,
                    "{}: roundtripped value does not match expected",
                    stringify!($name),
                );
            }}
        }

        $crate::macros::__test_matrix!($support, inner);
        $crate::macros::support::musli_value_rt($expr);
        $crate::macros::__test_extra!($expr $(, $($extra)*)*);
        expected
    }};
}

pub use assert_roundtrip_eq;

/// Assert that expression `$expr` can decode to expression `$expected` using
/// the encodings specified by `$support`.
///
/// This can be used to test upgrade stable formats.
///
/// The `$support` parameter is one of:
/// * `full` - All formats are tested.
/// * `no_json` - All formats except JSON are tested.
/// * `descriptive` - All fully self-descriptive formats are tested.
/// * `json` - Only JSON is tested.
/// * `upgrade_stable` - Only upgrade-stable formats are tested.
///
/// Extra tests can be specified using the `$extra` parameter:
/// * `json = <expected>` - Assert that the JSON encoding of `$expr` matched
///   exactly `$expected`.
///
/// # Examples
///
/// ```
/// use musli::{Decode, Encode};
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Version1 {
///     name: String,
/// }
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Version2 {
///     name: String,
///     #[musli(default)]
///     age: Option<u32>,
/// }
///
/// // Only upgrade stable formats can remove an existing field since they need
/// // to know how to skip it over.
/// musli::macros::assert_decode_eq! {
///     upgrade_stable,
///     Version2 {
///         name: String::from("Aristotle"),
///         age: Some(61),
///     },
///     Version1 {
///         name: String::from("Aristotle"),
///     },
///     json = r#"{"name":"Aristotle","age":61}"#,
/// };
///
/// // Every supported format can add a new field which has a default value.
/// musli::macros::assert_decode_eq! {
///     full,
///     Version1 {
///         name: String::from("Aristotle"),
///     },
///     Version2 {
///         name: String::from("Aristotle"),
///         age: None,
///     },
///     json = r#"{"name":"Aristotle"}"#,
/// };
/// ```
#[cfg_attr(doc_cfg, doc(cfg(feature = "test")))]
#[macro_export]
macro_rules! assert_decode_eq {
    ($support:ident, $expr:expr, $expected:expr $(, $($extra:tt)*)?) => {{
        let mut bytes = $crate::macros::support::Vec::<u8>::new();

        macro_rules! decode {
            ($name:ident, $mode:ident) => {{
                $crate::$name::test::decode::<_, _, $crate::mode::$mode>($expr, &mut bytes, &$expected);
            }}
        }

        $crate::macros::__test_matrix!($support, decode);
        $crate::macros::__test_extra!($expr $(, $($extra)*)*);
    }};
}

pub use assert_decode_eq;

#[doc(hidden)]
#[macro_export]
macro_rules! __test_extra {
    ($expr:expr $(,)?) => {};

    ($expr:expr, json = $json_expected:expr $(, $($extra:tt)*)?) => {{
        let json = $crate::json::test::to_vec::<_, $crate::mode::Text>($expr);
        let string = ::std::string::String::from_utf8(json).expect("Encoded JSON is not valid utf-8");

        assert_eq!(
            string, $json_expected,
            "json: encoded json does not match expected value"
        );

        $crate::macros::__test_extra!($expr $(, $($extra)*)*);
    }};

    ($expr:expr, json_binary = $json_expected:expr $(, $($extra:tt)*)?) => {{
        let json = $crate::json::test::to_vec::<_, $crate::mode::Binary>($expr);
        let string = ::std::string::String::from_utf8(json).expect("Encoded JSON is not valid utf-8");

        assert_eq!(
            string, $json_expected,
            "json: encoded json does not match expected value"
        );

        $crate::macros::__test_extra!($expr $(, $($extra)*)*);
    }};
}

pub use __test_extra;

#[doc(hidden)]
#[macro_export]
macro_rules! __test_matrix {
    (full, $call:path) => {
        $call!(storage, Binary);
        $call!(storage, Text);
        $call!(packed, Binary);
        $call!(wire, Binary);
        $call!(wire, Text);
        $call!(descriptive, Binary);
        $call!(descriptive, Text);
        $call!(json, Text);
    };

    (not_packed, $call:path) => {
        $call!(storage, Binary);
        $call!(wire, Binary);
        $call!(descriptive, Binary);
        $call!(json, Text);
    };

    (text_mode, $call:path) => {
        $call!(storage, Text);
        $call!(wire, Text);
        $call!(descriptive, Text);
        $call!(json, Text);
    };

    (binary_mode, $call:path) => {
        $call!(storage, Binary);
        $call!(packed, Binary);
        $call!(wire, Binary);
        $call!(descriptive, Binary);
        $call!(json, Binary);
    };

    (no_json, $call:path) => {
        $call!(storage, Binary);
        $call!(packed, Binary);
        $call!(wire, Binary);
        $call!(descriptive, Binary);
    };

    (descriptive, $call:path) => {
        $call!(descriptive, Binary);
        $call!(json, Text);
    };

    (json, $call:path) => {
        $call!(json, Text);
    };

    (packed, $call:path) => {
        $call!(packed, Binary);
    };

    (upgrade_stable, $call:path) => {
        $call!(wire, Binary);
        $call!(descriptive, Binary);
        $call!(json, Text);
    };
}

pub use __test_matrix;

#[doc(hidden)]
#[cfg(feature = "alloc")]
pub mod support {
    pub use rust_alloc::vec::Vec;

    use crate::alloc::Global;
    use crate::mode::Binary;
    use crate::value::{self, Value};
    use crate::{Decode, Encode};

    #[track_caller]
    pub fn musli_value_rt<T>(expected: T)
    where
        T: Encode<Binary> + for<'de> Decode<'de, Binary, Global>,
        T: PartialEq + core::fmt::Debug,
    {
        let value: Value<_> = value::encode(&expected).expect("value: Encoding should succeed");
        let actual: T = value::decode(&value).expect("value: Decoding should succeed");
        assert_eq!(
            actual, expected,
            "value: roundtripped value does not match expected"
        );
    }
}
