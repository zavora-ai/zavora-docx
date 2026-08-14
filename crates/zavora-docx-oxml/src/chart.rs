//! Charts — DrawingML `c:chart` construction.
//!
//! A chart is a separate part (`word/charts/chartN.xml`) referenced by a
//! `c:chart` drawing in the body via a relationship. This module builds both
//! the chart part XML and the referencing drawing run.

use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

use crate::error::Result;
use crate::units::Emu;

const C_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/chart";
const A_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
const R_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const WP_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing";

/// Supported chart types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChartKind {
    Bar,
    Column,
    Line,
    Pie,
    Area,
    Scatter,
}

/// A data series: a name and one value per category.
#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    pub name: String,
    pub values: Vec<f64>,
}

/// Where data labels are placed. Maps to `c:dLblPos@val`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LabelPosition {
    OutsideEnd,
    InsideEnd,
    Center,
    InsideBase,
    BestFit,
}

impl LabelPosition {
    fn as_str(self) -> &'static str {
        match self {
            LabelPosition::OutsideEnd => "outEnd",
            LabelPosition::InsideEnd => "inEnd",
            LabelPosition::Center => "ctr",
            LabelPosition::InsideBase => "inBase",
            LabelPosition::BestFit => "bestFit",
        }
    }
}

/// Configurable data-label display. All fields optional so the caller controls
/// placement, what is shown, and an optional fixed text color.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DataLabels {
    /// Label position; `None` lets Word use its default for the chart type.
    pub position: Option<LabelPosition>,
    pub show_value: bool,
    pub show_category: bool,
    pub show_series: bool,
    pub show_percent: bool,
    pub show_legend_key: bool,
    /// Optional fixed label text color (hex, e.g. "FFFFFF").
    pub color: Option<String>,
}

/// A chart definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Chart {
    pub kind: ChartKind,
    pub title: Option<String>,
    pub categories: Vec<String>,
    pub series: Vec<Series>,
    /// Data-label configuration; `None` = sensible per-kind default.
    pub labels: Option<DataLabels>,
}

impl Chart {
    /// Default data labels for a chart kind: pie shows category+percent placed
    /// outside; cartesian charts show values placed outside the bar/point.
    pub fn default_labels(kind: ChartKind) -> DataLabels {
        match kind {
            ChartKind::Pie => DataLabels {
                position: Some(LabelPosition::OutsideEnd),
                show_category: true,
                show_percent: true,
                ..Default::default()
            },
            _ => DataLabels {
                position: Some(LabelPosition::OutsideEnd),
                show_value: true,
                ..Default::default()
            },
        }
    }

    /// Build the full `word/charts/chartN.xml` part content.
    pub fn to_part_bytes(&self) -> Result<Vec<u8>> {
        let mut w = Writer::new(Vec::new());
        let mut space = BytesStart::new("c:chartSpace");
        space.push_attribute(("xmlns:c", C_NS));
        space.push_attribute(("xmlns:a", A_NS));
        space.push_attribute(("xmlns:r", R_NS));
        w.write_event(Event::Start(space))?;
        w.write_event(Event::Start(BytesStart::new("c:chart")))?;

        // Title
        if let Some(ref t) = self.title {
            w.write_event(Event::Start(BytesStart::new("c:title")))?;
            w.write_event(Event::Start(BytesStart::new("c:tx")))?;
            w.write_event(Event::Start(BytesStart::new("c:rich")))?;
            w.write_event(Event::Empty(BytesStart::new("a:bodyPr")))?;
            w.write_event(Event::Empty(BytesStart::new("a:lstStyle")))?;
            w.write_event(Event::Start(BytesStart::new("a:p")))?;
            w.write_event(Event::Start(BytesStart::new("a:r")))?;
            w.write_event(Event::Start(BytesStart::new("a:t")))?;
            w.write_event(Event::Text(BytesText::new(t)))?;
            w.write_event(Event::End(BytesEnd::new("a:t")))?;
            w.write_event(Event::End(BytesEnd::new("a:r")))?;
            w.write_event(Event::End(BytesEnd::new("a:p")))?;
            w.write_event(Event::End(BytesEnd::new("c:rich")))?;
            w.write_event(Event::End(BytesEnd::new("c:tx")))?;
            w.write_event(Event::End(BytesEnd::new("c:title")))?;
            bool_el(&mut w, "c:autoTitleDeleted", false)?;
        }

        w.write_event(Event::Start(BytesStart::new("c:plotArea")))?;
        w.write_event(Event::Empty(BytesStart::new("c:layout")))?;
        self.write_plot(&mut w)?;
        // Axes (category + value) for non-pie charts.
        if self.kind != ChartKind::Pie {
            self.write_axes(&mut w)?;
        }
        w.write_event(Event::End(BytesEnd::new("c:plotArea")))?;

        // Legend
        w.write_event(Event::Start(BytesStart::new("c:legend")))?;
        str_el(&mut w, "c:legendPos", "r")?;
        bool_el(&mut w, "c:overlay", false)?;
        w.write_event(Event::End(BytesEnd::new("c:legend")))?;
        bool_el(&mut w, "c:plotVisOnly", true)?;

        w.write_event(Event::End(BytesEnd::new("c:chart")))?;
        w.write_event(Event::End(BytesEnd::new("c:chartSpace")))?;
        Ok(w.into_inner())
    }

    fn write_plot<W: std::io::Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.kind == ChartKind::Scatter {
            return self.write_scatter(w);
        }
        let (tag, extra): (&str, &[(&str, &str)]) = match self.kind {
            ChartKind::Bar => ("c:barChart", &[]),
            ChartKind::Column => ("c:barChart", &[]),
            ChartKind::Line => ("c:lineChart", &[]),
            ChartKind::Pie => ("c:pieChart", &[]),
            ChartKind::Area => ("c:areaChart", &[]),
            ChartKind::Scatter => unreachable!(),
        };
        let _ = extra;
        w.write_event(Event::Start(BytesStart::new(tag)))?;
        match self.kind {
            ChartKind::Bar => str_el(w, "c:barDir", "bar")?,
            ChartKind::Column => str_el(w, "c:barDir", "col")?,
            _ => {}
        }
        if matches!(self.kind, ChartKind::Bar | ChartKind::Column) {
            str_el(w, "c:grouping", "clustered")?;
        }
        if self.kind == ChartKind::Pie {
            bool_el(w, "c:varyColors", true)?;
        }

        for (idx, s) in self.series.iter().enumerate() {
            w.write_event(Event::Start(BytesStart::new("c:ser")))?;
            idx_el(w, "c:idx", idx)?;
            idx_el(w, "c:order", idx)?;
            // series name
            w.write_event(Event::Start(BytesStart::new("c:tx")))?;
            w.write_event(Event::Start(BytesStart::new("c:strRef")))?;
            str_el(w, "c:f", &format!("Sheet1!$B${}", idx + 1))?;
            w.write_event(Event::Start(BytesStart::new("c:strCache")))?;
            idx_el(w, "c:ptCount", 1)?;
            pt_str(w, 0, &s.name)?;
            w.write_event(Event::End(BytesEnd::new("c:strCache")))?;
            w.write_event(Event::End(BytesEnd::new("c:strRef")))?;
            w.write_event(Event::End(BytesEnd::new("c:tx")))?;

            // categories
            w.write_event(Event::Start(BytesStart::new("c:cat")))?;
            w.write_event(Event::Start(BytesStart::new("c:strRef")))?;
            str_el(w, "c:f", "Sheet1!$A$1")?;
            w.write_event(Event::Start(BytesStart::new("c:strCache")))?;
            idx_el(w, "c:ptCount", self.categories.len())?;
            for (i, c) in self.categories.iter().enumerate() {
                pt_str(w, i, c)?;
            }
            w.write_event(Event::End(BytesEnd::new("c:strCache")))?;
            w.write_event(Event::End(BytesEnd::new("c:strRef")))?;
            w.write_event(Event::End(BytesEnd::new("c:cat")))?;

            // values
            w.write_event(Event::Start(BytesStart::new("c:val")))?;
            w.write_event(Event::Start(BytesStart::new("c:numRef")))?;
            str_el(w, "c:f", &format!("Sheet1!$B${}", idx + 1))?;
            w.write_event(Event::Start(BytesStart::new("c:numCache")))?;
            str_el(w, "c:formatCode", "General")?;
            idx_el(w, "c:ptCount", s.values.len())?;
            for (i, v) in s.values.iter().enumerate() {
                pt_num(w, i, *v)?;
            }
            w.write_event(Event::End(BytesEnd::new("c:numCache")))?;
            w.write_event(Event::End(BytesEnd::new("c:numRef")))?;
            w.write_event(Event::End(BytesEnd::new("c:val")))?;
            w.write_event(Event::End(BytesEnd::new("c:ser")))?;
        }

        // Data labels (configurable; falls back to a per-kind default).
        let labels = self
            .labels
            .clone()
            .unwrap_or_else(|| Chart::default_labels(self.kind));
        w.write_event(Event::Start(BytesStart::new("c:dLbls")))?;
        // Optional fixed text color via c:txPr (must precede the show* flags).
        if let Some(ref hex) = labels.color {
            w.write_event(Event::Start(BytesStart::new("c:txPr")))?;
            w.write_event(Event::Empty(BytesStart::new("a:bodyPr")))?;
            w.write_event(Event::Empty(BytesStart::new("a:lstStyle")))?;
            w.write_event(Event::Start(BytesStart::new("a:p")))?;
            w.write_event(Event::Start(BytesStart::new("a:pPr")))?;
            w.write_event(Event::Start(BytesStart::new("a:defRPr")))?;
            w.write_event(Event::Start(BytesStart::new("a:solidFill")))?;
            let mut c = BytesStart::new("a:srgbClr");
            c.push_attribute(("val", hex.as_str()));
            w.write_event(Event::Empty(c))?;
            w.write_event(Event::End(BytesEnd::new("a:solidFill")))?;
            w.write_event(Event::End(BytesEnd::new("a:defRPr")))?;
            w.write_event(Event::End(BytesEnd::new("a:pPr")))?;
            w.write_event(Event::Empty(BytesStart::new("a:endParaRPr")))?;
            w.write_event(Event::End(BytesEnd::new("a:p")))?;
            w.write_event(Event::End(BytesEnd::new("c:txPr")))?;
        }
        // dLblPos is only valid for pie/bar/column.
        if let Some(pos) = labels.position
            && matches!(
                self.kind,
                ChartKind::Pie | ChartKind::Bar | ChartKind::Column
            )
        {
            str_el(w, "c:dLblPos", pos.as_str())?;
        }
        bool_el(w, "c:showLegendKey", labels.show_legend_key)?;
        bool_el(w, "c:showVal", labels.show_value)?;
        bool_el(w, "c:showCatName", labels.show_category)?;
        bool_el(w, "c:showSerName", labels.show_series)?;
        bool_el(w, "c:showPercent", labels.show_percent)?;
        bool_el(w, "c:showBubbleSize", false)?;
        w.write_event(Event::End(BytesEnd::new("c:dLbls")))?;

        // Axis id wiring for cartesian charts.
        if self.kind != ChartKind::Pie {
            idx_el(w, "c:axId", 1)?;
            idx_el(w, "c:axId", 2)?;
        }
        w.write_event(Event::End(BytesEnd::new(tag)))?;
        Ok(())
    }

    /// Scatter chart: each series plots y-values (series.values) against
    /// x-values parsed from `categories` (falls back to 1..n if non-numeric).
    fn write_scatter<W: std::io::Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_event(Event::Start(BytesStart::new("c:scatterChart")))?;
        str_el(w, "c:scatterStyle", "lineMarker")?;
        let xs: Vec<f64> = self
            .categories
            .iter()
            .enumerate()
            .map(|(i, c)| c.parse().unwrap_or((i + 1) as f64))
            .collect();
        for (idx, s) in self.series.iter().enumerate() {
            w.write_event(Event::Start(BytesStart::new("c:ser")))?;
            idx_el(w, "c:idx", idx)?;
            idx_el(w, "c:order", idx)?;
            // series name
            w.write_event(Event::Start(BytesStart::new("c:tx")))?;
            w.write_event(Event::Start(BytesStart::new("c:strRef")))?;
            str_el(w, "c:f", &format!("Sheet1!$B${}", idx + 1))?;
            w.write_event(Event::Start(BytesStart::new("c:strCache")))?;
            idx_el(w, "c:ptCount", 1)?;
            pt_str(w, 0, &s.name)?;
            w.write_event(Event::End(BytesEnd::new("c:strCache")))?;
            w.write_event(Event::End(BytesEnd::new("c:strRef")))?;
            w.write_event(Event::End(BytesEnd::new("c:tx")))?;
            // xVal
            w.write_event(Event::Start(BytesStart::new("c:xVal")))?;
            self.write_num_ref(w, "Sheet1!$A$1", &xs)?;
            w.write_event(Event::End(BytesEnd::new("c:xVal")))?;
            // yVal
            w.write_event(Event::Start(BytesStart::new("c:yVal")))?;
            self.write_num_ref(w, &format!("Sheet1!$B${}", idx + 1), &s.values)?;
            w.write_event(Event::End(BytesEnd::new("c:yVal")))?;
            w.write_event(Event::End(BytesEnd::new("c:ser")))?;
        }
        idx_el(w, "c:axId", 1)?;
        idx_el(w, "c:axId", 2)?;
        w.write_event(Event::End(BytesEnd::new("c:scatterChart")))?;
        Ok(())
    }

    fn write_num_ref<W: std::io::Write>(
        &self,
        w: &mut Writer<W>,
        f: &str,
        vals: &[f64],
    ) -> Result<()> {
        w.write_event(Event::Start(BytesStart::new("c:numRef")))?;
        str_el(w, "c:f", f)?;
        w.write_event(Event::Start(BytesStart::new("c:numCache")))?;
        str_el(w, "c:formatCode", "General")?;
        idx_el(w, "c:ptCount", vals.len())?;
        for (i, v) in vals.iter().enumerate() {
            pt_num(w, i, *v)?;
        }
        w.write_event(Event::End(BytesEnd::new("c:numCache")))?;
        w.write_event(Event::End(BytesEnd::new("c:numRef")))?;
        Ok(())
    }

    fn write_axes<W: std::io::Write>(&self, w: &mut Writer<W>) -> Result<()> {
        // Category axis
        // First axis: value axis for scatter (numeric X), category axis otherwise.
        let first_ax = if self.kind == ChartKind::Scatter {
            "c:valAx"
        } else {
            "c:catAx"
        };
        w.write_event(Event::Start(BytesStart::new(first_ax)))?;
        idx_el(w, "c:axId", 1)?;
        w.write_event(Event::Start(BytesStart::new("c:scaling")))?;
        str_el(w, "c:orientation", "minMax")?;
        w.write_event(Event::End(BytesEnd::new("c:scaling")))?;
        bool_el(w, "c:delete", false)?;
        str_el(
            w,
            "c:axPos",
            if self.kind == ChartKind::Bar {
                "l"
            } else {
                "b"
            },
        )?;
        idx_el(w, "c:crossAx", 2)?;
        w.write_event(Event::End(BytesEnd::new(first_ax)))?;
        // Value axis
        w.write_event(Event::Start(BytesStart::new("c:valAx")))?;
        idx_el(w, "c:axId", 2)?;
        w.write_event(Event::Start(BytesStart::new("c:scaling")))?;
        str_el(w, "c:orientation", "minMax")?;
        w.write_event(Event::End(BytesEnd::new("c:scaling")))?;
        bool_el(w, "c:delete", false)?;
        str_el(
            w,
            "c:axPos",
            if self.kind == ChartKind::Bar {
                "b"
            } else {
                "l"
            },
        )?;
        idx_el(w, "c:crossAx", 1)?;
        w.write_event(Event::End(BytesEnd::new("c:valAx")))?;
        Ok(())
    }

    /// Build the `w:r > w:drawing` run that references the chart part by rel id.
    pub fn to_run_bytes(&self, rel_id: &str, width: Emu, height: Emu) -> Result<Vec<u8>> {
        let mut w = Writer::new(Vec::new());
        let mut buf = itoa::Buffer::new();
        w.write_event(Event::Start(BytesStart::new("w:r")))?;
        w.write_event(Event::Start(BytesStart::new("w:drawing")))?;
        let mut inl = BytesStart::new("wp:inline");
        inl.push_attribute(("xmlns:wp", WP_NS));
        for a in ["distT", "distB", "distL", "distR"] {
            inl.push_attribute((a, "0"));
        }
        w.write_event(Event::Start(inl))?;
        let mut ext = BytesStart::new("wp:extent");
        ext.push_attribute(("cx", buf.format(width.0)));
        ext.push_attribute(("cy", buf.format(height.0)));
        w.write_event(Event::Empty(ext))?;
        let mut dp = BytesStart::new("wp:docPr");
        dp.push_attribute(("id", "1"));
        dp.push_attribute(("name", "Chart"));
        w.write_event(Event::Empty(dp))?;
        let mut g = BytesStart::new("a:graphic");
        g.push_attribute(("xmlns:a", A_NS));
        w.write_event(Event::Start(g))?;
        let mut gd = BytesStart::new("a:graphicData");
        gd.push_attribute(("uri", C_NS));
        w.write_event(Event::Start(gd))?;
        let mut chart = BytesStart::new("c:chart");
        chart.push_attribute(("xmlns:c", C_NS));
        chart.push_attribute(("xmlns:r", R_NS));
        chart.push_attribute(("r:id", rel_id));
        w.write_event(Event::Empty(chart))?;
        w.write_event(Event::End(BytesEnd::new("a:graphicData")))?;
        w.write_event(Event::End(BytesEnd::new("a:graphic")))?;
        w.write_event(Event::End(BytesEnd::new("wp:inline")))?;
        w.write_event(Event::End(BytesEnd::new("w:drawing")))?;
        w.write_event(Event::End(BytesEnd::new("w:r")))?;
        Ok(w.into_inner())
    }
}

fn str_el<W: std::io::Write>(w: &mut Writer<W>, tag: &str, val: &str) -> Result<()> {
    let mut e = BytesStart::new(tag);
    e.push_attribute(("val", val));
    w.write_event(Event::Empty(e))?;
    Ok(())
}
fn bool_el<W: std::io::Write>(w: &mut Writer<W>, tag: &str, val: bool) -> Result<()> {
    str_el(w, tag, if val { "1" } else { "0" })
}
fn idx_el<W: std::io::Write>(w: &mut Writer<W>, tag: &str, val: usize) -> Result<()> {
    str_el(w, tag, &val.to_string())
}
fn pt_str<W: std::io::Write>(w: &mut Writer<W>, idx: usize, text: &str) -> Result<()> {
    let mut pt = BytesStart::new("c:pt");
    pt.push_attribute(("idx", idx.to_string().as_str()));
    w.write_event(Event::Start(pt))?;
    w.write_event(Event::Start(BytesStart::new("c:v")))?;
    w.write_event(Event::Text(BytesText::new(text)))?;
    w.write_event(Event::End(BytesEnd::new("c:v")))?;
    w.write_event(Event::End(BytesEnd::new("c:pt")))?;
    Ok(())
}
fn pt_num<W: std::io::Write>(w: &mut Writer<W>, idx: usize, val: f64) -> Result<()> {
    let mut pt = BytesStart::new("c:pt");
    pt.push_attribute(("idx", idx.to_string().as_str()));
    w.write_event(Event::Start(pt))?;
    w.write_event(Event::Start(BytesStart::new("c:v")))?;
    w.write_event(Event::Text(BytesText::new(&val.to_string())))?;
    w.write_event(Event::End(BytesEnd::new("c:v")))?;
    w.write_event(Event::End(BytesEnd::new("c:pt")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(kind: ChartKind) -> Chart {
        Chart {
            kind,
            title: Some("Sales".into()),
            categories: vec!["Q1".into(), "Q2".into(), "Q3".into()],
            series: vec![Series {
                name: "2024".into(),
                values: vec![10.0, 20.0, 15.0],
            }],
            labels: None,
        }
    }

    #[test]
    fn custom_labels_position_and_color() {
        let mut c = sample(ChartKind::Pie);
        c.labels = Some(DataLabels {
            position: Some(LabelPosition::Center),
            show_percent: true,
            color: Some("FFFFFF".into()),
            ..Default::default()
        });
        let x = String::from_utf8(c.to_part_bytes().unwrap()).unwrap();
        assert!(x.contains(r#"<c:dLblPos val="ctr"/>"#), "{x}");
        assert!(x.contains(r#"<a:srgbClr val="FFFFFF"/>"#), "{x}");
        assert!(x.contains(r#"<c:showPercent val="1"/>"#), "{x}");
    }

    #[test]
    fn bar_chart_part() {
        let bytes = sample(ChartKind::Bar).to_part_bytes().unwrap();
        // Must be well-formed (re-parse with a reader to catch tag mismatches).
        let mut rdr = quick_xml::Reader::from_reader(bytes.as_slice());
        loop {
            if rdr.read_event().expect("well-formed chart XML") == quick_xml::events::Event::Eof {
                break;
            }
        }
        let x = String::from_utf8(bytes).unwrap();
        assert!(x.contains("c:barChart"), "{x}");
        assert!(x.contains(r#"<c:barDir val="bar"/>"#), "{x}");
        assert!(x.contains("<c:v>Q2</c:v>"), "{x}");
        assert!(x.contains("<c:v>20</c:v>"), "{x}");
        assert!(x.contains("<c:catAx>"), "{x}");
    }

    #[test]
    fn scatter_chart_xy() {
        let c = Chart {
            kind: ChartKind::Scatter,
            title: Some("XY".into()),
            categories: vec!["1".into(), "2".into(), "3".into()],
            series: vec![Series {
                name: "pts".into(),
                values: vec![2.0, 4.0, 6.0],
            }],
            labels: None,
        };
        let bytes = c.to_part_bytes().unwrap();
        let mut rdr = quick_xml::Reader::from_reader(bytes.as_slice());
        loop {
            if rdr.read_event().expect("well-formed") == quick_xml::events::Event::Eof {
                break;
            }
        }
        let x = String::from_utf8(bytes).unwrap();
        assert!(x.contains("c:scatterChart"), "{x}");
        assert!(x.contains("<c:xVal>"), "{x}");
        assert!(x.contains("<c:yVal>"), "{x}");
        assert!(!x.contains("<c:catAx>"), "{x}");
    }

    #[test]
    fn pie_chart_has_no_axes() {
        let x = String::from_utf8(sample(ChartKind::Pie).to_part_bytes().unwrap()).unwrap();
        assert!(x.contains("c:pieChart"), "{x}");
        assert!(!x.contains("<c:catAx>"), "{x}");
    }

    #[test]
    fn drawing_references_rel() {
        let r = String::from_utf8(
            sample(ChartKind::Line)
                .to_run_bytes("rId9", Emu(5000000), Emu(3000000))
                .unwrap(),
        )
        .unwrap();
        assert!(r.contains(r#"r:id="rId9""#), "{r}");
        assert!(
            r.contains(r#"uri="http://schemas.openxmlformats.org/drawingml/2006/chart""#),
            "{r}"
        );
    }
}
