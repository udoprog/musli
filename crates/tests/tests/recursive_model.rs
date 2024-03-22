//! When this file is built in release mode, it was discovered that it there is an overflow in requirements:
//!
//! ```sh
//! cargo build --release -p tests --test recursive_model --features test
//! ```
//!
//! ```text
//! error[E0275]: overflow evaluating the requirement `TE: Encoder<TC>`
//!   |
//!   = help: consider increasing the recursion limit by adding a `#![recursion_limit = "256"]` attribute to your crate (`big_model`)
//! ```

use musli::{Decode, Encode};

#[derive(Encode, Decode)]
pub(crate) enum Value {
    Array(Vec<Value>),
}

#[derive(Encode, Decode)]
pub(crate) struct Change {
    value: Value,
}

pub(crate) fn save_changes<T>(changes: &T) -> tests::wire::Result<Vec<u8>>
where
    T: Encode,
{
    let mut w = Vec::new();
    tests::wire::to_writer(&mut w, changes)?;
    Ok(w)
}

#[test]
fn big_model() {
    let change = Change {
        value: Value::Array(Vec::new()),
    };

    save_changes(&change).unwrap();
}
