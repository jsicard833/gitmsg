use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Suggest a Conventional Commit message from staged changes"
)]
pub struct Cli {
    #[arg(long, help = "Run git commit -m with the generated message")]
    pub commit: bool,

    #[arg(long, help = "Print extra debugging information")]
    pub verbose: bool,
}
