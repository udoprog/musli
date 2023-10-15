use anyhow::{Context, Result};
use musli_zerocopy::phf;
use musli_zerocopy::pointer::Unsized;
use musli_zerocopy::{OwnedBuf, ZeroCopy};

#[derive(ZeroCopy)]
#[repr(C)]
struct Custom {
    field: u32,
    string: Unsized<str>,
}

fn main() -> Result<()> {
    let mut buf = OwnedBuf::new();

    let string = buf.store_unsized("string");

    let c1 = buf.store(&Custom { field: 1, string });
    let c2 = buf.store(&Custom { field: 2, string });

    let map = phf::store_map(&mut buf, [(1, c1), (2, c2)])?;
    let buf = buf.into_aligned();
    let map = buf.bind(map)?;

    let c1 = buf.load(map.get(&1)?.context("Missing key 1")?)?;
    assert_eq!(c1.field, 1);
    assert_eq!(buf.load(c1.string)?, "string");

    let c2 = buf.load(map.get(&2)?.context("Missing key 2")?)?;
    assert_eq!(c2.field, 2);
    assert_eq!(buf.load(c2.string)?, "string");

    assert!(map.get(&3)?.is_none());
    Ok(())
}
