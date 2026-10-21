use tauri::webview::PlatformWebview;
use webview2_com::{
    DownloadStartingEventHandler, Microsoft::Web::WebView2::Win32::ICoreWebView2_4,
};
use windows_core::Interface;

pub(crate) fn enable_native_ui(webview: PlatformWebview) -> windows_core::Result<()> {
    // SAFETY: with_webview runs on the WebView's UI thread. WebView2 retains
    // the COM event handler until the WebView is destroyed; it captures nothing.
    unsafe {
        let core: ICoreWebView2_4 = webview.controller().CoreWebView2()?.cast()?;
        let mut token = Default::default();
        core.add_DownloadStarting(
            &DownloadStartingEventHandler::create(Box::new(|_, args| {
                if let Some(args) = args {
                    // Wry 0.55 installs a handler even without Tauri's on_download
                    // and sets Handled=true. Register AFTER build() so this runs
                    // after Wry and restores progress, errors and Open file.
                    // Leave cancellation and the suggested destination intact.
                    args.SetHandled(false)?;
                }
                Ok(())
            })),
            &mut token,
        )?;
    }
    Ok(())
}
