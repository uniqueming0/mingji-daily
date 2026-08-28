# 铭记日常（本地桌面版）

> 以自定义记账周期为核心的个人记账软件 · 数据完全保存在本机 · 支持双形态分发（便携 zip / 安装 exe）
> 需求文档：上级目录《本地桌面版需求文档.md》V1.1

## 技术栈

- **桌面外壳**：Tauri 2（Rust）
- **前端**：Vue 3 + TypeScript + Vite
- **图表**（后续里程碑）：ECharts
- **数据库**（后续里程碑）：SQLite（tauri-plugin-sql）
- **打包**：NSIS 安装器 + 便携 zip 脚本

## 环境要求

| 依赖 | 本机状态（已检测） | 安装方式 |
| --- | --- | --- |
| Node.js ≥ 18 | ✅ v24.18.0 | — |
| Rust 工具链（MSVC） | ❌ 未安装 | 见下方「安装 Rust」 |
| MSVC C++ 构建工具 | ✅ D:\VS\visualstudio | — |
| WebView2 运行时 | ✅ 已内置 | — |
| VS Code | ✅ 1.132 | 安装下方推荐插件 |

### 安装 Rust（一次性）

方式一（winget）：
```
winget install --id Rustlang.Rustup -e
```
方式二（官方）：访问 https://rustup.rs 下载 rustup-init.exe 运行，一路默认即可。

> 安装时确认 host 为 `x86_64-pc-windows-msvc`（默认即是，因为本机已装 VS C++ 工具）。
> 装完后**重开终端**，执行 `rustc -V` 验证。

## 快速开始

```powershell
# 1. 安装前端依赖
npm install

# 2. 启动开发（自动打开桌面窗口，前端热更新）
npm run tauri dev

# 3. 双产物打包（产物在 release/ 目录）
npm run package
```

首次 `tauri dev` / 打包会下载并编译 Rust 依赖（约 5~15 分钟），属正常现象。

## VS Code 集成

1. 用 VS Code 打开本目录（`mingji-daily`）；
2. 右下角弹出提示时点击「安装推荐扩展」，或手动安装：
   - **Vue - Official**（Vue 语法与提示）
   - **rust-analyzer**（Rust 提示/跳转/报错）
   - **CodeLLDB**（vadimcn.vscode-lldb，Rust 断点调试）
   - **Even Better TOML**（Cargo.toml 高亮）
3. 调试方式：
   - **F5 一键调试**：运行「铭记日常：调试（Rust 断点 + 前端热更新）」——自动后台启动 vite（端口 1420）→ 编译 Rust → 启动并断点调试；应用窗口内**右键 →「检查元素」**可同时调试前端，前端改动即时热更新；调试结束后自动关闭 vite。
     > 原理：Tauri 2 的调试构建会**优先连接 devUrl（localhost:1420）**而不是内嵌资源，所以调试时必须同时跑 vite 开发服务器（`tauri dev` 内部也是这么做的）。
   - **纯前端开发**：运行任务 `dev: 桌面应用`（tauri dev）即可，无需 F5。
   > 说明：Tauri 官方 VS Code 扩展（tauri-vscode）已停止维护并下架，故采用社区标准方案 CodeLLDB 调试 Rust。
4. 常用任务（终端 → 运行任务）：`dev: 桌面应用`、`build: 前端 (vite)`、`package: 双产物打包`。

## 双产物打包说明

> 首次打包会联网下载 NSIS 构建工具（几分钟）；release 编译开启 LTO/strip 优化，耗时较长。
> 完整发布步骤见 `docs/发布流程.md`。

执行 `npm run package` 后，`release/` 目录产出：

| 产物 | 说明 |
| --- | --- |
| `铭记日常-便携版-v0.1.0-win-x64.zip` | 解压即用，数据保存在程序同目录 `data/` |
| `铭记日常-Setup-0.1.0.exe` | NSIS 安装向导（免管理员权限），数据在 `%APPDATA%\铭记日常\` |
| `sha256sums.txt` | 两产物的 SHA256 校验值 |

## 图标

- 开发期占位图标（账本+M）由 `scripts/make-icons.ps1` 生成，已内置于 `src-tauri/icons/`；
- 正式图标出图后，在项目根目录执行：
  ```
  npm run tauri icon 你的图标.png
  ```
  自动生成全套多尺寸图标。

## 目录结构

```
mingji-daily/
├── src/                  # Vue3 前端
├── src-tauri/            # Rust 壳（main.rs / lib.rs / tauri.conf.json / capabilities/）
├── scripts/              # package.ps1（双产物打包）、make-icons.ps1（占位图标）
├── docs/                 # 随包分发的使用说明与更新日志
├── .vscode/              # 推荐插件、任务、调试配置
└── release/              # 打包产物（npm run package 后生成，已 gitignore）
```

## 里程碑（对应需求文档第 12 章）

- [x] M0 脚手架与分发链路（本仓库）
- [x] M1 基础记账（记一笔/明细/分类/账户 + SQLite，已实现待自测）
- [x] M2 周期能力（周期规则引擎 + 周期管理页，已实现待自测）
- [x] M3 导入识别（txt/md/docx + 规则解析 + 预览确认 + 撤销，AI 兜底待后续）
- [x] M4 统计与媒体（统计页 + 照片/视频附件，已实现待自测）
- [x] M5 数据管理与打磨（备份/恢复/导出/孤儿清理/完整性检查/目录迁移，已实现待自测）
- [x] M6 打包发布（v1.0.0 双产物流水线就绪，待正式打包与朋友实测）
