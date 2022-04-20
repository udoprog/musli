use musli::{Decode, Encode};

/// Empty enums should work.
#[derive(Debug, Encode, Decode)]
enum EmptyEnum {}

#[test]
fn test_decode_empty() {
    let e = musli_tests::wire::decode::<_, EmptyEnum>(&[][..]).unwrap_err();
    assert_eq!(
        e.to_string(),
        "EmptyEnum: cannot decode uninhabitable types"
    );
}
