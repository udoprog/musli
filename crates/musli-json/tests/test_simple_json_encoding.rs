use musli::{Decode, Encode};
use musli_json::JsonEncoding;

// Mode marker indicating that some attributes should only apply when we're
// decoding in a JSON mode.
mod my_modes {
    pub(crate) enum Json {}
}

const CONFIG: JsonEncoding<my_modes::Json> = JsonEncoding::new();

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(mode = my_modes::Json, default_field_tag = "name")]
struct SimpleJsonStruct<'a> {
    name: &'a str,
    age: u32,
}

#[test]
fn test_simple_encoding() {
    let expected = vec![
        SimpleJsonStruct {
            name: "Aristotle",
            age: 61,
        },
        SimpleJsonStruct {
            name: "Socrates",
            age: 12,
        },
        SimpleJsonStruct {
            name: "Plato",
            age: 42,
        },
    ];

    let out = CONFIG.to_vec(&expected).unwrap();
    println!("{}", String::from_utf8(out).unwrap());

    let out = musli_json::to_vec(&expected).unwrap();
    println!("{}", String::from_utf8(out).unwrap());
    // let actual = CONFIG.decode(&out[..]).unwrap();

    // assert_eq!(expected, actual);
}
