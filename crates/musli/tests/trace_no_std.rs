#![allow(unused)]

use std::collections::HashMap;

use musli::allocator::{Stack, StackBuffer};
use musli::context::StackContext;
use musli::{Decode, Encode};

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
fn trace_no_std() {
    let mut buf = StackBuffer::<1024>::new();
    let alloc = Stack::new(&mut buf);
    let cx = StackContext::new(&alloc);

    let mut values = HashMap::new();

    values.insert("Hello".to_string(), "World".to_string());

    let from = From { values };

    let encoding = musli::json::Encoding::new();

    let Ok(bytes) = encoding.to_vec_with(&cx, &from) else {
        if let Some(error) = cx.errors().next() {
            panic!("{error}");
        }

        unreachable!()
    };

    let cx = StackContext::new(&alloc);

    let Ok(..) = encoding.from_slice_with::<_, Collection>(&cx, &bytes) else {
        if let Some(error) = cx.errors().next() {
            assert_eq!(
                error.to_string(),
                ".values[Hello]: Invalid numeric (at bytes 19-20)"
            );
            return;
        }

        unreachable!()
    };

    panic!("Expected decoding to error");
}