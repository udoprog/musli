use std::cell::{Ref, RefCell};
use std::env::consts::EXE_SUFFIX;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, ensure, Context, Result};
use clap::Parser;

use crate::manifest::ReportRef;
use crate::print_command;
use crate::tests::{self, Features};

#[derive(Default, Parser)]
pub(crate) struct BinArgs {
    /// Build new and clean up test binaries after running.
    #[arg(long)]
    clean: bool,
    /// Don't build test binaries in release mode.
    #[arg(long)]
    no_release: bool,
}

/// Build binaries.
pub(crate) struct Bins<'a> {
    pub(crate) paths: Paths,
    pub(crate) report: ReportRef<'a>,
    bins: InteriorBins<'a>,
}

impl<'a> Bins<'a> {
    pub(crate) fn new(
        output: &'a Path,
        target: &'a Path,
        report: ReportRef<'a>,
        bins: &BinArgs,
    ) -> Result<Self> {
        Ok(Self {
            paths: Paths::new(output, &report.id),
            report,
            bins: InteriorBins {
                binaries: target.join("tools"),
                report,
                clean: bins.clean || bins.no_release,
                no_release: bins.no_release,
                fuzz: RefCell::new(None),
                comparison: RefCell::new(None),
            },
        })
    }

    pub(crate) fn fuzz(&self) -> Result<Binary<'_>> {
        self.bins.build()?;

        let path = Ref::filter_map(self.bins.fuzz.borrow(), |f| f.as_deref())
            .ok()
            .context("Missing `fuzz` binary")?;

        Ok(Binary {
            path,
            report: self.report,
        })
    }

    pub(crate) fn comparison(&self) -> Result<Binary<'_>> {
        self.bins.build()?;

        let path = Ref::filter_map(self.bins.comparison.borrow(), |f| f.as_deref())
            .ok()
            .context("Missing `comparison` binary")?;

        Ok(Binary {
            path,
            report: self.report,
        })
    }
}

pub(crate) struct Binary<'a> {
    path: Ref<'a, Path>,
    report: ReportRef<'a>,
}

impl Binary<'_> {
    /// Construt a command with a preconfigured environment.
    pub(crate) fn command(&self) -> Command {
        let mut command = Command::new(&*self.path);

        for env in &self.report.env {
            command.env(&env.key, &env.value);
        }

        command
    }

    /// Run the given binary with the specified set of arguments.
    pub(crate) fn run(&self, args: &[&str], env: &[(&OsStr, &OsStr)]) -> Result<()> {
        let mut command = self.command();

        for arg in args {
            command.arg(arg);
        }

        for (key, value) in env {
            command.env(*key, *value);
        }

        print_command(&command);

        let status = command.status()?;

        ensure!(status.success(), "Command failed: {status}");
        Ok(())
    }
}

pub(crate) struct Paths {
    pub(crate) criterion_output: PathBuf,
    pub(crate) images: PathBuf,
}

impl Paths {
    fn new(output: &Path, id: &str) -> Self {
        let images = output.join("images");
        let criterion_output = output.join(format!("criterion-{id}"));

        Self {
            criterion_output,
            images,
        }
    }
}

struct Build {
    fuzz: PathBuf,
    comparison: PathBuf,
}

struct InteriorBins<'a> {
    binaries: PathBuf,
    report: ReportRef<'a>,
    clean: bool,
    no_release: bool,
    fuzz: RefCell<Option<PathBuf>>,
    comparison: RefCell<Option<PathBuf>>,
}

impl InteriorBins<'_> {
    fn build(&self) -> Result<()> {
        fn shuffle(path: &mut PathBuf, to: &Path) -> Result<()> {
            fs::rename(&*path, to)
                .with_context(|| anyhow!("{} to {}", path.display(), to.display()))?;
            to.clone_into(path);
            Ok(())
        }

        if self.fuzz.borrow().is_some() && self.comparison.borrow().is_some() {
            return Ok(());
        }

        if !self.binaries.is_dir() {
            fs::create_dir_all(&self.binaries)
                .with_context(|| anyhow!("{}", self.binaries.display()))?;
        }

        let to_comparison = self
            .binaries
            .join(format!("comparison-{}{}", self.report.id, EXE_SUFFIX));

        let to_fuzz = self
            .binaries
            .join(format!("fuzz-{}{}", self.report.id, EXE_SUFFIX));

        let rebuild = self.clean || self.no_release;

        if rebuild || !(to_comparison.is_file() && to_fuzz.is_file()) {
            let mut built =
                build_commands(!self.no_release, self.report).context("Building bench binaries")?;

            shuffle(&mut built.comparison, &to_comparison)?;
            shuffle(&mut built.fuzz, &to_fuzz)?;
        }

        println!("Comparison: {}", to_comparison.display());
        println!("Fuzz: {}", to_fuzz.display());

        *self.comparison.borrow_mut() = Some(to_comparison);
        *self.fuzz.borrow_mut() = Some(to_fuzz);
        Ok(())
    }
}

impl Drop for InteriorBins<'_> {
    fn drop(&mut self) {
        if self.clean {
            if let Some(fuzz) = self.fuzz.take() {
                _ = fs::remove_file(fuzz);
            }

            if let Some(comparison) = self.comparison.take() {
                _ = fs::remove_file(comparison);
            }
        }
    }
}

/// Build commands.
fn build_commands(release: bool, report: ReportRef<'_>) -> Result<Build> {
    let head = release.then_some("--release");

    let build = tests::build(
        report,
        "build",
        head.into_iter().chain(["--benches"]),
        None::<OsString>,
    )?;

    if !build.bad_features.is_empty() {
        for (name, bad_features) in build.bad_features {
            match bad_features {
                Features::Expected(expected) => {
                    println!("{}: Expected `{name}`: {expected:?}", report.id)
                }
                Features::Unexpected(unexpected) => {
                    println!("{}: Unexpected `{name}`: {unexpected:?}", report.id)
                }
            }
        }

        bail!("{}: Got bad features during build", report.id);
    }

    if !build.status.success() {
        for message in build.messages {
            print!("{message}");
        }

        bail!("Command failed: {}", build.status.success());
    }

    let fuzz = build
        .bin("bin", "fuzz")
        .with_context(|| anyhow!("missing fuzz in {build:?}"))?;
    let comparison = build
        .bin("bench", "comparison")
        .context("missing comparison")?;
    Ok(Build { fuzz, comparison })
}
