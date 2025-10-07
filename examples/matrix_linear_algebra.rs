//! Matrix operations: determinant and linear system solving with exact rational arithmetic.

use arith::Q;
use matrix::MatrixQ;

fn main() {
    println!("=== Matrix and Linear Algebra ===\n");

    // Example 1: Determinant of a 2x2 matrix
    let m2x2 = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
    println!("Matrix A (2x2):");
    println!("  [[1, 2],");
    println!("   [3, 4]]");

    let det = m2x2.det_bareiss().expect("determinant");
    println!("  det(A) = {}/{} = {}", det.0, det.1, det.0 as f64 / det.1 as f64);

    // Example 2: Identity matrix
    let identity = MatrixQ::identity(3);
    let det_identity = identity.det_bareiss().expect("determinant");
    println!("\nIdentity matrix (3x3): det = {}/{}", det_identity.0, det_identity.1);

    // Example 3: Solve a linear system
    // System: 2x + y = 5
    //         x + 3y = 10
    // Matrix form: [[2, 1], [1, 3]] * [x, y] = [5, 10]
    let coeff_matrix = MatrixQ::from_i64(2, 2, &[2, 1, 1, 3]);
    let rhs = vec![Q::new(5, 1), Q::new(10, 1)];

    println!("\nSolving linear system:");
    println!("  2x + y = 5");
    println!("  x + 3y = 10");

    match coeff_matrix.solve_bareiss(&rhs) {
        Ok(Some(solution)) => {
            println!("  Solution:");
            println!("    x = {}/{}", solution[0].0, solution[0].1);
            println!("    y = {}/{}", solution[1].0, solution[1].1);

            // Verify solution
            let x_val = solution[0].0 as f64 / solution[0].1 as f64;
            let y_val = solution[1].0 as f64 / solution[1].1 as f64;
            println!("  Verification:");
            println!("    2*{:.2} + {:.2} = {:.2} (expected 5)", x_val, y_val, 2.0 * x_val + y_val);
            println!(
                "    {:.2} + 3*{:.2} = {:.2} (expected 10)",
                x_val,
                y_val,
                x_val + 3.0 * y_val
            );
        }
        Ok(None) => println!("  System is singular (no unique solution)"),
        Err(e) => println!("  Error: {}", e),
    }

    // Example 4: 3x3 system with fractional coefficients
    let m3x3 = MatrixQ::new(
        3,
        3,
        vec![
            Q::new(2, 1),
            Q::new(1, 1),
            Q::new(0, 1),
            Q::new(1, 1),
            Q::new(3, 1),
            Q::new(1, 1),
            Q::new(0, 1),
            Q::new(2, 1),
            Q::new(1, 1),
        ],
    );
    let b3 = vec![Q::new(5, 1), Q::new(10, 1), Q::new(7, 1)];

    println!("\nSolving 3x3 system:");
    println!("  [[2, 1, 0],   [x]   [5]");
    println!("   [1, 3, 1], * [y] = [10]");
    println!("   [0, 2, 1]]   [z]   [7]");

    match m3x3.solve_bareiss(&b3) {
        Ok(Some(sol)) => {
            println!("  Solution:");
            for (i, var) in ['x', 'y', 'z'].iter().enumerate() {
                println!("    {} = {}/{}", var, sol[i].0, sol[i].1);
            }
        }
        Ok(None) => println!("  Singular system"),
        Err(e) => println!("  Error: {}", e),
    }

    // Example 5: Singular matrix (determinant = 0)
    let singular = MatrixQ::from_i64(2, 2, &[1, 2, 2, 4]); // Second row = 2 * first row
    let det_singular = singular.det_bareiss().expect("determinant");
    println!("\nSingular matrix [[1, 2], [2, 4]]:");
    println!("  det = {}/{} (should be 0)", det_singular.0, det_singular.1);

    println!("\n=== Done ===");
}
