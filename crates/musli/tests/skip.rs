#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Inner {
    a: u32,
    b: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct StructSkipped {
    #[musli(skip)]
    skip: u32,
    #[musli(skip, default = skip_default)]
    skip_default: u32,
    #[musli(skip, default = skip_complex_field)]
    complex_field: Option<Inner>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct TupleSkipped(
    #[musli(skip)] u32,
    #[musli(skip, default = skip_default)] u32,
    #[musli(skip, default = skip_complex_field)] Option<Inner>,
);

#[derive(Debug, PartialEq, Encode, Decode)]
enum EnumSkipped {
    Struct {
        #[musli(skip)]
        skip: u32,
        #[musli(skip, default = skip_default)]
        skip_default: u32,
        #[musli(skip, default = skip_complex_field)]
        complex_field: Option<Inner>,
    },
    Tuple(
        #[musli(skip)] u32,
        #[musli(skip, default = skip_default)] u32,
        #[musli(skip, default = skip_complex_field)] Option<Inner>,
    ),
}

fn skip_default() -> u32 {
    42
}

fn skip_complex_field() -> Option<Inner> {
    Some(Inner { a: 1, b: 2 })
}

#[test]
fn skip() {
    musli::macros::assert_roundtrip_eq!(
        full,
        StructSkipped {
            skip: 0,
            skip_default: 42,
            complex_field: Some(Inner { a: 1, b: 2 }),
        },
        json = r#"{}"#,
    );

    musli::macros::assert_decode_eq!(
        full,
        StructSkipped {
            skip: 10,
            skip_default: 52,
            complex_field: Some(Inner { a: 3, b: 4 }),
        },
        StructSkipped {
            skip: 0,
            skip_default: 42,
            complex_field: Some(Inner { a: 1, b: 2 }),
        },
        json = r#"{}"#,
    );

    musli::macros::assert_decode_eq!(
        full,
        EnumSkipped::Struct {
            skip: 10,
            skip_default: 52,
            complex_field: Some(Inner { a: 3, b: 4 }),
        },
        EnumSkipped::Struct {
            skip: 0,
            skip_default: 42,
            complex_field: Some(Inner { a: 1, b: 2 }),
        },
        json = r#"{"Struct":{}}"#,
    );

    musli::macros::assert_roundtrip_eq!(
        full,
        TupleSkipped(0, 42, Some(Inner { a: 1, b: 2 }),),
        json = r#"{}"#,
    );

    musli::macros::assert_decode_eq!(
        full,
        TupleSkipped(10, 52, Some(Inner { a: 3, b: 4 }),),
        TupleSkipped(0, 42, Some(Inner { a: 1, b: 2 }),),
        json = r#"{}"#,
    );

    musli::macros::assert_decode_eq!(
        full,
        EnumSkipped::Tuple(10, 52, Some(Inner { a: 3, b: 4 }),),
        EnumSkipped::Tuple(0, 42, Some(Inner { a: 1, b: 2 }),),
        json = r#"{"Tuple":{}}"#,
    );
}
