# Обзор архитектуры / Architecture tour

**English summary.** The current ownership map is the Realm/Zone/Shared libraries: Realm owns entry, world state, persistence, billing, and auctions; Zone owns live regional simulation; Shared is the neutral technical layer. A transitional package still builds the six original service binaries and preserves their wiring. Login coordinates entry, Auth handles one authentication route, World loads and persists characters, and Game owns live regional simulation. Billing handles account operations; Misc handles auctions. The diagram is a responsibility map, not a claim that every route has passed a client scenario.

## Связи служб в переходном запуске

Действующая карта владения — [компонентная структура Realm, Zone и Shared](../architecture/realm-and-zone.md): Realm объединил вход, мировые данные, сохранение, счёт и аукцион, Zone владеет живой симуляцией, Shared — нейтральными механизмами. Переходный запуск при этом по-прежнему собирает шесть бинарников исходной модели; историческое число процессов сохраняется только как способ запуска и как baseline оригинала, а не как текущая карта ownership. Следующая диаграмма показывает существующие связи этого запуска.

```mermaid
flowchart TB
    Client["Legacy client"] <-->|"entry and responses"| Login
    Login <-.->|"Auth route only"| Auth
    Login <-->|персонажи и маршрут| World
    Client <-->|игровая сессия| Game
    World <-->|снимки и сохранение| Game
    World <-->|данные персонажей| SQL[(Microsoft SQL Server)]
    Game <-->|баланс и операции| Billing
    World <-->|аукцион| Misc
```

Диаграмма показывает основные связи обмена, а не порядок установки TCP-соединений. Пунктир к Auth обозначает условный маршрут. В частности, Auth — один из путей проверки Login, а не обязательный шаг любой ветви: существуют GAS и другие маршруты. SQL используется также профильными DB-модулями служб; изображённый путь World не означает исключительное владение всеми таблицами БД.

| Служба | Что важно понимать новичку | Где изучать подробно |
| --- | --- | --- |
| Login | Вход ещё не является игровой сессией: клиент ожидает результаты других служб. | [Вход и службы](../server/auth-login-and-services.md) |
| Auth | Проверяет учётную запись в выбранном маршруте и возвращает результат ожидающему Login. | [Протокол входа](../protocol/login-auth.md) |
| World | Связывает персонажа с мировыми данными, БД и игровым процессом. | [World и Game](../server/world-and-game.md) |
| Game | Последовательно меняет живые объекты региона и публикует клиентские изменения. | [Симуляция](../gameplay/simulation.md) |
| Billing | Ведёт баланс и связанные операции. | [Аукцион и платежи](../gameplay/auction-and-payments.md) |
| Misc | Обслуживает аукцион через обмен с World. | [Службы](../server/auth-login-and-services.md) |

Обязанности этих исходных служб в действующем коде распределены между владельцами Realm и Zone; соответствие процессов текущим компонентам показано во [владении состоянием](../architecture/state-ownership.md#обязанности-исходных-процессов-историческая-основа).

## Где живёт состояние

В действующей карте живое региональное состояние принадлежит Zone, загрузка и сохранение мировой проекции персонажа — Realm; в переходном запуске эту пару исполняют бинарники Game и World. Передача между ними — явная граница: снимок, очередь, обработчик и результат записи могут относиться к разным моментам времени. Поэтому успешная отправка сообщения не доказывает сохранение, а поздняя ошибка не отменяет уже выполненные эффекты.

```mermaid
flowchart LR
    Live["Zone (Game): живое состояние"] <-->|межсерверные сообщения| Persistent["Realm (World): загрузка и сохранение"]
    Persistent <-->|параметры и результаты SQL| DB[(Данные персонажа)]
```

При добавлении сохраняемого поля нужно согласовать живой объект, передаваемый формат, чтение и запись. Подробные правила и ссылки на код остаются в [основном документе о владении состоянием](../architecture/state-ownership.md), [сохранении](../architecture/persistence.md) и [БД](../architecture/database.md).

## Современная инфраструктура и старые контракты

Переходный серверный пакет предоставляет шесть бинарников и делегирует домен библиотекам Shared, Realm и Zone; состав выделенных компонентов указан в [карте кода](../architecture/workspace.md). Целевой состав запуска — входы ролей Realm и Zone; переходные бинарники исчезают вместе с заменяемыми hub-ами. Каждый процесс создаёт своё состояние. Tokio обслуживает сеть и задачи; очереди сохраняют точки исполнения доменных операций. Tiberius заменяет ADO для SQL Server. Совместимость требует учитывать направление кадра, порядок полей, знаковость, переполнение, кодировки, таймеры и частичные эффекты — не только имена функций.

Продолжение знакомства: [путь игрока](player-journey.md) → [текущий статус](status.md). Для работы с реализацией: [карта кода](../architecture/workspace.md) и [общие механизмы](../architecture/shared-mechanisms.md).
