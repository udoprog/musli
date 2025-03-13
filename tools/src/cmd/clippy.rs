use std::ffi::OsString;

use anyhow::{bail, Result};
use clap::Parser;

use crate::tests::{self};
use crate::{Manifest, SharedArgs};

#[derive(Parser)]
pub(crate) struct Args {
    #[command(flatten)]
    shared: SharedArgs,
    remaining: Vec<OsString>,
}

pub(crate) fn entry(a: &Args, manifest: &Manifest) -> Result<()> {
    let mut ok = true;

    for report in manifest.reports(&a.shared) {
        let build = tests::build(report, "clippy", [], &a.remaining[..], true)?;
        ok &= build.report();
    }

    if !ok {
        bail!("One or more commands failed")
    }

    Ok(())
}
