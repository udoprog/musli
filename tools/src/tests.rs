use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::{command, ReportRef};

pub(crate) struct CustomBuild<'a> {
    pub(crate) status: ExitStatus,
    pub(crate) messages: Vec<String>,
    report: ReportRef<'a>,
    all: Vec<(String, String, PathBuf)>,
    bad_features: Vec<(String, Features)>,
}

impl CustomBuild<'_> {
    pub(crate) fn report(&self) -> bool {
        let mut ok = true;

        if !self.bad_features.is_empty() {
            for (name, bad_features) in &self.bad_features {
                match bad_features {
                    Features::Expected(expected) => {
                        println!("{}: Expected `{name}`: {expected:?}", self.report.id)
                    }
                    Features::Unexpected(unexpected) => {
                        println!("{}: Unexpected `{name}`: {unexpected:?}", self.report.id)
                    }
                }
            }

            ok = false;
        }

        if !self.status.success() {
            println!(
                "{}: Build failed: {}",
                self.report.id,
                self.status.success()
            );

            for message in &self.messages {
                println!("{message}");
            }

            ok = false;
        }

        ok
    }

    /// Fetch a built binary that matches the given kind and name.
    pub(crate) fn bin(&self, kind: &str, name: &str) -> Option<PathBuf> {
        let mut last = None;

        for (k, n, path) in &self.all {
            if k == kind && n == name {
                last = Some(path.clone());
            }
        }

        last
    }
}

#[derive(Debug)]
enum Features {
    Expected(BTreeSet<String>),
    Unexpected(BTreeSet<String>),
}

/// Build tests.
pub(crate) fn build<'a>(
    report: ReportRef<'_>,
    command: impl AsRef<OsStr>,
    head: impl IntoIterator<Item = &'a str>,
    remaining: impl IntoIterator<Item: AsRef<OsStr>>,
    print: bool,
) -> Result<CustomBuild<'_>> {
    let mut child = command::cargo(
        report,
        command,
        head.into_iter().chain(["--message-format=json"]),
        remaining,
    )?;

    child.stdout(Stdio::piped());

    command::print(&child);

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

                if print {
                    println!("{}", message.message.rendered);
                }

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
                let mut used = BTreeSet::new();

                for original in &a.features {
                    let mut feature = original.as_str();

                    let expected = loop {
                        if feature.is_empty() || feature == "no" {
                            break None;
                        }

                        if expected.contains(feature) {
                            break Some(feature);
                        }

                        if !feature.starts_with("no-") {
                            break None;
                        }

                        let Some((f, _)) = feature.rsplit_once('-') else {
                            break None;
                        };

                        feature = f;
                    };

                    if let Some(expected) = expected {
                        used.insert(expected);
                    } else {
                        unexpected.insert(original.clone());
                    }
                }

                for used in used {
                    expected.remove(used);
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
        messages,
        report,
        all,
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
