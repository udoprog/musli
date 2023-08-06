use core::hash::Hash;

use crate::sip::{Hash128, Hasher128, SipHasher13};

use crate::error::{Error, ErrorKind};
use crate::slice::Slice;

#[non_exhaustive]
pub(crate) struct Hashes {
    pub(crate) g: usize,
    pub(crate) f1: u32,
    pub(crate) f2: u32,
}

pub(crate) type HashKey = u64;

#[inline]
pub(crate) fn displace(f1: u32, f2: u32, d1: u32, d2: u32) -> u32 {
    d2.wrapping_add(f1.wrapping_mul(d1)).wrapping_add(f2)
}

#[inline]
pub(crate) fn hash<T>(value: &T, key: &HashKey) -> Hashes
where
    T: ?Sized + Hash,
{
    let mut hasher = SipHasher13::new_with_keys(0, *key);
    value.hash(&mut hasher);

    let Hash128 { h1, h2 } = hasher.finish128();

    Hashes {
        g: (h1 >> 32) as usize,
        f1: h1 as u32,
        f2: h2 as u32,
    }
}

#[inline]
pub(crate) fn get_index(
    &Hashes { g, f1, f2 }: &Hashes,
    displacements: &Slice<(u32, u32)>,
    len: usize,
) -> Result<usize, Error> {
    let index = g % displacements.len();

    let Some((d1, d2)) = displacements.get(index)? else {
        return Err(Error::new(ErrorKind::IndexOutOfBounds {
            index,
            len: displacements.len(),
        }));
    };

    Ok(displace(f1, f2, d1, d2) as usize % len)
}
