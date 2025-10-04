# io - Input/Output Module

## Overview

The `io` crate provides serialization and pretty-printing for Symmetrica expressions in multiple formats: LaTeX, S-expressions, and JSON. All formats support bidirectional conversion (parsing and generation).

## Modules

- **latex**: LaTeX pretty printer
- **sexpr**: S-expression parser and serializer
- **json**: JSON parser and serializer

## LaTeX Output

```rust
pub fn to_latex(st: &Store, id: ExprId) -> String
```

Converts an expression to LaTeX format suitable for mathematical typesetting.

### Features

- **Precedence-aware**: Automatically adds parentheses when needed
- **Deterministic**: Same expression always produces same output
- **Minimal**: No redundant formatting
- **Standard macros**: Uses `\sin`, `\cos`, `\exp`, `\ln`, `\frac`, `\cdot`

### Examples

**Basic expressions:**
```rust
use io::to_latex;

let x = st.sym("x");
let expr = st.pow(x, st.int(2));
assert_eq!(to_latex(&st, expr), "x^{2}");
```

**Fractions:**
```rust
let frac = st.rat(1, 3);
assert_eq!(to_latex(&st, frac), "\\frac{1}{3}");
```

**Functions:**
```rust
let sin_x = st.func("sin", vec![x]);
assert_eq!(to_latex(&st, sin_x), "\\sin(x)");

let cos_x2 = st.func("cos", vec![st.pow(x, st.int(2))]);
assert_eq!(to_latex(&st, cos_x2), "\\cos(x^{2})");
```

**Multiplication:**
```rust
let two_x = st.mul(vec![st.int(2), x]);
assert_eq!(to_latex(&st, two_x), "2 \\cdot x");
```

**Complex expressions:**
```rust
// (x + 1)^2
let xp1 = st.add(vec![x, st.int(1)]);
let expr = st.pow(xp1, st.int(2));
// Result: (x + 1)^{2} or (1 + x)^{2} (depends on canonical order)
```

### Parenthesization Rules

1. **Addition in multiplication**: `(a + b) * c`
2. **Lower precedence in power**: `(a + b)^c`, `(a * b)^c`
3. **Addition never parenthesized at top level**

### Special Character Handling

**Underscore escaping:**
```rust
let x1 = st.sym("x_1");
assert_eq!(to_latex(&st, x1), "x\\_1");
```

LaTeX treats `_` as subscript, so it's escaped as `\\_` for symbol names.

### Supported Functions

The following functions are translated to LaTeX macros:
- `sin` → `\sin`
- `cos` → `\cos`
- `exp` → `\exp`
- `ln` → `\ln`

Unknown functions are rendered as-is: `f(x)`.

## S-Expression Format

S-expressions provide a simple, Lisp-like syntax for expressions.

### Serialization

```rust
pub fn to_sexpr(st: &Store, id: ExprId) -> String
```

Converts an expression to S-expression format:
```rust
let x = st.sym("x");
let expr = st.pow(x, st.int(3));
assert_eq!(to_sexpr(&st, expr), "(^ (Sym x) (Int 3))");
```

### Parsing

```rust
pub fn from_sexpr(st: &mut Store, s: &str) -> Result<ExprId, String>
```

Parses an S-expression string into an expression:
```rust
let expr = from_sexpr(&mut st, "(+ (Int 1) (Int 2))").unwrap();
// Result: 1 + 2 (canonicalized to 3)
```

### Syntax

**Atoms:**
- `(Int n)`: Integer constant
- `(Rat n d)`: Rational n/d
- `(Sym name)`: Symbol

**Compound:**
- `(+ e1 e2 ... eN)`: Addition
- `(* e1 e2 ... eN)`: Multiplication
- `(^ base exp)`: Power (binary)
- `(Func name e1 e2 ... eN)`: Function application

### Examples

**Polynomial:**
```sexpr
(+ (* (Int 2) (^ (Sym x) (Int 2)))
   (* (Int 3) (Sym x))
   (Int 1))
```
Represents: `2x² + 3x + 1`

**Function:**
```sexpr
(Func sin (^ (Sym x) (Int 2)))
```
Represents: `sin(x²)`

**Rational:**
```sexpr
(* (Rat 1 2) (Sym x))
```
Represents: `(1/2) * x`

## JSON Format

JSON format enables integration with web applications and other tools.

### Serialization

```rust
pub fn to_json(st: &Store, id: ExprId) -> String
```

Converts an expression to JSON:
```rust
let x = st.sym("x");
let expr = st.pow(x, st.int(3));
let json = to_json(&st, expr);
// {"Pow": {"base": {"Symbol": "x"}, "exp": {"Integer": 3}}}
```

### Parsing

```rust
pub fn from_json(st: &mut Store, s: &str) -> Result<ExprId, String>
```

Parses a JSON string into an expression:
```rust
let json = r#"{"Integer": 5}"#;
let expr = from_json(&mut st, json).unwrap();
```

### Schema

**Atoms:**
```json
{"Integer": 42}
{"Rational": {"num": 1, "den": 2}}
{"Symbol": "x"}
```

**Compound:**
```json
{"Add": {"terms": [e1, e2, ...]}}
{"Mul": {"factors": [e1, e2, ...]}}
{"Pow": {"base": e1, "exp": e2}}
{"Function": {"name": "sin", "args": [e]}}
```

### Examples

**Simple expression:**
```json
{
  "Add": {
    "terms": [
      {"Integer": 1},
      {"Symbol": "x"}
    ]
  }
}
```
Represents: `1 + x`

**Complex expression:**
```json
{
  "Mul": {
    "factors": [
      {"Rational": {"num": 1, "den": 2}},
      {"Pow": {
        "base": {"Symbol": "x"},
        "exp": {"Integer": 2}
      }}
    ]
  }
}
```
Represents: `(1/2) * x²`

## Format Comparison

| Feature | LaTeX | S-expr | JSON |
|---------|-------|--------|------|
| **Human readable** | ++ | + | + |
| **Machine readable** | - | ++ | ++ |
| **Bidirectional** | No | Yes | Yes |
| **Compact** | + | ++ | - |
| **Math rendering** | ++ | - | - |
| **Web friendly** | - | - | ++ |

## Integration with CLI

The CLI uses these formats for input/output:

```bash
# Parse from S-expression
matika_cli parse --sexpr "(^ (Sym x) (Int 2))"

# Parse from JSON
matika_cli parse --json '{"Pow": {"base": {"Symbol": "x"}, "exp": {"Integer": 2}}}'

# Output formats:
# text:   x^2
# latex:  x^{2}
# json:   {"Pow": {...}}
# sexpr:  (^ (Sym x) (Int 2))
```

## Error Handling

### Parsing Errors

Both `from_sexpr` and `from_json` return `Result<ExprId, String>`:

```rust
let result = from_sexpr(&mut st, "(+ (Int 1");
assert!(result.is_err());  // Unclosed parenthesis

let result = from_json(&mut st, "{\"Integer\": abc}");
assert!(result.is_err());  // Invalid JSON syntax
```

Error messages are descriptive but minimal.

## Testing

All formats have comprehensive tests:
- Roundtrip conversion (expr → format → expr)
- Edge cases (empty lists, nested structures)
- Error handling (malformed input)
- Special characters (underscores, quotes)

Run tests:
```bash
cargo test -p io
```

## Performance

- **LaTeX**: O(n) in expression size
- **S-expr**: O(n) generation, O(n) parsing
- **JSON**: O(n) for both directions

All formats are designed for human-scale expressions (< 10K nodes).

## Limitations

### LaTeX
- **One-way**: No LaTeX parser (parsing LaTeX is very complex)
- **Limited functions**: Only sin, cos, exp, ln have special rendering
- **No inline math mode**: Output is plain LaTeX, wrap in `$...$` or `\[...\]`

### S-expr
- **Minimal validation**: Parser is lenient and may accept invalid input
- **No error recovery**: First error stops parsing
- **Whitespace sensitive**: Requires proper spacing

### JSON
- **Verbose**: Much larger than S-expr for same expression
- **No schema validation**: Invalid structures may parse but fail later
- **String-based**: Numbers are JSON integers, not arbitrary precision

## Example: Multi-Format Output

```rust
use expr_core::Store;
use io::{to_latex, to_sexpr, to_json};

let mut st = Store::new();
let x = st.sym("x");
let expr = st.add(vec![
    st.pow(x, st.int(2)),
    st.mul(vec![st.int(2), x]),
    st.int(1),
]);

println!("Text:  {}", st.to_string(expr));
println!("LaTeX: {}", to_latex(&st, expr));
println!("S-exp: {}", to_sexpr(&st, expr));
println!("JSON:  {}", to_json(&st, expr));

// Output:
// Text:  1 + 2 * x + x^2
// LaTeX: 1 + 2 \cdot x + x^{2}
// S-exp: (+ (Int 1) (* (Int 2) (Sym x)) (^ (Sym x) (Int 2)))
// JSON:  {"Add": {"terms": [...]}}
```

## Future Enhancements

- **LaTeX parser**: Enable LaTeX → Expr conversion
- **MathML**: XML-based math markup
- **Mathematica format**: Compatibility with Wolfram Language
- **Pretty-printing options**: Configurable indentation, line width
- **Binary format**: Compact serialization for large expressions
- **Streaming**: Parse/generate large expressions incrementally

## References

- Depends on: `expr_core`
- Used by: `cli`
- Related formats: OpenMath, Content MathML, Mathematica's FullForm
