# Copiwaifu Windows 维护说明

这份说明记录 Windows 版需要长期保留的平台边界，避免后续功能把 macOS 专用实现重新混进通用路径。

## 开发环境

- Windows 10/11
- Node.js 20 或更新版本
- pnpm
- Rust stable toolchain，目标为 `x86_64-pc-windows-msvc`
- Visual Studio Build Tools，安装 C++ 桌面开发组件
- WebView2 Runtime

## 常用命令

```powershell
pnpm install
pnpm run
pnpm tauri:build:windows
```

`pnpm run` 启动 Tauri 开发环境。`pnpm tauri:build:windows` 会构建 NSIS 和 MSI 安装包。

## 平台边界

- `tauri-nspanel` 只在 macOS 注册，Windows 不应依赖它。
- 开机自启使用 `tauri-plugin-autostart` 的跨平台入口，macOS launcher 参数由插件在非 macOS 平台忽略。
- 用户目录统一通过 `src-tauri/src/platform.rs` 解析，Windows 下会按 `HOME`、`USERPROFILE`、`HOMEDRIVE` + `HOMEPATH` 的顺序兜底。
- 端口文件优先写入 `%USERPROFILE%\.copiwaifu\port`，兜底写入 `%TEMP%\copiwaifu-port`。
- hook 脚本同样读取 `os.homedir()` 和 `os.tmpdir()`，不要重新硬编码 `/tmp`。
- 写入 Codex `notify` 的 TOML 字符串必须保留反斜杠转义测试。

## 验证清单

1. `pnpm install`
2. `pnpm run sidecar:build`
3. `pnpm build`
4. `cargo test`，在 `src-tauri` 目录执行
5. `pnpm tauri:build:windows`
6. 安装生成的包，确认透明窗口、托盘菜单、设置窗口、开机自启、hook 安装/卸载可用

