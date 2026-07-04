//! `dotfiles apply` の合成軸（overlay / strategy / when）の E2E（S3 enabler / #471）。
//!
//! dst=ファイルへ条件付き断片（overlay）を strategy で重ねる挙動と、§5.5 の評価順
//! 不変条件（①ユニット gate 短絡 / ②宣言順 / ③preserve 最後）を hermetic fixture で検証する。
//! when.deps（配列・AND）は PATH 先頭スタブ（[`crate::write_stub`]）の有無で、when.os は現在 OS
//! （`darwin`/`linux` 表記）で gate する。トップレベル when はユニットスコープ、`[[overlay]]` の when は
//! 断片スコープ（同じ語彙）。末尾は overlay/strategy/preserve の不正な組合せを load 時に弾く検証群。
//!
//! plist-shallow（#531）は、overlay の `cmd` 断片（外部コマンドの標準出力）を土台にする運用
//! （`configs/stats` の実例）を架空ツール `prefctl` で検証する。実 configs を名指ししない契約
//! テストなので `when` は `deps` のみ（`os` gate は付けない＝ Linux CI でも実行される）。

use crate::{dotfiles, write_stub};
use predicates::prelude::*;
use std::fs;
use std::path::Path;

/// 現在の OS を `when.os` 表記（macOS=darwin）で返す。`when.os` fixture に埋める。
fn current_os() -> &'static str {
    if cfg!(target_os = "macos") {
        "darwin"
    } else {
        std::env::consts::OS
    }
}

/// concat overlay（base 常時 ＋ faketool 断片 when.deps=["faketool"]）の単位を書き出す。
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
         src = \"faketool.txt\"\n\
         when = { deps = [\"faketool\"] }\n",
    )
    .unwrap();
    fs::write(unit.join("base.txt"), "BASE\n").unwrap();
    fs::write(unit.join("faketool.txt"), "FRAG\n").unwrap();
}

/// when.deps を満たす（faketool が PATH にある）と断片が宣言順で連結される。
#[cfg(unix)]
#[test]
fn apply_overlay_concat_includes_fragment_when_dep_present() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_stub(bin.path(), "faketool", "exit 0\n"); // 存在＋実行ビットだけで十分（実行はされない）。
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
        "BASE\nFRAG\n",
        "when.deps を満たす断片が宣言順で連結されていない",
    );
}

/// when.deps を満たさない（faketool 不在）と断片だけ脱落し、base は残る（dst は生成される）。
#[cfg(unix)]
#[test]
fn apply_overlay_concat_drops_fragment_when_dep_absent() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let empty_bin = tempfile::tempdir().unwrap(); // faketool を置かない。

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
        "faketool 不在でも base だけで dst が生成されるべき（overlay when=false は断片だけ脱落）",
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

/// アプリの settings.json 相当（json-shallow ＋ preserve）の合成単位を書き出す。
/// preserve=true で既存 dst を最下層の土台にし、overlay は base → faketool（when.deps）の宣言順で重なる。
/// base は dotfiles 所有キー（language / hook）だけを持ち、model 等の非管理キーは定義しない。
fn write_json_shallow_unit(work: &Path) {
    let unit = work.join("configs/app");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/app/settings.json\"\n\
         strategy = \"json-shallow\"\n\
         preserve = true\n\
         [[overlay]]\n\
         src = \"settings.json\"\n\
         [[overlay]]\n\
         src = \"faketool.json\"\n\
         when = { deps = [\"faketool\"] }\n",
    )
    .unwrap();
    // base は dotfiles 所有キー（language=共有値 / hook）を持つ。model は定義しない（=非管理）。
    fs::write(
        unit.join("settings.json"),
        "{\"language\":\"ja\",\"hook\":\"base\"}\n",
    )
    .unwrap();
    fs::write(unit.join("faketool.json"), "{\"faketoolKey\":\"on\"}\n").unwrap();
}

/// json-shallow ＋ preserve: faketool present。既存 dst を土台に非管理キー（model / effortLevel）を
/// 全保持し、dotfiles 所有キー（language）は断片が土台を上書きする（旧 $local + $forced）。
#[cfg(unix)]
#[test]
fn apply_json_shallow_preserves_unmanaged_and_overwrites_owned() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_stub(bin.path(), "faketool", "exit 0\n");
    write_json_shallow_unit(work.path());

    // 既存 dst: model/effortLevel=非管理（保持）、language=dotfiles 所有（断片で上書きされる）。
    let dst = home.path().join(".config/app/settings.json");
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
        out.contains("\"faketoolKey\": \"on\""),
        "when.deps=[\"faketool\"] を満たす断片が重なるべき:\n{out}",
    );
}

/// json-shallow ＋ preserve: faketool 不在でも base＋土台で settings.json は書かれる（回帰解消の核）。
#[cfg(unix)]
#[test]
fn apply_json_shallow_writes_base_without_gated_overlay() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let empty_bin = tempfile::tempdir().unwrap(); // faketool 不在。

    write_json_shallow_unit(work.path());

    let dst = home.path().join(".config/app/settings.json");
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
        !out.contains("faketoolKey"),
        "faketool 不在なら when.deps 断片は脱落するべき:\n{out}",
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

/// plist-shallow 単位（`configs/demo`）を書き出す。base overlay は架空ツール `prefctl` の
/// `cmd`（外部コマンド実行。標準出力を土台にする）、2 つ目の overlay は dotfiles 管理サブセット
/// （`managed.plist`）。hooks は `prefctl import` で合成済み dst を反映する（`configs/stats` の
/// `defaults export`/`defaults import` と同型。#531）。`frequency = "always"`（#546）: 反映対象
/// （ライブなドメイン）は dotfiles 管理外で変化しうるため、onchange（既定）ではなく毎 apply
/// 無条件に反映する。`when` は `deps` のみで `os` は付けない（実 configs の stats 自体は darwin
/// 限定だが、エンジンの契約は OS 非依存に保つ）。
fn write_plist_shallow_overlay_unit(work: &Path) {
    let unit = work.join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.cache/demo/pref.plist\"\n\
         strategy = \"plist-shallow\"\n\
         when = { deps = [\"prefctl\"] }\n\
         [[overlay]]\n\
         cmd = [\"prefctl\", \"export\", \"-\"]\n\
         [[overlay]]\n\
         src = \"managed.plist\"\n\
         [[hooks]]\n\
         cmd = [\"sh\", \"-c\", \"prefctl import \\\"$HOME/.cache/demo/pref.plist\\\"\"]\n\
         frequency = \"always\"\n",
    )
    .unwrap();
    // dotfiles 管理サブセット: Owned キーを true で上書きする（base は false）。
    fs::write(
        unit.join("managed.plist"),
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\"><dict><key>Owned</key><true/></dict></plist>\n",
    )
    .unwrap();
}

/// `prefctl` スタブ: `export -` で「生きたドメイン」相当の XML plist（非管理キー `WindowFrame` ＋
/// owned キーの旧値 `Owned=false`）を標準出力へ、`import <file>` で `$HOME/imported.plist` へ
/// コピーし `$HOME/import-count` へ 1 行追記する（反映の観測点・呼び出し回数の計測点）。
#[cfg(unix)]
fn write_prefctl_stub(bin: &Path) {
    write_stub(
        bin,
        "prefctl",
        r#"if [ "$1" = "export" ]; then
  cat <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>WindowFrame</key>
	<string>0 0 100 200</string>
	<key>Owned</key>
	<false/>
</dict>
</plist>
PLIST
  exit 0
elif [ "$1" = "import" ]; then
  cp "$2" "$HOME/imported.plist"
  printf 'x\n' >> "$HOME/import-count"
  exit 0
fi
exit 1
"#,
    );
}

/// `import-count` マーカーの行数（＝ prefctl import の呼び出し回数）。未作成なら 0。
fn import_count(home: &Path) -> usize {
    fs::read_to_string(home.join("import-count"))
        .map(|s| s.lines().count())
        .unwrap_or(0)
}

/// overlay の `cmd` 断片（外部コマンドの標準出力を土台にする）＋ plist-shallow 合成 ＋
/// `frequency = "always"` hooks 反映が一気通貫で動くことを検証する（#531 の Stats.plist 実装が
/// 拠って立つ経路。テスト品質レビュー指摘の Gap A: 実 configs の `real_configs` は `when.deps`
/// gate で `stats` を skip するため、この経路自体は他のどのテストも通っていなかった）。
///
/// 検証すること: ① base（`prefctl export` の標準出力）の非管理キーが dst に保持される、
/// ② dotfiles 管理サブセットの所有キーが base を上書きする（宣言順・後勝ち）、③ hooks が
/// 合成済み dst をそのまま反映する（reflect が composed output に結線されている）、
/// ④ **ソース（`managed.plist`）不変のまま 2 回目 apply しても hooks が skip されず再度反映される**
/// （#531 で発覚したバグ「配置は毎回正しいが反映は onchange で skip される」の直接の回帰防止。
/// `frequency = "always"` を使わず既定の onchange のままだったら、この assertion は失敗する）。
#[cfg(unix)]
#[test]
fn apply_plist_shallow_overlay_cmd_base_reflects_via_hook() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_prefctl_stub(bin.path());
    write_plist_shallow_overlay_unit(work.path());

    // stub 自身が sh 組み込みでない cat/cp を使うため、PATH は stub dir 専有ではなく
    // /bin:/usr/bin も残す（when.deps gate の対象は prefctl だけなので、実システムの
    // cat/cp/sh が resolve できても gate の意味は変わらない）。
    let path = format!("{}:/bin:/usr/bin", bin.path().display());
    let apply = || {
        dotfiles()
            .arg("apply")
            .current_dir(work.path())
            .env("HOME", home.path())
            .env("PATH", &path)
            .assert()
    };

    apply()
        .success()
        .stdout(predicate::str::contains("overlay/plist-shallow"))
        .stdout(predicate::str::contains("hook:"))
        .stdout(predicate::str::contains("ran (always)"));

    let dst = home.path().join(".cache/demo/pref.plist");
    let merged = plist::Value::from_file(&dst).expect("dst が有効な plist であるべき");
    let dict = merged
        .as_dictionary()
        .expect("dst のトップレベルは dict であるべき");
    assert_eq!(
        dict["WindowFrame"].as_string(),
        Some("0 0 100 200"),
        "base（cmd 断片）の非管理キーが保持されるべき: {dict:?}",
    );
    assert_eq!(
        dict["Owned"].as_boolean(),
        Some(true),
        "dotfiles 管理サブセットが base を上書きするべき（後勝ち）: {dict:?}",
    );

    let imported = home.path().join("imported.plist");
    assert!(
        imported.exists(),
        "hooks の prefctl import が呼ばれていない（反映されていない）",
    );
    assert_eq!(
        fs::read(&dst).unwrap(),
        fs::read(&imported).unwrap(),
        "hooks は合成済み dst をそのまま反映するべき（別内容を渡していない）",
    );
    assert_eq!(import_count(home.path()), 1, "初回は反映されるべき");

    // ソース（managed.plist）不変のまま 2 回目 apply。onchange（既定）なら skip されるはずだが、
    // frequency=always なので再度反映される（#531/#546 の回帰防止）。
    apply()
        .success()
        .stdout(predicate::str::contains("ran (always)"));
    assert_eq!(
        import_count(home.path()),
        2,
        "frequency=always はソース不変でも2回目 apply で再反映されるべき",
    );
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
