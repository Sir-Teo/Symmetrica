// Playground examples database
const examples = {
    basic: {
        code: `use expr_core::Store;
use simplify::simplify;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    
    // Build expression: x² + 3x + 1
    let expr = st.add(vec![
        st.pow(x, st.int(2)),
        st.mul(vec![st.int(3), x]),
        st.int(1),
    ]);
    
    let simplified = simplify(&mut st, expr);
    println!("{}", st.to_string(simplified));
}`,
        output: `1 + 3 * x + x^2`,
        explanation: `<p>This example demonstrates basic expression building and simplification:</p>
<ul>
    <li>Creates a <code>Store</code> to manage expressions</li>
    <li>Defines symbol <code>x</code></li>
    <li>Builds a polynomial expression</li>
    <li>Simplifies to canonical form with deterministic ordering</li>
</ul>`
    },
    diff: {
        code: `use expr_core::Store;
use calculus::diff;
use simplify::simplify;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    
    // Build: sin(x²)
    let x_squared = st.pow(x, st.int(2));
    let expr = st.func("sin", vec![x_squared]);
    
    // Differentiate with chain rule
    let derivative = diff(&mut st, expr, "x");
    let result = simplify(&mut st, derivative);
    
    println!("{}", st.to_string(result));
}`,
        output: `cos(x^2) * 2 * x`,
        explanation: `<p>This example shows symbolic differentiation with the chain rule:</p>
<ul>
    <li>Builds nested function sin(x²)</li>
    <li>Applies chain rule automatically</li>
    <li>Result: d/dx sin(x²) = cos(x²) · 2x</li>
</ul>`
    },
    integrate: {
        code: `use expr_core::Store;
use calculus::integrate;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    
    // Integrate x² dx
    let expr = st.pow(x, st.int(2));
    let integral = integrate(&mut st, expr, "x").unwrap();
    
    println!("{}", st.to_string(integral));
    
    // Integrate 1/x dx
    let inv_x = st.pow(x, st.int(-1));
    let ln_result = integrate(&mut st, inv_x, "x").unwrap();
    
    println!("{}", st.to_string(ln_result));
}`,
        output: `1/3 * x^3
ln(x)`,
        explanation: `<p>Symbolic integration with power rule:</p>
<ul>
    <li>∫ x² dx = x³/3 using power rule</li>
    <li>∫ 1/x dx = ln(x) as special case</li>
    <li>Returns <code>Option</code> - <code>None</code> if no pattern matches</li>
</ul>`
    },
    simplify: {
        code: `use expr_core::Store;
use simplify::simplify_with;
use assumptions::{Context, Prop};

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    
    // Setup assumptions
    let mut ctx = Context::new();
    ctx.assume("x", Prop::Positive);
    ctx.assume("y", Prop::Positive);
    
    // Simplify ln(x * y) → ln(x) + ln(y)
    let product = st.mul(vec![x, y]);
    let ln_product = st.func("ln", vec![product]);
    let result = simplify_with(&mut st, ln_product, &ctx);
    
    println!("{}", st.to_string(result));
}`,
        output: `ln(x) + ln(y)`,
        explanation: `<p>Domain-aware simplification with assumptions:</p>
<ul>
    <li>Assumes x and y are positive</li>
    <li>Applies logarithm product rule safely</li>
    <li>ln(xy) → ln(x) + ln(y) when x, y > 0</li>
    <li>Without assumptions, transformation would be unsafe</li>
</ul>`
    },
    solve: {
        code: `use expr_core::Store;
use solver::solve_univariate;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    
    // Solve x² + 3x + 2 = 0
    let eq = st.add(vec![
        st.pow(x, st.int(2)),
        st.mul(vec![st.int(3), x]),
        st.int(2),
    ]);
    
    let roots = solve_univariate(&mut st, eq, "x").unwrap();
    for root in roots {
        println!("x = {}", st.to_string(root));
    }
}`,
        output: `x = -2
x = -1`,
        explanation: `<p>Exact polynomial equation solving:</p>
<ul>
    <li>Solves x² + 3x + 2 = 0</li>
    <li>Factors as (x + 1)(x + 2) = 0</li>
    <li>Returns exact rational roots</li>
    <li>Supports up to quartic equations</li>
</ul>`
    },
    series: {
        code: `use expr_core::Store;
use calculus::maclaurin;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    
    // Taylor series for exp(x) at 0
    let expr = st.func("exp", vec![x]);
    let series = maclaurin(&st, expr, "x", 5).unwrap();
    
    // Print terms
    for (k, (n, d)) in series.coeffs.iter().enumerate() {
        if *n != 0 {
            if *d == 1 {
                println!("x^{}: {}", k, n);
            } else {
                println!("x^{}: {}/{}", k, n, d);
            }
        }
    }
}`,
        output: `x^0: 1
x^1: 1
x^2: 1/2
x^3: 1/6
x^4: 1/24
x^5: 1/120`,
        explanation: `<p>Maclaurin series expansion:</p>
<ul>
    <li>Computes Taylor series at x = 0</li>
    <li>exp(x) = 1 + x + x²/2! + x³/3! + ...</li>
    <li>Exact rational coefficients</li>
    <li>Supports composition and arithmetic</li>
</ul>`
    }
};

// Generate runnable JS code for an example using the WASM API
function getJsCodeForExample(exampleKey) {
    switch (exampleKey) {
        case 'basic':
            return `// Basic polynomial simplification
const x = Symmetrica.Expr.symbol('x');
const x2 = x.pow(new Symmetrica.Expr(2));
const three_x = new Symmetrica.Expr(3).mul(x);
const one = new Symmetrica.Expr(1);
const expr = x2.add(three_x).add(one);
const simplified = expr.simplify();
print(simplified.toString());`;
        case 'diff':
            return `// Differentiate sin(x^2) w.r.t. x
const x = Symmetrica.Expr.symbol('x');
const x2 = x.pow(new Symmetrica.Expr(2));
const sin_x2 = Symmetrica.sin(x2);
const derivative = sin_x2.diff('x');
print(derivative.toString());`;
        case 'integrate':
            return `// Integrate x^2 dx
const x = Symmetrica.Expr.symbol('x');
const x2 = x.pow(new Symmetrica.Expr(2));
const integral = x2.integrate('x');
print(integral.toString());`;
        case 'simplify':
            return `// Simplify ln(x*y)
const x = Symmetrica.Expr.symbol('x');
const y = Symmetrica.Expr.symbol('y');
const prod = x.mul(y);
const ln_prod = Symmetrica.ln(prod);
const simplified = ln_prod.simplify();
print(simplified.toString());`;
        case 'solve':
            return `// Solve x^2 + 3x + 2 = 0
const x = Symmetrica.Expr.symbol('x');
const x2 = x.pow(new Symmetrica.Expr(2));
const three_x = new Symmetrica.Expr(3).mul(x);
const two = new Symmetrica.Expr(2);
const eq = x2.add(three_x).add(two);
const roots = eq.solve('x');
print('Roots: ' + JSON.stringify(roots));`;
        case 'series':
            return `// Series expansion (Maclaurin) is not yet exposed in the WASM API.
print('Series example is not runnable in the browser yet.');`;
        default:
            return `print('Example not implemented');`;
    }
}

// Load example
function loadExample(exampleKey) {
    const example = examples[exampleKey];
    if (!example) return;
    
    // Update Rust reference code
    const codeEditorRust = document.getElementById('code-editor-rust');
    if (codeEditorRust) {
        codeEditorRust.className = 'language-rust';
        codeEditorRust.textContent = example.code;
        requestAnimationFrame(() => hljs.highlightElement(codeEditorRust));
    }

    // Update runnable JS code
    const codeEditorJs = document.getElementById('code-editor-js');
    if (codeEditorJs) {
        codeEditorJs.className = 'language-js';
        codeEditorJs.textContent = getJsCodeForExample(exampleKey);
        requestAnimationFrame(() => hljs.highlightElement(codeEditorJs));
    }
    
    // Update output
    const output = document.getElementById('output');
    output.textContent = example.output;
    
    // Update explanation
    const explanation = document.getElementById('explanation');
    explanation.innerHTML = example.explanation;
    
    // Update active button
    document.querySelectorAll('.example-btn').forEach(btn => {
        btn.classList.remove('active');
    });
    document.querySelector(`[data-example="${exampleKey}"]`).classList.add('active');
}

// Copy code
function copyCode() {
    // Determine active tab in the playground editor
    const activeTab = document.querySelector('.tabs-container .tab-button.active');
    const isJs = activeTab && activeTab.getAttribute('data-tab') === 'js';
    const editorEl = isJs
        ? document.getElementById('code-editor-js')
        : (document.getElementById('code-editor-rust') || document.getElementById('code-editor'));
    const code = editorEl ? editorEl.textContent : '';
    navigator.clipboard.writeText(code).then(() => {
        const btn = document.querySelector('.btn-copy');
        const originalText = btn.textContent;
        btn.textContent = 'Copied!';
        setTimeout(() => {
            btn.textContent = originalText;
        }, 2000);
    });
}

// Parse numbers from code editor
function extractNumbers(code) {
    const numbers = [];
    // Match st.int(number) patterns
    const intMatches = code.matchAll(/st\.int\((-?\d+)\)/g);
    for (const match of intMatches) {
        numbers.push(parseInt(match[1]));
    }
    return numbers;
}

// Append output helper
function printToOutput(line) {
    const output = document.getElementById('output');
    if (!output) return;
    const cur = output.textContent || '';
    const s = String(line);
    output.textContent = cur ? cur + '\n' + s : s;
}

// Execute user-provided JavaScript using the WASM API
async function runUserJS() {
    const status = document.getElementById('output-status');
    const output = document.getElementById('output');
    const codeEl = document.getElementById('code-editor-js');
    if (!codeEl) return;
    const code = codeEl.textContent;

    // Reset output
    output.textContent = '';
    status.textContent = 'Running...';

    const Sym = window.Symmetrica;
    const print = (msg) => printToOutput(msg);
    const logLike = (...args) => {
        try {
            const parts = args.map(a => {
                if (typeof a === 'string') return a;
                try { return JSON.stringify(a); } catch { return String(a); }
            });
            print(parts.join(' '));
        } catch (e) {
            print(String(args.join(' ')));
        }
    };

    try {
        const fn = new Function('Symmetrica', 'print', 'console', code);
        const maybe = fn(Sym, print, { log: logLike, warn: logLike, error: logLike });
        if (maybe && typeof maybe.then === 'function') {
            await maybe;
        }
        status.textContent = 'Success';
    } catch (e) {
        print('Error: ' + (e && e.message ? e.message : String(e)));
        status.textContent = 'Error';
        console.error('User code error:', e);
    }
}

// Run example with WASM
function runExample() {
    const status = document.getElementById('output-status');
    const output = document.getElementById('output');
    
    if (!window.SYM_WASM_READY) {
        status.textContent = 'Initializing WASM...';
        output.textContent = 'Please wait for WASM to load...';
        return;
    }
    
    // If JS tab is active, execute user JS code (actual code execution)
    const activeTab = document.querySelector('.tabs-container .tab-button.active');
    const isJs = activeTab && activeTab.getAttribute('data-tab') === 'js';
    if (isJs) {
        runUserJS();
        return;
    }

    status.textContent = 'Running...';

    try {
        const Sym = window.Symmetrica;
        
        // Get active example and code from editor
        const activeBtn = document.querySelector('.example-btn.active');
        let exampleKey = activeBtn ? activeBtn.getAttribute('data-example') : 'basic';
        const rustEditor = document.getElementById('code-editor-rust') || document.getElementById('code-editor');
        const code = rustEditor ? rustEditor.textContent : '';

        // Try to infer the example from the pasted code (helps when user copies another snippet)
        const c = code;
        if (c.includes('maclaurin(')) {
            exampleKey = 'series';
        } else if (c.includes('solve_univariate') || c.includes('.solve(')) {
            exampleKey = 'solve';
        } else if (c.includes('integrate(')) {
            exampleKey = 'integrate';
        } else if (c.includes('diff(')) {
            exampleKey = 'diff';
        } else if (c.includes('simplify_with(')) {
            exampleKey = 'simplify';
        }

        // Sync the active button with the inferred example for UX consistency
        const currentActive = activeBtn ? activeBtn.getAttribute('data-example') : null;
        if (exampleKey && exampleKey !== currentActive) {
            document.querySelectorAll('.example-btn').forEach(btn => btn.classList.remove('active'));
            const target = document.querySelector(`[data-example="${exampleKey}"]`);
            if (target) target.classList.add('active');
        }
        
        // Extract numbers from the edited code
        const numbers = extractNumbers(code);
        
        let result;
        
        switch(exampleKey) {
            case 'basic': {
                const x = Sym.Expr.symbol('x');
                // Use numbers from code if available, otherwise use defaults
                const pow_exp = numbers[0] || 2;
                const coeff = numbers[1] || 3;
                const const_term = numbers[2] || 1;
                
                const x_pow = x.pow(new Sym.Expr(pow_exp));
                const coeff_x = new Sym.Expr(coeff).mul(x);
                const one = new Sym.Expr(const_term);
                const expr = x_pow.add(coeff_x).add(one);
                const simplified = expr.simplify();
                result = simplified.toString();
                break;
            }
            case 'diff': {
                const x = Sym.Expr.symbol('x');
                const pow_exp = numbers[0] || 2;
                const x_pow = x.pow(new Sym.Expr(pow_exp));
                const sin_x_pow = Sym.sin(x_pow);
                const derivative = sin_x_pow.diff('x');
                result = derivative.toString();
                break;
            }
            case 'integrate': {
                const x = Sym.Expr.symbol('x');
                const pow_exp = numbers[0] || 2;
                const x_pow = x.pow(new Sym.Expr(pow_exp));
                const integral = x_pow.integrate('x');
                result = integral.toString();
                break;
            }
            case 'simplify': {
                const x = Sym.Expr.symbol('x');
                const y = Sym.Expr.symbol('y');
                const product = x.mul(y);
                const ln_product = Sym.ln(product);
                const simplified = ln_product.simplify();
                result = simplified.toString();
                break;
            }
            case 'solve': {
                const x = Sym.Expr.symbol('x');
                // Use numbers from code if available
                const pow_exp = numbers[0] || 2;
                const coeff = numbers[1] || 3;
                const const_term = numbers[2] || 2;
                
                const x_pow = x.pow(new Sym.Expr(pow_exp));
                const coeff_x = new Sym.Expr(coeff).mul(x);
                const const_expr = new Sym.Expr(const_term);
                const eq = x_pow.add(coeff_x).add(const_expr);
                const roots = eq.solve('x');
                result = 'Roots: ' + JSON.stringify(roots);
                break;
            }
            case 'series': {
                const x = Sym.Expr.symbol('x');
                const exp_x = Sym.exp(x);
                result = 'Series expansion: ' + exp_x.toString();
                break;
            }
            default:
                result = 'Example not implemented yet';
        }
        
        output.textContent = result;
        status.textContent = 'Success';
        
    } catch (e) {
        output.textContent = 'Error: ' + e.message;
        status.textContent = 'Error';
        console.error('Execution error:', e);
    }
}

// Setup event listeners
document.addEventListener('DOMContentLoaded', () => {
    // Example buttons
    document.querySelectorAll('.example-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const example = btn.getAttribute('data-example');
            loadExample(example);
        });
    });
    
    // Highlight both editors if present
    const jsEd = document.getElementById('code-editor-js');
    if (jsEd && !jsEd.classList.contains('hljs')) {
        if (!Array.from(jsEd.classList).some(c => c.startsWith('language-'))) {
            jsEd.classList.add('language-js');
        }
        hljs.highlightElement(jsEd);
    }

    const rustEd = document.getElementById('code-editor-rust');
    if (rustEd && !rustEd.classList.contains('hljs')) {
        if (!Array.from(rustEd.classList).some(c => c.startsWith('language-'))) {
            rustEd.classList.add('language-rust');
        }
        hljs.highlightElement(rustEd);
    }
});
