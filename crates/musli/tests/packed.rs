use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Decode, Encode)]
struct PackedFields {
    #[musli(packed)]
    tuple: (u16, u32, u64, u128),
    #[musli(packed)]
    array: [u32; 4],
}

#[test]
fn packed_fields() {
    musli::rt!(
        full,
        PackedFields {
            tuple: (11, 13, 15, 17),
            array: [11, 13, 15, 17],
        },
        json = r#"{"tuple":[11,13,15,17],"array":[11,13,15,17]}"#
    );
}
