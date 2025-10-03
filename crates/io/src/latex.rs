//! LaTeX printer for Symmetrica expressions.
//! Minimal, deterministic, precedence-aware pretty printer.

use expr_core::{ExprId, Op, Payload, Store};

/// Convert an expression to a LaTeX string.
pub fn to_latex(st: &Store, id: ExprId) -> String {
    fn prec(op: &Op) -> u8 {
        match op {
            Op::Add => 1,
            Op::Mul => 2,
            Op::Pow => 3,
            _ => 4,
        }
    }
    fn needs_paren(child_op: &Op, parent_prec: u8) -> bool {
        prec(child_op) < parent_prec
    }
    fn esc_ident(s: &str) -> String {
        // Minimal escaping for LaTeX: underscore is common in identifiers
        s.replace('_', "\\_")
    }
    fn mul_join(parts: Vec<String>) -> String {
        parts.join(" \\cdot ")
    }

    fn go(st: &Store, id: ExprId, parent_prec: u8) -> String {
        let n = st.get(id);
        let s = match (&n.op, &n.payload) {
            (Op::Integer, Payload::Int(k)) => k.to_string(),
            (Op::Rational, Payload::Rat(a, b)) => format!("\\frac{{{}}}{{{}}}", a, b),
            (Op::Symbol, Payload::Sym(name)) => esc_ident(name),
            (Op::Function, Payload::Func(name)) => {
                let head = match name.as_str() {
                    "sin" => "\\sin",
                    "cos" => "\\cos",
                    "exp" => "\\exp",
                    "ln" => "\\ln",
                    _ => name,
                };
                let args = n.children.iter().map(|c| go(st, *c, 0)).collect::<Vec<_>>().join(", ");
                format!("{}({})", head, args)
            }
            (Op::Add, _) => n
                .children
                .iter()
                .map(|c| go(st, *c, prec(&Op::Add)))
                .collect::<Vec<_>>()
                .join(" + "),
            (Op::Mul, _) => {
                let parts = n
                    .children
                    .iter()
                    .map(|c| {
                        let cn = st.get(*c);
                        // Call child with neutral precedence and parenthesize manually when needed
                        let inner = go(st, *c, 0);
                        if matches!(cn.op, Op::Add) {
                            format!("({})", inner)
                        } else {
                            inner
                        }
                    })
                    .collect::<Vec<_>>();
                mul_join(parts)
            }
            (Op::Pow, _) => {
                // base^{exp}, parenthesize base if needed
                let b_id = n.children[0];
                let e_id = n.children[1];
                let b_node = st.get(b_id);
                // Use neutral precedence and add parentheses manually if required
                let base_s = go(st, b_id, 0);
                let base = if needs_paren(&b_node.op, prec(&Op::Pow)) {
                    format!("({})", base_s)
                } else {
                    base_s
                };
                let exp = go(st, e_id, 0);
                format!("{}^{{{}}}", base, exp)
            }
            _ => "<unknown>".into(),
        };
        if prec(&n.op) < parent_prec {
            format!("({})", s)
        } else {
            s
        }
    }
    go(st, id, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latex_rational_power_mul() {
        let mut st = Store::new();
        let x = st.sym("x");
        let three = st.int(3);
        let x3 = st.pow(x, three);
        let one_over_three = st.rat(1, 3);
        let expr = st.mul(vec![one_over_three, x3]);
        let s = to_latex(&st, expr);
        // Expect a fraction multiplied by x^{3}
        assert!(s.contains("\\frac{1}{3}"));
        assert!(s.contains("x^{3}"));
    }

    #[test]
    fn latex_functions_and_parentheses() {
        let mut st = Store::new();
        let x = st.sym("x");
        let one = st.int(1);
        let xp1 = st.add(vec![x, one]);
        let two = st.int(2);
        let pow = st.pow(xp1, two); // (x+1)^2
        let sin_pow = st.func("sin", vec![pow]);
        let s = to_latex(&st, sin_pow);
        assert!(s.starts_with("\\sin("));
        assert!(s.contains("(x + 1)^{2}") || s.contains("(1 + x)^{2}"));
    }

    #[test]
    fn latex_common_funcs() {
        let mut st = Store::new();
        let x = st.sym("x");
        let sinx = st.func("sin", vec![x]);
        let cosx = st.func("cos", vec![x]);
        let expx = st.func("exp", vec![x]);
        let lnx = st.func("ln", vec![x]);
        assert_eq!(to_latex(&st, sinx), "\\sin(x)");
        assert_eq!(to_latex(&st, cosx), "\\cos(x)");
        assert_eq!(to_latex(&st, expx), "\\exp(x)");
        assert_eq!(to_latex(&st, lnx), "\\ln(x)");
    }

    #[test]
    fn latex_mul_add_parentheses() {
        let mut st = Store::new();
        let x = st.sym("x");
        let one = st.int(1);
        let xp1 = st.add(vec![x, one]);
        let y = st.sym("y");
        let expr = st.mul(vec![xp1, y]);
        let s = to_latex(&st, expr);
        assert!(s.contains("(x + 1) \\cdot y") || s.contains("(1 + x) \\cdot y"));
    }

    #[test]
    fn latex_symbol_underscore_escape() {
        let mut st = Store::new();
        let x1 = st.sym("x_1");
        let s = to_latex(&st, x1);
        assert_eq!(s, "x\\_1");
    }

    #[test]
    fn latex_pow_parentheses_for_mul_base() {
        let mut st = Store::new();
        let two = st.int(2);
        let x = st.sym("x");
        let base = st.mul(vec![two, x]);
        let three = st.int(3);
        let expr = st.pow(base, three);
        let s = to_latex(&st, expr);
        assert!(s.contains("(2 \\cdot x)^{3}"));
    }
}
