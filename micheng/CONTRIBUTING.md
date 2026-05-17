# Micheng Codex Fork 贡献指南

本文档规定 `MichengLiang/codex` fork 的维护流程。README 只描述仓库结构和文档入口；本文件是分支、同步、提交、验证和冲突处理的单一事实来源。

## 上游规则入口

本 fork 的个人维护规则建立在官方仓库规则之上。开始代码修改、同步上游或准备提交前，先阅读官方入口：

- [官方贡献指南](../docs/contributing.md)：记录官方对外部贡献、开发流程、测试、PR 和 CLA 的要求。
- [仓库代理规则](../AGENTS.md)：记录 `codex-rs` 的 Rust 编码约定、测试要求、Bazel 注意事项和高频维护规则。

本文件只记录 `MichengLiang/codex` fork 的附加纪律。官方文档已经规定的通用贡献要求不在这里重复；当官方规则更新时，以官方文件当前内容为准。

## 基本原则

- `upstream/main` 是官方基线，只从官方仓库拉取。
- `origin` 是个人 fork，只保存个人分支和个人提交。
- `main` 是个人 fork 主线，只接收已经开放完成并适合日常使用的个人改动。
- 个人功能必须落在 `custom/*` 或 `feature/*` 分支上。
- 已推送并长期维护的个人分支优先使用 merge 同步官方更新，避免重写公开历史。
- 需要重写历史时只使用 `--force-with-lease`，不得使用裸 `--force`。

## 分支职责

```text
main                      -> 个人 fork 主线
custom/micheng            -> 个人定制基线或长期汇总分支
feature/mcp-text-contract -> MCP Text Contract 功能维护分支
custom/* 或 feature/*     -> 按主题维护的个人功能分支
```

`main` 的职责是作为个人 fork 的稳定入口。它应持续从官方 `upstream/main` 同步基线，但个人功能只有在主题分支已经验证并准备开放使用后才合入。

`custom/micheng` 用作个人定制方向的汇总入口。稳定的个人改动可以合入该分支。

`feature/mcp-text-contract` 用于维护 MCP Text Contract 的历史实现线。功能开放完成并合入 `origin/main` 后，该分支不再是日常开发入口；后续 MCP 修补可从 `main` 另起主题分支。

## 同步官方上游

同步官方更新前，先确认当前工作区没有未提交改动：

```bash
git status --short
```

更新官方引用：

```bash
git fetch upstream --prune
```

如果只需要把个人 fork 主线推进到官方最新基线：

```bash
git switch main
git merge upstream/main
git push origin main
```

如果需要丢弃 fork 主线上的个人提交并重新变成纯官方镜像，必须先明确这是有意的破坏性整理，再使用 `git reset --hard upstream/main`。

如果需要把官方更新合入个人功能分支：

```bash
git switch feature/mcp-text-contract
git merge upstream/main
```

当 `git merge upstream/main` 返回 `Already up to date.` 时，不创建空 merge commit。该结果表示当前功能分支已经包含官方最新基线。

## 个人分支与推送

个人主题分支使用 `custom/<topic>` 或 `feature/<topic>` 命名：

```bash
git switch -c feature/<topic>
git push -u origin feature/<topic>
```

如果本地 `origin` 的 fetch refspec 没有维护 `feature/*` 的远端跟踪分支，先扩展 refspec：

```bash
git config --add remote.origin.fetch '+refs/heads/feature/*:refs/remotes/origin/feature/*'
git fetch origin --prune
```

推送普通提交使用：

```bash
git push
```

只有在明确需要重写个人远端分支历史时，才使用：

```bash
git push --force-with-lease
```

## MCP Text Contract 保护清单

MCP Text Contract 的行为边界以 [Codex MCP Text Contract 设计文档.md](<Codex MCP Text Contract/Codex MCP Text Contract 设计文档.md>) 为准。同步官方或修改相关代码时，必须保护以下行为：

- `model_content_only`：模型可见输出只投影为 MCP `content[0].text`。
- `mcp_freeform`：只有用户显式配置的 direct MCP server 参与 freeform 暴露。
- freeform schema：只接受唯一 required 字符串字段 `{ freeform: string }`。Schema 和字段级 `description` / `title` 等 annotation 不改变输入形状；字段级 `description` 应进入 Responses custom tool 的工具描述。显式允许额外输入字段的 schema 不得被识别为 freeform。
- custom freeform routing：Responses custom tool call 的原始文本必须包装为 `{ "freeform": <input> }` 后再调用 MCP。
- 隐藏工具边界：deferred、code-mode-only 或未暴露给当前模型的 MCP freeform 工具不得被 custom tool call 隐式路由。
- code mode 边界：MCP freeform 工具不进入 code-mode nested tools。
- 配置兼容性：MCP server TOML 的未知字段保持运行时兼容读取；支持字段仍通过 schema 表达。

这些规则是行为约束，不是实现细节。实现可以随官方代码演进调整，但外部行为不得悄悄改变。

## 高风险文件

官方更新或本地修改触及下列文件时，应按 MCP Text Contract 保护清单逐项复核：

```text
codex-rs/config/src/mcp_types.rs
codex-rs/config/src/mcp_types_tests.rs
codex-rs/core/src/tools/context.rs
codex-rs/core/src/tools/router.rs
codex-rs/core/src/tools/router_tests.rs
codex-rs/core/tests/suite/code_mode.rs
codex-rs/core/tests/suite/rmcp_client.rs
codex-rs/tools/src/code_mode.rs
codex-rs/tools/src/code_mode_tests.rs
codex-rs/tools/src/mcp_tool.rs
codex-rs/tools/src/mcp_tool_tests.rs
codex-rs/tools/src/tool_registry_plan.rs
codex-rs/tools/src/tool_registry_plan_tests.rs
```

触及这些文件不表示变更错误；它表示必须用目标测试证明行为仍成立。

## 提交前验证

提交 MCP Text Contract 相关改动前，至少运行：

```bash
cd codex-rs
just fmt
RUST_MIN_STACK=8388608 cargo nextest run -p codex-tools exact_freeform
RUST_MIN_STACK=8388608 cargo nextest run -p codex-config mcp_text_contract
RUST_MIN_STACK=8388608 cargo nextest run -p codex-core freeform
RUST_MIN_STACK=8388608 cargo nextest run -p codex-core tools::router
```

覆盖率目标以本 fork 关心的生产文件为准。MCP Text Contract 相关生产文件应尽量保持 90% 以上行覆盖率；如果整文件覆盖率被大型共享文件拖低，应说明新增或修改分支的覆盖证据。

当前关注的覆盖率文件包括：

```text
codex-rs/tools/src/mcp_tool.rs
codex-rs/tools/src/tool_registry_plan.rs
codex-rs/tools/src/code_mode.rs
codex-rs/core/src/tools/router.rs
```

可用以下命令生成摘要：

```bash
cargo llvm-cov nextest --no-clean --json --summary-only \
  --output-path ../tmp/mcp-text-contract-coverage/codex-tools.json \
  -p codex-tools

cargo llvm-cov nextest --no-clean --json --summary-only \
  --output-path ../tmp/mcp-text-contract-coverage/codex-core-router.json \
  -p codex-core tools::router
```

## Clippy 与全量测试

优先使用仓库工具：

```bash
cd codex-rs
just clippy --workspace --all-targets
RUST_MIN_STACK=8388608 cargo nextest run --no-fail-fast \
  -p codex-tools -p codex-core -p codex-config -p codex-mcp \
  -p codex-cli -p codex-protocol -p codex-rmcp-client
```

如果全仓库 clippy 或大范围 nextest 失败在与当前改动无关的官方基线问题上，最终记录必须区分：

- 当前改动相关验证是否通过。
- 失败命令的失败文件、失败测试或 lint 名称。
- 为什么该失败不属于当前改动路径。

不得把带无关失败的全量命令描述为“全部通过”。

## 本地 Cargo 加速与构建产物维护

本 fork 的日常 MCP Text Contract 验证主要走 Cargo 路径。`codex-rs` 是大型 Rust workspace，本地开发默认使用用户级 `sccache` 加速 Cargo 编译，但不把 `rustc-wrapper` 写入仓库 tracked `.cargo/config.toml`。

本机自用 release binary 使用 [local-release-build.md](local-release-build.md) 中记录的 `codex-micheng` 流程。默认入口是：

```bash
micheng/scripts/build-codex-micheng.sh
```

该脚本为本地自用构建覆盖 release profile：使用 thin LTO 和 16 个 codegen units，安装到 `~/.local/bin/codex-micheng`，并默认保留 workspace `target` 作为受控构建工作集。官方 release profile 仍保留在 `codex-rs/Cargo.toml`，需要复现官方发布构建时再直接运行原始 `cargo build --release -p codex-cli --bin codex`。

如果当前机器缺少 `sccache`，先安装用户可用的编译缓存工具：

```bash
sudo apt-get install -y sccache
```

用户级环境文件位于：

```text
~/.config/codex-dev/codex-rs-env.sh
```

进入 Rust workspace 后启用本地开发环境：

```bash
cd codex-rs
source ~/.config/codex-dev/codex-rs-env.sh
sccache --start-server
```

该环境设置：

```bash
export RUSTC_WRAPPER=sccache
export SCCACHE_DIR="$HOME/.cache/sccache-codex"
export SCCACHE_CACHE_SIZE=40G
export RUST_MIN_STACK=8388608
```

`sccache` 只服务本机 Cargo 开发循环。不要把该配置提交到上游同步分支，也不要用它接管 Bazel；Bazel 继续使用仓库 `.bazelrc` 中的 Bazel cache 配置。

日常热开发保留 Cargo incremental 默认行为。只有在清理 `target` 后重建、跨 worktree 复用缓存或需要更高 `sccache` 命中率的冷构建场景，才临时使用：

```bash
CARGO_INCREMENTAL=0 cargo nextest run -p codex-core freeform
```

覆盖率和大范围验证应避免污染普通开发 `target`。需要生成覆盖率摘要时，优先使用独立 target 目录：

```bash
CARGO_TARGET_DIR=target-coverage cargo llvm-cov nextest --no-clean --json --summary-only \
  --output-path ../tmp/mcp-text-contract-coverage/codex-tools.json \
  -p codex-tools
```

构建产物维护时只清理可再生目录，例如 `codex-rs/target/debug`、`codex-rs/target/release`、`codex-rs/target/rust-analyzer` 或专用 coverage target。`codex-rs/target` 默认允许增长，90GiB 是人工复核阈值而不是自动错误。需要回收空间时显式运行：

```bash
CODEX_MICHENG_CLEAN_TARGET=1 micheng/scripts/build-codex-micheng.sh
```

不要把 `tmp/mcp-text-contract-blackbox/`、`micheng/` 文档或其他验证证据当作构建缓存清理。

清理或调参后，用以下命令记录状态：

```bash
sccache --show-stats
du -sh target ~/.cache/sccache-codex
```

## 冲突处理

合并 `upstream/main` 出现冲突时，先判断冲突是否触及高风险文件。

冲突不触及 MCP Text Contract 时，按官方代码意图和本 fork 的普通维护规则解决。

冲突触及 MCP Text Contract 时，按以下顺序处理：

1. 重新阅读设计文档中的行为边界。
2. 判断官方变更是在重构位置、调整数据结构，还是改变同一行为面。
3. 保留本 fork 的外部行为约束，但优先贴合官方当前代码结构。
4. 避免用临时绕过、字符串拼接或重复逻辑硬保功能。
5. 解决冲突后运行目标 nextest 和必要的 llvm-cov。

如果官方变更与本 fork 行为目标发生真实冲突，应在提交说明中写明取舍，不得把行为变化藏在冲突解决里。

## 提交日志

提交标题使用简洁中文，说明实际对象和动作。

提交正文应包含：

- 变更原因。
- 行为变化。
- 兼容性说明。
- 测试与覆盖率证据。
- 已知未解决的无关基线失败。

文档提交应说明文档职责边界；代码提交应说明行为边界和验证证据。
