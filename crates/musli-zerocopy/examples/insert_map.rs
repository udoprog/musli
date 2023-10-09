use musli_zerocopy::{Error, OwnedBuf, Pair};

fn main() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let mut values = Vec::new();

    values.push(Pair::new(10u32, 1u32));
    values.push(Pair::new(20u32, 2u32));

    let values = buf.insert_map(&mut values)?;

    let buf = buf.as_aligned_buf();
    let values = buf.bind(values)?;

    assert_eq!(values.get(&10u32)?, Some(&1));
    assert_eq!(values.get(&20u32)?, Some(&2));
    assert_eq!(values.get(&30u32)?, None);
    Ok(())
}
