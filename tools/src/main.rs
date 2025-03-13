#![allow(clippy::type_complexity)]

mod bins;
mod cmd;
mod manifest;
mod tests;

use std::env;
use std::ffi::OsStr;
use std::fmt::Write;
use std::fs::{self};
use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
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
        Cmd::Build(a) => {
            cmd::build::entry(&a, &manifest)?;
        }
        Cmd::Check(a) => {
            cmd::check::entry(&a, &manifest, &target, &output)?;
        }
    }

    Ok(())
}

fn build_cargo(
    report: ReportRef<'_>,
    command: impl AsRef<OsStr>,
    head: impl IntoIterator<Item: AsRef<OsStr>>,
    remaining: impl IntoIterator<Item: AsRef<OsStr>, IntoIter: ExactSizeIterator>,
) -> Result<Command> {
    let mut child = Command::new("cargo");

    child.arg(command).args(["-p", "tests"]).args(head);

    for env in &report.env {
        child.env(&env.key, &env.value);
    }

    let features = report.cargo_features();

    child.args(["--no-default-features", "--features", &features]);

    let remaining = remaining.into_iter();

    if remaining.len() > 0 {
        child.arg("--");
        child.args(remaining);
    }

    Ok(child)
}

fn print_command(child: &Command) {
    let program = child.get_program().to_string_lossy();

    let args = child
        .get_args()
        .map(|args| args.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");

    let mut e = String::new();

    if child.get_envs().next().is_some() {
        for (key, value) in child.get_envs() {
            if let Some(value) = value {
                _ = write!(e, "{}={} ", key.to_string_lossy(), value.to_string_lossy());
            }
        }
    }

    println!("{e}{program} {args}");
}
