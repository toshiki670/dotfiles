# Changelog

## [0.71.3] - 2026-07-16
### Features
- 呼び出し時だけ CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS を有効化する claude-teams 関数を追加する ([#647](https://github.com/toshiki670/dotfiles/pull/647)) ([`c6f6d94`](https://github.com/toshiki670/dotfiles/commit/c6f6d94863a9734ad1ebe23404a0390e59a886c8))
- Design-creed の作業前列挙と prose-tidy の出所先行判定を追加する ([#651](https://github.com/toshiki670/dotfiles/pull/651)) ([`5223e5a`](https://github.com/toshiki670/dotfiles/commit/5223e5ab04a4c9875b2671c2e9aaec90a45b0845))
### Fixes
- Claude -p の JSON 出力を --json-schema で強制する ([#645](https://github.com/toshiki670/dotfiles/pull/645)) ([`977b01c`](https://github.com/toshiki670/dotfiles/commit/977b01c2bbe5b29cb61936bd5daaa58669149e36))


## [0.71.2] - 2026-07-14
### Features
- Prose-tidy skill に「置換より削除」の自己点検を追加する ([#640](https://github.com/toshiki670/dotfiles/pull/640)) ([`da1f043`](https://github.com/toshiki670/dotfiles/commit/da1f04375f0cc6671217d639bace6be86187291b))
### Refactor
- StepSource のパス表記検証をパースへ変え、下流の再パースを無くす ([#632](https://github.com/toshiki670/dotfiles/pull/632)) ([`dbc8006`](https://github.com/toshiki670/dotfiles/commit/dbc8006eb86903419db4bbc29899660e41319b7c))
- PATH 実行ファイル存在確認を which crate に統一する ([#635](https://github.com/toshiki670/dotfiles/pull/635)) ([`52b478a`](https://github.com/toshiki670/dotfiles/commit/52b478aa683f395ce2dd3be21c18ba05cb5d9c4e))
- Env-tools を upkeep へリネームし、サブコマンド名の重複と fish 補完欠落を解消する ([#639](https://github.com/toshiki670/dotfiles/pull/639)) ([`6241f4a`](https://github.com/toshiki670/dotfiles/commit/6241f4a189d208bed419062d89e4dc7c156b299e))
- Completion manifest.toml のコメント重複を削る ([#641](https://github.com/toshiki670/dotfiles/pull/641)) ([`6d85fba`](https://github.com/toshiki670/dotfiles/commit/6d85fbac58c95e7bdc02e899e31d548070738d6b))
- Fzf-picker E2E doc の「各 bin」表記をサブコマンドへ揃える ([#642](https://github.com/toshiki670/dotfiles/pull/642)) ([`3b51db1`](https://github.com/toshiki670/dotfiles/commit/3b51db14be7bfeeaeb0035a75ed34eca13dcd15d))


## [0.71.1] - 2026-07-13
### Features
- Homebrew tap 経由の自動 formula 更新を追加する ([#619](https://github.com/toshiki670/dotfiles/pull/619)) ([`c3893ad`](https://github.com/toshiki670/dotfiles/commit/c3893adcc09ccaa32ca8f808dbe3eb6f7a9886bf))
### Fixes
- Use fully-qualified brew install command ([#620](https://github.com/toshiki670/dotfiles/pull/620)) ([`0e235c1`](https://github.com/toshiki670/dotfiles/commit/0e235c15717c4145a0ec2b68b65a4140ae1fc1ad))
- When.os の typo を load 時に弾く（silent skip をやめる） ([#629](https://github.com/toshiki670/dotfiles/pull/629)) ([`bd5a571`](https://github.com/toshiki670/dotfiles/commit/bd5a5712ff1d1af893a187df68420d0da5e51906))
### Refactor
- Tree/Pipeline と step の input/output 択一を型で表現する ([#622](https://github.com/toshiki670/dotfiles/pull/622)) ([`6472cf9`](https://github.com/toshiki670/dotfiles/commit/6472cf9a3599f17480852e3c75189313a3858af9))


## [0.71.0] - 2026-07-12
### Features
- Rustdoc doc コメントの無駄を削る skill を追加する ([#583](https://github.com/toshiki670/dotfiles/pull/583)) ([`e546538`](https://github.com/toshiki670/dotfiles/commit/e546538600406c0698947d40e43219d05121a6d1))
- Memory-tidy skill に有用性判定と階層化エスカレーションを追加する ([#586](https://github.com/toshiki670/dotfiles/pull/586)) ([`bf71f07`](https://github.com/toshiki670/dotfiles/commit/bf71f0745b9c1e883da8a16a3b5fa2a4e97d65ac))
- Design-creed を prose 文書に適用する prose-tidy skill を追加する ([#599](https://github.com/toshiki670/dotfiles/pull/599)) ([`b67d151`](https://github.com/toshiki670/dotfiles/commit/b67d151cceb97f92f1803667082d958cdabed1d2))
- Prose-tidyとtextlintのskillを分離する ([#603](https://github.com/toshiki670/dotfiles/pull/603)) ([`c588492`](https://github.com/toshiki670/dotfiles/commit/c588492841ee11e1bb94bcc39b878ef9672cd80a))
- Steps の merge = "deep" を実装する ([#604](https://github.com/toshiki670/dotfiles/pull/604)) ([`689efb4`](https://github.com/toshiki670/dotfiles/commit/689efb43841cefceca52edcb3ef53ce2c2868bd7))
- 期待配置集合の導出とユニット間 output 衝突検出を実装する ([#605](https://github.com/toshiki670/dotfiles/pull/605)) ([`93acb00`](https://github.com/toshiki670/dotfiles/commit/93acb002b804940f765894095e72092b4d6d1543))
- Plist_dict! マクロでテストの Dictionary 手動構築を置き換える ([#612](https://github.com/toshiki670/dotfiles/pull/612)) ([`48de662`](https://github.com/toshiki670/dotfiles/commit/48de662c34308a19b15c3b7dbcc866d34e9bf071))
- 不要になった配置の追跡・退避を実装する ([#616](https://github.com/toshiki670/dotfiles/pull/616)) ([`112b760`](https://github.com/toshiki670/dotfiles/commit/112b7609336b5bf49a8a3e7104a38f43f93fddbc))
### Fixes
- Trash への空文字列引数渡しをフックで検知する ([#617](https://github.com/toshiki670/dotfiles/pull/617)) ([`1a031c8`](https://github.com/toshiki670/dotfiles/commit/1a031c8c330567fbfb4cfa74509befb11ebc0f2e))
### Refactor
- [**BREAKING**] Manifest.toml を steps パイプライン(input→merge→output)へ移行する ([`326fdc5`](https://github.com/toshiki670/dotfiles/commit/326fdc5aaeb160545d5204bb0c09fee666e99e97))


## [0.70.0] - 2026-07-04
### Features
- MacOS アプリの preference plist を一級扱いする(Stats.plist 移行) ([#559](https://github.com/toshiki670/dotfiles/pull/559)) ([`6940e68`](https://github.com/toshiki670/dotfiles/commit/6940e68e285fd730ed91f96bc67b9779a18a307d))
- Fish 本体を configs/ へ移行する ([#561](https://github.com/toshiki670/dotfiles/pull/561)) ([`068cf1d`](https://github.com/toshiki670/dotfiles/commit/068cf1daf7813a28a2b8805ed469cf31f222175a))
- Gh の config.yml を configs/ へ移行する ([#568](https://github.com/toshiki670/dotfiles/pull/568)) ([`1723b70`](https://github.com/toshiki670/dotfiles/commit/1723b70d83bdf85899a72f8e2eda9c8d8149a093))
- Merge-ready.toml を configs/ へ移行する ([#569](https://github.com/toshiki670/dotfiles/pull/569)) ([`8d2db1b`](https://github.com/toshiki670/dotfiles/commit/8d2db1b93992d6f3d6b50e382c2e55617d723904))
- Fish shim 8本を各ツールへ帰属させ configs/ へ移行する ([#570](https://github.com/toshiki670/dotfiles/pull/570)) ([`34a3664`](https://github.com/toshiki670/dotfiles/commit/34a366429bf5d19d4c06bba8debd800b56e23410))
- V-sync を chezmoi 非依存化する（lazy-lock を configs へ書き戻す） ([#573](https://github.com/toshiki670/dotfiles/pull/573)) ([`039e507`](https://github.com/toshiki670/dotfiles/commit/039e5074c46675fefbd94c168e03a9b165d2cdeb))
- [**BREAKING**] Chezmoi を撤去し dotfiles 単独運用にする ([#463](https://github.com/toshiki670/dotfiles/pull/463)) ([#575](https://github.com/toshiki670/dotfiles/pull/575)) ([`375c59a`](https://github.com/toshiki670/dotfiles/commit/375c59a26953b0aae06e1f39952e6219ac3faa2f))


## [0.69.10] - 2026-07-02
### Features
- Hook の実行頻度 frequency を追加(onchange/always) ([#555](https://github.com/toshiki670/dotfiles/pull/555)) ([`7a4b357`](https://github.com/toshiki670/dotfiles/commit/7a4b35705d45e5fbdd7bd4120ead76f89bb8a979))
### Fixes
- Force-push guard を native settings へ移植する ([#553](https://github.com/toshiki670/dotfiles/pull/553)) ([`2874c56`](https://github.com/toshiki670/dotfiles/commit/2874c56c20b85a61412de1411c0ac90b0edb7523))


## [0.69.9] - 2026-07-01
### Features
- Starship 設定を configs/ へ移行し $schema を網参照化 [#532] ([#536](https://github.com/toshiki670/dotfiles/pull/536)) ([`a6a11b1`](https://github.com/toshiki670/dotfiles/commit/a6a11b1a6c378e62d5b36189a6f2b118608a8c65))
- Bash profiles を configs/ へ移行し native source で合成 [#534] ([#538](https://github.com/toshiki670/dotfiles/pull/538)) ([`ece0303`](https://github.com/toshiki670/dotfiles/commit/ece0303c1a690c022def1fc355e51652772bb794))
- Cursor のルールを configs/ へ移行する ([#540](https://github.com/toshiki670/dotfiles/pull/540)) ([`6bf38c6`](https://github.com/toshiki670/dotfiles/commit/6bf38c61b17b743aecd8d85b179054dae751c18f))
- Git の hooks と ignore を configs/ へ移行する（§16 配置方式の決定を含む） ([#541](https://github.com/toshiki670/dotfiles/pull/541)) ([`ce4a745`](https://github.com/toshiki670/dotfiles/commit/ce4a745423d1f283b537816c0ab8d0f57cd65c25))
- _fzf_file / _fzf_history を configs/fish へ移行する ([#542](https://github.com/toshiki670/dotfiles/pull/542)) ([`0693fee`](https://github.com/toshiki670/dotfiles/commit/0693feedcef64b78bc387bc4b1cd5cc284e3b9d1))
### Fixes
- Worktree-remove の [y/N] を raw tty 経由で読めるよう cbreak 化 ([#537](https://github.com/toshiki670/dotfiles/pull/537)) ([`03c039d`](https://github.com/toshiki670/dotfiles/commit/03c039da26458267df235f352d2d6d9d1ea60e5a))


## [0.69.8] - 2026-06-29
### Features
- Machine-specific 設定を profile gate で配置（yt 集約・mise copy 移行）[#467] ([#520](https://github.com/toshiki670/dotfiles/pull/520)) ([`d187e58`](https://github.com/toshiki670/dotfiles/commit/d187e58bd45e2933e6e8e73d4c518f15bb26a8ed))
- Completions を top-level --completions フラグ化し Tab 候補から隠す [#526] ([#527](https://github.com/toshiki670/dotfiles/pull/527)) ([`38820b9`](https://github.com/toshiki670/dotfiles/commit/38820b9b4c46060575f27554dc27bf467bce5f1a))
### Refactor
- 配布物を root dotfiles 1パッケージ（複数 bin）へ統合 [#485] ([#523](https://github.com/toshiki670/dotfiles/pull/523)) ([`001fa2a`](https://github.com/toshiki670/dotfiles/commit/001fa2a2155db795fc24a9309ce733772e34603e))
- サブコマンドを secret set → local set へ改名 [#522] ([#525](https://github.com/toshiki670/dotfiles/pull/525)) ([`e2cb0c2`](https://github.com/toshiki670/dotfiles/commit/e2cb0c2c3d1ab67d63e585470c1ea01ec801d0ba))
- SHA-256 を std の非暗号学的指紋へ置換し strum を 0.28 へ [#524] ([#528](https://github.com/toshiki670/dotfiles/pull/528)) ([`a57bb09`](https://github.com/toshiki670/dotfiles/commit/a57bb091d60e798cd36025398b4f89a692ca068e))


## [0.69.7] - 2026-06-27
### Features
- ソース解決の二段構え（埋め込みフォールバック）[#462] ([#514](https://github.com/toshiki670/dotfiles/pull/514)) ([`7df9c52`](https://github.com/toshiki670/dotfiles/commit/7df9c52b9d209f3fd4134ef321a7554973279ac5))
### Refactor
- Src/ をドメイン責務でディレクトリ化（純移動のみ）[#515] ([`3fdbf6e`](https://github.com/toshiki670/dotfiles/commit/3fdbf6ea0782792b01b9a0e5fad40d69724ea6e1))
- Src/ をドメイン責務で配線（mod パス・doc リンク）[#515] ([`0353f80`](https://github.com/toshiki670/dotfiles/commit/0353f804784056e519d7bab9707c22d98f062f65))
- Locals::inject を private へ戻す（可視性基準を統一）[#515] ([`7a13f96`](https://github.com/toshiki670/dotfiles/commit/7a13f96ed3bd7ad875e13e074d1e3f6f7ccaeb20))


## [0.69.6] - 2026-06-24
### Features
- 相対パス hook を manifest.toml ディレクトリ基準で実行する [#498] ([#499](https://github.com/toshiki670/dotfiles/pull/499)) ([`99762c3`](https://github.com/toshiki670/dotfiles/commit/99762c35e8ee9e563d2dd4cbfad915a0113b11ea))
- AI 普遍設計信条 design-creed をグローバル rule に追加（6 + 1 / Stable core, humble edges）[#490] ([#501](https://github.com/toshiki670/dotfiles/pull/501)) ([`f759d19`](https://github.com/toshiki670/dotfiles/commit/f759d19adfc13f6b605754ce7536d201f8c0add6))
- Color sample を dotfiles へ吸収し旧 color クレートを掃除 [#460] ([#502](https://github.com/toshiki670/dotfiles/pull/502)) ([`1870fe2`](https://github.com/toshiki670/dotfiles/commit/1870fe24808192c6f5e96a677b6e0f680ff82157))
- Home/dot_claude の rules / skills / statusline を native configs へ移行 [#505] ([`c8e2542`](https://github.com/toshiki670/dotfiles/commit/c8e25420ca0ef48813b1a53b2efde87a9acdca28))
### Refactor
- Kind / Strategy の表示文字列を Display に集約し複製を解消 [#506] ([`3c88a7f`](https://github.com/toshiki670/dotfiles/commit/3c88a7f31ffa7b8494ead0623efaa216954d9228))
- Kind / Strategy の Display を strum derive へ（変種名を唯一の出所に）[#506] ([`b8238c9`](https://github.com/toshiki670/dotfiles/commit/b8238c9941eaad7bb0e297dbaa957b4d74edf371))


## [0.69.5] - 2026-06-22
### Features
- Apply 最小骨格（固定ソース configs/・copy のみ）[S0 #454] ([#464](https://github.com/toshiki670/dotfiles/pull/464)) ([`336bed3`](https://github.com/toshiki670/dotfiles/commit/336bed33d37c434e1bf7ee41d8f089b514508497))
- Copy 層拡張＋dotfiles list、eza/nvim/bat 移行 [S1 #455] ([#466](https://github.com/toshiki670/dotfiles/pull/466)) ([`bb64b86`](https://github.com/toshiki670/dotfiles/commit/bb64b86f499dce7d042ef33d72534a1a57bf8f4e))
- Generate 層＋deps gate、補完5本を configs 化 [S2 #456] ([#468](https://github.com/toshiki670/dotfiles/pull/468)) ([`2063a23`](https://github.com/toshiki670/dotfiles/commit/2063a238378d9de7e4ae54a8104e2c6982cb332f))
- Claude settings を json-shallow overlay で配置 [S3 #457] ([#469](https://github.com/toshiki670/dotfiles/pull/469)) ([`51f85d9`](https://github.com/toshiki670/dotfiles/commit/51f85d9cd64da5fd659df0abdd5d9370a26cfcc0))
- マシンローカル値 named value 機構を実装 [S4 #458] ([#484](https://github.com/toshiki670/dotfiles/pull/484)) ([`9906b0b`](https://github.com/toshiki670/dotfiles/commit/9906b0bda5efa150428627011a2a10c98ddbb67a))
- Onchange フック機構（manifest 宣言コマンドの汎用実行）を実装し bat/ghostty を移行 [S5 #459] ([#486](https://github.com/toshiki670/dotfiles/pull/486)) ([`eb2847b`](https://github.com/toshiki670/dotfiles/commit/eb2847b8de33ff54a3cbe9da17a02fad9128e6c3))
### Refactor
- 配置を2軸（生成方式×合成）＋overlay/when へ再構成 [#471] ([#472](https://github.com/toshiki670/dotfiles/pull/472)) ([`2d58ba2`](https://github.com/toshiki670/dotfiles/commit/2d58ba2ccaa9ad52a9737e46c4d73602b10589f2))
- Settings 既存温存を $local+$forced 意味論へ（preserve=true・json-shallow 土台化） [#475] ([#476](https://github.com/toshiki670/dotfiles/pull/476)) ([`a4a1a4a`](https://github.com/toshiki670/dotfiles/commit/a4a1a4accf28582072bcd194e660e79771f281e1))
- Force-push guard を Claude PreToolUse に一本化 [#473] ([#482](https://github.com/toshiki670/dotfiles/pull/482)) ([`9874929`](https://github.com/toshiki670/dotfiles/commit/9874929e923e6c1692b8a17da2a1ef5794dcc280))
- Gate 語彙を when へ一本化（unit deps/os 廃止）[#493] ([#495](https://github.com/toshiki670/dotfiles/pull/495)) ([`949f618`](https://github.com/toshiki670/dotfiles/commit/949f6184a88bdfc5879abe48fe4ffa3107e70b94))


## [0.69.4] - 2026-06-19
### Features
- Gradle のプロンプトから via を非表示化 ([#445](https://github.com/toshiki670/dotfiles/pull/445)) ([`9148e25`](https://github.com/toshiki670/dotfiles/commit/9148e25b693ccd2d5807fb1ecd9577b15ddc31dc))
- コミット時に gitleaks 検査するグローバル hook を全ホストに追加 ([#447](https://github.com/toshiki670/dotfiles/pull/447)) ([`03ac8a3`](https://github.com/toshiki670/dotfiles/commit/03ac8a38cc30e8607d25c71d20e738652551c983))
- Issue/PR 横断 fzf ピッカー追加 + gh ID 補完バグ修正 ([#448](https://github.com/toshiki670/dotfiles/pull/448)) ([`7236104`](https://github.com/toshiki670/dotfiles/commit/7236104c4ac74173bd2866e0f9d5ccef8591f691))
- ファイル削除を trash に誘導（指示ルール主役＋ガードフック保険） ([#449](https://github.com/toshiki670/dotfiles/pull/449)) ([`01ecf4e`](https://github.com/toshiki670/dotfiles/commit/01ecf4edc580a22e7018788dd630851787327378))


## [0.69.3] - 2026-06-18
### Features
- Copy-obj / c-file / c-path を clip コマンドへ統合 ([#435](https://github.com/toshiki670/dotfiles/pull/435)) ([`d9d247e`](https://github.com/toshiki670/dotfiles/commit/d9d247eb30bf6235a00594e0b10c84c79d917da0))
- Java/kotlin のプロンプトから via を非表示化 ([#440](https://github.com/toshiki670/dotfiles/pull/440)) ([`31f0f84`](https://github.com/toshiki670/dotfiles/commit/31f0f84b79c6ef39445f4cbe0ce799e668e6111b))
### Fixes
- トリガーを /memory-tidy のみに限定 ([#433](https://github.com/toshiki670/dotfiles/pull/433)) ([`355f16d`](https://github.com/toshiki670/dotfiles/commit/355f16df411ca65ed90b8531d7b45e6eec612310))
- Cargo-install フックを run_before 化し補完の初回取りこぼしを修正 ([#438](https://github.com/toshiki670/dotfiles/pull/438)) ([`9307bc7`](https://github.com/toshiki670/dotfiles/commit/9307bc7827e154f8e72a6485fd75015d30d3a837))
- 依存更新だけの空 patch bump を release_commits で抑止 ([#443](https://github.com/toshiki670/dotfiles/pull/443)) ([`4a5dcad`](https://github.com/toshiki670/dotfiles/commit/4a5dcad46bca85025c911f4639858f529c0a308a))


## [0.69.2] - 2026-06-17
### Fixes
- Root の Release 名から dotfiles- 接頭辞を外す ([#431](https://github.com/toshiki670/dotfiles/pull/431)) ([`72c6cf6`](https://github.com/toshiki670/dotfiles/commit/72c6cf6b43d2890191713be43ca337aa857112d0))


## [0.69.1] - 2026-06-16
### Features
- _fzf_ghq_cd を Rust 化（B群 Rust化 PR1/3） ([#413](https://github.com/toshiki670/dotfiles/pull/413)) ([`a041941`](https://github.com/toshiki670/dotfiles/commit/a041941bdc23a195a816dd496376d504c289646a))
- _fzf_worktree_remove を Rust 化（B群 Rust化 PR2/3） ([#414](https://github.com/toshiki670/dotfiles/pull/414)) ([`0884b97`](https://github.com/toshiki670/dotfiles/commit/0884b973f5e5f4d4ea0e5b47c0ac8ad929bfae9c))
- Cdabbr を Rust 化（B群 Rust化 PR3/3・完了） ([#415](https://github.com/toshiki670/dotfiles/pull/415)) ([`ef312d0`](https://github.com/toshiki670/dotfiles/commit/ef312d08243cfdc7ee23888f4669e39378847043))
- メモリ棚卸し Skill memory-tidy を追加 ([#420](https://github.com/toshiki670/dotfiles/pull/420)) ([`4d8ce5c`](https://github.com/toshiki670/dotfiles/commit/4d8ce5ccb7b36f8aff53efcf8ff863e99e827520))
- Cleanup-env・upgrade-env を Rust 化 ([#390](https://github.com/toshiki670/dotfiles/pull/390)) ([#421](https://github.com/toshiki670/dotfiles/pull/421)) ([`876dacd`](https://github.com/toshiki670/dotfiles/commit/876dacd14aff91ca396dd12a1c5441a15f007c85))
### Fixes
- Per-package CHANGELOG.md も markdown lint から除外 ([#411](https://github.com/toshiki670/dotfiles/pull/411)) ([`336fe09`](https://github.com/toshiki670/dotfiles/commit/336fe0996ec01f0d71283c9426651ed2d167ba16))


## [0.69.0] - 2026-06-16

Rust / Cargo workspace 化のブートストラップリリース（epic #388）。シェル関数群を Rust バイナリへ移行し、release-plz による per-package バージョニングを確立した。以降のフォーマットは release-plz（git-cliff）に統一される。

### Features

- 各コマンドを Rust バイナリ化し、`cargo install` × chezmoi で配布（color / copy-obj / v-sync / gh-clone / gcm / git-upstream、背景 worker の daily-check / git-background-fetch）
- lint/format オーケストレータを Rust 化（rumdl / mise でツール供給）
- Rust edition を 2024 に更新

### Refactor

- 単一利用モジュールを lib から各 bin 配下へ移動

### Build

- リリースを release-please から release-plz（git_only・per-package タグ）へ移行
- Nix を撤去し lint を cargo + mise に一本化

## [0.68.0](https://github.com/toshiki670/dotfiles/compare/v0.67.0...v0.68.0) (2026-06-10)


### Features

* **fish:** bat + fzf のファイル選択ショートカット bf を追加 ([#377](https://github.com/toshiki670/dotfiles/issues/377)) ([3443a13](https://github.com/toshiki670/dotfiles/commit/3443a13c81fc3917cdd80becf304e3ccbf39b2f0))


### Bug Fixes

* **claude:** use rtk hook claude instead of legacy rtk-rewrite.sh ([#374](https://github.com/toshiki670/dotfiles/issues/374)) ([6760f92](https://github.com/toshiki670/dotfiles/commit/6760f927475b4b40d999cac737ef23f188351b9d))
* **fish:** gcm でフェンス付き/単一オブジェクトの JSON 応答を許容 ([#376](https://github.com/toshiki670/dotfiles/issues/376)) ([5bb5b18](https://github.com/toshiki670/dotfiles/commit/5bb5b182db650d778349832ae023001d8bfddb78))

## [0.67.0](https://github.com/toshiki670/dotfiles/compare/v0.66.1...v0.67.0) (2026-06-03)


### Features

* **fish:** nvim プラグインを同期する v-sync 関数を追加 ([#372](https://github.com/toshiki670/dotfiles/issues/372)) ([2aae33f](https://github.com/toshiki670/dotfiles/commit/2aae33fd906a64307e7d864a6d05e7ba81bc0b5f))

## [0.66.1](https://github.com/toshiki670/dotfiles/compare/v0.66.0...v0.66.1) (2026-06-03)


### Bug Fixes

* **fish:** gcm のプロンプトでコードフェンス出力を抑止する ([#366](https://github.com/toshiki670/dotfiles/issues/366)) ([6b66039](https://github.com/toshiki670/dotfiles/commit/6b66039ebd2c3310e1d6958e6db7574823ee6013))

## [0.66.0](https://github.com/toshiki670/dotfiles/compare/v0.65.0...v0.66.0) (2026-06-03)


### Features

* **fish:** gh targeted commands に Fish ネイティブ補完を追加 (alternative) ([#363](https://github.com/toshiki670/dotfiles/issues/363)) ([062a415](https://github.com/toshiki670/dotfiles/commit/062a415e564282427fcbd8dad0affa4c60817a67))
* **ghostty:** カーソルを常に block 表示にする ([#365](https://github.com/toshiki670/dotfiles/issues/365)) ([8b556b4](https://github.com/toshiki670/dotfiles/commit/8b556b475daf9d1ed343ecdb05465b1157822e53))

## [0.65.0](https://github.com/toshiki670/dotfiles/compare/v0.64.0...v0.65.0) (2026-06-02)


### Features

* **fish:** add AI-powered gcm for Conventional Commits ([#358](https://github.com/toshiki670/dotfiles/issues/358)) ([59ab0eb](https://github.com/toshiki670/dotfiles/commit/59ab0ebbb4b61093d2f5a352ef9a623e2b57d86b))


### Code Refactoring

* **fish:** gh abbreviation をグローバル略語に変更 ([#360](https://github.com/toshiki670/dotfiles/issues/360)) ([f89131f](https://github.com/toshiki670/dotfiles/commit/f89131fdc759b5ade1c2ec04a0b56163a081b178))

## [0.64.0](https://github.com/toshiki670/dotfiles/compare/v0.63.0...v0.64.0) (2026-06-02)


### Features

* Fish でディレクトリ移動後に自動で ls (eza) する ([#355](https://github.com/toshiki670/dotfiles/issues/355)) ([59918b5](https://github.com/toshiki670/dotfiles/commit/59918b59706dbad84299568e814bcd96f1113119))
* fzf のカラーをシステム外観追従（light/dark）に対応 ([#356](https://github.com/toshiki670/dotfiles/issues/356)) ([f573a97](https://github.com/toshiki670/dotfiles/commit/f573a976894a60a83c79f54a7b6891ec4740d6ab))
* カラーテーマをシステム外観追従（light:One Half Light / dark:Ayu）に統一 ([#353](https://github.com/toshiki670/dotfiles/issues/353)) ([e70c7dc](https://github.com/toshiki670/dotfiles/commit/e70c7dc872c3405347d2eec451755354544f38ba))


### Code Refactoring

* fzf fish 関数を実態に合わせてリネーム ([#357](https://github.com/toshiki670/dotfiles/issues/357)) ([1828842](https://github.com/toshiki670/dotfiles/commit/1828842af321c33b48902faac403710927772d58))

## [0.63.0](https://github.com/toshiki670/dotfiles/compare/v0.62.3...v0.63.0) (2026-05-31)


### Features

* ghq/worktree ピッカーを統合し WT 削除ショートカットを追加 ([#349](https://github.com/toshiki670/dotfiles/issues/349)) ([5378ac9](https://github.com/toshiki670/dotfiles/commit/5378ac9e1db1980b3e2801167de66a247558308f))


### Bug Fixes

* nvim checkhealth の警告・エラーを解消しプラグインを更新 ([#351](https://github.com/toshiki670/dotfiles/issues/351)) ([f68cd71](https://github.com/toshiki670/dotfiles/commit/f68cd711d7109308e9dff6df4d63d0120b9d942f))

## [0.62.3](https://github.com/toshiki670/dotfiles/compare/v0.62.2...v0.62.3) (2026-05-30)


### Bug Fixes

* fzf キャンセル時にプロンプトが消える問題を修正 ([#346](https://github.com/toshiki670/dotfiles/issues/346)) ([f19a12a](https://github.com/toshiki670/dotfiles/commit/f19a12a33adb39c1439dff09b7b97a8f40156a79))

## [0.62.2](https://github.com/toshiki670/dotfiles/compare/v0.62.1...v0.62.2) (2026-05-26)


### Bug Fixes

* mise モジュールを無効化して Starship プロンプト呼び出しを削減 ([#344](https://github.com/toshiki670/dotfiles/issues/344)) ([85af381](https://github.com/toshiki670/dotfiles/commit/85af381c31b4f1ce176fafe8244a147eec04acc2))

## [0.62.1](https://github.com/toshiki670/dotfiles/compare/v0.62.0...v0.62.1) (2026-05-23)


### Bug Fixes

* mise の二重 activate を解消し fish 起動を高速化 ([#342](https://github.com/toshiki670/dotfiles/issues/342)) ([d96a43d](https://github.com/toshiki670/dotfiles/commit/d96a43d917ad7a1c2c227b1a81785ea96ab7598e))

## [0.62.0](https://github.com/toshiki670/dotfiles/compare/v0.61.0...v0.62.0) (2026-05-20)


### Features

* merge-ready の format を複数 PR 対応の $pr_ids に変更 ([#339](https://github.com/toshiki670/dotfiles/issues/339)) ([557529b](https://github.com/toshiki670/dotfiles/commit/557529b2201c53da3708b1cc0326f91720d0ec48))

## [0.61.0](https://github.com/toshiki670/dotfiles/compare/v0.60.0...v0.61.0) (2026-05-20)


### Features

* ~/.claude/settings.json を modify_ script 化してローカル編集値を保持 ([#336](https://github.com/toshiki670/dotfiles/issues/336)) ([#337](https://github.com/toshiki670/dotfiles/issues/337)) ([1f5043f](https://github.com/toshiki670/dotfiles/commit/1f5043f81dbc8fdb65d34f65d81fa4ea582c3a59))

## [0.60.0](https://github.com/toshiki670/dotfiles/compare/v0.59.0...v0.60.0) (2026-05-17)


### Features

* bin に cleanup-env コマンドを追加 ([#334](https://github.com/toshiki670/dotfiles/issues/334)) ([6fb8f35](https://github.com/toshiki670/dotfiles/commit/6fb8f35a98d268e12a21750705c0cfe7695e6af0))

## [0.59.0](https://github.com/toshiki670/dotfiles/compare/v0.58.2...v0.59.0) (2026-05-17)


### Features

* starship の gcloud に account/domain/region 表示の format を追加 ([#332](https://github.com/toshiki670/dotfiles/issues/332)) ([781c3d4](https://github.com/toshiki670/dotfiles/commit/781c3d4b65d3c54fb0edc9f8bce0609e94af73f9))

## [0.58.2](https://github.com/toshiki670/dotfiles/compare/v0.58.1...v0.58.2) (2026-05-17)


### Code Refactoring

* ホスト側削除完了後の .chezmoiremove.tmpl を撤去 ([#270](https://github.com/toshiki670/dotfiles/issues/270), [#325](https://github.com/toshiki670/dotfiles/issues/325)) ([#328](https://github.com/toshiki670/dotfiles/issues/328)) ([f7d39f0](https://github.com/toshiki670/dotfiles/commit/f7d39f06a9b338a52868f29ee1c929747a9f9ee3))

## [0.58.1](https://github.com/toshiki670/dotfiles/compare/v0.58.0...v0.58.1) (2026-05-17)


### Code Refactoring

* Zsh 専用設定 (sheldon/zeno) を chezmoi 管理から外す (1/2) ([#326](https://github.com/toshiki670/dotfiles/issues/326)) ([bbd9fca](https://github.com/toshiki670/dotfiles/commit/bbd9fca1ad8503eb0097e374c506e423a82e914f))
* Zsh 設定を chezmoi 管理から外しホスト側削除を予告 (1/2) ([#323](https://github.com/toshiki670/dotfiles/issues/323)) ([06a8620](https://github.com/toshiki670/dotfiles/commit/06a862058be991128557b5c23154ebf18de44ca3))

## [0.58.0](https://github.com/toshiki670/dotfiles/compare/v0.57.0...v0.58.0) (2026-05-17)


### Features

* Ctrl-j Ctrl-w で worktree を fzf 切り替えできるようにする ([#318](https://github.com/toshiki670/dotfiles/issues/318)) ([#319](https://github.com/toshiki670/dotfiles/issues/319)) ([af43854](https://github.com/toshiki670/dotfiles/commit/af4385463dd4e13943015946ad7c23dbd68fde1f))
* Fish に ps-grep 関数を追加 ([#322](https://github.com/toshiki670/dotfiles/issues/322)) ([dbdfef9](https://github.com/toshiki670/dotfiles/commit/dbdfef9d3e61b3e10e455e66f6d0b87549385b87))

## [0.57.0](https://github.com/toshiki670/dotfiles/compare/v0.56.0...v0.57.0) (2026-05-16)


### Features

* chezmoi apply からパッケージ更新を分離し upgrade-env コマンドを追加 ([#317](https://github.com/toshiki670/dotfiles/issues/317)) ([19c2594](https://github.com/toshiki670/dotfiles/commit/19c259425ba35bba378d82d6ce358d2e92ae6e17))
* merge-ready の fish completion を追加 ([#313](https://github.com/toshiki670/dotfiles/issues/313)) ([5a2330d](https://github.com/toshiki670/dotfiles/commit/5a2330d063fcf4aceec2b03fcef15be606f95bd5))
* merge-ready の全ステータスの format に PR ID を追加 ([#314](https://github.com/toshiki670/dotfiles/issues/314)) ([a3dafaa](https://github.com/toshiki670/dotfiles/commit/a3dafaabc64b2eec5ec9c66eb7ab613be5d286b0))


### Bug Fixes

* browser-use の PATH を末尾追加にして mise Python を優先 ([#279](https://github.com/toshiki670/dotfiles/issues/279)) ([#316](https://github.com/toshiki670/dotfiles/issues/316)) ([f7f9e25](https://github.com/toshiki670/dotfiles/commit/f7f9e25053916a6f2d596109c96c7a1a171e1484))

## [0.56.0](https://github.com/toshiki670/dotfiles/compare/v0.55.0...v0.56.0) (2026-05-02)


### Features

* merge-ready の全ステータスに format と新規ステート追加 ([#312](https://github.com/toshiki670/dotfiles/issues/312)) ([cae016b](https://github.com/toshiki670/dotfiles/commit/cae016bd68c2f87e3b7b6386ff8e67077aca2b3e))


### Bug Fixes

* gpup abbr に git pull を追加 ([#310](https://github.com/toshiki670/dotfiles/issues/310)) ([d2bb535](https://github.com/toshiki670/dotfiles/commit/d2bb535d2b29efcca31f3310840493b23c5c3e17)), closes [#309](https://github.com/toshiki670/dotfiles/issues/309)

## [0.55.0](https://github.com/toshiki670/dotfiles/compare/v0.54.2...v0.55.0) (2026-04-29)


### Features

* add gpup abbr for gh pr update-branch ([#303](https://github.com/toshiki670/dotfiles/issues/303)) ([2a3abad](https://github.com/toshiki670/dotfiles/commit/2a3abadeff0cdcc8e6d331aa7651762e1ada6d30))
* replace manual release flow with release-please automation ([#304](https://github.com/toshiki670/dotfiles/issues/304)) ([cb59ddf](https://github.com/toshiki670/dotfiles/commit/cb59ddfda4e4d01ea8bcfe2e2aa761208ea02510))


### Bug Fixes

* add workflow_dispatch trigger to release-please workflow ([#306](https://github.com/toshiki670/dotfiles/issues/306)) ([875bedf](https://github.com/toshiki670/dotfiles/commit/875bedf769334e17cfe35d89b0cc850ce3bfb220))
* prepend Homebrew to fish PATH ([#301](https://github.com/toshiki670/dotfiles/issues/301)) ([04ed35c](https://github.com/toshiki670/dotfiles/commit/04ed35c1662d9f6a46c2c8e10cf95dcea3fc8161))
* use PAT token for release-please to trigger CI on release PRs ([#307](https://github.com/toshiki670/dotfiles/issues/307)) ([69c8e7f](https://github.com/toshiki670/dotfiles/commit/69c8e7f03b1a6006ef686f19aca9ec9a0542b1fd))
