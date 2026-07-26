//! `outdated`: brew / mise / cargo でアップデート可能なパッケージを一覧表示する。
//!
//! `--explain` は取得できたリリースノートを [`claude::summarize`] で日本語要約する。
//! 1件の解決の流れは [`explain::resolve`]、複数件の走らせ方は [`ordered::resolve_each`]
//! を参照。

mod brew;
mod cargo;
mod claude;
mod explain;
mod mise;
mod ordered;
mod package;
mod registry;
mod release;
mod render;
mod upstream;

use super::banner::header;
use super::command::command_exists;
use package::OutdatedPackage;

pub fn run(explain: bool) {
    header("Outdated Packages");

    let mut packages: Vec<OutdatedPackage> = Vec::new();
    if command_exists("brew") {
        packages.extend(brew::detect());
    }
    if command_exists("mise") {
        packages.extend(mise::detect());
    }
    if command_exists("cargo") && command_exists("cargo-install-update") {
        packages.extend(cargo::detect());
    }

    if packages.is_empty() {
        println!("アップデート可能なものはありません");
        header("Done");
        return;
    }

    // claude 不在は一括で1回だけ警告する（per-package で警告すると同じ文言がN回出る）。
    let attempt_explain = explain && command_exists("claude");
    if explain && !attempt_explain {
        eprintln!("⚠️  claude コマンドが見つかりません。要約なしで一覧のみ表示します。");
    }

    if attempt_explain {
        let mut printed_any = false;
        ordered::resolve_each(&packages, |pkg, explanation| {
            // 解説付きは1件が複数行になる。空行を挟まないとどこまでが1パッケージか読めない。
            if printed_any {
                println!();
            }
            printed_any = true;
            println!("{}", render::format_package_line(pkg, Some(&explanation)));
        });
    } else {
        for pkg in &packages {
            println!("{}", render::format_package_line(pkg, None));
        }
    }

    header("Done");
}
