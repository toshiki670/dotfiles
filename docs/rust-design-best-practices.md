# Rust 設計ベストプラクティス

Rust で API・モジュール・型・エラーハンドリングを設計する際に参照するパターン集。複数の著者・情報源から集めた原則を、開発中にすぐ引ける形でまとめる。各主張の出典はセクション内に直接リンクし、資料の一覧は末尾にまとめる。

いずれのパターンにも適用条件や限界がある。断定文だけを鵜呑みにせず、各項目の但し書きを踏まえてプロジェクトごとに判断すること。

## 型設計: 不正な状態を表現不可能にする

複数の情報源に共通する中心的な考え方は「型はコンパイラを満足させるためではなく、不正な状態が物理的に存在できないようにする壁である」という点にある（出典: [nwiizo — 型は壁、Rustでもバグを直すな、表現できなくせよ](https://speakerdeck.com/nwiizo/xing-habi-rustdemobaguwozhi-suna-biao-xian-dekinakuseyo)）。

### newtype で意味を区別する

同じ基底型（`i32` や `String` など）を異なる意味で使うと、コンパイラは値の取り違えを検出できない。

```rust
struct CustomerId(i32);
struct OrderId(i32);
```

`CustomerId` と `OrderId` を別の型にしておけば、引数の渡し間違いはコンパイルエラーになる。

### ライフサイクルの状態ごとに型を分ける（状態分離）

「未検証」「検証済み」のようなドメイン上の状態を、同一型のフラグではなく別々の型として表現する（出典: [nwiizo](https://syu-m-5151.hatenablog.com/entry/2026/01/22/094654)）。

```rust
struct UnvalidatedOrder { /* ... */ }
struct ValidatedOrder { /* ... */ }

fn validate(order: UnvalidatedOrder) -> Result<ValidatedOrder, ValidationError> {
    // ...
}
```

所有権（ムーブセマンティクス）と組み合わせることで、検証前の値を検証後に誤って使い回すことを防ぎやすくなる（後述）。

### スマートコンストラクタで不変条件を強制する

バリデーションを行うコンストラクタ関数だけを公開し、フィールドを非公開にすることで「正当な値だけが存在できる」型を作る（出典: [Luca Palmieri — Using Types To Guarantee Domain Invariants](https://www.lpalmieri.com/posts/2020-12-11-zero-to-production-6-domain-modelling/)、"Parse, Don't Validate" 原則）。

```rust
pub struct Email(String);

impl Email {
    pub fn parse(raw: String) -> Result<Self, String> {
        if raw.contains('@') {
            Ok(Self(raw))
        } else {
            Err("invalid email".to_string())
        }
    }
}
```

上記の `contains('@')` は説明用の簡略化であり、実際のメール検証としては不十分（実務では専用クレートを使う）。また、入力値の生データをエラーメッセージへそのまま含めると PII 漏洩やログインジェクションにつながりうるため、検証失敗の理由だけを持たせる方が安全である。

この保証が成り立つのは、型の構築経路をすべて `parse` に集約できている場合に限る。`Deserialize` の derive、`Default` 実装、別途公開したコンストラクタ、`unsafe` によるフィールド操作などが残っていると、不変条件はそこから迂回されうる。

### 網羅的な enum で bool / Option の組み合わせ爆発を避ける

`bool` や `Option` の組み合わせで状態を表現すると、意味のない組み合わせ（無効な状態）まで表現可能になってしまう。有効な状態だけを列挙した `enum` に置き換える（出典: [nwiizo](https://speakerdeck.com/nwiizo/xing-habi-rustdemobaguwozhi-suna-biao-xian-dekinakuseyo)）。

### Typestate パターン

型パラメータ（多くは `PhantomData`）で状態を表現し、特定の状態でしか呼べないメソッドを型システムで制限する（出典: [Microsoft RustTraining: Newtype and Type-State Patterns](https://microsoft.github.io/RustTraining/rust-patterns-book/ch03-the-newtype-and-type-state-patterns.html)）。

```rust
struct Connection<State> {
    _state: PhantomData<State>,
}
struct Disconnected;
struct Authenticated;

impl Connection<Authenticated> {
    fn send(&self, data: &[u8]) { /* ... */ }
}
```

`Connection<Disconnected>` から `send` を呼ぶことはコンパイルエラーになる。

ただし Typestate には限界もある。状態が実行時の入力によって決まる場合は表現しにくく、異なる状態の値を同じコレクションへ入れるのも難しい（結局 enum でラップする必要が出てくる）。状態数が増えるほど型と `impl` ブロックの数も増え、内部実装のための状態が公開 API に漏れやすくなる。目安として、状態遷移が小さく静的なプロトコル（接続の確立・認証など）には Typestate が向き、実行時に分岐する状態機械には enum ベースの設計の方が扱いやすいことが多い（出典: [Yoshua Wuyts — State Machines III: Type States](https://blog.yoshuawuyts.com/state-machines-3/)）。

newtype に `Deref` を実装すると、対象型のメソッドをすべて呼び出せるようになり、型安全のために隠したかったメソッドまで意図せず公開してしまう可能性がある。そのため型安全・API 制限が目的の newtype には `Deref` を実装せず、必要な変換は [`AsRef`](https://doc.rust-lang.org/std/convert/trait.AsRef.html) や専用のアクセサメソッドで明示的に提供する（出典: [Microsoft RustTraining](https://microsoft.github.io/RustTraining/rust-patterns-book/ch03-the-newtype-and-type-state-patterns.html)、[std::ops::Deref](https://doc.rust-lang.org/std/ops/trait.Deref.html)）。

### Parse, Don't Validate

`is_valid_name() -> bool` のような検証関数は、その場限りの判定結果を返すだけで再利用できない。代わりに、より構造化された型を返すパース関数を書く（出典: [Luca Palmieri](https://www.lpalmieri.com/posts/2020-12-11-zero-to-production-6-domain-modelling/)）。

```rust
fn parse_name(raw: String) -> Result<SubscriberName, String> {
    // 検証を通過した値だけが SubscriberName として存在する
}
```

検証結果そのものを型に埋め込むことで、以降のコードは「検証済みかどうか」を気にする必要がなくなる。

## 所有権を活かした設計

### ムーブセマンティクスで状態遷移を扱う

`fn validate(order: UnvalidatedOrder) -> ValidatedOrder` のようなシグネチャにすると、`UnvalidatedOrder` が `Copy` を実装しておらず事前に複製もされていない限り、渡した値は呼び出しと同時にムーブされ、以降そのバインディングは使用できなくなる（出典: [nwiizo](https://syu-m-5151.hatenablog.com/entry/2026/01/22/094654)）。

Rust が防ぐのはあくまで「消費済みの同じ値を再利用すること」であり、値の複製がどこにも残らないことまで保証するわけではない。`Clone` で複製すれば、古い状態の値は別のバインディングとして生き続けられる。

### 所有権と自動リソース解放（RAII）

所有権が保証するのは「値が非エイリアスである」ことではない。`&T` による共有参照は複数同時に存在でき、`Rc<T>` / `Arc<T>` は所有権そのものを複数の場所で共有できる。Rust が保証するのは、所有者とライフタイムが静的に追跡され、可変アクセス（`&mut T`）は常に排他的である（同時に他の参照と共存しない）ことである。

RAII が成り立つのは、この所有権とライフタイムの追跡によって「値をいつ破棄してよいか」がコンパイル時に決定できるからであり、「非エイリアスだからデストラクタを走らせられる」という単純な因果関係ではない。ハンドル型で排他アクセスを表現するのは、`&mut T` の排他性を活かした典型的な API 設計パターンである（出典: [without.boats — Ownership](https://without.boats/blog/ownership/)）。

なお [`Drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html) は必ず実行されるとは限らない。`mem::forget` による意図的な抑制、プロセスの異常終了、`panic = "abort"` 設定時の巻き戻し省略などでは呼ばれない。

## エラーハンドリング設計

### Result と panic! の使い分け

Rust は例外機構を持たず、エラーを「回復可能」（`Result<T, E>` で表現し、呼び出し側に処理を委ねる）と「回復不能」（`panic!` で呼び出しを中断する）の 2 つの方針で扱う（出典: [The Rust Book](https://doc.rust-lang.org/book/)）。ただしこれは戻り値の型に現れる分類ではない — `T` を返す関数の内部で `panic!` が起きる可能性は、シグネチャだけからは分からない。ファイルが見つからないといった予期される失敗は `Result` で扱い、配列の範囲外アクセスのようなバグの兆候には `panic!` を使う、という運用上の使い分けである。

`panic!` は必ずしもプロセス全体を即座に停止させるわけではない。デフォルトではスタックを巻き戻し（unwind）、多くの場合は現在のスレッドだけが終了する。[`catch_unwind`](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html) で捕捉することも可能で、`panic = "abort"` を設定した場合のみプロセスが即座に終了する。「回復可能か不能か」は言語が固定した分類ではなく、API 境界や呼び出し側が回復を期待できるかという設計判断である。

### エラー処理の 5 つの責務

エラーハンドリングは単一の関心事ではなく、次の 5 つの責務に分解できる（出典: [Jane Lusby — Error Handling Isn't All About Errors](https://github.com/yaahc/rustconf/blob/master/error-handling-isnt-all-about-errors.md), RustConf 2020）。

1. **定義** — 型とトレイトでエラーを表現する
2. **伝播とコンテキスト収集** — パスやバックトレースなど、エラーが発生した文脈を運ぶ
3. **特定エラーへの対応** — 個別のケースに応じて処理を分岐する
4. **意図的な破棄** — エラーを無視すると決めたことを明示する
5. **報告** — 収集した文脈とともにエラーを提示する

この 5 分解を意識すると、「thiserror と anyhow のどちらを使うべきか」を思考停止で決めずに済む。

### thiserror と anyhow の使い分け

`thiserror` はエラー型そのものを提供するクレートではなく、`enum` / `struct` に `#[derive(thiserror::Error)]` を付けることで [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html)（`Display` / `From` / `source()`）の実装を自動生成する導出マクロである。型付きエラーは `thiserror` を使わず手動で実装してもよいし、`enum` に限らず単一の失敗理由を持つ `struct` として定義してもよい。一方 `anyhow::Error` は型を消去して 1 つの型に集約するが、`downcast` / `downcast_ref` で元の具体型へ戻すこともでき、完全に不透明というわけではない。

本質的な違いは「呼び出し側に安定したエラー契約（どの variant が起こりうるか）を公開するか」と「異種のエラーを 1 つの型でまとめて報告するか」にある。判断基準としては次の 2 つを併記できる（出典: [Luca Palmieri — Error Handling In Rust](https://www.lpalmieri.com/posts/error-handling-rust/)、[0h-n0 — Qiita](https://qiita.com/0h-n0/items/36b6071417025136f2a4)）。

- **呼び出し側の要求で選ぶ**: 失敗モードごとに呼び出し側が異なる振る舞いをする必要がある → 型付きエラー（`thiserror` 等）。失敗した事実と文脈が分かれば十分 → `anyhow` / `eyre` で集約する。
- **従来からの経験則**: ライブラリは呼び出し側に契約を示すため型付きエラーを、アプリケーションは末端で人間やログに報告するだけなので `anyhow` を使う、という区分も依然として広く使われている。

ドメイン層・インフラ層で `thiserror` による型付きエラーを定義し、アプリケーション層で `anyhow` に集約する構成は、レイヤードアーキテクチャでよく見られる選択肢の一つである（唯一の正解ではない）。

`Error::source()` を実装する構造化エラーでは、エラー自身の `Display` に `source` のメッセージまで含めるべきではない。標準ライブラリの指針では「source として返すか、Display に含めるかのどちらか一方」とされており、両方に含めるとレポーター側（ログ用の 1 行整形、ターミナル用の多行整形など）でメッセージが重複する原因になる（出典: [Jane Lusby](https://github.com/yaahc/rustconf/blob/master/error-handling-isnt-all-about-errors.md)）。

### エラー情報の階層化: 内部 vs 境界

エラーに含める情報量は、読み手の立場によって変える（出典: [Luca Palmieri — Error Handling In Rust](https://www.lpalmieri.com/posts/error-handling-rust/)）。

- **オペレーター**（システム内部にアクセスできる） → 診断に必要で、かつ安全に記録できる範囲の文脈を持たせる。秘密情報・認証トークン・個人情報・巨大なペイロードはログに含めない
- **エンドユーザー**（アプリケーション境界の外側） → 自分の振る舞いを調整するために必要な最小限の情報だけ渡す

「エラー報告としてのログ」は、エラーを最終的にハンドルする箇所で一度だけ出す。`?` で単に上位へ伝播するだけの関数でログを出すと、伝播経路の途中で何重にも同じ失敗が記録されてしまう。ただしこれはエラー報告用ログに限った話であり、リトライ回数の記録・監査ログ・メトリクス・分散システムの境界を跨ぐトレースなど、別の目的を持つ記録には当てはまらない。

## API / モジュール設計

### 公開レベル（pub）は必要最小限に

`pub` を必要以上に広げないことは、「このコードは外部からの再利用を意図していない」という情報をコンパイラと他の開発者の両方に伝える手段になる。`pub(super)` などの可視性修飾子でモジュール間インターフェースを絞り込む（出典: [msakuta — Zenn](https://zenn.dev/msakuta/articles/83f9991b2aba62)）。

### 実装ごとに一意な型は関連型、呼び出し側が選ぶ型は型パラメータ

型パラメータと関連型の使い分けは、パラメータ数の多寡ではなく意味関係で決める。ある実装に対して型が一つに定まるなら関連型を、同じトレイトを異なる型の組み合わせで複数回実装させたい・呼び出し側に型を選ばせたいなら型パラメータを使う。

複数のトレイト境界を持つジェネリック構造体（`Foo<S, C, I, M, G>` のように型パラメータが増えていく状態）で、それらの型が実装ごとに一意に決まる組み合わせであるなら、関連型を束ねた 1 つのトレイト（例: `Config`）にまとめて `Foo<Cfg: Config>` へ単純化できる（出典: [Microsoft RustTraining](https://microsoft.github.io/RustTraining/rust-patterns-book/ch03-the-newtype-and-type-state-patterns.html)）。ただしこの集約には代償もある。型の組み合わせが固定されるため個別の差し替えが難しくなり、トレイト境界やエラーメッセージが複雑になりやすい。単純に型エイリアスや型推論で足りるケースまで無理に関連型へ寄せる必要はない。

### ガイドラインは強制ではなく判断材料

[Rust API Guidelines](https://rust-lang.github.io/api-guidelines/about.html) 自体が明言している通り、これらは crate 作者への推奨事項であり、絶対的なルールではない。トレードオフと採用理由を意識した上で、プロジェクトごとに適用の可否を判断する。

## 参考資料

### 公式・準公式

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/about.html) — Rust ライブラリチームによる API 設計チェックリスト
- [The Rust Book](https://doc.rust-lang.org/book/) — Steve Klabnik ほか。エラー分類など言語の設計思想そのものを解説
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/) — コミュニティによる idiom / pattern / anti-pattern 集
- 標準ライブラリドキュメント: [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html) / [`std::ops::Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html) / [`std::ops::Drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html)

### 書籍

- [Effective Rust](https://www.lurklurk.org/effective-rust/)（David Drysdale, O'Reilly） — 型・トレイト・言語コア概念を軸にした 35 の推奨事項
- [Rust for Rustaceans](https://rust-for-rustaceans.com/)（Jon Gjengset） — 大規模コードベース向けの中級者以上向け設計書
- [Rust API Type Patterns](https://willcrichton.net/rust-api-type-patterns/)（Will Crichton） — Witness / Guard / Typestate など型駆動 API 設計パターン集

### 記事

- [Rust patterns that leverage the type system](https://deepengineering.net/p/rust-patterns-that-leverage-the-type-system)（Deep Engineering, Evan Williams） — 同著者の書籍 *Design Patterns and Best Practices in Rust*（Packt）からの抜粋記事

### 個人ブログ（英語圏）

- [matklad — Newtype Index Pattern](https://matklad.github.io/2018/06/04/newtype-index-pattern.html)（Alex Kladov, rust-analyzer 開発者）
- [Luca Palmieri — Using Types To Guarantee Domain Invariants](https://www.lpalmieri.com/posts/2020-12-11-zero-to-production-6-domain-modelling/) / [Error Handling In Rust](https://www.lpalmieri.com/posts/error-handling-rust/)（*Zero To Production In Rust* 著者）
- [Yoshua Wuyts — Error Handling Survey](https://blog.yoshuawuyts.com/error-handling-survey) / [State Machines III: Type States](https://blog.yoshuawuyts.com/state-machines-3/)
- [without.boats — Ownership](https://without.boats/blog/ownership/)
- [Jane Lusby — Error Handling Isn't All About Errors](https://github.com/yaahc/rustconf/blob/master/error-handling-isnt-all-about-errors.md)（RustConf 2020）
- [Microsoft RustTraining: Newtype and Type-State Patterns](https://microsoft.github.io/RustTraining/rust-patterns-book/ch03-the-newtype-and-type-state-patterns.html)

### 日本語圏

事例・補助資料として有用だが、一般原則の根拠としては上記の公式資料・書籍を優先すること。

- **nwiizo**（株式会社スリーシェイク） — [「型は壁、Rustでもバグを直すな、表現できなくせよ」](https://speakerdeck.com/nwiizo/xing-habi-rustdemobaguwozhi-suna-biao-xian-dekinakuseyo)（登壇資料）、[はてなブログ（syu-m-5151）](https://syu-m-5151.hatenablog.com/entry/2026/01/22/094654)。型設計 4 パターン（状態分離・newtype・スマートコンストラクタ・網羅的 enum）の出典。モジュール結合度可視化ツール `cargo-coupling` の開発記事も参照
- [taiki45 — 実用Rustアプリケーション開発](https://zenn.dev/taiki45/books/pragmatic-rust-application-development)（Zenn Book） — エラーハンドリング・交換可能性（trait 抽象化）を含む実務設計書
- [msakuta — Zenn](https://zenn.dev/msakuta/articles/83f9991b2aba62) — `pub` 可視性設計論
- [0h-n0 — Qiita](https://qiita.com/0h-n0/items/36b6071417025136f2a4) — thiserror/anyhow の判断基準
- [Caddi Tech Blog](https://caddi.tech/2024/03/06/184143) — gRPC サービスにおけるエラー型設計の実例
