# Каталог сообщений и границы покрытия

`MsgType` — полное 32-битное слово по смещению `+0x04` [внутреннего заголовка](message-header.md). Направление, допустимый диапазон, внешнее [TCP-обрамление](transport.md) и назначение конкретного типа нужно читать вместе: совпадение младших байт или имени обработчика само по себе не доказывает одинаковый layout. Эта страница фиксирует **достигнутые и сопоставленные** типы входа и перехода между Login, World и Game, а не объявляет полным каталогом всей игры. Значения и роли взяты из точных EXE/PDB, указанных в owner-комментариях [Login](../../server/rust/realm/src/access/logmessage.rs), [World](../../server/rust/realm/src/app/logmessage.rs) и [Game](../../server/rust/src/gameserver/appserver/message/logmessage.rs); это `VERIFIED` для достигнутых ветвей оригинала, но не runtime-проверка новой реализации.

## Приём и маршрутизация

| Точка входа | Фильтр и правило | Доказательство и предел |
| --- | --- | --- |
| Игровой клиент → LoginServer | `0x2FD01..=0x3FBFF` после проверки кадра; тип и socket/CD-key/IP попадают в FIFO | [Receive owner](../../server/rust/realm/src/app/login_server_client.rs), точная Login EXE/PDB. Диапазон `VERIFIED`, обработчики не для каждого числа в нём — `PARTIAL`. |
| Игровой клиент → GameServer | `0x8F701..=0x9F5FF` после проверки кадра; тип и socket/map/IP попадают в FIFO | [Receive owner](../../server/rust/zone/src/app/game_server_client.rs), Game `OnReceive` RVA `0x0001C7F0`. Диапазон `VERIFIED`, полное покрытие содержимого `PARTIAL`. |
| Внутри LoginServer | `Run` сначала проверяет диапазон Auth `0xCF301..0xDF1FE`, затем семейство `MsgType & 0xFFFFFF00`: GM `0x20000`, GMA `0x20100`, Log `0x1FF00/0x2FD00/0x10000`, Server `0xFF00/0x1FE00` | [Login message](../../server/rust/realm/src/app/login_message.rs), точная Login EXE/PDB. Неизвестный тип — no-op с возвратом `1`; это не доказательство успеха доменной операции. |
| Внутри GameServer | `Run` разрешает игрока по числовому map ID, его регион, затем выбирает обработчик по `MsgType & 0xFFFFFF00`; некоторые семейства требуют оба объекта | [Game message](../../server/rust/src/nets/netserver/message.rs), RVA `0x000149D0`. Семейства перечислены в owner-е; их маршрутизация `VERIFIED`, полнота вложенных handlers `PARTIAL`. |

Дополнительные маршруты действующего Rust-кода:

| Процесс | Выбор обработчика | Где продолжать |
| --- | --- | --- |
| Auth | Точный opcode через `AuthMessageKind::from_opcode`, включая проверку `0xCF501/0xCF502`, регистрацию и GM. Неизвестный тип не вызывает handler. | [auth_message.rs](../../server/rust/realm/src/app/auth_message.rs) → `AuthMessageHandlers::handle`. |
| Billing | Семейства `0xFF000/0xEF200` → billing; `0xEF100/0x10EF00` → server; затем точный тип в handler. | [billing_message.rs](../../server/rust/realm/src/app/billing_message.rs) → `BillingMessageHandler` / `ServerMessageHandler`. |
| Misc | `0x14ED00` → аукцион; `0x16EA00` → служебная функция; `0x14EC00` — no-op; остальные → `on_other_msg`. | [misc_message.rs](../../server/rust/realm/src/app/misc_message.rs) → `CGame::process_message`. |
| World, сообщения Misc | `0x15EB00` → `on_misc_auction`, затем `on_msg_m2w_auction`. Это общий принятый server-путь World. | [world_message.rs](../../server/rust/realm/src/app/world_message.rs) → [аукционный обработчик](../../server/rust/realm/src/app/onmsg_m2w_auction.rs). |

Выбор семейства не означает успех операции. Чтобы добавить тип, нужно проследить фильтр входа, диспетчер и точную ветвь handler; порядок работы приведён в [сетевом runtime](../server/network-runtime.md).

## Вход клиента и обслуживание роли через Login и World

Первичный источник для клиентских значений и клиентских ответов — [Login `OnLogMessage`](../../server/rust/realm/src/access/logmessage.rs); для межсерверных запросов и ответов — [Login `CGame`](../../server/rust/realm/src/access/game.rs) и [World `OnLogMessage`](../../server/rust/realm/src/app/logmessage.rs). Статус таблицы — `VERIFIED` для выбора ветви и указанного направления; payload каждого типа требует отдельного анализа и остаётся `PARTIAL`, если нет специальной страницы.

| Событие | Клиент → Login | Login → World | World → Login | Login → клиент |
| --- | --- | --- | --- | --- |
| Обычный вход | `0x2FD01` | Зависит от результата Auth/маршрута; здесь `UNKNOWN` | `0x1FF01` — результат выбора/входа | `0xAF501` и `0xAF503` используются на разных этапах; точное условие см. Login owner |
| Расширенный вход | `0x2FD0B` | `UNKNOWN` здесь | `UNKNOWN` здесь | `0xAF50A` |
| Список персонажей | `0x2FD02` | `0x4FB01` | `0x1FF02` | `0xAF502` |
| Данные персонажа / выбор | `0x2FD03` | `0x4FB05` | `0x1FF01` и/или переход к Game; детализация в owners | Не сводится к одному сообщению |
| Создание роли | `0x2FD04` | `0x4FB04` | `0x1FF05` | `0xAF504` |
| Удаление роли | `0x2FD05` | `0x4FB02` | `0x1FF03` | `0xAF505` |
| Восстановление роли | `0x2FD06` | `0x4FB03` | `0x1FF04` | `0xAF506` |
| Запрос списка миров | `0x2FD0C` | Не требуется для самой клиентской ветви | `UNKNOWN` | `0xAF50B` |
| Matrix / valid-code | `0x2FD08`, `0x2FD09`, `0x2FD0A` | Не устанавливается этой таблицей | Не устанавливается | Ответы зависят от условий, см. Login owner |
| Разрыв / очистка | `0x2FD07` или локальный `0x10001` после TCP close | `0x4FB06`, при иных переходах `0x4FB07` | `0x1FF06` на соответствующей ветви | Не сводится к одному ответу |

Таблица не обещает, что каждая ячейка одной строки всегда исполнится: фильтры account, CD-key, существование подключённого World, DB-результат и фаза соединения могут остановить последовательность раньше. Например, обычный Login нормализует account после проверки длины/версии, расширенный проверяет пароль раньше account и не переводит account в lowercase; поэтому их нельзя объединять только потому, что обе ветви «вход». [Доменный Login owner](../../server/rust/realm/src/access/logmessage.rs). `VERIFIED` для этих достигнутых ветвей; полный автомат состояний клиента `PARTIAL`.

## Передача игрока World → Game

| Тип | Направление | Смысл и порядок |
| --- | --- | --- |
| `0x8F702` | клиент → Game | Вход клиента на игровой сервер. [Game dispatcher](../../server/rust/src/gameserver/appserver/message/logmessage.rs) читает ID, проверяет отсутствие уже загруженного игрока, меняет тип на `0x5FB01`, отправляет World и затем назначает socket→player map ID. Это два упорядоченных side effects, а не один локальный login. `VERIFIED` по Game owner. |
| `0x5FB01` | Game → World | Запрос состояния/деталей игрока; [World dispatcher](../../server/rust/realm/src/app/logmessage.rs) выбирает live map, frozen save map, затем DB и может поставить player-load FIFO. `VERIFIED` для порядка ветвей, payload `PARTIAL`. |
| `0x7F901` | World → Game | Результат загрузки игрока. [Game dispatcher](../../server/rust/src/gameserver/appserver/message/logmessage.rs) трактует положительный status как ID игрока, `0`/`-1` как отказ, `-2` как служебный игнорируемый результат; затем декодирует GameSave и формирует клиентский снимок. `VERIFIED` для достигнутой ветви; полная byte parity GameSave `PARTIAL`. |
| `0xBF401` | Game → клиент | Полный начальный snapshot либо короткий отказ `long(0)`. Точный внешний порядок и доказанный однобайтовый `country_identity` приведены в [спецификации сообщения](game-login.md). Общий layout `PARTIAL`. |
| `0xEF201` | Game → Billing | Запрос баланса: C-строка account, затем `long player_id`. Game ставит его после вызова отправки начального snapshot, **без проверки её результата**. Приём — `BillingMessageHandler::on_account_request`; ответ — `0xFF001`, см. таблицу ниже. |
| `0x7F903`, `0x7F904`, `0x7F905` | World → Game | Исключение игрока и изменение присутствия друзей; [Game dispatcher](../../server/rust/src/gameserver/appserver/message/logmessage.rs) для friend notices переписывает тип на клиентские `0xBF404`/`0xBF405` и отправляет адресно. `VERIFIED` для достигнутых преобразований. |
| `0x6FA01` | локально внутри Game | Событие потери клиентского соединения с map ID и пустой C-строкой, создаётся компонентным `OnClose`; **не считать сетевым пакетом** без отдельного доказательства. [Game receive](../../server/rust/zone/src/app/game_server_client.rs). |

Клиентское `0x8F701` — нижняя граница допустимого диапазона, **не доказанный здесь смысл сообщения**. Семейство `0xBFxxx` — не гарантия одинакового payload; например `0xBF401` имеет две формы. Список типов выше не следует использовать как повод генерировать ответ для неописанного opcode.

## Подтверждение входа в регион

`0x8F801` завершает вход после загрузки карты. [Обработчик региона](../../server/rust/src/gameserver/appserver/message/regionmessage.rs) принимает два доказанных варианта payload: ровно один `i32 entry_token` из оригинального серверного контракта либо C-строку с обязательным NUL и ровно одним `i32 entry_token` после неё из локального клиента. Отдельный разбор нужен из-за различия этих двух артефактов; общий transport префикс не удаляет. `0x8F805` по-прежнему принимает только один `i32` назначения.

`VERIFIED` для форматов: оригинальный `GameServer/gameserver.exe`, `CServerRegion::OnMessage` VA `0x5BB270`, читает аргумент в `0x5BB37B`; [идентифицированный клиент](../status/audit.md#сдвиг-пакета-входа-перед-заданиями) вызывает загрузчик `0x4147D0` после `BF401` в `0x54A9E0`, добавляет C-строку в `0x414CD0` и аргумент в `0x414CDD`. При первоначальном входе аргумент равен `0` (`0x54A9AF`). **Смысл строки — `VERIFIED_DISASSEMBLY: account`**: writer `0x4147D0` берёт `std::string` глобального game-owner-а по `+0x80` (SSO-чтение `0x414CA5..0x414CCB`); клиентские сайты `0x4E54E1..0x4E5517` и `0x4E55E3` (accessor `0x4040D0`) передают ту же строку первым полем `0x2FD09`, а case `0x2FD09` парного `loginserver.exe` (sha256 по манифесту `_loginserver_export_manifest.toml`) по VA `0x48038E` читает это поле как account через `GetStr(0x14)`, чему соответствует текущий Rust Login owner. Машинная база — [клиентский wire-runtime](../reconstruction/client-wire-runtime.md). Сам Game эту строку не использует для выбора игрока или проверки его прав: игрок определяется текущей socket/map-сессией; проверки `EnterTime`, ожидания перехода и текущего региона остаются у [существующей операции входа](../gameplay/regions-and-visibility.md#назначение-обслуживает-тот-же-game).

После размещения Game отправляет вошедшему `0xBF501` с тремя `i32`: `1`, ID региона, `entry_token`, затем погоду и снимки соседей. `VERIFIED`: оригинальный отправитель `0x5BB52E..0x5BB589`; клиентская ветвь `0x55E2F3..0x55E396` переводит игру из состояния `5` в `6`, не читая payload. Отдельного завершающего сообщения после списка соседей в этом пути нет. Порядок публикации и владение объектами описаны в [регионах и видимости](../gameplay/regions-and-visibility.md#сценарий-1-фигура-появляется-и-пересекает-границу-области).

## Локальный переход в регион: `0xBF505`

Game → клиент: [отправитель `change_player_region_with_context`](../../server/rust/src/gameserver/gameserver/game.rs) формирует BF505 при переходе в другой регион того же Game и передаёт его через `send_to_around`. Клиент для собственного игрока читает назначение и запускает загрузку карты; для чужой фигуры после первых трёх полей только помечает найденную фигуру к удалению. Это отдельное сообщение перехода, а не часть обычного [BF401](game-login.md).

`VERIFIED` для порядка и ширины: оригинальный `GameServer/gameserver.exe`, `CPlayer::ChangeRegion` VA `0x44C400`, отправка `0x44C9EE..0x44CACD`; [идентифицированный клиент](../status/audit.md#сдвиг-пакета-входа-перед-заданиями), reader `0x55E935..0x55E9D7`, вызов загрузчика `CGame::4147D0` по `0x55EAA8`. Все `long` занимают четыре байта little-endian; последний `float` передаётся своим 32-битным IEEE-754 представлением.

| Поля в порядке payload | Запись оригинального Game | Чтение клиента |
| --- | --- | --- |
| `long object_type`, `long player_id` | `0x44CA0C`, `0x44CA19` | `0x55E937`, `0x55E940` |
| `long use_goods`, `long target_region_id` | `0x44CA2A`, `0x44CA3B` | `0x55E949`, `0x55E965` |
| `long tile_x`, `long tile_y`, `long direction` | `0x44CA49`, `0x44CA57`, `0x44CA65` | `0x55E96E`, `0x55E977`, `0x55E982` |
| Исходные байты имени региона и NUL | `0x44CA7D` | `0x55E9AC` |
| `long region_type`, `long war_region_type`, `long resource_id` | `0x44CA91`, `0x44CAA1`, `0x44CAAE` | `0x55E9B3`, `0x55E9BE`, `0x55E9C9` |
| `float exp_scale` | `0x44CAC1` | `0x55E9D2` |

`use_goods` — пятый аргумент серверного `ChangeRegion`; клиент передаёт его последним аргументом загрузчику, а ID региона и ResourceID — первым и вторым. Значение ненулевых вариантов `use_goods` за пределами этой передачи здесь `UNKNOWN`. Имя берётся через `get_name()`, без замены на путь `.rgn`; числовой хвост занимает 16 байт и не содержит отдельного `country`. Источники полей региона и причина сохранения битов `expScale` описаны [в BF401](game-login.md#поля-региона-после-player-snapshot). Сквозное выполнение перехода после исправления отправителя здесь не утверждается; результаты запусков фиксируются [в состоянии проекта](../status/audit.md).

## Уведомления свойств игрока

Game → клиент: результат CiQing отправляет `CGame::send_ci_qing_property_result`, полные свойства — `send_player_properties_changed` в [game.rs](../../server/rust/src/gameserver/gameserver/game.rs). Пересчёт сохраняет порядок `0xC010E → 0xBF721 → DoneTaoZhuang`; последняя операция зависит от modify-режима. Жизненный цикл пересчёта описан в [свойствах и состояниях](../gameplay/attributes-and-states.md).

`VERIFIED` для opcode, порядка и ширины полей: оригинальный Game `CPlayer::SendResultToClient` VA `0x44B390` и `OnChangeProperties` VA `0x42C620`, сопоставленные с [идентифицированным клиентским EXE](../status/audit.md#сдвиг-пакета-входа-перед-заданиями). Серверный артефакт — `original/server/Miracle_server/GameServer/gameserver.exe`, SHA-256 `4f5c98e0fdf6147d8aecf55f7937aaf6e2cf5e4f5a2c44491a6359228762c80e`; SHA совпадает с исследованной локальной копией `.exe/gameserver.exe`. Оба EXE имеют ImageBase `0x400000`, RVA получается вычитанием этой базы из VA. Здесь установлены форматы, а не формулы вычисления значений или причина падения клиента.

| Opcode | Payload и свидетельство получателя |
| --- | --- |
| `0xC010E` | `i32 object_type`, `i32 player_id`, затем значения объединённой CiQing/TaoZhuang map в порядке ключей, без ключей и счётчика. Для игрока `object_type = 400`. Клиентская ветвь VA `0x53990E` читает пару и ищет объект по `(type, id)`; после успешного поиска VA `0x53993B..0x539ACE` читает 15 четырёхбайтовых значений. Сервер выбирает opcode в VA `0x44B4FA`, пишет type/id в `0x44B515/0x44B522`, затем значения map в `0x44B539`. |
| `0xC0110` | Отдельное уведомление TaoZhuang: один `i32` с ID комплекта либо `0`. Клиентская ветвь VA `0x539B9A` читает только это число в `0x539B9C`; серверный `ComputerAddValue` VA `0x4585D0` формирует его без пары type/id. |
| `0xBF721` | Фиксированный payload ниже: 108 байт, с внутренним заголовком — 124. Клиентская ветвь VA `0x55245D` читает type/id и ищет объект по паре в `0x55247B`, затем читает свойства. |

Смещения BF721 отсчитываются от payload, после [внутреннего заголовка](message-header.md). Многобайтовые поля little-endian; внутри строк таблицы поля идут подряд в указанном порядке.

| Смещение | Поля Rust | Размер каждого поля |
| --- | --- | --- |
| `0x00` | `object_type`, `player_id` | 4 байта |
| `0x08` | `strength`, `dexterity`, `constitution`, `intelligence`, `minimum_attack`, `maximum_attack`, `add_element_attack`, `element_modify` | 4 байта |
| `0x28` | `attack_speed`, `cch` | 2 байта |
| `0x2C` | `defense`, `element_resistance` | 4 байта |
| `0x34` | `burden` | 2 байта |
| `0x36` | `maximum_hp`, `health`, `maximum_mp`, `mana` | 4 байта |
| `0x46` | `maximum_rp`, `rp`, `reank` | 2 байта |
| `0x4C` | `maximum_vigour`, `vigour`, `credit`, `mode`, `war_soul_state == 1`, `battle_fairy_recall`, `battle_fairy_died`, `exalt` | 4 байта |

Три ранее расширенных Rust-поля подтверждены независимо с обеих сторон: Game вызывает двухбайтовый writer VA `0x4131D0` в `0x42C753` (`burden`), `0x42C7AC` (`maximum_rp`), `0x42C7BD` (`rp`); клиент вызывает двухбайтовый reader VA `0x43B410` в `0x552545`, `0x552592`, `0x5525A1`. Этот reader сдвигает курсор на 2; четырёхбайтовый reader — VA `0x43B440`. Исправление возвращает длину BF721 с 130 к 124 байтам и заменяет ошибочную пару `(id, id)` на `(type, id)`.

## Конфигурация CiQing: `0x8FC30` → `0xC010D`

Клиент запрашивает глобальные рецепты CiQing сообщением `0x8FC30`. После разрешения игрока [goodsmessage](../../server/rust/src/gameserver/appserver/message/goodsmessage.rs) вызывает `CGame::query_ci_qing_setup`: Game сериализует текущие настройки и отправляет игроку `0xC010D`. В этой ветви нет дополнительной проверки открытия страницы CiQing или флага доступности механики. Payload начинается сразу с конфигурации, без ID игрока и отдельной длины блока. Оригинальная отправка находится по VA `0x496D23..0x496D70`; клиентский обработчик `0x539822` передаёт буфер и курсор в decoder `0x4684A0`.

World передаёт Game ту же структуру конфигурации в блоке с selector `0x35`, перед настройками LingBao. После чтения блока [Game servermessage](../../server/rust/src/gameserver/appserver/message/servermessage.rs) также рассылает `0xC010D` подключённым клиентам; оригинальная ветвь — VA `0x49E059..0x49E0F2`. Обе стороны используют [общий `CCiQingSetup`](../../server/rust/src/public/ciqing.rs): `add_byte_to_array` для записи и `de_byte_from_array` для чтения. Поэтому исправление этого формата требует согласованного обновления и перезапуска **World и Game**: старое состояние Game уже могло быть декодировано с перепутанными полями. Совместимость со старым ошибочным Rust-порядком не предусмотрена; различить его по длине пакета нельзя.

`VERIFIED` для порядка и ширины полей: GameServer `CCiQingSetup::AddByteToArray` VA `0x4E4740`, `DeByteFromArray` VA `0x4E6110`, `GameServer.pdb` и клиентский decoder VA `0x4684A0`. У WorldServer подтверждён `Nworldserver.exe + WorldServer.pdb`, writer VA `0x486080`. Точные артефакты и привязки находятся в [комментарии codec](../../server/rust/src/public/ciqing.rs). Все скаляры little-endian; значения полей — `u32`, счётчики списков занимают четыре байта. Rust допускает длины списков в диапазоне неотрицательного `i32`.

| Секция, в порядке передачи | Поля |
| --- | --- |
| Создание | Число записей; для каждой шесть `u32`: `destination_base_index`, `equipment_position`, `source_a_base_index`, `source_a_count`, `source_b_base_index`, `source_b_count`. |
| Объединение | Число записей; каждая запись описана ниже, включая свой список результатов. |
| Улучшение | Число записей; для каждой три `u32`: `level`, `base_index`, `probability`. |

Смещения следующей таблицы отсчитываются от начала **одной записи объединения**. Это смещения в сообщении, а не в PDB-структуре.

| Смещение | Поле | Размер |
| --- | --- | --- |
| `+0x00` | `source_a_base_index` | 4 байта |
| `+0x04` | `source_b_base_index` | 4 байта |
| `+0x08` | `compose_probability` | 4 байта |
| `+0x0C` | `crystal_count` | 4 байта |
| `+0x10` | `money` | 4 байта |
| `+0x14` | `declared_result_count` | 4 байта |
| `+0x18` | Фактическое число пар `results.len()` | 4 байта |
| `+0x1C` | Пары `(probability, result_base_index)` в порядке списка | По 8 байт |

Объявленное число результатов и размер списка — два самостоятельных поля. Reader сохраняет первое без подмены, а число читаемых пар берёт из второго; writer передаёт оба. Это сохраняет конфигурацию даже при их несовпадении и не меняет формулу выбора результата. В `GameServer.pdb`, тип `CCiQingSetup::stComposeNode` (`0xD48C`), объявленное число называется `dwResultNum` и находится по `+0x14`; вектор начинается по `+0x18`. Writer `0x4E4800..0x4E4852` передаёт поля структуры в порядке `0, 4, 8, 0x10, 0x0C, 0x14`, затем размер вектора; Game decoder `0x4E6258..0x4E62B5` и клиент `0x468603..0x468662` читают тот же порядок. Первый и второй индексы материалов не дублируются: их отдельное сравнение видно в Game `GetComputeNode` (`0x4E56B3/0x4E56B7`) и клиенте (`0x4679F4/0x4679F8`).

## Проверка account и расчёты Billing

Пары ниже прослежены по текущим отправителям и получателям. Они не дополняют неизвестные layouts предположениями.

| Запрос → ответ | Действие | Где записан контракт |
| --- | --- | --- |
| `0xCF501` → `0xCF601` | Login → Auth → Login: обычная проверка account с данными корреляции конечного клиента. | [Поля и локальный timeout](login-auth.md); ожидание и дубликаты — [служебные процессы](../server/auth-login-and-services.md). |
| `0xEF201` → `0xFF001` | Game → Billing → Game: баланс. Запрос описан выше; ответ: `long player_id`, `long result`, только при `result == 0` — `long point`. Пустой account в запросе даёт no-op до чтения ID. | [BillingMessageHandler](../../server/rust/realm/src/billing/billingmessage.rs), [CBillingPlayerManager::run](../../server/rust/realm/src/billing/billingplayermanager.rs). |
| `0xEF202` → `0xFF002` | Покупка в магазине; асинхронный запрос передаёт цену, товар, количество и session-контекст. | Те же обработчик и работник; игровой смысл и поля результата — [торговля](../gameplay/trade.md). Полный входной layout этой таблицей не устанавливается. |
| `0xEF203` → `0xFF003` | Расчёт сделки между игроками, включая идентификаторы участников, session/plugin и GUID товара. | Те же обработчик и работник; [торговля](../gameplay/trade.md). Полный входной layout этой таблицей не устанавливается. |

Локальные события accept/close проходят через те же типы сообщений, но не становятся от этого wire-запросами. В частности, Auth `0xCF401/0xCF402` создаются сетевым компонентом, а Login timeout создаёт `0xCF601` внутри процесса. Тип события и его источник нужно учитывать вместе.

## Игровые действия

Клиентские команды движения `0x8F901..0x8F905` и навыков `0x90001..0x90005`, а также их достигнутые ответы подробно описаны в [спецификации игровых действий](game-actions.md). Их нельзя выводить только из допустимого числового диапазона приёма.

## Что остаётся неизвестным

Полный перечень используемых opcode из всех игровых семейств, layout каждого payload, pre-auth handshake (если он есть), версии клиента, а также побайтовая проверка входа исходным клиентом имеют статус `UNKNOWN` или `PARTIAL` в соответствующих местах. Следующий подробный пакет документируется по [шаблону доказательства](../reconstruction/evidence-and-contracts.md): направление → событие → точный layout и типы → side effects/order → EXE/PDB RVA или capture → Rust owner → отдельно помеченные неизвестные поля. Диапазон приёма и имя константы не заменяют эту работу.
