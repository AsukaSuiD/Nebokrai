# Сборочные границы

У двух реализаций собственные входы. Команды ниже следуют из build-файлов и показывают, **что должно собираться**; они не являются протоколом успешной сборки текущего состояния. Актуальный результат отмечается в [аудите готовности](../status/audit.md).

## Активный Rust-сервер

[`server/rust/Cargo.toml`](../../server/rust/Cargo.toml) объявляет шесть бинарников и Rust edition 2024. Версия компилятора закреплена в [`rust-toolchain.toml`](../../server/rust/rust-toolchain.toml) как `1.97.1`. Процессный слой использует Unix-сигналы в [`process/mod.rs`](../../server/rust/src/process/mod.rs), поэтому целевая среда этой реконструкции — Linux. Сборочный вход из корня `server/rust`:

```sh
cargo build --release --bins
```

Команда требует компиляции **всех шести** входов, включая [`gameserver`](../../server/rust/src/bin/gameserver.rs). Успешная компиляция подтвердит типовую связность текущего дерева, но не загрузку конфигурации, подключение к MSSQL и обмен с клиентом. Бинарники используют текущий рабочий каталог как каталог runtime; связь запуска и данных описана в [модели процессов](../server/process-model.md).

[`deploy/hybrid/Dockerfile.rust`](../../deploy/hybrid/Dockerfile.rust) вызывает ту же `cargo build --release --bins` внутри Linux-образа. В конечный образ он копирует пять служб: Auth, Billing, Login, Misc и World. Rust GameServer при этом всё равно обязан успешно собраться, но контейнерный стенд его не запускает. Docker build использует два BuildKit cache-mount: registry и target; их повторное использование сокращает повторную компиляцию без копирования `target/` в исходное дерево.

## Сохранённый C++-проход

[`server/cpp/CMakeLists.txt`](../../server/cpp/CMakeLists.txt) задаёт C++20, статическую библиотеку и пять приложений: Auth, Login, Billing, Misc и World. Зависимости перечислены в [`vcpkg.json`](../../server/cpp/vcpkg.json): Asio, curl, OpenSSL, zlib, FreeType, spdlog, Iconv и, под Linux, unixODBC/libltdl. CMake выбирает непустые `.cpp`; пустые owner-файлы остаются маркерами структуры и не образуют единицы трансляции. GameServer-приложения в этом CMake нет.

Типовой конфигурационный вход для установленного vcpkg toolchain:

```sh
cmake -S server/cpp -B build/nebokrai-cpp -DCMAKE_TOOLCHAIN_FILE=/path/to/vcpkg/scripts/buildsystems/vcpkg.cmake
cmake --build build/nebokrai-cpp
```

Путь к vcpkg зависит от машины. Эти команды здесь не выполнялись; наличие CMake-целей не подтверждает сборку или полноту C++-приложений.

## Граница проверки

Для утверждения «собирается» нужен результат запуска команды на конкретном commit и целевой платформе. Для утверждения «работает» требуется запуск с совместимыми ресурсами и БД, а для «играбельно» — сквозной сценарий с точной версией исходного клиента. Ни один из этих уровней не следует из количества файлов или успешного старого бинарника.
