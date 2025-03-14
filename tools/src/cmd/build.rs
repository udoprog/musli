use std::ffi::OsString;

use anyhow::{ensure, Result};
use clap::Parser;

use crate::{tests, Manifest, SharedArgs};

#[derive(Parser)]
pub(crate) struct Args {
    #[command(flatten)]
    shared: SharedArgs,
    remaining: Vec<OsString>,
}

pub(crate) fn entry(args: &Args, manifest: &Manifest) -> Result<()> {
    let mut ok = true;

    for report in manifest.reports(&args.shared) {
        let build = tests::build(report, "build", [], &args.remaining[..], true)?;
        ok |= build.report();
    }

    ensure!(ok, "Build failed");
    Ok(())
}
