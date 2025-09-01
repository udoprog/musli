use musli_zerocopy::mem::PackedMaybeUninit;
use musli_zerocopy::{Error, OwnedBuf, Ref, ZeroCopy};

fn main() -> Result<(), Error> {
    #[derive(ZeroCopy)]
    #[repr(C)]
    struct Custom {
        string: Ref<str>,
    }

    let mut buf = OwnedBuf::with_capacity_and_alignment::<u8>(128)?;
    buf.extend_from_slice(&[1])?;

    let reference: Ref<PackedMaybeUninit<Custom>> = buf.store_uninit::<Custom>()?;

    let string = buf.store_unsized("Hello World!")?;

    buf.load_uninit_mut(reference)?.write(&Custom { string });

    buf.align_in_place()?;

    let reference = reference.assume_init();

    assert_eq!(reference.offset(), 4);

    let custom = buf.load(reference)?;
    assert_eq!(buf.load(custom.string)?, "Hello World!");
    Ok(())
}
