# WASM Bindings

WebAssembly bindings for Symmetrica (Phase K implementation).

## Overview

The `symmetrica-wasm` crate provides a lightweight WASM API for using Symmetrica in browser and Node.js environments. It includes resource guards to prevent excessive memory usage.

## Building

Install `wasm-pack`:
```bash
cargo install wasm-pack
```

Build for different targets:

**Web (ES modules):**
```bash
cd crates/wasm
wasm-pack build --target web
```

**Node.js:**
```bash
wasm-pack build --target nodejs
```

**Bundlers (webpack, rollup, etc):**
```bash
wasm-pack build --target bundler
```

## API

### Expr Class

The main class for symbolic expressions.

**Construction:**
```javascript
const x = Expr.symbol('x');          // Symbol
const five = new Expr(5);             // Integer
const half = Expr.rational(1, 2);     // Rational 1/2
```

**Arithmetic Operations:**
```javascript
const sum = x.add(five);              // x + 5
const diff = x.sub(five);             // x - 5
const prod = x.mul(five);             // x * 5
const quot = x.div(five);             // x / 5
const power = x.pow(new Expr(2));     // x^2
const negated = x.neg();              // -x
```

**Symbolic Operations:**
```javascript
// Simplification
const expr = x.add(x);
const simplified = expr.simplify();   // 2*x

// Differentiation
const f = x.pow(new Expr(2));
const df = f.diff('x');               // 2*x

// Integration
const g = x.pow(new Expr(2));
const integral = g.integrate('x');    // x^3/3

// Substitution
const h = x.pow(new Expr(2)).add(x);
const at5 = h.subs('x', new Expr(5)); // 25 + 5

// Solving
const eq = x.pow(new Expr(2)).sub(new Expr(1));
const roots = eq.solve('x');          // [-1, 1]
```

**Evaluation:**
```javascript
const expr = x.pow(new Expr(2)).add(y);
const value = expr.eval({ x: 3.0, y: 4.0 });  // 13.0
```

**Export:**
```javascript
expr.toString()    // "x^2 + y"
expr.toLatex()     // "x^{2} + y"
expr.toSExpr()     // "(+ (^ x 2) y)"
```

### Module Functions

Mathematical functions:

```javascript
import { sin, cos, tan, exp, ln, sqrt } from './pkg/symmetrica_wasm.js';

const sinX = sin(x);
const cosX = cos(x);
const tanX = tan(x);
const expX = exp(x);
const lnX = ln(x);
const sqrtX = sqrt(x);

// Chain operations
const deriv = sin(x.pow(new Expr(2))).diff('x');
```

## Resource Limits

To prevent DoS attacks and runaway computations:

- **Max nodes**: 10,000 nodes per expression tree
- Operations exceeding this throw a JavaScript error
- Configurable at compile time via `MAX_NODES` constant

## Examples

### Browser Usage

```html
<!DOCTYPE html>
<html>
<head><title>Symmetrica Demo</title></head>
<body>
    <div id="output"></div>
    <script type="module">
        import init, { Expr, sin } from './pkg/symmetrica_wasm.js';
        
        async function demo() {
            await init();
            
            const x = Expr.symbol('x');
            const expr = x.pow(new Expr(2)).add(new Expr(3).mul(x));
            
            document.getElementById('output').textContent = 
                `Expression: ${expr.toString()}\n` +
                `Derivative: ${expr.diff('x').toString()}`;
        }
        
        demo();
    </script>
</body>
</html>
```

### Node.js Usage

```javascript
const init = require('./pkg/symmetrica_wasm');

async function main() {
    await init();
    const { Expr, sin, cos } = require('./pkg/symmetrica_wasm');
    
    const x = Expr.symbol('x');
    const expr = sin(x).pow(new Expr(2)).add(cos(x).pow(new Expr(2)));
    
    console.log('Expression:', expr.toString());
    console.log('Simplified:', expr.simplify().toString());
}

main();
```

### React Integration

```jsx
import { useEffect, useState } from 'react';
import init, { Expr } from 'symmetrica-wasm';

function Calculator() {
    const [loaded, setLoaded] = useState(false);
    const [result, setResult] = useState('');
    
    useEffect(() => {
        init().then(() => setLoaded(true));
    }, []);
    
    const compute = () => {
        if (!loaded) return;
        
        const x = Expr.symbol('x');
        const expr = x.pow(new Expr(2)).add(new Expr(1));
        const deriv = expr.diff('x');
        
        setResult(deriv.toString());
    };
    
    return (
        <div>
            <button onClick={compute} disabled={!loaded}>
                Compute Derivative
            </button>
            <p>Result: {result}</p>
        </div>
    );
}
```

## Error Handling

All operations that can fail return `Result` types that throw JavaScript errors:

```javascript
try {
    const expr = Expr.rational(1, 0);  // Division by zero
} catch (error) {
    console.error('Error:', error);
}

try {
    const huge = buildHugeExpression();  // > 10k nodes
    huge.simplify();
} catch (error) {
    console.error('Expression too large:', error);
}
```

## Testing

WASM tests use `wasm-bindgen-test`:

```bash
cd crates/wasm
wasm-pack test --node
wasm-pack test --firefox --headless
wasm-pack test --chrome --headless
```

## Bundle Size

Approximate sizes (gzipped):
- Core WASM: ~200-300 KB
- JS glue code: ~10-20 KB

Optimize for production:
```bash
wasm-pack build --target web --release
```

## Roadmap Alignment

This implements **Phase K: WASM bindings** from the roadmap:
- ✅ Small surface API (parse → expr; evaluate/simplify/diff/solve; print)
- ✅ Resource caps enforced (max nodes limit)
- ✅ Browser and Node.js targets
- ✅ Deterministic outputs

## See Also

- [Python bindings](api.md) - PyO3-based Python API
- [CLI](cli.md) - Command-line interface
- [wasm-bindgen documentation](https://rustwasm.github.io/docs/wasm-bindgen/)
