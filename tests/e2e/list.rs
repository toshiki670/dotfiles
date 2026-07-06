//! `dotfiles list` の E2E — 名前順・属性ラベルを検証する。

use crate::dotfiles;
use std::fs;

/// `dotfiles list` が単位を名前順に並べ、dst と属性ラベルを表示することを検証する。
#[test]
fn list_shows_units_sorted_with_attrs() {
    let work = tempfile::tempdir().unwrap();

    let beta = work.path().join("configs/beta");
    let alpha = work.path().join("configs/alpha");
    fs::create_dir_all(&beta).unwrap();
    fs::create_dir_all(&alpha).unwrap();
    fs::write(
        beta.join("manifest.toml"),
        "[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/.config/beta\"\n",
    )
    .unwrap();
    fs::write(
        alpha.join("manifest.toml"),
        "private = true\n[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/.config/alpha\"\n",
    )
    .unwrap();

    let out = dotfiles()
        .arg("list")
        .current_dir(work.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(out).unwrap();

    // 並び順は先頭列（単位名）だけで判定する。dst や属性に名前の部分文字列が
    // 現れても（例: ~/.config/alpha）引っ張られないよう、行を split して先頭トークンで照合する。
    let name_row = |name: &str| {
        stdout
            .lines()
            .position(|l| l.split_whitespace().next() == Some(name))
    };
    let a = name_row("alpha").expect("alpha 行が無い");
    let b = name_row("beta").expect("beta 行が無い");
    assert!(a < b, "名前順（先頭列）に並んでいない:\n{stdout}");
    assert!(
        stdout.contains("~/.config/alpha"),
        "dst が出ていない:\n{stdout}"
    );
    assert!(
        stdout.contains("tree, private"),
        "属性ラベルが出ていない:\n{stdout}",
    );
}

// 「`configs/` が無い場所で list → エラー」は S8（#462）で挙動が変わった: 作業ツリーが
// 無ければ埋め込みフォールバックで解決し、出荷 configs を一覧する。解決の二段切替は
// [`crate::source`] が検証する。
