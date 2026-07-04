//! `dotfiles color sample`: ANSI カラーコード（16 色 + 256 色）の確認表を出力する。
//!
//! 旧 `crates/color`（さらに遡れば `bin/color`（Python））の責務を dotfiles へ吸収したもの。
//! 出力は端末で色を確認するための表で、入力は取らない。
//!
//! 参考:
//! - <http://ascii-table.com/ansi-escape-sequences.php>
//! - <http://archive.linux.or.jp/JF/JFdocs/Bash-Prompt-HOWTO-5.html>

/// 全シーケンスを既定へ戻す ANSI リセット。
const RESET: &str = "\x1b[0m";

/// (エスケープコード, 表示名) の 16 色定義。
const COLORS: &[(&str, &str)] = &[
    ("1;37", "White       "),
    ("37", "Light Gray  "),
    ("1;30", "Gray        "),
    ("30", "Black       "),
    ("31", "Red         "),
    ("1;31", "Light Red   "),
    ("32", "Green       "),
    ("1;32", "Light Green "),
    ("33", "Brown       "),
    ("1;33", "Yellow      "),
    ("34", "Blue        "),
    ("1;34", "Light Blue  "),
    ("35", "Purple      "),
    ("1;35", "Pink        "),
    ("36", "Cyan        "),
    ("1;36", "Light Cyan  "),
];

/// 16 色（前景 × 背景）と 256 色の確認表を stdout へ出力する。
pub fn sample() {
    println!("=== 16 Colors ===");
    println!(" On White(47)     On Black(40)     On Default     Color Code");

    for (code, name) in COLORS {
        let on_white = format!("\x1b[47m\x1b[{code}m  {name}{RESET}");
        let on_black = format!("\x1b[40m\x1b[{code}m  {name}{RESET}");
        let on_default = format!("\x1b[{code}m  {name}{RESET}");
        println!("{on_white}  {on_black}  {on_default}  {code}");
    }

    println!();
    println!("=== 256 Colors ===");

    let mut line = String::new();
    for code in 0..256 {
        line.push_str(&format!("\x1b[48;5;{code}m\x1b[38;5;0m {code:03} \x1b[0m"));
        if (code + 1) % 16 == 0 {
            println!("{line}{RESET}");
            line.clear();
        }
    }
    print!("{RESET}");
}
