# Fuzzing Symmetrica (Phase L)

This directory contains fuzz targets for testing Symmetrica components.

## Setup

Install cargo-fuzz:
```bash
cargo install cargo-fuzz
```

## Running Fuzz Tests

From the repository root:

**Fuzz simplifier:**
```bash
cargo fuzz run fuzz_simplify -- -max_total_time=60
```

**Fuzz expression operations:**
```bash
cargo fuzz run fuzz_expr_ops -- -max_total_time=60
```

**Fuzz S-expression parser:**
```bash
cargo fuzz run fuzz_sexpr_parse -- -max_total_time=60
```

**Fuzz differentiation:**
```bash
cargo fuzz run fuzz_diff -- -max_total_time=60
```

## Continuous Fuzzing

For longer runs (recommended for CI):
```bash
cargo fuzz run fuzz_simplify -- -max_total_time=300
```

## Corpus Management

Fuzz inputs that find bugs or interesting cases are saved in:
```
fuzz/corpus/<target_name>/
```

Crashes are saved in:
```
fuzz/artifacts/<target_name>/
```

## Properties Tested

### fuzz_simplify
- Simplification doesn't crash
- Simplify is idempotent: `simplify(simplify(e)) == simplify(e)`
- String conversion works on simplified expressions

### fuzz_expr_ops
- Basic operations (add, mul, pow, rat) don't crash
- Operations on extreme values are handled gracefully
- String conversion always succeeds

### fuzz_sexpr_parse
- Parser doesn't panic on arbitrary input
- Valid S-expressions round-trip correctly
- Parser errors are handled gracefully

### fuzz_diff
- Differentiation doesn't crash
- Repeated differentiation works
- All trig/exp functions can be differentiated

## Adding New Targets

1. Create `fuzz/fuzz_targets/fuzz_<name>.rs`
2. Add binary entry in `fuzz/Cargo.toml`
3. Implement with `#![no_main]` and `fuzz_target!` macro
4. Test properties, not specific outputs

## CI Integration

Fuzz tests can be run in CI with time limits:
```bash
for target in fuzz_simplify fuzz_expr_ops fuzz_sexpr_parse fuzz_diff; do
    cargo fuzz run $target -- -max_total_time=60 -seed=0
done
```

## Roadmap Alignment

This implements **Phase L: Hardening, Fuzzing, Differential Testing** deliverables:
- ✅ Fuzzing on parser/simplifier/rewriter
- 🔲 Differential tests against reference (future)
- 🔲 Metrics dashboards (future)
- 🔲 1.0 API finalization (future)
