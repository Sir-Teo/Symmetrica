//! JSON serializer for Symmetrica expressions (no external deps).
//! Format is stable and minimal:
//! - Integer: {"Integer": k}
//! - Rational: {"Rational": {"num": n, "den": d}}
//! - Symbol: {"Symbol": "name"}
//! - Function: {"Function": {"name": "f", "args": [ ... ]}}
//! - Add: {"Add": [ ... ]}
//! - Mul: {"Mul": [ ... ]}
//! - Pow: {"Pow": {"base": ..., "exp": ...}}

use expr_core::{ExprId, Op, Payload, Store};

/// Serialize an expression to the stable JSON format described above.
pub fn to_json(st: &Store, id: ExprId) -> String {
    fn esc(s: &str) -> String {
        // Minimal string escape for JSON: quotes and backslashes
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }
    fn go(st: &Store, id: ExprId) -> String {
        let n = st.get(id);
        match (&n.op, &n.payload) {
            (Op::Integer, Payload::Int(k)) => format!("{{\"Integer\": {k}}}"),
            (Op::Rational, Payload::Rat(a, b)) => {
                format!("{{\"Rational\": {{\"num\": {a}, \"den\": {b}}}}}")
            }
            (Op::Symbol, Payload::Sym(name)) => format!("{{\"Symbol\": \"{}\"}}", esc(name)),
            (Op::Function, Payload::Func(name)) => {
                let args = n.children.iter().map(|c| go(st, *c)).collect::<Vec<_>>().join(", ");
                format!("{{\"Function\": {{\"name\": \"{}\", \"args\": [{}]}}}}", esc(name), args)
            }
            (Op::Add, _) => {
                let parts = n.children.iter().map(|c| go(st, *c)).collect::<Vec<_>>().join(", ");
                format!("{{\"Add\": [{}]}}", parts)
            }
            (Op::Mul, _) => {
                let parts = n.children.iter().map(|c| go(st, *c)).collect::<Vec<_>>().join(", ");
                format!("{{\"Mul\": [{}]}}", parts)
            }
            (Op::Pow, _) => {
                let b = go(st, n.children[0]);
                let e = go(st, n.children[1]);
                format!("{{\"Pow\": {{\"base\": {b}, \"exp\": {e}}}}}")
            }
            _ => "{\"Unknown\": null}".into(),
        }
    }
    go(st, id)
}

/// Parse an expression from the stable JSON format produced by `to_json()`.
pub fn from_json(st: &mut Store, input: &str) -> Result<ExprId, String> {
    #[derive(Clone, Debug, PartialEq)]
    enum J {
        Obj(Vec<(String, J)>),
        Arr(Vec<J>),
        Str(String),
        Num(i64),
        Null,
    }
    struct P<'a> {
        s: &'a [u8],
        i: usize,
    }
    impl<'a> P<'a> {
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
        fn expect(&mut self, b: u8) -> Result<(), String> {
            self.skip_ws();
            if self.peek() == Some(b) {
                self.bump();
                Ok(())
            } else {
                Err(format!("expected '{}'", b as char))
            }
        }
        fn parse_str(&mut self) -> Result<String, String> {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err("expected string".into());
            }
            self.bump();
            let mut out = String::new();
            while let Some(c) = self.peek() {
                self.bump();
                match c {
                    b'"' => return Ok(out),
                    b'\\' => {
                        if let Some(nc) = self.peek() {
                            self.bump();
                            out.push(nc as char);
                        } else {
                            return Err("unterminated escape".into());
                        }
                    }
                    _ => out.push(c as char),
                }
            }
            Err("unterminated string".into())
        }
        fn parse_num(&mut self) -> Result<i64, String> {
            self.skip_ws();
            let start = self.i;
            if self.peek() == Some(b'-') {
                self.bump();
            }
            let mut saw = false;
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.bump();
                    saw = true;
                } else {
                    break;
                }
            }
            if !saw {
                return Err("expected integer".into());
            }
            let s = std::str::from_utf8(&self.s[start..self.i]).unwrap();
            s.parse::<i64>().map_err(|_| "invalid integer".into())
        }
        fn parse_value(&mut self) -> Result<J, String> {
            self.skip_ws();
            match self.peek() {
                Some(b'{') => {
                    self.bump();
                    let mut fields = Vec::new();
                    self.skip_ws();
                    if self.peek() == Some(b'}') {
                        self.bump();
                        return Ok(J::Obj(fields));
                    }
                    loop {
                        let k = self.parse_str()?;
                        self.skip_ws();
                        self.expect(b':')?;
                        let v = self.parse_value()?;
                        fields.push((k, v));
                        self.skip_ws();
                        match self.peek() {
                            Some(b',') => {
                                self.bump();
                            }
                            Some(b'}') => {
                                self.bump();
                                break;
                            }
                            _ => return Err("expected ',' or '}'".into()),
                        }
                    }
                    Ok(J::Obj(fields))
                }
                Some(b'[') => {
                    self.bump();
                    let mut items = Vec::new();
                    self.skip_ws();
                    if self.peek() == Some(b']') {
                        self.bump();
                        return Ok(J::Arr(items));
                    }
                    loop {
                        let v = self.parse_value()?;
                        items.push(v);
                        self.skip_ws();
                        match self.peek() {
                            Some(b',') => {
                                self.bump();
                            }
                            Some(b']') => {
                                self.bump();
                                break;
                            }
                            _ => return Err("expected ',' or ']'".into()),
                        }
                    }
                    Ok(J::Arr(items))
                }
                Some(b'"') => Ok(J::Str(self.parse_str()?)),
                Some(c) if c == b'-' || c.is_ascii_digit() => Ok(J::Num(self.parse_num()?)),
                _ => Err("unexpected token".into()),
            }
        }
    }

    fn build_expr(st: &mut Store, v: J) -> Result<ExprId, String> {
        match v {
            J::Obj(mut fields) => {
                // We expect a single top-level key
                if fields.len() != 1 {
                    return Err("expected single-key object".into());
                }
                let (k, v) = fields.remove(0);
                match k.as_str() {
                    "Integer" => match v {
                        J::Num(k) => Ok(st.int(k)),
                        _ => Err("Integer expects number".into()),
                    },
                    "Rational" => match v {
                        J::Obj(mut f2) => {
                            let mut n = None;
                            let mut d = None;
                            for (kk, vv) in f2.drain(..) {
                                match kk.as_str() {
                                    "num" => {
                                        if let J::Num(x) = vv {
                                            n = Some(x)
                                        } else {
                                            return Err("num must be number".into());
                                        }
                                    }
                                    "den" => {
                                        if let J::Num(x) = vv {
                                            d = Some(x)
                                        } else {
                                            return Err("den must be number".into());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            let (nn, dd) = (n.ok_or("missing num")?, d.ok_or("missing den")?);
                            Ok(st.rat(nn, dd))
                        }
                        _ => Err("Rational expects object".into()),
                    },
                    "Symbol" => match v {
                        J::Str(s) => Ok(st.sym(s)),
                        _ => Err("Symbol expects string".into()),
                    },
                    "Function" => match v {
                        J::Obj(mut f2) => {
                            let mut name = None;
                            let mut args = None;
                            for (kk, vv) in f2.drain(..) {
                                match kk.as_str() {
                                    "name" => {
                                        if let J::Str(s) = vv {
                                            name = Some(s)
                                        } else {
                                            return Err("name must be string".into());
                                        }
                                    }
                                    "args" => {
                                        if let J::Arr(a) = vv {
                                            args = Some(a)
                                        } else {
                                            return Err("args must be array".into());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            let nm = name.ok_or("missing name")?;
                            let aitems = args.ok_or("missing args")?;
                            let mut ch: Vec<ExprId> = Vec::with_capacity(aitems.len());
                            for it in aitems {
                                ch.push(build_expr(st, it)?);
                            }
                            Ok(st.func(nm, ch))
                        }
                        _ => Err("Function expects object".into()),
                    },
                    "Add" => match v {
                        J::Arr(items) => {
                            let mut ch = Vec::with_capacity(items.len());
                            for it in items {
                                ch.push(build_expr(st, it)?);
                            }
                            Ok(st.add(ch))
                        }
                        _ => Err("Add expects array".into()),
                    },
                    "Mul" => match v {
                        J::Arr(items) => {
                            let mut ch = Vec::with_capacity(items.len());
                            for it in items {
                                ch.push(build_expr(st, it)?);
                            }
                            Ok(st.mul(ch))
                        }
                        _ => Err("Mul expects array".into()),
                    },
                    "Pow" => match v {
                        J::Obj(mut f2) => {
                            let mut base = None;
                            let mut exp = None;
                            for (kk, vv) in f2.drain(..) {
                                match kk.as_str() {
                                    "base" => base = Some(vv),
                                    "exp" => exp = Some(vv),
                                    _ => {}
                                }
                            }
                            let b = build_expr(st, base.ok_or("missing base")?)?;
                            let e = build_expr(st, exp.ok_or("missing exp")?)?;
                            Ok(st.pow(b, e))
                        }
                        _ => Err("Pow expects object".into()),
                    },
                    _ => Err("unknown head".into()),
                }
            }
            _ => Err("expected object".into()),
        }
    }

    let mut p = P::new(input);
    let v = p.parse_value()?;
    build_expr(st, v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_contains_keys() {
        let mut st = Store::new();
        let x = st.sym("x");
        let one = st.int(1);
        let xp1 = st.add(vec![x, one]);
        let two = st.int(2);
        let expr = st.pow(xp1, two);
        let s = to_json(&st, expr);
        assert!(s.contains("\"Pow\""));
        assert!(s.contains("\"Add\""));
        assert!(s.contains("\"Integer\""));
        assert!(s.contains("\"Symbol\""));
    }

    #[test]
    fn json_roundtrip_basic() {
        let mut st = Store::new();
        let x = st.sym("x");
        let three = st.int(3);
        let pow = st.pow(x, three);
        let sinx = st.func("sin", vec![x]);
        let rat = st.rat(3, 2);
        let expr = st.add(vec![pow, sinx, rat]);
        let s = to_json(&st, expr);
        let mut st2 = Store::new();
        let parsed = from_json(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(expr), st2.to_string(parsed));
    }

    #[test]
    fn json_parse_errors() {
        let mut st = Store::new();
        // Missing closing brace
        assert!(from_json(&mut st, "{\"Symbol\": ").is_err());
        // Invalid integer
        assert!(from_json(&mut st, "{\"Integer\": abc}").is_err());
        // Unexpected token
        assert!(from_json(&mut st, "{\"Symbol\": 123}").is_err());
        // Missing field
        assert!(from_json(&mut st, "{\"Pow\": {\"base\": {\"Symbol\": \"x\"}}}").is_err());
        // Wrong type for Add children
        assert!(from_json(&mut st, "{\"Add\": 5}").is_err());
    }

    #[test]
    fn json_nested_objects() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let sum = st.add(vec![x, y]);
        let two = st.int(2);
        let prod = st.mul(vec![sum, two]);
        let s = to_json(&st, prod);
        let mut st2 = Store::new();
        let parsed = from_json(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(prod), st2.to_string(parsed));
    }

    #[test]
    fn json_rational() {
        let mut st = Store::new();
        let rat = st.rat(5, 3);
        let s = to_json(&st, rat);
        assert!(s.contains("\"num\""));
        assert!(s.contains("\"den\""));
        let mut st2 = Store::new();
        let parsed = from_json(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(rat), st2.to_string(parsed));
    }

    #[test]
    fn json_function_with_multiple_args() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let z = st.sym("z");
        let f = st.func("f", vec![x, y, z]);
        let s = to_json(&st, f);
        let mut st2 = Store::new();
        let parsed = from_json(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(f), st2.to_string(parsed));
    }

    #[test]
    fn json_pow_roundtrip() {
        let mut st = Store::new();
        let x = st.sym("x");
        let three = st.int(3);
        let pow_expr = st.pow(x, three);
        let s = to_json(&st, pow_expr);
        assert!(s.contains("\"Pow\""));
        assert!(s.contains("\"base\""));
        assert!(s.contains("\"exp\""));
        let mut st2 = Store::new();
        let parsed = from_json(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(pow_expr), st2.to_string(parsed));
    }

    #[test]
    fn json_empty_add_mul() {
        let mut st = Store::new();
        // Empty add canonicalizes to 0
        let empty_add = st.add(vec![]);
        assert_eq!(empty_add, st.int(0));
        let s = to_json(&st, empty_add);
        assert!(s.contains("\"Integer\""));
    }

    #[test]
    fn json_negative_integer() {
        let mut st = Store::new();
        let neg = st.int(-42);
        let s = to_json(&st, neg);
        assert!(s.contains("-42"));
        let mut st2 = Store::new();
        let parsed = from_json(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(neg), st2.to_string(parsed));
    }

    #[test]
    fn json_negative_rational() {
        let mut st = Store::new();
        let neg_rat = st.rat(-3, 4);
        let s = to_json(&st, neg_rat);
        let mut st2 = Store::new();
        let parsed = from_json(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(neg_rat), st2.to_string(parsed));
    }

    #[test]
    fn json_symbol_with_escape() {
        let mut st = Store::new();
        let sym = st.sym("test\"quote");
        let s = to_json(&st, sym);
        assert!(s.contains("test\\\"quote"));
        let mut st2 = Store::new();
        let parsed = from_json(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(sym), st2.to_string(parsed));
    }

    #[test]
    fn json_symbol_with_backslash() {
        let mut st = Store::new();
        let sym = st.sym("test\\backslash");
        let s = to_json(&st, sym);
        assert!(s.contains("test\\\\backslash"));
        let mut st2 = Store::new();
        let parsed = from_json(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(sym), st2.to_string(parsed));
    }

    #[test]
    fn json_empty_add() {
        let mut st = Store::new();
        let empty_add = st.add(vec![]); // Store converts to 0
        let s = to_json(&st, empty_add);
        assert!(s.contains("Integer"));
        assert!(s.contains("0"));
        let mut st2 = Store::new();
        let parsed = from_json(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(empty_add), st2.to_string(parsed));
    }

    #[test]
    fn json_empty_mul() {
        let mut st = Store::new();
        let empty_mul = st.mul(vec![]); // Store converts to 1
        let s = to_json(&st, empty_mul);
        assert!(s.contains("Integer"));
        assert!(s.contains("1"));
        let mut st2 = Store::new();
        let parsed = from_json(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(empty_mul), st2.to_string(parsed));
    }

    #[test]
    fn json_function_no_args() {
        let mut st = Store::new();
        let f = st.func("foo", vec![]);
        let s = to_json(&st, f);
        assert!(s.contains("\"name\": \"foo\""));
        assert!(s.contains("\"args\": []"));
        let mut st2 = Store::new();
        let parsed = from_json(&mut st2, &s).expect("parse");
        assert_eq!(st.to_string(f), st2.to_string(parsed));
    }

    #[test]
    fn json_parse_errors_comprehensive() {
        let mut st = Store::new();

        assert!(from_json(&mut st, "").is_err());
        assert!(from_json(&mut st, "{").is_err());
        assert!(from_json(&mut st, "{\"Unknown\": 123}").is_err());
        assert!(from_json(&mut st, "{\"Integer\": \"not_a_number\"}").is_err());
        assert!(from_json(&mut st, "{\"Add\": \"not_an_array\"}").is_err());
        assert!(from_json(&mut st, "{\"Mul\": 123}").is_err());
        assert!(from_json(&mut st, "{\"Pow\": {}}").is_err());
        assert!(from_json(&mut st, "{\"Pow\": {\"base\": {\"Integer\": 1}}}").is_err());
        assert!(from_json(&mut st, "{\"Function\": {}}").is_err());
        assert!(from_json(&mut st, "{\"Function\": {\"name\": \"f\"}}").is_err());
        assert!(from_json(&mut st, "{\"Rational\": {}}").is_err());
        assert!(from_json(&mut st, "{\"Rational\": {\"num\": 1}}").is_err());
    }

    #[test]
    fn json_complex_nested() {
        let mut st = Store::new();
        let x = st.sym("x");
        let y = st.sym("y");
        let two = st.int(2);
        let three = st.int(3);

        // ((x + y)^2) * 3
        let sum = st.add(vec![x, y]);
        let pow = st.pow(sum, two);
        let expr = st.mul(vec![pow, three]);

        let s = to_json(&st, expr);
        let mut st2 = Store::new();
        let parsed = from_json(&mut st2, &s).expect("parse");

        assert_eq!(st.to_string(expr), st2.to_string(parsed));
    }

    #[test]
    fn json_rational_with_spaces() {
        let mut st = Store::new();
        let json = r#"{ "Rational" :  { "num"  : 3  ,  "den" :  5  } }"#;
        let parsed = from_json(&mut st, json).expect("parse");
        assert_eq!(st.to_string(parsed), "3/5");
    }

    #[test]
    fn json_array_trailing_comma_rejected() {
        let mut st = Store::new();
        let json = r#"{"Add": [{"Integer": 1}, {"Integer": 2},]}"#;
        // Most JSON parsers reject trailing commas
        assert!(from_json(&mut st, json).is_err());
    }
}
