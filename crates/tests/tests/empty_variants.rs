use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(default_variant = "name")]
enum NamedVariant {
    Tuple(),
    Struct {},
}

#[test]
fn enum_with_empty_variant() {
    tests::rt!(NamedVariant::Tuple());
    tests::rt!(NamedVariant::Struct {});
}
