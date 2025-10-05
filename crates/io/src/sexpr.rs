//! S-expression serializer and parser for Symmetrica expressions.
//! Formats:
//! - Atoms: (Int k), (Rat n d), (Sym name)
//! - Composite: (+ e1 e2 ...), (* e1 e2 ...), (^ base exp), (Fn name arg1 arg2 ...)
//!
//! Parser is minimal and conservative; it expects the above structured forms.
//! Names in (Sym name) and (Fn name ...) accept bare tokens without spaces/parentheses; use
//! double quotes to include spaces or special characters.

use expr_core::{ExprId, Op, Payload, Store};

/// Convert an expression to an S-expression string.
pub fn to_sexpr(st: &Store, id: ExprId) -> String {
    fn esc_name(s: &str) -> String {
        if s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            s.to_string()
        } else {
            let escaped = s.replace('"', "\\\"");
            format!("\"{}\"", escaped)
        }
    }
    fn go(st: &Store, id: ExprId) -> String {
        let n = st.get(id);
        match (&n.op, &n.payload) {
            (Op::Integer, Payload::Int(k)) => format!("(Int {k})"),
            (Op::Rational, Payload::Rat(a, b)) => format!("(Rat {a} {b})"),
            (Op::Symbol, Payload::Sym(name)) => format!("(Sym {})", esc_name(name)),
            (Op::Function, Payload::Func(name)) => {
                let args = n.children.iter().map(|c| go(st, *c)).collect::<Vec<_>>().join(" ");
                format!("(Fn {} {})", esc_name(name), args)
            }
            (Op::Add, _) => {
                let parts = n.children.iter().map(|c| go(st, *c)).collect::<Vec<_>>().join(" ");
                format!("(+ {})", parts)
            }
            (Op::Mul, _) => {
                let parts = n.children.iter().map(|c| go(st, *c)).collect::<Vec<_>>().join(" ");
                format!("(* {})", parts)
            }
            (Op::Pow, _) => {
                let b = go(st, n.children[0]);
                let e = go(st, n.children[1]);
                format!("(^ {} {})", b, e)
            }
            _ => "(Unknown)".into(),
        }
    }
    go(st, id)
}

/// Parse an S-expression string into an expression in the provided Store.
pub fn from_sexpr(st: &mut Store, input: &str) -> Result<ExprId, String> {
    #[derive(Debug, Clone)]
    enum Tok {
        LParen,
        RParen,
        Str(String),
        Sym(String),
        Int(i64),
    }
    struct Lexer<'a> {
        s: &'a [u8],
        i: usize,
    }
    impl<'a> Lexer<'a> {
        fn new(s: &'a str) -> Self {
            Self { s: s.as_bytes(), i: 0 }
        }
        fn peek(&self) -> Option<u8> {
            self.s.get(self.i).copied()
        }
        fn bump(&mut self) {
            self.i += 1;
        }
        fn skip_ws(&mut self) {
            while let Some(c) = self.peek() {
                if c.is_ascii_whitespace() {
                    self.bump();
                } else {
                    break;
                }
            }
        }
        fn read_while<F: Fn(u8) -> bool>(&mut self, f: F) -> String {
            let start = self.i;
            while let Some(c) = self.peek() {
                if f(c) {
                    self.bump();
                } else {
                    break;
                }
            }
            String::from_utf8(self.s[start..self.i].to_vec()).unwrap()
        }
        fn read_string(&mut self) -> Result<String, String> {
            // assumes current is '"'
            self.bump();
            let mut out = String::new();
            while let Some(c) = self.peek() {
                self.bump();
                match c {
                    b'\\' => {
                        if let Some(nc) = self.peek() {
                            self.bump();
                            out.push(nc as char);
                        } else {
                            return Err("unterminated escape".into());
                        }
                    }
                    b'"' => return Ok(out),
                    _ => out.push(c as char),
                }
            }
            Err("unterminated string".into())
        }
        fn next_tok(&mut self) -> Result<Option<Tok>, String> {
            self.skip_ws();
            match self.peek() {
                None => Ok(None),
                Some(b'(') => {
                    self.bump();
                    Ok(Some(Tok::LParen))
                }
                Some(b')') => {
                    self.bump();
                    Ok(Some(Tok::RParen))
                }
                Some(b'"') => Ok(Some(Tok::Str(self.read_string()?))),
                Some(c) if c == b'-' || c.is_ascii_digit() => {
                    let s = self.read_while(|ch| ch == b'-' || ch.is_ascii_digit());
                    let k: i64 = s.parse().map_err(|_| format!("invalid int: {s}"))?;
                    Ok(Some(Tok::Int(k)))
                }
                Some(_) => {
                    let s =
                        self.read_while(|ch| !ch.is_ascii_whitespace() && ch != b'(' && ch != b')');
                    Ok(Some(Tok::Sym(s)))
                }
            }
        }
        fn all(mut self) -> Result<Vec<Tok>, String> {
            let mut v = Vec::new();
            while let Some(t) = self.next_tok()? {
                v.push(t);
            }
            Ok(v)
        }
    }

    #[derive(Clone)]
    struct Cursor {
        toks: Vec<Tok>,
        i: usize,
    }
    impl Cursor {
        fn new(toks: Vec<Tok>) -> Self {
            Self { toks, i: 0 }
        }
        fn peek(&self) -> Option<&Tok> {
            self.toks.get(self.i)
        }
        fn bump(&mut self) {
            self.i += 1;
        }
        fn expect_sym(&mut self) -> Result<String, String> {
            match self.peek() {
                Some(Tok::Sym(s)) => {
                    let out = s.clone();
                    self.bump();
                    Ok(out)
                }
                Some(Tok::Str(s)) => {
                    let out = s.clone();
                    self.bump();
                    Ok(out)
                }
                _ => Err("expected symbol or string".into()),
            }
        }
        fn expect_int(&mut self) -> Result<i64, String> {
            match self.peek() {
                Some(Tok::Int(k)) => {
                    let v = *k;
                    self.bump();
                    Ok(v)
                }
                _ => Err("expected integer".into()),
            }
        }
        fn expect(&mut self, want: &Tok) -> Result<(), String> {
            match (self.peek(), want) {
                (Some(Tok::LParen), Tok::LParen) => {
                    self.bump();
                    Ok(())
                }
                (Some(Tok::RParen), Tok::RParen) => {
                    self.bump();
                    Ok(())
                }
                _ => Err("unexpected token".into()),
            }
        }
    }

    fn parse_list(st: &mut Store, cur: &mut Cursor) -> Result<ExprId, String> {
        cur.expect(&Tok::LParen)?;
        // head
        let head = cur.expect_sym()?;
        let out = match head.as_str() {
            "+" => {
                let mut terms: Vec<ExprId> = Vec::new();
                while !matches!(cur.peek(), Some(Tok::RParen)) {
                    terms.push(parse_any(st, cur)?);
                }
                st.add(terms)
            }
            "*" => {
                let mut facs: Vec<ExprId> = Vec::new();
                while !matches!(cur.peek(), Some(Tok::RParen)) {
                    facs.push(parse_any(st, cur)?);
                }
                st.mul(facs)
            }
            "^" => {
                let b = parse_any(st, cur)?;
                let e = parse_any(st, cur)?;
                st.pow(b, e)
            }
            "Int" => {
                let k = cur.expect_int()?;
                st.int(k)
            }
            "Rat" => {
                let n = cur.expect_int()?;
                let d = cur.expect_int()?;
                st.rat(n, d)
            }
            "Sym" => {
                let name = cur.expect_sym()?;
                st.sym(name)
            }
            "Fn" => {
                let name = cur.expect_sym()?;
                let mut args: Vec<ExprId> = Vec::new();
                while !matches!(cur.peek(), Some(Tok::RParen)) {
                    args.push(parse_any(st, cur)?);
                }
                st.func(name, args)
            }
            _ => return Err(format!("unknown head: {head}")),
        };
        cur.expect(&Tok::RParen)?;
        Ok(out)
    }

    fn parse_any(st: &mut Store, cur: &mut Cursor) -> Result<ExprId, String> {
        match cur.peek() {
            Some(Tok::LParen) => parse_list(st, cur),
            Some(Tok::Int(k)) => {
                let v = *k;
                cur.bump();
                Ok(st.int(v))
            }
            Some(Tok::Sym(s)) => {
                // bare symbol token: interpret as (Sym s)
                let name = s.clone();
                cur.bump();
                Ok(st.sym(name))
            }
            Some(Tok::Str(s)) => {
                let name = s.clone();
                cur.bump();
                Ok(st.sym(name))
            }
            _ => Err("unexpected token while parsing".into()),
        }
    }

    let toks = Lexer::new(input).all()?;
    let mut cur = Cursor::new(toks);
    let id = parse_any(st, &mut cur)?;
    if cur.peek().is_some() {
        return Err("trailing tokens".into());
    }
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sexpr_roundtrip_basic() {
        let mut st = Store::new();
        let x = st.sym("x");
        let one = st.int(1);
        let xp1 = st.add(vec![x, one]);
        let two = st.int(2);
        let pow = st.pow(xp1, two);
        let three = st.int(3);
        let mul = st.mul(vec![three, xp1]);
        let expr = st.add(vec![pow, mul]);
        let s = to_sexpr(&st, expr);
        let mut st2 = Store::new();
        let parsed = from_sexpr(&mut st2, &s).expect("parse");
        // Compare printed forms via core printer to avoid dependency on term order
        assert_eq!(st.to_string(expr), st2.to_string(parsed));
    }

    #[test]
    fn sexpr_parse_symbols_and_funcs() {
        let mut st = Store::new();
        let sx = from_sexpr(&mut st, "(Sym x_1)").unwrap();
        assert_eq!(st.to_string(sx), "x_1");
        let f = from_sexpr(&mut st, "(Fn sin (Sym x))").unwrap();
        assert_eq!(st.to_string(f), "sin(x)");
    }

    #[test]
    fn sexpr_roundtrip_mul_pow_func() {
        let mut st = Store::new();
        // (* (Rat 3 2) (^ (Sym x) (Int 3)) (Fn sin (Sym x)))
        let s = "(* (Rat 3 2) (^ (Sym x) (Int 3)) (Fn sin (Sym x)))";
        let id = from_sexpr(&mut st, s).expect("parse");
        let out = to_sexpr(&st, id);
        let mut st2 = Store::new();
        let id2 = from_sexpr(&mut st2, &out).expect("parse2");
        assert_eq!(st.to_string(id), st2.to_string(id2));
    }

    #[test]
    fn sexpr_parse_errors() {
        let mut st = Store::new();
        // Unmatched paren
        assert!(from_sexpr(&mut st, "(+ (Int 1)").is_err());
        // Trailing tokens
        assert!(from_sexpr(&mut st, "(Int 5) extra").is_err());
        // Expected symbol
        assert!(from_sexpr(&mut st, "(Sym)").is_err());
        // Expected integer
        assert!(from_sexpr(&mut st, "(Int)").is_err());
        // Invalid quoted string (unclosed)
        assert!(from_sexpr(&mut st, "(Sym \"unclosed").is_err());
        // Unknown head
        assert!(from_sexpr(&mut st, "(Unknown 1)").is_err());
    }

    #[test]
    fn sexpr_nested_add_mul() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let two = st.int(2);
        let prod = st.mul(vec![two, x]);
        let sum = st.add(vec![prod, y]);
        let s = to_sexpr(&st, sum);
        let mut st2 = Store::new();
        let parsed = from_sexpr(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(sum), st2.to_string(parsed));
    }

    #[test]
    fn sexpr_rational() {
        let mut st = Store::new();
        let rat = st.rat(-7, 4);
        let s = to_sexpr(&st, rat);
        let mut st2 = Store::new();
        let parsed = from_sexpr(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(rat), st2.to_string(parsed));
    }

    #[test]
    fn sexpr_single_element_mul() {
        let mut st = Store::new();
        let x = st.sym("x");
        let single_mul = st.mul(vec![x]);
        // Single-element mul returns the element itself
        assert_eq!(single_mul, x);
        let s = to_sexpr(&st, single_mul);
        let mut st2 = Store::new();
        let parsed = from_sexpr(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(single_mul), st2.to_string(parsed));
    }

    #[test]
    fn sexpr_function_no_args() {
        let mut st = Store::new();
        let f = st.func("f", vec![]);
        let s = to_sexpr(&st, f);
        // Function with no args should be serializable
        assert!(s.contains("Fn") && s.contains("f"));
    }

    #[test]
    fn sexpr_complex_nested_expression() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let two = st.int(2);
        // ((x^2) * y) + (sin(x + y))
        let x2 = st.pow(x, two);
        let prod = st.mul(vec![x2, y]);
        let sum_args = st.add(vec![x, y]);
        let sin_sum = st.func("sin", vec![sum_args]);
        let expr = st.add(vec![prod, sin_sum]);

        let s = to_sexpr(&st, expr);
        let mut st2 = Store::new();
        let parsed = from_sexpr(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(expr), st2.to_string(parsed));
    }

    #[test]
    fn sexpr_quoted_symbol_names() {
        let mut st = Store::new();
        let s = "(Sym \"x_1\")";
        let parsed = from_sexpr(&mut st, s).expect("parse");
        assert!(st.to_string(parsed).contains("x_1"));
    }

    #[test]
    fn sexpr_whitespace_handling() {
        let mut st = Store::new();
        // Extra whitespace should be handled
        let s = "(  +   ( Int   1 )   ( Int   2 )  )";
        let parsed = from_sexpr(&mut st, s).expect("parse");
        let one = st.int(1);
        let two = st.int(2);
        let expected = st.add(vec![one, two]);
        assert_eq!(st.to_string(parsed), st.to_string(expected));
    }

    #[test]
    fn sexpr_empty_add() {
        let mut st = Store::new();
        let empty_add = st.add(vec![]); // Store converts this to (Int 0)
        let s = to_sexpr(&st, empty_add);
        assert_eq!(s, "(Int 0)");
        let mut st2 = Store::new();
        let parsed = from_sexpr(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(empty_add), st2.to_string(parsed));
    }

    #[test]
    fn sexpr_empty_mul() {
        let mut st = Store::new();
        let empty_mul = st.mul(vec![]); // Store converts this to (Int 1)
        let s = to_sexpr(&st, empty_mul);
        assert_eq!(s, "(Int 1)");
        let mut st2 = Store::new();
        let parsed = from_sexpr(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(empty_mul), st2.to_string(parsed));
    }

    #[test]
    fn sexpr_function_foo_no_args() {
        let mut st = Store::new();
        let f = st.func("foo", vec![]);
        let s = to_sexpr(&st, f);
        assert!(s.contains("Fn foo"));
        let mut st2 = Store::new();
        let parsed = from_sexpr(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(f), st2.to_string(parsed));
    }

    #[test]
    fn sexpr_symbol_with_spaces() {
        let mut st = Store::new();
        let sym = st.sym("hello world");
        let s = to_sexpr(&st, sym);
        assert!(s.contains("\"hello world\""));
        let mut st2 = Store::new();
        let parsed = from_sexpr(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(sym), st2.to_string(parsed));
    }

    #[test]
    fn sexpr_symbol_with_quote() {
        let mut st = Store::new();
        let sym = st.sym("test\"quote");
        let s = to_sexpr(&st, sym);
        assert!(s.contains("\\\""));
        let mut st2 = Store::new();
        let parsed = from_sexpr(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(sym), st2.to_string(parsed));
    }

    #[test]
    fn sexpr_parse_errors_comprehensive() {
        let mut st = Store::new();

        assert!(from_sexpr(&mut st, "").is_err());
        assert!(from_sexpr(&mut st, "(").is_err());
        assert!(from_sexpr(&mut st, "( Int )").is_err());
        assert!(from_sexpr(&mut st, "( Int not_a_number )").is_err());
        assert!(from_sexpr(&mut st, "( Rat 1 )").is_err());
        assert!(from_sexpr(&mut st, "( Rat not_num not_num )").is_err());
        assert!(from_sexpr(&mut st, "( Unknown )").is_err());
        assert!(from_sexpr(&mut st, "( + )").is_ok()); // Empty add is ok
        assert!(from_sexpr(&mut st, "( Fn )").is_err());
        assert!(from_sexpr(&mut st, "( ^ (Int 1) )").is_err()); // Needs 2 args
    }

    #[test]
    fn sexpr_negative_numbers() {
        let mut st = Store::new();
        let neg_int = st.int(-42);
        let s = to_sexpr(&st, neg_int);
        assert!(s.contains("-42"));
        let mut st2 = Store::new();
        let parsed = from_sexpr(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(neg_int), st2.to_string(parsed));
    }

    #[test]
    fn sexpr_complex_nested() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let two = st.int(2);
        let three = st.int(3);

        // sin((x + y)^2) * 3
        let sum = st.add(vec![x, y]);
        let pow = st.pow(sum, two);
        let sin = st.func("sin", vec![pow]);
        let expr = st.mul(vec![sin, three]);

        let s = to_sexpr(&st, expr);
        let mut st2 = Store::new();
        let parsed = from_sexpr(&mut st2, &s).expect("parse");

        assert_eq!(st.to_string(expr), st2.to_string(parsed));
    }

    #[test]
    fn sexpr_unclosed_paren() {
        let mut st = Store::new();
        assert!(from_sexpr(&mut st, "( + (Int 1) (Int 2)").is_err());
    }

    #[test]
    fn sexpr_unclosed_quote() {
        let mut st = Store::new();
        assert!(from_sexpr(&mut st, "(Sym \"unclosed").is_err());
    }
}
