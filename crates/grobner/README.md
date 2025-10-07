# grobner

Gröbner bases utilities for polynomial systems in Symmetrica.

- Monomial orders: `Lex`, `GrLex`, `GRevLex`.
- S-polynomial construction: `s_polynomial(f, g, vars, order)`.
- Naive multivariate reduction: `reduce(f, basis, vars, order)` returning structural remainder.
- Buchberger driver (simplified): `buchberger(store, polys, vars, order)`.

This is a foundational implementation intended for experimentation and testing.

## Example

```rust
use expr_core::Store;
use grobner::{MonomialOrder, s_polynomial, reduce};

let mut st = Store::new();
let x = st.sym("x");
let y = st.sym("y");
let two = st.int(2);

// f = x^2 + y
let f = st.add(vec![st.pow(x, two), y]);
// g = x*y + 1
let g = st.add(vec![st.mul(vec![x, y]), st.int(1)]);

let vars = vec!["x".to_string(), "y".to_string()];
let s = s_polynomial(&mut st, f, g, &vars, MonomialOrder::Lex).unwrap();
assert_eq!(st.get(s).op, expr_core::Op::Add);

// Reduce a simple polynomial by a monomial basis
let h = st.add(vec![st.pow(x, two), st.mul(vec![x, y]), st.int(1)]);
let r = reduce(&mut st, h, &[x, y], &vars, MonomialOrder::Lex);
// Expect a constant remainder (structural)
```

## Notes

- Coefficient arithmetic is minimal; `reduce()` performs structural cancellation and is suitable for basic testing.
- Future work: full multivariate division with coefficients and Buchberger criteria.

## License

Dual-licensed under MIT or Apache-2.0.
