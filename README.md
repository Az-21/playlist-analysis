# Playlist Analysis

Analyze a music library export and render a static HTML report. The tool reads
local export artifacts only; it never contacts a streaming service, so you
produce the artifact yourself with the service's own export feature.

Currently supported: Apple Music official XML library export.

## Usage

```sh
cargo run --release -- input/sample.xml
```

This parses the export, runs the analyses, and writes a self-contained report
to `report/index.html`.

```text
playlist-analysis <INPUT> [--output-dir <DIR>] [--output-format <FORMAT>] [--service <SERVICE>] [--input-format <FORMAT>] [--verbose]
```

- `INPUT`: path to the library export artifact.
- `--output-dir`: directory the exporter writes into, default `report/`.
- `--output-format`: exporter id, default `html`.
- `--service` and `--input-format`: optional source override, otherwise detected from the file content.
- `--verbose`: debug-level logs.

## Analyses

- Duplicate songs within a playlist: the same track id repeated, or distinct
  track ids sharing title, artist, and album.
- Same title, different album: a library-wide database of every song
  referenced by a playlist, grouped by normalized title and artist.
- Alternative versions: songs referenced by a playlist whose title, artist, and
  album are all fuzzily similar after decoration stripping (remaster, live,
  deluxe, and similar tags), clustered into groups of possible versions.

## Development

```sh
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

See `PLAN.md` for the architecture and design decisions.
