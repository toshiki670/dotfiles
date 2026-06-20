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
| **D2 copy/generate/merge の3層** | 配置は実体の書き出し（copy）。symlink は採用しない（理由は §5） |
| **D3 階層分散 manifest** | 設定単位（ディレクトリ）ごとに `manifest.toml` を置き、配置仕様を**明示**する |
| **D4 二段ソース** | 本番は**バイナリ埋め込み**、dev/移行期は**作業ツリー直読み**（§8） |
| **D5 color 統合** | 旧 `crates/color` を `dotfiles color` に吸収。テーマ切替＋カラーサンプルの2責務（§10） |
| **D6 chezmoi 併用移行** | `home/`（chezmoi）と `configs/`（dotfiles）を併存させ、ツール単位で段階移行（§12） |

---

## 5. 配置方式：なぜ symlink でなく copy か

将来 `cargo install dotfiles` でバイナリを配布する構想がある。symlink 方式は **「`~/` が、特定の場所に存在し続けるリポジトリ作業ツリーを指し続ける」** ことを恒久的に要求するため、この配布モデルと噛み合わない：

- 設定の実体が作業ツリーに残り、それが消える／動くと `~/` の設定が全て dangling する。
- 「バイナリを入れれば使える」が成立せず、**バイナリ＋永続クローンの2点セット**が必須になる。

よって **配置は実体を書き出す copy 方式**を採用する。失うのは「編集即反映」だけで、これは `dotfiles apply` で取り戻す（chezmoi と同じワークフロー。痛点だったのは apply ではなく「場所が分からない／color が面倒」であり、そこは本設計で解消される）。

### 3層モデル

| 層 | 処理 | 対象（調査での実数） |
| --- | --- | --- |
| **copy** | 実体をそのまま書き出す | 大多数（fish 断片・nvim・bat・ghostty・zellij・mise・rules 等） |
| **generate** | コマンド実行で生成 | 補完5本（`gh/docker/bat/clip/merge-ready`）。git config は **git native の `[include]` に置換**すれば copy に降格可能 |
| **merge** | 既存ファイルとマージ | `~/.claude/settings.json` の**1件のみ** |

**集約は copy で自然に解ける。** `~/.config/fish/conf.d/` などの合流点は「ディレクトリに置けば全部読まれる」設計なので、複数ツールの断片をその dst へ copy で書き出すだけでよい（マージ機構は不要）。本当に合成／マージが要るのは **補完5本＋settings.json 1件**だけで、これらだけ generate/merge として隔離する。

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

# 配置種別（省略時 = "copy"）。"generate" / "merge" のときだけ明記。
kind = "copy"

# パーミッション（copy。省略時 false）。chezmoi の private_ / executable_ 相当。
# private = 所有者のみ（0600 起点）、executable = 実行ビット付与（0644→0755 / 0600→0700）。
# private = true
# executable = true

# generate のとき: 実行するコマンド（src の代わり）。
# cmd = ["gh", "completion", "fish"]

# merge のとき: ローカル設定を温存するキー。
# preserve = ["model", "effortLevel"]

# テーマ対応方式（該当ツールのみ）: "source" / "follower" / "self"
theme = "follower"

# 依存バイナリ（該当のみ）。無い場合は配置/生成をスキップ（gate）。
deps = ["gh"]

# onchange フック（該当のみ）。
hooks = ["macos-symlink"]

# OS 条件（該当のみ）。例: "darwin"
os = "darwin"

# マシンローカル値の注入（該当のみ。§9）。
# secrets = ["user.email", "user.name"]
```

### 6.3 確定したルール

1. **管轄の再帰委譲**：あるディレクトリに `manifest.toml` があれば、そのディレクトリ＋（サブ manifest の無い）配下を管轄する。サブディレクトリに別の `manifest.toml` があれば、そこから先はそちらに委譲する（ツリーを manifest で分割統治）。
2. **kind ごとの src の扱い**：
   - `copy` … src ＝ 同ディレクトリの実ファイル（`manifest.toml` 以外）
   - `generate` … src 不要、`cmd` を持つ（ディレクトリは manifest だけでも可。補完がこれ）
   - `merge` … src ＝ base ファイル ＋ `preserve` 等のマージ仕様
3. **粒度の指針**：ディレクトリ分割は強制ではなく、**属性（kind/dst/theme/hooks/os）が分岐するときに使う道具**。同じ属性で済む範囲は1つの manifest にまとめてよい。
4. **読み込み順**：fish の `conf.d` 等の順序は、既存のファイル名番号（`05` < `40` < `90`）で表す。番号がグローバルな順序の単一の真実。

---

## 7. 属性スキーマ一覧（調査で実在を確認したもの）

| 属性 | 意味 | 必須 | 調査での実例 |
| --- | --- | --- | --- |
| `dst` | 配置先 | ✅ | 全ツール |
| `kind` | 3層種別（既定 copy） | 任意 | 補完=generate / claude=merge |
| `cmd` | 生成コマンド（generate 時） | generate 必須 | `gh completion fish` 等 |
| `private` | 所有者のみ（0600 起点。chezmoi `private_`） | 任意 | secrets 系 |
| `executable` | 実行ビット付与（0644→0755 / 0600→0700。chezmoi `executable_`） | 任意 | スクリプト/フック |
| `preserve` | 温存キー（merge 時） | merge 任意 | claude=`model`,`effortLevel` |
| `theme` | light/dark 対応方式 | 任意 | ghostty=source / eza・delta=follower / bat・nvim・fzf=self |
| `secrets` | マシンローカル注入 | 任意 | git `user.email`/`user.name` |
| `deps` | 依存バイナリ（gate） | 任意 | 補完=`gh`等 / worker=自前bin |
| `hooks` | onchange 処理 | 任意 | bat cache / ghostty symlink / cargo install |
| `os` | OS 条件 | 任意 | ghostty=darwin |

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

### 10.3 未確定の難所（PoC で検証）

> ⚠️ 以下は確認前の論点であり、事実として確定したものではない。

- テーマの**起点は Ghostty**（端末背景＋ANSIパレット）。真に手動で `dark` 固定するには、端末背景そのものを固定する必要があり、**Ghostty の theme 行も固定側へ切り替える**必要があるか検証が要る。Ghostty を OS追従のままにすると、端末背景は OS のままで手動固定と矛盾する可能性がある。
- `$fish_terminal_color_theme` は read-only のため、override は**別の状態変数**（例 `$dotfiles_theme_override`）を新設し、各連携をそれ優先に書き換える方針。実装時に詳細を詰める。

---

## 11. `dotfiles` コマンド体系（想定）

| コマンド | 役割 |
| --- | --- |
| `dotfiles apply [--source DIR]` | configs を走査し copy/generate/merge を実行＋ hooks 起動 |
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
| デプロイ（dot_/private_/executable_ 変換） | copy 層（dst・パーミッションは `private`/`executable` 属性で表現。§7） |
| `create_`（初回のみ生成・以後は温存。mise の machine-specific config 1件） | copy 層に **create-only 属性**（dst 既存なら上書きしない）が必要。未スライス（§14・後続 issue で追跡） |
| symlink_（git hooks 13本） | copy 層 or hooks（git hooks は dispatch への参照。配置方式は実装時確定） |
| 補完の動的生成（output） | generate 層（`cmd` ＋ `deps` gate） |
| ファイル合成（git config includeTemplate） | git native `[include]` に置換（copy 層へ降格）。bash 部品も同様に整理 |
| settings.json マージ（modify_） | merge 層（`preserve`） |
| シークレット注入（env） | secrets 属性＋ git native include（§9） |
| run_ フック（cargo install/bat cache/ghostty/doctor） | hooks 属性。onchange は前回適用ソースのハッシュを状態に保存して比較 |
| `.chezmoiignore` | 「configs に置かない」＝管理対象外。明示除外が要れば manifest で表現 |
| `.chezmoiroot` | 不要（configs がソースルート） |
| OS 分岐（`if eq .chezmoi.os`） | `os` 属性 |
| `.chezmoi.config.sourceDir` 参照 | `--source`／作業ツリー検出／埋め込みで解決（§8） |

---

## 14. 未決事項・今後の検討

- [ ] manifest の `dst` 表記（`~` 展開、`$XDG_*` の扱い）の正規化ルール
- [x] copy のパーミッション表現（`private_`=0600 / `executable_`=0700 相当）の属性化 → `private` / `executable` 属性（S1 #455）
- [ ] create-only 属性（chezmoi `create_` 相当: dst 既存なら上書きしない）＋ mise の `config.toml` 移行。**どのスライスにも無い**ため S1 で mise を見送り。**S9（home/ に残り無しが完了条件）の前提＝S9 ブロッカー**。後続 issue で追跡
- [ ] git hooks（symlink_ 13本）の配置方式（copy か、配置後にリンク生成か）
- [ ] hooks の onchange 検知方式（ソースハッシュ vs mtime）の確定
- [ ] color 手動固定時の Ghostty 起点の扱い（§10.3）
- [ ] chezmoi 併用時のフォールバック実挙動（§12.3）
- [ ] `dotfiles` のサブコマンド名最終確定（`apply` は仮称）

---

## 付録：合意済みの設計決定（サマリ）

1. 第一級エンティティは**ツール**。ソースは中身の帰属で並べ、配置先は属性。
2. 配置は **copy/generate/merge の3層**。symlink は不採用（cargo install 配布と非互換）。
3. データ構造は **`configs/` 階層 ＋ 各設定単位の `manifest.toml`（placement 明示）**。kind 混在はディレクトリ分割で解決。
4. ソースは **埋め込み（本番）＋作業ツリー直読み（dev/移行期）** の二段構え。
5. color は **`dotfiles color`** に統合（切替＋サンプル）。テーマは横断的関心事として状態ファイル＋ manifest の `theme` 属性で駆動。
6. 移行は **chezmoi 併用**（`chezmoi apply` → `dotfiles apply`、ツール単位で段階移行、止めれば chezmoi へ復帰）。
