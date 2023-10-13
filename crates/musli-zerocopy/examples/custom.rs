use anyhow::{Context, Result};
use musli_zerocopy::map::Entry;
use musli_zerocopy::pointer::Unsized;
use musli_zerocopy::{AlignedBuf, ZeroCopy};

#[derive(ZeroCopy)]
#[repr(C)]
struct Custom {
    field: u32,
    string: Unsized<str>,
}

fn main() -> Result<()> {
    let mut buf = AlignedBuf::new();

    let string = buf.store_unsized("string");

    let c1 = buf.store(&Custom { field: 1, string });
    let c2 = buf.store(&Custom { field: 2, string });

    let mut map = vec![Entry::new(1, c1), Entry::new(2, c2)];

    let map = buf.store_map(&mut map)?;
    let buf = buf.as_aligned();
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
