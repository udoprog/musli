use musli_zerocopy::ZeroCopy;
use musli_zerocopy::{Error, OwnedBuf, UnsizedRef};

#[derive(ZeroCopy)]
#[repr(C)]
struct Custom {
    field: u32,
    string: UnsizedRef<str>,
}

fn main() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let string = buf.insert_unsized("string")?;
    let custom = buf.insert_sized(Custom { field: 1, string })?;
    let custom2 = buf.insert_sized(Custom { field: 2, string })?;

    let buf = buf.as_buf()?;

    let custom = buf.load(custom)?;
    assert_eq!(custom.field, 1);
    assert_eq!(buf.load_unsized(custom.string)?, "string");

    let custom2 = buf.load(custom2)?;
    assert_eq!(custom2.field, 2);
    assert_eq!(buf.load_unsized(custom2.string)?, "string");
    Ok(())
}
