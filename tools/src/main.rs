#![allow(clippy::type_complexity)]

mod bins;
mod cmd;
mod command;
mod manifest;
mod tests;

use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use manifest::ReportRef;

use crate::manifest::{Manifest, Report};

const REPO: &str = "https://raw.githubusercontent.com/udoprog/musli";

#[derive(Default, Parser)]
struct SharedArgs {
    /// Only run benchmarks for the given report.
    #[arg(short = 'r', long)]
    report: Option<String>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run all benchmarks and generate report.
    Report(cmd::report::Args),
    /// Run `cargo bench` with over all supported feature configurations.
    Bench(cmd::bench::Args),
    /// Run `cargo clippy` with over all supported feature configurations.
    Clippy(cmd::clippy::Args),
    /// Run the built test commands.
    Run(cmd::run::Args),
    /// Run `cargo build` with over all supported feature configurations.
    Build(cmd::build::Args),
    /// Perform a basic check.
    Check(cmd::check::Args),
}

impl Default for Cmd {
    #[inline]
    fn default() -> Self {
        Self::Report(cmd::report::Args::default())
    }
}

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Option<Cmd>,
}

fn main() -> Result<()> {
    let root =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").context("missing `CARGO_MANIFEST_DIR`")?);

    let root = root.parent().context("Missing root directory")?;

    let target = root.join("target");

    let mut args = Args::try_parse()?;

    let command = args.command.take().unwrap_or_default();

    let reports_path = root.join("tools").join("report.toml");
    let reports =
        fs::read_to_string(&reports_path).with_context(|| anyhow!("{}", reports_path.display()))?;
    let manifest: Manifest =
        toml::from_str(&reports).with_context(|| anyhow!("{}", reports_path.display()))?;

    let output = match &command {
        Cmd::Report(cmd::report::Args {
            output: Some(output),
            ..
        }) => output.to_owned(),
        _ => root.join("benchmarks"),
    };

    match command {
        Cmd::Report(a) => {
            cmd::report::entry(&a, &manifest, &target, &output)?;
        }
        Cmd::Bench(a) => {
            cmd::bench::entry(&a, &manifest)?;
        }
        Cmd::Clippy(a) => {
            cmd::clippy::entry(&a, &manifest)?;
        }
        Cmd::Run(a) => {
            cmd::run::entry(&a, &manifest)?;
        }
        Cmd::Build(a) => {
            cmd::build::entry(&a, &manifest)?;
        }
        Cmd::Check(a) => {
            cmd::check::entry(&a, &manifest, &target, &output)?;
        }
    }

    Ok(())
}
