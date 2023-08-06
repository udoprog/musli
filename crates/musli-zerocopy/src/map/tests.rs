use crate::{Buf, Error, Map, MapBuilder, OwnedBuf, Slice, SliceBuilder, SliceRef};

#[test]
fn map() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let first = buf.insert_unsized("first");
    let second = buf.insert_unsized("second");

    let mut builder = MapBuilder::<u32, _>::new();
    builder.insert(1, first);
    builder.insert(2, second);
    let first = builder.build(&mut buf)?;

    let buf = buf.as_buf();

    let first: Map<u32, _> = buf.read(first)?;

    assert_eq!(first.get(&1)?.expect("missing 1"), "first");
    assert_eq!(first.get(&2)?.expect("missing 2"), "second");
    Ok(())
}

#[test]
fn slice() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let mut builder = SliceBuilder::new();
    builder.push(42);
    let first = builder.build(&mut buf);

    let mut builder = SliceBuilder::new();
    builder.push(43);
    builder.push(44);
    let second = builder.build(&mut buf);

    let mut builder = SliceBuilder::new();
    builder.push(first);
    builder.push(second);
    let repr: SliceRef<SliceRef<u32>> = builder.build(&mut buf);

    let value_ref = buf.insert(42u32);

    let data = buf.as_slice();
    let buf = Buf::new(data);

    let slice: Slice<Slice<u32>> = buf.read(repr)?;

    assert_eq!(
        slice
            .get(0)?
            .expect("missing 0")
            .get(0)?
            .expect("missing 0.0"),
        42
    );
    assert_eq!(
        slice
            .get(1)?
            .expect("missing 1")
            .get(0)?
            .expect("missing 1.0"),
        43
    );
    assert_eq!(
        slice
            .get(1)?
            .expect("missing 1")
            .get(1)?
            .expect("missing 1.1"),
        44
    );
    assert_eq!(buf.read(value_ref)?, 42);
    Ok(())
}
