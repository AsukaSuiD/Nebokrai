# Realm services: машинная доказательная база

Машинные точки служб, которые обслуживает Realm (Auth, Login, World, Billing, Misc): конструкторы, receive/send-пути, диспетчеры и lifecycle-функции. Сведено из верхних комментариев `server/rust/realm/src/` при нормализации production source; факты относятся к текущей реализации, чьи файлы указаны обратными ссылками `realm::<домен>::<файл>`. Правила статусов и достаточности оснований — в [evidence-and-contracts.md](evidence-and-contracts.md).

Если у таблицы или раздела не сказано иное, адреса — VA первой секции (`.text`) по дизассемблеру точной пары EXE/PDB из блока «Идентификаторы сборок», а статус — VERIFIED на уровне наблюдения машинных инструкций этой пары. Записи `1:xxxxxxxx` — секция:offset из S_PUB32 соответствующего PDB.

## Идентификаторы сборок

Realm-направления воспроизводят пять серверных сборок. Пары EXE/PDB ниже собраны из шапок владельцев `realm/src/app/`; условное сокращение `SHA-256 F3AC454D…` в соседних файлах относится к WorldServer из этой таблицы.

| Служба | EXE (SHA-256) | ImageBase / PE timestamp | PDB (CodeView RSDS) | Соответствие |
| --- | --- | --- | --- | --- |
| AuthServer | `.exe/authserver.exe` `AE0022429C135553092364F01838FA6EF8E631D558C96278123FF3ADE6AD3B15` | `0x400000` / `0x53A26AB9` | `.exe/authserver.pdb` GUID `938C8E25-5ACD-4441-A197-1CED0D3E1B87` age 1 | RSDS match |
| BillingServer | `.exe/billingserver.exe` `FA32E3C043CB49965686129696A4EB34B733ACA1D60CAF57D369F97D5E68FB19` | `0x400000` / `0x53ABDA0D` | `.exe/billingserver.pdb` GUID `CAFACA76-74E6-4ED3-981C-D2963EEB2C3C` age 1 | RSDS match |
| LoginServer | `.exe/loginserver.exe` `1C84006DF612053B007D69E0243497A8DA85E10FB1D825D0B462F016747E7876` | `0x400000` / `0x53E0C388` | `.exe/loginserver.pdb` GUID `48D4B1F2-97BB-4CF8-B9EA-B13F7E2F9645` age 1 | RSDS match |
| MiscServer | `.exe/miscserver.exe` `F4426942465E6E9D1397EEF7A977B87D0D8C5B12957832770F57656F998AED65` | `0x400000` / `0x53A26D7D` | `.exe/miscserver.pdb` GUID `FE6CDEF1-D110-4C25-ABE9-F66D5E665F3B` age 1 | RSDS match |
| WorldServer | `.exe/Nworldserver.exe` `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1` | `0x400000` / `0x53FB128F` | `.exe/WorldServer.pdb` GUID `289F1FB3-96A0-4FF4-8B5D-1FD17B50B751` age 1 | RSDS match |

Канонические экспортные манифесты сборок хранятся в `server/rust/src/manifest/` (`_authserver_`, `_billingserver_`, `_loginserver_`, `_miscserver_`, `_worldserver_` и соседние `_gameserver_`/`_serverupdate_export_manifest.toml`). Там, где шапка ссылается на манифест как источник идентификаторов (например, `realm::access::game` — на `_loginserver_export_manifest.toml`, `realm::persistence::savedb`/`saveworker` — на `_worldserver_export_manifest.toml`), основанием является манифест, а не повторный разбор EXE.

## Auth: сообщение и диспетчер

Реализация: `realm::app::auth_message`. Пара — AuthServer из блока идентификаторов.

| Функция / символ | Адрес (VA) | Суть факта |
| --- | --- | --- |
| ctor `CMessage` | `0x4136B0` | `MsgType` в header-слово `+4`, rigid runtime-поля обнулены, вложенная SSO std::string CD-key (capacity `0xF`), vtable `0x42E2BC`, объект `0x44` байт |
| `Run` | `0x413710` | Поиск ПОЛНОГО 32-битного opcode в статической map `0x43B260`: найденный вызывается, неизвестный — no-op; возврат всегда `1` |
| `InitMsgFuncPool` | `0x4141A0` | Десять записей: LSConnect `0x416900`, LSDisconnect `0x416AD0`, AuthAccount `0x416550`, AuthAccountEx `0x416720`, LSGetInfo `0x416BD0`, ResponseUpdateServerInfo `0x416CD0`, `0x10F101` → `0x4117E0` (тело из одного `ret`, OnMSG_GMGetInfo), GmKickPlayer `0x416010`, ResponseKickPlayer `0x416220`, LockAccount `0x4163F0`; завершает `mov al,1` |
| `SendToLogin` | `0x413830` | Sender через accessor `0x402080`; критической секции в теле нет (ни Enter, ни Leave); envelope helper `0x4128B0` (`total_len = len + 0xC`, scratch `0x43B20C`), два общих `DataCrc32 0x415F80`, виртуальный send `[vtable+0x38]` |
| `CreateMessage` (RLE) | `0x412800` | Собран у класса, но в живом receive-path направления не вызывается: `OnReceive 0x415C10` принимает только несжатый create |
| `CreateMessageWithoutRLE` | `0x413760` | Копирует четыре слова header и нормализует длину без отдельной ветви `len < 16` — класс повреждённого wire |
| recv-tick `+0x40` | ctor | Обнуляется конструктором и нигде в направлении не переписывается (в отличие от World, где `timeGetTime` писался на create-пути); живого читателя поля нет |

## Auth: принятое Login-соединение

Реализация: `realm::app::auth_server_client`.

| Функция / символ | Адрес (VA) | Суть факта |
| --- | --- | --- |
| ctor `CMyNetServerClient_Auth` | `0x415A10` | Base `0x4147E0`, receive buffer ровно `0xA00000`, vtable `0x42E470`, объект `0xC8` (см. `CreateServerClient 0x4127B0`) |
| `OnAccept` | `0x415AD0` | Clear byte `[+0xAC]` (close flag), `new(0x44)` `CMessage(0xCF401)`, контекст `[client+0x2C]→[+0x20]` IPv4, `[client+0x34]→[+0x1C]` socket, публикация в owner `+0xDC` через push helper `0x40D9F0` |
| `OnClose` | `0x415B60` | `CMessage(0xCF402)` с тем же socket/IP контекстом, публикация до общего base close `0x413290(0)` |
| `OnReceive` | `0x415C10` | Gate owner `+0xA8`; цикл при накоплении `>= 0xC`; CRC длины через общий `DataCrc32 0x415F80` (mismatch → обнуление accumulator), `declared > size` — останов без потери хвоста, create только `CreateMessageWithoutRLE 0x413760` (без аргумента времени — recv-tick остаётся нулём ctor), повторный CRC содержимого; контекст `[+0x34]→[+0x1C]` socket, `[+0x88]→[+0x18]` map, byte-string `[+0x90]→[+0x24]` CD-key, `[+0x2C]→[+0x20]` IPv4; публикация owner `+0xDC` через `0x40D9F0`; consume `sub size, declared`; shrink к `0x100000` (условие: capacity `[+0x64] > 0x100000` и size `≤ 0x100000`); reject очищает accumulator без отката опубликованного |
| ctor `CMyNetServer_Auth` | `0x412770` | Базовый `CServer` ctor `0x4118B0`, vtable `0x42E178`, поле `+0x120 = 0`, limits `+0x14C = 0x64` (100) и `+0x150 = 0x1000000` — точные константы component; реализация listener — `realm::app::auth_server` |
| `CreateServerClient` | `0x4127B0` | `new` объекта `0xC8` байт, ctor принятого `CMyNetServerClient_Auth 0x415A10` с владельцем `this` |

## Login: сообщение и направления

Реализация: `realm::app::login_message`. Пара — LoginServer из блока идентификаторов.

| Функция / символ | Адрес (VA) | Суть факта |
| --- | --- | --- |
| ctor `CMessage` | `0x465540` | Base `0x466360`, vtable `0x49CFBC`, вложенная std::string CD-key по `+0x2C` (capacity `0xF`), header `+4` = type |
| `Run` | `0x465490` | Сначала полный диапазон Auth (`cmp 0xCF300; jbe` / `cmp 0xDF1FF; jae` → handler `0x47F320` OnASMessage, возврат 1), затем маска `type - (type & 0xFF)`: `0xFF00/0x1FE00` → `0x480850` OnServerMessage, `0x1FF00/0x10000/0x2FD00` → `0x47F3F0` OnLogMessage, `0x20000` → `0x47F0A0` OnGMMessage, `0x20100` → `0x47F2A0` OnGMAMessage; любая ветвь, включая неизвестный тип, возвращает 1 |
| `SendToAS` | `0x4653E0` | Sender `g_Game+0x3E4`, **CRITICAL_SECTION `0x5E4864` вокруг build/CRC/send** — единственный send-владелец с lock; envelope helper `0x4658F0` (`total_len = len + 0xC`, scratch `0x5E4894`), два общих `DataCrc32 0x47F050` |
| `SendToWorldSocket` | `0x465230` | Во всём теле без Enter/LeaveCriticalSection — World/client send-методы lock не имеют (scratch тот же `0x5E4894`) |
| client send (socket / cdkey) | `0x465190` / `0x4651E0` | Полный внутренний буфер кодируется append/encode helper `0x4659A0`, новая длина `rle_len + 4` пишется первым dword scratch `0x5E4898`, затем виртуальные ветки `[+0x38]`/`[+0x40]`; без CS и без второго CRC-слоя |
| `CreateMessage` | `0x4655C0` | CS `0x5E487C` на decode/create/free (общий static scratch `0x4E4860`/`0x5E4860`); порог `cmp len,0x20000; jbe` — короче `0x20001` → capacity `0x100000`, больше → `len*8`; failure decode `0x465B30` → null |
| `CreateMessageWithoutRLE` | `0x465710` | Тот же несжатый 16-байтовый путь без отдельной проверки `len < 16` — класс повреждённого wire |
| `GetString` | `0x4657D0` | SSO-assign исходной C-строки с движением курсора |

## Login: исходящее Auth-соединение

Реализация: `realm::app::login_auth_client`.

| Функция / символ | Адрес (VA) | Суть факта |
| --- | --- | --- |
| ctor `CMyNetClientAuth` | `0x46CAC0` | Базовый `CClient` ctor `0x46AED0`, vtable `0x49D860`; состояния поверх `CClient` не добавлял |
| `HandleClose` (виртуальный слот) | `0x46CAF0` | `new(0x48)` `CMessage(0xCF301)` через ctor `0x465540`, публикация в FIFO `+0x100` через push helper `0x46AE90`; пустое synthetic сообщение, как у World и Misc; явный `Close` synthetic сообщения не создавал |
| `OnReceive` | `0x46CB90` | Один вызов читает не более `0x2800`; recv-ошибка (`!= WSAEWOULDBLOCK 0x2733`) и `recv == 0` — только журнал без публикации; цикл frames при накоплении `>= 0xC`: CRC длины `0x47F050`, `declared > size` — останов без потери хвоста, create только несжатым `0x465710`, повторный CRC содержимого, публикация в ту же FIFO `+0x100` через `0x46AE90`; shrink к `0x100000` при возврате под лимит; reject очищает accumulator без отката опубликованного |

## Login: принятое клиентское соединение

Реализация: `realm::app::login_server_client`.

| Функция / символ | Адрес (VA) | Суть факта |
| --- | --- | --- |
| ctor `CMyNetServerClient_Client` | `0x46E590` | Base `0x46F160`, vtable `0x49DB10`, receive buffer ровно `0x5000` (`push 0x5000` + alloc), два компаньона `0xC800` в `+0x70/+0x74` (исходный send accumulator) |
| `OnReceive` | `0x46E810` | Gate owner `+0xA8`; цикл при накоплении `>= 0xC`; при флаге `[owner+0x10C]` сначала предел полной длины (`declared > [owner+0x118]` → виртуальное `OnTotalMessageSizeOver [vtable+0x40]`), затем CRC длины `0x47F050`; `declared > size` — останов без потери хвоста; при флаге `[owner+0x10D]` — CRC по сжатым байтам `[+0xC, declared-0xC]`; create только RLE `0x4655C0`; opcode допустим в `(0x2FD00, 0x3FC00)` exclusive; контекст `[+0x34]→[+0x24]` socket, `[+0x88]→[+0x20]` map, byte-string `[+0x90]→[+0x2C]` CD-key, `[+0x2C]→[+0x28]` IPv4; publish через push helper `0x46AE90`; consume; shrink к `0x100000`; reject очищает accumulator без отката опубликованного |
| ban-реакции | `AddForbidIP 0x469390`, `QUIT 0x466730` | Четыре пути: предел длины, length CRC (`0x46EA79`), content CRC (`0x46EAD4`), opcode вне диапазона (`0x46EB1F`, плюс deleting dtor `[edx]` с `push 1`); create-null — без ban (доказанное отличие ветки) |
| `OnClose` | `0x46E660` | При пустом CD-key — ни публикации, ни вызова общего close (`je` сразу в эпилог); при непустом — `new(0x48)` `CMessage(0x10001)` через ctor `0x465540`, `Add` CD-key с NUL, publish owner `+0xDC` через `0x46AE90`, общий close `0x46C740(0)`; фактическое удаление соединения — у общего `CServer` |

## Login: принятое World-соединение

Реализация: `realm::app::login_world_server_client`.

| Функция / символ | Адрес (VA) | Суть факта |
| --- | --- | --- |
| ctor `CMyNetServerClient_World` | `0x46EC60` | Base `0x46F160`, vtable `0x49DC98`, receive buffer ровно `0xA00000` (`push 0xA00000` + alloc), send-компаньон `0x100000` (`+0x70` family) |
| `OnReceive` | `0x46DDF0` | Цикл при накоплении `>= 0xC`; CRC длины через `DataCrc32 0x47F050`, `declared > size` — останов без потери хвоста, create через `CreateMessageWithoutRLE 0x465710`, повторный CRC содержимого; контекст `[+0x34]→[+0x24]` socket, `[+0x88]→[+0x20]` map, byte-string `[+0x90]→[+0x2C]` CD-key (assign `0x401440`), `[+0x2C]→[+0x28]` IPv4; publish owner `+0xDC` через `0x46AE90`; consume; shrink к `0x100000`; reject очищает accumulator без отката; IP-ban и `QUIT` у исходника нет |
| `OnClose` | `0x46ED10` | `new(0x48)` `CMessage(0xFF01)` через ctor `0x465540`, `[client+0x88]→[+0x20]` (map identity), publish owner `+0xDC`, затем общий close `0x46C740(0)` — безусловно, всегда |

## Login: listeners

Реализация: `realm::app::login_server` (игровые клиенты), `realm::app::login_world_server` (World-соединения).

| Функция / символ | Адрес (VA) | Суть факта |
| --- | --- | --- |
| ctor `CMyNetServer_Client` | `0x46A870` | Базовый `CServer` ctor `0x46A480`, vtable `0x49D3B0`, поле `+0x120 = 0`, limits `+0x14C = 5` и `+0x150 = 0x400000` — точные константы component |
| `CreateServerClient` (client) | `0x46A8B0` | `new` объекта `0xC8` байт, ctor принятого `CMyNetServerClient_Client 0x46E590` с владельцем `this` |
| `OnMapStrError` | `0x46AA10` | Публикует то же synthetic сообщение `0x10001` с NUL-terminated строковой identity, что и disconnect клиента |
| ctor `CMyNetServer_World` | `0x46A9B0` | Базовый `CServer` ctor `0x46A480`, vtable `0x49D408`, поле `+0x120 = 0`, limits `+0x14C = 0x64` (100) и `+0x150 = 0x1000000` — точные константы component |
| `CreateServerClient` (world) | `0x46A9F0` | `new` объекта `0xC8` байт, ctor принятого `CMyNetServerClient_World 0x46EC60` с владельцем `this` |

## Billing: сообщение

Реализация: `realm::app::billing_message`. Пара — BillingServer из блока идентификаторов.

| Функция / символ | Адрес (VA) | Суть факта |
| --- | --- | --- |
| ctor `CMessage` | `0x40F860` | `MsgType` в header-слово `+4`, обнуление пяти dword runtime-полей `+0x18..+0x28`, пустая вложенная std::string CD-key по `+0x2C` (capacity `0xF`), vtable `0x42E040` |
| `CreateMessage` | `0x40F8E0` | Весь decode/create/copy/free закрыт `CRITICAL_SECTION 0x457958` (защищаемое состояние — общий static scratch: статический `0x457980`, выделяемый `0x557980`); порог `cmp len,0x20000; jbe` — capacity `0x100000`, больше → `len*8`; failure decode `0x40CE30` → null |
| `SendToGS` / `SendToAllGS` | `0x40F6F0` / `0x40F780` | Sender `g_Game+0x10C`; **critical section в обоих отсутствует** (тела из ~`0x80` байт без Enter/LeaveCriticalSection); envelope helper `0x40CD80` (`total_len = len + 0xC`, scratch `0x457970`), два общих `DataCrc32 0x413530`; null-проверки sender в телах нет — в доказанном call graph sender инициализирован к первому send |
| `Run` | `0x40F810` | Маска `type - (type & 0xFF)`: `0xFF000/0xEF200` → `0x4139D0` (OnBillingMessage), `0xEF100/0x10EF00` → `0x413580` (OnServerMessage); любая ветвь возвращает `1`, неизвестный тип — no-op |

## Billing: принятое Game-соединение

Реализация: `realm::app::billing_server_client`.

| Функция / символ | Адрес (VA) | Суть факта |
| --- | --- | --- |
| ctor `CClientForGS` | `0x40F180` | Base `0x40E0F0`, vtable `0x42DEE8`, receive buffer ровно `0xA00000` (`push 0xA00000` + alloc), два компаньона `0x100000` в семействе смещений `+0x64/+0x70`; send-buffer семьи `0x100000` — точных значений типа ObjectSize |
| `OnReceive` | `0x40F320` | Gate owner `+0xA8`; цикл при накоплении `>= 0xC`; CRC длины через общий `DataCrc32 0x413530` (mismatch → discard accumulator); `declared > size` — останов без потери хвоста; create через `CreateMessageWithoutRLE 0x40FA30`, повторный CRC содержимого; context assignment: `[client+0x34]→[+0x24]` socket, `[client+0x88]→[+0x20]` map, byte-string `[client+0x90]→[+0x2C]` CD-key, `[client+0x2C]→[+0x28]` IPv4; публикация в owner `+0xDC` через push helper `0x40CCC0`; consume `sub size, declared`; shrink к `0x100000`; reject обнуляет accumulator без отката опубликованного |
| `OnClose` | `0x40F230` | `new CMessage(0x10EF01)` размером `0x48`, `Add` long `[client+0x88]` (map identity), `Add` cstring `[client+0xC]` (peer IPv4 dotted с NUL — общий writer включает завершающий NUL), `Add` long `[client+0x8]` (long port); публикация в ту же FIFO до общего base close `0x40D850(0)` |

## Billing: listener

Реализация: `realm::app::billing_server`.

| Функция / символ | Адрес (VA) | Суть факта |
| --- | --- | --- |
| ctor `CServerForGS` | `0x40C780` | Базовый `CServer` ctor `0x40B8C0`, vtable `0x42DC48`, поле `+0x120 = 0`, limits `+0x14C = 0x64` (100) и `+0x150 = 0x1000000` — ровно константы компонента; в отличие от Auth и Login World, Billing `CGame` не перезаписывал лимиты из setup |
| `CreateServerClient` | `0x40C7C0` | `new` и ctor `CClientForGS 0x40F180` с владельцем `this` |

## Misc: сообщение

Реализация: `realm::app::misc_message`. Пара — MiscServer из блока идентификаторов.

| Функция / символ | Адрес (VA) | Суть факта |
| --- | --- | --- |
| ctor `CMessage` | `0x4107A0` | `MsgType` в header-слово `+4`, обнуление runtime-полей `+0x18/+0x1C/+0x20/+0x24` объекта `0x28` байт, vtable `0x4259F0` |
| `CreateMessage` | `0x4107D0` | Весь decode/create/copy/free закрыт `CRITICAL_SECTION 0x550564` до возврата (оба пути выхода; защищаемое состояние — общий static scratch `0x450548`/`0x550548`); порог `cmp len,0x20000; jbe` — capacity `0x100000`, больше → `len*8`; failure decode `0x410C00` → null; `timeGetTime` пишется в `+0x24` |
| `CreateMessageWithoutRLE` | `0x410920` | Null-вход или нулевая длина → null; та же 16-байтовая header-копия без отдельной проверки `len < 16` |
| `Send` | `0x4109F0` | Отправитель `g_Game+0x78`, null → 0; сериализация CS `0x55054C` вокруг envelope helper `0x410B50` (`total_len = len + 0xC`, scratch `0x55057C`, message копируется сразу после префикса), два общих `DataCrc32 0x405730`, виртуальный send `[vtable+0x40]` с `prioritized` и flags `0`, выход через LeaveCriticalSection |
| `Run` | `0x410AB0` | Маска `type - (type & 0xFF)`: `0x14EC00` → пустой destructor-стаб `0x4057A0` и возврат `1`, `0x14ED00` → `0x4036B0` (OnMSG_W2M_AUCTION) и `1`, `0x16EA00` → `0x403C10` (OnMSG_M2M_Fuction) и `1`, прочие → `0x403CC0` (OnOtherMsg) и строго `0` — в отличие от World-направления |

## Misc: исходящее World-соединение

Реализация: `realm::app::misc_client`.

| Функция / символ | Адрес (VA) | Суть факта |
| --- | --- | --- |
| ctor `CMyNetClient` | `0x412760` | Базовый `CClient` ctor `0x4111D0`, vtable `0x425D20` |
| `OnReceive` | `0x412820` | Один вызов читает не более `0x2800`; recv-ошибка (`!= WSAEWOULDBLOCK 0x2733`) и `recv == 0` — только журнал без публикации; цикл frames при накоплении `>= 0xC`: CRC длины через общий `DataCrc32 0x405730`, `declared < 0` — обнуление накопленного размера и ошибка, `declared > size` — останов без потери хвоста, create через `CreateMessageWithoutRLE 0x410920`, повторный CRC содержимого, публикация в FIFO `+0x100` через push helper `0x4139D0`, consume `declared`; shrink к `0x100000`; при любом reject оставшийся accumulator обнуляется, опубликованные сохраняются |
| `CClient::OnClose` | `0x4118B0` | Prereq-вызов `0x412E70(0)`, затем connect flag `[+0xB4] = 0` и виртуальный `HandleClose` (vtable `+0x4C`); `CMyNetClient::HandleClose 0x412790` публикует `new CMessage(0x16EA01)` в ту же FIFO `+0x100`; явный `Close` synthetic сообщение не создавал |

## World: wire-сообщение

Реализация: `realm::app::world_message`. Пара — WorldServer из блока идентификаторов; PDB-объект `E:\svn\fengyun_russia_dev\Nets\networld\Release\Message.obj`.

| Функция / символ | Адрес (VA) | Суть факта |
| --- | --- | --- |
| ctor `CMessage` | `0x422DA0` | `MsgType` в header-слово `+4`, обнуление runtime-полей `+0x18/+0x1C/+0x20/+0x24` объекта `0x28` байт |
| `CreateMessage` | `0x422DD0` | `cmp len,0x20000; jbe small` — вход короче `0x20001` получает capacity `0x100000` со статическим scratch `0x5B9BC8`, больший — `len*8` с временным buffer `0x6B9BC8`; failure decode `0x4235C0` → null; header 16 байт, внутренний length устанавливается `0x10`, `timeGetTime` пишется в `+0x24` (recv-время) |
| `CreateMessageWithoutRLE` | `0x422F20` | Null-вход или нулевая длина → null; та же 16-байтовая header-копия без отдельной проверки `len < 16` — там 32-битное переполнение длины payload |
| `SendToSocket` / `SendToMapID` / `SendAll` | `0x422FF0` / `0x4230B0` / `0x423170` | Server-отправитель из `g_Game+0x174`; null-отправитель возвращает 0 |
| `Send` | `0x423220` | Исходящий Login-клиент из `g_Game+0x170`; передаёт `prioritized` и flags `0` |
| send-сериализация | CS `0x6B9BCC` | Все четыре send-ветки сериализуются статическим CS; scratch `0x6B9BFC` строится как `[total_len, crc(total_len), crc(message), message]`; append-помощник `0x423510` вычисляет `total_len = len + 0xC` и копирует сообщение сразу после 12-байтового префикса; оба CRC — общий `DataCrc32 0x4A43A0` |
| `Run` | `0x4232E0` | Маска `type & 0xFFFFFF00` через `type - (type & 0xFF)`; тринадцать handler-целей: server `0x4ADCF0` (`0x3FC00/0x4FC00/0x5FA00`), log `0x4B0D10` (`0x4FB00/0x5FB00`), gma `0x4A6020` (`0x4FD00/0x60400`), player `0x4AD580`, other `0x4AC680`, gm `0x4AB370`, team `0x4AAD40`, orgasys `0x4A6110`, write-log `0x4A8AB0` (gate-byte `g_Game+0x290`), country `0x4A47F0` (`0x60300/0x7FF00`), server-auction `0x4A5650`, jjc `0x4A45E0`, misc-auction `0x4A5230`; любая ветвь, включая неизвестный тип, возвращает `1` |

## World: принятые Game-соединения

Реализация: `realm::app::world_server` (listener), `realm::app::world_server_client` (принятое состояние).

| Функция / символ | Адрес (VA) | Суть факта |
| --- | --- | --- |
| ctor `CMyNetServer` | `0x428250` | Базовый `CServer` ctor `0x427370`, vtable `0x54107C`, поле `+0x120 = 0`, limits `+0x14C = 100` и `+0x150 = 0x2000000` — ровно `WORLD_DEFAULT_MAX_IN_FLIGHT_SENDS` и `WORLD_DEFAULT_PERMITTED_SEND_BYTES` |
| `CreateServerClient` | `0x428290` | `new` объекта `0xC8` байт и ctor `CMyServerClient 0x42BBA0` с владельцем `this` |
| ctor `CMyServerClient` | `0x42BBA0` | Receive buffer ровно `0x1400000`; объект `0xC8` байт |
| `OnReceive` | `0x42BD00` | Цикл при накоплении `>= 0xC`; сначала CRC длины через общий `DataCrc32 0x4A43A0`, mismatch — обнуление всего накопленного размера (discard) и диагностика; `declared < 0` — discard и ошибка; `declared > size` — останов без потери хвоста; shrink к `0x1400000` при возврате под лимит; сообщение — `CreateMessageWithoutRLE 0x422F20`, CRC нормализованного содержимого, публикация в FIFO владельца (push helper `0x428710`) с контекстом `[client+0x34]→[+0x1C]`, `[client+0x88]→[+0x18]`, `[client+0x2C]→[+0x20]`; consume `sub size, declared`; `declared < 0xC` отдельной ветви не имело (подписанное переполнение payload-длины — класс повреждённого wire) |
| `OnClose` | `0x42BC50` | `new CMessage(0x3FC02)`, `Add(client+0x88)` (assigned map identity), публикация в ту же FIFO до общего base close (`0x42A2B0` с аргументом 0) |
| reconnect-тип `0x3FC03` | диспетчер `cmp eax,0x3FC03` по `0x4ADD4A` | Тип typed client handoff обрабатывает сам `OnServerMessage`; ранняя запись ошибочно относила сравнение к `OnTeamMessage` — в его теле ссылок на тип нет |

## World: исходящее Login-соединение

Реализация: `realm::app::world_client`.

| Функция / символ | Адрес (VA) | Суть факта |
| --- | --- | --- |
| ctor `CMyNetClient` | `0x429B70` | Базовый `CClient` ctor `0x428750`, vtable `0x54140C` (20 слотов; `OnReceive` — слот 7, `HandleClose` — слот 19) |
| `OnReceive` | `0x429C30` | Один вызов читает не более `0x2800`; recv-ошибка журналируется, `recv == 0` — журналируется без публикации; цикл frames при накоплении `>= 0xC`: CRC длины, `declared < 0` — обнуление накопленного размера и ошибка, `declared > size` — останов без потери хвоста, create через `CreateMessageWithoutRLE 0x422F20`, повторный CRC содержимого, публикация в FIFO `+0x100` (push helper `0x428710`), consume `declared`; shrink к `0x100000`; reject обнуляет оставшийся accumulator, опубликованные сохраняются |
| `CClient::OnClose` | `0x428E00` | Prereq-вызов `0x42A2B0(0)`, затем `m_bConnect [client+0xB4] = 0` и виртуальный `HandleClose` (vtable `+0x4C`); `CMyNetClient::HandleClose 0x429BA0` публикует `new CMessage(0x3FC01)` в ту же FIFO `+0x100` |

## World: диспетчер servermessage

Реализация: `realm::app::servermessage` — свободная `OnServerMessage` (`appworld/message/servermessage.cpp`), pub `?OnServerMessage@@YAXPAVCMessage@@@Z` RVA `0x000ACCF0` той же пары `Nworldserver.exe` + `WorldServer.pdb`. Ветвь `0x3FC02` (GameServer disconnect, synthetic close-сообщение `CMyServerClient::OnClose`) разобрана по машинному коду; статус контракта ветви — VERIFIED_DISASSEMBLY.

| Точка | Адрес (VA) | Суть факта |
| --- | --- | --- |
| decode начала ветви | `0x4ADD6F` | `CBaseMessage::GetDWord` (`0x423790`) → `GetGame` (`0x4017A0`) → `CGame::GetGameServer(K)` (`0x4132A0`, pub `1:0x122A0`) |
| not-found | `0x4ADE76` | `AddLogText` (`0x41E630`) формата `0x5473AC` (`!!!!!Unknown GameServer Lost!!!!!!!!![index = %d]`) |
| found | `0x4ADD94` | `connected = 0` (байт `tagGameServer+0`); `sprintf` (`0x51AFC0`) формата `0x5473FC` (`%s [%d]`, `strIP +0xC`, port `+0x24`) с удалением строки из Win32 listbox (`LB_FINDSTRINGEXACT 0x18F` → `LB_DELETESTRING 0x182`); `AddLogText` формата `0x5473E0` (`GameServer %s [%d] lost!`); legacy-неинициализированный port моделируется `Option` в Rust |
| auction slot | при `[esi+4] == 5` | `CMessage(0x80403)` (`0x422DA0`) + `Add<ulong>(0)` (`0x423C00`) + `SendAll` (`0x423170`, отправитель `g_Game+0x174`) |
| концовки | обе ветви | Завершают `CGame::OnGameServerLost(K)` (`0x411340`, pub `1:0x10340`) — миграция игроков offline и login-нотификация `0x1FE03` остаются inherent-методом владельца игры |

UI-эффекты (`LB_DELETESTRING` здесь, `LB_ADDSTRING 0x181` в ветви `0x5FA01`) находятся вне wire и в headless-реализации осознанно не воспроизводятся; player-listbox `AddPlayerList` подставлен no-op sink-ом по тому же принципу.

## World: write-log диспетчер

Реализация: `realm::app::writelogmessage`. Пара — WorldServer из блока идентификаторов (RSDS stream match).

| Символ | Адрес | Суть факта |
| --- | --- | --- |
| `?OnWriteLogMessage@@YAXPAVCMessage@@@Z` | pub `1:000a7ab0` (VA `0x4A8AB0`) | Цель Run-маски `0x60200` с gate-byte `g_Game+0x290`; отдельных pubs per-branch функций нет — ветви `0x60201..0x60218` собраны одним switch диспетчера |
| `CWriteLogQueue::{PushWriteLogData,PopWriteLogData,GetSize,Clear}` | pubs | Подтверждают FIFO-контракт write-log очереди |

## World: процесс и lifecycle

Реализация: `realm::app::world_network` (net-threads), `realm::app::world_runtime` (game thread), `realm::app::loginreconnectworker` (reconnect), `realm::app::worldserver` (`SendErrLog`), `realm::persistence::savedb` и `realm::persistence::saveworker` (save). S_PUB32-привязки пары `Nworldserver.exe` + `WorldServer.pdb`; канонические идентификаторы — `_worldserver_export_manifest.toml` (см. блок «Идентификаторы сборок»).

| Символ | Адрес (S_PUB32) | Суть факта | Реализация |
| --- | --- | --- | --- |
| `AcceptThreadFunc` | `0x24900` | Accept-поток `CServer` (`CreateAcceptThread 0x249C0`) | `world_network` |
| `NetThreadFunc` | `0x28060` | Цикл сетевого прохода → `DoNetThreadFunc 0x27740` (`CreateNetMainThread 0x28100`) | `world_network` |
| `WorkerThreadFunc` | `0x24870` | I/O-потоки → `DoWorkerThreadFunc 0x244B0` (`CreateWorkerThreads 0x26C90`) | `world_network` |
| `NetClientThreadFunc` | `0x29A80` | Исходящий Login → `DoNetClientThreadFunc 0x297D0` общего `CClient` | `world_network` |
| `?GameThreadFunc@@YGIPAX@Z` | `1:00019310` | Поток игры: `CreateGame` → `Init` → `MainLoop` → `Release` → `DeleteGame` | `world_runtime` |
| `?CreateGame@@YAHXZ` / `?DeleteGame@@YAHXZ` | `1:00014660` / `1:00000780` | Создание и удаление `CGame` | `world_runtime` |
| `?Init@CGame@@QAEHXZ` / `?MainLoop@CGame@@QAEHXZ` / `?Release@CGame@@QAEHXZ` | `1:00017ee0` / `1:00018a00` / `1:0000d7f0` | Lifecycle-методы `CGame` | `world_runtime` |
| `ConnectLoginServerFunc` | `1:000023b0` (VA `0x4033B0`) | Начальная проверка exit-флага `[0x56E48C]`, затем цикл `Sleep(0x1F40)` (cadence 8 секунд) → `ReConnectLoginServer` через `g_pGame [0x56E47C]`; выход при `== 1`, exit-флаг проверяется только после неуспешной попытки — спящий worker не прерывается stop; полный `WaitForSingleObject` — в `CreateConnectLoginThread 1:00003320` | `loginreconnectworker` |
| `ReConnectLoginServer` | `1:00002280` (VA `0x403280`) | Попытка восстановления соединения через process-global `g_pGame` | `loginreconnectworker` |
| `?SendErrLog@@YAXDJJPBD@Z` | `1:00000f30` | Cdecl-функция кода процесса WorldServer, а не nets-класса `CMessage`; публикует Login wire `0x0001_FE08` | `worldserver` |
| `?DoSaveData@@YAXXZ` | `1:0001b610` | Свободная save-оркестрация | `savedb` |
| `?ShowSaveInfo@@YAXPBDZZ` (+ `?g_bShowSaveInfo@@3_NA`) | `1:00000720` | Variadic gate save-лога с глобальным флагом | `savedb` |
| `?SaveThreadFunc@@YGIPAX@Z` | `1:00000e30` | Worker-вход сохранения; job собирается у process-owner-а | `savedb` / `saveworker` |
| `??0tagDBData@CGame@@QAE@XZ` / `??1tagDBData@CGame@@QAE@XZ` | `1:00010f60` / `1:0000e760` | ctor/dtor действующего save accumulator-а `tagDBData` владельца `CGame` | `savedata` |
| тела `CGame::process_message` (`1:00a30`) и `CGame::ai` (`1:138a0`) | pubs | Сверены полным машинным разбором по дампам `.local/verify-c5c/` (`dis_processmessage.txt`, `dis_ai.txt`); единственная правка — statement-order DIFF-A1 в `ai` | `world_main_loop` |

## World: контент и reload

Реализация: `realm::content::timetoreturn`, `realm::app::world_reload`.

| Функция | Адрес (VA) | Суть факта |
| --- | --- | --- |
| `TimeToReturn::on_time` | `0x472DC0` | Map-lookup по event ID (call map `find 0x472D50`); доступ к region через accessor `0x4017A0` (g_Game) → `0x413AC0` (region по signed map ID); только на существующем entry — `CMessage(0x7FA13)` через ctor `0x422DA0`, `Add` param `+0x10` (map ID) через writer `0x423C00`, `Add` param `+0x24` (buffer time), send через `SendToMapID 0x4230B0` с `[entry+0x4]`; weekly flag `(param+0x28)` gate-ит re-register через `push 7` в `0x4A36E0` (7 дней, как Rust `add_day(7)`) |
| `TimeToReturn::reload` | `0x473CF0` | Обходит весь map от iterator-start до end и отменяет event каждого param (`[param+0x2C] → 0x463F60 → 0x4637F0`) — reload отдельно отменяет IDs в map-order перед load |
| `TimeToReturn::initialize` | `0x473DF0` | Тот же entry-load через `load 0x4736F0`; имя ресурса `setup\TimeToReturn.ini` видно в `.rdata`-строке лога |
| тело `CGame::ReLoad` | pub `1:14740` | Диспетчер `reload_profiles` и таблица действий досверены по дампам `.local/verify-c5c/` (`dis_reload.txt`, `dis_reload_profiles.txt`): DIFF-3…DIFF-6 закрыты, DIFF-1/DIFF-2 без наблюдаемости; покрытие статических `Load*`-внутренностей не проверялось — статус тела `reload` PARTIAL |

## World: bai-tan и player data

Реализация: `realm::app::baitan`, `realm::app::playerdataqueue`, `realm::app::player_base`. Та же пара `Nworldserver.exe` + `WorldServer.pdb`; символы — S_PUB32, диапазоны — VA секции 1.

| Символ / точка | Адрес | Суть факта | Реализация |
| --- | --- | --- | --- |
| pubs bai-tan реестра | `?AddItemToBaiTanRequestList@CGame@@QAEXKJ@Z` `1:0000de30`, `?DelItemFromBaiTanList@CGame@@QAEXJ@Z` `1:000119f0`, `?AddItemToBaiTanList@CGame@@QAEXJK@Z` `1:00013140`, `?DoneBaiTanList@CGame@@QAEXXZ` `1:000131f0` | S_PUB32 методов `CGame` bai-tan реестра; поведение методов описано в шапке владельца | `baitan` |
| четыре исходных `std::map` реестра | `+0x56c..+0x594` в образце | Layout полей реестра в `CGame` | `baitan` |
| `?ProcessPlayerDataQueue@CGame@@QAEXXZ` + `CPlayerDataQueue::{GetSize,PopPlayerData,PushPlayerData}` | pubs `?ProcessPlayerDataQueue@CGame@@QAEXXZ`, `?GetSize@CPlayerDataQueue@@QAEIXZ`, `?PopPlayerData@CPlayerDataQueue@@QAEPAUtagPlayerDataQueue@@XZ`, `?PushPlayerData@CPlayerDataQueue@@QAE_NPAUtagPlayerDataQueue@@@Z` | Стадия MainLoop обрабатывает за проход не более одной non-null записи начального snapshot: null-pop уменьшает snapshot и повторяет pop | `playerdataqueue` |
| `CRsPlayer::OpenPlayerBase` (начало) | VA `0x0050F7E0..0x0050F845` | Байт успеха, account, 16-битный счётчик и повторный 16-битный ноль перед возвратом в ветке без строк; прежний Rust записывал там 32-битный ноль — исправлено | `player_base` |
| `CRsPlayer::OpenPlayerBase` (запрос части) | VA `0x0050F85B..0x0050F8A3` | `OpenPlayerBaseInDB` вызывается перед `OpenPlayerBaseInMem`; счётчик складывается 8-битной арифметикой | `player_base` |
