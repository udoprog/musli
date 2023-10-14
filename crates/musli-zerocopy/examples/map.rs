use musli_zerocopy::phf::{self, Entry};
use musli_zerocopy::{AlignedBuf, Error};

fn main() -> Result<(), Error> {
    let mut buf = AlignedBuf::new();

    let mut values = Vec::new();

    values.push(Entry::new(10u32, 1u32));
    values.push(Entry::new(20u32, 2u32));

    let values = phf::store_map(&mut buf, &mut values)?;

    let buf = buf.as_aligned();
    let values = buf.bind(values)?;

    assert_eq!(values.get(&10u32)?, Some(&1));
    assert_eq!(values.get(&20u32)?, Some(&2));
    assert_eq!(values.get(&30u32)?, None);
    Ok(())
}
