use musli_zerocopy::{Error, OwnedBuf, Pair, UnsizedRef, ZeroCopy};

#[derive(ZeroCopy)]
#[repr(C)]
struct Custom {
    field: u32,
    string: UnsizedRef<str>,
}

fn main() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let string = buf.insert_unsized("string")?;

    let custom1 = buf.insert_sized(Custom { field: 1, string })?;
    let custom2 = buf.insert_sized(Custom { field: 2, string })?;

    let mut map = Vec::new();

    map.push(Pair::new(1, custom1));
    map.push(Pair::new(2, custom2));

    let map = buf.insert_map(&mut map)?;

    let buf = buf.as_aligned_buf();

    if let Some(custom1) = map.get(buf, &1)? {
        let custom1 = buf.load(custom1)?;
        assert_eq!(custom1.field, 1);
        assert_eq!(buf.load(custom1.string)?, "string");
    } else {
        panic!("Missing key 1")
    }

    if let Some(custom2) = map.get(buf, &2)? {
        let custom2 = buf.load(custom2)?;
        assert_eq!(custom2.field, 2);
        assert_eq!(buf.load(custom2.string)?, "string");
    } else {
        panic!("Missing key 2");
    }

    assert!(map.get(buf, &3)?.is_none());
    Ok(())
}
