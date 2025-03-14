#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

/// Default random seed to use.
pub const RNG_SEED: u64 = 2718281828459045235;

pub use tests_macros::benchmarker;

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
            $({
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
        $enumerate_vis fn $enumerate(out: fn(&str, &dyn core::fmt::Debug)) {
            $({
                let key = concat!("MUSLI_", stringify!($ident));
                let value = $ident.get();
                out(key, &value);
            })*
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
        let (from, to) = input.split_once("..")?;
        Some(from.parse().ok()?..to.parse().ok()?)
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
    self::models::init_ranges();
    self::generate::init_ranges();
}

/// Enumerate all available statics.
pub fn enumerate_statics(out: fn(&str, &dyn core::fmt::Debug)) {
    self::models::enumerate_ranges(out);
    self::generate::enumerate_ranges(out);
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

/// Build common RNG with custom seed.
pub fn rng_with_seed(seed: u64) -> generate::Rng {
    generate::Rng::from_seed(seed)
}

/// Build common RNG.
pub fn rng() -> generate::Rng {
    rng_with_seed(RNG_SEED)
}
