//! root `dotfiles`（core）bin の E2E テスト（assert_cmd + predicates + rstest）。

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;
use std::fs;
use std::path::Path;

fn dotfiles() -> Command {
    Command::cargo_bin("dotfiles").unwrap()
}

#[rstest]
#[case("--help")]
#[case("-h")]
fn help_flag_succeeds(#[case] flag: &str) {
    dotfiles().arg(flag).assert().success();
}

#[rstest]
#[case("--version")]
#[case("-V")]
fn version_flag_prints_name_and_version(#[case] flag: &str) {
    dotfiles()
        .arg(flag)
        .assert()
        .success()
        .stdout(predicate::str::contains("dotfiles"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn no_args_prints_version() {
    dotfiles()
        .assert()
        .success()
        .stdout(predicate::str::contains("dotfiles"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

/// `dotfiles apply` が固定ソース `configs/` を読み、`manifest.toml`（dst / kind=copy 既定）
/// に従って一時 HOME へ実体を配置することを検証する（S0 / #454 の受け入れ条件）。
/// 実ソースである repo の `configs/zellij` をそのまま使い、configs 化が機能することを確かめる。
#[test]
fn apply_places_real_zellij_config_into_home() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let src = Path::new(repo_root).join("configs/zellij/config.kdl");
    let home = tempfile::tempdir().unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(repo_root)
        .env("HOME", home.path())
        .assert()
        .success();

    let placed = home.path().join(".config/zellij/config.kdl");
    assert!(
        placed.is_file(),
        "zellij config が配置されていない: {placed:?}"
    );
    assert_eq!(
        fs::read_to_string(&placed).unwrap(),
        fs::read_to_string(&src).unwrap(),
        "配置された内容がソースと一致しない",
    );
}

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

/// `configs/` が無い場所で apply するとエラー終了することを検証する。
#[test]
fn apply_errors_when_source_missing() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("が見つかりません"))
        .stderr(predicate::str::contains("探索先:"))
        .stderr(predicate::str::contains("リポジトリのルート"));
}

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

/// 実ソース configs/bat を一時 HOME へ apply し、themes/ サブディレクトリを含めて
/// 再帰配置されることを検証する（実ツールでのサブディレクトリ再帰）。
#[test]
fn apply_places_real_bat_config_with_theme_subdir() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let home = tempfile::tempdir().unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(repo_root)
        .env("HOME", home.path())
        .assert()
        .success();

    for rel in ["config", "themes/ayu-dark.tmTheme"] {
        let placed = home.path().join(".config/bat").join(rel);
        let src = Path::new(repo_root).join("configs/bat").join(rel);
        assert!(
            placed.is_file(),
            "bat の {rel} が配置されていない: {placed:?}"
        );
        assert_eq!(
            fs::read_to_string(&placed).unwrap(),
            fs::read_to_string(&src).unwrap(),
            "bat の {rel} の内容がソースと一致しない",
        );
    }
}

/// `dotfiles list` が実 configs の分散 manifest を集約し、配置先一覧を表示する。
#[test]
fn list_aggregates_real_manifests() {
    dotfiles()
        .arg("list")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .assert()
        .success()
        .stdout(predicate::str::contains("bat"))
        .stdout(predicate::str::contains("nvim"))
        .stdout(predicate::str::contains("zellij"))
        .stdout(predicate::str::contains("~/.config/"))
        .stdout(predicate::str::contains("copy"));
}

/// `dotfiles list` が単位を名前順に並べ、dst と属性ラベルを表示することを検証する。
#[test]
fn list_shows_units_sorted_with_attrs() {
    let work = tempfile::tempdir().unwrap();

    let beta = work.path().join("configs/beta");
    let alpha = work.path().join("configs/alpha");
    fs::create_dir_all(&beta).unwrap();
    fs::create_dir_all(&alpha).unwrap();
    fs::write(beta.join("manifest.toml"), "dst = \"~/.config/beta\"\n").unwrap();
    fs::write(
        alpha.join("manifest.toml"),
        "dst = \"~/.config/alpha\"\nprivate = true\n",
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

    let a = stdout.find("alpha").expect("alpha 行が無い");
    let b = stdout.find("beta").expect("beta 行が無い");
    assert!(a < b, "名前順に並んでいない:\n{stdout}");
    assert!(
        stdout.contains("~/.config/alpha"),
        "dst が出ていない:\n{stdout}"
    );
    assert!(
        stdout.contains("copy, private"),
        "属性ラベルが出ていない:\n{stdout}",
    );
}

/// `configs/` が無い場所で list するとエラー終了することを検証する。
#[test]
fn list_errors_when_source_missing() {
    let work = tempfile::tempdir().unwrap();

    dotfiles()
        .arg("list")
        .current_dir(work.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("が見つかりません"));
}
