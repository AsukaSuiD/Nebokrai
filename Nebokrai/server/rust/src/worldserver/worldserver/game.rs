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
//! `CGame::GetTeamSessionID` RVA `0x000070C0`,
//! `CGame::AppendOnlinePlayer` RVA `0x00010D50`,
//! `CGame::AppendOfflinePlayer` RVA `0x00010DF0`,
//! `CGame::AppendLoginPlayer` RVA `0x00010E50`,
//! numeric `CGame::GetRegion(long)` RVA `0x00011F20`,
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
//! `CGame::AppendDBCountry` RVA `0x00011070`,
//! `CGame::SetEnemyFactions` RVA `0x00014D90` и
//! `CGame::ClearDBData` RVA `0x0000D490`, live restore/deletion list-owner-ы
//! `0x00004B70/0x00004BB0/0x000054E0/0x00005530/0x00005560/0x00007140/
//! 0x00010CA0/0x00010CF0`, `CGame::AppendMapPlayer` RVA `0x00010A40`,
//! `CGame::CloneMapPlayer` RVA `0x00010AB0` и
//! полный `CGame::GenerateDBData` RVA `0x00012E50`,
//! `CGame::GeterateRegionDBData` RVA `0x00012860`,
//! `SaveThreadFunc` RVA `0x00001E30`, полный `CGame::AI` RVA `0x000148A0`,
//! полный `CGame::ProcessPlayerDataQueue` RVA `0x00013BD0`,
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
//! member level/position callback `0x6012A`
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
//! после чего profiling-порядок продолжается. Team/Largess/write/load/reback
//! counts принадлежат соседним owners и передаются явно; они не дублируются в
//! `CGame`. Непредставимый старым `uint` размер блокирует только refresh после
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
//! virtual region `AI` только у ненулевого `tagRegion::pRegion`; callback
//! получает `WorldRegionOwner`, поэтому raw virtual-граница сохраняет concrete
//! subtype без object slicing. Затем ровно один
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
//! `CCountryHandler::Run` и только после обоих обновляет minute start. Ещё
//! сырые `DisbandFaction` и `CCountry::AI` передаются как явные owner-callbacks;
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
//! `ResetHonorElimilateInfo` RVA `0x00014390` проходит `m_mPlayer` в map-order,
//! сбрасывает достигнутые day/week/month counters по исходной mask-семантике,
//! затем полностью очищает `m_HonorElimilateList` и повторяет reset под lock
//! `CPlayerDataQueue`. `BTreeMap`, `VecDeque` и `parking_lot::Mutex` заменяют
//! только STL/critical-section plumbing; накопительный total не меняется.
//! Ветка `OnOtherMessage(0x5FD0D)` использует тот же map как per-player список
//! уже учтённых eliminator ID: отсутствие online player и дубликат завершают
//! обработку, новая пара добавляется в хвост до чтения четырёх счётчиков.

use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::fmt;
use std::fs;
use std::future::Future;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use parking_lot::Mutex;
use rustix::system::uname;
use rustix::time::{ClockId, clock_gettime};

use crate::dbaccess::worlddb::dbcountry::{CountrySaveSnapshot, DbCountryOwner};
use crate::dbaccess::worlddb::dbgoods::DbGoodsOwner;
use crate::dbaccess::worlddb::dbmisc::{
    CDbMisc, DbMiscContext, DbMiscDoneInReport, DbMiscDoneOutBlock, DbMiscDoneOutReport,
    DbMiscLoadAuctionReport,
};
use crate::dbaccess::worlddb::largess::LargessOwner;
use crate::dbaccess::worlddb::playerdataqueue::CPlayerDataQueue;
use crate::dbaccess::worlddb::rsenemyfactions::{EnemyFactionSaveSnapshot, RsEnemyFactionsOwner};
use crate::dbaccess::worlddb::rsfaction::RsFactionOwner;
use crate::dbaccess::worlddb::rsgenvar::RsGenVarOwner;
use crate::dbaccess::worlddb::rsgodsbattle::{
    GodsBattleFactionXydSnapshot, GodsBattleNpcFactionSnapshot, RsGodsBattleOwner,
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
use crate::nets::clients::ClientConnectError;
use crate::nets::mysocket::{DEFAULT_SOCKET_TYPE, legacy_ipv4_word};
use crate::nets::networld::message::{CMessage, SendMessageError, WorldMessageHandlers};
use crate::nets::networld::mynetclient::CMyNetClient;
use crate::nets::networld::mynetserver::{CMyNetServer, WorldServerEvent};
use crate::nets::servers::{ServerCommandHandle, ServerHostError};
use crate::public::auctionlog::{
    AuctionBangUpdateOutcome, AuctionLogLoadOutcome, CAuctionLog,
};
use crate::public::date::TagTime;
use crate::public::netsessionmanager::{CNetSessionManager, NetSessionRunReport};
use crate::public::readwrite::read_to;
use crate::public::timer::{
    AsyncTimerCallbackDisposition, AsyncTimerCallbackHandler, AsyncTimerRunBlock,
    CalendarTimerRegistration, CTimer, TimerCallbackInvocation, TimerCallbackSource, TimerId,
    TimerRunReport,
};
use crate::public::tools::ini_decode;
use crate::transport::bind_tcp_ipv4;
use crate::worldserver::appworld::country::country::{CCountry, CountryKingSaveLimits};
use crate::worldserver::appworld::country::countryhandler::{
    CCountryHandler, CountryRunBlock, CountryRunReport,
};
use crate::worldserver::appworld::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, GoodsOriginalNameIndex,
};
use crate::worldserver::appworld::jjcsystem::{
    CJJcSystem, JjcRunBlock, JjcRunConfig, JjcRunContext, JjcRunReport,
};
use crate::worldserver::appworld::leiting::{
    CLeiTing, LeiTingBlock, LeiTingContext, LeiTingLocalTime, LeiTingRunReport,
};
use crate::worldserver::appworld::message::othermessage::{
    WorldOtherMessageDispatch, WorldOtherMessageOutcome, on_other_message,
};
use crate::worldserver::appworld::message::organsysmessage::{
    OrganizingConsumedLongDispatch, OrganizingDeclareFactionWarBlock,
    OrganizingDeclareFactionWarDispatch, OrganizingDeclareWarFactionListBlock,
    OrganizingDeclareWarFactionListDispatch, OrganizingFactionBillboardBlock,
    OrganizingFactionBillboardOutcome, OrganizingFactionContributorDispatch,
    OrganizingFactionExperienceDispatch, OrganizingFactionMemberStateDispatch,
    OrganizingFactionUpgradeBlock,
    OrganizingFactionUpgradeDispatch, OrganizingFactionUploadIconDispatch,
    OrganizingLeaveWordDispatch,
    OrganizingLeaveWordEditDispatch, OrganizingLeaveWordEnableDispatch,
    OrganizingPronounceDispatch, OrganizingSessionResultDispatch,
    OrganizingUnionApplicationDispatch,
    QueuedUnionApplicationTerminal,
    UnionApplicationConfirmationDelivery,
    WorldUnionApplicationEffectCallbacks, WorldUnionApplicationEffects,
    WorldUnionApplicationRuntimeOwner, dispatch_consumed_long, dispatch_declare_faction_war,
    dispatch_declare_war_faction_list, dispatch_faction_billboard, dispatch_faction_upgrade,
    dispatch_faction_contributor, dispatch_faction_experience, dispatch_faction_member_state,
    dispatch_faction_upload_icon,
    dispatch_leave_word, dispatch_leave_word_edit,
    dispatch_leave_word_enable, dispatch_organizing_session_result, dispatch_pronounce,
    dispatch_union_application,
};
use crate::worldserver::appworld::message::servermessage::{
    WorldLoginClientReplacement, WorldServerMessageDispatch, WorldServerMessageError,
    WorldServerMessageOutcome, on_login_client_reconnected, on_server_message,
};
use crate::worldserver::appworld::organizingsystem::faction::{
    CFaction, FactionExperienceBlock, FactionUploadIconBlock,
};
use crate::worldserver::appworld::organizingsystem::factionwarsys::{
    CFactionWarSys, FactionWarRunReport, FactionWarStopBlock, FactionWarStopContext,
};
use crate::worldserver::appworld::organizingsystem::organizingctrl::{
    COrganizingCtrl, OrganizingContributorBlock, OrganizingRunBlock, OrganizingRunReport,
    OrganizingSaveDataBlock,
    OrganizingLeaveWordBlock, OrganizingLeaveWordEditBlock, OrganizingLeaveWordEnableBlock,
    OrganizingPronounceBlock, OrganizingSaveDataReport, OrganizingUnionApplicationCallbackBlock,
    OrganizingUnionApplicationCallbackReport, OrganizingUnionApplyForJoinDispatchBlock,
    PlayerEnterGameOutcome, PlayerExitGameOutcome,
};
use crate::worldserver::appworld::organizingsystem::organizingparam::{
    COrganizingParam, OrganizingParamLoadError, OrganizingParamLoadReport,
    OrganizingTaxScheduleBlock, OrganizingTodayTaxRefreshReport, PreparedTodayTaxRefresh,
};
use crate::worldserver::appworld::organizingsystem::union::{
    CUnion, UnionApplicationEndpointBlock, UnionApplicationSessionBlock,
    UnionApplicationSessionReport, UnionFormatArgument,
};
use crate::worldserver::appworld::player::{
    CPlayer, PlayerCodecError, PlayerOrganizingUpdateError, PlayerPropertyCoefficients,
};
use crate::worldserver::appworld::region::RegionSerializationBlock;
use crate::worldserver::appworld::script::variablelist::VariableListSaveSource;
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
    WorldRegionSerializationBlock, WorldRegionSetupSerializationBlock,
};
use crate::worldserver::appworld::worldvillageregion::CWorldVillageRegion;
use crate::worldserver::appworld::worldwarregion::WorldWarRegionSerializationBlock;
use crate::worldserver::worldserver::honorranks::{
    CHonorRanks, HonorRanksNewDayBlock, HonorRanksNewDayReport,
};
use crate::worldserver::worldserver::playerranks::{
    CPlayerRanks, PlayerRanksGameServerUpdate, PlayerRanksInitializationConfig,
    PlayerRanksInitializationReport, PlayerRanksScheduleBlock, PlayerRanksSerializationBlock,
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
    InitializeCountryParameters,
    CreateGeneralVariableList,
    LoadGeneralVariableList,
    LoadGeneralVariableData,
    InitializeBaseMessage,
    InitializeSocket,
    RegisterClearShengSiShiSuCopyNumTime,
}

/// Boolean initialization calls с доказанным caller-решением.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldGameInitBooleanOwner {
    InitializeTimeToReturn,
    InitializeAttackCity,
    InitializeFourNationWar,
    InitializeVillageWar,
    InitializeCountryHandler,
    InitializeCountryWar,
    LoadIncrementShopLog,
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
    RsSetupOwnerCreated(LoadedSetupIds),
    VoidOwner(WorldGameInitVoidOwner),
    JjcConfigurationLoaded,
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
    RegionOwnerRelationInitialized {
        region_id: i32,
    },
    PlayerRanksInitialized(PlayerRanksInitializationReport),
    PlayerRanksLoaded(PlayerRanksStatRunReport),
    HonorRanksLoaded {
        started_at_ms: u32,
        finished_at_ms: u32,
        elapsed_ms: u32,
        outcome: HonorRanksLoadOutcome,
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
    WorkerStarted {
        kind: WorldGameInitWorkerKind,
        handle: WorldGameInitWorkerHandleState,
    },
    OperatorNotice(WorldGameInitOperatorNotice),
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
    DupliRegionSetup,
    Context(ContextBlock),
    Reload(WorldReloadBlock),
    JjcConfiguration,
    BooleanOwner(WorldGameInitBooleanOwner),
    OrganizingParameters(OrganizingParamLoadError),
    PlayerRanksSchedule(PlayerRanksScheduleBlock),
    PlayerRanksStat(PlayerRanksStatRunBlock),
    NetworkClient(WorldClientInitializationError),
    NetworkServer(WorldNetworkInitializationError),
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

    fn clear_string_table(&mut self);
    fn load_string_table(&mut self, package: &[u8]) -> bool;
    fn code_string_table(&mut self);

    /// Обязана сначала опубликовать новый owner, затем вызвать его `Load`.
    fn create_and_load_dupli_region_setup(&mut self) -> bool;

    fn initialize_database_layer(
        &mut self,
        initialization: WorldGameDatabaseInitialization,
    ) -> Result<(), Self::Block>;
    fn create_database_owner(&mut self, owner: WorldGameDatabaseOwner) -> Result<(), Self::Block>;
    fn create_rs_setup_owner(&mut self) -> Result<LoadedSetupIds, Self::Block>;

    fn initialize_void_owner(&mut self, owner: WorldGameInitVoidOwner);
    fn initialize_boolean_owner(&mut self, owner: WorldGameInitBooleanOwner) -> bool;
    fn load_jjc_configuration(&mut self) -> bool;
    fn load_region_parameters(&mut self, game: &mut CGame) -> bool;
    fn initialize_words_filter(&mut self, invalid_strings: &[u8], char_codes: &[u8]);
    fn initialize_region_owner_relation(&mut self, region_id: i32, region: &mut CWorldRegion);

    fn use_appellation_function(&mut self) -> bool;
    /// Возвращает достигнутый player DB-owner и его текущий caller-connection.
    fn player_database(
        &mut self,
    ) -> (&mut Self::PlayerDatabase, Option<&mut WorldTdsClient>);
    /// Возвращает уже открытый Log DB connection техническому auction-owner-у.
    fn auction_log_database(&mut self) -> Option<&mut WorldTdsClient>;
    /// Exact `CGlobeSetup::m_stSetup.dwIncrementLogDays` для history query.
    fn auction_increment_log_days(&mut self) -> u32;
    fn world_string_by_id(&mut self, string_id: &[u8]) -> Vec<u8>;

    /// Для write-worker сохраняет единственный handle, для load-worker
    /// добавляет даже пустой handle в исходный ordered owner.
    fn start_worker(&mut self, kind: WorldGameInitWorkerKind) -> WorldGameInitWorkerHandleState;
}

/// Clock/log adapters полного Init; доменные owners остаются в Context.
pub(crate) struct WorldGameInitCallbacks<'a> {
    pub(crate) get_tick: &'a mut dyn FnMut() -> u32,
    pub(crate) get_log_local_time: &'a mut dyn FnMut() -> WorldLogLocalTime,
    pub(crate) get_timer_local_time: &'a mut dyn FnMut() -> TagTime,
    pub(crate) put_log_info: &'a mut dyn FnMut(&[u8]),
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
    ReleaseIncrementShopList,
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
    ReleasePlayerRanks,
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

/// Итог `list::remove` online-ID и следующего organizing exit callback-а.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldOnlinePlayerRemoveOutcome {
    pub(crate) removed_occurrences: usize,
    pub(crate) organizing: PlayerExitGameOutcome,
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

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldUnionApplicationTerminalDispatch {
    pub(crate) request: QueuedUnionApplicationTerminal,
    pub(crate) outcome: Result<
        OrganizingUnionApplicationCallbackReport,
        OrganizingUnionApplicationCallbackBlock,
    >,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldUnionApplicationRuntimeReport {
    pub(crate) terminals: Vec<WorldUnionApplicationTerminalDispatch>,
    pub(crate) confirmations: Vec<UnionApplicationConfirmationDelivery>,
    pub(crate) endpoint_blocks: Vec<UnionApplicationEndpointBlock>,
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
    OtherMessage {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: WorldOtherMessageOutcome,
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
    OrganizingDeclareWarFactionList {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingDeclareWarFactionListDispatch,
            OrganizingDeclareWarFactionListBlock,
        >,
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
    OrganizingUnionApplication {
        source: WorldMessageSource,
        legacy_run_result: i32,
        outcome: Result<
            OrganizingUnionApplicationDispatch<UnionApplicationSessionReport>,
            WorldUnionApplicationStartBlock,
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
pub(crate) enum WorldTimerCallbackBlock {
    PlayerRanks(PlayerRanksTimerRefreshBlock),
    OrganizingTax(OrganizingTaxScheduleBlock),
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
    pub(crate) player_ranks: Vec<PlayerRanksTimerRefreshReport>,
    pub(crate) organizing_taxes: Vec<OrganizingTodayTaxRefreshReport>,
    pub(crate) finished_at_ms: u32,
    pub(crate) elapsed_ms: u32,
    pub(crate) accumulated_time_ms: u32,
    pub(crate) next_stage_started_at_ms: u32,
}

struct WorldTimerHandler<'a> {
    game: &'a CGame,
    organizing_parameters: &'a mut COrganizingParam,
    player_ranks: &'a mut CPlayerRanks,
    rs_player: &'a mut TiberiusRsPlayer,
    player_database: Option<&'a mut WorldTdsClient>,
    organizing: &'a COrganizingCtrl,
    log: &'a mut WorldLogTextOwner,
    get_log_local_time: &'a mut dyn FnMut() -> WorldLogLocalTime,
    put_log_info: &'a mut dyn FnMut(&[u8]),
    world_string_by_id: &'a mut dyn FnMut(&[u8]) -> Vec<u8>,
    refreshes: Vec<PlayerRanksTimerRefreshReport>,
    tax_refreshes: Vec<OrganizingTodayTaxRefreshReport>,
    pending_player_ranks_registration: Option<usize>,
    pending_tax_registration: Option<PreparedTodayTaxRefresh>,
}

impl<Callback, GetTick, GetTimerLocalTime>
    AsyncTimerCallbackHandler<Callback, GetTick, GetTimerLocalTime>
    for WorldTimerHandler<'_>
where
    Callback: Copy,
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
        if !is_player_ranks_event {
            return Ok(AsyncTimerCallbackDisposition::PassThrough);
        }

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

        Ok(AsyncTimerCallbackDisposition::Handled {
            next_calendar_event: Some(CalendarTimerRegistration {
                time: next_time,
                callback: invocation.callback,
                parameter: 0,
            }),
        })
    }

    fn calendar_event_registered(
        &mut self,
        _invocation: TimerCallbackInvocation<Callback>,
        event_id: TimerId,
    ) {
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

/// Результат сырой session/team/plug цепочки timeout-login owner-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldLoginTimeoutTeamExit {
    SessionMissingOrNotTeam,
    PlugMissing,
    Exited,
}

/// Явная граница ещё сырых `CSessionFactory -> CTeam -> CTeamPlug::Exit`.
pub(crate) trait WorldLoginTimeoutTeamOwner {
    fn exit_team_player(
        &mut self,
        session_id: i32,
        owner_type: i32,
        owner_id: i32,
    ) -> WorldLoginTimeoutTeamExit;
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
    pub(crate) process_message: &'a mut WorldProcessMessageStageState,
    pub(crate) refresh_high_water: &'a mut WorldRefreshInfoHighWater,
    pub(crate) collect_player_data: &'a mut WorldCollectPlayerDataRequestState,
    pub(crate) save_trigger: &'a mut WorldRunSaveTriggerState,
    pub(crate) save_lifecycle: &'a SaveDataLifecycleState,
    pub(crate) reload_flags: &'a WorldReloadProfileFlags,
    pub(crate) player_ranks_request: &'a WorldPlayerRanksRequestState,
    pub(crate) save_thread_handle: &'a mut WorldSaveThreadHandleState,
}

/// Доменные owners и точные ещё сырые callback-контексты полного MainLoop.
pub(crate) struct WorldMainLoopOwners<
    'a,
    TimerCallback,
    FactionContext,
    LeiTingContextOwner,
    DbMiscContextOwner,
    JjcContext,
    TeamOwner,
> {
    pub(crate) registry: &'a GoodsBasePropertiesRegistry,
    pub(crate) original_name_index: &'a GoodsOriginalNameIndex,
    pub(crate) coefficients: &'a PlayerPropertyCoefficients,
    pub(crate) organizing: &'a mut COrganizingCtrl,
    pub(crate) country: &'a mut CCountryHandler,
    pub(crate) honor_ranks: &'a mut CHonorRanks,
    pub(crate) organizing_parameters: &'a mut COrganizingParam,
    pub(crate) player_ranks: &'a mut CPlayerRanks,
    pub(crate) rs_player: &'a mut TiberiusRsPlayer,
    pub(crate) player_database: Option<&'a mut WorldTdsClient>,
    pub(crate) auction_log: &'a mut CAuctionLog,
    pub(crate) auction_log_database: Option<&'a mut WorldTdsClient>,
    pub(crate) session_factory: &'a mut CSessionFactory,
    pub(crate) timer: &'a mut CTimer<TimerCallback>,
    pub(crate) faction_war: &'a mut CFactionWarSys,
    pub(crate) lei_ting: &'a mut CLeiTing,
    pub(crate) db_misc: &'a mut CDbMisc,
    pub(crate) net_sessions: &'a CNetSessionManager,
    pub(crate) union_application_runtime: &'a WorldUnionApplicationRuntimeOwner,
    pub(crate) jjc: &'a mut CJJcSystem,
    pub(crate) faction_context: &'a mut FactionContext,
    pub(crate) lei_ting_context: &'a mut LeiTingContextOwner,
    pub(crate) db_misc_context: &'a mut DbMiscContextOwner,
    pub(crate) jjc_context: &'a mut JjcContext,
    pub(crate) team_owner: &'a mut TeamOwner,
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
    pub(crate) start_largess_worker: &'a mut dyn FnMut(),
    pub(crate) launch_save_thread:
        &'a mut dyn FnMut(&WorldSaveThreadLaunchRequest) -> WorldSaveThreadHandleState,
    pub(crate) region_ai: &'a mut dyn FnMut(&mut WorldRegionOwner),
    pub(crate) random: &'a mut dyn FnMut(i32) -> i32,
    pub(crate) get_timer_local_time: &'a mut dyn FnMut() -> TagTime,
    pub(crate) world_string_by_id: &'a mut dyn FnMut(&[u8]) -> Vec<u8>,
    pub(crate) format_union_world_string:
        &'a mut dyn FnMut(&[u8], &[UnionFormatArgument<'_>]) -> Vec<u8>,
    pub(crate) put_union_war_log: &'a mut dyn FnMut(&[u8]),
    pub(crate) refresh_union_owned_city: &'a mut dyn FnMut(i32, i32, i32),
    pub(crate) update_union_player: &'a mut dyn FnMut(i32),
    pub(crate) faction_level_log_enabled: bool,
    pub(crate) write_faction_level_log:
        &'a mut dyn FnMut(i32, &[u8], i32, i32, &[u8]),
    pub(crate) faction_experience_log_enabled: bool,
    pub(crate) write_faction_experience_log:
        &'a mut dyn FnMut(i32, &[u8], i32, &[u8], i32, i32),
    pub(crate) dispatch_timer:
        &'a mut dyn FnMut(&mut CTimer<TimerCallback>, TimerCallbackInvocation<TimerCallback>),
    pub(crate) get_lei_ting_local_time: &'a mut dyn FnMut() -> LeiTingLocalTime,
    pub(crate) disband_faction: &'a mut dyn FnMut(&mut COrganizingCtrl, i32, i32) -> bool,
    pub(crate) country_ai: &'a mut dyn FnMut(&mut CCountry),
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
    pub(crate) write_log_queue: u32,
    pub(crate) player_load_queue: u32,
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
    PlayerList,
    PlayerExpList,
    PlayerPropertiesUpgrade,
    Emotion,
    GoodsList,
    MonsterList,
    DropGoodsList,
    TradeList,
    SkillUsageCache,
    SkillCache,
    NewSkillMonsterList,
    GlobeSetup,
    GameSetup,
    LogSystem,
    GmList,
    PlayerGmList,
    RegionLevelSetup,
    HitLevelSetup,
    AttackCity,
    InvalidStrings,
    FourNationWar,
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

/// Прямой соседний owner без наблюдаемого return в исходном dispatcher-е.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldReloadVoidOwner {
    UpdateStringTable,
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
    PlayerList,
    Emotion,
    GoodsList,
    MonsterList,
    TradeList,
    SkillList,
    NewSkillMonsterList,
    GlobeSetup,
    LogSystem,
    GmList,
    RegionLevelSetup,
    HitLevelSetup,
    AttackCity,
    VillageWar,
    FourNationWar,
    Quest,
    IncrementShop,
    Contribute,
    Prison,
    PreciousBox,
    FairyExp,
    ChangeBody,
    BattleFairyExp,
    BattleFairyCombine,
    Synthesis,
    DaKongXiangQian,
    EquipmentCompose,
    GoodsDestroy,
    TaoZhuang,
    CiQingAndLingBao,
    AllThing,
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
    /// Возвращает исходный 32-битный result; bool owners обязаны дать `0/1`.
    fn call_boolean_owner(&mut self, owner: WorldReloadBooleanOwner) -> u32;
    fn call_void_owner(&mut self, owner: WorldReloadVoidOwner);
    fn serialize_owner(&mut self, owner: WorldReloadSerializationOwner) -> Vec<u8>;
    fn add_log_text(&mut self, payload: &[u8]);
    fn notify_reload_operator(&mut self, title: &[u8], message: &[u8]);

    /// Возвращает script paths в порядке исходного resource-owner-а.
    fn script_files(&mut self, pattern: &[u8], extension: &[u8]) -> Vec<Vec<u8>>;
    /// Выполняет оставшийся inline-parser `setup/sysboardcast.ini`, включая
    /// operator notice, random/tick и единственный success log.
    fn reload_broadcast_list(&mut self, game: &mut CGame);

    /// Сохраняет два process-global счётчика после прямого region-owner load.
    fn add_region_object_counts(&mut self, monsters: i32, npcs: i32) -> (i32, i32);
    fn region_object_counts(&mut self) -> (i32, i32);
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
    BlockedRegionOwner {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldReloadBlock {
    RegionList(WorldRegionListBlock),
    RegionSnapshot(WorldReloadRegionSnapshotBlock),
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

/// Минимальная достигнутая часть исходного `CGame::tagGameServer`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldGameServerEntry {
    pub(crate) connected: bool,
    pub(crate) index: u32,
    pub(crate) ip: Vec<u8>,
    pub(crate) port: Option<u32>,
    pub(crate) received_player_data: Option<i32>,
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

/// Достигнутая setup-часть исходного `CGame`; другие поля добавляются owners.
pub(crate) struct CGame {
    setup: WorldSetup,
    globe_variables: WorldGlobeVariables,
    net_client: Option<CMyNetClient>,
    net_server: Option<CMyNetServer>,
    regions: BTreeMap<i32, WorldRegionAssignment>,
    function_list_file_data: Option<Vec<u8>>,
    variable_list_file_data: Option<Vec<u8>>,
    script_file_data: BTreeMap<Vec<u8>, Vec<u8>>,
    game_servers: BTreeMap<u32, WorldGameServerEntry>,
    system_broadcasts: VecDeque<WorldSystemBroadcast>,
    player_data_queue: CPlayerDataQueue,
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

    /// Создаёт `tagSetup`, затем применяет четыре точные записи `CGame::CGame`.
    pub(crate) fn new() -> Self {
        Self {
            setup: WorldSetup::for_game(),
            globe_variables: WorldGlobeVariables::default(),
            net_client: None,
            net_server: None,
            regions: BTreeMap::new(),
            function_list_file_data: None,
            variable_list_file_data: None,
            script_file_data: BTreeMap::new(),
            game_servers: BTreeMap::new(),
            system_broadcasts: VecDeque::new(),
            player_data_queue: CPlayerDataQueue::new(),
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

    /// Возвращает byte-exact script buffer по case-sensitive normalized key.
    pub(crate) fn get_script_file_data(&self, path: &[u8]) -> Option<&[u8]> {
        self.script_file_data
            .get(legacy_c_string_prefix(path))
            .map(Vec::as_slice)
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

    /// Выполняет полный case-insensitive dispatcher `CGame::ReLoad`.
    pub(crate) fn reload<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
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
                let player = context.call_boolean_owner(WorldReloadBooleanOwner::PlayerList);
                context.add_log_text(if player != 0 {
                    b"Load PlayerList playerOrginEquip.ini...OK!"
                } else {
                    b"Load PlayerList playerOrginEquip.ini...FAILED!"
                });
                let experience = context.call_boolean_owner(WorldReloadBooleanOwner::PlayerExpList);
                context.add_log_text(if player & experience != 0 {
                    b"Load PlayerExpList playerExp.ini...OK!"
                } else {
                    b"Load PlayerExpList playerExp.ini...FAILED!"
                });
                let properties =
                    context.call_boolean_owner(WorldReloadBooleanOwner::PlayerPropertiesUpgrade);
                let player_complete = player & experience & properties != 0;
                context.add_log_text(if player_complete {
                    b"Load playerPropertiesUpgrade.ini...OK!"
                } else {
                    b"Load Player Property Upgrade List playerPropertiesUpgrade.ini...FAILED!"
                });
                if player_complete && send_to_game_servers {
                    self.serialize_reload_owner(
                        context,
                        WorldReloadSerializationOwner::PlayerList,
                        1,
                        true,
                        &mut legacy_result,
                    );
                }
                let emotion = Self::reload_boolean_with_log(
                    context,
                    WorldReloadBooleanOwner::Emotion,
                    b"Load Emotins.ini...OK!",
                    b"Load Emotins.ini...FAILED!",
                );
                if emotion && send_to_game_servers {
                    self.serialize_reload_owner(
                        context,
                        WorldReloadSerializationOwner::Emotion,
                        0x15,
                        true,
                        &mut legacy_result,
                    );
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
                if Self::reload_boolean_with_log(
                    context,
                    WorldReloadBooleanOwner::TradeList,
                    b"Load tradelist.ini...OK!",
                    b"Load tradelist.ini...FAILED!",
                ) && send_to_game_servers
                {
                    self.serialize_reload_owner(
                        context,
                        WorldReloadSerializationOwner::TradeList,
                        3,
                        true,
                        &mut legacy_result,
                    );
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
                if Self::reload_boolean_with_log(
                    context,
                    WorldReloadBooleanOwner::NewSkillMonsterList,
                    b"Load NewSkillMonsterList.xml...ok!",
                    b"Load NewSkillMonsterList.xml...failed!",
                ) && send_to_game_servers
                {
                    self.serialize_reload_owner(
                        context,
                        WorldReloadSerializationOwner::NewSkillMonsterList,
                        0x22,
                        true,
                        &mut legacy_result,
                    );
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
                context.call_void_owner(WorldReloadVoidOwner::UpdateStringTable);
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
            WorldReloadProfile::RegionLevelSetup | WorldReloadProfile::HitLevelSetup => {
                let (owner, serializer, subcode, ok, failed) =
                    if profile == WorldReloadProfile::RegionLevelSetup {
                        (
                            WorldReloadBooleanOwner::RegionLevelSetup,
                            WorldReloadSerializationOwner::RegionLevelSetup,
                            0x11,
                            b"Load regionlevelsetup.ini...OK!".as_slice(),
                            b"Load regionlevelsetup.ini...FAILED!".as_slice(),
                        )
                    } else {
                        (
                            WorldReloadBooleanOwner::HitLevelSetup,
                            WorldReloadSerializationOwner::HitLevelSetup,
                            0x14,
                            b"Load hitlevel.ini...OK!".as_slice(),
                            b"Load hitlevel.ini...FAILED!".as_slice(),
                        )
                    };
                if Self::reload_boolean_with_log(context, owner, ok, failed) && send_to_game_servers
                {
                    self.serialize_reload_owner(
                        context,
                        serializer,
                        subcode,
                        true,
                        &mut legacy_result,
                    );
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
                if context.call_boolean_owner(WorldReloadBooleanOwner::InvalidStrings) != 0 {
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
                self.reload_simple_serialized(
                    context,
                    WorldReloadBooleanOwner::IncrementShop,
                    WorldReloadSerializationOwner::IncrementShop,
                    4,
                    b"Load IncrementShopList...OK!",
                    b"Load IncrementShopList...FAILED!",
                    send_to_game_servers,
                    true,
                    &mut legacy_result,
                );
            }
            WorldReloadProfile::Contribute => {
                self.reload_simple_serialized(
                    context,
                    WorldReloadBooleanOwner::Contribute,
                    WorldReloadSerializationOwner::Contribute,
                    5,
                    b"Load ContributeSetup.ini...OK!",
                    b"Load ContributeSetup.ini...FAILED!",
                    send_to_game_servers,
                    true,
                    &mut legacy_result,
                );
            }
            WorldReloadProfile::Prison => {
                self.reload_simple_serialized(
                    context,
                    WorldReloadBooleanOwner::Prison,
                    WorldReloadSerializationOwner::Prison,
                    0x1D,
                    b"Load PrisonConf.ini...OK!",
                    b"Load PrisonConf.ini...FAILED!",
                    send_to_game_servers,
                    true,
                    &mut legacy_result,
                );
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
                let _ = Self::reload_boolean_with_log(
                    context,
                    WorldReloadBooleanOwner::CountryWar,
                    b"Load CountryWar...OK!",
                    b"Load CountryWar...FAILED!",
                );
            }
            WorldReloadProfile::BattleFairyExp => {
                self.reload_simple_serialized(
                    context,
                    WorldReloadBooleanOwner::BattleFairyExp,
                    WorldReloadSerializationOwner::BattleFairyExp,
                    0x2C,
                    b"Add BattleFairyExpConfig.ini...ok!",
                    b"Add BattleFairyExpConfig.ini...failed!",
                    send_to_game_servers,
                    true,
                    &mut legacy_result,
                );
            }
            WorldReloadProfile::BattleFairyCombine => {
                self.reload_simple_serialized(
                    context,
                    WorldReloadBooleanOwner::BattleFairyCombine,
                    WorldReloadSerializationOwner::BattleFairyCombine,
                    0x2D,
                    b"Add BattleFairyCombineConfig.xml...ok!",
                    b"Add BattleFairyCombineConfig.xml...failed!",
                    send_to_game_servers,
                    true,
                    &mut legacy_result,
                );
            }
            WorldReloadProfile::Synthesis => {
                self.reload_simple_serialized(
                    context,
                    WorldReloadBooleanOwner::Synthesis,
                    WorldReloadSerializationOwner::Synthesis,
                    0x21,
                    b"Load synthesis.xml...ok!",
                    b"Load synthesis.xml...failed!",
                    send_to_game_servers,
                    true,
                    &mut legacy_result,
                );
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
                self.reload_simple_serialized(
                    context,
                    WorldReloadBooleanOwner::EquipmentCompose,
                    WorldReloadSerializationOwner::EquipmentCompose,
                    0x30,
                    b"Load EquipmentCompose.ini...ok!",
                    b"Load EquipmentCompose.ini...failed!",
                    send_to_game_servers,
                    false,
                    &mut legacy_result,
                );
            }
            WorldReloadProfile::GoodsDestroy => {
                self.reload_simple_serialized(
                    context,
                    WorldReloadBooleanOwner::GoodsDestroy,
                    WorldReloadSerializationOwner::GoodsDestroy,
                    0x23,
                    b"Load GoodsDestroyConf.ini...ok!",
                    b"Load GoodsDestroyConf.ini...failed!",
                    send_to_game_servers,
                    false,
                    &mut legacy_result,
                );
            }
            WorldReloadProfile::HonorEliminate => {
                let _ = Self::reload_boolean_with_log(
                    context,
                    WorldReloadBooleanOwner::HonorEliminate,
                    b"Load HonorElimilate.ini Config...ok!",
                    b"Load HonorElimilate.ini Config...failed!",
                );
            }
            WorldReloadProfile::TaoZhuang => {
                self.reload_simple_serialized(
                    context,
                    WorldReloadBooleanOwner::TaoZhuang,
                    WorldReloadSerializationOwner::TaoZhuang,
                    0x34,
                    b"Load TaoZhuang config...ok!",
                    b"Load TaoZhuang config...failed!",
                    send_to_game_servers,
                    false,
                    &mut legacy_result,
                );
            }
            WorldReloadProfile::CiQing => {
                let succeeded = Self::reload_boolean_with_log(
                    context,
                    WorldReloadBooleanOwner::CiQing,
                    b"Add ciqing.ini...ok!",
                    b"Add ciqing.ini...failed!",
                );
                context.call_void_owner(WorldReloadVoidOwner::LingBao);
                if succeeded && send_to_game_servers {
                    self.serialize_reload_owner(
                        context,
                        WorldReloadSerializationOwner::CiQingAndLingBao,
                        0x35,
                        false,
                        &mut legacy_result,
                    );
                }
            }
            WorldReloadProfile::Jjc => {
                let _ = Self::reload_boolean_with_log(
                    context,
                    WorldReloadBooleanOwner::Jjc,
                    b"Load JJcConfig.ini...ok!",
                    b"Load JJcCoinfig.ini...failed!",
                );
            }
            WorldReloadProfile::AllThing => {
                self.reload_simple_serialized(
                    context,
                    WorldReloadBooleanOwner::AllThing,
                    WorldReloadSerializationOwner::AllThing,
                    0x36,
                    b"Load LeitingAction.ini...ok!",
                    b"Load...failed!",
                    send_to_game_servers,
                    false,
                    &mut legacy_result,
                );
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
                self.clone_map_player(player_id, registry, organizing_ctrl, coefficients)?
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
    #[allow(
        clippy::too_many_arguments,
        reason = "прямые PlayerRanks/timer owners заменяют два прежних opaque callbacks"
    )]
    pub(crate) async fn init<Context, TimerCallback>(
        &mut self,
        runtime_directory: &Path,
        context: &mut Context,
        organizing_parameters: &mut COrganizingParam,
        player_ranks: &mut CPlayerRanks,
        timer: &mut CTimer<TimerCallback>,
        organizing_tax_callback: TimerCallback,
        player_ranks_callback: TimerCallback,
        organizing: &COrganizingCtrl,
        honor_ranks: &mut CHonorRanks,
        auction_log: &mut CAuctionLog,
        log: &mut WorldLogTextOwner,
        callbacks: &mut WorldGameInitCallbacks<'_>,
    ) -> WorldGameInitResult<Context::Block>
    where
        Context: WorldGameInitContext,
        TimerCallback: Copy,
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

        context.clear_string_table();
        events.push(WorldGameInitEvent::StringTablesCleared);
        const DEFAULT_LANGUAGE: &[u8] = b"data/Language.lag";
        if !context.load_string_table(DEFAULT_LANGUAGE) {
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
        if !context.load_string_table(&configured_language) {
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
        context.code_string_table();
        events.push(WorldGameInitEvent::StringTablesCoded);

        if !context.create_and_load_dupli_region_setup() {
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
        if !context.load_jjc_configuration() {
            stop!(WorldGameInitBlockReason::JjcConfiguration);
        }
        events.push(WorldGameInitEvent::JjcConfigurationLoaded);

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

        const REMAINING_DATABASE_OWNERS: &[WorldGameDatabaseOwner] = &[
            WorldGameDatabaseOwner::RsGenVar,
            WorldGameDatabaseOwner::RsFaction,
            WorldGameDatabaseOwner::RsUnion,
            WorldGameDatabaseOwner::RsEnemyFactions,
            WorldGameDatabaseOwner::RsVillageWar,
            WorldGameDatabaseOwner::RsCityWar,
            WorldGameDatabaseOwner::RsRegion,
            WorldGameDatabaseOwner::DbCountry,
            WorldGameDatabaseOwner::GoodsWarMember,
            WorldGameDatabaseOwner::DbMisc,
            WorldGameDatabaseOwner::RsGodsBattle,
        ];
        for &owner in REMAINING_DATABASE_OWNERS {
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
            let legacy_result = match self.reload(context, profile, false, false) {
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
            let legacy_result = match self.reload(context, profile, false, false) {
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

        context.initialize_words_filter(b"setup/InvalidStr.ini", b"setup/charcode.ini");
        events.push(WorldGameInitEvent::WordsFilterInitialized);
        for &profile in &[
            b"BattleFairyExpConfig".as_slice(),
            b"BattleFairyCombineConfig",
        ] {
            let legacy_result = match self.reload(context, profile, false, false) {
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

        let legacy_result = match self.reload(context, b"godsBattle", false, false) {
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
            let localized = context.world_string_by_id(b"XBWS0021");
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
            let Some(region) = self
                .regions
                .get_mut(&region_id)
                .and_then(|assignment| assignment.region.as_mut())
            else {
                continue;
            };
            context.initialize_region_owner_relation(region_id, region.base_mut());
            events.push(WorldGameInitEvent::RegionOwnerRelationInitialized { region_id });
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

        let owner = WorldGameInitVoidOwner::InitializeCountryParameters;
        context.initialize_void_owner(owner);
        events.push(WorldGameInitEvent::VoidOwner(owner));

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

        let owner = WorldGameInitBooleanOwner::InitializeCountryHandler;
        let succeeded = context.initialize_boolean_owner(owner);
        events.push(WorldGameInitEvent::BooleanOwner { owner, succeeded });
        if !succeeded {
            stop!(WorldGameInitBlockReason::BooleanOwner(owner));
        }
        self.record_game_init_log(&mut events, log, callbacks, b"Load Country SUCCESS...");

        let owner = WorldGameInitBooleanOwner::InitializeCountryWar;
        let succeeded = context.initialize_boolean_owner(owner);
        events.push(WorldGameInitEvent::BooleanOwner { owner, succeeded });
        if !succeeded {
            stop!(WorldGameInitBlockReason::BooleanOwner(owner));
        }

        for owner in [
            WorldGameInitVoidOwner::CreateGeneralVariableList,
            WorldGameInitVoidOwner::LoadGeneralVariableList,
            WorldGameInitVoidOwner::LoadGeneralVariableData,
        ] {
            context.initialize_void_owner(owner);
            events.push(WorldGameInitEvent::VoidOwner(owner));
        }

        let owner = WorldGameInitBooleanOwner::LoadIncrementShopLog;
        let succeeded = context.initialize_boolean_owner(owner);
        events.push(WorldGameInitEvent::BooleanOwner { owner, succeeded });
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

        let increment_log_days = context.auction_increment_log_days();
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
        context.initialize_void_owner(WorldGameInitVoidOwner::RegisterClearShengSiShiSuCopyNumTime);
        events.push(WorldGameInitEvent::VoidOwner(
            WorldGameInitVoidOwner::RegisterClearShengSiShiSuCopyNumTime,
        ));

        let kind = WorldGameInitWorkerKind::WriteLog;
        let handle = context.start_worker(kind);
        events.push(WorldGameInitEvent::WorkerStarted { kind, handle });
        for worker_index in 0..player_load_thread_count {
            let kind = WorldGameInitWorkerKind::LoadPlayerData { worker_index };
            let handle = context.start_worker(kind);
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
    pub(crate) fn release<Context: WorldGameReleaseContext>(
        &mut self,
        context: &mut Context,
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

        for owner in [
            WorldGameReleaseVoidOwner::ReleaseIncrementShopList,
            WorldGameReleaseVoidOwner::UninitializeTimeToReturn,
            WorldGameReleaseVoidOwner::UninitializeIncrementLog,
            WorldGameReleaseVoidOwner::ReleaseCountryHandler,
            WorldGameReleaseVoidOwner::ReleaseWordsFilter,
            WorldGameReleaseVoidOwner::ReleaseOrganizingController,
            WorldGameReleaseVoidOwner::ReleaseAttackCity,
            WorldGameReleaseVoidOwner::ReleaseVillageWar,
            WorldGameReleaseVoidOwner::ReleaseOrganizingParameters,
            WorldGameReleaseVoidOwner::ReleaseQuestSystem,
            WorldGameReleaseVoidOwner::ReleaseFactionWar,
            WorldGameReleaseVoidOwner::ReleasePlayerRanks,
            WorldGameReleaseVoidOwner::ReleaseTimer,
        ] {
            context.release_void_owner(owner);
            events.push(WorldGameReleaseEvent::VoidOwner(owner));
        }

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
            WorldGameReleaseDatabaseOwner::GoodsWarMember,
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
        for owner in [
            WorldGameReleaseOptionalOwner::DefaultClientResource,
            WorldGameReleaseOptionalOwner::DupliRegionSetup,
        ] {
            let released = context.release_optional_owner(owner);
            events.push(WorldGameReleaseEvent::OptionalOwner { owner, released });
        }

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
    pub(crate) fn ai<RegionAi, GetTick, Random>(
        &mut self,
        region_ai: &mut RegionAi,
        get_tick: &mut GetTick,
        random: &mut Random,
    ) -> WorldGameAiReport
    where
        RegionAi: FnMut(&mut WorldRegionOwner),
        GetTick: FnMut() -> u32,
        Random: FnMut(i32) -> i32,
    {
        let mut region_ids_run = Vec::new();
        for (&region_id, assignment) in &mut self.regions {
            let Some(region) = assignment.region.as_mut() else {
                continue;
            };
            region_ai(region);
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
    pub(crate) fn run_main_loop_ai_stage<RegionAi, GetTick, Random>(
        &mut self,
        clocks: &mut WorldMainLoopClockState,
        profile_state: &mut WorldMainLoopProfileState,
        mut region_ai: RegionAi,
        mut get_tick: GetTick,
        mut random: Random,
    ) -> WorldMainLoopAiStageReport
    where
        RegionAi: FnMut(&mut WorldRegionOwner),
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
        let ai = self.ai(&mut region_ai, &mut get_tick, &mut random);
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
    /// ветви server-owner-а, honor `0x5FD0C/0x5FD0D`, organizing session
    /// result, union application `0x60118`, leave-word enable `0x6011A`, запись
    /// `0x6011B`, её удаление `0x6011C`, объявление `0x6011D`, список целей
    /// войны `0x6011E`, само объявление `0x6011F` и общий leaf
    /// `0x60121/0x60123` исполняются; остальные
    /// остаются owned pending. Terminal actions применяются FIFO до следующего
    /// сообщения.
    pub(crate) fn process_message(
        &mut self,
        honor_ranks: &mut CHonorRanks,
        organizing: &mut COrganizingCtrl,
        organizing_parameters: &COrganizingParam,
        faction_war_sys: &mut CFactionWarSys,
        registry: &GoodsBasePropertiesRegistry,
        original_name_index: &GoodsOriginalNameIndex,
        coefficients: &PlayerPropertyCoefficients,
        net_sessions: &CNetSessionManager,
        application_runtime: &WorldUnionApplicationRuntimeOwner,
        application_callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<WorldProcessMessageOutcome, WorldProcessMessageError> {
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
                            organizing,
                            organizing_parameters,
                            faction_war_sys,
                            registry,
                            original_name_index,
                            coefficients,
                            net_sessions,
                            application_runtime,
                            application_callbacks,
                            update_player,
                            WorldMessageSource::GameServer,
                            message,
                        ));
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
                    organizing,
                    organizing_parameters,
                    faction_war_sys,
                    registry,
                    original_name_index,
                    coefficients,
                    net_sessions,
                    application_runtime,
                    application_callbacks,
                    update_player,
                    WorldMessageSource::LoginServer,
                    message,
                ));
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
    pub(crate) fn process_message_main_loop_stage<GetTick>(
        &mut self,
        honor_ranks: &mut CHonorRanks,
        organizing: &mut COrganizingCtrl,
        organizing_parameters: &COrganizingParam,
        faction_war_sys: &mut CFactionWarSys,
        registry: &GoodsBasePropertiesRegistry,
        original_name_index: &GoodsOriginalNameIndex,
        coefficients: &PlayerPropertyCoefficients,
        net_sessions: &CNetSessionManager,
        application_runtime: &WorldUnionApplicationRuntimeOwner,
        application_callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
        update_player: &mut dyn FnMut(i32),
        clocks: &mut WorldMainLoopClockState,
        state: &mut WorldProcessMessageStageState,
        mut get_tick: GetTick,
    ) -> WorldProcessMessageStageReport
    where
        GetTick: FnMut() -> u32,
    {
        let started_at_ms = get_tick();
        let outcome = match self.process_message(
            honor_ranks,
            organizing,
            organizing_parameters,
            faction_war_sys,
            registry,
            original_name_index,
            coefficients,
            net_sessions,
            application_runtime,
            application_callbacks,
            update_player,
        ) {
            Ok(outcome) => outcome,
            Err(error) => {
                return WorldProcessMessageStageReport::Blocked {
                    started_at_ms,
                    error,
                };
            }
        };
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
        let ai = factory.ai();
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
            let Some(mut player) = entry.take_player() else {
                let login_delivery = self.send_player_data_queue_rejection(&cdkey);
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
                let login_delivery = self.send_player_data_queue_rejection(&cdkey);
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
                let login_delivery = self.send_player_data_queue_rejection(&cdkey);
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
            add_legacy_c_string(login_reply.base_mut(), &cdkey);
            add_legacy_c_string(login_reply.base_mut(), &game_server_ip);
            login_reply.base_mut().add_ulong(game_server_port);
            add_legacy_c_string(login_reply.base_mut(), player.get_name());
            login_reply.base_mut().add_byte(player.get_level());
            let login_delivery = login_reply.send(
                self.current_login_client().map(CMyNetClient::send_queue),
                false,
            );

            let mut friend_updates = Vec::with_capacity(player.friend_count());
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
                if !online {
                    friend_updates.push(WorldFriendPresenceUpdate {
                        friend_index,
                        player_id: 0,
                        online: false,
                        target_game_server_index: None,
                        delivery: None,
                    });
                    continue;
                }

                let target_game_server_index = self
                    .online_player_by_id(friend_player_id)
                    .and_then(|friend| self.get_region_game_server(friend.get_region_id()))
                    .map(|game_server| game_server.index);
                let mut presence = CMessage::new(0x0007_F904);
                presence.base_mut().add_ulong(friend_player_id);
                add_legacy_c_string(presence.base_mut(), player.get_name());
                let sender = self.current_game_server_sender();
                let delivery = presence.send_to_map_id(
                    sender.as_ref(),
                    target_game_server_index.unwrap_or(0) as i32,
                );
                friend_updates.push(WorldFriendPresenceUpdate {
                    friend_index,
                    player_id: friend_player_id,
                    online: true,
                    target_game_server_index,
                    delivery: Some(delivery),
                });
            }

            let online_removal = self.remove_online_player(organizing_ctrl, player_id);
            self.remove_offline_player(player_id);
            let login_time_ms = get_tick();
            self.append_login_player(player_id, login_time_ms);

            let replaced_existing_player = self.players.remove(&player_id).is_some();
            let append_outcome = self.append_map_player(player, |_| {});
            let WorldMapPlayerAppendOutcome::Inserted {
                player_id: inserted_player_id,
            } = append_outcome
            else {
                unreachable!("map key удалён непосредственно перед AppendMapPlayer")
            };
            debug_assert_eq!(inserted_player_id, player_id);

            return Ok(WorldProcessPlayerDataQueueOutcome::Accepted {
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
            });
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
        clocks: &mut WorldMainLoopClockState,
        profile_state: &mut WorldMainLoopProfileState,
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
        dispatch: &mut Dispatch,
    ) -> Result<WorldMainLoopTimerStageReport, WorldMainLoopTimerStageBlock>
    where
        Callback: Copy,
        GetTick: FnMut() -> u32 + ?Sized,
        GetTimerLocalTime: FnMut() -> TagTime + ?Sized,
        Dispatch: FnMut(&mut CTimer<Callback>, TimerCallbackInvocation<Callback>) + ?Sized,
    {
        let mut handler = WorldTimerHandler {
            game: self,
            organizing_parameters,
            player_ranks,
            rs_player,
            player_database,
            organizing,
            log,
            get_log_local_time,
            put_log_info,
            world_string_by_id,
            refreshes: Vec::new(),
            tax_refreshes: Vec::new(),
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
        let player_ranks = handler.refreshes;
        let organizing_taxes = handler.tax_refreshes;
        let finished_at_ms = get_tick();
        let elapsed_ms = finished_at_ms.wrapping_sub(clocks.stage_started_at_ms);
        profile_state.timer_time_ms = profile_state.timer_time_ms.wrapping_add(elapsed_ms);
        let next_stage_started_at_ms = get_tick();
        clocks.stage_started_at_ms = next_stage_started_at_ms;
        Ok(WorldMainLoopTimerStageReport {
            timer: timer_report,
            player_ranks,
            organizing_taxes,
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
        &self,
        lei_ting: &mut CLeiTing,
        context: &mut Context,
        mut get_local_time: GetLocalTime,
    ) -> Result<LeiTingRunReport, LeiTingBlock<Context::Block>>
    where
        Context: LeiTingContext,
        GetLocalTime: FnMut() -> LeiTingLocalTime,
    {
        let current = get_local_time();
        lei_ting.run(current, context)
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
    pub(crate) fn run_main_loop_minute_stage<GetTick, Disband, CountryAi>(
        &mut self,
        initialization: &mut WorldMainLoopInitializationState,
        clocks: &mut WorldMainLoopTailClockState,
        organizing: &mut COrganizingCtrl,
        country_handler: &mut CCountryHandler,
        mut get_tick: GetTick,
        disband_faction: Disband,
        country_ai: CountryAi,
    ) -> Result<WorldMainLoopMinuteStageReport, WorldMainLoopMinuteStageBlock>
    where
        GetTick: FnMut() -> u32,
        Disband: FnMut(&mut COrganizingCtrl, i32, i32) -> bool,
        CountryAi: FnMut(&mut CCountry),
    {
        let initialized = initialize_main_loop_tail_clocks(initialization, clocks, &mut get_tick);
        let current_tick_ms = get_tick();
        clocks.current_tick_ms = current_tick_ms;
        let minute_delta = current_tick_ms
            .wrapping_sub(clocks.minute_started_at_ms)
            .wrapping_div(60_000) as i32;

        let organizing = organizing
            .run(minute_delta, disband_faction)
            .map_err(WorldMainLoopMinuteStageBlock::Organizing)?;
        let country = country_handler
            .run(minute_delta, &mut get_tick, country_ai)
            .map_err(WorldMainLoopMinuteStageBlock::Country)?;
        clocks.minute_started_at_ms = current_tick_ms;

        Ok(WorldMainLoopMinuteStageReport {
            initialization: initialized,
            current_tick_ms,
            minute_delta,
            organizing,
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

    /// Обходит весь login-list по одному общему tick snapshot и освобождает
    /// только просроченные записи, у которых ещё существует player-owner.
    pub(crate) fn process_time_out_login_player<GetTick, TeamOwner>(
        &mut self,
        release_interval_ms: u32,
        organizing: &mut COrganizingCtrl,
        mut get_tick: GetTick,
        team_owner: &mut TeamOwner,
    ) -> WorldLoginTimeoutReport
    where
        GetTick: FnMut() -> u32,
        TeamOwner: WorldLoginTimeoutTeamOwner,
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
            let team_exit = team_owner.exit_team_player(team_session_id, owner_type, owner_id);

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
        reason = "clock, wait/debug adapters и сырой team-owner являются разными границами"
    )]
    pub(crate) fn run_main_loop_tail_stage<GetTick, Wait, DebugOutput, TeamOwner>(
        &mut self,
        clocks: &mut WorldMainLoopTailClockState,
        release_state: &mut WorldMainLoopLoginReleaseState,
        organizing: &mut COrganizingCtrl,
        mut get_tick: GetTick,
        mut wait: Wait,
        mut output_debug: DebugOutput,
        team_owner: &mut TeamOwner,
    ) -> WorldMainLoopTailStageReport
    where
        GetTick: FnMut() -> u32,
        Wait: FnMut(u32),
        DebugOutput: FnMut(&'static str),
        TeamOwner: WorldLoginTimeoutTeamOwner,
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
                team_owner,
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
        TeamOwner,
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
            TeamOwner,
        >,
        callbacks: &mut WorldMainLoopCallbacks<'_, TimerCallback>,
    ) -> WorldMainLoopResult<FactionContext::Block, LeiTingContextOwner::Block>
    where
        TimerCallback: Copy,
        FactionContext: FactionWarStopContext,
        LeiTingContextOwner: LeiTingContext,
        DbMiscContextOwner: DbMiscContext,
        JjcContext: JjcRunContext,
        TeamOwner: WorldLoginTimeoutTeamOwner,
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
                (callbacks.start_largess_worker)();
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
            &mut *callbacks.get_log_local_time,
        );
        let reload = match reload {
            complete @ WorldReloadProfilesReport::Complete { .. } => complete,
            blocked @ (WorldReloadProfilesReport::BlockedMissingFact { .. }
            | WorldReloadProfilesReport::BlockedRegionSetup { .. }
            | WorldReloadProfilesReport::BlockedRegionOwner { .. }) => {
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
            &mut *callbacks.region_ai,
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
            owners.organizing,
            owners.organizing_parameters,
            owners.faction_war,
            owners.registry,
            owners.original_name_index,
            owners.coefficients,
            owners.net_sessions,
            owners.union_application_runtime,
            &mut union_application_callbacks,
            &mut *callbacks.update_union_player,
            state.clocks,
            state.process_message,
            &mut *callbacks.get_tick,
        ) {
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
                state.clocks,
                state.profile,
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
                &mut *callbacks.get_tick,
                &mut *callbacks.disband_faction,
                &mut *callbacks.country_ai,
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
            owners.team_owner,
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
            write_log_queue: external.write_log_queue,
            player_load_queue: external.player_load_queue,
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

    /// Добавляет унаследованный ID игрока в хвост, если его ещё нет в списке.
    pub(crate) fn append_offline_player(&mut self, player: &CPlayer) {
        let _ = self.append_offline_player_id(player.get_id() as u32);
    }

    fn append_offline_player_id(&mut self, player_id: u32) -> bool {
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

    /// Удаляет первую login-запись с указанным ID либо сохраняет список.
    pub(crate) fn remove_login_player(&mut self, player_id: u32) {
        let Some(index) = self
            .login_players
            .iter()
            .position(|login_player| login_player.player_id == player_id)
        else {
            return;
        };
        let _ = self.login_players.remove(index);
    }

    /// Возвращает регион по signed numeric ID либо старый `nullptr` как `None`.
    pub(crate) fn region(&self, region_id: i32) -> Option<&WorldRegionAssignment> {
        self.regions.get(&region_id)
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

    let release = match game_slot
        .as_deref_mut()
        .expect("Release вызывается до DeleteGame")
        .release(runtime)
    {
        Ok(release) => release,
        Err(block) => {
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

fn process_world_message(
    game: &mut CGame,
    honor_ranks: &mut CHonorRanks,
    organizing: &mut COrganizingCtrl,
    organizing_parameters: &COrganizingParam,
    faction_war_sys: &mut CFactionWarSys,
    registry: &GoodsBasePropertiesRegistry,
    original_name_index: &GoodsOriginalNameIndex,
    coefficients: &PlayerPropertyCoefficients,
    net_sessions: &CNetSessionManager,
    application_runtime: &WorldUnionApplicationRuntimeOwner,
    application_callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
    source: WorldMessageSource,
    mut message: CMessage,
) -> ProcessedWorldEvent {
    let message_type = message.message_type();
    let mut selector = WorldOwnerSelector {
        write_log_enabled: game.setup.use_log_system,
        owner: None,
    };
    let legacy_run_result = message.run(&mut selector);

    if selector.owner == Some(WorldMessageOwner::Server) {
        match on_server_message(game, message) {
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

    if selector.owner == Some(WorldMessageOwner::Other) {
        match on_other_message(game, honor_ranks, message) {
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

    if selector.owner == Some(WorldMessageOwner::OrganizingSystem) {
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
    while let Some(request) = runtime.pop_terminal() {
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
    WorldUnionApplicationRuntimeReport {
        terminals,
        confirmations: runtime.take_confirmations(),
        endpoint_blocks: runtime.take_blocks(),
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
pub(crate) fn reload_profiles<Context, GetLocalTime>(
    game: &mut CGame,
    flags: &WorldReloadProfileFlags,
    context: &mut Context,
    mut get_local_time: GetLocalTime,
) -> WorldReloadProfilesReport
where
    Context: WorldReloadContext + ?Sized,
    GetLocalTime: FnMut() -> WorldLogLocalTime,
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
            WorldReloadActionKind::Reload => match game.reload(
                context,
                action.reload_profile,
                action.first_option,
                action.second_option,
            ) {
                Ok(result) => result,
                Err(block) => {
                    return WorldReloadProfilesReport::BlockedRegionOwner {
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

fn legacy_tick_ms() -> u32 {
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

// ============================================================================
// FUNCTION: CGame::CheckInvalidString
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4592
// RVA: 0x00001B30
// ADDRESS: 00401b30
// PROTOTYPE: bool __thiscall CheckInvalidString(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::GetPlayerEquipID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4602
// RVA: 0x00001B50
// ADDRESS: 00401b50
// PROTOTYPE: void __thiscall GetPlayerEquipID(CPlayer * param_1, ulong * param_2, ulong * param_3, ulong * param_4, ulong * param_5, ulong * param_6, ulong * param_7, ulong * param_8, ulong * param_9, ulong * param_10, ulong * param_11, ulong * param_12, uchar * param_13, uchar * param_14, uchar * param_15, uchar * param_16, uchar * param_17, uchar * param_18, uchar * param_19, uchar * param_20, uchar * param_21, uchar * param_22, uchar * param_23)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED выше: SaveThreadFunc, WorldServer RVA 0x00001E30.

// ============================================================================
// FUNCTION: CGame::CodeStringTable
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5345
// RVA: 0x00001F00
// ADDRESS: 00401f00
// PROTOTYPE: void __thiscall CodeStringTable(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::GetStringTableByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5350
// RVA: 0x00001F20
// ADDRESS: 00401f20
// PROTOTYPE: vector<unsigned_char,std::allocator<unsigned_char>_> * __thiscall GetStringTableByteArray(void)
//
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5775
// RVA: 0x00002250
// ADDRESS: 00402250
// PROTOTYPE: bool __thiscall GetOptMoneyJin(CGoodsNode * param_1, long * param_2, long * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GetStringByID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.h:944
// RVA: 0x00002FB0
// ADDRESS: 00402fb0
// PROTOTYPE: char * __cdecl GetStringByID(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5324
// RVA: 0x000033F0
// ADDRESS: 004033f0
// PROTOTYPE: bool __thiscall LoadStringTable(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4101
// RVA: 0x00005790
// ADDRESS: 00405790
// PROTOTYPE: bool __thiscall IsNameExistInDBCreation(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::IsNameExistInDBData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4131
// RVA: 0x000058D0
// ADDRESS: 004058d0
// PROTOTYPE: bool __thiscall IsNameExistInDBData(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::FindGoodsLink
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4181
// RVA: 0x00005A10
// ADDRESS: 00405a10
// PROTOTYPE: tagGoodsLink * __thiscall FindGoodsLink(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::AddOrginGoodsToPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4755
// RVA: 0x00005A40
// ADDRESS: 00405a40
// PROTOTYPE: void __thiscall AddOrginGoodsToPlayer(CPlayer * param_1)
//
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:3894
// RVA: 0x000073F0
// ADDRESS: 004073f0
// PROTOTYPE: bool __thiscall ValidateDBPlayerIDinCdkey(char * param_1, uint param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::ValidatePlayerIDinCdkey
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5078
// RVA: 0x00007490
// ADDRESS: 00407490
// PROTOTYPE: bool __thiscall ValidatePlayerIDinCdkey(char * param_1, uint param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::GetRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4299
// RVA: 0x00008680
// ADDRESS: 00408680
// PROTOTYPE: tagRegion * __thiscall GetRegion(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// IMPLEMENTED: `CGame::SaveCityRegion` RVA `0x00008750` находится выше; signed map-order и type `2` сохранены, две UB-границы локализованы.

// ============================================================================
// FUNCTION: CGame::ClearStringTable
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5339
// RVA: 0x000087D0
// ADDRESS: 004087d0
// PROTOTYPE: void __thiscall ClearStringTable(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::UpdateStringTable
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5355
// RVA: 0x00008820
// ADDRESS: 00408820
// PROTOTYPE: bool __thiscall UpdateStringTable(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::IsNameExitInFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5763
// RVA: 0x000089A0
// ADDRESS: 004089a0
// PROTOTYPE: bool __thiscall IsNameExitInFaction(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::LoadServerResource
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:567
// RVA: 0x00009110
// ADDRESS: 00409110
// PROTOTYPE: bool __thiscall LoadServerResource(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: LoadPlayerDataFromDB
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5119
// RVA: 0x000092C0
// ADDRESS: 004092c0
// PROTOTYPE: uint __stdcall LoadPlayerDataFromDB(void * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: DoSaveLog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5230
// RVA: 0x000095E0
// ADDRESS: 004095e0
// PROTOTYPE: void __cdecl DoSaveLog(void)
//
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:2962
// RVA: 0x0000D350
// ADDRESS: 0040d350
// PROTOTYPE: void __thiscall DeleteMapPlayer(uint param_1)
//
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5088
// RVA: 0x0000D770
// ADDRESS: 0040d770
// PROTOTYPE: uint __stdcall ProcessWriteLogDataFunc(void * param_1)
//
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:1081
// RVA: 0x000106E0
// ADDRESS: 004106e0
// PROTOTYPE: void __cdecl FindScriptFile(char * param_1, list<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>,std::allocator<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>_>_> * param_2)
//
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:3200
// RVA: 0x00010C70
// ADDRESS: 00410c70
// PROTOTYPE: CPlayer * __thiscall CloneCreationPlayer(uint param_1)
//
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4164
// RVA: 0x000112B0
// ADDRESS: 004112b0
// PROTOTYPE: void __thiscall AddGoodsLink(tagGoodsLink * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::OnGameServerLost
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4313
// RVA: 0x00011340
// ADDRESS: 00411340
// PROTOTYPE: void __thiscall OnGameServerLost(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// IMPLEMENTED: `CGame::ReLoadOneRegionSetup` RVA `0x000120F0` находится выше; exact EXE подтверждает отдельные true/false epilogue.

// ============================================================================
// IMPLEMENTED: `CGame::LoadRegionList` RVA `0x00012220` находится выше; поставочные subtype/load/serialize вызываются напрямую, COUNTRY type `3` остаётся raw-границей.

// ============================================================================
// FUNCTION: CGame::GetCreationPlayerVectorByCdkey
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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

// ============================================================================
// FUNCTION: CGame::RefreshOwnedCityOrg
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:4445
// RVA: 0x000128E0
// ADDRESS: 004128e0
// PROTOTYPE: void __thiscall RefreshOwnedCityOrg(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

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
