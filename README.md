# Messenger Desktop

A lightweight native desktop client for [Messenger](https://www.messenger.com/) built with [Tauri 2](https://tauri.app/) and Rust.

No Electron. No bloat. Just a thin native shell around the real Messenger web app.

## Features

- Loads `messenger.com` directly in a native WebView2 window
- External links (e.g. URLs shared in chats) open in your default browser — not inside the app
- `window.open()` and `target="_blank"` links are intercepted and handled correctly
- Allowed domains (Facebook CDN, Google APIs, Instagram, etc.) stay inside the app window
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

All logic lives in [`src-tauri/src/lib.rs`](src-tauri/src/lib.rs). There is no local frontend.

1. A `WebviewWindowBuilder` opens `https://www.messenger.com/` as an external URL.
2. An initialization script injected into the WebView converts `target="_blank"` anchor clicks and `window.open()` calls into same-window navigation, so Tauri's `on_navigation` hook can intercept them.
3. `on_navigation` checks the destination host against an allowlist:

   | Allowed domains |
   |---|
   | `*.messenger.com` |
   | `*.facebook.com` |
   | `*.fbcdn.net` |
   | `*.facebook.net` |
   | `*.google.com` / `*.googleapis.com` |
   | `*.instagram.com` |

   Anything outside the allowlist is opened in the system browser via `tauri-plugin-opener` and blocked from loading inside the app.

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
