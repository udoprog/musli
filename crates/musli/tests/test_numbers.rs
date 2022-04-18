use musli::{Decode, Encode};

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
struct StructWithNumbers {
    a: i16,
    b: i32,
    c: i64,
    d: i128,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[musli(packed)]
struct StructUnpacked {
    struct_type: u8,
    count: u8,
    a_tag_type: u8,
    a_tag: u8,
    a_type: u8,
    a: u8,
    b_tag_type: u8,
    b_tag: u8,
    b_type: u8,
    b: u8,
    c_tag_type: u8,
    c_tag: u8,
    c_type: u8,
    c: u8,
    d_tag_type: u8,
    d_tag: u8,
    d_type: u8,
    d: u8,
}

#[test]
fn test_signed_unpacked() -> Result<(), Box<dyn std::error::Error>> {
    let out = musli_wire::test::rt(StructWithNumbers {
        a: -1,
        b: 1,
        c: -3,
        d: 3,
    })?;

    let out = musli_wire::to_vec(&out)?;

    let unpacked: StructUnpacked = musli_storage::decode(&out[..])?;

    assert_eq! {
        unpacked,
        StructUnpacked {
            struct_type: musli_wire::types::PAIR_SEQUENCE,
            count: 4,
            a_tag_type: musli_wire::types::CONTINUATION,
            a_tag: 0,
            a_type: musli_wire::types::CONTINUATION,
            a: 1,
            b_tag_type: musli_wire::types::CONTINUATION,
            b_tag: 1,
            b_type: musli_wire::types::CONTINUATION,
            b: 2,
            c_tag_type: musli_wire::types::CONTINUATION,
            c_tag: 2,
            c_type: musli_wire::types::CONTINUATION,
            c: 5,
            d_tag_type: musli_wire::types::CONTINUATION,
            d_tag: 3,
            d_type: musli_wire::types::CONTINUATION,
            d: 6,
        }
    };

    Ok(())
}
