# Migration Guide

This document provides guidance for migrating between major versions of Symmetrica.

## Table of Contents

- [Migrating to 1.0.0](#migrating-to-100)
- [Breaking Changes](#breaking-changes)
- [Deprecations](#deprecations)
- [New Features](#new-features)

## Migrating to 1.0.0

Version 1.0.0 represents the first stable release of Symmetrica. If you've been using pre-1.0 versions like 0.1.x (development versions), here are the key changes from 0.1 to 1.0:

### What's New in 1.0.0

Symmetrica 1.0.0 introduces a complete symbolic mathematics system with:

- **Stable Core API**: Hash consing, canonical forms, and deterministic output
- **Calculus**: Symbolic differentiation and integration
- **Linear Algebra**: Matrix operations, determinants, and solving
- **Polynomial Operations**: GCD, factorization, and rational functions
- **Pattern Matching**: Powerful rewrite system for symbolic transformations
- **Special Functions**: Gamma, error functions, and Bessel functions (partial)
- **Number Theory**: GCD, primitive roots, Diophantine equations
- **Tensor Operations**: Einstein notation support
- **ODE Solvers**: First and second-order differential equations
- **Multiple Output Formats**: LaTeX, JSON, and S-expressions

### Core API Stabilization

The core API has been stabilized with the following guarantees:
- Hash consing and canonical representation
- Deterministic output for identical inputs
- No panics on valid input (returns Option/Result instead)

### API Changes

The following API changes were made in the transition to 1.0.0:

- **Store Construction**: Changed from `Store::default()` to `Store::new()`
- **Error Handling**: Many functions now return `Option<T>` or `Result<T, E>` instead of panicking
- **Function Signatures**: All public APIs have been reviewed and stabilized
- **Module Organization**: Reorganized into logical crates for better modularity

### Breaking Changes

#### 1. Store Construction

**Before (pre-1.0):**
```rust
let store = Store::default();
```

**After (1.0+):**
```rust
let mut store = Store::new();
```

#### 2. Error Handling

Many functions that previously panicked now return `Option` or `Result`:

**Before:**
```rust
let result = integrate(&mut store, expr, "x"); // Could panic
```

**After:**
```rust
let result = integrate(&mut store, expr, "x"); // Returns Option<ExprId>
if let Some(integral) = result {
    // Handle success
} else {
    // Handle failure to integrate
}
```

#### 3. Gröbner Basis API

The Gröbner basis solver has been improved with better performance:

**Before:**
```rust
// May have timed out on complex systems
let solution = solve_system(&mut store, equations, vars);
```

**After:**
```rust
// Improved performance with simplification
// Note: Some complex systems may still be slow and are marked as ignored in tests
let solution = solve_system(&mut store, equations, vars);
```

### Deprecations

No features have been deprecated in 1.0.0, but some experimental features are marked as such:
- Gröbner basis solving for complex multi-variable systems (performance limitations)
- Some special functions (Bessel functions) are still under development

### New Features in 1.0.0

- **Symbolic Tensors**: Full Einstein notation support
- **Special Functions**: Gamma, Error function, Bessel functions (partial)
- **Number Theory**: GCD, primitive roots, Diophantine equations
- **Radical Denesting**: Automatic simplification of nested radicals
- **ODE Solvers**: Exact, Bernoulli, and second-order equation solvers

## Migration Checklist

When migrating to 1.0.0:

- [ ] Update `Cargo.toml` to version `1.0.0` or higher
- [ ] Replace `Store::default()` with `Store::new()`
- [ ] Add error handling for functions returning `Option` or `Result`
- [ ] Review use of experimental features (check documentation)
- [ ] Run your test suite to identify any breaking changes
- [ ] Update any custom pattern matching code (API is now stable)

## Troubleshooting

### Common Migration Issues

**Issue**: Code panics with "Store is immutable"
- **Solution**: Ensure you're creating the Store with `Store::new()` and declaring it as `mut`

**Issue**: Function returns `None` unexpectedly
- **Solution**: Check the input constraints - many functions have domain restrictions (e.g., integration may not always succeed)

**Issue**: Performance degradation with Gröbner basis
- **Solution**: The current Gröbner implementation has performance limitations for complex systems. Consider using alternative solving methods or simplifying your system.

**Issue**: Bessel functions not evaluating
- **Solution**: Only integer-order Bessel functions (J and I) are currently supported for numerical evaluation. Symbolic forms work for all types.

### Debugging Tips

1. Enable verbose output to see what's happening:
   ```rust
   println!("Expression: {}", store.to_string(expr));
   ```

2. Check assumptions if simplification isn't working:
   ```rust
   use assumptions::assume;
   assume(&mut store, "x", "real");
   ```

3. For integration issues, try simplifying first:
   ```rust
   let simplified = simplify(&mut store, expr);
   let integral = integrate(&mut store, simplified, "x");
   ```

## Getting Help

If you encounter issues during migration:

1. Check the [API documentation](https://docs.rs/symmetrica)
2. Review the [CHANGELOG.md](CHANGELOG.md) for detailed changes
3. Search existing issues on [GitHub](https://github.com/Sir-Teo/Symmetrica/issues)
4. Open a new issue with a minimal reproducible example

## Future Migrations

This document will be updated with each major release to help you migrate between versions. We are committed to maintaining backward compatibility within major versions (e.g., all 1.x versions will be compatible).
