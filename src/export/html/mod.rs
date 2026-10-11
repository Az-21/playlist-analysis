//! Static HTML output

mod escape;
mod render;
mod style;

use std::path::Path;

use crate::analysis::Report;
use crate::domain::Library;
use crate::export::{Exporter, ReportMeta, WriteError};

/// Writes a single self-contained `index.html` into the output directory
#[derive(Debug, Default, Clone, Copy)]
pub struct HtmlExporter;

impl Exporter for HtmlExporter {
    fn id(&self) -> &'static str {
        "html"
    }

    fn title(&self) -> &'static str {
        "Static HTML"
    }

    fn export(
        &self,
        report: &Report,
        library: &Library,
        meta: &ReportMeta,
        output_dir: &Path,
    ) -> Result<(), WriteError> {
        std::fs::create_dir_all(output_dir).map_err(|source| WriteError::CreateDir {
            path: output_dir.to_path_buf(),
            source,
        })?;
        let path = output_dir.join("index.html");
        let html = render::render(report, library, meta);
        std::fs::write(&path, html).map_err(|source| WriteError::Write { path, source })
    }
}

#[cfg(test)]
mod tests;
