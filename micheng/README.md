# Micheng Codex Fork

这个目录记录 `MichengLiang/codex` fork 的本地事实来源、维护入口和专属设计资料。

## 定位

这个仓库是从 `openai/codex` fork 出来的个人改造版本。它不替代官方仓库；它在持续跟进官方 `main` 的基础上，维护个人需要的功能、实验和定制行为。

官方代码是上游基线。个人改动在个人分支上维护，避免把官方同步和个人开发混在同一个分支职责里。

## 远端

当前仓库使用两个 remote：

```text
upstream -> https://github.com/openai/codex.git
origin   -> git@github.com:MichengLiang/codex.git
```

`upstream` 表示官方仓库，只用于拉取官方更新。它的 push 地址被设置为 `DISABLED`，用于降低误推官方仓库的风险。

`origin` 表示个人 fork，用于保存个人分支和个人改动。

## 分支角色

当前维护结构包含这些分支角色：

```text
main                      -> 个人 fork 主线
custom/micheng            -> 个人定制基线或长期汇总分支
feature/mcp-text-contract -> MCP Text Contract 功能维护分支
custom/* 或 feature/*     -> 按主题维护的个人功能分支
```

`main` 是个人 fork 的主线，保存已经开放完成并适合继续日常使用的个人改动。同步官方时仍以 `upstream/main` 为上游基线，但不要把未完成主题开发直接堆在 `main` 上。

`custom/micheng` 保存个人定制方向的基线和汇总入口。

`feature/mcp-text-contract` 是 MCP Text Contract 的历史功能分支。功能开放完成后已经合并进 `origin/main`，后续可按需保留或删除远端分支引用。

## 文档索引

具体操作纪律见 [CONTRIBUTING.md](CONTRIBUTING.md)。README 只记录仓库结构和事实入口，不重复定义同步、提交、验证和冲突处理流程。

MCP Text Contract 的专属资料位于 [Codex MCP Text Contract/](<Codex MCP Text Contract/>)：

```text
Codex MCP Text Contract 设计文档.md -> 功能契约和设计边界
implementation-status.md            -> 实现状态和当前覆盖面
rmcp-counter-reference.md           -> rmcp 参考实验记录
```

其中设计文档是 MCP Text Contract 行为边界的首要事实来源。贡献指南只规定维护这些行为时的工作纪律。

本机 fork 使用笔记：

```text
local-release-build.md -> 本地日用 release 构建、codex-micheng 安装名和 target 清理约定
```

后续功能设计资料：

```text
Codex 原生文件阅读与文件上下文展开工具规格.md -> 原生 read_file 与 load_files_context 规格
关于上下文调试器.md                         -> Runtime Context Control Plane 设计草案
```

## 当前基线

MCP Text Contract 合并到 fork 主线后，当前 `origin/main` 顶端为：

```text
b245f5f2c7 32
```

该提交包含已经开放完成的 MCP Text Contract 功能分支状态。后续同步官方时，以 `upstream/main` 的实际提交为准。
