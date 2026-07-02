//! `manifest.toml` のスキーマと読み込み。
//!
//! 設計書（docs/dotfiles-native-design.md §5 / §5.5 / §6.2 / §7）の **2軸モデル**を解釈する:
//! - **生成方式 `kind`**（断片をどう実体化するか）= `copy` / `generate`（省略時 copy）。
//! - **合成 `strategy`**（複数の条件付き断片を1 dst=ファイルへどう重ねるか）= `concat` /
//!   `json-shallow`。`merge` は独立 kind ではなく合成軸の JSON 戦略（§5.5）。
//! - **条件付き overlay**（`[[overlay]]` ＋ `when`）= dst を「base ＋ gate された断片」の合成
//!   として組む。各 overlay は `src`（copy 断片）/ `cmd`（generate 断片）のどちらか ＋
//!   `when`（`deps` / `os`）。既存 dst の温存はユニット属性 `preserve = true`（§5.5）。
//!
//! gate 語彙は `when`（`deps` 配列 ＝ AND / `os` スカラ / `profile` スカラ）に一本化する。**書く位置で
//! スコープが決まる**: トップレベルの `when` はユニット全体 gate（false なら dst も `hooks` も触らず
//! skip ＝ all-or-nothing）、`[[overlay]]` 内の `when` はその断片だけの採否（§5.5）。両者は同じ評価規則を
//! [`crate::core::apply::gate`] で共有する。`profile` は環境検出（`deps`/`os`）と違い user が選んでおく
//! 状態（[`crate::core::state`]）を読む状態 gate で、`theme`（color スライスまで未配線）と同族（§10）。
//! `locals` / `sensitive` はマシンローカル値（named value）の宣言（§9, S4）。`hooks` は配置後フック
//! （§13, S5）の宣言で、各エントリが実行する `cmd`（argv）と実行頻度 `frequency`（§13.0・省略時
//! `onchange`, #546）を持ち、各 `cmd` が非空であることを load 時に検証する（実行は
//! [`crate::core::hooks`] の汎用エンジン。ツール固有ロジックは binary でなく manifest が持つ）。

use serde::Deserialize;
use std::path::Path;
use strum::Display;

/// 1 つの設定単位（`manifest.toml` を持つディレクトリ）の配置仕様。
///
/// `deny_unknown_fields` で未知キーを load 時に弾く。旧 gate 語彙（unit 属性 `deps` / `os`）が
/// 残った manifest はここでエラーになる（§5.5: 後方互換は持たせず、誤連想の再発を防ぐ）。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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
    /// ユニット全体 gate（§5.5・§7）。トップレベルに書いた `when` はユニットスコープで、
    /// 満たさなければユニット全体を skip する（dst も `hooks` も触らない ＝ all-or-nothing）。
    /// `when.deps`（配列・AND）が PATH に揃い、`when.os`（スカラ）が現在 OS と一致し、
    /// `when.profile`（スカラ・状態）が現在の profile 状態と一致した時だけ真。省略時は常時採用。
    /// 同じ語彙を `[[overlay]]` の `when` がその断片スコープで再利用する。
    #[serde(default)]
    pub when: Option<When>,
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
    /// 配置後フック（§13, S5 / #546）。このユニットの配置後（after フェーズ）に**宣言順**で実行する
    /// エントリの配列（例 `[[hooks]]` ＋ `cmd = ["bat", "cache", "--build"]`）。各エントリは実行する
    /// [`Hook::cmd`]（argv）と実行頻度 [`Hook::frequency`]（onchange / always・省略時 onchange）を持つ。
    /// ツール固有ロジックは binary に持たず、実行するコマンドをデータとして宣言する
    /// （[`crate::core::apply::generate`] の `cmd` と同思想）→ 新ツールのフック追加に binary 変更は不要・
    /// configs と疎結合。各 `cmd` が非空であることを load 時に検証する。頻度による実行モデルの分岐
    /// （onchange gate / 無条件実行）は [`crate::core::hooks`] が担う。トップレベル `when`（ユニット
    /// gate）が false のユニットは配置ごと skip されるため hooks も走らない（＝ `when.os` でフックを
    /// 分岐できる, §5.5 不変条件①）。
    #[serde(default)]
    pub hooks: Vec<Hook>,
}

/// 生成方式（断片の実体化方法）。copy / generate。`merge` は kind ではなく `strategy`（§5.5）。
/// 表示名（`Display`）と受理する TOML トークン（serde）を一致させるため `serialize_all` と
/// `rename_all` は同じ規則で揃える（ズレは tests の round-trip が検出する）。
#[derive(Debug, Deserialize, Default, PartialEq, Eq, Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Kind {
    #[default]
    Copy,
    Generate,
}

/// 合成戦略（複数断片を1 dst=ファイルへ重ねる方法, §5.5）。
/// 表示名と TOML トークンを揃える規則は [`Kind`] と同じ（`serialize_all` / `rename_all` を `kebab-case` に）。
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Strategy {
    /// テキスト連結（後ろへ連結）。境目に改行を 1 つ補う。
    Concat,
    /// JSON のトップレベル shallow merge（後勝ち）。deep merge はしない。
    JsonShallow,
}

/// フックの実行頻度（§13.0 / #546）。頻度は per-hook の実行モードで、別リスト（`always_hooks` 等）に
/// 分けず既存 `hooks` エントリの属性として持つ（分けると validation・表示が二重化する）。
/// 表示名（`Display`）と受理する TOML トークン（serde）を揃える規則は [`Kind`] / [`Strategy`] と同じ。
#[derive(Debug, Deserialize, Default, PartialEq, Eq, Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Frequency {
    /// ユニットのソースが前回適用時から変わった時だけ実行する（onchange gate, §13.1）。省略時の既定。
    #[default]
    Onchange,
    /// 毎 apply 無条件に実行する（gate を通さず状態も読み書きしない, §13.0）。反映対象が dotfiles
    /// 管理外で随時変わる用途向け。コマンドが冪等であること（何度走らせても同じ結果）を前提とする。
    Always,
}

/// 1 つの配置後フック（§13 / #546）。ユニット配置後に実行する `cmd`（argv）と実行頻度 `frequency`。
/// `deny_unknown_fields` でキーの typo（`frequancy` 等）を load 時に弾く（黙って無視しない）。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hook {
    /// 実行するコマンド（argv）。先頭が実行ファイル名、以降が引数。非空を load 時に検証する。
    pub cmd: Vec<String>,
    /// 実行頻度（省略時 onchange）。onchange = ソース変化時のみ / always = 毎 apply 無条件（§13.0）。
    #[serde(default)]
    pub frequency: Frequency,
}

/// 1 つの overlay（条件付き断片, §5.5）。`when` を満たす時だけ合成に参加する。
/// 断片の実体化方法は `src`（copy）/ `cmd`（generate）の択一。既存 dst の温存は overlay では
/// なくユニット属性 [`Manifest::preserve`]。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Overlay {
    /// copy 断片: 単位ディレクトリからの相対ファイル。内容をそのまま断片にする。
    #[serde(default)]
    pub src: Option<String>,
    /// generate 断片: 実行する argv。標準出力を断片にする。
    #[serde(default)]
    pub cmd: Vec<String>,
    /// 採用条件（省略 = 常時採用）。この断片スコープで `when`（`deps` / `os` / `profile`）を AND 評価する。
    #[serde(default)]
    pub when: Option<When>,
}

/// gate の採用条件（§5.5）。トップレベル（ユニットスコープ）と overlay（断片スコープ）で共有する
/// 1 つの語彙。複数キーは AND（全て満たす時だけ採用）。
#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct When {
    /// 依存バイナリ（配列・AND）。全て PATH にある時だけ採用（旧 `{{ if lookPath … }}`）。
    /// 単数 `dep` は廃止し、複数形＝配列で統一する（語感の破綻を避ける, §5.5）。
    #[serde(default)]
    pub deps: Vec<String>,
    /// OS（スカラ）。現在 OS と一致時だけ採用（旧 `{{ if eq .chezmoi.os … }}`）。chezmoi 互換表記。
    #[serde(default)]
    pub os: Option<String>,
    /// マシンクラス（スカラ・状態 gate）。`dotfiles profile <name>` が書いた現在の profile 状態
    /// （[`crate::core::state`]）と一致するときだけ採用する。`deps`（環境検出）・`os`（環境検出）と違い
    /// **user が選んでおく状態**を読む点が族として `theme`（color スライス）と同じ。未設定の既定は
    /// not-private（新規・仕事マシンへ private 設定が漏れないよう明示 opt-in）。
    #[serde(default)]
    pub profile: Option<String>,
}

impl When {
    /// 実効キー（`deps` / `os`）を 1 つも持たないか。
    ///
    /// 空テーブル `when = {}` や `when = { deps = [] }` は常時採用の silent no-op になり、
    /// 「gate を書いたのに効かない」typo（編集で内部キーだけ消えた等）を黙って通す。これを
    /// load 時に弾くため [`Manifest::validate`] が使う。`theme` 等のキー追加時はここに足す。
    fn has_no_effective_key(&self) -> bool {
        self.deps.is_empty() && self.os.is_none() && self.profile.is_none()
    }
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
    ///   ファイル合成は [`crate::core::apply`] の `uses_compose`（overlay 非空 or generate）でだけ
    ///   起動するため、overlay 無しの copy 直行では preserve が黙って無視される。実行されない
    ///   構成を load 時に弾く（土台だけ再直列化する退化形に意味は無い）。
    /// - **各 overlay は `src` / `cmd` のうちちょうど 1 つ**（生成方式は択一）。2 つは一方が
    ///   黙って無視される typo、0 個は断片を実体化できない。
    /// - **`sensitive ⊆ locals`**（§9.1）。`sensitive` に `locals` 未宣言の名前があると、その名は
    ///   非エコー/ログ抑制の対象にならず（resolve は `locals` を走査するため）秘匿値が黙って漏れる。
    ///   typo を配置前に弾く。
    /// - **`hooks` の各コマンド（`cmd`）は非空**（§13, S5）。空の `cmd` は実体化できないコマンドで、
    ///   apply で黙って無視/panic されると事故になるため load 時に弾く（他の検証群と同じ
    ///   「静かに無視しない」方針）。フック名のレジストリ検証は持たない ― フックはツール名でなく
    ///   コマンドそのものをデータとして宣言する（binary は実行するだけ）。
    fn validate(&self) -> Result<(), String> {
        if !self.overlay.is_empty() && self.strategy.is_none() {
            return Err(format!(
                "overlay を明示する場合は strategy（{} / {}）が必要です",
                Strategy::Concat,
                Strategy::JsonShallow,
            ));
        }
        if self.preserve && self.strategy != Some(Strategy::JsonShallow) {
            return Err(format!(
                "preserve = true は strategy = \"{}\" 専用です",
                Strategy::JsonShallow
            ));
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
        if self.hooks.iter().any(|h| h.cmd.is_empty()) {
            return Err("hooks の各要素は非空のコマンド（cmd）である必要があります".to_string());
        }
        if let Some(when) = &self.when
            && when.has_no_effective_key()
        {
            return Err(
                "when は実効キー（deps / os / profile）を 1 つ以上持つ必要があります（空の when は silent no-op）"
                    .to_string(),
            );
        }
        if let Some(i) = self
            .overlay
            .iter()
            .position(|ov| ov.when.as_ref().is_some_and(When::has_no_effective_key))
        {
            return Err(format!(
                "overlay[{i}] の when は実効キー（deps / os / profile）を 1 つ以上持つ必要があります（空の when は silent no-op）"
            ));
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
    fn kind_display_round_trips_through_serde() {
        // Display（表示名の出所）が serde の受理トークンと一致することを固定する。ズレると
        // apply / list の表示が manifest に書ける値からズレる。
        for kind in [Kind::Copy, Kind::Generate] {
            let parsed = parse(&format!("dst = \"~/x\"\nkind = \"{kind}\"\n")).unwrap();
            assert_eq!(
                parsed.kind, kind,
                "Display と serde 表現がズレている: {kind}"
            );
        }
    }

    #[test]
    fn strategy_display_round_trips_through_serde() {
        for strategy in [Strategy::Concat, Strategy::JsonShallow] {
            let parsed = parse(&format!("dst = \"~/x\"\nstrategy = \"{strategy}\"\n")).unwrap();
            assert_eq!(
                parsed.strategy,
                Some(strategy),
                "Display と serde 表現がズレている: {strategy}"
            );
        }
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
        // preserve = true ＋ json-shallow ＋ overlay は正規形（既存 dst を土台に断片を重ねる合成）。
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
        let err = parse(
            "dst = \"~/x\"\nstrategy = \"concat\"\n[[overlay]]\nwhen = { deps = [\"faketool\"] }\n",
        )
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
    fn frequency_display_round_trips_through_serde() {
        // Display（表示名の出所）が serde の受理トークンと一致することを固定する（Kind/Strategy と同じ）。
        // ズレると list の hooks 内訳表示が manifest に書ける値からズレる。
        for frequency in [Frequency::Onchange, Frequency::Always] {
            let parsed = parse(&format!(
                "dst = \"~/x\"\n[[hooks]]\ncmd = [\"faketool\"]\nfrequency = \"{frequency}\"\n"
            ))
            .unwrap();
            assert_eq!(
                parsed.hooks[0].frequency, frequency,
                "Display と serde 表現がズレている: {frequency}"
            );
        }
    }

    #[test]
    fn validate_accepts_command_hook() {
        // 非空の cmd を持つ structured hook は受理（エンジンは中身を解釈しない, §13）。frequency 省略時 onchange。
        let parsed =
            parse("dst = \"~/x\"\nhooks = [{ cmd = [\"cmd\", \"sub\", \"--flag\"] }]\n").unwrap();
        assert_eq!(
            parsed.hooks[0].frequency,
            Frequency::Onchange,
            "frequency 省略時は onchange であるべき"
        );
    }

    #[test]
    fn validate_rejects_empty_hook() {
        // 空の cmd は実体化できないコマンド。load 時に弾く（黙って無視/panic しない）。
        let err = parse("dst = \"~/x\"\nhooks = [{ cmd = [] }]\n").unwrap_err();
        assert!(
            err.contains("hooks") && err.contains("非空"),
            "空コマンドが弾かれていない: {err}"
        );
    }

    #[test]
    fn parse_rejects_legacy_bare_array_hook() {
        // 旧 bare-array 構文 `hooks = [["cmd"]]` は廃止（#546）。frequency を持たせるため構造体化し、
        // 後方互換は取らない。Vec<Hook>（cmd を持つ table を期待）と配列-of-配列の型不一致で parse
        // 時に弾く＝黙って誤解釈しない。
        assert!(toml::from_str::<Manifest>("dst = \"~/x\"\nhooks = [[\"faketool\"]]\n").is_err());
    }

    #[test]
    fn parse_rejects_hook_unknown_field() {
        // Hook の deny_unknown_fields: frequency の typo（frequancy）は load 時エラー（黙って無視しない）。
        assert!(
            toml::from_str::<Manifest>(
                "dst = \"~/x\"\n[[hooks]]\ncmd = [\"faketool\"]\nfrequancy = \"always\"\n"
            )
            .is_err()
        );
    }

    #[test]
    fn validate_accepts_well_formed_overlays() {
        // preserve = true（ユニット属性）＋ base(src) ＋ 条件付き断片(src+when) の正規形。
        assert!(
            parse(
                "dst = \"~/x\"\nstrategy = \"json-shallow\"\npreserve = true\n\
                 [[overlay]]\nsrc = \"base.json\"\n\
                 [[overlay]]\nsrc = \"faketool.json\"\nwhen = { deps = [\"faketool\"] }\n",
            )
            .is_ok()
        );
    }

    #[test]
    fn parse_accepts_top_level_when() {
        // トップレベル when（ユニットスコープ gate）= deps 配列 ＋ os スカラ。
        assert!(parse("dst = \"~/x\"\nwhen = { deps = [\"ghostty\"], os = \"darwin\" }\n").is_ok());
    }

    #[test]
    fn parse_accepts_profile_gate() {
        // profile（状態 gate）は実効キーとして受理する（単独・他キーとの併記とも）。
        assert!(parse("dst = \"~/x\"\nwhen = { profile = \"private\" }\n").is_ok());
        assert!(
            parse("dst = \"~/x\"\nwhen = { profile = \"private\", os = \"darwin\" }\n").is_ok()
        );
    }

    #[test]
    fn validate_rejects_top_level_empty_when() {
        // when = {} / when = { deps = [] } は常時採用の silent no-op。load 時に弾く（fail loud）。
        assert!(parse("dst = \"~/x\"\nwhen = {}\n").is_err());
        let err = parse("dst = \"~/x\"\nwhen = { deps = [] }\n").unwrap_err();
        assert!(
            err.contains("when") && err.contains("実効キー"),
            "実効キーの無いトップレベル when が弾かれていない: {err}"
        );
    }

    #[test]
    fn validate_rejects_overlay_empty_when() {
        // overlay の when も同様に実効キー必須（断片が常時採用される silent no-op を弾く）。
        let err =
            parse("dst = \"~/x\"\nstrategy = \"concat\"\n[[overlay]]\nsrc = \"a\"\nwhen = {}\n")
                .unwrap_err();
        assert!(
            err.contains("overlay[0]") && err.contains("実効キー"),
            "実効キーの無い overlay の when が弾かれていない: {err}"
        );
    }

    #[test]
    fn parse_rejects_legacy_unit_deps() {
        // 旧 unit 属性 deps は廃止。deny_unknown_fields が load 時に弾く（後方互換なし, §5.5）。
        assert!(toml::from_str::<Manifest>("dst = \"~/x\"\ndeps = [\"gh\"]\n").is_err());
    }

    #[test]
    fn parse_rejects_legacy_unit_os() {
        // 旧 unit 属性 os も同様に廃止。
        assert!(toml::from_str::<Manifest>("dst = \"~/x\"\nos = \"darwin\"\n").is_err());
    }

    #[test]
    fn parse_rejects_legacy_singular_dep_in_when() {
        // overlay の単数 dep は廃止し複数形 deps へ。旧キーは load 時エラー。
        assert!(
            toml::from_str::<Manifest>(
                "dst = \"~/x\"\nstrategy = \"concat\"\n[[overlay]]\nsrc = \"a\"\nwhen = { dep = \"faketool\" }\n",
            )
            .is_err()
        );
    }
}
