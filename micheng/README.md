# Micheng Codex Fork

这个目录记录 `MichengLiang/codex` 这个 fork 的本地维护约定。

## 定位

这个仓库是从 `openai/codex` fork 出来的个人改造版本。它的目标不是替代官方仓库，而是在持续跟进官方 `main` 的基础上，维护一组个人需要的功能、实验和定制行为。

官方代码作为上游基线。个人改动放在独立分支上维护，避免把官方同步和个人开发混在同一个分支里。

## 远端

当前仓库使用两个 remote：

```text
upstream -> https://github.com/openai/codex.git
origin   -> git@github.com:MichengLiang/codex.git
```

`upstream` 表示官方仓库，只用于拉取官方更新。它的 push 地址被设置为 `DISABLED`，用于降低误推官方仓库的风险。

`origin` 表示个人 fork，用于保存个人分支和个人改动。

## 分支

当前长期保留两个分支：

```text
main           -> 官方 main 的镜像
custom/micheng -> 个人魔改分支
```

`main` 只用于对齐官方 `upstream/main`。不要在 `main` 上直接开发个人功能。

`custom/micheng` 是个人功能分支。所有个人改动、实验性功能和长期维护的定制都应该从这个分支开始。

## 日常同步节奏

更新官方代码时，先让本地和 fork 的 `main` 对齐官方：

```bash
git switch main
git fetch upstream main
git reset --hard upstream/main
git push origin main
```

然后把个人分支变基到最新 `main`：

```bash
git switch custom/micheng
git rebase main
git push --force-with-lease
```

如果 rebase 期间出现冲突，应按个人改动的意图逐个解决。解决完成后继续：

```bash
git rebase --continue
```

## 开发规则

- `main` 保持干净，只承载官方最新基线。
- 个人功能从 `custom/micheng` 或它的子分支开始。
- 小改动可以直接提交到 `custom/micheng`。
- 较大的功能可以从 `custom/micheng` 再开临时分支，完成后合回 `custom/micheng`。
- 推送 rebase 后的个人分支时使用 `--force-with-lease`，不要使用裸 `--force`。

## 临时分支命名与远端跟踪

个人临时分支使用 `custom/<topic>` 命名，并推送到 `origin`：

```bash
git switch -c custom/mcp-text-contract
git push -u origin custom/mcp-text-contract
```

如果本地 `origin` 的 fetch refspec 仅包含 `main` 与 `custom/*`，则不会维护 `origin/feature/*` 的远端跟踪分支（remote-tracking branch）。
在该配置下，即使远端已存在 `feature/*` 分支，本地也无法基于 `origin/feature/*` 展示上游跟踪信息（例如 `git status -sb` 的 `...origin/<branch>`，以及 `@{u}` 解析）。
需要跟踪 `feature/*` 时，扩展 `remote.origin.fetch` 并重新 fetch：

```bash
git config --add remote.origin.fetch '+refs/heads/feature/*:refs/remotes/origin/feature/*'
git fetch origin --prune
```

## 当前基线

整理完成时，`main` 和 `custom/micheng` 都指向同一个官方提交：

```text
35aaa5d9fc Bound websocket request sends with idle timeout (#20751)
```

这表示个人分支从一个干净的官方基线开始，后续提交才代表本 fork 的实际改动。
