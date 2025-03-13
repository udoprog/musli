use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::{build_cargo, print_command, ReportRef};

#[derive(Default, Debug)]
pub(crate) struct CustomBuild {
    pub(crate) status: ExitStatus,
    all: Vec<(String, String, PathBuf)>,
    pub(crate) messages: Vec<String>,
    pub(crate) bad_features: Vec<(String, Features)>,
}

impl CustomBuild {
    /// Fetch a built binary that matches the given kind and name.
    pub(crate) fn bin(&self, kind: &str, name: &str) -> Option<PathBuf> {
        let mut bins = Vec::new();

        for (k, n, path) in &self.all {
            if k == kind && n == name {
                bins.push(path.clone());
            }
        }

        bins.pop()
    }
}

#[derive(Debug)]
pub(crate) enum Features {
    Expected(BTreeSet<String>),
    Unexpected(BTreeSet<String>),
}

/// Build tests.
pub(crate) fn build<'a>(
    report: ReportRef<'_>,
    command: impl AsRef<OsStr>,
    head: impl IntoIterator<Item = &'a str>,
    remaining: impl IntoIterator<Item: AsRef<OsStr>, IntoIter: ExactSizeIterator>,
) -> Result<CustomBuild> {
    let mut child = build_cargo(
        report,
        command,
        head.into_iter().chain(["--message-format=json"]),
        remaining,
    )?;

    child.stdout(Stdio::piped());

    print_command(&child);

    let mut child = child.spawn()?;

    let stdout = child.stdout.take().context("missing stdout")?;
    let stdout = BufReader::new(stdout);

    let mut all = Vec::new();
    let mut messages = Vec::new();
    let mut bad_features = Vec::new();

    for line in stdout.lines() {
        let line = line?;
        let line: Line = serde_json::from_str(&line)?;

        match line.reason.as_str() {
            "compiler-message" => {
                let message: CompilerMessage = serde_json::from_value(line.extra)?;
                messages.push(message.message.rendered);
            }
            "compiler-artifact" => {
                let a: CompilerArtifact = serde_json::from_value(line.extra.clone())?;

                let Some((_, last)) = a.package_id.rsplit_once('/') else {
                    continue;
                };

                let Some((package_name, _)) = last.rsplit_once('#') else {
                    continue;
                };

                if package_name != "tests" {
                    continue;
                }

                let mut expected = report.expected_features();

                let mut unexpected = BTreeSet::new();

                for feature in &a.features {
                    if !expected.remove(feature.as_str()) {
                        unexpected.insert(feature.clone());
                    }
                }

                if !expected.is_empty() {
                    bad_features.push((
                        a.target.name.clone(),
                        Features::Expected(expected.into_iter().map(str::to_owned).collect()),
                    ));
                }

                if !unexpected.is_empty() {
                    bad_features.push((a.target.name.clone(), Features::Unexpected(unexpected)));
                }

                if let (Some(kind), Some(executable)) = (
                    a.target.kind.first().map(|s| s.as_str()),
                    a.executable.as_deref(),
                ) {
                    if kind == "bin" && a.profile.test {
                        continue;
                    }

                    all.push((kind.to_owned(), a.target.name, PathBuf::from(executable)));
                }
            }
            _ => {}
        }
    }

    let status = child.wait()?;

    Ok(CustomBuild {
        status,
        all,
        messages,
        bad_features,
    })
}

#[derive(Deserialize)]
struct Line {
    reason: String,
    #[serde(flatten)]
    extra: serde_json::Value,
}

#[derive(Deserialize)]
struct Target {
    kind: Vec<String>,
    name: String,
}

#[derive(Deserialize)]
struct Profile {
    test: bool,
}

#[derive(Deserialize)]
struct CompilerArtifact {
    package_id: String,
    executable: Option<String>,
    features: Vec<String>,
    target: Target,
    profile: Profile,
}

#[derive(Deserialize)]
struct Message {
    rendered: String,
}

#[derive(Deserialize)]
struct CompilerMessage {
    message: Message,
}
