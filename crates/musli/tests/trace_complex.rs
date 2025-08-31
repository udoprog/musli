#![allow(unused)]

use std::collections::HashMap;

use musli::alloc::Global;
use musli::context;
use musli::{Decode, Encode};

#[derive(Encode)]
enum InnerFrom {
    Variant1,
    Variant2 { vector: Vec<u32>, ok: u32 },
}

#[derive(Encode)]
struct From {
    ok: u32,
    field: HashMap<String, InnerFrom>,
}

#[derive(Decode)]
enum InnerTo {
    Variant1,
    Variant2 { vector: Vec<String>, ok: u32 },
}

#[derive(Decode)]
struct To {
    ok: u32,
    #[musli(trace)]
    field: HashMap<String, InnerTo>,
}

#[test]
fn trace_complex() {
    musli::alloc::default(|alloc| {
        let cx = context::new_in(alloc).with_trace();

        let mut field = HashMap::new();

        field.insert(
            "hello".to_string(),
            InnerFrom::Variant2 {
                vector: vec![42],
                ok: 10004000,
            },
        );

        let from = From { ok: 10, field };

        let encoding = musli::json::Encoding::new();

        let Ok(bytes) = encoding.to_vec_with(&cx, &from) else {
            if let Some(error) = cx.errors().next() {
                panic!("{error}");
            }

            unreachable!()
        };

        let cx = context::new_in(alloc).with_trace();

        let Ok(..) = encoding.from_slice_with::<_, To>(&cx, &bytes) else {
            if let Some(error) = cx.errors().next() {
                assert_eq!(
                    error.to_string(),
                    ".field[hello] = Variant2 { .vector[0] }: Expected string, found <number> (at byte 49)"
                );
                return;
            }

            unreachable!()
        };

        panic!("Expected decoding to error");
    })
}
