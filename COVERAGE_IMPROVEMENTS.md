# Test Coverage Improvements

**Latest Coverage:** 89.82% (2267/2524 lines) - October 2024

## Summary
- **Previous Coverage**: 75.99% (1263/1662 lines)
- **New Coverage**: 81.91% (1354/1653 lines)
- **Improvement**: +5.92% (+91 lines covered)

## Tests Added

### calculus/integrate.rs (10 new tests)
- `integrate_constant_symbol` - integrating constant symbols
- `integrate_add_rule` - sum rule
- `integrate_mul_constant_factor` - constant factorization
- `integrate_rational_constant` - rational constants
- `integrate_fails_on_unsupported` - unsupported integrals
- `integrate_integer_const` - integer constants
- `integrate_power_negative_exponent` - negative powers
- `integrate_exp_constant_derivative` - exp(x) integration
- `integrate_rational_via_pf_fails_on_complex` - complex roots

### calculus/diff.rs (7 new tests)
- `diff_constant` - constant differentiation
- `diff_rational` - rational differentiation
- `diff_other_symbol` - symbol independence
- `diff_pow_zero_exp` - x^0 derivative
- `diff_log_alias` - log function alias
- `diff_unknown_function` - unsupported functions
- `diff_multiarg_function` - multi-argument functions

### calculus/series.rs (11 new tests)
- `series_basic_ops` - addition and multiplication
- `series_sub` - subtraction
- `series_scale` - scaling operations
- `series_truncate` - truncation
- `series_compose_requires_zero_const` - composition constraints
- `maclaurin_mul` - Maclaurin for products
- `maclaurin_pow` - Maclaurin for powers
- `maclaurin_negative_exponent_fails` - negative exponents
- `maclaurin_log_requires_one_at_zero` - log constraints
- `limit_poly_constant` - constant limits
- `limit_poly_rational_coeff` - rational coefficient limits
- `limit_poly_unsupported` - unsupported limits

### polys/lib.rs (13 new tests)
- `unipoly_zero_and_degree` - zero polynomial
- `unipoly_deriv` - derivative
- `unipoly_eval` - evaluation
- `unipoly_add_different_lengths` - addition edge cases
- `unipoly_sub` - subtraction
- `unipoly_mul_with_zero` - multiplication with zero
- `unipoly_div_rem_by_zero` - division by zero error
- `expr_to_unipoly_rational_coeff` - rational coefficients
- `expr_to_unipoly_pow_negative_fails` - negative powers
- `expr_to_unipoly_wrong_var` - wrong variable
- `expr_to_unipoly_function_fails` - non-polynomial expressions
- `unipoly_to_expr_zero` - zero conversion
- `unipoly_monic` - monic polynomials
- `unipoly_monic_zero` - monic zero

### simplify/lib.rs (4 new tests)
- `ln_power_rule_with_positivity` - logarithm power rule
- `ln_product_rule_with_positivity` - logarithm product rule
- `simplify_pow_rational_non_matching` - power simplification constraints
- `simplify_unknown_function` - unknown function handling

### pattern/lib.rs (3 new tests)
- `subst_in_function` - substitution in functions
- `subst_integer_unchanged` - integer substitution
- `subst_rational_unchanged` - rational substitution

### matrix/lib.rs (5 new tests)
- `det_non_square_error` - non-square determinant error
- `solve_non_square_error` - non-square solve error
- `solve_wrong_rhs_length` - RHS length mismatch
- `det_zero_size` - zero-size determinant
- `solve_zero_size` - zero-size solve

### solver/lib.rs (2 new tests)
- `solve_not_polynomial` - non-polynomial expressions
- `solve_quadratic_with_rational_discriminant` - rational roots

### plot/lib.rs (9 new tests)
- `eval_add` - addition evaluation
- `eval_mul` - multiplication evaluation
- `eval_cosx` - cosine evaluation
- `eval_expx` - exponential evaluation
- `eval_unknown_func` - unknown function
- `eval_multiarg_func` - multi-argument functions
- `eval_unbound_symbol` - unbound symbols
- `plot_single_sample` - single sample plotting

### arith/lib.rs (5 new tests)
- `gcd_zero_cases` - GCD with zeros
- `gcd_negative` - GCD with negatives
- `normalize_negative_den` - negative denominator normalization
- `q_operations` - rational operations
- `q_struct_methods` - Q struct methods

### expr_core/lib.rs (5 new tests)
- `test_add_empty` - empty addition
- `test_add_single` - single-element addition
- `test_mul_empty` - empty multiplication
- `test_mul_single` - single-element multiplication
- `test_pow_base_zero_exp_zero` - 0^0 handling
- `test_printer_rational` - rational printing

## Test Design Principles
- **Minimal**: Each test focuses on a single code path
- **Scalable**: Tests are isolated and can be extended
- **Coverage-driven**: Targeted uncovered branches and error paths
- **Edge cases**: Zero values, empty inputs, error conditions

## Quality Checks Passed
✅ `cargo fmt --all -- --check`
✅ `cargo clippy --workspace --all-targets -- -D warnings`
✅ `cargo test --workspace` (165 tests passing)
✅ `cargo doc --workspace --no-deps`
