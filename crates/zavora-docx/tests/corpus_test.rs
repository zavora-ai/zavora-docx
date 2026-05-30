//! Golden-file corpus: build a feature-rich document covering every major
//! construct, then assert load→save is structurally stable. Re-opening via
//! `from_bytes` exercises the parsers for every modeled part, so a successful
//! round-trip (with stable content count) is the structural-validity gate.

use zavora_docx::{Chart, ChartKind, Document, Length, PicProps, SdtKind, Series};

fn build_corpus() -> Document {
    let mut doc = Document::new();
    doc.set_title("Corpus");
    doc.set_company("Zavora");
    doc.add_paragraph("Heading").style("Heading1");
    {
        let mut p = doc.add_paragraph("Body with ");
        p.add_run("a French phrase").language("fr-FR");
    }
    doc.add_content_control(SdtKind::Checkbox(true), "agree", None);
    doc.add_equation_latex(r"x = \frac{-b \pm \sqrt{b^2-4ac}}{2a}");
    doc.add_text_box(Length::inches(2.0), Length::inches(1.0), vec!["Box".into()]);
    doc.add_shape(Length::inches(1.5), Length::inches(1.5), "ellipse", Some("FFCC00"));
    doc.add_chart(
        &Chart {
            kind: ChartKind::Scatter,
            title: Some("XY".into()),
            categories: vec!["1".into(), "2".into()],
            series: vec![Series { name: "s".into(), values: vec![3.0, 6.0] }],
            labels: None,
        },
        Length::inches(5.0),
        Length::inches(3.0),
    );
    doc.add_comment(1, "Rev", "note");
    doc.add_comment_reply(2, 1, "Rev2", "reply");
    doc
}

#[test]
fn corpus_load_save_is_structurally_stable() {
    let mut doc = build_corpus();
    let count = doc.content_count();
    let bytes1 = doc.to_bytes().expect("serialize");

    // Re-open (parses every modeled part) and re-save twice.
    let mut reopened = Document::from_bytes(&bytes1).expect("reopen");
    assert_eq!(reopened.content_count(), count, "content count drifted on reopen");
    let bytes2 = reopened.to_bytes().expect("re-serialize");
    let reopened2 = Document::from_bytes(&bytes2).expect("reopen 2");
    assert_eq!(reopened2.content_count(), count, "content count drifted on 2nd round-trip");

    // Output should be non-trivial and the two round-trips should be stable in size
    // (no relationship/part duplication blow-up).
    assert!(bytes2.len() > 1000, "suspiciously small output");
    let ratio = bytes2.len() as f64 / bytes1.len() as f64;
    assert!(ratio > 0.5 && ratio < 1.5, "round-trip size drift {ratio}");
}

#[test]
fn corpus_picture_with_props_round_trips() {
    // 1x1 PNG.
    let png: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x18, 0xDD, 0x8D, 0xB0, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    let mut doc = Document::new();
    doc.add_picture_with(
        png,
        "img.png",
        Length::inches(1.0),
        Length::inches(1.0),
        PicProps {
            rotation: Some(45 * 60_000),
            border: Some(("000000".into(), 12700)),
            title: Some("Alt".into()),
            ..Default::default()
        },
    );
    let bytes = doc.to_bytes().expect("serialize");
    let reopened = Document::from_bytes(&bytes).expect("reopen");
    assert_eq!(reopened.content_count(), 1);
}

#[test]
fn layout_primitives_column_widths_and_tab() {
    use zavora_docx::{BorderStyle, TabAlignment};
    let mut doc = Document::new();
    // Non-uniform columns + borders on a 2x2 table.
    let mut t = doc
        .add_table(2, 2)
        .borders(BorderStyle::Single, 4, "000000")
        .column_widths(&[Length::inches(4.0), Length::inches(2.0)]);
    if let Some(mut c) = t.cell(0, 0) { c.set_text("wide"); }
    if let Some(mut c) = t.cell(0, 1) { c.set_text("narrow"); }
    // A tab-aligned line: left text, right tab stop, trailing run.
    {
        let mut p = doc.add_paragraph("").add_tab_stop(TabAlignment::Right, Length::inches(6.5));
        p.add_run("Left");
        p.add_tab();
        p.add_run("Right");
    }
    let bytes = doc.to_bytes().expect("serialize");
    // Re-parse proves it's well-formed end to end.
    let reopened = Document::from_bytes(&bytes).expect("reopen");
    assert!(reopened.content_count() >= 2);
}

#[test]
fn to_plain_text_includes_table_cells_in_order() {
    let mut doc = Document::new();
    doc.add_paragraph("Before table");
    let mut t = doc.add_table(1, 2);
    if let Some(mut c) = t.cell(0, 0) { c.set_text("R0C0"); }
    if let Some(mut c) = t.cell(0, 1) { c.set_text("R0C1"); }
    doc.add_paragraph("After table");
    let text = doc.to_plain_text();
    for needle in ["Before table", "R0C0", "R0C1", "After table"] {
        assert!(text.contains(needle), "missing {needle}: {text}");
    }
    // Document order preserved: paragraph, then cells, then paragraph.
    let before = text.find("Before table").unwrap();
    let cell = text.find("R0C0").unwrap();
    let after = text.find("After table").unwrap();
    assert!(before < cell && cell < after, "out of order: {text}");
}
