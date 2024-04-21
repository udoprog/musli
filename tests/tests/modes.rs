use musli::json::Encoding;
use musli::{Decode, Encode};

enum Alt {}

#[derive(Decode, Encode)]
#[musli(mode = Alt, packed)]
struct Word<'a> {
    text: &'a str,
    teineigo: bool,
}

const CONFIG: Encoding = Encoding::new();
const ALT_CONFIG: Encoding<Alt> = Encoding::new().with_mode();

#[test]
fn alt_serialization() {
    let word = Word {
        text: "あります",
        teineigo: true,
    };

    let out = CONFIG.to_string(&word).unwrap();
    assert_eq!(out, r#"{"text":"あります","teineigo":true}"#);

    let out = ALT_CONFIG.to_string(&word).unwrap();
    assert_eq!(out, r#"["あります",true]"#);
}
