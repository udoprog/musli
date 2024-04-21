use std::collections::HashMap;

use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Serde {
    map: HashMap<String, String>,
}

#[derive(Encode, Decode)]
struct Struct {
    #[musli(with = musli::serde)]
    field: Serde,
}
