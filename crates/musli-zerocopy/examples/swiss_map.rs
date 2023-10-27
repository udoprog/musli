use musli_zerocopy::swiss;
use musli_zerocopy::{Error, OwnedBuf};

fn main() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let values = swiss::store_map(&mut buf, [(10u32, 1u32), (20u32, 2u32)])?;

    buf.align_in_place();

    let values = buf.bind(values)?;

    assert_eq!(values.get(&10u32)?, Some(&1));
    assert_eq!(values.get(&20u32)?, Some(&2));
    assert_eq!(values.get(&30u32)?, None);
    Ok(())
}
