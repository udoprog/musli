use musli_zerocopy::endian::Native;
use musli_zerocopy::mem::MaybeUninit;
use musli_zerocopy::{Buf, OwnedBuf, Ref};

#[test]
#[should_panic]
fn test_swap() {
    let mut buf = [1];
    let buf = unsafe { Buf::new_mut(&mut buf) };
    let r1 = Ref::<u8, Native, usize>::new(usize::MAX);
    let r2 = Ref::<u8, Native, usize>::new(0);
    buf.swap(r1, r2).unwrap();
}

#[test]
#[should_panic]
fn test_load_uninit() {
    let mut buf = OwnedBuf::new().with_size::<usize>();
    let r = Ref::<MaybeUninit<u8>, Native, usize>::new(usize::MAX);
    buf.load_uninit_mut(r);
}
