# dotfiles ネイティブ化 設計書（chezmoi 依存の解消）

> **ステータス**: 実装完了（chezmoi 撤去済み・#463）
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

## 2. 移行前の chezmoi 責務（調査結果）

「dotfiles で完結」させるには、chezmoi が担っていた責務をすべて代替する必要があった。調査で確定した責務は以下の6つ。

| # | 責務 | 具体 |
| --- | --- | --- |
| 1 | **デプロイ**（source→target） | `home/` → `~/`。`dot_`/`private_`/`executable_`/`symlink_`（git hooks 13本）の変換 |
| 2 | **補完の動的生成** | `output "gh completion fish"` 等でコマンドを実行し補完を生成（bat/gh/docker/clip/merge-ready） |
| 3 | **ファイル合成** | git config を `includeTemplate` で8部品から合成、bash を `include` で結合 |
| 4 | **settings.json マージ** | `modify_` スクリプトで既存のローカル設定を温存しつつ共有設定を上書き（jq） |
| 5 | **シークレット注入** | `env "DOTFILES_GIT_EMAIL/NAME"` でマシンローカルの git user |
| 6 | **フック** | cargo install / bat cache build / ghostty macOS symlink / brew・mise doctor。ソース指紋で onchange 検知 |

---

## 3. 中核となる課題：「中身の帰属」と「配置先」の many-to-many

設定を **中身の帰属（＝ツール）** で分類するときれいに括れる。しかし **配置先（OS が決める物理位置）** で見ると、複数ツールが同じディレクトリを共有している。

| 配置先（OS物理位置） | そこに集まる「中身の帰属（ツール）」 |
| --- | --- |
| `~/.config/fish/conf.d/` | fish, **fzf, eza, delta, zoxide, starship, claude, browser-use, git-worker, 環境(PATH/EDITOR)** の断片が混在 |
| `~/.config/fish/functions/` | fish(`_fzf_file`/`_fzf_history`), **fzf-picker**(`_fzf_gh`/`_fzf_ghq_cd`/`_fzf_worktree_remove`/`cdabbr`/preview 2本), **gh-clone**(gh-clone), **ps-grep**（単独帰属） |
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
| **D8 条件付き overlay で分岐を統合** | dst を「base ＋ `when` で gate された断片（overlay）」の合成として組む。`when`（`deps` / `os` / `theme`）が rtk 有無・OS・テーマ分岐を**1機構**に畳み、chezmoi テンプレート（`lookPath` / `if eq .os`）を不要にする。gate 語彙は `when` 一本で、スコープは書く位置で表す（トップレベル＝ユニット全体 / overlay 内＝断片）。color の build-time 切替もこの機構に乗る（§5.5・§10） |
| **D9 エンジン/テストはツールのライフサイクルから独立** | `dotfiles` バイナリは汎用エンジン、`configs/` の個々のツール（claude / rtk / bat …）は**いつか消える一時的なデータ**。**今のツールが全て入れ替わってもバイナリとテストは無変更で生存する**。判定基準＝「configs から特定ツールを削除/改名したらテストが壊れるか? 壊れるなら defect」。契約テストは hermetic な架空 fixture、実 configs の検証は data-driven 走査で表す（§15） |

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
| **合成（strategy）** | 複数の条件付き断片を**どう1つの dst へ重ねるか** | 単一 = そのまま書く / `concat`（テキスト連結）/ `json-shallow`（JSON のトップレベル shallow merge）/ `plist-shallow`（plist のトップレベル shallow merge） |

`merge` は「3つ目の kind」ではなく、**合成軸の戦略**（内容型ごとに JSON 版・plist 版がある）である（既存ファイルの温存も合成戦略で表現する＝`json-shallow` ＋ `preserve = true` で既存 dst を最下層の土台に重ねる。§5.5）。生成方式と合成は直交する：例えば「`generate` した断片に、テーマ別の `copy` 断片を `concat` で重ねる」も表現できる。

| 生成方式＼合成 | 単一（合成なし） | concat | json-shallow | plist-shallow |
| --- | --- | --- | --- | --- |
| **copy** | 大多数。**dst=ディレクトリへツリー配置**（nvim・bat・zellij、fish conf.d の断片群も各ファイルとして個別配置） | （将来）テーマ別断片を1ファイルへ連結 | — | — |
| **generate** | — | 補完5本（生成物＋独自ブロック連結。dst=ファイル） | — | — |
| **compose** | — | — | `~/.claude/settings.json`（既存温存＝土台＋base＋rtk 断片。dst=ファイル） | `~/.cache/dotfiles/eu.exelban.Stats.plist`（生きたドメインの export を土台＋dotfiles 管理サブセット。dst=ファイル。#531） |

**集約は copy で自然に解ける。** `~/.config/fish/conf.d/` などの合流点は「ディレクトリに置けば全部読まれる」設計なので、複数ツールの断片をその dst（ディレクトリ）へ各ファイルとして copy で書き出すだけでよい（**連結ではなくツリー配置**）。**本当に1ファイルへ合成が要るのは 補完5本＋settings.json＋（将来）テーマ別断片**だけで、これらが合成軸（concat / json-shallow ＋ overlay）の対象になる。

> **dst の種別が合成の要否を決める**（→ §5.5 不変条件）。**dst=ディレクトリ**なら配下を**ツリーのまま個別配置**（複数ファイルは普通に可・合成なし・overlay 不要）。**dst=ファイル**のときだけ、複数入力を1つに束ねる合成戦略（generate の sibling 連結や明示 overlay）が要る。

### 5.5 合成軸：条件付き overlay

1つの dst を「**順序付き overlay の合成**」として組む。各 overlay は **1断片（生成方式で実体化）＋ `when` gate（採用条件）** からなり、`when` を満たす overlay だけが合成戦略に従って重なる。

```text
最終 dst = strategy( [既存 dst], overlay_1, overlay_2, … )   # 既存 dst は preserve=true の時だけ最下層に。overlay は when を満たすものだけ宣言順に
overlay  = { src | cmd } + when?                            # when 省略 = 常時採用
preserve = true                                            # 既存 dst を最下層の土台として温存（dotfiles 非管理キーを保持）
```

**`when` が条件分岐を1機構へ統合する**（chezmoi テンプレートの代替）：

| `when` キー | 型 | 意味 | 旧 chezmoi | 例 |
| --- | --- | --- | --- | --- |
| `deps = ["rtk"]` | 配列・AND | 列挙バイナリが**全て** PATH にある時だけ採用 | `{{ if lookPath "rtk" }}` | settings.json の rtk hook 断片 |
| `os = "darwin"` | スカラ | OS 一致時だけ採用 | `{{ if eq .chezmoi.os "darwin" }}` | macOS 限定断片 |
| `profile = "private"` | スカラ | 現在の profile 状態（`dotfiles profile` で選ぶマシンクラス）が一致する時だけ採用 | （chezmoi に無い・`create_` 等で代替していた） | yt の private 限定 drop-in（§9.2） |
| `theme = "dark"` | スカラ | 現在のテーマ状態が一致する時だけ採用 | （chezmoi に無い） | テーマ別 color 断片（§10） |

> ⚠️ **`theme` は未配線**（color スライスまで）。`When` は `deny_unknown_fields` のため、color スライスが `theme` フィールドを足すまで `when = { theme = … }` は **load 時エラー**になる。現役の gate キーは `deps` / `os` / `profile` の 3 つ。`deps`/`os` が環境から都度判る ambient な条件なのに対し、`profile` は user が選んでおく**状態**を読む点で `theme` と同族（§10・状態駆動 gate）。`profile` がその機構（状態ファイル・評価器の状態 snapshot）の最初の配線で、`theme` は後で相乗りする。

**gate 語彙は `when` 一本**で、**スコープは「書く位置」で表す**（別名は持たない）。トップレベルに書いた `when` は**ユニット全体 gate**（満たさなければユニットごと skip ＝ all-or-nothing）、`[[overlay]]` 内の `when` は**その断片だけの採否**。両スコープは同じ評価規則（`deps` 配列の AND ・`os` スカラ一致・複数キー AND）を共有する。`merge` の **ローカル温存はユニット属性 `preserve = true`** で表す。これは「**既存 dst を `json-shallow` の最下層（土台）として読み込み、dotfiles 断片を上に重ねる**」指示で、**dotfiles が所有するキー（断片が定義するキー）だけを上書きし、それ以外の既存ローカルキーは全て保持する**（旧 chezmoi `modify_` の `jq '$local + $forced'` と同値）。列挙した特定キーだけ残す allowlist ではない点に注意（未列挙のローカル固有キーが落ちる事故を避ける）。

**保持・上書きはともにトップレベルキー単位**（`json-shallow` は deep merge をしない）。dotfiles が所有するトップレベルキー（例 `hooks`）は**オブジェクトごと置き換わる**ので、その配下のローカル独自項目（例: 既存 `hooks` 内に手で足したフック）は**保持されず** base/rtk の値で上書きされる。保持されるのは **dotfiles がトップレベルで一切定義しないキー**（例 `model` / `effortLevel` や任意のローカル固有キー）だけ。深い差分まで残したい設定が出てきたら別戦略（deep merge）を足す（§5.5 留保）。

これにより、いまバラバラだった分岐ニーズ ―依存 gate（`when.deps`）・OS 分岐（`when.os`）・マシンクラス分岐（`when.profile`）・テーマ追従（color）― が **`when` という1つの語彙**に畳まれる。

#### 例：settings.json（既存温存 ＋ base ＋ rtk 断片）

```toml
dst      = "~/.claude/settings.json"
strategy = "json-shallow"            # 合成戦略
preserve = true                      # 既存 dst を土台に温存（dotfiles 非管理キーを保持）

[[overlay]]
src = "settings.json"                # base（常時。dotfiles 所有の共有キー）

[[overlay]]
src  = "rtk.json"                    # rtk hook 断片だけ切り出す
when = { deps = ["rtk"] }            # rtk が PATH にある時だけ重ねる
```

合成は `既存 dst → base → rtk` の順（後勝ち）。`model` / `effortLevel` など **dotfiles が定義しないローカルキーは土台のまま残り**、`language` / `hooks` など **dotfiles が所有する共有キーだけが上書きされる**。rtk の有無で hook の有無が決まり、テンプレートも「rtk 不在で毎回 command-not-found」も無い。

#### 例：Stats.plist（生きたドメインの export を土台 ＋ dotfiles 管理サブセット）／`plist-shallow`（#531）

macOS ネイティブアプリの preference plist（`~/Library/Preferences/`、cfprefsd 管轄）を dotfiles で扱う例。settings.json（`preserve = true` で既存 dst を土台にする）と違い、**土台は dst ファイルではなく生きたドメイン**なので、`preserve` ではなく base overlay の `cmd` で土台を取得する。

**plist と XML の関係**: plist は Apple のデータモデル（`dict`/`array`/`string`/`integer`/`real`/`boolean`/`data`/`date`/`uid`）で、XML・binary・ASCII のいずれでも直列化できる（`defaults export <domain> <file>` は binary、`-`（標準出力）指定時は XML で書く）。XML 自体は plist 専用ではない汎用構文（RSS・SVG など任意の語彙を表現できる）なので、`json-shallow` と対にした戦略名は `xml-shallow` ではなく **`plist-shallow`**（データモデル名）にした ― shallow merge が要る「トップレベルの key-value 構造」を保証するのは plist の dict モデルであって、XML という構文そのものではない。実装（`plist::Value::from_reader`）は XML/binary/ASCII のどれでも入力として自動判別し、出力は差分可読性のため XML に固定する。

dst の置き場所にも注意が要る：この dst はライブドメインが真実のソースであるための**使い捨ての下書き**（次の apply で再計算され、消えても実害が無い）で、`~/.claude/settings.json` のように別ツールが直接読む「実体」ではない。よって永続状態（named value・hooks 実行済みハッシュ等）を置く `~/.config/dotfiles/` とは分け、使い捨てデータの置き場（他 config も使う `~/.cache/dotfiles/`。例: `configs/claude/statusline-command.py` の `~/.cache/claude/`）に置く。cfprefsd 管轄外・XML＝差分可読という #465 の決定はそのまま引き継ぐ。

```toml
dst      = "~/.cache/dotfiles/eu.exelban.Stats.plist"
strategy = "plist-shallow"
when     = { deps = ["defaults"], os = "darwin" }

# 実行順（① overlay を合成 → ② hooks で反映）に合わせ、overlay を先・hooks を後に書く。
[[overlay]]
cmd = ["defaults", "export", "eu.exelban.Stats", "-"]  # base（常時。生きたドメイン全体）

[[overlay]]
src = "Stats.plist"  # dotfiles 管理サブセット（上書き）

[[hooks]]
cmd       = ["sh", "-c", "defaults import eu.exelban.Stats \"$HOME/.cache/dotfiles/eu.exelban.Stats.plist\""]
frequency = "always"  # 反映対象（ライブドメイン）は dotfiles 管理外で変化しうるため（§13.0）
```

合成は `base（export）→ 管理サブセット` の順（後勝ち）。Window Frame・メニューバー位置などローカル状態は base のまま残り、dotfiles が管理するキーだけ上書きされる。マージ済み dst を書き出したあと、`hooks` の `defaults import` がライブドメインへ反映する ― マージを事前に確定させるため、`defaults import` 自体がマージ／全置換のどちらでも結果が変わらない（正しさが未文書化の OS 挙動に依存しない）。ドメイン未作成（対象アプリ未起動）でも `defaults export <domain> -` は空 dict を返す（exit 0）ので、初回 apply でも base なしと同じに安全に扱える。

`[[overlay]]`（table header）は settings.json と同じ書き方で、`[[hooks]]` を後ろに続けても問題ない（`[[hooks]]` 自体が新しい table header なので、直前の `[[overlay]]` のフィールドとして誤読されることはない）。overlay → hooks の順に書いているのは、実行順（① overlay 合成 → ② hooks 反映）にファイルの並びを揃えるための整理であって、TOML の制約ではない。

#### 評価順と不変条件（先に固定）

実装差分をレビューしやすくするため、評価順は次を**不変条件**として先に固定する（細目は実装で詰めるが、この骨格は動かさない）：

1. **トップレベル `when`（ユニット gate）を最初に評価し、false なら短絡**。ユニットスコープの `when`（`deps` / `os`）を overlay より先に評価する。満たさなければ**ユニット全体を skip**（dst は一切触らない）し、overlay は評価しない。これは S2 の `when.deps` gate（依存欠落で生成丸ごと skip）の挙動をそのまま保存する。
2. **生き残ったユニットで overlay を宣言順に合成**。各 overlay の `when` を宣言順に評価し、満たすものだけを `strategy` に従い宣言順で重ねる（`json-shallow` は後勝ち、`concat` は後ろへ連結）。
3. **`preserve = true` の既存 dst は常に最下層（最初）**。dotfiles 断片を上に重ね（`json-shallow` の後勝ち）、**dotfiles が所有するキーだけを上書き**する。dotfiles が定義しないローカルキーは土台のまま残る（旧 `$local + $forced`）。「local 優先」ではなく「**dotfiles が自分の所有キーだけ書き、残りは local のまま**」が本質。

**「false の意味」が階層で異なる**点が要：ユニット gate=false は **dst ごと無し**（all-or-nothing）、overlay `when`=false は **その断片だけ脱落**（dst は残りの overlay で生成される）。前者は「`gh` が無ければ補完を作らない」、後者は「rtk が無くても settings.json は既存温存＋base で書かれる」を表す。

**gate=false は「配置しない」であって「撤去する」ではない。** エンジンは prune しない（§12.1）ので、gate が false へ**転じても**配置済みの実体は自動では消えない。効き方は階層で分かれる:

- **ユニット gate が false へ転じた場合**: そのユニットは丸ごと skip されるため、以前 true だった時に置いた dst（copy ツリー）は**そのまま残る**（撤去は手動か、別途 prune を入れる将来課題）。例: `dotfiles profile private` で `30-yt.fish` を置いたマシンを後で `dotfiles profile work` に付け替えても、`30-yt.fish` は残る。これは安全側の既定（未設定/新規マシンへ private 設定が**漏れない**）とは別問題で、private→他へ**再分類**したマシンでだけ起きる取り残し。現状は手動削除前提とする。
- **overlay `when` が false へ転じた場合**: その dst は毎 apply で**再合成**されるため、脱落した断片は次の apply で**結果から消える**（ファイルが書き直されるので取り残しは生じない）。`theme` の build-time overlay（§10.2.1）はこちらなので、テーマ切替で stale な断片は残らない。`theme` を**ユニット gate**として使う場合は前者の論点が同じく当てはまる。

**バリデーション（typo を黙殺しない）**：`preserve = true` は `strategy = "json-shallow"` 専用。`strategy` 省略や `concat` 等と併記したら **load 時エラー**にする（既存の「overlay 明示時は `strategy` 必須」「overlay は `src` / `cmd` のどちらか 1 つ」と同じ方針＝静かに無視せず配置前に弾く）。

#### 留保（実装スライスで詰める）

> ⚠️ 確認前の論点。

- 合成戦略は**内容型依存**（JSON=shallow-merge / テキスト=concat）。型ごとの戦略を manifest でどう宣言するか（`strategy` 明示 か dst 拡張子からの推測か）は実装時に確定する。
- `json-shallow` はトップレベル単位の差し替え（deep merge はしない）。`preserve = true` のとき既存 dst を土台に dotfiles 断片を上書きするので、旧 `modify_` の `jq '$local + $forced'`（非管理ローカルキーは保持・共有キーは dotfiles が上書き）と同値になる。深いマージが要る設定が現れたら別戦略を足す。
- `when` の複数キー（例 `deps` ＋ `os`）の結合は **AND で確定**（全て満たす時だけ採用。#493）。`deps` も配列内 AND。残る留保は `when.os` の複数許容（OR セマンティクス）と `theme` 状態の供給元（§10 の状態ファイル）で、必要になった時に別途詰める。

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
                           #   dst=~/.config/fish/completions/gh.fish, when={deps=["gh"]}
  claude/
    settings/
      manifest.toml        # strategy=json-shallow, dst=~/.claude/settings.json,
                           #   preserve=true, overlay=[base, rtk(when.deps=["rtk"])]
      settings.json
  ghostty/
    manifest.toml          # dst=~/.config/ghostty, theme="source",
                           #   when={deps=["ghostty"],os="darwin"}, hooks=[{cmd=["sh","-c","ln -sf …"]}]
    config
```

ポイント：

- **設定単位ごとに `manifest.toml` を置く**。kind が分岐するもの（gh の `config`=copy と `completion`=generate）は**ディレクトリを分けて別 manifest** にすることで自然に表現できる。
- **placement は明示**（`dst` を毎回書く）。これは冗長に見えるが、**「configs/<tool> の manifest を見れば配置先が一目で分かる」**ため、探しやすさはむしろ向上する。

### 6.2 manifest.toml スキーマ

```toml
# 配置先（必須）。copy は実体（ツリー）の置き先、generate / 合成は生成物（ファイル）の置き先。
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

# ユニット全体 gate（§5.5・§7）。トップレベルの when はユニットスコープ:
# false なら dst も hooks も触らずユニットごと skip（all-or-nothing）。
# when.deps（配列・AND）が全て PATH にあり、when.os（スカラ）が現在 OS と一致した時だけ採用。
when = { deps = ["gh"], os = "darwin" }

# 配置後フック（該当のみ。§13）。各エントリは cmd（argv）＋ frequency（実行頻度・省略時 onchange, §13.0）。
# inline table は TOML 上1行必須。複数行で書く / 末尾カンマが要るときは末尾に [[hooks]] を並べる。
hooks = [{ cmd = ["bat", "cache", "--build"] }]
# frequency = "always" を足すと毎 apply 無条件実行（既定は onchange＝ソース変化時のみ）:
# hooks = [{ cmd = ["some-tool", "--reflect"], frequency = "always" }]

# マシンローカル値（named value）の宣言（該当のみ。§9）。未設定は doctor が警告。
# locals = ["git.email", "git.name"]
# sensitive = ["github.token"]   # locals のうち秘匿値（エコー/ログ抑制）

# --- 合成 overlay（複数の条件付き断片を1 dst=ファイルへ重ねるとき。§5.5）---
# overlay 未記述 = 生成方式の既定挙動。暗黙 concat ではない:
#   - copy（dst=ディレクトリ）… 配下をツリーのまま個別配置。複数ファイルは普通に可・合成なし。
#   - generate（dst=ファイル）… cmd 出力＋同ディレクトリの sibling を連結。
# overlay/strategy は「dst=単一ファイルへ条件付き断片を重ねたい」時にだけ明示する。
# preserve = true                  # 既存 dst を最下層の土台に温存（dotfiles 非管理キーを保持）
# [[overlay]]
# src = "settings.json"            # この overlay の断片（copy）。または cmd=[…]（generate）
#
# [[overlay]]
# src  = "rtk.json"
# when = { deps = ["rtk"] }        # この断片だけ採用条件。deps（配列・AND） / os / theme
```

### 6.3 確定したルール

1. **管轄の再帰委譲**：あるディレクトリに `manifest.toml` があれば、そのディレクトリ＋（サブ manifest の無い）配下を管轄する。サブディレクトリに別の `manifest.toml` があれば、そこから先はそちらに委譲する（ツリーを manifest で分割統治）。
2. **生成方式ごとの src の扱い**：
   - `copy` … src ＝ 同ディレクトリの実ファイル（`manifest.toml` 以外）
   - `generate` … src 不要、`cmd` を持つ（ディレクトリは manifest だけでも可。補完がこれ）
   - overlay を明示するとき … 各 overlay が `src`（copy）/ `cmd`（generate）のいずれかを持つ。既存 dst の温存は overlay ではなくユニット属性 `preserve = true`（§5.5）。`merge` という独立 kind は廃し、**`strategy = "json-shallow"` ＋ overlay（＋必要なら `preserve`）** で表現する（§5.5）。
3. **粒度の指針**：ディレクトリ分割は強制ではなく、**属性（kind/dst/theme/hooks/`when`）が分岐するときに使う道具**。同じ dst でも「条件で内容が変わる」だけなら、ディレクトリを割らず `when` 付き overlay で表現する（rtk・テーマ別など）。同じ属性で済む範囲は1つの manifest にまとめてよい。
4. **読み込み順**：fish の `conf.d` 等の順序は、既存のファイル名番号（`05` < `40` < `90`）で表す。番号がグローバルな順序の単一の真実。

---

## 7. 属性スキーマ一覧（調査で実在を確認したもの）

| 属性 | 意味 | 必須 | 調査での実例 |
| --- | --- | --- | --- |
| `dst` | 配置先 | ✅ | 全ツール |
| `kind` | 生成方式（既定 copy）。copy / generate | 任意 | 補完=generate |
| `strategy` | 合成戦略（dst=ファイルへ複数断片を束ねる時）。concat / json-shallow / plist-shallow | 任意 | claude=json-shallow / gh 補完=concat（生成物＋独自ブロック）/ stats=plist-shallow（#531） |
| `cmd` | 生成コマンド（generate 時） | generate 必須 | `gh completion fish` 等 |
| `private` | 所有者のみ（0600 起点。chezmoi `private_`） | 任意 | 秘匿系（locals ストア・ssh 鍵 等） |
| `executable` | 実行ビット付与（0644→0755 / 0600→0700。chezmoi `executable_`） | 任意 | スクリプト/フック |
| `overlay` | 条件付き断片の配列（§5.5）。各 overlay = `src`/`cmd` ＋ `when?` | 任意 | claude=base+rtk |
| `when` | gate の採用条件。`deps`（配列・AND） / `os`（スカラ） / `profile`（スカラ・状態） / `theme`。**書く位置でスコープが決まる**: トップレベル＝ユニット全体 gate（false で dst も hooks も skip ＝ all-or-nothing）、`[[overlay]]` 内＝その断片だけの採否 | 任意 | ghostty=`{deps=["ghostty"],os="darwin"}` / yt=`{profile="private"}` |
| `preserve` | 既存 dst を最下層の土台に温存（`true` で dotfiles 非管理ローカルキーを**トップレベル単位で**全保持）。`json-shallow` 専用（他 strategy と併記は load 時エラー） | 任意 | claude=`true` |
| `theme` | light/dark 対応方式（runtime 追従の宣言。§10） | 任意 | ghostty=source / eza・delta=follower / bat・nvim・fzf=self |
| `locals` | マシンローカル値（named value）の宣言。`sensitive` で秘匿指定（エコー/ログ抑制）（§9） | 任意 | git `user.email`/`user.name` / yt の `yt.browser` |
| `hooks` | 配置後フックの配列。各エントリ `cmd`（argv）＋ `frequency`（onchange / always・§13.0） | 任意 | bat cache / ghostty symlink / git hooks symlink |

> ⚠️ **`theme`（トップレベル属性／`when.theme` の両方）は未配線**（#493 時点）。`Manifest` / `When` は `deny_unknown_fields` のため、color スライスが `theme` フィールドを足すまで `theme = …` を書いた manifest は **load 時エラー**になる。現役属性は本表のうち `theme` を除いたもの。

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

## 9. マシンローカル値（旧シークレット）

マシン固有の値（git identity・yt のブラウザ選択 `yt.browser` 等）は、ツール別の特殊解（git native include / mise create-only）を採らず、**単一の汎用機構**で扱う。現状の `env "DOTFILES_GIT_EMAIL/NAME"` 注入は廃止する。なお「値の穴埋め」と「断片の採否（gate）」は別軸で、後者（マシンクラスで設定を採るか捨てるか）は named value でなく `when.profile`（§5.5・§10.2.2）で表す（#467・yt が実例）。

二軸に分けて考える。**① named value（§9.1）= 事前に名前で宣言した値の穴埋め**、**② 任意ローカル設定の拡張口（§9.3・後続課題）= 宣言しないマシン固有設定を user が自由に足す口**。**S4（#458）は ① のみを対象**とし、② は別 issue で設計する。

### 9.1 named value ストア（① の構成要素）

1. **宣言（manifest）**: unit が要る値を名前で宣言する。属性は `locals = ["git.email", "git.name"]`。秘匿値は `sensitive = ["github.token"]` で指定（エコー/ログを抑制）。email/name は秘密ではない（commit に載る）ため非 sensitive。**`sensitive` は `locals` の部分集合であることを load 時に検証する（違反はエラー）** — typo で名前が `locals` とズレると非エコー/ログ抑制が黙って効かず秘匿値が漏れる footgun を防ぐ（manifest 検証の既存方針 §5.5 と同列）。
2. **単一ストア**: `~/.config/dotfiles/local.toml`（0600・dotfiles 非管理・gitignore 相当）。名前→値を全ツール横断で集約する。**repo には値を一切置かない。**
3. **取得（apply）**: apply が全 unit の `locals` を集約し、ストアに無い値を解決する。
   - **TTY**: その場で対話入力（sensitive は非エコー・ログ抑制）→ ストアへ 0600 で書く。
   - **非 TTY（フック実行等）**: 入力を求めず継続し、doctor 用に「未設定」を残す（apply はブロックしない）。
   - 明示経路: `dotfiles local set <name> <value>`。
4. **注入（placeholder 置換）**: **`locals` を宣言した unit の配置ファイル**に対し、生成方式（`copy` / `generate`）を問わず materialize 後に **named placeholder の置換のみ**（既定構文 `@@git.email@@`）でストア値を埋める。`copy` 配置の git config 等にも placeholder を置ける（置換は `locals` 宣言の無い unit には走らないため、無関係ファイルの `@@…@@` を巻き込まない）。条件分岐・関数を持つ汎用テンプレートは導入しない。形式非依存（git config / toml / 何でも効く）。
5. **診断（doctor）**: 宣言名がストアに在るかを見るだけ。ツール別ロジック不要（git の `git config --includes` 解決スコープに依存しない）。

### 9.2 畳み込んだ旧設計

- env 注入（`DOTFILES_GIT_EMAIL/NAME`）＋ `user.tmpl` の if 分岐は廃止。
- yt のブラウザ選択（`yt.browser`）は named value としてここに乗る。ただし「そのマシンで yt を使うか」というマシンクラス差は**値の穴埋めではなく断片の採否**なので、named value ではなく **profile gate**（`when.profile`・§5.5/§10）で表す ― yt は `dotfiles profile private` したマシンにだけ fish drop-in（`configs/yt`）を配置し、ブラウザ値だけを `yt.browser` から注入する（#467）。旧 `YT_BROWSER` env 変数は廃止（消費者は abbr のみで、env を介す必要がない）。これにより create-only（旧 §14/§16）は不要になる ― profile gate ＋ drop-in が「初回以降 managed 更新が届かない」create-only の上位互換だからである。
- 旧案の git native include は ① では採らない（doctor が include 解決のスコープ依存に振り回されるため。`--global`/`--file` は `--includes` 既定 OFF で local を辿らず誤判定しうる）。なお §14 の「ファイル合成」で使う `[include]` は**共有断片の組み立て用**で別物（repo 管理ファイルを束ねる話）。ここで不採用なのは*マシンローカル値の注入*に include を使う旧案に限る。

### 9.3 ② 任意ローカル設定の拡張口（後続課題・S4 対象外）

named value は「事前宣言した名前」しか扱えない（閉じた集合）。dotfiles 作者が予期しないマシン固有設定（社内 credential helper・signingkey・ローカル alias 等）を user が自由に足す**開いた上書き口**は別軸で、後続 issue で設計する。見立て（確定設計ではない）:

| 層 | 仕組み | 例 |
| --- | --- | --- |
| **drop-in 対応** | ツールがディレクトリを総読みする。gitignore したローカル断片を1個置く | fish `conf.d/*.fish` |
| **include 対応** | 置いた config が `*.local` を include し user が所有 | git `[include]` / ssh `Include` |
| **どちらも無い** | dotfiles が apply 時に overlay/concat（§5.5）でローカル断片を合成（ツール非依存の最後の砦） | 単一 config のみのツール |

新エンジンは不要で、**既存 overlay の source origin を repo からローカル既知ディレクトリへ広げる**話に畳める見込み。prune 挙動への懸念（ディレクトリ copy が source 外ファイルを消すと user のローカル断片を消す）は、現エンジンの **copy が dst を prune しない**性質でそのまま解ける ― dotfiles が書くファイルだけを上書きし、それ以外（user 所有のローカル断片）は触らない。**mise（#467）がこの drop-in 共存の最初の実例**で、managed な `~/.config/mise/config.toml`（dst=`~/.config/mise` ディレクトリへ copy）の傍らに、user 所有の `~/.config/mise/conf.d/*.toml` を無干渉で残す（同じく fish `conf.d` は複数ツールの drop-in 合流点）。include 層・overlay 合成層はモデルとして記録し、必要なツールが出たとき実装する。明示的な「ローカル断片パスを prune 対象から除外する」仕組みは、copy が prune する設計を将来入れる場合にだけ要る。

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

➡ **overlay は color を build-time で切り替える「もう一つの道」を開く。** `when.theme` は rtk(`when.deps`)・OS(`when.os`) と同じ gate 語彙の一員で、テーマ別断片もこの1機構に乗る。ただし**現状の runtime 追従（§10.1）を全面置換するものではない**ので、どのツールを runtime のままにし、どれを build-time overlay へ寄せるかは color スライスで個別に決める（過度な build-time 化は、既に動く追従を壊しうる）。

### 10.2.2 状態駆動 gate 族（`profile` と機構を共有する）

`theme` と `profile`（§5.5・§9.2）は同じ **状態駆動 gate** の族である ― `deps`/`os` が環境（PATH・OS）からその場で判る ambient な条件なのに対し、両者は **user が一度選んでおく状態**を読み、`when` でその状態と一致する断片だけを採用する。族なので **機構を共有**し、二度実装しない：

- **状態ファイル**: `~/.config/dotfiles/<key>`（`profile` / `theme`）。スカラ1値・平文・通常パーミッション（秘匿でないので named value ストア §9.1 の 0600 とは別物）。読み書きは共通の小機構（`state`）で、キー名でだけ分ける。
- **評価器の状態**: gate は apply 開始時に現在状態を **1回スナップショットへ解決**し、全ユニット・全 overlay の評価で共有する（評価ごとにファイルを読み直さない）。
- **配線状況**: 最初に配線されるのは `profile`（#467）。`theme` は color スライスで `When` に `theme` フィールドを足し、**同じ状態ファイル機構・同じ snapshot へ相乗り**する。

違い: `profile` は build-time の採否 gate 専用（マシンクラスで断片を採るか捨てるか）。`theme` は runtime 追従（§10.2.1）と build-time overlay の2系統を持ち、build-time 側だけがこの状態 gate を使う。

### 10.3 未確定の難所（PoC で検証）

> ⚠️ 以下は確認前の論点であり、事実として確定したものではない。

- テーマの**起点は Ghostty**（端末背景＋ANSIパレット）。真に手動で `dark` 固定するには、端末背景そのものを固定する必要があり、**Ghostty の theme 行も固定側へ切り替える**必要があるか検証が要る。Ghostty を OS追従のままにすると、端末背景は OS のままで手動固定と矛盾する可能性がある。
- `$fish_terminal_color_theme` は read-only のため、override は**別の状態変数**（例 `$dotfiles_theme_override`）を新設し、各連携をそれ優先に書き換える方針。実装時に詳細を詰める。

---

## 11. `dotfiles` コマンド体系（想定）

| コマンド | 役割 |
| --- | --- |
| `dotfiles apply [--source DIR]` | configs を走査し生成（copy/generate）×合成（overlay）を実行＋ hooks 起動。`locals` の未設定値は TTY なら対話取得（§9） |
| `dotfiles local set <name> <value>` | locals（マシンローカル値）をストアへ設定（§9）。秘匿かどうかは manifest の `sensitive` 属性で別途宣言するため、コマンド名は秘匿に限定しない（email/name 等の非秘匿値も扱う）。ストア名 `local.toml`・manifest 属性 `locals` と語が揃う |
| `dotfiles list` / `dotfiles status` | 管理ツール一覧・配置状況の俯瞰（分散 manifest を集約表示） |
| `dotfiles profile [<name>]` | マシンクラス（profile）の状態 gate を設定／表示（§10.2.2）。`<name>`（例 `private`）で状態ファイルへ書き、引数なしで現在値を表示。未設定の既定は not-private |
| `dotfiles color <dark\|light\|auto>` | テーマ手動固定／追従 |
| `dotfiles color sample` | ANSIカラー確認表（旧 color クレート吸収） |
| `dotfiles doctor` | 依存バイナリ・locals 未設定などの診断（chezmoi の doctor-check 相当） |
| `dotfiles --version` | 既存（バージョンの source of truth） |

`dotfiles list` が分散 manifest を集約して俯瞰を提供することで、分散方式の弱点「全体一覧が横断的」を補う。

---

## 12. 移行戦略（chezmoi 併用・#463 で完了）

> 移行は #463 で完了し、`home/`（chezmoi）は撤去された。本節は取った移行戦略の記録。

### 12.1 並行運用モデル

- ソースは `home/`（chezmoi）と `configs/`（dotfiles）を**併存**させる。
- 適用は **`chezmoi apply`（全部配置）→ `dotfiles apply`（移行済みツールだけ上書き）** の二段。
- 既存の `home/` に手を入れずに dotfiles を PoC 検証できる。問題が起きたら **`dotfiles apply` をやめて `chezmoi apply` だけ**で元の状態に戻せる（後勝ち上書き＋安全なフォールバック）。
- **不変条件（安全網の土台）**：移行スライス中は `home/` を**変更も削除もしない**（dotfiles は後勝ち上書きのみ）。`home/` が無傷で残っていることが「`dotfiles apply` をやめれば `chezmoi apply` だけで元へ戻せる」ロールバック保証の根拠であり、これを毀損する操作はスライス内で行わない。`home/` の物理削除は **S9（#463）で一括 trash 撤去した**。
  - このロールバック保証が及ぶのは **chezmoi が管理する対象（`home/` 起源で dst を持つもの）に限る**。dotfiles が `home/` 起源を持たず**新規生成**した chezmoi 管理外ファイルは `chezmoi apply` では戻らない（手動 or `dotfiles` 側の撤去が要る）。そうしたファイルを移行スライスで作る場合は、ロールバック手順を当該スライスで個別に定義する（§12.3 の管理対象前提を参照）。

### 12.2 段階移行の手順（ツール単位）

1. あるツール（例 `eza`）の設定を `home/` から `configs/eza/` へ**コピーして起こし**（`home/` 側は残す）、`manifest.toml` を付ける。
2. `dotfiles apply` で配置を検証（`chezmoi apply` の後勝ちで上書きされることを確認）。
3. `home/` 側は**削除せず据え置き**、dotfiles の後勝ち上書きで二重管理を許容する。**移行完了の定義は「`dotfiles apply` で当該ツールの配置が検証できた状態」**であり、`home/` のファイル削除は含めない（§12.1 不変条件）。
4. 全ツール移行が終わった後の **S9（#463）**で、chezmoi 関連（`.chezmoiscripts/`・`.chezmoi*`・`home/`）を**一括で** trash 撤去した。スライス単位での `home/` 個別削除は行わなかった。

### 12.3 留保（移行完了により解消）

- 移行期の「`dotfiles` を止めれば `chezmoi apply` で復帰する」フォールバックは、段階移行を問題なく完了できたため実運用上の破綻は生じなかった（#463 で `home/` 撤去）。

---

## 13. hooks の仕様

`hooks` 属性（§6.2・§7）はユニットが宣言する**フックの配列**で、ユニット配置後（after フェーズ）に**宣言順**で実行する。各エントリは実行する `cmd`（argv・コマンド列）と実行頻度 `frequency`（§13.0・省略時 `onchange`）を持つ。エンジンはツール固有知識を持たず `cmd` を**データ**として実行するだけ（`generate` の `cmd` と同じ思想・D9／§15）。どのフックが macOS 専用か等の知識は manifest 側（ghostty の `when.os` ＋ コマンド本体）が持ち、binary は関知しない（新ツールのフック追加に再コンパイル不要）。

### 13.0 実行頻度 `frequency`（#546）

各フックは実行頻度を `frequency` で選ぶ（省略時 `onchange`）。頻度は per-hook の属性で、別リスト（`always_hooks` のような）は作らない ― 頻度は per-hook の実行モードであり、リストを分けると構造が分岐して validation・表示ロジックが二重化するため。

| `frequency` | 実行モデル | chezmoi 対応 | 用途 |
| --- | --- | --- | --- |
| `onchange`（既定） | ユニットのソースが前回適用時から変わった時だけ実行（§13.1 の gate を通す） | `run_onchange_` | ソースが変わった時だけ反応すればよい（bat cache 再構築・ghostty / git hooks の symlink 生成） |
| `always` | 毎 apply 無条件に実行（gate を通さず状態も読み書きしない） | `run_` | 反映対象が dotfiles 管理外で随時変わり、ソースが変わらなくても毎 apply 反映したい（copy / compose と同じ「常に再実行」の性質。例: Stats.plist（#531）の `defaults import`） |

- **`always` は冪等性が前提**。gate を通さず毎 apply 無条件に走るため、コマンドは何度実行しても同じ結果になるものに限る（`ln -sf` のような上書き系・再構築系。例: `defaults import` は同一内容の再書き込みが安全）。
- **未インストールの skip は頻度によらず共通**（§13.1 の bare 名ガード）。`always` でもプログラムが PATH に無ければ skip する（gate を持たないので、`onchange` のようなハッシュ未保存の作用は無い）。

**背景（#531 で発覚した不整合）**: Stats.plist の反映 hook（`defaults import`）を既定の `onchange` のまま使うと、配置（`compose`。overlay の `defaults export` 断片は毎 apply 無条件に実行される）は常に正しく再計算されるのに、反映（hook）はソース（`configs/stats/Stats.plist`）が無変更の限り skip され続け、GUI 操作等によるライブドメインのドリフトが apply しても補正されなかった（実機で確認）。配置は「常時」・反映は「onchange 限定」という頻度の食い違いが原因で、`frequency = "always"` を新設し全段を「毎 apply 無条件」に揃えて解消した。

**意図的に見送る軸（#546・YAGNI）**。いずれもユースケースが出た時点で再検討する:

- **`once` 頻度**（初回のみ・以後スキップ。chezmoi `run_once_`）: 対応する頻度をまだ持たない。現状 `run_once_` に該当する用途が無いため見送る。
- **タイミング軸（配置前 / 後）**: 現状フックは配置後（after）固定（Stats を含む全 hook が「配置済みの dst を消費して反映する」形なので配置前のユースケースが無い）。もし配置前（before）が要るユースケースが出ても、これは頻度と混ぜず**別キー**（例 `timing = "before" | "after"`）で表す ― 頻度（いつ再実行するか）とタイミング（配置の前か後か）は独立に選べる別軸で、1 キーに畳むと直交する 2 つが絡み合うため。

### 13.1 onchange gate（再実行の条件）

`frequency = "onchange"`（既定）のフックは毎 apply で無条件には走らず、**ユニットのソースが前回適用時から変わったか**で gate する（`always` はこの gate を通さず毎 apply 実行する・§13.0）。状態は `~/.config/dotfiles/hooks.toml` に「フックキー → 前回適用時のソースハッシュ」で持つ（秘匿値でないので平文・通常パーミッション。破損時は warn して空＝全フック再実行＝冪等なので致命にしない）。

- **状態キー** = `<unit-rel>::<コマンド argv の短ハッシュ>`。コマンド内容をキーへ織り込むので、`manifest.toml` 上で**コマンドを変えた場合も新キー＝必ず再実行**になる（`manifest.toml` 自体はソースハッシュ対象外なため、この織り込みが無いとコマンド変更を取りこぼす）。
- **値** = ユニットの**ソース指紋**（`onchange::hash_dir`）。ユニットのデプロイ対象ファイル（`manifest.toml` を除く）を相対パス＋内容で非暗号学的ハッシュ（`std::hash`、`u64`）に畳む。前回値との等値比較にしか使わない（敵対者なし・改ざん検知でもない）ので暗号学的ハッシュは要らない。**mtime ではなく内容**を見る（mtime は touch / clone で揺れるため。§16 の旧未決項目「mtime vs ソースハッシュ」はここで確定）。
- **未インストール時の skip**: フックの**実行プログラム（`argv[0]`）が bare コマンド名**（区切りを含まない＝PATH 探索される名前。`bat` 等）のとき、PATH に無ければ skip し**ハッシュを保存しない**（後でインストールすれば次回 apply で再実行）。chezmoi の `command -v` ガードを、ツール名を binary に持たず汎用化したもの。**判定は `argv[0]` のみ**で、`["sh", "-c", "…"]` のように shell 経由で呼ぶ内側のコマンドは含まれない（内側依存が無ければ shell は非ゼロ終了し apply はエラーになる）。そうした内側依存は `when.deps`（§5.5）でユニットごと gate する。
  - **絶対パス／区切り付き相対パスの不在はエラー**: `argv[0]` が絶対パス、または区切り付き相対パス（§13.3 で manifest dir 基準に解決される `./script.sh` 等）の場合、その実体が無い `NotFound` は「任意ツールの未インストール」ではなく**ユニット同梱物の不在（typo / コミット漏れ）**なので skip せず apply をエラーで止める（空 argv を load 時に弾くのと同じ「実体化できないものを黙殺しない」方針）。skip は bare 名に限定する。
- **ユニット gate との関係**: トップレベル `when`（§5.5）が false のユニットは配置ごと skip されるため hooks も走らない（＝ `when.os` でフックの OS 分岐ができる）。

### 13.2 onchange ハッシュは locals 注入値を含まない（意図どおり）

`hash_dir` は**ユニットのソース**（`configs/<unit>` のファイル。`@@name@@` placeholder は未置換のまま）だけを入力にする。`~/.config/dotfiles/local.toml`（named value／§9）の値変更は**ソースハッシュに影響せず、hook 再実行のトリガーにならない**。locals の置換は apply 時に**配置先**ファイルへ注入される処理で、ソース自体は書き換わらないため。

これは `manifest.toml` をハッシュ対象外にするのと同じ理屈で、ハッシュは「**ソース内容の変化**」だけを追う。locals は apply 時に解決・注入される値でありソースではない。現状 locals 依存の hook は無いため実害は無い。

> **将来拡張**: 「注入値が変わったら再実行したい」locals 依存 hook を許可するなら、状態キー／ハッシュに解決後の locals を織り込む等の仕様を別途確定する（§16 未決事項）。

### 13.3 hook 実行の CWD 基準

**仕様（確定・実装済み #498）**: 相対パスの hook プログラム（`hooks = [{ cmd = ["./script.sh"] }]` のように `cmd[0]` が区切り付き相対パス）は、その**ユニットの `manifest.toml` があるディレクトリ基準**で解決する（フックスクリプトは manifest と同じ `configs/<unit>/` に置く想定）。絶対パス／PATH 解決される bare コマンド名（`bat` 等）はこの基準の影響を受けない。

`hooks::exec` はユニットの `manifest.toml` ディレクトリを `current_dir` に設定して hook を起動する。これによりフックの**実行時 CWD** と**相対パス引数**が manifest dir 基準で解決される。さらにプログラムパス（`argv[0]`）が区切り付き相対パスのときは同ディレクトリへ明示 join して解決する ― `current_dir` 任せの相対プログラムパス解決はプラットフォーム依存（unstable）なため、絶対化して曖昧さを消す（`unit_dir` を `std::path::absolute` で絶対化し、`current_dir` とプログラムパスの二重適用も避ける）。`argv[0]` が**区切りを含まない bare 名**（`bat` 等）は PATH 探索に委ね、**絶対パス**はそのまま使う（いずれも CWD 非依存）。現行の hooks（bat cache・ghostty symlink）は絶対パス／`$HOME`／PATH 解決のみなので挙動は不変。

---

## 14. chezmoi 責務 → dotfiles 代替 対応表（網羅性チェック）

| chezmoi 責務 | dotfiles での代替 |
| --- | --- |
| デプロイ（dot_/private_/executable_ 変換） | copy（生成方式）。dst・パーミッションは `private`/`executable` 属性で表現（§7） |
| `create_`（初回のみ生成・以後は温存。mise の machine-specific config 1件） | **profile gate ＋ native drop-in** で代替（create-only 不採用・#467）。mise は managed 設定（`[settings]`/`[tools]`）だけを copy（`configs/mise`）し、machine-specific だった穴は分離 ― ブラウザ値は yt の `locals`（`yt.browser`）、「yt を使うマシンか」は `when.profile`（§9.2）。user 所有の任意設定は drop-in で共存（copy が prune しない・§9.3）。create-only は「初回以降 managed 更新が届かない」死に筋なので採らない |
| symlink_（git hooks 13本） | copy or hooks（git hooks は dispatch への参照。配置方式は実装時確定） |
| 補完の動的生成（output） | generate（`cmd` ＋ `when.deps` gate） |
| ファイル合成（git config includeTemplate） | git native `[include]` に置換（copy へ降格）。bash 部品も同様に整理 |
| settings.json マージ（modify_） | 合成軸：`strategy="json-shallow"` ＋ overlay（base＋rtk）＋ `preserve=true`（既存 dst を土台に dotfiles 共有キーのみ上書き＝旧 `jq '$local + $forced'`。§5.5） |
| **テンプレート条件**（`{{ if lookPath … }}` / 部分的な if） | **`when` gate**（`deps` / `os` / `profile` / `theme`）。テンプレートエンジンは持たず宣言的 gate で表現（§5.5） |
| シークレット注入（env） | `locals` 属性＋ named value ストア＋ apply 取得（TTY 対話 / `local set`）。env 注入と git native include は不採用（§9） |
| `run_onchange_` フック（bat cache/ghostty/git hooks symlink） | `hooks` 属性（`frequency = "onchange"`・既定）。前回適用ソースのハッシュを状態に保存して比較（§13.1） |
| `run_` フック（毎 apply 無条件・管理外状態への反映。#531 の Stats.plist 反映で実例化） | `hooks` 属性の `frequency = "always"`（gate を通さず毎 apply 実行・冪等前提。§13.0・#546） |
| `run_once_` フック（初回のみ） | 未対応（対応する頻度をまだ持たない・YAGNI で見送り。§13.0・#546） |
| `.chezmoiignore` | 「configs に置かない」＝管理対象外。明示除外が要れば manifest で表現 |
| `.chezmoiroot` | 不要（configs がソースルート） |
| OS 分岐（`if eq .chezmoi.os`） | トップレベル `when.os`（ユニット全体 gate。§5.5） |
| `.chezmoi.config.sourceDir` 参照 | `--source`／作業ツリー検出／埋め込みで解決（§8） |

---

## 15. テスト方針：エンジン/テストはツールのライフサイクルから独立（D9）

`dotfiles` バイナリは**汎用エンジン**であり、`configs/` の個々のツール（claude / rtk / bat / ghostty / git …）は**いつか消える一時的なデータ**である。エンジンとそのテストはツール群より長生きするので、**「今のツールが全て入れ替わってもバイナリとテストは無変更で生存する」**ことを設計原則とする（実装もテストもツールのライフサイクルから独立）。

### 15.1 判定基準

> **configs から特定ツールを削除/改名したらテストが壊れるか? 壊れるなら defect。**

エンジンのテストが特定の実 config（`claude` / `git` など）の存在・名前・中身に依存していたら、それはツールのライフサイクルへ結合してしまっている。ツールの増減・改名・撤去でテストが赤くなる構造は**バグ**として扱い、下記の2層構造へ是正する。

### 15.2 テストの2層構造

| 層 | 目的 | 入力 | ツール増減への追従 |
| --- | --- | --- | --- |
| **契約テスト**（engine contract） | エンジンの挙動（生成方式 × 合成 × `when` gate × prune …）が仕様通りかを固定する | **hermetic な架空 fixture**（`foo` / `faketool` / `demo` 等）。実 configs を**名指し参照しない** | fixture はテスト内で自給するので、実 configs が変わっても**無影響** |
| **実 configs の妥当性確認**（data-driven） | 実際の `configs/` が manifest スキーマ・不変条件を満たすかを確認する | `configs/` の**全ユニットを走査**（特定ツール名をハードコードしない） | ツール増減に**無変更で追従**（走査対象が増減するだけ） |

- **契約テストは実 config を名指ししない。** 「rtk の settings.json が…」のような実ツール前提のアサーションではなく、架空の `faketool` fixture で overlay/`when`/`preserve` の挙動を検証する。
- **実 configs の検証は data-driven。** 「`configs/` 配下の全 manifest がパースでき、`sensitive ⊆ locals` 等の不変条件を満たす」のように、ツールを列挙せず走査で表す。新ツール追加時もテストコードは無変更。

### 15.3 明文化の扱い（これ以降は繰り返さない）

本原則は**この設計書（D9・本節）に一度だけ明文化**する。実装・テスト・コメントでは原則を再記述せず、必要なら本節を参照するだけにする（各所に散在させると整理困難になり、変更時に何箇所も直すことになるため）。

---

## 16. 未決事項・今後の検討

- [ ] manifest の `dst` 表記（`~` 展開、`$XDG_*` の扱い）の正規化ルール
- [x] copy のパーミッション表現（`private_`=0600 / `executable_`=0700 相当）の属性化 → `private` / `executable` 属性（S1 #455）
- [x] テンプレート条件（`lookPath` 等の部分的 if）の置き場 → **合成軸の overlay ＋ `when` gate**（§5.5・D8）。テンプレートエンジンは持たない。設計確定、実装は後続（リファクタリングで `merge` を `strategy`＋overlay へ移行し、rtk hook を `when.deps` 化）
- [x] `when` の評価順・ユニット gate との優先関係 → **不変条件として確定**（§5.5「評価順と不変条件」: ①ユニット gate 先・false で短絡 ②overlay は宣言順 ③`preserve=true` の既存 dst は常に最下層＝土台）
- [x] gate 語彙の統一 → **`when` 一本へ統合**（#493）。unit 属性 `deps`/`os` を廃し、トップレベル `when`（ユニットスコープ）と overlay 内 `when`（断片スコープ）で同じ語彙を共有。`when.deps` は配列・AND、`when.os` はスカラ。`when` 複数キーの結合も **AND で確定**
- [ ] 合成戦略の宣言方法（`strategy` 明示 vs dst 拡張子からの推測）、`when.os` の複数許容（OR セマンティクス）（§5.5 留保）。なお `theme` 状態の**供給元**（状態ファイル機構）は `profile` が確立済み（§10.2.2）で、`theme` は `When` にフィールドを足して相乗りするだけ
- [ ] `when.theme` を build-time color に使う範囲（どのツールを runtime 追従のまま残すか。§10.2.1）
- [x] create-only 属性（chezmoi `create_` 相当）＋ mise の `config.toml` 移行 → **profile gate ＋ native drop-in で代替・create-only 不採用**（#467）。機械クラス差の正体は「値の穴埋め」ではなく「断片を採るか捨てるかの gate」なので、`when.profile`（マシンクラス状態 gate・§5.5/§10.2.2）で表す。mise は managed 設定だけを copy（`configs/mise`）、yt は `configs/yt` の fish drop-in（`when.profile="private"` ＋ `locals=["yt.browser"]`）へ集約。S1 で見送った mise が解け **S9（#463）のブロッカーが外れた**
- [x] `when.profile` 状態 gate の供給元・既定・コマンド → **確定**（#467）。状態源は `~/.config/dotfiles/profile`（`dotfiles profile <name>` が書く・スカラ1値・平文）、既定は **not-private**（新規/仕事マシンへ private 設定が漏れない明示 opt-in）、状態確認は `dotfiles profile`（引数なし）。状態ファイル機構・評価器の状態 snapshot は `theme` と共有（§10.2.2・二重実装回避）
- [ ] **② 任意ローカル設定の拡張口**（§9.3）= 宣言しないマシン固有設定（社内 credential helper・signingkey・ローカル alias 等）を user が自由に足す開いた上書き口。drop-in / include / overlay 合成の3層で受け、apply の prune 対象外を決める。S4（① named value）の後・近時期に別 issue で設計。S4 必須ではない
- [x] git hooks（symlink_ 13本）の配置方式（copy か、配置後にリンク生成か）→ **配置後にリンク生成で確定**（#535）。全 hook 種別は同じ内容（`dispatch`）を起動名で分岐するだけなので、`dispatch` 1 本だけを copy し（`configs/git/hooks`, `executable = true`）、hook 名 13 本は配置後の onchange フックで `dispatch` への symlink として生成する（ghostty の symlink フックと同じパターン、§13）。dispatch を 13 回複製 copy しない
- [x] hooks の onchange 検知方式（ソースハッシュ vs mtime）→ **ソースハッシュで確定**（§13.1・S5 #486）。mtime は touch/clone で揺れるため内容ハッシュを採る
- [ ] **locals 依存 hook** を許可する場合の仕様（注入値が変わったら再実行したい用途）。現状は onchange ハッシュが locals 注入値を含まない＝意図どおり（§13.2）。許可するなら状態キー/ハッシュへ解決後 locals を織り込む等を確定する
- [x] **相対パス hook**（`hooks = [{ cmd = ["./script.sh"] }]`）の実行基準 → **ユニットの `manifest.toml` ディレクトリ基準で確定・実装済み**（§13.3・#498）。`hooks::exec` が manifest dir を `current_dir` に設定し、区切り付き相対 `cmd[0]` を同ディレクトリへ明示解決する（bare 名は PATH 探索・絶対パスは素通し）
- [x] **hooks の実行頻度**（常時 / onchange / 一度きり）→ **`frequency` 属性で確定**（§13.0・#546）。既存 `hooks` エントリの属性として `onchange`（既定・ソース変化時のみ）/ `always`（毎 apply 無条件・冪等前提）を選ぶ（別リストは作らず、構造の分岐と表示・validation の二重化を避ける）。`once` 頻度（chezmoi `run_once_`）・タイミング軸（配置前 / 後）は該当ユースケースが無いため意図的に見送り（YAGNI・出た時点で再検討。タイミングは頻度と直交する別キーで表す）。#531 の Stats.plist 反映（配置は常時・反映は onchange 限定という食い違いで drift 補正が効かなかった）が動機
- [ ] color 手動固定時の Ghostty 起点の扱い（§10.3）
- [x] chezmoi 併用時のフォールバック実挙動（§12.3）→ 段階移行を問題なく完了し `home/` 撤去（#463）
- [x] named value の窓口を `secret set` → **`local set`** へ確定（§9・§11・#522）。`secret` は秘匿値以外（email/name 等）も扱い概念とズレるため改名。ストア名 `local.toml`・manifest 属性 `locals` と語が揃う（CLI 未配布＝互換性の制約は無くエイリアスは残さない）
- [ ] 残るサブコマンド名の最終確定（`apply` 等の仮称。§11）

---

## 付録：合意済みの設計決定（サマリ）

1. 第一級エンティティは**ツール**。ソースは中身の帰属で並べ、配置先は属性。
2. 配置は **2軸 ＝ 生成方式（copy / generate）× 合成（単一 / concat / json-shallow / plist-shallow）**。`merge` は独立 kind ではなく合成の戦略（内容型ごとに JSON 版・plist 版）。symlink は不採用（cargo install 配布と非互換）。
3. dst は **条件付き overlay の合成**として組む。各 overlay は断片＋ `when`（`deps` / `os` / `theme`）gate を持ち、rtk・OS・テーマ分岐を1機構へ統合（chezmoi テンプレート不要）。gate 語彙は `when` 一本で、スコープは書く位置で表す（トップレベル＝ユニット全体 gate / overlay 内＝断片 gate）。`preserve = true` は既存 dst を最下層の土台に温存（dotfiles 所有キーのみ上書き＝旧 `$local + $forced`）。
4. データ構造は **`configs/` 階層 ＋ 各設定単位の `manifest.toml`（placement 明示）**。属性が分岐するときはディレクトリ分割、内容だけが条件で変わるなら `when` 付き overlay。
5. ソースは **埋め込み（本番）＋作業ツリー直読み（dev/移行期）** の二段構え。
6. color は **`dotfiles color`** に統合（切替＋サンプル）。runtime 追従（既定）と build-time overlay（`when.theme`）の2系統をツールごとに使い分ける。
7. 移行は **chezmoi 併用で段階実施し #463 で完了**（`chezmoi apply` → `dotfiles apply` の二段でツール単位に移行、最後に `home/` を撤去）。
8. **エンジン/テストはツールのライフサイクルから独立**（D9・§15）。判定基準＝「configs から特定ツールを削除/改名したらテストが壊れるなら defect」。契約テストは hermetic な架空 fixture、実 configs の検証は data-driven 走査。原則は設計書に一度だけ明文化し、各所で繰り返さない。
