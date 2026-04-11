#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "${SCRIPT_DIR}"

echo "=== LiveLetters Tauri E2E ==="
echo ""

# --- Проверка системных зависимостей ---
echo "[check] WebKitWebDriver ..."
if ! command -v WebKitWebDriver &>/dev/null; then
  echo "  НЕ НАЙДЕН: WebKitWebDriver"
  echo "  Установите: sudo apt install -y webkit2gtk-driver"
  exit 1
fi
echo "  OK: $(command -v WebKitWebDriver)"

echo "[check] tauri-driver ..."
if ! command -v tauri-driver &>/dev/null; then
  echo "  НЕ НАЙДЕН: tauri-driver"
  echo "  Установите: cargo install tauri-driver"
  exit 1
fi
echo "  OK: $(command -v tauri-driver)"

# --- Проверка app binary ---
APP_BINARY="${SCRIPT_DIR}/target/debug/liveletters-rust-backend-app"
if [ ! -f "${APP_BINARY}" ]; then
  APP_BINARY="${SCRIPT_DIR}/target/release/liveletters-rust-backend-app"
fi

if [ ! -f "${APP_BINARY}" ]; then
  echo ""
  echo "[error] App binary не найден."
  echo "  Соберите: cd ${SCRIPT_DIR} && cargo build --features tauri-runtime"
  exit 1
fi
echo "[check] App binary: OK (${APP_BINARY})"

# --- Проверка frontend resources ---
FRONTEND_RESOURCES="${SCRIPT_DIR}/../liveletters-frontend-app/resources"
if [ ! -f "${FRONTEND_RESOURCES}/index.html" ]; then
  echo ""
  echo "[warn] Frontend resources не найдены (index.html)."
  echo "  Соберите frontend:"
  echo "    cd ${SCRIPT_DIR}/../liveletters-frontend-app"
  echo "    clojure -M:css && clojure -M:app"
  exit 1
fi
echo "[check] Frontend resources: OK"

# --- Проверка e2e зависимостей ---
if [ ! -d "${SCRIPT_DIR}/e2e/node_modules" ]; then
  echo ""
  echo "[install] e2e dependencies ..."
  cd "${SCRIPT_DIR}/e2e"
  pnpm install
  cd "${SCRIPT_DIR}"
fi

# --- Запуск ---
echo ""
if [ "${WDIO_DIAGNOSTIC:-0}" = "1" ]; then
  echo "[mode] diagnostic (verbose)"
else
  echo "[mode] smoke (quiet)"
fi

echo ""
cd "${SCRIPT_DIR}/e2e"
pnpm test
