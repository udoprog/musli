use musli_zerocopy::phf::{self, Entry};
use musli_zerocopy::{AlignedBuf, Error};

fn main() -> Result<(), Error> {
    let mut buf = AlignedBuf::new();

    let string = buf.store_unsized("Hello World!");

    let mut values = Vec::new();

    values.push(Entry::new(10u32, string));
    values.push(Entry::new(20u32, string));

    let values = phf::store_map(&mut buf, &mut values)?;

    let buf = buf.as_aligned();

    let values = buf.bind(values)?;

    let string = *values.get(&10u32)?.expect("expected element at 10");
    assert_eq!(buf.load(string)?, "Hello World!");
    assert!(values.get(&30u32)?.is_none());
    Ok(())
}
