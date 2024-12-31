#![cfg(feature = "test")]

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name(type = usize))]
pub struct Struct {
    field1: u32,
    field2: u32,
    field3: u32,
    field4: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name(type = usize))]
pub struct StructPattern {
    field1: u32,
    #[musli(pattern = 1..=2)]
    field2: u32,
    #[musli(name = 3)]
    field4: u32,
}

#[test]
fn struct_pattern() {
    musli::macros::assert_decode_eq!(
        upgrade_stable,
        Struct {
            field1: 0,
            field2: 1,
            field3: 2,
            field4: 3
        },
        StructPattern {
            field1: 0,
            field2: 2,
            field4: 3
        }
    );
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name(type = usize))]
pub struct StructRename {
    #[musli(name = 3)]
    field4: u32,
}

#[test]
fn struct_rename() {
    musli::macros::assert_decode_eq!(
        upgrade_stable,
        Struct {
            field1: 11,
            field2: 13,
            field3: 15,
            field4: 17,
        },
        StructRename { field4: 17 },
        json = r#"{"0":11,"1":13,"2":15,"3":17}"#,
    );
}
