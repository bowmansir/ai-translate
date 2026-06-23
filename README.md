# AI 翻译

AI 翻译是一款基于 Tauri v2 的轻量桌面翻译工具。前端负责极简交互，Rust 后端负责全局快捷键、剪贴板、窗口控制、配置持久化和翻译请求。

## 当前能力

- 主界面输入或粘贴文本后自动翻译，也可手动点击翻译。
- 全局划词翻译：选中文字后按划词快捷键，在鼠标附近弹出悬浮翻译窗。
- 支持自定义快捷键：划词翻译快捷键、打开主界面快捷键。
- 支持启动方式：启动后打开主界面，或仅驻留托盘。
- 支持翻译服务配置：
  - MyMemory 公共接口，无需 API Key。
  - DeepSeek，内置 OpenAI 兼容接口参数，只需填写 API Key。
  - 自定义 OpenAI 兼容服务，填写名称、Base URL、Model 和 API Key。
- 支持深色、浅色主题切换。
- 支持复制译文结果。
- 支持托盘菜单、单例运行和窗口最小化/最大化/关闭。

## 技术栈

- Desktop：Tauri v2 + Rust
- Frontend：Vite + Vanilla JavaScript + CSS
- Network：Rust `reqwest`
- Key storage：系统凭据管理优先，Windows 下使用 DPAPI 加密文件作为兜底
- Global shortcut：`tauri-plugin-global-shortcut`
- Clipboard：`tauri-plugin-clipboard-manager`

## 目录结构

```text
.
├── index.html              # 主窗口页面
├── popup.html              # 划词翻译悬浮窗页面
├── src/
│   ├── main.js             # 前端交互、状态、Tauri command 调用
│   └── style.css           # 主界面、悬浮窗、主题样式
├── src-tauri/
│   ├── src/
│   │   ├── app_state.rs    # 配置、Provider、密钥持久化
│   │   ├── lib.rs          # Tauri 入口、窗口、托盘、命令
│   │   ├── shortcut.rs     # 全局快捷键、模拟复制、悬浮窗定位
│   │   └── translation.rs  # MyMemory / AI Provider 翻译请求
│   ├── icons/              # 应用与托盘图标
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── pnpm-lock.yaml
```

## 本地开发

环境要求：

- Node.js 22+
- pnpm
- Rust stable
- Windows 需要 WebView2 Runtime

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

当前 Windows 发布目标为 NSIS，产物位于：

```text
src-tauri/target/release/bundle/nsis/
```

## 配置与数据边界

应用配置保存到系统应用配置目录，例如 Windows：

```text
%APPDATA%/com.codex.lighttranslate/config.json
```

API Key 不写入普通业务配置：

- 优先保存到系统凭据管理。
- Windows 下额外写入 DPAPI 加密后的 `secrets.json` 作为兜底。
- 前端只接收 `api_key_configured` 状态，不读取明文 Key。

## 翻译服务说明

DeepSeek 当前使用 OpenAI 兼容接口：

```text
Base URL: https://api.deepseek.com
Model: deepseek-v4-flash
Endpoint: /chat/completions
```

内置 Agent 约束为只返回译文，不输出解释、候选项、引用或 Markdown 包裹。

## 快捷键注意事项

划词翻译依赖当前应用允许复制选中文本，并通过模拟 `Ctrl+C` 读取剪贴板。浏览器或部分软件可能会占用某些快捷键，例如 `Alt+D` 常用于聚焦地址栏。建议使用 `Alt+Q`、`Ctrl+E`、`Shift+Q` 这类不易冲突的组合。

## 发布前检查

推荐在发布前执行：

```powershell
pnpm build
cd src-tauri
cargo check
cargo clippy --all-targets -- -D warnings
cd ..
pnpm tauri build
```

## 已知边界

- 划词翻译对目标应用的复制权限和快捷键冲突敏感。
- 当前没有自动化端到端 UI 测试，发布前仍建议手动验证主界面翻译、划词翻译、服务配置、主题切换、托盘行为和快捷键设置。
- MSI 打包在部分 Windows 环境会受 Windows Installer / WiX ICE 校验影响，当前发布目标使用 NSIS。
