use crate::ZeroCopy;

/// A type that can inhabit a packed representation.
pub trait Packable: Sized {
    /// The packed representation of the item.
    type Packed<O>: Copy + ZeroCopy
    where
        O: Copy + ZeroCopy;
}

impl Packable for () {
    type Packed<O> = () where O: Copy + ZeroCopy;
}

impl Packable for usize {
    type Packed<O> = O where O: Copy + ZeroCopy;
}

/// The trait for a value that can be pointed to by a [`Ref<P>`].
///
/// This ultimately determines the layout of [`Ref<P>`] as for unsized types it
/// needs to accommodate the size of the pointed-to type as well.
///
/// ```
/// use std::mem::size_of;
///
/// use musli_zerocopy::Ref;
///
/// assert_eq!(size_of::<Ref::<u32>>(), 4);
/// assert_eq!(size_of::<Ref::<[u32]>>(), 8);
/// ```
///
/// [`Ref<P>`]: crate::Ref
pub trait Pointee {
    /// Metadata associated with the pointee.
    type Metadata: Packable;
}

impl<T> Pointee for T
where
    T: ZeroCopy,
{
    type Metadata = ();
}

impl<T> Pointee for [T]
where
    T: ZeroCopy,
{
    type Metadata = usize;
}

impl Pointee for str {
    type Metadata = usize;
}
