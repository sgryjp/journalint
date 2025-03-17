use std::io::Write;
use std::sync::Arc;

use ariadne::{Color, Label, Report, ReportKind, Source};
use clap::ValueEnum;
use journalint_parse::diagnostic::Diagnostic;

use crate::line_mapper::LineMapper;

// Format of rule violation report.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ReportFormat {
    // Report rule violations using annotated source text.
    Fancy,

    // Report rule violations in one-line message format.
    OneLine,
}

/// Write a human readable report of a diagnostic
#[warn(unused_results)]
pub fn report<W: Write>(
    format: ReportFormat,
    content: &str,
    line_mapper: &Arc<LineMapper>,
    filename: Option<&str>,
    diagnostic: &Diagnostic,
    mut w: W,
) -> std::io::Result<()> {
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
                .write((filename, Source::from(content)), w)?
        }
        ReportFormat::OneLine => {
            let start = line_mapper.position_from_offset(diagnostic.span().start);
            let colon = Color::Cyan.paint(":");
            writeln!(
                w,
                "{}{colon}{}{colon}{}{colon} {} {}",
                Color::White.paint(filename).bold(),
                start.line + 1,
                start.character + 1,
                Color::Red.paint(diagnostic.rule()),
                diagnostic.message()
            )?;
        }
    };
    Ok(())
}
