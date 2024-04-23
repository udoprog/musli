#![cfg(feature = "test")]

use std::collections::{HashMap, HashSet};

use musli::mode::{Binary, Text};
use musli::{Decode, Encode};
use tests::Generate;

const OTHER: OtherStruct = OtherStruct {
    field1: 10,
    field2: 20,
};
const ENUM1: OtherEnum = OtherEnum::Variant1;
const ENUM2: OtherEnum = OtherEnum::Variant2 { field: 10 };
const ENUM3: OtherEnum = OtherEnum::Variant3(10);

#[derive(Debug, PartialEq, Encode, Decode, Generate)]
pub struct OtherStruct {
    field1: u32,
    field2: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Encode, Decode)]
pub enum OtherEnum {
    Variant1,
    Variant2 { field: u32 },
    Variant3(u32),
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(mode = Binary, bound = {M: Encode<Binary>}, decode_bound = {M: Decode<'de, Binary>})]
#[musli(mode = Text, bound = {M: Encode<Text>}, decode_bound = {M: Decode<'de, Text>})]
pub struct SimpleStructFrom<M>
where
    M: Generate,
{
    pub field: String,
    pub interior: u32,
    pub middle: M,
    pub option: Option<u32>,
    pub other: OtherStruct,
    #[musli(mode = Binary, name = 5)]
    #[musli(mode = Text, name = "field4")]
    pub other_enum: OtherEnum,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct SimpleStructTo {
    pub field: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct SimpleStructEnum {
    #[musli(mode = Binary, name = 5)]
    pub field4: OtherEnum,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Empty;

macro_rules! types {
    ($call:path) => {
        $call!(u32);
        $call!(Vec<String>);
        $call!(HashMap<u32, u32>);
        $call!(HashMap<u32, OtherStruct>);
        $call!(HashSet<u32>);
    }
}

#[test]
fn skip_to_empty() {
    let mut rnd = tests::rng();

    macro_rules! test_case {
        ($ty:ty) => {
            musli::assert_decode_eq! {
                upgrade_stable,
                SimpleStructFrom {
                    field: String::from("Aristotle"),
                    interior: 42,
                    middle: <$ty>::generate(&mut rnd),
                    option: Some(108),
                    other: OTHER,
                    other_enum: ENUM1,
                },
                Empty
            }
        };
    }

    types!(test_case);
}

#[test]
fn skip_to_single() {
    let mut rnd = tests::rng();

    macro_rules! test_case {
        ($ty:ty) => {
            musli::assert_decode_eq! {
                upgrade_stable,
                SimpleStructFrom {
                    field: String::from("Aristotle"),
                    interior: 42,
                    middle: <$ty>::generate(&mut rnd),
                    option: Some(108),
                    other: OTHER,
                    other_enum: ENUM1,
                },
                SimpleStructTo {
                    field: String::from("Aristotle"),
                },
            };
        };
    }

    types!(test_case);
}

#[test]
fn skip_to_enum() {
    let mut rnd = tests::rng();

    macro_rules! test_case {
        ($ty:ty) => {
            for expected in [ENUM1, ENUM2, ENUM3] {
                musli::assert_decode_eq! {
                    upgrade_stable,
                    SimpleStructFrom {
                        field: String::from("Aristotle"),
                        interior: 42,
                        middle: <$ty>::generate(&mut rnd),
                        option: Some(108),
                        other: OTHER,
                        other_enum: expected,
                    },
                    SimpleStructEnum {
                        field4: expected,
                    }
                };
            }
        };
    }

    types!(test_case);
}
