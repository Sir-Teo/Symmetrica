# special

Special functions for Symmetrica with symbolic and numeric support.

- Gamma function `Gamma(z)`
  - Symbolic constructor and derivative rule (uses digamma symbolically).
  - Numeric evaluation via Lanczos approximation with reflection for `z < 0.5`.
- Exponential integral `Ei(z)`
  - Symbolic constructor and derivative rule `d/dz Ei(z) = e^z / z`.
  - Numeric evaluation via convergent series for moderate |z|.
- Error function `erf(z)` (symbolic + basic numeric evaluation).

## Examples

```rust
use expr_core::Store;
use simplify::simplify;

let mut st = Store::new();
let x = st.sym("x");

// Gamma symbolic
let gx = st.func("Gamma", vec![x]);
assert!(st.to_string(gx).contains("Gamma"));

// Ei numeric (through direct API)
let val = special::expint::EiFunction.eval(&[1.0]).unwrap();
assert!((val - 1.895_117_816_355_936_8_f64).abs() < 1e-12);
```

## Notes

- For high-accuracy or large-|z| Ei, future work will add continued fractions and asymptotic expansions with switching.
- `Gamma` uses double-precision Lanczos coefficients suitable for robust f64 results.

## License

Dual-licensed under MIT or Apache-2.0.
