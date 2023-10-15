use musli_zerocopy::{Error, OwnedBuf};

fn main() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let values = vec![buf.store_unsized("first"), buf.store_unsized("second")];

    let slice_ref = buf.store_slice(&values);

    let buf = buf.into_aligned();

    let slice = buf.load(slice_ref)?;

    let mut strings = Vec::new();

    for value in slice {
        strings.push(buf.load(*value)?);
    }

    assert_eq!(&strings, &["first", "second"][..]);
    Ok(())
}
