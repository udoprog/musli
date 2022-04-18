#[test]
fn test_vec() -> Result<(), Box<dyn std::error::Error>> {
    let data: Vec<u32> = vec![1, 2, 3, 4];
    musli_wire::test::rt(data)?;
    Ok(())
}
