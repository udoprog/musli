use std::ffi::OsString;

use anyhow::{ensure, Result};
use clap::Parser;

use crate::{command, Manifest, SharedArgs};

#[derive(Parser)]
pub(crate) struct Args {
    #[command(flatten)]
    shared: SharedArgs,
    remaining: Vec<OsString>,
}

pub(crate) fn entry(args: &Args, manifest: &Manifest) -> Result<()> {
    let mut ok = true;

    for report in manifest.reports(&args.shared) {
        let mut child = command::cargo(report, "bench", None::<OsString>, &args.remaining[..])?;
        command::print(&child);
        ok &= child.status()?.success();
    }

    ensure!(ok, "Bench failed");
    Ok(())
}
