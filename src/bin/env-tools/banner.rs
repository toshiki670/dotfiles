//! セクションバナー出力。

/// 罫線で囲んだ見出しを stdout に出す（前後に空行）。
pub fn header(title: &str) {
    const RULE: &str = "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━";
    println!();
    println!("{RULE}");
    println!("  {title}");
    println!("{RULE}");
    println!();
}
