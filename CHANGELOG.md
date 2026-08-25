# Changelog

## [0.1.4] - 2026-08-25

### Security

- Upgraded `quick-xml` to 0.41 to close the namespace-allocation and duplicate-
  attribute denial-of-service advisories.
- Migrated text and entity-reference handling to the hardened parser event model
  without losing escaped document-property content.

## [0.1.3] - 2026-08-14

### Added
- First coordinated Zavora release of the complete DOCX engine stack: OPC,
  OOXML, high-level document APIs, layout, PDF/PNG rendering, HTML/Markdown
  conversion, and the `rdocx` CLI.
- Preserved standalone WebAssembly bindings as an explicitly non-published
  package.

### Changed
- Raised the workspace MSRV to Rust 1.94.1.
- Moved release metadata and automation to the owned
  `zavora-ai/zavora-docx` repository while retaining upstream attribution.
