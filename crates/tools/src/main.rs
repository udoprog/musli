use std::collections::{BTreeSet, HashMap};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt::Write;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{anyhow, bail, ensure, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

const REPO: &str = "https://raw.githubusercontent.com/udoprog/musli";

const LINKS: &[Link] = &[
    Link {
        title: "`rkyv`",
        href: "https://docs.rs/rkyv",
    },
    Link {
        title: "`zerocopy`",
        href: "https://docs.rs/zerocopy",
    },
    Link {
        title: "`musli-zerocopy`",
        href: "https://docs.rs/musli-zerocopy",
    },
];

#[derive(Clone, Copy)]
struct Link {
    title: &'static str,
    href: &'static str,
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
    footnotes: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct Report {
    id: String,
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
struct ArgsReport {
    /// The output directory to write results into.
    #[arg(long)]
    output: Option<PathBuf>,
    /// Filter to pass to benchmarks when running them.
    #[arg(short = 'f', long)]
    filter: Option<String>,
    /// Run benchmarks.
    #[arg(long)]
    bench: bool,
    /// Run `--quick` benchmarks.
    #[arg(long)]
    quick: bool,
    /// Reference graphics from the given branch.
    #[arg(long)]
    branch: Option<String>,
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
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").context("missing `CARGO_MANIFEST_DIR`")?)
            .join("..")
            .join("..");

    let args = Args::try_parse()?;

    let command = args.command.unwrap_or_default();

    let reports_path = root.join("crates").join("tools").join("report.toml");
    let reports = fs::read_to_string(&reports_path)?;
    let manifest: Manifest =
        toml::from_str(&reports).with_context(|| anyhow!("{}", reports_path.display()))?;

    match command {
        Cmd::Report(a) => {
            let output = match &a.output {
                Some(output) => output.to_owned(),
                None => root.join("benchmarks-new"),
            };

            let branch = a.branch.as_deref().unwrap_or(manifest.branch.as_str());

            let mut built_reports = Vec::new();
            let mut size_sets = Vec::new();

            for report in &manifest.reports {
                if let Some(do_report) = args.report.as_deref() {
                    if do_report != report.id {
                        continue;
                    }
                }

                println!("Building: {}", report.title);

                let (size_set, group_plots) =
                    build_report(&a, &manifest, report, &output, a.bench, a.filter.as_deref())?;

                size_sets.push((report, size_set));
                built_reports.push((report, group_plots));
            }

            let mut o = String::new();

            writeln!(o, "# Benchmarks and size comparisons")?;
            writeln!(o)?;

            for line in &manifest.header {
                writeln!(o, "> {line}")?;
            }

            writeln!(o)?;

            writeln!(
                o,
                "Summary of the different kinds of benchmarks we support."
            )?;
            writeln!(o)?;

            for Group {
                id, description, ..
            } in &manifest.groups
            {
                writeln!(o, "- `{id}` {description}")?;
            }

            writeln!(o)?;

            writeln!(o, "The following are one section for each kind of benchmark we perform. They range from \"Full features\" to more specialized ones like zerocopy comparisons.")?;

            for (
                Report {
                    id, title, link, ..
                },
                _,
            ) in &built_reports
            {
                writeln!(
                    o,
                    "- [{title}](#{link}) ([Full criterion report]({url}/criterion-{id}/report/))",
                    url = manifest.url
                )?;
            }

            writeln!(o)?;

            writeln!(
                o,
                "Below you'll also find [Size comparisons](#size-comparisons)."
            )?;

            for (report, group_plots) in &built_reports {
                writeln!(o, "# {}", report.title)?;

                writeln!(o)?;

                let missing = report
                    .expected
                    .iter()
                    .flat_map(|f| f.strip_prefix("model-no-"))
                    .map(|f| format!("`{f}`"))
                    .collect::<Vec<_>>();

                if !missing.is_empty() {
                    writeln!(o, "> **Missing features:** {}", missing.join(", "))?;
                    writeln!(o)?;
                }

                for line in &report.description {
                    writeln!(o, "{line}")?;
                }

                writeln!(o)?;
                writeln!(
                    o,
                    "[Full criterion report]({url}/criterion-{id}/report/)",
                    url = manifest.url,
                    id = report.id
                )?;
                writeln!(o)?;

                for (Group { id: group, .. }, plots) in group_plots {
                    let kinds = manifest
                        .kinds
                        .iter()
                        .map(|Kind { id, description }| format!("`{id}` - {description}"))
                        .collect::<Vec<_>>()
                        .join(", ");

                    write!(o, "`{group}`: {kinds}.")?;

                    writeln!(o)?;
                    writeln!(o)?;

                    for plot in plots {
                        writeln!(
                            o,
                            "<img style=\"background-color: white;\" src=\"{REPO}/{branch}/benchmarks/images/{plot}\">"
                        )?;
                        writeln!(o)?;
                    }
                }

                writeln!(o)?;
            }

            size_comparisons(&manifest, &mut o, size_sets)?;

            for Link { title, href } in LINKS {
                writeln!(o, "[{title}]: {href}")?;
            }

            let report = output.join("benchmarks.md");

            println!("Writing: {}", report.display());
            fs::write(&report, o.as_bytes())?;
        }
        Cmd::Clippy(a) => {
            let mut remaining = Vec::new();

            for arg in a.remaining {
                remaining.push(arg);
            }

            let mut builds = Vec::new();

            for report in &manifest.reports {
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
                    &[],
                    &remaining[..],
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
            let mut remaining = Vec::new();

            for arg in a.remaining {
                remaining.push(arg);
            }

            let mut builds = Vec::new();

            for report in &manifest.reports {
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
                    &[],
                    &remaining,
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
    }

    Ok(())
}

type ReportPairs<'a> = (Vec<SizeSet>, Vec<(&'a Group, Vec<String>)>);

fn build_report<'a>(
    a: &ArgsReport,
    manifest: &'a Manifest,
    report: &Report,
    output: &Path,
    run_bench: bool,
    filter: Option<&str>,
) -> Result<ReportPairs<'a>> {
    let criterion_output = output.join(format!("criterion-{}", report.id));
    let images = output.join("images");

    if !images.is_dir() {
        fs::create_dir_all(&images).with_context(|| anyhow!("{}", images.display()))?;
    }

    let bins = build_bench(manifest, report)?;

    if run_bench {
        // Just test the binaries.
        run_path(&bins.comparison, &[], &[])?;

        let mut args = vec!["--bench"];

        if a.quick {
            args.push("--quick");
        }

        if let Some(filter) = filter {
            args.push("--");
            args.push(filter);
        }

        let comparison_env = [(OsStr::new("CRITERION_HOME"), criterion_output.as_os_str())];
        run_path(&bins.comparison, &args, &comparison_env[..])?;
    }

    if !criterion_output.is_dir() {
        fs::create_dir_all(&criterion_output)
            .with_context(|| anyhow!("{}", criterion_output.display()))?;
    }

    let mut output_plots = Vec::new();

    for g @ Group { id: group, .. } in &manifest.groups {
        if !report.only.is_empty() && !report.only.iter().any(|o| *o == *group) {
            continue;
        }

        let mut plots = Vec::new();

        for Kind { id: kind, .. } in &manifest.kinds {
            let from = criterion_output
                .join(format!("{kind}_{group}"))
                .join("report")
                .join("violin.svg");

            ensure!(from.is_file(), "Missing {}", from.display());

            let name = format!("{kind}_{group}_{}.svg", report.id);
            let to = images.join(&name);
            copy_svg(&from, to).with_context(|| anyhow!("{}: {}", report.id, from.display()))?;
            plots.push(name);
        }

        output_plots.push((g, plots));
    }

    let size_sets = collect_size_sets(&bins.fuzz)?;
    Ok((size_sets, output_plots))
}

fn size_comparisons<W>(
    manifest: &Manifest,
    o: &mut W,
    size_sets: Vec<(&Report, Vec<SizeSet>)>,
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

    for (Report { title, .. }, size_sets) in size_sets {
        if size_sets.is_empty() {
            continue;
        }

        writeln!(o, "#### {title}")?;
        writeln!(o)?;

        let mut columns = Vec::new();
        let mut rows = BTreeSet::new();

        macro_rules! build_column {
            ($($name:ident, $ty:ty, $num:expr, $size_hint:expr),*) => {
                $(columns.push(stringify!($name));)*
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
            write!(o, " **{suite}** |")?;
        }

        writeln!(o)?;
        write!(o, "| - |")?;

        for _ in &columns {
            write!(o, " - |")?;
        }

        writeln!(o)?;

        let mut used_footnotes = BTreeSet::new();

        for framework in rows {
            let footnotes = match manifest.crate_footnotes.get(framework.as_str()) {
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
            write!(o, "| {framework}{footnote} |")?;

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

        if !used_footnotes.is_empty() {
            for footnote in used_footnotes {
                let Some(note) = manifest.footnotes.get(footnote) else {
                    continue;
                };

                writeln!(o, "[^{footnote}]: {note}")?;
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

fn build_tests<C, S>(
    manifest: &Manifest,
    features: &[String],
    expected: &[String],
    command: C,
    head: &[S],
    remaining: &[S],
) -> Result<CustomBuild>
where
    C: AsRef<OsStr>,
    S: AsRef<OsStr>,
{
    let mut child = Command::new("cargo");
    child.arg(command);
    child.args(["-p", "tests"]);

    if !head.is_empty() {
        child.args(head);
    }

    child.arg("--message-format=json");
    child.stdout(Stdio::piped());

    let features_argument = manifest
        .common
        .iter()
        .chain(features.iter())
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(",");

    child.args(["--no-default-features", "--features", &features_argument]);

    if !remaining.is_empty() {
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
fn build_bench(manifest: &Manifest, report: &Report) -> Result<Build> {
    let build = build_tests(
        manifest,
        &report.features,
        &report.expected,
        "build",
        &["--release", "--benches"],
        &[],
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

#[derive(Serialize, Deserialize)]
struct SizeSet {
    framework: String,
    suite: String,
    samples: Vec<i64>,
}
