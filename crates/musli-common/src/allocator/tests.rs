use crate::allocator::Allocator;
use musli::context::Buf;

fn basic_allocations<A: Allocator>(alloc: &A) {
    let mut a = alloc.alloc().unwrap();
    let mut b = alloc.alloc().unwrap();

    b.write(b"He11o");

    assert_eq!(b.as_slice(), b"He11o");
    assert_eq!(b.len(), 5);

    a.write(b.as_slice());

    assert_eq!(a.as_slice(), b"He11o");
    assert_eq!(a.len(), 5);

    a.write(b" W0rld");

    assert_eq!(a.as_slice(), b"He11o W0rld");
    assert_eq!(a.len(), 11);

    let mut c = alloc.alloc().unwrap();
    c.write(b"!");
    assert_eq!(c.len(), 1);

    a.write(c.as_slice());
    assert_eq!(a.as_slice(), b"He11o W0rld!");
    assert_eq!(a.len(), 12);
}

#[test]
fn alloc_basic() {
    let mut buf = crate::allocator::SystemBuffer::new();
    let alloc = crate::allocator::System::new(&mut buf);
    basic_allocations(&alloc);
}

#[test]
fn nostd_basic() {
    let mut buf = crate::allocator::StackBuffer::<4096>::new();
    let alloc = crate::allocator::Stack::new(&mut buf);
    basic_allocations(&alloc);
}
