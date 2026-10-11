//! Library-wide analyses and the registry that runs them

mod alternative_versions;
mod duplicate_songs;
mod report;
mod same_title_different_album;

use crate::domain::Library;
pub use report::{
    AlternativeVersions, DuplicateEntries, DuplicateOccurrence, Finding, Report,
    SameTitleDifferentAlbum,
};

/// A library-wide check that produces findings
pub trait Analysis: Send + Sync {
    /// Stable identifier used by the CLI and the report
    fn id(&self) -> &'static str;
    /// Human-readable title used in the report
    fn title(&self) -> &'static str;
    /// Runs the analysis and returns its findings
    fn analyze(&self, library: &Library) -> Vec<Finding>;
}

/// The analyses run by default
#[must_use]
pub fn default_analyses() -> Vec<Box<dyn Analysis>> {
    vec![
        Box::new(duplicate_songs::DuplicateSongs),
        Box::new(same_title_different_album::SameTitleDifferentAlbum),
        Box::new(alternative_versions::AlternativeVersions),
    ]
}

/// Runs every analysis and collects the findings
#[must_use]
pub fn run(library: &Library, analyses: &[Box<dyn Analysis>]) -> Report {
    let findings: Vec<Finding> = analyses
        .iter()
        .flat_map(|analysis| analysis.analyze(library))
        .collect();
    Report {
        total_findings: findings.len(),
        findings,
    }
}

#[cfg(test)]
mod tests;
