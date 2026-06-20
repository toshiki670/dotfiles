# dotfiles ネイティブ化 設計書（chezmoi 依存の解消）

> **ステータス**: ドラフト（合意待ち）
> **目的**: chezmoi への依存をなくし、`dotfiles` コマンド自身で設定の管理・配置・テーマ切替を完結させる。
> **本書のゴール**: 実装に先立つアーキテクチャ合意。実装手順書ではなく、決定事項とその根拠を記録する。

---

## 1. 背景と目的（Why）

| 動機 | 現状の課題 | 本設計での解 |
| --- | --- | --- |
| **依存を減らす** | 配置を chezmoi に依存している | `dotfiles` コマンドが配置を担い、chezmoi を不要にする |
| **コンフィグベース志向** | chezmoi はディレクトリベース。設定の物理配置先（OS が決める場所）がそのままソース構造になるため、「どこに何があるか」を OS の配置規則ごと覚えていないと探せない／整理できない | ソースを **「中身の帰属（ツール）」** で並べ、配置先を **属性に格下げ**する |
| **テーマ切替の簡易化** | `dotfiles color dark\|light\|auto` を1コマンドでやりたいが、chezmoi 経由だと複数ファイル編集＋ apply が要り面倒 | テーマを横断的関心事として `dotfiles color` に統合（旧 `crates/color` も吸収） |

---

## 2. 現状の chezmoi 責務（調査結果）

「dotfiles で完結」させるには、chezmoi が現在担っている責務をすべて代替する必要がある。調査で確定した責務は以下の6つ。

| # | 責務 | 具体 |
| --- | --- | --- |
| 1 | **デプロイ**（source→target） | `home/` → `~/`。`dot_`/`private_`/`executable_`/`symlink_`（git hooks 13本）の変換 |
| 2 | **補完の動的生成** | `output "gh completion fish"` 等でコマンドを実行し補完を生成（bat/gh/docker/clip/merge-ready） |
| 3 | **ファイル合成** | git config を `includeTemplate` で8部品から合成、bash を `include` で結合 |
| 4 | **settings.json マージ** | `modify_` スクリプトで既存のローカル設定を温存しつつ共有設定を上書き（jq） |
| 5 | **シークレット注入** | `env "DOTFILES_GIT_EMAIL/NAME"` でマシンローカルの git user |
| 6 | **フック** | cargo install / bat cache build / ghostty macOS symlink / brew・mise doctor。`sha256` で onchange 検知 |

---

## 3. 中核となる課題：「中身の帰属」と「配置先」の many-to-many

設定を **中身の帰属（＝ツール）** で分類するときれいに括れる。しかし **配置先（OS が決める物理位置）** で見ると、複数ツールが同じディレクトリを共有している。

| 配置先（OS物理位置） | そこに集まる「中身の帰属（ツール）」 |
| --- | --- |
| `~/.config/fish/conf.d/` | fish, **fzf, eza, delta, zoxide, starship, claude, browser-use, git-worker, 環境(PATH/EDITOR)** の断片が混在 |
| `~/.config/fish/functions/` | fish(cdabbr, ps-grep), **fzf**(`_fzf_*`), **gh**(gh-clone) |
| `~/.config/fish/completions/` | **bat, clip, docker, gh, merge-ready**（各バイナリから生成） |
| `~/.config/git/config` | git の8部品を合成（うち delta は色設定も持つ） |
| `~/.config/bat/` | bat 本体 + ayu-dark テーマ（**delta が共有**） |

➡ **「どこに何があるか分からない」の根本原因はこれ。** chezmoi は左列（配置先）でソースを並べるので、「eza の設定は？」の答えが「fish の中の `40-eza.fish`」になる。中身の帰属と物理位置が食い違うから探せない。

**本設計の核心は、ソースを右列（中身の帰属）で並べ直し、左列（配置先）を属性に格下げすること。** 代わりに、配置時に複数ツールの断片を合流点ディレクトリへ集約する処理が要る。

---

## 4. 設計方針（決定事項）

| 決定 | 内容 |
| --- | --- |
| **D1 ツール第一級** | ソースは `configs/<tool>/` に「中身の帰属」で並べる。配置先は属性 |
| **D2 配置は copy（symlink 不採用）** | 配置は実体の書き出し（copy）。symlink は採用しない（理由は §5） |
| **D3 階層分散 manifest** | 設定単位（ディレクトリ）ごとに `manifest.toml` を置き、配置仕様を**明示**する |
| **D4 二段ソース** | 本番は**バイナリ埋め込み**、dev/移行期は**作業ツリー直読み**（§8） |
| **D5 color 統合** | 旧 `crates/color` を `dotfiles color` に吸収。テーマ切替＋カラーサンプルの2責務（§10） |
| **D6 chezmoi 併用移行** | `home/`（chezmoi）と `configs/`（dotfiles）を併存させ、ツール単位で段階移行（§12） |
| **D7 配置は2軸（生成方式×合成）** | 配置を **生成方式**（1断片をどう実体化するか＝copy / generate）と **合成**（複数の条件付き断片を1 dst へどう重ねるか）の2軸で捉える。`merge` は独立 kind ではなく合成の JSON 戦略（§5.5） |
| **D8 条件付き overlay で分岐を統合** | dst を「base ＋ `when` で gate された断片（overlay）」の合成として組む。`when`（`dep` / `os` / `theme`）が rtk 有無・OS・テーマ分岐を**1機構**に畳み、chezmoi テンプレート（`lookPath` / `if eq .os`）を不要にする。color の build-time 切替もこの機構に乗る（§5.5・§10） |

---

## 5. 配置方式：なぜ symlink でなく copy か

将来 `cargo install dotfiles` でバイナリを配布する構想がある。symlink 方式は **「`~/` が、特定の場所に存在し続けるリポジトリ作業ツリーを指し続ける」** ことを恒久的に要求するため、この配布モデルと噛み合わない：

- 設定の実体が作業ツリーに残り、それが消える／動くと `~/` の設定が全て dangling する。
- 「バイナリを入れれば使える」が成立せず、**バイナリ＋永続クローンの2点セット**が必須になる。

よって **配置は実体を書き出す copy 方式**を採用する。失うのは「編集即反映」だけで、これは `dotfiles apply` で取り戻す（chezmoi と同じワークフロー。痛点だったのは apply ではなく「場所が分からない／color が面倒」であり、そこは本設計で解消される）。

### 配置の2軸：生成方式 × 合成

配置は独立した2つの軸で捉える（D7）。当初は copy/generate/merge を「3層」と並べていたが、これらは別の問いに答えるものなので軸を分ける：

| 軸 | 問い | 値 |
| --- | --- | --- |
| **生成方式（kind）** | 1つの断片を**どう実体化するか** | `copy`（ソース実ファイルをそのまま）/ `generate`（`cmd` 実行の stdout を採用） |
| **合成（strategy）** | 複数の条件付き断片を**どう1つの dst へ重ねるか** | 単一 = そのまま書く / `concat`（テキスト連結）/ `json-shallow`（JSON のトップレベル shallow merge） |

`merge` は「3つ目の kind」ではなく、**合成軸の JSON 戦略**である（既存ファイルの温存も「最後に重なる overlay」として表現する。§5.5）。生成方式と合成は直交する：例えば「`generate` した断片に、テーマ別の `copy` 断片を `concat` で重ねる」も表現できる。

| 生成方式＼合成 | 単一 | concat | json-shallow |
| --- | --- | --- | --- |
| **copy** | 大多数（nvim・bat・zellij 等のディレクトリ配置） | fish 合流点の断片群 | — |
| **generate** | — | 補完5本（生成物＋独自ブロック連結） | — |
| （混在） | — | テーマ別断片の重ね | `~/.claude/settings.json`（base＋rtk 断片＋既存温存） |

**集約は copy で自然に解ける。** `~/.config/fish/conf.d/` などの合流点は「ディレクトリに置けば全部読まれる」設計なので、複数ツールの断片をその dst へ copy で書き出すだけでよい。**本当に1ファイルへ合成が要るのは 補完5本＋settings.json＋（将来）テーマ別断片**だけで、これらが合成軸（concat / json-shallow ＋ overlay）の対象になる。

### 5.5 合成軸：条件付き overlay

1つの dst を「**順序付き overlay の合成**」として組む。各 overlay は **1断片（生成方式で実体化）＋ `when` gate（採用条件）** からなり、`when` を満たす overlay だけが合成戦略に従って重なる。

```text
最終 dst = strategy( overlay_1, overlay_2, … )      # when を満たすものだけ、宣言順に
overlay  = { src | cmd | preserve } + when?          # when 省略 = 常時採用
```

**`when` が条件分岐を1機構へ統合する**（chezmoi テンプレートの代替）：

| `when` キー | 意味 | 旧 chezmoi | 例 |
| --- | --- | --- | --- |
| `dep = "rtk"` | 依存バイナリが PATH にある時だけ採用 | `{{ if lookPath "rtk" }}` | settings.json の rtk hook 断片 |
| `os = "darwin"` | OS 一致時だけ採用 | `{{ if eq .chezmoi.os "darwin" }}` | macOS 限定断片 |
| `theme = "dark"` | 現在のテーマ状態が一致する時だけ採用 | （chezmoi に無い） | テーマ別 color 断片（§10） |

既存の **`deps` / `os`（ユニット単位の gate, §7）は「ユニット全体に係る `when`」の退化形**として包含する。`merge` の **`preserve`（ローカル温存キー）は「既存 dst を読む built-in overlay」**として表現する（base を土台に、preserve キーだけ既存値で上書きする最後の overlay）。

これにより、いまバラバラだった3つの分岐ニーズ ―依存 gate（`deps`）・OS 分岐（`os`）・テーマ追従（color）― が **`when` という1つの語彙**に畳まれる。

#### 例：settings.json（base ＋ rtk 断片 ＋ 既存温存）

```toml
dst      = "~/.claude/settings.json"
strategy = "json-shallow"            # 合成戦略

[[overlay]]
src = "settings.json"                # base（常時）

[[overlay]]
src  = "rtk.json"                    # rtk hook 断片だけ切り出す
when = { dep = "rtk" }               # rtk が PATH にある時だけ重ねる

[[overlay]]
preserve = ["model", "effortLevel"]  # 既存 dst から温存（最後に重なる overlay）
```

rtk の有無で hook の有無が決まり、テンプレートも「rtk 不在で毎回 command-not-found」も無い。

#### 留保（実装スライスで詰める）

> ⚠️ 確認前の論点。

- 合成戦略は**内容型依存**（JSON=shallow-merge / テキスト=concat）。型ごとの戦略を manifest でどう宣言するか（`strategy` 明示 か dst 拡張子からの推測か）は実装時に確定する。
- `json-shallow` はトップレベル単位の差し替え（deep merge はしない）。これは旧 `modify_` の `$local + $forced` と同じ粒度。深いマージが要る設定が現れたら別戦略を足す。
- overlay の `when` 評価順と、ユニット単位 gate（`deps`/`os`）との優先関係を実装時に固める。

---

## 6. データ構造：`configs/` 階層 ＋ `manifest.toml`

### 6.1 ディレクトリ構造（具体例）

```text
configs/
  eza/
    manifest.toml          # dst=~/.config/fish/conf.d, theme="follower"
    40-eza.fish
  gh/
    config/
      manifest.toml        # dst=~/.config/gh                （kind=copy）
      config.yml
    completion/
      manifest.toml        # kind=generate, cmd=["gh","completion","fish"],
                           #   dst=~/.config/fish/completions/gh.fish, deps=["gh"]
  claude/
    settings/
      manifest.toml        # kind=merge, dst=~/.claude/settings.json,
                           #   preserve=["model","effortLevel"]
      settings.base.json
  ghostty/
    manifest.toml          # dst=~/.config/ghostty, theme="source", os="darwin",
                           #   hooks=[macos-symlink]
    config
```

ポイント：

- **設定単位ごとに `manifest.toml` を置く**。kind が分岐するもの（gh の `config`=copy と `completion`=generate）は**ディレクトリを分けて別 manifest** にすることで自然に表現できる。
- **placement は明示**（`dst` を毎回書く）。これは冗長に見えるが、**「configs/<tool> の manifest を見れば配置先が一目で分かる」**ため、探しやすさはむしろ向上する。

### 6.2 manifest.toml スキーマ

```toml
# 配置先（必須）。copy/merge は実体の置き先、generate は生成物の置き先。
dst = "~/.config/fish/conf.d"

# 生成方式（省略時 = "copy"）。"generate" のときだけ明記（§5.5 の生成軸）。
kind = "copy"

# 合成戦略（複数 overlay を1 dst へ重ねるとき）。単一 overlay なら省略。
# "concat"（テキスト連結）/ "json-shallow"（JSON トップレベル shallow merge）。
# strategy = "json-shallow"

# パーミッション（copy。省略時 false）。chezmoi の private_ / executable_ 相当。
# private = 所有者のみ（0600 起点）、executable = 実行ビット付与（0644→0755 / 0600→0700）。
# private = true
# executable = true

# generate のとき: 実行するコマンド（src の代わり）。
# cmd = ["gh", "completion", "fish"]

# テーマ対応方式（該当ツールのみ）: "source" / "follower" / "self"
theme = "follower"

# 依存バイナリ（ユニット単位 gate。§7）。無い場合はユニット全体の配置/生成をスキップ。
# = ユニット全体に係る when（dep）の退化形（§5.5）。
deps = ["gh"]

# onchange フック（該当のみ）。
hooks = ["macos-symlink"]

# OS 条件（該当のみ）。例: "darwin"
os = "darwin"

# マシンローカル値の注入（該当のみ。§9）。
# secrets = ["user.email", "user.name"]

# --- 合成 overlay（複数の条件付き断片を1 dst へ重ねるとき。§5.5）---
# overlay を1つも書かなければ「ユニット直下の実ファイル群を単一 overlay として配置」と等価。
# [[overlay]]
# src = "settings.json"            # この overlay の断片（copy）。または cmd=[…]（generate）
#
# [[overlay]]
# src  = "rtk.json"
# when = { dep = "rtk" }           # when を満たす時だけ採用。dep / os / theme
#
# [[overlay]]
# preserve = ["model", "effortLevel"]   # 既存 dst を読む built-in overlay（merge の温存）
```

### 6.3 確定したルール

1. **管轄の再帰委譲**：あるディレクトリに `manifest.toml` があれば、そのディレクトリ＋（サブ manifest の無い）配下を管轄する。サブディレクトリに別の `manifest.toml` があれば、そこから先はそちらに委譲する（ツリーを manifest で分割統治）。
2. **生成方式ごとの src の扱い**：
   - `copy` … src ＝ 同ディレクトリの実ファイル（`manifest.toml` 以外）
   - `generate` … src 不要、`cmd` を持つ（ディレクトリは manifest だけでも可。補完がこれ）
   - overlay を明示するとき … 各 overlay が `src`（copy）/ `cmd`（generate）/ `preserve`（既存 dst）のいずれかを持つ。`merge` という独立 kind は廃し、**`strategy = "json-shallow"` ＋ overlay** で表現する（§5.5）。
3. **粒度の指針**：ディレクトリ分割は強制ではなく、**属性（kind/dst/theme/hooks/os）が分岐するときに使う道具**。同じ dst でも「条件で内容が変わる」だけなら、ディレクトリを割らず `when` 付き overlay で表現する（rtk・テーマ別など）。同じ属性で済む範囲は1つの manifest にまとめてよい。
4. **読み込み順**：fish の `conf.d` 等の順序は、既存のファイル名番号（`05` < `40` < `90`）で表す。番号がグローバルな順序の単一の真実。

---

## 7. 属性スキーマ一覧（調査で実在を確認したもの）

| 属性 | 意味 | 必須 | 調査での実例 |
| --- | --- | --- | --- |
| `dst` | 配置先 | ✅ | 全ツール |
| `kind` | 生成方式（既定 copy）。copy / generate | 任意 | 補完=generate |
| `strategy` | 合成戦略（複数 overlay 時）。concat / json-shallow | 任意 | claude=json-shallow / fish 合流点=concat |
| `cmd` | 生成コマンド（generate 時） | generate 必須 | `gh completion fish` 等 |
| `private` | 所有者のみ（0600 起点。chezmoi `private_`） | 任意 | secrets 系 |
| `executable` | 実行ビット付与（0644→0755 / 0600→0700。chezmoi `executable_`） | 任意 | スクリプト/フック |
| `overlay` | 条件付き断片の配列（§5.5）。各 overlay = `src`/`cmd`/`preserve` ＋ `when?` | 任意 | claude=base+rtk+preserve |
| `when` | overlay の採用条件。`dep` / `os` / `theme` | 任意 | rtk hook=`dep=rtk` |
| `preserve` | 既存 dst から温存するキー（built-in overlay の糖衣） | 任意 | claude=`model`,`effortLevel` |
| `theme` | light/dark 対応方式（runtime 追従の宣言。§10） | 任意 | ghostty=source / eza・delta=follower / bat・nvim・fzf=self |
| `secrets` | マシンローカル注入 | 任意 | git `user.email`/`user.name` |
| `deps` | 依存バイナリ（ユニット単位 gate ＝ ユニット全体に係る `when.dep` の退化形） | 任意 | 補完=`gh`等 / worker=自前bin |
| `hooks` | onchange 処理 | 任意 | bat cache / ghostty symlink / cargo install |
| `os` | OS 条件（ユニット単位 gate ＝ `when.os` の退化形） | 任意 | ghostty=darwin |

---

## 8. ソースの二段構え（埋め込み ＋ 作業ツリー直読み）

`cargo install dotfiles` で自己完結させるには、配置元（`configs/` の実体）をバイナリがどこから得るかが鍵。

| 文脈 | ソース解決元 | 再ビルド |
| --- | --- | --- |
| **dev / 移行期**（設定を編集する） | 作業ツリーの `configs/`（`--source ./configs` 明示、または自動検出） | 不要（即 apply で検証） |
| **本番 / 配布**（`cargo install` した環境） | バイナリ埋め込み（`include_dir!`） | — |

**解決優先順位**：`--source` 明示 ＞ 作業ツリー検出 ＞ 埋め込みフォールバック。

- 埋め込み = コンパイル時に `configs/` の中身をバイナリのデータ領域へ焼き込む（`include_dir` クレート）。設定ファイルは数十〜数百KB程度なのでサイズ増分は実用上無視できる。
- 埋め込みの弱点「変更のたび再ビルド」は、dev/移行期に作業ツリー直読みを使うことで回避する。**いま進める移行期は作業ツリー直読みが主役**で、埋め込みは将来の配布で効く完成形。

---

## 9. シークレット（マシンローカル値）

現状の `env "DOTFILES_GIT_EMAIL/NAME"` 注入は廃止し、**git native の include でローカル非管理ファイルから注入**する：

- `configs/git` の base には共有 config のみを置き、`user.email`/`user.name` は含めない。
- 配置した `~/.config/git/config` に `[include] path = ~/.config/git/local`（仮）を持たせ、`local` はユーザーが手で書く（dotfiles 管理外・gitignore 相当）。
- manifest 上は `secrets = ["user.email", "user.name"]` で「この配置はマシンローカル値を要する」ことを宣言（`dotfiles doctor` 等で未設定を警告できる）。

---

## 10. color（テーマ）の設計

### 10.1 現状のテーマ追従構造（調査結果）

- **起点**: Ghostty `theme = light:One Half Light,dark:Ayu`（OS外観 → 端末背景＋ANSIパレット）
- **追従①（fish 変数経由）**: fish が OSC11 で背景検出 → `$fish_terminal_color_theme` → eza/delta/fish が `--on-variable` で反応
- **追従②（各ツール自前）**: bat（`--theme=auto`）、nvim（起動時 OSC11）、fzf（ANSI 番号）
- **現状は 100% OS追従。手動固定の手段は無い。**

### 10.2 設計

- コマンド体系：
  - `dotfiles color dark` / `dotfiles color light` … テーマを手動固定
  - `dotfiles color auto` … OS外観追従（現状の挙動）に戻す
  - `dotfiles color sample` … ANSIカラー確認表（旧 `crates/color` の責務）
- **状態ファイル**（例 `$XDG_STATE_HOME/dotfiles/theme` または `~/.config/dotfiles/theme`）に `dark` / `light` / `auto` を書く。
- 各ツール連携は **「状態ファイルの override があればそれを優先、無ければ従来の OS追従」** を参照する。各ツールの追従方式は manifest の `theme` 属性（source/follower/self）で宣言され、`dotfiles color` はこの属性を横断して切替を駆動する。

### 10.2.1 overlay 機構との接続（§5.5）

color には **2系統**ある。両者は排他ではなく、ツールごとに使い分ける：

| 系統 | 仕組み | 対象 | 駆動 |
| --- | --- | --- | --- |
| **runtime 追従**（現状・既定） | 1ファイルを置き、ツールが実行時にテーマを検出して反応 | fish 変数追従（eza/delta/fish）・bat/nvim/fzf の self | 状態ファイル＋ツール側ロジック |
| **build-time 切替**（overlay） | `when.theme` で gate したテーマ別断片を合成し、`dotfiles color` が**ファイルを書き直す** | 実行時に追従**できない**ツール（設定値を起動時に焼くもの等） | `dotfiles color` が theme 状態を変えて apply 相当を再実行 |

➡ **overlay は color を build-time で切り替える「もう一つの道」を開く。** `when.theme` は rtk(`dep`)・OS(`os`) と同じ gate 語彙の一員で、テーマ別断片もこの1機構に乗る。ただし**現状の runtime 追従（§10.1）を全面置換するものではない**ので、どのツールを runtime のままにし、どれを build-time overlay へ寄せるかは color スライスで個別に決める（過度な build-time 化は、既に動く追従を壊しうる）。

### 10.3 未確定の難所（PoC で検証）

> ⚠️ 以下は確認前の論点であり、事実として確定したものではない。

- テーマの**起点は Ghostty**（端末背景＋ANSIパレット）。真に手動で `dark` 固定するには、端末背景そのものを固定する必要があり、**Ghostty の theme 行も固定側へ切り替える**必要があるか検証が要る。Ghostty を OS追従のままにすると、端末背景は OS のままで手動固定と矛盾する可能性がある。
- `$fish_terminal_color_theme` は read-only のため、override は**別の状態変数**（例 `$dotfiles_theme_override`）を新設し、各連携をそれ優先に書き換える方針。実装時に詳細を詰める。

---

## 11. `dotfiles` コマンド体系（想定）

| コマンド | 役割 |
| --- | --- |
| `dotfiles apply [--source DIR]` | configs を走査し生成（copy/generate）×合成（overlay）を実行＋ hooks 起動 |
| `dotfiles list` / `dotfiles status` | 管理ツール一覧・配置状況の俯瞰（分散 manifest を集約表示） |
| `dotfiles color <dark\|light\|auto>` | テーマ手動固定／追従 |
| `dotfiles color sample` | ANSIカラー確認表（旧 color クレート吸収） |
| `dotfiles doctor` | 依存バイナリ・secrets 未設定などの診断（chezmoi の doctor-check 相当） |
| `dotfiles --version` | 既存（バージョンの source of truth） |

`dotfiles list` が分散 manifest を集約して俯瞰を提供することで、分散方式の弱点「全体一覧が横断的」を補う。

---

## 12. 移行戦略（chezmoi 併用）

### 12.1 並行運用モデル

- ソースは `home/`（chezmoi）と `configs/`（dotfiles）を**併存**させる。
- 適用は **`chezmoi apply`（全部配置）→ `dotfiles apply`（移行済みツールだけ上書き）** の二段。
- 既存の `home/` に手を入れずに dotfiles を PoC 検証できる。問題が起きたら **`dotfiles apply` をやめて `chezmoi apply` だけ**で元の状態に戻せる（後勝ち上書き＋安全なフォールバック）。

### 12.2 段階移行の手順（ツール単位）

1. あるツール（例 `eza`）の設定を `home/` から `configs/eza/` へ移し、`manifest.toml` を付ける。
2. `dotfiles apply` で配置を検証（`chezmoi apply` の後勝ちで上書きされることを確認）。
3. 問題なければ `home/` 側の該当ファイルを削除して二重管理を解消（＝**移行完了の定義**）。
4. 全ツール移行が終われば chezmoi 関連（`.chezmoiscripts/`・`.chezmoi*`・`home/`）を撤去。

### 12.3 留保（PoC で要検証）

> ⚠️ 確認前の論点。

- 「chezmoi の管理対象のまま dotfiles が上書きし、`dotfiles` を止めれば `chezmoi apply` で復帰する」のは**想定**であり、chezmoi のステート追跡（`chezmoistate`）が絡むため実挙動を PoC で確認する。管理対象である限り成立する見込み。

---

## 13. chezmoi 責務 → dotfiles 代替 対応表（網羅性チェック）

| chezmoi 責務 | dotfiles での代替 |
| --- | --- |
| デプロイ（dot_/private_/executable_ 変換） | copy（生成方式）。dst・パーミッションは `private`/`executable` 属性で表現（§7） |
| `create_`（初回のみ生成・以後は温存。mise の machine-specific config 1件） | copy に **create-only 属性**（dst 既存なら上書きしない）が必要。未スライス（§14・後続 issue で追跡） |
| symlink_（git hooks 13本） | copy or hooks（git hooks は dispatch への参照。配置方式は実装時確定） |
| 補完の動的生成（output） | generate（`cmd` ＋ `deps` gate） |
| ファイル合成（git config includeTemplate） | git native `[include]` に置換（copy へ降格）。bash 部品も同様に整理 |
| settings.json マージ（modify_） | 合成軸：`strategy="json-shallow"` ＋ overlay（base＋`preserve` の既存温存 overlay。§5.5） |
| **テンプレート条件**（`{{ if lookPath … }}` / 部分的な if） | **overlay の `when`**（`dep` / `os` / `theme`）。テンプレートエンジンは持たず宣言的 gate で表現（§5.5） |
| シークレット注入（env） | secrets 属性＋ git native include（§9） |
| run_ フック（cargo install/bat cache/ghostty/doctor） | hooks 属性。onchange は前回適用ソースのハッシュを状態に保存して比較 |
| `.chezmoiignore` | 「configs に置かない」＝管理対象外。明示除外が要れば manifest で表現 |
| `.chezmoiroot` | 不要（configs がソースルート） |
| OS 分岐（`if eq .chezmoi.os`） | `os` 属性（＝ ユニット単位の `when.os`。§5.5） |
| `.chezmoi.config.sourceDir` 参照 | `--source`／作業ツリー検出／埋め込みで解決（§8） |

---

## 14. 未決事項・今後の検討

- [ ] manifest の `dst` 表記（`~` 展開、`$XDG_*` の扱い）の正規化ルール
- [x] copy のパーミッション表現（`private_`=0600 / `executable_`=0700 相当）の属性化 → `private` / `executable` 属性（S1 #455）
- [x] テンプレート条件（`lookPath` 等の部分的 if）の置き場 → **合成軸の overlay ＋ `when` gate**（§5.5・D8）。テンプレートエンジンは持たない。設計確定、実装は後続（リファクタリングで `merge` を `strategy`＋overlay へ移行し、rtk hook を `when.dep` 化）
- [ ] 合成戦略の宣言方法（`strategy` 明示 vs dst 拡張子からの推測）と、`when` の評価順・ユニット gate との優先関係（§5.5 留保）
- [ ] `when.theme` を build-time color に使う範囲（どのツールを runtime 追従のまま残すか。§10.2.1）
- [ ] create-only 属性（chezmoi `create_` 相当: dst 既存なら上書きしない）＋ mise の `config.toml` 移行。**どのスライスにも無い**ため S1 で mise を見送り。**S9（home/ に残り無しが完了条件）の前提＝S9 ブロッカー**。後続 issue で追跡
- [ ] git hooks（symlink_ 13本）の配置方式（copy か、配置後にリンク生成か）
- [ ] hooks の onchange 検知方式（ソースハッシュ vs mtime）の確定
- [ ] color 手動固定時の Ghostty 起点の扱い（§10.3）
- [ ] chezmoi 併用時のフォールバック実挙動（§12.3）
- [ ] `dotfiles` のサブコマンド名最終確定（`apply` は仮称）

---

## 付録：合意済みの設計決定（サマリ）

1. 第一級エンティティは**ツール**。ソースは中身の帰属で並べ、配置先は属性。
2. 配置は **2軸 ＝ 生成方式（copy / generate）× 合成（単一 / concat / json-shallow）**。`merge` は独立 kind ではなく合成の JSON 戦略。symlink は不採用（cargo install 配布と非互換）。
3. dst は **条件付き overlay の合成**として組む。各 overlay は断片＋ `when`（`dep` / `os` / `theme`）gate を持ち、rtk・OS・テーマ分岐を1機構へ統合（chezmoi テンプレート不要）。`deps`/`os` はユニット単位 gate、`preserve` は既存温存 overlay の糖衣。
4. データ構造は **`configs/` 階層 ＋ 各設定単位の `manifest.toml`（placement 明示）**。属性が分岐するときはディレクトリ分割、内容だけが条件で変わるなら `when` 付き overlay。
5. ソースは **埋め込み（本番）＋作業ツリー直読み（dev/移行期）** の二段構え。
6. color は **`dotfiles color`** に統合（切替＋サンプル）。runtime 追従（既定）と build-time overlay（`when.theme`）の2系統をツールごとに使い分ける。
7. 移行は **chezmoi 併用**（`chezmoi apply` → `dotfiles apply`、ツール単位で段階移行、止めれば chezmoi へ復帰）。
