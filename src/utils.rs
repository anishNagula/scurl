use console::{style, Term};

pub struct ProgressBar {
    total: u64,
    term: Term,
}

impl ProgressBar {
    pub fn new(total: u64) -> Self {
        Self {
            total,
            term: Term::stdout(),
        }
    }

    pub fn update(&mut self, current: u64) {
        if self.total == 0 {
            return;
        }
        let percent = (current as f64 / self.total as f64) * 100.0;
        let filled = (percent / 2.5) as usize; // 40 chars bar
        let bar = format!(
            "{}{}",
            style("█".repeat(filled)).green(),
            style("░".repeat(40 - filled)).blue()
        );
        let msg = format!("\r[{}] {:>6.2}%", bar, percent);
        let _ = self.term.write_str(&msg);
    }

    pub fn finish(&mut self) {
        let _ = self.term.write_str("\nDownload complete!\n");
    }
}

pub fn progress_bar(total: u64) -> ProgressBar {
    ProgressBar::new(total)
}