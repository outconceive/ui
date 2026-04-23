pub const EMPTY: char = ' ';
pub const LABEL: char = 'L';
pub const TEXT_INPUT: char = 'I';
pub const PASSWORD_INPUT: char = 'P';
pub const BUTTON: char = 'B';
pub const CHECKBOX: char = 'C';
pub const RADIO: char = 'R';
pub const SELECT: char = 'S';
pub const TEXTAREA: char = 'T';
pub const IMAGE: char = 'G';
pub const LINK: char = 'K';
pub const DIVIDER: char = 'D';
pub const CUSTOM: char = 'X';
pub const SPACER: char = '_';
pub const PILL: char = 'W';
pub const BADGE: char = 'J';
pub const PROGRESS: char = 'Q';
pub const SPARKLINE: char = 'Z';
pub const CONTINUATION: char = '.';

pub fn tag_for(component: char) -> &'static str {
    match component {
        LABEL => "span",
        TEXT_INPUT => "input",
        PASSWORD_INPUT => "input",
        BUTTON => "button",
        CHECKBOX => "input",
        RADIO => "input",
        SELECT => "select",
        TEXTAREA => "textarea",
        IMAGE => "img",
        LINK => "a",
        DIVIDER => "hr",
        SPACER => "span",
        PILL => "span",
        BADGE => "span",
        PROGRESS => "div",
        SPARKLINE => "svg",
        _ => "span",
    }
}

pub fn css_class_for(component: char) -> &'static str {
    match component {
        LABEL => "mc-label",
        TEXT_INPUT => "mc-input",
        PASSWORD_INPUT => "mc-input mc-input-password",
        BUTTON => "mc-button",
        CHECKBOX => "mc-checkbox",
        RADIO => "mc-radio",
        SELECT => "mc-select",
        TEXTAREA => "mc-textarea",
        IMAGE => "mc-image",
        LINK => "mc-link",
        DIVIDER => "mc-divider",
        SPACER => "mc-spacer",
        PILL => "mc-pill",
        BADGE => "mc-badge",
        PROGRESS => "mc-progress",
        SPARKLINE => "mc-sparkline",
        CUSTOM => "mc-custom",
        _ => "",
    }
}

pub fn input_type_for(component: char) -> Option<&'static str> {
    match component {
        TEXT_INPUT => Some("text"),
        PASSWORD_INPUT => Some("password"),
        CHECKBOX => Some("checkbox"),
        RADIO => Some("radio"),
        _ => None,
    }
}

pub fn is_interactive(component: char) -> bool {
    matches!(
        component,
        TEXT_INPUT | PASSWORD_INPUT | BUTTON | CHECKBOX | RADIO | SELECT | TEXTAREA
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentSpan {
    pub start: usize,
    pub end: usize,
    pub component: char,
    pub content: String,
    pub state_key: Option<String>,
    pub style: Option<char>,
    pub logic_ref: Option<String>,
    pub col: Option<(u8, u8)>,
    pub validate: Option<String>,
    pub responsive: Option<Vec<(String, u8, u8)>>,
    pub animate: Option<String>,
    pub popover: Option<String>,
}

pub fn parse_col(s: &str) -> Option<(u8, u8)> {
    let rest = s.strip_prefix("col-")?;
    if let Some(bracket) = rest.find('[') {
        let n: u8 = rest[..bracket].parse().ok()?;
        let total: u8 = rest[bracket + 1..].trim_end_matches(']').parse().ok()?;
        if n > 0 && total > 0 && n <= total { Some((n, total)) } else { None }
    } else {
        let n: u8 = rest.parse().ok()?;
        if n > 0 && n <= 12 { Some((n, 12)) } else { None }
    }
}

pub fn col_to_css(col: (u8, u8)) -> String {
    let pct = (col.0 as f64 / col.1 as f64) * 100.0;
    format!("flex:0 0 {:.4}%;max-width:{:.4}%", pct, pct)
}

pub fn group_spans(
    content: &str,
    components: &str,
    state_keys: &str,
    styles: &str,
    logic: &Option<std::collections::HashMap<usize, String>>,
) -> Vec<ComponentSpan> {
    let content_chars: Vec<char> = content.chars().collect();
    let comp_chars: Vec<char> = components.chars().collect();
    let key_chars: Vec<char> = state_keys.chars().collect();
    let style_chars: Vec<char> = styles.chars().collect();
    let len = content_chars.len();

    if len == 0 {
        return Vec::new();
    }

    let mut spans = Vec::new();
    let mut i = 0;

    while i < len {
        let comp = comp_chars.get(i).copied().unwrap_or(EMPTY);
        if comp == CONTINUATION {
            i += 1;
            continue;
        }

        let start = i;
        i += 1;
        while i < len {
            let next = comp_chars.get(i).copied().unwrap_or(EMPTY);
            if next == comp || next == CONTINUATION {
                i += 1;
            } else {
                break;
            }
        }

        let span_content: String = content_chars[start..i].iter().collect();
        let span_keys: String = key_chars[start..i.min(key_chars.len())]
            .iter()
            .collect::<String>()
            .trim_matches('_')
            .to_string();

        let state_key = if span_keys.is_empty() {
            None
        } else {
            Some(span_keys)
        };

        let style = style_chars.get(start).copied().filter(|&c| c != ' ');

        let logic_ref = logic
            .as_ref()
            .and_then(|l| l.get(&start).cloned());

        spans.push(ComponentSpan {
            start,
            end: i,
            component: comp,
            content: span_content,
            state_key,
            style,
            logic_ref,
            col: None,
            validate: None,
            responsive: None,
            animate: None,
            popover: None,
        });
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_label() {
        let spans = group_spans("Hello", "LLLLL", "_____", "     ", &None);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].component, LABEL);
        assert_eq!(spans[0].content, "Hello");
        assert_eq!(spans[0].start, 0);
        assert_eq!(spans[0].end, 5);
        assert!(spans[0].state_key.is_none());
    }

    #[test]
    fn test_label_and_input() {
        let spans = group_spans(
            "Name  ______",
            "LLLLLLIIIIIII",
            "______name___",
            "             ",
            &None,
        );
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].component, LABEL);
        assert_eq!(spans[0].content, "Name  ");
        assert_eq!(spans[1].component, TEXT_INPUT);
        assert_eq!(spans[1].content, "______");
        assert_eq!(spans[1].state_key, Some("name".to_string()));
    }

    #[test]
    fn test_three_components() {
        let spans = group_spans(
            "User  _______Go_",
            "LLLLLLIIIIIIIBBB",
            "______user___sub",
            "             ppp",
            &None,
        );
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].component, LABEL);
        assert_eq!(spans[1].component, TEXT_INPUT);
        assert_eq!(spans[1].state_key, Some("user".to_string()));
        assert_eq!(spans[2].component, BUTTON);
        assert!(spans[2].content.contains("Go"));
        assert_eq!(spans[2].state_key, Some("sub".to_string()));
        assert_eq!(spans[2].style, Some('p'));
    }

    #[test]
    fn test_continuation_markers() {
        let spans = group_spans(
            "Submit",
            "B.....",
            "action",
            "pppppp",
            &None,
        );
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].component, BUTTON);
        assert_eq!(spans[0].content, "Submit");
        assert_eq!(spans[0].end, 6);
    }

    #[test]
    fn test_empty_content() {
        let spans = group_spans("", "", "", "", &None);
        assert!(spans.is_empty());
    }

    #[test]
    fn test_style_mapping() {
        let spans = group_spans("OK", "BB", "__", "dd", &None);
        assert_eq!(spans[0].style, Some('d'));
    }

    #[test]
    fn test_logic_ref() {
        let mut logic = std::collections::HashMap::new();
        logic.insert(0, "handler_fn".to_string());
        let spans = group_spans("Go", "BB", "do", "pp", &Some(logic));
        assert_eq!(spans[0].logic_ref, Some("handler_fn".to_string()));
    }

    #[test]
    fn test_tag_for_components() {
        assert_eq!(tag_for(LABEL), "span");
        assert_eq!(tag_for(TEXT_INPUT), "input");
        assert_eq!(tag_for(BUTTON), "button");
        assert_eq!(tag_for(CHECKBOX), "input");
        assert_eq!(tag_for(SELECT), "select");
        assert_eq!(tag_for(TEXTAREA), "textarea");
        assert_eq!(tag_for(IMAGE), "img");
        assert_eq!(tag_for(LINK), "a");
        assert_eq!(tag_for(DIVIDER), "hr");
    }

    #[test]
    fn test_input_types() {
        assert_eq!(input_type_for(TEXT_INPUT), Some("text"));
        assert_eq!(input_type_for(PASSWORD_INPUT), Some("password"));
        assert_eq!(input_type_for(CHECKBOX), Some("checkbox"));
        assert_eq!(input_type_for(RADIO), Some("radio"));
        assert_eq!(input_type_for(LABEL), None);
    }

    #[test]
    fn test_interactive() {
        assert!(is_interactive(TEXT_INPUT));
        assert!(is_interactive(BUTTON));
        assert!(is_interactive(CHECKBOX));
        assert!(!is_interactive(LABEL));
        assert!(!is_interactive(EMPTY));
        assert!(!is_interactive(DIVIDER));
    }
}
