use std::collections::HashMap;

use musli::{Decode, Encode};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Complex {
    map: HashMap<String, String>,
}

#[derive(Encode, Decode)]
struct Struct {
    #[musli(with = musli_serde::<_>)]
    field: Complex,
}
