//! `dotfiles apply` の合成軸（overlay / strategy / when）の E2E（S3 enabler / #471）。
//!
//! dst=ファイルへ条件付き断片（overlay）を strategy で重ねる挙動と、§5.5 の評価順
//! 不変条件（①ユニット gate 短絡 / ②宣言順 / ③preserve 最後）を hermetic fixture で検証する。
//! when.deps（配列・AND）は PATH 先頭スタブ（[`crate::write_stub`]）の有無で、when.os は現在 OS
//! （chezmoi 互換表記）で gate する。トップレベル when はユニットスコープ、`[[overlay]]` の when は
//! 断片スコープ（同じ語彙）。末尾は overlay/strategy/preserve の不正な組合せを load 時に弾く検証群。

use crate::{dotfiles, write_stub};
use predicates::prelude::*;
use std::fs;
use std::path::Path;

/// 現在の OS を chezmoi 互換表記（macOS=darwin）で返す。`when.os` fixture に埋める。
fn current_os() -> &'static str {
    if cfg!(target_os = "macos") {
        "darwin"
    } else {
        std::env::consts::OS
    }
}

/// concat overlay（base 常時 ＋ rtk 断片 when.deps=["rtk"]）の単位を書き出す。
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
         when = { deps = [\"rtk\"] }\n",
    )
    .unwrap();
    fs::write(unit.join("base.txt"), "BASE\n").unwrap();
    fs::write(unit.join("rtk.txt"), "RTK\n").unwrap();
}

/// when.deps を満たす（rtk が PATH にある）と rtk 断片が宣言順で連結される。
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
        "when.deps を満たす rtk 断片が宣言順で連結されていない",
    );
}

/// when.deps を満たさない（rtk 不在）と rtk 断片だけ脱落し、base は残る（dst は生成される）。
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

/// 不変条件①: トップレベル when.os gate を満たさない単位は丸ごと skip し、dst を一切作らない。
/// gate がユニット共通（copy にも効く）ことも兼ねて確認する。
#[test]
fn apply_unit_os_gate_short_circuits_without_touching_dst() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/maconly");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/maconly\"\nwhen = { os = \"nonsuch-os\" }\n",
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
/// preserve=true で既存 dst を最下層の土台にし、overlay は base → rtk（when.deps）の宣言順で重なる。
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
         when = { deps = [\"rtk\"] }\n",
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
        "when.deps=[\"rtk\"] を満たす断片が重なるべき:\n{out}",
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
        "rtk 不在なら when.deps 断片は脱落するべき:\n{out}",
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
        "dst = \"~/.config/maconly\"\nwhen = { os = \"darwin\" }\n",
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
        .stdout(predicate::str::contains("when.os=darwin"));
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
