#![allow(unused)]

use std::collections::HashMap;

use musli::{Decode, Encode};
use musli_common::allocator::Alloc;
use musli_common::context::AllocContext;

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
    let mut alloc = Alloc::default();
    let mut cx = AllocContext::new(&alloc);

    let mut values = HashMap::new();

    values.insert("Hello".to_string(), "World".to_string());

    let from = From { values };

    let encoding = musli_json::Encoding::new();

    let Ok(bytes) = encoding.to_vec_with(&mut cx, &from) else {
        for error in cx.iter() {
            panic!("{error}");
        }

        unreachable!()
    };

    let mut cx = AllocContext::new(&alloc);

    let Ok(..) = encoding.from_slice_with::<_, Collection>(&mut cx, &bytes) else {
        if let Some(error) = cx.iter().next() {
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
