# Symmetrica Python Bindings

Python bindings for the Symmetrica symbolic computation engine using PyO3.

## Installation

Build the Python extension module:

```bash
cd crates/api
pip install maturin
maturin develop
```

## Usage

```python
import symmetrica as sym

# Create symbolic expressions
x = sym.Expr.sym("x")
y = sym.Expr.sym("y")

# Arithmetic operations
expr = x**2 + 3*x + 2
print(expr)  # Display expression

# Simplification
simplified = expr.simplify()
print(simplified)

# Differentiation
derivative = expr.diff("x")
print(f"d/dx: {derivative}")

# Integration
integral = x.integrate("x")
print(f"∫x dx: {integral}")

# Solve equations
# Solve x^2 + 3*x + 2 = 0
roots = expr.solve("x")
for root in roots:
    print(f"Root: {root}")

# Substitution
result = expr.subs("x", sym.Expr.int(5))
print(f"Substituted: {result}")

# Numerical evaluation
value = sym.Expr.int(42).evalf()
print(f"Value: {value}")

# Mathematical functions
sin_expr = sym.sin(x)
cos_expr = sym.cos(x)
exp_expr = sym.exp(x)
ln_expr = sym.ln(x)
sqrt_expr = sym.sqrt(x)

# Export to LaTeX
latex = expr.to_latex()
print(f"LaTeX: {latex}")

# Plot as SVG
svg = expr.plot("x", -10.0, 10.0, samples=200)
with open("plot.svg", "w") as f:
    f.write(svg)
```

## Features

- **Symbolic arithmetic**: `+`, `-`, `*`, `/`, `**` operators
- **Simplification**: Automatic algebraic simplification
- **Calculus**: Differentiation and integration
- **Equation solving**: Univariate polynomial equation solving
- **Pattern substitution**: Replace symbols with expressions
- **Numerical evaluation**: Evaluate expressions to floating-point
- **Export formats**: LaTeX and S-expression output
- **Plotting**: Generate SVG plots of functions
- **Mathematical functions**: `sin`, `cos`, `tan`, `exp`, `ln`, `log`, `sqrt`

## API Reference

### Expr Class

#### Static Methods

- `Expr.int(val: int) -> Expr` - Create integer expression
- `Expr.rat(num: int, den: int) -> Expr` - Create rational expression
- `Expr.sym(name: str) -> Expr` - Create symbol expression

#### Instance Methods

- `simplify() -> Expr` - Simplify expression
- `diff(var: str) -> Expr` - Differentiate with respect to variable
- `integrate(var: str) -> Expr` - Integrate with respect to variable
- `subs(var: str, val: Expr) -> Expr` - Substitute variable with value
- `solve(var: str) -> List[Expr]` - Solve equation for variable
- `evalf() -> float` - Evaluate numerically
- `to_latex() -> str` - Convert to LaTeX string
- `to_sexpr() -> str` - Convert to S-expression string
- `plot(var: str, x_min: float, x_max: float, samples: int = 200) -> str` - Generate SVG plot

#### Operators

- `+`, `-`, `*`, `/`, `**` - Arithmetic operations
- `-expr` - Negation
- `str(expr)` - String representation

### Module Functions

- `sin(x: Expr) -> Expr` - Sine function
- `cos(x: Expr) -> Expr` - Cosine function
- `tan(x: Expr) -> Expr` - Tangent function
- `exp(x: Expr) -> Expr` - Exponential function
- `ln(x: Expr) -> Expr` - Natural logarithm
- `log(x: Expr) -> Expr` - Logarithm (alias for ln)
- `sqrt(x: Expr) -> Expr` - Square root

## Examples

See `examples/python_demo.py` for more examples.

## License

Dual licensed under MIT or Apache-2.0.
