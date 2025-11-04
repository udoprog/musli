//! 🦞 Crawdad: ChaRActer-Wise Double-Array Dictionary
//!
//! Crawdad is a library of natural language dictionaries using character-wise double-array tries.
//! The implementation is optimized for strings of multibyte-characters,
//! and you can enjoy fast text processing on such strings such as Japanese or Chinese.
//!
//! # Data structures
//!
//! Crawdad contains the two trie implementations:
//!
//! - [`Trie`] is a standard trie form that often provides the fastest queries.
//! - [`MpTrie`] is a minimal-prefix trie form that is memory-efficient for long strings.
//!
//! # Examples
//!
//! ## Looking up an input key
//!
//! To get a value associated with an input key, use [`Trie::exact_match()`].
//!
//! ```
//! use musli_zerocopy::crawdad::Trie;
//!
//! let keys = vec!["世界", "世界中", "国民"];
//! let trie = Trie::from_keys(&keys)?;
//!
//! assert_eq!(trie.exact_match("世界中".chars()), Some(1));
//! assert_eq!(trie.exact_match("日本中".chars()), None);
//! # Ok::<_, musli_zerocopy::Error>(())
//! ```
//!
//! ## Finding all occurrences of keys in an input text
//!
//! To search for all occurrences of registered keys in an input text,
//! use [`Trie::common_prefix_search()`] for all starting positions in the text.
//!
//! ```
//! use musli_zerocopy::crawdad::Trie;
//!
//! let keys = vec!["世界", "世界中", "国民"];
//! let trie = Trie::from_keys(&keys)?;
//!
//! let haystack: Vec<char> = "国民が世界中にて".chars().collect();
//! let mut matches = vec![];
//!
//! for i in 0..haystack.len() {
//!     for (v, j) in trie.common_prefix_search(haystack[i..].iter().copied()) {
//!         matches.push((v, i..i + j));
//!     }
//! }
//!
//! assert_eq!(
//!     matches,
//!     vec![(2, 0..2), (0, 3..5), (1, 3..6)]
//! );
//! # Ok::<_, musli_zerocopy::Error>(())
//! ```
//!
//! ## Serializing and deserializing the data structure
//!
//! To serialize/deserialize the data structure into/from a byte sequence,
//! use [`Trie::serialize_to_vec()`]/[`Trie::deserialize_from_slice()`].
//!
//! ```
//! use musli_zerocopy::crawdad::Trie;
//!
//! let keys = vec!["世界", "世界中", "国民"];
//! let trie = Trie::from_keys(&keys)?;
//! # Ok::<_, musli_zerocopy::Error>(())
//! ```

mod mapper;

mod mptrie;
pub use mptrie::MpTrie;

mod trie;
pub use trie::Trie;

#[cfg(feature = "alloc")]
mod builder;

mod node;
mod utils;
use self::node::Node;

pub(crate) const OFFSET_MASK: u32 = 0x7fff_ffff;
pub(crate) const INVALID_IDX: u32 = 0xffff_ffff;
pub(crate) const MAX_VALUE: u32 = OFFSET_MASK;
pub(crate) const END_CODE: u32 = 0;

/// Special terminator, which must not be contained in keys.
pub const END_MARKER: char = '\u{ffff}';
