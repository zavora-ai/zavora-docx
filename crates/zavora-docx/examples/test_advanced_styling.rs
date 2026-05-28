use zavora_docx::{Document, Length, Alignment, BorderStyle};

fn main() {
    let mut doc = Document::new();
    doc.set_page_size(Length::inches(8.5), Length::inches(11.0));
    doc.set_margins(Length::inches(1.0), Length::inches(1.0), Length::inches(1.0), Length::inches(1.0));

    // Feature 8: Set theme
    doc.set_theme(
        &[("dk1", "000000"), ("lt1", "FFFFFF"), ("dk2", "1F497D"), ("lt2", "EEECE1"),
          ("accent1", "4472C4"), ("accent2", "ED7D31"), ("accent3", "A5A5A5"),
          ("accent4", "FFC000"), ("accent5", "5B9BD5"), ("accent6", "70AD47"),
          ("hlink", "0563C1"), ("folHlink", "954F72")],
        "Calibri Light", "Calibri"
    );

    // Title with theme color
    let mut p = doc.add_paragraph("");
    p = p.alignment(Alignment::Center);
    p.add_run("Advanced Styling Showcase").font("Calibri Light").size(28.0).bold(true).theme_color("accent1");

    let mut p = doc.add_paragraph("");
    p = p.alignment(Alignment::Center);
    p.add_run("zavora-docx 0.2.0").font("Calibri").size(14.0).theme_color("accent2");

    // Feature 1: Theme colors
    let mut p = doc.add_paragraph("");
    p = p.space_before(Length::pt(24.0));
    p.add_run("1. Theme Colors").font("Calibri Light").size(16.0).bold(true).theme_color("dk2");
    let mut p = doc.add_paragraph("");
    p.add_run("Accent 1 ").theme_color("accent1").size(11.0);
    p.add_run("Accent 2 ").theme_color("accent2").size(11.0);
    p.add_run("Accent 3 ").theme_color("accent3").size(11.0);
    p.add_run("Accent 4 ").theme_color("accent4").size(11.0);
    p.add_run("Accent 5 ").theme_color("accent5").size(11.0);
    p.add_run("Accent 6 ").theme_color("accent6").size(11.0);

    // Feature 2: Drop cap
    let mut p = doc.add_paragraph("");
    p = p.space_before(Length::pt(24.0));
    p.add_run("2. Drop Cap").font("Calibri Light").size(16.0).bold(true);
    let mut p = doc.add_paragraph("");
    p = p.drop_cap(3);
    p.add_run("O").font("Georgia").size(48.0);
    let mut p = doc.add_paragraph("");
    p.add_run("nce upon a time, in a land of pure Rust, there lived a document library that could do anything. It had theme colors, text effects, and even drop caps like this very paragraph demonstrates.").font("Georgia").size(11.0);

    // Feature 3: w14 Text Effects
    let mut p = doc.add_paragraph("");
    p = p.space_before(Length::pt(24.0));
    p.add_run("3. Text Effects (w14)").font("Calibri Light").size(16.0).bold(true);
    let mut p = doc.add_paragraph("");
    p.add_run("Shadow ").font("Calibri").size(18.0).bold(true).shadow(3.0, 2.0, "000000");
    p.add_run("Glow ").font("Calibri").size(18.0).bold(true).glow(6.0, "FF6600");
    p.add_run("Outline ").font("Calibri").size(18.0).bold(true).text_outline(0.5, "4472C4");
    p.add_run("Reflection").font("Calibri").size(18.0).bold(true).reflection();

    // Feature 4: Table with banded rows and header styling
    let mut p = doc.add_paragraph("");
    p = p.space_before(Length::pt(24.0));
    p.add_run("4. Table Conditional Formatting").font("Calibri Light").size(16.0).bold(true);

    let mut table = doc.add_table(5, 3);
    table = table.width_pct(100.0).borders(BorderStyle::Single, 4, "BFBFBF");
    if let Some(mut c) = table.cell(0, 0) { c.set_text("Feature"); }
    if let Some(mut c) = table.cell(0, 1) { c.set_text("Status"); }
    if let Some(mut c) = table.cell(0, 2) { c.set_text("Notes"); }
    if let Some(mut c) = table.cell(1, 0) { c.set_text("Theme Colors"); }
    if let Some(mut c) = table.cell(1, 1) { c.set_text("Complete"); }
    if let Some(mut c) = table.cell(1, 2) { c.set_text("All 12 theme slots"); }
    if let Some(mut c) = table.cell(2, 0) { c.set_text("Text Effects"); }
    if let Some(mut c) = table.cell(2, 1) { c.set_text("Complete"); }
    if let Some(mut c) = table.cell(2, 2) { c.set_text("Shadow, glow, outline, reflection"); }
    if let Some(mut c) = table.cell(3, 0) { c.set_text("Drop Caps"); }
    if let Some(mut c) = table.cell(3, 1) { c.set_text("Complete"); }
    if let Some(mut c) = table.cell(3, 2) { c.set_text("Configurable line span"); }
    if let Some(mut c) = table.cell(4, 0) { c.set_text("Kerning"); }
    if let Some(mut c) = table.cell(4, 1) { c.set_text("Complete"); }
    if let Some(mut c) = table.cell(4, 2) { c.set_text("OpenType kerning + ligatures"); }
    table.header_row_style("4472C4", "FFFFFF");
    table.banded_rows("D9E2F3");

    // Feature 5: Kerning + Ligatures
    let mut p = doc.add_paragraph("");
    p = p.space_before(Length::pt(24.0));
    p.add_run("5. Kerning & Ligatures").font("Calibri Light").size(16.0).bold(true);
    let mut p = doc.add_paragraph("");
    p.add_run("With kerning: ").font("Calibri").size(11.0);
    p.add_run("AVATAR WAV fly").font("Georgia").size(14.0).kerning(8.0).ligatures("standard");
    let mut p = doc.add_paragraph("");
    p.add_run("Fine typography with OpenType features enabled for professional output.").font("Georgia").size(11.0).kerning(8.0).ligatures("all");

    // Feature 6: Line numbering
    let mut p = doc.add_paragraph("");
    p = p.space_before(Length::pt(24.0));
    p.add_run("6. Line Numbering").font("Calibri Light").size(16.0).bold(true);
    let mut p = doc.add_paragraph("");
    p.add_run("Line numbering is enabled for this document (every 5th line). Check the left margin in Print Layout view.").font("Calibri").size(11.0);
    doc.set_line_numbering(5, "continuous");

    // Feature 7: Custom numbering
    let mut p = doc.add_paragraph("");
    p = p.space_before(Length::pt(24.0));
    p.add_run("7. Custom Numbering").font("Calibri Light").size(16.0).bold(true);

    doc.add_custom_list_item("First item (Roman)", 0, "upperRoman", None);
    doc.add_custom_list_item("Second item (Roman)", 0, "upperRoman", None);
    doc.add_custom_list_item("Third item (Roman)", 0, "upperRoman", None);

    doc.add_custom_list_item("Alpha item A", 0, "lowerLetter", None);
    doc.add_custom_list_item("Alpha item B", 0, "lowerLetter", None);

    doc.add_custom_list_item("Custom bullet: Star", 0, "bullet", Some("★"));
    doc.add_custom_list_item("Custom bullet: Arrow", 0, "bullet", Some("→"));
    doc.add_custom_list_item("Custom bullet: Diamond", 0, "bullet", Some("◆"));

    // Feature 8: Theme (already set above)
    let mut p = doc.add_paragraph("");
    p = p.space_before(Length::pt(24.0));
    p.add_run("8. Document Theme").font("Calibri Light").size(16.0).bold(true);
    let mut p = doc.add_paragraph("");
    p.add_run("This document uses a custom theme with Calibri Light/Calibri fonts and a blue accent color scheme. Theme colors automatically update when the theme is changed in Word.").font("Calibri").size(11.0);

    doc.save("/Users/jameskaranja/Downloads/09_advanced_styling.docx").unwrap();
    println!("✓ Saved ~/Downloads/09_advanced_styling.docx");
}
