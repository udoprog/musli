use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(default_variant = "name")]
enum MultipleNamedVariants {
    Variant1 { field: u32 },
    Variant2 { field: u32 },
}

#[test]
fn multiple_named_variants() {
    tests::rt!(MultipleNamedVariants::Variant1 { field: 1 });
    tests::rt!(MultipleNamedVariants::Variant2 { field: 2 });
}
