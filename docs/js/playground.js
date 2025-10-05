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

// Load example
function loadExample(exampleKey) {
    const example = examples[exampleKey];
    if (!example) return;
    
    // Update code
    const codeEditor = document.getElementById('code-editor');
    codeEditor.textContent = example.code;
    hljs.highlightElement(codeEditor);
    
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
    const code = document.getElementById('code-editor').textContent;
    navigator.clipboard.writeText(code).then(() => {
        const btn = document.querySelector('.btn-copy');
        const originalText = btn.textContent;
        btn.textContent = 'Copied!';
        setTimeout(() => {
            btn.textContent = originalText;
        }, 2000);
    });
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
    
    status.textContent = 'Running...';
    
    try {
        const Sym = window.Symmetrica;
        
        // Get active example
        const activeBtn = document.querySelector('.example-btn.active');
        const exampleKey = activeBtn ? activeBtn.getAttribute('data-example') : 'basic';
        
        let result;
        
        switch(exampleKey) {
            case 'basic': {
                const x = Sym.Expr.symbol('x');
                const x2 = x.pow(new Sym.Expr(2));
                const three_x = new Sym.Expr(3).mul(x);
                const one = new Sym.Expr(1);
                const expr = x2.add(three_x).add(one);
                const simplified = expr.simplify();
                result = simplified.toString();
                break;
            }
            case 'diff': {
                const x = Sym.Expr.symbol('x');
                const x2 = x.pow(new Sym.Expr(2));
                const sin_x2 = Sym.sin(x2);
                const derivative = sin_x2.diff('x');
                result = derivative.toString();
                break;
            }
            case 'integrate': {
                const x = Sym.Expr.symbol('x');
                const x2 = x.pow(new Sym.Expr(2));
                const integral = x2.integrate('x');
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
                const x2 = x.pow(new Sym.Expr(2));
                const three_x = new Sym.Expr(3).mul(x);
                const two = new Sym.Expr(2);
                const eq = x2.add(three_x).add(two);
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
    
    // Initial highlight
    hljs.highlightAll();
});
