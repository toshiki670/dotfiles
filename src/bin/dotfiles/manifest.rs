//! `manifest.toml` のスキーマと読み込み。
//!
//! ユニットを `[[steps]]` の列として解釈する。内容を空から始め、宣言順に各 step を畳む:
//! - **input** step: 内容へ中身を畳む（最初の input は内容＝中身、2 つ目以降は `merge` で重ねる）。
//! - **output** step: 内容を宛先へ書く。
//!
//! 各 step の `input` / `output` は択一で、どちらも「パス」か「`cmd`（argv・標準入出力）」の
//! 択一（[`InputSource`] / [`OutputSource`]）。重ね方の内容型は unit レベルの `format`（json / plist / text）が決め、
//! per-step の `merge`（shallow / deep / append）が「どう重ねるか」を step ごとに選ぶ ― 同じ unit の
//! 中で 2 つ目の input が `shallow`、3 つ目が `deep` のように混在できる。実行時の畳み込みの仕組みは
//! [`crate::apply::pipeline`]。
//!
//! gate 語彙は `when`（`deps` 配列 ＝ AND / `os` スカラ / `profile` スカラ）に一本化する。**書く位置で
//! スコープが決まる**: トップレベルの `when` はユニット全体 gate（false ならユニットごと skip ＝
//! all-or-nothing）、step 内の `when` はその step だけの採否。両者は同じ評価規則を
//! [`crate::apply::gate`] で共有する。`profile` は環境からその場で判る条件（`deps`/`os`）と違い
//! user が選んでおく状態（[`crate::state`]）を読む状態 gate で、`theme`（color スライスまで
//! 未配線）と同族。
//!
//! `locals` はマシンローカル値（named value）の宣言。

use serde::Deserialize;
use std::path::{Component, Path, PathBuf};
use strum::{Display, EnumIter};

/// 1 つの設定単位（`manifest.toml` を持つディレクトリ）の配置仕様（解釈済みの確定形）。
///
/// [`Manifest::load`] が生スキーマ（[`RawManifest`]）を検証しながらこの型へ解釈する。パースは
/// 通るが意味的に不正な形は load 時に弾かれ、その多くはそもそも表現できない（[`Steps`] /
/// [`Step`] が択一を variant で持つ）ため、この型の値は常に配置可能な宣言を表す。
#[derive(Debug)]
pub struct Manifest {
    /// ユニット全体 gate。トップレベルに書いた `when` はユニットスコープで、
    /// 満たさなければユニット全体を skip する（配置を一切行わない ＝ all-or-nothing）。
    /// `when.deps`（配列・AND）が PATH に揃い、`when.os`（スカラ）が現在 OS と一致し、
    /// `when.profile`（スカラ・状態）が現在の profile 状態と一致した時だけ真。省略時は常時採用。
    /// 同じ語彙を各 step の `when` がその step スコープで再利用する。
    pub when: Option<When>,
    /// マシンローカル値（named value）の宣言。ここで宣言した名前 `n` に対し、この単位の
    /// 配置ファイル中の `@@n@@` をストア（`~/.config/dotfiles/local.toml`）の値で置換する。
    /// 置換は `locals` を宣言した単位のファイルにだけ走る（無関係ファイルの `@@…@@` を巻き込まない）。
    /// 例: `locals = ["git.email", "git.name"]`。
    pub locals: Vec<String>,
    /// 配置の形。`[[steps]]` の列を解釈した結果（ツリー配置 or バイト内容パイプライン）。
    pub steps: Steps,
    /// 所有者のみアクセス可（0600 相当）。省略時 false。パス output を持つユニットのみ。
    pub private: bool,
    /// 実行ビットを付与（0644→0755 / 0600→0700）。省略時 false。パス output を持つユニットのみ。
    pub executable: bool,
}

/// 配置の形。`[[steps]]` の列は load 時にどちらかへ解釈される。ツリー配置とバイト内容パイプライン
/// は持てる語彙が違い、混在もできないため、別 variant で表現する。
#[derive(Debug)]
pub enum Steps {
    /// ツリー配置（`input = "."` ＋ パス output ＋ 任意の末尾 output.cmd）: 単位ディレクトリを丸ごと、
    /// 相対構造を保って配置し（[`crate::apply::copy`]）、宣言されていれば配置後に `post` の各コマンドを
    /// 無条件で実行する。バイト内容の合成語彙（merge / format）はツリーに意味を持たず、step の `when` は
    /// unit gate と冗長、`optional` の対象になる不在（repo 追跡の単位ディレクトリは常に在る）も無いため、
    /// input / パス output・末尾 output.cmd のどれに付けても load 時に弾かれる。
    Tree {
        /// 配置先ディレクトリ（検証済みの `~` 起点パス）。
        output: HomePath,
        /// 配置後に**宣言順**で実行する output.cmd（argv）。ツリーは内容を持たないため標準入力は空。
        /// 毎 apply 無条件に走る副作用で、コマンドが冪等であることを契約とする（bat cache 再構築・
        /// symlink 生成など）。ユニット gate が false なら配置ごと skip されこれも走らない。
        post: Vec<CmdSource>,
    },
    /// バイト内容パイプライン: 内容を空から始め、宣言順に input（読む）→ output（書く）を畳む
    /// （[`crate::apply::pipeline`]）。input 1 つ以上・output 1 つ以上。
    Pipeline {
        /// 合成の内容型（`merge` を使うユニットに必須・使わないユニットには書けない）。
        /// `json` / `plist` / `text`。実行時の畳み込みのバイト種別（内容の解釈のしかた）を決め、
        /// 「どう重ねるか」は各 input step 自身の `merge` が選ぶ（[`InputStep::merge`]）。
        format: Option<Format>,
        /// step 列（宣言順）。
        steps: Vec<Step>,
    },
}

/// 配置パイプラインの 1 step。input（読む）と output（書く）の択一を variant で表現する
/// （manifest.toml 上の併記・欠落は load 時に弾かれ、ここには到達しない）。
#[derive(Debug)]
pub enum Step {
    /// 読む: 中身を内容へ畳む。
    Input(InputStep),
    /// 書く: 内容を宛先へ書く。
    Output(OutputStep),
}

/// input step。読んだ中身を内容へ畳む。
#[derive(Debug)]
pub struct InputStep {
    /// 読む内容源（検証済みパス or cmd 標準出力）。
    pub source: InputSource,
    /// 重ね方（2 つ目以降の input に必須・最初の input には禁止）。値は `format` と両立する
    /// ものを選ぶ: json / plist → `shallow` | `deep`、text → `append`。この step の畳み込みを
    /// そのまま駆動する（[`crate::apply::pipeline`]）。
    pub merge: Option<Merge>,
    /// この step の採否（省略 = 常時採用）。unit gate と共通の語彙（`deps` / `os` / `profile`）。
    pub when: Option<When>,
    /// `~` 起点のパス input が存在しなければこの step を飛ばす（次の input が土台になる）。既定は
    /// 「無ければエラー」。`~` 起点のパス input のみ有効 ― 単位相対ファイルは repo 追跡の静的ファイルで
    /// 不在は typo の可能性が高く、恒久的に step を握り潰すため、cmd input への指定と併せて load 時
    /// エラー。唯一の正当な用途は宛先の現在内容（初回 apply 前は未在り得る `~` 起点ファイル）を
    /// 土台に読むこと。
    pub optional: bool,
}

/// output step。組み立てた内容を宛先へ書く。`merge` / `optional` は持たない（重ね方は input 側の
/// 語彙で、output の書き込みは常に実行される）。manifest.toml 上の指定は load 時に弾かれる。
#[derive(Debug)]
pub struct OutputStep {
    /// 書く宛先（検証済み home 起点パス or cmd 標準入力）。
    pub dest: OutputSource,
    /// この step の採否（省略 = 常時採用）。unit gate と共通の語彙（`deps` / `os` / `profile`）。
    pub when: Option<When>,
}

/// 検証済みの home 起点パス（`~` または `~/...`）。output はこの表記のみ許容し、input の home 分岐も
/// これを再利用する。`None` は `~` 単体、`Some(rest)` は `~/rest`（`rest` が空なら `~/`）。
#[derive(Debug, Clone)]
pub struct HomePath(Option<String>);

impl HomePath {
    /// output パスの表記をパースする。`$` 含み・絶対・相対・`~/` 後の `..` は load エラー
    /// （`..` は `~` 起点でも home の外へ脱出できてしまうため弾く）。
    fn parse(p: &str) -> Result<Self, String> {
        if p.contains('$') {
            return Err(format!(
                "output パス `{p}` に `$` を含めることはできません（環境変数展開は不採用）"
            ));
        }
        if p == "~" {
            return Ok(Self(None));
        }
        let Some(rest) = p.strip_prefix("~/") else {
            return Err(format!(
                "output パス `{p}` は ~ 起点（~ または ~/...）である必要があります（絶対パス・相対パスは不可）"
            ));
        };
        if has_parent_component(rest) {
            return Err(format!(
                "output パス `{p}` に `..`（親ディレクトリ参照）は使えません（home の外を指す）"
            ));
        }
        Ok(Self(Some(rest.to_string())))
    }

    /// `home` を基点にパスを解決する。`~` 単体・`~/`（空 rest）はどちらも home 自身になる。
    pub(crate) fn resolve(&self, home: &Path) -> PathBuf {
        match &self.0 {
            None => home.to_path_buf(),
            Some(rest) => home.join(rest),
        }
    }
}

impl std::fmt::Display for HomePath {
    /// 解決前の生表記へ戻す（`~` 単体・`~/`・`~/rest` を区別して復元する）。apply の 1 行出力・
    /// `list` の宛先表記が使う。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            None => write!(f, "~"),
            Some(rest) => write!(f, "~/{rest}"),
        }
    }
}

/// 検証済みの input パス表記。home 起点（[`HomePath`]）または単位相対。特例 `"."`（ツリー）は
/// 単位相対として表現される。
#[derive(Debug)]
pub enum InputPath {
    /// home 起点（`~` / `~/...`）。output の [`HomePath`] と表記・解決を共有する。
    Home(HomePath),
    /// 単位相対（`~` プレフィックス無し・絶対でない・`$` を含まない）。`unit_dir` を基点に解決する。
    UnitRelative(String),
}

impl InputPath {
    /// input パスの表記をパースする。home 起点（`~` / `~/...`）または単位相対を受け、`$` 含み・絶対・
    /// 空文字列・`..`（home / 単位ディレクトリの外へ脱出）は load エラー。特例 `"."` は `.` 単一成分で
    /// `..` を含まないため単位相対として通り、[`RawManifest::into_manifest`] がツリーへ振り分ける。
    fn parse(p: &str) -> Result<Self, String> {
        if p.contains('$') {
            return Err(format!(
                "input パス `{p}` に `$` を含めることはできません（環境変数展開は不採用）"
            ));
        }
        if p == "~" {
            return Ok(InputPath::Home(HomePath(None)));
        }
        if let Some(rest) = p.strip_prefix("~/") {
            if has_parent_component(rest) {
                return Err(format!(
                    "input パス `{p}` に `..`（親ディレクトリ参照）は使えません（home の外を指す）"
                ));
            }
            return Ok(InputPath::Home(HomePath(Some(rest.to_string()))));
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
        if p.is_empty() {
            return Err("input パスに空文字列は使えません（単位相対パスが必要）".to_string());
        }
        if has_parent_component(p) {
            return Err(format!(
                "input パス `{p}` に `..`（親ディレクトリ参照）は使えません（単位ディレクトリの外を指す）"
            ));
        }
        Ok(InputPath::UnitRelative(p.to_string()))
    }

    /// input パスを解決する: home 起点は `home`、単位相対は `unit_dir` を基点にする。
    pub(crate) fn resolve(&self, unit_dir: &Path, home: &Path) -> PathBuf {
        match self {
            InputPath::Home(home_path) => home_path.resolve(home),
            InputPath::UnitRelative(rest) => unit_dir.join(rest),
        }
    }
}

impl std::fmt::Display for InputPath {
    /// 解決前の生表記へ戻す（fold のパースエラーに添える input ラベル）。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputPath::Home(home_path) => write!(f, "{home_path}"),
            InputPath::UnitRelative(rest) => write!(f, "{rest}"),
        }
    }
}

/// input step の解釈済み内容源: 検証済みパス（[`InputPath`]）か cmd 標準出力。
#[derive(Debug)]
pub enum InputSource {
    /// 検証済みの input パス（単位相対 or home 起点）。
    Path(InputPath),
    /// cmd の標準出力を読む（argv）。
    Cmd(CmdSource),
}

/// output step の解釈済み内容源: 検証済み home 起点パス（[`HomePath`]）か cmd 標準入力。
#[derive(Debug)]
pub enum OutputSource {
    /// 検証済みの home 起点パス（`~` / `~/...`）。
    Path(HomePath),
    /// 内容を cmd の標準入力へ渡す（argv）。
    Cmd(CmdSource),
}

/// `[[steps]]` の `input` / `output` の生スキーマ（TOML と 1:1）。パス文字列
/// （`"settings.json"` / `"~/.claude/settings.json"`）か `cmd`（`{ cmd = ["gh", "completion", "fish"] }`）
/// の択一。TOML の型（bare string / inline table）で判別するため `untagged` で受ける（src/cmd の
/// 排他は型システムが保証し、実行時検証は要らない）。表記の検証は [`RawManifest::into_manifest`] が
/// 担い、[`InputPath`] / [`HomePath`] へ parse して確定形（[`InputSource`] / [`OutputSource`]）にする。
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum StepSource {
    /// パス: input は単位相対 or `~` 起点、output は `~` 起点のみ。特例として input `"."` は
    /// 「単位ディレクトリツリー」を表す。
    Path(String),
    /// コマンド: input は標準出力を読み、output は内容を標準入力へ渡す（argv）。
    Cmd(CmdSource),
}

/// cmd 内容源（`{ cmd = [...] }`）。`deny_unknown_fields` で `cmd` 以外のキーを弾く。
#[derive(Debug, Deserialize, Clone)]
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
    /// JSON。`merge = "shallow"` / `"deep"` と両立。
    Json,
    /// plist（Apple の property list）。`merge = "shallow"` / `"deep"` と両立。
    Plist,
    /// プレーンテキスト。`merge = "append"` と両立。
    Text,
}

/// 重ね方。表示名と TOML トークンを揃える規則は [`Format`] と同じ。
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Display, EnumIter)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Merge {
    /// トップレベルキー単位で後勝ち置換（json / plist）。
    Shallow,
    /// object はキー単位で再帰マージ（後勝ち）・配列は step 順に連結（dedup・位置対応はしない）・
    /// スカラおよび型不一致は後勝ち（json / plist）。
    Deep,
    /// テキスト連結（境目に改行 1 つ）。
    Append,
}

/// `when.os` が受理する OS。表示名と TOML トークンを揃える規則は [`Format`] と同じ。
///
/// 受理値を型で閉じ、typo を load 時に弾く ― 任意文字列を受けると、どの環境とも一致しない値が
/// ユニット / step を黙って恒久 skip させる。
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Display, EnumIter)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Os {
    /// macOS（Rust の `std::env::consts::OS` は `macos`）。
    Darwin,
    /// Linux。
    Linux,
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
    /// OS（スカラ）。現在 OS と一致時だけ採用（[`Os`]）。
    #[serde(default)]
    pub os: Option<Os>,
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
    /// load 時に弾くため [`RawManifest::into_manifest`] が使う。`theme` 等のキー追加時はここに足す。
    fn has_no_effective_key(&self) -> bool {
        self.deps.is_empty() && self.os.is_none() && self.profile.is_none()
    }
}

impl Manifest {
    /// `manifest.toml` を読み込む。TOML パース（[`RawManifest`]）→ 検証・解釈
    /// （[`std::str::FromStr`] 実装）。
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("{}: 読み込み失敗: {e}", path.display()))?;
        text.parse().map_err(|e| format!("{}: {e}", path.display()))
    }

    /// apply の 1 行出力と `list` が共有する宛先表記: 最初のパス output の生表記（`~/...`）。
    /// パス output を持たない（cmd output だけの）ユニットは `(cmd)` を返す。
    pub fn display_dst(&self) -> String {
        match &self.steps {
            Steps::Tree { output, .. } => output.to_string(),
            Steps::Pipeline { steps, .. } => steps
                .iter()
                .find_map(|step| match step {
                    Step::Output(OutputStep {
                        dest: OutputSource::Path(p),
                        ..
                    }) => Some(p.to_string()),
                    _ => None,
                })
                .unwrap_or_else(|| "(cmd)".to_string()),
        }
    }

    /// apply のラベルと `list` の属性が共有する steps サマリ。ツリーは `tree`（末尾 output.cmd が
    /// あれば `tree, output.cmd=N`）、それ以外は `steps=Nin/Mout`（＋ `format` ＋ cmd output があれば
    /// `output=cmd`）。
    pub fn summary(&self) -> String {
        let (format, steps) = match &self.steps {
            Steps::Tree { post, .. } => {
                return if post.is_empty() {
                    "tree".to_string()
                } else {
                    format!("tree, output.cmd={}", post.len())
                };
            }
            Steps::Pipeline { format, steps } => (format, steps),
        };
        let n_in = steps.iter().filter(|s| matches!(s, Step::Input(_))).count();
        let n_out = steps
            .iter()
            .filter(|s| matches!(s, Step::Output(_)))
            .count();
        let mut parts = vec![format!("steps={n_in}in/{n_out}out")];
        if let Some(format) = format {
            parts.push(format.to_string());
        }
        if steps.iter().any(|s| {
            matches!(
                s,
                Step::Output(OutputStep {
                    dest: OutputSource::Cmd(_),
                    ..
                })
            )
        }) {
            parts.push("output=cmd".to_string());
        }
        parts.join(", ")
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

impl std::str::FromStr for Manifest {
    type Err = String;

    /// TOML テキストを [`RawManifest`] へパースし、検証しながら [`Manifest`] へ解釈する。
    /// [`Manifest::load`] とテストが共有する（load はエラーへファイルパスを前置する）。
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let raw: RawManifest = toml::from_str(text).map_err(|e| format!("パース失敗: {e}"))?;
        raw.into_manifest()
    }
}

/// `manifest.toml` の生スキーマ（TOML と 1:1 の serde ミラー）。[`Manifest`] が表現できない形も
/// いったんパースで受け、[`RawManifest::into_manifest`] が検証しながら解釈する。
///
/// `deny_unknown_fields` で未知キーを load 時に弾く。旧スキーマの語彙（`dst` / `kind` / `strategy` /
/// `preserve` / ユニット直下 `cmd` / `[[overlay]]` / `sensitive` 等）が残った manifest はここで
/// エラーになる（後方互換は持たせず、誤連想の再発を防ぐ）。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest {
    #[serde(default)]
    when: Option<When>,
    #[serde(default)]
    locals: Vec<String>,
    #[serde(default)]
    format: Option<Format>,
    #[serde(default)]
    steps: Vec<RawStep>,
    #[serde(default)]
    private: bool,
    #[serde(default)]
    executable: bool,
}

/// `[[steps]]` の生スキーマ。`input` / `output` の択一は TOML では表現できない（どちらも任意キー）
/// ため、ここでは両方を受けて [`RawStep::into_sided`] が解決する。`deny_unknown_fields` でキーの
/// typo を load 時に弾く。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawStep {
    #[serde(default)]
    input: Option<StepSource>,
    #[serde(default)]
    output: Option<StepSource>,
    #[serde(default)]
    merge: Option<Merge>,
    #[serde(default)]
    when: Option<When>,
    #[serde(default)]
    optional: bool,
}

/// 向き（input / output）を解決した中間形の step。ツリー / パイプラインへの解釈前で、`merge` /
/// `optional` は output 側にも保持する ― output への指定は不正だが、弾く文言がツリーと
/// パイプラインで違うため、判定は解釈側（[`tree_steps`] / [`pipeline_steps`]）に置く。
struct SidedStep {
    /// step の向きと内容源。
    side: Side,
    merge: Option<Merge>,
    when: Option<When>,
    optional: bool,
}

/// step の向き（input / output）と内容源。
enum Side {
    Input(StepSource),
    Output(StepSource),
}

impl RawStep {
    /// `input` / `output` の択一を解決する（`i` はエラー表示用の step 位置）。両方・どちらも無しは
    /// typo として弾く。
    fn into_sided(self, i: usize) -> Result<SidedStep, String> {
        let side = match (self.input, self.output) {
            (Some(source), None) => Side::Input(source),
            (None, Some(source)) => Side::Output(source),
            (input, output) => {
                let n = [input.is_some(), output.is_some()]
                    .into_iter()
                    .filter(|&set| set)
                    .count();
                return Err(format!(
                    "steps[{i}] は input / output のうちちょうど 1 つを持つ必要があります（現在 {n} 個）"
                ));
            }
        };
        Ok(SidedStep {
            side,
            merge: self.merge,
            when: self.when,
            optional: self.optional,
        })
    }
}

impl SidedStep {
    /// パス表記をパースして [`ParsedStep`] へ解釈する（#579）。input は [`InputPath`]、output は
    /// [`HomePath`] へ parse し、cmd はそのまま移す。
    fn into_parsed(self) -> Result<ParsedStep, String> {
        let side = match self.side {
            Side::Input(StepSource::Path(p)) => {
                ParsedSide::Input(InputSource::Path(InputPath::parse(&p)?))
            }
            Side::Input(StepSource::Cmd(c)) => ParsedSide::Input(InputSource::Cmd(c)),
            Side::Output(StepSource::Path(p)) => {
                ParsedSide::Output(OutputSource::Path(HomePath::parse(&p)?))
            }
            Side::Output(StepSource::Cmd(c)) => ParsedSide::Output(OutputSource::Cmd(c)),
        };
        Ok(ParsedStep {
            side,
            merge: self.merge,
            when: self.when,
            optional: self.optional,
        })
    }
}

/// パス表記まで parse 済みの中間形 step（[`SidedStep`] を [`SidedStep::into_parsed`] が解釈した形）。
/// `merge` / `optional` を output 側にも保持する理由は [`SidedStep`] と同じ。
struct ParsedStep {
    /// step の向きと検証済み内容源。
    side: ParsedSide,
    merge: Option<Merge>,
    when: Option<When>,
    optional: bool,
}

/// step の向き（input / output）と検証済み内容源。
enum ParsedSide {
    Input(InputSource),
    Output(OutputSource),
}

impl ParsedStep {
    /// 「ツリー input」（`input = "."` ＝ 単位ディレクトリをそのまま配置）か。`"."` が単位相対として
    /// 通る理由は [`InputPath::parse`] を参照。
    fn is_tree_input(&self) -> bool {
        matches!(
            &self.side,
            ParsedSide::Input(InputSource::Path(InputPath::UnitRelative(p))) if p == "."
        )
    }
}

impl RawManifest {
    /// 生スキーマを検証しながら [`Manifest`] へ解釈する。manifest の typo を配置前に弾く（fail-loud）。
    ///
    /// - **各 step は input / output のちょうど 1 つ**（[`RawStep::into_sided`]）。
    /// - **steps は input を 1 つ以上・output を 1 つ以上**持つ（空パイプラインは無意味）。
    /// - **パス表記**（#579）: output は `~` 起点のみ。input は単位相対 or `~` 起点。`$`・絶対は不可。
    /// - **各 step の `cmd`（input.cmd / output.cmd）は非空**。
    /// - **`when` は実効キーを 1 つ以上**（unit・各 step とも。空 when の silent no-op を弾く）。
    /// - **`private` / `executable` はパス output を持つユニットのみ**（cmd output だけのユニットには
    ///   書き込み先ファイルが無い）。
    /// - **ツリー**（`input = "."`）は [`tree_steps`]、それ以外は [`pipeline_steps`] が形を検証して
    ///   [`Steps`] へ解釈する。
    fn into_manifest(self) -> Result<Manifest, String> {
        let sided: Vec<SidedStep> = self
            .steps
            .into_iter()
            .enumerate()
            .map(|(i, step)| step.into_sided(i))
            .collect::<Result<_, _>>()?;
        let n_input = sided
            .iter()
            .filter(|s| matches!(s.side, Side::Input(_)))
            .count();
        let n_output = sided.len() - n_input;
        if n_input == 0 || n_output == 0 {
            return Err(format!(
                "steps は input を 1 つ以上・output を 1 つ以上持つ必要があります（現在 input {n_input}・output {n_output}）"
            ));
        }

        // パス表記（#579）: 全 step の path 内容源を parse し、検証済みの中間形（ParsedStep）へ。
        // 以降の検証・解釈はこの parsed を見る（生文字列の再パースを残さない）。
        let parsed: Vec<ParsedStep> = sided
            .into_iter()
            .map(SidedStep::into_parsed)
            .collect::<Result<_, _>>()?;

        // 各 step の cmd（input.cmd / output.cmd）は非空。空 argv は cmd::run/run_piped の cmd[0]
        // インデックスで panic するため、load 時に弾く（実体化できない typo を静かに無視しない方針）。
        for (i, step) in parsed.iter().enumerate() {
            let (label, empty) = match &step.side {
                ParsedSide::Input(InputSource::Cmd(c)) => ("input", c.cmd.is_empty()),
                ParsedSide::Output(OutputSource::Cmd(c)) => ("output", c.cmd.is_empty()),
                _ => continue,
            };
            if empty {
                return Err(format!(
                    "steps[{i}].{label}.cmd は非空のコマンド（argv）である必要があります"
                ));
            }
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
        if let Some(i) = parsed
            .iter()
            .position(|s| s.when.as_ref().is_some_and(When::has_no_effective_key))
        {
            return Err(format!(
                "steps[{i}] の when は実効キー（deps / os / profile）を 1 つ以上持つ必要があります（空の when は silent no-op）"
            ));
        }

        // private / executable はパス output を持つユニットのみ。
        if (self.private || self.executable)
            && !parsed
                .iter()
                .any(|s| matches!(&s.side, ParsedSide::Output(OutputSource::Path(_))))
        {
            return Err(
                "private / executable はパス output を持つユニットのみに指定できます（cmd output だけの \
                 ユニットには書き込み先ファイルが無い）"
                    .to_string(),
            );
        }

        // ツリー（input = "."）は専用の形（Steps::Tree）へ、それ以外はパイプラインへ解釈する。
        let steps = if parsed.iter().any(ParsedStep::is_tree_input) {
            tree_steps(self.format, &parsed)?
        } else {
            pipeline_steps(self.format, parsed)?
        };

        Ok(Manifest {
            when: self.when,
            locals: self.locals,
            steps,
            private: self.private,
            executable: self.executable,
        })
    }
}

/// ツリー（`input = "."` を含む steps）の形を検証し、[`Steps::Tree`] へ解釈する。先頭はツリー input、
/// 2 番目はパス output、3 番目以降（0 個以上）は配置後に走らせる output.cmd。merge / format /
/// step の `when` / `optional` を禁止する理由は [`Steps::Tree`] の doc。
fn tree_steps(format: Option<Format>, steps: &[ParsedStep]) -> Result<Steps, String> {
    if steps.len() < 2 {
        return Err(format!(
            "ツリー input（input = \".\"）を持つユニットは steps を 2 つ以上（input = \".\" と \
             output、任意で末尾に output.cmd）持つ必要があります（現在 {} 個）",
            steps.len()
        ));
    }
    if !steps[0].is_tree_input() {
        return Err("ツリー input（input = \".\"）は最初の step である必要があります".to_string());
    }
    if steps[0].when.is_some() || steps[0].merge.is_some() || steps[0].optional {
        return Err(
            "ツリー input（input = \".\"）には when / merge / optional を指定できません（ユニット \
             全体の when gate を使ってください）"
                .to_string(),
        );
    }
    let out = &steps[1];
    let ParsedSide::Output(dest) = &out.side else {
        return Err("ツリー input の次の step は output である必要があります".to_string());
    };
    if out.when.is_some() || out.merge.is_some() || out.optional {
        return Err("ツリー output には when / merge / optional を指定できません".to_string());
    }
    let OutputSource::Path(output) = dest else {
        return Err(
            "ツリー output は cmd ではなくパスである必要があります（ツリーを標準入力へ渡すことは \
             できません）"
                .to_string(),
        );
    };
    if format.is_some() {
        return Err(
            "ツリーユニットに format は指定できません（merge / format はバイト内容の step のみ）"
                .to_string(),
        );
    }
    // steps[2..] は配置後に無条件で走る output.cmd だけを持てる（input・パス output は不可、
    // when / merge / optional も不可 ― 分岐はユニット全体の when gate を使う）。
    let mut post = Vec::new();
    for (i, step) in steps.iter().enumerate().skip(2) {
        let ParsedSide::Output(OutputSource::Cmd(c)) = &step.side else {
            return Err(format!(
                "steps[{i}]: ツリーの 3 つ目以降の step は output.cmd である必要があります（input・\
                 パス output は不可 ― ツリーは 1 つの output パスへ配置し、以降は配置後に走らせる \
                 output.cmd だけを持てる）"
            ));
        };
        if step.when.is_some() || step.merge.is_some() || step.optional {
            return Err(format!(
                "steps[{i}]: ツリー末尾の output.cmd には when / merge / optional を指定できません \
                 （配置のたび無条件に走る副作用。分岐はユニット全体の when gate を使ってください）"
            ));
        }
        post.push(c.clone());
    }
    Ok(Steps::Tree {
        output: output.clone(),
        post,
    })
}

/// バイト内容パイプライン（非ツリー）の形（先頭 step）と merge / format / optional を検証し、
/// [`Steps::Pipeline`] へ解釈する。
fn pipeline_steps(format: Option<Format>, steps: Vec<ParsedStep>) -> Result<Steps, String> {
    // 先頭 step は input。ここに来る時点で input・output が各 1 つ以上あるので steps は 2 つ以上 ―
    // `steps[0]` の添字は安全。output が先頭だと、宣言順に畳む apply で内容がまだ空のまま書き込みへ
    // 到達する（実行時エラーになる前に load で弾く）。
    if matches!(steps[0].side, ParsedSide::Output(_)) {
        return Err(
            "steps[0] が output です（先頭は input である必要があります）。output より前に内容を \
             組み立てる input が無く、apply 時に内容が空のまま書き込みに到達します"
                .to_string(),
        );
    }

    // merge: 最初の input と output には禁止・2 つ目以降の input には必須。
    let mut input_index = 0usize;
    for (i, step) in steps.iter().enumerate() {
        match step.side {
            ParsedSide::Input(_) => {
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
            ParsedSide::Output(_) => {
                if step.merge.is_some() {
                    return Err(format!("steps[{i}]: output step は merge を持てません"));
                }
            }
        }
    }

    // format: merge を使うユニットに必須・使わないユニットには禁止。
    let any_merge = steps.iter().any(|s| s.merge.is_some());
    match (format, any_merge) {
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
    // 各 merge の値と format の両立（shallow / deep ↔ json/plist・append ↔ text）。
    for (i, step) in steps.iter().enumerate() {
        if let Some(merge) = step.merge {
            let compatible = matches!(
                (merge, format),
                (
                    Merge::Shallow | Merge::Deep,
                    Some(Format::Json | Format::Plist)
                ) | (Merge::Append, Some(Format::Text))
            );
            if !compatible {
                let format = format.map_or_else(|| "（未設定）".to_string(), |f| f.to_string());
                return Err(format!(
                    "steps[{i}]: merge = \"{merge}\" は format = \"{format}\" と両立しません（shallow / \
                     deep は json / plist・append は text）"
                ));
            }
        }
    }

    // optional は ~ 起点のパス input のみ。単位相対ファイルは repo 追跡の静的ファイルで、typo に
    // よる不在を「無ければ飛ばす」で恒久的に握り潰してしまう（optional の正当な用途は宛先の現在
    // 内容＝初回 apply 前は未在り得る ~ 起点ファイルを土台に読むことだけ）。
    for (i, step) in steps.iter().enumerate() {
        if step.optional {
            match &step.side {
                ParsedSide::Input(InputSource::Path(InputPath::Home(_))) => {}
                ParsedSide::Input(InputSource::Path(InputPath::UnitRelative(_))) => {
                    return Err(format!(
                        "steps[{i}]: optional は ~ 起点の input にのみ使えます（単位相対ファイルは \
                         repo 追跡の静的ファイルで、不在は typo の可能性が高く step を恒久的に握り潰す）"
                    ));
                }
                ParsedSide::Input(InputSource::Cmd(_)) => {
                    return Err(format!(
                        "steps[{i}]: optional は cmd input には使えません（~ 起点のパス input のみ）"
                    ));
                }
                ParsedSide::Output(_) => {
                    return Err(format!(
                        "steps[{i}]: optional は output step には使えません（~ 起点のパス input のみ）"
                    ));
                }
            }
        }
    }

    // 検証済みの step を確定形へ。output 側の merge / optional は上で弾いた後なので現れない。
    let steps = steps
        .into_iter()
        .map(|step| match step.side {
            ParsedSide::Input(source) => Step::Input(InputStep {
                source,
                merge: step.merge,
                when: step.when,
                optional: step.optional,
            }),
            ParsedSide::Output(dest) => Step::Output(OutputStep {
                dest,
                when: step.when,
            }),
        })
        .collect();
    Ok(Steps::Pipeline { format, steps })
}

/// パス文字列が `..`（親ディレクトリ参照）成分を含むか。単位ディレクトリ / home の外への脱出を弾く。
fn has_parent_component(p: &str) -> bool {
    Path::new(p).components().any(|c| c == Component::ParentDir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    /// テキストから Manifest を読み込む（load のファイル I/O を介さず、パース＋解釈を通す）。
    fn parse(toml_src: &str) -> Result<Manifest, String> {
        toml_src.parse()
    }

    /// パイプラインとして解釈された (format, steps) を取り出す（ツリーなら panic）。
    fn pipeline(m: &Manifest) -> (Option<Format>, &[Step]) {
        match &m.steps {
            Steps::Pipeline { format, steps } => (*format, steps),
            Steps::Tree { .. } => panic!("パイプラインを期待したがツリーだった"),
        }
    }

    /// 最小のツリーユニット（`input = "."` ＋ output）。多くのテストのベース。
    const TREE: &str = "[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/x\"\n";

    // ── input/output の untagged 判別（bare string→パス / inline table→cmd。設計の土台） ──

    #[test]
    fn step_source_parses_path_and_cmd_forms() {
        // input = "x"（bare string）→ Path、input.cmd = [...]（inline table）→ Cmd。
        let m = parse("[[steps]]\ninput = \"a.txt\"\n[[steps]]\noutput = \"~/x\"\n").unwrap();
        assert!(matches!(
            &pipeline(&m).1[0],
            Step::Input(InputStep { source: InputSource::Path(p), .. }) if p.to_string() == "a.txt"
        ));

        let m =
            parse("[[steps]]\ninput.cmd = [\"gh\", \"completion\"]\n[[steps]]\noutput = \"~/x\"\n")
                .unwrap();
        assert!(matches!(
            &pipeline(&m).1[0],
            Step::Input(InputStep { source: InputSource::Cmd(c), .. }) if c.cmd == ["gh", "completion"]
        ));

        // output.cmd も同様。
        let m = parse("[[steps]]\ninput.cmd = [\"defaults\", \"export\"]\n[[steps]]\noutput.cmd = [\"defaults\", \"import\"]\n").unwrap();
        assert!(matches!(
            &pipeline(&m).1[1],
            Step::Output(OutputStep {
                dest: OutputSource::Cmd(_),
                ..
            })
        ));
    }

    // ── Format / Merge / Os の Display ↔ serde round-trip ──

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
                pipeline(&parsed).0,
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
            let Step::Input(second) = &pipeline(&parsed).1[1] else {
                panic!("steps[1] は input のはず");
            };
            assert_eq!(
                second.merge,
                Some(merge),
                "Display と serde 表現がズレている: {merge}"
            );
        }
    }

    #[test]
    fn os_display_round_trips_through_serde() {
        for os in Os::iter() {
            let parsed = parse(&format!("when = {{ os = \"{os}\" }}\n{TREE}")).unwrap();
            assert_eq!(
                parsed.when.and_then(|when| when.os),
                Some(os),
                "Display と serde 表現がズレている: {os}"
            );
        }
    }

    #[test]
    fn rejects_unknown_os() {
        let err = parse(&format!("when = {{ os = \"macos\" }}\n{TREE}")).unwrap_err();
        assert!(
            err.contains("darwin") && err.contains("linux"),
            "受理値を示さずに弾いている: {err}"
        );
    }

    // ── shape ──

    #[test]
    fn accepts_minimal_tree() {
        // ツリーは専用の形（output ＋ 末尾 output.cmd）へ解釈される。末尾 output.cmd 無しなら post は空。
        let m = parse(TREE).unwrap();
        assert!(
            matches!(&m.steps, Steps::Tree { output, post } if output.to_string() == "~/x" && post.is_empty())
        );
    }

    #[test]
    fn interprets_pipeline_step_annotations() {
        // merge / when / optional が各 step の確定形（InputStep / OutputStep）へ載ることを固定する。
        let m = parse(
            "format = \"json\"\n\
             [[steps]]\ninput = \"~/.claude/settings.json\"\noptional = true\n\
             [[steps]]\ninput = \"settings.json\"\nmerge = \"shallow\"\nwhen = { deps = [\"rtk\"] }\n\
             [[steps]]\noutput = \"~/.claude/settings.json\"\n",
        )
        .unwrap();
        let (format, steps) = pipeline(&m);
        assert_eq!(format, Some(Format::Json));
        let Step::Input(first) = &steps[0] else {
            panic!("steps[0] は input のはず");
        };
        assert!(first.optional && first.merge.is_none() && first.when.is_none());
        let Step::Input(second) = &steps[1] else {
            panic!("steps[1] は input のはず");
        };
        assert_eq!(second.merge, Some(Merge::Shallow));
        assert!(second.when.is_some() && !second.optional);
        assert!(matches!(
            &steps[2],
            Step::Output(OutputStep { dest: OutputSource::Path(p), .. }) if p.to_string() == "~/.claude/settings.json"
        ));
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

    #[test]
    fn preserves_bare_tilde_vs_tilde_slash_in_display() {
        // 生表記の `~`（末尾スラッシュ無し）と `~/`（末尾スラッシュ有り）は解決先は同じでも
        // 表示は区別して復元する（apply の 1 行出力・`list` の宛先表記）。
        let m = parse("[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~\"\n").unwrap();
        assert_eq!(m.display_dst(), "~", "bare `~` が `~/` に化けている");
        let m = parse("[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/\"\n").unwrap();
        assert_eq!(m.display_dst(), "~/", "`~/` が `~` に化けている");
    }

    // ── pipeline shape ──

    #[test]
    fn rejects_pipeline_starting_with_output() {
        // 非ツリー（`.` を含まない）で先頭が output。畳む内容がまだ無いまま書き込みへ到達するため
        // load 時に弾く。
        let err = parse("[[steps]]\noutput = \"~/x\"\n[[steps]]\ninput = \"a\"\n").unwrap_err();
        assert!(
            err.contains("steps[0]") && err.contains("output"),
            "先頭 output のパイプラインが弾かれていない: {err}"
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
        // deep ↔ text も非両立。
        let err = parse(
            "format = \"text\"\n[[steps]]\ninput = \"a\"\n[[steps]]\ninput = \"b\"\nmerge = \"deep\"\n[[steps]]\noutput = \"~/x\"\n",
        )
        .unwrap_err();
        assert!(
            err.contains("両立しません"),
            "deep×text が弾かれていない: {err}"
        );
    }

    #[test]
    fn accepts_deep_merge_mixed_with_shallow_in_same_unit() {
        // #554: 同じ unit（format="json"）内で 2 つ目の input は shallow、3 つ目は deep が混在できる
        // （settings ＝ shallow リセット → rtk 断片を deep で重ねる、の実例と同じ形）。
        assert!(
            parse(
                "format = \"json\"\n\
                 [[steps]]\ninput = \"a\"\n\
                 [[steps]]\ninput = \"b\"\nmerge = \"shallow\"\n\
                 [[steps]]\ninput = \"c\"\nmerge = \"deep\"\n\
                 [[steps]]\noutput = \"~/x\"\n",
            )
            .is_ok()
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
    fn accepts_tree_with_trailing_output_cmd() {
        // ツリーは output の後に output.cmd を末尾に並べられる（配置後に無条件で走る副作用）。
        let m = parse(
            "[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/x\"\n\
             [[steps]]\noutput.cmd = [\"bat\", \"cache\", \"--build\"]\n\
             [[steps]]\noutput.cmd = [\"sh\", \"-c\", \"true\"]\n",
        )
        .unwrap();
        let Steps::Tree { post, .. } = &m.steps else {
            panic!("ツリーを期待したがツリーでない");
        };
        assert_eq!(post.len(), 2);
        assert_eq!(post[0].cmd, ["bat", "cache", "--build"]);
        assert_eq!(post[1].cmd, ["sh", "-c", "true"]);
        assert_eq!(m.summary(), "tree, output.cmd=2");
    }

    #[test]
    fn rejects_tree_with_extra_input_step() {
        // ツリーの 2 番目が output でない（余分な input）は弾く（末尾 output.cmd は output だけ）。
        let err = parse(
            "[[steps]]\ninput = \".\"\n[[steps]]\ninput = \"a\"\n[[steps]]\noutput = \"~/x\"\n",
        )
        .unwrap_err();
        assert!(
            err.contains("output である必要があります"),
            "余分な input を持つツリーが弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_tree_trailing_non_cmd_step() {
        // 末尾がパス output（cmd でない）は弾く（3 つ目以降は output.cmd のみ）。
        let err = parse(
            "[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/x\"\n[[steps]]\noutput = \"~/y\"\n",
        )
        .unwrap_err();
        assert!(
            err.contains("output.cmd である必要があります"),
            "末尾のパス output が弾かれていない: {err}"
        );
    }

    #[test]
    fn rejects_tree_trailing_output_cmd_with_when() {
        // 末尾 output.cmd に per-step の when は書けない（分岐はユニット gate）。
        let err = parse(
            "[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/x\"\n\
             [[steps]]\noutput.cmd = [\"bat\"]\nwhen = { deps = [\"bat\"] }\n",
        )
        .unwrap_err();
        assert!(
            err.contains("when / merge / optional"),
            "末尾 output.cmd の when が弾かれていない: {err}"
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
        // `hooks`（onchange フックの語彙・#659 で撤去）も未知フィールドとして弾かれる ― 配置後の
        // コマンドはツリー末尾の output.cmd で宣言する。
        for legacy in [
            "dst = \"~/x\"\n",
            "kind = \"generate\"\n",
            "strategy = \"concat\"\n",
            "preserve = true\n",
            "cmd = [\"x\"]\n",
            "sensitive = [\"a\"]\n",
            "[[overlay]]\nsrc = \"a\"\n",
            "hooks = [{ cmd = [\"bat\"] }]\n",
        ] {
            let src = format!("{legacy}{TREE}");
            assert!(
                src.parse::<Manifest>().is_err(),
                "旧語彙が弾かれていない: {legacy}"
            );
        }
    }
}
