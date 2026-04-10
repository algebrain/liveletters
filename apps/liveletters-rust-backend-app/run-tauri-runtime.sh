#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_DIR="${SCRIPT_DIR}/../../.docs/runtime-logs"
ARCHIVE_ROOT="${LOG_DIR}/archive"
STAMP="$(date +%y%m%d-%H%M%S)"
STDOUT_LOG="${LOG_DIR}/stdout.log"
STDERR_LOG="${LOG_DIR}/stderr.log"

mkdir -p "${ARCHIVE_ROOT}"

if [[ -d "${LOG_DIR}" ]]; then
  shopt -s nullglob
  items=("${LOG_DIR}"/*)
  shopt -u nullglob

  to_move=()
  for item in "${items[@]}"; do
    if [[ "$(basename "${item}")" == "archive" ]]; then
      continue
    fi
    to_move+=("${item}")
  done

  if [[ "${#to_move[@]}" -gt 0 ]]; then
    archive_dir="${ARCHIVE_ROOT}/${STAMP}"
    mkdir -p "${archive_dir}"
    for item in "${to_move[@]}"; do
      mv "${item}" "${archive_dir}/"
    done
    echo "Archived old runtime logs to ${archive_dir}"
  fi
fi

cd "${SCRIPT_DIR}"
mkdir -p "${LOG_DIR}"

export LIVELETTERS_RUNTIME_LOG_DIR="${LOG_DIR}"

echo "Runtime logs:"
echo "  stdout: ${STDOUT_LOG}"
echo "  stderr: ${STDERR_LOG}"
echo "  frontend: ${LOG_DIR}/frontend-errors.log"
echo "  commands: ${LOG_DIR}/command-errors.log"
echo "  startup: ${LOG_DIR}/backend-startup.log"

cargo run --features tauri-runtime "$@" \
  > >(tee -a "${STDOUT_LOG}") \
  2> >(tee -a "${STDERR_LOG}" >&2)
