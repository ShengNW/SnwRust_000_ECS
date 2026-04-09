# SNW Rust 000 ECS

一个可直接运行的 `Bevy + egui` ECS 心智系统原型：
- 记忆分层：短期记忆 / 长期记忆
- 待办分层：Urgent / Daily / LifePlan
- 规则系统：短期记忆自动衰减 + 容量上限淘汰
- 运行时 UI：新增记忆/待办、固化最强短期记忆
- 数据持久化：CSV 存盘与回读

## 目录

- `src/main.rs`：主程序
- `data/`：运行后产生 `memories.csv` / `todos.csv`
- `scripts/check.ps1`：Windows + MSVC 编译检查
- `scripts/run.ps1`：Windows + MSVC 运行发布版

## Windows 运行（按你的环境约束）

在 `PowerShell` 中执行：

```powershell
cd D:\exe\SNW_ECS
powershell -ExecutionPolicy Bypass -File .\scripts\check.ps1
powershell -ExecutionPolicy Bypass -File .\scripts\run.ps1
```

脚本默认复用：
- `CARGO_HOME = D:\exe\SNW_\cargo`
- `RUSTUP_HOME = D:\exe\SNW_\rustup`
- `toolchain = stable-x86_64-pc-windows-msvc`

## 使用说明

1. 左侧 `全局心智参数` 调整衰减强度和短期容量。
2. 在 `记忆操作` 新增短期/长期记忆。
3. 点击 `固化最强短期记忆 -> 长期记忆` 把当前最重要的短期记忆沉淀。
4. 在 `待办操作` 新增分层待办。
5. 在 `CSV 存取` 保存或回读数据。
6. 按 `~` 打开/关闭 `bevy-inspector-egui` 世界检查器。

## 后续可扩展

- 把 CSV schema 独立成单独 crate
- 增加待办进度系统和压力反馈系统
- 增加记忆关联图和检索
- 第二阶段再接 OpenXR（保持 MSVC toolchain）
