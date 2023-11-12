use core::slice;

use crate::endian::Native;
use crate::error::ErrorKind;
use crate::slice::{binary_search_by, BinarySearch, Slice};
use crate::stack::Stack;
use crate::{Buf, Error, Ref, ZeroCopy};

use super::{prefix, Flavor, LinksRef, StackEntry};

pub(super) struct Walk<'a, 'buf, T, F: Flavor, S: Stack<StackEntry<'buf, T, F>>>
where
    T: ZeroCopy,
{
    // Buffer being walked.
    buf: &'buf Buf,
    // State of the current walker.
    state: WalkState<'a, 'buf, T, F>,
    // A stack which indicates the links who's children we should visit next,
    // and an index corresponding to the child to visit.
    stack: S,
}

impl<'a, 'buf, T, F: Flavor, S> Walk<'a, 'buf, T, F, S>
where
    T: ZeroCopy,
    S: Stack<StackEntry<'buf, T, F>>,
{
    pub(super) fn find(buf: &'buf Buf, links: LinksRef<T, F>, prefix: &'a [u8]) -> Self {
        Self {
            buf,
            state: WalkState::Find(links, prefix),
            stack: S::new(),
        }
    }

    pub(super) fn poll(&mut self) -> Result<Option<(&'buf [u8], &'buf T)>, Error> {
        'outer: loop {
            match self.state {
                WalkState::Find(this, &[]) => {
                    let iter = self.buf.load(this.values)?.iter();

                    if !self.stack.try_push((this, 0, &[])) {
                        return Err(Error::new(ErrorKind::StackOverflow {
                            capacity: S::CAPACITY,
                        }));
                    }

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

                    if !self.stack.try_push((node.links, 0, prefix)) {
                        return Err(Error::new(ErrorKind::StackOverflow {
                            capacity: S::CAPACITY,
                        }));
                    }

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

                    if !self.stack.try_push((links, index + 1, prefix)) {
                        return Err(Error::new(ErrorKind::StackOverflow {
                            capacity: S::CAPACITY,
                        }));
                    }

                    if !self.stack.try_push((node.links, 0, new_prefix)) {
                        return Err(Error::new(ErrorKind::StackOverflow {
                            capacity: S::CAPACITY,
                        }));
                    }

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
