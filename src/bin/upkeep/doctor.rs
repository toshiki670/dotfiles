//! `doctor`: brew / mise の健全性を診断する（`brew doctor` / `mise doctor`）。
//!
//! PATH 上に存在するパッケージマネージャだけを対象にする。問題があっても診断だけで、
//! `upkeep doctor` 自体はブロックしない（[`super::command::run`] が失敗を継続扱いする）。

use super::banner::header;
use super::command::{self, command_exists};

pub fn run() {
    header("Diagnosing Environment");

    if command_exists("brew") {
        command::run("Homebrew doctor", "brew", &["doctor"]);
    }

    if command_exists("mise") {
        command::run("mise doctor", "mise", &["doctor"]);
    }

    header("Done");
}
