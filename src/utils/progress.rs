use indicatif::{ProgressBar, ProgressStyle};

pub struct Spinner {
    bar: ProgressBar,
}

impl Spinner {
    pub fn new(msg: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner} {msg}")
                .unwrap(),
        );
        bar.set_message(msg.to_string());
        bar.enable_steady_tick(std::time::Duration::from_millis(100));
        Self { bar }
    }

    pub fn set_message(&self, msg: &str) {
        self.bar.set_message(msg.to_string());
    }

    pub fn succeed(self, msg: &str) {
        self.bar
            .finish_with_message(format!("{} {}", green("✓"), msg));
    }

    pub fn fail(self, msg: &str) {
        self.bar
            .finish_with_message(format!("{} {}", red("✗"), msg));
    }

    pub fn warn(self, msg: &str) {
        self.bar
            .finish_with_message(format!("{} {}", yellow("⚠"), msg));
    }

    pub fn finish(self) {
        self.bar.finish();
    }

    pub fn finish_with_message(self, msg: &str) {
        self.bar.finish_with_message(msg.to_string());
    }
}

pub struct Progress {
    bar: ProgressBar,
}

impl Progress {
    pub fn new(total: u64, msg: &str) -> Self {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "{{msg}} [{{bar:20.green/red}}] {{pos}}/{{len}} {}",
                    msg
                ))
                .unwrap()
                .progress_chars("█░ "),
        );
        bar.set_message(msg.to_string());
        Self { bar }
    }

    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    pub fn set_position(&self, pos: u64) {
        self.bar.set_position(pos);
    }

    pub fn finish(self) {
        self.bar.finish();
    }

    pub fn finish_with_message(self, msg: &str) {
        self.bar.finish_with_message(msg.to_string());
    }

    pub fn abandon(self) {
        self.bar.abandon();
    }
}

pub fn spinner(msg: &str) -> Spinner {
    Spinner::new(msg)
}

pub fn progress(total: u64, msg: &str) -> Progress {
    Progress::new(total, msg)
}

fn green(s: &str) -> String {
    format!("\x1b[32m{}\x1b[0m", s)
}

fn red(s: &str) -> String {
    format!("\x1b[31m{}\x1b[0m", s)
}

fn yellow(s: &str) -> String {
    format!("\x1b[33m{}\x1b[0m", s)
}
