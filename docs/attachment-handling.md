# Link and attachment handling

## Decisions

- Route navigations by protocol, host and path. Keep Messenger and Facebook
  messaging/login routes in the app; open Facebook home, profiles, posts,
  Instagram, Google and other websites in the default browser.
- This policy only covers document navigation. CDN images, videos, scripts and
  API requests do not need to be on the navigation allowlist.
- Open external URLs using the opener crate's Rust function, without registering
  its Tauri plugin. The plugin injects a second click handler that cancels HTTP(S)
  download links with `target="_blank"` (or Ctrl/Shift clicks) and attempts IPC
  from the remote page. No opener IPC permission or global Tauri API is needed.
- A `blob:` URL identifies data owned by the current browser session. Opening
  that string in Windows or a separate browser cannot transfer the data.
- Preserve existing `<a download>` links, especially their filename. Convert
  blob links without `download` and `window.open(blob)` into download anchors
  synchronously in the original document. No fetch, base64 conversion, IPC file
  transfer, duplicate in-memory copy or early object-URL revocation is needed.
- Catch direct blob navigation and native popup requests in Rust as fallbacks;
  cancel navigation and ask the original WebView to initiate a download.
- Use WebView2's default download UI for progress, cancellation, failure status
  and opening the saved file. Keep the chat open. Do not automatically execute
  downloaded attachments. If the page supplies no filename, WebView derives one
  from the blob and its MIME type; do not assume every attachment is a PDF.

Wry 0.55.1 installs an accepting download handler by default, even without
Tauri's `on_download`. That handler sets `Handled = true` and hides the native
download UI. After building the window, `downloads.rs` registers a native
`DownloadStarting` handler that restores `Handled = false` after Wry's handler.
It preserves the cancellation state and destination while enabling WebView2's
download dialog. Keep this registration after window creation, and verify the
native UI when upgrading Wry/Tauri.

Failure to enable the panel leaves the app running and shows a Windows warning.
Link-opening, navigation and download-start errors also show a warning, with at
most one warning dialog visible at a time. Technical details are appended to
`errors.log` in Tauri's app log directory:
`%LOCALAPPDATA%\com.fabia.messenger-desktop\logs\errors.log` on Windows.
Each entry starts with a Unix timestamp in seconds. If logging fails, the app
continues and falls back to stderr. A successful download-start call only means
the request was submitted; download failures are reported by WebView2's panel.

References: [blob URLs](https://developer.mozilla.org/en-US/docs/Web/URI/Reference/Schemes/blob),
[Wry download implementation](https://github.com/tauri-apps/wry/blob/wry-v0.55.1/src/webview2/mod.rs),
[WebView2 download UI](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2downloadstartingeventargs).

## Verification

Automated checks:

```sh
npm test
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
```

The JavaScript tests exercise the injected script and propagation to page click
handlers with a simulated DOM. They record download requests, not downloaded
files, and do not execute Tauri/plugin initialization. Rust tests cover the native
URL policy and appending error records. Neither substitutes for WebView2
integration testing with a logged-in Messenger account:

1. Open `https://www.facebook.com/?locale=pl_PL` from a conversation normally,
   with Ctrl+click and with middle-click. The browser opens once; the chat stays.
2. Open a profile, a post, Instagram, Google and an `l.facebook.com` shared link.
3. Sign in from a fresh session, including 2FA/checkpoint if applicable. Verify
   return to Messenger, switching chats and Facebook `/messages/` navigation.
4. Download a real PDF attachment. Verify native download progress, correct name
   and extension, then use **Open file** and check that the PDF is readable.
5. Repeat with an image and a non-PDF document, and download the same name twice.
   Verify that files are not renamed to `.pdf` and existing files are not lost.
6. Cancel a download; test a failed/expired attachment. Verify that the download
   UI reports the outcome and the conversation remains usable.
7. Exercise blob anchor links (with/without `download`), `window.open(blob)` and
   direct `location.href = blob` in WebView devtools to cover the fallback path.
8. From a page on the same origin as a real downloadable HTTP(S) attachment,
   click an `<a href="..." download="document.pdf" target="_blank">` link.
   Verify a download starts with no opener IPC error and no external browser.
   Repeat with Ctrl/Shift clicks. This is the regression check for the removed
   plugin handler; cross-origin URLs may ignore `download` by browser policy.

Limitations: `window.open()` still returns `null`, as in the original wrapper;
flows that open an empty window and populate it later need a separate managed
popup design. A revoked blob cannot be recovered; request the attachment again.
Facebook login endpoints can change, so the scoped auth routes need validation
against the live sign-in flow. Rust/build and live-account checks require the
Rust toolchain and a running Windows WebView2 app.
