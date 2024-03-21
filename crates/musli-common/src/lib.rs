//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/musli-common.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-common)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--common-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-common)
//!
//! Common utilities shared by [Müsli] encodings.
//!
//! The [Reader] and [Writer] traits are defined in here which determined the
//! types that can be used in collaboration with [Müsli].
//!
//! Please refer to <https://docs.rs/musli> for documentation.
//!
//! [Müsli]: <https://docs.rs/musli>
//! [Reader]: https://docs.rs/musli-common/latest/musli-common/reader/trait.Reader.html
//! [Writer]: https://docs.rs/musli-common/latest/musli-common/writer/trait.Writer.html

#![deny(missing_docs)]
#![no_std]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

#[cfg_attr(test, macro_use)]
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

pub(crate) mod context;
pub(crate) mod fixed;
#[macro_use]
pub(crate) mod options;
pub(crate) mod buf;
pub(crate) mod reader;
pub(crate) mod wrap;
pub(crate) mod writer;

#[doc(hidden)]
pub mod exports {
    pub mod context {
        //! Helper types to set up a basic Müsli [`Context`].
        //!
        //! [`Context`]: musli::Context

        #[doc(inline)]
        pub use crate::context::*;
    }

    pub mod fixed {
        //! A container which can store up to a fixed number of uninitialized bytes on
        //! the stack and read into and from it.

        #[doc(inline)]
        pub use crate::fixed::FixedBytes;
    }

    pub mod options {
        //! Serialization options.

        #[doc(inline)]
        pub use crate::options::*;
    }

    pub mod buf {
        //! Utilitioes for working with buffers.

        #[doc(inline)]
        pub use crate::buf::*;
    }

    pub mod reader {
        //! Trait for governing how a particular source of bytes is read.
        //!
        //! `musli` requires all sources to reference the complete data being read from
        //! it which allows it to make the assumption the bytes are always returned with
        //! the `'de` lifetime.

        #[doc(inline)]
        pub use crate::reader::*;
    }

    pub mod wrap {
        //! Helpers for integrating musli with I/O types like [std::io] and
        //! [std::io::Write].
        //!
        //! The central function in this module is the [wrap] function which constructs
        //! an adapter around an I/O type to work with musli.

        #[doc(inline)]
        pub use crate::wrap::*;
    }

    pub mod writer {
        //! Trait for governing how a particular sink of bytes is written to.
        //!
        //! To adapt [std::io::Write] types, see the [wrap][crate::wrap::wrap] function.

        #[doc(inline)]
        pub use crate::writer::*;
    }

    #[doc(inline)]
    pub use musli_allocator as allocator;
}

#[doc(hidden)]
pub mod buffered_writer;

#[doc(hidden)]
pub mod int;

#[doc(hidden)]
pub mod str;

#[macro_use]
mod macros;

#[cfg_attr(feature = "std", path = "system/std.rs")]
#[cfg_attr(not(feature = "std"), path = "system/no_std.rs")]
mod system;
