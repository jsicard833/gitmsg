use anyhow::{Context, Result, bail};
use std::process::{Command, Output};

pub fn staged_diff() -> Result<String> {
    ensure_git_repo()?;

    let output = run_git(&["diff", "--cached"])?;
    let diff =
        String::from_utf8(output.stdout).context("git diff --cached produced non-UTF-8 output")?;

    if diff.trim().is_empty() {
        bail!("no staged changes found; stage files with `git add` first");
    }

    Ok(diff)
}

pub fn commit(message: &str) -> Result<()> {
    let output = run_git(&["commit", "-m", message])?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        let details = if stderr.is_empty() { stdout } else { stderr };
        bail!("git commit failed: {details}");
    }

    Ok(())
}

fn ensure_git_repo() -> Result<()> {
    let output = run_git(&["rev-parse", "--is-inside-work-tree"])?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not a git repository") {
            bail!("not inside a git repository");
        }
        bail!(
            "unable to determine git repository status: {}",
            stderr.trim()
        );
    }

    Ok(())
}

fn run_git(args: &[&str]) -> Result<Output> {
    Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))
}
