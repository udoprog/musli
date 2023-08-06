use crate::traits::Size;
use crate::OwnedBuf;

#[test]
fn owned_buf_swap() {
    let mut buf = OwnedBuf::new();

    let a = buf.insert(1u32);
    let b = buf.insert(2u32);
    let c = buf.insert(3u32);

    buf.swap(0, 0, 1, u32::size());

    assert_eq!(buf.as_buf().read(a).unwrap(), 2u32);
    assert_eq!(buf.as_buf().read(b).unwrap(), 1u32);
    assert_eq!(buf.as_buf().read(c).unwrap(), 3u32);
}
