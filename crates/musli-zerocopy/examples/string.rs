use musli_zerocopy::{Error, OwnedBuf};

fn main() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let first = buf.store_unsized("first");
    let second = buf.store_unsized("second");

    let buf = buf.as_ref();

    assert_eq!(buf.load(first)?, "first");
    assert_eq!(buf.load(second)?, "second");
    Ok(())
}
