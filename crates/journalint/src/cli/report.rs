use std::sync::Arc;

use ariadne::{Color, Label, Report, ReportKind, Source};
use clap::ValueEnum;
use journalint_parse::diagnostic::Diagnostic;

use crate::linemap::LineMap;

// Format of violation report.
#[derive(Clone, Debug, ValueEnum)]
pub enum ReportFormat {
    // Report violations using annotated source text.
    Fancy,

    // Report violations in one-line message format.
    Oneline,
}

/// Write a human readable report of a diagnostic
pub fn report(
    format: &ReportFormat,
    content: &str,
    line_map: &Arc<LineMap>,
    filename: Option<&str>,
    diagnostic: &Diagnostic,
) {
    let stdin_source_name = "<STDIN>".to_string();
    let filename = filename.unwrap_or(&stdin_source_name);
    let message = diagnostic.message();
    match format {
        ReportFormat::Fancy => {
            let start = diagnostic.span().start;
            let end = diagnostic.span().end;
            Report::build(ReportKind::Error, filename, start)
                .with_message(message)
                .with_label(
                    Label::new((filename, start..end))
                        .with_color(Color::Red)
                        .with_message(message),
                )
                .finish()
                .eprint((filename, Source::from(content)))
                .unwrap()
        }
        ReportFormat::Oneline => {
            let start = line_map.position_from_offset(diagnostic.span().start);
            let colon = Color::Cyan.paint(":");
            eprintln!(
                "{}{colon}{}{colon}{}{colon} {} {}",
                Color::White.paint(filename).bold(),
                start.line + 1,
                start.character + 1,
                Color::Red.paint(diagnostic.violation()),
                diagnostic.message()
            )
        }
    };
}
