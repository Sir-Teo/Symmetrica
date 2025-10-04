# cli - Command-Line Interface

## Overview

The `cli` crate (`matika_cli`) provides a demonstration command-line interface for Symmetrica's core functionality. It supports parsing, simplification, differentiation, integration, equation solving, and plotting.

## Installation

Build from source:
```bash
cargo build --release -p cli
# Binary: target/release/matika_cli
```

Or run directly:
```bash
cargo run -p cli -- <command> <args>
```

## Commands

### parse

Parse an expression and display it in multiple formats.

**Usage:**
```bash
matika_cli parse (--sexpr <EXPR> | --json <EXPR>)
```

**Output:**
- **text**: Human-readable infix notation
- **latex**: LaTeX format for mathematical typesetting
- **json**: JSON serialization
- **sexpr**: S-expression format

**Examples:**
```bash
# Parse from S-expression
matika_cli parse --sexpr "(^ (Sym x) (Int 2))"

# Parse from JSON
matika_cli parse --json '{"Pow": {"base": {"Symbol": "x"}, "exp": {"Integer": 2}}}'
```

**Sample output:**
```
text:   x^2
latex:  x^{2}
json:   {"Pow":{"base":{"Symbol":"x"},"exp":{"Integer":2}}}
sexpr:  (^ (Sym x) (Int 2))
```

### simplify

Simplify an expression.

**Usage:**
```bash
matika_cli simplify (--sexpr <EXPR> | --json <EXPR>)
```

**Examples:**
```bash
# Collect like terms: 2x + 3x → 5x
matika_cli simplify --sexpr "(+ (* (Int 2) (Sym x)) (* (Int 3) (Sym x)))"
# Output: 5 * x

# Merge powers: x^2 * x^3 → x^5
matika_cli simplify --sexpr "(* (^ (Sym x) (Int 2)) (^ (Sym x) (Int 3)))"
# Output: x^5

# Simplify ln(exp(x)) → x
matika_cli simplify --sexpr "(Func ln (Func exp (Sym x)))"
# Output: x
```

### diff

Compute the derivative with respect to a variable.

**Usage:**
```bash
matika_cli diff (--sexpr <EXPR> | --json <EXPR>) --var <VARIABLE>
```

**Examples:**
```bash
# d/dx (x^3) = 3x^2
matika_cli diff --sexpr "(^ (Sym x) (Int 3))" --var x
# Output: 3 * x^2

# d/dx (sin(x^2)) = 2x * cos(x^2)
matika_cli diff --sexpr "(Func sin (^ (Sym x) (Int 2)))" --var x
# Output: cos(x^2) * 2 * x

# Product rule: d/dx (x * (x+1))
matika_cli diff --sexpr "(* (Sym x) (+ (Sym x) (Int 1)))" --var x
# Output: 1 + 2 * x
```

### integrate

Integrate an expression with respect to a variable.

**Usage:**
```bash
matika_cli integrate (--sexpr <EXPR> | --json <EXPR>) --var <VARIABLE>
```

**Exit codes:**
- `0`: Success
- `3`: Not integrable (no pattern matched)

**Examples:**
```bash
# ∫ x^2 dx = (1/3)x^3
matika_cli integrate --sexpr "(^ (Sym x) (Int 2))" --var x
# Output: 1/3 * x^3

# ∫ 1/x dx = ln(x)
matika_cli integrate --sexpr "(^ (Sym x) (Int -1))" --var x
# Output: ln(x)

# ∫ sin(2x) dx = -(1/2)cos(2x)
matika_cli integrate --sexpr "(Func sin (* (Int 2) (Sym x)))" --var x
# Output: -1/2 * cos(2 * x)

# Not integrable
matika_cli integrate --sexpr "(Func sin (^ (Sym x) (Int 2)))" --var x
# Stderr: not integrable
# Exit: 3
```

### solve

Solve a univariate polynomial equation.

**Usage:**
```bash
matika_cli solve (--sexpr <EXPR> | --json <EXPR>) --var <VARIABLE>
```

Solves `<EXPR> = 0` for `<VARIABLE>`.

**Exit codes:**
- `0`: Success (may have 0 or more roots)
- `4`: Cannot solve completely (not polynomial or no rational roots)

**Examples:**
```bash
# Solve x + 1 = 0
matika_cli solve --sexpr "(+ (Sym x) (Int 1))" --var x
# Output: -1

# Solve x^2 + 3x + 2 = 0
matika_cli solve --sexpr "(+ (^ (Sym x) (Int 2)) (* (Int 3) (Sym x)) (Int 2))" --var x
# Output:
# -1
# -2

# Solve x^2 - 2 = 0 (irrational roots)
matika_cli solve --sexpr "(+ (^ (Sym x) (Int 2)) (Int -2))" --var x
# Output:
# (2)^(1/2)
# -1 * (2)^(1/2)

# Cannot solve (no rational roots)
matika_cli solve --sexpr "(+ (^ (Sym x) (Int 3)) (Sym x) (Int 1))" --var x
# Stderr: cannot solve completely
# Exit: 4
```

### plot

Generate an SVG plot of an expression.

**Usage:**
```bash
matika_cli plot (--sexpr <EXPR> | --json <EXPR>) --var <VARIABLE> \
  [--xmin <MIN>] [--xmax <MAX>] [--samples <N>] \
  [--width <W>] [--height <H>]
```

**Options:**
- `--xmin`: Left boundary (default: -1.0)
- `--xmax`: Right boundary (default: 1.0)
- `--samples`: Number of sample points (default: 100)
- `--width`: SVG width in pixels (default: 640)
- `--height`: SVG height in pixels (default: 480)

**Output:** SVG document to stdout

**Examples:**
```bash
# Plot x^2 from -2 to 2
matika_cli plot --sexpr "(^ (Sym x) (Int 2))" --var x \
  --xmin -2 --xmax 2 --samples 100 --width 640 --height 480 > parabola.svg

# Plot sin(x)
matika_cli plot --sexpr "(Func sin (Sym x))" --var x \
  --xmin 0 --xmax 6.28 --samples 200 > sine.svg

# Plot 1/x
matika_cli plot --sexpr "(^ (Sym x) (Int -1))" --var x \
  --xmin -5 --xmax 5 --samples 300 > reciprocal.svg
```

## Input Formats

### S-Expression Syntax

**Atoms:**
```sexpr
(Int 5)         ; Integer
(Rat 1 2)       ; Rational 1/2
(Sym x)         ; Symbol
```

**Operations:**
```sexpr
(+ e1 e2 ...)   ; Addition
(* e1 e2 ...)   ; Multiplication
(^ base exp)    ; Power
(Func name e1 e2 ...)  ; Function
```

**Examples:**
```sexpr
; x^2 + 2x + 1
(+ (^ (Sym x) (Int 2)) 
   (* (Int 2) (Sym x)) 
   (Int 1))

; sin(x^2)
(Func sin (^ (Sym x) (Int 2)))

; (1/2) * x
(* (Rat 1 2) (Sym x))
```

### JSON Syntax

**Atoms:**
```json
{"Integer": 5}
{"Rational": {"num": 1, "den": 2}}
{"Symbol": "x"}
```

**Operations:**
```json
{"Add": {"terms": [e1, e2, ...]}}
{"Mul": {"factors": [e1, e2, ...]}}
{"Pow": {"base": e1, "exp": e2}}
{"Function": {"name": "sin", "args": [e]}}
```

**Examples:**
```json
// x^2 + 1
{
  "Add": {
    "terms": [
      {"Pow": {"base": {"Symbol": "x"}, "exp": {"Integer": 2}}},
      {"Integer": 1}
    ]
  }
}

// sin(x)
{"Function": {"name": "sin", "args": [{"Symbol": "x"}]}}
```

## Error Handling

### Parse Errors

Invalid input produces error message and exits with code 2:
```bash
matika_cli parse --sexpr "(+ (Int 1"
# Stderr: parse error: unclosed parenthesis
# Exit: 2
```

### Integration Failure

Returns error message and code 3:
```bash
matika_cli integrate --sexpr "(Func sin (Sym x))" --var y
# Stderr: not integrable
# Exit: 3
```

### Solver Failure

Returns error message and code 4:
```bash
matika_cli solve --sexpr "(Func sin (Sym x))" --var x
# Stderr: cannot solve completely
# Exit: 4
```

## Examples Workflows

### Differentiate and Simplify

```bash
# Define f(x) = x^3 + 2x^2 + x
EXPR="(+ (^ (Sym x) (Int 3)) (* (Int 2) (^ (Sym x) (Int 2))) (Sym x))"

# Compute derivative
DERIV=$(matika_cli diff --sexpr "$EXPR" --var x)
echo "f'(x) = $DERIV"
# Output: f'(x) = 1 + 4 * x + 3 * x^2
```

### Solve and Verify

```bash
# Solve x^2 - 5x + 6 = 0
ROOTS=$(matika_cli solve --sexpr "(+ (^ (Sym x) (Int 2)) (* (Int -5) (Sym x)) (Int 6))" --var x)
echo "Roots: $ROOTS"
# Output: Roots: 2
#                3
```

### Plot Derivative

```bash
# Define f(x) = x^2
EXPR="(^ (Sym x) (Int 2))"

# Differentiate
DERIV_SEXPR=$(matika_cli diff --sexpr "$EXPR" --var x | /* convert back */)

# Plot derivative (2x)
matika_cli plot --sexpr "(* (Int 2) (Sym x))" --var x \
  --xmin -2 --xmax 2 > derivative.svg
```

## Integration with Other Tools

### Pipe to File

```bash
matika_cli plot --sexpr "(Func sin (Sym x))" --var x > plot.svg
open plot.svg  # macOS
# xdg-open plot.svg  # Linux
```

### Batch Processing

```bash
for expr in "x^2" "x^3" "x^4"; do
  # (Convert to S-expr format)
  matika_cli plot --sexpr "..." --var x > "${expr}.svg"
done
```

### JSON API

```bash
# Use jq for JSON manipulation
echo '{"Integer": 5}' | matika_cli parse --json - | jq .
```

## Building and Packaging

### Release Build

```bash
cargo build --release -p cli
strip target/release/matika_cli  # Optional: reduce size
```

### Install Locally

```bash
cargo install --path crates/cli
# Binary in ~/.cargo/bin/matika_cli
```

### Cross-Compilation

```bash
# For Linux on macOS
cargo build --release --target x86_64-unknown-linux-gnu

# For Windows
cargo build --release --target x86_64-pc-windows-gnu
```

## Testing

Unit tests cover:
- Parse and simplify roundtrip
- Differentiation smoke test
- JSON parsing

Run tests:
```bash
cargo test -p cli
```

Integration tests (manual):
```bash
# Test all commands
./test_cli.sh
```

## Performance

Typical response times:
- **parse**: < 1ms
- **simplify**: < 10ms (small expressions)
- **diff**: < 5ms
- **integrate**: < 50ms (with partial fractions)
- **solve**: < 100ms (degree ≤ 5)
- **plot**: < 50ms (100 samples)

For large expressions (> 1000 nodes), times scale linearly.

## Limitations

### No Interactive Mode

CLI is batch-oriented. Each invocation is independent:
```bash
# No REPL; must re-parse each time
matika_cli parse --sexpr "..."
matika_cli simplify --sexpr "..."
```

### No File Input

Expressions must be inline:
```bash
# Cannot do:
# matika_cli parse --file expr.txt
```

**Workaround:** Use shell substitution:
```bash
matika_cli parse --sexpr "$(cat expr.txt)"
```

### Limited Error Messages

Errors are terse:
```
parse error: ...
not integrable
cannot solve completely
```

No detailed diagnostics or suggestions.

### No Configuration

All behavior is hardcoded. Cannot customize:
- Simplification rules
- Plot styling
- Output formatting

## Future Enhancements

- **REPL mode**: Interactive session with history and completion
- **File I/O**: Read expressions from files
- **Batch mode**: Process multiple expressions in one invocation
- **Verbose mode**: Detailed step-by-step output
- **Configuration**: Settings file for defaults
- **More formats**: Support MathML, Wolfram Language, etc.
- **Better errors**: Suggestions and detailed messages
- **Scripting**: Define functions, variables, multi-step computations

## Example Script

```bash
#!/bin/bash
# Workflow: differentiate, integrate back, verify identity

EXPR="(^ (Sym x) (Int 3))"

echo "Original: $EXPR"

# Differentiate
DERIV=$(matika_cli diff --sexpr "$EXPR" --var x)
echo "Derivative: $DERIV"

# Integrate back
# (Would need to convert text back to S-expr; omitted for brevity)

echo "Integrated: ..."
```

## References

- Depends on: `expr_core`, `simplify`, `calculus`, `solver`, `plot`, `io`
- Related: `bc`, `gnuplot`, `Mathematica CLI`, `SymPy`
