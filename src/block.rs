pub const PLAIN: char = '\0';
pub const ROW: char = 'R';
pub const CONTAINER_START: char = 'S';
pub const CONTAINER_END: char = 'E';
pub const GRID_ROW: char = 'G';
pub const FLEX_ROW: char = 'F';
pub const EACH_START: char = 'L';
pub const EACH_END: char = 'l';

pub fn is_container_boundary(format: char) -> bool {
    format == CONTAINER_START || format == CONTAINER_END
        || format == EACH_START || format == EACH_END
}

pub fn is_content_row(format: char) -> bool {
    !is_container_boundary(format)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_boundaries() {
        assert!(is_container_boundary(CONTAINER_START));
        assert!(is_container_boundary(CONTAINER_END));
        assert!(!is_container_boundary(PLAIN));
        assert!(!is_container_boundary(ROW));
    }

    #[test]
    fn test_content_rows() {
        assert!(is_content_row(PLAIN));
        assert!(is_content_row(ROW));
        assert!(is_content_row(GRID_ROW));
        assert!(is_content_row(FLEX_ROW));
        assert!(!is_content_row(CONTAINER_START));
    }
}
