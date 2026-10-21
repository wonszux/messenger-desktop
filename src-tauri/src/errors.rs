use std::{
    fmt::Display,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();

pub(crate) fn init(directory: tauri::Result<PathBuf>) {
    let result = directory.map_err(io::Error::other).and_then(|directory| {
        fs::create_dir_all(&directory)?;
        let path = directory.join("errors.log");
        OpenOptions::new().create(true).append(true).open(&path)?;
        let _ = LOG_PATH.set(path);
        Ok(())
    });
    if let Err(error) = result {
        report(
            "Diagnostic logging is unavailable. Messenger can still be used.",
            error,
        );
    }
}

fn append_log(path: &Path, message: &str) -> io::Result<()> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let entry = format!("[{timestamp}] {message}\n");
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?
        .write_all(entry.as_bytes())
}

pub(crate) fn report(message: &str, error: impl Display) {
    let details = format!("{message} {error}");
    eprintln!("{details}");
    if let Some(path) = LOG_PATH.get() {
        if let Err(error) = append_log(path, &details) {
            eprintln!("Could not write error log: {error}");
        }
    }

    #[cfg(windows)]
    show_warning(message);
}

#[cfg(windows)]
fn show_warning(message: &str) {
    use std::sync::atomic::{AtomicBool, Ordering};
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONWARNING, MB_OK};

    // Keep repeated failures from opening a stack of dialogs; every error is logged.
    static VISIBLE: AtomicBool = AtomicBool::new(false);
    if VISIBLE.swap(true, Ordering::Relaxed) {
        return;
    }
    let text: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
    // A separate thread avoids a modal message loop inside a WebView2 callback.
    if let Err(error) = std::thread::Builder::new()
        .name("error-dialog".into())
        .spawn(move || {
            // SAFETY: both strings are terminated and live until MessageBoxW returns.
            // NULL is a supported owner for this standalone Windows dialog.
            unsafe {
                MessageBoxW(
                    std::ptr::null_mut(),
                    text.as_ptr(),
                    windows_sys::w!("Messenger"),
                    MB_OK | MB_ICONWARNING,
                );
            }
            VISIBLE.store(false, Ordering::Relaxed);
        })
    {
        VISIBLE.store(false, Ordering::Relaxed);
        eprintln!("Could not show error dialog: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_log_appends_without_losing_previous_entries() {
        let path = std::env::temp_dir().join(format!(
            "messenger-errors-{}-{}.log",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        append_log(&path, "First failure").unwrap();
        append_log(&path, "Załącznik — second failure").unwrap();
        let contents = fs::read_to_string(&path).unwrap();
        fs::remove_file(path).unwrap();
        let lines: Vec<_> = contents.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].ends_with("First failure"));
        assert!(lines[1].ends_with("Załącznik — second failure"));
    }
}
