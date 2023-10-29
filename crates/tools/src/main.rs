use std::collections::{BTreeSet, HashMap};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt::Write;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

const REPO: &'static str = "https://raw.githubusercontent.com/udoprog/musli";

const COMMON: &'static [&'static str] = &["no-rt", "std"];

const REPORTS: &'static [Report] = &[
    Report {
        id: "full",
        title: "Full features",
        link: "full-features",
        description: &[
            "These frameworks provide a fair comparison against Müsli on various areas since",
            "they support the same set of features in what types of data they can represent.",
        ],
        features: &[
            "musli-wire",
            "musli-descriptive",
            "musli-storage",
            "musli-value",
            "bincode",
            "rmp-serde",
            "postcard",
        ],
        expected: &[],
        only: &[],
    },
    Report {
        id: "text",
        title: "Text-based formats",
        link: "text-based-formats",
        description: &[
            "These are text-based formats, which support the full feature set of this test suite.",
        ],
        features: &[
            "musli-json",
            "serde_json",
        ],
        expected: &[],
        only: &[],
    },
    Report {
        id: "fewer",
        title: "Fewer features",
        link: "fewer-features",
        description: &[
            "This is a suite where support for 128-bit integers and maps are disabled.",
            "Usually because the underlying framework lacks support for them.",
        ],
        features: &[
            "musli-wire",
            "musli-descriptive",
            "musli-storage",
            "musli-value",
            "serde_cbor",
            "bitcode",
            "bitcode-derive",
            // "dlhn", # broken
        ],
        expected: &["model-no-128", "model-no-map"],
        only: &[],
    },
    Report {
        id: "zerocopy-rkyv",
        link: "müsli-vs-rkyv",
        description: &[
            "Comparison between [`musli-zerocopy`] and [`rkyv`].",
            "",
            "Note that `musli-zerocopy` only supports the `primitives` benchmark.",
        ],
        title: "Müsli vs rkyv",
        features: &["musli-zerocopy", "rkyv"],
        expected: &[],
        only: &["primitives", "primpacked"],
    },
    Report {
        id: "zerocopy-zerocopy",
        link: "müsli-vs-zerocopy",
        description: &[
            "Compares [`musli-zerocopy`] with [`zerocopy`].",
            "",
            "Note that `zerocopy` only supports packed primitives, so we're only comparing with that suite.",
        ],
        title: "Müsli vs zerocopy",
        features: &["musli-zerocopy", "zerocopy"],
        expected: &[],
        only: &["primpacked"],
    },
];

const LINKS: &'static [Link] = &[
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

const KINDS: &'static [(&'static str, &'static str)] =
    &[("dec", "Decode a type"), ("enc", "Encode a type")];

const GROUPS: &'static [Group] = &[
    Group {
        id: "primitives",
        description: "which is a small object containing one of each primitive type and a string and a byte array.",
    },
    Group {
        id: "primpacked",
        description: "Tried to achieve the same goal as `primitives`, but with a packed layout to support certain zerocopy libraries.",
    },
    Group {
        id: "medium_enum",
        description: "A moderately sized enum with many field variations.",
    },
    Group {
        id: "large",
        description: "A really big and complex struct.",
    },
    Group {
        id: "allocated",
        description: "A sparse struct which contains fairly plain allocated data like strings and vectors.",
    },
];

#[derive(Clone, Copy)]
struct Link {
    title: &'static str,
    href: &'static str,
}

#[derive(Clone, Copy)]
struct Group {
    id: &'static str,
    description: &'static str,
}

#[derive(Clone, Copy)]
struct Report {
    id: &'static str,
    description: &'static [&'static str],
    title: &'static str,
    link: &'static str,
    features: &'static [&'static str],
    expected: &'static [&'static str],
    only: &'static [&'static str],
}

#[derive(Deserialize)]
struct Line {
    reason: String,
    #[serde(flatten)]
    extra: serde_json::Value,
}

#[derive(Deserialize)]
struct Target {
    crate_types: Vec<String>,
    kind: Vec<String>,
    name: String,
}

#[derive(Deserialize)]
struct Profile {
    test: bool,
}

#[derive(Deserialize)]
struct CompilerArtifact {
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
    /// Filter to pass to benchmarks when running them.
    #[arg(short = 'f', long)]
    filter: Option<String>,
    /// Run benchmarks.
    #[arg(short, long)]
    bench: bool,
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

    match command {
        Cmd::Report(a) => {
            let mut o = String::new();

            writeln!(o, "# Benchmarks and size comparisons")?;
            writeln!(o)?;

            writeln!(
                o,
                "> The following are the results of preliminary benchmarking and should be"
            )?;
            writeln!(o, "> taken with a big grain of 🧂.")?;
            writeln!(o)?;

            writeln!(
                o,
                "Summary of the different kinds of benchmarks we support."
            )?;
            writeln!(o)?;

            for Group {
                id, description, ..
            } in GROUPS
            {
                writeln!(o, "- `{id}` {description}")?;
            }

            writeln!(o)?;

            writeln!(o, "The following are one section for each kind of benchmark we perform. They range from \"Full features\" to more specialized ones like zerocopy comparisons.")?;

            for Report { title, link, .. } in REPORTS {
                writeln!(o, "- [{title}](#{link})")?;
            }

            writeln!(o)?;

            writeln!(
                o,
                "Below you'll also find [Size comparisons](#size-comparisons)."
            )?;

            let mut size_sets = Vec::new();

            for report in REPORTS {
                if let Some(do_report) = args.report.as_deref() {
                    if do_report != report.id {
                        continue;
                    }
                }

                println!("Building: {}", report.title);

                writeln!(o, "# {}", report.title)?;

                writeln!(o)?;

                if !report.expected.is_empty() {
                    let features = report
                        .expected
                        .iter()
                        .map(|f| format!("`{f}`"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    writeln!(o, "> **Missing features:** {features}")?;
                    writeln!(o)?;
                }

                for line in report.description.iter().copied() {
                    writeln!(o, "{line}")?;
                }

                writeln!(o)?;

                let size_set = build_report(
                    &mut o,
                    &root,
                    a.bench,
                    a.filter.as_deref(),
                    a.branch.as_deref().unwrap_or("main"),
                    *report,
                )?;

                size_sets.push((*report, size_set));
            }

            size_comparisons(&mut o, size_sets)?;

            for Link { title, href } in LINKS {
                writeln!(o, "[{title}]: {href}")?;
            }

            let report = root.join("benchmarks.md");

            println!("Writing: {}", report.display());
            fs::write(&report, o.as_bytes())?;
        }
        Cmd::Clippy(a) => {
            let mut remaining = Vec::new();

            for arg in a.remaining {
                remaining.push(arg);
            }

            let mut builds = Vec::new();

            for report in REPORTS {
                if let Some(do_report) = args.report.as_deref() {
                    if do_report != report.id {
                        continue;
                    }
                }

                let build = build_tests(
                    report.features,
                    report.expected,
                    "clippy",
                    &[],
                    &remaining[..],
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
        Cmd::Build(a) => {
            let mut remaining = Vec::new();

            for arg in a.remaining {
                remaining.push(arg);
            }

            let mut builds = Vec::new();

            for report in REPORTS {
                if let Some(do_report) = args.report.as_deref() {
                    if do_report != report.id {
                        continue;
                    }
                }

                let build =
                    build_tests(report.features, report.expected, "build", &[], &remaining)?;

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

fn build_report<W>(
    o: &mut W,
    root: &Path,
    run_bench: bool,
    filter: Option<&str>,
    branch: &str,
    report: Report,
) -> Result<Vec<SizeSet>>
where
    W: ?Sized + Write,
{
    let output = root.join("images");
    let target_dir = root.join("target");

    let bins = build_bench(report.features, report.expected)?;

    if run_bench {
        run_path(&bins.comparison, &[])?;

        let mut args = vec!["--bench"];

        if let Some(filter) = filter {
            args.push("--");
            args.push(filter);
        }

        args.extend([
            "--save-baseline",
            report.id,
            "--measurement-time",
            "0.5",
            "--warm-up-time",
            "0.1",
        ]);
        run_path(&bins.comparison, &args)?;
    }

    for Group { id: group, .. } in GROUPS {
        if !report.only.is_empty() && !report.only.contains(group) {
            continue;
        }

        let mut plots = Vec::new();

        for (kind, _) in KINDS {
            let name = format!("{kind}_{group}_{}.svg", report.id);

            let criterion_dir = target_dir
                .join("criterion")
                .join(format!("{kind}_{group}"))
                .join("report");

            let from = criterion_dir.join("violin.svg");
            let to = output.join(&name);

            if run_bench {
                copy_svg(&from, &to)
                    .with_context(|| anyhow!("{}: {}", report.id, from.display()))?;
            }

            plots.push(name);
        }

        let kinds = KINDS
            .iter()
            .map(|(k, d)| format!("`{k}` - {d}"))
            .collect::<Vec<_>>()
            .join(", ");

        write!(o, "`{group}`: {kinds}.")?;

        writeln!(o)?;
        writeln!(o)?;

        for plot in &plots {
            writeln!(
                o,
                "<img style=\"background-color: white;\" src=\"{REPO}/{branch}/images/{plot}\">"
            )?;
            writeln!(o)?;
        }
    }

    let size_sets = collect_size_sets(&bins.fuzz)?;
    Ok(size_sets)
}

fn size_comparisons<W>(o: &mut W, size_sets: Vec<(Report, Vec<SizeSet>)>) -> Result<()>
where
    W: Write,
{
    writeln!(o, "# Size comparisons")?;
    writeln!(o)?;
    writeln!(o, "This is not yet an area which has received much focus, but because people are bound to ask the following section performs a raw size comparison between different formats.")?;

    writeln!(o, "Each test suite serializes a collection of values, which have all been randomly populated.")?;

    for Group {
        id, description, ..
    } in GROUPS
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

    let mut footnotes = HashMap::new();
    footnotes.insert("[^incomplete]", "These formats do not support a wide range of Rust types. Exact level of support varies. But from a size perspective it makes size comparisons either unfair or simply an esoteric exercise since they can (or cannot) make stricter assumptions as a result.");
    footnotes.insert("[^i128]", "Lacks 128-bit support.");

    let mut crate_footnotes = HashMap::new();

    crate_footnotes.insert("musli_json", "[^incomplete]");
    crate_footnotes.insert("rkyv", "[^incomplete]");
    crate_footnotes.insert("serde_bitcode", "[^i128]");
    crate_footnotes.insert("serde_cbor", "[^i128]");
    crate_footnotes.insert("serde_dlhn", "[^i128]");
    crate_footnotes.insert("serde_json", "[^incomplete]");
    crate_footnotes.insert("derive_bitcode", "[^i128]");

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
            let footnote = match crate_footnotes.get(framework.as_str()).copied() {
                Some(footnote) => {
                    used_footnotes.insert(footnote);
                    footnote
                }
                None => "",
            };

            write!(o, "| {framework}{footnote} |")?;

            for suite in columns.iter().copied() {
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
                let Some(note) = footnotes.get(footnote) else {
                    continue;
                };

                writeln!(o, "{footnote}: {note}")?;
            }

            writeln!(o)?;
        }

        writeln!(o)?;
    }

    Ok(())
}

fn copy_svg(from: &Path, to: &Path) -> Result<()> {
    use std::io::Write;

    println!("copy: {} -> {}", from.display(), to.display());

    let from = BufReader::new(File::open(from)?);
    let mut to = File::create(to)?;

    for (index, line) in from.lines().enumerate() {
        if index == 1 {
            write!(
                to,
                "<rect width=\"100%\" height=\"100%\" fill=\"white\"></rect>\n"
            )?;
        }

        let line = line?;
        write!(to, "{}\n", line.trim())?;
    }

    Ok(())
}

fn run_path(path: &Path, args: &[&str]) -> Result<()> {
    let mut command = Command::new(path);

    for arg in args {
        command.arg(arg);
    }

    print_command(&command);

    let status = command.status()?;

    if !status.success() {
        bail!("Command failed: {status}")
    }

    Ok(())
}

#[derive(Default, Debug)]
struct CustomBuild {
    status: ExitStatus,
    all: Vec<(String, String, PathBuf)>,
    messages: Vec<String>,
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
    features: &[&str],
    expected_features: &[&str],
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

    let features = COMMON
        .iter()
        .chain(features)
        .copied()
        .collect::<Vec<_>>()
        .join(",");

    child.args(["--no-default-features", "--features", &features]);

    if !remaining.is_empty() {
        child.arg("--");
        child.args(remaining);
    }

    print_command(&child);

    let mut child = child.spawn()?;

    let stdout = BufReader::new(child.stdout.take().context("missing stdout")?);

    let mut all = Vec::new();
    let mut messages = Vec::new();

    for line in stdout.lines() {
        let line = line?;
        let line: Line = serde_json::from_str(&line)?;

        match line.reason.as_str() {
            "compiler-message" => {
                let message: CompilerMessage = serde_json::from_value(line.extra)?;
                messages.push(message.message.rendered);
            }
            "compiler-artifact" => {
                let artifact: CompilerArtifact = serde_json::from_value(line.extra.clone())?;

                if !(artifact
                    .target
                    .crate_types
                    .iter()
                    .any(|value| value == "bin"))
                {
                    continue;
                }

                let Some(executable) = artifact.executable else {
                    continue;
                };

                let mut expected = expected_features.iter().copied().collect::<BTreeSet<_>>();

                for feature in &artifact.features {
                    expected.remove(feature.as_str());
                }

                if !expected.is_empty() {
                    bail!(
                        "Building executable did not have model features: {:?}",
                        expected
                    );
                }

                match (
                    artifact.target.kind.first().map(|s| s.as_str()),
                    artifact.target.name.as_str(),
                ) {
                    (Some(kind), name) => {
                        if kind == "bin" && artifact.profile.test {
                            continue;
                        }

                        all.push((
                            kind.to_owned(),
                            name.to_owned(),
                            PathBuf::from(executable.clone()),
                        ));
                    }
                    _ => {}
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
    })
}

/// Build benchmarks.
fn build_bench(features: &[&str], expected_features: &[&str]) -> Result<Build> {
    let build = build_tests(
        features,
        expected_features,
        "build",
        &["--release", "--benches"],
        &[],
    )?;

    if !build.status.success() {
        for message in build.messages {
            print!("{}", message);
        }

        bail!("Command failed: {}", build.status.success());
    }

    dbg!(&build.all);
    let fuzz = build.bin("bin", "fuzz").context("missing fuzz")?;
    let comparison = build
        .bin("bench", "comparison")
        .context("missing comparison")?;
    Ok(Build { fuzz, comparison })
}

fn print_command(child: &Command) {
    let program = child.get_program().to_string_lossy();

    let args = child
        .get_args()
        .map(|args| args.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");

    println!("{program} {args}");
}

/// Collect size sets from the fuzz command.
fn collect_size_sets(path: &Path) -> Result<Vec<SizeSet>> {
    let mut child = Command::new(path);
    child.stdout(Stdio::piped());
    child.arg("--size");
    print_command(&child);

    let mut child = child.spawn()?;

    let stdout = BufReader::new(child.stdout.take().context("missing stdout")?);

    let mut size_sets = Vec::new();

    for line in stdout.lines() {
        let line = line?;
        size_sets.push(serde_json::from_str(&line)?);
    }

    let status = child.wait()?;

    if !status.success() {
        bail!("Command failed: {}", status.success());
    }

    Ok(size_sets)
}

#[derive(Serialize, Deserialize)]
struct SizeSet {
    framework: String,
    suite: String,
    samples: Vec<i64>,
}
