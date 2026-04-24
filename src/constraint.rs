use serde::Serialize;

// PartialEq compares pixels and raw string — sufficient for tests
#[derive(Debug, Clone)]
pub struct GapValue {
    pub pixels: f64,
    pub raw: String,
}

impl GapValue {
    pub fn from_str(s: &str) -> Option<Self> {
        if let Some(rest) = s.strip_suffix("rem") {
            let n: f64 = rest.parse().ok()?;
            Some(Self { pixels: n * 16.0, raw: s.to_string() })
        } else if let Some(rest) = s.strip_suffix("em") {
            let n: f64 = rest.parse().ok()?;
            Some(Self { pixels: n * 16.0, raw: s.to_string() })
        } else if let Some(rest) = s.strip_suffix("px") {
            let n: f64 = rest.parse().ok()?;
            Some(Self { pixels: n, raw: s.to_string() })
        } else if let Some(rest) = s.strip_suffix('%') {
            let n: f64 = rest.parse().ok()?;
            Some(Self { pixels: n, raw: s.to_string() })
        } else {
            let n: f64 = s.parse().ok()?;
            Some(Self { pixels: n, raw: format!("{}px", n) })
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConstraintKind {
    Left(String),
    Right(String),
    Top(String),
    Bottom(String),
    CenterX(String),
    CenterY(String),
    GapX(GapValue, Option<String>),
    GapY(GapValue, Option<String>),
    MatchWidth(String),
    MatchHeight(String),
    DistributeX(String, String),
    DistributeY(String, String),
}

impl ConstraintKind {
    pub fn references(&self) -> Vec<&str> {
        match self {
            Self::Left(r) | Self::Right(r) | Self::Top(r) | Self::Bottom(r)
            | Self::CenterX(r) | Self::CenterY(r)
            | Self::MatchWidth(r) | Self::MatchHeight(r) => vec![r.as_str()],
            Self::GapX(_, Some(r)) | Self::GapY(_, Some(r)) => vec![r.as_str()],
            Self::GapX(_, None) | Self::GapY(_, None) => vec![],
            Self::DistributeX(a, b) | Self::DistributeY(a, b) => vec![a.as_str(), b.as_str()],
        }
    }
}

pub fn parse_constraint(s: &str) -> Option<ConstraintKind> {
    if let Some(r) = s.strip_prefix("left:") {
        return Some(ConstraintKind::Left(r.to_string()));
    }
    if let Some(r) = s.strip_prefix("right:") {
        return Some(ConstraintKind::Right(r.to_string()));
    }
    if let Some(r) = s.strip_prefix("top:") {
        return Some(ConstraintKind::Top(r.to_string()));
    }
    if let Some(r) = s.strip_prefix("bottom:") {
        return Some(ConstraintKind::Bottom(r.to_string()));
    }
    if let Some(r) = s.strip_prefix("center-x:") {
        return Some(ConstraintKind::CenterX(r.to_string()));
    }
    if let Some(r) = s.strip_prefix("center-y:") {
        return Some(ConstraintKind::CenterY(r.to_string()));
    }
    if let Some(rest) = s.strip_prefix("gap-x:") {
        return parse_gap(rest, false);
    }
    if let Some(rest) = s.strip_prefix("gap-y:") {
        return parse_gap(rest, true);
    }
    if let Some(rest) = s.strip_prefix("width:") {
        if rest.parse::<f64>().is_err() {
            return Some(ConstraintKind::MatchWidth(rest.to_string()));
        }
    }
    if let Some(rest) = s.strip_prefix("height:") {
        if rest.parse::<f64>().is_err() {
            return Some(ConstraintKind::MatchHeight(rest.to_string()));
        }
    }
    if let Some(rest) = s.strip_prefix("distribute-x:") {
        let parts: Vec<&str> = rest.splitn(2, ':').collect();
        if parts.len() == 2 {
            return Some(ConstraintKind::DistributeX(parts[0].to_string(), parts[1].to_string()));
        }
    }
    if let Some(rest) = s.strip_prefix("distribute-y:") {
        let parts: Vec<&str> = rest.splitn(2, ':').collect();
        if parts.len() == 2 {
            return Some(ConstraintKind::DistributeY(parts[0].to_string(), parts[1].to_string()));
        }
    }
    None
}

fn parse_gap(rest: &str, vertical: bool) -> Option<ConstraintKind> {
    let parts: Vec<&str> = rest.splitn(2, ':').collect();
    let gap = GapValue::from_str(parts[0])?;
    let reference = parts.get(1).map(|s| s.to_string());
    if vertical {
        Some(ConstraintKind::GapY(gap, reference))
    } else {
        Some(ConstraintKind::GapX(gap, reference))
    }
}

pub fn is_constraint_token(s: &str) -> bool {
    s.starts_with("left:") || s.starts_with("right:") ||
    s.starts_with("top:") || s.starts_with("bottom:") ||
    s.starts_with("center-x:") || s.starts_with("center-y:") ||
    s.starts_with("gap-x:") || s.starts_with("gap-y:") ||
    s.starts_with("distribute-x:") || s.starts_with("distribute-y:") ||
    (s.starts_with("width:") && s[6..].parse::<f64>().is_err()) ||
    (s.starts_with("height:") && s[7..].parse::<f64>().is_err())
}

#[derive(Debug, Clone, Serialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct ConstrainedElement {
    pub name: String,
    pub component: char,
    pub content: String,
    pub constraints: Vec<ConstraintKind>,
    pub intrinsic_width: f64,
    pub intrinsic_height: f64,
}

#[derive(Debug, Clone)]
pub struct SolvedLayout {
    pub elements: Vec<(String, Rect)>,
    pub container_width: f64,
    pub container_height: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_left() {
        match parse_constraint("left:title").unwrap() {
            ConstraintKind::Left(r) => assert_eq!(r, "title"),
            _ => panic!("expected Left"),
        }
    }

    #[test]
    fn test_parse_center_x() {
        match parse_constraint("center-x:header").unwrap() {
            ConstraintKind::CenterX(r) => assert_eq!(r, "header"),
            _ => panic!("expected CenterX"),
        }
    }

    #[test]
    fn test_parse_gap_y_with_ref() {
        let c = parse_constraint("gap-y:16:title").unwrap();
        match c {
            ConstraintKind::GapY(gap, Some(r)) => {
                assert_eq!(gap.pixels, 16.0);
                assert_eq!(r, "title");
            }
            _ => panic!("expected GapY"),
        }
    }

    #[test]
    fn test_parse_gap_y_no_ref() {
        let c = parse_constraint("gap-y:16").unwrap();
        match c {
            ConstraintKind::GapY(gap, None) => assert_eq!(gap.pixels, 16.0),
            _ => panic!("expected GapY"),
        }
    }

    #[test]
    fn test_parse_gap_x() {
        let c = parse_constraint("gap-x:8:search").unwrap();
        match c {
            ConstraintKind::GapX(gap, Some(r)) => {
                assert_eq!(gap.pixels, 8.0);
                assert_eq!(r, "search");
            }
            _ => panic!("expected GapX"),
        }
    }

    #[test]
    fn test_parse_gap_rem() {
        let c = parse_constraint("gap-x:1rem:chart1").unwrap();
        match c {
            ConstraintKind::GapX(gap, Some(r)) => {
                assert_eq!(gap.pixels, 16.0);
                assert_eq!(gap.raw, "1rem");
                assert_eq!(r, "chart1");
            }
            _ => panic!("expected GapX"),
        }
    }

    #[test]
    fn test_parse_gap_em() {
        let c = parse_constraint("gap-y:2em").unwrap();
        match c {
            ConstraintKind::GapY(gap, None) => {
                assert_eq!(gap.pixels, 32.0);
                assert_eq!(gap.raw, "2em");
            }
            _ => panic!("expected GapY"),
        }
    }

    #[test]
    fn test_parse_width_ref() {
        match parse_constraint("width:title").unwrap() {
            ConstraintKind::MatchWidth(r) => assert_eq!(r, "title"),
            _ => panic!("expected MatchWidth"),
        }
    }

    #[test]
    fn test_parse_width_numeric_not_constraint() {
        assert!(parse_constraint("width:200").is_none());
    }

    #[test]
    fn test_parse_distribute() {
        match parse_constraint("distribute-x:a:b").unwrap() {
            ConstraintKind::DistributeX(a, b) => { assert_eq!(a, "a"); assert_eq!(b, "b"); }
            _ => panic!("expected DistributeX"),
        }
    }

    #[test]
    fn test_is_constraint_token() {
        assert!(is_constraint_token("left:title"));
        assert!(is_constraint_token("gap-y:16"));
        assert!(is_constraint_token("center-x:foo"));
        assert!(is_constraint_token("width:bar"));
        assert!(!is_constraint_token("width:200"));
        assert!(!is_constraint_token("primary"));
        assert!(!is_constraint_token("col-6"));
    }

    #[test]
    fn test_references() {
        let c = ConstraintKind::Left("title".into());
        assert_eq!(c.references(), vec!["title"]);

        let c = ConstraintKind::GapY(GapValue { pixels: 16.0, raw: "16px".into() }, None);
        assert!(c.references().is_empty());

        let c = ConstraintKind::DistributeX("a".into(), "b".into());
        assert_eq!(c.references(), vec!["a", "b"]);
    }
}
