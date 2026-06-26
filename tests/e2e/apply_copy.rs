//! `dotfiles apply` の copy 層（S0/S1）の E2E。
//!
//! kind 省略時の copy 既定と `~` 展開・サブディレクトリ再帰と入れ子 manifest の委譲・
//! パーミッション属性の合成を検証する。

use crate::dotfiles;
use rstest::rstest;
use std::fs;

/// kind 省略時に copy として扱われ、`~` が HOME に展開されることを、
/// 一時ソース fixture で検証する（hermetic）。
#[test]
fn apply_defaults_to_copy_and_expands_tilde() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    // 一時ソース configs/demo/{manifest.toml, hello.conf} を用意（kind は省略）。
    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(unit.join("manifest.toml"), "dst = \"~/.config/demo\"\n").unwrap();
    fs::write(unit.join("hello.conf"), "hello = 1\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success();

    let placed = home.path().join(".config/demo/hello.conf");
    assert!(placed.is_file(), "fixture が配置されていない: {placed:?}");
    assert_eq!(fs::read_to_string(&placed).unwrap(), "hello = 1\n");
    // manifest.toml 自体は配置対象外。
    assert!(
        !home.path().join(".config/demo/manifest.toml").exists(),
        "manifest.toml が誤って配置された",
    );
}

// 「`configs/` が無い場所で apply → エラー」は S8（#462）で挙動が変わった: 作業ツリーが
// 無ければ埋め込みフォールバックで解決するため、もうエラーにならない。解決の二段切替は
// [`crate::source`] が検証する（ソース欠落の旧契約はそこへ移った）。

/// S1 受け入れ条件: サブディレクトリ再帰・複数ファイル・manifest の再帰委譲。
/// 親単位は配下を再帰コピーするが、自前 manifest を持つサブツリー（child）は
/// 委譲先の責務として親のコピー対象から外れ、child 自身の dst へ配置される。
#[test]
fn apply_recurses_subdirs_and_delegates_nested_manifests() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let parent = work.path().join("configs/parent");
    fs::create_dir_all(parent.join("nested")).unwrap();
    fs::create_dir_all(parent.join("child")).unwrap();
    fs::write(parent.join("manifest.toml"), "dst = \"~/.config/parent\"\n").unwrap();
    fs::write(parent.join("a.conf"), "a\n").unwrap();
    fs::write(parent.join("nested/b.conf"), "b\n").unwrap();
    // child は自前の manifest を持つ別単位（管轄を委譲され、独立した dst へ）。
    fs::write(
        parent.join("child/manifest.toml"),
        "dst = \"~/.config/child\"\n",
    )
    .unwrap();
    fs::write(parent.join("child/c.conf"), "c\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success();

    // 親: 直下ファイル + サブディレクトリ再帰（複数ファイル）。
    assert_eq!(
        fs::read_to_string(home.path().join(".config/parent/a.conf")).unwrap(),
        "a\n",
    );
    assert_eq!(
        fs::read_to_string(home.path().join(".config/parent/nested/b.conf")).unwrap(),
        "b\n",
    );
    // 委譲: child は自身の dst へ配置され、親の配下には現れない。
    assert_eq!(
        fs::read_to_string(home.path().join(".config/child/c.conf")).unwrap(),
        "c\n",
    );
    assert!(
        !home.path().join(".config/parent/child").exists(),
        "委譲したサブツリーが親側にも配置された（再帰委譲が効いていない）",
    );
}

/// S1 受け入れ条件: パーミッション属性（private=0600 / executable）。
/// base 0644 を起点に private で 0600、executable で read 桁へ exec を合成する
/// （0644→0755 / 0600→0700）。chezmoi の private_ / executable_ と同じ合成規則。
#[cfg(unix)]
#[rstest]
#[case("plain", "", 0o644)]
#[case("priv", "private = true\n", 0o600)]
#[case("exec", "executable = true\n", 0o755)]
#[case("both", "private = true\nexecutable = true\n", 0o700)]
fn apply_sets_permissions_from_manifest(#[case] name: &str, #[case] attr: &str, #[case] want: u32) {
    use std::os::unix::fs::PermissionsExt;

    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs").join(name);
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        format!("dst = \"~/.config/{name}\"\n{attr}"),
    )
    .unwrap();
    fs::write(unit.join("f.txt"), "x\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success();

    let placed = home.path().join(".config").join(name).join("f.txt");
    let mode = fs::metadata(&placed).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, want, "{name}: パーミッションが期待と異なる");
}
