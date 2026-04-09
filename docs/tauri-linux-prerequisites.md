# Linux prerequisites для Tauri runtime и сборки `.deb`

## Назначение

Этот документ фиксирует проверенный набор системных пакетов для:

- компиляции `tauri-runtime` в `apps/liveletters-rust-backend-app`;
- локального dev-запуска Tauri на Linux;
- последующей сборки `.deb`.

## Проверенный набор пакетов

В текущей среде для прохождения `cargo check --features tauri-runtime` потребовался следующий набор:

```bash
sudo apt update && sudo apt install -y build-essential pkg-config curl wget file libssl-dev libglib2.0-dev libgtk-3-dev libpango1.0-dev libgdk-pixbuf-2.0-dev libcairo2-dev libjavascriptcoregtk-4.1-dev libwebkit2gtk-4.1-dev libsoup-3.0-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev
```

Для упаковки `.deb` дополнительно:

```bash
sudo apt install -y dpkg-dev fakeroot
```

## Проверка, что зависимости на месте

```bash
cd apps/liveletters-rust-backend-app
cargo check --features tauri-runtime
```

Если проверка зеленая, системные зависимости для compile-path считаются установленными.

## Важное ограничение про запуск окна

Даже при корректных пакетах `cargo run --features tauri-runtime` требует активную GUI-сессию Linux.

В headless-среде ожидаемое падение:

- `Failed to initialize GTK`

Это ограничение runtime-окружения, а не missing package.

## Источники

- [Tauri v2: Linux prerequisites](https://v2.tauri.app/start/prerequisites/)
- [Tauri v2: Debian packaging](https://v2.tauri.app/distribute/debian/)
