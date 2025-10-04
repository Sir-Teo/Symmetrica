#![no_main]

use libfuzzer_sys::fuzz_target;
use expr_core::Store;
use simplify::simplify;

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }

    let mut store = Store::new();
    
    // Build a random expression tree from the fuzz input
    let mut idx = 0;
    
    fn build_expr(store: &mut Store, data: &[u8], idx: &mut usize, depth: u8) -> Option<expr_core::ExprId> {
        if *idx >= data.len() || depth > 10 {
            return None;
        }
        
        let op_type = data[*idx] % 8;
        *idx += 1;
        
        match op_type {
            0 => {
                // Integer
                if *idx >= data.len() {
                    return None;
                }
                let val = data[*idx] as i64;
                *idx += 1;
                Some(store.int(val))
            }
            1 => {
                // Symbol
                if *idx >= data.len() {
                    return None;
                }
                let sym_idx = data[*idx] % 4;
                *idx += 1;
                let name = match sym_idx {
                    0 => "x",
                    1 => "y",
                    2 => "z",
                    _ => "w",
                };
                Some(store.sym(name))
            }
            2 => {
                // Add
                let left = build_expr(store, data, idx, depth + 1)?;
                let right = build_expr(store, data, idx, depth + 1)?;
                Some(store.add(vec![left, right]))
            }
            3 => {
                // Mul
                let left = build_expr(store, data, idx, depth + 1)?;
                let right = build_expr(store, data, idx, depth + 1)?;
                Some(store.mul(vec![left, right]))
            }
            4 => {
                // Pow
                let base = build_expr(store, data, idx, depth + 1)?;
                if *idx >= data.len() {
                    return None;
                }
                let exp_val = (data[*idx] % 5) as i64;
                *idx += 1;
                let exp = store.int(exp_val);
                Some(store.pow(base, exp))
            }
            5 => {
                // Rational
                if *idx + 1 >= data.len() {
                    return None;
                }
                let num = data[*idx] as i64;
                let den = (data[*idx + 1] % 10 + 1) as i64; // Avoid zero
                *idx += 2;
                Some(store.rat(num, den))
            }
            6 => {
                // Function (sin, cos, exp, ln)
                if *idx >= data.len() {
                    return None;
                }
                let func_idx = data[*idx] % 4;
                *idx += 1;
                let arg = build_expr(store, data, idx, depth + 1)?;
                let fname = match func_idx {
                    0 => "sin",
                    1 => "cos",
                    2 => "exp",
                    _ => "ln",
                };
                Some(store.func(fname, vec![arg]))
            }
            _ => {
                // Nested addition
                let a = build_expr(store, data, idx, depth + 1)?;
                let b = build_expr(store, data, idx, depth + 1)?;
                let c = build_expr(store, data, idx, depth + 1)?;
                Some(store.add(vec![a, b, c]))
            }
        }
    }
    
    if let Some(expr) = build_expr(&mut store, data, &mut idx, 0) {
        // Test simplify doesn't crash or panic
        let simplified = simplify(&mut store, expr);
        
        // Property: simplify should be idempotent
        let simplified2 = simplify(&mut store, simplified);
        
        // Verify both produce valid strings (no panics)
        let _ = store.to_string(simplified);
        let _ = store.to_string(simplified2);
        
        // Property: digests should match for idempotent simplify
        assert_eq!(
            store.get(simplified).digest,
            store.get(simplified2).digest,
            "Simplify is not idempotent"
        );
    }
});
