use core::iter;

use std::collections::{BTreeSet, HashMap};
use std::ops::Deref;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

use crate::bins::{BinArgs, Bins};
use crate::SharedArgs;

#[derive(Debug, Deserialize)]
pub(crate) struct Manifest {
    #[serde(default)]
    pub(crate) header: Vec<String>,
    #[serde(default)]
    pub(crate) common: Vec<String>,
    pub(crate) url: String,
    pub(crate) branch: String,
    #[serde(default)]
    pub(crate) kinds: Vec<Kind>,
    #[serde(default)]
    pub(crate) groups: Vec<Group>,
    #[serde(default)]
    pub(crate) reports: Vec<Report>,
    #[serde(default)]
    pub(crate) crate_footnotes: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub(crate) size_footnotes: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub(crate) footnotes: HashMap<String, String>,
    #[serde(default)]
    pub(crate) links: Vec<Link>,
    #[serde(default)]
    pub(crate) missing_features: HashMap<String, String>,
}

impl Manifest {
    /// Collect buildable binaries.
    pub(crate) fn bins<'a>(
        &'a self,
        target: &'a Path,
        output: &'a Path,
        shared: &SharedArgs,
        bins: &'a BinArgs,
    ) -> Result<Vec<Bins<'a>>> {
        let mut out = Vec::new();

        for report in self.reports(shared) {
            out.push(Bins::new(output, target, report, bins)?);
        }

        Ok(out)
    }

    /// Iterate over reports.
    pub(crate) fn reports<'a: 'this, 'this>(
        &'a self,
        args: &'this SharedArgs,
    ) -> impl Iterator<Item = ReportRef<'a>> + 'this {
        let mut it = self.reports.iter();

        iter::from_fn(move || loop {
            let report = it.next()?;

            if report.skip {
                continue;
            }

            if let Some(id) = args.report.as_deref() {
                if id != report.id {
                    continue;
                }
            }

            return Some(ReportRef {
                manifest: self,
                report,
            });
        })
    }
}

/// A reference to a report.
#[derive(Clone, Copy)]
pub(crate) struct ReportRef<'a> {
    pub(crate) manifest: &'a Manifest,
    report: &'a Report,
}

impl ReportRef<'_> {
    /// Get the cargo features string to build this report.
    pub(crate) fn cargo_features(&self) -> String {
        self.manifest
            .common
            .iter()
            .chain(self.report.features.iter())
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Get a collection of expected features for this report.
    ///
    /// These are the features that *should* be reported in a build.
    pub(crate) fn expected_features(&self) -> BTreeSet<&str> {
        self.report
            .features
            .iter()
            .chain(&self.report.expected)
            .chain(&self.manifest.common)
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
    }
}

impl Deref for ReportRef<'_> {
    type Target = Report;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.report
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Link {
    pub(crate) title: String,
    pub(crate) href: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Group {
    pub(crate) id: String,
    pub(crate) description: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Report {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) description: Vec<String>,
    pub(crate) title: String,
    pub(crate) link: String,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    pub(crate) expected: Vec<String>,
    #[serde(default)]
    pub(crate) only: Vec<String>,
    #[serde(default)]
    pub(crate) env: Vec<ReportEnv>,
    #[serde(default)]
    skip: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReportEnv {
    pub(crate) key: String,
    pub(crate) value: String,
    pub(crate) description: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Kind {
    pub(crate) id: String,
    pub(crate) description: String,
}
