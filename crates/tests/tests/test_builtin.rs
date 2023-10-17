#[test]
#[cfg(feature = "test")]
fn test_vec() {
    tests::rt!(Vec<u8>, vec![1, 2, 3, 4]);
}
