#![allow(unused)]

use musli::{Decode, Encode};
use musli_common::context::RichContext;

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
    let mut string = String::new();
    let mut cx = RichContext::new(&mut string);

    let from = From {
        ok: 10,
        field: InnerFrom::Variant2 {
            ok: 10,
            vector: vec![42],
        },
    };

    let encoding = musli_storage::Encoding::new();

    let Ok(bytes) = encoding.to_vec_with(&mut cx, &from) else {
        for error in cx.iter() {
            panic!("{error}");
        }

        unreachable!()
    };

    let Ok(..) = encoding.from_slice_with::<_, To>(&mut cx, &bytes) else {
        if let Some(error) = cx.iter().next() {
            assert_eq!(error.to_string(), ".field = Variant2 { .vector[0] }: buffer underflow (at byte 33)");
            return;
        }

        unreachable!()
    };

    panic!("expected decoding to error");
}
