//! Владелец faction-состояния исторического `WorldServer`.
//!
//! `CFaction::SetChangeData` RVA `0x000B4C60`, `CloneSaveData` RVA
//! `0x000C1210`, достигнутое чтение
//! `m_ChangeDataType` из `CRsFaction::SaveFaction` RVA `0x000FF070`,
//! `GetPronounceData` RVA
//! `0x000B4CA0`, `SetGoodsWarCount` RVA `0x000B4D70`,
//! `UpdatePronounceToClient` RVA `0x000B56C0`,
//! `UpdateLeaveWordToClient/EditLeaveWord` RVA `0x000B6240/0x000B6790`,
//! `LeaveWord/LoadLeavewords` RVA `0x000BCA40/0x000BD3F0`,
//! `UpdateAllApplyMemberToClient/UpdateApplyMemberToClient/RemoveApplyMember`
//! RVA `0x000B60D0/0x000BEC80/0x000B9F50`,
//! `ApplyForJoin/DoJoin/Exit/FireOut/DubAndSetJobLvl/EndueRightToMember/
//! AbolishRightToMember` RVA
//! `0x000BE520/0x000BEE40/0x000BAAB0/0x000BB140/0x000BB8A0/
//! 0x000BBFB0/0x000BC500`,
//! `SendInfoToAllMember/UpdateOtherFacInfoToClient` RVA
//! `0x000B5890/0x000B58F0`,
//! `Talk` RVA `0x000B5A50`,
//! `DeleteOrgaToClient` RVA `0x000B8A20`,
//! `Pronounce` RVA `0x000B8DC0`,
//! feature-setter-ы `SetLWFunction/SetPronounceFun/SetEndueRightFun/
//! SetJoinVillageWarFun/SetJoinCityWarFun/SetCreateUnionFun` RVA
//! `0x000B7030/0x000B7400/0x000B77D0/0x000B7BA0/0x000B7F70/0x000B8340`,
//! `AddMembersToByteArray` RVA `0x000B53D0`, PDB-inline
//! `AddApplyPersonsToByteArray/AddLeaveWordsToByteArray` RVA
//! `0x000B5D30/0x000B5DD0`,
//! `GetMembers/GetMemberNum` RVA `0x000BD7C0/0x000BD830` и
//! `CFaction::IsMember` RVA `0x000BD840` и
//! `UpdateMemberInfoToClient` RVA `0x000BA7C0`, а также
//! оба `CheckOperValidate` RVA `0x000B4CE0/0x000C17F0`,
//! `OnMemberExitGame` RVA `0x000B64D0`,
//! `ClearApplyList/IsInApplyMembers` RVA `0x000B6450/0x000B6760`,
//! free owner `GoodsWarCheckforFaction` RVA `0x000B5070`,
//! `InitialPropertyByLvl` RVA `0x000B4BA0`,
//! `AddEnemyFactionsToByteArray/AddCityWarEnemyFactionsToByteArray` RVA
//! `0x000B5E40/0x000B5ED0`,
//! `ClearEnemyFation/ClearCityWarEnemyFation` RVA `0x000B6030/0x000B6080`,
//! `IsHaveEnymyFaction/IsHaveCityEnemyFaction` RVA `0x000B50F0/0x000B5100`,
//! `ClearOwnedCity/DelOwnedCity` RVA `0x000B54C0/0x000B5F60`,
//! `RefreshOwnCityInfo` RVA `0x000B5570`,
//! обе перегрузки `GetMemberList` RVA `0x000B5530/0x000B9E60`,
//! оба `AddOwnedCity` RVA `0x000B9DE0/0x000BA650` и `SetOwnedCity` RVA
//! `0x000C16A0`,
//! `SetSuperiorOrganizing` RVA `0x000B5110`,
//! `IsOwnedCity` RVA `0x000B5490`, `GetOwnedCities` RVA `0x000BD7D0`,
//! `AddOwnedCitiesToByteArray` RVA `0x000BDD70`,
//! `UpdateExpToClient/SetExp` RVA `0x000B55B0/0x000B61F0`,
//! `OnMemberLvlChange` RVA `0x000B6590`,
//! базовые query-owner-ы `GetID/GetName/GetMasterID/GetLvl/GetExp/GetCountry`
//! RVA `0x000BD700..0x000BD760`, `GetEstablishedTime`/`IsLWFunction`/
//! `IsCreateUnionFun` RVA `0x000BD790..0x000BD7B0`, victor/delete getter-ы
//! RVA `0x000BD800..0x000BD820` и member query-owner-ы
//! `GetTitleByID/GetJobLvlByID` RVA `0x000BD870/0x000BD910`,
//! прямые mutator-owner-ы `SetPermitDemise/SetCountry` RVA
//! `0x000BD750/0x000BD770`, enemy-changed setter-ы RVA
//! `0x000BD7E0/0x000BD7F0` и `SetName` RVA `0x000C1B90`,
//! `DelMember` RVA `0x000B9EF0`,
//! `UpdatePropertyToClient` RVA `0x000B9FB0`,
//! `UpdateEnemyFactionToClient/UpdateCityWarEnemyFactionToClient` RVA
//! `0x000BA0F0/0x000BA210`,
//! `UpdateEnemyFaction/UpdateCityWarEnemyFaction` RVA `0x000B4E30/0x000B4E60`,
//! `Add/DelEnemyOrganizing` RVA `0x000BDE80/0x000BE030` и
//! `Add/DelCityWarEnemyOrganizing` RVA `0x000BE1E0/0x000BE390`,
//! `UpdateOwnedCityToClient` RVA `0x000C0BF0`,
//! `UpdatePlayerFactionInfo` RVA `0x000B5820`,
//! `IncMaxNumber` RVA `0x000B4E90`, by-value no-op `GetEnemyList` RVA
//! `0x000B5FF0`, `SetParam` RVA `0x000BA310`, `GetPlayerHeader` RVA
//! `0x000C0D10` и compiler-owned destructor RVA `0x000BD590`,
//! `AddDefence/Offense/VillageWarVictorCounts` RVA
//! `0x000BA3B0/0x000BA3D0/0x000BA3F0`,
//! `ReInitialPropertyByLvl` RVA `0x000BA630`,
//! `IsUsingPV/SetMemPV/AbolishMemPV` RVA
//! `0x000BA700/0x000BA760/0x000C1DD0`,
//! set-copy getter-ы `GetEnemyList/GetCityWarEnemyList` RVA
//! `0x000BA6A0/0x000BA6D0` и legacy `IsEnemyFaction` RVA `0x000C1830`,
//! `IsSuperiorOrganizing` RVA `0x000BD780`, `IsMaster` RVA `0x000C1EE0` и
//! `SetIsPermit/OnMemberEnterGame` RVA `0x000C09A0/0x000C0A10` —
//! `IMPLEMENTED`; спорные ключи и порядок side effect имеют статус
//! `VERIFIED_DISASSEMBLY`.
//! `OnMemberPosChange` RVA `0x000C0B40` — `IMPLEMENTED` и
//! `VERIFIED_DISASSEMBLY`.
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:185,242,243`
//! и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:265,344,1388,2519,2561`.
//!
//! PDB задаёт `m_lID` как signed `long` по offset `+0x4`, а ordered
//! `std::map<long, COrganizing::tagMemInfo> m_Members` — по `+0x28` старого
//! `CFaction`. Оба конструктора создают пустой map; public constructor RVA
//! `0x000C0EE0` до этого принимает и сохраняет faction ID. Rust partial-owner
//! хранит только эти достигнутые поля: `BTreeMap<i32, TagMemInfo>` заменяет
//! MSVC tree-node, sentinel, allocator и ручной cleanup, сохраняя signed
//! numeric порядок. Rust-layout не объявляется копией старого ABI, а полного
//! `CFaction` constructor-а до остальных полей не существует.
//! `InitialPropertyByLvl` принимает восстановленный `COrganizingParam` явно,
//! меняет шесть permission-флагов и maximum до проверки level-record, а
//! upgrade experience — только после неё. Поэтому отсутствующий уровень
//! сохраняет уже выполненный prefix и возвращает старый `false`; отсутствующий
//! live property остаётся отдельной safe-границей узкого Rust-owner-а.
//! `SetSuperiorOrganizing` так же принимает параметры явно. Машинный проход
//! `0x004B5110..0x004B5167` подтверждает, что union ID меняется первым, positive
//! ID только отменяет положительный countdown, а negative/zero ID сравнивает
//! member-count с порогом через unsigned `JNC`. При неполном live-state уже
//! выполненная смена union ID не откатывается.
//! `DelMember` отклоняет master до erase, после erase использует новый unsigned
//! member-count и запускает countdown только для faction без union. Exact
//! `0x004B9EF0..0x004B9F46` подтверждает порядок и unsigned `JNC`.
//! `UpdatePropertyToClient` публикует message `0x7FE0B`: recipient ID, затем
//! все `0x38` байт property вместе с padding. Получатели обходятся по signed
//! member key и допускаются только при online-owner, ненулевом GameServer ID и
//! уже выставленном `m_bGetFactionData`; результат каждого send игнорировался.
//! Три victor-counter owner-а машинно подтверждают единый порядок: wrapping
//! 32-битный `ADD`, property-send, затем dirty-bit `1`.
//! Permission owner-ы используют player ID как map key и PDB enum `0..10` как
//! индекс. `SetMemPV` в EXE единственный не проверял индекс и мог писать за
//! `listPV`; safe Rust явно отклоняет недопустимое значение. Это исправление
//! внутреннего memory bug, а не новая Miracle-семантика допустимых прав.
//! `SetParam` сравнивает полные legacy C-string ключи `Level`/`Experience`.
//! Level выше `12` выходит без эффектов; после `SetLvl` отсутствующая level-
//! запись сохраняет уже выполненный prefix и также выходит. Любой иной
//! нормальный путь, включая неизвестный ключ, публикует property, обновляет
//! online member-ов и запрашивает dirty-bit `1` именно в таком порядке.
//! `GetPlayerHeader` при положительном union ID использует nullable lookup и
//! только найденному `CUnion` делегирует header; miss возвращает собственного
//! master-а. Найденный, но внутренне разорванный союз может вернуть `0` и не
//! запускает faction fallback.
//! `IncMaxNumber` машинно состоит из `true; ret 4`. By-value `GetEnemyList` не
//! читает receiver или элементы и только уничтожает временный `std::list`;
//! отдельной игровой операции в Rust нет, а владение временным значением и
//! cleanup обеспечивает обычный `Drop`. По той же причине destructor не
//! получает ручной `Drop`: он выполнял только compiler/STL cleanup уже
//! представленных `Vec`/`String`/`BTree*`/`VecDeque` полей.
//! Apply-list clear проверяет право `ConMem = 3`, при отказе не меняет set и
//! возвращает `false`. Membership exact ASM ищет входной player ID, а не
//! ошибочную подстановку this в декомпиляте, и при hit возвращает faction ID.
//! Трёхаргументный `CheckOperValidate` проверяет право requester и запрещает
//! ему управлять target с тем же правом, кроме случая requester-master; exact
//! ASM подтверждает, что финальный `IsMaster` получает requester.
//! Enemy-update сообщения передают полный set, а не delta: `0x7FE11/0x7FE12`,
//! recipient ID, 32-битный count и signed IDs в tree-order. Объявленные
//! `enemy_id/operator` исходные функции не читали и в Rust-интерфейс не входят.
//! Standard refresh сначала безусловно ставит enemy-changed byte в `1`, затем
//! публикует `0x7FE11` и обновляет online player state. City-war refresh делает
//! `0x7FE12` и player update только при changed byte ровно `1`, не очищая его.
//! Exact ASM подтверждает порядок; неизвестный partial byte не превращается в
//! выдуманный `false`, а возвращается отдельным typed outcome.
//! Standard enemy add/del меняют set только при фактической вставке/наличии и
//! вообще не трогают changed byte. City-war add при новой записи ставит changed
//! byte перед вставкой; city-war del безусловно вызывает erase и не ставит
//! changed byte. Последний также пытается написать `WS0161` даже при miss,
//! тогда как остальные три логируют только фактическую мутацию. War-логи идут
//! после мутации, разрешают только положительный organizing ID и живой pointer,
//! используют `WS0158/WS0159/WS0160/WS0161`; delete-формы получают оставшийся
//! 32-битный count третьим аргументом. Exact ASM `0x004BDE80..0x004BE511`
//! подтверждает эти асимметрии и порядок. Локализация и запись в `war` оставлены
//! тонкому контексту, а небезопасный `_sprintf(char[256])` заменён typed
//! границей без воспроизведения переполнения.
//! Value-getter-ы обоих enemy-set создают независимые копии. Clear всегда
//! очищает set, но выставляет соответствующий changed-флаг только если до
//! очистки он был непустым. Отдельный virtual `IsEnemyFaction` в этой версии
//! является подтверждённым stub и всегда возвращает `0`; membership дают
//! другие owner-ы, поэтому эти контракты намеренно не объединены.
//! Owned-city wire сохраняет list-order: 32-битный count, затем для каждого
//! узла signed city ID и NUL-terminated region name. Missing key и null
//! `pRegion` дают пустую строку. Exact ASM подтверждает lookup по текущему city
//! ID и локальный `char[256]`; переполнение `strcpy` заменено typed-границей,
//! сохраняя уже дописанный prefix результата.
//! Owned-city update `0x7FE13` повторяет тот же full snapshot для каждого
//! готового member recipient после исходного предварительного, но не
//! использованного построения. Оба объявленных delta-аргумента не читаются.
//! Recipient filter и игнорирование send-result совпадают с property/enemy
//! update; Rust возвращает результаты и уже выполненный prefix явно.
//! `UpdatePlayerFactionInfo(0)` обходит member keys в signed порядке и вызывает
//! player-owner только для online entries; ненулевой ID проверяется один раз.
//! Exact ASM подтверждает обе ветви. Rust принимает update callback явно,
//! отделяя faction dispatch от ещё самостоятельного `CPlayer` owner-а.
//! Owned-city mutator-ы сохраняют list, а не set: одиночный add подавляет любой
//! существующий duplicate, range-add дописывает значения и выполняет только
//! adjacent `list::unique`, delete снимает первое совпадение. Clear/range-add/
//! delete всегда делают `0x7FE13`, затем player refresh; single-add — только
//! после фактической вставки, set — только `0x7FE13`. Exact ASM подтверждает,
//! что delete при miss всё равно публикует и возвращает `true`.
//! `RefreshOwnCityInfo` проходит тот же list-order и вызывает внешний
//! `RefreshOwnedCityOrg(region, faction, union)` для каждого элемента. Пустой
//! список не читает property; при непустом точный union ID берётся из
//! `tagFacBaseProperty +0x18`. ASM `0x004B5570..0x004B55AB` подтверждает порядок
//! чтений и трёх аргументов; singleton/game plumbing заменён узким callback-ом.
//! Experience-update `0x7FE14` получает только contributor либо master и несёт
//! recipient/current/upgrade exp. `SetExp` ставит dirty-bit до этой рассылки.
//! Простые query-owner-ы возвращают достигнутые scalar/property/member поля
//! без side effect. Для ещё не назначенного partial state Rust возвращает
//! `Option`, а bounded C-строка title не воспроизводит старое чтение за массивом.
//! Прямые setter-ы сохраняют отсутствие side effect: country меняет ровно один
//! property-byte, name копируется byte-exact и по старому контракту всегда
//! сообщает успех. Три transient bool до первого достигнутого присваивания
//! остаются `Option`, потому что один constructor сам их не инициализировал.
//! `SetIsPermit` получает union lookup явно, но вызывает его только после
//! master-gate и только с текущим `lConfederationID`. Exact ASM подтверждает
//! последующий порядок: запись permit-byte, полная property-рассылка, dirty `1`.
//! Достигнутый `SetPlayerOrganizing` дополнительно читает `m_strName`,
//! `m_lMastterID`, `m_Property.lLvl/lExp`, `m_OwnedCities` и два enemy-set.
//! Коллекции, которые constructor действительно создавал пустыми, хранятся
//! пустыми; ещё не назначенные narrow state scalar представлены `Option`, а не
//! выдуманным нулём. Member title/contribute берутся из того же `tagMemInfo`.
//!
//! Достигнутые save-поля расширяют partial-owner без притворного старого
//! layout: полный byte-exact `tagFacBaseProperty[0x38]`, established/delete
//! scalars, `m_ChangeDataType`, `m_lFactionWarWinCount` и
//! `m_szFactionWarLastWinTime`, а также ability-state `m_ApplyPersons`,
//! `m_Pronounce`, `m_LastUploadIconTime` и `m_IconData`. `SetChangeData(0)`
//! очищает всю mask, ненулевое значение добавляет отсутствующие bits через OR.
//! `SetGoodsWarCount` сначала
//! снимает один Windows-local wall-clock, пишет строку без leading zeroes,
//! ставит bit `1`, затем сохраняет `0` для входа `<= 0` либо сам положительный
//! signed `long`. Последняя нормализация подтверждена exact диапазоном
//! `0x004B4DC5..0x004B4DDF`; после ответа reverse прекращён.
//! `GoodsWarCheckforFaction` снимает один local-time snapshot только для
//! ненулевой faction и допускает ровно субботнее окно 19:30–21:10 включительно.
//! Лишь затем он передаёт faction ID в GoodsWar membership lookup. Exact ASM
//! `0x004B5074..0x004B50DA` подтверждает short-circuit и обе границы; `chrono`
//! заменяет только Win32 `GetLocalTime`, а сам lookup остаётся явным контекстом.
//!
//! Для достигнутого `SaveAbility` наблюдаемы только signed keys ordered map
//! `m_ApplyPersons`, а `AddApplyPersonsToByteArray` дополнительно подтверждает
//! полный PDB-value: `lID, strName[20], lOccu, lLvl`. Поэтому owner хранит
//! `BTreeMap<i32, TagApplyPerson>`, не перенося MSVC tree-layout.
//! PDB и `UpdatePronounceToClient` подтверждают полный `tagPronounceWord`:
//! `lPlayerID +0x00`, `strName[20] +0x04`, `tagTime +0x18` и
//! `strContent[2048] +0x28`, общий размер `0x828`. `GetPronounceData` буквально
//! дописывает эти bytes. `Vec<u8>` и
//! `TagTimeValue` заменяют только STL-vector и Windows `SYSTEMTIME`-совместимое
//! значение значка. Оба конструктора создавали пустые map/vector, а достигнутый
//! `Initial` обнулял весь pronounce-блок; тот же достигнутый baseline задаёт
//! `with_reached_member_state`.
//! `CloneSaveData` возвращает null при нулевой mask; иначе всегда переносит ID
//! и mask, а группы `property/members/leave-word/ability` копирует только для
//! bits `1/2/4/8`. Property переносится четырнадцатью DWORD вместе с тремя
//! padding-байтами после `btCountry`. Enemy-set и Goods War поля функция не
//! копирует. Ещё не материализованные scalar-поля narrow state дают локальный
//! `BLOCKED_MISSING_FACT`, а не выдуманный default.
//!
//! Достигнутый leave-word save-state хранит исходный list-order через
//! `VecDeque<TagLeaveWord>`. Три блока той же точной пары согласуются по полному
//! layout: list-copy в `CFaction` переносит `0x40` DWORD, `LoadLeaveWords`
//! заполняет `lID/lPlayerID`, `tagTime`, `strContent` и `strName`, а exact
//! `SaveLeaveWords` читает time по `+0x1C` и content по `+0x2C`. Поэтому
//! `tagLeaveWord` имеет размер `0x100`: signed ID по `+0x00/+0x04`,
//! `strName[20]` по `+0x08`, 16-байтовый `tagTime` по `+0x1C` и
//! `strContent[212]` по `+0x2C`. `repr(C)` и compile-time assertions фиксируют
//! этот value-layout; partial `CFaction` по-прежнему не выдаётся за полный x86
//! object-layout. Отсутствующий NUL content локализован до старого `CheckPoint`
//! и не воспроизводится чтением за массивом.
//!
//! `GetMembers` возвращает неизменяемое заимствование map, `GetMemberNum`
//! сохраняет младший 32-битный шаблон старого `_Mysize`, а `IsMember` ищет
//! только map-key и при наличии возвращает faction ID, иначе `0`. Exact EXE
//! `0x004BD840..0x004BD865` подтверждает, что ключом служит входной signed
//! player ID; подстановка this в декомпиляте является ошибкой восстановления stack-slot.
//!
//! `AddMembersToByteArray` первым дописывает 32-битный count и затем проходит
//! map в signed-key порядке. Каждое полное `tagMemInfo` публикуется как
//! `lID, lJobLvl, strTitle\0, listPV[0x2C], lLvl, lOccu, strName\0,
//! uint(bControbute), strRegion\0, LastOnlineTime[0x10]`. Отдельная non-delete
//! проекция `UpdateMemberInfoToClient` имеет порядок
//! `strName\0, lLvl, lOccu, lJobLvl, strTitle\0, listPV[0x2C], strRegion\0,
//! uint(bControbute), LastOnlineTime[0x10]`. Полный update в non-delete ветви
//! сначала ищет target и при наличии одним снимком локального времени заменяет
//! его `LastOnlineTime`; delete не ищет target и не снимает время. Затем обе
//! ветви проходят recipients в signed map-порядке, до фильтра выполняют lookup
//! online-player и GameServer ID и требуют ненулевого player, ненулевой
//! GameServer и его `m_bGetFactionData`. Сообщение `0x7FE0D` всегда начинается
//! `recipient ID, operator, target ID`, после чего non-delete добавляет эту
//! per-member проекцию. Исходно игнорировавшиеся результаты `SendToMapID`
//! сохраняются в typed-отчёте и не останавливают следующий send. При
//! отсутствующем NUL уже выполненные предыдущие отправки возвращаются вместе с
//! локальным `BLOCKED_MISSING_FACT`; текущий неполный message не отправляется,
//! а чтение за массивом не воспроизводится.
//! `chrono::Local::now` заменяет Windows `GetLocalTime`: один полученный local
//! wall-clock раскладывается в те же восемь `u16`, включая Sunday-based
//! `wDayOfWeek` и миллисекунды, до первой recipient-операции.
//! Exact EXE `0x004BA8B1..0x004BA8CB` подтверждает испорченный raw stack-slot:
//! в `find` передаётся адрес входного target `param_1` по `[esp+0x98]`, а не
//! iterator-local. Проверка recipient-флага по `CPlayer+0x868` видна напрямую
//! в `0x004BA82D..0x004BA834`; после этих двух ответов reverse прекращён.
//!
//! `OnMemberExitGame` ищет signed player ID и только при найденном member
//! снимает первый local-time snapshot. Пустой регион определяется сравнением
//! единственного первого байта с `""`; он оставляет member без изменений и не
//! публикует update. Для непустого региона owner записывает NUL только в
//! `strRegion[0]`, сохраняя остальные 63 bytes, копирует первый snapshot и
//! вызывает virtual `UpdateMemberInfoToClient(player, OP_Update)`. Последний
//! по исходному контракту снимает второй local-time snapshot и сразу заменяет
//! первый до recipient-прохода; обе позиции сохранены буквально. Exact EXE
//! `0x004B64D7..0x004B64E9` подтверждает ключ `param_1` по `[esp+0x30]`,
//! `0x004B6501..0x004B6544` — порядок time-before-region-check, а
//! `0x004B6546..0x004B6575` — очистку одного byte, копирование времени и
//! virtual slot `+0x1B8` с оператором `2`. После этих ответов reverse
//! прекращён.
//!
//! `OnMemberEnterGame` сохраняет исходный порядок: online-player lookup,
//! member lookup, virtual `CShape::GetRegionID`, numeric `CGame::GetRegion`,
//! копирование имени найденного `CWorldRegion` во временный `char[256]`,
//! `strcmp` с member `strRegion[64]`, условный `strcpy` и virtual update с
//! `OP_Update`. Exact EXE `0x004C0A4B..0x004C0A57` подтверждает, что потерянный
//! raw key — адрес входного `player_id` по `[ebp+8]`. Диапазон
//! `0x004C0A69..0x004C0A7D` обнуляет ровно 256 bytes, а
//! `0x004C0A92..0x004C0AA9` оставляет их пустой C-строкой при отсутствующем
//! region key либо через найденный `pRegion` читает унаследованный
//! `CBaseObject::m_strName` по `+0x20`. Отсутствующий region поэтому означает
//! пустое имя, а не ранний выход. После этих конкретных ответов reverse
//! прекращён.
//!
//! При равенстве C-string member не мутируется и update не вызывается. При
//! различии копируются байты имени и один NUL; оставшийся хвост fixed field не
//! очищается. Затем существующий `UpdateMemberInfoToClient` снимает local-time
//! и выполняет ordered recipient-рассылку. Rust использует slice equality и
//! bounded copy только там, где старые записи доказанно помещаются. Nullable
//! `pRegion`, имя длиннее 255 bytes, отсутствующий NUL в member field и имя
//! длиннее 63 bytes являются четырьмя локальными `BLOCKED_MISSING_FACT`:
//! исходник соответственно разыменовывал null либо читал/писал за границей, а
//! безопасная реализация не назначает этим UB-путям no-op, fail-closed или
//! `unsafe`-результат.
//!
//! PDB связывает slots organizing-base `+0x13C/+0x140` с enter/exit-функциями
//! этого owner-а; оба slot-а теперь замкнуты. Заменённые project-блоки удалены;
//! tree traversal, allocator, local scratch и cleanup остаются ненаблюдаемой
//! STL/compiler/CRT-семантикой, выраженной `BTreeMap`, slices и `Drop`.
//!
//! `OnMemberPosChange` не ищет online-player и не сравнивает старый регион:
//! member lookup идёт по первому аргументу, region lookup — по второму, затем
//! `CWorldRegion::m_strName` копируется прямо в `strRegion[64]` и безусловно
//! публикуется `OP_Update`. Exact ASM `0x004C0B45..0x004C0BDA` подтверждает оба
//! потерянных raw stack-key, destination `tagMemInfo + 0xAC` и порядок вызова.
//! Missing/null region остаётся исходным no-op; переполнение старого `strcpy`
//! заменено typed safe-границей без частичной записи.
//!
//! Обе перегрузки `GetMemberList` сначала очищают переданный list. Перегрузка
//! `list<COrganizing*>` оставляет его пустым, а `list<long>` затем добавляет
//! signed member keys в tree-order. Exact ASM `0x004B5530..0x004B5564` и
//! `0x004B9E60..0x004B9EDE` подтверждает, что raw early-return после удаления
//! старых nodes был ошибкой декомпиляции, а не условием пропуска заполнения.
//! Apply snapshot имеет wire-порядок
//! `count, record.id, name\0, occupation, level` в signed key-order.
//! Leave-word snapshot сохраняет list-order и поля
//! `count, id, player_id, time[0x10], content\0, name\0`. Exact ASM
//! `0x004B5D30..0x004B5E3B` подтверждает offsets и порядок. Нетерминированные
//! fixed C-строки локализованы typed-ошибками после уже записанного prefix,
//! вместо исходного чтения за массивом.
//! `UpdatePronounceToClient` не читает объявленный target ID: всем online
//! member-ам с ненулевым GameServer ID и `m_bGetFactionData` отправляется
//! `0x7FE10` с `recipient, operator, pronounce.player_id, name\0, content\0,
//! time[0x10]`. Exact ASM `0x004B56C0..0x004B57DB` подтверждает framing,
//! фильтр и неиспользованный первый аргумент. Нетерминированная C-строка
//! останавливает текущий message typed-ошибкой вместо чтения за `0x828`.
//! `SendInfoToAllMember` передаёт каждому signed member key обе исходные
//! `std::string`, signed kind, жёсткие `0xFFDAEDFE/0`; объявленный unsigned
//! четвёртый аргумент не читается. `UpdateOtherFacInfoToClient` сначала копирует
//! visible C-string имени в `char[20]`, затем готовым member-ам отправляет
//! `0x7FE15`: `recipient, operator, other_faction_id, name\0`. Exact ASM
//! `0x004B5890..0x004B5A44` подтверждает оба порядка и фильтр второй функции.
//! Переполнение старого `strcpy` заменено typed-границей до recipient-прохода.
//! `Talk` проверяет только ненулевой GameServer ID — без online lookup и
//! faction-data gate — и отправляет `0x7FA02` как
//! `recipient, 400, speaker_id, first_text\0, second_text\0`. Встроенный NUL в
//! исходной `std::string` обрезает visible C-string. Exact ASM
//! `0x004B5A50..0x004B5B63` подтверждает фильтр и порядок аргументов.
//! `DeleteOrgaToClient(target)` для положительного target требует online,
//! ненулевой GameServer и faction-data flag, затем отправляет только `0x7FE03`
//! (`target, faction_id`). Для `target <= 0` сначала один раз разрешается
//! `WS0191`, затем каждый online member с ненулевым GameServer — уже без
//! faction-data gate — получает тот же delete message, после чего отдельно
//! `SendOrgaInfoToClient(WS0191, WS0119, game_server_id, 0xFFDAEDFE, 0)`.
//! `WS0119` разрешается заново после каждой отправки. Exact ASM
//! `0x004B8A20..0x004B8DB2` подтверждает ветвление и порядок side effects;
//! старое переполнение `char[100]` строкой `WS0191` заменено typed-границей.
//! `Pronounce` сначала проверяет property-флаг и право `PV_Pronounce`, затем
//! обрезает переданную `std::string&` до глобального `0x800`, присваивает ID и
//! время, копирует только C-string prefix текста, а имя заменяет лишь для
//! online-автора. Весь `tagPronounceWord` не очищается: хвосты fixed-массивов и
//! прежнее имя offline-автора сохраняются. После `UpdatePronounceToClient(0,
//! OP_Update)` всегда ставится dirty-бит `8`. Exact ASM
//! `0x004B8DC0..0x004B8F1E` подтверждает порядок, размер глобала и обе ветки;
//! две старые `strcpy`-границы заменены typed-блокировкой до изменения state.
//! Шесть feature-setter-ов при неизменном флаге являются no-op. Иначе флаг
//! меняется до локализации, затем `WS0119` и feature-specific `WS0172..WS0185`
//! разрешаются с fallback `""`, после чего всем member key без online-фильтра
//! передаётся `SendInfoToAllMember(feature, WS0119, -1, 0x87A238)`. Последний
//! аргумент тот owner не использует: фактический цвет остаётся `0xFFDAEDFE`.
//! Exact ASM `0x004B7030..0x004B86C8` подтверждает offsets, пары ID и порядок;
//! общая Rust-реализация заменяет шесть копий одной технической процедуры.
//! `UpdateLeaveWordToClient` для `OP_Delete` шлёт переданный ID, но жёсткий
//! operator `0`; для любого другого operator игнорирует переданный ID и шлёт
//! последний leave-word как `operator, id, player, name\0, content\0, time`.
//! Обе ветки используют `0x7FE0F` и общий online/GameServer/faction-data фильтр.
//! `EditLeaveWord` проверяет feature и `PV_EditLeaveWord`, ищет ID в list-order,
//! удаляет его, публикует delete и ставит dirty-бит `4`; объявленный operator
//! не читается. Exact ASM `0x004B6240..0x004B644C` и
//! `0x004B6790..0x004B6823` подтверждает порядок и донорские расхождения.
//! Чтение sentinel-а при пустом non-delete list и выход за fixed C-строки
//! заменены typed-границами.
//! `LeaveWord` проверяет feature и `PV_LeaveWord`, обрезает входную
//! `std::string&` до global `MAX_PerWordCharNum=210`, затем wrapping-увеличивает
//! общий `CGame::m_nLeaveWordID`, копирует time/player/content и только online-
//! имя. После добавления в хвост публикуется `OP_Add`, затем dirty-бит `4`.
//! `MAX_LeaveWordNum=60`; `LoadLeavewords` делает только bounded append без
//! публикации/dirty. Exact ASM `0x004BCA40..0x004BCC58` и
//! `0x004BD3F0..0x004BD470` подтверждает порядок и глобалы. В EXE unlink старых
//! nodes не уменьшает `_Mysize`, из-за чего full-list loop повреждён; стандартный
//! `VecDeque::pop_front` сохраняет intended FIFO-limit. Неинициализированное имя
//! offline-автора и переполнение `char[20]` заменены typed-блокировкой после уже
//! выделенного ID; незначимые хвосты нового fixed-record безопасно обнуляются.
//! `UpdateAllApplyMemberToClient(target)` требует только target online,
//! ненулевой GameServer и faction-data flag, затем шлёт каждый apply-person в
//! signed key-order. Точечный `UpdateApplyMemberToClient(id, operator)` создаёт
//! нулевой local-record с заданным ID, заменяет его map-value при hit и шлёт
//! только готовым member-ам с `PV_ConMem`. Оба используют `0x7FE0A` и wire
//! `recipient, operator, record.id, name\0, occupation, level`; это совпадает
//! с PDB-layout `tagApplyPerson[0x20]` и Linux-донором. `RemoveApplyMember`
//! сначала стирает map-entry, затем публикует `OP_Delete` уже как ID + пустые
//! поля, ставит
//! dirty-бит `8` и возвращает faction ID; miss возвращает `0`. Exact ASM
//! `0x004B60D0..0x004B61E2`, `0x004BEC80..0x004BEE2E` и
//! `0x004B9F50..0x004B9FA7` подтверждает framing, фильтры и side effects.
//! `ApplyForJoin` последовательно запрещает заявку при standard/village-war,
//! city-war, полном member-limit и полном apply-limit `40`. Уже состоящий во
//! faction player получает тихий `false`; затем старые заявки удаляются из
//! всех faction и только после этого требуется online-player. Новый record
//! копирует C-string имя через старый bounded `strncpy[20]`, occupation и level,
//! отправляет заявителю `WS0166/WS0119`, публикует `OP_Add`, ставит dirty `8` и
//! опционально пишет faction-log type `2`. Exact ASM
//! `0x004BE520..0x004BEC6E` подтверждает short-circuit, signed member-limit,
//! unsigned apply-limit, порядок мутаций и неиспользованные аргументы 2/3.
//! War/controller/localization/log plumbing остаётся тонким контекстом; формат
//! DB-записи может быть параметризован библиотекой без переноса старого SQL-
//! буфера. Нетерминированное `char[20]` и переполнение 256-байтового `_sprintf`
//! заменены typed-границами в точках прежнего UB с сохранением уже выполненных
//! эффектов.
//! `DoJoin` повторяет те же war-gates, но Goods War проверяет после city-war и
//! только затем право `PV_ConMem`. Найденная заявка копируется и удаляется до
//! чтения approve-флага. Нулевой флаг означает отказ заявителю и старый `true`;
//! ненулевой продолжает через повторный member-limit, глобальное снятие заявок
//! и membership-gate. Новый `tagMemInfo` получает job-level `99`, права Exit и
//! LeaveWord, title `WS0169`, входной join-time и online либо сохранённые данные
//! кандидата. До вставки идут `WS0170` всем старым членам и `WS0171` заявителю;
//! после неё — player refresh, два controller snapshot callback-а, member
//! `OP_Add`, dirty `2`, отмена disband countdown и опциональный join-log.
//! Exact ASM `0x004BEE40..0x004BF9F5` подтверждает этот порядок, два отдельных
//! controller callback-а `0x00437380/0x00434D00`, unsigned countdown threshold
//! и игнорирование их bool-return. Fixed C-буферы и старые `_sprintf/strcpy`
//! заменены bounded массивами и typed-границами; controller/war/log plumbing
//! остаётся явным тонким контекстом.
//! `Exit` использует те же standard/village и city-war gates. В отличие от
//! Linux-донора, Goods War после `WS0162/WS0121` не завершает функцию: exact
//! ASM продолжает к `PV_Exit`. После уведомления `WS0187(name)/WS0188` старым
//! членам идут `DelMember`, player refresh, delete faction-state вышедшему,
//! member `OP_Delete`, dirty `2`, опциональный quit-log type `3` и безусловное
//! удаление из Goods War. Диапазон `0x004BAAB0..0x004BB131` подтверждает этот
//! порядок и старый `true` только после полного success-prefix. Linux-донор
//! переставлял player/member update и ошибочно возвращал `false` после Goods
//! War. Форматирование, лог и Goods War plumbing выражены bounded данными и
//! узким контекстом; старые `_sprintf/strcpy` не переносятся.
//! `FireOut` в отличие от `Exit` действительно завершает Goods War gate после
//! `WS0363 ` (ID содержит trailing space). Затем он делает трёхаргументный
//! `CheckOperValidate(manager, target, PV_FireOut)`, `WS0192(name)/WS0119`
//! старым членам и эффекты в машинном порядке: `DelMember`, delete
//! faction-state, member `OP_Delete`, player refresh, dirty `2`, fire-log type `1`,
//! Goods War delete и локальное fixed-frame сообщение `0x60508` как
//! `target_id + faction_name[32]`. Exact ASM `0x004BB140..0x004BB897`
//! подтверждает порядок, framing и игнорирование send/queue результатов;
//! Linux-донор переставлял три callback-а после удаления.
//! `DubAndSetJobLvl` сначала передаёт изменяемый title в общий invalid-string
//! owner, затем проверяет `PV_DubJobLevel` и диапазон job-level `1..=99`, после
//! чего обрезает сам входной `std::string&` до 20 bytes. Title и job-level
//! меняются независимыми ветками с `WS0195/WS0196`; только title-ветка делает
//! player refresh. Даже при отсутствии обеих изменений owner всегда публикует
//! member `OP_Update`, ставит dirty `2` и при включённом логе пишет title-log;
//! old-title там остаётся пустым, если title не менялся. Exact ASM
//! `0x004BB8A0..0x004BBFA5` подтверждает этот порядок, 100-байтовые notice-
//! буферы и старый 32-байтовый old-title scratch. Rust сохраняет C-string
//! prefix/tail семантику fixed title, но UB обеих `_sprintf/strcpy` границ
//! заменяет typed progress-блокировкой; фильтр и лог остаются контекстом.
//! Выдача и отзыв прав имеют общий gate: feature `EndueRight`, управляющее
//! право `PV_EndueRor` и целевой диапазон `2..=9`. Оба owner-а вызывают
//! permission-mutator, затем безусловно member `OP_Update` и dirty `2`, после
//! чего выбирают точный notice ID (`WS0197..WS0204` либо `WS0205..WS0212`) и
//! пишут purview-log type `0/1`. Только grant `PV_ConMem=3` дополнительно
//! отправляет полный apply snapshot, причём exact ASM ставит его после member-
//! update и dirty, а не до них, как Linux-донор. Диапазоны
//! `0x004BBFB0..0x004BC4CF` и `0x004BC500..0x004BCA13` подтверждают эту
//! асимметрию, 100-байтовые notice-буферы и string-ID mapping.
//! `SetLvl` принимает только `1..=12`, первым пишет level, затем в строгом
//! порядке применяет Pronounce, LeaveWord, EndueRight, CreateUnion,
//! JoinVillageWar, JoinCityWar и наконец maximum member count. Каждый реально
//! изменившийся feature setter публикует собственный notice. `SetMaxMememberNums`
//! сначала пишет `lMaxMemberNums`, затем форматирует `WS0186(max)` в старый
//! 256-байтовый buffer и рассылает его всем member key. Exact ASM
//! `0x004B8710..0x004B893C` и `0x004B8940..0x004B8A18` подтверждает порядок;
//! typed block сохраняет уже изменённое поле вместо воспроизведения overflow.
//! `Upgrade` проверяет master-owner (`vtable +0xD8`), level `<12`, current level-record, faction
//! experience, online-player, unsigned money, master-level и optional goods
//! именно в этом порядке; отказы `WS0220..WS0223` адресуются инициатору.
//! Success сначала вызывает полный `SetLvl`, затем `SetExp`, отправляет charge
//! `0x7FE1E` (`player, signed money, goods\0`) и лишь после этого ищет record
//! нового level. Поэтому late miss возвращает старый `false`, сохраняя level,
//! experience side effects и charge. При hit пишется next upgrade-exp, идёт
//! `WS0224(level)`, property update, dirty `1`, refresh всех online members и
//! optional level-log. Exact ASM `0x004BCC60..0x004BD3EB` подтверждает порядок,
//! unsigned money compare и quirk перехода на level 12: уже изменённый level
//! заставляет `SetExp` вернуть MaximumLevel без списания faction experience.
//! Goods catalog, inventory/money и DB-log остаются узким контекстом; charge и
//! wire framing строятся существующим `CMessage`.
//! `SetControbuter` допускает только текущего master-а как requester-а, при
//! включении проверяет signed maximum contributor-ов, затем меняет target flag.
//! После мутации exact order: member `Update`, player refresh, форматированный
//! `WS0227/WS0228(target name)`, broadcast с `WS0119`, типом `-1` и цветом
//! `0x87A238`, затем dirty `2`. ASM `0x004B92F0..0x004B9509` подтверждает два
//! разных requester/target ID, 256-байтовый `_sprintf` buffer и отсутствие
//! отдельной purview-проверки. Vtable подтверждает `+0xD8 = IsMaster`, а не
//! донорский `IsMember`. Локализация и player-owner остаются контекстом;
//! нетерминированное имя и overflow заменены typed-границами после уже
//! совершённых эффектов.
//! `UploadIcon`, вопреки имени, не принимает icon bytes и не выполняет file I/O:
//! аргумент `tagTime` не читается. Master-gate (`vtable +0xD8`) и property flag дают silent miss
//! либо `WS0225/WS0119`; разрешённый interval `<1` лишь ставит dirty `8` и
//! возвращает `true`, положительный interval отправляет `WS0226(minutes)` и
//! возвращает `false`. Exact ASM `0x004B8F30..0x004B92ED` подтверждает
//! 256-байтовый `_sprintf` buffer и отсутствие иных эффектов. Локализация
//! остаётся тонким контекстом, overflow старого buffer-а — typed-границей.
//! `Disband` проверяет `CheckOperValidate(player, PV_Disband)`, отсутствие
//! superior organizing, standard/village, city/attack и Goods War именно в
//! этом порядке. War-отказы отправляют `WS0189/WS0190/ws0362` с `WS0121`, а
//! country king получает silent false. Success последовательно удаляет Goods
//! War members/count, очищает apply-persons и leave-words, затем вызывает
//! `DeleteOrgaToClient(0)`; dirty-state функция не меняет. Exact ASM
//! `0x004BFA00..0x004BFDE0` подтверждает vtable slots и порядок. Старый null
//! country dereference заменён typed-границей, STL/allocator cleanup —
//! безопасными Rust containers и `Drop`.
//! `Demise` начинает с `IsMaster(old)` и `IsMember(new)`, затем дважды вызывает
//! один и тот же `CAttackCitySys::IsAlreadyDeclarForWar`: после обычного и
//! city enemy-set. Этот наблюдаемый quirk сохранён, как и literal `"???"` для
//! Goods War. Оба игрока обязаны быть online и не иметь
//! `m_bFactionWarOperator`; новый master проходит level, transient permit и
//! country-king gates. Success меняет master ID, title/job/purview обоих
//! member-ов, публикует `Update(old)`, `Update(new)`, dirty `1`, dirty `2`,
//! refresh `0`, затем `WS0219(old,new)` с цветом `0x87A238` и optional log.
//! Exact ASM `0x004BFDE0..0x004C0874` подтверждает этот порядок, true/false и
//! размеры buffers: 256 для block notice, 100 для success. Старый
//! `map::operator[]` при отсутствующем old-master создавал запись из частично
//! неинициализированного stack-value; corrupted invariant теперь даёт typed
//! block без UB. Fixed title/name/format overflows также остаются локальными
//! typed-границами, country/war/log plumbing — узким контекстом.
//! `OperatorTax` и `OperatorCityGate` одинаково требуют owned region и
//! `CheckOperValidate(player, PV_ObtainTax/PV_OperCityGate)`, но намеренно
//! получают union authority разными путями: tax через controller-wide
//! `IsFreeFaction(GetID())`, gate через собственный
//! `m_Property.lConfederationID`. Для найденного non-null union только master
//! faction проходит дальше; отсутствующий/null union не блокирует. Exact ASM
//! `0x004C0880..0x004C0925/0x004C0930..0x004C0991` подтверждает аргументы
//! `playerID, regionID`, порядок и отсутствие side effects. Null pointer в
//! `IsFreeFaction` scan и partial property остаются typed-границами.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;
use std::mem::{offset_of, size_of};

use chrono::{Datelike, Local, Timelike};

use super::organizing::{
    EOperator, EPurview, EPurviewOwnState, MemberPurviewMutation, TagMemInfo,
    TagTimeValue, UnterminatedMemberField,
};
use super::organizingparam::COrganizingParam;
use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::worldserver::game::{
    CGame, WorldLeaveWordIdBlock, WorldLocalMessageQueueBlock, WorldRegionNameLookup,
};

const MEMBER_UPDATE_MESSAGE_TYPE: i32 = 0x7FE0D;
const FACTION_MEMBER_REMOVED_LOCAL_MESSAGE_TYPE: i32 = 0x60508;
const FACTION_UPGRADE_CHARGE_MESSAGE_TYPE: i32 = 0x7FE1E;
const APPLY_MEMBER_UPDATE_MESSAGE_TYPE: i32 = 0x7FE0A;
const MAX_APPLY_PERSON_COUNT: u32 = 40;
const APPLY_JOIN_NOTICE_CAPACITY: usize = 256;
const PRONOUNCE_UPDATE_MESSAGE_TYPE: i32 = 0x7FE10;
const LEAVE_WORD_UPDATE_MESSAGE_TYPE: i32 = 0x7FE0F;
const OTHER_FACTION_UPDATE_MESSAGE_TYPE: i32 = 0x7FE15;
const FACTION_TALK_MESSAGE_TYPE: i32 = 0x7FA02;
const FACTION_TALK_CHANNEL: i32 = 400;
const DELETE_ORGANIZING_MESSAGE_TYPE: i32 = 0x7FE03;
const OWNED_CITY_UPDATE_MESSAGE_TYPE: i32 = 0x7FE13;
const ENTER_REGION_BUFFER_CAPACITY: usize = 256;
const ENEMY_WAR_LOG_BUFFER_CAPACITY: usize = 256;
const LEAVE_WORD_NAME_CAPACITY: usize = 20;
const LEAVE_WORD_CONTENT_CAPACITY: usize = 212;
const LEAVE_WORD_CONTENT_LIMIT: usize = 210;
const LEAVE_WORD_LIMIT: usize = 60;
const APPLY_PERSON_NAME_CAPACITY: usize = 20;
const FACTION_MEMBER_NAME_CAPACITY: usize = 32;
const FACTION_MEMBER_TEXT_CAPACITY: usize = 64;
const FACTION_MEMBER_NOTICE_CAPACITY: usize = 260;
const FACTION_DUB_NOTICE_CAPACITY: usize = 100;
const FACTION_DUB_OLD_TITLE_CAPACITY: usize = 32;
const FACTION_PURVIEW_NOTICE_CAPACITY: usize = 100;
const FACTION_MAXIMUM_MEMBERS_NOTICE_CAPACITY: usize = 256;
const FACTION_UPGRADE_NOTICE_CAPACITY: usize = 256;
const FACTION_CONTRIBUTOR_NOTICE_CAPACITY: usize = 256;
const FACTION_UPLOAD_ICON_NOTICE_CAPACITY: usize = 256;
const FACTION_DEMISE_BLOCK_NOTICE_CAPACITY: usize = 256;
const FACTION_DEMISE_NOTICE_CAPACITY: usize = 100;
const PRONOUNCE_NAME_CAPACITY: usize = 20;
const PRONOUNCE_CONTENT_CAPACITY: usize = 2048;
const OTHER_FACTION_NAME_CAPACITY: usize = 20;
const DELETE_ORGANIZING_INFO_CAPACITY: usize = 100;
const PRONOUNCE_DATA_SIZE: usize = 0x828;
const FACTION_BASE_PROPERTY_SIZE: usize = 0x38;

const ZERO_TIME: TagTimeValue = TagTimeValue {
    year: 0,
    month: 0,
    day_of_week: 0,
    day: 0,
    hour: 0,
    minute: 0,
    second: 0,
    milliseconds: 0,
};

/// Полный byte-exact блок исходного `CFaction::tagFacBaseProperty`.
///
/// Три байта выравнивания после `btCountry` сохраняются вместе с полями:
/// `CloneSaveData` копировал структуру четырнадцатью 32-битными словами.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct FactionBaseProperty {
    bytes: [u8; FACTION_BASE_PROPERTY_SIZE],
}

impl FactionBaseProperty {
    /// Принимает уже полностью восстановленные `0x38` байт без нормализации.
    pub(crate) const fn from_complete_bytes(bytes: [u8; FACTION_BASE_PROPERTY_SIZE]) -> Self {
        Self { bytes }
    }

    const fn signed_at(&self, offset: usize) -> i32 {
        i32::from_le_bytes([
            self.bytes[offset],
            self.bytes[offset + 1],
            self.bytes[offset + 2],
            self.bytes[offset + 3],
        ])
    }

    pub(crate) const fn level(&self) -> i32 {
        self.signed_at(0x00)
    }

    pub(crate) const fn experience(&self) -> i32 {
        self.signed_at(0x04)
    }

    pub(crate) const fn upgrade_experience(&self) -> i32 {
        self.signed_at(0x20)
    }

    pub(crate) const fn offense_victor_counts(&self) -> i32 {
        self.signed_at(0x08)
    }

    pub(crate) const fn defence_victor_counts(&self) -> i32 {
        self.signed_at(0x0C)
    }

    pub(crate) const fn village_war_victor_counts(&self) -> i32 {
        self.signed_at(0x10)
    }

    pub(crate) const fn member_count(&self) -> i32 {
        self.signed_at(0x14)
    }

    pub(crate) const fn union_id(&self) -> i32 {
        self.signed_at(0x18)
    }

    pub(crate) const fn permit(&self) -> bool {
        self.bytes[0x2B] != 0
    }

    pub(crate) const fn country(&self) -> u8 {
        self.bytes[0x2C]
    }

    pub(crate) const fn property_1(&self) -> i32 {
        self.signed_at(0x30)
    }

    pub(crate) const fn property_2(&self) -> i32 {
        self.signed_at(0x34)
    }

    pub(crate) const fn leave_word_function(&self) -> bool {
        self.bytes[0x25] != 0
    }

    pub(crate) const fn pronounce_function(&self) -> bool {
        self.bytes[0x24] != 0
    }

    pub(crate) const fn create_union_function(&self) -> bool {
        self.bytes[0x2A] != 0
    }

    pub(crate) const fn upload_icon_function(&self) -> bool {
        self.bytes[0x27] != 0
    }

    const fn wire_bytes(&self) -> &[u8; FACTION_BASE_PROPERTY_SIZE] {
        &self.bytes
    }

    fn write_signed(&mut self, offset: usize, value: i32) {
        self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn set_country(&mut self, country: u8) {
        self.bytes[0x2C] = country;
    }

    fn set_permit(&mut self, permit: bool) {
        self.bytes[0x2B] = u8::from(permit);
    }

    fn feature_function(&self, feature: FactionFeatureFunction) -> bool {
        self.bytes[feature.property_offset()] != 0
    }

    fn set_feature_function(&mut self, feature: FactionFeatureFunction, enabled: bool) {
        self.bytes[feature.property_offset()] = u8::from(enabled);
    }

    fn set_initial_level_permissions(&mut self, parameters: &COrganizingParam) {
        let level = self.level();
        self.bytes[0x24] = u8::from(parameters.pronounce_minimum_level() <= level);
        self.bytes[0x25] = u8::from(parameters.leave_word_minimum_level() <= level);
        self.bytes[0x26] = u8::from(parameters.endue_right_minimum_level() <= level);
        self.bytes[0x2A] = u8::from(parameters.create_union_minimum_level() <= level);
        self.bytes[0x28] = u8::from(parameters.attack_village_minimum_level() <= level);
        self.bytes[0x29] = u8::from(parameters.attack_city_minimum_level() <= level);
    }
}

/// Локальная safe-граница dirty-bit `1` для ещё узкого live-state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionCloneSaveBlock {
    MasterIdMissing,
    MissingBaseProperty,
    EstablishedTimeUnknown,
    DeleteRemainTimeAbsent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionInitialPropertyBlock;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionSuperiorOrganizingBlock {
    MissingBaseProperty,
    DeleteRemainTimeAbsent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionDelMemberBlock {
    MasterIdMissing,
    DeleteRemainTimeAbsent,
    MissingBaseProperty,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionDelMemberReport {
    pub(crate) removed: bool,
    pub(crate) disband_countdown_started: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionOperatorValidationBlock;

/// Результат одной исходно игнорировавшейся отправки полного property.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionPropertyDelivery {
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

/// Результат одной исходно игнорировавшейся отправки enemy-set.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionEnemyDelivery {
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionEnemyRefreshReport {
    pub(crate) deliveries: Vec<FactionEnemyDelivery>,
    pub(crate) refreshed_player_ids: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CityWarEnemyRefreshOutcome {
    ChangeFlagUnknown,
    Unchanged,
    Published(FactionEnemyRefreshReport),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionPronounceDelivery {
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionLeaveWordDelivery {
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionApplyMemberDelivery {
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionApplyMemberUpdateBuildError {
    pub(crate) candidate_player_id: i32,
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) completed_deliveries: Vec<FactionApplyMemberDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionRemoveApplyMemberOutcome {
    NotFound,
    Removed {
        faction_id: i32,
        deliveries: Result<Vec<FactionApplyMemberDelivery>, FactionApplyMemberUpdateBuildError>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionApplyForJoinRejection {
    StandardOrVillageWar,
    CityWar,
    MemberLimit,
    ApplyListLimit,
    PlayerAlreadyInFaction,
    PlayerOffline,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionApplyForJoinOutcome {
    Rejected(FactionApplyForJoinRejection),
    Applied {
        deliveries: Result<Vec<FactionApplyMemberDelivery>, FactionApplyMemberUpdateBuildError>,
        log_written: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionApplyForJoinContextOperation {
    PlayerMembershipLookup,
    RemovePreviousApplications,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionApplyForJoinBlock<ContextBlock> {
    MissingBaseProperty,
    Context {
        operation: FactionApplyForJoinContextOperation,
        source: ContextBlock,
    },
    PlayerNameWouldOverflow {
        player_id: i32,
        visible_len: usize,
    },
    SuccessNoticeWouldOverflow {
        player_id: i32,
        formatted_len: usize,
        application_inserted: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionDoJoinRejection {
    StandardOrVillageWar,
    CityWar,
    GoodsWar,
    PermissionDenied,
    ApplicationNotFound,
    MemberLimit,
    ApplicantAlreadyInFaction,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionDoJoinOutcome {
    Rejected {
        reason: FactionDoJoinRejection,
        application_removal: Option<FactionRemoveApplyMemberOutcome>,
    },
    ApplicationDenied {
        application_removal: FactionRemoveApplyMemberOutcome,
    },
    Joined {
        application_removal: FactionRemoveApplyMemberOutcome,
        member_information: FactionMemberInfoReport,
        refreshed_player_ids: Vec<i32>,
        add_faction_to_client_result: bool,
        add_all_faction_info_result: bool,
        member_update: Result<MemberUpdateReport, MemberUpdateBuildError>,
        disband_countdown_cancelled: bool,
        log_written: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionDoJoinContextOperation {
    RemovePreviousApplications,
    ApplicantMembershipLookup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionDoJoinStringField {
    DenialNotice,
    MemberLimitNotice,
    MemberTitle,
    OnlinePlayerName,
    OnlineRegionName,
    MemberJoinedNotice,
    ApplicantJoinedNotice,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionDoJoinBlock<ContextBlock> {
    Context {
        operation: FactionDoJoinContextOperation,
        source: ContextBlock,
        application_removed: bool,
    },
    MissingBaseProperty {
        application_removed: bool,
    },
    StringWouldOverflow {
        field: FactionDoJoinStringField,
        visible_len: usize,
        capacity: usize,
        application_removed: bool,
        member_inserted: bool,
    },
    UnterminatedManagerName {
        manager_id: i32,
        member_inserted: bool,
    },
    ManagerMemberMissing {
        manager_id: i32,
        member_inserted: bool,
    },
    MissingDeleteRemainTime {
        member_inserted: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionExitRejection {
    StandardOrVillageWar,
    CityWar,
    PermissionDenied,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionExitOutcome {
    Rejected {
        reason: FactionExitRejection,
        goods_war_notice_sent: bool,
    },
    Exited {
        goods_war_notice_sent: bool,
        member_information: FactionMemberInfoReport,
        member_removal: Option<FactionDelMemberReport>,
        delete_organizing: FactionDeleteOrganizingOutcome,
        refreshed_player_ids: Vec<i32>,
        member_update: Result<MemberUpdateReport, MemberUpdateBuildError>,
        log_written: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionExitBlock {
    UnterminatedMemberName {
        player_id: i32,
        goods_war_notice_sent: bool,
    },
    NoticeWouldOverflow {
        player_id: i32,
        formatted_len: usize,
        goods_war_notice_sent: bool,
    },
    DelMember {
        source: FactionDelMemberBlock,
        goods_war_notice_sent: bool,
        member_information: FactionMemberInfoReport,
        member_removed: bool,
    },
    DeleteOrganizing {
        source: FactionDeleteOrganizingBuildError,
        goods_war_notice_sent: bool,
        member_information: FactionMemberInfoReport,
        member_removal: Option<FactionDelMemberReport>,
        refreshed_player_ids: Vec<i32>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionFireOutRejection {
    StandardOrVillageWar,
    CityWar,
    GoodsWar,
    OperatorValidationFailed,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionFireOutOutcome {
    Rejected(FactionFireOutRejection),
    Fired {
        member_information: FactionMemberInfoReport,
        member_removal: Option<FactionDelMemberReport>,
        delete_organizing: FactionDeleteOrganizingOutcome,
        member_update: Result<MemberUpdateReport, MemberUpdateBuildError>,
        refreshed_player_ids: Vec<i32>,
        log_written: bool,
        local_message: Result<(), WorldLocalMessageQueueBlock>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionFireOutBlock {
    OperatorValidation(FactionOperatorValidationBlock),
    UnterminatedTargetName {
        target_id: i32,
    },
    NoticeWouldOverflow {
        target_id: i32,
        formatted_len: usize,
    },
    DelMember {
        source: FactionDelMemberBlock,
        member_information: FactionMemberInfoReport,
        member_removed: bool,
    },
    DeleteOrganizing {
        source: FactionDeleteOrganizingBuildError,
        member_information: FactionMemberInfoReport,
        member_removal: Option<FactionDelMemberReport>,
    },
    UnterminatedManagerName {
        manager_id: i32,
        member_information: FactionMemberInfoReport,
        member_removal: Option<FactionDelMemberReport>,
        refreshed_player_ids: Vec<i32>,
        member_update: Result<MemberUpdateReport, MemberUpdateBuildError>,
        delete_organizing: FactionDeleteOrganizingOutcome,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionDubRejection {
    InvalidTitle,
    OperatorValidationFailed,
    InvalidJobLevel,
    TargetNotFound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionDubNotice {
    Title,
    JobLevel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionDubFormatArgument<'a> {
    Text(&'a [u8]),
    Signed(i32),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionDubProgress {
    pub(crate) input_title_truncated: bool,
    pub(crate) title_changed: bool,
    pub(crate) job_level_changed: bool,
    pub(crate) title_information: Option<FactionMemberInfoReport>,
    pub(crate) title_refreshed_player_ids: Vec<i32>,
    pub(crate) job_level_information: Option<FactionMemberInfoReport>,
    pub(crate) member_update: Option<Result<MemberUpdateReport, MemberUpdateBuildError>>,
    pub(crate) dirty_set: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionDubOutcome {
    Rejected {
        reason: FactionDubRejection,
        input_title_truncated: bool,
    },
    Updated {
        progress: FactionDubProgress,
        log_written: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionDubBlock {
    OperatorValidation(FactionOperatorValidationBlock),
    UnterminatedMemberField {
        player_id: i32,
        source: UnterminatedMemberField,
        progress: FactionDubProgress,
    },
    OldTitleWouldOverflow {
        target_id: i32,
        visible_len: usize,
        progress: FactionDubProgress,
    },
    NoticeWouldOverflow {
        notice: FactionDubNotice,
        formatted_len: usize,
        progress: FactionDubProgress,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionPurviewChange {
    Grant,
    Revoke,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionPurviewChangeRejection {
    FunctionDisabled,
    OperatorValidationFailed,
    InvalidPurview,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionPurviewChangeProgress {
    pub(crate) mutation: MemberPurviewMutation,
    pub(crate) member_update: Result<MemberUpdateReport, MemberUpdateBuildError>,
    pub(crate) dirty_set: bool,
    pub(crate) apply_snapshot:
        Option<Result<Vec<FactionApplyMemberDelivery>, FactionApplyMemberUpdateBuildError>>,
    pub(crate) member_information: Option<FactionMemberInfoReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionPurviewChangeOutcome {
    Rejected(FactionPurviewChangeRejection),
    Changed {
        progress: FactionPurviewChangeProgress,
        log_written: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionPurviewChangeBlock {
    MissingBaseProperty,
    OperatorValidation(FactionOperatorValidationBlock),
    UnterminatedMemberName {
        player_id: i32,
        source: UnterminatedMemberField,
        progress: FactionPurviewChangeProgress,
    },
    NoticeWouldOverflow {
        formatted_len: usize,
        progress: FactionPurviewChangeProgress,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionMaximumMembersUpdate {
    Unchanged,
    Updated(FactionMemberInfoReport),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionMaximumMembersBlock {
    MissingBaseProperty,
    NoticeWouldOverflow {
        maximum_members: i32,
        formatted_len: usize,
        property_changed: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionLevelUpdate {
    Unchanged,
    Updated {
        pronounce: FactionFeatureFunctionUpdate,
        leave_word: FactionFeatureFunctionUpdate,
        endue_right: FactionFeatureFunctionUpdate,
        create_union: FactionFeatureFunctionUpdate,
        join_village_war: FactionFeatureFunctionUpdate,
        join_city_war: FactionFeatureFunctionUpdate,
        maximum_members: FactionMaximumMembersUpdate,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionLevelBlock {
    MissingBaseProperty {
        level_changed: bool,
    },
    MaximumMembers {
        source: FactionMaximumMembersBlock,
        level_changed: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionSetParameterKind {
    Level,
    Experience,
    Unknown,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionSetParameterProgress {
    pub(crate) parameter: FactionSetParameterKind,
    pub(crate) level_update: Option<FactionLevelUpdate>,
    pub(crate) experience_update: Option<FactionExperienceUpdate>,
    pub(crate) assigned_experience: Option<i32>,
    pub(crate) assigned_upgrade_experience: Option<i32>,
    pub(crate) property_deliveries: Option<Vec<FactionPropertyDelivery>>,
    pub(crate) refreshed_player_ids: Option<Vec<i32>>,
    pub(crate) dirty_bit_requested: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionSetParameterOutcome {
    LevelAboveMaximum {
        requested_level: i32,
    },
    LevelParametersMissing {
        level: i32,
        progress: FactionSetParameterProgress,
    },
    Applied(FactionSetParameterProgress),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionSetParameterBlock {
    MissingBaseProperty {
        progress: FactionSetParameterProgress,
    },
    Level {
        source: FactionLevelBlock,
        progress: FactionSetParameterProgress,
    },
    Experience {
        source: FactionExperienceBlock,
        progress: FactionSetParameterProgress,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionUpgradeRejection {
    PlayerNotMaster,
    MaximumLevel,
    CurrentLevelParametersMissing,
    InsufficientExperience,
    PlayerOffline,
    InsufficientMoney,
    MasterLevelTooLow,
    RequiredGoodsMissing,
    NextLevelParametersMissing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionUpgradeNotice {
    Experience,
    Money,
    MasterLevel,
    Goods,
    Success,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionUpgradeFormatArgument<'a> {
    Text(&'a [u8]),
    Signed(i32),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionUpgradeChargeDelivery {
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionUpgradeProgress {
    pub(crate) level_update: Option<FactionLevelUpdate>,
    pub(crate) experience_update: Option<FactionExperienceUpdate>,
    pub(crate) charge_delivery: Option<FactionUpgradeChargeDelivery>,
    pub(crate) next_upgrade_experience: Option<i32>,
    pub(crate) member_information: Option<FactionMemberInfoReport>,
    pub(crate) property_deliveries: Option<Vec<FactionPropertyDelivery>>,
    pub(crate) dirty_set: bool,
    pub(crate) refreshed_player_ids: Vec<i32>,
    pub(crate) log_written: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionUpgradeOutcome {
    Rejected {
        reason: FactionUpgradeRejection,
        notice_sent: bool,
        progress: FactionUpgradeProgress,
    },
    Upgraded(FactionUpgradeProgress),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionUpgradeBlock {
    MissingBaseProperty {
        progress: FactionUpgradeProgress,
    },
    MissingPlayerMoney {
        player_id: i32,
        progress: FactionUpgradeProgress,
    },
    NoticeWouldOverflow {
        notice: FactionUpgradeNotice,
        formatted_len: usize,
        progress: FactionUpgradeProgress,
    },
    Level {
        source: FactionLevelBlock,
        progress: FactionUpgradeProgress,
    },
    Experience {
        source: FactionExperienceBlock,
        progress: FactionUpgradeProgress,
    },
    UnterminatedMemberName {
        player_id: i32,
        source: UnterminatedMemberField,
        progress: FactionUpgradeProgress,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionContributorRejection {
    RequesterNotMaster,
    MaximumContributors,
    TargetNotMember,
    Unchanged,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionContributorProgress {
    pub(crate) contributor_changed: bool,
    pub(crate) member_update: Option<MemberUpdateReport>,
    pub(crate) refreshed_player_ids: Vec<i32>,
    pub(crate) member_information: Option<FactionMemberInfoReport>,
    pub(crate) dirty_set: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionContributorOutcome {
    Rejected(FactionContributorRejection),
    Updated(FactionContributorProgress),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionContributorBlock {
    MemberUpdate {
        source: MemberUpdateBuildError,
        progress: FactionContributorProgress,
    },
    UnterminatedMemberName {
        player_id: i32,
        source: UnterminatedMemberField,
        progress: FactionContributorProgress,
    },
    NoticeWouldOverflow {
        formatted_len: usize,
        progress: FactionContributorProgress,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionUploadIconRejection {
    PlayerNotMaster,
    FunctionDisabled,
    IntervalActive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionUploadIconOutcome {
    Rejected {
        reason: FactionUploadIconRejection,
        notice_sent: bool,
    },
    Accepted {
        dirty_set: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionUploadIconBlock {
    MissingBaseProperty,
    NoticeWouldOverflow {
        formatted_len: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionDisbandRejection {
    OperatorNotPermitted,
    HasSuperiorOrganizing,
    StandardWar,
    CityWar,
    GoodsWar,
    CountryKing,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionDisbandProgress {
    pub(crate) goods_war_members_deleted: bool,
    pub(crate) goods_war_faction_count_decremented: bool,
    pub(crate) cleared_apply_persons: usize,
    pub(crate) cleared_leave_words: usize,
    pub(crate) delete_organizing: Option<FactionDeleteOrganizingOutcome>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionDisbandOutcome {
    Rejected {
        reason: FactionDisbandRejection,
        notice_sent: bool,
    },
    Disbanded(FactionDisbandProgress),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionDisbandBlock {
    MissingBaseProperty,
    CountryMissing {
        country: u8,
    },
    DeleteOrganizing {
        source: FactionDeleteOrganizingBuildError,
        progress: FactionDisbandProgress,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionDemiseRejection {
    SamePlayer,
    OldPlayerNotMaster,
    NewPlayerNotMember,
    StandardWar,
    CityWar,
    GoodsWar,
    OldPlayerOffline,
    NewPlayerOffline,
    NewPlayerOperatingFactionWar,
    OldPlayerOperatingFactionWar,
    NewMasterLevelTooLow,
    DemiseForbidden,
    CountryKingBlocked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionDemiseMember {
    OldMaster,
    NewMaster,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionDemiseNotice {
    RequiredLevel,
    DemiseForbidden,
    CountryKingBlocked,
    Success,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionDemiseProgress {
    pub(crate) permit_demise_disabled: bool,
    pub(crate) master_changed: bool,
    pub(crate) new_member_changed: bool,
    pub(crate) old_member_changed: bool,
    pub(crate) old_member_update: Option<MemberUpdateReport>,
    pub(crate) new_member_update: Option<MemberUpdateReport>,
    pub(crate) base_dirty_set: bool,
    pub(crate) members_dirty_set: bool,
    pub(crate) refreshed_player_ids: Vec<i32>,
    pub(crate) member_information: Option<FactionMemberInfoReport>,
    pub(crate) log_written: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionDemiseOutcome {
    Rejected {
        reason: FactionDemiseRejection,
        notice_sent: bool,
    },
    Transferred(FactionDemiseProgress),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionDemiseBlock {
    MissingBaseProperty,
    PermitDemiseUnknown,
    OldMasterMemberMissing {
        player_id: i32,
    },
    TitleWouldOverflow {
        member: FactionDemiseMember,
        string_id: &'static [u8],
        visible_len: usize,
        progress: FactionDemiseProgress,
    },
    MemberUpdate {
        member: FactionDemiseMember,
        source: MemberUpdateBuildError,
        progress: FactionDemiseProgress,
    },
    UnterminatedMemberName {
        member: FactionDemiseMember,
        player_id: i32,
        source: UnterminatedMemberField,
        progress: FactionDemiseProgress,
    },
    NoticeWouldOverflow {
        notice: FactionDemiseNotice,
        formatted_len: usize,
        progress: FactionDemiseProgress,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionOperationRejection {
    UnionMasterMismatch {
        union_id: i32,
        master_faction_id: i32,
    },
    RegionNotOwned {
        region_id: i32,
    },
    PlayerNotPermitted {
        player_id: i32,
        purview: EPurview,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionOperationOutcome {
    Rejected(FactionOperationRejection),
    Authorized,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionOperationBlock<ContextBlock> {
    MissingBaseProperty,
    UnionMembershipLookup(ContextBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionLeaveWordUpdateBuildError {
    MissingLastLeaveWord,
    Recipient {
        source: UnterminatedLeaveWordField,
        recipient_player_id: i32,
        game_server_id: i32,
        completed_deliveries: Vec<FactionLeaveWordDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionEditLeaveWordOutcome {
    FunctionDisabled,
    PermissionDenied,
    LeaveWordNotFound,
    Deleted {
        deliveries: Result<Vec<FactionLeaveWordDelivery>, FactionLeaveWordUpdateBuildError>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionLeaveWordBlock {
    MissingBaseProperty,
    LeaveWordId(WorldLeaveWordIdBlock),
    OfflineAuthorNameUnknown {
        player_id: i32,
        allocated_leave_word_id: i32,
        input_truncated: bool,
    },
    PlayerNameWouldOverflow {
        player_id: i32,
        allocated_leave_word_id: i32,
        visible_len: usize,
        input_truncated: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionLeaveWordOutcome {
    FunctionDisabled,
    PermissionDenied,
    Published {
        leave_word_id: i32,
        input_truncated: bool,
        evicted_count: usize,
        deliveries: Result<Vec<FactionLeaveWordDelivery>, FactionLeaveWordUpdateBuildError>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionLoadLeaveWordReport {
    pub(crate) evicted_count: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionPronounceUpdateBuildError {
    pub(crate) source: UnterminatedPronounceField,
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) completed_deliveries: Vec<FactionPronounceDelivery>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionPronounceBlock {
    MissingBaseProperty,
    ContentWouldOverflow {
        visible_len: usize,
        input_truncated: bool,
    },
    PlayerNameWouldOverflow {
        player_id: i32,
        visible_len: usize,
        input_truncated: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionPronounceOutcome {
    FunctionDisabled,
    PermissionDenied,
    Published {
        input_truncated: bool,
        deliveries: Result<Vec<FactionPronounceDelivery>, FactionPronounceUpdateBuildError>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FactionFeatureFunction {
    LeaveWord,
    Pronounce,
    EndueRight,
    JoinVillageWar,
    JoinCityWar,
    CreateUnion,
}

impl FactionFeatureFunction {
    const fn property_offset(self) -> usize {
        match self {
            Self::Pronounce => 0x24,
            Self::LeaveWord => 0x25,
            Self::EndueRight => 0x26,
            Self::JoinVillageWar => 0x28,
            Self::JoinCityWar => 0x29,
            Self::CreateUnion => 0x2A,
        }
    }

    const fn notification_string_id(self, enabled: bool) -> &'static [u8] {
        match (self, enabled) {
            (Self::LeaveWord, true) => b"WS0172",
            (Self::LeaveWord, false) => b"WS0173",
            (Self::Pronounce, true) => b"WS0174",
            (Self::Pronounce, false) => b"WS0175",
            (Self::EndueRight, true) => b"WS0176",
            (Self::EndueRight, false) => b"WS0177",
            (Self::JoinVillageWar, true) => b"WS0180",
            (Self::JoinVillageWar, false) => b"WS0181",
            (Self::JoinCityWar, true) => b"WS0182",
            (Self::JoinCityWar, false) => b"WS0183",
            (Self::CreateUnion, true) => b"WS0184",
            (Self::CreateUnion, false) => b"WS0185",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionMemberInfoRequest<'a> {
    pub(crate) recipient_player_id: i32,
    pub(crate) first_text: &'a [u8],
    pub(crate) second_text: &'a [u8],
    pub(crate) information_type: i32,
    pub(crate) color: u32,
    pub(crate) trailing_value: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionMemberInfoReport {
    pub(crate) recipient_player_ids: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionFeatureFunctionUpdate {
    Unchanged,
    Updated(FactionMemberInfoReport),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionOtherInfoDelivery {
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionOtherInfoBuildError {
    pub(crate) visible_name_len: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionTalkDelivery {
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

/// Узкая граница исходных StringTable и organizing-info controller-а.
pub(crate) trait FactionOrganizingInfoContext {
    /// Возвращает независимую копию результата `StringTable::getStringByID`.
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>>;

    /// Повторяет `COrganizingCtrl::SendOrgaInfoToClient`.
    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>);
}

/// Узкая граница war-system, organizing-controller, локализации и apply-log.
pub(crate) trait FactionApplyForJoinContext: FactionOrganizingInfoContext {
    type Block;

    fn already_declared_for_village_war(&self, faction_id: i32) -> bool;

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool;

    fn player_already_in_faction(&self, player_id: i32) -> Result<bool, Self::Block>;

    fn remove_previous_faction_applications(
        &mut self,
        game: &CGame,
        player_id: i32,
    ) -> Result<(), Self::Block>;

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8>;

    fn faction_apply_log_enabled(&self) -> bool;

    fn write_faction_apply_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        player_id: i32,
        player_name: &[u8],
        log_type: i32,
    );
}

/// Узкая граница внешних систем полного approve/reject join-пути.
pub(crate) trait FactionDoJoinContext: FactionOrganizingInfoContext {
    type Block;

    fn already_declared_for_village_war(&self, faction_id: i32) -> bool;

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool;

    fn goods_war_blocks_join(&self, faction_id: i32, manager_id: i32) -> bool;

    fn remove_previous_faction_applications(
        &mut self,
        game: &CGame,
        player_id: i32,
    ) -> Result<(), Self::Block>;

    fn applicant_already_in_faction(&self, player_id: i32) -> Result<bool, Self::Block>;

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8>;

    fn update_player_faction_info(&mut self, player_id: i32);

    fn add_faction_to_client_by_player_id(&mut self, player_id: i32) -> bool;

    fn add_all_faction_info_to_client_by_player_id(&mut self, player_id: i32) -> bool;

    fn faction_join_log_enabled(&self) -> bool;

    #[allow(clippy::too_many_arguments)]
    fn write_faction_join_log(
        &mut self,
        member_id: i32,
        member_name: &[u8],
        manager_id: i32,
        manager_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
        log_type: i32,
    );
}

/// Узкая граница war-system, player-owner, quit-log и Goods War для `Exit`.
pub(crate) trait FactionExitContext: FactionOrganizingInfoContext {
    fn already_declared_for_village_war(&self, faction_id: i32) -> bool;

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool;

    fn goods_war_blocks_exit(&self, faction_id: i32, player_id: i32) -> bool;

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8>;

    fn update_player_faction_info(&mut self, player_id: i32);

    fn faction_quit_log_enabled(&self) -> bool;

    fn write_faction_quit_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        player_id: i32,
        player_name: &[u8],
        log_type: i32,
    );

    fn delete_goods_war_member(&mut self, player_id: i32);
}

/// Узкая граница war-system, player-owner, fire-log и Goods War для `FireOut`.
pub(crate) trait FactionFireOutContext: FactionOrganizingInfoContext {
    fn already_declared_for_village_war(&self, faction_id: i32) -> bool;

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool;

    fn goods_war_blocks_fire_out(&self, faction_id: i32, manager_id: i32) -> bool;

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8>;

    fn update_player_faction_info(&mut self, player_id: i32);

    fn faction_fire_out_log_enabled(&self) -> bool;

    #[allow(clippy::too_many_arguments)]
    fn write_faction_fire_out_log(
        &mut self,
        member_id: i32,
        member_name: &[u8],
        manager_id: i32,
        manager_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
        log_type: i32,
    );

    fn delete_goods_war_member(&mut self, player_id: i32);
}

/// Узкая граница общего text-filter, локализации, player-owner и title-log.
pub(crate) trait FactionDubContext: FactionOrganizingInfoContext {
    fn check_invalid_string(&mut self, value: &mut Vec<u8>, mode: bool) -> bool;

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[FactionDubFormatArgument<'_>],
    ) -> Vec<u8>;

    fn update_player_faction_info(&mut self, player_id: i32);

    fn faction_title_log_enabled(&self) -> bool;

    #[allow(clippy::too_many_arguments)]
    fn write_faction_title_log(
        &mut self,
        member_id: i32,
        member_name: &[u8],
        old_title: &[u8],
        new_title: &[u8],
        manager_id: i32,
        manager_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
    );
}

/// Узкая граница локализации и faction-purview log-а.
pub(crate) trait FactionPurviewChangeContext: FactionOrganizingInfoContext {
    fn format_world_string(&mut self, string_id: &'static [u8], member_name: &[u8]) -> Vec<u8>;

    fn faction_purview_log_enabled(&self, change: FactionPurviewChange) -> bool;

    #[allow(clippy::too_many_arguments)]
    fn write_faction_purview_log(
        &mut self,
        member_id: i32,
        member_name: &[u8],
        purview: i32,
        manager_id: i32,
        manager_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
        log_type: i32,
    );
}

/// Узкая граница форматирования `WS0186` для level/max-member owner-ов.
pub(crate) trait FactionLevelContext: FactionOrganizingInfoContext {
    fn format_world_string_signed(&mut self, string_id: &'static [u8], value: i32) -> Vec<u8>;
}

/// Узкая граница level-уведомлений и virtual player-refresh для `SetParam`.
pub(crate) trait FactionSetParameterContext: FactionLevelContext {
    fn update_player_faction_info(&mut self, player_id: i32);
}

/// Узкая граница player inventory/money, goods catalog, локализации и level-log.
pub(crate) trait FactionUpgradeContext: FactionLevelContext {
    fn player_money(&self, player_id: i32) -> Option<u32>;

    fn goods_in_packet(&self, player_id: i32, original_name: &[u8]) -> i32;

    fn goods_display_name(&self, original_name: &[u8]) -> Option<Vec<u8>>;

    fn format_upgrade_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[FactionUpgradeFormatArgument<'_>],
    ) -> Vec<u8>;

    fn update_player_faction_info(&mut self, player_id: i32);

    fn faction_level_log_enabled(&self) -> bool;

    fn write_faction_level_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        level: i32,
        master_id: i32,
        master_name: &[u8],
    );
}

/// Узкая граница локализации и player-owner для `SetControbuter`.
pub(crate) trait FactionContributorContext: FactionOrganizingInfoContext {
    fn format_contributor_string(
        &mut self,
        string_id: &'static [u8],
        member_name: &[u8],
    ) -> Vec<u8>;

    fn update_player_faction_info(&mut self, player_id: i32);
}

/// Узкая граница локализации для фактического gate-owner-а `UploadIcon`.
pub(crate) trait FactionUploadIconContext: FactionOrganizingInfoContext {
    fn format_upload_icon_interval(
        &mut self,
        string_id: &'static [u8],
        interval_minutes: i32,
    ) -> Vec<u8>;
}

/// Узкая граница war/country/Goods-War owner-ов для `Disband`.
pub(crate) trait FactionDisbandContext: FactionOrganizingInfoContext {
    fn village_war_declared(&self, faction_id: i32) -> bool;

    fn city_war_declared(&self, faction_id: i32) -> bool;

    fn goods_war_blocks_disband(&self, faction_id: i32, player_id: i32) -> bool;

    fn country_king_id(&self, country: u8) -> Option<i32>;

    fn delete_goods_war_members_by_faction_id(&mut self, faction_id: i32);

    fn decrement_goods_war_faction_count(&mut self, faction_id: i32);
}

/// Узкая граница war/country/localization/player-refresh/log для `Demise`.
pub(crate) trait FactionDemiseContext: FactionOrganizingInfoContext {
    fn attack_city_system_declared(&self, faction_id: i32) -> bool;

    fn goods_war_blocks_demise(&self, faction_id: i32, old_master_id: i32) -> bool;

    fn country_blocks_demise(&self, country: u8, old_master_id: i32) -> bool;

    fn format_demise_signed(
        &mut self,
        string_id: &'static [u8],
        value: i32,
    ) -> Vec<u8>;

    fn format_demise_change(
        &mut self,
        string_id: &'static [u8],
        old_master_name: &[u8],
        new_master_name: &[u8],
    ) -> Vec<u8>;

    fn update_player_faction_info(&mut self, player_id: i32);

    fn faction_master_log_enabled(&self) -> bool;

    #[allow(clippy::too_many_arguments)]
    fn write_faction_master_log(
        &mut self,
        old_master_id: i32,
        old_master_name: &[u8],
        new_master_id: i32,
        new_master_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
    );
}

/// Read-only union authority, отдельно сохраняющий два исходных lookup-пути.
pub(crate) trait FactionOperationAuthorityContext {
    type Block;

    /// Повторяет controller-wide `IsFreeFaction` и возвращает signed union ID.
    fn union_id_for_faction(&self, faction_id: i32) -> Result<i32, Self::Block>;

    /// Повторяет nullable `GetConfederationOrganizing(...)->GetMasterID()`.
    fn union_master_faction_id(&self, union_id: i32) -> Option<i32>;
}

/// Read-only lookup союза для точного virtual `GetPlayerHeader`.
pub(crate) trait FactionPlayerHeaderContext {
    type Block;

    /// `None` означает miss/null самого union; `Some(0)` — найденный союз,
    /// который штатно не смог разрешить свою master-faction.
    fn union_player_header(&self, union_id: i32) -> Result<Option<i32>, Self::Block>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionPlayerHeaderBlock<ContextBlock> {
    MissingBaseProperty,
    MissingMasterId,
    Context(ContextBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionDeleteOrganizingDelivery {
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionDeleteOrganizingOutcome {
    SingleTarget {
        delivery: Option<FactionDeleteOrganizingDelivery>,
    },
    Broadcast {
        deliveries: Vec<FactionDeleteOrganizingDelivery>,
        information_recipient_ids: Vec<i32>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionDeleteOrganizingBuildError {
    pub(crate) localized_info_len: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionEnemyWarLogArgument<'a> {
    Text(&'a [u8]),
    Unsigned(u32),
}

/// Узкая граница organizing lookup, локализации и исходного `war`-лога.
pub(crate) trait FactionEnemyMutationContext {
    /// Возвращает независимую byte-exact копию имени живого organizing.
    fn organizing_name(&self, organizing_id: i32) -> Option<Vec<u8>>;

    /// Повторяет `StringTable::getStringByID` с fallback `""` и `_sprintf`.
    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[FactionEnemyWarLogArgument<'_>],
    ) -> Vec<u8>;

    /// Повторяет `PutStringToFile("war", text)`.
    fn put_war_log(&mut self, text: &[u8]);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionEnemyMutationReport {
    pub(crate) state_changed: bool,
    pub(crate) changed_flag_set: bool,
    pub(crate) war_log_written: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionEnemyMutationBlock {
    pub(crate) state_changed: bool,
    pub(crate) changed_flag_set: bool,
    pub(crate) formatted_len: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionOwnedCityDelivery {
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionOwnedCityUpdateBuildError {
    Preflight(OwnedCitiesWireBuildError),
    Recipient {
        source: OwnedCitiesWireBuildError,
        recipient_player_id: i32,
        game_server_id: i32,
        completed_deliveries: Vec<FactionOwnedCityDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OwnedCityMutationReport {
    pub(crate) state_changed: bool,
    pub(crate) deliveries: Vec<FactionOwnedCityDelivery>,
    pub(crate) refreshed_player_ids: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OwnedCityBooleanMutationReport {
    pub(crate) legacy_result: bool,
    pub(crate) mutation: OwnedCityMutationReport,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OwnedCityMutationBuildError {
    pub(crate) state_changed: bool,
    pub(crate) source: FactionOwnedCityUpdateBuildError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionOwnedCityRefreshBlock {
    MissingBaseProperty,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionOwnedCityRefreshReport {
    pub(crate) refreshed_region_ids: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum OwnedCityAddOutcome {
    AlreadyOwned,
    Added(OwnedCityMutationReport),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionExperienceDelivery {
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionExperienceBlock {
    MissingBaseProperty,
    MasterIdMissing,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionExperienceUpdate {
    MaximumLevel,
    Unchanged { experience: i32 },
    Updated {
        experience: i32,
        deliveries: Vec<FactionExperienceDelivery>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionPermitBlock {
    MasterIdMissing,
    MissingBaseProperty,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FactionPermitUpdate {
    RequesterIsNotMaster,
    SuperiorMasterMismatch,
    Unchanged,
    Updated {
        deliveries: Vec<FactionPropertyDelivery>,
    },
}

#[derive(Clone, Copy)]
enum EnemyFactionSetKind {
    Standard,
    CityWar,
}

/// Полный результат `CFaction::ReInitialPropertyByLvl` после safe-границ.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FactionPropertyReinitialization {
    pub(crate) level_parameters_found: bool,
    pub(crate) deliveries: Vec<FactionPropertyDelivery>,
}

/// Результат одной исходно игнорировавшейся отправки member-update.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct MemberUpdateDelivery {
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    pub(crate) result: Result<i32, SendMessageError>,
}

/// Отчёт полного ordered recipient-прохода.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct MemberUpdateReport {
    /// `None` означает delete-ветвь без target lookup.
    pub(crate) target_found: Option<bool>,
    pub(crate) deliveries: Vec<MemberUpdateDelivery>,
}

/// Безопасная граница старого чтения за фиксированным member-массивом.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct MemberUpdateBuildError {
    pub(crate) field: UnterminatedMemberField,
    pub(crate) recipient_player_id: i32,
    pub(crate) game_server_id: i32,
    /// Уже выполненные send-ы не откатываются, как и в исходном ordered цикле.
    pub(crate) completed_deliveries: Vec<MemberUpdateDelivery>,
}

impl fmt::Display for MemberUpdateBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "member-update для игрока {} не построен: {}",
            self.recipient_player_id, self.field
        )
    }
}

impl Error for MemberUpdateBuildError {}

/// Локальная safe-граница старых nullable/неограниченных C-string операций.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MemberEnterBlockedReason {
    NullRegionPointer { region_id: i32 },
    RegionNameExceedsLocalBuffer { region_id: i32, byte_len: usize },
    UnterminatedMemberRegion(UnterminatedMemberField),
    RegionNameExceedsMemberField { region_id: i32, byte_len: usize },
}

impl fmt::Display for MemberEnterBlockedReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NullRegionPointer { region_id } => write!(
                formatter,
                "у tagRegion {} отсутствует исходный pRegion",
                region_id
            ),
            Self::RegionNameExceedsLocalBuffer {
                region_id,
                byte_len,
            } => write!(
                formatter,
                "имя региона {} длиной {} байт не помещается в старый char[256]",
                region_id, byte_len
            ),
            Self::UnterminatedMemberRegion(field) => field.fmt(formatter),
            Self::RegionNameExceedsMemberField {
                region_id,
                byte_len,
            } => write!(
                formatter,
                "имя региона {} длиной {} байт не помещается в старый strRegion[64]",
                region_id, byte_len
            ),
        }
    }
}

impl Error for MemberEnterBlockedReason {}

/// Итог virtual callback-а входа faction-member в игру.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum MemberEnterOutcome {
    PlayerNotOnline,
    MemberNotFound,
    RegionUnchanged,
    Blocked(MemberEnterBlockedReason),
    Published(Result<MemberUpdateReport, MemberUpdateBuildError>),
}

/// Итог virtual callback-а выхода faction-member из игры.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum MemberExitOutcome {
    MemberNotFound,
    RegionAlreadyEmpty,
    Published(Result<MemberUpdateReport, MemberUpdateBuildError>),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum MemberLevelChangeOutcome {
    MemberNotFound,
    Unchanged,
    Published(Result<MemberUpdateReport, MemberUpdateBuildError>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MemberPositionChangeBlocked {
    pub(crate) region_id: i32,
    pub(crate) byte_len: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum MemberPositionChangeOutcome {
    MemberNotFound,
    RegionNotFound,
    NullRegionPointer,
    Blocked(MemberPositionChangeBlocked),
    Published(Result<MemberUpdateReport, MemberUpdateBuildError>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OwnedCitiesWireBuildError {
    pub(crate) region_id: i32,
    pub(crate) byte_len: usize,
    pub(crate) completed_cities: usize,
}

impl fmt::Display for OwnedCitiesWireBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "имя региона {} длиной {} байт не помещается в старый char[256]",
            self.region_id, self.byte_len
        )
    }
}

impl Error for OwnedCitiesWireBuildError {}

/// Безопасная граница старого C-string чтения `tagApplyPerson::strName[20]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnterminatedApplyPersonName {
    pub(crate) player_id: i32,
    pub(crate) completed_persons: usize,
}

impl fmt::Display for UnterminatedApplyPersonName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "у кандидата {} отсутствует NUL в tagApplyPerson::strName",
            self.player_id
        )
    }
}

impl Error for UnterminatedApplyPersonName {}

/// Полное доказанное value исходного `m_ApplyPersons`.
#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) struct TagApplyPerson {
    pub(crate) id: i32,
    pub(crate) name: [u8; APPLY_PERSON_NAME_CAPACITY],
    pub(crate) occupation: i32,
    pub(crate) level: i32,
}

impl TagApplyPerson {
    pub(crate) const fn from_complete_fields(
        id: i32,
        name: [u8; APPLY_PERSON_NAME_CAPACITY],
        occupation: i32,
        level: i32,
    ) -> Self {
        Self {
            id,
            name,
            occupation,
            level,
        }
    }

    const fn empty_with_id(id: i32) -> Self {
        Self {
            id,
            name: [0; APPLY_PERSON_NAME_CAPACITY],
            occupation: 0,
            level: 0,
        }
    }

    fn name_wire_bytes(&self) -> Option<&[u8]> {
        let terminator = self.name.iter().position(|byte| *byte == 0)?;
        Some(&self.name[..=terminator])
    }
}

/// Ошибка безопасного C-string view одного fixed-поля `tagPronounceWord`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnterminatedPronounceField {
    pub(crate) field: &'static str,
}

impl fmt::Display for UnterminatedPronounceField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "в фиксированном поле {} отсутствует завершающий NUL",
            self.field
        )
    }
}

impl Error for UnterminatedPronounceField {}

/// Полное PDB-подтверждённое значение исходного `tagPronounceWord`.
#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) struct TagPronounceWord {
    pub(crate) player_id: i32,
    pub(crate) name: [u8; PRONOUNCE_NAME_CAPACITY],
    pub(crate) time: TagTimeValue,
    pub(crate) content: [u8; PRONOUNCE_CONTENT_CAPACITY],
}

impl TagPronounceWord {
    const ZERO: Self = Self {
        player_id: 0,
        name: [0; PRONOUNCE_NAME_CAPACITY],
        time: ZERO_TIME,
        content: [0; PRONOUNCE_CONTENT_CAPACITY],
    };

    pub(crate) const fn from_complete_fields(
        player_id: i32,
        name: [u8; PRONOUNCE_NAME_CAPACITY],
        time: TagTimeValue,
        content: [u8; PRONOUNCE_CONTENT_CAPACITY],
    ) -> Self {
        Self {
            player_id,
            name,
            time,
            content,
        }
    }

    fn name_wire_bytes(&self) -> Result<&[u8], UnterminatedPronounceField> {
        let Some(terminator) = self.name.iter().position(|byte| *byte == 0) else {
            return Err(UnterminatedPronounceField {
                field: "tagPronounceWord::strName",
            });
        };
        Ok(&self.name[..=terminator])
    }

    fn content_wire_bytes(&self) -> Result<&[u8], UnterminatedPronounceField> {
        let Some(terminator) = self.content.iter().position(|byte| *byte == 0) else {
            return Err(UnterminatedPronounceField {
                field: "tagPronounceWord::strContent",
            });
        };
        Ok(&self.content[..=terminator])
    }

    fn append_wire_bytes(&self, output: &mut Vec<u8>) {
        append_i32(output, self.player_id);
        output.extend_from_slice(&self.name);
        output.extend_from_slice(&self.time.wire_bytes());
        output.extend_from_slice(&self.content);
    }
}

/// Ошибка безопасного C-string view одного fixed-поля `tagLeaveWord`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnterminatedLeaveWordField {
    pub(crate) field: &'static str,
}

impl fmt::Display for UnterminatedLeaveWordField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "в фиксированном поле {} отсутствует завершающий NUL",
            self.field
        )
    }
}

impl Error for UnterminatedLeaveWordField {}

/// Полное доказанное значение исходного `CFaction::tagLeaveWord`.
#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) struct TagLeaveWord {
    pub(crate) id: i32,
    pub(crate) player_id: i32,
    pub(crate) name: [u8; LEAVE_WORD_NAME_CAPACITY],
    pub(crate) time: TagTimeValue,
    pub(crate) content: [u8; LEAVE_WORD_CONTENT_CAPACITY],
}

impl TagLeaveWord {
    /// Создаёт полный 0x100-байтовый leave-word snapshot.
    pub(crate) const fn from_complete_fields(
        id: i32,
        player_id: i32,
        name: [u8; LEAVE_WORD_NAME_CAPACITY],
        time: TagTimeValue,
        content: [u8; LEAVE_WORD_CONTENT_CAPACITY],
    ) -> Self {
        Self {
            id,
            player_id,
            name,
            time,
            content,
        }
    }

    /// Возвращает `strContent` до первого NUL включительно.
    pub(crate) fn content_wire_bytes(&self) -> Result<&[u8], UnterminatedLeaveWordField> {
        let Some(terminator) = self.content.iter().position(|byte| *byte == 0) else {
            // BLOCKED_MISSING_FACT: WorldServer SaveLeaveWords RVA 0x000FACA0
            // передавал `node + 0x34` в CheckPoint без размера; тот читал бы за
            // char[212], а достижимость и результат этого пути не доказаны.
            return Err(UnterminatedLeaveWordField {
                field: "tagLeaveWord::strContent",
            });
        };
        Ok(&self.content[..=terminator])
    }

    /// Возвращает `strName` до первого NUL включительно.
    pub(crate) fn name_wire_bytes(&self) -> Result<&[u8], UnterminatedLeaveWordField> {
        let Some(terminator) = self.name.iter().position(|byte| *byte == 0) else {
            return Err(UnterminatedLeaveWordField {
                field: "tagLeaveWord::strName",
            });
        };
        Ok(&self.name[..=terminator])
    }
}

const _: () = {
    assert!(size_of::<FactionBaseProperty>() == FACTION_BASE_PROPERTY_SIZE);
    assert!(size_of::<TagApplyPerson>() == 0x20);
    assert!(offset_of!(TagApplyPerson, id) == 0x00);
    assert!(offset_of!(TagApplyPerson, name) == 0x04);
    assert!(offset_of!(TagApplyPerson, occupation) == 0x18);
    assert!(offset_of!(TagApplyPerson, level) == 0x1C);
    assert!(size_of::<TagPronounceWord>() == PRONOUNCE_DATA_SIZE);
    assert!(offset_of!(TagPronounceWord, player_id) == 0x00);
    assert!(offset_of!(TagPronounceWord, name) == 0x04);
    assert!(offset_of!(TagPronounceWord, time) == 0x18);
    assert!(offset_of!(TagPronounceWord, content) == 0x28);
    assert!(size_of::<TagLeaveWord>() == 0x100);
    assert!(offset_of!(TagLeaveWord, id) == 0x00);
    assert!(offset_of!(TagLeaveWord, player_id) == 0x04);
    assert!(offset_of!(TagLeaveWord, name) == 0x08);
    assert!(offset_of!(TagLeaveWord, time) == 0x1C);
    assert!(offset_of!(TagLeaveWord, content) == 0x2C);
};

/// Достигнутая member-state часть исходного `CFaction`.
pub(crate) struct CFaction {
    faction_id: i32,
    name: Vec<u8>,
    master_id: Option<i32>,
    members: BTreeMap<i32, TagMemInfo>,
    base_property: Option<FactionBaseProperty>,
    established_time: Option<TagTimeValue>,
    delete_remain_time: Option<i32>,
    owned_cities: VecDeque<i32>,
    enemy_factions: BTreeSet<i32>,
    city_war_enemy_factions: BTreeSet<i32>,
    permit_demise: Option<bool>,
    enemy_factions_changed: Option<bool>,
    city_war_enemy_factions_changed: Option<bool>,
    apply_persons: BTreeMap<i32, TagApplyPerson>,
    pronounce: TagPronounceWord,
    leave_words: VecDeque<TagLeaveWord>,
    last_upload_icon_time: TagTimeValue,
    icon_data: Vec<u8>,
    change_data_type: i32,
    goods_war_count: i32,
    goods_war_last_win_time: String,
}

impl CFaction {
    /// Создаёт доказанный пустой `m_Members` с уже назначенным faction ID.
    pub(crate) const fn with_reached_member_state(faction_id: i32) -> Self {
        Self {
            faction_id,
            name: Vec::new(),
            master_id: None,
            members: BTreeMap::new(),
            base_property: None,
            established_time: None,
            delete_remain_time: None,
            owned_cities: VecDeque::new(),
            enemy_factions: BTreeSet::new(),
            city_war_enemy_factions: BTreeSet::new(),
            permit_demise: None,
            enemy_factions_changed: None,
            city_war_enemy_factions_changed: None,
            apply_persons: BTreeMap::new(),
            pronounce: TagPronounceWord::ZERO,
            leave_words: VecDeque::new(),
            last_upload_icon_time: ZERO_TIME,
            icon_data: Vec::new(),
            change_data_type: 0,
            goods_war_count: 0,
            goods_war_last_win_time: String::new(),
        }
    }

    /// Возвращает исходный signed `m_lID`.
    pub(crate) const fn faction_id(&self) -> i32 {
        self.faction_id
    }

    /// Возвращает byte-exact содержимое исходного `m_strName`.
    pub(crate) fn name(&self) -> &[u8] {
        &self.name
    }

    /// Копирует `std::string` byte-exact; исходный virtual всегда возвращал `true`.
    pub(crate) fn set_name(&mut self, name: &[u8]) -> bool {
        self.name.clear();
        self.name.extend_from_slice(name);
        true
    }

    /// Возвращает достигнутый `m_lMastterID`; narrow state его не назначает.
    pub(crate) const fn master_id(&self) -> Option<i32> {
        self.master_id
    }

    /// Возвращает reached `m_Property.lLvl` без выдуманного default.
    pub(crate) const fn level(&self) -> Option<i32> {
        match self.base_property {
            Some(property) => Some(property.level()),
            None => None,
        }
    }

    /// Возвращает reached `m_Property.lExp` без выдуманного default.
    pub(crate) const fn experience(&self) -> Option<i32> {
        match self.base_property {
            Some(property) => Some(property.experience()),
            None => None,
        }
    }

    /// Возвращает полный reached `m_Property` вместе с исходным padding.
    pub(crate) const fn base_property(&self) -> Option<FactionBaseProperty> {
        self.base_property
    }

    pub(crate) const fn country(&self) -> Option<u8> {
        match self.base_property {
            Some(property) => Some(property.country()),
            None => None,
        }
    }

    /// Меняет только `m_Property.btCountry`; partial property остаётся safe-границей.
    pub(crate) fn set_country(
        &mut self,
        country: u8,
    ) -> Result<(), FactionInitialPropertyBlock> {
        let property = self
            .base_property
            .as_mut()
            .ok_or(FactionInitialPropertyBlock)?;
        property.set_country(country);
        Ok(())
    }

    pub(crate) const fn permit_demise(&self) -> Option<bool> {
        self.permit_demise
    }

    pub(crate) fn set_permit_demise(&mut self, permit: bool) {
        self.permit_demise = Some(permit);
    }

    pub(crate) const fn enemy_factions_changed(&self) -> Option<bool> {
        self.enemy_factions_changed
    }

    pub(crate) fn set_enemy_factions_changed(&mut self, changed: bool) {
        self.enemy_factions_changed = Some(changed);
    }

    pub(crate) const fn city_war_enemy_factions_changed(&self) -> Option<bool> {
        self.city_war_enemy_factions_changed
    }

    pub(crate) fn set_city_war_enemy_factions_changed(&mut self, changed: bool) {
        self.city_war_enemy_factions_changed = Some(changed);
    }

    pub(crate) const fn is_permitted(&self) -> Option<bool> {
        match self.base_property {
            Some(property) => Some(property.permit()),
            None => None,
        }
    }

    /// Меняет faction-permit в точном порядке `SetIsPermit`.
    ///
    /// Lookup заменяет исходный singleton и получает ровно текущий union ID;
    /// `None` сохраняет общий null/missing результат старого controller-а.
    pub(crate) fn set_is_permitted<F>(
        &mut self,
        game: &CGame,
        requester_id: i32,
        permit: bool,
        superior_master_by_id: F,
    ) -> Result<FactionPermitUpdate, FactionPermitBlock>
    where
        F: FnOnce(i32) -> Option<i32>,
    {
        let master_id = self
            .master_id
            .ok_or(FactionPermitBlock::MasterIdMissing)?;
        if requester_id != master_id {
            return Ok(FactionPermitUpdate::RequesterIsNotMaster);
        }

        let property = self
            .base_property
            .ok_or(FactionPermitBlock::MissingBaseProperty)?;
        if superior_master_by_id(property.union_id())
            .is_some_and(|superior_master_id| superior_master_id != master_id)
        {
            return Ok(FactionPermitUpdate::SuperiorMasterMismatch);
        }
        if property.permit() == permit {
            return Ok(FactionPermitUpdate::Unchanged);
        }

        let property = self
            .base_property
            .as_mut()
            .ok_or(FactionPermitBlock::MissingBaseProperty)?;
        property.set_permit(permit);
        let deliveries = self
            .update_property_to_client(game)
            .map_err(|_| FactionPermitBlock::MissingBaseProperty)?;
        self.set_change_data(1);
        Ok(FactionPermitUpdate::Updated { deliveries })
    }

    pub(crate) const fn is_leave_word_function(&self) -> Option<bool> {
        match self.base_property {
            Some(property) => Some(property.leave_word_function()),
            None => None,
        }
    }

    pub(crate) const fn is_create_union_function(&self) -> Option<bool> {
        match self.base_property {
            Some(property) => Some(property.create_union_function()),
            None => None,
        }
    }

    pub(crate) const fn defence_victor_counts(&self) -> Option<i32> {
        match self.base_property {
            Some(property) => Some(property.defence_victor_counts()),
            None => None,
        }
    }

    pub(crate) const fn offense_victor_counts(&self) -> Option<i32> {
        match self.base_property {
            Some(property) => Some(property.offense_victor_counts()),
            None => None,
        }
    }

    /// Пересчитывает level-зависимый property prefix без клиентской публикации.
    pub(crate) fn initial_property_by_level(
        &mut self,
        parameters: &COrganizingParam,
    ) -> Result<bool, FactionInitialPropertyBlock> {
        let property = self
            .base_property
            .as_mut()
            .ok_or(FactionInitialPropertyBlock)?;
        let level = property.level();
        property.set_initial_level_permissions(parameters);

        let maximum_members = parameters.get_max_number_by_level(level);
        if property.signed_at(0x1C) != maximum_members {
            property.write_signed(0x1C, maximum_members);
        }

        let Some(level_parameters) = parameters.get_level_param(level) else {
            return Ok(false);
        };
        property.write_signed(0x20, level_parameters.experience);
        Ok(true)
    }

    /// Публикует полный `tagFacBaseProperty` всем готовым faction-members.
    pub(crate) fn update_property_to_client(
        &self,
        game: &CGame,
    ) -> Result<Vec<FactionPropertyDelivery>, FactionInitialPropertyBlock> {
        let property = self.base_property.ok_or(FactionInitialPropertyBlock)?;
        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(0x7FE0B);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add(property.wire_bytes());
            deliveries.push(FactionPropertyDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    /// Пересчитывает property и безусловно публикует его, как старый wrapper.
    pub(crate) fn reinitialize_property_by_level(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
    ) -> Result<FactionPropertyReinitialization, FactionInitialPropertyBlock> {
        let level_parameters_found = self.initial_property_by_level(parameters)?;
        let deliveries = self.update_property_to_client(game)?;
        Ok(FactionPropertyReinitialization {
            level_parameters_found,
            deliveries,
        })
    }

    fn add_victor_count(
        &mut self,
        game: &CGame,
        property_offset: usize,
    ) -> Result<Vec<FactionPropertyDelivery>, FactionInitialPropertyBlock> {
        let property = self
            .base_property
            .as_mut()
            .ok_or(FactionInitialPropertyBlock)?;
        let next = property.signed_at(property_offset).wrapping_add(1);
        property.write_signed(property_offset, next);
        let deliveries = self.update_property_to_client(game)?;
        self.set_change_data(1);
        Ok(deliveries)
    }

    pub(crate) fn add_defence_victor_count(
        &mut self,
        game: &CGame,
    ) -> Result<Vec<FactionPropertyDelivery>, FactionInitialPropertyBlock> {
        self.add_victor_count(game, 0x0C)
    }

    pub(crate) fn add_offense_victor_count(
        &mut self,
        game: &CGame,
    ) -> Result<Vec<FactionPropertyDelivery>, FactionInitialPropertyBlock> {
        self.add_victor_count(game, 0x08)
    }

    pub(crate) fn add_village_war_victor_count(
        &mut self,
        game: &CGame,
    ) -> Result<Vec<FactionPropertyDelivery>, FactionInitialPropertyBlock> {
        self.add_victor_count(game, 0x10)
    }

    /// Возвращает reached `m_EstablishedTime` без выдуманного default.
    pub(crate) const fn established_time(&self) -> Option<TagTimeValue> {
        self.established_time
    }

    /// Возвращает reached `m_lDelRemainTime` без выдуманного default.
    pub(crate) const fn delete_remain_time(&self) -> Option<i32> {
        self.delete_remain_time
    }

    /// Заменяет reached signed `m_lDelRemainTime`, если значение изменилось.
    pub(crate) fn set_delete_remain_time(&mut self, value: i32) {
        if self.delete_remain_time != Some(value) {
            self.delete_remain_time = Some(value);
        }
    }

    /// Возвращает исходный list-order `m_OwnedCities`.
    pub(crate) const fn owned_cities(&self) -> &VecDeque<i32> {
        &self.owned_cities
    }

    /// Синхронизирует каждый owned region с текущими faction/union ID.
    pub(crate) fn refresh_owned_city_info<F>(
        &self,
        mut refresh_owned_city: F,
    ) -> Result<FactionOwnedCityRefreshReport, FactionOwnedCityRefreshBlock>
    where
        F: FnMut(i32, i32, i32),
    {
        if self.owned_cities.is_empty() {
            return Ok(FactionOwnedCityRefreshReport {
                refreshed_region_ids: Vec::new(),
            });
        }
        let union_id = self
            .base_property
            .as_ref()
            .ok_or(FactionOwnedCityRefreshBlock::MissingBaseProperty)?
            .union_id();
        let mut refreshed_region_ids = Vec::with_capacity(self.owned_cities.len());
        for &region_id in &self.owned_cities {
            refresh_owned_city(region_id, self.faction_id, union_id);
            refreshed_region_ids.push(region_id);
        }
        Ok(FactionOwnedCityRefreshReport {
            refreshed_region_ids,
        })
    }

    /// Дописывает owned-city list вместе с исходными region C-строками.
    pub(crate) fn add_owned_cities_to_byte_array(
        &self,
        game: &CGame,
        output: &mut Vec<u8>,
    ) -> Result<bool, OwnedCitiesWireBuildError> {
        output.extend_from_slice(&(self.owned_cities.len() as u32).to_le_bytes());
        let mut completed_cities = 0usize;
        for &region_id in &self.owned_cities {
            append_i32(output, region_id);
            let region_name = match game.region_name(region_id) {
                WorldRegionNameLookup::RegionNotFound
                | WorldRegionNameLookup::NullRegionPointer => &[][..],
                WorldRegionNameLookup::Name(name) => name,
            };
            if region_name.len() >= 256 {
                return Err(OwnedCitiesWireBuildError {
                    region_id,
                    byte_len: region_name.len(),
                    completed_cities,
                });
            }
            output.extend_from_slice(region_name);
            output.push(0);
            completed_cities += 1;
        }
        Ok(true)
    }

    /// Публикует полный owned-city snapshot каждому готовому member recipient.
    pub(crate) fn update_owned_cities_to_client(
        &self,
        game: &CGame,
    ) -> Result<Vec<FactionOwnedCityDelivery>, FactionOwnedCityUpdateBuildError> {
        let mut preflight = Vec::new();
        self.add_owned_cities_to_byte_array(game, &mut preflight)
            .map_err(FactionOwnedCityUpdateBuildError::Preflight)?;

        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut serialized_cities = Vec::new();
            if let Err(source) =
                self.add_owned_cities_to_byte_array(game, &mut serialized_cities)
            {
                return Err(FactionOwnedCityUpdateBuildError::Recipient {
                    source,
                    recipient_player_id,
                    game_server_id,
                    completed_deliveries: deliveries,
                });
            }
            let mut message = CMessage::new(OWNED_CITY_UPDATE_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add(&serialized_cities);
            deliveries.push(FactionOwnedCityDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    /// Диспетчеризует `CPlayer::UpdateFactionInfo` только online-игрокам.
    pub(crate) fn update_player_faction_info<F>(
        &self,
        game: &CGame,
        player_id: i32,
        mut update_player: F,
    ) -> Vec<i32>
    where
        F: FnMut(i32),
    {
        let mut updated_player_ids = Vec::new();
        if player_id == 0 {
            for &member_id in self.members.keys() {
                if game.online_player_by_id(member_id as u32).is_none() {
                    continue;
                }
                update_player(member_id);
                updated_player_ids.push(member_id);
            }
        } else if game.online_player_by_id(player_id as u32).is_some() {
            update_player(player_id);
            updated_player_ids.push(player_id);
        }
        updated_player_ids
    }

    fn finish_owned_city_mutation<F>(
        &self,
        game: &CGame,
        state_changed: bool,
        update_player: F,
    ) -> Result<OwnedCityMutationReport, OwnedCityMutationBuildError>
    where
        F: FnMut(i32),
    {
        let deliveries = self.update_owned_cities_to_client(game).map_err(|source| {
            OwnedCityMutationBuildError {
                state_changed,
                source,
            }
        })?;
        let refreshed_player_ids = self.update_player_faction_info(game, 0, update_player);
        Ok(OwnedCityMutationReport {
            state_changed,
            deliveries,
            refreshed_player_ids,
        })
    }

    /// Очищает список, публикует snapshot и обновляет online member-ов.
    pub(crate) fn clear_owned_cities<F>(
        &mut self,
        game: &CGame,
        update_player: F,
    ) -> Result<OwnedCityBooleanMutationReport, OwnedCityMutationBuildError>
    where
        F: FnMut(i32),
    {
        let state_changed = !self.owned_cities.is_empty();
        self.owned_cities.clear();
        let mutation = self.finish_owned_city_mutation(game, state_changed, update_player)?;
        Ok(OwnedCityBooleanMutationReport {
            legacy_result: true,
            mutation,
        })
    }

    /// Добавляет один город только при полном отсутствии такого ID.
    pub(crate) fn add_owned_city<F>(
        &mut self,
        game: &CGame,
        region_id: i32,
        update_player: F,
    ) -> Result<OwnedCityAddOutcome, OwnedCityMutationBuildError>
    where
        F: FnMut(i32),
    {
        if self.owned_cities.contains(&region_id) {
            return Ok(OwnedCityAddOutcome::AlreadyOwned);
        }
        self.owned_cities.push_back(region_id);
        let report = self.finish_owned_city_mutation(game, true, update_player)?;
        Ok(OwnedCityAddOutcome::Added(report))
    }

    /// Дописывает range и повторяет только adjacent `std::list::unique`.
    pub(crate) fn add_owned_city_list<F>(
        &mut self,
        game: &CGame,
        region_ids: &VecDeque<i32>,
        update_player: F,
    ) -> Result<OwnedCityMutationReport, OwnedCityMutationBuildError>
    where
        F: FnMut(i32),
    {
        let previous_state = self.owned_cities.clone();
        self.owned_cities.extend(region_ids.iter().copied());
        let mut previous = None;
        self.owned_cities.retain(|region_id| {
            let keep = previous != Some(*region_id);
            previous = Some(*region_id);
            keep
        });
        let state_changed = self.owned_cities != previous_state;
        self.finish_owned_city_mutation(game, state_changed, update_player)
    }

    /// Удаляет первое совпадение и публикует snapshot даже при miss.
    pub(crate) fn delete_owned_city<F>(
        &mut self,
        game: &CGame,
        region_id: i32,
        update_player: F,
    ) -> Result<OwnedCityBooleanMutationReport, OwnedCityMutationBuildError>
    where
        F: FnMut(i32),
    {
        let position = self
            .owned_cities
            .iter()
            .position(|owned_region_id| *owned_region_id == region_id);
        let state_changed = position.is_some();
        if let Some(position) = position {
            self.owned_cities.remove(position);
        }
        let mutation = self.finish_owned_city_mutation(game, state_changed, update_player)?;
        Ok(OwnedCityBooleanMutationReport {
            legacy_result: true,
            mutation,
        })
    }

    /// Заменяет список и публикует snapshot без player refresh.
    pub(crate) fn set_owned_cities(
        &mut self,
        game: &CGame,
        region_ids: &VecDeque<i32>,
    ) -> Result<OwnedCityMutationReport, OwnedCityMutationBuildError> {
        let state_changed = self.owned_cities != *region_ids;
        self.owned_cities.clone_from(region_ids);
        let deliveries = self.update_owned_cities_to_client(game).map_err(|source| {
            OwnedCityMutationBuildError {
                state_changed,
                source,
            }
        })?;
        Ok(OwnedCityMutationReport {
            state_changed,
            deliveries,
            refreshed_player_ids: Vec::new(),
        })
    }

    /// Возвращает faction ID, только если город есть в исходном list-order.
    pub(crate) fn is_owned_city(&self, region_id: i32) -> i32 {
        if self.owned_cities.contains(&region_id) {
            self.faction_id
        } else {
            0
        }
    }

    /// Возвращает faction ID только для текущего `m_lMastterID`.
    pub(crate) const fn is_master(&self, player_id: i32) -> i32 {
        match self.master_id {
            Some(master_id) if master_id == player_id => self.faction_id,
            Some(_) | None => 0,
        }
    }

    /// Возвращает копируемый `m_EnemyFactions` в signed key-order.
    pub(crate) const fn enemy_factions(&self) -> &BTreeSet<i32> {
        &self.enemy_factions
    }

    /// Возвращает независимую value-копию исходного `std::set<long>`.
    pub(crate) fn enemy_factions_snapshot(&self) -> BTreeSet<i32> {
        self.enemy_factions.clone()
    }

    /// Очищает standard enemy-set и отмечает только фактическое изменение.
    pub(crate) fn clear_enemy_factions(&mut self) {
        if !self.enemy_factions.is_empty() {
            self.enemy_factions_changed = Some(true);
        }
        self.enemy_factions.clear();
    }

    /// Сохраняет exact `set::_Mysize != 0` без зависимости от MSVC layout.
    pub(crate) fn has_enemy_faction(&self) -> bool {
        !self.enemy_factions.is_empty()
    }

    /// Возвращает копируемый `m_CityWarEnemyFactions` в signed key-order.
    pub(crate) const fn city_war_enemy_factions(&self) -> &BTreeSet<i32> {
        &self.city_war_enemy_factions
    }

    /// Возвращает независимую value-копию city-war `std::set<long>`.
    pub(crate) fn city_war_enemy_factions_snapshot(&self) -> BTreeSet<i32> {
        self.city_war_enemy_factions.clone()
    }

    /// Очищает city-war enemy-set и отмечает только фактическое изменение.
    pub(crate) fn clear_city_war_enemy_factions(&mut self) {
        if !self.city_war_enemy_factions.is_empty() {
            self.city_war_enemy_factions_changed = Some(true);
        }
        self.city_war_enemy_factions.clear();
    }

    /// Сохраняет exact `set::_Mysize != 0` для city-war enemy-set.
    pub(crate) fn has_city_war_enemy_faction(&self) -> bool {
        !self.city_war_enemy_factions.is_empty()
    }

    /// Сохраняет отдельный исторический stub, не подменяя его membership-check.
    pub(crate) const fn is_enemy_faction(&self, _faction_id: i32) -> i32 {
        0
    }

    fn write_enemy_war_log<Context>(
        &self,
        context: &mut Context,
        enemy_id: i32,
        string_id: &'static [u8],
        remaining_enemy_count: Option<u32>,
        state_changed: bool,
        changed_flag_set: bool,
    ) -> Result<bool, FactionEnemyMutationBlock>
    where
        Context: FactionEnemyMutationContext,
    {
        if enemy_id <= 0 {
            return Ok(false);
        }
        let Some(enemy_name) = context.organizing_name(enemy_id) else {
            return Ok(false);
        };

        let mut arguments = vec![
            FactionEnemyWarLogArgument::Text(&self.name),
            FactionEnemyWarLogArgument::Text(&enemy_name),
        ];
        if let Some(remaining_enemy_count) = remaining_enemy_count {
            arguments.push(FactionEnemyWarLogArgument::Unsigned(
                remaining_enemy_count,
            ));
        }
        let text = context.format_world_string(string_id, &arguments);
        if text.len() >= ENEMY_WAR_LOG_BUFFER_CAPACITY {
            return Err(FactionEnemyMutationBlock {
                state_changed,
                changed_flag_set,
                formatted_len: text.len(),
            });
        }
        context.put_war_log(&text);
        Ok(true)
    }

    /// Добавляет standard enemy и только при новой записи пишет `WS0158`.
    pub(crate) fn add_enemy_organizing<Context>(
        &mut self,
        enemy_id: i32,
        context: &mut Context,
    ) -> Result<FactionEnemyMutationReport, FactionEnemyMutationBlock>
    where
        Context: FactionEnemyMutationContext,
    {
        let state_changed = self.enemy_factions.insert(enemy_id);
        if !state_changed {
            return Ok(FactionEnemyMutationReport {
                state_changed: false,
                changed_flag_set: false,
                war_log_written: false,
            });
        }
        let war_log_written =
            self.write_enemy_war_log(context, enemy_id, b"WS0158", None, true, false)?;
        Ok(FactionEnemyMutationReport {
            state_changed,
            changed_flag_set: false,
            war_log_written,
        })
    }

    /// Удаляет существующего standard enemy и после erase пишет `WS0159`.
    pub(crate) fn del_enemy_organizing<Context>(
        &mut self,
        enemy_id: i32,
        context: &mut Context,
    ) -> Result<FactionEnemyMutationReport, FactionEnemyMutationBlock>
    where
        Context: FactionEnemyMutationContext,
    {
        let state_changed = self.enemy_factions.remove(&enemy_id);
        if !state_changed {
            return Ok(FactionEnemyMutationReport {
                state_changed: false,
                changed_flag_set: false,
                war_log_written: false,
            });
        }
        let remaining_enemy_count = self.enemy_factions.len() as u32;
        let war_log_written = self.write_enemy_war_log(
            context,
            enemy_id,
            b"WS0159",
            Some(remaining_enemy_count),
            true,
            false,
        )?;
        Ok(FactionEnemyMutationReport {
            state_changed,
            changed_flag_set: false,
            war_log_written,
        })
    }

    /// Добавляет city-war enemy, выставляя changed byte перед вставкой.
    pub(crate) fn add_city_war_enemy_organizing<Context>(
        &mut self,
        enemy_id: i32,
        context: &mut Context,
    ) -> Result<FactionEnemyMutationReport, FactionEnemyMutationBlock>
    where
        Context: FactionEnemyMutationContext,
    {
        if self.city_war_enemy_factions.contains(&enemy_id) {
            return Ok(FactionEnemyMutationReport {
                state_changed: false,
                changed_flag_set: false,
                war_log_written: false,
            });
        }
        self.city_war_enemy_factions_changed = Some(true);
        self.city_war_enemy_factions.insert(enemy_id);
        let war_log_written =
            self.write_enemy_war_log(context, enemy_id, b"WS0160", None, true, true)?;
        Ok(FactionEnemyMutationReport {
            state_changed: true,
            changed_flag_set: true,
            war_log_written,
        })
    }

    /// Безусловно стирает city-war enemy и независимо от miss пытается писать `WS0161`.
    pub(crate) fn del_city_war_enemy_organizing<Context>(
        &mut self,
        enemy_id: i32,
        context: &mut Context,
    ) -> Result<FactionEnemyMutationReport, FactionEnemyMutationBlock>
    where
        Context: FactionEnemyMutationContext,
    {
        let state_changed = self.city_war_enemy_factions.remove(&enemy_id);
        let remaining_enemy_count = self.city_war_enemy_factions.len() as u32;
        let war_log_written = self.write_enemy_war_log(
            context,
            enemy_id,
            b"WS0161",
            Some(remaining_enemy_count),
            state_changed,
            false,
        )?;
        Ok(FactionEnemyMutationReport {
            state_changed,
            changed_flag_set: false,
            war_log_written,
        })
    }

    /// Дописывает полный standard enemy-set в исходном wire-формате.
    pub(crate) fn add_enemy_factions_to_byte_array(&self, output: &mut Vec<u8>) -> bool {
        append_signed_set(output, &self.enemy_factions);
        true
    }

    /// Дописывает полный city-war enemy-set в исходном wire-формате.
    pub(crate) fn add_city_war_enemy_factions_to_byte_array(
        &self,
        output: &mut Vec<u8>,
    ) -> bool {
        append_signed_set(output, &self.city_war_enemy_factions);
        true
    }

    fn update_enemy_set_to_client(
        &self,
        game: &CGame,
        kind: EnemyFactionSetKind,
    ) -> Vec<FactionEnemyDelivery> {
        let (message_type, enemy_factions) = match kind {
            EnemyFactionSetKind::Standard => (0x7FE11, &self.enemy_factions),
            EnemyFactionSetKind::CityWar => (0x7FE12, &self.city_war_enemy_factions),
        };
        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(message_type);
            message.base_mut().add_long(recipient_player_id);
            let mut serialized_set = Vec::new();
            append_signed_set(&mut serialized_set, enemy_factions);
            message.base_mut().add(&serialized_set);
            deliveries.push(FactionEnemyDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        deliveries
    }

    pub(crate) fn update_enemy_factions_to_client(
        &self,
        game: &CGame,
    ) -> Vec<FactionEnemyDelivery> {
        self.update_enemy_set_to_client(game, EnemyFactionSetKind::Standard)
    }

    pub(crate) fn update_city_war_enemy_factions_to_client(
        &self,
        game: &CGame,
    ) -> Vec<FactionEnemyDelivery> {
        self.update_enemy_set_to_client(game, EnemyFactionSetKind::CityWar)
    }

    /// Ставит standard changed-флаг, публикует set и обновляет online players.
    pub(crate) fn update_enemy_faction<F>(
        &mut self,
        game: &CGame,
        update_player: F,
    ) -> FactionEnemyRefreshReport
    where
        F: FnMut(i32),
    {
        self.enemy_factions_changed = Some(true);
        let deliveries = self.update_enemy_factions_to_client(game);
        let refreshed_player_ids = self.update_player_faction_info(game, 0, update_player);
        FactionEnemyRefreshReport {
            deliveries,
            refreshed_player_ids,
        }
    }

    /// Выполняет city-war refresh только при changed-флаге ровно `true`.
    pub(crate) fn update_city_war_enemy_faction<F>(
        &self,
        game: &CGame,
        update_player: F,
    ) -> CityWarEnemyRefreshOutcome
    where
        F: FnMut(i32),
    {
        match self.city_war_enemy_factions_changed {
            None => CityWarEnemyRefreshOutcome::ChangeFlagUnknown,
            Some(false) => CityWarEnemyRefreshOutcome::Unchanged,
            Some(true) => {
                let deliveries = self.update_city_war_enemy_factions_to_client(game);
                let refreshed_player_ids =
                    self.update_player_faction_info(game, 0, update_player);
                CityWarEnemyRefreshOutcome::Published(FactionEnemyRefreshReport {
                    deliveries,
                    refreshed_player_ids,
                })
            }
        }
    }

    /// Рассылает current/upgrade exp только contributor-ам и master-у.
    pub(crate) fn update_experience_to_client(
        &self,
        game: &CGame,
    ) -> Result<Vec<FactionExperienceDelivery>, FactionExperienceBlock> {
        let property = self
            .base_property
            .ok_or(FactionExperienceBlock::MissingBaseProperty)?;
        let master_id = self
            .master_id
            .ok_or(FactionExperienceBlock::MasterIdMissing)?;
        let mut deliveries = Vec::new();
        for (&recipient_player_id, member) in &self.members {
            if !member.contribute && recipient_player_id != master_id {
                continue;
            }
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(0x7FE14);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(property.experience());
            message.base_mut().add_long(property.upgrade_experience());
            deliveries.push(FactionExperienceDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    /// Рассылает полный pronounce-state всем готовым member recipient-ам.
    pub(crate) fn update_pronounce_to_client(
        &self,
        game: &CGame,
        operator: EOperator,
    ) -> Result<Vec<FactionPronounceDelivery>, FactionPronounceUpdateBuildError> {
        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(PRONOUNCE_UPDATE_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(operator.wire_value());
            message.base_mut().add_long(self.pronounce.player_id);
            let name = match self.pronounce.name_wire_bytes() {
                Ok(name) => name,
                Err(source) => {
                    return Err(FactionPronounceUpdateBuildError {
                        source,
                        recipient_player_id,
                        game_server_id,
                        completed_deliveries: deliveries,
                    });
                }
            };
            message.base_mut().add(name);
            let content = match self.pronounce.content_wire_bytes() {
                Ok(content) => content,
                Err(source) => {
                    return Err(FactionPronounceUpdateBuildError {
                        source,
                        recipient_player_id,
                        game_server_id,
                        completed_deliveries: deliveries,
                    });
                }
            };
            message.base_mut().add(content);
            message.base_mut().add(&self.pronounce.time.wire_bytes());
            deliveries.push(FactionPronounceDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    /// Публикует полный apply-list одному готовому клиенту без проверки права.
    pub(crate) fn update_all_apply_members_to_client(
        &self,
        game: &CGame,
        recipient_player_id: i32,
    ) -> Result<Vec<FactionApplyMemberDelivery>, FactionApplyMemberUpdateBuildError> {
        let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
        let player = game.online_player_by_id(recipient_player_id as u32);
        if game_server_id == 0 || player.is_none_or(|player| !player.faction_data_received()) {
            return Ok(Vec::new());
        }

        let mut deliveries = Vec::new();
        for person in self.apply_persons.values() {
            let Some(name) = person.name_wire_bytes() else {
                return Err(FactionApplyMemberUpdateBuildError {
                    candidate_player_id: person.id,
                    recipient_player_id,
                    game_server_id,
                    completed_deliveries: deliveries,
                });
            };

            let mut message = CMessage::new(APPLY_MEMBER_UPDATE_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(EOperator::Add.wire_value());
            message.base_mut().add_long(person.id);
            message.base_mut().add(name);
            message.base_mut().add_long(person.occupation);
            message.base_mut().add_long(person.level);
            deliveries.push(FactionApplyMemberDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    /// Публикует один apply-record только членам с правом `PV_ConMem`.
    pub(crate) fn update_apply_member_to_client(
        &self,
        game: &CGame,
        candidate_player_id: i32,
        operator: EOperator,
    ) -> Result<Vec<FactionApplyMemberDelivery>, FactionApplyMemberUpdateBuildError> {
        let person = self
            .apply_persons
            .get(&candidate_player_id)
            .copied()
            .unwrap_or_else(|| TagApplyPerson::empty_with_id(candidate_player_id));
        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received())
                || game_server_id == 0
                || !self.is_using_purview(recipient_player_id, EPurview::ConMem as i32)
            {
                continue;
            }

            let mut message = CMessage::new(APPLY_MEMBER_UPDATE_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(operator.wire_value());
            message.base_mut().add_long(person.id);
            let Some(name) = person.name_wire_bytes() else {
                return Err(FactionApplyMemberUpdateBuildError {
                    candidate_player_id: person.id,
                    recipient_player_id,
                    game_server_id,
                    completed_deliveries: deliveries,
                });
            };
            message.base_mut().add(name);
            message.base_mut().add_long(person.occupation);
            message.base_mut().add_long(person.level);
            deliveries.push(FactionApplyMemberDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    /// Удаляет кандидата до публикации пустого `OP_Delete`-record-а.
    pub(crate) fn remove_apply_member(
        &mut self,
        game: &CGame,
        candidate_player_id: i32,
    ) -> FactionRemoveApplyMemberOutcome {
        if self.apply_persons.remove(&candidate_player_id).is_none() {
            return FactionRemoveApplyMemberOutcome::NotFound;
        }

        let deliveries =
            self.update_apply_member_to_client(game, candidate_player_id, EOperator::Delete);
        self.set_change_data(8);
        FactionRemoveApplyMemberOutcome::Removed {
            faction_id: self.faction_id,
            deliveries,
        }
    }

    /// Создаёт заявку после полной исходной цепочки war/limit/member gates.
    pub(crate) fn apply_for_join<Context>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        player_id: i32,
        _legacy_second_parameter: i32,
        _legacy_third_parameter: i32,
        context: &mut Context,
    ) -> Result<FactionApplyForJoinOutcome, FactionApplyForJoinBlock<Context::Block>>
    where
        Context: FactionApplyForJoinContext + ?Sized,
    {
        if self.has_enemy_faction() || context.already_declared_for_village_war(self.faction_id) {
            send_apply_join_information(context, player_id, b"WS0162", b"WS0121");
            return Ok(FactionApplyForJoinOutcome::Rejected(
                FactionApplyForJoinRejection::StandardOrVillageWar,
            ));
        }
        if self.has_city_war_enemy_faction()
            || context.already_declared_for_city_war(self.faction_id)
        {
            send_apply_join_information(context, player_id, b"WS0163", b"WS0121");
            return Ok(FactionApplyForJoinOutcome::Rejected(
                FactionApplyForJoinRejection::CityWar,
            ));
        }

        let level = self
            .level()
            .ok_or(FactionApplyForJoinBlock::MissingBaseProperty)?;
        let maximum_members = parameters.get_max_number_by_level(level);
        if self.members.len() as u32 as i32 >= maximum_members {
            send_apply_join_information(context, player_id, b"WS0164", b"WS0121");
            return Ok(FactionApplyForJoinOutcome::Rejected(
                FactionApplyForJoinRejection::MemberLimit,
            ));
        }
        if self.apply_persons.len() as u32 >= MAX_APPLY_PERSON_COUNT {
            send_apply_join_information(context, player_id, b"WS0165", b"WS0121");
            return Ok(FactionApplyForJoinOutcome::Rejected(
                FactionApplyForJoinRejection::ApplyListLimit,
            ));
        }

        if context
            .player_already_in_faction(player_id)
            .map_err(|source| FactionApplyForJoinBlock::Context {
                operation: FactionApplyForJoinContextOperation::PlayerMembershipLookup,
                source,
            })?
        {
            return Ok(FactionApplyForJoinOutcome::Rejected(
                FactionApplyForJoinRejection::PlayerAlreadyInFaction,
            ));
        }
        context
            .remove_previous_faction_applications(game, player_id)
            .map_err(|source| FactionApplyForJoinBlock::Context {
                operation: FactionApplyForJoinContextOperation::RemovePreviousApplications,
                source,
            })?;

        let Some(player) = game.online_player_by_id(player_id as u32) else {
            return Ok(FactionApplyForJoinOutcome::Rejected(
                FactionApplyForJoinRejection::PlayerOffline,
            ));
        };
        let player_name = legacy_c_string_visible_bytes(player.get_name());
        if player_name.len() >= APPLY_PERSON_NAME_CAPACITY {
            return Err(FactionApplyForJoinBlock::PlayerNameWouldOverflow {
                player_id,
                visible_len: player_name.len(),
            });
        }
        let mut name = [0; APPLY_PERSON_NAME_CAPACITY];
        name[..player_name.len()].copy_from_slice(player_name);
        let person = TagApplyPerson::from_complete_fields(
            player_id,
            name,
            i32::from(player.get_occupation()),
            i32::from(player.get_level()),
        );
        self.apply_persons.insert(player_id, person);

        let formatted =
            context.format_world_string(b"WS0166", &[legacy_c_string_visible_bytes(&self.name)]);
        let formatted = legacy_c_string_visible_bytes(&formatted);
        if formatted.len() >= APPLY_JOIN_NOTICE_CAPACITY {
            return Err(FactionApplyForJoinBlock::SuccessNoticeWouldOverflow {
                player_id,
                formatted_len: formatted.len(),
                application_inserted: true,
            });
        }
        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        context.send_organizing_info(FactionMemberInfoRequest {
            recipient_player_id: player_id,
            first_text: formatted,
            second_text: legacy_c_string_visible_bytes(&second_text),
            information_type: -1,
            color: 0xFFDA_EDFE,
            trailing_value: 0,
        });

        let deliveries = self.update_apply_member_to_client(game, player_id, EOperator::Add);
        self.set_change_data(8);
        let log_written = context.faction_apply_log_enabled();
        if log_written {
            context.write_faction_apply_log(
                self.faction_id,
                legacy_c_string_visible_bytes(&self.name),
                player_id,
                player_name,
                2,
            );
        }
        Ok(FactionApplyForJoinOutcome::Applied {
            deliveries,
            log_written,
        })
    }

    /// Выпускает участника из faction с исходным порядком внешних эффектов.
    pub(crate) fn exit<Context>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        player_id: i32,
        context: &mut Context,
    ) -> Result<FactionExitOutcome, FactionExitBlock>
    where
        Context: FactionExitContext,
    {
        if self.has_enemy_faction() || context.already_declared_for_village_war(self.faction_id) {
            send_apply_join_information(context, player_id, b"WS0162", b"WS0121");
            return Ok(FactionExitOutcome::Rejected {
                reason: FactionExitRejection::StandardOrVillageWar,
                goods_war_notice_sent: false,
            });
        }
        if self.has_city_war_enemy_faction()
            || context.already_declared_for_city_war(self.faction_id)
        {
            send_apply_join_information(context, player_id, b"WS0163", b"WS0121");
            return Ok(FactionExitOutcome::Rejected {
                reason: FactionExitRejection::CityWar,
                goods_war_notice_sent: false,
            });
        }

        let goods_war_notice_sent = context.goods_war_blocks_exit(self.faction_id, player_id);
        if goods_war_notice_sent {
            send_apply_join_information(context, player_id, b"WS0162", b"WS0121");
        }

        if !self.is_using_purview(player_id, EPurview::Exit as i32) {
            return Ok(FactionExitOutcome::Rejected {
                reason: FactionExitRejection::PermissionDenied,
                goods_war_notice_sent,
            });
        }

        let member = self
            .members
            .get(&player_id)
            .expect("PV_Exit может принадлежать только существующему member");
        let player_name_wire = member.name_wire_bytes().map_err(|_| {
            FactionExitBlock::UnterminatedMemberName {
                player_id,
                goods_war_notice_sent,
            }
        })?;
        let player_name = player_name_wire[..player_name_wire.len() - 1].to_vec();
        let notice = context.format_world_string(b"WS0187", &[&player_name]);
        let notice = legacy_c_string_visible_bytes(&notice);
        if notice.len() >= FACTION_MEMBER_NOTICE_CAPACITY {
            return Err(FactionExitBlock::NoticeWouldOverflow {
                player_id,
                formatted_len: notice.len(),
                goods_war_notice_sent,
            });
        }
        let second_text = context.world_string(b"WS0188").unwrap_or_default();
        let member_information = self.send_info_to_all_members(
            notice,
            legacy_c_string_visible_bytes(&second_text),
            -1,
            |request| context.send_organizing_info(request),
        );

        let member_removal = match self.del_member(player_id, parameters) {
            Ok(report) => report,
            Err(source) => {
                return Err(FactionExitBlock::DelMember {
                    source,
                    goods_war_notice_sent,
                    member_information,
                    member_removed: !self.members.contains_key(&player_id),
                });
            }
        };
        let refreshed_player_ids =
            self.update_player_faction_info(game, player_id, |player_id| {
                context.update_player_faction_info(player_id);
            });
        let delete_organizing = match self.delete_organizing_to_client(game, player_id, context) {
            Ok(outcome) => outcome,
            Err(source) => {
                return Err(FactionExitBlock::DeleteOrganizing {
                    source,
                    goods_war_notice_sent,
                    member_information,
                    member_removal,
                    refreshed_player_ids,
                });
            }
        };
        let member_update = self.update_member_info_to_client(game, player_id, EOperator::Delete);
        self.set_change_data(2);

        let log_written = context.faction_quit_log_enabled();
        if log_written {
            context.write_faction_quit_log(
                self.faction_id,
                legacy_c_string_visible_bytes(&self.name),
                player_id,
                &player_name,
                3,
            );
        }
        context.delete_goods_war_member(player_id);

        Ok(FactionExitOutcome::Exited {
            goods_war_notice_sent,
            member_information,
            member_removal,
            delete_organizing,
            refreshed_player_ids,
            member_update,
            log_written,
        })
    }

    /// Исключает участника с точным FireOut-порядком уведомлений и callback-ов.
    pub(crate) fn fire_out<Context>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        manager_id: i32,
        target_id: i32,
        context: &mut Context,
    ) -> Result<FactionFireOutOutcome, FactionFireOutBlock>
    where
        Context: FactionFireOutContext,
    {
        if self.has_enemy_faction() || context.already_declared_for_village_war(self.faction_id) {
            send_apply_join_information(context, manager_id, b"WS0162", b"WS0121");
            return Ok(FactionFireOutOutcome::Rejected(
                FactionFireOutRejection::StandardOrVillageWar,
            ));
        }
        if self.has_city_war_enemy_faction()
            || context.already_declared_for_city_war(self.faction_id)
        {
            send_apply_join_information(context, manager_id, b"WS0163", b"WS0121");
            return Ok(FactionFireOutOutcome::Rejected(
                FactionFireOutRejection::CityWar,
            ));
        }
        if context.goods_war_blocks_fire_out(self.faction_id, manager_id) {
            send_apply_join_information(context, manager_id, b"WS0363 ", b"WS0121");
            return Ok(FactionFireOutOutcome::Rejected(
                FactionFireOutRejection::GoodsWar,
            ));
        }

        let operation_valid = self
            .check_operator_validate_target(manager_id, target_id, EPurview::FireOut as i32)
            .map_err(FactionFireOutBlock::OperatorValidation)?;
        if !operation_valid {
            return Ok(FactionFireOutOutcome::Rejected(
                FactionFireOutRejection::OperatorValidationFailed,
            ));
        }

        let target = self
            .members
            .get(&target_id)
            .expect("успешный CheckOperValidate гарантирует target member");
        let target_name_wire = target
            .name_wire_bytes()
            .map_err(|_| FactionFireOutBlock::UnterminatedTargetName { target_id })?;
        let target_name = target_name_wire[..target_name_wire.len() - 1].to_vec();
        let notice = context.format_world_string(b"WS0192", &[&target_name]);
        let notice = legacy_c_string_visible_bytes(&notice);
        if notice.len() >= FACTION_MEMBER_NOTICE_CAPACITY {
            return Err(FactionFireOutBlock::NoticeWouldOverflow {
                target_id,
                formatted_len: notice.len(),
            });
        }
        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        let member_information = self.send_info_to_all_members(
            notice,
            legacy_c_string_visible_bytes(&second_text),
            -1,
            |request| context.send_organizing_info(request),
        );

        let member_removal = match self.del_member(target_id, parameters) {
            Ok(report) => report,
            Err(source) => {
                return Err(FactionFireOutBlock::DelMember {
                    source,
                    member_information,
                    member_removed: !self.members.contains_key(&target_id),
                });
            }
        };
        let delete_organizing = match self.delete_organizing_to_client(game, target_id, context) {
            Ok(outcome) => outcome,
            Err(source) => {
                return Err(FactionFireOutBlock::DeleteOrganizing {
                    source,
                    member_information,
                    member_removal,
                });
            }
        };
        let member_update = self.update_member_info_to_client(game, target_id, EOperator::Delete);
        let refreshed_player_ids =
            self.update_player_faction_info(game, target_id, |player_id| {
                context.update_player_faction_info(player_id);
            });
        self.set_change_data(2);

        let log_written = context.faction_fire_out_log_enabled();
        if log_written {
            let manager = self
                .members
                .get(&manager_id)
                .expect("успешный CheckOperValidate сохраняет manager member");
            let manager_name = match manager.name_wire_bytes() {
                Ok(name) => &name[..name.len() - 1],
                Err(_) => {
                    return Err(FactionFireOutBlock::UnterminatedManagerName {
                        manager_id,
                        member_information,
                        member_removal,
                        refreshed_player_ids,
                        member_update,
                        delete_organizing,
                    });
                }
            };
            context.write_faction_fire_out_log(
                target_id,
                &target_name,
                manager.id,
                manager_name,
                self.faction_id,
                legacy_c_string_visible_bytes(&self.name),
                1,
            );
        }
        context.delete_goods_war_member(target_id);

        let mut message = CMessage::new(FACTION_MEMBER_REMOVED_LOCAL_MESSAGE_TYPE);
        message.base_mut().add_long(target_id);
        let mut fixed_faction_name = [0; FACTION_MEMBER_NAME_CAPACITY];
        let copied_name_len = self.name.len().min(FACTION_MEMBER_NAME_CAPACITY);
        fixed_faction_name[..copied_name_len].copy_from_slice(&self.name[..copied_name_len]);
        message.base_mut().add(&fixed_faction_name);
        let local_message = game.queue_local_world_message(message);

        Ok(FactionFireOutOutcome::Fired {
            member_information,
            member_removal,
            delete_organizing,
            member_update,
            refreshed_player_ids,
            log_written,
            local_message,
        })
    }

    /// Изменяет title и job-level двумя независимыми исходными ветками.
    pub(crate) fn dub_and_set_job_level<Context>(
        &mut self,
        game: &CGame,
        manager_id: i32,
        target_id: i32,
        title: &mut Vec<u8>,
        job_level: i32,
        context: &mut Context,
    ) -> Result<FactionDubOutcome, FactionDubBlock>
    where
        Context: FactionDubContext,
    {
        let mut progress = FactionDubProgress {
            input_title_truncated: false,
            title_changed: false,
            job_level_changed: false,
            title_information: None,
            title_refreshed_player_ids: Vec::new(),
            job_level_information: None,
            member_update: None,
            dirty_set: false,
        };

        if !context.check_invalid_string(title, false) {
            send_apply_join_information(context, manager_id, b"WS0194", b"WS0193");
            return Ok(FactionDubOutcome::Rejected {
                reason: FactionDubRejection::InvalidTitle,
                input_title_truncated: false,
            });
        }
        let operation_valid = self
            .check_operator_validate_target(manager_id, target_id, EPurview::DubJobLevel as i32)
            .map_err(FactionDubBlock::OperatorValidation)?;
        if !operation_valid {
            return Ok(FactionDubOutcome::Rejected {
                reason: FactionDubRejection::OperatorValidationFailed,
                input_title_truncated: false,
            });
        }
        if !(1..=99).contains(&job_level) {
            return Ok(FactionDubOutcome::Rejected {
                reason: FactionDubRejection::InvalidJobLevel,
                input_title_truncated: false,
            });
        }

        if 20 < title.len() {
            title.truncate(20);
            progress.input_title_truncated = true;
        }
        let Some(target) = self.members.get(&target_id) else {
            return Ok(FactionDubOutcome::Rejected {
                reason: FactionDubRejection::TargetNotFound,
                input_title_truncated: progress.input_title_truncated,
            });
        };
        let target_title_wire = match target.title_wire_bytes() {
            Ok(value) => value,
            Err(source) => {
                return Err(FactionDubBlock::UnterminatedMemberField {
                    player_id: target_id,
                    source,
                    progress,
                });
            }
        };
        let target_title = &target_title_wire[..target_title_wire.len() - 1];
        let new_title = legacy_c_string_visible_bytes(title).to_vec();
        let mut old_title = Vec::new();

        if target_title != new_title {
            if target_title.len() >= FACTION_DUB_OLD_TITLE_CAPACITY {
                return Err(FactionDubBlock::OldTitleWouldOverflow {
                    target_id,
                    visible_len: target_title.len(),
                    progress,
                });
            }
            old_title.extend_from_slice(target_title);
            let target = self
                .members
                .get_mut(&target_id)
                .expect("target найден до title-ветки");
            target.title[..new_title.len()].copy_from_slice(&new_title);
            target.title[new_title.len()] = 0;
            progress.title_changed = true;

            let target = self
                .members
                .get(&target_id)
                .expect("title-ветка не удаляет target");
            let target_name_wire = match target.name_wire_bytes() {
                Ok(value) => value,
                Err(source) => {
                    return Err(FactionDubBlock::UnterminatedMemberField {
                        player_id: target_id,
                        source,
                        progress,
                    });
                }
            };
            let target_name = &target_name_wire[..target_name_wire.len() - 1];
            let notice = context.format_world_string(
                b"WS0195",
                &[
                    FactionDubFormatArgument::Text(target_name),
                    FactionDubFormatArgument::Text(&new_title),
                ],
            );
            let notice = legacy_c_string_visible_bytes(&notice);
            if notice.len() >= FACTION_DUB_NOTICE_CAPACITY {
                return Err(FactionDubBlock::NoticeWouldOverflow {
                    notice: FactionDubNotice::Title,
                    formatted_len: notice.len(),
                    progress,
                });
            }
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            progress.title_information = Some(self.send_info_to_all_members(
                notice,
                legacy_c_string_visible_bytes(&second_text),
                -1,
                |request| context.send_organizing_info(request),
            ));
            progress.title_refreshed_player_ids =
                self.update_player_faction_info(game, target_id, |player_id| {
                    context.update_player_faction_info(player_id);
                });
        }

        let current_job_level = self
            .members
            .get(&target_id)
            .expect("target найден до job-level ветки")
            .job_level;
        if current_job_level != job_level {
            self.members
                .get_mut(&target_id)
                .expect("target найден до job-level записи")
                .job_level = job_level;
            progress.job_level_changed = true;

            let target = self
                .members
                .get(&target_id)
                .expect("job-level ветка не удаляет target");
            let target_name_wire = match target.name_wire_bytes() {
                Ok(value) => value,
                Err(source) => {
                    return Err(FactionDubBlock::UnterminatedMemberField {
                        player_id: target_id,
                        source,
                        progress,
                    });
                }
            };
            let target_name = &target_name_wire[..target_name_wire.len() - 1];
            let notice = context.format_world_string(
                b"WS0196",
                &[
                    FactionDubFormatArgument::Text(target_name),
                    FactionDubFormatArgument::Signed(job_level),
                ],
            );
            let notice = legacy_c_string_visible_bytes(&notice);
            if notice.len() >= FACTION_DUB_NOTICE_CAPACITY {
                return Err(FactionDubBlock::NoticeWouldOverflow {
                    notice: FactionDubNotice::JobLevel,
                    formatted_len: notice.len(),
                    progress,
                });
            }
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            progress.job_level_information = Some(self.send_info_to_all_members(
                notice,
                legacy_c_string_visible_bytes(&second_text),
                -1,
                |request| context.send_organizing_info(request),
            ));
        }

        progress.member_update = Some(self.update_member_info_to_client(
            game,
            target_id,
            EOperator::Update,
        ));
        self.set_change_data(2);
        progress.dirty_set = true;

        let log_written = context.faction_title_log_enabled();
        if log_written {
            let target = self
                .members
                .get(&target_id)
                .expect("успешный owner сохраняет target member");
            let target_name_wire = match target.name_wire_bytes() {
                Ok(value) => value,
                Err(source) => {
                    return Err(FactionDubBlock::UnterminatedMemberField {
                        player_id: target_id,
                        source,
                        progress,
                    });
                }
            };
            let manager = self
                .members
                .get(&manager_id)
                .expect("успешный CheckOperValidate сохраняет manager member");
            let manager_name_wire = match manager.name_wire_bytes() {
                Ok(value) => value,
                Err(source) => {
                    return Err(FactionDubBlock::UnterminatedMemberField {
                        player_id: manager_id,
                        source,
                        progress,
                    });
                }
            };
            context.write_faction_title_log(
                target.id,
                &target_name_wire[..target_name_wire.len() - 1],
                &old_title,
                &new_title,
                manager.id,
                &manager_name_wire[..manager_name_wire.len() - 1],
                self.faction_id,
                legacy_c_string_visible_bytes(&self.name),
            );
        }

        Ok(FactionDubOutcome::Updated {
            progress,
            log_written,
        })
    }

    pub(crate) fn endue_right_to_member<Context>(
        &mut self,
        game: &CGame,
        manager_id: i32,
        target_id: i32,
        purview: i32,
        context: &mut Context,
    ) -> Result<FactionPurviewChangeOutcome, FactionPurviewChangeBlock>
    where
        Context: FactionPurviewChangeContext,
    {
        self.change_member_purview(
            game,
            manager_id,
            target_id,
            purview,
            FactionPurviewChange::Grant,
            context,
        )
    }

    pub(crate) fn abolish_right_to_member<Context>(
        &mut self,
        game: &CGame,
        manager_id: i32,
        target_id: i32,
        purview: i32,
        context: &mut Context,
    ) -> Result<FactionPurviewChangeOutcome, FactionPurviewChangeBlock>
    where
        Context: FactionPurviewChangeContext,
    {
        self.change_member_purview(
            game,
            manager_id,
            target_id,
            purview,
            FactionPurviewChange::Revoke,
            context,
        )
    }

    /// Меняет maximum member count и публикует `WS0186` только при отличии.
    pub(crate) fn set_maximum_members<Context>(
        &mut self,
        maximum_members: i32,
        context: &mut Context,
    ) -> Result<FactionMaximumMembersUpdate, FactionMaximumMembersBlock>
    where
        Context: FactionLevelContext,
    {
        let property = self
            .base_property
            .ok_or(FactionMaximumMembersBlock::MissingBaseProperty)?;
        if property.signed_at(0x1C) == maximum_members {
            return Ok(FactionMaximumMembersUpdate::Unchanged);
        }
        self.base_property
            .as_mut()
            .ok_or(FactionMaximumMembersBlock::MissingBaseProperty)?
            .write_signed(0x1C, maximum_members);

        let notice = context.format_world_string_signed(b"WS0186", maximum_members);
        let notice = legacy_c_string_visible_bytes(&notice);
        if notice.len() >= FACTION_MAXIMUM_MEMBERS_NOTICE_CAPACITY {
            return Err(FactionMaximumMembersBlock::NoticeWouldOverflow {
                maximum_members,
                formatted_len: notice.len(),
                property_changed: true,
            });
        }
        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        Ok(FactionMaximumMembersUpdate::Updated(
            self.send_info_to_all_members(
                notice,
                legacy_c_string_visible_bytes(&second_text),
                -1,
                |request| context.send_organizing_info(request),
            ),
        ))
    }

    /// Старый virtual принимал amount, не читал его и всегда возвращал `true`.
    pub(crate) const fn inc_maximum_number(&self, _amount: i32) -> bool {
        true
    }

    /// Меняет level и последовательно применяет шесть feature-флагов и maximum.
    pub(crate) fn set_level<Context>(
        &mut self,
        level: i32,
        parameters: &COrganizingParam,
        context: &mut Context,
    ) -> Result<FactionLevelUpdate, FactionLevelBlock>
    where
        Context: FactionLevelContext,
    {
        let property = self
            .base_property
            .ok_or(FactionLevelBlock::MissingBaseProperty {
                level_changed: false,
            })?;
        if !(1..=12).contains(&level) || property.level() == level {
            return Ok(FactionLevelUpdate::Unchanged);
        }
        self.base_property
            .as_mut()
            .ok_or(FactionLevelBlock::MissingBaseProperty {
                level_changed: false,
            })?
            .write_signed(0x00, level);

        let feature_block = |_| FactionLevelBlock::MissingBaseProperty {
            level_changed: true,
        };
        let pronounce = self
            .set_feature_function(
                FactionFeatureFunction::Pronounce,
                parameters.pronounce_minimum_level() <= level,
                context,
            )
            .map_err(feature_block)?;
        let leave_word = self
            .set_feature_function(
                FactionFeatureFunction::LeaveWord,
                parameters.leave_word_minimum_level() <= level,
                context,
            )
            .map_err(feature_block)?;
        let endue_right = self
            .set_feature_function(
                FactionFeatureFunction::EndueRight,
                parameters.endue_right_minimum_level() <= level,
                context,
            )
            .map_err(feature_block)?;
        let create_union = self
            .set_feature_function(
                FactionFeatureFunction::CreateUnion,
                parameters.create_union_minimum_level() <= level,
                context,
            )
            .map_err(feature_block)?;
        let join_village_war = self
            .set_feature_function(
                FactionFeatureFunction::JoinVillageWar,
                parameters.attack_village_minimum_level() <= level,
                context,
            )
            .map_err(feature_block)?;
        let join_city_war = self
            .set_feature_function(
                FactionFeatureFunction::JoinCityWar,
                parameters.attack_city_minimum_level() <= level,
                context,
            )
            .map_err(feature_block)?;
        let maximum_members = self
            .set_maximum_members(parameters.get_max_number_by_level(level), context)
            .map_err(|source| FactionLevelBlock::MaximumMembers {
                source,
                level_changed: true,
            })?;

        Ok(FactionLevelUpdate::Updated {
            pronounce,
            leave_word,
            endue_right,
            create_union,
            join_village_war,
            join_city_war,
            maximum_members,
        })
    }

    /// Диспетчеризует legacy `Level`/`Experience` и сохраняет общий postfix.
    pub(crate) fn set_parameter<Context>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        parameter: &[u8],
        value: i32,
        context: &mut Context,
    ) -> Result<FactionSetParameterOutcome, FactionSetParameterBlock>
    where
        Context: FactionSetParameterContext,
    {
        let parameter = match legacy_c_string_visible_bytes(parameter) {
            b"Level" => FactionSetParameterKind::Level,
            b"Experience" => FactionSetParameterKind::Experience,
            _ => FactionSetParameterKind::Unknown,
        };
        if parameter == FactionSetParameterKind::Level && value > 12 {
            return Ok(FactionSetParameterOutcome::LevelAboveMaximum {
                requested_level: value,
            });
        }

        let mut progress = empty_faction_set_parameter_progress(parameter);
        match parameter {
            FactionSetParameterKind::Level => {
                progress.level_update = match self.set_level(value, parameters, context) {
                    Ok(update) => Some(update),
                    Err(source) => {
                        return Err(FactionSetParameterBlock::Level { source, progress });
                    }
                };

                let level = match self.base_property {
                    Some(property) => property.level(),
                    None => {
                        return Err(FactionSetParameterBlock::MissingBaseProperty { progress });
                    }
                };
                let Some(level_parameters) = parameters.get_level_param(level) else {
                    return Ok(FactionSetParameterOutcome::LevelParametersMissing {
                        level,
                        progress,
                    });
                };
                let Some(property) = self.base_property.as_mut() else {
                    return Err(FactionSetParameterBlock::MissingBaseProperty { progress });
                };
                property.write_signed(0x20, level_parameters.experience);
                progress.assigned_upgrade_experience = Some(level_parameters.experience);
            }
            FactionSetParameterKind::Experience => {
                let experience_before = self.base_property.map(|property| property.experience());
                progress.experience_update = match self.set_experience(game, value) {
                    Ok(update) => {
                        if let FactionExperienceUpdate::Updated { experience, .. } = &update {
                            progress.assigned_experience = Some(*experience);
                            progress.dirty_bit_requested = true;
                        }
                        Some(update)
                    }
                    Err(source) => {
                        let experience_after =
                            self.base_property.map(|property| property.experience());
                        if experience_after != experience_before {
                            progress.assigned_experience = experience_after;
                            progress.dirty_bit_requested = true;
                        }
                        return Err(FactionSetParameterBlock::Experience { source, progress });
                    }
                };
            }
            FactionSetParameterKind::Unknown => {}
        }

        progress.property_deliveries = match self.update_property_to_client(game) {
            Ok(deliveries) => Some(deliveries),
            Err(_) => {
                return Err(FactionSetParameterBlock::MissingBaseProperty { progress });
            }
        };
        progress.refreshed_player_ids = Some(self.update_player_faction_info(
            game,
            0,
            |player_id| context.update_player_faction_info(player_id),
        ));
        self.set_change_data(1);
        progress.dirty_bit_requested = true;
        Ok(FactionSetParameterOutcome::Applied(progress))
    }

    /// Повышает faction-level с исходными проверками и частичными эффектами.
    pub(crate) fn upgrade<Context>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        player_id: i32,
        context: &mut Context,
    ) -> Result<FactionUpgradeOutcome, FactionUpgradeBlock>
    where
        Context: FactionUpgradeContext,
    {
        let mut progress = empty_faction_upgrade_progress();
        if self.is_master(player_id) == 0 {
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::PlayerNotMaster,
                notice_sent: false,
                progress,
            });
        }
        let property = match self.base_property {
            Some(property) => property,
            None => return Err(FactionUpgradeBlock::MissingBaseProperty { progress }),
        };
        let current_level = property.level();
        if current_level >= 12 {
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::MaximumLevel,
                notice_sent: false,
                progress,
            });
        }
        let Some(level_parameters) = parameters.get_level_param(current_level).cloned() else {
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::CurrentLevelParametersMissing,
                notice_sent: false,
                progress,
            });
        };
        let current_experience = property.experience();
        if current_experience < level_parameters.experience {
            let notice = match format_faction_upgrade_notice(
                context,
                FactionUpgradeNotice::Experience,
                &[FactionUpgradeFormatArgument::Signed(
                    level_parameters.experience,
                )],
            ) {
                Ok(notice) => notice,
                Err(formatted_len) => {
                    return Err(FactionUpgradeBlock::NoticeWouldOverflow {
                        notice: FactionUpgradeNotice::Experience,
                        formatted_len,
                        progress,
                    });
                }
            };
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text: &notice,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::InsufficientExperience,
                notice_sent: true,
                progress,
            });
        }

        let Some(player) = game.online_player_by_id(player_id as u32) else {
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::PlayerOffline,
                notice_sent: false,
                progress,
            });
        };
        let player_money = match context.player_money(player_id) {
            Some(money) => money,
            None => {
                return Err(FactionUpgradeBlock::MissingPlayerMoney {
                    player_id,
                    progress,
                });
            }
        };
        if player_money < level_parameters.money as u32 {
            let notice = match format_faction_upgrade_notice(
                context,
                FactionUpgradeNotice::Money,
                &[FactionUpgradeFormatArgument::Signed(level_parameters.money)],
            ) {
                Ok(notice) => notice,
                Err(formatted_len) => {
                    return Err(FactionUpgradeBlock::NoticeWouldOverflow {
                        notice: FactionUpgradeNotice::Money,
                        formatted_len,
                        progress,
                    });
                }
            };
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text: &notice,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::InsufficientMoney,
                notice_sent: true,
                progress,
            });
        }
        if i32::from(player.get_level()) < level_parameters.master_level {
            let notice = match format_faction_upgrade_notice(
                context,
                FactionUpgradeNotice::MasterLevel,
                &[FactionUpgradeFormatArgument::Signed(
                    level_parameters.master_level,
                )],
            ) {
                Ok(notice) => notice,
                Err(formatted_len) => {
                    return Err(FactionUpgradeBlock::NoticeWouldOverflow {
                        notice: FactionUpgradeNotice::MasterLevel,
                        formatted_len,
                        progress,
                    });
                }
            };
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text: &notice,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::MasterLevelTooLow,
                notice_sent: true,
                progress,
            });
        }

        let goods_original = legacy_c_string_visible_bytes(&level_parameters.goods);
        if level_parameters.goods.as_slice() != b"0"
            && context.goods_in_packet(player_id, goods_original) < 1
        {
            let display_name = context
                .goods_display_name(goods_original)
                .unwrap_or_else(|| goods_original.to_vec());
            let notice = match format_faction_upgrade_notice(
                context,
                FactionUpgradeNotice::Goods,
                &[FactionUpgradeFormatArgument::Text(
                    legacy_c_string_visible_bytes(&display_name),
                )],
            ) {
                Ok(notice) => notice,
                Err(formatted_len) => {
                    return Err(FactionUpgradeBlock::NoticeWouldOverflow {
                        notice: FactionUpgradeNotice::Goods,
                        formatted_len,
                        progress,
                    });
                }
            };
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text: &notice,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::RequiredGoodsMissing,
                notice_sent: true,
                progress,
            });
        }

        let level_update = match self.set_level(current_level.wrapping_add(1), parameters, context) {
            Ok(update) => update,
            Err(source) => {
                return Err(FactionUpgradeBlock::Level { source, progress });
            }
        };
        progress.level_update = Some(level_update);
        let experience_update = match self.set_experience(
            game,
            current_experience.wrapping_sub(level_parameters.experience),
        ) {
            Ok(update) => update,
            Err(source) => {
                return Err(FactionUpgradeBlock::Experience { source, progress });
            }
        };
        progress.experience_update = Some(experience_update);

        let game_server_id = game.game_server_number_by_player_id(player_id);
        let mut charge = CMessage::new(FACTION_UPGRADE_CHARGE_MESSAGE_TYPE);
        charge.base_mut().add_long(player_id);
        charge.base_mut().add_long(level_parameters.money);
        charge
            .base_mut()
            .add(&legacy_c_string_wire_bytes(&level_parameters.goods));
        progress.charge_delivery = Some(FactionUpgradeChargeDelivery {
            game_server_id,
            result: game.send_msg_to_game_server(game_server_id, &charge),
        });

        let upgraded_level = match self.level() {
            Some(level) => level,
            None => return Err(FactionUpgradeBlock::MissingBaseProperty { progress }),
        };
        let Some(next_level_parameters) = parameters.get_level_param(upgraded_level) else {
            return Ok(FactionUpgradeOutcome::Rejected {
                reason: FactionUpgradeRejection::NextLevelParametersMissing,
                notice_sent: false,
                progress,
            });
        };
        let next_upgrade_experience = next_level_parameters.experience;
        match self.base_property.as_mut() {
            Some(property) => property.write_signed(0x20, next_upgrade_experience),
            None => return Err(FactionUpgradeBlock::MissingBaseProperty { progress }),
        }
        progress.next_upgrade_experience = Some(next_upgrade_experience);

        let notice = match format_faction_upgrade_notice(
            context,
            FactionUpgradeNotice::Success,
            &[FactionUpgradeFormatArgument::Signed(upgraded_level)],
        ) {
            Ok(notice) => notice,
            Err(formatted_len) => {
                return Err(FactionUpgradeBlock::NoticeWouldOverflow {
                    notice: FactionUpgradeNotice::Success,
                    formatted_len,
                    progress,
                });
            }
        };
        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        progress.member_information = Some(self.send_info_to_all_members(
            &notice,
            legacy_c_string_visible_bytes(&second_text),
            -1,
            |request| context.send_organizing_info(request),
        ));
        progress.property_deliveries = Some(match self.update_property_to_client(game) {
            Ok(deliveries) => deliveries,
            Err(_) => return Err(FactionUpgradeBlock::MissingBaseProperty { progress }),
        });
        self.set_change_data(1);
        progress.dirty_set = true;
        progress.refreshed_player_ids = self.update_player_faction_info(game, 0, |player_id| {
            context.update_player_faction_info(player_id);
        });

        if context.faction_level_log_enabled() {
            let member = self
                .members
                .get(&player_id)
                .expect("master invariant и Upgrade не удаляют member");
            let member_name_wire = match member.name_wire_bytes() {
                Ok(value) => value,
                Err(source) => {
                    return Err(FactionUpgradeBlock::UnterminatedMemberName {
                        player_id,
                        source,
                        progress,
                    });
                }
            };
            context.write_faction_level_log(
                self.faction_id,
                legacy_c_string_visible_bytes(&self.name),
                upgraded_level,
                member.id,
                &member_name_wire[..member_name_wire.len() - 1],
            );
            progress.log_written = true;
        }

        Ok(FactionUpgradeOutcome::Upgraded(progress))
    }

    #[allow(clippy::too_many_arguments)]
    fn change_member_purview<Context>(
        &mut self,
        game: &CGame,
        manager_id: i32,
        target_id: i32,
        purview: i32,
        change: FactionPurviewChange,
        context: &mut Context,
    ) -> Result<FactionPurviewChangeOutcome, FactionPurviewChangeBlock>
    where
        Context: FactionPurviewChangeContext,
    {
        let property = self
            .base_property
            .ok_or(FactionPurviewChangeBlock::MissingBaseProperty)?;
        if !property.feature_function(FactionFeatureFunction::EndueRight) {
            return Ok(FactionPurviewChangeOutcome::Rejected(
                FactionPurviewChangeRejection::FunctionDisabled,
            ));
        }
        let operation_valid = self
            .check_operator_validate_target(manager_id, target_id, EPurview::EndueRor as i32)
            .map_err(FactionPurviewChangeBlock::OperatorValidation)?;
        if !operation_valid {
            return Ok(FactionPurviewChangeOutcome::Rejected(
                FactionPurviewChangeRejection::OperatorValidationFailed,
            ));
        }
        if !(EPurview::DubJobLevel as i32..=EPurview::OperCityGate as i32).contains(&purview) {
            return Ok(FactionPurviewChangeOutcome::Rejected(
                FactionPurviewChangeRejection::InvalidPurview,
            ));
        }

        let mutation = match change {
            FactionPurviewChange::Grant => self.set_member_purview(target_id, purview),
            FactionPurviewChange::Revoke => self.abolish_member_purview(target_id, purview),
        };
        let member_update =
            self.update_member_info_to_client(game, target_id, EOperator::Update);
        self.set_change_data(2);
        let apply_snapshot = if change == FactionPurviewChange::Grant
            && purview == EPurview::ConMem as i32
        {
            Some(self.update_all_apply_members_to_client(game, target_id))
        } else {
            None
        };
        let mut progress = FactionPurviewChangeProgress {
            mutation,
            member_update,
            dirty_set: true,
            apply_snapshot,
            member_information: None,
        };

        let target = self
            .members
            .get(&target_id)
            .expect("успешный CheckOperValidate гарантирует target member");
        let target_name_wire = match target.name_wire_bytes() {
            Ok(value) => value,
            Err(source) => {
                return Err(FactionPurviewChangeBlock::UnterminatedMemberName {
                    player_id: target_id,
                    source,
                    progress,
                });
            }
        };
        let target_name = target_name_wire[..target_name_wire.len() - 1].to_vec();
        let string_id = faction_purview_notice_string_id(change, purview);
        let notice = context.format_world_string(string_id, &target_name);
        let notice = legacy_c_string_visible_bytes(&notice);
        if notice.len() >= FACTION_PURVIEW_NOTICE_CAPACITY {
            return Err(FactionPurviewChangeBlock::NoticeWouldOverflow {
                formatted_len: notice.len(),
                progress,
            });
        }
        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        progress.member_information = Some(self.send_info_to_all_members(
            notice,
            legacy_c_string_visible_bytes(&second_text),
            -1,
            |request| context.send_organizing_info(request),
        ));

        let log_written = context.faction_purview_log_enabled(change);
        if log_written {
            let target = self
                .members
                .get(&target_id)
                .expect("permission owner не удаляет target");
            let manager = self
                .members
                .get(&manager_id)
                .expect("успешный CheckOperValidate гарантирует manager member");
            let manager_name_wire = match manager.name_wire_bytes() {
                Ok(value) => value,
                Err(source) => {
                    return Err(FactionPurviewChangeBlock::UnterminatedMemberName {
                        player_id: manager_id,
                        source,
                        progress,
                    });
                }
            };
            context.write_faction_purview_log(
                target.id,
                &target_name,
                purview,
                manager.id,
                &manager_name_wire[..manager_name_wire.len() - 1],
                self.faction_id,
                legacy_c_string_visible_bytes(&self.name),
                match change {
                    FactionPurviewChange::Grant => 0,
                    FactionPurviewChange::Revoke => 1,
                },
            );
        }

        Ok(FactionPurviewChangeOutcome::Changed {
            progress,
            log_written,
        })
    }

    /// Отклоняет либо принимает уже существующую faction-заявку.
    pub(crate) fn do_join<Context>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        manager_id: i32,
        applicant_id: i32,
        approve_flag: i32,
        join_time: TagTimeValue,
        context: &mut Context,
    ) -> Result<FactionDoJoinOutcome, FactionDoJoinBlock<Context::Block>>
    where
        Context: FactionDoJoinContext + ?Sized,
    {
        if self.has_enemy_faction() || context.already_declared_for_village_war(self.faction_id) {
            send_apply_join_information(context, manager_id, b"WS0162", b"WS0121");
            return Ok(FactionDoJoinOutcome::Rejected {
                reason: FactionDoJoinRejection::StandardOrVillageWar,
                application_removal: None,
            });
        }
        if self.has_city_war_enemy_faction()
            || context.already_declared_for_city_war(self.faction_id)
        {
            send_apply_join_information(context, manager_id, b"WS0163", b"WS0121");
            return Ok(FactionDoJoinOutcome::Rejected {
                reason: FactionDoJoinRejection::CityWar,
                application_removal: None,
            });
        }
        if context.goods_war_blocks_join(self.faction_id, manager_id) {
            send_apply_join_information(context, manager_id, b"WS0162", b"WS0121");
            return Ok(FactionDoJoinOutcome::Rejected {
                reason: FactionDoJoinRejection::GoodsWar,
                application_removal: None,
            });
        }
        if !self.is_using_purview(manager_id, EPurview::ConMem as i32) {
            return Ok(FactionDoJoinOutcome::Rejected {
                reason: FactionDoJoinRejection::PermissionDenied,
                application_removal: None,
            });
        }

        let Some(apply_person) = self.apply_persons.get(&applicant_id).copied() else {
            return Ok(FactionDoJoinOutcome::Rejected {
                reason: FactionDoJoinRejection::ApplicationNotFound,
                application_removal: None,
            });
        };
        let application_removal = self.remove_apply_member(game, applicant_id);

        if approve_flag == 0 {
            let notice = context
                .format_world_string(b"WS0167", &[legacy_c_string_visible_bytes(&self.name)]);
            let notice = legacy_c_string_visible_bytes(&notice);
            ensure_do_join_string_fits::<Context::Block>(
                FactionDoJoinStringField::DenialNotice,
                notice.len(),
                APPLY_JOIN_NOTICE_CAPACITY,
                true,
                false,
            )?;
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: applicant_id,
                first_text: notice,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionDoJoinOutcome::ApplicationDenied {
                application_removal,
            });
        }

        let level = self
            .level()
            .ok_or(FactionDoJoinBlock::MissingBaseProperty {
                application_removed: true,
            })?;
        let maximum_members = parameters.get_max_number_by_level(level);
        if self.members.len() as u32 as i32 >= maximum_members {
            let first_text = context.world_string(b"WS0168").unwrap_or_default();
            let first_text = legacy_c_string_visible_bytes(&first_text);
            ensure_do_join_string_fits::<Context::Block>(
                FactionDoJoinStringField::MemberLimitNotice,
                first_text.len(),
                APPLY_JOIN_NOTICE_CAPACITY,
                true,
                false,
            )?;
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: manager_id,
                first_text,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionDoJoinOutcome::Rejected {
                reason: FactionDoJoinRejection::MemberLimit,
                application_removal: Some(application_removal),
            });
        }

        context
            .remove_previous_faction_applications(game, applicant_id)
            .map_err(|source| FactionDoJoinBlock::Context {
                operation: FactionDoJoinContextOperation::RemovePreviousApplications,
                source,
                application_removed: true,
            })?;
        if context
            .applicant_already_in_faction(applicant_id)
            .map_err(|source| FactionDoJoinBlock::Context {
                operation: FactionDoJoinContextOperation::ApplicantMembershipLookup,
                source,
                application_removed: true,
            })?
        {
            return Ok(FactionDoJoinOutcome::Rejected {
                reason: FactionDoJoinRejection::ApplicantAlreadyInFaction,
                application_removal: Some(application_removal),
            });
        }

        let title = context.world_string(b"WS0169").unwrap_or_default();
        let title = fixed_do_join_string::<FACTION_MEMBER_TEXT_CAPACITY, Context::Block>(
            FactionDoJoinStringField::MemberTitle,
            &title,
            true,
            false,
        )?;
        let mut purview = [EPurviewOwnState::No; 11];
        purview[EPurview::Exit as usize] = EPurviewOwnState::Permit;
        purview[EPurview::LeaveWord as usize] = EPurviewOwnState::Permit;

        let (name, member_level, occupation, region) =
            if let Some(player) = game.online_player_by_id(applicant_id as u32) {
                let name = fixed_do_join_string::<FACTION_MEMBER_NAME_CAPACITY, Context::Block>(
                    FactionDoJoinStringField::OnlinePlayerName,
                    player.get_name(),
                    true,
                    false,
                )?;
                let region_source = match game.region_name(player.get_region_id()) {
                    WorldRegionNameLookup::RegionNotFound
                    | WorldRegionNameLookup::NullRegionPointer => &[][..],
                    WorldRegionNameLookup::Name(name) => name,
                };
                let region = fixed_do_join_string::<FACTION_MEMBER_TEXT_CAPACITY, Context::Block>(
                    FactionDoJoinStringField::OnlineRegionName,
                    region_source,
                    true,
                    false,
                )?;
                (
                    name,
                    i32::from(player.get_level()),
                    i32::from(player.get_occupation()),
                    region,
                )
            } else {
                let mut name = [0; FACTION_MEMBER_NAME_CAPACITY];
                name[..APPLY_PERSON_NAME_CAPACITY].copy_from_slice(&apply_person.name);
                (
                    name,
                    apply_person.level,
                    apply_person.occupation,
                    [0; FACTION_MEMBER_TEXT_CAPACITY],
                )
            };

        let member = TagMemInfo::from_complete_fields(
            applicant_id,
            name,
            member_level,
            occupation,
            99,
            title,
            purview,
            region,
            join_time,
            false,
        );
        let member_name = member
            .name_wire_bytes()
            .expect("online и offline join-name ограничены до создания member");
        let member_name = member_name[..member_name.len() - 1].to_vec();

        let joined_notice = context.format_world_string(b"WS0170", &[&member_name]);
        let joined_notice = legacy_c_string_visible_bytes(&joined_notice);
        ensure_do_join_string_fits::<Context::Block>(
            FactionDoJoinStringField::MemberJoinedNotice,
            joined_notice.len(),
            APPLY_JOIN_NOTICE_CAPACITY,
            true,
            false,
        )?;
        let joined_second_text = context.world_string(b"WS0119").unwrap_or_default();
        let member_information = self.send_info_to_all_members(
            joined_notice,
            legacy_c_string_visible_bytes(&joined_second_text),
            -1,
            |request| context.send_organizing_info(request),
        );

        let applicant_notice =
            context.format_world_string(b"WS0171", &[legacy_c_string_visible_bytes(&self.name)]);
        let applicant_notice = legacy_c_string_visible_bytes(&applicant_notice);
        ensure_do_join_string_fits::<Context::Block>(
            FactionDoJoinStringField::ApplicantJoinedNotice,
            applicant_notice.len(),
            APPLY_JOIN_NOTICE_CAPACITY,
            true,
            false,
        )?;
        let applicant_second_text = context.world_string(b"WS0119").unwrap_or_default();
        context.send_organizing_info(FactionMemberInfoRequest {
            recipient_player_id: applicant_id,
            first_text: applicant_notice,
            second_text: legacy_c_string_visible_bytes(&applicant_second_text),
            information_type: -1,
            color: 0xFFDA_EDFE,
            trailing_value: 0,
        });

        self.members.insert(applicant_id, member);
        let refreshed_player_ids =
            self.update_player_faction_info(game, applicant_id, |player_id| {
                context.update_player_faction_info(player_id);
            });
        let add_faction_to_client_result = context.add_faction_to_client_by_player_id(applicant_id);
        let add_all_faction_info_result =
            context.add_all_faction_info_to_client_by_player_id(applicant_id);
        let member_update = self.update_member_info_to_client(game, applicant_id, EOperator::Add);
        self.set_change_data(2);

        let delete_remain_time =
            self.delete_remain_time
                .ok_or(FactionDoJoinBlock::MissingDeleteRemainTime {
                    member_inserted: true,
                })?;
        let disband_countdown_cancelled = parameters.disband_faction_minimum_members() as u32
            <= self.members.len() as u32
            && delete_remain_time >= 0;
        if disband_countdown_cancelled {
            self.delete_remain_time = Some(-1);
        }

        let log_written = context.faction_join_log_enabled();
        if log_written {
            let manager =
                self.members
                    .get(&manager_id)
                    .ok_or(FactionDoJoinBlock::ManagerMemberMissing {
                        manager_id,
                        member_inserted: true,
                    })?;
            let manager_name = manager.name_wire_bytes().map_err(|_| {
                FactionDoJoinBlock::UnterminatedManagerName {
                    manager_id,
                    member_inserted: true,
                }
            })?;
            context.write_faction_join_log(
                applicant_id,
                &member_name,
                manager_id,
                &manager_name[..manager_name.len() - 1],
                self.faction_id,
                legacy_c_string_visible_bytes(&self.name),
                0,
            );
        }

        Ok(FactionDoJoinOutcome::Joined {
            application_removal,
            member_information,
            refreshed_player_ids,
            add_faction_to_client_result,
            add_all_faction_info_result,
            member_update,
            disband_countdown_cancelled,
            log_written,
        })
    }

    /// Публикует удаление по переданному ID либо последний leave-word целиком.
    pub(crate) fn update_leave_word_to_client(
        &self,
        game: &CGame,
        leave_word_id: i32,
        operator: EOperator,
    ) -> Result<Vec<FactionLeaveWordDelivery>, FactionLeaveWordUpdateBuildError> {
        let leave_word = if operator == EOperator::Delete {
            None
        } else {
            self.leave_words.back()
        };

        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(LEAVE_WORD_UPDATE_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            if operator != EOperator::Delete {
                let Some(leave_word) = leave_word else {
                    return Err(FactionLeaveWordUpdateBuildError::MissingLastLeaveWord);
                };
                message.base_mut().add_long(operator.wire_value());
                message.base_mut().add_long(leave_word.id);
                message.base_mut().add_long(leave_word.player_id);
                let name = match leave_word.name_wire_bytes() {
                    Ok(name) => name,
                    Err(source) => {
                        return Err(FactionLeaveWordUpdateBuildError::Recipient {
                            source,
                            recipient_player_id,
                            game_server_id,
                            completed_deliveries: deliveries,
                        });
                    }
                };
                message.base_mut().add(name);
                let content = match leave_word.content_wire_bytes() {
                    Ok(content) => content,
                    Err(source) => {
                        return Err(FactionLeaveWordUpdateBuildError::Recipient {
                            source,
                            recipient_player_id,
                            game_server_id,
                            completed_deliveries: deliveries,
                        });
                    }
                };
                message.base_mut().add(content);
                message.base_mut().add(&leave_word.time.wire_bytes());
            } else {
                message.base_mut().add_long(EOperator::Delete.wire_value());
                message.base_mut().add_long(leave_word_id);
            }
            deliveries.push(FactionLeaveWordDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    /// Удаляет leave-word; объявленный operator исходная функция не читала.
    pub(crate) fn edit_leave_word(
        &mut self,
        game: &CGame,
        player_id: i32,
        leave_word_id: i32,
        _operator: EOperator,
    ) -> Result<FactionEditLeaveWordOutcome, FactionInitialPropertyBlock> {
        let property = self.base_property.ok_or(FactionInitialPropertyBlock)?;
        if !property.leave_word_function() {
            return Ok(FactionEditLeaveWordOutcome::FunctionDisabled);
        }
        if !self.is_using_purview(player_id, EPurview::EditLeaveWord as i32) {
            return Ok(FactionEditLeaveWordOutcome::PermissionDenied);
        }
        let Some(position) = self
            .leave_words
            .iter()
            .position(|leave_word| leave_word.id == leave_word_id)
        else {
            return Ok(FactionEditLeaveWordOutcome::LeaveWordNotFound);
        };

        self.leave_words.remove(position);
        let deliveries =
            self.update_leave_word_to_client(game, leave_word_id, EOperator::Delete);
        self.set_change_data(4);
        Ok(FactionEditLeaveWordOutcome::Deleted { deliveries })
    }

    /// Создаёт новый leave-word, вытесняет старейшие и публикует последний.
    pub(crate) fn leave_word(
        &mut self,
        game: &mut CGame,
        player_id: i32,
        content: &mut Vec<u8>,
        time: TagTimeValue,
    ) -> Result<FactionLeaveWordOutcome, FactionLeaveWordBlock> {
        let property = self
            .base_property
            .ok_or(FactionLeaveWordBlock::MissingBaseProperty)?;
        if !property.leave_word_function() {
            return Ok(FactionLeaveWordOutcome::FunctionDisabled);
        }
        if !self.is_using_purview(player_id, EPurview::LeaveWord as i32) {
            return Ok(FactionLeaveWordOutcome::PermissionDenied);
        }

        let input_truncated = content.len() > LEAVE_WORD_CONTENT_LIMIT;
        content.truncate(LEAVE_WORD_CONTENT_LIMIT);
        let visible_content = legacy_c_string_visible_bytes(content);
        let leave_word_id = game
            .allocate_leave_word_id()
            .map_err(FactionLeaveWordBlock::LeaveWordId)?;

        let Some(player) = game.online_player_by_id(player_id as u32) else {
            return Err(FactionLeaveWordBlock::OfflineAuthorNameUnknown {
                player_id,
                allocated_leave_word_id: leave_word_id,
                input_truncated,
            });
        };
        let visible_name = legacy_c_string_visible_bytes(player.get_name());
        if visible_name.len() >= LEAVE_WORD_NAME_CAPACITY {
            return Err(FactionLeaveWordBlock::PlayerNameWouldOverflow {
                player_id,
                allocated_leave_word_id: leave_word_id,
                visible_len: visible_name.len(),
                input_truncated,
            });
        }

        let mut name = [0; LEAVE_WORD_NAME_CAPACITY];
        name[..visible_name.len()].copy_from_slice(visible_name);
        let mut stored_content = [0; LEAVE_WORD_CONTENT_CAPACITY];
        stored_content[..visible_content.len()].copy_from_slice(visible_content);
        let leave_word = TagLeaveWord::from_complete_fields(
            leave_word_id,
            player_id,
            name,
            time,
            stored_content,
        );

        let mut evicted_count = 0;
        while self.leave_words.len() >= LEAVE_WORD_LIMIT {
            let _ = self.leave_words.pop_front();
            evicted_count += 1;
        }
        self.leave_words.push_back(leave_word);
        let deliveries =
            self.update_leave_word_to_client(game, leave_word_id, EOperator::Add);
        self.set_change_data(4);
        Ok(FactionLeaveWordOutcome::Published {
            leave_word_id,
            input_truncated,
            evicted_count,
            deliveries,
        })
    }

    /// Загружает один DB leave-word с восстановленным FIFO-limit.
    pub(crate) fn load_leave_word(
        &mut self,
        leave_word: TagLeaveWord,
    ) -> FactionLoadLeaveWordReport {
        let mut evicted_count = 0;
        while self.leave_words.len() >= LEAVE_WORD_LIMIT {
            let _ = self.leave_words.pop_front();
            evicted_count += 1;
        }
        self.leave_words.push_back(leave_word);
        FactionLoadLeaveWordReport { evicted_count }
    }

    /// Заменяет текущее объявление с исходными проверками и порядком эффектов.
    pub(crate) fn pronounce(
        &mut self,
        game: &CGame,
        player_id: i32,
        content: &mut Vec<u8>,
        time: TagTimeValue,
    ) -> Result<FactionPronounceOutcome, FactionPronounceBlock> {
        let property = self
            .base_property
            .ok_or(FactionPronounceBlock::MissingBaseProperty)?;
        if !property.pronounce_function() {
            return Ok(FactionPronounceOutcome::FunctionDisabled);
        }
        if !self.is_using_purview(player_id, EPurview::Pronounce as i32) {
            return Ok(FactionPronounceOutcome::PermissionDenied);
        }

        let input_truncated = content.len() > PRONOUNCE_CONTENT_CAPACITY;
        content.truncate(PRONOUNCE_CONTENT_CAPACITY);
        let visible_content = legacy_c_string_visible_bytes(content);
        if visible_content.len() >= PRONOUNCE_CONTENT_CAPACITY {
            return Err(FactionPronounceBlock::ContentWouldOverflow {
                visible_len: visible_content.len(),
                input_truncated,
            });
        }

        let online_player = game.online_player_by_id(player_id as u32);
        let visible_name =
            online_player.map(|player| legacy_c_string_visible_bytes(player.get_name()));
        if let Some(visible_name) = visible_name {
            if visible_name.len() >= PRONOUNCE_NAME_CAPACITY {
                return Err(FactionPronounceBlock::PlayerNameWouldOverflow {
                    player_id,
                    visible_len: visible_name.len(),
                    input_truncated,
                });
            }
        }

        self.pronounce.player_id = player_id;
        self.pronounce.time = time;
        self.pronounce.content[..visible_content.len()].copy_from_slice(visible_content);
        self.pronounce.content[visible_content.len()] = 0;
        if let Some(visible_name) = visible_name {
            self.pronounce.name[..visible_name.len()].copy_from_slice(visible_name);
            self.pronounce.name[visible_name.len()] = 0;
        }

        let deliveries = self.update_pronounce_to_client(game, EOperator::Update);
        self.set_change_data(8);
        Ok(FactionPronounceOutcome::Published {
            input_truncated,
            deliveries,
        })
    }

    /// Повторяет permission/interval gate старого `UploadIcon` без выдуманного I/O.
    pub(crate) fn upload_icon<Context>(
        &mut self,
        parameters: &COrganizingParam,
        player_id: i32,
        _time: &TagTimeValue,
        context: &mut Context,
    ) -> Result<FactionUploadIconOutcome, FactionUploadIconBlock>
    where
        Context: FactionUploadIconContext,
    {
        if self.is_master(player_id) == 0 {
            return Ok(FactionUploadIconOutcome::Rejected {
                reason: FactionUploadIconRejection::PlayerNotMaster,
                notice_sent: false,
            });
        }
        let property = self
            .base_property
            .ok_or(FactionUploadIconBlock::MissingBaseProperty)?;
        if !property.upload_icon_function() {
            send_apply_join_information(context, player_id, b"WS0225", b"WS0119");
            return Ok(FactionUploadIconOutcome::Rejected {
                reason: FactionUploadIconRejection::FunctionDisabled,
                notice_sent: true,
            });
        }

        let interval_minutes = parameters.upload_icon_interval_minutes();
        if interval_minutes < 1 {
            self.set_change_data(8);
            return Ok(FactionUploadIconOutcome::Accepted { dirty_set: true });
        }

        let notice = context.format_upload_icon_interval(b"WS0226", interval_minutes);
        let notice = legacy_c_string_visible_bytes(&notice);
        if notice.len() >= FACTION_UPLOAD_ICON_NOTICE_CAPACITY {
            return Err(FactionUploadIconBlock::NoticeWouldOverflow {
                formatted_len: notice.len(),
            });
        }
        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        context.send_organizing_info(FactionMemberInfoRequest {
            recipient_player_id: player_id,
            first_text: notice,
            second_text: legacy_c_string_visible_bytes(&second_text),
            information_type: -1,
            color: 0xFFDA_EDFE,
            trailing_value: 0,
        });
        Ok(FactionUploadIconOutcome::Rejected {
            reason: FactionUploadIconRejection::IntervalActive,
            notice_sent: true,
        })
    }

    /// Проверяет disband-контракт и очищает reached transient faction-state.
    pub(crate) fn disband<Context>(
        &mut self,
        game: &CGame,
        player_id: i32,
        context: &mut Context,
    ) -> Result<FactionDisbandOutcome, FactionDisbandBlock>
    where
        Context: FactionDisbandContext,
    {
        if !self.check_operator_validate(player_id, EPurview::Disband as i32) {
            return Ok(FactionDisbandOutcome::Rejected {
                reason: FactionDisbandRejection::OperatorNotPermitted,
                notice_sent: false,
            });
        }
        let superior_organizing = self
            .superior_organizing()
            .ok_or(FactionDisbandBlock::MissingBaseProperty)?;
        if superior_organizing > 0 {
            return Ok(FactionDisbandOutcome::Rejected {
                reason: FactionDisbandRejection::HasSuperiorOrganizing,
                notice_sent: false,
            });
        }
        if self.has_enemy_faction() || context.village_war_declared(self.faction_id) {
            send_apply_join_information(context, player_id, b"WS0189", b"WS0121");
            return Ok(FactionDisbandOutcome::Rejected {
                reason: FactionDisbandRejection::StandardWar,
                notice_sent: true,
            });
        }
        if self.has_city_war_enemy_faction() || context.city_war_declared(self.faction_id) {
            send_apply_join_information(context, player_id, b"WS0190", b"WS0121");
            return Ok(FactionDisbandOutcome::Rejected {
                reason: FactionDisbandRejection::CityWar,
                notice_sent: true,
            });
        }
        if context.goods_war_blocks_disband(self.faction_id, player_id) {
            send_apply_join_information(context, player_id, b"ws0362", b"WS0121");
            return Ok(FactionDisbandOutcome::Rejected {
                reason: FactionDisbandRejection::GoodsWar,
                notice_sent: true,
            });
        }

        let country = self
            .country()
            .ok_or(FactionDisbandBlock::MissingBaseProperty)?;
        let king_id = context
            .country_king_id(country)
            .ok_or(FactionDisbandBlock::CountryMissing { country })?;
        if king_id == player_id {
            return Ok(FactionDisbandOutcome::Rejected {
                reason: FactionDisbandRejection::CountryKing,
                notice_sent: false,
            });
        }

        let mut progress = FactionDisbandProgress {
            goods_war_members_deleted: false,
            goods_war_faction_count_decremented: false,
            cleared_apply_persons: 0,
            cleared_leave_words: 0,
            delete_organizing: None,
        };
        context.delete_goods_war_members_by_faction_id(self.faction_id);
        progress.goods_war_members_deleted = true;
        context.decrement_goods_war_faction_count(self.faction_id);
        progress.goods_war_faction_count_decremented = true;

        progress.cleared_apply_persons = self.apply_persons.len();
        self.apply_persons.clear();
        progress.cleared_leave_words = self.leave_words.len();
        self.leave_words.clear();
        progress.delete_organizing =
            Some(match self.delete_organizing_to_client(game, 0, context) {
                Ok(outcome) => outcome,
                Err(source) => {
                    return Err(FactionDisbandBlock::DeleteOrganizing { source, progress });
                }
            });
        Ok(FactionDisbandOutcome::Disbanded(progress))
    }

    /// Передаёт leadership с точными gates, member-state и порядком публикации.
    pub(crate) fn demise<Context>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        old_master_id: i32,
        new_master_id: i32,
        context: &mut Context,
    ) -> Result<FactionDemiseOutcome, FactionDemiseBlock>
    where
        Context: FactionDemiseContext,
    {
        if old_master_id == new_master_id {
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::SamePlayer,
                notice_sent: false,
            });
        }
        if self.is_master(old_master_id) == 0 {
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::OldPlayerNotMaster,
                notice_sent: false,
            });
        }
        if self.is_member(new_master_id) == 0 {
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::NewPlayerNotMember,
                notice_sent: false,
            });
        }
        if self.has_enemy_faction() || context.attack_city_system_declared(self.faction_id) {
            send_apply_join_information(context, old_master_id, b"WS0213", b"WS0119");
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::StandardWar,
                notice_sent: true,
            });
        }
        if self.has_city_war_enemy_faction()
            || context.attack_city_system_declared(self.faction_id)
        {
            send_apply_join_information(context, old_master_id, b"WS0214", b"WS0119");
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::CityWar,
                notice_sent: true,
            });
        }
        if context.goods_war_blocks_demise(self.faction_id, old_master_id) {
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: old_master_id,
                first_text: b"???",
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::GoodsWar,
                notice_sent: true,
            });
        }

        let old_player = game.online_player_by_id(old_master_id as u32);
        let new_player = game.online_player_by_id(new_master_id as u32);
        let Some(old_player) = old_player else {
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::OldPlayerOffline,
                notice_sent: false,
            });
        };
        let Some(new_player) = new_player else {
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::NewPlayerOffline,
                notice_sent: false,
            });
        };
        if new_player.faction_war_operator() {
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::NewPlayerOperatingFactionWar,
                notice_sent: false,
            });
        }
        if old_player.faction_war_operator() {
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::OldPlayerOperatingFactionWar,
                notice_sent: false,
            });
        }

        let required_level = parameters.create_faction_player_level();
        if i32::from(new_player.get_level()) < required_level {
            let notice = context.format_demise_signed(b"WS0215", required_level);
            let notice = bounded_demise_notice(&notice, FACTION_DEMISE_BLOCK_NOTICE_CAPACITY)
                .map_err(|formatted_len| FactionDemiseBlock::NoticeWouldOverflow {
                    notice: FactionDemiseNotice::RequiredLevel,
                    formatted_len,
                    progress: empty_faction_demise_progress(),
                })?;
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: old_master_id,
                first_text: &notice,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::NewMasterLevelTooLow,
                notice_sent: true,
            });
        }

        let permit_demise = self
            .permit_demise
            .ok_or(FactionDemiseBlock::PermitDemiseUnknown)?;
        if !permit_demise {
            let notice = context.world_string(b"WS0216").unwrap_or_default();
            let notice = bounded_demise_notice(&notice, FACTION_DEMISE_BLOCK_NOTICE_CAPACITY)
                .map_err(|formatted_len| FactionDemiseBlock::NoticeWouldOverflow {
                    notice: FactionDemiseNotice::DemiseForbidden,
                    formatted_len,
                    progress: empty_faction_demise_progress(),
                })?;
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: old_master_id,
                first_text: &notice,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::DemiseForbidden,
                notice_sent: true,
            });
        }

        let country = self
            .country()
            .ok_or(FactionDemiseBlock::MissingBaseProperty)?;
        if context.country_blocks_demise(country, old_master_id) {
            let notice = context.format_demise_signed(b"WS0217", required_level);
            let notice = bounded_demise_notice(&notice, FACTION_DEMISE_BLOCK_NOTICE_CAPACITY)
                .map_err(|formatted_len| FactionDemiseBlock::NoticeWouldOverflow {
                    notice: FactionDemiseNotice::CountryKingBlocked,
                    formatted_len,
                    progress: empty_faction_demise_progress(),
                })?;
            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id: old_master_id,
                first_text: &notice,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: -1,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            return Ok(FactionDemiseOutcome::Rejected {
                reason: FactionDemiseRejection::CountryKingBlocked,
                notice_sent: true,
            });
        }

        if !self.members.contains_key(&old_master_id) {
            return Err(FactionDemiseBlock::OldMasterMemberMissing {
                player_id: old_master_id,
            });
        }

        let mut progress = empty_faction_demise_progress();
        self.permit_demise = Some(false);
        progress.permit_demise_disabled = true;
        self.master_id = Some(new_master_id);
        progress.master_changed = true;

        let new_title_source = context.world_string(b"WS0157").unwrap_or_default();
        let new_title = match fixed_demise_title(&new_title_source) {
            Ok(title) => title,
            Err(visible_len) => {
                return Err(FactionDemiseBlock::TitleWouldOverflow {
                    member: FactionDemiseMember::NewMaster,
                    string_id: b"WS0157",
                    visible_len,
                    progress,
                });
            }
        };
        let new_member = self
            .members
            .get_mut(&new_master_id)
            .expect("IsMember(new) и Demise не удаляют member");
        new_member.title = new_title;
        new_member.job_level = 1;
        new_member.id = new_master_id;
        new_member.purview = demise_new_master_purview();
        progress.new_member_changed = true;

        let old_title_source = context.world_string(b"WS0218").unwrap_or_default();
        let old_title = match fixed_demise_title(&old_title_source) {
            Ok(title) => title,
            Err(visible_len) => {
                return Err(FactionDemiseBlock::TitleWouldOverflow {
                    member: FactionDemiseMember::OldMaster,
                    string_id: b"WS0218",
                    visible_len,
                    progress,
                });
            }
        };
        let old_member = self
            .members
            .get_mut(&old_master_id)
            .expect("old-master invariant проверен до leadership mutation");
        old_member.id = old_master_id;
        old_member.job_level = 99;
        old_member.title = old_title;
        old_member.purview = demise_old_master_purview();
        progress.old_member_changed = true;

        progress.old_member_update = Some(match self.update_member_info_to_client(
            game,
            old_master_id,
            EOperator::Update,
        ) {
            Ok(report) => report,
            Err(source) => {
                return Err(FactionDemiseBlock::MemberUpdate {
                    member: FactionDemiseMember::OldMaster,
                    source,
                    progress,
                });
            }
        });
        progress.new_member_update = Some(match self.update_member_info_to_client(
            game,
            new_master_id,
            EOperator::Update,
        ) {
            Ok(report) => report,
            Err(source) => {
                return Err(FactionDemiseBlock::MemberUpdate {
                    member: FactionDemiseMember::NewMaster,
                    source,
                    progress,
                });
            }
        });
        self.set_change_data(1);
        progress.base_dirty_set = true;
        self.set_change_data(2);
        progress.members_dirty_set = true;
        progress.refreshed_player_ids = self.update_player_faction_info(game, 0, |player_id| {
            context.update_player_faction_info(player_id);
        });

        let old_master_name = match self
            .members
            .get(&old_master_id)
            .expect("old-master member сохранён")
            .name_wire_bytes()
        {
            Ok(name) => name[..name.len() - 1].to_vec(),
            Err(source) => {
                return Err(FactionDemiseBlock::UnterminatedMemberName {
                    member: FactionDemiseMember::OldMaster,
                    player_id: old_master_id,
                    source,
                    progress,
                });
            }
        };
        let new_master_name = match self
            .members
            .get(&new_master_id)
            .expect("new-master member сохранён")
            .name_wire_bytes()
        {
            Ok(name) => name[..name.len() - 1].to_vec(),
            Err(source) => {
                return Err(FactionDemiseBlock::UnterminatedMemberName {
                    member: FactionDemiseMember::NewMaster,
                    player_id: new_master_id,
                    source,
                    progress,
                });
            }
        };
        let notice = context.format_demise_change(
            b"WS0219",
            &old_master_name,
            &new_master_name,
        );
        let notice = match bounded_demise_notice(&notice, FACTION_DEMISE_NOTICE_CAPACITY) {
            Ok(notice) => notice,
            Err(formatted_len) => {
                return Err(FactionDemiseBlock::NoticeWouldOverflow {
                    notice: FactionDemiseNotice::Success,
                    formatted_len,
                    progress,
                });
            }
        };
        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        progress.member_information = Some(self.send_info_to_all_members_with_color(
            &notice,
            legacy_c_string_visible_bytes(&second_text),
            -1,
            0x0087_A238,
            |request| context.send_organizing_info(request),
        ));

        if context.faction_master_log_enabled() {
            context.write_faction_master_log(
                old_master_id,
                &old_master_name,
                new_master_id,
                &new_master_name,
                self.faction_id,
                legacy_c_string_visible_bytes(&self.name),
            );
            progress.log_written = true;
        }

        Ok(FactionDemiseOutcome::Transferred(progress))
    }

    /// Проверяет authority на сбор налога в принадлежащем faction городе.
    pub(crate) fn operator_tax<Context>(
        &self,
        player_id: i32,
        region_id: i32,
        context: &Context,
    ) -> Result<FactionOperationOutcome, FactionOperationBlock<Context::Block>>
    where
        Context: FactionOperationAuthorityContext,
    {
        let union_id = context
            .union_id_for_faction(self.faction_id)
            .map_err(FactionOperationBlock::UnionMembershipLookup)?;
        if union_id > 0
            && let Some(master_faction_id) = context.union_master_faction_id(union_id)
            && master_faction_id != self.faction_id
        {
            return Ok(FactionOperationOutcome::Rejected(
                FactionOperationRejection::UnionMasterMismatch {
                    union_id,
                    master_faction_id,
                },
            ));
        }
        self.authorize_owned_city_operation(player_id, region_id, EPurview::ObtainTax)
    }

    /// Проверяет authority на управление воротами принадлежащего faction города.
    pub(crate) fn operator_city_gate<Context>(
        &self,
        player_id: i32,
        region_id: i32,
        context: &Context,
    ) -> Result<FactionOperationOutcome, FactionOperationBlock<Context::Block>>
    where
        Context: FactionOperationAuthorityContext,
    {
        let union_id = self
            .superior_organizing()
            .ok_or(FactionOperationBlock::MissingBaseProperty)?;
        if union_id > 0
            && let Some(master_faction_id) = context.union_master_faction_id(union_id)
            && master_faction_id != self.faction_id
        {
            return Ok(FactionOperationOutcome::Rejected(
                FactionOperationRejection::UnionMasterMismatch {
                    union_id,
                    master_faction_id,
                },
            ));
        }
        self.authorize_owned_city_operation(player_id, region_id, EPurview::OperCityGate)
    }

    fn authorize_owned_city_operation<ContextBlock>(
        &self,
        player_id: i32,
        region_id: i32,
        purview: EPurview,
    ) -> Result<FactionOperationOutcome, FactionOperationBlock<ContextBlock>> {
        if self.is_owned_city(region_id) == 0 {
            return Ok(FactionOperationOutcome::Rejected(
                FactionOperationRejection::RegionNotOwned { region_id },
            ));
        }
        if !self.check_operator_validate(player_id, purview as i32) {
            return Ok(FactionOperationOutcome::Rejected(
                FactionOperationRejection::PlayerNotPermitted { player_id, purview },
            ));
        }
        Ok(FactionOperationOutcome::Authorized)
    }

    /// Передаёт organizing-info каждому member без online-фильтра этого owner-а.
    pub(crate) fn send_info_to_all_members<'a, F>(
        &self,
        first_text: &'a [u8],
        second_text: &'a [u8],
        information_type: i32,
        send_organizing_info: F,
    ) -> FactionMemberInfoReport
    where
        F: FnMut(FactionMemberInfoRequest<'a>),
    {
        self.send_info_to_all_members_with_color(
            first_text,
            second_text,
            information_type,
            0xFFDA_EDFE,
            send_organizing_info,
        )
    }

    fn send_info_to_all_members_with_color<'a, F>(
        &self,
        first_text: &'a [u8],
        second_text: &'a [u8],
        information_type: i32,
        color: u32,
        mut send_organizing_info: F,
    ) -> FactionMemberInfoReport
    where
        F: FnMut(FactionMemberInfoRequest<'a>),
    {
        let mut recipient_player_ids = Vec::with_capacity(self.members.len());
        for &recipient_player_id in self.members.keys() {
            send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id,
                first_text,
                second_text,
                information_type,
                color,
                trailing_value: 0,
            });
            recipient_player_ids.push(recipient_player_id);
        }
        FactionMemberInfoReport {
            recipient_player_ids,
        }
    }

    pub(crate) fn set_leave_word_function<Context>(
        &mut self,
        enabled: bool,
        context: &mut Context,
    ) -> Result<FactionFeatureFunctionUpdate, FactionInitialPropertyBlock>
    where
        Context: FactionOrganizingInfoContext,
    {
        self.set_feature_function(FactionFeatureFunction::LeaveWord, enabled, context)
    }

    pub(crate) fn set_pronounce_function<Context>(
        &mut self,
        enabled: bool,
        context: &mut Context,
    ) -> Result<FactionFeatureFunctionUpdate, FactionInitialPropertyBlock>
    where
        Context: FactionOrganizingInfoContext,
    {
        self.set_feature_function(FactionFeatureFunction::Pronounce, enabled, context)
    }

    pub(crate) fn set_endue_right_function<Context>(
        &mut self,
        enabled: bool,
        context: &mut Context,
    ) -> Result<FactionFeatureFunctionUpdate, FactionInitialPropertyBlock>
    where
        Context: FactionOrganizingInfoContext,
    {
        self.set_feature_function(FactionFeatureFunction::EndueRight, enabled, context)
    }

    pub(crate) fn set_join_village_war_function<Context>(
        &mut self,
        enabled: bool,
        context: &mut Context,
    ) -> Result<FactionFeatureFunctionUpdate, FactionInitialPropertyBlock>
    where
        Context: FactionOrganizingInfoContext,
    {
        self.set_feature_function(FactionFeatureFunction::JoinVillageWar, enabled, context)
    }

    pub(crate) fn set_join_city_war_function<Context>(
        &mut self,
        enabled: bool,
        context: &mut Context,
    ) -> Result<FactionFeatureFunctionUpdate, FactionInitialPropertyBlock>
    where
        Context: FactionOrganizingInfoContext,
    {
        self.set_feature_function(FactionFeatureFunction::JoinCityWar, enabled, context)
    }

    pub(crate) fn set_create_union_function<Context>(
        &mut self,
        enabled: bool,
        context: &mut Context,
    ) -> Result<FactionFeatureFunctionUpdate, FactionInitialPropertyBlock>
    where
        Context: FactionOrganizingInfoContext,
    {
        self.set_feature_function(FactionFeatureFunction::CreateUnion, enabled, context)
    }

    fn set_feature_function<Context>(
        &mut self,
        feature: FactionFeatureFunction,
        enabled: bool,
        context: &mut Context,
    ) -> Result<FactionFeatureFunctionUpdate, FactionInitialPropertyBlock>
    where
        Context: FactionOrganizingInfoContext,
    {
        let property = self.base_property.ok_or(FactionInitialPropertyBlock)?;
        if property.feature_function(feature) == enabled {
            return Ok(FactionFeatureFunctionUpdate::Unchanged);
        }
        self.base_property
            .as_mut()
            .ok_or(FactionInitialPropertyBlock)?
            .set_feature_function(feature, enabled);

        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        let first_text = context
            .world_string(feature.notification_string_id(enabled))
            .unwrap_or_default();
        let report = self.send_info_to_all_members(
            legacy_c_string_visible_bytes(&first_text),
            legacy_c_string_visible_bytes(&second_text),
            -1,
            |request| context.send_organizing_info(request),
        );
        Ok(FactionFeatureFunctionUpdate::Updated(report))
    }

    /// Публикует имя другой фракции всем готовым member recipient-ам.
    pub(crate) fn update_other_faction_info_to_client(
        &self,
        game: &CGame,
        other_faction_id: i32,
        other_faction_name: &[u8],
        operator: EOperator,
    ) -> Result<Vec<FactionOtherInfoDelivery>, FactionOtherInfoBuildError> {
        let name_wire = legacy_c_string_wire_bytes(other_faction_name);
        let visible_name_len = name_wire.len() - 1;
        if visible_name_len >= OTHER_FACTION_NAME_CAPACITY {
            return Err(FactionOtherInfoBuildError {
                visible_name_len,
            });
        }

        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(OTHER_FACTION_UPDATE_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(operator.wire_value());
            message.base_mut().add_long(other_faction_id);
            message.base_mut().add(&name_wire);
            deliveries.push(FactionOtherInfoDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        Ok(deliveries)
    }

    /// Рассылает faction-talk всем member-ам с известным GameServer ID.
    pub(crate) fn talk(
        &self,
        game: &CGame,
        speaker_id: i32,
        first_text: &[u8],
        second_text: &[u8],
    ) -> Vec<FactionTalkDelivery> {
        let first_text = legacy_c_string_wire_bytes(first_text);
        let second_text = legacy_c_string_wire_bytes(second_text);
        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(FACTION_TALK_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(FACTION_TALK_CHANNEL);
            message.base_mut().add_long(speaker_id);
            message.base_mut().add(&first_text);
            message.base_mut().add(&second_text);
            deliveries.push(FactionTalkDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }
        deliveries
    }

    /// Удаляет faction-state у одного готового клиента либо у всех online member-ов.
    pub(crate) fn delete_organizing_to_client<Context>(
        &self,
        game: &CGame,
        target_player_id: i32,
        context: &mut Context,
    ) -> Result<FactionDeleteOrganizingOutcome, FactionDeleteOrganizingBuildError>
    where
        Context: FactionOrganizingInfoContext,
    {
        if target_player_id > 0 {
            let player = game.online_player_by_id(target_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(target_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                return Ok(FactionDeleteOrganizingOutcome::SingleTarget { delivery: None });
            }

            let mut message = CMessage::new(DELETE_ORGANIZING_MESSAGE_TYPE);
            message.base_mut().add_long(target_player_id);
            message.base_mut().add_long(self.faction_id);
            return Ok(FactionDeleteOrganizingOutcome::SingleTarget {
                delivery: Some(FactionDeleteOrganizingDelivery {
                    recipient_player_id: target_player_id,
                    game_server_id,
                    result: game.send_msg_to_game_server(game_server_id, &message),
                }),
            });
        }

        let first_text = context.world_string(b"WS0191").unwrap_or_default();
        let first_text = legacy_c_string_visible_bytes(&first_text);
        if first_text.len() >= DELETE_ORGANIZING_INFO_CAPACITY {
            return Err(FactionDeleteOrganizingBuildError {
                localized_info_len: first_text.len(),
            });
        }
        let first_text = first_text.to_vec();

        let mut deliveries = Vec::new();
        let mut information_recipient_ids = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none() || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(DELETE_ORGANIZING_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(self.faction_id);
            deliveries.push(FactionDeleteOrganizingDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });

            let second_text = context.world_string(b"WS0119").unwrap_or_default();
            context.send_organizing_info(FactionMemberInfoRequest {
                recipient_player_id,
                first_text: &first_text,
                second_text: legacy_c_string_visible_bytes(&second_text),
                information_type: game_server_id,
                color: 0xFFDA_EDFE,
                trailing_value: 0,
            });
            information_recipient_ids.push(recipient_player_id);
        }

        Ok(FactionDeleteOrganizingOutcome::Broadcast {
            deliveries,
            information_recipient_ids,
        })
    }

    /// Clamp-ит и публикует faction experience с исходным порядком эффектов.
    pub(crate) fn set_experience(
        &mut self,
        game: &CGame,
        experience: i32,
    ) -> Result<FactionExperienceUpdate, FactionExperienceBlock> {
        let property = self
            .base_property
            .as_mut()
            .ok_or(FactionExperienceBlock::MissingBaseProperty)?;
        if property.level() >= 12 {
            return Ok(FactionExperienceUpdate::MaximumLevel);
        }
        let experience = experience.clamp(0, 100_000_000);
        if property.experience() == experience {
            return Ok(FactionExperienceUpdate::Unchanged { experience });
        }
        property.write_signed(0x04, experience);
        self.set_change_data(1);
        let deliveries = self.update_experience_to_client(game)?;
        Ok(FactionExperienceUpdate::Updated {
            experience,
            deliveries,
        })
    }

    /// Возвращает достигнутый `m_Property.lConfederationID`.
    pub(crate) const fn superior_organizing(&self) -> Option<i32> {
        match self.base_property {
            Some(property) => Some(property.union_id()),
            None => None,
        }
    }

    /// Возвращает player-header с исходным union lookup и faction fallback.
    pub(crate) fn player_header<Context>(
        &self,
        context: &Context,
    ) -> Result<i32, FactionPlayerHeaderBlock<Context::Block>>
    where
        Context: FactionPlayerHeaderContext,
    {
        let property = self
            .base_property
            .ok_or(FactionPlayerHeaderBlock::MissingBaseProperty)?;
        if property.union_id() > 0 {
            if let Some(player_header) = context
                .union_player_header(property.union_id())
                .map_err(FactionPlayerHeaderBlock::Context)?
            {
                return Ok(player_header);
            }
        }
        self.master_id
            .ok_or(FactionPlayerHeaderBlock::MissingMasterId)
    }

    /// Назначает union ID и поддерживает исходный countdown роспуска.
    pub(crate) fn set_superior_organizing(
        &mut self,
        organizing_id: i32,
        parameters: &COrganizingParam,
    ) -> Result<(), FactionSuperiorOrganizingBlock> {
        let property = self
            .base_property
            .as_mut()
            .ok_or(FactionSuperiorOrganizingBlock::MissingBaseProperty)?;
        if property.union_id() != organizing_id {
            property.write_signed(0x18, organizing_id);
        }

        if organizing_id < 1 {
            let member_count = self.members.len() as u32;
            let minimum_members = parameters.disband_faction_minimum_members() as u32;
            if member_count < minimum_members {
                let delete_remain_time = self
                    .delete_remain_time
                    .ok_or(FactionSuperiorOrganizingBlock::DeleteRemainTimeAbsent)?;
                if delete_remain_time < 0 {
                    self.delete_remain_time = Some(parameters.disband_faction_minutes());
                }
            }
        } else {
            let delete_remain_time = self
                .delete_remain_time
                .ok_or(FactionSuperiorOrganizingBlock::DeleteRemainTimeAbsent)?;
            if delete_remain_time > 0 {
                self.delete_remain_time = Some(-1);
            }
        }
        Ok(())
    }

    /// Удаляет не-master участника и при необходимости запускает роспуск.
    pub(crate) fn del_member(
        &mut self,
        player_id: i32,
        parameters: &COrganizingParam,
    ) -> Result<Option<FactionDelMemberReport>, FactionDelMemberBlock> {
        let master_id = self.master_id.ok_or(FactionDelMemberBlock::MasterIdMissing)?;
        if player_id == master_id {
            return Ok(None);
        }

        let removed = self.members.remove(&player_id).is_some();
        let member_count = self.members.len() as u32;
        let minimum_members = parameters.disband_faction_minimum_members() as u32;
        let mut disband_countdown_started = false;
        if member_count < minimum_members {
            let delete_remain_time = self
                .delete_remain_time
                .ok_or(FactionDelMemberBlock::DeleteRemainTimeAbsent)?;
            if delete_remain_time < 0 {
                let union_id = self
                    .base_property
                    .ok_or(FactionDelMemberBlock::MissingBaseProperty)?
                    .union_id();
                if union_id < 1 {
                    self.delete_remain_time = Some(parameters.disband_faction_minutes());
                    disband_countdown_started = true;
                }
            }
        }

        Ok(Some(FactionDelMemberReport {
            removed,
            disband_countdown_started,
        }))
    }

    /// Возвращает member-title без завершающего NUL либо старую overread-границу.
    pub(crate) fn member_title(&self, player_id: i32) -> Result<Vec<u8>, UnterminatedMemberField> {
        let Some(member) = self.members.get(&player_id) else {
            return Ok(Vec::new());
        };
        let wire = member.title_wire_bytes()?;
        Ok(wire[..wire.len() - 1].to_vec())
    }

    /// Сохраняет точный missing-member результат `CFaction::IsControbute`.
    pub(crate) fn is_contribute(&self, player_id: i32) -> bool {
        self.members
            .get(&player_id)
            .is_some_and(|member| member.contribute)
    }

    /// Считает contributor-флаги с исходным 32-битным переполнением.
    pub(crate) fn contributor_count(&self) -> i32 {
        self.members.values().fold(0i32, |count, member| {
            if member.contribute {
                count.wrapping_add(1)
            } else {
                count
            }
        })
    }

    /// Меняет contributor-флаг с исходным порядком публикации и dirty-state.
    pub(crate) fn set_contributor<Context>(
        &mut self,
        game: &CGame,
        parameters: &COrganizingParam,
        requester_id: i32,
        target_id: i32,
        enabled: bool,
        context: &mut Context,
    ) -> Result<FactionContributorOutcome, FactionContributorBlock>
    where
        Context: FactionContributorContext,
    {
        if self.is_master(requester_id) == 0 {
            return Ok(FactionContributorOutcome::Rejected(
                FactionContributorRejection::RequesterNotMaster,
            ));
        }
        if enabled && parameters.maximum_contributors() <= self.contributor_count() {
            return Ok(FactionContributorOutcome::Rejected(
                FactionContributorRejection::MaximumContributors,
            ));
        }
        let Some(target) = self.members.get_mut(&target_id) else {
            return Ok(FactionContributorOutcome::Rejected(
                FactionContributorRejection::TargetNotMember,
            ));
        };
        if target.contribute == enabled {
            return Ok(FactionContributorOutcome::Rejected(
                FactionContributorRejection::Unchanged,
            ));
        }

        target.contribute = enabled;
        let mut progress = FactionContributorProgress {
            contributor_changed: true,
            member_update: None,
            refreshed_player_ids: Vec::new(),
            member_information: None,
            dirty_set: false,
        };
        progress.member_update = Some(match self.update_member_info_to_client(
            game,
            target_id,
            EOperator::Update,
        ) {
            Ok(report) => report,
            Err(source) => {
                return Err(FactionContributorBlock::MemberUpdate { source, progress });
            }
        });
        progress.refreshed_player_ids =
            self.update_player_faction_info(game, target_id, |player_id| {
                context.update_player_faction_info(player_id);
            });

        let target = self
            .members
            .get(&target_id)
            .expect("успешный target lookup и callbacks не удаляют member");
        let target_name_wire = match target.name_wire_bytes() {
            Ok(name) => name,
            Err(source) => {
                return Err(FactionContributorBlock::UnterminatedMemberName {
                    player_id: target_id,
                    source,
                    progress,
                });
            }
        };
        let notice = context.format_contributor_string(
            if enabled { b"WS0227" } else { b"WS0228" },
            &target_name_wire[..target_name_wire.len() - 1],
        );
        let notice = legacy_c_string_visible_bytes(&notice);
        if notice.len() >= FACTION_CONTRIBUTOR_NOTICE_CAPACITY {
            return Err(FactionContributorBlock::NoticeWouldOverflow {
                formatted_len: notice.len(),
                progress,
            });
        }
        let second_text = context.world_string(b"WS0119").unwrap_or_default();
        progress.member_information = Some(self.send_info_to_all_members_with_color(
            notice,
            legacy_c_string_visible_bytes(&second_text),
            -1,
            0x0087_A238,
            |request| context.send_organizing_info(request),
        ));
        self.set_change_data(2);
        progress.dirty_set = true;

        Ok(FactionContributorOutcome::Updated(progress))
    }

    /// Возвращает младшие 16 бит `lJobLvl` либо исходный `0` при miss.
    pub(crate) fn member_job_level(&self, player_id: i32) -> u16 {
        self.members
            .get(&player_id)
            .map_or(0, |member| member.job_level as u16)
    }

    /// Проверяет точное состояние `PST_Permit` одного member-права.
    pub(crate) fn is_using_purview(&self, player_id: i32, purview: i32) -> bool {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return false;
        };
        self.members
            .get(&player_id)
            .is_some_and(|member| member.purview[purview.index()] == EPurviewOwnState::Permit)
    }

    /// Переводит только `PST_No` в `PST_Permit`.
    pub(crate) fn set_member_purview(
        &mut self,
        player_id: i32,
        purview: i32,
    ) -> MemberPurviewMutation {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return MemberPurviewMutation::InvalidPurview;
        };
        let Some(member) = self.members.get_mut(&player_id) else {
            return MemberPurviewMutation::MemberNotFound;
        };
        let state = &mut member.purview[purview.index()];
        if *state != EPurviewOwnState::No {
            return MemberPurviewMutation::Unchanged;
        }
        *state = EPurviewOwnState::Permit;
        MemberPurviewMutation::Changed
    }

    /// Переводит любое не-`PST_No` состояние в `PST_No`.
    pub(crate) fn abolish_member_purview(
        &mut self,
        player_id: i32,
        purview: i32,
    ) -> MemberPurviewMutation {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return MemberPurviewMutation::InvalidPurview;
        };
        let Some(member) = self.members.get_mut(&player_id) else {
            return MemberPurviewMutation::MemberNotFound;
        };
        let state = &mut member.purview[purview.index()];
        if *state == EPurviewOwnState::No {
            return MemberPurviewMutation::Unchanged;
        }
        *state = EPurviewOwnState::No;
        MemberPurviewMutation::Changed
    }

    /// Проверяет членство и одно право requester-а.
    pub(crate) fn check_operator_validate(&self, requester_id: i32, purview: i32) -> bool {
        self.is_member(requester_id) != 0 && self.is_using_purview(requester_id, purview)
    }

    /// Проверяет операцию requester над другим faction-member.
    pub(crate) fn check_operator_validate_target(
        &self,
        requester_id: i32,
        target_id: i32,
        purview: i32,
    ) -> Result<bool, FactionOperatorValidationBlock> {
        let master_id = self.master_id.ok_or(FactionOperatorValidationBlock)?;
        if requester_id == target_id
            || target_id == master_id
            || self.is_member(requester_id) == 0
            || self.is_member(target_id) == 0
            || !self.is_using_purview(requester_id, purview)
        {
            return Ok(false);
        }
        if self.is_using_purview(target_id, purview) && requester_id != master_id {
            return Ok(false);
        }
        Ok(true)
    }

    /// Возвращает один снимок исходной signed dirty-bit mask.
    pub(crate) const fn change_data_type(&self) -> i32 {
        self.change_data_type
    }

    /// Применяет точную bit-mask семантику virtual `SetChangeData`.
    pub(crate) fn set_change_data(&mut self, change_data_type: i32) {
        if change_data_type == 0 {
            self.change_data_type = 0;
        } else if self.change_data_type & change_data_type == 0 {
            self.change_data_type |= change_data_type;
        }
    }

    /// Создаёт отдельную save-копию ровно по dirty-битам `1/2/4/8`.
    pub(crate) fn clone_save_data(&self) -> Result<Option<Self>, FactionCloneSaveBlock> {
        let change_data_type = self.change_data_type;
        if change_data_type == 0 {
            return Ok(None);
        }

        let copy_property = change_data_type & 1 != 0;
        let master_id = if copy_property {
            Some(
                self.master_id
                    .ok_or(FactionCloneSaveBlock::MasterIdMissing)?,
            )
        } else {
            None
        };
        let base_property = if copy_property {
            Some(
                self.base_property
                    .ok_or(FactionCloneSaveBlock::MissingBaseProperty)?,
            )
        } else {
            None
        };
        let established_time = if copy_property {
            Some(
                self.established_time
                    .ok_or(FactionCloneSaveBlock::EstablishedTimeUnknown)?,
            )
        } else {
            None
        };
        let delete_remain_time = if copy_property {
            Some(
                self.delete_remain_time
                    .ok_or(FactionCloneSaveBlock::DeleteRemainTimeAbsent)?,
            )
        } else {
            None
        };

        Ok(Some(Self {
            faction_id: self.faction_id,
            name: if copy_property {
                self.name.clone()
            } else {
                Vec::new()
            },
            master_id,
            members: if change_data_type & 2 != 0 {
                self.members.clone()
            } else {
                BTreeMap::new()
            },
            base_property,
            established_time,
            delete_remain_time,
            owned_cities: if change_data_type & 8 != 0 {
                self.owned_cities.clone()
            } else {
                VecDeque::new()
            },
            // CloneSaveData не копирует оба runtime enemy-set.
            enemy_factions: BTreeSet::new(),
            city_war_enemy_factions: BTreeSet::new(),
            // Private clone-constructor не назначает transient bool.
            permit_demise: None,
            enemy_factions_changed: None,
            city_war_enemy_factions_changed: None,
            // DB ability-owner читает ordered keys; wire-owner подтверждает
            // и сохраняемый вместе с ними полный tagApplyPerson value.
            apply_persons: if change_data_type & 8 != 0 {
                self.apply_persons.clone()
            } else {
                BTreeMap::new()
            },
            pronounce: if change_data_type & 8 != 0 {
                self.pronounce
            } else {
                TagPronounceWord::ZERO
            },
            leave_words: if change_data_type & 4 != 0 {
                self.leave_words.clone()
            } else {
                VecDeque::new()
            },
            last_upload_icon_time: if change_data_type & 8 != 0 {
                self.last_upload_icon_time
            } else {
                ZERO_TIME
            },
            icon_data: if change_data_type & 8 != 0 {
                self.icon_data.clone()
            } else {
                Vec::new()
            },
            change_data_type,
            // Private clone-constructor не назначает эти поля; текущие
            // безопасные значения остаются только до будущей canonical
            // нормализации SaveFactionProperty.
            goods_war_count: 0,
            goods_war_last_win_time: String::new(),
        }))
    }

    /// Обновляет Goods War count/time и возвращает сохранённый signed count.
    pub(crate) fn set_goods_war_count(&mut self, goods_war_count: i32) -> i32 {
        let now = Local::now();
        self.goods_war_last_win_time = format!(
            "{}-{}-{} {}:{}:{}",
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        );
        self.set_change_data(1);
        self.goods_war_count = if goods_war_count <= 0 {
            0
        } else {
            goods_war_count
        };
        self.goods_war_count
    }

    /// Возвращает текущий signed Goods War count save-копии.
    pub(crate) const fn goods_war_count(&self) -> i32 {
        self.goods_war_count
    }

    /// Возвращает текущую ASCII time-строку save-копии.
    pub(crate) fn goods_war_last_win_time(&self) -> &str {
        &self.goods_war_last_win_time
    }

    /// Заменяет только time-строку save-копии до ADO/TDS update.
    pub(crate) fn set_goods_war_last_win_time(&mut self, value: String) {
        self.goods_war_last_win_time = value;
    }

    /// Возвращает исходный const-view ordered `m_Members`.
    pub(crate) const fn get_members(&self) -> &BTreeMap<i32, TagMemInfo> {
        &self.members
    }

    /// Перегрузка `list<COrganizing*>`: faction не возвращает вложенные owners.
    pub(crate) fn get_organizing_member_list<T>(&self, output: &mut VecDeque<T>) {
        output.clear();
    }

    /// Перегрузка `list<long>`: заменяет output полным ordered списком ID.
    pub(crate) fn get_member_id_list(&self, output: &mut VecDeque<i32>) {
        output.clear();
        output.extend(self.members.keys().copied());
    }

    /// Возвращает полный ordered map исходного `m_ApplyPersons`.
    pub(crate) const fn get_apply_persons(&self) -> &BTreeMap<i32, TagApplyPerson> {
        &self.apply_persons
    }

    /// Даёт DB owner-у прежний zero-copy ordered view только ключей.
    pub(crate) fn get_apply_person_ids(&self) -> impl Iterator<Item = &i32> {
        self.apply_persons.keys()
    }

    /// Возвращает faction ID при наличии player-key в `m_ApplyPersons`.
    pub(crate) fn is_in_apply_members(&self, player_id: i32) -> i32 {
        if self.apply_persons.contains_key(&player_id) {
            self.faction_id
        } else {
            0
        }
    }

    /// Очищает apply-list только при точном состоянии права `ConMem`.
    pub(crate) fn clear_apply_list(&mut self, operator_id: i32) -> bool {
        if !self.is_using_purview(operator_id, EPurview::ConMem as i32) {
            return false;
        }
        self.apply_persons.clear();
        true
    }

    /// Дописывает byte-exact `m_Pronounce` в исходный output-vector.
    pub(crate) fn get_pronounce_data(&self, output: &mut Vec<u8>) -> bool {
        self.pronounce.append_wire_bytes(output);
        true
    }

    /// Возвращает исходный const-view ordered `m_LeaveWords`.
    pub(crate) const fn get_leave_words(&self) -> &VecDeque<TagLeaveWord> {
        &self.leave_words
    }

    /// Возвращает дату последней загрузки faction-icon.
    pub(crate) const fn last_upload_icon_time(&self) -> TagTimeValue {
        self.last_upload_icon_time
    }

    /// Возвращает byte-exact исходный `m_IconData`.
    pub(crate) fn icon_data(&self) -> &[u8] {
        &self.icon_data
    }

    /// Возвращает младший 32-битный шаблон старого `map::_Mysize`.
    pub(crate) fn get_member_num(&self) -> i32 {
        self.members.len() as u32 as i32
    }

    /// Возвращает faction ID только для существующего member key.
    pub(crate) fn is_member(&self, player_id: i32) -> i32 {
        if self.members.contains_key(&player_id) {
            self.faction_id
        } else {
            0
        }
    }

    /// Дописывает полный ordered member snapshot в исходном byte-array формате.
    pub(crate) fn add_members_to_byte_array(
        &self,
        output: &mut Vec<u8>,
    ) -> Result<bool, UnterminatedMemberField> {
        output.extend_from_slice(&(self.members.len() as u32).to_le_bytes());
        for member in self.members.values() {
            append_i32(output, member.id);
            append_i32(output, member.job_level);
            output.extend_from_slice(member.title_wire_bytes()?);
            output.extend_from_slice(&member.purview_wire_bytes());
            append_i32(output, member.level);
            append_i32(output, member.occupation);
            output.extend_from_slice(member.name_wire_bytes()?);
            output.extend_from_slice(&u32::from(member.contribute).to_le_bytes());
            output.extend_from_slice(member.region_wire_bytes()?);
            output.extend_from_slice(&member.last_online_wire_bytes());
        }
        Ok(true)
    }

    /// Дописывает полный ordered snapshot кандидатов на вступление.
    pub(crate) fn add_apply_persons_to_byte_array(
        &self,
        output: &mut Vec<u8>,
    ) -> Result<bool, UnterminatedApplyPersonName> {
        output.extend_from_slice(&(self.apply_persons.len() as u32).to_le_bytes());
        for (completed_persons, person) in self.apply_persons.values().enumerate() {
            append_i32(output, person.id);
            let Some(name) = person.name_wire_bytes() else {
                return Err(UnterminatedApplyPersonName {
                    player_id: person.id,
                    completed_persons,
                });
            };
            output.extend_from_slice(name);
            append_i32(output, person.occupation);
            append_i32(output, person.level);
        }
        Ok(true)
    }

    /// Дописывает полный leave-word snapshot в исходном list-order.
    pub(crate) fn add_leave_words_to_byte_array(
        &self,
        output: &mut Vec<u8>,
    ) -> Result<bool, UnterminatedLeaveWordField> {
        output.extend_from_slice(&(self.leave_words.len() as u32).to_le_bytes());
        for leave_word in &self.leave_words {
            append_i32(output, leave_word.id);
            append_i32(output, leave_word.player_id);
            output.extend_from_slice(&leave_word.time.wire_bytes());
            output.extend_from_slice(leave_word.content_wire_bytes()?);
            output.extend_from_slice(leave_word.name_wire_bytes()?);
        }
        Ok(true)
    }

    /// Публикует delete либо полный non-delete member-update всем готовым
    /// получателям.
    pub(crate) fn update_member_info_to_client(
        &mut self,
        game: &CGame,
        target_player_id: i32,
        operator: EOperator,
    ) -> Result<MemberUpdateReport, MemberUpdateBuildError> {
        let target_found = if operator == EOperator::Delete {
            None
        } else {
            let Some(target) = self.members.get_mut(&target_player_id) else {
                return Ok(MemberUpdateReport {
                    target_found: Some(false),
                    deliveries: Vec::new(),
                });
            };
            target.last_online_time = current_local_member_time();
            Some(true)
        };

        let mut deliveries = Vec::new();
        for &recipient_player_id in self.members.keys() {
            let player = game.online_player_by_id(recipient_player_id as u32);
            let game_server_id = game.game_server_number_by_player_id(recipient_player_id);
            if player.is_none_or(|player| !player.faction_data_received()) || game_server_id == 0 {
                continue;
            }

            let mut message = CMessage::new(MEMBER_UPDATE_MESSAGE_TYPE);
            message.base_mut().add_long(recipient_player_id);
            message.base_mut().add_long(operator.wire_value());
            message.base_mut().add_long(target_player_id);

            if operator != EOperator::Delete {
                let target = self
                    .members
                    .get(&target_player_id)
                    .expect("non-delete target найден до recipient-прохода");
                let mut fields = Vec::new();
                if let Err(field) = append_member_update_fields(&mut fields, target) {
                    return Err(MemberUpdateBuildError {
                        field,
                        recipient_player_id,
                        game_server_id,
                        completed_deliveries: deliveries,
                    });
                }
                message.base_mut().add(&fields);
            }

            deliveries.push(MemberUpdateDelivery {
                recipient_player_id,
                game_server_id,
                result: game.send_msg_to_game_server(game_server_id, &message),
            });
        }

        Ok(MemberUpdateReport {
            target_found,
            deliveries,
        })
    }

    /// Обновляет `tagMemInfo::lLvl` и публикует operator `Update`.
    pub(crate) fn on_member_level_change(
        &mut self,
        game: &CGame,
        player_id: i32,
        level: i32,
    ) -> MemberLevelChangeOutcome {
        let Some(member) = self.members.get_mut(&player_id) else {
            return MemberLevelChangeOutcome::MemberNotFound;
        };
        if member.level == level {
            return MemberLevelChangeOutcome::Unchanged;
        }
        member.level = level;
        MemberLevelChangeOutcome::Published(self.update_member_info_to_client(
            game,
            player_id,
            EOperator::Update,
        ))
    }

    /// Копирует имя заданного региона в member-state и публикует `Update`.
    pub(crate) fn on_member_position_change(
        &mut self,
        game: &CGame,
        player_id: i32,
        region_id: i32,
    ) -> MemberPositionChangeOutcome {
        if !self.members.contains_key(&player_id) {
            return MemberPositionChangeOutcome::MemberNotFound;
        }

        let region_name = match game.region_name(region_id) {
            WorldRegionNameLookup::RegionNotFound => {
                return MemberPositionChangeOutcome::RegionNotFound;
            }
            WorldRegionNameLookup::NullRegionPointer => {
                return MemberPositionChangeOutcome::NullRegionPointer;
            }
            WorldRegionNameLookup::Name(name) => name,
        };
        let member = self
            .members
            .get_mut(&player_id)
            .expect("member проверен до region lookup");
        if region_name.len() >= member.region.len() {
            return MemberPositionChangeOutcome::Blocked(MemberPositionChangeBlocked {
                region_id,
                byte_len: region_name.len(),
            });
        }

        member.region[..region_name.len()].copy_from_slice(region_name);
        member.region[region_name.len()] = 0;
        MemberPositionChangeOutcome::Published(self.update_member_info_to_client(
            game,
            player_id,
            EOperator::Update,
        ))
    }

    /// Обновляет byte-exact online-регион участника и публикует изменение.
    pub(crate) fn on_member_enter_game(
        &mut self,
        game: &CGame,
        player_id: i32,
    ) -> MemberEnterOutcome {
        let Some(player) = game.online_player_by_id(player_id as u32) else {
            return MemberEnterOutcome::PlayerNotOnline;
        };
        if !self.members.contains_key(&player_id) {
            return MemberEnterOutcome::MemberNotFound;
        }

        let region_id = player.get_region_id();
        let region_name = match game.region_name(region_id) {
            WorldRegionNameLookup::RegionNotFound => &[][..],
            WorldRegionNameLookup::NullRegionPointer => {
                // BLOCKED_MISSING_FACT: exact EXE `0x004C0A96..0x004C0A98`
                // разыменовывает найденный tagRegion::pRegion без null-check.
                // Достижимость и наблюдаемая реакция null не доказаны.
                return MemberEnterOutcome::Blocked(MemberEnterBlockedReason::NullRegionPointer {
                    region_id,
                });
            }
            WorldRegionNameLookup::Name(name) => name,
        };

        if region_name.len() >= ENTER_REGION_BUFFER_CAPACITY {
            // BLOCKED_MISSING_FACT: старый strcpy по `0x004C0AB0..0x004C0ABC`
            // переполнял бы локальный char[256]. Safe Rust не назначает этому
            // пути новый результат и не воспроизводит запись через unsafe.
            return MemberEnterOutcome::Blocked(
                MemberEnterBlockedReason::RegionNameExceedsLocalBuffer {
                    region_id,
                    byte_len: region_name.len(),
                },
            );
        }

        let member_region_matches = {
            let member = self
                .members
                .get(&player_id)
                .expect("member проверен до region lookup");
            let member_region = match member.region_wire_bytes() {
                Ok(bytes) => &bytes[..bytes.len() - 1],
                Err(field) => {
                    // BLOCKED_MISSING_FACT: strcmp читал бы за strRegion[64].
                    return MemberEnterOutcome::Blocked(
                        MemberEnterBlockedReason::UnterminatedMemberRegion(field),
                    );
                }
            };
            member_region == region_name
        };
        if member_region_matches {
            return MemberEnterOutcome::RegionUnchanged;
        }

        let member_region_capacity = self
            .members
            .get(&player_id)
            .expect("member проверен до region lookup")
            .region
            .len();
        if region_name.len() >= member_region_capacity {
            // BLOCKED_MISSING_FACT: второй strcpy по `0x004C0B07..0x004C0B11`
            // переполнял бы strRegion[64] уже после доказанного неравенства.
            return MemberEnterOutcome::Blocked(
                MemberEnterBlockedReason::RegionNameExceedsMemberField {
                    region_id,
                    byte_len: region_name.len(),
                },
            );
        }

        let member = self
            .members
            .get_mut(&player_id)
            .expect("member проверен до region lookup");
        member.region[..region_name.len()].copy_from_slice(region_name);
        member.region[region_name.len()] = 0;

        MemberEnterOutcome::Published(self.update_member_info_to_client(
            game,
            player_id,
            EOperator::Update,
        ))
    }

    /// Очищает online-регион участника и публикует исходный update при изменении.
    pub(crate) fn on_member_exit_game(
        &mut self,
        game: &CGame,
        player_id: i32,
    ) -> MemberExitOutcome {
        let Some(member) = self.members.get_mut(&player_id) else {
            return MemberExitOutcome::MemberNotFound;
        };

        let exit_time = current_local_member_time();
        if member.region[0] == 0 {
            return MemberExitOutcome::RegionAlreadyEmpty;
        }

        member.region[0] = 0;
        member.last_online_time = exit_time;
        MemberExitOutcome::Published(self.update_member_info_to_client(
            game,
            player_id,
            EOperator::Update,
        ))
    }
}

/// Проверяет субботнее Goods War окно и только затем membership faction ID.
pub(crate) fn goods_war_check_for_faction<F>(
    faction: Option<&CFaction>,
    is_in_faction_id_list: F,
) -> bool
where
    F: FnOnce(i32) -> bool,
{
    let Some(faction) = faction else {
        return false;
    };
    let now = Local::now();
    if now.weekday().num_days_from_sunday() != 6 {
        return false;
    }
    let inside_time_window = match now.hour() {
        19 => now.minute() >= 30,
        20 => true,
        21 => now.minute() <= 10,
        _ => false,
    };
    inside_time_window && is_in_faction_id_list(faction.faction_id())
}

fn current_local_member_time() -> TagTimeValue {
    let now = Local::now();
    TagTimeValue {
        year: now.year() as u16,
        month: now.month() as u16,
        day_of_week: now.weekday().num_days_from_sunday() as u16,
        day: now.day() as u16,
        hour: now.hour() as u16,
        minute: now.minute() as u16,
        second: now.second() as u16,
        milliseconds: now.timestamp_subsec_millis() as u16,
    }
}

/// Дописывает только доказанную per-member часть non-delete update-сообщения.
fn append_member_update_fields(
    output: &mut Vec<u8>,
    member: &TagMemInfo,
) -> Result<(), UnterminatedMemberField> {
    output.extend_from_slice(member.name_wire_bytes()?);
    append_i32(output, member.level);
    append_i32(output, member.occupation);
    append_i32(output, member.job_level);
    output.extend_from_slice(member.title_wire_bytes()?);
    output.extend_from_slice(&member.purview_wire_bytes());
    output.extend_from_slice(member.region_wire_bytes()?);
    output.extend_from_slice(&u32::from(member.contribute).to_le_bytes());
    output.extend_from_slice(&member.last_online_wire_bytes());
    Ok(())
}

fn append_i32(output: &mut Vec<u8>, value: i32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn legacy_c_string_visible_bytes(value: &[u8]) -> &[u8] {
    match value.iter().position(|byte| *byte == 0) {
        Some(terminator) => &value[..terminator],
        None => value,
    }
}

fn legacy_c_string_wire_bytes(value: &[u8]) -> Vec<u8> {
    let visible = legacy_c_string_visible_bytes(value);
    let mut wire = Vec::with_capacity(visible.len() + 1);
    wire.extend_from_slice(visible);
    wire.push(0);
    wire
}

fn empty_faction_set_parameter_progress(
    parameter: FactionSetParameterKind,
) -> FactionSetParameterProgress {
    FactionSetParameterProgress {
        parameter,
        level_update: None,
        experience_update: None,
        assigned_experience: None,
        assigned_upgrade_experience: None,
        property_deliveries: None,
        refreshed_player_ids: None,
        dirty_bit_requested: false,
    }
}

fn send_apply_join_information<Context>(
    context: &mut Context,
    recipient_player_id: i32,
    first_string_id: &'static [u8],
    second_string_id: &'static [u8],
) where
    Context: FactionOrganizingInfoContext + ?Sized,
{
    let second_text = context.world_string(second_string_id).unwrap_or_default();
    let first_text = context.world_string(first_string_id).unwrap_or_default();
    context.send_organizing_info(FactionMemberInfoRequest {
        recipient_player_id,
        first_text: legacy_c_string_visible_bytes(&first_text),
        second_text: legacy_c_string_visible_bytes(&second_text),
        information_type: -1,
        color: 0xFFDA_EDFE,
        trailing_value: 0,
    });
}

fn ensure_do_join_string_fits<ContextBlock>(
    field: FactionDoJoinStringField,
    visible_len: usize,
    capacity: usize,
    application_removed: bool,
    member_inserted: bool,
) -> Result<(), FactionDoJoinBlock<ContextBlock>> {
    if visible_len >= capacity {
        return Err(FactionDoJoinBlock::StringWouldOverflow {
            field,
            visible_len,
            capacity,
            application_removed,
            member_inserted,
        });
    }
    Ok(())
}

fn faction_purview_notice_string_id(
    change: FactionPurviewChange,
    purview: i32,
) -> &'static [u8] {
    match (change, purview) {
        (FactionPurviewChange::Grant, 2) => b"WS0197",
        (FactionPurviewChange::Grant, 3) => b"WS0198",
        (FactionPurviewChange::Grant, 4) => b"WS0199",
        (FactionPurviewChange::Grant, 5) => b"WS0200",
        (FactionPurviewChange::Grant, 6) => b"WS0201",
        (FactionPurviewChange::Grant, 7) => b"WS0204",
        (FactionPurviewChange::Grant, 8) => b"WS0203",
        (FactionPurviewChange::Grant, 9) => b"WS0202",
        (FactionPurviewChange::Revoke, 2) => b"WS0205",
        (FactionPurviewChange::Revoke, 3) => b"WS0206",
        (FactionPurviewChange::Revoke, 4) => b"WS0207",
        (FactionPurviewChange::Revoke, 5) => b"WS0208",
        (FactionPurviewChange::Revoke, 6) => b"WS0209",
        (FactionPurviewChange::Revoke, 7) => b"WS0212",
        (FactionPurviewChange::Revoke, 8) => b"WS0211",
        (FactionPurviewChange::Revoke, 9) => b"WS0210",
        _ => unreachable!("purview диапазон проверен до выбора notice ID"),
    }
}

fn empty_faction_upgrade_progress() -> FactionUpgradeProgress {
    FactionUpgradeProgress {
        level_update: None,
        experience_update: None,
        charge_delivery: None,
        next_upgrade_experience: None,
        member_information: None,
        property_deliveries: None,
        dirty_set: false,
        refreshed_player_ids: Vec::new(),
        log_written: false,
    }
}

fn empty_faction_demise_progress() -> FactionDemiseProgress {
    FactionDemiseProgress {
        permit_demise_disabled: false,
        master_changed: false,
        new_member_changed: false,
        old_member_changed: false,
        old_member_update: None,
        new_member_update: None,
        base_dirty_set: false,
        members_dirty_set: false,
        refreshed_player_ids: Vec::new(),
        member_information: None,
        log_written: false,
    }
}

fn bounded_demise_notice(value: &[u8], capacity: usize) -> Result<Vec<u8>, usize> {
    let visible = legacy_c_string_visible_bytes(value);
    if visible.len() >= capacity {
        return Err(visible.len());
    }
    Ok(visible.to_vec())
}

fn fixed_demise_title(
    value: &[u8],
) -> Result<[u8; FACTION_MEMBER_TEXT_CAPACITY], usize> {
    let visible = legacy_c_string_visible_bytes(value);
    if visible.len() >= FACTION_MEMBER_TEXT_CAPACITY {
        return Err(visible.len());
    }
    let mut title = [0; FACTION_MEMBER_TEXT_CAPACITY];
    title[..visible.len()].copy_from_slice(visible);
    Ok(title)
}

fn demise_new_master_purview() -> [EPurviewOwnState; 11] {
    [
        EPurviewOwnState::Permit,
        EPurviewOwnState::No,
        EPurviewOwnState::Permit,
        EPurviewOwnState::Permit,
        EPurviewOwnState::Permit,
        EPurviewOwnState::Permit,
        EPurviewOwnState::Permit,
        EPurviewOwnState::Permit,
        EPurviewOwnState::Permit,
        EPurviewOwnState::Permit,
        EPurviewOwnState::Permit,
    ]
}

fn demise_old_master_purview() -> [EPurviewOwnState; 11] {
    [
        EPurviewOwnState::No,
        EPurviewOwnState::Permit,
        EPurviewOwnState::No,
        EPurviewOwnState::No,
        EPurviewOwnState::No,
        EPurviewOwnState::No,
        EPurviewOwnState::Permit,
        EPurviewOwnState::No,
        EPurviewOwnState::No,
        EPurviewOwnState::No,
        EPurviewOwnState::No,
    ]
}

fn faction_upgrade_notice_string_id(notice: FactionUpgradeNotice) -> &'static [u8] {
    match notice {
        FactionUpgradeNotice::Experience => b"WS0220",
        FactionUpgradeNotice::Money => b"WS0221",
        FactionUpgradeNotice::MasterLevel => b"WS0222",
        FactionUpgradeNotice::Goods => b"WS0223",
        FactionUpgradeNotice::Success => b"WS0224",
    }
}

fn format_faction_upgrade_notice<Context>(
    context: &mut Context,
    notice: FactionUpgradeNotice,
    arguments: &[FactionUpgradeFormatArgument<'_>],
) -> Result<Vec<u8>, usize>
where
    Context: FactionUpgradeContext,
{
    let formatted = context.format_upgrade_string(
        faction_upgrade_notice_string_id(notice),
        arguments,
    );
    let formatted = legacy_c_string_visible_bytes(&formatted);
    if formatted.len() >= FACTION_UPGRADE_NOTICE_CAPACITY {
        return Err(formatted.len());
    }
    Ok(formatted.to_vec())
}

fn fixed_do_join_string<const CAPACITY: usize, ContextBlock>(
    field: FactionDoJoinStringField,
    value: &[u8],
    application_removed: bool,
    member_inserted: bool,
) -> Result<[u8; CAPACITY], FactionDoJoinBlock<ContextBlock>> {
    let visible = legacy_c_string_visible_bytes(value);
    ensure_do_join_string_fits::<ContextBlock>(
        field,
        visible.len(),
        CAPACITY,
        application_removed,
        member_inserted,
    )?;
    let mut output = [0; CAPACITY];
    output[..visible.len()].copy_from_slice(visible);
    Ok(output)
}

fn append_signed_set(output: &mut Vec<u8>, values: &BTreeSet<i32>) {
    output.extend_from_slice(&(values.len() as u32).to_le_bytes());
    for &value in values {
        append_i32(output, value);
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h

// ============================================================================
// FUNCTION: CFaction::InitialPropertyByLvl
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:155
// RVA: 0x000B4BA0
// ADDRESS: 004b4ba0
// PROTOTYPE: bool __thiscall InitialPropertyByLvl(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::CheckOperValidate
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:731
// RVA: 0x000B4CE0
// ADDRESS: 004b4ce0
// PROTOTYPE: bool __thiscall CheckOperValidate(long param_1, long param_2, ePurview param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CFaction::SetDelRemainTime` RVA `0x000B4DF0` находится выше.

// ============================================================================
// FUNCTION: CFaction::GetIsPermit
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2431
// RVA: 0x000B4E20
// ADDRESS: 004b4e20
// PROTOTYPE: bool __thiscall GetIsPermit(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateEnemyFaction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2639
// RVA: 0x000B4E30
// ADDRESS: 004b4e30
// PROTOTYPE: void __thiscall UpdateEnemyFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateCityWarEnemyFaction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2649
// RVA: 0x000B4E60
// ADDRESS: 004b4e60
// PROTOTYPE: void __thiscall UpdateCityWarEnemyFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IncMaxNumber
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2425
// RVA: 0x000B4E90
// ADDRESS: 004b4e90
// PROTOTYPE: bool __thiscall IncMaxNumber(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GoodsWarCheckforFaction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:50
// RVA: 0x000B5070
// ADDRESS: 004b5070
// PROTOTYPE: bool __cdecl GoodsWarCheckforFaction(CFaction * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsHaveEnymyFaction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:637
// RVA: 0x000B50F0
// ADDRESS: 004b50f0
// PROTOTYPE: bool __thiscall IsHaveEnymyFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsHaveCityEnemyFaction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:644
// RVA: 0x000B5100
// ADDRESS: 004b5100
// PROTOTYPE: bool __thiscall IsHaveCityEnemyFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetSuperiorOrganizing
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1260
// RVA: 0x000B5110
// ADDRESS: 004b5110
// PROTOTYPE: void __thiscall SetSuperiorOrganizing(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsOwnedCity
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:456
// RVA: 0x000B5490
// ADDRESS: 004b5490
// PROTOTYPE: long __thiscall IsOwnedCity(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::ClearOwnedCity
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:504
// RVA: 0x000B54C0
// ADDRESS: 004b54c0
// PROTOTYPE: bool __thiscall ClearOwnedCity(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetMemberList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:534
// RVA: 0x000B5530
// ADDRESS: 004b5530
// PROTOTYPE: void __thiscall GetMemberList(list<COrganizing*,std::allocator<COrganizing*>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::RefreshOwnCityInfo
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1288
// RVA: 0x000B5570
// ADDRESS: 004b5570
// PROTOTYPE: void __thiscall RefreshOwnCityInfo(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateExpToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1366
// RVA: 0x000B55B0
// ADDRESS: 004b55b0
// PROTOTYPE: void __thiscall UpdateExpToClient(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdatePronounceToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1497
// RVA: 0x000B56C0
// ADDRESS: 004b56c0
// PROTOTYPE: void __thiscall UpdatePronounceToClient(long param_1, eOperator param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetControbuterNum
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2507
// RVA: 0x000B57E0
// ADDRESS: 004b57e0
// PROTOTYPE: long __thiscall GetControbuterNum(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdatePlayerFactionInfo
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2613
// RVA: 0x000B5820
// ADDRESS: 004b5820
// PROTOTYPE: void __thiscall UpdatePlayerFactionInfo(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SendInfoToAllMember
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2730
// RVA: 0x000B5890
// ADDRESS: 004b5890
// PROTOTYPE: void __thiscall SendInfoToAllMember(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, long param_3, ulong param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateOtherFacInfoToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2740
// RVA: 0x000B58F0
// ADDRESS: 004b58f0
// PROTOTYPE: void __thiscall UpdateOtherFacInfoToClient(long param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, eOperator param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::Talk
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2776
// RVA: 0x000B5A50
// ADDRESS: 004b5a50
// PROTOTYPE: void __thiscall Talk(long param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddApplyPersonsToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:364
// RVA: 0x000B5D30
// ADDRESS: 004b5d30
// PROTOTYPE: bool __thiscall AddApplyPersonsToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddLeaveWordsToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:389
// RVA: 0x000B5DD0
// ADDRESS: 004b5dd0
// PROTOTYPE: bool __thiscall AddLeaveWordsToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddEnemyFactionsToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:423
// RVA: 0x000B5E40
// ADDRESS: 004b5e40
// PROTOTYPE: bool __thiscall AddEnemyFactionsToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddCityWarEnemyFactionsToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:435
// RVA: 0x000B5ED0
// ADDRESS: 004b5ed0
// PROTOTYPE: bool __thiscall AddCityWarEnemyFactionsToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::DelOwnedCity
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:490
// RVA: 0x000B5F60
// ADDRESS: 004b5f60
// PROTOTYPE: bool __thiscall DelOwnedCity(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetEnemyList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:551
// RVA: 0x000B5FF0
// ADDRESS: 004b5ff0
// PROTOTYPE: void __thiscall GetEnemyList(list<COrganizing*,std::allocator<COrganizing*>_> param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::ClearEnemyFation
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:675
// RVA: 0x000B6030
// ADDRESS: 004b6030
// PROTOTYPE: void __thiscall ClearEnemyFation(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::ClearCityWarEnemyFation
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:683
// RVA: 0x000B6080
// ADDRESS: 004b6080
// PROTOTYPE: void __thiscall ClearCityWarEnemyFation(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateAllApplyMemberToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:924
// RVA: 0x000B60D0
// ADDRESS: 004b60d0
// PROTOTYPE: void __thiscall UpdateAllApplyMemberToClient(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetExp
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1307
// RVA: 0x000B61F0
// ADDRESS: 004b61f0
// PROTOTYPE: void __thiscall SetExp(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateLeaveWordToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1453
// RVA: 0x000B6240
// ADDRESS: 004b6240
// PROTOTYPE: void __thiscall UpdateLeaveWordToClient(long param_1, eOperator param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::ClearApplyList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1520
// RVA: 0x000B6450
// ADDRESS: 004b6450
// PROTOTYPE: bool __thiscall ClearApplyList(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsControbute
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2457
// RVA: 0x000B64A0
// ADDRESS: 004b64a0
// PROTOTYPE: bool __thiscall IsControbute(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::OnMemberLvlChange
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2583
// RVA: 0x000B6590
// ADDRESS: 004b6590
// PROTOTYPE: void __thiscall OnMemberLvlChange(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsInApplyMembers
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:791
// RVA: 0x000B6760
// ADDRESS: 004b6760
// PROTOTYPE: long __thiscall IsInApplyMembers(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::EditLeaveWord
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2075
// RVA: 0x000B6790
// ADDRESS: 004b6790
// PROTOTYPE: bool __thiscall EditLeaveWord(long param_1, long param_2, eOperator param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetLWFunction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1101
// RVA: 0x000B7030
// ADDRESS: 004b7030
// PROTOTYPE: void __thiscall SetLWFunction(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetPronounceFun
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1117
// RVA: 0x000B7400
// ADDRESS: 004b7400
// PROTOTYPE: void __thiscall SetPronounceFun(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetEndueRightFun
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1131
// RVA: 0x000B77D0
// ADDRESS: 004b77d0
// PROTOTYPE: void __thiscall SetEndueRightFun(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetJoinVillageWarFun
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1159
// RVA: 0x000B7BA0
// ADDRESS: 004b7ba0
// PROTOTYPE: void __thiscall SetJoinVillageWarFun(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetJoinCityWarFun
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1173
// RVA: 0x000B7F70
// ADDRESS: 004b7f70
// PROTOTYPE: void __thiscall SetJoinCityWarFun(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetCreateUnionFun
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1187
// RVA: 0x000B8340
// ADDRESS: 004b8340
// PROTOTYPE: void __thiscall SetCreateUnionFun(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetMaxMememberNums
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1202
// RVA: 0x000B8710
// ADDRESS: 004b8710
// PROTOTYPE: void __thiscall SetMaxMememberNums(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetLvl
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1215
// RVA: 0x000B8940
// ADDRESS: 004b8940
// PROTOTYPE: void __thiscall SetLvl(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::DeleteOrgaToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1647
// RVA: 0x000B8A20
// ADDRESS: 004b8a20
// PROTOTYPE: void __thiscall DeleteOrgaToClient(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::Pronounce
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2000
// RVA: 0x000B8DC0
// ADDRESS: 004b8dc0
// PROTOTYPE: bool __thiscall Pronounce(long param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, tagTime * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UploadIcon
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2395
// RVA: 0x000B8F30
// ADDRESS: 004b8f30
// PROTOTYPE: bool __thiscall UploadIcon(long param_1, tagTime * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetControbuter
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2468
// RVA: 0x000B92F0
// ADDRESS: 004b92f0
// PROTOTYPE: void __thiscall SetControbuter(long param_1, long param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddOwnedCity
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:471
// RVA: 0x000B9DE0
// ADDRESS: 004b9de0
// PROTOTYPE: void __thiscall AddOwnedCity(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetMemberList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:520
// RVA: 0x000B9E60
// ADDRESS: 004b9e60
// PROTOTYPE: void __thiscall GetMemberList(list<long,std::allocator<long>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::DelMember
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:759
// RVA: 0x000B9EF0
// ADDRESS: 004b9ef0
// PROTOTYPE: bool __thiscall DelMember(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::RemoveApplyMember
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:775
// RVA: 0x000B9F50
// ADDRESS: 004b9f50
// PROTOTYPE: long __thiscall RemoveApplyMember(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdatePropertyToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1343
// RVA: 0x000B9FB0
// ADDRESS: 004b9fb0
// PROTOTYPE: void __thiscall UpdatePropertyToClient(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateEnemyFactionToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2659
// RVA: 0x000BA0F0
// ADDRESS: 004ba0f0
// PROTOTYPE: void __thiscall UpdateEnemyFactionToClient(long param_1, eOperator param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateCityWarEnemyFactionToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2681
// RVA: 0x000BA210
// ADDRESS: 004ba210
// PROTOTYPE: void __thiscall UpdateCityWarEnemyFactionToClient(long param_1, eOperator param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetParam
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2860
// RVA: 0x000BA310
// ADDRESS: 004ba310
// PROTOTYPE: void __thiscall SetParam(char * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddDefenceVictorCounts
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2902
// RVA: 0x000BA3B0
// ADDRESS: 004ba3b0
// PROTOTYPE: void __thiscall AddDefenceVictorCounts(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddOffenseVictorCounts
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2913
// RVA: 0x000BA3D0
// ADDRESS: 004ba3d0
// PROTOTYPE: void __thiscall AddOffenseVictorCounts(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddVillageWarVictorCounts
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2924
// RVA: 0x000BA3F0
// ADDRESS: 004ba3f0
// PROTOTYPE: void __thiscall AddVillageWarVictorCounts(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::ReInitialPropertyByLvl
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:205
// RVA: 0x000BA630
// ADDRESS: 004ba630
// PROTOTYPE: bool __thiscall ReInitialPropertyByLvl(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddOwnedCity
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:481
// RVA: 0x000BA650
// ADDRESS: 004ba650
// PROTOTYPE: void __thiscall AddOwnedCity(list<long,std::allocator<long>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetEnemyList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:539
// RVA: 0x000BA6A0
// ADDRESS: 004ba6a0
// PROTOTYPE: set<long,std::less<long>,std::allocator<long>_> __thiscall GetEnemyList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetCityWarEnemyList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:545
// RVA: 0x000BA6D0
// ADDRESS: 004ba6d0
// PROTOTYPE: set<long,std::less<long>,std::allocator<long>_> __thiscall GetCityWarEnemyList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsUsingPV
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:691
// RVA: 0x000BA700
// ADDRESS: 004ba700
// PROTOTYPE: bool __thiscall IsUsingPV(long param_1, ePurview param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetMemPV
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:705
// RVA: 0x000BA760
// ADDRESS: 004ba760
// PROTOTYPE: void __thiscall SetMemPV(long param_1, ePurview param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::Exit
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1535
// RVA: 0x000BAAB0
// ADDRESS: 004baab0
// PROTOTYPE: bool __thiscall Exit(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::FireOut
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1684
// RVA: 0x000BB140
// ADDRESS: 004bb140
// PROTOTYPE: bool __thiscall FireOut(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::DubAndSetJobLvl
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1759
// RVA: 0x000BB8A0
// ADDRESS: 004bb8a0
// PROTOTYPE: bool __thiscall DubAndSetJobLvl(long param_1, long param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::EndueRightToMember
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1835
// RVA: 0x000BBFB0
// ADDRESS: 004bbfb0
// PROTOTYPE: bool __thiscall EndueRightToMember(long param_1, long param_2, ePurview param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AbolishRightToMember
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1923
// RVA: 0x000BC500
// ADDRESS: 004bc500
// PROTOTYPE: bool __thiscall AbolishRightToMember(long param_1, long param_2, ePurview param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::LeaveWord
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2033
// RVA: 0x000BCA40
// ADDRESS: 004bca40
// PROTOTYPE: bool __thiscall LeaveWord(long param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, tagTime * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::Upgrade
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2288
// RVA: 0x000BCC60
// ADDRESS: 004bcc60
// PROTOTYPE: bool __thiscall Upgrade(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::LoadLeavewords
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2933
// RVA: 0x000BD3F0
// ADDRESS: 004bd3f0
// PROTOTYPE: void __thiscall LoadLeavewords(tagLeaveWord * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::~CFaction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:46
// RVA: 0x000BD590
// ADDRESS: 004bd590
// PROTOTYPE: void __thiscall ~CFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetID
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:161
// RVA: 0x000BD700
// ADDRESS: 004bd700
// PROTOTYPE: long __thiscall GetID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetName
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:162
// RVA: 0x000BD710
// ADDRESS: 004bd710
// PROTOTYPE: basic_string<char,std::char_traits<char>,std::allocator<char>_> * __thiscall GetName(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetMasterID
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:163
// RVA: 0x000BD720
// ADDRESS: 004bd720
// PROTOTYPE: long __thiscall GetMasterID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetLvl
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:164
// RVA: 0x000BD730
// ADDRESS: 004bd730
// PROTOTYPE: long __thiscall GetLvl(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetExp
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:166
// RVA: 0x000BD740
// ADDRESS: 004bd740
// PROTOTYPE: long __thiscall GetExp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetPermitDemise
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:168
// RVA: 0x000BD750
// ADDRESS: 004bd750
// PROTOTYPE: void __thiscall SetPermitDemise(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetCountry
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:171
// RVA: 0x000BD760
// ADDRESS: 004bd760
// PROTOTYPE: uchar __thiscall GetCountry(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetCountry
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:172
// RVA: 0x000BD770
// ADDRESS: 004bd770
// PROTOTYPE: void __thiscall SetCountry(uchar param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsSuperiorOrganizing
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:192
// RVA: 0x000BD780
// ADDRESS: 004bd780
// PROTOTYPE: long __thiscall IsSuperiorOrganizing(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetEstablishedTime
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:202
// RVA: 0x000BD790
// ADDRESS: 004bd790
// PROTOTYPE: tagTime * __thiscall GetEstablishedTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsLWFunction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:225
// RVA: 0x000BD7A0
// ADDRESS: 004bd7a0
// PROTOTYPE: bool __thiscall IsLWFunction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsCreateUnionFun
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:237
// RVA: 0x000BD7B0
// ADDRESS: 004bd7b0
// PROTOTYPE: bool __thiscall IsCreateUnionFun(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetOwnedCities
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:252
// RVA: 0x000BD7D0
// ADDRESS: 004bd7d0
// PROTOTYPE: list<long,std::allocator<long>_> * __thiscall GetOwnedCities(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetEneFacChanged
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:269
// RVA: 0x000BD7E0
// ADDRESS: 004bd7e0
// PROTOTYPE: void __thiscall SetEneFacChanged(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetCityEneFacChagned
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:270
// RVA: 0x000BD7F0
// ADDRESS: 004bd7f0
// PROTOTYPE: void __thiscall SetCityEneFacChagned(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetDefenceVictorCounts
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:288
// RVA: 0x000BD800
// ADDRESS: 004bd800
// PROTOTYPE: long __thiscall GetDefenceVictorCounts(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetOffenseVictorCounts
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:289
// RVA: 0x000BD810
// ADDRESS: 004bd810
// PROTOTYPE: long __thiscall GetOffenseVictorCounts(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetDelRemainTime
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:316
// RVA: 0x000BD820
// ADDRESS: 004bd820
// PROTOTYPE: long __thiscall GetDelRemainTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetTitleByID
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:207
// RVA: 0x000BD870
// ADDRESS: 004bd870
// PROTOTYPE: basic_string<char,std::char_traits<char>,std::allocator<char>_> __thiscall GetTitleByID(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetJobLvlByID
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:217
// RVA: 0x000BD910
// ADDRESS: 004bd910
// PROTOTYPE: ushort __thiscall GetJobLvlByID(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::Initial
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:66
// RVA: 0x000BD950
// ADDRESS: 004bd950
// PROTOTYPE: bool __thiscall Initial(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddOwnedCitiesToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:404
// RVA: 0x000BDD70
// ADDRESS: 004bdd70
// PROTOTYPE: bool __thiscall AddOwnedCitiesToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddEnemyOrganizing
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:558
// RVA: 0x000BDE80
// ADDRESS: 004bde80
// PROTOTYPE: void __thiscall AddEnemyOrganizing(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::DelEnemyOrganizing
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:576
// RVA: 0x000BE030
// ADDRESS: 004be030
// PROTOTYPE: void __thiscall DelEnemyOrganizing(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddCityWarEnemyOrganizing
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:601
// RVA: 0x000BE1E0
// ADDRESS: 004be1e0
// PROTOTYPE: void __thiscall AddCityWarEnemyOrganizing(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::DelCityWarEnemyOrganizing
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:619
// RVA: 0x000BE390
// ADDRESS: 004be390
// PROTOTYPE: void __thiscall DelCityWarEnemyOrganizing(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::ApplyForJoin
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:813
// RVA: 0x000BE520
// ADDRESS: 004be520
// PROTOTYPE: bool __thiscall ApplyForJoin(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateApplyMemberToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:891
// RVA: 0x000BEC80
// ADDRESS: 004bec80
// PROTOTYPE: void __thiscall UpdateApplyMemberToClient(long param_1, eOperator param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::DoJoin
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:947
// RVA: 0x000BEE40
// ADDRESS: 004bee40
// PROTOTYPE: bool __thiscall DoJoin(long param_1, long param_2, long param_3, tagTime * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::Disband
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:1597
// RVA: 0x000BFA00
// ADDRESS: 004bfa00
// PROTOTYPE: bool __thiscall Disband(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::Demise
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2099
// RVA: 0x000BFDE0
// ADDRESS: 004bfde0
// PROTOTYPE: bool __thiscall Demise(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::OperatorTax
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2253
// RVA: 0x000C0880
// ADDRESS: 004c0880
// PROTOTYPE: bool __thiscall OperatorTax(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::OperatorCityGate
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2268
// RVA: 0x000C0930
// ADDRESS: 004c0930
// PROTOTYPE: bool __thiscall OperatorCityGate(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetIsPermit
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2436
// RVA: 0x000C09A0
// ADDRESS: 004c09a0
// PROTOTYPE: void __thiscall SetIsPermit(long param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::OnMemberPosChange
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2598
// RVA: 0x000C0B40
// ADDRESS: 004c0b40
// PROTOTYPE: void __thiscall OnMemberPosChange(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::UpdateOwnedCityToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2702
// RVA: 0x000C0BF0
// ADDRESS: 004c0bf0
// PROTOTYPE: void __thiscall UpdateOwnedCityToClient(long param_1, eOperator param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::GetPlayerHeader
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:2762
// RVA: 0x000C0D10
// ADDRESS: 004c0d10
// PROTOTYPE: long __thiscall GetPlayerHeader(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::CFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:12
// RVA: 0x000C0D50
// ADDRESS: 004c0d50
// PROTOTYPE: undefined __thiscall CFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::CFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:38
// RVA: 0x000C0EE0
// ADDRESS: 004c0ee0
// PROTOTYPE: undefined __thiscall CFaction(long param_1, long param_2, tagTime * param_3, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:293
// RVA: 0x000C10B0
// ADDRESS: 004c10b0
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED выше: CFaction::CloneSaveData RVA 0x000C1210.

// ============================================================================
// FUNCTION: CFaction::SetOwnedCity
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:513
// RVA: 0x000C16A0
// ADDRESS: 004c16a0
// PROTOTYPE: void __thiscall SetOwnedCity(list<long,std::allocator<long>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::CheckOperValidate
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:749
// RVA: 0x000C17F0
// ADDRESS: 004c17f0
// PROTOTYPE: bool __thiscall CheckOperValidate(long param_1, ePurview param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsEnemyFaction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:464
// RVA: 0x000C1830
// ADDRESS: 004c1830
// PROTOTYPE: long __thiscall IsEnemyFaction(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::SetName
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:448
// RVA: 0x000C1B90
// ADDRESS: 004c1b90
// PROTOTYPE: bool __thiscall SetName(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::AbolishMemPV
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.cpp:716
// RVA: 0x000C1DD0
// ADDRESS: 004c1dd0
// PROTOTYPE: void __thiscall AbolishMemPV(long param_1, ePurview param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFaction::IsMaster
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\faction.h:178
// RVA: 0x000C1EE0
// ADDRESS: 004c1ee0
// PROTOTYPE: long __thiscall IsMaster(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




































// COMPONENT_VARIANT_END: WorldServer
