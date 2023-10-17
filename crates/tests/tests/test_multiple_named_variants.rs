use musli::{Decode, Encode};

#[derive(Encode, Decode)]
#[musli(default_variant_name = "name")]
enum MultipleNamedVariants {
    Variant1 { field1: u32 },
    Variant2 { field1: u32 },
}
