# matrix - Linear Algebra Module

## Overview

The `matrix` crate provides exact matrix operations over rational numbers (Q). It implements fraction-free determinant computation and linear system solving using Bareiss algorithm and Cramer's rule.

## Core Type: MatrixQ

```rust
pub struct MatrixQ {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Q>,  // Row-major storage
}
```

A matrix with rational entries stored in row-major order.

## Construction

### New Matrix
```rust
pub fn new(rows: usize, cols: usize, data: Vec<Q>) -> Self
```

Creates a matrix from a data vector:
```rust
use matrix::MatrixQ;
use arith::Q;

// 2x2 matrix: [[1, 2], [3, 4]]
let m = MatrixQ::new(2, 2, vec![
    Q(1, 1), Q(2, 1),
    Q(3, 1), Q(4, 1),
]);
```

**Precondition:** `data.len() == rows * cols` (panics otherwise)

### From i64 Array
```rust
pub fn from_i64(rows: usize, cols: usize, data: &[i64]) -> Self
```

Convenient constructor for integer matrices:
```rust
let m = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
// Same as above but from integers
```

### Identity Matrix
```rust
pub fn identity(n: usize) -> Self
```

Creates an n×n identity matrix:
```rust
let I = MatrixQ::identity(3);
// [[1, 0, 0],
//  [0, 1, 0],
//  [0, 0, 1]]
```

## Element Access

### Get
```rust
pub fn get(&self, r: usize, c: usize) -> Q
```

Returns the element at row `r`, column `c` (0-indexed):
```rust
let m = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
assert_eq!(m.get(0, 1), Q(2, 1));  // First row, second column
```

### Set
```rust
pub fn set(&mut self, r: usize, c: usize, v: Q)
```

Sets the element at position `(r, c)`:
```rust
let mut m = MatrixQ::identity(2);
m.set(0, 1, Q(5, 1));
// [[1, 5],
//  [0, 1]]
```

## Determinant

```rust
pub fn det_bareiss(&self) -> Result<Q, &'static str>
```

Computes the determinant using **rational Gaussian elimination with partial pivoting**.

**Algorithm:**
1. Forward elimination with row pivoting
2. Track sign changes from row swaps
3. Determinant = sign × product of diagonal

**Returns:**
- `Ok(Q)`: Determinant value (Q(0, 1) for singular matrices)
- `Err`: If matrix is not square

### Examples

**2×2 determinant:**
```rust
// det([[1, 2], [3, 4]]) = 1*4 - 2*3 = -2
let m = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
let det = m.det_bareiss().unwrap();
assert_eq!(det, Q(-2, 1));
```

**3×3 determinant:**
```rust
// [[2, 0, 1],
//  [1, 1, 0],
//  [0, 3, 1]]
// det = 2*(1*1 - 0*3) - 0 + 1*(1*3 - 1*0) = 2 + 3 = 5
let m = MatrixQ::from_i64(3, 3, &[
    2, 0, 1,
    1, 1, 0,
    0, 3, 1,
]);
let det = m.det_bareiss().unwrap();
assert_eq!(det, Q(5, 1));
```

**Identity determinant:**
```rust
let I = MatrixQ::identity(4);
assert_eq!(I.det_bareiss().unwrap(), Q(1, 1));
```

**Singular matrix:**
```rust
// [[1, 2], [2, 4]] has det = 0 (second row is 2× first)
let m = MatrixQ::from_i64(2, 2, &[1, 2, 2, 4]);
assert_eq!(m.det_bareiss().unwrap(), Q(0, 1));
```

## Linear System Solving

```rust
pub fn solve_bareiss(&self, b: &[Q]) -> Result<Option<Vec<Q>>, &'static str>
```

Solves the linear system `Ax = b` using **Cramer's rule**.

**Returns:**
- `Ok(Some(x))`: Unique solution vector
- `Ok(None)`: Singular matrix (no unique solution)
- `Err`: If matrix is not square or `b.len() ≠ rows`

### Algorithm (Cramer's Rule)

For each variable `x_i`:
1. Replace column `i` of A with b to get A_i
2. Compute `x_i = det(A_i) / det(A)`

**Note:** Inefficient for large systems (O(n⁴)), but exact and simple for small matrices.

### Examples

**2×2 system:**
```rust
// [[1, 2], [3, 4]] * [x, y]^T = [5, 11]^T
// Solution: x = 1, y = 2
let A = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
let b = vec![Q(5, 1), Q(11, 1)];

let x = A.solve_bareiss(&b).unwrap().unwrap();
assert_eq!(x, vec![Q(1, 1), Q(2, 1)]);
```

**3×3 system:**
```rust
// [[2, 1, 0],
//  [1, 3, 1],
//  [0, 2, 1]] * x = [5, 10, 7]^T
// Solution: x = [2, 1, 5]^T
let A = MatrixQ::from_i64(3, 3, &[
    2, 1, 0,
    1, 3, 1,
    0, 2, 1,
]);
let b = vec![Q(5, 1), Q(10, 1), Q(7, 1)];

let x = A.solve_bareiss(&b).unwrap().unwrap();
assert_eq!(x, vec![Q(2, 1), Q(1, 1), Q(5, 1)]);
```

**Singular system:**
```rust
// [[1, 2], [2, 4]] is singular (det = 0)
let A = MatrixQ::from_i64(2, 2, &[1, 2, 2, 4]);
let b = vec![Q(3, 1), Q(6, 1)];

let result = A.solve_bareiss(&b).unwrap();
assert!(result.is_none());  // No unique solution
```

**Empty system:**
```rust
let A = MatrixQ::new(0, 0, vec![]);
let b = vec![];
let x = A.solve_bareiss(&b).unwrap().unwrap();
assert_eq!(x.len(), 0);
```

## Storage Format

**Row-major order:** Element at position `(r, c)` is stored at index `r * cols + c`.

**Example:**
```rust
// Matrix [[1, 2, 3],
//         [4, 5, 6]]
// Stored as: [1, 2, 3, 4, 5, 6]
//            └─ row 0 ─┘└─ row 1 ─┘
```

## Error Handling

All operations return `Result` with descriptive error messages:

```rust
// Non-square matrix
let m = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
assert!(m.det_bareiss().is_err());

// Wrong RHS size
let A = MatrixQ::identity(2);
let b = vec![Q(1, 1)];  // Should have 2 elements
assert!(A.solve_bareiss(&b).is_err());
```

## Performance

### Time Complexity
- **det_bareiss**: O(n³) for n×n matrix (Gaussian elimination)
- **solve_bareiss**: O(n⁴) due to Cramer's rule (n determinants of size n)

### Space Complexity
- **det_bareiss**: O(n²) for matrix copy during elimination
- **solve_bareiss**: O(n²) for temporary matrices

### Practical Limits
- Efficient for n ≤ 10
- Usable for n ≤ 50 (with patience)
- For larger systems, use iterative methods or factorizations (not implemented)

## Numerical Stability

**Advantage of exact arithmetic:**
- No rounding errors
- Results are mathematically exact
- Determinants and solutions are precise rationals

**Disadvantage:**
- Intermediate denominators can grow large
- May require arbitrary precision for very large systems (not implemented)

## Testing

Comprehensive test suite:
- 2×2 and 3×3 determinants
- Identity and singular matrices
- Linear system solving (unique, singular, empty)
- Error conditions (non-square, wrong size)
- Edge cases (zero-size matrices)

Run tests:
```bash
cargo test -p matrix
```

## Integration with Other Modules

### arith
Depends on `Q` type for all element storage and arithmetic.

### Future: solver
Could be used for matrix equations, eigenvalue problems.

### Future: polys
Could implement resultants and Sylvester matrices.

## Limitations

### No Floating Point
Only exact rational arithmetic. For numerical work, use external libraries like `nalgebra`.

### No Matrix Operations
Missing:
- Addition/subtraction
- Multiplication
- Inversion
- Rank computation
- LU/QR decomposition
- Eigenvalues/eigenvectors

### Inefficient for Large Systems
Cramer's rule is O(n⁴). Future versions should implement:
- Gaussian elimination with back-substitution
- LU decomposition
- Sparse matrix support

### No Overdetermined/Underdetermined Systems
Only handles square systems. No least-squares or nullspace computation.

## Example: Complete Workflow

```rust
use matrix::MatrixQ;
use arith::Q;

// Define system: 2x + 3y = 8
//                x - y = -1

let A = MatrixQ::from_i64(2, 2, &[
    2, 3,
    1, -1,
]);

let b = vec![Q(8, 1), Q(-1, 1)];

// Check if system is solvable
let det = A.det_bareiss().unwrap();
println!("det(A) = {}/{}", det.0, det.1);

if !det.is_zero() {
    // Solve for x
    let x = A.solve_bareiss(&b).unwrap().unwrap();
    
    println!("Solution:");
    for (i, val) in x.iter().enumerate() {
        println!("  x_{} = {}/{}", i, val.0, val.1);
    }
    // Output: x_0 = 1/1, x_1 = 2/1
} else {
    println!("Singular system");
}
```

## Future Enhancements

- **Matrix arithmetic**: Add, subtract, multiply
- **Inversion**: Compute A⁻¹ via Gauss-Jordan
- **Factorizations**: LU, QR, Cholesky
- **Better solving**: Gaussian elimination instead of Cramer's rule
- **Eigenvalues**: Characteristic polynomial, power iteration
- **Sparse matrices**: CSR/CSC formats for large sparse systems
- **Expression integration**: Matrix elements as symbolic expressions

## References

- Depends on: `arith`
- Classic references:
  - Bareiss algorithm: "Sylvester's identity and multistep integer-preserving Gaussian elimination" (1968)
  - Cramer's rule: Standard linear algebra textbook
  - Matrix computations: Golub & Van Loan
