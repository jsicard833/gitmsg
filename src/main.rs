mod ai;
mod cli;
mod commit;
mod git;

use anyhow::Result;
use clap::Parser;
use std::io::{self, Write};

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

    let message = if cli.interactive {
        choose_interactive_message(&config, &diff, cli.verbose).await?
    } else {
        let raw_message = ai::suggest_commit_message(&config, &diff, cli.verbose).await?;
        commit::clean_commit_message(&raw_message)?
    };

    println!("{message}");

    if cli.commit {
        git::commit(&message)?;
        if cli.verbose {
            eprintln!("git commit completed");
        }
    }

    Ok(())
}

async fn choose_interactive_message(
    config: &ai::AiConfig,
    diff: &str,
    verbose: bool,
) -> Result<String> {
    let suggestions = ai::suggest_commit_messages(config, diff, 3, verbose).await?;

    for (index, suggestion) in suggestions.iter().enumerate() {
        println!("{}. {}", index + 1, suggestion);
    }

    let choice = prompt_choice(suggestions.len())?;
    suggestions
        .get(choice - 1)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("selected suggestion was not available"))
}

fn prompt_choice(max_choice: usize) -> Result<usize> {
    loop {
        print!("Select [1]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(1);
        }

        if let Ok(choice) = trimmed.parse::<usize>()
            && (1..=max_choice).contains(&choice)
        {
            return Ok(choice);
        }

        eprintln!("Please enter a number from 1 to {max_choice}.");
    }
}
