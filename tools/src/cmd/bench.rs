use std::ffi::OsString;

use anyhow::{ensure, Result};
use clap::Parser;

use crate::{build_cargo, print_command, Manifest, SharedArgs};

#[derive(Parser)]
pub(crate) struct Args {
    #[command(flatten)]
    shared: SharedArgs,
    remaining: Vec<OsString>,
}

pub(crate) fn entry(a: &Args, manifest: &Manifest) -> Result<()> {
    let mut ok = true;

    for report in manifest.reports(&a.shared) {
        let mut child = build_cargo(report, "bench", None::<OsString>, &a.remaining[..])?;
        print_command(&child);
        ok &= child.status()?.success();
    }

    ensure!(ok, "Bench failed");
    Ok(())
}
