//! A serialized prefix-trie.

#[cfg(test)]
mod tests;

#[cfg(feature = "alloc")]
pub use self::factory::{Builder, store};
#[cfg(feature = "alloc")]
mod factory;

use self::walk::Walk;
mod walk;

use core::fmt;
use core::marker::PhantomData;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::endian::Native;
use crate::lossy_str::LossyStr;
use crate::slice::{BinarySearch, Slice, binary_search_by};
use crate::stack::ArrayStack;
use crate::{Buf, ByteOrder, DefaultSize, Error, Ref, Size, ZeroCopy};

type StackEntry<'buf, T, F> = (LinksRef<T, F>, usize, &'buf [u8]);

/// The flavor of a trie. Allows for customization of implementation details to
/// for example use a more compact representation than the one provided by
/// default in case there is a known upper bound on the number of elements or
/// values.
///
/// # Examples
///
/// ```
/// use musli_zerocopy::{trie, Error, OwnedBuf, ZeroCopy};
/// use musli_zerocopy::slice::Packed;
///
/// struct PackedTrie;
///
/// impl trie::Flavor for PackedTrie {
///     // The maximum length of a string slice stored in the trie is `u8::MAX`.
///     type String = Packed<[u8], u32, u8>;
///
///     // The max number of values stored in a single node is `u16::MAX`.
///     type Values<T> = Packed<[T], u32, u16>
///     where
///         T: ZeroCopy;
///
///     // The maximum number of children for a single node is `u8::MAX`.
///     type Children<T> = Packed<[T], u32, u8>
///     where
///         T: ZeroCopy;
/// }
///
/// fn populate<F>(buf: &mut OwnedBuf, mut trie: trie::Builder<u32, F>) -> Result<trie::TrieRef<u32, F>, Error>
/// where
///     F: trie::Flavor
/// {
///     let a = buf.store_unsized("hello");
///     let b = buf.store_unsized("hello world");
///     trie.insert(buf, a, 1)?;
///     trie.insert(buf, b, 2)?;
///     trie.build(buf)
/// }
///
/// let mut b1 = OwnedBuf::new();
///
/// let mut trie = trie::Builder::<u32, PackedTrie>::with_flavor();
/// let trie = populate(&mut b1, trie)?;
///
/// assert_eq!(trie.get(&b1, "hello")?, Some(&[1][..]));
/// assert_eq!(trie.get(&b1, "hello world")?, Some(&[2][..]));
///
/// let mut b2 = OwnedBuf::new();
///
/// let mut trie = trie::Builder::new();
/// let trie = populate(&mut b2, trie)?;
///
/// assert_eq!(trie.get(&b2, "hello")?, Some(&[1][..]));
/// assert_eq!(trie.get(&b2, "hello world")?, Some(&[2][..]));
///
/// assert!(b1.len() < b2.len());
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub trait Flavor {
    /// The type representing a string in the trie.
    type String: Slice<Item = u8>;

    /// The type representing a collection of values in the trie.
    type Values<T>: Slice<Item = T>
    where
        T: ZeroCopy;

    /// The type representing a collection of children in the trie.
    type Children<T>: Slice<Item = T>
    where
        T: ZeroCopy;
}

/// Marker type indicating the default trie [`Flavor`] to use for a given
/// [`ByteOrder`] and [`Size`].
pub struct DefaultFlavor<E = Native, O = DefaultSize>(PhantomData<(E, O)>)
where
    E: ByteOrder,
    O: Size;

impl<E, O> Flavor for DefaultFlavor<E, O>
where
    E: ByteOrder,
    O: Size,
{
    type String = Ref<[u8], E, O>;
    type Values<T>
        = Ref<[T], E, O>
    where
        T: ZeroCopy;
    type Children<T>
        = Ref<[T], E, O>
    where
        T: ZeroCopy;
}

/// A stored reference to a trie.
#[derive(ZeroCopy)]
#[zero_copy(crate)]
#[repr(C)]
pub struct TrieRef<T, F = DefaultFlavor>
where
    T: ZeroCopy,
    F: Flavor,
{
    links: LinksRef<T, F>,
}

impl<T, F> TrieRef<T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    /// Debug print the current trie.
    ///
    /// This treats the keys as strings, with illegal unicode sequences being
    /// replaced with the `U+FFFD` escape sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{trie, OwnedBuf};
    ///
    /// let mut buf = OwnedBuf::new();
    /// let a = buf.store(b"\xe2\x28\xa1").array_into_slice();
    /// let b = buf.store_unsized("食べない");
    ///
    /// let mut trie = trie::Builder::new();
    ///
    /// trie.insert(&buf, b, 2)?;
    /// trie.insert(&buf, a, 1)?;
    ///
    /// let trie = trie.build(&mut buf)?;
    /// assert_eq!(format!("{:?}", trie.debug(&buf)), "{\"�(�\": 1, \"食べない\": 2}");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    pub fn debug<'a, 'buf>(&'a self, buf: &'buf Buf) -> Debug<'a, 'buf, T, F>
    where
        T: fmt::Debug,
    {
        Debug { trie: self, buf }
    }

    /// Debug print the current trie, with a fixed iteration depth of `N`.
    ///
    /// This treats the keys as strings, with illegal unicode sequences being
    /// replaced with the `U+FFFD` escape sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{trie, OwnedBuf};
    ///
    /// let mut buf = OwnedBuf::new();
    /// let a = buf.store(b"\xe2\x28\xa1").array_into_slice();
    /// let b = buf.store_unsized("食べない");
    ///
    /// let mut trie = trie::Builder::new();
    ///
    /// trie.insert(&buf, b, 2)?;
    /// trie.insert(&buf, a, 1)?;
    ///
    /// let trie = trie.build(&mut buf)?;
    /// assert_eq!(format!("{:?}", trie.debug_fixed::<16>(&buf)), "{\"�(�\": 1, \"食べない\": 2}");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn debug_fixed<'a, 'buf, const N: usize>(
        &'a self,
        buf: &'buf Buf,
    ) -> DebugFixed<'a, 'buf, N, T, F>
    where
        T: fmt::Debug,
    {
        DebugFixed { trie: self, buf }
    }

    /// Get all values associated with the given string.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{trie, OwnedBuf};
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let values = [
    ///     (buf.store_unsized("work"), 1),
    ///     (buf.store_unsized("worker"), 2),
    ///     (buf.store_unsized("workers"), 3),
    ///     (buf.store_unsized("working"), 4),
    ///     (buf.store_unsized("working"), 5),
    ///     (buf.store_unsized("working man"), 6),
    /// ];
    ///
    /// let trie = trie::store(&mut buf, values)?;
    ///
    /// assert_eq!(trie.get(&buf, "aard")?, None);
    /// assert_eq!(trie.get(&buf, "worker")?, Some(&[2][..]));
    /// assert_eq!(trie.get(&buf, "working")?, Some(&[4, 5][..]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn get<'buf, S>(&self, buf: &'buf Buf, string: &S) -> Result<Option<&'buf [T]>, Error>
    where
        S: ?Sized + AsRef<[u8]>,
    {
        let mut this = self.links;
        let mut string = string.as_ref();

        loop {
            let search =
                binary_search_by(buf, this.children, |c| Ok(buf.load(c.string)?.cmp(string)))?;

            match search {
                BinarySearch::Found(n) => {
                    let child = this.children.get_unchecked(n);
                    let child = buf.load(child)?;
                    let values = buf.load(child.links.values)?;
                    return Ok(Some(values));
                }
                BinarySearch::Missing(0) => {
                    return Ok(None);
                }
                BinarySearch::Missing(n) => {
                    let child = this.children.get_unchecked(n - 1);
                    let child = buf.load(child)?;

                    // Find common prefix and split nodes if necessary.
                    let prefix = prefix(buf.load(child.string)?, string);

                    if prefix == 0 {
                        return Ok(None);
                    }

                    string = &string[prefix..];
                    this = child.links;
                }
            };
        }
    }

    /// Construct an iterator over all values in the trie.
    ///
    /// Note that the iteration order is unspecified and might change in future
    /// versions.
    ///
    /// # Errors
    ///
    /// This errors in case the trie being iterated over is structurally
    /// invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{trie, OwnedBuf};
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let values = [
    ///     (buf.store_unsized("work"), 1),
    ///     (buf.store_unsized("worker"), 2),
    ///     (buf.store_unsized("workers"), 3),
    ///     (buf.store_unsized("working"), 4),
    ///     (buf.store_unsized("working"), 5),
    ///     (buf.store_unsized("working man"), 6),
    ///     (buf.store_unsized("run"), 7),
    ///     (buf.store_unsized("running"), 8),
    /// ];
    ///
    /// let trie = trie::store(&mut buf, values)?;
    ///
    /// let mut values = trie.values(&buf).collect::<Result<Vec<_>, _>>()?;
    /// values.sort();
    /// assert!(values.into_iter().copied().eq([1, 2, 3, 4, 5, 6, 7, 8]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    pub fn values<'buf>(&self, buf: &'buf Buf) -> Values<'buf, T, F> {
        Values {
            iter: Walk::find(buf, self.links, &[]),
        }
    }

    /// Construct an iterator over all values in the trie using a fixed max
    /// iteration depth of `N`.
    ///
    /// Note that the iteration order is unspecified and might change in future
    /// versions.
    ///
    /// # Errors
    ///
    /// This errors in case the trie being iterated over is structurally invalid
    /// or if the iteration depth exceeds `N`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{trie, OwnedBuf};
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let values = [
    ///     (buf.store_unsized("work"), 1),
    ///     (buf.store_unsized("worker"), 2),
    ///     (buf.store_unsized("workers"), 3),
    ///     (buf.store_unsized("working"), 4),
    ///     (buf.store_unsized("working"), 5),
    ///     (buf.store_unsized("working man"), 6),
    ///     (buf.store_unsized("run"), 7),
    ///     (buf.store_unsized("running"), 8),
    /// ];
    ///
    /// let trie = trie::store(&mut buf, values)?;
    ///
    /// let mut values = trie.values_fixed::<16>(&buf).collect::<Result<Vec<_>, _>>()?;
    /// values.sort();
    /// assert!(values.into_iter().copied().eq([1, 2, 3, 4, 5, 6, 7, 8]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn values_fixed<'buf, const N: usize>(&self, buf: &'buf Buf) -> ValuesFixed<'buf, N, T, F> {
        ValuesFixed {
            iter: Walk::find(buf, self.links, &[]),
        }
    }

    /// Construct an iterator over values that are inside of the specified
    /// `prefix` in the trie.
    ///
    /// Note that the iteration order is unspecified and might change in future
    /// versions.
    ///
    /// # Errors
    ///
    /// This errors in case the trie being iterated over is structurally
    /// invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{trie, OwnedBuf};
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let values = [
    ///     (buf.store_unsized("work"), 1),
    ///     (buf.store_unsized("worker"), 2),
    ///     (buf.store_unsized("workers"), 3),
    ///     (buf.store_unsized("working"), 4),
    ///     (buf.store_unsized("working"), 5),
    ///     (buf.store_unsized("working man"), 6),
    ///     (buf.store_unsized("run"), 7),
    ///     (buf.store_unsized("running"), 8),
    /// ];
    ///
    /// let trie = trie::store(&mut buf, values)?;
    ///
    /// let values = trie.values_in(&buf, "workin").collect::<Result<Vec<_>, _>>()?;
    /// assert!(values.into_iter().copied().eq([4, 5, 6]));
    ///
    /// let values = trie.values_in(&buf, "wor").collect::<Result<Vec<_>, _>>()?;
    /// assert!(values.into_iter().copied().eq([1, 2, 3, 4, 5, 6]));
    ///
    /// let values = trie.values_in(&buf, "runn").collect::<Result<Vec<_>, _>>()?;
    /// assert!(values.into_iter().copied().eq([8]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    pub fn values_in<'a, 'buf, S>(&self, buf: &'buf Buf, prefix: &'a S) -> ValuesIn<'a, 'buf, T, F>
    where
        S: ?Sized + AsRef<[u8]>,
    {
        ValuesIn {
            iter: Walk::find(buf, self.links, prefix.as_ref()),
        }
    }

    /// Construct an iterator over values that are inside of the specified
    /// `prefix` in the trie using a fixed max iteration depth of `N`.
    ///
    /// Note that the iteration order is unspecified and might change in future
    /// versions.
    ///
    /// # Errors
    ///
    /// This errors in case the trie being iterated over is structurally
    /// invalid or if the iteration depth exceeds `N`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{trie, OwnedBuf};
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let values = [
    ///     (buf.store_unsized("work"), 1),
    ///     (buf.store_unsized("worker"), 2),
    ///     (buf.store_unsized("workers"), 3),
    ///     (buf.store_unsized("working"), 4),
    ///     (buf.store_unsized("working"), 5),
    ///     (buf.store_unsized("working man"), 6),
    ///     (buf.store_unsized("run"), 7),
    ///     (buf.store_unsized("running"), 8),
    /// ];
    ///
    /// let trie = trie::store(&mut buf, values)?;
    ///
    /// let values = trie.values_in_fixed::<16, _>(&buf, "workin").collect::<Result<Vec<_>, _>>()?;
    /// assert!(values.into_iter().copied().eq([4, 5, 6]));
    ///
    /// let values = trie.values_in_fixed::<16, _>(&buf, "wor").collect::<Result<Vec<_>, _>>()?;
    /// assert!(values.into_iter().copied().eq([1, 2, 3, 4, 5, 6]));
    ///
    /// let values = trie.values_in_fixed::<16, _>(&buf, "runn").collect::<Result<Vec<_>, _>>()?;
    /// assert!(values.into_iter().copied().eq([8]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn values_in_fixed<'a, 'buf, const N: usize, S>(
        &self,
        buf: &'buf Buf,
        prefix: &'a S,
    ) -> ValuesInFixed<'a, 'buf, N, T, F>
    where
        S: ?Sized + AsRef<[u8]>,
    {
        ValuesInFixed {
            iter: Walk::find(buf, self.links, prefix.as_ref()),
        }
    }

    /// Construct an iterator over all entries in the trie.
    ///
    /// Note that the iteration order is unspecified and might change in future
    /// versions.
    ///
    /// # Errors
    ///
    /// This errors in case the trie being iterated over is structurally
    /// invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::from_utf8;
    ///
    /// use anyhow::Result;
    /// use musli_zerocopy::{trie, OwnedBuf};
    ///
    /// // Helper to convert output to utf-8.
    /// fn to_utf8<'buf, E>(result: Result<(&'buf [u8], &'buf i32), E>) -> Result<(&'buf str, i32)>
    /// where
    ///     anyhow::Error: From<E>
    /// {
    ///     let (k, v) = result?;
    ///     Ok((from_utf8(k)?, *v))
    /// }
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let values = [
    ///     (buf.store_unsized("work"), 1),
    ///     (buf.store_unsized("worker"), 2),
    ///     (buf.store_unsized("workers"), 3),
    ///     (buf.store_unsized("working"), 4),
    ///     (buf.store_unsized("working"), 5),
    ///     (buf.store_unsized("working man"), 6),
    ///     (buf.store_unsized("run"), 7),
    ///     (buf.store_unsized("running"), 8),
    /// ];
    ///
    /// let trie = trie::store(&mut buf, values)?;
    ///
    /// let mut values = trie.iter(&buf)
    ///     .map(to_utf8)
    ///     .collect::<Result<Vec<_>>>()?;
    /// values.sort();
    ///
    /// assert_eq! {
    ///     values,
    ///     [
    ///         ("run", 7),
    ///         ("running", 8),
    ///         ("work", 1),
    ///         ("worker", 2),
    ///         ("workers", 3),
    ///         ("working", 4),
    ///         ("working", 5),
    ///         ("working man", 6),
    ///     ]
    /// };
    /// # Ok::<_, anyhow::Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    pub fn iter<'buf>(&self, buf: &'buf Buf) -> Iter<'buf, T, F> {
        Iter {
            iter: Walk::find(buf, self.links, &[]),
        }
    }

    /// Construct an iterator over all entries in the trie using a fixed max
    /// iteration depth of `N`
    ///
    /// Note that the iteration order is unspecified and might change in future
    /// versions.
    ///
    /// # Errors
    ///
    /// This errors in case the trie being iterated over is structurally
    /// invalid or if the iteration depth exceeds `N`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::from_utf8;
    ///
    /// use anyhow::Result;
    /// use musli_zerocopy::{trie, OwnedBuf};
    ///
    /// // Helper to convert output to utf-8.
    /// fn to_utf8<'buf, E>(result: Result<(&'buf [u8], &'buf i32), E>) -> Result<(&'buf str, i32)>
    /// where
    ///     anyhow::Error: From<E>
    /// {
    ///     let (k, v) = result?;
    ///     Ok((from_utf8(k)?, *v))
    /// }
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let values = [
    ///     (buf.store_unsized("work"), 1),
    ///     (buf.store_unsized("worker"), 2),
    ///     (buf.store_unsized("workers"), 3),
    ///     (buf.store_unsized("working"), 4),
    ///     (buf.store_unsized("working"), 5),
    ///     (buf.store_unsized("working man"), 6),
    ///     (buf.store_unsized("run"), 7),
    ///     (buf.store_unsized("running"), 8),
    /// ];
    ///
    /// let trie = trie::store(&mut buf, values)?;
    ///
    /// let mut values = trie.iter_fixed::<16>(&buf)
    ///     .map(to_utf8)
    ///     .collect::<Result<Vec<_>>>()?;
    /// values.sort();
    ///
    /// assert_eq! {
    ///     values,
    ///     [
    ///         ("run", 7),
    ///         ("running", 8),
    ///         ("work", 1),
    ///         ("worker", 2),
    ///         ("workers", 3),
    ///         ("working", 4),
    ///         ("working", 5),
    ///         ("working man", 6),
    ///     ]
    /// };
    /// # Ok::<_, anyhow::Error>(())
    /// ```
    pub fn iter_fixed<'buf, const N: usize>(&self, buf: &'buf Buf) -> IterFixed<'buf, N, T, F> {
        IterFixed {
            iter: Walk::find(buf, self.links, &[]),
        }
    }

    /// Construct an iterator over all matching string prefixes in the trie
    /// which also returns the string of the entries.
    ///
    /// Note that the iteration order is unspecified and might change in future
    /// versions.
    ///
    /// # Errors
    ///
    /// This errors in case the trie being iterated over is structurally
    /// invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::from_utf8;
    ///
    /// use anyhow::Result;
    /// use musli_zerocopy::{trie, OwnedBuf};
    ///
    /// // Helper to convert output to utf-8.
    /// fn to_utf8<'buf, E>(result: Result<(&'buf [u8], &'buf i32), E>) -> Result<(&'buf str, i32)>
    /// where
    ///     anyhow::Error: From<E>
    /// {
    ///     let (k, v) = result?;
    ///     Ok((from_utf8(k)?, *v))
    /// }
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let values = [
    ///     (buf.store_unsized("work"), 1),
    ///     (buf.store_unsized("worker"), 2),
    ///     (buf.store_unsized("workers"), 3),
    ///     (buf.store_unsized("working"), 4),
    ///     (buf.store_unsized("working"), 5),
    ///     (buf.store_unsized("working man"), 6),
    ///     (buf.store_unsized("run"), 7),
    ///     (buf.store_unsized("running"), 8),
    /// ];
    ///
    /// let trie = trie::store(&mut buf, values)?;
    ///
    /// let mut values = trie
    ///     .iter_in(&buf, "workin")
    ///     .map(to_utf8)
    ///     .collect::<Result<Vec<_>>>()?;
    /// values.sort();
    ///
    /// assert_eq!(values, [("working", 4), ("working", 5), ("working man", 6)]);
    ///
    /// let mut values = trie
    ///     .iter_in(&buf, "wor")
    ///     .map(to_utf8)
    ///     .collect::<Result<Vec<_>>>()?;
    /// values.sort();
    ///
    /// assert_eq! {
    ///     values,
    ///     [
    ///         ("work", 1),
    ///         ("worker", 2),
    ///         ("workers", 3),
    ///         ("working", 4),
    ///         ("working", 5),
    ///         ("working man", 6),
    ///     ]
    /// };
    ///
    /// let mut values = trie
    ///     .iter_in(&buf, "runn")
    ///     .map(to_utf8)
    ///     .collect::<Result<Vec<_>>>()?;
    /// values.sort();
    ///
    /// assert_eq!(values, [("running", 8)]);
    /// # Ok::<_, anyhow::Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    pub fn iter_in<'a, 'buf, S>(&self, buf: &'buf Buf, prefix: &'a S) -> IterIn<'a, 'buf, T, F>
    where
        S: ?Sized + AsRef<[u8]>,
    {
        IterIn {
            iter: Walk::find(buf, self.links, prefix.as_ref()),
        }
    }

    /// Construct an iterator over all matching string prefixes in the trie
    /// which also returns the string of the entries using a fixed max iteration
    /// depth of `N`.
    ///
    /// Note that the iteration order is unspecified and might change in future
    /// versions.
    ///
    /// # Errors
    ///
    /// This errors in case the trie being iterated over is structurally
    /// invalid or if the iteration depth exceeds `N`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::from_utf8;
    ///
    /// use anyhow::Result;
    /// use musli_zerocopy::{trie, OwnedBuf};
    ///
    /// // Helper to convert output to utf-8.
    /// fn to_utf8<'buf, E>(result: Result<(&'buf [u8], &'buf i32), E>) -> Result<(&'buf str, i32)>
    /// where
    ///     anyhow::Error: From<E>
    /// {
    ///     let (k, v) = result?;
    ///     Ok((from_utf8(k)?, *v))
    /// }
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let values = [
    ///     (buf.store_unsized("work"), 1),
    ///     (buf.store_unsized("worker"), 2),
    ///     (buf.store_unsized("workers"), 3),
    ///     (buf.store_unsized("working"), 4),
    ///     (buf.store_unsized("working"), 5),
    ///     (buf.store_unsized("working man"), 6),
    ///     (buf.store_unsized("run"), 7),
    ///     (buf.store_unsized("running"), 8),
    /// ];
    ///
    /// let trie = trie::store(&mut buf, values)?;
    ///
    /// let mut values = trie
    ///     .iter_in_fixed::<16, _>(&buf, "workin")
    ///     .map(to_utf8)
    ///     .collect::<Result<Vec<_>>>()?;
    /// values.sort();
    ///
    /// assert_eq!(values, [("working", 4), ("working", 5), ("working man", 6)]);
    ///
    /// let mut values = trie
    ///     .iter_in_fixed::<16, _>(&buf, "wor")
    ///     .map(to_utf8)
    ///     .collect::<Result<Vec<_>>>()?;
    /// values.sort();
    ///
    /// assert_eq! {
    ///     values,
    ///     [
    ///         ("work", 1),
    ///         ("worker", 2),
    ///         ("workers", 3),
    ///         ("working", 4),
    ///         ("working", 5),
    ///         ("working man", 6),
    ///     ]
    /// };
    ///
    /// let mut values = trie
    ///     .iter_in_fixed::<16, _>(&buf, "runn")
    ///     .map(to_utf8)
    ///     .collect::<Result<Vec<_>>>()?;
    /// values.sort();
    ///
    /// assert_eq!(values, [("running", 8)]);
    /// # Ok::<_, anyhow::Error>(())
    /// ```
    pub fn iter_in_fixed<'a, 'buf, const N: usize, S>(
        &self,
        buf: &'buf Buf,
        prefix: &'a S,
    ) -> IterInFixed<'a, 'buf, N, T, F>
    where
        S: ?Sized + AsRef<[u8]>,
    {
        IterInFixed {
            iter: Walk::find(buf, self.links, prefix.as_ref()),
        }
    }
}

/// An iterator over values matching a `prefix` in a [`TrieRef`].
///
/// See [`TrieRef::values_in()`].
#[cfg(feature = "alloc")]
pub struct ValuesIn<'a, 'buf, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    iter: Walk<'a, 'buf, T, F, Vec<StackEntry<'buf, T, F>>>,
}

#[cfg(feature = "alloc")]
impl<'buf, T, F> Iterator for ValuesIn<'_, 'buf, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    type Item = Result<&'buf T, Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (_, value) = match self.iter.poll() {
            Ok(entry) => entry?,
            Err(error) => return Some(Err(error)),
        };

        Some(Ok(value))
    }
}

/// An iterator over values matching a `prefix` in a [`TrieRef`] using a fixed
/// max iteration depth of `N`
///
/// See [`TrieRef::values_in_fixed()`].
pub struct ValuesInFixed<'a, 'buf, const N: usize, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    iter: Walk<'a, 'buf, T, F, ArrayStack<StackEntry<'buf, T, F>, N>>,
}

impl<'buf, const N: usize, T, F> Iterator for ValuesInFixed<'_, 'buf, N, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    type Item = Result<&'buf T, Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (_, value) = match self.iter.poll() {
            Ok(entry) => entry?,
            Err(error) => return Some(Err(error)),
        };

        Some(Ok(value))
    }
}

/// An iterator over all values in a [`TrieRef`].
///
/// See [`TrieRef::values()`].
#[cfg(feature = "alloc")]
pub struct Values<'buf, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    iter: Walk<'static, 'buf, T, F, Vec<StackEntry<'buf, T, F>>>,
}

#[cfg(feature = "alloc")]
impl<'buf, T, F> Iterator for Values<'buf, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    type Item = Result<&'buf T, Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (_, value) = match self.iter.poll() {
            Ok(entry) => entry?,
            Err(error) => return Some(Err(error)),
        };

        Some(Ok(value))
    }
}

/// An iterator over all values in a [`TrieRef`] using a fixed max iteration
/// depth of `N`
///
/// See [`TrieRef::values_fixed()`].
pub struct ValuesFixed<'buf, const N: usize, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    iter: Walk<'static, 'buf, T, F, ArrayStack<StackEntry<'buf, T, F>, N>>,
}

impl<'buf, const N: usize, T, F> Iterator for ValuesFixed<'buf, N, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    type Item = Result<&'buf T, Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (_, value) = match self.iter.poll() {
            Ok(entry) => entry?,
            Err(error) => return Some(Err(error)),
        };

        Some(Ok(value))
    }
}

/// An iterator over all entries in a [`TrieRef`].
///
/// See [`TrieRef::iter()`].
#[cfg(feature = "alloc")]
pub struct Iter<'buf, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    iter: Walk<'static, 'buf, T, F, Vec<StackEntry<'buf, T, F>>>,
}

#[cfg(feature = "alloc")]
impl<'buf, T, F> Iterator for Iter<'buf, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    type Item = Result<(&'buf [u8], &'buf T), Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.poll().transpose()
    }
}

/// An iterator over all entries in a [`TrieRef`] using a fixed max iteration
/// depth of `N`
///
/// See [`TrieRef::iter_fixed()`].
pub struct IterFixed<'buf, const N: usize, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    iter: Walk<'static, 'buf, T, F, ArrayStack<StackEntry<'buf, T, F>, N>>,
}

impl<'buf, const N: usize, T, F> Iterator for IterFixed<'buf, N, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    type Item = Result<(&'buf [u8], &'buf T), Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.poll().transpose()
    }
}

/// An iterator over all entries inside of a `prefix` in a [`TrieRef`].
///
/// See [`TrieRef::iter_in()`].
#[cfg(feature = "alloc")]
pub struct IterIn<'a, 'buf, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    iter: Walk<'a, 'buf, T, F, Vec<StackEntry<'buf, T, F>>>,
}

#[cfg(feature = "alloc")]
impl<'buf, T, F> Iterator for IterIn<'_, 'buf, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    type Item = Result<(&'buf [u8], &'buf T), Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.poll().transpose()
    }
}

/// An iterator over all entries inside of a `prefix` in a [`TrieRef`] using a
/// fixed max iteration depth of `N`
///
/// See [`TrieRef::iter_in_fixed()`].
pub struct IterInFixed<'a, 'buf, const N: usize, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    iter: Walk<'a, 'buf, T, F, ArrayStack<StackEntry<'buf, T, F>, N>>,
}

impl<'buf, const N: usize, T, F> Iterator for IterInFixed<'_, 'buf, N, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    type Item = Result<(&'buf [u8], &'buf T), Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.poll().transpose()
    }
}

/// Debug printing of a trie.
///
/// See [`TrieRef::debug()`].
#[cfg(feature = "alloc")]
pub struct Debug<'a, 'buf, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    trie: &'a TrieRef<T, F>,
    buf: &'buf Buf,
}

#[cfg(feature = "alloc")]
impl<T, F> fmt::Debug for Debug<'_, '_, T, F>
where
    T: fmt::Debug + ZeroCopy,
    F: Flavor,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_map();

        for result in self.trie.iter(self.buf) {
            let (key, value) = result.map_err(|_| fmt::Error)?;
            f.entry(&LossyStr::new(key), value);
        }

        f.finish()
    }
}

/// Debug printing of a trie with a fixed iteration depth of `N`.
///
/// See [`TrieRef::debug_fixed()`].
pub struct DebugFixed<'a, 'buf, const N: usize, T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    trie: &'a TrieRef<T, F>,
    buf: &'buf Buf,
}

impl<const N: usize, T, F> fmt::Debug for DebugFixed<'_, '_, N, T, F>
where
    T: fmt::Debug + ZeroCopy,
    F: Flavor,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_map();

        for result in self.trie.iter_fixed::<N>(self.buf) {
            let (key, value) = result.map_err(|_| fmt::Error)?;
            f.entry(&LossyStr::new(key), value);
        }

        f.finish()
    }
}

impl<T, F> Clone for TrieRef<T, F>
where
    T: ZeroCopy,
    F: Flavor,
    F::Values<T>: Clone,
    F::Children<NodeRef<T, F>>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, F> Copy for TrieRef<T, F>
where
    T: ZeroCopy,
    F: Flavor,
    F::Values<T>: Copy,
    F::Children<NodeRef<T, F>>: Copy,
{
}

#[derive(ZeroCopy)]
#[zero_copy(crate)]
#[repr(C)]
struct LinksRef<T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    values: F::Values<T>,
    children: F::Children<NodeRef<T, F>>,
}

impl<T, F> Clone for LinksRef<T, F>
where
    T: ZeroCopy,
    F: Flavor,
    F::Values<T>: Copy,
    F::Children<NodeRef<T, F>>: Copy,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, F> Copy for LinksRef<T, F>
where
    T: ZeroCopy,
    F: Flavor,
    F::Values<T>: Copy,
    F::Children<NodeRef<T, F>>: Copy,
{
}

#[derive(ZeroCopy)]
#[zero_copy(crate)]
#[repr(C)]
struct NodeRef<T, F>
where
    T: ZeroCopy,
    F: Flavor,
{
    string: F::String,
    links: LinksRef<T, F>,
}

/// Calculate the common prefix between two strings.
fn prefix(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b.iter()).take_while(|(a, b)| a == b).count()
}
