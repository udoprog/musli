use musli_zerocopy::{Error, OwnedBuf};

fn main() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let first = buf.insert_unsized("first")?;
    let second = buf.insert_unsized("second")?;

    let buf = buf.as_buf()?;

    assert_eq!(buf.load(first)?, "first");
    assert_eq!(buf.load(second)?, "second");
    Ok(())
}
