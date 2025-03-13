use std::ffi::OsString;

use anyhow::{bail, Result};
use clap::Parser;

use crate::tests::{self, Features};
use crate::{Manifest, SharedArgs};

#[derive(Parser)]
pub(crate) struct Args {
    #[command(flatten)]
    shared: SharedArgs,
    remaining: Vec<OsString>,
}

pub(crate) fn entry(a: &Args, manifest: &Manifest) -> Result<()> {
    let mut builds = Vec::new();

    for report in manifest.reports(&a.shared) {
        let build = tests::build(report, "clippy", [], &a.remaining[..])?;
        builds.push((report, build));
    }

    if builds
        .iter()
        .any(|(_, b)| !b.status.success() || !b.bad_features.is_empty())
    {
        for (report, b) in builds {
            if !b.bad_features.is_empty() {
                for (name, bad_features) in b.bad_features {
                    match bad_features {
                        Features::Expected(expected) => {
                            println!("{}: Expected `{name}`: {expected:?}", report.id)
                        }
                        Features::Unexpected(unexpected) => {
                            println!("{}: Unexpected `{name}`: {unexpected:?}", report.id)
                        }
                    }
                }
            }

            for message in b.messages {
                print!("{message}")
            }
        }

        bail!("One or more commands failed")
    }

    Ok(())
}
