# Сборка сервера и контейнерных образов

Основная сборка проекта — Rust под Linux. Команды на этой странице предназначены для сборки на подготовленной машине; при чтении проекта и правке документации запускать их не требуется. Результаты на конкретной версии, платформе и составе стенда хранятся в [состоянии проекта](../status/audit.md).

## Активный Rust-сервер

[GitHub Actions](../../.github/workflows/rust-check.yml) проверяет библиотеки workspace и шесть серверных входов командой `cargo check --locked --workspace --lib --bins` при push в `main` и в pull request. Образ Rust и checkout action закреплены по digest/commit. Проверка не запускает тесты или игровые службы, не использует оригинальные материалы и не требует секретов репозитория. Успех этой проверки подтверждает типы и заимствования, а не линковку или работу клиента.

Пакет объявляет Rust edition 2024 и шесть бинарников: `authserver`, `billingserver`, `gameserver`, `loginserver`, `miscserver`, `worldserver`. В манифесте и toolchain закреплена версия `1.97.1`. Общий процессный модуль без условной компиляции импортирует Unix-сигналы: установка Rust в Windows сама по себе не даёт нативную Windows-сборку этого дерева. Основание выбора платформы — [ADR-0002](../decisions/0002-semantic-boundary.md); сборочные входы — [Cargo.toml](../../server/rust/Cargo.toml), [rust-toolchain.toml](../../server/rust/rust-toolchain.toml) и [process/mod.rs](../../server/rust/src/process/mod.rs).

Для быстрой проверки типов и заимствований на ПК с Docker Desktop есть [check-rust.ps1](../../deploy/check-rust.ps1). Запускайте из PowerShell 7.3 или новее, из корня проекта:

```powershell
./deploy/check-rust.ps1
./deploy/check-rust.ps1 -Mode Build
```

По умолчанию скрипт выполняет `cargo check --locked --workspace --lib --bins`: проверяет типы и заимствования библиотек workspace и шести серверных входов без линковки и запуска служб. `.local/rust-bin` — создаваемый инструментом каталог локальных сборочных артефактов, а не ссылка на приватный исследовательский корпус или часть поставки. `-Mode Build` выполняет `cargo build --locked --bins` и копирует шесть dev-бинарников в `.local/rust-bin` для [совместного запуска](rust-runtime.md). Windows PowerShell 5.1 не поддерживается: он иначе передаёт кавычки в Docker CLI; проверка версии останавливает скрипт до обращения к кэшу. Используется локальный контекст `desktop-linux`, образ `rust:1.97.1-bookworm`, исходники монтируются только для чтения.

Кэш находится в одном выделенном Docker-томе `nebokrai-rust-check-cache`. Incremental и debug-информация отключены; одновременно работают две задачи компиляции. Фиксированное имя контейнера `nebokrai-rust-check` не допускает второго одновременного запуска скрипта: очистка кэша не должна пересекаться с другой проверкой. При смене toolchain или `Cargo.lock` старое содержимое тома очищается. До и после проверки контролируется размер: при превышении `CacheLimitMiB` (по умолчанию 2048) удаляется target, затем при необходимости скачанные зависимости. Во время компиляции размер может временно превысить порог. Скрипт не чистит другие тома, образы или общий BuildKit-кэш; контейнер проверки удаляется после завершения. Сам Rust-образ в этот лимит не входит.

В Linux-оболочке, из корня репозитория:

```sh
cd server/rust
cargo build --release --bins
```

Без переопределения каталога Cargo результат окажется в `server/rust/target/release/`. Эта команда собирает все шесть входов. Выбор одного `--bin` сокращает набор исполняемых файлов, но не отделяет его серверные модули от остальных: каждый вход использует одну библиотеку, а `lib.rs` подключает в неё и Game, и соседние службы. Поэтому ошибка подключённого модуля Game может мешать сборке Auth. Связь видна в [lib.rs](../../server/rust/src/lib.rs) и [бинарных оболочках](../../server/rust/src/bin/).

Каталог сборки и каталог запуска решают разные задачи. Бинарник ищет runtime относительно **текущего рабочего каталога**, а не своего расположения; CLI для выбора каталога конфигурации нет. Например, запуск Auth из Linux-корня репозитория выглядит так:

```sh
cd runtime/AuthServer
../../server/rust/target/release/authserver
```

Пример предполагает подготовленные данные и доступные Auth адреса БД/соседей. Имена `mssql` и `nebokrai_*`, записанные подготовкой гибридного стенда, относятся к сети Compose и не обеспечивают подключение при запуске бинарника на хосте. Конфигурация описана в [локальном стенде](hybrid-runtime.md), получение каталога — в [модели процессов](../server/process-model.md).

## Образы гибридного стенда

Docker должен работать с Linux-контейнерами и поддерживать BuildKit cache-mount. Из корня репозитория образы можно собрать независимо от запуска Compose:

```powershell
docker build -f deploy/hybrid/Dockerfile.rust -t nebokrai-rust-services:local .
docker build -f deploy/hybrid/Dockerfile.game -t nebokrai-wine-gameserver:local deploy/hybrid
```

Контексты различаются намеренно. Rust Dockerfile копирует `server/rust/Cargo.toml`, `Cargo.lock`, toolchain и `src/` из корня репозитория. Wine Dockerfile получает только каталог `deploy/hybrid` и копирует оттуда entrypoint. Ни один образ не комплектуется локальными INI, backup или `gameserver.exe`: данные подключает Compose при запуске. Поэтому успешный `docker build` не подтверждает даже наличие необходимого runtime. Источники — [Dockerfile.rust](../../deploy/hybrid/Dockerfile.rust), [Dockerfile.game](../../deploy/hybrid/Dockerfile.game) и [Compose](../../deploy/hybrid/compose.yaml).

Rust builder вызывает `cargo build --locked --release --bins`, затем копирует в конечный Debian-образ все шесть служб, включая `/opt/nebokrai/gameserver`. Выбор Wine или Rust Game определяется составом запуска; наличие бинарника в образе само по себе не переключает базовый гибридный Compose. Причина нынешнего состава — [ADR-0006](../decisions/0006-hybrid-runtime.md).

Cache-mount для registry и `/source/target` позволяет повторно использовать скачанные зависимости и результат компиляции внутри Docker builder. Перед завершением того же шага бинарники переносятся в `/artifacts`, откуда доступны следующей стадии; host-каталог `server/rust/target` здесь не используется. Версия toolchain и `Cargo.lock` сохранены, однако образы заданы тегами без digest, а apt-пакеты не закреплены по версиям. Эти файлы описывают сборочный путь, но не обещают побайтово одинаковый образ при повторении в другой день. Источник — [Dockerfile.rust](../../deploy/hybrid/Dockerfile.rust).

Для изменения запуска сначала найдите нужный слой: общий Tokio runtime и сигналы в `process/mod.rs`, адаптер конкретной службы в `process/`, состав образа в Dockerfile. Существующий `run_process` уже объединяет платформенную оболочку; повторять её в `src/bin` не нужно. Путь изменения и обоснование разделения приведены в [модели процессов](../server/process-model.md).

## Сохранённый C++-проход

Этот отдельный проход не входит в Rust-сборку и гибридный Compose. [CMakeLists.txt](../../server/cpp/CMakeLists.txt) требует CMake 3.24, задаёт C++20, статическую библиотеку из непустых `.cpp` и пять приложений: Auth, Login, Billing, Misc и World. GameServer-приложения нет. Зависимости перечислены в [vcpkg.json](../../server/cpp/vcpkg.json); CMake отдельно ищет ODBC и, в Linux, libltdl.

Типовой конфигурационный вход для установленного vcpkg toolchain:

```sh
cmake -S server/cpp -B build/nebokrai-cpp -DCMAKE_TOOLCHAIN_FILE=/path/to/vcpkg/scripts/buildsystems/vcpkg.cmake
cmake --build build/nebokrai-cpp
```

Команды выполняются из корня репозитория; путь к установленному vcpkg зависит от машины. Наличие CMake-целей не подтверждает сборку или полноту C++-приложений.
