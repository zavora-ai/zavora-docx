//! Office Math (OMML) — `m:oMath` construction.
//!
//! A recursive `MathNode` tree serialized to `m:`-prefixed OMML. The document
//! root declares `xmlns:m`. This is the construction path; parsed equations
//! from existing documents are preserved verbatim elsewhere as raw XML.

use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

use crate::error::Result;

/// A node in an OMML expression tree.
#[derive(Debug, Clone, PartialEq)]
pub enum MathNode {
    /// A literal math run (`m:r`/`m:t`) — identifiers, numbers, operators.
    Run(String),
    /// Fraction: numerator over denominator (`m:f`).
    Fraction(Box<MathNode>, Box<MathNode>),
    /// Superscript / power (`m:sSup`).
    Sup(Box<MathNode>, Box<MathNode>),
    /// Subscript (`m:sSub`).
    Sub(Box<MathNode>, Box<MathNode>),
    /// Combined sub+superscript (`m:sSubSup`).
    SubSup(Box<MathNode>, Box<MathNode>, Box<MathNode>),
    /// Radical with optional degree (`m:rad`); `None` degree = square root.
    Radical(Option<Box<MathNode>>, Box<MathNode>),
    /// N-ary operator (sum/integral/product): char, lower, upper, operand (`m:nary`).
    Nary(String, Box<MathNode>, Box<MathNode>, Box<MathNode>),
    /// Delimited group with begin/end chars, e.g. parentheses (`m:d`).
    Delimiter(String, String, Vec<MathNode>),
    /// Named function (`m:func`), e.g. sin/cos with its argument.
    Func(String, Box<MathNode>),
    /// Accent over a base (`m:acc`), e.g. a hat or bar.
    Accent(String, Box<MathNode>),
    /// A matrix (`m:m`) of rows, each a vector of cell nodes.
    Matrix(Vec<Vec<MathNode>>),
    /// A bar over (or under) a base (`m:bar`).
    Bar(Box<MathNode>),
    /// A sequence of nodes rendered in order (no wrapper element).
    Seq(Vec<MathNode>),
}

impl MathNode {
    /// Convenience constructor for a run.
    pub fn run(s: impl Into<String>) -> MathNode {
        MathNode::Run(s.into())
    }

    /// Serialize this node as a standalone `m:oMathPara > m:oMath` block (with
    /// the `xmlns:m` declared locally, so it is valid even outside a w:document
    /// that declares it).
    pub fn to_omath_para_bytes(&self) -> Result<Vec<u8>> {
        let mut w = Writer::new(Vec::new());
        let mut para = BytesStart::new("m:oMathPara");
        para.push_attribute(("xmlns:m", crate::namespace::M_NS));
        w.write_event(Event::Start(para))?;
        self.write_omath(&mut w, false)?;
        w.write_event(Event::End(BytesEnd::new("m:oMathPara")))?;
        Ok(w.into_inner())
    }

    fn write_omath<W: std::io::Write>(&self, w: &mut Writer<W>, decl_ns: bool) -> Result<()> {
        let mut om = BytesStart::new("m:oMath");
        if decl_ns {
            om.push_attribute(("xmlns:m", crate::namespace::M_NS));
        }
        w.write_event(Event::Start(om))?;
        self.write(w)?;
        w.write_event(Event::End(BytesEnd::new("m:oMath")))?;
        Ok(())
    }

    /// Write the node's OMML (without an enclosing `m:oMath`).
    pub fn write<W: std::io::Write>(&self, w: &mut Writer<W>) -> Result<()> {
        match self {
            MathNode::Run(s) => run(w, s),
            MathNode::Seq(items) => {
                for it in items {
                    it.write(w)?;
                }
                Ok(())
            }
            MathNode::Fraction(num, den) => elem(w, "m:f", |w| {
                arg(w, "m:num", num)?;
                arg(w, "m:den", den)
            }),
            MathNode::Sup(base, sup) => elem(w, "m:sSup", |w| {
                arg(w, "m:e", base)?;
                arg(w, "m:sup", sup)
            }),
            MathNode::Sub(base, sub) => elem(w, "m:sSub", |w| {
                arg(w, "m:e", base)?;
                arg(w, "m:sub", sub)
            }),
            MathNode::SubSup(base, sub, sup) => elem(w, "m:sSubSup", |w| {
                arg(w, "m:e", base)?;
                arg(w, "m:sub", sub)?;
                arg(w, "m:sup", sup)
            }),
            MathNode::Radical(deg, radicand) => elem(w, "m:rad", |w| {
                // radPr controls whether the degree is hidden.
                elem(w, "m:radPr", |w| {
                    let mut d = BytesStart::new("m:degHide");
                    d.push_attribute(("m:val", if deg.is_none() { "1" } else { "0" }));
                    w.write_event(Event::Empty(d))?;
                    Ok(())
                })?;
                match deg {
                    Some(d) => arg(w, "m:deg", d)?,
                    None => elem(w, "m:deg", |_| Ok(()))?,
                }
                arg(w, "m:e", radicand)
            }),
            MathNode::Nary(chr, sub, sup, e) => elem(w, "m:nary", |w| {
                elem(w, "m:naryPr", |w| {
                    let mut c = BytesStart::new("m:chr");
                    c.push_attribute(("m:val", chr.as_str()));
                    w.write_event(Event::Empty(c))?;
                    Ok(())
                })?;
                arg(w, "m:sub", sub)?;
                arg(w, "m:sup", sup)?;
                arg(w, "m:e", e)
            }),
            MathNode::Delimiter(beg, end, items) => elem(w, "m:d", |w| {
                elem(w, "m:dPr", |w| {
                    let mut b = BytesStart::new("m:begChr");
                    b.push_attribute(("m:val", beg.as_str()));
                    w.write_event(Event::Empty(b))?;
                    let mut e = BytesStart::new("m:endChr");
                    e.push_attribute(("m:val", end.as_str()));
                    w.write_event(Event::Empty(e))?;
                    Ok(())
                })?;
                for it in items {
                    arg(w, "m:e", it)?;
                }
                Ok(())
            }),
            MathNode::Func(name, body) => elem(w, "m:func", |w| {
                elem(w, "m:fName", |w| run(w, name))?;
                arg(w, "m:e", body)
            }),
            MathNode::Accent(chr, base) => elem(w, "m:acc", |w| {
                elem(w, "m:accPr", |w| {
                    let mut c = BytesStart::new("m:chr");
                    c.push_attribute(("m:val", chr.as_str()));
                    w.write_event(Event::Empty(c))?;
                    Ok(())
                })?;
                arg(w, "m:e", base)
            }),
            MathNode::Bar(base) => elem(w, "m:bar", |w| {
                elem(w, "m:barPr", |w| {
                    let mut p = BytesStart::new("m:pos");
                    p.push_attribute(("m:val", "top"));
                    w.write_event(Event::Empty(p))?;
                    Ok(())
                })?;
                arg(w, "m:e", base)
            }),
            MathNode::Matrix(rows) => elem(w, "m:m", |w| {
                for row in rows {
                    elem(w, "m:mr", |w| {
                        for cell in row {
                            arg(w, "m:e", cell)?;
                        }
                        Ok(())
                    })?;
                }
                Ok(())
            }),
        }
    }
}

/// Write an `m:r` math run with text.
fn run<W: std::io::Write>(w: &mut Writer<W>, text: &str) -> Result<()> {
    w.write_event(Event::Start(BytesStart::new("m:r")))?;
    w.write_event(Event::Start(BytesStart::new("m:t")))?;
    w.write_event(Event::Text(BytesText::new(text)))?;
    w.write_event(Event::End(BytesEnd::new("m:t")))?;
    w.write_event(Event::End(BytesEnd::new("m:r")))?;
    Ok(())
}

/// Write a wrapper element with a body closure.
fn elem<W: std::io::Write, F: FnOnce(&mut Writer<W>) -> Result<()>>(
    w: &mut Writer<W>,
    tag: &str,
    body: F,
) -> Result<()> {
    w.write_event(Event::Start(BytesStart::new(tag)))?;
    body(w)?;
    w.write_event(Event::End(BytesEnd::new(tag)))?;
    Ok(())
}

/// Write an argument element (e.g. `m:num`) wrapping a child node.
fn arg<W: std::io::Write>(w: &mut Writer<W>, tag: &str, node: &MathNode) -> Result<()> {
    elem(w, tag, |w| node.write(w))
}

// ---- LaTeX-subset parser ----

/// Parse a small subset of LaTeX into a [`MathNode`]. Supports `\frac{}{}`,
/// `^{}`/`_{}` (single token or braced), `\sqrt{}`/`\sqrt[n]{}`, `\sum`/`\int`/
/// `\prod` with `_`/`^` limits, `\left(...\right)` and bare parentheses,
/// common functions (`\sin` etc.), Greek letters, and operators. Unknown
/// commands fall back to their literal name. Best-effort: never fails.
pub fn from_latex(src: &str) -> MathNode {
    let toks = lex(src);
    let mut pos = 0;
    parse_seq(&toks, &mut pos, None)
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Open,        // {
    Close,       // }
    Sup,         // ^
    Sub,         // _
    Cmd(String), // \name
    Sym(String), // a single visible char / number run
}

fn lex(src: &str) -> Vec<Tok> {
    let mut toks = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            '{' => {
                toks.push(Tok::Open);
                chars.next();
            }
            '}' => {
                toks.push(Tok::Close);
                chars.next();
            }
            '^' => {
                toks.push(Tok::Sup);
                chars.next();
            }
            '_' => {
                toks.push(Tok::Sub);
                chars.next();
            }
            ' ' | '\t' | '\n' => {
                chars.next();
            }
            '\\' => {
                chars.next();
                let mut name = String::new();
                while let Some(&n) = chars.peek() {
                    if n.is_ascii_alphabetic() {
                        name.push(n);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if name.is_empty() {
                    // escaped symbol like \{ or \,
                    if let Some(n) = chars.next() {
                        toks.push(Tok::Sym(n.to_string()));
                    }
                } else {
                    toks.push(Tok::Cmd(name));
                }
            }
            _ => {
                toks.push(Tok::Sym(c.to_string()));
                chars.next();
            }
        }
    }
    toks
}

/// Parse a sequence until `stop` token (or end). Handles sup/sub binding.
fn parse_seq(toks: &[Tok], pos: &mut usize, stop: Option<&Tok>) -> MathNode {
    let mut items: Vec<MathNode> = Vec::new();
    while *pos < toks.len() {
        if let Some(s) = stop
            && &toks[*pos] == s
        {
            break;
        }
        match &toks[*pos] {
            Tok::Close => break,
            Tok::Sup | Tok::Sub => {
                let is_sup = toks[*pos] == Tok::Sup;
                *pos += 1;
                let script = parse_atom(toks, pos);
                let base = items.pop().unwrap_or(MathNode::Run(String::new()));
                items.push(if is_sup {
                    MathNode::Sup(Box::new(base), Box::new(script))
                } else {
                    MathNode::Sub(Box::new(base), Box::new(script))
                });
            }
            _ => {
                let atom = parse_atom(toks, pos);
                items.push(atom);
            }
        }
    }
    if items.len() == 1 {
        items.pop().unwrap()
    } else {
        MathNode::Seq(items)
    }
}

/// Parse a single atom: a group `{...}`, a command, or a symbol.
fn parse_atom(toks: &[Tok], pos: &mut usize) -> MathNode {
    if *pos >= toks.len() {
        return MathNode::Run(String::new());
    }
    match toks[*pos].clone() {
        Tok::Open => {
            *pos += 1;
            let inner = parse_seq(toks, pos, Some(&Tok::Close));
            if *pos < toks.len() && toks[*pos] == Tok::Close {
                *pos += 1;
            }
            inner
        }
        Tok::Cmd(name) => {
            *pos += 1;
            parse_cmd(&name, toks, pos)
        }
        Tok::Sym(s) => {
            *pos += 1;
            MathNode::Run(s)
        }
        _ => {
            *pos += 1;
            MathNode::Run(String::new())
        }
    }
}

fn parse_cmd(name: &str, toks: &[Tok], pos: &mut usize) -> MathNode {
    match name {
        "frac" => {
            let num = parse_atom(toks, pos);
            let den = parse_atom(toks, pos);
            MathNode::Fraction(Box::new(num), Box::new(den))
        }
        "sqrt" => {
            // optional [degree] — represented in LaTeX with brackets, lexed as Syms
            if *pos < toks.len() && toks[*pos] == Tok::Sym("[".into()) {
                *pos += 1;
                let mut deg = Vec::new();
                while *pos < toks.len() && toks[*pos] != Tok::Sym("]".into()) {
                    deg.push(parse_atom(toks, pos));
                }
                if *pos < toks.len() {
                    *pos += 1;
                } // skip ]
                let radicand = parse_atom(toks, pos);
                let d = if deg.len() == 1 {
                    deg.pop().unwrap()
                } else {
                    MathNode::Seq(deg)
                };
                MathNode::Radical(Some(Box::new(d)), Box::new(radicand))
            } else {
                MathNode::Radical(None, Box::new(parse_atom(toks, pos)))
            }
        }
        "left" => {
            // \left( ... \right)
            let beg = take_delim(toks, pos);
            let mut inner = Vec::new();
            while *pos < toks.len() {
                if let Tok::Cmd(c) = &toks[*pos]
                    && c == "right"
                {
                    *pos += 1;
                    let _ = take_delim(toks, pos);
                    break;
                }
                inner.push(parse_atom(toks, pos));
            }
            let end = ")".to_string();
            MathNode::Delimiter(beg, end, vec![flatten(inner)])
        }
        "sum" | "int" | "prod" => {
            let chr = match name {
                "sum" => "\u{2211}",
                "int" => "\u{222B}",
                _ => "\u{220F}",
            };
            let (mut lo, mut hi) = (MathNode::Run(String::new()), MathNode::Run(String::new()));
            // consume _.. ^.. in any order
            for _ in 0..2 {
                if *pos < toks.len() && toks[*pos] == Tok::Sub {
                    *pos += 1;
                    lo = parse_atom(toks, pos);
                } else if *pos < toks.len() && toks[*pos] == Tok::Sup {
                    *pos += 1;
                    hi = parse_atom(toks, pos);
                }
            }
            let body = parse_atom(toks, pos);
            MathNode::Nary(chr.into(), Box::new(lo), Box::new(hi), Box::new(body))
        }
        "hat" => MathNode::Accent("\u{0302}".into(), Box::new(parse_atom(toks, pos))),
        "bar" | "overline" => MathNode::Accent("\u{0305}".into(), Box::new(parse_atom(toks, pos))),
        "vec" => MathNode::Accent("\u{20D7}".into(), Box::new(parse_atom(toks, pos))),
        "sin" | "cos" | "tan" | "log" | "ln" | "exp" | "lim" => {
            MathNode::Func(name.to_string(), Box::new(parse_atom(toks, pos)))
        }
        "cdot" => MathNode::Run("\u{22C5}".into()),
        "times" => MathNode::Run("\u{00D7}".into()),
        "pm" => MathNode::Run("\u{00B1}".into()),
        "leq" | "le" => MathNode::Run("\u{2264}".into()),
        "geq" | "ge" => MathNode::Run("\u{2265}".into()),
        "neq" | "ne" => MathNode::Run("\u{2260}".into()),
        "infty" => MathNode::Run("\u{221E}".into()),
        "to" | "rightarrow" => MathNode::Run("\u{2192}".into()),
        other => MathNode::Run(greek(other).unwrap_or(other).to_string()),
    }
}

fn flatten(mut v: Vec<MathNode>) -> MathNode {
    if v.len() == 1 {
        v.pop().unwrap()
    } else {
        MathNode::Seq(v)
    }
}

fn take_delim(toks: &[Tok], pos: &mut usize) -> String {
    if *pos < toks.len()
        && let Tok::Sym(s) = &toks[*pos]
    {
        let d = s.clone();
        *pos += 1;
        return d;
    }
    "(".to_string()
}

/// Map a LaTeX Greek command to its Unicode character.
fn greek(name: &str) -> Option<&'static str> {
    Some(match name {
        "alpha" => "\u{03B1}",
        "beta" => "\u{03B2}",
        "gamma" => "\u{03B3}",
        "delta" => "\u{03B4}",
        "epsilon" => "\u{03B5}",
        "theta" => "\u{03B8}",
        "lambda" => "\u{03BB}",
        "mu" => "\u{03BC}",
        "pi" => "\u{03C0}",
        "rho" => "\u{03C1}",
        "sigma" => "\u{03C3}",
        "phi" => "\u{03C6}",
        "omega" => "\u{03C9}",
        "tau" => "\u{03C4}",
        "Gamma" => "\u{0393}",
        "Delta" => "\u{0394}",
        "Theta" => "\u{0398}",
        "Lambda" => "\u{039B}",
        "Pi" => "\u{03A0}",
        "Sigma" => "\u{03A3}",
        "Phi" => "\u{03A6}",
        "Omega" => "\u{03A9}",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(n: &MathNode) -> String {
        let mut w = Writer::new(Vec::new());
        n.write(&mut w).unwrap();
        String::from_utf8(w.into_inner()).unwrap()
    }

    #[test]
    fn fraction() {
        let x = s(&MathNode::Fraction(
            Box::new(MathNode::run("1")),
            Box::new(MathNode::run("2")),
        ));
        assert!(x.contains("<m:f>"), "{x}");
        assert!(x.contains("<m:num><m:r><m:t>1</m:t>"), "{x}");
        assert!(x.contains("<m:den><m:r><m:t>2</m:t>"), "{x}");
    }

    #[test]
    fn power_and_sqrt() {
        let p = s(&MathNode::Sup(
            Box::new(MathNode::run("x")),
            Box::new(MathNode::run("2")),
        ));
        assert!(p.contains("<m:sSup>"), "{p}");
        let r = s(&MathNode::Radical(None, Box::new(MathNode::run("x"))));
        assert!(r.contains("<m:rad>"), "{r}");
        assert!(r.contains(r#"<m:degHide m:val="1"/>"#), "{r}");
    }

    #[test]
    fn nary_sum_and_delim() {
        let sum = s(&MathNode::Nary(
            "\u{2211}".into(),
            Box::new(MathNode::run("i=1")),
            Box::new(MathNode::run("n")),
            Box::new(MathNode::run("i")),
        ));
        assert!(sum.contains("<m:nary>"), "{sum}");
        assert!(sum.contains(r#"<m:chr m:val="∑"/>"#), "{sum}");
        let d = s(&MathNode::Delimiter(
            "(".into(),
            ")".into(),
            vec![MathNode::run("x")],
        ));
        assert!(d.contains(r#"<m:begChr m:val="("/>"#), "{d}");
    }

    #[test]
    fn omath_para_wraps_with_ns() {
        let bytes = MathNode::run("x").to_omath_para_bytes().unwrap();
        let x = String::from_utf8(bytes).unwrap();
        assert!(x.contains("m:oMathPara"), "{x}");
        assert!(x.contains("xmlns:m="), "{x}");
        assert!(x.contains("<m:oMath>"), "{x}");
    }

    #[test]
    fn matrix_and_bar() {
        let m = MathNode::Matrix(vec![
            vec![MathNode::run("a"), MathNode::run("b")],
            vec![MathNode::run("c"), MathNode::run("d")],
        ]);
        let x = s(&m);
        assert!(x.contains("<m:m>"), "{x}");
        assert!(x.contains("<m:mr>"), "{x}");
        assert_eq!(x.matches("<m:mr>").count(), 2, "{x}");
        let b = s(&MathNode::Bar(Box::new(MathNode::run("x"))));
        assert!(b.contains("<m:bar>"), "{b}");
        assert!(b.contains("<m:barPr>"), "{b}");
    }

    #[test]
    fn latex_fraction_power() {
        let n = from_latex(r"\frac{a}{b}^2");
        let x = s(&n);
        // \frac{a}{b} then ^2 binds to the fraction
        assert!(x.contains("<m:sSup>"), "{x}");
        assert!(x.contains("<m:f>"), "{x}");
        assert!(x.contains("<m:t>a</m:t>"), "{x}");
        assert!(x.contains("<m:t>2</m:t>"), "{x}");
    }

    #[test]
    fn latex_sqrt_sum_greek() {
        let r = s(&from_latex(r"\sqrt{x}"));
        assert!(r.contains("<m:rad>"), "{r}");
        let sum = s(&from_latex(r"\sum_{i=1}^{n} i"));
        assert!(sum.contains("<m:nary>"), "{sum}");
        assert!(sum.contains(r#"<m:chr m:val="∑"/>"#), "{sum}");
        let g = s(&from_latex(r"\alpha + \beta"));
        assert!(g.contains("<m:t>α</m:t>"), "{g}");
        assert!(g.contains("<m:t>β</m:t>"), "{g}");
    }

    #[test]
    fn latex_subscript_and_func() {
        let sub = s(&from_latex(r"x_{i}"));
        assert!(sub.contains("<m:sSub>"), "{sub}");
        let f = s(&from_latex(r"\sin{x}"));
        assert!(f.contains("<m:func>"), "{f}");
        assert!(f.contains("<m:t>sin</m:t>"), "{f}");
    }
}
