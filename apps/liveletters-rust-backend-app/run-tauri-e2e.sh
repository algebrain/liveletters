#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "${SCRIPT_DIR}"

echo "=== LiveLetters Tauri E2E ==="
echo ""

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

# --- Проверка Playwright browser ---
if [ ! -d "${SCRIPT_DIR}/e2e/node_modules/@playwright" ]; then
  echo ""
  echo "[warn] Playwright package не найден в node_modules."
  echo "  Выполните: cd ${SCRIPT_DIR}/e2e && pnpm install"
  exit 1
fi

# --- Запуск ---
echo ""
if [ "${E2E_DIAGNOSTIC:-0}" = "1" ]; then
  echo "[mode] diagnostic (verbose)"
  export E2E_DIAGNOSTIC=1
else
  echo "[mode] smoke (quiet)"
fi

echo ""
cd "${SCRIPT_DIR}/e2e"
pnpm test
