use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Arguments {
    /// File to lint.
    pub filename: Option<String>,

    /// Whether to start as a language server or not.
    #[arg(long)]
    pub stdio: bool,
}
