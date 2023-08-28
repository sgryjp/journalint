use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Arguments {
    /// File to lint.
    pub filename: Option<String>,

    /// Fix found problems.
    #[arg(short, long)]
    pub fix: bool,

    /// Start as a language server.
    #[arg(long)]
    pub stdio: bool,
}
