# AI 翻译

> 轻量、快速、可配置的桌面 AI 翻译工具。

[English](README.en.md) | 简体中文

![AI 翻译主界面](docs/images/main-window.png)

AI 翻译专注于一个清晰目标：让桌面翻译足够快、足够安静、足够顺手。它基于 Tauri v2 构建，使用 Rust 处理全局快捷键、剪贴板、窗口控制、配置持久化与翻译请求，前端只负责极简交互界面。

## 产品亮点

- **划词即译**：在任意应用中选中文字，按下 `Alt+D`，翻译结果会在鼠标附近的悬浮窗中出现。
- **输入即译**：主界面支持输入或粘贴文本后自动翻译，也保留手动翻译按钮。
- **AI 服务可配置**：内置 MyMemory 与 DeepSeek，支持添加 OpenAI 兼容服务。DeepSeek 只需要填写 API Key。
- **轻量桌面体验**：支持托盘驻留、启动后隐藏、单例运行、全局快捷键和无边框窗口。
- **清晰反馈**：翻译请求期间会显示 loading 状态，避免用户不知道请求是否已开始。
- **配置持久化**：快捷键、启动方式、主题和翻译服务会保存到本地；API Key 与普通配置分离保存。

## 下载

Windows 安装包：

[AI-Translate-1.0.1-x64-setup.exe](release/AI-Translate-1.0.1-x64-setup.exe)

本地打包产物目录：

```text
src-tauri/target/release/bundle/nsis/
```

## 使用流程

### 划词翻译

1. 在任意桌面应用中选中文字。
2. 按下 `Alt+D`。
3. AI 翻译会模拟复制、读取剪贴板、获取鼠标位置，并在附近打开悬浮窗。
4. 悬浮窗先显示翻译中状态，随后更新译文。

### 主界面翻译

1. 通过托盘菜单或 `Ctrl+D` 打开主界面。
2. 在原文区域粘贴或输入文本。
3. 应用会自动翻译，也可以点击立即翻译。
4. 可一键复制译文。

## 翻译服务

### MyMemory

默认公共接口，无需 API Key，适合开箱即用和基础测试。

### DeepSeek

DeepSeek 使用 OpenAI 兼容接口。界面中只需要填写 API Key，其余配置已内置。

```text
Base URL: https://api.deepseek.com
Model: deepseek-v4-flash
Endpoint: /chat/completions
```

内置 Agent 会约束模型只输出译文，不输出解释、候选项、引用或 Markdown 包裹。

### 自定义 OpenAI 兼容服务

必填项：

- 名称
- Base URL
- Model
- API Key

## 快捷键

默认快捷键：

- 划词翻译：`Alt+D`
- 打开主界面：`Ctrl+D`

自定义快捷键格式：

```text
Alt/Ctrl/Shift + 字母或数字
```

示例：

```text
Alt+D
Ctrl+E
Shift+Q
```

## 设置能力

- 启动方式：打开主界面或仅驻留托盘。
- 主题：深色和浅色。
- 翻译服务：首页直接切换。
- 快捷键：自定义划词翻译和打开主界面快捷键。
- 自动翻译：可在原文区域开启或关闭。

## 技术架构

```text
Frontend (Vite + Vanilla JS + CSS)
  - 主窗口 UI、自动翻译、设置和主题
  - 划词翻译悬浮窗 UI

Tauri Commands
  - 连接前端和 Rust 后端
  - 翻译、Provider、设置、剪贴板和窗口 API

Rust Backend
  - 全局快捷键注册
  - 模拟复制并读取剪贴板
  - 获取鼠标位置并放置悬浮窗
  - MyMemory / AI Provider HTTP 请求
  - 应用配置和 API Key 持久化
```

## 技术栈

- Tauri v2
- Rust
- Vite
- Vanilla JavaScript
- CSS
- NSIS Windows 安装包

## 本地开发

环境要求：

- Node.js 22+
- pnpm
- Rust stable
- Windows WebView2 Runtime

安装依赖：

```powershell
pnpm install
```

启动桌面开发版：

```powershell
pnpm desktop
```

前端生产构建：

```powershell
pnpm build
```

Rust 检查：

```powershell
cd src-tauri
cargo check
cargo clippy --all-targets -- -D warnings
```

打包 Windows 安装包：

```powershell
pnpm tauri build
```

## 目录结构

```text
.
├── docs/
│   └── images/                 # README 截图
├── index.html                  # 主窗口
├── popup.html                  # 划词翻译悬浮窗
├── release/                    # 发布安装包
├── scripts/
│   └── generate_icon.py        # 透明应用图标生成脚本
├── src/
│   ├── main.js                 # 前端状态、UI 和 Tauri command 调用
│   └── style.css               # 主界面、悬浮窗和主题样式
├── src-tauri/
│   ├── capabilities/           # Tauri 权限
│   ├── icons/                  # 应用和托盘图标
│   ├── src/
│   │   ├── app_state.rs        # 设置、Provider 和密钥持久化
│   │   ├── lib.rs              # Tauri 入口、窗口、托盘和命令
│   │   ├── main.rs             # 桌面入口
│   │   ├── shortcut.rs         # 全局快捷键、模拟复制和悬浮窗定位
│   │   └── translation.rs      # 翻译服务请求
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── pnpm-lock.yaml
```

## 数据与安全

- 普通配置与密钥分离存储。
- API Key 不写入 README、源码、日志或前端持久化状态。
- 前端只接收 API Key 是否已配置的状态。
- 翻译请求由 Rust 后端发起，避免前端跨域问题。
- 在 Windows 上，API Key 使用系统级密钥存储和 DPAPI 兜底保护。

## 发布检查

```powershell
pnpm build
cd src-tauri
cargo check
cargo clippy --all-targets -- -D warnings
cd ..
pnpm tauri build
```

## 许可证

本项目暂未声明许可证。
