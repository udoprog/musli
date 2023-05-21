use musli::{Decode, Encode};
use musli_common::context::AllocContext;

#[derive(Encode)]
enum InnerFrom {
    Variant1,
    Variant2 { field: u32 },
}

#[derive(Encode)]
struct From {
    field: InnerFrom,
}

#[derive(Decode)]
enum InnerTo {
    Variant1,
    Variant2 {
        #[musli(rename = 2)]
        field: u32,
    },
}

#[derive(Decode)]
struct To {
    field: InnerTo,
}

#[test]
fn test_trace() {
    let mut string = String::new();
    let mut cx = AllocContext::new(&mut string);

    let from = From {
        field: InnerFrom::Variant2 { field: 42 },
    };

    let encoding = musli_storage::Encoding::new();

    let Ok(bytes) = encoding.to_vec_with(&mut cx, &from) else {
        panic!("{cx}");
    };

    let Ok(to) = encoding.from_slice_with::<_, To>(&mut cx, &bytes) else {
        panic!("{cx}");
    };
}
