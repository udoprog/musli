use std::ffi::OsString;

use anyhow::{bail, Result};
use clap::Parser;

use crate::{build_cargo, print_command, Manifest, SharedArgs};

#[derive(Parser)]
pub(crate) struct Args {
    #[command(flatten)]
    shared: SharedArgs,
    remaining: Vec<OsString>,
}

pub(crate) fn entry(a: &Args, manifest: &Manifest) -> Result<()> {
    let mut builds = Vec::new();

    for report in manifest.reports(&a.shared) {
        let mut child = build_cargo(report, "bench", None::<OsString>, &a.remaining[..])?;

        print_command(&child);
        builds.push((report, child.status()?));
    }

    if builds.iter().any(|(_, status)| !status.success()) {
        bail!("One or more commands failed")
    }

    Ok(())
}
