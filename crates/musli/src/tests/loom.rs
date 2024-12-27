use crate::alloc::{System, Vec};

const BIG1: &[u8] = &[
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
];
const BIG2: &[u8] = &[
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
];

fn work(alloc: System) {
    let mut buf1 = Vec::new_in(alloc);
    let mut buf2 = Vec::new_in(alloc);

    assert!(buf1.write(BIG1));
    assert!(buf2.write(BIG2));

    buf1.extend(buf2);
    assert!(buf1.as_slice().iter().eq(BIG1.iter().chain(BIG2)));
}

#[test]
fn test_concurrent_allocator() {
    loom::model(|| {
        let alloc = System::new();
        loom::thread::spawn(move || work(alloc));
        work(alloc);
    });
}
