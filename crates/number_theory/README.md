# number_theory

Computational number theory utilities for Symmetrica.

- Strong probable-prime test for u64 (`is_prime_u64`) using Miller–Rabin bases suitable for 64-bit.
- Modular arithmetic helpers: `mod_inverse`, `crt_pair`, `crt`.
- Experimental factorization: Pollard's rho (Floyd) and Brent variants behind feature flag.

## Features

- Stable (default):
  - `is_prime_u64(n: u64) -> bool`
  - `mod_inverse(a: u64, m: u64) -> Option<u64>`
  - `crt_pair(a1, m1, a2, m2) -> Option<(x, M)>`
  - `crt(&[(a, m)]) -> Option<(x, M)>`
- Experimental:
  - `number_theory_experimental`: exposes `factor(n: u64) -> Vec<u64>` and internal rho variants.

## Examples

```rust
use number_theory::{is_prime_u64, mod_inverse, crt_pair, crt};

assert!(is_prime_u64(1_000_000_007));
assert!(!is_prime_u64(1_000_000_008));

assert_eq!(mod_inverse(3, 10), Some(7));
assert_eq!(mod_inverse(2, 4), None);

let (x, m) = crt_pair(2, 3, 3, 5).unwrap();
assert_eq!(m, 15);
assert_eq!(x % 3, 2);
assert_eq!(x % 5, 3);

let (x2, m2) = crt(&[(2, 3), (3, 5), (2, 7)]).unwrap();
assert_eq!(m2, 105);
assert_eq!(x2 % 3, 2);
assert_eq!(x2 % 5, 3);
assert_eq!(x2 % 7, 2);
```

### Factorization (experimental)

```toml
[dependencies]
number_theory = { version = "*", features = ["number_theory_experimental"] }
```

```rust
use number_theory::factor;
let mut fs = factor(999_983_u64 * 1_000_003);
fs.sort_unstable();
assert_eq!(fs, vec![999_983, 1_000_003]);
```

## License

Dual-licensed under MIT or Apache-2.0.
