# zavora-docx

[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![MSRV: 1.93](https://img.shields.io/badge/MSRV-1.93-blue.svg)](https://blog.rust-lang.org/2026/01/09/Rust-1.93.0.html)

A pure Rust DOCX library — create, read, and modify Word documents programmatically. Additionally, render pixel-identical PDFs and export to HTML and Markdown, all from the same document object. No LibreOffice, no unoconv, no C dependencies.

> **Credit:** zavora-docx began as a fork of [**rdocx**](https://github.com/tensorbee/rdocx) by **Atul Sharma** (MIT/Apache-2.0). The original project's pure-Rust DOCX engine, layout, and multi-format rendering are its foundation. zavora-docx extends it with expanded OOXML coverage and a library of parameterized business document templates. Sincere thanks to the original author.

## Why zavora-docx?

Most DOCX solutions in the ecosystem shell out to LibreOffice or wrap C/C++ libraries. zavora-docx is written entirely in Rust, so it compiles to a single binary with zero runtime dependencies. It works everywhere Rust does — including WASM.

The core focus is **DOCX**: a high-level, python-docx-inspired API for building and editing Word documents with paragraphs, tables, images, headers/footers, styles, and lists. On top of that, zavora-docx includes a built-in layout engine that paginates your document and can render it to **PDF** (with font subsetting, bookmarks, and selectable text) or export to **HTML** and **Markdown** — so you get faithful output in every format without leaving Rust.

## DOCX Features

- **Read & write** DOCX files with a high-level API
- **Tables** with merged cells, borders, shading, non-uniform column widths, and content-based sizing
- **Images** — inline and anchored, with header/footer background images
- **Headers & footers** with first-page support and per-section overrides
- **Styles** — paragraph and character styles, theme color resolution
- **Lists** with automatic numbering ID management
- **Math, charts, shapes, content controls, footnotes, bookmarks** and other rich OOXML constructs
- **Font embedding, glossary/building blocks, custom XML** parts
- **Template engine** with placeholder replacement (plain text and regex)
- **TOC generation** with internal hyperlinks and dot-leader tabs
- **Document merging** with style deduplication and numbering remapping

## Output Formats

- **PDF** — built-in layout engine with text shaping (rustybuzz), Unicode line breaking, multi-section pagination, font subsetting, ToUnicode CMap, bookmarks, and images
- **HTML** — semantic mapping from OOXML with CSS styling and base64-embedded images
- **Markdown** — GFM-compatible output with pipe tables and formatting
- **PNG** — page-to-image rendering via tiny-skia rasterizer

## Extras

- **WASM support** via standalone `zavora-docx-wasm` crate
- **CLI tool** (`zavora-docx-cli`, `rdocx` binary) — inspect, convert, diff, replace, validate, render

## Installation

```toml
[dependencies]
zavora-docx = "0.1"
```

To include bundled metric-compatible fonts (Carlito, Caladea, Liberation family):

```toml
[dependencies]
zavora-docx-layout = { version = "0.1", features = ["bundled-fonts"] }
```

## Quick Start

### Create a document

```rust
use zavora_docx::{Document, Length};

let mut doc = Document::new();

doc.add_paragraph("Hello, World!");

let mut para = doc.add_paragraph("");
para.add_run("Bold text").bold(true);
para.add_run(" and ");
para.add_run("italic text").italic(true);

doc.add_table(3, 4);

doc.save("output.docx").unwrap();
```

### Read a document

```rust
use zavora_docx::Document;

let doc = Document::open("report.docx").unwrap();

for para in doc.paragraphs() {
    println!("{}", para.text());
}

for table in doc.tables() {
    for row in table.rows() {
        for cell in row.cells() {
            print!("{}\t", cell.text());
        }
        println!();
    }
}
```

### Convert to PDF

```rust
use zavora_docx::Document;

let doc = Document::open("report.docx").unwrap();
doc.save_pdf("report.pdf").unwrap();

// Or get bytes directly
let pdf_bytes = doc.to_pdf().unwrap();
```

### Convert to HTML / Markdown

```rust
use zavora_docx::Document;

let doc = Document::open("report.docx").unwrap();

let html = doc.to_html();
let markdown = doc.to_markdown();
```

### Template replacement

```rust
use zavora_docx::Document;
use std::collections::HashMap;

let mut doc = Document::open("template.docx").unwrap();

let mut replacements = HashMap::new();
replacements.insert("{{name}}", "Jane Doe");
replacements.insert("{{date}}", "2025-01-15");
doc.replace_all(&replacements);

doc.save("filled.docx").unwrap();
```

### Merge documents

```rust
use zavora_docx::{Document, SectionBreak};

let mut doc = Document::open("part1.docx").unwrap();
let part2 = Document::open("part2.docx").unwrap();

doc.append_with_break(&part2, SectionBreak::NextPage);
doc.save("combined.docx").unwrap();
```

## CLI

Install the CLI:

```sh
cargo install zavora-docx-cli
```

The installed binary is named `rdocx`:

```sh
# Inspect document structure
rdocx inspect report.docx

# Extract plain text
rdocx text report.docx

# Convert to PDF
rdocx convert report.docx -o report.pdf

# Convert to HTML or Markdown
rdocx convert report.docx -o report.html
rdocx convert report.docx -o report.md

# Find and replace text
rdocx replace report.docx --find "Draft" --replace "Final" -o final.docx

# Diff two documents
rdocx diff v1.docx v2.docx
```

## How zavora-docx Compares

Most DOCX solutions shell out to LibreOffice or wrap C/C++/Java libraries. zavora-docx is a single native binary with zero runtime dependencies, and — beyond the original engine — now ships expanded OOXML coverage (math, charts, shapes, content controls, footnotes, bookmarks, font embedding, glossary/building blocks, custom XML) and non-uniform table column widths.

### vs. Python Libraries

| | zavora-docx | python-docx | docx2pdf | pypandoc |
|---|---|---|---|---|
| Create DOCX | Yes | Yes | -- | -- |
| Read DOCX | Yes | Yes | -- | -- |
| DOCX to PDF | Yes (built-in) | No | Via MS Word | Via Pandoc + LaTeX |
| DOCX to HTML | Yes (built-in) | No | No | Yes (lossy) |
| DOCX to Markdown | Yes (built-in) | No | No | Yes (lossy) |
| Math / charts / shapes | Yes | No | -- | -- |
| Layout engine | Yes | None | Delegates to Word | Delegates to LaTeX |
| External runtime | **None** | None (but no PDF) | **MS Word required** | **Pandoc + LaTeX** |
| Install size | **~4 MB binary** | ~5 MB | ~31 KB + Word | 300-650 MB |
| Runs in Docker / CI | Yes | Yes (no PDF) | No | Yes (huge image) |
| WASM / browser | Yes | No | No | No |

**python-docx** is the most popular DOCX library in any language (~14M downloads/month), but it has **zero conversion capabilities** — no PDF, no HTML, no Markdown. Users who need PDF must bolt on LibreOffice (~500 MB) or a commercial API. zavora-docx gives you the same read/write API *plus* built-in conversion in a single ~4 MB binary.

#### API parity with python-docx

zavora-docx now matches python-docx's full read/write API surface — including the corners python-docx is known for — and adds capabilities it has no equivalent for.

| python-docx feature | zavora-docx equivalent |
|---|---|
| `run.add_break(WD_BREAK.*)` | `Run::add_break(BreakKind::{Line,Page,Column})` |
| `run.add_picture(...)` | `Run::add_picture(rel_id, w, h)` (inline, in-run) |
| `document.core_properties.*` (all fields) | full `CoreProperties` + `set_category/content_status/identifier/language/revision/version/last_printed` |
| `document.sections[]` | `Document::sections()` / `section_count()` |
| `document.styles` collection + `style.base_style` | `style()/style_by_name()/styles_of_type()/default_style()` + `style_base_chain()` inheritance traversal |
| paragraph/run/table formatting | full parity (alignment, indents, spacing, borders, shading, fonts, colors, …) |

Beyond parity, zavora-docx authors constructs python-docx cannot — math/OMML, charts, shapes, content controls, footnotes/endnotes, comments + threaded replies, bookmarks, fields, watermarks, document protection, font embedding, building blocks, custom XML — and renders to PDF/HTML/Markdown/PNG, none of which python-docx offers.

The remaining difference is **ecosystem maturity**, not capability: python-docx has years of documentation, examples, and ~14M downloads/month. zavora-docx is the broader library; python-docx is the more widely known one.

### vs. Java Libraries

| | zavora-docx | Apache POI | docx4j | Aspose.Words |
|---|---|---|---|---|
| Create DOCX | Yes | Yes | Yes | Yes |
| Read DOCX | Yes | Yes | Yes | Yes |
| PDF (built-in) | Yes | No | Via FOP (limited) | Yes (high fidelity) |
| HTML (built-in) | Yes | No | Yes | Yes |
| License | MIT / Apache-2.0 | Apache-2.0 | Apache-2.0 | **$1,199+** |
| Total dependency size | **~4 MB** | 18-28 MB + JRE | 50-80 MB + JRE | 14 MB + JRE |
| Typical memory (moderate doc) | **10-50 MB** | 256 MB - 1 GB | 256 MB - 2 GB | 50-300 MB |
| Cold start | **< 10 ms** | 2-5 sec | 2-5 sec | 2-5 sec |
| Runtime required | None | JVM (~200 MB) | JVM (~200 MB) | JVM (~200 MB) |

Java solutions carry the JVM's baseline overhead: 50-100 MB of RAM before a single document is loaded, and 2-5 second cold starts from class loading. Apache POI has **no built-in PDF** at all. docx4j's FOP pipeline is acknowledged by its own maintainer as limited in fidelity. Aspose has excellent PDF output but costs $1,199+ per developer.

### vs. Other Rust Crates

| | zavora-docx | docx-rs | docx-rust | ooxmlsdk |
|---|---|---|---|---|
| Create DOCX | Yes | Yes | Yes | Low-level |
| Read DOCX | Yes | Yes | Yes | Low-level |
| Round-trip preservation | Yes | Limited | Limited | N/A |
| Tables, images, headers | Yes | Yes | Basic | Raw XML |
| Non-uniform column widths | **Yes** | Limited | No | Raw XML |
| Math / charts / shapes | **Yes** | No | No | Raw XML |
| Content controls / footnotes / bookmarks | **Yes** | Partial | No | Raw XML |
| Font embedding / custom XML | **Yes** | No | No | Raw XML |
| PDF conversion | **Yes** | No | No | No |
| HTML / Markdown export | **Yes** | No | No | No |
| Layout engine | **Yes** | No | No | No |
| Page-to-image rendering | **Yes** | No | No | No |
| Template engine | **Yes** | No | No | No |
| Document merging | **Yes** | No | No | No |
| Regex find/replace | **Yes** | No | No | No |
| CLI tool | **Yes** | No | No | No |
| WASM | Yes | Yes | No | No |

**docx-rs** (1M+ downloads, 500+ stars) is the most popular Rust DOCX crate, but it is a read/write library only — no conversion, no layout engine, no PDF. The same is true for every other Rust DOCX crate. zavora-docx is the only Rust crate that combines DOCX read/write with a built-in layout engine, rich OOXML constructs, and multi-format output (PDF, HTML, Markdown, PNG).

### Resource Footprint

| Metric | zavora-docx (native) | Python + LibreOffice | Java (POI + FOP) |
|---|---|---|---|
| Binary / install size | **~4 MB** | ~500 MB | ~250 MB (JARs + JRE) |
| Memory (moderate document) | **10-50 MB** | ~200-500 MB | ~300 MB - 1.5 GB |
| Cold start | **< 10 ms** | ~2-4 sec (LibreOffice) | ~2-5 sec (JVM) |
| Serverless / Lambda friendly | Yes | Difficult | Difficult |
| Docker image overhead | **~10 MB** (musl static) | ~500 MB+ | ~250 MB+ |
| WASM compatible | Yes | No | No |

## Crate Architecture

| Crate | Purpose |
|---|---|
| `zavora-docx` | High-level Document API |
| `zavora-docx-opc` | OPC/ZIP package I/O |
| `zavora-docx-oxml` | OOXML types (CT_Document, CT_PPr, CT_RPr, CT_Tbl, ...) |
| `zavora-docx-layout` | Layout engine (text shaping, line breaking, pagination) |
| `zavora-docx-pdf` | PDF rendering with font subsetting |
| `zavora-docx-html` | HTML and Markdown conversion |
| `zavora-docx-cli` | CLI binary (`rdocx`) |
| `zavora-docx-wasm` | WASM bindings (standalone, excluded from workspace) |

## Minimum Supported Rust Version

1.93 (edition 2024)

## License

Licensed under either of

- MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## Acknowledgments

Built on the foundation of [**rdocx**](https://github.com/tensorbee/rdocx) by **Atul Sharma**, whose pure-Rust DOCX, layout, and rendering engine made this project possible.
