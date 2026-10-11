//! Pure rendering of an analysis report into a static HTML document

use std::collections::BTreeSet;

use crate::analysis::{
    AlternativeVersions, DuplicateEntries, Finding, Report, SameTitleDifferentAlbum,
};
use crate::domain::{AlbumId, ArtistId, Library, Song, SongId};
use crate::export::ReportMeta;

use super::escape::escape_html;
use super::style::CSS;

/// Renders the complete, self-contained HTML document
pub(super) fn render(report: &Report, library: &Library, meta: &ReportMeta) -> String {
    let mut html = String::with_capacity(16_384);
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"utf-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    push_html(
        &mut html,
        format_args!("<title>{}</title>\n", escape_html(&meta.title)),
    );
    push_html(&mut html, format_args!("<style>{CSS}</style>\n"));
    html.push_str("</head>\n<body>\n");
    html.push_str(&header(meta, library));
    html.push_str("<main>\n");
    html.push_str(&summary(report, library));
    html.push_str(&album_variants_section(report, library));
    html.push_str(&alternative_versions_section(report, library));
    html.push_str(&duplicate_sections(report, library));
    html.push_str("</main>\n");
    html.push_str("</body>\n</html>\n");
    html
}

fn header(meta: &ReportMeta, library: &Library) -> String {
    format!(
        concat!(
            "<header>\n",
            "<h1>{title}</h1>\n",
            "<p class=\"meta\">Source: <span class=\"mono\">{source}</span> | Generated: <span class=\"mono\">{generated}</span></p>\n",
            "<p class=\"meta\">Library: {songs} songs, {albums} albums, {artists} artists, {playlists} playlists</p>\n",
            "</header>\n",
        ),
        title = escape_html(&meta.title),
        source = escape_html(&meta.source_name),
        generated = escape_html(&meta.generated_at),
        songs = library.songs.len(),
        albums = library.albums.len(),
        artists = library.artists.len(),
        playlists = library.playlists.len(),
    )
}

fn summary(report: &Report, library: &Library) -> String {
    let duplicates = duplicate_findings(report);
    let playlists_with_duplicates: BTreeSet<_> = duplicates
        .iter()
        .map(|duplicate| duplicate.playlist)
        .collect();
    let cards = [
        card("Playlists analyzed", &library.playlists.len().to_string()),
        card("Songs", &library.songs.len().to_string()),
        card(
            "Playlists with duplicates",
            &playlists_with_duplicates.len().to_string(),
        ),
        card("Duplicate findings", &duplicates.len().to_string()),
        card("Album variants", &album_variants(report).len().to_string()),
        card(
            "Alternative versions",
            &alternative_versions(report).len().to_string(),
        ),
    ];
    format!("<section class=\"cards\">\n{}</section>\n", cards.join(""))
}

fn card(label: &str, value: &str) -> String {
    format!(
        "<div class=\"card\"><span class=\"value\">{}</span><span class=\"label\">{}</span></div>\n",
        escape_html(value),
        escape_html(label)
    )
}

fn album_variants_section(report: &Report, library: &Library) -> String {
    let variants = album_variants(report);
    let mut html = String::from("<section>\n<h2>Same Title, Different Album</h2>\n");
    if variants.is_empty() {
        html.push_str("<p class=\"empty\">No songs share a title and artist across different albums.</p>\n</section>\n");
        return html;
    }
    html.push_str(
        "<table>\n<thead><tr><th>Title</th><th>Artist</th><th>Albums</th></tr></thead>\n<tbody>\n",
    );
    for variant in variants {
        html.push_str(&variant_row(variant, library));
    }
    html.push_str("</tbody>\n</table>\n</section>\n");
    html
}

fn alternative_versions_section(report: &Report, library: &Library) -> String {
    let alternatives = alternative_versions(report);
    let mut html = String::from("<section>\n<h2>Alternative Versions</h2>\n");
    if alternatives.is_empty() {
        html.push_str("<p class=\"empty\">No alternative versions of the same song were found.</p>\n</section>\n");
        return html;
    }
    html.push_str("<table>\n<thead><tr><th>Title</th><th>Artist</th><th>Album</th><th>Playlists</th></tr></thead>\n<tbody>\n");
    for (index, alternative) in alternatives.iter().enumerate() {
        let mut starts_group = index > 0;
        for song_id in &alternative.songs {
            let Some(song) = library.song(*song_id) else {
                continue;
            };
            html.push_str(&alternative_row(song, library, starts_group));
            starts_group = false;
        }
    }
    html.push_str("</tbody>\n</table>\n</section>\n");
    html
}

fn alternative_row(song: &Song, library: &Library, starts_group: bool) -> String {
    let class = if starts_group { " class=\"group\"" } else { "" };
    format!(
        "<tr{class}><td>{title}</td><td>{artist}</td><td>{album}</td><td>{playlists}</td></tr>\n",
        class = class,
        title = escape_html(&song.title),
        artist = escape_html(artist_name(library, song.artist)),
        album = escape_html(album_title(library, song.album)),
        playlists = playlists_cell(song.id, library),
    )
}

fn playlists_cell(song_id: SongId, library: &Library) -> String {
    let mut html = String::from("<ul class=\"tight\">");
    for playlist in &library.playlists {
        if playlist.entries.contains(&song_id) {
            push_html(
                &mut html,
                format_args!("<li>{}</li>", escape_html(&playlist.name)),
            );
        }
    }
    html.push_str("</ul>");
    html
}

fn variant_row(variant: &SameTitleDifferentAlbum, library: &Library) -> String {
    format!(
        concat!(
            "<tr><td>{title}</td><td>{artist}</td>",
            "<td>{albums}</td></tr>\n",
        ),
        title = escape_html(&variant.title),
        artist = escape_html(artist_name(library, variant.artist)),
        albums = album_listings(variant, library),
    )
}

fn album_listings(variant: &SameTitleDifferentAlbum, library: &Library) -> String {
    let variant_songs: BTreeSet<SongId> = variant.songs.iter().copied().collect();
    let mut html = String::from("<ul class=\"tight\">");
    for album in &variant.albums {
        push_html(
            &mut html,
            format_args!("<li>{}", escape_html(album_title(library, *album))),
        );
        html.push_str("<ul class=\"tight\">");
        let mut any = false;
        for playlist in &library.playlists {
            let contains = playlist.entries.iter().any(|song_id| {
                variant_songs.contains(song_id)
                    && library
                        .song(*song_id)
                        .is_some_and(|song| song.album == *album)
            });
            if contains {
                any = true;
                push_html(
                    &mut html,
                    format_args!("<li>{}</li>", escape_html(&playlist.name)),
                );
            }
        }
        if !any {
            html.push_str("<li>(none)</li>");
        }
        html.push_str("</ul></li>");
    }
    html.push_str("</ul>");
    html
}

fn duplicate_sections(report: &Report, library: &Library) -> String {
    let duplicates = duplicate_findings(report);
    let mut html = String::from("<section>\n<h2>Duplicate Songs</h2>\n");
    if duplicates.is_empty() {
        html.push_str(
            "<p class=\"empty\">No playlist contains the same song twice.</p>\n</section>\n",
        );
        return html;
    }
    for playlist in &library.playlists {
        let findings: Vec<&DuplicateEntries> = duplicates
            .iter()
            .copied()
            .filter(|duplicate| duplicate.playlist == playlist.id)
            .collect();
        if findings.is_empty() {
            continue;
        }
        push_html(
            &mut html,
            format_args!("<h3>{}</h3>\n", escape_html(&playlist.name)),
        );
        push_html(
            &mut html,
            format_args!(
                "<p class=\"meta\">{} entries | {} duplicate findings</p>\n",
                playlist.entries.len(),
                findings.len()
            ),
        );
        html.push_str("<table>\n<thead><tr><th>Title</th><th>Artist</th><th>Album</th><th class=\"num\">Count</th><th>Positions</th><th>Track IDs</th></tr></thead>\n<tbody>\n");
        for duplicate in findings {
            html.push_str(&duplicate_row(duplicate, library));
        }
        html.push_str("</tbody>\n</table>\n");
    }
    html.push_str("</section>\n");
    html
}

fn duplicate_row(duplicate: &DuplicateEntries, library: &Library) -> String {
    let positions = duplicate
        .occurrences
        .iter()
        .map(|occurrence| (occurrence.position + 1).to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let songs = duplicate
        .occurrences
        .iter()
        .map(|occurrence| occurrence.song.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        concat!(
            "<tr><td>{title}</td><td>{artist}</td><td>{album}</td>",
            "<td class=\"num\">{count}</td>",
            "<td class=\"mono\">{positions}</td><td class=\"mono\">{songs}</td></tr>\n",
        ),
        title = escape_html(&duplicate.title),
        artist = escape_html(artist_name(library, duplicate.artist)),
        album = escape_html(album_title(library, duplicate.album)),
        count = duplicate.occurrences.len(),
        positions = positions,
        songs = songs,
    )
}

fn duplicate_findings(report: &Report) -> Vec<&DuplicateEntries> {
    report
        .findings
        .iter()
        .filter_map(|finding| match finding {
            Finding::DuplicateEntries(duplicate) => Some(duplicate),
            Finding::SameTitleDifferentAlbum(_) | Finding::AlternativeVersions(_) => None,
        })
        .collect()
}

fn album_variants(report: &Report) -> Vec<&SameTitleDifferentAlbum> {
    report
        .findings
        .iter()
        .filter_map(|finding| match finding {
            Finding::SameTitleDifferentAlbum(variant) => Some(variant),
            Finding::DuplicateEntries(_) | Finding::AlternativeVersions(_) => None,
        })
        .collect()
}

fn alternative_versions(report: &Report) -> Vec<&AlternativeVersions> {
    report
        .findings
        .iter()
        .filter_map(|finding| match finding {
            Finding::AlternativeVersions(alternative) => Some(alternative),
            Finding::DuplicateEntries(_) | Finding::SameTitleDifferentAlbum(_) => None,
        })
        .collect()
}

fn album_title(library: &Library, id: AlbumId) -> &str {
    library
        .album(id)
        .map_or("(unknown album)", |album| album.title.as_str())
}

fn artist_name(library: &Library, id: ArtistId) -> &str {
    library
        .artist(id)
        .map_or("(unknown artist)", |artist| artist.name.as_str())
}

/// Appends formatted text to the output
///
/// Writing to a `String` cannot fail
fn push_html(output: &mut String, args: std::fmt::Arguments<'_>) {
    std::fmt::Write::write_fmt(output, args).expect("writing to a String cannot fail");
}

#[cfg(test)]
mod tests;
