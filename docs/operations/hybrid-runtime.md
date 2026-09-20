# Локальный гибридный стенд

Этот стенд соединяет восстановленные серверные процессы Rust с исходным Windows GameServer, работающим через Wine. Его состав задан в [Compose](../../deploy/hybrid/compose.yaml): AuthServer, LoginServer, BillingServer, WorldServer и MiscServer запускаются из Rust-образа, GameServer — из локального `gameserver.exe`. Наличие точки входа [Rust GameServer](../../server/rust/src/bin/gameserver.rs) само по себе не означает, что она используется в этом стенде.

## Источники данных и требования

Для запуска нужны Docker с Compose, PowerShell для [подготовки runtime](../../deploy/hybrid/prepare-runtime.ps1), доступ к образу SQL Server 2022 и локальный каталог `runtime/`. Последний исключён из Git через [`.gitignore`](../../.gitignore): обычный клон репозитория не содержит конфигов, игровых ресурсов, резервных копий БД и Windows GameServer. Стенд поэтому нельзя воспроизвести только из отслеживаемых файлов.

Ожидаемые каталоги `runtime/` — `AuthServer`, `LoginServer`, `BillingServer`, `WorldServer`, `MiscServer`, `GameServer` и `Database`. В `Database` нужны резервные копии `Account.bak`, `BillingDB.bak`, `GameDB05.bak`, `LogDB.bak` и `LoginDB.bak`; их имена и внутренние логические имена жёстко указаны в [скрипте инициализации БД](../../deploy/hybrid/database-init.sh). `GameServer` должен содержать `gameserver.exe` и связанные ресурсы. Остальные каталоги предоставляют собственные `setup.ini` и дополнительные файлы, читаемые соответствующими процессами. Передача бинарников и резервных копий вне Git — отдельная операционная задача.

`prepare-runtime.ps1` требует параметр `ClientAddress`: адрес, по которому игровой клиент достигает GameServer на порту 2347. Скрипт заменяет адреса БД и межсерверных узлов в локальных конфигурациях, записывает адрес GameServer в `WorldServer/serversetup.ini`, настраивает разрешённый GameServer для BillingServer и создаёт `deploy/hybrid/.env`, если файла ещё нет. Этот `.env` содержит пароли для локального SQL Server и учётной записи Miracle; он исключён из Git корневым [`.gitignore`](../../.gitignore). При повторном запуске скрипт **не обновляет существующий `.env`**. Если пароль в исходном `AuthServer/setup.ini` изменился, согласованность с `.env` нужно проверить отдельно. Скрипт читает и пишет изменяемые INI в кодировке Windows-1251.

## Порядок запуска и сети

В [Compose](../../deploy/hybrid/compose.yaml) сначала поднимается MSSQL, затем однократный `database_init`. Тот восстанавливает пять БД, если их ещё нет в постоянном томе `mssql-data`, создаёт отсутствующее представление `LoginDB.dbo.userinfo` поверх таблицы BillingDB и связывает SQL login Miracle с пятью базами. Повторный запуск с уже созданными базами **не восстанавливает их заново**: состояние тома и версия резервных копий могут расходиться.

Затем запускаются Rust AuthServer, BillingServer и LoginServer; WorldServer зависит от LoginServer, MiscServer — от WorldServer. GameServer зависит от WorldServer и BillingServer. Межсерверные адреса задаются именами сервисов Docker в локальных конфигурациях. Снаружи Compose публикует клиентские порты LoginServer `2346` и GameServer `2347`; `2345` — внутренний порт LoginServer для WorldServer. AuthServer слушает `7100`, WorldServer — `1747`, BillingServer — `8188`. Источником конкретных портов и настроек остаются локальные INI, а не эта страница.

[Rust-образ](../../deploy/hybrid/Dockerfile.rust) собирает все объявленные Cargo-бинарники, но копирует в конечный образ только пять перечисленных выше процессов. Каждый работает в `/runtime` с примонтированным собственным каталогом. [Образ GameServer](../../deploy/hybrid/Dockerfile.game) содержит 32-битный Wine и Xvfb; [его точка входа](../../deploy/hybrid/game-entrypoint.sh) ждёт WorldServer и BillingServer, при первом запуске создаёт Wine prefix в постоянном томе `wine-prefix` и запускает `/runtime/gameserver.exe`.

В Compose проверки состояния MSSQL выполняют SQL-запрос, а проверки AuthServer, LoginServer, BillingServer и WorldServer ищут слушающие порты в `/proc/net/tcp`. Эти проверки подтверждают лишь доступность сокета, а не вход игрока, игровые операции или корректность данных. Для MiscServer и GameServer отдельной проверки состояния нет. Перед выводом о пригодности стенда нужен отдельный проход с клиентом и проверкой сценариев после загрузки персонажа.

## Граница реализаций

Активный корпус расположен в [Rust](../../server/rust/); его шесть бинарных входов описаны в [Cargo.toml](../../server/rust/Cargo.toml). Сохранённая [C++-реконструкция](../../server/cpp/) имеет собственные [CMake](../../server/cpp/CMakeLists.txt) и [vcpkg](../../server/cpp/vcpkg.json), собирает пять отдельных серверных приложений и не подключена к этому Compose-стенду. Она служит дополнительным источником при восстановлении семантики, как описано в [README проекта](../../README.md), и не заменяет GameServer под Wine в текущей схеме.
