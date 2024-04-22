use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Version1 {
    name: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Version2 {
    name: String,
    #[musli(default)]
    age: Option<u32>,
}

#[test]
fn version1_to_2() {
    let version2 = musli::storage::to_vec(&Version2 {
        name: String::from("Aristotle"),
        age: Some(62),
    })
    .unwrap();

    assert!(musli::storage::decode::<_, Version1>(version2.as_slice()).is_err());

    let version1 = musli::storage::to_vec(&Version1 {
        name: String::from("Aristotle"),
    })
    .unwrap();

    let version2: Version2 = musli::storage::decode(version1.as_slice()).unwrap();

    assert_eq!(version2.name, "Aristotle");
    assert_eq!(version2.age, None);
}
