use musli_zerocopy::buf::AlignedBuf;
use musli_zerocopy::mem::MaybeUninit;
use musli_zerocopy::pointer::{Ref, Unsized};
use musli_zerocopy::{Error, ZeroCopy};

fn main() -> Result<(), Error> {
    #[derive(ZeroCopy)]
    #[repr(C)]
    struct Custom {
        string: Unsized<str>,
    }

    let mut buf = AlignedBuf::with_capacity_and_alignment::<u8>(128);
    buf.extend_from_slice(&[1]);

    let reference: Ref<MaybeUninit<Custom>> = buf.store_uninit::<Custom>();

    let string = buf.store_unsized("Hello World!");

    buf.load_uninit_mut(reference).write(&Custom { string });

    let buf = buf.as_aligned();
    let reference = reference.assume_init();

    assert_eq!(reference.offset(), 4);

    let custom = buf.load(reference)?;
    assert_eq!(buf.load(custom.string)?, "Hello World!");
    Ok(())
}
