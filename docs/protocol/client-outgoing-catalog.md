# Исходящие команды клиента: матрица покрытия dispatch

Страница собирает инвентарь исходящих сообщений локального клиента и сопоставляет его с точками dispatch текущего Rust-сервера. Источник клиентской стороны — машинный разбор `.exe/Nebokrai.exe` (идентификаторы и пределы метода — [клиентский wire-runtime](../reconstruction/client-wire-runtime.md)); источник серверной стороны — константы и ветви доменных диспетчеров. Это карта покрытия, а не спецификация payload: формат конкретного сообщения остаётся на странице его сценария.

## Метод и его границы

Клиент конструирует каждое исходящее сообщение через `CMessage` ctor VA `0x43AAF0` (тип → слово `+0x04` заголовка) и отправляет через универсальную обёртку VA `0x43AC80`, которая штампует `+0x08`/`+0x0C` заголовка только для типов в [`0x8F700`, `0x9F600`) — гейт относится именно к штампам, не к самой отправке. Инвентарь построен сканированием call-site этого ctor:

- разведка: 368 call-site, 133 уникальных типа;
- независимая повторная проверка (наивный поиск `E8` → `0x43AAF0` по `.text` с извлечением ближайшего `push imm32` в окне ~12 байт): 367 кандидатов, 175 разрешённых immediate-типов.

Обе оценки одного порядка, но наивная экстракция шумит в обе стороны: даёт ложные попадания в середине чужих инструкций (например, якобы-сайт `0x8F702` в `0x401632`, якобы-сайт `0x9F601` в `0x43D69A`) и пропускает реальные сайты с длинной прослойкой между push и call (`0x8F801` в `0x414C8B` ею не найден). Метод также не покрывает типы, переданные не immediate-операндом. Наблюдение разведки о непокрытии ctor-методом типа `0x8F702` — отдельное метод-ограничение: тип является документированной командой входа ([каталог](opcode-catalog.md)), значит как минимум один сайт существует, а найденный наивным сканом якобы-сайт оказался артефактом — шум работает в обе стороны. Нижняя граница оценки, не потолок: полный разбор layout всех типов за рамками этой страницы.

Принятие на серверной стороне: game-канал принимает типы в [`0x8F701`, `0x9F5FF`] ([`game_server_client.rs`](../../server/rust/zone/src/app/game_server_client.rs)).

## Матрица семейств

Счётчики типов/сайтов — наивная повторная проверка (см. шум выше); при расхождении с разведкой указаны обе цифры. «Покрытие» означает наличие доменной ветви в текущем Rust по константам диспетчера, а не полную паритетность payload.

| Семейство | Типов × сайтов | Rust dispatch-точка | Статус покрытия |
| --- | --- | --- | --- |
| `0x2FDxx` (клиент → Login) | 10 × 11 | [`realm/src/access/logmessage.rs`](../../server/rust/realm/src/access/logmessage.rs) | покрыто; `0x2FD07` в ctor-инвентаре отсутствует (cleanup идёт другим путём) |
| `0x8F7xx` | 1 × 1: `0x8F702` | [`gameserver/.../logmessage.rs`](../../server/rust/src/gameserver/appserver/message/logmessage.rs) (`CLIENT_ENTER`) | покрыто; `0x8F701` в инвентаре нет |
| `0x8F8xx` | 2 × 2: `0x8F802`, `0x8F805` | `0x8F805` → [`regionmessage.rs`](../../server/rust/src/gameserver/appserver/message/regionmessage.rs) | `0x8F805` покрыто; **`0x8F802` — owner в Rust не найден** (`UNKNOWN`); реальный сайт `0x8F801` (`0x414C8B`) наивной экстракцией пропущен |
| `0x8F9xx` | 5 × 8: `0x8F901..0x8F905` | [`zone/src/movement/commands.rs`](../../server/rust/zone/src/movement/commands.rs) (бывший `shapemessage`) | покрыто |
| `0x8FAxx` | 18 × 27 | [`playermessage.rs`](../../server/rust/src/gameserver/appserver/message/playermessage.rs) | покрыто по константам |
| `0x8FBxx` | 5 × 15 | [`othermessage.rs`](../../server/rust/src/gameserver/appserver/message/othermessage.rs); `0x8FB02` ×11 пересекается с [`onmsg_c2s_auction.rs`](../../server/rust/src/gameserver/appserver/message/onmsg_c2s_auction.rs) | покрыто |
| `0x8FCxx` | 39 × 63 | [`goodsmessage.rs`](../../server/rust/src/gameserver/appserver/message/goodsmessage.rs) | покрыто; **`0x8FC36` ×1 — owner не найден** (`UNKNOWN`) |
| `0x8FDxx` | 5 × 7 | [`shopmessage.rs`](../../server/rust/src/gameserver/appserver/message/shopmessage.rs) | покрыто |
| `0x8FExx` | 3 × 5 | [`depotmessage.rs`](../../server/rust/src/gameserver/appserver/message/depotmessage.rs) | покрыто |
| `0x8FFxx` | 10 × 19 | [`teammessage.rs`](../../server/rust/src/gameserver/appserver/message/teammessage.rs) | покрыто |
| `0x900xx` | 3 × 8: `0x90001/03/04` | [`skillmessage.rs`](../../server/rust/src/gameserver/appserver/message/skillmessage.rs) | покрыто; `0x90002` и `0x90005` в ctor-инвентаре отсутствуют, хотя обрабатываются — либо конструируются не immediate-типом (граница метода), либо сайта нет (`UNKNOWN`) |
| `0x901xx` | 39 × 49 | [`organsysmessage.rs`](../../server/rust/src/gameserver/appserver/message/organsysmessage.rs) | покрыто: `0x90101`, relay `0x90102..0x90121`, quest `0x90124..0x90128`, honor `0x9012A..0x9012D` |
| `0x902xx` | 8 × 12 | [`playershopmessage.rs`](../../server/rust/src/gameserver/appserver/message/playershopmessage.rs) | покрыто |
| `0x903xx` | 1 × 45 (разведка: 1 × 23): `0x90301` | [`containermessage.rs`](../../server/rust/src/gameserver/appserver/message/containermessage.rs) | покрыто; **самый частый тип инвентаря** — расхождение счётчиков методов зафиксировано, owner-терапия требует разбора семей сайтов |
| `0x904xx` | 2 × 2 | [`petmessage.rs`](../../server/rust/src/gameserver/appserver/message/petmessage.rs) | покрыто |
| `0x905xx` | 9 × 10 | [`countrymessage.rs`](../../server/rust/src/gameserver/appserver/message/countrymessage.rs) | покрыто |
| `0x906xx` | 4 × 15 | [`incrementshopmessage.rs`](../../server/rust/src/gameserver/appserver/message/incrementshopmessage.rs) | покрыто |
| `0x90Axx` | 10 × 25 | [`onmsg_c2s_auction.rs`](../../server/rust/src/gameserver/appserver/message/onmsg_c2s_auction.rs) | покрыто (`0x90A02..0x90A12` из инвентаря) |
| `0x9F6xx` | 1 × 1: `0x9F601` | owner не найден | **без покрытия**; единственный наивный сайт — артефакт в середине инструкции (`0x43D69A`), а настоящий `0x9F601` попал бы и за предел гейта штампов клиента, и за приёмный максимум сервера `0x9F5FF`; скорее всего недостижим, статус `UNKNOWN` |

## Несоответствия для полного анализа

1. **`0x9F601`** — приграничный тип: клиентский гейт штампов заканчивается на `0x9F600` (исключительно), серверный приём — на `0x9F5FF`; единственный кандидат-сайт вероятно артефакт сканера. Граничное смещение диапазонов `[0x8F700,0x9F600)` клиента против `[0x8F701,0x9F5FF]` сервера само по себе вред нейтрален для достигнутых типов и здесь только фиксируется.
2. **`0x900xx`-кластер отличий**: `0x90002`/`0x90005` обрабатываются Rust, но не видны ctor-инвентарю — место их клиентской конструкции не установлено.
3. **`0x90301` ×23/45 сайтов** — доминирующая масса исходящих вызовов; потребуется разложение сайтов по UI-семьям прежде чем закрывать owner-контракт `containermessage`.
4. **`0x90Axx` ×25 сайтов** — все в аукционный диспетчер; инвентарь типов неполон относительно констант владельца (разница — метод-шум или мёртвые ветви).
5. Типы без найденного owner: **`0x8F802`, `0x8FC36`** (по одному сайту) — либо необработанные в Rust, либо сканер-шум; подтверждения одиночной отправкой у оригинала нет.

## Где вход и где обработка

Клиентский кадр приходит в [`zone/app/game_server_client.rs`](../../server/rust/zone/src/app/game_server_client.rs) (envelope, CRC, RLE, диапазон типов), попадает в netserver FIFO и далее в доменный диспетчер [`src/gameserver/appserver/message/`](../../server/rust/src/gameserver/appserver/message/) либо [`zone/src/movement/`](../../server/rust/zone/src/movement/) по таблице выше. Порядок очередей и циклы — в [сетевом runtime](../server/network-runtime.md) и [серверных циклах](../server/runtime-ordering.md).
