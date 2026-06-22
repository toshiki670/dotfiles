//! `manifest.toml` のスキーマと読み込み。
//!
//! 設計書（docs/dotfiles-native-design.md §5 / §5.5 / §6.2 / §7）の **2軸モデル**を解釈する:
//! - **生成方式 `kind`**（断片をどう実体化するか）= `copy` / `generate`（省略時 copy）。
//! - **合成 `strategy`**（複数の条件付き断片を1 dst=ファイルへどう重ねるか）= `concat` /
//!   `json-shallow`。`merge` は独立 kind ではなく合成軸の JSON 戦略（§5.5）。
//! - **条件付き overlay**（`[[overlay]]` ＋ `when`）= dst を「base ＋ gate された断片」の合成
//!   として組む。各 overlay は `src`（copy 断片）/ `cmd`（generate 断片）のどちらか ＋
//!   `when`（`dep` / `os`）。既存 dst の温存はユニット属性 `preserve = true`（§5.5）。
//!
//! `deps` / `os` はユニット単位 gate（＝ ユニット全体に係る `when` の退化形, §5.5）。
//! `locals` / `sensitive` はマシンローカル値（named value）の宣言（§9, S4）。`hooks` は onchange
//! フック（§13, S5）の**コマンド（argv）**宣言で、各 argv が非空であることを load 時に検証する
//! （実行は [`crate::hooks`] の汎用エンジン。ツール固有ロジックは binary でなく manifest が持つ）。
//! theme は後続（color）スライスで追加する。

use serde::Deserialize;
use std::path::Path;

/// 1 つの設定単位（`manifest.toml` を持つディレクトリ）の配置仕様。
#[derive(Debug, Deserialize)]
pub struct Manifest {
    /// 配置先（必須）。`~` は HOME に展開する。
    /// copy は実体を置くディレクトリ、generate / 合成は生成物を書き出すファイルパス。
    pub dst: String,
    /// 生成方式（省略時 = copy）。断片をどう実体化するか（copy / generate）。
    #[serde(default)]
    pub kind: Kind,
    /// 合成戦略（複数 overlay を1 dst=ファイルへ重ねるとき）。単一 overlay なら省略。
    /// generate の既定挙動（cmd 出力＋sibling 連結）は暗黙 `concat`。
    #[serde(default)]
    pub strategy: Option<Strategy>,
    /// 既存 dst を `json-shallow` の最下層（土台）として温存する（§5.5）。`true` で dotfiles が
    /// 定義しないトップレベルキー（例 `model` / `effortLevel` や任意のローカル固有キー）を
    /// 全保持し、dotfiles 所有キーだけ断片で上書きする（旧 `jq '$local + $forced'` と同値）。
    /// `json-shallow` 専用（他 strategy・省略との併記は load 時エラー）。省略時 false。
    #[serde(default)]
    pub preserve: bool,
    /// 所有者のみアクセス可（chezmoi `private_` = 0600 相当）。省略時 false。
    #[serde(default)]
    pub private: bool,
    /// 実行ビットを付与（chezmoi `executable_` 相当。0644→0755 / 0600→0700）。省略時 false。
    #[serde(default)]
    pub executable: bool,
    /// generate のとき実行するコマンド（argv）。先頭が実行ファイル名、以降が引数。
    /// 標準出力を断片とする。copy では未使用。
    #[serde(default)]
    pub cmd: Vec<String>,
    /// 依存バイナリ（ユニット単位 gate, §7）。PATH に揃わないものがあればユニット全体を
    /// スキップする（＝ ユニット全体に係る `when.dep` の退化形）。
    #[serde(default)]
    pub deps: Vec<String>,
    /// OS 条件（ユニット単位 gate, §7）。chezmoi 互換表記（例 `darwin` / `linux`）。
    /// 不一致ならユニット全体をスキップする（＝ ユニット全体に係る `when.os` の退化形）。
    #[serde(default)]
    pub os: Option<String>,
    /// 合成 overlay（条件付き断片の配列, §5.5）。空 = 生成方式の既定挙動。
    /// 各 overlay は `src` / `cmd` のどちらか ＋ `when?` を持つ。
    #[serde(default)]
    pub overlay: Vec<Overlay>,
    /// マシンローカル値（named value）の宣言（§9）。ここで宣言した名前 `n` に対し、この単位の
    /// 配置ファイル中の `@@n@@` をストア（`~/.config/dotfiles/local.toml`）の値で置換する。
    /// 置換は `locals` を宣言した単位のファイルにだけ走る（無関係ファイルの `@@…@@` を巻き込まない）。
    /// 例: `locals = ["git.email", "git.name"]`。
    #[serde(default)]
    pub locals: Vec<String>,
    /// `locals` のうち秘匿値（対話取得時にエコー/ログを抑制する）。git の email/name は commit に
    /// 載るため**非 sensitive**。`sensitive ⊆ locals` を load 時に検証する（§9.1）— typo で名前が
    /// `locals` とズレると非エコー抑制が黙って効かず秘匿値が漏れる footgun を防ぐ。
    #[serde(default)]
    pub sensitive: Vec<String>,
    /// onchange フック（§13, S5）。このユニットの配置後（after フェーズ）に、ユニットのソースが
    /// 前回適用時から変わっていれば実行する**コマンド（argv）の配列**（例
    /// `hooks = [["bat", "cache", "--build"]]`）。ツール固有ロジックは binary に持たず、実行する
    /// コマンドをデータとして宣言する（[`crate::generate`] の `cmd` と同思想）→ 新ツールのフック
    /// 追加に binary 変更は不要・configs と疎結合。各 argv が非空であることを load 時に検証する。
    /// ユニット gate（`deps` / `os`）が false のユニットは配置ごと skip されるため hooks も走らない
    /// （＝ os 属性でフックを分岐できる, §5.5 不変条件①）。
    #[serde(default)]
    pub hooks: Vec<Vec<String>>,
}

/// 生成方式（断片の実体化方法）。copy / generate。`merge` は kind ではなく `strategy`（§5.5）。
#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    #[default]
    Copy,
    Generate,
}

/// 合成戦略（複数断片を1 dst=ファイルへ重ねる方法, §5.5）。
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Strategy {
    /// テキスト連結（後ろへ連結）。境目に改行を 1 つ補う。
    Concat,
    /// JSON のトップレベル shallow merge（後勝ち）。deep merge はしない。
    JsonShallow,
}

/// 1 つの overlay（条件付き断片, §5.5）。`when` を満たす時だけ合成に参加する。
/// 断片の実体化方法は `src`（copy）/ `cmd`（generate）の択一。既存 dst の温存は overlay では
/// なくユニット属性 [`Manifest::preserve`]。
#[derive(Debug, Deserialize)]
pub struct Overlay {
    /// copy 断片: 単位ディレクトリからの相対ファイル。内容をそのまま断片にする。
    #[serde(default)]
    pub src: Option<String>,
    /// generate 断片: 実行する argv。標準出力を断片にする。
    #[serde(default)]
    pub cmd: Vec<String>,
    /// 採用条件（省略 = 常時採用）。`dep` / `os` を AND で評価する。
    #[serde(default)]
    pub when: Option<When>,
}

/// overlay の採用条件（§5.5）。複数キーは AND（全て満たす時だけ採用）。
#[derive(Debug, Deserialize, Default)]
pub struct When {
    /// 依存バイナリが PATH にある時だけ採用（旧 `{{ if lookPath … }}`）。
    #[serde(default)]
    pub dep: Option<String>,
    /// OS 一致時だけ採用（旧 `{{ if eq .chezmoi.os … }}`）。chezmoi 互換表記。
    #[serde(default)]
    pub os: Option<String>,
}

impl Manifest {
    /// `manifest.toml` を読み込み、パース後にセマンティックバリデーション（§5.5）を行う。
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("{}: 読み込み失敗: {e}", path.display()))?;
        let manifest: Self =
            toml::from_str(&text).map_err(|e| format!("{}: パース失敗: {e}", path.display()))?;
        manifest
            .validate()
            .map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(manifest)
    }

    /// パース後のセマンティック検証（§5.5）。manifest の typo を配置前に弾く。
    ///
    /// - **overlay 明示時は `strategy` 必須**。合成戦略を一意に決めるため、暗黙の `concat` は
    ///   overlay 未記述の generate 既定挙動だけに限る（overlay を書いて戦略を省くと、意図しない
    ///   text concat になりうるのを防ぐ）。
    /// - **`preserve = true` は `strategy = "json-shallow"` 専用**。既存 dst を土台に重ねる
    ///   意味論は JSON shallow merge でしか定義しないため、他 strategy・省略との併記は typo
    ///   として弾く（静かに無視しない）。
    /// - **`preserve = true` は compose 経路（overlay 明示 か `kind = generate`）を要する**。
    ///   ファイル合成は [`crate::apply`] の `uses_compose`（overlay 非空 or generate）でだけ
    ///   起動するため、overlay 無しの copy 直行では preserve が黙って無視される。実行されない
    ///   構成を load 時に弾く（土台だけ再直列化する退化形に意味は無い）。
    /// - **各 overlay は `src` / `cmd` のうちちょうど 1 つ**（生成方式は択一）。2 つは一方が
    ///   黙って無視される typo、0 個は断片を実体化できない。
    /// - **`sensitive ⊆ locals`**（§9.1）。`sensitive` に `locals` 未宣言の名前があると、その名は
    ///   非エコー/ログ抑制の対象にならず（resolve は `locals` を走査するため）秘匿値が黙って漏れる。
    ///   typo を配置前に弾く。
    /// - **`hooks` の各コマンド（argv）は非空**（§13, S5）。空の argv は実体化できないコマンドで、
    ///   apply で黙って無視/panic されると事故になるため load 時に弾く（他の検証群と同じ
    ///   「静かに無視しない」方針）。フック名のレジストリ検証は持たない ― フックはツール名でなく
    ///   コマンドそのものをデータとして宣言する（binary は実行するだけ）。
    fn validate(&self) -> Result<(), String> {
        if !self.overlay.is_empty() && self.strategy.is_none() {
            return Err(
                "overlay を明示する場合は strategy（concat / json-shallow）が必要です".to_string(),
            );
        }
        if self.preserve && self.strategy != Some(Strategy::JsonShallow) {
            return Err("preserve = true は strategy = \"json-shallow\" 専用です".to_string());
        }
        if self.preserve && self.overlay.is_empty() && self.kind != Kind::Generate {
            return Err(
                "preserve = true は overlay か kind = generate を要します（overlay 無しの copy は \
                 compose 経路に入らず preserve が無視されます）"
                    .to_string(),
            );
        }
        for (i, ov) in self.overlay.iter().enumerate() {
            let kinds = [ov.src.is_some(), !ov.cmd.is_empty()]
                .into_iter()
                .filter(|&set| set)
                .count();
            if kinds != 1 {
                return Err(format!(
                    "overlay[{i}] は src / cmd のうちちょうど 1 つを持つ必要があります（現在 {kinds} 個）"
                ));
            }
        }
        if let Some(orphan) = self.sensitive.iter().find(|s| !self.locals.contains(s)) {
            return Err(format!(
                "sensitive `{orphan}` が locals に宣言されていません（sensitive ⊆ locals）"
            ));
        }
        if self.hooks.iter().any(|argv| argv.is_empty()) {
            return Err("hooks の各要素は非空のコマンド（argv）である必要があります".to_string());
        }
        Ok(())
    }

    /// この単位の配置ファイルへ与える Unix パーミッション（8 進）。
    ///
    /// base は `private` で決まる（0600 / 0644）。`executable` のとき、read ビットが
    /// 立っている桁へ execute ビットを足す（0644→0755 / 0600→0700）。chezmoi の
    /// `private_` / `executable_` 属性と同じ合成規則。
    #[cfg(unix)]
    pub fn mode(&self) -> u32 {
        let base: u32 = if self.private { 0o600 } else { 0o644 };
        if self.executable {
            base | ((base & 0o444) >> 2)
        } else {
            base
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// パース → validate を一括で通す（load のファイル I/O を介さずに検証）。
    fn parse(toml_src: &str) -> Result<Manifest, String> {
        let manifest: Manifest = toml::from_str(toml_src).map_err(|e| e.to_string())?;
        manifest.validate()?;
        Ok(manifest)
    }

    #[test]
    fn validate_accepts_generate_default_without_strategy() {
        // overlay 未記述の generate は strategy 省略可（暗黙 concat）。
        assert!(parse("dst = \"~/x\"\nkind = \"generate\"\ncmd = [\"foo\"]\n").is_ok());
    }

    #[test]
    fn validate_requires_strategy_when_overlay_present() {
        let err = parse("dst = \"~/x\"\n[[overlay]]\nsrc = \"a\"\n").unwrap_err();
        assert!(
            err.contains("strategy"),
            "strategy 必須のエラーが出ていない: {err}"
        );
    }

    #[test]
    fn validate_rejects_overlay_with_multiple_kinds() {
        // src と cmd を併記 → 一方が黙って無視される typo。
        let err = parse(
            "dst = \"~/x\"\nstrategy = \"concat\"\n[[overlay]]\nsrc = \"a\"\ncmd = [\"foo\"]\n",
        )
        .unwrap_err();
        assert!(
            err.contains("ちょうど 1 つ"),
            "排他違反が検知されていない: {err}"
        );
    }

    #[test]
    fn validate_rejects_preserve_without_json_shallow() {
        // preserve = true は json-shallow 専用。concat と併記 → load 時エラー（typo 黙殺しない）。
        let err = parse(
            "dst = \"~/x\"\nstrategy = \"concat\"\npreserve = true\n[[overlay]]\nsrc = \"a\"\n",
        )
        .unwrap_err();
        assert!(
            err.contains("preserve") && err.contains("json-shallow"),
            "preserve × 非 json-shallow が弾かれていない: {err}"
        );
        // strategy 省略との併記も同様にエラー。
        assert!(parse("dst = \"~/x\"\npreserve = true\n").is_err());
    }

    #[test]
    fn validate_accepts_preserve_with_json_shallow() {
        // preserve = true ＋ json-shallow ＋ overlay は正規形（claude/settings 相当）。
        assert!(
            parse(
                "dst = \"~/x\"\nstrategy = \"json-shallow\"\npreserve = true\n\
                 [[overlay]]\nsrc = \"settings.json\"\n",
            )
            .is_ok()
        );
        // generate（cmd 出力を土台へ重ねる）も compose 経路なので overlay 無しでも可。
        assert!(
            parse(
                "dst = \"~/x\"\nkind = \"generate\"\nstrategy = \"json-shallow\"\n\
                 preserve = true\ncmd = [\"foo\"]\n",
            )
            .is_ok()
        );
    }

    #[test]
    fn validate_rejects_preserve_without_compose_routing() {
        // preserve = true ＋ json-shallow だが overlay 無し ＋ kind=copy（既定）→ copy 直行で
        // preserve が黙って無視される構成。load 時に弾く（静かな no-op を許さない）。
        let err =
            parse("dst = \"~/x\"\nstrategy = \"json-shallow\"\npreserve = true\n").unwrap_err();
        assert!(
            err.contains("preserve") && err.contains("overlay"),
            "overlay 無しの copy 直行 preserve が弾かれていない: {err}"
        );
    }

    #[test]
    fn validate_rejects_overlay_with_no_kind() {
        // when だけで src/cmd/preserve が無い → 断片を実体化できない。
        let err =
            parse("dst = \"~/x\"\nstrategy = \"concat\"\n[[overlay]]\nwhen = { dep = \"rtk\" }\n")
                .unwrap_err();
        assert!(
            err.contains("ちょうど 1 つ"),
            "0 個が検知されていない: {err}"
        );
    }

    #[test]
    fn validate_accepts_sensitive_subset_of_locals() {
        // sensitive ⊆ locals は正規形（§9.1）。email/name は非 sensitive のまま宣言できる。
        assert!(
            parse(
                "dst = \"~/x\"\nlocals = [\"git.email\", \"git.name\", \"github.token\"]\n\
                 sensitive = [\"github.token\"]\n",
            )
            .is_ok()
        );
        // sensitive 省略（全て非秘匿）も可。
        assert!(parse("dst = \"~/x\"\nlocals = [\"git.email\", \"git.name\"]\n").is_ok());
    }

    #[test]
    fn validate_rejects_sensitive_not_in_locals() {
        // locals に無い名を sensitive に書く typo → 非エコー抑制が効かず漏れる footgun。
        let err = parse("dst = \"~/x\"\nlocals = [\"git.email\"]\nsensitive = [\"githb.token\"]\n")
            .unwrap_err();
        assert!(
            err.contains("sensitive") && err.contains("githb.token"),
            "sensitive ⊄ locals が弾かれていない: {err}"
        );
    }

    #[test]
    fn validate_accepts_command_hook() {
        // hooks はコマンド（argv）の配列。エンジンは中身を解釈しないので、検証は「非空」だけ。
        // 特定ツールに依存しない汎用テスト ― サンプルも実ツール名でなく中立な argv にする。
        assert!(parse("dst = \"~/x\"\nhooks = [[\"cmd\", \"sub\", \"--flag\"]]\n").is_ok());
    }

    #[test]
    fn validate_rejects_empty_hook() {
        // 空の argv は実体化できないコマンド。load 時に弾く（黙って無視/panic しない）。
        let err = parse("dst = \"~/x\"\nhooks = [[]]\n").unwrap_err();
        assert!(
            err.contains("hooks") && err.contains("非空"),
            "空コマンドが弾かれていない: {err}"
        );
    }

    #[test]
    fn validate_accepts_well_formed_overlays() {
        // preserve = true（ユニット属性）＋ base(src) ＋ rtk(src+when) の正規形。
        assert!(
            parse(
                "dst = \"~/x\"\nstrategy = \"json-shallow\"\npreserve = true\n\
                 [[overlay]]\nsrc = \"base.json\"\n\
                 [[overlay]]\nsrc = \"rtk.json\"\nwhen = { dep = \"rtk\" }\n",
            )
            .is_ok()
        );
    }
}
