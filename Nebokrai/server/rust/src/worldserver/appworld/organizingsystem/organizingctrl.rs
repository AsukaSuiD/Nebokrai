//! Владелец organizing-control исторического `WorldServer`.
//!
//! Статус `COrganizingCtrl::AddOneTopInfo` RVA `0x00036960`,
//! `SendTopInfoToClient` RVA `0x00033FC0` и
//! `SendAllTopInfoToInfoToOneClient` RVA `0x000352C0` — `IMPLEMENTED`;
//! локальные `CreateUnion::{CreateUnion,DoAsyncCall,Release}` RVA
//! `0x00034E90/0x00033680/0x00034EF0` — `IMPLEMENTED/VERIFIED_DISASSEMBLY`;
//! `CreateConfederation` RVA `0x000389D0` —
//! `IMPLEMENTED/VERIFIED_DISASSEMBLY`;
//! локальный `CreateUnion::OnAsyncCallback` RVA `0x00038D80` —
//! `IMPLEMENTED/VERIFIED_DISASSEMBLY`;
//! `OnPlayerInviteFaction` RVA `0x0003A780` —
//! `IMPLEMENTED/VERIFIED_DISASSEMBLY`;
//! `OnDeleteRole` RVA `0x0003A2C0` —
//! `IMPLEMENTED/VERIFIED_DISASSEMBLY`;
//! `CreateFaction` RVA `0x000381A0` —
//! `IMPLEMENTED/VERIFIED_DISASSEMBLY`;
//! публикация DB-staging `COrganizingCtrl::Initialize` RVA `0x0003B830` —
//! `IMPLEMENTED` для подтверждённой последовательности union/faction owner-ов;
//! `ReSetPermitDemise` RVA `0x00034810` и calendar-prefix
//! `Initialize/OnNewDay` RVA `0x0003B830/0x0003A490` — `IMPLEMENTED`;
//! полный `COrganizingCtrl::Run` RVA `0x0003A550` —
//! `IMPLEMENTED/VERIFIED_DISASSEMBLY`, `DisbandFaction` RVA `0x00038550` и
//! `UpdateOtherFacInfoToClient` RVA `0x00034980` и `DisbandConferation` RVA
//! `0x000393B0` — `IMPLEMENTED`;
//! `GenerateDBOrganizingID` RVA `0x000341C0`, `IsFreePlayer` RVA
//! `0x000343A0`, `IsFreeFaction` RVA `0x00034420`,
//! `SetAllCityFacEnemyChanged/ClearAllCityFacRelation/UpdateAllCityEneFacRelation`
//! RVA `0x00034240/0x000342C0/0x00034330`,
//! `RemovePersonFromApplyFactionList/GetFactionByPlayerInApplyList` RVA
//! `0x00034880/0x000348F0`,
//! `SetPlayerOrganizing` RVA `0x000370A0` и callback-цепочки
//! `OnPlayerEnterGame/OnPlayerExitGame` RVA `0x00037B70/0x00037BD0` —
//! `IMPLEMENTED/VERIFIED_DISASSEMBLY`; `GenerateSaveData` RVA `0x00034A10` —
//! `IMPLEMENTED`, `GetpFactionById` RVA `0x00034080` и
//! `GetConfederationOrganizing` RVA `0x00036BF0`,
//! `GetCountryByFaction` RVA `0x00037B20` и
//! `AddOwnedCityToFaction` RVA `0x00037C20`,
//! country-specific `GetFactionNumber` RVA `0x00033D10` и
//! `AddFactionListToByteArray` RVA `0x00033D90`,
//! `FindOrgaByName` RVA `0x000345A0`,
//! `AddAllFactinInfoToClientByPlayerID/AddFactionToClientByPlayerID/
//! AddUnionToClientByPlayerID` RVA `0x00034D00/0x00037380/0x000376F0`,
//! `IsFactionMaster` RVA `0x000344A0`, `IsConferationMaster` RVA `0x00034520`,
//! `ReInitialFacFactionByLvl` RVA
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
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:73,164,242,626,725,1105,1238,1352,1414,1641,1655,1910,1952,1960,1970,1998`.
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
//! Ingress `0x60110` повторяет `GetUnion(old master)` через
//! `IsFactionMaster -> IsFreeFaction -> GetConfederationOrganizing`, после
//! чего вызывает concrete `CUnion::Demise`. Rust временно извлекает только
//! найденный union из map-slot, чтобы его faction lookup/fan-out callbacks
//! видели остальной controller без raw aliasing, и обязательно возвращает
//! owner до typed result; дополнительных controller-gates не добавлено.
//! `GetCountryByFaction` сначала отбрасывает неположительный ID, затем делает
//! тот же faction-map lookup и только для ненулевого owner-а вызывает virtual
//! `GetCountry` slot `+0x190`. Exact ASM `0x00437B20..0x00437B6B`
//! подтверждает key-dataflow и нулевой возврат для всех lookup-gate-ов. Rust
//! отделяет этот нулевой miss от ещё не материализованного
//! `CFaction::m_Property`: первый есть `Ok(None)`, второй остаётся typed
//! safe-границей вместо выдуманной страны `0`.
//! `DisbandConferation` RVA `0x000393B0` сначала повторяет
//! standard/city war gates с `WS0246/WS0247`, очищает обе enemy-проекции и
//! вызывает concrete `CUnion::Disband`. Только его true-result добавляет union
//! ID в delete-очередь, сбрасывает union у owned cities master-faction,
//! удаляет owner из map и обновляет player-ов всех прежних member-фракций.
//! Rust временно detaches `CUnion`, чтобы сохранить reentrant faction-callback
//! без raw alias; владеющий `Box`/Drop заменяет map erase + delete.
//! Локальный `CreateUnion` хранит двух игроков, две фракции и имя будущего
//! союза в safe endpoint-е. Его `DoAsyncCall` отправляет второму игроку exact
//! `0x7FE16`: second player, first player, C-string имени, session ID и cookie;
//! ответ `AR_OK` с первым `long == 1` отделён от отказа и non-result terminal.
//! `Arc`, typed payload и автоматический `Drop` заменяют только множественное
//! наследование, raw `char *`, ручное выделение и virtual `Release` оригинала.
//! `CreateConfederation` проверяет reservation в порядке first/second, требует
//! оба ненулевых faction owner-а, читает их player header, затем проверяет у
//! первой faction `IsCreateUnionFun`. Отказ использует `WS0242/WS0193`, offline
//! второго мастера — `WS0243/WS0193`. Только online-ветвь получает process-wide
//! net-exchange ID, резервирует обе faction и начинает timeout `1000`.
//! Переданный аргумент имени EXE не читает: endpoint получает имя первой
//! faction. Rust на внутренней ошибке session setup снимает обе reservation
//! вместо исходного null dereference; внешнего Miracle-контракта у UB нет.
//! Creation callback различает approve, явный deny (`WS0245/WS0193`) и прочий
//! terminal. Approve создаёт union с новым organizing ID, выполняет `Initial`,
//! рассылает первой faction `WS0244/WS0119` с decimal color `10000`, добавляет
//! вторую faction и вставляет owner в union-map. Затем машина повторно ставит
//! superior обеим faction, обновляет их города и двух master-player-ов,
//! условно отправляет им `0x7FE04` и ещё раз обновляет города первой faction;
//! этот порядок и дублирующие side effects сохранены. Exact EXE
//! `0x0043930C..0x00439388` исправляет ошибочный RAW: callback всегда проходит
//! cleanup first, затем second reservation, без раннего возврата между ними.
//! `OnPlayerInviteFaction` сначала разрешает первую faction только через
//! `IsFactionMaster(player)`, затем проверяет обе concrete faction. War-gates
//! идут строго: first standard/village `WS0237`, first city-only `WS0238`,
//! invited standard/village `WS0239`, invited city/AttackCity `WS0240`; общий
//! второй текст этих четырёх отказов — `WS0121`. После двух `IsFreeFaction`
//! ветви literal: две свободные — `CreateConfederation`, свободная первая —
//! `ApplyForJoin` союза второй, свободная вторая — `Invite` союза первой,
//! два союза — `WS0241/WS0193`. Exact ASM `0x0043A780..0x0043AC6C`
//! подтверждает отсутствие source AttackCity gate и true-return после любого
//! вызванного owner-а независимо от его bool/result.
//! `CreateFaction` проверяет membership, затем mutable invalid-string filter и
//! пять player-name index-ов строго до `FindOrgaByName`. После успеха удаляет
//! все faction applications, генерирует ID, строит exact `CFaction::Initial`,
//! назначает country, вставляет owner, ставит dirty `0x0F`, публикует Add всем
//! faction и только затем пишет optional structured create-log. Exact ASM
//! `0x004381A0..0x00438549` подтверждает порядок и результаты `Fail/NameExist/
//! Ok`. Странная country-проверка в прологе машинно является тавтологией и не
//! ограничивает ни одно `u8`; Rust поэтому не добавляет собственного range
//! gate. SQL-formatting/heap-buffer остаются за библиотечным log adapter-ом.
//! `AddOwnedCityToFaction` повторяет те же positive-ID/map/null gates и затем
//! вызывает virtual `AddOwnedCity` slot `+0x80`. Exact ASM
//! `0x00437C20..0x00437C69` подтверждает порядок обоих аргументов и отсутствие
//! иных эффектов controller-а. Concrete `CFaction::add_owned_city` уже
//! сохраняет list/duplicate/wire/player семантику; controller возвращает её
//! typed report только для Rust caller-а, не добавляя legacy return value.
//! Country-specific faction list проходит map в signed order, пропускает null
//! owners и сравнивает exact virtual country byte. Страница нормализует только
//! значения `<1`, сохраняет 32-битную wrapping формулу `page*11-11`, пишет
//! signed count и пары `faction ID + name\0`. Exact ASM
//! `0x00433D10..0x00433D8D/0x00433D90..0x00433F31` подтверждает повторный
//! count-вызов на короткой последней странице и increment только для faction
//! выбранной страны. Небезопасный `strcpy` в `char[256]` заменён локальной
//! typed safe-границей только для имени, которое вместе с NUL не помещается;
//! уже записанные count/ID/предыдущие записи сохраняются в progress.
//! `FindOrgaByName` копирует visible C-string в две 260-байтовые границы,
//! применяет linked CRT `_strlwr` и сначала проходит faction-map, затем
//! union-map в signed order. Exact ASM `0x004345A0..0x00434801`, связанный
//! `_strlwr` `0x0051F25E..0x0051F371` и отсутствие project-call к `_setlocale`
//! подтверждают ASCII-only folding: high bytes сравниваются byte-exact. Rust
//! не хранит внутренний `szfacNewName`, который читался только этим owner-ом;
//! `BTreeMap` и `eq_ignore_ascii_case` заменяют tree/CRT storage. Переполнение
//! исходного `strcpy/_snprintf` и null map-value остаются typed safe-границей.
//! Для найденной `0x60108` faction её `ApplyForJoin` временно получает
//! detached concrete owner и reentrant adapter полного controller-map:
//! membership и удаление прежних заявок по-прежнему идут в signed key-order,
//! а target slot в своей точной позиции заменяется живой detached faction.
//! Union-ветвь так же временно извлекает target owner, заранее фиксирует
//! результат `IsFreeFaction(applicant)` и обязательно возвращает slot после
//! concrete вызова. Это safe ownership-замена raw-pointer aliasing; машинный
//! `CUnion::GetCountry` `0x004C1F10..0x004C1F12` возвращает literal `0`, что
//! намеренно важнее противоречащего Linux-донора.
//! `AddFactionToClientByPlayerID` повторно разрешает faction игрока, требует
//! online owner, строит `0x7FE02(player, full faction snapshot)`, явно делает
//! `Update`, отправляет по текущему GameServer route и только затем ставит
//! `m_bGetFactionData=true`, игнорируя send-result. Exact
//! `0x00437380..0x004374AB` подтверждает оба false-gate и этот порядок.
//! Misspelled `AddAllFactinInfoToClientByPlayerID` exact
//! `0x00434D00..0x00434E8C` после тех же membership/online gates строит
//! `0x7FE08(player, map size, ID/country/name всех faction)` в signed order и
//! отправляет его, не меняя player flag. Detached adapter `DoJoin` подставляет
//! живую target faction в обоих проходах на прежний map-key; `Cell<bool>` у
//! player-а заменяет только raw aliasing, сохраняя момент мутации флага.
//! `AddUnionToClientByPlayerID` последовательно разрешает faction и union,
//! требует online player только после live union lookup, отправляет ему
//! `0x7FE04(player, full union snapshot)` даже на route `0` и лишь после send
//! ставит `m_bGetFactionData=true`. Missing union после положительного
//! membership возвращает исходный `true`, offline player — `false`; exact
//! `0x004376F0..0x00437807` подтверждает эти разные return-path.
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
//! `OnDeleteRole` сначала принимает доказанный результат внешнего DB/country
//! префикса, затем ищет faction через `IsFreePlayer`. Job-level `<1` вызывает
//! concrete `CFaction::DelMember` и возвращает `0`. Для job-level `>=1`
//! exact virtual slot `+0x90` возвращает `GetMembers`; проверка `_Mysize > 0`
//! после успешного membership lookup штатно даёт код `1`, так что донорское
//! условие `GetMemberNum() > 1` неверно, а достигнутый `DisbandFaction` path
//! логически недостижим. Положительный union ID требует живой union-owner,
//! после чего concrete `CUnion::DelMember` отвязывает faction и всегда даёт
//! код `3`. Safe Rust останавливает только неполную faction/property проекцию,
//! соответствующую старому null/invalid-state UB.
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
//! `IsConferationMaster` симметрично проходит union-map и возвращает первый
//! положительный ID concrete union, чья master-faction равна входному ID. Null union-value
//! остаётся typed safe-границей старого virtual разыменования. Folded exact
//! `CUnion::IsMaster` `0x004C1EE0..0x004C1EF3` подтверждает сравнение поля
//! master-faction и возврат union ID.
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
//! `CPlayer::AddOwnedRegion` exact `0x0045DD10..0x0045DD75` подтверждает
//! unique-by-region-ID и append-order, но копирует в wire два
//! неинициализированных stack-padding байта. GameServer поглощает полные
//! восемь байт и использует только ID/type, поэтому safe Rust сохраняет ABI-
//! ширину и обнуляет этот внутренний stack leak. Owned city больше не
//! блокирует достигнутый `SetPlayerOrganizing`.
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
//! `DisbandFaction(master, faction)`. Внутренний fallible dispatch нужен только
//! для передачи Game/context без самозаимствования controller-а; MainLoop
//! подключает к нему concrete `disband_faction`, а не внешний owner-callback.
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
//! Rust завершает player/log continuation в `CGame` сразу после возврата core:
//! имя faction сохраняется до Drop удалённого owner-а, поэтому observable
//! порядок остаётся тем же без alias между immutable Game и mutable player.
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

use crate::dbaccess::worlddb::rsfaction::{
    FactionLoadOutcome, FactionPropertyLoadStaging,
};
use crate::dbaccess::worlddb::rsunion::{UnionDatabaseLoadRecord, UnionLoadOutcome};
use super::attackcitysys::CAttackCitySys;
use super::faction::{
    CFaction, CityWarEnemyRefreshOutcome, FactionCloneSaveBlock, FactionContributorBlock,
    FactionApplyForJoinBlock, FactionApplyForJoinContext, FactionApplyForJoinEffects,
    FactionApplyForJoinOutcome, FactionContributorContext, FactionDoJoinBlock,
    FactionDoJoinContext, FactionDoJoinEffects, FactionDoJoinOutcome,
    FactionContributorOutcome, FactionDeleteOrganizingBuildError,
    FactionDelMemberBlock, FactionDelMemberReport,
    FactionDeleteOrganizingOutcome, FactionDisbandBlock, FactionDisbandContext,
    FactionDisbandOutcome, FactionDisbandProgress, FactionDisbandRejection,
    FactionExperienceBlock, FactionExperienceUpdate,
    FactionEditLeaveWordOutcome, FactionEnemyDelivery, FactionEnemyMutationBlock,
    FactionEnemyMutationContext, FactionEnemyWarLogArgument, FactionFeatureFunctionUpdate,
    FactionFullSnapshotBlock, FactionInitialBlock, FactionInitialPropertyBlock,
    FactionLeaveWordBlock, FactionLeaveWordOutcome, FactionMemberInfoReport,
    FactionMemberInfoRequest,
    FactionOperationAuthorityContext, FactionOperationBlock, FactionOperationOutcome,
    FactionOrganizingInfoContext, FactionOtherInfoBuildError,
    FactionOtherInfoDelivery, FactionOwnedCityDelivery, FactionOwnedCityRefreshBlock,
    FactionOwnedCityRefreshReport, FactionOwnedCityUpdateBuildError, FactionPlayerHeaderContext,
    FactionPlayerHeaderBlock,
    FactionPermitBlock, FactionPermitUpdate,
    FactionPronounceBlock, FactionPronounceOutcome, FactionPropertyDelivery,
    FactionPropertyReinitialization, FactionRemoveApplyMemberOutcome, FactionSuperiorOrganizingBlock,
    FactionSetParameterBlock, FactionSetParameterContext, FactionSetParameterOutcome,
    FactionUpgradeBlock, FactionUpgradeContext, FactionUpgradeOutcome, FactionUploadIconBlock,
    FactionUploadIconContext, FactionUploadIconOutcome,
    MemberEnterOutcome, MemberExitOutcome, MemberLevelChangeOutcome,
    MemberPositionChangeOutcome, OwnedCityAddOutcome, OwnedCityBooleanMutationReport,
    OwnedCityMutationBuildError,
};
use super::factionwarsys::{
    CFactionWarSys, FactionWarDeclarationContext, FactionWarFactionSnapshot,
    FactionWarFormatArgument, FactionWarPlayerDiedContext,
};
use super::organizing::{ECityState, EOperator, TagTimeValue};
use super::organizingparam::COrganizingParam;
use super::union::{
    CUnion, UnionAddFactionBlock, UnionAddFactionEffects, UnionAddFactionOutcome,
    UnionApplicationFactionBlock,
    UnionApplicationFactionSnapshot, UnionApplicationTerminal, UnionApplyForJoinBlock,
    UnionApplyForJoinContext, UnionApplyForJoinEffects, UnionApplyForJoinOutcome,
    UnionClientSnapshotContext, UnionDoJoinBlock, UnionDoJoinContext, UnionDoJoinOutcome,
    UnionDemiseBlock, UnionDemiseOutcome, UnionDisbandBlock, UnionDisbandOutcome,
    UnionExitBlock, UnionExitContext,
    UnionExitOutcome,
    UnionFireOutBlock, UnionFireOutContext, UnionFireOutEffects, UnionFireOutOutcome,
    UnionFactionFanoutReport, UnionFactionJoinContext, UnionFactionLevelBlock,
    UnionFactionMemberContext,
    UnionFactionStateMutationContext, UnionInitialBlock, UnionInitialMutationContext,
    UnionInitialReport, UnionMasterFactionQueryContext,
    UnionInviteBlock, UnionInviteEffects, UnionInviteOutcome,
    UnionMemberSnapshotBlock, UnionOperatorValidationContext, UnionOwnedCityBooleanMutationReport,
    UnionOwnedCityFanoutReport, UnionOwnedCityMutationBlock, UnionOwnedCityMutationContext,
    UnionFormatArgument, UnionPlayerRefreshContext, UnionPlayerRefreshReport,
    UnionSendInfoContext,
    UnionVictorFanoutReport, UnionVictorMutationBlock,
};
use super::villagewarsys::CVillageWarSys;
use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::public::date::{TagTime, TagTimeArithmeticBlock};
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
use crate::worldserver::worldserver::game::{
    CGame, WorldPlayerNameLookupError, WorldRegionNameLookup,
};
use crate::public::timer::{CTimer, TimerId};

const TOP_INFO_MESSAGE_TYPE: i32 = 0x7FA04;
const UNION_INITIAL_MESSAGE_TYPE: i32 = 0x7FE04;
const EXPIRING_TIMER_FLAG: i32 = 2;
const DECLARE_WAR_FACTION_PAGE_SIZE: i32 = 11;
const DEFAULT_FACTION_BILLBOARD_SIZE: i32 = 10;
const CITY_TRANSFER_CONFIRMATION_MESSAGE_TYPE: i32 = 0x7FE2B;
const CONFEDERATION_CREATION_CONFIRMATION_MESSAGE_TYPE: i32 = 0x7FE16;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ConfederationMasterLookupBlock {
    pub(crate) map_key: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionWarPlayerDiedBlock {
    MasterLookup(FactionMasterLookupBlock),
    UnionMembership { map_key: i32 },
    UnionMaster(ConfederationMasterLookupBlock),
    PlayerMembership { map_key: i32 },
    MissingFactionInUnion { union_id: i32, faction_id: i32 },
    MissingFactionForMutation { faction_id: i32 },
    EnemyMutation {
        faction_id: i32,
        enemy_id: i32,
        source: FactionEnemyMutationBlock,
    },
}

/// Concrete adapter общих controller/player/string/transport owner-ов войны.
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

impl FactionWarPlayerDiedContext for WorldFactionWarDeclarationEffects<'_> {
    type Block = OrganizingFactionWarPlayerDiedBlock;

    fn faction_id_by_master_player(&self, player_id: i32) -> Result<i32, Self::Block> {
        self.organizing
            .faction_id_by_master_player(player_id)
            .map_err(OrganizingFactionWarPlayerDiedBlock::MasterLookup)
    }

    fn union_id_by_faction(&self, faction_id: i32) -> Result<i32, Self::Block> {
        match self.organizing.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => Ok(0),
            FreeFactionLookup::Union(union_id) => Ok(union_id),
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                Err(OrganizingFactionWarPlayerDiedBlock::UnionMembership { map_key })
            }
        }
    }

    fn union_id_by_master_faction(&self, faction_id: i32) -> Result<i32, Self::Block> {
        self.organizing
            .union_id_by_master_faction(faction_id)
            .map_err(OrganizingFactionWarPlayerDiedBlock::UnionMaster)
    }

    fn faction_id_by_player(&self, player_id: i32) -> Result<i32, Self::Block> {
        match self.organizing.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => Ok(0),
            FreePlayerLookup::Faction(faction_id) => Ok(faction_id),
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                Err(OrganizingFactionWarPlayerDiedBlock::PlayerMembership { map_key })
            }
        }
    }

    fn faction_exists(&self, faction_id: i32) -> bool {
        self.organizing.faction_by_id(faction_id).is_some()
    }

    fn faction_side(&self, root_faction_id: i32) -> Result<Vec<i32>, Self::Block> {
        match self.organizing.is_free_faction(root_faction_id) {
            FreeFactionLookup::NoUnion => Ok(vec![root_faction_id]),
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                Err(OrganizingFactionWarPlayerDiedBlock::UnionMembership { map_key })
            }
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
                        OrganizingFactionWarPlayerDiedBlock::MissingFactionInUnion {
                            union_id,
                            faction_id,
                        },
                    );
                }
                Ok(members)
            }
        }
    }

    fn del_enemy_organizing(
        &mut self,
        faction_id: i32,
        enemy_id: i32,
    ) -> Result<(), Self::Block> {
        let enemy_name = self
            .organizing
            .faction_by_id(enemy_id)
            .map(|faction| faction.name().to_vec())
            .ok_or(
                OrganizingFactionWarPlayerDiedBlock::MissingFactionForMutation {
                    faction_id: enemy_id,
                },
            )?;
        let faction = self.organizing.faction_by_id_mut(faction_id).ok_or(
            OrganizingFactionWarPlayerDiedBlock::MissingFactionForMutation { faction_id },
        )?;
        let mut effects = DeclarationEnemyMutationEffects {
            enemy_id,
            enemy_name,
            format_world_string: &mut *self.format_world_string,
            put_war_log: &mut *self.put_war_log,
        };
        faction
            .del_enemy_organizing(enemy_id, &mut effects)
            .map(|_| ())
            .map_err(|source| OrganizingFactionWarPlayerDiedBlock::EnemyMutation {
                faction_id,
                enemy_id,
                source,
            })
    }

    fn update_enemy_faction(&mut self, faction_id: i32) -> Result<(), Self::Block> {
        let faction = self.organizing.faction_by_id_mut(faction_id).ok_or(
            OrganizingFactionWarPlayerDiedBlock::MissingFactionForMutation { faction_id },
        )?;
        let _ = faction.update_enemy_faction(self.game, &mut *self.update_player);
        Ok(())
    }

    fn organizing_name(&self, faction_id: i32) -> Result<Vec<u8>, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .map(|faction| faction.name().to_vec())
            .ok_or(OrganizingFactionWarPlayerDiedBlock::MissingFactionForMutation {
                faction_id,
            })
    }

    fn format_world_string(&mut self, string_id: &[u8], arguments: &[&[u8]]) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| UnionFormatArgument::Text(argument))
            .collect::<Vec<_>>();
        (self.format_world_string)(string_id, &arguments)
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

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionClientSnapshotByPlayerOutcome {
    NoFaction,
    NoUnion { faction_id: i32 },
    MissingUnion { faction_id: i32, union_id: i32 },
    PlayerOffline { faction_id: i32, union_id: i32 },
    Sent {
        faction_id: i32,
        union_id: i32,
        game_server_id: i32,
        result: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UnionClientSnapshotByPlayerBlock {
    PlayerMembership { map_key: i32 },
    UnionMembership { map_key: i32 },
    Snapshot {
        faction_id: i32,
        union_id: i32,
        source: UnionMemberSnapshotBlock,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionCountryCountBlock {
    pub(crate) map_key: i32,
    pub(crate) matched_before_block: i32,
    pub(crate) source: FactionInitialPropertyBlock,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionListEntry {
    pub(crate) faction_id: i32,
    pub(crate) name: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionListPage {
    pub(crate) requested_page: i32,
    pub(crate) normalized_page: i32,
    pub(crate) country: u8,
    pub(crate) start_index: i32,
    pub(crate) total_factions: i32,
    pub(crate) entry_count: i32,
    pub(crate) entries: Vec<FactionListEntry>,
    pub(crate) payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionListCountPhase {
    Initial,
    LastPageRecount,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionListPageBlock {
    Count {
        requested_page: i32,
        normalized_page: i32,
        country: u8,
        phase: FactionListCountPhase,
        source: FactionCountryCountBlock,
    },
    Country {
        page: FactionListPage,
        map_key: i32,
        source: FactionInitialPropertyBlock,
    },
    NameWouldOverflow {
        page: FactionListPage,
        map_key: i32,
        required_bytes_with_nul: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingNameKind {
    Faction,
    Union,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrganizingNameMatch {
    pub(crate) kind: OrganizingNameKind,
    pub(crate) map_key: i32,
    pub(crate) organizing_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingNameLookupBlock {
    RequestedNameWouldOverflow {
        visible_len: usize,
    },
    NullOwner {
        kind: OrganizingNameKind,
        map_key: i32,
    },
    OwnerNameWouldOverflow {
        kind: OrganizingNameKind,
        map_key: i32,
        visible_len: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingNameCountryBlock {
    TargetMissing {
        kind: OrganizingNameKind,
        map_key: i32,
    },
    MissingFactionProperty {
        map_key: i32,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingNamedUnionApplicationBlock<SessionBlock> {
    TargetMissing {
        map_key: i32,
    },
    Apply {
        map_key: i32,
        union_id: i32,
        source: UnionApplyForJoinBlock<FactionUnionMembershipLookupBlock, SessionBlock>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum DetachedFactionApplicationContextBlock {
    MembershipNullFaction {
        map_key: i32,
    },
    RemovalNullFaction {
        map_key: i32,
        completed_removals: Vec<ApplyFactionRemoval>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionApplicationBlock {
    TargetMissing {
        map_key: i32,
    },
    Apply {
        map_key: i32,
        source: FactionApplyForJoinBlock<DetachedFactionApplicationContextBlock>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionClientSnapshotBlock {
    MembershipNullFaction {
        map_key: i32,
    },
    FullSnapshot {
        faction_id: i32,
        source: FactionFullSnapshotBlock,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AllFactionInfoClientBlock {
    MembershipNullFaction {
        map_key: i32,
    },
    NullFaction {
        map_key: i32,
        completed_factions: usize,
    },
    MissingCountry {
        map_key: i32,
        faction_id: i32,
        completed_factions: usize,
    },
    NameWouldOverflow {
        map_key: i32,
        faction_id: i32,
        visible_len: usize,
        completed_factions: usize,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum DetachedFactionDoJoinContextBlock {
    RemovalNullFaction {
        map_key: i32,
        completed_removals: Vec<ApplyFactionRemoval>,
    },
    MembershipNullFaction {
        map_key: i32,
    },
    FactionSnapshot(FactionClientSnapshotBlock),
    AllFactionInfo(AllFactionInfoClientBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionDoJoinOutcome {
    FactionNotFound {
        faction_id: i32,
    },
    Applied {
        faction_id: i32,
        join_time: TagTimeValue,
        outcome: FactionDoJoinOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingFactionDoJoinBlock {
    ManagerMembership {
        map_key: i32,
    },
    DoJoin {
        faction_id: i32,
        source: FactionDoJoinBlock<DetachedFactionDoJoinContextBlock>,
    },
}

/// Safe-границы точной цепочки `GetUnion(player ID)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingUnionByMasterBlock {
    FactionMaster(FactionMasterLookupBlock),
    UnionMembership(FactionUnionMembershipLookupBlock),
}

/// Safe-границы exact `IsFreePlayer -> IsFreeFaction` lookup для `0x6010E`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingUnionByPlayerBlock {
    PlayerMembership { map_key: i32 },
    UnionMembership { map_key: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingUnionFireOutOutcome {
    UnionNotFound,
    Applied {
        union_id: i32,
        outcome: UnionFireOutOutcome<UnionMemberDetachOutcome>,
        automatic_disband: Option<OrganizingConfederationDisbandOutcome>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingUnionFireOutBlock {
    Lookup(OrganizingUnionByMasterBlock),
    FireOut {
        union_id: i32,
        source: UnionFireOutBlock<FactionMasterLookupBlock, UnionMemberDetachBlock>,
    },
    AutomaticDisband {
        union_id: i32,
        fire_out: UnionFireOutOutcome<UnionMemberDetachOutcome>,
        source: OrganizingConfederationDisbandBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingUnionDemiseOutcome {
    UnionNotFound,
    Applied {
        union_id: i32,
        outcome: UnionDemiseOutcome,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingUnionDemiseBlock {
    Lookup(OrganizingUnionByMasterBlock),
    Demise {
        union_id: i32,
        source: UnionDemiseBlock<FactionMasterLookupBlock>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingUnionExitOutcome {
    UnionNotFound,
    Applied {
        union_id: i32,
        outcome: UnionExitOutcome<UnionMemberDetachOutcome>,
        automatic_disband: Option<OrganizingConfederationDisbandOutcome>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingUnionExitBlock {
    Lookup(OrganizingUnionByPlayerBlock),
    Exit {
        union_id: i32,
        source: UnionExitBlock<FactionMasterLookupBlock, UnionMemberDetachBlock>,
    },
    AutomaticPlayerHeader {
        union_id: i32,
        exit: UnionExitOutcome<UnionMemberDetachOutcome>,
        source: UnionPlayerHeaderLookupBlock,
    },
    AutomaticDisband {
        union_id: i32,
        exit: UnionExitOutcome<UnionMemberDetachOutcome>,
        player_header: i32,
        source: OrganizingConfederationDisbandBlock,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingConfederationDisbandRejection {
    StandardWar,
    CityWar,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingConfederationDisbandReport {
    pub(crate) union_id: i32,
    pub(crate) standard_enemy_clear: UnionFactionFanoutReport,
    pub(crate) city_enemy_clear: UnionFactionFanoutReport,
    pub(crate) union_outcome: UnionDisbandOutcome<UnionMemberDetachOutcome>,
    pub(crate) delete_queued: bool,
    pub(crate) owned_city_refreshes: Vec<(i32, i32, i32)>,
    pub(crate) player_refresh: Option<UnionPlayerRefreshReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingConfederationDisbandOutcome {
    UnionNotFound,
    Rejected(OrganizingConfederationDisbandRejection),
    Applied(OrganizingConfederationDisbandReport),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingConfederationDisbandBlock {
    Disband {
        union_id: i32,
        source: UnionDisbandBlock<
            FactionMasterLookupBlock,
            UnionMemberDetachBlock,
            UnionMemberDetachOutcome,
        >,
    },
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

pub(crate) type OrganizingUnionInviteBlock<SessionBlock> = UnionInviteBlock<
    FactionMasterLookupBlock,
    FactionUnionMembershipLookupBlock,
    SessionBlock,
>;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingUnionInviteOutcome<SessionReport> {
    UnionNotFound,
    Applied {
        union_id: i32,
        outcome: UnionInviteOutcome<SessionReport>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingUnionInviteDispatchBlock<SessionBlock> {
    TargetMissing { union_id: i32 },
    Invite {
        union_id: i32,
        source: OrganizingUnionInviteBlock<SessionBlock>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OrganizingUnionInvitationCallbackReport {
    pub(crate) union_id: i32,
    pub(crate) inviter_faction_id: i32,
    pub(crate) invited_faction_id: i32,
    pub(crate) union_found: bool,
    pub(crate) rejection_notice_sent: bool,
    pub(crate) join: Option<UnionDoJoinOutcome>,
    pub(crate) application_cleared: bool,
    pub(crate) establishment_reservation_removed: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingUnionInvitationCallbackBlock {
    InviterPlayerHeader {
        union_id: i32,
        inviter_faction_id: i32,
        invited_faction_id: i32,
        source: FactionPlayerHeaderBlock<UnionPlayerHeaderLookupBlock>,
        rejection_notice_sent: bool,
    },
    DoJoin {
        union_id: i32,
        inviter_faction_id: i32,
        invited_faction_id: i32,
        source: OrganizingUnionApplicationJoinBlock,
        rejection_notice_sent: bool,
    },
}

pub(crate) trait PlayerInviteFactionEffects:
    ConfederationCreationEffects + UnionApplyForJoinEffects + UnionInviteEffects
{
}

impl<T> PlayerInviteFactionEffects for T where
    T: ConfederationCreationEffects + UnionApplyForJoinEffects + UnionInviteEffects
{
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerInviteFactionRejection {
    MasterFactionNotFound,
    InviterFactionMissing,
    InvitedFactionMissing,
    InviterStandardWar { notice_sent: bool },
    InviterCityWar { notice_sent: bool },
    InvitedStandardWar { notice_sent: bool },
    InvitedCityWar { notice_sent: bool },
    BothAlreadyInUnion { notice_sent: bool },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum PlayerInviteFactionAction<CreationReport, ApplicationReport, InvitationReport> {
    Create(ConfederationCreationStartOutcome<CreationReport>),
    ApplyToInvitedUnion(UnionApplyForJoinOutcome<ApplicationReport>),
    InviteToInviterUnion(OrganizingUnionInviteOutcome<InvitationReport>),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum PlayerInviteFactionOutcome<CreationReport, ApplicationReport, InvitationReport> {
    Rejected(PlayerInviteFactionRejection),
    Dispatched {
        inviter_faction_id: i32,
        invited_faction_id: i32,
        action: PlayerInviteFactionAction<CreationReport, ApplicationReport, InvitationReport>,
    },
}

#[derive(Debug)]
pub(crate) enum PlayerInviteFactionBlock<CreationBlock, ApplicationBlock, InvitationBlock> {
    Master(FactionMasterLookupBlock),
    InviterMembership { map_key: i32 },
    InvitedMembership { map_key: i32 },
    Creation(ConfederationCreationStartBlock<CreationBlock>),
    Application(OrganizingNamedUnionApplicationBlock<ApplicationBlock>),
    Invitation(OrganizingUnionInviteDispatchBlock<InvitationBlock>),
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
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingRunBlock {
    NullFaction { map_key: i32 },
    DeleteRemainTimeAbsent { map_key: i32 },
    MasterIdAbsent { map_key: i32, faction_id: i32 },
    Disband {
        faction_id: i32,
        master_id: i32,
        source: OrganizingDisbandBlock,
    },
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

/// Наблюдаемый результат `COrganizingCtrl::OnDeleteRole` до wire-ответа
/// LoginServer. Числа совпадают с exact switch `0..=4`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDeleteRoleOutcome {
    AllowedNoFaction,
    CountryJob,
    MemberRemoved {
        faction_id: i32,
        removal: Option<FactionDelMemberReport>,
    },
    FactionMaster {
        faction_id: i32,
    },
    UnionMissing {
        faction_id: i32,
        union_id: i32,
    },
    UnionDetached {
        faction_id: i32,
        union_id: i32,
        detach: UnionMemberDetachOutcome,
    },
}

impl OrganizingDeleteRoleOutcome {
    pub(crate) const fn legacy_code(&self) -> i32 {
        match self {
            Self::AllowedNoFaction | Self::MemberRemoved { .. } | Self::UnionMissing { .. } => 0,
            Self::FactionMaster { .. } => 1,
            Self::UnionDetached { .. } => 3,
            Self::CountryJob => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDeleteRoleBlock {
    NullFactionDuringMembershipScan { map_key: i32 },
    MemberRemoval {
        faction_id: i32,
        source: FactionDelMemberBlock,
    },
    MissingFactionProperty { faction_id: i32 },
    UnionDetach(UnionMemberDetachBlock),
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

pub(crate) enum OrganizingDisbandOutcome {
    Rejected {
        reason: OrganizingDisbandRejection,
        notice_sent: bool,
        cleared_city_war_enemies: usize,
    },
    Disbanded {
        progress: OrganizingDisbandProgress,
        retired_faction: Box<CFaction>,
    },
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

/// Поля exact `CreateUnion::DoAsyncCall` и его terminal owner-а.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ConfederationCreationSessionRequest {
    pub(crate) first_player_id: i32,
    pub(crate) second_player_id: i32,
    pub(crate) first_faction_id: i32,
    pub(crate) second_faction_id: i32,
    pub(crate) requested_session_id: i32,
    pub(crate) timeout_ticks: u32,
    pub(crate) union_name: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConfederationCreationTerminal {
    Approved,
    Denied,
    NonResult { kind: NetSessionAsyncResultKind },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConfederationCreationEndpointBlock {
    BeginPayloadType,
    ResultPayloadType,
}

/// Потокобезопасная граница callback-а и единственного organizing owner-а.
pub(crate) trait ConfederationCreationSessionRuntime: Send + Sync {
    fn send_confederation_creation_confirmation(
        &self,
        recipient_player_id: i32,
        message: &CMessage,
    );

    fn finish_confederation_creation(
        &self,
        first_player_id: i32,
        second_player_id: i32,
        first_faction_id: i32,
        second_faction_id: i32,
        union_name: &[u8],
        terminal: ConfederationCreationTerminal,
    );

    fn block_confederation_creation_endpoint(&self, block: ConfederationCreationEndpointBlock);
}

/// Safe owner локального `CreateUnion` вместо двух C++ subobject-ов.
pub(crate) struct CreateConfederationEndpoint {
    first_player_id: i32,
    second_player_id: i32,
    first_faction_id: i32,
    second_faction_id: i32,
    union_name: Vec<u8>,
    runtime: Arc<dyn ConfederationCreationSessionRuntime>,
}

impl NetSessionEndpoint for CreateConfederationEndpoint {
    fn do_async_call(&self, session_id: i64, cookie_second: i32, payload: &dyn Any) {
        let Some(request) = payload.downcast_ref::<ConfederationCreationSessionRequest>() else {
            self.runtime.block_confederation_creation_endpoint(
                ConfederationCreationEndpointBlock::BeginPayloadType,
            );
            return;
        };

        let mut message = CMessage::new(CONFEDERATION_CREATION_CONFIRMATION_MESSAGE_TYPE);
        message.base_mut().add_long(request.second_player_id);
        message.base_mut().add_long(request.first_player_id);
        message
            .base_mut()
            .add(legacy_c_string_prefix(&request.union_name));
        message.base_mut().add_byte(0);
        message.base_mut().add_long64(session_id);
        message.base_mut().add_long(cookie_second);
        self.runtime.send_confederation_creation_confirmation(
            request.second_player_id,
            &message,
        );
    }

    fn on_async_callback(&self, result: NetSessionAsyncResult<'_>) {
        let terminal = if result.kind == NetSessionAsyncResultKind::Result {
            let Some(decision) = result
                .payload
                .and_then(|payload| payload.downcast_ref::<i32>())
            else {
                self.runtime.block_confederation_creation_endpoint(
                    ConfederationCreationEndpointBlock::ResultPayloadType,
                );
                return;
            };
            if *decision == 1 {
                ConfederationCreationTerminal::Approved
            } else {
                ConfederationCreationTerminal::Denied
            }
        } else {
            ConfederationCreationTerminal::NonResult { kind: result.kind }
        };
        self.runtime.finish_confederation_creation(
            self.first_player_id,
            self.second_player_id,
            self.first_faction_id,
            self.second_faction_id,
            &self.union_name,
            terminal,
        );
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ConfederationCreationSessionReport {
    pub(crate) session: CreatedNetSession,
}

pub(crate) enum ConfederationCreationSessionBlock {
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

impl std::fmt::Debug for ConfederationCreationSessionBlock {
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

/// Связывает exact creation request с готовым session manager в исходном order.
pub(crate) fn begin_confederation_creation_session(
    manager: &CNetSessionManager,
    request: ConfederationCreationSessionRequest,
    runtime: Arc<dyn ConfederationCreationSessionRuntime>,
    random: impl FnMut(i32) -> i32,
) -> Result<ConfederationCreationSessionReport, ConfederationCreationSessionBlock> {
    let session = manager
        .create_session(
            request.second_player_id,
            request.requested_session_id,
            random,
        )
        .map_err(ConfederationCreationSessionBlock::Create)?;
    let endpoint = Box::new(CreateConfederationEndpoint {
        first_player_id: request.first_player_id,
        second_player_id: request.second_player_id,
        first_faction_id: request.first_faction_id,
        second_faction_id: request.second_faction_id,
        union_name: request.union_name.clone(),
        runtime,
    });
    manager
        .set_callback_handle(session.id, endpoint)
        .map_err(|source| ConfederationCreationSessionBlock::SetCallback { session, source })?;
    manager
        .beging(session.id, request.timeout_ticks, &request)
        .map_err(|source| ConfederationCreationSessionBlock::Begin { session, source })?;
    Ok(ConfederationCreationSessionReport { session })
}

/// Внешние name-index, localization и structured-log границы CreateFaction.
pub(crate) trait FactionCreationEffects {
    fn check_invalid_organizing_string(&mut self, name: &mut Vec<u8>, strict: bool) -> bool;
    fn persistent_player_name_exists(&mut self, name: &[u8]) -> bool;
    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8>;
    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>);
    fn faction_create_log_enabled(&self) -> bool;
    fn write_faction_create_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        player_id: i32,
        player_name: &[u8],
    );
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionCreationRejection {
    PlayerAlreadyInFaction,
    InvalidName { notice_sent: bool },
    NameExists,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionCreationReport {
    pub(crate) faction_id: i32,
    pub(crate) application_removals: Vec<ApplyFactionRemoval>,
    pub(crate) other_faction_updates: Vec<OrganizingOtherFactionUpdate>,
    pub(crate) log_written: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionCreationOutcome {
    Rejected(FactionCreationRejection),
    Created(FactionCreationReport),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionCreationPreparation {
    Rejected(FactionCreationRejection),
    ReadyForPersistentLookup,
}

#[derive(Debug)]
pub(crate) enum FactionCreationBlock {
    PlayerMembership { map_key: i32 },
    CreationPlayerName(WorldPlayerNameLookupError),
    MapPlayerName(WorldPlayerNameLookupError),
    DbCreationPlayerName(WorldPlayerNameLookupError),
    DbDataPlayerName(WorldPlayerNameLookupError),
    OrganizingName(OrganizingNameLookupBlock),
    ApplicationRemoval {
        map_key: i32,
        completed_removals: Vec<ApplyFactionRemoval>,
    },
    Initial(FactionInitialBlock),
    GeneratedIdCollision { faction_id: i32 },
    OtherFactionUpdate {
        faction_id: i32,
        source: OrganizingOtherFactionUpdateBlock,
    },
}

/// Внешние границы синхронного `COrganizingCtrl::CreateConfederation`.
pub(crate) trait ConfederationCreationEffects {
    type SessionReport;
    type SessionBlock;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8>;
    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>);
    fn begin_confederation_creation_session(
        &mut self,
        request: ConfederationCreationSessionRequest,
    ) -> Result<Self::SessionReport, Self::SessionBlock>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConfederationCreationRejection {
    FirstFactionReserved,
    SecondFactionReserved,
    ZeroFactionId,
    FirstFactionMissing,
    SecondFactionMissing,
    CreationFunctionDisabled { notice_sent: bool },
    SecondMasterOffline { notice_sent: bool },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ConfederationCreationStartOutcome<SessionReport> {
    Rejected(ConfederationCreationRejection),
    Started {
        first_player_id: i32,
        second_player_id: i32,
        first_faction_id: i32,
        second_faction_id: i32,
        net_exchange_id: i32,
        session: SessionReport,
    },
}

#[derive(Debug)]
pub(crate) enum ConfederationCreationStartBlock<SessionBlock> {
    MissingFirstMaster { faction_id: i32 },
    MissingSecondMaster { faction_id: i32 },
    MissingFirstProperty { faction_id: i32 },
    Session {
        first_faction_id: i32,
        second_faction_id: i32,
        first_reservation_removed: bool,
        second_reservation_removed: bool,
        source: SessionBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ConfederationCreationSnapshotDelivery {
    pub(crate) player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ConfederationCreationCallbackReport {
    pub(crate) terminal: ConfederationCreationTerminal,
    pub(crate) rejection_notice_sent: bool,
    pub(crate) first_faction_found: bool,
    pub(crate) second_faction_found: bool,
    pub(crate) union_id: Option<i32>,
    pub(crate) initial: Option<UnionInitialReport>,
    pub(crate) first_faction_notice: Option<FactionMemberInfoReport>,
    pub(crate) second_faction_addition: Option<UnionAddFactionOutcome>,
    pub(crate) first_superior_assigned: Option<bool>,
    pub(crate) second_superior_assigned: Option<bool>,
    pub(crate) first_owned_city_refresh: Option<FactionOwnedCityRefreshReport>,
    pub(crate) second_owned_city_refresh: Option<FactionOwnedCityRefreshReport>,
    pub(crate) first_player_refresh: Option<Vec<i32>>,
    pub(crate) second_player_refresh: Option<Vec<i32>>,
    pub(crate) first_player_snapshot: Option<ConfederationCreationSnapshotDelivery>,
    pub(crate) second_player_snapshot: Option<ConfederationCreationSnapshotDelivery>,
    pub(crate) repeated_first_city_refreshes: Vec<i32>,
    pub(crate) first_reservation_removed: bool,
    pub(crate) second_reservation_removed: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ConfederationCreationCallbackBlock {
    Initial {
        report: ConfederationCreationCallbackReport,
        source: UnionInitialBlock,
    },
    AddSecondFaction {
        report: ConfederationCreationCallbackReport,
        source: UnionAddFactionBlock,
    },
    FirstSuperior {
        report: ConfederationCreationCallbackReport,
        source: FactionSuperiorOrganizingBlock,
    },
    SecondSuperior {
        report: ConfederationCreationCallbackReport,
        source: FactionSuperiorOrganizingBlock,
    },
    FirstOwnedCityRefresh {
        report: ConfederationCreationCallbackReport,
        source: FactionOwnedCityRefreshBlock,
    },
    SecondOwnedCityRefresh {
        report: ConfederationCreationCallbackReport,
        source: FactionOwnedCityRefreshBlock,
    },
    FirstSnapshot {
        report: ConfederationCreationCallbackReport,
        source: UnionMemberSnapshotBlock,
    },
    SecondSnapshot {
        report: ConfederationCreationCallbackReport,
        source: UnionMemberSnapshotBlock,
    },
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

fn send_confederation_creation_notice<Effects>(
    effects: &mut Effects,
    first_player_id: i32,
    text_id: &'static [u8],
) where
    Effects: ConfederationCreationEffects,
{
    let second_text = effects.world_string(b"WS0193");
    let first_text = effects.world_string(text_id);
    effects.send_organizing_info(FactionMemberInfoRequest {
        recipient_player_id: first_player_id,
        first_text: legacy_c_string_prefix(&first_text),
        second_text: legacy_c_string_prefix(&second_text),
        information_type: -1,
        color: 0xFFDA_EDFE,
        trailing_value: 0,
    });
}

fn send_player_invite_faction_notice<Effects>(
    effects: &mut Effects,
    player_id: i32,
    first_text_id: &'static [u8],
    second_text_id: &'static [u8],
) where
    Effects: PlayerInviteFactionEffects,
{
    let second_text =
        <Effects as ConfederationCreationEffects>::world_string(effects, second_text_id);
    let first_text =
        <Effects as ConfederationCreationEffects>::world_string(effects, first_text_id);
    <Effects as ConfederationCreationEffects>::send_organizing_info(
        effects,
        FactionMemberInfoRequest {
            recipient_player_id: player_id,
            first_text: legacy_c_string_prefix(&first_text),
            second_text: legacy_c_string_prefix(&second_text),
            information_type: -1,
            color: 0xFFDA_EDFE,
            trailing_value: 0,
        },
    );
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
    new_day_time: TagTime,
    new_day_event_id: Option<TimerId>,
}

/// Наблюдаемый результат публикации двух DB staging-map в organizing-control.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OrganizingDatabasePublishReport {
    pub(crate) published_unions: usize,
    pub(crate) published_factions: usize,
}

/// Safe-граница constructor `CUnion::Initial` при DB materialization.
pub(crate) struct OrganizingDatabasePublishBlock {
    pub(crate) union_id: i32,
    pub(crate) source: UnionInitialBlock,
}

/// Publication-контракт верхнего `COrganizingCtrl::Initialize` после DB owner-ов.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingDatabaseLoadDisposition {
    PublishedAll,
    FactionLoadFailed,
    FactionLoadBlocked,
}

pub(crate) struct OrganizingDatabaseLoadReport {
    pub(crate) published_unions: usize,
    pub(crate) published_factions: usize,
    /// Точный `int` из `CRsUnion::LoadAllConfederation`, который `Initialize`
    /// передаёт в `%d` log независимо от дальнейшей publication-семантики.
    pub(crate) union_reported_count: i32,
    /// Точный `int` из `CRsFaction::LoadAllFaction` для следующего `%d` log.
    pub(crate) faction_reported_count: i32,
    pub(crate) union_load_returned_true: bool,
    pub(crate) disposition: OrganizingDatabaseLoadDisposition,
}

/// Safe-граница ordered `ReSetPermitDemise`: старый virtual call
/// разыменовывал сохранённый null faction-pointer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OrganizingPermitDemiseResetBlock {
    pub(crate) map_key: i32,
    pub(crate) reset_faction_ids: Vec<i32>,
}

/// Результат первой calendar-постановки `COrganizingCtrl::Initialize`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct OrganizingNewDayScheduleReport {
    pub(crate) scheduled_time: TagTime,
    pub(crate) event_id: TimerId,
}

/// Результат callback-а `OnNewDay` до внешнего `CCountryHandler::SetNewDay`.
#[derive(Debug)]
pub(crate) struct OrganizingNewDayReport {
    pub(crate) reset_faction_ids: Vec<i32>,
    pub(crate) country_day: u16,
    pub(crate) scheduled_time: TagTime,
    pub(crate) event_id: TimerId,
}

#[derive(Debug)]
pub(crate) enum OrganizingNewDayBlock {
    ResetPermitDemise(OrganizingPermitDemiseResetBlock),
    DateArithmetic(TagTimeArithmeticBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingNewDayScheduleBlock {
    DateArithmetic(TagTimeArithmeticBlock),
}

/// Финальный достигнутый suffix `COrganizingCtrl::Initialize` после DB-load.
#[derive(Clone, Copy, Debug)]
pub(crate) struct OrganizingInitializeSuffixReport {
    pub(crate) new_day_schedule: OrganizingNewDayScheduleReport,
}

/// Safe-граница suffix-а не откатывает уже поставленный calendar event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingInitializeSuffixBlock {
    NewDaySchedule(TagTimeArithmeticBlock),
    Billboard(FactionBillboardStatBlock),
}

/// Наблюдаемый no-op либо обе city-war мутации `SetEnemyFactionRelation`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EnemyFactionRelationOutcome {
    FirstFactionMissing,
    SecondFactionMissing,
    Applied,
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

/// Временно соединяет detached target faction с остальным controller-map.
struct DetachedFactionApplicationContext<'a, Effects> {
    controller: &'a mut COrganizingCtrl,
    target_map_key: i32,
    effects: &'a mut Effects,
}

impl<Effects> FactionOrganizingInfoContext for DetachedFactionApplicationContext<'_, Effects>
where
    Effects: FactionApplyForJoinEffects,
{
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        self.effects.world_string(string_id)
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        self.effects.send_organizing_info(request);
    }
}

impl<Effects> FactionApplyForJoinEffects for DetachedFactionApplicationContext<'_, Effects>
where
    Effects: FactionApplyForJoinEffects,
{
    fn already_declared_for_village_war(&self, faction_id: i32) -> bool {
        self.effects.already_declared_for_village_war(faction_id)
    }

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool {
        self.effects.already_declared_for_city_war(faction_id)
    }

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8> {
        self.effects.format_world_string(string_id, arguments)
    }

    fn faction_apply_log_enabled(&self) -> bool {
        self.effects.faction_apply_log_enabled()
    }

    fn write_faction_apply_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        player_id: i32,
        player_name: &[u8],
        log_type: i32,
    ) {
        self.effects.write_faction_apply_log(
            faction_id,
            faction_name,
            player_id,
            player_name,
            log_type,
        );
    }
}

impl<Effects> FactionApplyForJoinContext for DetachedFactionApplicationContext<'_, Effects>
where
    Effects: FactionApplyForJoinEffects,
{
    type Block = DetachedFactionApplicationContextBlock;

    fn player_already_in_faction(
        &self,
        current_faction: &CFaction,
        player_id: i32,
    ) -> Result<bool, Self::Block> {
        for (&map_key, faction) in &self.controller.factions {
            let faction = if map_key == self.target_map_key {
                current_faction
            } else {
                faction.as_deref().ok_or(
                    DetachedFactionApplicationContextBlock::MembershipNullFaction { map_key },
                )?
            };
            if faction.is_member(player_id) > 0 {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn remove_previous_faction_applications(
        &mut self,
        game: &CGame,
        current_faction: &mut CFaction,
        player_id: i32,
    ) -> Result<(), Self::Block> {
        let mut completed_removals = Vec::with_capacity(self.controller.factions.len());
        for (&map_key, faction) in &mut self.controller.factions {
            let outcome = if map_key == self.target_map_key {
                current_faction.remove_apply_member(game, player_id)
            } else {
                let Some(faction) = faction.as_deref_mut() else {
                    return Err(
                        DetachedFactionApplicationContextBlock::RemovalNullFaction {
                            map_key,
                            completed_removals,
                        },
                    );
                };
                faction.remove_apply_member(game, player_id)
            };
            completed_removals.push(ApplyFactionRemoval { map_key, outcome });
        }
        Ok(())
    }
}

/// Соединяет detached target faction с controller-map на полном `DoJoin`.
struct DetachedFactionDoJoinContext<'a, Effects> {
    controller: &'a mut COrganizingCtrl,
    game: &'a CGame,
    target_map_key: i32,
    effects: &'a mut Effects,
}

impl<Effects> FactionOrganizingInfoContext for DetachedFactionDoJoinContext<'_, Effects>
where
    Effects: FactionDoJoinEffects,
{
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        self.effects.world_string(string_id)
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        self.effects.send_organizing_info(request);
    }
}

impl<Effects> FactionDoJoinEffects for DetachedFactionDoJoinContext<'_, Effects>
where
    Effects: FactionDoJoinEffects,
{
    fn already_declared_for_village_war(&self, faction_id: i32) -> bool {
        self.effects.already_declared_for_village_war(faction_id)
    }

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool {
        self.effects.already_declared_for_city_war(faction_id)
    }

    fn goods_war_blocks_join(&self, faction_id: i32, manager_id: i32) -> bool {
        self.effects.goods_war_blocks_join(faction_id, manager_id)
    }

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8> {
        self.effects.format_world_string(string_id, arguments)
    }

    fn update_player_faction_info(&mut self, player_id: i32) {
        self.effects.update_player_faction_info(player_id);
    }

    fn faction_join_log_enabled(&self) -> bool {
        self.effects.faction_join_log_enabled()
    }

    fn write_faction_join_log(
        &mut self,
        member_id: i32,
        member_name: &[u8],
        manager_id: i32,
        manager_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
        log_type: i32,
    ) {
        self.effects.write_faction_join_log(
            member_id,
            member_name,
            manager_id,
            manager_name,
            faction_id,
            faction_name,
            log_type,
        );
    }
}

impl<Effects> FactionDoJoinContext for DetachedFactionDoJoinContext<'_, Effects>
where
    Effects: FactionDoJoinEffects,
{
    type Block = DetachedFactionDoJoinContextBlock;

    fn remove_previous_faction_applications(
        &mut self,
        game: &CGame,
        current_faction: &mut CFaction,
        player_id: i32,
    ) -> Result<(), Self::Block> {
        let mut completed_removals = Vec::with_capacity(self.controller.factions.len());
        for (&map_key, faction) in &mut self.controller.factions {
            let outcome = if map_key == self.target_map_key {
                current_faction.remove_apply_member(game, player_id)
            } else {
                let Some(faction) = faction.as_deref_mut() else {
                    return Err(DetachedFactionDoJoinContextBlock::RemovalNullFaction {
                        map_key,
                        completed_removals,
                    });
                };
                faction.remove_apply_member(game, player_id)
            };
            completed_removals.push(ApplyFactionRemoval { map_key, outcome });
        }
        Ok(())
    }

    fn applicant_already_in_faction(
        &self,
        current_faction: &CFaction,
        player_id: i32,
    ) -> Result<bool, Self::Block> {
        self.controller
            .player_faction_with_detached(self.target_map_key, current_faction, player_id)
            .map(|faction_id| faction_id > 0)
            .map_err(|map_key| {
                DetachedFactionDoJoinContextBlock::MembershipNullFaction { map_key }
            })
    }

    fn add_faction_to_client_by_player_id(
        &mut self,
        current_faction: &CFaction,
        player_id: i32,
    ) -> Result<bool, Self::Block> {
        self.controller
            .add_faction_to_client_with_detached(
                self.game,
                self.target_map_key,
                current_faction,
                player_id,
            )
            .map_err(DetachedFactionDoJoinContextBlock::FactionSnapshot)
    }

    fn add_all_faction_info_to_client_by_player_id(
        &mut self,
        current_faction: &CFaction,
        player_id: i32,
    ) -> Result<bool, Self::Block> {
        self.controller
            .add_all_faction_info_to_client_with_detached(
                self.game,
                self.target_map_key,
                current_faction,
                player_id,
            )
            .map_err(DetachedFactionDoJoinContextBlock::AllFactionInfo)
    }
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
            new_day_time: TagTime::from_fields([0, 0, 0, 0, 14, 15, 0, 0]),
            new_day_event_id: None,
        }
    }

    /// Ставит первый `OnNewDay` на ближайшую полночь exact `Initialize`.
    ///
    /// Legacy template хранит только время `14:15`; его нулевые date-поля
    /// получают дату одного current local snapshot. Первый scheduling явно
    /// затирает hour/minute/second, а `milliseconds` сохраняет из template.
    pub(crate) fn schedule_initial_new_day<Callback: Copy>(
        &mut self,
        current_time: TagTime,
        timer: &mut CTimer<Callback>,
        callback: Callback,
    ) -> Result<OrganizingNewDayScheduleReport, OrganizingNewDayScheduleBlock> {
        let mut scheduled_time = self.new_day_time_for_date(current_time);
        scheduled_time.hour = 0;
        scheduled_time.minute = 0;
        scheduled_time.second = 0;
        if scheduled_time.legacy_lt(current_time) {
            scheduled_time
                .add_day(1)
                .map_err(OrganizingNewDayScheduleBlock::DateArithmetic)?;
        }
        let event_id = timer.set_time_event(scheduled_time, callback, 0);
        self.new_day_event_id = Some(event_id);
        Ok(OrganizingNewDayScheduleReport {
            scheduled_time,
            event_id,
        })
    }

    /// Выполняет `OnNewDay`: reset, day-of-next-event и новое событие 14:15.
    pub(crate) fn on_new_day<Callback: Copy, SetCountryDay>(
        &mut self,
        current_time: TagTime,
        timer: &mut CTimer<Callback>,
        callback: Callback,
        mut set_country_day: SetCountryDay,
    ) -> Result<OrganizingNewDayReport, OrganizingNewDayBlock>
    where
        SetCountryDay: FnMut(u16),
    {
        let reset_faction_ids = self
            .reset_permit_demise()
            .map_err(OrganizingNewDayBlock::ResetPermitDemise)?;
        let mut scheduled_time = self.new_day_time_for_date(current_time);
        scheduled_time
            .add_day(1)
            .map_err(OrganizingNewDayBlock::DateArithmetic)?;
        set_country_day(scheduled_time.day);
        let event_id = timer.set_time_event(scheduled_time, callback, 0);
        self.new_day_event_id = Some(event_id);
        Ok(OrganizingNewDayReport {
            reset_faction_ids,
            country_day: scheduled_time.day,
            scheduled_time,
            event_id,
        })
    }

    fn new_day_time_for_date(&self, current_time: TagTime) -> TagTime {
        TagTime {
            year: current_time.year,
            month: current_time.month,
            day_of_week: self.new_day_time.day_of_week,
            day: current_time.day,
            hour: self.new_day_time.hour,
            minute: self.new_day_time.minute,
            second: self.new_day_time.second,
            milliseconds: self.new_day_time.milliseconds,
        }
    }

    /// Выполняет часть `Initialize` строго после DB-publication: сначала
    /// `SetTimeEvent(OnNewDay, 0)`, затем member/offense/defence snapshot-ы.
    pub(crate) fn finish_database_initialize<Callback: Copy>(
        &mut self,
        current_time: TagTime,
        timer: &mut CTimer<Callback>,
        on_new_day: Callback,
    ) -> Result<OrganizingInitializeSuffixReport, OrganizingInitializeSuffixBlock> {
        let new_day_schedule = self
            .schedule_initial_new_day(current_time, timer, on_new_day)
            .map_err(|error| match error {
                OrganizingNewDayScheduleBlock::DateArithmetic(source) => {
                    OrganizingInitializeSuffixBlock::NewDaySchedule(source)
                }
            })?;
        self.stat_billboard()
            .map_err(OrganizingInitializeSuffixBlock::Billboard)?;
        Ok(OrganizingInitializeSuffixReport { new_day_schedule })
    }

    /// Публикует DB-готовые union/faction owner-ы в порядке `Initialize`.
    ///
    /// Exact `Initialize` сперва чистит request-list, затем `LoadAllConfederation`
    /// materialize-ит и вставляет unions, после чего `LoadAllFaction` вставляет
    /// factions. При положительном уже найденном union `AddFactionOrganizing`
    /// не вызывает повторный clear dirty-mask; faction DB owner уже выполнил
    /// этот финальный clear, поэтому безопасная insertion сохраняет наблюдаемое
    /// состояние без ненужного virtual dispatch. Repeated map-key заменяет
    /// прежний owner, но не сохраняет старую pointer leak.
    pub(crate) fn publish_database_organizing(
        &mut self,
        union_master_title: &[u8],
        unions: Vec<UnionDatabaseLoadRecord>,
        factions: FactionPropertyLoadStaging,
    ) -> Result<OrganizingDatabasePublishReport, OrganizingDatabasePublishBlock> {
        let published_unions = self.publish_database_unions(union_master_title, unions)?;
        let published_factions = self.publish_database_factions(factions);
        Ok(OrganizingDatabasePublishReport {
            published_unions,
            published_factions,
        })
    }

    /// Принимает завершённые DB owner-outcome-ы с их разной cleanup-семантикой.
    pub(crate) fn publish_database_load_outcomes(
        &mut self,
        union_master_title: &[u8],
        unions: UnionLoadOutcome,
        factions: FactionLoadOutcome,
    ) -> Result<OrganizingDatabaseLoadReport, OrganizingDatabasePublishBlock> {
        let (union_load_returned_true, union_records, union_reported_count) = match unions {
            UnionLoadOutcome::ReturnedTrue {
                records,
                reported_count,
            } => (true, records, reported_count),
            UnionLoadOutcome::ReturnedFalse {
                records,
                reported_count,
            } => (false, records, reported_count),
        };
        let published_unions = self.publish_database_unions(union_master_title, union_records)?;
        match factions {
            FactionLoadOutcome::ReturnedTrue {
                factions,
                reported_count,
            } => Ok(OrganizingDatabaseLoadReport {
                published_unions,
                published_factions: self.publish_database_factions(factions),
                union_reported_count,
                faction_reported_count: reported_count,
                union_load_returned_true,
                disposition: OrganizingDatabaseLoadDisposition::PublishedAll,
            }),
            // Exact LoadAllFaction deletes its temporary map on any failed
            // helper; its completed prefix therefore cannot become live.
            FactionLoadOutcome::ReturnedFalse { reported_count, .. } => Ok(OrganizingDatabaseLoadReport {
                published_unions,
                published_factions: 0,
                union_reported_count,
                faction_reported_count: reported_count,
                union_load_returned_true,
                disposition: OrganizingDatabaseLoadDisposition::FactionLoadFailed,
            }),
            FactionLoadOutcome::BlockedInitial { reported_count, .. }
            | FactionLoadOutcome::BlockedMember { reported_count, .. }
            | FactionLoadOutcome::BlockedPronounce { reported_count, .. } => Ok(OrganizingDatabaseLoadReport {
                published_unions,
                published_factions: 0,
                union_reported_count,
                faction_reported_count: reported_count,
                union_load_returned_true,
                disposition: OrganizingDatabaseLoadDisposition::FactionLoadBlocked,
            }),
        }
    }

    fn publish_database_unions(
        &mut self,
        union_master_title: &[u8],
        unions: Vec<UnionDatabaseLoadRecord>,
    ) -> Result<usize, OrganizingDatabasePublishBlock> {
        self.request_establishment_union_players.clear();
        let published_unions = unions.len();
        for record in unions {
            let union_id = record.union_id;
            let union = CUnion::from_database_load_state(
                union_id,
                record.name,
                record.master_id,
                union_master_title,
                record.members,
            )
            .map_err(|source| OrganizingDatabasePublishBlock { union_id, source })?;
            self.confederations.insert(union_id, Some(Box::new(union)));
        }
        Ok(published_unions)
    }

    fn publish_database_factions(
        &mut self,
        factions: FactionPropertyLoadStaging,
    ) -> usize {
        let published_factions = factions.len();
        for (faction_id, faction) in factions {
            self.factions.insert(faction_id, Some(Box::new(faction)));
        }
        published_factions
    }

    /// Выполняет exact ordered `ReSetPermitDemise` без сетевых/DB эффектов.
    ///
    /// Каждая concrete faction получает virtual `SetPermitDemise(true)` в
    /// signed map-order. Уже выполненный prefix не откатывается при safe
    /// остановке вместо старого null-pointer dereference.
    pub(crate) fn reset_permit_demise(
        &mut self,
    ) -> Result<Vec<i32>, OrganizingPermitDemiseResetBlock> {
        let mut reset_faction_ids = Vec::with_capacity(self.factions.len());
        for (&map_key, faction) in &mut self.factions {
            let Some(faction) = faction.as_deref_mut() else {
                return Err(OrganizingPermitDemiseResetBlock {
                    map_key,
                    reset_faction_ids,
                });
            };
            faction.set_permit_demise(true);
            reset_faction_ids.push(faction.faction_id());
        }
        Ok(reset_faction_ids)
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

    /// Выполняет nullable faction lookup и concrete `CFaction::SetParam`.
    pub(crate) fn set_faction_parameter<Context>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        faction_id: i32,
        parameter: &[u8],
        value: i32,
        context: &mut Context,
    ) -> Result<Option<FactionSetParameterOutcome>, FactionSetParameterBlock>
    where
        Context: FactionSetParameterContext,
    {
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(None);
        };
        faction
            .set_parameter(game, parameters, parameter, value, context)
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
    pub(crate) fn faction_by_id_mut(&mut self, faction_id: i32) -> Option<&mut CFaction> {
        self.factions
            .get_mut(&faction_id)
            .and_then(Option::as_deref_mut)
    }

    /// Повторяет exact `GetCountryByFaction` без singleton и virtual ABI.
    pub(crate) fn country_by_faction(
        &self,
        faction_id: i32,
    ) -> Result<Option<u8>, FactionInitialPropertyBlock> {
        if faction_id <= 0 {
            return Ok(None);
        }
        let Some(faction) = self.faction_by_id(faction_id) else {
            return Ok(None);
        };
        faction.country().map(Some).ok_or(FactionInitialPropertyBlock)
    }

    /// Повторяет exact `AddOwnedCityToFaction` через concrete faction owner.
    pub(crate) fn add_owned_city_to_faction(
        &mut self,
        game: &CGame,
        faction_id: i32,
        region_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<Option<OwnedCityAddOutcome>, OwnedCityMutationBuildError> {
        if faction_id <= 0 {
            return Ok(None);
        }
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return Ok(None);
        };
        faction
            .add_owned_city(game, region_id, update_player)
            .map(Some)
    }

    /// Узкий concrete dispatch достигнутого `CFaction::SetGoodsWarCount`.
    pub(crate) fn set_faction_goods_war_count(
        &mut self,
        faction_id: i32,
        count: i32,
    ) -> Option<i32> {
        self.faction_by_id_mut(faction_id)
            .map(|faction| faction.set_goods_war_count(count))
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

    /// Повторяет ordered `IsConferationMaster` и его null-owner границу.
    pub(crate) fn union_id_by_master_faction(
        &self,
        faction_id: i32,
    ) -> Result<i32, ConfederationMasterLookupBlock> {
        for (&map_key, union) in &self.confederations {
            let Some(union) = union.as_deref() else {
                return Err(ConfederationMasterLookupBlock { map_key });
            };
            if union.master_id() == faction_id && union.union_id() > 0 {
                return Ok(union.union_id());
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

    /// Считает concrete faction owner-ы одной страны в signed map-order.
    pub(crate) fn faction_count_by_country(
        &self,
        country: u8,
    ) -> Result<i32, FactionCountryCountBlock> {
        let mut count = 0_i32;
        for (&map_key, faction) in &self.factions {
            let Some(faction) = faction.as_deref() else {
                continue;
            };
            let faction_country = faction.country().ok_or(FactionCountryCountBlock {
                map_key,
                matched_before_block: count,
                source: FactionInitialPropertyBlock,
            })?;
            if faction_country == country {
                count = count.wrapping_add(1);
            }
        }
        Ok(count)
    }

    /// Строит exact 11-элементную страницу faction ID/name одной страны.
    pub(crate) fn faction_list_page(
        &self,
        requested_page: i32,
        country: u8,
    ) -> Result<FactionListPage, FactionListPageBlock> {
        const PAGE_SIZE: i32 = 11;

        let normalized_page = requested_page.max(1);
        let start_index = normalized_page
            .wrapping_mul(PAGE_SIZE)
            .wrapping_sub(PAGE_SIZE);
        let total_factions = self.faction_count_by_country(country).map_err(|source| {
            FactionListPageBlock::Count {
                requested_page,
                normalized_page,
                country,
                phase: FactionListCountPhase::Initial,
                source,
            }
        })?;
        let remaining = total_factions.wrapping_sub(start_index);
        let entry_count = if remaining <= PAGE_SIZE {
            self.faction_count_by_country(country)
                .map_err(|source| FactionListPageBlock::Count {
                    requested_page,
                    normalized_page,
                    country,
                    phase: FactionListCountPhase::LastPageRecount,
                    source,
                })?
                .wrapping_sub(start_index)
                .max(0)
        } else {
            PAGE_SIZE
        };
        let mut page = FactionListPage {
            requested_page,
            normalized_page,
            country,
            start_index,
            total_factions,
            entry_count,
            entries: Vec::with_capacity(entry_count as usize),
            payload: Vec::new(),
        };
        append_i32(&mut page.payload, entry_count);

        let end_index = start_index.wrapping_add(entry_count);
        let mut country_index = 0_i32;
        for (&map_key, faction) in &self.factions {
            let Some(faction) = faction.as_deref() else {
                continue;
            };
            let faction_country = match faction.country() {
                Some(country) => country,
                None => {
                    return Err(FactionListPageBlock::Country {
                        page,
                        map_key,
                        source: FactionInitialPropertyBlock,
                    });
                }
            };
            if faction_country != country {
                continue;
            }

            if country_index >= start_index && country_index < end_index {
                append_i32(&mut page.payload, faction.faction_id());
                let name = legacy_c_string_prefix(faction.name());
                let required_bytes_with_nul = name.len().saturating_add(1);
                if required_bytes_with_nul > 256 {
                    return Err(FactionListPageBlock::NameWouldOverflow {
                        page,
                        map_key,
                        required_bytes_with_nul,
                    });
                }
                page.payload.extend_from_slice(name);
                page.payload.push(0);
                page.entries.push(FactionListEntry {
                    faction_id: faction.faction_id(),
                    name: name.to_vec(),
                });
            }
            country_index = country_index.wrapping_add(1);
            if country_index >= end_index {
                break;
            }
        }
        Ok(page)
    }

    /// Ищет organizing по имени в exact порядке faction-map, затем union-map.
    pub(crate) fn organizing_by_name(
        &self,
        requested_name: &[u8],
    ) -> Result<Option<OrganizingNameMatch>, OrganizingNameLookupBlock> {
        const LEGACY_NAME_CAPACITY: usize = 260;

        let requested_name = legacy_c_string_prefix(requested_name);
        if requested_name.len() >= LEGACY_NAME_CAPACITY {
            return Err(OrganizingNameLookupBlock::RequestedNameWouldOverflow {
                visible_len: requested_name.len(),
            });
        }

        for (&map_key, faction) in &self.factions {
            let Some(faction) = faction.as_deref() else {
                return Err(OrganizingNameLookupBlock::NullOwner {
                    kind: OrganizingNameKind::Faction,
                    map_key,
                });
            };
            let owner_name = legacy_c_string_prefix(faction.name());
            if owner_name.len() >= LEGACY_NAME_CAPACITY {
                return Err(OrganizingNameLookupBlock::OwnerNameWouldOverflow {
                    kind: OrganizingNameKind::Faction,
                    map_key,
                    visible_len: owner_name.len(),
                });
            }
            if owner_name.eq_ignore_ascii_case(requested_name) {
                return Ok(Some(OrganizingNameMatch {
                    kind: OrganizingNameKind::Faction,
                    map_key,
                    organizing_id: faction.faction_id(),
                }));
            }
        }

        for (&map_key, union) in &self.confederations {
            let Some(union) = union.as_deref() else {
                return Err(OrganizingNameLookupBlock::NullOwner {
                    kind: OrganizingNameKind::Union,
                    map_key,
                });
            };
            let owner_name = legacy_c_string_prefix(union.name());
            if owner_name.len() >= LEGACY_NAME_CAPACITY {
                return Err(OrganizingNameLookupBlock::OwnerNameWouldOverflow {
                    kind: OrganizingNameKind::Union,
                    map_key,
                    visible_len: owner_name.len(),
                });
            }
            if owner_name.eq_ignore_ascii_case(requested_name) {
                return Ok(Some(OrganizingNameMatch {
                    kind: OrganizingNameKind::Union,
                    map_key,
                    organizing_id: union.union_id(),
                }));
            }
        }

        Ok(None)
    }

    /// Выполняет virtual `GetCountry` для результата `FindOrgaByName`.
    pub(crate) fn country_by_name_match(
        &self,
        matched: OrganizingNameMatch,
    ) -> Result<u8, OrganizingNameCountryBlock> {
        match matched.kind {
            OrganizingNameKind::Faction => {
                let faction = self
                    .factions
                    .get(&matched.map_key)
                    .and_then(Option::as_deref)
                    .ok_or(OrganizingNameCountryBlock::TargetMissing {
                        kind: matched.kind,
                        map_key: matched.map_key,
                    })?;
                faction.country().ok_or(
                    OrganizingNameCountryBlock::MissingFactionProperty {
                        map_key: matched.map_key,
                    },
                )
            }
            OrganizingNameKind::Union => {
                self
                    .confederations
                    .get(&matched.map_key)
                    .and_then(Option::as_deref)
                    .ok_or(OrganizingNameCountryBlock::TargetMissing {
                        kind: matched.kind,
                        map_key: matched.map_key,
                    })?;
                Ok(0)
            }
        }
    }

    /// Вызывает union `ApplyForJoin` по уже найденному map-key, не повторяя
    /// master-player lookup ветки `0x60118`.
    pub(crate) fn apply_for_named_union_join<Effects>(
        &mut self,
        game: &CGame,
        map_key: i32,
        applicant_faction_id: i32,
        second_parameter: i32,
        third_parameter: i32,
        effects: &mut Effects,
    ) -> Result<
        UnionApplyForJoinOutcome<Effects::SessionReport>,
        OrganizingNamedUnionApplicationBlock<Effects::SessionBlock>,
    >
    where
        Effects: UnionApplyForJoinEffects,
    {
        let Some(mut union) = self
            .confederations
            .get_mut(&map_key)
            .and_then(Option::take)
        else {
            return Err(OrganizingNamedUnionApplicationBlock::TargetMissing { map_key });
        };
        let union_id = union.union_id();
        let applicant_membership = self.is_free_faction(applicant_faction_id);
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
            .get_mut(&map_key)
            .expect("detached union slot не удаляется") = Some(union);
        result.map_err(|source| OrganizingNamedUnionApplicationBlock::Apply {
            map_key,
            union_id,
            source,
        })
    }

    /// Вызывает faction `ApplyForJoin` через detached owner, сохраняя полный
    /// signed map-order его reentrant membership/remove проходов.
    pub(crate) fn apply_for_faction_join_by_map_key<Effects>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        map_key: i32,
        player_id: i32,
        second_parameter: i32,
        third_parameter: i32,
        effects: &mut Effects,
    ) -> Result<FactionApplyForJoinOutcome, OrganizingFactionApplicationBlock>
    where
        Effects: FactionApplyForJoinEffects,
    {
        let Some(mut faction) = self.factions.get_mut(&map_key).and_then(Option::take) else {
            return Err(OrganizingFactionApplicationBlock::TargetMissing { map_key });
        };
        let result = {
            let mut context = DetachedFactionApplicationContext {
                controller: self,
                target_map_key: map_key,
                effects,
            };
            faction.apply_for_join(
                game,
                parameters,
                player_id,
                second_parameter,
                third_parameter,
                &mut context,
            )
        };
        *self
            .factions
            .get_mut(&map_key)
            .expect("detached faction slot не удаляется") = Some(faction);
        result.map_err(|source| OrganizingFactionApplicationBlock::Apply { map_key, source })
    }

    /// Выполняет точную controller-цепочку `0x6010A`: первый lookup faction,
    /// один local-time snapshot, повторный lookup и virtual `DoJoin`.
    pub(crate) fn do_faction_join_by_manager<Effects, GetLocalTime>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        manager_id: i32,
        applicant_id: i32,
        approve_flag: i32,
        get_local_time: GetLocalTime,
        effects: &mut Effects,
    ) -> Result<OrganizingFactionDoJoinOutcome, OrganizingFactionDoJoinBlock>
    where
        Effects: FactionDoJoinEffects,
        GetLocalTime: FnOnce() -> TagTimeValue,
    {
        let faction_id = match self.is_free_player(manager_id) {
            FreePlayerLookup::NoFaction => 0,
            FreePlayerLookup::Faction(faction_id) => faction_id,
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return Err(OrganizingFactionDoJoinBlock::ManagerMembership { map_key });
            }
        };
        if self.faction_by_id(faction_id).is_none() {
            return Ok(OrganizingFactionDoJoinOutcome::FactionNotFound { faction_id });
        }

        let join_time = get_local_time();
        let mut faction = self
            .factions
            .get_mut(&faction_id)
            .and_then(Option::take)
            .expect("первый GetFactionOrganizing подтвердил тот же live owner");
        let result = {
            let mut context = DetachedFactionDoJoinContext {
                controller: self,
                game,
                target_map_key: faction_id,
                effects,
            };
            faction.do_join(
                game,
                parameters,
                manager_id,
                applicant_id,
                approve_flag,
                join_time,
                &mut context,
            )
        };
        *self
            .factions
            .get_mut(&faction_id)
            .expect("detached faction slot не удаляется") = Some(faction);
        result
            .map(|outcome| OrganizingFactionDoJoinOutcome::Applied {
                faction_id,
                join_time,
                outcome,
            })
            .map_err(|source| OrganizingFactionDoJoinBlock::DoJoin {
                faction_id,
                source,
            })
    }

    fn player_faction_with_detached(
        &self,
        target_map_key: i32,
        current_faction: &CFaction,
        player_id: i32,
    ) -> Result<i32, i32> {
        for (&map_key, faction) in &self.factions {
            let faction = if map_key == target_map_key {
                current_faction
            } else {
                faction.as_deref().ok_or(map_key)?
            };
            let faction_id = faction.is_member(player_id);
            if faction_id > 0 {
                return Ok(faction_id);
            }
        }
        Ok(0)
    }

    fn faction_with_detached<'a>(
        &'a self,
        target_map_key: i32,
        current_faction: &'a CFaction,
        map_key: i32,
    ) -> Option<&'a CFaction> {
        if map_key == target_map_key {
            return Some(current_faction);
        }
        self.factions.get(&map_key).and_then(Option::as_deref)
    }

    /// Выполняет exact public `AddFactionToClientByPlayerID` без detach.
    pub(crate) fn add_faction_to_client_by_player_id(
        &self,
        game: &CGame,
        player_id: i32,
    ) -> Result<bool, FactionClientSnapshotBlock> {
        let faction_id = match self.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => return Ok(false),
            FreePlayerLookup::Faction(faction_id) => faction_id,
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return Err(FactionClientSnapshotBlock::MembershipNullFaction { map_key });
            }
        };
        let Some(faction) = self.faction_by_id(faction_id) else {
            return Ok(false);
        };
        self.add_faction_to_client_with_detached(game, faction_id, faction, player_id)
    }

    /// Выполняет exact `AddUnionToClientByPlayerID` для одного online player.
    pub(crate) fn add_union_to_client_by_player_id(
        &mut self,
        game: &CGame,
        player_id: i32,
    ) -> Result<UnionClientSnapshotByPlayerOutcome, UnionClientSnapshotByPlayerBlock> {
        let faction_id = match self.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => {
                return Ok(UnionClientSnapshotByPlayerOutcome::NoFaction);
            }
            FreePlayerLookup::Faction(faction_id) => faction_id,
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return Err(UnionClientSnapshotByPlayerBlock::PlayerMembership { map_key });
            }
        };
        let union_id = match self.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => {
                return Ok(UnionClientSnapshotByPlayerOutcome::NoUnion { faction_id });
            }
            FreeFactionLookup::Union(union_id) => union_id,
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                return Err(UnionClientSnapshotByPlayerBlock::UnionMembership { map_key });
            }
        };
        if self.confederation_by_id(union_id).is_none() {
            return Ok(UnionClientSnapshotByPlayerOutcome::MissingUnion {
                faction_id,
                union_id,
            });
        }
        let Some(player) = game.online_player_by_id(player_id as u32) else {
            return Ok(UnionClientSnapshotByPlayerOutcome::PlayerOffline {
                faction_id,
                union_id,
            });
        };
        let game_server_id = game.game_server_number_by_player_id(player_id);
        let (factions, confederations) = (&self.factions, &mut self.confederations);
        let union = confederations
            .get_mut(&union_id)
            .and_then(Option::as_deref_mut)
            .expect("union owner проверен до split borrow");
        let mut message = CMessage::new(UNION_INITIAL_MESSAGE_TYPE);
        message.base_mut().add_long(player_id);
        let mut snapshot = Vec::new();
        union
            .add_to_byte_array(&mut snapshot, &UnionFactionMapView { factions })
            .map_err(|source| UnionClientSnapshotByPlayerBlock::Snapshot {
                faction_id,
                union_id,
                source,
            })?;
        message.base_mut().add(&snapshot);
        let result = game.send_msg_to_game_server(game_server_id, &message);
        player.set_faction_data_received(true);
        Ok(UnionClientSnapshotByPlayerOutcome::Sent {
            faction_id,
            union_id,
            game_server_id,
            result,
        })
    }

    /// Выполняет exact public `AddAllFactinInfoToClientByPlayerID` без detach.
    pub(crate) fn add_all_faction_info_to_client_by_player_id(
        &self,
        game: &CGame,
        player_id: i32,
    ) -> Result<bool, AllFactionInfoClientBlock> {
        let faction_id = match self.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => return Ok(false),
            FreePlayerLookup::Faction(faction_id) => faction_id,
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return Err(AllFactionInfoClientBlock::MembershipNullFaction { map_key });
            }
        };
        let Some(faction) = self.faction_by_id(faction_id) else {
            return Ok(false);
        };
        self.add_all_faction_info_to_client_with_detached(
            game,
            faction_id,
            faction,
            player_id,
        )
    }

    /// Выполняет exact `AddFactionToClientByPlayerID` с временно detached
    /// target faction на её исходной позиции controller-map.
    fn add_faction_to_client_with_detached(
        &self,
        game: &CGame,
        target_map_key: i32,
        current_faction: &CFaction,
        player_id: i32,
    ) -> Result<bool, FactionClientSnapshotBlock> {
        let faction_id = self
            .player_faction_with_detached(target_map_key, current_faction, player_id)
            .map_err(|map_key| FactionClientSnapshotBlock::MembershipNullFaction { map_key })?;
        if faction_id <= 0 {
            return Ok(false);
        }
        let Some(faction) = self.faction_with_detached(
            target_map_key,
            current_faction,
            faction_id,
        ) else {
            return Ok(false);
        };
        let Some(player) = game.online_player_by_id(player_id as u32) else {
            return Ok(false);
        };
        let game_server_id = game.game_server_number_by_player_id(player_id);
        let mut snapshot = Vec::new();
        faction
            .add_full_snapshot_to_byte_array(game, &mut snapshot)
            .map_err(|source| FactionClientSnapshotBlock::FullSnapshot {
                faction_id,
                source,
            })?;
        let mut message = CMessage::new(0x7FE02);
        message.base_mut().add_long(player_id);
        message.base_mut().add(&snapshot);
        // Snapshot дописан непосредственно в vector и потому требует exact
        // `CBaseMessage::Update` перед `SendToMapID`.
        message.base_mut().update();
        let _ = game.send_msg_to_game_server(game_server_id, &message);
        player.set_faction_data_received(true);
        Ok(true)
    }

    /// Выполняет exact misspelled `AddAllFactinInfoToClientByPlayerID`.
    fn add_all_faction_info_to_client_with_detached(
        &self,
        game: &CGame,
        target_map_key: i32,
        current_faction: &CFaction,
        player_id: i32,
    ) -> Result<bool, AllFactionInfoClientBlock> {
        let faction_id = self
            .player_faction_with_detached(target_map_key, current_faction, player_id)
            .map_err(|map_key| AllFactionInfoClientBlock::MembershipNullFaction { map_key })?;
        if faction_id <= 0 || game.online_player_by_id(player_id as u32).is_none() {
            return Ok(false);
        }

        let game_server_id = game.game_server_number_by_player_id(player_id);
        let mut message = CMessage::new(0x7FE08);
        message.base_mut().add_long(player_id);
        message
            .base_mut()
            .add_long(self.factions.len() as u32 as i32);
        let mut completed_factions = 0usize;
        for (&map_key, faction) in &self.factions {
            let faction = if map_key == target_map_key {
                current_faction
            } else {
                faction.as_deref().ok_or(AllFactionInfoClientBlock::NullFaction {
                    map_key,
                    completed_factions,
                })?
            };
            let faction_id = faction.faction_id();
            message.base_mut().add_long(faction_id);
            let country = faction
                .country()
                .ok_or(AllFactionInfoClientBlock::MissingCountry {
                    map_key,
                    faction_id,
                    completed_factions,
                })?;
            message.base_mut().add_byte(country);
            let name = legacy_c_string_prefix(faction.name());
            if name.len() >= 256 {
                return Err(AllFactionInfoClientBlock::NameWouldOverflow {
                    map_key,
                    faction_id,
                    visible_len: name.len(),
                    completed_factions,
                });
            }
            message.base_mut().add(name);
            message.base_mut().add_byte(0);
            completed_factions += 1;
        }
        let _ = game.send_msg_to_game_server(game_server_id, &message);
        Ok(true)
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

    /// Повторяет exact lookup `0x6010E`: любой player membership, затем union.
    pub(crate) fn union_id_by_player_membership(
        &self,
        player_id: i32,
    ) -> Result<Option<i32>, OrganizingUnionByPlayerBlock> {
        let faction_id = match self.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => return Ok(None),
            FreePlayerLookup::Faction(faction_id) => faction_id,
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return Err(OrganizingUnionByPlayerBlock::PlayerMembership { map_key });
            }
        };
        let union_id = match self.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => return Ok(None),
            FreeFactionLookup::Union(union_id) => union_id,
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                return Err(OrganizingUnionByPlayerBlock::UnionMembership { map_key });
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

    /// Вызывает exact `CUnion::Invite(source faction, invited faction)`.
    pub(crate) fn invite_faction_to_union<Effects>(
        &mut self,
        game: &CGame,
        union_id: i32,
        inviter_faction_id: i32,
        invited_faction_id: i32,
        effects: &mut Effects,
    ) -> Result<
        OrganizingUnionInviteOutcome<Effects::SessionReport>,
        OrganizingUnionInviteDispatchBlock<Effects::SessionBlock>,
    >
    where
        Effects: UnionInviteEffects,
    {
        let invited_membership = self.is_free_faction(invited_faction_id);
        let Some(mut union) = self
            .confederations
            .get_mut(&union_id)
            .and_then(Option::take)
        else {
            return Err(OrganizingUnionInviteDispatchBlock::TargetMissing { union_id });
        };
        self.detached_union_membership_lookup
            .set(Some((invited_faction_id, invited_membership)));
        let result = union.invite(
            game,
            inviter_faction_id,
            invited_faction_id,
            self,
            effects,
        );
        self.detached_union_membership_lookup.set(None);
        *self
            .confederations
            .get_mut(&union_id)
            .expect("detached union slot не удаляется") = Some(union);
        result
            .map(|outcome| OrganizingUnionInviteOutcome::Applied { union_id, outcome })
            .map_err(|source| OrganizingUnionInviteDispatchBlock::Invite {
                union_id,
                source,
            })
    }

    /// Выполняет exact controller-цепочку `0x6010C`: `GetUnion(manager)` и
    /// virtual `CUnion::FireOut(manager, target faction)`.
    pub(crate) fn fire_out_union_by_master<Effects>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        manager_id: i32,
        target_faction_id: i32,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<OrganizingUnionFireOutOutcome, OrganizingUnionFireOutBlock>
    where
        Effects: UnionFireOutEffects,
    {
        let Some(union_id) = self
            .union_id_by_master_player(manager_id)
            .map_err(OrganizingUnionFireOutBlock::Lookup)?
        else {
            return Ok(OrganizingUnionFireOutOutcome::UnionNotFound);
        };
        let mut union = self
            .confederations
            .get_mut(&union_id)
            .and_then(Option::take)
            .expect("GetUnion вернул живой owner из того же controller map");
        let result = union.fire_out(
            game,
            parameters,
            manager_id,
            target_faction_id,
            self,
            effects,
            update_player,
        );
        let should_disband = union.member_count() <= 1;
        *self
            .confederations
            .get_mut(&union_id)
            .expect("detached union slot не удаляется") = Some(union);
        let outcome = result
            .map_err(|source| OrganizingUnionFireOutBlock::FireOut { union_id, source })?;
        let automatic_disband = if should_disband {
            match self.disband_confederation(
                game,
                parameters,
                manager_id,
                union_id,
                effects,
                update_player,
            ) {
                Ok(outcome) => Some(outcome),
                Err(source) => {
                    return Err(OrganizingUnionFireOutBlock::AutomaticDisband {
                        union_id,
                        fire_out: outcome,
                        source,
                    });
                }
            }
        } else {
            None
        };
        Ok(OrganizingUnionFireOutOutcome::Applied {
            union_id,
            outcome,
            automatic_disband,
        })
    }

    /// Выполняет exact controller-цепочку `0x6010E`, включая автоматический
    /// disband через заново вычисленный `CUnion::GetPlayerHeader()`.
    pub(crate) fn exit_union_by_player<Effects>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        player_id: i32,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<OrganizingUnionExitOutcome, OrganizingUnionExitBlock>
    where
        Effects: UnionFireOutEffects,
    {
        let Some(union_id) = self
            .union_id_by_player_membership(player_id)
            .map_err(OrganizingUnionExitBlock::Lookup)?
        else {
            return Ok(OrganizingUnionExitOutcome::UnionNotFound);
        };
        let mut union = self
            .confederations
            .get_mut(&union_id)
            .and_then(Option::take)
            .expect("GetConfederationOrganizing вернул живой owner из того же controller map");
        let result = union.exit(
            game,
            parameters,
            player_id,
            self,
            effects,
            update_player,
        );
        let should_disband = union.member_count() <= 1;
        let outcome = match result {
            Ok(outcome) => outcome,
            Err(source) => {
                *self
                    .confederations
                    .get_mut(&union_id)
                    .expect("detached union slot не удаляется") = Some(union);
                return Err(OrganizingUnionExitBlock::Exit { union_id, source });
            }
        };
        let player_header = should_disband.then(|| {
            union.player_header(|master_faction_id| {
                let Some(faction) = self.faction_by_id(master_faction_id) else {
                    return Ok(None);
                };
                faction
                    .master_id()
                    .map(Some)
                    .ok_or(UnionPlayerHeaderLookupBlock { master_faction_id })
            })
        });
        *self
            .confederations
            .get_mut(&union_id)
            .expect("detached union slot не удаляется") = Some(union);

        let automatic_disband = if let Some(player_header) = player_header {
            let player_header = match player_header {
                Ok(player_header) => player_header,
                Err(source) => {
                    return Err(OrganizingUnionExitBlock::AutomaticPlayerHeader {
                        union_id,
                        exit: outcome,
                        source,
                    });
                }
            };
            match self.disband_confederation(
                game,
                parameters,
                player_header,
                union_id,
                effects,
                update_player,
            ) {
                Ok(outcome) => Some(outcome),
                Err(source) => {
                    return Err(OrganizingUnionExitBlock::AutomaticDisband {
                        union_id,
                        exit: outcome,
                        player_header,
                        source,
                    });
                }
            }
        } else {
            None
        };
        Ok(OrganizingUnionExitOutcome::Applied {
            union_id,
            outcome,
            automatic_disband,
        })
    }

    /// Выполняет exact controller-цепочку `0x60110`: `GetUnion(old master)` и
    /// virtual `CUnion::Demise(old master, new master faction)`.
    pub(crate) fn demise_union_by_master<Effects>(
        &mut self,
        game: &CGame,
        old_master_player_id: i32,
        new_master_faction_id: i32,
        effects: &mut Effects,
        get_tick: &mut dyn FnMut() -> u32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<OrganizingUnionDemiseOutcome, OrganizingUnionDemiseBlock>
    where
        Effects: UnionFireOutEffects,
    {
        let Some(union_id) = self
            .union_id_by_master_player(old_master_player_id)
            .map_err(OrganizingUnionDemiseBlock::Lookup)?
        else {
            return Ok(OrganizingUnionDemiseOutcome::UnionNotFound);
        };
        let mut union = self
            .confederations
            .get_mut(&union_id)
            .and_then(Option::take)
            .expect("GetUnion вернул живой owner из того же controller map");
        let result = union.demise(
            game,
            old_master_player_id,
            new_master_faction_id,
            self,
            effects,
            get_tick,
            update_player,
        );
        *self
            .confederations
            .get_mut(&union_id)
            .expect("detached union slot не удаляется") = Some(union);
        result
            .map(|outcome| OrganizingUnionDemiseOutcome::Applied { union_id, outcome })
            .map_err(|source| OrganizingUnionDemiseBlock::Demise { union_id, source })
    }

    /// Выполняет exact `DisbandConferation`: war gates, concrete union
    /// disband, delete-очередь, owned-city refresh и удаление owner-а.
    pub(crate) fn disband_confederation<Effects>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        manager_id: i32,
        union_id: i32,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<OrganizingConfederationDisbandOutcome, OrganizingConfederationDisbandBlock>
    where
        Effects: UnionFireOutEffects,
    {
        let Some(mut union) = self
            .confederations
            .get_mut(&union_id)
            .and_then(Option::take)
        else {
            return Ok(OrganizingConfederationDisbandOutcome::UnionNotFound);
        };

        let rejection = if union.has_enemy_faction(self) {
            Some((OrganizingConfederationDisbandRejection::StandardWar, b"WS0246".as_slice()))
        } else if union.has_city_war_enemy_faction(self) {
            Some((OrganizingConfederationDisbandRejection::CityWar, b"WS0247".as_slice()))
        } else {
            None
        };
        if let Some((rejection, string_id)) = rejection {
            let first_text = effects.world_string(string_id);
            let second_text = effects.world_string(b"WS0121");
            effects.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: manager_id,
                first_text: &first_text,
                second_text: &second_text,
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            *self
                .confederations
                .get_mut(&union_id)
                .expect("detached union slot не удаляется") = Some(union);
            return Ok(OrganizingConfederationDisbandOutcome::Rejected(rejection));
        }

        let standard_enemy_clear = union.clear_enemy_factions(self);
        let city_enemy_clear = union.clear_city_war_enemy_factions(self);
        let union_outcome = match union.disband(
            game,
            parameters,
            manager_id,
            self,
            effects,
        ) {
            Ok(outcome) => outcome,
            Err(source) => {
                *self
                    .confederations
                    .get_mut(&union_id)
                    .expect("detached union slot не удаляется") = Some(union);
                return Err(OrganizingConfederationDisbandBlock::Disband {
                    union_id,
                    source,
                });
            }
        };
        if !matches!(union_outcome, UnionDisbandOutcome::Disbanded(_)) {
            *self
                .confederations
                .get_mut(&union_id)
                .expect("detached union slot не удаляется") = Some(union);
            return Ok(OrganizingConfederationDisbandOutcome::Applied(
                OrganizingConfederationDisbandReport {
                    union_id,
                    standard_enemy_clear,
                    city_enemy_clear,
                    union_outcome,
                    delete_queued: false,
                    owned_city_refreshes: Vec::new(),
                    player_refresh: None,
                },
            ));
        }

        self.delete_unions.push_back(union_id);
        let mut owned_city_refreshes = Vec::new();
        if let Some(master_faction) = self.faction_by_id(union.master_id()) {
            let master_faction_id = master_faction.faction_id();
            for &region_id in master_faction.owned_cities() {
                effects.refresh_owned_city(region_id, master_faction_id, 0);
                owned_city_refreshes.push((region_id, master_faction_id, 0));
            }
        }
        self.confederations.remove(&union_id);
        let player_refresh = union.update_player_faction_info(0, self, game, update_player);
        Ok(OrganizingConfederationDisbandOutcome::Applied(
            OrganizingConfederationDisbandReport {
                union_id,
                standard_enemy_clear,
                city_enemy_clear,
                union_outcome,
                delete_queued: true,
                owned_city_refreshes,
                player_refresh: Some(player_refresh),
            },
        ))
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
        Context: FactionDisbandContext,
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

        Ok(OrganizingDisbandOutcome::Disbanded {
            progress,
            retired_faction: faction,
        })
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

    /// Повторяет organizing-часть exact `OnDeleteRole` после синхронного
    /// `CRsPlayer::GetPlayerCountryByID`/`CCountry::HasJob` префикса.
    ///
    /// Для обычного участника `DelMember` выполняется до результата `0`.
    /// Master свободной faction получает `1`: машинный slot `+0x90` возвращает
    /// `GetMembers`, а проверка читает `_Mysize > 0`; после только что
    /// успешного `IsFreePlayer` нулевая ветвь `DisbandFaction` недостижима.
    /// Для faction в союзе достаточно существования union-owner-а: concrete
    /// `CUnion::DelMember` не читает receiver, отвязывает faction и всегда
    /// возвращает true, поэтому точный результат равен `3`, а не `2`.
    pub(crate) fn on_delete_role(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        player_id: i32,
        country_has_job: bool,
    ) -> Result<OrganizingDeleteRoleOutcome, OrganizingDeleteRoleBlock> {
        if country_has_job {
            return Ok(OrganizingDeleteRoleOutcome::CountryJob);
        }

        let faction_id = match self.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => {
                return Ok(OrganizingDeleteRoleOutcome::AllowedNoFaction);
            }
            FreePlayerLookup::Faction(faction_id) => faction_id,
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return Err(OrganizingDeleteRoleBlock::NullFactionDuringMembershipScan {
                    map_key,
                });
            }
        };

        let Some(faction) = self
            .factions
            .get_mut(&faction_id)
            .and_then(Option::as_deref_mut)
        else {
            return Ok(OrganizingDeleteRoleOutcome::AllowedNoFaction);
        };
        if faction.member_job_level(player_id) < 1 {
            let removal = faction.del_member(player_id, parameters).map_err(|source| {
                OrganizingDeleteRoleBlock::MemberRemoval { faction_id, source }
            })?;
            return Ok(OrganizingDeleteRoleOutcome::MemberRemoved {
                faction_id,
                removal,
            });
        }

        let union_id = faction
            .superior_organizing()
            .ok_or(OrganizingDeleteRoleBlock::MissingFactionProperty { faction_id })?;
        if union_id < 1 {
            debug_assert!(faction.get_member_num() > 0);
            return Ok(OrganizingDeleteRoleOutcome::FactionMaster { faction_id });
        }

        if self.confederation_by_id(union_id).is_none() {
            return Ok(OrganizingDeleteRoleOutcome::UnionMissing {
                faction_id,
                union_id,
            });
        }
        let detach = self
            .detach_union_member(game, parameters, faction_id)
            .map_err(OrganizingDeleteRoleBlock::UnionDetach)?;
        Ok(OrganizingDeleteRoleOutcome::UnionDetached {
            faction_id,
            union_id,
            detach,
        })
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

    /// Разворачивает faction в concrete faction-организации для city-war.
    ///
    /// Свободная faction даёт собственный ID только при живом owner-е. Union
    /// использует исходный signed member-order и pointer-identity dedupe
    /// `CUnion::GetAllFacs`; отсутствующие member-owner-ы пропускаются.
    pub(crate) fn expand_city_war_faction_organizings(
        &self,
        faction_id: i32,
    ) -> Result<Vec<i32>, FactionUnionMembershipLookupBlock> {
        match self.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => Ok(self
                .faction_by_id(faction_id)
                .map(|faction| vec![faction.faction_id()])
                .unwrap_or_default()),
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                Err(FactionUnionMembershipLookupBlock { map_key })
            }
            FreeFactionLookup::Union(union_id) => {
                let Some(union) = self.confederation_by_id(union_id) else {
                    return Ok(Vec::new());
                };
                Ok(union
                    .member_organizings(|member_id| self.faction_by_id(member_id))
                    .into_iter()
                    .map(CFaction::faction_id)
                    .collect())
            }
        }
    }

    /// Добавляет city-war enemy через concrete faction virtual-owner.
    pub(crate) fn add_city_war_enemy_organizing<Context>(
        &mut self,
        organizing_id: i32,
        enemy_organizing_id: i32,
        context: &mut Context,
    ) -> Result<bool, FactionEnemyMutationBlock>
    where
        Context: FactionEnemyMutationContext,
    {
        let Some(faction) = self.faction_by_id_mut(organizing_id) else {
            return Ok(false);
        };
        faction.add_city_war_enemy_organizing(enemy_organizing_id, context)?;
        Ok(true)
    }

    /// Применяет одну загруженную пару `CFactionWarSys` к обоим city-war set.
    ///
    /// Exact controller сначала отдельно ищет обе positive faction ID и
    /// выполняет оба virtual `AddCityWarEnemyOrganizing` только когда обе
    /// живы. Поэтому первый mutation и его `WS0160` log всегда происходят
    /// раньше второго; `FactionEnemyMutationContext` остаётся тем же внешним
    /// string-table/war-log owner-ом, а не упрощается до прямой вставки set.
    pub(crate) fn set_enemy_faction_relation<Context>(
        &mut self,
        first_faction_id: i32,
        second_faction_id: i32,
        context: &mut Context,
    ) -> Result<EnemyFactionRelationOutcome, FactionEnemyMutationBlock>
    where
        Context: FactionEnemyMutationContext,
    {
        let Some(first_id) = (first_faction_id > 0)
            .then(|| self.faction_by_id(first_faction_id).map(CFaction::faction_id))
            .flatten()
        else {
            return Ok(EnemyFactionRelationOutcome::FirstFactionMissing);
        };
        let Some(second_id) = (second_faction_id > 0)
            .then(|| self.faction_by_id(second_faction_id).map(CFaction::faction_id))
            .flatten()
        else {
            return Ok(EnemyFactionRelationOutcome::SecondFactionMissing);
        };

        self.add_city_war_enemy_organizing(first_id, second_id, context)?;
        self.add_city_war_enemy_organizing(second_id, first_id, context)?;
        Ok(EnemyFactionRelationOutcome::Applied)
    }

    /// Возвращает следующий organizing ID по exact signed maximum обоих map.
    pub(crate) fn generate_db_organizing_id(&self) -> i32 {
        let mut maximum = 1_i32;
        if let Some(&faction_maximum) = self.factions.keys().next_back()
            && 1 < faction_maximum
        {
            maximum = faction_maximum;
        }
        if let Some(&union_maximum) = self.confederations.keys().next_back()
            && maximum < union_maximum
        {
            return union_maximum.wrapping_add(1);
        }
        maximum.wrapping_add(1)
    }

    /// Ставит city-war changed-флаг всем живым faction в signed map-order.
    pub(crate) fn set_all_city_faction_enemy_changed(&mut self, changed: bool) {
        for faction in self.factions.values_mut().flatten() {
            faction.set_city_war_enemy_factions_changed(changed);
        }
    }

    /// Очищает city-war relation-set всех живых faction в signed map-order.
    pub(crate) fn clear_all_city_faction_relations(&mut self) {
        for faction in self.factions.values_mut().flatten() {
            faction.clear_city_war_enemy_factions();
        }
    }

    /// Публикует изменённые city-war relation-set в signed map-order.
    pub(crate) fn update_all_city_enemy_faction_relations(
        &self,
        game: &CGame,
        update_player: &mut dyn FnMut(i32),
    ) -> Vec<(i32, CityWarEnemyRefreshOutcome)> {
        self.factions
            .iter()
            .filter_map(|(&map_key, faction)| {
                faction.as_deref().map(|faction| {
                    (
                        map_key,
                        faction.update_city_war_enemy_faction(game, &mut *update_player),
                    )
                })
            })
            .collect()
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

    /// Выполняет exact `OnPlayerInviteFaction` до выбранного union owner-а.
    pub(crate) fn on_player_invite_faction<Effects>(
        &mut self,
        game: &CGame,
        village_war: &CVillageWarSys,
        attack_city: &CAttackCitySys,
        player_id: i32,
        invited_faction_id: i32,
        effects: &mut Effects,
    ) -> Result<
        PlayerInviteFactionOutcome<
            <Effects as ConfederationCreationEffects>::SessionReport,
            <Effects as UnionApplyForJoinEffects>::SessionReport,
            <Effects as UnionInviteEffects>::SessionReport,
        >,
        PlayerInviteFactionBlock<
            <Effects as ConfederationCreationEffects>::SessionBlock,
            <Effects as UnionApplyForJoinEffects>::SessionBlock,
            <Effects as UnionInviteEffects>::SessionBlock,
        >,
    >
    where
        Effects: PlayerInviteFactionEffects,
    {
        let inviter_faction_id = self
            .faction_id_by_master_player(player_id)
            .map_err(PlayerInviteFactionBlock::Master)?;
        if inviter_faction_id == 0 {
            return Ok(PlayerInviteFactionOutcome::Rejected(
                PlayerInviteFactionRejection::MasterFactionNotFound,
            ));
        }

        let Some(inviter) = self.faction_by_id(inviter_faction_id) else {
            return Ok(PlayerInviteFactionOutcome::Rejected(
                PlayerInviteFactionRejection::InviterFactionMissing,
            ));
        };
        let inviter_standard_war =
            inviter.has_enemy_faction() || village_war.is_already_declared_for_war(inviter_faction_id);
        let inviter_city_war = inviter.has_city_war_enemy_faction();
        if inviter_standard_war {
            send_player_invite_faction_notice(effects, player_id, b"WS0237", b"WS0121");
            return Ok(PlayerInviteFactionOutcome::Rejected(
                PlayerInviteFactionRejection::InviterStandardWar { notice_sent: true },
            ));
        }
        if inviter_city_war {
            send_player_invite_faction_notice(effects, player_id, b"WS0238", b"WS0121");
            return Ok(PlayerInviteFactionOutcome::Rejected(
                PlayerInviteFactionRejection::InviterCityWar { notice_sent: true },
            ));
        }

        let Some(invited) = self.faction_by_id(invited_faction_id) else {
            return Ok(PlayerInviteFactionOutcome::Rejected(
                PlayerInviteFactionRejection::InvitedFactionMissing,
            ));
        };
        let invited_standard_war = invited.has_enemy_faction()
            || village_war.is_already_declared_for_war(invited_faction_id);
        let invited_city_war = invited.has_city_war_enemy_faction()
            || attack_city.is_already_declared_for_war(invited_faction_id);
        if invited_standard_war {
            send_player_invite_faction_notice(effects, player_id, b"WS0239", b"WS0121");
            return Ok(PlayerInviteFactionOutcome::Rejected(
                PlayerInviteFactionRejection::InvitedStandardWar { notice_sent: true },
            ));
        }
        if invited_city_war {
            send_player_invite_faction_notice(effects, player_id, b"WS0240", b"WS0121");
            return Ok(PlayerInviteFactionOutcome::Rejected(
                PlayerInviteFactionRejection::InvitedCityWar { notice_sent: true },
            ));
        }

        let inviter_union_id = match self.is_free_faction(inviter_faction_id) {
            FreeFactionLookup::NoUnion => 0,
            FreeFactionLookup::Union(union_id) => union_id,
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                return Err(PlayerInviteFactionBlock::InviterMembership { map_key });
            }
        };
        let invited_union_id = match self.is_free_faction(invited_faction_id) {
            FreeFactionLookup::NoUnion => 0,
            FreeFactionLookup::Union(union_id) => union_id,
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                return Err(PlayerInviteFactionBlock::InvitedMembership { map_key });
            }
        };

        let action = match (inviter_union_id, invited_union_id) {
            (0, 0) => PlayerInviteFactionAction::Create(
                self.create_confederation(
                    game,
                    inviter_faction_id,
                    invited_faction_id,
                    b"",
                    effects,
                )
                .map_err(PlayerInviteFactionBlock::Creation)?,
            ),
            (0, invited_union_id) => {
                PlayerInviteFactionAction::ApplyToInvitedUnion(
                    self.apply_for_named_union_join(
                        game,
                        invited_union_id,
                        inviter_faction_id,
                        0,
                        0,
                        effects,
                    )
                    .map_err(PlayerInviteFactionBlock::Application)?,
                )
            }
            (inviter_union_id, 0) => PlayerInviteFactionAction::InviteToInviterUnion(
                self.invite_faction_to_union(
                    game,
                    inviter_union_id,
                    inviter_faction_id,
                    invited_faction_id,
                    effects,
                )
                .map_err(PlayerInviteFactionBlock::Invitation)?,
            ),
            _ => {
                send_player_invite_faction_notice(effects, player_id, b"WS0241", b"WS0193");
                return Ok(PlayerInviteFactionOutcome::Rejected(
                    PlayerInviteFactionRejection::BothAlreadyInUnion { notice_sent: true },
                ));
            }
        };
        Ok(PlayerInviteFactionOutcome::Dispatched {
            inviter_faction_id,
            invited_faction_id,
            action,
        })
    }

    /// Выполняет exact prefix до синхронного persistent player-name lookup.
    pub(crate) fn prepare_faction_creation<Effects>(
        &mut self,
        game: &CGame,
        player_id: i32,
        faction_name: &mut Vec<u8>,
        effects: &mut Effects,
    ) -> Result<FactionCreationPreparation, FactionCreationBlock>
    where
        Effects: FactionCreationEffects,
    {
        match self.is_free_player(player_id) {
            FreePlayerLookup::Faction(_) => {
                return Ok(FactionCreationPreparation::Rejected(
                    FactionCreationRejection::PlayerAlreadyInFaction,
                ));
            }
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return Err(FactionCreationBlock::PlayerMembership { map_key });
            }
            FreePlayerLookup::NoFaction => {}
        }

        if !effects.check_invalid_organizing_string(faction_name, false) {
            let second_text = effects.world_string(b"WS0193");
            let first_text = effects.world_string(b"WS0194");
            effects.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text: legacy_c_string_prefix(&first_text),
                second_text: legacy_c_string_prefix(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionCreationPreparation::Rejected(
                FactionCreationRejection::InvalidName { notice_sent: true },
            ));
        }

        let name = legacy_c_string_prefix(faction_name);
        let name_exists = game
            .creation_player_by_name(name)
            .map_err(FactionCreationBlock::CreationPlayerName)?
            .is_some()
            || game
                .is_name_exist_in_map_player(name)
                .map_err(FactionCreationBlock::MapPlayerName)?
            || game
                .is_name_exist_in_db_creation(name)
                .map_err(FactionCreationBlock::DbCreationPlayerName)?
            || game
                .is_name_exist_in_db_data(name)
                .map_err(FactionCreationBlock::DbDataPlayerName)?;
        if name_exists {
            return Ok(FactionCreationPreparation::Rejected(
                FactionCreationRejection::NameExists,
            ));
        }
        Ok(FactionCreationPreparation::ReadyForPersistentLookup)
    }

    /// Завершает `CreateFaction` после exact-position persistent lookup.
    #[allow(
        clippy::too_many_arguments,
        reason = "аргументы сохраняют исходную CreateFaction boundary"
    )]
    pub(crate) fn finish_faction_creation<Effects>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        player_id: i32,
        _reserved: i32,
        established_time: TagTimeValue,
        faction_name: &[u8],
        country: u8,
        persistent_name_exists: bool,
        effects: &mut Effects,
    ) -> Result<FactionCreationOutcome, FactionCreationBlock>
    where
        Effects: FactionCreationEffects,
    {
        let name = legacy_c_string_prefix(faction_name);
        if persistent_name_exists
            || self
                .organizing_by_name(name)
                .map_err(FactionCreationBlock::OrganizingName)?
                .is_some()
        {
            return Ok(FactionCreationOutcome::Rejected(
                FactionCreationRejection::NameExists,
            ));
        }

        let application_removals = match self.remove_person_from_apply_faction_list(
            game,
            player_id,
        ) {
            RemovePersonFromApplyFactionListOutcome::Completed { removals } => removals,
            RemovePersonFromApplyFactionListOutcome::BlockedNullFaction {
                map_key,
                completed_removals,
            } => {
                return Err(FactionCreationBlock::ApplicationRemoval {
                    map_key,
                    completed_removals,
                });
            }
        };

        let faction_id = self.generate_db_organizing_id();
        if self.factions.contains_key(&faction_id) {
            return Err(FactionCreationBlock::GeneratedIdCollision { faction_id });
        }
        let master_title = effects.world_string(b"WS0157");
        let mut faction = CFaction::for_creation(
            faction_id,
            player_id,
            established_time,
            name,
            &master_title,
            game,
            parameters,
        )
        .map_err(FactionCreationBlock::Initial)?;
        faction
            .set_country(country)
            .expect("for_creation материализует полный base property");
        self.factions.insert(faction_id, Some(Box::new(faction)));
        self.factions
            .get_mut(&faction_id)
            .and_then(Option::as_deref_mut)
            .expect("только что вставленный faction owner остаётся в map")
            .set_change_data(0x0F);

        let other_faction_updates = self
            .update_other_faction_info_to_client(
                game,
                faction_id,
                name,
                EOperator::Add,
            )
            .map_err(|source| FactionCreationBlock::OtherFactionUpdate {
                faction_id,
                source,
            })?;

        let mut log_written = false;
        if effects.faction_create_log_enabled()
            && let Some(player) = game.online_player_by_id(player_id as u32)
        {
            effects.write_faction_create_log(
                faction_id,
                name,
                player_id,
                legacy_c_string_prefix(player.get_name()),
            );
            log_written = true;
        }

        Ok(FactionCreationOutcome::Created(FactionCreationReport {
            faction_id,
            application_removals,
            other_faction_updates,
            log_written,
        }))
    }

    /// Синхронная exact composition для владельцев с готовым name-index.
    #[allow(
        clippy::too_many_arguments,
        reason = "аргументы сохраняют исходную CreateFaction boundary"
    )]
    pub(crate) fn create_faction<Effects>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        player_id: i32,
        reserved: i32,
        established_time: TagTimeValue,
        faction_name: &mut Vec<u8>,
        country: u8,
        effects: &mut Effects,
    ) -> Result<FactionCreationOutcome, FactionCreationBlock>
    where
        Effects: FactionCreationEffects,
    {
        match self.prepare_faction_creation(game, player_id, faction_name, effects)? {
            FactionCreationPreparation::Rejected(reason) => {
                Ok(FactionCreationOutcome::Rejected(reason))
            }
            FactionCreationPreparation::ReadyForPersistentLookup => {
                let persistent_name_exists = effects
                    .persistent_player_name_exists(legacy_c_string_prefix(faction_name));
                self.finish_faction_creation(
                    game,
                    parameters,
                    player_id,
                    reserved,
                    established_time,
                    faction_name,
                    country,
                    persistent_name_exists,
                    effects,
                )
            }
        }
    }

    /// Проверяет две faction и запускает exact подтверждение учреждения союза.
    ///
    /// Переданное старому API имя намеренно не используется: EXE копирует в
    /// endpoint имя первой faction через virtual slot `+0x60`.
    pub(crate) fn create_confederation<Effects>(
        &mut self,
        game: &CGame,
        first_faction_id: i32,
        second_faction_id: i32,
        _requested_name: &[u8],
        effects: &mut Effects,
    ) -> Result<
        ConfederationCreationStartOutcome<Effects::SessionReport>,
        ConfederationCreationStartBlock<Effects::SessionBlock>,
    >
    where
        Effects: ConfederationCreationEffects,
    {
        if self.is_union_application_reserved(first_faction_id) {
            return Ok(ConfederationCreationStartOutcome::Rejected(
                ConfederationCreationRejection::FirstFactionReserved,
            ));
        }
        if self.is_union_application_reserved(second_faction_id) {
            return Ok(ConfederationCreationStartOutcome::Rejected(
                ConfederationCreationRejection::SecondFactionReserved,
            ));
        }
        if first_faction_id == 0 || second_faction_id == 0 {
            return Ok(ConfederationCreationStartOutcome::Rejected(
                ConfederationCreationRejection::ZeroFactionId,
            ));
        }

        let Some(first_faction) = self.faction_by_id(first_faction_id) else {
            return Ok(ConfederationCreationStartOutcome::Rejected(
                ConfederationCreationRejection::FirstFactionMissing,
            ));
        };
        let first_player_id = first_faction.master_id().ok_or(
            ConfederationCreationStartBlock::MissingFirstMaster {
                faction_id: first_faction_id,
            },
        )?;
        let creation_enabled = first_faction.is_create_union_function().ok_or(
            ConfederationCreationStartBlock::MissingFirstProperty {
                faction_id: first_faction_id,
            },
        )?;
        let union_name = first_faction.name().to_vec();

        let Some(second_faction) = self.faction_by_id(second_faction_id) else {
            return Ok(ConfederationCreationStartOutcome::Rejected(
                ConfederationCreationRejection::SecondFactionMissing,
            ));
        };
        let second_player_id = second_faction.master_id().ok_or(
            ConfederationCreationStartBlock::MissingSecondMaster {
                faction_id: second_faction_id,
            },
        )?;

        if !creation_enabled {
            send_confederation_creation_notice(effects, first_player_id, b"WS0242");
            return Ok(ConfederationCreationStartOutcome::Rejected(
                ConfederationCreationRejection::CreationFunctionDisabled { notice_sent: true },
            ));
        }

        let Some(second_player) = game.online_player_by_id(second_player_id as u32) else {
            send_confederation_creation_notice(effects, first_player_id, b"WS0243");
            return Ok(ConfederationCreationStartOutcome::Rejected(
                ConfederationCreationRejection::SecondMasterOffline { notice_sent: true },
            ));
        };
        let net_exchange_id = second_player.get_net_exchange_id();
        self.push_to_establishment_list(first_faction_id);
        self.push_to_establishment_list(second_faction_id);
        let session = match effects.begin_confederation_creation_session(
            ConfederationCreationSessionRequest {
                first_player_id,
                second_player_id,
                first_faction_id,
                second_faction_id,
                requested_session_id: net_exchange_id,
                timeout_ticks: 0x3E8,
                union_name,
            },
        ) {
            Ok(session) => session,
            Err(source) => {
                // CreateSession/allocation failure старого кода приводил к
                // null dereference. Это внутренний UB, не wire-семантика.
                let first_reservation_removed =
                    self.remove_from_establishment_list(first_faction_id);
                let second_reservation_removed =
                    self.remove_from_establishment_list(second_faction_id);
                return Err(ConfederationCreationStartBlock::Session {
                    first_faction_id,
                    second_faction_id,
                    first_reservation_removed,
                    second_reservation_removed,
                    source,
                });
            }
        };

        Ok(ConfederationCreationStartOutcome::Started {
            first_player_id,
            second_player_id,
            first_faction_id,
            second_faction_id,
            net_exchange_id,
            session,
        })
    }

    /// Завершает exact `CreateUnion::OnAsyncCallback` в organizing owner-е.
    #[allow(
        clippy::too_many_arguments,
        reason = "terminal хранит exact поля локального C++ callback-owner-а"
    )]
    pub(crate) fn finish_confederation_creation<Effects>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        first_player_id: i32,
        second_player_id: i32,
        first_faction_id: i32,
        second_faction_id: i32,
        union_name: &[u8],
        terminal: ConfederationCreationTerminal,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<ConfederationCreationCallbackReport, ConfederationCreationCallbackBlock>
    where
        Effects: UnionAddFactionEffects,
    {
        let mut report = ConfederationCreationCallbackReport {
            terminal,
            rejection_notice_sent: false,
            first_faction_found: false,
            second_faction_found: false,
            union_id: None,
            initial: None,
            first_faction_notice: None,
            second_faction_addition: None,
            first_superior_assigned: None,
            second_superior_assigned: None,
            first_owned_city_refresh: None,
            second_owned_city_refresh: None,
            first_player_refresh: None,
            second_player_refresh: None,
            first_player_snapshot: None,
            second_player_snapshot: None,
            repeated_first_city_refreshes: Vec::new(),
            first_reservation_removed: false,
            second_reservation_removed: false,
        };

        if terminal == ConfederationCreationTerminal::Denied {
            let second_text = effects.world_string(b"WS0193");
            let first_text = effects.world_string(b"WS0245");
            effects.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: first_player_id,
                first_text: legacy_c_string_prefix(&first_text),
                second_text: legacy_c_string_prefix(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            report.rejection_notice_sent = true;
        }

        if terminal != ConfederationCreationTerminal::Approved {
            self.finish_confederation_creation_reservations(
                first_faction_id,
                second_faction_id,
                &mut report,
            );
            return Ok(report);
        }

        report.first_faction_found = self.faction_by_id(first_faction_id).is_some();
        report.second_faction_found = self.faction_by_id(second_faction_id).is_some();
        if !report.first_faction_found || !report.second_faction_found {
            self.finish_confederation_creation_reservations(
                first_faction_id,
                second_faction_id,
                &mut report,
            );
            return Ok(report);
        }

        let union_id = self.generate_db_organizing_id();
        report.union_id = Some(union_id);
        let (mut union, initial) = match CUnion::from_live_state(
            union_id,
            first_faction_id,
            union_name.to_vec(),
            None,
            self,
            parameters,
            game,
            update_player,
        ) {
            Ok(result) => result,
            Err((_union, source)) => {
                self.finish_confederation_creation_reservations(
                    first_faction_id,
                    second_faction_id,
                    &mut report,
                );
                return Err(ConfederationCreationCallbackBlock::Initial { report, source });
            }
        };
        report.initial = Some(initial);

        let second_text = effects.world_string(b"WS0119");
        let first_text = effects.world_string(b"WS0244");
        report.first_faction_notice = self.faction_by_id(first_faction_id).map(|faction| {
            faction.send_info_to_all_members_with_color(
                legacy_c_string_prefix(&first_text),
                legacy_c_string_prefix(&second_text),
                -1,
                10_000,
                |request| effects.send_organizing_info(request),
            )
        });

        report.second_faction_addition = match union.add_faction(
            second_faction_id,
            self,
            effects,
            parameters,
            game,
            update_player,
        ) {
            Ok(outcome) => Some(outcome),
            Err(source) => {
                self.finish_confederation_creation_reservations(
                    first_faction_id,
                    second_faction_id,
                    &mut report,
                );
                return Err(ConfederationCreationCallbackBlock::AddSecondFaction {
                    report,
                    source,
                });
            }
        };
        self.confederations.insert(union_id, Some(Box::new(union)));

        report.first_superior_assigned = Some(match self
            .faction_by_id_mut(first_faction_id)
            .expect("обе faction проверены перед созданием union")
            .set_superior_organizing(union_id, parameters)
        {
            Ok(()) => true,
            Err(source) => {
                self.finish_confederation_creation_reservations(
                    first_faction_id,
                    second_faction_id,
                    &mut report,
                );
                return Err(ConfederationCreationCallbackBlock::FirstSuperior {
                    report,
                    source,
                });
            }
        });
        report.second_superior_assigned = Some(match self
            .faction_by_id_mut(second_faction_id)
            .expect("обе faction проверены перед созданием union")
            .set_superior_organizing(union_id, parameters)
        {
            Ok(()) => true,
            Err(source) => {
                self.finish_confederation_creation_reservations(
                    first_faction_id,
                    second_faction_id,
                    &mut report,
                );
                return Err(ConfederationCreationCallbackBlock::SecondSuperior {
                    report,
                    source,
                });
            }
        });

        report.first_owned_city_refresh = Some(match self
            .faction_by_id(first_faction_id)
            .expect("обе faction проверены перед созданием union")
            .refresh_owned_city_info(|region_id, faction_id, union_id| {
                effects.refresh_owned_city(region_id, faction_id, union_id);
            })
        {
            Ok(refresh) => refresh,
            Err(source) => {
                self.finish_confederation_creation_reservations(
                    first_faction_id,
                    second_faction_id,
                    &mut report,
                );
                return Err(
                    ConfederationCreationCallbackBlock::FirstOwnedCityRefresh {
                        report,
                        source,
                    },
                );
            }
        });
        report.second_owned_city_refresh = Some(match self
            .faction_by_id(second_faction_id)
            .expect("обе faction проверены перед созданием union")
            .refresh_owned_city_info(|region_id, faction_id, union_id| {
                effects.refresh_owned_city(region_id, faction_id, union_id);
            })
        {
            Ok(refresh) => refresh,
            Err(source) => {
                self.finish_confederation_creation_reservations(
                    first_faction_id,
                    second_faction_id,
                    &mut report,
                );
                return Err(
                    ConfederationCreationCallbackBlock::SecondOwnedCityRefresh {
                        report,
                        source,
                    },
                );
            }
        });
        report.first_player_refresh = Some(
            self.faction_by_id(first_faction_id)
                .expect("обе faction проверены перед созданием union")
                .update_player_faction_info(game, first_player_id, &mut *update_player),
        );
        report.second_player_refresh = Some(
            self.faction_by_id(second_faction_id)
                .expect("обе faction проверены перед созданием union")
                .update_player_faction_info(game, second_player_id, &mut *update_player),
        );

        report.first_player_snapshot = match self.send_created_union_snapshot(
            game,
            union_id,
            first_player_id,
        ) {
            Ok(delivery) => delivery,
            Err(source) => {
                self.finish_confederation_creation_reservations(
                    first_faction_id,
                    second_faction_id,
                    &mut report,
                );
                return Err(ConfederationCreationCallbackBlock::FirstSnapshot {
                    report,
                    source,
                });
            }
        };
        report.second_player_snapshot = match self.send_created_union_snapshot(
            game,
            union_id,
            second_player_id,
        ) {
            Ok(delivery) => delivery,
            Err(source) => {
                self.finish_confederation_creation_reservations(
                    first_faction_id,
                    second_faction_id,
                    &mut report,
                );
                return Err(ConfederationCreationCallbackBlock::SecondSnapshot {
                    report,
                    source,
                });
            }
        };

        let first_owned_cities = self
            .faction_by_id(first_faction_id)
            .expect("обе faction проверены перед созданием union")
            .owned_cities()
            .clone();
        for region_id in first_owned_cities {
            effects.refresh_owned_city(region_id, first_faction_id, union_id);
            report.repeated_first_city_refreshes.push(region_id);
        }

        self.finish_confederation_creation_reservations(
            first_faction_id,
            second_faction_id,
            &mut report,
        );
        Ok(report)
    }

    fn send_created_union_snapshot(
        &mut self,
        game: &CGame,
        union_id: i32,
        player_id: i32,
    ) -> Result<Option<ConfederationCreationSnapshotDelivery>, UnionMemberSnapshotBlock> {
        let Some(player) = game.online_player_by_id(player_id as u32) else {
            return Ok(None);
        };
        if !player.faction_data_received() {
            return Ok(None);
        }

        let game_server_id = game.game_server_number_by_player_id(player_id);
        let (factions, confederations) = (&self.factions, &mut self.confederations);
        let union = confederations
            .get_mut(&union_id)
            .and_then(Option::as_deref_mut)
            .expect("created union вставлен до client snapshot");
        let mut message = CMessage::new(UNION_INITIAL_MESSAGE_TYPE);
        message.base_mut().add_long(player_id);
        let mut snapshot = Vec::new();
        union.add_to_byte_array(&mut snapshot, &UnionFactionMapView { factions })?;
        message.base_mut().add(&snapshot);
        Ok(Some(ConfederationCreationSnapshotDelivery {
            player_id,
            game_server_id,
            result: game.send_msg_to_game_server(game_server_id, &message),
        }))
    }

    /// Удаляет первое совпадение каждого ID в машинном порядке first/second.
    /// RAW ошибочно вставляет `return` после первого erase; exact EXE
    /// `0x0043930C..0x00439388` безусловно продолжает ко второму проходу.
    fn finish_confederation_creation_reservations(
        &mut self,
        first_faction_id: i32,
        second_faction_id: i32,
        report: &mut ConfederationCreationCallbackReport,
    ) {
        report.first_reservation_removed =
            self.remove_from_establishment_list(first_faction_id);
        report.second_reservation_removed =
            self.remove_from_establishment_list(second_faction_id);
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

    /// Завершает result/timeout локального `InviteJoinConfeder`.
    #[allow(
        clippy::too_many_arguments,
        reason = "аргументы повторяют exact поля callback-owner-а"
    )]
    pub(crate) fn finish_union_invitation<Effects>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        union_id: i32,
        inviter_faction_id: i32,
        invited_faction_id: i32,
        terminal: UnionApplicationTerminal,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<
        OrganizingUnionInvitationCallbackReport,
        OrganizingUnionInvitationCallbackBlock,
    >
    where
        Effects: UnionAddFactionEffects,
    {
        let rejection_notice_sent = if terminal == UnionApplicationTerminal::Denied {
            let recipient = match self.faction_by_id(inviter_faction_id) {
                Some(faction) => match faction.player_header(self) {
                    Ok(player_id) => Some(player_id),
                    Err(source) => {
                        let establishment_reservation_removed =
                            self.remove_from_establishment_list(invited_faction_id);
                        let _ = establishment_reservation_removed;
                        return Err(
                            OrganizingUnionInvitationCallbackBlock::InviterPlayerHeader {
                                union_id,
                                inviter_faction_id,
                                invited_faction_id,
                                source,
                                rejection_notice_sent: false,
                            },
                        );
                    }
                },
                None => None,
            };
            if let Some(recipient_player_id) = recipient {
                let second_text = effects.world_string(b"WS0193");
                let first_text = effects.world_string(b"WS0245");
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
            return Ok(OrganizingUnionInvitationCallbackReport {
                union_id,
                inviter_faction_id,
                invited_faction_id,
                union_found: false,
                rejection_notice_sent,
                join: None,
                application_cleared: false,
                establishment_reservation_removed: self
                    .remove_from_establishment_list(invited_faction_id),
            });
        }

        let detached_membership_lookup =
            (terminal == UnionApplicationTerminal::Approved)
                .then(|| self.is_free_faction(invited_faction_id));
        let mut union = self
            .confederations
            .get_mut(&union_id)
            .and_then(Option::take)
            .expect("union entry и pointer проверены до временного take");
        let join = if terminal == UnionApplicationTerminal::Approved {
            self.detached_union_membership_lookup.set(
                detached_membership_lookup.map(|lookup| (invited_faction_id, lookup)),
            );
            let join_result = union.do_join(
                game,
                parameters,
                inviter_faction_id,
                invited_faction_id,
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
                    let _ = self.remove_from_establishment_list(invited_faction_id);
                    return Err(OrganizingUnionInvitationCallbackBlock::DoJoin {
                        union_id,
                        inviter_faction_id,
                        invited_faction_id,
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
            self.remove_from_establishment_list(invited_faction_id);
        Ok(OrganizingUnionInvitationCallbackReport {
            union_id,
            inviter_faction_id,
            invited_faction_id,
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
        Disband: FnMut(&mut Self, i32, i32) -> Result<bool, OrganizingDisbandBlock>,
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
            let result = disband_faction(self, master_id, faction_id).map_err(|source| {
                OrganizingRunBlock::Disband {
                    faction_id,
                    master_id,
                    source,
                }
            })?;
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

impl UnionFireOutContext for COrganizingCtrl {
    type DetachBlock = UnionMemberDetachBlock;
    type DetachOutcome = UnionMemberDetachOutcome;

    fn detach_union_member_for_fire_out(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        faction_id: i32,
    ) -> Result<Self::DetachOutcome, Self::DetachBlock> {
        self.detach_union_member(game, parameters, faction_id)
    }
}

impl UnionExitContext for COrganizingCtrl {
    type DetachBlock = UnionMemberDetachBlock;
    type DetachOutcome = UnionMemberDetachOutcome;

    fn detach_union_member_for_exit(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        faction_id: i32,
    ) -> Result<Self::DetachOutcome, Self::DetachBlock> {
        self.detach_union_member(game, parameters, faction_id)
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
        self.add_owned_city_to_faction(game, faction_id, region_id, update_player)
            .map(|outcome| outcome.is_some())
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

    fn faction_clear_city_war_enemy_factions(&mut self, faction_id: i32) -> bool {
        let Some(faction) = self.faction_by_id_mut(faction_id) else {
            return false;
        };
        faction.clear_city_war_enemy_factions();
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
            organizing.clear_owned_regions();

            for &region_id in faction.owned_cities() {
                let Some(region_type) = self.region_types.get(&region_id) else {
                    continue;
                };
                let Some(region_type) = *region_type else {
                    return Err(PlayerOrganizingUpdateError::UninitializedRegionType { region_id });
                };
                organizing.add_owned_region(region_id, region_type);
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
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:937
// RVA: 0x00033680
// ADDRESS: 00433680
// PROTOTYPE: void __thiscall DoAsyncCall(__int64 param_1, long param_2, char * param_3)
//
// IMPLEMENTED_OWNER: `CreateConfederationEndpoint::do_async_call` и
// `begin_confederation_creation_session` сохраняют exact `0x7FE16` wire-order,
// route по second player, session/cookie и terminal-разбор ответа.
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
// STATUS: VERIFIED_DISASSEMBLY, IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:451
// RVA: 0x00033D10
// ADDRESS: 00433d10
// PROTOTYPE: long __thiscall GetFactionNumber(uchar param_1)
//
// Реализовано выше как `faction_count_by_country`; exact null-skip, virtual
// country slot `+0x190`, signed map-order и wrapping count сохранены.
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddFactionListToByteArray
// STATUS: VERIFIED_DISASSEMBLY, IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:492
// RVA: 0x00033D90
// ADDRESS: 00433d90
// PROTOTYPE: void __thiscall AddFactionListToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, long param_2, uchar param_3)
//
// Реализовано выше как `faction_list_page`; exact pagination/wire/map-order
// сохранены, старый `char[256]` overflow локализован typed block-ом.
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
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// IMPLEMENTED_OWNER: `COrganizingCtrl::generate_db_organizing_id` выше.
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
// STATUS: IMPLEMENTED
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
// STATUS: IMPLEMENTED
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
// STATUS: IMPLEMENTED
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
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1414
// RVA: 0x00034520
// ADDRESS: 00434520
// PROTOTYPE: long __thiscall IsConferationMaster(long param_1)
//
// IMPLEMENTED_OWNER: `COrganizingCtrl::union_id_by_master_faction` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::FindOrgaByName
// STATUS: VERIFIED_DISASSEMBLY, IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1436
// RVA: 0x000345A0
// ADDRESS: 004345a0
// PROTOTYPE: COrganizing * __thiscall FindOrgaByName(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// Реализовано выше как `organizing_by_name`; exact map-order, faction/union
// приоритет, ASCII CRT-folding и локальные 260-byte safe-границы сохранены.
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
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:405
// RVA: 0x00034D00
// ADDRESS: 00434d00
// PROTOTYPE: bool __thiscall AddAllFactinInfoToClientByPlayerID(long param_1)
//
// IMPLEMENTED_OWNER: `add_all_faction_info_to_client_with_detached` выше.
// RAW_REFERENCE_BEGIN: сохранённая декомпиляция реализованной функции.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// RAW_REFERENCE_END

// ============================================================================
// FUNCTION: `public:_enum_eCrOrgResult___thiscall_COrganizingCtrl::CreateConfederation(long,long,std::basic_string<char,std::char_traits<char>,std::allocator<char>_>&)'::__l28::CreateUnion::CreateUnion
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:931
// RVA: 0x00034E90
// ADDRESS: 00434e90
// PROTOTYPE: undefined __thiscall CreateUnion(long param_1, long param_2, long param_3, long param_4, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_5)
//
// IMPLEMENTED_OWNER: `CreateConfederationEndpoint` владеет exact полями
// first/second player, first/second faction и копией C-string имени союза.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_enum_eCrOrgResult___thiscall_COrganizingCtrl::CreateConfederation(long,long,std::basic_string<char,std::char_traits<char>,std::allocator<char>_>&)'::__l28::CreateUnion::Release
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:934
// RVA: 0x00034EF0
// ADDRESS: 00434ef0
// PROTOTYPE: void __thiscall Release(void)
//
// IMPLEMENTED_OWNER: Rust `Box`/`Arc`/`Drop` освобождают endpoint после
// terminal callback-а без ручного virtual delete и сохраняемого leak/UB.
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
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:379
// RVA: 0x00037380
// ADDRESS: 00437380
// PROTOTYPE: bool __thiscall AddFactionToClientByPlayerID(long param_1)
//
// IMPLEMENTED_OWNER: `add_faction_to_client_with_detached` выше.
// RAW_REFERENCE_BEGIN: сохранённая декомпиляция реализованной функции.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// RAW_REFERENCE_END

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
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:595
// RVA: 0x000376F0
// ADDRESS: 004376f0
// PROTOTYPE: bool __thiscall AddUnionToClientByPlayerID(long param_1)
//
// IMPLEMENTED_OWNER: `COrganizingCtrl::add_union_to_client_by_player_id` выше.
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
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1296
// RVA: 0x00037A40
// ADDRESS: 00437a40
// PROTOTYPE: void __thiscall SetEnemyFactionRelation(long param_1, long param_2)
//
// IMPLEMENTED_OWNER: `set_enemy_faction_relation` выше сохраняет две positive
// lookup-гранцы, no-op при nullable miss и ordered два virtual
// `AddCityWarEnemyOrganizing`; Rust `Result` локализует уже подтверждённую
// safe-границу formatter-а, не подменяя её чтением за old stack buffer.
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
// STATUS: VERIFIED_DISASSEMBLY, IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1514
// RVA: 0x00037B20
// ADDRESS: 00437b20
// PROTOTYPE: uchar __thiscall GetCountryByFaction(long param_1)
//
// Реализовано выше как `country_by_faction`; exact slot `+0x190`,
// positive-ID/map/null gates и нулевой miss сохранены.
//

// ============================================================================
// FUNCTION: COrganizingCtrl::AddOwnedCityToFaction
// STATUS: VERIFIED_DISASSEMBLY, IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1717
// RVA: 0x00037C20
// ADDRESS: 00437c20
// PROTOTYPE: void __thiscall AddOwnedCityToFaction(long param_1, long param_2)
//
// Реализовано выше как `add_owned_city_to_faction`; exact slot `+0x80`,
// positive-ID/map/null gates и отсутствие controller-side эффектов сохранены.
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
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:651
// RVA: 0x000381A0
// ADDRESS: 004381a0
// PROTOTYPE: eCrOrgResult __thiscall CreateFaction(long param_1, long param_2, tagTime * param_3, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_4, uchar param_5)
//
// IMPLEMENTED_OWNER: `COrganizingCtrl::create_faction` выше сохраняет exact
// lookup/mutation/publication/log order поверх безопасного faction owner-а.
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
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:867
// RVA: 0x000389D0
// ADDRESS: 004389d0
// PROTOTYPE: eCrOrgResult __thiscall CreateConfederation(long param_1, long param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3)
//
// IMPLEMENTED_OWNER: `COrganizingCtrl::create_confederation` сохраняет exact
// preflight, notice/order, first-faction name, reservation и timeout `0x3E8`;
// safe session failure очищает обе reservation вместо внутреннего UB.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_enum_eCrOrgResult___thiscall_COrganizingCtrl::CreateConfederation(long,long,std::basic_string<char,std::char_traits<char>,std::allocator<char>_>&)'::__l28::CreateUnion::OnAsyncCallback
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:953
// RVA: 0x00038D80
// ADDRESS: 00438d80
// PROTOTYPE: void __thiscall OnAsyncCallback(tagAsyncResult * param_1)
//
// сохраняет terminal/result, exact creation side effects и два независимых
// reservation erase; `send_created_union_snapshot` реализует `0x7FE04`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingCtrl::DisbandConferation
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// IMPLEMENTED_OWNER: `COrganizingCtrl::disband_confederation` выше.
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
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:1665
// RVA: 0x0003A2C0
// ADDRESS: 0043a2c0
// PROTOTYPE: int __thiscall OnDeleteRole(long param_1)
//
// IMPLEMENTED_OWNER: organizing state-machine находится выше в
// `COrganizingCtrl::on_delete_role`; DB/country prefix и wire caller-а — в
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
// fallible dispatch к подключённому `DisbandFaction` owner-у.

// ============================================================================
// FUNCTION: COrganizingCtrl::OnPlayerInviteFaction
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingctrl.cpp:790
// RVA: 0x0003A780
// ADDRESS: 0043a780
// PROTOTYPE: bool __thiscall OnPlayerInviteFaction(long param_1, long param_2)
//
// IMPLEMENTED_OWNER: `COrganizingCtrl::on_player_invite_faction` сохраняет
// exact war order/notices, две membership ветки и owner dispatch.
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
