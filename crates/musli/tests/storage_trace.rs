#![allow(unused)]

use musli::alloc::System;
use musli::context;
use musli::{Decode, Encode};

#[derive(Encode)]
enum InnerFrom {
    Variant1,
    Variant2 { ok: u32, vector: Vec<u32> },
}

#[derive(Encode)]
struct From {
    ok: u32,
    field: InnerFrom,
}

#[derive(Decode)]
enum InnerTo {
    Variant1,
    Variant2 { ok: u32, vector: Vec<String> },
}

#[derive(Decode)]
struct To {
    ok: u32,
    field: InnerTo,
}

#[test]
fn storage_trace() {
    musli::alloc::default(|alloc| {
        let cx = context::with_alloc(alloc);

        let from = From {
            ok: 10,
            field: InnerFrom::Variant2 {
                ok: 10,
                vector: vec![42],
            },
        };

        let encoding = musli::storage::Encoding::new();

        let Ok(bytes) = encoding.to_vec_with(&cx, &from) else {
            if let Some(error) = cx.errors().next() {
                panic!("{error}");
            }

            unreachable!()
        };

        let Ok(..) = encoding.from_slice_with::<_, To>(&cx, &bytes) else {
            if let Some(error) = cx.errors().next() {
                assert_eq!(error.to_string(), ".field = Variant2 { .vector[0] }: Tried to read 42 bytes from slice, with 0 byte remaining (at byte 11)");
                return;
            }

            unreachable!()
        };

        panic!("Expected decoding to error");
    })
}
