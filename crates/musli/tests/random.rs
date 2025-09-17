use anyhow::Result;
use musli::storage;
use musli::{Decode, Encode};

use tests::Generate;

#[derive(Generate, Debug, Clone, PartialEq, Encode, Decode)]
#[musli(Text, name_all = "snake_case", tag = "type")]
pub enum SimpleEnum2 {
    Variant { inner: String },
}

#[derive(Generate, Debug, Clone, PartialEq, Encode, Decode)]
#[musli(Text, name_all = "snake_case", tag = "type")]
pub enum SimpleEnum {
    #[musli(Text, name_all = "snake_case")]
    Variant {
        #[musli(default)]
        inner: Option<SimpleEnum2>,
    },
}

/// The serializable state of the agent.
#[derive(Generate, Debug, Clone, PartialEq, Encode, Decode)]
#[generate(ignore_bound = A)]
pub struct SimpleData {
    inner: SimpleEnum,
}

#[test]
fn nested_enum_option() -> Result<()> {
    let actual = SimpleData::random();
    let encoded = storage::to_vec(&actual)?;
    let decoded: SimpleData = storage::from_slice(&encoded)?;
    assert_eq!(actual, decoded);
    Ok(())
}
