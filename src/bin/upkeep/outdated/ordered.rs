//! 複数パッケージの `--explain` 解決を並行に走らせ、検出順で1件ずつ返す。
//!
//! 待ちはすべてサブプロセスの終了待ちなので、OS スレッドをそのまま待たせれば足りる。
//! 非同期ランタイムは持ち込まない。
//!
//! 完了順はばらつくが、表示は検出順に固定する。実行ごとに並びが変わると読めないため。
//! ただし全件そろうまで待つと体感が悪化するので、先頭から途切れずに揃った分だけ流す。

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;

use super::explain::{self, Explanation};
use super::package::OutdatedPackage;

/// 同時に走らせる解決の本数。
///
/// 1件の解決は最大1回 `claude -p` を呼ぶので、この値がそのまま claude の同時呼び出し数の
/// 上限になる。全体の所要時間は最も遅い1件に張り付くため、これ以上増やしてもほとんど縮まない。
const LANES: usize = 4;

/// パッケージを並行に解決し、確定した先頭から `on_ready` へ検出順で渡す。
pub fn resolve_each(
    packages: &[OutdatedPackage],
    mut on_ready: impl FnMut(&OutdatedPackage, Explanation),
) {
    let next = AtomicUsize::new(0);
    let (tx, rx) = mpsc::channel();

    thread::scope(|scope| {
        for _ in 0..LANES.min(packages.len()) {
            let tx = tx.clone();
            let next = &next;
            scope.spawn(move || {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(pkg) = packages.get(index) else {
                        break;
                    };
                    let _ = tx.send((index, explain::resolve(pkg)));
                }
            });
        }
        // 送信側の元を手放す。全 worker が終えた時点で rx が閉じ、下の for が抜ける。
        drop(tx);

        let mut buffer = InOrder::new(packages.len());
        for (index, explanation) in rx {
            for (index, explanation) in buffer.accept(index, explanation) {
                on_ready(&packages[index], explanation);
            }
        }
    });
}

/// 完了順に届く結果を、検出順へ並べ直す。
struct InOrder<T> {
    slots: Vec<Option<T>>,
    next: usize,
}

impl<T> InOrder<T> {
    fn new(len: usize) -> Self {
        Self {
            slots: (0..len).map(|_| None).collect(),
            next: 0,
        }
    }

    /// `index` 番目の結果を受け取り、先頭から途切れずに揃った分を返す。
    fn accept(&mut self, index: usize, value: T) -> Vec<(usize, T)> {
        self.slots[index] = Some(value);

        let mut ready = Vec::new();
        while let Some(value) = self.slots.get_mut(self.next).and_then(Option::take) {
            ready.push((self.next, value));
            self.next += 1;
        }
        ready
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holds_results_until_the_head_arrives() {
        let mut buffer = InOrder::new(3);
        assert!(buffer.accept(2, "c").is_empty());
        assert!(buffer.accept(1, "b").is_empty());
        assert_eq!(buffer.accept(0, "a"), vec![(0, "a"), (1, "b"), (2, "c")]);
    }

    #[test]
    fn releases_each_result_as_soon_as_its_turn_comes() {
        let mut buffer = InOrder::new(3);
        assert_eq!(buffer.accept(0, "a"), vec![(0, "a")]);
        assert!(buffer.accept(2, "c").is_empty());
        assert_eq!(buffer.accept(1, "b"), vec![(1, "b"), (2, "c")]);
    }

    #[test]
    fn single_result_is_released_immediately() {
        let mut buffer = InOrder::new(1);
        assert_eq!(buffer.accept(0, "only"), vec![(0, "only")]);
    }
}
