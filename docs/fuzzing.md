# Fuzzing (Phase L)

Fuzzing infrastructure for Symmetrica to ensure robustness and catch edge cases.

## Overview

Fuzzing uses `cargo-fuzz` (libFuzzer) to generate random inputs and test for:
- Crashes and panics
- Undefined behavior
- Property violations
- Memory safety issues

## Installation

```bash
cargo install cargo-fuzz
```

Requires nightly Rust:
```bash
rustup install nightly
```

## Fuzz Targets

### 1. fuzz_simplify

Tests the simplifier for correctness and robustness.

**Properties tested:**
- Simplification completes without panic
- **Idempotence**: `simplify(simplify(e)) == simplify(e)`
- String conversion succeeds on all simplified expressions
- Digest equality after repeated simplification

**Run:**
```bash
cargo +nightly fuzz run fuzz_simplify
```

### 2. fuzz_expr_ops

Tests basic expression operations.

**Properties tested:**
- Add, mul, pow, rat operations don't crash
- Operations handle extreme values gracefully  
- String conversion always succeeds
- Rational normalization works correctly

**Run:**
```bash
cargo +nightly fuzz run fuzz_expr_ops
```

### 3. fuzz_sexpr_parse

Tests S-expression parser robustness.

**Properties tested:**
- Parser never panics on arbitrary input
- Valid S-expressions round-trip correctly
- Parsing errors are handled gracefully
- Round-trip preserves semantics

**Run:**
```bash
cargo +nightly fuzz run fuzz_sexpr_parse
```

### 4. fuzz_diff

Tests symbolic differentiation.

**Properties tested:**
- Differentiation completes without panic
- Repeated differentiation works (d²/dx²)
- All supported functions can be differentiated
- Simplified derivatives are valid

**Run:**
```bash
cargo +nightly fuzz run fuzz_diff
```

## Running Fuzz Tests

**Quick test (60 seconds):**
```bash
cargo +nightly fuzz run fuzz_simplify -- -max_total_time=60
```

**Extended fuzzing (5 minutes):**
```bash
cargo +nightly fuzz run fuzz_simplify -- -max_total_time=300
```

**With specific seed (reproducible):**
```bash
cargo +nightly fuzz run fuzz_simplify -- -max_total_time=60 -seed=12345
```

**All targets:**
```bash
for target in fuzz_simplify fuzz_expr_ops fuzz_sexpr_parse fuzz_diff; do
    cargo +nightly fuzz run $target -- -max_total_time=60 -seed=0
done
```

## Corpus and Artifacts

**Corpus** (interesting inputs):
- Stored in `fuzz/corpus/<target>/`
- Contains inputs that increase code coverage
- Can be committed to git for regression testing

**Artifacts** (crashes):
- Stored in `fuzz/artifacts/<target>/`
- Contains inputs that caused crashes/failures
- Should be fixed and turned into regression tests

## CI Integration

Fuzzing can run in CI with time limits:

```yaml
# .github/workflows/fuzz.yml
- name: Fuzz tests
  run: |
    rustup install nightly
    cargo install cargo-fuzz
    cargo +nightly fuzz run fuzz_simplify -- -max_total_time=120 -seed=0
```

## Minimizing Crash Inputs

If fuzzing finds a crash:

```bash
cargo +nightly fuzz cmin fuzz_simplify
cargo +nightly fuzz tmin fuzz_simplify artifacts/fuzz_simplify/crash-xxx
```

## Coverage

Generate coverage report:

```bash
cargo +nightly fuzz coverage fuzz_simplify
```

## Property-Based Testing

Fuzz targets focus on **properties**, not specific outputs:

✅ **Good**: "Simplify should be idempotent"  
❌ **Bad**: "x+x should simplify to 2*x"

This makes fuzzing more effective at finding unexpected bugs.

## Best Practices

1. **Start small**: Run for 60 seconds first
2. **Seed for reproducibility**: Use `-seed=0` in CI
3. **Save interesting corpus**: Commit good inputs
4. **Fix crashes**: Turn artifacts into regression tests
5. **Monitor coverage**: Aim for high code coverage

## Roadmap Alignment

Phase L deliverables:
- ✅ Fuzzing on parser/simplifier/differentiation
- ✅ Property-based testing
- ✅ Differential testing vs reference CAS
- ✅ **Fuzz CI integration** (.github/workflows/fuzz.yml)
- ✅ **Crash-free operation** (bugs fixed via fuzzing)
- 🔲 Coverage metrics dashboard (future)

## See Also

- [cargo-fuzz documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer](https://llvm.org/docs/LibFuzzer.html)
- [Fuzzing Rust](https://rust-fuzz.github.io/book/)
