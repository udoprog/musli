use std::ffi::OsString;
use std::path::Path;

use anyhow::{ensure, Context, Result};
use clap::Parser;

use crate::bins::BinArgs;
use crate::tests;
use crate::{Manifest, SharedArgs};

#[derive(Parser)]
pub(crate) struct Args {
    #[command(flatten)]
    shared: SharedArgs,
    #[command(flatten)]
    bin: BinArgs,
    remaining: Vec<OsString>,
}

pub(crate) fn entry(a: &Args, manifest: &Manifest, target: &Path, output: &Path) -> Result<()> {
    let bins = manifest.bins(target, output, &a.shared, &a.bin)?;

    for b in &bins {
        println!("Sanity checking: {}", b.report.title);

        b.fuzz()?
            .run(&["--iter", "1"], &[])
            .context("Fuzz check failed")?;
        // Test benches binaries.
        b.comparison()?.run(&[], &[])?;
    }

    let mut ok = true;

    for report in manifest.reports(&a.shared) {
        let build = tests::build(report, "build", [], &a.remaining[..], true)?;
        ok |= build.report();
    }

    ensure!(ok, "Check failed");
    Ok(())
}
