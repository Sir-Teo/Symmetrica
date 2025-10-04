use expr_core::{ExprId, Store};
use io::json::from_json;
use simplify::simplify;

fn usage() {
    eprintln!("matika_cli commands:\n  parse (--sexpr <S> | --json <J>)\n  simplify (--sexpr <S> | --json <J>)\n  diff (--sexpr <S> | --json <J>) --var <x>\n  integrate (--sexpr <S> | --json <J>) --var <x>\n  solve (--sexpr <S> | --json <J>) --var <x>\n  plot (--sexpr <S> | --json <J>) --var <x> --xmin <a> --xmax <b> --samples <n> --width <w> --height <h>");
}

fn arg_val(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == key).map(|w| w[1].clone())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage();
        return;
    }
    let cmd = &args[0];
    let rest = &args[1..];

    match cmd.as_str() {
        "parse" => {
            let mut st = Store::new();
            match parse_input(&mut st, rest) {
                Ok(id) => {
                    println!("text:   {}", st.to_string(id));
                    println!("latex:  {}", io::to_latex(&st, id));
                    println!("json:   {}", io::to_json(&st, id));
                    println!("sexpr:  {}", io::to_sexpr(&st, id));
                }
                Err(e) => {
                    eprintln!("parse error: {e}");
                    std::process::exit(2);
                }
            }
        }
        "simplify" => {
            let mut st = Store::new();
            let id = match parse_input(&mut st, rest) {
                Ok(i) => i,
                Err(_) => {
                    usage();
                    return;
                }
            };
            let s = simplify(&mut st, id);
            println!("{}", st.to_string(s));
        }
        "diff" => {
            let Some(var) = arg_val(rest, "--var") else {
                usage();
                return;
            };
            let mut st = Store::new();
            let id = match parse_input(&mut st, rest) {
                Ok(i) => i,
                Err(_) => {
                    usage();
                    return;
                }
            };
            let d = calculus::diff(&mut st, id, &var);
            let ds = simplify(&mut st, d);
            println!("{}", st.to_string(ds));
        }
        "integrate" => {
            let Some(var) = arg_val(rest, "--var") else {
                usage();
                return;
            };
            let mut st = Store::new();
            let id = match parse_input(&mut st, rest) {
                Ok(i) => i,
                Err(_) => {
                    usage();
                    return;
                }
            };
            match calculus::integrate(&mut st, id, &var) {
                Some(ii) => println!("{}", st.to_string(ii)),
                None => {
                    eprintln!("not integrable");
                    std::process::exit(3);
                }
            }
        }
        "solve" => {
            let Some(var) = arg_val(rest, "--var") else {
                usage();
                return;
            };
            let mut st = Store::new();
            let id = match parse_input(&mut st, rest) {
                Ok(i) => i,
                Err(_) => {
                    usage();
                    return;
                }
            };
            match solver::solve_univariate(&mut st, id, &var) {
                Some(roots) => {
                    for r in roots {
                        println!("{}", st.to_string(r));
                    }
                }
                None => {
                    eprintln!("cannot solve completely");
                    std::process::exit(4);
                }
            }
        }
        "plot" => {
            let Some(var) = arg_val(rest, "--var") else {
                usage();
                return;
            };
            let xmin: f64 = arg_val(rest, "--xmin").and_then(|s| s.parse().ok()).unwrap_or(-1.0);
            let xmax: f64 = arg_val(rest, "--xmax").and_then(|s| s.parse().ok()).unwrap_or(1.0);
            let samples: usize =
                arg_val(rest, "--samples").and_then(|s| s.parse().ok()).unwrap_or(100);
            let width: u32 = arg_val(rest, "--width").and_then(|s| s.parse().ok()).unwrap_or(640);
            let height: u32 = arg_val(rest, "--height").and_then(|s| s.parse().ok()).unwrap_or(480);
            let mut st = Store::new();
            let id = match parse_input(&mut st, rest) {
                Ok(i) => i,
                Err(_) => {
                    usage();
                    return;
                }
            };
            let cfg = plot::PlotConfig::new(&var, xmin, xmax, samples, width, height);
            let svg = plot::plot_svg(&st, id, &cfg);
            println!("{}", svg);
        }
        _ => usage(),
    }
}

fn parse_input(st: &mut Store, args: &[String]) -> Result<ExprId, String> {
    if let Some(sx) = arg_val(args, "--sexpr") {
        io::from_sexpr(st, &sx)
    } else if let Some(js) = arg_val(args, "--json") {
        from_json(st, &js)
    } else {
        Err("missing input (--sexpr or --json)".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_simplify_smoke() {
        let mut st = Store::new();
        let id = io::from_sexpr(&mut st, "(+ (* 2 (Sym x)) (* 3 (Sym x)))").unwrap();
        let s = simplify(&mut st, id);
        // Expect 5*x
        assert!(st.to_string(s).contains("5"));
    }

    #[test]
    fn diff_smoke() {
        let mut st = Store::new();
        let id = io::from_sexpr(&mut st, "(^ (Sym x) (Int 3))").unwrap();
        let d = calculus::diff(&mut st, id, "x");
        let ds = simplify(&mut st, d);
        assert!(st.to_string(ds).contains("3"));
    }

    #[test]
    fn json_parse_smoke() {
        let mut st = Store::new();
        // {"Pow": {"base": {"Symbol": "x"}, "exp": {"Integer": 3}}}
        let j = "{\"Pow\": {\"base\": {\"Symbol\": \"x\"}, \"exp\": {\"Integer\": 3}}}";
        let id = from_json(&mut st, j).unwrap();
        assert!(st.to_string(id).contains("x"));
    }
}
