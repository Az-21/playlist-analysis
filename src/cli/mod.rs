//! Command-line interface definitions

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use playlist_analysis::parser::{LibraryFormat, Service};

/// Analyze a music library export and render a static report
#[derive(Debug, Parser)]
#[command(name = "playlist-analysis", version)]
pub struct Args {
    /// Path to the library export artifact to analyze
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

    /// Directory the exporter writes its artifacts into, created if missing
    #[arg(long, value_name = "DIR", default_value = "report")]
    pub output_dir: PathBuf,

    /// Export format id
    #[arg(long, value_name = "FORMAT", default_value = "html")]
    pub output_format: String,

    /// Source service, otherwise detected from the file content
    #[arg(long, value_name = "SERVICE")]
    pub service: Option<ServiceArg>,

    /// Source artifact format, otherwise detected from the file content
    #[arg(long, value_name = "FORMAT")]
    pub input_format: Option<FormatArg>,

    /// Enable debug-level logs
    #[arg(long)]
    pub verbose: bool,
}

/// Supported source services
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ServiceArg {
    /// Apple Music official export
    AppleMusic,
}

/// Supported source artifact formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FormatArg {
    /// XML property list
    Xml,
}

impl From<ServiceArg> for Service {
    fn from(value: ServiceArg) -> Self {
        match value {
            ServiceArg::AppleMusic => Self::AppleMusic,
        }
    }
}

impl From<FormatArg> for LibraryFormat {
    fn from(value: FormatArg) -> Self {
        match value {
            FormatArg::Xml => Self::Xml,
        }
    }
}
