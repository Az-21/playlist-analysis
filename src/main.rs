mod cli;

use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use tracing_subscriber::EnvFilter;

use playlist_analysis::analysis;
use playlist_analysis::export::{self, ReportMeta};
use playlist_analysis::parser::{self, LibraryFormat, ParseReport, Service, SourceKind};

fn main() -> Result<()> {
    let args = cli::Args::parse();
    init_tracing(args.verbose);

    let input = std::fs::read(&args.input)
        .with_context(|| format!("failed to read {}", args.input.display()))?;
    let source = resolve_source(&args, &input)?;
    let parsed = parser::parse(source, &input)
        .with_context(|| format!("failed to parse {}", args.input.display()))?;
    log_parse_report(&parsed.report);

    let analyses = analysis::default_analyses();
    let report = analysis::run(&parsed.library, &analyses);
    let meta = ReportMeta {
        title: "Playlist Analysis".to_owned(),
        source_name: source_name(&args.input),
        generated_at: generated_at()?,
    };
    let exporter = export::exporter(&args.output_format)?;
    exporter
        .export(&report, &parsed.library, &meta, &args.output_dir)
        .with_context(|| format!("failed to write report into {}", args.output_dir.display()))?;
    tracing::info!(
        findings = report.total_findings,
        output = %args.output_dir.display(),
        "report written",
    );
    Ok(())
}

fn init_tracing(verbose: bool) {
    let default_level = if verbose { "debug" } else { "info" };
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn resolve_source(args: &cli::Args, input: &[u8]) -> Result<SourceKind> {
    match (args.service, args.input_format) {
        (None, None) => parser::detect(input)
            .context("could not detect the export format; pass --service and --input-format"),
        (service, format) => Ok(SourceKind {
            service: service.map_or_else(Service::default, Service::from),
            format: format.map_or_else(LibraryFormat::default, LibraryFormat::from),
        }),
    }
}

fn log_parse_report(report: &ParseReport) {
    if *report != ParseReport::default() {
        tracing::warn!(
            skipped_tracks = report.skipped_tracks,
            skipped_playlist_entries = report.skipped_playlist_entries,
            skipped_playlists = report.skipped_playlists,
            "some items were skipped while parsing",
        );
    }
}

fn source_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

fn generated_at() -> Result<String> {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .context("failed to format the current timestamp")
}
