# Каталог сообщений и границы покрытия

`MsgType` — полное 32-битное слово по смещению `+0x04` [внутреннего заголовка](message-header.md). Направление, допустимый диапазон, внешнее [TCP-обрамление](transport.md) и назначение конкретного типа нужно читать вместе: совпадение младших байт или имени обработчика само по себе не доказывает одинаковый layout. Эта страница фиксирует **достигнутые и сопоставленные** типы входа и перехода между Login, World и Game, а не объявляет полным каталогом всей игры. Значения и роли взяты из точных EXE/PDB, указанных в owner-комментариях [Login](../../server/rust/src/loginserver/applogin/message/logmessage.rs), [World](../../server/rust/src/worldserver/appworld/message/logmessage.rs) и [Game](../../server/rust/src/gameserver/appserver/message/logmessage.rs); это `VERIFIED` для достигнутых ветвей оригинала, но не runtime-проверка новой реализации.

## Приём и маршрутизация

| Точка входа | Фильтр и правило | Доказательство и предел |
| --- | --- | --- |
| Игровой клиент → LoginServer | `0x2FD01..=0x3FBFF` после проверки кадра; тип и socket/CD-key/IP попадают в FIFO | [Receive owner](../../server/rust/src/nets/netlogin/mynetserverclient_client.rs), точная Login EXE/PDB. Диапазон `VERIFIED`, обработчики не для каждого числа в нём — `PARTIAL`. |
| Игровой клиент → GameServer | `0x8F701..=0x9F5FF` после проверки кадра; тип и socket/map/IP попадают в FIFO | [Receive owner](../../server/rust/src/nets/netserver/myserverclient.rs), Game `OnReceive` RVA `0x0001C7F0`. Диапазон `VERIFIED`, полное покрытие содержимого `PARTIAL`. |
| Внутри LoginServer | `Run` сначала проверяет диапазон Auth `0xCF301..0xDF1FE`, затем семейство `MsgType & 0xFFFFFF00`: GM `0x20000`, GMA `0x20100`, Log `0x1FF00/0x2FD00/0x10000`, Server `0xFF00/0x1FE00` | [Login message](../../server/rust/src/nets/netlogin/message.rs), точная Login EXE/PDB. Неизвестный тип — no-op с возвратом `1`; это не доказательство успеха доменной операции. |
| Внутри GameServer | `Run` разрешает игрока по числовому map ID, его регион, затем выбирает обработчик по `MsgType & 0xFFFFFF00`; некоторые семейства требуют оба объекта | [Game message](../../server/rust/src/nets/netserver/message.rs), RVA `0x000149D0`. Семейства перечислены в owner-е; их маршрутизация `VERIFIED`, полнота вложенных handlers `PARTIAL`. |

## Вход клиента и обслуживание роли через Login и World

Первичный источник для клиентских значений и клиентских ответов — [Login `OnLogMessage`](../../server/rust/src/loginserver/applogin/message/logmessage.rs); для межсерверных запросов и ответов — [Login `CGame`](../../server/rust/src/loginserver/loginserver/game.rs) и [World `OnLogMessage`](../../server/rust/src/worldserver/appworld/message/logmessage.rs). Статус таблицы — `VERIFIED` для выбора ветви и указанного направления; payload каждого типа требует отдельного анализа и остаётся `PARTIAL`, если нет специальной страницы.

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

Таблица не обещает, что каждая ячейка одной строки всегда исполнится: фильтры account, CD-key, существование подключённого World, DB-результат и фаза соединения могут остановить последовательность раньше. Например, обычный Login нормализует account после проверки длины/версии, расширенный проверяет пароль раньше account и не переводит account в lowercase; поэтому их нельзя объединять только потому, что обе ветви «вход». [Доменный Login owner](../../server/rust/src/loginserver/applogin/message/logmessage.rs). `VERIFIED` для этих достигнутых ветвей; полный автомат состояний клиента `PARTIAL`.

## Передача игрока World → Game

| Тип | Направление | Смысл и порядок |
| --- | --- | --- |
| `0x8F702` | клиент → Game | Вход клиента на игровой сервер. [Game dispatcher](../../server/rust/src/gameserver/appserver/message/logmessage.rs) читает ID, проверяет отсутствие уже загруженного игрока, меняет тип на `0x5FB01`, отправляет World и затем назначает socket→player map ID. Это два упорядоченных side effects, а не один локальный login. `VERIFIED` по Game owner. |
| `0x5FB01` | Game → World | Запрос состояния/деталей игрока; [World dispatcher](../../server/rust/src/worldserver/appworld/message/logmessage.rs) выбирает live map, frozen save map, затем DB и может поставить player-load FIFO. `VERIFIED` для порядка ветвей, payload `PARTIAL`. |
| `0x7F901` | World → Game | Результат загрузки игрока. [Game dispatcher](../../server/rust/src/gameserver/appserver/message/logmessage.rs) трактует положительный status как ID игрока, `0`/`-1` как отказ, `-2` как служебный игнорируемый результат; затем декодирует GameSave и формирует клиентский снимок. `VERIFIED` для достигнутой ветви; полная byte parity GameSave `PARTIAL`. |
| `0xBF401` | Game → клиент | Полный начальный snapshot либо короткий отказ `long(0)`. Точный внешний порядок и доказанный однобайтовый `country_identity` приведены в [спецификации сообщения](game-login.md). Общий layout `PARTIAL`. |
| `0xEF201` | Game → Billing | Отдельное уведомление после успешной отправки начального клиентского snapshot согласно [Game owner](../../server/rust/src/gameserver/gameserver/game.rs). Точный payload и обработка Billing в этой странице `UNKNOWN`. |
| `0x7F903`, `0x7F904`, `0x7F905` | World → Game | Исключение игрока и изменение присутствия друзей; [Game dispatcher](../../server/rust/src/gameserver/appserver/message/logmessage.rs) для friend notices переписывает тип на клиентские `0xBF404`/`0xBF405` и отправляет адресно. `VERIFIED` для достигнутых преобразований. |
| `0x6FA01` | локально внутри Game | Событие потери клиентского соединения с map ID и пустой C-строкой, создаётся компонентным `OnClose`; **не считать сетевым пакетом** без отдельного доказательства. [Game receive](../../server/rust/src/nets/netserver/myserverclient.rs). |

Клиентское `0x8F701` — нижняя граница допустимого диапазона, **не доказанный здесь смысл сообщения**. Семейство `0xBFxxx` — не гарантия одинакового payload; например `0xBF401` имеет две формы. Список типов выше не следует использовать как повод генерировать ответ для неописанного opcode.

## Что остаётся неизвестным

Полный перечень используемых opcode из всех игровых семейств, layout каждого payload, pre-auth handshake (если он есть), версии клиента, а также побайтовая проверка входа исходным клиентом имеют статус `UNKNOWN` или `PARTIAL` в соответствующих местах. Следующий подробный пакет документируется по [шаблону доказательства](../reconstruction/evidence-and-contracts.md): направление → событие → точный layout и типы → side effects/order → EXE/PDB RVA или capture → Rust owner → отдельно помеченные неизвестные поля. Диапазон приёма и имя константы не заменяют эту работу.
