//! Example showcasing how to build an archive which contains mixed endian data.

use anyhow::Result;
use musli_zerocopy::endian::{Big, Little};
use musli_zerocopy::{ByteOrder, Endian, OwnedBuf, Ref, ZeroCopy};

#[derive(ZeroCopy)]
#[repr(C)]
struct Header {
    big: Ref<Data<Big>, Little>,
    little: Ref<Data<Little>, Little>,
}

#[derive(ZeroCopy)]
#[repr(C)]
struct Data<E>
where
    E: ByteOrder,
{
    name: Ref<str, E>,
    age: Endian<u32, E>,
}

fn main() -> Result<()> {
    let mut buf = OwnedBuf::new();

    let header = buf.store_uninit::<Header>();

    // Byte-oriented data has no alignment, so we can re-use the string
    // allocation.
    let name = buf.store_unsized("John Doe");

    let big = buf.store(&Data {
        name: name.to_endian(),
        age: Endian::new(35),
    });

    let little = buf.store(&Data {
        name: name.to_endian(),
        age: Endian::new(35),
    });

    buf.load_uninit_mut(header).write(&Header { big, little });

    buf.align_in_place();

    let header = buf.load_at::<Header>(0)?;
    let little = buf.load(header.little)?;
    let big = buf.load(header.big)?;

    dbg!(buf.load(little.name)?);
    dbg!(little.age.to_ne(), little.age.to_raw());

    dbg!(buf.load(big.name)?);
    dbg!(big.age.to_ne(), big.age.to_raw());
    Ok(())
}
