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

// --- generate 層（S2 / #456） --------------------------------------------
//
// 実バイナリ（gh/bat 等）に依存せず、PATH 先頭に置いたスタブで `cmd` 実行と
// deps gate を検証する。スタブは sh スクリプトなので unix 限定。

/// PATH に置く実行可能スタブを書き出す（固定テキストを stdout に出す）。
#[cfg(unix)]
fn write_stub(dir: &Path, name: &str, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
}

/// generate 単位 `configs/foo/completion`（dst=ファイル / cmd=foo / deps=foo）を書き出す。
#[cfg(unix)]
fn write_generate_unit(work: &Path) -> std::path::PathBuf {
    let unit = work.join("configs/foo/completion");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/fish/completions/foo.fish\"\n\
         kind = \"generate\"\n\
         cmd = [\"foo\"]\n\
         deps = [\"foo\"]\n",
    )
    .unwrap();
    unit
}

/// kind=generate が `cmd` を実行し、その標準出力を dst のファイルへ書き出すことを検証する。
#[cfg(unix)]
#[test]
fn apply_generate_runs_cmd_and_writes_output() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_stub(bin.path(), "foo", "printf 'complete -c foo -f\\n'\n");
    write_generate_unit(work.path());

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", bin.path()) // スタブのみ。foo は PATH 上で解決される。
        .assert()
        .success();

    let placed = home.path().join(".config/fish/completions/foo.fish");
    assert_eq!(
        fs::read_to_string(&placed).unwrap(),
        "complete -c foo -f\n",
        "cmd の stdout がそのまま dst に書かれていない",
    );
}

/// deps gate: 依存バイナリが PATH に無ければ生成をスキップし、ファイルを作らない（成功終了）。
#[cfg(unix)]
#[test]
fn apply_generate_gate_skips_when_dep_missing() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let empty_bin = tempfile::tempdir().unwrap(); // foo を置かない＝依存欠落。

    write_generate_unit(work.path());

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", empty_bin.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("skip"))
        .stdout(predicate::str::contains("foo"));

    assert!(
        !home
            .path()
            .join(".config/fish/completions/foo.fish")
            .exists(),
        "gate が効かず依存欠落でも生成された",
    );
}

/// generate は単位内の `manifest.toml` 以外のファイル（gh の独自補完ブロック相当）を
/// 生成物の後ろへ連結する。
#[cfg(unix)]
#[test]
fn apply_generate_appends_sibling_files() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_stub(bin.path(), "foo", "printf 'GENERATED\\n'\n");
    let unit = write_generate_unit(work.path());
    fs::write(unit.join("custom.fish"), "# CUSTOM\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", bin.path())
        .assert()
        .success();

    let placed = home.path().join(".config/fish/completions/foo.fish");
    assert_eq!(
        fs::read_to_string(&placed).unwrap(),
        "GENERATED\n# CUSTOM\n",
        "生成物の後ろへ sibling が連結されていない",
    );
}

/// generate で `cmd` が無い manifest はエラー終了する。
#[test]
fn apply_generate_without_cmd_errors() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/foo/completion");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/fish/completions/foo.fish\"\nkind = \"generate\"\n",
    )
    .unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("cmd が必要"));
}

// --- merge 層（S3 / #457） ------------------------------------------------
//
// kind=merge は base JSON を土台に、既存 dst から preserve キーだけを温存する
// shallow merge。共有キー（hooks/statusLine/language 等）は base が常に勝つ。

/// merge 単位 `configs/claude/settings`（dst=ファイル / preserve=model,effortLevel）を
/// `work` 下に書き出す。base ファイル名は任意（merge は `manifest.toml` 以外の唯一の実ファイルを
/// base とする）なので、ここではあえて `base.json` という名で置き、名前非依存を兼ねて確認する。
fn write_merge_unit(work: &Path, base_json: &str) -> std::path::PathBuf {
    let unit = work.join("configs/claude/settings");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.claude/settings.json\"\n\
         kind = \"merge\"\n\
         preserve = [\"model\", \"effortLevel\"]\n",
    )
    .unwrap();
    fs::write(unit.join("base.json"), base_json).unwrap();
    unit
}

/// dst の settings.json を読み JSON に parse する（テスト用ヘルパ）。
fn read_settings(home: &Path) -> serde_json::Value {
    let placed = home.join(".claude/settings.json");
    let text = fs::read_to_string(&placed).unwrap_or_else(|e| panic!("{placed:?}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{placed:?} が JSON でない: {e}"))
}

/// 既存 settings.json があるとき、merge が preserve キー（model/effortLevel）を温存しつつ
/// 共有キー（language/hooks）を base で上書きすることを検証する（受け入れ条件の核）。
#[test]
fn apply_merge_preserves_local_keys_and_overrides_shared() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    write_merge_unit(
        work.path(),
        r#"{"language": "日本語", "hooks": {"PreToolUse": []}}"#,
    );

    // 既存 dst: ローカル編集（model/effortLevel）＋ 共有キーの古い値（language/hooks）。
    let dst = home.path().join(".claude/settings.json");
    fs::create_dir_all(dst.parent().unwrap()).unwrap();
    fs::write(
        &dst,
        r#"{"model": "opus", "effortLevel": "high", "language": "English", "hooks": {"PreToolUse": [{"old": true}]}}"#,
    )
    .unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("merge"));

    let got = read_settings(home.path());
    // preserve: ローカルが勝つ。
    assert_eq!(got["model"], "opus", "model が温存されていない");
    assert_eq!(got["effortLevel"], "high", "effortLevel が温存されていない");
    // 共有キー: base が勝つ（dotfiles 上書き）。
    assert_eq!(
        got["language"], "日本語",
        "共有キー language が上書きされていない"
    );
    assert_eq!(
        got["hooks"]["PreToolUse"],
        serde_json::json!([]),
        "共有キー hooks が base で置き換わっていない（shallow merge）",
    );
}

/// 既存 dst が無い初回 apply では、base がそのまま書き出される（preserve は no-op）。
#[test]
fn apply_merge_writes_base_when_dst_absent() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    write_merge_unit(work.path(), r#"{"language": "日本語", "theme": "auto"}"#);

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success();

    let got = read_settings(home.path());
    assert_eq!(got["language"], "日本語");
    assert_eq!(got["theme"], "auto");
    assert!(
        got.get("model").is_none(),
        "存在しない preserve キーが現れた"
    );
}

/// preserve に無いローカル固有キーは温存されない（base 土台＝明示 allowlist の意味論）。
#[test]
fn apply_merge_drops_local_keys_not_in_preserve() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    write_merge_unit(work.path(), r#"{"language": "日本語"}"#);

    let dst = home.path().join(".claude/settings.json");
    fs::create_dir_all(dst.parent().unwrap()).unwrap();
    // model は preserve に在り温存される。localOnly は preserve 外なので落ちる。
    fs::write(&dst, r#"{"model": "opus", "localOnly": "x"}"#).unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success();

    let got = read_settings(home.path());
    assert_eq!(got["model"], "opus", "preserve キーが温存されていない");
    assert!(
        got.get("localOnly").is_none(),
        "preserve 外のローカルキーが温存された（allowlist が効いていない）",
    );
}

/// 実ソース configs/claude/settings を一時 HOME へ apply し、共有設定（hooks/statusLine 等）が
/// 配置されることを検証する（実ツールでの merge）。
#[test]
fn apply_places_real_claude_settings() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let home = tempfile::tempdir().unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(repo_root)
        .env("HOME", home.path())
        .assert()
        .success();

    let got = read_settings(home.path());
    assert_eq!(got["language"], "日本語", "共有設定が配置されていない");
    assert!(
        got["statusLine"]["command"].is_string(),
        "statusLine が配置されていない",
    );
    assert!(
        got["hooks"]["PreToolUse"].is_array(),
        "hooks が配置されていない"
    );
}

/// base ファイルが無い merge 単位はエラー終了する。
#[test]
fn apply_merge_without_base_errors() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/claude/settings");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.claude/settings.json\"\nkind = \"merge\"\n",
    )
    .unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("base ファイルが必要"));
}

/// `dotfiles list` が merge 単位を merge ＋ preserve 付きで表示することを検証する。
#[test]
fn list_shows_merge_kind_with_preserve() {
    let work = tempfile::tempdir().unwrap();
    write_merge_unit(work.path(), "{}");

    dotfiles()
        .arg("list")
        .current_dir(work.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("merge"))
        .stdout(predicate::str::contains("preserve=model+effortLevel"));
}

/// `dotfiles list` が generate 単位を generate ＋ deps 付きで表示することを検証する。
#[test]
fn list_shows_generate_kind_with_deps() {
    let work = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/foo/completion");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/fish/completions/foo.fish\"\n\
         kind = \"generate\"\n\
         cmd = [\"foo\"]\n\
         deps = [\"foo\"]\n",
    )
    .unwrap();

    dotfiles()
        .arg("list")
        .current_dir(work.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("generate"))
        .stdout(predicate::str::contains("deps=foo"));
}
