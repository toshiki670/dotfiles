//! `dotfiles` 本体（core）コマンドのエントリポイント。ロジックは [`dotfiles::core::cli`]。

fn main() {
    dotfiles::core::cli::run();
}
