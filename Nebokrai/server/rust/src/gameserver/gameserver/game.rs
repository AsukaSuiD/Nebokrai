//! Достигнутая send/receive dispatch storage-часть `CGame` GameServer.
//!
//! PDB подтверждает nullable `s_pNetClientOfWS +0x8`,
//! `s_pNetClientOfBS +0xC`, `s_pNetServer +0x10`, ordered
//! `s_mapPlayer +0x14` типа `long -> CPlayer*` и ordered
//! `m_JJcLevelData` типа `long -> long` в startup dispatcher-е и
//! `m_mTeamSessionID +0x188` типа `unsigned long -> long`. `FindPlayer` RVA
//! `0x00014930`, `GetTeamSessionID` RVA `0x00013690`, `GetTeamID` RVA
//! `0x0008BEB0` и достигнутые
//! `CMessage` sends `0x00013910..0x00014923` имеют статус `IMPLEMENTED,
//! VERIFIED_DISASSEMBLY`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`, исходники
//! `server/gameserver/gameserver/game.h/.cpp`.
//! `CGame::AI` RVA `0x00005080` сохраняет signed region-map order, virtual
//! region AI и base-tail clear countdown с warning/return side effects.
//!
//! `BTreeMap` сохраняет наблюдаемый ordered-map lookup, owned `CPlayer`
//! заменяет сырой pointer только в достигнутой runtime-проекции, а
//! `CMyNetServer/CMyNetClient` остаются отдельными historical owners. Полный
//! `CSessionFactory` также принадлежит `CGame`: его session/plug maps доступны
//! живому goods dispatcher-у без process-static pointers, а concrete lifecycle
//! пока остаётся typed runtime-границей.
//! `ProcessMessage` RVA `0x00005830` атомарно забирает FIFO строго в порядке
//! World, Billing, accepted clients и для каждого элемента вызывает
//! `CMessage::Run`; это имеет статус `IMPLEMENTED`. `InitNetServer` RVA
//! `0x000020D0`, `InitNetClientOfWS/BS` RVA `0x00002C60/0x00002DE0`,
//! `ReConnectWorldServer/BillingServer` RVA `0x0000B7B0/0x0000B8D0`, retry
//! entries RVA `0x0000BAD0/0x0000BB80` и task owners RVA
//! `0x0000BDA0/0x0000BE10` также материализованы через общий Linux transport;
//! точный return listener-а подтверждён машинным кодом. Из `Release` RVA
//! `0x00009FD0` перенесён только начальный stop/join reconnect workers. Полный
//! позиционный разбор `LoadSetup/LoadSetupEx` RVA `0x00009960/0x00009160`
//! материализован с исходными defaults, частичной мутацией и игнорированием
//! labels. Открытие listener-а заменяет поздней проверкой bind старый
//! `FindWindow` single-instance guard; недоказанный default billing bind-port
//! остаётся typed-границей. `Release` и остальные resource/runtime owners ниже
//! остаются RAW;
//! `Init` связан через обязательный World client, общий MSVCRT RNG и sequence
//! registry до необязательной Billing-попытки, затем создаёт DupliRegion,
//! move-check, ranks и GoodsWar owners. GodsBattle startup snapshot хранится
//! отдельным `CGodsBattleMgr` historical owner-ом. GoodsWar constructor
//! сохраняет немедленный World request. Tokio/socket types заменяют ненаблюдаемые
//! `CBaseMessage::Initial` и `CMySocket::MySocketInit`; следующий незакрытый
//! шаг — resource/runtime owners после завершённого `Init`.
//! QuestSystem, CountryParam, CountryHandler, AttackCity, VillageWar,
//! CountryWarSys, CFourNationWarSys и CEmotion process singletons хранятся
//! owned-полями `CGame`, сохраняя exact startup wire и runtime lookup-
//! контракты без отдельных global allocations.
//! Function/variable/script file buffers, function registry и general-variable
//! list принадлежат `CGame` от startup FIFO до `Release`. Повторный
//! function/variable setter безопасно материализует исходный freed-owner
//! контракт как `None`; старый `length + 1` NUL-padding заменён bounded `Vec` и
//! C-string prefix adapter-ом.
//! World login `0x7F901` проходит из общего message FIFO через полный player
//! GameSave decoder, transport-route validation, canonical player map и
//! spatial region membership. Затем тот же main-loop runtime выполняет login
//! property recompute и ещё не материализованные client/GoodsAI virtual owners;
//! сам `CGame` ставит login/honor scripts в живой scheduler, публикует Billing
//! `0xEF201`, обходит все подтверждённые goods containers и выполняет
//! equipment-state `2→3` с `0xBF928`. Save/faction/region/release callers
//! используют один обратный codec с live companion snapshot.
//! Reached faction `Create/ApplyJoin` sessions хранят exact correlation,
//! `1000/2000` ms timeout, client prompts и World requests; успешный create
//! callback списывает обещанные packet goods и деньги через canonical player/
//! container effects. Общий legacy async manager заменён узкими owned maps.
//! Script city-gate path использует существующий owned `CServerCityRegion` и
//! `CityGateRuntimeContext`; отдельное shadow-состояние ворот не вводится.
//! Reached `SetMe("dwVigour")` пишет поле как generic DWORD без
//! `SetVigour` clamp, затем проводит обязательный virtual
//! `UpdateProperty` и публикует полный player `0xBF721` через тот
//! же properties runtime, который уже обслуживает equipment/fairy paths.
//! Тот же owner доводит battle-fairy script `9400..9411` до canonical players,
//! enhancement/equipment goods, общего RNG, skill/property mutation, client
//! wire и file audit; goods reset `0x8FC29` вызывает его напрямую.
//! `MyStringTable` также принадлежит `CGame`: reload сначала очищает map,
//! публикует decoded prefix и сохраняет пустой fallback `GetStringByID`.
//! `CHonorRanks` хранит четыре rank-type × четыре country snapshots; поля
//! полного игрока для total-reset остаются отдельной assembly-границей.
//! `CSkillFactory` принадлежит `CGame`: startup selector `0x06` заменяет весь
//! composite-key registry, а будущие skill/state owners получают те же runtime
//! lookup-ы без process-global raw pointers.
//! `CGoodsFactory` аналогично хранит startup selector `0x00`, включая оба
//! byte-name index-а для последующего container/goods lifecycle.
//! Goods, monster и skill registries `0x00/0x02/0x06` теперь публикуются из
//! реального World FIFO; monster registry после точного log выполняет полный
//! refresh lookup-связности уже живых region monster-ов.
//! Player templates, trade list, increment shop и contribution setup
//! `0x01/0x03/0x04/0x05` входят туда же одной resource-группой с исходными
//! partial publication, cursor и success-log границами.
//! Runtime `0x90601/02/04/05` теперь использует этот же Increment Shop owner:
//! player progress/movement, ordered catalog traversal и client/World packets
//! проходят непосредственно из общего MainLoop FIFO.
//! GlobeSetup, LogSystem и GM-list `0x07/0x08/0x09` также достигаются из FIFO:
//! router/DaKong/auction/area mutations, Goods-AI broadcast и permission
//! registry публикуются в подтверждённом порядке до соответствующих logs.
//! Game ID, hit-level, emotion и quest resources `0x12/0x14/0x15/0x16`
//! проходят общий player-rule FIFO pass с точными partial/cursor/log
//! контрактами; player-ranks `0x17` соседним owner-ом сохраняет missing-init и
//! исходный allocation source.
//! Script quest owner замыкает local add/complete/remove/update на persisted
//! player map и client `0xBFF2C..2F`; remote add/remove проходят существующий
//! `0x6013B/C → World → 0x7FE38/39` route без повторной пересылки.
//! CountryParam и CountryHandler `0x18/0x19` публикуются одним country-state
//! FIFO pass, сохраняя scalar/map partial mutation, replacement и exact logs.
//! Proxy/reload/region-level/dupli selectors `0x0F/0x10/0x11/0x1A` проходят
//! общий spatial FIFO pass с canonical region owners, ранним reload miss и
//! сохранённой allocation-error причиной duplicate registry.
//! Prison/PreciousBox `0x1D/0x1E` входят в общий environment-configuration
//! FIFO pass с clear/partial-decode owners и сохранённой причиной allocation
//! failure вложенных box-списков.
//! Synthesis/new-skill/goods-destruction/change-body `0x21..0x24` проходят
//! общий mutation-rules FIFO pass с полными decode reports, partial registries,
//! точными logs и сохранёнными allocation-error sources.
//! Item-use ChangeBody guard использует этот же configuration owner и live
//! player state; прошедший addon script добавляет состояние через canonical
//! script dispatcher, property recompute и around visual publication.
//! DaKong/WordsFilter/JJC levels `0x2B/0x31/0x32` проходят общий lookup/filter
//! FIFO pass с исходными clear/append, partial publication и success logs.
//! TaoZhuang и CiQing/LingBao `0x34/0x35` проходят общий enhancement FIFO pass:
//! owners сериализуются в client wire, broadcast-ятся и логируются в exact order.
//! Leiting/GodsBattle `0x36/0x39` проходят общий world-event FIFO pass с
//! dynamic/internal/final logs и typed file-audit effects в исходном порядке.
//! Honor configuration/ranks `0x26..0x2A` проходят полный FIFO pass; total
//! snapshot сбрасывает counters canonical player map и возвращает точные
//! AdjustHonorRank script-effects для внешнего script runtime.
//! Function/variable/general/script-file resources `0x0A..0x0D` публикуются из
//! живого FIFO с duplicate-owner семантикой; function map и variable snapshot
//! декодируются concrete owner-ами `CGame`, а general cursor остаётся внешне
//! неизменным, как при передаче `long` по значению в EXE.
//! World echo `0x7F805` продолжает тот же owner: tag `1/3` меняет integer либо
//! string state первого case-insensitive имени без отдельного script runtime.
//! Соседний World notice `0x7F804` переиспользует bounded `%s` formatter и
//! concrete net server до адресного client `0xBF806`; unsafe `sprintf` не
//! воспроизводится, visible `0xFF/0x3FF` wire limits сохраняются.
//! Kill confirmation `0x7F806` продолжает World runtime path через owned Globe
//! PK coefficient и player counters/timestamp до concrete around `0xBF70E`;
//! отсутствующий player получает exact reused-message `0x5FA06` ответ.
//! OrganSys war opcodes `0x7FE1F..0x7FE36` тем же FIFO меняют owned
//! AttackCity/Village schedules, concrete local/proxy region phases и
//! contender state с сохранением City/Village message/log side effects.
//! Control tail `0x7FE46..0x7FE4A` продолжает этот route: FourNation
//! exploit/time выполняют player property/client effects и owned time-map,
//! country treasury clamp публикует exact World `0x60314`, morale меняется в
//! owned schedule, а router response переиспользует входной wire как `0xBFF36`.
//! Nation combat callback-ы продолжают эту вертикаль: `OnBeenHurted` хранит
//! first-hit flags и exact World notices, `OnDied` исполняет morale/fail
//! packets, regional notices и одноразовый YuYingShi через concrete `AddNpc`.
//! Script-facing carriage return отдельно сохраняет saturating treasure-box
//! count, wrapping morale bonus и немедленный regional `0xBF818` snapshot.
//! Nation contend проходит через те же canonical player/region owners: enter,
//! cancel, damage и AI timeout исполняют `0xBFF29`, localized notices, захват
//! с morale/top-info и три concrete treasure-box `AddNpc` в исходном порядке.
//! Связанный player-state проход сохраняет `SetContendState` around-wire,
//! Nation `OnDied` с half-duration penalty, обе relive-копии `0xBFF2A` и
//! wrapping periodic countdown. Неопределённый `AL` раннего cancel-return
//! остаётся typed outcome, а не подменяется придуманным уведомлением.
//! Перед contend-таймером тот же Nation AI исполняет строгий gate четырёх
//! стражей и адмирала: локализованный stone NPC получает explosion/removal
//! around-пакеты, удаляется из spatial owner-а и заменяется monster-ом в
//! фиксированной точке.
//! CountryWar `0x7FF17..0x7FF22` продолжает тот же lifecycle: мутирует
//! country-region phases/results, выполняет clear через concrete runtime и
//! переиспользует входной message для all/country-filtered client broadcast.
//! GoodsWar `0x7FF20/21` тем же country route публикует member/faction/count
//! snapshots в process-owned owner и сохраняет World delete notification.
//! Battle-fairy combine теперь замыкает game player-map с GlobeSetup gate и
//! maximum fetch power, exact Game RNG, обеими exp-таблицами, goods/skill
//! registry и явным old-client serializer-ом; он возвращает ordered адресные
//! effects, потому что transport encoder этого семейства ещё отдельный owner.
//! Обе exp-таблицы, combine recipes и equipment-compose maps теперь также
//! достигаются из World startup FIFO selector-ами `0x20/0x2C/0x2D/0x30`,
//! сохраняя partial publication, cursor и точные startup log-effects.
//! Equipment add/remove проведены через canonical player registry до war-soul
//! state/skills, property callbacks, remove vitals clamp и typed around
//! `0xBF720`; полный virtual property owner остаётся caller adapter-ом.
//! Periodic battle-fairy death prefix теперь также доведён через equipment
//! addon lookup и четыре player state mutation до адресного `0xBF721` в
//! точном field order. Attack speed/CCH и base vigour/credit/mode читаются из
//! canonical player state; только add-element-attack, RP/max-vigour и exalt
//! остаются typed facts ещё не сведённого полного property owner-а.
//! Summon/recall тем же property adapter-ом исполняет ordered notifications,
//! around `0xBF605/0xBF930/0xBF92E` и terminal `0xBF721`; координаты move wire
//! кодируются IEEE-754 float bits, как в `TellClientMove`, а не signed DWORD.
//! Battle-fairy gear add/remove использует тот же properties owner и шлёт
//! `0xBF918(player, GUID, length, old-client payload)` в effect-order, включая
//! подтверждённый двойной update успешного remove.
//! Skill reset `0x8FC29` сохраняет два входных long, player detach, live
//! script callback с canonical game/player/region, повторный attach и World
//! ack `0xBF931`; произвольный script не удерживает raw equipment pointer.
//! Refresh-property `0x8FC2E` доведён через exact player container lookup до
//! обязательного runtime `UpdateProperty`; неизвестная формула не заменяется
//! текущим cached combat snapshot.
//! CiQing query `0x8FC2F/30` использует owned player set и global setup:
//! previews создаются общим factory/RNG, сериализуются old-client codec-ом и
//! сохраняют ранний return первого непустого payload.
//! Make `0x8FC31` использует тот же setup/player owner: глобальный `bCiQing`,
//! session gate, recipe, packet resources/space, batch factory, container
//! ownership, World audit `0x60218`, packet `0xC0101/02` и адресный result
//! проходят одним synchronous сценарием.
//! Compose `0x8FC32` продолжает те же containers: exact/fallback recipe,
//! реальный wallet/crystal payment с money/packet wire, два RNG,
//! source/result ownership с `0xC0101/02`, unlock/query и странный append
//! result-index после отправки `0xBF81B` сохранены буквально.
//! Delete `0x8FC33` продолжает container owner с reset-item audit/removal,
//! удалением одной единицы и concrete packet/CiQing wire, обязательным
//! `UpdateProperty` и отказом `PLAYER001004`; position gate остаётся во
//! входном message owner-е.
//! Mount `0x8FC34` сохраняет read-before-gate wire, addon-driven slot/chance,
//! failure destruction и success Clone→hand delete→CiQing add с hand/packet/
//! CiQing `0xC0101/02`; полный native Clone, old-client codec и addon/property
//! recompute остаются обязательными runtime-границами, поскольку их owners
//! ещё не материализованы полностью.
//! Other-person `0x8FC35` объединяет ordered CiQing/TaoZhuang property maps,
//! сериализует target identity, values-only sequence, owned CiQing goods и
//! TaoZhuang ID в адресный `0xC010F`. Delete/mount property tail получает от
//! обязательного runtime-а полные combat snapshots до/после универсальных
//! equipment/addon формул, а `CPlayer` сам вычисляет и сохраняет CiQing delta,
//! объединяет TaoZhuang values и при изменении шлёт values-only `0xC0110`.
//! Обычный skill request `0x90001` проходит через owned learned skills и
//! emotion state: optional `GS0090`, concrete around `0xBF611`, self/point/
//! object resolution и `0xBFE01` сохраняют native order до внешней очереди
//! ещё не материализованного `CPlayerAI`.
//! Остальная skill family `0x90002..04` сохраняет current-skill End gate,
//! script-data lookup/three path formats и item-skill state перед тем же AI
//! dispatch; concrete `CSkill`, `RunScript` VM и `CPlayerAI` названы отдельными
//! обязательными runtime owners, а не подменены синхронными заглушками.
//! Shape commands `0x8F901..05` подключены к main message route: exact payload
//! lengths, direction/emotion state, `0xBF601/03/502/611/738`, region lookup и
//! effect ordering принадлежат `CGame`; player relocation уже замыкает
//! region/area/block state и `GS0163`; quest route замыкает `BF605/BF738` и
//! destination FIFO, оставляя runtime-у только current action/skill facts и
//! фактическое хранение AI. Non-player relocation и serializers ещё внешние.
//! Friend commands `0x8FA0D..0F` проходят main route через canonical player
//! maps, owned ordered friend state, addressed client wire и WS `0x60501/02`.
//! Обратные WS presence `0x7F904/905` сохраняют исходный payload и доходят до
//! клиента как addressed `0xBF404/405`, замыкая online/offline контракт.
//! Public identity commands связывают appearance around wire, honor snapshot,
//! concrete country job lookup и appellation attempt со script runtime.
//! LeiTing bidirectional route связывает client claim с owned player codec,
//! `0xBF73E/0x5FD10`, WorldServer decode и обратным `0x7FA17`; process report
//! отдельно сохраняет reached other-message результат.
//! Potential allocation `0x8FC2A` теперь тем же dispatcher-ом исполняет каждую
//! ordered notification/property/goods публикацию и безусловный outer
//! `0xBF918`, сохраняя first-key-wins и wrapping `points * 10000` player owner-а.
//! Potential reset `0x8FC2B` продолжает тот же route: расход первого
//! `ZHQLS01` предшествует addon/player rollback, затем идут `0xBF721` и
//! `0xBF918`; packet stack публикуется concrete `0xC0101` при полном удалении
//! либо `0xC0102` с итоговым amount при частичном расходе.
//! Ordinary-fairy lifecycle тем же goods route замыкает hatch, implant и
//! syncretize: state/fragment moves идут адресными `0xC0101/0xC0102`, implant
//! сохраняет `OnChangeProperties` перед расходным packet wire, syncretize —
//! `0xBF704`, а grow/implant/incubate/syncretize публикуют exact World
//! `0x60210` с подтипами `0/2/3/4`. Внешними остаются только tick, old-client
//! codec и недостающие property facts достигнутого player virtual owner-а.
//! Synthesis `0x8FC17..0x8FC1B` продолжает packet/wallet owner: batch-space
//! проверяется exact temporary container simulation, coins и ingredients — в
//! исходной `uint64` арифметике, затем concrete wallet/packet remove/add wire
//! предшествует result, notice и optional World broadcast. Только safe-cell,
//! fight/team state открытия и old-client codec остаются runtime facts.
//! Goods destruction `0x8FC1C/1D` замыкает hand mutation и `0xC0101/0xC0102`,
//! exact LogSystem byte `55`, World `0x60202` с bank/region/IP facts и client
//! result. Только произвольный extend-ID `CPlayer::DeleteGoods` из open-route
//! остаётся polymorphic границей до единого dispatcher-а всех containers.
//! Hotkey `0x8FC08..0A` замыкает все 24 slots и возврат hand consumable:
//! concrete `0xC0101` сохраняет move/rollback/delete, фактическую destination
//! position/identity/amount и self-move normalization; success `0xBF908`
//! предшествует move, а terminal `'.'` отправляется только до hand removal.
//! Script-open upgrade/DaKong/compose создают concrete session/plug, меняют
//! progress/movement, подключают listeners и публикуют `0xBF912/29/2A` с
//! полным send-failure rollback; busy/team notices теперь также адресные.
//! Только запрос team skill-state остаётся внешним state-owner fact.
//! GodsBattle runtime продолжает startup owner: player Add/Remove tail
//! назначает persisted faction и поддерживает region membership, script XYD
//! producer ждёт World echo, а изменившиеся slots публикуют `0xBF80C` только
//! живым игрокам соответствующей faction во всех зарегистрированных regions.
//! GodsBattle death SZL tail сохраняет victim-tier base/revise formulas,
//! team factor с exact x87 truncation, player/appellation effects и region
//! notice. Старый donor округлял team share; EXE `0x44DC3F..0x44DC57` явно
//! переключает x87 на truncation. Missing `%s` argument region notice-а был
//! legacy UB и безопасно заменён буквальным bounded template text.
//! GodsBattle NPC-contend продолжает тот же owner: Add/Remove NPC поддерживает
//! faction sets, guard spawn меняет global monster race после создания,
//! AI-type `0x17` death ведёт per-NPC counter, а manager gate замыкается в
//! contender timer. Успешный timeout меняет faction, публикует World subtype
//! `2`, исполняет `#fengyinNpc` award-script и рассылает region/top-info wire.
//! Имена и форматирование остаются bounded byte/GBK, а подтверждённые legacy
//! cancel/first-faction несоответствия сохранены в concrete region owner-е.
//! GodsBattle return-point caller выбирает DiePos по `(region, faction)`,
//! пишет typed audit и при miss исполняет обычный country/region fallback.
//! Связанный `OnRelive` сохраняет dead/alive split, passive/region/property
//! callback order, полный HP/MP/action reset, in-place и die-back ветви,
//! random destination, ChangeRegion result и `0xBF703/0xBF613/0xBFF2A` wire;
//! ещё универсальные skill/state/spatial owner-ы заданы обязательным runtime
//! context, а не подменены пустым успехом.
//! Один `MainLoop` turn сохраняет static DWORD clocks как owned process state,
//! exact Script→AI→Message→Session→NetSession→Auction order, optional profile
//! reads, refresh/watch gates и wrapping pacing. Ещё не материализованные
//! concrete owner-ы подключаются через обязательный runtime trait; message
//! dispatch уже исполняется живым `CGame`.
//! `Release` сохраняет player-save/catch, city-save, registry/script/factory,
//! network и singleton teardown order; Rust `clear/take/Drop` заменяет manual
//! delete, а ещё внешние process-global owners вызываются typed runtime-ом.
//! Неопределённый security-cookie-derived int не объявляется результатом.
//! `GameThreadFunc` связан с достигнутыми `Init`, повторным `MainLoop` и
//! безусловным `Release`; COM, exit-event и window-close заменены platform
//! callbacks до появления конкретного Linux process runtime.
//! `RunAuction` хранит process-static tick, auction state/time и primary
//! ordered goods owner в `CGame`; exact World `0x80403` caller мутирует
//! его из живого message FIFO, а `MainLoop` публикует `0x60807/0x60808`
//! с исходными strict/wrapping time gates без внешнего auction hook-а.
//! Game-variant `CNetSessionManager` также принадлежит `CGame`: `MainLoop`
//! напрямую выполняет ordered timeout/callback pass, а `Release` очищает
//! его после socket cleanup в исходной lifecycle-позиции.
//! GM silence `0x7FC0B/0x7FC0E` достигает canonical player map из
//! `ProcessMessage`: byte-name lookup, lazy expiry и оба World response-а
//! исполняются до оставшегося внешним GM route owner-а. Адресный `0x7FC0F`
//! там же формирует player system message с local listener IP, а `0x7FC0D`
//! выбирает один из двух player wire и публикует его через `SendAll`.
//! Requester-localized `0x7FC0C` безопасно сохраняет подтверждённый
//! `GS0033/GS0034` `%s/%d/%s` contract и адресный `0xBF806` результат.
//! Country-filtered `0x7FC13` обходит ту же canonical player map в signed
//! ID-order и адресно публикует `0xBF806` каждому совпавшему country byte.
//! Mass-kick `0x7FC09` сохраняет requester, обходит остальные player ID в том
//! же порядке и ставит exact `QuitClientByMapID`; legacy `KickPlayer` при этом
//! всегда возвращает `false` независимо от queue result.
//! Named kick `0x7FC06` использует byte-exact ordered `FindPlayer(char*)`,
//! ставит тот же close side effect и только затем отвечает WorldServer.
//! Around-kick `0x7FC07` связывает name lookup, player father-region и
//! 7×7 `GetShape` scan с consecutive-only `list::unique`, ordered kicks и
//! итоговым World `0x5FD02`; повреждённые pointer/coordinate facts заменены
//! typed block outcome без выдуманного успешного ответа. Общий resolver
//! дополнен owned monster/NPC; любой ещё не owned goods/other registry entry
//! блокирует scan, чтобы не пропустить более ранний non-player `GetShape`.
//! Presence feedback `0x7FC08` сохраняет signed-char outcome, локализует
//! `GS0025/GS0026` с одним byte-string аргументом и отвечает requester-у.
//! Region-kick `0x7FC0A` связывает `FindRegion`, physical row-major area
//! registry и ordered `QuitClientByMapID`, исключая requester без дедупликации.
//! Входящий `0x5FF15` проверяет target player до чтения остатка payload и
//! публикует каждую list-строку отдельным адресным system message.
//! Полная GMA family `0x800xx` теперь входит в реальный FIFO message pass:
//! address kick возвращает World `0x60401`, count — `0x60402`, а unknown
//! сохраняет исходный warning без фиктивного handler success.
//! Depot family `0x7FBxx/0x8FExx` проходит тот же FIFO до generic route:
//! player guards, password mutation, bank/depot locks и адресные
//! `0xBFB07/0xBFB08` выполняются одним owner-ом.
//! Terminal server startup `0x7F801/0x3B` из FIFO действительно поднимает
//! client-facing listener, сохраняет ordered dialog/log effects и только
//! после них читает login/world IDs; другие startup selectors не перехватывает.
//! Language table проходит тот же server dispatcher как startup `0x2F` и
//! runtime `0x7F807`: обе ветви очищают один `CGame` owner, сохраняют partial
//! decode/error, точный cursor и ordered log-effect в process report.
//! Запросы числа игроков `0x7F809/0x7F80B` читают canonical player map и
//! публикуют World responses `0x5FA0A/0x5FA0C`; первая ветвь сохраняет ранний
//! nullable-client guard, вторая доходит до обычной nullable send-семантики.
//! `CMonsterList` хранит monster/drop registries selector-а `0x02`; runtime
//! lookup по original name становится общей базой concrete monster spawn.
//! Script shape removal `3303/3306/3313/3315` публикует `0xBF504` через
//! canonical around runtime до spatial removal либо deferred `CS_DELETE`.
//! Talk pair `3301/3304` публикует actor/name/text `0xBF801`; monster variant
//! дополнительно сохраняет exact-name area scan и строгий distance filter.
//! `s_mapProxyRegion` теперь является owned ordered registry: `AddProxyRegion`
//! `0x0000AD10` сохраняет map-assignment, а `FindProxyRegion` `0x0000AD30` —
//! lookup/null. Proxy snapshot `0x0F` публикуется целиком до startup log.
//! `s_mapRegion` хранит concrete enum всех шести subtype-ов; ID/name lookup,
//! replace assignment, startup monster/NPC totals и GodsBattle registration
//! связаны с selector-ом `0x0E`, не стирая subtype state через base slicing.
//! `with_send_state/register_*/attach_*` являются явной assembly-границей
//! baseline и не снимают их псевдокод. Network setup передаётся отдельной
//! post-`LoadSetup*` проекцией. Windows thread handles заменены owned Tokio
//! tasks с тем же порядком exit/sleep/retry/join. `SO_SNDBUF=0` и World
//! reconnect player snapshot остаются локальными границами; nullable раннюю
//! ветвь `CMessage::SendAll` принимает отдельно как `Option`.
//! Script `2249 / FairyExpUp` связывает reached dispatcher с enhancement
//! shadow, live packet/equipment owner, fairy grow-log, replacement factory,
//! GoodsAI registration и concrete client container wire.
//! Enhancement/precious-box confirm теперь также запускают сохранённый
//! server-trusted path прямо через тот же CScript player/region context.
//! Equipment compose и DaKong announcement paths входят в тот же dispatcher;
//! DaKong на точной позиции вызова временно возвращает извлечённого owned
//! player в canonical map, сохраняя C++ player-pointer context и mutations.
//! PreciousBox script `2221/2222/2237` соединяет trusted повторный action,
//! configuration RNG, goods create/upgrade/packet effects, `0xBF91A..1C` и
//! optional World announcement `0x5FF0E` в одном synchronous owner-е.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::CString;
use std::fmt;
use std::fs;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use rustix::system::uname;

use crate::gameserver::appserver::chbystate::ChangeBodyState;
use crate::gameserver::appserver::container::camountlimitgoodscontainer::AmountLimitGoodsTaken;
use crate::gameserver::appserver::container::cbattlefairycontainer::{
    BattleFairyCell, BattleFairyCombineCheck,
};
use crate::gameserver::appserver::container::ccontainer::PreviousContainer;
use crate::gameserver::appserver::container::cequipmentcomposeshadowcontainer::ComposeEquipmentCell;
use crate::gameserver::appserver::container::cequipmentcontainer::{
    EQUIPMENT_COLUMN_LIMIT, EquipmentAddOutcome, EquipmentRemoveOutcome,
};
use crate::gameserver::appserver::container::cequipmentupgradeshadowcontainer::UpgradeEquipmentCell;
use crate::gameserver::appserver::container::cfairycontainer::{
    FairyContainerAmountChange, FairyContainerGoodsUpdate, FairyContainerMoveOperation,
    FairyHatcherEntry, FairyImplantDelivery, FairyImplantReport, FairyIncubateLog,
    FairyStateChangeEffect, FairyStateChangeOutcome, FairySyncreticProperty, FairySyncretizeConfig,
    FairySyncretizeFragmentEffect, FairySyncretizeLog, FairySyncretizePlayer,
    FairySyncretizePlayerUpdate, FairySyncretizeRemoval, FairySyncretizeReport,
};
use crate::gameserver::appserver::container::cgoodscontainer::GoodsStackMergeOutcome;
use crate::gameserver::appserver::container::cvolumelimitgoodscontainer::{
    VolumeGoodsAddOutcome, VolumeGoodsRemoveOutcome,
};
use crate::gameserver::appserver::country::countryhandler::CCountryHandler;
use crate::gameserver::appserver::country::countryparam::CCountryParam;
use crate::gameserver::appserver::country::countrywarsys::CountryWarSys;
use crate::gameserver::appserver::cs2ccontainerobjectamountchange::CS2CContainerObjectAmountChange;
use crate::gameserver::appserver::cs2ccontainerobjectmove::{
    CS2CContainerObjectMove, ContainerObjectMoveOperation,
};
use crate::gameserver::appserver::exstate::{ExtendedState, ExtendedStateKind};
use crate::gameserver::appserver::goods::cbattlefairyproperty::{
    BattleFairyExpUpResult, BattleFairyPlayerFacts, CBattleFairyProperty,
};
use crate::gameserver::appserver::goods::cgoods::{CGoods, GoodsDecodeError};
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_BF_BATTLE_FAIRY, GAP_BF_BRAVE, GAP_BF_CURRENT_MAX_EXP, GAP_BF_DEFUALT_SKLL, GAP_BF_HP,
    GAP_BF_HUOXIESHU_SKILL, GAP_BF_LEVEL, GAP_BF_LINGZHISHU_SKILL, GAP_BF_MAX_MP, GAP_BF_MODULE,
    GAP_BF_PULLULATERATE, GAP_BF_SKY, GAP_BF_STRENGH, GAP_EQUIP_STATE, GAP_PARTICULAR_ATTRIBUTE,
    GOODS_TYPE_EQUIPMENT,
};
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::gameserver::appserver::goods::fairyproperties::{
    FairyExpRuntime, FairyExpUpResult, FairyGrowLog,
};
use crate::gameserver::appserver::goodswarmember::{
    CGoodsWarMember, GameGoodsWarMessageError, GameGoodsWarMessageReport,
    dispatch_game_goods_war_message,
};
use crate::gameserver::appserver::message::containermessage::{
    AuctionListingTransferBlock, AuctionListingTransferRemoval, AuctionListingTransferReport,
    AuctionListingWithdrawalBlock, AuctionListingWithdrawalOutcome,
    AuctionListingWithdrawalRemoval, AuctionListingWithdrawalReport, EnhancementTransferAddition,
    EnhancementTransferBlock, EnhancementTransferOutcome, EnhancementTransferRemoval,
    EnhancementTransferReport, EquipmentSessionClearBlock, EquipmentSessionSelectionBlock,
    EquipmentSessionSelectionReport, GameContainerMessageError, GameContainerMessageReport,
    PersonalShopClearBlock, PersonalShopSelectionBlock, PersonalShopSelectionReport,
    dispatch_game_container_message, send_enhancement_goods_collected,
    send_enhancement_shadow_deleted,
};
use crate::gameserver::appserver::message::countrymessage::{
    CountryWarMessageDispatchError, GameCountryWarMessageReport, GameCountryWarRuntime,
    dispatch_game_country_war_message,
};
use crate::gameserver::appserver::message::depotmessage::{
    DepotMessageReport, dispatch_depot_message,
};
use crate::gameserver::appserver::message::gmamessage::{
    GmaMessageError, GmaMessageReport, dispatch_gma_message,
};
use crate::gameserver::appserver::message::gmmessage::{
    GmMessageError, GmMessageReport, dispatch_gm_message,
};
use crate::gameserver::appserver::message::goodsmessage::{
    GameGoodsMessageError, GameGoodsMessageReport, GameGoodsMessageRuntime,
    dispatch_game_goods_message,
};
use crate::gameserver::appserver::message::incrementshopmessage::{
    GameIncrementShopMessageError, GameIncrementShopMessageReport, dispatch_increment_shop_message,
};
use crate::gameserver::appserver::message::logmessage::{
    GameLogMessageError, GameLogMessageReport, dispatch_game_log_message,
};
use crate::gameserver::appserver::message::onmsg_c2s_auction::dispatch_client_auction_message;
use crate::gameserver::appserver::message::onmsg_w2s_auction::{
    WorldAuctionMessageError, WorldAuctionMessageReport, WorldAuctionRuntime,
    dispatch_world_auction_message,
};
use crate::gameserver::appserver::message::organsysmessage::{
    GameOrganizingMessageError, GameOrganizingMessageReport, GameOrganizingWarRuntime,
    dispatch_game_organizing_message,
};
use crate::gameserver::appserver::message::othermessage::{
    GameOtherMessageError, GameOtherMessageReport, GameOtherMessageRuntime,
    dispatch_game_other_message,
};
use crate::gameserver::appserver::message::playermessage::{
    GamePlayerMessageError, GamePlayerMessageReport, GamePlayerMessageRuntime,
    dispatch_game_player_message,
};
use crate::gameserver::appserver::message::playershopmessage::{
    PlayerShopMessageError, PlayerShopMessageReport, dispatch_player_shop_message,
};
use crate::gameserver::appserver::message::regionmessage::dispatch_game_region_message;
use crate::gameserver::appserver::message::sequencestring::{
    CSequenceRegistry, SequenceRegistryInitializationError,
};
use crate::gameserver::appserver::message::servermessage::on_billing_client_reconnected;
use crate::gameserver::appserver::message::servermessage::{
    GameRegionChangeResponseContext, GameServerMessageError, GameServerMessageReport,
    InitialRegionStartupContext, WarScheduleSetupContext, dispatch_server_message,
};
use crate::gameserver::appserver::message::shapemessage::{
    GameShapeMessageError, GameShapeMessageReport, GameShapeMessageRuntime,
    dispatch_game_shape_message,
};
use crate::gameserver::appserver::message::shopmessage::{
    ShopMessageError, ShopMessageReport, dispatch_shop_message,
};
use crate::gameserver::appserver::message::skillmessage::{
    GameSkillMessageError, GameSkillMessageReport, GameSkillMessageRuntime,
    dispatch_game_skill_message,
};
use crate::gameserver::appserver::message::unibillmessage::{
    IncrementShopBillingContext, IncrementShopBillingMessageError, IncrementShopBillingReport,
    dispatch_increment_shop_billing_message,
};
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::moveshape::{
    CMoveShape, MoveShapeCommandBlock, MoveShapeCommandContext, MoveShapeResolver, UndeadState,
};
use crate::gameserver::appserver::organizingsystem::attackcitysys::CAttackCitySys;
use crate::gameserver::appserver::organizingsystem::fournationwarsys::{
    CFourNationWarSys, FourNationRect,
};
use crate::gameserver::appserver::organizingsystem::villagewarsys::CVillageWarSys;
use crate::gameserver::appserver::player::{
    AuctionSelfGoodsRefresh, BattleFairyCombineDelivery, BattleFairyCombineEffect,
    BattleFairyCombineReport, BattleFairyDeathReport, BattleFairyEquipmentMutationDelivery,
    BattleFairyEquipmentMutationEffect, BattleFairyEquipmentMutationReport,
    BattleFairyFollowDelivery, BattleFairyFollowEffect, BattleFairyFollowReport,
    BattleFairyObjectMove, BattleFairyObjectMoveOperation, BattleFairyPotentialAllocationDelivery,
    BattleFairyPotentialAllocationEffect, BattleFairyPotentialResetDelivery,
    BattleFairyPotentialResetEffect, BattleFairySkillAdded, BattleFairySkillDispatch,
    BattleFairySkillRequest, BattleFairySkillRequestDelivery, BattleFairySkillRequestEffect,
    BattleFairySkillRequestFacts, BattleFairySkillRequestReport, BattleFairySkillResetDelivery,
    BattleFairySkillResetEffect, BattleFairySkillResetReport, BattleFairySummonDelivery,
    BattleFairySummonEffect, BattleFairySummonReport, BattleFairyUpgradeDelivery,
    BattleFairyUpgradeEffect, BattleFairyWarSoulAction, CPlayer, CiQingContainerAddition,
    CiQingContainerConsumption, CiQingHandConsumption, CiQingPacketAddition,
    CiQingPacketConsumption, EnhancementDeselectionBlock, EnhancementDeselectionReport,
    EnhancementSelectionBlock, EnhancementSelectionReport, GoodsDestroyHandConsumption,
    HotkeyHandTransferOutcome, HotkeyHandTransferReport, PlayerAuctionGoodsReturn,
    PlayerAuctionMoneyChange, PlayerCombatProperties, PlayerEquipmentAddEffect,
    PlayerEquipmentAddReport, PlayerEquipmentAddRuntimeFacts, PlayerEquipmentDelivery,
    PlayerEquipmentRemoveEffect, PlayerEquipmentRemoveReport, PlayerEquipmentRemoveRuntimeFacts,
    PlayerGameSaveCodecError, PlayerGameSaveDecodeReport, PlayerHonorResetReport,
    PlayerLoginGoodsLocation, PlayerProgress, PlayerReliveMutation, PlayerSkillDispatch,
    PlayerSkillRequest, PlayerSkillRequestDelivery, PlayerSkillRequestEffect,
    PlayerSkillRequestFacts, PlayerSkillRequestReport, PlayerUncreatedCarriage, PlayerUncreatedPet,
    PlayerYuanBaoChange,
};
use crate::gameserver::appserver::proxyserverregion::CProxyServerRegion;
use crate::gameserver::appserver::region::{
    RegionCellAccessBlock, RegionRandomContext, RegionReturnPoint,
};
use crate::gameserver::appserver::ridestate::{RIDE_STATE_ID, RideState};
use crate::gameserver::appserver::script::function::ScriptFunctionRuntime;
use crate::gameserver::appserver::script::script::{
    ActiveScript, CScriptFunctionRegistry, ScriptExecutionContext, ScriptLoopReport,
    ScriptStepDisposition,
};
use crate::gameserver::appserver::script::variablelist::{
    CVariableList, GameVariableMutationOutcome, GameVariableSnapshotError,
    GameVariableSnapshotReport,
};
use crate::gameserver::appserver::servercityregion::{
    CServerCityRegion, CityGateRuntimeContext, CityReturnPointContext, CityReturnPointError,
};
use crate::gameserver::appserver::servercountryregion::{
    CServerCountryRegion, CountryBattleStateBlock, CountryReturnPointContext,
    CountryReturnPointError,
};
use crate::gameserver::appserver::servergodsbattleregion::{
    CGodsBattleMgr, CServerGodsBattleRegion, GodsBattleCancelByPlayer, GodsBattleContender,
};
use crate::gameserver::appserver::servernationregion::{
    NationCarriageReturnOutcome, NationContend, NationContendArithmeticBlock,
    NationContendCancelOutcome, NationContendCaptureMutation, NationContendDamageMutation,
    NationMonsterDamageNotice, NationMoraleMutation, ServerNationRegion,
    classify_nation_morale_target,
};
use crate::gameserver::appserver::serverregion::{
    CServerRegion, RegionMembershipBlock, ServerRegionClearPlayerTick, ServerRegionMonsterContext,
    ServerRegionNpcContext, ServerRegionNpcSetup, ServerRegionNpcSpawnBlock,
    ServerRegionNpcSpawnReport, ServerReturnPlayer, ServerReturnSetupBlock,
};
use crate::gameserver::appserver::servervillageregion::CServerVillageRegion;
use crate::gameserver::appserver::session::cequipmentcompose::{
    CEquipmentCompose, COMPOSE_CONSUME_REASON, COMPOSE_CREATE_REASON, COMPOSE_STONE_GOODS_INDEX,
    EquipmentComposeAuditLog, EquipmentComposeOutcome, EquipmentComposeReport,
    EquipmentComposeSourceConsumption, EquipmentComposeSourceRemoval,
    EquipmentComposeSourceSnapshot,
};
use crate::gameserver::appserver::session::cequipmentdakong::{
    CEquipmentDaKong, DA_KONG_USE_SINKER_INDEX, EquipmentDaKongAroundEffect,
    EquipmentDaKongAuditLog, EquipmentDaKongClientUpdate, EquipmentDaKongCloseOutcome,
    EquipmentDaKongCloseReport, EquipmentDaKongEnchaseEvent, EquipmentDaKongExternalRefreshOutcome,
    EquipmentDaKongExternalRefreshReport, EquipmentDaKongGemSnapshot, EquipmentDaKongGoodsSnapshot,
    EquipmentDaKongOperation, EquipmentDaKongOutcome, EquipmentDaKongReport, deal_enchase_gems,
    deal_with_da_kong_external_attributes, deal_with_da_kong_seven, equipment_da_kong_condition,
};
use crate::gameserver::appserver::session::cequipmentupgrade::{
    CEquipmentUpgrade, EQUIPMENT_UPGRADE_FAILURE_LOG_REASON, EQUIPMENT_UPGRADE_LOST_LOG_REASON,
    EQUIPMENT_UPGRADE_SUCCESS_LOG_REASON, EquipmentUpgradeAuditLog, EquipmentUpgradeClientUpdate,
    EquipmentUpgradeCloseOutcome, EquipmentUpgradeCloseReport, EquipmentUpgradeConsumption,
    EquipmentUpgradeConsumptionRemoval, EquipmentUpgradeGoodsSnapshot,
    EquipmentUpgradeLostAuditLog, EquipmentUpgradeOutcome, EquipmentUpgradeReport,
};
use crate::gameserver::appserver::session::csessionfactory::{
    CSessionFactory, EquipmentSessionPlugKind, EquipmentSessionShadowRemoved, SessionEndReport,
    TerminalEquipmentSessionCollected,
};
use crate::gameserver::appserver::session::ctrader::{
    TraderContainerKind, TraderOfferAdded, TraderOfferBlock, TraderOfferRemoved,
};
use crate::gameserver::appserver::shape::{
    CShape, MoveCheckCellRegistry, ShapeCoordinateBlock, ShapeFigure, ShapeIdentity, ShapeResolver,
    ShapeRuntimeFacts, ShapeView,
};
use crate::gameserver::appserver::skills::skillfactory::CSkillFactory;
use crate::gameserver::gameserver::honorranks::CHonorRanks;
use crate::gameserver::gameserver::playerranks::{
    CPlayerRanks, PlayerRanksRequestOutcome, PlayerRanksSerializeError,
};
use crate::nets::clients::ClientConnectError;
use crate::nets::mysocket::legacy_ipv4_word;
use crate::nets::netserver::message::{
    CMessage, GameMessageHandlers, GameServerAroundRuntime, SendMessageError,
};
use crate::nets::netserver::mynetclient::{
    CMyNetClient, GameClientIoError, GameClientIoStep, ServerType,
};
use crate::nets::netserver::mynetserver::{
    CMyNetServer, GameServerEvent, GameServerEventPublisher,
};
use crate::nets::servers::ServerHostError;
use crate::public::aucitionroom::CGameAuctionRoom;
use crate::public::ciqing::{CCiQingSetup, CiQingSerializationBlock};
use crate::public::dakongxiangqian::CDaKongXiangQian;
use crate::public::dupliregionsetup::CDupliRegionSetup;
use crate::public::equipmentcomposelist::EquipmentComposeList;
use crate::public::guid::CGuid;
use crate::public::mystringtable::{
    MyStringTable, MyStringTableDecodeError, MyStringTableDecodeReport,
};
use crate::public::netsessionmanager::{
    CNetSessionManager, NetSessionManagerVariant, NetSessionRunReport,
};
use crate::public::taozhuangsetup::CTaoZhuangSetup;
use crate::public::tools::put_string_to_file;
use crate::public::wordsfilter::CWordsFilter;
use crate::setup::cbattlefairyexpconfig::CBattleFairyExpConfig;
use crate::setup::changebody::CChangeBodyConf;
use crate::setup::contributesetup::CContributeSetup;
use crate::setup::emotion::CEmotion;
use crate::setup::fairyexpconf::CFairyExpConf;
use crate::setup::globesetup::GlobeSetupSnapshot;
use crate::setup::gmlist::CGMList;
use crate::setup::godsbattleconf::{GodsBattleFactionXydUpdate, GodsBattleSzlCalculation};
use crate::setup::goodsdestructionconfig::GoodsDestroySetup;
use crate::setup::hitlevelsetup::CHitLevelSetup;
use crate::setup::honorelimilateconfig::HonorElimilateConfig;
use crate::setup::incrementshoplist::CIncrementShopList;
use crate::setup::leitingsetup::CThingSetup;
use crate::setup::lingbao::CLingBaoSetup;
use crate::setup::logsystem::CLogSystem;
use crate::setup::monsterlist::{
    MonsterDropRegistry, MonsterListDecodeError, MonsterListDecodeReport, MonsterProperties,
    MonsterRegistry, decode_monster_list, get_monster_property_by_origin_name,
    get_monster_property_by_origin_name_mut,
};
use crate::setup::newskillmonsterlist::NewSkillMonsterConf;
use crate::setup::playerlist::CPlayerList;
use crate::setup::preciousboxconf::{PreciousBoxConf, PreciousBoxItem};
use crate::setup::prisonconf::PrisonConf;
use crate::setup::questsystem::CQuestSystem;
use crate::setup::regionrouter::RegionRouter;
use crate::setup::regionsetup::CRegionSetup;
use crate::setup::synthesis::CSynthesis;
use crate::setup::synthesis::SynthesisFormula;
use crate::setup::tradelist::CTradeList;
use crate::transport::bind_tcp_ipv4;

const PLAYER_TYPE: i32 = 400;
const SCRIPT_SCALAR_ERROR: i32 = 0x09ff_fff9;
const NPC_TYPE: i32 = 500;
const MONSTER_TYPE: i32 = 600;
const DEFAULT_SOCKET_TYPE: i32 = 1;
const WORLD_REGISTRATION: i32 = 0x0005_FA01;
const BILLING_REGISTRATION: i32 = 0x000E_F101;
const GAME_RELEASE_PLAYER_SAVE_MESSAGE: i32 = 0x0005_FB02;
const GAME_AUCTION_GOODS_SYNC_MESSAGE: i32 = 0x0006_0807;
const GAME_AUCTION_STATE_REQUEST_MESSAGE: i32 = 0x0006_0808;
const RECONNECT_RETRY_DELAY: Duration = Duration::from_millis(8_000);

#[derive(Clone, Debug, Eq, PartialEq)]
struct GameSetup {
    world_host: Vec<u8>,
    world_port: u32,
    billing_host: Vec<u8>,
    billing_port: u32,
    billing_backup_host: Vec<u8>,
    billing_backup_port: u32,
    billing_bind_ip: Vec<u8>,
    billing_bind_port: Option<u32>,
    listen_port: u32,
    local_ip: Vec<u8>,
    check_network: bool,
    maximum_bytes_per_second: u32,
    maximum_message_length: u32,
    forbid_time_ms: u32,
    check_message_content: bool,
    maximum_clients: i32,
    maximum_in_flight_sends: i32,
    permitted_send_bytes: i32,
    refresh_info_time_ms: u32,
    save_info_time_ms: u32,
    watch_runtime_info: bool,
    watch_runtime_time_ms: u32,
    enter_time: u32,
    message_validate_time_ms: u32,
    sequence_count: u32,
}

impl Default for GameSetup {
    fn default() -> Self {
        Self {
            world_host: b"127.0.0.1".to_vec(),
            world_port: 0x1fa4,
            billing_host: b"127.0.0.1".to_vec(),
            billing_port: 0x1f98,
            billing_backup_host: b"127.0.0.1".to_vec(),
            billing_backup_port: 0x1f98,
            billing_bind_ip: Vec::new(),
            billing_bind_port: None,
            listen_port: 0x092b,
            local_ip: b"127.0.0.1".to_vec(),
            check_network: true,
            maximum_bytes_per_second: 5_000,
            maximum_message_length: 0x1_9000,
            forbid_time_ms: 10,
            check_message_content: true,
            maximum_clients: 500,
            maximum_in_flight_sends: 3,
            permitted_send_bytes: 0xc800,
            refresh_info_time_ms: 1_000,
            save_info_time_ms: 60_000,
            watch_runtime_info: false,
            watch_runtime_time_ms: 60_000,
            enter_time: 5,
            message_validate_time_ms: 0,
            sequence_count: 0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GameSetupEx {
    maximum_block_connections: i32,
    first_receive_timeout_ms: i32,
    maximum_yuan_bao: Option<i32>,
    maximum_money: Option<i32>,
}

impl Default for GameSetupEx {
    fn default() -> Self {
        Self {
            maximum_block_connections: 10,
            first_receive_timeout_ms: 4_000,
            maximum_yuan_bao: None,
            maximum_money: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GameSetupLoadReport {
    pub(crate) parsed_pairs: usize,
    pub(crate) stopped_at_pair: Option<usize>,
}

#[derive(Debug)]
pub(crate) struct GameSetupOpenError {
    pub(crate) path: PathBuf,
    pub(crate) source: io::Error,
}

impl fmt::Display for GameSetupOpenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "не удалось прочитать {}: {}",
            self.path.display(),
            self.source
        )
    }
}

impl std::error::Error for GameSetupOpenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameRuntimePaths {
    pub(crate) setup: PathBuf,
    pub(crate) setup_ex: PathBuf,
}

impl GameRuntimePaths {
    pub(crate) fn from_runtime_directory(directory: impl AsRef<Path>) -> Self {
        let directory = directory.as_ref();
        Self {
            setup: directory.join("setup.ini"),
            setup_ex: directory.join("setupex.ini"),
        }
    }
}

#[derive(Debug)]
pub(crate) enum GameSetupExLoad {
    Loaded(GameSetupLoadReport),
    Unavailable(GameSetupOpenError),
}

#[derive(Debug)]
pub(crate) struct GameRuntimeSetupReport {
    pub(crate) setup: GameSetupLoadReport,
    pub(crate) setup_ex: GameSetupExLoad,
}

#[derive(Debug)]
pub(crate) enum GameRuntimeSetupError {
    Setup(GameSetupOpenError),
    MissingField(&'static str),
}

impl fmt::Display for GameRuntimeSetupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Setup(error) => error.fmt(formatter),
            Self::MissingField(field) => {
                write!(formatter, "GameServer setup не определил поле {field}")
            }
        }
    }
}

impl std::error::Error for GameRuntimeSetupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Setup(error) => Some(error),
            Self::MissingField(_) => None,
        }
    }
}

impl GameSetup {
    fn parse_positional(&mut self, bytes: &[u8]) -> GameSetupLoadReport {
        let mut tokens = GameSetupTokens::new(bytes);

        macro_rules! read_value {
            ($field:ident, $parser:expr) => {{
                let Some(raw) = tokens.next_value() else {
                    return tokens.report();
                };
                let Some(value) = $parser(raw) else {
                    return tokens.report();
                };
                self.$field = value;
                tokens.parsed();
            }};
        }

        macro_rules! read_number {
            ($field:ident, $type:ty) => {
                read_value!($field, |raw| parse_game_setup_number::<$type>(raw));
            };
        }

        macro_rules! read_bool {
            ($field:ident) => {
                read_value!($field, parse_game_setup_bool);
            };
        }

        macro_rules! read_bytes {
            ($field:ident) => {
                read_value!($field, |raw: &[u8]| Some(raw.to_vec()));
            };
        }

        read_bytes!(world_host);
        read_number!(world_port, u32);
        read_bytes!(billing_host);
        read_number!(billing_port, u32);
        read_bytes!(billing_backup_host);
        read_number!(billing_backup_port, u32);
        read_bytes!(billing_bind_ip);
        read_value!(billing_bind_port, |raw| {
            parse_game_setup_number::<u32>(raw).map(Some)
        });
        read_number!(listen_port, u32);
        read_bytes!(local_ip);
        read_bool!(check_network);
        read_number!(maximum_bytes_per_second, u32);
        read_number!(maximum_message_length, u32);
        read_number!(forbid_time_ms, u32);
        read_bool!(check_message_content);
        read_number!(maximum_clients, i32);
        read_number!(maximum_in_flight_sends, i32);
        read_number!(permitted_send_bytes, i32);
        read_number!(refresh_info_time_ms, u32);
        read_number!(save_info_time_ms, u32);
        read_bool!(watch_runtime_info);
        read_number!(watch_runtime_time_ms, u32);
        read_number!(enter_time, u32);
        read_number!(message_validate_time_ms, u32);
        read_number!(sequence_count, u32);
        tokens.report()
    }

    fn network_setup(
        &self,
        setup_ex: &GameSetupEx,
    ) -> Result<GameNetworkSetup, GameRuntimeSetupError> {
        let billing_bind_port = self
            .billing_bind_port
            .ok_or(GameRuntimeSetupError::MissingField("_bind_port_for_bs"))?;
        Ok(GameNetworkSetup::new(
            GameUpstreamEndpoint::new(&self.world_host, self.world_port),
            GameBillingPlan::new(
                GameUpstreamEndpoint::new(&self.billing_host, self.billing_port),
                GameUpstreamEndpoint::new(&self.billing_backup_host, self.billing_backup_port),
                &self.billing_bind_ip,
                billing_bind_port,
            ),
            &self.local_ip,
            GameListenerPlan::new(
                self.listen_port,
                self.check_network,
                self.maximum_bytes_per_second,
                self.forbid_time_ms,
                self.maximum_message_length,
                self.maximum_clients,
                self.maximum_in_flight_sends,
                self.permitted_send_bytes,
                setup_ex.maximum_block_connections,
                setup_ex.first_receive_timeout_ms,
            ),
            self.check_message_content,
        ))
    }
}

impl GameSetupEx {
    fn parse_positional(&mut self, bytes: &[u8]) -> GameSetupLoadReport {
        let mut tokens = GameSetupTokens::new(bytes);

        macro_rules! read_number {
            ($field:ident) => {{
                let Some(raw) = tokens.next_value() else {
                    return tokens.report();
                };
                let Some(value) = parse_game_setup_number::<i32>(raw) else {
                    return tokens.report();
                };
                self.$field = value;
                tokens.parsed();
            }};
        }

        read_number!(maximum_block_connections);
        read_number!(first_receive_timeout_ms);

        let Some(raw) = tokens.next_value() else {
            return tokens.report();
        };
        let Some(value) = parse_game_setup_number::<i32>(raw) else {
            return tokens.report();
        };
        self.maximum_yuan_bao = Some(value);
        tokens.parsed();

        let Some(raw) = tokens.next_value() else {
            return tokens.report();
        };
        let Some(value) = parse_game_setup_number::<i32>(raw) else {
            return tokens.report();
        };
        self.maximum_money = Some(value);
        tokens.parsed();
        tokens.report()
    }
}

struct GameSetupTokens<'a> {
    tokens: Vec<&'a [u8]>,
    next: usize,
    attempted_pairs: usize,
    parsed_pairs: usize,
}

impl<'a> GameSetupTokens<'a> {
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

    fn report(&self) -> GameSetupLoadReport {
        GameSetupLoadReport {
            parsed_pairs: self.parsed_pairs,
            stopped_at_pair: (self.parsed_pairs < self.attempted_pairs)
                .then_some(self.attempted_pairs),
        }
    }
}

fn parse_game_setup_number<T: FromStr>(raw: &[u8]) -> Option<T> {
    std::str::from_utf8(raw).ok()?.parse().ok()
}

fn parse_game_setup_bool(raw: &[u8]) -> Option<bool> {
    match raw {
        b"0" => Some(false),
        b"1" => Some(true),
        _ => None,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameUpstreamEndpoint {
    host: Vec<u8>,
    port: u32,
}

impl GameUpstreamEndpoint {
    pub(crate) fn new(host: &[u8], port: u32) -> Self {
        Self {
            host: host.to_vec(),
            port,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameBillingPlan {
    primary: GameUpstreamEndpoint,
    backup: GameUpstreamEndpoint,
    bind_ip: Vec<u8>,
    bind_port: u32,
}

impl GameBillingPlan {
    pub(crate) fn new(
        primary: GameUpstreamEndpoint,
        backup: GameUpstreamEndpoint,
        bind_ip: &[u8],
        bind_port: u32,
    ) -> Self {
        Self {
            primary,
            backup,
            bind_ip: bind_ip.to_vec(),
            bind_port,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GameListenerPlan {
    listen_port: u32,
    check_network: bool,
    maximum_bytes_per_second: u32,
    forbid_time_ms: u32,
    maximum_message_length: u32,
    maximum_clients: i32,
    maximum_in_flight_sends: i32,
    permitted_send_bytes: i32,
    maximum_block_connections: i32,
    first_receive_timeout_ms: i32,
}

impl GameListenerPlan {
    #[allow(
        clippy::too_many_arguments,
        reason = "план сохраняет десять достигнутых setup-полей Game listener"
    )]
    pub(crate) const fn new(
        listen_port: u32,
        check_network: bool,
        maximum_bytes_per_second: u32,
        forbid_time_ms: u32,
        maximum_message_length: u32,
        maximum_clients: i32,
        maximum_in_flight_sends: i32,
        permitted_send_bytes: i32,
        maximum_block_connections: i32,
        first_receive_timeout_ms: i32,
    ) -> Self {
        Self {
            listen_port,
            check_network,
            maximum_bytes_per_second,
            forbid_time_ms,
            maximum_message_length,
            maximum_clients,
            maximum_in_flight_sends,
            permitted_send_bytes,
            maximum_block_connections,
            first_receive_timeout_ms,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameNetworkSetup {
    world: GameUpstreamEndpoint,
    billing: GameBillingPlan,
    local_ip: Vec<u8>,
    listener: GameListenerPlan,
    check_message_content: bool,
}

impl GameNetworkSetup {
    pub(crate) fn new(
        world: GameUpstreamEndpoint,
        billing: GameBillingPlan,
        local_ip: &[u8],
        listener: GameListenerPlan,
        check_message_content: bool,
    ) -> Self {
        Self {
            world,
            billing,
            local_ip: local_ip.to_vec(),
            listener,
            check_message_content,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameUpstreamDirection {
    World,
    BillingPrimary,
    BillingBackup,
}

#[derive(Debug)]
pub(crate) enum GameClientInitializationFailure {
    MissingNetworkSetup,
    MissingNetworkServerOwner,
    AddressTooLong {
        direction: GameUpstreamDirection,
        length: usize,
    },
    AddressEncodingUnsupported {
        direction: GameUpstreamDirection,
    },
    AddressResolution {
        direction: GameUpstreamDirection,
    },
    BillingBindAddressResolution,
    Bind(io::Error),
    Connect {
        direction: GameUpstreamDirection,
        source: ClientConnectError,
    },
}

#[derive(Debug)]
pub(crate) struct GameConnectAttempt {
    pub(crate) direction: GameUpstreamDirection,
    pub(crate) failure: Option<GameClientInitializationFailure>,
}

#[derive(Debug)]
pub(crate) enum GameClientInitialization {
    Connected {
        endpoint: SocketAddrV4,
        used_billing_backup: bool,
        registration: Result<i32, SendMessageError>,
        attempts: Vec<GameConnectAttempt>,
    },
    Failed {
        attempts: Vec<GameConnectAttempt>,
    },
}

#[derive(Debug)]
pub(crate) enum GameReconnectPublication {
    Published {
        endpoint: SocketAddrV4,
        used_billing_backup: bool,
        attempts: Vec<GameConnectAttempt>,
    },
    Failed {
        attempts: Vec<GameConnectAttempt>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameReconnectWorkerEnd {
    Published,
    ExitRequested,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameReconnectTaskStartError {
    MissingNetworkSetup,
    MissingNetworkServerOwner,
}

struct GameReconnectTask {
    exit_requested: Arc<AtomicBool>,
    handle: tokio::task::JoinHandle<GameReconnectWorkerEnd>,
}

#[derive(Debug)]
pub(crate) enum GameNetworkInitializationError {
    MissingNetworkSetup,
    Host(ServerHostError),
}

#[derive(Debug)]
pub(crate) struct GameInitializationThroughBillingReport {
    pub(crate) setup: GameRuntimeSetupReport,
    pub(crate) world: GameClientInitialization,
    pub(crate) sequence_elements: usize,
    pub(crate) billing: GameClientInitialization,
}

#[derive(Debug)]
pub(crate) struct GameInitializationReport {
    pub(crate) through_billing: GameInitializationThroughBillingReport,
    pub(crate) move_check_cells: usize,
    pub(crate) player_ranks_initialized: bool,
    pub(crate) goods_war_request: Result<i32, SendMessageError>,
}

#[derive(Debug)]
pub(crate) enum GameInitializationThroughBillingError {
    Setup(GameRuntimeSetupError),
    WorldUnavailable {
        setup: GameRuntimeSetupReport,
        connection: GameClientInitialization,
    },
    Sequence(SequenceRegistryInitializationError),
}

impl fmt::Display for GameInitializationThroughBillingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Setup(error) => error.fmt(formatter),
            Self::WorldUnavailable { .. } => {
                formatter.write_str("GameServer не подключился к обязательному WorldServer")
            }
            Self::Sequence(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for GameInitializationThroughBillingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Setup(error) => Some(error),
            Self::WorldUnavailable { .. } => None,
            Self::Sequence(error) => Some(error),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameSingleFilePublication {
    Published,
    RepeatedOwnerFreed,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct MonsterBasePropertyRefreshReport {
    pub(crate) monsters: usize,
    pub(crate) resolved: usize,
    pub(crate) missing: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct RegionBlockRefreshResolver {
    facts: BTreeMap<ShapeIdentity, (ShapeView, bool)>,
}

impl ShapeResolver for RegionBlockRefreshResolver {
    fn resolve_shape(&self, identity: ShapeIdentity) -> Option<ShapeView> {
        self.facts.get(&identity).map(|(shape, _)| *shape)
    }
}

impl MoveShapeResolver for RegionBlockRefreshResolver {
    fn move_shape_is_alive(&self, identity: ShapeIdentity) -> Option<bool> {
        self.facts.get(&identity).map(|(_, is_alive)| *is_alive)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ServerRegionOwner {
    Base(CServerRegion),
    Village(CServerVillageRegion),
    City(CServerCityRegion),
    Country(CServerCountryRegion),
    Nation(ServerNationRegion),
    GodsBattle(CServerGodsBattleRegion),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameWarRegionHandle {
    Local(i32),
    Proxy(i32),
}

pub(crate) struct GameWarStartupOwners {
    pub(crate) attack_city: CAttackCitySys,
    pub(crate) village: CVillageWarSys,
    pub(crate) country: CountryWarSys,
    pub(crate) four_nation: CFourNationWarSys,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GodsBattleTopTenRequestReport {
    pub(crate) player_id: i32,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

pub(crate) trait GodsBattlePlayerContext {
    fn send_gods_battle_player_around(
        &mut self,
        region: &CServerRegion,
        origin: &CShape,
        message: &CMessage,
    ) -> Result<i32, ShapeCoordinateBlock>;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerPropertiesExternalFacts {
    pub(crate) add_element_attack: u32,
    pub(crate) maximum_rp: u32,
    pub(crate) rp: u32,
    pub(crate) maximum_vigour: u32,
    pub(crate) exalt: u32,
}

pub(crate) trait OldClientGoodsCodec {
    fn encode_goods_for_old_client(&mut self, goods: &CGoods) -> Vec<u8>;
}

pub(crate) trait CiQingComposeContext: OldClientGoodsCodec {
    fn clone_ci_qing_hand_goods(&mut self, goods: &CGoods) -> Option<CGoods>;
    fn mount_ci_qing_equipment(
        &mut self,
        game: &mut CGame,
        player_id: i32,
    ) -> CiQingPropertyRuntimeSnapshot;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairyImplantationLog {
    pub(crate) player_id: i32,
    pub(crate) goods: ShapeIdentity,
    pub(crate) goods_name: Vec<u8>,
    pub(crate) old_level: u32,
    pub(crate) resulting_level: u32,
    pub(crate) crystal_amount: u32,
}

pub(crate) trait FairyContext: BattleFairyDeathContext {
    fn current_fairy_tick(&mut self) -> u32;
}

pub(crate) trait PlayerEquipmentInspectionContext: OldClientGoodsCodec {}

impl<T: OldClientGoodsCodec> PlayerEquipmentInspectionContext for T {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerEquipmentInspectionOutcome {
    MissingTarget,
    ModeBlocked,
    Sent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEquipmentInspectionEntry {
    pub(crate) slot: u8,
    pub(crate) goods: ShapeIdentity,
    pub(crate) old_client_payload: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEquipmentInspectionReport {
    pub(crate) requester_id: i32,
    pub(crate) target_id: i32,
    pub(crate) outcome: PlayerEquipmentInspectionOutcome,
    pub(crate) head_picture: Option<i32>,
    pub(crate) face_picture: Option<i32>,
    pub(crate) entries: Vec<PlayerEquipmentInspectionEntry>,
    pub(crate) delivery: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContainerScriptActionOutcome {
    InvalidAction,
    SelectionCleared { shadows: usize },
    EmptyScript,
    Dispatched { script_data_found: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ContainerScriptActionReport {
    pub(crate) action: Option<i8>,
    pub(crate) script_name: Vec<u8>,
    pub(crate) outcome: ContainerScriptActionOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HotkeyAssignmentOutcome {
    InvalidSlot,
    Assigned,
    MissingHandAssigned,
    HandMoveFailed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HotkeyAssignmentReport {
    pub(crate) slot: u8,
    pub(crate) value: u32,
    pub(crate) outcome: HotkeyAssignmentOutcome,
    pub(crate) transfer: Option<HotkeyHandTransferReport>,
    pub(crate) transfer_deliveries: Vec<i32>,
    pub(crate) response_deliveries: Vec<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HotkeyRemovalOutcome {
    InvalidOrEmpty,
    Removed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HotkeyRemovalReport {
    pub(crate) slot: u8,
    pub(crate) outcome: HotkeyRemovalOutcome,
    pub(crate) delivery: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HotkeyChangeOutcome {
    InvalidOrEmpty,
    Changed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HotkeyChangeReport {
    pub(crate) slot: u8,
    pub(crate) value: u32,
    pub(crate) outcome: HotkeyChangeOutcome,
    pub(crate) delivery: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FairyHatchOutcome {
    Disabled,
    InvalidSlot,
    InvalidAction,
    MissingGoods,
    Unchanged,
    Changed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairyHatchReport {
    pub(crate) slot: u32,
    pub(crate) action: i8,
    pub(crate) outcome: FairyHatchOutcome,
    pub(crate) delivery: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairyHatcherRunReport {
    pub(crate) player_id: i32,
    pub(crate) entries: Vec<FairyHatcherEntry>,
    pub(crate) state_effect_deliveries: Vec<Vec<i32>>,
    pub(crate) world_deliveries: Vec<Vec<i32>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FairyImplantOutcome {
    Disabled,
    MissingFairy,
    MaximumLevel,
    InvalidVigour,
    InsufficientVigour,
    InsufficientCrystal,
    PropertyBlocked,
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairyImplantResultReport {
    pub(crate) requested_vigour: u32,
    pub(crate) consumed_vigour: u32,
    pub(crate) crystal_amount: u32,
    pub(crate) outcome: FairyImplantOutcome,
    pub(crate) result_delivery: Option<i32>,
    pub(crate) goods_update_delivery: Option<i32>,
    pub(crate) state_effect_deliveries: Vec<Vec<i32>>,
    pub(crate) packet_consumptions: Vec<CiQingPacketConsumption>,
    pub(crate) packet_deliveries: Vec<Vec<i32>>,
    pub(crate) property_delivery: Option<i32>,
    pub(crate) world_deliveries: Vec<Vec<i32>>,
    pub(crate) implantation: Option<FairyImplantReport>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairySyncretizeResultReport {
    pub(crate) report: FairySyncretizeReport,
    pub(crate) state_effect_deliveries: Vec<Vec<i32>>,
    pub(crate) amount_change_deliveries: Vec<Vec<i32>>,
    pub(crate) money_deliveries: Vec<i32>,
    pub(crate) player_update_deliveries: Vec<i32>,
    pub(crate) world_deliveries: Vec<Vec<i32>>,
    pub(crate) result_delivery: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairySetupQueryReport {
    pub(crate) enabled: bool,
    pub(crate) delivery: Option<i32>,
}

pub(crate) trait EquipmentComposeContext:
    OldClientGoodsCodec + PlayerEquipmentContext
{
    fn equipment_compose_remove_facts(
        &mut self,
        player: &CPlayer,
        goods: &CGoods,
        pack_add_enabled: bool,
    ) -> PlayerEquipmentRemoveRuntimeFacts;
    fn recompute_equipment_compose_player_properties(
        &mut self,
        player: &CPlayer,
    ) -> PlayerCombatProperties;
}

pub(crate) trait EquipmentDaKongContext: OldClientGoodsCodec {}

pub(crate) trait EquipmentUpgradeContext:
    OldClientGoodsCodec + PlayerEquipmentContext
{
    fn equipment_upgrade_remove_facts(
        &mut self,
        player: &CPlayer,
        goods: &CGoods,
        pack_add_enabled: bool,
    ) -> PlayerEquipmentRemoveRuntimeFacts;
    fn recompute_equipment_upgrade_player_properties(
        &mut self,
        player: &CPlayer,
    ) -> PlayerCombatProperties;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentSessionOpenOutcome {
    FeatureDisabled,
    MissingOrDeadPlayer,
    Busy,
    TeamStateBlocked,
    FactoryRejected,
    SendFailed,
    Opened,
}

#[must_use = "open report хранит session/plug identity, player lock и wire result"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentSessionOpenReport {
    pub(crate) kind: EquipmentSessionPlugKind,
    pub(crate) player_id: i32,
    pub(crate) outcome: EquipmentSessionOpenOutcome,
    pub(crate) session_id: Option<i32>,
    pub(crate) plug_id: Option<i32>,
    pub(crate) notification_delivery: Option<i32>,
    pub(crate) player_transition:
        Option<crate::gameserver::appserver::player::GoodsSessionPlayerRelease>,
    pub(crate) listener_attach: Option<[bool; 2]>,
    pub(crate) open_delivery: Option<i32>,
    pub(crate) collected_plug_ids: Vec<i32>,
}

pub(crate) trait EquipmentSessionOpenContext {
    fn equipment_session_has_team_state(&mut self, player_id: i32) -> bool;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoodsDestroyDeleteRequest {
    pub(crate) player_id: i32,
    pub(crate) container_extend_id: i32,
    pub(crate) goods_id: CGuid,
    pub(crate) requested_amount: u32,
    pub(crate) write_delete_log: bool,
}

#[must_use = "общий DeleteGoods report сохраняет mutation и client публикации"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GoodsDestroyDeleteReport {
    pub(crate) removed_amount: u32,
    pub(crate) deliveries: Vec<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GoodsDestroyAuditLog {
    pub(crate) reason: u8,
    pub(crate) player_id: i32,
    pub(crate) pk_count: u16,
    pub(crate) money: u32,
    pub(crate) goods: ShapeIdentity,
    pub(crate) price: u32,
    pub(crate) name: Vec<u8>,
    pub(crate) removed_amount: u32,
    pub(crate) region_id: Option<i32>,
    pub(crate) tile_x: Result<i32, ShapeCoordinateBlock>,
    pub(crate) tile_y: Result<i32, ShapeCoordinateBlock>,
}

pub(crate) trait GoodsDestroyContext {
    /// Вызывает полный polymorphic `CPlayer::DeleteGoods` для произвольного
    /// extend ID. Эта граница обязательна, пока wallet/depot/shadow owners не
    /// сведены в один Rust dispatcher; silent miss возвращает removed `0`.
    fn delete_goods_for_destroy_open(
        &mut self,
        game: &mut CGame,
        request: GoodsDestroyDeleteRequest,
    ) -> GoodsDestroyDeleteReport;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SynthesisOpenFacts {
    pub(crate) safe_region_cell: bool,
    pub(crate) fight_state_count: i32,
    pub(crate) has_team_state: bool,
}

pub(crate) trait SynthesisContext: OldClientGoodsCodec {
    fn synthesis_open_facts(&mut self, game: &CGame, player: &CPlayer) -> SynthesisOpenFacts;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SynthesisOpenOutcome {
    UnsafeRegion = 1,
    Trading = 2,
    Fighting = 3,
    StallOpen = 4,
    TeamState = 5,
    Opened = 6,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerTradeOfferBlock {
    MissingSessionOrPlug,
    OwnerMismatch,
    InvalidExtendId,
    UnsupportedSource,
    MissingSourceGoods,
    SourceMismatch,
    Trader(TraderOfferBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerTradeOfferMutation {
    Added(TraderOfferAdded),
    Removed(TraderOfferRemoved),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerTradeConditionBlock {
    SessionUnavailable,
    MissingPlayerOrPlug,
    MissingOfferedGoods,
    BurdenExceeded,
    PacketSpace,
    InsufficientGold,
    GoldCapacity,
    InsufficientYuanBao,
    YuanBaoCapacity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerTradeBillingRequest {
    pub(crate) payer_id: i32,
    pub(crate) receiver_id: i32,
    pub(crate) amount: u32,
    pub(crate) session_id: i32,
    pub(crate) payer_plug_id: i32,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerTradeReadyOutcome {
    MissingSessionOrPlug,
    SessionUnavailable,
    Changed { ready: bool },
    ConditionBlocked(PlayerTradeConditionBlock),
    BillingPending(PlayerTradeBillingRequest),
    Completed,
    RolledBack,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerTradeReadyReport {
    pub(crate) session_id: i32,
    pub(crate) plug_id: i32,
    pub(crate) contrary_plug_id: Option<i32>,
    pub(crate) ready_deliveries: Vec<i32>,
    pub(crate) notification_deliveries: Vec<i32>,
    pub(crate) packet_deliveries: Vec<Vec<i32>>,
    pub(crate) equipment_removals: Vec<PlayerEquipmentRemoveReport>,
    pub(crate) money_deliveries: Vec<i32>,
    pub(crate) audit_deliveries: Vec<Result<i32, SendMessageError>>,
    pub(crate) session_end: Option<SessionEndReport>,
    pub(crate) terminal_deliveries: Vec<i32>,
    pub(crate) collected_plug_ids: Vec<i32>,
    pub(crate) outcome: PlayerTradeReadyOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerTradeAbortReport {
    pub(crate) session_id: i32,
    pub(crate) plug_id: i32,
    pub(crate) session_abort: Option<SessionEndReport>,
    pub(crate) terminal_deliveries: Vec<i32>,
    pub(crate) collected_plug_ids: Vec<i32>,
}

#[derive(Clone, Debug)]
struct PlayerTradePartySnapshot {
    plug_id: i32,
    owner_id: i32,
    goods: Vec<crate::gameserver::appserver::container::cgoodsshadowcontainer::GoodsShadow>,
    gold: u32,
    yuan_bao: u32,
}

#[derive(Clone, Debug)]
struct DeliveredPlayerTradeGoods {
    source_plug_id: i32,
    receiver_id: i32,
    original: CGoods,
    addition: CiQingPacketAddition,
}

#[derive(Clone, Debug)]
struct PlayerTradeAuditParty {
    owner_id: i32,
    pk_count: u32,
    money: u32,
    tile_x: i32,
    tile_y: i32,
    client_ip: u32,
    name: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SynthesisOpenReport {
    pub(crate) outcome: SynthesisOpenOutcome,
    pub(crate) delivery: i32,
    pub(crate) player_mutation:
        Option<crate::gameserver::appserver::player::GoodsSessionPlayerRelease>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SynthesisComposeOutcome {
    MissingRecipe,
    MissingIngredients,
    InsufficientMoney,
    InsufficientContribution,
    InsufficientPacketSpace,
    RandomFailure,
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SynthesisComposeReport {
    pub(crate) synthesis_index: u32,
    pub(crate) requested_amount: u32,
    pub(crate) result_amount: u32,
    pub(crate) outcome: SynthesisComposeOutcome,
    pub(crate) result_delivery: Option<i32>,
    pub(crate) notice_delivery: Option<i32>,
    pub(crate) money_delivery: Vec<i32>,
    pub(crate) consumptions: Vec<CiQingPacketConsumption>,
    pub(crate) consumption_deliveries: Vec<Vec<i32>>,
    pub(crate) additions: Vec<CiQingPacketAddition>,
    pub(crate) addition_deliveries: Vec<Vec<i32>>,
    pub(crate) rejected_goods: Vec<ShapeIdentity>,
    pub(crate) broadcast_delivery: Option<Result<i32, SendMessageError>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsDestroyOpenOutcome {
    DeleteRequested,
    ConfigurationEnabled,
    ConfigurationDisabled,
}

#[must_use = "open report сохраняет decode, delete либо configuration wire"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GoodsDestroyOpenReport {
    pub(crate) player_id: i32,
    pub(crate) container_extend_id: i32,
    pub(crate) goods_id: Option<CGuid>,
    pub(crate) requested_amount: u32,
    pub(crate) outcome: GoodsDestroyOpenOutcome,
    pub(crate) deletion: Option<GoodsDestroyDeleteReport>,
    pub(crate) notice_delivery: Option<i32>,
    pub(crate) response_delivery: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsDestroyConfirmOutcome {
    ConfigurationDisabled,
    MissingHandGoods,
    MissingBaseProperties,
    OriginalNameRestricted,
    GoodsTypeRejected,
    EquipmentStateRestricted,
    Destroyed,
}

#[must_use = "confirm report сохраняет guards, container mutation, audit и result wire"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GoodsDestroyConfirmReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: GoodsDestroyConfirmOutcome,
    pub(crate) goods: Option<ShapeIdentity>,
    pub(crate) type_key: Option<u16>,
    pub(crate) equipment_state: Option<i32>,
    pub(crate) requested_amount: u32,
    pub(crate) removed_amount: u32,
    pub(crate) consumption: Option<GoodsDestroyHandConsumption>,
    pub(crate) consumption_deliveries: Vec<i32>,
    pub(crate) notice_delivery: Option<i32>,
    pub(crate) audit: Option<GoodsDestroyAuditLog>,
    pub(crate) audit_dispatched: bool,
    pub(crate) world_deliveries: Vec<i32>,
    pub(crate) result_delivery: Option<i32>,
}

pub(crate) trait BattleFairyDeathContext: OldClientGoodsCodec {
    fn player_properties_external_facts(&mut self, player_id: i32)
    -> PlayerPropertiesExternalFacts;
}

/// Exact virtual `CPlayer::UpdateProperty` после realm hidden-skill mutation.
/// Runtime владеет ещё не сведёнными equipment/state/GlobeSetup источниками;
/// CGame применяет возвращённый полный snapshot и сам публикует `0xBF721`.
pub(crate) trait RealmAppellationScriptContext: BattleFairyDeathContext {
    fn recompute_realm_appellation_player_properties(
        &mut self,
        player: &CPlayer,
    ) -> PlayerCombatProperties;

    /// `AddExState` type `0x12F` сначала снимает concrete
    /// `SKILL_GOD_BLESS`. Общий state registry ещё остаётся у runtime, но
    /// вызов идёт из реального script owner-а до replacement нового state.
    fn remove_script_god_bless_state(&mut self, player_id: i32);
}

pub(crate) trait BattleFairySkillResetContext {
    fn publish_battle_fairy_skill_reset_packet_consumption(
        &mut self,
        effect: &BattleFairySkillResetEffect,
    ) -> Vec<i32>;
}

/// Нематериализованные virtual owners полного player GameSave, pet/carriage
/// snapshot и skill-state остаются на runtime-границе. Region/player maps,
/// session state и message wire исполняет непосредственно `CGame`.
pub(crate) trait ScriptRegionChangeContext: NationCombatContext {
    fn finish_script_player_business(&mut self, player: &mut CPlayer);

    fn prepare_script_region_companions(
        &mut self,
        player: &mut CPlayer,
        source_region_id: i32,
        target_region_id: i32,
        target_tile_x: i32,
        target_tile_y: i32,
        carriage_distance: i32,
        changing_server: bool,
    );

    fn refresh_script_region_auto_protect(&mut self, player: &mut CPlayer);

    /// Live pet/carriage являются region/monster owner-ами; runtime возвращает
    /// точный snapshot, после чего общий Rust codec пишет весь player wire.
    fn snapshot_script_player_summons(
        &mut self,
        player: &CPlayer,
    ) -> (Vec<PlayerUncreatedPet>, PlayerUncreatedCarriage, bool);
}

pub(crate) trait GameRegionEnterContext: NationCombatContext {
    /// Remaining `8F801` serialization/weather/state tail after concrete
    /// destination membership has been established by `CGame`.
    fn publish_changed_player_region_entry(
        &mut self,
        game: &mut CGame,
        player_id: i32,
        region_id: i32,
        entry_token: i32,
        socket_id: i32,
    );
}

/// Внешняя половина initial-login owner-а: player codec, map, spatial
/// membership, login/honor scripts и Billing account entry исполняет `CGame`.
/// Ещё не материализованные полный client snapshot, virtual property recompute
/// и GoodsAI tree получают тот же live runtime в исходном порядке.
pub(crate) trait GamePlayerLoginContext: NationCombatContext + OldClientGoodsCodec {
    fn recompute_login_player_properties(&mut self, player: &CPlayer) -> PlayerCombatProperties;
    fn publish_initial_player_client_snapshot(
        &mut self,
        game: &mut CGame,
        player_id: i32,
        first_login: bool,
    ) -> Vec<i32>;
    /// Переводит packed local `tm` equipment-state в исходное условие
    /// `difftime(now, expiry) / 60 > 10079`. Time-zone/CRT conversion остаётся
    /// системной runtime-границей; gameplay mutation и wire принадлежат CGame.
    fn login_equipment_state_expired(&mut self, packed_local_time: i32) -> bool;
    fn register_login_goods_ai(&mut self, player_id: i32, goods: &CGoods);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerLoginBlock {
    PlayerIdMismatch { expected: i32, decoded: i32 },
    AlreadyRegistered { player_id: i32 },
    MissingRegion { region_id: i32 },
    Membership(RegionMembershipBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GamePlayerLoginReport {
    pub(crate) player_id: i32,
    pub(crate) team_id: i32,
    pub(crate) captain: bool,
    pub(crate) region_id: i32,
    pub(crate) first_login: bool,
    pub(crate) relocation: Option<(i32, i32)>,
    pub(crate) login_script_id: Option<i32>,
    pub(crate) client_deliveries: Vec<i32>,
    pub(crate) billing_delivery: i32,
    pub(crate) honor_script_id: Option<i32>,
    pub(crate) goods_ai_registrations: usize,
    pub(crate) equipment_state_updates: Vec<PlayerLoginEquipmentStateUpdate>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerLoginEquipmentStateUpdate {
    pub(crate) location: PlayerLoginGoodsLocation,
    pub(crate) goods: ShapeIdentity,
    pub(crate) delivery: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameRegionEnterReport {
    pub(crate) player_id: i32,
    pub(crate) region_id: i32,
    pub(crate) entry_token: i32,
    pub(crate) previous_changing_region: bool,
    pub(crate) relocation: Option<(i32, i32, i32)>,
    pub(crate) membership: Result<(), RegionMembershipBlock>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScriptRegionChangeKind {
    MissingPlayer,
    MissingSourceRegion,
    SameRegion,
    LocalRegion,
    RemoteServer,
}

#[must_use = "report сохраняет script destination, lifecycle и network result"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScriptRegionChangeReport {
    pub(crate) player_id: i32,
    pub(crate) source_region_id: Option<i32>,
    pub(crate) target_region_id: i32,
    pub(crate) tile_x: i32,
    pub(crate) tile_y: i32,
    pub(crate) direction: i32,
    pub(crate) use_goods: i32,
    pub(crate) range: i32,
    pub(crate) carriage_distance: i32,
    pub(crate) kind: ScriptRegionChangeKind,
    pub(crate) position_delivery: Option<Result<i32, ShapeCoordinateBlock>>,
    pub(crate) direction_delivery: Option<Result<i32, ShapeCoordinateBlock>>,
    pub(crate) region_delivery: Option<Result<i32, ShapeCoordinateBlock>>,
    pub(crate) faction_delivery: Option<Result<i32, SendMessageError>>,
    pub(crate) team_delivery: Option<Result<i32, SendMessageError>>,
    pub(crate) world_delivery: Option<Result<i32, SendMessageError>>,
    pub(crate) player_snapshot_size: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyScriptAction {
    AddSkill {
        player_name: Vec<u8>,
        skill_name: Vec<u8>,
        skill_level: i32,
        position: Option<i32>,
    },
    GetFetchPower {
        player_name: Vec<u8>,
    },
    SetAttribute {
        player_name: Vec<u8>,
        attribute: i32,
        value: i32,
    },
    ResetSkill {
        player_name: Vec<u8>,
        position: i32,
    },
    Revive {
        player_name: Vec<u8>,
    },
    GetSkillValue {
        player_name: Vec<u8>,
        position: i32,
        value_id: u32,
    },
    AddExperience {
        player_name: Vec<u8>,
        experience: i32,
    },
    GetAttribute {
        player_name: Vec<u8>,
        attribute: i32,
    },
    RecreateAttributes {
        player_name: Vec<u8>,
        mode: i32,
        minimum: i32,
        maximum: i32,
    },
}

pub(crate) trait BattleFairySkillRequestContext {
    fn queue_battle_fairy_skill(&mut self, player_id: i32, dispatch: BattleFairySkillDispatch);
}

pub(crate) trait PlayerSkillRequestContext {
    fn queue_player_skill(&mut self, player_id: i32, dispatch: PlayerSkillDispatch);
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyScriptSkillAttachReport {
    pub(crate) skills: Vec<BattleFairySkillAdded>,
    pub(crate) deliveries: Vec<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingGoodsPreview {
    pub(crate) base_index: u32,
    pub(crate) old_client_payload: Vec<u8>,
    pub(crate) delivery: i32,
}

#[must_use = "CiQing goods query хранит ранний preview-send tail"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingGoodsQueryReport {
    pub(crate) player_id: i32,
    pub(crate) previews: Vec<CiQingGoodsPreview>,
}

#[must_use = "CiQing setup query хранит serialization и player delivery"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingSetupQueryReport {
    pub(crate) player_id: i32,
    pub(crate) payload: Result<Vec<u8>, CiQingSerializationBlock>,
    pub(crate) delivery: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CiQingLog {
    pub(crate) player_id: i32,
    pub(crate) delta: i32,
    pub(crate) operation: u32,
    pub(crate) base_index: u32,
    pub(crate) amount: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CiQingMakeOutcome {
    UnknownNode,
    AmountOutOfRange,
    MissingRecipe,
    InsufficientSourceA,
    InsufficientSourceB,
    InsufficientPacketSpace,
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CiQingMakeDelivery {
    Audit(Vec<i32>),
    Consumption(Vec<i32>),
    Addition(Vec<i32>),
    Result(i32),
}

#[must_use = "CiQing make report хранит ресурсы, ownership, аудит и ответ"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingMakeReport {
    pub(crate) player_id: i32,
    pub(crate) requested_base_index: u32,
    pub(crate) requested_amount: u32,
    pub(crate) result_base_index: u32,
    pub(crate) outcome: CiQingMakeOutcome,
    pub(crate) logs: Vec<CiQingLog>,
    pub(crate) consumptions: Vec<CiQingPacketConsumption>,
    pub(crate) additions: Vec<CiQingPacketAddition>,
    pub(crate) rejected_goods: Vec<ShapeIdentity>,
    pub(crate) deliveries: Vec<CiQingMakeDelivery>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CiQingComposeOutcome {
    MissingSources,
    ResultSlotOccupied,
    InsufficientPayment,
    Failed,
    Succeeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CiQingComposeDelivery {
    Audit(Vec<i32>),
    Player(i32),
    Money(Vec<i32>),
    PacketConsumption(Vec<i32>),
    ContainerAddition(Vec<i32>),
    ContainerConsumption(Vec<i32>),
    GoodsQuery(CiQingGoodsQueryReport),
}

#[must_use = "CiQing compose report хранит payment, RNG, container и unlock tail"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingComposeReport {
    pub(crate) player_id: i32,
    pub(crate) source_indices: Option<(u32, u32)>,
    pub(crate) recipe_found: bool,
    pub(crate) roll: Option<u32>,
    pub(crate) selected_result: Option<u32>,
    pub(crate) outcome: CiQingComposeOutcome,
    pub(crate) logs: Vec<CiQingLog>,
    pub(crate) crystal_consumptions: Vec<CiQingPacketConsumption>,
    pub(crate) source_consumptions: Vec<CiQingContainerConsumption>,
    pub(crate) result_addition: Option<CiQingContainerAddition>,
    pub(crate) rejected_result: Option<ShapeIdentity>,
    pub(crate) deliveries: Vec<CiQingComposeDelivery>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CiQingDeleteOutcome {
    MissingGoodsOrResetItem,
    Deleted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CiQingDeleteDelivery {
    Audit(Vec<i32>),
    Player(i32),
    PacketConsumption(Vec<i32>),
    ContainerConsumption(Vec<i32>),
    PropertyUpdate(Vec<i32>),
}

#[must_use = "CiQing delete report хранит reset item, goods и property tail"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingDeleteReport {
    pub(crate) player_id: i32,
    pub(crate) position: u32,
    pub(crate) outcome: CiQingDeleteOutcome,
    pub(crate) logs: Vec<CiQingLog>,
    pub(crate) reset_consumptions: Vec<CiQingPacketConsumption>,
    pub(crate) goods_consumption: Option<CiQingContainerConsumption>,
    pub(crate) deliveries: Vec<CiQingDeleteDelivery>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CiQingMountOutcome {
    Rejected,
    Failed,
    Mounted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CiQingMountDelivery {
    Audit(Vec<i32>),
    HandConsumption(Vec<i32>),
    PacketConsumption(Vec<i32>),
    ContainerAddition(Vec<i32>),
    PropertyUpdate(Vec<i32>),
    Player(i32),
}

#[must_use = "CiQing mount report хранит hand clone, payment, RNG и result"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingMountReport {
    pub(crate) player_id: i32,
    pub(crate) amount: u32,
    pub(crate) position: Option<u32>,
    pub(crate) chance: Option<u32>,
    pub(crate) roll: Option<u32>,
    pub(crate) outcome: CiQingMountOutcome,
    pub(crate) logs: Vec<CiQingLog>,
    pub(crate) hand_consumption: Option<CiQingHandConsumption>,
    pub(crate) material_consumptions: Vec<CiQingPacketConsumption>,
    pub(crate) addition: Option<CiQingContainerAddition>,
    pub(crate) rejected_clone: Option<ShapeIdentity>,
    pub(crate) deliveries: Vec<CiQingMountDelivery>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CiQingPropertyRuntimeSnapshot {
    pub(crate) previous_type_values: BTreeMap<u32, u32>,
    pub(crate) current_type_values: BTreeMap<u32, u32>,
    pub(crate) tao_zhuang_add_values: BTreeMap<u32, u32>,
    pub(crate) tao_zhuang_id: u32,
    pub(crate) external_deliveries: Vec<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CiQingOtherPersonTarget {
    Id(i32),
    Name(Vec<u8>),
    UnsupportedMode(i8),
}

#[must_use = "other-person report хранит target resolution, payload и delivery"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingOtherPersonReport {
    pub(crate) requester_id: i32,
    pub(crate) target: CiQingOtherPersonTarget,
    pub(crate) target_player_id: Option<i32>,
    pub(crate) payload: Vec<u8>,
    pub(crate) delivery: Option<i32>,
}

pub(crate) trait PlayerEquipmentContext {
    fn publish_player_equipment_add_effect(
        &mut self,
        effect: &PlayerEquipmentAddEffect,
    ) -> Vec<i32>;
    fn publish_player_equipment_remove_effect(
        &mut self,
        effect: &PlayerEquipmentRemoveEffect,
    ) -> Vec<i32>;
}

/// Container transfer использует уже материализованные player/equipment
/// owners. Exact `CanMountEquip` теперь читает persisted player flags и goods
/// прямо у canonical owner-а; базовый property recompute, clock и GoodsAI
/// registration остаются обязательными runtime facts. RideState overlay и
/// personal-shop mount gate также принадлежат canonical player owner-у.
pub(crate) trait GameContainerMessageRuntime:
    OldClientGoodsCodec + PlayerEquipmentContext
{
    fn enhancement_equipment_remove_facts(
        &mut self,
        player: &CPlayer,
        goods: &CGoods,
        pack_add_enabled: bool,
    ) -> PlayerEquipmentRemoveRuntimeFacts;

    fn enhancement_equipment_add_facts(
        &mut self,
        player: &CPlayer,
        goods: &CGoods,
        pack_add_enabled: bool,
    ) -> PlayerEquipmentAddRuntimeFacts;

    fn recompute_enhancement_player_properties(
        &mut self,
        player: &CPlayer,
    ) -> PlayerCombatProperties;

    fn register_enhancement_goods_ai(&mut self, goods: &CGoods);
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GodsBattleTeamSnapshot {
    pub(crate) teammate_amount: u32,
    pub(crate) player_ids: Vec<i32>,
}

pub(crate) trait GodsBattleDeathContext {
    fn gods_battle_team_snapshot(&mut self, team_id: i32) -> Option<GodsBattleTeamSnapshot>;
    fn request_gods_battle_change_appellation(&mut self, player_id: i32, appellation_id: u32);
}

pub(crate) trait GodsBattleNpcContendContext:
    ServerRegionMonsterContext + GodsBattlePlayerContext + ScriptFunctionRuntime
{
    fn run_gods_battle_base_region_ai(&mut self, region: &mut CServerRegion);
    fn gods_battle_now_milliseconds(&mut self) -> u32;
    fn record_gods_battle_log(&mut self, event: GodsBattleNpcLog);
}

pub(crate) trait GodsBattleReturnPointContext {
    fn record_gods_battle_log(&mut self, event: GodsBattleNpcLog);
}

pub(crate) trait PlayerReliveContext:
    GodsBattleReturnPointContext + RegionRandomContext
{
    fn auto_start_player_passive_skills(&mut self, player: &mut CPlayer);
    fn clear_player_uncreated_summons(&mut self, player: &mut CPlayer);
    fn player_enter_region_after_relive(&mut self, player: &mut CPlayer);
    fn update_player_property_after_relive(&mut self, player: &mut CPlayer);
    fn set_player_moveable(&mut self, player: &mut CPlayer, moveable: bool);
    fn change_player_states_after_relive(&mut self, player: &mut CPlayer);
    fn enter_player_resident_state(&mut self, player: &mut CPlayer);
    fn enter_player_peace_state(&mut self, player: &mut CPlayer);
    fn send_player_relive_around(
        &mut self,
        region: &CServerRegion,
        origin: &CShape,
        message: &CMessage,
    ) -> Result<i32, ShapeCoordinateBlock>;
    fn change_relived_player_region(
        &mut self,
        player_id: i32,
        region_id: i32,
        x: i32,
        y: i32,
        direction: i32,
        reason: i32,
    ) -> bool;
    fn restore_relive_origin_block(&mut self, player_id: i32, x: i32, y: i32);
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleNpcLog {
    InvalidMonsterToken {
        token: Vec<u8>,
    },
    MonsterSpawnFailed {
        npc_name: Vec<u8>,
        monster: Vec<u8>,
    },
    MonsterSpawned {
        npc_name: Vec<u8>,
        monster: Vec<u8>,
        faction: i32,
    },
    FactionUpdated {
        npc_name: Vec<u8>,
        faction: i32,
    },
    MonsterWithoutNpc {
        monster: Vec<u8>,
    },
    MissingKillCounter {
        npc_name: Vec<u8>,
    },
    MonsterKilled {
        npc_name: Vec<u8>,
        monster: Vec<u8>,
        killer_type: i32,
        killer_id: i32,
    },
    NoConfiguredMonsters {
        npc_name: Vec<u8>,
    },
    KillCountExceeded {
        npc_name: Vec<u8>,
    },
    AwardVariableMissing {
        player_id: i32,
        npc_name: Vec<u8>,
    },
    AwardScriptFailed {
        player_id: i32,
        npc_name: Vec<u8>,
    },
    NpcCaptured {
        player_id: i32,
        npc_name: Vec<u8>,
        faction: i32,
    },
    ReturnPointSelected {
        player_id: i32,
        faction: i32,
        point: RegionReturnPoint,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleReturnPointSource {
    GodsBattleConfiguration,
    BaseRegionFallback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GodsBattleReturnPointReport {
    pub(crate) player_id: i32,
    pub(crate) faction: i32,
    pub(crate) point: RegionReturnPoint,
    pub(crate) source: GodsBattleReturnPointSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerReliveOutcome {
    PlayerMissing,
    AlreadyAlive {
        answer_delivery: i32,
    },
    PositionBlocked(ShapeCoordinateBlock),
    CurrentRegionMissing {
        mutation: PlayerReliveMutation,
    },
    InPlace {
        mutation: PlayerReliveMutation,
        answer_delivery: i32,
        shape_delivery: Option<Result<i32, ShapeCoordinateBlock>>,
        state_deliveries: Vec<Result<i32, ShapeCoordinateBlock>>,
    },
    ReturnPointBlocked(ServerReturnSetupBlock),
    RandomPositionBlocked(RegionCellAccessBlock),
    ReturnPoint {
        mutation: PlayerReliveMutation,
        return_point: GodsBattleReturnPointReport,
        x: i32,
        y: i32,
        changed_region: bool,
        answer_delivery: Option<i32>,
        state_deliveries: Vec<Result<i32, ShapeCoordinateBlock>>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerReliveReport {
    pub(crate) player_id: i32,
    pub(crate) relive_type: i32,
    pub(crate) outcome: PlayerReliveOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleContendEnterOutcome {
    PlayerMissing,
    NpcMissing,
    PlayerUnavailable,
    AlreadyContending,
    InvalidPlayerFaction,
    InvalidNpcFaction,
    GuardsRemain { remaining: u32 },
    AlreadyOwned,
    Entered { first_for_legacy_faction: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GodsBattleContendEnterReport {
    pub(crate) region_id: i32,
    pub(crate) player_id: i32,
    pub(crate) npc_id: i32,
    pub(crate) outcome: GodsBattleContendEnterOutcome,
    pub(crate) deliveries: Vec<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleContendCompletionOutcome {
    PlayerMissing,
    FactionChangeRejected,
    Captured {
        faction: i32,
        award_variable_result: i32,
        award_script_result: Option<i32>,
        top_info_delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GodsBattleContendCompletionReport {
    pub(crate) contender: GodsBattleContender,
    pub(crate) outcome: GodsBattleContendCompletionOutcome,
    pub(crate) deliveries: Vec<i32>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct GodsBattleContendAiReport {
    pub(crate) region_id: i32,
    pub(crate) progress_deliveries: Vec<(i32, i32, i32)>,
    pub(crate) completions: Vec<GodsBattleContendCompletionReport>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleMonsterTokenBlock {
    FieldCount { token: Vec<u8>, fields: usize },
    MissingMonsterProperty { original_name: Vec<u8> },
    Membership(RegionMembershipBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GodsBattleNpcFactionReport {
    pub(crate) region_id: i32,
    pub(crate) npc_id: i32,
    pub(crate) npc_name: Vec<u8>,
    pub(crate) faction: i32,
    pub(crate) spawned_monster_ids: Vec<i32>,
    pub(crate) spawn_blocks: Vec<GodsBattleMonsterTokenBlock>,
    pub(crate) world_delivery: Option<Result<i32, SendMessageError>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GodsBattleMonsterDeathReport {
    pub(crate) region_id: i32,
    pub(crate) monster_id: i32,
    pub(crate) npc_name: Option<Vec<u8>>,
    pub(crate) killed: Option<u32>,
    pub(crate) total: Option<u32>,
    pub(crate) player_notice_delivery: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GodsBattlePlayerRegionReport {
    pub(crate) region_id: i32,
    pub(crate) player_id: i32,
    pub(crate) assigned_faction: Option<i32>,
    pub(crate) faction_delivery: Option<Result<i32, ShapeCoordinateBlock>>,
    pub(crate) membership_changed: bool,
    pub(crate) region_state_delivery: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GodsBattleXydRequestReport {
    pub(crate) faction: i32,
    pub(crate) xyd: u32,
    pub(crate) delivery: Option<Result<i32, SendMessageError>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GodsBattleXydApplyReport {
    pub(crate) updates: [GodsBattleFactionXydUpdate; 2],
    pub(crate) deliveries: Vec<(i32, i32, i32)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GodsBattleSzlPlayerUpdate {
    pub(crate) player_id: i32,
    pub(crate) previous: u32,
    pub(crate) current: u32,
    pub(crate) property_delivery: i32,
    pub(crate) notice_delivery: i32,
    pub(crate) removed_attempt_appellation: Option<u32>,
    pub(crate) appellation_notice_delivery: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GodsBattleDeathSzlReport {
    pub(crate) killer_id: i32,
    pub(crate) victim_id: i32,
    pub(crate) gain: GodsBattleSzlCalculation,
    pub(crate) loss: GodsBattleSzlCalculation,
    pub(crate) team: Option<GodsBattleTeamSnapshot>,
    pub(crate) updates: Vec<GodsBattleSzlPlayerUpdate>,
    pub(crate) region_notice_delivery: Option<i32>,
}

pub(crate) trait NationCombatContext: ServerRegionNpcContext {
    fn now_milliseconds(&mut self) -> u32;
    fn add_log_text(&mut self, text: &[u8]);
    fn put_debug_string(&mut self, text: &[u8]);

    /// Выполняет concrete player-origin `CMessage::SendToAround`, включая
    /// соседние areas и удалённых team members исходного runtime-а.
    fn send_nation_player_around(
        &mut self,
        region: &CServerRegion,
        origin: &CShape,
        excluded_player_id: Option<i32>,
        message: &CMessage,
    ) -> Result<i32, ShapeCoordinateBlock>;
}

pub(crate) trait NationContendContext:
    NationCombatContext + ServerRegionMonsterContext
{
    fn run_base_region_ai(&mut self, region: &mut CServerRegion);

    /// Выполняет concrete `CMessage::SendToAround` для двух magic-stone
    /// сообщений до virtual удаления исходного NPC.
    fn send_nation_magic_stone_around(
        &mut self,
        region: &CServerRegion,
        origin: &CShape,
        message: &CMessage,
    ) -> i32;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NationMonsterDamageOutcome {
    AttackerMissing,
    AttackerUnavailable,
    MonsterMissing,
    MonsterUnavailable,
    MonsterPropertyMissing,
    CountryOutsideNation,
    SameCountry,
    NoFirstHitNotice,
    Notice(NationMonsterDamageNotice),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationMonsterDamageReport {
    pub(crate) region_id: i32,
    pub(crate) monster_id: i32,
    pub(crate) attacker_player_id: i32,
    pub(crate) outcome: NationMonsterDamageOutcome,
    pub(crate) world_delivery: Option<Result<i32, SendMessageError>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationCarriageReturnReport {
    pub(crate) region_id: i32,
    pub(crate) requested_country: i32,
    pub(crate) outcome: NationCarriageReturnOutcome,
    pub(crate) morale_delivery: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NationContendEnterOutcome {
    PlayerMissing,
    PlayerUnavailable,
    CountryAlreadyOwnsSymbol,
    CountryContenderExists { contender_player_id: i32 },
    Entered { first_for_country: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationContendEnterReport {
    pub(crate) region_id: i32,
    pub(crate) player_id: i32,
    pub(crate) outcome: NationContendEnterOutcome,
    pub(crate) deliveries: Vec<i32>,
    pub(crate) state_deliveries: Vec<Result<i32, ShapeCoordinateBlock>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationContendCancelReport {
    pub(crate) region_id: i32,
    pub(crate) player_id: i32,
    pub(crate) outcome: Option<NationContendCancelOutcome>,
    pub(crate) delivery: Option<i32>,
    pub(crate) state_delivery: Option<Result<i32, ShapeCoordinateBlock>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationContendCancelAllReport {
    pub(crate) time_deliveries: Vec<i32>,
    pub(crate) state_deliveries: Vec<(i32, Result<i32, ShapeCoordinateBlock>)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationContendDamageReport {
    pub(crate) region_id: i32,
    pub(crate) player_id: i32,
    pub(crate) mutation: Result<Option<NationContendDamageMutation>, NationContendArithmeticBlock>,
    pub(crate) delivery: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NationContendCompletionOutcome {
    PlayerMissing,
    PlayerUnavailable,
    CountryOutsideNation,
    Captured(NationContendCaptureMutation),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NationMagicStoneTransitionOutcome {
    NpcMissing,
    NpcNameAmbiguous { matches: usize },
    NpcCoordinateBlocked(ShapeCoordinateBlock),
    NpcRemovalBlocked(RegionMembershipBlock),
    MonsterPropertyMissing,
    MonsterSpawnBlocked(RegionMembershipBlock),
    Spawned { monster_id: i32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationMagicStoneTransitionReport {
    pub(crate) country: u8,
    pub(crate) npc_id: Option<i32>,
    pub(crate) explosion_delivery: Option<i32>,
    pub(crate) removal_delivery: Option<i32>,
    pub(crate) outcome: NationMagicStoneTransitionOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationContendAiReport {
    pub(crate) region_id: i32,
    pub(crate) magic_stone_transitions: Vec<NationMagicStoneTransitionReport>,
    pub(crate) progress_deliveries: Vec<(i32, i32, i32)>,
    pub(crate) completed: Option<NationContend>,
    pub(crate) completion_outcome: Option<NationContendCompletionOutcome>,
    pub(crate) completion_deliveries: Vec<i32>,
    pub(crate) state_deliveries: Vec<(i32, Result<i32, ShapeCoordinateBlock>)>,
    pub(crate) top_info_delivery: Option<Result<i32, SendMessageError>>,
    pub(crate) treasure_spawns: Vec<Result<ServerRegionNpcSpawnReport, ServerRegionNpcSpawnBlock>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NationPlayerDeathContendOutcome {
    NotContending,
    MissingReset,
    RemovedLegacyReturnIndeterminate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationPlayerDeathReport {
    pub(crate) region_id: i32,
    pub(crate) player_id: i32,
    pub(crate) timing_finished: bool,
    pub(crate) contend_outcome: NationPlayerDeathContendOutcome,
    pub(crate) contend_state_delivery: Option<Result<i32, ShapeCoordinateBlock>>,
    pub(crate) contend_time_delivery: Option<i32>,
    pub(crate) notice_delivery: Option<i32>,
    pub(crate) died_state_time_ms: i32,
    pub(crate) died_state_start_time_ms: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationPlayerDiedStatePublication {
    pub(crate) player_id: i32,
    pub(crate) state: bool,
    pub(crate) self_delivery: i32,
    pub(crate) around_delivery: Result<i32, ShapeCoordinateBlock>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NationPlayerDiedStateTick {
    Inactive,
    Waiting {
        elapsed_ms: u32,
    },
    Advanced {
        elapsed_ms: u32,
        remaining_ms: i32,
        time_delivery: Option<i32>,
    },
    Expired {
        elapsed_ms: u32,
        time_delivery: Option<i32>,
        state_publication: NationPlayerDiedStatePublication,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NationMonsterDeathOutcome {
    KillerMissing,
    MonsterMissing,
    MonsterPropertyMissing,
    Unclassified,
    CountryOutsideNation,
    MoraleChanged(NationMoraleMutation),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationMonsterDeathReport {
    pub(crate) region_id: i32,
    pub(crate) monster_id: i32,
    pub(crate) killer_player_id: i32,
    pub(crate) outcome: NationMonsterDeathOutcome,
    pub(crate) morale_delivery: Option<i32>,
    pub(crate) first_guard_delivery: Option<i32>,
    pub(crate) nation_fail_deliveries: Vec<Result<i32, SendMessageError>>,
    pub(crate) yu_ying_shi_spawns: Vec<NationYuYingShiSpawnReport>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationYuYingShiSpawnReport {
    pub(crate) country: u8,
    pub(crate) notify_country: u8,
    pub(crate) spawn: Result<ServerRegionNpcSpawnReport, ServerRegionNpcSpawnBlock>,
    pub(crate) region_delivery: Option<i32>,
    pub(crate) country_deliveries: Vec<i32>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GameMainLoopProfile {
    pub(crate) script_ms: u32,
    pub(crate) ai_ms: u32,
    pub(crate) message_ms: u32,
    pub(crate) session_ms: u32,
    pub(crate) net_session_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameMainLoopRuntimeLog {
    Compact {
        elapsed_ms: u32,
        ai_calls: i32,
    },
    Profiled {
        elapsed_ms: u32,
        ai_calls: i32,
        profile: GameMainLoopProfile,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameMainLoopStage {
    RefreshInfo,
    RuntimeLog,
    Script,
    Ai,
    FairyHatcher,
    Message,
    Session,
    NetSession,
    Auction,
    Wait { duration_ms: u32 },
    LagWarning { resync_tick_ms: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameMainLoopOutcome {
    Continue,
    ExitRequested,
}

#[must_use = "MainLoop report сохраняет ordering, pacing и legacy return"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameMainLoopReport<RegionRuntimeError> {
    pub(crate) outcome: GameMainLoopOutcome,
    pub(crate) return_value: i32,
    pub(crate) sampled_tick_ms: u32,
    pub(crate) ai_tick: i32,
    pub(crate) stages: Vec<GameMainLoopStage>,
    pub(crate) next_deadline_ms: Option<u32>,
    pub(crate) signed_lag_ms: Option<i32>,
    pub(crate) ai: Option<GameAiReport>,
    pub(crate) messages: Option<GameProcessMessagesReport<RegionRuntimeError>>,
    pub(crate) net_sessions: Option<NetSessionRunReport>,
    pub(crate) auction: Option<GameAuctionRunReport>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameReturnPointBlock {
    PlayerMissing,
    Base(ServerReturnSetupBlock),
    City(CityReturnPointError),
    Country(CountryReturnPointError),
    GodsBattleMissing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameReturnedRegionPlayer {
    pub(crate) player_id: i32,
    pub(crate) point: Result<RegionReturnPoint, GameReturnPointBlock>,
    pub(crate) destination: Option<(i32, i32)>,
    pub(crate) random_block: Option<RegionCellAccessBlock>,
    pub(crate) changed_region: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameRegionClearPlayerOutcome {
    Waiting {
        remaining_ms: i32,
        elapsed_ms: u32,
    },
    Warning {
        remaining_ms: i32,
        seconds: i32,
        delivery: i32,
    },
    Expired {
        players: Vec<GameReturnedRegionPlayer>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameRegionAiReport {
    pub(crate) region_id: i32,
    pub(crate) gods_battle: Option<GodsBattleContendAiReport>,
    pub(crate) region_changes: Vec<GameLocalRegionChange>,
    pub(crate) clear_player: Option<GameRegionClearPlayerOutcome>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameLocalRegionChange {
    pub(crate) player_id: i32,
    pub(crate) destination: (i32, i32, i32, i32),
    pub(crate) removal: Result<(), RegionMembershipBlock>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameAiReport {
    pub(crate) legacy_return: i32,
    pub(crate) regions: Vec<GameRegionAiReport>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GameRegionClearStarted {
    pub(crate) region_id: i32,
    pub(crate) buffer_seconds: i32,
    pub(crate) delay_ms: i32,
    pub(crate) previous_active: bool,
    pub(crate) previous_remaining_ms: i32,
    pub(crate) started_at_ms: u32,
}

struct CityReturnPointFacts {
    player_id: i32,
    tile_x: i32,
    tile_y: i32,
}

impl CityReturnPointContext for CityReturnPointFacts {
    fn read_city_player_tile_y(&mut self, player_id: i32) -> i32 {
        (player_id == self.player_id)
            .then_some(self.tile_y)
            .unwrap_or(0)
    }

    fn read_city_player_tile_x(&mut self, player_id: i32) -> i32 {
        (player_id == self.player_id)
            .then_some(self.tile_x)
            .unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameAuctionRunOutcome {
    FeatureDisabled,
    TickNotDue { elapsed_ms: u32 },
    Processed { state_expired: bool },
}

#[must_use = "RunAuction report сохраняет time gates и оба World effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameAuctionRunReport {
    pub(crate) outcome: GameAuctionRunOutcome,
    pub(crate) sampled_tick_ms: Option<u32>,
    pub(crate) sampled_wall_time_seconds: Option<u32>,
    pub(crate) synchronized_goods: Vec<CGuid>,
    pub(crate) goods_sync: Option<Result<i32, SendMessageError>>,
    pub(crate) state_request: Option<Result<i32, SendMessageError>>,
}

#[must_use = "recollection сохраняет signed player order и каждый World send"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PersonalShopRecollection {
    pub(crate) player_id: i32,
    pub(crate) client_ip: u32,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PersonalShopPurchaseOutcome {
    MissingSessionOrPlug,
    ShopClosed,
    MissingGoods,
    UnsupportedPriceType { price_type: u32 },
    InsufficientMoney { notice_delivery: i32 },
    PacketFull { notice_delivery: i32 },
    BurdenExceeded { notice_delivery: i32 },
    SellerMoneyCapacity { notice_delivery: i32 },
    SourceRemovalFailed,
    BuyerAddFailed,
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PersonalShopPurchaseReport {
    pub(crate) session_id: i32,
    pub(crate) buyer_plug_id: i32,
    pub(crate) seller_plug_id: Option<i32>,
    pub(crate) buyer_id: Option<i32>,
    pub(crate) seller_id: Option<i32>,
    pub(crate) goods_id: CGuid,
    pub(crate) price: Option<u32>,
    pub(crate) seller_delete_delivery: Option<i32>,
    pub(crate) buyer_packet_deliveries: Vec<Vec<i32>>,
    pub(crate) money_deliveries: Vec<i32>,
    pub(crate) audit_delivery: Option<Result<i32, SendMessageError>>,
    pub(crate) outcome: PersonalShopPurchaseOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PersonalShopTerminalReport {
    pub(crate) session_id: i32,
    pub(crate) seller_plug_id: Option<i32>,
    pub(crate) seller_id: Option<i32>,
    pub(crate) buyer_plug_ids: Vec<i32>,
    pub(crate) buyer_ids: Vec<i32>,
    pub(crate) deliveries: Vec<i32>,
    pub(crate) around_delivery: Option<Result<i32, ShapeCoordinateBlock>>,
    pub(crate) collected_plug_ids: Vec<i32>,
}

#[must_use = "ProcessMessage report сохраняет server и достигнутые gameplay effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameProcessMessagesReport<RegionRuntimeError> {
    pub(crate) legacy_return: i32,
    pub(crate) auction_messages: Vec<Result<WorldAuctionMessageReport, WorldAuctionMessageError>>,
    pub(crate) player_shop_messages: Vec<Result<PlayerShopMessageReport, PlayerShopMessageError>>,
    pub(crate) shop_messages: Vec<Result<ShopMessageReport, ShopMessageError>>,
    pub(crate) gm_messages: Vec<Result<GmMessageReport, GmMessageError>>,
    pub(crate) gma_messages: Vec<Result<GmaMessageReport, GmaMessageError>>,
    pub(crate) depot_messages: Vec<DepotMessageReport>,
    pub(crate) increment_shop_messages:
        Vec<Result<GameIncrementShopMessageReport, GameIncrementShopMessageError>>,
    pub(crate) increment_shop_billing_messages:
        Vec<Result<IncrementShopBillingReport, IncrementShopBillingMessageError>>,
    pub(crate) organizing_messages:
        Vec<Result<GameOrganizingMessageReport, GameOrganizingMessageError>>,
    pub(crate) country_war_messages: Vec<
        Result<
            GameCountryWarMessageReport,
            CountryWarMessageDispatchError<CountryBattleStateBlock>,
        >,
    >,
    pub(crate) goods_war_messages: Vec<Result<GameGoodsWarMessageReport, GameGoodsWarMessageError>>,
    pub(crate) container_messages:
        Vec<Result<GameContainerMessageReport, GameContainerMessageError>>,
    pub(crate) goods_messages: Vec<Result<GameGoodsMessageReport, GameGoodsMessageError>>,
    pub(crate) skill_messages: Vec<Result<GameSkillMessageReport, GameSkillMessageError>>,
    pub(crate) shape_messages: Vec<Result<GameShapeMessageReport, GameShapeMessageError>>,
    pub(crate) other_messages: Vec<Result<GameOtherMessageReport, GameOtherMessageError>>,
    pub(crate) player_messages: Vec<Result<GamePlayerMessageReport, GamePlayerMessageError>>,
    pub(crate) log_messages: Vec<Result<GameLogMessageReport, GameLogMessageError>>,
    pub(crate) server_messages:
        Vec<Result<GameServerMessageReport, GameServerMessageError<RegionRuntimeError>>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GameKickPlayerReport {
    pub(crate) player_id: i32,
    pub(crate) command_result: i32,
    pub(crate) legacy_return: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameKickAroundOutcome {
    TargetMissing,
    ServerRegionMissing,
    CoordinateBlocked(ShapeCoordinateBlock),
    RegionMissing,
    UnresolvedShape(ShapeIdentity),
    LookupBlocked(RegionMembershipBlock),
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameKickAroundReport {
    pub(crate) outcome: GameKickAroundOutcome,
    pub(crate) target_player_id: Option<i32>,
    pub(crate) server_region_id: Option<i32>,
    pub(crate) window_origin: Option<(i32, i32)>,
    pub(crate) matched_player_ids: Vec<i32>,
    pub(crate) kicks: Vec<GameKickPlayerReport>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct GameMainLoopState {
    initialized: bool,
    current_tick_ms: u32,
    runtime_log_tick_ms: u32,
    refresh_info_tick_ms: u32,
    calls_since_runtime_log: i32,
    ai_tick: i32,
    profile: GameMainLoopProfile,
    pacing_initialized: bool,
    pacing_deadline_ms: u32,
}

/// Concrete Script/region/AI/session owners подключаются сюда по мере их
/// материализации; message routing уже исполняется самим `CGame`, а region
/// decoder получает тот же live factory-контекст без отдельного shadow state.
pub(crate) trait GameMainLoopRuntime:
    GameMessageHandlers
    + InitialRegionStartupContext
    + GameRegionChangeResponseContext
    + GameRegionEnterContext
    + GamePlayerLoginContext
    + GameOrganizingWarRuntime
    + GameCountryWarRuntime
    + GameContainerMessageRuntime
    + GameGoodsMessageRuntime
    + GameSkillMessageRuntime
    + GameShapeMessageRuntime
    + GameOtherMessageRuntime
    + GamePlayerMessageRuntime
    + IncrementShopBillingContext
    + WorldAuctionRuntime
    + CountryReturnPointContext
    + GodsBattleNpcContendContext
{
    fn exit_requested(&self) -> bool;
    fn tick_interval_ms(&self) -> u32;
    fn get_tick_ms(&mut self) -> u32;
    fn wall_time_seconds(&mut self) -> u32;
    fn refresh_info_text(&mut self, game: &CGame);
    fn add_runtime_log(&mut self, log: GameMainLoopRuntimeLog);
    /// Выполняет virtual region AI до точного base-tail `ClearPlayerAI`.
    fn region_ai_before_clear_player(&mut self, game: &mut CGame, region_id: i32);
    fn wait(&mut self, duration_ms: u32);
    fn output_debug(&mut self, message: &'static str);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameReleaseExternalOwner {
    SocketRuntime,
    PkSystem,
    BaseMessageRuntime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameReleaseDebug {
    ServerExiting,
    PlayerSaveFailed {
        player_id: i32,
        processed: usize,
        total: usize,
    },
    PlayersSaved {
        processed: usize,
        total: usize,
    },
    CityRegionSaved,
    PlayersAndRegionsCleared,
    ProxyRegionsCleared,
    ScriptDataCleared,
    ServerExited,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameReleaseEvent {
    Debug(GameReleaseDebug),
    ReconnectTasksStopped,
    PlayerSave {
        player_id: i32,
        saved: bool,
    },
    CityRegionSaved,
    PlayersCleared {
        count: usize,
    },
    RegionsCleared {
        count: usize,
    },
    ProxyRegionsCleared {
        count: usize,
    },
    ScriptDataCleared {
        function_list: bool,
        variable_list: bool,
        files: usize,
        active_scripts: usize,
        function_registry: usize,
        general_variables: usize,
    },
    ExternalOwner(GameReleaseExternalOwner),
    SkillFactoryCleared,
    GoodsFactoryReleased,
    NetworkServerWorkerStopped {
        present: bool,
    },
    WorldClientReleased {
        present: bool,
    },
    BillingClientReleased {
        present: bool,
    },
    NetworkServerReleased {
        present: bool,
    },
    NetSessionsReleased {
        count: usize,
    },
    IncrementShopReleased,
    QuestSystemReleased,
    SequenceRegistryCleared {
        count: usize,
    },
    PlayerRanksReleased {
        present: bool,
    },
    WordsFilterReleased,
    HonorRanksReleased,
    GoodsWarReleased {
        present: bool,
    },
    CountryHandlerReleased,
    CountryParamReleased,
    AttackCitySystemReleased {
        schedules: usize,
    },
    VillageWarSystemReleased {
        schedules: usize,
    },
}

#[must_use = "Release report сохраняет полный достигнутый teardown ordering"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameReleaseReport {
    pub(crate) events: Vec<GameReleaseEvent>,
    /// `Release` объявлен как int, но достигнутый tail возвращает результат
    /// security-cookie thunk; gameplay caller значение игнорирует.
    pub(crate) legacy_return: Option<i32>,
}

pub(crate) trait GameReleaseRuntime {
    fn put_debug_string(&mut self, message: GameReleaseDebug);
    fn publish_player_save(
        &mut self,
        player_id: i32,
        message_type: i32,
        save_flag: i32,
        snapshot: &[u8],
    ) -> bool;
    fn save_city_region(&mut self, game: &CGame, region_id: i32);
    fn release_external_owner(&mut self, owner: GameReleaseExternalOwner);
    fn exit_network_server_worker(&mut self, server: &mut CMyNetServer);
}

pub(crate) trait GameThreadRuntime: GameMainLoopRuntime + GameReleaseRuntime {
    fn runtime_paths(&self) -> GameRuntimePaths;
    fn sequence_seed_ms(&mut self) -> u32;
    fn initialize_com(&mut self);
    fn signal_game_thread_exit(&mut self);
    fn post_process_close(&mut self);
    fn uninitialize_com(&mut self);
}

#[must_use = "GameThread report сохраняет Init/MainLoop/Release lifecycle"]
#[derive(Debug)]
pub(crate) struct GameThreadReport {
    pub(crate) initialization:
        Result<GameInitializationReport, GameInitializationThroughBillingError>,
    pub(crate) main_loop_calls: usize,
    pub(crate) release: GameReleaseReport,
}

impl ServerRegionOwner {
    pub(crate) const fn base(&self) -> &CServerRegion {
        match self {
            Self::Base(region) => region,
            Self::Village(region) => &region.war.base,
            Self::City(region) => &region.war.base,
            Self::Country(region) => &region.base,
            Self::Nation(region) => &region.war.base,
            Self::GodsBattle(region) => &region.war.base,
        }
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CServerRegion {
        match self {
            Self::Base(region) => region,
            Self::Village(region) => &mut region.war.base,
            Self::City(region) => &mut region.war.base,
            Self::Country(region) => &mut region.base,
            Self::Nation(region) => &mut region.war.base,
            Self::GodsBattle(region) => &mut region.war.base,
        }
    }

    pub(crate) const fn region_id(&self) -> i32 {
        self.base().id
    }

    pub(crate) fn name(&self) -> &[u8] {
        self.base().region.get_name()
    }

    pub(crate) const fn is_gods_battle(&self) -> bool {
        matches!(self, Self::GodsBattle(_))
    }

    /// Сохраняет virtual dispatch: war-derived owners дополнительно сбрасывают
    /// symbol state, base/country используют `CServerRegion` реализацию.
    pub(crate) fn reset_war_state(&mut self, war_number: i32, state: i32) {
        match self {
            Self::Base(region) => region.reset_war_state(war_number, state),
            Self::Village(region) => region.war.reset_war_state(war_number, state),
            Self::City(region) => region.war.reset_war_state(war_number, state),
            Self::Country(region) => region.base.reset_war_state(war_number, state),
            Self::Nation(region) => region.war.reset_war_state(war_number, state),
            Self::GodsBattle(region) => region.war.reset_war_state(war_number, state),
        }
    }
}

impl WarScheduleSetupContext for CGame {
    type Region = GameWarRegionHandle;

    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region> {
        if self.find_region(region_id).is_some() {
            Some(GameWarRegionHandle::Local(region_id))
        } else {
            self.find_proxy_region(region_id)
                .map(|_| GameWarRegionHandle::Proxy(region_id))
        }
    }

    fn reset_war_state(&mut self, region: Self::Region, war_number: i32, state: i32) {
        match region {
            GameWarRegionHandle::Local(region_id) => {
                if let Some(region) = self.find_region_mut(region_id) {
                    region.reset_war_state(war_number, state);
                }
            }
            GameWarRegionHandle::Proxy(region_id) => {
                if let Some(region) = self.find_proxy_region_mut(region_id) {
                    region.reset_war_state(war_number, state);
                }
            }
        }
    }

    fn region_country(&self, region: Self::Region) -> u8 {
        match region {
            GameWarRegionHandle::Local(region_id) => self
                .find_region(region_id)
                .map(|region| region.base().country)
                .unwrap_or(0),
            GameWarRegionHandle::Proxy(region_id) => self
                .find_proxy_region(region_id)
                .map(CProxyServerRegion::country)
                .unwrap_or(0),
        }
    }

    fn set_region_country(&mut self, region: Self::Region, country: u8) {
        match region {
            GameWarRegionHandle::Local(region_id) => {
                if let Some(region) = self.find_region_mut(region_id) {
                    region.base_mut().country = country;
                }
            }
            GameWarRegionHandle::Proxy(region_id) => {
                if let Some(region) = self.find_proxy_region_mut(region_id) {
                    region.set_country(country);
                }
            }
        }
    }

    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region> {
        matches!(
            self.find_region(region_id),
            Some(ServerRegionOwner::Country(_))
        )
        .then_some(GameWarRegionHandle::Local(region_id))
    }

    fn find_nation_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region> {
        matches!(
            self.find_region(region_id),
            Some(ServerRegionOwner::Nation(_))
        )
        .then_some(GameWarRegionHandle::Local(region_id))
    }

    fn reset_nation_war_state(&mut self, region: Self::Region, index: i32, state: i32) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        if let Some(ServerRegionOwner::Nation(region)) = self.find_region_mut(region_id) {
            region.war.reset_war_state(index, state);
        }
    }

    fn set_nation_relive_rects(&mut self, region: Self::Region, rects: [FourNationRect; 5]) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        if let Some(ServerRegionOwner::Nation(region)) = self.find_region_mut(region_id) {
            region.set_relive_rects(rects);
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PendingFactionCreation {
    pub(crate) session_id: i64,
    pub(crate) password: i32,
    pub(crate) player_id: i32,
    pub(crate) required_goods: Vec<u8>,
    pub(crate) required_money: i32,
    pub(crate) country: u8,
    pub(crate) world_request_sent: bool,
    pub(crate) expires_at_ms: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PendingFactionApplication {
    pub(crate) session_id: i64,
    pub(crate) password: i32,
    pub(crate) player_id: i32,
    pub(crate) page: i32,
    pub(crate) expires_at_ms: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PendingFactionWarDeclaration {
    pub(crate) session_id: i64,
    pub(crate) password: i32,
    pub(crate) player_id: i32,
    pub(crate) page: i32,
    pub(crate) declaration_pending: bool,
    pub(crate) expires_at_ms: u32,
}

pub(crate) struct CGame {
    setup: GameSetup,
    setup_ex: GameSetupEx,
    random_state: u32,
    sequence_registry: CSequenceRegistry,
    player_list: CPlayerList,
    trade_list: CTradeList,
    goods_factory: CGoodsFactory,
    skill_factory: CSkillFactory,
    monster_registry: MonsterRegistry,
    monster_drop_registry: MonsterDropRegistry,
    thing_setup: CThingSetup,
    increment_shop_list: CIncrementShopList,
    contribute_setup: CContributeSetup,
    log_system: CLogSystem,
    gm_list: CGMList,
    da_kong_xiang_qian: CDaKongXiangQian,
    globe_setup: GlobeSetupSnapshot,
    region_router: RegionRouter,
    area_width: i32,
    area_height: i32,
    auction_room: CGameAuctionRoom,
    auction_now: bool,
    auction_last_check_seconds: u32,
    auction_tick_ms: u32,
    function_list_file_data: Option<Vec<u8>>,
    variable_list_file_data: Option<Vec<u8>>,
    script_file_data: BTreeMap<Vec<u8>, Vec<u8>>,
    script_functions: CScriptFunctionRegistry,
    general_variables: CVariableList,
    active_scripts: BTreeMap<i32, ActiveScript>,
    next_script_id: i32,
    next_organizing_session_id: i64,
    next_organizing_password: i32,
    pending_faction_creations: BTreeMap<i64, PendingFactionCreation>,
    pending_faction_applications: BTreeMap<i64, PendingFactionApplication>,
    pending_faction_war_declarations: BTreeMap<i64, PendingFactionWarDeclaration>,
    string_table: MyStringTable,
    quest_system: CQuestSystem,
    country_param: CCountryParam,
    country_handler: CCountryHandler,
    attack_city_sys: CAttackCitySys,
    village_war_sys: CVillageWarSys,
    country_war_sys: CountryWarSys,
    four_nation_war_sys: CFourNationWarSys,
    emotion: CEmotion,
    region_setup: CRegionSetup,
    hit_level_setup: CHitLevelSetup,
    prison_conf: PrisonConf,
    precious_box_conf: PreciousBoxConf,
    fairy_exp_conf: CFairyExpConf,
    battle_fairy_exp_config: CBattleFairyExpConfig,
    battle_fairy_property: CBattleFairyProperty,
    equipment_compose_list: EquipmentComposeList,
    words_filter: CWordsFilter,
    jjc_level_data: BTreeMap<i32, i32>,
    tao_zhuang_setup: CTaoZhuangSetup,
    ci_qing_setup: CCiQingSetup,
    ling_bao_setup: CLingBaoSetup,
    gods_battle_mgr: CGodsBattleMgr,
    synthesis: CSynthesis,
    new_skill_monster_conf: NewSkillMonsterConf,
    goods_destroy_setup: GoodsDestroySetup,
    change_body_conf: CChangeBodyConf,
    honor_eliminate_config: HonorElimilateConfig,
    dupli_region_setup: Option<CDupliRegionSetup>,
    move_check_cells: MoveCheckCellRegistry,
    player_ranks: Option<CPlayerRanks>,
    honor_ranks: CHonorRanks,
    goods_war: Option<CGoodsWarMember>,
    id_index: u8,
    team_id_counter: u32,
    login_server_id: i32,
    world_server_id: i32,
    network_setup: Option<GameNetworkSetup>,
    world_client: Option<CMyNetClient>,
    billing_client: Option<CMyNetClient>,
    net_server: Option<CMyNetServer>,
    world_reconnect_task: Option<GameReconnectTask>,
    billing_reconnect_task: Option<GameReconnectTask>,
    net_session_manager: CNetSessionManager,
    session_factory: CSessionFactory,
    players: BTreeMap<i32, CPlayer>,
    regions: BTreeMap<i32, ServerRegionOwner>,
    proxy_regions: BTreeMap<i32, CProxyServerRegion>,
    initial_total_monsters: i32,
    initial_total_npcs: i32,
    team_session_ids: BTreeMap<u32, i32>,
    main_loop_state: GameMainLoopState,
}

impl CGame {
    pub(crate) fn personal_shop_session_available<Context: GameContainerMessageRuntime>(
        &self,
        session_id: i32,
        _context: &mut Context,
    ) -> bool {
        if !self
            .session_factory
            .personal_shop_session_available(session_id)
        {
            return false;
        }
        let Some(seller_plug_id) = self
            .session_factory
            .personal_shop_seller_plug_id(session_id)
        else {
            return false;
        };
        let Some(player) = self
            .session_factory
            .query_plug(seller_plug_id)
            .and_then(|plug| self.players.get(&plug.owner_id()))
        else {
            return false;
        };
        if player.is_dead() || player.is_rider() {
            return false;
        }
        let Some(region_id) = player.server_region_id() else {
            return false;
        };
        let Some(region) = self.find_region(region_id) else {
            return false;
        };
        let (Ok(tile_x), Ok(tile_y)) = (player.shape().get_tile_x(), player.shape().get_tile_y())
        else {
            return false;
        };
        region
            .base()
            .region
            .get_block(tile_x, tile_y)
            .is_ok_and(|block| block == 2)
    }

    pub(crate) fn purchase_personal_shop_goods<Context: GameContainerMessageRuntime>(
        &mut self,
        session_id: i32,
        buyer_plug_id: i32,
        goods_id: CGuid,
        context: &mut Context,
    ) -> PersonalShopPurchaseReport {
        self.transact_personal_shop_goods(session_id, buyer_plug_id, goods_id, false, context)
    }

    pub(crate) fn complete_personal_shop_billing_goods<Context: GameContainerMessageRuntime>(
        &mut self,
        session_id: i32,
        buyer_plug_id: i32,
        goods_id: CGuid,
        context: &mut Context,
    ) -> PersonalShopPurchaseReport {
        self.transact_personal_shop_goods(session_id, buyer_plug_id, goods_id, true, context)
    }

    fn transact_personal_shop_goods<Context: GameContainerMessageRuntime>(
        &mut self,
        session_id: i32,
        buyer_plug_id: i32,
        goods_id: CGuid,
        billing_completion: bool,
        context: &mut Context,
    ) -> PersonalShopPurchaseReport {
        let mut report = PersonalShopPurchaseReport {
            session_id,
            buyer_plug_id,
            seller_plug_id: None,
            buyer_id: None,
            seller_id: None,
            goods_id,
            price: None,
            seller_delete_delivery: None,
            buyer_packet_deliveries: Vec::new(),
            money_deliveries: Vec::new(),
            audit_delivery: None,
            outcome: PersonalShopPurchaseOutcome::MissingSessionOrPlug,
        };
        if !self.personal_shop_session_available(session_id, context) {
            return report;
        }
        let Some(buyer) = self
            .session_factory
            .personal_shop_buyer(buyer_plug_id)
            .filter(|buyer| buyer.session_id() == session_id)
            .copied()
        else {
            return report;
        };
        let Some(seller_plug_id) = self
            .session_factory
            .personal_shop_seller_plug_id(session_id)
        else {
            return report;
        };
        let Some(seller_id) = self
            .session_factory
            .query_plug(seller_plug_id)
            .map(|plug| plug.owner_id())
        else {
            return report;
        };
        report.seller_plug_id = Some(seller_plug_id);
        report.buyer_id = Some(buyer.owner_id());
        report.seller_id = Some(seller_id);
        let Some(seller) = self
            .session_factory
            .personal_shop_seller(seller_plug_id)
            .filter(|seller| seller.shop_opened())
        else {
            report.outcome = PersonalShopPurchaseOutcome::ShopClosed;
            return report;
        };
        let Some(price) = seller.goods_price(goods_id) else {
            report.outcome = PersonalShopPurchaseOutcome::MissingGoods;
            return report;
        };
        report.price = Some(price.price);
        if price.price == 0 {
            report.outcome = PersonalShopPurchaseOutcome::MissingGoods;
            return report;
        }
        if !billing_completion && price.price_type != 0 {
            report.outcome = PersonalShopPurchaseOutcome::UnsupportedPriceType {
                price_type: price.price_type,
            };
            return report;
        }
        let Some(previous) = seller
            .goods()
            .base()
            .base()
            .original_container_information(goods_id)
        else {
            report.outcome = PersonalShopPurchaseOutcome::MissingGoods;
            return report;
        };
        let Some(goods) = self
            .players
            .get(&seller_id)
            .and_then(|player| {
                player.trade_source_goods(
                    previous.container_extend_id,
                    previous.goods_position,
                    goods_id,
                )
            })
            .cloned()
        else {
            report.outcome = PersonalShopPurchaseOutcome::MissingGoods;
            return report;
        };
        let Some(buyer_player) = self.players.get(&buyer.owner_id()) else {
            return report;
        };
        if !billing_completion && buyer_player.money() < price.price {
            report.outcome = PersonalShopPurchaseOutcome::InsufficientMoney {
                notice_delivery: colored_player_notice_message(
                    0xffff_ffff,
                    0,
                    self.get_string_by_id(b"GS0261"),
                )
                .send_to_player(self.net_server(), buyer.owner_id()),
            };
            return report;
        }
        if buyer_player.packet().is_full(&self.goods_factory) {
            report.outcome = PersonalShopPurchaseOutcome::PacketFull {
                notice_delivery: colored_player_notice_message(
                    0xffff_ffff,
                    0,
                    self.get_string_by_id(b"GS0260"),
                )
                .send_to_player(self.net_server(), buyer.owner_id()),
            };
            return report;
        }
        let burden = buyer_player.current_burden(&self.goods_factory);
        if u32::from(buyer_player.combat_properties().burden)
            < burden.wrapping_add(goods.weight(&self.goods_factory))
        {
            report.outcome = PersonalShopPurchaseOutcome::BurdenExceeded {
                notice_delivery: colored_player_notice_message(
                    0xffff_ffff,
                    0,
                    self.get_string_by_id(b"GS0259"),
                )
                .send_to_player(self.net_server(), buyer.owner_id()),
            };
            return report;
        }
        let maximum_gold = self
            .goods_factory
            .query_goods_max_stack_number(self.goods_factory.get_gold_coin_index());
        if !billing_completion
            && maximum_gold
                < self
                    .players
                    .get(&seller_id)
                    .map_or(0, CPlayer::money)
                    .wrapping_add(price.price)
        {
            report.outcome = PersonalShopPurchaseOutcome::SellerMoneyCapacity {
                notice_delivery: colored_player_notice_message(
                    0xffff_ffff,
                    0,
                    self.get_string_by_id(b"GS0258"),
                )
                .send_to_player(self.net_server(), buyer.owner_id()),
            };
            return report;
        }
        let mut packet_probe = buyer_player.packet().clone();
        let mut probe_goods = Some(goods.clone());
        let probe = packet_probe.add_goods(&mut probe_goods, &self.goods_factory, true);
        if probe_goods.is_some() || matches!(probe, VolumeGoodsAddOutcome::Rejected(_)) {
            report.outcome = PersonalShopPurchaseOutcome::PacketFull {
                notice_delivery: colored_player_notice_message(
                    0xffff_ffff,
                    0,
                    self.get_string_by_id(b"GS0260"),
                )
                .send_to_player(self.net_server(), buyer.owner_id()),
            };
            return report;
        }
        let audit = {
            let seller = self.players.get(&seller_id).expect("seller проверен");
            let buyer = self.players.get(&buyer.owner_id()).expect("buyer проверен");
            (
                u32::from(seller.pk_count()),
                seller.money(),
                seller.shape().get_tile_x().unwrap_or(0),
                seller.shape().get_tile_y().unwrap_or(0),
                seller.client_ip(),
                u32::from(buyer.pk_count()),
                buyer.money(),
                buyer.shape().get_tile_x().unwrap_or(0),
                buyer.shape().get_tile_y().unwrap_or(0),
                buyer.client_ip(),
                goods.amount(),
                goods.name().to_vec(),
            )
        };
        let mut seller_player = self
            .players
            .remove(&seller_id)
            .expect("seller проверен перед source removal");
        let removed = if previous.container_extend_id == 1 {
            seller_player
                .packet_mut()
                .take_goods(
                    previous.goods_position,
                    goods.amount(),
                    &self.goods_factory,
                    |_| None,
                )
                .and_then(|outcome| match outcome {
                    VolumeGoodsRemoveOutcome::Removed(AmountLimitGoodsTaken::Removed(removed))
                    | VolumeGoodsRemoveOutcome::RemovedButCellMissing(
                        AmountLimitGoodsTaken::Removed(removed),
                    ) => Some(removed.goods),
                    _ => None,
                })
        } else if previous.container_extend_id == 2 {
            let facts = context.enhancement_equipment_remove_facts(
                &seller_player,
                &goods,
                self.globe_setup.pack_add_enabled(),
            );
            let mut recompute =
                |player: &CPlayer| context.recompute_enhancement_player_properties(player);
            let mut removal = seller_player.remove_equipment_goods(
                goods_id,
                &self.goods_factory,
                &self.skill_factory,
                facts,
                &mut recompute,
            );
            drop(recompute);
            self.publish_player_equipment_remove_report(&mut removal, context);
            match removal.outcome {
                EquipmentRemoveOutcome::Removed(removed) => Some(removed.goods),
                _ => None,
            }
        } else {
            None
        };
        self.players.insert(seller_id, seller_player);
        let Some(removed) = removed else {
            report.outcome = PersonalShopPurchaseOutcome::SourceRemovalFailed;
            return report;
        };
        self.session_factory
            .personal_shop_seller_mut(seller_plug_id)
            .expect("seller plug проверен")
            .remove_goods(goods_id);
        report.seller_delete_delivery = Some(self.send_container_object_delete(
            seller_id,
            &previous,
            removed.identity(),
            removed.amount(),
        ));

        let original = removed.clone();
        let (additions, rejected) = {
            let buyer_player = self
                .players
                .get_mut(&buyer.owner_id())
                .expect("buyer проверен перед packet add");
            let mut encode = |goods: &CGoods| context.encode_goods_for_old_client(goods);
            buyer_player.add_traded_goods_to_packet(vec![removed], &self.goods_factory, &mut encode)
        };
        for addition in &additions {
            report
                .buyer_packet_deliveries
                .push(self.send_player_packet_addition(addition));
        }
        if !rejected.is_empty()
            || additions
                .iter()
                .any(|addition| addition.resulting_amount.is_none())
        {
            let seller_player = self
                .players
                .get_mut(&seller_id)
                .expect("seller остаётся online для legacy packet rollback");
            let mut encode = |goods: &CGoods| context.encode_goods_for_old_client(goods);
            let (rollback, _) = seller_player.add_traded_goods_to_packet(
                if rejected.is_empty() {
                    vec![original]
                } else {
                    rejected
                },
                &self.goods_factory,
                &mut encode,
            );
            for addition in &rollback {
                report
                    .buyer_packet_deliveries
                    .push(self.send_player_packet_addition(addition));
            }
            report.outcome = PersonalShopPurchaseOutcome::BuyerAddFailed;
            return report;
        }

        if !billing_completion {
            let buyer_change = self
                .players
                .get_mut(&buyer.owner_id())
                .expect("buyer остаётся online")
                .decrease_money(price.price, &self.goods_factory);
            report
                .money_deliveries
                .extend(self.send_player_money_decrease(buyer.owner_id(), &buyer_change.outcome));
            let created =
                self.create_goods_batch(self.goods_factory.get_gold_coin_index(), price.price);
            let seller_increase = self
                .players
                .get_mut(&seller_id)
                .expect("seller остаётся online")
                .increase_money(price.price, &self.goods_factory, created);
            report
                .money_deliveries
                .extend(self.send_player_money_increase(seller_id, &seller_increase, context));
        }
        if !billing_completion && self.log_system.goods_trade_log_enabled() {
            let mut message = CMessage::new(0x0006_0201);
            message.add_byte(1);
            message.add_long(seller_id);
            message.add_ulong(audit.0);
            message.add_ulong(audit.1);
            message.add_long(audit.2);
            message.add_long(audit.3);
            message.add_long(buyer.owner_id());
            message.add_ulong(audit.5);
            message.add_ulong(audit.6);
            message.add_long(audit.7);
            message.add_long(audit.8);
            message.base_mut().add_guid(goods_id);
            message.add_ulong(price.price);
            message.add_ulong(audit.10);
            add_legacy_c_string(message.base_mut(), &audit.11);
            message.add_ulong(audit.9);
            message.add_ulong(audit.4);
            report.audit_delivery = Some(message.send(self, false));
        }
        report.outcome = PersonalShopPurchaseOutcome::Completed;
        report
    }

    pub(crate) fn finish_personal_shop_session(
        &mut self,
        session_id: i32,
    ) -> PersonalShopTerminalReport {
        let mut report = PersonalShopTerminalReport {
            session_id,
            seller_plug_id: None,
            seller_id: None,
            buyer_plug_ids: Vec::new(),
            buyer_ids: Vec::new(),
            deliveries: Vec::new(),
            around_delivery: None,
            collected_plug_ids: Vec::new(),
        };
        let Some((seller_plug_id, seller_id, buyers)) =
            self.session_factory.personal_shop_participants(session_id)
        else {
            return report;
        };
        report.seller_plug_id = Some(seller_plug_id);
        report.seller_id = Some(seller_id);
        if let Some(seller) = self
            .session_factory
            .personal_shop_seller_mut(seller_plug_id)
        {
            seller.close_down();
        }
        if let Some(player) = self.players.get_mut(&seller_id) {
            player.set_personal_shop_flag(0, 0);
            player.detach_equipment_session_listener(seller_plug_id);
            player.set_current_progress_snapshot(PlayerProgress::None);
        }
        let mut around = CMessage::new(0x000c_0007);
        around.add_long(seller_id);
        around.add_long(seller_plug_id);
        report.around_delivery = self.send_player_shape_around(seller_id, None, &around);
        for buyer in buyers {
            report.buyer_plug_ids.push(buyer.plug_id());
            report.buyer_ids.push(buyer.owner_id());
            if let Some(player) = self.players.get_mut(&buyer.owner_id()) {
                player.set_current_progress_snapshot(PlayerProgress::None);
                let mut message = CMessage::new(0x000c_0008);
                message.add_long(session_id);
                report
                    .deliveries
                    .push(message.send_to_player(self.net_server(), buyer.owner_id()));
            }
        }
        let _ = self.session_factory.end_session(session_id);
        report.collected_plug_ids = self.session_factory.garbage_collect_session(session_id);
        report
    }

    pub(crate) fn exit_personal_shop_buyer(
        &mut self,
        session_id: i32,
        plug_id: i32,
    ) -> PersonalShopTerminalReport {
        let mut report = PersonalShopTerminalReport {
            session_id,
            seller_plug_id: self
                .session_factory
                .personal_shop_seller_plug_id(session_id),
            seller_id: None,
            buyer_plug_ids: Vec::new(),
            buyer_ids: Vec::new(),
            deliveries: Vec::new(),
            around_delivery: None,
            collected_plug_ids: Vec::new(),
        };
        report.seller_id = report.seller_plug_id.and_then(|seller_plug_id| {
            self.session_factory
                .query_plug(seller_plug_id)
                .map(|plug| plug.owner_id())
        });
        let Some(buyer) = self
            .session_factory
            .remove_personal_shop_buyer(session_id, plug_id)
        else {
            return report;
        };
        report.buyer_plug_ids.push(plug_id);
        report.buyer_ids.push(buyer.owner_id());
        report.collected_plug_ids.push(plug_id);
        if let Some(player) = self.players.get_mut(&buyer.owner_id()) {
            player.set_current_progress_snapshot(PlayerProgress::None);
            let mut message = CMessage::new(0x000c_0008);
            message.add_long(session_id);
            report
                .deliveries
                .push(message.send_to_player(self.net_server(), buyer.owner_id()));
        }
        report
    }

    /// Создаёт достигнутую process-owned проекцию `CGame` с подтверждёнными
    /// setup defaults; ещё не материализованные gameplay owners не подменяет.
    pub(crate) fn new() -> Self {
        Self {
            setup: GameSetup::default(),
            setup_ex: GameSetupEx::default(),
            random_state: 1,
            sequence_registry: CSequenceRegistry::default(),
            player_list: CPlayerList::default(),
            trade_list: CTradeList::default(),
            goods_factory: CGoodsFactory::default(),
            skill_factory: CSkillFactory::default(),
            monster_registry: MonsterRegistry::new(),
            monster_drop_registry: MonsterDropRegistry::new(),
            thing_setup: CThingSetup::default(),
            increment_shop_list: CIncrementShopList::default(),
            contribute_setup: CContributeSetup::default(),
            log_system: CLogSystem::default(),
            gm_list: CGMList::default(),
            da_kong_xiang_qian: CDaKongXiangQian::default(),
            globe_setup: GlobeSetupSnapshot::default(),
            region_router: RegionRouter::default(),
            area_width: 15,
            area_height: 15,
            auction_room: CGameAuctionRoom::new(),
            auction_now: false,
            auction_last_check_seconds: 0,
            auction_tick_ms: 0,
            function_list_file_data: None,
            variable_list_file_data: None,
            script_file_data: BTreeMap::new(),
            script_functions: CScriptFunctionRegistry::default(),
            general_variables: CVariableList::default(),
            active_scripts: BTreeMap::new(),
            next_script_id: 0,
            next_organizing_session_id: 0,
            next_organizing_password: 0,
            pending_faction_creations: BTreeMap::new(),
            pending_faction_applications: BTreeMap::new(),
            pending_faction_war_declarations: BTreeMap::new(),
            string_table: MyStringTable::new(),
            quest_system: CQuestSystem::default(),
            country_param: CCountryParam::default(),
            country_handler: CCountryHandler::default(),
            attack_city_sys: CAttackCitySys::default(),
            village_war_sys: CVillageWarSys::default(),
            country_war_sys: CountryWarSys::default(),
            four_nation_war_sys: CFourNationWarSys::default(),
            emotion: CEmotion::default(),
            region_setup: CRegionSetup::default(),
            hit_level_setup: CHitLevelSetup::default(),
            prison_conf: PrisonConf::default(),
            precious_box_conf: PreciousBoxConf::default(),
            fairy_exp_conf: CFairyExpConf::default(),
            battle_fairy_exp_config: CBattleFairyExpConfig::default(),
            battle_fairy_property: CBattleFairyProperty::default(),
            equipment_compose_list: EquipmentComposeList::default(),
            words_filter: CWordsFilter::default(),
            jjc_level_data: BTreeMap::new(),
            tao_zhuang_setup: CTaoZhuangSetup::default(),
            ci_qing_setup: CCiQingSetup::default(),
            ling_bao_setup: CLingBaoSetup::default(),
            gods_battle_mgr: CGodsBattleMgr::default(),
            synthesis: CSynthesis::default(),
            new_skill_monster_conf: NewSkillMonsterConf::default(),
            goods_destroy_setup: GoodsDestroySetup::default(),
            change_body_conf: CChangeBodyConf::default(),
            honor_eliminate_config: HonorElimilateConfig::default(),
            dupli_region_setup: None,
            move_check_cells: MoveCheckCellRegistry::new(),
            player_ranks: None,
            honor_ranks: CHonorRanks::default(),
            goods_war: None,
            id_index: 0,
            team_id_counter: 1,
            login_server_id: 0,
            world_server_id: 0,
            network_setup: None,
            world_client: None,
            billing_client: None,
            net_server: None,
            world_reconnect_task: None,
            billing_reconnect_task: None,
            net_session_manager: CNetSessionManager::new(NetSessionManagerVariant::GameServer),
            session_factory: CSessionFactory::default(),
            players: BTreeMap::new(),
            regions: BTreeMap::new(),
            proxy_regions: BTreeMap::new(),
            initial_total_monsters: 0,
            initial_total_npcs: 0,
            team_session_ids: BTreeMap::new(),
            main_loop_state: GameMainLoopState::default(),
        }
    }

    pub(crate) fn with_send_state(net_server: CMyNetServer) -> Self {
        let mut game = Self::new();
        game.net_server = Some(net_server);
        game
    }

    /// Создаёт pre-network assembly после уже выполненного `LoadSetup*`.
    pub(crate) fn with_network_setup(network_setup: GameNetworkSetup) -> Self {
        let mut game = Self::new();
        game.network_setup = Some(network_setup);
        game
    }

    /// Читает обязательный `setup.ini`, затем необязательный `setupex.ini` и
    /// публикует единый network plan в исходной позиции перед `InitNetServer`.
    /// Некорректная пара останавливает последующие extraction-ы, сохраняя уже
    /// применённые значения и constructor defaults оставшихся полей.
    pub(crate) fn load_runtime_setup(
        &mut self,
        paths: &GameRuntimePaths,
    ) -> Result<GameRuntimeSetupReport, GameRuntimeSetupError> {
        // Производный plan не должен пережить неуспешную повторную загрузку.
        self.network_setup = None;
        let setup_bytes = fs::read(&paths.setup).map_err(|source| {
            GameRuntimeSetupError::Setup(GameSetupOpenError {
                path: paths.setup.clone(),
                source,
            })
        })?;
        let setup = self.setup.parse_positional(&setup_bytes);

        let setup_ex = match fs::read(&paths.setup_ex) {
            Ok(bytes) => GameSetupExLoad::Loaded(self.setup_ex.parse_positional(&bytes)),
            Err(source) => GameSetupExLoad::Unavailable(GameSetupOpenError {
                path: paths.setup_ex.clone(),
                source,
            }),
        };
        self.network_setup = Some(self.setup.network_setup(&self.setup_ex)?);
        Ok(GameRuntimeSetupReport { setup, setup_ex })
    }

    /// Выполняет достигнутый `Init` от первого RNG seed до Billing-попытки.
    /// World failure завершает цепочку; Billing failure только остаётся в
    /// отчёте, как исходное предупреждение с последующим продолжением.
    pub(crate) async fn init_through_billing(
        &mut self,
        paths: &GameRuntimePaths,
        wall_time_seconds: u32,
        sequence_seed_ms: u32,
    ) -> Result<GameInitializationThroughBillingReport, GameInitializationThroughBillingError> {
        // C++ constructor запоминал `_time` при создании process singleton-а.
        // Safe owner получает первый доказанный wall-clock на входе `Init`;
        // пока state=false, эта разница ненаблюдаема, а true-ветвь всегда
        // перезаписывает timestamp через exact `SetAuctionState`.
        self.auction_last_check_seconds = wall_time_seconds;
        self.random_state = wall_time_seconds;
        let _discarded_roll = game_legacy_random(&mut self.random_state, 100);

        let setup = self
            .load_runtime_setup(paths)
            .map_err(GameInitializationThroughBillingError::Setup)?;
        let world = self.init_world_client().await;
        if matches!(&world, GameClientInitialization::Failed { .. }) {
            return Err(GameInitializationThroughBillingError::WorldUnavailable {
                setup,
                connection: world,
            });
        }

        self.random_state = sequence_seed_ms;
        let sequence_count = self.setup.sequence_count;
        let random_state = &mut self.random_state;
        self.sequence_registry
            .initialize(sequence_count, || next_msvc_rand(random_state))
            .map_err(GameInitializationThroughBillingError::Sequence)?;
        let sequence_elements = self.sequence_registry.len();

        let billing = self.init_billing_client().await;
        Ok(GameInitializationThroughBillingReport {
            setup,
            world,
            sequence_elements,
            billing,
        })
    }

    /// Завершает точный хвост `CGame::Init` после Billing-попытки.
    pub(crate) async fn init(
        &mut self,
        paths: &GameRuntimePaths,
        wall_time_seconds: u32,
        sequence_seed_ms: u32,
    ) -> Result<GameInitializationReport, GameInitializationThroughBillingError> {
        let through_billing = self
            .init_through_billing(paths, wall_time_seconds, sequence_seed_ms)
            .await?;

        self.dupli_region_setup = Some(CDupliRegionSetup::default());
        self.move_check_cells.initialize();
        let move_check_cells = self.move_check_cells.total_len();

        let mut player_ranks = CPlayerRanks::new();
        let player_ranks_initialized = player_ranks.initialize();
        self.player_ranks = Some(player_ranks);

        let goods_war = CGoodsWarMember::new();
        let goods_war_request = goods_war.request_initial_state(self);
        self.goods_war = Some(goods_war);

        Ok(GameInitializationReport {
            through_billing,
            move_check_cells,
            player_ranks_initialized,
            goods_war_request,
        })
    }

    pub(crate) fn net_server(&self) -> &CMyNetServer {
        self.net_server
            .as_ref()
            .expect("Game send-family достигается после InitNetServer")
    }

    pub(crate) const fn current_net_server(&self) -> Option<&CMyNetServer> {
        self.net_server.as_ref()
    }

    /// Возвращает опубликованный listener-owner фактическому network runtime.
    pub(crate) fn current_net_server_mut(&mut self) -> Option<&mut CMyNetServer> {
        self.net_server.as_mut()
    }

    pub(crate) const fn world_client(&self) -> Option<&CMyNetClient> {
        self.world_client.as_ref()
    }

    pub(crate) const fn billing_client(&self) -> Option<&CMyNetClient> {
        self.billing_client.as_ref()
    }

    /// Сохраняет assignment `s_pNetClientOfWS`, затем exact server type writer.
    pub(crate) fn attach_world_client(&mut self, client: CMyNetClient) -> Option<CMyNetClient> {
        let previous = self.world_client.replace(client);
        self.world_client
            .as_mut()
            .expect("World client только что присвоен")
            .set_server_type(ServerType::World);
        previous
    }

    /// Сохраняет assignment `s_pNetClientOfBS`, затем exact server type writer.
    pub(crate) fn attach_billing_client(&mut self, client: CMyNetClient) -> Option<CMyNetClient> {
        let previous = self.billing_client.replace(client);
        self.billing_client
            .as_mut()
            .expect("Billing client только что присвоен")
            .set_server_type(ServerType::Billing);
        previous
    }

    /// Закрывает прежний Billing owner до присваивания reconnect replacement.
    pub(crate) fn replace_billing_client(&mut self, client: CMyNetClient) -> bool {
        let mut previous = self.billing_client.take();
        if let Some(previous) = previous.as_mut() {
            let _legacy_result = previous.close();
        }
        let previous_closed = previous.is_some();
        drop(previous);
        self.attach_billing_client(client);
        previous_closed
    }

    pub(crate) fn current_billing_client_mut(&mut self) -> Option<&mut CMyNetClient> {
        self.billing_client.as_mut()
    }

    /// Terminal startup присваивает login ID до чтения world ID; раздельные
    /// setter-ы сохраняют partial effect malformed хвоста.
    pub(crate) const fn set_login_server_id(&mut self, login_server_id: i32) {
        self.login_server_id = login_server_id;
    }

    pub(crate) const fn set_world_server_id(&mut self, world_server_id: i32) {
        self.world_server_id = world_server_id;
    }

    pub(crate) const fn server_ids(&self) -> (i32, i32) {
        (self.login_server_id, self.world_server_id)
    }

    /// Исторический `CGame::GetAreaID` возвращает именно login-server ID,
    /// а не ID текущего региона игрока.
    pub(crate) const fn area_id(&self) -> i32 {
        self.login_server_id
    }

    pub(crate) const fn set_id_index(&mut self, id_index: u8) {
        self.id_index = id_index;
    }

    pub(crate) const fn id_index(&self) -> u8 {
        self.id_index
    }

    /// Выдаёт исходный 24-bit team counter с Game index в старшем байте.
    /// Process-static C++ counter хранится здесь, поскольку `CGame` — singleton.
    pub(crate) fn get_team_id(&mut self, session_id: i32) -> u32 {
        if session_id == 0 {
            return 0;
        }

        let result = self.team_id_counter | (u32::from(self.id_index) << 24);
        self.team_id_counter += 1;
        if self.team_id_counter >= 0x00ff_ffff {
            self.team_id_counter = 1;
        }
        result
    }

    pub(crate) const fn sequence_registry(&self) -> &CSequenceRegistry {
        &self.sequence_registry
    }

    pub(crate) const fn player_list(&self) -> &CPlayerList {
        &self.player_list
    }

    pub(crate) const fn player_list_mut(&mut self) -> &mut CPlayerList {
        &mut self.player_list
    }

    pub(crate) const fn trade_list(&self) -> &CTradeList {
        &self.trade_list
    }

    pub(crate) const fn trade_list_mut(&mut self) -> &mut CTradeList {
        &mut self.trade_list
    }

    pub(crate) const fn goods_factory(&self) -> &CGoodsFactory {
        &self.goods_factory
    }

    pub(crate) const fn goods_factory_mut(&mut self) -> &mut CGoodsFactory {
        &mut self.goods_factory
    }

    pub(crate) fn decode_auction_goods(&self, source: &[u8]) -> Result<CGoods, GoodsDecodeError> {
        let mut goods = CGoods::default();
        let mut cursor = 0;
        goods.unserialize(
            source,
            &mut cursor,
            true,
            &self.goods_factory,
            |equip_level, level| self.fairy_exp_conf.dw_exp_up(equip_level, level),
            |equip_level, level| self.battle_fairy_exp_config.dw_exp_up(equip_level, level),
        )?;
        Ok(goods)
    }

    pub(crate) fn decode_player_game_save(
        &self,
        source: &[u8],
        cursor: &mut usize,
        now_ms: u32,
    ) -> Result<(CPlayer, PlayerGameSaveDecodeReport), PlayerGameSaveCodecError> {
        let mut ordinary_threshold =
            |equip_level, level| self.fairy_exp_conf.dw_exp_up(equip_level, level);
        let mut battle_threshold =
            |equip_level, level| self.battle_fairy_exp_config.dw_exp_up(equip_level, level);
        CPlayer::decode_game_save(
            source,
            cursor,
            &self.goods_factory,
            &self.skill_factory,
            self.variable_list_file_data.as_deref(),
            now_ms,
            self.globe_setup.one_pk_count_time_ms(),
            &mut ordinary_threshold,
            &mut battle_threshold,
        )
    }

    pub(crate) fn encode_player_game_save<Context: ScriptRegionChangeContext>(
        &self,
        player: &CPlayer,
        destination: &mut Vec<u8>,
        context: &mut Context,
    ) -> bool {
        let (pets, carriage, recreate_carriage) = context.snapshot_script_player_summons(player);
        player
            .encode_game_save(
                destination,
                &self.goods_factory,
                context.now_milliseconds(),
                self.globe_setup.one_pk_count_time_ms(),
                &pets,
                &carriage,
                recreate_carriage,
            )
            .unwrap_or(false)
    }

    pub(crate) fn select_player_enhancement_goods(
        &mut self,
        player_id: i32,
        source_extend_id: i32,
        source_position: u32,
        goods_id: CGuid,
        amount: u32,
    ) -> Result<EnhancementSelectionReport, EnhancementSelectionBlock> {
        let (players, goods_factory) = (&mut self.players, &self.goods_factory);
        players
            .get_mut(&player_id)
            .ok_or(EnhancementSelectionBlock::MissingGoods)?
            .select_enhancement_goods(
                source_extend_id,
                source_position,
                goods_id,
                amount,
                goods_factory,
            )
    }

    pub(crate) fn clear_player_enhancement_selection(
        &mut self,
        player_id: i32,
        shadow_position: u32,
        goods_id: CGuid,
        amount: u32,
    ) -> Result<EnhancementDeselectionReport, EnhancementDeselectionBlock> {
        self.players
            .get_mut(&player_id)
            .ok_or(EnhancementDeselectionBlock::MissingShadow)?
            .clear_enhancement_selection(shadow_position, goods_id, amount)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn select_player_equipment_session_goods(
        &mut self,
        player_id: i32,
        session_id: i32,
        session_extend_id: i32,
        requested_position: u32,
        source_extend_id: i32,
        source_position: u32,
        goods_id: CGuid,
        amount: u32,
    ) -> Result<EquipmentSessionSelectionReport, EquipmentSessionSelectionBlock> {
        if !matches!(source_extend_id, 1 | 2) {
            return Err(EquipmentSessionSelectionBlock::UnsupportedSourceContainer {
                extend_id: source_extend_id,
            });
        }
        let player = self
            .players
            .get(&player_id)
            .ok_or(EquipmentSessionSelectionBlock::MissingPlayer)?;
        let goods = match source_extend_id {
            1 => player.packet().get_goods(source_position),
            2 => player.equipment().get_goods(source_position),
            _ => unreachable!("source extend проверен выше"),
        }
        .ok_or(EquipmentSessionSelectionBlock::MissingGoods)?;
        if goods.identity().ex_id != goods_id || goods.amount() != amount {
            return Err(EquipmentSessionSelectionBlock::SourceMismatch);
        }
        let goods = goods.clone();
        let source = crate::gameserver::appserver::container::ccontainer::PreviousContainer {
            container_type: 400,
            container_id: player_id,
            container_extend_id: source_extend_id,
            goods_position: source_position,
        };
        let added = self
            .session_factory
            .record_equipment_session_shadow(
                session_id,
                session_extend_id,
                player_id,
                requested_position,
                &goods,
                source,
                &self.goods_factory,
            )
            .map_err(EquipmentSessionSelectionBlock::Shadow)?;
        Ok(EquipmentSessionSelectionReport {
            goods: goods.identity(),
            source,
            added,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn clear_player_equipment_session_selection(
        &mut self,
        player_id: i32,
        session_id: i32,
        session_extend_id: i32,
        shadow_position: u32,
        goods_id: CGuid,
        amount: u32,
        destination_extend_id: i32,
        destination_position: u32,
    ) -> Result<EquipmentSessionShadowRemoved, EquipmentSessionClearBlock> {
        let original = self
            .session_factory
            .equipment_session_shadow_original(session_id, session_extend_id, player_id, goods_id)
            .ok_or(EquipmentSessionClearBlock::MissingShadow)?;
        let recorded_position = self
            .session_factory
            .equipment_session_shadow_position(session_id, session_extend_id, player_id, goods_id)
            .ok_or(EquipmentSessionClearBlock::MissingShadow)?;
        if recorded_position != shadow_position
            || original.container_type != 400
            || original.container_id != player_id
            || original.container_extend_id != destination_extend_id
            || original.goods_position != destination_position
        {
            return Err(EquipmentSessionClearBlock::SourceMismatch);
        }
        let player = self
            .players
            .get(&player_id)
            .ok_or(EquipmentSessionClearBlock::MissingPlayer)?;
        let goods = match original.container_extend_id {
            1 => player.packet().get_goods(original.goods_position),
            2 => player.equipment().get_goods(original.goods_position),
            _ => None,
        }
        .filter(|goods| goods.identity().ex_id == goods_id && goods.amount() == amount)
        .ok_or(EquipmentSessionClearBlock::SourceMismatch)?;
        let _ = goods;
        self.session_factory
            .remove_equipment_session_shadow(session_id, session_extend_id, player_id, goods_id)
            .ok_or(EquipmentSessionClearBlock::MissingShadow)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn select_player_personal_shop_goods(
        &mut self,
        player_id: i32,
        session_id: i32,
        session_extend_id: i32,
        requested_position: u32,
        source_extend_id: i32,
        source_position: u32,
        goods_id: CGuid,
        amount: u32,
    ) -> Result<PersonalShopSelectionReport, PersonalShopSelectionBlock> {
        if !matches!(source_extend_id, 1 | 2) {
            return Err(PersonalShopSelectionBlock::UnsupportedSourceContainer {
                extend_id: source_extend_id,
            });
        }
        let seller_plug_id = session_extend_id >> 8;
        if self
            .session_factory
            .personal_shop_seller(seller_plug_id)
            .is_some_and(|seller| seller.shop_opened())
        {
            return Err(PersonalShopSelectionBlock::ShopOpened);
        }
        let player = self
            .players
            .get(&player_id)
            .ok_or(PersonalShopSelectionBlock::MissingPlayer)?;
        let goods = match source_extend_id {
            1 => player.packet().get_goods(source_position),
            2 => player.equipment().get_goods(source_position),
            _ => unreachable!("source extend проверен выше"),
        }
        .ok_or(PersonalShopSelectionBlock::MissingGoods)?;
        if goods.identity().ex_id != goods_id || goods.amount() != amount {
            return Err(PersonalShopSelectionBlock::SourceMismatch);
        }
        let goods = goods.clone();
        let source = crate::gameserver::appserver::container::ccontainer::PreviousContainer {
            container_type: 400,
            container_id: player_id,
            container_extend_id: source_extend_id,
            goods_position: source_position,
        };
        let added = self
            .session_factory
            .record_personal_shop_shadow(
                session_id,
                session_extend_id,
                player_id,
                requested_position,
                &goods,
                source,
            )
            .map_err(PersonalShopSelectionBlock::Shadow)?;
        Ok(PersonalShopSelectionReport {
            goods: goods.identity(),
            source,
            added,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn clear_player_personal_shop_selection(
        &mut self,
        player_id: i32,
        session_id: i32,
        session_extend_id: i32,
        shadow_position: u32,
        goods_id: CGuid,
        amount: u32,
        destination_extend_id: i32,
        destination_position: u32,
    ) -> Result<
        crate::gameserver::appserver::session::csessionfactory::PersonalShopShadowRemoved,
        PersonalShopClearBlock,
    > {
        let seller_plug_id = session_extend_id >> 8;
        if self
            .session_factory
            .personal_shop_seller(seller_plug_id)
            .is_some_and(|seller| seller.shop_opened())
        {
            return Err(PersonalShopClearBlock::ShopOpened);
        }
        let original = self
            .session_factory
            .personal_shop_shadow_original(session_id, session_extend_id, player_id, goods_id)
            .ok_or(PersonalShopClearBlock::MissingShadow)?;
        let recorded_position = self
            .session_factory
            .personal_shop_shadow_position(session_id, session_extend_id, player_id, goods_id)
            .ok_or(PersonalShopClearBlock::MissingShadow)?;
        if recorded_position != shadow_position
            || original.container_type != 400
            || original.container_id != player_id
            || original.container_extend_id != destination_extend_id
            || original.goods_position != destination_position
        {
            return Err(PersonalShopClearBlock::SourceMismatch);
        }
        let player = self
            .players
            .get(&player_id)
            .ok_or(PersonalShopClearBlock::MissingPlayer)?;
        let goods = match original.container_extend_id {
            1 => player.packet().get_goods(original.goods_position),
            2 => player.equipment().get_goods(original.goods_position),
            _ => None,
        }
        .filter(|goods| goods.identity().ex_id == goods_id && goods.amount() == amount)
        .ok_or(PersonalShopClearBlock::SourceMismatch)?;
        let _ = goods;
        self.session_factory
            .remove_personal_shop_shadow(session_id, session_extend_id, player_id, goods_id)
            .ok_or(PersonalShopClearBlock::MissingShadow)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn transfer_player_enhancement_goods<Context: GameContainerMessageRuntime>(
        &mut self,
        player_id: i32,
        shadow_position: u32,
        goods_id: CGuid,
        amount: u32,
        destination_extend_id: i32,
        destination_position: u32,
        context: &mut Context,
    ) -> Result<EnhancementTransferReport, EnhancementTransferBlock> {
        if !matches!(destination_extend_id, 1 | 2) {
            return Err(EnhancementTransferBlock::UnsupportedDestinationContainer {
                extend_id: destination_extend_id,
            });
        }
        let mut player = self
            .players
            .remove(&player_id)
            .ok_or(EnhancementTransferBlock::MissingPlayer)?;
        let result = self.transfer_player_enhancement_goods_inner(
            &mut player,
            shadow_position,
            goods_id,
            amount,
            destination_extend_id,
            destination_position,
            context,
        );
        self.players.insert(player_id, player);
        result
    }

    /// Замыкает direct `0x90301` ownership transfer в двухъячеечный
    /// `m_cAuctionContainer`. Equipment source проходит тот же полный remove
    /// callback/effect tail, что и другие достигнутые container transfers.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn move_player_goods_to_auction_listing<Context: GameContainerMessageRuntime>(
        &mut self,
        player_id: i32,
        source_extend_id: i32,
        source_position: u32,
        goods_id: CGuid,
        amount: u32,
        destination_position: u32,
        context: &mut Context,
    ) -> Result<AuctionListingTransferReport, AuctionListingTransferBlock> {
        if !matches!(source_extend_id, 1 | 2 | 14) {
            return Err(AuctionListingTransferBlock::UnsupportedSourceContainer {
                extend_id: source_extend_id,
            });
        }
        let mut player = self
            .players
            .remove(&player_id)
            .ok_or(AuctionListingTransferBlock::MissingSourceGoods)?;
        let result = self.move_player_goods_to_auction_listing_inner(
            &mut player,
            source_extend_id,
            source_position,
            goods_id,
            amount,
            destination_position,
            context,
        );
        self.players.insert(player_id, player);
        result
    }

    #[allow(clippy::too_many_arguments)]
    fn move_player_goods_to_auction_listing_inner<Context: GameContainerMessageRuntime>(
        &self,
        player: &mut CPlayer,
        source_extend_id: i32,
        source_position: u32,
        goods_id: CGuid,
        amount: u32,
        destination_position: u32,
        context: &mut Context,
    ) -> Result<AuctionListingTransferReport, AuctionListingTransferBlock> {
        if destination_position >= 2
            || !player
                .auction_listing()
                .is_space_enough(destination_position)
        {
            return Err(AuctionListingTransferBlock::InvalidDestinationPosition {
                position: destination_position,
            });
        }
        let goods = match source_extend_id {
            1 => player.packet().get_goods(source_position),
            2 => player.equipment().get_goods(source_position),
            14 => player.auction_goods().get_goods(source_position),
            _ => unreachable!("source extend проверен выше"),
        }
        .filter(|goods| goods.identity().ex_id == goods_id && goods.amount() == amount)
        .ok_or(AuctionListingTransferBlock::MissingSourceGoods)?;
        if matches!(
            goods.base_properties_index(),
            index if index == self.goods_factory.get_gold_coin_index()
                || index == self.goods_factory.get_yuan_bao_index()
        ) {
            return Err(AuctionListingTransferBlock::InvalidCurrency);
        }
        if self
            .goods_factory
            .query_goods_base_properties(goods.base_properties_index())
            .is_none()
        {
            return Err(AuctionListingTransferBlock::MissingBaseProperties);
        }
        let goods_identity = goods.identity();
        let slot_zero_was_empty = player.auction_listing().get_goods(0).is_none();
        let pack_add_enabled = self.globe_setup.pack_add_enabled();

        let (removal, mut incoming) = match source_extend_id {
            1 => {
                let removed = player
                    .packet_mut()
                    .remove_goods(goods_id)
                    .ok_or(AuctionListingTransferBlock::PacketRemovalFailed)?;
                let removed = match removed {
                    VolumeGoodsRemoveOutcome::Removed(AmountLimitGoodsTaken::Removed(removed)) => {
                        removed
                    }
                    _ => return Err(AuctionListingTransferBlock::PacketRemovalFailed),
                };
                let crate::gameserver::appserver::container::camountlimitgoodscontainer::AmountLimitGoodsRemoved {
                    owner_type,
                    owner_id,
                    position,
                    amount,
                    listeners,
                    goods,
                } = removed;
                (
                    AuctionListingTransferRemoval::Packet {
                        owner_type,
                        owner_id,
                        position: position.unwrap_or(source_position),
                        amount,
                        listeners,
                    },
                    Some(goods),
                )
            }
            14 => {
                let removed = player
                    .auction_goods_mut()
                    .remove_goods(goods_id)
                    .ok_or(AuctionListingTransferBlock::AuctionGoodsRemovalFailed)?;
                let removed = match removed {
                    VolumeGoodsRemoveOutcome::Removed(AmountLimitGoodsTaken::Removed(removed))
                    | VolumeGoodsRemoveOutcome::RemovedButCellMissing(
                        AmountLimitGoodsTaken::Removed(removed),
                    ) => removed,
                    _ => return Err(AuctionListingTransferBlock::AuctionGoodsRemovalFailed),
                };
                let crate::gameserver::appserver::container::camountlimitgoodscontainer::AmountLimitGoodsRemoved {
                    owner_type,
                    owner_id,
                    position,
                    amount,
                    listeners,
                    goods,
                } = removed;
                (
                    AuctionListingTransferRemoval::AuctionGoods {
                        owner_type,
                        owner_id,
                        position: position.unwrap_or(source_position),
                        amount,
                        listeners,
                    },
                    Some(goods),
                )
            }
            2 => {
                let remove_facts =
                    context.enhancement_equipment_remove_facts(player, goods, pack_add_enabled);
                let mut recompute =
                    |player: &CPlayer| context.recompute_enhancement_player_properties(player);
                let mut report = player.remove_equipment_goods(
                    goods_id,
                    &self.goods_factory,
                    &self.skill_factory,
                    remove_facts,
                    &mut recompute,
                );
                drop(recompute);
                self.publish_player_equipment_remove_report(&mut report, context);
                let outcome = std::mem::replace(
                    &mut report.outcome,
                    EquipmentRemoveOutcome::Missing {
                        partial_effects: Default::default(),
                    },
                );
                let removed = match outcome {
                    EquipmentRemoveOutcome::Removed(removed) => removed,
                    outcome => {
                        report.outcome = outcome;
                        return Err(AuctionListingTransferBlock::EquipmentRemovalFailed(report));
                    }
                };
                (
                    AuctionListingTransferRemoval::Equipment {
                        event: removed.event,
                        effects: report.effects,
                        deliveries: report.deliveries,
                    },
                    Some(removed.goods),
                )
            }
            _ => unreachable!("source extend проверен выше"),
        };
        let owner_progress_allows = player.current_progress() == PlayerProgress::None;
        let destination = player.auction_listing_mut().add_goods_at(
            destination_position,
            &mut incoming,
            &self.goods_factory,
            owner_progress_allows,
        );
        assert!(
            incoming.is_none() && matches!(destination, VolumeGoodsAddOutcome::Added(_)),
            "предварительно проверенный пустой auction listing slot обязан принять goods"
        );
        let previous_last_operated =
            player.record_last_operated_goods(source_extend_id, source_position);
        Ok(AuctionListingTransferReport {
            goods: goods_identity,
            removal,
            destination,
            listing_slot_zero_was_empty: slot_zero_was_empty,
            previous_last_operated,
        })
    }

    /// Замыкает обратный direct `0x90301` из `m_cAuctionContainer` в
    /// packet/equipment. Общий Move проверяет burden до source removal;
    /// blocked destination возвращает предмет в исходную listing-ячейку.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn withdraw_player_auction_listing_goods<Context: GameContainerMessageRuntime>(
        &mut self,
        player_id: i32,
        source_position: u32,
        goods_id: CGuid,
        amount: u32,
        destination_extend_id: i32,
        destination_position: u32,
        context: &mut Context,
    ) -> Result<AuctionListingWithdrawalReport, AuctionListingWithdrawalBlock> {
        if !matches!(destination_extend_id, 1 | 2) {
            return Err(
                AuctionListingWithdrawalBlock::UnsupportedDestinationContainer {
                    extend_id: destination_extend_id,
                },
            );
        }
        let mut player = self
            .players
            .remove(&player_id)
            .ok_or(AuctionListingWithdrawalBlock::MissingSourceGoods)?;
        let result = self.withdraw_player_auction_listing_goods_inner(
            &mut player,
            source_position,
            goods_id,
            amount,
            destination_extend_id,
            destination_position,
            context,
        );
        self.players.insert(player_id, player);
        result
    }

    #[allow(clippy::too_many_arguments)]
    fn withdraw_player_auction_listing_goods_inner<Context: GameContainerMessageRuntime>(
        &self,
        player: &mut CPlayer,
        source_position: u32,
        goods_id: CGuid,
        amount: u32,
        destination_extend_id: i32,
        destination_position: u32,
        context: &mut Context,
    ) -> Result<AuctionListingWithdrawalReport, AuctionListingWithdrawalBlock> {
        let goods = player
            .auction_listing()
            .get_goods(source_position)
            .filter(|goods| goods.identity().ex_id == goods_id && goods.amount() == amount)
            .ok_or(AuctionListingWithdrawalBlock::MissingSourceGoods)?;
        let goods_identity = goods.identity();
        if player
            .current_burden(&self.goods_factory)
            .wrapping_add(goods.weight(&self.goods_factory))
            > u32::from(player.combat_properties().burden)
        {
            return Err(AuctionListingWithdrawalBlock::BurdenExceeded);
        }

        let removed = player
            .auction_listing_mut()
            .remove_goods(goods_id)
            .ok_or(AuctionListingWithdrawalBlock::RemovalFailed)?;
        let removed = match removed {
            VolumeGoodsRemoveOutcome::Removed(AmountLimitGoodsTaken::Removed(removed)) => removed,
            VolumeGoodsRemoveOutcome::Removed(AmountLimitGoodsTaken::Split(_))
            | VolumeGoodsRemoveOutcome::RemovedButCellMissing(_) => {
                return Err(AuctionListingWithdrawalBlock::RemovalFailed);
            }
        };
        let removal = AuctionListingWithdrawalRemoval {
            owner_type: removed.owner_type,
            owner_id: removed.owner_id,
            position: removed.position.unwrap_or(source_position),
            amount: removed.amount,
            listeners: removed.listeners,
        };
        let mut incoming = Some(removed.goods);
        let pack_add_enabled = self.globe_setup.pack_add_enabled();
        let addition = self.add_enhancement_transfer_goods(
            player,
            destination_extend_id,
            destination_position,
            &mut incoming,
            pack_add_enabled,
            context,
        );
        let outcome = if incoming.is_none() {
            let (destination_goods, amount) = match &addition {
                EnhancementTransferAddition::Packet(VolumeGoodsAddOutcome::Added(added)) => {
                    (added.identity, added.amount)
                }
                EnhancementTransferAddition::Packet(VolumeGoodsAddOutcome::Stack(
                    GoodsStackMergeOutcome::Merged { target, amount },
                )) => (*target, *amount),
                EnhancementTransferAddition::Equipment(PlayerEquipmentAddReport {
                    outcome: EquipmentAddOutcome::Added(added),
                    ..
                }) => (added.identity, added.amount),
                _ => unreachable!("consumed destination goods требует successful add outcome"),
            };
            AuctionListingWithdrawalOutcome::Moved {
                addition,
                destination_goods,
                amount,
            }
        } else {
            let rejected = addition;
            let owner_progress_allows = player.current_progress() == PlayerProgress::None;
            let rollback = player.auction_listing_mut().add_goods_at(
                source_position,
                &mut incoming,
                &self.goods_factory,
                owner_progress_allows,
            );
            if incoming.is_none() {
                AuctionListingWithdrawalOutcome::RolledBack {
                    rejected,
                    restored: rollback,
                }
            } else {
                let goods = incoming
                    .take()
                    .expect("неуспешный auction-listing rollback сохраняет detached goods");
                let notification_delivery = send_enhancement_goods_collected(
                    self,
                    player.player_id(),
                    goods.name(),
                    goods.amount(),
                );
                AuctionListingWithdrawalOutcome::GoodsCollected {
                    rejected,
                    rollback,
                    goods: goods.identity(),
                    notification_delivery,
                }
            }
        };
        let previous_last_operated =
            matches!(&outcome, AuctionListingWithdrawalOutcome::Moved { .. })
                .then(|| player.record_last_operated_goods(13, source_position));
        Ok(AuctionListingWithdrawalReport {
            goods: goods_identity,
            removal,
            outcome,
            previous_last_operated,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn transfer_player_enhancement_goods_inner<Context: GameContainerMessageRuntime>(
        &self,
        player: &mut CPlayer,
        shadow_position: u32,
        goods_id: CGuid,
        amount: u32,
        destination_extend_id: i32,
        destination_position: u32,
        context: &mut Context,
    ) -> Result<EnhancementTransferReport, EnhancementTransferBlock> {
        let source = player
            .enhancement_original_container(shadow_position, goods_id)
            .ok_or(EnhancementTransferBlock::MissingShadow)?;
        if !matches!(source.container_extend_id, 1 | 2) {
            return Err(EnhancementTransferBlock::UnsupportedSourceContainer {
                extend_id: source.container_extend_id,
            });
        }
        let goods = match source.container_extend_id {
            1 => player.packet().get_goods(source.goods_position),
            2 => player.equipment().get_goods(source.goods_position),
            _ => unreachable!("source extend проверен выше"),
        }
        .filter(|goods| goods.identity().ex_id == goods_id && goods.amount() == amount)
        .ok_or(EnhancementTransferBlock::MissingSourceGoods)?;
        let goods_identity = goods.identity();
        let pack_add_enabled = self.globe_setup.pack_add_enabled();

        let (removal, mut incoming) = if source.container_extend_id == 1 {
            let removed = player
                .packet_mut()
                .remove_goods(goods_id)
                .ok_or(EnhancementTransferBlock::PacketRemovalFailed)?;
            let removed = match removed {
                VolumeGoodsRemoveOutcome::Removed(AmountLimitGoodsTaken::Removed(removed)) => {
                    removed
                }
                VolumeGoodsRemoveOutcome::Removed(AmountLimitGoodsTaken::Split(_))
                | VolumeGoodsRemoveOutcome::RemovedButCellMissing(_) => {
                    return Err(EnhancementTransferBlock::PacketRemovalFailed);
                }
            };
            let removal = EnhancementTransferRemoval::Packet {
                owner_type: removed.owner_type,
                owner_id: removed.owner_id,
                position: removed.position.unwrap_or(source.goods_position),
                amount: removed.amount,
                listeners: removed.listeners,
            };
            (removal, Some(removed.goods))
        } else {
            let remove_facts =
                context.enhancement_equipment_remove_facts(player, goods, pack_add_enabled);
            let mut recompute =
                |player: &CPlayer| context.recompute_enhancement_player_properties(player);
            let mut report = player.remove_equipment_goods(
                goods_id,
                &self.goods_factory,
                &self.skill_factory,
                remove_facts,
                &mut recompute,
            );
            drop(recompute);
            self.publish_player_equipment_remove_report(&mut report, context);
            let outcome = std::mem::replace(
                &mut report.outcome,
                EquipmentRemoveOutcome::Missing {
                    partial_effects: Default::default(),
                },
            );
            let removed = match outcome {
                EquipmentRemoveOutcome::Removed(removed) => removed,
                outcome => {
                    report.outcome = outcome;
                    return Err(EnhancementTransferBlock::EquipmentRemovalFailed(report));
                }
            };
            let removal = EnhancementTransferRemoval::Equipment {
                event: removed.event,
                effects: report.effects,
                deliveries: report.deliveries,
            };
            (removal, Some(removed.goods))
        };

        let shadow = player
            .enhancement_remove_shadow(goods_id)
            .expect("shadow проверен до удаления source goods");
        let delete_shadow_delivery =
            send_enhancement_shadow_deleted(self, player.player_id(), goods_identity, &shadow);
        let destination = self.add_enhancement_transfer_goods(
            player,
            destination_extend_id,
            destination_position,
            &mut incoming,
            pack_add_enabled,
            context,
        );
        let outcome = if incoming.is_none() {
            EnhancementTransferOutcome::Moved(destination)
        } else {
            let rejected = destination;
            let rollback = self.add_enhancement_transfer_goods(
                player,
                source.container_extend_id,
                source.goods_position,
                &mut incoming,
                pack_add_enabled,
                context,
            );
            if incoming.is_none() {
                EnhancementTransferOutcome::RolledBack {
                    rejected,
                    restored: rollback,
                }
            } else {
                let goods = incoming
                    .take()
                    .expect("неуспешный rollback сохраняет detached goods");
                let notification_delivery = send_enhancement_goods_collected(
                    self,
                    player.player_id(),
                    goods.name(),
                    goods.amount(),
                );
                let goods = goods.identity();
                EnhancementTransferOutcome::GoodsCollected {
                    rejected,
                    rollback,
                    goods,
                    notification_delivery,
                }
            }
        };
        Ok(EnhancementTransferReport {
            goods: goods_identity,
            source,
            removal,
            shadow,
            delete_shadow_delivery,
            outcome,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn transfer_player_equipment_session_goods<Context: GameContainerMessageRuntime>(
        &mut self,
        player_id: i32,
        session_id: i32,
        session_extend_id: i32,
        shadow_position: u32,
        goods_id: CGuid,
        amount: u32,
        destination_extend_id: i32,
        destination_position: u32,
        context: &mut Context,
    ) -> Result<EnhancementTransferReport, EnhancementTransferBlock> {
        if !matches!(destination_extend_id, 1 | 2) {
            return Err(EnhancementTransferBlock::UnsupportedDestinationContainer {
                extend_id: destination_extend_id,
            });
        }
        let source = self
            .session_factory
            .equipment_session_shadow_original(session_id, session_extend_id, player_id, goods_id)
            .ok_or(EnhancementTransferBlock::MissingShadow)?;
        let recorded_position = self
            .session_factory
            .equipment_session_shadow_position(session_id, session_extend_id, player_id, goods_id)
            .ok_or(EnhancementTransferBlock::MissingShadow)?;
        if recorded_position != shadow_position
            || source.container_type != 400
            || source.container_id != player_id
        {
            return Err(EnhancementTransferBlock::MissingShadow);
        }
        if !matches!(source.container_extend_id, 1 | 2) {
            return Err(EnhancementTransferBlock::UnsupportedSourceContainer {
                extend_id: source.container_extend_id,
            });
        }

        let mut player = self
            .players
            .remove(&player_id)
            .ok_or(EnhancementTransferBlock::MissingPlayer)?;
        let result = (|| {
            let goods = match source.container_extend_id {
                1 => player.packet().get_goods(source.goods_position),
                2 => player.equipment().get_goods(source.goods_position),
                _ => unreachable!("source extend проверен выше"),
            }
            .filter(|goods| goods.identity().ex_id == goods_id && goods.amount() == amount)
            .ok_or(EnhancementTransferBlock::MissingSourceGoods)?;
            let goods_identity = goods.identity();
            let pack_add_enabled = self.globe_setup.pack_add_enabled();

            let (removal, mut incoming) = if source.container_extend_id == 1 {
                let removed = player
                    .packet_mut()
                    .remove_goods(goods_id)
                    .ok_or(EnhancementTransferBlock::PacketRemovalFailed)?;
                let removed = match removed {
                    VolumeGoodsRemoveOutcome::Removed(AmountLimitGoodsTaken::Removed(removed)) => {
                        removed
                    }
                    VolumeGoodsRemoveOutcome::Removed(AmountLimitGoodsTaken::Split(_))
                    | VolumeGoodsRemoveOutcome::RemovedButCellMissing(_) => {
                        return Err(EnhancementTransferBlock::PacketRemovalFailed);
                    }
                };
                (
                    EnhancementTransferRemoval::Packet {
                        owner_type: removed.owner_type,
                        owner_id: removed.owner_id,
                        position: removed.position.unwrap_or(source.goods_position),
                        amount: removed.amount,
                        listeners: removed.listeners,
                    },
                    Some(removed.goods),
                )
            } else {
                let remove_facts =
                    context.enhancement_equipment_remove_facts(&player, goods, pack_add_enabled);
                let mut recompute =
                    |player: &CPlayer| context.recompute_enhancement_player_properties(player);
                let mut report = player.remove_equipment_goods(
                    goods_id,
                    &self.goods_factory,
                    &self.skill_factory,
                    remove_facts,
                    &mut recompute,
                );
                drop(recompute);
                self.publish_player_equipment_remove_report(&mut report, context);
                let outcome = std::mem::replace(
                    &mut report.outcome,
                    EquipmentRemoveOutcome::Missing {
                        partial_effects: Default::default(),
                    },
                );
                let removed = match outcome {
                    EquipmentRemoveOutcome::Removed(removed) => removed,
                    outcome => {
                        report.outcome = outcome;
                        return Err(EnhancementTransferBlock::EquipmentRemovalFailed(report));
                    }
                };
                (
                    EnhancementTransferRemoval::Equipment {
                        event: removed.event,
                        effects: report.effects,
                        deliveries: report.deliveries,
                    },
                    Some(removed.goods),
                )
            };

            let shadow = self
                .session_factory
                .remove_equipment_session_shadow(session_id, session_extend_id, player_id, goods_id)
                .expect("session shadow проверен до удаления source goods")
                .removed;
            let delete_shadow_delivery =
                send_enhancement_shadow_deleted(self, player_id, goods_identity, &shadow);
            let destination = self.add_enhancement_transfer_goods(
                &mut player,
                destination_extend_id,
                destination_position,
                &mut incoming,
                pack_add_enabled,
                context,
            );
            let outcome = if incoming.is_none() {
                EnhancementTransferOutcome::Moved(destination)
            } else {
                let rejected = destination;
                let rollback = self.add_enhancement_transfer_goods(
                    &mut player,
                    source.container_extend_id,
                    source.goods_position,
                    &mut incoming,
                    pack_add_enabled,
                    context,
                );
                if incoming.is_none() {
                    EnhancementTransferOutcome::RolledBack {
                        rejected,
                        restored: rollback,
                    }
                } else {
                    let goods = incoming
                        .take()
                        .expect("неуспешный rollback сохраняет detached goods");
                    let notification_delivery = send_enhancement_goods_collected(
                        self,
                        player_id,
                        goods.name(),
                        goods.amount(),
                    );
                    EnhancementTransferOutcome::GoodsCollected {
                        rejected,
                        rollback,
                        goods: goods.identity(),
                        notification_delivery,
                    }
                }
            };
            Ok(EnhancementTransferReport {
                goods: goods_identity,
                source,
                removal,
                shadow,
                delete_shadow_delivery,
                outcome,
            })
        })();
        self.players.insert(player_id, player);
        result
    }

    fn add_enhancement_transfer_goods<Context: GameContainerMessageRuntime>(
        &self,
        player: &mut CPlayer,
        extend_id: i32,
        position: u32,
        incoming: &mut Option<CGoods>,
        pack_add_enabled: bool,
        context: &mut Context,
    ) -> EnhancementTransferAddition {
        if extend_id == 1 {
            let owner_progress_allows = player.current_progress() == PlayerProgress::None;
            return EnhancementTransferAddition::Packet(player.packet_mut().add_goods_at(
                position,
                incoming,
                &self.goods_factory,
                owner_progress_allows,
            ));
        }
        let add_facts = context.enhancement_equipment_add_facts(
            player,
            incoming
                .as_ref()
                .expect("destination/rollback add получает detached goods"),
            pack_add_enabled,
        );
        let mut report = {
            let context_cell = std::cell::RefCell::new(&mut *context);
            let mut register = |goods: &CGoods| {
                context_cell
                    .borrow_mut()
                    .register_enhancement_goods_ai(goods);
            };
            let mut recompute = |player: &CPlayer| {
                context_cell
                    .borrow_mut()
                    .recompute_enhancement_player_properties(player)
            };
            player.add_equipment_goods(
                position,
                incoming,
                &self.goods_factory,
                &self.skill_factory,
                add_facts,
                &mut register,
                &mut recompute,
            )
        };
        self.publish_player_equipment_add_report(&mut report, context);
        EnhancementTransferAddition::Equipment(report)
    }

    /// Создаёт достигнутый GameServer goods core; исходный код игнорировал
    /// ошибку `CoCreateGuid`, поэтому failure оставляет нулевой GUID.
    pub(crate) fn create_goods_core(&mut self, goods_index: u32) -> Option<CGoods> {
        let random_state = &mut self.random_state;
        self.goods_factory.create_goods_core(
            goods_index,
            |upper_bound| game_legacy_random(random_state, upper_bound),
            || CGuid::create().unwrap_or(CGuid::GUID_INVALID),
        )
    }

    /// Общий exact overload `CreateGoods(index, amount, vector)` с теми же
    /// Game RNG, GUID и fairy threshold owner-ами, что и остальные callers.
    pub(crate) fn create_goods_batch(&mut self, goods_index: u32, amount: u32) -> Vec<CGoods> {
        let (random_state, goods_factory, fairy_exp_conf, battle_fairy_exp_config) = (
            &mut self.random_state,
            &self.goods_factory,
            &self.fairy_exp_conf,
            &self.battle_fairy_exp_config,
        );
        let mut random = |upper_bound| game_legacy_random(random_state, upper_bound);
        goods_factory.create_goods_batch(
            goods_index,
            amount,
            &mut random,
            || CGuid::create().unwrap_or(CGuid::GUID_INVALID),
            |equip_level, level| fairy_exp_conf.dw_exp_up(equip_level, level),
            |equip_level, level| battle_fairy_exp_config.dw_exp_up(equip_level, level),
        )
    }

    /// Достигнутый `AddGoods(name, amount, upgrade, particular)` готовит весь
    /// batch до передачи packet owner-у. Upgrade использует тот же Game RNG,
    /// particular attribute пишет modifier первой instance-пары, как native
    /// `SetAddonPropertyModifier`.
    pub(crate) fn create_script_goods_batch(
        &mut self,
        goods_index: u32,
        amount: u32,
        upgrade_level: i32,
        particular_attribute: i32,
    ) -> Vec<CGoods> {
        let mut goods = self.create_goods_batch(goods_index, amount);
        let (factory, random_state) = (&self.goods_factory, &mut self.random_state);
        for item in &mut goods {
            if upgrade_level != 0 {
                let _ = factory.upgrade_equipment(item, upgrade_level, |upper_bound| {
                    game_legacy_random(random_state, upper_bound)
                });
            }
            if particular_attribute != 0 {
                let _ = item.set_addon_property_modifier_core(
                    GAP_PARTICULAR_ATTRIBUTE,
                    1,
                    particular_attribute,
                );
            }
        }
        goods
    }

    pub(crate) const fn skill_factory(&self) -> &CSkillFactory {
        &self.skill_factory
    }

    pub(crate) const fn skill_factory_mut(&mut self) -> &mut CSkillFactory {
        &mut self.skill_factory
    }

    pub(crate) fn add_remote_player_skill(
        &mut self,
        player_id: i32,
        name: &[u8],
        level: u16,
    ) -> Option<crate::gameserver::appserver::player::PlayerRemoteSkillMutation> {
        let (players, skill_factory) = (&mut self.players, &self.skill_factory);
        players
            .get_mut(&player_id)?
            .add_remote_skill(name, level, skill_factory)
    }

    pub(crate) fn delete_remote_player_skill(
        &mut self,
        player_id: i32,
        name: &[u8],
    ) -> Option<crate::gameserver::appserver::player::PlayerRemoteSkillMutation> {
        let (players, skill_factory) = (&mut self.players, &self.skill_factory);
        Some(
            players
                .get_mut(&player_id)?
                .delete_remote_skill(name, skill_factory),
        )
    }

    pub(crate) fn decode_monster_list(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<MonsterListDecodeReport, MonsterListDecodeError> {
        decode_monster_list(
            &mut self.monster_registry,
            &mut self.monster_drop_registry,
            source,
            cursor,
        )
    }

    pub(crate) fn find_monster_property_by_origin_name(
        &self,
        origin_name: &[u8],
    ) -> Option<&MonsterProperties> {
        get_monster_property_by_origin_name(&self.monster_registry, origin_name)
    }

    pub(crate) fn find_monster_property_by_origin_name_mut(
        &mut self,
        origin_name: &[u8],
    ) -> Option<&mut MonsterProperties> {
        get_monster_property_by_origin_name_mut(&mut self.monster_registry, origin_name)
    }

    /// Эквивалент `RefreashAllMonsterBaseProperty`: old raw pointers заменены
    /// key lookup-ами, поэтому проход подтверждает состояние всех live owners.
    pub(crate) fn refresh_all_monster_base_property(&self) -> MonsterBasePropertyRefreshReport {
        let mut report = MonsterBasePropertyRefreshReport::default();
        for region in self.regions.values() {
            for key in region.base().monster_base_property_keys() {
                report.monsters = report.monsters.wrapping_add(1);
                if self.find_monster_property_by_origin_name(key).is_some() {
                    report.resolved = report.resolved.wrapping_add(1);
                } else {
                    report.missing = report.missing.wrapping_add(1);
                }
            }
        }
        report
    }

    /// Сохраняет исходный `std::map::operator[] = pointer`: повторный ID
    /// заменяет опубликованный proxy-owner.
    pub(crate) fn add_proxy_region(&mut self, region: CProxyServerRegion) -> bool {
        self.proxy_regions.insert(region.get_id(), region).is_some()
    }

    pub(crate) fn find_proxy_region(&self, region_id: i32) -> Option<&CProxyServerRegion> {
        self.proxy_regions.get(&region_id)
    }

    pub(crate) fn find_proxy_region_mut(
        &mut self,
        region_id: i32,
    ) -> Option<&mut CProxyServerRegion> {
        self.proxy_regions.get_mut(&region_id)
    }

    pub(crate) fn take_war_startup_owners(&mut self) -> GameWarStartupOwners {
        GameWarStartupOwners {
            attack_city: std::mem::take(&mut self.attack_city_sys),
            village: std::mem::take(&mut self.village_war_sys),
            country: std::mem::take(&mut self.country_war_sys),
            four_nation: std::mem::take(&mut self.four_nation_war_sys),
        }
    }

    pub(crate) fn restore_war_startup_owners(&mut self, owners: GameWarStartupOwners) {
        self.attack_city_sys = owners.attack_city;
        self.village_war_sys = owners.village;
        self.country_war_sys = owners.country;
        self.four_nation_war_sys = owners.four_nation;
    }

    pub(crate) fn add_region(&mut self, mut region: ServerRegionOwner) -> bool {
        if let ServerRegionOwner::Nation(nation) = &mut region {
            nation.set_country_names(std::array::from_fn(|country| {
                self.globe_setup
                    .country_name(country as u8)
                    .unwrap_or_default()
                    .to_vec()
            }));
        }
        self.regions.insert(region.region_id(), region).is_some()
    }

    pub(crate) fn find_region(&self, region_id: i32) -> Option<&ServerRegionOwner> {
        self.regions.get(&region_id)
    }

    pub(crate) fn find_region_mut(&mut self, region_id: i32) -> Option<&mut ServerRegionOwner> {
        self.regions.get_mut(&region_id)
    }

    pub(crate) fn take_region_owner(&mut self, region_id: i32) -> Option<ServerRegionOwner> {
        self.regions.remove(&region_id)
    }

    pub(crate) fn restore_region_owner(&mut self, region: ServerRegionOwner) {
        let replaced = self.regions.insert(region.region_id(), region);
        debug_assert!(
            replaced.is_none(),
            "scoped region owner не заменяет live owner"
        );
    }

    /// Script function `9304 / kScriptFunctionNationWarSendPlayerId`:
    /// father-region берётся у current script player, а timing запускается
    /// для переданного player ID только в concrete local Nation owner.
    pub(crate) fn script_nation_war_send_player_id(
        &mut self,
        script_player_id: i32,
        player_id: i32,
        now_ms: impl FnOnce() -> u32,
    ) -> bool {
        let Some(region_id) = self
            .find_player(script_player_id)
            .and_then(CPlayer::server_region_id)
        else {
            return false;
        };
        self.start_nation_war_player_timing(region_id, player_id, now_ms)
    }

    /// Exact `ServerNationRegion::OnPlayerTimgingStart`, включая
    /// morale snapshot `0xBF818` до мутации timing record.
    pub(crate) fn start_nation_war_player_timing(
        &mut self,
        region_id: i32,
        player_id: i32,
        now_ms: impl FnOnce() -> u32,
    ) -> bool {
        let Some((country, can_start)) = self
            .find_player(player_id)
            .map(|player| (player.country(), player.can_start_nation_war_timing()))
        else {
            return false;
        };
        if !can_start {
            return false;
        }
        let Some((morale, failed)) = self.find_region(region_id).and_then(|region| match region {
            ServerRegionOwner::Nation(region) => Some((*region.morale(), *region.nation_failed())),
            _ => None,
        }) else {
            return false;
        };

        let snapshot = self.four_nation_morale_snapshot(morale, failed);
        let _delivery = snapshot.send_to_player(self.net_server(), player_id);

        let timing_exists = self
            .find_region(region_id)
            .and_then(|region| match region {
                ServerRegionOwner::Nation(region) => Some(region.has_player_timing(player_id)),
                _ => None,
            })
            .unwrap_or(false);
        let now_ms = now_ms();
        let country_figure = if timing_exists {
            0
        } else {
            self.country_handler_mut()
                .country_mut(country)
                .map(|country| country.identity_for_player(player_id))
                .unwrap_or_default()
        };
        let Some(ServerRegionOwner::Nation(region)) = self.find_region_mut(region_id) else {
            return false;
        };
        region.start_player_timing(
            player_id,
            i32::from(country),
            i32::from(country_figure),
            now_ms,
        );
        true
    }

    /// Caller-side `CPlayer::OnLost`: changing-server ветвь не закрывает
    /// nation clock; ordinary loss передаёт `died=false`.
    pub(crate) fn finish_nation_war_timing_on_player_lost(
        &mut self,
        player_id: i32,
        now_ms: impl FnOnce() -> u32,
    ) -> bool {
        let Some((region_id, changing_server)) = self
            .find_player(player_id)
            .and_then(|player| Some((player.server_region_id()?, player.in_changing_server())))
        else {
            return false;
        };
        if changing_server {
            return false;
        }
        self.finish_nation_war_player_timing(region_id, player_id, false, now_ms)
    }

    fn finish_nation_war_player_timing(
        &mut self,
        region_id: i32,
        player_id: i32,
        died: bool,
        now_ms: impl FnOnce() -> u32,
    ) -> bool {
        let Some(ServerRegionOwner::Nation(region)) = self.find_region_mut(region_id) else {
            return false;
        };
        region
            .finish_player_timing(player_id, died, now_ms)
            .is_some()
    }

    /// Script primitive `NationWar_CarriageBackTown` reaches the same typed
    /// Nation owner that combat callbacks use; non-Nation region keeps the
    /// original dynamic-cast no-op as `None`.
    pub(crate) fn script_nation_carriage_back_town(
        &mut self,
        region_id: i32,
        country: i32,
    ) -> Option<NationCarriageReturnReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        let outcome = region.carriage_back_town(country);
        let morale_delivery =
            matches!(outcome, NationCarriageReturnOutcome::Applied(_)).then(|| {
                self.four_nation_morale_snapshot(*region.morale(), *region.nation_failed())
                    .send_to_region(Some(&region.war.base), None, self)
            });
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(NationCarriageReturnReport {
            region_id,
            requested_country: country,
            outcome,
            morale_delivery,
        })
    }

    pub(crate) fn nation_enter_contend<Context: NationCombatContext>(
        &mut self,
        region_id: i32,
        player_id: i32,
        max_time: u32,
        context: &mut Context,
    ) -> Option<NationContendEnterReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        let mut deliveries = Vec::new();
        let mut state_deliveries = Vec::new();
        let outcome = match self.find_player(player_id) {
            None => NationContendEnterOutcome::PlayerMissing,
            Some(player) if !player.can_attack_nation_monster() => {
                NationContendEnterOutcome::PlayerUnavailable
            }
            Some(player) => {
                let country = player.country();
                if region.flag_belong_to_id() == i32::from(country) {
                    deliveries.push(
                        self.send_nation_player_notice(player_id, self.get_string_by_id(b"GS1130")),
                    );
                    NationContendEnterOutcome::CountryAlreadyOwnsSymbol
                } else if let Some(contender_player_id) =
                    region.contender_for_country(i32::from(country))
                {
                    if let Some(contender) = self.find_player(contender_player_id) {
                        let text = format_legacy_text_fields(
                            self.get_string_by_id(b"GS1131"),
                            &[contender.shape().base_object().get_name()],
                            0xff,
                        );
                        deliveries.push(self.send_nation_player_notice(player_id, &text));
                        NationContendEnterOutcome::CountryContenderExists {
                            contender_player_id,
                        }
                    } else {
                        self.finish_nation_contend_entry(
                            &mut region,
                            player_id,
                            country,
                            max_time,
                            context,
                            &mut deliveries,
                            &mut state_deliveries,
                        )
                    }
                } else {
                    self.finish_nation_contend_entry(
                        &mut region,
                        player_id,
                        country,
                        max_time,
                        context,
                        &mut deliveries,
                        &mut state_deliveries,
                    )
                }
            }
        };
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(NationContendEnterReport {
            region_id,
            player_id,
            outcome,
            deliveries,
            state_deliveries,
        })
    }

    pub(crate) fn nation_cancel_contend_by_player_id<Context: NationCombatContext>(
        &mut self,
        region_id: i32,
        player_id: i32,
        context: &mut Context,
    ) -> Option<NationContendCancelReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        let (outcome, delivery, state_delivery) = if self.find_player(player_id).is_none() {
            (None, None, None)
        } else {
            let outcome = region.cancel_contend_by_player_id(player_id);
            let (delivery, state_delivery) =
                if matches!(outcome, NationContendCancelOutcome::MissingReset) {
                    let state_delivery = self.set_nation_player_contend_state(
                        &region.war.base,
                        player_id,
                        false,
                        context,
                    );
                    (
                        Some(self.send_nation_contend_time(player_id, 0)),
                        state_delivery,
                    )
                } else {
                    (None, None)
                };
            (Some(outcome), delivery, state_delivery)
        };
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(NationContendCancelReport {
            region_id,
            player_id,
            outcome,
            delivery,
            state_delivery,
        })
    }

    pub(crate) fn nation_cancel_all_contenders<Context: NationCombatContext>(
        &mut self,
        region_id: i32,
        context: &mut Context,
    ) -> Option<NationContendCancelAllReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        let mut time_deliveries = Vec::new();
        let mut state_deliveries = Vec::new();
        for player_id in region.contender_player_ids() {
            time_deliveries.push(self.send_nation_contend_time(player_id, 0));
            if let Some(delivery) =
                self.set_nation_player_contend_state(&region.war.base, player_id, false, context)
            {
                state_deliveries.push((player_id, delivery));
            }
        }
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(NationContendCancelAllReport {
            time_deliveries,
            state_deliveries,
        })
    }

    fn finish_nation_contend_entry<Context: NationCombatContext>(
        &mut self,
        region: &mut ServerNationRegion,
        player_id: i32,
        country: u8,
        max_time: u32,
        context: &mut Context,
        deliveries: &mut Vec<i32>,
        state_deliveries: &mut Vec<Result<i32, ShapeCoordinateBlock>>,
    ) -> NationContendEnterOutcome {
        if matches!(
            region.cancel_contend_by_player_id(player_id),
            NationContendCancelOutcome::MissingReset
        ) {
            if let Some(delivery) =
                self.set_nation_player_contend_state(&region.war.base, player_id, false, context)
            {
                state_deliveries.push(delivery);
            }
            deliveries.push(self.send_nation_contend_time(player_id, 0));
        }
        let first_for_country = region.add_contend(
            player_id,
            i32::from(country),
            max_time as i32,
            context.now_milliseconds(),
        );
        if let Some(delivery) =
            self.set_nation_player_contend_state(&region.war.base, player_id, true, context)
        {
            state_deliveries.push(delivery);
        }
        deliveries.push(self.send_nation_contend_time(player_id, 0));
        if first_for_country && (1..=4).contains(&country) {
            let text = format_legacy_text_fields(
                self.get_string_by_id(b"GS1133"),
                &[region.country_name(country)],
                0xff,
            );
            deliveries.push(
                nation_colored_text_message(0xbf806, 0xffff_ffff, 0xffff_0000, &text)
                    .send_to_region(Some(&region.war.base), None, self),
            );
        }
        deliveries
            .push(self.send_nation_player_notice(player_id, self.get_string_by_id(b"GS1132")));
        NationContendEnterOutcome::Entered { first_for_country }
    }

    pub(crate) fn nation_contender_damaged(
        &mut self,
        region_id: i32,
        player_id: i32,
        damage: i32,
    ) -> Option<NationContendDamageReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        let mutation = match self.find_player(player_id) {
            Some(player) if player.can_attack_nation_monster() => region.damage_contender(
                player_id,
                damage,
                player.maximum_health(),
                self.globe_setup.contend_damage_time_factor(),
            ),
            _ => Ok(None),
        };
        let delivery = mutation.as_ref().ok().and_then(|mutation| {
            mutation.map(|mutation| self.send_nation_contend_time(player_id, mutation.percentage))
        });
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(NationContendDamageReport {
            region_id,
            player_id,
            mutation,
            delivery,
        })
    }

    pub(crate) fn nation_contend_ai<Context: NationContendContext>(
        &mut self,
        region_id: i32,
        context: &mut Context,
    ) -> Option<Result<NationContendAiReport, NationContendArithmeticBlock>> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        context.run_base_region_ai(&mut region.war.base);
        let mut magic_stone_transitions = Vec::new();
        for country in region.take_due_magic_stone_transitions() {
            magic_stone_transitions.push(self.replace_nation_magic_stone(
                &mut region,
                country,
                context,
            ));
        }
        let advance = match region.advance_contenders(context.now_milliseconds()) {
            Ok(advance) => advance.unwrap_or_default(),
            Err(error) => {
                self.restore_region_owner(ServerRegionOwner::Nation(region));
                return Some(Err(error));
            }
        };
        let mut progress_deliveries = Vec::with_capacity(advance.progress.len());
        for (player_id, percentage) in advance.progress {
            let delivery = self.send_nation_contend_time(player_id, percentage);
            progress_deliveries.push((player_id, percentage, delivery));
        }

        let completed = advance.completed;
        let mut completion_outcome = None;
        let mut completion_deliveries = Vec::new();
        let mut state_deliveries = Vec::new();
        let mut top_info_delivery = None;
        let mut treasure_spawns = Vec::new();
        if let Some(contender) = completed {
            completion_deliveries.push(self.send_nation_contend_time(contender.player_id, 100));
            context.add_log_text(self.get_string_by_id(b"GS1072"));
            completion_outcome = Some(match self.find_player(contender.player_id) {
                None => NationContendCompletionOutcome::PlayerMissing,
                Some(player) if !player.can_attack_nation_monster() => {
                    NationContendCompletionOutcome::PlayerUnavailable
                }
                Some(player) => {
                    let country = player.country();
                    match region.capture_contend_symbol(country) {
                        None => NationContendCompletionOutcome::CountryOutsideNation,
                        Some(capture) => {
                            for cancelled_player_id in &capture.cancelled_player_ids {
                                completion_deliveries
                                    .push(self.send_nation_contend_time(*cancelled_player_id, 0));
                                if let Some(delivery) = self.set_nation_player_contend_state(
                                    &region.war.base,
                                    *cancelled_player_id,
                                    false,
                                    context,
                                ) {
                                    state_deliveries.push((*cancelled_player_id, delivery));
                                }
                            }
                            context.add_log_text(self.get_string_by_id(b"GS1073"));
                            completion_deliveries.push(
                                self.four_nation_morale_snapshot(
                                    *region.morale(),
                                    *region.nation_failed(),
                                )
                                .send_to_region(
                                    Some(&region.war.base),
                                    None,
                                    self,
                                ),
                            );
                            completion_deliveries.push(
                                nation_colored_text_message(
                                    0xbf806,
                                    0xffff_ffff,
                                    0xffff_0000,
                                    &format_legacy_text_fields(
                                        self.get_string_by_id(b"GS1082"),
                                        &[region.country_name(country)],
                                        0xff,
                                    ),
                                )
                                .send_to_region(
                                    Some(&region.war.base),
                                    None,
                                    self,
                                ),
                            );
                            let mut top = CMessage::new(0xbf804);
                            top.add_long(0);
                            top.add_long(-1);
                            top.add_long(1);
                            top.add_long(1);
                            add_legacy_c_string(
                                top.base_mut(),
                                &format_legacy_text_fields(
                                    self.get_string_by_id(b"GS1083"),
                                    &[region.country_name(country)],
                                    0xff,
                                ),
                            );
                            top_info_delivery = Some(top.send_all(self.current_net_server()));
                            for (name_id, script, x, y) in [
                                (
                                    b"GS1025".as_slice(),
                                    b"scripts/npc/npc_siguobaoxiang_01.script".as_slice(),
                                    0xfb,
                                    0x102,
                                ),
                                (
                                    b"GS1190".as_slice(),
                                    b"scripts/npc/npc_siguobaoxiang_02.script".as_slice(),
                                    0xf6,
                                    0xfd,
                                ),
                                (
                                    b"GS1189".as_slice(),
                                    b"scripts/npc/npc_siguobaoxiang_03.script".as_slice(),
                                    0xfb,
                                    0xf9,
                                ),
                            ] {
                                treasure_spawns.push(self.spawn_nation_treasure_box(
                                    &mut region,
                                    name_id,
                                    script,
                                    x,
                                    y,
                                    context,
                                ));
                            }
                            NationContendCompletionOutcome::Captured(capture)
                        }
                    }
                }
            });
            region.clear_contenders();
        }
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(Ok(NationContendAiReport {
            region_id,
            magic_stone_transitions,
            progress_deliveries,
            completed,
            completion_outcome,
            completion_deliveries,
            state_deliveries,
            top_info_delivery,
            treasure_spawns,
        }))
    }

    fn replace_nation_magic_stone<Context: NationContendContext>(
        &self,
        region: &mut ServerNationRegion,
        country: u8,
        context: &mut Context,
    ) -> NationMagicStoneTransitionReport {
        let (npc_name_id, monster_name_id, tile_x, tile_y) = match country {
            1 => (b"GS1084".as_slice(), b"GS1142".as_slice(), 0xfb, 0x35),
            2 => (b"GS1085".as_slice(), b"GS1139".as_slice(), 0xf8, 0x1c1),
            3 => (b"GS1086".as_slice(), b"GS1140".as_slice(), 0x25, 0xfc),
            4 => (b"GS1087".as_slice(), b"GS1141".as_slice(), 0x1dc, 0x105),
            _ => {
                return NationMagicStoneTransitionReport {
                    country,
                    npc_id: None,
                    explosion_delivery: None,
                    removal_delivery: None,
                    outcome: NationMagicStoneTransitionOutcome::NpcMissing,
                };
            }
        };
        let npc_name = self.get_string_by_id(npc_name_id);
        let npc = match region.war.base.find_npc_by_name(npc_name) {
            Ok(Some(npc)) => npc,
            Ok(None) => {
                return NationMagicStoneTransitionReport {
                    country,
                    npc_id: None,
                    explosion_delivery: None,
                    removal_delivery: None,
                    outcome: NationMagicStoneTransitionOutcome::NpcMissing,
                };
            }
            Err(block) => {
                return NationMagicStoneTransitionReport {
                    country,
                    npc_id: None,
                    explosion_delivery: None,
                    removal_delivery: None,
                    outcome: NationMagicStoneTransitionOutcome::NpcNameAmbiguous {
                        matches: block.matches,
                    },
                };
            }
        };
        let shape = npc.move_shape().shape();
        let npc_id = shape.identity().id;
        let npc_tile_x = match shape.get_tile_x() {
            Ok(tile_x) => tile_x,
            Err(error) => {
                return NationMagicStoneTransitionReport {
                    country,
                    npc_id: Some(npc_id),
                    explosion_delivery: None,
                    removal_delivery: None,
                    outcome: NationMagicStoneTransitionOutcome::NpcCoordinateBlocked(error),
                };
            }
        };
        let npc_tile_y = match shape.get_tile_y() {
            Ok(tile_y) => tile_y,
            Err(error) => {
                return NationMagicStoneTransitionReport {
                    country,
                    npc_id: Some(npc_id),
                    explosion_delivery: None,
                    removal_delivery: None,
                    outcome: NationMagicStoneTransitionOutcome::NpcCoordinateBlocked(error),
                };
            }
        };

        let mut explosion = CMessage::new(0xbf50a);
        explosion.add_long(2_000_000);
        explosion
            .base_mut()
            .add(&(npc_tile_x as f32 + 0.5).to_le_bytes());
        explosion
            .base_mut()
            .add(&(npc_tile_y as f32 + 0.5).to_le_bytes());
        let explosion_delivery =
            Some(context.send_nation_magic_stone_around(&region.war.base, shape, &explosion));

        let identity = shape.identity();
        let mut removal = CMessage::new(0xbf504);
        removal.add_long(identity.object_type);
        removal.add_long(identity.id);
        removal.add_long(0);
        let removal_delivery =
            Some(context.send_nation_magic_stone_around(&region.war.base, shape, &removal));

        if let Err(error) = region.war.base.remove_owned_npc_by_id(npc_id) {
            return NationMagicStoneTransitionReport {
                country,
                npc_id: Some(npc_id),
                explosion_delivery,
                removal_delivery,
                outcome: NationMagicStoneTransitionOutcome::NpcRemovalBlocked(error),
            };
        }

        let monster_name = self.get_string_by_id(monster_name_id);
        let Some(property) = self
            .find_monster_property_by_origin_name(monster_name)
            .cloned()
        else {
            return NationMagicStoneTransitionReport {
                country,
                npc_id: Some(npc_id),
                explosion_delivery,
                removal_delivery,
                outcome: NationMagicStoneTransitionOutcome::MonsterPropertyMissing,
            };
        };
        let (area_width, area_height) = self.area_dimensions();
        let outcome = match region.war.base.add_monster(
            &property,
            tile_x,
            tile_y,
            -1,
            true,
            false,
            context.now_milliseconds(),
            area_width,
            area_height,
            context,
        ) {
            Ok(monster_id) => NationMagicStoneTransitionOutcome::Spawned { monster_id },
            Err(error) => NationMagicStoneTransitionOutcome::MonsterSpawnBlocked(error),
        };
        NationMagicStoneTransitionReport {
            country,
            npc_id: Some(npc_id),
            explosion_delivery,
            removal_delivery,
            outcome,
        }
    }

    fn spawn_nation_treasure_box<Context: NationCombatContext>(
        &self,
        region: &mut ServerNationRegion,
        name_id: &[u8],
        script: &[u8],
        x: i32,
        y: i32,
        context: &mut Context,
    ) -> Result<ServerRegionNpcSpawnReport, ServerRegionNpcSpawnBlock> {
        let setup = ServerRegionNpcSetup {
            show_list: true,
            picture_id: 0x104,
            left: x,
            top: y,
            right: x,
            bottom: y,
            count: 1,
            direction: -1,
            time: 3_600_000,
            name: self.get_string_by_id(name_id).to_vec(),
            script: script.to_vec(),
        };
        let (area_width, area_height) = self.area_dimensions();
        region.war.base.add_npc(
            &setup,
            true,
            true,
            context.now_milliseconds(),
            area_width,
            area_height,
            context,
        )
    }

    fn set_nation_player_contend_state<Context: NationCombatContext>(
        &mut self,
        region: &CServerRegion,
        player_id: i32,
        contend_state: bool,
        context: &mut Context,
    ) -> Option<Result<i32, ShapeCoordinateBlock>> {
        let player = self.find_player_mut(player_id)?;
        if !player.set_contend_state(contend_state) {
            return None;
        }
        let mut message = CMessage::new(0xbff28);
        message.add_long(player_id);
        message.add_byte(u8::from(contend_state));
        let player = self
            .find_player(player_id)
            .expect("player сохранён между mutation и synchronous around-send");
        Some(context.send_nation_player_around(region, player.shape(), None, &message))
    }

    /// Concrete `CPlayer::SetContendState` effect для war-symbol entry:
    /// unchanged state ничего не публикует, mutation идёт до exact `0xBFF28`
    /// вокруг текущей позиции player-а.
    pub(crate) fn publish_war_player_contend_state(
        &mut self,
        region: &CServerRegion,
        player_id: i32,
        contend_state: bool,
    ) -> Option<Result<i32, ShapeCoordinateBlock>> {
        let player = self.find_player_mut(player_id)?;
        if !player.set_contend_state(contend_state) {
            return None;
        }
        let mut message = CMessage::new(0xbff28);
        message.add_long(player_id);
        message.add_byte(u8::from(contend_state));
        let player = self
            .find_player(player_id)
            .expect("war contender сохранён до synchronous around-send");
        let Some(runtime) = GameServerAroundRuntime::new(
            self,
            &self.session_factory,
            self.globe_setup.area_width(),
            self.globe_setup.area_height(),
        ) else {
            return Some(Ok(0));
        };
        Some(message.send_to_around(Some(region), player.shape(), None, &runtime))
    }

    fn publish_player_died_state<Context: NationCombatContext>(
        &mut self,
        region: &CServerRegion,
        player_id: i32,
        state: bool,
        context: &mut Context,
    ) -> Option<NationPlayerDiedStatePublication> {
        self.find_player_mut(player_id)?
            .set_city_war_died_state(state);
        let mut message = CMessage::new(0xbff2a);
        message.add_long(player_id);
        message.add_byte(u8::from(state));
        let self_delivery = message.send_to_player(self.net_server(), player_id);
        let player = self
            .find_player(player_id)
            .expect("player сохранён между self и synchronous around-send");
        let around_delivery =
            context.send_nation_player_around(region, player.shape(), Some(player_id), &message);
        Some(NationPlayerDiedStatePublication {
            player_id,
            state,
            self_delivery,
            around_delivery,
        })
    }

    fn set_player_died_state_time(&mut self, player_id: i32, time_ms: i32) -> Option<i32> {
        let player = self.find_player_mut(player_id)?;
        if !player.set_city_war_died_state_time_ms(time_ms) {
            return None;
        }
        let mut message = CMessage::new(0xbff2b);
        message.add_long(player_id);
        message.add_long(time_ms);
        Some(message.send_to_player(self.net_server(), player_id))
    }

    /// Полный достигнутый Nation-prefix `CPlayer::OnDied`: закрывает active
    /// war clock, исполняет virtual contend cancel, затем устанавливает
    /// половину global death penalty без setter-wire.
    pub(crate) fn player_died_in_nation_region<
        Context: NationCombatContext + RealmAppellationScriptContext,
    >(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> Option<NationPlayerDeathReport> {
        self.change_body_after_player_death(player_id, context);
        let region_id = self.find_player(player_id)?.server_region_id()?;
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        let timing_finished = region
            .finish_player_timing(player_id, true, || context.now_milliseconds())
            .is_some();
        let was_contending = self
            .find_player(player_id)
            .is_some_and(CPlayer::contend_state);
        let mut contend_state_delivery = None;
        let mut contend_time_delivery = None;
        let mut notice_delivery = None;
        let contend_outcome = if !was_contending {
            NationPlayerDeathContendOutcome::NotContending
        } else {
            match region.cancel_contend_by_player_id(player_id) {
                NationContendCancelOutcome::MissingReset => {
                    contend_state_delivery = self.set_nation_player_contend_state(
                        &region.war.base,
                        player_id,
                        false,
                        context,
                    );
                    contend_time_delivery = Some(self.send_nation_contend_time(player_id, 0));
                    notice_delivery = Some(
                        self.send_nation_player_notice(player_id, self.get_string_by_id(b"GS0136")),
                    );
                    NationPlayerDeathContendOutcome::MissingReset
                }
                NationContendCancelOutcome::Removed { .. } => {
                    NationPlayerDeathContendOutcome::RemovedLegacyReturnIndeterminate
                }
            }
        };
        let died_state_time_ms = self
            .globe_setup
            .died_state_time_seconds()
            .wrapping_mul(1000)
            / 2;
        let died_state_start_time_ms = (died_state_time_ms > 0).then(|| context.now_milliseconds());
        if let Some(player) = self.find_player_mut(player_id) {
            player.begin_city_war_death_countdown(
                died_state_time_ms,
                died_state_start_time_ms.unwrap_or_default(),
            );
        }
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(NationPlayerDeathReport {
            region_id,
            player_id,
            timing_finished,
            contend_outcome,
            contend_state_delivery,
            contend_time_delivery,
            notice_delivery,
            died_state_time_ms,
            died_state_start_time_ms,
        })
    }

    /// Две достижимые `OnRelive` ветви сходятся в этом exact tail: positive
    /// remaining time активирует state и публикует self, затем around.
    pub(crate) fn publish_nation_died_state_after_relive<Context: NationCombatContext>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> Option<NationPlayerDiedStatePublication> {
        let player = self.find_player(player_id)?;
        if player.city_war_died_state_time_ms() <= 0 {
            return None;
        }
        let region_id = player.server_region_id()?;
        let owner = self.take_region_owner(region_id)?;
        let publication = self.publish_player_died_state(owner.base(), player_id, true, context);
        self.restore_region_owner(owner);
        publication
    }

    /// Exact death-state tail `CPlayer::PeriodicalUpdate`: `timeGetTime`
    /// читается всегда, threshold строго `>1000`, DWORD elapsed wraps, а
    /// сравнение выполняется после signed cast.
    pub(crate) fn periodical_update_nation_died_state<Context: NationCombatContext>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> Option<NationPlayerDiedStateTick> {
        let now_ms = context.now_milliseconds();
        let player = self.find_player(player_id)?;
        let time_ms = player.city_war_died_state_time_ms();
        if time_ms <= 0 {
            return Some(NationPlayerDiedStateTick::Inactive);
        }
        let elapsed_ms = now_ms.wrapping_sub(player.died_state_start_time_ms());
        if elapsed_ms <= 1000 {
            return Some(NationPlayerDiedStateTick::Waiting { elapsed_ms });
        }
        self.find_player_mut(player_id)?
            .restart_died_state_clock(now_ms);
        if (elapsed_ms as i32) < time_ms {
            let remaining_ms = time_ms.wrapping_sub(elapsed_ms as i32);
            let time_delivery = self.set_player_died_state_time(player_id, remaining_ms);
            return Some(NationPlayerDiedStateTick::Advanced {
                elapsed_ms,
                remaining_ms,
                time_delivery,
            });
        }
        let time_delivery = self.set_player_died_state_time(player_id, 0);
        let region_id = self.find_player(player_id)?.server_region_id()?;
        let owner = self.take_region_owner(region_id)?;
        let state_publication =
            self.publish_player_died_state(owner.base(), player_id, false, context);
        self.restore_region_owner(owner);
        let state_publication = state_publication?;
        Some(NationPlayerDiedStateTick::Expired {
            elapsed_ms,
            time_delivery,
            state_publication,
        })
    }

    fn send_nation_contend_time(&self, player_id: i32, percentage: i32) -> i32 {
        let mut message = CMessage::new(0xbff29);
        message.add_long(percentage);
        message.send_to_player(self.net_server(), player_id)
    }

    fn send_nation_player_notice(&self, player_id: i32, text: &[u8]) -> i32 {
        nation_colored_text_message(0xbf806, 0xffff_ffff, 0xffff_0000, text)
            .send_to_player(self.net_server(), player_id)
    }

    /// Reached `CMonster::OnBeenHurted` branch: только player damage (`400`)
    /// вызывает Nation first-hit owner.
    pub(crate) fn monster_on_been_hurted(
        &mut self,
        region_id: i32,
        monster_id: i32,
        attacker_type: i32,
        attacker_id: i32,
    ) -> Option<NationMonsterDamageReport> {
        (attacker_type == 400)
            .then(|| self.nation_monster_damaged(region_id, monster_id, attacker_id))?
    }

    pub(crate) fn nation_monster_damaged(
        &mut self,
        region_id: i32,
        monster_id: i32,
        attacker_player_id: i32,
    ) -> Option<NationMonsterDamageReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };

        let outcome = match region.war.base.find_monster_by_id(monster_id) {
            None => NationMonsterDamageOutcome::MonsterMissing,
            Some(monster) if !monster.can_trigger_nation_damage() => {
                NationMonsterDamageOutcome::MonsterUnavailable
            }
            Some(monster) => {
                let Some(race) = monster
                    .base_property_key()
                    .and_then(|key| self.find_monster_property_by_origin_name(key))
                    .map(|property| property.race)
                else {
                    self.restore_region_owner(ServerRegionOwner::Nation(region));
                    return Some(NationMonsterDamageReport {
                        region_id,
                        monster_id,
                        attacker_player_id,
                        outcome: NationMonsterDamageOutcome::MonsterPropertyMissing,
                        world_delivery: None,
                    });
                };
                let monster_original_name = monster.original_name().to_vec();
                let Some(attacker) = self.find_player(attacker_player_id) else {
                    self.restore_region_owner(ServerRegionOwner::Nation(region));
                    return Some(NationMonsterDamageReport {
                        region_id,
                        monster_id,
                        attacker_player_id,
                        outcome: NationMonsterDamageOutcome::AttackerMissing,
                        world_delivery: None,
                    });
                };
                if !attacker.can_attack_nation_monster() {
                    NationMonsterDamageOutcome::AttackerUnavailable
                } else if u32::from(attacker.country()) == race {
                    NationMonsterDamageOutcome::SameCountry
                } else {
                    match u8::try_from(race).ok().and_then(|defender_country| {
                        region.register_monster_first_hit(
                            &monster_original_name,
                            defender_country,
                            attacker.country(),
                            |id| self.get_string_by_id(id).to_vec(),
                        )
                    }) {
                        Some(notice) => NationMonsterDamageOutcome::Notice(notice),
                        None if !(1..=4).contains(&race) => {
                            NationMonsterDamageOutcome::CountryOutsideNation
                        }
                        None => NationMonsterDamageOutcome::NoFirstHitNotice,
                    }
                }
            }
        };
        let world_delivery = match outcome {
            NationMonsterDamageOutcome::Notice(notice) => Some(
                self.nation_first_hit_notice_message(&region, notice)
                    .send(self, false),
            ),
            _ => None,
        };
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(NationMonsterDamageReport {
            region_id,
            monster_id,
            attacker_player_id,
            outcome,
            world_delivery,
        })
    }

    /// Reached `CMonster::OnDied` Nation callback. Context оставляет снаружи
    /// только уже существующие spatial/AI callbacks полноценного `AddNpc` и
    /// process log sinks; state и все client/World packets исполняются здесь.
    pub(crate) fn monster_on_died<Context: NationCombatContext>(
        &mut self,
        region_id: i32,
        monster_id: i32,
        killer_player_id: i32,
        context: &mut Context,
    ) -> Option<NationMonsterDeathReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        if region.war.base.find_monster_by_id(monster_id).is_some() {
            context.add_log_text(b"ServerNationRegion::OnMonsterDie");
        }
        let outcome = if !region
            .war
            .base
            .registered_player_ids()
            .contains(&killer_player_id)
        {
            NationMonsterDeathOutcome::KillerMissing
        } else {
            match region.war.base.find_monster_by_id(monster_id) {
                None => NationMonsterDeathOutcome::MonsterMissing,
                Some(monster) => {
                    let Some(race) = monster
                        .base_property_key()
                        .and_then(|key| self.find_monster_property_by_origin_name(key))
                        .map(|property| property.race)
                    else {
                        self.restore_region_owner(ServerRegionOwner::Nation(region));
                        return Some(NationMonsterDeathReport {
                            region_id,
                            monster_id,
                            killer_player_id,
                            outcome: NationMonsterDeathOutcome::MonsterPropertyMissing,
                            morale_delivery: None,
                            first_guard_delivery: None,
                            nation_fail_deliveries: Vec::new(),
                            yu_ying_shi_spawns: Vec::new(),
                        });
                    };
                    let target = classify_nation_morale_target(monster.original_name(), |id| {
                        self.get_string_by_id(id).to_vec()
                    });
                    match target {
                        None => NationMonsterDeathOutcome::Unclassified,
                        Some(target) => {
                            let attacker_country = self
                                .find_player(killer_player_id)
                                .expect("killer проверен в Nation m_vPlayers")
                                .country();
                            match u8::try_from(race).ok().and_then(|defender_country| {
                                region.apply_monster_morale(
                                    target,
                                    defender_country,
                                    attacker_country,
                                )
                            }) {
                                Some(mutation) => {
                                    NationMonsterDeathOutcome::MoraleChanged(mutation)
                                }
                                None => NationMonsterDeathOutcome::CountryOutsideNation,
                            }
                        }
                    }
                }
            }
        };
        if matches!(outcome, NationMonsterDeathOutcome::KillerMissing) {
            context.put_debug_string(self.get_string_by_id(b"GS1128"));
        }

        let mut first_guard_delivery = None;
        let mut nation_fail_deliveries = Vec::new();
        let mut yu_ying_shi_spawns = Vec::new();
        if let NationMonsterDeathOutcome::MoraleChanged(mutation) = outcome {
            if mutation.first_guard_notice {
                let defender = region.country_name(mutation.defender_country);
                let attacker = region.country_name(mutation.attacker_country);
                let text = format_legacy_text_fields(
                    self.get_string_by_id(b"GS1121"),
                    &[defender, attacker, attacker],
                    0xff,
                );
                first_guard_delivery = Some(
                    nation_colored_text_message(0xbf806, 0xffff_ffff, 0xffff_0000, &text)
                        .send_to_region(Some(&region.war.base), None, self),
                );
            }

            if mutation.check_morale_spawn
                && region.mark_yu_ying_shi_due_to_morale(mutation.defender_country)
            {
                yu_ying_shi_spawns.push(self.spawn_nation_yu_ying_shi(
                    &mut region,
                    mutation.defender_country,
                    mutation.attacker_country,
                    context,
                ));
            }
            if mutation.check_admiral_spawn
                && region.mark_yu_ying_shi_due_to_admiral(mutation.defender_country)
            {
                yu_ying_shi_spawns.push(self.spawn_nation_yu_ying_shi(
                    &mut region,
                    mutation.defender_country,
                    mutation.defender_country,
                    context,
                ));
            }

            if mutation.nation_failed {
                let defender = region.country_name(mutation.defender_country);
                let attacker = region.country_name(mutation.attacker_country);
                let (template, arguments): (&[u8], &[&[u8]]) =
                    if mutation.defender_country == mutation.attacker_country {
                        (self.get_string_by_id(b"GS1122"), &[defender])
                    } else {
                        (self.get_string_by_id(b"GS1123"), &[defender, attacker])
                    };
                let mut text = format_legacy_text_fields(template, arguments, 0xff);
                if region.treasure_box_count(mutation.defender_country) != 0 {
                    text.extend_from_slice(legacy_c_string_prefix(
                        self.get_string_by_id(b"GS1124"),
                    ));
                    text.truncate(0xff);
                }
                nation_fail_deliveries.push(nation_world_notice_message(&text).send(self, false));
                let mut failure = CMessage::new(0x6031d);
                failure.add_long(i32::from(mutation.defender_country));
                failure.add_long(i32::from(mutation.attacker_country));
                nation_fail_deliveries.push(failure.send(self, false));
            }

            context.add_log_text(self.get_string_by_id(b"GS1129"));
        }
        let morale_delivery =
            matches!(outcome, NationMonsterDeathOutcome::MoraleChanged(_)).then(|| {
                self.four_nation_morale_snapshot(*region.morale(), *region.nation_failed())
                    .send_to_region(Some(&region.war.base), None, self)
            });
        self.restore_region_owner(ServerRegionOwner::Nation(region));
        Some(NationMonsterDeathReport {
            region_id,
            monster_id,
            killer_player_id,
            outcome,
            morale_delivery,
            first_guard_delivery,
            nation_fail_deliveries,
            yu_ying_shi_spawns,
        })
    }

    fn nation_first_hit_notice_message(
        &self,
        region: &ServerNationRegion,
        notice: NationMonsterDamageNotice,
    ) -> CMessage {
        let (defender_country, attacker_country, template_id, stone) = match notice {
            NationMonsterDamageNotice::StoneGuard {
                defender_country,
                attacker_country,
            } => (
                defender_country,
                attacker_country,
                b"GS1125".as_slice(),
                true,
            ),
            NationMonsterDamageNotice::JinWeiJun {
                defender_country,
                attacker_country,
            } => (
                defender_country,
                attacker_country,
                b"GS1126".as_slice(),
                false,
            ),
            NationMonsterDamageNotice::MagicStone {
                defender_country,
                attacker_country,
            } => (
                defender_country,
                attacker_country,
                b"GS1127".as_slice(),
                false,
            ),
        };
        let defender = region.country_name(defender_country);
        let attacker = region.country_name(attacker_country);
        let arguments: &[&[u8]] = if stone {
            &[defender, attacker]
        } else {
            &[attacker, defender, defender]
        };
        let text = format_legacy_text_fields(self.get_string_by_id(template_id), arguments, 0xff);
        if stone {
            let mut message = CMessage::new(0x5fd09);
            message.add_byte(1);
            message.add_byte(defender_country);
            add_legacy_c_string(message.base_mut(), &text);
            message
        } else {
            nation_world_notice_message(&text)
        }
    }

    fn spawn_nation_yu_ying_shi<Context: NationCombatContext>(
        &self,
        region: &mut ServerNationRegion,
        country: u8,
        notify_country: u8,
        context: &mut Context,
    ) -> NationYuYingShiSpawnReport {
        let (script, x, y, coordinates): (&[u8], i32, i32, &[u8]) = match country {
            1 => (
                b"scripts/npc/npc_siguoyuyingshi_11000.script",
                214,
                71,
                b"[214,71]",
            ),
            2 => (
                b"scripts/npc/npc_siguoyuyingshi_12000.script",
                214,
                434,
                b"[214,434]",
            ),
            3 => (
                b"scripts/npc/npc_siguoyuyingshi_13000.script",
                50,
                217,
                b"[50,217]",
            ),
            4 => (
                b"scripts/npc/npc_siguoyuyingshi_14000.script",
                461,
                290,
                b"[461,290]",
            ),
            _ => unreachable!("YuYingShi gate принимает только страны 1..=4"),
        };
        let setup = ServerRegionNpcSetup {
            show_list: true,
            picture_id: 0x207,
            left: x,
            top: y,
            right: x,
            bottom: y,
            count: 1,
            direction: -1,
            time: 3_600_000,
            name: self.get_string_by_id(b"GS1136").to_vec(),
            script: script.to_vec(),
        };
        let (area_width, area_height) = self.area_dimensions();
        let spawn = region.war.base.add_npc(
            &setup,
            true,
            true,
            context.now_milliseconds(),
            area_width,
            area_height,
            context,
        );
        if spawn.is_err() {
            return NationYuYingShiSpawnReport {
                country,
                notify_country,
                spawn,
                region_delivery: None,
                country_deliveries: Vec::new(),
            };
        }

        let country_name = region.country_name(country);
        let region_text = format_legacy_text_fields(
            self.get_string_by_id(b"GS1137"),
            &[country_name, country_name],
            0xff,
        );
        let region_delivery = Some(
            nation_colored_text_message(0xbf806, 0xffff_ffff, 0xffa2_44ff, &region_text)
                .send_to_region(Some(&region.war.base), None, self),
        );
        let country_text =
            format_legacy_text_fields(self.get_string_by_id(b"GS1138"), &[coordinates], 0x7f);
        let country_message =
            nation_colored_text_message(0xbf811, 0xffff_ff00, 0xff00_0000, &country_text);
        let country_deliveries = (0..3)
            .map(|_| {
                country_message.send_to_region_contry_player(
                    Some(&region.war.base),
                    i32::from(notify_country),
                    self,
                )
            })
            .collect();
        NationYuYingShiSpawnReport {
            country,
            notify_country,
            spawn,
            region_delivery,
            country_deliveries,
        }
    }

    fn four_nation_morale_snapshot(&self, morale: [i32; 5], failed: [bool; 5]) -> CMessage {
        let colors = [0xfffc_0000, 0xffd1_eefe, 0xffe2_bf18, 0xff78_fcdb];
        let string_ids: [[&[u8]; 2]; 4] = [
            [b"GS1075", b"GS1074"],
            [b"GS1077", b"GS1076"],
            [b"GS1079", b"GS1078"],
            [b"GS1081", b"GS1080"],
        ];
        let mut snapshot = CMessage::new(0xbf818);
        for index in 0..4 {
            snapshot.base_mut().add_ulong(colors[index]);
            let text = format_single_legacy_i32(
                self.get_string_by_id(string_ids[index][usize::from(failed[index + 1])]),
                morale[index + 1],
                0xff,
            );
            let text = CString::new(text).expect("localized morale text обрезан до NUL");
            snapshot.base_mut().add_str(Some(&text));
        }
        snapshot
    }

    pub(crate) fn find_region_by_name(&self, name: &[u8]) -> Option<&ServerRegionOwner> {
        let end = name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(name.len());
        self.regions
            .values()
            .find(|region| region.name() == &name[..end])
    }

    pub(crate) const fn set_initial_region_totals(&mut self, monsters: i32, npcs: i32) {
        self.initial_total_monsters = monsters;
        self.initial_total_npcs = npcs;
    }

    pub(crate) const fn initial_region_totals(&self) -> (i32, i32) {
        (self.initial_total_monsters, self.initial_total_npcs)
    }

    pub(crate) const fn thing_setup(&self) -> &CThingSetup {
        &self.thing_setup
    }

    pub(crate) const fn thing_setup_mut(&mut self) -> &mut CThingSetup {
        &mut self.thing_setup
    }

    pub(crate) const fn increment_shop_list(&self) -> &CIncrementShopList {
        &self.increment_shop_list
    }

    pub(crate) const fn increment_shop_list_mut(&mut self) -> &mut CIncrementShopList {
        &mut self.increment_shop_list
    }

    pub(crate) const fn contribute_setup_mut(&mut self) -> &mut CContributeSetup {
        &mut self.contribute_setup
    }

    pub(crate) const fn log_system_mut(&mut self) -> &mut CLogSystem {
        &mut self.log_system
    }

    pub(crate) const fn log_system(&self) -> &CLogSystem {
        &self.log_system
    }

    pub(crate) const fn gm_list_mut(&mut self) -> &mut CGMList {
        &mut self.gm_list
    }

    pub(crate) const fn gm_list(&self) -> &CGMList {
        &self.gm_list
    }

    pub(crate) const fn da_kong_xiang_qian_mut(&mut self) -> &mut CDaKongXiangQian {
        &mut self.da_kong_xiang_qian
    }

    pub(crate) const fn globe_setup(&self) -> &GlobeSetupSnapshot {
        &self.globe_setup
    }

    pub(crate) const fn region_router(&self) -> &RegionRouter {
        &self.region_router
    }

    pub(crate) const fn globe_setup_and_region_router_mut(
        &mut self,
    ) -> (&mut GlobeSetupSnapshot, &mut RegionRouter) {
        (&mut self.globe_setup, &mut self.region_router)
    }

    pub(crate) const fn area_dimensions(&self) -> (i32, i32) {
        (self.area_width, self.area_height)
    }

    pub(crate) const fn set_area_dimensions(&mut self, width: i32, height: i32) {
        self.area_width = width;
        self.area_height = height;
    }

    pub(crate) const fn auction_now(&self) -> bool {
        self.auction_now
    }

    pub(crate) const fn auction_last_check_seconds(&self) -> u32 {
        self.auction_last_check_seconds
    }

    pub(crate) const fn auction_room(&self) -> &CGameAuctionRoom {
        &self.auction_room
    }

    pub(crate) const fn auction_room_mut(&mut self) -> &mut CGameAuctionRoom {
        &mut self.auction_room
    }

    pub(crate) fn refresh_player_auction_self_goods(
        &mut self,
        player_id: i32,
        tick_ms: impl FnMut() -> u32,
    ) -> Option<AuctionSelfGoodsRefresh> {
        let factory = &self.goods_factory;
        self.players
            .get_mut(&player_id)
            .map(|player| player.refresh_auction_self_goods(factory, tick_ms))
    }

    /// Возвращает process-owned Game variant для async producer-ов.
    pub(crate) const fn net_session_manager(&self) -> &CNetSessionManager {
        &self.net_session_manager
    }

    pub(crate) const fn session_factory(&self) -> &CSessionFactory {
        &self.session_factory
    }

    pub(crate) const fn session_factory_mut(&mut self) -> &mut CSessionFactory {
        &mut self.session_factory
    }

    pub(crate) fn player_trade_distance(&self, first_id: i32, second_id: i32) -> Option<i32> {
        let first_player = self.players.get(&first_id)?;
        let second_player = self.players.get(&second_id)?;
        let first = first_player.shape();
        let second = second_player.shape();
        let dx = ((first.get_pos_x().round_ties_even() as i32)
            .wrapping_sub(second.get_pos_x().round_ties_even() as i32)
            .unsigned_abs() as i32)
            .wrapping_sub(i32::from(first_player.figure().get(2)))
            .wrapping_sub(i32::from(second_player.figure().get(2)));
        let dy = ((first.get_pos_y().round_ties_even() as i32)
            .wrapping_sub(second.get_pos_y().round_ties_even() as i32)
            .unsigned_abs() as i32)
            .wrapping_sub(i32::from(first_player.figure().get(0)))
            .wrapping_sub(i32::from(second_player.figure().get(0)));
        Some(dx.max(dy))
    }

    pub(crate) fn create_player_trade_session(
        &mut self,
        inviter_id: i32,
        answerer_id: i32,
    ) -> Option<(i32, i32, i32)> {
        self.session_factory
            .create_player_trade_session(inviter_id, answerer_id)
    }

    pub(crate) fn record_player_trade_offer(
        &mut self,
        player_id: i32,
        session_id: i32,
        destination_extend_id: i32,
        destination_position: u32,
        source_extend_id: i32,
        source_position: u32,
        goods_id: CGuid,
        amount: u32,
    ) -> Result<TraderOfferAdded, PlayerTradeOfferBlock> {
        let plug_id = destination_extend_id >> 8;
        let kind = TraderContainerKind::from_index(destination_extend_id & 0xff)
            .ok_or(PlayerTradeOfferBlock::InvalidExtendId)?;
        let actual_plug_id = self
            .session_factory
            .trader_plug_by_owner(session_id, player_id)
            .ok_or(PlayerTradeOfferBlock::MissingSessionOrPlug)?;
        if actual_plug_id != plug_id {
            return Err(PlayerTradeOfferBlock::OwnerMismatch);
        }
        let expected_source = match kind {
            TraderContainerKind::Goods => matches!(source_extend_id, 1 | 2),
            TraderContainerKind::Gold => source_extend_id == 4,
            TraderContainerKind::YuanBao => source_extend_id == 5,
        };
        if !expected_source {
            return Err(PlayerTradeOfferBlock::UnsupportedSource);
        }
        let goods = self
            .players
            .get(&player_id)
            .and_then(|player| {
                player.trade_source_goods(source_extend_id, source_position, goods_id)
            })
            .cloned()
            .ok_or(PlayerTradeOfferBlock::MissingSourceGoods)?;
        let previous = crate::gameserver::appserver::container::ccontainer::PreviousContainer {
            container_type: 400,
            container_id: player_id,
            container_extend_id: source_extend_id,
            goods_position: source_position,
        };
        self.session_factory
            .query_trader_mut(plug_id)
            .ok_or(PlayerTradeOfferBlock::MissingSessionOrPlug)?
            .record_offer(
                kind,
                destination_position,
                &goods,
                amount,
                previous,
                &self.goods_factory,
            )
            .map_err(PlayerTradeOfferBlock::Trader)
    }

    pub(crate) fn remove_player_trade_offer(
        &mut self,
        player_id: i32,
        session_id: i32,
        source_extend_id: i32,
        source_position: u32,
        goods_id: CGuid,
        destination_extend_id: i32,
        destination_position: u32,
    ) -> Result<TraderOfferRemoved, PlayerTradeOfferBlock> {
        let plug_id = source_extend_id >> 8;
        let kind = TraderContainerKind::from_index(source_extend_id & 0xff)
            .ok_or(PlayerTradeOfferBlock::InvalidExtendId)?;
        let actual_plug_id = self
            .session_factory
            .trader_plug_by_owner(session_id, player_id)
            .ok_or(PlayerTradeOfferBlock::MissingSessionOrPlug)?;
        if actual_plug_id != plug_id {
            return Err(PlayerTradeOfferBlock::OwnerMismatch);
        }
        let expected_destination = match kind {
            TraderContainerKind::Goods => matches!(destination_extend_id, 1 | 2),
            TraderContainerKind::Gold => destination_extend_id == 4,
            TraderContainerKind::YuanBao => destination_extend_id == 5,
        };
        if !expected_destination {
            return Err(PlayerTradeOfferBlock::UnsupportedSource);
        }
        let original = self
            .session_factory
            .query_trader(plug_id)
            .and_then(|trader| match kind {
                TraderContainerKind::Goods => trader
                    .goods_offers()
                    .into_iter()
                    .find(|record| record.goods_id == goods_id),
                TraderContainerKind::Gold | TraderContainerKind::YuanBao => {
                    trader.currency_offer(kind)
                }
            });
        if let Some(original) = original
            && (original.original_container_extend_id != destination_extend_id
                || original.original_goods_position != destination_position)
        {
            return Err(PlayerTradeOfferBlock::SourceMismatch);
        }
        self.session_factory
            .query_trader_mut(plug_id)
            .and_then(|trader| trader.remove_offer(kind, source_position, goods_id))
            .ok_or(PlayerTradeOfferBlock::MissingSourceGoods)
    }

    pub(crate) fn reset_player_trade_ready(&mut self, session_id: i32) -> Vec<i32> {
        let plug_ids = self
            .session_factory
            .trade_session_plug_ids(session_id)
            .unwrap_or_default();
        for plug_id in &plug_ids {
            if let Some(trader) = self.session_factory.query_trader_mut(*plug_id) {
                trader.set_trade_state(false);
            }
        }
        let mut deliveries = Vec::new();
        for source_plug_id in &plug_ids {
            for target_plug_id in &plug_ids {
                if source_plug_id == target_plug_id {
                    continue;
                }
                let Some(target) = self.session_factory.query_trader(*target_plug_id) else {
                    continue;
                };
                let mut state = CMessage::new(0x000b_f716);
                state.add_long(*source_plug_id);
                state.add_byte(0);
                deliveries.push(state.send_to_player(self.net_server(), target.owner_id()));
            }
        }
        deliveries
    }

    pub(crate) fn player_trade_owner_ids(&self, session_id: i32) -> Vec<i32> {
        self.session_factory
            .trade_session_plug_ids(session_id)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|plug_id| {
                self.session_factory
                    .query_trader(plug_id)
                    .map(|trader| trader.owner_id())
            })
            .collect()
    }

    fn finish_player_trade_session(
        &mut self,
        session_id: i32,
        aborted: bool,
    ) -> (Option<SessionEndReport>, Vec<i32>, Vec<i32>) {
        let plug_ids = self
            .session_factory
            .trade_session_plug_ids(session_id)
            .unwrap_or_default();
        let owner_ids: Vec<_> = plug_ids
            .iter()
            .filter_map(|plug_id| {
                let trader = self.session_factory.query_trader_mut(*plug_id)?;
                let owner_id = trader.owner_id();
                let _cleared = trader.clear();
                Some(owner_id)
            })
            .collect();
        let terminal = if aborted {
            self.session_factory.abort_session(session_id)
        } else {
            self.session_factory.end_session(session_id)
        };
        let mut deliveries = Vec::new();
        for owner_id in owner_ids {
            if let Some(player) = self.players.get_mut(&owner_id) {
                player.set_current_progress_snapshot(PlayerProgress::None);
                deliveries
                    .push(CMessage::new(0x000b_f717).send_to_player(self.net_server(), owner_id));
            }
        }
        let collected = self.session_factory.garbage_collect_session(session_id);
        (terminal, deliveries, collected)
    }

    pub(crate) fn abort_player_trade(
        &mut self,
        player_id: i32,
        session_id: i32,
        requested_plug_id: i32,
    ) -> PlayerTradeAbortReport {
        let actual = self
            .session_factory
            .trader_plug_by_owner(session_id, player_id);
        if actual != Some(requested_plug_id)
            || !self
                .session_factory
                .trade_session_available(session_id, |owner_id| {
                    self.players.get(&owner_id).is_some_and(|player| {
                        !player.is_dead() && player.current_progress() == PlayerProgress::Trading
                    })
                })
        {
            return PlayerTradeAbortReport {
                session_id,
                plug_id: requested_plug_id,
                session_abort: None,
                terminal_deliveries: Vec::new(),
                collected_plug_ids: Vec::new(),
            };
        }
        let (session_abort, terminal_deliveries, collected_plug_ids) =
            self.finish_player_trade_session(session_id, true);
        PlayerTradeAbortReport {
            session_id,
            plug_id: requested_plug_id,
            session_abort,
            terminal_deliveries,
            collected_plug_ids,
        }
    }

    fn player_trade_snapshots(
        &self,
        session_id: i32,
    ) -> Result<[PlayerTradePartySnapshot; 2], PlayerTradeConditionBlock> {
        let plug_ids = self
            .session_factory
            .trade_session_plug_ids(session_id)
            .filter(|ids| ids.len() == 2)
            .ok_or(PlayerTradeConditionBlock::MissingPlayerOrPlug)?;
        let snapshot = |plug_id| {
            let trader = self
                .session_factory
                .query_trader(plug_id)
                .ok_or(PlayerTradeConditionBlock::MissingPlayerOrPlug)?;
            Ok(PlayerTradePartySnapshot {
                plug_id,
                owner_id: trader.owner_id(),
                goods: trader.goods_offers(),
                gold: trader.gold_amount(),
                yuan_bao: trader.yuan_bao_amount(),
            })
        };
        Ok([snapshot(plug_ids[0])?, snapshot(plug_ids[1])?])
    }

    fn validate_player_trade(
        &self,
        session_id: i32,
    ) -> Result<[PlayerTradePartySnapshot; 2], PlayerTradeConditionBlock> {
        if !self
            .session_factory
            .trade_session_available(session_id, |owner_id| {
                self.players.get(&owner_id).is_some_and(|player| {
                    !player.is_dead() && player.current_progress() == PlayerProgress::Trading
                })
            })
        {
            return Err(PlayerTradeConditionBlock::SessionUnavailable);
        }
        let parties = self.player_trade_snapshots(session_id)?;
        let maximum_gold = self
            .goods_factory
            .query_goods_max_stack_number(self.goods_factory.get_gold_coin_index());
        let maximum_yuan_bao = self
            .goods_factory
            .query_goods_max_stack_number(self.goods_factory.get_yuan_bao_index());
        for index in 0..2 {
            let party = &parties[index];
            let contrary = &parties[1 - index];
            let player = self
                .players
                .get(&party.owner_id)
                .ok_or(PlayerTradeConditionBlock::MissingPlayerOrPlug)?;
            let mut packet = player.packet().clone();
            let mut own_weight = 0u32;
            for offer in &party.goods {
                let goods = player
                    .trade_source_goods(
                        offer.original_container_extend_id,
                        offer.original_goods_position,
                        offer.goods_id,
                    )
                    .filter(|goods| {
                        if offer.original_container_extend_id == 1 {
                            goods.amount() >= offer.goods_amount
                        } else {
                            goods.amount() == offer.goods_amount
                        }
                    })
                    .ok_or(PlayerTradeConditionBlock::MissingOfferedGoods)?;
                let mut offered_goods = goods.clone();
                offered_goods.set_amount(offer.goods_amount);
                own_weight = own_weight.wrapping_add(offered_goods.weight(&self.goods_factory));
                if offer.original_container_extend_id == 1 {
                    let mut split = Some(goods.clone());
                    if packet
                        .take_goods(
                            offer.original_goods_position,
                            offer.goods_amount,
                            &self.goods_factory,
                            |_| split.take(),
                        )
                        .is_none()
                    {
                        return Err(PlayerTradeConditionBlock::MissingOfferedGoods);
                    }
                }
            }
            let contrary_player = self
                .players
                .get(&contrary.owner_id)
                .ok_or(PlayerTradeConditionBlock::MissingPlayerOrPlug)?;
            let mut incoming_weight = 0u32;
            for offer in &contrary.goods {
                let goods = contrary_player
                    .trade_source_goods(
                        offer.original_container_extend_id,
                        offer.original_goods_position,
                        offer.goods_id,
                    )
                    .filter(|goods| {
                        if offer.original_container_extend_id == 1 {
                            goods.amount() >= offer.goods_amount
                        } else {
                            goods.amount() == offer.goods_amount
                        }
                    })
                    .ok_or(PlayerTradeConditionBlock::MissingOfferedGoods)?;
                let mut offered_goods = goods.clone();
                offered_goods.set_amount(offer.goods_amount);
                incoming_weight =
                    incoming_weight.wrapping_add(offered_goods.weight(&self.goods_factory));
                let mut incoming = Some(offered_goods);
                let outcome = packet.add_goods(&mut incoming, &self.goods_factory, true);
                if incoming.is_some() || matches!(outcome, VolumeGoodsAddOutcome::Rejected(_)) {
                    return Err(PlayerTradeConditionBlock::PacketSpace);
                }
            }
            let resulting_burden = player
                .current_burden(&self.goods_factory)
                .wrapping_sub(own_weight)
                .wrapping_add(incoming_weight);
            if u32::from(player.combat_properties().burden) < resulting_burden {
                return Err(PlayerTradeConditionBlock::BurdenExceeded);
            }
            if player.money() < party.gold {
                return Err(PlayerTradeConditionBlock::InsufficientGold);
            }
            if maximum_gold
                < player
                    .money()
                    .wrapping_sub(party.gold)
                    .wrapping_add(contrary.gold)
            {
                return Err(PlayerTradeConditionBlock::GoldCapacity);
            }
            if player.yuan_bao() < party.yuan_bao {
                return Err(PlayerTradeConditionBlock::InsufficientYuanBao);
            }
            if maximum_yuan_bao
                < player
                    .yuan_bao()
                    .wrapping_sub(party.yuan_bao)
                    .wrapping_add(contrary.yuan_bao)
            {
                return Err(PlayerTradeConditionBlock::YuanBaoCapacity);
            }
        }
        Ok(parties)
    }

    fn send_player_trade_billing_request(
        &self,
        payer: &PlayerTradePartySnapshot,
        receiver: &PlayerTradePartySnapshot,
        amount: u32,
        session_id: i32,
    ) -> Result<i32, SendMessageError> {
        let payer_player = self
            .players
            .get(&payer.owner_id)
            .expect("validated trade payer остаётся online");
        let receiver_player = self
            .players
            .get(&receiver.owner_id)
            .expect("validated trade receiver остаётся online");
        let ip = |value: u32| {
            format!(
                "{}.{}.{}.{}",
                value & 0xff,
                value >> 8 & 0xff,
                value >> 16 & 0xff,
                value >> 24
            )
        };
        let mut message = CMessage::new(0x000e_f203);
        message.add_long(2);
        message.add_long(payer.owner_id);
        message.add_long(receiver.owner_id);
        add_legacy_c_string(message.base_mut(), payer_player.account());
        add_legacy_c_string(message.base_mut(), receiver_player.account());
        add_legacy_c_string(message.base_mut(), ip(payer_player.client_ip()).as_bytes());
        add_legacy_c_string(
            message.base_mut(),
            ip(receiver_player.client_ip()).as_bytes(),
        );
        add_legacy_c_string(message.base_mut(), payer_player.player_name());
        add_legacy_c_string(message.base_mut(), receiver_player.player_name());
        message.add_ulong(amount);
        message.add_long(1);
        message.add_long(1);
        message.add_long(session_id);
        message.add_long(payer.plug_id);
        message.add_long(self.login_server_id);
        message.add_long(self.world_server_id);
        message.base_mut().add_guid(CGuid::GUID_INVALID);
        message.send(self, false)
    }

    fn send_trade_notice(&self, session_id: i32, string_id: &[u8]) -> Vec<i32> {
        self.session_factory
            .trade_session_plug_ids(session_id)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|plug_id| self.session_factory.query_trader(plug_id))
            .map(|trader| {
                colored_player_notice_message(0xffff_ffff, 0, self.get_string_by_id(string_id))
                    .send_to_player(self.net_server(), trader.owner_id())
            })
            .collect()
    }

    fn trade_condition_notice(block: PlayerTradeConditionBlock) -> &'static [u8] {
        match block {
            PlayerTradeConditionBlock::PacketSpace => b"GS0267",
            PlayerTradeConditionBlock::BurdenExceeded => b"GS0268",
            PlayerTradeConditionBlock::MissingOfferedGoods => b"GS0269",
            PlayerTradeConditionBlock::MissingPlayerOrPlug
            | PlayerTradeConditionBlock::SessionUnavailable => b"GS0270",
            PlayerTradeConditionBlock::InsufficientGold => b"GS0271",
            PlayerTradeConditionBlock::GoldCapacity => b"GS0272",
            PlayerTradeConditionBlock::InsufficientYuanBao => b"GS0273",
            PlayerTradeConditionBlock::YuanBaoCapacity => b"GS0274",
        }
    }

    pub(crate) fn toggle_player_trade_ready<Context: GameContainerMessageRuntime>(
        &mut self,
        player_id: i32,
        session_id: i32,
        requested_plug_id: i32,
        context: &mut Context,
    ) -> PlayerTradeReadyReport {
        let mut report = PlayerTradeReadyReport {
            session_id,
            plug_id: requested_plug_id,
            contrary_plug_id: None,
            ready_deliveries: Vec::new(),
            notification_deliveries: Vec::new(),
            packet_deliveries: Vec::new(),
            equipment_removals: Vec::new(),
            money_deliveries: Vec::new(),
            audit_deliveries: Vec::new(),
            session_end: None,
            terminal_deliveries: Vec::new(),
            collected_plug_ids: Vec::new(),
            outcome: PlayerTradeReadyOutcome::MissingSessionOrPlug,
        };
        let actual = self
            .session_factory
            .trader_plug_by_owner(session_id, player_id);
        if actual != Some(requested_plug_id) {
            return report;
        }
        if !self
            .session_factory
            .trade_session_available(session_id, |owner_id| {
                self.players.get(&owner_id).is_some_and(|player| {
                    !player.is_dead() && player.current_progress() == PlayerProgress::Trading
                })
            })
        {
            report.outcome = PlayerTradeReadyOutcome::SessionUnavailable;
            return report;
        }
        let ready = {
            let trader = self
                .session_factory
                .query_trader_mut(requested_plug_id)
                .expect("owner lookup подтвердил trader plug");
            let ready = !trader.ready();
            trader.set_trade_state(ready);
            ready
        };
        let contrary_plug_id = self
            .session_factory
            .contrary_trader_id(session_id, requested_plug_id);
        report.contrary_plug_id = contrary_plug_id;
        if let Some(contrary_plug_id) = contrary_plug_id
            && let Some(contrary) = self.session_factory.query_trader(contrary_plug_id)
        {
            let mut state = CMessage::new(0x000b_f716);
            state.add_long(requested_plug_id);
            state.add_byte(u8::from(ready));
            report
                .ready_deliveries
                .push(state.send_to_player(self.net_server(), contrary.owner_id()));
        }
        if !ready
            || contrary_plug_id.is_none_or(|plug_id| {
                !self
                    .session_factory
                    .query_trader(plug_id)
                    .is_some_and(|trader| trader.ready())
            })
        {
            report.outcome = PlayerTradeReadyOutcome::Changed { ready };
            return report;
        }
        let parties = match self.validate_player_trade(session_id) {
            Ok(parties) => parties,
            Err(block) => {
                report.notification_deliveries =
                    self.send_trade_notice(session_id, Self::trade_condition_notice(block));
                report.outcome = PlayerTradeReadyOutcome::ConditionBlocked(block);
                return report;
            }
        };
        let yuan_difference = i64::from(parties[0].yuan_bao) - i64::from(parties[1].yuan_bao);
        if yuan_difference != 0 {
            let (payer, receiver) = if yuan_difference > 0 {
                (&parties[0], &parties[1])
            } else {
                (&parties[1], &parties[0])
            };
            let amount = yuan_difference.unsigned_abs() as u32;
            let delivery =
                self.send_player_trade_billing_request(payer, receiver, amount, session_id);
            report.outcome = PlayerTradeReadyOutcome::BillingPending(PlayerTradeBillingRequest {
                payer_id: payer.owner_id,
                receiver_id: receiver.owner_id,
                amount,
                session_id,
                payer_plug_id: payer.plug_id,
                delivery,
            });
            return report;
        }
        let completed = self.commit_player_trade(parties, None, 0, &[], context, &mut report);
        let (session_end, terminal_deliveries, collected_plug_ids) =
            self.finish_player_trade_session(session_id, false);
        report.session_end = session_end;
        report.terminal_deliveries = terminal_deliveries;
        report.collected_plug_ids = collected_plug_ids;
        report.outcome = if completed {
            PlayerTradeReadyOutcome::Completed
        } else {
            PlayerTradeReadyOutcome::RolledBack
        };
        report
    }

    fn commit_player_trade<Context: GameContainerMessageRuntime>(
        &mut self,
        parties: [PlayerTradePartySnapshot; 2],
        billing_payer_id: Option<i32>,
        billing_amount: u32,
        transaction: &[u8],
        context: &mut Context,
        report: &mut PlayerTradeReadyReport,
    ) -> bool {
        let audit_parties = parties.each_ref().map(|party| {
            let player = self
                .players
                .get(&party.owner_id)
                .expect("validated trade party остаётся online");
            PlayerTradeAuditParty {
                owner_id: party.owner_id,
                pk_count: u32::from(player.pk_count()),
                money: player.money(),
                tile_x: player.shape().get_tile_x().unwrap_or(0),
                tile_y: player.shape().get_tile_y().unwrap_or(0),
                client_ip: player.client_ip(),
                name: player.player_name().to_vec(),
            }
        });
        let mut removed_by_plug = BTreeMap::<i32, Vec<CGoods>>::new();
        let mut delivered = Vec::<DeliveredPlayerTradeGoods>::new();
        for party in &parties {
            let mut packet_splits = BTreeMap::<CGuid, CGoods>::new();
            for offer in &party.goods {
                if offer.original_container_extend_id != 1 {
                    continue;
                }
                let Some((source_amount, base_properties_index)) = self
                    .players
                    .get(&party.owner_id)
                    .and_then(|player| {
                        player.trade_source_goods(
                            offer.original_container_extend_id,
                            offer.original_goods_position,
                            offer.goods_id,
                        )
                    })
                    .map(|source| (source.amount(), source.base_properties_index()))
                else {
                    continue;
                };
                if offer.goods_amount < source_amount {
                    let Some(split) = self.create_goods_core(base_properties_index) else {
                        self.undo_delivered_trade_goods(&mut removed_by_plug, delivered, report);
                        self.rollback_detached_trade_goods(
                            &parties,
                            removed_by_plug,
                            context,
                            report,
                        );
                        return false;
                    };
                    packet_splits.insert(offer.goods_id, split);
                }
            }
            let Some(mut player) = self.players.remove(&party.owner_id) else {
                self.undo_delivered_trade_goods(&mut removed_by_plug, delivered, report);
                self.rollback_detached_trade_goods(&parties, removed_by_plug, context, report);
                return false;
            };
            let mut removed_goods = Vec::new();
            let mut failed = false;
            for offer in &party.goods {
                let source = player
                    .trade_source_goods(
                        offer.original_container_extend_id,
                        offer.original_goods_position,
                        offer.goods_id,
                    )
                    .filter(|goods| {
                        if offer.original_container_extend_id == 1 {
                            goods.amount() >= offer.goods_amount
                        } else {
                            goods.amount() == offer.goods_amount
                        }
                    })
                    .cloned();
                let Some(source) = source else {
                    failed = true;
                    break;
                };
                if offer.original_container_extend_id == 1 {
                    let previous_amount = source.amount();
                    let mut split = packet_splits.remove(&offer.goods_id);
                    let Some(outcome) = player.packet_mut().take_goods(
                        offer.original_goods_position,
                        offer.goods_amount,
                        &self.goods_factory,
                        |_| split.take(),
                    ) else {
                        failed = true;
                        break;
                    };
                    let taken = match outcome {
                        VolumeGoodsRemoveOutcome::Removed(taken)
                        | VolumeGoodsRemoveOutcome::RemovedButCellMissing(taken) => taken,
                    };
                    let (detached_goods, removed_position) = match taken {
                        AmountLimitGoodsTaken::Removed(removed) => {
                            (removed.goods, removed.position)
                        }
                        AmountLimitGoodsTaken::Split(split) => (split.goods, split.position),
                    };
                    let consumption = CiQingPacketConsumption {
                        player_id: party.owner_id,
                        goods: source.identity(),
                        position: removed_position.unwrap_or(offer.original_goods_position),
                        previous_amount,
                        remaining_amount: previous_amount.wrapping_sub(offer.goods_amount),
                        removal: None,
                    };
                    report
                        .packet_deliveries
                        .push(self.send_player_packet_consumption(&consumption));
                    removed_goods.push(detached_goods);
                } else if offer.original_container_extend_id == 2 {
                    let facts = context.enhancement_equipment_remove_facts(
                        &player,
                        &source,
                        self.globe_setup.pack_add_enabled(),
                    );
                    let mut recompute =
                        |player: &CPlayer| context.recompute_enhancement_player_properties(player);
                    let mut removal = player.remove_equipment_goods(
                        offer.goods_id,
                        &self.goods_factory,
                        &self.skill_factory,
                        facts,
                        &mut recompute,
                    );
                    drop(recompute);
                    self.publish_player_equipment_remove_report(&mut removal, context);
                    report.equipment_removals.push(removal.clone());
                    let outcome = std::mem::replace(
                        &mut removal.outcome,
                        EquipmentRemoveOutcome::Missing {
                            partial_effects: Default::default(),
                        },
                    );
                    match outcome {
                        EquipmentRemoveOutcome::Removed(removed) => {
                            let previous = crate::gameserver::appserver::container::ccontainer::PreviousContainer {
                                container_type: 400,
                                container_id: party.owner_id,
                                container_extend_id: 2,
                                goods_position: offer.original_goods_position,
                            };
                            report
                                .packet_deliveries
                                .push(vec![self.send_container_object_delete(
                                    party.owner_id,
                                    &previous,
                                    removed.goods.identity(),
                                    removed.goods.amount(),
                                )]);
                            removed_goods.push(removed.goods);
                        }
                        outcome => {
                            removal.outcome = outcome;
                            failed = true;
                            break;
                        }
                    }
                } else {
                    failed = true;
                    break;
                }
            }
            self.players.insert(party.owner_id, player);
            removed_by_plug.insert(party.plug_id, removed_goods);
            if failed {
                self.undo_delivered_trade_goods(&mut removed_by_plug, delivered, report);
                self.rollback_detached_trade_goods(&parties, removed_by_plug, context, report);
                return false;
            }
        }
        let audit_goods_by_plug = removed_by_plug.clone();

        for index in 0..2 {
            let source = &parties[index];
            let receiver = &parties[1 - index];
            let goods = removed_by_plug.remove(&source.plug_id).unwrap_or_default();
            let Some(player) = self.players.get_mut(&receiver.owner_id) else {
                removed_by_plug.insert(source.plug_id, goods);
                self.undo_delivered_trade_goods(&mut removed_by_plug, delivered, report);
                self.rollback_detached_trade_goods(&parties, removed_by_plug, context, report);
                return false;
            };
            let originals = goods.clone();
            let mut encode = |goods: &CGoods| context.encode_goods_for_old_client(goods);
            let (additions, rejected) =
                player.add_traded_goods_to_packet(goods, &self.goods_factory, &mut encode);
            for addition in &additions {
                report
                    .packet_deliveries
                    .push(self.send_player_packet_addition(addition));
            }
            let failed = !rejected.is_empty()
                || additions
                    .iter()
                    .any(|addition| addition.resulting_amount.is_none());
            for (original, addition) in originals.into_iter().zip(additions.iter()) {
                if addition.resulting_amount.is_some() {
                    delivered.push(DeliveredPlayerTradeGoods {
                        source_plug_id: source.plug_id,
                        receiver_id: receiver.owner_id,
                        original,
                        addition: addition.clone(),
                    });
                }
            }
            if failed {
                removed_by_plug.insert(source.plug_id, rejected);
                self.undo_delivered_trade_goods(&mut removed_by_plug, delivered, report);
                self.rollback_detached_trade_goods(&parties, removed_by_plug, context, report);
                return false;
            }
        }

        for index in 0..2 {
            let party = &parties[index];
            let contrary = &parties[1 - index];
            let current = self.players.get(&party.owner_id).map_or(0, CPlayer::money);
            let resulting = current.wrapping_sub(party.gold).wrapping_add(contrary.gold);
            if resulting < current {
                let change = self
                    .players
                    .get_mut(&party.owner_id)
                    .expect("trade party остаётся online")
                    .decrease_money(current.wrapping_sub(resulting), &self.goods_factory);
                report
                    .money_deliveries
                    .extend(self.send_player_money_decrease(party.owner_id, &change.outcome));
            } else if current < resulting {
                let delta = resulting.wrapping_sub(current);
                let created =
                    self.create_goods_batch(self.goods_factory.get_gold_coin_index(), delta);
                let outcome = self
                    .players
                    .get_mut(&party.owner_id)
                    .expect("trade party остаётся online")
                    .increase_money(delta, &self.goods_factory, created);
                report
                    .money_deliveries
                    .extend(self.send_player_money_increase(party.owner_id, &outcome, context));
            }
        }
        report
            .audit_deliveries
            .extend(self.send_player_trade_audits(
                &parties,
                &audit_parties,
                &audit_goods_by_plug,
                billing_payer_id,
                billing_amount,
                transaction,
            ));
        true
    }

    fn send_player_trade_audits(
        &self,
        parties: &[PlayerTradePartySnapshot; 2],
        audit_parties: &[PlayerTradeAuditParty; 2],
        goods_by_plug: &BTreeMap<i32, Vec<CGoods>>,
        billing_payer_id: Option<i32>,
        billing_amount: u32,
        transaction: &[u8],
    ) -> Vec<Result<i32, SendMessageError>> {
        let mut deliveries = Vec::new();
        if self.log_system.goods_trade_log_enabled() {
            for source_index in 0..2 {
                let receiver_index = 1 - source_index;
                let source = &audit_parties[source_index];
                let receiver = &audit_parties[receiver_index];
                for goods in goods_by_plug
                    .get(&parties[source_index].plug_id)
                    .into_iter()
                    .flatten()
                {
                    deliveries.push(self.send_player_trade_goods_audit(
                        source,
                        receiver,
                        goods.identity().ex_id,
                        goods.price(),
                        goods.amount(),
                        goods.name(),
                    ));
                }
                let gold = parties[source_index].gold;
                if gold != 0 {
                    deliveries.push(self.send_player_trade_goods_audit(
                        source,
                        receiver,
                        CGuid::GUID_INVALID,
                        gold,
                        gold,
                        b"",
                    ));
                }
            }
        }
        if self.log_system.increment_log_enabled()
            && billing_amount != 0
            && let Some(payer_id) = billing_payer_id
            && let Some(payer_index) = audit_parties
                .iter()
                .position(|party| party.owner_id == payer_id)
        {
            let receiver_index = 1 - payer_index;
            let payer = &audit_parties[payer_index];
            let receiver = &audit_parties[receiver_index];
            let payer_yuan_bao = self
                .players
                .get(&payer.owner_id)
                .map_or(0, CPlayer::yuan_bao);
            let receiver_yuan_bao = self
                .players
                .get(&receiver.owner_id)
                .map_or(0, CPlayer::yuan_bao);
            let payer_text = format_legacy_mixed(
                self.get_string_by_id(b"GS0275"),
                &[
                    LegacyFormatArgument::Bytes(&receiver.name),
                    LegacyFormatArgument::Signed(billing_amount as i32),
                ],
                0xff,
            );
            let receiver_text = format_legacy_mixed(
                self.get_string_by_id(b"GS0276"),
                &[
                    LegacyFormatArgument::Bytes(&payer.name),
                    LegacyFormatArgument::Signed(billing_amount as i32),
                ],
                0xff,
            );
            for (kind, party, balance, text) in [
                (2u8, payer, payer_yuan_bao, payer_text),
                (3u8, receiver, receiver_yuan_bao, receiver_text),
            ] {
                let mut audit = CMessage::new(0x0006_020d);
                audit.add_byte(kind);
                add_legacy_c_string(audit.base_mut(), transaction);
                audit.add_ulong(billing_amount);
                add_legacy_c_string(audit.base_mut(), &text);
                audit.add_long(party.owner_id);
                audit.add_ulong(balance);
                deliveries.push(audit.send(self, false));
            }
        }
        deliveries
    }

    fn send_player_trade_goods_audit(
        &self,
        source: &PlayerTradeAuditParty,
        receiver: &PlayerTradeAuditParty,
        goods_id: CGuid,
        price: u32,
        amount: u32,
        goods_name: &[u8],
    ) -> Result<i32, SendMessageError> {
        let mut audit = CMessage::new(0x0006_0201);
        audit.add_byte(0);
        audit.add_long(receiver.owner_id);
        audit.add_ulong(receiver.pk_count);
        audit.add_ulong(receiver.money);
        audit.add_long(receiver.tile_x);
        audit.add_long(receiver.tile_y);
        audit.add_long(source.owner_id);
        audit.add_ulong(source.pk_count);
        audit.add_ulong(source.money);
        audit.add_long(source.tile_x);
        audit.add_long(source.tile_y);
        audit.base_mut().add_guid(goods_id);
        audit.add_ulong(price);
        audit.add_ulong(amount);
        add_legacy_c_string(audit.base_mut(), goods_name);
        audit.add_ulong(receiver.client_ip);
        audit.add_ulong(source.client_ip);
        audit.send(self, false)
    }

    fn undo_delivered_trade_goods(
        &mut self,
        detached_by_plug: &mut BTreeMap<i32, Vec<CGoods>>,
        delivered: Vec<DeliveredPlayerTradeGoods>,
        report: &mut PlayerTradeReadyReport,
    ) {
        for delivered in delivered.into_iter().rev() {
            let Some(player) = self.players.get_mut(&delivered.receiver_id) else {
                continue;
            };
            let Some(consumption) = player
                .rollback_traded_packet_addition(&delivered.addition, delivered.original.amount())
            else {
                continue;
            };
            report
                .packet_deliveries
                .push(self.send_player_packet_consumption(&consumption));
            detached_by_plug
                .entry(delivered.source_plug_id)
                .or_default()
                .push(delivered.original);
        }
    }

    fn rollback_detached_trade_goods<Context: GameContainerMessageRuntime>(
        &mut self,
        parties: &[PlayerTradePartySnapshot; 2],
        mut goods_by_plug: BTreeMap<i32, Vec<CGoods>>,
        context: &mut Context,
        report: &mut PlayerTradeReadyReport,
    ) {
        for party in parties {
            let goods = goods_by_plug.remove(&party.plug_id).unwrap_or_default();
            if goods.is_empty() {
                continue;
            }
            let Some(player) = self.players.get_mut(&party.owner_id) else {
                continue;
            };
            let mut encode = |goods: &CGoods| context.encode_goods_for_old_client(goods);
            let (additions, unrecoverable) =
                player.add_traded_goods_to_packet(goods, &self.goods_factory, &mut encode);
            for addition in &additions {
                report
                    .packet_deliveries
                    .push(self.send_player_packet_addition(addition));
            }
            if !unrecoverable.is_empty()
                || additions
                    .iter()
                    .any(|addition| addition.resulting_amount.is_none())
            {
                report.notification_deliveries.push(
                    colored_player_notice_message(0xffff_ffff, 0, self.get_string_by_id(b"GS0277"))
                        .send_to_player(self.net_server(), party.owner_id),
                );
            }
        }
    }

    pub(crate) fn send_player_money_increase<Context: OldClientGoodsCodec>(
        &self,
        player_id: i32,
        outcome: &crate::gameserver::appserver::container::cwallet::CurrencyIncreaseOutcome,
        context: &mut Context,
    ) -> Vec<i32> {
        use crate::gameserver::appserver::container::cwallet::CurrencyIncreaseOutcome;
        match outcome {
            CurrencyIncreaseOutcome::Created(added) => {
                let Some(goods) = self
                    .players
                    .get(&player_id)
                    .and_then(|player| player.trade_source_goods(4, 0, added.identity.ex_id))
                else {
                    return Vec::new();
                };
                let mut message = CS2CContainerObjectMove::default();
                message.set_operation(ContainerObjectMoveOperation::NewObject);
                message.set_destination_container(added.owner_type, added.owner_id, 0);
                message.set_destination_container_extend_id(4);
                message.set_destination_object(added.identity.object_type, added.identity.ex_id);
                message.set_object_stream(context.encode_goods_for_old_client(goods));
                vec![message.send_to_player(self, player_id)]
            }
            CurrencyIncreaseOutcome::Increased(change) => {
                let mut message = CS2CContainerObjectAmountChange::default();
                message.set_source_container(change.owner_type, change.owner_id, change.position);
                message.set_source_container_extend_id(4);
                message.set_object(change.identity.object_type, change.identity.ex_id);
                message.set_object_amount(change.new_amount);
                vec![message.send_to_player(self, player_id)]
            }
            CurrencyIncreaseOutcome::NoChange
            | CurrencyIncreaseOutcome::CapacityExceeded { .. }
            | CurrencyIncreaseOutcome::InvalidStoredCurrency { .. }
            | CurrencyIncreaseOutcome::CreationFailed => Vec::new(),
        }
    }

    pub(crate) fn complete_player_trade_after_billing<Context: GameContainerMessageRuntime>(
        &mut self,
        session_id: i32,
        requested_plug_id: i32,
        payer_id: i32,
        billing_amount: u32,
        transaction: &[u8],
        context: &mut Context,
    ) -> PlayerTradeReadyReport {
        let selected_plug_id = self
            .session_factory
            .query_trader(requested_plug_id)
            .filter(|trader| trader.session_id() == session_id)
            .and_then(|trader| {
                if trader.owner_id() == payer_id {
                    Some(requested_plug_id)
                } else {
                    self.session_factory
                        .contrary_trader_id(session_id, requested_plug_id)
                        .filter(|contrary_id| {
                            self.session_factory
                                .query_trader(*contrary_id)
                                .is_some_and(|contrary| contrary.owner_id() == payer_id)
                        })
                }
            })
            .unwrap_or(requested_plug_id);
        let mut report = PlayerTradeReadyReport {
            session_id,
            plug_id: selected_plug_id,
            contrary_plug_id: self
                .session_factory
                .contrary_trader_id(session_id, selected_plug_id),
            ready_deliveries: Vec::new(),
            notification_deliveries: Vec::new(),
            packet_deliveries: Vec::new(),
            equipment_removals: Vec::new(),
            money_deliveries: Vec::new(),
            audit_deliveries: Vec::new(),
            session_end: None,
            terminal_deliveries: Vec::new(),
            collected_plug_ids: Vec::new(),
            outcome: PlayerTradeReadyOutcome::MissingSessionOrPlug,
        };
        if self
            .session_factory
            .query_trader(selected_plug_id)
            .is_none_or(|trader| trader.owner_id() != payer_id)
            || !self
                .session_factory
                .trade_session_available(session_id, |owner_id| {
                    self.players.get(&owner_id).is_some_and(|player| {
                        !player.is_dead() && player.current_progress() == PlayerProgress::Trading
                    })
                })
        {
            if self.players.contains_key(&payer_id) {
                report.notification_deliveries.push(
                    colored_player_notice_message(
                        0xffff_ffff,
                        0,
                        b"Personal Trade has been CANCELED",
                    )
                    .send_to_player(self.net_server(), payer_id),
                );
            }
            return report;
        }
        let Ok(parties) = self.player_trade_snapshots(session_id) else {
            return report;
        };
        let completed = self.commit_player_trade(
            parties,
            Some(payer_id),
            billing_amount,
            transaction,
            context,
            &mut report,
        );
        let (session_end, terminal_deliveries, collected_plug_ids) =
            self.finish_player_trade_session(session_id, false);
        report.session_end = session_end;
        report.terminal_deliveries = terminal_deliveries;
        report.collected_plug_ids = collected_plug_ids;
        report.outcome = if completed {
            PlayerTradeReadyOutcome::Completed
        } else {
            PlayerTradeReadyOutcome::RolledBack
        };
        report
    }

    fn detach_terminal_equipment_session_listeners(
        &mut self,
        sessions: &[TerminalEquipmentSessionCollected],
    ) {
        for plug in sessions.iter().flat_map(|session| &session.plugs) {
            if let Some(player) = self.players.get_mut(&plug.owner_id) {
                let _listener_detach = player.detach_equipment_session_listener(plug.plug_id);
            }
        }
    }

    /// Exact `CGame::SetAuctionState`: false не меняет saved wall-clock,
    /// true публикует полученный caller-ом `_time` sample.
    pub(crate) const fn set_auction_state(&mut self, enabled: bool, wall_time_seconds: u32) {
        self.auction_now = enabled;
        if enabled {
            self.auction_last_check_seconds = wall_time_seconds;
        }
    }

    /// Точный false-only effect Globe decoder-а без обновления timestamp.
    pub(crate) const fn force_auction_disabled(&mut self) {
        self.set_auction_state(false, self.auction_last_check_seconds);
    }

    pub(crate) fn set_function_file_data(&mut self, data: Vec<u8>) -> GameSingleFilePublication {
        if self.function_list_file_data.is_some() {
            self.function_list_file_data.take();
            return GameSingleFilePublication::RepeatedOwnerFreed;
        }
        self.function_list_file_data = Some(data);
        let published = self
            .function_list_file_data
            .as_deref()
            .expect("function list только что опубликован");
        self.script_functions
            .load(legacy_c_string_prefix(published));
        GameSingleFilePublication::Published
    }

    pub(crate) fn set_variable_file_data(&mut self, data: Vec<u8>) -> GameSingleFilePublication {
        if self.variable_list_file_data.is_some() {
            self.variable_list_file_data.take();
            return GameSingleFilePublication::RepeatedOwnerFreed;
        }
        self.variable_list_file_data = Some(data);
        GameSingleFilePublication::Published
    }

    pub(crate) fn set_general_variable_file_data(
        &mut self,
        source: &[u8],
        cursor: usize,
    ) -> Result<GameVariableSnapshotReport, GameVariableSnapshotError> {
        let mut local_cursor = cursor;
        self.general_variables.decode_world_snapshot(
            self.variable_list_file_data.as_deref(),
            source,
            &mut local_cursor,
        )
    }

    pub(crate) fn set_script_file_data(&mut self, path: Vec<u8>, data: Vec<u8>) -> bool {
        self.script_file_data
            .insert(legacy_c_string_prefix(&path).to_vec(), data)
            .is_some()
    }

    pub(crate) fn function_file_data(&self) -> Option<&[u8]> {
        self.function_list_file_data.as_deref()
    }

    pub(crate) fn variable_file_data(&self) -> Option<&[u8]> {
        self.variable_list_file_data.as_deref()
    }

    pub(crate) fn script_file_data(&self, path: &[u8]) -> Option<&[u8]> {
        self.script_file_data
            .get(legacy_c_string_prefix(path))
            .map(Vec::as_slice)
    }

    pub(crate) fn script_function_id(&self, name: &[u8]) -> Option<i32> {
        self.script_functions.query(name)
    }

    fn next_organizing_correlation(&mut self) -> (i64, i32) {
        self.next_organizing_session_id = self.next_organizing_session_id.wrapping_add(1);
        if self.next_organizing_session_id == 0 {
            self.next_organizing_session_id = 1;
        }
        self.next_organizing_password = self.next_organizing_password.wrapping_add(1);
        if self.next_organizing_password == 0 {
            self.next_organizing_password = 1;
        }
        (
            self.next_organizing_session_id,
            self.next_organizing_password,
        )
    }

    fn send_faction_script_notice(&self, player_id: i32, text: &[u8]) -> i32 {
        colored_player_notice_message(0xffff_ffff, 0, text)
            .send_to_player(self.net_server(), player_id)
    }

    pub(crate) fn start_script_faction_creation(
        &mut self,
        player_id: i32,
        required_level: i32,
        required_goods: &[u8],
        required_money: i32,
        country: u8,
        now_ms: u32,
    ) {
        let Some(player) = self.find_player(player_id) else {
            return;
        };
        if player.faction_id() > 0 {
            let text = self.get_string_by_id(b"GS0190").to_vec();
            let _ = self.send_faction_script_notice(player_id, &text);
            return;
        }
        if i32::from(player.level()) < required_level {
            let text = format_legacy_mixed(
                self.get_string_by_id(b"GS0191"),
                &[LegacyFormatArgument::Signed(required_level)],
                0xff,
            );
            let _ = self.send_faction_script_notice(player_id, &text);
            return;
        }
        if player.money() < required_money as u32 {
            let text = format_legacy_mixed(
                self.get_string_by_id(b"GS0192"),
                &[LegacyFormatArgument::Signed(required_money)],
                0xff,
            );
            let _ = self.send_faction_script_notice(player_id, &text);
            return;
        }
        if required_goods != b"0" {
            let goods_index = self
                .goods_factory
                .query_goods_id_by_original_name(Some(required_goods));
            let goods = self
                .goods_factory
                .query_goods_base_properties_by_original_name(Some(required_goods));
            let Some(goods) = goods else {
                let text = self.get_string_by_id(b"GS0194").to_vec();
                let _ = self.send_faction_script_notice(player_id, &text);
                return;
            };
            if player.check_item_in_packet(goods_index) == 0 {
                let text = format_legacy_mixed(
                    self.get_string_by_id(b"GS0193"),
                    &[LegacyFormatArgument::Bytes(goods.name())],
                    0xff,
                );
                let _ = self.send_faction_script_notice(player_id, &text);
                return;
            }
        }
        if player.create_faction_operator() {
            return;
        }

        let (session_id, password) = self.next_organizing_correlation();
        if let Some(player) = self.find_player_mut(player_id) {
            player.set_create_faction_operator(true);
        }
        self.pending_faction_creations.insert(
            session_id,
            PendingFactionCreation {
                session_id,
                password,
                player_id,
                required_goods: required_goods.to_vec(),
                required_money,
                country,
                world_request_sent: false,
                expires_at_ms: now_ms.wrapping_add(1000),
            },
        );
        let mut prompt = CMessage::new(0x000b_ff01);
        prompt.base_mut().add_long64(session_id);
        prompt.add_long(password);
        let _ = prompt.send_to_player(self.net_server(), player_id);
    }

    pub(crate) fn submit_script_faction_creation<Context: ScriptRegionChangeContext>(
        &mut self,
        player_id: i32,
        session_id: i64,
        password: i32,
        faction_name: Option<&[u8]>,
        context: &mut Context,
    ) -> bool {
        let Some(pending) = self.pending_faction_creations.get(&session_id).cloned() else {
            return false;
        };
        if pending.player_id != player_id || pending.password != password {
            return false;
        }
        let Some(faction_name) = faction_name else {
            self.pending_faction_creations.remove(&session_id);
            if let Some(player) = self.find_player_mut(player_id) {
                player.set_create_faction_operator(false);
            }
            return true;
        };
        if faction_name.is_empty() || faction_name.len() > 20 || faction_name.contains(&0) {
            return false;
        }
        let Some(player) = self.find_player(player_id) else {
            self.pending_faction_creations.remove(&session_id);
            return false;
        };
        let mut snapshot = Vec::new();
        if !self.encode_player_game_save(player, &mut snapshot, context) {
            return false;
        }
        let mut request = CMessage::new(0x0006_0103);
        request.base_mut().add_long64(session_id);
        request.add_long(password);
        request.add_long(player_id);
        request.add_byte(pending.country);
        add_legacy_c_string(request.base_mut(), faction_name);
        request.base_mut().add(&snapshot);
        let sent = matches!(request.send(self, false), Ok(value) if value != 0);
        if sent {
            if let Some(pending) = self.pending_faction_creations.get_mut(&session_id) {
                pending.world_request_sent = true;
            }
        }
        sent
    }

    pub(crate) fn finish_script_faction_creation(
        &mut self,
        session_id: i64,
        password: i32,
        player_id: i32,
        result: i32,
    ) -> bool {
        let Some(pending) = self.pending_faction_creations.remove(&session_id) else {
            return false;
        };
        if pending.password != password
            || pending.player_id != player_id
            || !pending.world_request_sent
        {
            self.pending_faction_creations.insert(session_id, pending);
            return false;
        }
        if result == 1 {
            if pending.required_goods != b"0" {
                let base_index = self
                    .goods_factory
                    .query_goods_id_by_original_name(Some(&pending.required_goods));
                let consumptions = self
                    .find_player_mut(player_id)
                    .map(|player| player.remove_item_in_packet(base_index, 1))
                    .unwrap_or_default();
                for consumption in consumptions {
                    let _ = self.send_player_packet_consumption(&consumption);
                }
            }
            if let Some(change) =
                self.decrease_player_money(player_id, pending.required_money as u32)
            {
                let _ = self.send_player_money_decrease(player_id, &change.outcome);
            }
        }
        if let Some(player) = self.find_player_mut(player_id) {
            player.set_create_faction_operator(false);
        }
        true
    }

    pub(crate) fn start_script_faction_application(
        &mut self,
        player_id: i32,
        required_level: i32,
        now_ms: u32,
    ) {
        let Some(player) = self.find_player(player_id) else {
            return;
        };
        if player.faction_id() > 0 {
            let text = self.get_string_by_id(b"GS0190").to_vec();
            let _ = self.send_faction_script_notice(player_id, &text);
            return;
        }
        if i32::from(player.level()) < required_level {
            let text = format_legacy_mixed(
                self.get_string_by_id(b"GS0191"),
                &[LegacyFormatArgument::Signed(required_level)],
                0xff,
            );
            let _ = self.send_faction_script_notice(player_id, &text);
            return;
        }
        if player.apply_join_faction_operator() {
            return;
        }
        let (session_id, password) = self.next_organizing_correlation();
        if let Some(player) = self.find_player_mut(player_id) {
            player.set_apply_join_faction_operator(true);
        }
        self.pending_faction_applications.insert(
            session_id,
            PendingFactionApplication {
                session_id,
                password,
                player_id,
                page: 1,
                expires_at_ms: now_ms.wrapping_add(2000),
            },
        );
        let _ = self.send_script_faction_list_request(session_id);
    }

    fn send_script_faction_list_request(&self, session_id: i64) -> bool {
        let Some(pending) = self.pending_faction_applications.get(&session_id) else {
            return false;
        };
        let mut request = CMessage::new(0x0006_0107);
        request.base_mut().add_long64(pending.session_id);
        request.add_long(pending.password);
        request.add_long(pending.player_id);
        request.add_long(pending.page);
        matches!(request.send(self, false), Ok(value) if value != 0)
    }

    pub(crate) fn continue_script_faction_application(
        &mut self,
        player_id: i32,
        session_id: i64,
    ) -> bool {
        let Some(pending) = self.pending_faction_applications.get_mut(&session_id) else {
            return false;
        };
        if pending.player_id != player_id {
            return false;
        }
        pending.page = pending.page.wrapping_add(1);
        self.send_script_faction_list_request(session_id)
    }

    pub(crate) fn finish_empty_script_faction_application(
        &mut self,
        player_id: i32,
        session_id: i64,
        password: i32,
    ) -> bool {
        let Some(pending) = self.pending_faction_applications.get(&session_id) else {
            return false;
        };
        if pending.player_id != player_id || pending.password != password {
            return false;
        }
        self.pending_faction_applications.remove(&session_id);
        if let Some(player) = self.find_player_mut(player_id) {
            player.set_apply_join_faction_operator(false);
        }
        true
    }

    pub(crate) fn select_script_faction_application(
        &mut self,
        player_id: i32,
        session_id: i64,
        password: i32,
        discarded: i32,
        accepted: i32,
        faction_name: &[u8],
    ) -> bool {
        let Some(pending) = self.pending_faction_applications.get(&session_id) else {
            return false;
        };
        if pending.player_id != player_id
            || pending.password != password
            || faction_name.len() > 20
            || faction_name.contains(&0)
        {
            return false;
        }
        if accepted != 0 {
            let mut request = CMessage::new(0x0006_0108);
            request.add_long(player_id);
            request.add_long(discarded);
            add_legacy_c_string(request.base_mut(), faction_name);
            let _ = request.send(self, false);
        }
        self.pending_faction_applications.remove(&session_id);
        if let Some(player) = self.find_player_mut(player_id) {
            player.set_apply_join_faction_operator(false);
        }
        true
    }

    pub(crate) fn script_faction_application_is_active(
        &self,
        player_id: i32,
        session_id: i64,
        password: i32,
    ) -> bool {
        self.pending_faction_applications
            .get(&session_id)
            .is_some_and(|pending| pending.player_id == player_id && pending.password == password)
    }

    pub(crate) fn start_script_faction_war_declaration(&mut self, player_id: i32, now_ms: u32) {
        let Some(player) = self.find_player(player_id) else {
            return;
        };
        if player.faction_id() <= 0 || player.faction_declare_operator() {
            return;
        }
        let (session_id, password) = self.next_organizing_correlation();
        if let Some(player) = self.find_player_mut(player_id) {
            player.set_faction_declare_operator(true);
        }
        self.pending_faction_war_declarations.insert(
            session_id,
            PendingFactionWarDeclaration {
                session_id,
                password,
                player_id,
                page: 1,
                declaration_pending: false,
                expires_at_ms: now_ms.wrapping_add(2000),
            },
        );
        let _ = self.send_script_faction_war_page_request(session_id);
    }

    fn send_script_faction_war_page_request(&self, session_id: i64) -> bool {
        let Some(pending) = self.pending_faction_war_declarations.get(&session_id) else {
            return false;
        };
        let mut request = CMessage::new(0x0006_011e);
        request.base_mut().add_long64(pending.session_id);
        request.add_long(pending.password);
        request.add_long(pending.player_id);
        request.add_long(pending.page);
        matches!(request.send(self, false), Ok(value) if value != 0)
    }

    pub(crate) fn continue_script_faction_war_page(
        &mut self,
        player_id: i32,
        session_id: i64,
        password: i32,
    ) -> bool {
        let Some(pending) = self.pending_faction_war_declarations.get_mut(&session_id) else {
            return false;
        };
        if pending.player_id != player_id || pending.password != password {
            return false;
        }
        pending.page = pending.page.wrapping_add(1);
        self.send_script_faction_war_page_request(session_id)
    }

    pub(crate) fn close_script_faction_war_declaration(
        &mut self,
        player_id: i32,
        session_id: i64,
        password: i32,
    ) -> bool {
        let Some(pending) = self.pending_faction_war_declarations.get(&session_id) else {
            return false;
        };
        if pending.player_id != player_id || pending.password != password {
            return false;
        }
        self.pending_faction_war_declarations.remove(&session_id);
        if let Some(player) = self.find_player_mut(player_id) {
            player.set_faction_declare_operator(false);
        }
        true
    }

    pub(crate) fn select_script_faction_war_target<Context: ScriptRegionChangeContext>(
        &mut self,
        player_id: i32,
        session_id: i64,
        password: i32,
        target_faction_id: i32,
        war_type: i32,
        context: &mut Context,
    ) -> bool {
        let Some(pending) = self.pending_faction_war_declarations.get(&session_id) else {
            return false;
        };
        if pending.player_id != player_id
            || pending.password != password
            || pending.declaration_pending
            || target_faction_id <= 0
            || war_type <= 0
        {
            return false;
        }
        let Some(player) = self.find_player(player_id) else {
            return false;
        };
        if player.faction_id() <= 0 {
            return false;
        }
        let mut snapshot = Vec::new();
        if !self.encode_player_game_save(player, &mut snapshot, context) {
            return false;
        }
        let mut request = CMessage::new(0x0006_011f);
        request.base_mut().add_long64(session_id);
        request.add_long(password);
        request.add_long(player_id);
        request.add_long(target_faction_id);
        request.add_long(war_type);
        request.base_mut().add(&snapshot);
        let sent = matches!(request.send(self, false), Ok(value) if value != 0);
        if sent {
            if let Some(pending) = self.pending_faction_war_declarations.get_mut(&session_id) {
                pending.declaration_pending = true;
            }
        }
        sent
    }

    pub(crate) fn script_faction_war_is_active(
        &self,
        player_id: i32,
        session_id: i64,
        password: i32,
    ) -> bool {
        self.pending_faction_war_declarations
            .get(&session_id)
            .is_some_and(|pending| pending.player_id == player_id && pending.password == password)
    }

    pub(crate) fn finish_script_faction_war_result(
        &mut self,
        player_id: i32,
        session_id: i64,
        password: i32,
        money: u32,
    ) -> Option<i32> {
        let pending = self.pending_faction_war_declarations.get_mut(&session_id)?;
        if pending.player_id != player_id
            || pending.password != password
            || !pending.declaration_pending
        {
            return None;
        }
        pending.declaration_pending = false;
        if money > 0 {
            if let Some(change) = self.decrease_player_money(player_id, money) {
                let _ = self.send_player_money_decrease(player_id, &change.outcome);
            }
        }
        let mut response = CMessage::new(0x000b_ff31);
        response.add_long(i32::from(money > 0));
        Some(response.send_to_player(self.net_server(), player_id))
    }

    pub(crate) fn expire_script_faction_sessions(&mut self, now_ms: u32) {
        let expired_creations: Vec<_> = self
            .pending_faction_creations
            .values()
            .filter(|pending| now_ms.wrapping_sub(pending.expires_at_ms) < 0x8000_0000)
            .map(|pending| (pending.session_id, pending.player_id))
            .collect();
        for (session_id, player_id) in expired_creations {
            self.pending_faction_creations.remove(&session_id);
            if let Some(player) = self.find_player_mut(player_id) {
                player.set_create_faction_operator(false);
            }
        }
        let expired_applications: Vec<_> = self
            .pending_faction_applications
            .values()
            .filter(|pending| now_ms.wrapping_sub(pending.expires_at_ms) < 0x8000_0000)
            .map(|pending| (pending.session_id, pending.player_id))
            .collect();
        for (session_id, player_id) in expired_applications {
            self.pending_faction_applications.remove(&session_id);
            if let Some(player) = self.find_player_mut(player_id) {
                player.set_apply_join_faction_operator(false);
            }
        }
        let expired_wars: Vec<_> = self
            .pending_faction_war_declarations
            .values()
            .filter(|pending| now_ms.wrapping_sub(pending.expires_at_ms) < 0x8000_0000)
            .map(|pending| (pending.session_id, pending.player_id))
            .collect();
        for (session_id, player_id) in expired_wars {
            self.pending_faction_war_declarations.remove(&session_id);
            if let Some(player) = self.find_player_mut(player_id) {
                player.set_faction_declare_operator(false);
            }
        }
    }

    pub(crate) fn apply_script_faction_upgrade_debit(
        &mut self,
        player_id: i32,
        money: u32,
        goods_name: &[u8],
    ) -> bool {
        if self.find_player(player_id).is_none() {
            return false;
        }
        if let Some(change) = self.decrease_player_money(player_id, money) {
            let _ = self.send_player_money_decrease(player_id, &change.outcome);
        }
        let base_index = self
            .goods_factory
            .query_goods_id_by_original_name(Some(goods_name));
        let consumptions = self
            .find_player_mut(player_id)
            .map(|player| player.remove_item_in_packet(base_index, 1))
            .unwrap_or_default();
        for consumption in consumptions {
            let _ = self.send_player_packet_consumption(&consumption);
        }
        true
    }

    pub(crate) fn script_city_gate_state(&self, region_id: i32, gate_id: i32) -> i32 {
        match self.find_region(region_id) {
            Some(ServerRegionOwner::City(region)) => region.get_city_gate_state(gate_id),
            _ => -1,
        }
    }

    pub(crate) fn script_city_gate_can_close<Context: CityGateRuntimeContext>(
        &self,
        region_id: i32,
        gate_id: i32,
        context: &Context,
    ) -> bool {
        match self.find_region(region_id) {
            Some(ServerRegionOwner::City(region)) => region.city_gate_is_close(gate_id, context),
            _ => false,
        }
    }

    pub(crate) fn operate_script_city_gate<Context: CityGateRuntimeContext>(
        &mut self,
        region_id: i32,
        gate_id: i32,
        operation: i32,
        context: &mut Context,
    ) -> bool {
        let Some(ServerRegionOwner::City(mut region)) = self.take_region_owner(region_id) else {
            return false;
        };
        let operated = region.operator_city_gate(gate_id, operation, context);
        if operated {
            region.update_city_gate_to_client(gate_id, context);
        }
        self.restore_region_owner(ServerRegionOwner::City(region));
        operated
    }

    /// Общий `public::random(max)` script-owner использует тот же process-wide
    /// MSVCRT stream, что goods, equipment и battle-fairy gameplay.
    pub(crate) fn script_random(&mut self, maximum: i32) -> i32 {
        game_legacy_random(&mut self.random_state, maximum)
    }

    /// Exact `RunScript` owner: загруженный instance получает wrapping ID и
    /// попадает в ordered `g_Scripts`; команды исполняет только Script-stage
    /// главного цикла. Повтор того же файла у того же player отклоняется как
    /// исходным `ScriptIfExit`.
    pub(crate) fn run_script_file<Runtime: ScriptFunctionRuntime>(
        &mut self,
        path: &[u8],
        context: ScriptExecutionContext,
        _runtime: &mut Runtime,
    ) -> Option<i32> {
        let player_id = context.player_id?;
        let path = legacy_c_string_prefix(path).to_vec();
        if self
            .active_scripts
            .values()
            .any(|script| script.player_id() == Some(player_id) && script.path == path)
        {
            return None;
        }
        let waiting_ids: Vec<i32> = self
            .active_scripts
            .iter()
            .filter_map(|(id, script)| {
                (script.player_id() == Some(player_id)
                    && matches!(script.waiting_function(), Some(2307 | 2324)))
                .then_some(*id)
            })
            .collect();
        for waiting_id in waiting_ids {
            let _ = self.delete_player_script(waiting_id, player_id, true);
        }
        let source = self.script_file_data(&path)?.to_vec();
        self.next_script_id = self.next_script_id.wrapping_add(1);
        let script_id = self.next_script_id;
        self.active_scripts.insert(
            script_id,
            ActiveScript::new(script_id, path, source, context),
        );
        Some(script_id)
    }

    pub(crate) fn run_script_loop<Runtime: ScriptFunctionRuntime>(
        &mut self,
        runtime: &mut Runtime,
    ) -> ScriptLoopReport {
        let mut report = ScriptLoopReport::default();
        let mut pending: Vec<i32> = self.active_scripts.keys().copied().collect();
        let mut position = 0usize;
        while let Some(script_id) = pending.get(position).copied() {
            position += 1;
            let Some(mut script) = self.active_scripts.remove(&script_id) else {
                continue;
            };
            if !script
                .player_id()
                .is_some_and(|player_id| self.find_player(player_id).is_some())
            {
                report.ended_scripts.push(script_id);
                continue;
            }
            let step = script.run_step(self, runtime);
            let disposition = step.disposition.clone();
            report.steps.push((script_id, step));
            match disposition {
                ScriptStepDisposition::Ended => report.ended_scripts.push(script_id),
                ScriptStepDisposition::YieldedCall { path } => {
                    let context = script.context();
                    self.active_scripts.insert(script_id, script);
                    if let Some(called_id) = self.run_script_file(&path, context, runtime) {
                        report.started_scripts.push(called_id);
                        pending.push(called_id);
                    }
                }
                ScriptStepDisposition::WaitingFunction { .. } => {
                    self.active_scripts.insert(script_id, script);
                }
            }
        }
        report
    }

    pub(crate) fn continue_player_script(
        &mut self,
        script_id: i32,
        player_id: i32,
        value: i32,
    ) -> bool {
        self.active_scripts
            .get_mut(&script_id)
            .filter(|script| script.player_id() == Some(player_id))
            .is_some_and(|script| script.continue_with(value))
    }

    pub(crate) fn player_script_is_running(
        &self,
        player_id: i32,
        path: &[u8],
        current_script_id: i32,
        current_script_path: &[u8],
    ) -> bool {
        (current_script_id != 0
            && current_script_path == path
            && self.find_player(player_id).is_some())
            || self
                .active_scripts
                .values()
                .any(|script| script.player_id() == Some(player_id) && script.path == path)
    }

    /// Safe `RemovePlayerScripts`: registry entries удаляются сразу, а
    /// текущий вынутый scheduler-ом instance сообщает caller-у terminal
    /// disposition и не возвращается в map после команды.
    pub(crate) fn remove_player_scripts(
        &mut self,
        player_id: i32,
        path: &[u8],
        current_script_id: i32,
        current_script_path: &[u8],
    ) -> bool {
        self.active_scripts
            .retain(|_, script| script.player_id() != Some(player_id) || script.path != path);
        current_script_id != 0
            && current_script_path == path
            && self.find_player(player_id).is_some()
    }

    pub(crate) fn delete_player_script(
        &mut self,
        script_id: i32,
        player_id: i32,
        close_talk_box: bool,
    ) -> bool {
        let Some(script) = self
            .active_scripts
            .get(&script_id)
            .filter(|script| script.player_id() == Some(player_id))
        else {
            return false;
        };
        let should_close = close_talk_box && matches!(script.waiting_function(), Some(2307 | 2324));
        self.active_scripts.remove(&script_id);
        if should_close {
            let mut message = CMessage::new(0x000b_f805);
            message.add_long(script_id);
            message.add_byte(0);
            message.add_byte(0);
            let _ = message.send_to_player(self.net_server(), player_id);
        }
        true
    }

    pub(crate) const fn general_variables(&self) -> &CVariableList {
        &self.general_variables
    }

    pub(crate) fn set_general_variable_integer(
        &mut self,
        name: &[u8],
        value: i32,
    ) -> GameVariableMutationOutcome {
        self.general_variables.set_integer(name, 0, value)
    }

    pub(crate) fn set_general_variable_string(
        &mut self,
        name: &[u8],
        value: &[u8],
    ) -> GameVariableMutationOutcome {
        self.general_variables.set_string(name, value)
    }

    /// Reached `SetMe` для DWORD-полей восстановительного script-а. До записи
    /// публикуется общий property packet `0xBF80C`, затем direct storage,
    /// `UpdateProperty` и адресный `0xBF721`. `dwExp` завершает наблюдаемый
    /// no-level tail `CheckLevel` сообщением `0xBF704`.
    pub(crate) fn set_script_player_property<Context>(
        &mut self,
        player_id: i32,
        property: &[u8],
        value: i32,
        context: &mut Context,
    ) -> Option<i32>
    where
        Context: GameContainerMessageRuntime + BattleFairyDeathContext + NationCombatContext,
    {
        let (region_id, identity) = self
            .find_player(player_id)
            .map(|player| (player.server_region_id(), player.shape().identity()))?;
        if let Some(region_id) = region_id
            && let Some(region) = self.find_region(region_id)
            && let Some(player) = self.find_player(player_id)
        {
            let mut changed = CMessage::new(0x000b_f80c);
            changed.add_long(identity.object_type);
            changed.add_long(identity.id);
            add_legacy_c_string(changed.base_mut(), property);
            changed.add_long(value);
            let _ =
                context.send_nation_player_around(region.base(), player.shape(), None, &changed);
        }
        let recomputed = {
            let player = self.players.get_mut(&player_id)?;
            let _ = player.set_script_value(property, value)?;
            context.recompute_enhancement_player_properties(player)
        };
        self.players
            .get_mut(&player_id)
            .expect("script-player сохранён между write и UpdateProperty")
            .apply_recomputed_combat_properties(recomputed);

        let external = context.player_properties_external_facts(player_id);
        let player = self
            .players
            .get(&player_id)
            .expect("script-player сохранён до OnChangeProperties");
        let _ = self.send_player_properties_changed(player, external);
        if property.eq_ignore_ascii_case(b"dwExp") {
            let mut result = CMessage::new(0x000b_f704);
            result.add_ulong(player.experience());
            result.add_ulong(player.vigour());
            result.add_ulong(player.fetch_power());
            let _ = result.send_to_player(self.net_server(), player_id);
        }
        Some(value)
    }

    /// `ChangePlayer` меняет локальное named property, затем исполняет
    /// обязательный virtual `UpdateProperty`. В отличие от `SetPlayer`, этот
    /// selector не публикует отдельный `0xBF80C`.
    pub(crate) fn change_named_script_player_property<Context>(
        &mut self,
        player_name: &[u8],
        property: &[u8],
        delta: i32,
        context: &mut Context,
    ) -> Option<i32>
    where
        Context: GameContainerMessageRuntime + BattleFairyDeathContext + NationCombatContext,
    {
        let player_id = self.find_player_by_name(player_name)?.player_id();
        let applied = self
            .players
            .get_mut(&player_id)?
            .change_script_value(property, delta)
            .unwrap_or(0);
        let recomputed = {
            let player = self.players.get(&player_id)?;
            context.recompute_enhancement_player_properties(player)
        };
        self.players
            .get_mut(&player_id)
            .expect("named script-player сохранён до ChangePlayer UpdateProperty")
            .apply_recomputed_combat_properties(recomputed);
        let external = context.player_properties_external_facts(player_id);
        let player = self
            .players
            .get(&player_id)
            .expect("named script-player сохранён до ChangePlayer OnChangeProperties");
        let _ = self.send_player_properties_changed(player, external);
        Some(applied)
    }

    /// `SetPlayer` записывает локальное named property, исполняет обязательный
    /// `UpdateProperty`, затем отправляет адресный legacy `0xBF80C` и общий
    /// `0xBF721` property tail. World mutation этим owner-ом не публикуется.
    pub(crate) fn set_named_script_player_property<Context>(
        &mut self,
        player_name: &[u8],
        property: &[u8],
        value: i32,
        context: &mut Context,
    ) -> Option<i32>
    where
        Context: GameContainerMessageRuntime + BattleFairyDeathContext + NationCombatContext,
    {
        let player_id = self.find_player_by_name(player_name)?.player_id();
        let applied = self
            .players
            .get_mut(&player_id)?
            .set_script_value(property, value)
            .unwrap_or(0);

        let recomputed = {
            let player = self.players.get(&player_id)?;
            context.recompute_enhancement_player_properties(player)
        };
        self.players
            .get_mut(&player_id)
            .expect("named script-player сохранён до UpdateProperty")
            .apply_recomputed_combat_properties(recomputed);
        let external = context.player_properties_external_facts(player_id);
        let player = self
            .players
            .get(&player_id)
            .expect("named script-player сохранён до OnChangeProperties");
        let _ = self.send_player_properties_changed(player, external);
        let mut changed = CMessage::new(0x000b_f80c);
        changed.add_long(400);
        changed.add_long(player_id);
        add_legacy_c_string(changed.base_mut(), property);
        changed.add_long(value);
        let _ = changed.send_to_player(self.net_server(), player_id);
        Some(applied)
    }

    /// `SetPlayerLevel`: local `SetLevel + SetExp(0)` и его faction/client
    /// publications либо exact World relay для игрока другого GameServer-а.
    pub(crate) fn set_script_player_level(
        &mut self,
        script_player_id: i32,
        target_name: &[u8],
        level: u8,
    ) -> i32 {
        let target_id = if target_name.is_empty() {
            self.find_player(script_player_id).map(CPlayer::player_id)
        } else {
            self.find_player_by_name(target_name)
                .map(CPlayer::player_id)
        };
        let Some(target_id) = target_id else {
            let mut request = CMessage::new(0x0005_fc04);
            add_legacy_c_string(request.base_mut(), target_name);
            request.add_byte(level);
            let _ = request.send(self, false);
            return 0;
        };
        let mutation = self
            .find_player_mut(target_id)
            .expect("script level target проверен до mutation")
            .apply_remote_level(level);
        if mutation.previous_level != level && mutation.faction_id > 0 {
            let mut faction = CMessage::new(0x0006_012a);
            faction.add_long(mutation.faction_id);
            faction.add_long(mutation.player_id);
            faction.add_long(1);
            faction.add_ulong(u32::from(level));
            let _ = faction.send(self, false);
        }
        let mut response = CMessage::new(0x000b_f708);
        response.add_byte(level);
        response.add_ulong(0);
        response.add_ulong(self.player_list.level_experience(level));
        let _ = response.send_to_player(self.net_server(), target_id);
        0
    }

    /// `DelSkill`/`SetSkillLevel` используют общий local/World routing, но
    /// local client result исторически адресован запускающему script игроку.
    pub(crate) fn delete_script_player_skill(
        &mut self,
        script_player_id: i32,
        target_name: &[u8],
        skill_name: &[u8],
    ) -> i32 {
        let target_id = if target_name.is_empty() {
            self.find_player(script_player_id).map(CPlayer::player_id)
        } else {
            self.find_player_by_name(target_name)
                .map(CPlayer::player_id)
        };
        let Some(target_id) = target_id else {
            let mut request = CMessage::new(0x0005_fc02);
            add_legacy_c_string(request.base_mut(), target_name);
            add_legacy_c_string(request.base_mut(), skill_name);
            let _ = request.send(self, false);
            return -1;
        };
        let mutation = self
            .delete_remote_player_skill(target_id, skill_name)
            .expect("script skill target проверен до mutation");
        let mut response = CMessage::new(0x000b_f71e);
        add_legacy_c_string(response.base_mut(), skill_name);
        let _ = response.send_to_player(self.net_server(), script_player_id);
        i32::from(mutation.legacy_result)
    }

    pub(crate) fn set_script_player_skill_level(
        &mut self,
        script_player_id: i32,
        target_name: &[u8],
        skill_name: &[u8],
        level: i32,
    ) -> i32 {
        let target_id = if target_name.is_empty() {
            self.find_player(script_player_id).map(CPlayer::player_id)
        } else {
            self.find_player_by_name(target_name)
                .map(CPlayer::player_id)
        };
        let Some(target_id) = target_id else {
            let mut request = CMessage::new(0x0005_fc01);
            add_legacy_c_string(request.base_mut(), target_name);
            add_legacy_c_string(request.base_mut(), skill_name);
            request.base_mut().add_short(level as i16);
            request.add_long(script_player_id);
            let _ = request.send(self, false);
            return -1;
        };
        let mutation = {
            let (players, skill_factory) = (&mut self.players, &self.skill_factory);
            players
                .get_mut(&target_id)
                .and_then(|player| player.set_script_skill_level(skill_name, level, skill_factory))
        };
        if let Some(mutation) = mutation {
            if let Some(response) = player_skill_learned_message(
                0x000b_f71d,
                mutation.skill_id,
                mutation.skill_level,
                level,
                skill_name,
                &self.skill_factory,
                false,
            ) {
                let _ = response.send_to_player(self.net_server(), script_player_id);
            }
            i32::from(mutation.legacy_result)
        } else {
            0
        }
    }

    pub(crate) fn script_player_skill_level(
        &self,
        script_player_id: i32,
        target_name: &[u8],
        skill_name: &[u8],
    ) -> i32 {
        let target = if target_name.is_empty() {
            self.find_player(script_player_id)
        } else {
            self.find_player_by_name(target_name)
        };
        let Some(target) = target else {
            return -1;
        };
        let skill_id = self.skill_factory.query_skill_id(Some(skill_name));
        target.item_skill_level(skill_id)
    }

    /// `AddSkill` достигнут из goods-script runtime: обычный skill публикует
    /// адресный `TellClient(0xBF71D)` целевому игроку, а внутренние realm title/
    /// bonus skills подавляют этот packet и проходят полный property recompute.
    pub(crate) fn add_script_player_skill<Context: RealmAppellationScriptContext>(
        &mut self,
        script_player_id: i32,
        target_name: &[u8],
        skill_name: &[u8],
        level: i32,
        context: &mut Context,
    ) -> i32 {
        let target_id = if target_name.is_empty() {
            self.find_player(script_player_id).map(CPlayer::player_id)
        } else {
            self.find_player_by_name(target_name)
                .map(CPlayer::player_id)
        };
        let Some(target_id) = target_id else {
            return -1;
        };
        let previous_properties = self
            .find_player(target_id)
            .expect("script skill target проверен до mutation")
            .combat_properties();
        let mutation = {
            let (players, skill_factory) = (&mut self.players, &self.skill_factory);
            players
                .get_mut(&target_id)
                .and_then(|player| player.set_script_skill_level(skill_name, level, skill_factory))
        };
        let Some(mutation) = mutation else {
            return 0;
        };

        if crate::gameserver::appserver::skills::realmappellation::is_internal_skill(
            mutation.skill_id,
        ) {
            let current_properties = {
                let player = self
                    .find_player(target_id)
                    .expect("realm skill mutation сохраняет canonical player");
                context.recompute_realm_appellation_player_properties(player)
            };
            self.find_player_mut(target_id)
                .expect("realm skill recompute сохраняет canonical player")
                .apply_recomputed_combat_properties(current_properties);
            if previous_properties != current_properties {
                let external = context.player_properties_external_facts(target_id);
                if let Some(player) = self.find_player(target_id) {
                    let _ = self.send_player_properties_changed(player, external);
                }
            }
        } else if let Some(response) = player_skill_learned_message(
            0x000b_f71d,
            mutation.skill_id,
            mutation.skill_level,
            mutation.skill_level,
            skill_name,
            &self.skill_factory,
            true,
        ) {
            let _ = response.send_to_player(self.net_server(), target_id);
        }

        i32::from(mutation.legacy_result)
    }

    pub(crate) fn add_script_player_quest(&mut self, player_id: i32, quest_id: u16) {
        if !self.players.contains_key(&player_id) {
            let mut request = CMessage::new(0x0006_013b);
            request.add_long(player_id);
            request.base_mut().add_short(quest_id as i16);
            let _ = request.send(self, false);
            return;
        }
        let Some(quest) = self.quest_system.quest_data_by_id(quest_id) else {
            return;
        };
        let player = self
            .players
            .get_mut(&player_id)
            .expect("local quest-player проверен до mutation");
        if !player.accept_script_quest(quest_id) {
            return;
        }

        let mut message = CMessage::new(0x000b_ff2c);
        message.base_mut().add_short(quest_id as i16);
        message.add_ulong(quest.old);
        message.add_ulong(quest.quest_type);
        message.add_ulong(quest.level);
        message.add_ulong(quest.difficulty);
        message.add_ulong(quest.track);
        add_legacy_c_string(message.base_mut(), &quest.short_description);
        add_legacy_c_string(message.base_mut(), &quest.name);
        add_legacy_c_string(message.base_mut(), &quest.description);
        message.add_byte(u8::from(quest.display));
        message.add_long(quest.region_id);
        message.add_long(quest.tile_x);
        message.add_long(quest.tile_y);
        message.add_long(quest.effect_id);
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    pub(crate) fn complete_script_player_quest(&mut self, player_id: i32, quest_id: u16) {
        let Some(quest_name) = self
            .quest_system
            .quest_data_by_id(quest_id)
            .map(|quest| quest.name.clone())
        else {
            return;
        };
        let Some(player) = self.players.get_mut(&player_id) else {
            return;
        };
        if !player.complete_script_quest(quest_id) {
            return;
        }

        let mut message = CMessage::new(0x000b_ff2d);
        message.base_mut().add_short(quest_id as i16);
        add_legacy_c_string(message.base_mut(), &quest_name);
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    pub(crate) fn remove_script_player_quest(&mut self, player_id: i32, quest_id: u16) {
        if !self.players.contains_key(&player_id) {
            let mut request = CMessage::new(0x0006_013c);
            request.add_long(player_id);
            request.base_mut().add_short(quest_id as i16);
            let _ = request.send(self, false);
            return;
        }
        let Some(quest_name) = self
            .quest_system
            .quest_data_by_id(quest_id)
            .map(|quest| quest.name.clone())
        else {
            return;
        };
        let player = self
            .players
            .get_mut(&player_id)
            .expect("local quest-player проверен до removal");
        if !player.remove_script_quest(quest_id) {
            return;
        }

        let mut message = CMessage::new(0x000b_ff2e);
        message.base_mut().add_short(quest_id as i16);
        add_legacy_c_string(message.base_mut(), &quest_name);
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    pub(crate) fn update_script_player_quest_position(
        &self,
        player_id: i32,
        quest_id: u16,
        region_id: i32,
        tile_x: i32,
        tile_y: i32,
    ) {
        let Some(player) = self.players.get(&player_id) else {
            return;
        };
        if !player.has_script_quest(quest_id) {
            return;
        }

        let mut message = CMessage::new(0x000b_ff2f);
        message.base_mut().add_short(quest_id as i16);
        message.add_long(region_id);
        message.add_long(tile_x);
        message.add_long(tile_y);
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    pub(crate) fn set_script_player_quest_enabled(&mut self, player_id: i32, enabled: bool) {
        let Some(player) = self.players.get_mut(&player_id) else {
            return;
        };
        player.set_quest_enabled(enabled);
        let mut message = CMessage::new(0x000b_f728);
        message.add_byte(u8::from(enabled));
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    pub(crate) fn begin_script_player_quest_time(
        &mut self,
        player_id: i32,
        now_seconds: i32,
        time_limit: i32,
    ) {
        let Some(player) = self.players.get_mut(&player_id) else {
            return;
        };
        player.begin_quest_time(now_seconds, time_limit);
        let mut message = CMessage::new(0x000b_f729);
        message.add_long(time_limit);
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    pub(crate) fn clear_script_player_quest_time(&mut self, player_id: i32) {
        let Some(player) = self.players.get_mut(&player_id) else {
            return;
        };
        player.clear_quest_time();
        let message = CMessage::new(0x000b_f72a);
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    /// Reached `ChangeRegion` gameplay path used by `nodupe.script`. The
    /// selector/default parsing remains in `CScript::RunFunction`; this owner
    /// performs session/player state, spatial randomization and exact client /
    /// World wire in the original order.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn change_script_player_region<
        Context: ScriptRegionChangeContext + RealmAppellationScriptContext,
    >(
        &mut self,
        player_id: i32,
        target_region_id: i32,
        mut tile_x: i32,
        mut tile_y: i32,
        mut direction: i32,
        use_goods: i32,
        range: i32,
        carriage_distance: i32,
        context: &mut Context,
    ) -> ScriptRegionChangeReport {
        let mut report = ScriptRegionChangeReport {
            player_id,
            source_region_id: None,
            target_region_id,
            tile_x,
            tile_y,
            direction,
            use_goods,
            range,
            carriage_distance,
            kind: ScriptRegionChangeKind::MissingPlayer,
            position_delivery: None,
            direction_delivery: None,
            region_delivery: None,
            faction_delivery: None,
            team_delivery: None,
            world_delivery: None,
            player_snapshot_size: None,
        };
        if self
            .find_player(player_id)
            .and_then(CPlayer::server_region_id)
            .is_some_and(|source_region_id| source_region_id != target_region_id)
        {
            self.change_body_after_region_transition(player_id, context);
        }
        let Some(mut player) = self.players.remove(&player_id) else {
            return report;
        };
        let Some(source_region_id) = player.server_region_id() else {
            self.players.insert(player_id, player);
            return report;
        };
        report.source_region_id = Some(source_region_id);
        let Some(mut source_owner) = self.take_region_owner(source_region_id) else {
            report.kind = ScriptRegionChangeKind::MissingSourceRegion;
            self.players.insert(player_id, player);
            return report;
        };

        let previous_progress = player.current_progress();
        context.finish_script_player_business(&mut player);
        if matches!(
            previous_progress,
            PlayerProgress::Trading
                | PlayerProgress::OpenStall
                | PlayerProgress::Upgrade
                | PlayerProgress::DaKong
                | PlayerProgress::Compose
        ) && let Some(session_id) = self
            .session_factory
            .query_session_id_by_owner(400, player_id)
        {
            let _ = self.session_factory.end_session(session_id);
        }
        player.set_current_progress_snapshot(PlayerProgress::None);

        if !(0..8).contains(&direction) {
            direction = context.random_below(8);
        }
        report.direction = direction;
        if target_region_id == source_region_id {
            report.kind = ScriptRegionChangeKind::SameRegion;
            context.prepare_script_region_companions(
                &mut player,
                source_region_id,
                target_region_id,
                tile_x,
                tile_y,
                carriage_distance,
                false,
            );
            if tile_x == -1 && tile_y == -1 {
                if let Ok(position) = source_owner.base().region.get_random_pos(context) {
                    tile_x = position.x;
                    tile_y = position.y;
                }
            } else if range > 0 {
                let width = range.wrapping_mul(2).wrapping_add(1);
                if let Ok(position) = source_owner.base().region.get_random_pos_in_range(
                    tile_x.wrapping_sub(range),
                    tile_y.wrapping_sub(range),
                    width,
                    width,
                    context,
                ) {
                    tile_x = position.x;
                    tile_y = position.y;
                }
            }
            let previous = (
                player.shape().get_tile_x().unwrap_or_default(),
                player.shape().get_tile_y().unwrap_or_default(),
            );
            if previous != (tile_x, tile_y) {
                let mut movement = CMessage::new(0x000b_f603);
                movement.add_long(400);
                movement.add_long(player_id);
                movement.add_long(tile_x);
                movement.add_long(tile_y);
                movement.add_long(use_goods);
                report.position_delivery = Some(context.send_nation_player_around(
                    source_owner.base(),
                    player.shape(),
                    None,
                    &movement,
                ));
                let facts = player.movement_position_facts(
                    self.globe_setup.area_width(),
                    self.globe_setup.area_height(),
                );
                let _ = source_owner.base_mut().set_move_shape_tile_position(
                    player.movement_shape_mut(),
                    tile_x,
                    tile_y,
                    facts,
                );
            }
            if player.shape().get_direction() != direction {
                player.movement_shape_mut().set_direction(direction);
                let mut changed = CMessage::new(0x000b_f601);
                changed.add_byte(direction as u8);
                changed.add_long(400);
                changed.add_long(player_id);
                report.direction_delivery = Some(context.send_nation_player_around(
                    source_owner.base(),
                    player.shape(),
                    None,
                    &changed,
                ));
            }
            context.refresh_script_region_auto_protect(&mut player);
            report.tile_x = tile_x;
            report.tile_y = tile_y;
            self.restore_region_owner(source_owner);
            self.players.insert(player_id, player);
            return report;
        }

        if let Some(target_owner) = self.take_region_owner(target_region_id) {
            report.kind = ScriptRegionChangeKind::LocalRegion;
            context.prepare_script_region_companions(
                &mut player,
                source_region_id,
                target_region_id,
                tile_x,
                tile_y,
                carriage_distance,
                false,
            );
            if tile_x == -1 && tile_y == -1 {
                if let Ok(position) = target_owner.base().region.get_random_pos(context) {
                    tile_x = position.x;
                    tile_y = position.y;
                }
            } else if range > 0 {
                let width = range.wrapping_mul(2).wrapping_add(1);
                if let Ok(position) = target_owner.base().region.get_random_pos_in_range(
                    tile_x.wrapping_sub(range),
                    tile_y.wrapping_sub(range),
                    width,
                    width,
                    context,
                ) {
                    tile_x = position.x;
                    tile_y = position.y;
                }
            }

            let target = target_owner.base();
            let mut changed = CMessage::new(0x000b_f505);
            changed.add_long(400);
            changed.add_long(player_id);
            changed.add_long(target_region_id);
            changed.add_long(target.region.region_type());
            changed.add_long(tile_x);
            changed.add_long(tile_y);
            changed.add_long(direction);
            add_legacy_c_string(changed.base_mut(), target.region.file_name());
            changed.add_long(target.region.resource_id());
            changed.add_long(target.war_region_type);
            changed.add_byte(target.country);
            changed.add_ulong(target.region.exp_scale_bits());
            report.region_delivery = Some(context.send_nation_player_around(
                source_owner.base(),
                player.shape(),
                None,
                &changed,
            ));

            player.stage_local_region_change(target_region_id, tile_x, tile_y, direction);
            let _ = source_owner
                .base_mut()
                .stage_region_transition(player.shape());
            if player.faction_id() > 0 {
                let mut faction = CMessage::new(0x0006_012a);
                faction.add_long(player.faction_id());
                faction.add_long(player_id);
                faction.add_long(2);
                faction.add_long(target_region_id);
                report.faction_delivery = Some(faction.send(self, false));
            }
            if player.team_id() != 0 {
                let mut team = CMessage::new(0x0006_0005);
                team.add_long(player.team_id());
                team.add_long(player_id);
                team.add_long(source_region_id);
                team.add_long(target_region_id);
                report.team_delivery = Some(team.send(self, false));
            }
            context.refresh_script_region_auto_protect(&mut player);
            report.tile_x = tile_x;
            report.tile_y = tile_y;
            self.restore_region_owner(target_owner);
            self.restore_region_owner(source_owner);
            self.players.insert(player_id, player);
            return report;
        }

        report.kind = ScriptRegionChangeKind::RemoteServer;
        context.prepare_script_region_companions(
            &mut player,
            source_region_id,
            target_region_id,
            tile_x,
            tile_y,
            carriage_distance,
            true,
        );
        player.begin_server_region_change();
        let mut snapshot = Vec::new();
        if self.encode_player_game_save(&player, &mut snapshot, context) {
            let mut request = CMessage::new(0x0005_fa02);
            request.add_long(player_id);
            request.add_long(target_region_id);
            request.add_long(tile_x);
            request.add_long(tile_y);
            request.add_long(direction);
            request.add_long(use_goods);
            request.add_long(range);
            request.base_mut().add(&snapshot);
            report.player_snapshot_size = Some(snapshot.len());
            let delivery = request.send(self, false);
            if !matches!(delivery, Ok(value) if value != 0) {
                player.cancel_server_region_change();
            }
            report.world_delivery = Some(delivery);
        } else {
            player.cancel_server_region_change();
        }
        context.refresh_script_region_auto_protect(&mut player);
        report.tile_x = tile_x;
        report.tile_y = tile_y;
        self.restore_region_owner(source_owner);
        self.players.insert(player_id, player);
        report
    }

    /// Successful World `0x7F802` tail corresponding to `CPlayer::OnLost`
    /// while `m_bInChangingServer` is set. The player leaves the source
    /// spatial registry and all owned scripts before its client route is
    /// released for the destination GameServer.
    pub(crate) fn finish_script_server_region_departure(
        &mut self,
        player_id: i32,
    ) -> Option<Result<(), RegionMembershipBlock>> {
        let mut player = self.players.remove(&player_id)?;
        let Some(region_id) = player.server_region_id() else {
            self.players.insert(player_id, player);
            return None;
        };
        let Some(mut owner) = self.take_region_owner(region_id) else {
            self.players.insert(player_id, player);
            return None;
        };
        let facts = ShapeRuntimeFacts {
            is_player: true,
            monster: None,
            is_npc: false,
            goods: None,
            is_move_shape: true,
            figure: player.figure(),
        };
        let removal = owner
            .base_mut()
            .remove_object(player.movement_shape_mut(), facts);
        self.restore_region_owner(owner);
        self.active_scripts
            .retain(|_, script| script.player_id() != Some(player_id));
        let _ = self.net_server().clear_player_map_id(player_id);
        Some(removal)
    }

    /// Client `8F801` acknowledgement completes a deferred local change. The
    /// player is added to the destination registry before snapshots/weather
    /// and state callbacks, matching `CServerRegion::OnMessage`.
    pub(crate) fn enter_changed_player_region<Context: GameRegionEnterContext>(
        &mut self,
        player_id: i32,
        region_id: i32,
        entry_token: i32,
        client_ip: u32,
        socket_id: i32,
        context: &mut Context,
    ) -> Option<GameRegionEnterReport> {
        let mut player = self.players.remove(&player_id)?;
        let previous_changing_region = player.in_changing_region();
        if !previous_changing_region || player.server_region_id() != Some(region_id) {
            self.players.insert(player_id, player);
            return None;
        }
        let Some(mut owner) = self.take_region_owner(region_id) else {
            self.players.insert(player_id, player);
            return None;
        };
        player.set_changing_state_snapshot(player.in_changing_server(), false);
        player.set_client_ip_snapshot(client_ip);
        let mut relocation = None;
        let current = (
            player.shape().get_tile_x().unwrap_or_default(),
            player.shape().get_tile_y().unwrap_or_default(),
        );
        if owner
            .base()
            .region
            .get_block(current.0, current.1)
            .unwrap_or_default()
            != 0
            && let Ok(position) = owner
                .base()
                .region
                .get_random_pos_in_range(current.0, current.1, 3, 3, context)
        {
            player
                .movement_shape_mut()
                .set_pos_xy_base(position.x as f32 + 0.5, position.y as f32 + 0.5);
            let mut moved = CMessage::new(0x000b_f603);
            moved.add_long(400);
            moved.add_long(player_id);
            moved.add_long(position.x);
            moved.add_long(position.y);
            moved.add_long(0);
            relocation = Some((
                position.x,
                position.y,
                moved.send_to_socket(self.net_server(), socket_id),
            ));
        }
        let facts = ShapeRuntimeFacts {
            is_player: true,
            monster: None,
            is_npc: false,
            goods: None,
            is_move_shape: true,
            figure: player.figure(),
        };
        let membership = owner.base_mut().add_object(
            player.movement_shape_mut(),
            facts,
            self.globe_setup.area_width(),
            self.globe_setup.area_height(),
            context.now_milliseconds(),
            context,
        );
        self.restore_region_owner(owner);
        self.players.insert(player_id, player);
        if membership.is_ok() {
            context.publish_changed_player_region_entry(
                self,
                player_id,
                region_id,
                entry_token,
                socket_id,
            );
        }
        Some(GameRegionEnterReport {
            player_id,
            region_id,
            entry_token,
            previous_changing_region,
            relocation,
            membership,
        })
    }

    /// Очищает и декодирует language table, пишет exact log и лишь затем
    /// сдвигает внешний message cursor на consumed length.
    pub(crate) fn create_string_table(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        mut add_log_text: impl FnMut(&[u8]),
    ) -> Result<MyStringTableDecodeReport, MyStringTableDecodeError> {
        let start = *cursor;
        self.string_table.table_mut().free();
        let payload = source.get(start..).unwrap_or_default();
        let report = self.string_table.from_byte_array(payload)?;
        if report.unique_entries == 0 {
            add_log_text(b"WARNING : Received a NULL language packet from WorldServer!");
        } else {
            add_log_text(b"Received Language packet from WorldServer OK!");
        }
        *cursor = start
            .checked_add(report.consumed)
            .expect("MyStringTable consumed length помещается в message cursor");
        Ok(report)
    }

    /// Exact `GetStringByID` fallback: отсутствующий key возвращает пустую строку.
    pub(crate) fn get_string_by_id(&self, id: &[u8]) -> &[u8] {
        self.string_table
            .table()
            .get_string_by_id(id)
            .unwrap_or_default()
    }

    pub(crate) const fn quest_system(&self) -> &CQuestSystem {
        &self.quest_system
    }

    pub(crate) const fn quest_system_mut(&mut self) -> &mut CQuestSystem {
        &mut self.quest_system
    }

    pub(crate) const fn country_param(&self) -> &CCountryParam {
        &self.country_param
    }

    pub(crate) const fn country_param_mut(&mut self) -> &mut CCountryParam {
        &mut self.country_param
    }

    pub(crate) const fn country_handler(&self) -> &CCountryHandler {
        &self.country_handler
    }

    pub(crate) const fn country_handler_mut(&mut self) -> &mut CCountryHandler {
        &mut self.country_handler
    }

    pub(crate) fn player_country_identity(&mut self, player_id: i32) -> u8 {
        let Some((country_id, player_id)) = self
            .find_player(player_id)
            .map(|player| (player.country(), player.player_id()))
        else {
            return 0;
        };
        self.country_handler
            .country_mut(country_id)
            .map_or(0, |country| country.identity_for_player(player_id))
    }

    pub(crate) const fn attack_city_sys(&self) -> &CAttackCitySys {
        &self.attack_city_sys
    }

    pub(crate) const fn attack_city_sys_mut(&mut self) -> &mut CAttackCitySys {
        &mut self.attack_city_sys
    }

    pub(crate) const fn village_war_sys(&self) -> &CVillageWarSys {
        &self.village_war_sys
    }

    pub(crate) const fn village_war_sys_mut(&mut self) -> &mut CVillageWarSys {
        &mut self.village_war_sys
    }

    pub(crate) const fn country_war_sys(&self) -> &CountryWarSys {
        &self.country_war_sys
    }

    pub(crate) const fn country_war_sys_mut(&mut self) -> &mut CountryWarSys {
        &mut self.country_war_sys
    }

    pub(crate) const fn four_nation_war_sys(&self) -> &CFourNationWarSys {
        &self.four_nation_war_sys
    }

    pub(crate) const fn four_nation_war_sys_mut(&mut self) -> &mut CFourNationWarSys {
        &mut self.four_nation_war_sys
    }

    pub(crate) const fn emotion(&self) -> &CEmotion {
        &self.emotion
    }

    pub(crate) const fn emotion_mut(&mut self) -> &mut CEmotion {
        &mut self.emotion
    }

    pub(crate) const fn region_setup_mut(&mut self) -> &mut CRegionSetup {
        &mut self.region_setup
    }

    pub(crate) const fn hit_level_setup_mut(&mut self) -> &mut CHitLevelSetup {
        &mut self.hit_level_setup
    }

    pub(crate) const fn prison_conf_mut(&mut self) -> &mut PrisonConf {
        &mut self.prison_conf
    }

    pub(crate) const fn precious_box_conf_mut(&mut self) -> &mut PreciousBoxConf {
        &mut self.precious_box_conf
    }

    pub(crate) const fn fairy_exp_conf_mut(&mut self) -> &mut CFairyExpConf {
        &mut self.fairy_exp_conf
    }

    pub(crate) const fn battle_fairy_exp_config_mut(&mut self) -> &mut CBattleFairyExpConfig {
        &mut self.battle_fairy_exp_config
    }

    pub(crate) const fn battle_fairy_property_mut(&mut self) -> &mut CBattleFairyProperty {
        &mut self.battle_fairy_property
    }

    pub(crate) const fn equipment_compose_list(&self) -> &EquipmentComposeList {
        &self.equipment_compose_list
    }

    pub(crate) const fn equipment_compose_list_mut(&mut self) -> &mut EquipmentComposeList {
        &mut self.equipment_compose_list
    }

    fn send_equipment_session_notification(&self, player_id: i32, string_id: &str) -> i32 {
        let text = self.get_string_by_id(string_id.as_bytes());
        colored_player_notice_message(0xffff_ffff, 0, text)
            .send_to_player(self.net_server(), player_id)
    }

    pub(crate) fn open_equipment_session<Context: EquipmentSessionOpenContext>(
        &mut self,
        player_id: i32,
        kind: EquipmentSessionPlugKind,
        context: &mut Context,
    ) -> EquipmentSessionOpenReport {
        let mut report = EquipmentSessionOpenReport {
            kind,
            player_id,
            outcome: EquipmentSessionOpenOutcome::MissingOrDeadPlayer,
            session_id: None,
            plug_id: None,
            notification_delivery: None,
            player_transition: None,
            listener_attach: None,
            open_delivery: None,
            collected_plug_ids: Vec::new(),
        };
        if kind == EquipmentSessionPlugKind::DaKong && !self.da_kong_xiang_qian.key() {
            report.outcome = EquipmentSessionOpenOutcome::FeatureDisabled;
            return report;
        }
        let Some(player) = self.find_player(player_id) else {
            return report;
        };
        if player.is_dead() {
            return report;
        }
        if player.current_progress() != PlayerProgress::None {
            report.outcome = EquipmentSessionOpenOutcome::Busy;
            let string_id = match kind {
                EquipmentSessionPlugKind::Upgrade => "GS0177",
                EquipmentSessionPlugKind::DaKong => "GS1057",
                EquipmentSessionPlugKind::Compose => "GS1061",
            };
            report.notification_delivery =
                Some(self.send_equipment_session_notification(player_id, string_id));
            return report;
        }
        if kind != EquipmentSessionPlugKind::Upgrade
            && context.equipment_session_has_team_state(player_id)
        {
            report.outcome = EquipmentSessionOpenOutcome::TeamStateBlocked;
            let string_id = match kind {
                EquipmentSessionPlugKind::DaKong => "GS1058",
                EquipmentSessionPlugKind::Compose => "GS1062",
                EquipmentSessionPlugKind::Upgrade => unreachable!(),
            };
            report.notification_delivery =
                Some(self.send_equipment_session_notification(player_id, string_id));
            return report;
        }
        let Some((session_id, plug_id)) = self
            .session_factory
            .create_equipment_session(kind, player_id)
        else {
            report.outcome = EquipmentSessionOpenOutcome::FactoryRejected;
            return report;
        };
        report.session_id = Some(session_id);
        report.plug_id = Some(plug_id);
        let (progress, lock_movement, message_type) = match kind {
            EquipmentSessionPlugKind::Upgrade => (PlayerProgress::Upgrade, false, 0x0b_f912),
            EquipmentSessionPlugKind::DaKong => (PlayerProgress::DaKong, false, 0x0b_f929),
            EquipmentSessionPlugKind::Compose => (PlayerProgress::Compose, true, 0x0b_f92a),
        };
        if let Some(player) = self.find_player_mut(player_id) {
            report.listener_attach = Some(player.attach_equipment_session_listener(plug_id));
            report.player_transition =
                Some(player.begin_equipment_session(progress, lock_movement));
        }
        let mut message = CMessage::new(message_type);
        message.add_long(session_id);
        message.add_long(plug_id);
        if kind == EquipmentSessionPlugKind::Upgrade {
            message.add_long(player_id);
        }
        let delivery = message.send_to_player(self.net_server(), player_id);
        report.open_delivery = Some(delivery);
        if delivery == 0 {
            report.outcome = EquipmentSessionOpenOutcome::SendFailed;
            report.collected_plug_ids = self.session_factory.garbage_collect_session(session_id);
            if let Some(player) = self.find_player_mut(player_id) {
                let _listener_detach = player.detach_equipment_session_listener(plug_id);
                let _ = player.release_goods_session_state();
            }
            return report;
        }
        report.outcome = EquipmentSessionOpenOutcome::Opened;
        report
    }

    pub(crate) fn upgrade_equipment<Context: EquipmentUpgradeContext>(
        &mut self,
        player_id: i32,
        session_id: i32,
        requested_plug_id: i32,
        context: &mut Context,
    ) -> EquipmentUpgradeReport {
        let mut report = EquipmentUpgradeReport {
            session_id,
            requested_plug_id,
            actual_plug_id: None,
            outcome: EquipmentUpgradeOutcome::MissingSessionOrPlug,
            price: 0,
            probability: 0,
            roll: None,
            previous_money: None,
            current_money: None,
            previous_level: None,
            resulting_level: None,
            notifications: Vec::new(),
            money_deliveries: Vec::new(),
            consumptions: Vec::new(),
            client_update: None,
            client_update_delivery: None,
            audit: None,
            lost_audit: None,
            world_deliveries: Vec::new(),
        };
        if self.session_factory.query_session(session_id).is_none() {
            return report;
        }
        report.actual_plug_id = self
            .session_factory
            .query_session_plug_by_owner(session_id, 400, player_id)
            .map(|plug| plug.id());
        let Some(actual_plug_id) = report.actual_plug_id else {
            return report;
        };
        if actual_plug_id != requested_plug_id {
            report.outcome = EquipmentUpgradeOutcome::PlugIdMismatch;
            return report;
        }
        let Some(mut plug) = self
            .session_factory
            .take_equipment_upgrade_plug(actual_plug_id)
        else {
            return report;
        };
        let Some(mut player) = self.players.remove(&player_id) else {
            self.session_factory
                .register_equipment_upgrade_plug(actual_plug_id, plug);
            return report;
        };
        report = self.upgrade_equipment_inner(&mut player, &mut plug, report, context);
        self.players.insert(player_id, player);
        self.session_factory
            .register_equipment_upgrade_plug(actual_plug_id, plug);
        report
    }

    fn upgrade_equipment_inner<Context: EquipmentUpgradeContext>(
        &mut self,
        player: &mut CPlayer,
        plug: &mut CEquipmentUpgrade,
        mut report: EquipmentUpgradeReport,
        context: &mut Context,
    ) -> EquipmentUpgradeReport {
        let player_id = player.player_id();
        let Some(region_id) = player.server_region_id() else {
            report.outcome = EquipmentUpgradeOutcome::MissingPlayerOrRegion;
            return report;
        };
        let tile_x = player.shape().get_tile_x().unwrap_or(0);
        let tile_y = player.shape().get_tile_y().unwrap_or(0);
        report.price = plug.upgrade_price(|goods_id| {
            player.get_goods_by_id(goods_id).map_or(0, |goods| {
                CEquipmentUpgrade::price_property(goods, &self.goods_factory)
            })
        });
        if player.money() < report.price {
            report.outcome = EquipmentUpgradeOutcome::InsufficientMoneyForValidation;
            report
                .notifications
                .push(self.send_equipment_upgrade_notification(
                    player_id,
                    "GS0250",
                    Some(report.price),
                ));
            return report;
        }
        let Some(equipment_id) = plug.goods_id(UpgradeEquipmentCell::Equipment) else {
            report.outcome = EquipmentUpgradeOutcome::MissingOrInvalidEquipment;
            report
                .notifications
                .push(self.send_equipment_upgrade_notification(player_id, "GS0249", None));
            return report;
        };
        let Some(equipment) = player.get_goods_by_id(equipment_id) else {
            report.outcome = EquipmentUpgradeOutcome::MissingOrInvalidEquipment;
            return report;
        };
        if !equipment.can_upgraded(&self.goods_factory) {
            report.outcome = EquipmentUpgradeOutcome::MissingOrInvalidEquipment;
            report
                .notifications
                .push(self.send_equipment_upgrade_notification(player_id, "GS0249", None));
            return report;
        }
        let current_level = equipment.addon_property_value(
            &self.goods_factory,
            crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_LEVEL,
            1,
        );
        report.previous_level = Some(current_level as u32);
        let Some(base_gem_id) = plug.goods_id(UpgradeEquipmentCell::BaseGem) else {
            report.outcome = EquipmentUpgradeOutcome::MissingBaseGem;
            report
                .notifications
                .push(self.send_equipment_upgrade_notification(player_id, "GS0248", None));
            return report;
        };
        let Some(base_gem) = player.get_goods_by_id(base_gem_id) else {
            report.outcome = EquipmentUpgradeOutcome::MissingBaseGem;
            return report;
        };
        let minimum_level = base_gem.addon_property_value(
            &self.goods_factory,
            crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_GEM_LEVEL,
            1,
        );
        let maximum_level = minimum_level.max(base_gem.addon_property_value(
            &self.goods_factory,
            crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_GEM_LEVEL,
            2,
        ));
        if current_level < minimum_level || maximum_level < current_level {
            report.outcome = EquipmentUpgradeOutcome::EquipmentLevelOutsideGemRange;
            report
                .notifications
                .push(self.send_equipment_upgrade_notification(player_id, "GS0247", None));
            return report;
        }
        if 98 < current_level as u32 {
            report.outcome = EquipmentUpgradeOutcome::MaximumLevel;
            report
                .notifications
                .push(self.send_equipment_upgrade_notification(player_id, "GS0257", None));
            return report;
        }
        report.probability = plug.probability(|goods_id, property, value_id| {
            player.get_goods_by_id(goods_id).map_or(0, |goods| {
                goods.addon_property_value(&self.goods_factory, property, value_id)
            })
        });
        if player.money() < report.price {
            report.outcome = EquipmentUpgradeOutcome::InsufficientMoneyAtExecution;
            report
                .notifications
                .push(self.send_equipment_upgrade_notification(player_id, "GS0256", None));
            return report;
        }

        let money = player.decrease_money(report.price, &self.goods_factory);
        report.previous_money = Some(money.previous);
        report.current_money = Some(money.current);
        report.money_deliveries = self.send_player_money_decrease(player_id, &money.outcome);
        let roll = game_legacy_random(&mut self.random_state, 100) as u32 + 1;
        report.roll = Some(roll);

        let gem_cells = [
            UpgradeEquipmentCell::BaseGem,
            UpgradeEquipmentCell::GemOne,
            UpgradeEquipmentCell::GemTwo,
            UpgradeEquipmentCell::GemThree,
        ];
        let gems = gem_cells.map(|cell| {
            plug.goods_id(cell)
                .and_then(|id| player.get_goods_by_id(id))
                .map(EquipmentUpgradeGoodsSnapshot::capture)
        });

        let success = roll <= report.probability;
        if success {
            let succeed = plug.succeed_result(
                |goods_id, property, value_id| {
                    player.get_goods_by_id(goods_id).map_or(0, |goods| {
                        goods.addon_property_value(&self.goods_factory, property, value_id)
                    })
                },
                |upper_bound| game_legacy_random(&mut self.random_state, upper_bound),
            );
            let target_level = (current_level as u32).wrapping_add(succeed).min(99) as i32;
            if let Some(equipment) = player.get_goods_by_id_mut(equipment_id) {
                let (factory, random_state) = (&self.goods_factory, &mut self.random_state);
                let _ = factory.upgrade_equipment(equipment, target_level, |upper_bound| {
                    game_legacy_random(random_state, upper_bound)
                });
            }
            report.resulting_level = Some(target_level as u32);
            report.outcome = EquipmentUpgradeOutcome::Succeeded;
            report
                .notifications
                .push(self.send_equipment_upgrade_notification(player_id, "GS0251", None));
            if self.log_system.goods_upgrade_success_enabled() {
                let audit = EquipmentUpgradeAuditLog {
                    reason: EQUIPMENT_UPGRADE_SUCCESS_LOG_REASON,
                    player_id,
                    equipment: player
                        .get_goods_by_id(equipment_id)
                        .map(EquipmentUpgradeGoodsSnapshot::capture)
                        .expect("успешный upgrade сохраняет equipment"),
                    gems,
                    region_id,
                    tile_x,
                    tile_y,
                };
                report
                    .world_deliveries
                    .extend(self.send_equipment_upgrade_audit(&audit));
                report.audit = Some(audit);
            }
            self.publish_equipment_upgrade_update(player, equipment_id, &mut report, context);
        } else {
            let equipment_snapshot = player
                .get_goods_by_id(equipment_id)
                .map(EquipmentUpgradeGoodsSnapshot::capture)
                .expect("equipment проверен до failure audit");
            if self.log_system.goods_upgrade_failure_enabled() {
                let audit = EquipmentUpgradeAuditLog {
                    reason: EQUIPMENT_UPGRADE_FAILURE_LOG_REASON,
                    player_id,
                    equipment: equipment_snapshot.clone(),
                    gems,
                    region_id,
                    tile_x,
                    tile_y,
                };
                report
                    .world_deliveries
                    .extend(self.send_equipment_upgrade_audit(&audit));
                report.audit = Some(audit);
            }
            match plug.failed_result(|goods_id, property, value_id| {
                player.get_goods_by_id(goods_id).map_or(0, |goods| {
                    goods.addon_property_value(&self.goods_factory, property, value_id)
                })
            }) {
                1 => {
                    report.outcome = EquipmentUpgradeOutcome::FailedUnchanged;
                    report.resulting_level = Some(current_level as u32);
                    report
                        .notifications
                        .push(self.send_equipment_upgrade_notification(player_id, "GS0252", None));
                }
                2 => {
                    let target_level = current_level.saturating_sub(1);
                    if let Some(equipment) = player.get_goods_by_id_mut(equipment_id) {
                        let (factory, random_state) = (&self.goods_factory, &mut self.random_state);
                        let _ = factory.upgrade_equipment(equipment, target_level, |upper_bound| {
                            game_legacy_random(random_state, upper_bound)
                        });
                    }
                    report.outcome = EquipmentUpgradeOutcome::FailedLevelLost;
                    report.resulting_level = Some(target_level as u32);
                    report
                        .notifications
                        .push(self.send_equipment_upgrade_notification(player_id, "GS0253", None));
                    self.publish_equipment_upgrade_update(
                        player,
                        equipment_id,
                        &mut report,
                        context,
                    );
                }
                3 => {
                    if let Some(equipment) = player.get_goods_by_id_mut(equipment_id) {
                        let (factory, random_state) = (&self.goods_factory, &mut self.random_state);
                        let _ = factory.upgrade_equipment(equipment, 0, |upper_bound| {
                            game_legacy_random(random_state, upper_bound)
                        });
                    }
                    report.outcome = EquipmentUpgradeOutcome::FailedReset;
                    report.resulting_level = Some(0);
                    report
                        .notifications
                        .push(self.send_equipment_upgrade_notification(player_id, "GS0254", None));
                    self.publish_equipment_upgrade_update(
                        player,
                        equipment_id,
                        &mut report,
                        context,
                    );
                }
                _ => {
                    report.outcome = EquipmentUpgradeOutcome::FailedEquipmentLost;
                    report
                        .notifications
                        .push(self.send_equipment_upgrade_notification(player_id, "GS0255", None));
                    if self.log_system.goods_lost_by_upgrade_enabled() {
                        let lost = EquipmentUpgradeLostAuditLog {
                            reason: EQUIPMENT_UPGRADE_LOST_LOG_REASON,
                            player_id,
                            pk_count: player.pk_count(),
                            money: player.money(),
                            depot_money: player.depot_money(),
                            equipment: equipment_snapshot,
                            amount: 1,
                            region_id,
                            tile_x,
                            tile_y,
                            client_ip: player.client_ip(),
                        };
                        report
                            .world_deliveries
                            .extend(self.send_equipment_upgrade_lost_audit(&lost));
                        report.lost_audit = Some(lost);
                    }
                    self.consume_equipment_upgrade_cell(
                        player,
                        plug,
                        UpgradeEquipmentCell::Equipment,
                        &mut report,
                        context,
                    );
                }
            }
        }

        for cell in gem_cells {
            self.consume_equipment_upgrade_cell(player, plug, cell, &mut report, context);
        }
        report
    }

    fn send_equipment_upgrade_notification(
        &self,
        player_id: i32,
        string_id: &str,
        format_value: Option<u32>,
    ) -> i32 {
        let template = self.get_string_by_id(string_id.as_bytes());
        let text = format_value.map_or_else(
            || legacy_c_string_prefix(template).to_vec(),
            |value| format_single_legacy_u32(template, value, 255),
        );
        colored_player_notice_message(0xffff_ffff, 0, &text)
            .send_to_player(self.net_server(), player_id)
    }

    fn send_equipment_upgrade_audit(&self, audit: &EquipmentUpgradeAuditLog) -> Vec<i32> {
        let mut message = CMessage::new(0x0006_0203);
        message.add_byte(audit.reason);
        message.add_long(audit.player_id);
        add_equipment_upgrade_log_goods(&mut message, Some(&audit.equipment));
        for gem in &audit.gems {
            add_equipment_upgrade_log_goods(&mut message, gem.as_ref());
        }
        message.add_long(audit.region_id);
        message.add_long(audit.tile_x);
        message.add_long(audit.tile_y);
        message.send(self, false).into_iter().collect()
    }

    fn send_equipment_upgrade_lost_audit(&self, audit: &EquipmentUpgradeLostAuditLog) -> Vec<i32> {
        let mut message = CMessage::new(0x0006_0202);
        message.add_byte(audit.reason);
        message.add_long(audit.player_id);
        message.base_mut().add_short(audit.pk_count as i16);
        message.add_ulong(audit.money);
        message.add_ulong(audit.depot_money);
        message.base_mut().add_guid(audit.equipment.identity.ex_id);
        // Primary GameServer сохраняет этот legacy wire mismatch: equipment
        // price занимает World-поле amount, literal amount — поле price.
        message.add_ulong(audit.equipment.price);
        add_legacy_c_string(message.base_mut(), &audit.equipment.name);
        message.add_ulong(audit.amount);
        message.add_long(audit.region_id);
        message.add_long(audit.tile_x);
        message.add_long(audit.tile_y);
        message.add_ulong(audit.client_ip);
        message.send(self, false).into_iter().collect()
    }

    fn send_equipment_upgrade_consumption(
        &self,
        previous: &crate::gameserver::appserver::container::ccontainer::PreviousContainer,
        consumption: &CiQingPacketConsumption,
    ) -> Vec<i32> {
        if consumption.remaining_amount == 0 {
            return vec![self.send_container_object_delete(
                consumption.player_id,
                previous,
                consumption.goods,
                consumption.previous_amount,
            )];
        }
        let mut message = CS2CContainerObjectAmountChange::default();
        message.set_source_container(
            previous.container_type,
            previous.container_id,
            previous.goods_position,
        );
        message.set_source_container_extend_id(previous.container_extend_id);
        message.set_object(consumption.goods.object_type, consumption.goods.ex_id);
        message.set_object_amount(consumption.remaining_amount);
        vec![message.send_to_player(self, consumption.player_id)]
    }

    fn send_container_object_delete(
        &self,
        player_id: i32,
        previous: &crate::gameserver::appserver::container::ccontainer::PreviousContainer,
        goods: ShapeIdentity,
        amount: u32,
    ) -> i32 {
        let mut message = CS2CContainerObjectMove::default();
        message.set_operation(ContainerObjectMoveOperation::DeleteObject);
        message.set_source_container(
            previous.container_type,
            previous.container_id,
            previous.goods_position,
        );
        message.set_source_container_extend_id(previous.container_extend_id);
        message.set_source_object(goods.object_type, goods.ex_id, amount);
        message.send_to_player(self, player_id)
    }

    fn publish_equipment_upgrade_update<Context: EquipmentUpgradeContext>(
        &self,
        player: &CPlayer,
        equipment_id: CGuid,
        report: &mut EquipmentUpgradeReport,
        context: &mut Context,
    ) {
        let Some(goods) = player.get_goods_by_id(equipment_id) else {
            return;
        };
        let update = EquipmentUpgradeClientUpdate {
            player_id: player.player_id(),
            goods: goods.identity(),
            old_client_payload: context.encode_goods_for_old_client(goods),
        };
        let mut message = CMessage::new(0x0b_f918);
        message.add_long(update.player_id);
        message.base_mut().add_guid(update.goods.ex_id);
        message.add_ulong(update.old_client_payload.len() as u32);
        message.base_mut().add(&update.old_client_payload);
        report.client_update_delivery =
            Some(message.send_to_player(self.net_server(), update.player_id));
        report.client_update = Some(update);
    }

    fn consume_equipment_upgrade_cell<Context: EquipmentUpgradeContext>(
        &self,
        player: &mut CPlayer,
        plug: &mut CEquipmentUpgrade,
        cell: UpgradeEquipmentCell,
        report: &mut EquipmentUpgradeReport,
        context: &mut Context,
    ) {
        let Some(goods_id) = plug.goods_id(cell) else {
            return;
        };
        let previous = plug
            .upgrade_container()
            .base()
            .base()
            .original_container_information(goods_id)
            .unwrap_or_default();
        let identity = player
            .get_goods_by_id(goods_id)
            .map(CGoods::identity)
            .unwrap_or(ShapeIdentity {
                object_type: 700,
                id: 0,
                ex_id: goods_id,
            });
        let previous_amount = player.get_goods_by_id(goods_id).map_or(0, CGoods::amount);
        let mut deliveries = Vec::new();
        let removal = if player.packet().base().find(goods_id).is_some() {
            match player.remove_packet_goods_by_id(goods_id, 1) {
                Some(consumption) => {
                    deliveries = self.send_equipment_upgrade_consumption(&previous, &consumption);
                    EquipmentUpgradeConsumptionRemoval::Packet(consumption)
                }
                None => EquipmentUpgradeConsumptionRemoval::Missing,
            }
        } else if let Some(goods) = player.equipment().find(goods_id) {
            let facts = context.equipment_upgrade_remove_facts(
                player,
                goods,
                self.globe_setup.pack_add_enabled(),
            );
            let mut recompute =
                |player: &CPlayer| context.recompute_equipment_upgrade_player_properties(player);
            let mut removal = player.remove_equipment_goods(
                goods_id,
                &self.goods_factory,
                &self.skill_factory,
                facts,
                &mut recompute,
            );
            drop(recompute);
            self.publish_player_equipment_remove_report(&mut removal, context);
            if matches!(removal.outcome, EquipmentRemoveOutcome::Removed(_)) {
                deliveries.push(self.send_container_object_delete(
                    player.player_id(),
                    &previous,
                    identity,
                    previous_amount,
                ));
            }
            EquipmentUpgradeConsumptionRemoval::Equipment(removal)
        } else {
            EquipmentUpgradeConsumptionRemoval::Missing
        };
        let source_removed = match &removal {
            EquipmentUpgradeConsumptionRemoval::Packet(_) => true,
            EquipmentUpgradeConsumptionRemoval::Equipment(removal) => {
                matches!(removal.outcome, EquipmentRemoveOutcome::Removed(_))
            }
            EquipmentUpgradeConsumptionRemoval::Missing => false,
        };
        if source_removed {
            let _ = plug.upgrade_container_mut().on_source_removed(goods_id, 1);
        }
        report.consumptions.push(EquipmentUpgradeConsumption {
            cell,
            goods: identity,
            previous,
            previous_amount,
            removal,
            deliveries,
        });
    }

    pub(crate) fn close_equipment_upgrade(
        &mut self,
        player_id: i32,
        session_id: i32,
    ) -> EquipmentUpgradeCloseReport {
        let mut report = EquipmentUpgradeCloseReport {
            session_id,
            actual_plug_id: None,
            outcome: EquipmentUpgradeCloseOutcome::MissingSessionOrPlug,
            session_end: None,
            listener_detach: None,
            previous_progress: None,
            cleared_shadows: 0,
            close_delivery: None,
            collected_plug_ids: Vec::new(),
        };
        if self.session_factory.query_session(session_id).is_none() {
            return report;
        }
        report.actual_plug_id = self
            .session_factory
            .query_session_plug_by_owner(session_id, 400, player_id)
            .map(|plug| plug.id());
        let Some(plug_id) = report.actual_plug_id else {
            return report;
        };
        let Some(mut plug) = self.session_factory.take_equipment_upgrade_plug(plug_id) else {
            return report;
        };
        report.session_end = self.session_factory.end_session(session_id);
        if let Some(player) = self.find_player_mut(player_id) {
            report.listener_detach = Some(player.detach_equipment_session_listener(plug_id));
            let previous = player.current_progress();
            player.set_current_progress_snapshot(PlayerProgress::None);
            report.previous_progress = Some(previous);
        }
        report.cleared_shadows = plug.close();
        let message = CMessage::new(0x0b_f913);
        report.close_delivery = Some(message.send_to_player(self.net_server(), player_id));
        report.collected_plug_ids = self.session_factory.garbage_collect_session(session_id);
        report.outcome = EquipmentUpgradeCloseOutcome::Closed;
        report
    }

    pub(crate) fn compose_equipment<Context: EquipmentComposeContext + ScriptFunctionRuntime>(
        &mut self,
        player_id: i32,
        session_id: i32,
        requested_plug_id: i32,
        context: &mut Context,
    ) -> EquipmentComposeReport {
        let mut report = EquipmentComposeReport {
            session_id,
            requested_plug_id,
            actual_plug_id: None,
            outcome: EquipmentComposeOutcome::MissingSessionOrPlug,
            result_index: 0,
            required_level: 0,
            notifications: Vec::new(),
            source_consumptions: Vec::new(),
            stone_consumptions: Vec::new(),
            stone_deliveries: Vec::new(),
            packet_additions: Vec::new(),
            packet_addition_deliveries: Vec::new(),
            rejected_result: None,
            result_shadow: None,
            script_dispatched: false,
            audit_logs: Vec::new(),
            world_deliveries: Vec::new(),
        };
        if self.session_factory.query_session(session_id).is_none() {
            return report;
        }
        report.actual_plug_id = self
            .session_factory
            .query_session_plug_by_owner(session_id, 400, player_id)
            .map(|plug| plug.id());
        let Some(actual_plug_id) = report.actual_plug_id else {
            return report;
        };
        if actual_plug_id != requested_plug_id {
            report.outcome = EquipmentComposeOutcome::PlugIdMismatch;
            return report;
        }
        let Some(mut plug) = self
            .session_factory
            .take_equipment_compose_plug(actual_plug_id)
        else {
            return report;
        };
        report = self.compose_equipment_inner(player_id, &mut plug, report, context);
        self.session_factory
            .register_equipment_compose_plug(actual_plug_id, plug);
        report
    }

    fn compose_equipment_inner<Context: EquipmentComposeContext + ScriptFunctionRuntime>(
        &mut self,
        player_id: i32,
        plug: &mut CEquipmentCompose,
        mut report: EquipmentComposeReport,
        context: &mut Context,
    ) -> EquipmentComposeReport {
        let Some(player) = self.find_player(player_id) else {
            return report;
        };
        if player.server_region_id().is_none() {
            report.outcome = EquipmentComposeOutcome::MissingRegion;
            return report;
        }
        let Some(base_id) = plug
            .compose_container()
            .goods_id(ComposeEquipmentCell::BaseEquipment)
        else {
            report.outcome = EquipmentComposeOutcome::MissingBase;
            report
                .notifications
                .push(self.send_equipment_compose_notification(player_id, "GS1156", &[]));
            return report;
        };
        let Some(sub_id) = plug
            .compose_container()
            .goods_id(ComposeEquipmentCell::SubEquipment)
        else {
            report.outcome = EquipmentComposeOutcome::MissingSub;
            report
                .notifications
                .push(self.send_equipment_compose_notification(player_id, "GS1157", &[]));
            return report;
        };
        let Some(base_source) = player
            .get_goods_by_id(base_id)
            .map(|goods| EquipmentComposeSourceSnapshot::capture(goods, &self.goods_factory))
        else {
            report.outcome = EquipmentComposeOutcome::MissingBase;
            return report;
        };
        let Some(sub_source) = player
            .get_goods_by_id(sub_id)
            .map(|goods| EquipmentComposeSourceSnapshot::capture(goods, &self.goods_factory))
        else {
            report.outcome = EquipmentComposeOutcome::MissingSub;
            return report;
        };
        if player.check_item_in_packet(COMPOSE_STONE_GOODS_INDEX) == 0 {
            report.outcome = EquipmentComposeOutcome::MissingStone;
            report
                .notifications
                .push(self.send_equipment_compose_notification(player_id, "GS1158", &[]));
            return report;
        }
        if base_source.base_index == 0 || base_source.base_index != sub_source.base_index {
            report.outcome = EquipmentComposeOutcome::DifferentEquipment;
            report
                .notifications
                .push(self.send_equipment_compose_notification(player_id, "GS1159", &[]));
            return report;
        }
        let first = self
            .equipment_compose_list
            .get_first_compose(base_source.base_index);
        let second = self
            .equipment_compose_list
            .get_second_compose(base_source.base_index);
        let (result_index, step, required_level) = if second != 0 {
            (second, 2, 20)
        } else {
            (first, 1, 15)
        };
        report.result_index = result_index;
        report.required_level = required_level;
        if result_index == 0 {
            report.outcome = EquipmentComposeOutcome::MissingRecipe;
            report
                .notifications
                .push(self.send_equipment_compose_notification(player_id, "GS1160", &[]));
            return report;
        }
        if base_source.weapon_level < required_level || sub_source.weapon_level < required_level {
            report.outcome = EquipmentComposeOutcome::InsufficientLevel {
                step,
                required: required_level,
            };
            report
                .notifications
                .push(self.send_equipment_compose_notification(
                    player_id,
                    "GS1161",
                    &[step, required_level],
                ));
            return report;
        }
        if base_source.anima_bind != 1 || sub_source.anima_bind != 1 {
            report.outcome = EquipmentComposeOutcome::NotBound;
            report
                .notifications
                .push(self.send_equipment_compose_notification(player_id, "GS1162", &[]));
            return report;
        }
        if base_source.quality != sub_source.quality {
            report.outcome = EquipmentComposeOutcome::DifferentQuality;
            report
                .notifications
                .push(self.send_equipment_compose_notification(player_id, "GS1163", &[]));
            return report;
        }

        let mut created = {
            let (random_state, goods_factory, fairy_exp_conf, battle_fairy_exp_config) = (
                &mut self.random_state,
                &self.goods_factory,
                &self.fairy_exp_conf,
                &self.battle_fairy_exp_config,
            );
            goods_factory.create_goods_batch(
                result_index,
                1,
                |upper_bound| game_legacy_random(random_state, upper_bound),
                || CGuid::create().unwrap_or(CGuid::GUID_INVALID),
                |equip_level, level| fairy_exp_conf.dw_exp_up(equip_level, level),
                |equip_level, level| battle_fairy_exp_config.dw_exp_up(equip_level, level),
            )
        };
        let Some(mut result) = created.pop() else {
            report.outcome = EquipmentComposeOutcome::FactoryRejected;
            return report;
        };

        for source in [&base_source, &sub_source] {
            let log = EquipmentComposeAuditLog {
                player_id,
                reason: COMPOSE_CONSUME_REASON,
                goods: source.identity,
                base_index: source.base_index,
                price: source.price,
                name: source.name.clone(),
            };
            self.record_equipment_compose_log(&mut report, &log);
        }
        for (cell, source) in [
            (ComposeEquipmentCell::BaseEquipment, base_source.clone()),
            (ComposeEquipmentCell::SubEquipment, sub_source.clone()),
        ] {
            let previous = plug
                .compose_container()
                .original_container_information(source.identity.ex_id)
                .unwrap_or_default();
            let mut consumption = EquipmentComposeSourceConsumption {
                cell,
                source,
                previous,
                removal: EquipmentComposeSourceRemoval::Missing,
                external_deliveries: Vec::new(),
            };
            let (removal, deliveries) =
                self.consume_equipment_compose_source(player_id, &consumption, context);
            consumption.removal = removal;
            consumption.external_deliveries = deliveries;
            let removed = match &consumption.removal {
                EquipmentComposeSourceRemoval::Packet(_) => true,
                EquipmentComposeSourceRemoval::Equipment(removal) => {
                    matches!(removal.outcome, EquipmentRemoveOutcome::Removed(_))
                }
                EquipmentComposeSourceRemoval::Missing => false,
            };
            if removed {
                plug.compose_container_mut()
                    .remove_shadow(consumption.source.identity.ex_id);
            }
            report.source_consumptions.push(consumption);
        }
        let stone_consumptions = self
            .find_player_mut(player_id)
            .expect("compose owner проверен до stone removal")
            .remove_item_in_packet(COMPOSE_STONE_GOODS_INDEX, 1);
        for consumption in stone_consumptions {
            let deliveries = self.send_player_packet_consumption(&consumption);
            report.stone_consumptions.push(consumption);
            report.stone_deliveries.push(deliveries);
        }

        let packet_position = self.find_player(player_id).and_then(|player| {
            player
                .packet()
                .find_position_for_goods(&result, &self.goods_factory)
        });
        let Some(packet_position) = packet_position else {
            report.rejected_result = Some(result.identity());
            report.outcome = EquipmentComposeOutcome::PacketFullAfterConsumption;
            return report;
        };
        let result_log = EquipmentComposeAuditLog {
            player_id,
            reason: COMPOSE_CREATE_REASON,
            goods: result.identity(),
            base_index: result.base_properties_index(),
            price: result.price(),
            name: result.name().to_vec(),
        };
        self.record_equipment_compose_log(&mut report, &result_log);
        for &(property, value_id, value) in &base_source.transferred_addons {
            let _ = result.set_addon_property_value_first_core(property, value_id, value);
        }
        let (factory, random_state) = (&self.goods_factory, &mut self.random_state);
        let _ = factory.upgrade_equipment(&mut result, required_level, |upper_bound| {
            game_legacy_random(random_state, upper_bound)
        });
        let _ = result.set_addon_property_value_first_core(
            crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_LEVEL,
            1,
            required_level,
        );
        let result_identity = result.identity();
        let (addition, rejected_result) = {
            let (players, goods_factory) = (&mut self.players, &self.goods_factory);
            let player = players
                .get_mut(&player_id)
                .expect("compose owner проверен до packet add");
            let mut incoming = Some(result);
            let outcome = player.packet_mut().add_goods_at(
                packet_position,
                &mut incoming,
                goods_factory,
                true,
            );
            let (old_client_payload, resulting_amount) = match &outcome {
                VolumeGoodsAddOutcome::Added(added) => {
                    let stored = player
                        .packet()
                        .base()
                        .find(added.identity.ex_id)
                        .expect("успешный packet add сохранил compose result");
                    (
                        Some(context.encode_goods_for_old_client(stored)),
                        Some(stored.amount()),
                    )
                }
                VolumeGoodsAddOutcome::Stack(GoodsStackMergeOutcome::Merged { target, .. }) => (
                    None,
                    player
                        .packet()
                        .base()
                        .find(target.ex_id)
                        .map(CGoods::amount),
                ),
                VolumeGoodsAddOutcome::Stack(_) | VolumeGoodsAddOutcome::Rejected(_) => {
                    (None, None)
                }
            };
            (
                CiQingPacketAddition {
                    player_id,
                    source: result_identity,
                    position: Some(packet_position),
                    outcome,
                    old_client_payload,
                    resulting_amount,
                },
                incoming.map(|goods| goods.identity()),
            )
        };
        if let Some(rejected_result) = rejected_result {
            report.rejected_result = Some(rejected_result);
            report.packet_additions.push(addition);
            report.outcome = EquipmentComposeOutcome::PacketAddRejected;
            return report;
        }
        let addition_deliveries = self.send_player_packet_addition(&addition);
        report.packet_additions.push(addition);
        report.packet_addition_deliveries.push(addition_deliveries);

        if let Some(stored) = self
            .find_player(player_id)
            .and_then(|player| player.packet().base().find(result_identity.ex_id))
        {
            let previous = PreviousContainer {
                container_type: 400,
                container_id: player_id,
                container_extend_id: 1,
                goods_position: packet_position,
            };
            report.result_shadow = Some(plug.compose_container_mut().insert_shadow(
                ComposeEquipmentCell::ComposeEquipment,
                stored,
                previous,
            ));
        }
        let region_id = self
            .find_player(player_id)
            .and_then(CPlayer::server_region_id);
        report.script_dispatched = self
            .run_script_file(
                b"scripts/goods/shenbing_gonggao.script",
                ScriptExecutionContext {
                    player_id: Some(player_id),
                    region_id,
                    ..ScriptExecutionContext::default()
                },
                context,
            )
            .is_some();
        report.outcome = EquipmentComposeOutcome::Completed;
        report
    }

    fn send_equipment_compose_notification(
        &self,
        player_id: i32,
        string_id: &str,
        values: &[i32],
    ) -> i32 {
        let template = self.get_string_by_id(string_id.as_bytes());
        let text = if let [first, second] = values {
            format_two_legacy_i32(template, *first, *second, 255)
        } else {
            legacy_c_string_prefix(template).to_vec()
        };
        colored_player_notice_message(0xffff_ffff, 0, &text)
            .send_to_player(self.net_server(), player_id)
    }

    fn record_equipment_compose_log(
        &self,
        report: &mut EquipmentComposeReport,
        log: &EquipmentComposeAuditLog,
    ) {
        if !self.log_system.equipment_compose_enabled() {
            return;
        }
        let Some(player) = self.find_player(log.player_id) else {
            return;
        };
        let mut message = CMessage::new(0x0006_0202);
        message.add_byte(log.reason);
        message.add_long(log.player_id);
        message.base_mut().add_short(player.pk_count() as i16);
        message.add_ulong(player.money());
        message.add_ulong(player.depot_money());
        message.base_mut().add_guid(log.goods.ex_id);
        message.add_ulong(log.price);
        add_legacy_c_string(message.base_mut(), &log.name);
        message.add_ulong(1);
        message.add_long(player.server_region_id().unwrap_or_default());
        message.add_ulong(player.shape().get_tile_x().unwrap_or_default() as u32);
        message.add_ulong(player.shape().get_tile_y().unwrap_or_default() as u32);
        message.add_ulong(player.client_ip());
        report.world_deliveries.extend(message.send(self, false));
        report.audit_logs.push(log.clone());
    }

    fn consume_equipment_compose_source<Context: EquipmentComposeContext>(
        &mut self,
        player_id: i32,
        consumption: &EquipmentComposeSourceConsumption,
        context: &mut Context,
    ) -> (EquipmentComposeSourceRemoval, Vec<i32>) {
        let Some(mut player) = self.players.remove(&player_id) else {
            return (EquipmentComposeSourceRemoval::Missing, Vec::new());
        };
        let goods_id = consumption.source.identity.ex_id;
        let mut deliveries = Vec::new();
        let removal = if player.packet().base().find(goods_id).is_some() {
            match player.remove_packet_goods_by_id(goods_id, consumption.source.amount) {
                Some(packet) => {
                    deliveries = self.send_player_packet_consumption(&packet);
                    EquipmentComposeSourceRemoval::Packet(packet)
                }
                None => EquipmentComposeSourceRemoval::Missing,
            }
        } else if let Some(goods) = player.equipment().find(goods_id) {
            let facts = context.equipment_compose_remove_facts(
                &player,
                goods,
                self.globe_setup.pack_add_enabled(),
            );
            let mut recompute =
                |player: &CPlayer| context.recompute_equipment_compose_player_properties(player);
            let mut equipment = player.remove_equipment_goods(
                goods_id,
                &self.goods_factory,
                &self.skill_factory,
                facts,
                &mut recompute,
            );
            drop(recompute);
            self.publish_player_equipment_remove_report(&mut equipment, context);
            if matches!(equipment.outcome, EquipmentRemoveOutcome::Removed(_)) {
                deliveries.push(self.send_container_object_delete(
                    player_id,
                    &consumption.previous,
                    consumption.source.identity,
                    consumption.source.amount,
                ));
            }
            EquipmentComposeSourceRemoval::Equipment(equipment)
        } else {
            EquipmentComposeSourceRemoval::Missing
        };
        self.players.insert(player_id, player);
        (removal, deliveries)
    }

    pub(crate) fn send_player_packet_addition(&self, addition: &CiQingPacketAddition) -> Vec<i32> {
        let Some(position) = addition.position else {
            return Vec::new();
        };
        match &addition.outcome {
            VolumeGoodsAddOutcome::Added(added) => {
                let mut message = CS2CContainerObjectMove::default();
                message.set_operation(ContainerObjectMoveOperation::NewObject);
                message.set_destination_container(PLAYER_TYPE, addition.player_id, position);
                message.set_destination_container_extend_id(1);
                message.set_destination_object(added.identity.object_type, added.identity.ex_id);
                message.set_object_stream(addition.old_client_payload.clone().unwrap_or_default());
                vec![message.send_to_player(self, addition.player_id)]
            }
            VolumeGoodsAddOutcome::Stack(GoodsStackMergeOutcome::Merged { target, .. }) => {
                let Some(amount) = addition.resulting_amount else {
                    return Vec::new();
                };
                let mut message = CS2CContainerObjectAmountChange::default();
                message.set_source_container(PLAYER_TYPE, addition.player_id, position);
                message.set_source_container_extend_id(1);
                message.set_object(target.object_type, target.ex_id);
                message.set_object_amount(amount);
                vec![message.send_to_player(self, addition.player_id)]
            }
            _ => Vec::new(),
        }
    }

    pub(crate) fn close_equipment_da_kong<Runtime: GameGoodsMessageRuntime>(
        &mut self,
        player_id: i32,
        session_id: i32,
        requested_plug_id: i32,
        runtime: &mut Runtime,
    ) -> EquipmentDaKongCloseReport {
        let mut report = EquipmentDaKongCloseReport {
            session_id,
            requested_plug_id,
            actual_plug_id: None,
            last_equipment_id: None,
            outcome: EquipmentDaKongCloseOutcome::MissingSessionOrPlug,
            session_end: None,
            previous_progress: None,
            plug_exit: None,
            client_update: None,
            client_update_delivery: None,
        };
        if self.session_factory.query_session(session_id).is_none() {
            return report;
        }
        report.actual_plug_id = self
            .session_factory
            .query_session_plug_by_owner(session_id, 400, player_id)
            .map(|plug| plug.id());
        let Some(actual_plug_id) = report.actual_plug_id else {
            return report;
        };
        if actual_plug_id != requested_plug_id {
            report.outcome = EquipmentDaKongCloseOutcome::PlugIdMismatch;
            return report;
        }
        let Some(last_equipment_id) = self
            .session_factory
            .query_equipment_da_kong_plug(actual_plug_id)
            .map(CEquipmentDaKong::last_equipment_id)
        else {
            return report;
        };
        report.last_equipment_id = Some(last_equipment_id);

        report.session_end = self.session_factory.end_session(session_id);
        report.previous_progress = self.find_player_mut(player_id).map(|player| {
            let previous = player.current_progress();
            player.set_current_progress_snapshot(PlayerProgress::None);
            previous
        });
        report.plug_exit = Some(self.session_factory.exit_plug(session_id, actual_plug_id));

        let Some(goods) = self
            .find_player(player_id)
            .and_then(|player| player.get_goods_by_id(last_equipment_id))
        else {
            report.outcome = EquipmentDaKongCloseOutcome::ClosedWithoutEquipment;
            return report;
        };
        let update = EquipmentDaKongClientUpdate {
            player_id,
            goods: goods.identity(),
            old_client_payload: runtime.encode_goods_for_old_client(goods),
        };
        let mut message = CMessage::new(0x0b_f918);
        message.add_long(update.player_id);
        message.base_mut().add_guid(update.goods.ex_id);
        message.add_ulong(update.old_client_payload.len() as u32);
        message.base_mut().add(&update.old_client_payload);
        report.client_update_delivery =
            Some(message.send_to_player(self.net_server(), update.player_id));
        report.client_update = Some(update);
        report.outcome = EquipmentDaKongCloseOutcome::ClosedAndUpdated;
        report
    }

    pub(crate) fn process_equipment_da_kong<Context: ScriptFunctionRuntime>(
        &mut self,
        player_id: i32,
        session_id: i32,
        requested_plug_id: i32,
        operation: EquipmentDaKongOperation,
        context: &mut Context,
    ) -> EquipmentDaKongReport {
        let mut report = EquipmentDaKongReport {
            session_id,
            requested_plug_id,
            actual_plug_id: None,
            operation,
            outcome: EquipmentDaKongOutcome::MissingSessionOrPlug,
            return_value: 0,
            notifications: Vec::new(),
            packet_consumptions: Vec::new(),
            packet_consumption_deliveries: Vec::new(),
            gem_consumptions: Vec::new(),
            logs: Vec::new(),
            world_deliveries: Vec::new(),
            client_updates: Vec::new(),
            client_update_deliveries: Vec::new(),
            scripts: Vec::new(),
        };
        if self.session_factory.query_session(session_id).is_none() {
            return report;
        }
        report.actual_plug_id = self
            .session_factory
            .query_session_plug_by_owner(session_id, 400, player_id)
            .map(|plug| plug.id());
        let Some(actual_plug_id) = report.actual_plug_id else {
            return report;
        };
        if actual_plug_id != requested_plug_id {
            report.outcome = EquipmentDaKongOutcome::PlugIdMismatch;
            return report;
        }
        let Some(mut plug) = self
            .session_factory
            .take_equipment_da_kong_plug(actual_plug_id)
        else {
            return report;
        };
        let Some(mut player) = self.players.remove(&player_id) else {
            self.session_factory
                .register_equipment_da_kong_plug(actual_plug_id, plug);
            return report;
        };
        report = self.process_equipment_da_kong_inner(&mut player, &mut plug, report, context);
        self.players.insert(player_id, player);
        self.session_factory
            .register_equipment_da_kong_plug(actual_plug_id, plug);
        report
    }

    pub(crate) fn reflush_equipment_da_kong_external_property<Context: EquipmentDaKongContext>(
        &mut self,
        player_id: i32,
        cost_original_name: &[u8],
        context: &mut Context,
    ) -> EquipmentDaKongExternalRefreshReport {
        let mut report = EquipmentDaKongExternalRefreshReport {
            player_id,
            cost_original_name: cost_original_name.to_vec(),
            equipment_id: None,
            outcome: EquipmentDaKongExternalRefreshOutcome::FeatureDisabled,
            consumption: None,
            consumption_deliveries: Vec::new(),
            log: None,
            log_deliveries: Vec::new(),
            effect: None,
            effect_delivery: None,
            client_update: None,
            client_update_deliveries: Vec::new(),
        };
        if !self.da_kong_xiang_qian.key() {
            return report;
        }
        if cost_original_name.is_empty() {
            report.outcome = EquipmentDaKongExternalRefreshOutcome::EmptyCostName;
            return report;
        }
        let Some(mut player) = self.players.remove(&player_id) else {
            report.outcome = EquipmentDaKongExternalRefreshOutcome::MissingSelection;
            return report;
        };
        report = self.reflush_equipment_da_kong_external_property_inner(
            &mut player,
            cost_original_name,
            report,
            context,
        );
        self.players.insert(player_id, player);
        report
    }

    fn reflush_equipment_da_kong_external_property_inner<Context: EquipmentDaKongContext>(
        &mut self,
        player: &mut CPlayer,
        cost_original_name: &[u8],
        mut report: EquipmentDaKongExternalRefreshReport,
        context: &mut Context,
    ) -> EquipmentDaKongExternalRefreshReport {
        use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
            GAP_DAKONG_1, GAP_DAKONG_EXTERN_1, GAP_DAKONG_EXTERN_2, GAP_DAKONG_EXTERN_3,
        };

        let Some(equipment_id) = player.enhancement_selected_goods_id() else {
            report.outcome = EquipmentDaKongExternalRefreshOutcome::MissingSelection;
            return report;
        };
        report.equipment_id = Some(equipment_id);
        let Some(equipment) = player.get_goods_by_id(equipment_id) else {
            report.outcome = EquipmentDaKongExternalRefreshOutcome::MissingEquipment;
            return report;
        };
        let old_seven = EquipmentDaKongGemSnapshot::from_catalog(
            equipment.addon_property_value(&self.goods_factory, GAP_DAKONG_1 + 6, 2) as u32,
            &self.goods_factory,
        );
        let equipment = player
            .get_goods_by_id_mut(equipment_id)
            .expect("external refresh equipment проверен до снятия бонусов");
        deal_with_da_kong_external_attributes(equipment, &self.goods_factory, old_seven, false);

        let refresh = match cost_original_name {
            b"FZ1052" => Some((0, GAP_DAKONG_EXTERN_1)),
            b"FZ1051" => Some((1, GAP_DAKONG_EXTERN_2)),
            b"FZ1050" => Some((2, GAP_DAKONG_EXTERN_3)),
            _ => None,
        };
        let mut refreshed = false;
        if let Some((group, property)) = refresh {
            let cost_base_index = self
                .goods_factory
                .query_goods_id_by_original_name(Some(cost_original_name));
            let can_refresh = self
                .goods_factory
                .query_goods_base_properties(cost_base_index)
                .is_some()
                && player.check_item_in_packet(cost_base_index) != 0
                && player
                    .get_goods_by_id(equipment_id)
                    .is_some_and(|equipment| {
                        equipment.addon_property_value(&self.goods_factory, property, 1) > 0
                            && equipment.addon_property_value(&self.goods_factory, property, 2) > 0
                    });
            if can_refresh {
                let (property_type, property_value) = {
                    let (setup, random_state) = (&self.da_kong_xiang_qian, &mut self.random_state);
                    setup
                        .make_sure_external_attribute(
                            group,
                            player
                                .get_goods_by_id(equipment_id)
                                .expect("external refresh equipment сохраняется до RNG")
                                .base_properties_index(),
                            |upper_bound| game_legacy_random(random_state, upper_bound),
                        )
                        .unwrap_or_default()
                };
                let equipment = player
                    .get_goods_by_id_mut(equipment_id)
                    .expect("external refresh equipment сохраняется до mutation");
                let _ = equipment.set_addon_property_value_first_core(property, 1, property_type);
                let _ = equipment.set_addon_property_modifier_core(property, 2, property_value);

                if let Some(cost) = self
                    .goods_factory
                    .query_goods_base_properties(cost_base_index)
                {
                    let log = EquipmentDaKongAuditLog {
                        player_id: player.player_id(),
                        reason: 4,
                        cost_base_index,
                        cost_price: cost.price(),
                        cost_name: cost.name().to_vec(),
                        equipment: EquipmentDaKongGoodsSnapshot::capture(
                            player
                                .get_goods_by_id(equipment_id)
                                .expect("external refresh equipment сохраняется до world log"),
                            &self.goods_factory,
                        ),
                    };
                    report
                        .log_deliveries
                        .extend(self.send_equipment_da_kong_log(player, &log));
                    report.log = Some(log);
                }
                if let Some(consumption) = player
                    .remove_item_in_packet(cost_base_index, 1)
                    .into_iter()
                    .next()
                {
                    report.consumption_deliveries =
                        self.send_player_packet_consumption(&consumption);
                    report.consumption = Some(consumption);
                }
                if let (Some(region_id), Ok(tile_x), Ok(tile_y)) = (
                    player.server_region_id(),
                    player.shape().get_tile_x(),
                    player.shape().get_tile_y(),
                ) {
                    let effect = EquipmentDaKongAroundEffect {
                        effect_id: 11,
                        region_id,
                        tile_x,
                        tile_y,
                    };
                    report.effect_delivery =
                        Some(self.send_equipment_da_kong_around_effect(&effect));
                    report.effect = Some(effect);
                }
                refreshed = true;
            }
        }

        let new_seven = player.get_goods_by_id(equipment_id).and_then(|equipment| {
            EquipmentDaKongGemSnapshot::from_catalog(
                equipment.addon_property_value(&self.goods_factory, GAP_DAKONG_1 + 6, 2) as u32,
                &self.goods_factory,
            )
        });
        let equipment = player
            .get_goods_by_id_mut(equipment_id)
            .expect("external refresh equipment сохраняется до возврата бонусов");
        deal_with_da_kong_external_attributes(equipment, &self.goods_factory, new_seven, true);
        let equipment = player
            .get_goods_by_id(equipment_id)
            .expect("external refresh equipment сохраняется до client update");
        let update = EquipmentDaKongClientUpdate {
            player_id: player.player_id(),
            goods: equipment.identity(),
            old_client_payload: context.encode_goods_for_old_client(equipment),
        };
        report.client_update_deliveries = vec![self.send_equipment_da_kong_update(&update)];
        report.client_update = Some(update);
        report.outcome = if refreshed {
            EquipmentDaKongExternalRefreshOutcome::Refreshed
        } else {
            EquipmentDaKongExternalRefreshOutcome::UpdatedWithoutRefresh
        };
        report
    }

    fn process_equipment_da_kong_inner<Context: ScriptFunctionRuntime>(
        &mut self,
        player: &mut CPlayer,
        plug: &mut CEquipmentDaKong,
        mut report: EquipmentDaKongReport,
        context: &mut Context,
    ) -> EquipmentDaKongReport {
        if player.server_region_id().is_none() {
            report.outcome = EquipmentDaKongOutcome::MissingRegion;
            if matches!(
                report.operation,
                EquipmentDaKongOperation::EnchaseGem { .. }
            ) {
                report.return_value = 1;
            }
            return report;
        }
        match report.operation {
            EquipmentDaKongOperation::DaKong { color_index } => {
                self.equipment_da_kong_create_socket(
                    player,
                    plug,
                    color_index,
                    &mut report,
                    context,
                );
            }
            EquipmentDaKongOperation::ChangeRoleColor { socket } => {
                self.equipment_da_kong_change_color(player, plug, socket, &mut report, context);
            }
            EquipmentDaKongOperation::QueryResult => {
                if self.equipment_da_kong_publish_preview(player, plug, &mut report, context) {
                    report.return_value = 1;
                    report.outcome = EquipmentDaKongOutcome::Completed;
                } else {
                    report.outcome = EquipmentDaKongOutcome::PreviewRejected;
                }
            }
            EquipmentDaKongOperation::EnchaseGem { parameter } => {
                self.equipment_da_kong_enchase(player, plug, parameter, &mut report, context);
            }
            EquipmentDaKongOperation::DestroyGem { socket } => {
                self.equipment_da_kong_destroy_gem(player, plug, socket, &mut report, context);
            }
        }
        report
    }

    fn equipment_da_kong_equipment_id(plug: &CEquipmentDaKong) -> Option<CGuid> {
        plug.upgrade_container().goods_id(
            crate::gameserver::appserver::container::cequipmentdakongcontainer::DaKongCell::Equipment,
        )
    }

    fn equipment_da_kong_gems(
        &self,
        player: &CPlayer,
        plug: &CEquipmentDaKong,
    ) -> [Option<EquipmentDaKongGemSnapshot>; 7] {
        use crate::gameserver::appserver::container::cequipmentdakongcontainer::DaKongCell;
        let cells = [
            DaKongCell::GemOne,
            DaKongCell::GemTwo,
            DaKongCell::GemThree,
            DaKongCell::GemFour,
            DaKongCell::GemFive,
            DaKongCell::GemSix,
            DaKongCell::GemSeven,
        ];
        std::array::from_fn(|index| {
            plug.upgrade_container()
                .goods_id(cells[index])
                .and_then(|goods_id| player.get_goods_by_id(goods_id))
                .map(|goods| EquipmentDaKongGemSnapshot::capture(goods, &self.goods_factory))
        })
    }

    fn equipment_da_kong_notify(
        &self,
        report: &mut EquipmentDaKongReport,
        player_id: i32,
        string_id: &'static str,
    ) {
        let text = self.get_string_by_id(string_id.as_bytes());
        report.notifications.push(
            colored_player_notice_message(0xffff_ffff, 0xffff_0000, text)
                .send_to_player(self.net_server(), player_id),
        );
    }

    fn equipment_da_kong_publish_update<Context: EquipmentDaKongContext>(
        &self,
        report: &mut EquipmentDaKongReport,
        context: &mut Context,
        player_id: i32,
        goods: &CGoods,
    ) {
        let update = EquipmentDaKongClientUpdate {
            player_id,
            goods: goods.identity(),
            old_client_payload: context.encode_goods_for_old_client(goods),
        };
        let deliveries = vec![self.send_equipment_da_kong_update(&update)];
        report.client_updates.push(update);
        report.client_update_deliveries.push(deliveries);
    }

    fn equipment_da_kong_consume_packet(
        &self,
        report: &mut EquipmentDaKongReport,
        player: &mut CPlayer,
        base_index: u32,
    ) {
        for consumption in player.remove_item_in_packet(base_index, 1) {
            let deliveries = self.send_player_packet_consumption(&consumption);
            report.packet_consumptions.push(consumption);
            report.packet_consumption_deliveries.push(deliveries);
        }
    }

    fn equipment_da_kong_log(
        &self,
        report: &mut EquipmentDaKongReport,
        player: &CPlayer,
        reason: u8,
        cost_base_index: u32,
        equipment: &CGoods,
    ) {
        self.equipment_da_kong_log_snapshot(
            report,
            player,
            reason,
            cost_base_index,
            EquipmentDaKongGoodsSnapshot::capture(equipment, &self.goods_factory),
        );
    }

    fn equipment_da_kong_log_snapshot(
        &self,
        report: &mut EquipmentDaKongReport,
        player: &CPlayer,
        reason: u8,
        cost_base_index: u32,
        equipment: EquipmentDaKongGoodsSnapshot,
    ) {
        if !self.da_kong_xiang_qian.log_key() {
            return;
        }
        let Some(cost) = self
            .goods_factory
            .query_goods_base_properties(cost_base_index)
        else {
            return;
        };
        let log = EquipmentDaKongAuditLog {
            player_id: player.player_id(),
            reason,
            cost_base_index,
            cost_price: cost.price(),
            cost_name: cost.name().to_vec(),
            equipment,
        };
        report
            .world_deliveries
            .extend(self.send_equipment_da_kong_log(player, &log));
        report.logs.push(log);
    }

    fn send_equipment_da_kong_update(&self, update: &EquipmentDaKongClientUpdate) -> i32 {
        let mut message = CMessage::new(0x0b_f918);
        message.add_long(update.player_id);
        message.base_mut().add_guid(update.goods.ex_id);
        message.add_ulong(update.old_client_payload.len() as u32);
        message.base_mut().add(&update.old_client_payload);
        message.send_to_player(self.net_server(), update.player_id)
    }

    pub(crate) fn send_player_packet_consumption(
        &self,
        consumption: &CiQingPacketConsumption,
    ) -> Vec<i32> {
        if consumption.remaining_amount == 0 {
            let mut message = CS2CContainerObjectMove::default();
            message.set_operation(ContainerObjectMoveOperation::DeleteObject);
            message.set_source_container(PLAYER_TYPE, consumption.player_id, consumption.position);
            message.set_source_container_extend_id(1);
            message.set_source_object(
                consumption.goods.object_type,
                consumption.goods.ex_id,
                consumption.previous_amount,
            );
            return vec![message.send_to_player(self, consumption.player_id)];
        }
        let mut message = CS2CContainerObjectAmountChange::default();
        message.set_source_container(PLAYER_TYPE, consumption.player_id, consumption.position);
        message.set_source_container_extend_id(1);
        message.set_object(consumption.goods.object_type, consumption.goods.ex_id);
        message.set_object_amount(consumption.remaining_amount);
        vec![message.send_to_player(self, consumption.player_id)]
    }

    /// Exact script `DelGoods(name, amount)` расходует packet stacks в
    /// insertion order, затем equipment в column order. Каждая мутация сразу
    /// публикует обычный container wire; equipment также проходит полный
    /// player property/skill/around tail до перехода к следующему кандидату.
    pub(crate) fn delete_script_goods<Runtime: GameContainerMessageRuntime>(
        &mut self,
        player_id: i32,
        base_index: u32,
        requested: u32,
        runtime: &mut Runtime,
    ) -> u32 {
        if base_index == 0 || requested == 0 {
            return 0;
        }
        let Some(mut player) = self.players.remove(&player_id) else {
            return 0;
        };
        let mut removed = 0u32;
        for consumption in player.remove_item_in_packet(base_index, requested) {
            removed = removed.wrapping_add(
                consumption
                    .previous_amount
                    .wrapping_sub(consumption.remaining_amount),
            );
            let _ = self.send_player_packet_consumption(&consumption);
        }

        if removed < requested {
            let candidates: Vec<_> = player
                .equipment()
                .traversing_goods()
                .into_iter()
                .filter(|(_, goods)| goods.base_properties_index() == base_index)
                .map(|(column, goods)| (column.position(), goods.identity(), goods.amount()))
                .collect();
            for (position, identity, amount) in candidates {
                if removed >= requested {
                    break;
                }
                let remaining_request = requested.wrapping_sub(removed);
                if remaining_request < amount {
                    if let Some(goods) = player.equipment_mut().get_goods_mut(position) {
                        goods.set_amount(amount.wrapping_sub(remaining_request));
                        let mut message = CS2CContainerObjectAmountChange::default();
                        message.set_source_container(PLAYER_TYPE, player_id, position);
                        message.set_source_container_extend_id(2);
                        message.set_object(identity.object_type, identity.ex_id);
                        message.set_object_amount(amount.wrapping_sub(remaining_request));
                        let _ = message.send_to_player(self, player_id);
                        removed = requested;
                    }
                    break;
                }
                let Some(goods) = player.equipment().find(identity.ex_id) else {
                    continue;
                };
                let facts = runtime.enhancement_equipment_remove_facts(
                    &player,
                    goods,
                    self.globe_setup.pack_add_enabled(),
                );
                let mut recompute =
                    |player: &CPlayer| runtime.recompute_enhancement_player_properties(player);
                let mut report = player.remove_equipment_goods(
                    identity.ex_id,
                    &self.goods_factory,
                    &self.skill_factory,
                    facts,
                    &mut recompute,
                );
                drop(recompute);
                self.publish_player_equipment_remove_report(&mut report, runtime);
                if let EquipmentRemoveOutcome::Removed(result) = &report.outcome {
                    removed = removed.wrapping_add(amount);
                    let previous = PreviousContainer {
                        container_type: PLAYER_TYPE,
                        container_id: player_id,
                        container_extend_id: 2,
                        goods_position: position,
                    };
                    let _ = self.send_container_object_delete(
                        player_id,
                        &previous,
                        result.goods.identity(),
                        result.goods.amount(),
                    );
                }
            }
        }
        self.players.insert(player_id, player);
        removed
    }

    fn send_ci_qing_log(&self, log: &CiQingLog) -> Vec<i32> {
        let mut message = CMessage::new(0x0006_0218);
        message.add_long(log.player_id);
        message.add_long(log.delta);
        message.add_ulong(log.operation);
        message.add_ulong(log.base_index);
        message.add_ulong(log.amount);
        message.send(self, false).into_iter().collect()
    }

    fn send_ci_qing_container_consumption(
        &self,
        consumption: &CiQingContainerConsumption,
    ) -> Vec<i32> {
        if consumption.remaining_amount == 0 {
            let mut message = CS2CContainerObjectMove::default();
            message.set_operation(ContainerObjectMoveOperation::DeleteObject);
            message.set_source_container(PLAYER_TYPE, consumption.player_id, consumption.position);
            message.set_source_container_extend_id(consumption.container_extend_id as i32);
            message.set_source_object(
                consumption.goods.object_type,
                consumption.goods.ex_id,
                consumption.previous_amount,
            );
            return vec![message.send_to_player(self, consumption.player_id)];
        }
        let mut message = CS2CContainerObjectAmountChange::default();
        message.set_source_container(PLAYER_TYPE, consumption.player_id, consumption.position);
        message.set_source_container_extend_id(consumption.container_extend_id as i32);
        message.set_object(consumption.goods.object_type, consumption.goods.ex_id);
        message.set_object_amount(consumption.remaining_amount);
        vec![message.send_to_player(self, consumption.player_id)]
    }

    fn send_ci_qing_container_addition(&self, addition: &CiQingContainerAddition) -> Vec<i32> {
        match &addition.outcome {
            VolumeGoodsAddOutcome::Added(added) => {
                let mut message = CS2CContainerObjectMove::default();
                message.set_operation(ContainerObjectMoveOperation::NewObject);
                message.set_destination_container(
                    PLAYER_TYPE,
                    addition.player_id,
                    addition.position,
                );
                message.set_destination_container_extend_id(addition.container_extend_id as i32);
                message.set_destination_object(added.identity.object_type, added.identity.ex_id);
                message.set_object_stream(addition.old_client_payload.clone().unwrap_or_default());
                vec![message.send_to_player(self, addition.player_id)]
            }
            VolumeGoodsAddOutcome::Stack(GoodsStackMergeOutcome::Merged { target, .. }) => {
                let Some(amount) = addition.resulting_amount else {
                    return Vec::new();
                };
                let mut message = CS2CContainerObjectAmountChange::default();
                message.set_source_container(PLAYER_TYPE, addition.player_id, addition.position);
                message.set_source_container_extend_id(addition.container_extend_id as i32);
                message.set_object(target.object_type, target.ex_id);
                message.set_object_amount(amount);
                vec![message.send_to_player(self, addition.player_id)]
            }
            _ => Vec::new(),
        }
    }

    fn send_ci_qing_hand_consumption(&self, consumption: &CiQingHandConsumption) -> Vec<i32> {
        if consumption.remaining_amount == 0 {
            let mut message = CS2CContainerObjectMove::default();
            message.set_operation(ContainerObjectMoveOperation::DeleteObject);
            message.set_source_container(PLAYER_TYPE, consumption.player_id, 0);
            message.set_source_container_extend_id(3);
            message.set_source_object(
                consumption.goods.object_type,
                consumption.goods.ex_id,
                consumption.previous_amount,
            );
            return vec![message.send_to_player(self, consumption.player_id)];
        }
        let mut message = CS2CContainerObjectAmountChange::default();
        message.set_source_container(PLAYER_TYPE, consumption.player_id, 0);
        message.set_source_container_extend_id(3);
        message.set_object(consumption.goods.object_type, consumption.goods.ex_id);
        message.set_object_amount(consumption.remaining_amount);
        vec![message.send_to_player(self, consumption.player_id)]
    }

    fn send_equipment_da_kong_log(
        &self,
        player: &CPlayer,
        log: &EquipmentDaKongAuditLog,
    ) -> Vec<i32> {
        let mut message = CMessage::new(0x0006_0212);
        message.add_byte(log.reason);
        message.add_long(log.player_id);
        message.base_mut().add_short(player.pk_count() as i16);
        message.add_ulong(player.money());
        message.add_ulong(player.depot_money());
        add_legacy_c_string(message.base_mut(), &log.equipment.description);
        message.base_mut().add_guid(log.equipment.identity.ex_id);
        message.add_ulong(log.cost_price);
        add_legacy_c_string(message.base_mut(), &log.cost_name);
        message.add_ulong(1);
        message.add_long(player.server_region_id().unwrap_or_default());
        message.add_ulong(player.shape().get_tile_x().unwrap_or_default() as u32);
        message.add_ulong(player.shape().get_tile_y().unwrap_or_default() as u32);
        message.add_ulong(player.client_ip());
        for &(_, _, value) in &log.equipment.socket_and_external_values {
            message.add_ulong(value as u32);
        }
        message.send(self, false).into_iter().collect()
    }

    fn send_equipment_da_kong_around_effect(&self, effect: &EquipmentDaKongAroundEffect) -> i32 {
        let mut message = CMessage::new(0x000b_f50a);
        message.add_long(effect.effect_id);
        message.add_ulong((effect.tile_x as f32 + 0.5).to_bits());
        message.add_ulong((effect.tile_y as f32 + 0.5).to_bits());
        let Some(runtime) = GameServerAroundRuntime::new(
            self,
            &self.session_factory,
            self.globe_setup.area_width(),
            self.globe_setup.area_height(),
        ) else {
            return 0;
        };
        let region = self
            .find_region(effect.region_id)
            .map(ServerRegionOwner::base);
        message.send_to_around_position(region, effect.tile_x, effect.tile_y, None, &runtime)
    }

    /// `5404 / PlayEffect`: region-point effect использует тот же canonical
    /// around runtime, что equipment effects, но координаты принадлежат
    /// script expression либо текущей клетке player-а.
    pub(crate) fn script_play_region_effect(
        &self,
        player_id: i32,
        region_id: i32,
        effect_id: i32,
        coordinates: Option<(i32, i32)>,
    ) -> Option<i32> {
        let player = self.find_player(player_id)?;
        let (tile_x, tile_y) = coordinates.unwrap_or((
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
        ));
        let region = self.find_region(region_id)?.base();
        let runtime = GameServerAroundRuntime::new(
            self,
            &self.session_factory,
            self.globe_setup.area_width(),
            self.globe_setup.area_height(),
        )?;
        let mut message = CMessage::new(0x000b_f50a);
        message.add_long(effect_id);
        message.add_ulong((tile_x as f32 + 0.5).to_bits());
        message.add_ulong((tile_y as f32 + 0.5).to_bits());
        Some(message.send_to_around_position(Some(region), tile_x, tile_y, None, &runtime))
    }

    /// `5410 / PlaySound`: direct и spatial branches используют один exact
    /// BF509 payload; ненулевой legacy flag включает рассылку из клетки
    /// script-player-а всем зарегистрированным игрокам соседних area.
    pub(crate) fn script_play_sound(
        &self,
        player_id: i32,
        region_id: i32,
        sound_file: &[u8],
        send_around: bool,
    ) -> Option<i32> {
        let player = self.find_player(player_id)?;
        let region = self.find_region(region_id)?.base();
        let mut message = CMessage::new(0x000b_f509);
        add_legacy_c_string(message.base_mut(), sound_file);
        if !send_around {
            return Some(message.send_to_player(self.net_server(), player_id));
        }
        let tile_x = player.shape().get_tile_x().ok()?;
        let tile_y = player.shape().get_tile_y().ok()?;
        let runtime = GameServerAroundRuntime::new(
            self,
            &self.session_factory,
            self.globe_setup.area_width(),
            self.globe_setup.area_height(),
        )?;
        Some(message.send_to_around_position(Some(region), tile_x, tile_y, None, &runtime))
    }

    /// `2308 / PlayerTalk`: это script-authored local speech, поэтому здесь
    /// нет client chat cooldown/cost/log route; сообщение сразу публикуется
    /// из canonical player shape через обычный spatial around owner.
    pub(crate) fn script_player_talk(
        &mut self,
        player_id: i32,
        text: &[u8],
    ) -> Option<Result<i32, ShapeCoordinateBlock>> {
        let player = self.find_player(player_id)?;
        let player_name = player.player_name().to_vec();
        let mut message = CMessage::new(0x000b_f801);
        message.add_long(0);
        message.add_long(PLAYER_TYPE);
        message.add_long(player_id);
        add_legacy_c_string(message.base_mut(), &player_name);
        add_legacy_c_string(message.base_mut(), text);
        self.send_player_shape_around(player_id, None, &message)
    }

    /// `8100 / OpenChangePlayerNameUI` не вычисляет аргументы. Разрешение
    /// совпадает с native `strstr(playerName, strSpeStr)`; последующий запрос
    /// клиента уже проходит concrete `0x8FB05 → World 0x5FD05 → 0x7FA0E`.
    pub(crate) fn open_script_player_rename(&self, player_id: i32) -> Option<i32> {
        let player = self.find_player(player_id)?;
        let name = player.player_name();
        let special = self.globe_setup.special_string();
        let allowed =
            special.is_empty() || name.windows(special.len()).any(|window| window == special);
        let mut message = CMessage::new(0x000b_f810);
        message.add_byte(u8::from(allowed));
        Some(message.send_to_player(self.net_server(), player_id))
    }

    /// Script `5402/5406` меняет тот же runtime-флаг, который проверяет
    /// ordinary attack owner; отсутствующий script-player сохраняет no-op.
    pub(crate) fn set_script_player_god_mode(&mut self, player_id: i32, enabled: bool) -> bool {
        let Some(player) = self.find_player_mut(player_id) else {
            return false;
        };
        player.set_god_mode(enabled);
        true
    }

    /// `CPlayer::GetGMLevel` требует записи в обеих startup maps и возвращает
    /// клиентский bool-level: любой non-player role в одной из записей даёт 1.
    pub(crate) fn script_player_gm_level(&self, player_id: i32) -> Option<i32> {
        let name = self.find_player(player_id)?.player_name();
        let gm = self.gm_list.gm_info().get(name)?;
        let player_gm = self.gm_list.player_gm_info().get(name)?;
        Some(i32::from(gm.level != 0 || player_gm.level != 0))
    }

    pub(crate) fn send_script_player_gm_mode(&self, player_id: i32) -> Option<i32> {
        let level = self.script_player_gm_level(player_id).unwrap_or(0);
        self.find_player(player_id)?;
        let mut message = CMessage::new(0x000b_fc01);
        message.add_long(level);
        message.add_long(player_id);
        Some(message.send_to_player(self.net_server(), player_id))
    }

    fn equipment_da_kong_run_script<Context: ScriptFunctionRuntime>(
        &mut self,
        report: &mut EquipmentDaKongReport,
        context: &mut Context,
        player: &mut CPlayer,
        script: &'static [u8],
    ) {
        let player_id = player.player_id();
        let region_id = player.server_region_id();
        // В C++ player pointer остаётся в game map во время синхронного script
        // call. Rust-владелец извлекает player для equipment mutation, поэтому
        // на время dispatcher-а возвращаем исходный owned value, оставляя clone
        // только как безопасный placeholder для ссылки вызывающего кода.
        let attached_player = std::mem::replace(player, player.clone());
        let displaced = self.players.insert(player_id, attached_player);
        assert!(
            displaced.is_none(),
            "DaKong owner извлекает игрока перед script dispatch"
        );
        let _ = self.run_script_file(
            script,
            ScriptExecutionContext {
                player_id: Some(player_id),
                region_id,
                ..ScriptExecutionContext::default()
            },
            context,
        );
        *player = self
            .players
            .remove(&player_id)
            .expect("синхронный DaKong script сохраняет canonical player owner");
        report.scripts.push(script.to_vec());
    }

    fn equipment_da_kong_create_socket<Context: ScriptFunctionRuntime>(
        &mut self,
        player: &mut CPlayer,
        plug: &CEquipmentDaKong,
        color_index: i32,
        report: &mut EquipmentDaKongReport,
        context: &mut Context,
    ) {
        const STONES: [&[u8]; 7] = [
            b"FZ1042", b"FZ1043", b"FZ1044", b"FZ1045", b"FZ1046", b"FZ1047", b"FZ1048",
        ];
        let player_id = player.player_id();
        let Some(equipment_id) = Self::equipment_da_kong_equipment_id(plug) else {
            report.outcome = EquipmentDaKongOutcome::MissingEquipment;
            return;
        };
        let Some(equipment) = player.get_goods_by_id(equipment_id) else {
            report.outcome = EquipmentDaKongOutcome::MissingEquipment;
            return;
        };
        let socket_count = equipment.da_kong_count(&self.goods_factory) as usize;
        if socket_count > 6 {
            report.outcome = EquipmentDaKongOutcome::ConditionRejected;
            return;
        }
        let stone_index = self
            .goods_factory
            .query_goods_id_by_original_name(Some(STONES[socket_count]));
        if player.check_item_in_packet(stone_index) == 0
            || equipment.addon_property_value(
                &self.goods_factory,
                crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_DAKONG_1
                    + socket_count as i32,
                1,
            ) != 1
        {
            self.equipment_da_kong_notify(report, player_id, "GS1166");
            report.outcome = EquipmentDaKongOutcome::MissingResource;
            return;
        }
        let succeeded = {
            let (setup, random_state) = (&self.da_kong_xiang_qian, &mut self.random_state);
            setup.get_success_probability(socket_count as i32, |upper_bound| {
                game_legacy_random(random_state, upper_bound)
            })
        };
        if succeeded {
            let mut color = {
                let (setup, random_state) = (&self.da_kong_xiang_qian, &mut self.random_state);
                setup.get_color(color_index, |upper_bound| {
                    game_legacy_random(random_state, upper_bound)
                })
            };
            if socket_count == 6 {
                color = 6;
            }
            let (new_count, equipment_base_index) = {
                let equipment = player
                    .get_goods_by_id_mut(equipment_id)
                    .expect("DaKong equipment проверен до socket mutation");
                let _ = equipment.set_addon_property_modifier_core(
                    crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_DAKONG_1
                        + socket_count as i32,
                    1,
                    color.wrapping_add(1),
                );
                (
                    equipment.da_kong_count(&self.goods_factory),
                    equipment.base_properties_index(),
                )
            };
            if new_count == 6 {
                self.equipment_da_kong_run_script(
                    report,
                    context,
                    player,
                    b"scripts/goods/hole06_gonggao.script",
                );
            } else if new_count == 7 {
                self.equipment_da_kong_run_script(
                    report,
                    context,
                    player,
                    b"scripts/goods/hole07_gonggao.script",
                );
            }
            if matches!(new_count, 3 | 6 | 7) {
                let group = match new_count {
                    3 => 0,
                    6 => 1,
                    _ => 2,
                };
                let external = {
                    let (setup, random_state) = (&self.da_kong_xiang_qian, &mut self.random_state);
                    setup.make_sure_external_attribute(group, equipment_base_index, |upper_bound| {
                        game_legacy_random(random_state, upper_bound)
                    })
                };
                let property =
                    crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_DAKONG_EXTERN_1
                        + group;
                let equipment = player
                    .get_goods_by_id_mut(equipment_id)
                    .expect("DaKong equipment сохраняется после announcement script");
                let (external_type, external_value) = external.unwrap_or_default();
                let _ = equipment.set_addon_property_value_first_core(property, 1, external_type);
                let _ = equipment.set_addon_property_modifier_core(property, 2, external_value);
            }
            self.equipment_da_kong_notify(report, player_id, "GS1164");
        } else {
            self.equipment_da_kong_notify(report, player_id, "GS1165");
        }
        let equipment = player
            .get_goods_by_id(equipment_id)
            .expect("DaKong equipment сохраняется до audit");
        self.equipment_da_kong_log(report, player, 1, stone_index, equipment);
        self.equipment_da_kong_consume_packet(report, player, stone_index);
        let _ = self.equipment_da_kong_publish_preview(player, plug, report, context);
        report.return_value = 1;
        report.outcome = EquipmentDaKongOutcome::Completed;
    }

    fn equipment_da_kong_publish_preview<Context: EquipmentDaKongContext>(
        &self,
        player: &CPlayer,
        plug: &CEquipmentDaKong,
        report: &mut EquipmentDaKongReport,
        context: &mut Context,
    ) -> bool {
        let goods = Self::equipment_da_kong_equipment_id(plug)
            .and_then(|goods_id| player.get_goods_by_id(goods_id))
            .or_else(|| player.get_goods_by_id(plug.upgrade_container().last_goods()));
        let Some(mut preview) = goods.cloned() else {
            return false;
        };
        let gems = self.equipment_da_kong_gems(player, plug);
        let _ = deal_enchase_gems(&mut preview, &gems, &self.goods_factory, false);
        self.equipment_da_kong_publish_update(report, context, player.player_id(), &preview);
        true
    }

    fn equipment_da_kong_change_color<Context: EquipmentDaKongContext>(
        &mut self,
        player: &mut CPlayer,
        plug: &CEquipmentDaKong,
        socket: i32,
        report: &mut EquipmentDaKongReport,
        context: &mut Context,
    ) {
        if !(1..=7).contains(&socket) {
            report.outcome = EquipmentDaKongOutcome::InvalidParameter;
            return;
        }
        let player_id = player.player_id();
        let Some(equipment_id) = Self::equipment_da_kong_equipment_id(plug) else {
            report.outcome = EquipmentDaKongOutcome::MissingEquipment;
            return;
        };
        let property =
            crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_DAKONG_1 + socket - 1;
        let Some(equipment) = player.get_goods_by_id(equipment_id) else {
            report.outcome = EquipmentDaKongOutcome::MissingEquipment;
            return;
        };
        let color = equipment.addon_property_value(&self.goods_factory, property, 1);
        let gem_index = equipment.addon_property_value(&self.goods_factory, property, 2);
        if !(2..=8).contains(&color) || gem_index != 0 {
            self.equipment_da_kong_notify(report, player_id, "GS1170");
            report.return_value = 1;
            report.outcome = EquipmentDaKongOutcome::ConditionRejected;
            return;
        }
        let stone_index = self
            .goods_factory
            .query_goods_id_by_original_name(Some(b"FZ1049"));
        if player.check_item_in_packet(stone_index) == 0 {
            self.equipment_da_kong_notify(report, player_id, "GS1171");
            report.return_value = 1;
            report.outcome = EquipmentDaKongOutcome::MissingResource;
            return;
        }
        self.equipment_da_kong_consume_packet(report, player, stone_index);
        if player
            .get_goods_by_id(equipment_id)
            .is_some_and(|equipment| {
                equipment.addon_property_value(&self.goods_factory, property, 2) >= 1
            })
        {
            self.equipment_da_kong_notify(report, player_id, "GS1172");
            report.return_value = 1;
            report.outcome = EquipmentDaKongOutcome::ConditionRejected;
            return;
        }
        let mut new_color = {
            let (setup, random_state) = (&self.da_kong_xiang_qian, &mut self.random_state);
            setup.get_color(socket - 1, |upper_bound| {
                game_legacy_random(random_state, upper_bound)
            })
        };
        if socket == 7 {
            new_color = 6;
        }
        let equipment = player
            .get_goods_by_id_mut(equipment_id)
            .expect("color equipment проверен до mutation");
        let _ = equipment.set_addon_property_modifier_core(property, 1, new_color.wrapping_add(1));
        self.equipment_da_kong_notify(report, player_id, "GS1173");
        let equipment = player
            .get_goods_by_id(equipment_id)
            .expect("color equipment сохраняется до audit/update");
        self.equipment_da_kong_log(report, player, 0, stone_index, equipment);
        self.equipment_da_kong_publish_update(report, context, player_id, equipment);
        report.return_value = 1;
        report.outcome = EquipmentDaKongOutcome::Completed;
    }

    fn equipment_da_kong_consume_shadow_gems(
        &self,
        report: &mut EquipmentDaKongReport,
        player: &mut CPlayer,
        plug: &mut CEquipmentDaKong,
    ) {
        use crate::gameserver::appserver::container::cequipmentdakongcontainer::DaKongCell;
        let ledger = plug
            .upgrade_container()
            .equipment_goods()
            .iter()
            .map(|(&position, &goods_id)| (position, goods_id))
            .collect::<Vec<_>>();
        for (position, goods_id) in ledger {
            let Some(cell) = DaKongCell::from_position(position) else {
                continue;
            };
            if goods_id == CGuid::GUID_INVALID {
                continue;
            }
            let previous = plug
                .upgrade_container()
                .original_container_information(goods_id)
                .unwrap_or_default();
            let identity = player
                .get_goods_by_id(goods_id)
                .map(CGoods::identity)
                .unwrap_or(ShapeIdentity {
                    object_type: 700,
                    id: 0,
                    ex_id: goods_id,
                });
            let mut external_deliveries = Vec::new();
            if let Some(consumption) = player.remove_packet_goods_by_id(goods_id, 1) {
                external_deliveries = self.send_player_packet_consumption(&consumption);
                report.packet_consumptions.push(consumption);
                report
                    .packet_consumption_deliveries
                    .push(external_deliveries.clone());
                let _ = plug
                    .upgrade_container_mut()
                    .on_source_removed(Some(goods_id), 1);
            } else {
                let _ = plug
                    .upgrade_container_mut()
                    .invalidate_equipment_goods(position);
            }
            report.gem_consumptions.push(
                crate::gameserver::appserver::session::cequipmentdakong::EquipmentDaKongGemConsumption {
                    cell,
                    goods: identity,
                    previous,
                    external_deliveries,
                },
            );
        }
    }

    fn equipment_da_kong_enchase<Context: ScriptFunctionRuntime>(
        &mut self,
        player: &mut CPlayer,
        plug: &mut CEquipmentDaKong,
        _parameter: i32,
        report: &mut EquipmentDaKongReport,
        context: &mut Context,
    ) {
        let player_id = player.player_id();
        let Some(equipment_id) = Self::equipment_da_kong_equipment_id(plug) else {
            report.outcome = EquipmentDaKongOutcome::MissingEquipment;
            report.return_value = 1;
            return;
        };
        let gems = self.equipment_da_kong_gems(player, plug);
        let effects = {
            let Some(equipment) = player.get_goods_by_id_mut(equipment_id) else {
                report.outcome = EquipmentDaKongOutcome::MissingEquipment;
                report.return_value = 1;
                return;
            };
            deal_enchase_gems(equipment, &gems, &self.goods_factory, true)
        };
        let changed = effects
            .events
            .iter()
            .any(|event| matches!(event, EquipmentDaKongEnchaseEvent::GemApplied { .. }));
        for event in effects.events {
            match event {
                EquipmentDaKongEnchaseEvent::Notification(string_id) => {
                    self.equipment_da_kong_notify(report, player_id, string_id);
                }
                EquipmentDaKongEnchaseEvent::GemApplied {
                    gem,
                    equipment,
                    audit,
                } => {
                    if audit {
                        self.equipment_da_kong_log_snapshot(
                            report,
                            player,
                            2,
                            gem.base_index,
                            equipment,
                        );
                    }
                }
                EquipmentDaKongEnchaseEvent::Script(script) => {
                    self.equipment_da_kong_run_script(report, context, player, script);
                }
            }
        }
        if changed {
            self.equipment_da_kong_notify(report, player_id, "GS1167");
        }
        self.equipment_da_kong_consume_shadow_gems(report, player, plug);
        let equipment = player
            .get_goods_by_id(equipment_id)
            .expect("enchase equipment сохраняется после gem consumption");
        self.equipment_da_kong_publish_update(report, context, player_id, equipment);
        report.return_value = 1;
        report.outcome = EquipmentDaKongOutcome::Completed;
    }

    fn equipment_da_kong_destroy_gem<Context: EquipmentDaKongContext>(
        &mut self,
        player: &mut CPlayer,
        plug: &CEquipmentDaKong,
        socket: u32,
        report: &mut EquipmentDaKongReport,
        context: &mut Context,
    ) {
        let player_id = player.player_id();
        let Some(equipment_id) = Self::equipment_da_kong_equipment_id(plug) else {
            report.outcome = EquipmentDaKongOutcome::MissingEquipment;
            return;
        };
        if player.check_item_in_packet(DA_KONG_USE_SINKER_INDEX) == 0 {
            self.equipment_da_kong_notify(report, player_id, "GS1174");
            report.outcome = EquipmentDaKongOutcome::MissingResource;
            return;
        }
        let gems = self.equipment_da_kong_gems(player, plug);
        let mut removed = false;
        let mut new_seven_after_removal = None;
        if (1..=7).contains(&socket) {
            let property = crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_DAKONG_1
                + socket as i32
                - 1;
            let (socket_color, gem_index) = player
                .get_goods_by_id(equipment_id)
                .map(|equipment| {
                    (
                        equipment.addon_property_value(&self.goods_factory, property, 1),
                        equipment.addon_property_value(&self.goods_factory, property, 2) as u32,
                    )
                })
                .unwrap_or_default();
            if !(2..=8).contains(&socket_color) || gem_index == 0 {
                self.equipment_da_kong_notify(report, player_id, "GS1175");
            } else {
                let slot_seven = gems[6].or_else(|| {
                    player.get_goods_by_id(equipment_id).and_then(|equipment| {
                        EquipmentDaKongGemSnapshot::from_catalog(
                            equipment.addon_property_value(
                                &self.goods_factory,
                                crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_DAKONG_1 + 6,
                                2,
                            ) as u32,
                            &self.goods_factory,
                        )
                    })
                });
                let equipment = player
                    .get_goods_by_id_mut(equipment_id)
                    .expect("destroy equipment проверен до mutation");
                deal_with_da_kong_external_attributes(
                    equipment,
                    &self.goods_factory,
                    slot_seven,
                    false,
                );
                if socket != 7 {
                    deal_with_da_kong_seven(equipment, &self.goods_factory, false);
                }
                let mut add_types = BTreeSet::new();
                CDaKongXiangQian::get_add_type(&mut add_types);
                if let Some(base) = self.goods_factory.query_goods_base_properties(gem_index) {
                    let gem =
                        EquipmentDaKongGemSnapshot::from_catalog(gem_index, &self.goods_factory);
                    let gem_color = base
                        .get_addon_property_values(
                            crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_BAOSHI_COLOR,
                        )
                        .iter()
                        .find(|value| value.id == 1)
                        .map_or(0, |value| value.base_value);
                    for addon in base.addon_properties() {
                        if !add_types.contains(&addon.property_type) {
                            continue;
                        }
                        let mut value = addon
                            .values
                            .iter()
                            .find(|value| value.id == 1)
                            .or_else(|| addon.values.first())
                            .map_or(0, |value| value.base_value);
                        if socket < 7 && gem_color == 8 {
                            value = 0;
                        }
                        if socket == 7 && gem_color != 8 {
                            value /= 2;
                        }
                        if socket == 7
                            && gem_color == 8
                            && !gem.is_some_and(|gem| {
                                equipment_da_kong_condition(gem, equipment, &self.goods_factory)
                            })
                        {
                            value = 0;
                        }
                        let _ = equipment.cut_addon_property_value(
                            &self.goods_factory,
                            addon.property_type,
                            1,
                            value,
                            gem_index,
                        );
                    }
                }
                let _ = equipment.set_addon_property_modifier_core(property, 2, 0);
                deal_with_da_kong_seven(equipment, &self.goods_factory, true);
                let new_seven_index = equipment.addon_property_value(
                    &self.goods_factory,
                    crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_DAKONG_1 + 6,
                    2,
                ) as u32;
                new_seven_after_removal = gems[6].or_else(|| {
                    EquipmentDaKongGemSnapshot::from_catalog(new_seven_index, &self.goods_factory)
                });
                removed = true;
            }
        }
        if removed {
            self.equipment_da_kong_consume_packet(report, player, DA_KONG_USE_SINKER_INDEX);
            let equipment = player
                .get_goods_by_id_mut(equipment_id)
                .expect("destroy equipment сохраняется до extern restore");
            deal_with_da_kong_external_attributes(
                equipment,
                &self.goods_factory,
                new_seven_after_removal,
                true,
            );
            let equipment = player
                .get_goods_by_id(equipment_id)
                .expect("destroy equipment сохраняется до audit");
            self.equipment_da_kong_log(report, player, 3, DA_KONG_USE_SINKER_INDEX, equipment);
        }
        let mut preview = player
            .get_goods_by_id(equipment_id)
            .expect("destroy equipment сохраняется до preview")
            .clone();
        let _ = deal_enchase_gems(&mut preview, &gems, &self.goods_factory, false);
        self.equipment_da_kong_publish_update(report, context, player_id, &preview);
        report.return_value = 1;
        report.outcome = EquipmentDaKongOutcome::Completed;
    }

    pub(crate) const fn words_filter(&self) -> &CWordsFilter {
        &self.words_filter
    }

    pub(crate) const fn words_filter_mut(&mut self) -> &mut CWordsFilter {
        &mut self.words_filter
    }

    pub(crate) const fn jjc_level_data(&self) -> &BTreeMap<i32, i32> {
        &self.jjc_level_data
    }

    pub(crate) fn clear_jjc_level_data(&mut self) {
        self.jjc_level_data.clear();
    }

    pub(crate) fn insert_jjc_level_data(&mut self, key: i32, value: i32) -> bool {
        if self.jjc_level_data.contains_key(&key) {
            return false;
        }
        self.jjc_level_data.insert(key, value);
        true
    }

    pub(crate) const fn tao_zhuang_setup(&self) -> &CTaoZhuangSetup {
        &self.tao_zhuang_setup
    }

    pub(crate) const fn tao_zhuang_setup_mut(&mut self) -> &mut CTaoZhuangSetup {
        &mut self.tao_zhuang_setup
    }

    pub(crate) const fn ci_qing_setup(&self) -> &CCiQingSetup {
        &self.ci_qing_setup
    }

    pub(crate) const fn ci_qing_setup_mut(&mut self) -> &mut CCiQingSetup {
        &mut self.ci_qing_setup
    }

    /// Полный player query `goodsmessage 0x8FC2F`. Первый непустой old-client
    /// preview сохраняет подтверждённый ранний return EXE; create miss либо
    /// пустой payload позволяют перейти к следующему ordered set entry.
    pub(crate) fn query_ci_qing_goods<Context: OldClientGoodsCodec>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> Option<CiQingGoodsQueryReport> {
        let base_indices: Vec<_> = self.find_player(player_id)?.ci_qing_list().collect();
        let mut previews = Vec::new();
        for base_index in base_indices {
            let created = {
                let (random_state, goods_factory, fairy_exp_conf, battle_fairy_exp_config) = (
                    &mut self.random_state,
                    &self.goods_factory,
                    &self.fairy_exp_conf,
                    &self.battle_fairy_exp_config,
                );
                let mut random = |upper_bound| game_legacy_random(random_state, upper_bound);
                goods_factory.create_goods(
                    base_index,
                    &mut random,
                    || CGuid::create().unwrap_or(CGuid::GUID_INVALID),
                    |equip_level, level| fairy_exp_conf.dw_exp_up(equip_level, level),
                    |equip_level, level| battle_fairy_exp_config.dw_exp_up(equip_level, level),
                )
            };
            let Some(goods) = created else {
                continue;
            };
            let old_client_payload = context.encode_goods_for_old_client(&goods);
            let mut message = CMessage::new(0x0c_010c);
            message.base_mut().add(&old_client_payload);
            let delivery = message.send_to_player(self.net_server(), player_id);
            let stop = !old_client_payload.is_empty();
            previews.push(CiQingGoodsPreview {
                base_index,
                old_client_payload,
                delivery,
            });
            if stop {
                break;
            }
        }
        Some(CiQingGoodsQueryReport {
            player_id,
            previews,
        })
    }

    /// Global setup query `goodsmessage 0x8FC30`; serialization block не
    /// превращается в пустой успешный packet.
    pub(crate) fn query_ci_qing_setup(&self, player_id: i32) -> CiQingSetupQueryReport {
        let mut payload = Vec::new();
        let payload = self
            .ci_qing_setup
            .add_byte_to_array(&mut payload)
            .map(|()| payload);
        let delivery = payload.as_ref().ok().map(|payload| {
            let mut message = CMessage::new(0x0c_010d);
            message.base_mut().add(payload);
            message.send_to_player(self.net_server(), player_id)
        });
        CiQingSetupQueryReport {
            player_id,
            payload,
            delivery,
        }
    }

    pub(crate) fn ci_qing_message_enabled(&self, player_id: i32) -> bool {
        self.globe_setup.ci_qing_enabled()
            && self
                .find_player(player_id)
                .is_some_and(CPlayer::ci_qing_open)
    }

    pub(crate) fn battle_fairy_enabled(&self) -> bool {
        self.globe_setup.battle_fairy_enabled()
    }

    /// Полный `CPlayer::MakeCiQingNode` и его `goodsmessage 0x8FC31` tail.
    /// Resource logs предшествуют каждому DeleteGoods-effect; финальный
    /// positive log сохраняет странный native count оставшихся в vector-е
    /// (то есть не добавленных), после чего всегда отправляется `0xBF932`.
    pub(crate) fn make_ci_qing_node<Context: OldClientGoodsCodec>(
        &mut self,
        player_id: i32,
        base_index: u32,
        amount: u32,
        context: &mut Context,
    ) -> Option<CiQingMakeReport> {
        let mut report = CiQingMakeReport {
            player_id,
            requested_base_index: base_index,
            requested_amount: amount,
            result_base_index: 0,
            outcome: CiQingMakeOutcome::UnknownNode,
            logs: Vec::new(),
            consumptions: Vec::new(),
            additions: Vec::new(),
            rejected_goods: Vec::new(),
            deliveries: Vec::new(),
        };
        let player = self.find_player(player_id)?;
        if !player.ci_qing_list().any(|entry| entry == base_index) {
            return Some(self.finish_ci_qing_make(report));
        }
        if amount >= 10 {
            report.outcome = CiQingMakeOutcome::AmountOutOfRange;
            return Some(self.finish_ci_qing_make(report));
        }
        let Some(recipe) = self
            .ci_qing_setup
            .make()
            .iter()
            .find(|node| node.destination_base_index == base_index)
            .copied()
        else {
            report.outcome = CiQingMakeOutcome::MissingRecipe;
            return Some(self.finish_ci_qing_make(report));
        };
        let need_a = recipe.source_a_count.wrapping_mul(amount);
        let need_b = recipe.source_b_count.wrapping_mul(amount);
        if player.check_item_in_packet(recipe.source_a_base_index) < need_a {
            report.outcome = CiQingMakeOutcome::InsufficientSourceA;
            return Some(self.finish_ci_qing_make(report));
        }
        if player.check_item_in_packet(recipe.source_b_base_index) < need_b {
            report.outcome = CiQingMakeOutcome::InsufficientSourceB;
            return Some(self.finish_ci_qing_make(report));
        }
        if !player.packet().check_space(amount) {
            report.outcome = CiQingMakeOutcome::InsufficientPacketSpace;
            return Some(self.finish_ci_qing_make(report));
        }

        for (source_base_index, required) in [
            (recipe.source_a_base_index, need_a),
            (recipe.source_b_base_index, need_b),
        ] {
            let log = CiQingLog {
                player_id,
                delta: -1,
                operation: 1,
                base_index: source_base_index,
                amount: required,
            };
            report
                .deliveries
                .push(CiQingMakeDelivery::Audit(self.send_ci_qing_log(&log)));
            report.logs.push(log);
            let consumptions = self
                .players
                .get_mut(&player_id)
                .expect("player проверен до CiQing resource removal")
                .remove_item_in_packet(source_base_index, required);
            for consumption in consumptions {
                report.deliveries.push(CiQingMakeDelivery::Consumption(
                    self.send_player_packet_consumption(&consumption),
                ));
                report.consumptions.push(consumption);
            }
        }

        let created = {
            let (random_state, goods_factory, fairy_exp_conf, battle_fairy_exp_config) = (
                &mut self.random_state,
                &self.goods_factory,
                &self.fairy_exp_conf,
                &self.battle_fairy_exp_config,
            );
            let mut random = |upper_bound| game_legacy_random(random_state, upper_bound);
            goods_factory.create_goods_batch(
                recipe.destination_base_index,
                amount,
                &mut random,
                || CGuid::create().unwrap_or(CGuid::GUID_INVALID),
                |equip_level, level| fairy_exp_conf.dw_exp_up(equip_level, level),
                |equip_level, level| battle_fairy_exp_config.dw_exp_up(equip_level, level),
            )
        };
        let (additions, rejected) = {
            let goods_factory = &self.goods_factory;
            let player = self
                .players
                .get_mut(&player_id)
                .expect("player проверен до CiQing packet add");
            let mut encode = |goods: &CGoods| context.encode_goods_for_old_client(goods);
            player.add_goods_to_packet(created, goods_factory, &mut encode)
        };
        for addition in additions {
            if addition.resulting_amount.is_some() {
                report.deliveries.push(CiQingMakeDelivery::Addition(
                    self.send_player_packet_addition(&addition),
                ));
            }
            report.additions.push(addition);
        }
        report.rejected_goods = rejected.iter().map(CGoods::identity).collect();
        let log = CiQingLog {
            player_id,
            delta: 1,
            operation: 1,
            base_index: recipe.destination_base_index,
            amount: rejected.len() as u32,
        };
        report
            .deliveries
            .push(CiQingMakeDelivery::Audit(self.send_ci_qing_log(&log)));
        report.logs.push(log);
        report.result_base_index = recipe.destination_base_index;
        report.outcome = CiQingMakeOutcome::Completed;
        Some(self.finish_ci_qing_make(report))
    }

    fn finish_ci_qing_make(&self, mut report: CiQingMakeReport) -> CiQingMakeReport {
        let mut message = CMessage::new(0x0b_f932);
        message.add_ulong(report.result_base_index);
        report.deliveries.push(CiQingMakeDelivery::Result(
            message.send_to_player(self.net_server(), report.player_id),
        ));
        report
    }

    pub(crate) fn compose_ci_qing_node<Context: CiQingComposeContext>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> Option<CiQingComposeReport> {
        let slots = self.find_player(player_id).map(|player| {
            (
                player
                    .ci_qing_compose_goods(0)
                    .map(|goods| (goods.base_properties_index(), goods.amount())),
                player
                    .ci_qing_compose_goods(1)
                    .map(|goods| (goods.base_properties_index(), goods.amount())),
                player.ci_qing_compose_goods(2).is_some(),
            )
        })?;
        let mut report = CiQingComposeReport {
            player_id,
            source_indices: None,
            recipe_found: false,
            roll: None,
            selected_result: None,
            outcome: CiQingComposeOutcome::MissingSources,
            logs: Vec::new(),
            crystal_consumptions: Vec::new(),
            source_consumptions: Vec::new(),
            result_addition: None,
            rejected_result: None,
            deliveries: Vec::new(),
        };
        let (Some(slot_zero), Some(slot_one)) = (slots.0, slots.1) else {
            let delivery = colored_player_notice_message(
                0xffff_ffff,
                0,
                self.get_string_by_id(b"PLAYER001006"),
            )
            .send_to_player(self.net_server(), player_id);
            report
                .deliveries
                .push(CiQingComposeDelivery::Player(delivery));
            return Some(report);
        };
        let (source_a_index, source_b_index) = if slot_zero.0 < slot_one.0 {
            (slot_zero.0, slot_one.0)
        } else {
            (slot_one.0, slot_zero.0)
        };
        report.source_indices = Some((source_a_index, source_b_index));
        if slots.2 {
            report.outcome = CiQingComposeOutcome::ResultSlotOccupied;
            let delivery = colored_player_notice_message(
                0xffff_ffff,
                0,
                self.get_string_by_id(b"PLAYER001005"),
            )
            .send_to_player(self.net_server(), player_id);
            report
                .deliveries
                .push(CiQingComposeDelivery::Player(delivery));
            return Some(report);
        }

        let recipe = self
            .ci_qing_setup
            .compute_node(source_a_index, source_b_index)
            .or_else(|| self.ci_qing_setup.compute_node(0, 0));
        report.recipe_found = recipe.is_some();
        let required_crystal = recipe.as_ref().map_or(100, |node| node.crystal_count);
        let required_money = recipe.as_ref().map_or(100, |node| node.money);
        let crystal_index = self
            .goods_factory
            .query_goods_id_by_original_name(Some(b"FZ0965"));
        let player = self
            .find_player(player_id)
            .expect("player snapshot получен перед CiQing compose");
        if player.money() < required_money
            || player.check_item_in_packet(crystal_index) < required_crystal
        {
            report.outcome = CiQingComposeOutcome::InsufficientPayment;
            let delivery = colored_player_notice_message(
                0xffff_0000,
                0xffff_ff00,
                self.get_string_by_id(b"PLAYER001001"),
            )
            .send_to_player(self.net_server(), player_id);
            report
                .deliveries
                .push(CiQingComposeDelivery::Player(delivery));
            return Some(report);
        }

        let roll = game_legacy_random(&mut self.random_state, 10_000) as u32;
        report.roll = Some(roll);
        let mut result_index = None;
        if let Some(recipe) = recipe.as_ref() {
            let money_change = {
                let (players, goods_factory) = (&mut self.players, &self.goods_factory);
                players
                    .get_mut(&player_id)
                    .expect("player проверен до CiQing money mutation")
                    .decrease_money(required_money, goods_factory)
            };
            report.deliveries.push(CiQingComposeDelivery::Money(
                self.send_player_money_decrease(player_id, &money_change.outcome),
            ));
            let crystal_log = CiQingLog {
                player_id,
                delta: -1,
                operation: 2,
                base_index: crystal_index,
                amount: required_crystal,
            };
            report.deliveries.push(CiQingComposeDelivery::Audit(
                self.send_ci_qing_log(&crystal_log),
            ));
            report.logs.push(crystal_log);
            let consumptions = self
                .players
                .get_mut(&player_id)
                .expect("player проверен до crystal removal")
                .remove_item_in_packet(crystal_index, required_crystal);
            for consumption in consumptions {
                report
                    .deliveries
                    .push(CiQingComposeDelivery::PacketConsumption(
                        self.send_player_packet_consumption(&consumption),
                    ));
                report.crystal_consumptions.push(consumption);
            }
            if roll < recipe.compose_probability {
                let selected = CCiQingSetup::random_choice(recipe, |upper_bound| {
                    game_legacy_random(&mut self.random_state, upper_bound)
                });
                result_index = Some(if selected == 0 {
                    source_a_index
                } else {
                    selected
                });
            }
        } else if roll < 8000 {
            result_index = Some(source_a_index);
        }
        report.selected_result = result_index;

        if let Some(result_index) = result_index {
            let created = {
                let (random_state, goods_factory, fairy_exp_conf, battle_fairy_exp_config) = (
                    &mut self.random_state,
                    &self.goods_factory,
                    &self.fairy_exp_conf,
                    &self.battle_fairy_exp_config,
                );
                let mut random = |upper_bound| game_legacy_random(random_state, upper_bound);
                goods_factory.create_goods(
                    result_index,
                    &mut random,
                    || CGuid::create().unwrap_or(CGuid::GUID_INVALID),
                    |equip_level, level| fairy_exp_conf.dw_exp_up(equip_level, level),
                    |equip_level, level| battle_fairy_exp_config.dw_exp_up(equip_level, level),
                )
            };
            if let Some(created) = created {
                let (addition, rejected) = {
                    let goods_factory = &self.goods_factory;
                    let player = self
                        .players
                        .get_mut(&player_id)
                        .expect("player проверен до CiQing result add");
                    let mut encode = |goods: &CGoods| context.encode_goods_for_old_client(goods);
                    player.add_goods_to_ci_qing(created, 2, true, goods_factory, &mut encode)
                };
                if addition.resulting_amount.is_some() {
                    report
                        .deliveries
                        .push(CiQingComposeDelivery::ContainerAddition(
                            self.send_ci_qing_container_addition(&addition),
                        ));
                }
                report.rejected_result = rejected.as_ref().map(CGoods::identity);
                report.result_addition = Some(addition);
            }
        }

        for (position, source) in [(0, slot_zero), (1, slot_one)] {
            let log = CiQingLog {
                player_id,
                delta: -1,
                operation: 2,
                base_index: source.0,
                amount: source.1,
            };
            report
                .deliveries
                .push(CiQingComposeDelivery::Audit(self.send_ci_qing_log(&log)));
            report.logs.push(log);
            if let Some(consumption) = self
                .players
                .get_mut(&player_id)
                .expect("player проверен до CiQing source removal")
                .remove_ci_qing_compose_goods(position)
            {
                report
                    .deliveries
                    .push(CiQingComposeDelivery::ContainerConsumption(
                        self.send_ci_qing_container_consumption(&consumption),
                    ));
                report.source_consumptions.push(consumption);
            }
        }

        let result = self
            .find_player(player_id)
            .and_then(|player| player.ci_qing_compose_goods(2))
            .map(|goods| (goods.base_properties_index(), goods.amount()));
        let string_id = if result.is_some() {
            b"PLAYER001002".as_slice()
        } else {
            b"PLAYER001003".as_slice()
        };
        let mut message = CMessage::new(0x0b_f81b);
        add_legacy_c_string(message.base_mut(), self.get_string_by_id(string_id));
        message.add_ulong(0);
        report.deliveries.push(CiQingComposeDelivery::Player(
            message.send_to_player(self.net_server(), player_id),
        ));
        if let Some((result_index, _)) = result {
            // Native добавляет index уже после SendToPlayer; mutation остаётся
            // намеренно невидимой клиенту.
            message.add_ulong(result_index);
        }
        if let Some((result_index, result_amount)) = result {
            self.players
                .get_mut(&player_id)
                .expect("player проверен до CiQing unlock")
                .restore_ci_qing_entry(result_index);
            if let Some(query) = self.query_ci_qing_goods(player_id, context) {
                report
                    .deliveries
                    .push(CiQingComposeDelivery::GoodsQuery(query));
            }
            let log = CiQingLog {
                player_id,
                delta: 1,
                operation: 2,
                base_index: result_index,
                amount: result_amount,
            };
            report
                .deliveries
                .push(CiQingComposeDelivery::Audit(self.send_ci_qing_log(&log)));
            report.logs.push(log);
            report.outcome = CiQingComposeOutcome::Succeeded;
        } else {
            report.outcome = CiQingComposeOutcome::Failed;
        }
        Some(report)
    }

    pub(crate) fn delete_goods_from_ci_qing<Context: CiQingComposeContext>(
        &mut self,
        player_id: i32,
        position: u32,
        context: &mut Context,
    ) -> Option<CiQingDeleteReport> {
        let mut report = CiQingDeleteReport {
            player_id,
            position,
            outcome: CiQingDeleteOutcome::MissingGoodsOrResetItem,
            logs: Vec::new(),
            reset_consumptions: Vec::new(),
            goods_consumption: None,
            deliveries: Vec::new(),
        };
        let reset_index = self
            .goods_factory
            .query_goods_id_by_original_name(Some(b"CQ0008"));
        let can_delete = self.find_player(player_id).is_some_and(|player| {
            player.ci_qing_goods(position).is_some()
                && player.check_item_in_packet(reset_index) != 0
        });
        if !can_delete {
            let delivery = colored_player_notice_message(
                0xffff_ffff,
                0,
                self.get_string_by_id(b"PLAYER001004"),
            )
            .send_to_player(self.net_server(), player_id);
            report
                .deliveries
                .push(CiQingDeleteDelivery::Player(delivery));
            return self.find_player(player_id).map(|_| report);
        }
        let reset_log = CiQingLog {
            player_id,
            delta: -1,
            operation: 4,
            base_index: reset_index,
            amount: 1,
        };
        report.deliveries.push(CiQingDeleteDelivery::Audit(
            self.send_ci_qing_log(&reset_log),
        ));
        report.logs.push(reset_log);
        let reset_consumptions = self
            .players
            .get_mut(&player_id)
            .expect("player проверен до CiQing reset-item removal")
            .remove_item_in_packet(reset_index, 1);
        for consumption in reset_consumptions {
            report
                .deliveries
                .push(CiQingDeleteDelivery::PacketConsumption(
                    self.send_player_packet_consumption(&consumption),
                ));
            report.reset_consumptions.push(consumption);
        }
        if let Some(consumption) = self
            .players
            .get_mut(&player_id)
            .expect("player проверен до CiQing goods removal")
            .remove_ci_qing_goods(position, 1)
        {
            report
                .deliveries
                .push(CiQingDeleteDelivery::ContainerConsumption(
                    self.send_ci_qing_container_consumption(&consumption),
                ));
            report.goods_consumption = Some(consumption);
        }
        report.deliveries.push(CiQingDeleteDelivery::PropertyUpdate(
            self.refresh_ci_qing_player_property(player_id, context),
        ));
        report.outcome = CiQingDeleteOutcome::Deleted;
        Some(report)
    }

    pub(crate) fn mount_ci_qing_from_hand<Context: CiQingComposeContext>(
        &mut self,
        player_id: i32,
        amount: u32,
        context: &mut Context,
    ) -> Option<CiQingMountReport> {
        let mut report = CiQingMountReport {
            player_id,
            amount,
            position: None,
            chance: None,
            roll: None,
            outcome: CiQingMountOutcome::Rejected,
            logs: Vec::new(),
            hand_consumption: None,
            material_consumptions: Vec::new(),
            addition: None,
            rejected_clone: None,
            deliveries: Vec::new(),
        };
        let player = self.find_player(player_id)?;
        let Some((position, improve_level, base_chance)) =
            player.ci_qing_mount_facts(&self.goods_factory)
        else {
            return Some(report);
        };
        report.position = Some(position);
        if player.ci_qing_goods(position).is_some() {
            return Some(report);
        }
        let Some(node) = self.ci_qing_setup.improve_node(improve_level) else {
            return Some(report);
        };
        if player.check_item_in_packet(node.base_index) < amount {
            return Some(report);
        }
        let chance = base_chance
            .wrapping_add(node.probability.wrapping_mul(amount))
            .min(10_000);
        let roll = game_legacy_random(&mut self.random_state, 0x2711) as u32;
        report.chance = Some(chance);
        report.roll = Some(roll);
        let succeeded = roll < chance;
        let hand_goods = self
            .find_player(player_id)
            .and_then(CPlayer::ci_qing_hand_goods)
            .expect("mount facts подтверждают hand goods");
        let hand_base_index = hand_goods.base_properties_index();
        let cloned_hand_goods = if succeeded {
            let Some(cloned) = context.clone_ci_qing_hand_goods(hand_goods) else {
                return Some(report);
            };
            Some(cloned)
        } else {
            None
        };

        if !succeeded {
            let log = CiQingLog {
                player_id,
                delta: -1,
                operation: 3,
                base_index: hand_base_index,
                amount: 1,
            };
            report
                .deliveries
                .push(CiQingMountDelivery::Audit(self.send_ci_qing_log(&log)));
            report.logs.push(log);
        }
        if let Some(consumption) = self
            .players
            .get_mut(&player_id)
            .expect("player проверен до CiQing hand removal")
            .remove_ci_qing_hand_goods()
        {
            report.deliveries.push(CiQingMountDelivery::HandConsumption(
                self.send_ci_qing_hand_consumption(&consumption),
            ));
            report.hand_consumption = Some(consumption);
        }

        if succeeded {
            let (addition, rejected) = {
                let goods_factory = &self.goods_factory;
                let player = self
                    .players
                    .get_mut(&player_id)
                    .expect("player проверен до CiQing mounted clone add");
                let mut encode = |goods: &CGoods| context.encode_goods_for_old_client(goods);
                player.add_goods_to_ci_qing(
                    cloned_hand_goods.expect("успешная ветвь проверила native Clone"),
                    position,
                    false,
                    goods_factory,
                    &mut encode,
                )
            };
            if addition.resulting_amount.is_some() {
                report
                    .deliveries
                    .push(CiQingMountDelivery::ContainerAddition(
                        self.send_ci_qing_container_addition(&addition),
                    ));
            }
            report.rejected_clone = rejected.as_ref().map(CGoods::identity);
            report.addition = Some(addition);
            report.deliveries.push(CiQingMountDelivery::PropertyUpdate(
                self.refresh_ci_qing_player_property(player_id, context),
            ));
        }

        let material_log = CiQingLog {
            player_id,
            delta: -1,
            operation: 3,
            base_index: node.base_index,
            amount,
        };
        report.deliveries.push(CiQingMountDelivery::Audit(
            self.send_ci_qing_log(&material_log),
        ));
        report.logs.push(material_log);
        let consumptions = self
            .players
            .get_mut(&player_id)
            .expect("player проверен до CiQing mount material removal")
            .remove_item_in_packet(node.base_index, amount);
        for consumption in consumptions {
            report
                .deliveries
                .push(CiQingMountDelivery::PacketConsumption(
                    self.send_player_packet_consumption(&consumption),
                ));
            report.material_consumptions.push(consumption);
        }
        let mut message = CMessage::new(0x0c_0111);
        message.add_ulong(u32::from(succeeded));
        report.deliveries.push(CiQingMountDelivery::Player(
            message.send_to_player(self.net_server(), player_id),
        ));
        report.outcome = if succeeded {
            CiQingMountOutcome::Mounted
        } else {
            CiQingMountOutcome::Failed
        };
        Some(report)
    }

    pub(crate) fn query_ci_qing_other_person<Context: CiQingComposeContext>(
        &mut self,
        requester_id: i32,
        target: CiQingOtherPersonTarget,
        context: &mut Context,
    ) -> CiQingOtherPersonReport {
        let target_player_id = match &target {
            CiQingOtherPersonTarget::Id(player_id) => {
                self.find_player(*player_id).map(CPlayer::player_id)
            }
            CiQingOtherPersonTarget::Name(name) => {
                self.find_player_by_name(name).map(CPlayer::player_id)
            }
            CiQingOtherPersonTarget::UnsupportedMode(_) => None,
        };
        let mut report = CiQingOtherPersonReport {
            requester_id,
            target,
            target_player_id,
            payload: Vec::new(),
            delivery: None,
        };
        let Some(target_player_id) = target_player_id else {
            return report;
        };
        let player = self
            .find_player(target_player_id)
            .expect("target ID разрешён через canonical player map");
        let merged = player.ci_qing_property_result();
        let mut payload = Vec::new();
        payload.extend_from_slice(&player.shape().identity().object_type.to_le_bytes());
        payload.extend_from_slice(&player.player_id().to_le_bytes());
        for value in merged.values() {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        payload.extend_from_slice(
            &player
                .ci_qing_goods_amount(&self.goods_factory)
                .to_le_bytes(),
        );
        for position in 0..8 {
            if let Some(goods) = player.ci_qing_goods(position) {
                payload.extend_from_slice(&context.encode_goods_for_old_client(goods));
            }
        }
        payload.extend_from_slice(&player.ci_qing_property_snapshot().2.to_le_bytes());
        let mut message = CMessage::new(0x0c_010f);
        message.base_mut().add(&payload);
        report.delivery = Some(message.send_to_player(self.net_server(), requester_id));
        report.payload = payload;
        report
    }

    fn refresh_ci_qing_player_property<Context: CiQingComposeContext>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> Vec<i32> {
        let snapshot = context.mount_ci_qing_equipment(self, player_id);
        let mut deliveries = snapshot.external_deliveries;
        let add_values = CPlayer::update_ci_qing_property_difference(
            &snapshot.previous_type_values,
            &snapshot.current_type_values,
        );
        let (changed, result_values) = {
            let player = self
                .players
                .get_mut(&player_id)
                .expect("property runtime получает canonical player");
            let previous_add_values = player.ci_qing_property_snapshot().0;
            let changed = add_values
                .as_ref()
                .is_some_and(|add_values| previous_add_values != add_values);
            let stored_add_values =
                add_values.unwrap_or_else(|| player.ci_qing_property_snapshot().0.clone());
            player.apply_ci_qing_property_snapshot(
                stored_add_values,
                snapshot.tao_zhuang_add_values,
                snapshot.tao_zhuang_id,
            );
            (changed, player.ci_qing_property_result())
        };
        if changed {
            let mut message = CMessage::new(0x0c_0110);
            message.add_long(player_id);
            message.add_long(player_id);
            for value in result_values.values() {
                message.add_ulong(*value);
            }
            deliveries.push(message.send_to_player(self.net_server(), player_id));
        }
        deliveries
    }

    pub(crate) const fn ling_bao_setup(&self) -> &CLingBaoSetup {
        &self.ling_bao_setup
    }

    pub(crate) const fn ling_bao_setup_mut(&mut self) -> &mut CLingBaoSetup {
        &mut self.ling_bao_setup
    }

    pub(crate) const fn gods_battle_mgr(&self) -> &CGodsBattleMgr {
        &self.gods_battle_mgr
    }

    pub(crate) const fn gods_battle_mgr_mut(&mut self) -> &mut CGodsBattleMgr {
        &mut self.gods_battle_mgr
    }

    /// Script primitive at call-site `0x4C3D96`: пустой `0x5FA10` сначала
    /// уходит WorldServer-у, затем single pending requester перезаписывается.
    pub(crate) fn request_gods_battle_top_ten(
        &mut self,
        player_id: i32,
    ) -> GodsBattleTopTenRequestReport {
        let request = CMessage::new(0x5fa10);
        let delivery = request.send(self, false);
        self.gods_battle_mgr.record_top_ten_request(player_id);
        GodsBattleTopTenRequestReport {
            player_id,
            delivery,
        }
    }

    /// Script producer at `0x4C3C11/0x4C3C2E`: factions 5/6 преобразуются
    /// в World indices 1/2; local XYD меняется только обратным `0x7F80E`.
    pub(crate) fn request_gods_battle_faction_xyd(
        &self,
        faction: i32,
        xyd: u32,
    ) -> GodsBattleXydRequestReport {
        let world_faction = match faction {
            5 => 1,
            6 => 2,
            _ => {
                return GodsBattleXydRequestReport {
                    faction,
                    xyd,
                    delivery: None,
                };
            }
        };
        let mut request = CMessage::new(0x5fa0f);
        request.add_byte(1);
        request.add_long(world_faction);
        request.add_ulong(xyd);
        GodsBattleXydRequestReport {
            faction,
            xyd,
            delivery: Some(request.send(self, false)),
        }
    }

    /// Точный player-tail после успешного base `CServerRegion::AddObject`.
    pub(crate) fn enter_gods_battle_player<Context: GodsBattlePlayerContext>(
        &mut self,
        region_id: i32,
        player_id: i32,
        context: &mut Context,
    ) -> Option<GodsBattlePlayerRegionReport> {
        let (country, previous_faction) = self
            .find_player(player_id)
            .map(|player| (player.country(), player.gods_battle_faction()))?;
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::GodsBattle(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        let assigned_faction = (previous_faction == 0).then(|| {
            self.gods_battle_mgr
                .faction_for_country(country)
                .unwrap_or(0)
        });
        let faction = assigned_faction.unwrap_or(previous_faction);
        let faction_delivery = if let Some(assigned_faction) = assigned_faction {
            self.find_player_mut(player_id)
                .expect("player сохранён после preflight")
                .set_gods_battle_faction(assigned_faction);
            let message =
                gods_battle_property_message(player_id, b"lGodsBattleFaciton", assigned_faction);
            let player = self
                .find_player(player_id)
                .expect("player сохранён после mutation");
            Some(context.send_gods_battle_player_around(&region.war.base, player.shape(), &message))
        } else {
            None
        };
        let membership_changed = region.add_faction_player(player_id, faction);
        let region_state_delivery = gods_battle_property_message(player_id, b"m_lIsInGodRegion", 1)
            .send_to_player(self.net_server(), player_id);
        self.restore_region_owner(ServerRegionOwner::GodsBattle(region));
        Some(GodsBattlePlayerRegionReport {
            region_id,
            player_id,
            assigned_faction,
            faction_delivery,
            membership_changed,
            region_state_delivery,
        })
    }

    /// Общий player-tail `RemoveObject/DelObj` после spatial/base удаления.
    pub(crate) fn leave_gods_battle_player(
        &mut self,
        region_id: i32,
        player_id: i32,
    ) -> Option<GodsBattlePlayerRegionReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::GodsBattle(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        let membership_changed = region.remove_faction_player(player_id);
        let region_state_delivery = gods_battle_property_message(player_id, b"m_lIsInGodRegion", 0)
            .send_to_player(self.net_server(), player_id);
        self.restore_region_owner(ServerRegionOwner::GodsBattle(region));
        Some(GodsBattlePlayerRegionReport {
            region_id,
            player_id,
            assigned_faction: None,
            faction_delivery: None,
            membership_changed,
            region_state_delivery,
        })
    }

    /// NPC-tail concrete GodsBattle `AddObject`: membership предшествует
    /// guard spawn, затем kill counter сбрасывается без World publication.
    pub(crate) fn enter_gods_battle_npc<Context: GodsBattleNpcContendContext>(
        &mut self,
        region_id: i32,
        npc_id: i32,
        context: &mut Context,
    ) -> Option<GodsBattleNpcFactionReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::GodsBattle(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        let Some((npc_name, configuration)) =
            region.war.base.find_npc_by_id(npc_id).and_then(|npc| {
                let name = npc.name().to_vec();
                self.gods_battle_mgr
                    .npc_configuration(&name)
                    .cloned()
                    .map(|configuration| (name, configuration))
            })
        else {
            self.restore_region_owner(ServerRegionOwner::GodsBattle(region));
            return None;
        };
        let faction = configuration.faction;
        if !region.add_faction_npc(npc_id, faction) {
            self.restore_region_owner(ServerRegionOwner::GodsBattle(region));
            return None;
        }

        let (spawned_monster_ids, spawn_blocks) =
            self.spawn_gods_battle_npc_monsters(&mut region, &configuration, faction, context);
        self.gods_battle_mgr
            .reset_npc_killed_monster_count(&npc_name);
        self.restore_region_owner(ServerRegionOwner::GodsBattle(region));
        Some(GodsBattleNpcFactionReport {
            region_id,
            npc_id,
            npc_name,
            faction,
            spawned_monster_ids,
            spawn_blocks,
            world_delivery: None,
        })
    }

    fn spawn_gods_battle_npc_monsters<Context: GodsBattleNpcContendContext>(
        &mut self,
        region: &mut CServerGodsBattleRegion,
        configuration: &crate::setup::godsbattleconf::GodsBattleFactionNpcName,
        faction: i32,
        context: &mut Context,
    ) -> (Vec<i32>, Vec<GodsBattleMonsterTokenBlock>) {
        let (area_width, area_height) = self.area_dimensions();
        let mut spawned_monster_ids = Vec::new();
        let mut spawn_blocks = Vec::new();
        for token in configuration
            .monsters
            .split(|byte| *byte == b',')
            .filter(|token| !token.is_empty())
        {
            let fields = token
                .split(|byte| *byte == b'|')
                .filter(|field| !field.is_empty())
                .collect::<Vec<_>>();
            if fields.len() != 3 {
                context.record_gods_battle_log(GodsBattleNpcLog::InvalidMonsterToken {
                    token: token.to_vec(),
                });
                spawn_blocks.push(GodsBattleMonsterTokenBlock::FieldCount {
                    token: token.to_vec(),
                    fields: fields.len(),
                });
                break;
            }
            let original_name = fields[0];
            let Some(property) = self
                .find_monster_property_by_origin_name(original_name)
                .cloned()
            else {
                context.record_gods_battle_log(GodsBattleNpcLog::MonsterSpawnFailed {
                    npc_name: configuration.name.clone(),
                    monster: original_name.to_vec(),
                });
                spawn_blocks.push(GodsBattleMonsterTokenBlock::MissingMonsterProperty {
                    original_name: original_name.to_vec(),
                });
                continue;
            };
            let spawn = region.war.base.add_monster(
                &property,
                legacy_atoi_i32(fields[1]),
                legacy_atoi_i32(fields[2]),
                -1,
                true,
                false,
                context.gods_battle_now_milliseconds(),
                area_width,
                area_height,
                context,
            );
            let monster_id = match spawn {
                Ok(monster_id) => monster_id,
                Err(block) => {
                    context.record_gods_battle_log(GodsBattleNpcLog::MonsterSpawnFailed {
                        npc_name: configuration.name.clone(),
                        monster: original_name.to_vec(),
                    });
                    spawn_blocks.push(GodsBattleMonsterTokenBlock::Membership(block));
                    continue;
                }
            };
            if let Some(property) = self.find_monster_property_by_origin_name_mut(original_name) {
                property.race = faction as u32;
            }
            context.record_gods_battle_log(GodsBattleNpcLog::MonsterSpawned {
                npc_name: configuration.name.clone(),
                monster: original_name.to_vec(),
                faction,
            });
            spawned_monster_ids.push(monster_id);
        }
        (spawned_monster_ids, spawn_blocks)
    }

    pub(crate) fn change_gods_battle_npc_faction<Context: GodsBattleNpcContendContext>(
        &mut self,
        region_id: i32,
        npc_id: i32,
        faction: i32,
        context: &mut Context,
    ) -> Option<GodsBattleNpcFactionReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::GodsBattle(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        let Some((npc_name, configuration)) =
            region.war.base.find_npc_by_id(npc_id).and_then(|npc| {
                let name = npc.name().to_vec();
                self.gods_battle_mgr
                    .npc_configuration(&name)
                    .cloned()
                    .map(|configuration| (name, configuration))
            })
        else {
            self.restore_region_owner(ServerRegionOwner::GodsBattle(region));
            return None;
        };
        if !region.change_npc_faction(npc_id, faction) {
            self.restore_region_owner(ServerRegionOwner::GodsBattle(region));
            return None;
        }
        let (spawned_monster_ids, spawn_blocks) =
            self.spawn_gods_battle_npc_monsters(&mut region, &configuration, faction, context);
        self.gods_battle_mgr
            .reset_npc_killed_monster_count(&npc_name);
        let world_delivery = self
            .gods_battle_mgr
            .update_npc_faction(&npc_name, faction)
            .map(|_| {
                let mut message = CMessage::new(0x5fa0f);
                message.add_byte(2);
                add_legacy_c_string(message.base_mut(), &npc_name);
                message.add_ulong(faction as u32);
                context.record_gods_battle_log(GodsBattleNpcLog::FactionUpdated {
                    npc_name: npc_name.clone(),
                    faction,
                });
                message.send(self, false)
            });
        self.restore_region_owner(ServerRegionOwner::GodsBattle(region));
        Some(GodsBattleNpcFactionReport {
            region_id,
            npc_id,
            npc_name,
            faction,
            spawned_monster_ids,
            spawn_blocks,
            world_delivery,
        })
    }

    pub(crate) fn leave_gods_battle_npc(&mut self, region_id: i32, npc_id: i32) -> bool {
        let Some(owner) = self.take_region_owner(region_id) else {
            return false;
        };
        let ServerRegionOwner::GodsBattle(mut region) = owner else {
            self.restore_region_owner(owner);
            return false;
        };
        let removed = region.remove_faction_npc(npc_id);
        self.restore_region_owner(ServerRegionOwner::GodsBattle(region));
        removed
    }

    /// Concrete GodsBattle virtual `GetReturnPoint`: faction-specific entry
    /// wins; a miss delegates to the already materialized base-region rule.
    pub(crate) fn gods_battle_return_point<Context: GodsBattleReturnPointContext>(
        &mut self,
        region_id: i32,
        player_id: i32,
        context: &mut Context,
    ) -> Result<Option<GodsBattleReturnPointReport>, ServerReturnSetupBlock> {
        let Some(player) = self.find_player(player_id) else {
            return Ok(None);
        };
        let faction = player.gods_battle_faction();
        let player_facts = ServerReturnPlayer {
            id: player_id,
            country: player.country(),
            faction_id: player.faction_id(),
        };
        let owner = match self.take_region_owner(region_id) {
            Some(owner) => owner,
            None => return Ok(None),
        };
        let ServerRegionOwner::GodsBattle(region) = owner else {
            self.restore_region_owner(owner);
            return Ok(None);
        };
        if let Some(point) = self.gods_battle_mgr.return_point(region_id, faction) {
            context.record_gods_battle_log(GodsBattleNpcLog::ReturnPointSelected {
                player_id,
                faction,
                point,
            });
            self.restore_region_owner(ServerRegionOwner::GodsBattle(region));
            return Ok(Some(GodsBattleReturnPointReport {
                player_id,
                faction,
                point,
                source: GodsBattleReturnPointSource::GodsBattleConfiguration,
            }));
        }
        let fallback = region
            .war
            .base
            .get_return_point(Some(player_facts), &mut self.country_param);
        self.restore_region_owner(ServerRegionOwner::GodsBattle(region));
        fallback.map(|point| {
            Some(GodsBattleReturnPointReport {
                player_id,
                faction,
                point,
                source: GodsBattleReturnPointSource::BaseRegionFallback,
            })
        })
    }

    /// Full reached `CPlayer::OnRelive` path. Universal skill/state/spatial
    /// owners остаются explicit runtime callbacks; player scalars, GodsBattle
    /// virtual return, random target position и client wires исполняются здесь.
    pub(crate) fn relive_gods_battle_player<Context: PlayerReliveContext>(
        &mut self,
        player_id: i32,
        relive_type: i32,
        context: &mut Context,
    ) -> PlayerReliveReport {
        let Some(player) = self.find_player(player_id) else {
            return PlayerReliveReport {
                player_id,
                relive_type,
                outcome: PlayerReliveOutcome::PlayerMissing,
            };
        };
        if !CMoveShape::is_died(player.health()) {
            let answer_delivery = self.send_player_relive_answer(player_id);
            return PlayerReliveReport {
                player_id,
                relive_type,
                outcome: PlayerReliveOutcome::AlreadyAlive { answer_delivery },
            };
        }
        let mutation = {
            let player = self
                .find_player_mut(player_id)
                .expect("relive player сохранён после synchronous dead-check");
            context.auto_start_player_passive_skills(player);
            context.clear_player_uncreated_summons(player);
            context.player_enter_region_after_relive(player);
            context.update_player_property_after_relive(player);
            context.set_player_moveable(player, true);
            let mutation = player.apply_relive_scalars();
            context.change_player_states_after_relive(player);
            mutation
        };
        let mutation = match mutation {
            Ok(mutation) => mutation,
            Err(block) => {
                return PlayerReliveReport {
                    player_id,
                    relive_type,
                    outcome: PlayerReliveOutcome::PositionBlocked(block),
                };
            }
        };

        if relive_type == 1 {
            {
                let player = self
                    .find_player_mut(player_id)
                    .expect("relive player сохранён до in-place states");
                context.enter_player_resident_state(player);
                context.enter_player_peace_state(player);
            }
            let answer_delivery = self.send_player_relive_answer(player_id);
            let mut state_deliveries = Vec::new();
            let region_id = self
                .find_player(player_id)
                .and_then(CPlayer::server_region_id);
            if self
                .find_player(player_id)
                .is_some_and(|player| player.city_war_died_state_time_ms() > 0)
            {
                state_deliveries
                    .extend(self.publish_relive_died_state(region_id, player_id, context));
            }
            let shape_delivery = region_id
                .and_then(|region_id| self.take_region_owner(region_id))
                .map(|owner| {
                    let mut message = CMessage::new(0xbf613);
                    message.add_long(player_id);
                    message.add_long(player_id);
                    let player = self
                        .find_player(player_id)
                        .expect("relive player сохранён до shape publication");
                    let delivery =
                        context.send_player_relive_around(owner.base(), player.shape(), &message);
                    self.restore_region_owner(owner);
                    delivery
                });
            if let Some(region_id) = region_id {
                let _ = context.change_relived_player_region(
                    player_id,
                    region_id,
                    mutation.previous_x,
                    mutation.previous_y,
                    mutation.direction,
                    3,
                );
                context.restore_relive_origin_block(
                    player_id,
                    mutation.previous_x,
                    mutation.previous_y,
                );
            }
            return PlayerReliveReport {
                player_id,
                relive_type,
                outcome: PlayerReliveOutcome::InPlace {
                    mutation,
                    answer_delivery,
                    shape_delivery,
                    state_deliveries,
                },
            };
        }

        let region_id = self
            .find_player(player_id)
            .and_then(CPlayer::server_region_id);
        let return_point = match region_id {
            Some(region_id) => match self.gods_battle_return_point(region_id, player_id, context) {
                Ok(Some(report)) => report,
                Ok(None) => {
                    return PlayerReliveReport {
                        player_id,
                        relive_type,
                        outcome: PlayerReliveOutcome::CurrentRegionMissing { mutation },
                    };
                }
                Err(block) => {
                    return PlayerReliveReport {
                        player_id,
                        relive_type,
                        outcome: PlayerReliveOutcome::ReturnPointBlocked(block),
                    };
                }
            },
            None => {
                return PlayerReliveReport {
                    player_id,
                    relive_type,
                    outcome: PlayerReliveOutcome::CurrentRegionMissing { mutation },
                };
            }
        };
        let mut x = return_point.point.left.wrapping_add(
            return_point
                .point
                .right
                .wrapping_sub(return_point.point.left)
                / 2,
        );
        let mut y = return_point.point.top.wrapping_add(
            return_point
                .point
                .bottom
                .wrapping_sub(return_point.point.top)
                / 2,
        );
        let width = return_point
            .point
            .right
            .wrapping_sub(return_point.point.left);
        let height = return_point
            .point
            .bottom
            .wrapping_sub(return_point.point.top);
        if width > 0 && height > 0 {
            if let Some(owner) = self.find_region(return_point.point.region_id) {
                match owner.base().region.get_random_pos_in_range(
                    return_point.point.left,
                    return_point.point.top,
                    width,
                    height,
                    context,
                ) {
                    Ok(position) => {
                        x = position.x;
                        y = position.y;
                    }
                    Err(block) => {
                        return PlayerReliveReport {
                            player_id,
                            relive_type,
                            outcome: PlayerReliveOutcome::RandomPositionBlocked(block),
                        };
                    }
                }
            }
        }
        {
            let player = self
                .find_player_mut(player_id)
                .expect("relive player сохранён до destination states");
            context.enter_player_resident_state(player);
            context.enter_player_peace_state(player);
        }
        let changed_region = context.change_relived_player_region(
            player_id,
            return_point.point.region_id,
            x,
            y,
            return_point.point.direction,
            0,
        );
        let answer_delivery = changed_region.then(|| self.send_player_relive_answer(player_id));
        let mut state_deliveries = Vec::new();
        if self
            .find_player(player_id)
            .is_some_and(|player| player.city_war_died_state_time_ms() > 0)
        {
            let destination_region_id = self
                .find_player(player_id)
                .and_then(CPlayer::server_region_id)
                .or(Some(return_point.point.region_id));
            state_deliveries.extend(self.publish_relive_died_state(
                destination_region_id,
                player_id,
                context,
            ));
        }
        PlayerReliveReport {
            player_id,
            relive_type,
            outcome: PlayerReliveOutcome::ReturnPoint {
                mutation,
                return_point,
                x,
                y,
                changed_region,
                answer_delivery,
                state_deliveries,
            },
        }
    }

    fn send_player_relive_answer(&self, player_id: i32) -> i32 {
        let Some(player) = self.find_player(player_id) else {
            return 0;
        };
        let mut message = CMessage::new(0xbf703);
        message
            .base_mut()
            .add_short(player.shape().get_action() as i16);
        message.add_ulong(player.health());
        message.send_to_player(self.net_server(), player_id)
    }

    fn publish_relive_died_state<Context: PlayerReliveContext>(
        &mut self,
        region_id: Option<i32>,
        player_id: i32,
        context: &mut Context,
    ) -> Vec<Result<i32, ShapeCoordinateBlock>> {
        let Some(player) = self.find_player_mut(player_id) else {
            return Vec::new();
        };
        player.set_city_war_died_state(true);
        let mut message = CMessage::new(0xbff2a);
        message.add_long(player_id);
        message.add_byte(1);
        let mut deliveries = vec![Ok(message.send_to_player(self.net_server(), player_id))];
        if let Some(owner) = region_id.and_then(|region_id| self.take_region_owner(region_id)) {
            if let Some(player) = self.find_player(player_id) {
                deliveries.push(context.send_player_relive_around(
                    owner.base(),
                    player.shape(),
                    &message,
                ));
            }
            self.restore_region_owner(owner);
        }
        deliveries
    }

    pub(crate) fn gods_battle_monster_died<Context: GodsBattleNpcContendContext>(
        &mut self,
        region_id: i32,
        monster_id: i32,
        killer_type: i32,
        killer_id: i32,
        context: &mut Context,
    ) -> Option<GodsBattleMonsterDeathReport> {
        let (original_name, display_name) = self
            .find_region(region_id)
            .and_then(|owner| match owner {
                ServerRegionOwner::GodsBattle(region) => {
                    region.war.base.find_monster_by_id(monster_id)
                }
                _ => None,
            })
            .map(|monster| {
                (
                    monster.original_name().to_vec(),
                    monster.display_name().to_vec(),
                )
            })?;
        if self
            .find_monster_property_by_origin_name(&original_name)
            .is_none_or(|property| property.ai != 0x17)
        {
            return None;
        }
        let Some(npc_name) = self
            .gods_battle_mgr
            .npc_name_by_monster(&original_name)
            .map(ToOwned::to_owned)
        else {
            context.record_gods_battle_log(GodsBattleNpcLog::MonsterWithoutNpc {
                monster: display_name,
            });
            return Some(GodsBattleMonsterDeathReport {
                region_id,
                monster_id,
                npc_name: None,
                killed: None,
                total: None,
                player_notice_delivery: None,
            });
        };
        let Some(killed) = self
            .gods_battle_mgr
            .increment_npc_killed_monster_count(&npc_name)
        else {
            context.record_gods_battle_log(GodsBattleNpcLog::MissingKillCounter {
                npc_name: npc_name.clone(),
            });
            return Some(GodsBattleMonsterDeathReport {
                region_id,
                monster_id,
                npc_name: Some(npc_name),
                killed: None,
                total: None,
                player_notice_delivery: None,
            });
        };
        context.record_gods_battle_log(GodsBattleNpcLog::MonsterKilled {
            npc_name: npc_name.clone(),
            monster: display_name.clone(),
            killer_type,
            killer_id,
        });
        let total = self.gods_battle_mgr.npc_monster_count(&npc_name);
        if total == 0 {
            context.record_gods_battle_log(GodsBattleNpcLog::NoConfiguredMonsters {
                npc_name: npc_name.clone(),
            });
            return Some(GodsBattleMonsterDeathReport {
                region_id,
                monster_id,
                npc_name: Some(npc_name),
                killed: Some(killed),
                total: Some(total),
                player_notice_delivery: None,
            });
        }
        let remaining = (total as i32).wrapping_sub(killed as i32);
        let player_notice_delivery =
            if killer_type == PLAYER_TYPE && self.find_player(killer_id).is_some() {
                let text = if remaining > 0 {
                    format_legacy_mixed(
                        self.get_string_by_id(b"SZLGS8"),
                        &[
                            LegacyFormatArgument::Bytes(&display_name),
                            LegacyFormatArgument::Signed(remaining),
                            LegacyFormatArgument::Bytes(&npc_name),
                        ],
                        0xff,
                    )
                } else if remaining == 0 {
                    format_legacy_mixed(
                        self.get_string_by_id(b"SZLGS9"),
                        &[LegacyFormatArgument::Bytes(&npc_name)],
                        0xff,
                    )
                } else {
                    context.record_gods_battle_log(GodsBattleNpcLog::KillCountExceeded {
                        npc_name: npc_name.clone(),
                    });
                    Vec::new()
                };
                (!text.is_empty()).then(|| {
                    colored_player_notice_message(0xffff_ffff, 0, &text)
                        .send_to_player(self.net_server(), killer_id)
                })
            } else {
                None
            };
        Some(GodsBattleMonsterDeathReport {
            region_id,
            monster_id,
            npc_name: Some(npc_name),
            killed: Some(killed),
            total: Some(total),
            player_notice_delivery,
        })
    }

    pub(crate) fn enter_gods_battle_contend<Context: GodsBattleNpcContendContext>(
        &mut self,
        region_id: i32,
        player_id: i32,
        npc_id: i32,
        max_time: i32,
        context: &mut Context,
    ) -> Option<GodsBattleContendEnterReport> {
        let mut deliveries = Vec::new();
        let player_facts = self.find_player(player_id).map(|player| {
            (
                player.can_enter_gods_battle_contend(),
                player.faction_id(),
                player.gods_battle_faction(),
            )
        });
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::GodsBattle(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        let npc_name = region
            .war
            .base
            .find_npc_by_id(npc_id)
            .map(|npc| npc.name().to_vec());
        let outcome = match (player_facts, npc_name) {
            (None, _) => GodsBattleContendEnterOutcome::PlayerMissing,
            (_, None) => GodsBattleContendEnterOutcome::NpcMissing,
            (Some((false, _, _)), Some(_)) => GodsBattleContendEnterOutcome::PlayerUnavailable,
            (Some((_, _, gods_faction)), Some(_)) if !matches!(gods_faction, 5 | 6) => {
                GodsBattleContendEnterOutcome::InvalidPlayerFaction
            }
            (Some((_, _, _)), Some(_)) if region.npc_faction(npc_id).is_none() => {
                GodsBattleContendEnterOutcome::InvalidNpcFaction
            }
            (Some((_, normal_faction, gods_faction)), Some(npc_name)) => {
                let total = self.gods_battle_mgr.npc_monster_count(&npc_name);
                let killed = self.gods_battle_mgr.npc_killed_monster_count(&npc_name);
                if killed != total {
                    let remaining = total.wrapping_sub(killed);
                    let text = format_legacy_mixed(
                        self.get_string_by_id(b"SZLGS6"),
                        &[LegacyFormatArgument::Bytes(&npc_name)],
                        0xff,
                    );
                    deliveries.push(
                        colored_player_notice_message(0xffff_ffff, 0xffff_0000, &text)
                            .send_to_player(self.net_server(), player_id),
                    );
                    GodsBattleContendEnterOutcome::GuardsRemain { remaining }
                } else if region.npc_faction(npc_id) == Some(gods_faction) {
                    let text = format_legacy_mixed(
                        self.get_string_by_id(b"SZLGS7"),
                        &[LegacyFormatArgument::Bytes(&npc_name)],
                        0xff,
                    );
                    deliveries.push(
                        colored_player_notice_message(0xffff_ffff, 0xffff_0000, &text)
                            .send_to_player(self.net_server(), player_id),
                    );
                    GodsBattleContendEnterOutcome::AlreadyOwned
                } else if region.is_player_contending_symbol(player_id, npc_id) {
                    GodsBattleContendEnterOutcome::AlreadyContending
                } else {
                    if matches!(
                        region.cancel_contend_by_player_id(player_id),
                        GodsBattleCancelByPlayer::MissingReset
                    ) {
                        if let Some(delivery) = self.set_gods_battle_player_contend_state(
                            &region.war.base,
                            player_id,
                            false,
                            context,
                        ) {
                            deliveries.push(delivery);
                        }
                        deliveries.push(self.send_gods_battle_contend_time(player_id, 0));
                    }
                    let first_for_legacy_faction = region.add_contender(
                        player_id,
                        normal_faction,
                        gods_faction,
                        npc_id,
                        &npc_name,
                        max_time,
                        context.gods_battle_now_milliseconds(),
                    );
                    if let Some(delivery) = self.set_gods_battle_player_contend_state(
                        &region.war.base,
                        player_id,
                        true,
                        context,
                    ) {
                        deliveries.push(delivery);
                    }
                    deliveries.push(self.send_gods_battle_contend_time(player_id, 0));
                    if first_for_legacy_faction {
                        let faction_text = match gods_faction {
                            5 => self.get_string_by_id(b"SZLGS1"),
                            6 => self.get_string_by_id(b"SZLGS2"),
                            _ => &[],
                        };
                        let text =
                            format_legacy_text_fields(b"%s%s", &[faction_text, &npc_name], 0xff);
                        deliveries.push(
                            nation_colored_text_message(0xbf806, 0xffff_ffff, 0xffff_0000, &text)
                                .send_to_region(Some(&region.war.base), None, self),
                        );
                    }
                    deliveries.push(
                        colored_player_notice_message(
                            0xffff_ffff,
                            0xffff_0000,
                            self.get_string_by_id(b"SZLGS3"),
                        )
                        .send_to_player(self.net_server(), player_id),
                    );
                    GodsBattleContendEnterOutcome::Entered {
                        first_for_legacy_faction,
                    }
                }
            }
        };
        self.restore_region_owner(ServerRegionOwner::GodsBattle(region));
        Some(GodsBattleContendEnterReport {
            region_id,
            player_id,
            npc_id,
            outcome,
            deliveries,
        })
    }

    pub(crate) fn gods_battle_contend_ai<Context: GodsBattleNpcContendContext>(
        &mut self,
        region_id: i32,
        context: &mut Context,
    ) -> Option<GodsBattleContendAiReport> {
        let owner = self.take_region_owner(region_id)?;
        let ServerRegionOwner::GodsBattle(mut region) = owner else {
            self.restore_region_owner(owner);
            return None;
        };
        context.run_gods_battle_base_region_ai(&mut region.war.base);
        let advance = region.advance_contenders(context.gods_battle_now_milliseconds());
        let mut report = GodsBattleContendAiReport {
            region_id,
            ..GodsBattleContendAiReport::default()
        };
        for (player_id, percentage) in advance.progress {
            let delivery = self.send_gods_battle_contend_time(player_id, percentage);
            report
                .progress_deliveries
                .push((player_id, percentage, delivery));
        }
        self.restore_region_owner(ServerRegionOwner::GodsBattle(region));
        for contender in advance.completed {
            report
                .completions
                .push(self.complete_gods_battle_contend(region_id, contender, context));
        }
        Some(report)
    }

    fn complete_gods_battle_contend<Context: GodsBattleNpcContendContext>(
        &mut self,
        region_id: i32,
        contender: GodsBattleContender,
        context: &mut Context,
    ) -> GodsBattleContendCompletionReport {
        let Some(faction) = self
            .find_player(contender.player_id)
            .map(CPlayer::gods_battle_faction)
        else {
            return GodsBattleContendCompletionReport {
                contender,
                outcome: GodsBattleContendCompletionOutcome::PlayerMissing,
                deliveries: Vec::new(),
            };
        };
        let changed =
            self.change_gods_battle_npc_faction(region_id, contender.symbol_id, faction, context);
        let mut deliveries =
            self.cancel_gods_battle_contend_symbol(region_id, contender.symbol_id, context);
        if changed.is_none() {
            return GodsBattleContendCompletionReport {
                contender,
                outcome: GodsBattleContendCompletionOutcome::FactionChangeRejected,
                deliveries,
            };
        }
        let capture_text = match faction {
            5 => self.get_string_by_id(b"SZLGS4").to_vec(),
            6 => self.get_string_by_id(b"SZLGS5").to_vec(),
            _ => Vec::new(),
        };
        if let Some(owner) = self.take_region_owner(region_id) {
            if let ServerRegionOwner::GodsBattle(region) = &owner {
                deliveries.push(
                    nation_colored_text_message(0xbf806, 0xffff_ffff, 0xffff_0000, &capture_text)
                        .send_to_region(Some(&region.war.base), None, self),
                );
            }
            self.restore_region_owner(owner);
        }
        let award_variable_result = self
            .find_player_mut(contender.player_id)
            .map(|player| {
                match player.set_string_variable(b"#fengyinNpc", &contender.symbol_name) {
                    GameVariableMutationOutcome::UpdatedString { .. } => 1,
                    GameVariableMutationOutcome::NameNotFound => -99_999_999,
                    GameVariableMutationOutcome::UpdatedInteger { .. }
                    | GameVariableMutationOutcome::UpdatedArrayElement { .. }
                    | GameVariableMutationOutcome::TypeMismatch { .. } => 0,
                }
            })
            .unwrap_or(-99_999_999);
        let award_script_result = if award_variable_result == 1 {
            let result = self
                .run_script_file(
                    b"scripts/npc/awardgoods.script",
                    ScriptExecutionContext {
                        player_id: Some(contender.player_id),
                        region_id: Some(region_id),
                        ..ScriptExecutionContext::default()
                    },
                    context,
                )
                .map_or(0, |_| 1);
            if result == 0 {
                context.record_gods_battle_log(GodsBattleNpcLog::AwardScriptFailed {
                    player_id: contender.player_id,
                    npc_name: contender.symbol_name.clone(),
                });
            }
            Some(result)
        } else {
            if award_variable_result == -99_999_999 {
                context.record_gods_battle_log(GodsBattleNpcLog::AwardVariableMissing {
                    player_id: contender.player_id,
                    npc_name: contender.symbol_name.clone(),
                });
            }
            None
        };
        let top_info_delivery = self.send_top_info_to_client(-1, 0, 1, 1, &capture_text);
        context.record_gods_battle_log(GodsBattleNpcLog::NpcCaptured {
            player_id: contender.player_id,
            npc_name: contender.symbol_name.clone(),
            faction,
        });
        GodsBattleContendCompletionReport {
            contender,
            outcome: GodsBattleContendCompletionOutcome::Captured {
                faction,
                award_variable_result,
                award_script_result,
                top_info_delivery,
            },
            deliveries,
        }
    }

    fn cancel_gods_battle_contend_symbol<Context: GodsBattleNpcContendContext>(
        &mut self,
        region_id: i32,
        symbol_id: i32,
        context: &mut Context,
    ) -> Vec<i32> {
        let Some(owner) = self.take_region_owner(region_id) else {
            return Vec::new();
        };
        let ServerRegionOwner::GodsBattle(mut region) = owner else {
            self.restore_region_owner(owner);
            return Vec::new();
        };
        let mut deliveries = Vec::new();
        if let Some(contender) = region.cancel_contend_by_symbol(symbol_id) {
            deliveries.push(self.send_gods_battle_contend_time(contender.player_id, 0));
            if let Some(delivery) = self.set_gods_battle_player_contend_state(
                &region.war.base,
                contender.player_id,
                false,
                context,
            ) {
                deliveries.push(delivery);
            }
        }
        self.restore_region_owner(ServerRegionOwner::GodsBattle(region));
        deliveries
    }

    fn set_gods_battle_player_contend_state<Context: GodsBattleNpcContendContext>(
        &mut self,
        region: &CServerRegion,
        player_id: i32,
        state: bool,
        context: &mut Context,
    ) -> Option<i32> {
        let player = self.find_player_mut(player_id)?;
        if !player.set_contend_state(state) {
            return None;
        }
        let mut message = CMessage::new(0xbff28);
        message.add_long(player_id);
        message.add_byte(u8::from(state));
        let player = self
            .find_player(player_id)
            .expect("GodsBattle player сохранён до synchronous around-send");
        context
            .send_gods_battle_player_around(region, player.shape(), &message)
            .ok()
    }

    fn send_gods_battle_contend_time(&self, player_id: i32, percentage: i32) -> i32 {
        let mut message = CMessage::new(0xbff29);
        message.add_long(percentage);
        message.send_to_player(self.net_server(), player_id)
    }

    pub(crate) fn send_top_info_to_client(
        &self,
        first: i32,
        target_player_id: i32,
        third: i32,
        fourth: i32,
        text: &[u8],
    ) -> Result<i32, SendMessageError> {
        let mut message = CMessage::new(0xbf804);
        message.add_long(target_player_id);
        message.add_long(first);
        message.add_long(third);
        message.add_long(fourth);
        add_legacy_c_string(message.base_mut(), text);
        if target_player_id == 0 {
            message.send_all(Some(self.net_server()))
        } else {
            Ok(message.send_to_player(self.net_server(), target_player_id))
        }
    }

    pub(crate) fn apply_gods_battle_xyd(
        &mut self,
        faction_a: u32,
        faction_b: u32,
    ) -> GodsBattleXydApplyReport {
        let updates = self.gods_battle_mgr.set_xyd(faction_a, faction_b);
        let region_ids = self.gods_battle_mgr.region_ids();
        let mut deliveries = Vec::new();
        for update in updates {
            let (faction, xyd, changed) = match update {
                GodsBattleFactionXydUpdate::FactionA { previous, current } => {
                    (5, current, previous != current)
                }
                GodsBattleFactionXydUpdate::FactionB { previous, current } => {
                    (6, current, previous != current)
                }
                GodsBattleFactionXydUpdate::IgnoredFaction { .. } => unreachable!(),
            };
            if !changed {
                continue;
            }
            for region_id in &region_ids {
                let Some(ServerRegionOwner::GodsBattle(region)) = self.find_region(*region_id)
                else {
                    continue;
                };
                let Some(player_ids) = region.faction_player_ids(faction) else {
                    continue;
                };
                for player_id in player_ids {
                    if self.find_player(player_id).is_none() {
                        continue;
                    }
                    let delivery = gods_battle_property_message(player_id, b"dwXYD", xyd as i32)
                        .send_to_player(self.net_server(), player_id);
                    deliveries.push((*region_id, player_id, delivery));
                }
            }
        }
        GodsBattleXydApplyReport {
            updates,
            deliveries,
        }
    }

    /// Достигнутый GodsBattle-tail `CPlayer::OnDied` после общих death effects.
    pub(crate) fn apply_gods_battle_death_szl<
        Context: GodsBattleDeathContext + RealmAppellationScriptContext,
    >(
        &mut self,
        killer_id: i32,
        victim_id: i32,
        context: &mut Context,
    ) -> Option<GodsBattleDeathSzlReport> {
        self.change_body_after_player_death(victim_id, context);
        let (killer_level, killer_szl, killer_faction, killer_team, killer_region) =
            self.find_player(killer_id).map(|player| {
                (
                    player.level(),
                    player.szl(),
                    player.gods_battle_faction(),
                    player.team_id(),
                    player.server_region_id(),
                )
            })?;
        let (victim_level, victim_szl, victim_faction, victim_region) =
            self.find_player(victim_id).map(|player| {
                (
                    player.level(),
                    player.szl(),
                    player.gods_battle_faction(),
                    player.server_region_id(),
                )
            })?;
        let victim_region = victim_region?;
        let killer_region = killer_region?;
        if !self.gods_battle_mgr.contains_region(victim_region) || killer_faction == victim_faction
        {
            return None;
        }

        let gain = if victim_szl == 0 {
            GodsBattleSzlCalculation::default()
        } else {
            self.gods_battle_mgr.calculate_szl_gain(
                killer_level,
                killer_szl,
                victim_level,
                victim_szl,
            )
        };
        let loss = self.gods_battle_mgr.calculate_szl_loss(
            killer_level,
            killer_szl,
            victim_level,
            victim_szl,
        );
        let team = (killer_team != 0)
            .then(|| context.gods_battle_team_snapshot(killer_team))
            .flatten();
        let mut updates = Vec::new();
        if let Some(team) = &team {
            if team.teammate_amount != 0 {
                let share = gods_battle_team_szl_share(gain.value, team.teammate_amount);
                if share != 0 {
                    for player_id in &team.player_ids {
                        let eligible = self.find_player(*player_id).is_some_and(|player| {
                            player.gods_battle_faction() != victim_faction
                                && player.server_region_id() == Some(killer_region)
                        });
                        if eligible {
                            let current = self
                                .find_player(*player_id)
                                .expect("проверенный GodsBattle teammate")
                                .szl();
                            updates.push(self.update_gods_battle_player_szl(
                                *player_id,
                                current.wrapping_add(share),
                                context,
                            ));
                        }
                    }
                }
            }
        } else if gain.value != 0 {
            updates.push(self.update_gods_battle_player_szl(
                killer_id,
                killer_szl.wrapping_add(gain.value),
                context,
            ));
        }

        if victim_szl < loss.value {
            if victim_szl != 0 {
                updates.push(self.update_gods_battle_player_szl(victim_id, 0, context));
            }
        } else {
            updates.push(self.update_gods_battle_player_szl(
                victim_id,
                victim_szl.wrapping_sub(loss.value),
                context,
            ));
        }

        let region_notice_delivery = (|| {
            let template_id = match (killer_faction, victim_faction) {
                (5, 6) => b"SZLGS11".as_slice(),
                (6, 5) => b"SZLGS12".as_slice(),
                _ => return None,
            };
            let text = legacy_c_string_prefix(self.get_string_by_id(template_id));
            let region = self.find_region(killer_region)?;
            let mut message = CMessage::new(0xbf816);
            message.add_byte(2);
            add_legacy_c_string(message.base_mut(), &text[..text.len().min(0xff)]);
            Some(message.send_to_region(Some(region.base()), None, self))
        })();
        Some(GodsBattleDeathSzlReport {
            killer_id,
            victim_id,
            gain,
            loss,
            team,
            updates,
            region_notice_delivery,
        })
    }

    fn update_gods_battle_player_szl<Context: GodsBattleDeathContext>(
        &mut self,
        player_id: i32,
        current: u32,
        context: &mut Context,
    ) -> GodsBattleSzlPlayerUpdate {
        let (previous, attempt_appellation_id) = {
            let player = self
                .find_player_mut(player_id)
                .expect("SZL update получает canonical player");
            let previous = player.szl();
            player.set_szl(current);
            (previous, player.attempt_appellation_id())
        };
        let property_delivery = gods_battle_property_message(player_id, b"dwSZL", current as i32)
            .send_to_player(self.net_server(), player_id);
        let text = format_single_legacy_u32(self.get_string_by_id(b"SZLGS13"), current, 0xff);
        let notice_delivery = colored_player_notice_message(0xffff_ffff, 0xffa8_069c, &text)
            .send_to_player(self.net_server(), player_id);

        let removed_attempt_appellation = if current < previous
            && (40_001..50_000).contains(&attempt_appellation_id)
            && self.gods_battle_mgr.szl_level(current).unwrap_or(0)
                < attempt_appellation_id % 40_000
        {
            self.find_player_mut(player_id)
                .expect("player сохранён после SZL publication")
                .clear_attempt_appellation();
            context.request_gods_battle_change_appellation(player_id, 0);
            Some(attempt_appellation_id)
        } else {
            None
        };
        let appellation_notice_delivery = removed_attempt_appellation.map(|_| {
            colored_player_notice_message(0xffff_ffff, 0, self.get_string_by_id(b"SZLGS10"))
                .send_to_player(self.net_server(), player_id)
        });
        GodsBattleSzlPlayerUpdate {
            player_id,
            previous,
            current,
            property_delivery,
            notice_delivery,
            removed_attempt_appellation,
            appellation_notice_delivery,
        }
    }

    /// Script scalar `11128 / ChangePlayerSZL`: вычисляется только первый
    /// аргумент; negative и parser sentinel являются успешным no-op.
    pub(crate) fn script_change_player_szl<Context: GodsBattleDeathContext>(
        &mut self,
        player_id: i32,
        value: i32,
        context: &mut Context,
    ) -> Option<GodsBattleSzlPlayerUpdate> {
        if value < 0 || value == SCRIPT_SCALAR_ERROR || self.find_player(player_id).is_none() {
            return None;
        }
        Some(self.update_gods_battle_player_szl(player_id, value as u32, context))
    }

    pub(crate) const fn synthesis_mut(&mut self) -> &mut CSynthesis {
        &mut self.synthesis
    }

    pub(crate) const fn synthesis(&self) -> &CSynthesis {
        &self.synthesis
    }

    pub(crate) fn query_player_equipment<Context: PlayerEquipmentInspectionContext>(
        &self,
        requester_id: i32,
        target_id: i32,
        context: &mut Context,
    ) -> PlayerEquipmentInspectionReport {
        let mut report = PlayerEquipmentInspectionReport {
            requester_id,
            target_id,
            outcome: PlayerEquipmentInspectionOutcome::MissingTarget,
            head_picture: None,
            face_picture: None,
            entries: Vec::new(),
            delivery: None,
        };
        let Some(target) = self.find_player(target_id) else {
            return report;
        };
        let (head_picture, face_picture, mode) = target.appearance_and_mode();
        report.head_picture = Some(head_picture);
        report.face_picture = Some(face_picture);
        if mode != 0 {
            report.outcome = PlayerEquipmentInspectionOutcome::ModeBlocked;
            report.delivery = Some(
                colored_player_notice_message(0xffff_0000, 0, self.get_string_by_id(b"GSN0336"))
                    .send_to_player(self.net_server(), requester_id),
            );
            return report;
        }

        let equipment_count = target.equipment().goods_amount(&self.goods_factory) as u8;
        for slot in 0..EQUIPMENT_COLUMN_LIMIT {
            let Some(goods) = target.equipment().get_goods(slot) else {
                continue;
            };
            report.entries.push(PlayerEquipmentInspectionEntry {
                slot: slot as u8,
                goods: goods.identity(),
                old_client_payload: context.encode_goods_for_old_client(goods),
            });
        }
        let mut response = CMessage::new(0x0b_f911);
        response.add_long(target_id);
        response.add_long(head_picture);
        response.add_long(face_picture);
        response.add_byte(equipment_count);
        for entry in &report.entries {
            response.add_byte(entry.slot);
            response.base_mut().add(&entry.old_client_payload);
        }
        report.delivery = Some(response.send_to_player(self.net_server(), requester_id));
        report.outcome = PlayerEquipmentInspectionOutcome::Sent;
        report
    }

    pub(crate) fn handle_container_script_action<Runtime: ScriptFunctionRuntime>(
        &mut self,
        player_id: i32,
        region_id: Option<i32>,
        action: i8,
        runtime: &mut Runtime,
    ) -> Option<ContainerScriptActionReport> {
        if action == 0 {
            let shadows = self
                .find_player_mut(player_id)?
                .clear_all_enhancement_selection();
            return Some(ContainerScriptActionReport {
                action: Some(action),
                script_name: Vec::new(),
                outcome: ContainerScriptActionOutcome::SelectionCleared { shadows },
            });
        }
        if action != 1 {
            return Some(ContainerScriptActionReport {
                action: Some(action),
                script_name: Vec::new(),
                outcome: ContainerScriptActionOutcome::InvalidAction,
            });
        }
        self.run_last_container_script(player_id, region_id, Some(action), runtime)
    }

    pub(crate) fn run_precious_box_item_script<Runtime: ScriptFunctionRuntime>(
        &mut self,
        player_id: i32,
        region_id: Option<i32>,
        runtime: &mut Runtime,
    ) -> Option<ContainerScriptActionReport> {
        self.run_last_container_script(player_id, region_id, None, runtime)
    }

    pub(crate) fn open_precious_box(&mut self, player_id: i32, script: &[u8]) -> i32 {
        if self.find_player(player_id).is_none() {
            return 0;
        }
        let mut message = CMessage::new(0x000b_f91a);
        message.add_long(player_id);
        let _ = message.send_to_player(self.net_server(), player_id);
        self.find_player_mut(player_id)
            .expect("PreciousBox player сохраняется после synchronous send")
            .set_last_container_script(script);
        1
    }

    pub(crate) fn close_precious_box(&self, player_id: i32) {
        if self.find_player(player_id).is_none() {
            return;
        }
        let mut message = CMessage::new(0x000b_f91c);
        message.add_long(player_id);
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    fn roll_precious_box_item(&mut self, box_id: i32) -> Option<PreciousBoxItem> {
        let (configuration, random_state) = (&self.precious_box_conf, &mut self.random_state);
        configuration.random_item(box_id, |upper_bound| {
            game_legacy_random(random_state, upper_bound)
        })
    }

    pub(crate) fn get_precious_box_item<Context: OldClientGoodsCodec>(
        &mut self,
        player_id: i32,
        box_id: i32,
        context: &mut Context,
    ) -> i32 {
        if self.find_player(player_id).is_none() {
            return -1;
        }
        let Some(item) = self.roll_precious_box_item(box_id) else {
            self.send_precious_box_result(player_id, -1, 0);
            return -1;
        };
        if item.item_idx <= 0 || item.amount <= 0 {
            self.send_precious_box_result(player_id, -1, 0);
            return -1;
        }
        let mut created = self.create_goods_batch(item.item_idx as u32, item.amount as u32);
        if item.min_level != 0 {
            let (factory, random_state) = (&self.goods_factory, &mut self.random_state);
            for goods in &mut created {
                let _ = factory.upgrade_equipment(goods, item.min_level, |upper_bound| {
                    game_legacy_random(random_state, upper_bound)
                });
            }
        }
        let (additions, _rejected) = {
            let (players, factory) = (&mut self.players, &self.goods_factory);
            let player = players
                .get_mut(&player_id)
                .expect("PreciousBox player проверен до packet add");
            let mut encode = |goods: &CGoods| context.encode_goods_for_old_client(goods);
            player.add_precious_box_goods_to_packet(created, factory, &mut encode)
        };
        let mut result_position = -1;
        for addition in additions {
            if addition.resulting_amount.is_some() {
                result_position = addition.position.map_or(-1, |position| position as i32);
                let _ = self.send_player_packet_addition(&addition);
            }
        }
        self.send_precious_box_result(player_id, result_position, item.amount);
        if result_position != -1 && item.broadcast {
            if let Some(goods) = u32::try_from(result_position).ok().and_then(|position| {
                self.find_player(player_id)
                    .and_then(|player| player.packet().get_goods(position))
            }) {
                let player_name = self
                    .find_player(player_id)
                    .map(CPlayer::player_name)
                    .unwrap_or_default();
                let template = self.get_string_by_id(if item.min_level == 0 {
                    b"GS0185"
                } else {
                    b"GS0184"
                });
                let arguments = if item.min_level == 0 {
                    vec![
                        LegacyFormatArgument::Bytes(player_name),
                        LegacyFormatArgument::Bytes(goods.name()),
                        LegacyFormatArgument::Signed(item.amount),
                    ]
                } else {
                    vec![
                        LegacyFormatArgument::Bytes(player_name),
                        LegacyFormatArgument::Bytes(goods.name()),
                        LegacyFormatArgument::Signed(item.min_level),
                        LegacyFormatArgument::Signed(item.amount),
                    ]
                };
                let text = format_legacy_mixed(template, &arguments, 1023);
                let mut message = CMessage::new(0x0005_ff0e);
                message.add_long(player_id);
                add_legacy_c_string(message.base_mut(), &text);
                message.add_ulong(0xffff_ff00);
                message.add_ulong(0xffff_0000);
                let _ = message.send(self, false);
            }
        }
        result_position
    }

    fn send_precious_box_result(&self, player_id: i32, position: i32, amount: i32) {
        let mut message = CMessage::new(0x000b_f91b);
        message.add_ulong(position as u32);
        if position != -1 {
            message.add_ulong(amount as u32);
        }
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    fn run_last_container_script<Runtime: ScriptFunctionRuntime>(
        &mut self,
        player_id: i32,
        region_id: Option<i32>,
        action: Option<i8>,
        runtime: &mut Runtime,
    ) -> Option<ContainerScriptActionReport> {
        let script_name = self
            .find_player(player_id)?
            .last_container_script()
            .to_vec();
        if script_name.is_empty() {
            return Some(ContainerScriptActionReport {
                action,
                script_name,
                outcome: ContainerScriptActionOutcome::EmptyScript,
            });
        }
        let script_data_found = self.script_file_data(&script_name).is_some();
        let _ = self.run_script_file(
            &script_name,
            ScriptExecutionContext {
                player_id: Some(player_id),
                region_id,
                ..ScriptExecutionContext::default()
            },
            runtime,
        );
        Some(ContainerScriptActionReport {
            action,
            script_name,
            outcome: ContainerScriptActionOutcome::Dispatched { script_data_found },
        })
    }

    fn send_hotkey_hand_transfer(
        &self,
        player: &CPlayer,
        transfer: &HotkeyHandTransferReport,
    ) -> Vec<i32> {
        use crate::gameserver::appserver::container::cwallet::CurrencyGoodsAddOutcome;

        let (Some(goods), Some(removal)) = (transfer.goods, transfer.hand_removal.as_ref()) else {
            return Vec::new();
        };
        let mut message = CS2CContainerObjectMove::default();
        message.set_source_container(removal.owner_type, removal.owner_id, 0);
        message.set_source_container_extend_id(3);
        message.set_source_object(goods.object_type, goods.ex_id, removal.amount);
        match transfer.outcome {
            HotkeyHandTransferOutcome::Moved => {
                let destination = match transfer.source_container_extend_id {
                    1 => transfer
                        .packet_adds
                        .iter()
                        .rev()
                        .find_map(|outcome| match outcome {
                            VolumeGoodsAddOutcome::Added(added) => {
                                let position = added.position?;
                                let stored = player.packet().base().find(added.identity.ex_id)?;
                                Some((position, added.identity, stored.amount()))
                            }
                            VolumeGoodsAddOutcome::Stack(GoodsStackMergeOutcome::Merged {
                                target,
                                ..
                            }) => {
                                let position =
                                    player.packet().query_goods_position(target.ex_id)?;
                                let stored = player.packet().base().find(target.ex_id)?;
                                Some((position, *target, stored.amount()))
                            }
                            _ => None,
                        }),
                    3 => transfer
                        .hand_rollback
                        .as_ref()
                        .and_then(|added| Some((added.position?, added.identity, added.amount))),
                    4 | 5 => {
                        transfer
                            .currency_adds
                            .iter()
                            .rev()
                            .find_map(|outcome| match outcome {
                                CurrencyGoodsAddOutcome::Added(added) => {
                                    Some((added.position, added.identity, added.amount))
                                }
                                CurrencyGoodsAddOutcome::Stack(
                                    GoodsStackMergeOutcome::Merged { target, .. },
                                ) => Some((
                                    0,
                                    *target,
                                    if transfer.source_container_extend_id == 4 {
                                        player.money()
                                    } else {
                                        player.yuan_bao()
                                    },
                                )),
                                _ => None,
                            })
                    }
                    _ => None,
                };
                let Some((position, identity, amount)) = destination else {
                    return Vec::new();
                };
                message.set_operation(ContainerObjectMoveOperation::MoveObject);
                message.set_destination_container(PLAYER_TYPE, player.player_id(), position);
                message.set_destination_container_extend_id(
                    transfer.source_container_extend_id as i32,
                );
                message.set_destination_object(identity.object_type, identity.ex_id);
                message.set_destination_object_amount(amount);
            }
            HotkeyHandTransferOutcome::RolledBack => {
                message.set_operation(ContainerObjectMoveOperation::RollBack);
            }
            HotkeyHandTransferOutcome::GarbageCollected => {
                message.set_operation(ContainerObjectMoveOperation::DeleteObject);
            }
            HotkeyHandTransferOutcome::MissingHandGoods
            | HotkeyHandTransferOutcome::NotConsumable
            | HotkeyHandTransferOutcome::UnsupportedSource => return Vec::new(),
        }
        vec![message.send_to_player(self, player.player_id())]
    }

    pub(crate) fn assign_hotkey(
        &mut self,
        player_id: i32,
        slot: u8,
        value: u32,
    ) -> Option<HotkeyAssignmentReport> {
        let mut report = HotkeyAssignmentReport {
            slot,
            value,
            outcome: HotkeyAssignmentOutcome::InvalidSlot,
            transfer: None,
            transfer_deliveries: Vec::new(),
            response_deliveries: Vec::new(),
        };
        if usize::from(slot) >= 24 {
            // Старый negative-value path писал за `dwHotKey[24]`; это UB без
            // подтверждённого wire-эффекта, поэтому malformed slot получает
            // тот же безопасный reject, что и обычная out-of-range ветвь.
            report
                .response_deliveries
                .push(send_hotkey_response(self, player_id, 0x0b_f908, b'.', None));
            return Some(report);
        }
        if (value as i32) < 0 {
            self.find_player_mut(player_id)?.set_hotkey(slot, value);
            report.outcome = HotkeyAssignmentOutcome::Assigned;
            report.response_deliveries.push(send_hotkey_response(
                self,
                player_id,
                0x0b_f908,
                b'-',
                Some((slot, Some(value))),
            ));
            return Some(report);
        }
        let hand_missing = self.find_player(player_id)?.ci_qing_hand_goods().is_none();
        if hand_missing {
            self.find_player_mut(player_id)?.set_hotkey(slot, value);
            report.outcome = HotkeyAssignmentOutcome::MissingHandAssigned;
            report.response_deliveries.push(send_hotkey_response(
                self,
                player_id,
                0x0b_f908,
                b'-',
                Some((slot, Some(value))),
            ));
            return Some(report);
        }

        let transfer = {
            let (players, factory) = (&mut self.players, &self.goods_factory);
            players
                .get_mut(&player_id)?
                .return_hotkey_hand_goods(factory)
        };
        if transfer.outcome == HotkeyHandTransferOutcome::Moved {
            self.find_player_mut(player_id)?.set_hotkey(slot, value);
            report.outcome = HotkeyAssignmentOutcome::Assigned;
            report.response_deliveries.push(send_hotkey_response(
                self,
                player_id,
                0x0b_f908,
                b'-',
                Some((slot, Some(value))),
            ));
        } else {
            report.outcome = HotkeyAssignmentOutcome::HandMoveFailed;
        }
        if transfer.hand_removal.is_some() || transfer.outcome == HotkeyHandTransferOutcome::Moved {
            report.transfer_deliveries = self
                .find_player(player_id)
                .map(|player| self.send_hotkey_hand_transfer(player, &transfer))
                .unwrap_or_default();
        }
        if transfer.hand_removal.is_none() {
            report
                .response_deliveries
                .push(send_hotkey_response(self, player_id, 0x0b_f908, b'.', None));
        }
        report.transfer = Some(transfer);
        Some(report)
    }

    pub(crate) fn remove_hotkey(
        &mut self,
        player_id: i32,
        slot: u8,
    ) -> Option<HotkeyRemovalReport> {
        let removed = self
            .find_player(player_id)?
            .hotkey(slot)
            .is_some_and(|value| value != 0);
        if removed {
            self.find_player_mut(player_id)?.set_hotkey(slot, 0);
        }
        let outcome = if removed {
            HotkeyRemovalOutcome::Removed
        } else {
            HotkeyRemovalOutcome::InvalidOrEmpty
        };
        let delivery = send_hotkey_response(
            self,
            player_id,
            0x0b_f909,
            if removed { b'/' } else { b'0' },
            removed.then_some((slot, None)),
        );
        Some(HotkeyRemovalReport {
            slot,
            outcome,
            delivery,
        })
    }

    pub(crate) fn change_hotkey(
        &mut self,
        player_id: i32,
        slot: u8,
        value: u32,
    ) -> Option<HotkeyChangeReport> {
        let changed = self
            .find_player(player_id)?
            .hotkey(slot)
            .is_some_and(|current| current != 0);
        let delivery = if changed {
            self.find_player_mut(player_id)?.set_hotkey(slot, value);
            Some(send_hotkey_response(
                self,
                player_id,
                0x0b_f90a,
                b'1',
                Some((slot, Some(value))),
            ))
        } else {
            None
        };
        Some(HotkeyChangeReport {
            slot,
            value,
            outcome: if changed {
                HotkeyChangeOutcome::Changed
            } else {
                HotkeyChangeOutcome::InvalidOrEmpty
            },
            delivery,
        })
    }

    fn send_fairy_state_effect(&self, effect: &FairyStateChangeEffect) -> Vec<i32> {
        let FairyStateChangeEffect::ObjectMove(object_move) = effect else {
            return Vec::new();
        };
        let Some(position) = object_move.position else {
            return Vec::new();
        };
        let mut message = CS2CContainerObjectMove::default();
        match object_move.operation {
            FairyContainerMoveOperation::DeleteObject => {
                message.set_operation(ContainerObjectMoveOperation::DeleteObject);
                message.set_source_container(
                    object_move.owner_type,
                    object_move.owner_id,
                    position,
                );
                message.set_source_container_extend_id(object_move.container_extend_id as i32);
                message.set_source_object(
                    object_move.goods.object_type,
                    object_move.goods.ex_id,
                    object_move.amount,
                );
            }
            FairyContainerMoveOperation::NewObject => {
                message.set_operation(ContainerObjectMoveOperation::NewObject);
                message.set_destination_container(
                    object_move.owner_type,
                    object_move.owner_id,
                    position,
                );
                message.set_destination_container_extend_id(object_move.container_extend_id as i32);
                message
                    .set_destination_object(object_move.goods.object_type, object_move.goods.ex_id);
                message.set_object_stream(object_move.old_client_payload.clone());
            }
        }
        vec![message.send_to_player(self, object_move.owner_id)]
    }

    fn send_fairy_amount_change(&self, change: &FairyContainerAmountChange) -> Vec<i32> {
        let mut message = CS2CContainerObjectAmountChange::default();
        message.set_source_container(change.owner_type, change.owner_id, change.position);
        message.set_source_container_extend_id(change.container_extend_id as i32);
        message.set_object(change.goods.object_type, change.goods.ex_id);
        message.set_object_amount(change.amount);
        vec![message.send_to_player(self, change.owner_id)]
    }

    fn send_fairy_player_update(&self, update: &FairySyncretizePlayerUpdate) -> Vec<i32> {
        let mut message = CMessage::new(0x0b_f704);
        message.add_ulong(update.experience);
        message.add_ulong(update.vigour);
        vec![message.send_to_player(self.net_server(), update.player_id)]
    }

    fn send_fairy_grow_log(&self, log: &FairyGrowLog) -> Vec<i32> {
        let mut message = CMessage::new(0x0006_0210);
        message.add_long(0);
        message.add_long(log.player_id);
        message.add_long(log.log_value);
        add_legacy_c_string(message.base_mut(), &log.fairy_guid);
        add_legacy_c_string(message.base_mut(), &log.fairy_name);
        message.add_ulong(log.level);
        message.send(self, false).into_iter().collect()
    }

    fn send_fairy_incubate_log(&self, log: &FairyIncubateLog) -> Vec<i32> {
        let mut message = CMessage::new(log.message_type as i32);
        message.add_long(log.log_type);
        message.add_long(log.player_id);
        message.base_mut().add_guid(log.goods.ex_id);
        add_legacy_c_string(message.base_mut(), &log.goods_name);
        message.send(self, false).into_iter().collect()
    }

    fn send_fairy_implantation_log(&self, log: &FairyImplantationLog) -> Vec<i32> {
        let mut message = CMessage::new(0x0006_0210);
        message.add_long(2);
        message.add_long(log.player_id);
        add_legacy_c_string(message.base_mut(), &log.goods_name);
        message.base_mut().add_guid(log.goods.ex_id);
        message.add_ulong(log.old_level);
        message.add_ulong(log.resulting_level);
        message.add_ulong(log.crystal_amount);
        message.send(self, false).into_iter().collect()
    }

    fn send_fairy_syncretize_log(&self, log: &FairySyncretizeLog) -> Vec<i32> {
        let mut message = CMessage::new(log.message_type as i32);
        message.add_long(log.log_type);
        message.add_long(log.player_id);
        message.base_mut().add_guid(log.primary_guid);
        add_legacy_c_string(message.base_mut(), &log.primary_name);
        message.add_ulong(log.primary_level);
        message.add_ulong(log.primary_growing_rate);
        message.base_mut().add_guid(log.secondary_guid);
        add_legacy_c_string(message.base_mut(), &log.secondary_name);
        message.add_ulong(log.secondary_level);
        message.add_ulong(log.secondary_growing_rate);
        message.add_ulong(log.needed_goods);
        message.base_mut().add_guid(log.result_guid);
        add_legacy_c_string(message.base_mut(), &log.result_name);
        message.add_ulong(log.main_ability);
        message.add_ulong(log.combinated_times);
        message.add_ulong(log.growing_rate);
        message.send(self, false).into_iter().collect()
    }

    pub(crate) fn update_fairy_hatch_state<Context: FairyContext>(
        &mut self,
        player_id: i32,
        slot: u32,
        action: i8,
        context: &mut Context,
    ) -> FairyHatchReport {
        let mut report = FairyHatchReport {
            slot,
            action,
            outcome: FairyHatchOutcome::Disabled,
            delivery: None,
        };
        let Some(player) = self.find_player(player_id) else {
            return report;
        };
        if !player.fairy_container_enabled() {
            return report;
        }
        if !(5..=9).contains(&slot) {
            report.outcome = FairyHatchOutcome::InvalidSlot;
            return report;
        }
        if action != b'b' as i8 && action != b's' as i8 {
            report.outcome = FairyHatchOutcome::InvalidAction;
            return report;
        }
        let now = (action == b'b' as i8).then(|| context.current_fairy_tick());
        let egg_max_level = self.globe_setup.fairy_egg_max_level();
        let Some(goods) = self
            .find_player_mut(player_id)
            .and_then(|player| player.fairy_container_mut().base_mut().get_goods_mut(slot))
        else {
            report.outcome = FairyHatchOutcome::MissingGoods;
            return report;
        };
        let changed = if let Some(now) = now {
            goods.hatch_begin(now, egg_max_level)
        } else {
            goods.hatch_stop()
        };
        if !changed {
            report.outcome = FairyHatchOutcome::Unchanged;
            return report;
        }
        let mut response = CMessage::new(0x0b_f91d);
        response.add_ulong(slot);
        response.add_byte(action as u8);
        if action == b'b' as i8 {
            response.add_ulong(self.globe_setup.fairy_hatch_time());
        }
        report.delivery = Some(response.send_to_player(self.net_server(), player_id));
        report.outcome = FairyHatchOutcome::Changed;
        report
    }

    /// Concrete `CPlayer::AI -> CFairyContainer::CheckHatcher` tail. Главный
    /// loop вызывает его сразу после общего AI owner-а, сохраняя timer,
    /// replacement, object-move и incubate-log в одном tick-е.
    pub(crate) fn run_fairy_hatchers<Context: FairyContext>(
        &mut self,
        context: &mut Context,
    ) -> Vec<FairyHatcherRunReport> {
        let player_ids: Vec<_> = self.players.keys().copied().collect();
        let mut reports = Vec::new();
        for player_id in player_ids {
            let enabled = self
                .players
                .get(&player_id)
                .is_some_and(CPlayer::fairy_container_enabled);
            if !enabled {
                continue;
            }
            let hatch_duration = self.globe_setup.fairy_hatch_time();
            let incubate_log_enabled = self.log_system.fairy_incubate_enabled();
            let entries = {
                let (players, random_state, goods_factory, fairy_exp_conf, battle_fairy_exp_config) = (
                    &mut self.players,
                    &mut self.random_state,
                    &self.goods_factory,
                    &self.fairy_exp_conf,
                    &self.battle_fairy_exp_config,
                );
                let player = players
                    .get_mut(&player_id)
                    .expect("fairy hatcher player ID собран из live map");
                let owner_progress_allows = player.current_progress() == PlayerProgress::None;
                let mut random = |upper_bound| game_legacy_random(random_state, upper_bound);
                let mut create_goods = |goods_index| {
                    goods_factory
                        .create_goods_batch(
                            goods_index,
                            1,
                            &mut random,
                            || CGuid::create().unwrap_or(CGuid::GUID_INVALID),
                            |equip_level, level| fairy_exp_conf.dw_exp_up(equip_level, level),
                            |equip_level, level| {
                                battle_fairy_exp_config.dw_exp_up(equip_level, level)
                            },
                        )
                        .into_iter()
                        .next()
                };
                let mut threshold =
                    |equip_level, level| fairy_exp_conf.dw_exp_up(equip_level, level);
                let context_cell = std::cell::RefCell::new(&mut *context);
                let mut current_tick = || context_cell.borrow_mut().current_fairy_tick();
                let mut encode =
                    |goods: &CGoods| context_cell.borrow_mut().encode_goods_for_old_client(goods);
                player.fairy_container_mut().check_hatcher(
                    &mut current_tick,
                    hatch_duration,
                    incubate_log_enabled,
                    goods_factory,
                    owner_progress_allows,
                    &mut threshold,
                    &mut create_goods,
                    &mut encode,
                )
            };
            let mut state_effect_deliveries = Vec::new();
            let mut world_deliveries = Vec::new();
            for entry in &entries {
                deliver_fairy_state_change(&entry.transition, self, &mut state_effect_deliveries);
                if let Some(log) = &entry.incubate_log {
                    world_deliveries.push(self.send_fairy_incubate_log(log));
                }
            }
            reports.push(FairyHatcherRunReport {
                player_id,
                entries,
                state_effect_deliveries,
                world_deliveries,
            });
        }
        reports
    }

    pub(crate) fn implant_fairy_experience<Context: FairyContext>(
        &mut self,
        player_id: i32,
        requested_vigour: u32,
        context: &mut Context,
    ) -> FairyImplantResultReport {
        let mut report = FairyImplantResultReport {
            requested_vigour,
            consumed_vigour: 0,
            crystal_amount: 0,
            outcome: FairyImplantOutcome::Disabled,
            result_delivery: None,
            goods_update_delivery: None,
            state_effect_deliveries: Vec::new(),
            packet_consumptions: Vec::new(),
            packet_deliveries: Vec::new(),
            property_delivery: None,
            world_deliveries: Vec::new(),
            implantation: None,
        };
        let Some(player) = self.find_player(player_id) else {
            return report;
        };
        if !player.fairy_container_enabled() {
            return report;
        }
        let Some(fairy) = player
            .fairy_container()
            .base()
            .get_goods(0)
            .and_then(CGoods::fairy_properties)
        else {
            report.outcome = FairyImplantOutcome::MissingFairy;
            report.result_delivery = Some(send_fairy_long(self, player_id, 0x0b_f91e, 1));
            return report;
        };
        if (fairy.fairy_state == 0 && self.globe_setup.fairy_egg_max_level() <= fairy.level)
            || fairy.ripe_max_level <= fairy.level
        {
            report.outcome = FairyImplantOutcome::MaximumLevel;
            report.result_delivery = Some(send_fairy_long(self, player_id, 0x0b_f91e, 2));
            return report;
        }
        if requested_vigour == 0 || 0x98_9681 <= requested_vigour {
            report.outcome = FairyImplantOutcome::InvalidVigour;
            return report;
        }
        if player.vigour() < requested_vigour {
            report.outcome = FairyImplantOutcome::InsufficientVigour;
            report.result_delivery = Some(send_fairy_long(self, player_id, 0x0b_f91e, 3));
            return report;
        }
        let crystal_scale = self.globe_setup.fairy_vigour_crystal_scale();
        let exp_scale = self.globe_setup.fairy_exp_vigour_scale();
        let initial_crystals = round_fairy_value(requested_vigour as f32 * crystal_scale + 0.999);
        let crystal_index = self
            .goods_factory
            .query_goods_id_by_original_name(Some(b"FZ0965"));
        if crystal_index == 0 || player.check_item_in_packet(crystal_index) < initial_crystals {
            report.outcome = FairyImplantOutcome::InsufficientCrystal;
            report.result_delivery = Some(send_fairy_long(self, player_id, 0x0b_f91e, 4));
            return report;
        }

        let experience = round_fairy_value(requested_vigour as f32 * exp_scale);
        let egg_max_level = self.globe_setup.fairy_egg_max_level();
        let upgrade_rate = self.globe_setup.fairy_upgrade_rate();
        let grow_log_enabled = self.log_system.fairy_grow_enabled();
        let implantation_log_enabled = self.log_system.fairy_implantation_enabled();
        let implantation = {
            let (players, random_state, goods_factory, fairy_exp_conf, battle_fairy_exp_config) = (
                &mut self.players,
                &mut self.random_state,
                &self.goods_factory,
                &self.fairy_exp_conf,
                &self.battle_fairy_exp_config,
            );
            let player = players
                .get_mut(&player_id)
                .expect("implantation player проверен до mutation");
            let owner_progress_allows = player.current_progress() == PlayerProgress::None;
            let mut random = |upper_bound| game_legacy_random(random_state, upper_bound);
            let mut create_goods = |goods_index| {
                goods_factory
                    .create_goods_batch(
                        goods_index,
                        1,
                        &mut random,
                        || CGuid::create().unwrap_or(CGuid::GUID_INVALID),
                        |equip_level, level| fairy_exp_conf.dw_exp_up(equip_level, level),
                        |equip_level, level| battle_fairy_exp_config.dw_exp_up(equip_level, level),
                    )
                    .into_iter()
                    .next()
            };
            let mut threshold = |equip_level, level| fairy_exp_conf.dw_exp_up(equip_level, level);
            let mut encode = |goods: &CGoods| context.encode_goods_for_old_client(goods);
            player.fairy_container_mut().implant_exp(
                experience,
                grow_log_enabled,
                egg_max_level,
                upgrade_rate,
                goods_factory,
                owner_progress_allows,
                &mut threshold,
                &mut create_goods,
                &mut encode,
            )
        };
        let Ok(Some(implantation)) = implantation else {
            report.outcome = FairyImplantOutcome::PropertyBlocked;
            return report;
        };
        for log in &implantation.exp.grow_logs {
            report.world_deliveries.push(self.send_fairy_grow_log(log));
        }
        if let FairyImplantDelivery::StateChanged(transition) = &implantation.delivery {
            deliver_fairy_state_change(transition, self, &mut report.state_effect_deliveries);
        }
        if implantation_log_enabled {
            let log = FairyImplantationLog {
                player_id,
                goods: implantation.goods,
                goods_name: implantation.goods_name.clone(),
                old_level: implantation.old_level,
                resulting_level: implantation.resulting_level,
                crystal_amount: initial_crystals,
            };
            report
                .world_deliveries
                .push(self.send_fairy_implantation_log(&log));
        }
        let consumed_vigour = if implantation.exp.remaining_experience != 0 && exp_scale != 0.0 {
            requested_vigour.wrapping_sub(round_fairy_value(
                implantation.exp.remaining_experience as f32 / exp_scale,
            ))
        } else {
            requested_vigour
        };
        let crystal_amount = if consumed_vigour == requested_vigour {
            initial_crystals
        } else {
            round_fairy_value(consumed_vigour as f32 * crystal_scale + 1.0)
        };
        report.consumed_vigour = consumed_vigour;
        report.crystal_amount = crystal_amount;
        {
            let player = self
                .find_player_mut(player_id)
                .expect("implantation player остаётся зарегистрирован");
            player.set_vigour(player.vigour().wrapping_sub(consumed_vigour));
        }
        let external = context.player_properties_external_facts(player_id);
        report.property_delivery = self
            .find_player(player_id)
            .map(|player| self.send_player_properties_changed(player, external));
        let consumptions = self
            .find_player_mut(player_id)
            .expect("implantation player остаётся зарегистрирован")
            .remove_item_in_packet(crystal_index, crystal_amount);
        for consumption in consumptions {
            report
                .packet_deliveries
                .push(self.send_player_packet_consumption(&consumption));
            report.packet_consumptions.push(consumption);
        }
        report.goods_update_delivery = Some(send_fairy_goods_update(
            self,
            &FairyContainerGoodsUpdate {
                message_type: 0x0b_f918,
                player_id,
                goods: implantation.goods,
                old_client_payload: implantation.old_client_payload.clone(),
            },
        ));
        report.result_delivery = Some(send_fairy_long(self, player_id, 0x0b_f91e, 5));
        report.outcome = FairyImplantOutcome::Completed;
        report.implantation = Some(implantation);
        report
    }

    pub(crate) fn syncretize_fairy<Context: FairyContext>(
        &mut self,
        player_id: i32,
        property: FairySyncreticProperty,
        context: &mut Context,
    ) -> Option<FairySyncretizeResultReport> {
        let (experience, money, vigour) = {
            let player = self.find_player(player_id)?;
            if !player.fairy_container_enabled() {
                return None;
            }
            (player.experience(), player.money(), player.vigour())
        };
        let successful_roll = (game_legacy_random(&mut self.random_state, 10_000) as f32)
            < self.globe_setup.fairy_syncretic_success_rate() * 10_000.0;
        let config = FairySyncretizeConfig {
            needed_goods: self.globe_setup.fairy_syncretic_needed_goods(),
            needed_experience: self.globe_setup.fairy_syncretic_needed_experience(),
            needed_money: self.globe_setup.fairy_syncretic_needed_money(),
            rate_a: self.globe_setup.fairy_syncretic_rate(0),
            rate_b: self.globe_setup.fairy_syncretic_rate(1),
            rate_c: self.globe_setup.fairy_syncretic_rate(2),
            rate_d: self.globe_setup.fairy_syncretic_rate(3),
            rate_e: self.globe_setup.fairy_syncretic_rate(4),
            rate_f: self.globe_setup.fairy_syncretic_rate(5),
            rate_g: self.globe_setup.fairy_syncretic_rate(6),
            rate_h: self.globe_setup.fairy_syncretic_rate(7),
            rate_n: self.globe_setup.fairy_syncretic_rate_n(),
            rate_y: self.globe_setup.fairy_syncretic_rate_y(),
            log_enabled: self.log_system.fairy_syncretize_enabled(),
        };
        let mut player_snapshot = FairySyncretizePlayer {
            id: player_id,
            experience,
            money,
            vigour,
        };
        let mut report = {
            let (players, random_state, goods_factory, fairy_exp_conf, battle_fairy_exp_config) = (
                &mut self.players,
                &mut self.random_state,
                &self.goods_factory,
                &self.fairy_exp_conf,
                &self.battle_fairy_exp_config,
            );
            let player = players.get_mut(&player_id)?;
            let owner_progress_allows = player.current_progress() == PlayerProgress::None;
            let mut random = |upper_bound| game_legacy_random(random_state, upper_bound);
            let mut create_goods = |goods_index| {
                goods_factory
                    .create_goods_batch(
                        goods_index,
                        1,
                        &mut random,
                        || CGuid::create().unwrap_or(CGuid::GUID_INVALID),
                        |equip_level, level| fairy_exp_conf.dw_exp_up(equip_level, level),
                        |equip_level, level| battle_fairy_exp_config.dw_exp_up(equip_level, level),
                    )
                    .into_iter()
                    .next()
            };
            let mut threshold = |equip_level, level| fairy_exp_conf.dw_exp_up(equip_level, level);
            let mut encode = |goods: &CGoods| context.encode_goods_for_old_client(goods);
            player.fairy_container_mut().fairy_syncretize(
                property,
                successful_roll,
                Some(&mut player_snapshot),
                config,
                goods_factory,
                owner_progress_allows,
                &mut threshold,
                &mut create_goods,
                &mut encode,
            )
        };
        let money_change = if let Some(mut update) = report.player_update {
            let (players, goods_factory) = (&mut self.players, &self.goods_factory);
            let player = players.get_mut(&player_id)?;
            player.set_experience(update.experience);
            player.set_vigour(update.vigour);
            let decrease = player.decrease_money(money.wrapping_sub(update.money), goods_factory);
            update.money = decrease.current;
            report.player_update = Some(update);
            Some(decrease)
        } else {
            None
        };
        let mut state_effect_deliveries = Vec::new();
        let mut amount_change_deliveries = Vec::new();
        deliver_fairy_syncretize_effects(
            &report,
            self,
            &mut state_effect_deliveries,
            &mut amount_change_deliveries,
        );
        let money_deliveries = money_change
            .as_ref()
            .map(|change| self.send_player_money_decrease(player_id, &change.outcome))
            .unwrap_or_default();
        let player_update_deliveries = report
            .player_update
            .map(|update| self.send_fairy_player_update(&update))
            .unwrap_or_default();
        let world_deliveries = report
            .log
            .as_ref()
            .map(|log| vec![self.send_fairy_syncretize_log(log)])
            .unwrap_or_default();
        let result_delivery = send_fairy_long(self, player_id, 0x0b_f91f, report.result as u32);
        Some(FairySyncretizeResultReport {
            report,
            state_effect_deliveries,
            amount_change_deliveries,
            money_deliveries,
            player_update_deliveries,
            world_deliveries,
            result_delivery,
        })
    }

    pub(crate) fn query_fairy_setup(&self, player_id: i32) -> FairySetupQueryReport {
        let enabled = self
            .find_player(player_id)
            .is_some_and(CPlayer::fairy_container_enabled);
        let delivery = enabled.then(|| {
            let mut response = CMessage::new(0x0b_f920);
            response.add_ulong(self.globe_setup.fairy_vigour_crystal_scale().to_bits());
            response.add_ulong(self.globe_setup.fairy_exp_vigour_scale().to_bits());
            response.add_ulong(self.globe_setup.fairy_syncretic_needed_goods());
            response.add_ulong(self.globe_setup.fairy_syncretic_needed_experience());
            response.add_ulong(self.globe_setup.fairy_syncretic_needed_money());
            response.add_ulong(self.globe_setup.fairy_syncretic_rate_n().to_bits());
            response.add_ulong(self.globe_setup.fairy_syncretic_rate_y().to_bits());
            response.send_to_player(self.net_server(), player_id)
        });
        FairySetupQueryReport { enabled, delivery }
    }

    pub(crate) fn open_synthesis<Context: SynthesisContext>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> Option<SynthesisOpenReport> {
        let player = self.find_player(player_id)?;
        let facts = context.synthesis_open_facts(self, player);
        let outcome = if !facts.safe_region_cell {
            SynthesisOpenOutcome::UnsafeRegion
        } else if player.current_progress() == PlayerProgress::Trading {
            SynthesisOpenOutcome::Trading
        } else if facts.fight_state_count > 0 {
            SynthesisOpenOutcome::Fighting
        } else if player.current_progress() == PlayerProgress::OpenStall {
            SynthesisOpenOutcome::StallOpen
        } else if facts.has_team_state {
            SynthesisOpenOutcome::TeamState
        } else {
            SynthesisOpenOutcome::Opened
        };
        let mut response = CMessage::new(0x0b_f922);
        response.base_mut().add_byte(outcome as u8);
        let delivery = response.send_to_player(self.net_server(), player_id);
        let player_mutation = (outcome == SynthesisOpenOutcome::Opened)
            .then(|| {
                self.find_player_mut(player_id)
                    .map(CPlayer::begin_synthesis)
            })
            .flatten();
        Some(SynthesisOpenReport {
            outcome,
            delivery,
            player_mutation,
        })
    }

    pub(crate) fn close_synthesis(
        &mut self,
        player_id: i32,
    ) -> Option<crate::gameserver::appserver::player::GoodsSessionPlayerRelease> {
        self.find_player_mut(player_id)?.close_synthesis()
    }

    fn synthesis_result_fits_packet(&self, player_id: i32, goods: &[CGoods]) -> bool {
        self.find_player(player_id).is_some_and(|player| {
            player
                .packet()
                .is_space_enough_for_goods(goods, &self.goods_factory)
        })
    }

    pub(crate) fn compose_synthesis<Context: SynthesisContext>(
        &mut self,
        player_id: i32,
        synthesis_index: u32,
        amount: u32,
        context: &mut Context,
    ) -> Option<SynthesisComposeReport> {
        let mut report = SynthesisComposeReport {
            synthesis_index,
            requested_amount: amount,
            result_amount: amount,
            outcome: SynthesisComposeOutcome::MissingRecipe,
            result_delivery: None,
            notice_delivery: None,
            money_delivery: Vec::new(),
            consumptions: Vec::new(),
            consumption_deliveries: Vec::new(),
            additions: Vec::new(),
            addition_deliveries: Vec::new(),
            rejected_goods: Vec::new(),
            broadcast_delivery: None,
        };
        let Some(recipe) = self
            .synthesis
            .recipe(synthesis_index)
            .filter(|recipe| {
                recipe.coins != -1
                    && recipe.prestige != -1
                    && recipe.probability != u16::MAX
                    && !recipe.formulas.is_empty()
            })
            .cloned()
        else {
            return Some(report);
        };
        let required = |formula: &SynthesisFormula| u64::from(formula.amount) * u64::from(amount);
        let (missing_ingredients, player_money, player_contribution) = {
            let player = self.find_player(player_id)?;
            (
                recipe.formulas.iter().any(|formula| {
                    u64::from(player.check_item_in_packet(formula.goods_index)) < required(formula)
                }),
                player.money(),
                player.contribution(),
            )
        };
        if missing_ingredients {
            report.outcome = SynthesisComposeOutcome::MissingIngredients;
            report.result_delivery = Some(send_synthesis_result(self, player_id, 0));
            return Some(report);
        }
        let total_coins = u64::from(recipe.coins as u32) * u64::from(amount);
        if u64::from(player_money) < total_coins {
            report.outcome = SynthesisComposeOutcome::InsufficientMoney;
            report.result_delivery = Some(send_synthesis_result(self, player_id, 1));
            return Some(report);
        }
        if player_contribution < recipe.prestige {
            report.outcome = SynthesisComposeOutcome::InsufficientContribution;
            report.result_delivery = Some(send_synthesis_result(self, player_id, 2));
            return Some(report);
        }
        if recipe.probability != 100
            && game_legacy_random(&mut self.random_state, 100).wrapping_add(1)
                > i32::from(recipe.probability)
        {
            report.result_amount = 0;
        }
        let created = if report.result_amount == 0 {
            Vec::new()
        } else {
            let (random_state, goods_factory, fairy_exp_conf, battle_fairy_exp_config) = (
                &mut self.random_state,
                &self.goods_factory,
                &self.fairy_exp_conf,
                &self.battle_fairy_exp_config,
            );
            let mut random = |upper_bound| game_legacy_random(random_state, upper_bound);
            goods_factory.create_goods_batch(
                recipe.goods_index,
                report.result_amount,
                &mut random,
                || CGuid::create().unwrap_or(CGuid::GUID_INVALID),
                |equip_level, level| fairy_exp_conf.dw_exp_up(equip_level, level),
                |equip_level, level| battle_fairy_exp_config.dw_exp_up(equip_level, level),
            )
        };
        if !created.is_empty() && !self.synthesis_result_fits_packet(player_id, &created) {
            report.outcome = SynthesisComposeOutcome::InsufficientPacketSpace;
            report.result_delivery = Some(send_synthesis_result(self, player_id, 3));
            return Some(report);
        }

        let money_change = {
            let (players, goods_factory) = (&mut self.players, &self.goods_factory);
            players
                .get_mut(&player_id)?
                .decrease_money(total_coins as u32, goods_factory)
        };
        report.money_delivery = self.send_player_money_decrease(player_id, &money_change.outcome);
        for formula in &recipe.formulas {
            let consumptions = self.find_player_mut(player_id)?.remove_item_in_packet(
                formula.goods_index,
                required(formula).min(u64::from(u32::MAX)) as u32,
            );
            for consumption in consumptions {
                report
                    .consumption_deliveries
                    .push(self.send_player_packet_consumption(&consumption));
                report.consumptions.push(consumption);
            }
        }
        if report.result_amount == 0 {
            report.outcome = SynthesisComposeOutcome::RandomFailure;
            report.result_delivery = Some(send_synthesis_result(self, player_id, 5));
            let text = self.get_string_by_id(b"GS1017");
            report.notice_delivery = Some(
                colored_player_notice_message(0xffff_ffff, 0, text)
                    .send_to_player(self.net_server(), player_id),
            );
            return Some(report);
        }
        let (additions, rejected) = {
            let (players, goods_factory) = (&mut self.players, &self.goods_factory);
            let player = players.get_mut(&player_id)?;
            let mut encode = |goods: &CGoods| context.encode_goods_for_old_client(goods);
            player.add_goods_to_packet(created, goods_factory, &mut encode)
        };
        for addition in additions {
            report
                .addition_deliveries
                .push(self.send_player_packet_addition(&addition));
            report.additions.push(addition);
        }
        report.rejected_goods = rejected.iter().map(CGoods::identity).collect();
        report.outcome = SynthesisComposeOutcome::Completed;
        report.result_delivery = Some(send_synthesis_result(self, player_id, 4));
        let text = self.get_string_by_id(b"GS1016");
        report.notice_delivery = Some(
            colored_player_notice_message(0xffff_ffff, 0, text)
                .send_to_player(self.net_server(), player_id),
        );
        report.broadcast_delivery = synthesis_broadcast_message(self, player_id, &recipe, amount)
            .map(|message| message.send(self, false));
        Some(report)
    }

    pub(crate) const fn new_skill_monster_conf_mut(&mut self) -> &mut NewSkillMonsterConf {
        &mut self.new_skill_monster_conf
    }

    pub(crate) const fn goods_destroy_setup_mut(&mut self) -> &mut GoodsDestroySetup {
        &mut self.goods_destroy_setup
    }

    fn send_goods_destroy_hand_consumption(
        &self,
        consumption: &GoodsDestroyHandConsumption,
    ) -> Vec<i32> {
        if consumption.removed_amount == 0 {
            return Vec::new();
        }
        if consumption.remaining_amount == 0 {
            let mut message = CS2CContainerObjectMove::default();
            message.set_operation(ContainerObjectMoveOperation::DeleteObject);
            message.set_source_container(PLAYER_TYPE, consumption.player_id, 0);
            message.set_source_container_extend_id(3);
            message.set_source_object(
                consumption.goods.object_type,
                consumption.goods.ex_id,
                consumption.previous_amount,
            );
            return vec![message.send_to_player(self, consumption.player_id)];
        }
        let mut message = CS2CContainerObjectAmountChange::default();
        message.set_source_container(PLAYER_TYPE, consumption.player_id, 0);
        message.set_source_container_extend_id(3);
        message.set_object(consumption.goods.object_type, consumption.goods.ex_id);
        message.set_object_amount(consumption.remaining_amount);
        vec![message.send_to_player(self, consumption.player_id)]
    }

    fn send_goods_destroy_log(&self, player: &CPlayer, log: &GoodsDestroyAuditLog) -> Vec<i32> {
        let mut message = CMessage::new(0x0006_0202);
        message.add_byte(log.reason);
        message.add_long(log.player_id);
        message.base_mut().add_short(log.pk_count as i16);
        message.add_ulong(log.money);
        message.add_ulong(player.depot_money());
        message.base_mut().add_guid(log.goods.ex_id);
        message.add_ulong(log.price);
        add_legacy_c_string(message.base_mut(), &log.name);
        message.add_ulong(log.removed_amount);
        message.add_long(log.region_id.unwrap_or_default());
        message.add_ulong(log.tile_x.unwrap_or_default() as u32);
        message.add_ulong(log.tile_y.unwrap_or_default() as u32);
        message.add_ulong(player.client_ip());
        message.send(self, false).into_iter().collect()
    }

    pub(crate) fn open_goods_destroy<Context: GoodsDestroyContext>(
        &mut self,
        player_id: i32,
        container_extend_id: i32,
        goods_id: Option<CGuid>,
        requested_amount: u32,
        context: &mut Context,
    ) -> GoodsDestroyOpenReport {
        let mut report = GoodsDestroyOpenReport {
            player_id,
            container_extend_id,
            goods_id,
            requested_amount,
            outcome: GoodsDestroyOpenOutcome::ConfigurationDisabled,
            deletion: None,
            notice_delivery: None,
            response_delivery: None,
        };
        if container_extend_id != 0 {
            report.outcome = GoodsDestroyOpenOutcome::DeleteRequested;
            report.deletion = goods_id.map(|goods_id| {
                context.delete_goods_for_destroy_open(
                    self,
                    GoodsDestroyDeleteRequest {
                        player_id,
                        container_extend_id,
                        goods_id,
                        requested_amount,
                        write_delete_log: false,
                    },
                )
            });
            return report;
        }

        let enabled = self.goods_destroy_setup.enabled();
        report.outcome = if enabled {
            GoodsDestroyOpenOutcome::ConfigurationEnabled
        } else {
            let text = self.get_string_by_id(b"GS1014");
            report.notice_delivery = Some(
                colored_player_notice_message(0xffff_ffff, 0xffff_0000, text)
                    .send_to_player(self.net_server(), player_id),
            );
            GoodsDestroyOpenOutcome::ConfigurationDisabled
        };
        let mut response = CMessage::new(0x0b_f926);
        response.base_mut().add_byte(u8::from(enabled));
        report.response_delivery = Some(response.send_to_player(self.net_server(), player_id));
        report
    }

    pub(crate) fn confirm_goods_destroy<Context: GoodsDestroyContext>(
        &mut self,
        player_id: i32,
        _context: &mut Context,
    ) -> GoodsDestroyConfirmReport {
        let mut report = GoodsDestroyConfirmReport {
            player_id,
            outcome: GoodsDestroyConfirmOutcome::ConfigurationDisabled,
            goods: None,
            type_key: None,
            equipment_state: None,
            requested_amount: 0,
            removed_amount: 0,
            consumption: None,
            consumption_deliveries: Vec::new(),
            notice_delivery: None,
            audit: None,
            audit_dispatched: false,
            world_deliveries: Vec::new(),
            result_delivery: None,
        };
        if !self.goods_destroy_setup.enabled() {
            return report;
        }

        let Some(hand_goods) = self
            .find_player(player_id)
            .and_then(CPlayer::ci_qing_hand_goods)
        else {
            report.outcome = GoodsDestroyConfirmOutcome::MissingHandGoods;
            return report;
        };
        let identity = hand_goods.identity();
        report.goods = Some(identity);
        let Some(properties) = self
            .goods_factory
            .query_goods_base_properties(hand_goods.base_properties_index())
        else {
            report.outcome = GoodsDestroyConfirmOutcome::MissingBaseProperties;
            return report;
        };
        if self
            .goods_destroy_setup
            .original_names()
            .iter()
            .any(|name| name.as_slice() == properties.original_name())
        {
            report.outcome = GoodsDestroyConfirmOutcome::OriginalNameRestricted;
            let text = self.get_string_by_id(b"GS1015");
            report.notice_delivery = Some(
                colored_player_notice_message(0xffff_ffff, 0xffff_0000, text)
                    .send_to_player(self.net_server(), player_id),
            );
            return report;
        }
        let type_key = if properties.goods_type() == GOODS_TYPE_EQUIPMENT {
            properties.equip_place().wrapping_add(1) as u16
        } else {
            properties.goods_type() as u16
        };
        report.type_key = Some(type_key);
        if !self.goods_destroy_setup.goods_types().contains(&type_key) {
            report.outcome = GoodsDestroyConfirmOutcome::GoodsTypeRejected;
            let text = self.get_string_by_id(b"GS1014");
            report.notice_delivery = Some(
                colored_player_notice_message(0xffff_ffff, 0xffff_0000, text)
                    .send_to_player(self.net_server(), player_id),
            );
            return report;
        }
        let equipment_state =
            hand_goods.addon_property_value(&self.goods_factory, GAP_EQUIP_STATE, 1);
        report.equipment_state = Some(equipment_state);
        if matches!(equipment_state, 1 | 2) {
            report.outcome = GoodsDestroyConfirmOutcome::EquipmentStateRestricted;
            let text = self.get_string_by_id(b"GSN1014");
            report.notice_delivery = Some(
                colored_player_notice_message(0xffff_ffff, 0xffff_0000, text)
                    .send_to_player(self.net_server(), player_id),
            );
            return report;
        }

        let price = hand_goods.price();
        let name = hand_goods.name().to_vec();
        let amount = hand_goods.amount();
        report.requested_amount = amount;
        let consumption = self
            .find_player_mut(player_id)
            .and_then(|player| player.destroy_hand_goods(identity.ex_id, amount));
        if let Some(consumption) = consumption {
            report.removed_amount = consumption.removed_amount;
            report.consumption_deliveries = self.send_goods_destroy_hand_consumption(&consumption);
            report.consumption = Some(consumption);
        }

        if report.removed_amount != 0 && self.log_system.goods_destroy_enabled() {
            let player = self
                .find_player(player_id)
                .expect("player с hand goods остаётся в CGame после synchronous удаления");
            let audit = GoodsDestroyAuditLog {
                reason: 0x15,
                player_id,
                pk_count: player.pk_count(),
                money: player.money(),
                goods: identity,
                price,
                name,
                removed_amount: report.removed_amount,
                region_id: player.server_region_id(),
                tile_x: player.shape().get_tile_x(),
                tile_y: player.shape().get_tile_y(),
            };
            report.world_deliveries = self.send_goods_destroy_log(player, &audit);
            report.audit = Some(audit);
            report.audit_dispatched = true;
        }
        let mut response = CMessage::new(0x0b_f927);
        response.add_ulong(report.removed_amount);
        report.result_delivery = Some(response.send_to_player(self.net_server(), player_id));
        report.outcome = GoodsDestroyConfirmOutcome::Destroyed;
        report
    }

    pub(crate) const fn change_body_conf_mut(&mut self) -> &mut CChangeBodyConf {
        &mut self.change_body_conf
    }

    pub(crate) const fn honor_eliminate_config_mut(&mut self) -> &mut HonorElimilateConfig {
        &mut self.honor_eliminate_config
    }

    pub(crate) const fn dupli_region_setup(&self) -> Option<&CDupliRegionSetup> {
        self.dupli_region_setup.as_ref()
    }

    pub(crate) const fn dupli_region_setup_mut(&mut self) -> Option<&mut CDupliRegionSetup> {
        self.dupli_region_setup.as_mut()
    }

    pub(crate) const fn move_check_cells(&self) -> &MoveCheckCellRegistry {
        &self.move_check_cells
    }

    pub(crate) const fn player_ranks(&self) -> Option<&CPlayerRanks> {
        self.player_ranks.as_ref()
    }

    pub(crate) const fn player_ranks_mut(&mut self) -> Option<&mut CPlayerRanks> {
        self.player_ranks.as_mut()
    }

    pub(crate) fn request_script_player_ranks(
        &mut self,
        player_id: i32,
        maximum_rank_count: i32,
        now_ms: u32,
    ) -> Option<Result<PlayerRanksRequestOutcome, PlayerRanksSerializeError>> {
        let (ranks, net_server) = (&mut self.player_ranks, &self.net_server);
        Some(ranks.as_mut()?.on_player_get_ranks(
            player_id,
            maximum_rank_count,
            now_ms,
            net_server.as_ref()?,
        ))
    }

    /// Scalar honor-rank selector читает тот же startup/timer snapshot, который
    /// WorldServer обновляет через `CHonorRanks::DecordFromByteArray`.
    pub(crate) fn script_honor_rank_position(&self, player_id: i32, rank_type: i32) -> i32 {
        let Some(player) = self.find_player(player_id) else {
            return 0;
        };
        self.honor_ranks.player_position(
            rank_type,
            i32::from(player.country()).wrapping_sub(1),
            player_id,
        )
    }

    /// Exact `SendTotalHonorRanks`: country DWORD precedes type-3 country
    /// payload, а malformed country останавливает отправку целиком.
    pub(crate) fn send_script_total_honor_ranks(&self, player_id: i32) -> i32 {
        let Some(player) = self.find_player(player_id) else {
            return 0;
        };
        let country = i32::from(player.country());
        let mut payload = Vec::new();
        if !self
            .honor_ranks
            .add_to_byte_array(&mut payload, 3, country.wrapping_sub(1))
        {
            return 0;
        }
        let mut message = CMessage::new(0x0b_ff35);
        message.add_long(country);
        message.base_mut().add(&payload);
        let _ = message.send_to_player(self.net_server(), player_id);
        0
    }

    pub(crate) fn script_attempt_appellation_id(&self, player_id: i32) -> Option<u32> {
        self.find_player(player_id)
            .map(CPlayer::attempt_appellation_id)
    }

    pub(crate) fn add_script_appellation_state<Context: RealmAppellationScriptContext>(
        &mut self,
        player_id: i32,
        state_id: u32,
        now_ms: u32,
        context: &mut Context,
    ) -> u32 {
        let mutation = {
            let (players, skill_factory) = (&mut self.players, &self.skill_factory);
            let Some(player) = players.get_mut(&player_id) else {
                return 0;
            };
            player.add_appellation_state(state_id, skill_factory, || now_ms)
        };
        for state in &mutation.removed {
            self.send_appellation_visual(player_id, state, false, now_ms);
        }
        if let Some(state) = &mutation.added {
            self.send_appellation_visual(player_id, state, true, now_ms);
        }
        if mutation.state_list_changed {
            self.refresh_script_change_body_properties(player_id, context);
            self.send_script_player_state_changed(player_id);
        }
        mutation.legacy_return
    }

    pub(crate) fn delete_script_appellation_state<Context: RealmAppellationScriptContext>(
        &mut self,
        player_id: i32,
        state_id: u32,
        now_ms: u32,
        context: &mut Context,
    ) -> u32 {
        let Some(player) = self.players.get_mut(&player_id) else {
            return 0;
        };
        let mutation = player.delete_appellation_state(state_id);
        for state in &mutation.removed {
            self.send_appellation_visual(player_id, state, false, now_ms);
        }
        if mutation.state_list_changed {
            self.refresh_script_change_body_properties(player_id, context);
            self.send_script_player_state_changed(player_id);
        }
        mutation.legacy_return
    }

    pub(crate) fn get_script_appellation_state(
        &self,
        player_id: i32,
        state_id: u32,
    ) -> Option<u32> {
        self.find_player(player_id)
            .map(|player| player.get_appellation_state(state_id))
    }

    pub(crate) fn script_change_body_check(&self, player_id: i32, explain_failure: bool) -> i32 {
        let Some(player) = self.find_player(player_id) else {
            return 0;
        };
        let allowed = player.change_body_check();
        if !allowed {
            if player.appearance_and_mode().2 != 0 {
                let text = self.get_string_by_id(b"GS1042");
                let _ = colored_player_notice_message(0xffff_ffff, 0, text)
                    .send_to_player(self.net_server(), player_id);
            }
            if explain_failure {
                let text = self.get_string_by_id(b"GS1063");
                let _ = colored_player_notice_message(0xffff_0000, 0, text)
                    .send_to_player(self.net_server(), player_id);
            }
        }
        i32::from(allowed)
    }

    /// Item-use prefix проверяет restrictions только пока уже действует
    /// `CChangeBodyState`; конфигурация и persisted state принадлежат этому же
    /// Game owner-у, поэтому внешний snapshot здесь больше не нужен.
    pub(crate) fn change_body_item_conflicts(&self, player_id: i32, goods_base_index: u32) -> bool {
        self.find_player(player_id)
            .is_some_and(CPlayer::has_change_body_state)
            && self
                .change_body_conf
                .restrictions_goods()
                .contains(&goods_base_index)
    }

    pub(crate) fn add_script_extended_state<Context: RealmAppellationScriptContext>(
        &mut self,
        player_id: i32,
        state_id: u32,
        kind: ExtendedStateKind,
        now_ms: u32,
        context: &mut Context,
    ) -> u32 {
        if kind == ExtendedStateKind::Original
            && self
                .skill_factory
                .query_skill_base_properties(kind.state_id(), state_id as i32)
                .is_some_and(|properties| properties.query_property(20_010) == 0x12f)
        {
            context.remove_script_god_bless_state(player_id);
        }
        let mutation = {
            let (players, skill_factory) = (&mut self.players, &self.skill_factory);
            let Some(player) = players.get_mut(&player_id) else {
                return 0;
            };
            player.add_extended_state(kind, state_id, skill_factory, now_ms)
        };
        for removed in &mutation.removed {
            self.send_extended_state_visual(player_id, removed, false, now_ms);
        }
        if let Some(added) = mutation.added.as_ref() {
            self.send_extended_state_visual(player_id, added, true, now_ms);
        }
        if !mutation.removed.is_empty() || mutation.added.is_some() {
            self.refresh_script_change_body_properties(player_id, context);
            self.send_script_player_state_changed(player_id);
        }
        mutation.legacy_return
    }

    pub(crate) fn delete_script_extended_state<Context: RealmAppellationScriptContext>(
        &mut self,
        player_id: i32,
        state_id: u32,
        kind: ExtendedStateKind,
        now_ms: u32,
        context: &mut Context,
    ) -> u32 {
        let mutation = self
            .find_player_mut(player_id)
            .map(|player| player.delete_extended_state(kind, state_id));
        let Some(mutation) = mutation else { return 0 };
        for removed in &mutation.removed {
            self.send_extended_state_visual(player_id, removed, false, now_ms);
        }
        if !mutation.removed.is_empty() {
            self.refresh_script_change_body_properties(player_id, context);
            self.send_script_player_state_changed(player_id);
        }
        mutation.legacy_return
    }

    pub(crate) fn delete_script_extended_state_by_type<Context: RealmAppellationScriptContext>(
        &mut self,
        player_id: i32,
        state_type: u16,
        now_ms: u32,
        context: &mut Context,
    ) -> u32 {
        let Some(mutation) = self
            .find_player_mut(player_id)
            .map(|player| player.delete_extended_state_by_type(state_type))
        else {
            return 0;
        };
        for removed in &mutation.removed {
            self.send_extended_state_visual(player_id, removed, false, now_ms);
        }
        if !mutation.removed.is_empty() {
            self.refresh_script_change_body_properties(player_id, context);
            self.send_script_player_state_changed(player_id);
        }
        mutation.legacy_return
    }

    pub(crate) fn set_script_jing_li_dan_count(&mut self, player_id: i32, used_count: i32) -> i32 {
        if used_count < 0 || used_count > i32::from(self.globe_setup.total_jing_li_dan_count()) {
            return -1;
        }
        let remaining = self
            .globe_setup
            .total_jing_li_dan_count()
            .wrapping_sub(used_count as u16);
        let Some(player) = self.find_player_mut(player_id) else {
            return -1;
        };
        player.set_remain_jing_li_dan_count(remaining);
        let identity = player.shape().identity();
        let mut changed = CMessage::new(0x000b_f80c);
        changed.add_long(identity.object_type);
        changed.add_long(identity.id);
        add_legacy_c_string(changed.base_mut(), b"wRemainJingLiDanCnt");
        changed.add_long(i32::from(remaining));
        let _ = changed.send_to_player(self.net_server(), player_id);
        -1
    }

    fn send_extended_state_visual(
        &mut self,
        player_id: i32,
        state: &ExtendedState,
        begin: bool,
        now_ms: u32,
    ) {
        let Some(player) = self.find_player(player_id) else {
            return;
        };
        let identity = player.shape().identity();
        let mut message = CMessage::new(if begin { 0x0b_fe03 } else { 0x0b_fe04 });
        message.add_long(identity.object_type);
        message.add_long(player_id);
        message.add_long(state.state_id() as i32);
        message.add_ulong(state.level);
        if begin {
            message.add_ulong(state.remaining_time_ms(now_ms));
            message.add_ulong(u32::from(state.state_type));
        }
        let _ = self.send_player_shape_around(player_id, None, &message);
    }

    fn update_extended_states<Context: RealmAppellationScriptContext>(
        &mut self,
        now_ms: u32,
        context: &mut Context,
    ) {
        let player_ids: Vec<_> = self.players.keys().copied().collect();
        for player_id in player_ids {
            let (expired, item_due) = self
                .find_player_mut(player_id)
                .map(|player| player.extended_state_tick(now_ms))
                .unwrap_or_default();
            for (kind, state_id) in expired {
                let _ =
                    self.delete_script_extended_state(player_id, state_id, kind, now_ms, context);
            }
            for (kind, state_id, item_index, item_amount) in item_due {
                let enough = self
                    .find_player(player_id)
                    .is_some_and(|player| player.check_item_in_packet(item_index) >= item_amount);
                if !enough {
                    let goods_name = self
                        .goods_factory
                        .query_goods_name(item_index)
                        .unwrap_or_default();
                    let text = format_legacy_text_fields(
                        self.get_string_by_id(b"GS0128"),
                        &[goods_name],
                        0xff,
                    );
                    let _ = colored_player_notice_message(0xffff_ffff, 0, &text)
                        .send_to_player(self.net_server(), player_id);
                    let _ = self
                        .delete_script_extended_state(player_id, state_id, kind, now_ms, context);
                    continue;
                }
                let consumptions = self
                    .find_player_mut(player_id)
                    .map(|player| player.remove_item_in_packet(item_index, item_amount))
                    .unwrap_or_default();
                let removed = consumptions.iter().fold(0_u32, |total, consumption| {
                    total.wrapping_add(
                        consumption
                            .previous_amount
                            .wrapping_sub(consumption.remaining_amount),
                    )
                });
                for consumption in &consumptions {
                    let _ = self.send_player_packet_consumption(consumption);
                }
                if removed != item_amount {
                    let _ = self
                        .delete_script_extended_state(player_id, state_id, kind, now_ms, context);
                }
            }
        }
    }

    fn update_appellation_states<Context: RealmAppellationScriptContext>(
        &mut self,
        now_ms: u32,
        context: &mut Context,
    ) {
        let player_ids: Vec<_> = self.players.keys().copied().collect();
        for player_id in player_ids {
            let (ended, item_due) = self
                .find_player_mut(player_id)
                .map(|player| player.appellation_state_tick(now_ms))
                .unwrap_or_default();
            for state_id in ended {
                let _ = self.delete_script_appellation_state(player_id, state_id, now_ms, context);
            }
            for (state_id, item_index, item_amount) in item_due {
                let enough = self
                    .find_player(player_id)
                    .is_some_and(|player| player.check_item_in_packet(item_index) >= item_amount);
                if !enough {
                    let goods_name = self
                        .goods_factory
                        .query_goods_name(item_index)
                        .unwrap_or_default();
                    let text = format_legacy_text_fields(
                        self.get_string_by_id(b"GS0128"),
                        &[goods_name],
                        0xff,
                    );
                    let _ = colored_player_notice_message(0xffff_ffff, 0, &text)
                        .send_to_player(self.net_server(), player_id);
                    let _ =
                        self.delete_script_appellation_state(player_id, state_id, now_ms, context);
                    continue;
                }
                let consumptions = self
                    .find_player_mut(player_id)
                    .map(|player| player.remove_item_in_packet(item_index, item_amount))
                    .unwrap_or_default();
                let removed = consumptions.iter().fold(0_u32, |total, consumption| {
                    total.wrapping_add(
                        consumption
                            .previous_amount
                            .wrapping_sub(consumption.remaining_amount),
                    )
                });
                for consumption in &consumptions {
                    let _ = self.send_player_packet_consumption(consumption);
                }
                if removed != item_amount {
                    let _ =
                        self.delete_script_appellation_state(player_id, state_id, now_ms, context);
                }
            }
        }
    }

    fn send_ride_visual(&mut self, player_id: i32, state: &RideState, begin: bool) {
        let Some(player) = self.find_player(player_id) else {
            return;
        };
        let identity = player.shape().identity();
        let mut message = CMessage::new(if begin { 0x0b_fe03 } else { 0x0b_fe04 });
        message.add_long(identity.object_type);
        message.add_long(player_id);
        message.add_long(RIDE_STATE_ID as i32);
        if begin {
            message.add_ulong(0); // base `CState::GetClientStateTime()`
            message.add_ulong(state.additional_data());
        }
        let _ = self.send_player_shape_around(player_id, None, &message);
    }

    pub(crate) fn begin_player_ride(
        &mut self,
        player_id: i32,
        mount_type: u32,
        level: u32,
        role_limit: u32,
        goods_name: &[u8],
    ) -> bool {
        let Some(state) = self
            .find_player_mut(player_id)
            .and_then(|player| player.begin_ride_state(mount_type, level, role_limit, goods_name))
        else {
            return false;
        };
        self.send_ride_visual(player_id, &state, true);
        true
    }

    pub(crate) fn end_player_ride(&mut self, player_id: i32) -> bool {
        let Some(state) = self
            .find_player_mut(player_id)
            .and_then(CPlayer::end_ride_state)
        else {
            return false;
        };
        self.send_ride_visual(player_id, &state, false);
        true
    }

    pub(crate) fn apply_player_state_properties<Context: BattleFairyDeathContext>(
        &mut self,
        player_id: i32,
        properties: PlayerCombatProperties,
        context: &mut Context,
    ) {
        let coefficients = self.globe_setup.player_property_coefficients();
        let goods_factory = self.goods_factory.clone();
        let Some(player) = self.find_player_mut(player_id) else {
            return;
        };
        player.apply_change_body_properties(properties, coefficients, &goods_factory);
        let external = context.player_properties_external_facts(player_id);
        if let Some(player) = self.find_player(player_id) {
            let _ = self.send_player_properties_changed(player, external);
        }
    }

    fn refresh_ride_properties<Context: RealmAppellationScriptContext>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) {
        let Some(properties) = self
            .find_player(player_id)
            .map(|player| context.recompute_realm_appellation_player_properties(player))
        else {
            return;
        };
        self.apply_player_state_properties(player_id, properties, context);
    }

    fn update_ride_states<Context: RealmAppellationScriptContext>(
        &mut self,
        now_ms: u32,
        context: &mut Context,
    ) {
        let due: Vec<_> = self
            .players
            .iter()
            .filter(|(_, player)| player.ride_goods_check_due(now_ms))
            .map(|(&player_id, _)| player_id)
            .collect();
        let goods_factory = self.goods_factory.clone();
        for player_id in due {
            let exists = self
                .find_player_mut(player_id)
                .is_some_and(|player| player.refresh_ride_goods_cache(&goods_factory));
            if !exists && self.end_player_ride(player_id) {
                self.refresh_ride_properties(player_id, context);
            }
        }
    }

    pub(crate) fn add_script_change_body_state<Context: RealmAppellationScriptContext>(
        &mut self,
        player_id: i32,
        state_id: u32,
        now_ms: u32,
        context: &mut Context,
    ) -> u32 {
        if self.script_change_body_check(player_id, false) == 0 {
            return 0;
        }
        let mutation = {
            let (players, skill_factory) = (&mut self.players, &self.skill_factory);
            let Some(player) = players.get_mut(&player_id) else {
                return 0;
            };
            player.add_change_body_state(state_id, skill_factory, now_ms)
        };
        if let Some(removed) = mutation.removed.as_ref() {
            self.send_change_body_visual(player_id, removed, false);
        }
        let Some(added) = mutation.added.as_ref() else {
            return mutation.legacy_return;
        };
        self.send_change_body_visual(player_id, added, true);
        self.send_change_body_hotkeys(
            player_id,
            added
                .skills
                .iter()
                .enumerate()
                .filter_map(|(index, (skill_id, _))| (*skill_id != 0).then_some(index + 12))
                .chain(std::iter::once(17)),
        );
        self.refresh_script_change_body_properties(player_id, context);
        self.send_script_player_state_changed(player_id);
        mutation.legacy_return
    }

    pub(crate) fn delete_script_change_body_state<Context: RealmAppellationScriptContext>(
        &mut self,
        player_id: i32,
        state_id: u32,
        context: &mut Context,
    ) -> u32 {
        let mutation = {
            let (players, skill_factory) = (&mut self.players, &self.skill_factory);
            let Some(player) = players.get_mut(&player_id) else {
                return 0;
            };
            player.delete_change_body_state(state_id, skill_factory)
        };
        let Some(removed) = mutation.removed.as_ref() else {
            return mutation.legacy_return;
        };
        self.send_change_body_visual(player_id, removed, false);
        self.send_change_body_hotkeys(player_id, 12..=23);
        self.refresh_script_change_body_properties(player_id, context);
        self.send_script_player_state_changed(player_id);
        mutation.legacy_return
    }

    fn end_change_body_states<Context: RealmAppellationScriptContext>(
        &mut self,
        player_id: i32,
        state_ids: Vec<u32>,
        notice_id: Option<&[u8]>,
        context: &mut Context,
    ) {
        for state_id in state_ids {
            if let Some(notice_id) = notice_id {
                let text = self.get_string_by_id(notice_id);
                let _ = colored_player_notice_message(0xffff_ffff, 0, text)
                    .send_to_player(self.net_server(), player_id);
            }
            let _ = self.delete_script_change_body_state(player_id, state_id, context);
        }
    }

    fn expire_change_body_states<Context: RealmAppellationScriptContext>(
        &mut self,
        now_ms: u32,
        context: &mut Context,
    ) {
        let expired: Vec<_> = self
            .players
            .iter()
            .flat_map(|(&player_id, player)| {
                player
                    .expired_change_body_state_ids(now_ms)
                    .into_iter()
                    .map(move |state_id| (player_id, state_id))
            })
            .collect();
        for (player_id, state_id) in expired {
            self.end_change_body_states(player_id, vec![state_id], Some(b"GS1145"), context);
        }
        let dead_players: Vec<_> = self
            .players
            .iter()
            .filter(|(_, player)| {
                player.is_dead() && !player.change_body_death_end_ids().is_empty()
            })
            .map(|(&player_id, _)| player_id)
            .collect();
        for player_id in dead_players {
            self.change_body_after_player_death(player_id, context);
        }
    }

    pub(crate) fn change_body_after_region_transition<Context: RealmAppellationScriptContext>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) {
        let state_ids = self
            .find_player_mut(player_id)
            .map(CPlayer::change_body_region_transition_end_ids)
            .unwrap_or_default();
        self.end_change_body_states(player_id, state_ids, Some(b"GS1146"), context);
    }

    pub(crate) fn change_body_after_player_lost<Context: RealmAppellationScriptContext>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) {
        let state_ids = self
            .find_player_mut(player_id)
            .map(CPlayer::change_body_player_lost_end_ids)
            .unwrap_or_default();
        self.end_change_body_states(player_id, state_ids, None, context);
    }

    fn change_body_after_player_death<Context: RealmAppellationScriptContext>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) {
        let state_ids = self
            .find_player(player_id)
            .map(CPlayer::change_body_death_end_ids)
            .unwrap_or_default();
        self.end_change_body_states(player_id, state_ids, None, context);
    }

    fn refresh_script_change_body_properties<Context: RealmAppellationScriptContext>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) {
        let properties = match self.find_player(player_id) {
            Some(player) => context.recompute_realm_appellation_player_properties(player),
            None => return,
        };
        let coefficients = self.globe_setup.player_property_coefficients();
        let goods_factory = self.goods_factory.clone();
        self.find_player_mut(player_id)
            .expect("ChangeBody recompute сохраняет player")
            .apply_change_body_properties(properties, coefficients, &goods_factory);
        let external = context.player_properties_external_facts(player_id);
        if let Some(player) = self.find_player(player_id) {
            let _ = self.send_player_properties_changed(player, external);
        }
    }

    fn send_change_body_hotkeys(&self, player_id: i32, slots: impl IntoIterator<Item = usize>) {
        let Some(player) = self.find_player(player_id) else {
            return;
        };
        for slot in slots {
            let Some(value) = player.hotkey(slot as u8) else {
                continue;
            };
            let mut message = CMessage::new(0x0b_f908);
            message.add_byte(b'-');
            message.add_byte(slot as u8);
            message.add_ulong(value);
            let _ = message.send_to_player(self.net_server(), player_id);
        }
    }

    /// Exact selector `2560`: slot проверяется до mutation, skill type `1`
    /// кодируется старшим битом, затем тот же `0xBF908` подтверждает значение.
    pub(crate) fn set_script_player_hotkey(&mut self, player_id: i32, slot: u8, value: u32) {
        let Some(player) = self.find_player_mut(player_id) else {
            return;
        };
        if !player.set_hotkey(slot, value) {
            return;
        }
        let mut message = CMessage::new(0x0b_f908);
        message.add_byte(b'-');
        message.add_byte(slot);
        message.add_ulong(value);
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    fn send_change_body_visual(&mut self, player_id: i32, state: &ChangeBodyState, begin: bool) {
        let Some(player) = self.find_player(player_id) else {
            return;
        };
        let identity = player.shape().identity();
        let mut message = CMessage::new(if begin { 0x0b_fe03 } else { 0x0b_fe04 });
        message.add_long(identity.object_type);
        message.add_long(player_id);
        message.add_long(0x37);
        if begin {
            message.add_ulong(state.keep_time_ms);
            message.add_long(0);
            message.add_ulong(state.mode);
            message.add_ulong(state.level);
            message.add_ulong(u32::from(state.visual_effect));
            message.add_byte(u8::from(state.continue_after_death));
            for (skill_id, level) in state.skills {
                message.base_mut().add_short(skill_id as i16);
                message
                    .base_mut()
                    .add_short(if skill_id == 0 { 0 } else { level as i16 });
                let properties = self
                    .skill_factory
                    .query_skill_base_properties(u32::from(skill_id), i32::from(level));
                message.add_long(properties.map_or(0, |p| p.query_property(10_005) as i32));
                message.add_long(properties.map_or(0, |p| {
                    let range = p.query_property(5_003) as i32;
                    if range > 0 { range } else { 1 }
                }));
                message.add_long(properties.map_or(0, |p| p.query_property(2) as i32));
                message.add_long(properties.map_or(0, |p| p.query_property(10_001) as i32));
            }
        } else {
            message.add_long(0);
            message.add_ulong(state.level);
            for (skill_id, _) in state.skills {
                message.base_mut().add_short(skill_id as i16);
            }
        }
        let _ = self.send_player_shape_around(player_id, None, &message);
    }

    /// Reached `AddJingJieBuff` tail: hidden skill и max-HP/max-MP mutation
    /// принадлежат realm owner-у, а изменившийся property snapshot публикуется
    /// тем же адресным `CPlayer::OnChangeProperties` wire `0xBF721`.
    pub(crate) fn set_script_realm_appellation_bonus<Context: RealmAppellationScriptContext>(
        &mut self,
        player_id: i32,
        appellation_id: u32,
        context: &mut Context,
    ) -> Option<i32> {
        let mutation = {
            let (players, skill_factory) = (&mut self.players, &self.skill_factory);
            let player = players.get_mut(&player_id)?;
            crate::gameserver::appserver::skills::realmappellation::set_bonus(
                player,
                appellation_id,
                skill_factory,
            )
        };
        let current_properties = {
            let player = self
                .find_player(player_id)
                .expect("realm mutation сохраняет canonical player");
            context.recompute_realm_appellation_player_properties(player)
        };
        self.find_player_mut(player_id)
            .expect("realm recompute сохраняет canonical player")
            .apply_recomputed_combat_properties(current_properties);
        if mutation.previous_properties != current_properties {
            let external = context.player_properties_external_facts(player_id);
            if let Some(player) = self.find_player(player_id) {
                let _ = self.send_player_properties_changed(player, external);
            }
        }
        Some(i32::from(mutation.succeeded))
    }

    fn send_appellation_visual(
        &self,
        player_id: i32,
        state: &UndeadState,
        begin: bool,
        now_ms: u32,
    ) {
        let Some(player) = self.find_player(player_id) else {
            return;
        };
        let Some(region) = player
            .server_region_id()
            .and_then(|region_id| self.find_region(region_id))
            .map(ServerRegionOwner::base)
        else {
            return;
        };
        let Some(runtime) = GameServerAroundRuntime::new(
            self,
            &self.session_factory,
            self.globe_setup.area_width(),
            self.globe_setup.area_height(),
        ) else {
            return;
        };
        let mut message = CMessage::new(if begin { 0x0b_fe03 } else { 0x0b_fe04 });
        message.add_long(player.shape().identity().object_type);
        message.add_long(player_id);
        message.add_long(56);
        message.add_ulong(state.state_id());
        if begin {
            message.add_ulong(state.remaining_time_ms(now_ms));
            message.add_ulong(u32::from(state.state_type()));
        }
        let _ = message.send_to_around(Some(region), player.shape(), None, &runtime);
    }

    fn send_script_player_state_changed(&self, player_id: i32) {
        let Some(player) = self.find_player(player_id) else {
            return;
        };
        let mut message = CMessage::new(0x0b_fe02);
        message.add_long(player.shape().identity().object_type);
        message.add_long(player_id);
        message.add_ulong(player.health());
        message.add_ulong(player.mana());
        message.base_mut().add_short(0);
        message.base_mut().add_short(0);
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    pub(crate) const fn honor_ranks(&self) -> &CHonorRanks {
        &self.honor_ranks
    }

    pub(crate) const fn honor_ranks_mut(&mut self) -> &mut CHonorRanks {
        &mut self.honor_ranks
    }

    pub(crate) const fn goods_war(&self) -> Option<&CGoodsWarMember> {
        self.goods_war.as_ref()
    }

    pub(crate) const fn goods_war_mut(&mut self) -> Option<&mut CGoodsWarMember> {
        self.goods_war.as_mut()
    }

    pub(crate) fn take_goods_war(&mut self) -> Option<CGoodsWarMember> {
        self.goods_war.take()
    }

    pub(crate) fn restore_goods_war(&mut self, goods_war: CGoodsWarMember) {
        self.goods_war = Some(goods_war);
    }

    /// Публикует listener-owner до `Host`, затем сохраняет setup-порядок.
    pub(crate) fn init_net_server(
        &mut self,
        now_ms: u32,
    ) -> Result<(), GameNetworkInitializationError> {
        let setup = self
            .network_setup
            .as_ref()
            .cloned()
            .ok_or(GameNetworkInitializationError::MissingNetworkSetup)?;
        self.net_server = Some(CMyNetServer::new(now_ms));
        let server = self
            .net_server
            .as_mut()
            .expect("Game server owner только что опубликован");
        server
            .host(setup.listener.listen_port, None, DEFAULT_SOCKET_TYPE, true)
            .map_err(GameNetworkInitializationError::Host)?;

        if let Some(address) = resolve_first_local_ipv4() {
            let dotted = address.to_string();
            server.set_local_identity(dotted.as_bytes(), legacy_ipv4_word(address));
        }
        server.configure_after_host(
            setup.listener.check_network,
            setup.listener.maximum_in_flight_sends,
            setup.listener.maximum_bytes_per_second,
            setup.listener.maximum_clients,
            setup.check_message_content,
            setup.listener.forbid_time_ms,
            setup.listener.maximum_message_length,
            setup.listener.permitted_send_bytes,
        );
        server.configure_accept_limits_after_host(
            setup.listener.maximum_block_connections,
            setup.listener.first_receive_timeout_ms,
        );
        Ok(())
    }

    /// Пересоздаёт initial World client и ставит исходную регистрацию.
    pub(crate) async fn init_world_client(&mut self) -> GameClientInitialization {
        let Some(setup) = self.network_setup.as_ref().cloned() else {
            return GameClientInitialization::Failed {
                attempts: vec![GameConnectAttempt {
                    direction: GameUpstreamDirection::World,
                    failure: Some(GameClientInitializationFailure::MissingNetworkSetup),
                }],
            };
        };
        self.close_and_remove_world_client();
        self.attach_world_client(CMyNetClient::new());

        let mut attempts = Vec::with_capacity(1);
        let result = connect_game_client(
            self.world_client
                .as_mut()
                .expect("World client только что опубликован"),
            &setup.world,
            GameUpstreamDirection::World,
            None,
            0,
        )
        .await;
        let endpoint = match result {
            Ok(endpoint) => {
                attempts.push(GameConnectAttempt {
                    direction: GameUpstreamDirection::World,
                    failure: None,
                });
                endpoint
            }
            Err(failure) => {
                attempts.push(GameConnectAttempt {
                    direction: GameUpstreamDirection::World,
                    failure: Some(failure),
                });
                self.close_and_remove_world_client();
                return GameClientInitialization::Failed { attempts };
            }
        };

        self.world_client
            .as_mut()
            .expect("подключённый World client остаётся опубликованным")
            .enable_control_send();
        let mut registration = CMessage::new(WORLD_REGISTRATION);
        registration.add_byte(0);
        registration.add_ulong(setup.listener.listen_port);
        add_legacy_c_string(registration.base_mut(), &setup.local_ip);
        let registration = registration.send(self, false);
        GameClientInitialization::Connected {
            endpoint,
            used_billing_backup: false,
            registration,
            attempts,
        }
    }

    /// Пересоздаёт Billing client, пробуя master и затем backup endpoint.
    pub(crate) async fn init_billing_client(&mut self) -> GameClientInitialization {
        let Some(setup) = self.network_setup.as_ref().cloned() else {
            return GameClientInitialization::Failed {
                attempts: vec![GameConnectAttempt {
                    direction: GameUpstreamDirection::BillingPrimary,
                    failure: Some(GameClientInitializationFailure::MissingNetworkSetup),
                }],
            };
        };
        self.close_and_remove_billing_client();
        self.attach_billing_client(CMyNetClient::new());

        let bind_ip = match resolve_billing_bind_ipv4(&setup.billing.bind_ip) {
            Ok(address) => address,
            Err(failure) => {
                self.close_and_remove_billing_client();
                return GameClientInitialization::Failed {
                    attempts: vec![GameConnectAttempt {
                        direction: GameUpstreamDirection::BillingPrimary,
                        failure: Some(failure),
                    }],
                };
            }
        };
        let mut attempts = Vec::with_capacity(2);
        let plans = [
            (
                &setup.billing.primary,
                GameUpstreamDirection::BillingPrimary,
            ),
            (&setup.billing.backup, GameUpstreamDirection::BillingBackup),
        ];
        let mut connected = None;
        for (endpoint_plan, direction) in plans {
            let result = connect_game_client(
                self.billing_client
                    .as_mut()
                    .expect("Billing client остаётся опубликованным между попытками"),
                endpoint_plan,
                direction,
                Some(bind_ip),
                setup.billing.bind_port,
            )
            .await;
            match result {
                Ok(endpoint) => {
                    attempts.push(GameConnectAttempt {
                        direction,
                        failure: None,
                    });
                    connected = Some((endpoint, direction));
                    break;
                }
                Err(failure) => attempts.push(GameConnectAttempt {
                    direction,
                    failure: Some(failure),
                }),
            }
        }
        let Some((endpoint, direction)) = connected else {
            self.close_and_remove_billing_client();
            return GameClientInitialization::Failed { attempts };
        };

        self.billing_client
            .as_mut()
            .expect("подключённый Billing client остаётся опубликованным")
            .enable_control_send();
        let registration = CMessage::new(BILLING_REGISTRATION).send_to_bs(self, false);
        GameClientInitialization::Connected {
            endpoint,
            used_billing_backup: direction == GameUpstreamDirection::BillingBackup,
            registration,
            attempts,
        }
    }

    /// Подключает новый World owner и публикует typed reconnect event.
    pub(crate) async fn reconnect_world_server(&self) -> GameReconnectPublication {
        let Some(setup) = self.network_setup.as_ref().cloned() else {
            return missing_setup_reconnect(GameUpstreamDirection::World);
        };
        let publisher = self.net_server.as_ref().map(CMyNetServer::event_publisher);
        reconnect_world_with(setup, publisher).await
    }

    /// Подключает новый Billing owner и публикует typed reconnect event.
    pub(crate) async fn reconnect_billing_server(&self) -> GameReconnectPublication {
        let Some(setup) = self.network_setup.as_ref().cloned() else {
            return missing_setup_reconnect(GameUpstreamDirection::BillingPrimary);
        };
        let publisher = self.net_server.as_ref().map(CMyNetServer::event_publisher);
        reconnect_billing_with(setup, publisher).await
    }

    /// Останавливает прежний World worker, закрывает текущий канал и запускает retry entry.
    pub(crate) async fn create_connect_world_task(
        &mut self,
    ) -> Result<(), GameReconnectTaskStartError> {
        let setup = self
            .network_setup
            .as_ref()
            .cloned()
            .ok_or(GameReconnectTaskStartError::MissingNetworkSetup)?;
        let publisher = self
            .net_server
            .as_ref()
            .map(CMyNetServer::event_publisher)
            .ok_or(GameReconnectTaskStartError::MissingNetworkServerOwner)?;

        stop_reconnect_task(&mut self.world_reconnect_task).await;
        if let Some(client) = self.world_client.as_mut() {
            client.disable_control_send();
            let _legacy_result = client.close();
        }

        let exit_requested = Arc::new(AtomicBool::new(false));
        let worker_exit = Arc::clone(&exit_requested);
        let handle =
            tokio::spawn(
                async move { run_world_reconnect_task(setup, publisher, worker_exit).await },
            );
        self.world_reconnect_task = Some(GameReconnectTask {
            exit_requested,
            handle,
        });
        Ok(())
    }

    /// Останавливает прежний Billing worker, закрывает текущий канал и запускает retry entry.
    pub(crate) async fn create_connect_billing_task(
        &mut self,
    ) -> Result<(), GameReconnectTaskStartError> {
        let setup = self
            .network_setup
            .as_ref()
            .cloned()
            .ok_or(GameReconnectTaskStartError::MissingNetworkSetup)?;
        let publisher = self
            .net_server
            .as_ref()
            .map(CMyNetServer::event_publisher)
            .ok_or(GameReconnectTaskStartError::MissingNetworkServerOwner)?;

        stop_reconnect_task(&mut self.billing_reconnect_task).await;
        if let Some(client) = self.billing_client.as_mut() {
            client.disable_control_send();
            let _legacy_result = client.close();
        }

        let exit_requested = Arc::new(AtomicBool::new(false));
        let worker_exit = Arc::clone(&exit_requested);
        let handle =
            tokio::spawn(
                async move { run_billing_reconnect_task(setup, publisher, worker_exit).await },
            );
        self.billing_reconnect_task = Some(GameReconnectTask {
            exit_requested,
            handle,
        });
        Ok(())
    }

    /// Материализует начальный порядок остановки reconnect workers из `Release`.
    pub(crate) async fn stop_reconnect_tasks(&mut self) {
        stop_reconnect_task(&mut self.world_reconnect_task).await;
        stop_reconnect_task(&mut self.billing_reconnect_task).await;
    }

    /// Полный достигнутый `Release` teardown. Manual deletes заменены Drop и
    /// `take/clear`, а отсутствующие process-global owners вызываются строго в
    /// исходной позиции через runtime; неизвестный legacy int не выдумывается.
    pub(crate) async fn release<Runtime: GameThreadRuntime>(
        &mut self,
        runtime: &mut Runtime,
    ) -> GameReleaseReport {
        let mut events = Vec::new();

        let debug = GameReleaseDebug::ServerExiting;
        GameReleaseRuntime::put_debug_string(runtime, debug.clone());
        events.push(GameReleaseEvent::Debug(debug));

        self.stop_reconnect_tasks().await;
        events.push(GameReleaseEvent::ReconnectTasksStopped);

        let player_ids: Vec<i32> = self.players.keys().copied().collect();
        let total_players = player_ids.len();
        for (index, player_id) in player_ids.into_iter().enumerate() {
            let mut snapshot = Vec::new();
            let encoded = self
                .players
                .get(&player_id)
                .is_some_and(|player| self.encode_player_game_save(player, &mut snapshot, runtime));
            let saved = encoded
                && runtime.publish_player_save(
                    player_id,
                    GAME_RELEASE_PLAYER_SAVE_MESSAGE,
                    1,
                    &snapshot,
                );
            if !saved {
                let debug = GameReleaseDebug::PlayerSaveFailed {
                    player_id,
                    processed: index,
                    total: total_players,
                };
                GameReleaseRuntime::put_debug_string(runtime, debug.clone());
                events.push(GameReleaseEvent::Debug(debug));
                // Safe Result-граница заменяет native exception catch, который
                // немедленно erase-ил проблемный map node и продолжал обход.
                self.players.remove(&player_id);
            }
            events.push(GameReleaseEvent::PlayerSave { player_id, saved });
        }
        let debug = GameReleaseDebug::PlayersSaved {
            processed: total_players,
            total: total_players,
        };
        GameReleaseRuntime::put_debug_string(runtime, debug.clone());
        events.push(GameReleaseEvent::Debug(debug));

        runtime.save_city_region(self, 0);
        events.push(GameReleaseEvent::CityRegionSaved);
        let debug = GameReleaseDebug::CityRegionSaved;
        GameReleaseRuntime::put_debug_string(runtime, debug.clone());
        events.push(GameReleaseEvent::Debug(debug));

        let players = self.players.len();
        self.players.clear();
        events.push(GameReleaseEvent::PlayersCleared { count: players });
        let regions = self.regions.len();
        self.regions.clear();
        events.push(GameReleaseEvent::RegionsCleared { count: regions });
        let debug = GameReleaseDebug::PlayersAndRegionsCleared;
        GameReleaseRuntime::put_debug_string(runtime, debug.clone());
        events.push(GameReleaseEvent::Debug(debug));

        let proxy_regions = self.proxy_regions.len();
        self.proxy_regions.clear();
        events.push(GameReleaseEvent::ProxyRegionsCleared {
            count: proxy_regions,
        });
        let debug = GameReleaseDebug::ProxyRegionsCleared;
        GameReleaseRuntime::put_debug_string(runtime, debug.clone());
        events.push(GameReleaseEvent::Debug(debug));

        let function_list = self.function_list_file_data.take().is_some();
        let variable_list = self.variable_list_file_data.take().is_some();
        let script_files = self.script_file_data.len();
        self.script_file_data.clear();
        let active_scripts = self.active_scripts.len();
        self.active_scripts.clear();
        let function_registry = self.script_functions.release();
        let general_variables = self.general_variables.release();
        events.push(GameReleaseEvent::ScriptDataCleared {
            function_list,
            variable_list,
            files: script_files,
            active_scripts,
            function_registry,
            general_variables,
        });
        let debug = GameReleaseDebug::ScriptDataCleared;
        GameReleaseRuntime::put_debug_string(runtime, debug.clone());
        events.push(GameReleaseEvent::Debug(debug));

        self.skill_factory.clear_skill_cache();
        events.push(GameReleaseEvent::SkillFactoryCleared);
        self.goods_factory.release();
        events.push(GameReleaseEvent::GoodsFactoryReleased);

        let network_server_present = self.net_server.is_some();
        if let Some(server) = self.net_server.as_mut() {
            runtime.exit_network_server_worker(server);
        }
        events.push(GameReleaseEvent::NetworkServerWorkerStopped {
            present: network_server_present,
        });

        let world_client_present = self.world_client.is_some();
        if let Some(client) = self.world_client.as_mut() {
            client.disable_control_send();
            let _legacy_result = client.close();
        }
        let billing_client_present = self.billing_client.is_some();
        if let Some(client) = self.billing_client.as_mut() {
            client.disable_control_send();
            let _legacy_result = client.close();
        }
        self.world_client = None;
        events.push(GameReleaseEvent::WorldClientReleased {
            present: world_client_present,
        });
        self.billing_client = None;
        events.push(GameReleaseEvent::BillingClientReleased {
            present: billing_client_present,
        });
        let network_server_present = self.net_server.take().is_some();
        events.push(GameReleaseEvent::NetworkServerReleased {
            present: network_server_present,
        });

        runtime.release_external_owner(GameReleaseExternalOwner::SocketRuntime);
        events.push(GameReleaseEvent::ExternalOwner(
            GameReleaseExternalOwner::SocketRuntime,
        ));
        let released_net_sessions = self.net_session_manager.release();
        events.push(GameReleaseEvent::NetSessionsReleased {
            count: released_net_sessions,
        });

        self.increment_shop_list.release();
        events.push(GameReleaseEvent::IncrementShopReleased);
        runtime.release_external_owner(GameReleaseExternalOwner::PkSystem);
        events.push(GameReleaseEvent::ExternalOwner(
            GameReleaseExternalOwner::PkSystem,
        ));
        self.quest_system = CQuestSystem::default();
        events.push(GameReleaseEvent::QuestSystemReleased);
        runtime.release_external_owner(GameReleaseExternalOwner::BaseMessageRuntime);
        events.push(GameReleaseEvent::ExternalOwner(
            GameReleaseExternalOwner::BaseMessageRuntime,
        ));

        let sequence_count = self.sequence_registry.len();
        self.sequence_registry.clear();
        events.push(GameReleaseEvent::SequenceRegistryCleared {
            count: sequence_count,
        });
        let player_ranks_present = self.player_ranks.take().is_some();
        events.push(GameReleaseEvent::PlayerRanksReleased {
            present: player_ranks_present,
        });
        self.words_filter.clear();
        events.push(GameReleaseEvent::WordsFilterReleased);
        self.honor_ranks = CHonorRanks::default();
        events.push(GameReleaseEvent::HonorRanksReleased);
        let goods_war_present = self.goods_war.take().is_some();
        events.push(GameReleaseEvent::GoodsWarReleased {
            present: goods_war_present,
        });
        self.country_handler = CCountryHandler::default();
        events.push(GameReleaseEvent::CountryHandlerReleased);
        self.country_param = CCountryParam::default();
        events.push(GameReleaseEvent::CountryParamReleased);

        let attack_city_schedules = self.attack_city_sys.attacks.len();
        self.attack_city_sys = CAttackCitySys::default();
        events.push(GameReleaseEvent::AttackCitySystemReleased {
            schedules: attack_city_schedules,
        });
        let village_war_schedules = self.village_war_sys.village_wars.len();
        self.village_war_sys = CVillageWarSys::default();
        events.push(GameReleaseEvent::VillageWarSystemReleased {
            schedules: village_war_schedules,
        });

        let debug = GameReleaseDebug::ServerExited;
        GameReleaseRuntime::put_debug_string(runtime, debug.clone());
        events.push(GameReleaseEvent::Debug(debug));
        GameReleaseReport {
            events,
            legacy_return: None,
        }
    }

    /// Выполняет один awaitable I/O шаг текущего World направления.
    pub(crate) async fn run_world_io_once(
        &mut self,
    ) -> Result<GameClientIoStep, GameClientIoError> {
        self.world_client
            .as_mut()
            .ok_or(GameClientIoError::NotConnected)?
            .run_io_once()
            .await
    }

    /// Выполняет один awaitable I/O шаг текущего Billing направления.
    pub(crate) async fn run_billing_io_once(
        &mut self,
    ) -> Result<GameClientIoStep, GameClientIoError> {
        self.billing_client
            .as_mut()
            .ok_or(GameClientIoError::NotConnected)?
            .run_io_once()
            .await
    }

    fn close_and_remove_world_client(&mut self) -> bool {
        let mut client = self.world_client.take();
        if let Some(client) = client.as_mut() {
            let _legacy_result = client.close();
        }
        client.is_some()
    }

    fn close_and_remove_billing_client(&mut self) -> bool {
        let mut client = self.billing_client.take();
        if let Some(client) = client.as_mut() {
            let _legacy_result = client.close();
        }
        client.is_some()
    }

    pub(crate) fn register_player(&mut self, mut player: CPlayer) -> Option<CPlayer> {
        player.initialize_variable_list(self.variable_list_file_data.as_deref());
        self.players.insert(player.player_id(), player)
    }

    pub(crate) fn discard_player_login(&mut self, player_id: i32) -> bool {
        let removed = self.players.remove(&player_id).is_some();
        let _ = self.net_server().clear_player_map_id(player_id);
        removed
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn complete_world_player_login<
        Context: GamePlayerLoginContext + ScriptFunctionRuntime,
    >(
        &mut self,
        expected_player_id: i32,
        mut player: CPlayer,
        captain: bool,
        team_id: i32,
        context: &mut Context,
    ) -> Result<GamePlayerLoginReport, GamePlayerLoginBlock> {
        let decoded = player.player_id();
        if decoded != expected_player_id {
            return Err(GamePlayerLoginBlock::PlayerIdMismatch {
                expected: expected_player_id,
                decoded,
            });
        }
        if self.players.contains_key(&expected_player_id) {
            return Err(GamePlayerLoginBlock::AlreadyRegistered {
                player_id: expected_player_id,
            });
        }
        player.restore_login_team(captain, team_id);
        let region_id = player.server_region_id().unwrap_or_default();
        self.players.insert(expected_player_id, player);

        let mut player = self
            .players
            .remove(&expected_player_id)
            .expect("login player только что зарегистрирован");
        let Some(mut owner) = self.take_region_owner(region_id) else {
            self.players.insert(expected_player_id, player);
            return Err(GamePlayerLoginBlock::MissingRegion { region_id });
        };
        let mut relocation = None;
        if player.shape().get_pos_x() == -0.5 && player.shape().get_pos_y() == -0.5 {
            if let Ok(position) = owner.base().region.get_random_pos(context) {
                player
                    .movement_shape_mut()
                    .set_pos_xy_base(position.x as f32 + 0.5, position.y as f32 + 0.5);
                relocation = Some((position.x, position.y));
            }
        }
        let tile = (
            player.shape().get_tile_x().unwrap_or_default(),
            player.shape().get_tile_y().unwrap_or_default(),
        );
        if owner
            .base()
            .region
            .get_block(tile.0, tile.1)
            .unwrap_or_default()
            != 0
            && let Ok(position) = owner
                .base()
                .region
                .get_random_pos_in_range(tile.0, tile.1, 3, 3, context)
        {
            player
                .movement_shape_mut()
                .set_pos_xy_base(position.x as f32 + 0.5, position.y as f32 + 0.5);
            relocation = Some((position.x, position.y));
        }
        let facts = ShapeRuntimeFacts {
            is_player: true,
            monster: None,
            is_npc: false,
            goods: None,
            is_move_shape: true,
            figure: player.figure(),
        };
        let membership = owner.base_mut().add_object(
            player.movement_shape_mut(),
            facts,
            self.globe_setup.area_width(),
            self.globe_setup.area_height(),
            context.now_milliseconds(),
            context,
        );
        self.restore_region_owner(owner);
        self.players.insert(expected_player_id, player);
        membership.map_err(GamePlayerLoginBlock::Membership)?;

        let login_tick_ms = context.now_milliseconds();
        let loaded_change_body_states = self
            .players
            .get_mut(&expected_player_id)
            .expect("spatial login сохраняет player map owner")
            .activate_loaded_change_body_states(login_tick_ms);
        let loaded_ride_state = self
            .players
            .get_mut(&expected_player_id)
            .expect("spatial login сохраняет player map owner")
            .activate_loaded_ride_state();
        let loaded_extended_states = self
            .players
            .get_mut(&expected_player_id)
            .expect("spatial login сохраняет player map owner")
            .activate_loaded_extended_states(login_tick_ms);
        let loaded_appellation_states = self
            .players
            .get_mut(&expected_player_id)
            .expect("spatial login сохраняет player map owner")
            .activate_loaded_appellation_states(login_tick_ms);
        for state in &loaded_appellation_states {
            self.send_appellation_visual(expected_player_id, state, true, login_tick_ms);
        }
        for state in &loaded_extended_states {
            self.send_extended_state_visual(expected_player_id, state, true, login_tick_ms);
        }
        for state in &loaded_change_body_states {
            self.send_change_body_visual(expected_player_id, state, true);
        }
        if let Some(state) = &loaded_ride_state {
            self.send_ride_visual(expected_player_id, state, true);
        }

        let first_login = self
            .players
            .get_mut(&expected_player_id)
            .expect("spatial login сохраняет player map owner")
            .mark_login_script_started();
        let login_script_id = if first_login {
            let path = self.quest_system.player_login_script.clone();
            self.run_script_file(
                &path,
                ScriptExecutionContext {
                    player_id: Some(expected_player_id),
                    region_id: Some(region_id),
                    ..ScriptExecutionContext::default()
                },
                context,
            )
        } else {
            None
        };
        let recomputed = context.recompute_login_player_properties(
            self.players
                .get(&expected_player_id)
                .expect("login script не удаляет player owner"),
        );
        let coefficients = self.globe_setup.player_property_coefficients();
        let goods_factory = self.goods_factory.clone();
        self.players
            .get_mut(&expected_player_id)
            .expect("login property callback не удаляет player owner")
            .apply_change_body_properties(recomputed, coefficients, &goods_factory);
        let client_deliveries =
            context.publish_initial_player_client_snapshot(self, expected_player_id, first_login);
        let mut billing = CMessage::new(0x000e_f201);
        add_legacy_c_string(
            billing.base_mut(),
            self.players
                .get(&expected_player_id)
                .expect("client snapshot сохраняет player owner")
                .account(),
        );
        billing.add_long(expected_player_id);
        let billing_delivery = billing.send_to_bs(self, false).unwrap_or_default();
        let adjust_honor = self.globe_setup.use_appellation_function()
            && self
                .players
                .get(&expected_player_id)
                .is_some_and(|player| player.honor_snapshot().rank_of_nobility_id != 0);
        let honor_script_id = adjust_honor
            .then(|| {
                self.run_script_file(
                    b"scripts/circle/honorrank/adjusthonorrank.script",
                    ScriptExecutionContext {
                        player_id: Some(expected_player_id),
                        region_id: Some(region_id),
                        ..ScriptExecutionContext::default()
                    },
                    context,
                )
            })
            .flatten();

        let mut goods_ai_registrations = 0usize;
        let mut pending_equipment_state_updates = Vec::new();
        {
            let (players, goods_factory) = (&mut self.players, &self.goods_factory);
            let player = players
                .get_mut(&expected_player_id)
                .expect("honor script scheduling сохраняет player owner");
            player.visit_login_goods_mut(|location, goods| {
                context.register_login_goods_ai(expected_player_id, goods);
                goods_ai_registrations += 1;
                if !matches!(
                    location,
                    PlayerLoginGoodsLocation::Equipment
                        | PlayerLoginGoodsLocation::Packet
                        | PlayerLoginGoodsLocation::Depot
                ) || goods.addon_property_value(goods_factory, GAP_EQUIP_STATE, 1) != 2
                {
                    return true;
                }
                let packed_expiration =
                    goods.addon_property_value(goods_factory, GAP_EQUIP_STATE, 2);
                if packed_expiration == 0 {
                    return false;
                }
                if context.login_equipment_state_expired(packed_expiration) {
                    let _ = goods.set_addon_property_modifier_core(GAP_EQUIP_STATE, 1, 3);
                    pending_equipment_state_updates.push((
                        location,
                        goods.identity(),
                        context.encode_goods_for_old_client(goods),
                    ));
                }
                true
            });
        }
        let mut equipment_state_updates = Vec::new();
        for (location, goods, payload) in pending_equipment_state_updates {
            let mut update = CMessage::new(0x000b_f928);
            update.add_long(expected_player_id);
            update.base_mut().add_guid(goods.ex_id);
            update.add_ulong(payload.len() as u32);
            update.base_mut().add(&payload);
            let delivery = if location == PlayerLoginGoodsLocation::Depot {
                self.send_player_shape_around(expected_player_id, None, &update)
                    .and_then(Result::ok)
                    .unwrap_or_default()
            } else {
                update.send_to_player(self.net_server(), expected_player_id)
            };
            equipment_state_updates.push(PlayerLoginEquipmentStateUpdate {
                location,
                goods,
                delivery,
            });
        }

        Ok(GamePlayerLoginReport {
            player_id: expected_player_id,
            team_id,
            captain,
            region_id,
            first_login,
            relocation,
            login_script_id,
            client_deliveries,
            billing_delivery,
            honor_script_id,
            goods_ai_registrations,
            equipment_state_updates,
        })
    }

    /// Exact `s_mapPlayer.size()` для GMA `0x80002`; x86 `size_type` — DWORD.
    pub(crate) fn player_count(&self) -> u32 {
        u32::try_from(self.players.len()).expect("x86 player map не может превысить DWORD")
    }

    pub(crate) fn reset_total_honor_eliminate(
        &mut self,
        reset_mask: u32,
    ) -> Vec<PlayerHonorResetReport> {
        self.players
            .values_mut()
            .map(|player| player.reset_total_honor_eliminate(reset_mask))
            .collect()
    }

    pub(crate) fn register_team_session(&mut self, team_id: u32, session_id: i32) -> Option<i32> {
        self.team_session_ids.insert(team_id, session_id)
    }

    pub(crate) fn get_team_session_id(&self, team_id: u32) -> i32 {
        self.team_session_ids.get(&team_id).copied().unwrap_or(0)
    }

    pub(crate) fn find_player(&self, player_id: i32) -> Option<&CPlayer> {
        self.players.get(&player_id)
    }

    pub(crate) fn find_player_mut(&mut self, player_id: i32) -> Option<&mut CPlayer> {
        self.players.get_mut(&player_id)
    }

    pub(crate) fn set_player_yuan_bao(
        &mut self,
        player_id: i32,
        current: u32,
        created_currency: Vec<CGoods>,
    ) -> Option<PlayerYuanBaoChange> {
        let (players, goods_factory) = (&mut self.players, &self.goods_factory);
        players
            .get_mut(&player_id)
            .map(|player| player.set_yuan_bao(current, goods_factory, created_currency))
    }

    pub(crate) fn increase_player_auction_money(
        &mut self,
        player_id: i32,
        requested: u32,
        created_currency: Vec<CGoods>,
    ) -> Option<PlayerAuctionMoneyChange> {
        let (players, goods_factory) = (&mut self.players, &self.goods_factory);
        players
            .get_mut(&player_id)
            .map(|player| player.increase_auction_money(requested, goods_factory, created_currency))
    }

    pub(crate) fn return_player_auction_goods(
        &mut self,
        player_id: i32,
        goods: CGoods,
        bind_type: i32,
    ) -> Option<PlayerAuctionGoodsReturn> {
        let (players, goods_factory) = (&mut self.players, &self.goods_factory);
        players
            .get_mut(&player_id)?
            .return_auction_goods(goods, bind_type, goods_factory)
    }

    pub(crate) fn add_increment_shop_goods_to_packet(
        &mut self,
        player_id: i32,
        goods: Vec<CGoods>,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Option<(Vec<CiQingPacketAddition>, Vec<CGoods>)> {
        let (players, goods_factory) = (&mut self.players, &self.goods_factory);
        players.get_mut(&player_id).map(|player| {
            player.add_increment_shop_goods_to_packet(goods, goods_factory, encode_old_client)
        })
    }

    pub(crate) fn add_npc_shop_goods_to_packet<Context: OldClientGoodsCodec>(
        &mut self,
        player_id: i32,
        goods: Vec<CGoods>,
        context: &mut Context,
    ) -> Option<(Vec<CiQingPacketAddition>, Vec<CGoods>)> {
        let (players, goods_factory) = (&mut self.players, &self.goods_factory);
        let player = players.get_mut(&player_id)?;
        let mut encode = |goods: &CGoods| context.encode_goods_for_old_client(goods);
        Some(player.add_shop_goods_to_packet(goods, goods_factory, &mut encode))
    }

    pub(crate) fn increase_player_money<Context: OldClientGoodsCodec>(
        &mut self,
        player_id: i32,
        amount: u32,
        context: &mut Context,
    ) -> Option<(
        crate::gameserver::appserver::container::cwallet::CurrencyIncreaseOutcome,
        Vec<i32>,
    )> {
        let created = self.create_goods_batch(self.goods_factory.get_gold_coin_index(), amount);
        let outcome = {
            let (players, goods_factory) = (&mut self.players, &self.goods_factory);
            players
                .get_mut(&player_id)?
                .increase_money(amount, goods_factory, created)
        };
        let deliveries = self.send_player_money_increase(player_id, &outcome, context);
        Some((outcome, deliveries))
    }

    pub(crate) fn add_region_tax(
        &mut self,
        region_id: i32,
        amount: u32,
    ) -> Vec<Result<i32, SendMessageError>> {
        let mut deliveries = Vec::new();
        let mut pending = vec![(region_id, amount)];
        let mut visited = BTreeSet::new();
        let mut updates = Vec::new();
        while let Some((current_id, current_amount)) = pending.pop() {
            if !visited.insert(current_id) {
                let mut transfer = CMessage::new(0x0006_012e);
                transfer.add_long(current_id);
                transfer.add_ulong(current_amount);
                deliveries.push(transfer.send(self, false));
                continue;
            }
            let Some(stage) = self
                .find_region_mut(current_id)
                .map(|region| region.base_mut().add_tax_money(current_amount))
            else {
                let mut transfer = CMessage::new(0x0006_012e);
                transfer.add_long(current_id);
                transfer.add_ulong(current_amount);
                deliveries.push(transfer.send(self, false));
                continue;
            };
            updates.push(stage);
            if let Some(parent_id) = stage
                .superior_region_id
                .filter(|_| stage.superior_share != 0)
            {
                if self.find_region(parent_id).is_some() && !visited.contains(&parent_id) {
                    pending.push((parent_id, stage.superior_share));
                } else {
                    let mut transfer = CMessage::new(0x0006_012e);
                    transfer.add_long(parent_id);
                    transfer.add_ulong(stage.superior_share);
                    deliveries.push(transfer.send(self, false));
                }
            }
        }
        for stage in updates.into_iter().rev() {
            let mut update = CMessage::new(0x0006_012d);
            update.add_long(stage.region_id);
            update.add_ulong(stage.today_total_tax);
            update.add_ulong(stage.total_tax);
            update.add_long(stage.current_tax_rate);
            deliveries.push(update.send(self, false));
        }
        deliveries
    }

    pub(crate) fn add_goods_to_player_packet(
        &mut self,
        player_id: i32,
        goods: Vec<CGoods>,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Option<(Vec<CiQingPacketAddition>, Vec<CGoods>)> {
        let (players, goods_factory) = (&mut self.players, &self.goods_factory);
        players
            .get_mut(&player_id)
            .map(|player| player.add_goods_to_packet(goods, goods_factory, encode_old_client))
    }

    /// Exact `CGame::KickPlayer`: queue side effect сохраняется, публичный
    /// bool исходника всегда остаётся `false`.
    pub(crate) fn kick_player(&self, player_id: i32) -> GameKickPlayerReport {
        let command_result = self.net_server().command_handle().quit_by_map_id(player_id);
        GameKickPlayerReport {
            player_id,
            command_result,
            legacy_return: false,
        }
    }

    /// Exact `OnGMMessage 0x7FC09` recipient pass: requester остаётся,
    /// остальные canonical player ID закрываются в signed map-order.
    pub(crate) fn kick_players_except(
        &self,
        preserved_player_id: i32,
    ) -> Vec<GameKickPlayerReport> {
        self.players
            .keys()
            .copied()
            .filter(|player_id| *player_id != preserved_player_id)
            .map(|player_id| self.kick_player(player_id))
            .collect()
    }

    /// Exact `OnOtherMessage 0x7FA0C`: snapshot signed player-map order и
    /// отдельный `KickPlayer` network command для каждого текущего owner-а.
    pub(crate) fn kick_all_players(&self) -> Vec<GameKickPlayerReport> {
        self.players
            .keys()
            .copied()
            .map(|player_id| self.kick_player(player_id))
            .collect()
    }

    /// Exact `OnGMMessage 0x7FC0A`: region последовательно обходит все area,
    /// а найденные `CPlayer` передаются в `KickPlayer` в исходном порядке.
    pub(crate) fn kick_players_in_region_except(
        &self,
        region_id: i32,
        preserved_player_id: i32,
    ) -> Vec<GameKickPlayerReport> {
        let Some(region) = self.regions.get(&region_id) else {
            return Vec::new();
        };
        let mut player_ids = Vec::new();
        region.base().find_all_player_ids(&mut player_ids);
        player_ids
            .into_iter()
            .filter(|player_id| *player_id != preserved_player_id)
            .map(|player_id| self.kick_player(player_id))
            .collect()
    }

    /// Exact `OnGMMessage 0x7FC07` до network response: X-major 7×7 scan
    /// использует region `GetShape`, а `Vec::dedup` повторяет именно
    /// consecutive-only семантику исходного `std::list::unique`.
    pub(crate) fn kick_players_around_name(&self, name: &[u8]) -> GameKickAroundReport {
        let Some(player) = self.find_player_by_name(name) else {
            return game_kick_around_block(GameKickAroundOutcome::TargetMissing, None, None);
        };
        let target_player_id = player.player_id();
        let Some(server_region_id) = player.server_region_id() else {
            return game_kick_around_block(
                GameKickAroundOutcome::ServerRegionMissing,
                Some(target_player_id),
                None,
            );
        };
        let tile_x = match player.shape().get_tile_x() {
            Ok(tile_x) => tile_x,
            Err(block) => {
                return game_kick_around_block(
                    GameKickAroundOutcome::CoordinateBlocked(block),
                    Some(target_player_id),
                    Some(server_region_id),
                );
            }
        };
        let tile_y = match player.shape().get_tile_y() {
            Ok(tile_y) => tile_y,
            Err(block) => {
                return game_kick_around_block(
                    GameKickAroundOutcome::CoordinateBlocked(block),
                    Some(target_player_id),
                    Some(server_region_id),
                );
            }
        };
        let Some(region) = self.regions.get(&server_region_id) else {
            return game_kick_around_block(
                GameKickAroundOutcome::RegionMissing,
                Some(target_player_id),
                Some(server_region_id),
            );
        };
        let region = region.base();
        if let Some(identity) = region
            .registered_shape_identities()
            .into_iter()
            .find(|identity| self.resolve_shape(*identity).is_none())
        {
            return game_kick_around_block(
                GameKickAroundOutcome::UnresolvedShape(identity),
                Some(target_player_id),
                Some(server_region_id),
            );
        }
        let window_x = gm_kick_window_origin(tile_x, region.region.width);
        let window_y = gm_kick_window_origin(tile_y, region.region.height);
        let (area_width, area_height) = self.area_dimensions();
        let mut matched_player_ids = Vec::new();
        let window_end_x = window_x.wrapping_add(7);
        let window_end_y = window_y.wrapping_add(7);
        let mut scan_x = window_x;
        while scan_x < window_end_x {
            let mut scan_y = window_y;
            while scan_y < window_end_y {
                let shape = match region.get_shape(scan_x, scan_y, area_width, area_height, self) {
                    Ok(shape) => shape,
                    Err(block) => {
                        return GameKickAroundReport {
                            outcome: GameKickAroundOutcome::LookupBlocked(block),
                            target_player_id: Some(target_player_id),
                            server_region_id: Some(server_region_id),
                            window_origin: Some((window_x, window_y)),
                            matched_player_ids,
                            kicks: Vec::new(),
                        };
                    }
                };
                if let Some(shape) = shape.filter(|shape| shape.identity.object_type == PLAYER_TYPE)
                {
                    matched_player_ids.push(shape.identity.id);
                }
                scan_y = scan_y.wrapping_add(1);
            }
            scan_x = scan_x.wrapping_add(1);
        }
        matched_player_ids.dedup();
        let kicks = matched_player_ids
            .iter()
            .copied()
            .map(|player_id| self.kick_player(player_id))
            .collect();
        GameKickAroundReport {
            outcome: GameKickAroundOutcome::Completed,
            target_player_id: Some(target_player_id),
            server_region_id: Some(server_region_id),
            window_origin: Some((window_x, window_y)),
            matched_player_ids,
            kicks,
        }
    }

    /// Exact X-major 7×7 `SetPlayerRegionEx` target snapshot. Как исходный
    /// `GetShape`, каждая клетка даёт не более одного player; consecutive
    /// повторы снимаются до последующих `ChangeRegion` side effects.
    pub(crate) fn script_players_around_name(&self, name: &[u8]) -> Vec<i32> {
        let Some(player) = self.find_player_by_name(name) else {
            return Vec::new();
        };
        let (Some(region_id), Ok(tile_x), Ok(tile_y)) = (
            player.server_region_id(),
            player.shape().get_tile_x(),
            player.shape().get_tile_y(),
        ) else {
            return Vec::new();
        };
        let Some(region) = self.regions.get(&region_id).map(ServerRegionOwner::base) else {
            return Vec::new();
        };
        let window_x = gm_kick_window_origin(tile_x, region.region.width);
        let window_y = gm_kick_window_origin(tile_y, region.region.height);
        let (area_width, area_height) = self.area_dimensions();
        let mut player_ids = Vec::new();
        let mut scan_x = window_x;
        while scan_x < window_x.wrapping_add(7) {
            let mut scan_y = window_y;
            while scan_y < window_y.wrapping_add(7) {
                let Ok(shape) = region.get_shape(scan_x, scan_y, area_width, area_height, self)
                else {
                    return Vec::new();
                };
                if let Some(shape) = shape.filter(|shape| shape.identity.object_type == PLAYER_TYPE)
                {
                    player_ids.push(shape.identity.id);
                }
                scan_y = scan_y.wrapping_add(1);
            }
            scan_x = scan_x.wrapping_add(1);
        }
        player_ids.dedup();
        player_ids
    }

    /// Exact name lookup + `KickPlayer` side effect для GM `0x7FC06`.
    pub(crate) fn kick_player_by_name(&self, name: &[u8]) -> Option<GameKickPlayerReport> {
        let player_id = self.find_player_by_name(name)?.player_id();
        Some(self.kick_player(player_id))
    }

    fn send_script_shape_exit_around(&self, region: &CServerRegion, shape: &CShape) -> Option<i32> {
        let runtime = GameServerAroundRuntime::new(
            self,
            &self.session_factory,
            self.globe_setup.area_width(),
            self.globe_setup.area_height(),
        )?;
        let identity = shape.identity();
        let mut message = CMessage::new(0x000b_f504);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.add_long(0);
        message
            .send_to_around(Some(region), shape, None, &runtime)
            .ok()
    }

    /// Exact script `3303 / DeleteNpc`: current player region lookup,
    /// `0xBF504(type,id,0)` around publication и только затем spatial removal.
    pub(crate) fn delete_script_npc(&mut self, player_id: i32, npc_id: i32) -> Option<i32> {
        let region_id = self.find_player(player_id)?.server_region_id()?;
        let mut owner = self.take_region_owner(region_id)?;
        let result = (|| {
            let npc = owner.base().find_npc_by_id(npc_id)?;
            let delivery =
                self.send_script_shape_exit_around(owner.base(), npc.move_shape().shape())?;
            owner
                .base_mut()
                .remove_owned_npc_by_id(npc_id)
                .ok()?
                .then_some(delivery)
        })();
        self.restore_region_owner(owner);
        result
    }

    /// Exact script `3306 / DeleteMonster` не запускает death owner: после
    /// around-exit monster лишь получает `CS_DELETE` для обычного AI cleanup.
    pub(crate) fn delete_script_monster(&mut self, player_id: i32, monster_id: i32) -> Option<i32> {
        let region_id = self.find_player(player_id)?.server_region_id()?;
        let mut owner = self.take_region_owner(region_id)?;
        let result = (|| {
            let monster = owner.base().find_monster_by_id(monster_id)?;
            let delivery =
                self.send_script_shape_exit_around(owner.base(), monster.move_shape().shape())?;
            owner
                .base_mut()
                .find_monster_by_id_mut(monster_id)
                .expect("around publication сохраняет owned monster")
                .stage_for_delete();
            Some(delivery)
        })();
        self.restore_region_owner(owner);
        result
    }

    /// Script `3315` использует explicit/source region уже после вычисления
    /// имени; ambiguous duplicate-name lookup безопасно остаётся no-op.
    pub(crate) fn delete_script_npc_by_name(&mut self, region_id: i32, name: &[u8]) -> Option<i32> {
        let mut owner = self.take_region_owner(region_id)?;
        let result = (|| {
            let npc = owner.base().find_npc_by_name(name).ok()??;
            let npc_id = npc.move_shape().shape().identity().id;
            let delivery =
                self.send_script_shape_exit_around(owner.base(), npc.move_shape().shape())?;
            owner
                .base_mut()
                .remove_owned_npc_by_id(npc_id)
                .ok()?
                .then_some(delivery)
        })();
        self.restore_region_owner(owner);
        result
    }

    /// Script `3313` сначала фиксирует ordered rectangle snapshot, затем для
    /// каждого совпадения публикует exit и ставит `CS_DELETE`.
    pub(crate) fn delete_script_monsters_in_rect(
        &mut self,
        region_id: i32,
        rectangle: [i32; 4],
        original_name: Option<&[u8]>,
    ) -> Vec<i32> {
        let Some(mut owner) = self.take_region_owner(region_id) else {
            return Vec::new();
        };
        let monster_ids = owner.base().script_monster_ids_in_rect(
            rectangle[0],
            rectangle[1],
            rectangle[2],
            rectangle[3],
            original_name,
        );
        let mut deliveries = Vec::new();
        for monster_id in monster_ids {
            let delivery = owner
                .base()
                .find_monster_by_id(monster_id)
                .and_then(|monster| {
                    self.send_script_shape_exit_around(owner.base(), monster.move_shape().shape())
                });
            let Some(delivery) = delivery else {
                continue;
            };
            owner
                .base_mut()
                .find_monster_by_id_mut(monster_id)
                .expect("rectangle snapshot сохраняет owned monster")
                .stage_for_delete();
            deliveries.push(delivery);
        }
        self.restore_region_owner(owner);
        deliveries
    }

    /// `3301 / NpcTalk` сохраняет caller-supplied display name и публикует
    /// один local-chat frame через обычный shape-around runtime.
    pub(crate) fn script_npc_talk(
        &mut self,
        region_id: i32,
        npc_id: i32,
        name: &[u8],
        text: &[u8],
    ) -> Option<i32> {
        let owner = self.take_region_owner(region_id)?;
        let result = (|| {
            let npc = owner.base().find_npc_by_id(npc_id)?;
            let shape = npc.move_shape().shape();
            let mut message = CMessage::new(0x000b_f801);
            message.add_long(0);
            message.add_long(shape.identity().object_type);
            message.add_long(shape.identity().id);
            message.base_mut().add(name);
            message.add_byte(0);
            message.base_mut().add(text);
            message.add_byte(0);
            let runtime = GameServerAroundRuntime::new(
                self,
                &self.session_factory,
                self.globe_setup.area_width(),
                self.globe_setup.area_height(),
            )?;
            message
                .send_to_around(Some(owner.base()), shape, None, &runtime)
                .ok()
        })();
        self.restore_region_owner(owner);
        result
    }

    /// `3304 / MonsterTalk` сначала выбирает exact-name monsters в девяти
    /// area игрока, затем каждый monster независимо фильтрует получателей по
    /// строгому `abs(dx/dy) < AREA_WIDTH/HEIGHT` и шлёт `0xBF801`.
    pub(crate) fn script_monsters_talk(
        &mut self,
        player_id: i32,
        name: &[u8],
        text: &[u8],
    ) -> Vec<i32> {
        let Some((region_id, player_area_index)) = self
            .find_player(player_id)
            .and_then(|player| Some((player.server_region_id()?, player.shape().area_index()?)))
        else {
            return Vec::new();
        };
        let Some(owner) = self.take_region_owner(region_id) else {
            return Vec::new();
        };
        let monster_ids = owner
            .base()
            .script_monster_ids_around_area(player_area_index);
        let area_width = self.globe_setup.area_width();
        let area_height = self.globe_setup.area_height();
        let mut deliveries = Vec::new();
        if area_width > 0 && area_height > 0 {
            for monster_id in monster_ids {
                let Some(monster) = owner
                    .base()
                    .find_monster_by_id(monster_id)
                    .filter(|monster| monster.display_name() == name)
                else {
                    continue;
                };
                let shape = monster.move_shape().shape();
                let (Ok(tile_x), Ok(tile_y)) = (shape.get_tile_x(), shape.get_tile_y()) else {
                    continue;
                };
                let mut player_ids = Vec::new();
                for offset_x in -1..=1 {
                    for offset_y in -1..=1 {
                        owner.base().find_player_ids_in_area(
                            tile_x / area_width + offset_x,
                            tile_y / area_height + offset_y,
                            &mut player_ids,
                        );
                    }
                }
                let mut message = CMessage::new(0x000b_f801);
                message.add_long(0);
                message.add_long(shape.identity().object_type);
                message.add_long(shape.identity().id);
                message.base_mut().add(monster.display_name());
                message.add_byte(0);
                message.base_mut().add(text);
                message.add_byte(0);
                for target_id in player_ids {
                    let Some(target) = self.find_player(target_id) else {
                        continue;
                    };
                    let (Ok(target_x), Ok(target_y)) =
                        (target.shape().get_tile_x(), target.shape().get_tile_y())
                    else {
                        continue;
                    };
                    if i64::from(target_x).abs_diff(i64::from(tile_x)) < area_width as u64
                        && i64::from(target_y).abs_diff(i64::from(tile_y)) < area_height as u64
                    {
                        deliveries.push(message.send_to_player(self.net_server(), target_id));
                    }
                }
            }
        }
        self.restore_region_owner(owner);
        deliveries
    }

    /// Exact recipient pass `OnGMMessage 0x7FC13`: unsigned `long` country
    /// сравнивается с promoted player byte, обход сохраняет signed ID-order.
    pub(crate) fn player_ids_in_country(&self, country: u32) -> Vec<i32> {
        self.players
            .iter()
            .filter_map(|(player_id, player)| {
                (u32::from(player.country()) == country).then_some(*player_id)
            })
            .collect()
    }

    /// CountryWar start обходит canonical player map один раз и выбирает обе
    /// участвующие страны, не группируя получателей по стране.
    pub(crate) fn player_ids_in_countries(&self, countries: [u8; 2]) -> Vec<i32> {
        self.players
            .iter()
            .filter_map(|(player_id, player)| {
                countries.contains(&player.country()).then_some(*player_id)
            })
            .collect()
    }

    /// Exact `FindPlayer(char const*)`: обходит canonical player map по
    /// signed ID-order и сравнивает byte-exact C-string имя.
    pub(crate) fn find_player_by_name(&self, name: &[u8]) -> Option<&CPlayer> {
        let name = legacy_c_string_prefix(name);
        self.players
            .values()
            .find(|player| legacy_c_string_prefix(player.shape().base_object().get_name()) == name)
    }

    /// Exact `FindPlayerByAccount(char*)`: canonical player map в signed
    /// ID-order и byte-exact C-string comparison загруженного account.
    pub(crate) fn find_player_by_account(&self, account: &[u8]) -> Option<&CPlayer> {
        let account = legacy_c_string_prefix(account);
        self.players
            .values()
            .find(|player| legacy_c_string_prefix(player.account()) == account)
    }

    /// Замыкает GM silence mutation от ordered name lookup до exact
    /// `SetSilence` timestamp. Отсутствующий player не создаёт state.
    pub(crate) fn silence_player_by_name(
        &mut self,
        name: &[u8],
        minutes: i32,
        now_milliseconds: impl FnOnce() -> u32,
    ) -> Option<i32> {
        let player_id = self.find_player_by_name(name)?.player_id();
        self.players
            .get_mut(&player_id)
            .expect("FindPlayer name lookup вернул canonical player")
            .set_silence(minutes, now_milliseconds());
        Some(player_id)
    }

    /// Один ordered pass исходного GM silence query. Clock читается
    /// только для player-а с ненулевым silence, как в `IsInSilence`.
    /// Caller выполняет два pass-а: сначала capacity, затем payload.
    pub(crate) fn silenced_player_names_pass(
        &mut self,
        mut now_milliseconds: impl FnMut() -> u32,
    ) -> Vec<Vec<u8>> {
        let player_ids: Vec<i32> = self.players.keys().copied().collect();
        let mut names = Vec::new();
        for player_id in player_ids {
            let player = self
                .players
                .get_mut(&player_id)
                .expect("snapshot ID принадлежит canonical player map");
            if player.silence_minutes() == 0 {
                continue;
            }
            if player.is_in_silence(now_milliseconds()) {
                names.push(player.shape().base_object().get_name().to_vec());
            }
        }
        names
    }

    /// Замыкает caller `goodsmessage` с runtime player-map и рецептом,
    /// опубликованным WorldServer selector-ом `SI_BATLLE_FAIRY_COMBINE`.
    /// Отсутствующий player не создаёт уведомления или пакет, как outer
    /// lookup исходного `CheckBattleFairyCombine`.
    pub(crate) fn check_battle_fairy_combine(&self, player_id: i32) -> BattleFairyCombineCheck {
        let mut check =
            self.find_player(player_id)
                .map_or_else(BattleFairyCombineCheck::default, |player| {
                    player.check_battle_fairy_combine(
                        &self.goods_factory,
                        self.battle_fairy_property.compose(),
                    )
                });
        if let Some(notification) = check.notification {
            let delivery = colored_player_notice_message(
                0xffff_ffff,
                0,
                self.get_string_by_id(notification.string_id().as_bytes()),
            )
            .send_to_player(self.net_server(), player_id);
            check.deliveries.push(delivery);
        }
        if let Some(availability) = check.availability {
            let mut message = CMessage::new(availability.message_type as i32);
            message.add_ulong(availability.deplete_fetch);
            message.add_ulong(availability.truncated_success_rate);
            check
                .deliveries
                .push(message.send_to_player(self.net_server(), player_id));
        }
        check
    }

    /// Исполняемый caller combine из `goodsmessage` после lookup player-а.
    /// Отсутствующий player, как и исходный outer lookup, не посылает packet.
    /// Old-client serializer остаётся explicit transport boundary: его нельзя
    /// заменить пустым payload без изменения `OT_NEW_OBJECT/0xbf918`.
    pub(crate) fn combine_battle_fairy<Context: OldClientGoodsCodec>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> Option<BattleFairyCombineReport> {
        let battle_fairy_enabled = self.globe_setup.battle_fairy_enabled();
        let maximum_fetch_power = self.globe_setup.maximum_fetch_power();
        let (
            players,
            random_state,
            goods_factory,
            skill_factory,
            fairy_exp_conf,
            battle_fairy_exp_config,
            battle_fairy_property,
        ) = (
            &mut self.players,
            &mut self.random_state,
            &self.goods_factory,
            &self.skill_factory,
            &self.fairy_exp_conf,
            &self.battle_fairy_exp_config,
            &self.battle_fairy_property,
        );
        let player = players.get_mut(&player_id)?;
        let mut random = |upper_bound| game_legacy_random(random_state, upper_bound);
        let mut create_goods = |goods_index, random: &mut dyn FnMut(i32) -> i32| {
            goods_factory.create_goods(
                goods_index,
                |upper_bound| random(upper_bound),
                || CGuid::create().unwrap_or(CGuid::GUID_INVALID),
                |equip_level, level| fairy_exp_conf.dw_exp_up(equip_level, level),
                |equip_level, level| battle_fairy_exp_config.dw_exp_up(equip_level, level),
            )
        };
        let mut report = {
            let mut encode_old_client = |goods: &CGoods| context.encode_goods_for_old_client(goods);
            player.combine_battle_fairy(
                battle_fairy_enabled,
                maximum_fetch_power,
                goods_factory,
                battle_fairy_property.compose(),
                skill_factory,
                &mut random,
                &mut create_goods,
                &mut encode_old_client,
            )
        };
        for effect in report.effects.clone() {
            match effect {
                BattleFairyCombineEffect::Notification {
                    player_id,
                    string_id,
                    color,
                } => {
                    let delivery = colored_player_notice_message(
                        color,
                        0,
                        self.get_string_by_id(string_id.as_bytes()),
                    )
                    .send_to_player(self.net_server(), player_id);
                    report
                        .deliveries
                        .push(BattleFairyCombineDelivery::Player(delivery));
                }
                BattleFairyCombineEffect::FetchPowerChanged {
                    message_type,
                    player_id,
                    subject_id,
                    property_name,
                    value,
                } => {
                    let mut message = CMessage::new(message_type as i32);
                    message.add_long(PLAYER_TYPE);
                    message.add_long(subject_id);
                    add_legacy_c_string(message.base_mut(), property_name.as_bytes());
                    message.add_ulong(value);
                    report
                        .deliveries
                        .push(BattleFairyCombineDelivery::FetchPower(
                            message.send_to_player(self.net_server(), player_id),
                        ));
                }
                BattleFairyCombineEffect::ObjectMove(object_move) => {
                    let delivery = self.send_battle_fairy_container_object_move(&object_move);
                    report
                        .deliveries
                        .push(BattleFairyCombineDelivery::ObjectMove(vec![delivery]));
                }
                BattleFairyCombineEffect::SkillAdded(skill) => {
                    if let Some(message) = player_skill_learned_message(
                        skill.message_type,
                        skill.skill_id,
                        skill.skill_level,
                        skill.skill_level,
                        &skill.skill_name,
                        &self.skill_factory,
                        false,
                    ) {
                        report
                            .deliveries
                            .push(BattleFairyCombineDelivery::SkillAdded(
                                message.send_to_player(self.net_server(), skill.player_id),
                            ));
                    }
                }
                BattleFairyCombineEffect::GoodsUpdated(update) => {
                    report
                        .deliveries
                        .push(BattleFairyCombineDelivery::GoodsUpdated(
                            self.send_battle_fairy_goods_update(&update),
                        ));
                }
                BattleFairyCombineEffect::Audit(audit) => {
                    let template = self.get_string_by_id(audit.string_id.as_bytes());
                    // В ветке ZHGS0003 оригинал передаёт в GetName null goods;
                    // безопасная Rust-проекция сохраняет второй форматный аргумент
                    // пустым, не выдумывая имя не созданного предмета.
                    let formatted = if audit.string_id == "ZHGS0007" {
                        format_legacy_text_fields(template, &[&audit.account], 0xff)
                    } else {
                        format_legacy_text_fields(
                            template,
                            &[&audit.account, &audit.goods_name],
                            0xff,
                        )
                    };
                    put_string_to_file("BattleFairy", &formatted);
                    report.deliveries.push(BattleFairyCombineDelivery::Audit);
                }
            }
        }
        Some(report)
    }

    fn send_battle_fairy_container_object_move(&self, object_move: &BattleFairyObjectMove) -> i32 {
        let mut message = CS2CContainerObjectMove::default();
        match object_move.operation {
            BattleFairyObjectMoveOperation::Delete => {
                message.set_operation(ContainerObjectMoveOperation::DeleteObject);
                message.set_source_container(
                    PLAYER_TYPE,
                    object_move.player_id,
                    object_move.position,
                );
                message.set_source_container_extend_id(object_move.container_extend_id as i32);
                message.set_source_object(
                    object_move.goods.object_type,
                    object_move.goods.ex_id,
                    object_move.amount,
                );
            }
            BattleFairyObjectMoveOperation::New => {
                message.set_operation(ContainerObjectMoveOperation::NewObject);
                message.set_destination_container(
                    PLAYER_TYPE,
                    object_move.player_id,
                    object_move.position,
                );
                message.set_destination_container_extend_id(object_move.container_extend_id as i32);
                message
                    .set_destination_object(object_move.goods.object_type, object_move.goods.ex_id);
                message
                    .set_object_stream(object_move.old_client_payload.clone().unwrap_or_default());
            }
        }
        message.send_to_player(self, object_move.player_id)
    }

    /// Исполняемый entry point для `goodsmessage` opcode `0x8FC2C/0x8FC2D`.
    /// Player сохраняет порядок guards и broadcast effects, а region map
    /// меняется здесь, потому что `CGame` — первый живой owner обоих runtime
    /// объектов. Around transport использует те же owned session и area maps.
    pub(crate) fn summon_battle_fairy<Context: BattleFairyDeathContext>(
        &mut self,
        player_id: i32,
        mode: i32,
        context: &mut Context,
    ) -> Option<BattleFairySummonReport> {
        let battle_fairy_enabled = self.globe_setup.battle_fairy_enabled();
        let mut report = {
            let player = self.players.get_mut(&player_id)?;
            player.summon_battle_fairy(battle_fairy_enabled, mode, &self.goods_factory)
        };
        if let Some(action) = report.spatial_action {
            let spatial_applied = report.region_id.is_some_and(|region_id| {
                let Some(region) = self.regions.get_mut(&region_id) else {
                    return false;
                };
                match action {
                    BattleFairyWarSoulAction::SetPosition { previous, target } => region
                        .base_mut()
                        .set_war_soul_position(player_id as u32, previous, target),
                    BattleFairyWarSoulAction::Delete { previous, .. } => region
                        .base_mut()
                        .delete_war_soul(player_id as u32, previous),
                }
            });
            report.spatial_applied = spatial_applied;
            if let Some(player) = self.players.get_mut(&player_id) {
                player.apply_war_soul_action(action, spatial_applied);
            }
        }
        self.deliver_battle_fairy_summon_effects(&mut report, context);
        Some(report)
    }

    fn deliver_battle_fairy_summon_effects<Context: BattleFairyDeathContext>(
        &mut self,
        report: &mut BattleFairySummonReport,
        context: &mut Context,
    ) {
        for effect in report.effects.clone() {
            match effect {
                BattleFairySummonEffect::Notification {
                    player_id,
                    string_id,
                    color,
                } => {
                    let text = self.get_string_by_id(string_id.as_bytes());
                    let delivery = colored_player_notice_message(color, 0, text)
                        .send_to_player(self.net_server(), player_id);
                    report
                        .deliveries
                        .push(BattleFairySummonDelivery::Player(delivery));
                }
                BattleFairySummonEffect::AroundMessage {
                    message_type,
                    player_id,
                    values,
                } => {
                    let mut message = CMessage::new(message_type as i32);
                    if message_type == 0x0b_f605 && values.len() == 4 {
                        message.add_long(values[0]);
                        message.add_ulong(values[1] as u32);
                        message.add_ulong((values[2] as f32).to_bits());
                        message.add_ulong((values[3] as f32).to_bits());
                    } else {
                        for value in values {
                            message.add_long(value);
                        }
                    }
                    let delivery = report
                        .region_id
                        .and_then(|region_id| self.take_region_owner(region_id))
                        .map(|owner| {
                            let player = self.find_player(player_id);
                            let delivery = player.map(|player| {
                                self.send_battle_fairy_around(
                                    owner.base(),
                                    player.shape(),
                                    &message,
                                )
                            });
                            self.restore_region_owner(owner);
                            delivery
                        })
                        .flatten();
                    report
                        .deliveries
                        .push(BattleFairySummonDelivery::Around(delivery));
                }
                BattleFairySummonEffect::PropertiesChanged { player_id } => {
                    let external = context.player_properties_external_facts(player_id);
                    if let Some(player) = self.find_player(player_id) {
                        let delivery = self.send_player_properties_changed(player, external);
                        report
                            .deliveries
                            .push(BattleFairySummonDelivery::Properties(delivery));
                    }
                }
            }
        }
    }

    /// Замыкает positional add battle-fairy container-а с player property и
    /// загруженными GlobeSetup coefficients. Old-client codec остаётся
    /// transport boundary и вызывается только на подтверждённом update path.
    pub(crate) fn add_battle_fairy_goods<Context: BattleFairyDeathContext>(
        &mut self,
        player_id: i32,
        cell: BattleFairyCell,
        incoming: &mut Option<CGoods>,
        owner_progress_allows: bool,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
        context: &mut Context,
    ) -> Option<BattleFairyEquipmentMutationReport> {
        let coefficients = self.globe_setup.player_property_coefficients();
        let mut report = self.players.get_mut(&player_id).map(|player| {
            player.add_battle_fairy_goods(
                cell,
                incoming,
                &self.goods_factory,
                coefficients,
                owner_progress_allows,
                encode_old_client,
            )
        })?;
        self.deliver_battle_fairy_equipment_effects(&mut report, context);
        Some(report)
    }

    /// Полный player-owned tail `CEquipmentContainer::Remove`: callback
    /// materializes reached результат ещё отдельного virtual
    /// `PropertiesChanged`, наблюдая player уже без removed slot-а.
    pub(crate) fn remove_player_equipment<Context: PlayerEquipmentContext>(
        &mut self,
        player_id: i32,
        ex_id: CGuid,
        runtime: PlayerEquipmentRemoveRuntimeFacts,
        recompute_properties: &mut dyn FnMut(&CPlayer) -> PlayerCombatProperties,
        context: &mut Context,
    ) -> Option<PlayerEquipmentRemoveReport> {
        let (players, goods_factory, skill_factory) =
            (&mut self.players, &self.goods_factory, &self.skill_factory);
        let mut report = players.get_mut(&player_id).map(|player| {
            player.remove_equipment_goods(
                ex_id,
                goods_factory,
                skill_factory,
                runtime,
                recompute_properties,
            )
        })?;
        self.publish_player_equipment_remove_report(&mut report, context);
        Some(report)
    }

    fn publish_player_equipment_remove_report<Context: PlayerEquipmentContext>(
        &self,
        report: &mut PlayerEquipmentRemoveReport,
        context: &mut Context,
    ) {
        for effect in report.effects.clone() {
            match effect {
                PlayerEquipmentRemoveEffect::WarSoulSkillDetached { .. } => {}
                PlayerEquipmentRemoveEffect::SkillRemoved(skill) => {
                    let mut message = CMessage::new(skill.message_type as i32);
                    add_legacy_c_string(message.base_mut(), &skill.skill_name);
                    report
                        .deliveries
                        .push(PlayerEquipmentDelivery::SkillRemoved(
                            message.send_to_player(self.net_server(), skill.player_id),
                        ));
                }
                effect @ (PlayerEquipmentRemoveEffect::WarSoulStatusAround { .. }
                | PlayerEquipmentRemoveEffect::PropertiesChangedWithoutRemovedSlot {
                    ..
                }
                | PlayerEquipmentRemoveEffect::VitalsClamped { .. }
                | PlayerEquipmentRemoveEffect::AroundUpdate(_)) => {
                    report.deliveries.push(PlayerEquipmentDelivery::Runtime(
                        context.publish_player_equipment_remove_effect(&effect),
                    ));
                }
            }
        }
    }

    /// Полный player-owned tail positional `CEquipmentContainer::Add`; оба
    /// callback-а вызываются в native порядке относительно container commit.
    pub(crate) fn add_player_equipment<Context: PlayerEquipmentContext>(
        &mut self,
        player_id: i32,
        position: u32,
        incoming: &mut Option<CGoods>,
        runtime: PlayerEquipmentAddRuntimeFacts,
        register_with_goods_ai: &mut dyn FnMut(&CGoods),
        recompute_properties: &mut dyn FnMut(&CPlayer) -> PlayerCombatProperties,
        context: &mut Context,
    ) -> Option<PlayerEquipmentAddReport> {
        let (players, goods_factory, skill_factory) =
            (&mut self.players, &self.goods_factory, &self.skill_factory);
        let mut report = players.get_mut(&player_id).map(|player| {
            player.add_equipment_goods(
                position,
                incoming,
                goods_factory,
                skill_factory,
                runtime,
                register_with_goods_ai,
                recompute_properties,
            )
        })?;
        self.publish_player_equipment_add_report(&mut report, context);
        Some(report)
    }

    fn publish_player_equipment_add_report<Context: PlayerEquipmentContext>(
        &self,
        report: &mut PlayerEquipmentAddReport,
        context: &mut Context,
    ) {
        for effect in report.effects.clone() {
            match effect {
                PlayerEquipmentAddEffect::WarSoulSkillAttached { .. } => {}
                PlayerEquipmentAddEffect::SkillAdded(skill) => {
                    if let Some(message) = player_skill_learned_message(
                        skill.message_type,
                        skill.skill_id,
                        skill.skill_level,
                        skill.skill_level,
                        &skill.skill_name,
                        &self.skill_factory,
                        true,
                    ) {
                        report.deliveries.push(PlayerEquipmentDelivery::SkillAdded(
                            message.send_to_player(self.net_server(), skill.player_id),
                        ));
                    }
                }
                effect @ (PlayerEquipmentAddEffect::PropertiesChanged { .. }
                | PlayerEquipmentAddEffect::AroundUpdate(_)
                | PlayerEquipmentAddEffect::PackageExtensionLogged { .. }) => {
                    report.deliveries.push(PlayerEquipmentDelivery::Runtime(
                        context.publish_player_equipment_add_effect(&effect),
                    ));
                }
            }
        }
    }

    /// Замыкает remove по GUID с тем же player/equipment state и сохраняет
    /// подтверждённую двойную публикацию `0xBF918` после property removal.
    pub(crate) fn remove_battle_fairy_goods<Context: BattleFairyDeathContext>(
        &mut self,
        player_id: i32,
        ex_id: CGuid,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
        context: &mut Context,
    ) -> Option<BattleFairyEquipmentMutationReport> {
        let coefficients = self.globe_setup.player_property_coefficients();
        let mut report = self.players.get_mut(&player_id).map(|player| {
            player.remove_battle_fairy_goods(
                ex_id,
                &self.goods_factory,
                coefficients,
                encode_old_client,
            )
        })?;
        self.deliver_battle_fairy_equipment_effects(&mut report, context);
        Some(report)
    }

    fn deliver_battle_fairy_equipment_effects<Context: BattleFairyDeathContext>(
        &self,
        report: &mut BattleFairyEquipmentMutationReport,
        context: &mut Context,
    ) {
        for effect in report.effects.clone() {
            match effect {
                BattleFairyEquipmentMutationEffect::PropertiesChanged { player_id } => {
                    let external = context.player_properties_external_facts(player_id);
                    if let Some(player) = self.find_player(player_id) {
                        report
                            .deliveries
                            .push(BattleFairyEquipmentMutationDelivery::Properties(
                                self.send_player_properties_changed(player, external),
                            ));
                    }
                }
                BattleFairyEquipmentMutationEffect::BattleFairyUpdated(update) => {
                    let delivery = self.send_battle_fairy_goods_update(&update);
                    report
                        .deliveries
                        .push(BattleFairyEquipmentMutationDelivery::GoodsUpdated(delivery));
                }
            }
        }
    }

    fn send_battle_fairy_goods_update(
        &self,
        update: &crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate,
    ) -> i32 {
        let mut message = CMessage::new(update.message_type as i32);
        message.add_long(update.player_id);
        message.base_mut().add_guid(update.goods.ex_id);
        message.add_ulong(update.old_client_payload.len() as u32);
        message.base_mut().add(&update.old_client_payload);
        message.send_to_player(self.net_server(), update.player_id)
    }

    /// Исполняемый entry point goods-message `0x8FC2A`; decoder передаёт пары
    /// property/client-points без предварительного масштабирования.
    pub(crate) fn allocate_battle_fairy_potential<Context: BattleFairyDeathContext>(
        &mut self,
        player_id: i32,
        allocations: &[(i32, i32)],
        context: &mut Context,
    ) -> Option<crate::gameserver::appserver::player::BattleFairyPotentialAllocationReport> {
        let enabled = self.globe_setup.battle_fairy_enabled();
        let coefficients = self.globe_setup.player_property_coefficients();
        let mut report = {
            let player = self.players.get_mut(&player_id)?;
            let mut encode_old_client = |goods: &CGoods| context.encode_goods_for_old_client(goods);
            player.allocate_battle_fairy_potential(
                enabled,
                allocations,
                &self.goods_factory,
                coefficients,
                &mut encode_old_client,
            )
        };
        for effect in report.effects.clone() {
            match effect {
                BattleFairyPotentialAllocationEffect::Notification {
                    player_id,
                    string_id,
                    color,
                } => {
                    let delivery = colored_player_notice_message(
                        color,
                        0,
                        self.get_string_by_id(string_id.as_bytes()),
                    )
                    .send_to_player(self.net_server(), player_id);
                    report
                        .deliveries
                        .push(BattleFairyPotentialAllocationDelivery::Player(delivery));
                }
                BattleFairyPotentialAllocationEffect::PropertiesChanged { player_id } => {
                    let external = context.player_properties_external_facts(player_id);
                    if let Some(player) = self.find_player(player_id) {
                        report
                            .deliveries
                            .push(BattleFairyPotentialAllocationDelivery::Properties(
                                self.send_player_properties_changed(player, external),
                            ));
                    }
                }
                BattleFairyPotentialAllocationEffect::GoodsUpdated(update) => {
                    report
                        .deliveries
                        .push(BattleFairyPotentialAllocationDelivery::GoodsUpdated(
                            self.send_battle_fairy_goods_update(&update),
                        ));
                }
            }
        }
        Some(report)
    }

    /// Полный runtime entry point goods-message `0x8FC28`: общий Game RNG,
    /// live log gates, factory, player wallet и positional BF-container
    /// исполняются в одном mutable snapshot-е.
    pub(crate) fn upgrade_battle_fairy_equipment<Context: OldClientGoodsCodec>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> Option<crate::gameserver::appserver::player::BattleFairyUpgradeReport> {
        let log_gates = crate::gameserver::appserver::player::BattleFairyUpgradeLogGates {
            success: self.log_system.goods_upgrade_success_enabled(),
            failure: self.log_system.goods_upgrade_failure_enabled(),
            lost_target: self.log_system.goods_lost_by_upgrade_enabled(),
        };
        let mut report = {
            let (players, random_state, goods_factory) = (
                &mut self.players,
                &mut self.random_state,
                &self.goods_factory,
            );
            let player = players.get_mut(&player_id)?;
            let mut random = |upper_bound| game_legacy_random(random_state, upper_bound);
            let mut encode_old_client = |goods: &CGoods| context.encode_goods_for_old_client(goods);
            player.upgrade_battle_fairy_equipment(
                goods_factory,
                log_gates,
                &mut random,
                &mut encode_old_client,
            )
        };
        for effect in report.effects.clone() {
            match effect {
                BattleFairyUpgradeEffect::Notification {
                    player_id,
                    string_id,
                    color,
                    format_value,
                } => {
                    let template = self.get_string_by_id(string_id.as_bytes());
                    let text = format_value.map_or_else(
                        || legacy_c_string_prefix(template).to_vec(),
                        |value| format_single_legacy_u32(template, value, 255),
                    );
                    let delivery = colored_player_notice_message(color, 0, &text)
                        .send_to_player(self.net_server(), player_id);
                    report
                        .deliveries
                        .push(BattleFairyUpgradeDelivery::Player(delivery));
                }
                BattleFairyUpgradeEffect::MoneyChanged {
                    player_id, outcome, ..
                } => {
                    report.deliveries.push(BattleFairyUpgradeDelivery::Money(
                        self.send_player_money_decrease(player_id, &outcome),
                    ));
                }
                BattleFairyUpgradeEffect::GoodsUpdated(update) => {
                    report
                        .deliveries
                        .push(BattleFairyUpgradeDelivery::GoodsUpdated(
                            self.send_battle_fairy_goods_update(&update),
                        ));
                }
                effect @ (BattleFairyUpgradeEffect::GemConsumed { .. }
                | BattleFairyUpgradeEffect::TargetDeleted { .. }) => {
                    report
                        .deliveries
                        .push(BattleFairyUpgradeDelivery::Container(
                            self.send_battle_fairy_upgrade_container(&effect),
                        ));
                }
                effect @ BattleFairyUpgradeEffect::Audit { .. } => {
                    report.deliveries.push(BattleFairyUpgradeDelivery::Audit(
                        self.send_battle_fairy_upgrade_audit(&effect),
                    ));
                }
            }
        }
        Some(report)
    }

    pub(crate) fn send_player_money_decrease(
        &self,
        player_id: i32,
        outcome: &crate::gameserver::appserver::container::cwallet::CurrencyDecreaseOutcome,
    ) -> Vec<i32> {
        use crate::gameserver::appserver::container::cwallet::CurrencyDecreaseOutcome;

        let mut message = CS2CContainerObjectMove::default();
        match outcome {
            CurrencyDecreaseOutcome::Decreased(change) => {
                message.set_operation(ContainerObjectMoveOperation::MoveObject);
                message.set_source_container(change.owner_type, change.owner_id, change.position);
                message.set_source_container_extend_id(4);
                message.set_source_object(
                    change.identity.object_type,
                    change.identity.ex_id,
                    change.amount,
                );
            }
            CurrencyDecreaseOutcome::Removed(removed) => {
                message.set_operation(ContainerObjectMoveOperation::DeleteObject);
                message.set_source_container(
                    removed.owner_type,
                    removed.owner_id,
                    removed.position,
                );
                message.set_source_container_extend_id(4);
                let identity = removed.goods.identity();
                message.set_source_object(identity.object_type, identity.ex_id, removed.amount);
            }
            CurrencyDecreaseOutcome::NoChange
            | CurrencyDecreaseOutcome::InvalidStoredCurrency { .. } => return Vec::new(),
        }
        vec![message.send_to_player(self, player_id)]
    }

    pub(crate) fn decrease_player_money(
        &mut self,
        player_id: i32,
        amount: u32,
    ) -> Option<crate::gameserver::appserver::player::PlayerMoneyDecrease> {
        let (players, goods_factory) = (&mut self.players, &self.goods_factory);
        Some(
            players
                .get_mut(&player_id)?
                .decrease_money(amount, goods_factory),
        )
    }

    /// Exact `CPlayer::SetMoney` boundary для script family `3011..3016`:
    /// wallet остаётся canonical state owner-ом, а CGame публикует соответствующий
    /// create/amount/delete packet с extend ID 4. Как и оригинал, caller считает
    /// найденного игрока успехом даже при отказе wallet создать currency object.
    pub(crate) fn set_script_player_money<Context: OldClientGoodsCodec>(
        &mut self,
        player_id: i32,
        requested: u32,
        context: &mut Context,
    ) -> bool {
        let Some(previous) = self.find_player(player_id).map(CPlayer::money) else {
            return false;
        };
        if requested < previous {
            let Some(change) = self.decrease_player_money(player_id, previous - requested) else {
                return false;
            };
            let _ = self.send_player_money_decrease(player_id, &change.outcome);
        } else if requested > previous {
            let amount = requested - previous;
            let created = self.create_goods_batch(self.goods_factory.get_gold_coin_index(), amount);
            let outcome = {
                let (players, goods_factory) = (&mut self.players, &self.goods_factory);
                let Some(player) = players.get_mut(&player_id) else {
                    return false;
                };
                player.increase_money(amount, goods_factory, created)
            };
            let _ = self.send_player_money_increase(player_id, &outcome, context);
        }
        true
    }

    /// World `0x7FE34/0x7FE37` повторяет старый `GetMoney - signed fee`, затем
    /// `SetMoney(max(signed(result), 0))`. Обычная положительная плата идёт
    /// через тот же wallet/container wire, что остальные gameplay debits;
    /// отрицательный legacy параметр сохраняет историческое пополнение.
    pub(crate) fn apply_war_application_money<Context: OldClientGoodsCodec>(
        &mut self,
        player_id: i32,
        fee: i32,
        context: &mut Context,
    ) -> Option<(u32, u32, Vec<i32>)> {
        let previous = self.find_player(player_id)?.money();
        let wrapped = previous.wrapping_sub(fee as u32);
        let resulting = if (wrapped as i32) < 0 { 0 } else { wrapped };
        let deliveries = if resulting < previous {
            let change = self.decrease_player_money(player_id, previous - resulting)?;
            self.send_player_money_decrease(player_id, &change.outcome)
        } else if resulting > previous {
            let amount = resulting - previous;
            let created = self.create_goods_batch(self.goods_factory.get_gold_coin_index(), amount);
            let outcome = {
                let (players, goods_factory) = (&mut self.players, &self.goods_factory);
                players
                    .get_mut(&player_id)?
                    .increase_money(amount, goods_factory, created)
            };
            self.send_player_money_increase(player_id, &outcome, context)
        } else {
            Vec::new()
        };
        let current = self.find_player(player_id)?.money();
        Some((previous, current, deliveries))
    }

    fn send_battle_fairy_upgrade_container(&self, effect: &BattleFairyUpgradeEffect) -> Vec<i32> {
        match effect {
            BattleFairyUpgradeEffect::GemConsumed {
                player_id,
                consumed,
            } if consumed.removed => {
                vec![self.send_battle_fairy_upgrade_delete_for_player(
                    *player_id,
                    consumed.goods,
                    consumed.cell.position(),
                    consumed.previous_amount,
                )]
            }
            BattleFairyUpgradeEffect::GemConsumed {
                player_id,
                consumed,
            } => {
                let mut message = CS2CContainerObjectAmountChange::default();
                message.set_source_container(PLAYER_TYPE, *player_id, consumed.cell.position());
                message.set_source_container_extend_id(12);
                message.set_object(consumed.goods.object_type, consumed.goods.ex_id);
                message.set_object_amount(consumed.remaining_amount);
                vec![message.send_to_player(self, *player_id)]
            }
            BattleFairyUpgradeEffect::TargetDeleted {
                player_id,
                goods,
                position,
                ..
            } => vec![self.send_battle_fairy_upgrade_delete_for_player(
                *player_id,
                goods.identity,
                *position,
                goods.amount,
            )],
            _ => Vec::new(),
        }
    }

    fn send_battle_fairy_upgrade_delete_for_player(
        &self,
        player_id: i32,
        goods: crate::gameserver::appserver::shape::ShapeIdentity,
        position: u32,
        amount: u32,
    ) -> i32 {
        let mut message = CS2CContainerObjectMove::default();
        message.set_operation(ContainerObjectMoveOperation::DeleteObject);
        message.set_source_container(PLAYER_TYPE, player_id, position);
        message.set_source_container_extend_id(12);
        message.set_source_object(goods.object_type, goods.ex_id, amount);
        message.send_to_player(self, player_id)
    }

    fn send_battle_fairy_upgrade_audit(&self, effect: &BattleFairyUpgradeEffect) -> Vec<i32> {
        let BattleFairyUpgradeEffect::Audit {
            message_type,
            event,
            player_id,
            player,
            target,
            gems,
        } = effect
        else {
            return Vec::new();
        };
        let mut message = CMessage::new(*message_type as i32);
        message.add_byte(*event);
        message.add_long(*player_id);
        if *message_type == 0x0006_0203 {
            add_battle_fairy_upgrade_log_goods(&mut message, Some(target));
            for gem in gems {
                add_battle_fairy_upgrade_log_goods(&mut message, gem.as_ref());
            }
            message.add_long(player.region_id);
            message.add_long(player.tile_x);
            message.add_long(player.tile_y);
        } else {
            message.base_mut().add_short(player.pk_count as i16);
            message.add_ulong(player.money);
            message.add_ulong(player.depot_money);
            message.base_mut().add_guid(target.identity.ex_id);
            // Primary GameServer пишет price в поле, которое WorldServer
            // исторически называет amount, и literal 1 в поле price.
            message.add_ulong(target.price);
            add_legacy_c_string(message.base_mut(), &target.name);
            message.add_ulong(1);
            message.add_long(player.region_id);
            message.add_long(player.tile_x);
            message.add_long(player.tile_y);
            message.add_ulong(player.client_ip);
        }
        message.send(self, false).into_iter().collect()
    }

    fn send_battle_fairy_packet_consumption(
        &self,
        effect: &BattleFairyPotentialResetEffect,
    ) -> Vec<i32> {
        let BattleFairyPotentialResetEffect::PacketItemConsumed {
            player_id,
            goods,
            position,
            previous_amount,
            remaining_amount,
            consumed,
            ..
        } = effect
        else {
            return Vec::new();
        };
        let Some(position) = position else {
            return Vec::new();
        };
        if !consumed {
            return Vec::new();
        }
        if *remaining_amount == 0 {
            let mut message = CS2CContainerObjectMove::default();
            message.set_operation(ContainerObjectMoveOperation::DeleteObject);
            message.set_source_container(PLAYER_TYPE, *player_id, *position);
            message.set_source_container_extend_id(1);
            message.set_source_object(goods.object_type, goods.ex_id, *previous_amount);
            return vec![message.send_to_player(self, *player_id)];
        }

        let mut message = CS2CContainerObjectAmountChange::default();
        message.set_source_container(PLAYER_TYPE, *player_id, *position);
        message.set_source_container_extend_id(1);
        message.set_object(goods.object_type, goods.ex_id);
        message.set_object_amount(*remaining_amount);
        vec![message.send_to_player(self, *player_id)]
    }

    /// Исполняемый entry point goods-message `0x8FC2B`: reset item ищется и
    /// расходуется в owned player packet до potential/player mutations.
    pub(crate) fn reset_battle_fairy_potential<Context: BattleFairyDeathContext>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> Option<crate::gameserver::appserver::player::BattleFairyPotentialResetReport> {
        let enabled = self.globe_setup.battle_fairy_enabled();
        let mut report = {
            let player = self.players.get_mut(&player_id)?;
            let mut encode_old_client = |goods: &CGoods| context.encode_goods_for_old_client(goods);
            player.reset_battle_fairy_potential(
                enabled,
                &self.goods_factory,
                &mut encode_old_client,
            )
        };
        for effect in report.effects.clone() {
            match effect {
                BattleFairyPotentialResetEffect::Notification {
                    player_id,
                    string_id,
                    color,
                } => {
                    let delivery = colored_player_notice_message(
                        color,
                        0,
                        self.get_string_by_id(string_id.as_bytes()),
                    )
                    .send_to_player(self.net_server(), player_id);
                    report
                        .deliveries
                        .push(BattleFairyPotentialResetDelivery::Player(delivery));
                }
                effect @ BattleFairyPotentialResetEffect::PacketItemConsumed { .. } => {
                    report
                        .deliveries
                        .push(BattleFairyPotentialResetDelivery::PacketItem(
                            self.send_battle_fairy_packet_consumption(&effect),
                        ));
                }
                BattleFairyPotentialResetEffect::PropertiesChanged { player_id } => {
                    let external = context.player_properties_external_facts(player_id);
                    if let Some(player) = self.find_player(player_id) {
                        report
                            .deliveries
                            .push(BattleFairyPotentialResetDelivery::Properties(
                                self.send_player_properties_changed(player, external),
                            ));
                    }
                }
                BattleFairyPotentialResetEffect::GoodsUpdated(update) => {
                    report
                        .deliveries
                        .push(BattleFairyPotentialResetDelivery::GoodsUpdated(
                            self.send_battle_fairy_goods_update(&update),
                        ));
                }
            }
        }
        Some(report)
    }

    /// Runtime entry point `CBattleFairyContainer::ResetSkill`, общий для
    /// script-functions распределения обычного/special skill и прямого caller-а
    /// с расходом reset item. RNG принадлежит одному `CGame` sequence.
    pub(crate) fn reset_battle_fairy_skill<Context>(
        &mut self,
        player_id: i32,
        position: i32,
        consume_item: bool,
        context: &mut Context,
    ) -> Option<BattleFairySkillResetReport>
    where
        Context: BattleFairySkillResetContext + OldClientGoodsCodec,
    {
        let enabled = self.globe_setup.battle_fairy_enabled();
        let mut report = {
            let (players, random_state, goods_factory, skill_factory) = (
                &mut self.players,
                &mut self.random_state,
                &self.goods_factory,
                &self.skill_factory,
            );
            let player = players.get_mut(&player_id)?;
            let mut random = |upper_bound| game_legacy_random(random_state, upper_bound);
            let mut encode_old_client = |goods: &CGoods| context.encode_goods_for_old_client(goods);
            player.reset_battle_fairy_skill(
                enabled,
                position,
                consume_item,
                goods_factory,
                skill_factory,
                &mut random,
                &mut encode_old_client,
            )
        };
        for effect in report.effects.clone() {
            match effect {
                BattleFairySkillResetEffect::Notification {
                    player_id,
                    string_id,
                    color,
                } => {
                    let delivery = colored_player_notice_message(
                        color,
                        0,
                        self.get_string_by_id(string_id.as_bytes()),
                    )
                    .send_to_player(self.net_server(), player_id);
                    report
                        .deliveries
                        .push(BattleFairySkillResetDelivery::Player(delivery));
                }
                effect @ BattleFairySkillResetEffect::PacketItemConsumed { .. } => {
                    report
                        .deliveries
                        .push(BattleFairySkillResetDelivery::PacketItem(
                            context.publish_battle_fairy_skill_reset_packet_consumption(&effect),
                        ));
                }
                BattleFairySkillResetEffect::SkillRemoved(skill) => {
                    let mut message = CMessage::new(skill.message_type as i32);
                    add_legacy_c_string(message.base_mut(), &skill.skill_name);
                    report
                        .deliveries
                        .push(BattleFairySkillResetDelivery::SkillRemoved(
                            message.send_to_player(self.net_server(), skill.player_id),
                        ));
                }
                BattleFairySkillResetEffect::SkillAdded(skill) => {
                    if let Some(message) = player_skill_learned_message(
                        skill.message_type,
                        skill.skill_id,
                        skill.skill_level,
                        skill.skill_level,
                        &skill.skill_name,
                        &self.skill_factory,
                        true,
                    ) {
                        report
                            .deliveries
                            .push(BattleFairySkillResetDelivery::SkillAdded(
                                message.send_to_player(self.net_server(), skill.player_id),
                            ));
                    }
                }
                BattleFairySkillResetEffect::SelectedSkillLearned(skill) => {
                    if let Some(message) = player_skill_learned_message(
                        skill.message_type,
                        skill.skill_id,
                        skill.skill_level,
                        skill.skill_level,
                        &skill.skill_name,
                        &self.skill_factory,
                        false,
                    ) {
                        report.deliveries.push(
                            BattleFairySkillResetDelivery::SelectedSkillLearned(
                                message.send_to_player(self.net_server(), skill.player_id),
                            ),
                        );
                    }
                }
                BattleFairySkillResetEffect::GoodsUpdated(update) => {
                    report
                        .deliveries
                        .push(BattleFairySkillResetDelivery::GoodsUpdated(
                            self.send_battle_fairy_goods_update(&update),
                        ));
                }
            }
        }
        Some(report)
    }

    /// Начальный player/equipment участок goods-message `0x8FC29`.
    /// Отсутствующий либо не-BF headgear не запускает script и не шлёт ack.
    pub(crate) fn detach_battle_fairy_script_skills(&mut self, player_id: i32) -> Option<Vec<u32>> {
        let valid = self
            .find_player(player_id)?
            .war_soul_goods(&self.goods_factory)
            .is_some();
        if !valid {
            return None;
        }
        let (players, goods_factory, skill_factory) =
            (&mut self.players, &self.goods_factory, &self.skill_factory);
        Some(
            players
                .get_mut(&player_id)
                .expect("проверенный player остаётся в game map")
                .detach_battle_fairy_script_skills(goods_factory, skill_factory),
        )
    }

    /// Завершающий player/equipment участок `0x8FC29` после script mutation.
    /// Safe owner перечитывает equipped headgear вместо удержания native raw
    /// pointer через произвольный script callback.
    pub(crate) fn attach_battle_fairy_script_skills(
        &mut self,
        player_id: i32,
    ) -> BattleFairyScriptSkillAttachReport {
        let skills = self
            .players
            .get_mut(&player_id)
            .map_or_else(Vec::new, |player| {
                player.attach_battle_fairy_script_skills(&self.goods_factory, &self.skill_factory)
            });
        let deliveries = skills
            .iter()
            .filter_map(|skill| {
                player_skill_learned_message(
                    skill.message_type,
                    skill.skill_id,
                    skill.skill_level,
                    skill.skill_level,
                    &skill.skill_name,
                    &self.skill_factory,
                    true,
                )
                .map(|message| message.send_to_player(self.net_server(), skill.player_id))
            })
            .collect();
        BattleFairyScriptSkillAttachReport { skills, deliveries }
    }

    /// Script `2249 / FairyExpUp`: enhancement хранит только shadow, поэтому
    /// mutation всегда разрешает исходный live goods. При созревании native
    /// owner сначала окончательно удаляет старый экземпляр из packet/equipment,
    /// затем пытается положить ripe replacement в packet; поздний отказ не
    /// откатывает уже выполненное удаление.
    pub(crate) fn fairy_exp_up_selected_goods<Context: GameContainerMessageRuntime>(
        &mut self,
        player_id: i32,
        experience: u32,
        context: &mut Context,
    ) -> bool {
        let Some((goods_id, source)) = self.players.get(&player_id).and_then(|player| {
            let goods_id = player.enhancement_selected_goods_id()?;
            let source = player.enhancement_original_container(0, goods_id)?;
            player
                .trade_source_goods(source.container_extend_id, source.goods_position, goods_id)
                .map(|_| (goods_id, source))
        }) else {
            return false;
        };

        let grow_log_enabled = self.log_system.fairy_grow_enabled();
        let egg_max_level = self.globe_setup.fairy_egg_max_level();
        let upgrade_rate = self.globe_setup.fairy_upgrade_rate();
        let mut remaining = experience;
        let (exp, ripe_id, update) = {
            let (players, factory, exp_config) =
                (&mut self.players, &self.goods_factory, &self.fairy_exp_conf);
            let Some(goods) = players
                .get_mut(&player_id)
                .and_then(|player| player.get_goods_by_id_mut(goods_id))
            else {
                return false;
            };
            let fairy_guid = goods.identity().ex_id.to_string().into_bytes();
            let fairy_name = goods.name().to_vec();
            let runtime = FairyExpRuntime {
                player_id,
                fairy_guid: &fairy_guid,
                fairy_name: &fairy_name,
                log_value: 1,
                suppress_grow_log: false,
                grow_log_enabled,
                egg_max_level,
                upgrade_rate,
            };
            let Ok(Some(exp)) =
                goods.fairy_exp_up(&mut remaining, runtime, |equip_level, level| {
                    exp_config.dw_exp_up(equip_level, level)
                })
            else {
                return false;
            };
            if exp.result <= FairyExpUpResult::None {
                return true;
            }
            if !goods.save_fairy_properties(factory).unwrap_or(false) {
                return false;
            }
            let ripe_id = (exp.result == FairyExpUpResult::ChangeState).then(|| {
                goods
                    .fairy_properties()
                    .expect("successful fairy state change сохраняет property owner")
                    .ripe_id
            });
            let update = ripe_id
                .is_none()
                .then(|| (goods.identity(), context.encode_goods_for_old_client(goods)));
            (exp, ripe_id, update)
        };

        for log in &exp.grow_logs {
            let _ = self.send_fairy_grow_log(log);
        }
        if let Some((goods, payload)) = update {
            let mut message = CMessage::new(0x0b_f918);
            message.add_long(player_id);
            message.base_mut().add_guid(goods.ex_id);
            message.add_ulong(payload.len() as u32);
            message.base_mut().add(&payload);
            let _ = message.send_to_player(self.net_server(), player_id);
            return true;
        }

        let Some(ripe_id) = ripe_id else {
            return true;
        };
        let Some(mut replacement) = self.create_goods_batch(ripe_id, 1).into_iter().next() else {
            return false;
        };
        let copied = self
            .players
            .get(&player_id)
            .and_then(|player| player.get_goods_by_id(goods_id))
            .is_some_and(|goods| {
                replacement
                    .copy_fairy_addon_properties_from(
                        goods,
                        &self.goods_factory,
                        |equip_level, level| self.fairy_exp_conf.dw_exp_up(equip_level, level),
                    )
                    .is_ok_and(|loaded| loaded)
            });
        if !copied {
            return false;
        }
        let Some(fairy) = replacement.fairy_properties_mut() else {
            return false;
        };
        fairy.fairy_state = 2;
        if !replacement
            .save_fairy_properties(&self.goods_factory)
            .unwrap_or(false)
        {
            return false;
        }

        let mut player = self
            .players
            .remove(&player_id)
            .expect("selected fairy player остаётся в game map");
        let old_identity = player
            .get_goods_by_id(goods_id)
            .expect("selected fairy проверена до replacement")
            .identity();
        let old_amount = player
            .get_goods_by_id(goods_id)
            .expect("selected fairy проверена до replacement")
            .amount();
        let removed = if source.container_extend_id == 1 {
            matches!(
                player.packet_mut().remove_goods(goods_id),
                Some(VolumeGoodsRemoveOutcome::Removed(
                    AmountLimitGoodsTaken::Removed(_)
                ))
            )
        } else if source.container_extend_id == 2 {
            let facts = {
                let goods = player
                    .get_goods_by_id(goods_id)
                    .expect("equipment fairy проверена до remove facts");
                context.enhancement_equipment_remove_facts(
                    &player,
                    goods,
                    self.globe_setup.pack_add_enabled(),
                )
            };
            let mut recompute =
                |player: &CPlayer| context.recompute_enhancement_player_properties(player);
            let mut report = player.remove_equipment_goods(
                goods_id,
                &self.goods_factory,
                &self.skill_factory,
                facts,
                &mut recompute,
            );
            drop(recompute);
            self.publish_player_equipment_remove_report(&mut report, context);
            matches!(report.outcome, EquipmentRemoveOutcome::Removed(_))
        } else {
            false
        };
        if !removed {
            let update = player
                .get_goods_by_id(goods_id)
                .map(|goods| (goods.identity(), context.encode_goods_for_old_client(goods)));
            self.players.insert(player_id, player);
            if let Some((goods, payload)) = update {
                let mut message = CMessage::new(0x0b_f918);
                message.add_long(player_id);
                message.base_mut().add_guid(goods.ex_id);
                message.add_ulong(payload.len() as u32);
                message.base_mut().add(&payload);
                let _ = message.send_to_player(self.net_server(), player_id);
            }
            return true;
        }

        let _ = player.enhancement_remove_shadow(goods_id);
        let _ = self.send_container_object_delete(player_id, &source, old_identity, old_amount);
        let (additions, _rejected) = player.add_script_fairy_goods_to_packet(
            vec![replacement],
            &self.goods_factory,
            &mut |goods| context.encode_goods_for_old_client(goods),
        );
        for addition in &additions {
            if let Some(position) = addition.position
                && matches!(addition.outcome, VolumeGoodsAddOutcome::Added(_))
                && let Some(goods) = player.packet().get_goods(position)
            {
                context.register_enhancement_goods_ai(goods);
            }
        }
        self.players.insert(player_id, player);
        for addition in &additions {
            let _ = self.send_player_packet_addition(addition);
        }
        true
    }

    /// Gameplay owner script-family `9400..9411`. Selector и вычисление
    /// аргументов остаются в `CScript::RunFunction`; здесь замкнуты canonical
    /// player/equipment mutation, RNG, client wire и локальный audit.
    pub(crate) fn run_battle_fairy_script_action<Context>(
        &mut self,
        script_player_id: Option<i32>,
        action: BattleFairyScriptAction,
        context: &mut Context,
    ) -> i32
    where
        Context: BattleFairyDeathContext + BattleFairySkillResetContext,
    {
        let target_id = |game: &Self, player_name: &[u8]| {
            if player_name.is_empty() {
                script_player_id.filter(|player_id| game.find_player(*player_id).is_some())
            } else {
                game.find_player_by_name(player_name)
                    .map(CPlayer::player_id)
            }
        };
        match action {
            BattleFairyScriptAction::GetFetchPower { player_name } => target_id(self, &player_name)
                .and_then(|player_id| self.find_player(player_id))
                .map_or(0, |player| player.fetch_power() as i32),
            BattleFairyScriptAction::GetSkillValue {
                player_name,
                position,
                value_id,
            } => target_id(self, &player_name)
                .and_then(|player_id| self.find_player(player_id))
                .and_then(|player| player.equipment().get_goods(10))
                .map_or(0, |goods| {
                    goods.addon_property_value(&self.goods_factory, GAP_BF_SKY + position, value_id)
                }),
            BattleFairyScriptAction::GetAttribute {
                player_name,
                attribute,
            } => {
                if !is_battle_fairy_script_attribute(attribute) {
                    return 0;
                }
                let stored = target_id(self, &player_name)
                    .and_then(|player_id| self.find_player(player_id))
                    .and_then(|player| player.equipment().get_goods(10))
                    .map_or(0, |goods| {
                        goods.addon_property_value(&self.goods_factory, attribute, 1)
                    });
                if matches!(attribute, GAP_BF_LEVEL..=GAP_BF_CURRENT_MAX_EXP)
                    || attribute == GAP_BF_PULLULATERATE
                {
                    stored
                } else {
                    (f64::from(stored) * 0.0001_f64) as i32
                }
            }
            BattleFairyScriptAction::AddSkill {
                player_name,
                skill_name,
                skill_level,
                position,
            } => {
                let Some(player_id) = target_id(self, &player_name) else {
                    return -1;
                };
                let skill_level = if skill_level == 0x09ff_fff9 {
                    1
                } else {
                    skill_level
                };
                let skill_id = self.skill_factory.query_skill_id(Some(&skill_name));
                if skill_id == 0
                    || !self.find_player(player_id).is_some_and(|player| {
                        player.equipment().get_goods(10).is_some_and(|goods| {
                            goods.addon_property_value(&self.goods_factory, GAP_BF_HP, 1) != 0
                        })
                    })
                {
                    return -1;
                }
                let Some(mutation) =
                    self.add_remote_player_skill(player_id, &skill_name, skill_level as u16)
                else {
                    return -1;
                };
                let legacy_return = i32::from(mutation.legacy_result);
                if legacy_return == 0 {
                    return 0;
                }
                let Some(position) = position.filter(|position| (0..=6).contains(position)) else {
                    return legacy_return;
                };
                let update = {
                    let player = self
                        .find_player_mut(player_id)
                        .expect("resolved battle-fairy script player остаётся в map");
                    let goods = player
                        .equipment_mut()
                        .get_goods_mut(10)
                        .expect("AddSkill guard сохранил slot 10");
                    let property = GAP_BF_SKY + position;
                    if position >= 3 {
                        let _ = goods.set_addon_property_value_core(property, 1, 0);
                        let _ = goods.set_addon_property_value_core(property, 2, 0);
                    }
                    let _ = goods.set_addon_property_value_core(property, 1, skill_level);
                    if position >= 3 {
                        let _ = goods.set_addon_property_value_core(property, 2, skill_id as i32);
                    }
                    crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate {
                        message_type: 0x0b_f918,
                        player_id,
                        goods: goods.identity(),
                        old_client_payload: context.encode_goods_for_old_client(goods),
                    }
                };
                let _ = self.send_battle_fairy_goods_update(&update);
                if let Some(initiator_id) = script_player_id {
                    if let Some(message) = player_skill_learned_message(
                        0x0b_f71d,
                        skill_id,
                        mutation.skill_level,
                        mutation.skill_level,
                        &skill_name,
                        &self.skill_factory,
                        true,
                    ) {
                        let _ = message.send_to_player(self.net_server(), initiator_id);
                    }
                    if let Some(player) = self.find_player(initiator_id) {
                        let account = player.account();
                        let skill_id_text = skill_id.to_string();
                        let skill_level_text = skill_level.to_string();
                        let text = format_legacy_text_fields(
                            self.get_string_by_id(b"ZHGS0039"),
                            &[
                                account,
                                skill_id_text.as_bytes(),
                                skill_level_text.as_bytes(),
                            ],
                            0xff,
                        );
                        put_string_to_file("BattleFairy", &text);
                    }
                }
                legacy_return
            }
            BattleFairyScriptAction::SetAttribute {
                player_name,
                attribute,
                value,
            } => {
                if !is_battle_fairy_script_attribute(attribute) {
                    return 0;
                }
                let resolved = target_id(self, &player_name);
                if let Some(player_id) = resolved {
                    let update = {
                        let (players, factory) = (&mut self.players, &self.goods_factory);
                        let Some(player) = players.get_mut(&player_id) else {
                            return 0;
                        };
                        let Some(goods) = player.equipment_mut().get_goods_mut(10) else {
                            return 0;
                        };
                        if matches!(attribute, GAP_BF_LEVEL..=GAP_BF_CURRENT_MAX_EXP) {
                            let _ = goods.set_addon_property_value_core(attribute, 1, value);
                        } else {
                            let scaled = i64::from(value).wrapping_mul(10_000);
                            if (GAP_BF_BRAVE..=GAP_BF_STRENGH).contains(&attribute) {
                                let balanced =
                                    i64::from(goods.addon_property_value(factory, attribute, 1))
                                        .wrapping_add(scaled)
                                        .wrapping_sub(i64::from(
                                            goods.addon_property_value(factory, attribute, 2),
                                        ));
                                let _ = goods.set_addon_property_value_core(attribute, 1, 0);
                                let _ = goods.set_addon_property_value_core(attribute, 2, 0);
                                let _ = goods.set_addon_property_value_core(
                                    attribute,
                                    1,
                                    clamp_battle_fairy_script_value(balanced),
                                );
                                let _ = goods.set_addon_property_value_core(
                                    attribute,
                                    2,
                                    scaled as i32,
                                );
                            } else {
                                let changed =
                                    i64::from(goods.addon_property_value(factory, attribute, 1))
                                        .wrapping_add(scaled);
                                let _ = goods.set_addon_property_value_core(
                                    attribute,
                                    1,
                                    clamp_battle_fairy_script_value(changed),
                                );
                            }
                        }
                        crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate {
                            message_type: 0x0b_f918,
                            player_id,
                            goods: goods.identity(),
                            old_client_payload: context.encode_goods_for_old_client(goods),
                        }
                    };
                    let _ = self.send_battle_fairy_goods_update(&update);
                }
                if let Some(initiator_id) = script_player_id {
                    if let Some(player) = self.find_player(initiator_id) {
                        let attribute_text = attribute.to_string();
                        let value_text = value.to_string();
                        let text = format_legacy_text_fields(
                            self.get_string_by_id(b"ZHGS0040"),
                            &[
                                player.account(),
                                attribute_text.as_bytes(),
                                value_text.as_bytes(),
                            ],
                            0xff,
                        );
                        put_string_to_file("BattleFairy", &text);
                    }
                }
                0
            }
            BattleFairyScriptAction::ResetSkill {
                player_name,
                position,
            } => {
                let Some(player_id) = target_id(self, &player_name) else {
                    return 0;
                };
                if self
                    .find_player(player_id)
                    .and_then(|player| player.equipment().get_goods(10))
                    .is_none()
                {
                    return 0;
                }
                let _ = self.reset_battle_fairy_skill(player_id, position, false, context);
                0
            }
            BattleFairyScriptAction::Revive { player_name } => {
                let Some(player_id) = target_id(self, &player_name) else {
                    return 0;
                };
                let alive = self.find_player(player_id).is_some_and(|player| {
                    player.equipment().get_goods(10).is_some_and(|goods| {
                        goods.addon_property_value(&self.goods_factory, GAP_BF_HP, 1) > 0
                    })
                });
                if alive {
                    let text = self.get_string_by_id(b"ZHGS0041");
                    let _ = colored_player_notice_message(0xffff_ffff, 0, text)
                        .send_to_player(self.net_server(), player_id);
                    return 0;
                }
                let revived = {
                    let (players, factory) = (&mut self.players, &self.goods_factory);
                    players
                        .get_mut(&player_id)
                        .is_some_and(|player| player.revive_battle_fairy(factory))
                };
                if !revived {
                    return 0;
                }
                let update = self.find_player(player_id).and_then(|player| {
                    player.equipment().get_goods(10).map(|goods| {
                        crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate {
                            message_type: 0x0b_f918,
                            player_id,
                            goods: goods.identity(),
                            old_client_payload: context.encode_goods_for_old_client(goods),
                        }
                    })
                });
                if let Some(update) = update {
                    let _ = self.send_battle_fairy_goods_update(&update);
                }
                let external = context.player_properties_external_facts(player_id);
                if let Some(player) = self.find_player(player_id) {
                    let _ = self.send_player_properties_changed(player, external);
                }
                1
            }
            BattleFairyScriptAction::AddExperience {
                player_name,
                experience,
            } => {
                if experience <= 0 {
                    return 0;
                }
                let Some(target_player_id) = target_id(self, &player_name) else {
                    return 0;
                };
                let account = self
                    .find_player(target_player_id)
                    .map(|player| player.account().to_vec())
                    .unwrap_or_default();
                let Some(source_player_id) = script_player_id else {
                    return 0;
                };
                let mut remaining = experience as u32;
                let report_and_update = {
                    let (players, factory, exp_config) = (
                        &mut self.players,
                        &self.goods_factory,
                        &self.battle_fairy_exp_config,
                    );
                    let Some(goods) = players
                        .get_mut(&source_player_id)
                        .and_then(CPlayer::enhancement_selected_goods_mut)
                    else {
                        return 0;
                    };
                    if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
                        return 0;
                    }
                    let facts = BattleFairyPlayerFacts {
                        player_id: target_player_id,
                        account: &account,
                    };
                    let Ok(Some(report)) = goods.battle_fairy_exp_up(
                        factory,
                        Some(facts),
                        &mut remaining,
                        |equip_level, level| exp_config.dw_exp_up(equip_level, level),
                    ) else {
                        return 0;
                    };
                    if report.result <= BattleFairyExpUpResult::None {
                        return 0;
                    }
                    let _ = goods.save_battle_fairy_property(factory);
                    let update = crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate {
                        message_type: 0x0b_f918,
                        player_id: target_player_id,
                        goods: goods.identity(),
                        old_client_payload: context.encode_goods_for_old_client(goods),
                    };
                    (report, update)
                };
                for log in report_and_update.0.level_logs {
                    let level = log.level.to_string();
                    let text = format_legacy_text_fields(
                        self.get_string_by_id(b"ZHGS0017"),
                        &[&log.account, &log.goods_name, level.as_bytes()],
                        0xff,
                    );
                    put_string_to_file("BattleFairy", &text);
                }
                let _ = self.send_battle_fairy_goods_update(&report_and_update.1);
                0
            }
            BattleFairyScriptAction::RecreateAttributes {
                player_name,
                mode,
                minimum,
                maximum,
            } => {
                let Some(target_player_id) = target_id(self, &player_name) else {
                    return 0;
                };
                let Some(source_player_id) = script_player_id else {
                    return 0;
                };
                let update = {
                    let (players, factory, random_state) = (
                        &mut self.players,
                        &self.goods_factory,
                        &mut self.random_state,
                    );
                    let Some(goods) = players
                        .get_mut(&source_player_id)
                        .and_then(CPlayer::enhancement_selected_goods_mut)
                    else {
                        return 0;
                    };
                    if !factory.recreate_battle_fairy_attributes(
                        goods,
                        mode,
                        minimum,
                        maximum,
                        |upper_bound| game_legacy_random(random_state, upper_bound),
                    ) {
                        return 0;
                    }
                    crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate {
                        message_type: 0x0b_f918,
                        player_id: target_player_id,
                        goods: goods.identity(),
                        old_client_payload: context.encode_goods_for_old_client(goods),
                    }
                };
                let external = context.player_properties_external_facts(target_player_id);
                if let Some(player) = self.find_player(target_player_id) {
                    let _ = self.send_player_properties_changed(player, external);
                }
                let _ = self.send_battle_fairy_goods_update(&update);
                0
            }
        }
    }

    /// Runtime entry point уже декодированного `skillmessage 0x90001`.
    /// Player mutation и effects сохраняют native order: optional contend
    /// notice, безусловный `ClearEmotion 0xBF611`, authorization, socket
    /// reject либо очередь concrete `CPlayerAI`.
    pub(crate) fn request_player_skill<Context: PlayerSkillRequestContext>(
        &mut self,
        player_id: i32,
        socket_id: i32,
        request: PlayerSkillRequest,
        facts: PlayerSkillRequestFacts,
        context: &mut Context,
    ) -> Option<PlayerSkillRequestReport> {
        let report = self
            .players
            .get_mut(&player_id)
            .map(|player| player.request_player_skill(request, facts, &self.skill_factory))?;
        Some(self.deliver_player_skill_report(player_id, socket_id, report, context))
    }

    pub(crate) fn request_item_skill<Context: PlayerSkillRequestContext>(
        &mut self,
        player_id: i32,
        socket_id: i32,
        request: PlayerSkillRequest,
        skill_level: i32,
        facts: PlayerSkillRequestFacts,
        context: &mut Context,
    ) -> Option<PlayerSkillRequestReport> {
        let report = self.players.get_mut(&player_id).map(|player| {
            player.request_item_skill(request, skill_level, facts, &self.skill_factory)
        })?;
        Some(self.deliver_player_skill_report(player_id, socket_id, report, context))
    }

    fn deliver_player_skill_report<Context: PlayerSkillRequestContext>(
        &mut self,
        player_id: i32,
        socket_id: i32,
        mut report: PlayerSkillRequestReport,
        context: &mut Context,
    ) -> PlayerSkillRequestReport {
        for effect in report.effects.clone() {
            match effect {
                PlayerSkillRequestEffect::Notification {
                    player_id,
                    string_id,
                    color,
                    message_type,
                } => {
                    let delivery = colored_player_notice_message(
                        color,
                        message_type,
                        self.get_string_by_id(string_id.as_bytes()),
                    )
                    .send_to_player(self.net_server(), player_id);
                    report
                        .deliveries
                        .push(PlayerSkillRequestDelivery::Player(delivery));
                }
                PlayerSkillRequestEffect::ClearEmotion => {
                    let delivery = report
                        .region_id
                        .and_then(|region_id| self.take_region_owner(region_id))
                        .and_then(|owner| {
                            let delivery = self.find_player(player_id).map(|player| {
                                let mut message = CMessage::new(0x0b_f611);
                                message.add_long(player.shape().identity().object_type);
                                message.add_long(player_id);
                                message.add_long(0);
                                self.send_player_around_excluding_self(
                                    owner.base(),
                                    player.shape(),
                                    player_id,
                                    &message,
                                )
                            });
                            self.restore_region_owner(owner);
                            delivery
                        });
                    report
                        .deliveries
                        .push(PlayerSkillRequestDelivery::EmotionAround(delivery));
                }
                PlayerSkillRequestEffect::SocketReject {
                    message_type,
                    reason,
                    code,
                } => {
                    let mut message = CMessage::new(message_type as i32);
                    message.base_mut().add_byte(reason as u8);
                    message.base_mut().add_byte(code);
                    report
                        .deliveries
                        .push(PlayerSkillRequestDelivery::SocketReject(
                            message.send_to_socket(self.net_server(), socket_id),
                        ));
                }
                PlayerSkillRequestEffect::AiDispatch(dispatch) => {
                    context.queue_player_skill(player_id, dispatch);
                    report.deliveries.push(PlayerSkillRequestDelivery::AiQueued);
                }
            }
        }
        report
    }

    fn send_player_around_excluding_self(
        &self,
        region: &CServerRegion,
        origin: &CShape,
        player_id: i32,
        message: &CMessage,
    ) -> Result<i32, ShapeCoordinateBlock> {
        let Some(runtime) = GameServerAroundRuntime::new(
            self,
            &self.session_factory,
            self.globe_setup.area_width(),
            self.globe_setup.area_height(),
        ) else {
            return Ok(0);
        };
        message.send_to_around(Some(region), origin, Some(player_id), &runtime)
    }

    pub(crate) fn send_player_shape_around(
        &mut self,
        player_id: i32,
        excluded_player_id: Option<i32>,
        message: &CMessage,
    ) -> Option<Result<i32, ShapeCoordinateBlock>> {
        let region_id = self.find_player(player_id)?.server_region_id()?;
        let owner = self.take_region_owner(region_id)?;
        let delivery = self.find_player(player_id).map(|player| {
            let Some(runtime) = GameServerAroundRuntime::new(
                self,
                &self.session_factory,
                self.globe_setup.area_width(),
                self.globe_setup.area_height(),
            ) else {
                return Ok(0);
            };
            message.send_to_around(
                Some(owner.base()),
                player.shape(),
                excluded_player_id,
                &runtime,
            )
        });
        self.restore_region_owner(owner);
        delivery
    }

    pub(crate) fn send_shape_position_around(
        &mut self,
        region_id: i32,
        tile_x: i32,
        tile_y: i32,
        message: &CMessage,
    ) -> Option<i32> {
        let owner = self.take_region_owner(region_id)?;
        let delivery = GameServerAroundRuntime::new(
            self,
            &self.session_factory,
            self.globe_setup.area_width(),
            self.globe_setup.area_height(),
        )
        .map(|runtime| {
            message.send_to_around_position(Some(owner.base()), tile_x, tile_y, None, &runtime)
        })
        .unwrap_or(0);
        self.restore_region_owner(owner);
        Some(delivery)
    }

    pub(crate) fn relocate_player_shape(
        &mut self,
        player_id: i32,
        region_id: i32,
        tile_x: i32,
        tile_y: i32,
    ) -> Option<Result<(), RegionMembershipBlock>> {
        let mut player = self.players.remove(&player_id)?;
        let Some(mut owner) = self.take_region_owner(region_id) else {
            self.players.insert(player_id, player);
            return None;
        };
        let facts = player.movement_position_facts(
            self.globe_setup.area_width(),
            self.globe_setup.area_height(),
        );
        let result = owner.base_mut().set_move_shape_tile_position(
            player.movement_shape_mut(),
            tile_x,
            tile_y,
            facts,
        );
        self.restore_region_owner(owner);
        self.players.insert(player_id, player);
        Some(result)
    }

    /// Exact script `8000 / RefeashBlock`: snapshot заменяет только временные
    /// C++ pointers, пока canonical region mutably пересобирает BLOCK_SHAPE.
    /// Координаты и `!IsDied` берутся у достигнутых player/monster/NPC owners.
    pub(crate) fn refresh_script_region_blocks(
        &mut self,
        region_id: i32,
    ) -> Option<Result<(), RegionMembershipBlock>> {
        let region = self.find_region(region_id)?.base();
        let identities = region.registered_shape_identities();
        let mut resolver = RegionBlockRefreshResolver::default();
        for identity in identities {
            let fact = match identity.object_type {
                PLAYER_TYPE => self
                    .find_player(identity.id)
                    .and_then(|player| player.shape_view().map(|shape| (shape, !player.is_dead()))),
                MONSTER_TYPE => region.find_monster_by_id(identity.id).and_then(|monster| {
                    shape_view(monster.move_shape().shape(), ShapeFigure::default())
                        .map(|shape| (shape, !CMoveShape::is_died(monster.hit_points())))
                }),
                NPC_TYPE => region
                    .find_npc_by_id(identity.id)
                    .and_then(|npc| npc.shape_view().map(|shape| (shape, true))),
                _ => None,
            };
            let Some((shape, is_alive)) = fact else {
                continue;
            };
            resolver.facts.insert(identity, (shape, is_alive));
        }

        let mut owner = self
            .take_region_owner(region_id)
            .expect("script refresh сохраняет найденный region owner");
        let result = owner.base_mut().refresh_blocks(&resolver);
        self.restore_region_owner(owner);
        Some(result)
    }

    /// Runtime bridge script `3010`: удерживает target player и его concrete
    /// region одним mutable проходом, чтобы `CMoveShape::ForceMove` одновременно
    /// опубликовал around wire, переставил spatial membership и поставил AI
    /// stand-event на вычисленную сценарием длительность.
    pub(crate) fn force_move_script_player<Context: MoveShapeCommandContext>(
        &mut self,
        player_id: i32,
        destination_x: i32,
        destination_y: i32,
        duration_ms: u32,
        context: &mut Context,
    ) -> Option<Result<bool, MoveShapeCommandBlock>> {
        let mut player = self.players.remove(&player_id)?;
        let Some(region_id) = player.server_region_id() else {
            self.players.insert(player_id, player);
            return None;
        };
        let Some(mut owner) = self.take_region_owner(region_id) else {
            self.players.insert(player_id, player);
            return None;
        };
        let area_width = self.globe_setup.area_width();
        let area_height = self.globe_setup.area_height();
        let result = {
            let Some(around) =
                GameServerAroundRuntime::new(self, &self.session_factory, area_width, area_height)
            else {
                self.restore_region_owner(owner);
                self.players.insert(player_id, player);
                return None;
            };
            player.force_move(
                owner.base_mut(),
                destination_x,
                destination_y,
                duration_ms,
                area_width,
                area_height,
                &around,
                context,
            )
        };
        self.restore_region_owner(owner);
        self.players.insert(player_id, player);
        Some(result)
    }

    pub(crate) fn find_shape_in_region(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<ShapeView> {
        self.regions.get(&region_id)?.base().find_child_object(
            identity.object_type,
            identity.id,
            identity.ex_id,
            self,
        )
    }

    pub(crate) fn emotion_repeated(&self, emotion_id: i32) -> bool {
        self.emotion.repeated(emotion_id) != 0
    }

    /// Runtime entry point уже декодированного `skillmessage 0x90005`.
    /// Facts оставляют explicit boundaries для ещё сырого `CPlayerAI`,
    /// `SymbolIsAttackAble` и monster registry, не выдавая player-only resolver
    /// текущего `CGame` за полный region lookup.
    pub(crate) fn request_battle_fairy_skill<Context: BattleFairySkillRequestContext>(
        &self,
        player_id: i32,
        socket_id: i32,
        request: BattleFairySkillRequest,
        facts: BattleFairySkillRequestFacts,
        context: &mut Context,
    ) -> Option<BattleFairySkillRequestReport> {
        let enabled = self.globe_setup.battle_fairy_enabled();
        let mut report = self.players.get(&player_id).map(|player| {
            player.request_battle_fairy_skill(
                enabled,
                request,
                facts,
                &self.goods_factory,
                &self.skill_factory,
            )
        })?;
        for effect in report.effects.clone() {
            match effect {
                BattleFairySkillRequestEffect::Notification {
                    player_id,
                    string_id,
                    color,
                    message_type,
                } => {
                    let delivery = colored_player_notice_message(
                        color,
                        message_type,
                        self.get_string_by_id(string_id.as_bytes()),
                    )
                    .send_to_player(self.net_server(), player_id);
                    report
                        .deliveries
                        .push(BattleFairySkillRequestDelivery::Player(delivery));
                }
                BattleFairySkillRequestEffect::SocketReject {
                    message_type,
                    reason,
                    code,
                } => {
                    let mut message = CMessage::new(message_type as i32);
                    message.base_mut().add_byte(reason as u8);
                    message.base_mut().add_byte(code);
                    report
                        .deliveries
                        .push(BattleFairySkillRequestDelivery::SocketReject(
                            message.send_to_socket(self.net_server(), socket_id),
                        ));
                }
                BattleFairySkillRequestEffect::AiDispatch(dispatch) => {
                    context.queue_battle_fairy_skill(player_id, dispatch);
                    report
                        .deliveries
                        .push(BattleFairySkillRequestDelivery::AiQueued);
                }
            }
        }
        Some(report)
    }

    /// Завершает periodic `ComputeWarSoulXY` tick через тот же region area-map,
    /// который обслуживает summon/recall. Skill restored-state остаётся exact
    /// fact ещё не перенесённого concrete `CSkill`.
    pub(crate) fn compute_war_soul_xy(
        &mut self,
        player_id: i32,
        current_war_soul_skill_restored: Option<bool>,
    ) -> Option<BattleFairyFollowReport> {
        let mut report = self
            .players
            .get_mut(&player_id)?
            .compute_war_soul_xy(current_war_soul_skill_restored);
        let Some(action) = report.spatial_action else {
            return Some(report);
        };
        let spatial_applied = report.region_id.is_some_and(|region_id| {
            let Some(region) = self.regions.get_mut(&region_id) else {
                return false;
            };
            match action {
                BattleFairyWarSoulAction::SetPosition { previous, target } => region
                    .base_mut()
                    .set_war_soul_position(player_id as u32, previous, target),
                BattleFairyWarSoulAction::Delete { .. } => false,
            }
        });
        report.spatial_applied = spatial_applied;
        if let Some(player) = self.players.get_mut(&player_id) {
            player.apply_war_soul_action(action, spatial_applied);
        }
        for effect in report.effects.clone() {
            match effect {
                BattleFairyFollowEffect::AroundMove {
                    message_type,
                    player_id,
                    object_type,
                    x,
                    y,
                } => {
                    let mut message = CMessage::new(message_type as i32);
                    message.add_long(player_id);
                    message.add_long(object_type);
                    message.add_ulong(x);
                    message.add_ulong(y);
                    let delivery = report
                        .region_id
                        .and_then(|region_id| self.take_region_owner(region_id))
                        .map(|owner| {
                            let player = self.find_player(player_id);
                            let delivery = player.map(|player| {
                                self.send_battle_fairy_around(
                                    owner.base(),
                                    player.shape(),
                                    &message,
                                )
                            });
                            self.restore_region_owner(owner);
                            delivery
                        })
                        .flatten();
                    report
                        .deliveries
                        .push(BattleFairyFollowDelivery::Around(delivery));
                }
            }
        }
        Some(report)
    }

    fn send_battle_fairy_around(
        &self,
        region: &CServerRegion,
        origin: &CShape,
        message: &CMessage,
    ) -> Result<i32, ShapeCoordinateBlock> {
        let Some(runtime) = GameServerAroundRuntime::new(
            self,
            &self.session_factory,
            self.globe_setup.area_width(),
            self.globe_setup.area_height(),
        ) else {
            return Ok(0);
        };
        message.send_to_around(Some(region), origin, None, &runtime)
    }

    /// Выполняет periodic HP-death prefix `CPlayer::AI` и немедленно замыкает
    /// reached virtual `OnChangeProperties` точным адресным `0xBF721`.
    pub(crate) fn refresh_battle_fairy_death<Context: BattleFairyDeathContext>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> Option<BattleFairyDeathReport> {
        let factory = &self.goods_factory;
        let mut report = self
            .players
            .get_mut(&player_id)
            .map(|player| player.refresh_battle_fairy_death(factory))?;
        if report.outcome == crate::gameserver::appserver::player::BattleFairyDeathOutcome::Died {
            let external = context.player_properties_external_facts(player_id);
            report.property_delivery = self
                .find_player(player_id)
                .map(|player| self.send_player_properties_changed(player, external));
        }
        Some(report)
    }

    fn send_player_properties_changed(
        &self,
        player: &CPlayer,
        external: PlayerPropertiesExternalFacts,
    ) -> i32 {
        let combat = player.combat_properties();
        let base = player.base_properties();
        let mut message = CMessage::new(0xbf721);
        message.add_long(player.player_id());
        message.add_long(player.player_id());
        message.add_ulong(combat.strength);
        message.add_ulong(combat.dexterity);
        message.add_ulong(combat.constitution);
        message.add_ulong(combat.intelligence);
        message.add_ulong(combat.minimum_attack);
        message.add_ulong(combat.maximum_attack);
        message.add_ulong(external.add_element_attack);
        message.add_ulong(combat.element_modify as u32);
        message.base_mut().add_short(combat.attack_speed as i16);
        message.base_mut().add_short(combat.cch as i16);
        message.add_ulong(combat.defense);
        message.add_ulong(combat.element_resistance);
        message.add_ulong(u32::from(combat.burden));
        message.add_ulong(combat.maximum_hp);
        message.add_ulong(base.health);
        message.add_ulong(combat.maximum_mp);
        message.add_ulong(base.mana);
        message.add_ulong(external.maximum_rp);
        message.add_ulong(external.rp);
        message.base_mut().add_short(combat.reank as i16);
        message.add_ulong(external.maximum_vigour);
        message.add_ulong(base.vigour);
        message.add_ulong(base.credit);
        message.add_ulong(base.mode);
        message.add_ulong(u32::from(player.war_soul_state() == 1));
        message.add_ulong(u32::from(base.battle_fairy_recall));
        message.add_ulong(u32::from(base.battle_fairy_died));
        message.add_ulong(external.exalt);
        message.send_to_player(self.net_server(), player.player_id())
    }

    /// Один exact `CGame::RunAuction` pass. Feature gate не читает часы;
    /// strict 999-ms gate читает wall-clock только при срабатывании.
    /// Goods snapshot и оба World sends сохраняют исходный порядок.
    pub(crate) fn run_auction<Runtime: GameMainLoopRuntime>(
        &mut self,
        runtime: &mut Runtime,
    ) -> GameAuctionRunReport {
        if !self.globe_setup.auction_enabled() {
            return GameAuctionRunReport {
                outcome: GameAuctionRunOutcome::FeatureDisabled,
                sampled_tick_ms: None,
                sampled_wall_time_seconds: None,
                synchronized_goods: Vec::new(),
                goods_sync: None,
                state_request: None,
            };
        }

        let sampled_tick_ms = runtime.get_tick_ms();
        let elapsed_ms = sampled_tick_ms.wrapping_sub(self.auction_tick_ms);
        if elapsed_ms <= 999 {
            return GameAuctionRunReport {
                outcome: GameAuctionRunOutcome::TickNotDue { elapsed_ms },
                sampled_tick_ms: Some(sampled_tick_ms),
                sampled_wall_time_seconds: None,
                synchronized_goods: Vec::new(),
                goods_sync: None,
                state_request: None,
            };
        }
        self.auction_tick_ms = sampled_tick_ms;

        let synchronized_goods = if self.auction_now {
            self.auction_room.count_goods()
        } else {
            Vec::new()
        };
        let goods_sync = if self.auction_now {
            let mut message = CMessage::new(GAME_AUCTION_GOODS_SYNC_MESSAGE);
            message.add_ulong(synchronized_goods.len() as u32);
            for guid in &synchronized_goods {
                message.base_mut().add_guid(*guid);
            }
            Some(message.send(self, false))
        } else {
            None
        };

        let sampled_wall_time_seconds = runtime.wall_time_seconds();
        let state_expired =
            self.auction_last_check_seconds.wrapping_add(4) < sampled_wall_time_seconds;
        if state_expired {
            self.auction_now = false;
        }
        let state_request = CMessage::new(GAME_AUCTION_STATE_REQUEST_MESSAGE).send(self, false);

        GameAuctionRunReport {
            outcome: GameAuctionRunOutcome::Processed { state_expired },
            sampled_tick_ms: Some(sampled_tick_ms),
            sampled_wall_time_seconds: Some(sampled_wall_time_seconds),
            synchronized_goods,
            goods_sync,
            state_request: Some(state_request),
        }
    }

    /// Exact `ReCollectBaiTanInGs`: signed player-map order, null-free owned
    /// traversal и `SendToGSBaiTan` progress gate перед каждым `0x60811`.
    pub(crate) fn recollect_personal_shops(&self) -> Vec<PersonalShopRecollection> {
        self.players
            .values()
            .filter(|player| player.current_progress() == PlayerProgress::OpenStall)
            .map(|player| {
                let player_id = player.player_id();
                let client_ip = player.client_ip();
                let mut request = CMessage::new(0x0006_0811);
                request.base_mut().add_long(player_id);
                request.base_mut().add_ulong(client_ip);
                PersonalShopRecollection {
                    player_id,
                    client_ip,
                    delivery: request.send(self, false),
                }
            })
            .collect()
    }

    pub(crate) fn start_region_clear_player(
        &mut self,
        region_id: i32,
        buffer_seconds: i32,
        now_ms: u32,
    ) -> Option<GameRegionClearStarted> {
        let region = self.find_region_mut(region_id)?.base_mut();
        let report = GameRegionClearStarted {
            region_id,
            buffer_seconds,
            delay_ms: buffer_seconds.wrapping_mul(1_000),
            previous_active: region.kick_out_player,
            previous_remaining_ms: region.kick_out_player_time,
            started_at_ms: now_ms,
        };
        region.start_clear_player_out_at(report.delay_ms, now_ms);
        Some(report)
    }

    /// Exact `CGame::AI`: signed region-map order и virtual region AI,
    /// после которого выполняется base-tail `ClearPlayerAI` того же owner-а.
    pub(crate) fn ai<Runtime: GameMainLoopRuntime>(
        &mut self,
        runtime: &mut Runtime,
    ) -> GameAiReport {
        if self.net_server.is_none() {
            return GameAiReport {
                legacy_return: 1,
                regions: Vec::new(),
            };
        }
        let region_ids: Vec<_> = self.regions.keys().copied().collect();
        let mut regions = Vec::with_capacity(region_ids.len());
        for region_id in region_ids {
            let gods_battle = if self
                .find_region(region_id)
                .is_some_and(ServerRegionOwner::is_gods_battle)
            {
                self.gods_battle_contend_ai(region_id, runtime)
            } else {
                runtime.region_ai_before_clear_player(self, region_id);
                None
            };
            let Some(mut owner) = self.take_region_owner(region_id) else {
                continue;
            };
            let region_changes: Vec<GameLocalRegionChange> = owner
                .base_mut()
                .take_staged_region_transitions()
                .into_iter()
                .filter(|identity| identity.object_type == 400)
                .filter_map(|identity| {
                    let mut player = self.players.remove(&identity.id)?;
                    let facts = ShapeRuntimeFacts {
                        is_player: true,
                        monster: None,
                        is_npc: false,
                        goods: None,
                        is_move_shape: true,
                        figure: player.figure(),
                    };
                    let removal = owner
                        .base_mut()
                        .remove_object(player.movement_shape_mut(), facts);
                    let destination = player.apply_staged_local_region_change();
                    self.players.insert(identity.id, player);
                    Some(GameLocalRegionChange {
                        player_id: identity.id,
                        destination,
                        removal,
                    })
                })
                .collect();
            let clear_player = if owner.base().kick_out_player {
                let tick = owner.base_mut().clear_player_ai_at(runtime.get_tick_ms());
                match tick {
                    ServerRegionClearPlayerTick::Waiting {
                        remaining_ms,
                        elapsed_ms,
                    } => Some(GameRegionClearPlayerOutcome::Waiting {
                        remaining_ms,
                        elapsed_ms,
                    }),
                    ServerRegionClearPlayerTick::Warning {
                        remaining_ms,
                        seconds,
                    } => {
                        let text = format_single_legacy_i32(
                            self.get_string_by_id(b"GS0230"),
                            seconds,
                            0xff,
                        );
                        let mut warning = CMessage::new(0x000b_f807);
                        warning.base_mut().add_long(-1);
                        add_legacy_c_string(warning.base_mut(), &text);
                        let delivery = warning.send_to_region(Some(owner.base()), None, self);
                        Some(GameRegionClearPlayerOutcome::Warning {
                            remaining_ms,
                            seconds,
                            delivery,
                        })
                    }
                    ServerRegionClearPlayerTick::Expired => {
                        let player_ids = owner.base().registered_player_ids();
                        self.restore_region_owner(owner);
                        for change in &region_changes {
                            self.change_body_after_region_transition(change.player_id, runtime);
                        }
                        let players = player_ids
                            .into_iter()
                            .map(|player_id| {
                                self.return_region_player(region_id, player_id, runtime)
                            })
                            .collect();
                        regions.push(GameRegionAiReport {
                            region_id,
                            gods_battle,
                            region_changes,
                            clear_player: Some(GameRegionClearPlayerOutcome::Expired { players }),
                        });
                        continue;
                    }
                }
            } else {
                None
            };
            self.restore_region_owner(owner);
            for change in &region_changes {
                self.change_body_after_region_transition(change.player_id, runtime);
            }
            regions.push(GameRegionAiReport {
                region_id,
                gods_battle,
                region_changes,
                clear_player,
            });
        }
        GameAiReport {
            legacy_return: 1,
            regions,
        }
    }

    fn return_region_player<Runtime: GameMainLoopRuntime>(
        &mut self,
        source_region_id: i32,
        player_id: i32,
        runtime: &mut Runtime,
    ) -> GameReturnedRegionPlayer {
        let Some(player) = self.find_player(player_id) else {
            return GameReturnedRegionPlayer {
                player_id,
                point: Err(GameReturnPointBlock::PlayerMissing),
                destination: None,
                random_block: None,
                changed_region: None,
            };
        };
        let facts = ServerReturnPlayer {
            id: player_id,
            country: player.country(),
            faction_id: player.faction_id(),
        };
        let direction = player.shape().get_direction();
        let city_facts = CityReturnPointFacts {
            player_id,
            tile_x: player.shape().get_tile_x().unwrap_or_default(),
            tile_y: player.shape().get_tile_y().unwrap_or_default(),
        };
        let point = if self
            .find_region(source_region_id)
            .is_some_and(ServerRegionOwner::is_gods_battle)
        {
            self.gods_battle_return_point(source_region_id, player_id, runtime)
                .map_err(GameReturnPointBlock::Base)
                .and_then(|point| {
                    point
                        .map(|point| point.point)
                        .ok_or(GameReturnPointBlock::GodsBattleMissing)
                })
        } else {
            let Some(mut owner) = self.take_region_owner(source_region_id) else {
                return GameReturnedRegionPlayer {
                    player_id,
                    point: Err(GameReturnPointBlock::GodsBattleMissing),
                    destination: None,
                    random_block: None,
                    changed_region: None,
                };
            };
            let mut city_facts = city_facts;
            let point = match &mut owner {
                ServerRegionOwner::Base(region) => region
                    .get_return_point(Some(facts), &mut self.country_param)
                    .map_err(GameReturnPointBlock::Base),
                ServerRegionOwner::Village(region) => region
                    .war
                    .base
                    .get_return_point(Some(facts), &mut self.country_param)
                    .map_err(GameReturnPointBlock::Base),
                ServerRegionOwner::City(region) => region
                    .get_return_point(Some(facts), &mut self.country_param, &mut city_facts)
                    .map_err(GameReturnPointBlock::City),
                ServerRegionOwner::Country(region) => region
                    .get_return_point(Some(facts), &mut self.country_param, runtime)
                    .map_err(GameReturnPointBlock::Country),
                ServerRegionOwner::Nation(region) => region
                    .war
                    .base
                    .get_return_point(Some(facts), &mut self.country_param)
                    .map_err(GameReturnPointBlock::Base),
                ServerRegionOwner::GodsBattle(_) => unreachable!("GodsBattle обработан до take"),
            };
            self.restore_region_owner(owner);
            point
        };
        let Ok(point_value) = point else {
            return GameReturnedRegionPlayer {
                player_id,
                point,
                destination: None,
                random_block: None,
                changed_region: None,
            };
        };
        let width = point_value.right.wrapping_sub(point_value.left);
        let height = point_value.bottom.wrapping_sub(point_value.top);
        let mut x = point_value.left.wrapping_add(width / 2);
        let mut y = point_value.top.wrapping_add(height / 2);
        let mut random_block = None;
        if width > 0 && height > 0 {
            if let Some(destination) = self.find_region(point_value.region_id) {
                match destination.base().region.get_random_pos_in_range(
                    point_value.left,
                    point_value.top,
                    width,
                    height,
                    runtime,
                ) {
                    Ok(position) => {
                        x = position.x;
                        y = position.y;
                    }
                    Err(block) => random_block = Some(block),
                }
            }
        }
        let changed_region = runtime.change_relived_player_region(
            player_id,
            point_value.region_id,
            x,
            y,
            direction,
            0,
        );
        GameReturnedRegionPlayer {
            player_id,
            point: Ok(point_value),
            destination: Some((x, y)),
            random_block,
            changed_region: Some(changed_region),
        }
    }

    /// Один exact `CGame::MainLoop` turn. Wrapping DWORD clocks, strict
    /// interval comparisons, profiling reads и pacing deadline сохраняют
    /// наблюдаемый Win32 порядок; wait заменён platform callback-ом.
    pub(crate) fn main_loop<Runtime: GameMainLoopRuntime>(
        &mut self,
        runtime: &mut Runtime,
    ) -> GameMainLoopReport<Runtime::RuntimeError> {
        let mut state = self.main_loop_state;
        if !state.initialized {
            state.current_tick_ms = runtime.get_tick_ms();
            state.runtime_log_tick_ms = state.current_tick_ms;
            state.refresh_info_tick_ms = state.current_tick_ms;
            state.initialized = true;
        }

        state.current_tick_ms = runtime.get_tick_ms();
        self.expire_script_faction_sessions(state.current_tick_ms);
        self.expire_change_body_states(state.current_tick_ms, runtime);
        self.update_extended_states(state.current_tick_ms, runtime);
        self.update_appellation_states(state.current_tick_ms, runtime);
        self.update_ride_states(state.current_tick_ms, runtime);
        state.calls_since_runtime_log = state.calls_since_runtime_log.wrapping_add(1);
        let mut stages = Vec::new();

        let refresh_elapsed = state
            .current_tick_ms
            .wrapping_sub(state.refresh_info_tick_ms);
        if self.setup.refresh_info_time_ms < refresh_elapsed {
            state.refresh_info_tick_ms = state.current_tick_ms;
            runtime.refresh_info_text(self);
            stages.push(GameMainLoopStage::RefreshInfo);
        }

        let runtime_elapsed = state
            .current_tick_ms
            .wrapping_sub(state.runtime_log_tick_ms);
        if self.setup.watch_runtime_time_ms < runtime_elapsed {
            let log = if self.setup.watch_runtime_info {
                let profile = state.profile;
                state.profile = GameMainLoopProfile::default();
                GameMainLoopRuntimeLog::Profiled {
                    elapsed_ms: runtime_elapsed,
                    ai_calls: state.calls_since_runtime_log,
                    profile,
                }
            } else {
                GameMainLoopRuntimeLog::Compact {
                    elapsed_ms: runtime_elapsed,
                    ai_calls: state.calls_since_runtime_log,
                }
            };
            runtime.add_runtime_log(log);
            stages.push(GameMainLoopStage::RuntimeLog);
            state.calls_since_runtime_log = 0;
            state.runtime_log_tick_ms = state.current_tick_ms;
        }

        if runtime.exit_requested() {
            self.main_loop_state = state;
            return GameMainLoopReport {
                outcome: GameMainLoopOutcome::ExitRequested,
                return_value: 0,
                sampled_tick_ms: state.current_tick_ms,
                ai_tick: state.ai_tick,
                stages,
                next_deadline_ms: state.pacing_initialized.then_some(state.pacing_deadline_ms),
                signed_lag_ms: None,
                ai: None,
                messages: None,
                net_sessions: None,
                auction: None,
            };
        }

        state.ai_tick = state.ai_tick.wrapping_add(1);
        let ai;
        let messages;
        let net_sessions;
        if self.setup.watch_runtime_info {
            let started = runtime.get_tick_ms();
            let _scripts = self.run_script_loop(runtime);
            state.profile.script_ms = state
                .profile
                .script_ms
                .wrapping_add(runtime.get_tick_ms().wrapping_sub(started));
            stages.push(GameMainLoopStage::Script);

            let started = runtime.get_tick_ms();
            ai = self.ai(runtime);
            let _fairy_hatchers = self.run_fairy_hatchers(runtime);
            state.profile.ai_ms = state
                .profile
                .ai_ms
                .wrapping_add(runtime.get_tick_ms().wrapping_sub(started));
            stages.push(GameMainLoopStage::Ai);
            stages.push(GameMainLoopStage::FairyHatcher);

            let started = runtime.get_tick_ms();
            messages = self.process_messages(runtime);
            state.profile.message_ms = state
                .profile
                .message_ms
                .wrapping_add(runtime.get_tick_ms().wrapping_sub(started));
            stages.push(GameMainLoopStage::Message);

            let started = runtime.get_tick_ms();
            let terminal_equipment_sessions = self
                .session_factory
                .garbage_collect_terminal_equipment_sessions();
            self.detach_terminal_equipment_session_listeners(&terminal_equipment_sessions);
            state.profile.session_ms = state
                .profile
                .session_ms
                .wrapping_add(runtime.get_tick_ms().wrapping_sub(started));
            stages.push(GameMainLoopStage::Session);

            let started = runtime.get_tick_ms();
            net_sessions = self.net_session_manager.run();
            state.profile.net_session_ms = state
                .profile
                .net_session_ms
                .wrapping_add(runtime.get_tick_ms().wrapping_sub(started));
            stages.push(GameMainLoopStage::NetSession);
        } else {
            let _scripts = self.run_script_loop(runtime);
            stages.push(GameMainLoopStage::Script);
            ai = self.ai(runtime);
            stages.push(GameMainLoopStage::Ai);
            let _fairy_hatchers = self.run_fairy_hatchers(runtime);
            stages.push(GameMainLoopStage::FairyHatcher);
            messages = self.process_messages(runtime);
            stages.push(GameMainLoopStage::Message);
            let terminal_equipment_sessions = self
                .session_factory
                .garbage_collect_terminal_equipment_sessions();
            self.detach_terminal_equipment_session_listeners(&terminal_equipment_sessions);
            stages.push(GameMainLoopStage::Session);
            net_sessions = self.net_session_manager.run();
            stages.push(GameMainLoopStage::NetSession);
        }

        let auction = self.run_auction(runtime);
        stages.push(GameMainLoopStage::Auction);

        if !state.pacing_initialized {
            state.pacing_deadline_ms = runtime.get_tick_ms();
            state.pacing_initialized = true;
        }
        let pacing_tick_ms = runtime.get_tick_ms();
        state.current_tick_ms = pacing_tick_ms;
        let interval_ms = runtime.tick_interval_ms();
        if pacing_tick_ms.wrapping_sub(state.pacing_deadline_ms) < interval_ms {
            let duration_ms = state
                .pacing_deadline_ms
                .wrapping_sub(pacing_tick_ms)
                .wrapping_add(interval_ms);
            runtime.wait(duration_ms);
            stages.push(GameMainLoopStage::Wait { duration_ms });
        }

        state.pacing_deadline_ms = state.pacing_deadline_ms.wrapping_add(interval_ms);
        let signed_lag_ms = pacing_tick_ms.wrapping_sub(state.pacing_deadline_ms) as i32;
        if 1_000 < signed_lag_ms {
            runtime.output_debug("warning!!! 1 second not call AI()\n");
            let resync_tick_ms = runtime.get_tick_ms();
            state.pacing_deadline_ms = resync_tick_ms;
            stages.push(GameMainLoopStage::LagWarning { resync_tick_ms });
        }

        self.main_loop_state = state;
        GameMainLoopReport {
            outcome: GameMainLoopOutcome::Continue,
            return_value: 1,
            sampled_tick_ms: pacing_tick_ms,
            ai_tick: state.ai_tick,
            stages,
            next_deadline_ms: Some(state.pacing_deadline_ms),
            signed_lag_ms: Some(signed_lag_ms),
            ai: Some(ai),
            messages: Some(messages),
            net_sessions: Some(net_sessions),
            auction: Some(auction),
        }
    }

    /// Исполняет один исходный snapshot входящих FIFO в порядке WS, BS, GS.
    pub(crate) fn process_messages<Runtime: GameMainLoopRuntime>(
        &mut self,
        runtime: &mut Runtime,
    ) -> GameProcessMessagesReport<Runtime::RuntimeError> {
        let mut auction_messages = Vec::new();
        let mut player_shop_messages = Vec::new();
        let mut shop_messages = Vec::new();
        let mut gm_messages = Vec::new();
        let mut gma_messages = Vec::new();
        let mut depot_messages = Vec::new();
        let mut increment_shop_messages = Vec::new();
        let mut increment_shop_billing_messages = Vec::new();
        let mut organizing_messages = Vec::new();
        let mut country_war_messages = Vec::new();
        let mut goods_war_messages = Vec::new();
        let mut container_messages = Vec::new();
        let mut goods_messages = Vec::new();
        let mut skill_messages = Vec::new();
        let mut shape_messages = Vec::new();
        let mut other_messages = Vec::new();
        let mut player_messages = Vec::new();
        let mut log_messages = Vec::new();
        let mut server_messages = Vec::new();
        let world_messages = self
            .world_client
            .as_ref()
            .map(CMyNetClient::take_all_messages)
            .unwrap_or_default();
        for mut message in world_messages {
            self.run_incoming_message(
                &mut message,
                runtime,
                &mut auction_messages,
                &mut player_shop_messages,
                &mut shop_messages,
                &mut gm_messages,
                &mut gma_messages,
                &mut depot_messages,
                &mut increment_shop_messages,
                &mut increment_shop_billing_messages,
                &mut organizing_messages,
                &mut country_war_messages,
                &mut goods_war_messages,
                &mut container_messages,
                &mut goods_messages,
                &mut skill_messages,
                &mut shape_messages,
                &mut other_messages,
                &mut player_messages,
                &mut log_messages,
                &mut server_messages,
            );
        }
        let billing_messages = self
            .billing_client
            .as_ref()
            .map(CMyNetClient::take_all_messages)
            .unwrap_or_default();
        for mut message in billing_messages {
            self.run_incoming_message(
                &mut message,
                runtime,
                &mut auction_messages,
                &mut player_shop_messages,
                &mut shop_messages,
                &mut gm_messages,
                &mut gma_messages,
                &mut depot_messages,
                &mut increment_shop_messages,
                &mut increment_shop_billing_messages,
                &mut organizing_messages,
                &mut country_war_messages,
                &mut goods_war_messages,
                &mut container_messages,
                &mut goods_messages,
                &mut skill_messages,
                &mut shape_messages,
                &mut other_messages,
                &mut player_messages,
                &mut log_messages,
                &mut server_messages,
            );
        }
        let server_events = self
            .net_server
            .as_ref()
            .map(CMyNetServer::take_all_events)
            .unwrap_or_default();
        for event in server_events {
            match event {
                GameServerEvent::Message(mut message) => {
                    self.run_incoming_message(
                        &mut message,
                        runtime,
                        &mut auction_messages,
                        &mut player_shop_messages,
                        &mut shop_messages,
                        &mut gm_messages,
                        &mut gma_messages,
                        &mut depot_messages,
                        &mut increment_shop_messages,
                        &mut increment_shop_billing_messages,
                        &mut organizing_messages,
                        &mut country_war_messages,
                        &mut goods_war_messages,
                        &mut container_messages,
                        &mut goods_messages,
                        &mut skill_messages,
                        &mut shape_messages,
                        &mut other_messages,
                        &mut player_messages,
                        &mut log_messages,
                        &mut server_messages,
                    );
                }
                GameServerEvent::WorldClientReconnected(client) => {
                    runtime.handle_world_client_reconnected(self, client);
                }
                GameServerEvent::BillingClientReconnected(client) => {
                    let _legacy_ignored = on_billing_client_reconnected(self, client);
                }
            }
        }
        GameProcessMessagesReport {
            legacy_return: 1,
            auction_messages,
            player_shop_messages,
            shop_messages,
            gm_messages,
            gma_messages,
            depot_messages,
            increment_shop_messages,
            increment_shop_billing_messages,
            organizing_messages,
            country_war_messages,
            goods_war_messages,
            container_messages,
            goods_messages,
            skill_messages,
            shape_messages,
            other_messages,
            player_messages,
            log_messages,
            server_messages,
        }
    }

    fn run_incoming_message<Runtime: GameMainLoopRuntime>(
        &mut self,
        message: &mut CMessage,
        runtime: &mut Runtime,
        auction_messages: &mut Vec<Result<WorldAuctionMessageReport, WorldAuctionMessageError>>,
        player_shop_messages: &mut Vec<Result<PlayerShopMessageReport, PlayerShopMessageError>>,
        shop_messages: &mut Vec<Result<ShopMessageReport, ShopMessageError>>,
        gm_messages: &mut Vec<Result<GmMessageReport, GmMessageError>>,
        gma_messages: &mut Vec<Result<GmaMessageReport, GmaMessageError>>,
        depot_messages: &mut Vec<DepotMessageReport>,
        increment_shop_messages: &mut Vec<
            Result<GameIncrementShopMessageReport, GameIncrementShopMessageError>,
        >,
        increment_shop_billing_messages: &mut Vec<
            Result<IncrementShopBillingReport, IncrementShopBillingMessageError>,
        >,
        organizing_messages: &mut Vec<
            Result<GameOrganizingMessageReport, GameOrganizingMessageError>,
        >,
        country_war_messages: &mut Vec<
            Result<
                GameCountryWarMessageReport,
                CountryWarMessageDispatchError<CountryBattleStateBlock>,
            >,
        >,
        goods_war_messages: &mut Vec<Result<GameGoodsWarMessageReport, GameGoodsWarMessageError>>,
        container_messages: &mut Vec<Result<GameContainerMessageReport, GameContainerMessageError>>,
        goods_messages: &mut Vec<Result<GameGoodsMessageReport, GameGoodsMessageError>>,
        skill_messages: &mut Vec<Result<GameSkillMessageReport, GameSkillMessageError>>,
        shape_messages: &mut Vec<Result<GameShapeMessageReport, GameShapeMessageError>>,
        other_messages: &mut Vec<Result<GameOtherMessageReport, GameOtherMessageError>>,
        player_messages: &mut Vec<Result<GamePlayerMessageReport, GamePlayerMessageError>>,
        log_messages: &mut Vec<Result<GameLogMessageReport, GameLogMessageError>>,
        server_messages: &mut Vec<
            Result<GameServerMessageReport, GameServerMessageError<Runtime::RuntimeError>>,
        >,
    ) {
        if let Some(report) =
            dispatch_server_message(message, self, runtime, |runtime| runtime.get_tick_ms())
        {
            server_messages.push(report);
        } else if dispatch_client_auction_message(message, self, runtime, |runtime| {
            runtime.get_tick_ms()
        })
        .is_some()
        {
        } else if let Some(report) =
            dispatch_world_auction_message(message, self, runtime, |runtime| runtime.get_tick_ms())
        {
            auction_messages.push(report);
        } else if let Some(report) = dispatch_player_shop_message(message, self, runtime) {
            player_shop_messages.push(report);
        } else if let Some(report) = dispatch_shop_message(message, self, runtime) {
            shop_messages.push(report);
        } else if let Some(report) = dispatch_gm_message(message, self, runtime) {
            gm_messages.push(report);
        } else if let Some(report) = dispatch_gma_message(message, self) {
            gma_messages.push(report);
        } else if let Some(report) = dispatch_depot_message(message, self) {
            depot_messages.push(report);
        } else if let Some(report) = dispatch_increment_shop_message(message, self) {
            increment_shop_messages.push(report);
        } else if let Some(report) = dispatch_increment_shop_billing_message(message, self, runtime)
        {
            increment_shop_billing_messages.push(report);
        } else if let Some(report) = dispatch_game_organizing_message(message, self, runtime) {
            organizing_messages.push(report);
        } else if let Some(report) = dispatch_game_country_war_message(message, self, runtime) {
            country_war_messages.push(report);
        } else if let Some(report) = dispatch_game_goods_war_message(message, self) {
            goods_war_messages.push(report);
        } else if let Some(report) = dispatch_game_container_message(message, self, runtime) {
            container_messages.push(report);
        } else if let Some(report) = dispatch_game_goods_message(message, self, runtime) {
            goods_messages.push(report);
        } else if let Some(report) = dispatch_game_skill_message(message, self, runtime) {
            skill_messages.push(report);
        } else if dispatch_game_region_message(message, self, runtime).is_some() {
        } else if let Some(report) = dispatch_game_shape_message(message, self, runtime) {
            shape_messages.push(report);
        } else if let Some(report) = dispatch_game_other_message(message, self, runtime) {
            other_messages.push(report);
        } else if let Some(report) = dispatch_game_player_message(message, self, runtime) {
            player_messages.push(report);
        } else if let Some(report) = dispatch_game_log_message(message, self, runtime) {
            log_messages.push(report);
        } else {
            message.run(self, runtime);
        }
    }
}

/// Safe process-owned замена `GameThreadFunc`: COM/platform notifications
/// остаются runtime callbacks, а `CGame` всегда проходит Release даже после
/// неуспешного Init, как исходный ненулевой singleton `GetGame`.
pub(crate) async fn game_thread_func<Runtime: GameThreadRuntime>(
    game: &mut CGame,
    runtime: &mut Runtime,
) -> GameThreadReport {
    runtime.initialize_com();
    let paths = runtime.runtime_paths();
    let wall_time_seconds = runtime.wall_time_seconds();
    let sequence_seed_ms = runtime.sequence_seed_ms();
    let initialization = game.init(&paths, wall_time_seconds, sequence_seed_ms).await;

    let mut main_loop_calls = 0usize;
    if initialization.is_ok() {
        loop {
            let turn = game.main_loop(runtime);
            main_loop_calls = main_loop_calls.wrapping_add(1);
            if turn.return_value == 0 {
                break;
            }
        }
    }

    let release = game.release(runtime).await;
    runtime.signal_game_thread_exit();
    runtime.post_process_close();
    runtime.uninitialize_com();
    GameThreadReport {
        initialization,
        main_loop_calls,
        release,
    }
}

impl Default for CGame {
    fn default() -> Self {
        Self::new()
    }
}

fn missing_setup_reconnect(direction: GameUpstreamDirection) -> GameReconnectPublication {
    GameReconnectPublication::Failed {
        attempts: vec![GameConnectAttempt {
            direction,
            failure: Some(GameClientInitializationFailure::MissingNetworkSetup),
        }],
    }
}

fn next_msvc_rand(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(214_013).wrapping_add(2_531_011);
    (*state >> 16) & 0x7fff
}

fn game_legacy_random(state: &mut u32, upper_bound: i32) -> i32 {
    if upper_bound <= 0 {
        return 0;
    }
    loop {
        let value = (i64::from(next_msvc_rand(state)) * i64::from(upper_bound) / 0x7fff) as i32;
        if value != upper_bound || value <= 0 {
            return value;
        }
    }
}

async fn stop_reconnect_task(task: &mut Option<GameReconnectTask>) {
    let Some(task) = task.take() else {
        return;
    };
    task.exit_requested.store(true, Ordering::Release);
    let _legacy_ignored = task.handle.await;
}

async fn run_world_reconnect_task(
    setup: GameNetworkSetup,
    publisher: GameServerEventPublisher,
    exit_requested: Arc<AtomicBool>,
) -> GameReconnectWorkerEnd {
    loop {
        if exit_requested.load(Ordering::Acquire) {
            return GameReconnectWorkerEnd::ExitRequested;
        }
        tokio::time::sleep(RECONNECT_RETRY_DELAY).await;
        if matches!(
            reconnect_world_with(setup.clone(), Some(publisher.clone())).await,
            GameReconnectPublication::Published { .. }
        ) {
            return GameReconnectWorkerEnd::Published;
        }
    }
}

async fn run_billing_reconnect_task(
    setup: GameNetworkSetup,
    publisher: GameServerEventPublisher,
    exit_requested: Arc<AtomicBool>,
) -> GameReconnectWorkerEnd {
    loop {
        if exit_requested.load(Ordering::Acquire) {
            return GameReconnectWorkerEnd::ExitRequested;
        }
        tokio::time::sleep(RECONNECT_RETRY_DELAY).await;
        if matches!(
            reconnect_billing_with(setup.clone(), Some(publisher.clone())).await,
            GameReconnectPublication::Published { .. }
        ) {
            return GameReconnectWorkerEnd::Published;
        }
    }
}

async fn reconnect_world_with(
    setup: GameNetworkSetup,
    publisher: Option<GameServerEventPublisher>,
) -> GameReconnectPublication {
    let Some(publisher) = publisher else {
        return GameReconnectPublication::Failed {
            attempts: vec![GameConnectAttempt {
                direction: GameUpstreamDirection::World,
                failure: Some(GameClientInitializationFailure::MissingNetworkServerOwner),
            }],
        };
    };
    let mut client = CMyNetClient::new();
    client.set_server_type(ServerType::World);
    let endpoint = match connect_game_client(
        &mut client,
        &setup.world,
        GameUpstreamDirection::World,
        None,
        0,
    )
    .await
    {
        Ok(endpoint) => endpoint,
        Err(failure) => {
            let _legacy_result = client.close();
            return GameReconnectPublication::Failed {
                attempts: vec![GameConnectAttempt {
                    direction: GameUpstreamDirection::World,
                    failure: Some(failure),
                }],
            };
        }
    };
    publisher.publish_reconnected_world_client(client);
    GameReconnectPublication::Published {
        endpoint,
        used_billing_backup: false,
        attempts: vec![GameConnectAttempt {
            direction: GameUpstreamDirection::World,
            failure: None,
        }],
    }
}

async fn reconnect_billing_with(
    setup: GameNetworkSetup,
    publisher: Option<GameServerEventPublisher>,
) -> GameReconnectPublication {
    let Some(publisher) = publisher else {
        return GameReconnectPublication::Failed {
            attempts: vec![GameConnectAttempt {
                direction: GameUpstreamDirection::BillingPrimary,
                failure: Some(GameClientInitializationFailure::MissingNetworkServerOwner),
            }],
        };
    };
    let bind_ip = match resolve_billing_bind_ipv4(&setup.billing.bind_ip) {
        Ok(address) => address,
        Err(failure) => {
            return GameReconnectPublication::Failed {
                attempts: vec![GameConnectAttempt {
                    direction: GameUpstreamDirection::BillingPrimary,
                    failure: Some(failure),
                }],
            };
        }
    };
    let mut client = CMyNetClient::new();
    client.set_server_type(ServerType::Billing);
    let mut attempts = Vec::with_capacity(2);
    let mut connected = None;
    for (endpoint_plan, direction) in [
        (
            &setup.billing.primary,
            GameUpstreamDirection::BillingPrimary,
        ),
        (&setup.billing.backup, GameUpstreamDirection::BillingBackup),
    ] {
        match connect_game_client(
            &mut client,
            endpoint_plan,
            direction,
            Some(bind_ip),
            setup.billing.bind_port,
        )
        .await
        {
            Ok(endpoint) => {
                attempts.push(GameConnectAttempt {
                    direction,
                    failure: None,
                });
                connected = Some((endpoint, direction));
                break;
            }
            Err(failure) => attempts.push(GameConnectAttempt {
                direction,
                failure: Some(failure),
            }),
        }
    }
    let Some((endpoint, direction)) = connected else {
        let _legacy_result = client.close();
        return GameReconnectPublication::Failed { attempts };
    };
    publisher.publish_reconnected_billing_client(client);
    GameReconnectPublication::Published {
        endpoint,
        used_billing_backup: direction == GameUpstreamDirection::BillingBackup,
        attempts,
    }
}

async fn connect_game_client(
    client: &mut CMyNetClient,
    endpoint_plan: &GameUpstreamEndpoint,
    direction: GameUpstreamDirection,
    bind_ip: Option<Ipv4Addr>,
    bind_port: u32,
) -> Result<SocketAddrV4, GameClientInitializationFailure> {
    let socket =
        bind_tcp_ipv4(bind_ip, bind_port).map_err(GameClientInitializationFailure::Bind)?;
    let endpoint = resolve_game_endpoint(endpoint_plan, direction)?;
    client
        .connect(socket, endpoint)
        .await
        .map_err(|source| GameClientInitializationFailure::Connect { direction, source })?;
    Ok(endpoint)
}

fn resolve_game_endpoint(
    endpoint: &GameUpstreamEndpoint,
    direction: GameUpstreamDirection,
) -> Result<SocketAddrV4, GameClientInitializationFailure> {
    let host = legacy_c_string_prefix(&endpoint.host);
    if host.len() > 63 {
        // BLOCKED_MISSING_FACT: Game CClient::Connect RVA 0x00019CE0 копировал
        // hostname без проверки в `char[64]`; переполнение не имитируется.
        return Err(GameClientInitializationFailure::AddressTooLong {
            direction,
            length: host.len(),
        });
    }
    let host = std::str::from_utf8(host)
        .map_err(|_| GameClientInitializationFailure::AddressEncodingUnsupported { direction })?;
    (host, endpoint.port as u16)
        .to_socket_addrs()
        .ok()
        .and_then(|mut addresses| {
            addresses.find_map(|address| match address {
                SocketAddr::V4(address) => Some(address),
                SocketAddr::V6(_) => None,
            })
        })
        .ok_or(GameClientInitializationFailure::AddressResolution { direction })
}

fn resolve_billing_bind_ipv4(raw: &[u8]) -> Result<Ipv4Addr, GameClientInitializationFailure> {
    std::str::from_utf8(legacy_c_string_prefix(raw))
        .ok()
        .and_then(|value| value.parse().ok())
        .ok_or(GameClientInitializationFailure::BillingBindAddressResolution)
}

fn add_legacy_c_string(message: &mut crate::nets::basemessage::CBaseMessage, value: &[u8]) {
    message.add(legacy_c_string_prefix(value));
    message.add_byte(0);
}

fn add_battle_fairy_upgrade_log_goods(
    message: &mut CMessage,
    goods: Option<&crate::gameserver::appserver::player::BattleFairyUpgradeGoodsSnapshot>,
) {
    if let Some(goods) = goods {
        message.base_mut().add_guid(goods.identity.ex_id);
        add_legacy_c_string(message.base_mut(), &goods.name);
    } else {
        message.base_mut().add_guid(CGuid::GUID_INVALID);
        add_legacy_c_string(message.base_mut(), b"");
    }
}

fn add_equipment_upgrade_log_goods(
    message: &mut CMessage,
    goods: Option<&EquipmentUpgradeGoodsSnapshot>,
) {
    if let Some(goods) = goods {
        message.base_mut().add_guid(goods.identity.ex_id);
        add_legacy_c_string(message.base_mut(), &goods.name);
    } else {
        message.base_mut().add_guid(CGuid::GUID_INVALID);
        add_legacy_c_string(message.base_mut(), b"");
    }
}

fn gods_battle_property_message(player_id: i32, property: &[u8], value: i32) -> CMessage {
    let mut message = CMessage::new(0xbf80c);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    add_legacy_c_string(message.base_mut(), property);
    message.add_long(value);
    message
}

pub(crate) fn colored_player_notice_message(
    first_color: u32,
    second_color: u32,
    text: &[u8],
) -> CMessage {
    nation_colored_text_message(0xbf806, first_color, second_color, text)
}

pub(crate) fn colored_text_message(
    message_type: i32,
    first_color: u32,
    second_color: u32,
    text: &[u8],
) -> CMessage {
    nation_colored_text_message(message_type, first_color, second_color, text)
}

fn round_fairy_value(value: f32) -> u32 {
    value.round() as u32
}

fn send_hotkey_response(
    game: &CGame,
    player_id: i32,
    message_type: i32,
    marker: u8,
    payload: Option<(u8, Option<u32>)>,
) -> i32 {
    let mut message = CMessage::new(message_type);
    message.add_byte(marker);
    if let Some((slot, value)) = payload {
        message.add_byte(slot);
        if let Some(value) = value {
            message.add_ulong(value);
        }
    }
    message.send_to_player(game.net_server(), player_id)
}

fn send_fairy_long(game: &CGame, player_id: i32, message_type: u32, value: u32) -> i32 {
    let mut message = CMessage::new(message_type as i32);
    message.add_ulong(value);
    message.send_to_player(game.net_server(), player_id)
}

fn send_fairy_goods_update(game: &CGame, update: &FairyContainerGoodsUpdate) -> i32 {
    let mut message = CMessage::new(update.message_type as i32);
    message.add_long(update.player_id);
    message.base_mut().add_guid(update.goods.ex_id);
    message.add_ulong(update.old_client_payload.len() as u32);
    message.base_mut().add(&update.old_client_payload);
    message.send_to_player(game.net_server(), update.player_id)
}

fn deliver_fairy_state_change(
    outcome: &FairyStateChangeOutcome,
    game: &CGame,
    deliveries: &mut Vec<Vec<i32>>,
) {
    let effects = match outcome {
        FairyStateChangeOutcome::ReplacementRejected { effects, .. }
        | FairyStateChangeOutcome::Changed { effects, .. } => effects,
        _ => return,
    };
    for effect in effects {
        deliveries.push(game.send_fairy_state_effect(effect));
    }
}

fn deliver_fairy_syncretize_effects(
    report: &FairySyncretizeReport,
    game: &CGame,
    state_deliveries: &mut Vec<Vec<i32>>,
    amount_deliveries: &mut Vec<Vec<i32>>,
) {
    if let Some(state_change) = &report.state_change {
        deliver_fairy_state_change(state_change, game, state_deliveries);
    }
    if let Some(FairySyncretizeRemoval::Removed { effects, .. }) = &report.secondary_removal {
        for effect in effects {
            state_deliveries.push(game.send_fairy_state_effect(effect));
        }
    }
    match &report.fragment_effect {
        Some(FairySyncretizeFragmentEffect::AmountChanged(change)) => {
            amount_deliveries.push(game.send_fairy_amount_change(change));
        }
        Some(FairySyncretizeFragmentEffect::Removed { effects, .. }) => {
            for effect in effects {
                state_deliveries.push(game.send_fairy_state_effect(effect));
            }
        }
        Some(FairySyncretizeFragmentEffect::RemovalFailed(_)) | None => {}
    }
}

fn send_synthesis_result(game: &CGame, player_id: i32, result: u8) -> i32 {
    let mut message = CMessage::new(0x0b_f925);
    message.base_mut().add_byte(result);
    message.send_to_player(game.net_server(), player_id)
}

fn replace_all_bytes(value: &mut Vec<u8>, pattern: &[u8], replacement: &[u8]) {
    if pattern.is_empty() {
        return;
    }
    let mut cursor = 0;
    while let Some(offset) = value[cursor..]
        .windows(pattern.len())
        .position(|candidate| candidate == pattern)
    {
        let start = cursor + offset;
        value.splice(start..start + pattern.len(), replacement.iter().copied());
        cursor = start + replacement.len();
    }
}

fn synthesis_broadcast_message(
    game: &CGame,
    player_id: i32,
    recipe: &crate::setup::synthesis::SynthesisRecipe,
    amount: u32,
) -> Option<CMessage> {
    if recipe.broadcast_tag == 0 {
        return None;
    }
    let broadcast_id = game.synthesis.broadcasts().get(&recipe.broadcast_tag)?;
    let mut text = game.get_string_by_id(broadcast_id).to_vec();
    if text.is_empty() {
        return None;
    }
    let player_name = game
        .find_player(player_id)?
        .shape()
        .base_object()
        .get_name();
    replace_all_bytes(&mut text, game.get_string_by_id(b"GS1018"), player_name);
    replace_all_bytes(&mut text, game.get_string_by_id(b"GS1019"), &recipe.key);
    replace_all_bytes(
        &mut text,
        game.get_string_by_id(b"GS1020"),
        amount.to_string().as_bytes(),
    );
    let mut message = CMessage::new(0x05_ff0e);
    add_legacy_c_string(message.base_mut(), player_name);
    add_legacy_c_string(message.base_mut(), &text);
    message.add_ulong(0xffff_ffff);
    message.add_ulong(0xffa4_40ff);
    Some(message)
}

/// `TellClient(skill, true)` и отдельный `ResetSkill::SendSkillLearned`
/// используют один opcode, но первый добавляет delay к restore и масштабирует
/// MP только для float-skill, а второй передаёт чистый restore и масштабирует
/// стоимость безусловно.
fn is_battle_fairy_script_attribute(attribute: i32) -> bool {
    (GAP_BF_LEVEL..=GAP_BF_MODULE).contains(&attribute)
        || (GAP_BF_SKY..=GAP_BF_MAX_MP).contains(&attribute)
        || matches!(
            attribute,
            GAP_BF_HUOXIESHU_SKILL | GAP_BF_LINGZHISHU_SKILL | GAP_BF_DEFUALT_SKLL
        )
}

fn clamp_battle_fairy_script_value(value: i64) -> i32 {
    let low = value as i32;
    if low <= 0 { 0 } else { low.min(2_000_000_000) }
}

pub(crate) fn player_skill_learned_message(
    message_type: u32,
    skill_id: u32,
    skill_level: i32,
    wire_skill_level: i32,
    skill_name: &[u8],
    factory: &CSkillFactory,
    player_tell_client: bool,
) -> Option<CMessage> {
    const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
    const SKILL_USAGE_TARGET_MAX_DISTANT: u32 = 5003;
    const SKILL_USAGE_TARGET_MIN_DISTANT: u32 = 5004;
    const SKILL_USAGE_DELAY_TIME: u32 = 10001;
    const SKILL_USAGE_REUSE_SKILL_DELAY_TIME: u32 = 10005;

    let properties = factory.query_skill_base_properties(skill_id, skill_level)?;
    let range = |usage| {
        let value = properties.query_property(usage) as i32;
        if value > 0 { value } else { 1 }
    };
    let restore_time = properties.query_property(SKILL_USAGE_REUSE_SKILL_DELAY_TIME);
    let delay_time = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let raw_cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let scale_cost = !player_tell_client || CSkillFactory::is_need_float(skill_id);
    let cost = if scale_cost {
        (f64::from(raw_cost) * 0.0001_f64).round() as i32
    } else {
        raw_cost as i32
    };

    let mut message = CMessage::new(message_type as i32);
    add_legacy_c_string(message.base_mut(), skill_name);
    message.base_mut().add_short(wire_skill_level as i16);
    message.add_ulong(if player_tell_client {
        restore_time.wrapping_add(delay_time)
    } else {
        restore_time
    });
    message
        .base_mut()
        .add_short(range(SKILL_USAGE_TARGET_MIN_DISTANT) as i16);
    message
        .base_mut()
        .add_short(range(SKILL_USAGE_TARGET_MAX_DISTANT) as i16);
    message.base_mut().add_short(cost as i16);
    Some(message)
}

fn gods_battle_team_szl_share(total: u32, teammate_amount: u32) -> u32 {
    debug_assert_ne!(teammate_amount, 0);
    let multiplier = f64::from(teammate_amount - 1) * 0.05 + 1.0;
    (f64::from(total) * multiplier / f64::from(teammate_amount)) as u32
}

fn legacy_atoi_i32(value: &[u8]) -> i32 {
    let mut value = legacy_c_string_prefix(value);
    while value.first().is_some_and(u8::is_ascii_whitespace) {
        value = &value[1..];
    }
    let (negative, value) = match value.first() {
        Some(b'-') => (true, &value[1..]),
        Some(b'+') => (false, &value[1..]),
        _ => (false, value),
    };
    let mut parsed = 0_i32;
    let mut found = false;
    for byte in value {
        if !byte.is_ascii_digit() {
            break;
        }
        found = true;
        parsed = parsed.wrapping_mul(10).wrapping_add(i32::from(byte - b'0'));
    }
    if !found {
        0
    } else if negative {
        parsed.wrapping_neg()
    } else {
        parsed
    }
}

fn nation_colored_text_message(
    message_type: i32,
    first_color: u32,
    second_color: u32,
    text: &[u8],
) -> CMessage {
    let mut message = CMessage::new(message_type);
    message.add_ulong(first_color);
    message.add_ulong(second_color);
    add_legacy_c_string(message.base_mut(), text);
    message
}

fn nation_world_notice_message(text: &[u8]) -> CMessage {
    let mut message = CMessage::new(0x5fd06);
    message.add_long(0);
    message.add_long(0);
    message.add_ulong(0xffff_ffff);
    message.add_ulong(0xff00_6ee1);
    add_legacy_c_string(message.base_mut(), text);
    message
}

/// Bounded replacement for the reached `__snprintf` `%s` subset. The EXE
/// buffers reserve one byte for NUL (`0x100`/`0x80`), hence visible limits
/// `0xff` and `0x7f` at callers.
pub(crate) fn format_legacy_text_fields(
    template: &[u8],
    arguments: &[&[u8]],
    maximum_bytes: usize,
) -> Vec<u8> {
    let template = legacy_c_string_prefix(template);
    let mut output = Vec::with_capacity(template.len());
    let mut argument_index = 0usize;
    let mut offset = 0usize;
    while offset < template.len() && output.len() < maximum_bytes {
        if template[offset] != b'%' {
            output.push(template[offset]);
            offset += 1;
            continue;
        }
        match template.get(offset + 1).copied() {
            Some(b'%') => {
                output.push(b'%');
                offset += 2;
            }
            Some(b's') => {
                let Some(argument) = arguments.get(argument_index) else {
                    output.extend_from_slice(&template[offset..]);
                    break;
                };
                let remaining = maximum_bytes.saturating_sub(output.len());
                let argument = legacy_c_string_prefix(argument);
                output.extend_from_slice(&argument[..argument.len().min(remaining)]);
                argument_index += 1;
                offset += 2;
            }
            _ => {
                output.push(b'%');
                offset += 1;
            }
        }
    }
    output.truncate(maximum_bytes);
    output
}

enum LegacyFormatArgument<'a> {
    Bytes(&'a [u8]),
    Signed(i32),
}

/// Bounded replacement for the reached SZLGS `%s`/`%d` templates. It keeps
/// `%%` and leaves an unmatched conversion literal instead of reading a
/// missing vararg beyond the proven call contract.
fn format_legacy_mixed(
    template: &[u8],
    arguments: &[LegacyFormatArgument<'_>],
    maximum_bytes: usize,
) -> Vec<u8> {
    let template = legacy_c_string_prefix(template);
    let mut output = Vec::with_capacity(template.len());
    let mut argument_index = 0usize;
    let mut offset = 0usize;
    while offset < template.len() && output.len() < maximum_bytes {
        if template[offset] != b'%' {
            output.push(template[offset]);
            offset += 1;
            continue;
        }
        let conversion = template.get(offset + 1).copied();
        if conversion == Some(b'%') {
            output.push(b'%');
            offset += 2;
            continue;
        }
        let Some(argument) = arguments.get(argument_index) else {
            output.push(b'%');
            offset += 1;
            continue;
        };
        let rendered = match (conversion, argument) {
            (Some(b's'), LegacyFormatArgument::Bytes(value)) => {
                legacy_c_string_prefix(value).to_vec()
            }
            (Some(b'd' | b'i'), LegacyFormatArgument::Signed(value)) => {
                value.to_string().into_bytes()
            }
            _ => {
                output.push(b'%');
                offset += 1;
                continue;
            }
        };
        let remaining = maximum_bytes.saturating_sub(output.len());
        output.extend_from_slice(&rendered[..rendered.len().min(remaining)]);
        argument_index += 1;
        offset += 2;
    }
    output.truncate(maximum_bytes);
    output
}

fn game_kick_around_block(
    outcome: GameKickAroundOutcome,
    target_player_id: Option<i32>,
    server_region_id: Option<i32>,
) -> GameKickAroundReport {
    GameKickAroundReport {
        outcome,
        target_player_id,
        server_region_id,
        window_origin: None,
        matched_player_ids: Vec::new(),
        kicks: Vec::new(),
    }
}

fn gm_kick_window_origin(tile: i32, extent: i32) -> i32 {
    if tile.wrapping_add(3) >= extent {
        extent.wrapping_sub(7)
    } else if tile.wrapping_sub(3) <= 0 {
        0
    } else {
        tile.wrapping_sub(3)
    }
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}

fn format_single_legacy_i32(template: &[u8], value: i32, maximum_bytes: usize) -> Vec<u8> {
    let template = legacy_c_string_prefix(template);
    let Some(marker) = template.windows(2).position(|window| window == b"%d") else {
        return template[..template.len().min(maximum_bytes)].to_vec();
    };
    let value = value.to_string();
    let mut result = Vec::with_capacity(template.len().saturating_add(value.len()));
    result.extend_from_slice(&template[..marker]);
    result.extend_from_slice(value.as_bytes());
    result.extend_from_slice(&template[marker + 2..]);
    result.truncate(maximum_bytes);
    result
}

fn format_two_legacy_i32(
    template: &[u8],
    first: i32,
    second: i32,
    maximum_bytes: usize,
) -> Vec<u8> {
    let template = legacy_c_string_prefix(template);
    let mut output = Vec::with_capacity(template.len().saturating_add(20));
    let mut remaining = template;
    for value in [first, second] {
        let Some(marker) = remaining.windows(2).position(|window| window == b"%d") else {
            output.extend_from_slice(remaining);
            output.truncate(maximum_bytes);
            return output;
        };
        output.extend_from_slice(&remaining[..marker]);
        output.extend_from_slice(value.to_string().as_bytes());
        remaining = &remaining[marker + 2..];
    }
    output.extend_from_slice(remaining);
    output.truncate(maximum_bytes);
    output
}

fn format_single_legacy_u32(template: &[u8], value: u32, maximum_bytes: usize) -> Vec<u8> {
    let template = legacy_c_string_prefix(template);
    let marker = template
        .windows(2)
        .position(|window| window == b"%u" || window == b"%d");
    let Some(marker) = marker else {
        return template[..template.len().min(maximum_bytes)].to_vec();
    };
    let formatted = if template[marker + 1] == b'u' {
        value.to_string()
    } else {
        (value as i32).to_string()
    };
    let mut output = Vec::with_capacity(template.len().saturating_add(formatted.len()));
    output.extend_from_slice(&template[..marker]);
    output.extend_from_slice(formatted.as_bytes());
    output.extend_from_slice(&template[marker + 2..]);
    output.truncate(maximum_bytes);
    output
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

impl ShapeResolver for CGame {
    fn resolve_shape(&self, identity: ShapeIdentity) -> Option<ShapeView> {
        match identity.object_type {
            PLAYER_TYPE => {
                let player = self.find_player(identity.id)?;
                let view = player.shape_view()?;
                (view.identity.object_type == identity.object_type
                    && view.identity.id == identity.id)
                    .then_some(view)
            }
            MONSTER_TYPE => {
                let monster = self
                    .regions
                    .values()
                    .find_map(|region| region.base().find_monster_by_id(identity.id))?;
                let property =
                    self.find_monster_property_by_origin_name(monster.base_property_key()?)?;
                shape_view(monster.move_shape().shape(), CMonster::figure(property))
            }
            NPC_TYPE => {
                let npc = self
                    .regions
                    .values()
                    .find_map(|region| region.base().find_npc_by_id(identity.id))?;
                shape_view(npc.move_shape().shape(), ShapeFigure::default())
            }
            _ => None,
        }
    }
}

fn shape_view(
    shape: &crate::gameserver::appserver::shape::CShape,
    figure: ShapeFigure,
) -> Option<ShapeView> {
    Some(ShapeView {
        identity: shape.identity(),
        tile_x: shape.get_tile_x().ok()?,
        tile_y: shape.get_tile_y().ok()?,
        pos_x_bits: shape.get_pos_x().to_bits(),
        pos_y_bits: shape.get_pos_y().to_bits(),
        figure,
    })
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.h

// ============================================================================
// FUNCTION: Catch@00401323
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// RVA: 0x00001323
// ADDRESS: 00401323
// PROTOTYPE: undefined Catch@00401323()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0040134e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// RVA: 0x0000134E
// ADDRESS: 0040134e
// PROTOTYPE: undefined FUN_0040134e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004013bd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// RVA: 0x000013BD
// ADDRESS: 004013bd
// PROTOTYPE: undefined Catch@004013bd()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED, VERIFIED_DISASSEMBLY: `InitNetServer` материализован выше;
// exact эпилог возвращает `0` после Host-error и `1` после setup-записей.

// IMPLEMENTED, VERIFIED_DISASSEMBLY: `CGame::KickPlayer` RVA `0x000022F0`
// материализован выше с exact `QuitClientByMapID` side effect и постоянным
// `false` return; покрытый raw удалён.

// `SetFunctionFileData`, `SetVariableFileData` и `SetGeneralVariableFileData`
// материализованы выше с подтверждённой семантикой владения и повторной публикации.

// IMPLEMENTED: `SetAuctionState` RVA `0x00002390` материализован выше
// и связан с exact World `0x80403` caller-ом; покрытый raw удалён.

// IMPLEMENTED: `InitNetClientOfWS/BS` материализованы выше с исходными
// registration packets и master -> backup Billing порядком.

// `CreateStringTable` материализован выше с clear/decode/log/cursor порядком.

// IMPLEMENTED, VERIFIED_DISASSEMBLY: `FindPlayer(char const*)` RVA `0x00003A60`
// материализован выше как ordered byte-name lookup; покрытый raw удалён.
// IMPLEMENTED, VERIFIED_DISASSEMBLY: `FindPlayerByAccount(char*)` materialized
// above as ordered byte-exact account lookup; covered raw removed.
// ============================================================================
// FUNCTION: CGame::SendTopInfoToClient
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1105
// RVA: 0x00003BE0
// ADDRESS: 00403be0
// PROTOTYPE: void __thiscall SendTopInfoToClient(long param_1, long param_2, long param_3, long param_4, basic_string<char,std::char_traits<char>,std::allocator<char>_> param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CGame::AI` signed region-map order и virtual region AI с
// base-tail `ClearPlayerAI` материализованы выше.

// ============================================================================
// FUNCTION: CGame::SaveCityRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1085
// RVA: 0x000051B0
// ADDRESS: 004051b0
// PROTOTYPE: void __thiscall SaveCityRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `ProcessMessage` материализован выше; static scratch deque
// заменён тремя последовательными owned snapshot-очередями.

// ============================================================================
// FUNCTION: CGame::tagSetup::tagSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.h:166
// RVA: 0x00008810
// ADDRESS: 00408810
// PROTOTYPE: void __thiscall tagSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::RemoveSequenceMap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1221
// RVA: 0x00008970
// ADDRESS: 00408970
// PROTOTYPE: void __thiscall RemoveSequenceMap(uint param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::CleanSequenceMap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1237
// RVA: 0x000089D0
// ADDRESS: 004089d0
// PROTOTYPE: void __thiscall CleanSequenceMap(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::LoadSetupEx
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:189
// RVA: 0x00009160
// ADDRESS: 00409160
// PROTOTYPE: bool __thiscall LoadSetupEx(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::ReloadSetupEx
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:207
// RVA: 0x00009310
// ADDRESS: 00409310
// PROTOTYPE: bool __thiscall ReloadSetupEx(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::AppendSequenceMap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1203
// RVA: 0x00009340
// ADDRESS: 00409340
// PROTOTYPE: bool __thiscall AppendSequenceMap(uint param_1, CSequenceString * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::AppendValidateTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1280
// RVA: 0x000093D0
// ADDRESS: 004093d0
// PROTOTYPE: void __thiscall AppendValidateTime(uint param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::LoadSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:116
// RVA: 0x00009960
// ADDRESS: 00409960
// PROTOTYPE: bool __thiscall LoadSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::ReLoadSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:178
// RVA: 0x00009DD0
// ADDRESS: 00409dd0
// PROTOTYPE: bool __thiscall ReLoadSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::Init
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:217
// RVA: 0x00009E70
// ADDRESS: 00409e70
// PROTOTYPE: int __thiscall Init(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: достигнутый `Release` teardown материализован выше. Широкий
// RAW-блок и split funclets ниже сохранены как доказательство ещё не
// материализованных player serializer-а, SaveCityRegion, validate-time map и
// exception-specific debug paths; он не считается полностью заменённым.

// ============================================================================
// FUNCTION: CGame::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:293
// RVA: 0x00009FD0
// ADDRESS: 00409fd0
// PROTOTYPE: int __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0040a090
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:326
// RVA: 0x0000A090
// ADDRESS: 0040a090
// PROTOTYPE: undefined FUN_0040a090()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040a15b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:341
// RVA: 0x0000A15B
// ADDRESS: 0040a15b
// PROTOTYPE: undefined Catch@0040a15b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0040a2a0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:379
// RVA: 0x0000A2A0
// ADDRESS: 0040a2a0
// PROTOTYPE: undefined FUN_0040a2a0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040a2d3
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:384
// RVA: 0x0000A2D3
// ADDRESS: 0040a2d3
// PROTOTYPE: undefined Catch@0040a2d3()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0040a370
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:394
// RVA: 0x0000A370
// ADDRESS: 0040a370
// PROTOTYPE: undefined FUN_0040a370()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040a3aa
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:399
// RVA: 0x0000A3AA
// ADDRESS: 0040a3aa
// PROTOTYPE: undefined Catch@0040a3aa()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040a48a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:415
// RVA: 0x0000A48A
// ADDRESS: 0040a48a
// PROTOTYPE: undefined Catch@0040a48a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::RemoveValidateTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:1293
// RVA: 0x0000AA40
// ADDRESS: 0040aa40
// PROTOTYPE: void __thiscall RemoveValidateTime(uint param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::~CGame
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:110
// RVA: 0x0000AF50
// ADDRESS: 0040af50
// PROTOTYPE: void __thiscall ~CGame(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::CGame
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:83
// RVA: 0x0000B260
// ADDRESS: 0040b260
// PROTOTYPE: void __thiscall CGame(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// `SetScriptFileData` материализован выше: ключ ограничен первой NUL,
// повторная публикация заменяет прежний owned buffer.

// ============================================================================
// FUNCTION: GetGame
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp:76
// RVA: 0x0000B750
// ADDRESS: 0040b750
// PROTOTYPE: CGame * __cdecl GetGame(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: обе `ReConnect*` функции и retry thread-entry материализованы
// выше; C++ exception funclets заменены typed outcomes Rust.

// IMPLEMENTED: `RunAuction` RVA `0x0000BC30` материализован выше и вызывается
// напрямую из `MainLoop`; покрытый raw удалён.

// IMPLEMENTED: `CreateConnectWorldThread` RVA `0x0000BDA0` и
// `CreateConnectBillingThread` RVA `0x0000BE10` материализованы выше как owned
// Tokio tasks; начальный stop/join обеих задач из `Release` также перенесён.

// IMPLEMENTED: `GetTeamSessionID` и `FindPlayer` материализованы выше;
// покрытые raw-блоки удалены.

// `GetStringByID` материализован выше с исходным empty-string fallback.

// `GetScriptFileData` материализован выше как lookup без вставки отсутствующего ключа.

// ============================================================================
// FUNCTION: $L82323
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// RVA: 0x0022A3A0
// ADDRESS: 0062a3a0
// PROTOTYPE: undefined $L82323()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L83447
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// RVA: 0x0022A3C0
// ADDRESS: 0062a3c0
// PROTOTYPE: undefined $L83447()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L83127
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// RVA: 0x0022A3E0
// ADDRESS: 0062a3e0
// PROTOTYPE: undefined $L83127()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $E2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// RVA: 0x0024A5F0
// ADDRESS: 0064a5f0
// PROTOTYPE: void __cdecl $E2(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $E5
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\game.cpp
// RVA: 0x0024A600
// ADDRESS: 0064a600
// PROTOTYPE: void __cdecl $E5(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
