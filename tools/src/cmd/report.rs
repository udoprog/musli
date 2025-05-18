use std::collections::{BTreeSet, HashMap};
use std::ffi::OsStr;
use std::fmt;
use std::fmt::Write;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{anyhow, bail, ensure, Context, Result};
use clap::Parser;
use criterion::Criterion;
use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::bins::{BinArgs, Binary, Bins};
use crate::manifest::{Group, Kind, Link, ReportEnv};
use crate::{command, Manifest, Report, ReportRef, SharedArgs, REPO};

#[derive(Default, Parser)]
pub(crate) struct Args {
    #[command(flatten)]
    shared: SharedArgs,
    #[command(flatten)]
    bins: BinArgs,
    /// The output directory to write results into.
    #[arg(long)]
    pub(crate) output: Option<PathBuf>,
    /// Custom path to write index output.
    #[arg(long)]
    index_output: Option<PathBuf>,
    /// Filter to pass to benchmarks when running them.
    #[arg(short = 'f', long)]
    filter: Option<String>,
    /// Disable running benchmarks.
    ///
    /// This will otherwise be done automatically for a report if its output is
    /// missing.
    #[arg(long)]
    no_bench: bool,
    /// Run `--quick` benchmarks.
    #[arg(long)]
    quick: bool,
    /// Skip size comparisons.
    #[arg(long)]
    no_size: bool,
    /// Reference graphics from the given branch.
    #[arg(long)]
    branch: Option<String>,
}

pub(crate) fn entry(args: &Args, manifest: &Manifest, target: &Path, output: &Path) -> Result<()> {
    let bins = manifest.bins(target, output, &args.shared, &args.bins)?;

    let mut size_sets = Vec::new();

    if !args.no_bench {
        for bins in &bins {
            println!("{}: Testing comparison benchmark", bins.report.title);
            bins.comparison()?.run(&[], &[])?;
        }

        let mut errored = 0usize;

        for bins in &bins {
            println!("{}: Benchmarking", bins.report.title);

            if let Err(error) = build_report(args, bins, args.filter.as_deref()) {
                errored += 1;

                println!("{}: Failed benchmark", bins.report.title);

                for error in error.chain() {
                    println!("  Caused by: {error}");
                }
            }
        }

        if errored > 0 {
            bail!("{errored} builds failed");
        }
    }

    if !args.no_size {
        for bins in &bins {
            println!("{}: Testing fuzz tool", bins.report.title);

            bins.tests()?
                .run(&["--iter", "1"], &[])
                .context("Sanity check failed")?;
        }

        for bins in &bins {
            println!("{}: Sizing", bins.report.title);
            let size_set = collect_size_sets(bins.tests()?).context("Collecting size sets")?;
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

    let mut reports = Vec::new();

    for bins in &bins {
        reports.push(output_plots(bins)?);
    }

    if !reports.is_empty() {
        writeln!(o, "The following are one section for each kind of benchmark we perform. They range from \"Full features\" to more specialized ones like zerocopy comparisons.")?;

        for bins in &bins {
            let Report {
                id, title, link, ..
            } = &*bins.report;

            writeln!(
                o,
                "- [**{title}**](#{link}) ([Report 📓]({url}/criterion-{id}/report/), [Sizes](#{link}-sizes))",
                url = manifest.url
            )?;
        }

        writeln!(o)?;
    }

    writeln!(
        o,
        "Below you'll also find [size comparisons](#size-comparisons)."
    )?;
    writeln!(o)?;

    render_system_info(&mut o)?;
    render_reports(&mut o, args, manifest, &bins, &reports, &mut used_footnotes)?;
    size_comparisons(&mut o, manifest, size_sets, &mut used_footnotes)?;

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

    let report = match &args.index_output {
        Some(report) => report.clone(),
        None => output.join("index.md"),
    };

    println!("Writing: {}", report.display());

    if let Some(dir) = report.parent() {
        if !dir.is_dir() {
            fs::create_dir_all(dir).with_context(|| dir.display().to_string())?;
        }
    }

    fs::write(report, o.as_bytes())?;
    Ok(())
}

fn build_report(a: &Args, bins: &Bins<'_>, filter: Option<&str>) -> Result<()> {
    if !bins.paths.images.is_dir() {
        fs::create_dir_all(&bins.paths.images)
            .with_context(|| anyhow!("{}", bins.paths.images.display()))?;
    }

    let done_path = bins.paths.criterion_output.join(".done");

    let mut ran_benchmarks = false;

    if !done_path.exists() {
        let mut args = vec!["--bench"];

        if a.quick {
            args.push("--quick");
        }

        if let Some(filter) = filter {
            args.push("--");
            args.push(filter);
        }

        // Disable final summary since we do it here.
        let env = [
            (
                OsStr::new("CRITERION_HOME"),
                bins.paths.criterion_output.as_os_str(),
            ),
            (OsStr::new("MUSLI_FINAL_SUMMARY"), OsStr::new("no")),
        ];

        bins.comparison()?.run(&args, &env[..])?;
        ran_benchmarks = true;
    }

    // Generate all needed graphics and reports.
    Criterion::default()
        .output_directory(&bins.paths.criterion_output)
        .final_summary();

    if ran_benchmarks {
        fs::File::create(&done_path).with_context(|| done_path.display().to_string())?;
    }

    Ok(())
}

fn output_plots<'a>(bins: &Bins<'a>) -> Result<Vec<(&'a Group, HashMap<&'a str, String>)>> {
    let mut output_plots = Vec::new();

    for g @ Group { id: group, .. } in &bins.report.manifest.groups {
        if !bins.report.only.is_empty() && !bins.report.only.contains(group) {
            continue;
        }

        let mut plots = HashMap::new();

        for Kind { id: kind, .. } in &bins.report.manifest.kinds {
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

fn render_reports<'a, O>(
    o: &mut O,
    args: &Args,
    manifest: &'a Manifest,
    bins: &[Bins<'_>],
    reports: &[Vec<(&Group, HashMap<&str, String>)>],
    used_footnotes: &mut BTreeSet<&'a str>,
) -> Result<()>
where
    O: ?Sized + Write,
{
    let branch = args.branch.as_deref().unwrap_or(manifest.branch.as_str());

    writeln!(o, "## Reports")?;
    writeln!(o)?;

    for (bins @ &Bins { report, .. }, group_plots) in bins.iter().zip(reports) {
        writeln!(o, "### {}", report.title)?;
        render_preamble(o, report)?;

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
                        used_footnotes.extend(footnotes.iter().map(String::as_str));

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

    Ok(())
}

fn size_comparisons<'a, W>(
    o: &mut W,
    manifest: &'a Manifest,
    size_sets: Vec<(ReportRef<'_>, Vec<SizeSet>)>,
    used_footnotes: &mut BTreeSet<&'a str>,
) -> Result<()>
where
    W: Write,
{
    writeln!(o, "## Size comparisons")?;
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
        render_preamble(o, report)?;

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
                    used_footnotes.extend(footnotes.iter().map(String::as_str));
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

fn render_preamble<W>(o: &mut W, report: ReportRef<'_>) -> Result<()>
where
    W: ?Sized + Write,
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
            if let Some(description) = report.manifest.missing_features.get(feature) {
                writeln!(o, "> - `{feature}` - {description}")?;
            } else {
                writeln!(o, "> - `{feature}`")?;
            }
        }

        writeln!(o)?;
    }

    if !report.env.is_empty() {
        writeln!(o, "> **Custom environment:**")?;

        for ReportEnv {
            key,
            value,
            description,
        } in &report.env
        {
            writeln!(o, "> - `{key}={value}` - {description}")?;
        }

        writeln!(o)?;
    }

    for line in &report.description {
        writeln!(o, "{line}")?;
    }

    writeln!(o)?;
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

/// Collect size sets from the tests command.
fn collect_size_sets(path: Binary<'_>) -> Result<Vec<SizeSet>> {
    let mut child = path.command();
    child.stdout(Stdio::piped());
    child.arg("--size");

    command::print(&child);

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
    #[serde(default)]
    slope: Option<Sample>,
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

fn render_system_info<O>(o: &mut O) -> Result<()>
where
    O: ?Sized + fmt::Write,
{
    let s = System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::everything())
            .with_cpu(CpuRefreshKind::everything()),
    );

    writeln!(o, "## System Information")?;
    writeln!(o)?;

    for cpu in s.cpus().iter().take(1) {
        writeln!(o, "**CPU:** {} {}MHz", cpu.brand(), cpu.frequency())?;
        writeln!(o)?;
    }

    writeln!(o, "**Memory:** {}MB", s.total_memory() / 1_000_000)?;
    writeln!(o)?;
    Ok(())
}
