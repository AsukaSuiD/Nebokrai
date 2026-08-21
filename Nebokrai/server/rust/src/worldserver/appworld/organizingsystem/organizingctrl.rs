//! Владелец organizing-control исторического `WorldServer`.
//!
//! Статус `COrganizingCtrl::AddOneTopInfo` RVA `0x00036960`,
//! `SendTopInfoToClient` RVA `0x00033FC0` и
//! `SendAllTopInfoToInfoToOneClient` RVA `0x000352C0` — `IMPLEMENTED`;
//! полный `COrganizingCtrl::Run` RVA `0x0003A550` —
//! `IMPLEMENTED/VERIFIED_DISASSEMBLY`, `DisbandFaction` RVA `0x00038550` и
//! `UpdateOtherFacInfoToClient` RVA `0x00034980` — `IMPLEMENTED`;
//! `IsFreePlayer` RVA `0x000343A0`, `IsFreeFaction` RVA `0x00034420`,
//! `RemovePersonFromApplyFactionList/GetFactionByPlayerInApplyList` RVA
//! `0x00034880/0x000348F0`,
//! `SetPlayerOrganizing` RVA `0x000370A0` и callback-цепочки
//! `OnPlayerEnterGame/OnPlayerExitGame` RVA `0x00037B70/0x00037BD0` —
//! `IMPLEMENTED/VERIFIED_DISASSEMBLY`; `GenerateSaveData` RVA `0x00034A10` —
//! `IMPLEMENTED`, `GetpFactionById` RVA `0x00034080` и
//! `GetConfederationOrganizing` RVA `0x00036BF0`,
//! `IsFactionMaster` RVA `0x000344A0`, `ReInitialFacFactionByLvl` RVA
//! `0x00034C80` и `AddUnionToClientByFactionID` RVA `0x00038010` —
//! `IMPLEMENTED`; `PushToEstaList` RVA `0x000367C0` и оба overload-а
//! `SendOrgaInfoToClient` RVA `0x00033750/0x00033840`, billboard serializer-ы
//! RVA `0x000339D0/0x00033A50/0x00033AD0/0x00033CC0`, три stat-owner-а
//! RVA `0x0003AC70/0x0003B050/0x0003B430`, `TransferIOwnerCity` RVA
//! `0x00039890` и его локальные constructor/`DoAsyncCall`/`OnAsyncCallback`
//! RVA `0x00033F40/0x00037810/0x0003A070` — `IMPLEMENTED`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.h`
//! и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:73,164,242,626,725,1105,1238,1352,1641,1655,1910,1952,1960,1970,1998`.
//!
//! `GenerateSaveData` проходит faction-map, затем union-map в signed key-order.
//! `force_all=true` ставит bits `1/2/4/8` четырьмя virtual-вызовами; concrete
//! clone дописывается в `CGame::tagDBData`, после чего live mask сбрасывается.
//! Затем delete-faction и delete-union ID переносятся в list-order, и только
//! после полного переноса каждый исходный список очищается. Все достигнутые
//! caller-ы полного `CGame::GenerateDBData` передают `false`, но доказанная
//! bool-ветвь сохранена. Null map-value исходно разыменовывался при clone и
//! остаётся локальной typed-блокировкой; уже выполненные append/reset эффекты
//! не откатываются. `BTreeMap`, `VecDeque`, `Clone` и `Drop` заменяют только
//! MSVC tree/list/RTTI/allocator и compiler cleanup.
//!
//! `GetpFactionById` ищет signed ID только в `m_FacOrg`. Miss возвращает
//! `nullptr`; найденный null value возвращается тем же `nullptr` без
//! разыменования. Узкая Rust-граница возвращает borrow найденного concrete
//! `CFaction`; текущий DB-caller читает у него только Goods War count и затем
//! переносит его в save-копию. `BTreeMap::get`, `Option` и borrow заменяют
//! только MSVC iterator и nullable pointer; raw owner-блок после реализации
//! удалён.
//! `GetConfederationOrganizing` принимает только положительный signed ID,
//! ищет его в `m_ConfedeOrganizings` и возвращает сохранённый pointer либо
//! null при miss/null value. Exact ASM `0x00436BF0..0x00436C37` исправляет
//! повреждённое raw-имя key; Rust `confederation_by_id` сохраняет контракт.
//! Через read-only `FactionOperationAuthorityContext` этот lookup и уже
//! материализованный `IsFreeFaction` обслуживают faction tax/city-gate owner-ы;
//! null во время membership scan остаётся typed-границей старого UB.
//! Тот же nullable lookup обслуживает `CFaction::GetPlayerHeader`: найденный
//! союз отдельно разрешает master-faction через `m_FacOrg`, где miss/null даёт
//! старый `0`, а отсутствующий reached master ID concrete Rust-faction остаётся
//! typed-блокировкой.
//! `IsFactionMaster` проходит faction-map в signed order и возвращает только
//! первый положительный concrete faction ID, чей master равен входному player
//! ID. Null value был разыменованием, а отсутствующий reached Rust master ID
//! не получает выдуманного продолжения: обе границы возвращаются typed.
//!
//! `IsFreePlayer` проходит `m_FacOrg` в порядке исходного ordered map и для
//! каждого `COrganizing*` вызывает virtual slot `+0xDC`. Точный PDB
//! именует faction-реализацию `CFaction::IsMember(long)`, а exact EXE
//! `0x004BD840..0x004BD865` подтверждает: ключом `m_Members.find` служит входной
//! player ID; при наличии возвращается signed `m_lID`, иначе `0`. Первый
//! положительный faction ID немедленно завершает обход. В отличие от двух
//! последующих callbacks, сам scan не проверяет map-value на null. Rust
//! сохраняет такую запись через `Option<Box<CFaction>>`, но останавливает её
//! локальным `BLOCKED_MISSING_FACT`, а не назначает старому null-dereference
//! продолжение либо fail-closed результат.
//!
//! `IsFreeFaction` симметрично проходит `m_ConfedeOrganizings`; PDB публикует
//! `CUnion::IsMember(long)` на том же folded RVA `0x000BD840`, поскольку ID и
//! member-map обоих concrete owners лежат по одинаковым offsets. Он ищет
//! входной faction ID и возвращает первый положительный union ID. Null map-
//! value остаётся такой же локальной неизвестностью старого разыменования.
//!
//! `AddUnionToClientByFactionID` при нулевом union ID сначала вызывает
//! `IsFreeFaction`; неположительный результат и отсутствующий/null union-owner
//! возвращают `false`. Найденная faction обходится по всем member-key в signed
//! order — Linux-донор ошибочно ограничивал путь master-ом. Допускается только
//! online player с ненулевым GameServer ID и полученными faction data. Wire
//! `0x7FE04` получает live `CPlayer::GetID`, затем заново построенный полный
//! union snapshot; поэтому cached faction name/level могут обновиться перед
//! каждым send. Нормальный owner-путь возвращает `true` даже без получателей и
//! игнорирует send result. `CMessage`, `BTreeMap`, split borrow и `Result`
//! заменяют transport/MSVC tree/null plumbing; отсутствие faction, которое
//! машина разыменовывала, остаётся typed safe-границей с выполненным prefix-ом.
//! `m_RequestEstaUnionPlayers` материализован как `VecDeque<i32>`: scan идёт
//! от начала, `PushToEstaList` без duplicate-gate добавляет literal ID в хвост,
//! а terminal callback удаляет только первое совпадение. Проверку отсутствия
//! перед push сохраняет вызывающий `CUnion::ApplyForJoin`; `VecDeque` заменяет
//! только MSVC list nodes/allocator и не меняет порядок либо duplicates.
//! Terminal owner сначала при положительном решении выполняет точный `DoJoin`
//! найденного union, при отказе отправляет `WS0266/WS0193` на literal faction
//! ID, затем на normal return сбрасывает pending application и удаляет первое
//! совпадение из списка. Для безопасного split-borrow union временно извлекается
//! из собственного map-slot и обязательно возвращается до terminal cleanup;
//! результат предстоящего `IsFreeFaction(applicant)` снимается до извлечения и
//! ровно на этот вызов передаётся `DoJoin`, поэтому даже уже оказавшийся в этом
//! же union applicant или более ранний null/map match не теряется. Остальные
//! достигнутые callbacks работают с faction-map либо получают union явно;
//! ownership-замена не меняет их lookup и порядок эффектов.
//!
//! `TransferIOwnerCity` exact `0x00439890..0x0043A068` проверяет живой регион
//! и положительный ID, затем village/attack state, master-faction, страны,
//! country king, владение регионом, пустой city-list цели, обе war-заявки,
//! source/target reservation и online target-master. Отказы `WS0248..WS0256`
//! используют title `WS0193`; отсутствующие region/faction/master и чужой
//! регион остаются тихими. После успеха source и target ID без duplicate-gate
//! добавляются в общий establishment-list, online player выдаёт process-wide
//! NetEx ID и начинается session на `0x2710` ticks. Старый Linux-донор менял
//! этот порядок и добавлял rollback/session checks; они не являются контрактом
//! целевого EXE и не перенесены.
//!
//! Локальный `DoAsyncCall` exact `0x00437810..0x00437A31` строит `0x7FE2B`:
//! target master, source faction name C-строка, region name C-строка, session
//! ID и `cookie.second`, затем маршрутизирует по target player. Callback exact
//! `0x0043A070..0x0043A2B9` всегда удаляет первые source и target reservation;
//! только result с decision `1` требует обе faction, вызывает у source именно
//! безаргументный `ClearOwnedCity` (очищая весь список, а не только переданный
//! регион), у target `AddOwnedCity(region)`, затем `IsFreeFaction(target ID)`,
//! `RefreshOwnedCityOrg` и общий `WS0257` broadcast с kind/color
//! `0xFFDAEDFE/0x328F93FC`. Очередь под `parking_lot::Mutex`, `Arc`, `Box` и
//! готовый `CNetSessionManager` заменяют singleton/interface lifetime и ручной
//! `new/delete`, не меняя wire, timeout, FIFO terminal-порядок или mutations.
//! `SendOrgaInfoToClient(player, first, second, server, color, trailing)` при
//! `server == -1` разрешает GameServer через live player; только literal `-1`
//! после lookup прекращает путь. Exact EXE `0x0043378F..0x0043380B` строит
//! `0x7F804` в порядке `player, first\0, color, trailing, second\0` и отправляет
//! по numeric map ID. Нулевой lookup не отбрасывается: готовый `CGame` именно
//! его возвращает при miss, а исходный caller проверял только `-1`.
//!
//! `RemovePersonFromApplyFactionList` вызывает `RemoveApplyMember(player)` у
//! каждой faction в signed map-order, игнорирует все concrete return values и
//! после полного прохода всегда возвращает `0`. Каждое фактическое удаление
//! поэтому успевает опубликовать пустой `OP_Delete`-record и поставить dirty
//! bit `8`, прежде чем обход продолжится. `GetFactionByPlayerInApplyList`
//! проходит тот же map и возвращает `GetID()` первой faction, чей
//! `IsInApplyMembers(player)` дал положительное значение; miss возвращает `0`.
//! Exact ASM `0x00434880..0x004348EC/0x004348F0..0x0043496F` подтверждает
//! virtual slots, ранний выход lookup и порядок. Rust отчёты сохраняют исходно
//! игнорировавшиеся результаты удалений; null map-value остаётся локальной
//! typed-границей после уже завершённого prefix-а.
//!
//! Exact `SetPlayerOrganizing` `0x004370A0..0x0043737A` сначала всегда пишет
//! результат `IsFreePlayer`. Для найденной faction он в точном порядке пишет
//! level, exp, contribute, name, member-title, master, оба enemy-set, очищает
//! owned-регионы и проходит faction city-list. Только существующий ненулевой
//! `tagRegion::pRegion` добавляется с младшими 16 битами `REGION_TYPE`; затем
//! независимо выполняются `IsFreeFaction` и условная запись union-master.
//! Exact key-dataflow исправляет повреждённые raw stack-slot имена для обоих
//! map lookup, `IsControbute` и `GetTitleByID`.
//!
//! `CPlayer::AddOwnedRegion` копировал все восемь байт локального `tagOwnedReg`,
//! хотя PDB задаёт только `long +0` и `unsigned short +4`: два последних байта
//! не инициализировались и позже наблюдались в player-wire. Rust сохраняет все
//! предшествующие мутации, но останавливается перед первым таким добавлением с
//! `BLOCKED_MISSING_FACT`; нули или иное значение padding не выдумываются.
//!
//! Constructor RVA `0x00036C90` создаёт `m_FacOrg` пустым. `CreateFaction` RVA
//! `0x000381A0` выделяет concrete `CFaction` и сохраняет base-pointer в map, а
//! `Release` RVA `0x00036F40` virtual-удаляет каждое ненулевое значение и
//! обнуляет slot. `BTreeMap<i32, Option<Box<CFaction>>>` заменяет MSVC tree,
//! pointer/null и доказанное владение, сохраняя signed key-порядок. Достигнутый
//! `m_ConfedeOrganizings` выражен симметричным map `CUnion`; Rust-layout не
//! выдаётся за Windows ABI, а остальные поля singleton-а остаются raw.
//!
//! `ReInitialFacFactionByLvl` проходит тот же signed faction-map. Exact EXE
//! `0x00434C80..0x00434CFE` делает RTTI cast каждого value, пропускает null либо
//! иной concrete type и вызывает `CFaction::ReInitialPropertyByLvl`; typed map
//! исключает посторонний concrete owner, а `Option::None` сохраняет skip.
//! Результаты уже выполненных property-send не откатываются при safe-границе
//! неполного Rust-owner-а.
//!
//! Exact `CUnion::DelMember` `0x004C2B30..0x004C2B8B` не читает union receiver:
//! для положительного входного ID он дважды использует один stack-slot как key
//! faction-map, затем вызывает `SetSuperiorOrganizing(0)` и
//! `UpdatePropertyToClient`; miss/null и неположительный ID пропускаются, а
//! возврат всегда `true`. Singleton lookup перенесён к фактическому map-owner-у
//! `COrganizingCtrl::detach_union_member`, без изменения порядка эффектов.
//!
//! Оба callback-а сначала сохраняют исходный player ID, вызывают
//! `IsFreePlayer`, а при положительном результате ищут именно этот faction ID
//! в `m_FacOrg`. Декомпилят ошибочно подставил this вместо ключа поиска find:
//! exact EXE `0x00437B7A..0x00437BA6` и `0x00437BDA..0x00437C06` сохраняет
//! результат в stack-slot и передаёт его и в `find`, и в `operator[]`.
//! Ненулевой faction получает исходный player ID: enter через slot `+0x13C`,
//! exit через `+0x140`; PDB именует их
//! `CFaction::OnMemberEnterGame/OnMemberExitGame`. Enter после любой
//! faction-ветви безусловно вызывает `SendAllTopInfoToInfoToOneClient`, exit
//! дополнительных эффектов не имеет. «Безусловно» относится к нормальному
//! возврату faction callback-а: если safe Rust достигает уже локализованного
//! старого UB внутри faction, top-info не исполняется как выдуманное
//! продолжение после невозвратившегося исходного пути. Exit так же передаёт
//! blocked-результат наружу, чтобы будущий `CGame::RemoveOnlinePlayer` не мог
//! продолжить операции после исходного невозврата.
//!
//! Typed dispatch различает отсутствие membership, отсутствие возвращённого
//! key, явно null target и выполненный concrete callback. Faction slots мутируют
//! полный `COrganizing::tagMemInfo` и публикуют member-update; enter
//! дополнительно читает online-player/region, затем на нормальном возврате
//! выполняет уже готовую top-info рассылку. Внутренности MSVC tree/string/list
//! и compiler cleanup не являются отдельной Rust-семантикой.
//!
//! `stTopInfo` содержит четыре 32-битных поля `lID/lTimerFlag/lParam/
//! dwStartTime` и byte-exact `std::string strInfo`; list-node размером `0x34`
//! подтверждает значение размером `0x2C`. Rust не копирует старый ABI:
//! `VecDeque<StTopInfo>` сохраняет хвостовую вставку и порядок обхода, `Vec<u8>`
//! — полное содержимое строки, а `Drop` — освобождение list/string storage.
//! Public Rust-граница принимает строковые bytes как `&[u8]`, потому что
//! by-value уничтожение входного MSVC `std::string` было только механизмом
//! владения; наблюдаемое копирование в запись либо сообщение сохранено.
//! Process-static `GetTopInfoID::lID` по exact адресу `0x0056A888` начинается с
//! `1`. Exact EXE `0x00436987..0x00436A68` возвращает прежний ID, wrapping
//! увеличивает static до снятия tick и не возвращает результат
//! `__security_check_cookie`; безопасный `AtomicI32::fetch_add` сохраняет
//! process-lifetime и 32-битное переполнение без исходной data race.
//!
//! Broadcast `SendTopInfoToClient` строит `0x7FA04` с player ID `0`, затем
//! `ID/timer/param/info\0`. Персональная отправка при непустом списке один раз
//! получает GameServer ID игрока и один wrapping boot tick, проходит записи в
//! порядке list и для `timer == 2` пропускает `elapsed >= param`; иначе в wire
//! попадает wrapping `param - elapsed`. Остальные timer-флаги передают исходный
//! param. Внутренний NUL сохранённой `std::string` обрезает именно C-string wire,
//! но не storage. Готовые World `CMessage` и `CGame` routing сохраняют
//! `SendAll/SendToMapID`; их исходно игнорировавшиеся результаты лишь входят в
//! typed Rust-отчёт и не останавливают следующий элемент. Пустой список не
//! читает GameServer registry и не снимает tick.
//!
//! В отличие от персональной отправки, top-info хвост `Run` безусловно снимает
//! ровно один tick даже при пустом списке. Exact EXE `0x0043A6BD..0x0043A742`
//! подтверждает unsigned условие удаления `timer == 2 && param <= now-start`,
//! предварительное сохранение следующего узла, уменьшение list-size и полный
//! проход до sentinel после каждого erase. `VecDeque::retain` сохраняет порядок
//! оставшихся записей, а `Drop` заменяет освобождение node/string. Этот участок
//! не читает и не меняет process-static top-info ID. Остальные поля полного
//! constructor-а остаются raw. Первая часть полного `Run` проходит faction-map
//! в signed order, уменьшает только положительный `m_lDelRemainTime` через
//! wrapping subtraction, собирает достигшие `<= 0` пары `(faction ID,
//! master ID)` во временный ordered map и лишь после traversal вызывает
//! `DisbandFaction(master, faction)`. `Run` принимает этот достигнутый вызов как
//! явный callback, чтобы caller мог передать Game/context без самозаимствования
//! controller-а; concrete `disband_faction` теперь материализован ниже.
//!
//! `UpdateOtherFacInfoToClient` проходит оставшиеся faction-owner-ы в signed
//! map-order, пропускает null value и вызывает concrete update slot `+0x170`.
//! `DisbandFaction` сначала повторяет standard/village и city/attack gates с
//! собственными `WS0235/WS0236 + WS0121`, очищает city enemy-set и вызывает
//! внутренний `CFaction::Disband`. После success exact order: erase map-owner,
//! второй `DeleteOrgaToClient(0)`, append ID в `m_DeleteFactions`, broadcast
//! empty-name `OP_Delete` оставшимся faction-ам, сброс player faction-data flag,
//! optional log и virtual delete. Exact ASM `0x00438550..0x004389CA`
//! подтверждает двойную delete-рассылку и исправляет raw dataflow: faction ID
//! в update/log — второй аргумент, player ID — первый. Наблюдаемая дубликация
//! сохранена; утечка 256-байтового SQL buffer при offline player устранена как
//! чисто внутренний дефект, а DB/war/player owners оставлены узким context.
//!
//! Billboard snapshot очищается и строится из faction-map: null value
//! пропускается, ID/name/value копируются из concrete faction. Specialized
//! `std::map::insert` по exact ASM `0x00436AF0..0x00436BEB` сортирует signed
//! value по убыванию; при равенстве member-таблица ставит более раннее время
//! основания первой, offense/defence — более позднее. `tagTime::operator>`
//! сравнивает year/month/day/hour/minute/second и игнорирует day-of-week с
//! milliseconds. Равные value и эти шесть полей являются одним map-key:
//! faction-ID tie-break отсутствует, поэтому остаётся первая faction из
//! signed map-order. Stable Rust sort и явное удаление повторных key сохраняют
//! этот эффект; старый Linux-донор ошибочно добавлял tie-break по faction ID.
//! Отсутствующие reached time/property останавливают конкретный stat-owner
//! после очистки его списка, не получая выдуманного default.
//!
//! Три serializer-а пишут `min(unsigned list size, unsigned configured size)`,
//! затем для каждого элемента `faction ID, name\0, value`; constructor задаёт
//! size `10`. Dispatcher намеренно принимает members/offense/defence как
//! `1/2/3`, иначе пишет нулевой count. Запрос `0x60125` допускает лишь `0/1/2`,
//! поэтому type `0` получает пустой payload, `1` — members, `2` — offense, а
//! defence через этот opcode недостижим. Наблюдаемый off-by-one подтверждён
//! EXE и не «исправлен» по старому донору. Запрос сериализует сохранённый
//! snapshot без скрытого пересчёта: его lifecycle остаётся `Initialize` и
//! явным `StatBillboard` после завершения city-war.
//!
//! `OnAttackCityEnd` exact ASM `0x00437C70..0x00438003` сначала разрешает
//! faction атакующего игрока, затем копирует byte-exact имя региона (miss/null
//! дают пустую строку) и только после этого применяет zero-faction gate;
//! далее независимо поднимает attacker/defender до union-owner-а.
//! Ненулевой defender обязан владеть регионом. При результате `1` порядок
//! side effects: offense victor атакующему owner-у, `DelOwnedCity` защитнику,
//! `WS0261`, `AddOwnedCity` атакующему и `RefreshOwnedCityOrg(region,
//! attacker faction, attacker union)`. При `0`: defence victor защитнику и
//! `WS0262`; прочие значения ничего не меняют. Union virtual-ы используют уже
//! подтверждённые fan-out и legacy `DelOwnedCity`, который очищает city-list
//! master-фракции вместо удаления одного ID.
//!
//! В ветви `WS0262` машина берёт имя attacker owner-а, а не defender, и
//! передаёт 28-байтовый MSVC `std::string` в variadic `sprintf` без `c_str()`.
//! Выбор attacker-name сохранён как наблюдаемая семантика; UB/зависимость от
//! SSO исправлены безопасной visible C-string-проекцией. Исходные region
//! `char[0x100]` и два notice `char[0x400]` заменены явными границами;
//! уже завершённые счётчики при переполнении notice не откатываются.

use std::any::Any;
use std::cell::Cell;
use std::cmp::Ordering as CmpOrdering;
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use rustix::time::{ClockId, clock_gettime};

use super::attackcitysys::CAttackCitySys;
use super::faction::{
    CFaction, FactionCloneSaveBlock, FactionContributorBlock, FactionContributorContext,
    FactionContributorOutcome, FactionDeleteOrganizingBuildError,
    FactionDeleteOrganizingOutcome, FactionDisbandBlock, FactionDisbandContext,
    FactionDisbandOutcome, FactionDisbandProgress, FactionDisbandRejection,
    FactionExperienceBlock, FactionExperienceUpdate,
    FactionEditLeaveWordOutcome, FactionEnemyDelivery, FactionEnemyMutationBlock,
    FactionEnemyMutationContext, FactionEnemyWarLogArgument, FactionFeatureFunctionUpdate,
    FactionInitialPropertyBlock,
    FactionLeaveWordBlock, FactionLeaveWordOutcome, FactionMemberInfoReport,
    FactionMemberInfoRequest,
    FactionOperationAuthorityContext, FactionOperationBlock, FactionOperationOutcome,
    FactionOrganizingInfoContext, FactionOtherInfoBuildError,
    FactionOtherInfoDelivery, FactionOwnedCityDelivery, FactionOwnedCityRefreshBlock,
    FactionOwnedCityRefreshReport, FactionOwnedCityUpdateBuildError, FactionPlayerHeaderContext,
    FactionPermitBlock, FactionPermitUpdate,
    FactionPronounceBlock, FactionPronounceOutcome, FactionPropertyDelivery,
    FactionPropertyReinitialization, FactionRemoveApplyMemberOutcome, FactionSuperiorOrganizingBlock,
    FactionUpgradeBlock, FactionUpgradeContext, FactionUpgradeOutcome, FactionUploadIconBlock,
    FactionUploadIconContext, FactionUploadIconOutcome,
    MemberEnterOutcome, MemberExitOutcome, MemberLevelChangeOutcome,
    MemberPositionChangeOutcome, OwnedCityAddOutcome, OwnedCityBooleanMutationReport,
    OwnedCityMutationBuildError,
};
use super::factionwarsys::{
    CFactionWarSys, FactionWarDeclarationContext, FactionWarFactionSnapshot,
    FactionWarFormatArgument,
};
use super::organizing::{ECityState, EOperator, TagTimeValue};
use super::organizingparam::COrganizingParam;
use super::union::{
    CUnion, UnionAddFactionEffects, UnionApplicationFactionBlock,
    UnionApplicationFactionSnapshot, UnionApplicationTerminal, UnionApplyForJoinBlock,
    UnionApplyForJoinContext, UnionApplyForJoinEffects, UnionApplyForJoinOutcome,
    UnionClientSnapshotContext, UnionDoJoinBlock, UnionDoJoinContext, UnionDoJoinOutcome,
    UnionFactionJoinContext, UnionFactionLevelBlock, UnionFactionMemberContext,
    UnionFactionStateMutationContext, UnionInitialMutationContext, UnionMasterFactionQueryContext,
    UnionMemberSnapshotBlock, UnionOperatorValidationContext, UnionOwnedCityBooleanMutationReport,
    UnionOwnedCityFanoutReport, UnionOwnedCityMutationBlock, UnionOwnedCityMutationContext,
    UnionFormatArgument, UnionPlayerRefreshContext, UnionSendInfoContext,
    UnionVictorFanoutReport, UnionVictorMutationBlock,
};
use super::villagewarsys::CVillageWarSys;
use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::public::netsession::{
    NetSessionAsyncResult, NetSessionAsyncResultKind, NetSessionEndpoint,
};
use crate::public::netsessionmanager::{
    CNetSessionManager, CreatedNetSession, NetSessionCreateBlock, NetSessionManagerBeginBlock,
    NetSessionSetCallbackBlock,
};
use crate::worldserver::appworld::country::countryhandler::CCountryHandler;
use crate::worldserver::appworld::player::{
    PlayerOrganizingState, PlayerOrganizingUpdateError, PlayerOrganizingUpdater,
};
use crate::worldserver::worldserver::game::{CGame, WorldRegionNameLookup};

const TOP_INFO_MESSAGE_TYPE: i32 = 0x7FA04;
const UNION_INITIAL_MESSAGE_TYPE: i32 = 0x7FE04;
const EXPIRING_TIMER_FLAG: i32 = 2;
const DECLARE_WAR_FACTION_PAGE_SIZE: i32 = 11;
const DEFAULT_FACTION_BILLBOARD_SIZE: i32 = 10;
const CITY_TRANSFER_CONFIRMATION_MESSAGE_TYPE: i32 = 0x7FE2B;

const UNUSED_UNION_APPLICATION_TIME: TagTimeValue = TagTimeValue {
    year: 0,
    month: 0,
    day_of_week: 0,
    day: 0,
    hour: 0,
    minute: 0,
    second: 0,
    milliseconds: 0,
};

static NEXT_TOP_INFO_ID: AtomicI32 = AtomicI32::new(1);

/// Достигнутая семантика исходного `stTopInfo` без копирования Windows ABI.
struct StTopInfo {
    id: i32,
    timer_flag: i32,
    param: i32,
    started_at_ms: u32,
    info: Vec<u8>,
}

/// Rust-представление исходного `tagFacBillboard` без MSVC string ABI.
#[derive(Clone, Debug, Eq, PartialEq)]
struct FactionBillboardEntry {
    faction_id: i32,
    name: Vec<u8>,
    number: i32,
}

/// Конкретный billboard, на котором остановился общий snapshot-проход.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionBillboardKind {
    MemberCount,
    OffenseVictories,
    DefenceVictories,
}

/// Safe-граница ещё не загруженного reached-поля concrete faction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionBillboardStatBlock {
    MissingEstablishedTime {
        map_key: i32,
        billboard: FactionBillboardKind,
    },
    MissingBaseProperty {
        map_key: i32,
        billboard: FactionBillboardKind,
    },
}

/// Результат одной исходно игнорировавшейся отправки top-info.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct TopInfoDelivery {
    pub(crate) top_info_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

/// Отчёт полного list-прохода для одного игрока.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct TopInfoDeliveryReport {
    /// `None` означает точную раннюю ветвь пустого `m_TopInfos`.
    pub(crate) game_server_id: Option<i32>,
    pub(crate) skipped_expired: usize,
    pub(crate) deliveries: Vec<TopInfoDelivery>,
}

/// Наблюдаемая отправка первого overload `SendOrgaInfoToClient`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingInfoDelivery {
    RouteRejected {
        recipient_player_id: i32,
    },
    Sent {
        recipient_player_id: i32,
        game_server_id: i32,
        result: Result<i32, SendMessageError>,
    },
}

/// Safe-границы concrete controller-owner-а для объявления войны.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionWarDeclarationBlock {
    MasterLookup(FactionMasterLookupBlock),
    MissingFactionProperty { faction_id: i32 },
    UnionMembership { map_key: i32 },
    MissingFactionInUnion { union_id: i32, faction_id: i32 },
    MissingFactionForMutation { faction_id: i32 },
    EnemyMutation {
        faction_id: i32,
        enemy_id: i32,
        source: FactionEnemyMutationBlock,
    },
}

/// Concrete adapter controller/player/string/transport owner-ов войны.
pub(crate) struct WorldFactionWarDeclarationEffects<'a> {
    game: &'a CGame,
    organizing: &'a mut COrganizingCtrl,
    world_string: &'a mut dyn FnMut(&[u8]) -> Vec<u8>,
    format_world_string:
        &'a mut dyn FnMut(&[u8], &[UnionFormatArgument<'_>]) -> Vec<u8>,
    put_war_log: &'a mut dyn FnMut(&[u8]),
    update_player: &'a mut dyn FnMut(i32),
}

impl<'a> WorldFactionWarDeclarationEffects<'a> {
    pub(crate) fn new(
        game: &'a CGame,
        organizing: &'a mut COrganizingCtrl,
        world_string: &'a mut dyn FnMut(&[u8]) -> Vec<u8>,
        format_world_string: &'a mut dyn FnMut(
            &[u8],
            &[UnionFormatArgument<'_>],
        ) -> Vec<u8>,
        put_war_log: &'a mut dyn FnMut(&[u8]),
        update_player: &'a mut dyn FnMut(i32),
    ) -> Self {
        Self {
            game,
            organizing,
            world_string,
            format_world_string,
            put_war_log,
            update_player,
        }
    }
}

struct DeclarationEnemyMutationEffects<'a> {
    enemy_id: i32,
    enemy_name: Vec<u8>,
    format_world_string:
        &'a mut dyn FnMut(&[u8], &[UnionFormatArgument<'_>]) -> Vec<u8>,
    put_war_log: &'a mut dyn FnMut(&[u8]),
}

impl FactionEnemyMutationContext for DeclarationEnemyMutationEffects<'_> {
    fn organizing_name(&self, organizing_id: i32) -> Option<Vec<u8>> {
        (organizing_id == self.enemy_id).then(|| self.enemy_name.clone())
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[FactionEnemyWarLogArgument<'_>],
    ) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                FactionEnemyWarLogArgument::Text(text) => UnionFormatArgument::Text(text),
                FactionEnemyWarLogArgument::Unsigned(value) => {
                    UnionFormatArgument::Signed(*value as i32)
                }
            })
            .collect::<Vec<_>>();
        (self.format_world_string)(string_id, &arguments)
    }

    fn put_war_log(&mut self, text: &[u8]) {
        (self.put_war_log)(text);
    }
}

impl FactionWarDeclarationContext for WorldFactionWarDeclarationEffects<'_> {
    type Block = OrganizingFactionWarDeclarationBlock;

    fn faction_id_by_master_player(&self, player_id: i32) -> Result<i32, Self::Block> {
        self.organizing
            .faction_id_by_master_player(player_id)
            .map_err(OrganizingFactionWarDeclarationBlock::MasterLookup)
    }

    fn faction_snapshot(
        &self,
        faction_id: i32,
    ) -> Result<Option<FactionWarFactionSnapshot>, Self::Block> {
        let Some(faction) = self.organizing.faction_by_id(faction_id) else {
            return Ok(None);
        };
        let superior_organizing_id = faction.superior_organizing().ok_or(
            OrganizingFactionWarDeclarationBlock::MissingFactionProperty { faction_id },
        )?;
        Ok(Some(FactionWarFactionSnapshot {
            faction_id: faction.faction_id(),
            superior_organizing_id,
        }))
    }

    fn faction_side(&self, root_faction_id: i32) -> Result<Vec<i32>, Self::Block> {
        match self.organizing.is_free_faction(root_faction_id) {
            FreeFactionLookup::NoUnion => Ok(vec![root_faction_id]),
            FreeFactionLookup::BlockedNullConfederation { map_key } => Err(
                OrganizingFactionWarDeclarationBlock::UnionMembership { map_key },
            ),
            FreeFactionLookup::Union(union_id) => {
                let Some(union) = self.organizing.confederation_by_id(union_id) else {
                    return Ok(Vec::new());
                };
                let members = union.member_ids_snapshot();
                if let Some(&faction_id) = members
                    .iter()
                    .find(|&&faction_id| self.organizing.faction_by_id(faction_id).is_none())
                {
                    return Err(
                        OrganizingFactionWarDeclarationBlock::MissingFactionInUnion {
                            union_id,
                            faction_id,
                        },
                    );
                }
                Ok(members)
            }
        }
    }

    fn player_money(&self, player_id: i32) -> Option<u32> {
        self.game
            .online_player_by_id(player_id as u32)
            .map(|player| player.money())
    }

    fn add_enemy_organizing(
        &mut self,
        faction_id: i32,
        enemy_id: i32,
    ) -> Result<(), Self::Block> {
        let enemy_name = self
            .organizing
            .faction_by_id(enemy_id)
            .map(|faction| faction.name().to_vec())
            .ok_or(OrganizingFactionWarDeclarationBlock::MissingFactionForMutation {
                faction_id: enemy_id,
            })?;
        let faction = self.organizing.faction_by_id_mut(faction_id).ok_or(
            OrganizingFactionWarDeclarationBlock::MissingFactionForMutation { faction_id },
        )?;
        let mut effects = DeclarationEnemyMutationEffects {
            enemy_id,
            enemy_name,
            format_world_string: &mut *self.format_world_string,
            put_war_log: &mut *self.put_war_log,
        };
        faction
            .add_enemy_organizing(enemy_id, &mut effects)
            .map(|_| ())
            .map_err(|source| OrganizingFactionWarDeclarationBlock::EnemyMutation {
                faction_id,
                enemy_id,
                source,
            })
    }

    fn update_enemy_faction(&mut self, faction_id: i32) -> Result<(), Self::Block> {
        let faction = self.organizing.faction_by_id_mut(faction_id).ok_or(
            OrganizingFactionWarDeclarationBlock::MissingFactionForMutation { faction_id },
        )?;
        let _ = faction.update_enemy_faction(self.game, &mut *self.update_player);
        Ok(())
    }

    fn organizing_name(&self, faction_id: i32) -> Result<Vec<u8>, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .map(|faction| faction.name().to_vec())
            .ok_or(OrganizingFactionWarDeclarationBlock::MissingFactionForMutation { faction_id })
    }

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        (self.world_string)(string_id)
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[FactionWarFormatArgument<'_>],
    ) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                FactionWarFormatArgument::Text(text) => UnionFormatArgument::Text(text),
                FactionWarFormatArgument::Signed(value) => UnionFormatArgument::Signed(*value),
            })
            .collect::<Vec<_>>();
        (self.format_world_string)(string_id, &arguments)
    }

    fn send_player_info(
        &mut self,
        player_id: i32,
        first_text: &[u8],
        second_text: &[u8],
    ) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(
            self.game,
            FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text,
                second_text,
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            },
        );
    }

    fn send_orga_info_to_all(&mut self, info: &[u8], kind: u32, color: u32) {
        let _ = COrganizingCtrl::send_organizing_info_to_all(self.game, info, kind, color);
    }

    fn put_war_log(&mut self, info: &[u8]) {
        (self.put_war_log)(info);
    }
}

/// Typed-результат ordered `m_FacOrg` scan вместо старого null-dereference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FreePlayerLookup {
    NoFaction,
    Faction(i32),
    BlockedNullFaction { map_key: i32 },
}

/// Typed-результат ordered `m_ConfedeOrganizings` scan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FreeFactionLookup {
    NoUnion,
    Union(i32),
    BlockedNullConfederation { map_key: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AddUnionToFactionRejection {
    UnionIdNotResolved { requested_union_id: i32 },
    UnionEntryMissing { union_id: i32 },
    NullUnionPointer { union_id: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct AddUnionToFactionDelivery {
    pub(crate) member_map_key: i32,
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum AddUnionToFactionOutcome {
    Rejected(AddUnionToFactionRejection),
    Sent {
        union_id: i32,
        faction_id: i32,
        deliveries: Vec<AddUnionToFactionDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum AddUnionToFactionBlock {
    FreeFactionScan { map_key: i32 },
    FactionUnavailable {
        faction_id: i32,
        entry_present: bool,
    },
    Snapshot {
        union_id: i32,
        faction_id: i32,
        member_map_key: i32,
        recipient_player_id: i32,
        game_server_id: i32,
        source: UnionMemberSnapshotBlock,
        completed_deliveries: Vec<AddUnionToFactionDelivery>,
    },
}

/// Safe-граница исходного null-dereference внутри `IsFreeFaction` scan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionUnionMembershipLookupBlock {
    pub(crate) map_key: i32,
}

/// Найденная master-faction союза ещё не имеет достигнутого master ID.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnionPlayerHeaderLookupBlock {
    pub(crate) master_faction_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionMasterLookupBlock {
    NullFaction { map_key: i32 },
    MissingMasterId { map_key: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingLeaveWordEnableOutcome {
    FactionNotFound,
    Updated {
        faction_id: i32,
        update: FactionFeatureFunctionUpdate,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingLeaveWordEnableBlock {
    MasterLookup(FactionMasterLookupBlock),
    Property {
        faction_id: i32,
        source: FactionInitialPropertyBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingLeaveWordOutcome {
    FactionNotFound,
    Applied {
        faction_id: i32,
        outcome: FactionLeaveWordOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingLeaveWordBlock {
    Membership { map_key: i32 },
    Faction {
        faction_id: i32,
        source: FactionLeaveWordBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingLeaveWordEditOutcome {
    FactionNotFound,
    Applied {
        faction_id: i32,
        outcome: FactionEditLeaveWordOutcome,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingLeaveWordEditBlock {
    Membership { map_key: i32 },
    Property {
        faction_id: i32,
        source: FactionInitialPropertyBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingPronounceOutcome {
    FactionNotFound,
    Applied {
        faction_id: i32,
        outcome: FactionPronounceOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingContributorOutcome {
    FactionNotFound,
    Applied {
        faction_id: i32,
        outcome: FactionContributorOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingContributorBlock {
    Membership { map_key: i32 },
    Contributor(FactionContributorBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionExperienceMutation {
    FactionNotFound,
    PlayerNotContributor,
    Applied {
        faction_id: i32,
        faction_name: Vec<u8>,
        before_experience: i32,
        update: FactionExperienceUpdate,
    },
}

/// Результат virtual callback-а состояния участника из `0x6012A`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionMemberStateOutcome {
    FactionNotFound,
    Level(MemberLevelChangeOutcome),
    Position(MemberPositionChangeOutcome),
    UnknownOperation,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingPronounceBlock {
    Membership { map_key: i32 },
    Faction {
        faction_id: i32,
        source: FactionPronounceBlock,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub(crate) enum DeclareWarFactionRelation {
    None = 0,
    SelfFaction = 1,
    SameUnion = 2,
    Enemy = 3,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct DeclareWarFactionEntry {
    pub(crate) faction_id: i32,
    pub(crate) country: u8,
    pub(crate) name: Vec<u8>,
    pub(crate) relation: DeclareWarFactionRelation,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct DeclareWarFactionPage {
    pub(crate) requested_page: i32,
    pub(crate) normalized_page: i32,
    pub(crate) total_factions: i32,
    pub(crate) entries: Vec<DeclareWarFactionEntry>,
    pub(crate) payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DeclareWarFactionPageBlock {
    FactionCountOutsideLegacyRange { count: usize },
    MissingSourceFaction { faction_id: i32 },
    NullFaction { map_key: i32 },
    MissingCountry { faction_id: i32 },
    UnionMembership { faction_id: i32, map_key: i32 },
}

/// Safe-границы точной цепочки `GetUnion(player ID)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingUnionByMasterBlock {
    FactionMaster(FactionMasterLookupBlock),
    UnionMembership(FactionUnionMembershipLookupBlock),
}

pub(crate) type OrganizingUnionApplyForJoinBlock<SessionBlock> =
    UnionApplyForJoinBlock<FactionUnionMembershipLookupBlock, SessionBlock>;

/// Результат nullable `GetUnion` и последующего virtual `ApplyForJoin`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingUnionApplyForJoinOutcome<SessionReport> {
    UnionNotFound,
    Applied {
        union_id: i32,
        outcome: UnionApplyForJoinOutcome<SessionReport>,
    },
}

/// Ошибка достигнутого lookup либо safe-остановка уже выбранного union-owner-а.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingUnionApplyForJoinDispatchBlock<SessionBlock> {
    Lookup(OrganizingUnionByMasterBlock),
    Apply {
        union_id: i32,
        source: OrganizingUnionApplyForJoinBlock<SessionBlock>,
    },
}

pub(crate) type OrganizingUnionApplicationJoinBlock = UnionDoJoinBlock<
    FactionUnionMembershipLookupBlock,
    FactionMasterLookupBlock,
    AddUnionToFactionBlock,
>;

/// Normal-return terminal callback заявки на вступление в союз.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingUnionApplicationCallbackReport {
    pub(crate) union_id: i32,
    pub(crate) applicant_faction_id: i32,
    pub(crate) union_found: bool,
    pub(crate) rejection_notice_sent: bool,
    pub(crate) join: Option<UnionDoJoinOutcome>,
    pub(crate) application_cleared: bool,
    pub(crate) establishment_reservation_removed: bool,
}

/// Safe-остановка callback-а в месте недостижимого старого malformed state.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingUnionApplicationCallbackBlock {
    PlayerHeader {
        union_id: i32,
        applicant_faction_id: i32,
        source: UnionPlayerHeaderLookupBlock,
        rejection_notice_sent: bool,
    },
    DoJoin {
        union_id: i32,
        applicant_faction_id: i32,
        source: OrganizingUnionApplicationJoinBlock,
        rejection_notice_sent: bool,
    },
}

/// Результат одного вызова `CFaction::RemoveApplyMember` в map-order.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ApplyFactionRemoval {
    pub(crate) map_key: i32,
    pub(crate) outcome: FactionRemoveApplyMemberOutcome,
}

/// Полный normal-return либо точная null-pointer граница ordered прохода.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum RemovePersonFromApplyFactionListOutcome {
    Completed {
        removals: Vec<ApplyFactionRemoval>,
    },
    BlockedNullFaction {
        map_key: i32,
        completed_removals: Vec<ApplyFactionRemoval>,
    },
}

/// Результат поиска первой faction, содержащей player в apply-list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ApplyFactionLookup {
    NoFaction,
    Faction(i32),
    BlockedNullFaction { map_key: i32 },
}

/// Локальная safe-граница ordered `GenerateSaveData` traversal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingSaveDataBlock {
    NullFaction {
        map_key: i32,
    },
    FactionClone {
        map_key: i32,
        reason: FactionCloneSaveBlock,
    },
    NullConfederation {
        map_key: i32,
    },
}

/// Счётчики полностью завершённого `GenerateSaveData` прохода.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrganizingSaveDataReport {
    pub(crate) saved_factions: usize,
    pub(crate) saved_unions: usize,
    pub(crate) deleted_factions: usize,
    pub(crate) deleted_unions: usize,
}

/// Локальная safe-граница полного faction countdown traversal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingRunBlock {
    NullFaction { map_key: i32 },
    DeleteRemainTimeAbsent { map_key: i32 },
    MasterIdAbsent { map_key: i32, faction_id: i32 },
}

/// Один вызов отдельного `DisbandFaction(master, faction)` owner-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrganizingRunDisband {
    pub(crate) faction_id: i32,
    pub(crate) master_id: i32,
    pub(crate) result: bool,
}

/// Полный результат `COrganizingCtrl::Run` после normal return callbacks.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingRunReport {
    pub(crate) decremented_factions: usize,
    pub(crate) disbands: Vec<OrganizingRunDisband>,
    pub(crate) expired_top_infos: usize,
}

/// Фактически выбранная faction-ветвь enter callback-а.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionEnterDispatch {
    NoFactionMembership,
    FactionEntryMissing {
        faction_id: i32,
    },
    NullFactionPointer {
        faction_id: i32,
    },
    Called {
        faction_id: i32,
        outcome: MemberEnterOutcome,
    },
}

/// Полный результат `COrganizingCtrl::OnPlayerEnterGame`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum PlayerEnterGameOutcome {
    BlockedDuringFactionScan {
        map_key: i32,
    },
    BlockedDuringFactionCallback {
        faction_id: i32,
        outcome: MemberEnterOutcome,
    },
    Completed {
        faction: FactionEnterDispatch,
        top_info: TopInfoDeliveryReport,
    },
}

/// Фактически выбранная faction-ветвь exit callback-а.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionExitDispatch {
    NoFactionMembership,
    FactionEntryMissing {
        faction_id: i32,
    },
    NullFactionPointer {
        faction_id: i32,
    },
    Called {
        faction_id: i32,
        outcome: MemberExitOutcome,
    },
}

/// Полный результат `COrganizingCtrl::OnPlayerExitGame`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum PlayerExitGameOutcome {
    BlockedDuringFactionScan {
        map_key: i32,
    },
    BlockedDuringFactionCallback {
        faction_id: i32,
        outcome: MemberExitOutcome,
    },
    Dispatched(FactionExitDispatch),
}

/// Один успешно переинициализированный faction-owner в signed map-order.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionReinitializationEntry {
    pub(crate) map_key: i32,
    pub(crate) result: FactionPropertyReinitialization,
}

/// Safe-граница неполного concrete `CFaction` внутри старого pointer-map.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionReinitializationBlock {
    pub(crate) map_key: i32,
    pub(crate) source: FactionInitialPropertyBlock,
    /// Уже выполненные публикации исходный ordered проход не откатывал бы.
    pub(crate) completed: Vec<FactionReinitializationEntry>,
}

/// Нормальный результат исходного `CUnion::DelMember`, всегда возвращавшего true.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionMemberDetachOutcome {
    NonPositiveFactionId,
    FactionEntryMissing,
    NullFactionPointer,
    Detached {
        deliveries: Vec<FactionPropertyDelivery>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnionMemberDetachBlockSource {
    SuperiorOrganizing(FactionSuperiorOrganizingBlock),
    Property(FactionInitialPropertyBlock),
}

/// Safe-граница частично материализованного faction-owner-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnionMemberDetachBlock {
    pub(crate) faction_id: i32,
    pub(crate) source: UnionMemberDetachBlockSource,
}

/// Один concrete faction-result controller-wide other-faction broadcast-а.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingOtherFactionUpdate {
    pub(crate) map_key: i32,
    pub(crate) deliveries: Vec<FactionOtherInfoDelivery>,
}

/// Safe-граница после уже выполненного prefix-а signed map traversal.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingOtherFactionUpdateBlock {
    pub(crate) map_key: i32,
    pub(crate) source: FactionOtherInfoBuildError,
    pub(crate) completed: Vec<OrganizingOtherFactionUpdate>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OrganizingDisbandPlayer {
    pub(crate) player_id: i32,
    pub(crate) player_name: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDisbandRejection {
    FactionEntryMissing,
    StandardWar,
    CityWar,
    Faction(FactionDisbandRejection),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingDisbandProgress {
    pub(crate) cleared_city_war_enemies: usize,
    pub(crate) faction: FactionDisbandProgress,
    pub(crate) map_removed: bool,
    pub(crate) second_delete_organizing: Option<FactionDeleteOrganizingOutcome>,
    pub(crate) delete_faction_queued: bool,
    pub(crate) other_faction_updates: Option<Vec<OrganizingOtherFactionUpdate>>,
    pub(crate) player: Option<OrganizingDisbandPlayer>,
    pub(crate) log_written: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDisbandOutcome {
    Rejected {
        reason: OrganizingDisbandRejection,
        notice_sent: bool,
        cleared_city_war_enemies: usize,
    },
    Disbanded(OrganizingDisbandProgress),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDisbandBlock {
    NullFaction {
        faction_id: i32,
    },
    Faction {
        source: FactionDisbandBlock,
        cleared_city_war_enemies: usize,
    },
    SecondDeleteOrganizing {
        source: FactionDeleteOrganizingBuildError,
        progress: OrganizingDisbandProgress,
    },
    OtherFactionUpdate {
        source: OrganizingOtherFactionUpdateBlock,
        progress: OrganizingDisbandProgress,
    },
}

/// Player/log continuation внешнего `COrganizingCtrl::DisbandFaction`.
pub(crate) trait OrganizingDisbandContext: FactionDisbandContext {
    fn clear_player_faction_data_received(
        &mut self,
        player_id: i32,
    ) -> Option<OrganizingDisbandPlayer>;

    fn faction_disband_log_enabled(&self) -> bool;

    fn write_faction_disband_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        player_id: i32,
        player_name: &[u8],
    );
}

/// Поля синхронного `PlayerTransferOwnerCity::DoAsyncCall`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CityTransferSessionRequest {
    pub(crate) requester_player_id: i32,
    pub(crate) source_faction_id: i32,
    pub(crate) target_master_player_id: i32,
    pub(crate) target_faction_id: i32,
    pub(crate) region_id: i32,
    pub(crate) requested_session_id: i32,
    pub(crate) timeout_ticks: u32,
    pub(crate) source_faction_name: Vec<u8>,
    pub(crate) region_name: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CityTransferTerminal {
    Approved,
    Denied,
    NonResult { kind: NetSessionAsyncResultKind },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CityTransferEndpointBlock {
    BeginPayloadType,
    ResultPayloadType,
}

/// Потокобезопасная граница callback-а и единственного organizing owner-а.
pub(crate) trait CityTransferSessionRuntime: Send + Sync {
    fn send_city_transfer_confirmation(&self, recipient_player_id: i32, message: &CMessage);

    fn finish_city_transfer(
        &self,
        source_faction_id: i32,
        target_faction_id: i32,
        region_id: i32,
        region_name: &[u8],
        terminal: CityTransferTerminal,
    );

    fn block_city_transfer_endpoint(&self, block: CityTransferEndpointBlock);
}

/// Safe owner локального `PlayerTransferOwnerCity` вместо двух C++ subobject-ов.
pub(crate) struct PlayerTransferOwnerCity {
    source_faction_id: i32,
    target_faction_id: i32,
    region_id: i32,
    region_name: Vec<u8>,
    runtime: Arc<dyn CityTransferSessionRuntime>,
}

impl NetSessionEndpoint for PlayerTransferOwnerCity {
    fn do_async_call(&self, session_id: i64, cookie_second: i32, payload: &dyn Any) {
        let Some(request) = payload.downcast_ref::<CityTransferSessionRequest>() else {
            self.runtime
                .block_city_transfer_endpoint(CityTransferEndpointBlock::BeginPayloadType);
            return;
        };

        let mut message = CMessage::new(CITY_TRANSFER_CONFIRMATION_MESSAGE_TYPE);
        message.base_mut().add_long(request.target_master_player_id);
        message
            .base_mut()
            .add(legacy_c_string_prefix(&request.source_faction_name));
        message.base_mut().add_byte(0);
        message
            .base_mut()
            .add(legacy_c_string_prefix(&request.region_name));
        message.base_mut().add_byte(0);
        message.base_mut().add_long64(session_id);
        message.base_mut().add_long(cookie_second);
        self.runtime
            .send_city_transfer_confirmation(request.target_master_player_id, &message);
    }

    fn on_async_callback(&self, result: NetSessionAsyncResult<'_>) {
        let terminal = if result.kind == NetSessionAsyncResultKind::Result {
            let Some(decision) = result
                .payload
                .and_then(|payload| payload.downcast_ref::<i32>())
            else {
                self.runtime
                    .block_city_transfer_endpoint(CityTransferEndpointBlock::ResultPayloadType);
                return;
            };
            if *decision == 1 {
                CityTransferTerminal::Approved
            } else {
                CityTransferTerminal::Denied
            }
        } else {
            CityTransferTerminal::NonResult { kind: result.kind }
        };
        self.runtime.finish_city_transfer(
            self.source_faction_id,
            self.target_faction_id,
            self.region_id,
            &self.region_name,
            terminal,
        );
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CityTransferSessionReport {
    pub(crate) session: CreatedNetSession,
}

pub(crate) enum CityTransferSessionBlock {
    Create(NetSessionCreateBlock),
    SetCallback {
        session: CreatedNetSession,
        source: NetSessionSetCallbackBlock,
    },
    Begin {
        session: CreatedNetSession,
        source: NetSessionManagerBeginBlock,
    },
}

impl std::fmt::Debug for CityTransferSessionBlock {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Create(source) => formatter.debug_tuple("Create").field(source).finish(),
            Self::SetCallback { session, source } => {
                let source = match source {
                    NetSessionSetCallbackBlock::SessionNotFound { .. } => "SessionNotFound",
                    NetSessionSetCallbackBlock::AlreadyAssigned(_) => "AlreadyAssigned",
                };
                formatter
                    .debug_struct("SetCallback")
                    .field("session", session)
                    .field("source", &source)
                    .finish()
            }
            Self::Begin { session, source } => formatter
                .debug_struct("Begin")
                .field("session", session)
                .field("source", source)
                .finish(),
        }
    }
}

pub(crate) fn begin_city_transfer_session(
    manager: &CNetSessionManager,
    request: CityTransferSessionRequest,
    runtime: Arc<dyn CityTransferSessionRuntime>,
    random: impl FnMut(i32) -> i32,
) -> Result<CityTransferSessionReport, CityTransferSessionBlock> {
    let session = manager
        .create_session(
            request.target_master_player_id,
            request.requested_session_id,
            random,
        )
        .map_err(CityTransferSessionBlock::Create)?;
    let endpoint = Box::new(PlayerTransferOwnerCity {
        source_faction_id: request.source_faction_id,
        target_faction_id: request.target_faction_id,
        region_id: request.region_id,
        region_name: request.region_name.clone(),
        runtime,
    });
    manager
        .set_callback_handle(session.id, endpoint)
        .map_err(|source| CityTransferSessionBlock::SetCallback { session, source })?;
    manager
        .beging(session.id, request.timeout_ticks, &request)
        .map_err(|source| CityTransferSessionBlock::Begin { session, source })?;
    Ok(CityTransferSessionReport { session })
}

pub(crate) trait CityTransferEffects {
    type SessionReport;
    type SessionBlock;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8>;
    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>);
    fn begin_city_transfer_session(
        &mut self,
        request: CityTransferSessionRequest,
    ) -> Result<Self::SessionReport, Self::SessionBlock>;
    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8>;
    fn refresh_owned_city(&mut self, region_id: i32, faction_id: i32, union_id: i32);
    fn broadcast_city_transfer(&mut self, text: &[u8]) -> Result<i32, SendMessageError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CityTransferRejection {
    RegionNotFound,
    NullRegionPointer,
    NonPositiveRegion,
    CityWarActive { notice_sent: bool },
    MasterFactionNotFound,
    ZeroTargetFactionId,
    SourceFactionMissing,
    TargetFactionMissing,
    CountriesDiffer { notice_sent: bool },
    SourceMasterIsCountryKing { notice_sent: bool },
    SourceDoesNotOwnRegion,
    TargetAlreadyOwnsCity { notice_sent: bool },
    TargetDeclaredAttackWar { notice_sent: bool },
    TargetDeclaredVillageWar { notice_sent: bool },
    SourceFactionReserved { notice_sent: bool },
    TargetFactionReserved { notice_sent: bool },
    TargetMasterOffline { notice_sent: bool },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CityTransferStartOutcome<SessionReport> {
    Rejected(CityTransferRejection),
    Started {
        source_faction_id: i32,
        target_master_player_id: i32,
        net_exchange_id: i32,
        session: SessionReport,
    },
}

#[derive(Debug)]
pub(crate) enum CityTransferStartBlock<SessionBlock> {
    FactionMaster(FactionMasterLookupBlock),
    MissingSourceCountry { faction_id: i32 },
    MissingTargetCountry { faction_id: i32 },
    MissingSourceMaster { faction_id: i32 },
    MissingTargetMaster { faction_id: i32 },
    Session {
        source_faction_id: i32,
        target_faction_id: i32,
        source_reserved: bool,
        target_reserved: bool,
        source: SessionBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CityTransferFinishReport {
    pub(crate) source_reservation_removed: bool,
    pub(crate) target_reservation_removed: bool,
    pub(crate) source_faction_found: bool,
    pub(crate) target_faction_found: bool,
    pub(crate) source_cities: Option<OwnedCityBooleanMutationReport>,
    pub(crate) target_city: Option<OwnedCityAddOutcome>,
    pub(crate) target_union_id: Option<i32>,
    pub(crate) broadcast: Option<Result<i32, SendMessageError>>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CityTransferFinishBlock {
    SourceCities {
        report: CityTransferFinishReport,
        source: OwnedCityMutationBuildError,
    },
    TargetCity {
        report: CityTransferFinishReport,
        source: OwnedCityMutationBuildError,
    },
    TargetUnion {
        report: CityTransferFinishReport,
        map_key: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CityWarOrganizingOwner {
    Faction(i32),
    Union(i32),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CityWarVictorMutation {
    Faction(Vec<FactionPropertyDelivery>),
    Union(UnionVictorFanoutReport),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CityWarOwnedCityRemoval {
    Faction(OwnedCityBooleanMutationReport),
    Union(UnionOwnedCityBooleanMutationReport),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CityWarOwnedCityAddition {
    Faction(OwnedCityAddOutcome),
    Union(UnionOwnedCityFanoutReport),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct AttackCityEndReport {
    pub(crate) attacker_faction_id: Option<i32>,
    pub(crate) attacker_union_id: Option<i32>,
    pub(crate) attacker_owner: Option<CityWarOrganizingOwner>,
    pub(crate) defender_union_id: Option<i32>,
    pub(crate) defender_owner: Option<CityWarOrganizingOwner>,
    pub(crate) defender_owns_city: Option<bool>,
    pub(crate) offense_victors: Option<CityWarVictorMutation>,
    pub(crate) defender_city_removal: Option<CityWarOwnedCityRemoval>,
    pub(crate) attacker_city_addition: Option<CityWarOwnedCityAddition>,
    pub(crate) defence_victors: Option<CityWarVictorMutation>,
    pub(crate) refreshed_owner: Option<(i32, i32, i32)>,
    pub(crate) broadcast: Option<Result<i32, SendMessageError>>,
}

impl AttackCityEndReport {
    const fn pending() -> Self {
        Self {
            attacker_faction_id: None,
            attacker_union_id: None,
            attacker_owner: None,
            defender_union_id: None,
            defender_owner: None,
            defender_owns_city: None,
            offense_victors: None,
            defender_city_removal: None,
            attacker_city_addition: None,
            defence_victors: None,
            refreshed_owner: None,
            broadcast: None,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CityWarVictorMutationBlock {
    Faction(FactionInitialPropertyBlock),
    Union(UnionVictorMutationBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CityWarOwnedCityMutationBlock {
    Faction(OwnedCityMutationBuildError),
    Union(UnionOwnedCityMutationBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum AttackCityEndBlock {
    RegionNameWouldOverflow {
        report: AttackCityEndReport,
        visible_len: usize,
    },
    AttackerMembership {
        report: AttackCityEndReport,
        map_key: i32,
    },
    AttackerUnionMembership {
        report: AttackCityEndReport,
        map_key: i32,
    },
    DefenderUnionMembership {
        report: AttackCityEndReport,
        map_key: i32,
    },
    OffenseVictors {
        report: AttackCityEndReport,
        source: CityWarVictorMutationBlock,
    },
    DefenderCityRemoval {
        report: AttackCityEndReport,
        source: CityWarOwnedCityMutationBlock,
    },
    AttackerCityAddition {
        report: AttackCityEndReport,
        source: CityWarOwnedCityMutationBlock,
    },
    DefenceVictors {
        report: AttackCityEndReport,
        source: CityWarVictorMutationBlock,
    },
    MissingAttackerOwnerForDefenceNotice {
        report: AttackCityEndReport,
    },
    NoticeWouldOverflow {
        report: AttackCityEndReport,
        string_id: &'static [u8],
        formatted_len: usize,
    },
}

pub(crate) trait AttackCityEndEffects {
    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8>;

    fn refresh_owned_city(&mut self, region_id: i32, faction_id: i32, union_id: i32);

    fn broadcast_city_war_result(&mut self, text: &[u8]) -> Result<i32, SendMessageError>;
}

fn send_city_transfer_notice<Effects>(
    effects: &mut Effects,
    requester_player_id: i32,
    text_id: &'static [u8],
) where
    Effects: CityTransferEffects,
{
    let second_text = effects.world_string(b"WS0193");
    let first_text = effects.world_string(text_id);
    effects.send_organizing_info(FactionMemberInfoRequest {
        recipient_player_id: requester_player_id,
        first_text: legacy_c_string_prefix(&first_text),
        second_text: legacy_c_string_prefix(&second_text),
        information_type: -1,
        color: 0xFFDA_EDFE,
        trailing_value: 0,
    });
}

/// Достигнутые faction-callback и top-info части исходного singleton owner-а.
pub(crate) struct COrganizingCtrl {
    factions: BTreeMap<i32, Option<Box<CFaction>>>,
    confederations: BTreeMap<i32, Option<Box<CUnion>>>,
    delete_factions: VecDeque<i32>,
    delete_unions: VecDeque<i32>,
    top_infos: VecDeque<StTopInfo>,
    request_establishment_union_players: VecDeque<i32>,
    faction_billboard_size: i32,
    member_count_billboard: Vec<FactionBillboardEntry>,
    offense_victories_billboard: Vec<FactionBillboardEntry>,
    defence_victories_billboard: Vec<FactionBillboardEntry>,
    detached_union_membership_lookup: Cell<Option<(i32, FreeFactionLookup)>>,
}

/// Разделённый borrow faction-map для union snapshot во время mutable union lookup.
struct UnionFactionMapView<'a> {
    factions: &'a BTreeMap<i32, Option<Box<CFaction>>>,
}

/// Явная per-call замена singleton/controller и `CGame::s_mapRegionList`.
pub(crate) struct COrganizingPlayerUpdater<'a> {
    controller: &'a COrganizingCtrl,
    region_types: &'a BTreeMap<i32, Option<u16>>,
}

impl COrganizingCtrl {
    /// Создаёт доказанные пустые organizing-map и `m_TopInfos`.
    pub(crate) const fn with_reached_callback_state() -> Self {
        Self {
            factions: BTreeMap::new(),
            confederations: BTreeMap::new(),
            delete_factions: VecDeque::new(),
            delete_unions: VecDeque::new(),
            top_infos: VecDeque::new(),
            request_establishment_union_players: VecDeque::new(),
            faction_billboard_size: DEFAULT_FACTION_BILLBOARD_SIZE,
            member_count_billboard: Vec::new(),
            offense_victories_billboard: Vec::new(),
            defence_victories_billboard: Vec::new(),
            detached_union_membership_lookup: Cell::new(None),
        }
    }

    /// Перестраивает три snapshot-а в точном порядке исходного `StatBillboard`.
    pub(crate) fn stat_billboard(&mut self) -> Result<(), FactionBillboardStatBlock> {
        self.stat_member_count_billboard()?;
        self.stat_offense_victories_billboard()?;
        self.stat_defence_victories_billboard()
    }

    /// Сериализует выбранный snapshot с исходной нумерацией dispatcher-а.
    ///
    /// Значения `1/2/3` означают members/offense/defence. Любое другое
    /// значение добавляет только нулевой 32-битный count; это сохраняет
    /// наблюдаемый off-by-one относительно запроса `0x60125`, принимающего
    /// только `0/1/2`.
    pub(crate) fn add_faction_billboard_to_byte_array(
        &self,
        output: &mut Vec<u8>,
        billboard_type: i32,
    ) {
        match billboard_type {
            1 => self.add_billboard_to_byte_array(output, &self.member_count_billboard),
            2 => self.add_billboard_to_byte_array(output, &self.offense_victories_billboard),
            3 => self.add_billboard_to_byte_array(output, &self.defence_victories_billboard),
            _ => append_i32(output, 0),
        }
    }

    /// Выполняет nullable faction lookup и уже восстановленный `CFaction::Upgrade`.
    pub(crate) fn upgrade_faction<Context>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        faction_id: i32,
        player_id: i32,
        context: &mut Context,
    ) -> Result<Option<FactionUpgradeOutcome>, FactionUpgradeBlock>
    where
        Context: FactionUpgradeContext,
    {
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(None);
        };
        faction
            .upgrade(game, parameters, player_id, context)
            .map(Some)
    }

    /// Повторно разрешает faction перед уже восстановленным `CFaction::UploadIcon`.
    pub(crate) fn upload_faction_icon<Context>(
        &mut self,
        parameters: &COrganizingParam,
        faction_id: i32,
        player_id: i32,
        time: &TagTimeValue,
        context: &mut Context,
    ) -> Result<Option<FactionUploadIconOutcome>, FactionUploadIconBlock>
    where
        Context: FactionUploadIconContext,
    {
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(None);
        };
        faction
            .upload_icon(parameters, player_id, time, context)
            .map(Some)
    }

    /// Разрешает faction requester-а через `IsFreePlayer` и меняет contributor.
    pub(crate) fn set_contributor_for_player<Context>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        requester_id: i32,
        target_id: i32,
        enabled: bool,
        context: &mut Context,
    ) -> Result<OrganizingContributorOutcome, OrganizingContributorBlock>
    where
        Context: FactionContributorContext,
    {
        let faction_id = match self.is_free_player(requester_id) {
            FreePlayerLookup::NoFaction => {
                return Ok(OrganizingContributorOutcome::FactionNotFound);
            }
            FreePlayerLookup::Faction(faction_id) => faction_id,
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return Err(OrganizingContributorBlock::Membership { map_key });
            }
        };
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(OrganizingContributorOutcome::FactionNotFound);
        };
        let outcome = faction
            .set_contributor(game, parameters, requester_id, target_id, enabled, context)
            .map_err(OrganizingContributorBlock::Contributor)?;
        Ok(OrganizingContributorOutcome::Applied {
            faction_id,
            outcome,
        })
    }

    /// Применяет wrapping experience delta после exact contributor gate.
    pub(crate) fn add_contributor_experience(
        &mut self,
        game: &CGame,
        faction_id: i32,
        player_id: i32,
        experience_delta: i32,
    ) -> Result<OrganizingFactionExperienceMutation, FactionExperienceBlock> {
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(OrganizingFactionExperienceMutation::FactionNotFound);
        };
        if !faction.is_contribute(player_id) {
            return Ok(OrganizingFactionExperienceMutation::PlayerNotContributor);
        }
        let before_experience = faction
            .experience()
            .ok_or(FactionExperienceBlock::MissingBaseProperty)?;
        let current_experience = faction
            .experience()
            .ok_or(FactionExperienceBlock::MissingBaseProperty)?;
        let update = faction.set_experience(
            game,
            current_experience.wrapping_add(experience_delta),
        )?;
        Ok(OrganizingFactionExperienceMutation::Applied {
            faction_id: faction.faction_id(),
            faction_name: faction.name().to_vec(),
            before_experience,
            update,
        })
    }

    /// Разрешает faction до чтения operation-specific поля и вызывает
    /// соответствующий соседний virtual owner `OnMember*Change`.
    pub(crate) fn change_faction_member_state<ReadValue>(
        &mut self,
        game: &CGame,
        faction_id: i32,
        player_id: i32,
        operation: i32,
        mut read_value: ReadValue,
    ) -> OrganizingFactionMemberStateOutcome
    where
        ReadValue: FnMut() -> i32,
    {
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return OrganizingFactionMemberStateOutcome::FactionNotFound;
        };
        match operation {
            1 => OrganizingFactionMemberStateOutcome::Level(
                faction.on_member_level_change(game, player_id, read_value()),
            ),
            2 => OrganizingFactionMemberStateOutcome::Position(
                faction.on_member_position_change(game, player_id, read_value()),
            ),
            _ => OrganizingFactionMemberStateOutcome::UnknownOperation,
        }
    }

    /// Выполняет конкретный `CFaction::OperatorTax` после внешних war-gates.
    pub(crate) fn operate_faction_tax(
        &self,
        faction_id: i32,
        player_id: i32,
        region_id: i32,
    ) -> Result<
        Option<FactionOperationOutcome>,
        FactionOperationBlock<FactionUnionMembershipLookupBlock>,
    > {
        let Some(faction) = self.faction_by_id(faction_id) else {
            return Ok(None);
        };
        faction
            .operator_tax(player_id, region_id, self)
            .map(Some)
    }

    /// Выполняет конкретный `CFaction::OperatorCityGate` после faction lookup.
    pub(crate) fn operate_faction_city_gate(
        &self,
        faction_id: i32,
        player_id: i32,
        region_id: i32,
    ) -> Result<
        Option<FactionOperationOutcome>,
        FactionOperationBlock<FactionUnionMembershipLookupBlock>,
    > {
        let Some(faction) = self.faction_by_id(faction_id) else {
            return Ok(None);
        };
        faction
            .operator_city_gate(player_id, region_id, self)
            .map(Some)
    }

    /// Вызывает concrete `SetIsPermit` с точным nullable lookup текущего union.
    pub(crate) fn set_faction_admission_permit(
        &mut self,
        game: &CGame,
        faction_id: i32,
        player_id: i32,
        permit: bool,
    ) -> Result<Option<FactionPermitUpdate>, FactionPermitBlock> {
        let Some(mut faction) = self
            .factions
            .get_mut(&faction_id)
            .and_then(Option::take)
        else {
            return Ok(None);
        };
        let outcome = faction.set_is_permitted(game, player_id, permit, |union_id| {
            self.confederation_by_id(union_id).map(CUnion::master_id)
        });
        *self
            .factions
            .get_mut(&faction_id)
            .expect("временный faction slot не удаляется") = Some(faction);
        outcome.map(Some)
    }

    fn add_billboard_to_byte_array(
        &self,
        output: &mut Vec<u8>,
        billboard: &[FactionBillboardEntry],
    ) {
        let stored_size = u32::from_le_bytes(self.faction_billboard_size.to_le_bytes());
        let list_size = u32::try_from(billboard.len()).unwrap_or(u32::MAX);
        let count = list_size.min(stored_size);
        output.extend_from_slice(&count.to_le_bytes());
        for entry in billboard.iter().take(count as usize) {
            append_i32(output, entry.faction_id);
            output.extend_from_slice(legacy_c_string_prefix(&entry.name));
            output.push(0);
            append_i32(output, entry.number);
        }
    }

    fn stat_member_count_billboard(&mut self) -> Result<(), FactionBillboardStatBlock> {
        self.member_count_billboard.clear();
        let mut rows = Vec::new();
        for (&map_key, faction) in &self.factions {
            let Some(faction) = faction.as_deref() else {
                continue;
            };
            let established_time = faction.established_time().ok_or(
                FactionBillboardStatBlock::MissingEstablishedTime {
                    map_key,
                    billboard: FactionBillboardKind::MemberCount,
                },
            )?;
            rows.push((
                faction.get_member_num(),
                established_time,
                FactionBillboardEntry {
                    faction_id: faction.faction_id(),
                    name: faction.name().to_vec(),
                    number: faction.get_member_num(),
                },
            ));
        }
        finish_billboard_rows(&mut rows, BillboardTimeOrder::Early);
        self.member_count_billboard = rows.into_iter().map(|(_, _, entry)| entry).collect();
        Ok(())
    }

    fn stat_offense_victories_billboard(&mut self) -> Result<(), FactionBillboardStatBlock> {
        self.offense_victories_billboard.clear();
        let mut rows = Vec::new();
        for (&map_key, faction) in &self.factions {
            let Some(faction) = faction.as_deref() else {
                continue;
            };
            let number = faction.offense_victor_counts().ok_or(
                FactionBillboardStatBlock::MissingBaseProperty {
                    map_key,
                    billboard: FactionBillboardKind::OffenseVictories,
                },
            )?;
            let established_time = faction.established_time().ok_or(
                FactionBillboardStatBlock::MissingEstablishedTime {
                    map_key,
                    billboard: FactionBillboardKind::OffenseVictories,
                },
            )?;
            rows.push((
                number,
                established_time,
                FactionBillboardEntry {
                    faction_id: faction.faction_id(),
                    name: faction.name().to_vec(),
                    number,
                },
            ));
        }
        finish_billboard_rows(&mut rows, BillboardTimeOrder::Late);
        self.offense_victories_billboard =
            rows.into_iter().map(|(_, _, entry)| entry).collect();
        Ok(())
    }

    fn stat_defence_victories_billboard(&mut self) -> Result<(), FactionBillboardStatBlock> {
        self.defence_victories_billboard.clear();
        let mut rows = Vec::new();
        for (&map_key, faction) in &self.factions {
            let Some(faction) = faction.as_deref() else {
                continue;
            };
            let number = faction.defence_victor_counts().ok_or(
                FactionBillboardStatBlock::MissingBaseProperty {
                    map_key,
                    billboard: FactionBillboardKind::DefenceVictories,
                },
            )?;
            let established_time = faction.established_time().ok_or(
                FactionBillboardStatBlock::MissingEstablishedTime {
                    map_key,
                    billboard: FactionBillboardKind::DefenceVictories,
                },
            )?;
            rows.push((
                number,
                established_time,
                FactionBillboardEntry {
                    faction_id: faction.faction_id(),
                    name: faction.name().to_vec(),
                    number,
                },
            ));
        }
        finish_billboard_rows(&mut rows, BillboardTimeOrder::Late);
        self.defence_victories_billboard =
            rows.into_iter().map(|(_, _, entry)| entry).collect();
        Ok(())
    }

    /// Материализует organizing save/delete очереди в `CGame::tagDBData`.
    ///
    /// Оба map обходятся в signed key-order. После успешного clone live-mask
    /// сбрасывается, а delete-list очищается только после полного переноса.
    pub(crate) fn generate_save_data(
        &mut self,
        game: &CGame,
        force_all: bool,
    ) -> Result<OrganizingSaveDataReport, OrganizingSaveDataBlock> {
        let mut saved_factions = 0;
        for (&map_key, faction) in &mut self.factions {
            let Some(faction) = faction.as_deref_mut() else {
                // BLOCKED_MISSING_FACT: RVA 0x00034A10 вызывает virtual
                // CloneSaveData через null map-value. Продолжение не доказано.
                return Err(OrganizingSaveDataBlock::NullFaction { map_key });
            };
            if force_all {
                for change_data_type in [1, 2, 4, 8] {
                    faction.set_change_data(change_data_type);
                }
            }
            let save_copy = faction
                .clone_save_data()
                .map_err(|reason| OrganizingSaveDataBlock::FactionClone { map_key, reason })?;
            if let Some(save_copy) = save_copy {
                game.append_save_faction(Box::new(save_copy));
                faction.set_change_data(0);
                saved_factions += 1;
            }
        }

        let mut saved_unions = 0;
        for (&map_key, union) in &mut self.confederations {
            let Some(union) = union.as_deref_mut() else {
                return Err(OrganizingSaveDataBlock::NullConfederation { map_key });
            };
            if force_all {
                for change_data_type in [1, 2, 4, 8] {
                    union.set_change_data(change_data_type);
                }
            }
            if let Some(save_copy) = union.clone_save_data() {
                game.append_save_union(Box::new(save_copy));
                union.set_change_data(0);
                saved_unions += 1;
            }
        }

        let deleted_factions = self.delete_factions.len();
        for &faction_id in &self.delete_factions {
            game.append_delete_faction(faction_id);
        }
        self.delete_factions.clear();

        let deleted_unions = self.delete_unions.len();
        for &union_id in &self.delete_unions {
            game.append_delete_union(union_id);
        }
        self.delete_unions.clear();

        Ok(OrganizingSaveDataReport {
            saved_factions,
            saved_unions,
            deleted_factions,
            deleted_unions,
        })
    }

    /// Возвращает ту же canonical faction, которую искал `GetpFactionById`.
    ///
    /// Отсутствующий key и сохранённый null pointer оба дают исходный
    /// `nullptr`; caller сам сохраняет последующую pointer-семантику.
    pub(crate) fn faction_by_id(&self, faction_id: i32) -> Option<&CFaction> {
        self.factions.get(&faction_id).and_then(Option::as_deref)
    }

    /// Mutable-вариант того же nullable faction lookup для virtual dispatch.
    fn faction_by_id_mut(&mut self, faction_id: i32) -> Option<&mut CFaction> {
        self.factions
            .get_mut(&faction_id)
            .and_then(Option::as_deref_mut)
    }

    /// Повторяет nullable `GetConfederationOrganizing` для положительного ID.
    pub(crate) fn confederation_by_id(&self, union_id: i32) -> Option<&CUnion> {
        if union_id < 1 {
            return None;
        }
        self.confederations
            .get(&union_id)
            .and_then(Option::as_deref)
    }

    /// Повторяет ordered `IsFactionMaster` и не продолжает после старого UB.
    pub(crate) fn faction_id_by_master_player(
        &self,
        player_id: i32,
    ) -> Result<i32, FactionMasterLookupBlock> {
        for (&map_key, faction) in &self.factions {
            let Some(faction) = faction.as_deref() else {
                return Err(FactionMasterLookupBlock::NullFaction { map_key });
            };
            let master_id = faction
                .master_id()
                .ok_or(FactionMasterLookupBlock::MissingMasterId { map_key })?;
            let faction_id = if master_id == player_id {
                faction.faction_id()
            } else {
                0
            };
            if faction_id > 0 {
                return Ok(faction_id);
            }
        }
        Ok(0)
    }

    /// Разрешает master-faction и включает её leave-word feature.
    pub(crate) fn enable_leave_word_for_master<Context>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> Result<OrganizingLeaveWordEnableOutcome, OrganizingLeaveWordEnableBlock>
    where
        Context: FactionOrganizingInfoContext,
    {
        let faction_id = self
            .faction_id_by_master_player(player_id)
            .map_err(OrganizingLeaveWordEnableBlock::MasterLookup)?;
        if faction_id < 1 {
            return Ok(OrganizingLeaveWordEnableOutcome::FactionNotFound);
        }
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(OrganizingLeaveWordEnableOutcome::FactionNotFound);
        };
        let update = faction
            .set_leave_word_function(true, context)
            .map_err(|source| OrganizingLeaveWordEnableBlock::Property { faction_id, source })?;
        Ok(OrganizingLeaveWordEnableOutcome::Updated { faction_id, update })
    }

    /// Разрешает faction автора и выполняет concrete `CFaction::LeaveWord`.
    pub(crate) fn leave_word_for_player(
        &mut self,
        game: &mut CGame,
        player_id: i32,
        content: &mut Vec<u8>,
        time: TagTimeValue,
    ) -> Result<OrganizingLeaveWordOutcome, OrganizingLeaveWordBlock> {
        let faction_id = match self.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => {
                return Ok(OrganizingLeaveWordOutcome::FactionNotFound);
            }
            FreePlayerLookup::Faction(faction_id) => faction_id,
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return Err(OrganizingLeaveWordBlock::Membership { map_key });
            }
        };
        if faction_id < 1 {
            return Ok(OrganizingLeaveWordOutcome::FactionNotFound);
        }
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(OrganizingLeaveWordOutcome::FactionNotFound);
        };
        faction
            .leave_word(game, player_id, content, time)
            .map(|outcome| OrganizingLeaveWordOutcome::Applied {
                faction_id,
                outcome,
            })
            .map_err(|source| OrganizingLeaveWordBlock::Faction { faction_id, source })
    }

    /// Разрешает faction редактора и удаляет выбранный leave-word.
    pub(crate) fn edit_leave_word_for_player(
        &mut self,
        game: &CGame,
        player_id: i32,
        leave_word_id: i32,
        operator: EOperator,
    ) -> Result<OrganizingLeaveWordEditOutcome, OrganizingLeaveWordEditBlock> {
        let faction_id = match self.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => {
                return Ok(OrganizingLeaveWordEditOutcome::FactionNotFound);
            }
            FreePlayerLookup::Faction(faction_id) => faction_id,
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return Err(OrganizingLeaveWordEditBlock::Membership { map_key });
            }
        };
        if faction_id < 1 {
            return Ok(OrganizingLeaveWordEditOutcome::FactionNotFound);
        }
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(OrganizingLeaveWordEditOutcome::FactionNotFound);
        };
        faction
            .edit_leave_word(game, player_id, leave_word_id, operator)
            .map(|outcome| OrganizingLeaveWordEditOutcome::Applied {
                faction_id,
                outcome,
            })
            .map_err(|source| OrganizingLeaveWordEditBlock::Property { faction_id, source })
    }

    /// Разрешает faction автора и заменяет её текущее объявление.
    pub(crate) fn pronounce_for_player(
        &mut self,
        game: &CGame,
        player_id: i32,
        content: &mut Vec<u8>,
        time: TagTimeValue,
    ) -> Result<OrganizingPronounceOutcome, OrganizingPronounceBlock> {
        let faction_id = match self.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => {
                return Ok(OrganizingPronounceOutcome::FactionNotFound);
            }
            FreePlayerLookup::Faction(faction_id) => faction_id,
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return Err(OrganizingPronounceBlock::Membership { map_key });
            }
        };
        if faction_id < 1 {
            return Ok(OrganizingPronounceOutcome::FactionNotFound);
        }
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(OrganizingPronounceOutcome::FactionNotFound);
        };
        faction
            .pronounce(game, player_id, content, time)
            .map(|outcome| OrganizingPronounceOutcome::Applied {
                faction_id,
                outcome,
            })
            .map_err(|source| OrganizingPronounceBlock::Faction { faction_id, source })
    }

    /// Возвращает bit-exact Windows `long` для исходного `m_FacOrg.size()`.
    pub(crate) fn declare_war_faction_count(
        &self,
    ) -> Result<i32, DeclareWarFactionPageBlock> {
        i32::try_from(self.factions.len()).map_err(|_| {
            DeclareWarFactionPageBlock::FactionCountOutsideLegacyRange {
                count: self.factions.len(),
            }
        })
    }

    /// Строит payload одной 11-элементной страницы целей объявления войны.
    ///
    /// Exact `AddDeclareWarFactionInfoToByteArray` сначала вызывал
    /// `GetCountry` у source faction и отбрасывал результат. Этот внутренний
    /// бессмысленный вызов не переносится: он не менял wire или state. Signed
    /// page arithmetic остаётся wrapping, а remaining сравнивается unsigned —
    /// это сохраняет исходный wire-quirk переполненного положительного page.
    /// Старый `strcpy` через 256-byte stack buffer заменён прямой C-string
    /// сериализацией: для допустимых имён wire тот же, memory overflow удалён.
    pub(crate) fn declare_war_faction_page(
        &self,
        source_faction_id: i32,
        requested_page: i32,
        faction_wars: &CFactionWarSys,
    ) -> Result<DeclareWarFactionPage, DeclareWarFactionPageBlock> {
        if self.faction_by_id(source_faction_id).is_none() {
            return Err(DeclareWarFactionPageBlock::MissingSourceFaction {
                faction_id: source_faction_id,
            });
        }

        let total_factions = self.declare_war_faction_count()?;
        let normalized_page = requested_page.max(1);
        let start = normalized_page
            .wrapping_mul(DECLARE_WAR_FACTION_PAGE_SIZE)
            .wrapping_sub(DECLARE_WAR_FACTION_PAGE_SIZE);
        let remaining = total_factions.wrapping_sub(start);
        let entry_count = if (remaining as u32) <= DECLARE_WAR_FACTION_PAGE_SIZE as u32 {
            remaining
        } else {
            DECLARE_WAR_FACTION_PAGE_SIZE
        };
        let end = start.wrapping_add(entry_count);

        let mut entries = Vec::with_capacity(entry_count as usize);
        let mut payload = Vec::new();
        append_i32(&mut payload, entry_count);
        for (index, (&map_key, faction)) in self.factions.iter().enumerate() {
            let index = index as i32;
            if index < start {
                continue;
            }
            if index >= end {
                break;
            }
            let Some(faction) = faction.as_deref() else {
                return Err(DeclareWarFactionPageBlock::NullFaction { map_key });
            };
            let faction_id = faction.faction_id();
            let country = faction
                .country()
                .ok_or(DeclareWarFactionPageBlock::MissingCountry { faction_id })?;
            let name = legacy_c_string_prefix(faction.name()).to_vec();
            let relation = if faction_id == source_faction_id {
                DeclareWarFactionRelation::SelfFaction
            } else {
                let target_union = declare_war_union_id(self, faction_id)?;
                let source_union = declare_war_union_id(self, source_faction_id)?;
                if target_union.is_some() && target_union == source_union {
                    DeclareWarFactionRelation::SameUnion
                } else if faction_wars.is_enemy_relation(faction_id, source_faction_id) {
                    DeclareWarFactionRelation::Enemy
                } else {
                    DeclareWarFactionRelation::None
                }
            };

            append_i32(&mut payload, faction_id);
            payload.push(country);
            payload.extend_from_slice(&name);
            payload.push(0);
            append_i32(&mut payload, relation as i32);
            entries.push(DeclareWarFactionEntry {
                faction_id,
                country,
                name,
                relation,
            });
        }

        Ok(DeclareWarFactionPage {
            requested_page,
            normalized_page,
            total_factions,
            entries,
            payload,
        })
    }

    /// Повторяет `GetUnion`: master-player -> faction -> union -> nullable owner.
    pub(crate) fn union_id_by_master_player(
        &self,
        player_id: i32,
    ) -> Result<Option<i32>, OrganizingUnionByMasterBlock> {
        let faction_id = self
            .faction_id_by_master_player(player_id)
            .map_err(OrganizingUnionByMasterBlock::FactionMaster)?;
        if faction_id < 1 {
            return Ok(None);
        }

        let union_id = match self.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => return Ok(None),
            FreeFactionLookup::Union(union_id) => union_id,
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                return Err(OrganizingUnionByMasterBlock::UnionMembership(
                    FactionUnionMembershipLookupBlock { map_key },
                ));
            }
        };
        Ok(self.confederation_by_id(union_id).map(CUnion::union_id))
    }

    /// Выполняет exact virtual-call ветки `OnOrgasysMessage(0x60118)`.
    pub(crate) fn apply_for_union_join<Effects>(
        &mut self,
        game: &CGame,
        master_player_id: i32,
        applicant_faction_id: i32,
        second_parameter: i32,
        third_parameter: i32,
        effects: &mut Effects,
    ) -> Result<
        OrganizingUnionApplyForJoinOutcome<Effects::SessionReport>,
        OrganizingUnionApplyForJoinDispatchBlock<Effects::SessionBlock>,
    >
    where
        Effects: UnionApplyForJoinEffects,
    {
        let Some(union_id) = self
            .union_id_by_master_player(master_player_id)
            .map_err(OrganizingUnionApplyForJoinDispatchBlock::Lookup)?
        else {
            return Ok(OrganizingUnionApplyForJoinOutcome::UnionNotFound);
        };

        // `ApplyForJoin` повторно сканирует union-map по applicant. Временный
        // safe take выбранного owner-а не должен менять результат этого scan.
        let applicant_membership = self.is_free_faction(applicant_faction_id);
        let mut union = self
            .confederations
            .get_mut(&union_id)
            .and_then(Option::take)
            .expect("GetUnion вернул живой owner из того же controller map");
        self.detached_union_membership_lookup
            .set(Some((applicant_faction_id, applicant_membership)));
        let result = union.apply_for_join(
            game,
            applicant_faction_id,
            second_parameter,
            third_parameter,
            self,
            effects,
        );
        self.detached_union_membership_lookup.set(None);
        *self
            .confederations
            .get_mut(&union_id)
            .expect("временный union slot не удаляется") = Some(union);

        result
            .map(|outcome| OrganizingUnionApplyForJoinOutcome::Applied { union_id, outcome })
            .map_err(|source| OrganizingUnionApplyForJoinDispatchBlock::Apply {
                union_id,
                source,
            })
    }

    /// Строит и маршрутизирует точный player-targeted organizing-info wire.
    pub(crate) fn send_organizing_info_to_client(
        game: &CGame,
        request: FactionMemberInfoRequest<'_>,
    ) -> OrganizingInfoDelivery {
        let game_server_id = if request.information_type == -1 {
            game.game_server_number_by_player_id(request.recipient_player_id)
        } else {
            request.information_type
        };
        if game_server_id == -1 {
            return OrganizingInfoDelivery::RouteRejected {
                recipient_player_id: request.recipient_player_id,
            };
        }

        let mut message = CMessage::new(0x0007_F804);
        message.base_mut().add_long(request.recipient_player_id);
        message
            .base_mut()
            .add(legacy_c_string_prefix(request.first_text));
        message.base_mut().add_byte(0);
        message.base_mut().add_ulong(request.color);
        message.base_mut().add_ulong(request.trailing_value);
        message
            .base_mut()
            .add(legacy_c_string_prefix(request.second_text));
        message.base_mut().add_byte(0);
        OrganizingInfoDelivery::Sent {
            recipient_player_id: request.recipient_player_id,
            game_server_id,
            result: game.send_msg_to_game_server(game_server_id, &message),
        }
    }

    /// Строит exact broadcast-overload `0x7FA03` и вызывает `SendAll`.
    pub(crate) fn send_organizing_info_to_all(
        game: &CGame,
        info: &[u8],
        kind: u32,
        color: u32,
    ) -> Result<i32, SendMessageError> {
        let mut message = CMessage::new(0x0007_FA03);
        message.base_mut().add_long(0);
        message.base_mut().add_long(0);
        message.base_mut().add_ulong(kind);
        message.base_mut().add_ulong(color);
        message.base_mut().add(legacy_c_string_prefix(info));
        message.base_mut().add_byte(0);
        let sender = game.current_game_server_sender();
        message.send_all(sender.as_ref())
    }

    /// Публикует одно other-faction изменение всем concrete faction-owner-ам.
    pub(crate) fn update_other_faction_info_to_client(
        &self,
        game: &CGame,
        faction_id: i32,
        faction_name: &[u8],
        operator: EOperator,
    ) -> Result<Vec<OrganizingOtherFactionUpdate>, OrganizingOtherFactionUpdateBlock> {
        let mut completed = Vec::new();
        for (&map_key, faction) in &self.factions {
            let Some(faction) = faction.as_deref() else {
                continue;
            };
            let deliveries = match faction.update_other_faction_info_to_client(
                game,
                faction_id,
                faction_name,
                operator,
            ) {
                Ok(deliveries) => deliveries,
                Err(source) => {
                    return Err(OrganizingOtherFactionUpdateBlock {
                        map_key,
                        source,
                        completed,
                    });
                }
            };
            completed.push(OrganizingOtherFactionUpdate {
                map_key,
                deliveries,
            });
        }
        Ok(completed)
    }

    /// Выполняет оба исходных disband-слоя и удаляет concrete faction-owner.
    pub(crate) fn disband_faction<Context>(
        &mut self,
        game: &CGame,
        player_id: i32,
        faction_id: i32,
        context: &mut Context,
    ) -> Result<OrganizingDisbandOutcome, OrganizingDisbandBlock>
    where
        Context: OrganizingDisbandContext,
    {
        let Some(faction) = self.factions.get_mut(&faction_id) else {
            return Ok(OrganizingDisbandOutcome::Rejected {
                reason: OrganizingDisbandRejection::FactionEntryMissing,
                notice_sent: false,
                cleared_city_war_enemies: 0,
            });
        };
        let Some(faction) = faction.as_deref_mut() else {
            return Err(OrganizingDisbandBlock::NullFaction { faction_id });
        };

        if faction.has_enemy_faction() || context.village_war_declared(faction_id) {
            send_disband_information(context, player_id, b"WS0235");
            return Ok(OrganizingDisbandOutcome::Rejected {
                reason: OrganizingDisbandRejection::StandardWar,
                notice_sent: true,
                cleared_city_war_enemies: 0,
            });
        }
        if faction.has_city_war_enemy_faction() || context.city_war_declared(faction_id) {
            send_disband_information(context, player_id, b"WS0236");
            return Ok(OrganizingDisbandOutcome::Rejected {
                reason: OrganizingDisbandRejection::CityWar,
                notice_sent: true,
                cleared_city_war_enemies: 0,
            });
        }

        let cleared_city_war_enemies = faction.city_war_enemy_factions().len();
        faction.clear_city_war_enemy_factions();
        let faction_progress = match faction.disband(game, player_id, context) {
            Ok(FactionDisbandOutcome::Rejected {
                reason,
                notice_sent,
            }) => {
                return Ok(OrganizingDisbandOutcome::Rejected {
                    reason: OrganizingDisbandRejection::Faction(reason),
                    notice_sent,
                    cleared_city_war_enemies,
                });
            }
            Ok(FactionDisbandOutcome::Disbanded(progress)) => progress,
            Err(source) => {
                return Err(OrganizingDisbandBlock::Faction {
                    source,
                    cleared_city_war_enemies,
                });
            }
        };

        let faction = self
            .factions
            .remove(&faction_id)
            .and_then(|faction| faction)
            .expect("успешный faction Disband не меняет controller map");
        let mut progress = OrganizingDisbandProgress {
            cleared_city_war_enemies,
            faction: faction_progress,
            map_removed: true,
            second_delete_organizing: None,
            delete_faction_queued: false,
            other_faction_updates: None,
            player: None,
            log_written: false,
        };
        progress.second_delete_organizing = Some(
            match faction.delete_organizing_to_client(game, 0, context) {
                Ok(outcome) => outcome,
                Err(source) => {
                    return Err(OrganizingDisbandBlock::SecondDeleteOrganizing {
                        source,
                        progress,
                    });
                }
            },
        );

        self.delete_factions.push_back(faction.faction_id());
        progress.delete_faction_queued = true;
        progress.other_faction_updates = Some(
            match self.update_other_faction_info_to_client(game, faction_id, b"", EOperator::Delete)
            {
                Ok(updates) => updates,
                Err(source) => {
                    return Err(OrganizingDisbandBlock::OtherFactionUpdate { source, progress });
                }
            },
        );

        progress.player = context.clear_player_faction_data_received(player_id);
        if context.faction_disband_log_enabled()
            && let Some(player) = progress.player.as_ref()
        {
            context.write_faction_disband_log(
                faction_id,
                legacy_c_string_prefix(faction.name()),
                player.player_id,
                legacy_c_string_prefix(&player.player_name),
            );
            progress.log_written = true;
        }

        Ok(OrganizingDisbandOutcome::Disbanded(progress))
    }

    /// Пересчитывает и публикует property всех concrete faction-owner-ов.
    pub(crate) fn reinitialize_factions_by_level(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
    ) -> Result<Vec<FactionReinitializationEntry>, FactionReinitializationBlock> {
        let mut completed = Vec::new();
        for (&map_key, faction) in &mut self.factions {
            let Some(faction) = faction.as_deref_mut() else {
                // Exact RTTI cast null/non-CFaction pointer пропускал.
                continue;
            };
            let result = match faction.reinitialize_property_by_level(game, parameters) {
                Ok(result) => result,
                Err(source) => {
                    return Err(FactionReinitializationBlock {
                        map_key,
                        source,
                        completed,
                    });
                }
            };
            completed.push(FactionReinitializationEntry { map_key, result });
        }
        Ok(completed)
    }

    /// Отвязывает faction от union в точном порядке `CUnion::DelMember`.
    pub(crate) fn detach_union_member(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        faction_id: i32,
    ) -> Result<UnionMemberDetachOutcome, UnionMemberDetachBlock> {
        if faction_id <= 0 {
            return Ok(UnionMemberDetachOutcome::NonPositiveFactionId);
        }
        let Some(faction) = self.factions.get_mut(&faction_id) else {
            return Ok(UnionMemberDetachOutcome::FactionEntryMissing);
        };
        let Some(faction) = faction.as_deref_mut() else {
            return Ok(UnionMemberDetachOutcome::NullFactionPointer);
        };

        faction
            .set_superior_organizing(0, parameters)
            .map_err(|source| UnionMemberDetachBlock {
                faction_id,
                source: UnionMemberDetachBlockSource::SuperiorOrganizing(source),
            })?;
        let deliveries = faction.update_property_to_client(game).map_err(|source| {
            UnionMemberDetachBlock {
                faction_id,
                source: UnionMemberDetachBlockSource::Property(source),
            }
        })?;
        Ok(UnionMemberDetachOutcome::Detached { deliveries })
    }

    /// Ищет первый положительный faction ID в signed map-порядке.
    pub(crate) fn is_free_player(&self, player_id: i32) -> FreePlayerLookup {
        for (&map_key, faction) in &self.factions {
            let Some(faction) = faction.as_deref() else {
                // BLOCKED_MISSING_FACT: `IsFreePlayer` RVA `0x000343A0`
                // разыменовывает map-value без null-check. Достижимость и
                // наблюдаемая реакция null не доказаны.
                return FreePlayerLookup::BlockedNullFaction { map_key };
            };
            let faction_id = faction.is_member(player_id);
            if faction_id > 0 {
                return FreePlayerLookup::Faction(faction_id);
            }
        }
        FreePlayerLookup::NoFaction
    }

    /// Ищет первый положительный union ID в signed map-порядке.
    pub(crate) fn is_free_faction(&self, faction_id: i32) -> FreeFactionLookup {
        for (&map_key, union) in &self.confederations {
            let Some(union) = union.as_deref() else {
                // BLOCKED_MISSING_FACT: `IsFreeFaction` RVA `0x00034420`
                // разыменовывает map-value без null-check. Достижимость и
                // наблюдаемая реакция null не доказаны.
                return FreeFactionLookup::BlockedNullConfederation { map_key };
            };
            let union_id = union.is_member(faction_id);
            if union_id > 0 {
                return FreeFactionLookup::Union(union_id);
            }
        }
        FreeFactionLookup::NoUnion
    }

    /// Проверяет literal ID в исходном list-order без изменения списка.
    pub(crate) fn is_union_application_reserved(&self, faction_id: i32) -> bool {
        self.request_establishment_union_players
            .iter()
            .any(|reserved_id| *reserved_id == faction_id)
    }

    /// Добавляет ID в хвост `m_RequestEstaUnionPlayers`, не устраняя duplicate.
    pub(crate) fn push_to_establishment_list(&mut self, faction_id: i32) {
        self.request_establishment_union_players
            .push_back(faction_id);
    }

    /// Удаляет первое совпадение, как terminal union callback.
    pub(crate) fn remove_from_establishment_list(&mut self, faction_id: i32) -> bool {
        let Some(position) = self
            .request_establishment_union_players
            .iter()
            .position(|reserved_id| *reserved_id == faction_id)
        else {
            return false;
        };
        self.request_establishment_union_players.remove(position);
        true
    }

    fn city_war_owner_for_faction(
        &self,
        faction_id: i32,
    ) -> Result<(Option<CityWarOrganizingOwner>, i32), i32> {
        let faction_owner = (faction_id > 0)
            .then(|| self.faction_by_id(faction_id))
            .flatten()
            .map(|_| CityWarOrganizingOwner::Faction(faction_id));
        match self.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => Ok((faction_owner, 0)),
            FreeFactionLookup::Union(union_id) => Ok((
                self.confederation_by_id(union_id)
                    .map(|_| CityWarOrganizingOwner::Union(union_id)),
                union_id,
            )),
            FreeFactionLookup::BlockedNullConfederation { map_key } => Err(map_key),
        }
    }

    fn city_war_owner_name(&self, owner: CityWarOrganizingOwner) -> &[u8] {
        match owner {
            CityWarOrganizingOwner::Faction(faction_id) => self
                .faction_by_id(faction_id)
                .expect("resolved city-war faction owner не удаляется")
                .name(),
            CityWarOrganizingOwner::Union(union_id) => self
                .confederation_by_id(union_id)
                .expect("resolved city-war union owner не удаляется")
                .name(),
        }
    }

    fn city_war_owner_has_city(
        &self,
        owner: CityWarOrganizingOwner,
        region_id: i32,
    ) -> bool {
        match owner {
            CityWarOrganizingOwner::Faction(faction_id) => self
                .faction_by_id(faction_id)
                .expect("resolved city-war faction owner не удаляется")
                .is_owned_city(region_id)
                != 0,
            CityWarOrganizingOwner::Union(union_id) => self
                .confederation_by_id(union_id)
                .expect("resolved city-war union owner не удаляется")
                .is_owned_city(region_id, self)
                != 0,
        }
    }

    fn add_city_war_victor_counts(
        &mut self,
        game: &CGame,
        owner: CityWarOrganizingOwner,
        offence: bool,
    ) -> Result<CityWarVictorMutation, CityWarVictorMutationBlock> {
        match owner {
            CityWarOrganizingOwner::Faction(faction_id) => {
                let faction = self
                    .faction_by_id_mut(faction_id)
                    .expect("resolved city-war faction owner не удаляется");
                let outcome = if offence {
                    faction.add_offense_victor_count(game)
                } else {
                    faction.add_defence_victor_count(game)
                };
                outcome
                    .map(CityWarVictorMutation::Faction)
                    .map_err(CityWarVictorMutationBlock::Faction)
            }
            CityWarOrganizingOwner::Union(union_id) => {
                let union = self
                    .confederations
                    .get_mut(&union_id)
                    .and_then(Option::take)
                    .expect("resolved city-war union slot не удаляется");
                let outcome = if offence {
                    union.add_offense_victor_counts(self, game)
                } else {
                    union.add_defence_victor_counts(self, game)
                };
                *self
                    .confederations
                    .get_mut(&union_id)
                    .expect("временный city-war union slot не удаляется") = Some(union);
                outcome
                    .map(CityWarVictorMutation::Union)
                    .map_err(CityWarVictorMutationBlock::Union)
            }
        }
    }

    fn delete_city_war_owner_city(
        &mut self,
        game: &CGame,
        owner: CityWarOrganizingOwner,
        region_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<CityWarOwnedCityRemoval, CityWarOwnedCityMutationBlock> {
        match owner {
            CityWarOrganizingOwner::Faction(faction_id) => self
                .faction_by_id_mut(faction_id)
                .expect("resolved city-war faction owner не удаляется")
                .delete_owned_city(game, region_id, update_player)
                .map(CityWarOwnedCityRemoval::Faction)
                .map_err(CityWarOwnedCityMutationBlock::Faction),
            CityWarOrganizingOwner::Union(union_id) => {
                let union = self
                    .confederations
                    .get_mut(&union_id)
                    .and_then(Option::take)
                    .expect("resolved city-war union slot не удаляется");
                let outcome = union.delete_owned_city(self, game, region_id, update_player);
                *self
                    .confederations
                    .get_mut(&union_id)
                    .expect("временный city-war union slot не удаляется") = Some(union);
                outcome
                    .map(CityWarOwnedCityRemoval::Union)
                    .map_err(CityWarOwnedCityMutationBlock::Union)
            }
        }
    }

    fn add_city_war_owner_city(
        &mut self,
        game: &CGame,
        owner: CityWarOrganizingOwner,
        region_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<CityWarOwnedCityAddition, CityWarOwnedCityMutationBlock> {
        match owner {
            CityWarOrganizingOwner::Faction(faction_id) => self
                .faction_by_id_mut(faction_id)
                .expect("resolved city-war faction owner не удаляется")
                .add_owned_city(game, region_id, update_player)
                .map(CityWarOwnedCityAddition::Faction)
                .map_err(CityWarOwnedCityMutationBlock::Faction),
            CityWarOrganizingOwner::Union(union_id) => {
                let union = self
                    .confederations
                    .get_mut(&union_id)
                    .and_then(Option::take)
                    .expect("resolved city-war union slot не удаляется");
                let outcome = union.add_owned_city(self, game, region_id, update_player);
                *self
                    .confederations
                    .get_mut(&union_id)
                    .expect("временный city-war union slot не удаляется") = Some(union);
                outcome
                    .map(CityWarOwnedCityAddition::Union)
                    .map_err(CityWarOwnedCityMutationBlock::Union)
            }
        }
    }

    /// Применяет полный результат войны за город в exact virtual-порядке.
    #[allow(
        clippy::too_many_arguments,
        reason = "аргументы являются четырьмя wire-полями и явными process owners"
    )]
    pub(crate) fn on_attack_city_end<Effects>(
        &mut self,
        game: &CGame,
        result: i32,
        region_id: i32,
        attacker_player_id: i32,
        defender_faction_id: i32,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<AttackCityEndReport, AttackCityEndBlock>
    where
        Effects: AttackCityEndEffects,
    {
        let mut report = AttackCityEndReport::pending();
        let attacker_faction_id = match self.is_free_player(attacker_player_id) {
            FreePlayerLookup::NoFaction => None,
            FreePlayerLookup::Faction(faction_id) => {
                report.attacker_faction_id = Some(faction_id);
                Some(faction_id)
            }
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return Err(AttackCityEndBlock::AttackerMembership { report, map_key });
            }
        };
        let region_name = match game.region_name(region_id) {
            WorldRegionNameLookup::RegionNotFound | WorldRegionNameLookup::NullRegionPointer => {
                Vec::new()
            }
            WorldRegionNameLookup::Name(name) => {
                let name = legacy_c_string_prefix(name);
                if name.len() >= 0x100 {
                    return Err(AttackCityEndBlock::RegionNameWouldOverflow {
                        report,
                        visible_len: name.len(),
                    });
                }
                name.to_vec()
            }
        };
        let Some(attacker_faction_id) = attacker_faction_id else {
            return Ok(report);
        };

        let (attacker_owner, attacker_union_id) =
            match self.city_war_owner_for_faction(attacker_faction_id) {
                Ok(owner) => owner,
                Err(map_key) => {
                    return Err(AttackCityEndBlock::AttackerUnionMembership { report, map_key });
                }
            };
        report.attacker_union_id = Some(attacker_union_id);
        report.attacker_owner = attacker_owner;

        let (defender_owner, defender_union_id) =
            match self.city_war_owner_for_faction(defender_faction_id) {
                Ok(owner) => owner,
                Err(map_key) => {
                    return Err(AttackCityEndBlock::DefenderUnionMembership { report, map_key });
                }
            };
        report.defender_union_id = Some(defender_union_id);
        report.defender_owner = defender_owner;
        if let Some(defender_owner) = defender_owner {
            let owns_city = self.city_war_owner_has_city(defender_owner, region_id);
            report.defender_owns_city = Some(owns_city);
            if !owns_city {
                return Ok(report);
            }
        }

        if result == 1 {
            let Some(attacker_owner) = attacker_owner else {
                return Ok(report);
            };
            if let Some(defender_owner) = defender_owner {
                let offense = match self.add_city_war_victor_counts(game, attacker_owner, true) {
                    Ok(outcome) => outcome,
                    Err(source) => {
                        return Err(AttackCityEndBlock::OffenseVictors { report, source });
                    }
                };
                report.offense_victors = Some(offense);
                let removal = match self.delete_city_war_owner_city(
                    game,
                    defender_owner,
                    region_id,
                    update_player,
                ) {
                    Ok(outcome) => outcome,
                    Err(source) => {
                        return Err(AttackCityEndBlock::DefenderCityRemoval { report, source });
                    }
                };
                report.defender_city_removal = Some(removal);

                let attacker_name = self.city_war_owner_name(attacker_owner).to_vec();
                let text = effects.format_world_string(
                    b"WS0261",
                    &[
                        UnionFormatArgument::Text(legacy_c_string_prefix(&attacker_name)),
                        UnionFormatArgument::Text(&region_name),
                    ],
                );
                let text = legacy_c_string_prefix(&text);
                if text.len() >= 0x400 {
                    return Err(AttackCityEndBlock::NoticeWouldOverflow {
                        report,
                        string_id: b"WS0261",
                        formatted_len: text.len(),
                    });
                }
                report.broadcast = Some(effects.broadcast_city_war_result(text));
            }

            let addition = match self.add_city_war_owner_city(
                game,
                attacker_owner,
                region_id,
                update_player,
            ) {
                Ok(outcome) => outcome,
                Err(source) => {
                    return Err(AttackCityEndBlock::AttackerCityAddition { report, source });
                }
            };
            report.attacker_city_addition = Some(addition);
            effects.refresh_owned_city(region_id, attacker_faction_id, attacker_union_id);
            report.refreshed_owner = Some((region_id, attacker_faction_id, attacker_union_id));
        } else if result == 0 {
            let Some(defender_owner) = defender_owner else {
                return Ok(report);
            };
            let defence = match self.add_city_war_victor_counts(game, defender_owner, false) {
                Ok(outcome) => outcome,
                Err(source) => {
                    return Err(AttackCityEndBlock::DefenceVictors { report, source });
                }
            };
            report.defence_victors = Some(defence);
            let Some(attacker_owner) = attacker_owner else {
                return Err(AttackCityEndBlock::MissingAttackerOwnerForDefenceNotice { report });
            };
            let attacker_name = self.city_war_owner_name(attacker_owner).to_vec();
            let text = effects.format_world_string(
                b"WS0262",
                &[
                    UnionFormatArgument::Text(legacy_c_string_prefix(&attacker_name)),
                    UnionFormatArgument::Text(&region_name),
                ],
            );
            let text = legacy_c_string_prefix(&text);
            if text.len() >= 0x400 {
                return Err(AttackCityEndBlock::NoticeWouldOverflow {
                    report,
                    string_id: b"WS0262",
                    formatted_len: text.len(),
                });
            }
            report.broadcast = Some(effects.broadcast_city_war_result(text));
        }
        Ok(report)
    }

    /// Выполняет preflight и запускает `TransferIOwnerCity` в машинном порядке.
    #[allow(
        clippy::too_many_arguments,
        reason = "явные Game/country/war/session owners заменяют process singletons"
    )]
    pub(crate) fn transfer_city_owner<Effects>(
        &mut self,
        game: &CGame,
        countries: &CCountryHandler,
        attack_city: &CAttackCitySys,
        village_war: &CVillageWarSys,
        requester_player_id: i32,
        target_faction_id: i32,
        region_id: i32,
        effects: &mut Effects,
    ) -> Result<
        CityTransferStartOutcome<Effects::SessionReport>,
        CityTransferStartBlock<Effects::SessionBlock>,
    >
    where
        Effects: CityTransferEffects,
    {
        let region_name = match game.region_name(region_id) {
            WorldRegionNameLookup::RegionNotFound => {
                return Ok(CityTransferStartOutcome::Rejected(
                    CityTransferRejection::RegionNotFound,
                ));
            }
            WorldRegionNameLookup::NullRegionPointer => {
                return Ok(CityTransferStartOutcome::Rejected(
                    CityTransferRejection::NullRegionPointer,
                ));
            }
            WorldRegionNameLookup::Name(name) => name,
        };
        if region_id < 1 {
            return Ok(CityTransferStartOutcome::Rejected(
                CityTransferRejection::NonPositiveRegion,
            ));
        }
        if village_war.get_region_state(region_id) != ECityState::No
            || attack_city.get_city_state(region_id) != ECityState::No
        {
            send_city_transfer_notice(effects, requester_player_id, b"WS0248");
            return Ok(CityTransferStartOutcome::Rejected(
                CityTransferRejection::CityWarActive { notice_sent: true },
            ));
        }

        let source_faction_id = self
            .faction_id_by_master_player(requester_player_id)
            .map_err(CityTransferStartBlock::FactionMaster)?;
        if source_faction_id == 0 {
            return Ok(CityTransferStartOutcome::Rejected(
                CityTransferRejection::MasterFactionNotFound,
            ));
        }
        if target_faction_id == 0 {
            return Ok(CityTransferStartOutcome::Rejected(
                CityTransferRejection::ZeroTargetFactionId,
            ));
        }

        let (source_country, source_owns_region) = {
            let Some(source) = self.faction_by_id(source_faction_id) else {
                return Ok(CityTransferStartOutcome::Rejected(
                    CityTransferRejection::SourceFactionMissing,
                ));
            };
            let country = source.country().ok_or(
                CityTransferStartBlock::MissingSourceCountry {
                    faction_id: source_faction_id,
                },
            )?;
            (country, source.is_owned_city(region_id) != 0)
        };
        let (target_country, target_has_city) = {
            let Some(target) = self.faction_by_id(target_faction_id) else {
                return Ok(CityTransferStartOutcome::Rejected(
                    CityTransferRejection::TargetFactionMissing,
                ));
            };
            let country = target.country().ok_or(
                CityTransferStartBlock::MissingTargetCountry {
                    faction_id: target_faction_id,
                },
            )?;
            (country, !target.owned_cities().is_empty())
        };

        if source_country != target_country {
            send_city_transfer_notice(effects, requester_player_id, b"WS0249");
            return Ok(CityTransferStartOutcome::Rejected(
                CityTransferRejection::CountriesDiffer { notice_sent: true },
            ));
        }
        if let Some(country) = countries.get_country(source_country) {
            let source_master_id = self
                .faction_by_id(source_faction_id)
                .expect("source faction owner проверен")
                .master_id()
                .ok_or(CityTransferStartBlock::MissingSourceMaster {
                    faction_id: source_faction_id,
                })?;
            if country.king.id == source_master_id {
                send_city_transfer_notice(effects, requester_player_id, b"WS0250");
                return Ok(CityTransferStartOutcome::Rejected(
                    CityTransferRejection::SourceMasterIsCountryKing { notice_sent: true },
                ));
            }
        }
        if !source_owns_region {
            return Ok(CityTransferStartOutcome::Rejected(
                CityTransferRejection::SourceDoesNotOwnRegion,
            ));
        }
        if target_has_city {
            send_city_transfer_notice(effects, requester_player_id, b"WS0251");
            return Ok(CityTransferStartOutcome::Rejected(
                CityTransferRejection::TargetAlreadyOwnsCity { notice_sent: true },
            ));
        }
        if attack_city.is_already_declared_for_war(target_faction_id) {
            send_city_transfer_notice(effects, requester_player_id, b"WS0252");
            return Ok(CityTransferStartOutcome::Rejected(
                CityTransferRejection::TargetDeclaredAttackWar { notice_sent: true },
            ));
        }
        if village_war.is_already_declared_for_war(target_faction_id) {
            send_city_transfer_notice(effects, requester_player_id, b"WS0253");
            return Ok(CityTransferStartOutcome::Rejected(
                CityTransferRejection::TargetDeclaredVillageWar { notice_sent: true },
            ));
        }
        if self.is_union_application_reserved(source_faction_id) {
            send_city_transfer_notice(effects, requester_player_id, b"WS0254");
            return Ok(CityTransferStartOutcome::Rejected(
                CityTransferRejection::SourceFactionReserved { notice_sent: true },
            ));
        }
        if self.is_union_application_reserved(target_faction_id) {
            send_city_transfer_notice(effects, requester_player_id, b"WS0255");
            return Ok(CityTransferStartOutcome::Rejected(
                CityTransferRejection::TargetFactionReserved { notice_sent: true },
            ));
        }

        let target_master_player_id = self
            .faction_by_id(target_faction_id)
            .expect("target faction owner проверен")
            .master_id()
            .ok_or(CityTransferStartBlock::MissingTargetMaster {
                faction_id: target_faction_id,
            })?;
        let Some(target_master) = game.online_player_by_id(target_master_player_id as u32) else {
            send_city_transfer_notice(effects, requester_player_id, b"WS0256");
            return Ok(CityTransferStartOutcome::Rejected(
                CityTransferRejection::TargetMasterOffline { notice_sent: true },
            ));
        };
        self.push_to_establishment_list(source_faction_id);
        self.push_to_establishment_list(target_faction_id);
        let net_exchange_id = target_master.get_net_exchange_id();
        let source_name = self
            .faction_by_id(source_faction_id)
            .expect("source faction owner проверен")
            .name()
            .to_vec();
        let session = effects
            .begin_city_transfer_session(CityTransferSessionRequest {
                requester_player_id,
                source_faction_id,
                target_master_player_id,
                target_faction_id,
                region_id,
                requested_session_id: net_exchange_id,
                timeout_ticks: 0x2710,
                source_faction_name: source_name,
                region_name: region_name.to_vec(),
            })
            .map_err(|source| CityTransferStartBlock::Session {
                source_faction_id,
                target_faction_id,
                source_reserved: true,
                target_reserved: true,
                source,
            })?;
        Ok(CityTransferStartOutcome::Started {
            source_faction_id,
            target_master_player_id,
            net_exchange_id,
            session,
        })
    }

    /// Завершает callback: снимает обе reservation и при approve передаёт город.
    pub(crate) fn finish_city_transfer<Effects>(
        &mut self,
        game: &CGame,
        source_faction_id: i32,
        target_faction_id: i32,
        region_id: i32,
        region_name: &[u8],
        terminal: CityTransferTerminal,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<CityTransferFinishReport, CityTransferFinishBlock>
    where
        Effects: CityTransferEffects,
    {
        let mut report = CityTransferFinishReport {
            source_reservation_removed: self.remove_from_establishment_list(source_faction_id),
            target_reservation_removed: self.remove_from_establishment_list(target_faction_id),
            source_faction_found: false,
            target_faction_found: false,
            source_cities: None,
            target_city: None,
            target_union_id: None,
            broadcast: None,
        };
        if terminal != CityTransferTerminal::Approved {
            return Ok(report);
        }

        report.source_faction_found = self.faction_by_id(source_faction_id).is_some();
        report.target_faction_found = self.faction_by_id(target_faction_id).is_some();
        if !report.source_faction_found || !report.target_faction_found {
            return Ok(report);
        }
        let source_name = self
            .faction_by_id(source_faction_id)
            .expect("оба faction owner-а проверены")
            .name()
            .to_vec();
        let target_name = self
            .faction_by_id(target_faction_id)
            .expect("оба faction owner-а проверены")
            .name()
            .to_vec();
        let target_canonical_faction_id = self
            .faction_by_id(target_faction_id)
            .expect("оба faction owner-а проверены")
            .faction_id();

        let source_cities = self
            .faction_by_id_mut(source_faction_id)
            .expect("source faction owner проверен")
            .clear_owned_cities(game, &mut *update_player);
        report.source_cities = Some(match source_cities {
            Ok(source_cities) => source_cities,
            Err(source) => {
                return Err(CityTransferFinishBlock::SourceCities { report, source });
            }
        });
        let target_city = self
            .faction_by_id_mut(target_faction_id)
            .expect("target faction owner проверен")
            .add_owned_city(game, region_id, &mut *update_player);
        report.target_city = Some(match target_city {
            Ok(target_city) => target_city,
            Err(source) => {
                return Err(CityTransferFinishBlock::TargetCity { report, source });
            }
        });
        let target_union_id = match self.is_free_faction(target_canonical_faction_id) {
            FreeFactionLookup::NoUnion => 0,
            FreeFactionLookup::Union(union_id) => union_id,
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                return Err(CityTransferFinishBlock::TargetUnion { report, map_key });
            }
        };
        report.target_union_id = Some(target_union_id);
        effects.refresh_owned_city(region_id, target_faction_id, target_union_id);
        let source_name = legacy_c_string_prefix(&source_name);
        let region_name = legacy_c_string_prefix(region_name);
        let target_name = legacy_c_string_prefix(&target_name);
        let text = effects.format_world_string(
            b"WS0257",
            &[
                UnionFormatArgument::Text(source_name),
                UnionFormatArgument::Text(region_name),
                UnionFormatArgument::Text(target_name),
            ],
        );
        report.broadcast = Some(effects.broadcast_city_transfer(&text));
        Ok(report)
    }

    /// Завершает result/timeout локального `PlayerApplyForJoinConfeder`.
    pub(crate) fn finish_union_application<Effects>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        union_id: i32,
        applicant_faction_id: i32,
        terminal: UnionApplicationTerminal,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<
        OrganizingUnionApplicationCallbackReport,
        OrganizingUnionApplicationCallbackBlock,
    >
    where
        Effects: UnionAddFactionEffects,
    {
        let rejection_notice_sent = if matches!(terminal, UnionApplicationTerminal::Denied) {
            let recipient = if applicant_faction_id < 1 {
                None
            } else {
                self.faction_by_id(applicant_faction_id)
                    .map(CFaction::faction_id)
            };
            if let Some(recipient_player_id) = recipient {
                let second_text = effects.world_string(b"WS0193");
                let first_text = effects.world_string(b"WS0266");
                effects.send_organizing_info(FactionMemberInfoRequest {
                    recipient_player_id,
                    first_text: legacy_c_string_prefix(&first_text),
                    second_text: legacy_c_string_prefix(&second_text),
                    information_type: -1,
                    color: 0xFFDA_EDFE,
                    trailing_value: 0,
                });
                true
            } else {
                false
            }
        } else {
            false
        };

        let union_found = union_id > 0
            && self
                .confederations
                .get(&union_id)
                .is_some_and(Option::is_some);
        if !union_found {
            return Ok(OrganizingUnionApplicationCallbackReport {
                union_id,
                applicant_faction_id,
                union_found: false,
                rejection_notice_sent,
                join: None,
                application_cleared: false,
                establishment_reservation_removed: self
                    .remove_from_establishment_list(applicant_faction_id),
            });
        }

        let detached_membership_lookup =
            matches!(terminal, UnionApplicationTerminal::Approved)
                .then(|| self.is_free_faction(applicant_faction_id));
        let mut union = self
            .confederations
            .get_mut(&union_id)
            .and_then(Option::take)
            .expect("union entry и pointer проверены до временного take");
        let join = if let UnionApplicationTerminal::Approved = terminal {
            let manager_id = match union.player_header(|master_faction_id| {
                let Some(faction) = self.faction_by_id(master_faction_id) else {
                    return Ok(None);
                };
                faction
                    .master_id()
                    .map(Some)
                    .ok_or(UnionPlayerHeaderLookupBlock { master_faction_id })
            }) {
                Ok(manager_id) => manager_id,
                Err(source) => {
                    *self
                        .confederations
                        .get_mut(&union_id)
                        .expect("временный union slot не удаляется") = Some(union);
                    return Err(OrganizingUnionApplicationCallbackBlock::PlayerHeader {
                        union_id,
                        applicant_faction_id,
                        source,
                        rejection_notice_sent,
                    });
                }
            };
            self.detached_union_membership_lookup
                .set(detached_membership_lookup.map(|lookup| (applicant_faction_id, lookup)));
            let join_result = union.do_join(
                game,
                parameters,
                manager_id,
                applicant_faction_id,
                1,
                UNUSED_UNION_APPLICATION_TIME,
                self,
                effects,
                update_player,
            );
            self.detached_union_membership_lookup.set(None);
            match join_result {
                Ok(outcome) => Some(outcome),
                Err(source) => {
                    *self
                        .confederations
                        .get_mut(&union_id)
                        .expect("временный union slot не удаляется") = Some(union);
                    return Err(OrganizingUnionApplicationCallbackBlock::DoJoin {
                        union_id,
                        applicant_faction_id,
                        source,
                        rejection_notice_sent,
                    });
                }
            }
        } else {
            None
        };

        union.finish_union_application_callback();
        *self
            .confederations
            .get_mut(&union_id)
            .expect("временный union slot не удаляется") = Some(union);
        let establishment_reservation_removed =
            self.remove_from_establishment_list(applicant_faction_id);
        Ok(OrganizingUnionApplicationCallbackReport {
            union_id,
            applicant_faction_id,
            union_found: true,
            rejection_notice_sent,
            join,
            application_cleared: true,
            establishment_reservation_removed,
        })
    }

    /// Отправляет полный union snapshot каждому готовому клиенту одной faction.
    ///
    /// Нулевой union ID сначала разрешается через `IsFreeFaction`. Получатели
    /// обходятся по signed member-key, но в сообщение попадает live player ID;
    /// snapshot заново строится для каждого получателя, как virtual
    /// `CUnion::AddToByteArray` в исходном цикле.
    pub(crate) fn add_union_to_client_by_faction_id(
        &mut self,
        game: &CGame,
        requested_union_id: i32,
        faction_id: i32,
    ) -> Result<AddUnionToFactionOutcome, AddUnionToFactionBlock> {
        let union_id = if requested_union_id == 0 {
            match self.is_free_faction(faction_id) {
                FreeFactionLookup::NoUnion => 0,
                FreeFactionLookup::Union(union_id) => union_id,
                FreeFactionLookup::BlockedNullConfederation { map_key } => {
                    return Err(AddUnionToFactionBlock::FreeFactionScan { map_key });
                }
            }
        } else {
            requested_union_id
        };
        if union_id < 1 {
            return Ok(AddUnionToFactionOutcome::Rejected(
                AddUnionToFactionRejection::UnionIdNotResolved { requested_union_id },
            ));
        }

        match self.confederations.get(&union_id) {
            None => {
                return Ok(AddUnionToFactionOutcome::Rejected(
                    AddUnionToFactionRejection::UnionEntryMissing { union_id },
                ));
            }
            Some(None) => {
                return Ok(AddUnionToFactionOutcome::Rejected(
                    AddUnionToFactionRejection::NullUnionPointer { union_id },
                ));
            }
            Some(Some(_)) => {}
        }

        let (factions, confederations) = (&self.factions, &mut self.confederations);
        let union = confederations
            .get_mut(&union_id)
            .and_then(Option::as_deref_mut)
            .expect("union entry и pointer проверены до разделения borrow");
        send_union_snapshot_to_faction(game, factions, union, faction_id)
    }

    /// Удаляет player из apply-list каждой faction и на normal return даёт `0`.
    pub(crate) fn remove_person_from_apply_faction_list(
        &mut self,
        game: &CGame,
        player_id: i32,
    ) -> RemovePersonFromApplyFactionListOutcome {
        let mut removals = Vec::with_capacity(self.factions.len());
        for (&map_key, faction) in &mut self.factions {
            let Some(faction) = faction.as_deref_mut() else {
                return RemovePersonFromApplyFactionListOutcome::BlockedNullFaction {
                    map_key,
                    completed_removals: removals,
                };
            };
            removals.push(ApplyFactionRemoval {
                map_key,
                outcome: faction.remove_apply_member(game, player_id),
            });
        }
        RemovePersonFromApplyFactionListOutcome::Completed { removals }
    }

    /// Возвращает ID первой faction с положительным apply-membership.
    pub(crate) fn faction_by_player_in_apply_list(&self, player_id: i32) -> ApplyFactionLookup {
        for (&map_key, faction) in &self.factions {
            let Some(faction) = faction.as_deref() else {
                return ApplyFactionLookup::BlockedNullFaction { map_key };
            };
            if faction.is_in_apply_members(player_id) > 0 {
                return ApplyFactionLookup::Faction(faction.faction_id());
            }
        }
        ApplyFactionLookup::NoFaction
    }

    /// Связывает один точный SetPlayerOrganizing с достигнутым region snapshot.
    pub(crate) const fn player_updater<'a>(
        &'a self,
        region_types: &'a BTreeMap<i32, Option<u16>>,
    ) -> COrganizingPlayerUpdater<'a> {
        COrganizingPlayerUpdater {
            controller: self,
            region_types,
        }
    }

    /// Выполняет faction enter-ветвь и затем безусловную top-info отправку.
    pub(crate) fn on_player_enter_game(
        &mut self,
        game: &CGame,
        player_id: i32,
    ) -> PlayerEnterGameOutcome {
        let faction_id = match self.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => None,
            FreePlayerLookup::Faction(faction_id) => Some(faction_id),
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return PlayerEnterGameOutcome::BlockedDuringFactionScan { map_key };
            }
        };

        let faction = match faction_id {
            None => FactionEnterDispatch::NoFactionMembership,
            Some(faction_id) => match self.factions.get_mut(&faction_id) {
                None => FactionEnterDispatch::FactionEntryMissing { faction_id },
                Some(None) => FactionEnterDispatch::NullFactionPointer { faction_id },
                Some(Some(faction)) => {
                    let outcome = faction.on_member_enter_game(game, player_id);
                    if matches!(
                        &outcome,
                        MemberEnterOutcome::Blocked(_) | MemberEnterOutcome::Published(Err(_))
                    ) {
                        // Старый faction callback на этих UB-границах не
                        // возвращался бы к последующей top-info отправке.
                        return PlayerEnterGameOutcome::BlockedDuringFactionCallback {
                            faction_id,
                            outcome,
                        };
                    }
                    FactionEnterDispatch::Called {
                        faction_id,
                        outcome,
                    }
                }
            },
        };

        let top_info = self.send_all_top_info_to_one_client(game, player_id);
        PlayerEnterGameOutcome::Completed { faction, top_info }
    }

    /// Выполняет единственную faction exit-ветвь без дополнительных эффектов.
    pub(crate) fn on_player_exit_game(
        &mut self,
        game: &CGame,
        player_id: i32,
    ) -> PlayerExitGameOutcome {
        let faction_id = match self.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => None,
            FreePlayerLookup::Faction(faction_id) => Some(faction_id),
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return PlayerExitGameOutcome::BlockedDuringFactionScan { map_key };
            }
        };

        let faction = match faction_id {
            None => FactionExitDispatch::NoFactionMembership,
            Some(faction_id) => match self.factions.get_mut(&faction_id) {
                None => FactionExitDispatch::FactionEntryMissing { faction_id },
                Some(None) => FactionExitDispatch::NullFactionPointer { faction_id },
                Some(Some(faction)) => {
                    let outcome = faction.on_member_exit_game(game, player_id);
                    if matches!(&outcome, MemberExitOutcome::Published(Err(_))) {
                        return PlayerExitGameOutcome::BlockedDuringFactionCallback {
                            faction_id,
                            outcome,
                        };
                    }
                    FactionExitDispatch::Called {
                        faction_id,
                        outcome,
                    }
                }
            },
        };
        PlayerExitGameOutcome::Dispatched(faction)
    }

    /// Сохраняет новую top-info запись в хвост и возвращает process-static ID.
    pub(crate) fn add_one_top_info(&mut self, timer_flag: i32, param: i32, info: &[u8]) -> i32 {
        let id = NEXT_TOP_INFO_ID.fetch_add(1, Ordering::Relaxed);
        let started_at_ms = legacy_tick_ms();
        self.top_infos.push_back(StTopInfo {
            id,
            timer_flag,
            param,
            started_at_ms,
            info: info.to_vec(),
        });
        id
    }

    /// Удаляет все истёкшие timer-2 записи, сохраняя порядок остальных.
    pub(crate) fn run_top_info_expiry(&mut self) -> usize {
        let now_ms = legacy_tick_ms();
        let old_len = self.top_infos.len();
        self.top_infos.retain(|top_info| {
            top_info.timer_flag != EXPIRING_TIMER_FLAG
                || (top_info.param as u32) > now_ms.wrapping_sub(top_info.started_at_ms)
        });
        old_len - self.top_infos.len()
    }

    /// Выполняет полный ordered countdown, disband-dispatch и top-info expiry.
    pub(crate) fn run<Disband>(
        &mut self,
        minute_delta: i32,
        mut disband_faction: Disband,
    ) -> Result<OrganizingRunReport, OrganizingRunBlock>
    where
        Disband: FnMut(&mut Self, i32, i32) -> bool,
    {
        let mut pending_disbands = BTreeMap::new();
        let mut decremented_factions = 0;
        for (&map_key, faction) in &mut self.factions {
            let Some(faction) = faction.as_deref_mut() else {
                return Err(OrganizingRunBlock::NullFaction { map_key });
            };
            let Some(remaining) = faction.delete_remain_time() else {
                return Err(OrganizingRunBlock::DeleteRemainTimeAbsent { map_key });
            };
            if 0 < remaining {
                let remaining = remaining.wrapping_sub(minute_delta);
                faction.set_delete_remain_time(remaining);
                decremented_factions += 1;
                if remaining < 1 {
                    let faction_id = faction.faction_id();
                    let master_id =
                        faction
                            .master_id()
                            .ok_or(OrganizingRunBlock::MasterIdAbsent {
                                map_key,
                                faction_id,
                            })?;
                    pending_disbands.insert(faction_id, master_id);
                }
            }
        }

        let mut disbands = Vec::with_capacity(pending_disbands.len());
        for (faction_id, master_id) in pending_disbands {
            let result = disband_faction(self, master_id, faction_id);
            disbands.push(OrganizingRunDisband {
                faction_id,
                master_id,
                result,
            });
        }
        let expired_top_infos = self.run_top_info_expiry();
        Ok(OrganizingRunReport {
            decremented_factions,
            disbands,
            expired_top_infos,
        })
    }

    /// Рассылает одну top-info запись всем GameServer с player ID `0`.
    pub(crate) fn send_top_info_to_client(
        &self,
        game: &CGame,
        top_info_id: i32,
        timer_flag: i32,
        param: i32,
        info: &[u8],
    ) -> Result<i32, SendMessageError> {
        let message = top_info_message(0, top_info_id, timer_flag, param, info);
        let sender = game.current_game_server_sender();
        message.send_all(sender.as_ref())
    }

    /// Отправляет одному игроку все неистёкшие top-info записи в list-порядке.
    pub(crate) fn send_all_top_info_to_one_client(
        &self,
        game: &CGame,
        player_id: i32,
    ) -> TopInfoDeliveryReport {
        if self.top_infos.is_empty() {
            return TopInfoDeliveryReport {
                game_server_id: None,
                skipped_expired: 0,
                deliveries: Vec::new(),
            };
        }

        let game_server_id = game.game_server_number_by_player_id(player_id);
        let now_ms = legacy_tick_ms();
        let mut skipped_expired = 0;
        let mut deliveries = Vec::with_capacity(self.top_infos.len());

        for top_info in &self.top_infos {
            let param = if top_info.timer_flag == EXPIRING_TIMER_FLAG {
                let elapsed_ms = now_ms.wrapping_sub(top_info.started_at_ms);
                if elapsed_ms >= top_info.param as u32 {
                    skipped_expired += 1;
                    continue;
                }
                (top_info.param as u32).wrapping_add(top_info.started_at_ms.wrapping_sub(now_ms))
                    as i32
            } else {
                top_info.param
            };
            let message = top_info_message(
                player_id,
                top_info.id,
                top_info.timer_flag,
                param,
                &top_info.info,
            );
            deliveries.push(TopInfoDelivery {
                top_info_id: top_info.id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }

        TopInfoDeliveryReport {
            game_server_id: Some(game_server_id),
            skipped_expired,
            deliveries,
        }
    }
}

impl FactionOperationAuthorityContext for COrganizingCtrl {
    type Block = FactionUnionMembershipLookupBlock;

    fn union_id_for_faction(&self, faction_id: i32) -> Result<i32, Self::Block> {
        match self.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => Ok(0),
            FreeFactionLookup::Union(union_id) => Ok(union_id),
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                Err(FactionUnionMembershipLookupBlock { map_key })
            }
        }
    }

    fn union_master_faction_id(&self, union_id: i32) -> Option<i32> {
        self.confederation_by_id(union_id).map(CUnion::master_id)
    }
}

impl FactionPlayerHeaderContext for COrganizingCtrl {
    type Block = UnionPlayerHeaderLookupBlock;

    fn union_player_header(&self, union_id: i32) -> Result<Option<i32>, Self::Block> {
        let Some(union) = self.confederation_by_id(union_id) else {
            return Ok(None);
        };
        let player_header = union.player_header(|master_faction_id| {
            let Some(faction) = self.faction_by_id(master_faction_id) else {
                return Ok(None);
            };
            faction
                .master_id()
                .map(Some)
                .ok_or(UnionPlayerHeaderLookupBlock {
                    master_faction_id,
                })
        })?;
        Ok(Some(player_header))
    }
}

impl UnionOperatorValidationContext for COrganizingCtrl {
    type Block = FactionMasterLookupBlock;

    fn faction_id_by_master_player(&self, player_id: i32) -> Result<i32, Self::Block> {
        COrganizingCtrl::faction_id_by_master_player(self, player_id)
    }
}

impl UnionDoJoinContext for COrganizingCtrl {
    type FreeFactionBlock = FactionUnionMembershipLookupBlock;
    type InitialSnapshotBlock = AddUnionToFactionBlock;

    fn union_id_for_joining_faction(
        &self,
        faction_id: i32,
    ) -> Result<i32, Self::FreeFactionBlock> {
        let cached = self.detached_union_membership_lookup.get();
        let lookup = match cached {
            Some((cached_faction_id, lookup)) if cached_faction_id == faction_id => {
                self.detached_union_membership_lookup.set(None);
                lookup
            }
            _ => self.is_free_faction(faction_id),
        };
        match lookup {
            FreeFactionLookup::NoUnion => Ok(0),
            FreeFactionLookup::Union(union_id) => Ok(union_id),
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                Err(FactionUnionMembershipLookupBlock { map_key })
            }
        }
    }

    fn add_current_union_to_client_by_faction_id(
        &mut self,
        game: &CGame,
        union: &mut CUnion,
        faction_id: i32,
    ) -> Result<bool, Self::InitialSnapshotBlock> {
        send_union_snapshot_to_faction(game, &self.factions, union, faction_id)
            .map(|outcome| matches!(outcome, AddUnionToFactionOutcome::Sent { .. }))
    }
}

impl UnionApplyForJoinContext for COrganizingCtrl {
    fn union_application_is_reserved(&self, faction_id: i32) -> bool {
        self.is_union_application_reserved(faction_id)
    }

    fn union_application_faction(
        &self,
        faction_id: i32,
    ) -> Result<Option<UnionApplicationFactionSnapshot>, UnionApplicationFactionBlock> {
        if faction_id < 1 {
            return Ok(None);
        }
        let Some(faction) = self.faction_by_id(faction_id) else {
            return Ok(None);
        };
        let player_header = faction
            .master_id()
            .ok_or(UnionApplicationFactionBlock { faction_id })?;
        Ok(Some(UnionApplicationFactionSnapshot {
            faction_id,
            name: faction.name().to_vec(),
            player_header,
        }))
    }

    fn reserve_union_application(&mut self, faction_id: i32) {
        self.push_to_establishment_list(faction_id);
    }
}

impl UnionMasterFactionQueryContext for COrganizingCtrl {
    fn faction_is_owned_city(&self, faction_id: i32, region_id: i32) -> Option<i32> {
        self.faction_by_id(faction_id)
            .map(|faction| faction.is_owned_city(region_id))
    }

    fn faction_is_enemy_faction(&self, faction_id: i32, enemy_id: i32) -> Option<i32> {
        self.faction_by_id(faction_id)
            .map(|faction| faction.is_enemy_faction(enemy_id))
    }

    fn faction_owned_cities(&self, faction_id: i32) -> Option<VecDeque<i32>> {
        self.faction_by_id(faction_id)
            .map(|faction| faction.owned_cities().clone())
    }

    fn faction_has_enemy(&self, faction_id: i32) -> Option<bool> {
        self.faction_by_id(faction_id)
            .map(CFaction::has_enemy_faction)
    }

    fn faction_has_city_war_enemy(&self, faction_id: i32) -> Option<bool> {
        self.faction_by_id(faction_id)
            .map(CFaction::has_city_war_enemy_faction)
    }

    fn faction_enemy_leader_organizing_id(&self, faction_id: i32) -> Option<i32> {
        self.faction_by_id(faction_id)
            .map(CFaction::enemy_leader_organizing_id)
    }
}

impl UnionOwnedCityMutationContext for COrganizingCtrl {
    fn faction_add_owned_city(
        &mut self,
        faction_id: i32,
        game: &CGame,
        region_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<bool, OwnedCityMutationBuildError> {
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(false);
        };
        faction
            .add_owned_city(game, region_id, update_player)
            .map(|_| true)
    }

    fn faction_add_owned_cities(
        &mut self,
        faction_id: i32,
        game: &CGame,
        region_ids: &VecDeque<i32>,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<bool, OwnedCityMutationBuildError> {
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(false);
        };
        faction
            .add_owned_city_list(game, region_ids, update_player)
            .map(|_| true)
    }

    fn faction_clear_owned_cities(
        &mut self,
        faction_id: i32,
        game: &CGame,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<bool, OwnedCityMutationBuildError> {
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(false);
        };
        faction
            .clear_owned_cities(game, update_player)
            .map(|_| true)
    }

    fn faction_set_owned_cities(
        &mut self,
        faction_id: i32,
        game: &CGame,
        region_ids: &VecDeque<i32>,
    ) -> Result<bool, OwnedCityMutationBuildError> {
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(false);
        };
        faction.set_owned_cities(game, region_ids).map(|_| true)
    }
}

impl UnionFactionStateMutationContext for COrganizingCtrl {
    fn faction_clear_enemy_factions(&mut self, faction_id: i32) -> bool {
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return false;
        };
        faction.clear_enemy_factions();
        true
    }

    fn faction_add_defence_victor_count(
        &mut self,
        faction_id: i32,
        game: &CGame,
    ) -> Result<Option<Vec<FactionPropertyDelivery>>, FactionInitialPropertyBlock> {
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(None);
        };
        faction.add_defence_victor_count(game).map(Some)
    }

    fn faction_add_offense_victor_count(
        &mut self,
        faction_id: i32,
        game: &CGame,
    ) -> Result<Option<Vec<FactionPropertyDelivery>>, FactionInitialPropertyBlock> {
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(None);
        };
        faction.add_offense_victor_count(game).map(Some)
    }

    fn faction_add_village_war_victor_count(
        &mut self,
        faction_id: i32,
        game: &CGame,
    ) -> Result<Option<Vec<FactionPropertyDelivery>>, FactionInitialPropertyBlock> {
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(None);
        };
        faction.add_village_war_victor_count(game).map(Some)
    }
}

impl UnionPlayerRefreshContext for COrganizingCtrl {
    fn faction_update_player_info(
        &self,
        faction_id: i32,
        game: &CGame,
        update_player: &mut dyn FnMut(i32),
    ) -> Option<Vec<i32>> {
        self.faction_by_id(faction_id).map(|faction| {
            faction.update_player_faction_info(game, 0, update_player)
        })
    }
}

impl UnionInitialMutationContext for COrganizingCtrl {
    fn faction_set_superior_organizing(
        &mut self,
        faction_id: i32,
        union_id: i32,
        parameters: &COrganizingParam,
    ) -> Result<bool, FactionSuperiorOrganizingBlock> {
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(false);
        };
        faction.set_superior_organizing(union_id, parameters)?;
        Ok(true)
    }
}

impl UnionFactionJoinContext for COrganizingCtrl {
    fn faction_refresh_owned_city_info(
        &self,
        faction_id: i32,
        refresh_owned_city: &mut dyn FnMut(i32, i32, i32),
    ) -> Result<Option<FactionOwnedCityRefreshReport>, FactionOwnedCityRefreshBlock> {
        self.faction_by_id(faction_id)
            .map(|faction| faction.refresh_owned_city_info(refresh_owned_city))
            .transpose()
    }
}

impl UnionClientSnapshotContext for COrganizingCtrl {
    fn faction_update_enemy_snapshot(
        &self,
        faction_id: i32,
        game: &CGame,
    ) -> Option<Vec<FactionEnemyDelivery>> {
        self.faction_by_id(faction_id)
            .map(|faction| faction.update_enemy_factions_to_client(game))
    }

    fn faction_update_city_war_enemy_snapshot(
        &self,
        faction_id: i32,
        game: &CGame,
    ) -> Option<Vec<FactionEnemyDelivery>> {
        self.faction_by_id(faction_id)
            .map(|faction| faction.update_city_war_enemy_factions_to_client(game))
    }

    fn faction_update_owned_city_snapshot(
        &self,
        faction_id: i32,
        game: &CGame,
    ) -> Result<Option<Vec<FactionOwnedCityDelivery>>, FactionOwnedCityUpdateBuildError> {
        self.faction_by_id(faction_id)
            .map(|faction| faction.update_owned_cities_to_client(game))
            .transpose()
    }
}

impl UnionSendInfoContext for COrganizingCtrl {
    fn faction_send_info_to_members<'a>(
        &self,
        faction_id: i32,
        first_text: &'a [u8],
        second_text: &'a [u8],
        information_type: i32,
        color: u32,
        send_organizing_info: &mut dyn FnMut(FactionMemberInfoRequest<'a>),
    ) -> Option<FactionMemberInfoReport> {
        self.faction_by_id(faction_id).map(|faction| {
            faction.send_info_to_all_members_with_color(
                first_text,
                second_text,
                information_type,
                color,
                send_organizing_info,
            )
        })
    }
}

impl UnionFactionMemberContext for COrganizingCtrl {
    fn faction_member_player_ids(&self, faction_id: i32) -> Option<Vec<i32>> {
        self.faction_by_id(faction_id)
            .map(|faction| faction.get_members().keys().copied().collect())
    }

    fn faction_name(&self, faction_id: i32) -> Option<Vec<u8>> {
        self.faction_by_id(faction_id)
            .map(|faction| faction.name().to_vec())
    }

    fn faction_level(
        &self,
        faction_id: i32,
    ) -> Result<Option<i32>, UnionFactionLevelBlock> {
        let Some(faction) = self.faction_by_id(faction_id) else {
            return Ok(None);
        };
        faction
            .level()
            .map(Some)
            .ok_or(UnionFactionLevelBlock { faction_id })
    }
}

impl UnionFactionMemberContext for UnionFactionMapView<'_> {
    fn faction_member_player_ids(&self, faction_id: i32) -> Option<Vec<i32>> {
        self.factions
            .get(&faction_id)
            .and_then(Option::as_deref)
            .map(|faction| faction.get_members().keys().copied().collect())
    }

    fn faction_name(&self, faction_id: i32) -> Option<Vec<u8>> {
        self.factions
            .get(&faction_id)
            .and_then(Option::as_deref)
            .map(|faction| faction.name().to_vec())
    }

    fn faction_level(
        &self,
        faction_id: i32,
    ) -> Result<Option<i32>, UnionFactionLevelBlock> {
        let Some(faction) = self
            .factions
            .get(&faction_id)
            .and_then(Option::as_deref)
        else {
            return Ok(None);
        };
        faction
            .level()
            .map(Some)
            .ok_or(UnionFactionLevelBlock { faction_id })
    }
}

fn send_union_snapshot_to_faction(
    game: &CGame,
    factions: &BTreeMap<i32, Option<Box<CFaction>>>,
    union: &mut CUnion,
    faction_id: i32,
) -> Result<AddUnionToFactionOutcome, AddUnionToFactionBlock> {
    let member_map_keys = match factions.get(&faction_id) {
        Some(Some(faction)) if faction_id > 0 => {
            faction.get_members().keys().copied().collect::<Vec<_>>()
        }
        faction => {
            return Err(AddUnionToFactionBlock::FactionUnavailable {
                faction_id,
                entry_present: faction.is_some(),
            });
        }
    };

    let union_id = union.union_id();
    let faction_view = UnionFactionMapView { factions };
    let mut deliveries = Vec::new();
    for member_map_key in member_map_keys {
        let Some(player) = game.online_player_by_id(member_map_key as u32) else {
            continue;
        };
        let game_server_id = game.game_server_number_by_player_id(member_map_key);
        if game_server_id == 0 || !player.faction_data_received() {
            continue;
        }

        let recipient_player_id = player.get_id();
        let mut message = CMessage::new(UNION_INITIAL_MESSAGE_TYPE);
        message.base_mut().add_long(recipient_player_id);
        let mut snapshot = Vec::new();
        if let Err(source) = union.add_to_byte_array(&mut snapshot, &faction_view) {
            return Err(AddUnionToFactionBlock::Snapshot {
                union_id,
                faction_id,
                member_map_key,
                recipient_player_id,
                game_server_id,
                source,
                completed_deliveries: deliveries,
            });
        }
        message.base_mut().add(&snapshot);
        deliveries.push(AddUnionToFactionDelivery {
            member_map_key,
            recipient_player_id,
            game_server_id,
            result: game.send_msg_to_game_server(game_server_id, &message),
        });
    }

    Ok(AddUnionToFactionOutcome::Sent {
        union_id,
        faction_id,
        deliveries,
    })
}

impl PlayerOrganizingUpdater for COrganizingPlayerUpdater<'_> {
    fn set_player_organizing(
        &mut self,
        player_id: i32,
        organizing: &mut PlayerOrganizingState,
    ) -> Result<(), PlayerOrganizingUpdateError> {
        let faction_id = match self.controller.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => 0,
            FreePlayerLookup::Faction(faction_id) => faction_id,
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return Err(
                    PlayerOrganizingUpdateError::NullFactionDuringMembershipScan { map_key },
                );
            }
        };
        organizing.faction_id = faction_id;

        if faction_id > 0
            && let Some(Some(faction)) = self.controller.factions.get(&faction_id)
        {
            let level =
                faction
                    .level()
                    .ok_or(PlayerOrganizingUpdateError::UninitializedFactionField {
                        faction_id,
                        field: "m_Property.lLvl",
                    })?;
            organizing.faction_level = level as u16;

            organizing.faction_experience = faction.experience().ok_or(
                PlayerOrganizingUpdateError::UninitializedFactionField {
                    faction_id,
                    field: "m_Property.lExp",
                },
            )?;
            organizing.faction_contribute = faction.is_contribute(player_id);
            organizing.faction_name = faction.name().to_vec();
            organizing.faction_title = faction.member_title(player_id).map_err(|_| {
                PlayerOrganizingUpdateError::UnterminatedFactionMemberTitle {
                    faction_id,
                    player_id,
                }
            })?;
            organizing.faction_master_id = faction.master_id().ok_or(
                PlayerOrganizingUpdateError::UninitializedFactionField {
                    faction_id,
                    field: "m_lMastterID",
                },
            )?;
            organizing.enemy_factions = faction.enemy_factions().clone();
            organizing.city_war_enemy_factions = faction.city_war_enemy_factions().clone();
            organizing.owned_regions.clear();

            for &region_id in faction.owned_cities() {
                let Some(region_type) = self.region_types.get(&region_id) else {
                    continue;
                };
                let Some(_region_type) = *region_type else {
                    return Err(PlayerOrganizingUpdateError::UninitializedRegionType { region_id });
                };
                // BLOCKED_MISSING_FACT: AddOwnedRegion RVA `0x0005DD10`
                // копирует два неинициализированных padding-байта local
                // tagOwnedReg. Они входят в последующий player-wire.
                return Err(
                    PlayerOrganizingUpdateError::UninitializedOwnedRegionPadding { region_id },
                );
            }
        }

        let union_id = match self.controller.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => 0,
            FreeFactionLookup::Union(union_id) => union_id,
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                return Err(
                    PlayerOrganizingUpdateError::NullConfederationDuringMembershipScan { map_key },
                );
            }
        };
        organizing.union_id = union_id;

        if union_id > 0
            && let Some(Some(union)) = self.controller.confederations.get(&union_id)
        {
            organizing.union_master_id = union.master_id();
        }
        Ok(())
    }
}

fn send_disband_information<Context>(
    context: &mut Context,
    player_id: i32,
    first_string_id: &'static [u8],
) where
    Context: FactionOrganizingInfoContext + ?Sized,
{
    let second_text = context.world_string(b"WS0121").unwrap_or_default();
    let first_text = context.world_string(first_string_id).unwrap_or_default();
    context.send_organizing_info(FactionMemberInfoRequest {
        recipient_player_id: player_id,
        first_text: legacy_c_string_prefix(&first_text),
        second_text: legacy_c_string_prefix(&second_text),
        information_type: -1,
        color: 0xFFDA_EDFE,
        trailing_value: 0,
    });
}

fn declare_war_union_id(
    organizing: &COrganizingCtrl,
    faction_id: i32,
) -> Result<Option<i32>, DeclareWarFactionPageBlock> {
    match organizing.is_free_faction(faction_id) {
        FreeFactionLookup::NoUnion => Ok(None),
        FreeFactionLookup::Union(union_id) => Ok(Some(union_id)),
        FreeFactionLookup::BlockedNullConfederation { map_key } => {
            Err(DeclareWarFactionPageBlock::UnionMembership {
                faction_id,
                map_key,
            })
        }
    }
}

fn append_i32(output: &mut Vec<u8>, value: i32) {
    output.extend_from_slice(&value.to_le_bytes());
}

#[derive(Clone, Copy)]
enum BillboardTimeOrder {
    Early,
    Late,
}

fn finish_billboard_rows(
    rows: &mut Vec<(i32, TagTimeValue, FactionBillboardEntry)>,
    time_order: BillboardTimeOrder,
) {
    rows.sort_by(|left, right| {
        let number_order = right.0.cmp(&left.0);
        if number_order != CmpOrdering::Equal {
            return number_order;
        }
        match time_order {
            BillboardTimeOrder::Early => {
                billboard_time_fields(left.1).cmp(&billboard_time_fields(right.1))
            }
            BillboardTimeOrder::Late => {
                billboard_time_fields(right.1).cmp(&billboard_time_fields(left.1))
            }
        }
    });

    // Исходный `std::map<tagKey, ...>` не имел faction-ID tie-break:
    // одинаковые number и шесть сравниваемых полей времени оставляли первую
    // faction из signed map-order. Stable sort + retain сохраняют именно её.
    let mut previous_key = None;
    rows.retain(|(number, time, _)| {
        let key = (*number, billboard_time_fields(*time));
        if previous_key == Some(key) {
            false
        } else {
            previous_key = Some(key);
            true
        }
    });
}

fn billboard_time_fields(time: TagTimeValue) -> [u16; 6] {
    [
        time.year,
        time.month,
        time.day,
        time.hour,
        time.minute,
        time.second,
    ]
}

fn top_info_message(
    player_id: i32,
    top_info_id: i32,
    timer_flag: i32,
    param: i32,
    info: &[u8],
) -> CMessage {
    let mut message = CMessage::new(TOP_INFO_MESSAGE_TYPE);
    let payload = message.base_mut();
    payload.add_long(player_id);
    payload.add_long(top_info_id);
    payload.add_long(timer_flag);
    payload.add_long(param);
    let info = legacy_c_string_prefix(info);
    payload.add(info);
    payload.add_byte(0);
    message
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}

fn legacy_tick_ms() -> u32 {
    let now = clock_gettime(ClockId::Boottime);
    let seconds_ms = (now.tv_sec as u64).wrapping_mul(1_000);
    let nanoseconds_ms = (now.tv_nsec as u64) / 1_000_000;
    seconds_ms.wrapping_add(nanoseconds_ms) as u32
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp

// ============================================================================
// FUNCTION: COrganizingCtrl::GetFactionOrganizing
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.h:117
// RVA: 0x0001C050
// ADDRESS: 0041c050
// PROTOTYPE: COrganizing * __thiscall GetFactionOrganizing(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::GetFactionNumber
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:445
// RVA: 0x00033560
// ADDRESS: 00433560
// PROTOTYPE: long __thiscall GetFactionNumber(void)
//
// IMPLEMENTED_OWNER: `declare_war_faction_count` возвращает exact map-size
// как Windows `long`; oversized safe-state остаётся явной block-границей.
// RAW_REFERENCE_BEGIN: сохранённая декомпиляция реализованной функции.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
// RAW_REFERENCE_END: `GetFactionNumber()`.

// ============================================================================
// FUNCTION: `public:_enum_eCrOrgResult___thiscall_COrganizingCtrl::CreateConfederation(long,long,std::basic_string<char,std::char_traits<char>,std::allocator<char>_>&)'::__l28::CreateUnion::DoAsyncCall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:937
// RVA: 0x00033680
// ADDRESS: 00433680
// PROTOTYPE: void __thiscall DoAsyncCall(__int64 param_1, long param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::SendOrgaInfoToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1923
// RVA: 0x00033750
// ADDRESS: 00433750
// PROTOTYPE: void __thiscall SendOrgaInfoToClient(long param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3, long param_4, ulong param_5, ulong param_6)
// IMPLEMENTED_OWNER: `send_organizing_info_to_client` сохраняет route gate,
// byte-exact `0x7F804` payload и исходно игнорировавшийся send result.

// ============================================================================
// FUNCTION: COrganizingCtrl::SendOrgaInfoToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1939
// RVA: 0x00033840
// ADDRESS: 00433840
// PROTOTYPE: void __thiscall SendOrgaInfoToClient(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, ulong param_2, ulong param_3)
//
// IMPLEMENTED_OWNER: `send_organizing_info_to_all` сохраняет exact `0x7FA03`
// wire-order, C-string prefix и исходно игнорировавшийся `SendAll` result.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddFactionMemBillboardToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:333
// RVA: 0x000339D0
// ADDRESS: 004339d0
// PROTOTYPE: void __thiscall AddFactionMemBillboardToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddFactionOffBillboardToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:348
// RVA: 0x00033A50
// ADDRESS: 00433a50
// PROTOTYPE: void __thiscall AddFactionOffBillboardToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddFactionDefBillboardToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:363
// RVA: 0x00033AD0
// ADDRESS: 00433ad0
// PROTOTYPE: void __thiscall AddFactionDefBillboardToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::IsInEstaList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.h:143
// RVA: 0x00033BA0
// ADDRESS: 00433ba0
// PROTOTYPE: bool __thiscall IsInEstaList(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddFactionBillboardToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:312
// RVA: 0x00033CC0
// ADDRESS: 00433cc0
// PROTOTYPE: void __thiscall AddFactionBillboardToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::GetFactionNumber
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:451
// RVA: 0x00033D10
// ADDRESS: 00433d10
// PROTOTYPE: long __thiscall GetFactionNumber(uchar param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddFactionListToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:492
// RVA: 0x00033D90
// ADDRESS: 00433d90
// PROTOTYPE: void __thiscall AddFactionListToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, long param_2, uchar param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_bool___thiscall_COrganizingCtrl::TransferIOwnerCity(long,long,long)'::__l42::PlayerTransferOwnerCity::PlayerTransferOwnerCity
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1211
// RVA: 0x00033F40
// ADDRESS: 00433f40
// PROTOTYPE: undefined __thiscall PlayerTransferOwnerCity(long param_1, long param_2, long param_3, long param_4, long param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::GenerateDBOrganizingID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:214
// RVA: 0x000341C0
// ADDRESS: 004341c0
// PROTOTYPE: long __thiscall GenerateDBOrganizingID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::SetAllCityFacEnemyChanged
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1312
// RVA: 0x00034240
// ADDRESS: 00434240
// PROTOTYPE: void __thiscall SetAllCityFacEnemyChanged(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::ClearAllCityFacRelation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1322
// RVA: 0x000342C0
// ADDRESS: 004342c0
// PROTOTYPE: void __thiscall ClearAllCityFacRelation(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::UpdateAllCityEneFacRelation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1340
// RVA: 0x00034330
// ADDRESS: 00434330
// PROTOTYPE: void __thiscall UpdateAllCityEneFacRelation(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::IsFreeFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1368
// RVA: 0x00034420
// ADDRESS: 00434420
// PROTOTYPE: long __thiscall IsFreeFaction(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::IsFactionMaster
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1398
// RVA: 0x000344A0
// ADDRESS: 004344a0
// PROTOTYPE: long __thiscall IsFactionMaster(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::IsConferationMaster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1414
// RVA: 0x00034520
// ADDRESS: 00434520
// PROTOTYPE: long __thiscall IsConferationMaster(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::FindOrgaByName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1436
// RVA: 0x000345A0
// ADDRESS: 004345a0
// PROTOTYPE: COrganizing * __thiscall FindOrgaByName(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::ReSetPermitDemise
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1522
// RVA: 0x00034810
// ADDRESS: 00434810
// PROTOTYPE: void __thiscall ReSetPermitDemise(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::RemovePersonFromApplyFactionList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1531
// RVA: 0x00034880
// ADDRESS: 00434880
// PROTOTYPE: long __thiscall RemovePersonFromApplyFactionList(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::GetFactionByPlayerInApplyList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1542
// RVA: 0x000348F0
// ADDRESS: 004348f0
// PROTOTYPE: long __thiscall GetFactionByPlayerInApplyList(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::UpdateOtherFacInfoToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1910
// RVA: 0x00034980
// ADDRESS: 00434980
// PROTOTYPE: void __thiscall UpdateOtherFacInfoToClient(long param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED выше: COrganizingCtrl::GenerateSaveData RVA 0x00034A10.

// ============================================================================
// FUNCTION: COrganizingCtrl::ReInitialFacFactionByLvl
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:2050
// RVA: 0x00034C80
// ADDRESS: 00434c80
// PROTOTYPE: void __thiscall ReInitialFacFactionByLvl(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddAllFactinInfoToClientByPlayerID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:405
// RVA: 0x00034D00
// ADDRESS: 00434d00
// PROTOTYPE: bool __thiscall AddAllFactinInfoToClientByPlayerID(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_enum_eCrOrgResult___thiscall_COrganizingCtrl::CreateConfederation(long,long,std::basic_string<char,std::char_traits<char>,std::allocator<char>_>&)'::__l28::CreateUnion::CreateUnion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:931
// RVA: 0x00034E90
// ADDRESS: 00434e90
// PROTOTYPE: undefined __thiscall CreateUnion(long param_1, long param_2, long param_3, long param_4, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_enum_eCrOrgResult___thiscall_COrganizingCtrl::CreateConfederation(long,long,std::basic_string<char,std::char_traits<char>,std::allocator<char>_>&)'::__l28::CreateUnion::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:934
// RVA: 0x00034EF0
// ADDRESS: 00434ef0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_bool___thiscall_COrganizingCtrl::TransferIOwnerCity(long,long,long)'::__l42::PlayerTransferOwnerCity::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1214
// RVA: 0x00034F10
// ADDRESS: 00434f10
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::~COrganizingCtrl
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:87
// RVA: 0x00036830
// ADDRESS: 00436830
// PROTOTYPE: void __thiscall ~COrganizingCtrl(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::GetConfederationOrganizing
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.h:132
// RVA: 0x00036BF0
// ADDRESS: 00436bf0
// PROTOTYPE: COrganizing * __thiscall GetConfederationOrganizing(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::COrganizingCtrl
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:73
// RVA: 0x00036C90
// ADDRESS: 00436c90
// PROTOTYPE: undefined __thiscall COrganizingCtrl(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::getInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:93
// RVA: 0x00036E90
// ADDRESS: 00436e90
// PROTOTYPE: COrganizingCtrl * __cdecl getInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:143
// RVA: 0x00036F40
// ADDRESS: 00436f40
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddFactionOrganizing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:234
// RVA: 0x00037030
// ADDRESS: 00437030
// PROTOTYPE: void __thiscall AddFactionOrganizing(long param_1, COrganizing * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::SetPlayerOrganizing
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:242
// RVA: 0x000370A0
//
// Реализация находится в `COrganizingPlayerUpdater`; exact диапазон
// `0x004370A0..0x0043737A` исправляет raw key/dataflow и восстанавливает
// пропущенные city-war/owned-region операции перед union-ветвью.
//
// ============================================================================
// FUNCTION: COrganizingCtrl::AddFactionToClientByPlayerID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:379
// RVA: 0x00037380
// ADDRESS: 00437380
// PROTOTYPE: bool __thiscall AddFactionToClientByPlayerID(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddDeclareWarFactionInfoToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:532
// RVA: 0x000374B0
// ADDRESS: 004374b0
// PROTOTYPE: void __thiscall AddDeclareWarFactionInfoToByteArray(long param_1, vector<unsigned_char,std::allocator<unsigned_char>_> * param_2, long param_3)
//
// IMPLEMENTED_OWNER: `declare_war_faction_page` сохраняет page clamp, signed
// map-order, exact entry wire и relation codes; нечитавшийся source-country
// вызов как ненаблюдаемый внутренний дефект удалён.
// RAW_REFERENCE_BEGIN: сохранённая декомпиляция реализованной функции.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
// RAW_REFERENCE_END: `AddDeclareWarFactionInfoToByteArray`.

// ============================================================================
// FUNCTION: COrganizingCtrl::AddUnionToClientByPlayerID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:595
// RVA: 0x000376F0
// ADDRESS: 004376f0
// PROTOTYPE: bool __thiscall AddUnionToClientByPlayerID(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_bool___thiscall_COrganizingCtrl::TransferIOwnerCity(long,long,long)'::__l42::PlayerTransferOwnerCity::DoAsyncCall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1217
// RVA: 0x00037810
// ADDRESS: 00437810
// PROTOTYPE: void __thiscall DoAsyncCall(__int64 param_1, long param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::SetEnemyFactionRelation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1296
// RVA: 0x00037A40
// ADDRESS: 00437a40
// PROTOTYPE: void __thiscall SetEnemyFactionRelation(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::GetUnion
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1502
// RVA: 0x00037AF0
// ADDRESS: 00437af0
// PROTOTYPE: COrganizing * __thiscall GetUnion(long param_1)
// IMPLEMENTED_OWNER: `union_id_by_master_player` сохраняет ordered lookup,
// nullable result и safe-границы старых malformed null owner-ов.

// ============================================================================
// FUNCTION: COrganizingCtrl::GetCountryByFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1514
// RVA: 0x00037B20
// ADDRESS: 00437b20
// PROTOTYPE: uchar __thiscall GetCountryByFaction(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddOwnedCityToFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1717
// RVA: 0x00037C20
// ADDRESS: 00437c20
// PROTOTYPE: void __thiscall AddOwnedCityToFaction(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::OnAttackCityEnd
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// IMPLEMENTED_OWNER: `COrganizingCtrl::on_attack_city_end` выше сохраняет
// virtual owner selection, side effects, union quirks и safe varargs-замену.
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1834
// RVA: 0x00037C70
// ADDRESS: 00437c70
// PROTOTYPE: void __thiscall OnAttackCityEnd(long param_1, long param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::CreateFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:651
// RVA: 0x000381A0
// ADDRESS: 004381a0
// PROTOTYPE: eCrOrgResult __thiscall CreateFaction(long param_1, long param_2, tagTime * param_3, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_4, uchar param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::DisbandFaction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:725
// RVA: 0x00038550
// ADDRESS: 00438550
// PROTOTYPE: bool __thiscall DisbandFaction(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::CreateConfederation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:867
// RVA: 0x000389D0
// ADDRESS: 004389d0
// PROTOTYPE: eCrOrgResult __thiscall CreateConfederation(long param_1, long param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_enum_eCrOrgResult___thiscall_COrganizingCtrl::CreateConfederation(long,long,std::basic_string<char,std::char_traits<char>,std::allocator<char>_>&)'::__l28::CreateUnion::OnAsyncCallback
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:953
// RVA: 0x00038D80
// ADDRESS: 00438d80
// PROTOTYPE: void __thiscall OnAsyncCallback(tagAsyncResult * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::DisbandConferation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1053
// RVA: 0x000393B0
// ADDRESS: 004393b0
// PROTOTYPE: bool __thiscall DisbandConferation(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::TransferIOwnerCity
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1105
// RVA: 0x00039890
// ADDRESS: 00439890
// PROTOTYPE: bool __thiscall TransferIOwnerCity(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_bool___thiscall_COrganizingCtrl::TransferIOwnerCity(long,long,long)'::__l42::PlayerTransferOwnerCity::OnAsyncCallback
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1238
// RVA: 0x0003A070
// ADDRESS: 0043a070
// PROTOTYPE: void __thiscall OnAsyncCallback(tagAsyncResult * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::OnDeleteRole
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1665
// RVA: 0x0003A2C0
// ADDRESS: 0043a2c0
// PROTOTYPE: int __thiscall OnDeleteRole(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::OnNewDay
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:2064
// RVA: 0x0003A490
// ADDRESS: 0043a490
// PROTOTYPE: void __stdcall OnNewDay(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// VERIFIED_DISASSEMBLY, IMPLEMENTED: полный `COrganizingCtrl::Run` RVA
// `0x0003A550` находится выше; temporary-map/STL traversal заменён typed
// callback к материализованному `DisbandFaction` owner-у.

// ============================================================================
// FUNCTION: COrganizingCtrl::OnPlayerInviteFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:790
// RVA: 0x0003A780
// ADDRESS: 0043a780
// PROTOTYPE: bool __thiscall OnPlayerInviteFaction(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::StatMemberNumBillboard
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1738
// RVA: 0x0003AC70
// ADDRESS: 0043ac70
// PROTOTYPE: void __thiscall StatMemberNumBillboard(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::StatOffenseVictorCountsBillboard
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1768
// RVA: 0x0003B050
// ADDRESS: 0043b050
// PROTOTYPE: void __thiscall StatOffenseVictorCountsBillboard(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::StatDefenceVictorCountsBillboard
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1801
// RVA: 0x0003B430
// ADDRESS: 0043b430
// PROTOTYPE: void __thiscall StatDefenceVictorCountsBillboard(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::StatBillboard
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1731
// RVA: 0x0003B810
// ADDRESS: 0043b810
// PROTOTYPE: void __thiscall StatBillboard(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::Initialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:106
// RVA: 0x0003B830
// ADDRESS: 0043b830
// PROTOTYPE: bool __thiscall Initialize(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00440d6d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp
// RVA: 0x00040D6D
// ADDRESS: 00440d6d
// PROTOTYPE: undefined Catch@00440d6d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00440e6d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp
// RVA: 0x00040E6D
// ADDRESS: 00440e6d
// PROTOTYPE: undefined Catch@00440e6d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00441181
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp
// RVA: 0x00041181
// ADDRESS: 00441181
// PROTOTYPE: undefined Catch@00441181()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044123b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp
// RVA: 0x0004123B
// ADDRESS: 0044123b
// PROTOTYPE: undefined Catch@0044123b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052e0b0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp
// RVA: 0x0012E0B0
// ADDRESS: 0052e0b0
// PROTOTYPE: undefined Unwind@0052e0b0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//






// COMPONENT_VARIANT_END: WorldServer
