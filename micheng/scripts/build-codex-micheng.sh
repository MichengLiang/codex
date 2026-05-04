#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/../.." && pwd)"
codex_rs="$repo_root/codex-rs"
install_path="${CODEX_MICHENG_INSTALL_PATH:-$HOME/.local/bin/codex-micheng}"
target_review_gb="${CODEX_MICHENG_TARGET_REVIEW_GB:-90}"

if [[ -f "$HOME/.config/codex-dev/codex-rs-env.sh" ]]; then
  # shellcheck source=/dev/null
  source "$HOME/.config/codex-dev/codex-rs-env.sh"
fi

if command -v sccache >/dev/null 2>&1; then
  sccache --start-server >/dev/null 2>&1 || true
fi

# Local-use release profile override.
#
# The upstream release profile uses fat LTO and codegen-units=1 because the
# official packaged binary optimizes for distribution. This fork's local binary
# optimizes for rebuild latency while keeping release-mode codegen.
export CARGO_PROFILE_RELEASE_LTO="${CARGO_PROFILE_RELEASE_LTO:-thin}"
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS="${CARGO_PROFILE_RELEASE_CODEGEN_UNITS:-16}"
export CARGO_PROFILE_RELEASE_STRIP="${CARGO_PROFILE_RELEASE_STRIP:-symbols}"

cd "$codex_rs"
cargo build --release -p codex-cli --bin codex "$@"

mkdir -p "$(dirname -- "$install_path")"
install -m 755 target/release/codex "$install_path"
"$install_path" --version

if [[ "${CODEX_MICHENG_CLEAN_TARGET:-0}" = "1" ]]; then
  cd "$repo_root"
  cargo clean --manifest-path codex-rs/Cargo.toml
else
  target_kib="$(du -sk "$codex_rs/target" 2>/dev/null | awk '{print $1}')"
  if [[ -n "${target_kib:-}" ]]; then
    target_gib="$(( (target_kib + 1024 * 1024 - 1) / (1024 * 1024) ))"
    if (( target_gib >= target_review_gb )); then
      printf 'codex-rs/target is %sGiB; review threshold is %sGiB. Keeping it by design.\n' \
        "$target_gib" "$target_review_gb" >&2
    fi
  fi
fi

du -sh "$repo_root" "$codex_rs/target" "${SCCACHE_DIR:-$HOME/.cache/sccache}" "$install_path" 2>/dev/null || true
