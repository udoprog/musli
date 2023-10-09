use musli_zerocopy::{Error, OwnedBuf};

fn main() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let values = vec![buf.insert_unsized("first")?, buf.insert_unsized("second")?];

    let slice_ref = buf.insert_slice(&values)?;

    let buf = buf.as_aligned_buf();

    let slice = buf.load(slice_ref)?;

    let mut strings = Vec::new();

    for value in slice {
        strings.push(buf.load(*value)?);
    }

    assert_eq!(&strings, &["first", "second"][..]);
    Ok(())
}
