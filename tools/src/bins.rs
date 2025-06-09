use std::cell::{Ref, RefCell};
use std::env::consts::EXE_SUFFIX;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, ensure, Context, Result};
use clap::Parser;

use crate::command;
use crate::manifest::ReportRef;
use crate::tests;

#[derive(Default, Parser)]
pub(crate) struct BinArgs {
    /// Build new and clean up test binaries after running.
    #[arg(long)]
    clean: bool,
    /// Build binaries in release mode.
    #[arg(long)]
    release: bool,
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
        args: &'a BinArgs,
    ) -> Result<Self> {
        Ok(Self {
            paths: Paths::new(output, &report.id),
            report,
            bins: InteriorBins {
                binaries: target.join("tools"),
                report,
                args,
                tests: RefCell::new(None),
                comparison: RefCell::new(None),
            },
        })
    }

    pub(crate) fn tests(&self) -> Result<Binary<'_>> {
        self.bins.build()?;

        let path = Ref::filter_map(self.bins.tests.borrow(), |f| f.as_deref())
            .ok()
            .context("Missing tests binary")?;

        Ok(Binary {
            path,
            report: self.report,
        })
    }

    pub(crate) fn comparison(&self) -> Result<Binary<'_>> {
        self.bins.build()?;

        let path = Ref::filter_map(self.bins.comparison.borrow(), |f| f.as_deref())
            .ok()
            .context("Missing comparison binary")?;

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

    /// Get the path to the binary.
    pub(crate) fn path(&self) -> &Path {
        &self.path
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

        command::print(&command);

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

struct InteriorBins<'a> {
    binaries: PathBuf,
    report: ReportRef<'a>,
    args: &'a BinArgs,
    tests: RefCell<Option<PathBuf>>,
    comparison: RefCell<Option<PathBuf>>,
}

impl InteriorBins<'_> {
    fn build(&self) -> Result<()> {
        fn rename(path: &Path, to: &Path) -> Result<()> {
            fs::rename(path, to).with_context(|| anyhow!("{} to {}", path.display(), to.display()))
        }

        if self.tests.borrow().is_some() && self.comparison.borrow().is_some() {
            return Ok(());
        }

        let binaries = self.binaries.join(if self.args.release {
            "release"
        } else {
            "debug"
        });

        let to_comparison = binaries.join(format!("comparison-{}{EXE_SUFFIX}", self.report.id));
        let to_tests = binaries.join(format!("tests-{}{EXE_SUFFIX}", self.report.id));

        if !binaries.is_dir() {
            fs::create_dir_all(&binaries).with_context(|| anyhow!("{}", binaries.display()))?;
        }

        if self.args.clean {
            for f in fs::read_dir(&binaries)? {
                let f = f?;
                _ = fs::remove_file(f.path());
            }
        }

        let head = self.args.release.then_some("--release");

        let build = tests::build(
            self.report,
            "build",
            head.into_iter().chain(["--benches"]),
            None::<OsString>,
            false,
        )?;

        ensure!(build.report(), "Build failed");

        let tests = build
            .bin("bin", "tests")
            .with_context(|| "missing tests binary")?;

        let comparison = build
            .bin("bench", "comparison")
            .context("missing comparison binary")?;

        rename(&comparison, &to_comparison)?;
        rename(&tests, &to_tests)?;

        println!("Tests: {}", to_tests.display());
        println!("Comparison: {}", to_comparison.display());

        *self.comparison.borrow_mut() = Some(to_comparison);
        *self.tests.borrow_mut() = Some(to_tests);
        Ok(())
    }
}

impl Drop for InteriorBins<'_> {
    fn drop(&mut self) {
        if self.args.clean {
            if let Some(tests) = self.tests.take() {
                _ = fs::remove_file(tests);
            }

            if let Some(comparison) = self.comparison.take() {
                _ = fs::remove_file(comparison);
            }
        }
    }
}
