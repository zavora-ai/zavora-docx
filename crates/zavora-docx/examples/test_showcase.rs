use zavora_docx::Document;

fn main() {
    let mut doc = Document::new();

    // Feature 5: Watermark
    doc.set_text_watermark("CONFIDENTIAL", "D0D0D0", Some(-45));

    // Title
    let mut p = doc.add_paragraph("");
    p.add_run("zavora-docx Feature Showcase").font("Arial").size(22.0).bold(true);

    // Feature 1: Footnotes
    let mut p = doc.add_paragraph("");
    p.add_run("1. Footnotes & Endnotes").font("Arial").size(14.0).bold(true);
    let fn1 = doc.add_footnote("This footnote demonstrates the footnotes feature in zavora-docx.");
    let en1 = doc.add_endnote("Endnotes appear at the end of the document section.");
    let mut p = doc.add_paragraph("");
    p.add_run("This text has a footnote").font("Arial").size(11.0);
    p.add_run("").footnote_ref(fn1);
    p.add_run(" and this has an endnote").font("Arial").size(11.0);
    p.add_run("").endnote_ref(en1);
    p.add_run(".").font("Arial").size(11.0);

    // Feature 2: Hyperlinks
    let mut p = doc.add_paragraph("");
    p.add_run("2. Hyperlinks").font("Arial").size(14.0).bold(true);
    let rel = doc.add_hyperlink_rel("https://github.com/zavora-ai");
    let mut p = doc.add_paragraph("");
    p.add_run("External link: ").font("Arial").size(11.0);
    p.add_hyperlink_run("Zavora AI GitHub", Some(&rel), None).color("0563C1").underline(true);
    p.add_run(" | Internal link: ").font("Arial").size(11.0);
    p.add_hyperlink_run("Jump to Form Fields", None, Some("form_fields")).color("0563C1").underline(true);

    // Feature 3: Bookmarks
    let mut p = doc.add_paragraph("");
    p.add_run("3. Bookmarks").font("Arial").size(14.0).bold(true);
    p.bookmark(10, "bookmarks_section");
    let mut p = doc.add_paragraph("");
    p.add_run("This section is bookmarked as 'bookmarks_section' for cross-referencing.").font("Arial").size(11.0);

    // Feature 4: Comments
    let mut p = doc.add_paragraph("");
    p.add_run("4. Comments").font("Arial").size(14.0).bold(true);
    doc.add_comment(1, "Reviewer", "This section demonstrates inline comments.");
    let mut p = doc.add_paragraph("");
    p.comment_start(1);
    p.add_run("This text has a review comment attached to it.").font("Arial").size(11.0);
    p.comment_end(1);

    // Feature 5: (watermark already set above)
    let mut p = doc.add_paragraph("");
    p.add_run("5. Watermark").font("Arial").size(14.0).bold(true);
    let mut p = doc.add_paragraph("");
    p.add_run("Look behind the text — there's a diagonal 'CONFIDENTIAL' watermark on every page.").font("Arial").size(11.0);

    // Feature 6: Track Changes
    let mut p = doc.add_paragraph("");
    p.add_run("6. Track Changes").font("Arial").size(14.0).bold(true);
    let mut p = doc.add_paragraph("");
    p.add_run("Original text remains. ").font("Arial").size(11.0);
    p.add_tracked_insert("This was inserted by the editor.", "Editor");
    let mut p = doc.add_paragraph("");
    p.add_tracked_delete("This text was deleted.", "Editor");
    p.add_tracked_insert(" Replaced with this.", "Editor");

    // Feature 7: Form Fields
    let mut p = doc.add_paragraph("");
    p.add_run("7. Form Fields").font("Arial").size(14.0).bold(true);
    p.bookmark(20, "form_fields");
    let mut p = doc.add_paragraph("");
    p.add_run("Name: ").font("Arial").size(11.0);
    p.add_text_field("Name", "Your Name");
    let mut p = doc.add_paragraph("");
    p.add_run("Agree: ").font("Arial").size(11.0);
    p.add_checkbox("Agree", false);
    let mut p = doc.add_paragraph("");
    p.add_run("Role: ").font("Arial").size(11.0);
    p.add_dropdown("Role", &["Developer", "Designer", "Manager"], 0);

    // Feature 8: (protection not applied to showcase so you can edit it)
    let mut p = doc.add_paragraph("");
    p.add_run("8. Document Protection").font("Arial").size(14.0).bold(true);
    let mut p = doc.add_paragraph("");
    p.add_run("Protection is available via protect_readonly(), protect_forms_only(), etc. Not applied here so you can interact with the form fields above.").font("Arial").size(11.0);

    doc.save("/Users/jameskaranja/Downloads/00_showcase_all_features.docx").unwrap();
    println!("✓ Saved ~/Downloads/00_showcase_all_features.docx");
}
