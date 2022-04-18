use musli::{Encode, Decode};

/// Non-empty fallback variant.
#[derive(Encode, Decode)]
enum Enum1 {
    #[musli(default)]
    Variant {
        field: u32,
    }
}

/// Multiple fallback variants.
#[derive(Encode, Decode)]
enum Enum2 {
    #[musli(default)]
    Fallback1,
    #[musli(default)]
    Fallback2,
}

fn main() {
}
