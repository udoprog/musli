//! Tests that ensure that serde compatibility encodes and decodes correctly.

/// Default random seed to use.
pub const RNG_SEED: u64 = 2818281828459045235;

use std::collections::HashMap;
use std::fmt;

use bstr::BStr;
use musli::de::DecodeOwned;
use musli::{Decode, Encode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tests::generate::{Generate, Rng};

#[derive(Encode)]
#[musli(transparent)]
struct EncodeSerde<'a, T>(#[musli(with = musli_serde)] &'a T)
where
    T: Serialize;

#[derive(Decode)]
#[musli(transparent)]
struct DecodeSerde<T>(#[musli(with = musli_serde)] T)
where
    T: DeserializeOwned;

macro_rules! tester {
    ($module:ident $(,)?) => {
        mod $module {
            use super::*;

            #[track_caller]
            pub(super) fn random<T>()
            where
                T: Eq + fmt::Debug + Generate + Encode + DecodeOwned + Serialize + DeserializeOwned,
            {
                guided(<T as Generate>::generate);
            }

            #[track_caller]
            pub(super) fn guided<T>(value: fn(&mut Rng) -> T)
            where
                T: Eq + fmt::Debug + Encode + DecodeOwned + Serialize + DeserializeOwned,
            {
                let mut rng = tests::rng_with_seed(RNG_SEED);
                let value1 = value(&mut rng);

                let bytes1 = ::$module::to_vec(&value1).expect("Encode musli");

                let value2: T = ::$module::from_slice(&bytes1).expect("Decode musli");
                assert_eq!(value1, value2, "Musli decoding is incorrect");

                let bytes2 = ::$module::to_vec(&EncodeSerde(&value1)).expect("Encode serde");

                // TODO: Do we want serialization to be compatible?
                // assert! {
                //     &bytes1 == &bytes2,
                //     "Serde encoding is incorrect\nExpected: {:?}\nActual: {:?}",
                //     BStr::new(&bytes1),
                //     BStr::new(&bytes2),
                // };

                let DecodeSerde(value3) = ::$module::from_slice(&bytes2).expect("Decode serde");

                assert_eq! {
                    value1,
                    value3,
                    "Serde decoding is incorrect\nBytes: {:?}",
                    BStr::new(&bytes2),
                };

                // TODO: Do we want serialization to be compatible?
                // let value4: T =
                //     ::$module::from_slice(&bytes2).expect("Decode musli from serde-encoded bytes");
                // assert_eq!(&value1, &value4, "Serde to musli roundtrip is incorrect");
            }
        }
    };
}

tester!(musli_storage);
tester!(musli_wire);
tester!(musli_descriptive);

#[derive(Debug, PartialEq, Eq, Generate, Encode, Decode, Serialize, Deserialize)]
#[generate(crate)]
enum Enum {
    Empty,
    Tuple(u32, u32),
    Struct { a: u32, b: u32 },
}

#[derive(Debug, PartialEq, Eq, Generate, Encode, Decode, Serialize, Deserialize)]
#[generate(crate)]
struct Struct {
    a: u32,
    b: u64,
    enum_: Enum,
}

#[derive(Debug, PartialEq, Eq, Generate, Encode, Decode, Serialize, Deserialize)]
#[generate(crate)]
struct StructWithString {
    a: u32,
    b: String,
}

#[test]
fn serde_compat() {
    macro_rules! test {
        ($ty:ty) => {
            musli_storage::random::<$ty>();
            musli_wire::random::<$ty>();
            // musli_descriptive::random::<$ty>();
        };

        ($ty:ty, $factory:expr) => {
            musli_storage::guided::<$ty>($factory);
            musli_wire::guided::<$ty>($factory);
            // musli_descriptive::guided::<$ty>(|$rng| $factory);
        };
    }

    test!(String);
    test!(StructWithString);

    test!(u8);
    test!(u16);
    test!(u32);
    test!(u64);
    test!(u128);

    test!(i8);
    test!(i16);
    test!(i32);
    test!(i64);
    test!(i128);

    test!(HashMap<String, u32>);
    test!(Enum, |_| Enum::Empty);

    test!(Struct, |r| Struct {
        a: r.next(),
        b: r.next(),
        enum_: Enum::Empty,
    });

    test!(Enum, |r| Enum::Tuple(r.next(), r.next()));

    test!(Struct, |r| Struct {
        a: r.next(),
        b: r.next(),
        enum_: Enum::Tuple(r.next(), r.next()),
    });

    test!(Enum, |r| Enum::Struct {
        a: r.next(),
        b: r.next(),
    });

    test!(Struct, |r| Struct {
        a: r.next(),
        b: r.next(),
        enum_: Enum::Struct {
            a: r.next(),
            b: r.next(),
        },
    });
}

#[test]
fn musli_wire() {
    musli_wire::random::<u32>();
    musli_wire::random::<HashMap<String, u32>>();
    musli_wire::random::<Enum>();
    musli_wire::random::<Struct>();
}

#[test]
fn musli_descriptive() {
    musli_descriptive::random::<u32>();
    musli_descriptive::random::<HashMap<String, u32>>();
    musli_descriptive::random::<Enum>();
    musli_descriptive::random::<Struct>();
}
