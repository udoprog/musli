use musli_zerocopy::{Error, OwnedBuf};

fn main() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let mut values = Vec::new();

    values.push(buf.insert_unsized("first")?);
    values.push(buf.insert_unsized("second")?);

    let slice_ref = buf.insert_slice(&values)?;

    let buf = buf.as_aligned_buf();

    // assert_eq!(buf.load(first)?, "first");
    // assert_eq!(buf.load(second)?, "second");
    Ok(())
}
