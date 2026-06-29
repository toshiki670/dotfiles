//! `cleanup-env` のエントリポイント。ロジックは [`dotfiles::env_tools::cleanup_env`]。

fn main() {
    dotfiles::env_tools::cleanup_env::run();
}
