use std::ffi::OsString;

use anyhow::{ensure, Result};
use clap::Parser;

use crate::tests::{self};
use crate::{Manifest, SharedArgs};

#[derive(Parser)]
pub(crate) struct Args {
    #[command(flatten)]
    shared: SharedArgs,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    remaining: Vec<OsString>,
}

pub(crate) fn entry(args: &Args, manifest: &Manifest) -> Result<()> {
    let mut ok = true;

    for report in manifest.reports(&args.shared) {
        let build = tests::build(report, "run", [], &args.remaining[..], true)?;
        ok &= build.report();
    }

    ensure!(ok, "Run failed");
    Ok(())
}
