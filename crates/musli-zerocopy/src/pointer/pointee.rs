use crate::ZeroCopy;

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
pub trait Pointee<O: ?Sized> {
    /// Metadata associated with the pointee.
    type Metadata: Copy;

    /// The packed representation of the pointer metadata.
    type Packed: Copy;
}

impl<T, O> Pointee<O> for T
where
    T: ZeroCopy,
{
    type Metadata = ();
    type Packed = ();
}

impl<T, O> Pointee<O> for [T]
where
    T: ZeroCopy,
    O: Copy,
{
    type Metadata = usize;
    type Packed = O;
}

impl<O> Pointee<O> for str
where
    O: Copy,
{
    type Metadata = usize;
    type Packed = O;
}
