use musli_zerocopy::swiss;
use musli_zerocopy::{AlignedBuf, Error};

fn main() -> Result<(), Error> {
    let mut buf = AlignedBuf::new();

    let values = vec![(10u32, 1u32), (20u32, 2u32)];

    let values = swiss::store_map(&mut buf, &values)?;

    let buf = buf.as_aligned();

    assert_eq!(values.get(buf, &10u32)?, Some(&1));
    assert_eq!(values.get(buf, &20u32)?, Some(&2));
    assert_eq!(values.get(buf, &30u32)?, None);
    Ok(())
}
