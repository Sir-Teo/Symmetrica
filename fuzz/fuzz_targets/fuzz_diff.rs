#![no_main]

use libfuzzer_sys::fuzz_target;
use expr_core::Store;
use calculus::diff;
use simplify::simplify;

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }

    let mut store = Store::new();
    let mut idx = 0;
    
    fn build_differentiable_expr(
        store: &mut Store,
        data: &[u8],
        idx: &mut usize,
        depth: u8
    ) -> Option<expr_core::ExprId> {
        if *idx >= data.len() || depth > 8 {
            return None;
        }
        
        let op_type = data[*idx] % 6;
        *idx += 1;
        
        match op_type {
            0 => {
                // Constant
                if *idx >= data.len() {
                    return None;
                }
                let val = (data[*idx] as i64) % 100;
                *idx += 1;
                Some(store.int(val))
            }
            1 => {
                // Variable (x, y, or z)
                if *idx >= data.len() {
                    return None;
                }
                let var = match data[*idx] % 3 {
                    0 => "x",
                    1 => "y",
                    _ => "z",
                };
                *idx += 1;
                Some(store.sym(var))
            }
            2 => {
                // Add
                let left = build_differentiable_expr(store, data, idx, depth + 1)?;
                let right = build_differentiable_expr(store, data, idx, depth + 1)?;
                Some(store.add(vec![left, right]))
            }
            3 => {
                // Mul
                let left = build_differentiable_expr(store, data, idx, depth + 1)?;
                let right = build_differentiable_expr(store, data, idx, depth + 1)?;
                Some(store.mul(vec![left, right]))
            }
            4 => {
                // Pow with small exponent
                let base = build_differentiable_expr(store, data, idx, depth + 1)?;
                if *idx >= data.len() {
                    return None;
                }
                let exp_val = (data[*idx] % 5 + 1) as i64; // 1-5
                *idx += 1;
                let exp = store.int(exp_val);
                Some(store.pow(base, exp))
            }
            _ => {
                // Trig function
                if *idx >= data.len() {
                    return None;
                }
                let func = match data[*idx] % 3 {
                    0 => "sin",
                    1 => "cos",
                    _ => "exp",
                };
                *idx += 1;
                let arg = build_differentiable_expr(store, data, idx, depth + 1)?;
                Some(store.func(func, vec![arg]))
            }
        }
    }
    
    if let Some(expr) = build_differentiable_expr(&mut store, data, &mut idx, 0) {
        // Choose differentiation variable
        let var = match data[0] % 3 {
            0 => "x",
            1 => "y",
            _ => "z",
        };
        
        // Differentiate - should not panic
        let deriv = diff(&mut store, expr, var);
        let simplified = simplify(&mut store, deriv);
        
        // Should produce valid output
        let _ = store.to_string(simplified);
        
        // Property: differentiating twice should also work
        let deriv2 = diff(&mut store, deriv, var);
        let _ = store.to_string(deriv2);
    }
});
