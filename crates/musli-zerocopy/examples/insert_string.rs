use musli_zerocopy::{AlignedBuf, Error};

fn main() -> Result<(), Error> {
    let mut buf = AlignedBuf::new();

    let first = buf.write_unsized("first")?;
    let second = buf.write_unsized("second")?;

    let buf = buf.as_buf()?;

    assert_eq!(buf.load(first)?, "first");
    assert_eq!(buf.load(second)?, "second");
    Ok(())
}
