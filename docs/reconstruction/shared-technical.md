# Shared: машинная доказательная база технического слоя

**English summary.** Address tables and machine-level evidence moved out of the production header comments of `server/rust/shared/`. The code headers now state the mechanism contract and local quirks in short form; this document keeps the per-build RVA tables, PDB provenance, and disassembly anchors they were proven from. Build identification follows [reconstruction methodology](overview.md); sufficiency of each claim follows [evidence rules](evidence-and-contracts.md).

Верхние комментарии production source описывают контракт механизма и его границу. Полный машинный адресный слой, который эти контракты обосновывает, хранится здесь один раз и не дублируется в исходниках. Обратные ссылки даны как `shared::<module>::<file>`; канонические описания форматов остаются в `docs/protocol/message-header.md` и `docs/protocol/transport.md`.

## Идентификаторы сборок

Сетевые и технические владельцы восстановлены по парам EXE/PDB исторических серверных служб. SHA-256 зафиксированы манифестами экспорта в `server/rust/src/manifest/`; сведены здесь один раз для всей зоны Shared.

| Пара EXE/PDB | Манифест | EXE SHA-256 | PDB SHA-256 |
| --- | --- | --- | --- |
| AuthServer/authserver.exe + authserver.pdb | `_authserver_export_manifest.toml` | `AE0022429C…6AD3B15` | `26F8936605…028403D5` |
| BillingServer/billingserver.exe + billingserver.pdb | `_billingserver_export_manifest.toml` | `FA32E3C043…E68FB19` | `F900CD0330…AACA0B21` |
| LoginServer/loginserver.exe + LoginServer.pdb | `_loginserver_export_manifest.toml` | `1C84006DF6…47E7876` | `FBBCEB3B18…A1C98A902C` |
| MiscServer/miscserver.exe + miscserver.pdb | `_miscserver_export_manifest.toml` | `F442694246…998AED65` | `ED5F482DAD…0FC11FA7` |
| GameServer/gameserver.exe + GameServer.pdb | `_gameserver_export_manifest.toml` | `4F5C98E0FD…762C80E` | `B17BB9B7D6…2D66016` |
| WorldServer/Nworldserver.exe + WorldServer.pdb | `_worldserver_export_manifest.toml` | `F3AC454DAF…A2F466EF1` | `04E2CC4CE1…F6EF1E4` |
| GameServer/ServerUpdate.exe + ServerUpdate.pdb | `_serverupdate_export_manifest.toml` | `21CDB7E22D…950B2252` | `FF4C7D2515…42F2E77` |

Полные хеши — в манифестах; здесь даны префикс/суффикс для опознания. Клиентское приложение `Miracle game.exe`, использованное при подтверждении читающей стороны CiQing: SHA-256 `5b41ebfcea8c40ab8756e3fb676c8e03ec1f45fba527f33bc163b084cf520971`.

Ветки исходников по CodeView-записям PDB: `h:\fengyun\fy_russia\src\`, `d:\complite_version\fengyun_russia\trunk\` и `e:\svn\fengyun_russia_dev\`. Конкретный файл указывается в разделе механизма.

## Владелец входящих TCP-соединений CServer

Реализация: `shared::network::servers`. Исходные владельцы: `nets/servers.cpp`, `nets/servers.h` (все три ветки выше).

Threading-модель оригинала: все варианты передавали изменения одному net-thread через `CSocketCommands`; concurrent была только очередь, а `std::map`-реестры изменялись одним владельцем. Rust сохраняет границу через `ServerCommandHandle` + `CSocketCommands<ServerSocketCommand>`, а `BTreeMap` — детерминированный порядок исходного `std::map`.

| Функция | Auth | Billing | Login | Game | World |
| --- | --- | --- | --- | --- | --- |
| `Host` | `0x00012680` | `0x0000C690` | `0x0006A780` | `0x00018D80` | `0x00028160` |
| `OnAccept` | `0x00010A10` | `0x0000AA00` | `0x00068DB0` | `0x000176F0` | `0x00026740` |
| `AddAClient` | `0x00011050` | `0x0000B060` | `0x00069330` | `0x00017900` | `0x00026DA0` |
| `DelOneClient` | `0x000105C0` | `0x0000A3D0` | `0x00068490` | `0x000172C0` | `0x00026020` |
| `DoNetThreadFunc` | `0x00011C80` | `0x0000BC90` | `0x00069AE0` | `0x000180C0` | `0x00027740` |
| `LoadAllowedClient` | `0x000110D0` | `0x0000B0E0` | — | — | — |

`LoadAllowedClient` присутствует только в Auth и Billing; оба emitted-варианта совпадают. Статус реализованного слоя: `IMPLEMENTED`.

Набор producer-вариантов различался по сборкам: Auth не emitted `SendAll`; Login добавлял quit/set по строковой map identity; Game — quit/get по числовой; Billing и World использовали set числовой identity. Общий owner хранит доказанное объединение команд; компонент вызывает только свой подтверждённый набор.

## Очередь socket-команд CSocketCommands

Реализация: `shared::network::socketcommands`. Исходный владелец: `nets/socketcommands.cpp`.

| Функция | Auth | Billing | Login | Misc | Game | World |
| --- | --- | --- | --- | --- | --- | --- |
| `GetSize` | `0x00014BC0` | `0x0000E4D0` | `0x0006D8C0` | `0x00013B00` | `0x0001B910` | `0x0002AEB0` |
| `Pop_Front` | — | — | `0x0006D920` | `0x00013B60` | `0x0001B930` | `0x0002AF10` |
| конструктор | `0x00014E60` | `0x0000E770` | `0x0006D9A0` | `0x00013BE0` | `0x0001B9B0` | `0x0002AF90` |
| `Clear` | `0x00014E80` | `0x0000E790` | `0x0006D9C0` | `0x00013C00` | `0x0001B9D0` | `0x0002AFB0` |
| деструктор | `0x00015090` | `0x0000E9A0` | `0x0006DBD0` | `0x00013E10` | `0x0001BBE0` | `0x0002B1C0` |
| `Push_Front` | — | — | `0x0006E3D0` | `0x00014610` | `0x0001C400` | `0x0002B9E0` |
| `Push_Back` | `0x00015730` | `0x0000F040` | `0x0006E410` | `0x00014650` | `0x0001C440` | `0x0002BA20` |
| `AddCommandsQueueToFront` | — | — | `0x0006E450` | `0x00014690` | `0x0001C480` | `0x0002BA60` |
| `CopyAllCommand` | `0x00015830` | `0x0000F140` | `0x0006E550` | `0x00014790` | `0x0001C580` | `0x0002BB60` |

Auth и Billing не содержат неиспользованных в этих EXE `Pop_Front`, `Push_Front` и `AddCommandsQueueToFront`; остальные методы совпадают с полными вариантами того же source-файла. Статус: `IMPLEMENTED`.

Точный PDB фиксирует исходный `eSocketOperaType`: `ADD=0`, `CDKEYJOIN=1`, `PLAYERJOIN=2`, `DELBYSOCKETID=3`, `QUITBYSOCKETID=4`, `QUITBYMAPID=5`, `QUITBYMAPSTR=6`, `QUITALL=7`, `RECIEVE=8`, `SENDTOSOCKET=9`, `SENDTOMAPID=10`, `SENDTOMAPSTR=11`, `SENDALL=12`, `ONRECEIVE=13`, `ONSEND=14`, `ONCLOSE=15`, `ONCONNECT=16`, `SENDEND=17`. `tagSocketOper` занимает 24 байта в старом 32-битном ABI: `OperaType +0`, `lSocketID +4`, `pStrID +8`, `pBuf +12`, `lNum1 +16`, `lNum2 +20`. Смысл полей и допустимые сочетания доказывают производители/потребители в `clients.rs` и `servers.rs`; очередь сознательно остаётся generic `T` и не фиксирует layout за всех владельцев.

## Состояние принятого соединения CServerClient

Реализация: `shared::network::serverclient`. Исходные владельцы: `nets/serverclient.cpp`, `nets/serverclient.h`. Формат внешнего TCP-обрамления — `docs/protocol/transport.md`.

| Функция | Auth | Billing | Login | Game | World |
| --- | --- | --- | --- | --- | --- |
| `ReadFromCompletionPort` | `0x000142C0` | `0x0000DBC0` | `0x0006CFC0` | `0x0001B080` | `0x0002A620` |
| `Send` | `0x00014320` | `0x0000DC20` | `0x0006D020` | `0x0001B0E0` | `0x0002A680` |
| `AddReceiveData` | `0x00014490` | `0x0000DD90` | `0x0006D190` | `0x0001B250` | `0x0002A7F0` |
| `AddSendData` | `0x00014540` | `0x0000DE40` | `0x0006D240` | `0x0001B300` | `0x0002A8A0` |
| `AddPackageSize` | `0x000146D0` | `0x0000DFD0` | `0x0006D3D0` | `0x0001B480` | `0x0002AA20` |
| конструктор | `0x000147E0` | `0x0000E0F0` | `0x0006D4E0` | `0x0001B850` | `0x0002ADF0` |
| `OnReceive` | `0x000148A0` | `0x0000E1B0` | `0x0006D5A0` | `0x0001B580` | `0x0002AB20` |

Аудит частичного send completion по `CServer::DoWorkerThreadFunc`: Auth `0x0000DE60`, Billing `0x00007D00`, Login `0x000669D0`, Game `0x00015290`, World `0x000244B0`. Если completion передавал меньше байт, код только логировал отличие, освобождал весь buffer и публиковал `SENDEND`; остаток повторно не отправлялся. Это факт об IOCP completion, а не о частичном Linux `write`: Rust дописывает batch до завершения операции.

`OnReceive` разбирал envelope `[total_len, crc(total_len), crc(message), message]`, проверял IEEE CRC, создавал компонентный `CMessage`, затем ставил socket ID, map ID, peer IPv4; Auth/Billing/Login дополнительно копировали байтовую map/CD-key строку. Только после этого сообщение передавалось `CServer::m_RecvMessages`.

## Общие факты CMySocket

Реализация: `shared::network::mysocket`. Исходные владельцы: `nets/mysocket.cpp`, `nets/mysocket.h`.

Порядок таблицы: `SetIP`, init (`WSAStartup`-обёртка), cleanup, ctor, dtor, `Create`, `Bind`, `Close`, `OnClose`, `Recv`, `Send`, `RecvFrom`, `Sendto`, `GetSocketID`, `WSACreate`.

| Сборка | RVA в указанном порядке |
| --- | --- |
| Auth | `0x00001EA0`, `0x00013030`, `0x000130C0`, `0x000130D0`, `0x00013130`, `0x00013160`, `0x00013170`, `0x00013270`, `0x00013290`, `0x000132A0`, `0x00013320`, `0x00013330`, `0x00013420`, `0x000134E0`, `0x00013530` |
| Billing | `0x00001860`, `0x0000D600`, `0x0000D690`, `0x0000D6A0`, `0x0000D6F0`, `0x0000D720`, `0x0000D730`, `0x0000D830`, `0x0000D850`, `0x0000D860`, `0x0000D8E0`, `0x0000D8F0`, `0x0000D9E0`, `0x0000DAA0`, `0x0000DAF0` |
| Login | `0x00002AF0`, `0x0006C4F0`, `0x0006C580`, `0x0006C590`, `0x0006C5E0`, `0x0006C610`, `0x0006C620`, `0x0006C720`, `0x0006C740`, `0x0006C750`, `0x0006C7D0`, `0x0006C7E0`, `0x0006C8D0`, `0x0006C990`, `0x0006C9F0` |
| Misc | `SetIP/WSACreate` не emitted; init `0x00012C20`, cleanup `0x00012CB0`, ctor `0x00012CC0`, dtor `0x00012D10`, `Create` `0x00012D40`, `Bind` `0x00012D50`, `Close` `0x00012E50`, `OnClose` `0x00012E70`, `Recv` `0x00012E80`, `Send` `0x00012F00`, `RecvFrom` `0x00012F10`, `Sendto` `0x00013000`, `GetSocketID` `0x000130C0` |
| Game | `0x00001DD0`, `0x0001AAC0`, `0x0001AB50`, `0x0001AB60`, `0x0001ABB0`, `0x0001ABE0`, `0x0001ABF0`, `0x0001ACF0`, `0x0001AD10`, `0x0001AD20`; `Send` вынесен linker-ом: `0x001B7020`; `RecvFrom` `0x0001ADA0`, `Sendto` `0x0001AEA0`, `GetSocketID` `0x0001AF60`, `WSACreate` `0x0001AFB0` |
| World | `0x000011C0`, `0x0002A060`, `0x0002A0F0`, `0x0002A100`, `0x0002A150`, `0x0002A180`, `0x0002A190`, `0x0002A290`, `0x0002A2B0`, `0x0002A2C0`; `Send` вынесен linker-ом: `0x000DBD10`; `RecvFrom` `0x0002A340`, `Sendto` `0x0002A440`, `GetSocketID` `0x0002A500`, `WSACreate` `0x0002A550` |

`VERIFIED_DISASSEMBLY` для `GetSocketID`: Misc RVA `0x000130C0` читает, увеличивает и записывает storage RVA `0x001505B0`; адрес лежит в нулевом virtual tail секции `.data`, поэтому loader задавал исходный ноль. Все найденные `CServer::Host` передают type `1` (`TCP`) в `WSACreate`.

## Очередь сообщений CMsgQueue

Реализация: `shared::network::msgqueue`. Исходный владелец: `nets/msgqueue.cpp`.

Auth / Billing / Login содержат по шесть методов; «соответственно» в таблице означает тот же порядок: `GetSize`, `PopMessage`, конструктор, `Clear`, деструктор, `PushMessage`.

| Функция | Auth | Billing | Login |
| --- | --- | --- | --- |
| `GetSize` | `0x0000D5E0` | `0x0000C840` | `0x0006AA70` |
| `PopMessage` | `0x0000D600` | `0x0000C8D0` | `0x0006AA90` |
| конструктор | `0x0000D680` | `0x0000C950` | `0x0006AB20` |
| `Clear` | `0x0000D6B0` | `0x0000C980` | `0x0006AB50` |
| деструктор | `0x0000D720` | `0x0000C9F0` | `0x0006ABC0` |
| `PushMessage` | `0x0000D9F0` | `0x0000CCC0` | `0x0006AE90` |

Misc: конструктор `0x00013230`, `Clear` `0x00013260`, деструктор `0x000132D0`, `PushMessage` `0x000139D0`, `GetAllMessage` `0x00013AD0`. Game — соответственно Misc: конструктор `0x00011F30`, `Clear` `0x00011F60`, деструктор `0x00011FD0`, `PushMessage` `0x000126D0`, `GetAllMessage` `0x000127D0`. World: `GetSize` `0x00028310`, `PopMessage` `0x00028330`, конструктор `0x000283B0`, `Clear` `0x000283E0`, деструктор `0x00028450`, `PushMessage` `0x00028710`.

Статус реализованного контракта: `IMPLEMENTED`. Различающиеся имена специализаций `std::deque` в декомпиляции — ошибки восстановления типов: прототипы и вызовы последовательно используют `CBaseMessage*`.

## Исходящий TCP-клиент CClient

Реализация: `shared::network::clients`. Исходные владельцы: `nets/clients.cpp`, `nets/clients.h`.

| Функция | Login | Misc | Game | World |
| --- | --- | --- | --- | --- |
| `SendToServer` | `0x0006B0F0` | `0x000113F0` | `0x000191D0` | `0x00028960` |
| `OnReceive` | `0x0006B210` | `0x00011510` | `0x000192F0` | `0x00028A80` |
| `OnConnect` | `0x0006B580` | `0x00011880` | `0x00019640` | `0x00028DD0` |
| `ConnectServer` | `0x0006B5D0` | `0x000118D0` | `0x00019690` | `0x00028E20` |
| `AddSendSize` | `0x0006B6F0` | `0x000119F0` | `0x000197B0` | `0x00028F40` |
| `Create` | `0x0006B9C0` | `0x00011CC0` | `0x00019B10` | `0x00029210` |
| `Send` | `0x0006BAD0` | `0x00011DD0` | `0x00019C20` | `0x00029320` |
| `Connect` | `0x0006BB90` | `0x00011E90` | `0x00019CE0` | `0x000293E0` |
| `DoNetClientThreadFunc` | `0x0006C140` | `0x000123B0` | `0x0001A110` | `0x000297D0` |

Game дополнительно содержит `AddRecvSize` `0x00019840`. Четыре реализации `IsSameSocketID` безусловно возвращали `true`, поэтому поле `lSocketID` команды не влияло на путь отправки.

## Базовый wire-буфер и RLE CBaseMessage

Реализация: `shared::network::basemessage`. Исходные владельцы: `nets/basemessage.cpp`, `nets/basemessage.h`. Каноническое описание формата и статусов полей — `docs/protocol/message-header.md`. ImageBase всех EXE — `0x400000`.

Конструктор: Auth `0x00012FB0`, Billing `0x0000D530`, Login `0x00066360`, Misc `0x00011100`, Game `0x00013520`, World `0x00023EE0`. Числовые `Get`: Auth `0x000129B0..0x00012A10`, Billing `0x0000CF10`, Login `0x00065C10..0x00065C70`, Misc `0x00010CC0..0x00010CF0`, Game `0x00012BF0..0x00012CC0`, World `0x000236F0..0x000237C0`. Сырые `Get`: Billing `0x0000CFA0`, Login `0x00065D00`, Misc `0x00010D80`, Game `0x00012D50`, World `0x00023850`. `CGUID Get/Add`: Billing `0x0000D030/0x0000D5B0`, Misc `0x00010E00/0x00011180`, Game `0x00012E50/0x000135A0`, World `0x000238A0/0x00023F60`. `AddEx`: Game `0x00013490`, World `0x00023E50`; inline `Update`: Game `0x00005060`, World `0x0001A990`.

`DoRLE`: Login `0x000659A0`, Game `0x00012930`. `DecodeRLE_SAFE`: Billing `0x0000CE30`, Login `0x00065B30`, Misc `0x00010C00`, Game `0x00012AF0`, World `0x000235C0`.

Аудит времени жизни scratch-указателя send-path: `CServer::SendAll` и `SendBySocketID` копируют вход до возврата (Billing `0x000078D0/0x00007950`, Login `0x000664C0/0x00066540`, Game `0x00014E10/0x00014E90`, World `0x00024080/0x00024100`); `CClient::SendToServer` делает то же в Login `0x0006B0F0`, Misc `0x000113F0`, Game `0x000191D0`, World `0x00028960`. Часть send-методов при этом держит `m_CSTemptBuffer` на последовательности build/CRC/send, а часть аналогичных методов не захватывает её вообще; решение о межпоточной сериализации send остаётся у конкретного `CMessage` после аудита его достижимости.

## CRC32 CCrc32Static

Реализация: `shared::protocol::crc32`. Исходный владелец: `public/crc32static.cpp`.

`DataCrc32`: Auth `0x00015F80`, Billing `0x00013530`, Login `0x0007F050`, Misc `0x00005730`, Game `0x0007B0A0`, World `0x000A43A0`. Все шесть тел совпадают: регистр `0xFFFF_FFFF`, reflected-таблица с полиномом IEEE CRC-32, инверсия результата — ровно алгоритм `crc32fast`. Статус: `IMPLEMENTED`.

ServerUpdate дополнительно содержит `GetFileSizeQW` `0x00003D10` и `FileCrc32Filemap` `0x00003D70` из пары `GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb`; Windows file mapping блоками до `0xA00000` — технический механизм, заменён потоковым чтением без изменения checksum.

## Технические функции public/tools.cpp

Реализация: `shared::runtime::tools`. Исходные PDB-пути: `h:\fengyun\fy_russia\src\public\tools.cpp:198`, `d:\complite_version\fengyun_russia\trunk\public\tools.cpp:769`, `e:\svn\fengyun_russia_dev\public\tools.cpp:198/769`.

| Функция | Сборка | RVA | Статус |
| --- | --- | --- | --- |
| `IniDecoder` | Login | `0x00020A90` | `IMPLEMENTED` |
| `IniDecoder` | World | `0x00053C50` | `IMPLEMENTED` |
| `PutStringToFile` | Billing | `0x0000FC60` | `IMPLEMENTED` |
| `PutStringToFile` | Game | `0x0001CEB0` | `IMPLEMENTED` |
| `GetLineDir` | Game | `0x0001D080` | `VERIFIED_DISASSEMBLY` |
| `AddLogText`/`AddErrorLogText`/`PutDebugString` | Game | — | `IMPLEMENTED` |

Остальной корпус `public/tools.cpp` остаётся `UNKNOWN`. `GetLineDir` делит плоскость углами из точного EXE: `0.39259999990463257` и `1.1779999732971191`.

## Календарное значение tagTime

Реализация: `shared::values::date`. Исходные владельцы: `public/date.cpp`, `public/date.h` (ветка `e:\svn\fengyun_russia_dev`). Пары: LoginServer, GameServer, WorldServer.

RVA WorldServer: constructor `0x000A3CF0`, `IsLeap` `0x000A36A0`, comparisons `0x000A3560..0x000A3680`, `AddDay` `0x000A36E0`, `AddHour`/`AddMinute`/`AddSecond` `0x000A3990/0x000A3A10/0x000A3A90`, `GetTimeDifference` `0x000A3B10`, `GetFormatStr` `0x000A41B0`. GameServer и LoginServer содержат тот же достигнутый контракт. `VERIFIED_DISASSEMBLY`: обе строки `dtab` прочитаны по VA `0x0056B640` точного `Nworldserver.exe`: `0,31,28,...` и `0,31,29,...`.

PDB задаёт восемь последовательных `u16` и размер `0x10`. Достигнутый owner: field constructor, string constructor, `IsLeap`, пять сравнений, `AddDay/AddHour/AddMinute/AddSecond`, `GetTimeDifference`, `GetFormatStr` — `IMPLEMENTED`.

## Глобальный setup CGlobeSetup

Реализация: `shared::resources::globesetup`. Подтверждён парами WorldServer и GameServer. Основной wire — raw `0x1114`-байтный `tagSetup`, затем полный `CRegionRouter`; парный Game decoder копирует snapshot, затем очищает и восстанавливает router.

Подтверждённые чтения полей по точному GameServer EXE:

| Область полей | Основание |
| --- | --- |
| ordinary player-scale коэффициенты `+0x8AC..+0x8B8` | `CPlayer::MountEquip` VA `0x443CB6/0x443E5C/0x444002/0x44426B` |
| battle-fairy `+0x8D4..+0x8E0` | absolute reads `0xEF4694..0xEF46A0` |
| общий base snapshot `0xEF3DC0` и public-talk поля | absolute reads `0xEF4528/68/6C/70/B0/B4` и `0xEF4604/08` в `OnOtherMessage` |
| auto-inc progression `+0x734/+0x75C/+0x81C..+0x828` | reached `CPlayerAI::Run`, полный `CheckLevel` tail |
| goods disappear/protection `+0x34C/+0x350` | `CArea::AI` |
| pet lifecycle `+0x728/+0x72C/+0x73C` | pet tracing/follow пути |
| `+0x07C/+0x080` monster hit min/max | absolute VA `0xEF410C/0xEF4110` |
| `bAllowClientRunScript +0x50C`, `bAllowClientChangePos +0x50D` | позиционная проекция GameServer; разрешают `othermessage 0x8FB01/channel 9` и `shapemessage 0x8F902` |
| `bRotation +0xC48` | PDB: непосредственно перед `bGoodsAi +0xC4C`; серверный байт поворота квестового шага `0x8F903` |
| recovery-поля `+0xC50..+0xC6C` | PDB `CGlobeSetup::tagSetup`; player property projection |
| RP-блок `+0x3F0..+0x41F` | reached `CPlayer::IncreaseRp` |
| `fDecTimeParam +0x568`, `lDiedStateTime +0x56C` | nation contender damage, death penalty |
| `dwAutoProtectTime +0x804` | `STATE_AUTO_PROTECT` при смене региона |
| `lResumeTimer +0x354` | `CPlayer::OnEnterRegion`: VA `0xEF4114` при base `0xEF3DC0`, calls `0x0045A2CD/0x0045A37D` |

Typed accessors накладываются только на подтверждённые offsets; raw snapshot остаётся единым wire owner-ом без дублирующей config-модели.

## LingBao CLingBaoSetup

Реализация: `shared::resources::lingbao`. Исходный owner PDB: `e:\svn\fengyun_russia_dev\server\setup\lingbao.cpp/.h`. Loader/serializer — пара WorldServer; decoder `DecodeFromArrayLingBao` — пара GameServer.

Вставка в исходную map: `_Insert` (gameserver VA `0x5ca150`, WorldServer VA `0x47da80`) при равном ключе уходит вправо и вставляет новый node без выхода по совпадению — один node на запись даже для одинаковых имён; gameplay-поиск (gameserver VA `0x5c9370`) поэтому сканирует map парой (ключ, ticket). Decoder: VA `0x5ca240`, выход по `JE`; отрицательный signed count оригинал принял бы за огромный unsigned loop с чтением за пределами буфера, Rust трактует его как пустой вход (`max(0)`).

## Конфигурация CiQing CCiQingSetup

Реализация: `shared::resources::ciqing`. Исходный owner PDB: `e:\svn\fengyun_russia_dev\public\ciqing.cpp/.h`. ImageBase обоих EXE — `0x400000`.

World `CCiQingSetup::ReadSetupFile` VA `0x4877F0`, `AddByteToArray` VA `0x486080`; Game `AddByteToArray` VA `0x4E4740`, `DeByteFromArray` VA `0x4E6110`. Game writer VA `0x4E4800..0x4E4852` пишет шесть отдельных полей `stComposeNode` по смещениям `0, 4, 8, 0x10, 0x0C, 0x14`, затем размер result-vector. GameServer.pdb, тип `CCiQingSetup::stComposeNode` (`0xD48C`), подтверждает отдельные source A/B и `dwResultNum` по `+0x14`: дубля source A нет. Клиент `Miracle game.exe` (SHA-256 в блоке идентификаторов), decoder VA `0x4684A0`, читает ту же часть в `0x468603..0x468662`. Общий формат World → Game → клиент — в `docs/protocol/opcode-catalog.md`, раздел конфигурации CiQing.

## Комплекты TaoZhuang CTaoZhuangSetup

Реализация: `shared::resources::taozhuangsetup`. Исходный owner PDB: `e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp/.h`. World `ReadFile/AddByteToArray` — пара WorldServer; Game `DeCodeFromByte` — пара GameServer.

Доказанные особенности, сохранённые в заголовке реализации: declared counts передаются как сохранённые `u32`, а не пересчитываются; duplicate item ID оставляет первую запись (не исправляется как внутренний дефект); полный ранее декодированный item-prefix сохраняется на safe short-buffer границе.

## Таблицы преобразования экипировки EquipmentComposeList

Реализация: `shared::resources::equipmentcomposelist`. Исходный owner PDB: `e:\svn\fengyun_russia_dev\public\equipmentcomposelist.cpp`. `LoadList/AddToByteArray` — пара WorldServer; `DecordFromByteArray/GetFirstCompose/GetSecondCompose` — пара GameServer.

Доказанная особенность: исходный Game decoder всегда возвращал `false` даже после успешной записи, но reconnect caller игнорировал результат, поэтому Rust возвращает содержательный report.
