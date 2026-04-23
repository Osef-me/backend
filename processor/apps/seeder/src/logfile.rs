use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::state::LogType;

pub struct LogFile {
    path: PathBuf,
    inner: Mutex<BufWriter<File>>,
}

static GLOBAL: OnceLock<LogFile> = OnceLock::new();

pub fn init(dir: &Path) -> std::io::Result<&'static LogFile> {
    fs::create_dir_all(dir)?;
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let path = dir.join(format!("seeder-{ts}.log"));
    let file = OpenOptions::new().create(true).append(true).open(&path)?;
    let lf = LogFile {
        path,
        inner: Mutex::new(BufWriter::new(file)),
    };
    let _ = GLOBAL.set(lf);
    Ok(GLOBAL.get().expect("global logfile just set"))
}

pub fn global() -> Option<&'static LogFile> {
    GLOBAL.get()
}

impl LogFile {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn write(&self, tag: &str, message: &str) {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        if let Ok(mut w) = self.inner.lock() {
            let _ = writeln!(w, "[{now}] [{tag}] {message}");
            let _ = w.flush();
        }
    }
}

pub fn log_state_entry(kind: &LogType, message: &str) {
    if let Some(lf) = global() {
        lf.write(tag_for(kind), message);
    }
}

fn tag_for(kind: &LogType) -> &'static str {
    match kind {
        LogType::Network => "NET",
        LogType::Db => "DB",
        LogType::Calc => "CALC",
        LogType::Info => "INFO",
        LogType::Retry => "RETRY",
    }
}

pub fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "<unknown>".into());
        let payload = payload_to_string(info.payload());
        let thread = std::thread::current()
            .name()
            .map(String::from)
            .unwrap_or_else(|| format!("{:?}", std::thread::current().id()));
        let bt = std::backtrace::Backtrace::force_capture();
        let msg = format!(
            "thread '{thread}' panicked at {location}: {payload}\nbacktrace:\n{bt}"
        );
        if let Some(lf) = global() {
            lf.write("PANIC", &msg);
        }
        default(info);
    }));
}

fn payload_to_string(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".into()
    }
}
