// utils.rs
use console::{style, Term};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ProgressBar {
    inner: Arc<Mutex<ProgressBarInner>>,
}

struct ProgressBarInner {
    total: u64,
    current: u64,
    term: Term,
}

impl ProgressBar {
    pub fn new(total: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ProgressBarInner {
                total,
                current: 0,
                term: Term::stdout(),
            })),
        }
    }

    // This method will increment the current progress
    pub fn inc(&self, amount: u64) {
        let mut inner = self.inner.lock().unwrap();
        inner.current += amount; // Increment the current value
        self.draw_bar(&mut inner);
    }

    fn draw_bar(&self, inner: &mut std::sync::MutexGuard<'_, ProgressBarInner>) {
        if inner.total == 0 {
            return; // Avoid division by zero and drawing for unknown total
        }
        let percent = (inner.current as f64 / inner.total as f64) * 100.0;
        let filled = (percent / 2.5) as usize; // 40 chars bar
        let bar = format!(
            "{}{}",
            style("#".repeat(filled)).green(),
            style("-".repeat(40 - filled)).blue()
        );
        let msg = format!("\r[{}] {:>6.2}%", bar, percent);
        let _ = inner.term.write_str(&msg);
    }

    pub fn finish(&self) {
        let inner = self.inner.lock().unwrap(); // Lock for final message
        let _ = inner.term.write_str("\nDownload complete!\n");
    }
}

pub fn progress_bar(total: u64) -> ProgressBar {
    ProgressBar::new(total)
}