use std::cell::{Ref, RefCell};
use std::collections::{BTreeSet, HashMap};
use std::env;
use std::env::consts::EXE_SUFFIX;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fmt::Write;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{anyhow, bail, ensure, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

struct Paths {
    criterion_output: PathBuf,
    images: PathBuf,
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

const REPO: &str = "https://raw.githubusercontent.com/udoprog/musli";

#[derive(Debug, Deserialize)]
struct Link {
    title: String,
    href: String,
}

#[derive(Debug, Deserialize)]
struct Group {
    id: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct Kind {
    id: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    #[serde(default)]
    header: Vec<String>,
    #[serde(default)]
    common: Vec<String>,
    url: String,
    branch: String,
    #[serde(default)]
    kinds: Vec<Kind>,
    #[serde(default)]
    groups: Vec<Group>,
    #[serde(default)]
    reports: Vec<Report>,
    #[serde(default)]
    crate_footnotes: HashMap<String, Vec<String>>,
    #[serde(default)]
    size_footnotes: HashMap<String, Vec<String>>,
    #[serde(default)]
    footnotes: HashMap<String, String>,
    #[serde(default)]
    links: Vec<Link>,
    #[serde(default)]
    missing_features: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct Report {
    id: String,
    #[serde(default)]
    skip: bool,
    #[serde(default)]
    description: Vec<String>,
    title: String,
    link: String,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    expected: Vec<String>,
    #[serde(default)]
    only: Vec<String>,
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

#[derive(Default, Parser)]
struct BinArgs {
    /// Build new and clean up test binaries after running.
    #[arg(long)]
    clean: bool,
    /// Don't build test binaries in release mode.
    #[arg(long)]
    no_release: bool,
}

#[derive(Default, Parser)]
struct ArgsReport {
    /// The output directory to write results into.
    #[arg(long)]
    output: Option<PathBuf>,
    /// Custom path to write index output.
    #[arg(long)]
    index_output: Option<PathBuf>,
    /// Filter to pass to benchmarks when running them.
    #[arg(short = 'f', long)]
    filter: Option<String>,
    /// Run benchmarks.
    #[arg(long)]
    bench: bool,
    /// Run `--quick` benchmarks.
    #[arg(long)]
    quick: bool,
    /// Skip size comparisons.
    #[arg(long)]
    no_size: bool,
    /// Reference graphics from the given branch.
    #[arg(long)]
    branch: Option<String>,
    #[command(flatten)]
    bins: BinArgs,
}

#[derive(Parser)]
struct ArgsClippy {
    remaining: Vec<OsString>,
}

#[derive(Parser)]
struct ArgsBuild {
    remaining: Vec<OsString>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run all benchmarks and generate report.
    Report(ArgsReport),
    /// Run `cargo clippy` with over all supported feature configurations.
    Clippy(ArgsClippy),
    /// Run `cargo build` with over all supported feature configurations.
    Build(ArgsBuild),
    /// Perform a basic check.
    Check(BinArgs),
}

impl Default for Cmd {
    #[inline]
    fn default() -> Self {
        Self::Report(ArgsReport::default())
    }
}

#[derive(Parser)]
struct Args {
    /// Only run benchmarks for the given report.
    #[arg(short = 'r', long)]
    report: Option<String>,
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
        Cmd::Report(ArgsReport {
            output: Some(output),
            ..
        }) => output.to_owned(),
        _ => root.join("benchmarks"),
    };

    match command {
        Cmd::Report(a) => {
            let bins = build_bins(&target, &output, &args, &manifest, &a.bins)?;

            for b in &bins {
                println!("Sanity checking: {}", b.report.title);

                sanity_check(&b.bins.fuzz()?).context("Sanity check failed")?;
                // Test benches binaries.
                run_path(&b.bins.comparison()?, &[], &[])?;
            }

            let branch = a.branch.as_deref().unwrap_or(manifest.branch.as_str());

            let mut built_reports = Vec::new();
            let mut size_sets = Vec::new();

            let mut errored = Vec::new();

            for bins in &bins {
                println!("Building: {}", bins.report.title);

                match build_report(&a, bins, a.bench, a.filter.as_deref()) {
                    Ok(group_plots) => {
                        built_reports.push((bins, group_plots));
                    }
                    Err(error) => {
                        errored.push((bins.report, error));
                    }
                }
            }

            if !errored.is_empty() {
                for (report, error) in &errored {
                    println!("Failed to build report: {}", report.title);

                    for error in error.chain() {
                        println!("Caused by: {error}");
                    }
                }

                bail!("{} builds failed", errored.len());
            }

            if !a.no_size {
                for bins in &bins {
                    println!("Sizing: {}", bins.report.title);
                    let size_set =
                        collect_size_sets(&bins.bins.fuzz()?).context("Collecting size sets")?;
                    size_sets.push((bins.report, size_set));
                }
            }

            let mut used_footnotes = BTreeSet::new();

            let mut o = String::new();

            writeln!(o, "# Benchmarks and size comparisons")?;
            writeln!(o)?;

            for line in &manifest.header {
                writeln!(o, "> {line}")?;
            }

            writeln!(o)?;
            writeln!(o, "Identifiers which are used in tests:")?;
            writeln!(o)?;

            for Kind {
                id, description, ..
            } in &manifest.kinds
            {
                writeln!(o, "- `{id}` - {description}")?;
            }

            for Group {
                id, description, ..
            } in &manifest.groups
            {
                writeln!(o, "- `{id}` - {description}")?;
            }

            writeln!(o)?;
            writeln!(o, "The following are one section for each kind of benchmark we perform. They range from \"Full features\" to more specialized ones like zerocopy comparisons.")?;

            for (bins, _) in &built_reports {
                let Report {
                    id, title, link, ..
                } = bins.report;

                writeln!(
                    o,
                    "- [**{title}**](#{link}) ([Report 📓]({url}/criterion-{id}/report/), [Sizes](#{link}-sizes))",
                    url = manifest.url
                )?;
            }

            writeln!(o)?;

            writeln!(
                o,
                "Below you'll also find [size comparisons](#size-comparisons)."
            )?;

            for (bins @ Bins { report, .. }, group_plots) in &built_reports {
                writeln!(o, "### {}", report.title)?;
                render_preamble(&mut o, &manifest, report)?;

                writeln!(o, "**More:**")?;
                writeln!(o)?;
                writeln!(
                    o,
                    "* [Report 📓]({url}/criterion-{id}/report/)",
                    url = manifest.url,
                    id = report.id
                )?;
                writeln!(o, "* [Sizes](#{link}-sizes)", link = report.link)?;
                writeln!(o)?;

                for (group, plots) in group_plots {
                    for kind in &manifest.kinds {
                        let outcome = bins
                            .paths
                            .criterion_output
                            .join(format!("{}_{}", kind.id, group.id));

                        writeln!(o, "<table>")?;
                        writeln!(o, "<tr>")?;
                        writeln!(o, "<th colspan=\"3\">")?;
                        writeln!(o, "<code>{}/{}/{}</code>", report.id, kind.id, group.id)?;
                        writeln!(o, "<br />")?;
                        writeln!(
                            o,
                            "<a href=\"{url}/criterion-{id}/{kind}_{group}/report/\">Report 📓</a>",
                            url = manifest.url,
                            id = report.id,
                            kind = kind.id,
                            group = group.id,
                        )?;
                        writeln!(o, "</th>")?;
                        writeln!(o, "</tr>")?;

                        if let Some(plot) = plots.get(kind.id.as_str()) {
                            writeln!(o, "<tr>")?;
                            let url = format!("{REPO}/{branch}/benchmarks/images/{plot}");

                            writeln!(o, "<td colspan=\"3\">")?;
                            write!(o, "<a href=\"{url}\">")?;
                            write!(o, "<img style=\"background-color: white;\" src=\"{url}\">")?;
                            write!(o, "</a>")?;
                            writeln!(o, "</td>")?;

                            writeln!(o, "</tr>")?;
                        }

                        writeln!(o, "</table>")?;

                        writeln!(o)?;
                        writeln!(o, "| Group | Mean | Interval | Link |")?;
                        writeln!(o, "|-|-|-|-|")?;

                        let mut estimates = Vec::new();

                        for e in fs::read_dir(&outcome)? {
                            let e = e?;
                            let p = e.path();

                            let Some(file_name) = p.file_name().and_then(|f| f.to_str()) else {
                                continue;
                            };

                            if file_name == "report" {
                                continue;
                            }

                            let bytes = fs::read(p.join("new").join("estimates.json"))?;
                            let data: Estimates = serde_json::from_slice(&bytes)?;
                            estimates.push((file_name.to_owned(), data));
                        }

                        estimates.sort_by(|a, b| a.0.cmp(&b.0));

                        for (file_name, data) in estimates {
                            let mean = &data.mean;
                            let interval = &mean.confidence_interval;

                            write!(o, "| `{}/{}/{file_name}`", kind.id, group.id)?;

                            if let Some(footnotes) = manifest.crate_footnotes.get(&file_name) {
                                used_footnotes.extend(footnotes);

                                for footnote in footnotes {
                                    write!(o, "[^{footnote}]")?;
                                }
                            }

                            write!(
                                o,
                                " | **{}** ± {} | {} &mdash; {}",
                                timing(mean.point_estimate),
                                timing(mean.standard_error),
                                timing(interval.lower_bound),
                                timing(interval.upper_bound),
                            )?;

                            writeln!(
                                o,
                                " | [Report 📓]({url}/criterion-{id}/{kind}_{group}/{file_name}/report/) |",
                                url = manifest.url,
                                id = report.id,
                                kind = kind.id,
                                group = group.id,
                            )?;
                        }

                        writeln!(o)?;
                    }

                    writeln!(o)?;
                }

                writeln!(o)?;
            }

            size_comparisons(&mut o, &manifest, size_sets, &mut used_footnotes)?;

            if !used_footnotes.is_empty() {
                writeln!(o)?;

                for footnote in used_footnotes {
                    let Some(note) = manifest.footnotes.get(footnote) else {
                        continue;
                    };

                    writeln!(o, "[^{footnote}]: {note}")?;
                }
            }

            for Link { title, href } in &manifest.links {
                writeln!(o, "[{title}]: {href}")?;
            }

            let report = match &a.index_output {
                Some(report) => report.clone(),
                None => output.join("index.md"),
            };

            println!("Writing: {}", report.display());
            fs::write(report, o.as_bytes())?;
        }
        Cmd::Clippy(a) => {
            let mut builds = Vec::new();

            for report in &manifest.reports {
                if report.skip {
                    continue;
                }

                if let Some(do_report) = args.report.as_deref() {
                    if do_report != report.id {
                        continue;
                    }
                }

                let build = build_tests(
                    &manifest,
                    &report.features,
                    &report.expected,
                    "clippy",
                    None::<OsString>,
                    &a.remaining[..],
                )?;

                builds.push((report, build));
            }

            if builds
                .iter()
                .any(|(_, b)| !b.status.success() || !b.bad_features.is_empty())
            {
                for (report, b) in builds {
                    if !b.bad_features.is_empty() {
                        for (name, bad_features) in b.bad_features {
                            match bad_features {
                                Features::Expected(expected) => {
                                    println!("{}: Expected `{name}`: {expected:?}", report.id)
                                }
                                Features::Unexpected(unexpected) => {
                                    println!("{}: Unexpected `{name}`: {unexpected:?}", report.id)
                                }
                            }
                        }
                    }

                    for message in b.messages {
                        print!("{message}")
                    }
                }

                bail!("One or more commands failed")
            }
        }
        Cmd::Build(a) => {
            let mut builds = Vec::new();

            for report in &manifest.reports {
                if report.skip {
                    continue;
                }

                if let Some(do_report) = args.report.as_deref() {
                    if do_report != report.id {
                        continue;
                    }
                }

                let build = build_tests(
                    &manifest,
                    &report.features,
                    &report.expected,
                    "build",
                    None::<OsString>,
                    &a.remaining[..],
                )?;

                builds.push(build);
            }

            if builds.iter().any(|b| !b.status.success()) {
                for build in builds {
                    for message in build.messages {
                        print!("{message}")
                    }
                }

                bail!("One or more commands failed")
            }
        }
        Cmd::Check(a) => {
            let bins = build_bins(&target, &output, &args, &manifest, &a)?;

            for b in &bins {
                println!("Sanity checking: {}", b.report.title);

                sanity_check(&b.bins.fuzz()?).context("Sanity check failed")?;
                // Test benches binaries.
                run_path(&b.bins.comparison()?, &[], &[])?;
            }
        }
    }

    Ok(())
}

fn build_bins<'a>(
    target: &'a Path,
    output: &'a Path,
    args: &Args,
    manifest: &'a Manifest,
    bins: &BinArgs,
) -> Result<Vec<Bins<'a>>> {
    let mut out = Vec::new();

    for report in &manifest.reports {
        if report.skip {
            continue;
        }

        if let Some(do_report) = args.report.as_deref() {
            if do_report != report.id {
                continue;
            }
        }

        out.push(Bins::new(output, target, manifest, report, bins)?);
    }

    Ok(out)
}

struct InteriorBins<'a> {
    binaries: PathBuf,
    report: &'a Report,
    manifest: &'a Manifest,
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
            let mut built = build_bench(self.manifest, !self.no_release, self.report)
                .context("Building bench binaries")?;

            shuffle(&mut built.comparison, &to_comparison)?;
            shuffle(&mut built.fuzz, &to_fuzz)?;
        }

        println!("Comparison: {}", to_comparison.display());
        println!("Fuzz: {}", to_fuzz.display());

        *self.comparison.borrow_mut() = Some(to_comparison);
        *self.fuzz.borrow_mut() = Some(to_fuzz);
        Ok(())
    }

    fn fuzz(&self) -> Result<Ref<'_, Path>> {
        self.build()?;
        Ref::filter_map(self.fuzz.borrow(), |f| f.as_deref())
            .ok()
            .context("Missing `fuzz` binary")
    }

    fn comparison(&self) -> Result<Ref<'_, Path>> {
        self.build()?;
        Ref::filter_map(self.comparison.borrow(), |f| f.as_deref())
            .ok()
            .context("Missing `comparison` binary")
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

struct Bins<'a> {
    report: &'a Report,
    manifest: &'a Manifest,
    bins: InteriorBins<'a>,
    paths: Paths,
}

impl<'a> Bins<'a> {
    fn new(
        output: &'a Path,
        target: &'a Path,
        manifest: &'a Manifest,
        report: &'a Report,
        bins: &BinArgs,
    ) -> Result<Self> {
        Ok(Self {
            report,
            manifest,
            bins: InteriorBins {
                binaries: target.join("tools"),
                report,
                manifest,
                clean: bins.clean || bins.no_release,
                no_release: bins.no_release,
                fuzz: RefCell::new(None),
                comparison: RefCell::new(None),
            },
            paths: Paths::new(output, &report.id),
        })
    }
}

fn build_report<'a>(
    a: &ArgsReport,
    bins: &Bins<'a>,
    run_bench: bool,
    filter: Option<&str>,
) -> Result<Vec<(&'a Group, HashMap<&'a str, String>)>> {
    if !bins.paths.images.is_dir() {
        fs::create_dir_all(&bins.paths.images)
            .with_context(|| anyhow!("{}", bins.paths.images.display()))?;
    }

    if run_bench {
        let mut args = vec!["--bench"];

        if a.quick {
            args.push("--quick");
        }

        if let Some(filter) = filter {
            args.push("--");
            args.push(filter);
        }

        let comparison_env = [(
            OsStr::new("CRITERION_HOME"),
            bins.paths.criterion_output.as_os_str(),
        )];
        run_path(&bins.bins.comparison()?, &args, &comparison_env[..])?;
    }

    if !bins.paths.criterion_output.is_dir() {
        fs::create_dir_all(&bins.paths.criterion_output)
            .with_context(|| anyhow!("{}", bins.paths.criterion_output.display()))?;
    }

    let mut output_plots = Vec::new();

    for g @ Group { id: group, .. } in &bins.manifest.groups {
        if !bins.report.only.is_empty() && !bins.report.only.iter().any(|o| *o == *group) {
            continue;
        }

        let mut plots = HashMap::new();

        for Kind { id: kind, .. } in &bins.manifest.kinds {
            let from = bins
                .paths
                .criterion_output
                .join(format!("{kind}_{group}"))
                .join("report")
                .join("violin.svg");

            ensure!(from.is_file(), "Missing {}", from.display());

            let name = format!("{kind}_{group}_{}.svg", bins.report.id);
            let to = bins.paths.images.join(&name);
            copy_svg(&from, to)
                .with_context(|| anyhow!("{}: {}", bins.report.id, from.display()))?;
            plots.insert(kind.as_str(), name);
        }

        output_plots.push((g, plots));
    }

    Ok(output_plots)
}

fn render_preamble<W>(o: &mut W, manifest: &Manifest, report: &Report) -> Result<()>
where
    W: Write,
{
    writeln!(o)?;

    let missing = report
        .expected
        .iter()
        .flat_map(|f| f.strip_prefix("no-"))
        .collect::<Vec<_>>();

    if !missing.is_empty() {
        writeln!(o, "> **Missing features:**")?;

        for feature in missing {
            if let Some(description) = manifest.missing_features.get(feature) {
                writeln!(o, "> - `{feature}` - {description}")?;
            } else {
                writeln!(o, "> - `{feature}`")?;
            }
        }

        writeln!(o)?;
    }

    for line in &report.description {
        writeln!(o, "{line}")?;
    }

    writeln!(o)?;
    Ok(())
}

fn size_comparisons<'a, W>(
    o: &mut W,
    manifest: &'a Manifest,
    size_sets: Vec<(&Report, Vec<SizeSet>)>,
    used_footnotes: &mut BTreeSet<&'a String>,
) -> Result<()>
where
    W: Write,
{
    writeln!(o, "# Size comparisons")?;
    writeln!(o)?;
    writeln!(o, "This is not yet an area which has received much focus, but because people are bound to ask the following section performs a raw size comparison between different formats.")?;

    writeln!(o, "Each test suite serializes a collection of values, which have all been randomly populated.")?;

    for Group {
        id, description, ..
    } in &manifest.groups
    {
        writeln!(o, "- {description} (`{id}`)")?;
    }

    writeln!(o)?;

    writeln!(
        o,
        "> **Note** so far these are all synthetic examples. Real world data is"
    )?;
    writeln!(
        o,
        "> rarely *this* random. But hopefully it should give an idea of the extreme"
    )?;
    writeln!(o, "> ranges.")?;

    writeln!(o)?;

    for (report, size_sets) in size_sets {
        if size_sets.is_empty() {
            continue;
        }

        writeln!(o, "#### {} sizes", report.title)?;
        render_preamble(o, manifest, report)?;

        let mut columns = Vec::new();
        let mut rows = BTreeSet::new();

        macro_rules! build_column {
            ($name:ident, $ty:ty, $num:expr, $size_hint:expr) => {
                columns.push(stringify!($name));
            };
        }

        tests::types!(build_column);

        let mut index = HashMap::<_, SizeSet>::new();

        for set in size_sets {
            rows.insert(set.framework.clone());
            let replaced = index.insert((set.suite.clone(), set.framework.clone()), set);
            assert!(replaced.is_none());
        }

        write!(o, "| **framework** |")?;

        for suite in &columns {
            write!(o, " `{suite}` |")?;
        }

        writeln!(o)?;
        write!(o, "| - |")?;

        for _ in &columns {
            write!(o, " - |")?;
        }

        writeln!(o)?;

        for framework in rows {
            let footnotes = match manifest.size_footnotes.get(framework.as_str()) {
                Some(footnotes) => {
                    used_footnotes.extend(footnotes);
                    &footnotes[..]
                }
                None => &[],
            };

            let footnote = footnotes
                .iter()
                .map(|f| format!("[^{f}]"))
                .collect::<Vec<_>>()
                .join("");

            write!(o, "| `{framework}`{footnote} |")?;

            for &suite in columns.iter() {
                let Some(mut set) = index
                    .remove(&(suite.to_owned(), framework.clone()))
                    .filter(|s| !s.samples.is_empty())
                else {
                    write!(o, " - |")?;
                    continue;
                };

                let len = set.samples.len() as f64;

                set.samples.sort();
                let mean = set.samples.iter().sum::<i64>() as f64 / len;

                let (Some(mn), Some(mx)) = (set.samples.first(), set.samples.last()) else {
                    write!(o, " - |")?;
                    continue;
                };

                let ss = set.samples.iter().map(|s| (*s as f64 - mean).powf(2.0));
                let stddev = (ss.sum::<f64>() / len).sqrt();

                write!(o, " <a title=\"samples: {len}, min: {mn}, max: {mx}, stddev: {stddev}\">{mean:.2} ± {stddev:.2}</a> |")?;
            }

            writeln!(o)?;
        }

        writeln!(o)?;
    }

    Ok(())
}

fn copy_svg(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    use std::io::Write;

    let from = from.as_ref();
    let to = to.as_ref();

    println!("copy: {} -> {}", from.display(), to.display());

    let from = BufReader::new(File::open(from)?);
    let mut to = File::create(to)?;

    for (index, line) in from.lines().enumerate() {
        if index == 1 {
            writeln!(
                to,
                "<rect width=\"100%\" height=\"100%\" fill=\"white\"></rect>"
            )?;
        }

        let line = line?;
        writeln!(to, "{}", line.trim())?;
    }

    Ok(())
}

fn run_path(path: &Path, args: &[&str], env: &[(&OsStr, &OsStr)]) -> Result<()> {
    let mut command = Command::new(path);

    for arg in args {
        command.arg(arg);
    }

    for (key, value) in env {
        command.env(*key, *value);
    }

    print_command(&command, env);

    let status = command.status()?;

    ensure!(status.success(), "Command failed: {status}");
    Ok(())
}

#[derive(Debug)]
enum Features {
    Expected(BTreeSet<String>),
    Unexpected(BTreeSet<String>),
}

#[derive(Default, Debug)]
struct CustomBuild {
    status: ExitStatus,
    all: Vec<(String, String, PathBuf)>,
    messages: Vec<String>,
    bad_features: Vec<(String, Features)>,
}

impl CustomBuild {
    fn bin(&self, kind: &str, name: &str) -> Option<PathBuf> {
        let mut bins = Vec::new();

        for (k, n, path) in &self.all {
            if k == kind && n == name {
                bins.push(path.clone());
            }
        }

        bins.pop()
    }
}

struct Build {
    fuzz: PathBuf,
    comparison: PathBuf,
}

fn build_tests(
    manifest: &Manifest,
    features: &[String],
    expected: &[String],
    command: impl AsRef<OsStr>,
    head: impl IntoIterator<Item: AsRef<OsStr>>,
    remaining: impl IntoIterator<Item: AsRef<OsStr>, IntoIter: ExactSizeIterator>,
) -> Result<CustomBuild> {
    let mut child = Command::new("cargo");
    child
        .arg(command)
        .args(["-p", "tests"])
        .args(head)
        .arg("--message-format=json");
    child.stdout(Stdio::piped());

    let features_argument = manifest
        .common
        .iter()
        .chain(features.iter())
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(",");

    child.args(["--no-default-features", "--features", &features_argument]);

    let remaining = remaining.into_iter();

    if remaining.len() > 0 {
        child.arg("--");
        child.args(remaining);
    }

    print_command(&child, &[]);

    let mut child = child.spawn()?;

    let stdout = BufReader::new(child.stdout.take().context("missing stdout")?);

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

                let mut expected = features
                    .iter()
                    .chain(expected)
                    .chain(manifest.common.iter())
                    .cloned()
                    .collect::<BTreeSet<_>>();

                let mut unexpected = BTreeSet::new();

                for feature in &a.features {
                    if !expected.remove(feature.as_str()) {
                        unexpected.insert(feature.clone());
                    }
                }

                if !expected.is_empty() {
                    bad_features.push((a.target.name.clone(), Features::Expected(expected)));
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

/// Build benchmarks.
fn build_bench(manifest: &Manifest, release: bool, report: &Report) -> Result<Build> {
    let head = release.then_some("--release");

    let build = build_tests(
        manifest,
        &report.features,
        &report.expected,
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
            print!("{}", message);
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

fn print_command(child: &Command, env: &[(&OsStr, &OsStr)]) {
    let program = child.get_program().to_string_lossy();

    let args = child
        .get_args()
        .map(|args| args.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");

    let mut e = String::new();

    if !env.is_empty() {
        for (key, value) in env {
            _ = write!(e, "{}={} ", key.to_string_lossy(), value.to_string_lossy());
        }
    }

    println!("{e}{program} {args}");
}

/// This ensures that the roundtrip encoding is correct.
fn sanity_check(path: &Path) -> Result<()> {
    let mut child = Command::new(path);
    child.args(["--iter", "1"]);
    print_command(&child, &[]);
    let status = child.status()?;
    ensure!(status.success(), "Command failed: {}", status.success());
    Ok(())
}

/// Collect size sets from the fuzz command.
fn collect_size_sets(path: &Path) -> Result<Vec<SizeSet>> {
    let mut child = Command::new(path);
    child.stdout(Stdio::piped());
    child.arg("--size");
    print_command(&child, &[]);

    let mut child = child.spawn()?;

    let stdout = BufReader::new(child.stdout.take().context("missing stdout")?);

    let mut size_sets = Vec::new();

    for line in stdout.lines() {
        let line = line?;
        size_sets.push(serde_json::from_str(&line)?);
    }

    let status = child.wait()?;

    ensure!(status.success(), "Command failed: {}", status.success());
    Ok(size_sets)
}

#[derive(Debug, Serialize, Deserialize)]
struct SizeSet {
    framework: String,
    suite: String,
    samples: Vec<i64>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Estimates {
    mean: Sample,
    median: Sample,
    median_abs_dev: Sample,
    slope: Sample,
    std_dev: Sample,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct ConfidenceInterval {
    confidence_level: f64,
    lower_bound: f64,
    upper_bound: f64,
}

#[derive(Debug, Deserialize)]
struct Sample {
    confidence_interval: ConfidenceInterval,
    point_estimate: f64,
    standard_error: f64,
}

struct Timing(f64);

fn timing(timing: f64) -> Timing {
    Timing(timing)
}

impl fmt::Display for Timing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut v = self.0;

        if v < 1000.0 {
            return write!(f, "{v:.2}ns");
        }

        v /= 1000.0;

        if v < 1000.0 {
            return write!(f, "{v:.2}μs");
        }

        v /= 1000.0;

        if v < 1000.0 {
            return write!(f, "{v:.2}ms");
        }

        v /= 1000.0;
        write!(f, "{v:.2}s")
    }
}
