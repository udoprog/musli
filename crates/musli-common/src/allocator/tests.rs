use crate::allocator::Allocator;
use musli::context::Buffer;

#[test]
fn test_allocator() {
    let alloc = crate::allocator::Default::default();
    let alloc = &alloc;

    let mut a = alloc.alloc();
    let mut b = alloc.alloc();

    b.write(b"He11o");
    a.copy_back(b);

    assert_eq!(a.as_slice(), b"He11o");
    assert_eq!(a.len(), 5);

    a.write(b" W0rld");

    assert_eq!(a.as_slice(), b"He11o W0rld");
    assert_eq!(a.len(), 11);

    let mut c = alloc.alloc();
    c.write(b"!");
    assert!(a.write_at(7, b"o"));
    assert!(!a.write_at(11, b"!"));
    a.copy_back(c);

    assert_eq!(a.as_slice(), b"He11o World!");
    assert_eq!(a.len(), 12);

    assert!(a.write_at(2, b"ll"));

    assert_eq!(a.as_slice(), b"Hello World!");
    assert_eq!(a.len(), 12);
}
