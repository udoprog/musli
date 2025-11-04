use core::cmp::Ordering;
use core::marker::PhantomData;
use core::mem::replace;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::endian::Native;
use crate::pointer::{Coerce, Pointee};
use crate::slice::{BinarySearch, Slice};
use crate::{Buf, ByteOrder, Error, OwnedBuf, Ref, Size, ZeroCopy};

use super::{DefaultFlavor, Flavor, LinksRef, NodeRef, TrieRef, prefix};

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
///     (buf.store_unsized("work")?, 1),
///     (buf.store_unsized("worker")?, 2),
///     (buf.store_unsized("workers")?, 3),
///     (buf.store_unsized("working")?, 4),
///     (buf.store_unsized("working")?, 5),
///     (buf.store_unsized("working man")?, 6),
/// ];
///
/// let trie = trie::store(&mut buf, values)?;
///
/// assert_eq!(trie.get(&buf, "aard")?, None);
/// assert_eq!(trie.get(&buf, "worker")?, Some(&[2][..]));
/// assert_eq!(trie.get(&buf, "working")?, Some(&[4, 5][..]));
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
#[cfg(feature = "alloc")]
pub fn store<S, E, O, I, T>(
    buf: &mut OwnedBuf<E, O>,
    it: I,
) -> Result<TrieRef<T, E, DefaultFlavor<O>>, Error>
where
    I: IntoIterator<Item = (Ref<S, E, O>, T)>,
    T: ZeroCopy,
    S: ?Sized + Pointee + Coerce<[u8]>,
    E: ByteOrder,
    O: Size,
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
#[cfg(feature = "alloc")]
pub struct Builder<T, F = DefaultFlavor>
where
    F: Flavor,
{
    links: Links<T>,
    _marker: PhantomData<F>,
}

#[cfg(feature = "alloc")]
impl<T> Builder<T> {
    /// Construct a new empty trie builder with the default [`DefaultFlavor`].
    #[inline]
    pub const fn new() -> Self {
        Self::with_flavor()
    }
}

#[cfg(feature = "alloc")]
impl<T> Default for Builder<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "alloc")]
impl<T, F> Builder<T, F>
where
    F: Flavor,
{
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
    /// ```
    /// use musli_zerocopy::{trie, OwnedBuf};
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let mut trie = trie::Builder::new();
    ///
    /// let key = buf.store_unsized("working")?;
    /// trie.insert(&buf, key, 4)?;
    /// let key = buf.store_unsized("working man")?;
    /// trie.insert(&buf, key, 6)?;
    /// let key = buf.store_unsized("work")?;
    /// trie.insert(&buf, key, 1)?;
    /// let key = buf.store_unsized("worker")?;
    /// trie.insert(&buf, key, 2)?;
    /// let key = buf.store_unsized("workers")?;
    /// trie.insert(&buf, key, 3)?;
    /// let key = buf.store_unsized("working")?;
    /// trie.insert(&buf, key, 5)?;
    ///
    /// let trie = trie.build(&mut buf)?;
    ///
    /// assert_eq!(trie.get(&buf, "aard")?, None);
    /// assert_eq!(trie.get(&buf, "worker")?, Some(&[2][..]));
    /// assert_eq!(trie.get(&buf, "working")?, Some(&[4, 5][..]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn insert<S, E, O>(
        &mut self,
        buf: &Buf,
        string: Ref<S, E, O>,
        value: T,
    ) -> Result<(), Error>
    where
        S: ?Sized + Pointee + Coerce<[u8]>,
        E: ByteOrder,
        O: Size,
    {
        let mut string = string.coerce::<[u8]>();
        let mut current = buf.load(string)?;
        let mut this = &mut self.links;

        loop {
            let search = try_binary_search_by(&this.children, |c| {
                Ok::<_, Error>(buf.load(c.string)?.cmp(current))
            })?;

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
    /// ```
    /// use musli_zerocopy::{trie, OwnedBuf};
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let mut trie = trie::Builder::new();
    ///
    /// let key = buf.store_unsized("work")?;
    /// trie.insert(&buf, key, 1)?;
    /// let key = buf.store_unsized("working")?;
    /// trie.insert(&buf, key, 4)?;
    ///
    /// let trie = trie.build(&mut buf)?;
    ///
    /// assert_eq!(trie.get(&buf, "aard")?, None);
    /// assert_eq!(trie.get(&buf, "work")?, Some(&[1][..]));
    /// assert_eq!(trie.get(&buf, "working")?, Some(&[4][..]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn build<E, O>(self, buf: &mut OwnedBuf<E, O>) -> Result<TrieRef<T, E, F>, Error>
    where
        T: ZeroCopy,
        E: ByteOrder,
        O: Size,
    {
        Ok(TrieRef {
            links: self.links.into_ref(buf)?,
        })
    }
}

#[cfg(feature = "alloc")]
struct Links<T> {
    values: Vec<T>,
    children: Vec<Node<T>>,
}

#[cfg(feature = "alloc")]
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

    fn into_ref<E, O, F>(self, buf: &mut OwnedBuf<E, O>) -> Result<LinksRef<T, E, F>, Error>
    where
        T: ZeroCopy,
        E: ByteOrder,
        O: Size,
        F: Flavor,
    {
        let values = F::Values::try_from_ref(buf.store_slice(&self.values)?)?;

        let mut children = Vec::with_capacity(self.children.len());

        for node in self.children {
            children.push(node.into_ref(buf)?);
        }

        let children = F::Children::try_from_ref(buf.store_slice(&children)?)?;
        Ok(LinksRef { values, children })
    }
}

#[cfg(feature = "alloc")]
struct Node<T> {
    string: Ref<[u8], Native, usize>,
    links: Links<T>,
}

#[cfg(feature = "alloc")]
impl<T> Node<T> {
    const fn new(string: Ref<[u8], Native, usize>) -> Self {
        Self {
            string,
            links: Links::empty(),
        }
    }

    fn into_ref<E, O, F>(self, buf: &mut OwnedBuf<E, O>) -> Result<NodeRef<T, E, F>, Error>
    where
        T: ZeroCopy,
        E: ByteOrder,
        O: Size,
        F: Flavor,
    {
        Ok(NodeRef {
            string: F::String::try_from_ref(self.string)?,
            links: self.links.into_ref(buf)?,
        })
    }
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
