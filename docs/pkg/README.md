# Symmetrica WASM Bindings

WebAssembly bindings for the Symmetrica symbolic computation engine.

## Building

Requirements:
- Rust toolchain
- `wasm-pack` (install with `cargo install wasm-pack`)

Build for web:
```bash
cd crates/wasm
wasm-pack build --target web
```

Build for Node.js:
```bash
wasm-pack build --target nodejs
```

Build for bundlers (webpack, etc):
```bash
wasm-pack build --target bundler
```

## Usage

### Browser (ES Modules)

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Symmetrica WASM Demo</title>
</head>
<body>
    <script type="module">
        import init, { Expr, sin, cos, exp, ln, sqrt } from './pkg/symmetrica_wasm.js';

        async function run() {
            await init();

            // Create expressions
            const x = Expr.symbol('x');
            const two = new Expr(2);
            
            // x^2
            const x2 = x.pow(two);
            console.log('x^2 =', x2.toString());
            
            // Differentiate: d/dx(x^2) = 2*x
            const deriv = x2.diff('x');
            console.log('d/dx(x^2) =', deriv.toString());
            
            // Integrate: ∫x^2 dx = x^3/3
            const integral = x2.integrate('x');
            console.log('∫x^2 dx =', integral.toString());
            
            // Mathematical functions
            const sinX = sin(x);
            console.log('sin(x) =', sinX.toString());
            
            // Solve equations: x^2 - 5x + 6 = 0
            const five = new Expr(5);
            const six = new Expr(6);
            const negFiveX = five.neg().mul(x);
            const equation = x2.add(negFiveX).add(six);
            const roots = equation.solve('x');
            console.log('Roots:', roots);
            
            // Numerical evaluation
            const expr = x.pow(two).add(x);
            const value = expr.subs('x', new Expr(5)).eval({});
            console.log('f(5) =', value);
            
            // Export to LaTeX
            console.log('LaTeX:', x2.toLatex());
        }

        run();
    </script>
</body>
</html>
```

### Node.js

```javascript
const { Expr, sin, cos, exp, ln, sqrt } = require('./pkg/symmetrica_wasm');

// Create symbolic expressions
const x = Expr.symbol('x');
const two = new Expr(2);

// Operations
const x2 = x.pow(two);
console.log('x^2 =', x2.toString());

// Calculus
const deriv = x2.diff('x');
console.log('d/dx(x^2) =', deriv.toString());

const integral = x2.integrate('x');
console.log('∫x^2 dx =', integral.toString());

// Solve equations
const roots = x2.add(x.mul(new Expr(-5))).add(new Expr(6)).solve('x');
console.log('Roots:', roots);
```

## API Reference

### Expr Class

**Constructors:**
- `new Expr(value: number)` - Create integer expression
- `Expr.symbol(name: string)` - Create symbolic variable
- `Expr.rational(num: number, den: number)` - Create rational number

**Methods:**
- `add(other: Expr): Expr` - Addition
- `sub(other: Expr): Expr` - Subtraction
- `mul(other: Expr): Expr` - Multiplication
- `div(other: Expr): Expr` - Division
- `pow(other: Expr): Expr` - Exponentiation
- `neg(): Expr` - Negation
- `simplify(): Expr` - Algebraic simplification
- `diff(var: string): Expr` - Differentiation
- `integrate(var: string): Expr` - Integration
- `subs(var: string, value: Expr): Expr` - Substitution
- `solve(var: string): string[]` - Solve equation
- `eval(bindings: object): number` - Numerical evaluation
- `toString(): string` - String representation
- `toLatex(): string` - LaTeX format
- `toSExpr(): string` - S-expression format

**Module Functions:**
- `sin(x: Expr): Expr` - Sine
- `cos(x: Expr): Expr` - Cosine
- `tan(x: Expr): Expr` - Tangent
- `exp(x: Expr): Expr` - Exponential
- `ln(x: Expr): Expr` - Natural logarithm
- `sqrt(x: Expr): Expr` - Square root

## Resource Limits

To prevent DoS attacks and excessive memory usage:
- Maximum expression tree size: 10,000 nodes
- Operations that exceed this limit throw an error

## Examples

See `examples/` directory for:
- `demo.html` - Browser demo
- `node_demo.js` - Node.js demo
- `calculus.html` - Calculus examples

## License

Dual licensed under MIT or Apache-2.0.
