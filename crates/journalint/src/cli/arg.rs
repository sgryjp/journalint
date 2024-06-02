use clap::Parser;

use crate::cli::export::ExportFormat;
use crate::cli::report::ReportFormat;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub(crate) struct Arguments {
    /// File to lint.
    pub filename: Option<String>,

    /// Fix found problems.
    #[arg(short, long)]
    pub fix: bool,

    /// Report rule violations in the specified format.
    #[clap(value_enum)]
    #[arg(long, value_name = "FORMAT", default_value_t = ReportFormat::Fancy)]
    pub report: ReportFormat,

    /// Export journal entries in the specified format.
    #[arg(short, long, value_name = "FORMAT")]
    pub export: Option<ExportFormat>,

    /// Whether to extract activity prefix as codes on exporting.
    #[arg(long, default_value_t = false)]
    pub extract_activity_prefixes: bool,

    /// Start as a language server.
    #[arg(long)]
    pub stdio: bool,
}
