use super::nand::{and, or, xor};

pub fn add_bit(a: bool, b: bool, carry_in: bool) -> (bool, bool) {
    let sum = xor(xor(a, b), carry_in);
    let carry_out = or(and(a, b), and(xor(a, b), carry_in));
    (sum, carry_out)
}

pub fn add_u8(a: u8, b: u8) -> u8 {
    let mut result = 0u8;
    let mut carry = false;

    for i in 0..8 {
        let bit_a = (a >> i) & 1 == 1;
        let bit_b = (b >> i) & 1 == 1;
        let (sum, new_carry) = add_bit(bit_a, bit_b, carry);
        if sum {
            result |= 1 << i;
        }
        carry = new_carry;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_bit_no_carry() {
        let (sum, carry) = add_bit(false, false, false);
        assert!(!sum);
        assert!(!carry);
    }

    #[test]
    fn add_bit_with_carry() {
        let (sum, carry) = add_bit(true, true, false);
        assert!(!sum);
        assert!(carry);
    }

    #[test]
    fn add_u8_basic() {
        assert_eq!(add_u8(0, 0), 0);
        assert_eq!(add_u8(1, 0), 1);
        assert_eq!(add_u8(1, 1), 2);
        assert_eq!(add_u8(255, 1), 0);
        assert_eq!(add_u8(42, 58), 100);
    }
}
