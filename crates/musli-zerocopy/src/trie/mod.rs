//! A serialized prefix-trie.

#[cfg(test)]
mod tests;

use core::cmp::Ordering;
use core::fmt;
use core::mem::replace;
use core::slice::Iter;
use core::str;

use alloc::vec::Vec;

use crate::endian::Native;
use crate::pointer::Pointee;
use crate::slice::{self, BinarySearch, Slice};
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
pub struct Builder<T, F: Flavor> {
    links: Links<T, F>,
}

impl<T> Builder<T, DefaultFlavor> {
    /// Construct a new empty trie builder.
    pub const fn new() -> Self {
        Self::with_flavor()
    }
}

impl<T, F: Flavor> Builder<T, F> {
    /// Construct a new empty trie builder with a custom [`Flavor`].
    pub const fn with_flavor() -> Self {
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
                            string: F::String::from_ref(string),
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
                                string: F::String::from_ref(string),
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

struct Links<T, F: Flavor> {
    values: Vec<T>,
    children: Vec<Node<T, F>>,
}

impl<T, F: Flavor> Links<T, F> {
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

    fn into_ref<E: ByteOrder, O: Size>(
        self,
        buf: &mut OwnedBuf<E, O>,
    ) -> Result<LinksRef<T, F>, Error>
    where
        T: ZeroCopy,
    {
        let values = F::Values::from_ref(buf.store_slice(&self.values));

        let mut children = Vec::with_capacity(self.children.len());

        for node in self.children {
            children.push(node.into_ref(buf)?);
        }

        let children = F::Children::from_ref(buf.store_slice(&children));
        Ok(LinksRef { values, children })
    }
}

struct Node<T, F: Flavor> {
    string: F::String,
    links: Links<T, F>,
}

impl<T, F: Flavor> Node<T, F> {
    const fn new(string: F::String) -> Self {
        Self {
            string,
            links: Links::empty(),
        }
    }

    fn into_ref<E: ByteOrder, O: Size>(
        self,
        buf: &mut OwnedBuf<E, O>,
    ) -> Result<NodeRef<T, F>, Error>
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
    /// assert_eq!(format!("{:?}", trie.debug(&buf)), "{\"�(�\": [1], \"食べない\": [2]}");
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
            let search = slice::binary_search_by(buf, this.children, |c| {
                Ok(buf.load(c.string)?.cmp(string))
            })?;

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

    /// Construct an iterator over all matching string prefixes in the trie.
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
    /// let values = trie.prefix(&buf, "workin").collect::<Result<Vec<_>, _>>()?;
    /// assert!(values.into_iter().copied().eq([4, 5, 6]));
    ///
    /// let values = trie.prefix(&buf, "wor").collect::<Result<Vec<_>, _>>()?;
    /// assert!(values.into_iter().copied().eq([1, 2, 3, 4, 5, 6]));
    ///
    /// let values = trie.prefix(&buf, "runn").collect::<Result<Vec<_>, _>>()?;
    /// assert!(values.into_iter().copied().eq([8]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn prefix<'a, 'buf, S>(&self, buf: &'buf Buf, string: &'a S) -> Prefix<'a, 'buf, T, F>
    where
        S: ?Sized + AsRef<[u8]>,
    {
        Prefix {
            buf,
            state: PrefixState::Initial(self.links, string.as_ref()),
            stack: Vec::new(),
        }
    }
}

enum PrefixState<'a, 'buf, T, F: Flavor>
where
    T: ZeroCopy,
{
    Initial(LinksRef<T, F>, &'a [u8]),
    Iter(Iter<'buf, T>),
    Stack,
}

/// A prefix iterator over a trie.
pub struct Prefix<'a, 'buf, T, F: Flavor>
where
    T: ZeroCopy,
{
    buf: &'buf Buf,
    state: PrefixState<'a, 'buf, T, F>,
    stack: Vec<(LinksRef<T, F>, usize)>,
}

impl<'a, 'buf, T, F: Flavor> Prefix<'a, 'buf, T, F>
where
    T: ZeroCopy,
{
    fn poll(&mut self) -> Result<Option<&'buf T>, Error> {
        'outer: loop {
            match &mut self.state {
                PrefixState::Initial(mut this, mut string) => {
                    let links = 'links: loop {
                        let search = slice::binary_search_by(self.buf, this.children, |c| {
                            Ok(self.buf.load(c.string)?.cmp(string))
                        })?;

                        match search {
                            BinarySearch::Found(n) => {
                                break self.buf.load(this.children.get_unchecked(n))?.links;
                            }
                            BinarySearch::Missing(n) => {
                                // For missing nodes, we need to find any
                                // neighbor for which the current string is a
                                // prefix. So unless `n` is out of bounds we
                                // look at the prior or current index `n`.
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

                                    if prefix == string.len() {
                                        break 'links child.links;
                                    }

                                    string = &string[prefix..];
                                    this = child.links;
                                    continue 'links;
                                }

                                // Falling through here indicates that we have
                                // not found anything. Assigning the stack state
                                // with an empty stack will cause the iterator
                                // to continuously return `None`.
                                self.state = PrefixState::Stack;
                                continue 'outer;
                            }
                        };
                    };

                    self.stack.push((links, 0));
                    self.state = PrefixState::Iter(self.buf.load(links.values)?.iter());
                }
                PrefixState::Iter(values) => {
                    let Some(value) = values.next() else {
                        self.state = PrefixState::Stack;
                        continue;
                    };

                    return Ok(Some(value));
                }
                PrefixState::Stack => loop {
                    let Some((links, index)) = self.stack.pop() else {
                        break 'outer;
                    };

                    let Some(node) = links.children.get(index) else {
                        continue;
                    };

                    let node = self.buf.load(node)?;
                    self.state = PrefixState::Iter(self.buf.load(node.links.values)?.iter());
                    self.stack.push((links, index + 1));
                    self.stack.push((node.links, 0));
                    continue 'outer;
                },
            }
        }

        Ok(None)
    }
}

impl<'a, 'buf, T, F: Flavor> Iterator for Prefix<'a, 'buf, T, F>
where
    T: ZeroCopy,
{
    type Item = Result<&'buf T, Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.poll().transpose()
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
        struct LossyStr<'a>(&'a [u8]);

        impl fmt::Debug for LossyStr<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut bytes = self.0;

                write!(f, "\"")?;

                loop {
                    let (string, replacement) = match str::from_utf8(bytes) {
                        Ok(s) => (s, false),
                        Err(e) => {
                            let (valid, invalid) = bytes.split_at(e.valid_up_to());
                            bytes = invalid.get(1..).unwrap_or_default();
                            (unsafe { str::from_utf8_unchecked(valid) }, true)
                        }
                    };

                    for c in string.chars() {
                        match c {
                            '\0' => write!(f, "\\0")?,
                            '\x01'..='\x08' | '\x0b' | '\x0c' | '\x0e'..='\x19' | '\x7f' => {
                                write!(f, "\\x{:02x}", c as u32)?;
                            }
                            _ => {
                                write!(f, "{}", c.escape_debug())?;
                            }
                        }
                    }

                    if !replacement {
                        break;
                    }

                    write!(f, "\u{FFFD}")?;
                }

                write!(f, "\"")?;
                Ok(())
            }
        }

        fn walk<T, F: Flavor>(
            buf: &Buf,
            s: &mut Vec<u8>,
            f: &mut fmt::DebugMap<'_, '_>,
            links: LinksRef<T, F>,
        ) -> fmt::Result
        where
            T: fmt::Debug + ZeroCopy,
        {
            let children = buf.load(links.children).map_err(|_| fmt::Error)?;

            for child in children {
                let string = buf.load(child.string).map_err(|_| fmt::Error)?;

                let len = s.len();
                s.extend(string);

                if !child.links.values.is_empty() {
                    let values = buf.load(child.links.values).map_err(|_| fmt::Error)?;
                    f.entry(&LossyStr(s), &values);
                }

                walk(buf, s, f, child.links)?;
                s.truncate(len);
            }

            Ok(())
        }

        let mut s = Vec::new();
        let mut f = f.debug_map();
        walk(self.buf, &mut s, &mut f, self.trie.links)?;
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
#[repr(C, packed)]
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
#[repr(C, packed)]
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
