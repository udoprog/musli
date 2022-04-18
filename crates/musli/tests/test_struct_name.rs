use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(field = "name")]
pub struct Named {
    string: String,
    number: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(packed)]
pub struct NamedUnpacked {
    field_count: u8,
    field1_name: String,
    field1_value: String,
    field2_name: String,
    field2_value: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(field = "index")]
pub struct Indexed {
    string: String,
    number: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(packed)]
pub struct IndexedUnpacked {
    field_count: u8,
    field1_index: u8,
    field1_value: String,
    field2_index: u8,
    field2_value: u32,
}

#[test]
fn struct_named_fields() -> Result<(), Box<dyn std::error::Error>> {
    musli_wire::test::rt(Named {
        string: String::from("foo"),
        number: 42,
    })?;

    let out = musli_wire::to_vec(&Named {
        string: String::from("foo"),
        number: 42,
    })?;

    /*
    let unpacked: NamedUnpacked = musli_wire::decode(&out[..])?;

    assert_eq!(
        unpacked,
        NamedUnpacked {
            field_count: 2,
            field1_name: String::from("string"),
            field1_value: String::from("foo"),
            field2_name: String::from("number"),
            field2_value: 42,
        }
    );

    musli_wire::test::rt(unpacked)?;
    */
    Ok(())
}

#[test]
fn struct_indexed_fields() -> Result<(), Box<dyn std::error::Error>> {
    musli_wire::test::rt(Indexed {
        string: String::from("foo"),
        number: 42,
    })?;

    let out = musli_wire::to_vec(&Indexed {
        string: String::from("foo"),
        number: 42,
    })?;

    /*
    let unpacked: IndexedUnpacked = musli_wire::decode(&out[..])?;

    assert_eq!(
        unpacked,
        IndexedUnpacked {
            field_count: 2,
            field1_index: 0,
            field1_value: String::from("foo"),
            field2_index: 1,
            field2_value: 42,
        }
    );

    musli_wire::test::rt(unpacked)?;
    */
    Ok(())
}
