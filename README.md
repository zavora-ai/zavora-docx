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

## Why pure Rust wins

Compared to the common alternatives, a native Rust engine avoids the heavy runtimes and external tools they depend on:

- **vs. python-docx** — the most popular DOCX library anywhere (~14M downloads/month), but it has *zero* conversion capability. PDF requires bolting on LibreOffice (~500 MB). zavora-docx gives you the same read/write API *plus* built-in PDF/HTML/Markdown in a single ~4 MB binary.
- **vs. Java (Apache POI / docx4j / Aspose)** — the JVM alone costs 50-100 MB of RAM and 2-5 s cold starts; POI has no built-in PDF; Aspose's high-fidelity PDF costs $1,199+ per developer.
- **vs. other Rust crates (docx-rs, docx-rust, ooxmlsdk)** — these are read/write only. zavora-docx is the only Rust crate combining DOCX read/write with a built-in layout engine and multi-format output (PDF, HTML, Markdown, PNG).

| Metric | zavora-docx (native) | Python + LibreOffice | Java (POI + FOP) |
|---|---|---|---|
| Binary / install size | **~4 MB** | ~500 MB | ~250 MB (JARs + JRE) |
| Memory (moderate document) | **10-50 MB** | ~200-500 MB | ~300 MB - 1.5 GB |
| Cold start | **< 10 ms** | ~2-4 sec | ~2-5 sec |
| Serverless / Lambda friendly | Yes | Difficult | Difficult |
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
