//! Property-based tests for polys

use arith::{add_q, mul_q, Q};
use expr_core::Store;
use polys::{expr_to_unipoly, unipoly_to_expr, UniPoly};
use proptest::prelude::*;

fn small_q() -> impl Strategy<Value = Q> {
    // Use smaller range to avoid overflow in polynomial GCD operations
    (-2i64..=2, 1i64..=2).prop_map(|(n, d)| Q::new(n, d))
}

fn small_poly() -> impl Strategy<Value = UniPoly> {
    // Degree up to 3 (len 0..=4)
    prop::collection::vec(small_q(), 0..=4).prop_map(|coeffs| UniPoly::new("x", coeffs))
}

fn nonzero_poly() -> impl Strategy<Value = UniPoly> {
    small_poly().prop_filter("non-zero polynomial", |p| !p.is_zero() && p.degree().is_some())
}

proptest! {
    #[test]
    fn prop_add_eval_consistency(p in small_poly(), q in small_poly(), x in small_q()) {
        let lhs = p.add(&q).eval_q(x);
        let rhs = add_q(p.eval_q(x), q.eval_q(x));
        prop_assert_eq!(lhs, rhs);
    }

    #[test]
    fn prop_mul_eval_consistency(p in small_poly(), q in small_poly(), x in small_q()) {
        let lhs = p.mul(&q).eval_q(x);
        let rhs = mul_q(p.eval_q(x), q.eval_q(x));
        prop_assert_eq!(lhs, rhs);
    }

    #[test]
    fn prop_div_rem_identity(a in small_poly(), b in nonzero_poly()) {
        let (q, r) = a.div_rem(&b).expect("div");
        let recomposed = q.mul(&b).add(&r);
        prop_assert_eq!(recomposed, a);
    }

    #[test]
    fn prop_gcd_divides(a in small_poly(), b in small_poly()) {
        let g = UniPoly::gcd(a.clone(), b.clone());
        // Skip if gcd is zero (both inputs were zero)
        if g.is_zero() {
            return Ok(());
        }
        // a % g == 0 and b % g == 0
        let r1 = a.div_rem(&g).expect("div").1;
        let r2 = b.div_rem(&g).expect("div").1;
        prop_assert!(r1.is_zero());
        prop_assert!(r2.is_zero());
    }

    #[test]
    fn prop_expr_roundtrip(p in small_poly()) {
        let mut st = Store::new();
        let e = unipoly_to_expr(&mut st, &p);
        let back = expr_to_unipoly(&st, e, "x").expect("convertible");
        prop_assert_eq!(back, p);
    }

    #[test]
    fn prop_discriminant_repeated_root_zero(a in -3i64..=3) {
        // (x - a)^2 = x^2 - 2a x + a^2 => discriminant == 0
        let p = UniPoly::new(
            "x",
            vec![Q::new(a * a, 1), Q::new(-2 * a, 1), Q::new(1, 1)],
        );
        let disc = p.discriminant().expect("degree >= 1");
        prop_assert!(disc.is_zero());
    }

    #[test]
    fn prop_divides_after_gcd_reduction(a in nonzero_poly(), b in nonzero_poly()) {
        // After dividing out gcd, the reduced polynomials should be coprime
        let g = UniPoly::gcd(a.clone(), b.clone());
        let (a_red, _) = a.div_rem(&g).expect("div");
        let (b_red, _) = b.div_rem(&g).expect("div");
        let g2 = UniPoly::gcd(a_red, b_red);
        // gcd should be constant (degree 0) after reduction
        prop_assert!(matches!(g2.degree(), Some(0)) || g2.is_zero());
    }
}
