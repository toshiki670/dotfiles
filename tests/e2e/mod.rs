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
        .stdout(predicate::str::contains("claude/settings"))
        .stdout(predicate::str::contains("~/.config/"))
        .stdout(predicate::str::contains("~/.claude/settings.json"))
        .stdout(predicate::str::contains("copy"))
        .stdout(predicate::str::contains("json-shallow"));
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

// --- 合成軸: overlay / strategy / when（S3 enabler / #471） -----------------
//
// dst=ファイルへ条件付き断片（overlay）を strategy で重ねる挙動と、§5.5 の評価順
// 不変条件（①ユニット gate 短絡 / ②宣言順 / ③preserve 最後）を hermetic fixture で検証する。
// when.dep は PATH 先頭スタブの有無で、when.os は現在 OS（chezmoi 互換表記）で gate する。

/// 現在の OS を chezmoi 互換表記（macOS=darwin）で返す。`when.os` fixture に埋める。
fn current_os() -> &'static str {
    if cfg!(target_os = "macos") {
        "darwin"
    } else {
        std::env::consts::OS
    }
}

/// concat overlay（base 常時 ＋ rtk 断片 when.dep=rtk）の単位を書き出す。
fn write_concat_overlay_unit(work: &Path) {
    let unit = work.join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/demo/out.txt\"\n\
         strategy = \"concat\"\n\
         [[overlay]]\n\
         src = \"base.txt\"\n\
         [[overlay]]\n\
         src = \"rtk.txt\"\n\
         when = { dep = \"rtk\" }\n",
    )
    .unwrap();
    fs::write(unit.join("base.txt"), "BASE\n").unwrap();
    fs::write(unit.join("rtk.txt"), "RTK\n").unwrap();
}

/// when.dep を満たす（rtk が PATH にある）と rtk 断片が宣言順で連結される。
#[cfg(unix)]
#[test]
fn apply_overlay_concat_includes_fragment_when_dep_present() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_stub(bin.path(), "rtk", "exit 0\n"); // 存在＋実行ビットだけで十分（実行はされない）。
    write_concat_overlay_unit(work.path());

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", bin.path())
        .assert()
        .success();

    let placed = home.path().join(".config/demo/out.txt");
    assert_eq!(
        fs::read_to_string(&placed).unwrap(),
        "BASE\nRTK\n",
        "when.dep を満たす rtk 断片が宣言順で連結されていない",
    );
}

/// when.dep を満たさない（rtk 不在）と rtk 断片だけ脱落し、base は残る（dst は生成される）。
#[cfg(unix)]
#[test]
fn apply_overlay_concat_drops_fragment_when_dep_absent() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let empty_bin = tempfile::tempdir().unwrap(); // rtk を置かない。

    write_concat_overlay_unit(work.path());

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", empty_bin.path())
        .assert()
        .success();

    let placed = home.path().join(".config/demo/out.txt");
    assert_eq!(
        fs::read_to_string(&placed).unwrap(),
        "BASE\n",
        "rtk 不在でも base だけで dst が生成されるべき（overlay when=false は断片だけ脱落）",
    );
}

/// when.os は現在 OS 一致の断片だけ採用し、不一致の断片は脱落する。
#[test]
fn apply_overlay_when_os_gates_by_current_os() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        format!(
            "dst = \"~/.config/demo/out.txt\"\n\
             strategy = \"concat\"\n\
             [[overlay]]\n\
             src = \"base.txt\"\n\
             [[overlay]]\n\
             src = \"here.txt\"\n\
             when = {{ os = \"{os}\" }}\n\
             [[overlay]]\n\
             src = \"elsewhere.txt\"\n\
             when = {{ os = \"nonsuch-os\" }}\n",
            os = current_os(),
        ),
    )
    .unwrap();
    fs::write(unit.join("base.txt"), "BASE\n").unwrap();
    fs::write(unit.join("here.txt"), "HERE\n").unwrap();
    fs::write(unit.join("elsewhere.txt"), "ELSEWHERE\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success();

    let placed = home.path().join(".config/demo/out.txt");
    assert_eq!(
        fs::read_to_string(&placed).unwrap(),
        "BASE\nHERE\n",
        "現在 OS の断片だけ採用され、不一致 OS の断片は脱落するべき",
    );
}

/// 不変条件①: ユニット os gate を満たさない単位は丸ごと skip し、dst を一切作らない。
/// gate がユニット共通（copy にも効く）ことも兼ねて確認する。
#[test]
fn apply_unit_os_gate_short_circuits_without_touching_dst() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/maconly");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/maconly\"\nos = \"nonsuch-os\"\n",
    )
    .unwrap();
    fs::write(unit.join("f.txt"), "x\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("skip"))
        .stdout(predicate::str::contains("maconly"));

    assert!(
        !home.path().join(".config/maconly").exists(),
        "os gate=false でユニット全体が skip されず dst が作られた",
    );
}

/// claude/settings.json 相当（json-shallow ＋ preserve）の合成単位を書き出す。
/// preserve=true で既存 dst を最下層の土台にし、overlay は base → rtk（when.dep）の宣言順で重なる。
/// base は dotfiles 所有キー（language / hook）だけを持ち、model 等の非管理キーは定義しない。
fn write_json_shallow_unit(work: &Path) {
    let unit = work.join("configs/claude/settings");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.claude/settings.json\"\n\
         strategy = \"json-shallow\"\n\
         preserve = true\n\
         [[overlay]]\n\
         src = \"settings.json\"\n\
         [[overlay]]\n\
         src = \"rtk.json\"\n\
         when = { dep = \"rtk\" }\n",
    )
    .unwrap();
    // base は dotfiles 所有キー（language=共有値 / hook）を持つ。model は定義しない（=非管理）。
    fs::write(
        unit.join("settings.json"),
        "{\"language\":\"ja\",\"hook\":\"base\"}\n",
    )
    .unwrap();
    fs::write(unit.join("rtk.json"), "{\"rtkHook\":\"on\"}\n").unwrap();
}

/// json-shallow ＋ preserve: rtk present。既存 dst を土台に非管理キー（model / effortLevel）を
/// 全保持し、dotfiles 所有キー（language）は断片が土台を上書きする（旧 $local + $forced）。
#[cfg(unix)]
#[test]
fn apply_json_shallow_preserves_unmanaged_and_overwrites_owned() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_stub(bin.path(), "rtk", "exit 0\n");
    write_json_shallow_unit(work.path());

    // 既存 dst: model/effortLevel=非管理（保持）、language=dotfiles 所有（断片で上書きされる）。
    let dst = home.path().join(".claude/settings.json");
    fs::create_dir_all(dst.parent().unwrap()).unwrap();
    fs::write(
        &dst,
        "{\"model\":\"local\",\"effortLevel\":\"high\",\"language\":\"en\"}\n",
    )
    .unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", bin.path())
        .assert()
        .success();

    let out = fs::read_to_string(&dst).unwrap();
    assert!(
        out.contains("\"model\": \"local\""),
        "dotfiles 非管理キー model は土台のまま保持されるべき:\n{out}",
    );
    assert!(
        out.contains("\"effortLevel\": \"high\""),
        "dotfiles 非管理キー effortLevel は土台のまま保持されるべき:\n{out}",
    );
    assert!(
        out.contains("\"language\": \"ja\""),
        "dotfiles 所有キー language は断片が土台を上書きすべき:\n{out}",
    );
    assert!(
        out.contains("\"hook\": \"base\""),
        "base のキーが残るべき:\n{out}"
    );
    assert!(
        out.contains("\"rtkHook\": \"on\""),
        "when.dep=rtk を満たす断片が重なるべき:\n{out}",
    );
}

/// json-shallow ＋ preserve: rtk 不在でも base＋土台で settings.json は書かれる（回帰解消の核）。
#[cfg(unix)]
#[test]
fn apply_json_shallow_writes_base_without_gated_overlay() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let empty_bin = tempfile::tempdir().unwrap(); // rtk 不在。

    write_json_shallow_unit(work.path());

    let dst = home.path().join(".claude/settings.json");
    fs::create_dir_all(dst.parent().unwrap()).unwrap();
    fs::write(&dst, "{\"model\":\"local\"}\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", empty_bin.path())
        .assert()
        .success();

    let out = fs::read_to_string(&dst).unwrap();
    assert!(
        out.contains("\"hook\": \"base\""),
        "base が書かれるべき:\n{out}"
    );
    assert!(
        out.contains("\"model\": \"local\""),
        "非管理キー model が土台のまま保持されるべき:\n{out}",
    );
    assert!(
        !out.contains("rtkHook"),
        "rtk 不在なら when.dep 断片は脱落するべき:\n{out}",
    );
}

/// preserve 無しの json-shallow は既存 dst を土台にしない（純 dotfiles 所有 json は従来挙動）。
#[test]
fn apply_json_shallow_without_preserve_ignores_existing() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/owned");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/owned/out.json\"\n\
         strategy = \"json-shallow\"\n\
         [[overlay]]\n\
         src = \"a.json\"\n",
    )
    .unwrap();
    fs::write(unit.join("a.json"), "{\"k\":\"new\"}\n").unwrap();

    // 既存 dst にローカルキーを置いても、preserve 無しなら土台にされず破棄される。
    let dst = home.path().join(".config/owned/out.json");
    fs::create_dir_all(dst.parent().unwrap()).unwrap();
    fs::write(&dst, "{\"stale\":\"x\"}\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success();

    let out = fs::read_to_string(&dst).unwrap();
    assert!(out.contains("\"k\": \"new\""), "断片が書かれるべき:\n{out}");
    assert!(
        !out.contains("stale"),
        "preserve 無しなら既存 dst は土台にされないべき:\n{out}",
    );
}

/// 不変条件②（宣言順・後勝ち）: json-shallow で後ろの overlay が同名キーを上書きする。
#[test]
fn apply_json_shallow_later_overlay_wins() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/demo/out.json\"\n\
         strategy = \"json-shallow\"\n\
         [[overlay]]\n\
         src = \"a.json\"\n\
         [[overlay]]\n\
         src = \"b.json\"\n",
    )
    .unwrap();
    fs::write(unit.join("a.json"), "{\"k\":\"first\",\"only_a\":1}\n").unwrap();
    fs::write(unit.join("b.json"), "{\"k\":\"second\"}\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success();

    let out = fs::read_to_string(home.path().join(".config/demo/out.json")).unwrap();
    assert!(
        out.contains("\"k\": \"second\""),
        "宣言順で後ろの overlay が後勝ちすべき:\n{out}",
    );
    assert!(
        out.contains("\"only_a\": 1"),
        "前の overlay のキーは残るべき:\n{out}"
    );
}

/// `dotfiles list` が overlay/strategy/os 属性を表示する。
#[test]
fn list_shows_overlay_strategy_and_os_attrs() {
    let work = tempfile::tempdir().unwrap();
    write_json_shallow_unit(work.path());

    let os_unit = work.path().join("configs/maconly");
    fs::create_dir_all(&os_unit).unwrap();
    fs::write(
        os_unit.join("manifest.toml"),
        "dst = \"~/.config/maconly\"\nos = \"darwin\"\n",
    )
    .unwrap();

    dotfiles()
        .arg("list")
        .current_dir(work.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("json-shallow"))
        .stdout(predicate::str::contains("overlay=2"))
        .stdout(predicate::str::contains("preserve"))
        .stdout(predicate::str::contains("os=darwin"));
}

/// overlay を明示しながら strategy を省略すると load 時にエラー（暗黙 concat を許さない）。
#[test]
fn apply_errors_when_overlay_without_strategy() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/demo/out.txt\"\n[[overlay]]\nsrc = \"a.txt\"\n",
    )
    .unwrap();
    fs::write(unit.join("a.txt"), "A\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("strategy"));
}

/// overlay に src/cmd を 2 つ以上書くと load 時にエラー（黙殺される typo を弾く）。
#[test]
fn apply_errors_when_overlay_mixes_kinds() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/demo/out.json\"\n\
         strategy = \"json-shallow\"\n\
         [[overlay]]\n\
         src = \"a.json\"\n\
         cmd = [\"echo\"]\n",
    )
    .unwrap();
    fs::write(unit.join("a.json"), "{}\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("ちょうど 1 つ"));
}

/// preserve = true を json-shallow 以外と併記すると load 時にエラー（typo を黙殺しない）。
#[test]
fn apply_errors_when_preserve_without_json_shallow() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/demo/out.txt\"\n\
         strategy = \"concat\"\n\
         preserve = true\n\
         [[overlay]]\n\
         src = \"a.txt\"\n",
    )
    .unwrap();
    fs::write(unit.join("a.txt"), "A\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("preserve"));
}

/// preserve = true ＋ json-shallow でも overlay 無し ＋ kind=copy（既定）なら load 時にエラー。
/// この構成は compose 経路に入らず preserve が黙って無視されるため、配置前に弾いて固定する。
#[test]
fn apply_errors_when_preserve_without_compose_routing() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/demo/out.json\"\n\
         strategy = \"json-shallow\"\n\
         preserve = true\n",
    )
    .unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("preserve"));
}

// --- claude/settings 実 config（S3 / #457） --------------------------------
//
// 実ソース configs/claude/settings を一時 work へ隔離コピーし、PATH で rtk の有無を
// 決定的に与えて apply する。json-shallow overlay（base ＋ rtk(when.dep) ＋ preserve=true）が
// 実ファイルで「ローカル（非管理キー）全温存・共有上書き・rtk 条件付き hook」を満たすことを
// 検証する（overlay/strategy/when/preserve の純機構は上の hermetic 群が網羅。ここは実 config の
// 結線確認）。rtk スタブ／PATH 制御は unix 限定。

/// 実ソース configs/claude/settings の全ファイルを work/configs/claude/settings へコピーする。
#[cfg(unix)]
fn copy_real_claude_settings(work: &Path) {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("configs/claude/settings");
    let dst = work.join("configs/claude/settings");
    fs::create_dir_all(&dst).unwrap();
    for entry in fs::read_dir(&src).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            fs::copy(entry.path(), dst.join(entry.file_name())).unwrap();
        }
    }
}

/// 既存 ~/.claude/settings.json を用意する（ローカル編集を模す）。
#[cfg(unix)]
fn write_existing_settings(home: &Path, body: &str) {
    let dst = home.join(".claude/settings.json");
    fs::create_dir_all(dst.parent().unwrap()).unwrap();
    fs::write(&dst, body).unwrap();
}

/// rtk 在: ローカルの非管理キー（model / effortLevel / 任意の localOnly）を全温存しつつ共有
/// （language）を base で上書きし、when.dep=rtk を満たして rtk hook が入る（旧 `$local + $forced`）。
#[cfg(unix)]
#[test]
fn apply_real_claude_settings_preserves_local_and_overrides_with_rtk() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    copy_real_claude_settings(work.path());
    write_stub(bin.path(), "rtk", "exit 0\n"); // 存在＋実行ビットだけで十分（実行はされない）。
    write_existing_settings(
        home.path(),
        "{\"model\":\"opus\",\"effortLevel\":\"high\",\"language\":\"English\",\"localOnly\":\"keepme\"}\n",
    );

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", bin.path())
        .assert()
        .success();

    let out = fs::read_to_string(home.path().join(".claude/settings.json")).unwrap();
    assert!(
        out.contains("\"model\": \"opus\""),
        "ローカル model が温存されるべき:\n{out}"
    );
    assert!(
        out.contains("\"effortLevel\": \"high\""),
        "ローカル effortLevel が温存されるべき:\n{out}"
    );
    assert!(
        out.contains("\"language\": \"日本語\""),
        "共有 language が base で上書きされるべき:\n{out}"
    );
    assert!(
        out.contains("\"localOnly\": \"keepme\""),
        "preserve=true は dotfiles 非管理キー localOnly も土台のまま全温存すべき:\n{out}"
    );
    assert!(
        out.contains("rtk hook claude"),
        "rtk 在で when.dep=rtk 断片の rtk hook が入るべき:\n{out}"
    );
    assert!(
        out.contains("rm guard (trash)"),
        "rm→trash ガードは常に入るべき:\n{out}"
    );
}

/// rtk 不在: rtk hook は脱落するが、base＋preserve で settings.json は書かれる
/// （rm→trash ガード・共有設定・ローカル温存は残る ＝ 無条件化の回帰解消の核）。
#[cfg(unix)]
#[test]
fn apply_real_claude_settings_omits_rtk_hook_when_absent() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let empty_bin = tempfile::tempdir().unwrap(); // rtk を置かない。

    copy_real_claude_settings(work.path());
    write_existing_settings(home.path(), "{\"model\":\"opus\"}\n");

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", empty_bin.path())
        .assert()
        .success();

    let out = fs::read_to_string(home.path().join(".claude/settings.json")).unwrap();
    assert!(
        !out.contains("rtk hook claude"),
        "rtk 不在なら when.dep 断片は脱落し rtk hook は入らないべき:\n{out}"
    );
    assert!(
        out.contains("rm guard (trash)"),
        "rtk 不在でも rm→trash ガードは残るべき:\n{out}"
    );
    assert!(
        out.contains("\"language\": \"日本語\""),
        "rtk 不在でも base の共有設定が書かれるべき:\n{out}"
    );
    assert!(
        out.contains("\"model\": \"opus\""),
        "rtk 不在でも preserve（ローカル温存）は効くべき:\n{out}"
    );
}
