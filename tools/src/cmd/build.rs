use std::ffi::OsString;

use anyhow::{bail, Result};
use clap::Parser;

use crate::{tests, Manifest, SharedArgs};

#[derive(Parser)]
pub(crate) struct Args {
    #[command(flatten)]
    shared: SharedArgs,
    remaining: Vec<OsString>,
}

pub(crate) fn entry(a: &Args, manifest: &Manifest) -> Result<()> {
    let mut builds = Vec::new();

    for report in manifest.reports(&a.shared) {
        builds.push(tests::build(report, "build", [], &a.remaining[..])?);
    }

    if builds.iter().any(|b| !b.status.success()) {
        for build in builds {
            for message in build.messages {
                print!("{message}")
            }
        }

        bail!("One or more commands failed")
    }

    Ok(())
}
