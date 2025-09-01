//! Example showcasing how to build an archive which contains mixed endian data.

use anyhow::Result;

use musli_zerocopy::endian;
use musli_zerocopy::{ByteOrder, Endian, OwnedBuf, Ref, ZeroCopy};

#[derive(ZeroCopy)]
#[repr(C)]
struct Header {
    big: Ref<Data<endian::Big>, endian::Big>,
    little: Ref<Data<endian::Little>, endian::Little>,
}

#[derive(ZeroCopy)]
#[repr(C)]
struct Data<E = endian::Native>
where
    E: ByteOrder,
{
    name: Ref<str, E>,
    age: Endian<u32, E>,
}

fn main() -> Result<()> {
    let mut buf = OwnedBuf::new();

    let header = buf.store_uninit::<Header>()?;

    // Byte-oriented data has no alignment, so we can re-use the string
    // allocation.
    let name = buf.store_unsized("John Doe")?;

    let big = buf.store(&Data {
        name: name.to_endian(),
        age: Endian::new(35),
    })?;

    let little = buf.store(&Data {
        name: name.to_endian(),
        age: Endian::new(35),
    })?;

    buf.load_uninit_mut(header)?.write(&Header {
        big: big.to_endian(),
        little: little.to_endian(),
    });

    buf.align_in_place()?;

    let header = buf.load_at::<Header>(0)?;
    let data = buf.load(endian::pick!("big" => header.big, "little" => header.little))?;

    dbg!(buf.load(data.name)?);
    dbg!(*data.age + 1);
    Ok(())
}
