use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(transparent)]
struct TransparentStruct {
    string: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(transparent)]
struct TransparentTuple(String);

#[derive(Debug, PartialEq, Encode, Decode)]
enum TransparentEnum {
    NotTransparent {
        a: u32,
        b: u32,
    },
    #[musli(transparent)]
    Transparent(u32),
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(packed)]
struct TransparentEnumUnpacked {
    type_tag: u8,
    variant_tag_type: u8,
    variant_tag: u8,
    value_type: u8,
    value: u32,
}

#[test]
fn test_transparent_struct() {
    musli_wire::test::rt(TransparentStruct {
        string: String::from("Hello"),
    });
    let string = musli_wire::test::transcode::<_, String>(TransparentStruct {
        string: String::from("Hello"),
    });
    assert_eq!(string, "Hello");

    musli_wire::test::rt(TransparentTuple(String::from("Hello")));
    let string = musli_wire::test::transcode::<_, String>(TransparentTuple(String::from("Hello")));
    assert_eq!(string, "Hello");
}

#[test]
fn test_transparent_enum() {
    musli_wire::test::rt(TransparentEnum::Transparent(42));

    /*
    let unpacked = musli_wire::test::transcode::<_, TransparentEnumUnpacked>(
        TransparentEnum::Transparent(42),
    )?;

    assert_eq!(
        unpacked,
        TransparentEnumUnpacked {
            type_tag: musli_wire::tag::VARIANT,
            variant_tag_type: musli_wire::tag::Continuation,
            variant_tag: 1,
            value_type: musli_wire::tag::Continuation,
            value: 42,
        }
    );
    */
}
