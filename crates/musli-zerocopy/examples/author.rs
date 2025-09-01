use musli_zerocopy::{Error, OwnedBuf, Ref, ZeroCopy};

#[derive(ZeroCopy)]
#[repr(C)]
struct Person {
    age: u8,
    name: Ref<str>,
}

fn main() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let person = buf.store_uninit::<Person>()?;

    let value = Person {
        age: 35,
        name: buf.store_unsized("John-John")?,
    };

    buf.load_uninit_mut(person)?.write(&value);
    println!("{:?}", &buf[..]);
    Ok(())
}
