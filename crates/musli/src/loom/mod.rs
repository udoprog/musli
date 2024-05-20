#![cfg(feature = "alloc")]

pub(crate) mod cell;
pub(crate) mod sync;

#[cfg(loom)]
pub(crate) use loom::hint::spin_loop;

#[cfg(not(loom))]
pub(crate) use core::hint::spin_loop;
