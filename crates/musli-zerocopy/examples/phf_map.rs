use musli_zerocopy::phf;
use musli_zerocopy::{Error, OwnedBuf};

fn main() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let string = buf.store_unsized("Hello World!");

    let values = phf::store_map(&mut buf, [(10u32, string), (20u32, string)])?;

    buf.align_in_place();

    let values = buf.bind(values)?;

    let string = *values.get(&10u32)?.expect("expected element at 10");
    assert_eq!(buf.load(string)?, "Hello World!");
    assert!(values.get(&30u32)?.is_none());
    Ok(())
}
