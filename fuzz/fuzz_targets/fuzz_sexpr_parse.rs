#![no_main]

use libfuzzer_sys::fuzz_target;
use expr_core::Store;
use io::from_sexpr;

fuzz_target!(|data: &[u8]| {
    // Try to parse the fuzz input as a string
    if let Ok(s) = std::str::from_utf8(data) {
        let mut store = Store::new();
        
        // Attempt to parse - should never panic, just return Err
        let _ = from_sexpr(&mut store, s);
        
        // If it parses successfully, test round-trip
        if let Ok(expr) = from_sexpr(&mut store, s) {
            let sexpr_out = io::to_sexpr(&store, expr);
            
            // Parse it again - should succeed
            let mut store2 = Store::new();
            if let Ok(expr2) = from_sexpr(&mut store2, &sexpr_out) {
                // Verify the expression is structurally equivalent
                let s1 = store.to_string(expr);
                let s2 = store2.to_string(expr2);
                // Note: string representation should match for round-trip
                assert_eq!(s1, s2, "Round-trip parse failed");
            }
        }
    }
});
