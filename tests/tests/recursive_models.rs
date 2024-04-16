//! When this file is built in release mode, it was discovered that it there is an overflow in requirements:
//!
//! ```sh
//! cargo build --release -p tests --test recursive_models --features test
//! ```
//!
//! ```text
//! error[E0275]: overflow evaluating the requirement `<<TE as Encoder<TC>>::EncodeVariant as VariantEncoder<TC>>::EncodeData<'_>: Encoder<TC>`
//!  |
//!  = help: consider increasing the recursion limit by adding a `#![recursion_limit = "256"]` attribute to your crate (`recursive_models`)
//! ```

use musli::de::DecodeOwned;
use musli::{Decode, Encode};

pub(crate) fn encode<T>(changes: &T) -> tests::storage::Result<Vec<u8>>
where
    T: Encode,
{
    tests::storage::to_vec(changes)
}

pub(crate) fn decode<T>(buf: &[u8]) -> tests::storage::Result<T>
where
    T: DecodeOwned,
{
    tests::storage::from_slice(buf)
}

#[test]
fn big_model() {
    use std::ffi::OsString;
    use std::ops::Range;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[derive(Encode, Decode)]
    #[non_exhaustive]
    pub(crate) struct RustVersion {
        pub(crate) major: u64,
        pub(crate) minor: u64,
        pub(crate) patch: u64,
    }

    #[derive(Encode, Decode)]
    pub(crate) enum CargoKey {}

    #[derive(Encode, Decode)]
    pub(crate) enum CargoIssue {}

    #[derive(Encode, Decode)]
    pub(crate) enum WorkflowError {
        Error { name: String, reason: String },
    }

    #[derive(Encode, Decode)]
    pub(crate) enum EditChange {
        Insert {
            reason: String,
            key: String,
            value: Value,
        },
        Set {
            reason: String,
            value: Value,
        },
        RemoveKey {
            reason: String,
            key: String,
        },
    }

    #[derive(Encode, Decode)]
    pub(crate) enum Value {
        String(String),
        Array(Vec<Value>),
        Mapping(Vec<(String, Value)>),
    }

    #[derive(Encode, Decode)]
    pub(crate) struct Edits {
        changes: Vec<EditChange>,
    }

    #[derive(Encode, Decode)]
    pub(crate) struct File {
        data: String,
        line_starts: Vec<usize>,
    }

    #[derive(Encode, Decode)]
    pub(crate) struct RepoRef {}

    #[derive(Encode, Decode)]
    pub(crate) struct Manifest {}

    #[derive(Encode, Decode)]
    pub(crate) struct Replaced {
        path: PathBuf,
        content: Vec<u8>,
        replacement: Box<str>,
        ranges: Vec<Range<usize>>,
    }

    #[derive(Encode, Decode)]
    pub(crate) enum Change {
        MissingWorkflow {
            id: String,
            repo: RepoRef,
        },
        BadWorkflow {
            edits: Edits,
            errors: Vec<WorkflowError>,
        },
        UpdateLib {
            lib: Arc<File>,
        },
        UpdateReadme {
            readme: Arc<File>,
        },
        CargoTomlIssues {
            cargo: Option<Manifest>,
            issues: Vec<CargoIssue>,
        },
        SetRustVersion {
            repo: RepoRef,
            version: RustVersion,
        },
        RemoveRustVersion {
            repo: RepoRef,
            version: RustVersion,
        },
        SavePackage {
            manifest: Manifest,
        },
        Replace {
            replaced: Replaced,
        },
        ReleaseCommit {},
        Publish {
            name: String,
            dry_run: bool,
            no_verify: bool,
            args: Vec<OsString>,
        },
    }

    let changes = Vec::<Change>::new();
    encode(&changes).unwrap();
    assert!(decode::<Vec<Change>>(&[]).is_err());
}

#[test]
fn recursive_model() {
    #[derive(Encode, Decode)]
    pub(crate) struct Value {
        recursive: Vec<Value>,
    }

    #[derive(Encode, Decode)]
    pub(crate) struct Model {
        value: Value,
    }

    let model = Model {
        value: Value {
            recursive: Vec::new(),
        },
    };

    encode(&model).unwrap();
    assert!(decode::<Model>(&[]).is_err());
}
