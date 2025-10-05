# Serverless WASM Playground

The Symmetrica playground runs **100% in your browser** using WebAssembly. No server required!

## How It Works

1. **WASM Bundle**: The `docs/pkg/` directory contains the compiled WebAssembly binary and JavaScript glue code
2. **ES Module Loading**: `playground.html` loads the WASM as an ES module
3. **Live Execution**: Click "Run" to execute symbolic math operations directly in your browser

## Architecture

```
User clicks "Run"
    ↓
JavaScript calls WASM API
    ↓
Symmetrica WASM (Rust compiled to WASM)
    ↓
Result displayed in browser
```

## WASM API

The playground exposes the following API:

### Expression Creation
```javascript
const x = Symmetrica.Expr.symbol('x');
const num = new Symmetrica.Expr(42);
const rational = Symmetrica.Expr.rational(1, 2);
```

### Arithmetic
```javascript
const sum = x.add(num);
const product = x.mul(num);
const power = x.pow(new Symmetrica.Expr(2));
const quotient = x.div(num);
```

### Mathematical Functions
```javascript
const sin_x = Symmetrica.sin(x);
const cos_x = Symmetrica.cos(x);
const exp_x = Symmetrica.exp(x);
const ln_x = Symmetrica.ln(x);
const sqrt_x = Symmetrica.sqrt(x);
```

### Operations
```javascript
const simplified = expr.simplify();
const derivative = expr.diff('x');
const integral = expr.integrate('x');
const roots = expr.solve('x'); // Returns array
```

### Output Formats
```javascript
const str = expr.toString();      // Human-readable
const latex = expr.toLatex();     // LaTeX format
const sexpr = expr.toSExpr();     // S-expression
```

### Numeric Evaluation
```javascript
const result = expr.eval({ x: 3.14, y: 2.71 });
```

## Examples

### Differentiation
```javascript
const x = Symmetrica.Expr.symbol('x');
const x2 = x.pow(new Symmetrica.Expr(2));
const sin_x2 = Symmetrica.sin(x2);
const derivative = sin_x2.diff('x');
console.log(derivative.toString());
// Output: cos(x^2) * 2 * x
```

### Integration
```javascript
const x = Symmetrica.Expr.symbol('x');
const x2 = x.pow(new Symmetrica.Expr(2));
const integral = x2.integrate('x');
console.log(integral.toString());
// Output: 1/3 * x^3
```

### Equation Solving
```javascript
const x = Symmetrica.Expr.symbol('x');
const x2 = x.pow(new Symmetrica.Expr(2));
const three_x = new Symmetrica.Expr(3).mul(x);
const two = new Symmetrica.Expr(2);
const eq = x2.add(three_x).add(two);
const roots = eq.solve('x');
console.log(roots);
// Output: ["-2", "-1"]
```

## Building the WASM Bundle

The WASM bundle is automatically built by GitHub Actions on every push. To build locally:

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build WASM bundle
cd crates/wasm
wasm-pack build --release --target web --out-dir ../../docs/pkg

# The bundle is now in docs/pkg/
```

## GitHub Pages Deployment

The `.github/workflows/pages.yml` workflow:
1. Builds the WASM bundle on every push to `main`
2. Copies `docs/` to a staging directory
3. Deploys to GitHub Pages

No manual intervention required - push to `main` and the playground updates automatically!

## Performance

- **Bundle Size**: ~210KB WASM + 23KB JS (gzipped: ~85KB total)
- **Load Time**: <1 second on modern browsers
- **Execution**: Near-native speed (WASM is compiled, not interpreted)
- **Memory**: Efficient with resource limits (10,000 node max)

## Browser Compatibility

Requires a browser with WebAssembly support:
- ✅ Chrome 57+
- ✅ Firefox 52+
- ✅ Safari 11+
- ✅ Edge 16+

## Security

- **Sandboxed**: WASM runs in browser sandbox
- **No Server**: All computation client-side
- **Resource Limits**: Expression size capped at 10,000 nodes
- **No Network**: Zero external requests after initial load

## Troubleshooting

### "WASM load failed"
- Check browser console for errors
- Ensure browser supports WebAssembly
- Verify `docs/pkg/` files are accessible

### "Initializing WASM..."
- Wait a few seconds for WASM to load
- Check network tab for 404 errors on `.wasm` file

### Execution errors
- Open browser console (F12) for detailed error messages
- Verify expression syntax matches API

## Development

To add new examples to the playground:

1. Edit `docs/js/playground.js`
2. Add case to `runExample()` switch statement
3. Add example metadata to `examples` object
4. Test locally by opening `docs/playground.html` in browser

## Links

- **Live Playground**: https://sir-teo.github.io/Symmetrica/playground.html
- **GitHub Repository**: https://github.com/Sir-Teo/Symmetrica
- **WASM Source**: `crates/wasm/src/lib.rs`
