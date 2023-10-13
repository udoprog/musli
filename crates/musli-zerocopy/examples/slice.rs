use musli_zerocopy::{AlignedBuf, Error};

fn main() -> Result<(), Error> {
    let mut buf = AlignedBuf::new();

    let values = vec![buf.store_unsized("first"), buf.store_unsized("second")];

    let slice_ref = buf.store_slice(&values);

    let buf = buf.as_aligned();

    let slice = buf.load(slice_ref)?;

    let mut strings = Vec::new();

    for value in slice {
        strings.push(buf.load(*value)?);
    }

    assert_eq!(&strings, &["first", "second"][..]);
    Ok(())
}
