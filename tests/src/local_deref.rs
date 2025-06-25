pub trait LocalDeref {
    /// A local deref trait allowing us to implement deref for types.
    type Target: ?Sized;

    /// Returns a reference to the dereferenced value.
    fn local_deref(&self) -> &Self::Target;
}

impl<T> LocalDeref for &T
where
    T: ?Sized + LocalDeref,
{
    type Target = T::Target;

    #[inline]
    fn local_deref(&self) -> &Self::Target {
        (*self).local_deref()
    }
}

#[macro_export]
macro_rules! __local_deref_sized {
    (
        $(
            $(#[$($meta:meta)*])* $({$($p:ident),*})? $t:ty
            $(
                where $($bound:ident: $bound_path:path),* $(,)?
            )?
        ),* $(,)?
    ) => {
        $(
            $(#[$($meta)*])*
            impl $(<$($p),*>)* $crate::LocalDeref for $t
            $(where $($bound: $bound_path),*)*
            {
                type Target = Self;

                #[inline]
                fn local_deref(&self) -> &Self::Target {
                    self
                }
            }
        )+
    };
}

pub use __local_deref_sized as local_deref_sized;

macro_rules! deref {
    ($target:ty, $($(#[$($meta:meta)*])* $({$($p:ident),*})? $t:ty),* $(,)?) => {
        $(
            $(#[$($meta)*])*
            impl $(<$($p),*>)* LocalDeref for $t {
                type Target = $target;

                #[inline]
                fn local_deref(&self) -> &Self::Target {
                    self
                }
            }
        )+
    };
}

local_deref_sized! {
    bool,
    char,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    {T} core::num::Wrapping<T>,
    {T} core::num::Saturating<T>,
    core::num::NonZeroU8,
    core::num::NonZeroU16,
    core::num::NonZeroU32,
    core::num::NonZeroU64,
    core::num::NonZeroU128,
    core::num::NonZeroUsize,
    core::num::NonZeroI8,
    core::num::NonZeroI16,
    core::num::NonZeroI32,
    core::num::NonZeroI64,
    core::num::NonZeroI128,
    core::num::NonZeroIsize,
    #[cfg(feature = "alloc")]
    {T} alloc::collections::vec_deque::VecDeque<T>,
    #[cfg(feature = "alloc")]
    {T} alloc::collections::binary_heap::BinaryHeap<T>,
}

#[cfg(feature = "rkyv")]
local_deref_sized! {
    rkyv::rend::char_le,
    rkyv::rend::u16_le,
    rkyv::rend::u32_le,
    rkyv::rend::u64_le,
    rkyv::rend::u128_le,
    rkyv::rend::i16_le,
    rkyv::rend::i32_le,
    rkyv::rend::i64_le,
    rkyv::rend::i128_le,
    rkyv::rend::f32_le,
    rkyv::rend::f64_le,
    rkyv::rend::NonZeroU16_le,
    rkyv::rend::NonZeroU32_le,
    rkyv::rend::NonZeroU64_le,
    rkyv::rend::NonZeroU128_le,
    rkyv::rend::NonZeroI16_le,
    rkyv::rend::NonZeroI32_le,
    rkyv::rend::NonZeroI64_le,
    rkyv::rend::NonZeroI128_le,
}

#[cfg(feature = "alloc")]
deref! {
    str,
    str,
    alloc::string::String,
    #[cfg(feature = "rkyv")]
    rkyv::string::ArchivedString,
    #[cfg(feature = "rkyv")]
    rkyv::boxed::ArchivedBox<str>,
    #[cfg(feature = "rkyv")]
    rkyv::rc::ArchivedRc<str, rkyv::rc::RcFlavor>,
    #[cfg(feature = "rkyv")]
    rkyv::rc::ArchivedRc<str, rkyv::rc::ArcFlavor>,
}

#[cfg(feature = "std")]
deref! {
    std::path::Path,
    std::path::Path,
    std::path::PathBuf,
}

#[cfg(feature = "std")]
deref! {
    std::ffi::OsStr,
    std::ffi::OsStr,
    std::ffi::OsString,
}

deref! {
    core::ffi::CStr,
    core::ffi::CStr,
    #[cfg(feature = "alloc")]
    alloc::ffi::CString,
}

deref! {
    [T],
    {T} [T],
    #[cfg(feature = "alloc")]
    {T} alloc::vec::Vec<T>,
    #[cfg(feature = "rkyv")]
    {T} rkyv::vec::ArchivedVec<T>,
}

#[cfg(feature = "std")]
local_deref_sized! {
    {T} std::collections::BTreeSet<T>,
    {T} std::collections::HashSet<T>,
    {K, V} std::collections::BTreeMap<K, V>,
    {K, V} std::collections::HashMap<K, V>,
}

#[cfg(feature = "alloc")]
impl<T> LocalDeref for alloc::boxed::Box<T>
where
    T: ?Sized,
{
    type Target = T;

    #[inline]
    fn local_deref(&self) -> &Self::Target {
        self
    }
}

#[cfg(feature = "alloc")]
impl<T> LocalDeref for alloc::rc::Rc<T>
where
    T: ?Sized,
{
    type Target = T;

    #[inline]
    fn local_deref(&self) -> &Self::Target {
        self
    }
}

#[cfg(feature = "alloc")]
impl<T> LocalDeref for alloc::sync::Arc<T>
where
    T: ?Sized,
{
    type Target = T;

    #[inline]
    fn local_deref(&self) -> &Self::Target {
        self
    }
}
