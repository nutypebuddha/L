//! Continuous-truth-value NAND gates (Sheffer stroke extension to f64).
//!
//! For Boolean inputs {0, 1}, these are exact. For continuous inputs in
//! [0, 1], the output is `1 - a*b`, which smoothly interpolates — but
//! derived gates (not, and, or, etc.) are **non-linear** off the endpoints
//! and should not be treated as fuzzy logic.

/// The Sheffer stroke — functionally complete Boolean primitive.
#[inline]
pub fn nand(a: f64, b: f64) -> f64 {
    1.0 - a * b
}

/// NOT: `not(a) = nand(a, a) = 1 - a*a`
#[inline]
pub fn not(a: f64) -> f64 {
    nand(a, a)
}

/// AND: `and(a, b) = not(nand(a, b))`
#[inline]
pub fn and(a: f64, b: f64) -> f64 {
    not(nand(a, b))
}

/// OR: `or(a, b) = nand(not(a), not(b))`
#[inline]
pub fn or(a: f64, b: f64) -> f64 {
    nand(not(a), not(b))
}

/// NOR: `nor(a, b) = not(or(a, b))`
#[inline]
pub fn nor(a: f64, b: f64) -> f64 {
    not(or(a, b))
}

/// XOR: `xor(a, b) = or(and(a, not(b)), and(not(a), b))`
#[inline]
pub fn xor(a: f64, b: f64) -> f64 {
    or(and(a, not(b)), and(not(a), b))
}

/// XNOR: `xnor(a, b) = not(xor(a, b))`
#[inline]
pub fn xnor(a: f64, b: f64) -> f64 {
    not(xor(a, b))
}

/// Implies: `implies(a, b) = or(not(a), b)`
#[inline]
pub fn implies(a: f64, b: f64) -> f64 {
    or(not(a), b)
}

/// Half adder: returns `(sum, carry)` for two 1-bit f64 inputs.
#[inline]
pub fn half_adder(a: f64, b: f64) -> (f64, f64) {
    (xor(a, b), and(a, b))
}

/// Full adder: returns `(sum, carry_out)` for two 1-bit f64 inputs and carry-in.
#[inline]
pub fn full_adder(a: f64, b: f64, carry_in: f64) -> (f64, f64) {
    let (s1, c1) = half_adder(a, b);
    let (sum, c2) = half_adder(s1, carry_in);
    (sum, or(c1, c2))
}

/// 4-bit ripple-carry adder: `a + b` (each a `[bit3, bit2, bit1, bit0]`).
/// Returns `(result_bits, overflow)` where `result_bits` is little-endian.
pub fn add4(a: [f64; 4], b: [f64; 4]) -> ([f64; 4], f64) {
    let (s0, c0) = half_adder(a[0], b[0]);
    let (s1, c1) = full_adder(a[1], b[1], c0);
    let (s2, c2) = full_adder(a[2], b[2], c1);
    let (s3, c3) = full_adder(a[3], b[3], c2);
    ([s0, s1, s2, s3], c3)
}

/// Decode a little-endian `[f64; 4]` bit array to a `u8`.
pub fn bits_to_u8(bits: [f64; 4]) -> u8 {
    let mut v = 0u8;
    for (i, bit) in bits.iter().enumerate() {
        if *bit > 0.5 {
            v |= 1 << i;
        }
    }
    v
}

/// Encode a `u8` (masked to 4 bits) into a little-endian `[f64; 4]` bit array.
pub fn u8_to_bits(n: u8) -> [f64; 4] {
    [
        (n & 1) as f64,
        ((n >> 1) & 1) as f64,
        ((n >> 2) & 1) as f64,
        ((n >> 3) & 1) as f64,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nand_truth_table() {
        assert!((nand(0.0, 0.0) - 1.0).abs() < 1e-12);
        assert!((nand(0.0, 1.0) - 1.0).abs() < 1e-12);
        assert!((nand(1.0, 0.0) - 1.0).abs() < 1e-12);
        assert!((nand(1.0, 1.0) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn not_involution() {
        assert!((not(not(0.0)) - 0.0).abs() < 1e-12);
        assert!((not(not(1.0)) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn and_idempotent() {
        assert!((and(0.0, 0.0) - 0.0).abs() < 1e-12);
        assert!((and(1.0, 1.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn or_idempotent() {
        assert!((or(0.0, 0.0) - 0.0).abs() < 1e-12);
        assert!((or(1.0, 1.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn xor_commutative() {
        assert!((xor(0.3, 0.7) - xor(0.7, 0.3)).abs() < 1e-12);
    }

    #[test]
    fn de_morgan_boolean() {
        for a in [0.0, 1.0] {
            for b in [0.0, 1.0] {
                assert!(
                    (not(and(a, b)) - or(not(a), not(b))).abs() < 1e-12,
                    "De Morgan failed for a={a}, b={b}"
                );
                assert!(
                    (not(or(a, b)) - and(not(a), not(b))).abs() < 1e-12,
                    "De Morgan 2 failed for a={a}, b={b}"
                );
            }
        }
    }

    #[test]
    fn implies_definition() {
        for a in [0.0, 0.5, 1.0] {
            for b in [0.0, 0.5, 1.0] {
                assert!(
                    (implies(a, b) - or(not(a), b)).abs() < 1e-12,
                    "implies({a}, {b}) mismatch"
                );
            }
        }
    }

    #[test]
    fn all_gates_from_nand_only() {
        let vals: [f64; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];
        for a in vals {
            for b in vals {
                assert!((not(a) - nand(a, a)).abs() < 1e-12);
                assert!((and(a, b) - not(nand(a, b))).abs() < 1e-12);
                assert!((or(a, b) - nand(not(a), not(b))).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn w10_nand_descent_worked_example() {
        let a = u8_to_bits(2);
        let b = u8_to_bits(3);
        let (sum_bits, overflow) = add4(a, b);
        let result = bits_to_u8(sum_bits);
        assert_eq!(result, 5, "2 + 3 must descend to 5 at the NAND floor");
        assert_eq!(overflow, 0.0, "4-bit sum of 2 + 3 has no overflow");
    }
}
