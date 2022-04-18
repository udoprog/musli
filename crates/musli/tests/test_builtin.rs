#[test]
fn test_vec() {
    let data: Vec<u32> = vec![1, 2, 3, 4];
    musli_wire::test::rt(data);
}
