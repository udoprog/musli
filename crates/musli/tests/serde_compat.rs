//! Tests that ensure that serde compatibility encodes and decodes correctly.

/// Default random seed to use.
pub const RNG_SEED: u64 = 2818281828459045235;

use std::collections::HashMap;
use std::fmt;

use bstr::BStr;
use musli::de::DecodeOwned;
use musli::mode::{Binary, Text};
use musli::{Decode, Encode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tests::{Generate, Rng};

#[derive(Encode)]
#[musli(transparent)]
struct EncodeSerde<'a, T>(#[musli(with = musli::serde)] &'a T)
where
    T: Serialize;

#[derive(Decode)]
#[musli(transparent)]
struct DecodeSerde<T>(#[musli(with = musli::serde)] T)
where
    T: DeserializeOwned;

mod value {
    use musli::alloc::Global;

    use super::*;

    #[track_caller]
    pub(super) fn random<T>(module: &str)
    where
        T: Eq
            + fmt::Debug
            + Generate
            + Encode<Binary>
            + DecodeOwned<Binary, Global>
            + Serialize
            + DeserializeOwned,
    {
        guided(module, <T as Generate>::generate);
    }

    #[track_caller]
    pub(super) fn guided<T>(module: &str, value: fn(&mut Rng) -> T)
    where
        T: Eq
            + fmt::Debug
            + Encode<Binary>
            + DecodeOwned<Binary, Global>
            + Serialize
            + DeserializeOwned,
    {
        macro_rules! do_try {
            ($expr:expr, $msg:expr) => {
                match $expr {
                    Ok(value) => value,
                    Err(err) => panic! {
                        "{module}<{}>: {}:\n{}",
                        ::std::any::type_name::<T>(),
                        $msg,
                        err
                    },
                }
            };

            ($expr:expr, $msg:expr, $encoded:expr) => {
                match $expr {
                    Ok(value) => value,
                    Err(err) => panic! {
                        "{module}<{}>: {}:\n{}\n{:?}",
                        ::std::any::type_name::<T>(),
                        $msg,
                        err,
                        $encoded
                    },
                }
            };
        }

        let mut rng = tests::rng_with_seed(RNG_SEED);
        let value1 = value(&mut rng);

        let encoded1 = do_try!(::musli::value::encode(&value1), "Encode musli");

        let value2: T = do_try!(::musli::value::decode(&encoded1), "Decode musli");
        assert_eq!(value1, value2, "Musli decoding is incorrect");

        let encoded2 = do_try!(::musli::value::encode(EncodeSerde(&value1)), "Encode serde");

        let DecodeSerde(value3) =
            do_try!(::musli::value::decode(&encoded2), "Decode serde", encoded2);

        assert_eq! {
            value1,
            value3,
            "Serde decoding is incorrect\nBytes: {encoded2:?}",
        };
    }
}

macro_rules! tester {
    ($module:ident, $mode:ty $(,)?) => {
        mod $module {
            use musli::alloc::Global;

            use super::*;

            #[track_caller]
            pub(super) fn random<T>(module: &str)
            where
                T: Encode<$mode> + DecodeOwned<$mode, Global>,
                T: Eq + fmt::Debug + Generate + Serialize + DeserializeOwned,
            {
                guided(module, <T as Generate>::generate);
            }

            #[track_caller]
            pub(super) fn guided<T>(module: &str, value: fn(&mut Rng) -> T)
            where
                T: Encode<$mode> + DecodeOwned<$mode, Global>,
                T: Eq + fmt::Debug + Serialize + DeserializeOwned,
            {
                macro_rules! do_try {
                    ($expr:expr, $msg:expr) => {
                        match $expr {
                            Ok(value) => value,
                            Err(err) => panic! {
                                "{module}<{}>: {}:\n{}",
                                ::std::any::type_name::<T>(),
                                $msg,
                                err
                            },
                        }
                    };

                    ($expr:expr, $msg:expr, $bytes:expr) => {
                        match $expr {
                            Ok(value) => value,
                            Err(err) => panic! {
                                "{module}<{}>: {}:\n{}\n{:?}",
                                ::std::any::type_name::<T>(),
                                $msg,
                                err,
                                BStr::new(&$bytes)
                            },
                        }
                    };
                }

                let mut rng = tests::rng_with_seed(RNG_SEED);
                let value1 = value(&mut rng);

                let bytes1 = do_try!(::musli::$module::to_vec(&value1), "Encode musli");

                let value2: T = do_try!(::musli::$module::from_slice(&bytes1), "Decode musli");
                assert_eq!(value1, value2, "Musli decoding is incorrect");

                let bytes2 = do_try!(
                    ::musli::$module::to_vec(&EncodeSerde(&value1)),
                    "Encode serde"
                );

                // TODO: Do we want serialization to be compatible?
                // assert! {
                //     &bytes1 == &bytes2,
                //     "Serde encoding is incorrect\nExpected: {:?}\nActual: {:?}",
                //     BStr::new(&bytes1),
                //     BStr::new(&bytes2),
                // };

                let DecodeSerde(value3) = do_try!(
                    ::musli::$module::from_slice(&bytes2),
                    "Decode serde",
                    bytes2
                );

                assert_eq! {
                    value1,
                    value3,
                    "Serde decoding is incorrect\nBytes: {:?}",
                    BStr::new(&bytes2),
                };

                // TODO: Do we want serialization to be compatible?
                // let value4: T =
                //     ::musli::$module::from_slice(&bytes2).expect("Decode musli from serde-encoded bytes");
                // assert_eq!(&value1, &value4, "Serde to musli roundtrip is incorrect");
            }
        }
    };
}

tester!(storage, Binary);
tester!(wire, Binary);
tester!(descriptive, Binary);
tester!(json, Text);

#[derive(Debug, PartialEq, Eq, Generate, Encode, Decode, Serialize, Deserialize)]
enum Enum {
    Empty,
    Tuple(u32, u32),
    Struct { a: u32, b: u32 },
}

#[derive(Debug, PartialEq, Eq, Generate, Encode, Decode, Serialize, Deserialize)]
struct Struct {
    a: u32,
    b: u64,
    inner_enum: Enum,
}

#[derive(Debug, PartialEq, Eq, Generate, Encode, Decode, Serialize, Deserialize)]
#[musli(Binary, bound = {T: Encode<Binary>}, decode_bound<'de, A> = {T: Decode<'de, Binary, A>})]
#[musli(Text, bound = {T: Encode<Text>}, decode_bound<'de, A> = {T: Decode<'de, Text, A>})]
struct StructWith<T> {
    a: u32,
    b: T,
}

macro_rules! build_test {
    ($module:ident) => {{
        $module::random::<Vec<u8>>(stringify!($module));
        $module::random::<String>(stringify!($module));
        $module::random::<StructWith<String>>(stringify!($module));
        $module::random::<StructWith<Vec<u8>>>(stringify!($module));
        $module::random::<StructWith<()>>(stringify!($module));

        $module::random::<u8>(stringify!($module));
        $module::random::<u16>(stringify!($module));
        $module::random::<u32>(stringify!($module));
        $module::random::<u64>(stringify!($module));
        $module::random::<u128>(stringify!($module));

        $module::random::<i8>(stringify!($module));
        $module::random::<i16>(stringify!($module));
        $module::random::<i32>(stringify!($module));
        $module::random::<i64>(stringify!($module));
        $module::random::<i128>(stringify!($module));
        $module::random::<()>(stringify!($module));

        $module::random::<HashMap<String, u32>>(stringify!($module));
        $module::random::<HashMap<u32, String>>(stringify!($module));
        $module::guided::<Enum>(stringify!($module), |_| Enum::Empty);

        $module::guided::<Struct>(stringify!($module), |r| Struct {
            a: r.next(),
            b: r.next(),
            inner_enum: Enum::Empty,
        });

        $module::guided::<Enum>(stringify!($module), |r| Enum::Tuple(r.next(), r.next()));

        $module::guided::<Struct>(stringify!($module), |r| Struct {
            a: r.next(),
            b: r.next(),
            inner_enum: Enum::Tuple(r.next(), r.next()),
        });

        $module::guided::<Enum>(stringify!($module), |r| Enum::Struct {
            a: r.next(),
            b: r.next(),
        });

        $module::guided::<Struct>(stringify!($module), |r| Struct {
            a: r.next(),
            b: r.next(),
            inner_enum: Enum::Struct {
                a: r.next(),
                b: r.next(),
            },
        });
    }};
}

#[test]
fn musli_storage() {
    build_test!(storage);
}

#[test]
fn musli_wire() {
    build_test!(wire);
}

#[test]
fn musli_descriptive() {
    build_test!(descriptive);
}

#[test]
fn musli_json() {
    build_test!(json);
}

#[test]
fn musli_value() {
    build_test!(value);
}
