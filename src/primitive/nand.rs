pub fn nand(a: bool, b: bool) -> bool {
    !(a && b)
}

pub fn not(a: bool) -> bool {
    nand(a, a)
}

pub fn and(a: bool, b: bool) -> bool {
    not(nand(a, b))
}

pub fn or(a: bool, b: bool) -> bool {
    nand(not(a), not(b))
}

pub fn xor(a: bool, b: bool) -> bool {
    and(nand(a, b), or(a, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nand_gate() {
        assert!(!nand(true, true));
        assert!(nand(true, false));
        assert!(nand(false, true));
        assert!(nand(false, false));
    }

    #[test]
    fn not_gate() {
        assert!(!not(true));
        assert!(not(false));
    }

    #[test]
    fn and_gate() {
        assert!(and(true, true));
        assert!(!and(true, false));
        assert!(!and(false, true));
        assert!(!and(false, false));
    }

    #[test]
    fn or_gate() {
        assert!(or(true, true));
        assert!(or(true, false));
        assert!(or(false, true));
        assert!(!or(false, false));
    }

    #[test]
    fn xor_gate() {
        assert!(!xor(true, true));
        assert!(xor(true, false));
        assert!(xor(false, true));
        assert!(!xor(false, false));
    }
}
