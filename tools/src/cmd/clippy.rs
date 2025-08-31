use std::ffi::OsString;

use anyhow::{Result, ensure};
use clap::Parser;

use crate::tests::{self};
use crate::{Manifest, SharedArgs};

#[derive(Parser)]
pub(crate) struct Args {
    #[command(flatten)]
    shared: SharedArgs,
    remaining: Vec<OsString>,
}

pub(crate) fn entry(args: &Args, manifest: &Manifest) -> Result<()> {
    let mut ok = true;

    for report in manifest.reports(&args.shared) {
        let build = tests::build(report, "clippy", [], &args.remaining[..], true)?;
        ok &= build.report();
    }

    ensure!(ok, "Clippy failed");
    Ok(())
}
