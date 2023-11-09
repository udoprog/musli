//! A serialized prefix-trie.

#[cfg(test)]
mod tests;

use core::cmp::Ordering;
use core::fmt;
use core::mem::replace;

use alloc::vec::Vec;

use crate::endian::Native;
use crate::error::ErrorKind;
use crate::slice::{self, BinarySearch};
use crate::{Buf, ByteOrder, DefaultSize, Error, OwnedBuf, Ref, Size, ZeroCopy};

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
pub fn store<E: ByteOrder, O: Size, I, T>(
    buf: &mut OwnedBuf<E, O>,
    it: I,
) -> Result<TrieRef<T, E, O>, Error>
where
    I: IntoIterator<Item = (Ref<str, E, O>, T)>,
    T: ZeroCopy,
{
    // First step is to construct the trie in-memory.
    let mut trie = Builder::new();

    for (string, value) in it {
        trie.insert(buf, string, value)?;
    }

    trie.build(buf)
}

/// An in-memory trie structure as it's being constructed.
///
/// This can be used over [`store()`] to provide more control.
pub struct Builder<T, E: ByteOrder = Native, O: Size = DefaultSize> {
    links: Links<T, E, O>,
}

impl<T, E: ByteOrder, O: Size> Builder<T, E, O> {
    /// Construct a new empty trie builder.
    pub const fn new() -> Self {
        Self {
            links: Links::empty(),
        }
    }

    /// Insert a value into the trie.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use musli_zerocopy::{trie, OwnedBuf};
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
    pub fn insert(&mut self, buf: &Buf, mut string: Ref<str, E, O>, value: T) -> Result<(), Error> {
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
                            string,
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
                                string,
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
                        let new_node = Node::new(Ref::with_metadata(child.string.offset(), prefix));
                        let mut replaced = replace(child, new_node);

                        replaced.string = Ref::with_metadata(
                            replaced.string.offset() + prefix,
                            replaced.string.len() - prefix,
                        );

                        child.links.children.push(replaced);
                    }

                    current = &current[prefix..];
                    string = Ref::with_metadata(string.offset() + prefix, string.len() - prefix);
                    this = &mut child.links;
                }
            }
        }
    }

    /// Construct a [`TrieRef`] out of the current [`Builder`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use musli_zerocopy::{trie, OwnedBuf};
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
    pub fn build(self, buf: &mut OwnedBuf<E, O>) -> Result<TrieRef<T, E, O>, Error>
    where
        T: ZeroCopy,
    {
        Ok(TrieRef {
            links: self.links.into_ref(buf)?,
        })
    }
}

struct Links<T, E: ByteOrder = Native, O: Size = DefaultSize> {
    values: Vec<T>,
    children: Vec<Node<T, E, O>>,
}

impl<T, E: ByteOrder, O: Size> Links<T, E, O> {
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

    fn into_ref(self, buf: &mut OwnedBuf<E, O>) -> Result<LinksRef<T, E, O>, Error>
    where
        T: ZeroCopy,
    {
        let values = buf.store_slice(&self.values);

        let mut children = Vec::with_capacity(self.children.len());

        for node in self.children {
            children.push(node.into_ref(buf)?);
        }

        let children = buf.store_slice(&children);
        Ok(LinksRef { values, children })
    }
}

struct Node<T, E: ByteOrder = Native, O: Size = DefaultSize> {
    string: Ref<str, E, O>,
    links: Links<T, E, O>,
}

impl<T, E: ByteOrder, O: Size> Node<T, E, O> {
    const fn new(string: Ref<str, E, O>) -> Self {
        Self {
            string,
            links: Links::empty(),
        }
    }

    fn into_ref(self, buf: &mut OwnedBuf<E, O>) -> Result<NodeRef<T, E, O>, Error>
    where
        T: ZeroCopy,
    {
        Ok(NodeRef {
            string: self.string,
            links: self.links.into_ref(buf)?,
        })
    }
}

/// A stored reference to a trie.
#[derive(ZeroCopy)]
#[zero_copy(crate)]
#[repr(C)]
pub struct TrieRef<T, E: ByteOrder = Native, O: Size = DefaultSize>
where
    T: ZeroCopy,
{
    links: LinksRef<T, E, O>,
}

impl<T, E: ByteOrder, O: Size> TrieRef<T, E, O>
where
    T: ZeroCopy,
{
    /// Debug print the current trie.
    pub fn debug<'a>(&'a self, buf: &'a Buf) -> Debug<'a, T, E, O>
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
    pub fn get<'buf>(&self, buf: &'buf Buf, mut string: &str) -> Result<Option<&'buf [T]>, Error> {
        let mut this = &self.links;

        loop {
            let search = slice::binary_search_by(buf, this.children, |c| {
                Ok(buf.load(c.string)?.cmp(string))
            })?;

            match search {
                BinarySearch::Found(n) => {
                    let Some(child) = this.children.get(n) else {
                        return Err(Error::new(ErrorKind::IndexOutOfBounds {
                            index: n,
                            len: this.children.len(),
                        }));
                    };

                    let child = buf.load(child)?;
                    let values = buf.load(child.links.values)?;
                    return Ok(Some(values));
                }
                BinarySearch::Missing(0) => {
                    return Ok(None);
                }
                BinarySearch::Missing(n) => {
                    let index = n - 1;

                    let Some(child) = this.children.get(index) else {
                        return Err(Error::new(ErrorKind::IndexOutOfBounds {
                            index,
                            len: this.children.len(),
                        }));
                    };

                    let child = buf.load(child)?;

                    // Find common prefix and split nodes if necessary.
                    let prefix = prefix(buf.load(child.string)?, string);

                    if prefix == 0 {
                        return Ok(None);
                    }

                    string = &string[prefix..];
                    this = &child.links;
                }
            };
        }
    }
}

/// Debug printing of a trie.
///
/// See [`TrieRef::debug`].
pub struct Debug<'a, T, E: ByteOrder, O: Size>
where
    T: ZeroCopy,
{
    trie: &'a TrieRef<T, E, O>,
    buf: &'a Buf,
}

impl<'a, T, E: ByteOrder, O: Size> fmt::Debug for Debug<'a, T, E, O>
where
    T: fmt::Debug + ZeroCopy,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use alloc::string::String;

        fn walk<T, E: ByteOrder, O: Size>(
            buf: &Buf,
            s: &mut String,
            f: &mut fmt::DebugMap<'_, '_>,
            links: &LinksRef<T, E, O>,
        ) -> fmt::Result
        where
            T: fmt::Debug + ZeroCopy,
        {
            let children = buf.load(links.children).map_err(|_| fmt::Error)?;

            for child in children {
                let string = buf.load(child.string).map_err(|_| fmt::Error)?;

                let len = s.len();
                s.push_str(string);

                if !child.links.values.is_empty() {
                    let values = buf.load(child.links.values).map_err(|_| fmt::Error)?;
                    f.entry(&s.as_str(), &values);
                }

                walk(buf, s, f, &child.links)?;
                s.truncate(len);
            }

            Ok(())
        }

        let mut s = String::new();
        let mut f = f.debug_map();
        walk(self.buf, &mut s, &mut f, &self.trie.links)?;
        f.finish()
    }
}

impl<T, E: ByteOrder, O: Size> Clone for TrieRef<T, E, O>
where
    T: ZeroCopy,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, E: ByteOrder, O: Size> Copy for TrieRef<T, E, O> where T: ZeroCopy {}

#[derive(ZeroCopy)]
#[zero_copy(crate)]
#[repr(C)]
struct LinksRef<T, E: ByteOrder = Native, O: Size = DefaultSize>
where
    T: ZeroCopy,
{
    values: Ref<[T], E, O>,
    children: Ref<[NodeRef<T, E, O>], E, O>,
}

impl<T, E: ByteOrder, O: Size> Clone for LinksRef<T, E, O>
where
    T: ZeroCopy,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, E: ByteOrder, O: Size> Copy for LinksRef<T, E, O> where T: ZeroCopy {}

#[derive(ZeroCopy)]
#[zero_copy(crate)]
#[repr(C)]
struct NodeRef<T, E: ByteOrder = Native, O: Size = DefaultSize>
where
    T: ZeroCopy,
{
    string: Ref<str, E, O>,
    links: LinksRef<T, E, O>,
}

/// Calculate the common prefix between two strings.
fn prefix(a: &str, b: &str) -> usize {
    a.chars()
        .zip(b.chars())
        .take_while(|(a, b)| a == b)
        .map(|(c, _)| c.len_utf8())
        .sum()
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
