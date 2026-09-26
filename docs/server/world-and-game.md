# World и Game

World ведёт мировые данные и сохранение; Game исполняет события региона. [Архитектурная граница](../architecture/state-ownership.md) объясняет причину разделения. Имена World и Game сохраняются на межсерверной границе: в переходном запуске это два бинарника со своими соединениями и wire-направлениями. Действующие владельцы тех же обязанностей — Realm (загрузка, мировое состояние, сохранение) и Zone (живое состояние региона); ссылки на `src/worldserver` и `src/gameserver` ниже ведут на переходные shim-реэкспорты и hub, а не на основной код. Эта страница помогает проследить исполнение в коде.

## Организация кода

| Часть | Game | World |
| --- | --- | --- |
| Состояние процесса и основной цикл | переходный hub [gameserver/game.rs](../../server/rust/src/gameserver/gameserver/game.rs) | Realm app [world_game.rs](../../server/rust/realm/src/app/world_game.rs) |
| Сеть, ресурсы и фоновые задачи | переходный [Game runtime](../../server/rust/src/gameserver/gameserver/runtime.rs) | [World runtime в Realm](../../server/rust/realm/src/app/world_runtime.rs) |
| Игровые подсистемы | владельцы [Zone](../../server/rust/zone/src/); переходные адаптеры — `gameserver/appserver/` | владельцы [Realm](../../server/rust/realm/src/) |
| Загрузка и сохранение | Обмен состоянием с World | [Realm characters playerloadworker](../../server/rust/realm/src/characters/playerloadworker.rs), [Realm persistence savedb](../../server/rust/realm/src/persistence/savedb.rs), [worlddb](../../server/rust/realm/src/persistence/) |

Названия `CGame` и `CPlayer` используются по обе стороны границы, но их обязанности различаются. Доменный `CGame` World перенесён в [Realm app](../../server/rust/realm/src/app/world_game.rs); `CGame` Game пока остаётся переходным hub-ом старого пакета и растворяется по мере появления владельцев Zone. Живой игрок хранит фигуру и активное поведение; мировая запись — мировую проекцию и связь с БД.

## Путь команды в Game

В переходном запуске сетевой runtime и `nets/netserver` помещают принятые сообщения в очередь. Переходный hub `CGame::process_messages` передаёт их в `run_incoming_message`, который выбирает обработчик из `appserver/message/`: движение, навыки, предметы, торговля и другие семейства. Сами операции при этом в основном уже делегированы владельцам Zone.

Обработчик меняет состояние своего владельца либо ставит команду в очередь исполнения. Например, шаг передаётся `CPlayerAI` и применяется в игровом AI-проходе. Общая схема — в [симуляции](../gameplay/simulation.md), точный порядок AI и сообщений — в [серверном цикле](runtime-ordering.md).

Для навыка недостаточно добавить ID в фабрику. Нужны конкретные пути Begin, AI, End и поддерживаемая форма цели; [модель навыков](../gameplay/skills.md) показывает их связь.

## Путь данных в World

World получает сообщения Game через направление `networld`: владельцы края перенесены в Realm app ([server](../../server/rust/realm/src/app/world_server.rs), [принятый Game-клиент](../../server/rust/realm/src/app/world_server_client.rs), [исходящий к Login](../../server/rust/realm/src/app/world_client.rs)), в `nets/networld` остались переходные реэкспорты. Мировой диспетчер обрабатывает серверную очередь, затем очередь соединения с Login.

Загрузка персонажа проходит через мирового [CPlayer](../../server/rust/realm/src/characters/player.rs), [worker загрузки](../../server/rust/realm/src/characters/playerloadworker.rs) и [rsplayer](../../server/rust/realm/src/persistence/rsplayer.rs). При сохранении World формирует снимки, передаёт их очередям и выполняет [SQL-фазы](../architecture/database.md). Живое состояние Game и DB-снимок имеют разный жизненный цикл.

При изменении персонажа проверяйте обе стороны сериализации и обратную загрузку. Правила online/offline, возврат из Game и потеря игрового сервера описаны в [жизненном цикле персонажа](../gameplay/player-lifecycle.md).

## Инициализация и диагностика

Game читает обязательный `setup.ini` и необязательный `setupex.ini`. `init_through_billing` прекращает инициализацию при недоступном World; ошибка подключения Billing регистрируется и не обрывает эту стадию. Завершение проходит через `Release`.

World последовательно создаёт настройки, ресурсы, DB-модули, игровые реестры, таймеры и сеть. Его `game_thread_func` различает штатный результат, аварийную ветвь и остановку на неизвестном контракте. Последняя может вернуть диагностический результат с ещё живым владельцем; поэтому её нельзя обрабатывать как обычное завершение.

Пример такой границы — Largess: `dwLoadLargessTime` и `dwNumber` должны быть прочитаны из настроек. Отсутствующие значения представлены явно; при диагностике сначала проверяйте конфигурацию и указанную в результате фазу.

## Как читать оставшийся RAW-код

Некоторые исторические файлы содержат только псевдокод, другие объединяют его с Rust-реализацией. Например, старые Game `gameserver.rs` и `billclient.rs` сами по себе не являются рабочими входами, а `servernationregion.rs` содержит обе части. Проверяйте подключение модуля и конкретный вызов.

`updatesys/` не подключён в [lib.rs](../../server/rust/src/lib.rs). Состояние ServerUpdate и остальных подсистем собрано в [аудите](../status/audit.md).
