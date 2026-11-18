#!/bin/sh
# Build the plugin wasm and require render() bytes to match shots/*.ansi.txt.
#
# zellij-plugin-snapshot is a command. [dependencies] and [dev-dependencies]
# link a library into the wasm or into `cargo test`, so they leave this binary
# uninstalled. Install the pinned release first:
#   export PATH="$(dirname "$(rustup which cargo)"):$PATH"
#   export DYLD_LIBRARY_PATH="$(cd "$(dirname "$(rustup which cargo)")/../lib" && pwd)" # macOS only; omit on Linux
#   cargo install zellij-plugin-snapshot --version 0.2.2 --locked
# https://github.com/fulldecent/zellij-plugin-snapshot
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
cd "$root"

# Put the toolchain from rust-toolchain.toml first on PATH.
if command -v rustup >/dev/null 2>&1; then
  bin=$(dirname "$(rustup which cargo)")
  PATH="$bin:${HOME}/.cargo/bin:${PATH}"
  DYLD_LIBRARY_PATH="$(CDPATH= cd -- "$bin/../lib" && pwd)${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
  export PATH DYLD_LIBRARY_PATH
else
  PATH="${HOME}/.cargo/bin:${PATH}"
  export PATH
fi

if ! command -v zellij-plugin-snapshot >/dev/null 2>&1; then
  echo "zellij-plugin-snapshot 0.2.2 is not on PATH." >&2
  echo 'export PATH="$(dirname "$(rustup which cargo)"):$PATH"' >&2
  echo 'export DYLD_LIBRARY_PATH="$(cd "$(dirname "$(rustup which cargo)")/../lib" && pwd)" # macOS only; omit on Linux' >&2
  echo 'cargo install zellij-plugin-snapshot --version 0.2.2 --locked' >&2
  exit 1
fi

# .cargo/config.toml already selects wasm32-wasip1 for a plain `cargo build`.
# Name the target here so this script still builds the plugin wasm if that
# default is removed. `cargo test` cannot do this job: it locks the target
# directory, and it runs the host harness rather than the plugin wasm.
cargo build --release --locked --target wasm32-wasip1

out=$(mktemp -d)
trap 'rm -rf "$out"' EXIT

# Each shots/*.yaml `name` is the stem of `{name}.ansi.txt` and `{name}.svg`.
failed=0
for yaml in shots/*.yaml; do
  stem=$(basename "$yaml" .yaml)
  zellij-plugin-snapshot "$yaml" --out "$out"
  if ! diff -u "shots/${stem}.ansi.txt" "$out/${stem}.ansi.txt"; then
    echo "Render bytes differ from shots/${stem}.ansi.txt." >&2
    echo "When that change is intended:" >&2
    echo "  zellij-plugin-snapshot shots/${stem}.yaml --out /tmp/shots" >&2
    echo "  cp /tmp/shots/${stem}.ansi.txt shots/${stem}.ansi.txt" >&2
    # screenshot.svg is the 80-column picture linked from the repository root.
    # The other pictures live beside their YAML scripts.
    if [ "$stem" = screenshot ]; then
      echo "  cp /tmp/shots/${stem}.svg screenshot.svg" >&2
    else
      echo "  cp /tmp/shots/${stem}.svg shots/${stem}.svg" >&2
    fi
    failed=1
  fi
done
if [ "$failed" -ne 0 ]; then
  exit 1
fi
