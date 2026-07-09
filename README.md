# AI 翻译 / AI Translate

> 轻量、快速、可配置的桌面 AI 翻译工具。<br>
> A lightweight, fast, and configurable AI-powered desktop translator.

![AI 翻译主界面](docs/images/main-window.png)

AI 翻译专注于一个清晰目标：让桌面翻译足够快、足够安静、足够顺手。它基于 Tauri v2 构建，使用 Rust 处理全局快捷键、剪贴板、窗口控制、配置持久化与翻译请求，前端只负责极简交互界面。

AI Translate is built for one focused job: make desktop translation fast, quiet, and effortless. It is powered by Tauri v2, with Rust handling global shortcuts, clipboard access, window control, persistence, and translation requests, while the frontend stays minimal and responsive.

## Highlights / 产品亮点

- **划词即译 / Translate selected text**<br>
  在任意应用中选中文字，按下 `Alt+D`，翻译结果会在鼠标附近的悬浮窗中出现。<br>
  Select text in any app, press `Alt+D`, and get the translation in a floating popup near your cursor.

- **输入即译 / Type and translate automatically**<br>
  主界面支持输入或粘贴文本后自动翻译，也保留手动翻译按钮。<br>
  The main window translates automatically as you type or paste, with manual translation still available.

- **AI Provider 可配置 / Configurable AI providers**<br>
  内置 MyMemory 与 DeepSeek，支持添加 OpenAI 兼容服务。DeepSeek 只需要填写 API Key。<br>
  Ships with MyMemory and DeepSeek, and supports custom OpenAI-compatible providers. DeepSeek only requires an API key.

- **轻量桌面体验 / Lightweight desktop experience**<br>
  支持托盘驻留、启动后隐藏、单例运行、全局快捷键和无边框窗口。<br>
  Supports tray mode, hidden startup, single-instance behavior, global shortcuts, and frameless windows.

- **清晰反馈 / Clear request feedback**<br>
  翻译请求期间会显示 loading 状态，避免用户不知道请求是否已开始。<br>
  Translation requests show a visible loading state, so users know the app is working.

- **配置持久化 / Persistent settings**<br>
  快捷键、启动方式、主题和翻译服务会保存到本地；API Key 与普通配置分离保存。<br>
  Shortcuts, startup behavior, theme, and provider settings are persisted locally; API keys are stored separately from regular config.

## Download / 下载

Windows installer:

[AI-Translate-1.0.1-x64-setup.exe](release/AI-Translate-1.0.1-x64-setup.exe)

Local build output:

```text
src-tauri/target/release/bundle/nsis/
```

## Product Flow / 使用流程

### Selection Translation / 划词翻译

1. Select text in any desktop app.<br>
   在任意桌面应用中选中文字。
2. Press `Alt+D`.<br>
   按下 `Alt+D`。
3. AI Translate copies the selection, reads the clipboard, gets the cursor position, and opens a popup nearby.<br>
   AI 翻译会模拟复制、读取剪贴板、获取鼠标位置，并在附近打开悬浮窗。
4. The popup first shows a loading state, then updates with the translation result.<br>
   悬浮窗先显示翻译中状态，随后更新译文。

### Main Window Translation / 主界面翻译

1. Open the main window with the tray menu or `Ctrl+D`.<br>
   通过托盘菜单或 `Ctrl+D` 打开主界面。
2. Paste or type text into the source panel.<br>
   在原文区域粘贴或输入文本。
3. Translation starts automatically, or you can click the translate button.<br>
   应用会自动翻译，也可以点击立即翻译。
4. Copy the translated text with one click.<br>
   可一键复制译文。

## Translation Providers / 翻译服务

### MyMemory

中文：默认公共接口，无需 API Key，适合开箱即用和基础测试。<br>
English: The default public provider. No API key is required, making it suitable for quick testing and out-of-the-box use.

### DeepSeek

中文：DeepSeek 使用 OpenAI 兼容接口。界面中只需要填写 API Key，其余配置已内置。<br>
English: DeepSeek uses an OpenAI-compatible API. Only the API key is required in the UI; the rest is preconfigured.

```text
Base URL: https://api.deepseek.com
Model: deepseek-v4-flash
Endpoint: /chat/completions
```

The built-in translation agent asks the model to return only the translated text, without explanations, alternatives, quotes, or Markdown wrappers.

内置 Agent 会约束模型只输出译文，不输出解释、候选项、引用或 Markdown 包裹。

### Custom OpenAI-Compatible Provider / 自定义 OpenAI 兼容服务

Required fields / 必填项：

- Name / 名称
- Base URL
- Model
- API Key

## Shortcuts / 快捷键

Default shortcuts / 默认快捷键：

- Selection translation / 划词翻译：`Alt+D`
- Show main window / 打开主界面：`Ctrl+D`

Custom shortcut format / 自定义快捷键格式：

```text
Alt/Ctrl/Shift + letter or number
```

Examples / 示例：

```text
Alt+D
Ctrl+E
Shift+Q
```

## Settings / 设置能力

- Startup mode: open main window or stay in tray.<br>
  启动方式：打开主界面或仅驻留托盘。
- Theme: dark and light themes.<br>
  主题：深色和浅色。
- Provider: switch provider directly from the home screen.<br>
  翻译服务：首页直接切换。
- Shortcuts: customize selection translation and main-window shortcuts.<br>
  快捷键：自定义划词翻译和打开主界面快捷键。
- Auto translation: can be turned on or off from the source panel.<br>
  自动翻译：可在原文区域开启或关闭。

## Architecture / 技术架构

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

## Tech Stack / 技术栈

- Tauri v2
- Rust
- Vite
- Vanilla JavaScript
- CSS
- NSIS for Windows installer

## Local Development / 本地开发

Requirements / 环境要求：

- Node.js 22+
- pnpm
- Rust stable
- Windows WebView2 Runtime

Install dependencies / 安装依赖：

```powershell
pnpm install
```

Run desktop dev build / 启动桌面开发版：

```powershell
pnpm desktop
```

Build frontend / 前端生产构建：

```powershell
pnpm build
```

Rust checks / Rust 检查：

```powershell
cd src-tauri
cargo check
cargo clippy --all-targets -- -D warnings
```

Build Windows installer / 打包 Windows 安装包：

```powershell
pnpm tauri build
```

## Project Structure / 目录结构

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

## Data & Security / 数据与安全

- Regular settings and secrets are stored separately.<br>
  普通配置与密钥分离存储。
- API keys are not written into README, source code, logs, or frontend persistent state.<br>
  API Key 不写入 README、源码、日志或前端持久化状态。
- The frontend only receives whether an API key is configured.<br>
  前端只接收 API Key 是否已配置的状态。
- Translation requests are sent from the Rust backend to avoid browser CORS issues.<br>
  翻译请求由 Rust 后端发起，避免前端跨域问题。
- On Windows, API keys are protected with system-level secret storage and DPAPI fallback.<br>
  在 Windows 上，API Key 使用系统级密钥存储和 DPAPI 兜底保护。

## Release Checklist / 发布检查

```powershell
pnpm build
cd src-tauri
cargo check
cargo clippy --all-targets -- -D warnings
cd ..
pnpm tauri build
```

## License / 许可证

This project has not declared a license yet.<br>
本项目暂未声明许可证。
