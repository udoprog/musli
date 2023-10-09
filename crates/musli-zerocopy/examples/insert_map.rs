use musli_zerocopy::{Error, OwnedBuf, Pair};

fn main() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let mut values = Vec::new();

    values.push(Pair::new(buf.insert_unsized("first")?, 1u32));
    values.push(Pair::new(buf.insert_unsized("second")?, 2u32));

    let values = buf.insert_map(&mut values)?;

    let buf = buf.as_aligned_buf();

    assert_eq!(values.get(buf, &"first")?, Some(&1));
    assert_eq!(values.get(buf, &"second")?, Some(&2));
    assert_eq!(values.get(buf, &"third")?, None);
    Ok(())
}
