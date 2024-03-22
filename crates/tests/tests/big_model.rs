//! When this file is built in release mode, it was discovered that it there is an overflow in requirements:
//!
//! ```sh
//! cargo build --release -p tests --test big_model --features test
//! ```
//!
//! ```text
//! error[E0275]: overflow evaluating the requirement `<<TE as Encoder<TC>>::EncodeVariant as VariantEncoder<TC>>::EncodeValue<'_>: Encoder<TC>`
//!  |
//!  = help: consider increasing the recursion limit by adding a `#![recursion_limit = "256"]` attribute to your crate (`big_model`)
//! ```

use std::ffi::OsString;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;

use musli::{Decode, Encode};

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

pub(crate) fn load_changes(buf: &[u8]) -> musli_descriptive::Result<Option<Vec<Change>>> {
    let mut system_buffer = musli_descriptive::allocator::SystemBuffer::new();
    let alloc = musli_descriptive::allocator::System::new(&mut system_buffer);
    let mut cx = musli_descriptive::context::SystemContext::new(&alloc);
    cx.include_type();

    let value: Vec<Change> = match musli_descriptive::encoding::DEFAULT.from_slice_with(&cx, buf) {
        Ok(value) => value,
        Err(error) => {
            if let Some(error) = cx.errors().next() {
                panic!("{}", error);
            }

            panic!("{error}")
        }
    };

    Ok(Some(value))
}

pub(crate) fn save_changes(changes: &Vec<Change>) -> musli_descriptive::Result<()> {
    let mut w = Vec::new();
    musli_descriptive::to_writer(&mut w, changes)?;
    Ok(())
}

#[test]
fn big_model() {
    let changes = Vec::new();
    save_changes(&changes).unwrap();

    let changes2 = load_changes(&[]).unwrap().unwrap();
    assert_eq!(changes.len(), changes2.len());
}
