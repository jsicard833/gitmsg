use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Suggest a Conventional Commit message from staged changes"
)]
pub struct Cli {
    #[arg(
        short = 'i',
        long,
        help = "Show multiple suggestions and let you choose one"
    )]
    pub interactive: bool,

    #[arg(long, help = "Run git commit -m with the generated message")]
    pub commit: bool,

    #[arg(short = 'v', long, help = "Print extra debugging information")]
    pub verbose: bool,
}
