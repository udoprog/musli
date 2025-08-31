#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

/// Default random seed to use.
pub const RNG_SEED: u64 = 2718281828459045235;

#[cfg(feature = "alloc")]
use alloc::collections::binary_heap::BinaryHeap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub use tests_macros::benchmarker;

mod local_deref;
pub(crate) use self::local_deref::local_deref_sized;
pub use local_deref::LocalDeref;

#[macro_export]
macro_rules! options {
    (
        $init_vis:vis unsafe fn $init:ident();
        $enumerate_vis:vis fn $enumerate:ident();
        $($(#[$($meta:meta)*])* $vis:vis static $ident:ident: $value_ty:ty = $range:expr, $miri:expr;)*
    ) => {
        /// Initialize the specified statics.
        ///
        /// # Safety
        ///
        /// Must only be called ONCE at the start of a program.
        $init_vis unsafe fn $init() {
            $(unsafe {
                let key = concat!("MUSLI_", stringify!($ident));

                if let Ok(var) = ::std::env::var(key) {
                    if let Some(value) = $crate::parse::<$value_ty>(&var) {
                        _ = $ident.replace(value);
                    } else {
                        std::eprintln!("Could not parse {key}={var}")
                    }
                }
            })*
        }

        #[allow(unused)]
        $enumerate_vis fn $enumerate<F, E>(mut out: F) -> Result<(), E>
        where
            F: FnMut(&str, &dyn core::fmt::Debug) -> Result<(), E>
        {
            $({
                let key = concat!("MUSLI_", stringify!($ident));
                let value = $ident.get();
                out(key, &value)?;
            })*

            Ok(())
        }

        $(
            $(#[$($meta)*])*
            #[cfg(miri)]
            $vis static $ident: $crate::Opt<$value_ty> = $crate::Opt::new($miri);
            $(#[$($meta)*])*
            #[cfg(not(miri))]
            $vis static $ident: $crate::Opt<$value_ty> = $crate::Opt::new($range);
        )*
    }
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for ::core::ops::Range<usize> {}
    impl Sealed for usize {}
}

pub trait Parse: Sized + self::sealed::Sealed {
    fn parse(input: &str) -> Option<Self>;
}

impl Parse for ::core::ops::Range<usize> {
    #[inline]
    fn parse(input: &str) -> Option<Self> {
        if let Some((from, to)) = input.split_once("..=") {
            let from = from.parse().ok()?;
            let to: usize = to.parse().ok()?;
            return Some(from..(to + 1));
        }

        if let Some((from, to)) = input.split_once("..") {
            return Some(from.parse().ok()?..to.parse().ok()?);
        }

        let value: usize = input.parse().ok()?;
        Some(value..(value + 1))
    }
}

impl Parse for usize {
    #[inline]
    fn parse(input: &str) -> Option<Self> {
        input.parse().ok()
    }
}

#[doc(hidden)]
pub fn parse<T: Parse>(input: &str) -> Option<T> {
    T::parse(input)
}

/// Initialize the specified statics.
///
/// # Safety
///
/// Must only be called ONCE at the start of a program.
pub unsafe fn init_statics() {
    unsafe {
        self::models::init_ranges();
        self::generate::init_ranges();
    }
}

/// Enumerate all available statics.
pub fn enumerate_statics<F, E>(mut out: F) -> Result<(), E>
where
    F: FnMut(&str, &dyn core::fmt::Debug) -> Result<(), E>,
{
    self::models::enumerate_ranges(&mut out)?;
    self::generate::enumerate_ranges(&mut out)?;
    Ok(())
}

pub mod generate;
#[doc(inline)]
pub use self::generate::{Generate, Rng};
#[cfg(feature = "musli")]
pub mod mode;
pub mod models;
mod opt;
pub mod utils;
#[doc(hidden)]
pub use self::opt::Opt;

pub use self::aligned_buf::AlignedBuf;
mod aligned_buf;

/// Call the given macro with the existing feature matrix.
#[macro_export]
macro_rules! feature_matrix {
    ($call:path $(, $($tt:tt)*)?) => {
        $call!(mock $(, $($tt)*)*);
        #[cfg(feature = "musli-json")]
        $call!(musli_json $(, $($tt)*)*);
        #[cfg(feature = "musli-wire")]
        $call!(musli_wire $(, $($tt)*)*);
        #[cfg(feature = "musli-descriptive")]
        $call!(musli_descriptive $(, $($tt)*)*);
        #[cfg(feature = "musli-storage")]
        $call!(musli_storage $(, $($tt)*)*);
        #[cfg(feature = "musli-packed")]
        $call!(musli_packed $(, $($tt)*)*);
        #[cfg(feature = "musli-value")]
        $call!(musli_value $(, $($tt)*)*);
        #[cfg(feature = "musli-zerocopy")]
        $call!(musli_zerocopy $(, $($tt)*)*);
        #[cfg(feature = "serde_json")]
        $call!(serde_json $(, $($tt)*)*);
        #[cfg(feature = "simd-json")]
        $call!(simd_json $(, $($tt)*)*);
        #[cfg(feature = "bincode1")]
        $call!(bincode1 $(, $($tt)*)*);
        #[cfg(feature = "bincode-serde")]
        $call!(bincode_serde $(, $($tt)*)*);
        #[cfg(feature = "bincode-derive")]
        $call!(bincode_derive $(, $($tt)*)*);
        #[cfg(feature = "rmp-serde")]
        $call!(serde_rmp $(, $($tt)*)*);
        #[cfg(feature = "zerocopy")]
        $call!(zerocopy $(, $($tt)*)*);
        #[cfg(feature = "epserde")]
        $call!(epserde $(, $($tt)*)*);
        #[cfg(feature = "dlhn")]
        $call!(serde_dlhn $(, $($tt)*)*);
        #[cfg(feature = "serde_cbor")]
        $call!(serde_cbor $(, $($tt)*)*);
        #[cfg(all(feature = "bitcode", feature = "serde"))]
        $call!(serde_bitcode $(, $($tt)*)*);
        #[cfg(feature = "bitcode-derive")]
        $call!(derive_bitcode $(, $($tt)*)*);
        #[cfg(feature = "rkyv")]
        $call!(rkyv $(, $($tt)*)*);
        #[cfg(feature = "postcard")]
        $call!(postcard $(, $($tt)*)*);
        #[cfg(feature = "bson")]
        $call!(bson $(, $($tt)*)*);
        #[cfg(feature = "miniserde")]
        $call!(miniserde $(, $($tt)*)*);
        #[cfg(feature = "speedy")]
        $call!(speedy $(, $($tt)*)*);
    };
}

/// Only expand `$block` if the given test is supported by this framework.
#[macro_export]
macro_rules! if_supported {
    (musli_value, decode_bytes, $($tt:tt)*) => {};

    (musli_zerocopy, large, $($tt:tt)*) => {};
    (musli_zerocopy, allocated, $($tt:tt)*) => {};
    (musli_zerocopy, full_enum, $($tt:tt)*) => {};
    (musli_zerocopy, mesh, $($tt:tt)*) => {};

    (zerocopy, primitives, $($tt:tt)*) => {};
    (zerocopy, large, $($tt:tt)*) => {};
    (zerocopy, allocated, $($tt:tt)*) => {};
    (zerocopy, full_enum, $($tt:tt)*) => {};
    (zerocopy, mesh, $($tt:tt)*) => {};

    (epserde, large, $($tt:tt)*) => {};
    (epserde, allocated, $($tt:tt)*) => {};
    (epserde, full_enum, $($tt:tt)*) => {};

    (bincode_derive, mesh, $($tt:tt)*) => {};

    ($framework:ident, $test:ident, $($tt:tt)*) => { $($tt)* };
}

#[macro_export]
macro_rules! types {
    ($call:path) => {
        $call!(primitives, Primitives, PRIMITIVES, 1000);
        $call!(packed, Packed, PACKED, 1000);
        #[cfg(feature = "alloc")]
        $call!(large, Large, LARGE, 10000);
        #[cfg(feature = "alloc")]
        $call!(allocated, Allocated, ALLOCATED, 5000);
        #[cfg(any(not(feature = "no-empty"), not(feature = r#"no-nonunit-variant"#)))]
        $call!(full_enum, FullEnum, FULL_ENUM, 1000);
        #[cfg(feature = "alloc")]
        $call!(mesh, Mesh, MESHES, 1000);
    };
}

#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr, $message:literal $($tt:tt)*) => {
        if !$crate::partial_eq($left, $right) {
            panic!($message $($tt)*);
        }
    }
}

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! debug_assert_eq {
    ($left:expr, $right:expr $(,)?) => {
        if !$crate::partial_eq($left, $right) {
            panic!(
                "assertion failed: `(left == right)`\n  left: `{:?}`,\n right: `{:?}`",
                $left, $right
            );
        }
    };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! debug_assert_eq {
    ($left:expr, $right:expr $(,)?) => {};
}

#[doc(hidden)]
#[inline]
pub fn partial_eq<A, B>(a: &A, b: &B) -> bool
where
    A: ?Sized + LocalDeref<Target: PartialEq<B::Target>>,
    B: ?Sized + LocalDeref,
{
    PartialEq::eq(a.local_deref(), b.local_deref())
}

#[doc(hidden)]
#[inline]
#[cfg(feature = "alloc")]
pub fn binary_heap_eq<A, B>(a: &BinaryHeap<A>, b: &BinaryHeap<B>) -> bool
where
    A: Ord + PartialEq<B>,
    B: Ord,
{
    if a.len() != b.len() {
        return false;
    }

    let mut a = a.iter().collect::<Vec<_>>();
    let mut b = b.iter().collect::<Vec<_>>();

    a.sort();
    b.sort();

    a == b
}

#[doc(hidden)]
pub trait AtomicEq<B = Self>
where
    B: ?Sized,
{
    fn atomic_eq(&self, other: &B) -> bool;
}

impl<A, B> AtomicEq<&B> for &A
where
    A: ?Sized + AtomicEq<B>,
    B: ?Sized,
{
    #[inline]
    fn atomic_eq(&self, other: &&B) -> bool {
        AtomicEq::atomic_eq(*self, *other)
    }
}

macro_rules! atomic_impl {
    ($size:literal $(, $ty:ident)*) => {
        $(
            #[cfg(target_has_atomic = $size)]
            impl AtomicEq for ::core::sync::atomic::$ty {
                #[inline]
                fn atomic_eq(&self, other: &Self) -> bool {
                    use core::sync::atomic::Ordering::Relaxed;

                    self.load(Relaxed) == other.load(Relaxed)
                }
            }
        )*
    }
}

atomic_impl!("8", AtomicBool, AtomicI8, AtomicU8);
atomic_impl!("16", AtomicI16, AtomicU16);
atomic_impl!("32", AtomicI32, AtomicU32);
atomic_impl!("64", AtomicI64, AtomicU64);
atomic_impl!("ptr", AtomicIsize, AtomicUsize);

#[doc(hidden)]
#[inline]
pub fn atomic_eq<A, B>(a: A, b: B) -> bool
where
    A: AtomicEq<B>,
{
    a.atomic_eq(&b)
}

#[macro_export]
macro_rules! basic_types {
    ($call:path $(, $($tt:tt)*)?) => {
        #[cfg(not(feature = "no-bool"))]
        $call!(bool, bool, $crate::partial_eq $(, $($tt)*)*);
        $call!(u8, u8, $crate::partial_eq $(, $($tt)*)*);
        $call!(u16, u16, $crate::partial_eq $(, $($tt)*)*);
        $call!(u32, u32, $crate::partial_eq $(, $($tt)*)*);
        $call!(u64, u64, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-u128"))]
        $call!(u128, u128, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-usize"))]
        $call!(usize, usize, $crate::partial_eq $(, $($tt)*)*);
        $call!(i8, i8, $crate::partial_eq $(, $($tt)*)*);
        $call!(i16, i16, $crate::partial_eq $(, $($tt)*)*);
        $call!(i32, i32, $crate::partial_eq $(, $($tt)*)*);
        $call!(i64, i64, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-i128"))]
        $call!(i128, i128, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-isize"))]
        $call!(isize, isize, $crate::partial_eq $(, $($tt)*)*);

        #[cfg(not(feature = "no-saturating"))]
        $call!(saturating_u8, core::num::Saturating<u8>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-saturating"))]
        $call!(saturating_u16, core::num::Saturating<u16>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-saturating"))]
        $call!(saturating_u32, core::num::Saturating<u32>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-saturating-u64"))]
        $call!(saturating_u64, core::num::Saturating<u64>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-saturating-u128"))]
        $call!(saturating_u128, core::num::Saturating<u128>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-saturating-usize"))]
        $call!(saturating_usize, core::num::Saturating<usize>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-saturating"))]
        $call!(saturating_i8, core::num::Saturating<i8>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-saturating"))]
        $call!(saturating_i16, core::num::Saturating<i16>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-saturating"))]
        $call!(saturating_i32, core::num::Saturating<i32>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-saturating"))]
        $call!(saturating_i64, core::num::Saturating<i64>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-saturating-i128"))]
        $call!(saturating_i128, core::num::Saturating<i128>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-saturating-isize"))]
        $call!(saturating_isize, core::num::Saturating<isize>, $crate::partial_eq $(, $($tt)*)*);

        #[cfg(not(feature = "no-wrapping"))]
        $call!(wrapping_u8, core::num::Wrapping<u8>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-wrapping"))]
        $call!(wrapping_u16, core::num::Wrapping<u16>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-wrapping"))]
        $call!(wrapping_u32, core::num::Wrapping<u32>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-wrapping-u64"))]
        $call!(wrapping_u64, core::num::Wrapping<u64>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-wrapping-u128"))]
        $call!(wrapping_u128, core::num::Wrapping<u128>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-wrapping-usize"))]
        $call!(wrapping_usize, core::num::Wrapping<usize>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-wrapping"))]
        $call!(wrapping_i8, core::num::Wrapping<i8>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-wrapping"))]
        $call!(wrapping_i16, core::num::Wrapping<i16>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-wrapping"))]
        $call!(wrapping_i32, core::num::Wrapping<i32>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-wrapping"))]
        $call!(wrapping_i64, core::num::Wrapping<i64>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-wrapping-i128"))]
        $call!(wrapping_i128, core::num::Wrapping<i128>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-wrapping-isize"))]
        $call!(wrapping_isize, core::num::Wrapping<isize>, $crate::partial_eq $(, $($tt)*)*);

        #[cfg(not(feature = "no-nonzero-u8"))]
        $call!(nonzero_u8, core::num::NonZeroU8, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-nonzero-u16"))]
        $call!(nonzero_u16, core::num::NonZeroU16, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-nonzero-u32"))]
        $call!(nonzero_u32, core::num::NonZeroU32, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-nonzero-u64"))]
        $call!(nonzero_u64, core::num::NonZeroU64, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-nonzero-u128"))]
        $call!(nonzero_u128, core::num::NonZeroU128, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-nonzero-usize"))]
        $call!(nonzero_usize, core::num::NonZeroUsize, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-nonzero-signed"))]
        $call!(nonzero_i8, core::num::NonZeroI8, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-nonzero-signed"))]
        $call!(nonzero_i16, core::num::NonZeroI16, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-nonzero-signed"))]
        $call!(nonzero_i32, core::num::NonZeroI32, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-nonzero-signed"))]
        $call!(nonzero_i64, core::num::NonZeroI64, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-nonzero-i128"))]
        $call!(nonzero_i128, core::num::NonZeroI128, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-nonzero-isize"))]
        $call!(nonzero_isize, core::num::NonZeroIsize, $crate::partial_eq $(, $($tt)*)*);

        #[cfg(all(target_has_atomic = "8", not(feature = "no-atomic-bool")))]
        $call!(atomic_bool, ::core::sync::atomic::AtomicBool, $crate::atomic_eq $(, $($tt)*)*);
        #[cfg(all(target_has_atomic = "8", not(feature = "no-atomic")))]
        $call!(atomic_u8, ::core::sync::atomic::AtomicU8, $crate::atomic_eq $(, $($tt)*)*);
        #[cfg(all(target_has_atomic = "8", not(feature = "no-atomic")))]
        $call!(atomic_i8, ::core::sync::atomic::AtomicI8, $crate::atomic_eq $(, $($tt)*)*);
        #[cfg(all(target_has_atomic = "16", not(feature = "no-atomic")))]
        $call!(atomic_u16, ::core::sync::atomic::AtomicU16, $crate::atomic_eq $(, $($tt)*)*);
        #[cfg(all(target_has_atomic = "16", not(feature = "no-atomic")))]
        $call!(atomic_i16, ::core::sync::atomic::AtomicI16, $crate::atomic_eq $(, $($tt)*)*);
        #[cfg(all(target_has_atomic = "32", not(feature = "no-atomic")))]
        $call!(atomic_u32, ::core::sync::atomic::AtomicU32, $crate::atomic_eq $(, $($tt)*)*);
        #[cfg(all(target_has_atomic = "32", not(feature = "no-atomic")))]
        $call!(atomic_i32, ::core::sync::atomic::AtomicI32, $crate::atomic_eq $(, $($tt)*)*);
        #[cfg(all(target_has_atomic = "64", not(feature = "no-atomic-u64")))]
        $call!(atomic_u64, ::core::sync::atomic::AtomicU64, $crate::atomic_eq $(, $($tt)*)*);
        #[cfg(all(target_has_atomic = "64", not(feature = "no-atomic")))]
        $call!(atomic_i64, ::core::sync::atomic::AtomicI64, $crate::atomic_eq $(, $($tt)*)*);
        #[cfg(all(target_has_atomic = "ptr", not(feature = "no-atomic-usize")))]
        $call!(atomic_usize, ::core::sync::atomic::AtomicUsize, $crate::atomic_eq $(, $($tt)*)*);
        #[cfg(all(target_has_atomic = "ptr", not(feature = "no-atomic-isize")))]
        $call!(atomic_isize, ::core::sync::atomic::AtomicIsize, $crate::atomic_eq $(, $($tt)*)*);
        $call!(f32, f32, $crate::partial_eq $(, $($tt)*)*);
        $call!(f64, f64, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-char"))]
        $call!(char, char, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-str"))]
        $call!(string, ::alloc::string::String, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-cstr"))]
        $call!(c_string, ::alloc::ffi::CString, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-osstr"))]
        $call!(os_string, ::std::ffi::OsString, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-path"))]
        $call!(path, ::std::path::PathBuf, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-alloc-vec"))]
        $call!(vec_u32, ::alloc::vec::Vec<u32>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(any(feature = "no-alloc-vec", feature = "no-char")))]
        $call!(vec_char, ::alloc::vec::Vec<char>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-alloc-map"))]
        $call!(hash_map_string_u32, ::std::collections::HashMap<String, u32>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-alloc-map"))]
        $call!(btree_map_string_u32, ::std::collections::BTreeMap<String, u32>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(any(feature = "no-alloc-map", feature = "no-number-key")))]
        $call!(hash_map_u32_u32, ::std::collections::HashMap<u32, u32>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(any(feature = "no-alloc-btree", feature = "no-alloc-map", feature = "no-number-key")))]
        $call!(btree_map_u32_u32, ::std::collections::BTreeMap<u32, u32>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-alloc-set"))]
        $call!(hash_set_string, ::std::collections::HashSet<String>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-alloc-set"))]
        $call!(hash_set_u32, ::std::collections::HashSet<u32>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-alloc-btree"))]
        $call!(btree_set_u32, ::std::collections::BTreeSet<u32>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-alloc-binaryheap"))]
        $call!(btree_binaryheap_u32, ::std::collections::BinaryHeap<u32>, $crate::binary_heap_eq $(, $($tt)*)*);
        #[cfg(not(feature = "no-alloc-vecdeque"))]
        $call!(btree_vecdeque_u32, ::std::collections::VecDeque<u32>, $crate::partial_eq $(, $($tt)*)*);

        #[cfg(not(any(feature = "no-unsized-box", feature = "no-unsized-str")))]
        $call!(box_str, ::alloc::boxed::Box<str>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(any(feature = "no-unsized-box", feature = "no-unsized-path")))]
        $call!(box_path, ::alloc::boxed::Box<::std::path::Path>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(any(feature = "no-unsized-box", feature = "no-unsized-osstr")))]
        $call!(box_osstr, ::alloc::boxed::Box<::std::ffi::OsStr>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(any(feature = "no-unsized-box", feature = "no-unsized-cstr")))]
        $call!(box_cstr, ::alloc::boxed::Box<::std::ffi::CStr>, $crate::partial_eq $(, $($tt)*)*);

        #[cfg(not(any(feature = "no-unsized-rc", feature = "no-unsized-str")))]
        $call!(rc_str, ::alloc::rc::Rc<str>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(any(feature = "no-unsized-rc", feature = "no-unsized-path")))]
        $call!(rc_path, ::alloc::rc::Rc<::std::path::Path>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(any(feature = "no-unsized-rc", feature = "no-unsized-osstr")))]
        $call!(rc_osstr, ::alloc::rc::Rc<::std::ffi::OsStr>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(any(feature = "no-unsized-rc", feature = "no-unsized-cstr")))]
        $call!(rc_cstr, ::alloc::rc::Rc<::std::ffi::CStr>, $crate::partial_eq $(, $($tt)*)*);

        #[cfg(not(any(feature = "no-unsized-rc", feature = "no-unsized-str")))]
        $call!(arc_str, ::alloc::sync::Arc<str>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(any(feature = "no-unsized-rc", feature = "no-unsized-path")))]
        $call!(arc_path, ::alloc::sync::Arc<::std::path::Path>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(any(feature = "no-unsized-rc", feature = "no-unsized-osstr")))]
        $call!(arc_osstr, ::alloc::sync::Arc<::std::ffi::OsStr>, $crate::partial_eq $(, $($tt)*)*);
        #[cfg(not(any(feature = "no-unsized-rc", feature = "no-unsized-cstr")))]
        $call!(arc_cstr, ::alloc::sync::Arc<::std::ffi::CStr>, $crate::partial_eq $(, $($tt)*)*);
    };
}

/// Build common RNG with custom seed.
pub fn rng_with_seed(seed: u64) -> generate::Rng {
    generate::Rng::from_seed(seed)
}

/// Build common RNG.
pub fn rng() -> generate::Rng {
    rng_with_seed(RNG_SEED)
}
