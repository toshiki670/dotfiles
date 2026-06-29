//! `upgrade-env` のエントリポイント。ロジックは [`dotfiles::env_tools::upgrade_env`]。

fn main() {
    dotfiles::env_tools::upgrade_env::run();
}
