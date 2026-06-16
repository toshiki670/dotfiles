//! サマリ/JSON 出力。

use super::Orchestrator;

impl Orchestrator {
    pub fn print_summary(&self) {
        println!("lint: failures={}", self.failures.len());
        for r in &self.failures {
            println!(
                "- {}:{} {} -> ({}) {}",
                r.phase, r.rule, r.file, r.exit_code, r.command
            );
        }
    }

    pub fn print_json(&self, failed: bool) {
        let mut items = Vec::with_capacity(self.failures.len());
        for r in &self.failures {
            items.push(format!(
                "{{\"file\":{},\"rule\":{},\"phase\":{},\"command\":{},\"exitCode\":{}}}",
                json_str(&r.file),
                json_str(&r.rule),
                json_str(&r.phase),
                json_str(&r.command),
                r.exit_code,
            ));
        }
        println!(
            "{{\"failed\":{},\"failureCount\":{},\"failures\":[{}]}}",
            i32::from(failed),
            self.failures.len(),
            items.join(",")
        );
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::json_str;

    #[test]
    fn json_str_escapes() {
        assert_eq!(json_str("a\"b\\c"), "\"a\\\"b\\\\c\"");
    }
}
