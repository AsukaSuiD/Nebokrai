# World и Game

World ведёт мировые данные и сохранение; Game исполняет события региона. [Архитектурная граница](../architecture/state-ownership.md) объясняет причину разделения. Эта страница помогает проследить исполнение в коде.

## Организация кода

| Часть | Game | World |
| --- | --- | --- |
| Состояние процесса и основной цикл | [gameserver/game.rs](../../server/rust/src/gameserver/gameserver/game.rs) | [worldserver/game.rs](../../server/rust/src/worldserver/worldserver/game.rs) |
| Сеть, ресурсы и фоновые задачи | [Game runtime](../../server/rust/src/gameserver/gameserver/runtime.rs) | [World runtime](../../server/rust/src/worldserver/worldserver/runtime.rs) |
| Игровые подсистемы | `gameserver/appserver/` | `worldserver/appworld/` |
| Загрузка и сохранение | Обмен состоянием с World | [playerloadworker](../../server/rust/src/worldserver/worldserver/playerloadworker.rs), [savedb](../../server/rust/src/worldserver/worldserver/savedb.rs), [worlddb](../../server/rust/src/dbaccess/worlddb/) |

Названия `CGame` и `CPlayer` сохранены у обоих процессов, но их обязанности различаются. Game хранит фигуру и активное поведение; World — мировую проекцию и связь с БД.

## Путь команды в Game

Сетевой runtime и `nets/netserver` помещают принятые сообщения в очередь. `CGame::process_messages` передаёт их в `run_incoming_message`, который выбирает обработчик из `appserver/message/`: движение, навыки, предметы, торговля и другие семейства.

Обработчик меняет состояние своего владельца либо ставит команду в очередь исполнения. Например, шаг передаётся `CPlayerAI` и применяется в игровом AI-проходе. Общая схема — в [симуляции](../gameplay/simulation.md), точный порядок AI и сообщений — в [серверном цикле](runtime-ordering.md).

Для навыка недостаточно добавить ID в фабрику. Нужны конкретные пути Begin, AI, End и поддерживаемая форма цели; [модель навыков](../gameplay/skills.md) показывает их связь.

## Путь данных в World

World получает сообщения Game через `nets/networld`. Мировой диспетчер обрабатывает серверную очередь, затем очередь соединения с Login.

Загрузка персонажа проходит через мирового [CPlayer](../../server/rust/src/worldserver/appworld/player.rs), worker загрузки и [rsplayer](../../server/rust/src/dbaccess/worlddb/rsplayer.rs). При сохранении World формирует снимки, передаёт их очередям и выполняет [SQL-фазы](../architecture/database.md). Живое состояние Game и DB-снимок имеют разный жизненный цикл.

При изменении персонажа проверяйте обе стороны сериализации и обратную загрузку. Правила online/offline, возврат из Game и потеря игрового сервера описаны в [жизненном цикле персонажа](../gameplay/player-lifecycle.md).

## Инициализация и диагностика

Game читает обязательный `setup.ini` и необязательный `setupex.ini`. `init_through_billing` прекращает инициализацию при недоступном World; ошибка подключения Billing регистрируется и не обрывает эту стадию. Завершение проходит через `Release`.

World последовательно создаёт настройки, ресурсы, DB-модули, игровые реестры, таймеры и сеть. Его `game_thread_func` различает штатный результат, аварийную ветвь и остановку на неизвестном контракте. Последняя может вернуть диагностический результат с ещё живым владельцем; поэтому её нельзя обрабатывать как обычное завершение.

Пример такой границы — Largess: `dwLoadLargessTime` и `dwNumber` должны быть прочитаны из настроек. Отсутствующие значения представлены явно; при диагностике сначала проверяйте конфигурацию и указанную в результате фазу.

## Как читать оставшийся RAW-код

Некоторые исторические файлы содержат только псевдокод, другие объединяют его с Rust-реализацией. Например, старые Game `gameserver.rs` и `billclient.rs` сами по себе не являются рабочими входами, а `servernationregion.rs` содержит обе части. Проверяйте подключение модуля и конкретный вызов.

`updatesys/` не подключён в [lib.rs](../../server/rust/src/lib.rs). Состояние ServerUpdate и остальных подсистем собрано в [аудите](../status/audit.md).
