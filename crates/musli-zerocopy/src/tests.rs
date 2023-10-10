use core::mem::{align_of, size_of};

use crate::{AlignedBuf, Error, ZeroCopy};

#[test]
fn test_weird_alignment() -> Result<(), Error> {
    #[derive(Debug, PartialEq, ZeroCopy)]
    #[repr(C, align(128))]
    #[zero_copy(crate)]
    struct WeirdAlignment {
        array: [u32; 3],
        field: u128,
    }

    let weird = WeirdAlignment {
        array: [0xffffffff, 0xffff0000, 0x0000ffff],
        field: 0x0000ffff0000ffff0000ffff0000ffffu128,
    };

    let mut buf = AlignedBuf::with_alignment(align_of::<WeirdAlignment>());
    let w = buf.store(&weird)?;
    let buf = buf.as_aligned();

    assert_eq!(buf.len(), size_of::<WeirdAlignment>());
    assert_eq!(buf.load(w)?, &weird);
    Ok(())
}
