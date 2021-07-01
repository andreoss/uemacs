use super::{
    Bindings, Display, Editor, MacroKey, Result, TerminalBackend, next_rng, parse_key_name,
    parse_leading_int, stol,
};
use std::borrow::Cow;

impl Editor {
    pub(super) fn invoke_function(&self, fname: &str, v1: &str, v2: &str, v3: &str) -> String {
        let parse_i = parse_leading_int;
        let ltos = |b: bool| if b { "T".to_string() } else { "F".to_string() };
        match fname {
            "add" => (parse_i(v1) + parse_i(v2)).to_string(),
            "sub" => (parse_i(v1) - parse_i(v2)).to_string(),
            "tim" => (parse_i(v1) * parse_i(v2)).to_string(),
            "div" => {
                let d = parse_i(v2);
                if d == 0 {
                    "0".to_string()
                } else {
                    (parse_i(v1) / d).to_string()
                }
            }
            "mod" => {
                let d = parse_i(v2);
                if d == 0 {
                    "0".to_string()
                } else {
                    (parse_i(v1) % d).to_string()
                }
            }
            "neg" => (-parse_i(v1)).to_string(),
            "cat" => format!("{v1}{v2}"),
            "lef" => {
                let n = usize::try_from(parse_i(v2).max(0)).unwrap_or(usize::MAX);
                v1.chars().take(n).collect()
            }
            "rig" => {
                let n = usize::try_from(parse_i(v2).max(0)).unwrap_or(usize::MAX);
                let len = v1.chars().count();
                v1.chars().skip(len.saturating_sub(n)).collect()
            }
            "mid" => {
                let start = usize::try_from((parse_i(v2) - 1).max(0)).unwrap_or(usize::MAX);
                let len = usize::try_from(parse_i(v3).max(0)).unwrap_or(usize::MAX);
                v1.chars().skip(start).take(len).collect()
            }
            "not" => ltos(!stol(v1)),
            "equ" => ltos(parse_i(v1) == parse_i(v2)),
            "les" => ltos(parse_i(v1) < parse_i(v2)),
            "gre" => ltos(parse_i(v1) > parse_i(v2)),
            "seq" => ltos(v1 == v2),
            "sle" => ltos(v1 < v2),
            "sgr" => ltos(v1 > v2),
            "ind" => self.evaluate_expression(v1),
            "and" => ltos(stol(v1) && stol(v2)),
            "or" => ltos(stol(v1) || stol(v2)),
            "len" => v1.chars().count().to_string(),
            "upp" => v1.to_uppercase(),
            "low" => v1.to_lowercase(),
            "tru" => ltos(parse_i(v1) == 42),
            _ => Self::invoke_function_ext(fname, v1, v2, v3),
        }
    }

    fn invoke_function_ext(fname: &str, v1: &str, v2: &str, v3: &str) -> String {
        let parse_i = parse_leading_int;
        let ltos = |b: bool| if b { "T".to_string() } else { "F".to_string() };
        match fname {
            "asc" => i64::from(v1.bytes().next().unwrap_or(0)).to_string(),
            "chr" => {
                let n = u32::try_from(parse_i(v1)).unwrap_or(0);
                char::from_u32(n).map(|c| c.to_string()).unwrap_or_default()
            }
            "rnd" => {
                let n = parse_i(v1).abs();
                if n == 0 {
                    "0".to_string()
                } else {
                    let r = next_rng();
                    ((r % u64::try_from(n).unwrap_or(1)) + 1).to_string()
                }
            }
            "abs" => parse_i(v1).abs().to_string(),
            "sin" => v1.find(v2).map_or_else(
                || "0".to_string(),
                |i| (i64::try_from(i).unwrap_or(i64::MAX) + 1).to_string(),
            ),
            "env" => std::env::var(v1).unwrap_or_default(),
            "bin" => parse_key_name(v1).map_or_else(
                || "ERROR".to_string(),
                |kc| {
                    Bindings::new()
                        .lookup(kc)
                        .map_or(Cow::Borrowed("ERROR"), crate::bind::command_name)
                        .to_string()
                },
            ),
            "exi" => ltos(std::path::Path::new(v1).exists()),
            "fin" => {
                if std::path::Path::new(v1).exists() {
                    v1.to_string()
                } else {
                    String::new()
                }
            }
            "ban" => (parse_i(v1) & parse_i(v2)).to_string(),
            "bor" => (parse_i(v1) | parse_i(v2)).to_string(),
            "bxo" => (parse_i(v1) ^ parse_i(v2)).to_string(),
            "bno" => (!parse_i(v1)).to_string(),
            "xla" => {
                let from: Vec<char> = v2.chars().collect();
                let to: Vec<char> = v3.chars().collect();
                v1.chars()
                    .map(|c| match from.iter().position(|&f| f == c) {
                        Some(i) if i < to.len() => to[i],
                        _ => c,
                    })
                    .collect()
            }
            _ => String::new(),
        }
    }

    pub(super) fn execute_cmd_str<T: TerminalBackend>(
        &mut self,
        input: &str,
        term: &mut T,
        display: &mut Display,
        bindings: &Bindings,
    ) -> Result<()> {
        let parts: Vec<&str> = input.trim().splitn(2, char::is_whitespace).collect();
        let (cmd_name, prefix_f, prefix_n) =
            if parts.len() == 2 && parts[0].chars().all(|c| c.is_ascii_digit()) {
                (
                    parts[1].trim(),
                    true,
                    parts[0].parse::<usize>().unwrap_or(1),
                )
            } else {
                (parts[0], false, 1)
            };
        if let Some(cmd) = Bindings::lookup_name(cmd_name) {
            self.dispatch(cmd, term, display, bindings, prefix_f, prefix_n)
        } else {
            display.write_echo(term, &format!("(No such function: {cmd_name})"))?;
            Ok(())
        }
    }

    pub(super) fn execute_stored_macro<T: TerminalBackend>(
        &mut self,
        slot: usize,
        term: &mut T,
        display: &mut Display,
        bindings: &Bindings,
        _f: bool,
        n: usize,
    ) -> Result<()> {
        if let Some(keys) = self.stored_macros[slot].clone() {
            let was_recording = self.recording_macro;
            self.recording_macro = false;
            let result = (|| -> Result<()> {
                for _ in 0..n.max(1) {
                    for &MacroKey { cmd, f, n } in &keys {
                        self.dispatch(cmd, term, display, bindings, f, n)?;
                    }
                }
                Ok(())
            })();
            self.recording_macro = was_recording;
            return result;
        }
        Ok(())
    }
}
