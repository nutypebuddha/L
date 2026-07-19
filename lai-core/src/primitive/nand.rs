//! # NAND Gate — the single primitive operation
//!
//! All logic gates derive from the Sheffer stroke (NAND):
//!
//! ```text
//! nand(a, b) = 1 - a * b
//! ```
//!
//! Inputs are `f64` values interpreted as continuous truth values in [0, 1].

/// The Sheffer stroke — functionally complete Boolean primitive.
///
/// `nand(a, b) = 1 - a * b`
#[inline]
pub fn nand(a: f64, b: f64) -> f64 {
    1.0 - a * b
}

/// NOT: `not(a) = nand(a, a) = 1 - a`
#[inline]
pub fn not(a: f64) -> f64 {
    nand(a, a)
}

/// AND: `and(a, b) = not(nand(a, b)) = a * b`
#[inline]
pub fn and(a: f64, b: f64) -> f64 {
    not(nand(a, b))
}

/// OR: `or(a, b) = nand(not(a), not(b)) = a + b - a*b`
#[inline]
pub fn or(a: f64, b: f64) -> f64 {
    nand(not(a), not(b))
}

/// NOR: `nor(a, b) = not(or(a, b)) = 1 - (a + b - a*b)`
#[inline]
pub fn nor(a: f64, b: f64) -> f64 {
    not(or(a, b))
}

/// XOR: `xor(a, b) = or(and(a, not(b)), and(not(a), b)) = a + b - 2*a*b`
#[inline]
pub fn xor(a: f64, b: f64) -> f64 {
    or(and(a, not(b)), and(not(a), b))
}

/// XNOR: `xnor(a, b) = not(xor(a, b)) = 1 - a - b + 2*a*b`
#[inline]
pub fn xnor(a: f64, b: f64) -> f64 {
    not(xor(a, b))
}

/// Implies: `implies(a, b) = or(not(a), b) = 1 - a + a*b`
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
    fn test_nand_identity() {
        assert!((nand(0.0, 1.0) - 1.0).abs() < 1e-12);
        assert!((nand(1.0, 1.0) - 0.0).abs() < 1e-12);
        assert!((nand(0.0, 0.0) - 1.0).abs() < 1e-12);
        assert!((nand(1.0, 0.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_not_involution() {
        assert!((not(not(0.0)) - 0.0).abs() < 1e-12);
        assert!((not(not(1.0)) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_and_idempotent() {
        assert!((and(0.0, 0.0) - 0.0).abs() < 1e-12);
        assert!((and(1.0, 1.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_or_idempotent() {
        assert!((or(0.0, 0.0) - 0.0).abs() < 1e-12);
        assert!((or(1.0, 1.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_xor_commutative() {
        assert!((xor(0.3, 0.7) - xor(0.7, 0.3)).abs() < 1e-12);
    }

    #[test]
    fn test_de_morgan_boolean() {
        for a in [0.0, 1.0] {
            for b in [0.0, 1.0] {
                let lhs = not(and(a, b));
                let rhs = or(not(a), not(b));
                assert!((lhs - rhs).abs() < 1e-12);

                let lhs2 = not(or(a, b));
                let rhs2 = and(not(a), not(b));
                assert!((lhs2 - rhs2).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn test_implies_definition() {
        for a in [0.0, 0.5, 1.0] {
            for b in [0.0, 0.5, 1.0] {
                let direct = implies(a, b);
                let derived = or(not(a), b);
                assert!((direct - derived).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn test_all_gates_from_nand_only() {
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
    fn test_half_adder() {
        let (s, c) = half_adder(0.0, 0.0);
        assert!((s - 0.0).abs() < 1e-12);
        assert!((c - 0.0).abs() < 1e-12);

        let (s, c) = half_adder(1.0, 0.0);
        assert!((s - 1.0).abs() < 1e-12);
        assert!((c - 0.0).abs() < 1e-12);

        let (s, c) = half_adder(1.0, 1.0);
        assert!((s - 0.0).abs() < 1e-12);
        assert!((c - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_full_adder() {
        let (s, c) = full_adder(1.0, 1.0, 0.0);
        assert!((s - 0.0).abs() < 1e-12);
        assert!((c - 1.0).abs() < 1e-12);

        let (s, c) = full_adder(1.0, 1.0, 1.0);
        assert!((s - 1.0).abs() < 1e-12);
        assert!((c - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_add4_and_bits_roundtrip() {
        let a = u8_to_bits(2);
        let b = u8_to_bits(3);
        let (sum_bits, overflow) = add4(a, b);
        let result = bits_to_u8(sum_bits);
        assert_eq!(result, 5, "2 + 3 must equal 5");
        assert_eq!(overflow, 0.0, "no overflow for 2 + 3");
    }

    #[test]
    fn test_bits_roundtrip() {
        for n in 0u8..16 {
            let bits = u8_to_bits(n);
            assert_eq!(bits_to_u8(bits), n);
        }
    }
}
