use log::{Level, LevelFilter, Log, Metadata, Record};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static HAS_ERROR: AtomicBool = AtomicBool::new(false);
static WARNING_COUNT: AtomicUsize = AtomicUsize::new(0);

pub struct Logger;

impl Logger {
    pub fn has_error() -> bool {
        HAS_ERROR.load(Ordering::Relaxed)
    }

    pub fn count_warnings() -> usize {
        WARNING_COUNT.load(Ordering::Relaxed)
    }
}

impl Logger {
    pub fn init() {
        log::set_logger(&Logger).unwrap();
        log::set_max_level(LevelFilter::Info);
    }
}

impl Log for Logger {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        match record.level() {
            Level::Warn => {
                WARNING_COUNT.fetch_add(1, Ordering::Relaxed);
            }
            Level::Error => HAS_ERROR.store(true, Ordering::Relaxed),
            lvl => return println!("[{lvl}] {}", record.args()),
        }

        eprintln!("{}", record.args())
    }

    fn flush(&self) {}
}
