//! Rendering of analysis reports into output artifacts

mod html;

use std::path::{Path, PathBuf};

use crate::analysis::Report;
use crate::domain::Library;
pub use html::HtmlExporter;

/// Metadata used by exporters for the report header
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportMeta {
    pub title: String,
    /// Display name of the parsed source artifact
    pub source_name: String,
    /// Preformatted generation timestamp
    pub generated_at: String,
}

/// Errors from preparing or writing report artifacts
#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    /// No exporter is registered under the requested id
    #[error("unknown export format `{0}`")]
    UnknownFormat(String),
    /// The output directory could not be created
    #[error("failed to create output directory `{path}`: {source}")]
    CreateDir {
        /// Directory that could not be created
        path: PathBuf,
        /// Underlying I/O error
        #[source]
        source: std::io::Error,
    },
    /// An artifact could not be written
    #[error("failed to write `{path}`: {source}")]
    Write {
        /// File that could not be written
        path: PathBuf,
        /// Underlying I/O error
        #[source]
        source: std::io::Error,
    },
}

/// An output format that owns its artifact layout and file naming
pub trait Exporter {
    /// Stable identifier used by the CLI
    fn id(&self) -> &'static str;
    /// Human-readable title
    fn title(&self) -> &'static str;
    /// Writes the report artifacts into `output_dir`
    ///
    /// # Errors
    ///
    /// Returns [`WriteError`] when an artifact cannot be written
    fn export(
        &self,
        report: &Report,
        library: &Library,
        meta: &ReportMeta,
        output_dir: &Path,
    ) -> Result<(), WriteError>;
}

/// The exporters available by default
#[must_use]
pub fn default_exporters() -> Vec<Box<dyn Exporter>> {
    vec![Box::new(HtmlExporter)]
}

/// Looks up an exporter by id
///
/// # Errors
///
/// Returns [`WriteError::UnknownFormat`] when no exporter uses the id
pub fn exporter(id: &str) -> Result<Box<dyn Exporter>, WriteError> {
    default_exporters()
        .into_iter()
        .find(|exporter| exporter.id() == id)
        .ok_or_else(|| WriteError::UnknownFormat(id.to_owned()))
}

#[cfg(test)]
mod tests;
