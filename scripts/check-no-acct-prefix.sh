#!/usr/bin/env bash
# Запрет появления префикса `acct_` в производственном коде и в формировании
# строк вида `format!("acct_<имя>")`. Идентификатор `acct_<имя>` — внутренний
# служебный, его не должно быть в коде, в выводе, в сетевых данных или в
# черновиках `~/.liveletters/drafts/*.toml`.

set -euo pipefail

errors=0

# Только src/ (без tests/), потому что в тестах допустимы строки вида
# `AccountId::new("acct_bob")` (передача произвольной строки в конструктор
# для проверки) и legacy-JSON `subscriber_account_id: "acct_bob"`.
if rg -q "acct_" modules/*/src/ apps/lltt/src/ --type rust; then
  echo "ERROR: acct_ найден в производственном коде:"
  rg "acct_" modules/*/src/ apps/lltt/src/ --type rust
  errors=1
fi

if rg -q 'format!\("acct_' modules/*/src/ apps/lltt/src/ --type rust; then
  echo "ERROR: format!(\"acct_...\") найден в производственном коде:"
  rg 'format!\("acct_' modules/*/src/ apps/lltt/src/ --type rust
  errors=1
fi

if [ "$errors" -ne 0 ]; then
  exit 1
fi

echo "OK: acct_ не найден в производственном коде"
