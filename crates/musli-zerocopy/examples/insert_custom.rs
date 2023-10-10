use musli_zerocopy::{AlignedBuf, Error, Pair, Unsized, ZeroCopy};

#[derive(ZeroCopy)]
#[repr(C)]
struct Custom {
    field: u32,
    string: Unsized<str>,
}

fn main() -> Result<(), Error> {
    let mut buf = AlignedBuf::new();

    let string = buf.store_unsized("string")?;

    let custom1 = buf.store(&Custom { field: 1, string })?;
    let custom2 = buf.store(&Custom { field: 2, string })?;

    let mut map = Vec::new();

    map.push(Pair::new(1, custom1));
    map.push(Pair::new(2, custom2));

    let map = buf.insert_map(&mut map)?;

    let buf = buf.as_aligned();
    let map = buf.bind(map)?;

    let custom1 = map.get(&1)?.expect("Missing key 1");
    let custom1 = buf.load(custom1)?;
    assert_eq!(custom1.field, 1);
    assert_eq!(buf.load(custom1.string)?, "string");

    let custom2 = map.get(&2)?.expect("Missing key 2");
    let custom2 = buf.load(custom2)?;
    assert_eq!(custom2.field, 2);
    assert_eq!(buf.load(custom2.string)?, "string");

    assert!(map.get(&3)?.is_none());
    Ok(())
}
