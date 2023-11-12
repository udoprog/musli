//! A serialized prefix-trie.

#[cfg(test)]
mod tests;

use core::cmp::Ordering;
use core::fmt;
use core::marker::PhantomData;
use core::mem::replace;
use core::slice;

use alloc::vec::Vec;

use crate::endian::Native;
use crate::error::ErrorKind;
use crate::lossy_str::LossyStr;
use crate::pointer::Pointee;
use crate::slice::{binary_search_by, BinarySearch, Slice};
use crate::{Buf, ByteOrder, DefaultSize, Error, OwnedBuf, Ref, Size, ZeroCopy};

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
///     // The max number of values stored in a single node is `u16::MAX`.
///     type Values<T> = Packed<[T], u32, u16> where T: ZeroCopy;
///     // The maximum number of children for a single node is `u8::MAX`.
///     type Children<T> = Packed<[T], u32, u8> where T: ZeroCopy;
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
pub struct DefaultFlavor<E: ByteOrder = Native, O: Size = DefaultSize>(
    core::marker::PhantomData<(E, O)>,
);

impl<E: ByteOrder, O: Size> Flavor for DefaultFlavor<E, O> {
    type String = Ref<[u8], E, O>;
    type Values<T> = Ref<[T], E, O> where T: ZeroCopy;
    type Children<T> = Ref<[T], E, O> where T: ZeroCopy;
}

/// Store the given collection in a trie.
///
/// The trie is stored as a graph, where each level contains a sorted collection
/// of strings. Each level is traversed using a binary search. Since levels are
/// expected to be relatively small, this produces a decent time to complexity
/// tradeoff.
///
/// Note that construction of the trie is the most performant if the input keys
/// are already sorted. Otherwise trie construction might require many
/// re-allocations.
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
pub fn store<S, E: ByteOrder, O: Size, I, T>(
    buf: &mut OwnedBuf<E, O>,
    it: I,
) -> Result<TrieRef<T, DefaultFlavor<E, O>>, Error>
where
    I: IntoIterator<Item = (Ref<S, E, O>, T)>,
    T: ZeroCopy,
    S: ?Sized + Pointee<O, Packed = <[u8] as Pointee<O>>::Packed>,
{
    // First step is to construct the trie in-memory.
    let mut trie = Builder::with_flavor();

    for (string, value) in it {
        trie.insert(buf, string, value)?;
    }

    trie.build(buf)
}

/// An in-memory trie structure as it's being constructed.
///
/// This can be used over [`store()`] to provide more control.
pub struct Builder<T, F: Flavor = DefaultFlavor> {
    links: Links<T>,
    _marker: PhantomData<F>,
}

impl<T> Builder<T> {
    /// Construct a new empty trie builder with the default [`DefaultFlavor`].
    #[inline]
    pub const fn new() -> Self {
        Self::with_flavor()
    }
}

impl<T, F: Flavor> Builder<T, F> {
    /// Construct a new empty trie builder with a custom [`Flavor`].
    pub const fn with_flavor() -> Self {
        Self {
            links: Links::empty(),
            _marker: PhantomData,
        }
    }

    /// Insert a value into the trie.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use musli_zerocopy::{trie, OwnedBuf};
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let mut trie = trie::Builder::new();
    ///
    /// let key = buf.store_unsized("working");
    /// trie.insert(&buf, key, 4)?;
    /// let key = buf.store_unsized("working man");
    /// trie.insert(&buf, key, 6)?;
    /// let key = buf.store_unsized("work");
    /// trie.insert(&buf, key, 1)?;
    /// let key = buf.store_unsized("worker");
    /// trie.insert(&buf, key, 2)?;
    /// let key = buf.store_unsized("workers");
    /// trie.insert(&buf, key, 3)?;
    /// let key = buf.store_unsized("working");
    /// trie.insert(&buf, key, 5)?;
    ///
    /// let trie = trie.build(&mut buf)?;
    ///
    /// assert_eq!(trie.get(&buf, "aard")?, None);
    /// assert_eq!(trie.get(&buf, "worker")?, Some(&[2][..]));
    /// assert_eq!(trie.get(&buf, "working")?, Some(&[4, 5][..]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn insert<S, E: ByteOrder, O: Size>(
        &mut self,
        buf: &Buf,
        string: Ref<S, E, O>,
        value: T,
    ) -> Result<(), Error>
    where
        S: ?Sized + Pointee<O, Packed = <[u8] as Pointee<O>>::Packed>,
    {
        let mut string = string.cast::<[u8]>();
        let mut current = buf.load(string)?;
        let mut this = &mut self.links;

        loop {
            let search =
                try_binary_search_by(&this.children, |c| Ok(buf.load(c.string)?.cmp(current)))?;

            match search {
                BinarySearch::Found(n) => {
                    this.children[n].links.values.push(value);
                    return Ok(());
                }
                BinarySearch::Missing(0) => {
                    this.children.insert(
                        0,
                        Node {
                            string: Ref::try_with_metadata(string.offset(), string.len())?,
                            links: Links::new(value),
                        },
                    );
                    return Ok(());
                }
                BinarySearch::Missing(n) => {
                    let pre = n - 1;

                    // Find common prefix and split nodes if necessary.
                    let prefix = prefix(buf.load(this.children[pre].string)?, current);

                    // No common prefix in prior node, so a new node is needed.
                    if prefix == 0 {
                        this.children.insert(
                            n,
                            Node {
                                string: Ref::try_with_metadata(string.offset(), string.len())?,
                                links: Links::new(value),
                            },
                        );
                        return Ok(());
                    }

                    let child = &mut this.children[pre];

                    // This happens if `current` contains a shorter subset match
                    // than the string represented by `child`, like `work` and
                    // the child string is `working` (common prefix is `work`).
                    //
                    // In that scenario, the child node has to be split up, so we transpose it from:
                    //
                    // ```
                    // "working" => { values = [1, 2, 3] }
                    // =>
                    // "work" => { "ing" => { values = [1, 2, 3] } }
                    // ```
                    if prefix != child.string.len() {
                        let (prefix, suffix) = child.string.split_at(prefix);
                        let new_node = Node::new(prefix);
                        let mut replaced = replace(child, new_node);
                        replaced.string = suffix;
                        child.links.children.push(replaced);
                    }

                    current = &current[prefix..];
                    string = string.split_at(prefix).1;
                    this = &mut child.links;
                }
            }
        }
    }

    /// Construct a [`TrieRef`] out of the current [`Builder`].
    ///
    /// # Errors
    ///
    /// Trie construction will error in case an interior node overflows its
    /// representation as per its [`Flavor`] defined by `F`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use musli_zerocopy::{trie, OwnedBuf};
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let mut trie = trie::Builder::new();
    ///
    /// let key = buf.store_unsized("work");
    /// trie.insert(&buf, key, 1)?;
    /// let key = buf.store_unsized("working");
    /// trie.insert(&buf, key, 4)?;
    ///
    /// let trie = trie.build(&mut buf)?;
    ///
    /// assert_eq!(trie.get(&buf, "aard")?, None);
    /// assert_eq!(trie.get(&buf, "work")?, Some(&[1][..]));
    /// assert_eq!(trie.get(&buf, "working")?, Some(&[4][..]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn build<E: ByteOrder, O: Size>(
        self,
        buf: &mut OwnedBuf<E, O>,
    ) -> Result<TrieRef<T, F>, Error>
    where
        T: ZeroCopy,
    {
        Ok(TrieRef {
            links: self.links.into_ref(buf)?,
        })
    }
}

struct Links<T> {
    values: Vec<T>,
    children: Vec<Node<T>>,
}

impl<T> Links<T> {
    const fn empty() -> Self {
        Self {
            values: Vec::new(),
            children: Vec::new(),
        }
    }

    fn new(value: T) -> Self {
        Self {
            values: alloc::vec![value],
            children: Vec::new(),
        }
    }

    fn into_ref<E: ByteOrder, O: Size, F: Flavor>(
        self,
        buf: &mut OwnedBuf<E, O>,
    ) -> Result<LinksRef<T, F>, Error>
    where
        T: ZeroCopy,
    {
        let values = F::Values::try_from_ref(buf.store_slice(&self.values))?;

        let mut children = Vec::with_capacity(self.children.len());

        for node in self.children {
            children.push(node.into_ref(buf)?);
        }

        let children = F::Children::try_from_ref(buf.store_slice(&children))?;
        Ok(LinksRef { values, children })
    }
}

struct Node<T> {
    string: Ref<[u8], Native, usize>,
    links: Links<T>,
}

impl<T> Node<T> {
    const fn new(string: Ref<[u8], Native, usize>) -> Self {
        Self {
            string,
            links: Links::empty(),
        }
    }

    fn into_ref<E: ByteOrder, O: Size, F: Flavor>(
        self,
        buf: &mut OwnedBuf<E, O>,
    ) -> Result<NodeRef<T, F>, Error>
    where
        T: ZeroCopy,
    {
        Ok(NodeRef {
            string: F::String::try_from_ref(self.string)?,
            links: self.links.into_ref(buf)?,
        })
    }
}

/// A stored reference to a trie.
#[derive(ZeroCopy)]
#[zero_copy(crate)]
#[repr(C)]
pub struct TrieRef<T, F: Flavor = DefaultFlavor>
where
    T: ZeroCopy,
{
    links: LinksRef<T, F>,
}

impl<T, F: Flavor> TrieRef<T, F>
where
    T: ZeroCopy,
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
    pub fn debug<'a, 'buf>(&'a self, buf: &'buf Buf) -> Debug<'a, 'buf, T, F>
    where
        T: fmt::Debug,
    {
        Debug { trie: self, buf }
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
    pub fn values<'buf>(&self, buf: &'buf Buf) -> Values<'buf, T, F> {
        let iter = Walk {
            buf,
            state: WalkState::Find(self.links, &[]),
            stack: Vec::new(),
        };

        Values { iter }
    }

    /// Construct an iterator over values that are inside of the specified
    /// `prefix` in the trie.
    ///
    /// Note that the iteration order is unspecified and might change in future
    /// versions.
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
    pub fn values_in<'a, 'buf, S>(&self, buf: &'buf Buf, prefix: &'a S) -> ValuesIn<'a, 'buf, T, F>
    where
        S: ?Sized + AsRef<[u8]>,
    {
        let iter = Walk {
            buf,
            state: WalkState::Find(self.links, prefix.as_ref()),
            stack: Vec::new(),
        };

        ValuesIn { iter }
    }

    /// Construct an iterator over all entries in the trie.
    ///
    /// Note that the iteration order is unspecified and might change in future
    /// versions.
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
    pub fn iter<'buf>(&self, buf: &'buf Buf) -> Iter<'buf, T, F> {
        let iter = Walk {
            buf,
            state: WalkState::Find(self.links, b""),
            stack: Vec::new(),
        };

        Iter { iter }
    }

    /// Construct an iterator over all matching string prefixes in the trie
    /// which also returns the string of the entries.
    ///
    /// Note that the iteration order is unspecified and might change in future
    /// versions.
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
    pub fn iter_in<'a, 'buf, S>(&self, buf: &'buf Buf, string: &'a S) -> IterIn<'a, 'buf, T, F>
    where
        S: ?Sized + AsRef<[u8]>,
    {
        let iter = Walk {
            buf,
            state: WalkState::Find(self.links, string.as_ref()),
            stack: Vec::new(),
        };

        IterIn { iter }
    }
}

enum WalkState<'a, 'buf, T, F: Flavor>
where
    T: ZeroCopy,
{
    // Initial state where we need to lookup the specified prefix in the trie.
    Find(LinksRef<T, F>, &'a [u8]),
    // Values are being yielded.
    Values(&'buf [u8], slice::Iter<'buf, T>),
    // Stack traversal.
    Stack,
}

struct Walk<'a, 'buf, T, F: Flavor>
where
    T: ZeroCopy,
{
    // Buffer being walked.
    buf: &'buf Buf,
    // State of the current walker.
    state: WalkState<'a, 'buf, T, F>,
    // A stack which indicates the links who's children we should visit next,
    // and an index corresponding to the child to visit.
    stack: Vec<(LinksRef<T, F>, usize, &'buf [u8])>,
}

impl<'a, 'buf, T, F: Flavor> Walk<'a, 'buf, T, F>
where
    T: ZeroCopy,
{
    fn poll(&mut self) -> Result<Option<(&'buf [u8], &'buf T)>, Error> {
        'outer: loop {
            match self.state {
                WalkState::Find(this, &[]) => {
                    let iter = self.buf.load(this.values)?.iter();
                    self.stack.push((this, 0, &[]));
                    self.state = WalkState::Values(&[], iter);
                    continue;
                }
                WalkState::Find(this, string) => {
                    let mut this = this;
                    let mut string = string;
                    let mut len = 0;

                    let node = 'node: loop {
                        let search = binary_search_by(self.buf, this.children, |c| {
                            Ok(self.buf.load(c.string)?.cmp(string))
                        })?;

                        match search {
                            BinarySearch::Found(n) => {
                                break 'node self.buf.load(this.children.get_unchecked(n))?;
                            }
                            BinarySearch::Missing(n) => {
                                // For missing nodes, we need to find any
                                // neighbor for which the current string is a
                                // prefix. So unless `n` is out of bounds we
                                // look at the prior or current index `n`.
                                //
                                // Note that thanks to structural invariants,
                                // only one node may be a matching prefix.
                                let iter = n
                                    .checked_sub(1)
                                    .into_iter()
                                    .chain((n < this.children.len()).then_some(n));

                                for n in iter {
                                    let child = self.buf.load(this.children.get_unchecked(n))?;

                                    // Find common prefix and split nodes if necessary.
                                    let prefix = prefix(self.buf.load(child.string)?, string);

                                    if prefix == 0 {
                                        continue;
                                    }

                                    if prefix != string.len() {
                                        len += prefix;
                                        string = &string[prefix..];
                                        this = child.links;
                                        continue 'node;
                                    }

                                    break 'node child;
                                }

                                // Falling through here indicates that we have
                                // not found anything. Assigning the stack state
                                // with an empty stack will cause the iterator
                                // to continuously return `None`.
                                self.state = WalkState::Stack;
                                continue 'outer;
                            }
                        };
                    };

                    let prefix = prefix_string(node.string, len)?;
                    let prefix = self.buf.load(prefix)?;

                    self.stack.push((node.links, 0, prefix));
                    self.state =
                        WalkState::Values(prefix, self.buf.load(node.links.values)?.iter());
                }
                WalkState::Values(prefix, ref mut values) => {
                    let Some(value) = values.next() else {
                        self.state = WalkState::Stack;
                        continue;
                    };

                    return Ok(Some((prefix, value)));
                }
                WalkState::Stack => loop {
                    let Some((links, index, prefix)) = self.stack.pop() else {
                        break 'outer;
                    };

                    let Some(node) = links.children.get(index) else {
                        continue;
                    };

                    let node = self.buf.load(node)?;

                    let new_prefix = prefix_string(node.string, prefix.len())?;
                    let new_prefix = self.buf.load(new_prefix)?;

                    self.state =
                        WalkState::Values(new_prefix, self.buf.load(node.links.values)?.iter());
                    self.stack.push((links, index + 1, prefix));
                    self.stack.push((node.links, 0, new_prefix));
                    continue 'outer;
                },
            }
        }

        Ok(None)
    }
}

/// Calculate a prefix string based on an existing string in the trie.
///
/// We use the fact that during construction the trie must have been provided a
/// complete string reference, so any substring that we constructed must be
/// prefixed with its complete counterpart.
fn prefix_string<S>(string: S, prefix_len: usize) -> Result<Ref<[u8], Native, usize>, Error>
where
    S: Slice<Item = u8>,
{
    // NB: All of these operations have to be checked, since they are preformed
    // over untrusted data and we'd like to avoid a panic.

    let string_offset = string.offset();
    let string_len = string.len();

    let Some(real_start) = string_offset.checked_sub(prefix_len) else {
        return Err(Error::new(ErrorKind::Underflow {
            at: string_offset,
            len: prefix_len,
        }));
    };

    let Some(real_end) = string_offset.checked_add(string_len) else {
        return Err(Error::new(ErrorKind::Overflow {
            at: string_offset,
            len: string_len,
        }));
    };

    Ref::try_with_metadata(real_start, real_end - real_start)
}

/// An iterator over values matching a `prefix` in a [`TrieRef`].
///
/// See [`TrieRef::values_in`].
pub struct ValuesIn<'a, 'buf, T, F: Flavor>
where
    T: ZeroCopy,
{
    iter: Walk<'a, 'buf, T, F>,
}

impl<'a, 'buf, T, F: Flavor> Iterator for ValuesIn<'a, 'buf, T, F>
where
    T: ZeroCopy,
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
/// See [`TrieRef::values`].
pub struct Values<'buf, T, F: Flavor>
where
    T: ZeroCopy,
{
    iter: Walk<'static, 'buf, T, F>,
}

impl<'buf, T, F: Flavor> Iterator for Values<'buf, T, F>
where
    T: ZeroCopy,
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
/// See [`TrieRef::iter`].
pub struct Iter<'buf, T, F: Flavor>
where
    T: ZeroCopy,
{
    iter: Walk<'static, 'buf, T, F>,
}

impl<'buf, T, F: Flavor> Iterator for Iter<'buf, T, F>
where
    T: ZeroCopy,
{
    type Item = Result<(&'buf [u8], &'buf T), Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.poll().transpose()
    }
}

/// An iterator over all entries inside of a `prefix` in a [`TrieRef`].
///
/// See [`TrieRef::iter_in`].
pub struct IterIn<'a, 'buf, T, F: Flavor>
where
    T: ZeroCopy,
{
    iter: Walk<'a, 'buf, T, F>,
}

impl<'a, 'buf, T, F: Flavor> Iterator for IterIn<'a, 'buf, T, F>
where
    T: ZeroCopy,
{
    type Item = Result<(&'buf [u8], &'buf T), Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.poll().transpose()
    }
}

/// Debug printing of a trie.
///
/// See [`TrieRef::debug`].
pub struct Debug<'a, 'buf, T, F: Flavor>
where
    T: ZeroCopy,
{
    trie: &'a TrieRef<T, F>,
    buf: &'buf Buf,
}

impl<'a, 'buf, T, F: Flavor> fmt::Debug for Debug<'a, 'buf, T, F>
where
    T: fmt::Debug + ZeroCopy,
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

impl<T, F: Flavor> Clone for TrieRef<T, F>
where
    T: ZeroCopy,
    F::Values<T>: Clone,
    F::Children<NodeRef<T, F>>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, F: Flavor> Copy for TrieRef<T, F>
where
    T: ZeroCopy,
    F::Values<T>: Copy,
    F::Children<NodeRef<T, F>>: Copy,
{
}

#[derive(ZeroCopy)]
#[zero_copy(crate)]
#[repr(C)]
struct LinksRef<T, F: Flavor>
where
    T: ZeroCopy,
{
    values: F::Values<T>,
    children: F::Children<NodeRef<T, F>>,
}

impl<T, F: Flavor> Clone for LinksRef<T, F>
where
    T: ZeroCopy,
    F::Values<T>: Copy,
    F::Children<NodeRef<T, F>>: Copy,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, F: Flavor> Copy for LinksRef<T, F>
where
    T: ZeroCopy,
    F::Values<T>: Copy,
    F::Children<NodeRef<T, F>>: Copy,
{
}

#[derive(ZeroCopy)]
#[zero_copy(crate)]
#[repr(C)]
struct NodeRef<T, F: Flavor>
where
    T: ZeroCopy,
{
    string: F::String,
    links: LinksRef<T, F>,
}

/// Calculate the common prefix between two strings.
fn prefix(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b.iter()).take_while(|(a, b)| a == b).count()
}

/// Helper function to perform a binary search over a loaded slice.
fn try_binary_search_by<T, F, E>(slice: &[T], mut f: F) -> Result<BinarySearch, E>
where
    F: FnMut(&T) -> Result<Ordering, E>,
{
    // INVARIANTS:
    // - 0 <= left <= left + size = right <= slice.len()
    // - f returns Less for everything in slice[..left]
    // - f returns Greater for everything in slice[right..]
    let mut size = slice.len();
    let mut left = 0;
    let mut right = size;

    while left < right {
        let mid = left + size / 2;

        // SAFETY: The while condition means `size` is strictly positive, so
        // `size/2 < size`. Thus `left + size/2 < left + size`, which coupled
        // with the `left + size <= slice.len()` invariant means we have `left +
        // size/2 < slice.len()`, and this is in-bounds.
        let value = unsafe { slice.get_unchecked(mid) };
        let cmp = f(value)?;

        // The reason why we use if/else control flow rather than match
        // is because match reorders comparison operations, which is perf sensitive.
        // This is x86 asm for u8: https://rust.godbolt.org/z/8Y8Pra.
        if cmp == Ordering::Less {
            left = mid + 1;
        } else if cmp == Ordering::Greater {
            right = mid;
        } else {
            return Ok(BinarySearch::Found(mid));
        }

        size = right - left;
    }

    Ok(BinarySearch::Missing(left))
}
