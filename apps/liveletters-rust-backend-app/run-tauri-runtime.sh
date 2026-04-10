#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "${SCRIPT_DIR}"

export LIVELETTERS_DEBUG_LOGS=1

echo "Debug runtime logging enabled."
echo "Runtime logs will be written inside <effective-home>/.liveletters/runtime-logs"
echo "Use --home-dir=/tmp/some-home to override the effective home for this run."

cargo run --features tauri-runtime -- "$@"
