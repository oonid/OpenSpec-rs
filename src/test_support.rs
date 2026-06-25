use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

pub fn cwd_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct CwdReset {
    previous: PathBuf,
}

impl Drop for CwdReset {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.previous);
    }
}

pub fn with_current_dir<T>(dir: &Path, f: impl FnOnce() -> T) -> T {
    let _guard = cwd_lock().lock().unwrap_or_else(|err| err.into_inner());
    let previous = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir).unwrap();
    let _reset = CwdReset { previous };
    f()
}
