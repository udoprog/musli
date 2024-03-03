#![allow(clippy::manual_map)]

#[macro_use]
mod macros;

cfg_if! {
    // Use the SSE2 implementation if possible: it allows us to scan 16 buckets
    // at once instead of 8. We don't bother with AVX since it would require
    // runtime dispatch and wouldn't gain us much anyways: the probability of
    // finding a match drops off drastically after the first few buckets.
    //
    // I attempted an implementation on ARM using NEON instructions, but it
    // turns out that most NEON instructions have multi-cycle latency, which in
    // the end outweighs any gains over the generic implementation.
    if #[cfg(all(
        target_feature = "sse2",
        any(target_arch = "x86", target_arch = "x86_64"),
        not(miri)
    ))] {
        mod sse2;
        use sse2 as imp;
    } else if #[cfg(all(target_arch = "aarch64", target_feature = "neon"))] {
        mod neon;
        use neon as imp;
    } else {
        mod generic;
        use generic as imp;
    }
}

mod bitmask;

use crate::error::Error;
use crate::error::ErrorKind;

pub(crate) use self::imp::Group;

use core::mem;

/// Probe sequence based on triangular numbers, which is guaranteed (since our
/// table size is a power of two) to visit every group of elements exactly once.
///
/// A triangular probe has us jump by 1 more group every time. So first we
/// jump by 1 group (meaning we just continue our linear scan), then 2 groups
/// (skipping over 1 group), then 3 groups (skipping over 2 groups), and so on.
///
/// Proof that the probe will visit every group in the table:
/// <https://fgiesen.wordpress.com/2015/02/22/triangular-numbers-mod-2n/>
#[derive(Debug)]
pub(crate) struct ProbeSeq {
    pub(crate) pos: usize,
    stride: usize,
}

impl ProbeSeq {
    #[inline]
    pub(crate) fn move_next(&mut self, bucket_mask: usize) -> Result<(), Error> {
        if self.stride > bucket_mask {
            return Err(Error::new(ErrorKind::StrideOutOfBounds {
                index: self.stride,
                len: bucket_mask,
            }));
        }

        self.stride += Group::WIDTH;
        self.pos += self.stride;
        self.pos &= bucket_mask;
        Ok(())
    }
}

// Constant for h2 function that grabing the top 7 bits of the hash.
const MIN_HASH_LEN: usize = if mem::size_of::<usize>() < mem::size_of::<u64>() {
    mem::size_of::<usize>()
} else {
    mem::size_of::<u64>()
};

/// Returns an iterator-like object for a probe sequence on the table.
///
/// This iterator never terminates, but is guaranteed to visit each bucket
/// group exactly once. The loop using `probe_seq` must terminate upon
/// reaching a group containing an empty bucket.
#[inline]
pub(crate) fn probe_seq(bucket_mask: usize, hash: u64) -> ProbeSeq {
    ProbeSeq {
        // This is the same as `hash as usize % self.buckets()` because the number
        // of buckets is a power of two, and `self.bucket_mask = self.buckets() - 1`.
        pos: h1(hash) & bucket_mask,
        stride: 0,
    }
}

/// Primary hash function, used to select the initial bucket to probe from.
#[inline]
#[allow(clippy::cast_possible_truncation)]
fn h1(hash: u64) -> usize {
    // On 32-bit platforms we simply ignore the higher hash bits.
    hash as usize
}

/// Secondary hash function, saved in the low 7 bits of the control byte.
#[inline]
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn h2(hash: u64) -> u8 {
    // Grab the top 7 bits of the hash. While the hash is normally a full 64-bit
    // value, some hash functions (such as FxHash) produce a usize result
    // instead, which means that the top 32 bits are 0 on 32-bit platforms.
    // So we use MIN_HASH_LEN constant to handle this.
    let top7 = hash >> (MIN_HASH_LEN * 8 - 7);
    (top7 & 0x7f) as u8 // truncation
}

/// Control byte value for an empty bucket.
pub(crate) const EMPTY: u8 = 0b1111_1111;

/// Checks whether a control byte represents a full bucket (top bit is clear).
#[inline]
#[cfg(feature = "alloc")]
pub(crate) fn is_full(ctrl: u8) -> bool {
    ctrl & 0x80 == 0
}

/// Checks whether a control byte represents a special value (top bit is set).
#[inline]
#[cfg(feature = "alloc")]
pub(crate) fn is_special(ctrl: u8) -> bool {
    ctrl & 0x80 != 0
}

/// Checks whether a special control value is EMPTY (just check 1 bit).
#[inline]
#[cfg(feature = "alloc")]
pub(crate) fn special_is_empty(ctrl: u8) -> bool {
    debug_assert!(is_special(ctrl));
    ctrl & 0x01 != 0
}

/// Returns the number of buckets needed to hold the given number of items,
/// taking the maximum load factor into account.
///
/// Returns `None` if an overflow occurs.
// Workaround for emscripten bug emscripten-core/emscripten-fastcomp#258
#[cfg_attr(target_os = "emscripten", inline(never))]
#[cfg_attr(not(target_os = "emscripten"), inline)]
#[cfg(feature = "alloc")]
pub(crate) fn capacity_to_buckets(cap: usize) -> Option<usize> {
    // For small tables we require at least 1 empty bucket so that lookups are
    // guaranteed to terminate if an element doesn't exist in the table.
    if cap < 8 {
        // We don't bother with a table size of 2 buckets since that can only
        // hold a single element. Instead we skip directly to a 4 bucket table
        // which can hold 3 elements.
        return Some(if cap < 4 { 4 } else { 8 });
    }

    // Otherwise require 1/8 buckets to be empty (87.5% load)
    //
    // Be careful when modifying this, calculate_layout relies on the
    // overflow check here.
    let adjusted_cap = cap.checked_mul(8)? / 7;

    // Any overflows will have been caught by the checked_mul. Also, any
    // rounding errors from the division above will be cleaned up by
    // next_power_of_two (which can't overflow because of the previous division).
    Some(adjusted_cap.next_power_of_two())
}
