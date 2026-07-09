# AI Translate

> A lightweight, fast, and configurable AI-powered desktop translator.

English | [简体中文](README.md)

![AI Translate main window](docs/images/main-window.png)

AI Translate is built for one focused job: make desktop translation fast, quiet, and effortless. It is powered by Tauri v2, with Rust handling global shortcuts, clipboard access, window control, persistence, and translation requests, while the frontend stays minimal and responsive.

## Highlights

- **Translate selected text**: Select text in any app, press `Alt+D`, and get the translation in a floating popup near your cursor.
- **Type and translate automatically**: The main window translates automatically as you type or paste, with manual translation still available.
- **Configurable AI providers**: Ships with MyMemory and DeepSeek, and supports custom OpenAI-compatible providers. DeepSeek only requires an API key.
- **Lightweight desktop experience**: Supports tray mode, hidden startup, single-instance behavior, global shortcuts, and frameless windows.
- **Clear request feedback**: Translation requests show a visible loading state, so users know the app is working.
- **Persistent settings**: Shortcuts, startup behavior, theme, and provider settings are persisted locally; API keys are stored separately from regular config.

## Download

Windows installer:

[AI-Translate-1.0.1-x64-setup.exe](release/AI-Translate-1.0.1-x64-setup.exe)

Local build output:

```text
src-tauri/target/release/bundle/nsis/
```

## Product Flow

### Selection Translation

1. Select text in any desktop app.
2. Press `Alt+D`.
3. AI Translate copies the selection, reads the clipboard, gets the cursor position, and opens a popup nearby.
4. The popup first shows a loading state, then updates with the translation result.

### Main Window Translation

1. Open the main window with the tray menu or `Ctrl+D`.
2. Paste or type text into the source panel.
3. Translation starts automatically, or you can click the translate button.
4. Copy the translated text with one click.

## Translation Providers

### MyMemory

The default public provider. No API key is required, making it suitable for quick testing and out-of-the-box use.

### DeepSeek

DeepSeek uses an OpenAI-compatible API. Only the API key is required in the UI; the rest is preconfigured.

```text
Base URL: https://api.deepseek.com
Model: deepseek-v4-flash
Endpoint: /chat/completions
```

The built-in translation agent asks the model to return only the translated text, without explanations, alternatives, quotes, or Markdown wrappers.

### Custom OpenAI-Compatible Provider

Required fields:

- Name
- Base URL
- Model
- API Key

## Shortcuts

Default shortcuts:

- Selection translation: `Alt+D`
- Show main window: `Ctrl+D`

Custom shortcut format:

```text
Alt/Ctrl/Shift + letter or number
```

Examples:

```text
Alt+D
Ctrl+E
Shift+Q
```

## Settings

- Startup mode: open main window or stay in tray.
- Theme: dark and light themes.
- Provider: switch provider directly from the home screen.
- Shortcuts: customize selection translation and main-window shortcuts.
- Auto translation: can be turned on or off from the source panel.

## Architecture

```text
Frontend (Vite + Vanilla JS + CSS)
  - Main window UI, automatic translation, settings, themes
  - Popup UI for selection translation

Tauri Commands
  - Bridge between frontend and Rust backend
  - Translation, provider, settings, clipboard, and window APIs

Rust Backend
  - Global shortcut registration
  - Simulated copy and clipboard reading
  - Mouse position lookup and popup placement
  - MyMemory / AI provider HTTP requests
  - App settings and API key persistence
```

## Tech Stack

- Tauri v2
- Rust
- Vite
- Vanilla JavaScript
- CSS
- NSIS for Windows installer

## Local Development

Requirements:

- Node.js 22+
- pnpm
- Rust stable
- Windows WebView2 Runtime

Install dependencies:

```powershell
pnpm install
```

Run desktop dev build:

```powershell
pnpm desktop
```

Build frontend:

```powershell
pnpm build
```

Rust checks:

```powershell
cd src-tauri
cargo check
cargo clippy --all-targets -- -D warnings
```

Build Windows installer:

```powershell
pnpm tauri build
```

## Project Structure

```text
.
├── docs/
│   └── images/                 # README screenshots
├── index.html                  # Main window
├── popup.html                  # Selection translation popup
├── release/                    # Public installer file
├── scripts/
│   └── generate_icon.py        # Transparent app icon generator
├── src/
│   ├── main.js                 # Frontend state, UI, and Tauri command calls
│   └── style.css               # Main window, popup, and theme styles
├── src-tauri/
│   ├── capabilities/           # Tauri permissions
│   ├── icons/                  # App and tray icons
│   ├── src/
│   │   ├── app_state.rs        # Settings, providers, and secret persistence
│   │   ├── lib.rs              # Tauri entry, windows, tray, and commands
│   │   ├── main.rs             # Desktop entry
│   │   ├── shortcut.rs         # Global shortcuts, copy simulation, popup placement
│   │   └── translation.rs      # Translation provider requests
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── pnpm-lock.yaml
```

## Data & Security

- Regular settings and secrets are stored separately.
- API keys are not written into README, source code, logs, or frontend persistent state.
- The frontend only receives whether an API key is configured.
- Translation requests are sent from the Rust backend to avoid browser CORS issues.
- On Windows, API keys are protected with system-level secret storage and DPAPI fallback.

## Release Checklist

```powershell
pnpm build
cd src-tauri
cargo check
cargo clippy --all-targets -- -D warnings
cd ..
pnpm tauri build
```

## License

This project has not declared a license yet.
