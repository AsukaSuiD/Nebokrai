//! Главный владелец исторического `WorldServer`.
//!
//! Статус `CGame::tagSetup::tagSetup` RVA `0x00004C30`, позиционной части
//! `CGame::LoadSetup` RVA `0x0000F890`, `CGame::InitNetServer` RVA
//! `0x00001860`, `CGame::InitNetClient` RVA `0x000030F0` и
//! `CGame::ReConnectLoginServer` RVA `0x00003280` и
//! `CGame::SendCdkeyToLoginServer` RVA `0x000083D0`,
//! `CGame::SendMsg2GameServer` RVA `0x00001B10`,
//! `CGame::SendGlobeVariableToGS` RVA `0x000017E0`,
//! `CGame::LoadServerSetup` RVA `0x00013850`, полный `CGame::Init` RVA
//! `0x00018EE0`, `CGame::SaveCityRegion` RVA `0x00008750`,
//! `CGame::ClearMapPlayer` RVA `0x0000D450`, полный `CGame::Release` RVA
//! `0x0000E7F0`, `CreateGame/DeleteGame/GetGame` RVA
//! `0x00015660/0x00001780/0x000017A0`, destructor `CGame` RVA
//! `0x00014EF0` и внешний `GameThreadFunc` RVA `0x0001A310`,
//! `CGame::GetConnectedGameServerCount` RVA `0x000084C0`,
//! `CGame::GetConnectedGameServerCountEx` RVA `0x00008520`,
//! обе перегрузки `CGame::GetGameServer` RVA `0x00008590/0x000132A0`,
//! `CGame::IsConnect` RVA `0x000A5600`,
//! `CGame::ClearOfflinePlayer` RVA `0x00004BF0`,
//! `CGame::ClearMapPlayerForOffline` RVA `0x0000D3A0`,
//! `CGame::GetMapPlayer` RVA `0x00007100`,
//! `CGame::RemoveOnlinePlayer` RVA `0x00007190`,
//! `CGame::GetOnlinePlayerByID` RVA `0x00007210`,
//! `CGame::GetOnlinePlayerIDByName` RVA `0x00005590`,
//! `CGame::GetLoginPlayerIDByName` RVA `0x00007270`,
//! `CGame::GetFactionById` RVA `0x00001FD0`,
//! `CGame::ToStrlwr` RVA `0x00002000`,
//! `CGame::IsNameExistInMapPlayer` RVA `0x00005190`,
//! `CGame::ClearCreationPlayer` RVA `0x00004B30`,
//! `CGame::GetCreationPlayerCountInCdkey` RVA `0x000052D0`,
//! `CGame::GetCreationPlayerByName` RVA `0x00005390`,
//! `CGame::RemoveOfflinePlayer` RVA `0x00007330`,
//! `CGame::RemoveLoginPlayer` RVA `0x00007340`,
//! `CGame::GetLoginPlayerByID` RVA `0x00007390`,
//! `CGame::ValidateDBPlayerIDinCdkey` RVA `0x000073F0`,
//! `CGame::ValidatePlayerIDinCdkey` RVA `0x00007490`,
//! `CGame::GetTeamSessionID` RVA `0x000070C0`,
//! `CGame::AppendOnlinePlayer` RVA `0x00010D50`,
//! `CGame::AppendOfflinePlayer` RVA `0x00010DF0`,
//! `CGame::AppendLoginPlayer` RVA `0x00010E50`,
//! numeric `CGame::GetRegion(long)` RVA `0x00011F20`,
//! name-overload `CGame::GetRegion(char const*)` RVA `0x00008680`,
//! достигнутой связи `CGame::tagRegion::pRegion` с `CWorldRegion` и его
//! унаследованным именем,
//! `CGame::GetRegionGameServer` RVA `0x00013AC0`,
//! `CGame::GetPlayerGameServer` RVA `0x00013B40`,
//! `CGame::GetGameServerNumber_ByRegionID` RVA `0x00013B70`,
//! `CGame::GetGameServerNumber_ByPlayerID` RVA `0x00013B90`, а также
//! `ShowSaveInfo` RVA `0x00001720`,
//! `CGame::CheckPoint` RVA `0x00009A90` и
//! `CGame::ProcessMessage` RVA `0x00001A30`, player-часть
//! `CGame::tagDBData::tagDBData` RVA `0x00011F60`,
//! `CGame::AppendDBPlayer` RVA `0x0000ED90` и
//! `CGame::AppendDBCreationPlayer` RVA `0x00010FB0`,
//! `CGame::AppendSaveFaction/AppendSaveUnion` RVA `0x000110D0/0x00011130`,
//! `CGame::AppendDelFaction/AppendDelUnion` RVA `0x00011190/0x000111F0`,
//! `CGame::AppendRegionParam` RVA `0x00011250`,
//! `CGame::AddGoodsLink/FindGoodsLink` RVA `0x000112B0/0x00005A10`,
//! `CGame::AddOrginGoodsToPlayer` RVA `0x00005A40`,
//! `CGame::AppendDBCountry` RVA `0x00011070`,
//! `CGame::SetEnemyFactions` RVA `0x00014D90` и
//! `CGame::ClearDBData` RVA `0x0000D490`, live restore/deletion list-owner-ы
//! `0x00004B70/0x00004BB0/0x000054E0/0x00005530/0x00005560/0x00007140/
//! 0x00010CA0/0x00010CF0`, `CGame::AppendMapPlayer` RVA `0x00010A40`,
//! `CGame::CloneMapPlayer` RVA `0x00010AB0`, `CGame::CloneSavingPlayer` RVA
//! `0x00010EB0`, `CGame::GetCreationPlayerVectorByCdkey` RVA `0x00012760` и
//! полный `CGame::GenerateDBData` RVA `0x00012E50`,
//! `CGame::GeterateRegionDBData` RVA `0x00012860`,
//! `CGame::RefreshOwnedCityOrg` RVA `0x000128E0`,
//! `CGame::CheckInvalidString` RVA `0x00001B30`,
//! `SaveThreadFunc` RVA `0x00001E30`, полный `CGame::AI` RVA `0x000148A0`,
//! полный `CGame::ProcessPlayerDataQueue` RVA `0x00013BD0`,
//! worker `LoadPlayerDataFromDB` RVA `0x000092C0`,
//! `CGame::ProcessTimeOutLoginPlayer` RVA `0x00014A60` и весь достигнутый
//! `CGame::MainLoop` RVA `0x00019A00` — `IMPLEMENTED`;
//! остальной корпус ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.h:116,174,377,401,423,609,655`
//! и `e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:141,495,502,513,520,563,587,786,2072,2215,2284,2302,2346,2446,2790,2976,3011,3021,3045,3083,3105,3161,3299,3322,3357,3372,3393,3498,3514,3577,3590,3620,3678,3786,4194,4208,4285,4410,4464,4555,4565,4574,4583,4789,4822,5532,5967`.
//!
//! Конструктор `tagSetup` создаёт 23 пустые `std::string` и задаёт только
//! `dwRefeashInfoTime = 1000`, `dwSaveInfoTime = 60000`, `bUseLogSys = false`
//! и `bUseOldSaveLargessWay = true`. Затем `CGame::CGame` RVA `0x00015210`
//! меняет имя на `WorldServer`, Login IP на `127.0.0.1`, порты на `2345/8100`
//! и позднее создаёт пустой `m_vectorPingGameServerInfo`, ставит
//! `_login_server_id = 0`, `m_bInPing = false` и снимает отдельный начальный
//! ping-tick. Остальные достигнутые
//! numeric/bool члены исходно не инициализированы; `Option` сохраняет эту
//! границу и не назначает ей выдуманный ноль.
//!
//! `AddOrginGoodsToPlayer` сохраняет list-order, occupation filter, original-
//! name lookup, factory roll-order, GUID-before-positional-add и продолжение
//! после rejected equipment. `uuid`/`getrandom` заменяют только `CoCreateGuid`,
//! а системный отказ остаётся typed block вместо скрытого нулевого GUID.
//!
//! Исключение сделано только для `m_GlobeVariable` по итогам точной проверки
//! всего EXE. PDB задаёт четыре signed `long` по `CGame+0x04..+0x13`, точный
//! конструктор `0x00415210` после vtable сразу начинает `m_mPlayer` с `+0x14`,
//! а полный проход декомпиляции и ссылок не нашёл записей в эти 16 байт.
//! Единственный настоящий потребитель, `SendGlobeVariableToGS` `0x004017E0`,
//! копирует их как сырой payload `0x7F80D`. Старый сервер тем самым выдавал
//! наружу недетерминированную память, а не устойчивую Miracle-семантику. Rust
//! исправляет эту ошибку работы с памятью четырьмя нулями; явное little-endian кодирование
//! сохраняет доказанные размер, порядок полей и wire framing без `unsafe`.
//!
//! `GetFactionById` не использует `this`: он возвращает null при ID `0` либо
//! отсутствующем singleton-е, иначе делегирует signed ID точному
//! `COrganizingCtrl::GetpFactionById`. Rust передаёт controller явно вместо
//! global singleton и возвращает `Option<&CFaction>` вместо nullable pointer;
//! нулевой ID по-прежнему отсекается до map lookup. Raw owner после реализации
//! удалён.
//!
//! `RefreshOwnedCityOrg` сначала проверяет обе ступени `GetRegion` и только для
//! живого owner-а получает country через exact `GetCountryByFaction`. Затем он
//! вызывает virtual `SetOwnedCityOrg`, присваивает унаследованный country byte и
//! уже после обеих мутаций рассылает `0x7FE27 + region/faction/union/country`.
//! Точная дизассемблировка `0x004128E0..0x004129E8` подтверждает, что Windows
//! owner не сохраняет регион и не использует отдельный governance wire-кодек:
//! эти добавления старого Linux-донора не перенесены. `BTreeMap`, typed region
//! owner и готовый `CMessage::send_all` заменяют только STL, virtual ABI и
//! сетевую инфраструктуру; ignored send-result остаётся наблюдаемым отчётом.
//!
//! Name-overload `GetRegion` обходит `s_mapRegionList` в signed key-order и
//! возвращает первый `tagRegion`, чей `pRegion->m_strName` равен входной
//! C-строке через case-sensitive `strcmp`. Точная дизассемблировка
//! `0x00408680..0x00408741` не содержит map lookup по имени или fallback-а.
//! `named_region_lookup` сохраняет порядок и first-match, а также возвращает
//! достигнутый route snapshot вместо сырого указателя. Null `pRegion` в EXE
//! разыменовывался; Rust считает его повреждённым внутренним состоянием,
//! пропускает и явно считает, не приписывая падению игровую семантику.
//!
//! `CheckInvalidString` остаётся точным однострочным делегатом, но process-global
//! `CWordsFilter` теперь является owned полем единственного `CGame`. `Init`
//! читает оба ресурса на прежней позиции и игнорирует bool `Initial`, а Release
//! очищает owner на позиции исходного singleton delete. Это устраняет global
//! lifetime, не меняя порядок загрузки, фильтрации или teardown. Смена имени
//! вызывает двухаргументный overload, создание роли — трёхаргументный с exact
//! all-numbers gate; initial-config проверяет `IsValid` и кодирует subtype
//! `0x31` из того же owner-а. Прежние внешние callback/snapshot-ы этих путей
//! удалены.
//! Reload-профиль `InvalidStr` по exact
//! `0x00416C75..0x00416CA8` вызывает тот же owned `ReloadFilter`: failure
//! молча идёт в общий epilogue, success пишет только `Load InvalidStr...OK!`,
//! а общий legacy return-slot остаётся нулевым. Resource-context заменяет
//! `fopen`, но чтение charcode начинается только после доступного word-list.
//!
//! `EquipmentComposeList` также принадлежит единственному `CGame`: reload и
//! initial-config serializer читают одну пару ordered map. Exact caller
//! `0x00417AF6..0x00417B7F` передаёт `data/EquipmentCompose.ini`, сохраняет
//! return `LoadList`, пишет прежний log и только при success + send-флаге
//! публикует `0x7F801/0x30`. Прежние boolean/serialization callback-owner-ы
//! для этой ветви удалены; resource backend и `CMessage` остаются общими
//! техническими границами.
//!
//! `CCiQingSetup` теперь также принадлежит `CGame`. Reload `ciqing` по exact
//! `0x00417D96..0x00417E63` читает `/data/ciqing.ini`, пишет прежний success/
//! failure log, сохраняет bool в общий legacy return-slot и затем безусловно
//! перезагружает LingBao; только успешный CiQing при включённой рассылке
//! кодирует owned CiQing перед оставшимся LingBao-owner-ом в subtype `0x35`.
//! Initial-config читает тот же owned экземпляр,
//! поэтому внешний CiQing snapshot и два прежних reload callback-а удалены.
//! `Vec` и resource-context заменяют только STL/file backend; original-name
//! lookup остаётся явной границей уже загруженного World `CGoodsFactory`.
//!
//! `CTaoZhuangSetup` следует той же owned-модели: exact dispatcher
//! `0x00417CE6..0x00417D91` загружает `data/taozhuang.ini`, сохраняет bool в
//! legacy return-slot, пишет прежний log и публикует subtype `0x34` только при
//! success + send-флаге. Loader сохраняет first-wins `std::map`/`std::set`
//! порядок и exact duplicate-failure logs; дубли ID самих комплектов не
//! блокируют reload и оставляют первый item, как World EXE. Initial-config
//! кодирует этот же owned owner, без внешнего TaoZhuang snapshot/callback.
//!
//! `CHitLevelSetup` теперь тоже принадлежит `CGame`: exact dispatcher
//! `0x00416768..0x00416850` очищает/читает `data/hitlevel.ini`, сохраняет
//! именно bool load-result до и после optional subtype `0x14`, а не размер
//! payload. Missing file сохраняет исходный operator notice `ERROR`; initial-
//! config использует этот же owner. Прежние HitLevel boolean/serialization
//! callbacks и внешний snapshot удалены.
//!
//! `CIncrementShopList` также принадлежит `CGame`: dispatcher читает
//! `setup/incrementshoplist.ini`, сохраняет bool load-result в legacy
//! return-slot и при success + send-флаге публикует subtype `4`. Loader
//! последовательно резолвит original/display names через тот же `CGoodsFactory`,
//! сохраняет частичное state при ошибке разбора и отдельно загружает affiche.
//! Initial-config и Release используют тот же owner; прежние IncrementShop
//! boolean/serialization/release callbacks и внешний snapshot удалены.
//!
//! `PrisonConf` owned `CGame`: exact loader `data/PrisonConf.ini` очищает map
//! до открытия, но сохраняет PK scalar при missing resource; dispatcher
//! сохраняет bool load-result до optional subtype `0x1D`. Initial-config
//! читает этот же owner, без внешних Prison callbacks/snapshot.
//!
//! `CTradeList` owned `CGame`: `data/tradelist.ini` сначала очищает map, затем
//! в exact порядке получает NPC text из owned StringTable и goods ID из
//! `CGoodsFactory`. Reload сохраняет прежний return mapping: только успешная
//! рассылка subtype `3` заменяет legacy return размером payload; initial-config
//! читает тот же owner без внешнего TradeList snapshot/callback.
//!
//! `CEmotion` owned `CGame`: `data/Emotions.ini` не очищает static map и
//! возвращает false только при missing resource. Reload сохраняет исходный
//! порядок после PlayerList и legacy return length от optional subtype `0x15`;
//! initial-config читает тот же owner без внешнего snapshot/callback.
//!
//! `CBattleFairyExpConfig` остаётся отдельным World owner: reload читает
//! `BattleFairyReleate/BattleFairyExp.xml`, очищает map до открытия, при
//! ошибке берёт exact `ZHGS0029..0036` из owned StringTable и лишь после
//! успешного XML loader-а публикует subtype `0x2C`. Legacy return получает
//! длину wire только при отправке. `quick-xml` заменяет TinyXML внутри owner-а,
//! а CGame сохраняет наблюдаемые путь, диагностики, log-order и wire boundary.
//!
//! `CBattleFairyProperty` аналогично остаётся внешним runtime owner-ом:
//! `BattleFairyCombineConfig` читает `BattleFairyReleate/CombineConfig.ini`,
//! не меняет старый compose-vector при missing resource и после успешного
//! чтения публикует его exact `0x2D` payload. Legacy return — размер payload
//! только при send. Так CGame устраняет generic callback, но не копирует
//! owner-state и сохраняет observed resource/lifecycle контракт.
//!
//! `CSynthesis` — внешний static owner двух разных контейнеров: reload
//! `data/synthesis.xml` очищает лишь recipes до resource-open, выполняет
//! original-name/display-name lookup через тот же `CGoodsFactory`, а broadcast
//! map сохраняет даже при последующем XML failure. После успеха тот же owner
//! кодирует subtype `0x21`; generic callback и отдельная serialization-копия
//! удалены. `quick-xml` и safe temporary extraction owner-а заменяют только
//! TinyXML и aliasing, не меняя порядок state transitions или lookup-ов.
//!
//! `HonorElimilateConfig` также освобождён от generic callback: reload читает
//! `data/honorelimilate.ini` в единственные два runtime scalar-а, при missing
//! resource сохраняет старые значения и выдаёт exact operator notice, а при
//! доступном, но повреждённом тексте сохраняет legacy success/partial-write.
//! Dispatcher не отправляет его eight-byte wire и не меняет legacy return.
//!
//! `CContributeSetup` теперь owned `CGame`: dispatcher
//! `0x004171CA..0x004172B1` читает `data/ContributeSetup.ini`, сохраняет bool
//! load-result в legacy return-slot и при success + send-флаге публикует
//! subtype `5`. Missing resource показывает `ERROR`, очищает только item-
//! vector, но оставляет 11 scalar-параметров прежними, как exact loader.
//! Initial-config читает тот же owner; внешние Contribute callbacks/snapshot
//! удалены.
//!
//! `CDupliRegionSetup` создаётся и публикуется в `CGame` перед своим `Load`,
//! как exact Init `0x0041901B..0x00419053`, и остаётся owned даже при
//! load-failure до общего Release. Init читает точный
//! `setup/DupliRegionsSetup.ini`; reconnect и create-role используют тот же
//! owner без внешней параллельной копии. `Option` сохраняет исходный nullable
//! lifecycle до позиции создания и после позиции удаления.
//!
//! `LoadSetup` сначала пробует обычный `setup.ini`, а только при ошибке
//! открытия — декодированный `setup.dat`. Поток читает пары `label + value`,
//! label не проверяет, строки хранит byte-exact и при EOF/fail-state оставляет
//! уже выполненные мутации и прежние значения хвоста. DAT — отдельный старый
//! формат: после `bUseLogSys` он не назначает `strLogSysProvider`, читает четыре
//! оставшихся LogSys-строки, пять CostDB-строк и затем повторно назначает
//! `strName` перед `dwLoadLargessTime`; этот порядок не выравнивается с INI.
//! После чтения plain-ветвь закрепляет single-instance title
//! `WorldServer[<name>]-Saga3D2`, encoded-ветвь — `WorldServer[<name>]`.
//! Linux owner передаётся callback-ом вместо `FindWindowA/SetWindowTextA`;
//! занятый title сохраняет исходный операторский `ERROR` и false-ветвь.
//! `IniDecoder` заменён уже доказанным `public::tools::ini_decode`, а owned
//! `Vec<u8>` и `fs::read` заменяют `new[]/fread/stringstream` без Windows ABI.
//!
//! Полный `Init` сохраняет исходный порядок crash/random/setup/language/DB,
//! reload и subsystem owners, region relation, log/network/queue и worker-start
//! эффектов. Готовые `LoadSetup`, `LoadServerSetup`, `InitNetClient`,
//! `InitNetServer`, `CRsSetup` ID-публикация и `CPlayerDataQueue::Clear`
//! исполняются непосредственно. `COrganizingParam::Initialize` напрямую читает
//! позиционный `data/FactionParam.ini` и ставит tax event; затем
//! `CPlayerRanks::Initialize` получает его загруженные rank-поля, ставит своё
//! calendar event, после чего начальный
//! `StatPlayerRanks` напрямую читает DB без публикации — точный caller на
//! `0x004197C1/0x004197CD` не проверяет bool первого вызова, а второй является
//! void. Ещё сырые соседние owners переданы одним
//! ordered context-контрактом и не объявлены реализованными. Первый доказанный
//! false-result прекращает дальнейшие эффекты, а старые `MessageBoxA` и
//! `__beginthreadex` представлены operator/worker boundaries. Async-форма DB и
//! connect-owner-ов сохраняет их старые синхронные caller-позиции. Повторная
//! запись control-send после `InitNetClient` схлопнута с уже выполненной
//! идемпотентной публикацией того же значения.
//!
//! `Release` сохраняет полный порядок: queue/city-save, остановка network
//! workers, шесть live lists, player/DB data, save-worker join, goods/region/
//! script owners, subsystem/network/DB teardown, cache и runtime cleanup,
//! write/player-load workers, Largess/ADO и последние resource owners. Exact
//! EXE содержит единственный `ret` в `0x0040ED87`; псевдокодовые ранние выходы
//! после STL `operator_delete` являются ошибкой декомпилятора и не перенесены.
//! `CDbMisc` в Release доказанно не удалялся и отмечен retained, а не потерян
//! внутри общего DB списка. `VecDeque/BTreeMap/Box/Drop` заменяют только
//! container nodes, deleting destructors и allocator cleanup. Для
//! `SaveCityRegion(0)` остаются две локальные safe-границы: неназначенный
//! `REGION_TYPE` и исходное разыменование null city-region pointer.
//!
//! Внешний `GameThreadFunc` теперь владеет nullable game-slot: создаёт `CGame`,
//! вызывает готовые Init/MainLoop через typed runtime adapter, ждёт save barrier
//! только после successful Init, всегда выполняет Release перед DeleteGame на
//! штатной ветви и публикует exit-event до window-close request. Linux future,
//! cooperative worker adapters и Rust Drop заменяют Win32 thread/event/window
//! mechanics без Windows FFI. Fatal `_exit(1)` из Init и typed safe-block-и
//! возвращают live `Box<CGame>` и потому не получают выдуманный cleanup.
//!
//! `InitNetServer` создаёт точный World `CMyNetServer`, публикует owner до
//! результата `Host`, а при ошибке сохраняет его у `CGame`. После успешного
//! listen первый IPv4 локального hostname заменяет унаследованные адресные
//! поля, затем восемь setup-полей записываются в исходном порядке offset-ов.
//! `rustix::system::uname` и стандартный `ToSocketAddrs` заменяют
//! `gethostname/gethostbyname`, `clock_gettime(CLOCK_BOOTTIME)` — wrapping
//! `timeGetTime` конструктора общего сетевого owner-а.
//!
//! `dwMaxMsgLen` подтверждён как `CServer+0x118`, а `bCheckMsgCon` — как
//! `+0x10D` по соседнему consumer `nets/netserver/CMyServerClient::OnReceive`.
//! Точный World `networld` parser эти поля не читает, поэтому Rust сохраняет
//! позднее состояние, но не переносит из другого компонента его ban/QUIT и
//! условные CRC-ветви. Успех метода соответствует старому ненулевому результату,
//! а `Host`-ошибка — нулю; это подтверждает непосредственный вызов из `Init`.
//!
//! `InitNetClient` сначала уничтожает прежний World-to-Login owner, публикует
//! новый до создания socket и bind-ит `0.0.0.0:0`. `CreateSocketThread` заменён
//! awaitable readiness самого `CMyNetClient`. Общий десятисекундный connect
//! получает первый IPv4 из исходной C-строки `strLoginIP`; после успеха
//! включается control-send и ставится `0x1FE01 + dwNumber + strName\0` без
//! приоритета. Результат старого `Send` игнорировался и поэтому сохраняется в
//! typed-отчёте, не меняя legacy success `1`. Ошибка bind/resolution/connect
//! выполняет исходный close/delete и снова оставляет nullable owner.
//!
//! World-вариант общего `CClient::Connect` RVA `0x000293E0` копировал вход в
//! `char[64]`, предварительно разрешал non-digit имя через `gethostbyname`, а
//! `ConnectServer` RVA `0x00028E20` применял `inet_addr` с тем же DNS fallback.
//! `ToSocketAddrs` заменяет эти системные вызовы и выбирает первый IPv4.
//! Найденный baseline Login IP — короткий ASCII strict dotted IPv4; его значение
//! не публикуется. Длина свыше 63 bytes и non-UTF8 hostname остаются локальными
//! safe-границами старого переполнения/ANSI resolver, а не получают `unsafe`.
//!
//! `ReConnectLoginServer` создаёт такой же новый client локально и не меняет
//! текущий `s_pNetClient`. Точный PDB задаёт virtual slot как
//! `CClient::Create(unsigned int, char const*, int, long, bool)`, а World-тело
//! RVA `0x00029210` подтверждает `(0, nullptr, 1, 0x33, true)`: TCP socket,
//! bind `0.0.0.0:0` и WinSock event mask. Tokio readiness и тот же
//! `bind_tcp_ipv4` сохраняют технический эффект без Windows runtime.
//!
//! После connect старый код публиковал `0x3FC03 + CMyNetClient*` прямо в FIFO
//! World `CMyNetServer`. Rust передаёт той же FIFO владение typed event без
//! pointer-to-integer. `ProcessMessage` фиксирует один размер server FIFO,
//! исполняет handoff на его позиции, затем заново фиксирует FIFO уже текущего
//! Login client. `OnServerMessage` RVA `0x000ADCF0` сначала закрывает прежний
//! owner, ставит CD-key snapshot и приоритетный `0x1FE01`, а control-send
//! включает только после попытки отправки. Поэтому reconnect не включает его
//! заранее.
//!
//! Оба цикла `ProcessMessage` уменьшают только исходный signed snapshot даже
//! при пустом `PopMessage`; новые элементы остаются следующему проходу. Вторая
//! очередь читается через текущий client на каждой итерации. `Drop` заменяет
//! virtual deleting destructor, а два wrapping millisecond аккумулятора
//! сохраняют `g_lGSMessageTime/g_lLSMessageTime`. Обычный `CMessage::Run`
//! выполняет точный numeric selector; готовые ветви server-owner
//! `0x4FC01..=0x4FC03`, other honor, достигнутые organizing owner-ы вплоть до
//! billboard `0x60125`, faction upgrade `0x60126`, upload-icon gate `0x60127`
//! и contributor gate `0x60128`, faction-experience `0x60129`, а также
//! member level/position callback `0x6012A`, city-tax gates `0x6012B/0x6012C`
//! и region-param broadcast/route `0x6012D/0x6012E`, city-gate `0x6012F`
//! исполняются сразу. Остальные сообщения возвращаются owned вместе с
//! выбранным сырым owner-ом и не выдаются за no-op исполнение. Session manager
//! передаётся тому же
//! `ProcessMessage` явно вместо
//! process-global singleton-а; cookie/result и terminal removal остаются у
//! уже восстановленного manager-owner-а.
//! Внешний MainLoop call-site теперь также готов: отдельный tick снимается до
//! `ProcessMessage`, следующий после успешного возврата даёт wrapping elapsed
//! для `DAT_0056e514`, а третий становится start tick следующей SessionFactory-
//! стадии и назначается общему clock-state. Safe block сохраняет только уже
//! снятый первый tick и не назначает исходно недостигнутые
//! accumulator/end/next-stage эффекты.
//! Самостоятельный 600-секундный profiling gate также материализован без
//! вызова сырых producers. Он использует wrapping `now - last` и строгое
//! `> 600000`, снимает двенадцать 32-битных bit-pattern-ов, интерпретирует их
//! как signed `%d` в точной многострочной строке и вызывает один `AddLogText`.
//! Только после него счётчики обнуляются в исходном порядке, а last tick
//! получает текущий MainLoop tick. Prerequisite initial tick теперь также
//! готов: отдельный `WorldMainLoopInitializationState` хранит точный общий
//! `DAT_0056e550`. Bit `1` и следующий bit `2` устанавливаются отдельными
//! owner-ами с сохранением всех остальных bits до своих clock-call; результаты
//! назначаются соответственно last-report и last-save state. Bit `4` clock не
//! вызывает: он копирует прежний общий current tick в initial refresh tick.
//! Следующий MainLoop `timeGetTime` теперь также материализован: ровно один
//! clock-call заменяет общий current tick и возвращает то же значение caller-у.
//! Следующий Largess gate читает setup до остальных эффектов, wrapping
//! увеличивает общий pass-counter и использует short-circuit
//! `interval != 0 && interval < current - last`. Достигнутый gate сначала
//! обновляет last tick и возвращает typed `StartWorkerRequested`: отдельный
//! сырой `CLargess::StartWorkerThread` не выдаётся за созданный поток. Полный
//! MainLoop на этой позиции вызывает переданную границу этого owner-а. Более
//! поздний profiling-start также снимает ровно один tick в общий scratch,
//! который следующие стадии исходно переиспользуют. Цельный следующий участок
//! теперь также готов: strict refresh gate сначала назначает last-refresh tick,
//! вызывает фактический `RefeashInfoText`, снимает end tick, wrapping добавляет
//! `DAT_0056e530` и сразу выполняет готовый 600-секундный profiling gate.
//! Отсутствующий network owner сохраняет исходный no-op только внутри refresh,
//! после чего profiling-порядок продолжается. Write-log count теперь читается
//! из concrete `CGame` FIFO; Team/Largess/load/reback counts принадлежат
//! соседним owners и передаются явно. Непредставимый старым `uint` размер
//! блокирует только refresh после
//! уже выполненного last-tick присваивания.
//!
//! `CGame::ReLoad` RVA `0x00015740`, `reload_conf_log` RVA `0x00002FD0` и
//! `reload_profiles` RVA `0x00018110` имеют статус `IMPLEMENTED`. Внутренний
//! dispatcher сохраняет 47 case-insensitive сравнений, точные load/log/
//! serialize/send ветви, включая безусловные AttackCity/VillageWar/CityWar/
//! Quest send, no-op `GeneralVariableList` и исходный length-accumulator.
//! Exact EXE подтверждает единственный epilogue `ReLoad`; startup call-sites
//! `0x004193D8..0x00419694` передают оба boolean как `false`. Соседние ещё
//! сырые configuration owners представлены раздельными typed вызовами, а не
//! одним непрозрачным reload callback.
//! `Allthing` больше не является таким callback-ом: exact caller
//! `0x00417F12..0x00417F45` передаёт byte-path `/data/LeitingAction.ini`,
//! проверяет явный `0/1` loader-а и только после успеха допускает subtype
//! `0x36`. Конкретный `CThingSetup` живёт у единственного `CGame`, поэтому
//! startup/reload, initial-config serializer и daily LeiTing читают один
//! owner; `VecDeque`, resource-context и safe codec заменяют лишь static STL,
//! `CRFile` и безразмерный byte buffer.
//! CountryWar-ветвь внешнего main-loop dispatcher-а теперь передаёт прямо
//! живые `CountryWarSys`, `CTimer`, девять callback-ключей, local time,
//! resource-context и текущий GameServer sender. Старый
//! `WorldReloadBooleanOwner::CountryWar` удалён. Обычный `CGame::ReLoad` без
//! этих owners возвращает явный `CountryWarOwnerRequired`, а единственный
//! достигнутый flag-dispatcher вызывает concrete overload в той же позиции;
//! reload-server-resources, внутренние логи, `end_war`, повторная загрузка и
//! итоговый `0/1` сохраняют исходный порядок. Exact
//! `0x004176D5..0x0041771D` кладёт zero-extended bool reload-owner-а в общий
//! return slot `CGame::ReLoad`, что исправляет прежний потерянный Rust-result.
//! Результат `SendAll(0x7FF1D)`
//! старый код игнорировал; Rust хранит его как `Result` только в typed report,
//! не назначая искусственный legacy error code.
//!
//! Внешний flag-dispatcher сохраняет все 42 проверки в
//! исходном порядке, отдельные 32-битные low/high чтения и записи, снятие флага
//! до вызова reload-owner-а и странные поздние повторные/перекрывающиеся маски.
//! В частности, большинство low-веток после своей low-записи отдельно обнуляют
//! всю high-половину, а `ChangeBodyConf`, `SynthesisList` и поздний `Allthing`
//! этого не делают. Две `AtomicU32` с relaxed load/store заменяют исходные
//! несинхронизированные x86 word-accesses без `unsafe`: это намеренно
//! не atomic read-modify-write, поэтому совместный producer всё ещё может
//! потерять конкурентную запись между load и store. Готовые `Init/MainLoop`
//! теперь вызывают этот же `CGame` owner напрямую.
//!
//! Script-цепочка `GetScriptFileData/LoadOneScript/ReLoadOneScript/
//! LoadScriptFileData` RVA `0x000132E0/0x00013440/0x00013760/0x00014450`
//! хранит byte-exact buffers в `BTreeMap`, нормализует только доказанный
//! ведущий `\\` и `\\ -> /`, очищает три owner-а до reload и публикует
//! `0x7F801` subcodes `0x0A/0x0B/0x0D` в string-key order. Доступ к overlay и
//! порядок перечисления `.script` остаются у resource-context: plain fs не
//! назначен каноническим поверх loose/data/patch01.
//!
//! `reload_conf_log` игнорирует переданный reload-result, дважды снимает local
//! time для последовательных `_strdate`/`_strtime`, строит byte-exact
//! `0x1FE06 + local IPv4 word + world number + C-string` и посылает его
//! LoginServer с `prioritized=false`. Nullable/empty profile остаётся no-op;
//! отсутствующие обязательные server/setup owners и переполнение старого
//! `char[128]` являются локальными `BLOCKED_MISSING_FACT`. MSVC SEH/security-
//! cookie bookkeeping удалён как compiler noise без Linux-наблюдаемости.
//!
//! Следующий MainLoop maintenance-сегмент от `g_bStatPlayerRanks` до
//! collect-player-data также `IMPLEMENTED`. Он снимает rank-request до двух
//! отдельных singleton-вызовов, сохраняет Appellation guard, nullable
//! HonorRanks instance, strict day mismatch, точный tick/log/OnNewDay/log
//! порядок и затем безусловно достигает AuctionBang daily gate. Сырые
//! PlayerRanks DB-чтение, `CHonorRanks` и `CAuctionLog` исполняются напрямую.
//! PlayerRanks сохраняет clear/log/tick/stream-prefix/log/send порядок и
//! продолжает публикацию даже после DB `false`, как исходный void-wrapper.
//! Тот же stat/send helper используется calendar `OnStatRanks`, но именно
//! timer-owner после него снимает отдельное local time и ставит следующий день.
//! Суточная
//! AuctionBang-ветвь передаёт реальный Log DB connection, присваивает день до
//! update и сохраняет его неатомарный outcome. Неинициализированный исходным
//! constructor-ом `m_lAucOldDay` остаётся typed `BLOCKED_MISSING_FACT`, а не
//! получает придуманное стартовое значение. Отдельный init-проход напрямую
//! вызывает потоковый `CAuctionLog::LoadItem`, сохраняет partial publication и
//! исходно продолжает инициализацию после `false`, меняя только текст лога.
//! `AtomicBool` использует отдельные relaxed load/store, а не `swap`, сохраняя
//! исходную границу между проверкой producer-флага и его очисткой.
//!
//! `CGame::AI` сначала обходит `s_mapRegionList` в signed-key порядке и зовёт
//! virtual region `AI` только у ненулевого `tagRegion::pRegion`. Exact vtable
//! slot всех поставочных subtype-ов ведёт в общий `0x00401000: ret`, поэтому
//! `WorldRegionOwner::ai` сохраняет concrete dispatch как доказанный no-op без
//! внешнего callback-а и object slicing. Затем ровно один
//! `timeGetTime` задаёт секунды для всего ordered `m_listBroadcast`. Cadence
//! использует wrapping `now - last` и строгое `interval < elapsed`;
//! `random(100)` вызывается только после gate, а interval-random — только после
//! выбранной рассылки и попытки send. Сообщение `0x7FA03` содержит signed
//! region/import, два color DWORD и byte-exact C-строку именно в этом порядке.
//! Нулевой region идёт в `SendAll`, ненулевой — в
//! `GetRegionGameServer -> SendToMapID`; при отсутствующей записи send
//! пропускается, но last/interval всё равно меняются. `BTreeMap`, `VecDeque`,
//! owned `Vec<u8>` и готовый `CMessage` заменяют только MSVC tree/list/string,
//! allocation и SEH. Достигнутая AI-проекция `tagSysBroadcast` сохраняет поля
//! struct offsets `+0x04..+0x40`; нечитавшийся `_login_type +0x00` не получает
//! выдуманного значения, и Rust ABI не объявлен копией старых 68 bytes.
//!
//! Окружающий MainLoop call-site сначала отдельным tick закрывает время
//! SavePoint в `DAT_0056e52c`, wrapping увеличивает `s_lAITick`, затем новым
//! tick назначает `_DAT_0056e534`, вызывает полный AI и последним tick добавляет
//! `DAT_0056e510`. Эти bit-pattern-ы хранятся соответственно в уже существующих
//! `save_point_time_ms`, `ai_calls` и `ai_time_ms`; следующим остаётся готовый
//! `process_message_main_loop_stage`. Последующие готовые stage-owner-ы
//! сохраняют все accumulator/tick позиции вплоть до финального pacing-хвоста.
//!
//! Следующий `CSessionFactory::AI` теперь также связан с profiling-порядком:
//! он использует start tick, назначенный успешным `ProcessMessage`, после
//! полного factory traversal одним tick wrapping добавляет `DAT_0056e528`, а
//! следующим назначает общий start для ещё сырого `ProcessPlayerDataQueue`.
//! Factory передаётся явно вместо process-global static registry; это меняет
//! форму API, но не порядок вызова, clock-read либо accumulator side effects.
//!
//! Неуверенный AuctionBang calendar field точечно проверен в exact EXE
//! (`VERIFIED_DISASSEMBLY`): `0x00419C3A..0x00419C46` копирует девять DWORD
//! CRT `tm`, после восстановления ESP `0x00419C4D` читает destination `+0x0C`,
//! то есть `tm_mday`; сравнение с `m_lAucOldDay` выполняется at `0x00419C51`,
//! запись — at `0x00419C68`. Rust callback возвращает только наблюдаемый
//! `tm_mday`; остальные восемь stack-copy полей и `rep movsd` являются
//! ненаблюдаемым CRT/compiler mechanism. После ответа reverse прекращён.
//!
//! `SendCdkeyToLoginServer` при nullable client остаётся no-op, иначе ставит
//! приоритетный `0x1FE02 + dwNumber + m_lOnlinePlayer.size() + account...`.
//! `m_lOnlinePlayer` — `std::list<unsigned int>` с `CBaseObject::m_lID`, а
//! `m_mPlayer` — владеющий `std::map<unsigned int, CPlayer*>`; `VecDeque<u32>`,
//! `BTreeMap<u32, Box<CPlayer>>` и обычное Rust-владение заменяют только
//! STL/allocation. Порядок списка и byte-exact account C-строки сохраняются.
//! Потерянное экспортом присваивание ключа имеет статус `VERIFIED_DISASSEMBLY`:
//! exact EXE `0x00408440..0x00408453` читает `_Myval` текущего list-node,
//! сохраняет его в локальный key и передаёт адрес в `m_mPlayer.find`.
//! Отсутствующий map-owner в старом коде приводил к null-dereference на
//! `CPlayer+0x744`; safe Rust не назначает ему новое поведение и останавливает
//! только эту границу до send с локальным `BLOCKED_MISSING_FACT`.
//!
//! `GetOnlinePlayerByID` сначала линейно проходит `m_lOnlinePlayer` в его
//! list-порядке и только после первого совпадения ищет тот же unsigned ID во
//! владеющем `m_mPlayer`. Поэтому запись только в map не считается online, а
//! отсутствие на любой ступени возвращает `Option::None`. `VecDeque::iter` и
//! `BTreeMap::get` заменяют только STL traversal; запрос не мутирует владельцев.
//! Прямой `GetMapPlayer` выполняет только вторую ступень. Login-вариант сначала
//! так же линейно проверяет `m_lLoginPlayer`, чей точный PDB-элемент размером
//! `0x8` содержит `unsigned long dwPlayerID` по `+0x0` и `dwLoginTime` по
//! `+0x4`, а затем вызывает тот же map lookup. Потерянные экспортом ключи имеют
//! статус `VERIFIED_DISASSEMBLY`: `0x00407105` берёт адрес входного аргумента
//! `GetMapPlayer`, а `0x004073A0/0x004073BF` загружает и передаёт тот же
//! аргумент после сравнения с login-record. Rust хранит полный достигнутый
//! record, хотя эти два lookup-а читают только ID.
//! `AppendLoginPlayer` оставляет первый существующий ID и его прежнее время
//! полностью неизменными, иначе дописывает новую пару в хвост списка.
//! `RemoveLoginPlayer` удаляет только первый совпавший ID и ничего не делает
//! при отсутствии. `VecDeque::push_back/remove` заменяют allocation и link-
//! мутации STL-list, сохраняя порядок и наблюдаемое содержимое записей.
//!
//! `ProcessTimeOutLoginPlayer` снимает один `timeGetTime` на весь login-list и
//! использует strict unsigned `release < now - login_time`. Неистёкший узел
//! сохраняется; истёкший узел без `m_mPlayer` owner-а тоже сохраняется. Для
//! найденного игрока сначала без приоритета отправляется
//! `0x1FF06 + account\0 + name\0 + level`, затем signed `m_lTeamID` ищется в
//! PDB-map `std::map<unsigned long,long>` и даже нулевой session ID проходит
//! явную границу `QuerySession -> CTeam -> QueryPlugByOwner(type,id) -> Exit`.
//! Эти ещё сырые session/team/plug owner-ы не объявлены готовыми и передаются
//! одним typed callback-ом. После него удаляется текущий login-node, затем в
//! точном порядке выполняются `RemoveOnlinePlayer`, уникальная offline-вставка
//! и friend-list рассылки `0x7F905 + friendID + expiredName\0`. Friend lookup
//! использует только online owner; route берётся через online player/region/
//! GameServer и при любом miss остаётся исходным нулём, но send всё равно
//! выполняется. Результаты send-ов не меняют дальнейшие эффекты.
//!
//! Ошибочный raw decompile показывал `return` сразу после удаления login-node.
//! `VERIFIED_DISASSEMBLY` exact EXE `0x00414BE4..0x00414C05` подтверждает
//! unlink/free, а `0x00414C09..0x00414D37` — последующие offline и friend
//! эффекты без возврата. Поэтому Rust продолжает с сохранённым следующим
//! узлом. Полный псевдокод заменён этой локальной спецификацией; STL/RTTI/SEH
//! mechanics удалены, `VecDeque/BTreeMap/Box` и готовый message-owner заменяют
//! только технический механизм.
//!
//! Финальный MainLoop хвост сначала снимает pacing tick, при unsigned delta
//! меньше `40` вызывает внешний Linux wait-adapter на точный остаток, затем
//! wrapping увеличивает deadline на `40`. Warning использует signed cast
//! `sample - deadline` и строгое `1000 < lag`; после точной строки он отдельным
//! tick пересинхронизирует deadline. Следующий tick проверяет strict unsigned
//! release gate против отдельного BSS-state, назначает last tick до owner-а и
//! только затем вызывает `ProcessTimeOutLoginPlayer`, который снимает свой
//! собственный snapshot tick. Неназначенный setup interval блокирует только
//! уже достигнутую границу после pacing; Windows process handle/FFI не вводятся.
//!
//! Полный `CGame::MainLoop` теперь связывает все эти стадии одним owner-ом:
//! три lazy initializer-а, current/Largess/refresh/reload/maintenance/collect,
//! save pre-gate, AI и весь последующий хвост выполняются строго в исходном
//! порядке. `CLargess::StartWorkerThread` остаётся сырым доменным owner-ом, но
//! его callback вызывается синхронно ровно на доказанном gate. Аналогично
//! внешний save-thread launcher вызывается непосредственно после snapshot и
//! cleanup, до повторного connected-count и `SaveNotify`; callback возвращает
//! только наблюдаемое `Open/Empty` состояние opaque handle-а. State/domain/
//! platform зависимости сгруппированы в borrowing-структуры, но не объединены
//! в новый singleton и не меняют владельцев поведения. Normal path возвращает
//! исходный `1`; первый typed block прекращает оставшиеся стадии без rollback.
//! Большой block boxed только на ошибочной границе, поэтому обычный turn не
//! получает дополнительной heap allocation. Полный заменённый псевдокод и
//! SEH/STL/Windows wait mechanics удалены.
//! Отдельный `m_lOfflinePlayer` содержит только unsigned ID. Его `Clear`
//! удаляет весь список, а `RemoveOfflinePlayer` сохраняет семантику
//! `std::list::remove` и удаляет все равные ID. `VecDeque::clear/retain`
//! заменяют только STL node traversal и освобождение, не смешивая offline-
//! состояние с online/login списками.
//! `AppendOfflinePlayer` читает signed `CBaseObject::m_lID` через точную
//! base-цепочку игрока, сохраняет его 32-битный шаблон как list `unsigned int`,
//! оставляет первый duplicate без изменений и иначе добавляет ID в хвост.
//! Живая `&CPlayer` заменяет исходный сразу разыменовываемый pointer; nullable
//! API не вводится, поскольку null-поведение оригинала не было контрактом.
//! `ClearMapPlayerForOffline` проходит player-map в unsigned key-order. Для
//! каждого ID он сначала линейно ищет online-list и только при miss — login-
//! list; первое совпадение сохраняет owner. ID, отсутствующий в обоих списках,
//! virtual-уничтожается и стирается, после чего обход продолжается с iterator,
//! возвращённого `erase`. `BTreeMap::retain` сохраняет этот key-order и
//! short-circuit lookup, а `Box`/`Drop` заменяют deleting destructor. Offline-
//! list функция не читает, критическую секцию не берёт и другие списки не
//! меняет.
//!
//! `AppendOnlinePlayer` читает тот же унаследованный ID, линейно ищет его в
//! `m_lOnlinePlayer` и добавляет в хвост только при отсутствии. Organizing
//! enter callback вызывается всегда: и для первого добавления после list-
//! мутации, и для уже существующего duplicate без мутации. `RemoveOnlinePlayer`
//! сначала выполняет исходный `std::list::remove`, то есть удаляет все
//! совпадения, а затем всегда вызывает organizing exit независимо от их числа.
//! `VecDeque::contains/push_back/retain` сохраняют linear traversal, порядок и
//! освобождение list-node через `Drop`.
//!
//! Точный PDB задаёт `CGame::tagDBData` размером `0xA8`: два scalar ID по
//! `+0x0/+0x8`, `liDBCreationPlayer` по `+0xC`, `lDBRestorePlayer` по `+0x18`,
//! `liDBDeletionPlayer` по `+0x24`, `mDBPlayer` по `+0x30`, faction/union
//! save-списки по `+0x3C/+0x48`, delete-ID списки по `+0x54/+0x60`, enemy-
//! список по `+0x6C`, region-список по `+0x90` и country-список по `+0x9C`.
//! Pointer `pVariableList` по `+0x4`, village/city-war списки `+0x78/+0x84`
//! пока остаются у своих RAW-владельцев.
//! Конструктор создавал пустыми только списки/map и не назначал scalar-поля,
//! поэтому достигнутая Rust-часть хранит ID как `Option`, не придумывая нули.
//!
//! `VecDeque<Box<CPlayer>>` и `BTreeMap<u32, Box<CPlayer>>` заменяют только
//! list/map nodes и virtual deleting destructor. Один `parking_lot::Mutex`
//! сохраняет границу `g_CriticalSectionSavePlayerList`: поиск duplicate,
//! уничтожение прежней копии, удаление записи и вставка новой остаются под
//! одним lock. Он не удаляется вслед за общим scratch-буфером и пока не
//! выдаётся за отдельную сериализацию внешнего `SaveThreadFunc`.
//! `AppendDBCreationPlayer` при совпадении ID именно заменяет первую старую
//! копию новой: переходы exact EXE `0x00410FF0..0x00411024` после destructor и
//! unlink продолжают к tail-insert; это `VERIFIED_DISASSEMBLY`, после ответа
//! reverse прекращён. `AppendDBPlayer` уничтожает прежнее map-value до вставки
//! новой пары с unsigned bit-pattern inherited signed ID. Остальные шесть
//! append-owner-ов только отклоняют null через входной Rust-тип и дописывают
//! значение в хвост соответствующей очереди под тем же lock.
//!
//! `ClearDBData` строго очищает creation, restore, deletion, player-map,
//! save-factions, save-unions, delete-factions, delete-unions и region nodes
//! под одним lock; scalar ID не сбрасываются. Non-null player/faction/union
//! копии уничтожаются в list/key order. Region values представлены
//! `Option<RegionSaveSnapshot>`: предшествующая save-фаза ставит `None`, а
//! `ClearDBData`, как exact EXE, удаляет только nodes. Диапазон
//! `0x0040D490..0x0040D76B` подтвердил непрерывный проход без ложных raw-return
//! и единственный unlock; это `VERIFIED_DISASSEMBLY`, после ответа reverse
//! прекращён. Enemy/war/country поля функция исходно не трогала.
//!
//! Исходный singleton заменён явно переданным единственным mutable
//! `COrganizingCtrl`; это меняет форму API, но не порядок либо владельца
//! callback-эффектов. Typed отчёты сохраняют факт вставки/число удалений и
//! полный organizing outcome. Если callback достигает старого локализованного
//! UB, уже выполненная list-мутация не откатывается, а вызывающий owner получает
//! blocked-результат и не обязан придумывать продолжение. Новая append-ветвь и
//! remove-хвост после callback-а вызывали `AddPlayerList`; duplicate append
//! возвращался раньше. Exact адрес `0x00401000` содержит единственный `ret`,
//! поэтому map/name lookup remove-ветви и строковый выбор новой append-ветви не
//! имеют наблюдаемого эффекта и Linux-аналога не получают.
//!
//! `GetOnlinePlayerIDByName` проходит владеющий player-map в unsigned numeric
//! порядке. Для каждого `CPlayer` он сравнивает унаследованное имя с входной
//! C-строкой через `_strcmpi`, а при равенстве линейно проверяет тот же map-key
//! в online-list и возвращает list ID. Не-online совпадение не завершает map-
//! обход. Exact linked CRT `0x0052A629..0x0052A691` при неизменённой C-locale
//! использует ASCII-only folding по `0x00525D70..0x00525DBD`; в executable-
//! секциях нет project-call к `_setlocale` `0x00520AA9`. Поэтому
//! `eq_ignore_ascii_case` над C-string prefix сохраняет ASCII-регистр и
//! byte-exact high bytes без Windows CRT в Linux runtime.
//! `GetMapPlayerIDByName` использует тот же map-order и `_strcmpi`, но не
//! проверяет online-list: первое совпавшее имя сразу возвращает map-key, иначе
//! результат равен нулю. Общий Rust helper C-string prefix сохраняет остановку
//! на NUL, а `BTreeMap` — исходный unsigned порядок MSVC map.
//!
//! `GetLoginPlayerIDByName`, напротив, проходит login-list по порядку, ищет
//! player-map owner по `tagLoginPlayer::dwPlayerID`, пропускает отсутствующий
//! owner и сравнивает имя с учётом регистра обычным C-string `strcmp`-
//! эквивалентом. При совпадении возвращается унаследованный ID найденного
//! `CPlayer`, а не безусловно list-key. Потерянный raw key имеет статус
//! `VERIFIED_DISASSEMBLY`: exact EXE `0x00407292..0x004072A5` копирует
//! `[login_node+0x8]` в локальный ключ и передаёт его в `m_mPlayer.find`.
//! `BTreeMap`, `VecDeque`, `Box` и slice-заимствование заменяют только
//! STL/pointer/string storage; `&[u8]` исключает исходный null-аргумент, который
//! обе функции немедленно разыменовывали.
//!
//! `ToStrlwr` не был CRT lowercase: он проходил C-строку до NUL и применял
//! project switch только к 33 входным байтам. Exact диапазон
//! `0x00402000..0x00402139` и его jump tables подтверждают отображения
//! `0xC0..=0xD7 -> +0x20`, `0xDA..=0xDF -> +0x20` и две наблюдаемые странности:
//! ASCII `0x54 ('T') -> 0xAC`, а оба `0xD8/0xD9 -> 0xF9`. Эти значения
//! сохраняются буквально; остальные байты не меняются. Mutable Rust-slice
//! заменяет возвращаемый тот же `char*`; caller задаёт живые байты C-строки
//! без обязательного хранения служебного NUL, поэтому null/overread не
//! переносятся в Linux runtime.
//!
//! `IsNameExistInMapPlayer` и `GetCreationPlayerByName` на каждой map-записи
//! независимо копируют input и inherited player name в два `char[260]`, затем
//! применяют этот custom lowercase и сравнивают C-строки byte-exact. Максимум
//! 259 bytes плюс NUL безопасен; при 260 и более exact linked `_snprintf`
//! `0x0051B932..0x0051B945` оставлял buffer без terminator, а следующий
//! `ToStrlwr` читал за стеком. Достижимость и реакция такого пути не доказаны,
//! поэтому Rust возвращает локальный
//! `BLOCKED_MISSING_FACT`, не назначая старому UB совпадение либо отказ.
//!
//! Первый lookup возвращает `true` при первом lower-case совпадении независимо
//! от operational list-ов. Второй после совпадения имени линейно требует тот же
//! unsigned map-key как тот же 32-битный шаблон в signed
//! `m_lCreationPlayer` и только тогда возвращает map-value; иначе map-обход
//! продолжается. Точный PDB задаёт этот список как `std::list<long>` по
//! `CGame+0x20`; `VecDeque<i32>` сохраняет list-порядок, signed storage и
//! duplicate-наблюдаемость, а `BTreeMap` — unsigned map-порядок. Испорченные
//! raw return-ы имеют статус `VERIFIED_DISASSEMBLY`: exact
//! `0x004052AF..0x004052CC` возвращает `false/true`, а
//! `0x004054BF..0x004054DD` — `nullptr/[map_node+0x10]`.
//!
//! `CPlayer::ChangeName` exact `0x0045D1C0..0x0045D39A` использует эти
//! lookup-и в порядке map -> frozen DB data -> frozen DB creation -> ADO.
//! До них он ограничивает новое C-string имя 16 байтами, требует
//! case-sensitive вхождения `strSpeStr` именно в текущем имени и передаёт
//! отдельную `std::string`-копию в `CheckInvalidString(false)`, но после
//! проверки продолжает работать с исходным input. Rust сохраняет этот порядок,
//! byte-exact input и коды `1/8/2/3/4/5/6/7/0`; Tiberius parameter binding,
//! `Vec` и explicit borrows заменяют только ADO/STL/global plumbing.
//!
//! `ClearCreationPlayer` удаляет все list-node без player-map мутаций и без
//! уничтожения самих `CPlayer`; `VecDeque::clear` заменяет только link traversal
//! и освобождение узлов. `GetCreationPlayerCountInCdkey` проходит player-map в
//! unsigned порядке, сравнивает byte-exact account C-строки через тот же
//! доказанный ASCII-only CRT `_strcmpi` и для каждого совпавшего map-key
//! полностью проходит creation-list. Каждый равный list-node увеличивает
//! `unsigned char`, поэтому duplicate-ы считаются отдельно, а результат
//! сохраняет 8-битное wrapping. Rust-срез исключает немедленно разыменовываемый
//! null account-аргумент; player-map по достигнутому ownership хранит живой
//! `Box<CPlayer>` вместо исходного raw pointer.
//!
//! `AppendCreationPlayer` материализован полностью, включая обе ненормальные
//! ветви.
//! После проверки duplicate ID она сначала добавляет новый ID в хвост
//! creation-list, затем ищет map-key. Exact
//! `0x00410C12..0x00410C5D` подтверждает: существующий ненулевой map-value
//! вызывает лог и ранний return уже после list-мутации; отсутствующий key либо
//! существующий null переходит в `operator[] = incoming player`. Duplicate ID
//! по `0x00410C36..0x00410C4A` virtual-удаляет incoming player до возврата.
//! Текст лога `MapPlayer Not Found or NULL.` противоречит условию, но условие
//! сохраняется по машинному коду.
//!
//! Executable-wide direct-xref нашёл единственный caller `0x004B1AE0` в
//! create-role ветви `OnLogMessage`: он передаёт freshly allocated player с
//! только что увеличенным `m_nPlayerID`, не проверяет void-результат и сразу
//! продолжает читать тот же pointer для equipment/response. После normal insert
//! map владеет объектом, а caller использует non-owning alias. Duplicate-ветвь
//! поэтому была бы use-after-free, existing-non-null ветвь — дальнейшим
//! использованием и затем утечкой incoming pointer; отдельного delete до выхода
//! нет. Rust normal path принимает `Box`, вставляет list ID и передаёт Box map-у.
//! Duplicate-ветвь завершает safe Rust-caller typed результатом после
//! доказанного `Drop`. Existing-owner ветвь возвращает incoming `Box` caller-у,
//! потому что сама исходная функция его не потребляла; collision сохраняет уже
//! выполненный list `push_back`. Судьба этого pointer после возврата относится
//! к единственному caller-у `OnLogMessage`, а не к append-owner-у. Текущий
//! `BTreeMap<u32, Box<CPlayer>>` также сознательно не получает nullable value по
//! одному защитному условию оригинала. Typed log-callback вызывается в точных
//! местах обоих `AddLogText`: до уничтожения duplicate и после list-мутации у
//! existing owner; `Display` сохраняет исходные строки.
//!
//! Точный PDB задаёт `CGame::m_nPlayerID` как `unsigned long` по `+0x68`.
//! `CGame::CGame` RVA `0x00015210` это поле не пишет. `CRsSetup::LoadPlayerID`
//! загружает его из `csl_setup.playerID`, а при пустой выборке или exception
//! пишет `0`; create-role по `0x004B1AC1` выполняет обычный x86 `add 1` с
//! 32-битным wrapping. Исполняемый файл содержит только эти четыре записи через
//! `GetGame()` и ни одной записи через 98 прямых загрузок `g_pGame`. Проверки
//! счётчика против player-table/map нет, поэтому collision не объявляется
//! недостижимым: fallback в ноль, отставшее сохранение либо wrapping допускают
//! повторный ID.
//!
//! Live restore/deletion owner-ы сохраняют точные PDB-типы
//! `list<unsigned int>` по `+0x2C` и `list<tagDeletionPlayer>` по `+0x38`.
//! Clear очищает весь список, Delete удаляет первое совпадение, оба Append
//! оставляют первый duplicate без изменения, а deletion-time lookup возвращает
//! время первого совпадения либо `0`. `LoadedSetupIds` записывает результаты
//! constructor-load в ранее неинициализированные `m_nPlayerID/m_nLeaveWordID`;
//! до этого Rust хранит `None` и не читает старое UB.
//! `CFaction::LeaveWord` RVA `0x000BCA40` увеличивает `m_nLeaveWordID` обычным
//! signed x86 `add 1`; `allocate_leave_word_id` сохраняет wrapping, а
//! неинициализированное constructor-state отделяет typed-результатом.
//!
//! Player-prefix `GenerateDBData` сначала копирует оба scalar ID, затем обходит
//! signed creation-list, unsigned restore-list, deletion-list и unsigned
//! player-map. `CloneMapPlayer` теперь буквально находит unsigned key, создаёт
//! новый player constructor-state, сериализует живой owner с
//! `include_child=true`, декодирует из `vector.data()` с нулевым cursor и тем
//! же флагом, а decoder-`false` уничтожает копию и возвращает `None`.
//! Encoder сохраняет наблюдаемые мутации `SetPlayerOrganizing` и
//! `UpdateProperty`; registry, concrete organizing-owner и property
//! coefficients передаются явно вместо process-static singleton-ов. Перед
//! mutable player-borrow снимается reached проекция `tagRegion::REGION_TYPE`.
//! Exact диапазон
//! `0x00410AB0..0x00410B98` подтвердил исправленный key/dataflow и обе virtual
//! позиции. Restore-копии добавляются без save-lock, как exact тело; каждый
//! deletion-record и каждая player-копия отдельно проходят тот же
//! `g_CriticalSectionSavePlayerList`, выраженный `parking_lot::Mutex`.
//! Последующие generators `COrganizingCtrl::GenerateSaveData`,
//! `CFactionWarSys::GenerateSaveData`, а также собственный
//! `GeterateRegionDBData` и `CCountryHandler::GenerateSaveData` восстановлены
//! у своих owner-ов; `CHonorRanks::GenerateSaveData` также восстановлен в
//! отдельном static-state owner-е. Полный метод вызывает их после player-
//! prefix строго в порядке organizing с literal `force_all=false`, faction-
//! war, region, country и honor-ranks. Прежние singleton/static зависимости
//! передаются явными ссылками; это изменение формы API, а не новая
//! оркестрация. Локальная player/organizing safe-граница прекращает дальнейшие
//! стадии: продолжение после исходного UB не назначается. Сам метод не очищает
//! live player/restore/creation/deletion/offline списки — эти операции
//! принадлежат следующему caller-участку после возврата.
//! Отдельная `g_bSaveAllOrg` ветвь не вызывает player-prefix и Country:
//! `materialize_save_all_organizations_snapshot` сохраняет только organizing,
//! faction-war, region и HonorRanks, затем достигает того же launch call-site.
//!
//! `ShowSaveInfo` сначала проверяет process-global `g_bShowSaveInfo`. При
//! `false` он не форматирует аргументы и не касается log-owner-а. При `true`
//! первый `_vsprintf` материализует ANSI C-string в `char[256]`, после чего
//! передаёт её как format без varargs в `AddLogText`. Rust принимает уже
//! отформатированный call-site payload, проверяет точную первую вместимость и
//! делегирует второе форматирование готовому `WorldLogTextOwner`. Переполнение
//! первого stack-buffer остаётся локальным `BLOCKED_MISSING_FACT` до любого
//! log/tick эффекта; неизвестный `%` второго форматирования блокируется уже у
//! `AddLogText` после его доказанных rotation/time эффектов.
//! Предшествующая ручная collect-player-data ветвь теперь отдельно очищает
//! `g_bSendCollectPlayerDataMsgNow`, строит пустой `0x7F808` и ровно один раз
//! вызывает готовый `CMessage::SendAll`. Nullable server-owner сохраняет
//! старый нулевой результат, а ошибка envelope остаётся в отчёте и не меняет
//! уже очищенный флаг.
//! Reached save-trigger участок `CGame::Run` после успешного
//! `TryEnterCriticalSection` теперь исполняет обе доказанные формы решения.
//! `g_bSaveAllOrg` очищается до четырёх организационных generators. Иначе
//! manual/no-GameServer путь пишет точный log, обновляет save tick, очищает
//! `g_bSaveNowData` и связывает успешный полный snapshot с последовательностью
//! `ClearMapPlayerForOffline`, `ClearRestorePlayer`, `ClearCreationPlayer`,
//! `ClearDeletionPlayer`, `ClearOfflinePlayer` и launch request. Затем
//! подключения считаются повторно: при их наличии tick и `m_nDBResponsed`
//! сбрасываются, а пустой `0x7F803` идёт каждой connected записи в unsigned
//! map-order. Поэтому manual-save при живых GameServer выполняет и snapshot,
//! и последующую notify-рассылку, а не выбирает одно из них.
//! При локальном blocked-результате generator-а cleanup/notify не продолжаются
//! и typed guard не выдаётся за снятый. Предшествующий pre-gate также готов:
//! `g_bSendSaveMsgNow` сначала пишет точный log, назначает `last = now -
//! first_interval` и очищается. Отдельный profiling tick снимается до wrapping
//! elapsed-проверки, а `dwSavePointTime` читается повторно: при неизменном
//! setup текущая итерация видит равенство и попадает в исходное `<=`, не пробуя
//! lock. Try-lock callback вызывается только при строгом `elapsed >
//! second_interval`; его `false` делает wrapping `last += 1000` и не создаёт
//! guard, а `true` входит в готовое save-решение. `SaveThreadFunc` удерживает
//! typed guard той же внешней сериализации, пишет точные start/end events
//! вокруг готового `DoSaveData`
//! lifecycle и возвращает guard при границе, где исходник не дошёл до unlock.
//! COM apartment удалён как заменённый Windows DB-механизм. Оба достигнутых
//! `CloseHandle/__beginthreadex` caller-а закрывают только opaque handle-state и
//! возвращают одноразовый launch request; thread не создаётся.
//! `WorldDbDataSaveSession` теперь предоставляет узкие owner-операции для всех
//! четырёх player-коллекций frozen snapshot-а. Creation/restore/deletion
//! удаляют текущий `VecDeque` node только после доказанного успеха; player-map
//! уничтожает `Box<CPlayer>` и стирает key только после successful Save
//! Character. Failure сохраняет owner для retry. Эксклюзивный `&mut CGame`
//! заменяет внешнюю save-thread сериализацию, поэтому внутренний mutex не
//! удерживается через DB-await и новая конкурентная политика не вводится.
//!
//! `SetEnemyFactions` полностью заменяет pointer-list по `tagDBData+0x6C` под
//! той же save-блокировкой. Старые non-null значения уничтожаются до очистки
//! nodes, затем входной pointer-list копируется в исходном порядке; временные
//! list-копии освобождают только nodes. Owned `VecDeque<Option<_>>` и `Drop`
//! выражают этот контракт без raw pointers. `ClearDBData` этот список не
//! трогал, поэтому отдельная EnemyFactions save-phase по-прежнему отвечает за
//! уничтожение значений и очистку nodes.
//!
//! `GeterateRegionDBData` сохраняет исходную опечатку имени, проходит
//! `s_mapRegionList` в signed key-order и пропускает отсутствующий
//! `tagRegion::pRegion`. Каждый живой `CWorldRegion` создаёт отдельную полную
//! девятиполевую копию `tagRegionParam`, после чего `AppendRegionParam`
//! добавляет её в хвост DB-очереди под save-lock. Другие поля живого региона
//! не копируются. Owned snapshot заменяет выделенный базовый `CWorldRegion`,
//! поскольку следующий `CRsRegion::Save` наблюдает только `m_Param`, а
//! save-phase затем уничтожает копию.
//!
//! `AppendDBCountry` добавляет отдельный non-null country snapshot в хвост
//! `ltDBCountrys` под тем же save-lock. Nullable list-форма сохраняется для
//! уже готовой Country save-фазы, хотя достигнутый generator создаёт только
//! `Some`. `ClearDBData` country-list не трогал, поэтому его очистка не
//! присваивается этому owner-у.
//!
//! Numeric `GetRegion(long)` выполняет nullable lookup signed ключа в исходном
//! `std::map<long, tagRegion>` без `operator[]`-вставки. Прямой
//! `GetRegionGameServer` затем читает `dwGameServerIndex` найденного региона,
//! ищет его в `std::map<unsigned long, tagGameServer>` и возвращает nullable
//! указатель.
//! Два `BTreeMap` сохраняют ordered lookup, `Option` — отсутствие любой
//! ступени, а короткий numeric getter возвращает исходный `0`. Пересылка
//! принимает гарантированно живую `&CMessage`, поэтому старый nullable check
//! становится типовой границей; `u32 -> i32` сохраняет тот же 32-битный
//! map identity готового Linux/Rust `SendToMapID`.
//! Player-варианты сначала проходят тот же online-list/map lookup, затем читают
//! унаследованный `CShape::GetRegionID` и только после этого ищут назначенный
//! региону GameServer. Слот `CPlayer` vtable `+0x50` имеет статус
//! `VERIFIED_DISASSEMBLY`: exact EXE хранит vftable по `0x00543DE4`, ячейку
//! слота по `0x00543E34` и адрес `CShape::GetRegionID` `0x004530F0`.
//! `Option` сохраняет nullable результат первого варианта, второй возвращает
//! исходный ноль либо signed представление `dwIndex`; signed player ID
//! переводится в unsigned map-key с сохранением 32-битного шаблона.
//!
//! `ProcessPlayerDataQueue` снимает размер под отдельной queue-блокировкой и
//! повторяет null-pop только в пределах этого snapshot. Первый фактически
//! извлечённый record всегда завершает вызов. Null player, отсутствующий регион
//! и отсутствующий либо disconnected GameServer отправляют `0x1FF01` со
//! статусом `0x1C`; две последние ветви уничтожают player-owner. Success после
//! `SetPlayerOrganizing` и exact virtual `CShape::SetState(0)` отправляет
//! `0x1FF01` со статусом `0x1D`, account, GameServer IP/port, именем и level.
//! Затем friends обходятся строго в list-order: online lookup предшествует
//! login lookup, `bOnline` меняется до optional `0x7F904`, а найденный только в
//! login-list адресуется map ID `0`, как и отсутствующий online owner/region.
//! После обхода безусловный `RemoveOnlinePlayer` выполняет organizing-exit,
//! offline-list удаляет все совпадения, login-list получает текущий tick,
//! прежний map-owner уничтожается и incoming `Box<CPlayer>` передаётся map.
//! `VecDeque/BTreeMap/Box` заменяют только STL nodes и deleting destructor.
//!
//! Lost stack keys region/GameServer/player-map восстановлены из прямого
//! dataflow согласованного тела: соответственно `GetRegionID`,
//! `tagRegion::dwGameServerIndex` и inherited player ID. Единственная virtual
//! неоднозначность имеет статус `VERIFIED_DISASSEMBLY`: CPlayer vtable ячейка
//! `0x00543E68` содержит `0x004531D0`, точный `CShape::SetState(unsigned short)`.
//! `tagGameServer::dwPort`, оставшийся `None` после доказанного malformed
//! loader-а, останавливает только эту safe-границу после уже выполненного
//! organizing/state, не отправляя придуманный port. Ошибки send по-прежнему не
//! меняют последующие мутации и лишь возвращаются в наблюдаемом отчёте.
//! Record с `szCdkey[20]` без NUL блокируется сразу после pop: исходные
//! `CBaseMessage::Add(char*)` читали бы за fixed buffer, а наблюдаемая реакция
//! такого повреждённого producer-state не доказана.
//! `LoadPlayerDataFromDB` проверяет game-exit перед player-load-exit, делает
//! `Sleep(1)`, атомарно дренирует всю load FIFO и последовательно обрабатывает
//! каждый non-null record с ненулевым ID. Успех и DB-failure одинаково создают
//! data-record; failure несёт nullable player и поэтому позднее даёт `0x1C`.
//! Largess вызывается после DB-load и до публикации в data FIFO, start/end logs
//! используют два отдельных `timeGetTime` с wrapping elapsed. Exact
//! disassembly `0x0040952F..0x00409544` восстановил скрытый переход к следующей
//! list-записи, а `0x0040958B..0x00409599` — внешний polling-loop; Ghidra
//! ошибочно считала оба `operator delete` call-site терминаторами. Rust Drop
//! исправляет только утечки invalid/raw-pointer records, Tokio timer заменяет
//! `Sleep`, а async DB-load остаётся явным соседним owner-контрактом. Concrete
//! `WorldPlayerLoadWorkerPool` завершает system-thread/exit/join lifecycle;
//! cloneable queue-spec заменяет только исходный global `g_pGame` lookup.
//! MainLoop использует назначенный предыдущей стадией shared tick, добавляет
//! wrapping elapsed в `DAT_0056e524` и отдельным tick начинает сырой
//! `CTimer::Run`; при safe block эти недостигнутые clock-эффекты не создаются.
//! Timer-dispatch теперь сам выполняет exact `CPlayerRanks::OnStatRanks`:
//! stat, публикация, отдельный local-time, событие следующего дня и только
//! затем удаление текущей calendar-записи. Тот же adapter выполняет tax-
//! callback `COrganizingParam`: broadcast `0x7FE26`, следующий день, регистрация
//! и лишь затем lookup/log `WS0263`. Async TDS await остаётся внутри исходной
//! позиции PlayerRanks; остальные callbacks проходят прежний sync dispatcher.
//! После полного timer-call wrapping закрывает `DAT_0056e520`, а отдельный tick
//! начинает `CFactionWarSys::Run`. Faction-war call после полного expiry-пути
//! единственным end tick увеличивает
//! `DAT_0056e51c`. Перед следующим `CLeiTing::Run` новый shared tick в exact
//! MainLoop отсутствует, поэтому Rust его не добавляет. Сам следующий
//! непрофилированный `CLeiTing::Run` также связан: caller снимает ровно один
//! local `tm`, а после полного daily owner-а оставляет следующей сырой
//! границей `CDbMisc::DoneOutList`. Соседний DB batch теперь также связан
//! целиком: output limit `8`, полное снятие input queue, ordered DB-dispatch и
//! один `LoadAuction`. Только после них единственный tick назначает start
//! следующей профилированной границы `CNetSessionManager::Run`. Полный
//! ordered timeout pass теперь закрывает `DAT_0056e518` одним end tick; нового
//! shared start перед следующей ping-границей исходник не назначает. Ping-
//! граница MainLoop RVA `0x00019A00` также закрыта целиком: при active flag
//! она сравнивает unsigned число подключённых GameServer с числом ответов либо
//! ждёт строгий wrapping timeout `5000 < elapsed`; завершение сначала очищает
//! flag, затем отправляет LoginServer `0x1FE04` с online count и ordered
//! `(IP C-string, map ID, player count)` snapshot. LoginServer decoder того же
//! wire независимо подтверждает порядок полей.
//!
//! BaiTan registry-cluster также восстановлен: `DelItemToIpList` RVA
//! `0x0000D830`, `AddItemToBaiTanRequestList` RVA `0x0000EE30`,
//! `DelItemFromBaiTanList` RVA `0x000129F0`, `AddItemToBaiTanList` RVA
//! `0x00014140` и `DoneBaiTanList` RVA `0x000141F0`. Потерянные raw-
//! присваивания `std::pair` имеют статус `VERIFIED_DISASSEMBLY`: exact EXE
//! `0x0040EE33..0x0040EE51` подтверждает request `(IP, player_id)`, а
//! `0x00414150..0x004141D9` — `(player_id, IP)`, `(IP, 1)` с wrapping
//! increment и `(player_id, GameServer index либо 0)`. Независимые
//! `std::map::insert` игнорируют duplicate каждый сам по себе; Rust сохраняет
//! это, включая возможный отставший refcount. Batch идёт по unsigned IP-order,
//! для каждого сначала мутирует registries, затем отправляет `0x8040D +
//! player_id + 1` даже на route `0`, и только после всего обхода очищает
//! request map.
//!
//! Следующая MainLoop minute-chain теперь объединяет lazy bits `0x10`, `0x20`
//! и `0x40`, их точные два initial clock-call, новый current tick, wrapping
//! `(current - minute_start) / 60000`, полный `COrganizingCtrl::Run`, полный
//! `CCountryHandler::Run` и только после обоих обновляет minute start.
//! `DisbandFaction` вызывается concrete organizing owner-ом через живые
//! Village/City/Goods War/country owners; player flag и optional log завершают
//! его exact success-порядок сразу после удаления faction;
//! та же finalization-последовательность обслуживает прямой message-ingress
//! `0x60111`,
//! поэтому retired owner не задерживается внутри event-report, а DB-log и
//! player flag сохраняют общий машинный порядок;
//! `CCountry::AI` выполняется concrete country owner-ом через governance context;
//! при локальном blocked-path последующие эффекты не выдумываются.
//! Сразу после этого `run_main_loop_bai_tan_jjc_stage` без нового clock-call
//! завершает весь BaiTan batch и запускает полный `CJJcSystem::Run`. JJC сам
//! сохраняет rank gate, weekly/season reset, fight timeout, message и recycle;
//! ещё сырые `CRsJJcSys`/INI/time/log границы остаются явным контекстом, а не
//! объявляются готовыми через MainLoop wrapper.
//!
//! Точный PDB задаёт старому `CGame::tagRegion` размер `0xC`: nullable
//! `CWorldRegion* pRegion` по `+0x0`, `unsigned long dwGameServerIndex` по
//! `+0x4` и `REGION_TYPE RegionType` по `+0x8`. `LoadRegionList` RVA
//! `0x00012220` теперь materializes все десять positional полей и сохраняет
//! полный numeric selector. `WorldRegionOwner` сохраняет исходный virtual
//! выбор `0/4/5 -> CWorldRegion`, `1 -> CWorldVillageRegion`,
//! `2 -> CWorldCityRegion`; COUNTRY `3`, которого нет в поставочном list,
//! остаётся единственной узкой raw-границей. `Option<enum>` заменяет nullable
//! владеющий pointer и allocation, но не выдаётся за x86 layout.
//! Владение подтверждено парой lifecycle-границ: `LoadRegionList` RVA
//! `0x00012220` выделяет конкретный virtual region и записывает pointer в map,
//! а `CGame::Release` RVA `0x0000E7F0` обходит map, вызывает virtual deleting
//! destructor каждого ненулевого pointer и обнуляет его. Resource reads теперь
//! происходят у concrete owner-а в исходном порядке, а два global object-count
//! остаются явной process-state границей. Region snapshot также вызывается
//! прямо; точечный exact disassembly `0x004165C1..0x004165D2` подтверждает
//! `include_child=true` перед virtual slot `+0x10`. Безопасные parser/wire
//! неизвестности возвращаются через reload/init/MainLoop как локальный block,
//! а не получают придуманный legacy result.
//! `ReLoadOneRegionSetup/ReLoadAllRegionSetup` RVA
//! `0x000120F0/0x000108F0` уже сохраняют signed map-order, null-skip и точный
//! `0x7F801/0x10` send соответствующему GameServer.
//! Init-pass после organizing load больше не делегирует целый
//! `InitOwnerRelation` внешнему контексту: каждый concrete region owner прямо
//! проверяет faction/union, восстанавливает faction city-list и country byte.
//! Внешней остаётся только узкая player-refresh граница уже готового
//! `CFaction::AddOwnedCity`.
//!
//! `region_name` выполняет ровно цепочку numeric `GetRegion -> pRegion ->
//! CWorldRegion -> CRegion -> CBaseObject::m_strName`. Typed lookup сохраняет
//! разницу между отсутствующим map-key и существующим `tagRegion` с null
//! pointer: первый в enter-ветви означает пустую C-строку, второй исходно
//! разыменовывался и остаётся локальным `BLOCKED_MISSING_FACT`. Живое имя
//! заимствуется byte-exact без UTF-8 и без копирования MSVC `std::string` SSO.
//!
//! `LoadServerSetup` читает byte-exact whitespace-токены: произвольный header,
//! signed число записей, затем после каждого точного `#` — unsigned ID,
//! IP-строку и unsigned port. Map заранее не очищается, повторный ID полностью
//! заменяет пять полей существующей записи, а отсутствующий файл возвращает
//! старый `false`. ASCII-insensitive поиск имени сохраняет Windows-доступ к
//! найденному lowercase `serversetup.ini`; `fs::read`, `Vec` и `BTreeMap`
//! заменяют `CRFile`, stringstream, `std::string` и `std::map`.
//!
//! Единственный неуверенный stack-факт имеет статус `VERIFIED_DISASSEMBLY`:
//! exact EXE `0x004139FE` читает неинициализированный `[esp+0xC8]`, а
//! `0x00413A08` пишет его в `tagGameServer::lReceivedPlayerData`; до чтения
//! слот не получает записи и не передаётся extraction-вызовам. `Option<i32>`
//! сохраняет эту неизвестность без выдуманного нуля. Missing-file ветвь
//! `0x004138C8` возвращает `0`, успешный хвост `0x00413A95` — `1`.
//!
//! Пять прямых запросов registry используют тот же ordered
//! `std::map<unsigned long, tagGameServer>`. `BTreeMap` сохраняет числовой
//! порядок обхода: обычный count считает все подключённые записи, вариант Ex
//! исключает `dwIndex == 5`, а 32-битный `wrapping_add` воспроизводит машинный
//! `add` старого x86. Адресный lookup возвращает первую запись с byte-exact,
//! case-sensitive полным равенством IP и тем же unsigned port; numeric lookup
//! и отсутствие записи представлены `Option`. Если malformed loader оставил
//! port неизвестным именно у совпавшего IP, typed-ошибка локализует сравнение
//! старого неинициализированного DWORD вместо притворного miss. Signed аргумент
//! `IsConnect(long)` переводится в unsigned map-key с сохранением 32-битного
//! шаблона. STL tree traversal и `operator[]` удалены как библиотечный механизм;
//! запросы не мутируют registry и не требуют повторного reverse.
//!
//! Найденный локальный `WorldServer/setup.ini` с SHA-256
//! `F2E3AB113D98554787CE0A76C1524564D5CC8F62E19B5BDF6B6119299814434A`
//! содержит 40 полных пар. Оригинал пытается прочитать 41-ю пару в
//! `bUseOldSaveLargessWay`, поэтому на этом файле поле сохраняет constructor-
//! default `true`. Значения, включая credentials, в код, отчёты и ошибки не
//! переносятся.
//!
//! После positional-чтения оригинал строил GUI single-instance заголовок:
//! `WorldServer[name]-Saga3D2` для INI и `WorldServer[name]` для DAT. Готовая
//! Linux callback-граница закрепляет тот же title либо возвращает занятую
//! instance-ветвь в полный `Init`, не вводя `FindWindowA`/MFC runtime.
//!
//! `CheckPoint(char*)` удваивает каждый байт одинарной кавычки для старого SQL-
//! literal. Exact `0x00409B17..0x00409B49` обнуляет локальный `char[256]`,
//! `0x00409C55..0x00409C68` вызывает `_snprintf(..., 0x100, "%s", ...)`, а
//! `0x00409CBB` возвращает адрес этого уже заканчивающего lifetime stack-
//! буфера; `this` функция не читает. Достигнутый caller
//! `0x004FADCB..0x004FADEB` копировал байты в свой SQL-buffer без промежуточного
//! вызова. Rust меняет только форму API и
//! возвращает owned `Vec<u8>`, сохраняя точные escaped bytes. При длине
//! результата `>= 256` linked MSVC `_snprintf` не ставил NUL, после чего caller
//! читал за stack-buffer; это локальный `BLOCKED_MISSING_FACT`, а не основание
//! для truncation, `unsafe` либо нового fail-closed результата.
//!
//! Write-log producer `0x6020D` хранит в `CGame` FIFO структурированных
//! `IncrementLog` DB-команд вместо готовых SQL literals. Это техническая
//! замена для параметризованного Tiberius worker-а: packet-поля, enqueue
//! position и исходный enqueue-before-live-Add сохраняются, а временные
//! `_sprintf/CheckPoint` buffers и SQL injection не воспроизводятся.
//! Consumer batch также достигнут: exact `DoSaveLog` сначала снимал размер,
//! затем удалял каждый SQL до Execute и после failure восстанавливал connection,
//! не повторяя потерянную запись. `WorldWriteLogWorkerSpec::process_batch`
//! сохраняет этот observable data-loss/FIFO контракт и продолжимый snapshot-
//! checkpoint; typed Execute и connection open находятся в соседнем
//! `writelogworker`.
//! Внешние thread/exit/poll/reconnect-delay owners пока остаются RAW.
//!
//! `ResetHonorElimilateInfo` RVA `0x00014390` проходит `m_mPlayer` в map-order,
//! сбрасывает достигнутые day/week/month counters по исходной mask-семантике,
//! затем полностью очищает `m_HonorElimilateList` и повторяет reset под lock
//! `CPlayerDataQueue`. `BTreeMap`, `VecDeque` и `parking_lot::Mutex` заменяют
//! только STL/critical-section plumbing; накопительный total не меняется.
//! Ветка `OnOtherMessage(0x5FD0D)` использует тот же map как per-player список
//! уже учтённых eliminator ID: отсутствие online player и дубликат завершают
//! обработку, новая пара добавляется в хвост до чтения четырёх счётчиков.

use std::collections::{BTreeMap, VecDeque};
use std::convert::Infallible;
use std::error::Error;
use std::ffi::CString;
use std::fmt;
use std::fs;
use std::future::Future;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use parking_lot::Mutex;
use rustix::system::uname;
use rustix::time::{ClockId, clock_gettime};
use tiberius::Query;
use walkdir::WalkDir;

use crate::dbaccess::worlddb::dbcountry::{CountrySaveSnapshot, DbCountryOwner};
use crate::dbaccess::worlddb::dbgoods::DbGoodsOwner;
use crate::dbaccess::worlddb::dbmisc::{
    CDbMisc, DbMiscContext, DbMiscDoneInReport, DbMiscDoneOutBlock, DbMiscDoneOutReport,
    DbMiscLoadAuctionReport,
};
use crate::dbaccess::worlddb::largess::{
    LargessOwner, LoadLargessBlock, LoadLargessReport, TiberiusLargess,
};
use crate::dbaccess::worlddb::playerdataqueue::{
    CPlayerDataQueue, PlayerDataQueueEntry,
};
use crate::dbaccess::worlddb::playerloadqueue::{
    CPlayerLoadQueue, PLAYER_LOAD_CDKEY_CAPACITY, PlayerLoadPushOutcome,
    PlayerLoadQueueEntry,
};
use crate::dbaccess::worlddb::rsenemyfactions::{EnemyFactionSaveSnapshot, RsEnemyFactionsOwner};
use crate::dbaccess::worlddb::rsfaction::RsFactionOwner;
use crate::dbaccess::worlddb::rsgenvar::RsGenVarOwner;
use crate::dbaccess::worlddb::rsgodsbattle::{
    GodsBattleFactionXydSnapshot, GodsBattleNpcFactionSnapshot, RsGodsBattleOwner,
    TiberiusRsGodsBattle,
};
use crate::dbaccess::worlddb::rsjjcsys::RsJjcSysOwner;
use crate::dbaccess::worlddb::rsplayer::{
    HonorRanksLoadOutcome, PlayerRanksStatBlock, PlayerRanksStatOutcome, RsPlayerOwner,
    TiberiusRsPlayer,
};
use crate::dbaccess::worlddb::rsregion::{RegionSaveSnapshot, RsRegionOwner};
use crate::dbaccess::worlddb::rssetup::{
    LoadedSetupIds, RsSetupOwner, WorldDatabaseSettings, WorldDatabaseSettingsParts,
    WorldTdsClient,
};
use crate::dbaccess::worlddb::rsunion::RsUnionOwner;
use crate::dbaccess::worlddb::writelogqueue::WorldWriteLogQueue;
use crate::nets::clients::ClientConnectError;
use crate::nets::mysocket::{DEFAULT_SOCKET_TYPE, legacy_ipv4_word};
use crate::nets::networld::message::{CMessage, SendMessageError, WorldMessageHandlers};
use crate::nets::networld::mynetclient::CMyNetClient;
use crate::nets::networld::mynetserver::{CMyNetServer, WorldServerEvent};
use crate::nets::servers::{ServerCommandHandle, ServerHostError};
use crate::public::auctionlog::{
    AuctionBangUpdateOutcome, AuctionLogLoadOutcome, CAuctionLog,
};
use crate::public::ciqing::{CCiQingSetup, CiQingSerializationBlock};
use crate::public::date::TagTime;
use crate::public::dupliregionsetup::CDupliRegionSetup;
use crate::public::equipmentcomposelist::{
    EquipmentComposeList, EquipmentComposeSerializeError,
};
use crate::public::taozhuangsetup::{CTaoZhuangSetup, TaoZhuangSerializationBlock};
use crate::setup::hitlevelsetup::{CHitLevelSetup, HitLevelFormatError, HitLevelSerializeError};
use crate::setup::honorelimilateconfig::HonorElimilateConfig;
use crate::setup::contributesetup::{
    CContributeSetup, ContributeSetupFormatError, ContributeSetupSerializeError,
};
use crate::setup::cbattlefairyexpconfig::{BattleFairyExpSerializeError, CBattleFairyExpConfig};
use crate::setup::emotion::{CEmotion, EmotionFormatError, EmotionSerializeError};
use crate::setup::goodsdestructionconfig::{
    GoodsDestroyFormatError, GoodsDestroySerializeError, GoodsDestroySetup,
};
use crate::setup::incrementshoplist::{
    CIncrementShopList, IncrementShopGoodsQuery, IncrementShopGoodsResult,
    IncrementShopSerializeError,
};
use crate::setup::prisonconf::{PrisonConf, PrisonConfFormatError, PrisonConfSerializeError};
use crate::setup::tradelist::{CTradeList, TradeListFormatError, TradeListSerializeError};
use crate::public::mystringtable::MyStringTable;
use crate::public::netsessionmanager::{CNetSessionManager, NetSessionRunReport};
use crate::public::wordsfilter::CWordsFilter;
use crate::public::readwrite::read_to;
use crate::public::timer::{
    AsyncTimerCallbackDisposition, AsyncTimerCallbackHandler, AsyncTimerRunBlock,
    CalendarTimerRegistration, CTimer, TimerCallbackInvocation, TimerCallbackSource, TimerId,
    TimerRunReport,
};
use crate::setup::globesetup::GlobeSetupSnapshot;
use crate::setup::godsbattleconf::CGodsBattleConf;
use crate::setup::leitingsetup::{CThingSetup, ThingSetupCodecError};
use crate::setup::newskillmonsterlist::{
    NewSkillMonsterConf, NewSkillMonsterSerializeError,
};
use crate::setup::playerlist::{CPlayerList, PlayerListFormatError, PlayerListSerializeError};
use crate::setup::synthesis::{CSynthesis, SynthesisSerializeError};
use crate::setup::regionrouter::RegionRouter;
use crate::public::tools::{ini_decode, put_string_to_file};
use crate::transport::bind_tcp_ipv4;
use crate::worldserver::appworld::country::country::{
    CountryAbsolveCounterReset, CountryExileMessageDelivery, CountryExileResultContext,
    CountryExileTarget, CountryExileTextArgument, CountryFactionSnapshot, CountryNewTermContext,
    CountrySetNewDayContext,
    CountryGovernanceContextBlock, CountryKingSaveLimits, CountryOnlinePlayer,
    CountryPlayersListContext, CountryPlayersListContextBlock,
    CountryVillageTaxContext, CountryVillageTaxContextBlock, CountryVillageTaxRegion,
};
use crate::worldserver::appworld::goods::cbattlefairyproperty::{
    BattleFairyComposeWireError, CBattleFairyProperty,
};
use crate::worldserver::appworld::country::countryhandler::{
    CCountryHandler, CountryHandlerInitializeReport, CountryInfoDeliveryContext,
    CountryRunBlock, CountryRunReport,
};
use crate::worldserver::appworld::country::countryparam::{
    CCountryParam, CountryParamLoadError, CountryParamLoadReport,
};
use crate::worldserver::appworld::country::countrywarsys::{
    CountryWarCallbackKind, CountryWarCallbacks, CountryWarDeclarationAuthority,
    CountryWarDeclarationContext, CountryWarDeclarationPlayer, CountryWarFinishBlock,
    CountryWarFinishReport, CountryWarLoadError, CountryWarLoadReport, CountryWarPhase,
    CountryWarPhaseBlock, CountryWarPhaseContext, CountryWarPhaseReport, CountryWarReloadBlock,
    CountryWarStartBlock, CountryWarStartReport, CountryWarSys, CountryWarTopInfoBlock,
    CountryWarTopInfoContext, CountryWarTopInfoKind, CountryWarTopInfoReport,
    CountryWarVictoryContext, CountryWarVictoryRegion,
};
use crate::worldserver::appworld::goods::cgoods::CGoods;
use crate::worldserver::appworld::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, GoodsOriginalNameIndex,
};
use crate::worldserver::appworld::goodswarmember::{
    CGoodsWarMember, GoodsWarDatabaseLoadReport, GoodsWarDeliveryContext,
    GoodsWarMemberBlock,
};
use crate::worldserver::appworld::jjcsystem::{
    CJJcSystem, JJC_CONFIG_PATH, JJC_LEVEL_LIST_PATH, JJC_REGION_LIST_PATH,
    JjcConfigurationLoadReport, JjcRunBlock, JjcRunConfig, JjcRunContext, JjcRunReport,
};
use crate::worldserver::appworld::organizingsystem::fournationwarsys::{
    CFourNationWarSys, FourNationCountryFailContext, FourNationExploitContext,
    FourNationExploitLoadedDisposition, FourNationWarResultContext,
};
use crate::worldserver::appworld::leiting::{
    CLeiTing, LeiTingBlock, LeiTingContext, LeiTingLocalTime, LeiTingRunReport,
};
use crate::worldserver::appworld::message::othermessage::{
    WorldOtherMessageDispatch, WorldOtherMessageOutcome, on_other_message,
};
use crate::worldserver::appworld::message::jjcsysmessage::{
    JjcSystemMessageOutcome, on_jjc_system_message,
};
use crate::worldserver::appworld::message::countrymessage::{
    WorldCountryMessageDispatch, WorldCountryMessageOutcome,
    WorldFourNationExploitDatabaseDisposition, WorldFourNationExploitSync,
    decode_four_nation_exploit_message,
    dispatch_country_absolve_request_message,
    dispatch_country_appoint_minister_message,
    dispatch_country_demise_message,
    dispatch_country_depose_minister_message,
    dispatch_country_direct_appointment_message,
    dispatch_country_new_day_message,
    dispatch_country_player_change_message,
    dispatch_country_exile_result_message,
    dispatch_country_exile_request_message,
    dispatch_country_info_message,
    dispatch_country_players_list_message,
    dispatch_country_silence_request_message,
    dispatch_country_war_declaration_message, dispatch_country_war_victory_message,
    dispatch_four_nation_country_fail_message, dispatch_four_nation_war_result_message,
    dispatch_four_nation_war_time_message, on_country_message,
};
use crate::worldserver::appworld::message::gmamessage::{
    WorldGmaMessageDispatch, WorldGmaMessageOutcome, on_gma_message,
};
use crate::worldserver::appworld::message::gmmessage::{
    WorldGmMessageDispatch, WorldGmMessageOutcome, on_gm_message,
};
use crate::worldserver::appworld::message::logmessage::{
    WorldLogMessageDispatch, WorldLogMessageOutcome, on_log_message,
};
use crate::worldserver::appworld::message::playermessage::{
    WorldPlayerMessageDispatch, WorldPlayerMessageOutcome, on_player_message,
};
use crate::worldserver::appworld::message::organsysmessage::{
    CityTransferConfirmationDelivery, ConfederationCreationConfirmationDelivery,
    OrganizingAdmissionPermitBlock,
    OrganizingAdmissionPermitDispatch, OrganizingAttackCityEndDispatch,
    OrganizingCityGateBlock, OrganizingCityGateDispatch, OrganizingCityTransferDispatch,
    OrganizingCityWarApplicationBlock, OrganizingCityWarApplicationDispatch,
    OrganizingChangeRegionRouterDispatch, OrganizingCityWarResultDispatch,
    OrganizingConsumedLongDispatch,
    OrganizingDeclareFactionWarBlock,
    OrganizingDeclareFactionWarDispatch, OrganizingDeclareWarFactionListBlock,
    OrganizingDeclareWarFactionListDispatch, OrganizingFactionBillboardBlock,
    OrganizingFactionApplicationCancelBlock, OrganizingFactionApplicationCancelDispatch,
    OrganizingFactionApplicationDispatch, OrganizingFactionApplicationDispatchBlock,
    OrganizingFactionApplicationDecisionDispatch,
    OrganizingFactionListBlock, OrganizingFactionListDispatch,
    OrganizingFactionBillboardOutcome, OrganizingFactionContributorDispatch,
    OrganizingFactionExperienceDispatch, OrganizingFactionMemberStateDispatch,
    OrganizingFactionDubBlock, OrganizingFactionDubDispatch,
    OrganizingFactionPurviewBlock, OrganizingFactionPurviewDispatch,
    OrganizingFactionDemiseBlock, OrganizingFactionDemiseDispatch,
    OrganizingFactionDisbandBlock, OrganizingFactionDisbandDispatch,
    OrganizingFactionFireOutBlock, OrganizingFactionFireOutDispatch,
    OrganizingFactionWarPlayerDiedDispatch,
    OrganizingInitialDataDispatch,
    OrganizingFactionExitBlock, OrganizingFactionExitDispatch,
    OrganizingUnionDemiseDispatch,
    OrganizingUnionDisbandBlock, OrganizingUnionDisbandDispatch,
    OrganizingUnionExitDispatch,
    OrganizingUnionFireOutDispatch,
    OrganizingFactionTaxBlock, OrganizingFactionTaxDispatch, OrganizingFactionUpgradeBlock,
    OrganizingFactionUpgradeDispatch, OrganizingFactionUploadIconDispatch,
    OrganizingRegionParamDispatch, OrganizingRegionRouteDispatch,
    OrganizingGoodsWarCommandDispatch, OrganizingGoodsWarContextBlock,
    OrganizingGoodsWarFactionWinBlock, OrganizingGoodsWarFactionWinDispatch,
    OrganizingPlayerQuestCommandDispatch,
    OrganizingPlayerRunScriptDispatch,
    OrganizingFactionParameterBlock, OrganizingFactionParameterDispatch,
    OrganizingVillageWarApplicationBlock, OrganizingVillageWarApplicationDispatch,
    OrganizingVillageWarResultDispatch,
    OrganizingLeaveWordDispatch,
    OrganizingLeaveWordEditDispatch, OrganizingLeaveWordEnableDispatch,
    OrganizingCreateFactionBlock, OrganizingCreateFactionDispatch,
    OrganizingPronounceDispatch, OrganizingSessionResultDispatch,
    OrganizingPlayerInviteFactionDispatch, OrganizingUnionApplicationDispatch,
    QueuedCityTransferTerminal, QueuedConfederationCreationTerminal,
    QueuedOrganizingSessionTerminal,
    QueuedUnionApplicationTerminal, QueuedUnionInvitationTerminal,
    UnionApplicationConfirmationDelivery,
    WorldUnionApplicationEffectCallbacks, WorldUnionApplicationEffects,
    WorldUnionApplicationRuntimeOwner, dispatch_admission_permit, dispatch_attack_city_end,
    dispatch_city_gate, dispatch_city_transfer, dispatch_city_war_application,
    dispatch_change_region_router, dispatch_city_war_result,
    dispatch_consumed_long, dispatch_create_faction, dispatch_declare_faction_war,
    dispatch_faction_war_player_died,
    dispatch_initial_organizing_data,
    dispatch_declare_war_faction_list, dispatch_faction_application,
    dispatch_faction_application_decision,
    dispatch_faction_dub,
    dispatch_faction_purview,
    dispatch_faction_demise,
    dispatch_faction_disband,
    dispatch_faction_fire_out,
    dispatch_faction_exit,
    dispatch_union_demise,
    dispatch_union_disband,
    dispatch_union_exit,
    dispatch_union_fire_out,
    dispatch_faction_billboard, dispatch_faction_list,
    dispatch_faction_application_cancel,
    dispatch_faction_upgrade,
    dispatch_faction_contributor, dispatch_faction_experience, dispatch_faction_member_state,
    dispatch_faction_tax, dispatch_faction_upload_icon,
    dispatch_goods_war_command, dispatch_goods_war_faction_win,
    dispatch_player_quest_command,
    dispatch_player_invite_faction, dispatch_player_run_script,
    dispatch_faction_parameter,
    dispatch_leave_word, dispatch_leave_word_edit,
    dispatch_leave_word_enable, dispatch_organizing_session_result, dispatch_pronounce,
    dispatch_region_param_update, dispatch_region_route, dispatch_union_application,
    finalize_faction_disband_dispatch,
    dispatch_village_war_application, dispatch_village_war_result,
};
use crate::worldserver::appworld::message::servermessage::{
    WorldLoginClientReplacement, WorldServerMessageDispatch, WorldServerMessageError,
    WorldServerMessageOutcome, on_login_client_reconnected, on_server_message,
};
use crate::worldserver::appworld::message::teammessage::{
    WorldTeamMessageOutcome, on_team_message,
};
use crate::worldserver::appworld::message::writelogmessage::{
    WorldWriteLogCommand, WorldWriteLogMessageDispatch, WorldWriteLogMessageOutcome,
    on_write_log_message,
};
use crate::worldserver::appworld::misc::{
    CopyNumberResetReport, CopyNumberScheduleBlock, CopyNumberScheduleReport,
    CopyNumberTimerState,
};
use crate::worldserver::appworld::incrementlog::incrementlog::{
    CIncrementLog, IncrementLogLoadOutcome,
};
use crate::worldserver::appworld::organizingsystem::faction::{
    goods_war_check_for_faction_id, CFaction, FactionDemiseContext, FactionDemiseOutcome,
    FactionDisbandContext, FactionExperienceBlock, FactionMemberInfoRequest,
    FactionInitialPropertyBlock, FactionOrganizingInfoContext, FactionUploadIconBlock,
};
use crate::worldserver::appworld::organizingsystem::attackcitysys::{
    AttackCityCallbacks, CAttackCitySys,
};
use crate::worldserver::appworld::organizingsystem::factionwarsys::{
    CFactionWarSys, FactionWarRunReport, FactionWarStopBlock, FactionWarStopContext,
};
use crate::worldserver::appworld::organizingsystem::organizingctrl::{
    AttackCityEndBlock, COrganizingCtrl, CityTransferEndpointBlock, CityTransferFinishBlock,
    CityTransferFinishReport, CityTransferSessionBlock, CityTransferSessionReport,
    CityTransferStartBlock, ConfederationCreationCallbackBlock,
    ConfederationCreationCallbackReport, ConfederationCreationEndpointBlock,
    ConfederationCreationSessionBlock, ConfederationCreationSessionReport,
    OrganizingContributorBlock, OrganizingDisbandOutcome,
    OrganizingDisbandPlayer, OrganizingRunBlock, OrganizingRunReport, OrganizingSaveDataBlock,
    OrganizingLeaveWordBlock, OrganizingLeaveWordEditBlock, OrganizingLeaveWordEnableBlock,
    OrganizingFactionDoJoinBlock,
    OrganizingUnionDemiseBlock, OrganizingUnionExitBlock, OrganizingUnionFireOutBlock,
    OrganizingNameLookupBlock,
    OrganizingPronounceBlock, OrganizingSaveDataReport, OrganizingUnionApplicationCallbackBlock,
    OrganizingUnionApplicationCallbackReport, OrganizingUnionApplyForJoinDispatchBlock,
    OrganizingUnionInvitationCallbackBlock, OrganizingUnionInvitationCallbackReport,
    FreeFactionLookup, FreePlayerLookup, PlayerEnterGameOutcome, PlayerExitGameOutcome,
    PlayerInviteFactionBlock,
};
use crate::worldserver::appworld::organizingsystem::organizingparam::{
    COrganizingParam, OrganizingParamLoadError, OrganizingParamLoadReport,
    OrganizingTaxScheduleBlock, OrganizingTodayTaxRefreshReport, PreparedTodayTaxRefresh,
};
use crate::worldserver::appworld::organizingsystem::union::{
    CUnion, UnionApplicationEndpointBlock, UnionApplicationSessionBlock,
    UnionApplicationSessionReport, UnionFormatArgument,
};
use crate::worldserver::appworld::organizingsystem::villagewarsys::{
    CVillageWarSys, VillageWarCallbacks,
};
use crate::worldserver::appworld::player::{
    CPlayer, PlayerCodecError, PlayerCountryChangeReport, PlayerExploitUpdate,
    PlayerDbProjectionBlock, PlayerEquipmentWireSnapshot,
    PlayerFactionInfoContext, PlayerFactionInfoDelivery, PlayerFactionInfoUpdateBlock,
    PlayerFactionInfoUpdateReport,
    PlayerLoadDataOutcome, PlayerLoadDataOwner,
    PlayerLeiTingClock, PlayerLeiTingUpdateBlock, PlayerLeiTingUpdateReport,
    PlayerMurderCounterReset, PlayerMurderCounterUpdate, PlayerOrganizingUpdateError,
    PlayerOrganizingState, PlayerOrganizingUpdater, PlayerOriginEquipmentBlock,
    PlayerOriginEquipmentOutcome, PlayerPropertyCoefficients,
};
use crate::worldserver::appworld::region::{CRegion, RegionSerializationBlock};
use crate::worldserver::appworld::script::variablelist::{
    CVariableList, VariableListSaveSource,
};
use crate::worldserver::appworld::session::csessionfactory::{
    CSessionFactory, WorldSessionFactoryAiReport,
};
use crate::worldserver::appworld::worldcityregion::{
    CWorldCityRegion, WorldCityRegionLoadError, WorldCityRegionSerializationBlock,
};
use crate::worldserver::appworld::worldcountrywarregion::{
    WorldCountryWarRegion, WorldCountryWarRegionLoadError, WorldCountryWarRegionSerializationBlock,
};
use crate::worldserver::appworld::worldregion::{
    CWorldRegion, WorldRegionLoadError, WorldRegionLoadedCounts, WorldRegionResourceContext,
    WorldRegionOwnerRelationBlock, WorldRegionOwnerRelationReport, WorldRegionParamDecodeError,
    WorldRegionSerializationBlock, WorldRegionSetupSerializationBlock,
};
use crate::worldserver::appworld::worldvillageregion::CWorldVillageRegion;
use crate::worldserver::appworld::worldwarregion::WorldWarRegionSerializationBlock;
use crate::worldserver::worldserver::honorranks::{
    CHonorRanks, HonorRanksNewDayBlock, HonorRanksNewDayReport,
};
use crate::worldserver::worldserver::playerranks::{
    CPlayerRanks, PlayerRanksGameServerUpdate, PlayerRanksInitializationConfig,
    PlayerRanksInitializationReport, PlayerRanksReleaseReport, PlayerRanksScheduleBlock,
    PlayerRanksSerializationBlock,
};
use crate::worldserver::worldserver::savedb::{
    DoSaveDataLifecycleReport, SaveDataFinalDisposition, SaveDataLifecycleState, SaveDataLogEvent,
    SaveDataLogPublishBlock, SaveDataLogPublishDisposition, SaveDataLogSink, SaveDataLogTarget,
    SaveDataMonitoringReport, SaveDataMonitoringSnapshot, do_save_data_lifecycle,
};
use crate::worldserver::worldserver::worldserver::{
    AddLogTextDisposition, WorldLogLocalTime, WorldLogTextOwner, WorldRefreshInfoCurrent,
    WorldRefreshInfoHighWater, WorldRefreshInfoReport, WorldRefreshSaveState, refresh_info_text,
};
use crate::worldserver::worldserver::writelogworker::WorldWriteLogWorkerSpec;

/// Источник, который исходный World `LoadSetup` смог открыть первым.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldSetupSource {
    Plain,
    Encoded,
}

/// Итог positional-чтения и последующей попытки закрепить instance title.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldSetupLoadReport {
    pub(crate) source: WorldSetupSource,
    pub(crate) parsed_pairs: usize,
    pub(crate) stopped_at_pair: Option<usize>,
    pub(crate) instance_title: Vec<u8>,
    pub(crate) instance_claimed: bool,
}

/// Нечувствительный итог чтения `serverSetup.ini` без публикации адресов.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldServerSetupLoadReport {
    /// Signed число записей после header либо исходный инициализированный ноль.
    pub(crate) declared_records: i32,
    /// Число записей, для которых старый `map::operator[]` был воспроизведён.
    pub(crate) applied_records: usize,
    /// Итоговое число уникальных ключей с учётом прежних записей и замен.
    pub(crate) unique_game_servers: usize,
    /// Не перешёл ли formatted stream в fail/EOF-state.
    pub(crate) stream_complete: bool,
    /// Первая запись, map-key которой зависел бы от неизвестного stack-значения.
    pub(crate) blocked_at_record: Option<usize>,
    /// Достигнута ли исходная позиция `AddLogText("GS Setup Read END.")`.
    pub(crate) read_end_notice: bool,
}

/// Локальная safe-граница адресного поиска в GameServer registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldGameServerLookupError {
    /// Совпавший IP достиг поля port, которое старый loader не назначил.
    PortUnavailable { index: u32 },
}

impl fmt::Display for WorldGameServerLookupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PortUnavailable { index } => write!(
                formatter,
                "у GameServer {index} не назначен port для точного сравнения"
            ),
        }
    }
}

impl Error for WorldGameServerLookupError {}

#[derive(Clone, Copy)]
struct WorldNetworkConfig {
    ban_ip_time_ms: u32,
    maximum_client_send_buffer: i32,
    maximum_message_length: u32,
    maximum_byte_count: u32,
    check_message_content: bool,
    maximum_connections: i32,
    maximum_io_sends: i32,
    check_net: bool,
}

/// Ошибка достигнутой сетевой границы World `CGame::InitNetServer`.
#[derive(Debug)]
pub(crate) enum WorldNetworkInitializationError {
    /// Setup-чтение не назначило поле, которое исходник затем читал.
    MissingSetupField(&'static str),
    /// Общий listener не смог выполнить доказанный bind/listen.
    Host(ServerHostError),
}

impl fmt::Display for WorldNetworkInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSetupField(field) => {
                write!(formatter, "World setup не назначил поле {field}")
            }
            Self::Host(error) => write!(formatter, "World listener не запущен: {error}"),
        }
    }
}

impl Error for WorldNetworkInitializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingSetupField(_) => None,
            Self::Host(error) => Some(error),
        }
    }
}

/// Успешный итог World `InitNetClient`; старый return при этом равен `1`.
#[derive(Debug)]
pub(crate) struct WorldClientInitialization {
    /// Первый IPv4 endpoint, переданный общему десятисекундному connect.
    pub(crate) endpoint: SocketAddrV4,
    /// Исходно игнорировавшийся результат постановки `0x1FE01`.
    pub(crate) registration: Result<i32, SendMessageError>,
}

/// Ошибка достигнутой initial World-to-Login границы.
#[derive(Debug)]
pub(crate) enum WorldClientInitializationError {
    /// Setup-чтение не назначило поле, которое исходник затем читал.
    MissingSetupField(&'static str),
    /// `char[64]` старого `CClient::Connect` был бы переполнен.
    LoginAddressTooLongReactionUnknown { length: usize },
    /// ANSI hostname нельзя без доказательства преобразовать в Linux resolver.
    LoginAddressEncodingUnsupported,
    /// `inet_addr/gethostbyname` не дали пригодный IPv4 endpoint.
    LoginAddressResolution,
    /// Не создан и не bind-нут исходящий IPv4 socket.
    Bind(io::Error),
    /// Общий десятисекундный connect завершился неуспешно.
    Connect(ClientConnectError),
}

impl fmt::Display for WorldClientInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSetupField(field) => {
                write!(formatter, "World setup не назначил поле {field}")
            }
            Self::LoginAddressTooLongReactionUnknown { length } => write!(
                formatter,
                "LoginServer-адрес длиной {length} bytes выходит за старый char[64]"
            ),
            Self::LoginAddressEncodingUnsupported => formatter.write_str(
                "кодировка LoginServer-адреса не поддерживается безопасным Linux resolver",
            ),
            Self::LoginAddressResolution => {
                formatter.write_str("LoginServer-адрес не разрешён в IPv4")
            }
            Self::Bind(error) => write!(formatter, "не создан World-to-Login socket: {error}"),
            Self::Connect(error) => {
                write!(formatter, "WorldServer не подключён к LoginServer: {error}")
            }
        }
    }
}

impl Error for WorldClientInitializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Bind(error) => Some(error),
            Self::Connect(error) => Some(error),
            Self::MissingSetupField(_)
            | Self::LoginAddressTooLongReactionUnknown { .. }
            | Self::LoginAddressEncodingUnsupported
            | Self::LoginAddressResolution => None,
        }
    }
}

/// DB snapshot исходного семиаргументного `CMyAdoBase::Initialize`.
///
/// Тип намеренно не реализует `Debug`, чтобы credentials не попали в logs.
pub(crate) struct WorldGameDatabaseInitialization {
    pub(crate) settings: WorldDatabaseSettings,
    pub(crate) connection_type: Vec<u8>,
    pub(crate) legacy_zero: &'static [u8],
    pub(crate) integrated_security: &'static [u8],
}

/// DB owners, которые `CGame::Init` создавал вокруг отдельного `CRsSetup`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldGameDatabaseOwner {
    RsPlayer,
    RsGenVar,
    RsFaction,
    RsUnion,
    RsEnemyFactions,
    RsVillageWar,
    RsCityWar,
    RsRegion,
    DbCountry,
    GoodsWarMember,
    DbMisc,
    RsGodsBattle,
}

/// Void initialization calls, границы которых принадлежат соседним owners.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldGameInitVoidOwner {
    InitializeLargess,
    LoadGodsBattleFactionXyd,
    InitializeAttackCityEnemyRelations,
    InitializeOrganizingController,
    InitializeFactionWar,
    InitializeQuestSystem,
    CreateGeneralVariableList,
    LoadGeneralVariableList,
    LoadGeneralVariableData,
    InitializeBaseMessage,
    InitializeSocket,
}

/// Boolean initialization calls с доказанным caller-решением.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldGameInitBooleanOwner {
    InitializeTimeToReturn,
    InitializeAttackCity,
    InitializeFourNationWar,
    InitializeVillageWar,
}

/// Opaque результат одного старого `__beginthreadex` call-site.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldGameInitWorkerHandleState {
    Open,
    Empty,
}

/// Тип запускаемого worker-а в финальном участке `CGame::Init`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldGameInitWorkerKind {
    WriteLog,
    LoadPlayerData { worker_index: u32 },
}

/// Один операторский notice вместо Windows `MessageBoxA`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldGameInitOperatorNotice {
    pub(crate) title: Vec<u8>,
    pub(crate) message: Vec<u8>,
}

/// Ordered наблюдаемый эффект полного initialization owner-а.
#[derive(Debug)]
pub(crate) enum WorldGameInitEvent {
    CrashReporterInstalled,
    RandomInitialized {
        seed: u32,
        discarded_roll: i32,
    },
    RustLocksReady,
    DebugStartPublished,
    ServerResourcesLoaded,
    SetupLoaded(WorldSetupLoadReport),
    ServerSetupLoaded(WorldServerSetupLoadReport),
    PlayerLoadThreadCountValidated(u32),
    StringTablesCleared,
    StringTableLoaded {
        package: Vec<u8>,
    },
    StringTablesCoded,
    DupliRegionSetupLoaded,
    DatabaseLayerInitialized,
    DatabaseOwnerCreated(WorldGameDatabaseOwner),
    GoodsWarMemberLoaded(GoodsWarDatabaseLoadReport),
    RsSetupOwnerCreated(LoadedSetupIds),
    VoidOwner(WorldGameInitVoidOwner),
    JjcConfigurationLoaded(JjcConfigurationLoadReport),
    Reload {
        profile: &'static [u8],
        legacy_result: i32,
    },
    RegionParametersLoaded {
        succeeded: bool,
    },
    WordsFilterInitialized,
    BooleanOwner {
        owner: WorldGameInitBooleanOwner,
        succeeded: bool,
    },
    OrganizingParametersLoaded(OrganizingParamLoadReport),
    CountryParametersLoaded(CountryParamLoadReport),
    CountryHandlerInitialized(CountryHandlerInitializeReport),
    CountryWarInitialized(CountryWarLoadReport),
    RegionOwnerRelationInitialized {
        region_id: i32,
        report: WorldRegionOwnerRelationReport,
    },
    PlayerRanksInitialized(PlayerRanksInitializationReport),
    PlayerRanksLoaded(PlayerRanksStatRunReport),
    HonorRanksLoaded {
        started_at_ms: u32,
        finished_at_ms: u32,
        elapsed_ms: u32,
        outcome: HonorRanksLoadOutcome,
    },
    IncrementLogLoaded {
        outcome: IncrementLogLoadOutcome,
    },
    AuctionLogLoaded {
        outcome: AuctionLogLoadOutcome,
    },
    Log {
        payload: Vec<u8>,
        disposition: AddLogTextDisposition,
    },
    NetworkClientInitialized(WorldClientInitialization),
    NetworkServerInitialized,
    PlayerDataQueueCleared,
    CopyNumberResetScheduled(CopyNumberScheduleReport),
    WorkerStarted {
        kind: WorldGameInitWorkerKind,
        handle: WorldGameInitWorkerHandleState,
    },
    OperatorNotice(WorldGameInitOperatorNotice),
}

/// Safe-граница единственного legacy `long` count string-table wire.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldStringTableEncodingBlock {
    pub(crate) entry_count: usize,
}

/// Наблюдаемый результат exact `CGame::LoadStringTable`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldStringTableLoadReport {
    pub(crate) package: Vec<u8>,
    pub(crate) succeeded: bool,
    pub(crate) log_payload: Vec<u8>,
}

/// Последняя достигнутая ветвь `CGame::UpdateStringTable`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldStringTableUpdateCompletion {
    DefaultLanguageFailed,
    ConfiguredLanguageFailed,
    EncodingBlocked(WorldStringTableEncodingBlock),
    Empty,
    Broadcast {
        message_type: i32,
        payload_length: usize,
        delivery: Result<i32, SendMessageError>,
    },
}

/// Полный результат reload-а; `requested_package` фиксирует игнорируемый
/// исходной функцией аргумент вместо того, чтобы молча приписать ему смысл.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldStringTableUpdateReport {
    pub(crate) requested_package: Vec<u8>,
    pub(crate) completion: WorldStringTableUpdateCompletion,
}

/// Первая fail/nonreturn граница полного `CGame::Init`.
#[derive(Debug)]
pub(crate) enum WorldGameInitBlockReason<ContextBlock> {
    SetupOpen(WorldSetupOpenError),
    ExistingInstance { title: Vec<u8> },
    ServerSetup(io::Error),
    MissingPlayerLoadThreadCount,
    InvalidPlayerLoadThreadCount { count: u32, legacy_exit_code: i32 },
    DefaultLanguageTable,
    ConfiguredLanguageTable,
    StringTableEncoding(WorldStringTableEncodingBlock),
    DupliRegionSetup,
    Context(ContextBlock),
    Reload(WorldReloadBlock),
    JjcConfiguration(JjcConfigurationLoadReport),
    BooleanOwner(WorldGameInitBooleanOwner),
    OrganizingParameters(OrganizingParamLoadError),
    CountryParameters(CountryParamLoadError),
    CountryHandler,
    CountryWarLoad(CountryWarLoadError),
    CountryWar,
    RegionOwnerRelation {
        region_id: i32,
        source: WorldRegionOwnerRelationBlock,
    },
    PlayerRanksSchedule(PlayerRanksScheduleBlock),
    PlayerRanksStat(PlayerRanksStatRunBlock),
    NetworkClient(WorldClientInitializationError),
    NetworkServer(WorldNetworkInitializationError),
    CopyNumberSchedule(CopyNumberScheduleBlock),
}

/// Completed prefix и точная причина, по которой Init не вернул legacy `1`.
#[derive(Debug)]
pub(crate) struct WorldGameInitBlock<ContextBlock> {
    pub(crate) events: Vec<WorldGameInitEvent>,
    pub(crate) reason: WorldGameInitBlockReason<ContextBlock>,
}

/// Полностью успешный `CGame::Init` и исходный return value.
#[derive(Debug)]
pub(crate) struct WorldGameInitReport {
    pub(crate) events: Vec<WorldGameInitEvent>,
    pub(crate) legacy_result: i32,
}

pub(crate) type WorldGameInitResult<ContextBlock> =
    Result<WorldGameInitReport, Box<WorldGameInitBlock<ContextBlock>>>;

/// Прямые ещё сырые owners, достигнутые полным `CGame::Init`.
pub(crate) trait WorldGameInitContext: WorldReloadContext {
    type Block;
    type PlayerDatabase: RsPlayerOwner;

    fn install_crash_reporter(&mut self);
    fn current_time_seconds(&mut self) -> i64;
    fn seed_random(&mut self, seed: u32);
    fn random(&mut self, upper_bound: i32) -> i32;
    fn put_debug_string(&mut self, payload: &[u8]);
    fn load_server_resources(&mut self, game: &mut CGame);

    /// Возвращает `true`, если Linux single-instance owner закрепил title.
    fn claim_single_instance(&mut self, title: &[u8]) -> bool;
    fn notify_operator(&mut self, notice: &WorldGameInitOperatorNotice);

    fn initialize_database_layer(
        &mut self,
        initialization: WorldGameDatabaseInitialization,
    ) -> Result<(), Self::Block>;
    fn create_database_owner(&mut self, owner: WorldGameDatabaseOwner) -> Result<(), Self::Block>;
    fn create_rs_setup_owner(&mut self) -> Result<LoadedSetupIds, Self::Block>;

    fn initialize_void_owner(&mut self, owner: WorldGameInitVoidOwner);
    fn initialize_boolean_owner(&mut self, owner: WorldGameInitBooleanOwner) -> bool;
    fn load_region_parameters(&mut self, game: &mut CGame) -> bool;
    fn country_parameter_source(&mut self) -> Option<Vec<u8>>;
    fn country_war_source(&mut self) -> Option<Vec<u8>>;
    fn use_appellation_function(&mut self) -> bool;
    /// Возвращает достигнутый player DB-owner и его текущий caller-connection.
    fn player_database(
        &mut self,
    ) -> (&mut Self::PlayerDatabase, Option<&mut WorldTdsClient>);
    /// Возвращает уже открытый Log DB connection техническому increment-owner-у.
    fn increment_log_database(&mut self) -> Option<&mut WorldTdsClient>;
    /// Возвращает уже открытый Log DB connection техническому auction-owner-у.
    fn auction_log_database(&mut self) -> Option<&mut WorldTdsClient>;
    /// Exact `CGlobeSetup::m_stSetup.dwIncrementLogDays` для обеих history query.
    fn increment_log_days(&mut self) -> u32;
    /// Регистрирует exact calendar-цепочку `ClearCopyNum` на owned timer-е.
    fn register_clear_copy_number_time(
        &mut self,
    ) -> Result<CopyNumberScheduleReport, CopyNumberScheduleBlock>;
    /// Получает concrete Log DB setup/FIFO и сохраняет единственный handle.
    fn start_write_log_worker(
        &mut self,
        worker: WorldWriteLogWorkerSpec,
    ) -> WorldGameInitWorkerHandleState;
    /// Для load-worker передаёт cloneable queue-owner и добавляет даже пустой
    /// handle в исходный ordered owner.
    fn start_player_load_worker(
        &mut self,
        worker: WorldPlayerLoadWorkerSpec,
        worker_index: u32,
    ) -> WorldGameInitWorkerHandleState;
}

/// Clock/log и узкий player-refresh adapters полного Init.
pub(crate) struct WorldGameInitCallbacks<'a> {
    pub(crate) get_tick: &'a mut dyn FnMut() -> u32,
    pub(crate) get_log_local_time: &'a mut dyn FnMut() -> WorldLogLocalTime,
    pub(crate) get_timer_local_time: &'a mut dyn FnMut() -> TagTime,
    pub(crate) put_log_info: &'a mut dyn FnMut(&[u8]),
    pub(crate) update_player: &'a mut dyn FnMut(i32),
}

/// Локальная safe-граница `SaveCityRegion(0)` до virtual save-вызова.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldSaveCityRegionBlock {
    UninitializedRegionType { region_id: i32 },
    NullCityRegion { region_id: i32 },
}

/// Один live list, который `CGame::Release` очищал до player-map-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldGameReleaseLiveList {
    Creation,
    Restore,
    Deletion,
    Online,
    Offline,
    Login,
}

/// Безусловные соседние release-owner-ы в точном порядке `CGame::Release`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldGameReleaseVoidOwner {
    ReleaseGoodsLinks,
    UninitializeTimeToReturn,
    UninitializeIncrementLog,
    ReleaseCountryHandler,
    ReleaseWordsFilter,
    ReleaseOrganizingController,
    ReleaseAttackCity,
    ReleaseVillageWar,
    ReleaseOrganizingParameters,
    ReleaseQuestSystem,
    ReleaseFactionWar,
    ReleaseTimer,
    ClearSkillCache,
    ClearSkillUsageCache,
    ReleaseGoodsFactory,
    CleanupSocket,
    ReleaseBaseMessage,
    ReleaseNetSessionManager,
    RequestWriteLogWorkerExit,
    UninitializeLargess,
    UninitializeDatabaseLayer,
}

/// Nullable owners, существование которых проверялось перед удалением.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldGameReleaseOptionalOwner {
    FunctionListFileData,
    VariableListFileData,
    ScriptFileData,
    GeneralVariableList,
    DefaultClientResource,
    DupliRegionSetup,
}

/// DB pointers, удаляемые Release; `CDbMisc` намеренно отсутствует в списке.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldGameReleaseDatabaseOwner {
    RsPlayer,
    RsSetup,
    RsGenVar,
    RsFaction,
    RsUnion,
    RsEnemyFactions,
    RsVillageWar,
    RsCityWar,
    GoodsWarMember,
    RsRegion,
    DbCountry,
    RsGodsBattle,
}

/// Один завершённый эффект полного World shutdown-owner-а.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldGameReleaseEvent {
    DebugPublished(&'static [u8]),
    PlayerDataQueueCleared,
    CityRegionSaved {
        region_id: i32,
    },
    NetworkServerWorkerExited,
    NetworkClientWorkerExited,
    LiveListCleared {
        owner: WorldGameReleaseLiveList,
        entries: usize,
    },
    PlayerMapCleared {
        entries: usize,
    },
    DbDataCleared,
    PlayerRanksReleased(PlayerRanksReleaseReport),
    SaveWorkerJoined {
        previous_handle: WorldSaveThreadHandleState,
    },
    VoidOwner(WorldGameReleaseVoidOwner),
    RegionOwnerReleased {
        region_id: i32,
    },
    OptionalOwner {
        owner: WorldGameReleaseOptionalOwner,
        released: bool,
    },
    NetworkClientReleased,
    NetworkServerReleased,
    DatabaseOwner {
        owner: WorldGameReleaseDatabaseOwner,
        released: bool,
    },
    DatabaseMiscRetained,
    RustLocksRetired,
    WriteLogWorkerJoined {
        previous_handle: WorldGameInitWorkerHandleState,
    },
    PlayerLoadWorkersStopped {
        workers: u32,
    },
}

/// Успешный полный `CGame::Release` и его старый return value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldGameReleaseReport {
    pub(crate) events: Vec<WorldGameReleaseEvent>,
    pub(crate) legacy_result: i32,
}

/// Выполненный prefix Release перед единственной локальной unknown-границей.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldGameReleaseBlock {
    pub(crate) events: Vec<WorldGameReleaseEvent>,
    pub(crate) block: WorldSaveCityRegionBlock,
}

pub(crate) type WorldGameReleaseResult = Result<WorldGameReleaseReport, Box<WorldGameReleaseBlock>>;

/// Ещё сырые domain/platform owners, непосредственно достигнутые Release.
pub(crate) trait WorldGameReleaseContext {
    fn put_debug_string(&mut self, payload: &'static [u8]);
    fn save_city_region(&mut self, region_id: i32, region: &mut WorldRegionOwner);
    fn exit_network_server_worker(&mut self, server: &mut CMyNetServer);
    fn exit_network_client_worker(&mut self, client: &mut CMyNetClient);
    fn release_void_owner(&mut self, owner: WorldGameReleaseVoidOwner);
    fn release_optional_owner(&mut self, owner: WorldGameReleaseOptionalOwner) -> bool;
    fn release_database_owner(&mut self, owner: WorldGameReleaseDatabaseOwner) -> bool;
    /// Снимает rank-event с ещё живого timer-а, затем уничтожает rank-owner.
    fn release_player_ranks(&mut self) -> PlayerRanksReleaseReport;

    /// Ждёт и закрывает даже исходный пустой `g_hSavingThread`, затем обнуляет owner.
    fn join_save_worker(&mut self) -> WorldSaveThreadHandleState;
    /// Ждёт/закрывает write-log handle после публикации exit-флага.
    fn join_write_log_worker(&mut self) -> WorldGameInitWorkerHandleState;
    /// Останавливает и закрывает все handles в исходном vector-order.
    fn stop_player_load_workers(&mut self) -> u32;
}

/// Создание нового `g_pGame` запрещено поверх ещё опубликованного owner-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldCreateGameBlock;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldCreateGameReport {
    pub(crate) legacy_result: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldDeleteGameReport {
    pub(crate) owner_was_present: bool,
    pub(crate) legacy_result: i32,
}

/// Причина штатного выхода из do/while части `GameThreadFunc`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldGameThreadStop {
    InitializationFailed,
    ExitRequested,
    MainLoopReturned { legacy_result: i32 },
}

pub(crate) enum WorldGameThreadInitialization<ContextBlock> {
    Complete(WorldGameInitReport),
    Failed(Box<WorldGameInitBlock<ContextBlock>>),
}

/// Полный lifecycle либо safe-граница с сохранённым live game-owner-ом.
pub(crate) enum WorldGameThreadReport<InitBlock, MainLoopBlock> {
    Complete {
        creation: WorldCreateGameReport,
        initialization: WorldGameThreadInitialization<InitBlock>,
        main_loop_calls: u64,
        stop: WorldGameThreadStop,
        release: WorldGameReleaseReport,
        deletion: WorldDeleteGameReport,
        legacy_exit_code: u32,
    },
    FatalInitialization {
        game: Box<CGame>,
        creation: WorldCreateGameReport,
        block: Box<WorldGameInitBlock<InitBlock>>,
    },
    BlockedInitialization {
        game: Box<CGame>,
        creation: WorldCreateGameReport,
        block: Box<WorldGameInitBlock<InitBlock>>,
    },
    BlockedMainLoop {
        game: Box<CGame>,
        creation: WorldCreateGameReport,
        initialization: WorldGameInitReport,
        main_loop_calls: u64,
        block: MainLoopBlock,
    },
    BlockedRelease {
        game: Box<CGame>,
        creation: WorldCreateGameReport,
        initialization: WorldGameThreadInitialization<InitBlock>,
        main_loop_calls: u64,
        stop: WorldGameThreadStop,
        block: Box<WorldGameReleaseBlock>,
    },
}

/// Адаптер готовых `CGame::Init/MainLoop` и process-global lifecycle owners.
pub(crate) trait WorldGameThreadRuntime: WorldGameReleaseContext {
    type InitBlock;
    type MainLoopBlock;

    fn initialize_game<'game>(
        &'game mut self,
        game: &'game mut CGame,
    ) -> Pin<Box<dyn Future<Output = WorldGameInitResult<Self::InitBlock>> + 'game>>;
    fn game_thread_exit_requested(&self) -> bool;
    fn run_main_loop(&mut self, game: &mut CGame) -> Result<i32, Self::MainLoopBlock>;
    fn wait_for_save_barrier(&mut self);
    /// Временно передаёт concrete Goods War owner полному Release.
    fn take_goods_war_member(&mut self) -> CGoodsWarMember;
    fn restore_goods_war_member(&mut self, owner: CGoodsWarMember);
    /// Временно передаёт process-global increment-log owner полному Release.
    fn take_increment_log(&mut self) -> CIncrementLog;
    fn restore_increment_log(&mut self, owner: CIncrementLog);
    fn signal_game_thread_exit(&mut self);
    fn request_window_close(&mut self);
}

/// Успешный итог отдельной попытки World `ReConnectLoginServer`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldLoginReconnect {
    /// Первый IPv4 endpoint, к которому подключён переданный FIFO client.
    pub(crate) endpoint: SocketAddrV4,
}

/// Успешно построенный и поставленный World CD-key snapshot.
#[derive(Debug)]
pub(crate) struct WorldCdkeySnapshot {
    /// Точное 32-битное значение исходного `m_lOnlinePlayer.size()`.
    pub(crate) declared_online_players: u32,
    /// Исходно игнорировавшийся результат приоритетного `CMessage::Send`.
    pub(crate) delivery: Result<i32, SendMessageError>,
}

/// Итог хвостовой вставки online-ID и следующего organizing enter callback-а.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldOnlinePlayerAppendOutcome {
    pub(crate) inserted: bool,
    pub(crate) organizing: PlayerEnterGameOutcome,
}

/// Владение player после одного decode-элемента reconnect-хвоста `0x5FA01`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldReconnectedPlayerOwner {
    Existing,
    Created {
        replaced_existing_decoded_id: bool,
        offline_inserted: bool,
    },
}

/// Итог decode-а одного player snapshot из reconnect-хвоста `0x5FA01`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldReconnectedPlayerDecode {
    pub(crate) requested_player_id: u32,
    pub(crate) decoded_player_id: i32,
    pub(crate) owner: WorldReconnectedPlayerOwner,
}

/// Владение player после subtype `1` server-снимка `0x5FA09`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldServerSnapshotPlayerOwner {
    Existing,
    Created {
        replaced_existing_decoded_id: bool,
    },
}

/// Итог полного player decode из обычной ветки `0x5FA09/1`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldServerSnapshotPlayerDecode {
    pub(crate) requested_player_id: u32,
    pub(crate) decoded_player_id: i32,
    pub(crate) owner: WorldServerSnapshotPlayerOwner,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldReturnedPlayerDecodeOwner {
    Existing,
    Created {
        replaced_existing_decoded_id: bool,
        login_removed: bool,
        online_removal: WorldOnlinePlayerRemoveOutcome,
        offline_inserted: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldReturnedPlayerDecode {
    pub(crate) requested_player_id: u32,
    pub(crate) decoded_player_id: i32,
    pub(crate) owner: WorldReturnedPlayerDecodeOwner,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldReturnedPlayerSnapshot {
    pub(crate) account: Vec<u8>,
    pub(crate) name: Vec<u8>,
    pub(crate) level: u8,
    pub(crate) team_id: i32,
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) friend_names: Vec<Vec<u8>>,
}

/// Состояние `m_nDBResponsed` после одного server opcode `0x5FA03`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldPlayerSaveResponseProgress {
    pub(crate) previous_responses: i32,
    pub(crate) completion_counted: bool,
    pub(crate) responses_before_reset: i32,
    pub(crate) connected_game_servers: i32,
    pub(crate) save_triggered: bool,
}

/// Итог `list::remove` online-ID и следующего organizing exit callback-а.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldOnlinePlayerRemoveOutcome {
    pub(crate) removed_occurrences: usize,
    pub(crate) organizing: PlayerExitGameOutcome,
}

/// Один player state-transition из exact `CGame::OnGameServerLost`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldLostGameServerPlayer {
    pub(crate) player_id: u32,
    pub(crate) player_name: Vec<u8>,
    pub(crate) online_removal: WorldOnlinePlayerRemoveOutcome,
    pub(crate) login_removed: bool,
    pub(crate) offline_inserted: bool,
}

/// Полный достигнутый результат отключения одного GameServer.
#[derive(Debug)]
pub(crate) struct WorldGameServerLostReport {
    pub(crate) game_server_index: u32,
    pub(crate) affected_region_ids: Vec<i32>,
    pub(crate) skipped_null_region_owners: usize,
    pub(crate) players: Vec<WorldLostGameServerPlayer>,
    pub(crate) login_notice_type: i32,
    pub(crate) login_notice_delivery: Result<i32, SendMessageError>,
}

/// State-переход живого игрока из server opcode `0x5FA02`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldRegionChangePlayerTransition {
    pub(crate) requested_player_id: u32,
    pub(crate) decoded_player_id: u32,
    pub(crate) target_region_id: i32,
    pub(crate) tile_x: i32,
    pub(crate) tile_y: i32,
    pub(crate) direction: i32,
    pub(crate) direction_applied: bool,
    pub(crate) team_id: i32,
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) offline_removal_completed: bool,
    pub(crate) online_removal: WorldOnlinePlayerRemoveOutcome,
    pub(crate) login_time_ms: u32,
}

/// Безопасная граница достигнутого `CGame::SendCdkeyToLoginServer`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldCdkeySnapshotError {
    /// Setup-чтение не назначило исходно неинициализированный `dwNumber`.
    MissingWorldNumber,
    /// Rust-коллекция вышла за 32-битный размер старого MSVC списка.
    OnlinePlayerCountOutsideLegacyRange { count: usize },
    /// Online-list нарушил исходный инвариант наличия owning map-entry.
    MissingPlayerOwner { player_id: u32 },
}

impl fmt::Display for WorldCdkeySnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingWorldNumber => {
                formatter.write_str("World setup не назначил поле dwNumber")
            }
            Self::OnlinePlayerCountOutsideLegacyRange { count } => write!(
                formatter,
                "online-list содержит {count} записей вне 32-битного диапазона оригинала"
            ),
            Self::MissingPlayerOwner { player_id } => write!(
                formatter,
                "online player {player_id} отсутствует в owning m_mPlayer"
            ),
        }
    }
}

impl Error for WorldCdkeySnapshotError {}

/// Источник сообщения в двух последовательных FIFO `ProcessMessage`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldMessageSource {
    GameServer,
    LoginServer,
}

/// Владелец, выбранный точным numeric selector `CMessage::Run`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldMessageOwner {
    Server,
    Log,
    Gma,
    Player,
    Other,
    Gm,
    Team,
    OrganizingSystem,
    WriteLog,
    Country,
    ServerAuction,
    JjcSystem,
    MiscAuction,
}

/// Результат exact duplicate-ledger gate ветки `0x5FD0D`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldHonorEliminatorRegistration {
    MissingOnlinePlayer,
    Duplicate,
    Accepted,
}

/// Safe-граница обязательного `s_pNetServer` для локальных World-сообщений.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldLocalMessageQueueBlock {
    pub(crate) message_type: i32,
}

impl fmt::Display for WorldLocalMessageQueueBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "локальное World-сообщение {:#08X} не поставлено: s_pNetServer отсутствует",
            self.message_type
        )
    }
}

impl Error for WorldLocalMessageQueueBlock {}

/// Сообщение, для которого `Run` выбрал owner, но сам owner ещё не исполнен.
pub(crate) struct RoutedWorldMessage {
    pub(crate) source: WorldMessageSource,
    pub(crate) message_type: i32,
    pub(crate) owner: Option<WorldMessageOwner>,
    pub(crate) legacy_run_result: i32,
    /// Полное сообщение остаётся owned до реализации выбранного handler-а.
    pub(crate) message: CMessage,
}

impl fmt::Debug for RoutedWorldMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RoutedWorldMessage")
            .field("source", &self.source)
            .field("message_type", &self.message_type)
            .field("owner", &self.owner)
            .field("legacy_run_result", &self.legacy_run_result)
            .field("wire_length", &self.message.as_wire_bytes().len())
            .finish()
    }
}

pub(crate) type WorldUnionApplicationStartBlock =
    OrganizingUnionApplyForJoinDispatchBlock<UnionApplicationSessionBlock>;
pub(crate) type WorldPlayerInviteFactionDispatch = OrganizingPlayerInviteFactionDispatch<
    ConfederationCreationSessionReport,
    UnionApplicationSessionReport,
    UnionApplicationSessionReport,
>;
pub(crate) type WorldPlayerInviteFactionStartBlock = PlayerInviteFactionBlock<
    ConfederationCreationSessionBlock,
    UnionApplicationSessionBlock,
    UnionApplicationSessionBlock,
>;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldUnionApplicationTerminalDispatch {
    pub(crate) request: QueuedUnionApplicationTerminal,
    pub(crate) outcome: Result<
        OrganizingUnionApplicationCallbackReport,
        OrganizingUnionApplicationCallbackBlock,
    >,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldUnionInvitationTerminalDispatch {
    pub(crate) request: QueuedUnionInvitationTerminal,
    pub(crate) outcome: Result<
        OrganizingUnionInvitationCallbackReport,
        OrganizingUnionInvitationCallbackBlock,
    >,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCityTransferTerminalDispatch {
    pub(crate) request: QueuedCityTransferTerminal,
    pub(crate) outcome: Result<CityTransferFinishReport, CityTransferFinishBlock>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldConfederationCreationTerminalDispatch {
    pub(crate) request: QueuedConfederationCreationTerminal,
    pub(crate) outcome: Result<
        ConfederationCreationCallbackReport,
        ConfederationCreationCallbackBlock,
    >,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldUnionApplicationRuntimeReport {
    pub(crate) terminals: Vec<WorldUnionApplicationTerminalDispatch>,
    pub(crate) invitation_terminals: Vec<WorldUnionInvitationTerminalDispatch>,
    pub(crate) confirmations: Vec<UnionApplicationConfirmationDelivery>,
    pub(crate) endpoint_blocks: Vec<UnionApplicationEndpointBlock>,
    pub(crate) city_terminals: Vec<WorldCityTransferTerminalDispatch>,
    pub(crate) city_confirmations: Vec<CityTransferConfirmationDelivery>,
    pub(crate) city_endpoint_blocks: Vec<CityTransferEndpointBlock>,
    pub(crate) confederation_creation_terminals:
        Vec<WorldConfederationCreationTerminalDispatch>,
    pub(crate) confederation_creation_confirmations:
        Vec<ConfederationCreationConfirmationDelivery>,
    pub(crate) confederation_creation_endpoint_blocks:
        Vec<ConfederationCreationEndpointBlock>,
}

/// Один фактически извлечённый элемент двух FIFO `ProcessMessage`.
#[derive(Debug)]
pub(crate) enum ProcessedWorldEvent {
    Message(RoutedWorldMessage),
    ServerMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldServerMessageOutcome,
    },
    LogMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldLogMessageOutcome,
    },
    OtherMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldOtherMessageOutcome,
    },
    WriteLogMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldWriteLogMessageOutcome,
    },
    PlayerMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldPlayerMessageOutcome,
    },
    CountryMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldCountryMessageOutcome,
    },
    GmaMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldGmaMessageOutcome,
    },
    GmMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldGmMessageOutcome,
    },
    JjcMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: JjcSystemMessageOutcome,
    },
    TeamMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldTeamMessageOutcome,
    },
    OrganizingSessionResult {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingSessionResultDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingConsumedLong {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingConsumedLongDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingCreateFaction {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingCreateFactionDispatch, OrganizingCreateFactionBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionWarPlayerDied {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingFactionWarPlayerDiedDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingInitialData {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingInitialDataDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingDeclareWarFactionList {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingDeclareWarFactionListDispatch,
            OrganizingDeclareWarFactionListBlock,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionList {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionListDispatch, OrganizingFactionListBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionApplicationCancel {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingFactionApplicationCancelDispatch,
            OrganizingFactionApplicationCancelBlock,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionApplication {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingFactionApplicationDispatch<UnionApplicationSessionReport>,
            OrganizingFactionApplicationDispatchBlock<UnionApplicationSessionBlock>,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionApplicationDecision {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingFactionApplicationDecisionDispatch,
            OrganizingFactionDoJoinBlock,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionFireOut {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionFireOutDispatch, OrganizingFactionFireOutBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionExit {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionExitDispatch, OrganizingFactionExitBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingUnionExit {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingUnionExitDispatch, OrganizingUnionExitBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionDemise {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionDemiseDispatch, OrganizingFactionDemiseBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingUnionDemise {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingUnionDemiseDispatch, OrganizingUnionDemiseBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionDisband {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionDisbandDispatch, OrganizingFactionDisbandBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingUnionDisband {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingUnionDisbandDispatch, OrganizingUnionDisbandBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionDub {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionDubDispatch, OrganizingFactionDubBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionPurview {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionPurviewDispatch, OrganizingFactionPurviewBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingUnionFireOut {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingUnionFireOutDispatch, OrganizingUnionFireOutBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingDeclareFactionWar {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingDeclareFactionWarDispatch, OrganizingDeclareFactionWarBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionBillboard {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionBillboardOutcome, OrganizingFactionBillboardBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionUpgrade {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionUpgradeDispatch, OrganizingFactionUpgradeBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionUploadIcon {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionUploadIconDispatch, FactionUploadIconBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionContributor {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionContributorDispatch, OrganizingContributorBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionExperience {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionExperienceDispatch, FactionExperienceBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionMemberState {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingFactionMemberStateDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionTax {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionTaxDispatch, OrganizingFactionTaxBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingRegionParamUpdate {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingRegionParamDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingRegionRoute {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingRegionRouteDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingCityGate {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingCityGateDispatch, OrganizingCityGateBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingCityTransfer {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingCityTransferDispatch<CityTransferSessionReport>,
            CityTransferStartBlock<CityTransferSessionBlock>,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingAdmissionPermit {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingAdmissionPermitDispatch, OrganizingAdmissionPermitBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingAttackCityEnd {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingAttackCityEndDispatch, AttackCityEndBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingVillageWarApplication {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingVillageWarApplicationDispatch,
            OrganizingVillageWarApplicationBlock,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingCityWarApplication {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingCityWarApplicationDispatch,
            OrganizingCityWarApplicationBlock,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingCityWarResult {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingCityWarResultDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingGoodsWarCommand {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingGoodsWarCommandDispatch,
            GoodsWarMemberBlock<OrganizingGoodsWarContextBlock>,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingGoodsWarFactionWin {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingGoodsWarFactionWinDispatch,
            OrganizingGoodsWarFactionWinBlock,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingPlayerQuestCommand {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingPlayerQuestCommandDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingPlayerRunScript {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingPlayerRunScriptDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingFactionParameter {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingFactionParameterDispatch, OrganizingFactionParameterBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingChangeRegionRouter {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingChangeRegionRouterDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingVillageWarResult {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: OrganizingVillageWarResultDispatch,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingUnionApplication {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingUnionApplicationDispatch<UnionApplicationSessionReport>,
            WorldUnionApplicationStartBlock,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingPlayerInviteFaction {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            WorldPlayerInviteFactionDispatch,
            WorldPlayerInviteFactionStartBlock,
        >,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingLeaveWordEnable {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingLeaveWordEnableDispatch, OrganizingLeaveWordEnableBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingLeaveWord {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingLeaveWordDispatch, OrganizingLeaveWordBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingLeaveWordEdit {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingLeaveWordEditDispatch, OrganizingLeaveWordEditBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    OrganizingPronounce {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<OrganizingPronounceDispatch, OrganizingPronounceBlock>,
        runtime: WorldUnionApplicationRuntimeReport,
    },
    LoginClientReconnected(WorldLoginClientReplacement),
}

/// Итог одного точного snapshot-прохода `CGame::ProcessMessage`.
#[derive(Debug)]
pub(crate) struct WorldProcessMessageOutcome {
    pub(crate) legacy_result: i32,
    pub(crate) initial_server_events: i32,
    pub(crate) initial_login_messages: Option<i32>,
    pub(crate) server_slots_visited: i32,
    pub(crate) login_slots_visited: i32,
    pub(crate) events: Vec<ProcessedWorldEvent>,
    pub(crate) game_server_message_time_ms: u32,
    pub(crate) login_server_message_time_ms: u32,
}

/// Единственный MainLoop-накопитель внешнего call-site `ProcessMessage`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorldProcessMessageStageState {
    pub(crate) accumulated_time_ms: u32,
}

/// Точный общий bitmask одноразовых lazy initializer-ов внутри MainLoop.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorldMainLoopInitializationState {
    pub(crate) mask: u32,
}

/// Три process-global tick-а current/refresh/profiling участка MainLoop.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorldMainLoopClockState {
    pub(crate) current_tick_ms: u32,
    pub(crate) last_refresh_tick_ms: u32,
    pub(crate) stage_started_at_ms: u32,
}

/// Три tail tick-а MainLoop: current, pacing deadline и minute-owner start.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorldMainLoopTailClockState {
    pub(crate) current_tick_ms: u32,
    pub(crate) pacing_deadline_ms: u32,
    pub(crate) minute_started_at_ms: u32,
}

/// Отдельный BSS tick strict release-login gate после 40-ms pacing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorldMainLoopLoginReleaseState {
    pub(crate) last_checked_at_ms: u32,
}

/// Все впервые выполненные bits `0x10/0x20/0x40` в исходном порядке.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldMainLoopTailClockInitialization {
    pub(crate) previous_mask: u32,
    pub(crate) initialized_mask: u32,
    pub(crate) initial_current_tick_ms: Option<u32>,
    pub(crate) initial_pacing_deadline_ms: Option<u32>,
    pub(crate) initial_minute_started_at_ms: Option<u32>,
}

/// Два process-global поля Largess-участка MainLoop.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorldMainLoopLargessState {
    pub(crate) pass_count: u32,
    pub(crate) last_start_request_tick_ms: u32,
}

/// Остальные process-global счётчики одного 600-секундного MainLoop-окна.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldMainLoopProfileState {
    pub(crate) last_published_at_ms: u32,
    pub(crate) ai_calls: u32,
    pub(crate) ai_time_ms: u32,
    pub(crate) refresh_text_time_ms: u32,
    pub(crate) net_session_time_ms: u32,
    pub(crate) faction_war_time_ms: u32,
    pub(crate) timer_time_ms: u32,
    pub(crate) process_player_data_queue_time_ms: u32,
    pub(crate) session_factory_time_ms: u32,
    pub(crate) save_point_time_ms: u32,
}

/// Один полный вызов `CGame::AI` вместе с окружающими MainLoop-счётчиками.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldMainLoopAiStageReport {
    pub(crate) previous_stage_finished_at_ms: u32,
    pub(crate) save_point_elapsed_ms: u32,
    pub(crate) accumulated_save_point_time_ms: u32,
    pub(crate) ai_calls: u32,
    pub(crate) ai_started_at_ms: u32,
    pub(crate) ai: WorldGameAiReport,
    pub(crate) ai_finished_at_ms: u32,
    pub(crate) ai_elapsed_ms: u32,
    pub(crate) accumulated_ai_time_ms: u32,
}

/// Один вызов `CSessionFactory::AI` и его точный MainLoop accumulator.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldMainLoopSessionFactoryStageReport {
    pub(crate) ai: WorldSessionFactoryAiReport,
    pub(crate) finished_at_ms: u32,
    pub(crate) elapsed_ms: u32,
    pub(crate) accumulated_time_ms: u32,
    pub(crate) next_stage_started_at_ms: u32,
}

/// Причина одной доказанной reject-ветви `ProcessPlayerDataQueue`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerDataQueueRejectReason {
    NullPlayer,
    MissingRegion {
        region_id: i32,
    },
    MissingOrDisconnectedGameServer {
        region_id: i32,
        game_server_index: u32,
    },
}

/// Одна ordered friend-мутация и её optional presence-рассылка.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldFriendPresenceUpdate {
    pub(crate) friend_index: usize,
    pub(crate) player_id: u32,
    pub(crate) online: bool,
    pub(crate) target_game_server_index: Option<u32>,
    pub(crate) delivery: Option<Result<i32, SendMessageError>>,
}

/// Два подтверждённых порядка одного route: direct `GetPlayerData` публикует
/// player до friend-loop, а `ProcessPlayerDataQueue` — после него.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldLoadedPlayerRouteOrder {
    Direct,
    LoadedQueue,
}

/// Итог полного snapshot/retry прохода `CGame::ProcessPlayerDataQueue`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldProcessPlayerDataQueueOutcome {
    NoRecord {
        initial_size: u32,
        null_pops: u32,
    },
    Rejected {
        initial_size: u32,
        null_pops: u32,
        queue_player_id: u32,
        client_ip: u32,
        reason: WorldPlayerDataQueueRejectReason,
        login_delivery: Result<i32, SendMessageError>,
    },
    Accepted {
        initial_size: u32,
        null_pops: u32,
        player_id: u32,
        client_ip: u32,
        game_server_index: u32,
        login_delivery: Result<i32, SendMessageError>,
        friend_updates: Vec<WorldFriendPresenceUpdate>,
        online_removal: WorldOnlinePlayerRemoveOutcome,
        replaced_existing_player: bool,
        login_time_ms: u32,
    },
}

/// Локальная safe-граница уже извлечённого player-record-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldProcessPlayerDataQueueBlock {
    UnterminatedCdkey,
    Organizing(PlayerOrganizingUpdateError),
    UninitializedGameServerPort { game_server_index: u32 },
}

/// Контекст stopped-ветви, которая в исходнике не возвращалась бы штатно.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldProcessPlayerDataQueueError {
    pub(crate) initial_size: u32,
    pub(crate) null_pops: u32,
    pub(crate) player_id: u32,
    pub(crate) block: WorldProcessPlayerDataQueueBlock,
}

/// Safe-граница fixed `char szCdkey[20]` одного DB-load запроса.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldPlayerLoadRequestBlock {
    pub(crate) account_length: usize,
}

/// Exact bool-смысл `CPlayerLoadQueue::PushPlayerLoadData`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerLoadRequestOutcome {
    Queued,
    Duplicate,
}

/// Async DB-owner, которому worker передаёт уже инициализированный `CPlayer`.
pub(crate) trait WorldPlayerDataLoadOwner {
    fn load_player_data<'a>(
        &'a mut self,
        player: &'a mut CPlayer,
    ) -> impl Future<Output = bool> + 'a;
}

/// Связывает полный `CPlayer::LoadData` с bool-контрактом фонового World worker-а.
///
/// DB-owner, player-list и setup snapshots остаются явно принадлежащими
/// вызывающему коду. Это заменяет только process-global singleton lookup-и;
/// порядок `CRsPlayer::LoadPlayer` и последующей post-load стадии не меняется.
pub(crate) struct WorldPlayerLoadDataAdapter<'owner, Loader> {
    loader: &'owner mut Loader,
    player_list: &'owner mut CPlayerList,
    globe_setup: &'owner GlobeSetupSnapshot,
    coefficients: &'owner PlayerPropertyCoefficients,
}

impl<'owner, Loader> WorldPlayerLoadDataAdapter<'owner, Loader> {
    pub(crate) fn new(
        loader: &'owner mut Loader,
        player_list: &'owner mut CPlayerList,
        globe_setup: &'owner GlobeSetupSnapshot,
        coefficients: &'owner PlayerPropertyCoefficients,
    ) -> Self {
        Self {
            loader,
            player_list,
            globe_setup,
            coefficients,
        }
    }
}

impl<Loader> WorldPlayerDataLoadOwner for WorldPlayerLoadDataAdapter<'_, Loader>
where
    Loader: PlayerLoadDataOwner,
{
    fn load_player_data<'a>(
        &'a mut self,
        player: &'a mut CPlayer,
    ) -> impl Future<Output = bool> + 'a {
        async move {
            matches!(
                player
                    .load_data(
                        self.loader,
                        self.player_list,
                        self.globe_setup,
                        self.coefficients,
                    )
                    .await,
                PlayerLoadDataOutcome::Loaded(_)
            )
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerLoadBatchRecordOutcome {
    SkippedZeroPlayerId,
    Loaded,
    LoadFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldPlayerLoadBatchRecordReport {
    pub(crate) player_id: i32,
    pub(crate) client_ip: u32,
    pub(crate) started_at_ms: Option<u32>,
    pub(crate) finished_at_ms: Option<u32>,
    pub(crate) elapsed_ms: Option<u32>,
    pub(crate) outcome: WorldPlayerLoadBatchRecordOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldPlayerLoadBatchReport {
    pub(crate) worker_index: u32,
    pub(crate) drained_records: usize,
    pub(crate) records: Vec<WorldPlayerLoadBatchRecordReport>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldPlayerLoadBatchBlock {
    pub(crate) worker_index: u32,
    pub(crate) player_id: i32,
    pub(crate) processed_records: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerLoadWorkerExit {
    GameThread,
    PlayerLoadThreads,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldPlayerLoadWorkerReport {
    pub(crate) worker_index: u32,
    pub(crate) completed_batches: u32,
    pub(crate) drained_records: u32,
    pub(crate) exit: WorldPlayerLoadWorkerExit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldPlayerLoadWorkerBlock {
    pub(crate) worker_index: u32,
    pub(crate) completed_batches: u32,
    pub(crate) drained_records: u32,
    pub(crate) source: WorldPlayerLoadBatchBlock,
}

/// Cloneable queue-owner, который можно безопасно передать системному
/// `LoadPlayerDataFromDB` потоку без передачи всего mutable `CGame`.
#[derive(Clone)]
pub(crate) struct WorldPlayerLoadWorkerSpec {
    player_load_queue: CPlayerLoadQueue,
    player_data_queue: CPlayerDataQueue,
}

impl WorldPlayerLoadWorkerSpec {
    pub(crate) const fn new(
        player_load_queue: CPlayerLoadQueue,
        player_data_queue: CPlayerDataQueue,
    ) -> Self {
        Self {
            player_load_queue,
            player_data_queue,
        }
    }

    /// Выполняет один exact drain/process batch фонового DB-load worker-а.
    pub(crate) async fn process_batch<Loader, LoadLargess, GetTick>(
        &self,
        worker_index: u32,
        loader: &mut Loader,
        load_largess: &mut LoadLargess,
        mut get_tick: GetTick,
    ) -> Result<WorldPlayerLoadBatchReport, WorldPlayerLoadBatchBlock>
    where
        Loader: WorldPlayerDataLoadOwner + ?Sized,
        LoadLargess: FnMut(&mut CPlayer) + ?Sized,
        GetTick: FnMut() -> u32,
    {
        let mut drained = self.player_load_queue.pop_player_load_data_to_list();
        let drained_records = drained.len();
        if drained_records != 0 {
            put_string_to_file(
                "TemptLoadDataLog",
                format!(
                    "Thread {} Need To Process {} DB Request.",
                    worker_index as i32, drained_records as u32 as i32,
                )
                .as_bytes(),
            );
        }

        let mut records = Vec::with_capacity(drained_records);
        while let Some(entry) = drained.pop_front() {
            let player_id = entry.player_id();
            let client_ip = entry.client_ip();
            if player_id == 0 {
                records.push(WorldPlayerLoadBatchRecordReport {
                    player_id,
                    client_ip,
                    started_at_ms: None,
                    finished_at_ms: None,
                    elapsed_ms: None,
                    outcome: WorldPlayerLoadBatchRecordOutcome::SkippedZeroPlayerId,
                });
                continue;
            }
            let Some(account) = entry.cdkey().map(<[u8]>::to_vec) else {
                return Err(WorldPlayerLoadBatchBlock {
                    worker_index,
                    player_id,
                    processed_records: records.len(),
                });
            };

            let started_at_ms = get_tick();
            put_string_to_file(
                "TemptLoadDataLog",
                format!(
                    "{} Request Read DB Start. (PID:{})",
                    player_id, worker_index as i32,
                )
                .as_bytes(),
            );

            let mut player = Box::new(CPlayer::with_clone_decode_constructor_state());
            player.set_database_load_identity(player_id, &account);
            let loaded = loader.load_player_data(&mut player).await;
            let mut player = if loaded {
                Some(player)
            } else {
                put_string_to_file(
                    "debug-DB",
                    format!("Read Player DB Error. (ID:{player_id})").as_bytes(),
                );
                None
            };

            if let Some(player) = player.as_deref_mut() {
                load_largess(player);
            }
            let fixed_account = entry.fixed_cdkey();
            let _ = self
                .player_data_queue
                .push_player_data(PlayerDataQueueEntry::new(
                    fixed_account,
                    player_id as u32,
                    client_ip,
                    player,
                ));

            let finished_at_ms = get_tick();
            let elapsed_ms = finished_at_ms.wrapping_sub(started_at_ms);
            put_string_to_file(
                "TemptLoadDataLog",
                format!(
                    "{} Read DB End. (PID:{}) (time:{})",
                    player_id, worker_index as i32, elapsed_ms,
                )
                .as_bytes(),
            );
            records.push(WorldPlayerLoadBatchRecordReport {
                player_id,
                client_ip,
                started_at_ms: Some(started_at_ms),
                finished_at_ms: Some(finished_at_ms),
                elapsed_ms: Some(elapsed_ms),
                outcome: if loaded {
                    WorldPlayerLoadBatchRecordOutcome::Loaded
                } else {
                    WorldPlayerLoadBatchRecordOutcome::LoadFailed
                },
            });
        }

        Ok(WorldPlayerLoadBatchReport {
            worker_index,
            drained_records,
            records,
        })
    }

    /// Повторяет polling-loop до exact приоритетного game/load exit-флага.
    pub(crate) async fn run<Loader, LoadLargess, GetTick>(
        &self,
        worker_index: u32,
        game_thread_exit: &AtomicBool,
        player_load_threads_exit: &AtomicBool,
        loader: &mut Loader,
        load_largess: &mut LoadLargess,
        mut get_tick: GetTick,
    ) -> Result<WorldPlayerLoadWorkerReport, WorldPlayerLoadWorkerBlock>
    where
        Loader: WorldPlayerDataLoadOwner + ?Sized,
        LoadLargess: FnMut(&mut CPlayer) + ?Sized,
        GetTick: FnMut() -> u32,
    {
        let mut completed_batches = 0_u32;
        let mut drained_records = 0_u32;
        loop {
            let exit = if game_thread_exit.load(Ordering::Relaxed) {
                Some(WorldPlayerLoadWorkerExit::GameThread)
            } else if player_load_threads_exit.load(Ordering::Relaxed) {
                Some(WorldPlayerLoadWorkerExit::PlayerLoadThreads)
            } else {
                None
            };
            if let Some(exit) = exit {
                return Ok(WorldPlayerLoadWorkerReport {
                    worker_index,
                    completed_batches,
                    drained_records,
                    exit,
                });
            }

            tokio::time::sleep(Duration::from_millis(1)).await;
            let batch = self
                .process_batch(worker_index, loader, load_largess, &mut get_tick)
                .await
                .map_err(|source| WorldPlayerLoadWorkerBlock {
                    worker_index,
                    completed_batches,
                    drained_records,
                    source,
                })?;
            completed_batches = completed_batches.wrapping_add(1);
            drained_records = drained_records.wrapping_add(batch.drained_records as u32);
        }
    }
}

#[derive(Debug)]
pub(crate) struct WorldPlayerLargessLoadReport {
    pub(crate) load: LoadLargessReport,
    pub(crate) write_log_queue_length: Option<usize>,
}

/// Профилированная `DAT_0056e524` стадия перед `CTimer::Run`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldMainLoopPlayerDataQueueStageReport {
    Complete {
        outcome: WorldProcessPlayerDataQueueOutcome,
        finished_at_ms: u32,
        elapsed_ms: u32,
        accumulated_time_ms: u32,
        next_stage_started_at_ms: u32,
    },
    Blocked {
        error: WorldProcessPlayerDataQueueError,
    },
}

/// Один exact `CPlayerRanks::OnStatRanks`, выполненный внутри calendar callback.
#[derive(Debug)]
pub(crate) struct PlayerRanksTimerRefreshReport {
    pub(crate) stat: PlayerRanksStatRunReport,
    pub(crate) publication: PlayerRanksGameServerUpdate,
    pub(crate) next_time: TagTime,
    pub(crate) next_event_id: Option<TimerId>,
}

#[derive(Debug)]
pub(crate) enum PlayerRanksTimerRefreshBlock {
    Stat(PlayerRanksStatRunBlock),
    Serialization(PlayerRanksSerializationBlock),
    Schedule(PlayerRanksScheduleBlock),
}

#[derive(Debug)]
pub(crate) enum CountryWarTimerReport {
    Phase {
        callback: CountryWarCallbackKind,
        war_id: i32,
        report: CountryWarPhaseReport,
    },
    Start {
        war_id: i32,
        report: CountryWarStartReport,
    },
    End {
        war_id: i32,
        report: CountryWarFinishReport,
    },
    TopInfo {
        callback: CountryWarCallbackKind,
        report: CountryWarTopInfoReport,
    },
}

#[derive(Debug)]
pub(crate) enum CountryWarTimerBlock {
    Phase(CountryWarPhaseBlock<Infallible>),
    Start(CountryWarStartBlock<Infallible>),
    End(CountryWarFinishBlock<Infallible>),
    TopInfo(CountryWarTopInfoBlock<Infallible>),
}

#[derive(Debug)]
pub(crate) enum WorldTimerCallbackBlock {
    CopyNumber(CopyNumberScheduleBlock),
    PlayerRanks(PlayerRanksTimerRefreshBlock),
    OrganizingTax(OrganizingTaxScheduleBlock),
    CountryWar(CountryWarTimerBlock),
}

/// Выполненный prefix `CTimer::Run` перед domain callback safe-границей.
#[derive(Debug)]
pub(crate) struct WorldMainLoopTimerStageBlock {
    pub(crate) timer: TimerRunReport,
    pub(crate) source: WorldTimerCallbackBlock,
}

/// Полный `CTimer::Run` и окружающий его accumulator `DAT_0056e520`.
#[derive(Debug)]
pub(crate) struct WorldMainLoopTimerStageReport {
    pub(crate) timer: TimerRunReport,
    pub(crate) copy_number_resets: Vec<CopyNumberResetReport>,
    pub(crate) player_ranks: Vec<PlayerRanksTimerRefreshReport>,
    pub(crate) organizing_taxes: Vec<OrganizingTodayTaxRefreshReport>,
    pub(crate) country_wars: Vec<CountryWarTimerReport>,
    pub(crate) finished_at_ms: u32,
    pub(crate) elapsed_ms: u32,
    pub(crate) accumulated_time_ms: u32,
    pub(crate) next_stage_started_at_ms: u32,
}

struct WorldTimerHandler<'a, Callback> {
    game: &'a CGame,
    country_war: &'a mut CountryWarSys,
    country_handler: &'a mut CCountryHandler,
    country_war_callbacks: CountryWarCallbacks<Callback>,
    globe_setup: &'a GlobeSetupSnapshot,
    organizing_parameters: &'a mut COrganizingParam,
    player_ranks: &'a mut CPlayerRanks,
    rs_player: &'a mut TiberiusRsPlayer,
    player_database: Option<&'a mut WorldTdsClient>,
    organizing: &'a COrganizingCtrl,
    copy_number_timer: &'a mut CopyNumberTimerState,
    log: &'a mut WorldLogTextOwner,
    get_log_local_time: &'a mut dyn FnMut() -> WorldLogLocalTime,
    put_log_info: &'a mut dyn FnMut(&[u8]),
    world_string_by_id: &'a mut dyn FnMut(&[u8]) -> Vec<u8>,
    format_world_string:
        &'a mut dyn FnMut(&[u8], &[UnionFormatArgument<'_>]) -> Vec<u8>,
    copy_number_resets: Vec<CopyNumberResetReport>,
    refreshes: Vec<PlayerRanksTimerRefreshReport>,
    tax_refreshes: Vec<OrganizingTodayTaxRefreshReport>,
    country_wars: Vec<CountryWarTimerReport>,
    pending_copy_number_registration: Option<usize>,
    pending_player_ranks_registration: Option<usize>,
    pending_tax_registration: Option<PreparedTodayTaxRefresh>,
}

impl<Callback, GetTick, GetTimerLocalTime>
    AsyncTimerCallbackHandler<Callback, GetTick, GetTimerLocalTime>
    for WorldTimerHandler<'_, Callback>
where
    Callback: Copy + PartialEq,
    GetTick: FnMut() -> u32,
    GetTimerLocalTime: FnMut() -> TagTime,
{
    type Block = WorldTimerCallbackBlock;

    async fn dispatch(
        &mut self,
        invocation: TimerCallbackInvocation<Callback>,
        get_tick: &mut GetTick,
        get_timer_local_time: &mut GetTimerLocalTime,
    ) -> Result<AsyncTimerCallbackDisposition<Callback>, Self::Block> {
        let copy_number_event = matches!(
            invocation.source,
            TimerCallbackSource::Calendar(event_id)
                if self.copy_number_timer.is_event(event_id)
        );
        if copy_number_event {
            let current_time = get_timer_local_time();
            let report = self
                .copy_number_timer
                .prepare_reset(current_time)
                .map_err(WorldTimerCallbackBlock::CopyNumber)?;
            let next_time = report.scheduled_time;
            let report_index = self.copy_number_resets.len();
            self.copy_number_resets.push(report);
            self.pending_copy_number_registration = Some(report_index);
            return Ok(AsyncTimerCallbackDisposition::Handled {
                next_calendar_event: Some(CalendarTimerRegistration {
                    time: next_time,
                    callback: invocation.callback,
                    parameter: 0,
                }),
            });
        }

        let tax_event_id = match invocation.source {
            TimerCallbackSource::Calendar(event_id)
                if self.organizing_parameters.is_tax_event(event_id) => Some(event_id),
            _ => None,
        };
        if let Some(event_id) = tax_event_id {
            let current_time = get_timer_local_time();
            let prepared = self
                .organizing_parameters
                .prepare_today_tax_refresh(
                    event_id,
                    current_time,
                    self.game.current_game_server_sender().as_ref(),
                )
                .map_err(WorldTimerCallbackBlock::OrganizingTax)?;
            let next_time = prepared.next_time;
            self.pending_tax_registration = Some(prepared);
            return Ok(AsyncTimerCallbackDisposition::Handled {
                next_calendar_event: Some(CalendarTimerRegistration {
                    time: next_time,
                    callback: invocation.callback,
                    parameter: 0,
                }),
            });
        }

        let is_player_ranks_event = matches!(
            invocation.source,
            TimerCallbackSource::Calendar(event_id)
                if self.player_ranks.stat_event_id() == Some(event_id)
        );
        if is_player_ranks_event {
            let stat = self
                .game
                .stat_player_ranks(
                    self.player_ranks,
                    self.rs_player,
                    self.player_database.as_deref_mut(),
                    self.organizing,
                    self.log,
                    get_tick,
                    &mut *self.get_log_local_time,
                    &mut *self.put_log_info,
                )
                .await
                .map_err(PlayerRanksTimerRefreshBlock::Stat)
                .map_err(WorldTimerCallbackBlock::PlayerRanks)?;
            let sender = self.game.current_game_server_sender();
            let publication = self
                .player_ranks
                .update_ranks_to_game_server(sender.as_ref())
                .map_err(PlayerRanksTimerRefreshBlock::Serialization)
                .map_err(WorldTimerCallbackBlock::PlayerRanks)?;
            let current_time = get_timer_local_time();
            let next_time = self
                .player_ranks
                .next_stat_time(current_time)
                .map_err(PlayerRanksTimerRefreshBlock::Schedule)
                .map_err(WorldTimerCallbackBlock::PlayerRanks)?;
            let refresh_index = self.refreshes.len();
            self.refreshes.push(PlayerRanksTimerRefreshReport {
                stat,
                publication,
                next_time,
                next_event_id: None,
            });
            self.pending_player_ranks_registration = Some(refresh_index);

            return Ok(AsyncTimerCallbackDisposition::Handled {
                next_calendar_event: Some(CalendarTimerRegistration {
                    time: next_time,
                    callback: invocation.callback,
                    parameter: 0,
                }),
            });
        }

        let Some(callback) = self.country_war_callbacks.kind(invocation.callback) else {
            return Ok(AsyncTimerCallbackDisposition::PassThrough);
        };
        let mut effects = WorldCountryWarEffects {
            game: self.game,
            country_handler: &mut *self.country_handler,
            globe_setup: self.globe_setup,
            world_string: &mut *self.world_string_by_id,
            format_world_string: &mut *self.format_world_string,
        };
        let report = match callback {
            CountryWarCallbackKind::Clear
            | CountryWarCallbackKind::DeclareBegin
            | CountryWarCallbackKind::DeclareEnd
            | CountryWarCallbackKind::PrepareBegin
            | CountryWarCallbackKind::PrepareEnd => {
                let phase = match callback {
                    CountryWarCallbackKind::Clear => CountryWarPhase::Clear,
                    CountryWarCallbackKind::DeclareBegin => CountryWarPhase::DeclareBegin,
                    CountryWarCallbackKind::DeclareEnd => CountryWarPhase::DeclareEnd,
                    CountryWarCallbackKind::PrepareBegin => CountryWarPhase::PrepareBegin,
                    CountryWarCallbackKind::PrepareEnd => CountryWarPhase::PrepareEnd,
                    _ => unreachable!("ветка ограничена phase callbacks"),
                };
                let report = self
                    .country_war
                    .run_phase(phase, invocation.parameter, &mut effects)
                    .map_err(CountryWarTimerBlock::Phase)
                    .map_err(WorldTimerCallbackBlock::CountryWar)?;
                CountryWarTimerReport::Phase {
                    callback,
                    war_id: invocation.parameter,
                    report,
                }
            }
            CountryWarCallbackKind::Start => CountryWarTimerReport::Start {
                war_id: invocation.parameter,
                report: self
                    .country_war
                    .run_war_start(invocation.parameter, &mut effects)
                    .map_err(CountryWarTimerBlock::Start)
                    .map_err(WorldTimerCallbackBlock::CountryWar)?,
            },
            CountryWarCallbackKind::End => {
                let now = get_timer_local_time();
                CountryWarTimerReport::End {
                    war_id: invocation.parameter,
                    report: self
                        .country_war
                        .run_war_end(invocation.parameter, now, get_tick, &mut effects)
                        .map_err(CountryWarTimerBlock::End)
                        .map_err(WorldTimerCallbackBlock::CountryWar)?,
                }
            }
            CountryWarCallbackKind::StartInfo | CountryWarCallbackKind::EndInfo => {
                let kind = match callback {
                    CountryWarCallbackKind::StartInfo => CountryWarTopInfoKind::Start,
                    CountryWarCallbackKind::EndInfo => CountryWarTopInfoKind::End,
                    _ => unreachable!("ветка ограничена top-info callbacks"),
                };
                let now = get_timer_local_time();
                CountryWarTimerReport::TopInfo {
                    callback,
                    report: self
                        .country_war
                        .run_top_info(kind, invocation.parameter, now, get_tick, &mut effects)
                        .map_err(CountryWarTimerBlock::TopInfo)
                        .map_err(WorldTimerCallbackBlock::CountryWar)?,
                }
            }
        };
        self.country_wars.push(report);
        Ok(AsyncTimerCallbackDisposition::Handled {
            next_calendar_event: None,
        })
    }

    fn calendar_event_registered(
        &mut self,
        _invocation: TimerCallbackInvocation<Callback>,
        event_id: TimerId,
    ) {
        if let Some(report_index) = self.pending_copy_number_registration.take() {
            self.copy_number_timer.finish_reset(
                &mut self.copy_number_resets[report_index],
                event_id,
            );
            return;
        }

        if let Some(prepared) = self.pending_tax_registration.take() {
            let report = self.organizing_parameters.finish_today_tax_refresh(
                prepared,
                event_id,
                self.world_string_by_id,
            );
            self.tax_refreshes.push(report);
            return;
        }

        self.player_ranks.finish_stat_schedule(event_id);
        let refresh_index = self
            .pending_player_ranks_registration
            .take()
            .expect("handled PlayerRanks callback всегда просит следующее событие");
        self.refreshes[refresh_index].next_event_id = Some(event_id);
    }
}

/// Полный `CFactionWarSys::Run` и окружающий accumulator `DAT_0056e51c`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldMainLoopFactionWarStageReport {
    pub(crate) faction_war: FactionWarRunReport,
    pub(crate) finished_at_ms: u32,
    pub(crate) elapsed_ms: u32,
    pub(crate) accumulated_time_ms: u32,
}

/// Непрофилированный DB batch и tick начала следующего `CNetSessionManager`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldMainLoopDbMiscStageReport {
    pub(crate) output: DbMiscDoneOutReport,
    pub(crate) input: DbMiscDoneInReport,
    pub(crate) auction: DbMiscLoadAuctionReport,
    pub(crate) next_stage_started_at_ms: u32,
}

/// Полный `CNetSessionManager::Run` и accumulator `DAT_0056e518`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldMainLoopNetSessionStageReport {
    pub(crate) sessions: NetSessionRunReport,
    pub(crate) union_applications: WorldUnionApplicationRuntimeReport,
    pub(crate) finished_at_ms: u32,
    pub(crate) elapsed_ms: u32,
    pub(crate) accumulated_time_ms: u32,
}

/// Полный pair minute-owner-ов перед BaiTan.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldMainLoopMinuteStageReport {
    pub(crate) initialization: WorldMainLoopTailClockInitialization,
    pub(crate) current_tick_ms: u32,
    pub(crate) minute_delta: i32,
    pub(crate) organizing: OrganizingRunReport,
    pub(crate) country: CountryRunReport,
}

/// Safe-граница одного из двух ordered minute-owner-ов.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldMainLoopMinuteStageBlock {
    Organizing(OrganizingRunBlock),
    Country(CountryRunBlock),
}

/// Точная соседняя пара `DoneBaiTanList -> CJJcSystem::Run`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldMainLoopBaiTanJjcStageReport {
    pub(crate) bai_tan: WorldDoneBaiTanListReport,
    pub(crate) jjc: JjcRunReport,
}

/// Результат точной session/team/plug цепочки timeout-login owner-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldLoginTimeoutTeamExit {
    SessionMissingOrNotTeam,
    PlugMissing,
    Exited,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldRegionChangeTeamUpdate {
    SessionMissingOrNotTeam,
    PlugMissing,
    Updated,
}

/// Одна friend-ветвь после перевода просроченного игрока в offline-list.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldLoginTimeoutFriendOutcome {
    Offline {
        friend_index: usize,
    },
    Notified {
        friend_index: usize,
        friend_player_id: u32,
        target_game_server_index: Option<u32>,
        delivery: Result<i32, SendMessageError>,
    },
}

/// Полный наблюдаемый результат одного login-list узла в исходном порядке.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldLoginTimeoutEntryOutcome {
    Waiting {
        player_id: u32,
        elapsed_ms: u32,
    },
    ExpiredMissingPlayer {
        player_id: u32,
        elapsed_ms: u32,
    },
    Released {
        player_id: u32,
        elapsed_ms: u32,
        login_delivery: Result<i32, SendMessageError>,
        team_id: i32,
        team_session_id: i32,
        team_exit: WorldLoginTimeoutTeamExit,
        online_removal: WorldOnlinePlayerRemoveOutcome,
        offline_inserted: bool,
        friend_outcomes: Vec<WorldLoginTimeoutFriendOutcome>,
    },
}

/// Один полный snapshot-проход `CGame::ProcessTimeOutLoginPlayer`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldLoginTimeoutReport {
    pub(crate) snapshot_tick_ms: u32,
    pub(crate) entries: Vec<WorldLoginTimeoutEntryOutcome>,
}

/// Выполненный 40-ms prefix перед чтением setup release interval.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldMainLoopPacingReport {
    pub(crate) sampled_tick_ms: u32,
    pub(crate) wait_duration_ms: Option<u32>,
    pub(crate) next_deadline_ms: u32,
    pub(crate) signed_lag_ms: i32,
    pub(crate) warning_resync_tick_ms: Option<u32>,
    pub(crate) release_gate_tick_ms: u32,
    pub(crate) release_gate_elapsed_ms: u32,
}

/// Полный хвост MainLoop либо точная граница неназначенного setup-поля.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldMainLoopTailStageReport {
    BlockedMissingReleaseInterval {
        pacing: WorldMainLoopPacingReport,
    },
    Complete {
        pacing: WorldMainLoopPacingReport,
        release_interval_ms: u32,
        login_timeout: Option<WorldLoginTimeoutReport>,
    },
}

/// Нормализованный owned-результат save pre-gate внутри полного MainLoop.
#[derive(Debug)]
pub(crate) struct WorldMainLoopSaveStageReport {
    pub(crate) manual_request: Option<WorldManualSaveRequestReport>,
    pub(crate) profile_started_at_ms: u32,
    pub(crate) elapsed_ms: u32,
    pub(crate) save_point_time_ms: u32,
    pub(crate) disposition: WorldMainLoopSaveStageDisposition,
}

/// Три возвращающиеся ветви save pre-gate; blocked путь завершает весь MainLoop.
#[derive(Debug)]
pub(crate) enum WorldMainLoopSaveStageDisposition {
    IntervalNotElapsed,
    SaveLockBusy {
        adjusted_last_save_point_time_ms: u32,
    },
    Triggered(WorldRunSaveTriggerDisposition),
}

/// Неизменяемые значения соседних singleton/setup owners одного MainLoop turn.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WorldMainLoopConfiguration {
    pub(crate) refresh_external_counts: WorldRefreshExternalCounts,
    pub(crate) use_appellation_function: bool,
    pub(crate) country_limits: CountryKingSaveLimits,
    pub(crate) jjc: JjcRunConfig,
}

/// Все process-global состояния, которые исходный MainLoop мутировал напрямую.
pub(crate) struct WorldMainLoopStateOwners<'a> {
    pub(crate) initialization: &'a mut WorldMainLoopInitializationState,
    pub(crate) clocks: &'a mut WorldMainLoopClockState,
    pub(crate) tail_clocks: &'a mut WorldMainLoopTailClockState,
    pub(crate) login_release: &'a mut WorldMainLoopLoginReleaseState,
    pub(crate) largess: &'a mut WorldMainLoopLargessState,
    pub(crate) profile: &'a mut WorldMainLoopProfileState,
    pub(crate) copy_number_timer: &'a mut CopyNumberTimerState,
    pub(crate) process_message: &'a mut WorldProcessMessageStageState,
    pub(crate) refresh_high_water: &'a mut WorldRefreshInfoHighWater,
    pub(crate) collect_player_data: &'a mut WorldCollectPlayerDataRequestState,
    pub(crate) save_trigger: &'a mut WorldRunSaveTriggerState,
    pub(crate) save_lifecycle: &'a SaveDataLifecycleState,
    pub(crate) reload_flags: &'a WorldReloadProfileFlags,
    pub(crate) player_ranks_request: &'a WorldPlayerRanksRequestState,
    pub(crate) save_thread_handle: &'a mut WorldSaveThreadHandleState,
}

/// Доменные owners и точные callback-контексты полного MainLoop.
pub(crate) struct WorldMainLoopOwners<
    'a,
    TimerCallback,
    FactionContext,
    LeiTingContextOwner,
    DbMiscContextOwner,
    JjcContext,
> {
    pub(crate) registry: &'a GoodsBasePropertiesRegistry,
    pub(crate) original_name_index: &'a GoodsOriginalNameIndex,
    pub(crate) coefficients: &'a PlayerPropertyCoefficients,
    pub(crate) load_player_largess: &'a mut dyn FnMut(&mut CPlayer),
    pub(crate) organizing: &'a mut COrganizingCtrl,
    pub(crate) country: &'a mut CCountryHandler,
    pub(crate) country_parameters: &'a mut CCountryParam,
    pub(crate) player_list: &'a mut CPlayerList,
    pub(crate) country_war: &'a mut CountryWarSys,
    pub(crate) country_war_callbacks: CountryWarCallbacks<TimerCallback>,
    pub(crate) four_nation_war: &'a mut CFourNationWarSys,
    pub(crate) honor_ranks: &'a mut CHonorRanks,
    pub(crate) organizing_parameters: &'a mut COrganizingParam,
    pub(crate) player_ranks: &'a mut CPlayerRanks,
    pub(crate) rs_player: &'a mut TiberiusRsPlayer,
    pub(crate) player_database: Option<&'a mut WorldTdsClient>,
    pub(crate) general_variables: Option<&'a mut CVariableList>,
    pub(crate) gods_battle: &'a mut CGodsBattleConf,
    pub(crate) rs_gods_battle: Option<&'a mut TiberiusRsGodsBattle>,
    pub(crate) gods_battle_database: Option<&'a mut WorldTdsClient>,
    pub(crate) auction_log: &'a mut CAuctionLog,
    pub(crate) auction_log_database: Option<&'a mut WorldTdsClient>,
    pub(crate) session_factory: &'a mut CSessionFactory,
    pub(crate) increment_log: &'a mut CIncrementLog,
    pub(crate) timer: &'a mut CTimer<TimerCallback>,
    pub(crate) faction_war: &'a mut CFactionWarSys,
    pub(crate) attack_city: &'a mut CAttackCitySys,
    pub(crate) attack_city_callbacks: AttackCityCallbacks<TimerCallback>,
    pub(crate) globe_setup: &'a GlobeSetupSnapshot,
    pub(crate) region_router: &'a RegionRouter,
    pub(crate) village_war: &'a mut CVillageWarSys,
    pub(crate) goods_war: &'a mut CGoodsWarMember,
    pub(crate) village_war_callbacks: VillageWarCallbacks<TimerCallback>,
    pub(crate) lei_ting: &'a mut CLeiTing,
    pub(crate) db_misc: &'a mut CDbMisc,
    pub(crate) net_sessions: &'a CNetSessionManager,
    pub(crate) union_application_runtime: &'a WorldUnionApplicationRuntimeOwner,
    pub(crate) jjc: &'a mut CJJcSystem,
    pub(crate) faction_context: &'a mut FactionContext,
    pub(crate) lei_ting_context: &'a mut LeiTingContextOwner,
    pub(crate) db_misc_context: &'a mut DbMiscContextOwner,
    pub(crate) jjc_context: &'a mut JjcContext,
    pub(crate) log: &'a mut WorldLogTextOwner,
}

pub(crate) struct WorldMainLoopCallbacks<'a, TimerCallback> {
    pub(crate) get_tick: &'a mut dyn FnMut() -> u32,
    pub(crate) get_save_point_time: &'a mut dyn FnMut() -> u32,
    pub(crate) try_enter_save: &'a mut dyn FnMut() -> bool,
    pub(crate) get_log_local_time: &'a mut dyn FnMut() -> WorldLogLocalTime,
    pub(crate) put_log_info: &'a mut dyn FnMut(&[u8]),
    pub(crate) get_auction_month_day: &'a mut dyn FnMut() -> i32,
    pub(crate) reload_context: &'a mut dyn WorldReloadContext,
    /// Конкретный `CLargess` owner для исходного `StartWorkerThread` вызова.
    pub(crate) largess: &'a TiberiusLargess,
    pub(crate) launch_save_thread:
        &'a mut dyn FnMut(&WorldSaveThreadLaunchRequest) -> WorldSaveThreadHandleState,
    pub(crate) random: &'a mut dyn FnMut(i32) -> i32,
    pub(crate) get_timer_local_time: &'a mut dyn FnMut() -> TagTime,
    pub(crate) world_string_by_id: &'a mut dyn FnMut(&[u8]) -> Vec<u8>,
    pub(crate) format_union_world_string:
        &'a mut dyn FnMut(&[u8], &[UnionFormatArgument<'_>]) -> Vec<u8>,
    pub(crate) put_union_war_log: &'a mut dyn FnMut(&[u8]),
    pub(crate) refresh_union_owned_city: &'a mut dyn FnMut(i32, i32, i32),
    pub(crate) update_union_player: &'a mut dyn FnMut(i32),
    pub(crate) check_invalid_organizing_string:
        &'a mut dyn FnMut(&mut Vec<u8>, bool) -> bool,
    /// Внешние feature-gates exact `CLogSystem::bFactionChat/bPrivateChat`.
    pub(crate) faction_chat_log_enabled: bool,
    pub(crate) private_chat_log_enabled: bool,
    /// Внешний feature-gate exact `CLogSystem::bDeleteLog`.
    pub(crate) delete_log_enabled: bool,
    /// Внешний feature-gate `CLogSystem::FactionCreateEnabled`.
    pub(crate) faction_create_log_enabled: bool,
    pub(crate) write_faction_create_log:
        &'a mut dyn FnMut(i32, &[u8], i32, &[u8]),
    /// Внешний feature-gate `CLogSystem::FactionTitleEnabled`.
    pub(crate) faction_title_log_enabled: bool,
    pub(crate) write_faction_title_log:
        &'a mut dyn FnMut(i32, &[u8], &[u8], &[u8], i32, &[u8], i32, &[u8]),
    pub(crate) faction_purview_add_log_enabled: bool,
    pub(crate) faction_purview_revoke_log_enabled: bool,
    pub(crate) write_faction_purview_log:
        &'a mut dyn FnMut(i32, &[u8], i32, i32, &[u8], i32, &[u8], i32),
    pub(crate) faction_level_log_enabled: bool,
    pub(crate) write_faction_level_log:
        &'a mut dyn FnMut(i32, &[u8], i32, i32, &[u8]),
    pub(crate) faction_experience_log_enabled: bool,
    pub(crate) write_faction_experience_log:
        &'a mut dyn FnMut(i32, &[u8], i32, &[u8], i32, i32),
    /// Внешний feature-gate `CLogSystem::FactionApplyEnabled`.
    pub(crate) faction_apply_log_enabled: bool,
    pub(crate) write_faction_apply_log:
        &'a mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
    /// Внешний feature-gate `CLogSystem::FactionJoinEnabled`.
    pub(crate) faction_join_log_enabled: bool,
    pub(crate) write_faction_join_log:
        &'a mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
    /// Внешний feature-gate `CLogSystem::FactionQuitEnabled`.
    pub(crate) faction_quit_log_enabled: bool,
    pub(crate) write_faction_quit_log:
        &'a mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
    /// Внешний feature-gate `CLogSystem::FactionFireOutEnabled`.
    pub(crate) faction_fire_out_log_enabled: bool,
    pub(crate) write_faction_fire_out_log:
        &'a mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
    /// Внешний feature-gate `CLogSystem::FactionMasterChangedEnabled`.
    pub(crate) faction_master_log_enabled: bool,
    pub(crate) write_faction_master_log:
        &'a mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8]),
    /// Внешний feature-gate `CLogSystem::FactionDisbandEnabled`.
    pub(crate) faction_disband_log_enabled: bool,
    pub(crate) write_faction_disband_log:
        &'a mut dyn FnMut(i32, &[u8], i32, &[u8]),
    pub(crate) dispatch_timer:
        &'a mut dyn FnMut(&mut CTimer<TimerCallback>, TimerCallbackInvocation<TimerCallback>),
    pub(crate) get_lei_ting_local_time: &'a mut dyn FnMut() -> LeiTingLocalTime,
    pub(crate) wait: &'a mut dyn FnMut(u32),
    pub(crate) output_debug: &'a mut dyn FnMut(&'static str),
}

/// Первый недоказанный/невозвращающийся участок полного MainLoop.
pub(crate) enum WorldMainLoopBlock<FactionContextBlock, LeiTingContextBlock> {
    Largess(WorldMainLoopLargessGateReport),
    Refresh(WorldMainLoopRefreshStageReport),
    Reload(WorldReloadProfilesReport),
    Maintenance(WorldMainLoopMaintenanceBlock),
    SaveAllOrganizations {
        block: OrganizingSaveDataBlock,
    },
    ImmediateSave {
        log: AddLogTextDisposition,
        block: WorldGenerateDbDataBlock,
    },
    ProcessMessage(WorldProcessMessageStageReport),
    PlayerDataQueue(WorldMainLoopPlayerDataQueueStageReport),
    Timer(WorldMainLoopTimerStageBlock),
    FactionWar(FactionWarStopBlock<FactionContextBlock>),
    LeiTing(LeiTingBlock<LeiTingContextBlock>),
    DbMisc(DbMiscDoneOutBlock),
    Ping(WorldMainLoopPingError),
    Minute(WorldMainLoopMinuteStageBlock),
    Jjc(JjcRunBlock),
    Tail(WorldMainLoopPacingReport),
}

pub(crate) type WorldMainLoopResult<FactionContextBlock, LeiTingContextBlock> =
    Result<WorldMainLoopReport, Box<WorldMainLoopBlock<FactionContextBlock, LeiTingContextBlock>>>;

/// Один полностью возвращённый `CGame::MainLoop`, включая все ordered stages.
#[derive(Debug)]
pub(crate) struct WorldMainLoopReport {
    pub(crate) profile_initialization: Option<WorldMainLoopProfileInitialization>,
    pub(crate) save_initialization: Option<WorldMainLoopSaveInitialization>,
    pub(crate) refresh_initialization: Option<WorldMainLoopRefreshInitialization>,
    pub(crate) current_tick_ms: u32,
    pub(crate) largess: WorldMainLoopLargessGateReport,
    pub(crate) refresh_profile_started_at_ms: u32,
    pub(crate) refresh: WorldMainLoopRefreshStageReport,
    pub(crate) reload: WorldReloadProfilesReport,
    pub(crate) maintenance: WorldMainLoopMaintenanceReport,
    pub(crate) collect_player_data: Option<WorldCollectPlayerDataBroadcast>,
    pub(crate) save: WorldMainLoopSaveStageReport,
    pub(crate) ai: WorldMainLoopAiStageReport,
    pub(crate) process_message: WorldProcessMessageStageReport,
    pub(crate) session_factory: WorldMainLoopSessionFactoryStageReport,
    pub(crate) player_data_queue: WorldMainLoopPlayerDataQueueStageReport,
    pub(crate) timer: WorldMainLoopTimerStageReport,
    pub(crate) faction_war: WorldMainLoopFactionWarStageReport,
    pub(crate) lei_ting: LeiTingRunReport,
    pub(crate) db_misc: WorldMainLoopDbMiscStageReport,
    pub(crate) net_sessions: WorldMainLoopNetSessionStageReport,
    pub(crate) ping: WorldMainLoopPingStageReport,
    pub(crate) minute: WorldMainLoopMinuteStageReport,
    pub(crate) bai_tan_jjc: WorldMainLoopBaiTanJjcStageReport,
    pub(crate) tail: WorldMainLoopTailStageReport,
    pub(crate) legacy_result: i32,
}

/// Наблюдаемый результат одной MainLoop-проверки GameServer ping.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldMainLoopPingStageReport {
    Idle,
    Waiting {
        connected_game_servers: i32,
        received_responses: u32,
        elapsed_ms: u32,
    },
    Published {
        connected_game_servers: i32,
        received_responses: u32,
        elapsed_ms: u32,
        all_connected_responded: bool,
        timed_out: bool,
        online_players: u32,
        delivery: Result<i32, SendMessageError>,
    },
}

/// Размер Rust-коллекции, не представимый старым 32-битным container size.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldMainLoopPingError {
    ResponseCountOutsideLegacyRange { count: usize },
    OnlinePlayerCountOutsideLegacyRange { count: usize },
}

/// Выполненная установка bit `1` и первый last-report tick.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldMainLoopProfileInitialization {
    pub(crate) previous_mask: u32,
    pub(crate) initialized_mask: u32,
    pub(crate) initial_report_tick_ms: u32,
}

/// Выполненная установка bit `2` и первый last-save tick.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldMainLoopSaveInitialization {
    pub(crate) previous_mask: u32,
    pub(crate) initialized_mask: u32,
    pub(crate) initial_save_tick_ms: u32,
}

/// Выполненная установка bit `4` и копия прежнего current tick.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldMainLoopRefreshInitialization {
    pub(crate) previous_mask: u32,
    pub(crate) initialized_mask: u32,
    pub(crate) copied_current_tick_ms: u32,
}

/// Счётчики соседних owners, которые `RefeashInfoText` читал как globals.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorldRefreshExternalCounts {
    pub(crate) team_sessions: i32,
    pub(crate) largess_entries: u32,
    pub(crate) reback_messages: i32,
}

/// Непредставимый в старом 32-битном container-count safe Rust owner-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldRefreshSnapshotBlock {
    pub(crate) field: &'static str,
    pub(crate) count: usize,
}

/// Выполнился ли strict refresh gate текущего MainLoop turn.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldMainLoopRefreshDisposition {
    NotDue,
    MissingNetworkOwner,
    Refreshed(WorldRefreshInfoReport),
}

/// Цельный участок refresh -> profiling accumulator -> 600s publish gate.
#[allow(
    clippy::large_enum_variant,
    reason = "полный отчёт возвращается по значению, чтобы не добавлять heap allocation в каждый MainLoop turn"
)]
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldMainLoopRefreshStageReport {
    BlockedMissingFact {
        elapsed_since_refresh_ms: u32,
        assigned_last_refresh_tick_ms: u32,
        block: WorldRefreshSnapshotBlock,
    },
    Complete {
        elapsed_since_refresh_ms: u32,
        refresh: WorldMainLoopRefreshDisposition,
        finished_at_ms: u32,
        elapsed_stage_ms: u32,
        accumulated_refresh_time_ms: u32,
        profile: Option<WorldMainLoopProfileReport>,
    },
}

/// Диагностический снимок двух отдельно прочитанных половин reload-флага.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorldReloadProfileFlagsSnapshot {
    pub(crate) low: u32,
    pub(crate) high: u32,
}

/// Safe Linux-owner двух независимо читаемых/записываемых 32-битных половин.
#[derive(Debug, Default)]
pub(crate) struct WorldReloadProfileFlags {
    low: AtomicU32,
    high: AtomicU32,
}

/// Прямой соседний owner, который `CGame::ReLoad` вызывал с boolean-result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldReloadBooleanOwner {
    GoodsList,
    MonsterList,
    DropGoodsList,
    SkillUsageCache,
    SkillCache,
    GlobeSetup,
    GameSetup,
    LogSystem,
    GmList,
    PlayerGmList,
    RegionLevelSetup,
    AttackCity,
    FourNationWar,
    TimeToReturn,
    PreciousBox,
    FairyExp,
    ChangeBody,
    DaKongXiangQian,
    GodsBattle,
}

/// Прямой соседний owner без наблюдаемого return в исходном dispatcher-е.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldReloadVoidOwner {
    LoadOrganizingParameters,
    ReinitializeFactionsByLevel,
    VillageWar,
    AttackCityUnchecked,
    FactionWarParameters,
    Quest,
    CountryParameters,
    LingBao,
    GodsBattleNpcFaction,
}

/// Владелец точного payload, который следует за успешной reload-операцией.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldReloadSerializationOwner {
    GoodsList,
    MonsterList,
    SkillList,
    GlobeSetup,
    LogSystem,
    GmList,
    RegionLevelSetup,
    AttackCity,
    VillageWar,
    FourNationWar,
    Quest,
    PreciousBox,
    FairyExp,
    ChangeBody,
    DaKongXiangQian,
    LingBao,
    GodsBattle,
}

/// Доказанный positional record `setup/regionlist.ini` до virtual region-owner-а.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorldRegionLoadSpec {
    pub(crate) region_id: i32,
    pub(crate) resource_id: u32,
    pub(crate) exp_scale: f32,
    pub(crate) region_type: i32,
    pub(crate) no_pk: bool,
    pub(crate) no_contribute: bool,
    pub(crate) name: Vec<u8>,
    pub(crate) game_server_index: u32,
    pub(crate) country: u8,
    pub(crate) notify: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldReloadOneScriptBlock {
    pub(crate) requested_path: Vec<u8>,
    pub(crate) normalized_map_key: Vec<u8>,
}

pub(crate) type WorldReloadOneScriptResult = Result<bool, WorldReloadOneScriptBlock>;

/// Resource/domain границы, непосредственно вызываемые готовым `CGame::ReLoad`.
pub(crate) trait WorldReloadContext: WorldRegionResourceContext {
    fn load_reload_server_resources(&mut self, game: &mut CGame);
    /// Отдельный mutable owner исторических static `CPlayerList` data.
    ///
    /// Он остаётся вне `CGame`, поскольку тот же экземпляр участвует в
    /// create-role и DB-load runtime; это исключает расходящиеся config копии.
    fn player_list(&mut self) -> &mut CPlayerList;
    /// Отдельный owner правил уничтожения предметов, разделяемый с initial-config.
    fn goods_destroy_setup(&mut self) -> &mut GoodsDestroySetup;
    /// Отдельный owner списков монстров новых навыков для reload и initial-config.
    fn new_skill_monster_conf(&mut self) -> &mut NewSkillMonsterConf;
    /// Таблица опыта боевых духов, общая для reload и initial-config wire.
    fn battle_fairy_exp_config(&mut self) -> &mut CBattleFairyExpConfig;
    /// Конфигурация объединения боевых духов, общая для reload и `0x2D` wire.
    fn battle_fairy_property(&mut self) -> &mut CBattleFairyProperty;
    /// Статические map/vector синтеза, shared с игровыми запросами и reload wire.
    fn synthesis(&mut self) -> &mut CSynthesis;
    /// Два scalar-а honor-eliminate, общие для runtime и initial-config wire.
    fn honor_eliminate_config(&mut self) -> &mut HonorElimilateConfig;
    /// Возвращает исходный 32-битный result; bool owners обязаны дать `0/1`.
    fn call_boolean_owner(&mut self, owner: WorldReloadBooleanOwner) -> u32;
    fn call_void_owner(&mut self, owner: WorldReloadVoidOwner);
    fn serialize_owner(&mut self, owner: WorldReloadSerializationOwner) -> Vec<u8>;
    /// Concrete lookup уже загруженного World `CGoodsFactory`.
    fn query_goods_id_by_original_name(&mut self, original_name: &[u8]) -> u32;
    /// Concrete display-name lookup того же `CGoodsFactory`.
    fn query_goods_name(&mut self, goods_id: u32) -> Option<Vec<u8>>;
    fn add_log_text(&mut self, payload: &[u8]);
    fn notify_reload_operator(&mut self, title: &[u8], message: &[u8]);

    /// Возвращает script paths в порядке конкретного resource-owner-а.
    ///
    /// Пока package-resource ещё не материализован, default является безопасной
    /// host-filesystem заменой Win32 `FindScriptFile`. Будущий resource owner
    /// может переопределить метод, не меняя script-loading контракт `CGame`.
    fn script_files(&mut self, pattern: &[u8], extension: &[u8]) -> Vec<Vec<u8>> {
        find_script_files(pattern, extension)
    }
    /// Выполняет оставшийся inline-parser `setup/sysboardcast.ini`, включая
    /// operator notice, random/tick и единственный success log.
    fn reload_broadcast_list(&mut self, game: &mut CGame);

    /// Сохраняет два process-global счётчика после прямого region-owner load.
    fn add_region_object_counts(&mut self, monsters: i32, npcs: i32) -> (i32, i32);
    fn region_object_counts(&mut self) -> (i32, i32);
}

/// Рекурсивно собирает host-файлы старого `FindScriptFile`.
///
/// `walkdir` заменяет `FindFirstFileA/FindNextFileA/FindClose` и ручную
/// рекурсию. Symlink-каталоги не обходятся: циклическая ссылка была внутренним
/// дефектом неограниченной C++-рекурсии, а не Miracle-контрактом. Фильтр
/// расширения остаётся ASCII case-insensitive, полный возвращаемый путь —
/// lowercase с `/`, как `_strlwr` плюс последующая нормализация map-key.
pub(crate) fn find_script_files(pattern: &[u8], extension: &[u8]) -> Vec<Vec<u8>> {
    let pattern = legacy_c_string_prefix(pattern)
        .iter()
        .map(|byte| if *byte == b'\\' { b'/' } else { *byte })
        .collect::<Vec<_>>();
    let root = script_search_root(&pattern);
    let requested_extension = legacy_c_string_prefix(extension)
        .strip_prefix(b".")
        .unwrap_or_else(|| legacy_c_string_prefix(extension));

    let mut files = WalkDir::new(legacy_path_from_bytes(root))
        .min_depth(1)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let extension = entry.path().extension()?;
            let extension = legacy_path_component_bytes(extension);
            if !extension.eq_ignore_ascii_case(requested_extension) {
                return None;
            }
            let mut path = legacy_path_bytes(entry.path());
            for byte in &mut path {
                if *byte == b'\\' {
                    *byte = b'/';
                } else {
                    byte.make_ascii_lowercase();
                }
            }
            Some(path)
        })
        .collect::<Vec<_>>();
    // Win32 не обещал directory order. Стабильная сортировка устраняет только
    // внутреннюю зависимость от host FS; wire всё равно публикуется из BTreeMap.
    files.sort_unstable();
    files
}

fn script_search_root(pattern: &[u8]) -> &[u8] {
    let wildcard = pattern.iter().position(|byte| matches!(*byte, b'*' | b'?'));
    let parent_end = wildcard
        .and_then(|position| pattern[..position].iter().rposition(|byte| *byte == b'/'))
        .or_else(|| pattern.iter().rposition(|byte| *byte == b'/'));
    match parent_end {
        Some(0) => b"/",
        Some(end) => &pattern[..end],
        None => b".",
    }
}

#[cfg(unix)]
fn legacy_path_from_bytes(bytes: &[u8]) -> PathBuf {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    PathBuf::from(OsString::from_vec(bytes.to_vec()))
}

#[cfg(not(unix))]
fn legacy_path_from_bytes(bytes: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(bytes).into_owned())
}

#[cfg(unix)]
fn legacy_path_component_bytes(value: &std::ffi::OsStr) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;

    value.as_bytes().to_vec()
}

#[cfg(not(unix))]
fn legacy_path_component_bytes(value: &std::ffi::OsStr) -> Vec<u8> {
    value.to_string_lossy().as_bytes().to_vec()
}

#[cfg(unix)]
fn legacy_path_bytes(path: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;

    path.as_os_str().as_bytes().to_vec()
}

#[cfg(not(unix))]
fn legacy_path_bytes(path: &Path) -> Vec<u8> {
    path.to_string_lossy().as_bytes().to_vec()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorldReloadProfile {
    PlayerList,
    GoodsList,
    MonsterList,
    TradeList,
    SkillList,
    NewSkillMonsterList,
    GlobeSetup,
    GameSetup,
    StringTable,
    LogSystem,
    GmList,
    ScriptFile,
    RegionList,
    RegionLevelSetup,
    HitLevelSetup,
    Broadcast,
    AttackCity,
    InvalidStrings,
    GeneralVariableList,
    FactionParameters,
    VillageWar,
    FourNationWar,
    CityWar,
    FactionWar,
    Quest,
    CountryParameters,
    IncrementShop,
    Contribute,
    Prison,
    TimeToReturn,
    PreciousBox,
    FairyExp,
    ChangeBody,
    CountryWar,
    BattleFairyExp,
    BattleFairyCombine,
    Synthesis,
    DaKongXiangQian,
    EquipmentCompose,
    GoodsDestroy,
    HonorEliminate,
    TaoZhuang,
    CiQing,
    Jjc,
    AllThing,
    GodsBattle,
}

impl WorldReloadProfile {
    fn parse(value: &[u8]) -> Option<Self> {
        const NAMES: &[(&[u8], WorldReloadProfile)] = &[
            (b"PlayerList", WorldReloadProfile::PlayerList),
            (b"GoodsList", WorldReloadProfile::GoodsList),
            (b"MonsterList", WorldReloadProfile::MonsterList),
            (b"TradeList", WorldReloadProfile::TradeList),
            (b"SkillList", WorldReloadProfile::SkillList),
            (
                b"NewSkillMonsterList",
                WorldReloadProfile::NewSkillMonsterList,
            ),
            (b"GlobeSetup", WorldReloadProfile::GlobeSetup),
            (b"GameSetup", WorldReloadProfile::GameSetup),
            (b"StringTable", WorldReloadProfile::StringTable),
            (b"LogSystem", WorldReloadProfile::LogSystem),
            (b"GMList", WorldReloadProfile::GmList),
            (b"ScriptFile", WorldReloadProfile::ScriptFile),
            (b"RegionList", WorldReloadProfile::RegionList),
            (b"RegionLevelSetup", WorldReloadProfile::RegionLevelSetup),
            (b"HitLevelSetup", WorldReloadProfile::HitLevelSetup),
            (b"Broadcast", WorldReloadProfile::Broadcast),
            (b"AttackCitySys", WorldReloadProfile::AttackCity),
            (b"InvalidStr", WorldReloadProfile::InvalidStrings),
            (
                b"GeneralVariableList",
                WorldReloadProfile::GeneralVariableList,
            ),
            (b"FactionPara", WorldReloadProfile::FactionParameters),
            (b"VilWarPara", WorldReloadProfile::VillageWar),
            (b"FourNationWar", WorldReloadProfile::FourNationWar),
            (b"CityWarPara", WorldReloadProfile::CityWar),
            (b"FactionWarPara", WorldReloadProfile::FactionWar),
            (b"QuestData", WorldReloadProfile::Quest),
            (b"CountryParam", WorldReloadProfile::CountryParameters),
            (b"CountryPara", WorldReloadProfile::CountryParameters),
            (b"IncrementShopList", WorldReloadProfile::IncrementShop),
            (b"ContributeSetup", WorldReloadProfile::Contribute),
            (b"PrisonConf", WorldReloadProfile::Prison),
            (b"TimeToReturn", WorldReloadProfile::TimeToReturn),
            (b"PreciousBoxConf", WorldReloadProfile::PreciousBox),
            (b"FairyExpConf", WorldReloadProfile::FairyExp),
            (b"ChangeBodyConf", WorldReloadProfile::ChangeBody),
            (b"CountryWar", WorldReloadProfile::CountryWar),
            (b"BattleFairyExpConfig", WorldReloadProfile::BattleFairyExp),
            (
                b"BattleFairyCombineConfig",
                WorldReloadProfile::BattleFairyCombine,
            ),
            (b"SynthesisList", WorldReloadProfile::Synthesis),
            (b"DaKongXiangQian", WorldReloadProfile::DaKongXiangQian),
            (b"EquipmentCompose", WorldReloadProfile::EquipmentCompose),
            (b"GoodsDestroyConf", WorldReloadProfile::GoodsDestroy),
            (b"HonorElimilate", WorldReloadProfile::HonorEliminate),
            (b"taozhuang", WorldReloadProfile::TaoZhuang),
            (b"ciqing", WorldReloadProfile::CiQing),
            (b"JJcConfig", WorldReloadProfile::Jjc),
            (b"Allthing", WorldReloadProfile::AllThing),
            (b"godsBattle", WorldReloadProfile::GodsBattle),
        ];
        let value = legacy_c_string_prefix(value);
        NAMES
            .iter()
            .find(|(name, _)| value.eq_ignore_ascii_case(name))
            .map(|(_, profile)| *profile)
    }
}

impl WorldReloadProfileFlags {
    /// Создаёт точное начальное low/high состояние без объединения в один RMW.
    pub(crate) const fn new(low: u32, high: u32) -> Self {
        Self {
            low: AtomicU32::new(low),
            high: AtomicU32::new(high),
        }
    }

    /// Снимает диагностический low/high snapshot двумя отдельными чтениями.
    pub(crate) fn snapshot(&self) -> WorldReloadProfileFlagsSnapshot {
        WorldReloadProfileFlagsSnapshot {
            low: self.low.load(Ordering::Relaxed),
            high: self.high.load(Ordering::Relaxed),
        }
    }

    /// Повторяет producer-последовательность `load low -> OR -> store low`.
    pub(crate) fn set_low_bits(&self, mask: u32) -> u32 {
        let updated = self.low.load(Ordering::Relaxed) | mask;
        self.low.store(updated, Ordering::Relaxed);
        updated
    }

    /// Повторяет producer-последовательность `load high -> OR -> store high`.
    pub(crate) fn set_high_bits(&self, mask: u32) -> u32 {
        let updated = self.high.load(Ordering::Relaxed) | mask;
        self.high.store(updated, Ordering::Relaxed);
        updated
    }

    fn has_pending(&self) -> bool {
        self.low.load(Ordering::Relaxed) != 0 || self.high.load(Ordering::Relaxed) != 0
    }

    fn contains(&self, half: WorldReloadFlagHalf, mask: u32) -> bool {
        match half {
            WorldReloadFlagHalf::Low => self.low.load(Ordering::Relaxed) & mask != 0,
            WorldReloadFlagHalf::High => self.high.load(Ordering::Relaxed) & mask != 0,
        }
    }

    fn consume(&self, action: WorldReloadAction) {
        match action.half {
            WorldReloadFlagHalf::Low => {
                let remaining = self.low.load(Ordering::Relaxed) & !action.mask;
                self.low.store(remaining, Ordering::Relaxed);
                if action.zero_high_after_low {
                    self.high.store(0, Ordering::Relaxed);
                }
            }
            WorldReloadFlagHalf::High => {
                let remaining = self.high.load(Ordering::Relaxed) & !action.mask;
                self.high.store(remaining, Ordering::Relaxed);
            }
        }
    }
}

/// Половина исторического 64-битного reload-флага, проверенная веткой.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldReloadFlagHalf {
    Low,
    High,
}

/// Конкретная safe-граница `reload_conf_log`, где исходник уходил в UB.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldReloadConfLogBlock {
    MessageOutsideLegacyStackBuffer { required_bytes_with_nul: usize },
    MissingNetworkServerOwner,
    MissingWorldNumber,
}

/// Наблюдаемый итог отдельного `reload_conf_log`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldReloadConfLogDisposition {
    SuppressedEmptyProfile,
    Published {
        text: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

/// Одна выполненная ветка `reload_profiles` после reload и operator-message.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldReloadProfileEvent {
    pub(crate) half: WorldReloadFlagHalf,
    pub(crate) mask: u32,
    pub(crate) reload_profile: &'static [u8],
    pub(crate) log_profile: &'static [u8],
    pub(crate) flags_after_clear: WorldReloadProfileFlagsSnapshot,
    pub(crate) reload_result: i32,
    pub(crate) log: WorldReloadConfLogDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldReloadRegionSetupBlock {
    pub(crate) region_id: i32,
    pub(crate) source: WorldRegionSetupSerializationBlock,
}

/// Итог одного последовательного dispatcher-прохода.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldReloadProfilesReport {
    Complete {
        events: Vec<WorldReloadProfileEvent>,
        remaining_flags: WorldReloadProfileFlagsSnapshot,
    },
    BlockedMissingFact {
        completed_events: Vec<WorldReloadProfileEvent>,
        half: WorldReloadFlagHalf,
        mask: u32,
        reload_profile: &'static [u8],
        log_profile: &'static [u8],
        flags_after_clear: WorldReloadProfileFlagsSnapshot,
        reload_result: i32,
        block: WorldReloadConfLogBlock,
    },
    BlockedRegionSetup {
        completed_events: Vec<WorldReloadProfileEvent>,
        half: WorldReloadFlagHalf,
        mask: u32,
        flags_after_clear: WorldReloadProfileFlagsSnapshot,
        block: WorldReloadRegionSetupBlock,
    },
    BlockedReloadOwner {
        completed_events: Vec<WorldReloadProfileEvent>,
        half: WorldReloadFlagHalf,
        mask: u32,
        reload_profile: &'static [u8],
        log_profile: &'static [u8],
        flags_after_clear: WorldReloadProfileFlagsSnapshot,
        block: WorldReloadBlock,
    },
}

#[derive(Clone, Copy)]
enum WorldReloadActionKind {
    Reload,
    ReloadAllRegions,
}

#[derive(Clone, Copy)]
struct WorldReloadAction {
    half: WorldReloadFlagHalf,
    mask: u32,
    zero_high_after_low: bool,
    reload_profile: &'static [u8],
    log_profile: &'static [u8],
    first_option: bool,
    second_option: bool,
    kind: WorldReloadActionKind,
}

impl WorldReloadAction {
    const fn reload_low(
        mask: u32,
        profile: &'static [u8],
        first_option: bool,
        zero_high_after_low: bool,
    ) -> Self {
        Self {
            half: WorldReloadFlagHalf::Low,
            mask,
            zero_high_after_low,
            reload_profile: profile,
            log_profile: profile,
            first_option,
            second_option: true,
            kind: WorldReloadActionKind::Reload,
        }
    }

    const fn reload_high(mask: u32, profile: &'static [u8]) -> Self {
        Self {
            half: WorldReloadFlagHalf::High,
            mask,
            zero_high_after_low: false,
            reload_profile: profile,
            log_profile: profile,
            first_option: true,
            second_option: true,
            kind: WorldReloadActionKind::Reload,
        }
    }

    const fn reload_high_with_log(
        mask: u32,
        reload_profile: &'static [u8],
        log_profile: &'static [u8],
    ) -> Self {
        Self {
            half: WorldReloadFlagHalf::High,
            mask,
            zero_high_after_low: false,
            reload_profile,
            log_profile,
            first_option: true,
            second_option: true,
            kind: WorldReloadActionKind::Reload,
        }
    }

    const fn reload_all_regions(mask: u32) -> Self {
        Self {
            half: WorldReloadFlagHalf::Low,
            mask,
            zero_high_after_low: true,
            reload_profile: b"AllRegion",
            log_profile: b"AllRegion",
            first_option: false,
            second_option: false,
            kind: WorldReloadActionKind::ReloadAllRegions,
        }
    }
}

const WORLD_RELOAD_ACTIONS: &[WorldReloadAction] = &[
    WorldReloadAction::reload_low(0x4000_0000, b"StringTable", true, true),
    WorldReloadAction::reload_low(0x0000_0001, b"LogSystem", true, true),
    WorldReloadAction::reload_low(0x0000_0002, b"GMList", true, true),
    WorldReloadAction::reload_low(0x0000_0004, b"Broadcast", false, true),
    WorldReloadAction::reload_low(0x0000_0008, b"VilWarPara", false, true),
    WorldReloadAction::reload_low(0x0000_0010, b"CityWarPara", false, true),
    WorldReloadAction::reload_low(0x0000_0020, b"IncrementShopList", true, true),
    WorldReloadAction::reload_low(0x0000_0040, b"GameSetup", true, true),
    WorldReloadAction::reload_low(0x0000_0080, b"InvalidStr", true, true),
    WorldReloadAction::reload_low(0x0000_0100, b"PlayerList", true, true),
    WorldReloadAction::reload_low(0x0000_0200, b"GoodsList", true, true),
    WorldReloadAction::reload_low(0x0000_0400, b"MonsterList", true, true),
    WorldReloadAction::reload_low(0x0000_0800, b"TradeList", true, true),
    WorldReloadAction::reload_low(0x0000_1000, b"SkillList", true, true),
    WorldReloadAction::reload_low(0x0000_2000, b"GlobeSetup", true, true),
    WorldReloadAction::reload_low(0x0000_4000, b"ScriptFile", true, true),
    WorldReloadAction::reload_high(0x0000_0001, b"NewSkillMonsterList"),
    WorldReloadAction::reload_low(0x0001_0000, b"GeneralVariableList", false, true),
    WorldReloadAction::reload_low(0x0002_0000, b"RegionList", true, true),
    WorldReloadAction::reload_low(0x0004_0000, b"RegionLevelSetup", true, true),
    WorldReloadAction::reload_all_regions(0x0010_0000),
    WorldReloadAction::reload_low(0x0020_0000, b"FactionPara", false, true),
    WorldReloadAction::reload_low(0x0040_0000, b"FactionWarPara", false, true),
    WorldReloadAction::reload_low(0x0080_0000, b"QuestData", false, true),
    WorldReloadAction::reload_low(0x0100_0000, b"ContributeSetup", true, true),
    WorldReloadAction::reload_low(0x0200_0000, b"PrisonConf", true, true),
    WorldReloadAction::reload_low(0x0400_0000, b"TimeToReturn", false, true),
    WorldReloadAction::reload_low(0x0800_0000, b"PreciousBoxConf", true, true),
    WorldReloadAction::reload_low(0x2000_0000, b"FairyExpConf", true, true),
    WorldReloadAction::reload_low(0x8000_0000, b"ChangeBodyConf", true, false),
    WorldReloadAction::reload_low(0x1000_0000, b"CountryWar", false, true),
    WorldReloadAction::reload_high(0x0000_0020, b"FourNationWar"),
    WorldReloadAction::reload_high(0x0000_0400, b"BattleFairyExpConfig"),
    WorldReloadAction::reload_high(0x0000_0800, b"BattleFairyCombineConfig"),
    WorldReloadAction::reload_low(0x5000_0000, b"SynthesisList", true, false),
    WorldReloadAction::reload_high(0x0000_0080, b"EquipmentCompose"),
    WorldReloadAction::reload_high(0x0000_1000, b"HonorElimilate"),
    WorldReloadAction::reload_high(0x0000_2000, b"ciqing"),
    WorldReloadAction::reload_high_with_log(0x0001_0000, b"godsBattle", b"godsbattle"),
    WorldReloadAction::reload_high(0x0000_8000, b"taozhuang"),
    WorldReloadAction::reload_high(0x0000_4000, b"JJcConfig"),
    WorldReloadAction::reload_low(0x0000_0100, b"Allthing", true, false),
];

/// Потокобезопасный одноразовый запрос пересчёта PlayerRanks.
#[derive(Debug, Default)]
pub(crate) struct WorldPlayerRanksRequestState {
    requested: AtomicBool,
}

impl WorldPlayerRanksRequestState {
    /// Устанавливает запрос, не меняя уже установленное состояние.
    pub(crate) fn request(&self) {
        self.requested.store(true, Ordering::Relaxed);
    }

    /// Возвращает текущее значение request-флага.
    pub(crate) fn is_requested(&self) -> bool {
        self.requested.load(Ordering::Relaxed)
    }

    fn take_if_requested(&self) -> bool {
        if !self.requested.load(Ordering::Relaxed) {
            return false;
        }
        self.requested.store(false, Ordering::Relaxed);
        true
    }
}

/// Итог ручной PlayerRanks-ветви одного MainLoop turn.
#[derive(Debug)]
pub(crate) enum WorldPlayerRanksMaintenanceDisposition {
    NotRequested,
    Updated {
        stat: PlayerRanksStatRunReport,
        publication: PlayerRanksGameServerUpdate,
    },
}

/// Полный exact `CPlayerRanks::StatPlayerRanks` без последующей публикации.
#[derive(Debug)]
pub(crate) struct PlayerRanksStatRunReport {
    pub(crate) started_at_ms: u32,
    pub(crate) finished_at_ms: u32,
    pub(crate) elapsed_ms: u32,
    pub(crate) outcome: PlayerRanksStatOutcome,
    pub(crate) start_log: AddLogTextDisposition,
    pub(crate) complete_log: AddLogTextDisposition,
}

/// Выполненный prefix `StatPlayerRanks` перед безопасной unknown-границей.
#[derive(Debug)]
pub(crate) struct PlayerRanksStatRunBlock {
    pub(crate) started_at_ms: u32,
    pub(crate) start_log: AddLogTextDisposition,
    pub(crate) source: PlayerRanksStatBlock,
}

/// Итог optional daily HonorRanks-ветви.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldHonorRanksMaintenanceDisposition {
    Disabled,
    AlreadyCurrent {
        current_day: u32,
        sort_day: u32,
    },
    Updated {
        current_day: u32,
        previous_sort_day: u32,
        started_at_ms: u32,
        finished_at_ms: u32,
        elapsed_ms: u32,
        start_log: AddLogTextDisposition,
        complete_log: AddLogTextDisposition,
        rollover: HonorRanksNewDayReport,
    },
}

/// Safe-граница реального `CHonorRanks::OnNewDay` внутри MainLoop.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldHonorRanksMaintenanceBlock {
    pub(crate) current_day: u32,
    pub(crate) previous_sort_day: u32,
    pub(crate) started_at_ms: u32,
    pub(crate) start_log: AddLogTextDisposition,
    pub(crate) source: HonorRanksNewDayBlock,
}

/// Итог безусловно достигнутого AuctionBang day gate.
#[derive(Debug)]
pub(crate) enum WorldAuctionBangMaintenanceDisposition {
    AlreadyCurrent {
        current_month_day: i32,
        old_month_day: i32,
    },
    Updated {
        current_month_day: i32,
        previous_old_month_day: i32,
        update_succeeded: bool,
        outcome: AuctionBangUpdateOutcome,
        start_log: AddLogTextDisposition,
        result_log: AddLogTextDisposition,
    },
}

/// Первая безопасно неразрешимая граница maintenance-блока.
#[derive(Debug)]
pub(crate) enum WorldMainLoopMaintenanceBlock {
    PlayerRanksStat(PlayerRanksStatRunBlock),
    PlayerRanksSerialization(PlayerRanksSerializationBlock),
    HonorRanks(WorldHonorRanksMaintenanceBlock),
    AuctionOldDayUnknown { current_month_day: i32 },
}

/// Полный maintenance-сегмент между reload и collect-player-data.
#[derive(Debug)]
pub(crate) struct WorldMainLoopMaintenanceReport {
    pub(crate) player_ranks: WorldPlayerRanksMaintenanceDisposition,
    pub(crate) honor_ranks: WorldHonorRanksMaintenanceDisposition,
    pub(crate) auction_bang: WorldAuctionBangMaintenanceDisposition,
}

/// Результат одного Largess gate без исполнения отдельного worker-owner-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldMainLoopLargessGateReport {
    BlockedMissingFact {
        field: &'static str,
    },
    Disabled {
        load_interval_ms: u32,
        pass_count: u32,
    },
    Waiting {
        load_interval_ms: u32,
        pass_count: u32,
        elapsed_ms: u32,
    },
    StartWorkerRequested {
        load_interval_ms: u32,
        pass_count: u32,
        elapsed_ms: u32,
        requested_at_ms: u32,
    },
}

/// Снимок двенадцати bit-pattern-ов до точного reset profiling-окна.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldMainLoopProfileSnapshot {
    pub(crate) ai_calls: u32,
    pub(crate) ai_time_ms: u32,
    pub(crate) refresh_text_time_ms: u32,
    pub(crate) process_message_time_ms: u32,
    pub(crate) login_server_message_time_ms: u32,
    pub(crate) game_server_message_time_ms: u32,
    pub(crate) net_session_time_ms: u32,
    pub(crate) faction_war_time_ms: u32,
    pub(crate) timer_time_ms: u32,
    pub(crate) process_player_data_queue_time_ms: u32,
    pub(crate) session_factory_time_ms: u32,
    pub(crate) save_point_time_ms: u32,
}

/// Опубликованное 600-секундное окно и точный logger-результат.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldMainLoopProfileReport {
    pub(crate) elapsed_since_last_publish_ms: u32,
    pub(crate) snapshot: WorldMainLoopProfileSnapshot,
    pub(crate) log: AddLogTextDisposition,
}

/// Один раз устанавливает profiling bit и снимает initial last-report tick.
pub(crate) fn initialize_main_loop_profile_if_needed<GetTick>(
    initialization: &mut WorldMainLoopInitializationState,
    profile: &mut WorldMainLoopProfileState,
    mut get_tick: GetTick,
) -> Option<WorldMainLoopProfileInitialization>
where
    GetTick: FnMut() -> u32,
{
    if initialization.mask & 1 != 0 {
        return None;
    }

    let previous_mask = initialization.mask;
    initialization.mask |= 1;
    let initial_report_tick_ms = get_tick();
    profile.last_published_at_ms = initial_report_tick_ms;
    Some(WorldMainLoopProfileInitialization {
        previous_mask,
        initialized_mask: initialization.mask,
        initial_report_tick_ms,
    })
}

/// Один раз устанавливает save bit и снимает initial last-save tick.
pub(crate) fn initialize_main_loop_save_if_needed<GetTick>(
    initialization: &mut WorldMainLoopInitializationState,
    save: &mut WorldRunSaveTriggerState,
    mut get_tick: GetTick,
) -> Option<WorldMainLoopSaveInitialization>
where
    GetTick: FnMut() -> u32,
{
    if initialization.mask & 2 != 0 {
        return None;
    }

    let previous_mask = initialization.mask;
    initialization.mask |= 2;
    let initial_save_tick_ms = get_tick();
    save.last_save_point_time_ms = initial_save_tick_ms;
    Some(WorldMainLoopSaveInitialization {
        previous_mask,
        initialized_mask: initialization.mask,
        initial_save_tick_ms,
    })
}

/// Один раз копирует прежний current tick в initial refresh tick.
pub(crate) fn initialize_main_loop_refresh_if_needed(
    initialization: &mut WorldMainLoopInitializationState,
    clocks: &mut WorldMainLoopClockState,
) -> Option<WorldMainLoopRefreshInitialization> {
    if initialization.mask & 4 != 0 {
        return None;
    }

    let previous_mask = initialization.mask;
    initialization.mask |= 4;
    let copied_current_tick_ms = clocks.current_tick_ms;
    clocks.last_refresh_tick_ms = copied_current_tick_ms;
    Some(WorldMainLoopRefreshInitialization {
        previous_mask,
        initialized_mask: initialization.mask,
        copied_current_tick_ms,
    })
}

/// Снимает один tick, сохраняет его как текущий и возвращает caller-у.
pub(crate) fn update_main_loop_current_tick<GetTick>(
    clocks: &mut WorldMainLoopClockState,
    mut get_tick: GetTick,
) -> u32
where
    GetTick: FnMut() -> u32,
{
    let current_tick_ms = get_tick();
    clocks.current_tick_ms = current_tick_ms;
    current_tick_ms
}

/// Снимает один tick и сохраняет его как начало следующей profiling-стадии.
pub(crate) fn start_main_loop_profile_stage<GetTick>(
    clocks: &mut WorldMainLoopClockState,
    mut get_tick: GetTick,
) -> u32
where
    GetTick: FnMut() -> u32,
{
    let started_at_ms = get_tick();
    clocks.stage_started_at_ms = started_at_ms;
    started_at_ms
}

/// Инициализирует tail bits `0x10/0x20/0x40` с точными clock-call positions.
pub(crate) fn initialize_main_loop_tail_clocks<GetTick>(
    initialization: &mut WorldMainLoopInitializationState,
    clocks: &mut WorldMainLoopTailClockState,
    mut get_tick: GetTick,
) -> WorldMainLoopTailClockInitialization
where
    GetTick: FnMut() -> u32,
{
    let previous_mask = initialization.mask;
    let initial_current_tick_ms = if initialization.mask & 0x10 == 0 {
        initialization.mask |= 0x10;
        let tick = get_tick();
        clocks.current_tick_ms = tick;
        Some(tick)
    } else {
        None
    };
    let initial_pacing_deadline_ms = if initialization.mask & 0x20 == 0 {
        initialization.mask |= 0x20;
        clocks.pacing_deadline_ms = clocks.current_tick_ms;
        Some(clocks.pacing_deadline_ms)
    } else {
        None
    };
    let initial_minute_started_at_ms = if initialization.mask & 0x40 == 0 {
        initialization.mask |= 0x40;
        let tick = get_tick();
        clocks.minute_started_at_ms = tick;
        Some(tick)
    } else {
        None
    };
    WorldMainLoopTailClockInitialization {
        previous_mask,
        initialized_mask: initialization.mask,
        initial_current_tick_ms,
        initial_pacing_deadline_ms,
        initial_minute_started_at_ms,
    }
}

/// Внешний profiling-результат готового `CGame::ProcessMessage`.
#[derive(Debug)]
pub(crate) enum WorldProcessMessageStageReport {
    Blocked {
        started_at_ms: u32,
        error: WorldProcessMessageError,
    },
    Complete {
        started_at_ms: u32,
        outcome: WorldProcessMessageOutcome,
        finished_at_ms: u32,
        elapsed_ms: u32,
        accumulated_time_ms: u32,
        next_stage_started_at_ms: u32,
    },
}

/// Безопасная граница одного World message snapshot.
#[derive(Debug)]
pub(crate) enum WorldProcessMessageError {
    MissingNetworkServerOwner,
    ServerMessage(WorldServerMessageError),
}

impl fmt::Display for WorldProcessMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingNetworkServerOwner => formatter
                .write_str("World ProcessMessage не может прочитать обязательный server-owner"),
            Self::ServerMessage(error) => error.fmt(formatter),
        }
    }
}

impl Error for WorldProcessMessageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingNetworkServerOwner => None,
            Self::ServerMessage(error) => Some(error),
        }
    }
}

/// Ошибка одной достигнутой попытки World-to-Login reconnect.
#[derive(Debug)]
pub(crate) enum WorldLoginReconnectError {
    /// Setup-чтение не назначило поле, которое исходник затем читал.
    MissingSetupField(&'static str),
    /// `char[64]` старого `CClient::Connect` был бы переполнен.
    LoginAddressTooLongReactionUnknown { length: usize },
    /// ANSI hostname нельзя без доказательства преобразовать в Linux resolver.
    LoginAddressEncodingUnsupported,
    /// `inet_addr/gethostbyname` не дали пригодный IPv4 endpoint.
    LoginAddressResolution,
    /// Не создан и не bind-нут новый IPv4 socket.
    Bind(io::Error),
    /// Общий десятисекундный connect завершился неуспешно.
    Connect(ClientConnectError),
    /// Успешный client невозможно передать через отсутствующий server-owner.
    MissingNetworkServerOwner,
}

impl fmt::Display for WorldLoginReconnectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSetupField(field) => {
                write!(formatter, "World setup не назначил поле {field}")
            }
            Self::LoginAddressTooLongReactionUnknown { length } => write!(
                formatter,
                "LoginServer-адрес длиной {length} bytes выходит за старый char[64]"
            ),
            Self::LoginAddressEncodingUnsupported => formatter.write_str(
                "кодировка LoginServer-адреса не поддерживается безопасным Linux resolver",
            ),
            Self::LoginAddressResolution => {
                formatter.write_str("LoginServer-адрес не разрешён в IPv4")
            }
            Self::Bind(error) => write!(formatter, "не создан reconnect socket: {error}"),
            Self::Connect(error) => {
                write!(
                    formatter,
                    "WorldServer повторно не подключён к LoginServer: {error}"
                )
            }
            Self::MissingNetworkServerOwner => formatter.write_str(
                "подключённый LoginServer client некуда передать: World server-owner отсутствует",
            ),
        }
    }
}

impl Error for WorldLoginReconnectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Bind(error) => Some(error),
            Self::Connect(error) => Some(error),
            Self::MissingSetupField(_)
            | Self::LoginAddressTooLongReactionUnknown { .. }
            | Self::LoginAddressEncodingUnsupported
            | Self::LoginAddressResolution
            | Self::MissingNetworkServerOwner => None,
        }
    }
}

/// Оба исходных World setup-файла недоступны.
#[derive(Debug)]
pub(crate) struct WorldSetupOpenError {
    plain_path: PathBuf,
    plain: io::Error,
    encoded_path: PathBuf,
    encoded: io::Error,
}

impl fmt::Display for WorldSetupOpenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "не открыты {} ({}) и {} ({})",
            self.plain_path.display(),
            self.plain,
            self.encoded_path.display(),
            self.encoded
        )
    }
}

impl Error for WorldSetupOpenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.encoded)
    }
}

/// Owned-форма исходного `CGame::tagSetup`.
#[derive(Clone, Debug)]
pub(crate) struct WorldSetup {
    world_number: Option<u32>,
    name: Vec<u8>,
    login_ip: Vec<u8>,
    login_port: Option<u32>,
    listen_port: Option<u32>,
    sql_connection_type: Vec<u8>,
    sql_server_ip: Vec<u8>,
    sql_user_name: Vec<u8>,
    sql_password: Vec<u8>,
    database_name: Vec<u8>,
    check_net: Option<bool>,
    maximum_byte_count: Option<u32>,
    maximum_message_length: Option<u32>,
    ban_ip_time_ms: Option<u32>,
    check_message_content: Option<bool>,
    maximum_connections: Option<i32>,
    maximum_io_sends: Option<i32>,
    maximum_client_send_buffer: Option<i32>,
    refresh_info_time_ms: u32,
    save_info_time_ms: u32,
    release_login_player_time_ms: Option<u32>,
    use_log_system: bool,
    log_system_provider: Vec<u8>,
    log_system_server: Vec<u8>,
    log_system_database: Vec<u8>,
    log_system_user: Vec<u8>,
    log_system_password: Vec<u8>,
    cost_database_provider: Vec<u8>,
    cost_database_ip: Vec<u8>,
    cost_database_name: Vec<u8>,
    cost_database_user: Vec<u8>,
    cost_database_password: Vec<u8>,
    load_largess_time_ms: Option<u32>,
    login_cost_database_provider: Vec<u8>,
    login_cost_database_ip: Vec<u8>,
    login_cost_database_name: Vec<u8>,
    login_cost_database_user: Vec<u8>,
    login_cost_database_password: Vec<u8>,
    player_load_thread_count: Option<u32>,
    language_package: Vec<u8>,
    use_old_save_largess_way: bool,
}

impl Default for WorldSetup {
    /// Воспроизводит только точный `tagSetup::tagSetup`, до записей `CGame`.
    fn default() -> Self {
        Self {
            world_number: None,
            name: Vec::new(),
            login_ip: Vec::new(),
            login_port: None,
            listen_port: None,
            sql_connection_type: Vec::new(),
            sql_server_ip: Vec::new(),
            sql_user_name: Vec::new(),
            sql_password: Vec::new(),
            database_name: Vec::new(),
            check_net: None,
            maximum_byte_count: None,
            maximum_message_length: None,
            ban_ip_time_ms: None,
            check_message_content: None,
            maximum_connections: None,
            maximum_io_sends: None,
            maximum_client_send_buffer: None,
            refresh_info_time_ms: 1_000,
            save_info_time_ms: 60_000,
            release_login_player_time_ms: None,
            use_log_system: false,
            log_system_provider: Vec::new(),
            log_system_server: Vec::new(),
            log_system_database: Vec::new(),
            log_system_user: Vec::new(),
            log_system_password: Vec::new(),
            cost_database_provider: Vec::new(),
            cost_database_ip: Vec::new(),
            cost_database_name: Vec::new(),
            cost_database_user: Vec::new(),
            cost_database_password: Vec::new(),
            load_largess_time_ms: None,
            login_cost_database_provider: Vec::new(),
            login_cost_database_ip: Vec::new(),
            login_cost_database_name: Vec::new(),
            login_cost_database_user: Vec::new(),
            login_cost_database_password: Vec::new(),
            player_load_thread_count: None,
            language_package: Vec::new(),
            use_old_save_largess_way: true,
        }
    }
}

impl WorldSetup {
    #[allow(
        clippy::field_reassign_with_default,
        reason = "две стадии буквально сохраняют tagSetup::tagSetup и последующие записи CGame::CGame"
    )]
    fn for_game() -> Self {
        let mut setup = Self::default();
        setup.name = b"WorldServer".to_vec();
        setup.login_ip = b"127.0.0.1".to_vec();
        setup.login_port = Some(2_345);
        setup.listen_port = Some(8_100);
        setup
    }

    fn network_config_after_host(
        &self,
    ) -> Result<WorldNetworkConfig, WorldNetworkInitializationError> {
        Ok(WorldNetworkConfig {
            // Порядок чтения повторяет локальные значения RVA 0x00001860.
            ban_ip_time_ms: self.ban_ip_time_ms.ok_or(
                WorldNetworkInitializationError::MissingSetupField("dwBanIPTime"),
            )?,
            maximum_client_send_buffer: self.maximum_client_send_buffer.ok_or(
                WorldNetworkInitializationError::MissingSetupField("lMaxClientSendBuf"),
            )?,
            maximum_message_length: self.maximum_message_length.ok_or(
                WorldNetworkInitializationError::MissingSetupField("dwMaxMsgLen"),
            )?,
            maximum_byte_count: self.maximum_byte_count.ok_or(
                WorldNetworkInitializationError::MissingSetupField("dwMaxByteNum"),
            )?,
            check_message_content: self.check_message_content.ok_or(
                WorldNetworkInitializationError::MissingSetupField("bCheckMsgCon"),
            )?,
            maximum_connections: self.maximum_connections.ok_or(
                WorldNetworkInitializationError::MissingSetupField("lMaxConnectNum"),
            )?,
            maximum_io_sends: self.maximum_io_sends.ok_or(
                WorldNetworkInitializationError::MissingSetupField("lMaxIOSendNum"),
            )?,
            check_net: self
                .check_net
                .ok_or(WorldNetworkInitializationError::MissingSetupField(
                    "bCheckNet",
                ))?,
        })
    }

    fn parse_plain(&mut self, bytes: &[u8]) -> (usize, Option<usize>) {
        let mut tokens = SetupTokens::new(bytes);

        macro_rules! read_value {
            ($field:ident, $parser:expr) => {{
                let Some(raw) = tokens.next_value() else {
                    return tokens.outcome();
                };
                let Some(value) = $parser(raw) else {
                    // BLOCKED_MISSING_FACT: для лексически неверного numeric/bool
                    // token не доказана мутация destination старым MSVC iostream.
                    // Найденный setup содержит только корректные такие значения.
                    return tokens.outcome();
                };
                self.$field = value;
                tokens.parsed();
            }};
        }
        macro_rules! read_number {
            ($field:ident, $type:ty) => {
                read_value!($field, |raw| parse_ascii::<$type>(raw).map(Some));
            };
        }
        macro_rules! read_number_with_default {
            ($field:ident, $type:ty) => {
                read_value!($field, |raw| parse_ascii::<$type>(raw));
            };
        }
        macro_rules! read_bool {
            ($field:ident) => {
                read_value!($field, parse_legacy_bool);
            };
        }
        macro_rules! read_optional_bool {
            ($field:ident) => {
                read_value!($field, |raw| parse_legacy_bool(raw).map(Some));
            };
        }
        macro_rules! read_bytes {
            ($field:ident) => {
                read_value!($field, |raw: &[u8]| Some(raw.to_vec()));
            };
        }

        read_number!(world_number, u32);
        read_bytes!(name);
        read_bytes!(login_ip);
        read_number!(login_port, u32);
        read_number!(listen_port, u32);
        read_bytes!(sql_connection_type);
        read_bytes!(sql_server_ip);
        read_bytes!(sql_user_name);
        read_bytes!(sql_password);
        read_bytes!(database_name);
        read_optional_bool!(check_net);
        read_number!(maximum_byte_count, u32);
        read_number!(maximum_message_length, u32);
        read_number!(ban_ip_time_ms, u32);
        read_optional_bool!(check_message_content);
        read_number!(maximum_connections, i32);
        read_number!(maximum_io_sends, i32);
        read_number!(maximum_client_send_buffer, i32);
        read_number_with_default!(refresh_info_time_ms, u32);
        read_number_with_default!(save_info_time_ms, u32);
        read_number!(release_login_player_time_ms, u32);
        read_bool!(use_log_system);
        read_bytes!(log_system_provider);
        read_bytes!(log_system_server);
        read_bytes!(log_system_database);
        read_bytes!(log_system_user);
        read_bytes!(log_system_password);
        read_bytes!(cost_database_provider);
        read_bytes!(cost_database_ip);
        read_bytes!(cost_database_name);
        read_bytes!(cost_database_user);
        read_bytes!(cost_database_password);
        read_number!(load_largess_time_ms, u32);
        read_bytes!(login_cost_database_provider);
        read_bytes!(login_cost_database_ip);
        read_bytes!(login_cost_database_name);
        read_bytes!(login_cost_database_user);
        read_bytes!(login_cost_database_password);
        read_number!(player_load_thread_count, u32);
        read_bytes!(language_package);
        read_bool!(use_old_save_largess_way);

        tokens.outcome()
    }

    fn parse_encoded(&mut self, bytes: &[u8]) -> (usize, Option<usize>) {
        let mut tokens = SetupTokens::new(bytes);

        macro_rules! read_value {
            ($field:ident, $parser:expr) => {{
                let Some(raw) = tokens.next_value() else {
                    return tokens.outcome();
                };
                let Some(value) = $parser(raw) else {
                    // BLOCKED_MISSING_FACT: malformed numeric/bool token не
                    // встречается в найденном oracle; MSVC destination не угадываем.
                    return tokens.outcome();
                };
                self.$field = value;
                tokens.parsed();
            }};
        }
        macro_rules! read_number {
            ($field:ident, $type:ty) => {
                read_value!($field, |raw| parse_ascii::<$type>(raw).map(Some));
            };
        }
        macro_rules! read_number_with_default {
            ($field:ident, $type:ty) => {
                read_value!($field, |raw| parse_ascii::<$type>(raw));
            };
        }
        macro_rules! read_bool {
            ($field:ident) => {
                read_value!($field, parse_legacy_bool);
            };
        }
        macro_rules! read_optional_bool {
            ($field:ident) => {
                read_value!($field, |raw| parse_legacy_bool(raw).map(Some));
            };
        }
        macro_rules! read_bytes {
            ($field:ident) => {
                read_value!($field, |raw: &[u8]| Some(raw.to_vec()));
            };
        }

        read_number!(world_number, u32);
        read_bytes!(name);
        read_bytes!(login_ip);
        read_number!(login_port, u32);
        read_number!(listen_port, u32);
        read_bytes!(sql_connection_type);
        read_bytes!(sql_server_ip);
        read_bytes!(sql_user_name);
        read_bytes!(sql_password);
        read_bytes!(database_name);
        read_optional_bool!(check_net);
        read_number!(maximum_byte_count, u32);
        read_number!(maximum_message_length, u32);
        read_number!(ban_ip_time_ms, u32);
        read_optional_bool!(check_message_content);
        read_number!(maximum_connections, i32);
        read_number!(maximum_io_sends, i32);
        read_number!(maximum_client_send_buffer, i32);
        read_number_with_default!(refresh_info_time_ms, u32);
        read_number_with_default!(save_info_time_ms, u32);
        read_number!(release_login_player_time_ms, u32);
        read_bool!(use_log_system);

        // Точный старый DAT-порядок RVA 0x0000F890: provider не назначается.
        read_bytes!(log_system_server);
        read_bytes!(log_system_database);
        read_bytes!(log_system_user);
        read_bytes!(log_system_password);
        read_bytes!(cost_database_provider);
        read_bytes!(cost_database_ip);
        read_bytes!(cost_database_name);
        read_bytes!(cost_database_user);
        read_bytes!(cost_database_password);
        read_bytes!(name);
        read_number!(load_largess_time_ms, u32);
        read_bytes!(login_cost_database_provider);
        read_bytes!(login_cost_database_ip);
        read_bytes!(login_cost_database_name);
        read_bytes!(login_cost_database_user);
        read_bytes!(login_cost_database_password);
        read_number!(player_load_thread_count, u32);
        read_bytes!(language_package);
        read_bool!(use_old_save_largess_way);

        tokens.outcome()
    }
}

fn parse_ascii<T: std::str::FromStr>(raw: &[u8]) -> Option<T> {
    std::str::from_utf8(raw).ok()?.parse().ok()
}

fn parse_legacy_bool(raw: &[u8]) -> Option<bool> {
    match raw {
        b"0" => Some(false),
        b"1" => Some(true),
        _ => None,
    }
}

struct SetupTokens<'a> {
    tokens: Vec<&'a [u8]>,
    next: usize,
    attempted_pairs: usize,
    parsed_pairs: usize,
}

impl<'a> SetupTokens<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            tokens: bytes
                .split(|byte| byte.is_ascii_whitespace())
                .filter(|token| !token.is_empty())
                .collect(),
            next: 0,
            attempted_pairs: 0,
            parsed_pairs: 0,
        }
    }

    fn next_value(&mut self) -> Option<&'a [u8]> {
        self.attempted_pairs += 1;
        let _label = self.tokens.get(self.next)?;
        let value = self.tokens.get(self.next + 1).copied()?;
        self.next += 2;
        Some(value)
    }

    fn parsed(&mut self) {
        self.parsed_pairs += 1;
    }

    const fn outcome(&self) -> (usize, Option<usize>) {
        let stopped_at_pair = if self.parsed_pairs < self.attempted_pairs {
            Some(self.attempted_pairs)
        } else {
            None
        };
        (self.parsed_pairs, stopped_at_pair)
    }
}

struct WorldServerSetupTokens<'a> {
    tokens: Vec<&'a [u8]>,
    next: usize,
    failed: bool,
}

impl<'a> WorldServerSetupTokens<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            tokens: bytes
                .split(|byte| byte.is_ascii_whitespace())
                .filter(|token| !token.is_empty())
                .collect(),
            next: 0,
            failed: false,
        }
    }

    fn seek_to(&mut self, expected: &[u8]) -> bool {
        if self.failed {
            return false;
        }
        let remaining = &self.tokens[self.next..];
        let mut iterator = remaining.iter().copied();
        let found = read_to(&mut iterator, expected);
        let consumed = remaining.len() - iterator.len();
        // Общий ReadTo возвращал false на успешно прочитанном `<end>`, не
        // переводя сам formatted stream в fail-state.
        let stopped_at_end =
            !found && consumed != 0 && self.tokens[self.next + consumed - 1] == b"<end>";
        self.next += consumed;
        if !found && !stopped_at_end {
            self.failed = true;
        }
        found
    }

    fn next_bytes(&mut self) -> Option<&'a [u8]> {
        if self.failed {
            return None;
        }
        let Some(token) = self.tokens.get(self.next).copied() else {
            self.failed = true;
            return None;
        };
        self.next += 1;
        Some(token)
    }

    fn next_ascii<T: std::str::FromStr>(&mut self) -> Option<T> {
        let raw = self.next_bytes()?;
        let parsed = parse_ascii(raw);
        if parsed.is_none() {
            self.failed = true;
        }
        parsed
    }

    const fn failed(&self) -> bool {
        self.failed
    }
}

fn resolve_world_runtime_file(
    runtime_directory: &Path,
    requested_name: &str,
) -> Result<PathBuf, io::Error> {
    let requested_path = runtime_directory.join(requested_name);
    match fs::metadata(&requested_path) {
        Ok(metadata) if metadata.is_file() => return Ok(requested_path),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    for entry in fs::read_dir(runtime_directory)? {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if name.eq_ignore_ascii_case(requested_name) && entry.file_type()?.is_file() {
            return Ok(entry.path());
        }
    }
    Ok(requested_path)
}

/// Семантическая замена старого 36-байтового `tagPingGameServerInfo`.
///
/// Ветка `0x5FA0A` подтверждает `std::string strIP` и два signed `long`:
/// map ID из metadata сообщения и число игроков из payload. Rust-layout не
/// выдаётся за Windows ABI; owned bytes и `Vec` заменяют только STL-владение.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldPingGameServerInfo {
    pub(crate) ip: Vec<u8>,
    pub(crate) map_id: i32,
    pub(crate) player_count: i32,
}

/// Итог независимых insert-ов `CGame::AddItemToBaiTanList`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldBaiTanRegistration {
    pub(crate) player_id: i32,
    pub(crate) ip: u32,
    pub(crate) player_ip_inserted: bool,
    pub(crate) ip_refcount: i32,
    pub(crate) game_server_index: i32,
    pub(crate) player_route_inserted: bool,
}

/// Итог `CGame::DelItemFromBaiTanList` и связанного IP refcount.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldBaiTanRemoval {
    pub(crate) player_id: i32,
    pub(crate) mapped_ip: Option<u32>,
    pub(crate) remaining_ip_refcount: Option<i32>,
    pub(crate) player_ip_removed: bool,
    pub(crate) player_route_removed: bool,
}

/// Один элемент ordered `DoneBaiTanList` batch.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldBaiTanCompletion {
    pub(crate) requested_ip: u32,
    pub(crate) player_id: i32,
    pub(crate) registration: WorldBaiTanRegistration,
    pub(crate) route_game_server_index: i32,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

/// Полный результат `DoneBaiTanList` до и после очистки request map.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldDoneBaiTanListReport {
    pub(crate) completions: Vec<WorldBaiTanCompletion>,
    pub(crate) cleared_requests: usize,
}

/// Материализованный virtual owner поставочных region type-ов.
pub(crate) enum WorldRegionOwner {
    Base(Box<CWorldRegion>),
    Village(Box<CWorldVillageRegion>),
    City(Box<CWorldCityRegion>),
    Country(Box<WorldCountryWarRegion>),
}

impl WorldRegionOwner {
    pub(crate) fn base(&self) -> &CWorldRegion {
        match self {
            Self::Base(region) => region,
            Self::Village(region) => region.war().base(),
            Self::City(region) => region.war().base(),
            Self::Country(region) => region.base(),
        }
    }

    pub(crate) fn base_mut(&mut self) -> &mut CWorldRegion {
        match self {
            Self::Base(region) => region,
            Self::Village(region) => region.war_mut().base_mut(),
            Self::City(region) => region.war_mut().base_mut(),
            Self::Country(region) => region.base_mut(),
        }
    }

    /// Выполняет exact virtual AI всех поставочных World region owner-ов.
    /// Их slot `+0x40` указывает на общий однокомандный `ret` `0x00401000`.
    pub(crate) const fn ai(&mut self) {}

    fn add_full_initial_snapshot(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), WorldRegionOwnerSerializationBlock> {
        match self {
            Self::Base(region) => {
                let _ = region
                    .add_to_byte_array(destination, true)
                    .map_err(WorldRegionOwnerSerializationBlock::Base)?;
            }
            Self::Village(region) => {
                let _ = region
                    .add_to_byte_array(destination, true)
                    .map_err(WorldRegionOwnerSerializationBlock::Village)?;
            }
            Self::City(region) => {
                let _ = region
                    .add_to_byte_array(destination, true)
                    .map_err(WorldRegionOwnerSerializationBlock::City)?;
            }
            Self::Country(region) => {
                let _ = region
                    .add_to_byte_array(destination, true)
                    .map_err(WorldRegionOwnerSerializationBlock::Country)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldRegionOwnerLoadBlock {
    Base(WorldRegionLoadError),
    Village(WorldRegionLoadError),
    City(WorldCityRegionLoadError),
    Country(WorldCountryWarRegionLoadError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldRegionOwnerSerializationBlock {
    Base(WorldRegionSerializationBlock),
    Village(WorldWarRegionSerializationBlock),
    City(WorldCityRegionSerializationBlock),
    Country(WorldCountryWarRegionSerializationBlock),
}

/// Выбранный исходным initial-config virtual wire одного региона.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldInitialRegionSnapshotKind {
    Assigned { region_type: i32 },
    Proxy,
}

/// Один уже сериализованный элемент ordered `s_mapRegionList`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldInitialRegionSnapshot {
    pub(crate) map_key: i32,
    pub(crate) region_id: i32,
    pub(crate) kind: WorldInitialRegionSnapshotKind,
    pub(crate) payload: Vec<u8>,
}

/// Точная safe-граница initial-config region traversal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldInitialRegionSnapshotSource {
    MissingRegionOwner,
    UninitializedRegionType,
    Full(WorldRegionOwnerSerializationBlock),
    Proxy(RegionSerializationBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldInitialRegionSnapshotBlock {
    pub(crate) map_key: i32,
    pub(crate) source: WorldInitialRegionSnapshotSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldRegionListBlock {
    pub(crate) region_id: i32,
    pub(crate) source: WorldRegionOwnerLoadBlock,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldReloadRegionSnapshotBlock {
    pub(crate) region_id: i32,
    pub(crate) source: WorldRegionOwnerSerializationBlock,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldReloadBlock {
    RegionList(WorldRegionListBlock),
    RegionSnapshot(WorldReloadRegionSnapshotBlock),
    ThingSetupCodec(ThingSetupCodecError),
    EmotionFormat(EmotionFormatError),
    EmotionSerialization(EmotionSerializeError),
    PlayerListFormat(PlayerListFormatError),
    PlayerListSerialization(PlayerListSerializeError),
    GoodsDestroyFormat(GoodsDestroyFormatError),
    GoodsDestroySerialization(GoodsDestroySerializeError),
    NewSkillMonsterSerialization(NewSkillMonsterSerializeError),
    BattleFairyExpSerialization(BattleFairyExpSerializeError),
    BattleFairyCombineSerialization(BattleFairyComposeWireError),
    SynthesisSerialization(SynthesisSerializeError),
    EquipmentComposeSerialization(EquipmentComposeSerializeError),
    CiQingSerialization(CiQingSerializationBlock),
    TaoZhuangSerialization(TaoZhuangSerializationBlock),
    HitLevelFormat(HitLevelFormatError),
    HitLevelSerialization(HitLevelSerializeError),
    TradeListFormat(TradeListFormatError),
    TradeListSerialization(TradeListSerializeError),
    IncrementShopSerialization(IncrementShopSerializeError),
    PrisonFormat(PrisonConfFormatError),
    PrisonSerialization(PrisonConfSerializeError),
    ContributeFormat(ContributeSetupFormatError),
    ContributeSerialization(ContributeSetupSerializeError),
    CountryWarOwnerRequired,
    CountryWar(CountryWarReloadBlock),
}

pub(crate) type WorldReloadResult = Result<i32, WorldReloadBlock>;

enum WorldRegionMaterialization {
    Direct {
        owner: WorldRegionOwner,
        counts: WorldRegionLoadedCounts,
        loaded: bool,
    },
    MissingSubtype,
}

/// Достигнутая часть исходного 12-байтового `CGame::tagRegion`.
pub(crate) struct WorldRegionAssignment {
    region: Option<WorldRegionOwner>,
    game_server_index: u32,
    /// PDB `REGION_TYPE +0x8`; до reached loader-а значение неизвестно.
    region_type: Option<i32>,
}

/// Достигнутая AI-проекция исходного `CGame::tagSysBroadcast`.
///
/// Поля идут по смыслу struct-layout `+0x04..+0x40`; `_login_type` AI не читает,
/// а Rust-layout не выдаётся за старый 68-байтовый Windows ABI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldSystemBroadcast {
    import_level: i32,
    region_id: i32,
    min_time_seconds: u32,
    max_time_seconds: u32,
    odds: u32,
    text_color: u32,
    back_color: u32,
    message: Vec<u8>,
    interval_seconds: u32,
    last_notify_time_seconds: u32,
}

const INITIAL_GOODS_LINK_PLACEHOLDERS: usize = 500;
const LEGACY_GOODS_LINK_MAX_SIZE: usize = 0x0CCC_CCCC;
static NEXT_GOODS_LINK_INDEX: AtomicU32 = AtomicU32::new(1);

/// Владеющая Rust-форма точного 20-байтового `CGame::tagGoodsLink`.
///
/// `Box<CGoods>` заменяет сырой owning pointer только для `bChange != 0`;
/// unchanged-запись хранит исходные `dwType/lNum`. Старый padding не
/// материализуется, потому что ни lookup, ни wire его не наблюдают.
pub(crate) enum WorldGoodsLinkPayload {
    Changed(Box<CGoods>),
    Original { goods_type: u32, amount: u8 },
}

pub(crate) struct WorldGoodsLink {
    index: u32,
    payload: WorldGoodsLinkPayload,
}

/// Результат exact `CGame::GetOptMoneyJin` для одной аукционной цены.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldAuctionSellerMoney {
    pub(crate) fee: i32,
    pub(crate) seller_money_after_fee: i32,
}

/// Точно отбрасывает дробную часть произведения signed `long` на один `f32`.
///
/// EXE оставляет произведение в 80-битном x87 до `_ftol2`. Разложение IEEE-754
/// в целую мантиссу и степень сохраняет этот результат без промежуточного
/// округления Rust `f32`; только нештатный overflow/NaN получает определённое
/// насыщение вместо неопределённого C++ float-to-long cast.
fn truncate_scaled_legacy_money(amount: i32, factor: f32) -> i32 {
    let bits = factor.to_bits();
    let exponent = (bits >> 23) & 0xFF;
    let fraction = bits & 0x007F_FFFF;
    if exponent == 0xFF {
        if fraction != 0 || amount == 0 {
            return 0;
        }
        return if (amount < 0) ^ (bits >> 31 != 0) {
            i32::MIN
        } else {
            i32::MAX
        };
    }

    let (mantissa, binary_exponent) = if exponent == 0 {
        (u128::from(fraction), -149)
    } else {
        (
            u128::from((1 << 23) | fraction),
            exponent as i32 - 127 - 23,
        )
    };
    let magnitude = u128::from(amount.unsigned_abs()) * mantissa;
    let magnitude = if binary_exponent >= 0 {
        let shift = binary_exponent as u32;
        if shift >= u128::BITS || magnitude > (u128::MAX >> shift) {
            u128::MAX
        } else {
            magnitude << shift
        }
    } else {
        magnitude
            .checked_shr(binary_exponent.unsigned_abs())
            .unwrap_or(0)
    };
    let negative = (amount < 0) ^ (bits >> 31 != 0);
    if negative {
        if magnitude >= 0x8000_0000 {
            i32::MIN
        } else {
            -(magnitude as i32)
        }
    } else {
        magnitude.min(i32::MAX as u128) as i32
    }
}

fn truncate_legacy_money(value: f64) -> i32 {
    if value.is_nan() {
        0
    } else if value >= f64::from(i32::MAX) {
        i32::MAX
    } else if value <= f64::from(i32::MIN) {
        i32::MIN
    } else {
        value.trunc() as i32
    }
}

impl WorldGoodsLink {
    fn placeholder() -> Self {
        Self {
            index: 0,
            payload: WorldGoodsLinkPayload::Original {
                goods_type: 0,
                amount: 0,
            },
        }
    }

    pub(crate) fn changed(goods: Box<CGoods>) -> Self {
        Self {
            index: goods.get_id() as u32,
            payload: WorldGoodsLinkPayload::Changed(goods),
        }
    }

    pub(crate) const fn original(goods_type: u32, amount: u8) -> Self {
        Self {
            index: 0,
            payload: WorldGoodsLinkPayload::Original { goods_type, amount },
        }
    }

    pub(crate) const fn payload(&self) -> &WorldGoodsLinkPayload {
        &self.payload
    }
}

/// Точная send-ветвь выбранного системного broadcast-а.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldSystemBroadcastTarget {
    All {
        delivery: Result<i32, SendMessageError>,
    },
    Region {
        region_id: i32,
        game_server_index: Option<u32>,
        delivery: Option<Result<i32, SendMessageError>>,
    },
}

/// Результат одной позиции ordered `m_listBroadcast`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldSystemBroadcastDisposition {
    Waiting {
        elapsed_seconds: u32,
        interval_seconds: u32,
    },
    OddsMissed {
        roll: i32,
        odds: u32,
    },
    Broadcast {
        roll: i32,
        target: WorldSystemBroadcastTarget,
        assigned_last_notify_time_seconds: u32,
        assigned_interval_seconds: u32,
    },
}

/// Наблюдаемый результат полного `CGame::AI` RVA `0x000148A0`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldGameAiReport {
    pub(crate) region_ids_run: Vec<i32>,
    pub(crate) broadcast_tick_ms: u32,
    pub(crate) broadcasts: Vec<WorldSystemBroadcastDisposition>,
    pub(crate) legacy_result: i32,
}

/// Три наблюдаемых результата цепочки `GetRegion -> tagRegion::pRegion`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldRegionNameLookup<'a> {
    RegionNotFound,
    NullRegionPointer,
    Name(&'a [u8]),
}

/// Найденный case-sensitive `GetRegion(name)` route snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldNamedRegionMatch {
    pub(crate) region_id: i32,
    pub(crate) game_server_index: u32,
    pub(crate) game_server_entry_found: bool,
    pub(crate) game_server_connected: bool,
}

/// Безопасный отчёт исходного ordered name lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldNamedRegionLookup {
    pub(crate) skipped_null_owners: usize,
    pub(crate) matched: Option<WorldNamedRegionMatch>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldRegionIdRoute {
    pub(crate) map_key: i32,
    pub(crate) game_server_id: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldRegionIdRouteScan {
    pub(crate) skipped_null_owners: usize,
    pub(crate) matching_region_keys: usize,
    pub(crate) routes: Vec<WorldRegionIdRoute>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldRegionParamUpdateOutcome {
    RegionNotFound,
    NullRegionPointer,
    Applied,
}

/// Наблюдаемый снимок успешного `CGame::RefreshOwnedCityOrg`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldOwnedCityRefreshReport {
    pub(crate) region_id: i32,
    pub(crate) faction_id: i32,
    pub(crate) union_id: i32,
    pub(crate) country_id: u8,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

/// Обе nullable-ступени region lookup и успешная owner-мутация.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldOwnedCityRefreshOutcome {
    RegionNotFound,
    NullRegionPointer,
    Refreshed(WorldOwnedCityRefreshReport),
}

/// Результат virtual selective decoder-а `CWorldRegion` из server `0x5FA07`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldRegionParamDecodeOutcome {
    RegionNotFound,
    NullRegionPointer,
    Decoded(Result<bool, WorldRegionParamDecodeError>),
}

/// Минимальная достигнутая часть исходного `CGame::tagGameServer`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldGameServerEntry {
    pub(crate) connected: bool,
    pub(crate) index: u32,
    pub(crate) ip: Vec<u8>,
    pub(crate) port: Option<u32>,
    pub(crate) received_player_data: Option<i32>,
}

/// Изменение `tagGameServer::lReceivedPlayerData` при subtype `0/1`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldReceivedPlayerDataUpdate {
    GameServerNotFound,
    Uninitialized,
    Updated {
        previous: Option<i32>,
        current: i32,
    },
}

/// Чтение `lReceivedPlayerData` для итогового subtype `2`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldReceivedPlayerDataRead {
    GameServerNotFound { legacy_value: i32 },
    Uninitialized,
    Value(i32),
}

/// Точный переход состояния `tagGameServer::bConnected = true` из `0x5FA01`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldGameServerConnectionState {
    pub(crate) index: u32,
    pub(crate) previous_connected: bool,
}

/// Безопасное содержимое точного 16-байтового `CGame::tagGlobeVariable`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorldGlobeVariables {
    pub(crate) world_cap_team_1: i32,
    pub(crate) world_cap_team_2: i32,
    pub(crate) world_cap_team_3: i32,
    pub(crate) world_cap_team_4: i32,
}

impl WorldGlobeVariables {
    fn values(self) -> [i32; 4] {
        [
            self.world_cap_team_1,
            self.world_cap_team_2,
            self.world_cap_team_3,
            self.world_cap_team_4,
        ]
    }
}

/// Результат точной отправки `CGame::SendGlobeVariableToGS`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldGlobeVariablesDelivery {
    pub(crate) socket_id: i32,
    pub(crate) variables: WorldGlobeVariables,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

/// Точная достигнутая семантика полей исходного `CGame::tagLoginPlayer`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WorldLoginPlayerEntry {
    player_id: u32,
    login_time_ms: u32,
}

/// Снимок первого login-list игрока, чей mapped account совпал с запросом.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldLoginAccountPlayer {
    pub(crate) team_id: i32,
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
}

/// Неизменяемые route-поля первого login-player lookup по numeric ID.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldLoginPlayerRouteSnapshot {
    pub(crate) map_key: u32,
    pub(crate) owner_id: i32,
    pub(crate) region_id: i32,
}

/// Первый online-list account, для которого достигнут назначенный GameServer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldOnlineAccountPlayerRoute {
    pub(crate) team_id: i32,
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) game_server_index: u32,
}

/// Владеющая копия точного 8-байтного `CGame::tagDeletionPlayer`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DeletionPlayerSnapshot {
    pub(crate) player_id: u32,
    pub(crate) deletion_time: i32,
}

/// Локальные safe-границы полного `CGame::GenerateDBData`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldGenerateDbDataBlock {
    UninitializedLeaveWordId,
    UninitializedPlayerId,
    PlayerCodec(PlayerCodecError),
    Organizing(OrganizingSaveDataBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldLeaveWordIdBlock;

impl From<PlayerCodecError> for WorldGenerateDbDataBlock {
    fn from(error: PlayerCodecError) -> Self {
        Self::PlayerCodec(error)
    }
}

impl From<OrganizingSaveDataBlock> for WorldGenerateDbDataBlock {
    fn from(error: OrganizingSaveDataBlock) -> Self {
        Self::Organizing(error)
    }
}

/// Отчёт полного прохода `CGame::GenerateDBData`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldGenerateDbDataReport {
    pub(crate) organizing: OrganizingSaveDataReport,
}

/// Материализованная достигнутая часть исходного `CGame::tagDBData`.
///
/// Неинициализированные конструктором scalar ID представлены `Option`; Rust-
/// layout не является копией 32-битного MSVC ABI. Exact `DoSaveData` не имеет
/// Village/City War snapshot-полей или save-фаз, поэтому они здесь не
/// резервируются по одному лишь имени пустых DB-адаптеров.
struct WorldDbData {
    player_id: Option<u32>,
    leave_word_id: Option<i32>,
    creation_players: VecDeque<Box<CPlayer>>,
    restore_players: VecDeque<u32>,
    deletion_players: VecDeque<DeletionPlayerSnapshot>,
    players: BTreeMap<u32, Box<CPlayer>>,
    save_factions: VecDeque<Box<CFaction>>,
    save_unions: VecDeque<Box<CUnion>>,
    delete_factions: VecDeque<i32>,
    delete_unions: VecDeque<i32>,
    enemy_factions: VecDeque<Option<EnemyFactionSaveSnapshot>>,
    regions: VecDeque<Option<RegionSaveSnapshot>>,
    countries: VecDeque<Option<CountrySaveSnapshot>>,
}

impl WorldDbData {
    const fn new() -> Self {
        Self {
            player_id: None,
            leave_word_id: None,
            creation_players: VecDeque::new(),
            restore_players: VecDeque::new(),
            deletion_players: VecDeque::new(),
            players: BTreeMap::new(),
            save_factions: VecDeque::new(),
            save_unions: VecDeque::new(),
            delete_factions: VecDeque::new(),
            delete_unions: VecDeque::new(),
            enemy_factions: VecDeque::new(),
            regions: VecDeque::new(),
            countries: VecDeque::new(),
        }
    }
}

/// Эксклюзивный доступ `DoSaveData` к уже сформированному `tagDBData`.
///
/// `CGame::Run` и `SaveThreadFunc` сериализовали generation/save одним
/// `g_CriticalSectionSaveThread`. Заимствование `&mut CGame` выражает эту
/// внешнюю границу, а внутренний mutex больше не удерживается во время DB I/O.
/// Создание самой save-thread остаётся у ещё сырого caller-а.
pub(crate) struct WorldDbDataSaveSession<'game> {
    data: &'game mut WorldDbData,
}

impl WorldDbDataSaveSession<'_> {
    /// Возвращает два scalar ID одного frozen snapshot либо его точную дыру.
    pub(crate) const fn setup_ids(&self) -> Option<(u32, i32)> {
        match (self.data.player_id, self.data.leave_word_id) {
            (Some(player_id), Some(leave_word_id)) => Some((player_id, leave_word_id)),
            _ => None,
        }
    }

    pub(crate) fn creation_players_len(&self) -> usize {
        self.data.creation_players.len()
    }

    pub(crate) fn creation_player(&self, index: usize) -> Option<&CPlayer> {
        self.data.creation_players.get(index).map(Box::as_ref)
    }

    /// Уничтожает successful creation-owner и удаляет только его list-node.
    pub(crate) fn remove_creation_player(&mut self, index: usize) {
        drop(self.data.creation_players.remove(index));
    }

    pub(crate) fn restore_players_len(&self) -> usize {
        self.data.restore_players.len()
    }

    pub(crate) fn restore_player_id(&self, index: usize) -> Option<u32> {
        self.data.restore_players.get(index).copied()
    }

    /// Удаляет только successful restore ID-node.
    pub(crate) fn remove_restore_player(&mut self, index: usize) {
        let _ = self.data.restore_players.remove(index);
    }

    pub(crate) fn deletion_players_len(&self) -> usize {
        self.data.deletion_players.len()
    }

    pub(crate) fn deletion_player(&self, index: usize) -> Option<DeletionPlayerSnapshot> {
        self.data.deletion_players.get(index).copied()
    }

    /// Удаляет только successful deletion-record node.
    pub(crate) fn remove_deletion_player(&mut self, index: usize) {
        let _ = self.data.deletion_players.remove(index);
    }

    pub(crate) fn players(&self) -> &BTreeMap<u32, Box<CPlayer>> {
        &self.data.players
    }

    /// Уничтожает successful player-owner и стирает его единственный map-entry.
    pub(crate) fn remove_player(&mut self, player_id: u32) {
        drop(self.data.players.remove(&player_id));
    }

    pub(crate) fn delete_union_ids(&mut self) -> &[i32] {
        self.data.delete_unions.make_contiguous()
    }

    pub(crate) fn clear_delete_unions(&mut self) {
        self.data.delete_unions.clear();
    }

    pub(crate) fn delete_faction_ids(&mut self) -> &[i32] {
        self.data.delete_factions.make_contiguous()
    }

    pub(crate) fn clear_delete_factions(&mut self) {
        self.data.delete_factions.clear();
    }

    pub(crate) fn save_factions_len(&self) -> usize {
        self.data.save_factions.len()
    }

    pub(crate) fn first_saved_faction_mut(&mut self) -> Option<&mut CFaction> {
        self.data.save_factions.front_mut().map(Box::as_mut)
    }

    /// Удаляет текущий faction node и уничтожает его non-null save-копию.
    pub(crate) fn remove_first_saved_faction(&mut self) {
        drop(self.data.save_factions.pop_front());
    }

    pub(crate) fn clear_saved_faction_nodes(&mut self) {
        self.data.save_factions.clear();
    }

    pub(crate) fn save_unions_len(&self) -> usize {
        self.data.save_unions.len()
    }

    pub(crate) fn first_saved_union(&self) -> Option<&CUnion> {
        self.data.save_unions.front().map(Box::as_ref)
    }

    /// Удаляет текущий union node и уничтожает его non-null save-копию.
    pub(crate) fn remove_first_saved_union(&mut self) {
        drop(self.data.save_unions.pop_front());
    }

    pub(crate) fn clear_saved_union_nodes(&mut self) {
        self.data.save_unions.clear();
    }

    pub(crate) fn regions_len(&self) -> usize {
        self.data.regions.len()
    }

    pub(crate) fn region(&self, index: usize) -> Option<Option<RegionSaveSnapshot>> {
        self.data.regions.get(index).copied()
    }

    /// Уничтожает non-null region value, сохраняя его list-node до общего clear.
    pub(crate) fn destroy_saved_region(&mut self, index: usize) {
        if let Some(region) = self.data.regions.get_mut(index) {
            *region = None;
        }
    }

    pub(crate) fn clear_saved_region_nodes(&mut self) {
        self.data.regions.clear();
    }

    pub(crate) fn enemy_factions(&mut self) -> &[Option<EnemyFactionSaveSnapshot>] {
        self.data.enemy_factions.make_contiguous()
    }

    /// Уничтожает enemy values в list-order и очищает pointer-list.
    pub(crate) fn clear_saved_enemy_factions(&mut self) {
        for enemy_faction in &mut self.data.enemy_factions {
            *enemy_faction = None;
        }
        self.data.enemy_factions.clear();
    }

    pub(crate) fn countries_len(&self) -> usize {
        self.data.countries.len()
    }

    pub(crate) fn first_country(&self) -> Option<Option<&CountrySaveSnapshot>> {
        self.data.countries.front().map(Option::as_ref)
    }

    /// Удаляет текущий country node и его non-null save-копию перед next.
    pub(crate) fn remove_first_saved_country(&mut self) {
        drop(self.data.countries.pop_front());
    }
}

/// Безопасная граница двух исходных `char[260]` перед custom lowercase.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerNameLookupError {
    PlayerNameTooLongForLegacyBuffer { player_id: u32, length: usize },
    RequestedNameTooLongForLegacyBuffer { length: usize },
}

/// Exact terminal CPlayer::ChangeName branch до однобайтового ответа GS.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerNameChangeDisposition {
    PlayerMissing,
    NullName,
    NameTooLong { length: usize },
    CurrentNameMissingSpecialString,
    InvalidString,
    MapPlayerNameExists,
    DbDataNameExists,
    DbCreationNameExists,
    PersistentNameExists,
    Changed { previous_name: Vec<u8> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldPlayerNameChangeReport {
    pub(crate) player_id: u32,
    pub(crate) requested_name: Vec<u8>,
    pub(crate) legacy_result: i32,
    pub(crate) disposition: WorldPlayerNameChangeDisposition,
}

/// Результат `AppendCreationPlayer` с явным владением на каждой ветви.
pub(crate) enum WorldCreationPlayerAppendOutcome {
    Inserted {
        player_id: u32,
    },
    DuplicateReleased {
        player_id: u32,
    },
    ExistingMapOwnerKept {
        player_id: u32,
        incoming: Box<CPlayer>,
    },
}

/// Ordered результат полного `CGame::AddOrginGoodsToPlayer`.
#[derive(Debug)]
pub(crate) struct WorldOriginGoodsReport {
    pub(crate) entries: Vec<PlayerOriginEquipmentOutcome>,
}

#[derive(Debug)]
pub(crate) struct WorldOriginGoodsBlock {
    pub(crate) origin_index: usize,
    pub(crate) source: PlayerOriginEquipmentBlock,
}

/// Safe-граница constructor-loaded process-wide player ID.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldPlayerIdBlock;

/// Результат точной nullable insertion-границы `AppendMapPlayer`.
pub(crate) enum WorldMapPlayerAppendOutcome {
    Inserted {
        player_id: u32,
    },
    ExistingOwnerKept {
        player_id: u32,
        incoming: Box<CPlayer>,
    },
}

/// Безопасная граница старого 256-байтового результата `CGame::CheckPoint`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldCheckPointBlock {
    pub(crate) input_length: usize,
    pub(crate) escaped_length: usize,
}

impl fmt::Display for WorldCheckPointBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CheckPoint расширяет {} входных байт до {}, поэтому старый char[256] остаётся без NUL",
            self.input_length, self.escaped_length
        )
    }
}

impl Error for WorldCheckPointBlock {}

/// Точный payload двух `AddLogText` внутри `AppendCreationPlayer`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldCreationPlayerAppendLog {
    Duplicate { player_id: u32 },
    ExistingMapOwner,
}

impl fmt::Display for WorldCreationPlayerAppendLog {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate { player_id } => {
                write!(formatter, "{player_id} Player Is In CreationPlayerList.")
            }
            Self::ExistingMapOwner => formatter.write_str("MapPlayer Not Found or NULL."),
        }
    }
}

/// Тонкий concrete adapter organizing/region/transport owner-ов
/// `CPlayer::UpdateFactionInfo`.
struct WorldPlayerFactionInfoContext<'a> {
    game: &'a CGame,
    organizing: &'a COrganizingCtrl,
    region_types: &'a BTreeMap<i32, Option<u16>>,
}

impl PlayerOrganizingUpdater for WorldPlayerFactionInfoContext<'_> {
    fn set_player_organizing(
        &mut self,
        player_id: i32,
        organizing: &mut PlayerOrganizingState,
    ) -> Result<(), PlayerOrganizingUpdateError> {
        let mut updater = self.organizing.player_updater(self.region_types);
        updater.set_player_organizing(player_id, organizing)
    }
}

impl PlayerFactionInfoContext for WorldPlayerFactionInfoContext<'_> {
    fn send_player_faction_info(
        &mut self,
        player_id: i32,
        message: &CMessage,
    ) -> PlayerFactionInfoDelivery {
        let game_server_id = self.game.game_server_number_by_player_id(player_id);
        PlayerFactionInfoDelivery {
            game_server_id,
            result: self.game.send_msg_to_game_server(game_server_id, message),
        }
    }
}

/// Достигнутая setup-часть исходного `CGame`; другие поля добавляются owners.
pub(crate) struct CGame {
    setup: WorldSetup,
    /// Process-global `CThingSetup` привязан к единственному World `CGame`.
    thing_setup: CThingSetup,
    emotion: CEmotion,
    globe_variables: WorldGlobeVariables,
    string_table: MyStringTable,
    string_table_array: Vec<u8>,
    words_filter: CWordsFilter,
    dupli_region_setup: Option<CDupliRegionSetup>,
    equipment_compose_list: EquipmentComposeList,
    ci_qing_setup: CCiQingSetup,
    tao_zhuang_setup: CTaoZhuangSetup,
    hit_level_setup: CHitLevelSetup,
    trade_list: CTradeList,
    increment_shop_list: CIncrementShopList,
    prison_conf: PrisonConf,
    contribute_setup: CContributeSetup,
    net_client: Option<CMyNetClient>,
    net_server: Option<CMyNetServer>,
    regions: BTreeMap<i32, WorldRegionAssignment>,
    function_list_file_data: Option<Vec<u8>>,
    variable_list_file_data: Option<Vec<u8>>,
    script_file_data: BTreeMap<Vec<u8>, Vec<u8>>,
    game_servers: BTreeMap<u32, WorldGameServerEntry>,
    system_broadcasts: VecDeque<WorldSystemBroadcast>,
    goods_links: VecDeque<WorldGoodsLink>,
    write_log_queue: WorldWriteLogQueue,
    player_data_queue: CPlayerDataQueue,
    player_load_queue: CPlayerLoadQueue,
    players: BTreeMap<u32, Box<CPlayer>>,
    team_session_ids: BTreeMap<u32, i32>,
    creation_players: VecDeque<i32>,
    restore_players: VecDeque<u32>,
    deletion_players: VecDeque<DeletionPlayerSnapshot>,
    player_id: Option<u32>,
    leave_word_id: Option<i32>,
    online_players: VecDeque<u32>,
    offline_players: VecDeque<u32>,
    login_players: VecDeque<WorldLoginPlayerEntry>,
    db_responses: i32,
    db_data: Mutex<WorldDbData>,
    ping_game_servers: Vec<WorldPingGameServerInfo>,
    bai_tan_requests: BTreeMap<u32, i32>,
    bai_tan_routes: BTreeMap<i32, i32>,
    bai_tan_ip_refcounts: BTreeMap<u32, i32>,
    bai_tan_player_ips: BTreeMap<i32, u32>,
    honor_eliminate_list: BTreeMap<u32, VecDeque<u32>>,
    login_server_id: i32,
    ping_in_progress: bool,
    last_ping_game_server_time_ms: u32,
    game_server_message_time_ms: u32,
    login_server_message_time_ms: u32,
}

impl CGame {
    /// Возвращает exact faction-owner через явную замену organizing singleton-а.
    pub(crate) fn get_faction_by_id(
        organizing: &COrganizingCtrl,
        faction_id: i32,
    ) -> Option<&CFaction> {
        if faction_id == 0 {
            None
        } else {
            organizing.faction_by_id(faction_id)
        }
    }

    /// Делегирует exact World overload достигнутому owned `CWordsFilter`.
    pub(crate) fn check_invalid_string(&self, value: &mut Vec<u8>, replace: bool) -> bool {
        self.words_filter.check(value, replace)
    }

    /// Делегирует create-role overload с отдельным all-numbers gate.
    pub(crate) fn check_create_role_name(
        &self,
        value: &mut Vec<u8>,
        replace: bool,
        reject_all_numbers: bool,
    ) -> bool {
        self.words_filter
            .check_with_numeric_gate(value, replace, reject_all_numbers)
    }

    pub(crate) fn words_filter(&self) -> &CWordsFilter {
        &self.words_filter
    }

    pub(crate) fn emotion(&self) -> &CEmotion {
        &self.emotion
    }

    pub(crate) fn equipment_compose_list(&self) -> &EquipmentComposeList {
        &self.equipment_compose_list
    }

    pub(crate) fn ci_qing_setup(&self) -> &CCiQingSetup {
        &self.ci_qing_setup
    }

    pub(crate) fn tao_zhuang_setup(&self) -> &CTaoZhuangSetup {
        &self.tao_zhuang_setup
    }

    pub(crate) fn hit_level_setup(&self) -> &CHitLevelSetup {
        &self.hit_level_setup
    }

    pub(crate) fn trade_list(&self) -> &CTradeList {
        &self.trade_list
    }

    pub(crate) fn increment_shop_list(&self) -> &CIncrementShopList {
        &self.increment_shop_list
    }

    pub(crate) fn prison_conf(&self) -> &PrisonConf {
        &self.prison_conf
    }

    pub(crate) fn contribute_setup(&self) -> &CContributeSetup {
        &self.contribute_setup
    }

    pub(crate) fn dupli_region_setup(&self) -> &CDupliRegionSetup {
        self.dupli_region_setup
            .as_ref()
            .expect("CDupliRegionSetup доступен только после успешного CGame::Init")
    }

    /// Создаёт `tagSetup`, затем применяет четыре точные записи `CGame::CGame`.
    pub(crate) fn new() -> Self {
        Self {
            setup: WorldSetup::for_game(),
            thing_setup: CThingSetup::new(),
            emotion: CEmotion::default(),
            globe_variables: WorldGlobeVariables::default(),
            string_table: MyStringTable::new(),
            string_table_array: Vec::new(),
            words_filter: CWordsFilter::new(),
            dupli_region_setup: None,
            equipment_compose_list: EquipmentComposeList::default(),
            ci_qing_setup: CCiQingSetup::default(),
            tao_zhuang_setup: CTaoZhuangSetup::default(),
            hit_level_setup: CHitLevelSetup::default(),
            trade_list: CTradeList::default(),
            increment_shop_list: CIncrementShopList::default(),
            prison_conf: PrisonConf::default(),
            contribute_setup: CContributeSetup::default(),
            net_client: None,
            net_server: None,
            regions: BTreeMap::new(),
            function_list_file_data: None,
            variable_list_file_data: None,
            script_file_data: BTreeMap::new(),
            game_servers: BTreeMap::new(),
            system_broadcasts: VecDeque::new(),
            goods_links: std::iter::repeat_with(WorldGoodsLink::placeholder)
                .take(INITIAL_GOODS_LINK_PLACEHOLDERS)
                .collect(),
            write_log_queue: WorldWriteLogQueue::default(),
            player_data_queue: CPlayerDataQueue::new(),
            player_load_queue: CPlayerLoadQueue::new(),
            players: BTreeMap::new(),
            team_session_ids: BTreeMap::new(),
            creation_players: VecDeque::new(),
            restore_players: VecDeque::new(),
            deletion_players: VecDeque::new(),
            player_id: None,
            leave_word_id: None,
            online_players: VecDeque::new(),
            offline_players: VecDeque::new(),
            login_players: VecDeque::new(),
            db_responses: 0,
            db_data: Mutex::new(WorldDbData::new()),
            ping_game_servers: Vec::new(),
            bai_tan_requests: BTreeMap::new(),
            bai_tan_routes: BTreeMap::new(),
            bai_tan_ip_refcounts: BTreeMap::new(),
            bai_tan_player_ips: BTreeMap::new(),
            honor_eliminate_list: BTreeMap::new(),
            login_server_id: 0,
            ping_in_progress: false,
            last_ping_game_server_time_ms: legacy_tick_ms(),
            game_server_message_time_ms: 0,
            login_server_message_time_ms: 0,
        }
    }

    /// Exact `ClearStringTable`: очищает map и прежний coded buffer.
    pub(crate) fn clear_string_table(&mut self) {
        self.string_table.table_mut().free();
        self.string_table_array.clear();
    }

    /// Выполняет file-overload через уже выбранный caller-ом resource backend.
    pub(crate) fn load_string_table_resource(
        &mut self,
        package: &[u8],
        source: Option<&[u8]>,
    ) -> WorldStringTableLoadReport {
        let package = legacy_c_string_prefix(package);
        let succeeded = if package.is_empty() {
            self.string_table
                .table_mut()
                .reject_empty_resource_name();
            false
        } else if let Some(source) = source {
            self.string_table.table_mut().load_bytes(source)
        } else {
            self.string_table
                .table_mut()
                .reject_missing_resource(package);
            false
        };

        let mut log_payload = b"Load language packet [".to_vec();
        log_payload.extend_from_slice(package);
        if succeeded {
            log_payload.extend_from_slice(b"]...OK!");
        } else {
            log_payload.extend_from_slice(b"]...FAILED! : ");
            log_payload.extend_from_slice(self.string_table.table().last_error());
        }

        WorldStringTableLoadReport {
            package: package.to_vec(),
            succeeded,
            log_payload,
        }
    }

    /// Дописывает current ordered table в coded buffer, как исходный owner.
    pub(crate) fn code_string_table(
        &mut self,
    ) -> Result<(), WorldStringTableEncodingBlock> {
        self.string_table
            .to_byte_array(&mut self.string_table_array)
            .map_err(|entry_count| WorldStringTableEncodingBlock { entry_count })
    }

    pub(crate) fn get_string_table_byte_array(&self) -> &[u8] {
        &self.string_table_array
    }

    /// CGame-обёртка превращает nullable miss базового owner-а в пустую строку.
    pub(crate) fn get_string_by_id(&self, string_id: &[u8]) -> &[u8] {
        self.string_table
            .table()
            .get_string_by_id(legacy_c_string_prefix(string_id))
            .unwrap_or_default()
    }

    /// Exact reload всегда игнорирует переданное имя и перечитывает default и
    /// настроенный packages. Resource I/O остаётся инфраструктурным callback-ом.
    pub(crate) fn update_string_table<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
        requested_package: &[u8],
    ) -> WorldStringTableUpdateReport {
        const DEFAULT_LANGUAGE: &[u8] = b"data/Language.lag";

        self.clear_string_table();
        let source = context.read_resource(DEFAULT_LANGUAGE);
        let default = self.load_string_table_resource(DEFAULT_LANGUAGE, source.as_deref());
        context.add_log_text(&default.log_payload);
        if !default.succeeded {
            return WorldStringTableUpdateReport {
                requested_package: requested_package.to_vec(),
                completion: WorldStringTableUpdateCompletion::DefaultLanguageFailed,
            };
        }

        let configured_package = self.setup.language_package.clone();
        let source = context.read_resource(&configured_package);
        let configured =
            self.load_string_table_resource(&configured_package, source.as_deref());
        context.add_log_text(&configured.log_payload);
        if !configured.succeeded {
            return WorldStringTableUpdateReport {
                requested_package: requested_package.to_vec(),
                completion: WorldStringTableUpdateCompletion::ConfiguredLanguageFailed,
            };
        }

        if let Err(block) = self.code_string_table() {
            return WorldStringTableUpdateReport {
                requested_package: requested_package.to_vec(),
                completion: WorldStringTableUpdateCompletion::EncodingBlocked(block),
            };
        }
        if self.string_table_array.is_empty() {
            context.add_log_text(
                b"WARNING : Language packet is NULL, will NOT send to WorldServer.",
            );
            return WorldStringTableUpdateReport {
                requested_package: requested_package.to_vec(),
                completion: WorldStringTableUpdateCompletion::Empty,
            };
        }

        let mut message = CMessage::new(0x0007_F807);
        message.base_mut().add(&self.string_table_array);
        let delivery = message.send_all(self.current_game_server_sender().as_ref());
        context.add_log_text(b"Send the new language packet to all the GameServers.");
        WorldStringTableUpdateReport {
            requested_package: requested_package.to_vec(),
            completion: WorldStringTableUpdateCompletion::Broadcast {
                message_type: 0x0007_F807,
                payload_length: self.string_table_array.len(),
                delivery,
            },
        }
    }

    /// Добавляет точную POD-запись в хвост `m_listGoodsLink`.
    ///
    /// Constructor уже создал 500 нулевых placeholder-ов, а process-global
    /// индекс начинается с `1`. Changed-запись сохраняет ID декодированного
    /// товара и global не двигает. Редкая `list::max_size` ветвь удаляет голову;
    /// Rust одновременно освобождает её owned товар, исправляя только утечку.
    pub(crate) fn add_goods_link(&mut self, mut link: WorldGoodsLink) -> u32 {
        if self.goods_links.len() == LEGACY_GOODS_LINK_MAX_SIZE {
            let _ = self.goods_links.pop_front();
        }
        if matches!(&link.payload, WorldGoodsLinkPayload::Original { .. }) {
            link.index = NEXT_GOODS_LINK_INDEX.fetch_add(1, Ordering::Relaxed);
        }
        let index = link.index;
        self.goods_links.push_back(link);
        index
    }

    /// Ставит структурированную DB-команду в хвост исходного write-log FIFO.
    pub(crate) fn push_write_log_command(&self, command: WorldWriteLogCommand) -> usize {
        self.write_log_queue.push(command)
    }

    /// Ставит единственную созданную `CLargess::LoadLargess` запись в общий FIFO.
    pub(crate) fn publish_largess_load_log(
        &self,
        report: &mut LoadLargessReport,
    ) -> Option<usize> {
        report.write_log.take().map(|record| {
            self.push_write_log_command(WorldWriteLogCommand::LargessLog(record))
        })
    }

    /// Выполняет доменную выдачу и сразу публикует её optional log в общий FIFO.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn load_player_largess<Random, Upgrade>(
        &self,
        largess: &TiberiusLargess,
        player: &mut CPlayer,
        registry: &GoodsBasePropertiesRegistry,
        gold_coin_index: u32,
        gold_coin_limit: u32,
        use_log_system: bool,
        random: &mut Random,
        upgrade_equipment: &mut Upgrade,
    ) -> Result<WorldPlayerLargessLoadReport, LoadLargessBlock>
    where
        Random: FnMut(i32) -> i32 + ?Sized,
        Upgrade: FnMut(&mut CGoods, i32) + ?Sized,
    {
        let mut load = largess.load_largess(
            player,
            registry,
            gold_coin_index,
            gold_coin_limit,
            use_log_system,
            random,
            upgrade_equipment,
        )?;
        let write_log_queue_length = self.publish_largess_load_log(&mut load);
        Ok(WorldPlayerLargessLoadReport {
            load,
            write_log_queue_length,
        })
    }

    /// Копирует четыре credential-поля отдельного Log DB connection-owner-а.
    fn write_log_worker_spec(&self) -> WorldWriteLogWorkerSpec {
        let settings = WorldDatabaseSettings::from_parts(WorldDatabaseSettingsParts {
            host: self.setup.log_system_server.clone(),
            database: self.setup.log_system_database.clone(),
            user: self.setup.log_system_user.clone(),
            password: self.setup.log_system_password.clone(),
        });
        WorldWriteLogWorkerSpec::new(
            self.setup.use_log_system,
            settings,
            self.write_log_queue.clone(),
        )
    }

    /// Отделяет две shared FIFO от остального mutable `CGame` для DB worker-а.
    fn player_load_worker_spec(&self) -> WorldPlayerLoadWorkerSpec {
        WorldPlayerLoadWorkerSpec::new(
            self.player_load_queue.clone(),
            self.player_data_queue.clone(),
        )
    }

    /// Возвращает первое совпадение в list-order, включая constructor-ный
    /// placeholder для индекса `0`.
    pub(crate) fn find_goods_link(&self, index: u32) -> Option<&WorldGoodsLink> {
        self.goods_links.iter().find(|link| link.index == index)
    }

    /// Возвращает exact appearance snapshot экипировки игрока.
    ///
    /// Rust-ссылка исключает недоказанный null-вызов; EXE без проверок проходит
    /// slots `0,1,3,4,2,9,10,12,13,14,15`, оставляет нули для пустых slots и
    /// сужает signed `GAP_WEAPON_LEVEL` до младшего байта.
    pub(crate) fn get_player_equip_id(
        &self,
        player: &CPlayer,
    ) -> Result<PlayerEquipmentWireSnapshot, PlayerDbProjectionBlock> {
        player.equipment_wire_snapshot()
    }

    /// Вычисляет комиссию и остаток продавца по exact World auction-контракту.
    ///
    /// `None` заменяет единственную исходную проверку nullable `CGoodsNode*`.
    /// `dwMoneySeller` сначала читается как signed Windows `long`; затем EXE
    /// умножает его на `fAuctionFactorC`, отбрасывает дробную часть и поочерёдно
    /// ограничивает `fSxfJinMin/fSxfJinMax`. Целочисленное разложение factor-а
    /// сохраняет x87-произведение без лишнего `f32`-округления. Невалидные и
    /// out-of-range setup-значения определённо насыщаются вместо UB старого cast.
    pub(crate) fn get_opt_money_jin(
        &self,
        globe_setup: &GlobeSetupSnapshot,
        seller_money: Option<u32>,
    ) -> Option<WorldAuctionSellerMoney> {
        let seller_money = seller_money? as i32;
        let mut fee =
            truncate_scaled_legacy_money(seller_money, globe_setup.auction_factor_c());
        if f64::from(fee) < f64::from(globe_setup.auction_fee_minimum()) {
            fee = truncate_legacy_money(f64::from(globe_setup.auction_fee_minimum()));
        }
        if f64::from(globe_setup.auction_fee_maximum()) < f64::from(fee) {
            fee = truncate_legacy_money(f64::from(globe_setup.auction_fee_maximum()));
        }

        let seller_money_after_fee = if fee <= seller_money {
            seller_money.wrapping_sub(fee)
        } else {
            fee = seller_money;
            0
        };
        Some(WorldAuctionSellerMoney {
            fee,
            seller_money_after_fee,
        })
    }

    /// Возвращает byte-exact script buffer по case-sensitive normalized key.
    pub(crate) fn get_script_file_data(&self, path: &[u8]) -> Option<&[u8]> {
        self.script_file_data
            .get(legacy_c_string_prefix(path))
            .map(Vec::as_slice)
    }

    /// Возвращает загруженный LeiTing setup для initial-config serializer-а.
    pub(crate) const fn thing_setup(&self) -> &CThingSetup {
        &self.thing_setup
    }

    /// Nullable raw owner начального пакета function-list subtype `0x0A`.
    pub(crate) fn function_list_file_data(&self) -> Option<&[u8]> {
        self.function_list_file_data.as_deref()
    }

    /// Nullable raw owner начального пакета variable-list subtype `0x0B`.
    pub(crate) fn variable_list_file_data(&self) -> Option<&[u8]> {
        self.variable_list_file_data.as_deref()
    }

    /// Ordered C-string view initial-config `m_mapScript_FileData`.
    pub(crate) fn initial_script_files(&self) -> impl Iterator<Item = (&[u8], &[u8])> + '_ {
        self.script_file_data.iter().map(|(path, data)| {
            (
                legacy_c_string_prefix(path),
                legacy_c_string_prefix(data),
            )
        })
    }

    /// Загружает один script и заменяет прежний owner с тем же normalized key.
    pub(crate) fn load_one_script<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
        path: &[u8],
    ) -> bool {
        let path = legacy_c_string_prefix(path);
        let Some(data) = context.read_resource(path) else {
            let mut message = b"Can't found ".to_vec();
            message.extend_from_slice(path);
            message.push(b'!');
            context.notify_reload_operator(b"Message", &message);
            return false;
        };

        let mut normalized = path;
        if normalized.first() == Some(&b'\\') {
            normalized = &normalized[1..];
        }
        let normalized = normalized
            .iter()
            .map(|byte| if *byte == b'\\' { b'/' } else { *byte })
            .collect::<Vec<_>>();
        self.script_file_data.insert(normalized, data);
        true
    }

    /// Очищает три прежних script-owner-а и повторяет exact load order.
    pub(crate) fn load_script_file_data<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
        _script_directory: &[u8],
        function_file: &[u8],
        variable_file: &[u8],
        _general_variable_data_file: &[u8],
    ) -> bool {
        self.function_list_file_data = None;
        self.variable_list_file_data = None;
        self.script_file_data.clear();

        let Some(function_data) = context.read_resource(function_file) else {
            Self::notify_missing_reload_file(context, function_file);
            return false;
        };
        self.function_list_file_data = Some(function_data);

        let Some(variable_data) = context.read_resource(variable_file) else {
            Self::notify_missing_reload_file(context, variable_file);
            return false;
        };
        self.variable_list_file_data = Some(variable_data);

        // Исходный RVA 0x00014450 использует literal, а не `script_directory`.
        for path in context.script_files(b"scripts/*.*", b".script") {
            let _ = self.load_one_script(context, &path);
        }
        true
    }

    fn notify_missing_reload_file<Context: WorldReloadContext + ?Sized>(
        context: &mut Context,
        path: &[u8],
    ) {
        let mut message = b"Can't found ".to_vec();
        message.extend_from_slice(legacy_c_string_prefix(path));
        message.push(b'!');
        context.notify_reload_operator(b"Message", &message);
    }

    /// Перезагружает один script, затем публикует exact `0x7F801/0x0D` wire.
    pub(crate) fn reload_one_script<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
        path: &[u8],
    ) -> WorldReloadOneScriptResult {
        let path = legacy_c_string_prefix(path);
        if !self.load_one_script(context, path) {
            let mut log = b"Reload Script(".to_vec();
            log.extend_from_slice(path);
            log.extend_from_slice(b")...FAILED!");
            context.add_log_text(&log);
            return Ok(false);
        }

        let mut log = b"Reload Script(".to_vec();
        log.extend_from_slice(path);
        log.extend_from_slice(b")...OK!");
        context.add_log_text(&log);

        let Some(data) = self.get_script_file_data(path).map(legacy_c_string_prefix) else {
            // BLOCKED_MISSING_FACT: RVA 0x00013760 передаёт исходный path в
            // GetScriptFileData после того, как LoadOneScript нормализовал
            // только map-key. При несовпадении оригинал вызывает lstrlen(NULL).
            return Err(WorldReloadOneScriptBlock {
                requested_path: path.to_vec(),
                normalized_map_key: normalize_script_path(path),
            });
        };
        let mut message = CMessage::new(0x0007_F801);
        message.base_mut().add_long(0x0D);
        add_legacy_c_string(message.base_mut(), path);
        message.base_mut().add_long(data.len() as u32 as i32);
        add_legacy_c_string(message.base_mut(), data);
        let sender = self.current_game_server_sender();
        let _ = message.send_all(sender.as_ref());
        Ok(true)
    }

    fn configure_region_owner(region: &mut CWorldRegion, spec: &WorldRegionLoadSpec) {
        region.set_region_identity(spec.region_id, &spec.name);
        region.set_region_list_base_fields(
            spec.resource_id,
            spec.exp_scale,
            spec.country,
            spec.notify,
        );
        region.set_world_region_list_fields(spec.region_type, spec.no_pk, spec.no_contribute);
    }

    fn materialize_region_owner<Context: WorldReloadContext + ?Sized>(
        context: &mut Context,
        spec: &WorldRegionLoadSpec,
    ) -> Result<WorldRegionMaterialization, WorldRegionListBlock> {
        if !matches!(spec.region_type, 0..=5) {
            return Ok(WorldRegionMaterialization::MissingSubtype);
        }

        match spec.region_type {
            0 | 4 | 5 => {
                let mut region = Box::new(CWorldRegion::with_constructor_region_base());
                Self::configure_region_owner(&mut region, spec);
                let counts =
                    region
                        .load_from_context(context)
                        .map_err(|source| WorldRegionListBlock {
                            region_id: spec.region_id,
                            source: WorldRegionOwnerLoadBlock::Base(source),
                        })?;
                let loaded = counts.base_failure.is_none();
                Ok(WorldRegionMaterialization::Direct {
                    owner: WorldRegionOwner::Base(region),
                    counts,
                    loaded,
                })
            }
            1 => {
                let mut region = Box::new(CWorldVillageRegion::with_constructor_state());
                Self::configure_region_owner(region.war_mut().base_mut(), spec);
                let counts =
                    region
                        .load_from_context(context)
                        .map_err(|source| WorldRegionListBlock {
                            region_id: spec.region_id,
                            source: WorldRegionOwnerLoadBlock::Village(source),
                        })?;
                Ok(WorldRegionMaterialization::Direct {
                    owner: WorldRegionOwner::Village(region),
                    counts,
                    // RVA 0x00079F90 игнорирует base-result и возвращает `1`.
                    loaded: true,
                })
            }
            2 => {
                let mut region = Box::new(CWorldCityRegion::with_constructor_state());
                Self::configure_region_owner(region.war_mut().base_mut(), spec);
                let outcome =
                    region
                        .load_from_context(context)
                        .map_err(|source| WorldRegionListBlock {
                            region_id: spec.region_id,
                            source: WorldRegionOwnerLoadBlock::City(source),
                        })?;
                Ok(WorldRegionMaterialization::Direct {
                    owner: WorldRegionOwner::City(region),
                    counts: outcome.counts,
                    loaded: outcome.loaded,
                })
            }
            3 => {
                let mut region = Box::new(WorldCountryWarRegion::with_constructor_state());
                Self::configure_region_owner(region.base_mut(), spec);
                let outcome =
                    region
                        .load_from_context(context)
                        .map_err(|source| WorldRegionListBlock {
                            region_id: spec.region_id,
                            source: WorldRegionOwnerLoadBlock::Country(source),
                        })?;
                Ok(WorldRegionMaterialization::Direct {
                    owner: WorldRegionOwner::Country(region),
                    counts: outcome.counts,
                    loaded: outcome.loaded,
                })
            }
            _ => unreachable!("region type отфильтрован выше"),
        }
    }

    /// Загружает positional region-list и публикует только успешно loaded owners.
    pub(crate) fn load_region_list<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
        path: &[u8],
    ) -> Result<bool, WorldRegionListBlock> {
        let Some(data) = context.read_resource(path) else {
            let mut message = b"Can't find file ".to_vec();
            message.extend_from_slice(legacy_c_string_prefix(path));
            context.notify_reload_operator(b"message", &message);
            return Ok(false);
        };

        let mut tokens = WorldServerSetupTokens::new(&data);
        let mut previous_monsters = 0i32;
        let mut previous_npcs = 0i32;

        while tokens.seek_to(b"#") {
            let region_id = tokens.next_ascii().unwrap_or(0);
            let resource_id = tokens.next_ascii().unwrap_or(0);
            let exp_scale = tokens.next_ascii().unwrap_or(1.0);
            let region_type = tokens.next_ascii().unwrap_or(0);
            let no_pk = tokens.next_ascii::<i32>().unwrap_or(0) != 0;
            let no_contribute = tokens.next_ascii::<i32>().unwrap_or(0) != 0;
            let string_id = tokens.next_bytes().unwrap_or_default().to_vec();
            let game_server_index = tokens.next_ascii().unwrap_or(0);
            let country = tokens.next_ascii::<i32>().unwrap_or(0) as u8;
            let notify = tokens.next_ascii().unwrap_or(0);
            let name = context.reload_world_string_by_id(&string_id);
            let spec = WorldRegionLoadSpec {
                region_id,
                resource_id,
                exp_scale,
                region_type,
                no_pk,
                no_contribute,
                name,
                game_server_index,
                country,
                notify,
            };

            let (owner, total_monster_count, total_npc_count, loaded) =
                match Self::materialize_region_owner(context, &spec)? {
                    WorldRegionMaterialization::MissingSubtype => {
                        let mut log = format!("Region ({region_id}) ").into_bytes();
                        log.extend_from_slice(&spec.name);
                        log.extend_from_slice(b"... Read Setup FAILED!");
                        context.add_log_text(&log);
                        continue;
                    }
                    WorldRegionMaterialization::Direct {
                        owner,
                        counts,
                        loaded,
                    } => {
                        let (total_monster_count, total_npc_count) =
                            context.add_region_object_counts(counts.monsters, counts.npcs);
                        (owner, total_monster_count, total_npc_count, loaded)
                    }
                };
            if !loaded {
                let mut log = format!("Region ({region_id}) ").into_bytes();
                log.extend_from_slice(&spec.name);
                log.extend_from_slice(b"...Load FAILED!");
                context.add_log_text(&log);
                continue;
            }
            let mut log = format!("Region ({region_id}) ").into_bytes();
            log.extend_from_slice(&spec.name);
            log.extend_from_slice(
                format!(
                    " [m={} n={}]...OK!",
                    total_monster_count.wrapping_sub(previous_monsters),
                    total_npc_count.wrapping_sub(previous_npcs),
                )
                .as_bytes(),
            );
            context.add_log_text(&log);
            previous_monsters = total_monster_count;
            previous_npcs = total_npc_count;
            // Доказанный loose fixture SHA-256
            // A479A3104813832A2C65A0983C55C00537F6942248EA3BEE9FEBE4B69C8B9406
            // содержит 549 записей и 549 уникальных ID; поэтому Rust Drop
            // заменённого Box не достигается в baseline и не подменяет утечку.
            self.regions.insert(
                region_id,
                WorldRegionAssignment {
                    region: Some(owner),
                    game_server_index,
                    region_type: Some(region_type),
                },
            );
        }
        let (final_monsters, final_npcs) = context.region_object_counts();
        context.add_log_text(format!("Monster={final_monsters} Npc={final_npcs}!").as_bytes());
        Ok(true)
    }

    /// Перечитывает setup одного региона и отправляет его owning GameServer.
    pub(crate) fn reload_one_region_setup<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
        region_id: i32,
    ) -> Result<bool, WorldReloadRegionSetupBlock> {
        let Some(assignment) = self.regions.get_mut(&region_id) else {
            return Ok(false);
        };
        let Some(region) = assignment.region.as_mut() else {
            return Ok(false);
        };
        let region = region.base_mut();
        Self::reload_region_setup_owner(context, region);
        let bytes = region
            .add_setup_to_byte_array()
            .map_err(|source| WorldReloadRegionSetupBlock { region_id, source })?;
        let map_id = assignment.game_server_index as i32;
        let sender = self.current_game_server_sender();
        let mut message = CMessage::new(0x0007_F801);
        message.base_mut().add_long(0x10);
        message.base_mut().add_long(region_id);
        message.base_mut().add(&bytes);
        let _ = message.send_to_map_id(sender.as_ref(), map_id);
        Ok(true)
    }

    /// Перечитывает setup всех живых регионов в signed map-key order.
    pub(crate) fn reload_all_region_setup<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
    ) -> Result<bool, WorldReloadRegionSetupBlock> {
        let sender = self.current_game_server_sender();
        for assignment in self.regions.values_mut() {
            let Some(region) = assignment.region.as_mut() else {
                continue;
            };
            let region = region.base_mut();
            Self::reload_region_setup_owner(context, region);
            let region_id = region.get_id();
            let bytes = region
                .add_setup_to_byte_array()
                .map_err(|source| WorldReloadRegionSetupBlock { region_id, source })?;
            let mut message = CMessage::new(0x0007_F801);
            message.base_mut().add_long(0x10);
            message.base_mut().add_long(region_id);
            message.base_mut().add(&bytes);
            let _ = message.send_to_map_id(sender.as_ref(), assignment.game_server_index as i32);
        }
        Ok(true)
    }

    fn reload_region_setup_owner<Context: WorldReloadContext + ?Sized>(
        context: &mut Context,
        region: &mut CWorldRegion,
    ) {
        let path = format!("regions/{}.rs", region.get_id()).into_bytes();
        if let Some(bytes) = context.read_resource(&path) {
            region.load_setup_bytes(&bytes);
            return;
        }
        let mut message = b"file '".to_vec();
        message.extend_from_slice(&path);
        message.extend_from_slice(b"' can't found!");
        context.notify_reload_operator(b"ERROR", &message);
    }

    fn send_reload_payload(&self, subcode: i32, bytes: &[u8]) {
        let mut message = CMessage::new(0x0007_F801);
        message.base_mut().add_long(subcode);
        message.base_mut().add(bytes);
        let sender = self.current_game_server_sender();
        let _ = message.send_all(sender.as_ref());
    }

    fn reload_boolean_with_log<Context: WorldReloadContext + ?Sized>(
        context: &mut Context,
        owner: WorldReloadBooleanOwner,
        success_log: &[u8],
        failure_log: &[u8],
    ) -> bool {
        let succeeded = context.call_boolean_owner(owner) != 0;
        context.add_log_text(if succeeded { success_log } else { failure_log });
        succeeded
    }

    fn serialize_reload_owner<Context: WorldReloadContext + ?Sized>(
        &self,
        context: &mut Context,
        owner: WorldReloadSerializationOwner,
        subcode: i32,
        update_legacy_result: bool,
        legacy_result: &mut i32,
    ) {
        let bytes = context.serialize_owner(owner);
        if update_legacy_result {
            *legacy_result = bytes.len() as u32 as i32;
        }
        self.send_reload_payload(subcode, &bytes);
    }

    /// Выполняет concrete `CountryWarSys::reload` для main-loop профиля.
    fn reload_country_war<Context, TimerCallback>(
        &mut self,
        context: &mut Context,
        country_war: &mut CountryWarSys,
        timer: &mut CTimer<TimerCallback>,
        callbacks: CountryWarCallbacks<TimerCallback>,
        now: TagTime,
        reload_server_resources: bool,
    ) -> WorldReloadResult
    where
        Context: WorldReloadContext + ?Sized,
        TimerCallback: Copy,
    {
        if reload_server_resources {
            context.load_reload_server_resources(self);
        }
        let source = context.read_resource(b"setup/CountryWarSys.ini");
        let sender = self.current_game_server_sender();
        let report = country_war
            .reload(
                source.as_deref(),
                now,
                timer,
                callbacks,
                |payload| context.add_log_text(payload),
                |message| message.send_all(sender.as_ref()),
            )
            .map_err(WorldReloadBlock::CountryWar)?;
        let succeeded = report.load.legacy_result;
        context.add_log_text(if succeeded {
            b"Load CountryWar...OK!"
        } else {
            b"Load CountryWar...FAILED!"
        });
        Ok(i32::from(succeeded))
    }

    /// Выполняет полный case-insensitive dispatcher `CGame::ReLoad`.
    pub(crate) fn reload<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
        jjc: &mut CJJcSystem,
        profile: &[u8],
        send_to_game_servers: bool,
        reload_server_resources: bool,
    ) -> WorldReloadResult {
        let mut legacy_result = 0i32;
        if reload_server_resources {
            context.load_reload_server_resources(self);
        }
        let profile_name = legacy_c_string_prefix(profile);
        let Some(profile) = WorldReloadProfile::parse(profile_name) else {
            return Ok(legacy_result);
        };

        match profile {
            WorldReloadProfile::PlayerList => {
                const PLAYER_LIST_PATH: &[u8] = b"data/playerlist.ini";
                const ORIGIN_EQUIPMENT_PATH: &[u8] = b"data/playerOrginEquip.ini";
                const EXPERIENCE_PATH: &[u8] = b"data/playerExp.ini";
                const UPGRADES_PATH: &[u8] = b"data/playerPropertiesUpgrade.ini";

                // `LoadPlayerList` открывает второй файл только после успешного
                // первого. Каждая missing-file ветвь сохраняет exact clear scope.
                let player = match context.read_resource(PLAYER_LIST_PATH) {
                    Some(source) => {
                        context
                            .player_list()
                            .load_player_properties_from_bytes(&source)
                            .map_err(WorldReloadBlock::PlayerListFormat)?;
                        match context.read_resource(ORIGIN_EQUIPMENT_PATH) {
                            Some(source) => {
                                context
                                    .player_list()
                                    .load_origin_equipment_from_bytes(&source)
                                    .map_err(WorldReloadBlock::PlayerListFormat)?;
                                true
                            }
                            None => {
                                context.player_list().clear_origin_equipment();
                                false
                            }
                        }
                    }
                    None => {
                        context.player_list().clear_player_properties();
                        false
                    }
                };
                context.add_log_text(if player {
                    b"Load PlayerList playerOrginEquip.ini...OK!"
                } else {
                    b"Load PlayerList playerOrginEquip.ini...FAILED!"
                });
                let experience = match context.read_resource(EXPERIENCE_PATH) {
                    Some(source) => {
                        context
                            .player_list()
                            .load_player_experience_from_bytes(&source)
                            .map_err(WorldReloadBlock::PlayerListFormat)?;
                        true
                    }
                    None => {
                        context.player_list().clear_player_experience();
                        false
                    }
                };
                context.add_log_text(if player & experience {
                    b"Load PlayerExpList playerExp.ini...OK!"
                } else {
                    b"Load PlayerExpList playerExp.ini...FAILED!"
                });
                let properties = match context.read_resource(UPGRADES_PATH) {
                    Some(source) => {
                        let string_table = self.string_table.table();
                        context
                            .player_list()
                            .load_properties_upgrades_from_bytes(&source, &mut |key| {
                                string_table.get_string_by_id(key).map(ToOwned::to_owned)
                            })
                            .map_err(WorldReloadBlock::PlayerListFormat)?;
                        true
                    }
                    None => {
                        context.player_list().clear_properties_upgrades();
                        false
                    }
                };
                let player_complete = player & experience & properties;
                context.add_log_text(if player_complete {
                    b"Load playerPropertiesUpgrade.ini...OK!"
                } else {
                    b"Load Player Property Upgrade List playerPropertiesUpgrade.ini...FAILED!"
                });
                if player_complete && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .player_list()
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::PlayerListSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(1, &payload);
                }
                let emotion = match context.read_resource(b"data/Emotions.ini") {
                    Some(source) => self
                        .emotion
                        .load_from_bytes(&source)
                        .map(|_| true)
                        .map_err(WorldReloadBlock::EmotionFormat)?,
                    None => false,
                };
                context.add_log_text(if emotion {
                    b"Load Emotins.ini...OK!"
                } else {
                    b"Load Emotins.ini...FAILED!"
                });
                if emotion && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.emotion
                        .serialize(&mut payload)
                        .map_err(WorldReloadBlock::EmotionSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(0x15, &payload);
                }
            }
            WorldReloadProfile::GoodsList => {
                if Self::reload_boolean_with_log(
                    context,
                    WorldReloadBooleanOwner::GoodsList,
                    b"Load goodslist.dat...OK!",
                    b"Load goodslist.dat...FAILED!",
                ) && send_to_game_servers
                {
                    self.serialize_reload_owner(
                        context,
                        WorldReloadSerializationOwner::GoodsList,
                        0,
                        true,
                        &mut legacy_result,
                    );
                }
            }
            WorldReloadProfile::MonsterList => {
                let monsters = context.call_boolean_owner(WorldReloadBooleanOwner::MonsterList);
                context.add_log_text(if monsters != 0 {
                    b"Load monsterlist.ini...OK!"
                } else {
                    b"Load monsterlist.ini...FAILED!"
                });
                let drops = context.call_boolean_owner(WorldReloadBooleanOwner::DropGoodsList);
                context.add_log_text(if monsters & drops != 0 {
                    b"Load dropgoodslist.ini...OK!"
                } else {
                    b"Load dropgoodslist.ini...FAILED!"
                });
                if monsters & drops != 0 && send_to_game_servers {
                    self.serialize_reload_owner(
                        context,
                        WorldReloadSerializationOwner::MonsterList,
                        2,
                        true,
                        &mut legacy_result,
                    );
                }
            }
            WorldReloadProfile::TradeList => {
                const PATH: &[u8] = b"data/tradelist.ini";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => {
                        let string_table = self.string_table.table();
                        self.trade_list
                            .load_from_bytes(
                                &source,
                                &mut |id| {
                                    string_table
                                        .get_string_by_id(id)
                                        .map(ToOwned::to_owned)
                                },
                                &mut |original_name| {
                                    context.query_goods_id_by_original_name(original_name)
                                },
                            )
                            .map(|_| true)
                            .map_err(WorldReloadBlock::TradeListFormat)?
                    }
                    None => {
                        self.trade_list.clear();
                        let mut message = b"file '".to_vec();
                        message.extend_from_slice(PATH);
                        message.extend_from_slice(b"' can't found!");
                        context.notify_reload_operator(b"ERROR", &message);
                        false
                    }
                };
                context.add_log_text(if loaded {
                    b"Load tradelist.ini...OK!"
                } else {
                    b"Load tradelist.ini...FAILED!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.trade_list
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::TradeListSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(3, &payload);
                }
            }
            WorldReloadProfile::SkillList => {
                if context.call_boolean_owner(WorldReloadBooleanOwner::SkillUsageCache) == 0 {
                    context.add_log_text(b"Load Skill Usage List...FAILED!");
                    return Ok(legacy_result);
                }
                if Self::reload_boolean_with_log(
                    context,
                    WorldReloadBooleanOwner::SkillCache,
                    b"Load Skillist...OK!",
                    b"Load Skillist...FAILED!",
                ) && send_to_game_servers
                {
                    self.serialize_reload_owner(
                        context,
                        WorldReloadSerializationOwner::SkillList,
                        6,
                        true,
                        &mut legacy_result,
                    );
                }
            }
            WorldReloadProfile::NewSkillMonsterList => {
                const PATH: &[u8] = b"data/NewSkillMonsterList.xml";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => {
                        let string_table = self.string_table.table();
                        match context.new_skill_monster_conf().load_from_bytes(
                            &source,
                            &mut |key| string_table.get_string_by_id(key).map(ToOwned::to_owned),
                        ) {
                            Ok(report) => {
                                for count in report.read_monster_counts {
                                    let count = count as u32 as i32;
                                    context.add_log_text(
                                        format!("read monster num: {count}").as_bytes(),
                                    );
                                }
                                true
                            }
                            Err(error) => {
                                context.add_log_text(error.log_payload());
                                false
                            }
                        }
                    }
                    None => {
                        context.new_skill_monster_conf().clear();
                        context.add_log_text(
                            b"error: original name in file [NewSkillMonsterList.xml] not exist!!",
                        );
                        false
                    }
                };
                context.add_log_text(if loaded {
                    b"Load NewSkillMonsterList.xml...ok!"
                } else {
                    b"Load NewSkillMonsterList.xml...failed!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .new_skill_monster_conf()
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::NewSkillMonsterSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(0x22, &payload);
                }
            }
            WorldReloadProfile::GlobeSetup | WorldReloadProfile::GameSetup => {
                let (owner, ok, failed) = if profile == WorldReloadProfile::GlobeSetup {
                    (
                        WorldReloadBooleanOwner::GlobeSetup,
                        b"Load globesetup.ini...OK!".as_slice(),
                        b"Load globesetup.ini...FAILED!".as_slice(),
                    )
                } else {
                    (
                        WorldReloadBooleanOwner::GameSetup,
                        b"Load gamesetup.ini...OK!".as_slice(),
                        b"Load gamesetup.ini...FAILED!".as_slice(),
                    )
                };
                let succeeded = Self::reload_boolean_with_log(context, owner, ok, failed);
                context.add_log_text(if succeeded {
                    b"Load AuctionList.ini...OK!"
                } else {
                    b"Load AuctionList.ini...FAILED!"
                });
                if succeeded && send_to_game_servers {
                    self.serialize_reload_owner(
                        context,
                        WorldReloadSerializationOwner::GlobeSetup,
                        7,
                        true,
                        &mut legacy_result,
                    );
                }
            }
            WorldReloadProfile::StringTable => {
                let _ = self.update_string_table(context, profile_name);
            }
            WorldReloadProfile::LogSystem => {
                if Self::reload_boolean_with_log(
                    context,
                    WorldReloadBooleanOwner::LogSystem,
                    b"Load LogSystem.ini...OK!",
                    b"Load LogSystem.ini...FAILED!",
                ) && send_to_game_servers
                {
                    self.serialize_reload_owner(
                        context,
                        WorldReloadSerializationOwner::LogSystem,
                        8,
                        true,
                        &mut legacy_result,
                    );
                }
            }
            WorldReloadProfile::GmList => {
                let gm = Self::reload_boolean_with_log(
                    context,
                    WorldReloadBooleanOwner::GmList,
                    b"Load GMList.ini...OK!",
                    b"Load gmlist.ini...FAILED!",
                );
                let player_gm = Self::reload_boolean_with_log(
                    context,
                    WorldReloadBooleanOwner::PlayerGmList,
                    b"Load playerGMList.ini...OK!",
                    b"Load playerGMList.ini...FAILED!",
                );
                if gm && player_gm && send_to_game_servers {
                    self.serialize_reload_owner(
                        context,
                        WorldReloadSerializationOwner::GmList,
                        9,
                        true,
                        &mut legacy_result,
                    );
                }
            }
            WorldReloadProfile::ScriptFile => {
                let succeeded = self.load_script_file_data(
                    context,
                    b"scripts/",
                    b"data/function.ini",
                    b"data/variable.ini",
                    b"data/general_variable_data.ini",
                );
                context.add_log_text(if succeeded {
                    b"Load function.ini...OK!"
                } else {
                    b"Load function.ini...FAILED!"
                });
                if succeeded && send_to_game_servers {
                    self.send_script_reload_data();
                }
            }
            WorldReloadProfile::RegionList => {
                let succeeded = self
                    .load_region_list(context, b"setup/regionlist.ini")
                    .map_err(WorldReloadBlock::RegionList)?;
                context.add_log_text(if succeeded {
                    b"Load regionlist.ini...OK!"
                } else {
                    b"Load regionlist.ini...FAILED!"
                });
                if succeeded && send_to_game_servers {
                    self.send_loaded_regions(&mut legacy_result)
                        .map_err(WorldReloadBlock::RegionSnapshot)?;
                }
            }
            WorldReloadProfile::RegionLevelSetup => {
                if Self::reload_boolean_with_log(
                    context,
                    WorldReloadBooleanOwner::RegionLevelSetup,
                    b"Load regionlevelsetup.ini...OK!",
                    b"Load regionlevelsetup.ini...FAILED!",
                ) && send_to_game_servers {
                    self.serialize_reload_owner(
                        context,
                        WorldReloadSerializationOwner::RegionLevelSetup,
                        0x11,
                        true,
                        &mut legacy_result,
                    );
                }
            }
            WorldReloadProfile::HitLevelSetup => {
                const PATH: &[u8] = b"data/hitlevel.ini";
                let succeeded = match context.read_resource(PATH) {
                    Some(source) => self
                        .hit_level_setup
                        .load_from_bytes(&source)
                        .map(|_| true)
                        .map_err(WorldReloadBlock::HitLevelFormat)?,
                    None => {
                        self.hit_level_setup.clear();
                        let mut message = b"file '".to_vec();
                        message.extend_from_slice(PATH);
                        message.extend_from_slice(b"' can't found!");
                        context.notify_reload_operator(b"ERROR", &message);
                        false
                    }
                };
                legacy_result = i32::from(succeeded);
                context.add_log_text(if succeeded {
                    b"Load hitlevel.ini...OK!"
                } else {
                    b"Load hitlevel.ini...FAILED!"
                });
                if succeeded && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.hit_level_setup
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::HitLevelSerialization)?;
                    self.send_reload_payload(0x14, &payload);
                }
            }
            WorldReloadProfile::Broadcast => {
                context.reload_broadcast_list(self);
            }
            WorldReloadProfile::AttackCity => {
                if context.call_boolean_owner(WorldReloadBooleanOwner::AttackCity) != 0 {
                    self.serialize_reload_owner(
                        context,
                        WorldReloadSerializationOwner::AttackCity,
                        0x1B,
                        true,
                        &mut legacy_result,
                    );
                    context.add_log_text(b"Load AttackCitySys List...OK!");
                }
            }
            WorldReloadProfile::InvalidStrings => {
                let filter_path = self.words_filter.filter_file_name().to_vec();
                let char_code_path = self.words_filter.char_code_file_name().to_vec();
                let filter_source = context.read_resource(&filter_path);
                let char_code_source = filter_source
                    .as_ref()
                    .and_then(|_| context.read_resource(&char_code_path));
                if self
                    .words_filter
                    .reload(filter_source.as_deref(), char_code_source.as_deref())
                {
                    context.add_log_text(b"Load InvalidStr...OK!");
                }
            }
            WorldReloadProfile::GeneralVariableList => {}
            WorldReloadProfile::FactionParameters => {
                context.call_void_owner(WorldReloadVoidOwner::LoadOrganizingParameters);
                context.call_void_owner(WorldReloadVoidOwner::ReinitializeFactionsByLevel);
                context.add_log_text(b"Load FactionPara...OK!");
            }
            WorldReloadProfile::VillageWar => {
                context.call_void_owner(WorldReloadVoidOwner::VillageWar);
                self.serialize_reload_owner(
                    context,
                    WorldReloadSerializationOwner::VillageWar,
                    0x1C,
                    true,
                    &mut legacy_result,
                );
                context.add_log_text(b"Load VilWarPara...OK!");
            }
            WorldReloadProfile::FourNationWar => {
                let succeeded =
                    context.call_boolean_owner(WorldReloadBooleanOwner::FourNationWar) != 0;
                if !succeeded || !send_to_game_servers {
                    context.add_log_text(b"Reload the time of FourNationWar...Fail! Please examine wheather did reloading operation when the war was still on!!");
                } else {
                    self.serialize_reload_owner(
                        context,
                        WorldReloadSerializationOwner::FourNationWar,
                        0x25,
                        true,
                        &mut legacy_result,
                    );
                    context.add_log_text(b"Reload file FourNationWarSys.ini...ok!");
                }
            }
            WorldReloadProfile::CityWar => {
                context.call_void_owner(WorldReloadVoidOwner::AttackCityUnchecked);
                self.serialize_reload_owner(
                    context,
                    WorldReloadSerializationOwner::AttackCity,
                    0x1B,
                    true,
                    &mut legacy_result,
                );
                context.add_log_text(b"Load CityWarPara...OK!");
            }
            WorldReloadProfile::FactionWar => {
                context.call_void_owner(WorldReloadVoidOwner::FactionWarParameters);
                context.add_log_text(b"Load FactionWarPara...OK!");
            }
            WorldReloadProfile::Quest => {
                context.call_void_owner(WorldReloadVoidOwner::Quest);
                context.add_log_text(b"Load QuestData...OK!");
                self.serialize_reload_owner(
                    context,
                    WorldReloadSerializationOwner::Quest,
                    0x16,
                    true,
                    &mut legacy_result,
                );
            }
            WorldReloadProfile::CountryParameters => {
                context.call_void_owner(WorldReloadVoidOwner::CountryParameters);
                context.add_log_text(if profile_name.eq_ignore_ascii_case(b"CountryParam") {
                    b"Load CountryParam...OK!"
                } else {
                    b"Load CountryPara...OK!"
                });
            }
            WorldReloadProfile::IncrementShop => {
                const PATH: &[u8] = b"setup/incrementshoplist.ini";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => {
                        let result = self.increment_shop_list.load_from_bytes(
                            &source,
                            &mut |query| match query {
                                IncrementShopGoodsQuery::OriginalName(name) => {
                                    IncrementShopGoodsResult::Id(
                                        context.query_goods_id_by_original_name(name),
                                    )
                                }
                                IncrementShopGoodsQuery::DisplayName(goods_id) => {
                                    IncrementShopGoodsResult::Name(
                                        context.query_goods_name(goods_id),
                                    )
                                }
                            },
                        );
                        match result {
                            Ok(report) => {
                                for warning in report.warnings {
                                    context.add_log_text(&warning);
                                }
                                true
                            }
                            Err(error) => {
                                let diagnostic = error.log_payload();
                                if !diagnostic.is_empty() {
                                    context.add_log_text(&diagnostic);
                                }
                                false
                            }
                        }
                    }
                    None => {
                        self.increment_shop_list.release();
                        let mut message = b"IncShopList : file '".to_vec();
                        message.extend_from_slice(PATH);
                        message.extend_from_slice(b"' can't found!");
                        context.add_log_text(&message);
                        false
                    }
                };
                legacy_result = i32::from(loaded);
                context.add_log_text(if loaded {
                    b"Load IncrementShopList...OK!"
                } else {
                    b"Load IncrementShopList...FAILED!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.increment_shop_list
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::IncrementShopSerialization)?;
                    self.send_reload_payload(4, &payload);
                }
            }
            WorldReloadProfile::Contribute => {
                const PATH: &[u8] = b"data/ContributeSetup.ini";
                let succeeded = match context.read_resource(PATH) {
                    Some(source) => self
                        .contribute_setup
                        .load_from_bytes(&source)
                        .map(|_| true)
                        .map_err(WorldReloadBlock::ContributeFormat)?,
                    None => {
                        self.contribute_setup.clear_items();
                        let mut message = b"file '".to_vec();
                        message.extend_from_slice(PATH);
                        message.extend_from_slice(b"' can't found!");
                        context.notify_reload_operator(b"ERROR", &message);
                        false
                    }
                };
                legacy_result = i32::from(succeeded);
                context.add_log_text(if succeeded {
                    b"Load ContributeSetup.ini...OK!"
                } else {
                    b"Load ContributeSetup.ini...FAILED!"
                });
                if succeeded && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.contribute_setup
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::ContributeSerialization)?;
                    self.send_reload_payload(5, &payload);
                }
            }
            WorldReloadProfile::Prison => {
                const PATH: &[u8] = b"data/PrisonConf.ini";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => self
                        .prison_conf
                        .load_from_bytes(&source)
                        .map(|_| true)
                        .map_err(WorldReloadBlock::PrisonFormat)?,
                    None => {
                        self.prison_conf.clear_prison_params();
                        let mut message = b"file '".to_vec();
                        message.extend_from_slice(PATH);
                        message.extend_from_slice(b"' can't found!");
                        context.notify_reload_operator(b"ERROR", &message);
                        false
                    }
                };
                legacy_result = i32::from(loaded);
                context.add_log_text(if loaded {
                    b"Load PrisonConf.ini...OK!"
                } else {
                    b"Load PrisonConf.ini...FAILED!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.prison_conf
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::PrisonSerialization)?;
                    self.send_reload_payload(0x1D, &payload);
                }
            }
            WorldReloadProfile::TimeToReturn => {
                let _ = Self::reload_boolean_with_log(
                    context,
                    WorldReloadBooleanOwner::TimeToReturn,
                    b"Load TimeToReturn...OK!",
                    b"Load TimeToReturn...FAILED!",
                );
            }
            WorldReloadProfile::PreciousBox => {
                self.reload_simple_serialized(
                    context,
                    WorldReloadBooleanOwner::PreciousBox,
                    WorldReloadSerializationOwner::PreciousBox,
                    0x1E,
                    b"Load PreciousBoxConf.xml...OK!",
                    b"Load PreciousBoxConf.xml...FAILED!",
                    send_to_game_servers,
                    true,
                    &mut legacy_result,
                );
            }
            WorldReloadProfile::FairyExp => {
                self.reload_simple_serialized(
                    context,
                    WorldReloadBooleanOwner::FairyExp,
                    WorldReloadSerializationOwner::FairyExp,
                    0x20,
                    b"Load FairyExp ....OK!",
                    b"Load FairyExp....failed!",
                    send_to_game_servers,
                    true,
                    &mut legacy_result,
                );
            }
            WorldReloadProfile::ChangeBody => {
                self.reload_simple_serialized(
                    context,
                    WorldReloadBooleanOwner::ChangeBody,
                    WorldReloadSerializationOwner::ChangeBody,
                    0x24,
                    b"Load CHBYRestrictionsGoods.xml...ok!",
                    b"Load CHBYRestrictionsGoods.xml...failed!",
                    send_to_game_servers,
                    true,
                    &mut legacy_result,
                );
            }
            WorldReloadProfile::CountryWar => {
                return Err(WorldReloadBlock::CountryWarOwnerRequired);
            }
            WorldReloadProfile::BattleFairyExp => {
                const PATH: &[u8] = b"BattleFairyReleate/BattleFairyExp.xml";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => match context.battle_fairy_exp_config().load_from_bytes(&source)
                    {
                        Ok(_) => true,
                        Err(error) => {
                            let diagnostic = self
                                .string_table
                                .table()
                                .get_string_by_id(error.string_id())
                                .map(ToOwned::to_owned)
                                .unwrap_or_default();
                            context.add_log_text(&diagnostic);
                            false
                        }
                    },
                    None => {
                        context.battle_fairy_exp_config().clear();
                        let diagnostic = self
                            .string_table
                            .table()
                            .get_string_by_id(b"ZHGS0029")
                            .map(ToOwned::to_owned)
                            .unwrap_or_default();
                        context.add_log_text(&diagnostic);
                        false
                    }
                };
                context.add_log_text(if loaded {
                    b"Add BattleFairyExpConfig.ini...ok!"
                } else {
                    b"Add BattleFairyExpConfig.ini...failed!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .battle_fairy_exp_config()
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::BattleFairyExpSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(0x2C, &payload);
                }
            }
            WorldReloadProfile::BattleFairyCombine => {
                const PATH: &[u8] = b"BattleFairyReleate/CombineConfig.ini";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => {
                        context.battle_fairy_property().load_combine_config(&source);
                        true
                    }
                    None => false,
                };
                context.add_log_text(if loaded {
                    b"Add BattleFairyCombineConfig.xml...ok!"
                } else {
                    b"Add BattleFairyCombineConfig.xml...failed!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .battle_fairy_property()
                        .serialize_combine(&mut payload)
                        .map_err(WorldReloadBlock::BattleFairyCombineSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(0x2D, &payload);
                }
            }
            WorldReloadProfile::Synthesis => {
                const PATH: &[u8] = b"data/synthesis.xml";
                // EXE очищает recipe-vector до rfOpen, но broadcast-map остаётся static.
                context.synthesis().clear_recipes();
                let loaded = match context.read_resource(PATH) {
                    Some(source) => {
                        // На время goods lookup owner извлечён безопасно: lookup идёт в
                        // тот же `WorldReloadContext`, а после точного loader-а state
                        // возвращается в его единственный runtime slot.
                        let mut synthesis = std::mem::take(context.synthesis());
                        let result = synthesis.load_from_bytes(
                            &source,
                            |original_name| {
                                let goods_id =
                                    context.query_goods_id_by_original_name(original_name);
                                let goods_name = (goods_id != 0)
                                    .then(|| context.query_goods_name(goods_id))
                                    .flatten();
                                (goods_id, goods_name)
                            },
                        );
                        *context.synthesis() = synthesis;
                        match result {
                            Ok(_) => true,
                            Err(error) => {
                                context.add_log_text(error.log_payload());
                                false
                            }
                        }
                    }
                    None => {
                        context.add_log_text(b"error: compose file is not exist!");
                        false
                    }
                };
                context.add_log_text(if loaded {
                    b"Load synthesis.xml...ok!"
                } else {
                    b"Load synthesis.xml...failed!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .synthesis()
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::SynthesisSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(0x21, &payload);
                }
            }
            WorldReloadProfile::DaKongXiangQian => {
                self.reload_simple_serialized(
                    context,
                    WorldReloadBooleanOwner::DaKongXiangQian,
                    WorldReloadSerializationOwner::DaKongXiangQian,
                    0x2B,
                    b"Load DaKongXiangQian.ini...ok!",
                    b"Load DaKongXiangQian.ini...failed!",
                    send_to_game_servers,
                    true,
                    &mut legacy_result,
                );
            }
            WorldReloadProfile::EquipmentCompose => {
                const PATH: &[u8] = b"data/EquipmentCompose.ini";
                let source = context.read_resource(PATH);
                let loaded = self.equipment_compose_list.load_list(source.as_deref());
                legacy_result = i32::from(loaded);
                context.add_log_text(if loaded {
                    b"Load EquipmentCompose.ini...ok!"
                } else {
                    b"Load EquipmentCompose.ini...failed!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.equipment_compose_list
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::EquipmentComposeSerialization)?;
                    self.send_reload_payload(0x30, &payload);
                }
            }
            WorldReloadProfile::GoodsDestroy => {
                const PATH: &[u8] = b"data/GoodsDestroyConf.ini";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => {
                        context
                            .goods_destroy_setup()
                            .load_from_bytes(&source)
                            .map_err(WorldReloadBlock::GoodsDestroyFormat)?;
                        true
                    }
                    None => {
                        context.goods_destroy_setup().clear_lists();
                        false
                    }
                };
                context.add_log_text(if loaded {
                    b"Load GoodsDestroyConf.ini...ok!"
                } else {
                    b"Load GoodsDestroyConf.ini...failed!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .goods_destroy_setup()
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::GoodsDestroySerialization)?;
                    // Dispatcher не переписывал legacy result для GoodsDestroy.
                    self.send_reload_payload(0x23, &payload);
                }
            }
            WorldReloadProfile::HonorEliminate => {
                const PATH: &[u8] = b"data/honorelimilate.ini";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => {
                        context.honor_eliminate_config().load_from_bytes(&source);
                        true
                    }
                    None => {
                        let mut message = b"file '".to_vec();
                        message.extend_from_slice(PATH);
                        message.extend_from_slice(b"' can't found!");
                        context.notify_reload_operator(b"error", &message);
                        false
                    }
                };
                context.add_log_text(if loaded {
                    b"Load HonorElimilate.ini Config...ok!"
                } else {
                    b"Load HonorElimilate.ini Config...failed!"
                });
            }
            WorldReloadProfile::TaoZhuang => {
                const PATH: &[u8] = b"data/taozhuang.ini";
                let source = context.read_resource(PATH);
                let succeeded = self
                    .tao_zhuang_setup
                    .read_file(source.as_deref(), |payload| context.add_log_text(payload));
                legacy_result = i32::from(succeeded);
                context.add_log_text(if succeeded {
                    b"Load TaoZhuang config...ok!"
                } else {
                    b"Load TaoZhuang config...failed!"
                });
                if succeeded && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.tao_zhuang_setup
                        .add_byte_to_array(&mut payload)
                        .map_err(WorldReloadBlock::TaoZhuangSerialization)?;
                    self.send_reload_payload(0x34, &payload);
                }
            }
            WorldReloadProfile::CiQing => {
                const PATH: &[u8] = b"/data/ciqing.ini";
                let source = context.read_resource(PATH);
                let succeeded = self.ci_qing_setup.read_setup_file(
                    source.as_deref(),
                    |original_name| context.query_goods_id_by_original_name(original_name),
                );
                context.add_log_text(if succeeded {
                    b"Add ciqing.ini...ok!"
                } else {
                    b"Add ciqing.ini...failed!"
                });
                legacy_result = i32::from(succeeded);
                context.call_void_owner(WorldReloadVoidOwner::LingBao);
                if succeeded && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.ci_qing_setup
                        .add_byte_to_array(&mut payload)
                        .map_err(WorldReloadBlock::CiQingSerialization)?;
                    payload.extend_from_slice(
                        &context.serialize_owner(WorldReloadSerializationOwner::LingBao),
                    );
                    self.send_reload_payload(0x35, &payload);
                }
            }
            WorldReloadProfile::Jjc => {
                let report = Self::load_jjc_configuration_from_resources(jjc, context);
                if let Some(path) = report.missing_path() {
                    context.notify_reload_operator(b"file not found", path);
                }
                context.add_log_text(if report.legacy_result() {
                    b"Load JJcConfig.ini...ok!"
                } else {
                    b"Load JJcCoinfig.ini...failed!"
                });
            }
            WorldReloadProfile::AllThing => {
                const PATH: &[u8] = b"/data/LeitingAction.ini";
                let loaded = if let Some(source) = context.read_resource(PATH) {
                    self.thing_setup
                        .load_all_thing_list(&source, PATH, |payload| {
                            context.add_log_text(payload)
                        })
                        .is_ok()
                } else {
                    self.thing_setup.clear_all_things_for_load();
                    let mut message = b"file '".to_vec();
                    message.extend_from_slice(PATH);
                    message.extend_from_slice(b"' can't found!");
                    context.notify_reload_operator(b"error", &message);
                    false
                };
                context.add_log_text(if loaded {
                    b"Load LeitingAction.ini...ok!"
                } else {
                    b"Load...failed!"
                });
                if loaded && send_to_game_servers {
                    let mut bytes = Vec::new();
                    self.thing_setup
                        .add_to_byte_array(&mut bytes)
                        .map_err(WorldReloadBlock::ThingSetupCodec)?;
                    self.send_reload_payload(0x36, &bytes);
                }
            }
            WorldReloadProfile::GodsBattle => {
                let succeeded = Self::reload_boolean_with_log(
                    context,
                    WorldReloadBooleanOwner::GodsBattle,
                    b"Load Gods-Battle...ok!",
                    b"Load Gods-Battle...failed!",
                );
                context.call_void_owner(WorldReloadVoidOwner::GodsBattleNpcFaction);
                if succeeded && send_to_game_servers {
                    self.serialize_reload_owner(
                        context,
                        WorldReloadSerializationOwner::GodsBattle,
                        0x39,
                        false,
                        &mut legacy_result,
                    );
                }
            }
        }
        Ok(legacy_result)
    }

    #[allow(clippy::too_many_arguments)]
    fn reload_simple_serialized<Context: WorldReloadContext + ?Sized>(
        &self,
        context: &mut Context,
        load_owner: WorldReloadBooleanOwner,
        serialization_owner: WorldReloadSerializationOwner,
        subcode: i32,
        success_log: &[u8],
        failure_log: &[u8],
        send_to_game_servers: bool,
        update_legacy_result: bool,
        legacy_result: &mut i32,
    ) {
        if Self::reload_boolean_with_log(context, load_owner, success_log, failure_log)
            && send_to_game_servers
        {
            self.serialize_reload_owner(
                context,
                serialization_owner,
                subcode,
                update_legacy_result,
                legacy_result,
            );
        }
    }

    fn send_script_reload_data(&self) {
        let sender = self.current_game_server_sender();
        for (subcode, data) in [
            (0x0A, self.function_list_file_data.as_deref()),
            (0x0B, self.variable_list_file_data.as_deref()),
        ] {
            let Some(data) = data else { continue };
            let data = legacy_c_string_prefix(data);
            let mut message = CMessage::new(0x0007_F801);
            message.base_mut().add_long(subcode);
            message.base_mut().add_long(data.len() as u32 as i32);
            add_legacy_c_string(message.base_mut(), data);
            let _ = message.send_all(sender.as_ref());
        }
        for (path, data) in &self.script_file_data {
            let data = legacy_c_string_prefix(data);
            let mut message = CMessage::new(0x0007_F801);
            message.base_mut().add_long(0x0D);
            add_legacy_c_string(message.base_mut(), path);
            message.base_mut().add_long(data.len() as u32 as i32);
            add_legacy_c_string(message.base_mut(), data);
            let _ = message.send_all(sender.as_ref());
        }
    }

    fn send_loaded_regions(
        &mut self,
        legacy_result: &mut i32,
    ) -> Result<(), WorldReloadRegionSnapshotBlock> {
        let sender = self.current_game_server_sender();
        for assignment in self.regions.values_mut() {
            let Some(region) = assignment.region.as_mut() else {
                continue;
            };
            let region_id = region.base().get_id();
            let mut bytes = Vec::new();
            match region {
                WorldRegionOwner::Base(region) => {
                    let _ = region
                        .add_to_byte_array(&mut bytes, true)
                        .map_err(|source| WorldReloadRegionSnapshotBlock {
                            region_id,
                            source: WorldRegionOwnerSerializationBlock::Base(source),
                        })?;
                }
                WorldRegionOwner::Village(region) => {
                    let _ = region
                        .add_to_byte_array(&mut bytes, true)
                        .map_err(|source| WorldReloadRegionSnapshotBlock {
                            region_id,
                            source: WorldRegionOwnerSerializationBlock::Village(source),
                        })?;
                }
                WorldRegionOwner::City(region) => {
                    let _ = region
                        .add_to_byte_array(&mut bytes, true)
                        .map_err(|source| WorldReloadRegionSnapshotBlock {
                            region_id,
                            source: WorldRegionOwnerSerializationBlock::City(source),
                        })?;
                }
                WorldRegionOwner::Country(region) => {
                    let _ = region
                        .add_to_byte_array(&mut bytes, true)
                        .map_err(|source| WorldReloadRegionSnapshotBlock {
                            region_id,
                            source: WorldRegionOwnerSerializationBlock::Country(source),
                        })?;
                }
            }
            *legacy_result = bytes.len() as u32 as i32;
            let mut message = CMessage::new(0x0007_F801);
            message.base_mut().add_long(0x0E);
            message
                .base_mut()
                .add_long(assignment.region_type.unwrap_or_default());
            message.base_mut().add(&bytes);
            let _ = message.send_to_map_id(sender.as_ref(), assignment.game_server_index as i32);
        }
        Ok(())
    }

    /// Открывает frozen DB snapshot только внутри внешне сериализованного save.
    pub(crate) fn db_data_save_session(&mut self) -> WorldDbDataSaveSession<'_> {
        WorldDbDataSaveSession {
            data: self.db_data.get_mut(),
        }
    }

    /// Применяет два constructor-load результата `CRsSetup` к live `CGame`.
    pub(crate) const fn apply_loaded_setup_ids(&mut self, loaded: LoadedSetupIds) {
        self.player_id = Some(loaded.player_id);
        self.leave_word_id = Some(loaded.leave_world_id);
    }

    /// Выделяет следующий signed leave-word ID с точным x86 wrapping.
    pub(crate) fn allocate_leave_word_id(&mut self) -> Result<i32, WorldLeaveWordIdBlock> {
        let leave_word_id = self
            .leave_word_id
            .as_mut()
            .ok_or(WorldLeaveWordIdBlock)?;
        *leave_word_id = leave_word_id.wrapping_add(1);
        Ok(*leave_word_id)
    }

    /// Выполняет точный `++m_nPlayerID` create-role ветки с x86 wrapping.
    pub(crate) fn allocate_player_id(&mut self) -> Result<i32, WorldPlayerIdBlock> {
        let player_id = self.player_id.as_mut().ok_or(WorldPlayerIdBlock)?;
        *player_id = player_id.wrapping_add(1);
        Ok(*player_id as i32)
    }

    /// Полностью очищает live restore-list.
    pub(crate) fn clear_restore_player(&mut self) {
        self.restore_players.clear();
    }

    /// Полностью очищает live deletion-list.
    pub(crate) fn clear_deletion_player(&mut self) {
        self.deletion_players.clear();
    }

    /// Удаляет player-map owners, отсутствующие и в online-, и в login-list.
    pub(crate) fn clear_map_player_for_offline(&mut self) {
        let online_players = &self.online_players;
        let login_players = &self.login_players;
        self.players.retain(|player_id, _| {
            online_players.contains(player_id)
                || login_players
                    .iter()
                    .any(|entry| entry.player_id == *player_id)
        });
    }

    /// Удаляет только первое совпадение из live restore-list.
    pub(crate) fn delete_restore_player(&mut self, player_id: u32) {
        if let Some(index) = self
            .restore_players
            .iter()
            .position(|existing| *existing == player_id)
        {
            self.restore_players.remove(index);
        }
    }

    /// Проверяет наличие ID в live restore-list.
    pub(crate) fn is_restore_player_exist(&self, player_id: u32) -> bool {
        self.restore_players.contains(&player_id)
    }

    /// Возвращает время первого совпавшего deletion-record либо исходный ноль.
    pub(crate) fn deletion_player_time(&self, player_id: u32) -> i32 {
        self.deletion_players
            .iter()
            .find(|entry| entry.player_id == player_id)
            .map_or(0, |entry| entry.deletion_time)
    }

    /// Удаляет только первое совпадение из live deletion-list.
    pub(crate) fn delete_deletion_player(&mut self, player_id: u32) {
        if let Some(index) = self
            .deletion_players
            .iter()
            .position(|entry| entry.player_id == player_id)
        {
            self.deletion_players.remove(index);
        }
    }

    /// Добавляет уникальный ID в хвост live restore-list.
    pub(crate) fn append_restore_player(&mut self, player_id: u32) {
        if !self.restore_players.contains(&player_id) {
            self.restore_players.push_back(player_id);
        }
    }

    /// Добавляет первый deletion-record ID и не меняет его при duplicate.
    pub(crate) fn append_deletion_player(&mut self, player_id: u32, deletion_time: i32) {
        if self
            .deletion_players
            .iter()
            .any(|entry| entry.player_id == player_id)
        {
            return;
        }
        self.deletion_players.push_back(DeletionPlayerSnapshot {
            player_id,
            deletion_time,
        });
    }

    /// Создаёт byte-array копию player-map owner-а либо возвращает `None` при miss.
    ///
    /// Encoder mutates исходный player в доказанных `SetPlayerOrganizing` и
    /// `UpdateProperty`; decoder начинает с нулевого cursor и `include_child=true`.
    /// Его `false` уничтожает новую копию, как virtual deleting destructor старого
    /// owner-а. Typed codec-error останавливает только недоказанную safe-границу.
    pub(crate) fn clone_map_player(
        &mut self,
        player_id: u32,
        registry: &GoodsBasePropertiesRegistry,
        organizing_ctrl: &COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<Box<CPlayer>>, PlayerCodecError> {
        let region_types = self.player_organizing_region_types();
        let Some(source) = self.players.get_mut(&player_id) else {
            return Ok(None);
        };

        let mut cloned = CPlayer::with_clone_decode_constructor_state();
        let mut wire = Vec::new();
        let mut updater = organizing_ctrl.player_updater(&region_types);
        let _ = source.add_to_byte_array(&mut wire, true, registry, &mut updater, coefficients)?;
        let mut cursor = 0;
        if !cloned.decord_from_byte_array(&wire, &mut cursor, true, registry, coefficients)? {
            return Ok(None);
        }
        Ok(Some(Box::new(cloned)))
    }

    /// Клонирует map-owner только если ID ещё состоит в creation-list.
    ///
    /// Exact caller сначала линейно проходил весь `m_lCreationPlayer`, а при
    /// первом совпадении без дополнительной мутации вызывал `CloneMapPlayer`.
    pub(crate) fn clone_creation_player(
        &mut self,
        player_id: u32,
        registry: &GoodsBasePropertiesRegistry,
        organizing_ctrl: &COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<Box<CPlayer>>, PlayerCodecError> {
        if !self
            .creation_players
            .iter()
            .any(|creation_id| *creation_id as u32 == player_id)
        {
            return Ok(None);
        }
        self.clone_map_player(player_id, registry, organizing_ctrl, coefficients)
    }

    /// Уничтожает единственного player-owner-а по exact unsigned map-key.
    ///
    /// `Box`/`BTreeMap::remove` заменяют virtual deleting destructor и erase;
    /// bool сообщает caller-у только наблюдаемый факт наличия, которого старый
    /// void API наружу не выдавал.
    pub(crate) fn delete_map_player(&mut self, player_id: u32) -> bool {
        self.players.remove(&player_id).is_some()
    }

    /// Повторяет `CloneSavingPlayer` под save-list lock через тот же clone-codec.
    pub(crate) fn clone_saving_player(
        &self,
        player_id: u32,
        registry: &GoodsBasePropertiesRegistry,
        organizing_ctrl: &COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<Box<CPlayer>>, PlayerCodecError> {
        let region_types = self.player_organizing_region_types();
        let mut db_data = self.db_data.lock();
        let Some(source) = db_data.players.get_mut(&player_id) else {
            return Ok(None);
        };

        let mut cloned = CPlayer::with_clone_decode_constructor_state();
        let mut wire = Vec::new();
        let mut updater = organizing_ctrl.player_updater(&region_types);
        let _ = source.add_to_byte_array(&mut wire, true, registry, &mut updater, coefficients)?;
        let mut cursor = 0;
        if !cloned.decord_from_byte_array(&wire, &mut cursor, true, registry, coefficients)? {
            return Ok(None);
        }
        Ok(Some(Box::new(cloned)))
    }

    /// Материализует player-prefix `GenerateDBData` до доменных generators.
    ///
    /// Restore-копирование выполняется через эксклюзивный `&mut self` без lock;
    /// deletion и обе player-очереди используют исходную save-блокировку.
    pub(crate) fn generate_db_data_player_prefix(
        &mut self,
        registry: &GoodsBasePropertiesRegistry,
        organizing_ctrl: &COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<(), WorldGenerateDbDataBlock> {
        let leave_word_id = self
            .leave_word_id
            .ok_or(WorldGenerateDbDataBlock::UninitializedLeaveWordId)?;
        let player_id = self
            .player_id
            .ok_or(WorldGenerateDbDataBlock::UninitializedPlayerId)?;

        let db_data = self.db_data.get_mut();
        db_data.player_id = Some(player_id);
        db_data.leave_word_id = Some(leave_word_id);

        let creation_ids = self
            .creation_players
            .iter()
            .map(|player_id| *player_id as u32)
            .collect::<Vec<_>>();
        for player_id in creation_ids {
            if let Some(player) =
                self.clone_creation_player(player_id, registry, organizing_ctrl, coefficients)?
            {
                self.append_db_creation_player(player);
            }
        }

        self.db_data
            .get_mut()
            .restore_players
            .extend(self.restore_players.iter().copied());

        for entry in self.deletion_players.iter().copied() {
            self.db_data.lock().deletion_players.push_back(entry);
        }

        let player_ids = self.players.keys().copied().collect::<Vec<_>>();
        for player_id in player_ids {
            if let Some(player) =
                self.clone_map_player(player_id, registry, organizing_ctrl, coefficients)?
            {
                self.append_db_player(player);
            }
        }

        Ok(())
    }

    /// Материализует полный DB snapshot в исходном порядке владельцев.
    ///
    /// Функция ничего не очищает в live player-list после snapshot: это
    /// отдельные операции caller-а, следующие за `GenerateDBData`.
    #[allow(
        clippy::too_many_arguments,
        reason = "семь прежних singleton/static зависимостей передаются явно без нового общего owner-а"
    )]
    pub(crate) fn generate_db_data(
        &mut self,
        registry: &GoodsBasePropertiesRegistry,
        organizing_ctrl: &mut COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
        faction_war_sys: &CFactionWarSys,
        country_handler: &CCountryHandler,
        country_limits: CountryKingSaveLimits,
        honor_ranks: &mut CHonorRanks,
    ) -> Result<WorldGenerateDbDataReport, WorldGenerateDbDataBlock> {
        self.generate_db_data_player_prefix(registry, organizing_ctrl, coefficients)?;
        let organizing = organizing_ctrl.generate_save_data(self, false)?;
        faction_war_sys.generate_save_data(self);
        self.geterate_region_db_data();
        country_handler.generate_save_data(self, country_limits);
        honor_ranks.generate_save_data();

        Ok(WorldGenerateDbDataReport { organizing })
    }

    /// Материализует отдельную `g_bSaveAllOrg` ветвь `CGame::Run`.
    ///
    /// Она не вызывает player-prefix и Country generator: исходник выполнял
    /// только organizing, faction-war, region и HonorRanks перед тем же launch.
    pub(crate) fn materialize_save_all_organizations_snapshot<LaunchSaveThread>(
        &mut self,
        organizing_ctrl: &mut COrganizingCtrl,
        faction_war_sys: &CFactionWarSys,
        honor_ranks: &mut CHonorRanks,
        save_thread_handle: &mut WorldSaveThreadHandleState,
        launch_save_thread: &mut LaunchSaveThread,
    ) -> Result<WorldSaveAllOrganizationsLaunchReport, OrganizingSaveDataBlock>
    where
        LaunchSaveThread: FnMut(&WorldSaveThreadLaunchRequest) -> WorldSaveThreadHandleState,
    {
        let organizing = organizing_ctrl.generate_save_data(self, false)?;
        faction_war_sys.generate_save_data(self);
        self.geterate_region_db_data();
        honor_ranks.generate_save_data();
        let launch = prepare_save_thread_launch(save_thread_handle);
        let resulting_handle = launch_save_thread(&launch);
        *save_thread_handle = resulting_handle;
        Ok(WorldSaveAllOrganizationsLaunchReport {
            organizing,
            launch,
            resulting_handle,
        })
    }

    /// Выполняет snapshot и live-cleanup save-trigger ветви `CGame::Run`.
    ///
    /// Только после snapshot/cleanup закрывает прежний handle-state, вызывает
    /// внешний launcher и сохраняет возвращённое opaque `Open/Empty` состояние.
    #[allow(
        clippy::too_many_arguments,
        reason = "исходный Run обращался к тем же семи singleton/static зависимостям"
    )]
    pub(crate) fn materialize_run_save_snapshot<LaunchSaveThread>(
        &mut self,
        registry: &GoodsBasePropertiesRegistry,
        organizing_ctrl: &mut COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
        faction_war_sys: &CFactionWarSys,
        country_handler: &CCountryHandler,
        country_limits: CountryKingSaveLimits,
        honor_ranks: &mut CHonorRanks,
        save_thread_handle: &mut WorldSaveThreadHandleState,
        launch_save_thread: &mut LaunchSaveThread,
    ) -> Result<WorldRunSaveLaunchReport, WorldGenerateDbDataBlock>
    where
        LaunchSaveThread: FnMut(&WorldSaveThreadLaunchRequest) -> WorldSaveThreadHandleState,
    {
        let snapshot = self.generate_db_data(
            registry,
            organizing_ctrl,
            coefficients,
            faction_war_sys,
            country_handler,
            country_limits,
            honor_ranks,
        )?;
        self.clear_map_player_for_offline();
        self.clear_restore_player();
        self.clear_creation_player();
        self.clear_deletion_player();
        self.clear_offline_player();
        let launch = prepare_save_thread_launch(save_thread_handle);
        let resulting_handle = launch_save_thread(&launch);
        *save_thread_handle = resulting_handle;
        Ok(WorldRunSaveLaunchReport {
            snapshot,
            launch,
            resulting_handle,
        })
    }

    /// Исполняет ручную collect-player-data ветвь `CGame::Run`.
    ///
    /// Флаг очищается до создания пустого broadcast; исходно игнорировавшийся
    /// результат `SendAll` сохраняется только как наблюдаемый отчёт.
    pub(crate) fn materialize_collect_player_data_request(
        &self,
        state: &mut WorldCollectPlayerDataRequestState,
    ) -> Option<WorldCollectPlayerDataBroadcast> {
        if !state.send_now {
            return None;
        }

        state.send_now = false;
        let message_type = 0x0007_F808;
        let message = CMessage::new(message_type);
        let sender = self.current_game_server_sender();
        let delivery = message.send_all(sender.as_ref());
        Some(WorldCollectPlayerDataBroadcast {
            message_type,
            delivery,
        })
    }

    /// Выполняет точный pre-gate save-участка `CGame::Run`.
    ///
    /// `try_enter` вызывается только после строгого прохождения wrapping-
    /// интервала. Его `false` соответствует занятому critical section и
    /// сдвигает прежний save tick на исходные `1000` миллисекунд.
    #[allow(
        clippy::too_many_arguments,
        reason = "Run обращался к тем же process-global и singleton владельцам"
    )]
    pub(crate) fn materialize_run_save_pre_gate<
        'game,
        TryEnter,
        GetSavePointTime,
        GetTick,
        GetLocalTime,
        PutLogInfo,
        LaunchSaveThread,
    >(
        &'game mut self,
        state: &mut WorldRunSaveTriggerState,
        now_ms: u32,
        get_save_point_time: &mut GetSavePointTime,
        try_enter: TryEnter,
        registry: &GoodsBasePropertiesRegistry,
        organizing_ctrl: &mut COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
        faction_war_sys: &CFactionWarSys,
        country_handler: &CCountryHandler,
        country_limits: CountryKingSaveLimits,
        honor_ranks: &mut CHonorRanks,
        save_thread_handle: &mut WorldSaveThreadHandleState,
        log: &mut WorldLogTextOwner,
        get_tick: &mut GetTick,
        get_local_time: &mut GetLocalTime,
        put_log_info: &mut PutLogInfo,
        launch_save_thread: &mut LaunchSaveThread,
    ) -> WorldRunSavePreGateReport<'game>
    where
        TryEnter: FnOnce() -> bool,
        GetSavePointTime: FnMut() -> u32,
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
        LaunchSaveThread: FnMut(&WorldSaveThreadLaunchRequest) -> WorldSaveThreadHandleState,
    {
        let manual_request = if state.send_save_message_now {
            let log = log.add_log_text(
                b"Manual Send Save Request!",
                self.setup.save_info_time_ms,
                &mut *get_tick,
                &mut *get_local_time,
                &mut *put_log_info,
            );
            let save_point_time_ms = get_save_point_time();
            state.last_save_point_time_ms = now_ms.wrapping_sub(save_point_time_ms);
            state.send_save_message_now = false;
            Some(WorldManualSaveRequestReport {
                log,
                save_point_time_ms,
            })
        } else {
            None
        };

        let profile_started_at_ms = get_tick();
        let elapsed_ms = now_ms.wrapping_sub(state.last_save_point_time_ms);
        let save_point_time_ms = get_save_point_time();
        if elapsed_ms <= save_point_time_ms {
            return WorldRunSavePreGateReport::IntervalNotElapsed {
                manual_request,
                profile_started_at_ms,
                elapsed_ms,
                save_point_time_ms,
            };
        }

        if !try_enter() {
            state.last_save_point_time_ms = state.last_save_point_time_ms.wrapping_add(1_000);
            return WorldRunSavePreGateReport::SaveLockBusy {
                manual_request,
                profile_started_at_ms,
                elapsed_ms,
                save_point_time_ms,
                adjusted_last_save_point_time_ms: state.last_save_point_time_ms,
            };
        }

        let trigger = self.materialize_run_save_trigger_after_lock(
            state,
            now_ms,
            registry,
            organizing_ctrl,
            coefficients,
            faction_war_sys,
            country_handler,
            country_limits,
            honor_ranks,
            save_thread_handle,
            log,
            get_tick,
            get_local_time,
            put_log_info,
            launch_save_thread,
        );
        WorldRunSavePreGateReport::AfterLock {
            manual_request,
            profile_started_at_ms,
            elapsed_ms,
            save_point_time_ms,
            trigger,
        }
    }

    /// Выполняет достигнутое save-решение `CGame::Run` после успешного try-lock.
    ///
    /// При manual-save и живых GameServer исходник сначала строит локальный
    /// snapshot/launch, затем повторно считает подключения и рассылает notify.
    /// Blocked generator сохраняет guard: исходный невозвратившийся путь не
    /// достигает ни второй проверки, ни `LeaveCriticalSection`.
    #[allow(
        clippy::too_many_arguments,
        reason = "Run обращался к тем же process-global и singleton владельцам"
    )]
    pub(crate) fn materialize_run_save_trigger_after_lock<
        'game,
        GetTick,
        GetLocalTime,
        PutLogInfo,
        LaunchSaveThread,
    >(
        &'game mut self,
        state: &mut WorldRunSaveTriggerState,
        now_ms: u32,
        registry: &GoodsBasePropertiesRegistry,
        organizing_ctrl: &mut COrganizingCtrl,
        coefficients: &PlayerPropertyCoefficients,
        faction_war_sys: &CFactionWarSys,
        country_handler: &CCountryHandler,
        country_limits: CountryKingSaveLimits,
        honor_ranks: &mut CHonorRanks,
        save_thread_handle: &mut WorldSaveThreadHandleState,
        log: &mut WorldLogTextOwner,
        get_tick: &mut GetTick,
        get_local_time: &mut GetLocalTime,
        put_log_info: &mut PutLogInfo,
        launch_save_thread: &mut LaunchSaveThread,
    ) -> WorldRunSaveTriggerReport<'game>
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
        LaunchSaveThread: FnMut(&WorldSaveThreadLaunchRequest) -> WorldSaveThreadHandleState,
    {
        let save_info_time_ms = self.setup.save_info_time_ms;
        let guard = WorldSaveThreadGuard { game: self };

        if state.save_all_organizations {
            state.save_all_organizations = false;
            state.last_save_point_time_ms = now_ms;
            let save = match guard.game.materialize_save_all_organizations_snapshot(
                organizing_ctrl,
                faction_war_sys,
                honor_ranks,
                save_thread_handle,
                launch_save_thread,
            ) {
                Ok(save) => save,
                Err(block) => {
                    return WorldRunSaveTriggerReport::BlockedSaveAllOrganizations { guard, block };
                }
            };
            guard.release();
            return WorldRunSaveTriggerReport::Complete(
                WorldRunSaveTriggerDisposition::SaveAllOrganizations(save),
            );
        }

        let immediate = if state.save_now_data || guard.game.connected_game_server_count() < 1 {
            let log = log.add_log_text(
                b"Manual Saveing Player Data Now...",
                save_info_time_ms,
                &mut *get_tick,
                &mut *get_local_time,
                &mut *put_log_info,
            );
            state.last_save_point_time_ms = now_ms;
            state.save_now_data = false;
            let save = match guard.game.materialize_run_save_snapshot(
                registry,
                organizing_ctrl,
                coefficients,
                faction_war_sys,
                country_handler,
                country_limits,
                honor_ranks,
                save_thread_handle,
                launch_save_thread,
            ) {
                Ok(save) => save,
                Err(block) => {
                    return WorldRunSaveTriggerReport::BlockedImmediateSave { guard, log, block };
                }
            };
            Some(WorldRunImmediateSaveReport { log, save })
        } else {
            None
        };

        let notify = if guard.game.connected_game_server_count() > 0 {
            let log = log.add_log_text(
                b"Send SaveNotify to all GameServer...",
                save_info_time_ms,
                &mut *get_tick,
                &mut *get_local_time,
                &mut *put_log_info,
            );
            state.last_save_point_time_ms = now_ms;
            let previous_db_responses = std::mem::replace(&mut guard.game.db_responses, 0);
            let message = CMessage::new(0x0007_F803);
            let sender = guard.game.current_game_server_sender();
            let deliveries = guard
                .game
                .game_servers
                .values()
                .filter(|game_server| game_server.connected)
                .map(|game_server| WorldSaveNotifyDelivery {
                    game_server_index: game_server.index,
                    delivery: message.send_to_map_id(sender.as_ref(), game_server.index as i32),
                })
                .collect();
            Some(WorldSaveNotifyReport {
                log,
                previous_db_responses,
                message_type: message.message_type(),
                deliveries,
            })
        } else {
            None
        };

        guard.release();
        WorldRunSaveTriggerReport::Complete(WorldRunSaveTriggerDisposition::PlayerData {
            immediate,
            notify,
        })
    }

    /// Материализует region-очередь `CGame::tagDBData` в signed map-order.
    pub(crate) fn geterate_region_db_data(&self) {
        for assignment in self.regions.values() {
            let Some(region) = assignment.region.as_ref().map(WorldRegionOwner::base) else {
                continue;
            };
            self.append_region_param(region.generate_save_data());
        }
    }

    /// Снимает только наблюдаемую SetPlayerOrganizing-проекцию region-map.
    fn player_organizing_region_types(&self) -> BTreeMap<i32, Option<u16>> {
        self.regions
            .iter()
            .filter_map(|(&region_id, assignment)| {
                assignment.region.as_ref()?;
                Some((
                    region_id,
                    assignment.region_type.map(|region_type| region_type as u16),
                ))
            })
            .collect()
    }

    /// Добавляет save-копию в `m_stDBData.liDBCreationPlayer`.
    ///
    /// При первом совпадении inherited ID старая копия уничтожается и
    /// удаляется, после чего новая всегда дописывается в хвост под тем же lock.
    pub(crate) fn append_db_creation_player(&self, player: Box<CPlayer>) {
        let player_id = player.get_id();
        let mut db_data = self.db_data.lock();

        if let Some(index) = db_data
            .creation_players
            .iter()
            .position(|existing| existing.get_id() == player_id)
        {
            drop(db_data.creation_players.remove(index));
        }
        db_data.creation_players.push_back(player);
    }

    /// Вставляет save-копию в `m_stDBData.mDBPlayer` по unsigned ID.
    ///
    /// Существующая копия уничтожается до вставки новой и всё изменение
    /// остаётся внутри исходной critical-section границы.
    pub(crate) fn append_db_player(&self, player: Box<CPlayer>) {
        let player_id = player.get_id() as u32;
        let mut db_data = self.db_data.lock();

        if let Some(previous) = db_data.players.remove(&player_id) {
            drop(previous);
        }
        db_data.players.insert(player_id, player);
    }

    /// Дописывает non-null save-копию в `m_stDBData.listSaveFactions`.
    pub(crate) fn append_save_faction(&self, faction: Box<CFaction>) {
        self.db_data.lock().save_factions.push_back(faction);
    }

    /// Дописывает non-null save-копию в `m_stDBData.listSaveUnions`.
    pub(crate) fn append_save_union(&self, union: Box<CUnion>) {
        self.db_data.lock().save_unions.push_back(union);
    }

    /// Дописывает signed ID в `m_stDBData.listDeleteFactions`.
    pub(crate) fn append_delete_faction(&self, faction_id: i32) {
        self.db_data.lock().delete_factions.push_back(faction_id);
    }

    /// Дописывает signed ID в `m_stDBData.listDeleteUnions`.
    pub(crate) fn append_delete_union(&self, union_id: i32) {
        self.db_data.lock().delete_unions.push_back(union_id);
    }

    /// Дописывает материализованную non-null region save-копию.
    pub(crate) fn append_region_param(&self, region: RegionSaveSnapshot) {
        self.db_data.lock().regions.push_back(Some(region));
    }

    /// Дописывает non-null country save-копию в `m_stDBData.ltDBCountrys`.
    pub(crate) fn append_db_country(&self, country: CountrySaveSnapshot) {
        self.db_data.lock().countries.push_back(Some(country));
    }

    /// Заменяет весь `m_stDBData.listEnemyFactions` под save-lock.
    ///
    /// Вход уже владеет отдельными копиями. `Option` сохраняет допустимый
    /// null pointer-list элемент, хотя готовый generator создаёт только
    /// non-null записи.
    pub(crate) fn set_enemy_factions(
        &self,
        enemy_factions: VecDeque<Option<EnemyFactionSaveSnapshot>>,
    ) {
        let mut db_data = self.db_data.lock();
        db_data.enemy_factions = enemy_factions;
    }

    /// Выполняет полный достигнутый `CGame::ClearDBData` под одним lock.
    ///
    /// Scalar ID исходная функция не сбрасывала. Region-часть удаляет только
    /// nodes: их non-null save-копии обязан уничтожить предшествующий save.
    pub(crate) fn clear_db_data(&self) {
        let mut db_data = self.db_data.lock();

        while let Some(player) = db_data.creation_players.pop_front() {
            drop(player);
        }
        db_data.restore_players.clear();
        db_data.deletion_players.clear();
        while let Some((_player_id, player)) = db_data.players.pop_first() {
            drop(player);
        }
        while let Some(faction) = db_data.save_factions.pop_front() {
            drop(faction);
        }
        while let Some(union) = db_data.save_unions.pop_front() {
            drop(union);
        }
        db_data.delete_factions.clear();
        db_data.delete_unions.clear();
        // WorldServer RVA 0x0000D490 не вызывает virtual destructor здесь:
        // save-фаза уничтожает value, сохраняя node до этой общей очистки.
        db_data.regions.clear();
    }

    /// Удваивает одинарные кавычки как исходный `CheckPoint`.
    ///
    /// Вход уже является доказанным видимым C-string prefix; отсутствие NUL в
    /// конкретном fixed field проверяет его владелец до этого вызова.
    pub(crate) fn check_point(input: &[u8]) -> Result<Vec<u8>, WorldCheckPointBlock> {
        let escaped_length = input
            .len()
            .saturating_add(input.iter().filter(|byte| **byte == b'\'').count());
        if escaped_length >= 256 {
            // WorldServer RVA 0x00009A90, exact 0x00409C5B/0x00409CBB:
            // `__snprintf(local, 0x100, "%s", escaped); return local;`.
            // При >=256 байтах NUL отсутствовал, а caller читал за уже мёртвым
            // stack-buffer; результат этого пути не назначается по догадке.
            return Err(WorldCheckPointBlock {
                input_length: input.len(),
                escaped_length,
            });
        }

        let mut escaped = Vec::with_capacity(escaped_length);
        for byte in input {
            escaped.push(*byte);
            if *byte == b'\'' {
                escaped.push(*byte);
            }
        }
        Ok(escaped)
    }

    /// Позиционно читает `setup.ini`, а при ошибке открытия — `setup.dat`.
    ///
    /// Успешное открытие остаётся успешной загрузкой даже после stream
    /// fail-state. Метод не запускает сервисы и не публикует значения файла.
    pub(crate) fn load_setup<ClaimSingleInstance>(
        &mut self,
        runtime_directory: &Path,
        mut claim_single_instance: ClaimSingleInstance,
    ) -> Result<WorldSetupLoadReport, WorldSetupOpenError>
    where
        ClaimSingleInstance: FnMut(&[u8]) -> bool,
    {
        let plain_path = runtime_directory.join("setup.ini");
        let encoded_path = runtime_directory.join("setup.dat");

        let (source, parsed_pairs, stopped_at_pair) = match fs::read(&plain_path) {
            Ok(bytes) => {
                let (parsed, stopped) = self.setup.parse_plain(&bytes);
                (WorldSetupSource::Plain, parsed, stopped)
            }
            Err(plain) => match fs::read(&encoded_path) {
                Ok(bytes) => {
                    let decoded = ini_decode(&bytes);
                    let c_string_len = decoded
                        .iter()
                        .position(|byte| *byte == 0)
                        .unwrap_or(decoded.len());
                    let (parsed, stopped) = self.setup.parse_encoded(&decoded[..c_string_len]);
                    (WorldSetupSource::Encoded, parsed, stopped)
                }
                Err(encoded) => {
                    return Err(WorldSetupOpenError {
                        plain_path,
                        plain,
                        encoded_path,
                        encoded,
                    });
                }
            },
        };

        let mut instance_title = b"WorldServer[".to_vec();
        instance_title.extend_from_slice(&self.setup.name);
        instance_title.push(b']');
        if source == WorldSetupSource::Plain {
            instance_title.extend_from_slice(b"-Saga3D2");
        }
        let instance_claimed = claim_single_instance(&instance_title);

        Ok(WorldSetupLoadReport {
            source,
            parsed_pairs,
            stopped_at_pair,
            instance_title,
            instance_claimed,
        })
    }

    /// Читает `serverSetup.ini`, не очищая прежний GameServer registry.
    ///
    /// Ошибка открытия не меняет map и соответствует старому `false`.
    /// Успешное открытие сохраняет legacy-успех даже после stream fail-state;
    /// безопасная граница останавливает только запись с неизвестным первым ID.
    pub(crate) fn load_server_setup(
        &mut self,
        runtime_directory: &Path,
    ) -> Result<WorldServerSetupLoadReport, io::Error> {
        let path = resolve_world_runtime_file(runtime_directory, "serverSetup.ini")?;
        let bytes = fs::read(path)?;
        let mut tokens = WorldServerSetupTokens::new(&bytes);

        let _header = tokens.next_bytes();
        let declared_records = tokens.next_ascii::<i32>().unwrap_or(0);
        let mut current_index = None;
        let mut current_ip = Vec::new();
        let mut current_port = None;
        let mut applied_records = 0;
        let mut blocked_at_record = None;

        for record_index in 0..declared_records {
            let _marker_found = tokens.seek_to(b"#");
            if let Some(index) = tokens.next_ascii::<u32>() {
                current_index = Some(index);
            }
            if let Some(ip) = tokens.next_bytes() {
                current_ip = ip.to_vec();
            }
            if let Some(port) = tokens.next_ascii::<u32>() {
                current_port = Some(port);
            }

            let Some(index) = current_index else {
                // BLOCKED_MISSING_FACT: при неуспехе первого numeric
                // extraction exact EXE всё равно использовал неизвестный
                // stack-key в map::operator[]. Safe Rust не выбирает ключ.
                blocked_at_record = Some(record_index as usize + 1);
                break;
            };
            self.game_servers.insert(
                index,
                WorldGameServerEntry {
                    connected: false,
                    index,
                    ip: current_ip.clone(),
                    port: current_port,
                    // VERIFIED_DISASSEMBLY: 0x004139FE -> 0x00413A08
                    // переносит неинициализированный stack DWORD.
                    received_player_data: None,
                },
            );
            applied_records += 1;
        }

        Ok(WorldServerSetupLoadReport {
            declared_records,
            applied_records,
            unique_game_servers: self.game_servers.len(),
            stream_complete: !tokens.failed(),
            blocked_at_record,
            read_end_notice: blocked_at_record.is_none(),
        })
    }

    fn record_game_init_log(
        &self,
        events: &mut Vec<WorldGameInitEvent>,
        log: &mut WorldLogTextOwner,
        callbacks: &mut WorldGameInitCallbacks<'_>,
        payload: &[u8],
    ) {
        let disposition = log.add_log_text(
            payload,
            self.setup.save_info_time_ms,
            &mut *callbacks.get_tick,
            &mut *callbacks.get_log_local_time,
            &mut *callbacks.put_log_info,
        );
        events.push(WorldGameInitEvent::Log {
            payload: payload.to_vec(),
            disposition,
        });
    }

    fn record_game_init_notice<Context: WorldGameInitContext>(
        events: &mut Vec<WorldGameInitEvent>,
        context: &mut Context,
        title: &[u8],
        message: &[u8],
    ) {
        let notice = WorldGameInitOperatorNotice {
            title: title.to_vec(),
            message: message.to_vec(),
        };
        context.notify_operator(&notice);
        events.push(WorldGameInitEvent::OperatorNotice(notice));
    }

    fn load_jjc_configuration_from_resources<Context: WorldReloadContext + ?Sized>(
        jjc: &mut CJJcSystem,
        context: &mut Context,
    ) -> JjcConfigurationLoadReport {
        let Some(region_source) = context.read_resource(JJC_REGION_LIST_PATH) else {
            return jjc.load_configuration(None, None, None);
        };
        let Some(level_source) = context.read_resource(JJC_LEVEL_LIST_PATH) else {
            return jjc.load_configuration(Some(&region_source), None, None);
        };
        let config_source = context.read_resource(JJC_CONFIG_PATH);
        jjc.load_configuration(
            Some(&region_source),
            Some(&level_source),
            config_source.as_deref(),
        )
    }

    fn database_initialization_snapshot(&self) -> WorldGameDatabaseInitialization {
        WorldGameDatabaseInitialization {
            settings: WorldDatabaseSettings::from_parts(WorldDatabaseSettingsParts {
                host: self.setup.sql_server_ip.clone(),
                database: self.setup.database_name.clone(),
                user: self.setup.sql_user_name.clone(),
                password: self.setup.sql_password.clone(),
            }),
            connection_type: self.setup.sql_connection_type.clone(),
            legacy_zero: b"0",
            integrated_security: b"SSPI",
        }
    }

    /// Выполняет полный `CGame::Init` до запуска write/player-load workers.
    ///
    /// Windows crash reporter, GUI notice и thread creation передаются точным
    /// внешним границам; file/network owners и live `CGame` мутации исполняются
    /// непосредственно здесь. Первый false/block прекращает оставшийся порядок.
    ///
    /// Для country-участка EXE/PDB подтверждают порядок: загрузить параметры,
    /// при включённой appellation-функции загрузить honor ranks, передать
    /// текущий локальный день в `CCountryHandler::Initialize`, проверить его
    /// результат, записать `Load Country SUCCESS...` и лишь затем запускать
    /// country war. Rust использует те же живые `CCountryParam`,
    /// `CCountryHandler` и `CountryWarSys`; resource bytes,
    /// Tiberius-соединение, `CTimer` и календарный контекст передаются явно
    /// вместо resource manager, глобальных singleton-ов и process-global
    /// времени оригинала. Эти технические замены не меняют доказанный
    /// fail-fast порядок и вызов `SetNewDay` после успешной DB-load.
    /// Goods War сохраняет отдельный соседний контракт constructor-а: exact
    /// `reInitDB` выполняется между `DbCountry` и `DbMisc`, но собственный
    /// DB-error поглощается после сохранения прочитанного prefix-а. Поэтому
    /// typed load-report входит в event stream и не становится init block-ом.
    /// Increment log также является concrete owner-ом этого порядка: после
    /// general variables он потоково читает Log DB, пишет исходный
    /// success/failure log и не превращает старый непроверяемый результат в
    /// новый init-block. `Release` очищает тот же owner ровно между
    /// `TimeToReturn::uninitialize` и `CCountryHandler::Release`. Открытое
    /// Tiberius-соединение и явная Rust-ссылка заменяют только внутренние
    /// ADO/singleton mechanics.
    #[allow(
        clippy::too_many_arguments,
        reason = "прямые PlayerRanks/country/timer/increment owners заменяют прежние opaque callbacks"
    )]
    pub(crate) async fn init<Context, TimerCallback, CountryDatabase, CountryContext>(
        &mut self,
        runtime_directory: &Path,
        context: &mut Context,
        jjc: &mut CJJcSystem,
        organizing_parameters: &mut COrganizingParam,
        player_ranks: &mut CPlayerRanks,
        timer: &mut CTimer<TimerCallback>,
        organizing_tax_callback: TimerCallback,
        player_ranks_callback: TimerCallback,
        organizing: &mut COrganizingCtrl,
        country_handler: &mut CCountryHandler,
        country_parameters: &mut CCountryParam,
        country_database: &mut CountryDatabase,
        country_database_connection: Option<&mut WorldTdsClient>,
        goods_war: &mut CGoodsWarMember,
        goods_war_database_connection: Option<&mut WorldTdsClient>,
        country_context: &mut CountryContext,
        country_war_system: &mut CountryWarSys,
        country_war_callbacks: CountryWarCallbacks<TimerCallback>,
        honor_ranks: &mut CHonorRanks,
        increment_log: &mut CIncrementLog,
        auction_log: &mut CAuctionLog,
        log: &mut WorldLogTextOwner,
        callbacks: &mut WorldGameInitCallbacks<'_>,
    ) -> WorldGameInitResult<Context::Block>
    where
        Context: WorldGameInitContext,
        TimerCallback: Copy,
        CountryDatabase: DbCountryOwner,
        CountryContext: CountrySetNewDayContext + ?Sized,
    {
        let mut events = Vec::new();
        macro_rules! stop {
            ($reason:expr) => {
                return Err(Box::new(WorldGameInitBlock {
                    events,
                    reason: $reason,
                }))
            };
        }

        context.install_crash_reporter();
        events.push(WorldGameInitEvent::CrashReporterInstalled);

        let seed = context.current_time_seconds() as u32;
        context.seed_random(seed);
        let discarded_roll = context.random(100);
        events.push(WorldGameInitEvent::RandomInitialized {
            seed,
            discarded_roll,
        });

        // Оба critical section уже являются Rust owners: `db_data` Mutex и
        // эксклюзивная save-thread guard-граница. До worker-start их не видно.
        events.push(WorldGameInitEvent::RustLocksReady);
        context.put_debug_string(b"WorldServer start!");
        events.push(WorldGameInitEvent::DebugStartPublished);
        context.load_server_resources(self);
        events.push(WorldGameInitEvent::ServerResourcesLoaded);

        let setup = match self.load_setup(runtime_directory, |title| {
            context.claim_single_instance(title)
        }) {
            Ok(setup) => setup,
            Err(error) => stop!(WorldGameInitBlockReason::SetupOpen(error)),
        };
        let instance_claimed = setup.instance_claimed;
        let instance_title = setup.instance_title.clone();
        events.push(WorldGameInitEvent::SetupLoaded(setup));
        if !instance_claimed {
            let mut message = instance_title.clone();
            message.extend_from_slice(b" App Is Running!");
            Self::record_game_init_notice(&mut events, context, b"ERROR", &message);
            stop!(WorldGameInitBlockReason::ExistingInstance {
                title: instance_title,
            });
        }

        let server_setup = match self.load_server_setup(runtime_directory) {
            Ok(setup) => setup,
            Err(error) => stop!(WorldGameInitBlockReason::ServerSetup(error)),
        };
        events.push(WorldGameInitEvent::ServerSetupLoaded(server_setup));

        let Some(player_load_thread_count) = self.setup.player_load_thread_count else {
            stop!(WorldGameInitBlockReason::MissingPlayerLoadThreadCount);
        };
        if player_load_thread_count == 0 || 8 < player_load_thread_count {
            Self::record_game_init_notice(
                &mut events,
                context,
                b"message",
                b"Player I/O Threads Must between 1 And 8",
            );
            stop!(WorldGameInitBlockReason::InvalidPlayerLoadThreadCount {
                count: player_load_thread_count,
                legacy_exit_code: 1,
            });
        }
        events.push(WorldGameInitEvent::PlayerLoadThreadCountValidated(
            player_load_thread_count,
        ));

        self.clear_string_table();
        events.push(WorldGameInitEvent::StringTablesCleared);
        const DEFAULT_LANGUAGE: &[u8] = b"data/Language.lag";
        let default_language_source = context.read_resource(DEFAULT_LANGUAGE);
        let default_language = self.load_string_table_resource(
            DEFAULT_LANGUAGE,
            default_language_source.as_deref(),
        );
        self.record_game_init_log(
            &mut events,
            log,
            callbacks,
            &default_language.log_payload,
        );
        if !default_language.succeeded {
            self.record_game_init_log(
                &mut events,
                log,
                callbacks,
                b"read language (data/Language.lag).....failed!",
            );
            stop!(WorldGameInitBlockReason::DefaultLanguageTable);
        }
        events.push(WorldGameInitEvent::StringTableLoaded {
            package: DEFAULT_LANGUAGE.to_vec(),
        });
        let configured_language = self.setup.language_package.clone();
        let configured_language_source = context.read_resource(&configured_language);
        let configured_language_load = self.load_string_table_resource(
            &configured_language,
            configured_language_source.as_deref(),
        );
        self.record_game_init_log(
            &mut events,
            log,
            callbacks,
            &configured_language_load.log_payload,
        );
        if !configured_language_load.succeeded {
            self.record_game_init_log(
                &mut events,
                log,
                callbacks,
                b"Load language packet [data/Language.lag]...FAILED! ",
            );
            stop!(WorldGameInitBlockReason::ConfiguredLanguageTable);
        }
        events.push(WorldGameInitEvent::StringTableLoaded {
            package: configured_language,
        });
        if let Err(block) = self.code_string_table() {
            stop!(WorldGameInitBlockReason::StringTableEncoding(block));
        }
        events.push(WorldGameInitEvent::StringTablesCoded);

        const DUPLI_REGION_SETUP_PATH: &[u8] = b"setup/DupliRegionsSetup.ini";
        self.dupli_region_setup = Some(CDupliRegionSetup::default());
        let dupli_region_source = context.read_resource(DUPLI_REGION_SETUP_PATH);
        let dupli_region_loaded = self
            .dupli_region_setup
            .as_mut()
            .expect("owner опубликован перед Load")
            .load(dupli_region_source.as_deref());
        if !dupli_region_loaded {
            Self::record_game_init_notice(
                &mut events,
                context,
                b"message",
                b"Can't find file setup/DupliRegionsSetup.ini",
            );
            stop!(WorldGameInitBlockReason::DupliRegionSetup);
        }
        events.push(WorldGameInitEvent::DupliRegionSetupLoaded);

        if let Err(block) =
            context.initialize_database_layer(self.database_initialization_snapshot())
        {
            stop!(WorldGameInitBlockReason::Context(block));
        }
        events.push(WorldGameInitEvent::DatabaseLayerInitialized);
        context.initialize_void_owner(WorldGameInitVoidOwner::InitializeLargess);
        events.push(WorldGameInitEvent::VoidOwner(
            WorldGameInitVoidOwner::InitializeLargess,
        ));
        let jjc_configuration = Self::load_jjc_configuration_from_resources(jjc, context);
        if let Some(path) = jjc_configuration.missing_path() {
            Self::record_game_init_notice(
                &mut events,
                context,
                b"file not found",
                path,
            );
            stop!(WorldGameInitBlockReason::JjcConfiguration(
                jjc_configuration,
            ));
        }
        events.push(WorldGameInitEvent::JjcConfigurationLoaded(jjc_configuration));

        if let Err(block) = context.create_database_owner(WorldGameDatabaseOwner::RsPlayer) {
            stop!(WorldGameInitBlockReason::Context(block));
        }
        events.push(WorldGameInitEvent::DatabaseOwnerCreated(
            WorldGameDatabaseOwner::RsPlayer,
        ));
        let loaded_setup_ids = match context.create_rs_setup_owner() {
            Ok(ids) => ids,
            Err(block) => stop!(WorldGameInitBlockReason::Context(block)),
        };
        self.apply_loaded_setup_ids(loaded_setup_ids);
        events.push(WorldGameInitEvent::RsSetupOwnerCreated(loaded_setup_ids));

        const DATABASE_OWNERS_BEFORE_GOODS_WAR: &[WorldGameDatabaseOwner] = &[
            WorldGameDatabaseOwner::RsGenVar,
            WorldGameDatabaseOwner::RsFaction,
            WorldGameDatabaseOwner::RsUnion,
            WorldGameDatabaseOwner::RsEnemyFactions,
            WorldGameDatabaseOwner::RsVillageWar,
            WorldGameDatabaseOwner::RsCityWar,
            WorldGameDatabaseOwner::RsRegion,
            WorldGameDatabaseOwner::DbCountry,
        ];
        for &owner in DATABASE_OWNERS_BEFORE_GOODS_WAR {
            if let Err(block) = context.create_database_owner(owner) {
                stop!(WorldGameInitBlockReason::Context(block));
            }
            events.push(WorldGameInitEvent::DatabaseOwnerCreated(owner));
        }

        // Exact constructor ловил DB/COM error внутри `reInitDB`: owner
        // оставался опубликованным, а CGame::Init продолжал следующий шаг.
        // Замена прежнего Rust owner-а повторяет `new`; старый owner штатно
        // освобождается Drop вместо исходной утечки при повторном Init.
        *goods_war = CGoodsWarMember::with_reached_empty_state();
        goods_war.begin_lifecycle();
        let goods_war_report = goods_war
            .reinitialize_database(goods_war_database_connection)
            .await;
        events.push(WorldGameInitEvent::DatabaseOwnerCreated(
            WorldGameDatabaseOwner::GoodsWarMember,
        ));
        events.push(WorldGameInitEvent::GoodsWarMemberLoaded(goods_war_report));

        const DATABASE_OWNERS_AFTER_GOODS_WAR: &[WorldGameDatabaseOwner] = &[
            WorldGameDatabaseOwner::DbMisc,
            WorldGameDatabaseOwner::RsGodsBattle,
        ];
        for &owner in DATABASE_OWNERS_AFTER_GOODS_WAR {
            if let Err(block) = context.create_database_owner(owner) {
                stop!(WorldGameInitBlockReason::Context(block));
            }
            events.push(WorldGameInitEvent::DatabaseOwnerCreated(owner));
        }

        const INITIAL_RELOADS: &[&[u8]] = &[
            b"Allthing",
            b"PlayerList",
            b"GoodsList",
            b"MonsterList",
            b"PreciousBoxConf",
            b"FairyExpConf",
            b"TradeList",
            b"IncrementShopList",
            b"ContributeSetup",
            b"SkillList",
            b"GlobeSetup",
            b"LogSystem",
            b"ScriptFile",
            b"GMList",
            b"RegionList",
            b"NewSkillMonsterList",
            b"SynthesisList",
            b"EquipmentCompose",
            b"DaKongXiangQian",
            b"GoodsDestroyConf",
            b"HonorElimilate",
        ];
        for &profile in INITIAL_RELOADS {
            let legacy_result = match self.reload(context, jjc, profile, false, false) {
                Ok(result) => result,
                Err(block) => stop!(WorldGameInitBlockReason::Reload(block)),
            };
            events.push(WorldGameInitEvent::Reload {
                profile,
                legacy_result,
            });
        }

        let region_parameters_loaded = context.load_region_parameters(self);
        events.push(WorldGameInitEvent::RegionParametersLoaded {
            succeeded: region_parameters_loaded,
        });
        self.record_game_init_log(
            &mut events,
            log,
            callbacks,
            if region_parameters_loaded {
                b"load region tax rate from db...OK!"
            } else {
                b"load region tax rate from db...FAILED!"
            },
        );
        const SECONDARY_RELOADS: &[&[u8]] = &[
            b"RegionLevelSetup",
            b"HitLevelSetup",
            b"Help",
            b"Broadcast",
            b"PrisonConf",
            b"ciqing",
            b"taozhuang",
        ];
        for &profile in SECONDARY_RELOADS {
            let legacy_result = match self.reload(context, jjc, profile, false, false) {
                Ok(result) => result,
                Err(block) => stop!(WorldGameInitBlockReason::Reload(block)),
            };
            events.push(WorldGameInitEvent::Reload {
                profile,
                legacy_result,
            });
        }

        let owner = WorldGameInitBooleanOwner::InitializeTimeToReturn;
        let succeeded = context.initialize_boolean_owner(owner);
        events.push(WorldGameInitEvent::BooleanOwner { owner, succeeded });
        if !succeeded {
            self.record_game_init_log(
                &mut events,
                log,
                callbacks,
                b"Load CityRetern Timing FAILED...",
            );
            stop!(WorldGameInitBlockReason::BooleanOwner(owner));
        }

        const INVALID_STRINGS: &[u8] = b"setup/InvalidStr.ini";
        const CHAR_CODES: &[u8] = b"setup/charcode.ini";
        let invalid_strings = context.read_resource(INVALID_STRINGS);
        let char_codes = invalid_strings
            .as_ref()
            .and_then(|_| context.read_resource(CHAR_CODES));
        let _ = self.words_filter.initial(
            INVALID_STRINGS,
            CHAR_CODES,
            invalid_strings.as_deref(),
            char_codes.as_deref(),
        );
        events.push(WorldGameInitEvent::WordsFilterInitialized);
        for &profile in &[
            b"BattleFairyExpConfig".as_slice(),
            b"BattleFairyCombineConfig",
        ] {
            let legacy_result = match self.reload(context, jjc, profile, false, false) {
                Ok(result) => result,
                Err(block) => stop!(WorldGameInitBlockReason::Reload(block)),
            };
            events.push(WorldGameInitEvent::Reload {
                profile,
                legacy_result,
            });
        }

        let organizing_parameters_now = (callbacks.get_timer_local_time)();
        let organizing_parameters_load = match organizing_parameters.initialize(
            runtime_directory,
            organizing_parameters_now,
            timer,
            organizing_tax_callback,
        ) {
            Ok(report) => report,
            Err(source) => {
                if matches!(&source, OrganizingParamLoadError::Open { .. }) {
                    Self::record_game_init_notice(
                        &mut events,
                        context,
                        b"ERROR",
                        b"file 'data/FactionParam.ini' can't found!",
                    );
                }
                self.record_game_init_log(
                    &mut events,
                    log,
                    callbacks,
                    b"Load OrganizingParam FAILED...",
                );
                stop!(WorldGameInitBlockReason::OrganizingParameters(source));
            }
        };
        events.push(WorldGameInitEvent::OrganizingParametersLoaded(
            organizing_parameters_load,
        ));

        let legacy_result = match self.reload(context, jjc, b"godsBattle", false, false) {
            Ok(result) => result,
            Err(block) => stop!(WorldGameInitBlockReason::Reload(block)),
        };
        events.push(WorldGameInitEvent::Reload {
            profile: b"godsBattle",
            legacy_result,
        });
        context.initialize_void_owner(WorldGameInitVoidOwner::LoadGodsBattleFactionXyd);
        events.push(WorldGameInitEvent::VoidOwner(
            WorldGameInitVoidOwner::LoadGodsBattleFactionXyd,
        ));

        let owner = WorldGameInitBooleanOwner::InitializeAttackCity;
        let succeeded = context.initialize_boolean_owner(owner);
        events.push(WorldGameInitEvent::BooleanOwner { owner, succeeded });
        if !succeeded {
            self.record_game_init_log(
                &mut events,
                log,
                callbacks,
                b"Load setup/CityWarSys.ini FAILED...",
            );
            stop!(WorldGameInitBlockReason::BooleanOwner(owner));
        }
        context.initialize_void_owner(WorldGameInitVoidOwner::InitializeAttackCityEnemyRelations);
        events.push(WorldGameInitEvent::VoidOwner(
            WorldGameInitVoidOwner::InitializeAttackCityEnemyRelations,
        ));

        let owner = WorldGameInitBooleanOwner::InitializeFourNationWar;
        let succeeded = context.initialize_boolean_owner(owner);
        events.push(WorldGameInitEvent::BooleanOwner { owner, succeeded });
        if !succeeded {
            let localized = self.get_string_by_id(b"XBWS0021").to_vec();
            self.record_game_init_log(&mut events, log, callbacks, &localized);
            stop!(WorldGameInitBlockReason::BooleanOwner(owner));
        }

        let owner = WorldGameInitBooleanOwner::InitializeVillageWar;
        let succeeded = context.initialize_boolean_owner(owner);
        events.push(WorldGameInitEvent::BooleanOwner { owner, succeeded });
        if !succeeded {
            self.record_game_init_log(
                &mut events,
                log,
                callbacks,
                b"Load setup/villageWarSys.ini failed!",
            );
            stop!(WorldGameInitBlockReason::BooleanOwner(owner));
        }

        context.initialize_void_owner(WorldGameInitVoidOwner::InitializeOrganizingController);
        events.push(WorldGameInitEvent::VoidOwner(
            WorldGameInitVoidOwner::InitializeOrganizingController,
        ));
        let region_ids = self.regions.keys().copied().collect::<Vec<_>>();
        for region_id in region_ids {
            let Some(mut region_owner) = self
                .regions
                .get_mut(&region_id)
                .and_then(|assignment| assignment.region.take())
            else {
                continue;
            };
            let relation = region_owner.base_mut().init_owner_relation(
                organizing,
                &*self,
                &mut *callbacks.update_player,
            );
            self.regions
                .get_mut(&region_id)
                .expect("region-map key не удаляется во время owner relation")
                .region = Some(region_owner);
            let report = match relation {
                Ok(report) => report,
                Err(source) => stop!(WorldGameInitBlockReason::RegionOwnerRelation {
                    region_id,
                    source,
                }),
            };
            events.push(WorldGameInitEvent::RegionOwnerRelationInitialized {
                region_id,
                report,
            });
        }

        for owner in [
            WorldGameInitVoidOwner::InitializeFactionWar,
            WorldGameInitVoidOwner::InitializeQuestSystem,
        ] {
            context.initialize_void_owner(owner);
            events.push(WorldGameInitEvent::VoidOwner(owner));
        }

        let player_ranks_configuration = PlayerRanksInitializationConfig {
            stat_time: organizing_parameters.stat_player_ranks_time(),
            maximum_count: organizing_parameters.player_ranks_count(),
        };
        let player_ranks_now = (callbacks.get_timer_local_time)();
        let player_ranks_initialization = match player_ranks.initialize(
            player_ranks_configuration,
            player_ranks_now,
            timer,
            player_ranks_callback,
        ) {
            Ok(report) => report,
            Err(source) => stop!(WorldGameInitBlockReason::PlayerRanksSchedule(source)),
        };
        events.push(WorldGameInitEvent::PlayerRanksInitialized(
            player_ranks_initialization,
        ));
        let player_ranks_stat = {
            let (database, active_transaction) = context.player_database();
            self.stat_player_ranks(
                player_ranks,
                database,
                active_transaction,
                organizing,
                log,
                &mut *callbacks.get_tick,
                &mut *callbacks.get_log_local_time,
                &mut *callbacks.put_log_info,
            )
            .await
        };
        let player_ranks_stat = match player_ranks_stat {
            Ok(report) => report,
            Err(source) => stop!(WorldGameInitBlockReason::PlayerRanksStat(source)),
        };
        events.push(WorldGameInitEvent::PlayerRanksLoaded(player_ranks_stat));

        let country_parameter_source = context.country_parameter_source();
        let country_parameter_report = match country_parameters
            .initialize(country_parameter_source.as_deref())
        {
            Ok(report) => report,
            Err(source) => stop!(WorldGameInitBlockReason::CountryParameters(source)),
        };
        events.push(WorldGameInitEvent::CountryParametersLoaded(
            country_parameter_report,
        ));

        if context.use_appellation_function() {
            let _unused_system_time = (callbacks.get_log_local_time)();
            let started_at_ms = (callbacks.get_tick)();
            self.record_game_init_log(&mut events, log, callbacks, b"Start total HonorRankks!");
            let outcome = {
                let (database, active_transaction) = context.player_database();
                honor_ranks
                    .load_honor_ranks(database, active_transaction)
                    .await
            };
            let finished_at_ms = (callbacks.get_tick)();
            let elapsed_ms = finished_at_ms.wrapping_sub(started_at_ms);
            let complete = format!(
                "Total today HonorRankks complete,consume time {} millisecond!",
                elapsed_ms as i32,
            )
            .into_bytes();
            self.record_game_init_log(&mut events, log, callbacks, &complete);
            events.push(WorldGameInitEvent::HonorRanksLoaded {
                started_at_ms,
                finished_at_ms,
                elapsed_ms,
                outcome,
            });
        }

        let country_local_time = (callbacks.get_timer_local_time)();
        let country_initialization = country_handler
            .initialize(
                i32::from(country_local_time.day),
                country_database,
                country_database_connection,
                country_parameters,
                country_context,
            )
            .await;
        let succeeded = country_initialization.legacy_result;
        events.push(WorldGameInitEvent::CountryHandlerInitialized(
            country_initialization,
        ));
        if !succeeded {
            stop!(WorldGameInitBlockReason::CountryHandler);
        }
        self.record_game_init_log(&mut events, log, callbacks, b"Load Country SUCCESS...");

        let country_war_source = context.country_war_source();
        let country_war_now = (callbacks.get_timer_local_time)();
        let country_war_initialization = country_war_system.initialize(
            country_war_source.as_deref(),
            country_war_now,
            timer,
            country_war_callbacks,
            |payload| {
                let disposition = log.add_log_text(
                    payload,
                    self.setup.save_info_time_ms,
                    &mut *callbacks.get_tick,
                    &mut *callbacks.get_log_local_time,
                    &mut *callbacks.put_log_info,
                );
                events.push(WorldGameInitEvent::Log {
                    payload: payload.to_vec(),
                    disposition,
                });
            },
        );
        let country_war_initialization = match country_war_initialization {
            Ok(report) => report,
            Err(source) => stop!(WorldGameInitBlockReason::CountryWarLoad(source)),
        };
        let succeeded = country_war_initialization.legacy_result;
        events.push(WorldGameInitEvent::CountryWarInitialized(
            country_war_initialization,
        ));
        if !succeeded {
            stop!(WorldGameInitBlockReason::CountryWar);
        }

        for owner in [
            WorldGameInitVoidOwner::CreateGeneralVariableList,
            WorldGameInitVoidOwner::LoadGeneralVariableList,
            WorldGameInitVoidOwner::LoadGeneralVariableData,
        ] {
            context.initialize_void_owner(owner);
            events.push(WorldGameInitEvent::VoidOwner(owner));
        }

        let increment_log_days = context.increment_log_days();
        let outcome = increment_log
            .load(context.increment_log_database(), increment_log_days)
            .await;
        let succeeded = outcome.succeeded();
        events.push(WorldGameInitEvent::IncrementLogLoaded { outcome });
        self.record_game_init_log(
            &mut events,
            log,
            callbacks,
            if succeeded {
                b"Load IncShop Log SUCCESS..."
            } else {
                b"Load IncShop Log FAILED..."
            },
        );

        let outcome = auction_log
            .load_item(context.auction_log_database(), increment_log_days)
            .await;
        let succeeded = matches!(&outcome, AuctionLogLoadOutcome::ReturnedTrue);
        events.push(WorldGameInitEvent::AuctionLogLoaded { outcome });
        self.record_game_init_log(
            &mut events,
            log,
            callbacks,
            if succeeded {
                b"load Auction Log SUCCESS!"
            } else {
                b"load auctionLog FAILED!"
            },
        );

        for owner in [
            WorldGameInitVoidOwner::InitializeBaseMessage,
            WorldGameInitVoidOwner::InitializeSocket,
        ] {
            context.initialize_void_owner(owner);
            events.push(WorldGameInitEvent::VoidOwner(owner));
        }

        match self.init_net_client().await {
            Ok(initialized) => {
                events.push(WorldGameInitEvent::NetworkClientInitialized(initialized))
            }
            Err(error) => {
                Self::record_game_init_notice(
                    &mut events,
                    context,
                    b"Message",
                    b"Can't connect to LoginServer, please run LoginServer first!",
                );
                stop!(WorldGameInitBlockReason::NetworkClient(error));
            }
        }
        match self.init_net_server() {
            Ok(()) => events.push(WorldGameInitEvent::NetworkServerInitialized),
            Err(error) => {
                Self::record_game_init_notice(
                    &mut events,
                    context,
                    b"Message",
                    b"Can't init NetServer!",
                );
                stop!(WorldGameInitBlockReason::NetworkServer(error));
            }
        }

        self.player_data_queue.clear();
        events.push(WorldGameInitEvent::PlayerDataQueueCleared);
        let copy_number_schedule = match context.register_clear_copy_number_time() {
            Ok(schedule) => schedule,
            Err(error) => stop!(WorldGameInitBlockReason::CopyNumberSchedule(error)),
        };
        events.push(WorldGameInitEvent::CopyNumberResetScheduled(
            copy_number_schedule,
        ));

        let kind = WorldGameInitWorkerKind::WriteLog;
        let handle = context.start_write_log_worker(self.write_log_worker_spec());
        events.push(WorldGameInitEvent::WorkerStarted { kind, handle });
        for worker_index in 0..player_load_thread_count {
            let kind = WorldGameInitWorkerKind::LoadPlayerData { worker_index };
            let handle = context
                .start_player_load_worker(self.player_load_worker_spec(), worker_index);
            events.push(WorldGameInitEvent::WorkerStarted { kind, handle });
        }

        Ok(WorldGameInitReport {
            events,
            legacy_result: 1,
        })
    }

    /// Вызывает city-region save только при selector `0` и `REGION_TYPE == 2`.
    fn save_city_region<SaveRegion>(
        &mut self,
        selector: i32,
        mut save_region: SaveRegion,
    ) -> Result<Vec<i32>, (Vec<i32>, WorldSaveCityRegionBlock)>
    where
        SaveRegion: FnMut(i32, &mut WorldRegionOwner),
    {
        if selector != 0 {
            return Ok(Vec::new());
        }

        let region_ids = self.regions.keys().copied().collect::<Vec<_>>();
        let mut saved = Vec::new();
        for region_id in region_ids {
            let assignment = self
                .regions
                .get_mut(&region_id)
                .expect("region ID взят из текущего map");
            let Some(region_type) = assignment.region_type else {
                return Err((
                    saved,
                    WorldSaveCityRegionBlock::UninitializedRegionType { region_id },
                ));
            };
            if region_type != 2 {
                continue;
            }
            let Some(region) = assignment.region.as_mut() else {
                // WorldServer RVA 0x00008750 разыменовывал `pRegion` без null-check.
                return Err((
                    saved,
                    WorldSaveCityRegionBlock::NullCityRegion { region_id },
                ));
            };
            save_region(region_id, region);
            saved.push(region_id);
        }
        Ok(saved)
    }

    /// Уничтожает все live player-owner-ы в unsigned key-order.
    fn clear_map_player(&mut self) -> usize {
        let entries = self.players.len();
        while let Some((_player_id, player)) = self.players.pop_first() {
            drop(player);
        }
        entries
    }

    /// Выполняет полный `CGame::Release` до legacy result `1`.
    /// Concrete Goods War owner освобождается между CityWar и RsRegion, как
    /// exact pointer-owner, но его Rust collections использует обычный Drop.
    pub(crate) fn release<Context: WorldGameReleaseContext>(
        &mut self,
        context: &mut Context,
        goods_war: &mut CGoodsWarMember,
        increment_log: &mut CIncrementLog,
    ) -> WorldGameReleaseResult {
        let mut events = Vec::new();

        context.put_debug_string(b"WorldServer Exiting...");
        events.push(WorldGameReleaseEvent::DebugPublished(
            b"WorldServer Exiting...",
        ));
        self.player_data_queue.clear();
        events.push(WorldGameReleaseEvent::PlayerDataQueueCleared);

        let saved_city_regions = match self.save_city_region(0, |region_id, region| {
            context.save_city_region(region_id, region);
        }) {
            Ok(saved) => saved,
            Err((saved, block)) => {
                for region_id in saved {
                    events.push(WorldGameReleaseEvent::CityRegionSaved { region_id });
                }
                return Err(Box::new(WorldGameReleaseBlock { events, block }));
            }
        };
        for region_id in saved_city_regions {
            events.push(WorldGameReleaseEvent::CityRegionSaved { region_id });
        }

        if let Some(server) = self.net_server.as_mut() {
            context.exit_network_server_worker(server);
            events.push(WorldGameReleaseEvent::NetworkServerWorkerExited);
        }
        if let Some(client) = self.net_client.as_mut() {
            context.exit_network_client_worker(client);
            events.push(WorldGameReleaseEvent::NetworkClientWorkerExited);
        }

        macro_rules! clear_live_list {
            ($field:ident, $owner:expr) => {{
                let entries = self.$field.len();
                self.$field.clear();
                events.push(WorldGameReleaseEvent::LiveListCleared {
                    owner: $owner,
                    entries,
                });
            }};
        }
        clear_live_list!(creation_players, WorldGameReleaseLiveList::Creation);
        clear_live_list!(restore_players, WorldGameReleaseLiveList::Restore);
        clear_live_list!(deletion_players, WorldGameReleaseLiveList::Deletion);
        clear_live_list!(online_players, WorldGameReleaseLiveList::Online);
        clear_live_list!(offline_players, WorldGameReleaseLiveList::Offline);
        clear_live_list!(login_players, WorldGameReleaseLiveList::Login);

        let entries = self.clear_map_player();
        events.push(WorldGameReleaseEvent::PlayerMapCleared { entries });
        self.clear_db_data();
        events.push(WorldGameReleaseEvent::DbDataCleared);

        let previous_handle = context.join_save_worker();
        events.push(WorldGameReleaseEvent::SaveWorkerJoined { previous_handle });

        context.release_void_owner(WorldGameReleaseVoidOwner::ReleaseGoodsLinks);
        self.goods_links.clear();
        events.push(WorldGameReleaseEvent::VoidOwner(
            WorldGameReleaseVoidOwner::ReleaseGoodsLinks,
        ));

        let region_ids = self.regions.keys().copied().collect::<Vec<_>>();
        for region_id in region_ids {
            let Some(region) = self
                .regions
                .get_mut(&region_id)
                .and_then(|assignment| assignment.region.take())
            else {
                continue;
            };
            drop(region);
            events.push(WorldGameReleaseEvent::RegionOwnerReleased { region_id });
        }

        for (owner, released) in [
            (
                WorldGameReleaseOptionalOwner::FunctionListFileData,
                self.function_list_file_data.take().is_some(),
            ),
            (
                WorldGameReleaseOptionalOwner::VariableListFileData,
                self.variable_list_file_data.take().is_some(),
            ),
            (
                WorldGameReleaseOptionalOwner::ScriptFileData,
                !self.script_file_data.is_empty(),
            ),
        ] {
            events.push(WorldGameReleaseEvent::OptionalOwner { owner, released });
        }
        self.script_file_data.clear();
        let owner = WorldGameReleaseOptionalOwner::GeneralVariableList;
        let released = context.release_optional_owner(owner);
        events.push(WorldGameReleaseEvent::OptionalOwner { owner, released });

        self.increment_shop_list.release();
        for owner in [WorldGameReleaseVoidOwner::UninitializeTimeToReturn] {
            context.release_void_owner(owner);
            events.push(WorldGameReleaseEvent::VoidOwner(owner));
        }
        increment_log.uninitialize();
        events.push(WorldGameReleaseEvent::VoidOwner(
            WorldGameReleaseVoidOwner::UninitializeIncrementLog,
        ));
        context.release_void_owner(WorldGameReleaseVoidOwner::ReleaseCountryHandler);
        events.push(WorldGameReleaseEvent::VoidOwner(
            WorldGameReleaseVoidOwner::ReleaseCountryHandler,
        ));
        self.words_filter.clear();
        events.push(WorldGameReleaseEvent::VoidOwner(
            WorldGameReleaseVoidOwner::ReleaseWordsFilter,
        ));
        for owner in [
            WorldGameReleaseVoidOwner::ReleaseOrganizingController,
            WorldGameReleaseVoidOwner::ReleaseAttackCity,
            WorldGameReleaseVoidOwner::ReleaseVillageWar,
            WorldGameReleaseVoidOwner::ReleaseOrganizingParameters,
            WorldGameReleaseVoidOwner::ReleaseQuestSystem,
            WorldGameReleaseVoidOwner::ReleaseFactionWar,
        ] {
            context.release_void_owner(owner);
            events.push(WorldGameReleaseEvent::VoidOwner(owner));
        }
        let player_ranks = context.release_player_ranks();
        events.push(WorldGameReleaseEvent::PlayerRanksReleased(player_ranks));
        let owner = WorldGameReleaseVoidOwner::ReleaseTimer;
        context.release_void_owner(owner);
        events.push(WorldGameReleaseEvent::VoidOwner(owner));

        if let Some(client) = self.net_client.take() {
            drop(client);
            events.push(WorldGameReleaseEvent::NetworkClientReleased);
        }
        if let Some(server) = self.net_server.take() {
            drop(server);
            events.push(WorldGameReleaseEvent::NetworkServerReleased);
        }

        for owner in [
            WorldGameReleaseDatabaseOwner::RsPlayer,
            WorldGameReleaseDatabaseOwner::RsSetup,
            WorldGameReleaseDatabaseOwner::RsGenVar,
            WorldGameReleaseDatabaseOwner::RsFaction,
            WorldGameReleaseDatabaseOwner::RsUnion,
            WorldGameReleaseDatabaseOwner::RsEnemyFactions,
            WorldGameReleaseDatabaseOwner::RsVillageWar,
            WorldGameReleaseDatabaseOwner::RsCityWar,
        ] {
            let released = context.release_database_owner(owner);
            events.push(WorldGameReleaseEvent::DatabaseOwner { owner, released });
        }
        let owner = WorldGameReleaseDatabaseOwner::GoodsWarMember;
        let released = goods_war.release_lifecycle();
        events.push(WorldGameReleaseEvent::DatabaseOwner { owner, released });
        for owner in [
            WorldGameReleaseDatabaseOwner::RsRegion,
            WorldGameReleaseDatabaseOwner::DbCountry,
            WorldGameReleaseDatabaseOwner::RsGodsBattle,
        ] {
            let released = context.release_database_owner(owner);
            events.push(WorldGameReleaseEvent::DatabaseOwner { owner, released });
        }
        // WorldServer RVA 0x0000E7F0 создаёт `m_pRsMisc` в Init, но не удаляет
        // и не обнуляет его ни в одном Release call-site до skill-cache cleanup.
        events.push(WorldGameReleaseEvent::DatabaseMiscRetained);

        for owner in [
            WorldGameReleaseVoidOwner::ClearSkillCache,
            WorldGameReleaseVoidOwner::ClearSkillUsageCache,
            WorldGameReleaseVoidOwner::ReleaseGoodsFactory,
        ] {
            context.release_void_owner(owner);
            events.push(WorldGameReleaseEvent::VoidOwner(owner));
        }

        // `db_data` и save serialization являются Rust owners; после этой
        // позиции Release к ним больше не обращается, фактический Drop — DeleteGame.
        events.push(WorldGameReleaseEvent::RustLocksRetired);
        for owner in [
            WorldGameReleaseVoidOwner::CleanupSocket,
            WorldGameReleaseVoidOwner::ReleaseBaseMessage,
            WorldGameReleaseVoidOwner::ReleaseNetSessionManager,
            WorldGameReleaseVoidOwner::RequestWriteLogWorkerExit,
        ] {
            context.release_void_owner(owner);
            events.push(WorldGameReleaseEvent::VoidOwner(owner));
        }

        let previous_handle = context.join_write_log_worker();
        events.push(WorldGameReleaseEvent::WriteLogWorkerJoined { previous_handle });
        let workers = context.stop_player_load_workers();
        events.push(WorldGameReleaseEvent::PlayerLoadWorkersStopped { workers });

        for owner in [
            WorldGameReleaseVoidOwner::UninitializeLargess,
            WorldGameReleaseVoidOwner::UninitializeDatabaseLayer,
        ] {
            context.release_void_owner(owner);
            events.push(WorldGameReleaseEvent::VoidOwner(owner));
        }
        let owner = WorldGameReleaseOptionalOwner::DefaultClientResource;
        let released = context.release_optional_owner(owner);
        events.push(WorldGameReleaseEvent::OptionalOwner { owner, released });
        let owner = WorldGameReleaseOptionalOwner::DupliRegionSetup;
        let released = self.dupli_region_setup.take().is_some();
        events.push(WorldGameReleaseEvent::OptionalOwner { owner, released });

        context.put_debug_string(b"WorldServer Exited!");
        events.push(WorldGameReleaseEvent::DebugPublished(
            b"WorldServer Exited!",
        ));
        Ok(WorldGameReleaseReport {
            events,
            legacy_result: 1,
        })
    }

    /// Создаёт и публикует World listener-owner, затем применяет setup.
    ///
    /// Ошибка `Host` оставляет новый owner опубликованным. Отсутствующее позднее
    /// поле возвращается только после успешного listen и hostname-resolution,
    /// то есть уже выполненные исходные побочные эффекты не откатываются.
    pub(crate) fn init_net_server(&mut self) -> Result<(), WorldNetworkInitializationError> {
        let server = CMyNetServer::new(legacy_tick_ms());
        let listen_port =
            self.setup
                .listen_port
                .ok_or(WorldNetworkInitializationError::MissingSetupField(
                    "dwListenPort",
                ))?;

        // RVA 0x00001860 публиковал s_pNetServer до проверки результата Host.
        self.net_server = Some(server);
        let server = self
            .net_server
            .as_mut()
            .expect("World network owner только что опубликован");
        server
            .host(listen_port, None, DEFAULT_SOCKET_TYPE, true)
            .map_err(WorldNetworkInitializationError::Host)?;

        if let Some(address) = resolve_first_local_ipv4() {
            let dotted = address.to_string();
            server.set_local_identity(dotted.as_bytes(), legacy_ipv4_word(address));
        }

        let config = self.setup.network_config_after_host()?;
        server.configure_after_host(
            config.check_net,
            config.maximum_io_sends,
            config.maximum_byte_count,
            config.maximum_connections,
            config.check_message_content,
            config.ban_ip_time_ms,
            config.maximum_message_length,
            config.maximum_client_send_buffer,
        );
        Ok(())
    }

    /// Пересоздаёт initial World-to-Login client и ставит регистрацию мира.
    ///
    /// Bind, resolution либо connect error выполняют исходный close/delete и
    /// оставляют `net_client = None`. Если безопасная граница `dwNumber`
    /// достигается уже после connect, опубликованный подключённый owner и
    /// включённый control-send не откатываются.
    pub(crate) async fn init_net_client(
        &mut self,
    ) -> Result<WorldClientInitialization, WorldClientInitializationError> {
        self.net_client.take();

        // RVA 0x000030F0 записывал s_pNetClient до Create(0, 0) и Connect.
        self.net_client = Some(CMyNetClient::new());
        let socket = bind_tcp_ipv4(None, 0);
        let login_port =
            self.setup
                .login_port
                .ok_or(WorldClientInitializationError::MissingSetupField(
                    "dwLoginPort",
                ))?;
        let endpoint = resolve_login_endpoint(&self.setup.login_ip, login_port)
            .map_err(WorldClientInitializationError::from);

        let socket = match socket {
            Ok(socket) => socket,
            Err(error) => {
                self.close_and_remove_net_client();
                return Err(WorldClientInitializationError::Bind(error));
            }
        };
        let endpoint = match endpoint {
            Ok(endpoint) => endpoint,
            Err(error) => {
                self.close_and_remove_net_client();
                return Err(error);
            }
        };

        let connect_result = self
            .net_client
            .as_mut()
            .expect("World client owner только что опубликован")
            .connect(socket, endpoint)
            .await;
        if let Err(error) = connect_result {
            self.close_and_remove_net_client();
            return Err(WorldClientInitializationError::Connect(error));
        }

        self.net_client
            .as_mut()
            .expect("успешно подключённый World client остаётся опубликованным")
            .enable_control_send();

        // BLOCKED_MISSING_FACT: при успешно открытом, но оборванном до первой
        // пары setup исходный `dwNumber` не инициализирован. Уже выполненные
        // connect/control-send не откатываем и неизвестный DWORD не выбираем.
        let world_number =
            self.setup
                .world_number
                .ok_or(WorldClientInitializationError::MissingSetupField(
                    "dwNumber",
                ))?;
        let mut registration = CMessage::new(0x0001_FE01);
        registration.base_mut().add_ulong(world_number);
        add_legacy_c_string(registration.base_mut(), &self.setup.name);
        let registration = registration.send(
            self.net_client.as_ref().map(CMyNetClient::send_queue),
            false,
        );

        Ok(WorldClientInitialization {
            endpoint,
            registration,
        })
    }

    /// Подключает новый LoginServer client и передаёт его World FIFO.
    ///
    /// Текущий `net_client` остаётся неизменным. Новый owner не получает
    /// control-send: его включит только будущая обработка typed handoff в
    /// исходной позиции `0x3FC03` после замены и постановки регистрации.
    pub(crate) async fn reconnect_login_server(
        &mut self,
    ) -> Result<WorldLoginReconnect, WorldLoginReconnectError> {
        // RVA 0x00003280 держал новый CMyNetClient только в локальном pointer.
        let mut client = CMyNetClient::new();
        let socket = match bind_tcp_ipv4(None, 0) {
            Ok(socket) => socket,
            Err(error) => {
                let _legacy_result = client.close();
                return Err(WorldLoginReconnectError::Bind(error));
            }
        };
        let login_port = match self.setup.login_port {
            Some(port) => port,
            None => {
                // BLOCKED_MISSING_FACT: старый dwLoginPort здесь был
                // неинициализирован; неизвестное значение не выбираем.
                let _legacy_result = client.close();
                return Err(WorldLoginReconnectError::MissingSetupField("dwLoginPort"));
            }
        };
        let endpoint = match resolve_login_endpoint(&self.setup.login_ip, login_port) {
            Ok(endpoint) => endpoint,
            Err(error) => {
                let _legacy_result = client.close();
                return Err(WorldLoginReconnectError::from(error));
            }
        };

        if let Err(error) = client.connect(socket, endpoint).await {
            let _legacy_result = client.close();
            return Err(WorldLoginReconnectError::Connect(error));
        }

        let server = match self.net_server.as_ref() {
            Some(server) => server,
            None => {
                // BLOCKED_MISSING_FACT: исходник после успешного connect
                // разыменовывал обязательный g_pGame->s_pNetServer. Safe Rust
                // закрывает ещё не опубликованный owner и не имитирует UB.
                let _legacy_result = client.close();
                return Err(WorldLoginReconnectError::MissingNetworkServerOwner);
            }
        };
        server.publish_reconnected_login_client(client);

        Ok(WorldLoginReconnect { endpoint })
    }

    /// Ставит LoginServer полный snapshot аккаунтов в порядке online-list.
    ///
    /// `Ok(None)` буквально соответствует nullable `s_pNetClient` и не создаёт
    /// сообщения. Ошибки безопасной границы возникают до единственного send;
    /// уже собранный локальный payload при этом, как и старый stack-owner, не
    /// становится наблюдаемым соседним процессом.
    pub(crate) fn send_cdkey_to_login_server(
        &self,
    ) -> Result<Option<WorldCdkeySnapshot>, WorldCdkeySnapshotError> {
        let Some(client) = self.net_client.as_ref() else {
            return Ok(None);
        };
        let world_number = self
            .setup
            .world_number
            .ok_or(WorldCdkeySnapshotError::MissingWorldNumber)?;
        let declared_online_players = u32::try_from(self.online_players.len()).map_err(|_| {
            WorldCdkeySnapshotError::OnlinePlayerCountOutsideLegacyRange {
                count: self.online_players.len(),
            }
        })?;

        let mut snapshot = CMessage::new(0x0001_FE02);
        snapshot.base_mut().add_ulong(world_number);
        snapshot.base_mut().add_ulong(declared_online_players);
        for &player_id in &self.online_players {
            // BLOCKED_MISSING_FACT: World RVA 0x000083D0 выполнял
            // При отсутствии узла используется ноль; иначе node->second, затем чтение со смещением 0x744.
            // Достижимость/реакция null-dereference не доказана; safe Rust не
            // отправляет частичный snapshot и не выдаёт эту ошибку за legacy.
            let player = self
                .players
                .get(&player_id)
                .ok_or(WorldCdkeySnapshotError::MissingPlayerOwner { player_id })?;
            add_legacy_c_string(snapshot.base_mut(), player.get_account());
        }
        let delivery = snapshot.send(Some(client.send_queue()), true);

        Ok(Some(WorldCdkeySnapshot {
            declared_online_players,
            delivery,
        }))
    }

    /// Выполняет полный `CGame::AI` в исходном порядке RVA `0x000148A0`.
    ///
    /// Ordered region map вызывает отдельного virtual owner-а только для
    /// ненулевого `pRegion`. После всего обхода снимается один общий broadcast
    /// tick; list cadence, оба random-вызова, wire-поля, send и последующие
    /// мутации сохраняют исходный порядок и wrapping 32-битную арифметику.
    pub(crate) fn ai<GetTick, Random>(
        &mut self,
        get_tick: &mut GetTick,
        random: &mut Random,
    ) -> WorldGameAiReport
    where
        GetTick: FnMut() -> u32,
        Random: FnMut(i32) -> i32,
    {
        let mut region_ids_run = Vec::new();
        for (&region_id, assignment) in &mut self.regions {
            let Some(region) = assignment.region.as_mut() else {
                continue;
            };
            region.ai();
            region_ids_run.push(region_id);
        }

        let broadcast_tick_ms = get_tick();
        let now_seconds = broadcast_tick_ms / 1000;
        let mut broadcasts = Vec::with_capacity(self.system_broadcasts.len());
        for index in 0..self.system_broadcasts.len() {
            let broadcast = &self.system_broadcasts[index];
            let elapsed_seconds = now_seconds.wrapping_sub(broadcast.last_notify_time_seconds);
            if broadcast.interval_seconds >= elapsed_seconds {
                broadcasts.push(WorldSystemBroadcastDisposition::Waiting {
                    elapsed_seconds,
                    interval_seconds: broadcast.interval_seconds,
                });
                continue;
            }

            let roll = random(100);
            if (roll as u32) >= broadcast.odds {
                broadcasts.push(WorldSystemBroadcastDisposition::OddsMissed {
                    roll,
                    odds: broadcast.odds,
                });
                continue;
            }

            let region_id = broadcast.region_id;
            let import_level = broadcast.import_level;
            let text_color = broadcast.text_color;
            let back_color = broadcast.back_color;
            let message_text = broadcast.message.clone();
            let min_time_seconds = broadcast.min_time_seconds;
            let max_time_seconds = broadcast.max_time_seconds;

            let mut message = CMessage::new(0x0007_FA03);
            message.base_mut().add_long(region_id);
            message.base_mut().add_long(import_level);
            message.base_mut().add_ulong(text_color);
            message.base_mut().add_ulong(back_color);
            add_legacy_c_string(message.base_mut(), &message_text);

            let target = if region_id == 0 {
                let sender = self.current_game_server_sender();
                WorldSystemBroadcastTarget::All {
                    delivery: message.send_all(sender.as_ref()),
                }
            } else {
                let game_server_index = self
                    .get_region_game_server(region_id)
                    .map(|game_server| game_server.index);
                let delivery = game_server_index.map(|game_server_index| {
                    let sender = self.current_game_server_sender();
                    message.send_to_map_id(sender.as_ref(), game_server_index as i32)
                });
                WorldSystemBroadcastTarget::Region {
                    region_id,
                    game_server_index,
                    delivery,
                }
            };

            let random_range = (max_time_seconds as i32).wrapping_sub(min_time_seconds as i32);
            let assigned_interval_seconds =
                random(random_range).wrapping_add(min_time_seconds as i32) as u32;
            let broadcast = &mut self.system_broadcasts[index];
            broadcast.last_notify_time_seconds = now_seconds;
            broadcast.interval_seconds = assigned_interval_seconds;
            broadcasts.push(WorldSystemBroadcastDisposition::Broadcast {
                roll,
                target,
                assigned_last_notify_time_seconds: now_seconds,
                assigned_interval_seconds,
            });
        }

        WorldGameAiReport {
            region_ids_run,
            broadcast_tick_ms,
            broadcasts,
            legacy_result: 1,
        }
    }

    /// Выполняет непосредственно окружающий `CGame::AI` profiling-порядок.
    ///
    /// Первый tick закрывает SavePoint-стадию, второй становится общим началом
    /// AI, внутренний tick принадлежит самому `CGame::AI`, четвёртый закрывает
    /// AI. Следующим исходным owner-ом остаётся готовый
    /// `process_message_main_loop_stage`.
    pub(crate) fn run_main_loop_ai_stage<GetTick, Random>(
        &mut self,
        clocks: &mut WorldMainLoopClockState,
        profile_state: &mut WorldMainLoopProfileState,
        mut get_tick: GetTick,
        mut random: Random,
    ) -> WorldMainLoopAiStageReport
    where
        GetTick: FnMut() -> u32,
        Random: FnMut(i32) -> i32,
    {
        let previous_stage_finished_at_ms = get_tick();
        let save_point_elapsed_ms =
            previous_stage_finished_at_ms.wrapping_sub(clocks.stage_started_at_ms);
        profile_state.ai_calls = profile_state.ai_calls.wrapping_add(1);
        profile_state.save_point_time_ms = profile_state
            .save_point_time_ms
            .wrapping_add(save_point_elapsed_ms);

        let ai_started_at_ms = get_tick();
        clocks.stage_started_at_ms = ai_started_at_ms;
        let ai = self.ai(&mut get_tick, &mut random);
        let ai_finished_at_ms = get_tick();
        let ai_elapsed_ms = ai_finished_at_ms.wrapping_sub(clocks.stage_started_at_ms);
        profile_state.ai_time_ms = profile_state.ai_time_ms.wrapping_add(ai_elapsed_ms);

        WorldMainLoopAiStageReport {
            previous_stage_finished_at_ms,
            save_point_elapsed_ms,
            accumulated_save_point_time_ms: profile_state.save_point_time_ms,
            ai_calls: profile_state.ai_calls,
            ai_started_at_ms,
            ai,
            ai_finished_at_ms,
            ai_elapsed_ms,
            accumulated_ai_time_ms: profile_state.ai_time_ms,
        }
    }

    /// Обрабатывает два FIFO в точных snapshot-границах RVA `0x00001A30`.
    ///
    /// Сначала фиксируется число server-событий. Только после их исчерпания
    /// заново читается текущий Login client и фиксируется его число сообщений.
    /// Поэтому typed reconnect из первой очереди заменяет owner до второго
    /// snapshot. Обычные сообщения проходят точный `Run` selector: готовые
    /// ветви server-owner-а, GMA `0x4FD01/0x4FD04/0x60401/0x60402`, полный GM
    /// owner `0x5FF01..0x5FF16`,
    /// player relay `0x5FC01..0x5FC04`, country relay `0x60310/0x60311`, other
    /// transport/cursor `0x5FD02/0x5FD06..0x5FD09/0x5FD0E`, goods-link
    /// `0x5FD03/0x5FD04`, copy-number
    /// `0x5FD0B`, LeiTing update `0x5FD10`, honor
    /// `0x5FD0C/0x5FD0D`, server `0x5FA01..=0x5FA07/0x5FA09/0x5FA0F/0x5FA10`,
    /// organizing session
    /// result, смерть faction-master-а `0x60101`, создание faction `0x60103`,
    /// initial organizing data `0x60104`, список faction страны `0x60107`,
    /// подача заявки `0x60108`,
    /// отмена заявки `0x60109`, решение по ней `0x6010A`, member/faction/union
    /// mutations `0x6010B..0x60113`, union application
    /// `0x60118`, leave-word enable `0x6011A`, запись
    /// `0x6011B`, её удаление `0x6011C`, объявление `0x6011D`, список целей
    /// войны `0x6011E`, само объявление `0x6011F`, общий leaf
    /// `0x60121/0x60123`, передача города `0x60130`, admission permit
    /// `0x60132`, terminal войны за город `0x60133`, заявка village-war
    /// `0x60135`, её result `0x60136`, city-war заявка `0x60137` и её result
    /// `0x60138`, Goods War command `0x60139`, faction-win `0x6013A` и player
    /// quest routes `0x6013B/0x6013C`, run-script `0x6013D` и faction parameter
    /// `0x6013E` и region-router request `0x60144` исполняются; остальные
    /// остаются owned pending. Terminal
    /// actions применяются FIFO до следующего сообщения. Async TDS lookup
    /// `0x5FF12` завершается до следующего slot-а, как синхронный ADO EXE;
    /// JJC owner `0x60901..0x60907` и Team owner `0x60001..0x6000C`
    /// исполняются полностью. Write-log owners `0x6020D/0x60214` ставят typed
    /// DB-command в FIFO и сразу публикуют запись в concrete increment/auction
    /// live-owner.
    pub(crate) async fn process_message<TimerCallback, JjcContext>(
        &mut self,
        honor_ranks: &mut CHonorRanks,
        increment_log: &mut CIncrementLog,
        auction_log: &mut CAuctionLog,
        organizing: &mut COrganizingCtrl,
        organizing_parameters: &COrganizingParam,
        country_handler: &mut CCountryHandler,
        country_parameters: &mut CCountryParam,
        player_list: &mut CPlayerList,
        country_war: &mut CountryWarSys,
        four_nation_war: &mut CFourNationWarSys,
        country_limits: CountryKingSaveLimits,
        faction_war_sys: &mut CFactionWarSys,
        attack_city: &mut CAttackCitySys,
        attack_city_callbacks: AttackCityCallbacks<TimerCallback>,
        globe_setup: &GlobeSetupSnapshot,
        region_router: &RegionRouter,
        village_war: &mut CVillageWarSys,
        goods_war: &mut CGoodsWarMember,
        timer: &mut CTimer<TimerCallback>,
        village_war_callbacks: VillageWarCallbacks<TimerCallback>,
        registry: &GoodsBasePropertiesRegistry,
        original_name_index: &GoodsOriginalNameIndex,
        coefficients: &PlayerPropertyCoefficients,
        load_player_largess: &mut dyn FnMut(&mut CPlayer),
        net_sessions: &CNetSessionManager,
        jjc: &mut CJJcSystem,
        jjc_config: JjcRunConfig,
        jjc_context: &mut JjcContext,
        application_runtime: &WorldUnionApplicationRuntimeOwner,
        application_callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
        check_invalid_organizing_string: &mut dyn FnMut(&mut Vec<u8>, bool) -> bool,
        faction_chat_log_enabled: bool,
        private_chat_log_enabled: bool,
        delete_log_enabled: bool,
        faction_create_log_enabled: bool,
        write_faction_create_log: &mut dyn FnMut(i32, &[u8], i32, &[u8]),
        faction_title_log_enabled: bool,
        write_faction_title_log:
            &mut dyn FnMut(i32, &[u8], &[u8], &[u8], i32, &[u8], i32, &[u8]),
        faction_purview_add_log_enabled: bool,
        faction_purview_revoke_log_enabled: bool,
        write_faction_purview_log:
            &mut dyn FnMut(i32, &[u8], i32, i32, &[u8], i32, &[u8], i32),
        faction_apply_log_enabled: bool,
        write_faction_apply_log:
            &mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
        faction_join_log_enabled: bool,
        write_faction_join_log:
            &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
        faction_quit_log_enabled: bool,
        write_faction_quit_log:
            &mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
        faction_fire_out_log_enabled: bool,
        write_faction_fire_out_log:
            &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
        faction_master_log_enabled: bool,
        write_faction_master_log:
            &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8]),
        faction_disband_log_enabled: bool,
        write_faction_disband_log: &mut dyn FnMut(i32, &[u8], i32, &[u8]),
        rs_player: &mut TiberiusRsPlayer,
        mut player_database: Option<&mut WorldTdsClient>,
        save_thread_handle: &mut WorldSaveThreadHandleState,
        launch_save_thread: &mut dyn FnMut(
            &WorldSaveThreadLaunchRequest,
        ) -> WorldSaveThreadHandleState,
        session_factory: &mut CSessionFactory,
        mut general_variables: Option<&mut CVariableList>,
        gods_battle: &mut CGodsBattleConf,
        mut rs_gods_battle: Option<&mut TiberiusRsGodsBattle>,
        mut gods_battle_database: Option<&mut WorldTdsClient>,
        reload_context: &mut dyn WorldReloadContext,
        add_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<WorldProcessMessageOutcome, WorldProcessMessageError>
    where
        TimerCallback: Copy,
        JjcContext: JjcRunContext + ?Sized,
    {
        let server_started_at = legacy_tick_ms();
        let mut server_remaining = self
            .net_server
            .as_ref()
            .ok_or(WorldProcessMessageError::MissingNetworkServerOwner)?
            .pending_events();
        let initial_server_events = server_remaining;
        let mut server_slots_visited = 0_i32;
        let mut events = Vec::new();

        while server_remaining > 0 {
            let event = self
                .net_server
                .as_ref()
                .expect("server-owner проверен до snapshot")
                .pop_received_event();
            if let Some(event) = event {
                match event {
                    WorldServerEvent::Message(message) => {
                        events.push(process_world_message(
                            self,
                            honor_ranks,
                            increment_log,
                            auction_log,
                            organizing,
                            organizing_parameters,
                            country_handler,
                            country_parameters,
                            player_list,
                            country_war,
                            four_nation_war,
                            country_limits,
                            faction_war_sys,
                            attack_city,
                            attack_city_callbacks,
                            globe_setup,
                            region_router,
                            village_war,
                            goods_war,
                            timer,
                            village_war_callbacks,
                            registry,
                            original_name_index,
                            coefficients,
                            &mut *load_player_largess,
                            net_sessions,
                            jjc,
                            jjc_config,
                            jjc_context,
                            application_runtime,
                            application_callbacks,
                            &mut *check_invalid_organizing_string,
                            faction_chat_log_enabled,
                            private_chat_log_enabled,
                            delete_log_enabled,
                            faction_create_log_enabled,
                            &mut *write_faction_create_log,
                            faction_title_log_enabled,
                            &mut *write_faction_title_log,
                            faction_purview_add_log_enabled,
                            faction_purview_revoke_log_enabled,
                            &mut *write_faction_purview_log,
                            faction_apply_log_enabled,
                            &mut *write_faction_apply_log,
                            faction_join_log_enabled,
                            &mut *write_faction_join_log,
                            faction_quit_log_enabled,
                            &mut *write_faction_quit_log,
                            faction_fire_out_log_enabled,
                            &mut *write_faction_fire_out_log,
                            faction_master_log_enabled,
                            &mut *write_faction_master_log,
                            faction_disband_log_enabled,
                            &mut *write_faction_disband_log,
                            &mut *rs_player,
                            player_database.as_deref_mut(),
                            &mut *save_thread_handle,
                            &mut *launch_save_thread,
                            &mut *session_factory,
                            general_variables.as_deref_mut(),
                            &mut *gods_battle,
                            rs_gods_battle.as_deref_mut(),
                            gods_battle_database.as_deref_mut(),
                            &mut *reload_context,
                            &mut *add_log_text,
                            update_player,
                            WorldMessageSource::GameServer,
                            message,
                        )
                        .await);
                    }
                    WorldServerEvent::LoginClientReconnected(client) => {
                        let replacement = on_login_client_reconnected(self, client)
                            .map_err(WorldProcessMessageError::ServerMessage)?;
                        events.push(ProcessedWorldEvent::LoginClientReconnected(replacement));
                    }
                }
            }
            server_remaining -= 1;
            server_slots_visited += 1;
        }
        self.game_server_message_time_ms = self
            .game_server_message_time_ms
            .wrapping_add(legacy_tick_ms().wrapping_sub(server_started_at));

        let login_started_at = legacy_tick_ms();
        let initial_login_messages = self.net_client.as_ref().map(CMyNetClient::pending_messages);
        let mut login_remaining = initial_login_messages.unwrap_or(0);
        let mut login_slots_visited = 0_i32;
        while login_remaining > 0 {
            let message = self
                .net_client
                .as_ref()
                .and_then(CMyNetClient::pop_received_message);
            if let Some(message) = message {
                events.push(process_world_message(
                    self,
                    honor_ranks,
                    increment_log,
                    auction_log,
                    organizing,
                    organizing_parameters,
                    country_handler,
                    country_parameters,
                    player_list,
                    country_war,
                    four_nation_war,
                    country_limits,
                    faction_war_sys,
                    attack_city,
                    attack_city_callbacks,
                    globe_setup,
                    region_router,
                    village_war,
                    goods_war,
                    timer,
                    village_war_callbacks,
                    registry,
                    original_name_index,
                    coefficients,
                    &mut *load_player_largess,
                    net_sessions,
                    jjc,
                    jjc_config,
                    jjc_context,
                    application_runtime,
                    application_callbacks,
                    &mut *check_invalid_organizing_string,
                    faction_chat_log_enabled,
                    private_chat_log_enabled,
                    delete_log_enabled,
                    faction_create_log_enabled,
                    &mut *write_faction_create_log,
                    faction_title_log_enabled,
                    &mut *write_faction_title_log,
                    faction_purview_add_log_enabled,
                    faction_purview_revoke_log_enabled,
                    &mut *write_faction_purview_log,
                    faction_apply_log_enabled,
                    &mut *write_faction_apply_log,
                    faction_join_log_enabled,
                    &mut *write_faction_join_log,
                    faction_quit_log_enabled,
                    &mut *write_faction_quit_log,
                    faction_fire_out_log_enabled,
                    &mut *write_faction_fire_out_log,
                    faction_master_log_enabled,
                    &mut *write_faction_master_log,
                    faction_disband_log_enabled,
                    &mut *write_faction_disband_log,
                    &mut *rs_player,
                    player_database.as_deref_mut(),
                    &mut *save_thread_handle,
                    &mut *launch_save_thread,
                    &mut *session_factory,
                    general_variables.as_deref_mut(),
                    &mut *gods_battle,
                    rs_gods_battle.as_deref_mut(),
                    gods_battle_database.as_deref_mut(),
                    &mut *reload_context,
                    &mut *add_log_text,
                    update_player,
                    WorldMessageSource::LoginServer,
                    message,
                )
                .await);
            }
            login_remaining -= 1;
            login_slots_visited += 1;
        }
        self.login_server_message_time_ms = self
            .login_server_message_time_ms
            .wrapping_add(legacy_tick_ms().wrapping_sub(login_started_at));

        Ok(WorldProcessMessageOutcome {
            legacy_result: 1,
            initial_server_events,
            initial_login_messages,
            server_slots_visited,
            login_slots_visited,
            events,
            game_server_message_time_ms: self.game_server_message_time_ms,
            login_server_message_time_ms: self.login_server_message_time_ms,
        })
    }

    /// Выполняет внешний MainLoop profiling call-site `ProcessMessage`.
    ///
    /// Safe block не получает придуманных end/next-stage ticks и не меняет
    /// накопитель: исходный невозвратившийся путь их не достигал.
    pub(crate) async fn process_message_main_loop_stage<
        TimerCallback,
        JjcContext,
        GetTick,
    >(
        &mut self,
        honor_ranks: &mut CHonorRanks,
        increment_log: &mut CIncrementLog,
        auction_log: &mut CAuctionLog,
        organizing: &mut COrganizingCtrl,
        organizing_parameters: &COrganizingParam,
        country_handler: &mut CCountryHandler,
        country_parameters: &mut CCountryParam,
        player_list: &mut CPlayerList,
        country_war: &mut CountryWarSys,
        four_nation_war: &mut CFourNationWarSys,
        country_limits: CountryKingSaveLimits,
        faction_war_sys: &mut CFactionWarSys,
        attack_city: &mut CAttackCitySys,
        attack_city_callbacks: AttackCityCallbacks<TimerCallback>,
        globe_setup: &GlobeSetupSnapshot,
        region_router: &RegionRouter,
        village_war: &mut CVillageWarSys,
        goods_war: &mut CGoodsWarMember,
        timer: &mut CTimer<TimerCallback>,
        village_war_callbacks: VillageWarCallbacks<TimerCallback>,
        registry: &GoodsBasePropertiesRegistry,
        original_name_index: &GoodsOriginalNameIndex,
        coefficients: &PlayerPropertyCoefficients,
        load_player_largess: &mut dyn FnMut(&mut CPlayer),
        net_sessions: &CNetSessionManager,
        jjc: &mut CJJcSystem,
        jjc_config: JjcRunConfig,
        jjc_context: &mut JjcContext,
        application_runtime: &WorldUnionApplicationRuntimeOwner,
        application_callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
        check_invalid_organizing_string: &mut dyn FnMut(&mut Vec<u8>, bool) -> bool,
        faction_chat_log_enabled: bool,
        private_chat_log_enabled: bool,
        delete_log_enabled: bool,
        faction_create_log_enabled: bool,
        write_faction_create_log: &mut dyn FnMut(i32, &[u8], i32, &[u8]),
        faction_title_log_enabled: bool,
        write_faction_title_log:
            &mut dyn FnMut(i32, &[u8], &[u8], &[u8], i32, &[u8], i32, &[u8]),
        faction_purview_add_log_enabled: bool,
        faction_purview_revoke_log_enabled: bool,
        write_faction_purview_log:
            &mut dyn FnMut(i32, &[u8], i32, i32, &[u8], i32, &[u8], i32),
        faction_apply_log_enabled: bool,
        write_faction_apply_log:
            &mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
        faction_join_log_enabled: bool,
        write_faction_join_log:
            &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
        faction_quit_log_enabled: bool,
        write_faction_quit_log:
            &mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
        faction_fire_out_log_enabled: bool,
        write_faction_fire_out_log:
            &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
        faction_master_log_enabled: bool,
        write_faction_master_log:
            &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8]),
        faction_disband_log_enabled: bool,
        write_faction_disband_log: &mut dyn FnMut(i32, &[u8], i32, &[u8]),
        rs_player: &mut TiberiusRsPlayer,
        player_database: Option<&mut WorldTdsClient>,
        save_thread_handle: &mut WorldSaveThreadHandleState,
        launch_save_thread: &mut dyn FnMut(
            &WorldSaveThreadLaunchRequest,
        ) -> WorldSaveThreadHandleState,
        session_factory: &mut CSessionFactory,
        general_variables: Option<&mut CVariableList>,
        gods_battle: &mut CGodsBattleConf,
        rs_gods_battle: Option<&mut TiberiusRsGodsBattle>,
        gods_battle_database: Option<&mut WorldTdsClient>,
        reload_context: &mut dyn WorldReloadContext,
        log: &mut WorldLogTextOwner,
        get_log_local_time: &mut dyn FnMut() -> WorldLogLocalTime,
        put_log_info: &mut dyn FnMut(&[u8]),
        update_player: &mut dyn FnMut(i32),
        clocks: &mut WorldMainLoopClockState,
        state: &mut WorldProcessMessageStageState,
        mut get_tick: GetTick,
    ) -> WorldProcessMessageStageReport
    where
        TimerCallback: Copy,
        JjcContext: JjcRunContext + ?Sized,
        GetTick: FnMut() -> u32,
    {
        let started_at_ms = get_tick();
        let save_info_time_ms = self.setup.save_info_time_ms;
        let mut add_log_text = |message: &[u8]| {
            log.add_log_text(
                message,
                save_info_time_ms,
                &mut get_tick,
                &mut *get_log_local_time,
                &mut *put_log_info,
            )
        };
        let outcome = match self.process_message(
            honor_ranks,
            increment_log,
            auction_log,
            organizing,
            organizing_parameters,
            country_handler,
            country_parameters,
            player_list,
            country_war,
            four_nation_war,
            country_limits,
            faction_war_sys,
            attack_city,
            attack_city_callbacks,
            globe_setup,
            region_router,
            village_war,
            goods_war,
            timer,
            village_war_callbacks,
            registry,
            original_name_index,
            coefficients,
            load_player_largess,
            net_sessions,
            jjc,
            jjc_config,
            jjc_context,
            application_runtime,
            application_callbacks,
            check_invalid_organizing_string,
            faction_chat_log_enabled,
            private_chat_log_enabled,
            delete_log_enabled,
            faction_create_log_enabled,
            write_faction_create_log,
            faction_title_log_enabled,
            write_faction_title_log,
            faction_purview_add_log_enabled,
            faction_purview_revoke_log_enabled,
            write_faction_purview_log,
            faction_apply_log_enabled,
            write_faction_apply_log,
            faction_join_log_enabled,
            write_faction_join_log,
            faction_quit_log_enabled,
            write_faction_quit_log,
            faction_fire_out_log_enabled,
            write_faction_fire_out_log,
            faction_master_log_enabled,
            write_faction_master_log,
            faction_disband_log_enabled,
            write_faction_disband_log,
            rs_player,
            player_database,
            save_thread_handle,
            launch_save_thread,
            session_factory,
            general_variables,
            gods_battle,
            rs_gods_battle,
            gods_battle_database,
            reload_context,
            &mut add_log_text,
            update_player,
        )
        .await
        {
            Ok(outcome) => outcome,
            Err(error) => {
                return WorldProcessMessageStageReport::Blocked {
                    started_at_ms,
                    error,
                };
            }
        };
        drop(add_log_text);
        let finished_at_ms = get_tick();
        let elapsed_ms = finished_at_ms.wrapping_sub(started_at_ms);
        state.accumulated_time_ms = state.accumulated_time_ms.wrapping_add(elapsed_ms);
        let next_stage_started_at_ms = get_tick();
        clocks.stage_started_at_ms = next_stage_started_at_ms;
        WorldProcessMessageStageReport::Complete {
            started_at_ms,
            outcome,
            finished_at_ms,
            elapsed_ms,
            accumulated_time_ms: state.accumulated_time_ms,
            next_stage_started_at_ms,
        }
    }

    /// Выполняет следующий MainLoop owner после `ProcessMessage`.
    ///
    /// Входной shared tick уже назначен успешным
    /// `process_message_main_loop_stage`; первый новый tick закрывает
    /// `CSessionFactory::AI` в `DAT_0056e528`, второй начинает соседний
    /// готовый `ProcessPlayerDataQueue`.
    pub(crate) fn run_main_loop_session_factory_stage<GetTick>(
        &mut self,
        factory: &mut CSessionFactory,
        clocks: &mut WorldMainLoopClockState,
        profile_state: &mut WorldMainLoopProfileState,
        mut get_tick: GetTick,
    ) -> WorldMainLoopSessionFactoryStageReport
    where
        GetTick: FnMut() -> u32,
    {
        let ai = factory.ai(self);
        let finished_at_ms = get_tick();
        let elapsed_ms = finished_at_ms.wrapping_sub(clocks.stage_started_at_ms);
        profile_state.session_factory_time_ms = profile_state
            .session_factory_time_ms
            .wrapping_add(elapsed_ms);
        let next_stage_started_at_ms = get_tick();
        clocks.stage_started_at_ms = next_stage_started_at_ms;
        WorldMainLoopSessionFactoryStageReport {
            ai,
            finished_at_ms,
            elapsed_ms,
            accumulated_time_ms: profile_state.session_factory_time_ms,
            next_stage_started_at_ms,
        }
    }

    fn update_detached_player_friends(
        &self,
        player: &mut CPlayer,
    ) -> Vec<WorldFriendPresenceUpdate> {
        let mut updates = Vec::with_capacity(player.friend_count());
        for friend_index in 0..player.friend_count() {
            let friend_name = player
                .friend_name(friend_index)
                .expect("friend index получен из текущего len")
                .to_vec();
            let mut friend_player_id = self.online_player_id_by_name(&friend_name);
            if friend_player_id == 0 {
                friend_player_id = self.login_player_id_by_name(&friend_name);
            }
            let online = friend_player_id != 0;
            let updated = player.set_friend_online(friend_index, online);
            debug_assert!(updated, "friend index не менялся между read и write");
            updates.push(self.send_friend_presence_update(
                friend_index,
                friend_player_id,
                online,
                player.get_name(),
            ));
        }
        updates
    }

    fn update_published_player_friends(
        &mut self,
        player_id: u32,
    ) -> Vec<WorldFriendPresenceUpdate> {
        let friend_count = self
            .map_player(player_id)
            .map_or(0, CPlayer::friend_count);
        let mut updates = Vec::with_capacity(friend_count);
        for friend_index in 0..friend_count {
            let (friend_name, player_name) = {
                let player = self
                    .map_player(player_id)
                    .expect("direct route уже опубликовал selected player");
                (
                    player
                        .friend_name(friend_index)
                        .expect("friend index получен из текущего len")
                        .to_vec(),
                    player.get_name().to_vec(),
                )
            };
            let mut friend_player_id = self.online_player_id_by_name(&friend_name);
            if friend_player_id == 0 {
                friend_player_id = self.login_player_id_by_name(&friend_name);
            }
            let online = friend_player_id != 0;
            let updated = self
                .players
                .get_mut(&player_id)
                .expect("selected player остаётся опубликованным")
                .set_friend_online(friend_index, online);
            debug_assert!(updated, "friend index не менялся между read и write");
            updates.push(self.send_friend_presence_update(
                friend_index,
                friend_player_id,
                online,
                &player_name,
            ));
        }
        updates
    }

    fn send_friend_presence_update(
        &self,
        friend_index: usize,
        friend_player_id: u32,
        online: bool,
        player_name: &[u8],
    ) -> WorldFriendPresenceUpdate {
        if !online {
            return WorldFriendPresenceUpdate {
                friend_index,
                player_id: 0,
                online: false,
                target_game_server_index: None,
                delivery: None,
            };
        }
        let target_game_server_index = self
            .online_player_by_id(friend_player_id)
            .and_then(|friend| self.get_region_game_server(friend.get_region_id()))
            .map(|game_server| game_server.index);
        let mut presence = CMessage::new(0x0007_F904);
        presence.base_mut().add_ulong(friend_player_id);
        add_legacy_c_string(presence.base_mut(), player_name);
        let sender = self.current_game_server_sender();
        let delivery = presence.send_to_map_id(
            sender.as_ref(),
            target_game_server_index.unwrap_or(0) as i32,
        );
        WorldFriendPresenceUpdate {
            friend_index,
            player_id: friend_player_id,
            online: true,
            target_game_server_index,
            delivery: Some(delivery),
        }
    }

    fn publish_loaded_player<GetTick>(
        &mut self,
        organizing_ctrl: &mut COrganizingCtrl,
        player_id: u32,
        player: Box<CPlayer>,
        mut get_tick: GetTick,
    ) -> (WorldOnlinePlayerRemoveOutcome, bool, u32)
    where
        GetTick: FnMut() -> u32,
    {
        let online_removal = self.remove_online_player(organizing_ctrl, player_id);
        self.remove_offline_player(player_id);
        let login_time_ms = get_tick();
        self.append_login_player(player_id, login_time_ms);
        let replaced_existing_player = self.delete_map_player(player_id);
        let append_outcome = self.append_map_player(player, |_| {});
        let WorldMapPlayerAppendOutcome::Inserted {
            player_id: inserted_player_id,
        } = append_outcome
        else {
            unreachable!("map key удалён непосредственно перед AppendMapPlayer")
        };
        debug_assert_eq!(inserted_player_id, player_id);
        (
            online_removal,
            replaced_existing_player,
            login_time_ms,
        )
    }

    /// Проводит уже полученного player-owner-а по общей direct/DB-load цепочке.
    #[allow(
        clippy::too_many_arguments,
        reason = "queue metadata сохраняет exact diagnostic outcome producer-а"
    )]
    pub(crate) fn route_loaded_player<GetTick>(
        &mut self,
        organizing_ctrl: &mut COrganizingCtrl,
        initial_size: u32,
        null_pops: u32,
        queue_player_id: u32,
        client_ip: u32,
        cdkey: &[u8],
        player: Option<Box<CPlayer>>,
        route_order: WorldLoadedPlayerRouteOrder,
        after_login_send: &mut dyn FnMut(&mut CPlayer),
        mut get_tick: GetTick,
    ) -> Result<WorldProcessPlayerDataQueueOutcome, WorldProcessPlayerDataQueueError>
    where
        GetTick: FnMut() -> u32,
    {
        let Some(mut player) = player else {
            let login_delivery = self.send_player_data_queue_rejection(cdkey);
            return Ok(WorldProcessPlayerDataQueueOutcome::Rejected {
                initial_size,
                null_pops,
                queue_player_id,
                client_ip,
                reason: WorldPlayerDataQueueRejectReason::NullPlayer,
                login_delivery,
            });
        };

        let player_id = player.get_id() as u32;
        let region_types = self.player_organizing_region_types();
        let organizing_result = {
            let mut updater = organizing_ctrl.player_updater(&region_types);
            player.set_player_organizing(&mut updater)
        };
        if let Err(error) = organizing_result {
            return Err(WorldProcessPlayerDataQueueError {
                initial_size,
                null_pops,
                player_id,
                block: WorldProcessPlayerDataQueueBlock::Organizing(error),
            });
        }

        let region_id = player.get_region_id();
        let Some(game_server_index) = self
            .region(region_id)
            .map(|region| region.game_server_index)
        else {
            let login_delivery = self.send_player_data_queue_rejection(cdkey);
            drop(player);
            return Ok(WorldProcessPlayerDataQueueOutcome::Rejected {
                initial_size,
                null_pops,
                queue_player_id,
                client_ip,
                reason: WorldPlayerDataQueueRejectReason::MissingRegion { region_id },
                login_delivery,
            });
        };

        // CPlayer vtable +0x84 = exact `CShape::SetState(0)`.
        player.set_state(0);
        let Some(game_server) = self
            .game_server(game_server_index)
            .filter(|game_server| game_server.connected)
        else {
            let login_delivery = self.send_player_data_queue_rejection(cdkey);
            drop(player);
            return Ok(WorldProcessPlayerDataQueueOutcome::Rejected {
                initial_size,
                null_pops,
                queue_player_id,
                client_ip,
                reason: WorldPlayerDataQueueRejectReason::MissingOrDisconnectedGameServer {
                    region_id,
                    game_server_index,
                },
                login_delivery,
            });
        };

        let game_server_ip = game_server.ip.clone();
        let game_server_port = game_server.port.ok_or(WorldProcessPlayerDataQueueError {
            initial_size,
            null_pops,
            player_id,
            block: WorldProcessPlayerDataQueueBlock::UninitializedGameServerPort {
                game_server_index,
            },
        })?;

        let mut login_reply = CMessage::new(0x0001_FF01);
        login_reply.base_mut().add_byte(0x1D);
        add_legacy_c_string(login_reply.base_mut(), cdkey);
        add_legacy_c_string(login_reply.base_mut(), &game_server_ip);
        login_reply.base_mut().add_ulong(game_server_port);
        add_legacy_c_string(login_reply.base_mut(), player.get_name());
        login_reply.base_mut().add_byte(player.get_level());
        let login_delivery = login_reply.send(
            self.current_login_client().map(CMyNetClient::send_queue),
            false,
        );
        after_login_send(&mut player);

        let (friend_updates, online_removal, replaced_existing_player, login_time_ms) =
            match route_order {
                WorldLoadedPlayerRouteOrder::LoadedQueue => {
                    let friend_updates = self.update_detached_player_friends(&mut player);
                    let (online_removal, replaced_existing_player, login_time_ms) = self
                        .publish_loaded_player(
                            organizing_ctrl,
                            player_id,
                            player,
                            &mut get_tick,
                        );
                    (
                        friend_updates,
                        online_removal,
                        replaced_existing_player,
                        login_time_ms,
                    )
                }
                WorldLoadedPlayerRouteOrder::Direct => {
                    let (online_removal, replaced_existing_player, login_time_ms) = self
                        .publish_loaded_player(
                            organizing_ctrl,
                            player_id,
                            player,
                            &mut get_tick,
                        );
                    let friend_updates = self.update_published_player_friends(player_id);
                    self.players
                        .get_mut(&player_id)
                        .expect("direct route сохраняет опубликованного player-owner-а")
                        .reset_selected_login_flags();
                    (
                        friend_updates,
                        online_removal,
                        replaced_existing_player,
                        login_time_ms,
                    )
                }
            };

        Ok(WorldProcessPlayerDataQueueOutcome::Accepted {
            initial_size,
            null_pops,
            player_id,
            client_ip,
            game_server_index,
            login_delivery,
            friend_updates,
            online_removal,
            replaced_existing_player,
            login_time_ms,
        })
    }

    /// Обрабатывает не более одного non-null record-а из начального snapshot.
    ///
    /// Null-pop не завершает метод: он уменьшает только сохранённый snapshot и
    /// повторяет pop. Любая фактически извлечённая запись проходит ровно одну
    /// reject либо success цепочку и затем безусловно завершает вызов.
    pub(crate) fn process_player_data_queue<GetTick>(
        &mut self,
        organizing_ctrl: &mut COrganizingCtrl,
        mut get_tick: GetTick,
    ) -> Result<WorldProcessPlayerDataQueueOutcome, WorldProcessPlayerDataQueueError>
    where
        GetTick: FnMut() -> u32,
    {
        let initial_size = self.player_data_queue.get_size();
        let mut remaining = initial_size;
        let mut null_pops = 0_u32;

        while remaining != 0 {
            let Some(mut entry) = self.player_data_queue.pop_player_data() else {
                remaining = remaining.wrapping_sub(1);
                null_pops = null_pops.wrapping_add(1);
                continue;
            };

            let queue_player_id = entry.player_id();
            let client_ip = entry.client_ip();
            let Some(cdkey) = entry.cdkey().map(<[u8]>::to_vec) else {
                return Err(WorldProcessPlayerDataQueueError {
                    initial_size,
                    null_pops,
                    player_id: queue_player_id,
                    block: WorldProcessPlayerDataQueueBlock::UnterminatedCdkey,
                });
            };
            let mut after_login_send = |_player: &mut CPlayer| {};
            return self.route_loaded_player(
                organizing_ctrl,
                initial_size,
                null_pops,
                queue_player_id,
                client_ip,
                &cdkey,
                entry.take_player(),
                WorldLoadedPlayerRouteOrder::LoadedQueue,
                &mut after_login_send,
                &mut get_tick,
            );
        }

        Ok(WorldProcessPlayerDataQueueOutcome::NoRecord {
            initial_size,
            null_pops,
        })
    }

    /// Закрывает `DAT_0056e524` и назначает shared start следующего `CTimer::Run`.
    pub(crate) fn run_main_loop_player_data_queue_stage<GetTick>(
        &mut self,
        organizing_ctrl: &mut COrganizingCtrl,
        clocks: &mut WorldMainLoopClockState,
        profile_state: &mut WorldMainLoopProfileState,
        mut get_tick: GetTick,
    ) -> WorldMainLoopPlayerDataQueueStageReport
    where
        GetTick: FnMut() -> u32,
    {
        let outcome = match self.process_player_data_queue(organizing_ctrl, &mut get_tick) {
            Ok(outcome) => outcome,
            Err(error) => {
                return WorldMainLoopPlayerDataQueueStageReport::Blocked { error };
            }
        };
        let finished_at_ms = get_tick();
        let elapsed_ms = finished_at_ms.wrapping_sub(clocks.stage_started_at_ms);
        profile_state.process_player_data_queue_time_ms = profile_state
            .process_player_data_queue_time_ms
            .wrapping_add(elapsed_ms);
        let next_stage_started_at_ms = get_tick();
        clocks.stage_started_at_ms = next_stage_started_at_ms;
        WorldMainLoopPlayerDataQueueStageReport::Complete {
            outcome,
            finished_at_ms,
            elapsed_ms,
            accumulated_time_ms: profile_state.process_player_data_queue_time_ms,
            next_stage_started_at_ms,
        }
    }

    /// Выполняет `CTimer::Run`, закрывает `DAT_0056e520` и начинает faction-war стадию.
    ///
    /// `profile_state.ai_calls` является текущим `CGame::s_lAITick`. Timer
    /// получает тот же tick-provider, которым затем MainLoop закрывает стадию
    /// и отдельно назначает shared start для сырого `CFactionWarSys::Run`.
    #[allow(
        clippy::too_many_arguments,
        reason = "timer callback сохраняет явные DB, ranking, clock и log owners"
    )]
    pub(crate) async fn run_main_loop_timer_stage<
        Callback,
        GetTick,
        GetTimerLocalTime,
        Dispatch,
    >(
        &self,
        timer: &mut CTimer<Callback>,
        country_war: &mut CountryWarSys,
        country_handler: &mut CCountryHandler,
        country_war_callbacks: CountryWarCallbacks<Callback>,
        globe_setup: &GlobeSetupSnapshot,
        clocks: &mut WorldMainLoopClockState,
        profile_state: &mut WorldMainLoopProfileState,
        copy_number_timer: &mut CopyNumberTimerState,
        organizing_parameters: &mut COrganizingParam,
        player_ranks: &mut CPlayerRanks,
        rs_player: &mut TiberiusRsPlayer,
        player_database: Option<&mut WorldTdsClient>,
        organizing: &COrganizingCtrl,
        log: &mut WorldLogTextOwner,
        get_tick: &mut GetTick,
        get_timer_local_time: &mut GetTimerLocalTime,
        get_log_local_time: &mut dyn FnMut() -> WorldLogLocalTime,
        put_log_info: &mut dyn FnMut(&[u8]),
        world_string_by_id: &mut dyn FnMut(&[u8]) -> Vec<u8>,
        format_world_string:
            &mut dyn FnMut(&[u8], &[UnionFormatArgument<'_>]) -> Vec<u8>,
        dispatch: &mut Dispatch,
    ) -> Result<WorldMainLoopTimerStageReport, WorldMainLoopTimerStageBlock>
    where
        Callback: Copy + PartialEq,
        GetTick: FnMut() -> u32 + ?Sized,
        GetTimerLocalTime: FnMut() -> TagTime + ?Sized,
        Dispatch: FnMut(&mut CTimer<Callback>, TimerCallbackInvocation<Callback>) + ?Sized,
    {
        let mut handler = WorldTimerHandler {
            game: self,
            country_war,
            country_handler,
            country_war_callbacks,
            globe_setup,
            organizing_parameters,
            player_ranks,
            rs_player,
            player_database,
            organizing,
            copy_number_timer,
            log,
            get_log_local_time,
            put_log_info,
            world_string_by_id,
            format_world_string,
            copy_number_resets: Vec::new(),
            refreshes: Vec::new(),
            tax_refreshes: Vec::new(),
            country_wars: Vec::new(),
            pending_copy_number_registration: None,
            pending_player_ranks_registration: None,
            pending_tax_registration: None,
        };
        let timer_report = timer
            .run_with_async_handler(
                profile_state.ai_calls,
                &mut *get_tick,
                &mut *get_timer_local_time,
                &mut handler,
                &mut *dispatch,
            )
            .await;
        let timer_report = match timer_report {
            Ok(timer_report) => timer_report,
            Err(AsyncTimerRunBlock { timer, source }) => {
                return Err(WorldMainLoopTimerStageBlock { timer, source });
            }
        };
        let copy_number_resets = handler.copy_number_resets;
        let player_ranks = handler.refreshes;
        let organizing_taxes = handler.tax_refreshes;
        let country_wars = handler.country_wars;
        let finished_at_ms = get_tick();
        let elapsed_ms = finished_at_ms.wrapping_sub(clocks.stage_started_at_ms);
        profile_state.timer_time_ms = profile_state.timer_time_ms.wrapping_add(elapsed_ms);
        let next_stage_started_at_ms = get_tick();
        clocks.stage_started_at_ms = next_stage_started_at_ms;
        Ok(WorldMainLoopTimerStageReport {
            timer: timer_report,
            copy_number_resets,
            player_ranks,
            organizing_taxes,
            country_wars,
            finished_at_ms,
            elapsed_ms,
            accumulated_time_ms: profile_state.timer_time_ms,
            next_stage_started_at_ms,
        })
    }

    /// Выполняет `CFactionWarSys::Run` и закрывает `DAT_0056e51c`.
    ///
    /// Raw MainLoop не назначает новый shared start перед следующим
    /// `CLeiTing::Run`, поэтому этот call-site делает только один end tick и
    /// оставляет `clocks.stage_started_at_ms` без изменения.
    pub(crate) fn run_main_loop_faction_war_stage<Context, GetTick>(
        &self,
        faction_war_sys: &mut CFactionWarSys,
        context: &mut Context,
        clocks: &WorldMainLoopClockState,
        profile_state: &mut WorldMainLoopProfileState,
        mut get_tick: GetTick,
    ) -> Result<WorldMainLoopFactionWarStageReport, FactionWarStopBlock<Context::Block>>
    where
        Context: FactionWarStopContext,
        GetTick: FnMut() -> u32,
    {
        let faction_war = faction_war_sys.run(context, &mut get_tick)?;
        let finished_at_ms = get_tick();
        let elapsed_ms = finished_at_ms.wrapping_sub(clocks.stage_started_at_ms);
        profile_state.faction_war_time_ms =
            profile_state.faction_war_time_ms.wrapping_add(elapsed_ms);
        Ok(WorldMainLoopFactionWarStageReport {
            faction_war,
            finished_at_ms,
            elapsed_ms,
            accumulated_time_ms: profile_state.faction_war_time_ms,
        })
    }

    /// Выполняет следующий непрофилированный MainLoop owner `CLeiTing::Run`.
    pub(crate) fn run_main_loop_lei_ting_stage<Context, GetLocalTime>(
        &mut self,
        lei_ting: &mut CLeiTing,
        globe_setup: &GlobeSetupSnapshot,
        context: &mut Context,
        mut get_local_time: GetLocalTime,
    ) -> Result<LeiTingRunReport, LeiTingBlock<Context::Block>>
    where
        Context: LeiTingContext,
        GetLocalTime: FnMut() -> LeiTingLocalTime,
    {
        let current = get_local_time();
        lei_ting.run(current, self, globe_setup, context)
    }

    /// Выполняет соседний `DoneOutList -> Pop/DoneListIn -> LoadAuction` batch.
    ///
    /// Между `CLeiTing::Run` и этими тремя owners исходный MainLoop не снимал
    /// tick. Единственный clock-call после `LoadAuction` назначает shared start
    /// следующей пока сырой стадии `CNetSessionManager::Run`.
    pub(crate) fn run_main_loop_db_misc_stage<Context, GetTick>(
        &self,
        db_misc: &mut CDbMisc,
        context: &mut Context,
        clocks: &mut WorldMainLoopClockState,
        mut get_tick: GetTick,
    ) -> Result<WorldMainLoopDbMiscStageReport, DbMiscDoneOutBlock>
    where
        Context: DbMiscContext,
        GetTick: FnMut() -> u32,
    {
        let output = db_misc.done_out_list(context)?;
        let input_notes = db_misc.pop_item_from_list_in(context, 0);
        let input = db_misc.done_list_in(context, input_notes);
        let auction = db_misc.load_auction(context);
        let next_stage_started_at_ms = get_tick();
        clocks.stage_started_at_ms = next_stage_started_at_ms;
        Ok(WorldMainLoopDbMiscStageReport {
            output,
            input,
            auction,
            next_stage_started_at_ms,
        })
    }

    /// Выполняет полный session timeout pass, применяет его terminal actions и
    /// закрывает `DAT_0056e518`.
    ///
    /// Следующий участок MainLoop начинает проверку ping без нового shared
    /// start tick, поэтому `clocks.stage_started_at_ms` здесь не меняется.
    pub(crate) fn run_main_loop_net_session_stage<GetTick>(
        &self,
        manager: &CNetSessionManager,
        organizing: &mut COrganizingCtrl,
        organizing_parameters: &COrganizingParam,
        application_runtime: &WorldUnionApplicationRuntimeOwner,
        application_callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
        update_player: &mut dyn FnMut(i32),
        clocks: &WorldMainLoopClockState,
        profile_state: &mut WorldMainLoopProfileState,
        mut get_tick: GetTick,
    ) -> WorldMainLoopNetSessionStageReport
    where
        GetTick: FnMut() -> u32,
    {
        let sessions = manager.run();
        let callbacks = WorldUnionApplicationEffectCallbacks {
            random: &mut *application_callbacks.random,
            world_string: &mut *application_callbacks.world_string,
            format_world_string: &mut *application_callbacks.format_world_string,
            put_war_log: &mut *application_callbacks.put_war_log,
            refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
            faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
            write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
            faction_experience_log_enabled:
                application_callbacks.faction_experience_log_enabled,
            write_faction_experience_log:
                &mut *application_callbacks.write_faction_experience_log,
        };
        let mut effects =
            WorldUnionApplicationEffects::new(self, manager, application_runtime, callbacks);
        let union_applications = drain_union_application_runtime(
            self,
            organizing,
            organizing_parameters,
            application_runtime,
            &mut effects,
            update_player,
        );
        let finished_at_ms = get_tick();
        let elapsed_ms = finished_at_ms.wrapping_sub(clocks.stage_started_at_ms);
        profile_state.net_session_time_ms =
            profile_state.net_session_time_ms.wrapping_add(elapsed_ms);
        WorldMainLoopNetSessionStageReport {
            sessions,
            union_applications,
            finished_at_ms,
            elapsed_ms,
            accumulated_time_ms: profile_state.net_session_time_ms,
        }
    }

    /// Завершает либо продолжает один GameServer ping без нового clock-call.
    ///
    /// При публикации `m_bInPing` очищается до сборки и неприоритетной отправки
    /// `0x1FE04`; порядок ответов совпадает с порядком исходного vector.
    pub(crate) fn run_main_loop_ping_stage(
        &mut self,
        clocks: &WorldMainLoopClockState,
    ) -> Result<WorldMainLoopPingStageReport, WorldMainLoopPingError> {
        if !self.ping_in_progress {
            return Ok(WorldMainLoopPingStageReport::Idle);
        }

        let received_responses = u32::try_from(self.ping_game_servers.len()).map_err(|_| {
            WorldMainLoopPingError::ResponseCountOutsideLegacyRange {
                count: self.ping_game_servers.len(),
            }
        })?;
        let connected_game_servers = self.connected_game_server_count();
        let elapsed_ms = clocks
            .current_tick_ms
            .wrapping_sub(self.last_ping_game_server_time_ms);
        let all_connected_responded = connected_game_servers as u32 <= received_responses;
        let timed_out = 5_000 < elapsed_ms;
        if !all_connected_responded && !timed_out {
            return Ok(WorldMainLoopPingStageReport::Waiting {
                connected_game_servers,
                received_responses,
                elapsed_ms,
            });
        }

        let online_players = u32::try_from(self.online_players.len()).map_err(|_| {
            WorldMainLoopPingError::OnlinePlayerCountOutsideLegacyRange {
                count: self.online_players.len(),
            }
        })?;
        self.ping_in_progress = false;

        let declared_responses = received_responses as i32;
        let mut snapshot = CMessage::new(0x0001_FE04);
        snapshot.base_mut().add_ulong(online_players);
        snapshot.base_mut().add_long(declared_responses);
        if 0 < declared_responses {
            for response in &self.ping_game_servers {
                add_legacy_c_string(snapshot.base_mut(), &response.ip);
                snapshot.base_mut().add_long(response.map_id);
                snapshot.base_mut().add_long(response.player_count);
            }
        }
        let delivery = snapshot.send(
            self.current_login_client().map(CMyNetClient::send_queue),
            false,
        );

        Ok(WorldMainLoopPingStageReport::Published {
            connected_game_servers,
            received_responses,
            elapsed_ms,
            all_connected_responded,
            timed_out,
            online_players,
            delivery,
        })
    }

    /// Выполняет общий minute delta, `COrganizingCtrl::Run` и country `Run`.
    #[allow(
        clippy::too_many_arguments,
        reason = "два ещё отдельных downstream owner-а и clock передаются явно"
    )]
    pub(crate) fn run_main_loop_minute_stage<GetTick>(
        &mut self,
        initialization: &mut WorldMainLoopInitializationState,
        clocks: &mut WorldMainLoopTailClockState,
        organizing: &mut COrganizingCtrl,
        country_handler: &mut CCountryHandler,
        country_parameters: &CCountryParam,
        organizing_parameters: &COrganizingParam,
        attack_city: &CAttackCitySys,
        village_war: &CVillageWarSys,
        goods_war: &mut CGoodsWarMember,
        globe_setup: &GlobeSetupSnapshot,
        mut get_tick: GetTick,
        world_string: &mut dyn FnMut(&[u8]) -> Vec<u8>,
        format_world_string:
            &mut dyn FnMut(&[u8], &[UnionFormatArgument<'_>]) -> Vec<u8>,
        refresh_owned_city: &mut dyn FnMut(i32, i32, i32),
        update_player: &mut dyn FnMut(i32),
        faction_master_log_enabled: bool,
        write_faction_master_log:
            &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8]),
        faction_disband_log_enabled: bool,
        write_faction_disband_log: &mut dyn FnMut(i32, &[u8], i32, &[u8]),
    ) -> Result<WorldMainLoopMinuteStageReport, WorldMainLoopMinuteStageBlock>
    where
        GetTick: FnMut() -> u32,
    {
        let initialized = initialize_main_loop_tail_clocks(initialization, clocks, &mut get_tick);
        let current_tick_ms = get_tick();
        clocks.current_tick_ms = current_tick_ms;
        let minute_delta = current_tick_ms
            .wrapping_sub(clocks.minute_started_at_ms)
            .wrapping_div(60_000) as i32;

        let faction_disband_log_enabled =
            self.setup.use_log_system && faction_disband_log_enabled;
        let organizing_report = organizing
            .run(minute_delta, |organizing, player_id, faction_id| {
                let outcome = {
                    let mut effects = WorldOrganizingDisbandEffects {
                        game: &*self,
                        village_war,
                        attack_city,
                        country_handler: &*country_handler,
                        goods_war: &mut *goods_war,
                        world_string: &mut *world_string,
                    };
                    organizing.disband_faction(
                        &*self,
                        player_id,
                        faction_id,
                        &mut effects,
                    )?
                };
                let OrganizingDisbandOutcome::Disbanded {
                    mut progress,
                    retired_faction,
                } = outcome
                else {
                    return Ok(false);
                };
                progress.player = self.clear_disbanded_player_faction_data(player_id);
                if faction_disband_log_enabled
                    && let Some(player) = progress.player.as_ref()
                {
                    write_faction_disband_log(
                        faction_id,
                        legacy_c_string_prefix(retired_faction.name()),
                        player.player_id,
                        legacy_c_string_prefix(&player.player_name),
                    );
                    progress.log_written = true;
                }
                drop(retired_faction);
                Ok(true)
            })
            .map_err(WorldMainLoopMinuteStageBlock::Organizing)?;
        let faction_master_log_enabled = self.setup.use_log_system && faction_master_log_enabled;
        let base = WorldCountryExileResultEffects {
            game: self,
            globe_setup,
            format_world_string,
        };
        let mut effects = WorldCountryDemiseEffects {
            base,
            organizing,
            organizing_parameters,
            attack_city,
            goods_war,
            world_string,
            refresh_owned_city,
            update_player,
            faction_master_log_enabled,
            write_faction_master_log,
        };
        let country = country_handler
            .run(
                minute_delta,
                &mut get_tick,
                country_parameters,
                &mut effects,
            )
            .map_err(WorldMainLoopMinuteStageBlock::Country)?;
        clocks.minute_started_at_ms = current_tick_ms;

        Ok(WorldMainLoopMinuteStageReport {
            initialization: initialized,
            current_tick_ms,
            minute_delta,
            organizing: organizing_report,
            country,
        })
    }

    /// Завершает BaiTan batch и без промежуточного clock-call запускает JJC.
    pub(crate) fn run_main_loop_bai_tan_jjc_stage<Context: JjcRunContext>(
        &mut self,
        jjc_system: &mut CJJcSystem,
        jjc_config: JjcRunConfig,
        context: &mut Context,
    ) -> Result<WorldMainLoopBaiTanJjcStageReport, JjcRunBlock> {
        let bai_tan = self.done_bai_tan_list();
        let jjc = jjc_system.run(self, jjc_config, context)?;
        Ok(WorldMainLoopBaiTanJjcStageReport { bai_tan, jjc })
    }

    /// Возвращает mapped team-session либо исходный ноль при отсутствии key.
    pub(crate) fn get_team_session_id(&self, team_id: u32) -> i32 {
        self.team_session_ids.get(&team_id).copied().unwrap_or(0)
    }

    /// Воспроизводит `m_mTeamSessionID[teamID] = sessionID` из `CTeam::Start`.
    pub(crate) fn publish_team_session(&mut self, team_id: u32, session_id: i32) {
        self.team_session_ids.insert(team_id, session_id);
    }

    /// Удаляет найденный team key без проверки прежнего session pointer-а.
    pub(crate) fn remove_team_session(&mut self, team_id: u32) {
        self.team_session_ids.remove(&team_id);
    }

    /// Выполняет точную `CTeam -> CTeamate::Exit` цепочку timeout-login.
    pub(crate) fn exit_team_player(
        &mut self,
        factory: &mut CSessionFactory,
        session_id: i32,
        owner_type: i32,
        owner_id: i32,
    ) -> WorldLoginTimeoutTeamExit {
        let Some(plug_id) = factory
            .with_team(self, session_id, |team| {
                team.query_plug_by_owner(owner_type, owner_id)
            })
            .flatten()
        else {
            return if factory.is_team(session_id) {
                WorldLoginTimeoutTeamExit::PlugMissing
            } else {
                WorldLoginTimeoutTeamExit::SessionMissingOrNotTeam
            };
        };
        if factory
            .with_teamate(self, plug_id, |teamate| teamate.exit())
            .is_some()
        {
            WorldLoginTimeoutTeamExit::Exited
        } else {
            WorldLoginTimeoutTeamExit::PlugMissing
        }
    }

    /// Выполняет точную RTTI-цепочку смены региона участника команды.
    pub(crate) fn set_team_player_owner_region(
        &mut self,
        factory: &mut CSessionFactory,
        session_id: i32,
        owner_type: i32,
        owner_id: i32,
        region_id: i32,
    ) -> WorldRegionChangeTeamUpdate {
        let Some(plug_id) = factory
            .with_team(self, session_id, |team| {
                team.query_plug_by_owner(owner_type, owner_id)
            })
            .flatten()
        else {
            return if factory.is_team(session_id) {
                WorldRegionChangeTeamUpdate::PlugMissing
            } else {
                WorldRegionChangeTeamUpdate::SessionMissingOrNotTeam
            };
        };
        if factory
            .with_teamate(self, plug_id, |teamate| {
                teamate.set_owner_region_id(region_id)
            })
            .is_some()
        {
            WorldRegionChangeTeamUpdate::Updated
        } else {
            WorldRegionChangeTeamUpdate::PlugMissing
        }
    }

    /// Обходит весь login-list по одному общему tick snapshot и освобождает
    /// только просроченные записи, у которых ещё существует player-owner.
    pub(crate) fn process_time_out_login_player<GetTick>(
        &mut self,
        release_interval_ms: u32,
        organizing: &mut COrganizingCtrl,
        mut get_tick: GetTick,
        session_factory: &mut CSessionFactory,
    ) -> WorldLoginTimeoutReport
    where
        GetTick: FnMut() -> u32,
    {
        let snapshot_tick_ms = get_tick();
        let mut entries = Vec::with_capacity(self.login_players.len());
        let mut login_index = 0;

        while login_index < self.login_players.len() {
            let login = self.login_players[login_index];
            let elapsed_ms = snapshot_tick_ms.wrapping_sub(login.login_time_ms);
            if release_interval_ms >= elapsed_ms {
                entries.push(WorldLoginTimeoutEntryOutcome::Waiting {
                    player_id: login.player_id,
                    elapsed_ms,
                });
                login_index += 1;
                continue;
            }

            let Some(player) = self.map_player(login.player_id) else {
                // Exact EXE оставляет такой просроченный list-node на месте.
                entries.push(WorldLoginTimeoutEntryOutcome::ExpiredMissingPlayer {
                    player_id: login.player_id,
                    elapsed_ms,
                });
                login_index += 1;
                continue;
            };

            let account = player.get_account().to_vec();
            let player_name = player.get_name().to_vec();
            let player_level = player.get_level();
            let owner_type = player.get_type();
            let owner_id = player.get_id();
            let team_id = player.get_team_id();
            let friend_names = (0..player.friend_count())
                .map(|friend_index| {
                    player
                        .friend_name(friend_index)
                        .expect("friend index получен из текущего len")
                        .to_vec()
                })
                .collect::<Vec<_>>();

            let mut release = CMessage::new(0x0001_FF06);
            add_legacy_c_string(release.base_mut(), &account);
            add_legacy_c_string(release.base_mut(), &player_name);
            release.base_mut().add_byte(player_level);
            let login_delivery = release.send(
                self.current_login_client().map(CMyNetClient::send_queue),
                false,
            );

            let team_session_id = self.get_team_session_id(team_id as u32);
            let team_exit = self.exit_team_player(
                session_factory,
                team_session_id,
                owner_type,
                owner_id,
            );

            let removed = self.login_players.remove(login_index);
            debug_assert_eq!(removed.map(|entry| entry.player_id), Some(login.player_id));

            let online_removal = self.remove_online_player(organizing, login.player_id);
            let offline_inserted = self.append_offline_player_id(login.player_id);

            let mut friend_outcomes = Vec::with_capacity(friend_names.len());
            for (friend_index, friend_name) in friend_names.into_iter().enumerate() {
                let friend_player_id = self.online_player_id_by_name(&friend_name);
                if friend_player_id == 0 {
                    friend_outcomes.push(WorldLoginTimeoutFriendOutcome::Offline { friend_index });
                    continue;
                }

                let target_game_server_index = self
                    .online_player_by_id(friend_player_id)
                    .and_then(|friend| self.get_region_game_server(friend.get_region_id()))
                    .map(|game_server| game_server.index);
                let mut presence = CMessage::new(0x0007_F905);
                presence.base_mut().add_ulong(friend_player_id);
                add_legacy_c_string(presence.base_mut(), &player_name);
                let sender = self.current_game_server_sender();
                let delivery = presence.send_to_map_id(
                    sender.as_ref(),
                    target_game_server_index.unwrap_or(0) as i32,
                );
                friend_outcomes.push(WorldLoginTimeoutFriendOutcome::Notified {
                    friend_index,
                    friend_player_id,
                    target_game_server_index,
                    delivery,
                });
            }

            entries.push(WorldLoginTimeoutEntryOutcome::Released {
                player_id: login.player_id,
                elapsed_ms,
                login_delivery,
                team_id,
                team_session_id,
                team_exit,
                online_removal,
                offline_inserted,
                friend_outcomes,
            });
            // После erase сохранённый next node занимает тот же VecDeque index.
        }

        WorldLoginTimeoutReport {
            snapshot_tick_ms,
            entries,
        }
    }

    /// Выполняет точный 40-ms pacing, warning-resync и strict login-release gate.
    #[allow(
        clippy::too_many_arguments,
        reason = "clock, wait/debug adapters и session factory являются разными границами"
    )]
    pub(crate) fn run_main_loop_tail_stage<GetTick, Wait, DebugOutput>(
        &mut self,
        clocks: &mut WorldMainLoopTailClockState,
        release_state: &mut WorldMainLoopLoginReleaseState,
        organizing: &mut COrganizingCtrl,
        mut get_tick: GetTick,
        mut wait: Wait,
        mut output_debug: DebugOutput,
        session_factory: &mut CSessionFactory,
    ) -> WorldMainLoopTailStageReport
    where
        GetTick: FnMut() -> u32,
        Wait: FnMut(u32),
        DebugOutput: FnMut(&'static str),
    {
        let sampled_tick_ms = get_tick();
        clocks.current_tick_ms = sampled_tick_ms;
        let wait_duration_ms = if sampled_tick_ms.wrapping_sub(clocks.pacing_deadline_ms) < 40 {
            let duration = clocks
                .pacing_deadline_ms
                .wrapping_sub(sampled_tick_ms)
                .wrapping_add(40);
            wait(duration);
            Some(duration)
        } else {
            None
        };

        clocks.pacing_deadline_ms = clocks.pacing_deadline_ms.wrapping_add(40);
        let signed_lag_ms = sampled_tick_ms.wrapping_sub(clocks.pacing_deadline_ms) as i32;
        let warning_resync_tick_ms = if 1_000 < signed_lag_ms {
            output_debug("warning!!! 1 second not call AI()\n");
            let resync_tick_ms = get_tick();
            clocks.pacing_deadline_ms = resync_tick_ms;
            Some(resync_tick_ms)
        } else {
            None
        };

        let release_gate_tick_ms = get_tick();
        clocks.current_tick_ms = release_gate_tick_ms;
        let release_gate_elapsed_ms =
            release_gate_tick_ms.wrapping_sub(release_state.last_checked_at_ms);
        let pacing = WorldMainLoopPacingReport {
            sampled_tick_ms,
            wait_duration_ms,
            next_deadline_ms: clocks.pacing_deadline_ms,
            signed_lag_ms,
            warning_resync_tick_ms,
            release_gate_tick_ms,
            release_gate_elapsed_ms,
        };

        let Some(release_interval_ms) = self.setup.release_login_player_time_ms else {
            // BLOCKED_MISSING_FACT: constructor не задавал это поле, а safe Rust
            // не выбирает значение для исходного чтения неинициализированного DWORD.
            return WorldMainLoopTailStageReport::BlockedMissingReleaseInterval { pacing };
        };

        let login_timeout = if release_interval_ms < release_gate_elapsed_ms {
            release_state.last_checked_at_ms = release_gate_tick_ms;
            Some(self.process_time_out_login_player(
                release_interval_ms,
                organizing,
                &mut get_tick,
                session_factory,
            ))
        } else {
            None
        };

        WorldMainLoopTailStageReport::Complete {
            pacing,
            release_interval_ms,
            login_timeout,
        }
    }

    /// Выполняет весь `CGame::MainLoop` в исходном порядке прямых owner-вызовов.
    ///
    /// Возвращаемый block означает только недоказанную safe-границу, на которой
    /// старый путь не имел подтверждённого штатного продолжения. Уже выполненные
    /// мутации, sends, clock reads и callback-и не откатываются.
    #[allow(
        clippy::too_many_arguments,
        reason = "явные state/domain/platform owners сохраняют исходные границы процесса"
    )]
    pub(crate) async fn main_loop<
        TimerCallback,
        FactionContext,
        LeiTingContextOwner,
        DbMiscContextOwner,
        JjcContext,
    >(
        &mut self,
        configuration: WorldMainLoopConfiguration,
        state: &mut WorldMainLoopStateOwners<'_>,
        owners: &mut WorldMainLoopOwners<
            '_,
            TimerCallback,
            FactionContext,
            LeiTingContextOwner,
            DbMiscContextOwner,
            JjcContext,
        >,
        callbacks: &mut WorldMainLoopCallbacks<'_, TimerCallback>,
    ) -> WorldMainLoopResult<FactionContext::Block, LeiTingContextOwner::Block>
    where
        TimerCallback: Copy + PartialEq,
        FactionContext: FactionWarStopContext,
        LeiTingContextOwner: LeiTingContext,
        DbMiscContextOwner: DbMiscContext,
        JjcContext: JjcRunContext,
    {
        let profile_initialization = initialize_main_loop_profile_if_needed(
            state.initialization,
            state.profile,
            &mut *callbacks.get_tick,
        );
        let save_initialization = initialize_main_loop_save_if_needed(
            state.initialization,
            state.save_trigger,
            &mut *callbacks.get_tick,
        );
        let refresh_initialization =
            initialize_main_loop_refresh_if_needed(state.initialization, state.clocks);
        let current_tick_ms = update_main_loop_current_tick(state.clocks, &mut *callbacks.get_tick);

        let largess = self.evaluate_main_loop_largess_gate(state.clocks, state.largess);
        match largess {
            WorldMainLoopLargessGateReport::BlockedMissingFact { .. } => {
                return Err(Box::new(WorldMainLoopBlock::Largess(largess)));
            }
            WorldMainLoopLargessGateReport::StartWorkerRequested { .. } => {
                let world_number = self
                    .setup
                    .world_number
                    .expect("Largess gate проверил dwNumber до запроса worker-а");
                let _ = callbacks.largess.start_worker(world_number);
            }
            WorldMainLoopLargessGateReport::Disabled { .. }
            | WorldMainLoopLargessGateReport::Waiting { .. } => {}
        }

        let refresh_profile_started_at_ms =
            start_main_loop_profile_stage(state.clocks, &mut *callbacks.get_tick);
        let refresh = self.run_main_loop_refresh_stage(
            state.clocks,
            state.profile,
            state.process_message,
            state.refresh_high_water,
            configuration.refresh_external_counts,
            state.save_lifecycle,
            owners.log,
            &mut callbacks.get_tick,
            &mut callbacks.get_save_point_time,
            &mut callbacks.get_log_local_time,
            &mut callbacks.put_log_info,
        );
        let refresh = match refresh {
            complete @ WorldMainLoopRefreshStageReport::Complete { .. } => complete,
            blocked @ WorldMainLoopRefreshStageReport::BlockedMissingFact { .. } => {
                return Err(Box::new(WorldMainLoopBlock::Refresh(blocked)));
            }
        };

        let reload = reload_profiles(
            self,
            state.reload_flags,
            &mut *callbacks.reload_context,
            owners.jjc,
            &mut *callbacks.get_log_local_time,
            owners.country_war,
            owners.timer,
            owners.country_war_callbacks,
            &mut *callbacks.get_timer_local_time,
        );
        let reload = match reload {
            complete @ WorldReloadProfilesReport::Complete { .. } => complete,
            blocked @ (WorldReloadProfilesReport::BlockedMissingFact { .. }
            | WorldReloadProfilesReport::BlockedRegionSetup { .. }
            | WorldReloadProfilesReport::BlockedReloadOwner { .. }) => {
                return Err(Box::new(WorldMainLoopBlock::Reload(blocked)));
            }
        };

        let maintenance = self.run_main_loop_maintenance_stage(
            state.player_ranks_request,
            configuration.use_appellation_function,
            owners.player_ranks,
            owners.rs_player,
            owners.player_database.as_deref_mut(),
            owners.organizing,
            owners.honor_ranks,
            owners.auction_log,
            owners.auction_log_database.as_deref_mut(),
            owners.log,
            &mut callbacks.get_tick,
            &mut callbacks.get_log_local_time,
            &mut callbacks.get_auction_month_day,
            &mut callbacks.put_log_info,
        )
        .await;
        let maintenance = match maintenance {
            Ok(maintenance) => maintenance,
            Err(block) => return Err(Box::new(WorldMainLoopBlock::Maintenance(block))),
        };
        let collect_player_data =
            self.materialize_collect_player_data_request(state.collect_player_data);

        let try_enter_save = || (callbacks.try_enter_save)();
        let save = self.materialize_run_save_pre_gate(
            state.save_trigger,
            current_tick_ms,
            &mut callbacks.get_save_point_time,
            try_enter_save,
            owners.registry,
            owners.organizing,
            owners.coefficients,
            owners.faction_war,
            owners.country,
            configuration.country_limits,
            owners.honor_ranks,
            state.save_thread_handle,
            owners.log,
            &mut callbacks.get_tick,
            &mut callbacks.get_log_local_time,
            &mut callbacks.put_log_info,
            &mut callbacks.launch_save_thread,
        );
        let save = match save {
            WorldRunSavePreGateReport::IntervalNotElapsed {
                manual_request,
                profile_started_at_ms,
                elapsed_ms,
                save_point_time_ms,
            } => WorldMainLoopSaveStageReport {
                manual_request,
                profile_started_at_ms,
                elapsed_ms,
                save_point_time_ms,
                disposition: WorldMainLoopSaveStageDisposition::IntervalNotElapsed,
            },
            WorldRunSavePreGateReport::SaveLockBusy {
                manual_request,
                profile_started_at_ms,
                elapsed_ms,
                save_point_time_ms,
                adjusted_last_save_point_time_ms,
            } => WorldMainLoopSaveStageReport {
                manual_request,
                profile_started_at_ms,
                elapsed_ms,
                save_point_time_ms,
                disposition: WorldMainLoopSaveStageDisposition::SaveLockBusy {
                    adjusted_last_save_point_time_ms,
                },
            },
            WorldRunSavePreGateReport::AfterLock {
                manual_request,
                profile_started_at_ms,
                elapsed_ms,
                save_point_time_ms,
                trigger: WorldRunSaveTriggerReport::Complete(trigger),
            } => WorldMainLoopSaveStageReport {
                manual_request,
                profile_started_at_ms,
                elapsed_ms,
                save_point_time_ms,
                disposition: WorldMainLoopSaveStageDisposition::Triggered(trigger),
            },
            WorldRunSavePreGateReport::AfterLock {
                trigger: WorldRunSaveTriggerReport::BlockedSaveAllOrganizations { guard, block },
                ..
            } => {
                guard.stop_outer_owner();
                return Err(Box::new(WorldMainLoopBlock::SaveAllOrganizations { block }));
            }
            WorldRunSavePreGateReport::AfterLock {
                trigger: WorldRunSaveTriggerReport::BlockedImmediateSave { guard, log, block },
                ..
            } => {
                guard.stop_outer_owner();
                return Err(Box::new(WorldMainLoopBlock::ImmediateSave { log, block }));
            }
        };
        state.clocks.stage_started_at_ms = save.profile_started_at_ms;

        let ai = self.run_main_loop_ai_stage(
            state.clocks,
            state.profile,
            &mut *callbacks.get_tick,
            &mut *callbacks.random,
        );
        let mut union_application_callbacks = WorldUnionApplicationEffectCallbacks {
            random: &mut *callbacks.random,
            world_string: &mut *callbacks.world_string_by_id,
            format_world_string: &mut *callbacks.format_union_world_string,
            put_war_log: &mut *callbacks.put_union_war_log,
            refresh_owned_city: &mut *callbacks.refresh_union_owned_city,
            faction_level_log_enabled: callbacks.faction_level_log_enabled,
            write_faction_level_log: &mut *callbacks.write_faction_level_log,
            faction_experience_log_enabled: callbacks.faction_experience_log_enabled,
            write_faction_experience_log: &mut *callbacks.write_faction_experience_log,
        };
        let process_message = match self.process_message_main_loop_stage(
            owners.honor_ranks,
            owners.increment_log,
            owners.auction_log,
            owners.organizing,
            owners.organizing_parameters,
            owners.country,
            owners.country_parameters,
            owners.player_list,
            owners.country_war,
            owners.four_nation_war,
            configuration.country_limits,
            owners.faction_war,
            owners.attack_city,
            owners.attack_city_callbacks,
            owners.globe_setup,
            owners.region_router,
            owners.village_war,
            owners.goods_war,
            owners.timer,
            owners.village_war_callbacks,
            owners.registry,
            owners.original_name_index,
            owners.coefficients,
            &mut *owners.load_player_largess,
            owners.net_sessions,
            owners.jjc,
            configuration.jjc,
            owners.jjc_context,
            owners.union_application_runtime,
            &mut union_application_callbacks,
            &mut *callbacks.check_invalid_organizing_string,
            callbacks.faction_chat_log_enabled,
            callbacks.private_chat_log_enabled,
            callbacks.delete_log_enabled,
            callbacks.faction_create_log_enabled,
            &mut *callbacks.write_faction_create_log,
            callbacks.faction_title_log_enabled,
            &mut *callbacks.write_faction_title_log,
            callbacks.faction_purview_add_log_enabled,
            callbacks.faction_purview_revoke_log_enabled,
            &mut *callbacks.write_faction_purview_log,
            callbacks.faction_apply_log_enabled,
            &mut *callbacks.write_faction_apply_log,
            callbacks.faction_join_log_enabled,
            &mut *callbacks.write_faction_join_log,
            callbacks.faction_quit_log_enabled,
            &mut *callbacks.write_faction_quit_log,
            callbacks.faction_fire_out_log_enabled,
            &mut *callbacks.write_faction_fire_out_log,
            callbacks.faction_master_log_enabled,
            &mut *callbacks.write_faction_master_log,
            callbacks.faction_disband_log_enabled,
            &mut *callbacks.write_faction_disband_log,
            owners.rs_player,
            owners.player_database.as_deref_mut(),
            state.save_thread_handle,
            &mut *callbacks.launch_save_thread,
            owners.session_factory,
            owners.general_variables.as_deref_mut(),
            owners.gods_battle,
            owners.rs_gods_battle.as_deref_mut(),
            owners.gods_battle_database.as_deref_mut(),
            &mut *callbacks.reload_context,
            owners.log,
            &mut *callbacks.get_log_local_time,
            &mut *callbacks.put_log_info,
            &mut *callbacks.update_union_player,
            state.clocks,
            state.process_message,
            &mut *callbacks.get_tick,
        )
        .await
        {
            complete @ WorldProcessMessageStageReport::Complete { .. } => complete,
            blocked @ WorldProcessMessageStageReport::Blocked { .. } => {
                return Err(Box::new(WorldMainLoopBlock::ProcessMessage(blocked)));
            }
        };
        let session_factory = self.run_main_loop_session_factory_stage(
            owners.session_factory,
            state.clocks,
            state.profile,
            &mut *callbacks.get_tick,
        );
        let player_data_queue = match self.run_main_loop_player_data_queue_stage(
            owners.organizing,
            state.clocks,
            state.profile,
            &mut *callbacks.get_tick,
        ) {
            complete @ WorldMainLoopPlayerDataQueueStageReport::Complete { .. } => complete,
            blocked @ WorldMainLoopPlayerDataQueueStageReport::Blocked { .. } => {
                return Err(Box::new(WorldMainLoopBlock::PlayerDataQueue(blocked)));
            }
        };
        let timer = self
            .run_main_loop_timer_stage(
                owners.timer,
                owners.country_war,
                owners.country,
                owners.country_war_callbacks,
                owners.globe_setup,
                state.clocks,
                state.profile,
                state.copy_number_timer,
                owners.organizing_parameters,
                owners.player_ranks,
                owners.rs_player,
                owners.player_database.as_deref_mut(),
                owners.organizing,
                owners.log,
                &mut *callbacks.get_tick,
                &mut *callbacks.get_timer_local_time,
                &mut *callbacks.get_log_local_time,
                &mut *callbacks.put_log_info,
                &mut *callbacks.world_string_by_id,
                &mut *callbacks.format_union_world_string,
                &mut *callbacks.dispatch_timer,
            )
            .await
            .map_err(|block| Box::new(WorldMainLoopBlock::Timer(block)))?;
        let faction_war = self
            .run_main_loop_faction_war_stage(
                owners.faction_war,
                owners.faction_context,
                state.clocks,
                state.profile,
                &mut *callbacks.get_tick,
            )
            .map_err(|block| Box::new(WorldMainLoopBlock::FactionWar(block)))?;
        let lei_ting = self
            .run_main_loop_lei_ting_stage(
                owners.lei_ting,
                owners.globe_setup,
                owners.lei_ting_context,
                &mut *callbacks.get_lei_ting_local_time,
            )
            .map_err(|block| Box::new(WorldMainLoopBlock::LeiTing(block)))?;
        let db_misc = self
            .run_main_loop_db_misc_stage(
                owners.db_misc,
                owners.db_misc_context,
                state.clocks,
                &mut *callbacks.get_tick,
            )
            .map_err(|block| Box::new(WorldMainLoopBlock::DbMisc(block)))?;
        let mut union_application_callbacks = WorldUnionApplicationEffectCallbacks {
            random: &mut *callbacks.random,
            world_string: &mut *callbacks.world_string_by_id,
            format_world_string: &mut *callbacks.format_union_world_string,
            put_war_log: &mut *callbacks.put_union_war_log,
            refresh_owned_city: &mut *callbacks.refresh_union_owned_city,
            faction_level_log_enabled: callbacks.faction_level_log_enabled,
            write_faction_level_log: &mut *callbacks.write_faction_level_log,
            faction_experience_log_enabled: callbacks.faction_experience_log_enabled,
            write_faction_experience_log: &mut *callbacks.write_faction_experience_log,
        };
        let net_sessions = self.run_main_loop_net_session_stage(
            owners.net_sessions,
            owners.organizing,
            owners.organizing_parameters,
            owners.union_application_runtime,
            &mut union_application_callbacks,
            &mut *callbacks.update_union_player,
            state.clocks,
            state.profile,
            &mut *callbacks.get_tick,
        );
        let ping = self
            .run_main_loop_ping_stage(state.clocks)
            .map_err(|block| Box::new(WorldMainLoopBlock::Ping(block)))?;
        let minute = self
            .run_main_loop_minute_stage(
                state.initialization,
                state.tail_clocks,
                owners.organizing,
                owners.country,
                owners.country_parameters,
                owners.organizing_parameters,
                owners.attack_city,
                owners.village_war,
                owners.goods_war,
                owners.globe_setup,
                &mut *callbacks.get_tick,
                &mut *callbacks.world_string_by_id,
                &mut *callbacks.format_union_world_string,
                &mut *callbacks.refresh_union_owned_city,
                &mut *callbacks.update_union_player,
                callbacks.faction_master_log_enabled,
                &mut *callbacks.write_faction_master_log,
                callbacks.faction_disband_log_enabled,
                &mut *callbacks.write_faction_disband_log,
            )
            .map_err(|block| Box::new(WorldMainLoopBlock::Minute(block)))?;
        let bai_tan_jjc = self
            .run_main_loop_bai_tan_jjc_stage(owners.jjc, configuration.jjc, owners.jjc_context)
            .map_err(|block| Box::new(WorldMainLoopBlock::Jjc(block)))?;
        let tail = self.run_main_loop_tail_stage(
            state.tail_clocks,
            state.login_release,
            owners.organizing,
            &mut *callbacks.get_tick,
            &mut *callbacks.wait,
            &mut *callbacks.output_debug,
            owners.session_factory,
        );
        let tail = match tail {
            complete @ WorldMainLoopTailStageReport::Complete { .. } => complete,
            WorldMainLoopTailStageReport::BlockedMissingReleaseInterval { pacing } => {
                return Err(Box::new(WorldMainLoopBlock::Tail(pacing)));
            }
        };

        Ok(WorldMainLoopReport {
            profile_initialization,
            save_initialization,
            refresh_initialization,
            current_tick_ms,
            largess,
            refresh_profile_started_at_ms,
            refresh,
            reload,
            maintenance,
            collect_player_data,
            save,
            ai,
            process_message,
            session_factory,
            player_data_queue,
            timer,
            faction_war,
            lei_ting,
            db_misc,
            net_sessions,
            ping,
            minute,
            bai_tan_jjc,
            tail,
            legacy_result: 1,
        })
    }

    fn send_player_data_queue_rejection(&self, cdkey: &[u8]) -> Result<i32, SendMessageError> {
        let mut rejection = CMessage::new(0x0001_FF01);
        rejection.base_mut().add_byte(0x1C);
        add_legacy_c_string(rejection.base_mut(), cdkey);
        rejection.send(
            self.current_login_client().map(CMyNetClient::send_queue),
            false,
        )
    }

    /// Выполняет один strict Largess gate и не запускает отдельный worker.
    pub(crate) fn evaluate_main_loop_largess_gate(
        &self,
        clocks: &WorldMainLoopClockState,
        state: &mut WorldMainLoopLargessState,
    ) -> WorldMainLoopLargessGateReport {
        let Some(load_interval_ms) = self.setup.load_largess_time_ms else {
            // BLOCKED_MISSING_FACT: constructor не задавал dwLoadLargessTime;
            // неизвестное C++-чтение не позволяет назначить последующий counter.
            return WorldMainLoopLargessGateReport::BlockedMissingFact {
                field: "dwLoadLargessTime",
            };
        };

        state.pass_count = state.pass_count.wrapping_add(1);
        if load_interval_ms == 0 {
            return WorldMainLoopLargessGateReport::Disabled {
                load_interval_ms,
                pass_count: state.pass_count,
            };
        }
        if self.setup.world_number.is_none() {
            // BLOCKED_MISSING_FACT: TransferLargessThread форматировал `%d`
            // непосредственно из исходно неинициализированного dwNumber.
            return WorldMainLoopLargessGateReport::BlockedMissingFact {
                field: "dwNumber",
            };
        }

        let elapsed_ms = clocks
            .current_tick_ms
            .wrapping_sub(state.last_start_request_tick_ms);
        if load_interval_ms < elapsed_ms {
            state.last_start_request_tick_ms = clocks.current_tick_ms;
            WorldMainLoopLargessGateReport::StartWorkerRequested {
                load_interval_ms,
                pass_count: state.pass_count,
                elapsed_ms,
                requested_at_ms: clocks.current_tick_ms,
            }
        } else {
            WorldMainLoopLargessGateReport::Waiting {
                load_interval_ms,
                pass_count: state.pass_count,
                elapsed_ms,
            }
        }
    }

    /// Выполняет цельный MainLoop refresh/profile участок до `reload_profiles`.
    #[allow(
        clippy::too_many_arguments,
        reason = "caller связывает CGame, пять process-global state owners и три точных callbacks"
    )]
    pub(crate) fn run_main_loop_refresh_stage<GetTick, GetSavePointTime, GetLocalTime, PutLogInfo>(
        &mut self,
        clocks: &mut WorldMainLoopClockState,
        profile_state: &mut WorldMainLoopProfileState,
        process_message_state: &mut WorldProcessMessageStageState,
        high_water: &mut WorldRefreshInfoHighWater,
        external_counts: WorldRefreshExternalCounts,
        save_state: &SaveDataLifecycleState,
        log: &mut WorldLogTextOwner,
        get_tick: &mut GetTick,
        get_save_point_time: &mut GetSavePointTime,
        get_local_time: &mut GetLocalTime,
        put_log_info: &mut PutLogInfo,
    ) -> WorldMainLoopRefreshStageReport
    where
        GetTick: FnMut() -> u32,
        GetSavePointTime: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        let elapsed_since_refresh_ms = clocks
            .current_tick_ms
            .wrapping_sub(clocks.last_refresh_tick_ms);
        let mut refresh = WorldMainLoopRefreshDisposition::NotDue;
        if self.setup.refresh_info_time_ms < elapsed_since_refresh_ms {
            clocks.last_refresh_tick_ms = clocks.current_tick_ms;
            let current = match self.capture_refresh_info_current(external_counts) {
                Ok(current) => current,
                Err(block) => {
                    return WorldMainLoopRefreshStageReport::BlockedMissingFact {
                        elapsed_since_refresh_ms,
                        assigned_last_refresh_tick_ms: clocks.last_refresh_tick_ms,
                        block,
                    };
                }
            };
            refresh = match current {
                None => WorldMainLoopRefreshDisposition::MissingNetworkOwner,
                Some(current) => {
                    let save_point_time_ms = get_save_point_time();
                    let last_save_time = save_state.last_save_time;
                    let save = WorldRefreshSaveState {
                        last_save_time: WorldLogLocalTime {
                            year: last_save_time.year,
                            month: last_save_time.month,
                            day: last_save_time.day,
                            hour: last_save_time.hour,
                            minute: last_save_time.minute,
                            second: last_save_time.second,
                        },
                        save_point_time_ms,
                        last_save_tick_ms: save_state.last_save_tick_ms,
                        this_save_start_tick_ms: save_state.this_save_start_tick_ms,
                    };
                    WorldMainLoopRefreshDisposition::Refreshed(refresh_info_text(
                        log,
                        current,
                        high_water,
                        save,
                        &mut *get_tick,
                    ))
                }
            };
        }

        let finished_at_ms = get_tick();
        let elapsed_stage_ms = finished_at_ms.wrapping_sub(clocks.stage_started_at_ms);
        profile_state.refresh_text_time_ms = profile_state
            .refresh_text_time_ms
            .wrapping_add(elapsed_stage_ms);
        let profile = self.publish_main_loop_profile_if_due(
            clocks.current_tick_ms,
            profile_state,
            process_message_state,
            log,
            get_tick,
            get_local_time,
            put_log_info,
        );

        WorldMainLoopRefreshStageReport::Complete {
            elapsed_since_refresh_ms,
            refresh,
            finished_at_ms,
            elapsed_stage_ms,
            accumulated_refresh_time_ms: profile_state.refresh_text_time_ms,
            profile,
        }
    }

    /// Выполняет exact clear/log/tick/DB/tick/log owner `StatPlayerRanks`.
    #[allow(
        clippy::too_many_arguments,
        reason = "явные DB, organizing, clock и log owners сохраняют исходный порядок"
    )]
    async fn stat_player_ranks<PlayerDatabase, GetTick, GetLocalTime, PutLogInfo>(
        &self,
        player_ranks: &mut CPlayerRanks,
        rs_player: &mut PlayerDatabase,
        player_database: Option<&mut WorldTdsClient>,
        organizing: &COrganizingCtrl,
        log: &mut WorldLogTextOwner,
        get_tick: &mut GetTick,
        get_local_time: &mut GetLocalTime,
        put_log_info: &mut PutLogInfo,
    ) -> Result<PlayerRanksStatRunReport, PlayerRanksStatRunBlock>
    where
        PlayerDatabase: RsPlayerOwner,
        GetTick: FnMut() -> u32 + ?Sized,
        GetLocalTime: FnMut() -> WorldLogLocalTime + ?Sized,
        PutLogInfo: FnMut(&[u8]) + ?Sized,
    {
        player_ranks.clear();
        let start_log = log.add_log_text(
            b"PlayerRanks Stat. START...",
            self.setup.save_info_time_ms,
            &mut *get_tick,
            &mut *get_local_time,
            &mut *put_log_info,
        );
        let started_at_ms = get_tick();
        let outcome = rs_player
            .stat_ranks(player_ranks, organizing, player_database)
            .await;
        if let PlayerRanksStatOutcome::BlockedMissingFact(source) = &outcome {
            return Err(PlayerRanksStatRunBlock {
                started_at_ms,
                start_log,
                source: *source,
            });
        }
        let finished_at_ms = get_tick();
        let elapsed_ms = finished_at_ms.wrapping_sub(started_at_ms);
        let complete_text = format!(
            "PlayerRanks Stat. END(USED TIME:{}MS)",
            elapsed_ms as i32,
        )
        .into_bytes();
        let complete_log = log.add_log_text(
            &complete_text,
            self.setup.save_info_time_ms,
            &mut *get_tick,
            &mut *get_local_time,
            &mut *put_log_info,
        );
        Ok(PlayerRanksStatRunReport {
            started_at_ms,
            finished_at_ms,
            elapsed_ms,
            outcome,
            start_log,
            complete_log,
        })
    }

    /// Выполняет цельный PlayerRanks/HonorRanks/AuctionBang maintenance-блок.
    #[allow(
        clippy::too_many_arguments,
        reason = "caller сохраняет clock/log callbacks и явные границы доменных owners"
    )]
    pub(crate) async fn run_main_loop_maintenance_stage<
        GetTick,
        GetLocalTime,
        GetAuctionMonthDay,
        PutLogInfo,
    >(
        &self,
        player_ranks_request: &WorldPlayerRanksRequestState,
        use_appellation_function: bool,
        player_ranks: &mut CPlayerRanks,
        rs_player: &mut TiberiusRsPlayer,
        player_database: Option<&mut WorldTdsClient>,
        organizing: &COrganizingCtrl,
        honor_ranks_owner: &mut CHonorRanks,
        auction_log: &mut CAuctionLog,
        auction_log_database: Option<&mut WorldTdsClient>,
        log: &mut WorldLogTextOwner,
        get_tick: &mut GetTick,
        get_local_time: &mut GetLocalTime,
        get_auction_month_day: &mut GetAuctionMonthDay,
        put_log_info: &mut PutLogInfo,
    ) -> Result<WorldMainLoopMaintenanceReport, WorldMainLoopMaintenanceBlock>
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        GetAuctionMonthDay: FnMut() -> i32,
        PutLogInfo: FnMut(&[u8]),
    {
        let player_ranks = if player_ranks_request.take_if_requested() {
            let stat = self
                .stat_player_ranks(
                    player_ranks,
                    rs_player,
                    player_database,
                    organizing,
                    log,
                    get_tick,
                    get_local_time,
                    put_log_info,
                )
                .await
                .map_err(WorldMainLoopMaintenanceBlock::PlayerRanksStat)?;
            let sender = self.current_game_server_sender();
            let publication = player_ranks
                .update_ranks_to_game_server(sender.as_ref())
                .map_err(WorldMainLoopMaintenanceBlock::PlayerRanksSerialization)?;
            WorldPlayerRanksMaintenanceDisposition::Updated {
                stat,
                publication,
            }
        } else {
            WorldPlayerRanksMaintenanceDisposition::NotRequested
        };

        let honor_ranks = if !use_appellation_function {
            WorldHonorRanksMaintenanceDisposition::Disabled
        } else {
            let current_day = u32::from(get_local_time().day);
            match honor_ranks_owner.sort_day() {
                sort_day if sort_day == current_day => {
                    WorldHonorRanksMaintenanceDisposition::AlreadyCurrent {
                        current_day,
                        sort_day,
                    }
                }
                previous_sort_day => {
                    let started_at_ms = get_tick();
                    let start_log = log.add_log_text(
                        b"Start total HonorRankks!",
                        self.setup.save_info_time_ms,
                        &mut *get_tick,
                        &mut *get_local_time,
                        &mut *put_log_info,
                    );
                    let rollover = honor_ranks_owner.on_new_day(self, false).map_err(|source| {
                        WorldMainLoopMaintenanceBlock::HonorRanks(
                            WorldHonorRanksMaintenanceBlock {
                                current_day,
                                previous_sort_day,
                                started_at_ms,
                                start_log: start_log.clone(),
                                source,
                            },
                        )
                    })?;
                    let finished_at_ms = get_tick();
                    let elapsed_ms = finished_at_ms.wrapping_sub(started_at_ms);
                    let complete_text = format!(
                        "Total today HonorRankks complete,consume time {} millisecond!",
                        elapsed_ms as i32,
                    )
                    .into_bytes();
                    let complete_log = log.add_log_text(
                        &complete_text,
                        self.setup.save_info_time_ms,
                        &mut *get_tick,
                        &mut *get_local_time,
                        &mut *put_log_info,
                    );
                    WorldHonorRanksMaintenanceDisposition::Updated {
                        current_day,
                        previous_sort_day,
                        started_at_ms,
                        finished_at_ms,
                        elapsed_ms,
                        start_log,
                        complete_log,
                        rollover,
                    }
                }
            }
        };

        let current_month_day = get_auction_month_day();
        let Some(old_month_day) = auction_log.old_auction_day() else {
            return Err(WorldMainLoopMaintenanceBlock::AuctionOldDayUnknown {
                current_month_day,
            });
        };
        let auction_bang = if current_month_day == old_month_day {
            WorldAuctionBangMaintenanceDisposition::AlreadyCurrent {
                current_month_day,
                old_month_day,
            }
        } else {
            let start_log = log.add_log_text(
                b"Start AuctionBang Update",
                self.setup.save_info_time_ms,
                &mut *get_tick,
                &mut *get_local_time,
                &mut *put_log_info,
            );
            auction_log.set_old_auction_day(current_month_day);
            let outcome = auction_log
                .update_auction_bang_db(auction_log_database)
                .await;
            let update_succeeded = matches!(&outcome, AuctionBangUpdateOutcome::ReturnedTrue);
            let result_text = if update_succeeded {
                b"AuctionBang Update Success!".as_slice()
            } else {
                b"AuctionBang Update Fail!".as_slice()
            };
            let result_log = log.add_log_text(
                result_text,
                self.setup.save_info_time_ms,
                &mut *get_tick,
                &mut *get_local_time,
                &mut *put_log_info,
            );
            WorldAuctionBangMaintenanceDisposition::Updated {
                current_month_day,
                previous_old_month_day: old_month_day,
                update_succeeded,
                outcome,
                start_log,
                result_log,
            }
        };

        Ok(WorldMainLoopMaintenanceReport {
            player_ranks,
            honor_ranks,
            auction_bang,
        })
    }

    fn capture_refresh_info_current(
        &self,
        external: WorldRefreshExternalCounts,
    ) -> Result<Option<WorldRefreshInfoCurrent>, WorldRefreshSnapshotBlock> {
        let Some(net_server) = self.net_server.as_ref() else {
            return Ok(None);
        };
        let map_players = legacy_refresh_count("m_mPlayer", self.players.len())? as i32;
        let online_players = legacy_refresh_count("m_lOnlinePlayer", self.online_players.len())?;
        let offline_players = legacy_refresh_count("m_lOfflinePlayer", self.offline_players.len())?;
        let login_players = legacy_refresh_count("m_lLoginPlayer", self.login_players.len())?;
        let creation_players =
            legacy_refresh_count("m_lCreationPlayer", self.creation_players.len())?;
        let deletion_players =
            legacy_refresh_count("m_lDeletionPlayer", self.deletion_players.len())?;
        let restore_players = legacy_refresh_count("m_lRestorePlayer", self.restore_players.len())?;
        let saving_players = {
            let db_data = self.db_data.lock();
            legacy_refresh_count("m_stDBData.mDBPlayer", db_data.players.len())? as i32
        };
        let write_log_queue =
            legacy_refresh_count("m_qWriteLogData", self.write_log_queue.len())?;

        Ok(Some(WorldRefreshInfoCurrent {
            connections: net_server.client_count(),
            map_players,
            online_players,
            offline_players,
            login_players,
            creation_players,
            deletion_players,
            restore_players,
            saving_players,
            team_sessions: external.team_sessions,
            largess_entries: external.largess_entries,
            write_log_queue,
            player_load_queue: self.player_load_queue.get_size(),
            reback_messages: external.reback_messages,
        }))
    }

    /// Публикует и сбрасывает строго просроченное 600-секундное profiling-окно.
    #[allow(
        clippy::too_many_arguments,
        reason = "gate связывает два точных accumulator-owner-а и готовый logger"
    )]
    pub(crate) fn publish_main_loop_profile_if_due<GetTick, GetLocalTime, PutLogInfo>(
        &mut self,
        now_ms: u32,
        state: &mut WorldMainLoopProfileState,
        process_message_state: &mut WorldProcessMessageStageState,
        log: &mut WorldLogTextOwner,
        get_tick: &mut GetTick,
        get_local_time: &mut GetLocalTime,
        put_log_info: &mut PutLogInfo,
    ) -> Option<WorldMainLoopProfileReport>
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        let elapsed_since_last_publish_ms = now_ms.wrapping_sub(state.last_published_at_ms);
        if elapsed_since_last_publish_ms <= 600_000 {
            return None;
        }

        let snapshot = WorldMainLoopProfileSnapshot {
            ai_calls: state.ai_calls,
            ai_time_ms: state.ai_time_ms,
            refresh_text_time_ms: state.refresh_text_time_ms,
            process_message_time_ms: process_message_state.accumulated_time_ms,
            login_server_message_time_ms: self.login_server_message_time_ms,
            game_server_message_time_ms: self.game_server_message_time_ms,
            net_session_time_ms: state.net_session_time_ms,
            faction_war_time_ms: state.faction_war_time_ms,
            timer_time_ms: state.timer_time_ms,
            process_player_data_queue_time_ms: state.process_player_data_queue_time_ms,
            session_factory_time_ms: state.session_factory_time_ms,
            save_point_time_ms: state.save_point_time_ms,
        };
        let message = format!(
            "In 600 S Called {} AI.\r\nAI:{}\r\nRefeashText:{}\r\nMessage:{} (LS:{}  GS:{})\r\nNetSession:{}\r\nFactionWarSys:{}\r\nTimer:{}\r\nProcessPlayerDataQueue:{}\r\nSessionFactory:{}\r\nSavePoint:{}\r\n",
            snapshot.ai_calls as i32,
            snapshot.ai_time_ms as i32,
            snapshot.refresh_text_time_ms as i32,
            snapshot.process_message_time_ms as i32,
            snapshot.login_server_message_time_ms as i32,
            snapshot.game_server_message_time_ms as i32,
            snapshot.net_session_time_ms as i32,
            snapshot.faction_war_time_ms as i32,
            snapshot.timer_time_ms as i32,
            snapshot.process_player_data_queue_time_ms as i32,
            snapshot.session_factory_time_ms as i32,
            snapshot.save_point_time_ms as i32,
        );
        let log = log.add_log_text(
            message.as_bytes(),
            self.setup.save_info_time_ms,
            &mut *get_tick,
            &mut *get_local_time,
            &mut *put_log_info,
        );

        state.ai_time_ms = 0;
        state.refresh_text_time_ms = 0;
        process_message_state.accumulated_time_ms = 0;
        self.login_server_message_time_ms = 0;
        self.game_server_message_time_ms = 0;
        state.net_session_time_ms = 0;
        state.faction_war_time_ms = 0;
        state.timer_time_ms = 0;
        state.process_player_data_queue_time_ms = 0;
        state.session_factory_time_ms = 0;
        state.save_point_time_ms = 0;
        state.ai_calls = 0;
        state.last_published_at_ms = now_ms;

        Some(WorldMainLoopProfileReport {
            elapsed_since_last_publish_ms,
            snapshot,
            log,
        })
    }

    /// Закрывает прежний Login owner до присваивания нового, как `0x3FC03`.
    pub(crate) fn replace_login_client(&mut self, client: CMyNetClient) -> bool {
        let mut previous = self.net_client.take();
        if let Some(previous) = previous.as_mut() {
            let _legacy_result = previous.close();
        }
        let previous_client_closed = previous.is_some();
        drop(previous);
        self.net_client = Some(client);
        previous_client_closed
    }

    /// Возвращает world number после успешного snapshot, который уже его прочёл.
    pub(crate) fn world_number_after_cdkey_snapshot(&self) -> u32 {
        self.setup
            .world_number
            .expect("успешный CD-key snapshot проверил dwNumber")
    }

    /// Возвращает назначенный setup world number без выбора старого UB.
    pub(crate) const fn configured_world_number(&self) -> Option<u32> {
        self.setup.world_number
    }

    /// Возвращает byte-exact setup-имя для старой C-строки регистрации.
    pub(crate) fn world_name(&self) -> &[u8] {
        &self.setup.name
    }

    /// Возвращает текущий Login owner после typed replacement.
    pub(crate) fn current_login_client(&self) -> Option<&CMyNetClient> {
        self.net_client.as_ref()
    }

    /// Возвращает текущий Login owner для позднего включения control-send.
    pub(crate) fn current_login_client_mut(&mut self) -> Option<&mut CMyNetClient> {
        self.net_client.as_mut()
    }

    /// Возвращает producer handle текущего nullable GameServer owner-а.
    pub(crate) fn current_game_server_sender(&self) -> Option<ServerCommandHandle> {
        self.net_server.as_ref().map(CMyNetServer::command_handle)
    }

    /// Публикует `0x7F80D` с четырьмя последовательными signed Windows `long`.
    pub(crate) fn send_globe_variables_to_game_server(
        &self,
        socket_id: i32,
    ) -> WorldGlobeVariablesDelivery {
        let mut message = CMessage::new(0x0007_F80D);
        for value in self.globe_variables.values() {
            message.base_mut().add_long(value);
        }
        let sender = self.current_game_server_sender();
        let delivery = message.send_to_socket(sender.as_ref(), socket_id);
        WorldGlobeVariablesDelivery {
            socket_id,
            variables: self.globe_variables,
            delivery,
        }
    }

    /// Считает все подключённые GameServer в исходном ordered registry.
    pub(crate) fn connected_game_server_count(&self) -> i32 {
        self.game_servers
            .values()
            .filter(|game_server| game_server.connected)
            .fold(0_i32, |count, _| count.wrapping_add(1))
    }

    /// Возвращает `dwIndex` подключённых GameServer в исходном map-order.
    pub(crate) fn connected_game_server_indices(&self) -> impl Iterator<Item = i32> + '_ {
        self.game_servers
            .values()
            .filter(|game_server| game_server.connected)
            .map(|game_server| game_server.index as i32)
    }

    /// Считает подключённые GameServer, кроме записи с `dwIndex == 5`.
    pub(crate) fn connected_game_server_count_ex(&self) -> i32 {
        self.game_servers
            .values()
            .filter(|game_server| game_server.connected && game_server.index != 5)
            .fold(0_i32, |count, _| count.wrapping_add(1))
    }

    /// Возвращает первую запись с полным byte-exact IP и тем же port.
    ///
    /// Входной slice соответствует байтам старой C-строки до первого NUL.
    /// Неизвестный port блокирует только сравнение уже совпавшего IP.
    pub(crate) fn game_server_by_address(
        &self,
        ip: &[u8],
        port: u32,
    ) -> Result<Option<&WorldGameServerEntry>, WorldGameServerLookupError> {
        for game_server in self.game_servers.values() {
            if game_server.ip != ip {
                continue;
            }
            match game_server.port {
                Some(candidate) if candidate == port => return Ok(Some(game_server)),
                Some(_) => {}
                None => {
                    return Err(WorldGameServerLookupError::PortUnavailable {
                        index: game_server.index,
                    });
                }
            }
        }
        Ok(None)
    }

    /// Находит настроенный адрес и безусловно ставит его `bConnected`.
    pub(crate) fn connect_game_server_by_address(
        &mut self,
        ip: &[u8],
        port: u32,
    ) -> Result<Option<WorldGameServerConnectionState>, WorldGameServerLookupError> {
        let index = self
            .game_server_by_address(ip, port)?
            .map(|game_server| game_server.index);
        let Some(index) = index else {
            return Ok(None);
        };
        let game_server = self
            .game_servers
            .get_mut(&index)
            .expect("адресный поиск вернул живой ключ того же реестра");
        let previous_connected = game_server.connected;
        game_server.connected = true;
        Ok(Some(WorldGameServerConnectionState {
            index,
            previous_connected,
        }))
    }

    /// Возвращает запись по unsigned numeric ID либо старый `nullptr` как `None`.
    pub(crate) fn game_server(&self, index: u32) -> Option<&WorldGameServerEntry> {
        self.game_servers.get(&index)
    }

    /// Сбрасывает received-player counter найденного GameServer в ноль.
    pub(crate) fn reset_received_player_data(
        &mut self,
        game_server_index: i32,
    ) -> WorldReceivedPlayerDataUpdate {
        let Some(game_server) = self.game_servers.get_mut(&(game_server_index as u32)) else {
            return WorldReceivedPlayerDataUpdate::GameServerNotFound;
        };
        let previous = game_server.received_player_data.replace(0);
        WorldReceivedPlayerDataUpdate::Updated {
            previous,
            current: 0,
        }
    }

    /// Выполняет native signed increment только после доказанной инициализации.
    pub(crate) fn increment_received_player_data(
        &mut self,
        game_server_index: i32,
    ) -> WorldReceivedPlayerDataUpdate {
        let Some(game_server) = self.game_servers.get_mut(&(game_server_index as u32)) else {
            return WorldReceivedPlayerDataUpdate::GameServerNotFound;
        };
        let Some(previous) = game_server.received_player_data else {
            return WorldReceivedPlayerDataUpdate::Uninitialized;
        };
        let current = previous.wrapping_add(1);
        game_server.received_player_data = Some(current);
        WorldReceivedPlayerDataUpdate::Updated {
            previous: Some(previous),
            current,
        }
    }

    /// Возвращает exact counter; отсутствие map-entry сохраняет local-ноль EXE.
    pub(crate) fn received_player_data(
        &self,
        game_server_index: i32,
    ) -> WorldReceivedPlayerDataRead {
        let Some(game_server) = self.game_servers.get(&(game_server_index as u32)) else {
            return WorldReceivedPlayerDataRead::GameServerNotFound { legacy_value: 0 };
        };
        game_server.received_player_data.map_or(
            WorldReceivedPlayerDataRead::Uninitialized,
            WorldReceivedPlayerDataRead::Value,
        )
    }

    /// Читает `bConnected`, сохраняя bit-pattern signed Windows `long` ключа.
    pub(crate) fn is_game_server_connected(&self, server_number: i32) -> bool {
        self.game_server(server_number as u32)
            .is_some_and(|game_server| game_server.connected)
    }

    /// Применяет точную project byte-table `ToStrlwr` к живой C-строке.
    pub(crate) fn to_strlwr(value: &mut [u8]) -> &mut [u8] {
        let end = value
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(value.len());
        for byte in &mut value[..end] {
            *byte = match *byte {
                0x54 => 0xAC,
                0xC0..=0xD7 | 0xDA..=0xDF => byte.wrapping_add(0x20),
                0xD8 | 0xD9 => 0xF9,
                unchanged => unchanged,
            };
        }
        value
    }

    /// Возвращает игрока непосредственно из владеющего map либо `None`.
    pub(crate) fn map_player(&self, player_id: u32) -> Option<&CPlayer> {
        self.players.get(&player_id).map(Box::as_ref)
    }

    /// Повторяет `ValidatePlayerIDinCdkey`: lookup идёт только по live map,
    /// а account сравнивается старым `_strcmpi` до первого NUL.
    pub(crate) fn validate_player_id_in_cdkey(
        &self,
        account: &[u8],
        player_id: u32,
    ) -> bool {
        let Some(player) = self.map_player(player_id) else {
            return false;
        };
        legacy_c_string_prefix(player.get_account())
            .eq_ignore_ascii_case(legacy_c_string_prefix(account))
    }

    /// Повторяет locked `ValidateDBPlayerIDinCdkey` над frozen save-map.
    pub(crate) fn validate_db_player_id_in_cdkey(
        &self,
        account: &[u8],
        player_id: u32,
    ) -> bool {
        let db_data = self.db_data.lock();
        let Some(player) = db_data.players.get(&player_id) else {
            return false;
        };
        legacy_c_string_prefix(player.get_account())
            .eq_ignore_ascii_case(legacy_c_string_prefix(account))
    }

    pub(crate) fn set_map_player_jjc_identity(
        &mut self,
        player_id: u32,
        level: u8,
        jjc_level: u32,
    ) -> bool {
        let Some(player) = self.players.get_mut(&player_id) else {
            return false;
        };
        player.set_jjc_identity(level, jjc_level);
        true
    }

    pub(crate) fn set_map_player_jjc_snapshot(
        &mut self,
        player_id: u32,
        level: u8,
        jjc_level: u32,
        jjc_score: u32,
        counters: [u8; 0x10],
    ) -> bool {
        let Some(player) = self.players.get_mut(&player_id) else {
            return false;
        };
        player.set_jjc_snapshot(level, jjc_level, jjc_score, counters);
        true
    }

    /// Снимает unsigned map-order ключей для exact `CLeiTing` прохода.
    pub(crate) fn player_map_keys(&self) -> Vec<u32> {
        self.players.keys().copied().collect()
    }

    /// Выполняет concrete `CPlayer::UpdateLeiTing` без online-gate.
    pub(crate) fn update_map_player_lei_ting<Clock: PlayerLeiTingClock>(
        &mut self,
        map_key: u32,
        update_kind: u32,
        stamp: &mut LeiTingLocalTime,
        globe_setup: &GlobeSetupSnapshot,
        clock: &mut Clock,
    ) -> Result<
        Option<PlayerLeiTingUpdateReport>,
        PlayerLeiTingUpdateBlock<Clock::Block>,
    > {
        let Some(player) = self.players.get_mut(&map_key) else {
            return Ok(None);
        };
        player
            .update_lei_ting(
                update_kind,
                stamp,
                globe_setup.total_jing_li_dan_count(),
                &self.thing_setup,
                clock,
            )
            .map(Some)
    }

    /// Повторяет reached continuation `DisbandFaction`: snapshot имени берётся
    /// до прямой записи `m_bGetFactionData=false` тому же online map-owner-у.
    fn clear_disbanded_player_faction_data(
        &mut self,
        player_id: i32,
    ) -> Option<OrganizingDisbandPlayer> {
        let player_id = player_id as u32;
        if !self
            .online_players
            .iter()
            .any(|&online_id| online_id == player_id)
        {
            return None;
        }
        self.players.get_mut(&player_id).map(|player| {
            let player = player.as_mut();
            let snapshot = OrganizingDisbandPlayer {
                player_id: player.get_id(),
                player_name: legacy_c_string_prefix(player.get_name()).to_vec(),
            };
            player.set_faction_data_received(false);
            snapshot
        })
    }

    /// Выполняет прямую wrapping-мутацию `dwExploit` только map-owner-а.
    pub(crate) fn add_map_player_exploit_wrapping(
        &mut self,
        player_id: u32,
        increment: i32,
    ) -> Option<PlayerExploitUpdate> {
        self.players
            .get_mut(&player_id)
            .map(|player| player.add_exploit_wrapping(increment))
    }

    /// Возвращает первый map-key с `_strcmpi`-равным именем, без online-gate.
    pub(crate) fn map_player_id_by_name(&self, name: &[u8]) -> u32 {
        let name = legacy_c_string_prefix(name);
        self.players
            .iter()
            .find_map(|(&player_id, player)| {
                legacy_c_string_prefix(player.get_name())
                    .eq_ignore_ascii_case(name)
                    .then_some(player_id)
            })
            .unwrap_or(0)
    }

    /// Передаёт non-null player-owner map либо сохраняет его у caller-а.
    pub(crate) fn append_map_player(
        &mut self,
        incoming: Box<CPlayer>,
        mut add_log_text: impl FnMut(&'static str),
    ) -> WorldMapPlayerAppendOutcome {
        let player_id = incoming.get_id() as u32;
        if self.players.contains_key(&player_id) {
            add_log_text("MapPlayer Not Found or NULL.");
            return WorldMapPlayerAppendOutcome::ExistingOwnerKept {
                player_id,
                incoming,
            };
        }
        self.players.insert(player_id, incoming);
        WorldMapPlayerAppendOutcome::Inserted { player_id }
    }

    /// Возвращает игрока только после подтверждения ID в online-list.
    pub(crate) fn online_player_by_id(&self, player_id: u32) -> Option<&CPlayer> {
        let is_online = self
            .online_players
            .iter()
            .any(|&online_id| online_id == player_id);
        if !is_online {
            return None;
        }
        self.map_player(player_id)
    }

    /// Повторяет online-list scan `0x4FB07`: account-match без назначенного
    /// GameServer не завершает поиск, а переходит к следующему list-node.
    pub(crate) fn online_player_route_by_account(
        &self,
        account: &[u8],
    ) -> Option<WorldOnlineAccountPlayerRoute> {
        let account = legacy_c_string_prefix(account);
        for &online_id in &self.online_players {
            let Some(player) = self.map_player(online_id) else {
                continue;
            };
            if !legacy_c_string_prefix(player.get_account()).eq_ignore_ascii_case(account) {
                continue;
            }
            let Some(game_server) = self.get_region_game_server(player.get_region_id()) else {
                continue;
            };
            return Some(WorldOnlineAccountPlayerRoute {
                team_id: player.get_team_id(),
                owner_type: player.get_type(),
                owner_id: player.get_id(),
                game_server_index: game_server.index,
            });
        }
        None
    }

    /// Выполняет concrete `CPlayer::UpdateFactionInfo` для map-owner-а.
    ///
    /// Временное извлечение из map заменяет старый raw alias: organizing-
    /// updater и queue-send не выполняют повторный lookup этого player-а.
    /// Owner возвращается в map и при typed block, поэтому Rust lifecycle не
    /// зависит от результата сериализации.
    pub(crate) fn update_player_faction_info(
        &mut self,
        organizing: &COrganizingCtrl,
        player_id: i32,
    ) -> Result<Option<PlayerFactionInfoUpdateReport>, PlayerFactionInfoUpdateBlock> {
        let region_types = self.player_organizing_region_types();
        let player_key = player_id as u32;
        let Some(mut player) = self.players.remove(&player_key) else {
            return Ok(None);
        };
        let outcome = {
            let mut context = WorldPlayerFactionInfoContext {
                game: self,
                organizing,
                region_types: &region_types,
            };
            player.update_faction_info(&mut context)
        };
        self.players.insert(player_key, player);
        outcome.map(Some)
    }

    /// Меняет country только у owner-а, подтверждённого exact online-list.
    pub(crate) fn change_online_player_country(
        &mut self,
        player_id: u32,
        requested_country: u8,
        country_exists: impl FnOnce(u8) -> bool,
    ) -> Option<PlayerCountryChangeReport> {
        if !self
            .online_players
            .iter()
            .any(|&online_id| online_id == player_id)
        {
            return None;
        }
        self.players
            .get_mut(&player_id)
            .map(|player| player.change_country(requested_country, country_exists))
    }

    /// Мутирует silence-поле только игрока, подтверждённого online-list.
    pub(crate) fn replace_online_player_silience_time(
        &mut self,
        player_id: u32,
        silience_time: i32,
    ) -> Option<i32> {
        if !self
            .online_players
            .iter()
            .any(|&online_id| online_id == player_id)
        {
            return None;
        }
        self.players
            .get_mut(&player_id)
            .map(|player| player.replace_silience_time(silience_time))
    }

    /// Увеличивает kill/PK только у owner-а, подтверждённого online-list.
    pub(crate) fn increment_online_player_murder_counters(
        &mut self,
        player_id: u32,
    ) -> Option<PlayerMurderCounterUpdate> {
        if !self
            .online_players
            .iter()
            .any(|&online_id| online_id == player_id)
        {
            return None;
        }
        self.players
            .get_mut(&player_id)
            .map(|player| player.increment_murder_counters())
    }

    pub(crate) fn reset_online_player_murder_counters(
        &mut self,
        player_id: u32,
    ) -> Option<PlayerMurderCounterReset> {
        if !self.online_players.iter().any(|&online_id| online_id == player_id) {
            return None;
        }
        self.players
            .get_mut(&player_id)
            .map(|player| player.reset_murder_counters())
    }

    /// Декодирует player snapshot только после exact online-list lookup.
    pub(crate) fn decord_online_player_by_id(
        &mut self,
        player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<bool, PlayerCodecError> {
        if !self
            .online_players
            .iter()
            .any(|&online_id| online_id == player_id)
        {
            return Ok(false);
        }
        let Some(player) = self.players.get_mut(&player_id) else {
            return Ok(false);
        };
        let _ = player.decord_from_byte_array(source, cursor, true, registry, coefficients)?;
        Ok(true)
    }

    /// Выполняет state-часть `0x5FA02` после доказанных route/online gates.
    #[allow(
        clippy::too_many_arguments,
        reason = "positional поля wire остаются видимыми у точного call-site"
    )]
    pub(crate) fn transition_online_player_region(
        &mut self,
        organizing: &mut COrganizingCtrl,
        requested_player_id: u32,
        target_region_id: i32,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<WorldRegionChangePlayerTransition>, PlayerCodecError> {
        if !self.online_players.contains(&requested_player_id) {
            return Ok(None);
        }
        let Some(player) = self.players.get_mut(&requested_player_id) else {
            return Ok(None);
        };

        let _ = player.decord_from_byte_array(source, cursor, true, registry, coefficients)?;
        player.set_region_id(target_region_id);
        player.set_tile_xy(tile_x, tile_y);
        let direction_applied = player.set_direction(direction);
        let decoded_player_id = player.get_id() as u32;
        let team_id = player.get_team_id();
        let owner_type = player.get_type();
        let owner_id = player.get_id();

        self.remove_offline_player(decoded_player_id);
        let online_removal = self.remove_online_player(organizing, decoded_player_id);
        let login_time_ms = legacy_tick_ms();
        self.append_login_player(decoded_player_id, login_time_ms);

        Ok(Some(WorldRegionChangePlayerTransition {
            requested_player_id,
            decoded_player_id,
            target_region_id,
            tile_x,
            tile_y,
            direction,
            direction_applied,
            team_id,
            owner_type,
            owner_id,
            offline_removal_completed: true,
            online_removal,
            login_time_ms,
        }))
    }

    /// Декодирует LeiTing-хвост только у игрока из exact online-list.
    pub(crate) fn decode_online_player_lei_ting(
        &mut self,
        player_id: u32,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<bool, PlayerCodecError> {
        if !self
            .online_players
            .iter()
            .any(|&online_id| online_id == player_id)
        {
            return Ok(false);
        }
        let Some(player) = self.players.get_mut(&player_id) else {
            return Ok(false);
        };
        player.decode_byte_array_lei_ting(source, cursor)?;
        Ok(true)
    }

    /// Возвращает первый по map-порядку online-ID с ASCII-case-insensitive именем.
    pub(crate) fn online_player_id_by_name(&self, name: &[u8]) -> u32 {
        let name = legacy_c_string_prefix(name);
        for (&player_id, player) in &self.players {
            if !legacy_c_string_prefix(player.get_name()).eq_ignore_ascii_case(name) {
                continue;
            }
            if self
                .online_players
                .iter()
                .any(|&online_id| online_id == player_id)
            {
                return player_id;
            }
        }
        0
    }

    /// Возвращает первого по map-порядку online-player с `_strcmpi`-равным account.
    pub(crate) fn online_player_by_cdkey(&self, cdkey: &[u8]) -> Option<&CPlayer> {
        let cdkey = legacy_c_string_prefix(cdkey);
        for (&player_id, player) in &self.players {
            if !legacy_c_string_prefix(player.get_account()).eq_ignore_ascii_case(cdkey) {
                continue;
            }
            if self
                .online_players
                .iter()
                .any(|&online_id| online_id == player_id)
            {
                return Some(player);
            }
        }
        None
    }

    /// Возвращает `_Mysize` exact online-list для wire/count owners.
    pub(crate) fn online_player_count(&self) -> usize {
        self.online_players.len()
    }

    /// Добавляет ID в хвост только при отсутствии и всегда вызывает enter.
    pub(crate) fn append_online_player(
        &mut self,
        organizing: &mut COrganizingCtrl,
        player: &CPlayer,
    ) -> WorldOnlinePlayerAppendOutcome {
        self.append_online_player_id(organizing, player.get_id())
    }

    /// Добавляет уже декодированный ID, сохраняя map до organizing callback-а.
    pub(crate) fn append_online_player_id(
        &mut self,
        organizing: &mut COrganizingCtrl,
        player_id: i32,
    ) -> WorldOnlinePlayerAppendOutcome {
        let online_id = player_id as u32;
        let inserted = if self.online_players.contains(&online_id) {
            false
        } else {
            self.online_players.push_back(online_id);
            true
        };
        let organizing = organizing.on_player_enter_game(self, player_id);
        WorldOnlinePlayerAppendOutcome {
            inserted,
            organizing,
        }
    }

    /// Декодирует существующий либо новый player reconnect-записи.
    pub(crate) fn decord_reconnected_player(
        &mut self,
        requested_player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<WorldReconnectedPlayerDecode, PlayerCodecError> {
        if let Some(player) = self.players.get_mut(&requested_player_id) {
            let _ = player.decord_from_byte_array(source, cursor, true, registry, coefficients)?;
            return Ok(WorldReconnectedPlayerDecode {
                requested_player_id,
                decoded_player_id: player.get_id(),
                owner: WorldReconnectedPlayerOwner::Existing,
            });
        }

        let mut player = Box::new(CPlayer::with_clone_decode_constructor_state());
        let _ = player.decord_from_byte_array(source, cursor, true, registry, coefficients)?;
        let decoded_player_id = player.get_id();
        let decoded_key = decoded_player_id as u32;
        let replaced_existing_decoded_id = self.players.insert(decoded_key, player).is_some();
        let offline_inserted = self.append_offline_player_id(decoded_key);
        Ok(WorldReconnectedPlayerDecode {
            requested_player_id,
            decoded_player_id,
            owner: WorldReconnectedPlayerOwner::Created {
                replaced_existing_decoded_id,
                offline_inserted,
            },
        })
    }

    /// Декодирует обычный `0x5FA03/0x5FA09` snapshot без reconnect offline-эффекта.
    pub(crate) fn decord_server_snapshot_player(
        &mut self,
        requested_player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<WorldServerSnapshotPlayerDecode, PlayerCodecError> {
        if let Some(player) = self.players.get_mut(&requested_player_id) {
            let _ = player.decord_from_byte_array(source, cursor, true, registry, coefficients)?;
            return Ok(WorldServerSnapshotPlayerDecode {
                requested_player_id,
                decoded_player_id: player.get_id(),
                owner: WorldServerSnapshotPlayerOwner::Existing,
            });
        }

        let mut player = Box::new(CPlayer::with_clone_decode_constructor_state());
        let _ = player.decord_from_byte_array(source, cursor, true, registry, coefficients)?;
        let decoded_player_id = player.get_id();
        let decoded_key = decoded_player_id as u32;
        let replaced_existing_decoded_id = self.players.remove(&decoded_key).is_some();
        self.players.insert(decoded_key, player);
        Ok(WorldServerSnapshotPlayerDecode {
            requested_player_id,
            decoded_player_id,
            owner: WorldServerSnapshotPlayerOwner::Created {
                replaced_existing_decoded_id,
            },
        })
    }

    /// Принимает subtype `1` из `0x5FB02`, очищает transient pet vector и
    /// выполняет ранний offline-переход только для вновь созданного owner-а.
    pub(crate) fn decord_returned_player(
        &mut self,
        organizing: &mut COrganizingCtrl,
        requested_player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<WorldReturnedPlayerDecode, PlayerCodecError> {
        if let Some(player) = self.players.get_mut(&requested_player_id) {
            let _ = player.decord_from_byte_array(source, cursor, true, registry, coefficients)?;
            player.clear_uncreated_pets();
            player.set_faction_data_received(false);
            return Ok(WorldReturnedPlayerDecode {
                requested_player_id,
                decoded_player_id: player.get_id(),
                owner: WorldReturnedPlayerDecodeOwner::Existing,
            });
        }

        let mut player = Box::new(CPlayer::with_clone_decode_constructor_state());
        let _ = player.decord_from_byte_array(source, cursor, true, registry, coefficients)?;
        player.clear_uncreated_pets();
        player.set_faction_data_received(false);
        let decoded_player_id = player.get_id();
        let decoded_key = decoded_player_id as u32;
        let replaced_existing_decoded_id = self.players.remove(&decoded_key).is_some();
        self.players.insert(decoded_key, player);
        let login_removed = self.remove_login_player(decoded_key);
        let online_removal = self.remove_online_player(organizing, decoded_key);
        let offline_inserted = self.append_offline_player_id(decoded_key);
        Ok(WorldReturnedPlayerDecode {
            requested_player_id,
            decoded_player_id,
            owner: WorldReturnedPlayerDecodeOwner::Created {
                replaced_existing_decoded_id,
                login_removed,
                online_removal,
                offline_inserted,
            },
        })
    }

    pub(crate) fn returned_player_snapshot(
        &self,
        player_id: u32,
    ) -> Option<WorldReturnedPlayerSnapshot> {
        let player = self.map_player(player_id)?;
        Some(WorldReturnedPlayerSnapshot {
            account: legacy_c_string_prefix(player.get_account()).to_vec(),
            name: legacy_c_string_prefix(player.get_name()).to_vec(),
            level: player.get_level(),
            team_id: player.get_team_id(),
            owner_type: player.get_type(),
            owner_id: player.get_id(),
            friend_names: (0..player.friend_count())
                .filter_map(|index| player.friend_name(index))
                .map(legacy_c_string_prefix)
                .map(<[u8]>::to_vec)
                .collect(),
        })
    }

    /// Повторяет wrapping increment и точное equality-решение `0x5FA03`.
    ///
    /// Проверка выполняется после каждого batch, даже не terminal. При равенстве
    /// счётчик сбрасывается до `GenerateDBData`, как в EXE.
    pub(crate) fn record_player_save_response(
        &mut self,
        completion_counted: bool,
    ) -> WorldPlayerSaveResponseProgress {
        let previous_responses = self.db_responses;
        if completion_counted {
            self.db_responses = self.db_responses.wrapping_add(1);
        }
        let responses_before_reset = self.db_responses;
        let connected_game_servers = self.connected_game_server_count_ex();
        let save_triggered = responses_before_reset == connected_game_servers;
        if save_triggered {
            self.db_responses = 0;
        }
        WorldPlayerSaveResponseProgress {
            previous_responses,
            completion_counted,
            responses_before_reset,
            connected_game_servers,
            save_triggered,
        }
    }

    /// Удаляет все совпадения online-ID и затем всегда вызывает exit.
    pub(crate) fn remove_online_player(
        &mut self,
        organizing: &mut COrganizingCtrl,
        player_id: u32,
    ) -> WorldOnlinePlayerRemoveOutcome {
        let old_len = self.online_players.len();
        self.online_players
            .retain(|online_id| *online_id != player_id);
        let removed_occurrences = old_len - self.online_players.len();
        let organizing = organizing.on_player_exit_game(self, player_id as i32);
        WorldOnlinePlayerRemoveOutcome {
            removed_occurrences,
            organizing,
        }
    }

    /// Переводит игроков потерянного GameServer в offline и уведомляет Login.
    ///
    /// Сначала собираются фактические `pRegion->ID` всех assignments указанного
    /// GS в signed map-order. Затем online-list и login-list в таком порядке
    /// дают уникальные player ID. Для каждого выполняются exact side effects:
    /// удаление всех online-дубликатов, organizing exit, `AddPlayerList`, удаление
    /// первой login-записи и unique offline append. В конце Login получает
    /// `0x1FE03`, signed count и C-string имена в том же player-list order.
    pub(crate) fn on_game_server_lost<AddPlayerList>(
        &mut self,
        organizing: &mut COrganizingCtrl,
        game_server_index: u32,
        mut add_player_list: AddPlayerList,
    ) -> WorldGameServerLostReport
    where
        AddPlayerList: FnMut(&[u8]),
    {
        let mut skipped_null_region_owners = 0;
        let affected_region_ids = self
            .regions
            .values()
            .filter(|assignment| assignment.game_server_index == game_server_index)
            .filter_map(|assignment| match assignment.region.as_ref() {
                Some(region) => Some(region.base().get_id()),
                None => {
                    // Старый код разыменовывал повреждённый null pRegion. Такой
                    // внутренний UB не является compatibility-поведением.
                    skipped_null_region_owners += 1;
                    None
                }
            })
            .collect::<Vec<_>>();

        let candidate_ids = self
            .online_players
            .iter()
            .copied()
            .chain(self.login_players.iter().map(|entry| entry.player_id))
            .collect::<Vec<_>>();
        let mut affected_players = Vec::<(u32, Vec<u8>)>::new();
        for player_id in candidate_ids {
            let Some(player) = self.players.get(&player_id) else {
                continue;
            };
            if !affected_region_ids.contains(&player.get_region_id())
                || affected_players
                    .iter()
                    .any(|(affected_id, _)| *affected_id == player_id)
            {
                continue;
            }
            affected_players.push((
                player_id,
                legacy_c_string_prefix(player.get_name()).to_vec(),
            ));
        }

        let mut players = Vec::with_capacity(affected_players.len());
        for (player_id, player_name) in &affected_players {
            let online_removal = self.remove_online_player(organizing, *player_id);
            add_player_list(player_name);
            let login_removed = self.remove_login_player(*player_id);
            let offline_inserted = self.append_offline_player_id(*player_id);
            players.push(WorldLostGameServerPlayer {
                player_id: *player_id,
                player_name: player_name.clone(),
                online_removal,
                login_removed,
                offline_inserted,
            });
        }

        let mut notice = CMessage::new(0x0001_FE03);
        notice
            .base_mut()
            .add_long(affected_players.len() as i32);
        for (_, player_name) in &affected_players {
            add_legacy_c_string(notice.base_mut(), player_name);
        }
        let login_notice_delivery = notice.send(
            self.current_login_client().map(CMyNetClient::send_queue),
            false,
        );

        WorldGameServerLostReport {
            game_server_index,
            affected_region_ids,
            skipped_null_region_owners,
            players,
            login_notice_type: 0x0001_FE03,
            login_notice_delivery,
        }
    }

    /// Возвращает игрока только после подтверждения ID в login-list.
    pub(crate) fn login_player_by_id(&self, player_id: u32) -> Option<&CPlayer> {
        let is_login = self
            .login_players
            .iter()
            .any(|login_player| login_player.player_id == player_id);
        if !is_login {
            return None;
        }
        self.map_player(player_id)
    }

    /// Фиксирует route-поля до сериализации и последующих list-переходов.
    pub(crate) fn login_player_route_snapshot(
        &self,
        player_id: u32,
    ) -> Option<WorldLoginPlayerRouteSnapshot> {
        let player = self.login_player_by_id(player_id)?;
        Some(WorldLoginPlayerRouteSnapshot {
            map_key: player_id,
            owner_id: player.get_id(),
            region_id: player.get_region_id(),
        })
    }

    /// Сериализует полный mapped `CPlayer` с тем же concrete organizing
    /// adapter-ом, не меняя login/online/offline списки при safe-block-е.
    pub(crate) fn encode_map_player_full_snapshot(
        &mut self,
        organizing: &COrganizingCtrl,
        map_key: u32,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<Vec<u8>>, PlayerCodecError> {
        let Some(mut player) = self.players.remove(&map_key) else {
            return Ok(None);
        };
        let region_types = self.player_organizing_region_types();
        let mut payload = Vec::new();
        let encoded = {
            let mut updater = WorldPlayerFactionInfoContext {
                game: self,
                organizing,
                region_types: &region_types,
            };
            player.add_to_byte_array(
                &mut payload,
                true,
                registry,
                &mut updater,
                coefficients,
            )
        };
        self.players.insert(map_key, player);
        encoded.map(|_| Some(payload))
    }

    /// Сбрасывает `m_bGetFactionData` только у достигнутого map-owner-а.
    pub(crate) fn reset_map_player_faction_data(&self, map_key: u32) -> bool {
        let Some(player) = self.map_player(map_key) else {
            return false;
        };
        player.set_faction_data_received(false);
        true
    }

    /// Возвращает inherited ID первого login-игрока с byte-exact именем.
    pub(crate) fn login_player_id_by_name(&self, name: &[u8]) -> u32 {
        let name = legacy_c_string_prefix(name);
        for login_player in &self.login_players {
            let Some(player) = self.map_player(login_player.player_id) else {
                continue;
            };
            if legacy_c_string_prefix(player.get_name()) == name {
                return player.get_id() as u32;
            }
        }
        0
    }

    /// Проверяет custom-lowercase имя во всём player-map.
    pub(crate) fn is_name_exist_in_map_player(
        &self,
        name: &[u8],
    ) -> Result<bool, WorldPlayerNameLookupError> {
        for (&player_id, player) in &self.players {
            // 0x0040520D lower-case-ит player-buffer раньше requested-buffer.
            let player_name =
                copy_name_for_legacy_lowercase(player.get_name()).map_err(|length| {
                    WorldPlayerNameLookupError::PlayerNameTooLongForLegacyBuffer {
                        player_id,
                        length,
                    }
                })?;
            let requested_name = copy_name_for_legacy_lowercase(name).map_err(|length| {
                WorldPlayerNameLookupError::RequestedNameTooLongForLegacyBuffer { length }
            })?;
            if player_name == requested_name {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Возвращает первый map-owner с custom-lowercase именем и creation-ID.
    pub(crate) fn creation_player_by_name(
        &self,
        name: &[u8],
    ) -> Result<Option<&CPlayer>, WorldPlayerNameLookupError> {
        for (&player_id, player) in &self.players {
            // 0x0040540A сохраняет обратный порядок двух ToStrlwr-вызовов.
            let requested_name = copy_name_for_legacy_lowercase(name).map_err(|length| {
                WorldPlayerNameLookupError::RequestedNameTooLongForLegacyBuffer { length }
            })?;
            let player_name =
                copy_name_for_legacy_lowercase(player.get_name()).map_err(|length| {
                    WorldPlayerNameLookupError::PlayerNameTooLongForLegacyBuffer {
                        player_id,
                        length,
                    }
                })?;
            if player_name == requested_name && self.creation_players.contains(&(player_id as i32))
            {
                return Ok(Some(player.as_ref()));
            }
        }
        Ok(None)
    }

    /// Ищет имя в frozen DB-creation list под Rust mutex вместо Win32 CS.
    pub(crate) fn is_name_exist_in_db_creation(
        &self,
        name: &[u8],
    ) -> Result<bool, WorldPlayerNameLookupError> {
        let requested_name = copy_name_for_legacy_lowercase(name).map_err(|length| {
            WorldPlayerNameLookupError::RequestedNameTooLongForLegacyBuffer { length }
        })?;
        let db_data = self.db_data.lock();
        for player in &db_data.creation_players {
            let player_name = copy_name_for_legacy_lowercase(player.get_name()).map_err(
                |length| WorldPlayerNameLookupError::PlayerNameTooLongForLegacyBuffer {
                    player_id: player.get_id() as u32,
                    length,
                },
            )?;
            if player_name == requested_name {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Ищет имя в frozen DB-player map в исходном unsigned key-order.
    pub(crate) fn is_name_exist_in_db_data(
        &self,
        name: &[u8],
    ) -> Result<bool, WorldPlayerNameLookupError> {
        let requested_name = copy_name_for_legacy_lowercase(name).map_err(|length| {
            WorldPlayerNameLookupError::RequestedNameTooLongForLegacyBuffer { length }
        })?;
        let db_data = self.db_data.lock();
        for (&player_id, player) in &db_data.players {
            let player_name = copy_name_for_legacy_lowercase(player.get_name()).map_err(
                |length| WorldPlayerNameLookupError::PlayerNameTooLongForLegacyBuffer {
                    player_id,
                    length,
                },
            )?;
            if player_name == requested_name {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Повторяет exact `IsNameExitInFaction`: общий organizing lookup ищет
    /// сначала faction, затем union и сворачивает любой match в `true`.
    pub(crate) fn is_name_exit_in_faction(
        &self,
        organizing: &COrganizingCtrl,
        name: &[u8],
    ) -> Result<bool, OrganizingNameLookupBlock> {
        organizing
            .organizing_by_name(name)
            .map(|matched| matched.is_some())
    }

    /// Выполняет полный reached `CPlayer::ChangeName` без global singleton-ов.
    /// Filter получает отдельную mutable копию, а последующие проверки и
    /// финальное присваивание используют исходные bytes, как exact owner.
    pub(crate) async fn change_map_player_name<Database>(
        &mut self,
        player_id: u32,
        requested_name: Option<&[u8]>,
        globe_setup: &GlobeSetupSnapshot,
        database: &mut Database,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> Result<WorldPlayerNameChangeReport, WorldPlayerNameLookupError>
    where
        Database: RsPlayerOwner + ?Sized,
    {
        let report = |requested_name: &[u8], legacy_result, disposition| {
            WorldPlayerNameChangeReport {
                player_id,
                requested_name: requested_name.to_vec(),
                legacy_result,
                disposition,
            }
        };

        let Some(player) = self.players.get(&player_id) else {
            return Ok(report(
                requested_name.unwrap_or_default(),
                1,
                WorldPlayerNameChangeDisposition::PlayerMissing,
            ));
        };
        let Some(requested_name) = requested_name else {
            return Ok(report(
                &[],
                1,
                WorldPlayerNameChangeDisposition::NullName,
            ));
        };
        let requested_name = legacy_c_string_prefix(requested_name);
        if requested_name.len() > 0x10 {
            return Ok(report(
                requested_name,
                8,
                WorldPlayerNameChangeDisposition::NameTooLong {
                    length: requested_name.len(),
                },
            ));
        }

        let current_name = legacy_c_string_prefix(player.get_name()).to_vec();
        let special_string = globe_setup.special_string();
        let contains_special_string = special_string.is_empty()
            || current_name
                .windows(special_string.len())
                .any(|window| window == special_string);
        if !contains_special_string {
            return Ok(report(
                requested_name,
                2,
                WorldPlayerNameChangeDisposition::CurrentNameMissingSpecialString,
            ));
        }

        let mut checked_name = requested_name.to_vec();
        if !self.check_invalid_string(&mut checked_name, false) {
            return Ok(report(
                requested_name,
                3,
                WorldPlayerNameChangeDisposition::InvalidString,
            ));
        }
        if self.is_name_exist_in_map_player(requested_name)? {
            return Ok(report(
                requested_name,
                4,
                WorldPlayerNameChangeDisposition::MapPlayerNameExists,
            ));
        }
        if self.is_name_exist_in_db_data(requested_name)? {
            return Ok(report(
                requested_name,
                5,
                WorldPlayerNameChangeDisposition::DbDataNameExists,
            ));
        }
        if self.is_name_exist_in_db_creation(requested_name)? {
            return Ok(report(
                requested_name,
                6,
                WorldPlayerNameChangeDisposition::DbCreationNameExists,
            ));
        }
        if database
            .is_name_exist(requested_name, active_transaction)
            .await
        {
            return Ok(report(
                requested_name,
                7,
                WorldPlayerNameChangeDisposition::PersistentNameExists,
            ));
        }

        self.players
            .get_mut(&player_id)
            .expect("эксклюзивный CGame borrow сохраняет map-owner через DB await")
            .set_validated_name(requested_name);
        Ok(report(
            requested_name,
            0,
            WorldPlayerNameChangeDisposition::Changed {
                previous_name: current_name,
            },
        ))
    }

    /// Полностью очищает creation-list, не меняя владеющий player-map.
    pub(crate) fn clear_creation_player(&mut self) {
        self.creation_players.clear();
    }

    /// Считает все creation-list вхождения игроков с совпавшим account.
    pub(crate) fn creation_player_count_in_cdkey(&self, cdkey: &[u8]) -> u8 {
        let cdkey = legacy_c_string_prefix(cdkey);
        let mut count = 0_u8;
        for (&player_id, player) in &self.players {
            if !legacy_c_string_prefix(player.get_account()).eq_ignore_ascii_case(cdkey) {
                continue;
            }
            for &creation_id in &self.creation_players {
                if creation_id as u32 == player_id {
                    count = count.wrapping_add(1);
                }
            }
        }
        count
    }

    /// Материализует exact `GetCreationPlayerVectorByCdkey` ID-vector.
    pub(crate) fn creation_player_ids_by_cdkey(&self, cdkey: &[u8]) -> Vec<u32> {
        let cdkey = legacy_c_string_prefix(cdkey);
        let mut player_ids = Vec::new();
        for (&player_id, player) in &self.players {
            if !legacy_c_string_prefix(player.get_account()).eq_ignore_ascii_case(cdkey) {
                continue;
            }
            for &creation_id in &self.creation_players {
                if creation_id as u32 == player_id {
                    player_ids.push(player_id);
                }
            }
        }
        player_ids
    }

    /// Передаёт уникального creation-игрока владеющему map после list-вставки.
    ///
    /// На обеих collision-ветвях синхронно передаёт точный payload исходного
    /// `AddLogText`; duplicate уничтожается только после возврата callback-а.
    pub(crate) fn append_creation_player(
        &mut self,
        incoming: Box<CPlayer>,
        mut add_log_text: impl FnMut(WorldCreationPlayerAppendLog),
    ) -> WorldCreationPlayerAppendOutcome {
        let signed_player_id = incoming.get_id();
        let player_id = signed_player_id as u32;
        if self.creation_players.contains(&signed_player_id) {
            add_log_text(WorldCreationPlayerAppendLog::Duplicate { player_id });
            // `0x00410C36..0x00410C4A` удаляет incoming до исходного UAF.
            // Box::drop сохраняет destruction; typed outcome запрещает caller-у
            // продолжить с уже уничтоженным non-owning alias.
            drop(incoming);
            return WorldCreationPlayerAppendOutcome::DuplicateReleased { player_id };
        }

        self.creation_players.push_back(signed_player_id);
        if self.players.contains_key(&player_id) {
            add_log_text(WorldCreationPlayerAppendLog::ExistingMapOwner);
            // Original уже добавил list-ID, оставил старый map-owner и вернул
            // incoming pointer caller-у. Box выражает именно это непринятое
            // владение; дальнейшая судьба объекта принадлежит OnLogMessage.
            return WorldCreationPlayerAppendOutcome::ExistingMapOwnerKept {
                player_id,
                incoming,
            };
        }

        self.players.insert(player_id, incoming);
        WorldCreationPlayerAppendOutcome::Inserted { player_id }
    }

    /// Выполняет exact list-order `AddOrginGoodsToPlayer`; reject одного slot-а
    /// не останавливает дальнейший обход, как исходный debug-only failure.
    pub(crate) fn add_origin_goods_to_player<Random>(
        &self,
        player: &mut CPlayer,
        player_list: &CPlayerList,
        registry: &GoodsBasePropertiesRegistry,
        original_name_index: &GoodsOriginalNameIndex,
        random: &mut Random,
    ) -> Result<WorldOriginGoodsReport, WorldOriginGoodsBlock>
    where
        Random: FnMut(i32) -> i32 + ?Sized,
    {
        let mut entries = Vec::with_capacity(player_list.origin_equipment().len());
        for (origin_index, origin) in player_list.origin_equipment().iter().enumerate() {
            let outcome = player
                .add_origin_equipment(origin, registry, original_name_index, random)
                .map_err(|source| WorldOriginGoodsBlock {
                    origin_index,
                    source,
                })?;
            entries.push(outcome);
        }
        Ok(WorldOriginGoodsReport { entries })
    }

    /// Добавляет унаследованный ID игрока в хвост, если его ещё нет в списке.
    pub(crate) fn append_offline_player(&mut self, player: &CPlayer) {
        let _ = self.append_offline_player_id(player.get_id() as u32);
    }

    pub(crate) fn append_offline_player_id(&mut self, player_id: u32) -> bool {
        if self.offline_players.contains(&player_id) {
            return false;
        }
        self.offline_players.push_back(player_id);
        true
    }

    /// Полностью очищает список offline-ID.
    pub(crate) fn clear_offline_player(&mut self) {
        self.offline_players.clear();
    }

    /// Удаляет все совпадения ID, как исходный `std::list::remove`.
    pub(crate) fn remove_offline_player(&mut self, player_id: u32) {
        self.offline_players
            .retain(|offline_id| *offline_id != player_id);
    }

    /// Добавляет login-запись в хвост, не меняя существующий duplicate ID.
    pub(crate) fn append_login_player(&mut self, player_id: u32, login_time_ms: u32) {
        if self
            .login_players
            .iter()
            .any(|login_player| login_player.player_id == player_id)
        {
            return;
        }
        self.login_players.push_back(WorldLoginPlayerEntry {
            player_id,
            login_time_ms,
        });
    }

    /// Возвращает первый mapped player в порядке login-list с `_strcmpi`
    /// совпавшим account, не переставляя и не очищая отсутствующие map-owner-ы.
    pub(crate) fn login_player_by_account(
        &self,
        account: &[u8],
    ) -> Option<WorldLoginAccountPlayer> {
        let account = legacy_c_string_prefix(account);
        for login in &self.login_players {
            let Some(player) = self.map_player(login.player_id) else {
                continue;
            };
            if !legacy_c_string_prefix(player.get_account()).eq_ignore_ascii_case(account) {
                continue;
            }
            return Some(WorldLoginAccountPlayer {
                team_id: player.get_team_id(),
                owner_type: player.get_type(),
                owner_id: player.get_id(),
            });
        }
        None
    }

    /// Удаляет первый ожидающий DB-load record и синхронно пишет exact log.
    pub(crate) fn remove_player_load_data(&self, player_id: i32) -> bool {
        self.player_load_queue
            .remove_player_load_data(player_id)
            .is_some()
    }

    /// Копирует request в exact fixed record и передаёт его DB-load FIFO.
    pub(crate) fn push_player_load_request(
        &self,
        account: &[u8],
        player_id: u32,
        client_ip: u32,
    ) -> Result<WorldPlayerLoadRequestOutcome, WorldPlayerLoadRequestBlock> {
        let account = legacy_c_string_prefix(account);
        if account.len() >= PLAYER_LOAD_CDKEY_CAPACITY {
            return Err(WorldPlayerLoadRequestBlock {
                account_length: account.len(),
            });
        }

        let mut fixed_account = [0_u8; PLAYER_LOAD_CDKEY_CAPACITY];
        fixed_account[..account.len()].copy_from_slice(account);
        let entry = PlayerLoadQueueEntry::new(
            fixed_account,
            player_id as i32,
            client_ip,
        );
        Ok(match self.player_load_queue.push_player_load_data(entry) {
            PlayerLoadPushOutcome::Queued => WorldPlayerLoadRequestOutcome::Queued,
            PlayerLoadPushOutcome::Duplicate(_) => WorldPlayerLoadRequestOutcome::Duplicate,
        })
    }

    /// Выполняет один точный drain/process batch фонового DB-load worker-а.
    ///
    /// Несколько worker-ов конкурируют только за атомарный drain очереди; один
    /// победитель последовательно обрабатывает весь полученный FIFO-list.
    pub(crate) async fn process_player_load_batch<Loader, LoadLargess, GetTick>(
        &self,
        worker_index: u32,
        loader: &mut Loader,
        load_largess: &mut LoadLargess,
        get_tick: GetTick,
    ) -> Result<WorldPlayerLoadBatchReport, WorldPlayerLoadBatchBlock>
    where
        Loader: WorldPlayerDataLoadOwner + ?Sized,
        LoadLargess: FnMut(&mut CPlayer) + ?Sized,
        GetTick: FnMut() -> u32,
    {
        self.player_load_worker_spec()
            .process_batch(worker_index, loader, load_largess, get_tick)
            .await
    }

    /// Повторяет polling-loop `LoadPlayerDataFromDB` до одного из двух flags.
    pub(crate) async fn run_player_load_worker<Loader, LoadLargess, GetTick>(
        &self,
        worker_index: u32,
        game_thread_exit: &AtomicBool,
        player_load_threads_exit: &AtomicBool,
        loader: &mut Loader,
        load_largess: &mut LoadLargess,
        get_tick: GetTick,
    ) -> Result<WorldPlayerLoadWorkerReport, WorldPlayerLoadWorkerBlock>
    where
        Loader: WorldPlayerDataLoadOwner + ?Sized,
        LoadLargess: FnMut(&mut CPlayer) + ?Sized,
        GetTick: FnMut() -> u32,
    {
        self.player_load_worker_spec()
            .run(
                worker_index,
                game_thread_exit,
                player_load_threads_exit,
                loader,
                load_largess,
                get_tick,
            )
            .await
    }

    /// Удаляет первую login-запись с указанным ID либо сохраняет список.
    pub(crate) fn remove_login_player(&mut self, player_id: u32) -> bool {
        let Some(index) = self
            .login_players
            .iter()
            .position(|login_player| login_player.player_id == player_id)
        else {
            return false;
        };
        let _ = self.login_players.remove(index);
        true
    }

    /// Возвращает регион по signed numeric ID либо старый `nullptr` как `None`.
    pub(crate) fn region(&self, region_id: i32) -> Option<&WorldRegionAssignment> {
        self.regions.get(&region_id)
    }

    /// Обновляет владельца города и рассылает exact `0x7FE27` всем GameServer.
    pub(crate) fn refresh_owned_city_org(
        &mut self,
        organizing: &COrganizingCtrl,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
    ) -> Result<WorldOwnedCityRefreshOutcome, FactionInitialPropertyBlock> {
        let Some(assignment) = self.regions.get(&region_id) else {
            return Ok(WorldOwnedCityRefreshOutcome::RegionNotFound);
        };
        if assignment.region.is_none() {
            return Ok(WorldOwnedCityRefreshOutcome::NullRegionPointer);
        }

        let country_id = organizing.country_by_faction(faction_id)?.unwrap_or(0);
        let region = self
            .regions
            .get_mut(&region_id)
            .and_then(|assignment| assignment.region.as_mut())
            .expect("materialized region owner проверен до country lookup");
        region
            .base_mut()
            .set_owned_city_org(faction_id, union_id);
        region
            .base_mut()
            .region_base_mut()
            .set_country(country_id);

        let mut message = CMessage::new(0x0007_FE27);
        message.base_mut().add_long(region_id);
        message.base_mut().add_long(faction_id);
        message.base_mut().add_long(union_id);
        message.base_mut().add_byte(country_id);
        let delivery = message.send_all(self.current_game_server_sender().as_ref());
        Ok(WorldOwnedCityRefreshOutcome::Refreshed(
            WorldOwnedCityRefreshReport {
                region_id,
                faction_id,
                union_id,
                country_id,
                delivery,
            },
        ))
    }

    /// Возвращает только живой concrete `CRegion` create-role ветки.
    pub(crate) fn creation_region_base(&self, region_id: i32) -> Option<&CRegion> {
        self.regions
            .get(&region_id)?
            .region
            .as_ref()
            .map(WorldRegionOwner::base)
            .map(CWorldRegion::creation_region_base)
    }

    /// Применяет три tax-поля к достигнутому `tagRegion::pRegion`.
    pub(crate) fn set_region_param_from_game_server(
        &mut self,
        region_id: i32,
        current_tax_rate: i32,
        today_total_tax: u32,
        total_tax: u32,
    ) -> WorldRegionParamUpdateOutcome {
        let Some(assignment) = self.regions.get_mut(&region_id) else {
            return WorldRegionParamUpdateOutcome::RegionNotFound;
        };
        let Some(region) = assignment.region.as_mut() else {
            return WorldRegionParamUpdateOutcome::NullRegionPointer;
        };
        region.base_mut().set_param_from_gs(
            current_tax_rate,
            today_total_tax,
            total_tax,
        );
        WorldRegionParamUpdateOutcome::Applied
    }

    /// Передаёт remaining wire и общий cursor точному region-param owner-у.
    pub(crate) fn decode_region_param_from_game_server(
        &mut self,
        region_id: i32,
        source: &[u8],
        cursor: &mut usize,
    ) -> WorldRegionParamDecodeOutcome {
        let Some(assignment) = self.regions.get_mut(&region_id) else {
            return WorldRegionParamDecodeOutcome::RegionNotFound;
        };
        let Some(region) = assignment.region.as_mut() else {
            return WorldRegionParamDecodeOutcome::NullRegionPointer;
        };
        WorldRegionParamDecodeOutcome::Decoded(
            region
                .base_mut()
                .decord_region_param_from_byte_array(source, cursor, true),
        )
    }

    /// Сериализует initial-config регионы в signed map-order и передаёт каждый
    /// элемент visitor-у до перехода к следующему узлу.
    pub(crate) fn visit_initial_region_snapshots<Visit>(
        &self,
        target_game_server_index: u32,
        mut visit: Visit,
    ) -> Result<(), WorldInitialRegionSnapshotBlock>
    where
        Visit: FnMut(WorldInitialRegionSnapshot),
    {
        for (&map_key, assignment) in &self.regions {
            let region = assignment.region.as_ref().ok_or(
                WorldInitialRegionSnapshotBlock {
                    map_key,
                    source: WorldInitialRegionSnapshotSource::MissingRegionOwner,
                },
            )?;
            let region_id = region.base().get_id();
            let mut payload = Vec::new();
            let kind = if assignment.game_server_index == target_game_server_index {
                let region_type = assignment.region_type.ok_or(
                    WorldInitialRegionSnapshotBlock {
                        map_key,
                        source: WorldInitialRegionSnapshotSource::UninitializedRegionType,
                    },
                )?;
                region
                    .add_full_initial_snapshot(&mut payload)
                    .map_err(|source| WorldInitialRegionSnapshotBlock {
                        map_key,
                        source: WorldInitialRegionSnapshotSource::Full(source),
                    })?;
                WorldInitialRegionSnapshotKind::Assigned { region_type }
            } else {
                let _ = region
                    .base()
                    .add_to_byte_array_for_proxy(&mut payload, true)
                    .map_err(|source| WorldInitialRegionSnapshotBlock {
                        map_key,
                        source: WorldInitialRegionSnapshotSource::Proxy(source),
                    })?;
                WorldInitialRegionSnapshotKind::Proxy
            };
            visit(WorldInitialRegionSnapshot {
                map_key,
                region_id,
                kind,
                payload,
            });
        }
        Ok(())
    }

    /// Проходит `tagRegion::pRegion`, не смешивая отсутствующий key и null.
    pub(crate) fn region_name(&self, region_id: i32) -> WorldRegionNameLookup<'_> {
        let Some(assignment) = self.region(region_id) else {
            return WorldRegionNameLookup::RegionNotFound;
        };
        let Some(region) = assignment.region.as_ref().map(WorldRegionOwner::base) else {
            return WorldRegionNameLookup::NullRegionPointer;
        };
        WorldRegionNameLookup::Name(region.get_name())
    }

    /// Повторяет ordered `GetRegion(char const*)` с case-sensitive `strcmp`.
    ///
    /// Route-поля являются typed snapshot найденного `tagRegion`, а не новой
    /// ступенью поиска; null owner безопасно учитывается вместо старого UB.
    pub(crate) fn named_region_lookup(&self, name: &[u8]) -> WorldNamedRegionLookup {
        let name = legacy_c_string_prefix(name);
        let mut skipped_null_owners = 0;
        for assignment in self.regions.values() {
            let Some(region) = assignment.region.as_ref().map(WorldRegionOwner::base) else {
                // В EXE `GetRegion(name)` разыменовывал null `pRegion`. Это
                // внутренний UB повреждённого состояния, а не wire-контракт.
                skipped_null_owners += 1;
                continue;
            };
            if legacy_c_string_prefix(region.get_name()) != name {
                continue;
            }
            let game_server = self.game_server(assignment.game_server_index);
            return WorldNamedRegionLookup {
                skipped_null_owners,
                matched: Some(WorldNamedRegionMatch {
                    region_id: region.get_id(),
                    game_server_index: assignment.game_server_index,
                    game_server_entry_found: game_server.is_some(),
                    game_server_connected: game_server.is_some_and(|server| server.connected),
                }),
            };
        }
        WorldNamedRegionLookup {
            skipped_null_owners,
            matched: None,
        }
    }

    /// Собирает exact ordered fan-out по фактическому `pRegion->ID`.
    pub(crate) fn region_routes_by_owner_id(&self, region_id: i32) -> WorldRegionIdRouteScan {
        let mut skipped_null_owners = 0;
        let mut matching_region_keys = 0;
        let mut routes = Vec::new();
        for (&map_key, assignment) in &self.regions {
            let Some(region) = assignment.region.as_ref().map(WorldRegionOwner::base) else {
                skipped_null_owners += 1;
                continue;
            };
            if region.get_id() != region_id {
                continue;
            }
            matching_region_keys += 1;
            let game_server_id = self.game_server_number_by_region_id(map_key);
            if game_server_id != 0 {
                routes.push(WorldRegionIdRoute {
                    map_key,
                    game_server_id,
                });
            }
        }
        WorldRegionIdRouteScan {
            skipped_null_owners,
            matching_region_keys,
            routes,
        }
    }

    /// Проверяет обе ступени исходного `GetRegion -> tagRegion::pRegion`.
    pub(crate) fn has_materialized_region(&self, region_id: i32) -> bool {
        self.region(region_id)
            .and_then(|assignment| assignment.region.as_ref())
            .is_some()
    }

    /// Возвращает владельца только для materialized `tagRegion::pRegion`.
    pub(crate) fn region_owned_faction_id(&self, region_id: i32) -> Option<i32> {
        self.region(region_id)
            .and_then(|assignment| assignment.region.as_ref())
            .map(|region| region.base().get_owned_city_faction())
    }

    /// Возвращает reached country byte только живого region-owner-а.
    pub(crate) fn region_country_id(&self, region_id: i32) -> Option<u8> {
        self.region(region_id)
            .and_then(|assignment| assignment.region.as_ref())
            .and_then(|region| region.base().region_base().country())
    }

    /// Находит назначенный региону GameServer через две исходные map-ступени.
    pub(crate) fn get_region_game_server(&self, region_id: i32) -> Option<&WorldGameServerEntry> {
        let region = self.region(region_id)?;
        self.game_server(region.game_server_index)
    }

    /// Возвращает `dwIndex` назначенного GameServer либо исходный ноль.
    pub(crate) fn game_server_number_by_region_id(&self, region_id: i32) -> i32 {
        self.get_region_game_server(region_id)
            .map_or(0, |game_server| game_server.index as i32)
    }

    /// Находит GameServer через online-игрока и его унаследованный region ID.
    pub(crate) fn player_game_server(&self, player_id: i32) -> Option<&WorldGameServerEntry> {
        let player = self.online_player_by_id(player_id as u32)?;
        self.get_region_game_server(player.get_region_id())
    }

    /// Возвращает `dwIndex` GameServer online-игрока либо исходный ноль.
    pub(crate) fn game_server_number_by_player_id(&self, player_id: i32) -> i32 {
        let Some(player) = self.online_player_by_id(player_id as u32) else {
            return 0;
        };
        let Some(game_server) = self.get_region_game_server(player.get_region_id()) else {
            return 0;
        };
        game_server.index as i32
    }

    /// Делегирует живое сообщение готовому `CMessage::SendToMapID`.
    pub(crate) fn send_msg_to_game_server(
        &self,
        map_id: i32,
        message: &CMessage,
    ) -> Result<i32, SendMessageError> {
        let sender = self.current_game_server_sender();
        message.send_to_map_id(sender.as_ref(), map_id)
    }

    /// Начинает точный цикл опроса GameServer из ветки `0x4FC01`.
    pub(crate) fn begin_game_server_ping(&mut self) -> (usize, u32) {
        self.ping_in_progress = true;
        let cleared_responses = self.ping_game_servers.len();
        self.ping_game_servers.clear();
        let started_at_ms = legacy_tick_ms();
        self.last_ping_game_server_time_ms = started_at_ms;
        (cleared_responses, started_at_ms)
    }

    /// Добавляет один ответ GameServer без проверки текущего ping-флага.
    pub(crate) fn record_game_server_ping(&mut self, response: WorldPingGameServerInfo) -> usize {
        self.ping_game_servers.push(response);
        self.ping_game_servers.len()
    }

    /// Добавляет request `(IP, player_id)` без замены прежнего duplicate IP.
    pub(crate) fn add_item_to_bai_tan_request_list(&mut self, ip: u32, player_id: i32) -> bool {
        if let std::collections::btree_map::Entry::Vacant(entry) = self.bai_tan_requests.entry(ip) {
            entry.insert(player_id);
            true
        } else {
            false
        }
    }

    /// Выполняет три независимых insert-а active BaiTan registries.
    pub(crate) fn add_item_to_bai_tan_list(
        &mut self,
        player_id: i32,
        ip: u32,
    ) -> WorldBaiTanRegistration {
        let player_ip_inserted = if let std::collections::btree_map::Entry::Vacant(entry) =
            self.bai_tan_player_ips.entry(player_id)
        {
            entry.insert(ip);
            true
        } else {
            false
        };

        let ip_refcount = match self.bai_tan_ip_refcounts.get_mut(&ip) {
            Some(refcount) => {
                *refcount = refcount.wrapping_add(1);
                *refcount
            }
            None => {
                self.bai_tan_ip_refcounts.insert(ip, 1);
                1
            }
        };

        let game_server_index = self.game_server_number_by_player_id(player_id);
        let player_route_inserted = if let std::collections::btree_map::Entry::Vacant(entry) =
            self.bai_tan_routes.entry(player_id)
        {
            entry.insert(game_server_index);
            true
        } else {
            false
        };

        WorldBaiTanRegistration {
            player_id,
            ip,
            player_ip_inserted,
            ip_refcount,
            game_server_index,
            player_route_inserted,
        }
    }

    /// Удаляет player->IP и player->route, уменьшая найденный IP refcount.
    pub(crate) fn del_item_from_bai_tan_list(&mut self, player_id: i32) -> WorldBaiTanRemoval {
        let mapped_ip = self.bai_tan_player_ips.get(&player_id).copied();
        let remaining_ip_refcount = mapped_ip.and_then(|ip| self.del_item_to_bai_tan_ip_list(ip));
        let player_ip_removed = self.bai_tan_player_ips.remove(&player_id).is_some();
        let player_route_removed = self.bai_tan_routes.remove(&player_id).is_some();
        WorldBaiTanRemoval {
            player_id,
            mapped_ip,
            remaining_ip_refcount,
            player_ip_removed,
            player_route_removed,
        }
    }

    /// Обрабатывает весь request map в unsigned IP-order и очищает его в конце.
    pub(crate) fn done_bai_tan_list(&mut self) -> WorldDoneBaiTanListReport {
        let requests: Vec<(u32, i32)> = self
            .bai_tan_requests
            .iter()
            .map(|(&ip, &player_id)| (ip, player_id))
            .collect();
        let mut completions = Vec::with_capacity(requests.len());
        for (requested_ip, player_id) in requests {
            let registration = self.add_item_to_bai_tan_list(player_id, requested_ip);

            let mut response = CMessage::new(0x0008_040D);
            response.base_mut().add_long(player_id);
            response.base_mut().add_long(1);
            let route_game_server_index = self.game_server_number_by_player_id(player_id);
            let delivery = self.send_msg_to_game_server(route_game_server_index, &response);
            completions.push(WorldBaiTanCompletion {
                requested_ip,
                player_id,
                registration,
                route_game_server_index,
                delivery,
            });
        }
        let cleared_requests = self.bai_tan_requests.len();
        self.bai_tan_requests.clear();
        WorldDoneBaiTanListReport {
            completions,
            cleared_requests,
        }
    }

    /// Возвращает текущий active route, сохранённый при первом player insert.
    pub(crate) fn bai_tan_game_server_index(&self, player_id: i32) -> Option<i32> {
        self.bai_tan_routes.get(&player_id).copied()
    }

    /// Возвращает текущий refcount одного BaiTan IP.
    pub(crate) fn bai_tan_ip_refcount(&self, ip: u32) -> Option<i32> {
        self.bai_tan_ip_refcounts.get(&ip).copied()
    }

    /// Безусловно присваивает signed LoginServer ID из ветки `0x4FC03`.
    pub(crate) fn assign_login_server_id(&mut self, login_server_id: i32) -> i32 {
        std::mem::replace(&mut self.login_server_id, login_server_id)
    }

    /// Возвращает signed LoginServer ID для финального initial packet `0x3B`.
    pub(crate) const fn login_server_id(&self) -> i32 {
        self.login_server_id
    }

    /// Сбрасывает live/queued honor counters и общий elimination-ledger.
    pub(crate) fn reset_honor_eliminate_info(&mut self, rank_mask: u32) -> bool {
        for player in self.players.values_mut() {
            player.reset_honor_eliminate_info(rank_mask);
        }
        self.honor_eliminate_list.clear();
        self.player_data_queue
            .reset_honor_eliminate_info(rank_mask);
        true
    }

    /// Повторяет online-check и per-player duplicate-ledger ветки `0x5FD0D`.
    pub(crate) fn register_honor_eliminator(
        &mut self,
        player_id: u32,
        eliminator_id: u32,
    ) -> WorldHonorEliminatorRegistration {
        if self.online_player_by_id(player_id).is_none() {
            return WorldHonorEliminatorRegistration::MissingOnlinePlayer;
        }

        let eliminators = self.honor_eliminate_list.entry(player_id).or_default();
        if eliminators
            .iter()
            .any(|tracked_id| *tracked_id == eliminator_id)
        {
            return WorldHonorEliminatorRegistration::Duplicate;
        }
        eliminators.push_back(eliminator_id);
        WorldHonorEliminatorRegistration::Accepted
    }

    /// Публикует owned сообщение в исходную receive FIFO `s_pNetServer`.
    pub(crate) fn queue_local_world_message(
        &self,
        message: CMessage,
    ) -> Result<(), WorldLocalMessageQueueBlock> {
        let message_type = message.message_type();
        let Some(net_server) = self.net_server.as_ref() else {
            // BLOCKED_MISSING_FACT: exact owner безусловно разыменовывал
            // обязательный s_pNetServer. Safe Rust не подменяет этот путь
            // прямым вызовом handler-а и сохраняет границу FIFO.
            return Err(WorldLocalMessageQueueBlock { message_type });
        };
        net_server.publish_local_message(message);
        Ok(())
    }

    fn close_and_remove_net_client(&mut self) {
        if let Some(client) = self.net_client.as_mut() {
            let _legacy_result = client.close();
        }
        self.net_client = None;
    }

    fn del_item_to_bai_tan_ip_list(&mut self, ip: u32) -> Option<i32> {
        let refcount = self.bai_tan_ip_refcounts.get_mut(&ip)?;
        if 1 < *refcount {
            *refcount -= 1;
            return Some(*refcount);
        }
        self.bai_tan_ip_refcounts.remove(&ip);
        None
    }
}

/// Создаёт и публикует единственный World game-owner, возвращая старый `1`.
pub(crate) fn create_game(
    game: &mut Option<Box<CGame>>,
) -> Result<WorldCreateGameReport, WorldCreateGameBlock> {
    if game.is_some() {
        // Повторный CreateGame перезаписывал бы global pointer и терял прежний
        // owner. Такой caller не доказан, поэтому safe API не создаёт утечку.
        return Err(WorldCreateGameBlock);
    }
    *game = Some(Box::new(CGame::new()));
    Ok(WorldCreateGameReport { legacy_result: 1 })
}

/// Возвращает текущий `g_pGame`, сохраняя исходный nullable результат.
pub(crate) fn get_game(game: &mut Option<Box<CGame>>) -> Option<&mut CGame> {
    game.as_deref_mut()
}

/// Вызывает Rust Drop для опубликованного owner-а, обнуляет slot и возвращает `1`.
pub(crate) fn delete_game(game: &mut Option<Box<CGame>>) -> WorldDeleteGameReport {
    let owner_was_present = game.is_some();
    drop(game.take());
    WorldDeleteGameReport {
        owner_was_present,
        legacy_result: 1,
    }
}

#[derive(Clone, Copy)]
enum WorldGameInitCallerDisposition {
    ReturnedFalse,
    FatalNonreturn,
    Blocked,
}

fn classify_game_init_for_caller<ContextBlock>(
    block: &WorldGameInitBlock<ContextBlock>,
) -> WorldGameInitCallerDisposition {
    match &block.reason {
        WorldGameInitBlockReason::InvalidPlayerLoadThreadCount { .. } => {
            WorldGameInitCallerDisposition::FatalNonreturn
        }
        WorldGameInitBlockReason::MissingPlayerLoadThreadCount
        | WorldGameInitBlockReason::Context(_)
        | WorldGameInitBlockReason::Reload(_)
        | WorldGameInitBlockReason::OrganizingParameters(_)
        | WorldGameInitBlockReason::PlayerRanksSchedule(_)
        | WorldGameInitBlockReason::PlayerRanksStat(_)
        | WorldGameInitBlockReason::NetworkClient(
            WorldClientInitializationError::MissingSetupField(_)
            | WorldClientInitializationError::LoginAddressTooLongReactionUnknown { .. }
            | WorldClientInitializationError::LoginAddressEncodingUnsupported,
        )
        | WorldGameInitBlockReason::NetworkServer(
            WorldNetworkInitializationError::MissingSetupField(_),
        ) => WorldGameInitCallerDisposition::Blocked,
        _ => WorldGameInitCallerDisposition::ReturnedFalse,
    }
}

/// Выполняет `CreateGame -> Init -> MainLoop -> Release -> DeleteGame`.
///
/// Fatal `_exit(1)` и typed safe-blocks возвращают live `Box<CGame>`, поэтому
/// Rust не приписывает им исходно отсутствовавший Release/DeleteGame. Только
/// штатный конец публикует exit-event, затем window-close request и код `0`.
pub(crate) async fn game_thread_func<Runtime: WorldGameThreadRuntime>(
    runtime: &mut Runtime,
) -> WorldGameThreadReport<Runtime::InitBlock, Runtime::MainLoopBlock> {
    let mut game_slot = None;
    let creation = create_game(&mut game_slot).expect("GameThreadFunc начинает с пустого g_pGame");

    let initialization_result = runtime
        .initialize_game(
            game_slot
                .as_deref_mut()
                .expect("CreateGame только что опубликовал owner"),
        )
        .await;

    let mut main_loop_calls = 0_u64;
    let (initialization, stop) = match initialization_result {
        Ok(initialization) => {
            let stop = loop {
                if runtime.game_thread_exit_requested() {
                    break WorldGameThreadStop::ExitRequested;
                }
                let result = runtime.run_main_loop(
                    game_slot
                        .as_deref_mut()
                        .expect("game owner жив до Release/DeleteGame"),
                );
                main_loop_calls = main_loop_calls.wrapping_add(1);
                match result {
                    Ok(legacy_result) if legacy_result != 0 => {}
                    Ok(legacy_result) => {
                        break WorldGameThreadStop::MainLoopReturned { legacy_result };
                    }
                    Err(block) => {
                        return WorldGameThreadReport::BlockedMainLoop {
                            game: game_slot
                                .take()
                                .expect("blocked MainLoop сохраняет live game-owner"),
                            creation,
                            initialization,
                            main_loop_calls,
                            block,
                        };
                    }
                }
            };
            runtime.wait_for_save_barrier();
            (
                WorldGameThreadInitialization::Complete(initialization),
                stop,
            )
        }
        Err(block) => match classify_game_init_for_caller(&block) {
            WorldGameInitCallerDisposition::FatalNonreturn => {
                return WorldGameThreadReport::FatalInitialization {
                    game: game_slot
                        .take()
                        .expect("fatal Init сохраняет опубликованный owner"),
                    creation,
                    block,
                };
            }
            WorldGameInitCallerDisposition::Blocked => {
                return WorldGameThreadReport::BlockedInitialization {
                    game: game_slot
                        .take()
                        .expect("blocked Init сохраняет опубликованный owner"),
                    creation,
                    block,
                };
            }
            WorldGameInitCallerDisposition::ReturnedFalse => (
                WorldGameThreadInitialization::Failed(block),
                WorldGameThreadStop::InitializationFailed,
            ),
        },
    };

    let mut goods_war = runtime.take_goods_war_member();
    let mut increment_log = runtime.take_increment_log();
    let release = match game_slot
        .as_deref_mut()
        .expect("Release вызывается до DeleteGame")
        .release(runtime, &mut goods_war, &mut increment_log)
    {
        Ok(release) => release,
        Err(block) => {
            runtime.restore_goods_war_member(goods_war);
            runtime.restore_increment_log(increment_log);
            return WorldGameThreadReport::BlockedRelease {
                game: game_slot
                    .take()
                    .expect("blocked Release сохраняет live game-owner"),
                creation,
                initialization,
                main_loop_calls,
                stop,
                block,
            };
        }
    };
    runtime.restore_goods_war_member(goods_war);
    runtime.restore_increment_log(increment_log);
    let deletion = delete_game(&mut game_slot);
    runtime.signal_game_thread_exit();
    runtime.request_window_close();
    WorldGameThreadReport::Complete {
        creation,
        initialization,
        main_loop_calls,
        stop,
        release,
        deletion,
        legacy_exit_code: 0,
    }
}

struct WorldCountryInfoDelivery<'a> {
    game: &'a CGame,
}

impl CountryInfoDeliveryContext for WorldCountryInfoDelivery<'_> {
    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }
}

struct WorldCountryWarEffects<'a> {
    game: &'a CGame,
    country_handler: &'a mut CCountryHandler,
    globe_setup: &'a GlobeSetupSnapshot,
    world_string: &'a mut dyn FnMut(&[u8]) -> Vec<u8>,
    format_world_string:
        &'a mut dyn FnMut(&[u8], &[UnionFormatArgument<'_>]) -> Vec<u8>,
}

struct WorldCountryExileResultEffects<'a> {
    game: &'a mut CGame,
    globe_setup: &'a GlobeSetupSnapshot,
    format_world_string:
        &'a mut dyn FnMut(&[u8], &[UnionFormatArgument<'_>]) -> Vec<u8>,
}

struct WorldCountryPlayersListEffects<'a> {
    game: &'a CGame,
    organizing: &'a COrganizingCtrl,
    globe_setup: &'a GlobeSetupSnapshot,
    format_world_string:
        &'a mut dyn FnMut(&[u8], &[UnionFormatArgument<'_>]) -> Vec<u8>,
}

struct WorldCountryDemiseEffects<'a> {
    base: WorldCountryExileResultEffects<'a>,
    organizing: &'a mut COrganizingCtrl,
    organizing_parameters: &'a COrganizingParam,
    attack_city: &'a CAttackCitySys,
    goods_war: &'a CGoodsWarMember,
    world_string: &'a mut dyn FnMut(&[u8]) -> Vec<u8>,
    refresh_owned_city: &'a mut dyn FnMut(i32, i32, i32),
    update_player: &'a mut dyn FnMut(i32),
    faction_master_log_enabled: bool,
    write_faction_master_log:
        &'a mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8]),
}

struct WorldCountryFactionDemiseEffects<'a> {
    game: &'a CGame,
    attack_city: &'a CAttackCitySys,
    goods_war: &'a CGoodsWarMember,
    world_string: &'a mut dyn FnMut(&[u8]) -> Vec<u8>,
    format_world_string:
        &'a mut dyn FnMut(&[u8], &[UnionFormatArgument<'_>]) -> Vec<u8>,
    update_player: &'a mut dyn FnMut(i32),
    country_id: u8,
    king_id: i32,
    demise_faction: bool,
    faction_master_log_enabled: bool,
    write_faction_master_log:
        &'a mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8]),
}

/// Concrete war/country/Goods-War owners внешнего `DisbandFaction`.
struct WorldOrganizingDisbandEffects<'a> {
    game: &'a CGame,
    village_war: &'a CVillageWarSys,
    attack_city: &'a CAttackCitySys,
    country_handler: &'a CCountryHandler,
    goods_war: &'a mut CGoodsWarMember,
    world_string: &'a mut dyn FnMut(&[u8]) -> Vec<u8>,
}

/// Отдельный immutable transport-view устраняет искусственную зависимость
/// Goods War delete-публикаций от mutable organizing lookup-а.
struct WorldGoodsWarDelivery<'a> {
    game: &'a CGame,
}

struct WorldFourNationWarResultEffects<'a> {
    game: &'a CGame,
}

struct WorldFourNationExploitEffects<'a> {
    game: &'a mut CGame,
}

struct WorldFourNationCountryFailEffects<'a> {
    game: &'a CGame,
    organizing: &'a COrganizingCtrl,
    globe_setup: &'a GlobeSetupSnapshot,
    format_world_string:
        &'a mut dyn FnMut(&[u8], &[UnionFormatArgument<'_>]) -> Vec<u8>,
}

impl CountryNewTermContext for WorldCountryExileResultEffects<'_> {
    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError> {
        message.send_all(self.game.current_game_server_sender().as_ref())
    }
}

impl CountryVillageTaxContext for WorldCountryExileResultEffects<'_> {
    fn village_regions(
        &mut self,
        country_id: u8,
    ) -> Result<Vec<CountryVillageTaxRegion>, CountryVillageTaxContextBlock> {
        let mut regions = Vec::new();
        for (&map_key, assignment) in &self.game.regions {
            let Some(region_type) = assignment.region_type else {
                return Err(CountryVillageTaxContextBlock::UninitializedRegionType { map_key });
            };
            if region_type != 1 {
                continue;
            }
            let Some(owner) = assignment.region.as_ref() else {
                continue;
            };
            let Some(region_country) = owner.base().region_base().country() else {
                return Err(CountryVillageTaxContextBlock::UninitializedRegionCountry { map_key });
            };
            if region_country == country_id {
                regions.push(CountryVillageTaxRegion {
                    map_key,
                    name: legacy_c_string_prefix(owner.base().get_name()).to_vec(),
                });
            }
        }
        Ok(regions)
    }

    fn country_name(&mut self, country_id: u8) -> Vec<u8> {
        self.globe_setup
            .country_name(country_id)
            .unwrap_or_default()
            .to_vec()
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
    ) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                CountryExileTextArgument::Text(text) => UnionFormatArgument::Text(text),
                CountryExileTextArgument::Signed(value) => UnionFormatArgument::Signed(*value),
            })
            .collect::<Vec<_>>();
        (self.format_world_string)(string_id, &arguments)
    }

    fn put_king_log(&mut self, text: &[u8]) {
        put_string_to_file("king", text);
    }
}

impl CountryNewTermContext for WorldCountryDemiseEffects<'_> {
    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError> {
        CountryNewTermContext::send_all(&mut self.base, message)
    }
}

impl CountryVillageTaxContext for WorldCountryDemiseEffects<'_> {
    fn village_regions(
        &mut self,
        country_id: u8,
    ) -> Result<Vec<CountryVillageTaxRegion>, CountryVillageTaxContextBlock> {
        CountryVillageTaxContext::village_regions(&mut self.base, country_id)
    }

    fn country_name(&mut self, country_id: u8) -> Vec<u8> {
        CountryVillageTaxContext::country_name(&mut self.base, country_id)
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
    ) -> Vec<u8> {
        CountryVillageTaxContext::format_world_string(
            &mut self.base,
            string_id,
            arguments,
        )
    }

    fn put_king_log(&mut self, text: &[u8]) {
        CountryVillageTaxContext::put_king_log(&mut self.base, text);
    }
}

impl FourNationWarResultContext for WorldFourNationWarResultEffects<'_> {
    fn game_server_number_by_region_id(&mut self, region_id: i32) -> i32 {
        self.game.game_server_number_by_region_id(region_id)
    }

    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError> {
        message.send_to_map_id(self.game.current_game_server_sender().as_ref(), map_id)
    }
}

impl CountryExileResultContext for WorldCountryExileResultEffects<'_> {
    fn map_player_name(&mut self, player_id: i32) -> Option<Vec<u8>> {
        self.game
            .map_player(player_id as u32)
            .map(|player| legacy_c_string_prefix(player.get_name()).to_vec())
    }

    fn online_player(&mut self, player_id: i32) -> Option<CountryExileTarget> {
        self.game
            .online_player_by_id(player_id as u32)
            .map(|player| CountryExileTarget {
                name: legacy_c_string_prefix(player.get_name()).to_vec(),
                country: player.country(),
                level: player.get_level(),
                credit: player.credit(),
                pk_count: player.pk_count(),
                is_god: player.is_god(),
            })
    }

    fn reset_online_player_murder_counters(
        &mut self,
        player_id: i32,
    ) -> Option<CountryAbsolveCounterReset> {
        self.game
            .reset_online_player_murder_counters(player_id as u32)
            .map(|reset| CountryAbsolveCounterReset {
                previous_kill_count: reset.previous_kill_count,
                previous_pk_count: reset.previous_pk_count,
            })
    }

    fn country_name(&mut self, country_id: u8) -> Vec<u8> {
        self.globe_setup
            .country_name(country_id)
            .unwrap_or_default()
            .to_vec()
    }

    fn country_identity_name(&mut self, identity: u8) -> Vec<u8> {
        self.globe_setup
            .country_identity_name(identity)
            .unwrap_or_default()
            .to_vec()
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
    ) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                CountryExileTextArgument::Text(text) => UnionFormatArgument::Text(text),
                CountryExileTextArgument::Signed(value) => UnionFormatArgument::Signed(*value),
            })
            .collect::<Vec<_>>();
        (self.format_world_string)(string_id, &arguments)
    }

    fn game_server_number_by_player_id(&mut self, player_id: i32) -> i32 {
        self.game.game_server_number_by_player_id(player_id)
    }

    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError> {
        message.send_to_map_id(self.game.current_game_server_sender().as_ref(), map_id)
    }

    fn send_to_connected_game_servers(
        &mut self,
        message: &CMessage,
    ) -> Vec<CountryExileMessageDelivery> {
        let sender = self.game.current_game_server_sender();
        self.game
            .connected_game_server_indices()
            .map(|map_id| CountryExileMessageDelivery {
                map_id,
                delivery: message.send_to_map_id(sender.as_ref(), map_id),
            })
            .collect()
    }

    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError> {
        message.send_all(self.game.current_game_server_sender().as_ref())
    }

    fn put_king_log(&mut self, text: &[u8]) {
        put_string_to_file("king", text);
    }
}

impl CountryPlayersListContext for WorldCountryPlayersListEffects<'_> {
    fn online_players(&mut self) -> Vec<CountryOnlinePlayer> {
        self.game
            .online_players
            .iter()
            .filter_map(|&player_id| self.game.map_player(player_id))
            .map(|player| CountryOnlinePlayer {
                id: player.get_id(),
                name: legacy_c_string_prefix(player.get_name()).to_vec(),
                country: player.country(),
                occupation: player.get_occupation(),
                level: player.get_level(),
                is_god: player.is_god(),
            })
            .collect()
    }

    fn player_faction(
        &mut self,
        player_id: i32,
    ) -> Result<(Vec<u8>, bool), CountryPlayersListContextBlock> {
        let faction_id = match self.organizing.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => 0,
            FreePlayerLookup::Faction(faction_id) => faction_id,
            FreePlayerLookup::BlockedNullFaction { .. } => {
                return Err(CountryPlayersListContextBlock::PlayerFactionLookup);
            }
        };
        let faction_name = if faction_id > 0 {
            self.organizing
                .faction_by_id(faction_id)
                .map(|faction| faction.name().to_vec())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let is_faction_master = self
            .organizing
            .faction_id_by_master_player(player_id)
            .map_err(|_| CountryPlayersListContextBlock::FactionMasterLookup)?
            != 0;
        Ok((faction_name, is_faction_master))
    }

    fn country_name(&mut self, country_id: u8) -> Vec<u8> {
        self.globe_setup
            .country_name(country_id)
            .unwrap_or_default()
            .to_vec()
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
    ) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                CountryExileTextArgument::Text(text) => UnionFormatArgument::Text(text),
                CountryExileTextArgument::Signed(value) => UnionFormatArgument::Signed(*value),
            })
            .collect::<Vec<_>>();
        (self.format_world_string)(string_id, &arguments)
    }

    fn game_server_number_by_player_id(&mut self, player_id: i32) -> i32 {
        self.game.game_server_number_by_player_id(player_id)
    }

    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError> {
        message.send_to_map_id(self.game.current_game_server_sender().as_ref(), map_id)
    }

    fn put_king_log(&mut self, text: &[u8]) {
        put_string_to_file("king", text);
    }
}

impl FactionOrganizingInfoContext for WorldCountryFactionDemiseEffects<'_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some((self.world_string)(string_id))
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl GoodsWarDeliveryContext for WorldGoodsWarDelivery<'_> {
    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }
}

impl FactionOrganizingInfoContext for WorldOrganizingDisbandEffects<'_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some((self.world_string)(string_id))
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionDisbandContext for WorldOrganizingDisbandEffects<'_> {
    fn village_war_declared(&self, faction_id: i32) -> bool {
        self.village_war.is_already_declared_for_war(faction_id)
    }

    fn city_war_declared(&self, faction_id: i32) -> bool {
        self.attack_city.is_already_declared_for_war(faction_id)
    }

    fn goods_war_blocks_disband(&self, faction_id: i32, _player_id: i32) -> bool {
        goods_war_check_for_faction_id(faction_id, |candidate| {
            self.goods_war.contains_faction_id(candidate)
        })
    }

    fn country_king_id(&self, country: u8) -> Option<i32> {
        self.country_handler
            .get_country(country)
            .map(|country| country.king.id)
    }

    fn delete_goods_war_members_by_faction_id(&mut self, faction_id: i32) {
        let mut delivery = WorldGoodsWarDelivery { game: self.game };
        let _ = self
            .goods_war
            .delete_members_by_faction_id(faction_id, &mut delivery);
    }

    fn decrement_goods_war_faction_count(&mut self, _faction_id: i32, faction_name: &[u8]) {
        let mut delivery = WorldGoodsWarDelivery { game: self.game };
        let _ = self
            .goods_war
            .delete_one_faction_count_by_name(faction_name, &mut delivery);
    }
}

impl FactionDemiseContext for WorldCountryFactionDemiseEffects<'_> {
    fn attack_city_system_declared(&self, faction_id: i32) -> bool {
        self.attack_city.is_already_declared_for_war(faction_id)
    }

    fn goods_war_blocks_demise(&self, faction_id: i32, _old_master_id: i32) -> bool {
        goods_war_check_for_faction_id(faction_id, |candidate| {
            self.goods_war.contains_faction_id(candidate)
        })
    }

    fn country_blocks_demise(&self, country: u8, old_master_id: i32) -> bool {
        country == self.country_id
            && old_master_id == self.king_id
            && !self.demise_faction
    }

    fn format_demise_signed(
        &mut self,
        string_id: &'static [u8],
        value: i32,
    ) -> Vec<u8> {
        (self.format_world_string)(string_id, &[UnionFormatArgument::Signed(value)])
    }

    fn format_demise_change(
        &mut self,
        string_id: &'static [u8],
        old_master_name: &[u8],
        new_master_name: &[u8],
    ) -> Vec<u8> {
        (self.format_world_string)(
            string_id,
            &[
                UnionFormatArgument::Text(old_master_name),
                UnionFormatArgument::Text(new_master_name),
            ],
        )
    }

    fn update_player_faction_info(&mut self, player_id: i32) {
        (self.update_player)(player_id);
    }

    fn faction_master_log_enabled(&self) -> bool {
        self.faction_master_log_enabled
    }

    fn write_faction_master_log(
        &mut self,
        old_master_id: i32,
        old_master_name: &[u8],
        new_master_id: i32,
        new_master_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
    ) {
        (self.write_faction_master_log)(
            old_master_id,
            old_master_name,
            new_master_id,
            new_master_name,
            faction_id,
            faction_name,
        );
    }
}

impl CountryExileResultContext for WorldCountryDemiseEffects<'_> {
    fn map_player_name(&mut self, player_id: i32) -> Option<Vec<u8>> {
        self.base.map_player_name(player_id)
    }

    fn online_player(&mut self, player_id: i32) -> Option<CountryExileTarget> {
        self.base.online_player(player_id)
    }

    fn reset_online_player_murder_counters(
        &mut self,
        player_id: i32,
    ) -> Option<CountryAbsolveCounterReset> {
        self.base.reset_online_player_murder_counters(player_id)
    }

    fn faction_id_by_master_player(
        &mut self,
        player_id: i32,
    ) -> Result<i32, CountryGovernanceContextBlock> {
        self.organizing
            .faction_id_by_master_player(player_id)
            .map_err(|_| CountryGovernanceContextBlock::FactionMasterLookup)
    }

    fn faction_id_by_player(
        &mut self,
        player_id: i32,
    ) -> Result<i32, CountryGovernanceContextBlock> {
        match self.organizing.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => Ok(0),
            FreePlayerLookup::Faction(faction_id) => Ok(faction_id),
            FreePlayerLookup::BlockedNullFaction { .. } => {
                Err(CountryGovernanceContextBlock::PlayerFactionLookup)
            }
        }
    }

    fn faction_snapshot(&mut self, faction_id: i32) -> Option<CountryFactionSnapshot> {
        self.organizing.faction_by_id(faction_id).map(|faction| CountryFactionSnapshot {
            faction_id: faction.faction_id(),
            name: faction.name().to_vec(),
            owned_cities: faction.owned_cities().iter().copied().collect(),
        })
    }

    fn union_id_for_faction(
        &mut self,
        faction_id: i32,
    ) -> Result<i32, CountryGovernanceContextBlock> {
        match self.organizing.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => Ok(0),
            FreeFactionLookup::Union(union_id) => Ok(union_id),
            FreeFactionLookup::BlockedNullConfederation { .. } => {
                Err(CountryGovernanceContextBlock::UnionLookup)
            }
        }
    }

    fn clear_faction_owned_cities(
        &mut self,
        faction_id: i32,
    ) -> Result<(), CountryGovernanceContextBlock> {
        self.organizing
            .faction_by_id_mut(faction_id)
            .ok_or(CountryGovernanceContextBlock::OwnedCityMutation)?
            .clear_owned_cities(&*self.base.game, &mut *self.update_player)
            .map(|_| ())
            .map_err(|_| CountryGovernanceContextBlock::OwnedCityMutation)
    }

    fn add_faction_owned_city(
        &mut self,
        faction_id: i32,
        city_id: i32,
    ) -> Result<(), CountryGovernanceContextBlock> {
        self.organizing
            .faction_by_id_mut(faction_id)
            .ok_or(CountryGovernanceContextBlock::OwnedCityMutation)?
            .add_owned_city(&*self.base.game, city_id, &mut *self.update_player)
            .map(|_| ())
            .map_err(|_| CountryGovernanceContextBlock::OwnedCityMutation)
    }

    fn refresh_owned_city(&mut self, city_id: i32, faction_id: i32, union_id: i32) {
        (self.refresh_owned_city)(city_id, faction_id, union_id);
    }

    fn demise_faction(
        &mut self,
        faction_id: i32,
        old_master_id: i32,
        new_master_id: i32,
        country_id: u8,
        king_id: i32,
        demise_faction: bool,
    ) -> Result<bool, CountryGovernanceContextBlock> {
        let Some(faction) = self.organizing.faction_by_id_mut(faction_id) else {
            return Ok(false);
        };
        let mut effects = WorldCountryFactionDemiseEffects {
            game: &*self.base.game,
            attack_city: self.attack_city,
            goods_war: self.goods_war,
            world_string: &mut *self.world_string,
            format_world_string: &mut *self.base.format_world_string,
            update_player: &mut *self.update_player,
            country_id,
            king_id,
            demise_faction,
            faction_master_log_enabled: self.faction_master_log_enabled,
            write_faction_master_log: &mut *self.write_faction_master_log,
        };
        faction
            .demise(
                &*self.base.game,
                self.organizing_parameters,
                old_master_id,
                new_master_id,
                &mut effects,
            )
            .map(|outcome| matches!(outcome, FactionDemiseOutcome::Transferred(_)))
            .map_err(|_| CountryGovernanceContextBlock::FactionDemise)
    }

    fn current_tick_ms(&mut self) -> u32 {
        legacy_tick_ms()
    }

    fn country_name(&mut self, country_id: u8) -> Vec<u8> {
        CountryExileResultContext::country_name(&mut self.base, country_id)
    }

    fn country_identity_name(&mut self, identity: u8) -> Vec<u8> {
        self.base.country_identity_name(identity)
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
    ) -> Vec<u8> {
        CountryExileResultContext::format_world_string(
            &mut self.base,
            string_id,
            arguments,
        )
    }

    fn game_server_number_by_player_id(&mut self, player_id: i32) -> i32 {
        self.base.game_server_number_by_player_id(player_id)
    }

    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError> {
        self.base.send_to_map_id(message, map_id)
    }

    fn send_to_connected_game_servers(
        &mut self,
        message: &CMessage,
    ) -> Vec<CountryExileMessageDelivery> {
        self.base.send_to_connected_game_servers(message)
    }

    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError> {
        CountryExileResultContext::send_all(&mut self.base, message)
    }

    fn put_king_log(&mut self, text: &[u8]) {
        CountryExileResultContext::put_king_log(&mut self.base, text);
    }
}

impl FourNationExploitContext for WorldFourNationExploitEffects<'_> {
    fn map_player_exists(&mut self, player_id: u32) -> bool {
        self.game.map_player(player_id).is_some()
    }

    fn player_game_server_map_id(&mut self, player_id: i32) -> Option<i32> {
        self.game
            .player_game_server(player_id)
            .map(|game_server| game_server.index as i32)
    }

    fn add_local_player_exploit(
        &mut self,
        player_id: u32,
        increment: i32,
    ) -> Option<PlayerExploitUpdate> {
        self.game
            .add_map_player_exploit_wrapping(player_id, increment)
    }

    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError> {
        message.send_to_map_id(self.game.current_game_server_sender().as_ref(), map_id)
    }
}

impl FourNationCountryFailContext for WorldFourNationCountryFailEffects<'_> {
    fn country_name(&mut self, country: i32) -> Vec<u8> {
        u8::try_from(country)
            .ok()
            .and_then(|country| self.globe_setup.country_name(country))
            .map(|name| name[..name.len().min(9)].to_vec())
            .unwrap_or_default()
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[&[u8]],
    ) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| UnionFormatArgument::Text(argument))
            .collect::<Vec<_>>();
        (self.format_world_string)(string_id, &arguments)
    }

    fn send_top_info(&mut self, text: &[u8]) -> Result<i32, SendMessageError> {
        self.organizing
            .send_top_info_to_client(self.game, -1, 1, 2, text)
    }
}

impl CountryWarDeclarationContext for WorldCountryWarEffects<'_> {
    fn online_player_country(&mut self, player_id: i32) -> CountryWarDeclarationPlayer {
        let Some(player) = self.game.online_player_by_id(player_id as u32) else {
            return CountryWarDeclarationPlayer::Missing;
        };
        match player.country() {
            Some(country) => CountryWarDeclarationPlayer::Country(country),
            None => CountryWarDeclarationPlayer::CountryUnavailable,
        }
    }

    fn declaration_authority(
        &mut self,
        country: u8,
        player_id: i32,
    ) -> CountryWarDeclarationAuthority {
        let Some(owner) = self.country_handler.get_country(country) else {
            return CountryWarDeclarationAuthority::CountryMissing;
        };
        let is_king = owner.has_king_id(player_id);
        let is_minister = owner.has_minister_id(player_id);
        let country_name = self
            .globe_setup
            .country_name(country)
            .unwrap_or_default()
            .to_vec();

        if is_king {
            return CountryWarDeclarationAuthority::Authorized;
        }
        let king_log = (self.format_world_string)(
            b"WS0034",
            &[UnionFormatArgument::Text(&country_name)],
        );
        let king_log = legacy_c_string_prefix(&king_log);
        put_string_to_file("king", &king_log[..king_log.len().min(0x103)]);

        if is_minister {
            return CountryWarDeclarationAuthority::Authorized;
        }
        let identity_name = self
            .globe_setup
            .country_identity_name(5)
            .unwrap_or_default();
        let minister_log = (self.format_world_string)(
            b"WS0037",
            &[
                UnionFormatArgument::Text(&country_name),
                UnionFormatArgument::Text(identity_name),
            ],
        );
        let minister_log = legacy_c_string_prefix(&minister_log);
        put_string_to_file(
            "king",
            &minister_log[..minister_log.len().min(0x103)],
        );
        CountryWarDeclarationAuthority::Rejected
    }

    fn region(&mut self, region_id: i32) -> Option<CountryWarVictoryRegion> {
        match self.game.region_name(region_id) {
            WorldRegionNameLookup::Name(name) => Some(CountryWarVictoryRegion {
                name: name.to_vec(),
            }),
            WorldRegionNameLookup::RegionNotFound
            | WorldRegionNameLookup::NullRegionPointer => None,
        }
    }

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        (self.world_string)(string_id)
    }

    fn format_declaration_notice(
        &mut self,
        attack_country: u8,
        defend_country: i32,
        region_name: &[u8],
    ) -> Vec<u8> {
        let attack_name = self
            .globe_setup
            .country_name(attack_country)
            .unwrap_or_default();
        let defend_name = u8::try_from(defend_country)
            .ok()
            .and_then(|country| self.globe_setup.country_name(country))
            .unwrap_or_default();
        let notice = (self.format_world_string)(
            b"WS0104",
            &[
                UnionFormatArgument::Text(attack_name),
                UnionFormatArgument::Text(defend_name),
                UnionFormatArgument::Text(region_name),
            ],
        );
        let notice = legacy_c_string_prefix(&notice);
        notice[..notice.len().min(0x1ff)].to_vec()
    }

    fn send_private_to_country_king(
        &mut self,
        country: u8,
        text: &[u8],
    ) -> Option<Result<i32, SendMessageError>> {
        let text = legacy_c_string_prefix(text);
        if text.is_empty() {
            return None;
        }
        let king_id = self.country_handler.get_country(country)?.king.id;
        let map_id = self.game.game_server_number_by_player_id(king_id);
        if map_id == 0 {
            return None;
        }
        let text = CString::new(text).expect("legacy C-string prefix не содержит NUL");
        let mut message = CMessage::new(0x7ff13);
        message.base_mut().add_long(king_id);
        message.base_mut().add_str(Some(&text));
        Some(message.send_to_map_id(self.game.current_game_server_sender().as_ref(), map_id))
    }

    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError> {
        message.send_all(self.game.current_game_server_sender().as_ref())
    }

    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError> {
        message.send_to_map_id(self.game.current_game_server_sender().as_ref(), map_id)
    }

    fn send_country_info(&mut self, text: &[u8], title: u32, color: u32) -> i32 {
        let text = CString::new(legacy_c_string_prefix(text))
            .expect("legacy C-string prefix не содержит внутреннего NUL");
        let mut delivery = WorldCountryInfoDelivery { game: self.game };
        self.country_handler
            .send_info_to_client(&text, title, color, &mut delivery)
    }
}

impl CountryWarVictoryContext for WorldCountryWarEffects<'_> {
    type Block = Infallible;

    fn region(
        &mut self,
        region_id: i32,
    ) -> Result<Option<CountryWarVictoryRegion>, Self::Block> {
        Ok(match self.game.region_name(region_id) {
            WorldRegionNameLookup::Name(name) => Some(CountryWarVictoryRegion {
                name: name.to_vec(),
            }),
            WorldRegionNameLookup::RegionNotFound
            | WorldRegionNameLookup::NullRegionPointer => None,
        })
    }

    fn country_exists(&mut self, country: u8) -> Result<bool, Self::Block> {
        Ok(self.country_handler.get_country(country).is_some())
    }

    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }

    fn set_country_war_result(
        &mut self,
        country: u8,
        result: i32,
    ) -> Result<(), Self::Block> {
        let _ = self
            .country_handler
            .set_country_war_result(country, result);
        Ok(())
    }

    fn format_victory_notice(
        &mut self,
        string_id: &'static [u8],
        attack_country: i32,
        defend_country: i32,
        region_name: &[u8],
    ) -> Result<Vec<u8>, Self::Block> {
        let attack_name = u8::try_from(attack_country)
            .ok()
            .and_then(|country| self.globe_setup.country_name(country))
            .unwrap_or_default();
        let defend_name = u8::try_from(defend_country)
            .ok()
            .and_then(|country| self.globe_setup.country_name(country))
            .unwrap_or_default();
        let formatted = (self.format_world_string)(
            string_id,
            &[
                UnionFormatArgument::Text(attack_name),
                UnionFormatArgument::Text(defend_name),
                UnionFormatArgument::Text(region_name),
            ],
        );
        let visible = legacy_c_string_prefix(&formatted);
        // Нормальный output сохраняется byte-exact; переполнение старого
        // 256-byte `_sprintf` было внутренним UB, поэтому safe adapter
        // оставляет место под C-string NUL вместо чтения за stack-buffer.
        Ok(visible[..visible.len().min(0xff)].to_vec())
    }

    fn send_country_info(
        &mut self,
        text: &[u8],
        title: u32,
        color: u32,
    ) -> Result<i32, Self::Block> {
        let text = CString::new(legacy_c_string_prefix(text))
            .expect("legacy C-string prefix не содержит внутреннего NUL");
        let mut delivery = WorldCountryInfoDelivery { game: self.game };
        Ok(self
            .country_handler
            .send_info_to_client(&text, title, color, &mut delivery))
    }
}

impl CountryWarPhaseContext for WorldCountryWarEffects<'_> {
    type Block = Infallible;

    fn reset_country_war_result_if_present(
        &mut self,
        country: u8,
    ) -> Result<bool, Self::Block> {
        Ok(self.country_handler.set_country_war_result(country, 0))
    }

    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError> {
        message.send_all(self.game.current_game_server_sender().as_ref())
    }

    fn format_phase_notice(
        &mut self,
        string_id: &'static [u8],
    ) -> Result<Vec<u8>, Self::Block> {
        let formatted = (self.format_world_string)(string_id, &[]);
        let visible = legacy_c_string_prefix(&formatted);
        Ok(visible[..visible.len().min(0xff)].to_vec())
    }

    fn send_country_info(
        &mut self,
        text: &[u8],
        title: u32,
        color: u32,
    ) -> Result<i32, Self::Block> {
        let text = CString::new(legacy_c_string_prefix(text))
            .expect("legacy C-string prefix не содержит внутреннего NUL");
        let mut delivery = WorldCountryInfoDelivery { game: self.game };
        Ok(self
            .country_handler
            .send_info_to_client(&text, title, color, &mut delivery))
    }
}

impl CountryWarTopInfoContext for WorldCountryWarEffects<'_> {
    type Block = Infallible;

    fn format_top_info_notice(
        &mut self,
        string_id: &'static [u8],
    ) -> Result<Vec<u8>, Self::Block> {
        let formatted = (self.format_world_string)(string_id, &[]);
        let visible = legacy_c_string_prefix(&formatted);
        Ok(visible[..visible.len().min(0xff)].to_vec())
    }

    fn add_top_info(
        &mut self,
        timer_flag: i32,
        duration_ms: i32,
        text: &[u8],
        get_tick: &mut dyn FnMut() -> u32,
    ) -> i32 {
        self.country_handler
            .add_one_top_info(timer_flag, duration_ms, text, get_tick)
    }

    fn send_top_info(
        &mut self,
        top_info_id: i32,
        timer_flag: i32,
        duration_ms: i32,
        text: &[u8],
    ) -> i32 {
        let mut delivery = WorldCountryInfoDelivery { game: self.game };
        self.country_handler.send_top_info_to_client(
            top_info_id,
            timer_flag,
            duration_ms,
            text,
            &mut delivery,
        )
    }
}

async fn process_world_message<TimerCallback, JjcContext>(
    game: &mut CGame,
    honor_ranks: &mut CHonorRanks,
    increment_log: &mut CIncrementLog,
    auction_log: &mut CAuctionLog,
    organizing: &mut COrganizingCtrl,
    organizing_parameters: &COrganizingParam,
    country_handler: &mut CCountryHandler,
    country_parameters: &mut CCountryParam,
    player_list: &mut CPlayerList,
    country_war: &mut CountryWarSys,
    four_nation_war: &mut CFourNationWarSys,
    country_limits: CountryKingSaveLimits,
    faction_war_sys: &mut CFactionWarSys,
    attack_city: &mut CAttackCitySys,
    attack_city_callbacks: AttackCityCallbacks<TimerCallback>,
    globe_setup: &GlobeSetupSnapshot,
    region_router: &RegionRouter,
    village_war: &mut CVillageWarSys,
    goods_war: &mut CGoodsWarMember,
    timer: &mut CTimer<TimerCallback>,
    village_war_callbacks: VillageWarCallbacks<TimerCallback>,
    registry: &GoodsBasePropertiesRegistry,
    original_name_index: &GoodsOriginalNameIndex,
    coefficients: &PlayerPropertyCoefficients,
    load_player_largess: &mut dyn FnMut(&mut CPlayer),
    net_sessions: &CNetSessionManager,
    jjc: &mut CJJcSystem,
    jjc_config: JjcRunConfig,
    jjc_context: &mut JjcContext,
    application_runtime: &WorldUnionApplicationRuntimeOwner,
    application_callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    check_invalid_organizing_string: &mut dyn FnMut(&mut Vec<u8>, bool) -> bool,
    faction_chat_log_enabled: bool,
    private_chat_log_enabled: bool,
    delete_log_enabled: bool,
    faction_create_log_enabled: bool,
    write_faction_create_log: &mut dyn FnMut(i32, &[u8], i32, &[u8]),
    faction_title_log_enabled: bool,
    write_faction_title_log:
        &mut dyn FnMut(i32, &[u8], &[u8], &[u8], i32, &[u8], i32, &[u8]),
    faction_purview_add_log_enabled: bool,
    faction_purview_revoke_log_enabled: bool,
    write_faction_purview_log:
        &mut dyn FnMut(i32, &[u8], i32, i32, &[u8], i32, &[u8], i32),
    faction_apply_log_enabled: bool,
    write_faction_apply_log:
        &mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
    faction_join_log_enabled: bool,
    write_faction_join_log:
        &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
    faction_quit_log_enabled: bool,
    write_faction_quit_log:
        &mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
    faction_fire_out_log_enabled: bool,
    write_faction_fire_out_log:
        &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
    faction_master_log_enabled: bool,
    write_faction_master_log:
        &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8]),
    faction_disband_log_enabled: bool,
    write_faction_disband_log: &mut dyn FnMut(i32, &[u8], i32, &[u8]),
    rs_player: &mut TiberiusRsPlayer,
    mut player_database: Option<&mut WorldTdsClient>,
    save_thread_handle: &mut WorldSaveThreadHandleState,
    launch_save_thread: &mut dyn FnMut(
        &WorldSaveThreadLaunchRequest,
    ) -> WorldSaveThreadHandleState,
    session_factory: &mut CSessionFactory,
    general_variables: Option<&mut CVariableList>,
    gods_battle: &mut CGodsBattleConf,
    rs_gods_battle: Option<&mut TiberiusRsGodsBattle>,
    gods_battle_database: Option<&mut WorldTdsClient>,
    reload_context: &mut dyn WorldReloadContext,
    add_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    update_player: &mut dyn FnMut(i32),
    source: WorldMessageSource,
    mut message: CMessage,
) -> ProcessedWorldEvent
where
    TimerCallback: Copy,
    JjcContext: JjcRunContext + ?Sized,
{
    let message_type = message.message_type();
    let mut selector = WorldOwnerSelector {
        write_log_enabled: game.setup.use_log_system,
        owner: None,
    };
    let legacy_run_result = message.run(&mut selector);

    if selector.owner == Some(WorldMessageOwner::Server) {
        match on_server_message(
            game,
            message,
            registry,
            coefficients,
            organizing,
            faction_war_sys,
            country_handler,
            country_limits,
            honor_ranks,
            save_thread_handle,
            launch_save_thread,
            add_log_text,
            session_factory,
            general_variables,
            gods_battle,
            rs_gods_battle,
            gods_battle_database,
        )
        .await
        {
            WorldServerMessageDispatch::Handled(outcome) => {
                return ProcessedWorldEvent::ServerMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldServerMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::Log) {
        let delete_log_enabled = game.setup.use_log_system && delete_log_enabled;
        match on_log_message(
            game,
            organizing,
            organizing_parameters,
            country_handler,
            country_parameters,
            player_list,
            session_factory,
            registry,
            original_name_index,
            coefficients,
            load_player_largess,
            globe_setup,
            rs_player,
            player_database.as_deref_mut(),
            &mut *application_callbacks.format_world_string,
            delete_log_enabled,
            add_log_text,
            &mut *application_callbacks.random,
            message,
        )
        .await
        {
            WorldLogMessageDispatch::Handled(outcome) => {
                return ProcessedWorldEvent::LogMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldLogMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::Player) {
        match on_player_message(game, message) {
            WorldPlayerMessageDispatch::Handled(outcome) => {
                return ProcessedWorldEvent::PlayerMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldPlayerMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::Gma) {
        match on_gma_message(game, message, add_log_text) {
            WorldGmaMessageDispatch::Handled(outcome) => {
                return ProcessedWorldEvent::GmaMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldGmaMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::Other) {
        let faction_chat_log_enabled =
            game.setup.use_log_system && faction_chat_log_enabled;
        let private_chat_log_enabled =
            game.setup.use_log_system && private_chat_log_enabled;
        match on_other_message(
            game,
            organizing,
            honor_ranks,
            increment_log,
            globe_setup,
            registry,
            &mut *application_callbacks.random,
            rs_player,
            player_database.as_deref_mut(),
            faction_chat_log_enabled,
            private_chat_log_enabled,
            add_log_text,
            message,
        )
        .await
        {
            WorldOtherMessageDispatch::Handled(outcome) => {
                return ProcessedWorldEvent::OtherMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldOtherMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::Gm) {
        match on_gm_message(
            game,
            jjc,
            rs_player,
            player_database.as_deref_mut(),
            reload_context,
            &mut *application_callbacks.world_string,
            message,
        )
        .await
        {
            WorldGmMessageDispatch::Handled(outcome) => {
                return ProcessedWorldEvent::GmMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldGmMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::Team) {
        let outcome = on_team_message(game, session_factory, &mut message);
        return ProcessedWorldEvent::TeamMessage {
            source,
            legacy_run_result,
            outcome,
        };
    }

    if selector.owner == Some(WorldMessageOwner::JjcSystem) {
        let outcome = on_jjc_system_message(game, jjc, jjc_config, jjc_context, &mut message);
        return ProcessedWorldEvent::JjcMessage {
            source,
            legacy_run_result,
            outcome,
        };
    }

    if selector.owner == Some(WorldMessageOwner::WriteLog) {
        match on_write_log_message(game, increment_log, auction_log, add_log_text, message) {
            WorldWriteLogMessageDispatch::Handled(outcome) => {
                return ProcessedWorldEvent::WriteLogMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldWriteLogMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::Country) {
        if let Some(request) = decode_four_nation_exploit_message(&mut message) {
            let initial = {
                let mut effects = WorldFourNationExploitEffects { game };
                four_nation_war.convert_loaded_morale_to_exploit(
                    request.player_id,
                    request.increment,
                    &mut effects,
                )
            };
            let mut database = WorldFourNationExploitDatabaseDisposition::NotRequired;
            let mut after_database = None;

            if matches!(
                &initial.disposition,
                FourNationExploitLoadedDisposition::PlayerMissing
            ) {
                match player_database.as_deref_mut() {
                    None => {
                        let log = add_log_text(b"Error:failed to connect to DB!!");
                        database =
                            WorldFourNationExploitDatabaseDisposition::ConnectionUnavailable {
                                log,
                            };
                        let mut effects = WorldFourNationExploitEffects { game };
                        after_database = Some(
                            four_nation_war.convert_loaded_morale_to_exploit(
                                request.player_id,
                                request.increment,
                                &mut effects,
                            ),
                        );
                    }
                    Some(active_database) => {
                        let mut query = Query::new(
                            "UPDATE CSL_PLAYER_ABILITY SET Exploit = Exploit + @P1 WHERE ID = @P2",
                        );
                        query.bind(request.increment);
                        query.bind(request.player_id);
                        match query.execute(&mut *active_database).await {
                            Ok(_) => {
                                database = WorldFourNationExploitDatabaseDisposition::Applied;
                                let mut effects = WorldFourNationExploitEffects { game };
                                after_database = Some(
                                    four_nation_war.convert_loaded_morale_to_exploit(
                                        request.player_id,
                                        request.increment,
                                        &mut effects,
                                    ),
                                );
                            }
                            Err(error) => {
                                database = WorldFourNationExploitDatabaseDisposition::ExecutionFailed {
                                    error: error.to_string(),
                                };
                            }
                        }
                    }
                }
            }

            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::FourNationExploit(
                    WorldFourNationExploitSync {
                        request,
                        initial,
                        database,
                        after_database,
                    },
                ),
            };
        }
        if let Some(sync) = dispatch_country_player_change_message(
            &mut message,
            game,
            &*country_handler,
        ) {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::PlayerCountryChanged(sync),
            };
        }
        let new_day = {
            let faction_master_log_enabled =
                game.setup.use_log_system && faction_master_log_enabled;
            let base = WorldCountryExileResultEffects {
                game,
                globe_setup,
                format_world_string: &mut *application_callbacks.format_world_string,
            };
            let mut effects = WorldCountryDemiseEffects {
                base,
                organizing,
                organizing_parameters,
                attack_city: &*attack_city,
                goods_war: &*goods_war,
                world_string: &mut *application_callbacks.world_string,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                update_player,
                faction_master_log_enabled,
                write_faction_master_log: &mut *write_faction_master_log,
            };
            dispatch_country_new_day_message(
                &message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = new_day {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::NewDaySet(sync),
            };
        }
        let direct_appointment = {
            let faction_master_log_enabled =
                game.setup.use_log_system && faction_master_log_enabled;
            let base = WorldCountryExileResultEffects {
                game,
                globe_setup,
                format_world_string: &mut *application_callbacks.format_world_string,
            };
            let mut effects = WorldCountryDemiseEffects {
                base,
                organizing,
                organizing_parameters,
                attack_city: &*attack_city,
                goods_war: &*goods_war,
                world_string: &mut *application_callbacks.world_string,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                update_player,
                faction_master_log_enabled,
                write_faction_master_log: &mut *write_faction_master_log,
            };
            dispatch_country_direct_appointment_message(
                &mut message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = direct_appointment {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::CountryAppointedDirectly(sync),
            };
        }
        let country_info = {
            let mut effects = WorldCountryExileResultEffects {
                game,
                globe_setup,
                format_world_string: &mut *application_callbacks.format_world_string,
            };
            dispatch_country_info_message(
                &mut message,
                &*country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = country_info {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::CountryInfoSent(sync),
            };
        }
        let players_list = {
            let mut effects = WorldCountryPlayersListEffects {
                game: &*game,
                organizing: &*organizing,
                globe_setup,
                format_world_string: &mut *application_callbacks.format_world_string,
            };
            dispatch_country_players_list_message(
                &mut message,
                &*country_handler,
                &mut effects,
            )
        };
        if let Some(sync) = players_list {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::CountryPlayersListed(sync),
            };
        }
        let exile_result = {
            let mut effects = WorldCountryExileResultEffects {
                game,
                globe_setup,
                format_world_string: &mut *application_callbacks.format_world_string,
            };
            dispatch_country_exile_result_message(
                &mut message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = exile_result {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::ExileResultSynchronized(sync),
            };
        }
        let silence_request = {
            let mut effects = WorldCountryExileResultEffects {
                game,
                globe_setup,
                format_world_string: &mut *application_callbacks.format_world_string,
            };
            dispatch_country_silence_request_message(
                &mut message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = silence_request {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::SilenceRequested(sync),
            };
        }
        let absolve_request = {
            let mut effects = WorldCountryExileResultEffects {
                game,
                globe_setup,
                format_world_string: &mut *application_callbacks.format_world_string,
            };
            dispatch_country_absolve_request_message(
                &mut message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = absolve_request {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::AbsolveRequested(sync),
            };
        }
        let demise = {
            let faction_master_log_enabled =
                game.setup.use_log_system && faction_master_log_enabled;
            let base = WorldCountryExileResultEffects {
                game,
                globe_setup,
                format_world_string: &mut *application_callbacks.format_world_string,
            };
            let mut effects = WorldCountryDemiseEffects {
                base,
                organizing,
                organizing_parameters,
                attack_city: &*attack_city,
                goods_war: &*goods_war,
                world_string: &mut *application_callbacks.world_string,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                update_player,
                faction_master_log_enabled,
                write_faction_master_log: &mut *write_faction_master_log,
            };
            dispatch_country_demise_message(
                &mut message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = demise {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::KingDemised(sync),
            };
        }
        let depose_minister = {
            let mut effects = WorldCountryExileResultEffects {
                game,
                globe_setup,
                format_world_string: &mut *application_callbacks.format_world_string,
            };
            dispatch_country_depose_minister_message(
                &mut message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = depose_minister {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::MinisterDeposed(sync),
            };
        }
        let appoint_minister = {
            let mut effects = WorldCountryExileResultEffects {
                game,
                globe_setup,
                format_world_string: &mut *application_callbacks.format_world_string,
            };
            dispatch_country_appoint_minister_message(
                &mut message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = appoint_minister {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::MinisterAppointed(sync),
            };
        }
        let exile_request = {
            let mut effects = WorldCountryExileResultEffects {
                game,
                globe_setup,
                format_world_string: &mut *application_callbacks.format_world_string,
            };
            dispatch_country_exile_request_message(
                &mut message,
                country_handler,
                country_parameters,
                &mut effects,
            )
        };
        if let Some(sync) = exile_request {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::ExileRequested(sync),
            };
        }
        let four_nation_country_fail = {
            let mut effects = WorldFourNationCountryFailEffects {
                game,
                organizing: &*organizing,
                globe_setup,
                format_world_string: &mut *application_callbacks.format_world_string,
            };
            dispatch_four_nation_country_fail_message(&mut message, &mut effects)
        };
        if let Some(sync) = four_nation_country_fail {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::FourNationCountryFail(sync),
            };
        }
        let four_nation_war_time = {
            let mut effects = WorldFourNationWarResultEffects { game };
            dispatch_four_nation_war_time_message(
                &mut message,
                four_nation_war,
                &mut effects,
            )
        };
        if let Some(sync) = four_nation_war_time {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::FourNationWarTime(sync),
            };
        }
        let four_nation_result = {
            let mut effects = WorldFourNationWarResultEffects { game };
            dispatch_four_nation_war_result_message(
                &mut message,
                four_nation_war,
                &mut effects,
            )
        };
        if let Some(sync) = four_nation_result {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::FourNationWarResult(sync),
            };
        }
        let declaration = {
            let mut effects = WorldCountryWarEffects {
                game,
                country_handler,
                globe_setup,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
            };
            dispatch_country_war_declaration_message(&mut message, country_war, &mut effects)
        };
        if let Some(sync) = declaration {
            return ProcessedWorldEvent::CountryMessage {
                source,
                legacy_run_result,
                outcome: WorldCountryMessageOutcome::CountryWarDeclared(sync),
            };
        }
        let victory = {
            let mut effects = WorldCountryWarEffects {
                game,
                country_handler,
                globe_setup,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
            };
            dispatch_country_war_victory_message(&mut message, country_war, &mut effects)
        };
        match victory {
            Ok(Some(sync)) => {
                return ProcessedWorldEvent::CountryMessage {
                    source,
                    legacy_run_result,
                    outcome: WorldCountryMessageOutcome::CountryWarVictory(sync),
                };
            }
            Ok(None) => {}
            Err(block) => match block.source {},
        }
        match on_country_message(
            game,
            country_handler,
            country_parameters,
            globe_setup,
            message,
        ) {
            WorldCountryMessageDispatch::Handled(outcome) => {
                return ProcessedWorldEvent::CountryMessage {
                    source,
                    legacy_run_result,
                    outcome,
                };
            }
            WorldCountryMessageDispatch::Pending(pending) => message = pending,
        }
    }

    if selector.owner == Some(WorldMessageOwner::OrganizingSystem) {
        if let Some(outcome) = dispatch_faction_war_player_died(
            &mut message,
            game,
            organizing,
            faction_war_sys,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionWarPlayerDied {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        let faction_create = dispatch_create_faction(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            country_handler,
            registry,
            original_name_index,
            coefficients,
            rs_player,
            player_database.as_deref_mut(),
            application_callbacks,
            check_invalid_organizing_string,
            game.setup.use_log_system && faction_create_log_enabled,
            write_faction_create_log,
        )
        .await;
        if let Some(outcome) = faction_create {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingCreateFaction {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) =
            dispatch_initial_organizing_data(&mut message, game, organizing)
        {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingInitialData {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_leave_word(&mut message, game, organizing) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingLeaveWord {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_leave_word_edit(&mut message, game, organizing) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingLeaveWordEdit {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_pronounce(&mut message, game, organizing) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingPronounce {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        let game_server_sender = game.current_game_server_sender();
        if let Some(outcome) = dispatch_declare_faction_war(
            &mut message,
            game,
            organizing,
            faction_war_sys,
            registry,
            coefficients,
            application_callbacks,
            update_player,
            game_server_sender.as_ref(),
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingDeclareFactionWar {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        let game_server_sender = game.current_game_server_sender();
        if let Some(outcome) = dispatch_faction_billboard(
            &mut message,
            organizing,
            &mut *application_callbacks.world_string,
            game_server_sender.as_ref(),
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionBillboard {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        let use_log_system = game.setup.use_log_system;
        if let Some(outcome) = dispatch_faction_upgrade(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            registry,
            original_name_index,
            coefficients,
            use_log_system,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionUpgrade {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_upload_icon(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            application_callbacks,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionUploadIcon {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_contributor(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionContributor {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_experience(
            &mut message,
            game,
            organizing,
            use_log_system,
            application_callbacks,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionExperience {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_member_state(
            &mut message,
            game,
            organizing,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionMemberState {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        let game_server_sender = game.current_game_server_sender();
        if let Some(outcome) = dispatch_region_param_update(
            &mut message,
            game,
            game_server_sender.as_ref(),
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingRegionParamUpdate {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) =
            dispatch_goods_war_command(&mut message, game, organizing, goods_war)
        {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingGoodsWarCommand {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) =
            dispatch_goods_war_faction_win(&mut message, game, organizing, goods_war)
        {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingGoodsWarFactionWin {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_player_quest_command(
            &mut message,
            game,
            game.current_game_server_sender().as_ref(),
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingPlayerQuestCommand {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_player_run_script(
            &mut message,
            game,
            game.current_game_server_sender().as_ref(),
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingPlayerRunScript {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_parameter(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionParameter {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_change_region_router(
            &mut message,
            region_router,
            game_server_sender.as_ref(),
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingChangeRegionRouter {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_village_war_application(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            attack_city,
            village_war,
            application_callbacks,
            game_server_sender.as_ref(),
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingVillageWarApplication {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_village_war_result(
            &mut message,
            game,
            organizing,
            village_war,
            timer,
            village_war_callbacks,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingVillageWarResult {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_city_war_application(
            &mut message,
            game,
            globe_setup,
            organizing,
            organizing_parameters,
            attack_city,
            village_war,
            application_callbacks,
            update_player,
            game_server_sender.as_ref(),
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingCityWarApplication {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_city_war_result(
            &mut message,
            game,
            organizing,
            country_handler,
            country_parameters,
            attack_city,
            timer,
            attack_city_callbacks,
            application_callbacks,
            update_player,
            globe_setup,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingCityWarResult {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_application_decision(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            village_war,
            attack_city,
            goods_war,
            game.setup.use_log_system,
            faction_join_log_enabled,
            write_faction_join_log,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionApplicationDecision {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_fire_out(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            village_war,
            attack_city,
            goods_war,
            game.setup.use_log_system,
            faction_fire_out_log_enabled,
            write_faction_fire_out_log,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionFireOut {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_exit(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            village_war,
            attack_city,
            goods_war,
            game.setup.use_log_system,
            faction_quit_log_enabled,
            write_faction_quit_log,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionExit {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_union_exit(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingUnionExit {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_demise(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            &*country_handler,
            &*attack_city,
            &*goods_war,
            game.setup.use_log_system && faction_master_log_enabled,
            write_faction_master_log,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionDemise {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_union_demise(
            &mut message,
            game,
            organizing,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingUnionDemise {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        let faction_disband = {
            let mut effects = WorldOrganizingDisbandEffects {
                game: &*game,
                village_war: &*village_war,
                attack_city: &*attack_city,
                country_handler: &*country_handler,
                goods_war: &mut *goods_war,
                world_string: &mut *application_callbacks.world_string,
            };
            dispatch_faction_disband(&mut message, &*game, organizing, &mut effects)
        };
        if let Some(outcome) = faction_disband {
            let faction_disband_log_enabled =
                game.setup.use_log_system && faction_disband_log_enabled;
            let outcome = outcome.map(|pending| {
                finalize_faction_disband_dispatch(
                    pending,
                    faction_disband_log_enabled,
                    |player_id| game.clear_disbanded_player_faction_data(player_id),
                    |faction_id, faction_name, player_id, player_name| {
                        write_faction_disband_log(
                            faction_id,
                            faction_name,
                            player_id,
                            player_name,
                        );
                    },
                )
            });
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionDisband {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_union_disband(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingUnionDisband {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_dub(
            &mut message,
            game,
            organizing,
            application_callbacks,
            check_invalid_organizing_string,
            game.setup.use_log_system,
            faction_title_log_enabled,
            write_faction_title_log,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionDub {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_purview(
            &mut message,
            game,
            organizing,
            application_callbacks,
            game.setup.use_log_system,
            faction_purview_add_log_enabled,
            faction_purview_revoke_log_enabled,
            write_faction_purview_log,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionPurview {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_union_fire_out(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            application_callbacks,
            update_player,
        ) {
            let callbacks = WorldUnionApplicationEffectCallbacks {
                random: &mut *application_callbacks.random,
                world_string: &mut *application_callbacks.world_string,
                format_world_string: &mut *application_callbacks.format_world_string,
                put_war_log: &mut *application_callbacks.put_war_log,
                refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
                faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
                write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
                faction_experience_log_enabled:
                    application_callbacks.faction_experience_log_enabled,
                write_faction_experience_log:
                    &mut *application_callbacks.write_faction_experience_log,
            };
            let mut effects = WorldUnionApplicationEffects::new(
                game,
                net_sessions,
                application_runtime,
                callbacks,
            );
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingUnionFireOut {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        let callbacks = WorldUnionApplicationEffectCallbacks {
            random: &mut *application_callbacks.random,
            world_string: &mut *application_callbacks.world_string,
            format_world_string: &mut *application_callbacks.format_world_string,
            put_war_log: &mut *application_callbacks.put_war_log,
            refresh_owned_city: &mut *application_callbacks.refresh_owned_city,
            faction_level_log_enabled: application_callbacks.faction_level_log_enabled,
            write_faction_level_log: &mut *application_callbacks.write_faction_level_log,
            faction_experience_log_enabled:
                application_callbacks.faction_experience_log_enabled,
            write_faction_experience_log:
                &mut *application_callbacks.write_faction_experience_log,
        };
        let mut effects = WorldUnionApplicationEffects::new(
            game,
            net_sessions,
            application_runtime,
            callbacks,
        );
        if let Some(outcome) = dispatch_region_route(&mut message, game) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingRegionRoute {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_city_gate(&mut message, game, organizing) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingCityGate {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_city_transfer(
            &mut message,
            game,
            country_handler,
            organizing,
            attack_city,
            village_war,
            &mut effects,
        ) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingCityTransfer {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_admission_permit(&mut message, game, organizing) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingAdmissionPermit {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_attack_city_end(
            &mut message,
            game,
            organizing,
            &mut effects,
            update_player,
        ) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingAttackCityEnd {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_tax(
            &mut message,
            organizing,
            attack_city,
            village_war,
            &mut effects,
            game_server_sender.as_ref(),
        ) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionTax {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_list(
            &mut message,
            organizing,
            &mut effects,
            game_server_sender.as_ref(),
        ) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionList {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_application(
            &mut message,
            game,
            organizing,
            organizing_parameters,
            village_war,
            attack_city,
            game.setup.use_log_system,
            faction_apply_log_enabled,
            write_faction_apply_log,
            &mut effects,
        ) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionApplication {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_faction_application_cancel(
            &mut message,
            game,
            organizing,
            &mut effects,
        ) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingFactionApplicationCancel {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_declare_war_faction_list(
            &mut message,
            organizing,
            faction_war_sys,
            &mut effects,
            game_server_sender.as_ref(),
        ) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingDeclareWarFactionList {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_consumed_long(&mut message) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingConsumedLong {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) = dispatch_player_invite_faction(
            &mut message,
            game,
            organizing,
            village_war,
            attack_city,
            &mut effects,
        ) {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingPlayerInviteFaction {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        match dispatch_organizing_session_result(&mut message, net_sessions) {
            OrganizingSessionResultDispatch::NotHandled => {}
            outcome => {
                let runtime = drain_union_application_runtime(
                    game,
                    organizing,
                    organizing_parameters,
                    application_runtime,
                    &mut effects,
                    update_player,
                );
                return ProcessedWorldEvent::OrganizingSessionResult {
                    source,
                    legacy_run_result,
                    outcome,
                    runtime,
                };
            }
        }
        if let Some(outcome) =
            dispatch_union_application(&mut message, game, organizing, &mut effects)
        {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingUnionApplication {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
        if let Some(outcome) =
            dispatch_leave_word_enable(&mut message, organizing, &mut effects)
        {
            let runtime = drain_union_application_runtime(
                game,
                organizing,
                organizing_parameters,
                application_runtime,
                &mut effects,
                update_player,
            );
            return ProcessedWorldEvent::OrganizingLeaveWordEnable {
                source,
                legacy_run_result,
                outcome,
                runtime,
            };
        }
    }

    ProcessedWorldEvent::Message(RoutedWorldMessage {
        source,
        message_type,
        owner: selector.owner,
        legacy_run_result,
        message,
    })
}

fn drain_union_application_runtime(
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    organizing_parameters: &COrganizingParam,
    runtime: &WorldUnionApplicationRuntimeOwner,
    effects: &mut WorldUnionApplicationEffects<'_>,
    update_player: &mut dyn FnMut(i32),
) -> WorldUnionApplicationRuntimeReport {
    let mut terminals = Vec::new();
    let mut invitation_terminals = Vec::new();
    let mut city_terminals = Vec::new();
    let mut confederation_creation_terminals = Vec::new();
    while let Some(request) = runtime.pop_terminal() {
        match request {
            QueuedOrganizingSessionTerminal::Union(request) => {
                let outcome = organizing.finish_union_application(
                    game,
                    organizing_parameters,
                    request.union_id,
                    request.applicant_faction_id,
                    request.terminal,
                    effects,
                    update_player,
                );
                terminals.push(WorldUnionApplicationTerminalDispatch { request, outcome });
            }
            QueuedOrganizingSessionTerminal::UnionInvitation(request) => {
                let outcome = organizing.finish_union_invitation(
                    game,
                    organizing_parameters,
                    request.union_id,
                    request.inviter_faction_id,
                    request.invited_faction_id,
                    request.terminal,
                    effects,
                    update_player,
                );
                invitation_terminals.push(WorldUnionInvitationTerminalDispatch {
                    request,
                    outcome,
                });
            }
            QueuedOrganizingSessionTerminal::ConfederationCreation(request) => {
                let outcome = organizing.finish_confederation_creation(
                    game,
                    organizing_parameters,
                    request.first_player_id,
                    request.second_player_id,
                    request.first_faction_id,
                    request.second_faction_id,
                    &request.union_name,
                    request.terminal,
                    effects,
                    update_player,
                );
                confederation_creation_terminals.push(
                    WorldConfederationCreationTerminalDispatch { request, outcome },
                );
            }
            QueuedOrganizingSessionTerminal::CityTransfer(request) => {
                let outcome = organizing.finish_city_transfer(
                    game,
                    request.source_faction_id,
                    request.target_faction_id,
                    request.region_id,
                    &request.region_name,
                    request.terminal,
                    effects,
                    update_player,
                );
                city_terminals.push(WorldCityTransferTerminalDispatch { request, outcome });
            }
        }
    }
    WorldUnionApplicationRuntimeReport {
        terminals,
        invitation_terminals,
        confirmations: runtime.take_confirmations(),
        endpoint_blocks: runtime.take_blocks(),
        city_terminals,
        city_confirmations: runtime.take_city_confirmations(),
        city_endpoint_blocks: runtime.take_city_blocks(),
        confederation_creation_terminals,
        confederation_creation_confirmations:
            runtime.take_confederation_creation_confirmations(),
        confederation_creation_endpoint_blocks:
            runtime.take_confederation_creation_blocks(),
    }
}

struct WorldOwnerSelector {
    write_log_enabled: bool,
    owner: Option<WorldMessageOwner>,
}

impl WorldOwnerSelector {
    fn select(&mut self, owner: WorldMessageOwner) {
        self.owner = Some(owner);
    }
}

impl WorldMessageHandlers for WorldOwnerSelector {
    fn write_log_enabled(&self) -> bool {
        self.write_log_enabled
    }

    fn on_server(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::Server);
    }

    fn on_log(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::Log);
    }

    fn on_gma(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::Gma);
    }

    fn on_player(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::Player);
    }

    fn on_other(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::Other);
    }

    fn on_gm(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::Gm);
    }

    fn on_team(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::Team);
    }

    fn on_orgasys(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::OrganizingSystem);
    }

    fn on_write_log(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::WriteLog);
    }

    fn on_country(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::Country);
    }

    fn on_server_auction(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::ServerAuction);
    }

    fn on_jjc_system(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::JjcSystem);
    }

    fn on_misc_auction(&mut self, _message: &mut CMessage) {
        self.select(WorldMessageOwner::MiscAuction);
    }
}

/// Внешняя сериализация исходного `g_CriticalSectionSaveThread`.
///
/// Guard возвращается только на границе, где старый `SaveThreadFunc` не дошёл
/// до `LeaveCriticalSection`. Сам mutex не удерживается через async DB I/O:
/// эксклюзивный borrow всего `CGame` даёт ту же единственность owner-а.
pub(crate) struct WorldSaveThreadGuard<'game> {
    game: &'game mut CGame,
}

impl<'game> WorldSaveThreadGuard<'game> {
    /// Явно завершает заблокированный typed owner после решения его границы.
    pub(crate) fn into_game(self) -> &'game mut CGame {
        self.game
    }

    /// Завершает normal-path сериализацию перед отдельным end-log.
    fn release(self) {}

    /// Заканчивает внешний owner немедленно, не изображая normal unlock-путь.
    fn stop_outer_owner(self) {}
}

/// Доказанный результат тела `SaveThreadFunc` без создания системного потока.
pub(crate) enum WorldSaveThreadReport<'game> {
    /// Start-log остановился после входа в save-сериализацию.
    BlockedStartLog {
        guard: WorldSaveThreadGuard<'game>,
        block: SaveDataLogPublishBlock,
    },
    /// `DoSaveData` не вернулся; COM-uninit/unlock/end-log не назначены.
    BlockedLifecycle {
        guard: WorldSaveThreadGuard<'game>,
        start_log: SaveDataLogPublishDisposition,
        lifecycle: DoSaveDataLifecycleReport,
    },
    /// Lifecycle вернулся и serialization снята, но последний log заблокирован.
    BlockedEndLog {
        start_log: SaveDataLogPublishDisposition,
        lifecycle: DoSaveDataLifecycleReport,
        block: SaveDataLogPublishBlock,
    },
    /// Оба thread-log-а и весь lifecycle завершены; старый exit code равен нулю.
    Complete {
        start_log: SaveDataLogPublishDisposition,
        lifecycle: DoSaveDataLifecycleReport,
        end_log: SaveDataLogPublishDisposition,
        exit_code: u32,
    },
}

/// Единственное наблюдаемое состояние opaque `g_hSavingThread` у caller-ов.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldSaveThreadHandleState {
    Empty,
    Open,
}

/// Одноразовая обязанность достигнутого `__beginthreadex(SaveThreadFunc)`.
///
/// Сам request не угадывает handle: внешний launcher возвращает наблюдаемое
/// `Open/Empty` состояние после фактической попытки запуска.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldSaveThreadLaunchRequest {
    pub(crate) previous_handle_closed: bool,
    pub(crate) security_attributes_is_null: bool,
    pub(crate) stack_size: u32,
    pub(crate) argument_is_null: bool,
    pub(crate) creation_flags: u32,
    pub(crate) thread_id_output_requested: bool,
}

/// Snapshot/cleanup отчёт Run-ветви вместе с достигнутым launch call-site.
#[derive(Debug)]
pub(crate) struct WorldRunSaveLaunchReport {
    pub(crate) snapshot: WorldGenerateDbDataReport,
    pub(crate) launch: WorldSaveThreadLaunchRequest,
    pub(crate) resulting_handle: WorldSaveThreadHandleState,
}

/// Организационный snapshot `g_bSaveAllOrg` и тот же launch call-site.
#[derive(Debug)]
pub(crate) struct WorldSaveAllOrganizationsLaunchReport {
    pub(crate) organizing: OrganizingSaveDataReport,
    pub(crate) launch: WorldSaveThreadLaunchRequest,
    pub(crate) resulting_handle: WorldSaveThreadHandleState,
}

/// Process-global ручной флаг collect-player-data broadcast-а.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorldCollectPlayerDataRequestState {
    pub(crate) send_now: bool,
}

/// Итог пустого `0x7F808`, отправленного через исходный `SendAll`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCollectPlayerDataBroadcast {
    pub(crate) message_type: i32,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

/// Process-global save-флаги и tick, прочитанные достигнутым участком Run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldRunSaveTriggerState {
    pub(crate) send_save_message_now: bool,
    pub(crate) save_all_organizations: bool,
    pub(crate) save_now_data: bool,
    pub(crate) last_save_point_time_ms: u32,
}

/// Manual save-request log и первое достигнутое чтение `dwSavePointTime`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldManualSaveRequestReport {
    pub(crate) log: AddLogTextDisposition,
    pub(crate) save_point_time_ms: u32,
}

/// Результат interval/try-lock pre-gate до уже восстановленного save-решения.
pub(crate) enum WorldRunSavePreGateReport<'game> {
    IntervalNotElapsed {
        manual_request: Option<WorldManualSaveRequestReport>,
        profile_started_at_ms: u32,
        elapsed_ms: u32,
        save_point_time_ms: u32,
    },
    SaveLockBusy {
        manual_request: Option<WorldManualSaveRequestReport>,
        profile_started_at_ms: u32,
        elapsed_ms: u32,
        save_point_time_ms: u32,
        adjusted_last_save_point_time_ms: u32,
    },
    AfterLock {
        manual_request: Option<WorldManualSaveRequestReport>,
        profile_started_at_ms: u32,
        elapsed_ms: u32,
        save_point_time_ms: u32,
        trigger: WorldRunSaveTriggerReport<'game>,
    },
}

/// Локальный player snapshot после точного manual/no-GameServer лога.
#[derive(Debug)]
pub(crate) struct WorldRunImmediateSaveReport {
    pub(crate) log: AddLogTextDisposition,
    pub(crate) save: WorldRunSaveLaunchReport,
}

/// Одна попытка `SaveNotify` для connected записи исходного ordered map.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldSaveNotifyDelivery {
    pub(crate) game_server_index: u32,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

/// Полная рассылка пустого `0x7F803` после сброса `m_nDBResponsed`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldSaveNotifyReport {
    pub(crate) log: AddLogTextDisposition,
    pub(crate) previous_db_responses: i32,
    pub(crate) message_type: i32,
    pub(crate) deliveries: Vec<WorldSaveNotifyDelivery>,
}

/// Успешно завершённое решение save-trigger перед снятием critical section.
#[derive(Debug)]
pub(crate) enum WorldRunSaveTriggerDisposition {
    SaveAllOrganizations(WorldSaveAllOrganizationsLaunchReport),
    PlayerData {
        immediate: Option<WorldRunImmediateSaveReport>,
        notify: Option<WorldSaveNotifyReport>,
    },
}

/// Результат участка Run с точной судьбой внешнего save-guard.
pub(crate) enum WorldRunSaveTriggerReport<'game> {
    BlockedSaveAllOrganizations {
        guard: WorldSaveThreadGuard<'game>,
        block: OrganizingSaveDataBlock,
    },
    BlockedImmediateSave {
        guard: WorldSaveThreadGuard<'game>,
        log: AddLogTextDisposition,
        block: WorldGenerateDbDataBlock,
    },
    Complete(WorldRunSaveTriggerDisposition),
}

/// Закрывает прежний opaque handle-state и материализует точные launch-аргументы.
pub(crate) fn prepare_save_thread_launch(
    handle: &mut WorldSaveThreadHandleState,
) -> WorldSaveThreadLaunchRequest {
    let previous_handle_closed = matches!(*handle, WorldSaveThreadHandleState::Open);
    *handle = WorldSaveThreadHandleState::Empty;
    WorldSaveThreadLaunchRequest {
        previous_handle_closed,
        security_attributes_is_null: true,
        stack_size: 0,
        argument_is_null: true,
        creation_flags: 0,
        thread_id_output_requested: true,
    }
}

fn save_data_lifecycle_completed(report: &DoSaveDataLifecycleReport) -> bool {
    matches!(
        report,
        DoSaveDataLifecycleReport::Final {
            report: crate::worldserver::worldserver::savedb::SaveDataFinalReport {
                disposition: SaveDataFinalDisposition::Complete(_),
                ..
            },
            ..
        }
    )
}

fn save_thread_log_event(payload: &'static [u8]) -> SaveDataLogEvent {
    SaveDataLogEvent {
        target: SaveDataLogTarget::AddLogText,
        payload: payload.to_vec(),
    }
}

/// Выполняет exact body `SaveThreadFunc`, но не создаёт и не завершает thread.
///
/// `CoInitialize/CoUninitialize` не имеют Linux runtime-аналога: достигнутые
/// DB-owner-ы используют Tiberius, поэтому COM apartment был заменяемым
/// техническим механизмом, а не наблюдаемым серверным контрактом.
#[allow(
    clippy::too_many_arguments,
    reason = "SaveThreadFunc передаёт прежние process-global owner-ы явно"
)]
pub(crate) async fn save_thread_func<
    'game,
    S,
    O,
    V,
    P,
    J,
    G,
    U,
    F,
    R,
    B,
    E,
    C,
    L,
    Log,
    GetMonitoring,
    SendMonitoring,
>(
    game: &'game mut CGame,
    settings: &WorldDatabaseSettings,
    state: &mut SaveDataLifecycleState,
    variables: &S,
    registry: &GoodsBasePropertiesRegistry,
    organizing_ctrl: &mut COrganizingCtrl,
    honor_ranks: &mut CHonorRanks,
    gods_battle_faction_xyd: GodsBattleFactionXydSnapshot,
    gods_battle_npc_factions: &[GodsBattleNpcFactionSnapshot],
    use_old_save_largess_way: bool,
    setup_database: &mut O,
    variable_database: &mut V,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    union_database: &mut U,
    faction_database: &mut F,
    region_database: &mut R,
    gods_battle_database: &mut B,
    enemy_factions_database: &mut E,
    country_database: &mut C,
    largess: &mut L,
    log_sink: &mut Log,
    get_monitoring: GetMonitoring,
    send_monitoring: SendMonitoring,
) -> WorldSaveThreadReport<'game>
where
    S: VariableListSaveSource,
    O: RsSetupOwner,
    V: RsGenVarOwner,
    P: RsPlayerOwner,
    J: RsJjcSysOwner,
    G: DbGoodsOwner,
    U: RsUnionOwner,
    F: RsFactionOwner,
    R: RsRegionOwner,
    B: RsGodsBattleOwner,
    E: RsEnemyFactionsOwner,
    C: DbCountryOwner,
    L: LargessOwner,
    Log: SaveDataLogSink,
    GetMonitoring: FnOnce() -> SaveDataMonitoringSnapshot,
    SendMonitoring: FnOnce(&SaveDataMonitoringReport),
{
    let guard = WorldSaveThreadGuard { game };
    let start_log = match log_sink.publish(&save_thread_log_event(b"SaveThread Starting...")) {
        SaveDataLogPublishDisposition::BlockedMissingFact(block) => {
            return WorldSaveThreadReport::BlockedStartLog { guard, block };
        }
        disposition => disposition,
    };

    let lifecycle = {
        let mut session = guard.game.db_data_save_session();
        do_save_data_lifecycle(
            settings,
            state,
            &mut session,
            variables,
            registry,
            organizing_ctrl,
            honor_ranks,
            gods_battle_faction_xyd,
            gods_battle_npc_factions,
            use_old_save_largess_way,
            setup_database,
            variable_database,
            player_database,
            jjc_database,
            goods_database,
            union_database,
            faction_database,
            region_database,
            gods_battle_database,
            enemy_factions_database,
            country_database,
            largess,
            log_sink,
            get_monitoring,
            send_monitoring,
        )
        .await
    };

    if !save_data_lifecycle_completed(&lifecycle) {
        return WorldSaveThreadReport::BlockedLifecycle {
            guard,
            start_log,
            lifecycle,
        };
    }

    // Эта точка одновременно заменяет CoUninitialize и исходный unlock.
    guard.release();
    let end_log = match log_sink.publish(&save_thread_log_event(b"SaveThread end...")) {
        SaveDataLogPublishDisposition::BlockedMissingFact(block) => {
            return WorldSaveThreadReport::BlockedEndLog {
                start_log,
                lifecycle,
                block,
            };
        }
        disposition => disposition,
    };

    WorldSaveThreadReport::Complete {
        start_log,
        lifecycle,
        end_log,
        exit_code: 0,
    }
}

/// Публикует LoginServer-сообщение о reload либо сохраняет точную safe-границу.
pub(crate) fn reload_conf_log<GetLocalTime>(
    game: &CGame,
    profile: Option<&[u8]>,
    _reload_result: i32,
    get_local_time: &mut GetLocalTime,
) -> Result<WorldReloadConfLogDisposition, WorldReloadConfLogBlock>
where
    GetLocalTime: FnMut() -> WorldLogLocalTime,
{
    let Some(profile) = profile else {
        return Ok(WorldReloadConfLogDisposition::SuppressedEmptyProfile);
    };
    let profile = legacy_c_string_prefix(profile);
    if profile.is_empty() {
        return Ok(WorldReloadConfLogDisposition::SuppressedEmptyProfile);
    }

    let date = get_local_time();
    let time = get_local_time();
    let date_and_time = format!(
        "{:02}/{:02}/{:02} {:02}:{:02}:{:02}",
        date.month,
        date.day,
        date.year % 100,
        time.hour,
        time.minute,
        time.second,
    );
    let mut text = Vec::with_capacity(32 + profile.len());
    text.extend_from_slice(b"WS On ");
    text.extend_from_slice(date_and_time.as_bytes());
    text.extend_from_slice(b" Reload ");
    text.extend_from_slice(profile);
    text.push(b'.');

    let required_bytes_with_nul = text.len() + 1;
    if required_bytes_with_nul > 128 {
        // BLOCKED_MISSING_FACT: RVA 0x00002FD0 писал `_sprintf` в `char[128]`.
        // Реакция переполнения stack-buffer неизвестна и не имитируется.
        return Err(WorldReloadConfLogBlock::MessageOutsideLegacyStackBuffer {
            required_bytes_with_nul,
        });
    }

    let net_server = game
        .net_server
        .as_ref()
        .ok_or(WorldReloadConfLogBlock::MissingNetworkServerOwner)?;
    let world_number = game
        .setup
        .world_number
        .ok_or(WorldReloadConfLogBlock::MissingWorldNumber)?;

    let mut message = CMessage::new(0x0001_FE06);
    message.base_mut().add_ulong(net_server.local_ipv4_word());
    message.base_mut().add_ulong(world_number);
    add_legacy_c_string(message.base_mut(), &text);
    let delivery = message.send(
        game.net_client.as_ref().map(CMyNetClient::send_queue),
        false,
    );

    Ok(WorldReloadConfLogDisposition::Published { text, delivery })
}

/// Снимает и обрабатывает полный ordered набор `RELOAD_PROFILE_FLAGS`.
pub(crate) fn reload_profiles<Context, GetLocalTime, GetTimerLocalTime, TimerCallback>(
    game: &mut CGame,
    flags: &WorldReloadProfileFlags,
    context: &mut Context,
    jjc: &mut CJJcSystem,
    mut get_local_time: GetLocalTime,
    country_war: &mut CountryWarSys,
    timer: &mut CTimer<TimerCallback>,
    country_war_callbacks: CountryWarCallbacks<TimerCallback>,
    mut get_timer_local_time: GetTimerLocalTime,
) -> WorldReloadProfilesReport
where
    Context: WorldReloadContext + ?Sized,
    GetLocalTime: FnMut() -> WorldLogLocalTime,
    GetTimerLocalTime: FnMut() -> TagTime,
    TimerCallback: Copy,
{
    let mut events = Vec::new();
    if !flags.has_pending() {
        return WorldReloadProfilesReport::Complete {
            events,
            remaining_flags: flags.snapshot(),
        };
    }

    for &action in WORLD_RELOAD_ACTIONS {
        if !flags.contains(action.half, action.mask) {
            continue;
        }

        flags.consume(action);
        let flags_after_clear = flags.snapshot();
        let reload_result = match action.kind {
            WorldReloadActionKind::Reload => match if action.reload_profile == b"CountryWar" {
                game.reload_country_war(
                    context,
                    country_war,
                    timer,
                    country_war_callbacks,
                    get_timer_local_time(),
                    action.second_option,
                )
            } else {
                game.reload(
                    context,
                    jjc,
                    action.reload_profile,
                    action.first_option,
                    action.second_option,
                )
            } {
                Ok(result) => result,
                Err(block) => {
                    return WorldReloadProfilesReport::BlockedReloadOwner {
                        completed_events: events,
                        half: action.half,
                        mask: action.mask,
                        reload_profile: action.reload_profile,
                        log_profile: action.log_profile,
                        flags_after_clear,
                        block,
                    };
                }
            },
            WorldReloadActionKind::ReloadAllRegions => {
                match game.reload_all_region_setup(context) {
                    Ok(true) => 1,
                    Ok(false) => 0,
                    Err(block) => {
                        return WorldReloadProfilesReport::BlockedRegionSetup {
                            completed_events: events,
                            half: action.half,
                            mask: action.mask,
                            flags_after_clear,
                            block,
                        };
                    }
                }
            }
        };
        let log = match reload_conf_log(
            game,
            Some(action.log_profile),
            reload_result,
            &mut get_local_time,
        ) {
            Ok(log) => log,
            Err(block) => {
                return WorldReloadProfilesReport::BlockedMissingFact {
                    completed_events: events,
                    half: action.half,
                    mask: action.mask,
                    reload_profile: action.reload_profile,
                    log_profile: action.log_profile,
                    flags_after_clear,
                    reload_result,
                    block,
                };
            }
        };
        events.push(WorldReloadProfileEvent {
            half: action.half,
            mask: action.mask,
            reload_profile: action.reload_profile,
            log_profile: action.log_profile,
            flags_after_clear,
            reload_result,
            log,
        });
    }

    WorldReloadProfilesReport::Complete {
        events,
        remaining_flags: flags.snapshot(),
    }
}

fn legacy_refresh_count(
    field: &'static str,
    count: usize,
) -> Result<u32, WorldRefreshSnapshotBlock> {
    u32::try_from(count).map_err(|_| WorldRefreshSnapshotBlock { field, count })
}

fn resolve_first_local_ipv4() -> Option<Ipv4Addr> {
    let hostname = uname();
    let hostname = hostname.nodename().to_str().ok()?;
    (hostname, 0)
        .to_socket_addrs()
        .ok()?
        .find_map(|address| match address {
            SocketAddr::V4(address) => Some(*address.ip()),
            SocketAddr::V6(_) => None,
        })
}

pub(crate) fn legacy_tick_ms() -> u32 {
    let now = clock_gettime(ClockId::Boottime);
    let seconds_ms = (now.tv_sec as u64).wrapping_mul(1_000);
    let nanoseconds_ms = (now.tv_nsec as u64) / 1_000_000;
    seconds_ms.wrapping_add(nanoseconds_ms) as u32
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LoginEndpointError {
    TooLongReactionUnknown { length: usize },
    EncodingUnsupported,
    Resolution,
}

impl From<LoginEndpointError> for WorldClientInitializationError {
    fn from(error: LoginEndpointError) -> Self {
        match error {
            LoginEndpointError::TooLongReactionUnknown { length } => {
                Self::LoginAddressTooLongReactionUnknown { length }
            }
            LoginEndpointError::EncodingUnsupported => Self::LoginAddressEncodingUnsupported,
            LoginEndpointError::Resolution => Self::LoginAddressResolution,
        }
    }
}

impl From<LoginEndpointError> for WorldLoginReconnectError {
    fn from(error: LoginEndpointError) -> Self {
        match error {
            LoginEndpointError::TooLongReactionUnknown { length } => {
                Self::LoginAddressTooLongReactionUnknown { length }
            }
            LoginEndpointError::EncodingUnsupported => Self::LoginAddressEncodingUnsupported,
            LoginEndpointError::Resolution => Self::LoginAddressResolution,
        }
    }
}

fn resolve_login_endpoint(raw_host: &[u8], port: u32) -> Result<SocketAddrV4, LoginEndpointError> {
    let host = legacy_c_string_prefix(raw_host);
    if host.len() > 63 {
        // BLOCKED_MISSING_FACT: World CClient::Connect RVA 0x000293E0 копировал
        // строку без проверки в `char local_44[64]`; stack overflow не имитируем.
        return Err(LoginEndpointError::TooLongReactionUnknown { length: host.len() });
    }
    let host = std::str::from_utf8(host).map_err(|_| LoginEndpointError::EncodingUnsupported)?;
    (host, port as u16)
        .to_socket_addrs()
        .ok()
        .and_then(|mut addresses| {
            addresses.find_map(|address| match address {
                SocketAddr::V4(address) => Some(address),
                SocketAddr::V6(_) => None,
            })
        })
        .ok_or(LoginEndpointError::Resolution)
}

fn add_legacy_c_string(message: &mut crate::nets::basemessage::CBaseMessage, value: &[u8]) {
    message.add(legacy_c_string_prefix(value));
    message.add_byte(0);
}

fn normalize_script_path(path: &[u8]) -> Vec<u8> {
    let path = legacy_c_string_prefix(path);
    let path = if path.first() == Some(&b'\\') {
        &path[1..]
    } else {
        path
    };
    path.iter()
        .map(|byte| if *byte == b'\\' { b'/' } else { *byte })
        .collect()
}

/// Неизвестный результат первого `_vsprintf(char[256], ...)` в `ShowSaveInfo`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ShowSaveInfoBufferBlock {
    pub(crate) required_bytes_with_nul: usize,
}

/// Достигнутый результат условной публикации `ShowSaveInfo`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ShowSaveInfoDisposition {
    Suppressed,
    Logged(AddLogTextDisposition),
    BlockedMissingFact(ShowSaveInfoBufferBlock),
}

/// Выполняет exact gate и два последовательных formatting-владельца.
///
/// `formatted_message` — результат первого variadic call-site форматирования
/// без конечного NUL. При выключенном `show_save_info` значение сознательно не
/// читается, как и в исходном теле.
pub(crate) fn show_save_info<GetTick, GetLocalTime, PutLogInfo>(
    show_save_info: bool,
    formatted_message: &[u8],
    save_info_time_ms: u32,
    log: &mut WorldLogTextOwner,
    get_tick: GetTick,
    get_local_time: GetLocalTime,
    put_log_info: PutLogInfo,
) -> ShowSaveInfoDisposition
where
    GetTick: FnMut() -> u32,
    GetLocalTime: FnMut() -> WorldLogLocalTime,
    PutLogInfo: FnMut(&[u8]),
{
    if !show_save_info {
        return ShowSaveInfoDisposition::Suppressed;
    }

    const LEGACY_SHOW_SAVE_INFO_CAPACITY: usize = 256;
    let formatted_message = legacy_c_string_prefix(formatted_message);
    let required_bytes_with_nul = formatted_message.len() + 1;
    if required_bytes_with_nul > LEGACY_SHOW_SAVE_INFO_CAPACITY {
        return ShowSaveInfoDisposition::BlockedMissingFact(ShowSaveInfoBufferBlock {
            required_bytes_with_nul,
        });
    }

    ShowSaveInfoDisposition::Logged(log.add_log_text_no_arguments(
        formatted_message,
        save_info_time_ms,
        get_tick,
        get_local_time,
        put_log_info,
    ))
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}

fn copy_name_for_legacy_lowercase(value: &[u8]) -> Result<Vec<u8>, usize> {
    const LEGACY_BUFFER_CAPACITY: usize = 0x104;

    let value = legacy_c_string_prefix(value);
    if value.len() >= LEGACY_BUFFER_CAPACITY {
        // BLOCKED_MISSING_FACT: exact `_snprintf(char[260], 260, "%s", ...)`
        // по 0x0051B932..0x0051B945 не дописывал NUL при исчерпанном count,
        // после чего `ToStrlwr` читал за стеком. Не назначаем неизвестному UB
        // ни совпадение, ни отказ.
        return Err(value.len());
    }
    let mut copy = value.to_vec();
    CGame::to_strlwr(&mut copy);
    Ok(copy)
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.h

// IMPLEMENTED: `ShowSaveInfo` RVA `0x00001720` находится выше.

// ============================================================================
// IMPLEMENTED: `DeleteGame` RVA `0x00001780` находится выше; Rust `Drop` заменяет deleting destructor и обнуляет owned slot.

// ============================================================================
// IMPLEMENTED: `GetGame` RVA `0x000017A0` находится выше; nullable global pointer заменён заимствованием из owned slot.

// IMPLEMENTED: `CGame::SendGlobeVariableToGS` RVA `0x000017E0` находится
// выше. Точный конструктор и полный проход ссылок доказали отсутствие producer-а
// четырёх исходно неинициализированных `long`; безопасная замена нулями описана
// owner-комментарием.

// IMPLEMENTED: `CGame::SendMsg2GameServer` RVA `0x00001B10` находится выше.

// IMPLEMENTED: `CGame::CheckInvalidString` RVA `0x00001B30` находится выше;
// process-global singleton заменён owned `CWordsFilter` без изменения dispatch.

// ============================================================================
// FUNCTION: CGame::GetPlayerEquipID
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4602
// RVA: 0x00001B50
// ADDRESS: 00401b50
// PROTOTYPE: void __thiscall GetPlayerEquipID(CPlayer * param_1, ulong * param_2, ulong * param_3, ulong * param_4, ulong * param_5, ulong * param_6, ulong * param_7, ulong * param_8, ulong * param_9, ulong * param_10, ulong * param_11, ulong * param_12, uchar * param_13, uchar * param_14, uchar * param_15, uchar * param_16, uchar * param_17, uchar * param_18, uchar * param_19, uchar * param_20, uchar * param_21, uchar * param_22, uchar * param_23)
//
// IMPLEMENTED_OWNER: `CGame::get_player_equip_id` делегирует достигнутому
// `CPlayer::equipment_wire_snapshot`; один typed snapshot заменяет 22 output-
// ссылки, не меняя порядок slots и byte-cast уровней.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED выше: SaveThreadFunc, WorldServer RVA 0x00001E30.

// ============================================================================
// FUNCTION: CGame::CodeStringTable
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5345
// RVA: 0x00001F00
// ADDRESS: 00401f00
// PROTOTYPE: void __thiscall CodeStringTable(void)
//
// IMPLEMENTED_OWNER: `CGame::code_string_table`; exact append делегирован
// достигнутому `MyStringTable::to_byte_array`, без не-Miracle vector plumbing.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::GetStringTableByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5350
// RVA: 0x00001F20
// ADDRESS: 00401f20
// PROTOTYPE: vector<unsigned_char,std::allocator<unsigned_char>_> * __thiscall GetStringTableByteArray(void)
//
// читает тот же owned buffer напрямую, без внешней дублирующей ссылки.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: SendErrLog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5399
// RVA: 0x00001F30
// ADDRESS: 00401f30
// PROTOTYPE: void __cdecl SendErrLog(char param_1, long param_2, long param_3, char * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CGame::ToStrlwr` RVA `0x00002000` находится выше; exact
// switch-table и обе доказанные странности сохранены byte-for-byte.

// ============================================================================
// FUNCTION: CGame::GetOptMoneyJin
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5775
// RVA: 0x00002250
// ADDRESS: 00402250
// PROTOTYPE: bool __thiscall GetOptMoneyJin(CGoodsNode * param_1, long * param_2, long * param_3)
//
// IMPLEMENTED_OWNER: `CGame::get_opt_money_jin`; `Option<u32>` заменяет
// nullable goods-owner, а `WorldAuctionSellerMoney` — две output-ссылки.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GetStringByID
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.h:944
// RVA: 0x00002FB0
// ADDRESS: 00402fb0
// PROTOTYPE: char * __cdecl GetStringByID(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// IMPLEMENTED_OWNER: `CGame::get_string_by_id`; явный game-owner заменяет
// nullable process-global `g_pGame`, а miss по-прежнему даёт пустые bytes.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `reload_conf_log` RVA `0x00002FD0` находится выше.

// ============================================================================
// FUNCTION: ConnectLoginServerFunc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4883
// RVA: 0x000033B0
// ADDRESS: 004033b0
// PROTOTYPE: uint __stdcall ConnectLoginServerFunc(void * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::LoadStringTable
// STATUS: IMPLEMENTED / INFRASTRUCTURE_SPLIT
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5324
// RVA: 0x000033F0
// ADDRESS: 004033f0
// PROTOTYPE: bool __thiscall LoadStringTable(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// выполняет process context, parser/error и обе exact log-ветви принадлежат CGame.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::CreateConnectLoginThread
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:2246
// RVA: 0x00004320
// ADDRESS: 00404320
// PROTOTYPE: void __thiscall CreateConnectLoginThread(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CGame::ClearCreationPlayer` RVA `0x00004B30` находится выше;
// весь подтверждённый PDB `std::list<long>` очищается одним `VecDeque::clear`.

// IMPLEMENTED: `CGame::ClearRestorePlayer` RVA `0x00004B70` и
// `CGame::ClearDeletionPlayer` RVA `0x00004BB0` находятся выше; `VecDeque`
// заменяет только полный STL list-node traversal и освобождение.

// IMPLEMENTED: `CGame::ClearOfflinePlayer` RVA `0x00004BF0` находится выше;
// весь `std::list<unsigned int>` очищается одним `VecDeque::clear`.

// CGame::tagSetup::tagSetup RVA 0x00004C30: IMPLEMENTED выше.
// Создание/освобождение std::string заменено обычным владением Vec<u8>.
//
// IMPLEMENTED: `CGame::IsNameExistInMapPlayer` RVA `0x00005190` находится
// выше; custom lowercase, map-order и safe char[260] граница сохранены.

// IMPLEMENTED: `CGame::GetCreationPlayerCountInCdkey` RVA `0x000052D0`
// находится выше; map/list order, `_strcmpi` и `u8` wrapping сохранены.

// IMPLEMENTED: `CGame::GetCreationPlayerByName` RVA `0x00005390` находится
// выше; map/creation-list contract и safe char[260] граница сохранены.

// IMPLEMENTED: `CGame::DeleteRestorePlayer` RVA `0x000054E0` и
// `CGame::IsRestorePlayerExist` RVA `0x00005530` находятся выше; удаляется
// только первый list-node, а проверка сохраняет линейный поиск.

// IMPLEMENTED: `CGame::GetDeletionPlayerTime` RVA `0x00005560` находится
// выше; первый совпавший record возвращает signed time, отсутствие — `0`.

// IMPLEMENTED: `CGame::GetOnlinePlayerIDByName` RVA `0x00005590` находится
// выше; map/list traversal и CRT ASCII-case-insensitive сравнение сохранены.

// ============================================================================
// FUNCTION: CGame::GetMapPlayerIDByName
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:3408
// RVA: 0x00005640
// ADDRESS: 00405640
// PROTOTYPE: ulong __thiscall GetMapPlayerIDByName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::GetOnlinePlayerByCdkey
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:3477
// RVA: 0x000056D0
// ADDRESS: 004056d0
// PROTOTYPE: CPlayer * __thiscall GetOnlinePlayerByCdkey(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::IsNameExistInDBCreation
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4101
// RVA: 0x00005790
// ADDRESS: 00405790
// PROTOTYPE: bool __thiscall IsNameExistInDBCreation(char * param_1)
//
// IMPLEMENTED_OWNER: `CGame::is_name_exist_in_db_creation` выше сохраняет
// list-order и legacy lowercase поверх Rust mutex.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::IsNameExistInDBData
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4131
// RVA: 0x000058D0
// ADDRESS: 004058d0
// PROTOTYPE: bool __thiscall IsNameExistInDBData(char * param_1)
//
// IMPLEMENTED_OWNER: `CGame::is_name_exist_in_db_data` выше сохраняет
// unsigned map-order и legacy lowercase поверх Rust mutex.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::FindGoodsLink
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4181
// RVA: 0x00005A10
// ADDRESS: 00405a10
// PROTOTYPE: tagGoodsLink * __thiscall FindGoodsLink(ulong param_1)
//
// `0x00405A10..0x00405A33` выполняет первый linear list-order match по
// `dwIndex +8`. Constructor-ные 500 нулевых записей сохранены явно.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::AddOrginGoodsToPlayer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4755
// RVA: 0x00005A40
// ADDRESS: 00405a40
// PROTOTYPE: void __thiscall AddOrginGoodsToPlayer(CPlayer * param_1)
//
// IMPLEMENTED_OWNER: `CGame::add_origin_goods_to_player` и
// `CPlayer::add_origin_equipment` выше сохраняют exact list/factory/GUID/add.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CGame::GetTeamSessionID` RVA `0x000070C0` находится выше;
// exact PDB map `unsigned long -> long` сохранён как `BTreeMap<u32, i32>`.

// IMPLEMENTED: `CGame::GetMapPlayer` RVA `0x00007100` находится выше;
// входной unsigned ID подтверждён как точный map-key по `0x00407105`.

// IMPLEMENTED: `CGame::DeleteDeletionPlayer` RVA `0x00007140` находится
// выше; удаляется только первый record совпавшего unsigned player ID.

// IMPLEMENTED: `CGame::GetLoginPlayerIDByName` RVA `0x00007270` находится
// выше; login-list order, map lookup и case-sensitive C-string сохранены.

// IMPLEMENTED: `CGame::RemoveOfflinePlayer` RVA `0x00007330` находится выше;
// `VecDeque::retain` сохраняет удаление всех совпадений исходного `list::remove`.

// IMPLEMENTED: `CGame::RemoveLoginPlayer` RVA `0x00007340` находится выше;
// удаляется только первый list-узел совпавшего ID.

// IMPLEMENTED: `CGame::GetLoginPlayerByID` RVA `0x00007390` находится выше;
// list-record взят из PDB, а входной map-key подтверждён по exact EXE.

// ============================================================================
// FUNCTION: CGame::ValidateDBPlayerIDinCdkey
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:3894
// RVA: 0x000073F0
// ADDRESS: 004073f0
// PROTOTYPE: bool __thiscall ValidateDBPlayerIDinCdkey(char * param_1, uint param_2)
//
// Реализация находится в `CGame::validate_db_player_id_in_cdkey` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::ValidatePlayerIDinCdkey
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5078
// RVA: 0x00007490
// ADDRESS: 00407490
// PROTOTYPE: bool __thiscall ValidatePlayerIDinCdkey(char * param_1, uint param_2)
//
// Реализация находится в `CGame::validate_player_id_in_cdkey` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: name-overload `CGame::GetRegion` RVA `0x00008680` находится
// выше как `named_region_lookup`; exact traversal/strcmp подтверждены, а raw
// MSVC tree plumbing удалён.

// ============================================================================
// IMPLEMENTED: `CGame::SaveCityRegion` RVA `0x00008750` находится выше; signed map-order и type `2` сохранены, две UB-границы локализованы.

// ============================================================================
// FUNCTION: CGame::ClearStringTable
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5339
// RVA: 0x000087D0
// ADDRESS: 004087d0
// PROTOTYPE: void __thiscall ClearStringTable(void)
//
// IMPLEMENTED_OWNER: `CGame::clear_string_table`; Rust `clear` заменяет map,
// vector delete и три сырых iterator pointer assignment-а.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::UpdateStringTable
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5355
// RVA: 0x00008820
// ADDRESS: 00408820
// PROTOTYPE: bool __thiscall UpdateStringTable(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// IMPLEMENTED_OWNER: `CGame::update_string_table`; аргумент намеренно
// игнорируется, перечитываются default/configured packages, затем exact
// `0x7F807` raw broadcast и обе исходные log-ветви.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::IsNameExitInFaction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5763
// RVA: 0x000089A0
// ADDRESS: 004089a0
// PROTOTYPE: bool __thiscall IsNameExitInFaction(char * param_1)
//
// IMPLEMENTED_OWNER: `CGame::is_name_exit_in_faction` выше делегирует exact
// `FindOrgaByName` owner-у; typed overflow/null block заменяет потерянный raw
// return, который декомпилятор ошибочно принял за security-cookie результат.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::LoadServerResource
// STATUS: UNKNOWN (сохранены только метаданные исследования) / VERIFIED_DISASSEMBLY_BOUNDARY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:567
// RVA: 0x00009110
// ADDRESS: 00409110
// PROTOTYPE: bool __thiscall LoadServerResource(void)
//
// VERIFIED_NOTE: сохранённый raw ниже ошибочно обрывается после освобождения
// cwd-buffer. Exact тело продолжается до `0x004092BB`: удаляет прежний global
// `CClientResource`, создаёт новый с `GAME_RES=2`, cwd и `FilesInfo.ril`, вызывает
// `LoadEx`, игнорирует его bool, пишет `Load package file OK!` и возвращает
// `true`. Owner остаётся RAW до реконструкции `CClientResource/rfOpen`; текущая
// `WorldGameInitContext::load_server_resources` честно удерживает эту границу.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: LoadPlayerDataFromDB
// STATUS: VERIFIED_DISASSEMBLY, IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5119
// RVA: 0x000092C0
// ADDRESS: 004092c0
// PROTOTYPE: uint __stdcall LoadPlayerDataFromDB(void * param_1)
//
// Реализация находится в `run_player_load_worker` и
// `process_player_load_batch` выше; `WorldPlayerLoadDataAdapter` вызывает
// полный `CPlayer::LoadData`, а его exact bool-смысл передаёт worker-у.
// `WorldPlayerLoadWorkerPool` в `playerloadworker.rs` владеет системными
// потоками, двумя exit-флагами и ordered join без process-global singleton-а.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: DoSaveLog
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5230
// RVA: 0x000095E0
// ADDRESS: 004095e0
// PROTOTYPE: void __cdecl DoSaveLog(void)
//
// IMPLEMENTED_OWNER: `WorldWriteLogWorker` в `writelogworker.rs` соединяет
// exact GetSize/Pop/Execute batch с 1-ms polling, drain-on-exit и 10-sec
// reconnect; отмена reconnect на shutdown исправляет внутреннее зависание.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040985c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5255
// RVA: 0x0000985C
// ADDRESS: 0040985c
// PROTOTYPE: undefined Catch@0040985c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00409899
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5313
// RVA: 0x00009899
// ADDRESS: 00409899
// PROTOTYPE: undefined Catch@00409899()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00409a34
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5294
// RVA: 0x00009A34
// ADDRESS: 00409a34
// PROTOTYPE: undefined Catch@00409a34()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::DeleteMapPlayer
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:2962
// RVA: 0x0000D350
// ADDRESS: 0040d350
// PROTOTYPE: void __thiscall DeleteMapPlayer(uint param_1)
//
// IMPLEMENTED_OWNER: `CGame::delete_map_player`; `Box` уничтожается перед
// удалением map-entry, а miss остаётся no-op.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED выше: CGame::ClearMapPlayerForOffline, WorldServer RVA
// 0x0000D3A0. Полный заменённый raw и MSVC tree traversal удалены.

// ============================================================================
// IMPLEMENTED: `CGame::ClearMapPlayer` RVA `0x0000D450` находится выше; `BTreeMap::pop_first` сохраняет unsigned key-order и owned deletion.

// ============================================================================
// FUNCTION: CGame::ClearDBData
// STATUS: IMPLEMENTED + VERIFIED_DISASSEMBLY
// RVA: 0x0000D490; exact continuous body `0x0040D490..0x0040D76B`.
// Реализация и компактная карта происхождения находятся выше.

// ============================================================================
// FUNCTION: ProcessWriteLogDataFunc
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5088
// RVA: 0x0000D770
// ADDRESS: 0040d770
// PROTOTYPE: uint __stdcall ProcessWriteLogDataFunc(void * param_1)
//
// IMPLEMENTED_OWNER: `WorldWriteLogWorker::start` создаёт owned системный поток,
// ранний `bUseLogSys=false` возвращает `Disabled`, Tokio Handle заменяет COM
// apartment для TDS, `request_exit`/`join` заменяют глобальный флаг и CRT handle.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CGame::DelItemToIpList` RVA `0x0000D830` находится выше;
// ошибочный raw template-type `CPlayer*` заменён доказанным signed refcount.

// ============================================================================
// IMPLEMENTED + VERIFIED_DISASSEMBLY: полный `CGame::Release` RVA `0x0000E7F0` находится выше; exact body имеет единственный `ret` в `0x0040ED87`.

// ============================================================================
// FUNCTION: CGame::LoadSetup
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:587
// RVA: 0x0000F890
//
// Positional open/decode/extraction и различающиеся plain/encoded title готовы
// выше. Callback заменяет `FindWindowA/SetWindowTextA`; занятый title возвращает
// false в `CGame::Init`, где сохраняются точные `ERROR` и message bytes.
//
// ============================================================================
// FUNCTION: FindScriptFile
// STATUS: IMPLEMENTED / SAFE_INFRASTRUCTURE_REPLACEMENT
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:1081
// RVA: 0x000106E0
// ADDRESS: 004106e0
// PROTOTYPE: void __cdecl FindScriptFile(char * param_1, list<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>,std::allocator<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>_>_> * param_2)
//
// IMPLEMENTED_OWNER: `find_script_files`; `walkdir` заменяет Win32 handles и
// recursion, а default `WorldReloadContext::script_files` подключает host-
// fallback, не закрывая будущий package-resource override.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// IMPLEMENTED: `CGame::ReLoadAllRegionSetup` RVA `0x000108F0` находится выше; exact EXE подтверждает единый успешный return.

// ============================================================================
// FUNCTION: CGame::AppendMapPlayer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:2943
// RVA: 0x00010A40
// ADDRESS: 00410a40
// PROTOTYPE: void __thiscall AppendMapPlayer(CPlayer * param_1)
//
// Реализация находится в `CGame::append_map_player`. `Box<CPlayer>` сохраняет
// владение incoming pointer на collision-ветви; replaced STL tree-body удалён.

// ============================================================================
// FUNCTION: CGame::CloneMapPlayer
// STATUS: VERIFIED_DISASSEMBLY, IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:3058
// RVA: 0x00010AB0
// ADDRESS: 00410ab0
// PROTOTYPE: CPlayer * __thiscall CloneMapPlayer(uint param_1)
//
// Реализация находится в `CGame::clone_map_player`. Exact диапазон
// `0x00410AB0..0x00410B98` подтвердил входной `player_id` как map-key,
// `vector.data()`, нулевой cursor, оба `include_child=true` и уничтожение
// decoder-копии при `false`.

// IMPLEMENTED: `CGame::AppendCreationPlayer` RVA `0x00010BA0` находится выше.
// Duplicate уничтожает incoming; existing-owner сохраняет list-мутацию и
// возвращает непринятый `Box` caller-у. Его продолжение принадлежит
// `OnLogMessage`, поэтому append-owner не имитирует ни UAF, ни последующую leak.

// ============================================================================
// FUNCTION: CGame::CloneCreationPlayer
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:3200
// RVA: 0x00010C70
// ADDRESS: 00410c70
// PROTOTYPE: CPlayer * __thiscall CloneCreationPlayer(uint param_1)
//
// list сравнивается по тем же 32 битам с unsigned ID, затем используется общий
// достигнутый `clone_map_player`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CGame::AppendRestorePlayer` RVA `0x00010CA0` и
// `CGame::AppendDeletionPlayer` RVA `0x00010CF0` находятся выше. Оба оставляют
// первый duplicate неизменным и дописывают только новый ID в хвост.

// IMPLEMENTED: `CGame::AppendOfflinePlayer` RVA `0x00010DF0` находится выше;
// exact PDB задаёт `m_lID` как signed long, list хранит тот же шаблон как u32.

// IMPLEMENTED: `CGame::AppendLoginPlayer` RVA `0x00010E50` находится выше;
// raw-имя `tagDeletionPlayer` было ошибкой типов, точный PDB задаёт login-пару.

// ============================================================================
// FUNCTION: CGame::CloneSavingPlayer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:3870
// RVA: 0x00010EB0
// ADDRESS: 00410eb0
// PROTOTYPE: CPlayer * __thiscall CloneSavingPlayer(uint param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED выше: `CGame::AppendDBCreationPlayer` RVA `0x00010FB0`;
// duplicate continuation имеет статус `VERIFIED_DISASSEMBLY`.

// IMPLEMENTED: `CGame::AppendDBCountry` RVA `0x00011070` находится выше.
// Rust принимает уже материализованный non-null country snapshot.

// ============================================================================
// FUNCTION: CGame::AppendSaveFaction
// STATUS: IMPLEMENTED
// RVA: 0x000110D0; реализация находится выше.

// ============================================================================
// FUNCTION: CGame::AppendSaveUnion
// STATUS: IMPLEMENTED
// RVA: 0x00011130; реализация находится выше.

// ============================================================================
// FUNCTION: CGame::AppendDelFaction
// STATUS: IMPLEMENTED
// RVA: 0x00011190; реализация находится выше.

// ============================================================================
// FUNCTION: CGame::AppendDelUnion
// STATUS: IMPLEMENTED
// RVA: 0x000111F0; реализация находится выше.

// ============================================================================
// FUNCTION: CGame::AppendRegionParam
// STATUS: IMPLEMENTED
// RVA: 0x00011250; Rust принимает уже материализованный non-null DB snapshot.

// ============================================================================
// FUNCTION: CGame::AddGoodsLink
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4164
// RVA: 0x000112B0
// ADDRESS: 004112b0
// PROTOTYPE: void __thiscall AddGoodsLink(tagGoodsLink * param_1)
//
// `0x004112B0..0x00411336` удаляет голову только при MSVC list max-size
// `0x0CCCCCCC`, назначает unchanged-записи process-global wrapping index с
// initial `1` и копирует POD в хвост. `VecDeque` заменяет только STL plumbing;
// Rust освобождает owned `CGoods` при крайне редком удалении вместо утечки.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::OnGameServerLost
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4313
// RVA: 0x00011340
// ADDRESS: 00411340
// PROTOTYPE: void __thiscall OnGameServerLost(ulong param_1)
//
// IMPLEMENTED_OWNER: `CGame::on_game_server_lost`; typed report сохраняет
// affected region/player order, каждый list-transition и Login delivery.
// Exact `0x0041165D..0x004116D2` подтвердил, что raw `return` после удаления
// login-node был ошибкой decompiler-а: цикл продолжает offline append и следующий
// affected player. Null region-owner исправлен только как внутренний UB.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// IMPLEMENTED: `CGame::ReLoadOneRegionSetup` RVA `0x000120F0` находится выше; exact EXE подтверждает отдельные true/false epilogue.

// ============================================================================
// IMPLEMENTED: `CGame::LoadRegionList` RVA `0x00012220` находится выше; поставочные subtype/load/serialize вызываются напрямую, COUNTRY type `3` остаётся raw-границей.

// ============================================================================
// FUNCTION: CGame::GetCreationPlayerVectorByCdkey
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:3129
// RVA: 0x00012760
// ADDRESS: 00412760
// PROTOTYPE: void __thiscall GetCreationPlayerVectorByCdkey(char * param_1, vector<long,std::allocator<long>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: CGame::GeterateRegionDBData, WorldServer RVA 0x00012860.
// Полный заменённый псевдокод и inlined MSVC tree traversal удалены; контракт
// и provenance сохранены в верхнем `//!`.

// IMPLEMENTED: `CGame::RefreshOwnedCityOrg` RVA `0x000128E0` находится выше;
// exact mutation/wire-порядок подтверждён дизассемблировкой, а Linux-добавления
// сохранения региона и отдельного codec-а намеренно не перенесены.

// IMPLEMENTED: `CGame::DelItemFromBaiTanList` RVA `0x000129F0` находится
// выше; оба erase выполняются даже при отсутствии player->IP записи.

// IMPLEMENTED выше: CGame::GenerateDBData, WorldServer RVA 0x00012E50.
// Полный заменённый raw и STL traversal удалены.

// ============================================================================
// IMPLEMENTED: `CGame::GetScriptFileData` RVA `0x000132E0` находится выше.

// ============================================================================
// IMPLEMENTED: `CGame::LoadOneScript` RVA `0x00013440` находится выше.

// ============================================================================
// IMPLEMENTED: `CGame::ReLoadOneScript` RVA `0x00013760` находится выше.

// ============================================================================
// FUNCTION: CGame::ProcessPlayerDataQueue
// STATUS: VERIFIED_DISASSEMBLY, IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4910
// RVA: 0x00013BD0
// ADDRESS: 00413bd0
// PROTOTYPE: void __thiscall ProcessPlayerDataQueue(void)
//
// Полная реализация и локальная спецификация находятся в
// `CGame::process_player_data_queue` выше. Exact virtual slot `+0x84`
// подтверждён как `CShape::SetState(0)`; заменённое C++/STL/SEH тело удалено.

// VERIFIED_DISASSEMBLY, IMPLEMENTED: `CGame::AddItemToBaiTanList` RVA
// `0x00014140` и `CGame::DoneBaiTanList` RVA `0x000141F0` находятся выше;
// заменённый STL traversal и raw с потерянными pair-присваиваниями удалены.

// ============================================================================
// FUNCTION: CGame::ResetHonorElimilateInfo
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5986
// RVA: 0x00014390
// ADDRESS: 00414390
// PROTOTYPE: bool __thiscall ResetHonorElimilateInfo(ulong param_1)
//
// Реализовано выше через reached player/queue owners и Rust-коллекции.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// IMPLEMENTED: `CGame::LoadScriptFileData` RVA `0x00014450` находится выше.

// ============================================================================
// FUNCTION: CGame::AI
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:2346
// RVA: 0x000148A0
// ADDRESS: 004148a0
// PROTOTYPE: int __thiscall AI(void)
//
// Реализация и локальная спецификация находятся в `CGame::ai` выше.
// `BTreeMap` сохраняет signed-key order region map, `VecDeque` — list order;
// virtual region slot остаётся явным callback-ом. Rust-owned message/string и
// готовые network helpers заменяют только STL/SEH/allocation mechanics.
//

// IMPLEMENTED: CGame::ProcessTimeOutLoginPlayer RVA `0x00014A60` находится выше.
// VERIFIED_DISASSEMBLY: exact EXE `0x00414BE4..0x00414D37` подтверждает
// продолжение RemoveOnline/AppendOffline/friend loop после erase.
//

// IMPLEMENTED: CGame::SetEnemyFactions, WorldServer RVA 0x00014D90.
// Полный заменённый псевдокод и inlined STL cleanup удалены; контракт и
// provenance сохранены в верхнем `//!`.

// ============================================================================
// IMPLEMENTED: destructor `CGame` RVA `0x00014EF0` выражен автоматическим Drop полей после обязательного Release; STL/EH cleanup удалён как compiler/library noise.

// ============================================================================
// FUNCTION: CGame::CGame
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:520
// RVA: 0x00015210
// ADDRESS: 00415210
// PROTOTYPE: undefined __thiscall CGame(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// IMPLEMENTED: `CreateGame` RVA `0x00015660` находится выше; `Box<CGame>` заменяет `operator_new` и nullable global publication.

// ============================================================================
// IMPLEMENTED: полный `CGame::ReLoad` RVA `0x00015740` находится выше; exact EXE подтверждает единый epilogue и возвращаемый length-accumulator.

// ============================================================================
// IMPLEMENTED: `CGame::Init` RVA `0x00018EE0` находится выше.
// Полный raw owner удалён после переноса; соседние owners остаются ниже.

// IMPLEMENTED: `CGame::MainLoop` RVA `0x00019A00` находится выше.
// Полная ordered-композиция заканчивается legacy result `1`; технические
// SEH/stack cleanup и Windows wait/process-handle mechanics удалены.
//

// ============================================================================
// IMPLEMENTED: полный `GameThreadFunc` RVA `0x0001A310` находится выше; typed runtime adapter сохраняет Init/MainLoop/barrier/Release/Delete/event/close порядок.

// ============================================================================
// FUNCTION: tagAppCrashMgr::tagAppCrashMgr
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp
// RVA: 0x00020660
// ADDRESS: 00420660
// PROTOTYPE: undefined __thiscall tagAppCrashMgr(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagAppCrashMgr::~tagAppCrashMgr
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp
// RVA: 0x000206D0
// ADDRESS: 004206d0
// PROTOTYPE: void __thiscall ~tagAppCrashMgr(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0053bc30
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp
// RVA: 0x0013BC30
// ADDRESS: 0053bc30
// PROTOTYPE: undefined FUN_0053bc30()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
