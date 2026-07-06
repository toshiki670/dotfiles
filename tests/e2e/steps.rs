//! `dotfiles apply` の `[[steps]]` パイプライン（#588 スライス1）の E2E。
//!
//! 文書 D を input（読む）→ merge（重ねる）→ output（書く）で畳む挙動と、評価順不変条件
//! （①ユニット gate 短絡 ②宣言順 ③2 つ目以降の input は merge・最初の input は土台）を hermetic
//! fixture で検証する。when.deps（配列・AND）は PATH 先頭スタブ（[`crate::write_stub`]）の有無で、
//! when.os は現在 OS（`darwin`/`linux` 表記）で gate する。トップレベル when はユニットスコープ、
//! step の when は step スコープ（同じ語彙）。末尾は steps/merge/format/optional の不正な組合せを
//! load 時に弾く検証群。
//!
//! output.cmd（#560）は、既存宛先を土台に読む input.cmd（外部コマンドの標準出力）を土台にする運用
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

/// text append 単位（base 常時 ＋ faketool 断片 when.deps=["faketool"]）の単位を書き出す。
fn write_text_append_unit(work: &Path) {
    let unit = work.join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "format = \"text\"\n\
         [[steps]]\n\
         input = \"base.txt\"\n\
         [[steps]]\n\
         input = \"faketool.txt\"\n\
         when  = { deps = [\"faketool\"] }\n\
         merge = \"append\"\n\
         [[steps]]\n\
         output = \"~/.config/demo/out.txt\"\n",
    )
    .unwrap();
    fs::write(unit.join("base.txt"), "BASE\n").unwrap();
    fs::write(unit.join("faketool.txt"), "FRAG\n").unwrap();
}

/// when.deps を満たす（faketool が PATH にある）と断片が宣言順で連結される。
#[cfg(unix)]
#[test]
fn apply_text_append_includes_step_when_dep_present() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_stub(bin.path(), "faketool", "exit 0\n"); // 存在＋実行ビットだけで十分（実行はされない）。
    write_text_append_unit(work.path());

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
        "when.deps を満たす step が宣言順で連結されていない",
    );
}

/// when.deps を満たさない（faketool 不在）と該当 step だけ脱落し、最初の input は残る（D は不変）。
#[cfg(unix)]
#[test]
fn apply_text_append_drops_step_when_dep_absent() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let empty_bin = tempfile::tempdir().unwrap(); // faketool を置かない。

    write_text_append_unit(work.path());

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
        "faketool 不在でも最初の input だけで output が生成されるべき（when=false は該当 step だけ脱落）",
    );
}

/// when.os は現在 OS 一致の step だけ採用し、不一致の step は脱落する。
#[test]
fn apply_step_when_os_gates_by_current_os() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        format!(
            "format = \"text\"\n\
             [[steps]]\n\
             input = \"base.txt\"\n\
             [[steps]]\n\
             input = \"here.txt\"\n\
             when  = {{ os = \"{os}\" }}\n\
             merge = \"append\"\n\
             [[steps]]\n\
             input = \"elsewhere.txt\"\n\
             when  = {{ os = \"nonsuch-os\" }}\n\
             merge = \"append\"\n\
             [[steps]]\n\
             output = \"~/.config/demo/out.txt\"\n",
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
        "現在 OS の step だけ採用され、不一致 OS の step は脱落するべき",
    );
}

/// 不変条件①: トップレベル when.os gate を満たさない単位は丸ごと skip し、宛先を一切作らない。
/// gate がユニット共通（ツリーにも効く）ことも兼ねて確認する。
#[test]
fn apply_unit_os_gate_short_circuits_without_touching_dst() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/maconly");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "when = { os = \"nonsuch-os\" }\n[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/.config/maconly\"\n",
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
        "os gate=false でユニット全体が skip されず宛先が作られた",
    );
}

/// アプリの settings.json 相当（json ＋ 宛先の現在内容を土台に読む optional input）の単位を書き出す。
/// 最初の input は宛先自身（optional・初回は無い）、2 つ目以降は shallow で重なる
/// （base → faketool（when.deps）の宣言順）。base は dotfiles 所有キー（language / hook）だけを持ち、
/// model 等の非管理キーは定義しない。
fn write_json_shallow_unit(work: &Path) {
    let unit = work.join("configs/app");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "format = \"json\"\n\
         [[steps]]\n\
         input    = \"~/.config/app/settings.json\"\n\
         optional = true\n\
         [[steps]]\n\
         input = \"settings.json\"\n\
         merge = \"shallow\"\n\
         [[steps]]\n\
         input = \"faketool.json\"\n\
         when  = { deps = [\"faketool\"] }\n\
         merge = \"shallow\"\n\
         [[steps]]\n\
         output = \"~/.config/app/settings.json\"\n",
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

/// json ＋ 宛先読み: faketool present。既存宛先を土台に非管理キー（model / effortLevel）を
/// 全保持し、dotfiles 所有キー（language）は断片が土台を上書きする。
#[cfg(unix)]
#[test]
fn apply_json_shallow_preserves_unmanaged_and_overwrites_owned() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_stub(bin.path(), "faketool", "exit 0\n");
    write_json_shallow_unit(work.path());

    // 既存宛先: model/effortLevel=非管理（保持）、language=dotfiles 所有（断片で上書きされる）。
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
        "settings.json の入力キーが残るべき:\n{out}"
    );
    assert!(
        out.contains("\"faketoolKey\": \"on\""),
        "when.deps=[\"faketool\"] を満たす step が重なるべき:\n{out}",
    );
}

/// json ＋ 宛先読み: faketool 不在でも settings.json 土台で output は書かれる（回帰解消の核）。
#[cfg(unix)]
#[test]
fn apply_json_shallow_writes_base_without_gated_step() {
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
        "settings.json の内容が書かれるべき:\n{out}"
    );
    assert!(
        out.contains("\"model\": \"local\""),
        "非管理キー model が土台のまま保持されるべき:\n{out}",
    );
    assert!(
        !out.contains("faketoolKey"),
        "faketool 不在なら when.deps step は脱落するべき:\n{out}",
    );
}

/// 宛先を土台に読む input が無い（純 dotfiles 所有 json）は既存宛先を無視する（従来挙動）。
#[test]
fn apply_json_without_dst_read_ignores_existing() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/owned");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "[[steps]]\ninput = \"a.json\"\n[[steps]]\noutput = \"~/.config/owned/out.json\"\n",
    )
    .unwrap();
    fs::write(unit.join("a.json"), "{\"k\":\"new\"}\n").unwrap();

    // 既存宛先にローカルキーを置いても、宛先読み step が無ければ土台にされず破棄される。
    let dst = home.path().join(".config/owned/out.json");
    fs::create_dir_all(dst.parent().unwrap()).unwrap();
    fs::write(&dst, "{\"stale\":\"x\"}\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success();

    // 単一 input（merge 無し・format 省略）なので内容は verbatim（json 再直列化を経ない）。
    let out = fs::read_to_string(&dst).unwrap();
    assert_eq!(
        out, "{\"k\":\"new\"}\n",
        "input が verbatim で書かれるべき:\n{out}"
    );
    assert!(
        !out.contains("stale"),
        "宛先読み step が無ければ既存宛先は土台にされないべき:\n{out}",
    );
}

/// 不変条件②（宣言順・後勝ち）: json で後ろの input が同名キーを上書きする。
#[test]
fn apply_json_shallow_later_input_wins() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "format = \"json\"\n\
         [[steps]]\n\
         input = \"a.json\"\n\
         [[steps]]\n\
         input = \"b.json\"\n\
         merge = \"shallow\"\n\
         [[steps]]\n\
         output = \"~/.config/demo/out.json\"\n",
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
        "宣言順で後ろの input が後勝ちすべき:\n{out}",
    );
    assert!(
        out.contains("\"only_a\": 1"),
        "前の input のキーは残るべき:\n{out}"
    );
}

/// `dotfiles list` が steps サマリ・format・when.os 属性を表示する。
#[test]
fn list_shows_steps_summary_format_and_os_attrs() {
    let work = tempfile::tempdir().unwrap();
    write_json_shallow_unit(work.path());

    let os_unit = work.path().join("configs/maconly");
    fs::create_dir_all(&os_unit).unwrap();
    fs::write(
        os_unit.join("manifest.toml"),
        "when = { os = \"darwin\" }\n[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/.config/maconly\"\n",
    )
    .unwrap();

    dotfiles()
        .arg("list")
        .current_dir(work.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("steps=3in/1out, json"))
        .stdout(predicate::str::contains("when.os=darwin"));
}

/// `output.cmd`（#560）単位（`configs/demo`）を書き出す。最初の input は架空ツール `prefctl` の
/// `input.cmd`（外部コマンド実行。標準出力を土台にする）、2 つ目の input は dotfiles 管理サブセット
/// （`managed.plist`）、output は `output.cmd`（合成済み D を標準入力へ渡し、生きたドメインへ反映する）
/// （`configs/stats` の `defaults export`/`defaults import` と同型。#531/#560）。output.cmd は
/// 毎 apply 無条件に実行される（hooks を介さない）。`when` は `deps` のみで `os` は付けない（実
/// configs の stats 自体は darwin 限定だが、エンジンの契約は OS 非依存に保つ）。
fn write_output_cmd_unit(work: &Path) {
    let unit = work.join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "format = \"plist\"\n\
         when   = { deps = [\"prefctl\"] }\n\
         [[steps]]\n\
         input.cmd = [\"prefctl\", \"export\", \"-\"]\n\
         [[steps]]\n\
         input = \"managed.plist\"\n\
         merge = \"shallow\"\n\
         [[steps]]\n\
         output.cmd = [\"prefctl\", \"import\"]\n",
    )
    .unwrap();
    // dotfiles 管理サブセット: Owned キーを true で上書きする（土台は false）。
    fs::write(
        unit.join("managed.plist"),
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\"><dict><key>Owned</key><true/></dict></plist>\n",
    )
    .unwrap();
}

/// `prefctl` スタブ: `export -` で「生きたドメイン」相当の XML plist（非管理キー `WindowFrame` ＋
/// owned キーの旧値 `Owned=false`）を標準出力へ、`import`（引数無し・標準入力から読む）で
/// 標準入力の中身を `$HOME/imported.plist` へ書き、`$HOME/import-count` へ 1 行追記する
/// （反映の観測点・呼び出し回数の計測点）。
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
  cat > "$HOME/imported.plist"
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

/// input.cmd（外部コマンドの標準出力を土台にする）＋ merge=shallow ＋ output.cmd 反映が
/// 一気通貫で動くことを検証する（#531 の Stats.plist 実装 / #560 の output.cmd 移行が拠って立つ経路）。
///
/// 検証すること: ① 土台（`prefctl export` の標準出力）の非管理キーが D に保持される、
/// ② dotfiles 管理サブセットの所有キーが土台を上書きする（宣言順・後勝ち）、③ output.cmd が
/// 合成済み D をそのまま標準入力へ渡す（反映が composed D に結線されている）、④ **ソース
/// （`managed.plist`）不変のまま 2 回目 apply しても output.cmd は毎回実行される**（hooks を介さない
/// cmd output の契約 ― onchange gate を通らず常に実行される）。
#[cfg(unix)]
#[test]
fn apply_output_cmd_reflects_composed_document_every_apply() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_prefctl_stub(bin.path());
    write_output_cmd_unit(work.path());

    // stub 自身が sh 組み込みでない cat を使うため、PATH は stub dir 専有ではなく
    // /bin:/usr/bin も残す（when.deps gate の対象は prefctl だけなので、実システムの
    // cat/sh が resolve できても gate の意味は変わらない）。
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
        .stdout(predicate::str::contains("output=cmd"));

    let imported = home.path().join("imported.plist");
    assert!(
        imported.exists(),
        "output.cmd の prefctl import が呼ばれていない（反映されていない）",
    );
    let merged =
        plist::Value::from_file(&imported).expect("反映された内容が有効な plist であるべき");
    let dict = merged
        .as_dictionary()
        .expect("反映された内容のトップレベルは dict であるべき");
    assert_eq!(
        dict["WindowFrame"].as_string(),
        Some("0 0 100 200"),
        "土台（cmd input）の非管理キーが保持されるべき: {dict:?}",
    );
    assert_eq!(
        dict["Owned"].as_boolean(),
        Some(true),
        "dotfiles 管理サブセットが土台を上書きするべき（後勝ち）: {dict:?}",
    );
    assert_eq!(import_count(home.path()), 1, "初回は反映されるべき");

    // ソース（managed.plist）不変のまま 2 回目 apply。output.cmd は hooks を介さず毎 apply
    // 無条件に実行されるため、onchange のような skip は起きない（#560 の回帰防止）。
    apply().success();
    assert_eq!(
        import_count(home.path()),
        2,
        "output.cmd はソース不変でも 2 回目 apply で再反映されるべき",
    );
}

// ── load 時検証（#588 スライス1で新設した規則の代表例。網羅は manifest.rs の #[cfg(test)]） ──

/// 2 つ目以降の input に `merge` を書かないと load 時にエラー（暗黙の合成規則を持たない）。
#[test]
fn apply_errors_when_second_input_missing_merge() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "format = \"text\"\n[[steps]]\ninput = \"a.txt\"\n[[steps]]\ninput = \"b.txt\"\n[[steps]]\noutput = \"~/.config/demo/out.txt\"\n",
    )
    .unwrap();
    fs::write(unit.join("a.txt"), "A\n").unwrap();
    fs::write(unit.join("b.txt"), "B\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("merge"));
}

/// 最初の input に `merge` を書くと load 時にエラー（最初の input は文書の土台）。
#[test]
fn apply_errors_when_first_input_has_merge() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "format = \"text\"\n[[steps]]\ninput = \"a.txt\"\nmerge = \"append\"\n[[steps]]\noutput = \"~/.config/demo/out.txt\"\n",
    )
    .unwrap();
    fs::write(unit.join("a.txt"), "A\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("最初の input"));
}

/// `merge` を宣言する step があるのに `format` を書かないと load 時にエラー。
#[test]
fn apply_errors_when_merge_without_format() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "[[steps]]\ninput = \"a.txt\"\n[[steps]]\ninput = \"b.txt\"\nmerge = \"append\"\n[[steps]]\noutput = \"~/.config/demo/out.txt\"\n",
    )
    .unwrap();
    fs::write(unit.join("a.txt"), "A\n").unwrap();
    fs::write(unit.join("b.txt"), "B\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("format"));
}

/// `merge` の値が `format` と両立しない（例: text で shallow）と load 時にエラー。
#[test]
fn apply_errors_when_merge_format_mismatch() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "format = \"text\"\n[[steps]]\ninput = \"a.txt\"\n[[steps]]\ninput = \"b.txt\"\nmerge = \"shallow\"\n[[steps]]\noutput = \"~/.config/demo/out.txt\"\n",
    )
    .unwrap();
    fs::write(unit.join("a.txt"), "A\n").unwrap();
    fs::write(unit.join("b.txt"), "B\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("両立しません"));
}

/// `optional` を cmd input に書くと load 時にエラー（パス input のみ有効）。
#[test]
fn apply_errors_when_optional_on_cmd_input() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "[[steps]]\ninput.cmd = [\"foo\"]\noptional = true\n[[steps]]\noutput = \"~/.config/demo/out.txt\"\n",
    )
    .unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("optional"));
}

/// `optional` を output step に書くと load 時にエラー。
#[test]
fn apply_errors_when_optional_on_output() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "[[steps]]\ninput = \"a.txt\"\n[[steps]]\noutput = \"~/.config/demo/out.txt\"\noptional = true\n",
    )
    .unwrap();
    fs::write(unit.join("a.txt"), "A\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("optional"));
}

/// output パスが `~` 起点でない（bare 相対・絶対・`$` 含み）と load 時にエラー（#579）。
#[test]
fn apply_errors_when_output_path_not_tilde_rooted() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "[[steps]]\ninput = \"a.txt\"\n[[steps]]\noutput = \"/etc/demo\"\n",
    )
    .unwrap();
    fs::write(unit.join("a.txt"), "A\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("output パス"));
}

/// input パスが絶対パスだと load 時にエラー（単位相対 or ~ 起点のみ許容・#579）。
#[test]
fn apply_errors_when_input_path_is_absolute() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "[[steps]]\ninput = \"/etc/passwd\"\n[[steps]]\noutput = \"~/.config/demo\"\n",
    )
    .unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("絶対パス"));
}
