//! Examples demonstrating symbolic simplification capabilities
//!
//! Run with: cargo run --example symbolic_simplification

use calculus::simplify_calculus;
use expr_core::Store;

fn main() {
    println!("🧮 Symmetrica Symbolic Simplification Examples\n");
    println!("{}", "=".repeat(60));

    // Example 1: Perfect square roots
    println!("\n📐 Example 1: Perfect Square Roots");
    println!("{}", "-".repeat(60));
    {
        let mut st = Store::new();
        let four = st.int(4);
        let sqrt_four = st.func("sqrt", vec![four]);
        println!("Before: {}", st.to_string(sqrt_four));
        
        let result = simplify_calculus(&mut st, sqrt_four);
        println!("After:  {}", st.to_string(result));
        println!("✓ √4 → 2");
    }

    {
        let mut st = Store::new();
        let nine = st.int(9);
        let sqrt_nine = st.func("sqrt", vec![nine]);
        println!("\nBefore: {}", st.to_string(sqrt_nine));
        
        let result = simplify_calculus(&mut st, sqrt_nine);
        println!("After:  {}", st.to_string(result));
        println!("✓ √9 → 3");
    }

    {
        let mut st = Store::new();
        let ratio = st.rat(4, 9);
        let sqrt_ratio = st.func("sqrt", vec![ratio]);
        println!("\nBefore: {}", st.to_string(sqrt_ratio));
        
        let result = simplify_calculus(&mut st, sqrt_ratio);
        println!("After:  {}", st.to_string(result));
        println!("✓ √(4/9) → 2/3");
    }

    // Example 2: Exponential/Logarithmic identities
    println!("\n\n📈 Example 2: Exponential/Logarithmic Identities");
    println!("{}", "-".repeat(60));
    {
        let mut st = Store::new();
        let x = st.sym("x");
        let exp_x = st.func("exp", vec![x]);
        let ln_exp_x = st.func("ln", vec![exp_x]);
        println!("Before: {}", st.to_string(ln_exp_x));
        
        let result = simplify_calculus(&mut st, ln_exp_x);
        println!("After:  {}", st.to_string(result));
        println!("✓ ln(e^x) → x");
    }

    {
        let mut st = Store::new();
        let x = st.sym("x");
        let ln_x = st.func("ln", vec![x]);
        let exp_ln_x = st.func("exp", vec![ln_x]);
        println!("\nBefore: {}", st.to_string(exp_ln_x));
        
        let result = simplify_calculus(&mut st, exp_ln_x);
        println!("After:  {}", st.to_string(result));
        println!("✓ e^(ln x) → x");
    }

    {
        let mut st = Store::new();
        let zero = st.int(0);
        let exp_zero = st.func("exp", vec![zero]);
        println!("\nBefore: {}", st.to_string(exp_zero));
        
        let result = simplify_calculus(&mut st, exp_zero);
        println!("After:  {}", st.to_string(result));
        println!("✓ e^0 → 1");
    }

    // Example 3: Inverse trigonometric identities
    println!("\n\n🔺 Example 3: Inverse Trigonometric Identities");
    println!("{}", "-".repeat(60));
    {
        let mut st = Store::new();
        let x = st.sym("x");
        let tan_x = st.func("tan", vec![x]);
        let atan_tan_x = st.func("atan", vec![tan_x]);
        println!("Before: {}", st.to_string(atan_tan_x));
        
        let result = simplify_calculus(&mut st, atan_tan_x);
        println!("After:  {}", st.to_string(result));
        println!("✓ atan(tan x) → x");
    }

    {
        let mut st = Store::new();
        let zero = st.int(0);
        let atan_zero = st.func("atan", vec![zero]);
        println!("\nBefore: {}", st.to_string(atan_zero));
        
        let result = simplify_calculus(&mut st, atan_zero);
        println!("After:  {}", st.to_string(result));
        println!("✓ atan(0) → 0");
    }

    // Example 4: Pythagorean identity (NEW!)
    println!("\n\n⭐ Example 4: Pythagorean Identity (NEW!)");
    println!("{}", "-".repeat(60));
    {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let two = st.int(2);
        let sin2 = st.pow(sinx, two);
        let cos2 = st.pow(cosx, two);
        let sum = st.add(vec![sin2, cos2]);
        println!("Before: {}", st.to_string(sum));
        
        let result = simplify_calculus(&mut st, sum);
        println!("After:  {}", st.to_string(result));
        println!("✓ sin²(x) + cos²(x) → 1");
    }

    {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let two = st.int(2);
        let sin2 = st.pow(sinx, two);
        let cos2 = st.pow(cosx, two);
        let sum = st.add(vec![cos2, sin2]); // Order doesn't matter
        println!("\nBefore: {}", st.to_string(sum));
        
        let result = simplify_calculus(&mut st, sum);
        println!("After:  {}", st.to_string(result));
        println!("✓ cos²(x) + sin²(x) → 1 (order independent)");
    }

    {
        let mut st = Store::new();
        let x = st.sym("x");
        let two_const = st.int(2);
        let two_x = st.mul(vec![two_const, x]);
        let sin_2x = st.func("sin", vec![two_x]);
        let cos_2x = st.func("cos", vec![two_x]);
        let two_exp = st.int(2);
        let sin2 = st.pow(sin_2x, two_exp);
        let cos2 = st.pow(cos_2x, two_exp);
        let sum = st.add(vec![sin2, cos2]);
        println!("\nBefore: {}", st.to_string(sum));
        
        let result = simplify_calculus(&mut st, sum);
        println!("After:  {}", st.to_string(result));
        println!("✓ sin²(2x) + cos²(2x) → 1 (works with any argument)");
    }

    {
        let mut st = Store::new();
        let x = st.sym("x");
        let three = st.int(3);
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let two = st.int(2);
        let sin2 = st.pow(sinx, two);
        let cos2 = st.pow(cosx, two);
        let sum = st.add(vec![three, sin2, cos2]);
        println!("\nBefore: {}", st.to_string(sum));
        
        let result = simplify_calculus(&mut st, sum);
        println!("After:  {}", st.to_string(result));
        println!("✓ 3 + sin²(x) + cos²(x) → 4 (combines with other terms)");
    }

    // Example 5: Nested simplification
    println!("\n\n🌀 Example 5: Nested Simplification");
    println!("{}", "-".repeat(60));
    {
        let mut st = Store::new();
        let x = st.sym("x");
        let ln_x = st.func("ln", vec![x]);
        let exp_ln_x = st.func("exp", vec![ln_x]);
        let four = st.int(4);
        let sqrt_four = st.func("sqrt", vec![four]);
        let product = st.mul(vec![exp_ln_x, sqrt_four]);
        println!("Before: {}", st.to_string(product));
        
        let result = simplify_calculus(&mut st, product);
        println!("After:  {}", st.to_string(result));
        println!("✓ e^(ln x) · √4 → 2x");
    }

    {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let two = st.int(2);
        let sin2 = st.pow(sinx, two);
        let cos2 = st.pow(cosx, two);
        let sum_trig = st.add(vec![sin2, cos2]);
        
        let nine = st.int(9);
        let sqrt_nine = st.func("sqrt", vec![nine]);
        let product = st.mul(vec![sum_trig, sqrt_nine]);
        println!("\nBefore: {}", st.to_string(product));
        
        let result = simplify_calculus(&mut st, product);
        println!("After:  {}", st.to_string(result));
        println!("✓ (sin²x + cos²x) · √9 → 3");
    }

    println!("\n{}", "=".repeat(60));
    println!("✅ All simplification examples complete!");
    println!("\nKey Features:");
    println!("  • Perfect square root simplification");
    println!("  • Exponential/logarithmic inverse cancellation");
    println!("  • Inverse trigonometric simplification");
    println!("  • Pythagorean identity: sin²x + cos²x → 1");
    println!("  • Recursive simplification of nested expressions");
    println!("  • Argument-independent pattern matching");
}
