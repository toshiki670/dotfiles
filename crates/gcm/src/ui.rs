//! 表示処理。

use crate::proposals::Commit;

/// 提案コミットを表示する。
pub(crate) fn display(proposals: &[Commit]) {
    let count = proposals.len();
    println!();
    if count == 1 {
        println!("提案されたコミット:");
    } else {
        println!("提案されたコミット ({count} 件):");
    }
    for (idx, commit) in proposals.iter().enumerate() {
        println!();
        // bold cyan
        println!("\x1b[1;36m  {}. {}\x1b[0m", idx + 1, commit.message);
        for file in &commit.files {
            println!("     {file}");
        }
    }
    println!();
}
