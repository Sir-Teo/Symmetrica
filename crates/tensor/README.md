# tensor

Tensor algebra for Symmetrica: arbitrary-rank tensors with shape, strides, and operations.

## Features

- **Construction:** `new()`, `from_vec()`, `reshape()`, `permute_axes()`
- **Indexing:** `get()`, `set()` with multi-dimensional indices
- **Operations:**
  - `outer()` - outer (tensor) product
  - `elem_add()`, `elem_mul()` - elementwise operations
  - `contract()` - tensordot along specified axes
  - `sum_axis()` - reduction along axis
  - `dot()` - vector dot product
  - `transpose2()` - matrix transpose
  - `matmul()` - matrix multiplication
  - `trace_pair()` - generalized trace along two axes

## Examples

### Basic tensor creation and indexing

```rust
use tensor::Tensor;

let mut t = Tensor::new(vec![2, 3], 0i64);
t.set(&[0, 0], 1);
t.set(&[1, 2], 7);
assert_eq!(*t.get(&[0, 0]), 1);
assert_eq!(*t.get(&[1, 2]), 7);
```

### Matrix operations

```rust
use tensor::Tensor;

// Matrix multiplication via matmul
let a = Tensor::from_vec(vec![2, 3], vec![1i64, 2, 3, 4, 5, 6]);
let b = Tensor::from_vec(vec![3, 2], vec![7i64, 8, 9, 10, 11, 12]);
let c = a.matmul(&b);
assert_eq!(c.shape(), &[2, 2]);

// Transpose
let a_t = a.transpose2();
assert_eq!(a_t.shape(), &[3, 2]);

// Trace
let m = Tensor::from_vec(vec![2, 2], vec![1i64, 2, 3, 4]);
let tr = m.trace_pair(0, 1);
assert_eq!(*tr.get(&[0]), 5); // 1 + 4
```

### Outer product and contraction

```rust
use tensor::Tensor;

let a = Tensor::from_vec(vec![2], vec![1i64, 2]);
let b = Tensor::from_vec(vec![3], vec![3i64, 4, 5]);
let outer = a.outer(&b);
assert_eq!(outer.shape(), &[2, 3]);

// Contract along axes (generalized tensordot)
let result = outer.contract(&b, 1, 0);
```

### Elementwise operations and reduction

```rust
use tensor::Tensor;

let a = Tensor::from_vec(vec![2, 2], vec![1i64, 2, 3, 4]);
let b = Tensor::from_vec(vec![2, 2], vec![5i64, 6, 7, 8]);

// Elementwise add
let sum = a.elem_add(&b);
assert_eq!(sum, Tensor::from_vec(vec![2, 2], vec![6, 8, 10, 12]));

// Sum along axis
let sum0 = a.sum_axis(0);
assert_eq!(sum0, Tensor::from_vec(vec![2], vec![4, 6]));
```

## Notes

- Tensors must have rank ≥ 1 (no rank-0 scalars; use shape `[1]` for scalars)
- Operations assume matching shapes or dimensions where required
- Generic over `T` with appropriate trait bounds (Clone, Add, Mul, Default)

## License

Dual-licensed under MIT or Apache-2.0.
