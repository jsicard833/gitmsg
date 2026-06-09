mod ai;
mod cli;
mod commit;
mod git;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = cli::Cli::parse();
    let config = ai::AiConfig::from_env()?;
    let diff = git::staged_diff()?;

    if cli.verbose {
        eprintln!(
            "staged diff: {} bytes across {} lines",
            diff.len(),
            diff.lines().count()
        );
    }

    let raw_message = ai::suggest_commit_message(&config, &diff, cli.verbose).await?;
    let message = commit::clean_commit_message(&raw_message)?;

    println!("{message}");

    if cli.commit {
        git::commit(&message)?;
        if cli.verbose {
            eprintln!("git commit completed");
        }
    }

    Ok(())
}
