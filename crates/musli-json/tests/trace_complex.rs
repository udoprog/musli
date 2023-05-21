#![allow(unused)]

use std::collections::HashMap;

use musli::{Decode, Encode};
use musli_common::context::RichContext;

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
    let mut string = String::new();
    let mut cx = RichContext::new(&mut string);

    let mut field = HashMap::new();

    field.insert(
        "hello".to_string(),
        InnerFrom::Variant2 {
            vector: vec![42],
            ok: 10004000,
        },
    );

    let from = From { ok: 10, field };

    let encoding = musli_json::Encoding::new();

    let Ok(bytes) = encoding.to_vec_with(&mut cx, &from) else {
        for error in cx.iter() {
            panic!("{error}");
        }

        unreachable!()
    };

    let mut cx = RichContext::new(&mut string);

    let Ok(..) = encoding.from_slice_with::<_, To>(&mut cx, &bytes) else {
        if let Some(error) = cx.iter().next() {
            assert_eq!(error.to_string(), ".field[hello] = Variant2 { .vector[0] }: expected string, found <number> (at byte 36)");
            return;
        }

        unreachable!()
    };

    panic!("expected decoding to error");
}
