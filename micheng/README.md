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
main                      -> 官方 main 的本地镜像
custom/micheng            -> 个人定制基线或长期汇总分支
feature/mcp-text-contract -> MCP Text Contract 功能维护分支
custom/* 或 feature/*     -> 按主题维护的个人功能分支
```

`main` 只用于对齐官方 `upstream/main`，不承载个人功能开发。

`custom/micheng` 保存个人定制方向的基线和汇总入口。

`feature/mcp-text-contract` 是当前 MCP Text Contract 的主要维护分支。该分支在官方 `main` 之上叠加个人功能提交，并通过 `origin/feature/mcp-text-contract` 保存远端状态。

## 文档索引

具体操作纪律见 [CONTRIBUTING.md](CONTRIBUTING.md)。README 只记录仓库结构和事实入口，不重复定义同步、提交、验证和冲突处理流程。

MCP Text Contract 的专属资料位于 [Codex MCP Text Contract/](<Codex MCP Text Contract/>)：

```text
Codex MCP Text Contract 设计文档.md -> 功能契约和设计边界
implementation-status.md            -> 实现状态和当前覆盖面
rmcp-counter-reference.md           -> rmcp 参考实验记录
```

其中设计文档是 MCP Text Contract 行为边界的首要事实来源。贡献指南只规定维护这些行为时的工作纪律。

## 当前基线

整理完成时，官方基线为：

```text
35aaa5d9fc Bound websocket request sends with idle timeout (#20751)
```

当前 MCP Text Contract 功能分支在该官方基线之上维护个人提交。后续同步官方时，以 `upstream/main` 的实际提交为准。
