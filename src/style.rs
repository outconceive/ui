pub const DEFAULT: char = ' ';
pub const PRIMARY: char = 'p';
pub const SECONDARY: char = 's';
pub const DANGER: char = 'd';
pub const WARNING: char = 'w';
pub const INFO: char = 'i';
pub const DARK: char = 'k';
pub const LIGHT: char = 'l';
pub const OUTLINE: char = 'o';
pub const GHOST: char = 'g';

pub fn css_class_for(style: char) -> &'static str {
    match style {
        PRIMARY => "mc-primary",
        SECONDARY => "mc-secondary",
        DANGER => "mc-danger",
        WARNING => "mc-warning",
        INFO => "mc-info",
        DARK => "mc-dark",
        LIGHT => "mc-light",
        OUTLINE => "mc-outline",
        GHOST => "mc-ghost",
        '1'..='9' => match style {
            '1' => "mc-size-1",
            '2' => "mc-size-2",
            '3' => "mc-size-3",
            '4' => "mc-size-4",
            '5' => "mc-size-5",
            '6' => "mc-size-6",
            '7' => "mc-size-7",
            '8' => "mc-size-8",
            '9' => "mc-size-9",
            _ => "",
        },
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_classes() {
        assert_eq!(css_class_for(PRIMARY), "mc-primary");
        assert_eq!(css_class_for(SECONDARY), "mc-secondary");
        assert_eq!(css_class_for(DANGER), "mc-danger");
        assert_eq!(css_class_for(DEFAULT), "");
    }

    #[test]
    fn test_size_classes() {
        assert_eq!(css_class_for('1'), "mc-size-1");
        assert_eq!(css_class_for('5'), "mc-size-5");
        assert_eq!(css_class_for('9'), "mc-size-9");
    }
}
