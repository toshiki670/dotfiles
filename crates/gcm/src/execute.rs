//! 提案コミットの実行。

use std::process::ExitCode;

use crate::git::{git_status, run_status};
use crate::proposals::Commit;

/// 提案コミットを実行する。
pub(crate) fn execute(proposals: &[Commit]) -> ExitCode {
    if proposals.len() == 1 {
        return run_status(&["commit", "-m", &proposals[0].message]);
    }

    // 複数コミット: 全 unstage してからエントリごとに stage + commit。
    let _ = git_status(&["restore", "--staged", "."]);
    for (idx, commit) in proposals.iter().enumerate() {
        let mut add_args = vec!["add"];
        add_args.extend(commit.files.iter().map(String::as_str));
        let _ = git_status(&add_args);

        if !git_status(&["commit", "-m", &commit.message]) {
            eprintln!("コミット {idx} 失敗。残りのファイルはステージされたままです。");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
