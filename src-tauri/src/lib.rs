#[cfg(windows)]
mod downloads;
mod errors;
mod navigation;

use navigation::{destination, Destination};
use tauri::{webview::NewWindowResponse, Manager, WebviewUrl, WebviewWindowBuilder};

const LINK_INTERCEPTOR: &str = include_str!("links.js");

// A blob belongs to the WebView's session. Only that WebView can download it;
// handing its URL to the OS or another browser cannot transfer the file.
fn download_blob(app: &tauri::AppHandle, url: &tauri::Url) {
    if let Some(window) = app.get_webview_window("main") {
        let url = serde_json::to_string(url.as_str()).expect("URL is serializable");
        let script = format!(
            r#"(() => {{
                const link = document.createElement('a');
                link.href = {url};
                link.download = '';
                document.documentElement.appendChild(link);
                link.click();
                link.remove();
            }})();"#
        );
        if let Err(error) = window.eval(&script) {
            errors::report(
                "Could not start the attachment download. Please try again.",
                error,
            );
        }
    }
}

fn open_external(url: &tauri::Url) {
    let url = url.to_string();
    std::thread::spawn(move || {
        // Use only the Rust API. Registering the opener plugin also injects a
        // click handler that cancels HTTP download links with target="_blank".
        if let Err(error) = tauri_plugin_opener::open_url(&url, None::<&str>) {
            errors::report(
                "Could not open the link. Please try opening it in your browser.",
                error,
            );
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            errors::init(app.path().app_log_dir());
            let navigation_handle = app.handle().clone();
            let popup_handle = app.handle().clone();

            let window = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External("https://www.messenger.com/".parse().unwrap()),
            )
            .title("Messenger")
            .inner_size(1280.0, 800.0)
            .min_inner_size(800.0, 600.0)
            .center()
            .resizable(true)
            .initialization_script(LINK_INTERCEPTOR)
            .on_navigation(move |url| match destination(url) {
                Destination::Internal => true,
                Destination::Blob => {
                    download_blob(&navigation_handle, url);
                    false
                }
                Destination::External => {
                    open_external(url);
                    false
                }
                Destination::Blocked => false,
            })
            // Cover new-window requests that bypass the injected script too.
            .on_new_window(move |url, _features| {
                match destination(&url) {
                    Destination::Internal => {
                        if let Some(window) = popup_handle.get_webview_window("main") {
                            if let Err(error) = window.navigate(url) {
                                errors::report("Could not open the Messenger page. Please try again.", error);
                            }
                        }
                    }
                    Destination::Blob => download_blob(&popup_handle, &url),
                    Destination::External => open_external(&url),
                    Destination::Blocked => {}
                }
                NewWindowResponse::Deny
            })
            .build()?;

            #[cfg(windows)]
            if let Err(error) = window.with_webview(|webview| {
                if let Err(error) = downloads::enable_native_ui(webview) {
                    errors::report("The download panel is unavailable. Restart Messenger before downloading attachments.", error);
                }
            }) {
                errors::report("The download panel is unavailable. Restart Messenger before downloading attachments.", error);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
