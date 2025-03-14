use std::ffi::OsStr;
use std::process::Command;

use anyhow::Result;

use crate::ReportRef;

pub(crate) fn cargo(
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

// Print a command.
pub(crate) fn print(child: &Command) {
    use std::fmt::Write;

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
