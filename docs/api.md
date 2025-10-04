# API and Bindings

Python bindings for Symmetrica via PyO3 (Phase K implementation).

## Python Bindings (PyO3)

The `api` crate provides Python bindings for the Symmetrica symbolic computation engine.

### Installation

Using `maturin` for development:

```bash
cd crates/api
pip install maturin
maturin develop
```

For production builds:

```bash
maturin build --release
pip install target/wheels/symmetrica-*.whl
```

### Core API

#### Expression Class

The `Expr` class wraps symbolic expressions with Python-friendly methods:

**Construction:**
- `Expr.int(val: int)` - Create integer literal
- `Expr.rat(num: int, den: int)` - Create rational number
- `Expr.sym(name: str)` - Create symbolic variable

**Operators:**
- Arithmetic: `+`, `-`, `*`, `/`, `**`, unary `-`
- These create new symbolic expressions that can be further manipulated

**Methods:**
- `simplify()` - Apply algebraic simplification
- `diff(var: str)` - Differentiate with respect to variable
- `integrate(var: str)` - Integrate (raises error if unsupported)
- `subs(var: str, val: Expr)` - Substitute symbol with expression
- `solve(var: str)` - Solve univariate equation, returns list of roots
- `evalf()` - Numerical evaluation to f64
- `to_latex()` - Export as LaTeX string
- `to_sexpr()` - Export as S-expression
- `plot(var: str, x_min: float, x_max: float, samples: int)` - Generate SVG plot

#### Module Functions

Common mathematical functions:
- `sin(x)`, `cos(x)`, `tan(x)`
- `exp(x)`, `ln(x)`, `log(x)`
- `sqrt(x)`

### Examples

**Basic Algebra:**

```python
import symmetrica as sym

x = sym.Expr.sym("x")
y = sym.Expr.sym("y")

# Build expression: (x + 1)^2
expr = (x + sym.Expr.int(1))**sym.Expr.int(2)
simplified = expr.simplify()
print(simplified)  # Output: (1 + x)^2
```

**Calculus:**

```python
# Differentiation
x = sym.Expr.sym("x")
f = x**sym.Expr.int(3) + sym.Expr.int(2)*x
df = f.diff("x")
print(df)  # 2 + 3*x^2

# Integration
g = x**sym.Expr.int(2)
integral = g.integrate("x")
print(integral)  # 1/3 * x^3
```

**Equation Solving:**

```python
# Solve x^2 - 5x + 6 = 0
x = sym.Expr.sym("x")
eq = x**sym.Expr.int(2) + sym.Expr.int(-5)*x + sym.Expr.int(6)
roots = eq.solve("x")
for root in roots:
    print(f"x = {root}")  # x = 2, x = 3
```

**Substitution:**

```python
x = sym.Expr.sym("x")
y = sym.Expr.sym("y")
expr = x**sym.Expr.int(2) + x

# Substitute x with y+1
result = expr.subs("x", y + sym.Expr.int(1))
print(result.simplify())
```

**Functions:**

```python
x = sym.Expr.sym("x")

# Trigonometric derivative
sin_x = sym.sin(x)
cos_x = sin_x.diff("x")  # cos(x)

# Exponential integral
exp_x = sym.exp(x)
integral = exp_x.integrate("x")  # exp(x)
```

**Plotting:**

```python
x = sym.Expr.sym("x")
expr = x**sym.Expr.int(2)

svg = expr.plot("x", -5.0, 5.0, samples=200)
with open("plot.svg", "w") as f:
    f.write(svg)
```

### Implementation Notes

- Each `Expr` instance owns a private `Store` for memory efficiency via hash-consing
- Operations between expressions rebuild them in a fresh store to maintain immutability
- Errors (e.g., unsupported integration, division by zero) raise Python `ValueError`
- The bindings use PyO3 0.20 for seamless Rust-Python interop

### Testing

Integration tests in `crates/api/tests/integration_test.rs` verify the Rust API.
Python tests can be added in `crates/api/python/tests/`.

### Roadmap Alignment

This implements **Phase K** of the roadmap:
- ✅ Python bindings (pyo3) with idiomatic API
- ✅ Core operations: build, simplify, diff, integrate, solve
- ✅ Serialization: LaTeX, S-expr
- ✅ Numeric evaluation and plotting
- 🔲 WASM bindings (future)
- 🔲 C FFI/ABI (future)

### See Also

- [PyO3 Documentation](https://pyo3.rs/)
- [Maturin](https://github.com/PyO3/maturin) for building Python packages
- Main README for Rust API usage
