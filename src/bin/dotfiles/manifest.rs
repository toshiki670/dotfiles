//! `manifest.toml` のスキーマと読み込み。
//!
//! ユニットを `[[steps]]` の列として解釈する。内容を空から始め、宣言順に各 step を畳む:
//! - **input** step: 内容へ中身を畳む（最初の input は内容＝中身、2 つ目以降は `merge` で重ねる）。
//! - **output** step: 内容を宛先へ書く。
//!
//! 各 step の `input` / `output` は択一で、どちらも「パス文字列」か「`cmd`（argv・標準入出力）」の
//! 択一（[`StepSource`]）。重ね方の内容型は unit レベルの `format`（json / plist / text）、per-step の
//! `merge`（shallow / append）が「どう重ねるか」を宣言する。`merge` は load 時の整合検証のための注釈で、
//! 実行時の畳み込みの仕組みは [`crate::apply::pipeline`]。
//!
//! gate 語彙は `when`（`deps` 配列 ＝ AND / `os` スカラ / `profile` スカラ）に一本化する。**書く位置で
//! スコープが決まる**: トップレベルの `when` はユニット全体 gate（false ならユニットごと skip ＝
//! all-or-nothing）、step 内の `when` はその step だけの採否。両者は同じ評価規則を
//! [`crate::apply::gate`] で共有する。`profile` は環境からその場で判る条件（`deps`/`os`）と違い
//! user が選んでおく状態（[`crate::state`]）を読む状態 gate で、`theme`（color スライスまで
//! 未配線）と同族。
//!
//! `locals` はマシンローカル値（named value）の宣言。`hooks` は配置後フック（onchange 固定）の宣言で、
//! 各エントリが実行する `cmd`（argv）を持ち、非空であることを load 時に検証する（実行は
//! [`crate::hooks`] の汎用エンジン。ツール固有ロジックは binary でなく manifest が持つ）。

use serde::Deserialize;
use std::path::{Component, Path};
use strum::{Display, EnumIter};

/// 1 つの設定単位（`manifest.toml` を持つディレクトリ）の配置仕様。
///
/// `deny_unknown_fields` で未知キーを load 時に弾く。旧スキーマの語彙（`dst` / `kind` / `strategy` /
/// `preserve` / ユニット直下 `cmd` / `[[overlay]]` / `sensitive` 等）が残った manifest はここで
/// エラーになる（後方互換は持たせず、誤連想の再発を防ぐ）。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// ユニット全体 gate。トップレベルに書いた `when` はユニットスコープで、
    /// 満たさなければユニット全体を skip する（配置も `hooks` も触らない ＝ all-or-nothing）。
    /// `when.deps`（配列・AND）が PATH に揃い、`when.os`（スカラ）が現在 OS と一致し、
    /// `when.profile`（スカラ・状態）が現在の profile 状態と一致した時だけ真。省略時は常時採用。
    /// 同じ語彙を各 step の `when` がその step スコープで再利用する。
    #[serde(default)]
    pub when: Option<When>,
    /// マシンローカル値（named value）の宣言。ここで宣言した名前 `n` に対し、この単位の
    /// 配置ファイル中の `@@n@@` をストア（`~/.config/dotfiles/local.toml`）の値で置換する。
    /// 置換は `locals` を宣言した単位のファイルにだけ走る（無関係ファイルの `@@…@@` を巻き込まない）。
    /// 例: `locals = ["git.email", "git.name"]`。
    #[serde(default)]
    pub locals: Vec<String>,
    /// 合成の内容型（`merge` を使うユニットに必須・使わないユニットには書けない）。
    /// `json` / `plist` / `text`。実行時の畳み込みはこの単一値だけで駆動する。
    #[serde(default)]
    pub format: Option<Format>,
    /// 配置パイプライン。input（読む）→ output（書く）の step 列。空は load 時エラー
    /// （input を 1 つ以上・output を 1 つ以上必須）。
    #[serde(default)]
    pub steps: Vec<Step>,
    /// 所有者のみアクセス可（0600 相当）。省略時 false。パス output を持つユニットのみ。
    #[serde(default)]
    pub private: bool,
    /// 実行ビットを付与（0644→0755 / 0600→0700）。省略時 false。パス output を持つユニットのみ。
    #[serde(default)]
    pub executable: bool,
    /// 配置後フック（onchange 固定）。このユニットの配置後に**宣言順**で実行するエントリの配列
    /// （例 `[[hooks]]` ＋ `cmd = ["bat", "cache", "--build"]`）。各エントリは実行する [`Hook::cmd`]
    /// （argv）を持つ。ツール固有ロジックは binary に持たず、実行するコマンドをデータとして宣言する
    /// → 新ツールのフック追加に binary 変更は不要・configs と疎結合。各 `cmd` が非空であることを
    /// load 時に検証する。実行は [`crate::hooks`]。トップレベル `when`（ユニット gate）が false の
    /// ユニットは配置ごと skip されるため hooks も走らない（＝ `when.os` でフックを分岐できる）。
    #[serde(default)]
    pub hooks: Vec<Hook>,
}

/// 配置パイプラインの 1 step。`input` / `output` の択一を持つ。
/// `deny_unknown_fields` でキーの typo を load 時に弾く。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    /// 読む内容源（パス or cmd 標準出力）。`output` との併記・双方省略は load 時エラー。
    #[serde(default)]
    pub input: Option<StepSource>,
    /// 書く宛先（パス or cmd 標準入力）。`input` との併記・双方省略は load 時エラー。
    #[serde(default)]
    pub output: Option<StepSource>,
    /// 重ね方（2 つ目以降の input に必須・最初の input と output には禁止）。値は `format` に従う:
    /// json / plist → `shallow`、text → `append`。load 時の整合検証のための注釈で、実行時の
    /// 畳み込みの駆動については [`crate::apply::pipeline`]。
    #[serde(default)]
    pub merge: Option<Merge>,
    /// この step の採否（省略 = 常時採用）。unit gate と共通の語彙（`deps` / `os` / `profile`）。
    #[serde(default)]
    pub when: Option<When>,
    /// `~` 起点のパス input が存在しなければこの step を飛ばす（次の input が土台になる）。既定は
    /// 「無ければエラー」。`~` 起点のパス input のみ有効 ― 単位相対ファイルは repo 追跡の静的ファイルで
    /// 不在は typo の可能性が高く、恒久的に step を握り潰すため、cmd input・output への指定と併せて
    /// load 時エラー。唯一の正当な用途は宛先の現在内容（初回 apply 前は未在り得る `~` 起点ファイル）を
    /// 土台に読むこと。
    #[serde(default)]
    pub optional: bool,
}

/// step の内容源。パス文字列（`"settings.json"` / `"~/.claude/settings.json"`）か
/// `cmd`（`{ cmd = ["gh", "completion", "fish"] }`）の択一。TOML の型（bare string / inline table）で
/// 判別するため `untagged` で受ける（src/cmd の排他は型システムが保証し、実行時検証は要らない）。
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum StepSource {
    /// パス: input は単位相対 or `~` 起点、output は `~` 起点のみ（[`Manifest::validate`] が検証）。
    /// 特例として input `"."` は「単位ディレクトリツリー」を表す。
    Path(String),
    /// コマンド: input は標準出力を読み、output は内容を標準入力へ渡す（argv）。
    Cmd(CmdSource),
}

/// cmd 内容源（`{ cmd = [...] }`）。`deny_unknown_fields` で `cmd` 以外のキーを弾く。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CmdSource {
    /// 実行する argv。先頭が実行ファイル名、以降が引数。
    pub cmd: Vec<String>,
}

/// 合成の内容型。表示名（`Display`）と受理する TOML トークン（serde）を一致させるため
/// `serialize_all` と `rename_all` を同じ規則で揃える（ズレは tests の round-trip が検出する）。
/// `EnumIter` は tests の round-trip が全 variant を列挙するのに使う（手書き列挙の更新漏れを防ぐ）。
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Display, EnumIter)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Format {
    /// JSON。`merge = "shallow"` と両立。
    Json,
    /// plist（Apple の property list）。`merge = "shallow"` と両立。
    Plist,
    /// プレーンテキスト。`merge = "append"` と両立。
    Text,
}

/// 重ね方。表示名と TOML トークンを揃える規則は [`Format`] と同じ。
/// deep（object 再帰マージ）は本スライスでは未実装 ― variant を足さないことで「shallow のつもりが
/// deep」等の取り違えを型で防ぐ（deep はスライス2で追加予定）。
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Display, EnumIter)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Merge {
    /// トップレベルキー単位で後勝ち置換（json / plist）。
    Shallow,
    /// テキスト連結（境目に改行 1 つ）。
    Append,
}

/// 1 つの配置後フック（onchange 固定）。ユニット配置後に実行する `cmd`（argv）。
/// `deny_unknown_fields` でキーの typo を load 時に弾く。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hook {
    /// 実行するコマンド（argv）。先頭が実行ファイル名、以降が引数。非空を load 時に検証する。
    pub cmd: Vec<String>,
}

/// gate の採用条件。トップレベル（ユニットスコープ）と step（step スコープ）で共有する
/// 1 つの語彙。複数キーは AND（全て満たす時だけ採用）。
#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct When {
    /// 依存バイナリ（配列・AND）。全て PATH にある時だけ採用。
    /// 単数 `dep` は廃止し、複数形＝配列で統一する（語感の破綻を避ける）。
    #[serde(default)]
    pub deps: Vec<String>,
    /// OS（スカラ）。現在 OS と一致時だけ採用（`darwin` / `linux` 表記）。
    #[serde(default)]
    pub os: Option<String>,
    /// マシンクラス（スカラ・状態 gate）。`dotfiles profile <name>` が書いた現在の profile 状態
    /// （[`crate::state`]）と一致するときだけ採用する。`deps`（環境検出）・`os`（環境検出）と違い
    /// **user が選んでおく状態**を読む点が族として `theme`（color スライス）と同じ。未設定の既定は
    /// not-private（新規・仕事マシンへ private 設定が漏れないよう明示 opt-in）。
    #[serde(default)]
    pub profile: Option<String>,
}

impl When {
    /// 実効キー（`deps` / `os` / `profile`）を 1 つも持たないか。
    ///
    /// 空テーブル `when = {}` や `when = { deps = [] }` は常時採用の silent no-op になり、
    /// 「gate を書いたのに効かない」typo（編集で内部キーだけ消えた等）を黙って通す。これを
    /// load 時に弾くため [`Manifest::validate`] が使う。`theme` 等のキー追加時はここに足す。
    fn has_no_effective_key(&self) -> bool {
        self.deps.is_empty() && self.os.is_none() && self.profile.is_none()
    }
}

/// step が「ツリー input」（`input = "."` ＝ 単位ディレクトリをそのまま配置）か。
fn is_tree_input(step: &Step) -> bool {
    matches!(&step.input, Some(StepSource::Path(p)) if p == ".")
}

impl Manifest {
    /// `manifest.toml` を読み込み、パース後にセマンティックバリデーションを行う。
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

    /// このユニットがツリー配置（単位ディレクトリを丸ごと配置）か。`validate` 後に呼ぶ前提
    /// （検証済みなら `steps[0]` が唯一のツリー input）。list / apply の表示に使う。
    pub fn is_tree(&self) -> bool {
        self.steps.iter().any(is_tree_input)
    }

    /// apply の 1 行出力と `list` が共有する宛先表記: 最初のパス output の生表記（`~/...`）。
    /// パス output を持たない（cmd output だけの）ユニットは `(cmd)` を返す。
    pub fn display_dst(&self) -> String {
        self.steps
            .iter()
            .find_map(|s| match &s.output {
                Some(StepSource::Path(p)) => Some(p.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "(cmd)".to_string())
    }

    /// apply のラベルと `list` の属性が共有する steps サマリ。ツリーは `tree`、それ以外は
    /// `steps=Nin/Mout`（＋ `format` ＋ cmd output があれば `output=cmd`）。
    pub fn summary(&self) -> String {
        if self.is_tree() {
            return "tree".to_string();
        }
        let n_in = self.steps.iter().filter(|s| s.input.is_some()).count();
        let n_out = self.steps.iter().filter(|s| s.output.is_some()).count();
        let mut parts = vec![format!("steps={n_in}in/{n_out}out")];
        if let Some(format) = self.format {
            parts.push(format.to_string());
        }
        if self
            .steps
            .iter()
            .any(|s| matches!(&s.output, Some(StepSource::Cmd(_))))
        {
            parts.push("output=cmd".to_string());
        }
        parts.join(", ")
    }

    /// パース後のセマンティック検証。manifest の typo を配置前に弾く（fail-loud）。
    ///
    /// - **各 step は input / output のちょうど 1 つ**。両方・どちらも無しは typo。
    /// - **steps は input を 1 つ以上・output を 1 つ以上**持つ（空パイプラインは無意味）。
    /// - **パス表記**（#579）: output は `~` 起点のみ。input は単位相対 or `~` 起点。`$`・絶対は不可。
    /// - **`hooks` の各 `cmd` は非空**。
    /// - **`when` は実効キーを 1 つ以上**（unit・各 step とも。空 when の silent no-op を弾く）。
    /// - **`private` / `executable` はパス output を持つユニットのみ**（cmd output だけのユニットには
    ///   書き込み先ファイルが無い）。
    /// - **ツリー**（`input = "."`）: steps はちょうど 2 つ（`input = "."` ＋ output パス）、step に
    ///   `when` / `merge` / `optional` は付けない、`format` は書けない（merge / format はバイト内容の
    ///   step のみ意味を持ち、step の `when` は unit gate と冗長になる）。
    /// - **`merge`**: 2 つ目以降の input に必須・最初の input / output には禁止（暗黙の合成規則を持た
    ///   ない・#580）。
    /// - **`format`**: `merge` を使うユニットに必須・使わないユニットには禁止。`shallow` は json / plist・
    ///   `append` は text とだけ両立。
    /// - **`optional`**: `~` 起点のパス input のみ（単位相対ファイルは静的なので不在は typo・cmd input・
    ///   output への指定も typo）。
    fn validate(&self) -> Result<(), String> {
        // 各 step は input / output のちょうど 1 つ。
        for (i, step) in self.steps.iter().enumerate() {
            let n = [step.input.is_some(), step.output.is_some()]
                .into_iter()
                .filter(|&set| set)
                .count();
            if n != 1 {
                return Err(format!(
                    "steps[{i}] は input / output のうちちょうど 1 つを持つ必要があります（現在 {n} 個）"
                ));
            }
        }
        let n_input = self.steps.iter().filter(|s| s.input.is_some()).count();
        let n_output = self.steps.iter().filter(|s| s.output.is_some()).count();
        if n_input == 0 || n_output == 0 {
            return Err(format!(
                "steps は input を 1 つ以上・output を 1 つ以上持つ必要があります（現在 input {n_input}・output {n_output}）"
            ));
        }

        // パス表記（#579）: 全 step の path 内容源を検証する。
        for step in &self.steps {
            if let Some(StepSource::Path(p)) = &step.input {
                validate_input_path(p)?;
            }
            if let Some(StepSource::Path(p)) = &step.output {
                validate_output_path(p)?;
            }
        }

        // 各 step の cmd（input.cmd / output.cmd）は非空。空 argv は cmd::run/run_piped の cmd[0]
        // インデックスで panic するため、load 時に弾く（hooks の非空検証と同じ「静かに無視しない」方針）。
        for (i, step) in self.steps.iter().enumerate() {
            let empty_side = if matches!(&step.input, Some(StepSource::Cmd(c)) if c.cmd.is_empty())
            {
                Some("input")
            } else if matches!(&step.output, Some(StepSource::Cmd(c)) if c.cmd.is_empty()) {
                Some("output")
            } else {
                None
            };
            if let Some(side) = empty_side {
                return Err(format!(
                    "steps[{i}].{side}.cmd は非空のコマンド（argv）である必要があります"
                ));
            }
        }

        // hooks の cmd は非空。
        if self.hooks.iter().any(|h| h.cmd.is_empty()) {
            return Err("hooks の各要素は非空のコマンド（cmd）である必要があります".to_string());
        }

        // when は実効キー必須（unit・各 step）。
        if let Some(when) = &self.when
            && when.has_no_effective_key()
        {
            return Err(
                "when は実効キー（deps / os / profile）を 1 つ以上持つ必要があります（空の when は silent no-op）"
                    .to_string(),
            );
        }
        if let Some(i) = self
            .steps
            .iter()
            .position(|s| s.when.as_ref().is_some_and(When::has_no_effective_key))
        {
            return Err(format!(
                "steps[{i}] の when は実効キー（deps / os / profile）を 1 つ以上持つ必要があります（空の when は silent no-op）"
            ));
        }

        // private / executable はパス output を持つユニットのみ。
        if (self.private || self.executable)
            && !self
                .steps
                .iter()
                .any(|s| matches!(&s.output, Some(StepSource::Path(_))))
        {
            return Err(
                "private / executable はパス output を持つユニットのみに指定できます（cmd output だけの \
                 ユニットには書き込み先ファイルが無い）"
                    .to_string(),
            );
        }

        // ツリー（input = "."）は専用の形に固定する。
        if self.steps.iter().any(is_tree_input) {
            return self.validate_tree();
        }

        self.validate_pipeline()
    }

    /// ツリー（`input = "."`）ユニットの形を検証する。バイト内容の合成語彙（merge / format）は
    /// ツリーには意味を持たず、step の `when` は unit gate と冗長になるため、いずれも禁止する。
    fn validate_tree(&self) -> Result<(), String> {
        if self.steps.len() != 2 {
            return Err(format!(
                "ツリー input（input = \".\"）を持つユニットは steps がちょうど 2 つ（input = \".\" と \
                 output）である必要があります（現在 {} 個）",
                self.steps.len()
            ));
        }
        if !is_tree_input(&self.steps[0]) {
            return Err(
                "ツリー input（input = \".\"）は最初の step である必要があります".to_string(),
            );
        }
        if self.steps[0].when.is_some() || self.steps[0].merge.is_some() || self.steps[0].optional {
            return Err(
                "ツリー input（input = \".\"）には when / merge / optional を指定できません（ユニット \
                 全体の when gate を使ってください）"
                    .to_string(),
            );
        }
        let out = &self.steps[1];
        if out.output.is_none() {
            return Err("ツリー input の次の step は output である必要があります".to_string());
        }
        if out.when.is_some() || out.merge.is_some() || out.optional {
            return Err("ツリー output には when / merge / optional を指定できません".to_string());
        }
        if !matches!(&out.output, Some(StepSource::Path(_))) {
            return Err(
                "ツリー output は cmd ではなくパスである必要があります（ツリーを標準入力へ渡すことは \
                 できません）"
                    .to_string(),
            );
        }
        if self.format.is_some() {
            return Err(
                "ツリーユニットに format は指定できません（merge / format はバイト内容の step のみ）"
                    .to_string(),
            );
        }
        Ok(())
    }

    /// バイト内容パイプライン（非ツリー）の merge / format / optional を検証する。
    fn validate_pipeline(&self) -> Result<(), String> {
        // merge: 最初の input と output には禁止・2 つ目以降の input には必須。
        let mut input_index = 0usize;
        for (i, step) in self.steps.iter().enumerate() {
            if step.input.is_some() {
                if input_index == 0 && step.merge.is_some() {
                    return Err(format!(
                        "steps[{i}]: 最初の input step は merge を持てません（最初の input は内容の土台）"
                    ));
                }
                if input_index >= 1 && step.merge.is_none() {
                    return Err(format!(
                        "steps[{i}]: 2 つ目以降の input step は merge が必須です（暗黙の合成規則を持たない）"
                    ));
                }
                input_index += 1;
            }
            if step.output.is_some() && step.merge.is_some() {
                return Err(format!("steps[{i}]: output step は merge を持てません"));
            }
        }

        // format: merge を使うユニットに必須・使わないユニットには禁止。
        let any_merge = self.steps.iter().any(|s| s.merge.is_some());
        match (self.format, any_merge) {
            (Some(_), false) => {
                return Err(
                    "format は merge を宣言する step が無いユニットには書けません（merge を使うユニット \
                     のみ）"
                        .to_string(),
                );
            }
            (None, true) => {
                return Err(
                    "merge を宣言する step があるユニットには format（json / plist / text）が必要です"
                        .to_string(),
                );
            }
            _ => {}
        }
        // 各 merge の値と format の両立（shallow ↔ json/plist・append ↔ text）。
        for (i, step) in self.steps.iter().enumerate() {
            if let Some(merge) = step.merge {
                let compatible = matches!(
                    (merge, self.format),
                    (Merge::Shallow, Some(Format::Json | Format::Plist))
                        | (Merge::Append, Some(Format::Text))
                );
                if !compatible {
                    let format = self
                        .format
                        .map_or_else(|| "（未設定）".to_string(), |f| f.to_string());
                    return Err(format!(
                        "steps[{i}]: merge = \"{merge}\" は format = \"{format}\" と両立しません（shallow は \
                         json / plist・append は text）"
                    ));
                }
            }
        }

        // optional は ~ 起点のパス input のみ。単位相対ファイルは repo 追跡の静的ファイルで、typo に
        // よる不在を「無ければ飛ばす」で恒久的に握り潰してしまう（optional の正当な用途は宛先の現在
        // 内容＝初回 apply 前は未在り得る ~ 起点ファイルを土台に読むことだけ）。
        for (i, step) in self.steps.iter().enumerate() {
            if step.optional {
                match &step.input {
                    Some(StepSource::Path(p)) if p == "~" || p.starts_with("~/") => {}
                    Some(StepSource::Path(_)) => {
                        return Err(format!(
                            "steps[{i}]: optional は ~ 起点の input にのみ使えます（単位相対ファイルは \
                             repo 追跡の静的ファイルで、不在は typo の可能性が高く step を恒久的に握り潰す）"
                        ));
                    }
                    Some(StepSource::Cmd(_)) => {
                        return Err(format!(
                            "steps[{i}]: optional は cmd input には使えません（~ 起点のパス input のみ）"
                        ));
                    }
                    None => {
                        return Err(format!(
                            "steps[{i}]: optional は output step には使えません（~ 起点のパス input のみ）"
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// この単位の配置ファイルへ与える Unix パーミッション（8 進）。
    ///
    /// base は `private` で決まる（0600 / 0644）。`executable` のとき、read ビットが
    /// 立っている桁へ execute ビットを足す（0644→0755 / 0600→0700）。`private` /
    /// `executable` 属性の合成規則。
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

/// output パスの表記検証（#579）: `~` / `~/...` のみ。`$` 含み・絶対・相対・`~/` 後の `..` は load
/// エラー（`..` は `~` 起点でも home の外へ脱出できてしまうため弾く）。
fn validate_output_path(p: &str) -> Result<(), String> {
    if p.contains('$') {
        return Err(format!(
            "output パス `{p}` に `$` を含めることはできません（環境変数展開は不採用）"
        ));
    }
    if p == "~" {
        return Ok(());
    }
    if let Some(rest) = p.strip_prefix("~/") {
        if has_parent_component(rest) {
            return Err(format!(
                "output パス `{p}` に `..`（親ディレクトリ参照）は使えません（home の外を指す）"
            ));
        }
        return Ok(());
    }
    Err(format!(
        "output パス `{p}` は ~ 起点（~ または ~/...）である必要があります（絶対パス・相対パスは不可）"
    ))
}

/// input パスの表記検証（#579）: `~` / `~/...`（home 起点）または単位相対（`~` プレフィックス無し・
/// 絶対でない・`$` を含まない）。特例 `"."` はツリーを表し、単位相対として許容される。`~/` 起点・
/// 単位相対のいずれも `..`（home / 単位ディレクトリの外へ脱出）を弾き、単位相対では加えて空文字列
/// （実行時に単位ディレクトリ自身を指し無意味）も弾く。
fn validate_input_path(p: &str) -> Result<(), String> {
    if p.contains('$') {
        return Err(format!(
            "input パス `{p}` に `$` を含めることはできません（環境変数展開は不採用）"
        ));
    }
    if p == "~" {
        return Ok(());
    }
    if let Some(rest) = p.strip_prefix("~/") {
        if has_parent_component(rest) {
            return Err(format!(
                "input パス `{p}` に `..`（親ディレクトリ参照）は使えません（home の外を指す）"
            ));
        }
        return Ok(());
    }
    if p.starts_with('~') {
        return Err(format!(
            "input パス `{p}` が不正です（~ 起点は ~ または ~/... のみ）"
        ));
    }
    if p.starts_with('/') {
        return Err(format!(
            "input パス `{p}` に絶対パスは使えません（単位相対 または ~ 起点）"
        ));
    }
    // 単位相対（特例 `"."` は `.` 単一成分で `..` を含まないため通る）。
    if p.is_empty() {
        return Err("input パスに空文字列は使えません（単位相対パスが必要）".to_string());
    }
    if has_parent_component(p) {
        return Err(format!(
            "input パス `{p}` に `..`（親ディレクトリ参照）は使えません（単位ディレクトリの外を指す）"
        ));
    }
    Ok(())
}

/// パス文字列が `..`（親ディレクトリ参照）成分を含むか。単位ディレクトリ / home の外への脱出を弾く。
fn has_parent_component(p: &str) -> bool {
    Path::new(p).components().any(|c| c == Component::ParentDir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    /// パース → validate を一括で通す（load のファイル I/O を介さずに検証）。
    fn parse(toml_src: &str) -> Result<Manifest, String> {
        let manifest: Manifest = toml::from_str(toml_src).map_err(|e| e.to_string())?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// 最小のツリーユニット（`input = "."` ＋ output）。多くのテストのベース。
    const TREE: &str = "[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/x\"\n";

    // ── untagged StepSource の round-trip（設計の土台。ここが崩れると全体が崩れる） ──

    #[test]
    fn step_source_parses_path_and_cmd_forms() {
        // input = "x"（bare string）→ Path、input.cmd = [...]（inline table）→ Cmd。
        let m = parse("[[steps]]\ninput = \"a.txt\"\n[[steps]]\noutput = \"~/x\"\n").unwrap();
        assert!(matches!(&m.steps[0].input, Some(StepSource::Path(p)) if p == "a.txt"));

        let m =
            parse("[[steps]]\ninput.cmd = [\"gh\", \"completion\"]\n[[steps]]\noutput = \"~/x\"\n")
                .unwrap();
        assert!(
            matches!(&m.steps[0].input, Some(StepSource::Cmd(c)) if c.cmd == ["gh", "completion"])
        );

        // output.cmd も同様。
        let m = parse("[[steps]]\ninput.cmd = [\"defaults\", \"export\"]\n[[steps]]\noutput.cmd = [\"defaults\", \"import\"]\n").unwrap();
        assert!(matches!(&m.steps[1].output, Some(StepSource::Cmd(_))));
    }

    // ── Format / Merge の Display ↔ serde round-trip ──

    #[test]
    fn format_display_round_trips_through_serde() {
        // Display（表示名の出所）が serde の受理トークンと一致することを固定する。ズレると
        // apply / list の表示が manifest に書ける値からズレる。
        for format in Format::iter() {
            let src = format!(
                "format = \"{format}\"\n[[steps]]\ninput = \"a\"\n[[steps]]\ninput = \"b\"\nmerge = \"{}\"\n[[steps]]\noutput = \"~/x\"\n",
                if format == Format::Text {
                    "append"
                } else {
                    "shallow"
                },
            );
            let parsed = parse(&src).unwrap();
            assert_eq!(
                parsed.format,
                Some(format),
                "Display と serde 表現がズレている: {format}"
            );
        }
    }

    #[test]
    fn merge_display_round_trips_through_serde() {
        for merge in Merge::iter() {
            let format = if merge == Merge::Append {
                "text"
            } else {
                "json"
            };
            let src = format!(
                "format = \"{format}\"\n[[steps]]\ninput = \"a\"\n[[steps]]\ninput = \"b\"\nmerge = \"{merge}\"\n[[steps]]\noutput = \"~/x\"\n",
            );
            let parsed = parse(&src).unwrap();
            assert_eq!(
                parsed.steps[1].merge,
                Some(merge),
                "Display と serde 表現がズレている: {merge}"
            );
        }
    }

    // ── shape ──

    #[test]
    fn accepts_minimal_tree() {
        assert!(parse(TREE).is_ok());
    }

    #[test]
    fn rejects_step_with_both_input_and_output() {
        let err = parse("[[steps]]\ninput = \"a\"\noutput = \"~/x\"\n").unwrap_err();
        assert!(
            err.contains("ちょうど 1 つ"),
            "input+output 併記が弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_step_with_neither_input_nor_output() {
        let err = parse("[[steps]]\nwhen = { deps = [\"x\"] }\n").unwrap_err();
        assert!(
            err.contains("ちょうど 1 つ"),
            "空 step が弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_no_input_or_no_output() {
        // output だけ（input 0）。
        let err = parse("[[steps]]\noutput = \"~/x\"\n").unwrap_err();
        assert!(err.contains("input"), "input 0 が弾かれていない: {err}");
        // input だけ（output 0）。
        let err = parse("[[steps]]\ninput = \"a\"\n").unwrap_err();
        assert!(err.contains("output"), "output 0 が弾かれていない: {err}");
        // 空 steps（省略）。
        assert!(parse("private = true\n").is_err());
    }

    // ── path validation（#579） ──

    #[test]
    fn accepts_valid_paths() {
        // input: 単位相対・~ 起点、output: ~ 起点。
        assert!(
            parse("[[steps]]\ninput = \"sub/dir/a.txt\"\n[[steps]]\noutput = \"~/.config/x\"\n")
                .is_ok()
        );
        assert!(
            parse("[[steps]]\ninput = \"~/.claude/settings.json\"\n[[steps]]\noutput = \"~\"\n")
                .is_ok()
        );
    }

    #[test]
    fn rejects_output_non_tilde() {
        for bad in ["out.txt", "/etc/x", "$HOME/x"] {
            let err = parse(&format!(
                "[[steps]]\ninput = \"a\"\n[[steps]]\noutput = \"{bad}\"\n"
            ))
            .unwrap_err();
            assert!(
                err.contains("output パス"),
                "不正 output `{bad}` が弾かれていない: {err}"
            );
        }
    }

    #[test]
    fn rejects_input_absolute_or_dollar() {
        let err =
            parse("[[steps]]\ninput = \"/etc/x\"\n[[steps]]\noutput = \"~/x\"\n").unwrap_err();
        assert!(
            err.contains("絶対パス"),
            "絶対 input が弾かれていない: {err}"
        );
        let err =
            parse("[[steps]]\ninput = \"$XDG/x\"\n[[steps]]\noutput = \"~/x\"\n").unwrap_err();
        assert!(err.contains("$"), "$ 含み input が弾かれていない: {err}");
    }

    #[test]
    fn rejects_input_parent_escape_or_empty() {
        // 単位相対で `..`（単位ディレクトリの外へ脱出）は不可。
        let err = parse("[[steps]]\ninput = \"../../outside\"\n[[steps]]\noutput = \"~/x\"\n")
            .unwrap_err();
        assert!(
            err.contains(".."),
            "`..` を含む input が弾かれていない: {err}"
        );
        // `~/` 起点でも `..` で home の外へ脱出するのは不可（output と同じ規則）。
        let err = parse("[[steps]]\ninput = \"~/../etc/passwd\"\n[[steps]]\noutput = \"~/x\"\n")
            .unwrap_err();
        assert!(
            err.contains(".."),
            "`..` を含む ~ 起点 input が弾かれていない: {err}"
        );
        // 空文字列は不可（`.` はツリーの特例として別途許容される）。
        let err = parse("[[steps]]\ninput = \"\"\n[[steps]]\noutput = \"~/x\"\n").unwrap_err();
        assert!(err.contains("空"), "空 input パスが弾かれていない: {err}");
        // 単位相対の `.`（ツリー）は引き続き許容される。
        assert!(parse(TREE).is_ok());
    }

    #[test]
    fn rejects_output_parent_escape() {
        // `~/` 起点でも `..` で home の外へ脱出するのは不可。
        let err =
            parse("[[steps]]\ninput = \"a\"\n[[steps]]\noutput = \"~/../etc/x\"\n").unwrap_err();
        assert!(
            err.contains(".."),
            "`..` を含む output が弾かれていない: {err}"
        );
    }

    // ── merge / format ──

    #[test]
    fn accepts_text_append_pipeline() {
        assert!(
            parse(
                "format = \"text\"\n\
                 [[steps]]\ninput = \"a\"\n\
                 [[steps]]\ninput = \"b\"\nmerge = \"append\"\n\
                 [[steps]]\noutput = \"~/x\"\n",
            )
            .is_ok()
        );
    }

    #[test]
    fn rejects_merge_on_first_input() {
        let err = parse(
            "format = \"json\"\n[[steps]]\ninput = \"a\"\nmerge = \"shallow\"\n[[steps]]\noutput = \"~/x\"\n",
        )
        .unwrap_err();
        assert!(
            err.contains("最初の input"),
            "最初の input の merge が弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_missing_merge_on_second_input() {
        let err = parse(
            "format = \"json\"\n[[steps]]\ninput = \"a\"\n[[steps]]\ninput = \"b\"\n[[steps]]\noutput = \"~/x\"\n",
        )
        .unwrap_err();
        assert!(
            err.contains("2 つ目以降"),
            "2 つ目 input の merge 欠落が弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_merge_on_output() {
        let err = parse(
            "format = \"json\"\n[[steps]]\ninput = \"a\"\n[[steps]]\noutput = \"~/x\"\nmerge = \"shallow\"\n",
        )
        .unwrap_err();
        assert!(
            err.contains("output step は merge"),
            "output の merge が弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_format_without_merge() {
        let err =
            parse("format = \"json\"\n[[steps]]\ninput = \"a\"\n[[steps]]\noutput = \"~/x\"\n")
                .unwrap_err();
        assert!(
            err.contains("format"),
            "merge 無しの format が弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_merge_without_format() {
        let err = parse(
            "[[steps]]\ninput = \"a\"\n[[steps]]\ninput = \"b\"\nmerge = \"shallow\"\n[[steps]]\noutput = \"~/x\"\n",
        )
        .unwrap_err();
        assert!(
            err.contains("format"),
            "format 無しの merge が弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_merge_format_mismatch() {
        // shallow ↔ text は非両立。
        let err = parse(
            "format = \"text\"\n[[steps]]\ninput = \"a\"\n[[steps]]\ninput = \"b\"\nmerge = \"shallow\"\n[[steps]]\noutput = \"~/x\"\n",
        )
        .unwrap_err();
        assert!(
            err.contains("両立しません"),
            "shallow×text が弾かれていない: {err}"
        );
        // append ↔ json も非両立。
        let err = parse(
            "format = \"json\"\n[[steps]]\ninput = \"a\"\n[[steps]]\ninput = \"b\"\nmerge = \"append\"\n[[steps]]\noutput = \"~/x\"\n",
        )
        .unwrap_err();
        assert!(
            err.contains("両立しません"),
            "append×json が弾かれていない: {err}"
        );
    }

    // ── optional ──

    #[test]
    fn accepts_optional_on_path_input() {
        assert!(
            parse(
                "format = \"json\"\n\
                 [[steps]]\ninput = \"~/.claude/settings.json\"\noptional = true\n\
                 [[steps]]\ninput = \"settings.json\"\nmerge = \"shallow\"\n\
                 [[steps]]\noutput = \"~/.claude/settings.json\"\n",
            )
            .is_ok()
        );
    }

    #[test]
    fn rejects_optional_on_cmd_input() {
        let err =
            parse("[[steps]]\ninput.cmd = [\"x\"]\noptional = true\n[[steps]]\noutput = \"~/x\"\n")
                .unwrap_err();
        assert!(
            err.contains("optional"),
            "cmd input の optional が弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_optional_on_output() {
        let err = parse("[[steps]]\ninput = \"a\"\n[[steps]]\noutput = \"~/x\"\noptional = true\n")
            .unwrap_err();
        assert!(
            err.contains("optional"),
            "output の optional が弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_optional_on_unit_relative_input() {
        // 単位相対パス input への optional は不可（repo 追跡の静的ファイルなので不在は typo の
        // 可能性が高く、恒久的に step を握り潰す）。~ 起点の input だけが optional を持てる。
        let err = parse(
            "format = \"json\"\n\
             [[steps]]\ninput = \"settings.json\"\noptional = true\n\
             [[steps]]\ninput = \"~/.claude/settings.json\"\nmerge = \"shallow\"\n\
             [[steps]]\noutput = \"~/.claude/settings.json\"\n",
        )
        .unwrap_err();
        assert!(
            err.contains("optional") && err.contains("~ 起点"),
            "単位相対 input の optional が弾かれていない: {err}"
        );
    }

    // ── tree special case ──

    #[test]
    fn rejects_tree_with_extra_step() {
        let err = parse(
            "[[steps]]\ninput = \".\"\n[[steps]]\ninput = \"a\"\n[[steps]]\noutput = \"~/x\"\n",
        )
        .unwrap_err();
        assert!(
            err.contains("ちょうど 2 つ"),
            "3 step のツリーが弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_tree_input_not_first() {
        let err = parse("[[steps]]\noutput = \"~/x\"\n[[steps]]\ninput = \".\"\n").unwrap_err();
        assert!(
            err.contains("最初の step"),
            "先頭でないツリー input が弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_tree_with_step_when_or_optional() {
        let err = parse(
            "[[steps]]\ninput = \".\"\nwhen = { deps = [\"x\"] }\n[[steps]]\noutput = \"~/x\"\n",
        )
        .unwrap_err();
        assert!(
            err.contains("when / merge / optional"),
            "ツリー input の when が弾かれていない: {err}"
        );
        let err = parse(
            "[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/x\"\nwhen = { deps = [\"x\"] }\n",
        )
        .unwrap_err();
        assert!(
            err.contains("ツリー output"),
            "ツリー output の when が弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_tree_input_with_merge() {
        // ツリー input（input = "."）に merge を書くのも when/optional と同様に禁止（バイト内容の
        // 合成語彙はツリーに意味を持たない）。
        let err =
            parse("[[steps]]\ninput = \".\"\nmerge = \"shallow\"\n[[steps]]\noutput = \"~/x\"\n")
                .unwrap_err();
        assert!(
            err.contains("when / merge / optional"),
            "ツリー input の merge が弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_tree_with_format() {
        let err =
            parse("format = \"text\"\n[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/x\"\n")
                .unwrap_err();
        assert!(
            err.contains("ツリー"),
            "ツリー + format が弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_tree_output_cmd() {
        let err = parse("[[steps]]\ninput = \".\"\n[[steps]]\noutput.cmd = [\"x\"]\n").unwrap_err();
        assert!(
            err.contains("パス"),
            "ツリー output の cmd が弾かれていない: {err}"
        );
    }

    // ── private / executable ──

    #[test]
    fn accepts_private_with_path_output() {
        assert!(parse(&format!("private = true\n{TREE}")).is_ok());
    }

    #[test]
    fn rejects_private_without_path_output() {
        // cmd output だけのユニット（stats 相当）で private を付ける。
        let err = parse(
            "private = true\n[[steps]]\ninput.cmd = [\"x\"]\n[[steps]]\noutput.cmd = [\"y\"]\n",
        )
        .unwrap_err();
        assert!(
            err.contains("private"),
            "パス output 無しの private が弾かれていない: {err}"
        );
    }

    // ── step cmd 非空 ──

    #[test]
    fn accepts_nonempty_step_cmd() {
        assert!(parse("[[steps]]\ninput.cmd = [\"x\"]\n[[steps]]\noutput.cmd = [\"y\"]\n").is_ok());
    }

    #[test]
    fn rejects_empty_input_cmd() {
        // input.cmd = [] は cmd::run の cmd[0] で panic するため load 時に弾く。
        let err = parse("[[steps]]\ninput.cmd = []\n[[steps]]\noutput = \"~/x\"\n").unwrap_err();
        assert!(
            err.contains("steps[0].input.cmd") && err.contains("非空"),
            "空の input.cmd が弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_empty_output_cmd() {
        // output.cmd = [] は cmd::run_piped の cmd[0] で panic するため load 時に弾く。
        let err = parse("[[steps]]\ninput = \"a\"\n[[steps]]\noutput.cmd = []\n").unwrap_err();
        assert!(
            err.contains("steps[1].output.cmd") && err.contains("非空"),
            "空の output.cmd が弾かれていない: {err}"
        );
    }

    // ── hooks ──

    #[test]
    fn accepts_command_hook() {
        let m = parse(&format!("hooks = [{{ cmd = [\"cmd\", \"sub\"] }}]\n{TREE}")).unwrap();
        assert_eq!(m.hooks[0].cmd, ["cmd", "sub"]);
    }

    #[test]
    fn rejects_empty_hook() {
        let err = parse(&format!("hooks = [{{ cmd = [] }}]\n{TREE}")).unwrap_err();
        assert!(
            err.contains("hooks") && err.contains("非空"),
            "空コマンドが弾かれていない: {err}"
        );
    }

    // ── when effective key ──

    #[test]
    fn accepts_unit_when_and_step_when() {
        assert!(
            parse(&format!(
                "when = {{ deps = [\"gh\"], os = \"darwin\" }}\n{TREE}"
            ))
            .is_ok()
        );
        assert!(
            parse(
                "format = \"json\"\n\
                 [[steps]]\ninput = \"a\"\n\
                 [[steps]]\ninput = \"b\"\nwhen = { deps = [\"rtk\"] }\nmerge = \"shallow\"\n\
                 [[steps]]\noutput = \"~/x\"\n",
            )
            .is_ok()
        );
    }

    #[test]
    fn rejects_empty_unit_when() {
        assert!(parse(&format!("when = {{}}\n{TREE}")).is_err());
        let err = parse(&format!("when = {{ deps = [] }}\n{TREE}")).unwrap_err();
        assert!(
            err.contains("実効キー"),
            "空 unit when が弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_empty_step_when() {
        let err = parse(
            "format = \"json\"\n[[steps]]\ninput = \"a\"\n[[steps]]\ninput = \"b\"\nwhen = {}\nmerge = \"shallow\"\n[[steps]]\noutput = \"~/x\"\n",
        )
        .unwrap_err();
        assert!(
            err.contains("実効キー"),
            "空 step when が弾かれていない: {err}"
        );
    }

    // ── legacy vocabulary rejected via deny_unknown_fields ──

    #[test]
    fn rejects_legacy_vocabulary() {
        // 旧スキーマの語彙は全て未知フィールドとして parse 時に弾かれる（後方互換なし）。
        for legacy in [
            "dst = \"~/x\"\n",
            "kind = \"generate\"\n",
            "strategy = \"concat\"\n",
            "preserve = true\n",
            "cmd = [\"x\"]\n",
            "sensitive = [\"a\"]\n",
            "[[overlay]]\nsrc = \"a\"\n",
        ] {
            let src = format!("{legacy}{TREE}");
            assert!(
                toml::from_str::<Manifest>(&src).is_err(),
                "旧語彙が弾かれていない: {legacy}"
            );
        }
        // hooks[].frequency も Hook の deny_unknown_fields で弾く。
        assert!(
            toml::from_str::<Manifest>(&format!(
                "{TREE}[[hooks]]\ncmd = [\"x\"]\nfrequency = \"always\"\n"
            ))
            .is_err()
        );
    }
}
