# Игровой вход: сообщение `0xBF401`

`0xBF401` отправляет **GameServer клиенту конкретного игрока** при завершении входа, а также в короткой ветви отказа. WorldServer передаёт GameServer результат `0x7F901`, после которого GameServer строит этот клиентский ответ.

## Свидетельства и область действия

Основной вариант — `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`, исходный owner `gameserver/gameserver.cpp`, вызов `CGame::OnLogMessage` по записанному в [игровом owner-е](../../server/rust/src/gameserver/gameserver/game.rs) адресу **VA `0x0049F140`**. Его **RVA `0x0009F140` вычислен** из указанного VA при базе образа `0x00400000`, а не записан в owner-е отдельно. Парный сериализатор игрока — `CPlayer::AddToByteArray_ForClient` по [сохранённому RAW-разбору](../../server/rust/src/gameserver/appserver/player.rs) **VA `0x0044A480`, RVA `0x0004A480`**, исходный `appserver/player.cpp:941`. База сериализации `CMoveShape::AddToByteArray_ForClient` отмечена как VA `0x004CDD30` в [её owner-е](../../server/rust/src/gameserver/appserver/moveshape.rs). Размеры базовых операций `CBaseMessage` и заголовок описаны [отдельно](message-header.md).

Порядок внешних блоков ниже подтверждён статически сохранённой семантикой EXE/PDB в этих owner-ах и последовательностью [Rust-сборки сообщения](../../server/rust/src/gameserver/gameserver/game.rs). Для блока игрока и двух C-строк длина зависит от данных, поэтому **общей таблицы абсолютных offset-ов payload нет**. Полная побайтовая сверка всех вложенных контейнеров с исходным клиентом не проведена; статус всей спецификации — `PARTIAL`.

## Условия отправки

[Диспетчер `logmessage`](../../server/rust/src/gameserver/appserver/message/logmessage.rs) принимает `0x7F901` от WorldServer. Положительный status служит ID игрока; требуется действующий client-route. Далее декодируется GameSave, проверяются ID и отсутствие дубликата, подбирается регион, создаётся player snapshot. После успешной сборки snapshot [GameServer](../../server/rust/src/gameserver/gameserver/game.rs) формирует полный `0xBF401` и вызывает `send_to_player`. Перед этим выполняются login-script и выбор начального навыка. Затем ставится запрос баланса Billing `0xEF201` — без проверки результата `send_to_player`. Это порядок постановки, а не подтверждение доставки клиенту; поля баланса описаны в [каталоге opcode](opcode-catalog.md). Неустановленные реакции исходного кода на остальные состояния входа — `UNKNOWN`.

При status `0` или `-1`, а также при ряде ошибок дальнейшего входа [ветвь отказа](../../server/rust/src/gameserver/appserver/message/logmessage.rs) создаёт тот же тип `0xBF401`, записывает **один 32-битный ноль** и отправляет его игроку. Этот короткий вариант нельзя декодировать как полный login snapshot. Status `-2` игнорируется и сам по себе `0xBF401` не создаёт. Для ветвей отказа имеется реализованный Rust-маршрут и owner-контракт; точный оригинальный byte capture для них здесь не представлен (`PARTIAL`).

## Порядок данных полного варианта

После [16-байтового внутреннего заголовка](message-header.md), в котором `MsgType = 0x000BF401`, идёт следующая последовательность. Числа `long` здесь 32-битные little-endian; `byte` занимает один байт. Отправка одному игроку проходит через [Game `CMessage::send_to_player`](../../server/rust/src/nets/netserver/message.rs), который обрамляет RLE-представление внутреннего сообщения отдельной 32-битной длиной транспортного кадра. Таблица описывает **внутренний payload до RLE**.

| № | Часть payload | Размер и условие | Статус и свидетельство |
| ---: | --- | --- | --- |
| 1 | ID игрока | `long`, 4 байта | `VERIFIED`: `CGame::OnLogMessage` VA `0x0049F140`; `initial.add_long(expected_player_id)` в [Game owner-е](../../server/rust/src/gameserver/gameserver/game.rs). |
| 2 | Разреженная проекция `tagSetup` и соседних настроек | Переменный размер из-за двух C-строк; точный порядок источников ниже | `VERIFIED` для указанного порядка в [GlobeSetup owner-е](../../server/rust/src/setup/globesetup.rs). Назначение каждого сырого байта внутри диапазонов не выводится из этой страницы (`UNKNOWN`). |
| 3 | `CPlayer::AddToByteArray_ForClient(true)` | Переменный блок, без отдельной длины перед ним в `0xBF401` | `PARTIAL`: оригинальный serializer RVA `0x0004A480`, реализованный [player snapshot](../../server/rust/src/gameserver/appserver/player.rs). Отдельные доказанные поля описаны ниже; весь вложенный layout здесь не заявлен. |
| 4 | Имя региона | C-строка до первого NUL, затем `0x00` | `VERIFIED` для [порядка записи](../../server/rust/src/gameserver/gameserver/game.rs); код берёт байты `region.name`. Кодировка текста этой страницей не устанавливается (`UNKNOWN`). |
| 5–8 | `region_type`, `war_region_type`, ширина и высота региона | Четыре последовательных `long`, по 4 байта | `VERIFIED`: четыре `add_long` после имени в [Game owner-е](../../server/rust/src/gameserver/gameserver/game.rs). |
| 9 | Признак первого входа | `byte`, 1 байт: `0` или `1` | `VERIFIED`: `add_byte(u8::from(first_login))`; флаг получают при первом `mark_login_script_started`. |
| 10 | Дублирующие регионы | Только если первый вход и существует setup-owner; знаковый 32-битный счётчик, затем по два 32-битных ID на запись | `VERIFIED` для этой условной формы в [CDupliRegionSetup](../../server/rust/src/public/dupliregionsetup.rs); обязательность присутствия owner-а при любом запуске не утверждается. |
| 11 | Расширение `tagSetup` | `AddEx`: 32-битная длина `0x400`, затем ровно `0x400` байт из setup offset `0xD08..0x1108` | `VERIFIED` для вызова и длины: [Game owner](../../server/rust/src/gameserver/gameserver/game.rs), [GlobeSetup](../../server/rust/src/setup/globesetup.rs), [базовый `AddEx`](../../server/rust/src/nets/basemessage.rs). Семантика всех 1024 байт здесь не заявлена. |

## Проекция `tagSetup` перед блоком игрока

[GlobeSetup](../../server/rust/src/setup/globesetup.rs) пишет исходные байты **в указанном порядке**, а не сплошной `tagSetup`. Таблица фиксирует только доказанное копирование и размер, не придумывая названия каждому полю исходной структуры.

| Порядок | Источник | Что записывается |
| ---: | --- | --- |
| 1–6 | `0x000..0x084`, `0x424..0x428`, `0x3F0..0x3F2`, `0x3F2..0x3F4`, `0x4F4..0x4F6`, `0x2EC..0x2F8` | Последовательно `0x84`, 4, 2, 2, 2 и 12 сырых байт. |
| 7–10 | `0x7B0..0x7F0`, `0x7F0..0x7F8`, `0x768..0x7A8`, `0x7A8..0x7B0` | C-строка из 64-байтного поля до первого NUL (либо всё поле) плюс завершающий NUL; 8 сырых байт; вторая такая строка; ещё 8 сырых байт. |
| 11–14 | Два `contribute_combat_levels`, `tagSetup[0x905]`, `tagSetup[0xCD1]` | По одному младшему байту каждого combat level, затем два одиночных байта setup. |
| 15–16 | `tagSetup[0x514..0x518]`, `tagSetup[0x518..0x51C]` | По четыре байта ширины и высоты area без перестановки. |

Этот порядок имеет статус `VERIFIED` по сохранённому owner-контракту и непосредственной записи Rust. Поля, для которых исходные имена/назначения здесь не приведены, остаются семантически `UNKNOWN`; нельзя считать их свободным padding.

## `country_identity` внутри блока игрока

Полный player snapshot сначала пишет [снимок `CMoveShape`](../../server/rust/src/gameserver/appserver/moveshape.rs), затем 0x194 байта базовых свойств и переменные строки, навыки, друзей, контейнеры, валюты, организации и задания. Ближе к концу, **после quest snapshot**, идут: `country` (`byte`), `contribution` (`long`), **`country_identity` (`byte`)**, `loan_time_limit` (`DWORD`). Затем следуют контейнеры боевой феи/CiQing и завершающие поля. Поэтому абсолютный offset `country_identity` от начала `0xBF401` без разбора всех предшествующих переменных частей неизвестен.

Ширина `country_identity` — **ровно один байт**, статус `VERIFIED`: в RAW оригинального `CPlayer::AddToByteArray_ForClient` (RVA `0x0004A480`) результат `get_country_identity` имеет тип `uchar uVar3` и передаётся `_AddToByteArray`; в [Rust player owner-е](../../server/rust/src/gameserver/appserver/player.rs) параметр `u8` записывается `write_u8(country_identity)`. В [Game owner-е](../../server/rust/src/gameserver/gameserver/game.rs) поиск identity также возвращает `u8` через [country owner](../../server/rust/src/gameserver/appserver/country/country.rs). Четырёхбайтовые ID игроков внутри состояния страны и соседний `contribution` не расширяют это wire-поле до `long`.

## Предел проверки

Направление, внешний порядок блоков, перечисленные размеры и один байт `country_identity` имеют конкретные статические свидетельства. Полный вложенный формат player snapshot, значения непрозрачных setup-диапазонов, текстовая кодировка имени региона и реакция исходного клиента на повреждённый пакет остаются `PARTIAL`/`UNKNOWN` в названных местах.
