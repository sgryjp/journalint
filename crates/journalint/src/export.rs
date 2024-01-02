/// Export data format.
#[derive(Clone, Debug, clap::ValueEnum)]
pub enum ExportFormat {
    /// JSON Lines.
    Json,

    /// CSV with a header line.
    Csv,
}
