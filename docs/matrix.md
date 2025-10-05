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

## Matrix Arithmetic

### Addition

```rust
pub fn add(&self, other: &MatrixQ) -> Result<MatrixQ, &'static str>
```

Adds two matrices element-wise.

**Returns:**
- `Ok(MatrixQ)`: Sum matrix
- `Err`: If dimensions don't match

**Example:**
```rust
let a = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
let b = MatrixQ::from_i64(2, 2, &[5, 6, 7, 8]);
let c = a.add(&b).unwrap();
// Result: [[6, 8], [10, 12]]
```

### Subtraction

```rust
pub fn sub(&self, other: &MatrixQ) -> Result<MatrixQ, &'static str>
```

Subtracts two matrices element-wise.

**Returns:**
- `Ok(MatrixQ)`: Difference matrix
- `Err`: If dimensions don't match

**Example:**
```rust
let a = MatrixQ::from_i64(2, 2, &[5, 6, 7, 8]);
let b = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
let c = a.sub(&b).unwrap();
// Result: [[4, 4], [4, 4]]
```

### Multiplication

```rust
pub fn mul(&self, other: &MatrixQ) -> Result<MatrixQ, &'static str>
```

Multiplies two matrices using standard matrix multiplication.

**Returns:**
- `Ok(MatrixQ)`: Product matrix (m×p) × (p×n) = (m×n)
- `Err`: If dimensions are incompatible (self.cols ≠ other.rows)

**Example:**
```rust
let a = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
let b = MatrixQ::from_i64(2, 2, &[5, 6, 7, 8]);
let c = a.mul(&b).unwrap();
// [[1,2],[3,4]] * [[5,6],[7,8]] = [[19,22],[43,50]]
assert_eq!(c.get(0, 0), Q(19, 1));
assert_eq!(c.get(0, 1), Q(22, 1));
```

**Rectangular matrices:**
```rust
// (2×3) * (3×2) = (2×2)
let a = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
let b = MatrixQ::from_i64(3, 2, &[1, 2, 3, 4, 5, 6]);
let c = a.mul(&b).unwrap();
assert_eq!(c.rows, 2);
assert_eq!(c.cols, 2);
```

### Transpose

```rust
pub fn transpose(&self) -> MatrixQ
```

Transposes the matrix by swapping rows and columns. Returns a new matrix where `result[i,j] = self[j,i]`.

**Properties:**
- For an m×n matrix: transpose is n×m
- (A^T)^T = A (double transpose is identity)
- (A + B)^T = A^T + B^T (distributes over addition)
- (AB)^T = B^T A^T (reverses multiplication order)
- det(A^T) = det(A) for square matrices

**Examples:**

**Square matrix:**
```rust
// [[1, 2], [3, 4]]^T = [[1, 3], [2, 4]]
let m = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
let mt = m.transpose();
assert_eq!(mt.get(0, 0), Q(1, 1));
assert_eq!(mt.get(0, 1), Q(3, 1));
assert_eq!(mt.get(1, 0), Q(2, 1));
assert_eq!(mt.get(1, 1), Q(4, 1));
```

**Rectangular matrix:**
```rust
// [[1, 2, 3], [4, 5, 6]]^T = [[1, 4], [2, 5], [3, 6]]
let m = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
let mt = m.transpose();
assert_eq!(mt.rows, 3);
assert_eq!(mt.cols, 2);
```

**Row vector → Column vector:**
```rust
// [1, 2, 3]^T = [[1], [2], [3]]
let row = MatrixQ::from_i64(1, 3, &[1, 2, 3]);
let col = row.transpose();
assert_eq!(col.rows, 3);
assert_eq!(col.cols, 1);
```

**Symmetric matrix:**
```rust
// Symmetric matrices satisfy A = A^T
let m = MatrixQ::from_i64(2, 2, &[1, 2, 2, 3]);
assert_eq!(m, m.transpose());
```

**Properties verification:**
```rust
// (A^T)^T = A
let m = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
assert_eq!(m, m.transpose().transpose());

// det(A^T) = det(A)
let m = MatrixQ::from_i64(3, 3, &[2, 0, 1, 1, 1, 0, 0, 3, 1]);
assert_eq!(m.det_bareiss().unwrap(), m.transpose().det_bareiss().unwrap());

// (AB)^T = B^T A^T
let a = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
let b = MatrixQ::from_i64(3, 2, &[1, 2, 3, 4, 5, 6]);
let ab_t = a.mul(&b).unwrap().transpose();
let bt_at = b.transpose().mul(&a.transpose()).unwrap();
assert_eq!(ab_t, bt_at);
```

### Scalar Multiplication

```rust
pub fn scalar_mul(&self, scalar: Q) -> MatrixQ
```

Multiplies every element of the matrix by a scalar (rational number).

**Properties:**
- c(A + B) = cA + cB (distributive over addition)
- (ab)M = a(bM) (associative with scalar multiplication)
- 1·A = A (identity)
- 0·A = 0 (zero matrix)

**Examples:**

**Basic scalar multiplication:**
```rust
let m = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
let result = m.scalar_mul(Q(3, 1));
// [[3, 6], [9, 12]]
```

**Scalar with rational number:**
```rust
let m = MatrixQ::from_i64(2, 2, &[2, 4, 6, 8]);
let half = m.scalar_mul(Q(1, 2));
// [[1, 2], [3, 4]]
```

**Negating a matrix:**
```rust
let m = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
let neg_m = m.scalar_mul(Q(-1, 1));
// [[-1, -2], [-3, -4]]
```

**Distributive property:**
```rust
// c(A + B) = cA + cB
let a = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
let b = MatrixQ::from_i64(2, 2, &[5, 6, 7, 8]);
let c = Q(3, 1);
let left = a.add(&b).unwrap().scalar_mul(c);
let right = a.scalar_mul(c).add(&b.scalar_mul(c)).unwrap();
assert_eq!(left, right);
```

### Trace

```rust
pub fn trace(&self) -> Result<Q, &'static str>
```

Computes the trace (sum of diagonal elements) of a square matrix.

**Properties:**
- tr(A + B) = tr(A) + tr(B) (additive)
- tr(cA) = c·tr(A) (scalar multiplication)
- tr(A^T) = tr(A) (transpose invariant)
- tr(AB) = tr(BA) (cyclic property)
- For identity matrix I_n: tr(I) = n

**Examples:**

**Basic trace:**
```rust
// [[1, 2], [3, 4]] has trace = 1 + 4 = 5
let m = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
assert_eq!(m.trace().unwrap(), Q(5, 1));
```

**Identity matrix:**
```rust
let m = MatrixQ::identity(5);
assert_eq!(m.trace().unwrap(), Q(5, 1));
```

**With rational entries:**
```rust
// [[1/2, 1/3], [1/4, 1/5]] has trace = 1/2 + 1/5 = 7/10
let m = MatrixQ::new(2, 2, vec![Q(1, 2), Q(1, 3), Q(1, 4), Q(1, 5)]);
assert_eq!(m.trace().unwrap(), Q(7, 10));
```

**Additive property:**
```rust
// tr(A + B) = tr(A) + tr(B)
let a = MatrixQ::from_i64(3, 3, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
let b = MatrixQ::from_i64(3, 3, &[9, 8, 7, 6, 5, 4, 3, 2, 1]);
let tr_sum = a.add(&b).unwrap().trace().unwrap();
let sum_tr = add_q(a.trace().unwrap(), b.trace().unwrap());
assert_eq!(tr_sum, sum_tr);
```

**Cyclic property:**
```rust
// tr(AB) = tr(BA)
let a = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
let b = MatrixQ::from_i64(3, 2, &[1, 2, 3, 4, 5, 6]);
let ab = a.mul(&b).unwrap(); // 2×2
let ba = b.mul(&a).unwrap(); // 3×3
assert_eq!(ab.trace().unwrap(), ba.trace().unwrap());
```

**Non-square matrix error:**
```rust
let m = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
assert!(m.trace().is_err());
```

## Matrix Inversion

```rust
pub fn inverse(&self) -> Result<Option<MatrixQ>, &'static str>
```

Computes the inverse of a square matrix using **Gauss-Jordan elimination**.

**Algorithm:**
1. Check if matrix is singular (det = 0)
2. Create augmented matrix [A | I]
3. Row reduce to [I | A⁻¹]
4. Extract inverse from right half

**Returns:**
- `Ok(Some(A⁻¹))`: Inverse matrix
- `Ok(None)`: Singular matrix (not invertible)
- `Err`: If matrix is not square

**Example (2×2):**
```rust
// [[1,2],[3,4]] has inverse [[-2,1],[3/2,-1/2]]
let a = MatrixQ::from_i64(2, 2, &[1, 2, 3, 4]);
let inv = a.inverse().unwrap().expect("invertible");

assert_eq!(inv.get(0, 0), Q(-2, 1));
assert_eq!(inv.get(0, 1), Q(1, 1));
assert_eq!(inv.get(1, 0), Q(3, 2));
assert_eq!(inv.get(1, 1), Q(-1, 2));

// Verify A * A⁻¹ = I
let product = a.mul(&inv).unwrap();
let identity = MatrixQ::identity(2);
assert_eq!(product, identity);
```

**Singular matrix:**
```rust
let a = MatrixQ::from_i64(2, 2, &[1, 2, 2, 4]);
let result = a.inverse().unwrap();
assert!(result.is_none());  // Not invertible
```

## Matrix Rank

```rust
pub fn rank(&self) -> usize
```

Computes the rank of the matrix (number of linearly independent rows/columns) using row reduction to row echelon form.

**Algorithm:**
1. Perform row reduction with partial pivoting
2. Count number of non-zero rows in row echelon form
3. Return the count (equals number of pivot positions)

**Properties:**
- For an m×n matrix: `rank ≤ min(m, n)`
- Full rank square matrix: `rank = n` and det ≠ 0
- Rank-deficient square matrix: `rank < n` and det = 0
- Rank is invariant under row operations

**Examples:**

**Full rank square matrix:**
```rust
let m = MatrixQ::identity(5);
assert_eq!(m.rank(), 5);

let m2 = MatrixQ::from_i64(3, 3, &[2, 0, 1, 1, 1, 0, 0, 3, 1]);
assert_eq!(m2.rank(), 3);  // Full rank
```

**Rank-deficient matrix:**
```rust
// Singular matrix: second row = 2 × first row
let m = MatrixQ::from_i64(2, 2, &[1, 2, 2, 4]);
assert_eq!(m.rank(), 1);
```

**Rectangular matrices:**
```rust
// 2×3 matrix with rank 2 (full row rank)
let m = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
assert_eq!(m.rank(), 2);

// 3×2 matrix with rank 2 (full column rank)
let m = MatrixQ::from_i64(3, 2, &[1, 0, 0, 1, 0, 0]);
assert_eq!(m.rank(), 2);
```

**Rank-1 matrix:**
```rust
// All rows are multiples of the first row
let m = MatrixQ::from_i64(3, 3, &[1, 2, 3, 2, 4, 6, 3, 6, 9]);
assert_eq!(m.rank(), 1);
```

**Zero matrix:**
```rust
let m = MatrixQ::from_i64(3, 3, &[0, 0, 0, 0, 0, 0, 0, 0, 0]);
assert_eq!(m.rank(), 0);
```

**Relationship with determinant:**
```rust
// For square matrices:
// - rank = n  ⟺  det ≠ 0  (invertible)
// - rank < n  ⟺  det = 0  (singular)

let m = MatrixQ::from_i64(3, 3, &[2, 0, 1, 1, 1, 0, 0, 3, 1]);
assert_eq!(m.rank(), 3);
assert_ne!(m.det_bareiss().unwrap(), Q(0, 1));
```

## Nullspace (Kernel)

```rust
pub fn nullspace(&self) -> Vec<Vec<Q>>
```

Computes a basis for the nullspace (kernel) of the matrix. The nullspace is the set of all vectors **x** such that **Ax = 0**.

**Algorithm:**
1. Reduce matrix to reduced row echelon form (RREF)
2. Identify free variables (columns without pivots)
3. For each free variable, construct a basis vector by setting it to 1 and back-substituting
4. Return the collection of basis vectors

**Properties:**
- For an m×n matrix: nullspace dimension = n - rank (**Rank-Nullity Theorem**)
- Full rank matrix (rank = n): trivial nullspace (empty basis)
- Zero matrix: nullspace is entire ℚⁿ (n basis vectors)
- All returned vectors satisfy Ax = 0

**Examples:**

**Trivial nullspace (full rank):**
```rust
let m = MatrixQ::identity(3);
let null = m.nullspace();
assert_eq!(null.len(), 0);  // No non-zero vectors in nullspace
```

**Rank-deficient matrix:**
```rust
// [[1, 2], [2, 4]] - second row = 2 × first row
// Nullspace is span{[-2, 1]^T}
let m = MatrixQ::from_i64(2, 2, &[1, 2, 2, 4]);
let null = m.nullspace();
assert_eq!(null.len(), 1);  // 1-dimensional nullspace

// Verify: Ax = 0
// [1, 2] · [-2, 1] = 0 ✓
// [2, 4] · [-2, 1] = 0 ✓
```

**Wide matrix (more columns than rows):**
```rust
// 2×3 matrix [[1, 2, 3], [4, 5, 6]]
// rank = 2, so nullspace dimension = 3 - 2 = 1
let m = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
let null = m.nullspace();
assert_eq!(null.len(), 1);
```

**Zero matrix:**
```rust
// Entire space is the nullspace
let m = MatrixQ::from_i64(2, 3, &[0, 0, 0, 0, 0, 0]);
let null = m.nullspace();
assert_eq!(null.len(), 3);  // Full dimensional nullspace
```

**Rank-Nullity Theorem verification:**
```rust
let m = MatrixQ::from_i64(3, 5, &[1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 0, 0, 1, 2, 3]);
let rank = m.rank();
let nullity = m.nullspace().len();
assert_eq!(rank + nullity, 5);  // rank + nullity = # columns
```

**Homogeneous system solving:**
```rust
// Find all solutions to Ax = 0
let m = MatrixQ::from_i64(2, 3, &[1, 2, 1, 2, 4, 2]);
let null_basis = m.nullspace();

// Any linear combination of basis vectors is a solution
// x = c₁·v₁ + c₂·v₂ for any c₁, c₂ ∈ ℚ
```

**Properties of basis vectors:**
- All vectors are non-zero
- Vectors are linearly independent
- Every vector in nullspace is a linear combination of basis vectors
- Number of basis vectors = nullity = n - rank

## Column Space (Range)

```rust
pub fn columnspace(&self) -> Vec<Vec<Q>>
```

Computes a basis for the column space (range) of the matrix. The column space is the span of the columns of **A**.

**Algorithm:**
1. Reduce matrix to row echelon form (REF) to identify pivot columns
2. Extract the pivot columns from the ORIGINAL matrix (not the reduced form)
3. Return the collection of basis vectors

**Properties:**
- For an m×n matrix: column space dimension = rank (**dimension theorem**)
- Column space is a subspace of ℚᵐ (vectors have m components)
- The basis consists of actual columns from the original matrix
- Column space is orthogonal to the left nullspace (nullspace of Aᵀ)

**Examples:**

**Full rank square matrix:**
```rust
// Identity matrix: all columns are independent
let m = MatrixQ::identity(3);
let cols = m.columnspace();
assert_eq!(cols.len(), 3);  // All columns form basis

// Returns the standard basis vectors
assert_eq!(cols[0], vec![Q(1, 1), Q(0, 1), Q(0, 1)]);
assert_eq!(cols[1], vec![Q(0, 1), Q(1, 1), Q(0, 1)]);
assert_eq!(cols[2], vec![Q(0, 1), Q(0, 1), Q(1, 1)]);
```

**Rank-deficient matrix:**
```rust
// [[1, 2, 3], [2, 4, 6]] - third column = 3 × first column
let m = MatrixQ::from_i64(2, 3, &[1, 2, 3, 2, 4, 6]);
let cols = m.columnspace();
assert_eq!(cols.len(), 2);  // Only first two columns are independent

// Basis vectors are from the original matrix
assert_eq!(cols[0], vec![Q(1, 1), Q(2, 1)]);
assert_eq!(cols[1], vec![Q(2, 1), Q(4, 1)]);
```

**Tall matrix:**
```rust
// 3×2 matrix with full column rank
let m = MatrixQ::from_i64(3, 2, &[1, 0, 0, 1, 1, 1]);
let cols = m.columnspace();
assert_eq!(cols.len(), 2);  // Both columns are independent
```

**Zero matrix:**
```rust
// Zero matrix has trivial column space
let m = MatrixQ::from_i64(3, 3, &[0, 0, 0, 0, 0, 0, 0, 0, 0]);
let cols = m.columnspace();
assert_eq!(cols.len(), 0);
```

**Dimension equals rank:**
```rust
// Column space dimension always equals rank
let m = MatrixQ::from_i64(3, 5, &[1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 0, 0, 1, 2, 3]);
let rank = m.rank();
let colspace = m.columnspace();
assert_eq!(colspace.len(), rank);
```

**Relationship with linear systems:**
```rust
// Vector b is in column space ⟺ Ax = b has a solution
let m = MatrixQ::from_i64(2, 3, &[1, 2, 3, 4, 5, 6]);
let cols = m.columnspace();

// The column space is 2-dimensional (spans ℚ²)
// Any vector in ℚ² can be expressed as a linear combination of basis vectors
```

**Properties of basis vectors:**
- All vectors are from the original matrix columns
- Vectors are linearly independent
- Every column of A can be expressed as a linear combination of basis vectors
- Number of basis vectors = rank = dimension of column space

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
- **add/sub**: O(n²) for n×n matrix (element-wise operations)
- **mul**: O(n³) for n×n matrices (standard algorithm)
- **transpose**: O(n²) for n×n matrix (element copying)
- **scalar_mul**: O(n²) for n×n matrix (element-wise multiplication)
- **trace**: O(n) for n×n matrix (sum diagonal elements)
- **det_bareiss**: O(n³) for n×n matrix (Gaussian elimination)
- **inverse**: O(n³) for n×n matrix (Gauss-Jordan elimination)
- **rank**: O(min(m,n)²·max(m,n)) for m×n matrix (row reduction)
- **nullspace**: O(min(m,n)²·max(m,n)) for m×n matrix (RREF)
- **solve_bareiss**: O(n⁴) due to Cramer's rule (n determinants of size n)

### Space Complexity
- **add/sub**: O(n²) for result matrix
- **mul**: O(n²) for result matrix
- **transpose**: O(n²) for transposed matrix
- **scalar_mul**: O(n²) for result matrix
- **trace**: O(1) (single rational number)
- **det_bareiss**: O(n²) for matrix copy during elimination
- **inverse**: O(2n²) for augmented matrix
- **rank**: O(mn) for matrix copy during row reduction
- **nullspace**: O(mn) for RREF computation + O(n·nullity) for basis vectors
- **solve_bareiss**: O(n²) for temporary matrices

### Practical Limits
- **add/sub/mul**: Efficient for any reasonable size
- **det/inverse/rank**: Efficient for n ≤ 100
- **solve**: Usable for n ≤ 20 (Cramer's rule is slow)
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

Comprehensive test suite (108 unit tests):
- **Determinant**: 2×2, 3×3, identity, singular matrices
- **Solving**: Unique solutions, singular systems, empty systems
- **Addition**: Element-wise, dimension mismatch, fractions
- **Subtraction**: Element-wise, self-subtraction to zero
- **Multiplication**: 2×2, identity, rectangular matrices, dimension errors
- **Transpose**: Square, rectangular, symmetric, involution property, algebraic properties
- **Scalar multiplication**: Basic, zero, one, negative, rational, distributive, associative properties
- **Trace**: Basic, identity, rational entries, additive, scalar multiplication, transpose, cyclic properties
- **Inverse**: 2×2, 3×3, singular matrices, verification via A×A⁻¹=I
- **Rank**: Full rank, rank-deficient, zero matrix, rectangular matrices, rank-nullity theorem
- **Nullspace**: Trivial/non-trivial cases, rank-nullity theorem, verification via Ax=0
- **Column space**: Full rank, rank-deficient, dimension equals rank, orthogonality properties
- **Error conditions**: Non-square matrices, dimension mismatches
- **Edge cases**: Zero-size matrices

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

### Missing Advanced Operations
Still not implemented:
- LU/QR/Cholesky decomposition
- Eigenvalues/eigenvectors
- Singular Value Decomposition (SVD)

### Inefficient for Large Systems
- Cramer's rule for solving is O(n⁴)
- Gauss-Jordan for inverse is O(n³)
- Future versions should implement:
  - Gaussian elimination with back-substitution
  - LU decomposition (more efficient for repeated solves)
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

- ✅ ~~Matrix arithmetic: Add, subtract, multiply~~ (Implemented)
- ✅ ~~Inversion: Compute A⁻¹ via Gauss-Jordan~~ (Implemented)
- ✅ ~~Rank computation: Row reduction to determine rank~~ (Implemented)
- ✅ ~~Nullspace and column space: Basis computation~~ (Implemented)
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
