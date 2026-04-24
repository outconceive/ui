use std::collections::{HashMap, HashSet, VecDeque};
use crate::constraint::*;
use crate::component;
use crate::line::Line;

pub fn intrinsic_size(comp: char, content_len: usize) -> (f64, f64) {
    match comp {
        component::TEXT_INPUT | component::PASSWORD_INPUT => (200.0, 36.0),
        component::BUTTON => {
            let w = (content_len as f64 * 9.0).max(80.0);
            (w, 36.0)
        }
        component::LABEL => {
            let w = (content_len as f64 * 8.0).max(20.0);
            (w, 24.0)
        }
        component::CHECKBOX | component::RADIO => (20.0, 20.0),
        component::SELECT => (160.0, 36.0),
        component::TEXTAREA => (300.0, 100.0),
        component::DIVIDER => (400.0, 1.0),
        component::IMAGE => (120.0, 120.0),
        component::PILL => {
            let w = (content_len as f64 * 8.0).max(40.0);
            (w, 28.0)
        }
        component::BADGE => (32.0, 20.0),
        component::PROGRESS => (200.0, 8.0),
        component::SPARKLINE => (100.0, 30.0),
        _ => {
            let w = (content_len as f64 * 8.0).max(40.0);
            (w, 24.0)
        }
    }
}

pub fn solve_layout(lines: &[Line]) -> SolvedLayout {
    let mut elements = Vec::new();
    let mut anon_counter = 0usize;

    for line in lines {
        let spans = line.spans();
        for span in &spans {
            let name = span.state_key.clone().unwrap_or_else(|| {
                let n = format!("_anon_{}", anon_counter);
                anon_counter += 1;
                n
            });

            let raw_constraints = span.constraints.as_ref()
                .map(|v| v.iter().filter_map(|s| parse_constraint(s)).collect::<Vec<_>>())
                .unwrap_or_default();

            let content_len = span.content.trim().len();
            let (w, h) = intrinsic_size(span.component, content_len);

            elements.push(ConstrainedElement {
                name,
                component: span.component,
                content: span.content.clone(),
                constraints: raw_constraints,
                intrinsic_width: w,
                intrinsic_height: h,
            });
        }
    }

    solve(&elements)
}

pub fn solve(elements: &[ConstrainedElement]) -> SolvedLayout {
    let name_set: HashSet<&str> = elements.iter().map(|e| e.name.as_str()).collect();

    // Build dependency graph
    let mut deps: HashMap<&str, Vec<&str>> = HashMap::new();
    for el in elements {
        let mut el_deps = Vec::new();
        for c in &el.constraints {
            for r in c.references() {
                if name_set.contains(r) && r != el.name {
                    el_deps.push(r);
                }
            }
        }
        deps.insert(el.name.as_str(), el_deps);
    }

    // Topological sort (Kahn's algorithm)
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    for el in elements {
        in_degree.entry(el.name.as_str()).or_insert(0);
        for d in deps.get(el.name.as_str()).unwrap_or(&vec![]) {
            *in_degree.entry(d).or_insert(0) += 0;
        }
    }

    let mut reverse_deps: HashMap<&str, Vec<&str>> = HashMap::new();
    for (node, node_deps) in &deps {
        for d in node_deps {
            reverse_deps.entry(*d).or_default().push(node);
        }
        *in_degree.entry(node).or_insert(0) += node_deps.len();
    }

    let mut queue: VecDeque<&str> = VecDeque::new();
    for (node, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(node);
        }
    }

    let mut order: Vec<&str> = Vec::new();
    while let Some(node) = queue.pop_front() {
        order.push(node);
        if let Some(dependents) = reverse_deps.get(node) {
            for dep in dependents {
                if let Some(deg) = in_degree.get_mut(dep) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(dep);
                    }
                }
            }
        }
    }

    // Elements not in order (cycles) get appended at the end
    for el in elements {
        if !order.contains(&el.name.as_str()) {
            order.push(el.name.as_str());
        }
    }

    let el_map: HashMap<&str, &ConstrainedElement> = elements.iter()
        .map(|e| (e.name.as_str(), e))
        .collect();

    let mut solved: HashMap<String, Rect> = HashMap::new();

    for name in &order {
        let el = match el_map.get(name) {
            Some(e) => e,
            None => continue,
        };

        let mut rect = Rect {
            x: 0.0,
            y: 0.0,
            width: el.intrinsic_width,
            height: el.intrinsic_height,
        };

        // Resolve implicit refs for gap constraints
        let first_ref = el.constraints.iter()
            .flat_map(|c| c.references())
            .next()
            .map(|s| s.to_string());

        for c in &el.constraints {
            match c {
                ConstraintKind::Left(r) => {
                    if let Some(ref_rect) = solved.get(r) {
                        rect.x = ref_rect.x;
                    }
                }
                ConstraintKind::Right(r) => {
                    if let Some(ref_rect) = solved.get(r) {
                        rect.x = ref_rect.x + ref_rect.width - rect.width;
                    }
                }
                ConstraintKind::Top(r) => {
                    if let Some(ref_rect) = solved.get(r) {
                        rect.y = ref_rect.y;
                    }
                }
                ConstraintKind::Bottom(r) => {
                    if let Some(ref_rect) = solved.get(r) {
                        rect.y = ref_rect.y + ref_rect.height - rect.height;
                    }
                }
                ConstraintKind::CenterX(r) => {
                    if let Some(ref_rect) = solved.get(r) {
                        rect.x = ref_rect.x + ref_rect.width / 2.0 - rect.width / 2.0;
                    }
                }
                ConstraintKind::CenterY(r) => {
                    if let Some(ref_rect) = solved.get(r) {
                        rect.y = ref_rect.y + ref_rect.height / 2.0 - rect.height / 2.0;
                    }
                }
                ConstraintKind::GapX(gap, ref_opt) => {
                    let r = ref_opt.as_ref().or(first_ref.as_ref());
                    if let Some(ref_rect) = r.and_then(|n| solved.get(n.as_str())) {
                        rect.x = ref_rect.x + ref_rect.width + gap.pixels;
                    }
                }
                ConstraintKind::GapY(gap, ref_opt) => {
                    let r = ref_opt.as_ref().or(first_ref.as_ref());
                    if let Some(ref_rect) = r.and_then(|n| solved.get(n.as_str())) {
                        rect.y = ref_rect.y + ref_rect.height + gap.pixels;
                    }
                }
                ConstraintKind::MatchWidth(r) => {
                    if let Some(ref_rect) = solved.get(r) {
                        rect.width = ref_rect.width;
                    }
                }
                ConstraintKind::MatchHeight(r) => {
                    if let Some(ref_rect) = solved.get(r) {
                        rect.height = ref_rect.height;
                    }
                }
                ConstraintKind::DistributeX(a, b) => {
                    if let (Some(ra), Some(rb)) = (solved.get(a), solved.get(b)) {
                        let mid = (ra.x + ra.width + rb.x) / 2.0;
                        rect.x = mid - rect.width / 2.0;
                    }
                }
                ConstraintKind::DistributeY(a, b) => {
                    if let (Some(ra), Some(rb)) = (solved.get(a), solved.get(b)) {
                        let mid = (ra.y + ra.height + rb.y) / 2.0;
                        rect.y = mid - rect.height / 2.0;
                    }
                }
            }
        }

        // Stretch between left: and right: constraints
        let left_ref = el.constraints.iter().find_map(|c| match c {
            ConstraintKind::Left(r) => Some(r.as_str()),
            _ => None,
        });
        let right_ref = el.constraints.iter().find_map(|c| match c {
            ConstraintKind::Right(r) => Some(r.as_str()),
            _ => None,
        });
        if let (Some(lr), Some(rr)) = (left_ref, right_ref) {
            if let (Some(l_rect), Some(r_rect)) = (solved.get(lr), solved.get(rr)) {
                rect.x = l_rect.x;
                rect.width = (r_rect.x + r_rect.width) - l_rect.x;
            }
        }

        solved.insert(el.name.to_string(), rect);
    }

    let mut max_x: f64 = 0.0;
    let mut max_y: f64 = 0.0;
    let mut result_elements = Vec::new();

    for el in elements {
        if let Some(rect) = solved.get(&el.name) {
            let right = rect.x + rect.width;
            let bottom = rect.y + rect.height;
            if right > max_x { max_x = right; }
            if bottom > max_y { max_y = bottom; }
            result_elements.push((el.name.clone(), rect.clone()));
        }
    }

    SolvedLayout {
        elements: result_elements,
        container_width: max_x,
        container_height: max_y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn el(name: &str, comp: char, constraints: Vec<ConstraintKind>) -> ConstrainedElement {
        let (w, h) = intrinsic_size(comp, 6);
        ConstrainedElement {
            name: name.to_string(),
            component: comp,
            content: "      ".to_string(),
            constraints,
            intrinsic_width: w,
            intrinsic_height: h,
        }
    }

    #[test]
    fn test_single_element_at_origin() {
        let elements = vec![el("title", component::LABEL, vec![])];
        let layout = solve(&elements);
        assert_eq!(layout.elements.len(), 1);
        assert_eq!(layout.elements[0].1.x, 0.0);
        assert_eq!(layout.elements[0].1.y, 0.0);
    }

    #[test]
    fn test_gap_y() {
        let elements = vec![
            el("title", component::LABEL, vec![]),
            el("search", component::TEXT_INPUT, vec![
                ConstraintKind::GapY(GapValue { pixels: 16.0, raw: "16px".into() }, Some("title".into())),
            ]),
        ];
        let layout = solve(&elements);
        let title = &layout.elements[0].1;
        let search = &layout.elements[1].1;
        assert_eq!(search.y, title.y + title.height + 16.0);
    }

    #[test]
    fn test_center_x() {
        let elements = vec![
            el("title", component::LABEL, vec![]),
            el("search", component::TEXT_INPUT, vec![
                ConstraintKind::CenterX("title".into()),
                ConstraintKind::GapY(GapValue { pixels: 16.0, raw: "16px".into() }, Some("title".into())),
            ]),
        ];
        let layout = solve(&elements);
        let title = &layout.elements[0].1;
        let search = &layout.elements[1].1;
        let title_center = title.x + title.width / 2.0;
        let search_center = search.x + search.width / 2.0;
        assert!((title_center - search_center).abs() < 0.1);
    }

    #[test]
    fn test_right_align() {
        let elements = vec![
            el("search", component::TEXT_INPUT, vec![]),
            el("go", component::BUTTON, vec![
                ConstraintKind::Right("search".into()),
                ConstraintKind::CenterY("search".into()),
                ConstraintKind::GapX(GapValue { pixels: 8.0, raw: "8px".into() }, Some("search".into())),
            ]),
        ];
        let layout = solve(&elements);
        let search = &layout.elements[0].1;
        let go = &layout.elements[1].1;
        assert_eq!(go.x, search.x + search.width + 8.0);
    }

    #[test]
    fn test_match_width() {
        let elements = vec![
            el("title", component::TEXT_INPUT, vec![]),
            el("desc", component::TEXT_INPUT, vec![
                ConstraintKind::MatchWidth("title".into()),
                ConstraintKind::GapY(GapValue { pixels: 8.0, raw: "8px".into() }, Some("title".into())),
            ]),
        ];
        let layout = solve(&elements);
        assert_eq!(layout.elements[0].1.width, layout.elements[1].1.width);
    }

    #[test]
    fn test_three_element_chain() {
        let elements = vec![
            el("a", component::LABEL, vec![]),
            el("b", component::TEXT_INPUT, vec![
                ConstraintKind::GapY(GapValue { pixels: 10.0, raw: "10px".into() }, Some("a".into())),
                ConstraintKind::Left("a".into()),
            ]),
            el("c", component::BUTTON, vec![
                ConstraintKind::GapY(GapValue { pixels: 10.0, raw: "10px".into() }, Some("b".into())),
                ConstraintKind::Left("a".into()),
            ]),
        ];
        let layout = solve(&elements);
        let a = &layout.elements[0].1;
        let b = &layout.elements[1].1;
        let c = &layout.elements[2].1;
        assert_eq!(b.x, a.x);
        assert_eq!(c.x, a.x);
        assert_eq!(b.y, a.y + a.height + 10.0);
        assert_eq!(c.y, b.y + b.height + 10.0);
    }

    #[test]
    fn test_divider_stretch() {
        let elements = vec![
            el("title", component::LABEL, vec![]),
            el("go", component::BUTTON, vec![
                ConstraintKind::GapX(GapValue { pixels: 100.0, raw: "100px".into() }, Some("title".into())),
                ConstraintKind::CenterY("title".into()),
            ]),
            el("line", component::DIVIDER, vec![
                ConstraintKind::Left("title".into()),
                ConstraintKind::Right("go".into()),
                ConstraintKind::GapY(GapValue { pixels: 16.0, raw: "16px".into() }, Some("title".into())),
            ]),
        ];
        let layout = solve(&elements);
        let title = &layout.elements[0].1;
        let go = &layout.elements[1].1;
        let line = &layout.elements[2].1;
        assert_eq!(line.x, title.x);
        assert_eq!(line.width, (go.x + go.width) - title.x);
    }

    #[test]
    fn test_container_bounds() {
        let elements = vec![
            el("a", component::LABEL, vec![]),
            el("b", component::BUTTON, vec![
                ConstraintKind::GapX(GapValue { pixels: 50.0, raw: "50px".into() }, Some("a".into())),
            ]),
        ];
        let layout = solve(&elements);
        let a = &layout.elements[0].1;
        let b = &layout.elements[1].1;
        assert_eq!(layout.container_width, b.x + b.width);
        assert_eq!(layout.container_height, (a.y + a.height).max(b.y + b.height));
    }

    #[test]
    fn test_implicit_gap_ref() {
        let elements = vec![
            el("title", component::LABEL, vec![]),
            el("search", component::TEXT_INPUT, vec![
                ConstraintKind::CenterX("title".into()),
                ConstraintKind::GapY(GapValue { pixels: 16.0, raw: "16px".into() }, None),
            ]),
        ];
        let layout = solve(&elements);
        let title = &layout.elements[0].1;
        let search = &layout.elements[1].1;
        assert_eq!(search.y, title.y + title.height + 16.0);
    }
}
