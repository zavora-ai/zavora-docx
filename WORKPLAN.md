# zavora-docx Feature Completion Workplan

## Goal
Implement 8 missing DOCX features in zavora-docx (rdocx-oxml + zavora-docx high-level API), then expose them as MCP tools in docx-mcp-server.

## Verification
Each feature produces a test .docx file in `/tmp/zavora-docx-tests/` that can be opened in Word/LibreOffice to visually confirm.

---

## Feature 1: Footnotes & Endnotes
- **OXML**: Already parsed in `rdocx-oxml/src/footnotes.rs`
- **High-level API needed**:
  - `doc.add_footnote(text) -> i32` — returns footnote ID
  - `para.add_footnote_ref(id)` or `run.footnote_ref(id)` — inserts superscript reference
  - `doc.footnotes() -> Vec<FootnoteRef>` — read existing
- **Test file**: Document with body text containing footnote references, footnotes at bottom

## Feature 2: Hyperlinks
- **OXML**: Need `w:hyperlink` element wrapping runs, plus relationship in `word/_rels/document.xml.rels`
- **High-level API needed**:
  - `para.add_hyperlink(url, text)` — external URL
  - `para.add_internal_link(bookmark_name, text)` — internal bookmark link
  - Read: already have `doc.links()`
- **Test file**: Document with clickable external links and internal cross-references

## Feature 3: Bookmarks
- **OXML**: `w:bookmarkStart` (id, name) + `w:bookmarkEnd` (id) elements in body
- **High-level API needed**:
  - `para.add_bookmark(name)` — marks this paragraph as a bookmark target
  - `doc.bookmarks() -> Vec<(String, usize)>` — list bookmark names and positions
- **Test file**: Document with named bookmarks and internal links pointing to them

## Feature 4: Comments
- **OXML**: `word/comments.xml` part + `w:commentRangeStart`/`w:commentRangeEnd` + `w:r/w:commentReference` in body
- **High-level API needed**:
  - `doc.add_comment(author, text) -> i32` — returns comment ID
  - `para.comment_range_start(id)` / `para.comment_range_end(id)` — mark commented text
  - `doc.comments() -> Vec<Comment>` — read existing
- **Test file**: Document with highlighted commented text, comment bubbles visible in Word

## Feature 5: Watermarks
- **OXML**: VML shape (`v:shape`) in default header with `w:pict` wrapper, or DrawingML
- **High-level API needed**:
  - `doc.set_text_watermark(text, color, rotation)` — diagonal text watermark (e.g. "DRAFT", "CONFIDENTIAL")
  - `doc.set_image_watermark(image_data, filename)` — image watermark
  - `doc.remove_watermark()`
- **Test file**: Document with "DRAFT" diagonal watermark visible on every page

## Feature 6: Track Changes (Read + Basic Write)
- **OXML**: `w:ins` (insertion), `w:del` (deletion), `w:rPrChange` (format change) elements
- **High-level API needed**:
  - `doc.enable_track_changes(author)`
  - `doc.insert_tracked(index, text, author)` — insert with revision mark
  - `doc.delete_tracked(index, author)` — delete with revision mark (strikethrough)
  - `doc.revisions() -> Vec<Revision>` — read existing revisions
- **Test file**: Document showing red insertions and strikethrough deletions in review mode

## Feature 7: Form Fields
- **OXML**: Legacy form fields (`w:fldChar` + `w:ffData`) or content controls (`w:sdt`)
- **High-level API needed**:
  - `para.add_text_field(name, default_value)` — text input field
  - `para.add_checkbox(name, checked)` — checkbox
  - `para.add_dropdown(name, options, selected)` — dropdown list
  - `doc.form_fields() -> Vec<FormField>` — read existing
- **Test file**: Document with fillable form (text boxes, checkboxes, dropdown)

## Feature 8: Document Protection
- **OXML**: `w:documentProtection` element in `word/settings.xml`
- **High-level API needed**:
  - `doc.protect_readonly(password)` — prevent editing
  - `doc.protect_forms_only(password)` — only form fields editable
  - `doc.protect_comments_only(password)` — only comments allowed
  - `doc.unprotect()`
  - `doc.is_protected() -> bool`
- **Test file**: Protected document that prompts for password when trying to edit in Word

---

## Implementation Order
1. Footnotes (OXML already done, just wire up)
2. Hyperlinks (small OXML addition + relationship)
3. Bookmarks (small OXML addition)
4. Comments (new XML part + body markers)
5. Watermarks (VML in header)
6. Track Changes (revision elements)
7. Form Fields (legacy fldChar approach)
8. Document Protection (settings.xml)

## After all 8:
- Add corresponding MCP tools to docx-mcp-server
- Generate a single showcase document demonstrating all features
- Bump zavora-docx to 0.2.0
