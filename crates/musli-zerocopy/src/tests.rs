use crate as musli_zerocopy;
use crate::{AlignedBuf, Error, ZeroCopy};

#[derive(Debug, PartialEq, ZeroCopy)]
#[repr(C, align(128))]
struct WeirdAlignment {
    fields: [u32; 3],
}

#[test]
fn test_weird_alignment() -> Result<(), Error> {
    let mut buf = AlignedBuf::new();

    let weird = WeirdAlignment {
        fields: [0xffffffff, 0xffff0000, 0x0000ffff],
    };

    let w = buf.write(&weird)?;
    let buf = buf.as_aligned_buf();

    std::dbg!(buf.as_bytes());
    assert_eq!(buf.load(w)?, &weird);
    Ok(())
}
