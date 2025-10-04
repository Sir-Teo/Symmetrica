#![no_main]

use libfuzzer_sys::fuzz_target;
use expr_core::Store;

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }

    let mut store = Store::new();
    
    // Extract operation type and operands
    let op = data[0] % 5;
    let val1 = i64::from_le_bytes([
        data[1], data.get(2).copied().unwrap_or(0),
        data.get(3).copied().unwrap_or(0), data.get(4).copied().unwrap_or(0),
        data.get(5).copied().unwrap_or(0), data.get(6).copied().unwrap_or(0),
        data.get(7).copied().unwrap_or(0), 0
    ]);
    
    let val2 = if data.len() > 8 {
        i64::from_le_bytes([
            data[8], data.get(9).copied().unwrap_or(0),
            data.get(10).copied().unwrap_or(0), data.get(11).copied().unwrap_or(0),
            data.get(12).copied().unwrap_or(0), data.get(13).copied().unwrap_or(0),
            data.get(14).copied().unwrap_or(0), 0
        ])
    } else {
        1
    };
    
    // Limit values to prevent overflow
    let val1 = val1.clamp(-1000, 1000);
    let val2 = val2.clamp(-1000, 1000);
    
    let expr1 = store.int(val1);
    let expr2 = store.int(val2);
    
    // Test various operations don't crash
    match op {
        0 => {
            let result = store.add(vec![expr1, expr2]);
            let _ = store.to_string(result);
        }
        1 => {
            let result = store.mul(vec![expr1, expr2]);
            let _ = store.to_string(result);
        }
        2 => {
            // Test rational creation
            if val2 != 0 {
                let rat = store.rat(val1, val2);
                let _ = store.to_string(rat);
            }
        }
        3 => {
            // Test power
            if val2.abs() < 100 {
                let result = store.pow(expr1, expr2);
                let _ = store.to_string(result);
            }
        }
        _ => {
            // Test symbols and functions
            let x = store.sym("x");
            let result = store.add(vec![x, expr1]);
            let _ = store.to_string(result);
        }
    }
});
