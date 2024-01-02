use clap::Parser;

use crate::export::ExportFormat;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Arguments {
    /// File to lint.
    pub filename: Option<String>,

    /// Fix found problems.
    #[arg(short, long)]
    pub fix: bool,

    #[arg(short, long, value_name = "FORMAT")]
    pub export: Option<ExportFormat>,

    /// Start as a language server.
    #[arg(long)]
    pub stdio: bool,
}
