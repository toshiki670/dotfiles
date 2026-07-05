//! `dotfiles` 本体コマンドのエントリポイント。ロジックは [`dotfiles::cli`]。

fn main() {
    dotfiles::cli::run();
}
