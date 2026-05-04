# 本地 release 构建与安装笔记

本文记录 `MichengLiang/codex` fork 在本机 WSL 环境中的本地日用 release 构建、安装命名和构建产物清理约定。

## 目标

本 fork 的 release binary 用于本机日常使用和验证个人改造效果。它不替代官方 `codex` 启动入口，不写入官方上游构建配置，也不改变官方上游同步流程。

本机安装名使用个人后缀：

```text
codex-micheng
```

该名称表示“从当前个人 fork 构建出来的 Codex release 版”。不要直接覆盖已有 `codex` 命令；需要切换默认入口时，应另行显式处理。

## 构建前环境

进入 Rust workspace 后启用本地 Cargo 加速环境：

```bash
cd /home/t103o/workbench/projects/codex/codex-rs
source ~/.config/codex-dev/codex-rs-env.sh
sccache --start-server
```

环境文件由本机维护，位置为：

```text
~/.config/codex-dev/codex-rs-env.sh
```

该文件设置 `RUSTC_WRAPPER=sccache`、独立 `SCCACHE_DIR`、`SCCACHE_CACHE_SIZE=40G` 和 `RUST_MIN_STACK=8388608`。

## 推荐构建命令

默认使用本 fork 的本地日用构建脚本：

```bash
cd /home/t103o/workbench/projects/codex
micheng/scripts/build-codex-micheng.sh
```

该脚本会执行：

```bash
source ~/.config/codex-dev/codex-rs-env.sh
sccache --start-server
CARGO_PROFILE_RELEASE_LTO=thin
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
CARGO_PROFILE_RELEASE_STRIP=symbols
cargo build --release -p codex-cli --bin codex
install -m 755 target/release/codex ~/.local/bin/codex-micheng
du -sh /home/t103o/workbench/projects/codex \
  /home/t103o/workbench/projects/codex/codex-rs/target \
  ~/.cache/sccache-codex \
  ~/.local/bin/codex-micheng
```

该脚本仍使用 release-mode codegen，但覆盖官方 release profile 的最重发布参数：本地默认使用 thin LTO 和 16 个 codegen units，而不是官方的 fat LTO 和单 codegen unit。这个取舍面向本机自用 binary 的重建延迟，不追求官方发布包的最激进体积/优化策略。

脚本默认保留 `codex-rs/target`，因为本机开发策略是用受控磁盘空间换取重建速度。`CODEX_MICHENG_TARGET_REVIEW_GB` 默认值为 `90`；当 `target` 达到或超过该值时，脚本只提示人工复核，不自动清理。

官方 release profile 仍然保留在 `codex-rs/Cargo.toml` 中。需要复现官方发布取舍时，直接运行：

```bash
cd /home/t103o/workbench/projects/codex/codex-rs
cargo build --release -p codex-cli --bin codex
```

官方 release profile 可能进入较长的 fat LTO/link 阶段。长时间无日志不等于卡住；确认时可旁路查看 `rustc` 是否仍在占用 CPU。

## 安装到用户 PATH

脚本构建成功后，会把 release binary 安装为个人命名入口：

```bash
install -m 755 target/release/codex ~/.local/bin/codex-micheng
```

验证：

```bash
command -v codex-micheng
codex-micheng --version
```

`~/.local/bin` 应位于 PATH 前部。当前约定仍保留原有 `codex` 命令，不用 `codex-micheng` 覆盖它。

## 构建产物清理

安装完成后，`target/release/deps`、`target/release/build`、`target/release/gn_out` 等目录是可再生中间产物，但它们也是下一次本地 release 构建的直接加速来源。本机默认保留 `codex-rs/target`，把它视为受控工作集，而不是每次构建后的垃圾。

如果某次需要回收空间，显式设置：

```bash
CODEX_MICHENG_CLEAN_TARGET=1 micheng/scripts/build-codex-micheng.sh
```

该清理不会删除已经安装到 `~/.local/bin/codex-micheng` 的可执行文件。

构建后记录空间：

```bash
du -sh /home/t103o/workbench/projects/codex \
  /home/t103o/workbench/projects/codex/codex-rs/target \
  /home/t103o/.cache/sccache-codex \
  /home/t103o/.local/bin/codex-micheng
```

## 当前基线记录

最近一次本地日用 release 脚本构建结果：

```text
命令: micheng/scripts/build-codex-micheng.sh
结果: 成功
Cargo 构建耗时: 9m 38s
脚本总耗时: 9m 43s
安装路径: /home/t103o/.local/bin/codex-micheng
版本输出: codex-cli 0.0.0
binary 大小: 280M
构建后清理: Removed 9912 files, 4.4GiB total
```

该记录来自脚本默认策略切换前的一次构建。当前策略已改为默认保留 `codex-rs/target`，以便后续本地重建复用 release 工作集。

对照：最近一次官方 release profile 构建结果：

```text
命令: cargo build --release -p codex-cli --bin codex
结果: 成功
耗时: 40m 16s
安装路径: /home/t103o/.local/bin/codex-micheng
版本输出: codex-cli 0.0.0
binary 大小: 201M
```

该次官方构建后曾执行 `cargo clean --manifest-path codex-rs/Cargo.toml` 清理 release 中间产物。

本机自用构建默认走 `micheng/scripts/build-codex-micheng.sh`，以降低 rebuild latency。官方 profile 的 binary 更小；本地日用 profile 的 binary 更大，但构建时间显著更短。

## 空间预算

本机开发允许用磁盘空间换取构建速度。当前约定：

```text
sccache-codex: 40G 硬上限，由 sccache 自己淘汰旧缓存
codex-rs/target: 默认保留，90GiB 作为人工复核阈值
codex-micheng: 安装到 ~/.local/bin/codex-micheng，长期保留
```

`codex-rs/target` 达到 90GiB 不自动视为错误；它只表示需要确认这些构建产物是否仍服务当前开发。只有在明确需要回收空间、切换大方向或 target 状态失去价值时，才清理。

## 边界

- 本笔记只记录本机个人 fork 使用流程。
- 不修改官方 `docs/contributing.md`。
- 不把 `RUSTC_WRAPPER` 写入 tracked `.cargo/config.toml`。
- 不用 `sccache` 接管 Bazel；Bazel 继续使用仓库 `.bazelrc` 的缓存配置。
- 不把 `tmp/mcp-text-contract-blackbox/` 或 `micheng/` 文档当作构建缓存清理。
