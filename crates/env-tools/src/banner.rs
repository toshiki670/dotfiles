//! セクションバナー出力（旧 bash の `header()` 相当）。

/// 罫線で囲んだ見出しを stdout に出す（前後に空行）。旧 bash の `header` と同じ体裁。
pub fn header(title: &str) {
    const RULE: &str = "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━";
    println!();
    println!("{RULE}");
    println!("  {title}");
    println!("{RULE}");
    println!();
}
