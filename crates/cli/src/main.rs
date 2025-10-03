use expr_core::Store;
use simplify::simplify;

fn main() {
    let mut st = Store::new();
    let x = st.sym("x");
    let one = st.int(1);

    // Build (x + 1)^2 + 3*(x + 1)
    let xp1 = st.add(vec![x, one]);
    let two = st.int(2);
    let pow = st.pow(xp1, two);
    let three = st.int(3);
    let three_times = st.mul(vec![three, xp1]);
    let expr = st.add(vec![pow, three_times]);

    println!("Raw:        {}", st.to_string(expr));
    let s = simplify(&mut st, expr);
    println!("Simplified: {}", st.to_string(s));

    // Another: (2x + 3x) + (1/2)x + 1/2
    let x = st.sym("x");
    let two_b = st.int(2);
    let two_x = st.mul(vec![two_b, x]);
    let three_b = st.int(3);
    let three_x = st.mul(vec![three_b, x]);
    let half = st.rat(1, 2);
    let half_x = st.mul(vec![half, x]);
    let expr2 = st.add(vec![two_x, three_x, half_x, half]);
    println!("Expr2 raw:  {}", st.to_string(expr2));
    let s2 = simplify(&mut st, expr2);
    println!("Expr2 simp: {}", st.to_string(s2));
}
