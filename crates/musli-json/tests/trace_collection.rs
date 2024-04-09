#![allow(unused)]

use std::collections::HashMap;

use musli::{Decode, Encode};
use musli_utils::allocator::{System, SystemBuffer};
use musli_utils::context::SystemContext;

#[derive(Encode)]
struct From {
    values: HashMap<String, String>,
}

#[derive(Decode)]
struct Collection {
    #[musli(trace)]
    values: HashMap<String, u32>,
}

#[test]
fn trace_collection() {
    let mut buf = SystemBuffer::new();
    let alloc = System::new(&mut buf);
    let cx = SystemContext::new(&alloc);

    let mut values = HashMap::new();

    values.insert("Hello".to_string(), "World".to_string());

    let from = From { values };

    let encoding = musli_json::Encoding::new();

    let Ok(bytes) = encoding.to_vec_with(&cx, &from) else {
        if let Some(error) = cx.errors().next() {
            panic!("{error}");
        }

        unreachable!()
    };

    let cx = SystemContext::new(&alloc);

    let Ok(..) = encoding.from_slice_with::<_, Collection>(&cx, &bytes) else {
        if let Some(error) = cx.errors().next() {
            assert_eq!(
                error.to_string(),
                ".values[Hello]: Invalid numeric (at bytes 15-16)"
            );
            return;
        }

        unreachable!()
    };

    panic!("Expected decoding to error");
}
