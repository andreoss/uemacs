use super::{Editor, gtfun_arity, next_token};

impl Editor {
    pub(crate) fn evaluate_expression(&self, expr: &str) -> String {
        if expr.is_empty() {
            return String::new();
        }
        if expr.as_bytes()[0] == b'"' {
            let s = &expr[1..];
            return s.strip_suffix('"').unwrap_or(s).to_string();
        }
        let expr = expr.trim();
        if expr.is_empty() {
            return String::new();
        }
        match expr.as_bytes()[0] {
            b'&' => {
                let (v, _) = self.eval_function(&expr[1..]);
                v
            }
            b'$' => self.evaluate_env_var(&expr[1..]),
            b'%' => self.user_vars.get(&expr[1..]).cloned().unwrap_or_default(),
            _ => expr.to_string(),
        }
    }

    pub(super) fn parse_arg<'a>(&self, input: &'a str) -> (String, &'a str) {
        let (tok, mut rest) = next_token(input);
        if tok.is_empty() {
            return (String::new(), rest);
        }
        if let Some(fname_full) = tok.strip_prefix('&') {
            let fname: String = fname_full
                .chars()
                .take(3)
                .collect::<String>()
                .to_lowercase();
            let n = gtfun_arity(&fname);
            let mut args = [String::new(), String::new(), String::new()];
            for arg in args.iter_mut().take(n) {
                let (v, r) = self.parse_arg(rest);
                *arg = v;
                rest = r;
            }
            return (
                self.invoke_function(&fname, &args[0], &args[1], &args[2]),
                rest,
            );
        }
        if let Some(s) = tok.strip_prefix('"') {
            return (s.to_string(), rest);
        }
        (self.evaluate_expression(&tok), rest)
    }

    pub(super) fn eval_function<'a>(&self, input: &'a str) -> (String, &'a str) {
        let (fname_full, mut rest) = next_token(input);
        if fname_full.is_empty() {
            return (String::new(), rest);
        }
        let fname: String = fname_full
            .chars()
            .take(3)
            .collect::<String>()
            .to_lowercase();
        let n = gtfun_arity(&fname);
        let mut args = [String::new(), String::new(), String::new()];
        for arg in args.iter_mut().take(n) {
            let (v, r) = self.parse_arg(rest);
            *arg = v;
            rest = r;
        }
        (
            self.invoke_function(&fname, &args[0], &args[1], &args[2]),
            rest,
        )
    }
}
