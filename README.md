# Messenger Desktop

A lightweight native desktop client for [Messenger](https://www.messenger.com/) built with [Tauri 2](https://tauri.app/) and Rust.

No Electron. No bloat. Just a thin native shell around the real Messenger web app.

## Features

- Loads `messenger.com` directly in a native WebView2 window
- External links (e.g. URLs shared in chats) open in your default browser — not inside the app
- `window.open()` and `target="_blank"` links are intercepted and handled correctly
- Messenger conversations and scoped Facebook login routes stay inside the app
- Facebook pages, Instagram, Google and other websites open in the default browser
- Blob attachments download through WebView2; its download panel can open the saved file
- Recoverable native errors show a Windows warning and are saved to an error log
- Optimized release build: LTO, stripped binary, size-optimized

## Requirements

- [Rust](https://rustup.rs/)
- [Bun](https://bun.sh/) (or npm/pnpm)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 11)

## Getting started

```bash
bun install
bun run dev       # run in development mode (Rust recompiles on change)
bun run build     # production installer
```

The installer is output to:

```
src-tauri/target/release/bundle/nsis/Messenger_1.0.0_x64-setup.exe
```

No admin rights required — installs per-user.

## How it works

There is no local frontend. The native window lives in
[`lib.rs`](src-tauri/src/lib.rs), navigation rules in
[`navigation.rs`](src-tauri/src/navigation.rs), and the injected link handler in
[`links.js`](src-tauri/src/links.js).

1. A `WebviewWindowBuilder` opens `https://www.messenger.com/` as an external URL.
2. New-tab and modified link clicks reach the same native navigation policy as
   ordinary navigation. Native popup requests also pass through that policy.
3. HTTPS Messenger pages and specific Facebook messaging/authentication paths
   stay inside the app. Other HTTP(S) pages open in the default browser.
   `mailto:` and `tel:` use their system handlers. Other protocols are blocked.
   These rules do not restrict images, scripts or API requests.
4. Existing download links retain their filenames. Blob links and
   `window.open(blob)` become downloads in the original WebView session,
   keeping the conversation open. Use **Open file** in WebView2's download panel
   to open the saved PDF or other attachment.

See [attachment handling](docs/attachment-handling.md) for design decisions,
test commands, the Windows integration checklist and known limitations.

## Tech stack

| Layer | Technology |
|---|---|
| Shell | Tauri 2 |
| Language | Rust |
| WebView | WebView2 (Windows) |
| Package manager | Bun |
| Installer | NSIS (no-admin, currentUser) |

## License

MIT
