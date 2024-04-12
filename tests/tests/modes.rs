use musli::mode::DefaultMode;
use musli::{Decode, Encode};
use musli_json::Encoding;

enum Alt {}

#[derive(Decode, Encode)]
#[musli(mode = Alt, packed)]
#[musli(name_all = "name")]
struct Word<'a> {
    text: &'a str,
    teineigo: bool,
}

const CONFIG: Encoding<DefaultMode> = Encoding::new();
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
