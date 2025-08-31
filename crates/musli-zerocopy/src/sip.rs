// Copied and trimmed from the siphasher project under the MIT license.
//
// See https://github.com/jedisct1/rust-siphash/tree/e085f9387d14b40d92e2478f11dcb772149147be

use core::cmp;
use core::hash;
use core::marker::PhantomData;
use core::mem;
use core::ptr;

/// A 128-bit (2x64) hash output
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Hash128 {
    pub(crate) h1: u64,
    pub(crate) h2: u64,
}

/// An implementation of SipHash128 1-3.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SipHasher13 {
    hasher: Hasher<Sip13Rounds>,
}

#[derive(Debug, Copy)]
struct Hasher<S>
where
    S: Sip,
{
    k0: u64,
    k1: u64,
    // how many bytes we've processed
    length: usize,
    // hash State
    state: State,
    // unprocessed bytes le
    tail: u64,
    // how many bytes in tail are valid
    ntail: usize,
    _marker: PhantomData<S>,
}

#[derive(Debug, Clone, Copy)]
struct State {
    // v0, v2 and v1, v3 show up in pairs in the algorithm, and simd
    // implementations of SipHash will use vectors of v02 and v13. By placing
    // them in this order in the struct, the compiler can pick up on just a few
    // simd optimizations by itself.
    v0: u64,
    v2: u64,
    v1: u64,
    v3: u64,
}

macro_rules! compress {
    ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }};
    ($v0:expr, $v1:expr, $v2:expr, $v3:expr) => {{
        $v0 = $v0.wrapping_add($v1);
        $v1 = $v1.rotate_left(13);
        $v1 ^= $v0;
        $v0 = $v0.rotate_left(32);
        $v2 = $v2.wrapping_add($v3);
        $v3 = $v3.rotate_left(16);
        $v3 ^= $v2;
        $v0 = $v0.wrapping_add($v3);
        $v3 = $v3.rotate_left(21);
        $v3 ^= $v0;
        $v2 = $v2.wrapping_add($v1);
        $v1 = $v1.rotate_left(17);
        $v1 ^= $v2;
        $v2 = $v2.rotate_left(32);
    }};
}

/// Loads an integer of the desired type from a byte stream, in LE order. Uses
/// `copy_nonoverlapping` to let the compiler generate the most efficient way
/// to load it from a possibly unaligned address.
///
/// Unsafe because: unchecked indexing at `i..i+size_of(int_ty)`
macro_rules! load_int_le {
    ($buf:expr, $i:expr, $int_ty:ident) => {{
        debug_assert!($i + mem::size_of::<$int_ty>() <= $buf.len());
        let mut data = 0 as $int_ty;
        ptr::copy_nonoverlapping(
            $buf.as_ptr().add($i),
            &mut data as *mut _ as *mut u8,
            mem::size_of::<$int_ty>(),
        );
        data.to_le()
    }};
}

/// Loads a u64 using up to 7 bytes of a byte slice. It looks clumsy but the
/// `copy_nonoverlapping` calls that occur (via `load_int_le!`) all have fixed
/// sizes and avoid calling `memcpy`, which is good for speed.
///
/// Unsafe because: unchecked indexing at start..start+len
#[inline]
unsafe fn u8to64_le(buf: &[u8], start: usize, len: usize) -> u64 {
    unsafe {
        debug_assert!(len < 8);
        let mut i = 0; // current byte index (from LSB) in the output u64
        let mut out = 0;
        if i + 3 < len {
            out = load_int_le!(buf, start + i, u32) as u64;
            i += 4;
        }
        if i + 1 < len {
            out |= (load_int_le!(buf, start + i, u16) as u64) << (i * 8);
            i += 2
        }
        if i < len {
            out |= (*buf.get_unchecked(start + i) as u64) << (i * 8);
            i += 1;
        }
        debug_assert_eq!(i, len);
        out
    }
}

pub(crate) trait Hasher128 {
    /// Return a 128-bit hash
    fn finish128(&self) -> Hash128;
}

impl SipHasher13 {
    /// Creates a `SipHasher13` that is keyed off the provided keys.
    #[inline]
    pub(crate) fn new_with_keys(key0: u64, key1: u64) -> SipHasher13 {
        SipHasher13 {
            hasher: Hasher::new_with_keys(key0, key1),
        }
    }
}

impl Hasher128 for SipHasher13 {
    /// Return a 128-bit hash
    #[inline]
    fn finish128(&self) -> Hash128 {
        self.hasher.finish128()
    }
}

impl<S> Hasher<S>
where
    S: Sip,
{
    #[inline]
    fn new_with_keys(key0: u64, key1: u64) -> Hasher<S> {
        let mut state = Hasher {
            k0: key0,
            k1: key1,
            length: 0,
            state: State {
                v0: 0,
                v1: 0xee,
                v2: 0,
                v3: 0,
            },
            tail: 0,
            ntail: 0,
            _marker: PhantomData,
        };
        state.reset();
        state
    }

    #[inline]
    fn reset(&mut self) {
        self.length = 0;
        self.state.v0 = self.k0 ^ 0x736f6d6570736575;
        self.state.v1 = self.k1 ^ 0x646f72616e646f83;
        self.state.v2 = self.k0 ^ 0x6c7967656e657261;
        self.state.v3 = self.k1 ^ 0x7465646279746573;
        self.ntail = 0;
    }

    // A specialized write function for values with size <= 8.
    //
    // The hashing of multi-byte integers depends on endianness. E.g.:
    // - little-endian: `write_u32(0xDDCCBBAA)` == `write([0xAA, 0xBB, 0xCC, 0xDD])`
    // - big-endian:    `write_u32(0xDDCCBBAA)` == `write([0xDD, 0xCC, 0xBB, 0xAA])`
    //
    // This function does the right thing for little-endian hardware. On
    // big-endian hardware `x` must be byte-swapped first to give the right
    // behaviour. After any byte-swapping, the input must be zero-extended to
    // 64-bits. The caller is responsible for the byte-swapping and
    // zero-extension.
    #[inline]
    fn short_write<T>(&mut self, _x: T, x: u64) {
        let size = mem::size_of::<T>();
        self.length += size;

        // The original number must be zero-extended, not sign-extended.
        debug_assert!(if size < 8 { x >> (8 * size) == 0 } else { true });

        // The number of bytes needed to fill `self.tail`.
        let needed = 8 - self.ntail;

        self.tail |= x << (8 * self.ntail);
        if size < needed {
            self.ntail += size;
            return;
        }

        // `self.tail` is full, process it.
        self.state.v3 ^= self.tail;
        S::c_rounds(&mut self.state);
        self.state.v0 ^= self.tail;

        self.ntail = size - needed;
        self.tail = if needed < 8 { x >> (8 * needed) } else { 0 };
    }
}

impl<S> Hasher<S>
where
    S: Sip,
{
    #[inline]
    pub(crate) fn finish128(&self) -> Hash128 {
        let mut state = self.state;

        let b: u64 = ((self.length as u64 & 0xff) << 56) | self.tail;

        state.v3 ^= b;
        S::c_rounds(&mut state);
        state.v0 ^= b;

        state.v2 ^= 0xee;
        S::d_rounds(&mut state);
        let h1 = state.v0 ^ state.v1 ^ state.v2 ^ state.v3;

        state.v1 ^= 0xdd;
        S::d_rounds(&mut state);
        let h2 = state.v0 ^ state.v1 ^ state.v2 ^ state.v3;

        Hash128 { h1, h2 }
    }
}

impl hash::Hasher for SipHasher13 {
    #[inline]
    fn write(&mut self, msg: &[u8]) {
        self.hasher.write(msg)
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hasher.finish()
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.hasher.write_usize(i);
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.hasher.write_u8(i);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.hasher.write_u16(i);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.hasher.write_u32(i);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.hasher.write_u64(i);
    }
}

impl<S> hash::Hasher for Hasher<S>
where
    S: Sip,
{
    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.short_write(i, i.to_le() as u64);
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.short_write(i, i as u64);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.short_write(i, i.to_le() as u64);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.short_write(i, i.to_le());
    }

    #[inline]
    fn write(&mut self, msg: &[u8]) {
        let length = msg.len();
        self.length += length;

        let mut needed = 0;

        if self.ntail != 0 {
            needed = 8 - self.ntail;
            self.tail |= unsafe { u8to64_le(msg, 0, cmp::min(length, needed)) } << (8 * self.ntail);
            if length < needed {
                self.ntail += length;
                return;
            } else {
                self.state.v3 ^= self.tail;
                S::c_rounds(&mut self.state);
                self.state.v0 ^= self.tail;
                self.ntail = 0;
            }
        }

        // Buffered tail is now flushed, process new input.
        let len = length - needed;
        let left = len & 0x7;

        let mut i = needed;
        while i < len - left {
            let mi = unsafe { load_int_le!(msg, i, u64) };

            self.state.v3 ^= mi;
            S::c_rounds(&mut self.state);
            self.state.v0 ^= mi;

            i += 8;
        }

        self.tail = unsafe { u8to64_le(msg, i, left) };
        self.ntail = left;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.finish128().h2
    }
}

impl<S> Clone for Hasher<S>
where
    S: Sip,
{
    #[inline]
    fn clone(&self) -> Hasher<S> {
        Hasher {
            k0: self.k0,
            k1: self.k1,
            length: self.length,
            state: self.state,
            tail: self.tail,
            ntail: self.ntail,
            _marker: self._marker,
        }
    }
}

trait Sip {
    fn c_rounds(_: &mut State);

    fn d_rounds(_: &mut State);
}

#[derive(Debug, Clone, Copy, Default)]
struct Sip13Rounds;

impl Sip for Sip13Rounds {
    #[inline]
    fn c_rounds(state: &mut State) {
        compress!(state);
    }

    #[inline]
    fn d_rounds(state: &mut State) {
        compress!(state);
        compress!(state);
        compress!(state);
    }
}
