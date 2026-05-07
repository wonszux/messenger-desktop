use tauri::{WebviewUrl, WebviewWindowBuilder};

// Converts window.open and _blank link clicks into same-window navigation.
// Rust's on_navigation then decides: allow internal domains, open external ones in browser.
const LINK_INTERCEPTOR: &str = r#"
(function () {
  // Redirect _blank link clicks to same-window so on_navigation can decide.
  document.addEventListener('click', function (e) {
    var path = e.composedPath();
    for (var i = 0; i < path.length; i++) {
      var el = path[i];
      if (el.tagName === 'A' && el.target === '_blank' && el.href) {
        e.preventDefault();
        window.location.href = el.href;
        return;
      }
    }
  }, true);

  // Convert window.open to same-window navigation.
  window.open = function (url) {
    if (url) window.location.href = String(url);
    return null;
  };
})();
"#;

const ALLOWED_HOSTS: &[&str] = &[
    "messenger.com",
    "facebook.com",
    "fbcdn.net",
    "facebook.net",
    "google.com",
    "googleapis.com",
    "instagram.com",
];

fn is_allowed_host(host: &str) -> bool {
    ALLOWED_HOSTS
        .iter()
        .any(|d| host == *d || host.ends_with(&format!(".{d}")))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();

            WebviewWindowBuilder::new(
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
            .on_navigation(move |url| {
                let host = url.host_str().unwrap_or("");
                if is_allowed_host(host) {
                    return true;
                }
                let url_str = url.to_string();
                let h = handle.clone();
                std::thread::spawn(move || {
                    use tauri_plugin_opener::OpenerExt;
                    let _ = h.opener().open_url(&url_str, None::<&str>);
                });
                false
            })
            .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
