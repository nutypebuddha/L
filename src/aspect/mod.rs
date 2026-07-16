/// Pure function: Validate a graha index is within 9-graha range.
pub fn validate_graha_index(graha_index: u8) -> bool {
    graha_index < 9
}

/// Pure function: Compute aspect between two grahas by position.
pub fn compute_aspect(left_position: u8, right_position: u8) -> f64 {
    let difference = (left_position as f64 - right_position as f64).abs();
    if difference > 4.5 {
        360.0 - (difference * 40.0)
    } else {
        difference * 40.0
    }
}

/// Pure function: Check if two positions form a conjunction (0 degrees).
pub fn is_conjunction(left_position: u8, right_position: u8) -> bool {
    left_position == right_position
}

/// Pure function: Check if two positions are adjacent (40 degrees).
pub fn is_adjacent(left_position: u8, right_position: u8) -> bool {
    let difference = (left_position as i8 - right_position as i8).abs();
    difference == 1 || difference == 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_graha_index_basic() {
        assert!(validate_graha_index(0));
        assert!(validate_graha_index(8));
        assert!(!validate_graha_index(9));
    }

    #[test]
    fn compute_aspect_basic() {
        assert_eq!(compute_aspect(0, 0), 0.0);
        assert_eq!(compute_aspect(0, 1), 40.0);
        assert_eq!(compute_aspect(0, 3), 120.0);
    }

    #[test]
    fn is_conjunction_basic() {
        assert!(is_conjunction(0, 0));
        assert!(is_conjunction(5, 5));
        assert!(!is_conjunction(0, 1));
    }

    #[test]
    fn is_adjacent_basic() {
        assert!(is_adjacent(0, 1));
        assert!(is_adjacent(8, 0));
        assert!(!is_adjacent(0, 2));
    }
}
