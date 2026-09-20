# Совместный локальный запуск Rust-служб

Auth, Login, World, Game, Billing и Misc запускаются через [compose.rust.yaml](../../deploy/hybrid/compose.rust.yaml), наложенный на общий [Compose](../../deploy/hybrid/compose.yaml). Дополнение заменяет Wine Game на Rust и подключает бинарники из `.local/rust-bin`. Сеть, базы, данные и порядок зависимостей остаются общими с [гибридным стендом](hybrid-runtime.md).

## Сборка и запуск

Из корня репозитория в PowerShell, при подготовленном `runtime/` и разрешённом запуске:

```powershell
./deploy/hybrid/prepare-runtime.ps1 -ClientAddress 127.0.0.1 -RustOnly
./deploy/check-rust.ps1 -Mode Build
docker --context desktop-linux compose -f deploy/hybrid/compose.yaml -f deploy/hybrid/compose.rust.yaml up -d --no-build
```

`Build` собирает все шесть бинарников в dev-профиле без debug-информации и incremental, затем копирует их в `.local/rust-bin`. Используется тот же ограниченный кэш, что у `cargo check`; отдельной release-сборки и BuildKit-кэша этот путь не создаёт. Rust-образ содержит библиотеки окружения сборки, поэтому он же используется для локального запуска. Сборка компактного образа для развёртывания описана в [сборке](build.md).

`127.0.0.1:2346` — вход клиента в Login, `127.0.0.1:2347` — вход в Game. Дополнение публикует оба порта только на loopback. Для доступа с другого компьютера нужно согласованно изменить публикации и клиентский адрес в настройках. MSSQL доступен внутри Docker-сети. Подготовка конфигов и `database_init` имеют те же [побочные эффекты и ограничения](hybrid-runtime.md#базы-и-постоянные-данные), что у базового стенда: отсутствующие БД восстанавливаются, существующие не заменяются, login `Miracle` согласуется с локальным `.env`.

Все команды управления этим составом должны включать **оба** файла. Базовый Compose без дополнения снова выбирает Wine Game. Том Wine в Rust Game не подключается. Автоперезапуск Rust-служб отключён, чтобы аварийное завершение оставалось видимым; Docker-журнал каждой службы ограничен двумя файлами по 10 МиБ.

`-RustOnly` задаёт Misc объявленный адрес `127.0.0.1:2382`, добавляет его в World registry с ID 5 и пересчитывает число записей. Это identity для регистрации; к World Misc по-прежнему подключается через Docker DNS. World сопоставляет объявленные адрес и порт, затем разбирает IPv4: одного имени `nebokrai_misc` без маршрутной записи недостаточно. Этот режим также отключает обработчик внешних пополнений, описанный ниже. `VERIFIED` по [подготовке](../../deploy/hybrid/prepare-runtime.ps1) и [обработчику регистрации](../../server/rust/src/worldserver/appworld/message/servermessage.rs).

## Наблюдение и повторный запуск

```powershell
docker --context desktop-linux compose -f deploy/hybrid/compose.yaml -f deploy/hybrid/compose.rust.yaml ps -a
docker --context desktop-linux compose -f deploy/hybrid/compose.yaml -f deploy/hybrid/compose.rust.yaml logs --tail 100 nebokrai_game nebokrai_world
docker --context desktop-linux compose -f deploy/hybrid/compose.yaml -f deploy/hybrid/compose.rust.yaml stop
```

После изменения Rust-кода снова выполните `-Mode Build`, затем `up -d --no-build` и `restart` изменённых служб: замена файла не обновляет уже исполняемый процесс. Сборка и проверка используют одно фиксированное имя контейнера и не выполняются одновременно. Артефакты `.local/rust-bin` остаются после очистки кэша.

Game подключается к World и Billing, но клиентский listener создаёт позднее, после завершающего сообщения конфигурации World `0x7F801/0x3B`. Поэтому существование процесса и TCP-соединения с World ещё не означает получение регионов и готовность принимать игрока. Дополнительный healthcheck Game требует состояние `LISTEN` на `2347`; это подтверждает только открытие порта. Проверять нужно также присвоенный server ID, загруженные регионы и результат клиентского входа. Источники — [Game Init](../../server/rust/src/gameserver/gameserver/game.rs), [startup handler](../../server/rust/src/gameserver/appserver/message/servermessage.rs); фактические результаты — в [состоянии проекта](../status/audit.md).

## Внешние пополнения Billing

Если в поставочной BillingDB отсутствует `TBL_NeedUpdate` и нет внешней системы пополнений, локальный стенд запускается с `PlayerFillCheckSrvSwitch 0` в `runtime/BillingServer/setup.ini`. Это штатное отключение worker-а уведомлений об изменении баланса, а не восстановление платёжной интеграции. Обычные запросы Billing продолжают обслуживаться. При включённом worker-е отсутствующая таблица даёт SQL 208 каждые пять секунд. Запрос подтверждён оригинальным Billing EXE; полный DDL и производитель очереди ещё неизвестны, поэтому пустая таблица для подавления ошибки не создаётся. Источники — [PlayerFill worker](../../server/rust/src/billingserver/appbilling/playerfillmgr.rs), [SQL-потребитель](../../server/rust/src/dbaccess/dbbilling/rsplayerfillmgr.rs).
