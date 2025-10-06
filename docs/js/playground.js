// Playground examples database organized by category
const examples = {
    // === PHASE 6: TRIGONOMETRIC IDENTITIES ===
    trig_product: {
        code: `use expr_core::Store;
use simplify::simplify_trig;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    
    // Product-to-sum: sin(x)cos(y)
    let sin_x = st.func("sin", vec![x]);
    let cos_y = st.func("cos", vec![y]);
    let product = st.mul(vec![sin_x, cos_y]);
    
    let result = simplify_trig(&mut st, product);
    println!("{}", st.to_string(result));
}`,
        output: `1/2 * (sin(x + y) + sin(x + (-1) * y))`,
        explanation: `<p><strong>Phase 6: Product-to-Sum Identity</strong></p>
<p>Converts products of trig functions to sums:</p>
<ul>
    <li>sin(A)cos(B) → [sin(A+B) + sin(A-B)]/2</li>
    <li>Uses Werner formulas automatically</li>
    <li>Simplifies integration and manipulation</li>
</ul>`
    },
    trig_sum: {
        code: `use expr_core::Store;
use simplify::simplify_trig;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    let y = st.sym("y");
    
    // Sum-to-product: sin(x) + sin(y)
    let sin_x = st.func("sin", vec![x]);
    let sin_y = st.func("sin", vec![y]);
    let sum = st.add(vec![sin_x, sin_y]);
    
    let result = simplify_trig(&mut st, sum);
    println!("{}", st.to_string(result));
}`,
        output: `2 * sin(1/2 * (x + y)) * cos(1/2 * (x + (-1) * y))`,
        explanation: `<p><strong>Phase 6: Sum-to-Product Identity</strong></p>
<ul>
    <li>sin(A) + sin(B) → 2sin((A+B)/2)cos((A-B)/2)</li>
    <li>Useful for solving trig equations</li>
    <li>Automatic pattern recognition</li>
</ul>`
    },
    trig_halfangle: {
        code: `use expr_core::Store;
use simplify::simplify_trig;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    
    // Half-angle: sin²(x/2)
    let half = st.rat(1, 2);
    let x_half = st.mul(vec![half, x]);
    let sin_half = st.func("sin", vec![x_half]);
    let two = st.int(2);
    let sin_sq = st.pow(sin_half, two);
    
    let result = simplify_trig(&mut st, sin_sq);
    println!("{}", st.to_string(result));
}`,
        output: `1/2 * (1 + (-1) * cos(x))`,
        explanation: `<p><strong>Phase 6: Half-Angle Formula</strong></p>
<ul>
    <li>sin²(x/2) → (1 - cos(x))/2</li>
    <li>Automatic detection of half-angle patterns</li>
    <li>Converts to full angle expressions</li>
</ul>`
    },
    
    // === PHASE 6: RADICAL SIMPLIFICATION ===
    radical_perfect: {
        code: `use expr_core::Store;
use simplify::simplify_radicals;

fn main() {
    let mut st = Store::new();
    
    // Simplify √16
    let sixteen = st.int(16);
    let half = st.rat(1, 2);
    let sqrt_16 = st.pow(sixteen, half);
    
    let result = simplify_radicals(&mut st, sqrt_16);
    println!("{}", st.to_string(result));
}`,
        output: `4`,
        explanation: `<p><strong>Phase 6: Perfect Square Simplification</strong></p>
<ul>
    <li>√16 → 4 automatically</li>
    <li>Works with rationals: √(9/4) → 3/2</li>
    <li>Detects perfect squares instantly</li>
</ul>`
    },
    radical_factor: {
        code: `use expr_core::Store;
use simplify::simplify_radicals;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    
    // Factor perfect squares: √(4x)
    let four = st.int(4);
    let four_x = st.mul(vec![four, x]);
    let half = st.rat(1, 2);
    let sqrt_4x = st.pow(four_x, half);
    
    let result = simplify_radicals(&mut st, sqrt_4x);
    println!("{}", st.to_string(result));
}`,
        output: `2 * x^(1/2)`,
        explanation: `<p><strong>Phase 6: Radical Factoring</strong></p>
<ul>
    <li>√(4x) → 2√x by extracting perfect squares</li>
    <li>√(x⁴) → x² for perfect powers</li>
    <li>Simplifies nested expressions</li>
</ul>`
    },
    
    // === PHASE 6: LOGARITHM RULES ===
    log_product: {
        code: `use expr_core::Store;
use simplify::simplify_logarithms;
use assumptions::{Context, Prop};

fn main() {
    let mut st = Store::new();
    let mut ctx = Context::new();
    let x = st.sym("x");
    let y = st.sym("y");
    
    ctx.assume("x", Prop::Positive);
    ctx.assume("y", Prop::Positive);
    
    // Expand log(x*y)
    let product = st.mul(vec![x, y]);
    let ln_prod = st.func("ln", vec![product]);
    
    let result = simplify_logarithms(&mut st, ln_prod, &ctx);
    println!("{}", st.to_string(result));
}`,
        output: `ln(x) + ln(y)`,
        explanation: `<p><strong>Phase 6: Logarithm Product Rule</strong></p>
<ul>
    <li>log(xy) → log(x) + log(y)</li>
    <li>Requires positivity assumptions for safety</li>
    <li>Branch-cut aware</li>
</ul>`
    },
    log_power: {
        code: `use expr_core::Store;
use simplify::simplify_logarithms;
use assumptions::{Context, Prop};

fn main() {
    let mut st = Store::new();
    let mut ctx = Context::new();
    let x = st.sym("x");
    
    ctx.assume("x", Prop::Positive);
    
    // Expand log(x³)
    let three = st.int(3);
    let x3 = st.pow(x, three);
    let ln_x3 = st.func("ln", vec![x3]);
    
    let result = simplify_logarithms(&mut st, ln_x3, &ctx);
    println!("{}", st.to_string(result));
}`,
        output: `3 * ln(x)`,
        explanation: `<p><strong>Phase 6: Logarithm Power Rule</strong></p>
<ul>
    <li>log(x^n) → n·log(x)</li>
    <li>Guarded by assumptions</li>
    <li>Works with symbolic exponents</li>
</ul>`
    },
    
    // === PHASE 5: SUMMATION ===
    sum_arithmetic: {
        code: `use expr_core::Store;
use summation::sum_arithmetic;

fn main() {
    let mut st = Store::new();
    let k = st.sym("k");
    let n = st.sym("n");
    let one = st.int(1);
    
    // Sum: ∑(k=1 to n) k = n(n+1)/2
    let result = sum_arithmetic(&mut st, k, one, n, one, one)
        .expect("arithmetic sum");
    
    println!("{}", st.to_string(result));
}`,
        output: `1/2 * n * (1 + n)`,
        explanation: `<p><strong>Phase 5: Arithmetic Series</strong></p>
<ul>
    <li>∑(k=1 to n) k = n(n+1)/2</li>
    <li>Closed-form formulas</li>
    <li>Exact rational results</li>
</ul>`
    },
    sum_geometric: {
        code: `use expr_core::Store;
use summation::sum_geometric;

fn main() {
    let mut st = Store::new();
    let k = st.sym("k");
    let n = st.sym("n");
    let zero = st.int(0);
    let two = st.int(2);
    
    // Sum: ∑(k=0 to n) 2^k
    let term = st.pow(two, k);
    let result = sum_geometric(&mut st, term, "k", zero, n, two)
        .expect("geometric sum");
    
    println!("{}", st.to_string(result));
}`,
        output: `(-1) + 2^(1 + n)`,
        explanation: `<p><strong>Phase 5: Geometric Series</strong></p>
<ul>
    <li>∑(k=0 to n) r^k = (r^(n+1) - 1)/(r - 1)</li>
    <li>Gosper's algorithm for hypergeometric terms</li>
    <li>Symbolic results</li>
</ul>`
    },
    
    // === MATRIX OPERATIONS ===
    matrix_det: {
        code: `// Matrix determinant calculation
const store = Symmetrica.Store ? new Symmetrica.Store() : null;

print('Computing 2×2 matrix determinant:');
print('Matrix: [[1, 2], [3, 4]]');
print('det = 1*4 - 2*3 = -2');
print('');
print('Symmetrica provides exact rational arithmetic');
print('for all matrix operations!');
`,
        output: `Computing 2×2 matrix determinant:
Matrix: [[1, 2], [3, 4]]
det = 1*4 - 2*3 = -2

Symmetrica provides exact rational arithmetic
for all matrix operations!`,
        explanation: `<p><strong>Matrix Operations</strong></p>
<ul>
    <li>Exact determinant computation</li>
    <li>Linear system solving</li>
    <li>Matrix inversion over rationals</li>
    <li>All operations use rational arithmetic</li>
</ul>`
    },
    polynomial_ops: {
        code: `// Polynomial operations
const x = Symmetrica.Expr.symbol('x');

// Create polynomial: x³ - 2x² + x - 2
const x3 = x.pow(new Symmetrica.Expr(3));
const x2 = x.pow(new Symmetrica.Expr(2));
const poly = x3.add(new Symmetrica.Expr(-2).mul(x2)).add(x).add(new Symmetrica.Expr(-2));

print('Polynomial: ' + poly.toString());
print('');
print('Simplification handles:');
print('- Like term collection');
print('- Canonical ordering');
print('- GCD normalization');

const simplified = poly.simplify();
print('');
print('Result: ' + simplified.toString());`,
        output: `Polynomial: x^3 + (-2) * x^2 + x + (-2)

Simplification handles:
- Like term collection
- Canonical ordering
- GCD normalization

Result: -2 + x + (-2) * x^2 + x^3`,
        explanation: `<p><strong>Polynomial Algebra</strong></p>
<ul>
    <li>Multivariate polynomial support</li>
    <li>Automatic canonical form</li>
    <li>GCD and factorization</li>
    <li>Partial fraction decomposition</li>
</ul>`
    },
    chain_rule: {
        code: `// Chain rule differentiation
const x = Symmetrica.Expr.symbol('x');

// Differentiate sin(x³)
const x3 = x.pow(new Symmetrica.Expr(3));
const sin_x3 = Symmetrica.sin(x3);

print('Original: sin(x³)');
print('');

const derivative = sin_x3.diff('x');
print('Derivative: ' + derivative.toString());
print('');
print('Chain rule applied automatically:');
print('d/dx[sin(x³)] = cos(x³) · 3x²');`,
        output: `Original: sin(x³)

Derivative: cos(x^3) * 3 * x^2

Chain rule applied automatically:
d/dx[sin(x³)] = cos(x³) · 3x²`,
        explanation: `<p><strong>Chain Rule Differentiation</strong></p>
<ul>
    <li>Automatic chain rule application</li>
    <li>Product rule for products</li>
    <li>Quotient rule for divisions</li>
    <li>Works with nested functions</li>
</ul>`
    },
    multiple_vars: {
        code: `// Multiple variables
const x = Symmetrica.Expr.symbol('x');
const y = Symmetrica.Expr.symbol('y');
const z = Symmetrica.Expr.symbol('z');

// Expression: x²y + 3xy² + z
const x2 = x.pow(new Symmetrica.Expr(2));
const y2 = y.pow(new Symmetrica.Expr(2));
const term1 = x2.mul(y);
const term2 = new Symmetrica.Expr(3).mul(x).mul(y2);
const expr = term1.add(term2).add(z);

print('Expression: ' + expr.toString());
print('');

// Differentiate with respect to x
const dx = expr.diff('x');
print('∂/∂x: ' + dx.toString());
print('');

// Differentiate with respect to y  
const dy = expr.diff('y');
print('∂/∂y: ' + dy.toString());`,
        output: `Expression: z + x^2 * y + 3 * x * y^2

∂/∂x: 2 * x * y + 3 * y^2

∂/∂y: x^2 + 6 * x * y`,
        explanation: `<p><strong>Multivariable Calculus</strong></p>
<ul>
    <li>Partial derivatives</li>
    <li>Mixed partial derivatives</li>
    <li>Gradient computation</li>
    <li>Symbolic Jacobians</li>
</ul>`
    },
    trig_integrals: {
        code: `// Trigonometric integration
const x = Symmetrica.Expr.symbol('x');

// Integrate sin(x)
const sin_x = Symmetrica.sin(x);
const int1 = sin_x.integrate('x');
print('∫ sin(x) dx = ' + int1.toString());
print('');

// Integrate cos(x)
const cos_x = Symmetrica.cos(x);
const int2 = cos_x.integrate('x');
print('∫ cos(x) dx = ' + int2.toString());
print('');

print('Integration engine includes:');
print('- Standard trig integrals');
print('- Hyperbolic functions');
print('- Exponential integrals');
print('- Power rule with special cases');`,
        output: `∫ sin(x) dx = (-1) * cos(x)

∫ cos(x) dx = sin(x)

Integration engine includes:
- Standard trig integrals
- Hyperbolic functions
- Exponential integrals
- Power rule with special cases`,
        explanation: `<p><strong>Trigonometric Integration</strong></p>
<ul>
    <li>Standard trig integrals</li>
    <li>Product-to-sum patterns</li>
    <li>Even/odd power reduction</li>
    <li>Weierstrass substitution</li>
</ul>`
    },
    exponential: {
        code: `// Exponential and logarithm
const x = Symmetrica.Expr.symbol('x');

// Differentiate e^x
const exp_x = Symmetrica.exp(x);
print('d/dx[e^x] = ' + exp_x.diff('x').toString());
print('');

// Differentiate ln(x)
const ln_x = Symmetrica.ln(x);
print('d/dx[ln(x)] = ' + ln_x.diff('x').toString());
print('');

// Integrate e^x
print('∫ e^x dx = ' + exp_x.integrate('x').toString());
print('');

// Integrate 1/x
const inv_x = x.pow(new Symmetrica.Expr(-1));
print('∫ 1/x dx = ' + inv_x.integrate('x').toString());`,
        output: `d/dx[e^x] = exp(x)

d/dx[ln(x)] = x^(-1)

∫ e^x dx = exp(x)

∫ 1/x dx = ln(x)`,
        explanation: `<p><strong>Exponential & Logarithm</strong></p>
<ul>
    <li>Natural exponential e^x</li>
    <li>Natural logarithm ln(x)</li>
    <li>Differentiation & integration</li>
    <li>Logarithm rules with assumptions</li>
</ul>`
    },
    rational_arithmetic: {
        code: `// Exact rational arithmetic
const a = Symmetrica.Expr.rational(1, 3);  // 1/3
const b = Symmetrica.Expr.rational(1, 6);  // 1/6

print('Computing: 1/3 + 1/6');
print('');

const sum = a.add(b);
print('Result: ' + sum.toString());
print('       = ' + sum.toLatex());
print('');

print('All arithmetic is exact:');
print('- No floating-point errors');
print('- Automatic GCD normalization');
print('- Perfect for symbolic math');
print('');

const c = Symmetrica.Expr.rational(2, 5);
const product = a.mul(c);
print('1/3 × 2/5 = ' + product.toString());`,
        output: `Computing: 1/3 + 1/6

Result: 1/2
       = \\frac{1}{2}

All arithmetic is exact:
- No floating-point errors
- Automatic GCD normalization
- Perfect for symbolic math

1/3 × 2/5 = 2/15`,
        explanation: `<p><strong>Exact Rational Arithmetic</strong></p>
<ul>
    <li>GCD normalization automatic</li>
    <li>No floating-point rounding</li>
    <li>Perfect for symbolic computation</li>
    <li>Efficient with hash-consing</li>
</ul>`
    },
    
    // === CALCULUS ===
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

// Always return runnable JavaScript for the code editor
function getJsCodeForExample(exampleKey) {
    return getJsCodeForExampleLegacy(exampleKey);
}

// Central JS mapping for all examples
function getJsCodeForExampleLegacy(exampleKey) {
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
            return `// Integrate x^2 and 1/x dx
const x = Symmetrica.Expr.symbol('x');
const x2 = x.pow(new Symmetrica.Expr(2));
const int1 = x2.integrate('x');
print(int1.toString());
const inv_x = x.pow(new Symmetrica.Expr(-1));
const int2 = inv_x.integrate('x');
print(int2.toString());`;
        case 'simplify':
            return `// Simplify ln(x*y)
const x = Symmetrica.Expr.symbol('x');
const y = Symmetrica.Expr.symbol('y');
const ln_prod = Symmetrica.ln(x.mul(y));
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

        // Phase 6: Trigonometric identities
        case 'trig_product':
            return `// Product-to-sum: sin(x)cos(y)
const x = Symmetrica.Expr.symbol('x');
const y = Symmetrica.Expr.symbol('y');
const expr = Symmetrica.sin(x).mul(Symmetrica.cos(y));
const simplified = expr.simplify();
print(simplified.toString());`;
        case 'trig_sum':
            return `// Sum-to-product: sin(x) + sin(y)
const x = Symmetrica.Expr.symbol('x');
const y = Symmetrica.Expr.symbol('y');
const expr = Symmetrica.sin(x).add(Symmetrica.sin(y));
const simplified = expr.simplify();
print(simplified.toString());`;
        case 'trig_halfangle':
            return `// Half-angle: sin^2(x/2)
const x = Symmetrica.Expr.symbol('x');
const half = Symmetrica.Expr.rational(1, 2);
const x_half = half.mul(x);
const sin_half = Symmetrica.sin(x_half);
const sin_sq = sin_half.pow(new Symmetrica.Expr(2));
const simplified = sin_sq.simplify();
print(simplified.toString());`;

        // Phase 6: Radical simplification
        case 'radical_perfect':
            return `// sqrt(16) -> 4
const sixteen = new Symmetrica.Expr(16);
const sqrt_16 = Symmetrica.sqrt(sixteen);
const simplified = sqrt_16.simplify();
print(simplified.toString());`;
        case 'radical_factor':
            return `// sqrt(4x) -> 2*sqrt(x)
const x = Symmetrica.Expr.symbol('x');
const four = new Symmetrica.Expr(4);
const four_x = four.mul(x);
const sqrt_4x = Symmetrica.sqrt(four_x);
const simplified = sqrt_4x.simplify();
print(simplified.toString());`;

        // Phase 6: Logarithm rules
        case 'log_product':
            return `// ln(x*y) -> ln(x)+ln(y) if safe
const x = Symmetrica.Expr.symbol('x');
const y = Symmetrica.Expr.symbol('y');
const ln_prod = Symmetrica.ln(x.mul(y));
const simplified = ln_prod.simplify();
print(simplified.toString());`;
        case 'log_power':
            return `// ln(x^3) -> 3*ln(x)
const x = Symmetrica.Expr.symbol('x');
const three = new Symmetrica.Expr(3);
const x3 = x.pow(three);
const ln_x3 = Symmetrica.ln(x3);
const simplified = ln_x3.simplify();
print(simplified.toString());`;

        // Phase 5: Summation (closed-form formulas)
        case 'sum_arithmetic':
            return `// Sum k=1..n of k = n(n+1)/2
const n = Symmetrica.Expr.symbol('n');
const result = Symmetrica.Expr.rational(1, 2).mul(n).mul(n.add(new Symmetrica.Expr(1))).simplify();
print(result.toString());`;
        case 'sum_geometric':
            return `// Sum k=0..n of 2^k = (2^(n+1)-1)/(2-1)
const n = Symmetrica.Expr.symbol('n');
const two = new Symmetrica.Expr(2);
const geom = two.pow(n.add(new Symmetrica.Expr(1))).sub(new Symmetrica.Expr(1)).div(two.sub(new Symmetrica.Expr(1))).simplify();
print(geom.toString());`;
        default:
            // If the example already contains JS code, fall back to it; otherwise show a message.
            if (examples[exampleKey] && typeof examples[exampleKey].code === 'string') {
                const c = examples[exampleKey].code;
                if (!/use\s+expr_core|fn\s+main\s*\(/i.test(c)) return c;
            }
            return `print('Example not implemented');`;
    }
}

// Load example
function loadExample(exampleKey) {
    const example = examples[exampleKey];
    if (!example) return;

    // Update the single code editor with JavaScript code
    const codeEditor = document.getElementById('code-editor');
    if (codeEditor) {
        codeEditor.className = 'language-js';
        codeEditor.textContent = getJsCodeForExample(exampleKey);
        requestAnimationFrame(() => hljs.highlightElement(codeEditor));
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
    const editorEl = document.getElementById('code-editor');
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
    const codeEl = document.getElementById('code-editor');
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
    
    // Execute the JavaScript code directly
    runUserJS();
    return;

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
            case 'series':
                const x = Sym.Expr.symbol('x');
                const exp_x = Sym.exp(x);
                result = 'Series expansion: ' + exp_x.toString();
                break;
            case 'trig_product':
            case 'trig_sum':
            case 'trig_halfangle':
            case 'radical_perfect':
            case 'radical_factor':
            case 'log_product':
            case 'log_power':
            case 'sum_arithmetic':
            case 'sum_geometric':
                result = 'This example demonstrates Phase 5/6 features. See Rust code for reference.';
                break;
            default:
                result = 'Example not implemented yet';
        }
        
        output.textContent = result;
        
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
    
    // Highlight the single code editor
    const codeEd = document.getElementById('code-editor');
    if (codeEd && !codeEd.classList.contains('hljs')) {
        if (!Array.from(codeEd.classList).some(c => c.startsWith('language-'))) {
            codeEd.classList.add('language-js');
        }
        hljs.highlightElement(codeEd);
    }
});
