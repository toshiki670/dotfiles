---
name: deps-changelog-scout
description: "Dependabot 等が作成した依存更新 PR(`label:dependencies state:open`)の Changelog を確認し、Dotfiles で活用できる新機能・そのバージョンアップに伴うバグ修正/脆弱性対応のうち、PR をマージするだけでは解決しない(コード変更やバージョン制約の引き上げなど追加対応が要る)ものが見つかれば Issue を起票する。該当が無い、または PR マージのみで解決するなら起票しない。このリポジトリ(dotfiles)限定のスキル。次のような依頼で使う: 「dependency 更新 PR の Changelog を確認して」「新機能ないか見て」「Dependabot PR に新機能あるか見て」「依存更新 PR の内容を検討して」。"
---

# deps-changelog-scout — 依存更新 PR の Changelog から追加対応の要否を判定する

依存更新 PR の Changelog を確認し、その PR をマージするだけでは解決しない追加対応(新機能の活用・バグ修正・脆弱性対応)が見つかれば Issue を起票する。見つからない、または PR マージのみで解決するなら起票しない。

## 前提

脆弱性・バグ修正は「PR マージだけで解決するなら Issue 不要」(ユーザー判断)。バージョンアップのみで足りることが多いが、常にそうとは限らないため、追加対応が要るかは都度確認する。

## 手順

1. `gh pr list --repo toshiki670/dotfiles --label dependencies --state open` で該当 PR を探す。0 件なら終了(起票なし)。
2. 該当 PR ごとに `gh pr view <number> --repo toshiki670/dotfiles --json body,title` で本文を確認する。Dependabot はグループ更新で複数依存を1 PR に束ねることがあるため、束ねられた依存ごとに Changelog(PR 本文のリリースノート引用、または依存先の CHANGELOG.md)を分けて追う。
3. 依存ごとに、以下のいずれかが見つかるか判定する。
   - Dotfiles で活用できる新機能
   - そのバージョンアップに伴うバグ修正
   - そのバージョンアップに伴う脆弱性対応
4. 見つかった依存について、PR をマージするだけで解決するか判定する。解決するなら Issue 化しない(次の依存/PR へ)。解決しない(コード変更・`Cargo.toml` のバージョン制約引き上げなど追加対応が要る)場合のみ Issue 化の対象とする。
5. Issue 化の対象が一つも無ければ終了。あれば対象ごとに(このスキルの既定として 1 finding = 1 Issue)下記の構成で本文をドラフトする。
6. `gh issue create` を実行する前に、ドラフト内容をユーザーに提示して承認を得る。Issue 起票は close・コメント投稿と同様に他者へ見える操作であり、宣言だけで進めない。
7. 承認を得たら `gh issue create --repo toshiki670/dotfiles --label <ラベル> --title ... --body ...` で起票する。ラベルは finding の種類に対応するもの(新機能なら `enhancement`、バグ修正/脆弱性対応なら `bug` 等)を選び、`gh label list --repo toshiki670/dotfiles` で実在を確認してから使う。

## Issue 本文の構成

- 背景: どの PR・依存・バージョンの Changelog に何が書かれていたか。
- 対象: このリポジトリのどのコードが対応する(または対応しうる)か。
- 実装ステップ: 番号付きの実施手順。新機能を使う場合は「`Cargo.toml` の該当依存のバージョン制約を、その機能が使えるバージョンまで引き上げる」を含める。
- 補足: 適用範囲の限定や、対応不要な部分がある場合はその理由。

## 出力

1. 検討した PR と依存の一覧、それぞれの判定結果(該当なし/PR マージのみで解決/追加対応が要る)。
2. 起票した Issue があればその URL。無ければ「起票なし」とその理由。
