//! Embedded stylesheet for the HTML report

/// The complete stylesheet, inlined into the report so the artifact is self-contained and has no external assets
pub(super) const CSS: &str = r#"
:root {
  color-scheme: light dark;
  --bg: #f7f7f8;
  --fg: #1c1c1e;
  --muted: #6b6b70;
  --card: #ffffff;
  --border: #e3e3e8;
}
@media (prefers-color-scheme: dark) {
  :root {
    --bg: #141416;
    --fg: #f2f2f4;
    --muted: #9a9aa0;
    --card: #1e1e21;
    --border: #2c2c31;
  }
}
* { box-sizing: border-box; }
body {
  margin: 0;
  font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
  background: var(--bg);
  color: var(--fg);
  line-height: 1.5;
}
header, main { width: 91.666667%; margin: 0 auto; padding: 1.25rem 1rem; }
h1 { font-size: 1.6rem; margin: 0 0 0.5rem; }
h2 { font-size: 1.25rem; border-bottom: 1px solid var(--border); padding-bottom: 0.35rem; margin-top: 2rem; }
h3 { font-size: 1.05rem; margin: 1.5rem 0 0.25rem; }
.meta { color: var(--muted); font-size: 0.85rem; margin: 0.15rem 0; }
.mono { font-family: ui-monospace, "SF Mono", Consolas, monospace; font-size: 0.85em; }
.cards { display: flex; flex-wrap: wrap; gap: 0.75rem; padding: 0.5rem 0; }
.card {
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 0.75rem 1rem;
  min-width: 150px;
  display: flex;
  flex-direction: column;
}
.card .value { font-size: 1.4rem; font-weight: 600; }
.card .label { color: var(--muted); font-size: 0.8rem; }
table {
  width: 100%;
  border-collapse: collapse;
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: 8px;
  overflow: hidden;
  margin: 0.5rem 0 1.5rem;
}
th, td { text-align: left; padding: 0.5rem 0.75rem; border-bottom: 1px solid var(--border); vertical-align: top; font-size: 0.9rem; }
th { font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.04em; color: var(--muted); }
tr:last-child td { border-bottom: none; }
tr.group td { border-top: 2px solid var(--border); }
ul.tight { margin: 0; padding-left: 1.1rem; }
.empty { color: var(--muted); font-style: italic; }
.num { text-align: right; }
"#;
