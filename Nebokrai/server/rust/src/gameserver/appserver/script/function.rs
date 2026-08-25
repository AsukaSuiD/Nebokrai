//! Диспетчер сценарных функций исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец —
//! `server/gameserver/appserver/script/function.cpp`. Из плотного диспетчера
//! семейство фракционных меню `3012/6001/6002/6003/6011/6015/6017/6030`
//! проходит от вычисленных аргументов и проверки расстояния между игроком и
//! NPC в принадлежащее `CGame` состояние сеанса; создание и заявка завершаются
//! только через действующий диспетчер ответов OrganSys от клиента и World.
//! Отмена заявки сохраняет ту же проверку расстояния и World `0x60109`.
//! Функция `6011` добавляет точный GameSave игрока в `0x60126`; авторитетное
//! списание World `0x7FE1E` возвращается через тот же диспетчер OrganSys.
//! Объявление войны `6030` держит операторский сеанс `2000` мс между страницей
//! World, выбором клиента и авторитетным результатом со списанием.
//! Городские ворота `6004/6019/6020` используют только локальный
//! `CServerCityRegion`: проверки состояния и геометрии предшествуют прямому
//! изменению либо авторизации World `0x6012F`; каждое изменение публикует
//! обновление постройки.
//! Запрос главы `6017` читает идентичность, обновляемую World `0x7FE06`, а не
//! отдельную сценарную копию или постоянную заглушку `false`.
//! Налоговые функции `6005/6006/6049/6050/6052/6053` используют текущий
//! регион сценария. Меню проходит авторизацию World `0x6012B/0x6012C`, затем
//! реальный `CNetSession`, клиентские ответы `0x90122/0x90123`, изменение
//! кошелька или ставки и обратный снимок `0x6012D`.
//! `6043 / EnterContendState` сохраняет разговорный distance gate, вычисляет
//! два числа и четыре строки до faction/region checks и входит в concrete
//! City/Village `CServerWarRegion`; общий owner выполняет membership/goods,
//! contender replacement, player `0xBFF28/29`, `GS0229/43..46` и сохраняет
//! village goods для последующего victory cleanup. Faction name/union для
//! contender snapshot берутся из того же авторитетного World `0x7FE06`.
//! Достигнутые GodsBattle scripts связывают `5413 / GetAreaID` с настоящим
//! login-server ID, а `11124/11128` — с persisted player SZL и уже существующим
//! `CGame::UpdateSZL` effect-проходом: property/notice, merit-level downgrade
//! и appellation callback выполняются одним owner-ом.
//! `2570 / AddIncrementLog` проводит item-script audit в существующий World
//! `0x6020D` owner: limits, defaults, byte-narrowed type и conditional
//! item-tail сохраняются до DB FIFO/live publication; tail вычисляется только
//! для полного `type == 0`, как в исходном `CScript`.
//! Territory-war menu `6040..6047/6054..6056` сохраняет ранние
//! player/NPC/region gates, local-only time query и local-before-proxy
//! ownership/state/country lookups. Заявка проверяет live faction/union,
//! business и конкурирующие war schedules, публикует `GS0201..GS0207`, затем
//! проходит `0x60135` до авторитетного World accept; ответ `0x7FE34` возвращён
//! в общий OrganSys dispatcher и меняет canonical player wallet/client wire.
//! Городская заявка в той же цепочке добавляет master/country/owner/state
//! gates, `GS0209..GS0211`, wire `0x60137` и симметричный ответ `0x7FE37`.
//! Также материализованы ID `9351 / ReflushExternProperty`, `9350 / OpenRolePage`,
//! `9354 / OpenEquipmentCompose`, `2216 / OpenGoodsUpgrade` и
//! `8100 / OpenChangePlayerNameUI`. Rename-вход без аргументов публикует
//! `0xBF810` из canonical player/GlobeSetup и тем самым открывает уже живой
//! `0x8FB05 → World/DB → 0x7FA0E → 0xBF80F` контур. Refresh вычисляет первую
//! строка, DaKong gate предшествует lookup выбранного enhancement goods, а
//! gameplay передаётся canonical `CGame`, который сам исполняет localized
//! notices, session/plug lifecycle и client wire; runtime сообщает только
//! ещё не owned team skill-state. Country scalar family `9001/9003/9009/9011/
//! 9013` сохраняет byte country lookup, field-specific clamp, local mutation и
//! World `0x60314`. Quest-switch `9018/9019` сохраняет read-only lookup и
//! странность writer-а: второй аргумент влияет только на country fallback, а
//! применяемое значение всегда `true`; `0x60315` уходит до local map write.
//! Exile-time `9021` остаётся полностью локальным: страна script-player, один
//! runtime clock sample и wrapping `CCountry` calculation. `9317 / AddKingPoint`
//! складывает delta как DWORD, меняет local control point и публикует selector
//! `5`, сохраняя нулевой script result. Government identity `9004/9005/9006`
//! читает mutating CI, отправляет назначение `0x60304` без преждевременной
//! local mutation и читает отдельный краткоживущий king-ID response state.
//! Country scalar query family `9000/9002/9008/9010/9012` одним контрактом
//! сужает explicit country до byte либо использует страну script-player и
//! возвращает `-1` при недоступном owner-е.
//! Aliases `2633/9020` разрешают local target и проходят canonical
//! `CGame::player_country_identity`, который mutating-читает ordered CI `1..8`.
//! Country-war declaration `9100` сохраняет NPC distance gate, фазу объявления,
//! king-CI, обе duplicate-проверки, idle-region gate, `GS0213..GS0218` и
//! terminal World request `0x60317 [player, target country]`.
//! Соседняя read-only family `9101..9104/9106..9109/9111` одним dispatcher-ом
//! читает phase flags, declaration membership, country-region win symbols,
//! ordered war region/camp/opponent и сохранённый country result.
//! Action family `9105/9110` сохраняет war/distance/argument ordering, вызывает
//! concrete `ServerCountryRegion::OnEnterContend` с миллисекундным duration,
//! публикует player state `0xBFF28`, timer `0xBFF29`, localized result и
//! отправляет byte-narrowed победу World сообщением `0x60318`.
//! Nation-war family `9304..9313/9315/9316` связывает current Nation region,
//! timing/carriage/contender player-effects, FourNation seconds/time/morale,
//! weak-state и World signup `0x6031B`; debug `GS1053..GS1056` остаётся в
//! точном порядке вокруг соответствующих concrete вызовов.
//! Battle-fairy family `9400..9411` сохраняет mixed string/integer parameter
//! routing, name/current-player lookup, skill/equipment mutation, reset RNG,
//! revive, experience/recreate RNG, old-client `0xBF918`, properties wire и
//! локальные журналы `BattleFairy`.
//! `2249 / FairyExpUp` разрешает enhancement-shadow обратно в live packet или
//! equipment goods, сохраняет grow-log, replacement ownership и concrete
//! delete/new-object wire, включая необратимый late packet-add failure.
//! Предметная family `2231..2236/2243..2246` получает GUID запускающего goods
//! из того же CScript instance, ограничивает used-item lookup настоящей
//! сумкой, изменяет addon/durability storage и публикует delete/amount/update
//! wire; selected durability разрешается через live enhancement-shadow.
//! Login-script family `2650/2651` разрешает persisted LeiTing `tagThing` из
//! того же player owner-а. Setter сохраняет только положительное увеличение,
//! energy/daily-stamp mutation и общий `0xBF73E + 0x5FD10` snapshot; getter
//! возвращает count либо `-1`, а исходный setter result остаётся `-1`.
//! Достигнутая там же notice-family `3316/5201/5202` единообразно вычисляет
//! text/color/background: личный круг использует `0xBF811` и opaque-black
//! default, региональный — `0xBF806`, мировой сохраняет World `0x5FF0E` relay.
//! Quest family `6200/6201/6202/6203/6207` сохраняет ushort narrowing,
//! computed target-player selector, persisted completion/removal, transient
//! position wire и межсерверный add/remove round-trip для удалённого target-а.
//! Соседний `3500..3503/3507` owner меняет persisted quest-enabled/countdown
//! поля, публикует `0xBF728..2A` и возвращает тот же signed-wrap remainder,
//! который client может отдельно запросить через уже достигнутый `0xBF72B`.
//! Семейство повозки `3504..3506/3508` через тот же достигнутый диспетчер
//! передаёт вычисленные имена каноническим владельцам создания и привязки к
//! игроку, публикации `C0205/BF504`, журналирования в World, живых запросов
//! расстояния и индекса, а также уже восстановленного жизненного цикла AI и
//! сохранения.
//! `GetCopyNumForShengSiShiSu 9314` передаёт вычисленный признак резервирования,
//! игрока и ID сценария в World-запрос `0x5FD0B`; ответ `0x7FA15` возвращается
//! через общего владельца продолжения, а десятисекундный тайм-аут записывает ноль
//! в `$m_TalkRet` и освобождает сохранённую позицию сценария.
//! `ListBanedPlayer 5106` тем же способом передаёт ID игрока и сценария по
//! цепочке GameServer → WorldServer → LoginServer, получает до 256 действующих
//! блокировок, продолжает только ожидающую функцию `5106` и публикует строки
//! клиенту; отказ и десятисекундный тайм-аут возвращают `-1`.
//! Связка `OpenCiQingPage 9628` и `PushItemToCiQing 9629` использует
//! канонические состояние игрока, фабрику предметов и кодек старого клиента:
//! открытие отправляет `0xC0112`, а подтверждённый предмет с дополнением `243`
//! сохраняется в списке и публикует весь упорядоченный набор через `0xC010C`.
//! PreciousBox `2221/2222/2237` сохраняет доверенный сценарий действия у
//! игрока, клиентский обмен открытия, результата и закрытия, общий RNG игры,
//! бросок конфигурации, владение фабрикой, улучшением и пакетом предметов, а
//! также необязательное объявление World; повторный запуск приходит из живого
//! кода предметов `0x8FC12`.
//! Перенос предметов `2200..2203` сохраняет необязательную подготовку
//! улучшения и особого свойства, приоритет рюкзака в `DelGoods`, переход к
//! экипировке с полным завершением игрока и обычные клиентские сообщения
//! добавления, удаления и изменения количества; достигнутый вызов находится
//! в первом блоке очистки реально запускаемого `scripts/quest/nodupe.script`.
//! `2306 / GameMessage` публикует `0xBF808` с текстом, видом окна и
//! идентификатором сценария, после чего тот же экземпляр ожидает клиентский
//! ответ `0x8FB02` и получает `$m_TalkRet`. `2309 / Help` использует малое
//! окно `0xBF71C` и тот же жизненный цикл ожидания, что `TalkBoxSmall`.
//! Журналы `2314/2315` не вычисляют аргументы при выключенных настройках;
//! включённые ветви создают временные предметы через общую фабрику и отправляют
//! полные записи `0x60204/0x60205` владельцу журнала WorldServer.
//! `2319 / OpenNewHelpWindow` открывает клиентское окно пакетом `0xBF812` без
//! аргументов и ответного состояния. Соседний ID `2318` в целевом EXE не
//! поддерживается и намеренно не получает выдуманного обработчика.
//! `2321 / GetGoodsProperty` разрешает исходное имя через `CGoodsFactory`,
//! создаёт временный предмет тем же игровым генератором и читает указанную
//! пару дополнения без изменения контейнеров игрока.
//! Диапазон `2501..2504` связывает запрос смены страны с уже действующим
//! ответом WorldServer, а вклад и YuanBao читает из канонического `CPlayer`.
//! Установка вклада сохраняет ограничение `SetContribute`, пакет `0xBF724`
//! вместе с силой подбора и возвращает фактически сохранённое значение.
//! `3312 / AttackPlayer` разрешает имя только среди игроков девяти соседних
//! областей сценарного игрока и назначает найденную цель искусственному
//! интеллекту монстров из того же пространственного окна.
//! `3314 / MovePlayer` фиксирует игроков исходного прямоугольника в порядке
//! областей региона, выбирает каждому случайную точку целевого прямоугольника
//! и проводит через общий `CPlayer::ChangeRegion` со всеми его сообщениями.
//! Селекторы `3310 / OpenPlayerUI` и `3311 / CallMonster` в целевом EXE не
//! читают аргументы и не создают побочных эффектов, но различаются точными
//! возвращаемыми значениями `0` и `1`.
//! Соседняя группа `2204/2205/2212/2217/2218/2220` связывает подсчёты рюкзака
//! и депо, выбранный предмет контейнера улучшения, локальное либо удалённое
//! удаление и доверенный путь сценария окна `0xBF919` с подтверждением
//! `0x8FC11`.
//! Группа `2209/2223..2230` продолжает тот же живой выбранный предмет: читает
//! имя, цену и значения свойств, меняет модификаторы, улучшает, полностью
//! удаляет либо пересоздаёт случайные дополнения с сохранением уровня,
//! долговечности, камней и их итоговых эффектов. Изменения доходят до
//! клиентского `0xBF918`, а удаление экипировки проходит общие свойства игрока.
//! Функции `2240..2242` получают контекст из живого хвоста смерти монстра:
//! индивидуальный и общий сценарии несут один базовый индекс, а диспетчер
//! возвращает его знаковое представление либо находит исходное имя и уровень в
//! упорядоченном реестре `CMonsterList`.
//! Следующий блок проверки `2002/2316/2317/2500` читает живые регион и страну
//! игрока и работает с тем же принадлежащим серверу реестром сценариев;
//! удаление текущего сценария завершается на границе команды без прежнего
//! обращения к освобождённой памяти.
//! `5108 / GetOnlinePlayers` замыкает уже существующий контракт World
//! `0x5FF01 → 0x7FC01`: планировщик ждёт удалённое число игроков и повторяет
//! исходное выражение без повторной отправки запроса.
//! Семейство `5103–5105/5109` из того же диспетчера публикует локальные
//! списки GM и игроков, запрашивает межсерверные части списка GM и молчания,
//! а также запускает `0x5FF13 → 0x7F803 → 0x5FA03`, поэтому сохранение всех
//! игроков доходит до полного снимка GameSave и владельца сохранения World.
//! Соседние `5301 / KickAll` и `5302 / KickMap` используют тот же живой
//! диспетчер GM: первый рассылает отключение всем GameServer с исключением
//! запросившего игрока, второй сначала обслуживает локальный регион, а при его
//! отсутствии маршрутизирует запрос фактическому владельцу региона.
//! В той же системной группе `5403 / Weather` заменяет погоду текущего региона
//! и рассылает `0xBF507`, а `5405 / PlayAction` публикует 16-битное действие
//! `0xBF508` вокруг игрока без изменения его серверного состояния.
//! Недельный сброс из того же достигнутого сценария читает локальные поля
//! `SYSTEMTIME` через `TagTime`, меняет принадлежащие планировщику общие
//! переменные и публикует `PostWorldInfo` по уже замкнутому контракту
//! `0x5FF0E → 0x7FC0D → 0xBF806/0xBF804`.
//! Соседняя ветвь нормализации бодрости читает и записывает живой
//! `dwVigour`; `SetMe` передаёт прямую запись DWORD в `CGame`, после чего
//! проходит обязательный вызов `UpdateProperty` и адресный `0xBF721` с уже
//! изменённой бодростью.
//! `2304 / ChangeRegion`, достигнутый следующим блоком `nodupe.script`,
//! сохраняет все семь позиционных аргументов и их неодинаковые значения по
//! умолчанию. Вызов проходит канонический `CGame`: хвост деловых и сеансовых
//! состояний, прежний, локальный либо замещающий регион, `BF603/BF601/BF505`,
//! сообщения фракции и команды, полный GameSave `5FA02`, World `7F802` и
//! клиентский `BF506`; локальная ветвь завершается через очередь ИИ
//! `CS_CHANGEREGION` и настоящий входной вызов назначения `8F801`.
//! Следующая календарная ветвь того же сценария достигает `13..19/23` и
//! `5001 / Reload`. Временные селекторы сохраняют точную упакованную раскладку
//! `year:9/month:4/day:5/hour:5/minute:6/weekday:3`; `Year..DayOfWeek`
//! принимают необязательное упакованное значение, а `Second` всегда читает
//! местные часы.
//! `Reload` публикует `0x5FF06 + ID игрока + строка C профиля`; настоящий
//! диспетчер GM World выполняет достигнутых владельцев `JJcConfig` и
//! `IncrementShopList`, а для `IncrementShopList` рассылает обновлённый снимок
//! GameServer. Недостижимое обычным сценарием разыменование пустого игрока
//! безопасно заменено отсутствием действия без сетевой публикации.
//! Часовая ветвь `game_enter.script` добавляет `20/21`, `8003` и `3302` одним
//! проходом того же CScript. Packed local time декодируется через CRT `mktime`;
//! random position пишет player-owned `$m_Temp[0..1]`; все 12 вычисленных
//! аргументов `CreateNpc` доходят до concrete `CServerRegion::AddNpc`, включая
//! spatial/AI publication, show-list, script и lifetime. Для чужого региона
//! тот же owner сохраняет исходный маршрут World `0x5FA0B -> 0x7F80A`.
//! Восстановительная ветвь `nodupe.script` достигает строкового `2998 /
//! GetName`, `3002 / SetPlayerLevel` и skill pair `3102/3103`. String result
//! возвращается непосредственно expression evaluator-у; level/experience и
//! skill mutations проходят live `CPlayer/CMoveShape`, адресные client packets
//! и существующие World relay `5FC01/5FC02/5FC04` для удалённого target-а.
//! `GetMe/SetMe` той же ветви читают level/occupation/experience, а DWORD write
//! сохраняет pre-mutation `BF80C`, `UpdateProperty`, `BF721` и reached
//! `CheckLevel` terminal result.
//! Группа `2004..2008` вызывает того же владельца `CPlayer::CheckLevel` с
//! нулевыми приростами, а текущая и максимальная энергия изменяются у живого
//! игрока с точными ограничениями и ответами `0xBF72C/0xBF72D`.
//! `ChangeMe 2001` использует общий каталог игрока, но отдельно сохраняет
//! масштаб сценарного опыта, нижнюю границу опыта и жизненный цикл знака
//! убийцы; результат доходит до `0xBF80B/0xBF70E` и полного `CheckLevel`.
//! Достигнутый `xinlian/hy_pg.script` расширяет этот owner до `GetMe(lID/
//! btCountry/lPos)`, named family `2999..3001` и `5203 / PostCountryInfo`.
//! `ChangePlayer` замыкает shipped `Experience` alias без отдельного client
//! packet; `SetPlayer` сохраняет narrowing и порядок write → UpdateProperty →
//! `BF80C`, а удалённый `GetPlayer` проходит
//! `5FF02 → 7FC03 → 5FF03 → 7FC02` до continuation исходного CScript. Notice
//! проходит существующий строгий маршрут
//! `0x5FF16 → 0x7FC13 → 0xBF806` только игрокам выбранной страны.
//! Player-identity pair `3003/3004` разделяет global и local lookup: только
//! `IsPlayerOnline` при local miss и доступном World transport приостанавливает
//! CScript по существующему `5FF05 → 7FC05` continuation contract; имя длиннее
//! 255 байт и пустое имя возвращают zero без запроса.
//! Region-routing family `3005..3007` связывает named lookup
//! `5FF04 → 7FC04`, локальный полный `ChangeRegion`, remote broadcast
//! `5FF11 → 7FC10` и X-major 7×7 массовый перенос через тот же spatial owner.
//! Соседний `3008 / KickPlayerEx` переиспользует этот 7×7 scan: локально
//! публикует `BF806(GS0030)`, удалённо проходит
//! `5FF08 → 7FC07 → 5FD02 → 7FA05` до requester-а.
//! Money family `3011..3016` сохраняет локальный lookup по имени/ID, legacy
//! signed arithmetic и вызывает достигнутый `CPlayer::SetMoney` wallet owner:
//! state, create/change/delete container wire и old-client goods stream
//! исполняются из того же `CScript::RunFunction` runtime caller-а.
//! Соседний `3010 / ForceMove` ограничивает имя exact 23 байтами и проводит
//! вычисленные X/Y/time через local player lookup, region spatial membership,
//! around `0xBF708` и AI stand-event достигнутого `CMoveShape` owner-а.
//! Diagnostic `3017 / GetPlayerAllVariables` требует live script-player,
//! снимает insertion-order snapshot локального target `CVariableList` и
//! публикует scalar/string/array строки точным зелёным `BF806` caller-у.
//! Парный `3009 / GetPlayerAllProperties` читает exact base/current wire
//! slots PDB-layout, форматирует `GS0186..GS0188` в исходном vararg-порядке и
//! публикует три цветных `BF806` тому же live script-player.
//! Moderation family `3201..3203` сохраняет local-before-World lookup:
//! named kick завершает `CGame::KickPlayer` и `GS0025`, ban публикует
//! `0x5FF12`, а silence либо меняет exact player timestamp, либо проходит
//! `0x5FF0C -> 0x7FC0B/0x5FF0D -> 0x7FC0C` через общий GM dispatcher.
//! `3309 / GetMapInfo` читает concrete cell текущего script-region и сохраняет
//! приоритет war-marker над safe/fight security с legacy кодами `2/3/1/0`.
//! Region collision selector `8000 / RefeashBlock` вычисляет только первый
//! аргумент, находит canonical region owner и тем же runtime-вызовом очищает
//! старые BLOCK_SHAPE и восстанавливает живые player/monster и все NPC tiles.
//! `3305 / CreateMonster` проводит local spawn через canonical property,
//! random-position, AI/spatial и around owners; чужой region сохраняет
//! существующий `0x5FA0B -> 0x7F80A` межсерверный маршрут.
//! Explicit removal pair `3303/3306` разрешает shape только в current player
//! region, публикует `0xBF504` до mutation и сохраняет различие: NPC сразу
//! покидает spatial owner, monster лишь ставится в `CS_DELETE` для AI cleanup.
//! Batch selectors `3313/3315` переиспользуют тот же terminal owner после
//! rectangle/original-name либо region/name lookup, сохраняя publication-first
//! порядок для каждого найденного shape.
//! Talk pair `3301/3304` формирует exact `0xBF801` actor/name/text wire: NPC
//! использует canonical around-send, monster family сохраняет 3x3 area scan,
//! exact-name match и строгий AREA_WIDTH/HEIGHT recipient filter.
//! Игровой caller `2308 / PlayerTalk` дополняет ту же chat family: проверяет
//! live player до выражения, формирует actor type `400` с canonical именем и
//! отправляет `0xBF801` через фактический father-region spatial membership.
//! `3308 / PlayerMessage` выполняет local named notice с принудительным type
//! zero либо отправляет `0x5FA04`; существующий World/Game callback завершает
//! online delivery или offline feedback исходному script-player.
//! Его terminal `5404 / PlayEffect` проверяет live player/local region до
//! вычисления аргументов, выбирает explicit либо player tile и публикует
//! точный `0xBF50A(effect, x+0.5f, y+0.5f)` через canonical around runtime.
//! Соседний `5410 / PlaySound` сохраняет тот же pre-evaluation caller gate,
//! exact `0xBF509` C-string wire и переключает direct/around delivery только
//! вычисленным вторым аргументом; отсутствующий и ошибочный флаг равны нулю.
//! GM control family `5401/5402/5406/5407` не вычисляет аргументы: legacy
//! invisible остаётся intentional no-op, God/Resident меняют canonical
//! `CMoveShape::m_bIsGod`, а GMMode читает startup `CGMList` и шлёт `0xBFC01`.
//! Reached rank NPC scripts `6051 / RequestPlayerRanks` проверяют player/NPC
//! до выражения, затем передают limit и один clock sample в owned
//! `CPlayerRanks`: двухсекундный per-player cooldown и `0xBFF30` client wire
//! остаются у snapshot owner-а, загруженного startup/timer цепочкой World.
//! Honor NPC family `2625..2630/2634..2637` читает соседний авторитетный
//! `CHonorRanks` snapshot, публикует type-3 список `0xBFF35` и связывает
//! attempt-appellation с настоящим `CNotDisappearAfterDead` lifecycle:
//! skill-registry lookup, replacement по type/ID, around `0xBFE03/04` и
//! адресный player state `0xBFE02` выполняются из того же script caller-а.
//! Продолжение `changeappellation.script` регистрирует отсутствующий в RU
//! function.ini `11131 / AddJingJieBuff`, снимает прежнюю hidden-skill family,
//! проверяет progression entitlement, ставит новый realm bonus и завершает
//! полный virtual property recompute адресным `0xBF721`.
//! Тот же honor script расширяет единый `GetMe/SetMe` catalog полями
//! `dwAppellationID/dwRankOfNobilityID/dwCredit/dwSZL/lContribute`: чтение
//! идёт из canonical player storage, а обе записи сохраняют общий порядок
//! `0xBF80C → SetScriptValue → UpdateProperty → 0xBF721`.
//! Persisted fairy pair `bFairyContainerEnabled/bBattleFairyEnabled` проходит
//! тот же reached `GetMe/SetMe/ChangePlayer/SetPlayer` dispatcher: bool write
//! нормализует любое ненулевое значение, затем исполняет общий property/wire
//! tail и влияет на следующий exact `CanMountEquip` без shadow-state.
//! `3307 / KillMonster` проводит цель через общего владельца смерти монстра:
//! состояние, сетевое исчезновение, награда, добыча и сценарии смерти остаются
//! в исходном порядке. Вызванный ими `3401 / DropGoods` наследует точку смерти,
//! создаёт товары, занимает ячейки региона и публикует `CS2CContainerObjectMove`
//! игроку и окружающим через настоящую цепочку исполнения `CScript`.
//! Соседний `3402 / AutoMove` вычисляет только две координаты и отправляет
//! игроку точный `0xBF723`; позиция сервера не меняется, хвост аргументов и
//! результат сетевой доставки не прерывают дальнейшее исполнение сценария.
//! Движение `2100..2103` использует тот же пространственный владелец игрока:
//! шаг ходьбой или бегом проходит через `CMoveShape::OnMove`, установка позиции
//! переставляет ячейку региона и публикует `0xBF603`, а направление сохраняет
//! исходный особый порядок полей `0xBF601`.
//! Подтверждённые функции экипировки `2206/2208` разрешают текущего либо
//! именованного игрока, читают позиционный контейнер, а повышение уровня
//! завершает старую сериализацию товара и `0xBF918` вокруг владельца.
//! Числовой селектор получает вычисленные параметры из собственного
//! `CScript`; результат или приостановка диалога возвращается в ту же цепочку
//! исполнения. Остальные идентификаторы функций и неподтверждённые семейства
//! ожидания ниже пока остаются `RAW`.
//! `ReLive 2400` вычисляет только первый аргумент, вызывает общего владельца
//! `CPlayer::OnRelive` и всегда возвращает сценарный ноль; лишние аргументы
//! не вычисляются.
//! `AddState 2323` вычисляет ровно три аргумента и возвращает результат
//! фабрики `CMoveShape`, а `GetStatesNum 2322` вычисляет один ID и считает
//! живые материализованные состояния игрока через тот же достигнутый вызов.

use crate::gameserver::appserver::country::country::{
    CountryExileRestTimeReport, CountryScalarMutationReport,
};
use crate::gameserver::appserver::exstate::ExtendedStateKind;
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::message::gmmessage::publish_local_online_gm_list;
use crate::gameserver::appserver::moveshape::MoveShapeCommandContext;
use crate::gameserver::appserver::organizingsystem::attackcitysys::AttackCityMembershipBlock;
use crate::gameserver::appserver::player::{
    CPlayer, PlayerLeiTingThingCountOutcome, PlayerProgress,
};
use crate::gameserver::appserver::script::buffskillfunc::{
    BuffSkillScriptFunctionOutcome, SCRIPT_FUNCTION_ADD_JING_JIE_BUFF,
    run_buff_skill_script_function,
};
use crate::gameserver::appserver::script::variablelist::GameVariableValue;
use crate::gameserver::appserver::servercityregion::CityGateRuntimeContext;
use crate::gameserver::appserver::servercountryregion::{
    CountryContendEntryContext, CountryContendPlayer, CountryNullPlayerCancelBlock,
};
use crate::gameserver::appserver::serverregion::{
    CServerRegion, RegionTaxSessionKind, ServerRegionMonsterContext, ServerRegionNpcSetup,
};
use crate::gameserver::appserver::serverwarregion::{
    ContendPlayerState, WarContendEntryContext, WarRegionContext,
};
use crate::gameserver::appserver::session::cequipmentdakong::EquipmentDaKongExternalRefreshReport;
use crate::gameserver::appserver::session::csessionfactory::EquipmentSessionPlugKind;
use crate::gameserver::appserver::shape::{ShapeCoordinateBlock, ShapeIdentity, ShapeResolver};
use crate::gameserver::gameserver::game::{
    BattleFairyDeathContext, BattleFairyScriptAction, BattleFairySkillResetContext, CGame,
    EquipmentDaKongContext, EquipmentSessionOpenReport, GameClockContext,
    GameContainerMessageRuntime, GameKickAroundOutcome, GodsBattleDeathContext,
    GodsBattleSzlPlayerUpdate, MonsterDeathContext, NationCarriageReturnReport,
    NationCombatContext, NationContendEnterReport, PlayerReliveContext,
    RealmAppellationScriptContext, ScriptDepotOpenOutcome, ScriptNpcShopOpenOutcome,
    ScriptRegionChangeContext, ServerRegionOwner, colored_player_notice_message,
    colored_text_message, format_legacy_text_fields,
};
use crate::nets::netserver::message::{CMessage, SendMessageError};
use crate::public::date::TagTime;
use crate::public::guid::CGuid;
use crate::public::tools::put_debug_string;
use crate::setup::leitingsetup::{CThingSetup, LeiTingLocalTime};

pub(crate) const SCRIPT_FUNCTION_REFLUSH_EXTERN_PROPERTY: i32 = 9351;
pub(crate) const SCRIPT_FUNCTION_OPEN_DA_KONG: i32 = 9350;
pub(crate) const SCRIPT_FUNCTION_OPEN_CI_QING_PAGE: i32 = 9628;
pub(crate) const SCRIPT_FUNCTION_PUSH_ITEM_TO_CI_QING: i32 = 9629;
pub(crate) const SCRIPT_FUNCTION_OPEN_EQUIPMENT_COMPOSE: i32 = 9354;
pub(crate) const SCRIPT_FUNCTION_OPEN_EQUIPMENT_UPGRADE: i32 = 2216;
pub(crate) const SCRIPT_FUNCTION_OPEN_PRECIOUS_BOX: i32 = 2221;
pub(crate) const SCRIPT_FUNCTION_GET_PRECIOUS_ITEM: i32 = 2222;
pub(crate) const SCRIPT_FUNCTION_DELETE_USED_GOODS: i32 = 2231;
pub(crate) const SCRIPT_FUNCTION_CHECK_USED_GOODS: i32 = 2232;
pub(crate) const SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_1: i32 = 2233;
pub(crate) const SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_2: i32 = 2234;
pub(crate) const SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_1: i32 = 2235;
pub(crate) const SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_2: i32 = 2236;
pub(crate) const SCRIPT_FUNCTION_CLOSE_PRECIOUS_BOX: i32 = 2237;
pub(crate) const SCRIPT_FUNCTION_GET_CURRENT_DURABILITY: i32 = 2243;
pub(crate) const SCRIPT_FUNCTION_SET_CURRENT_DURABILITY: i32 = 2244;
pub(crate) const SCRIPT_FUNCTION_GET_SELECTED_DURABILITY: i32 = 2245;
pub(crate) const SCRIPT_FUNCTION_SET_SELECTED_DURABILITY: i32 = 2246;
pub(crate) const SCRIPT_FUNCTION_FAIRY_EXP_UP: i32 = 2249;
pub(crate) const SCRIPT_FUNCTION_RANDOM: i32 = 8;
pub(crate) const SCRIPT_FUNCTION_RGB: i32 = 9;
pub(crate) const SCRIPT_FUNCTION_TIME: i32 = 13;
pub(crate) const SCRIPT_FUNCTION_YEAR: i32 = 14;
pub(crate) const SCRIPT_FUNCTION_MONTH: i32 = 15;
pub(crate) const SCRIPT_FUNCTION_DAY: i32 = 16;
pub(crate) const SCRIPT_FUNCTION_HOUR: i32 = 17;
pub(crate) const SCRIPT_FUNCTION_MINUTE: i32 = 18;
pub(crate) const SCRIPT_FUNCTION_DAY_OF_WEEK: i32 = 19;
pub(crate) const SCRIPT_FUNCTION_HOUR_DIFF: i32 = 20;
pub(crate) const SCRIPT_FUNCTION_MINUTE_DIFF: i32 = 21;
pub(crate) const SCRIPT_FUNCTION_SECOND: i32 = 23;
pub(crate) const SCRIPT_FUNCTION_GET_STRING_BY_ID: i32 = 2000;
pub(crate) const SCRIPT_FUNCTION_CHANGE_ME: i32 = 2001;
pub(crate) const SCRIPT_FUNCTION_GET_ME: i32 = 2002;
pub(crate) const SCRIPT_FUNCTION_SET_ME: i32 = 2003;
pub(crate) const SCRIPT_FUNCTION_CHECK_LEVEL: i32 = 2004;
pub(crate) const SCRIPT_FUNCTION_SET_ENERGY: i32 = 2005;
pub(crate) const SCRIPT_FUNCTION_SET_MAXIMUM_ENERGY: i32 = 2006;
pub(crate) const SCRIPT_FUNCTION_GET_ENERGY: i32 = 2007;
pub(crate) const SCRIPT_FUNCTION_GET_MAXIMUM_ENERGY: i32 = 2008;
pub(crate) const SCRIPT_FUNCTION_WALK_STEP: i32 = 2100;
pub(crate) const SCRIPT_FUNCTION_RUN_STEP: i32 = 2101;
pub(crate) const SCRIPT_FUNCTION_SET_PLAYER_POSITION: i32 = 2102;
pub(crate) const SCRIPT_FUNCTION_SET_PLAYER_DIRECTION: i32 = 2103;
pub(crate) const SCRIPT_FUNCTION_GET_EQUIPMENT_ID_BY_POSITION: i32 = 2206;
pub(crate) const SCRIPT_FUNCTION_UPGRADE_EQUIPMENT: i32 = 2208;
pub(crate) const SCRIPT_FUNCTION_RE_LIVE: i32 = 2400;
pub(crate) const SCRIPT_FUNCTION_GET_STATES_NUMBER: i32 = 2322;
pub(crate) const SCRIPT_FUNCTION_ADD_STATE: i32 = 2323;
pub(crate) const SCRIPT_FUNCTION_PLAYER_TALK: i32 = 2308;
pub(crate) const SCRIPT_FUNCTION_GET_NAME: i32 = 2998;
pub(crate) const SCRIPT_FUNCTION_IS_CHARGED: i32 = 2516;
pub(crate) const SCRIPT_FUNCTION_SET_CHARGED: i32 = 2517;
pub(crate) const SCRIPT_FUNCTION_ADD_CHANGE_BODY_STATE: i32 = 2518;
pub(crate) const SCRIPT_FUNCTION_DELETE_CHANGE_BODY_STATE: i32 = 2519;
pub(crate) const SCRIPT_FUNCTION_GET_CHANGE_BODY_STATE: i32 = 2520;
pub(crate) const SCRIPT_FUNCTION_CHANGE_BODY_CHECK: i32 = 2521;
pub(crate) const SCRIPT_FUNCTION_CHECK_MODE: i32 = 2522;
pub(crate) const SCRIPT_FUNCTION_GET_PROGRESS: i32 = 2576;
pub(crate) const SCRIPT_FUNCTION_ADD_EX_STATE: i32 = 2550;
pub(crate) const SCRIPT_FUNCTION_DELETE_EX_STATE: i32 = 2551;
pub(crate) const SCRIPT_FUNCTION_GET_EX_STATE: i32 = 2552;
pub(crate) const SCRIPT_FUNCTION_ADD_EX_STATE_NEW: i32 = 2553;
pub(crate) const SCRIPT_FUNCTION_DELETE_EX_STATE_NEW: i32 = 2554;
pub(crate) const SCRIPT_FUNCTION_GET_EX_STATE_NEW: i32 = 2555;
pub(crate) const SCRIPT_FUNCTION_ADD_UNDEAD_STATE: i32 = 2556;
pub(crate) const SCRIPT_FUNCTION_DELETE_UNDEAD_STATE: i32 = 2557;
pub(crate) const SCRIPT_FUNCTION_GET_UNDEAD_STATE: i32 = 2558;
pub(crate) const SCRIPT_FUNCTION_SET_HOTKEY: i32 = 2560;
pub(crate) const SCRIPT_FUNCTION_ADD_LOG: i32 = 2571;
pub(crate) const SCRIPT_FUNCTION_IS_COMBAT_STATE: i32 = 2574;
pub(crate) const SCRIPT_FUNCTION_DRAW_AWARDS: i32 = 2575;
pub(crate) const SCRIPT_FUNCTION_CHANGE_PLAYER: i32 = 2999;
pub(crate) const SCRIPT_FUNCTION_SET_PLAYER: i32 = 3000;
pub(crate) const SCRIPT_FUNCTION_GET_PLAYER: i32 = 3001;
pub(crate) const SCRIPT_FUNCTION_SET_PLAYER_LEVEL: i32 = 3002;
pub(crate) const SCRIPT_FUNCTION_IS_PLAYER_ONLINE: i32 = 3003;
pub(crate) const SCRIPT_FUNCTION_GET_PLAYER_ID: i32 = 3004;
pub(crate) const SCRIPT_FUNCTION_GET_REGION_ID: i32 = 3005;
pub(crate) const SCRIPT_FUNCTION_SET_PLAYER_REGION: i32 = 3006;
pub(crate) const SCRIPT_FUNCTION_SET_PLAYER_REGION_EX: i32 = 3007;
pub(crate) const SCRIPT_FUNCTION_KICK_PLAYER_EX: i32 = 3008;
pub(crate) const SCRIPT_FUNCTION_GET_PLAYER_ALL_PROPERTIES: i32 = 3009;
pub(crate) const SCRIPT_FUNCTION_FORCE_MOVE: i32 = 3010;
pub(crate) const SCRIPT_FUNCTION_CHANGE_MONEY_BY_NAME: i32 = 3011;
pub(crate) const SCRIPT_FUNCTION_GET_MONEY_BY_NAME: i32 = 3012;
pub(crate) const SCRIPT_FUNCTION_SET_MONEY_BY_NAME: i32 = 3013;
pub(crate) const SCRIPT_FUNCTION_CHANGE_MONEY_BY_ID: i32 = 3014;
pub(crate) const SCRIPT_FUNCTION_GET_MONEY_BY_ID: i32 = 3015;
pub(crate) const SCRIPT_FUNCTION_SET_MONEY_BY_ID: i32 = 3016;
pub(crate) const SCRIPT_FUNCTION_GET_PLAYER_ALL_VARIABLES: i32 = 3017;
pub(crate) const SCRIPT_FUNCTION_DELETE_SKILL: i32 = 3102;
pub(crate) const SCRIPT_FUNCTION_SET_SKILL_LEVEL: i32 = 3103;
pub(crate) const SCRIPT_FUNCTION_ADD_SKILL: i32 = 3101;
pub(crate) const SCRIPT_FUNCTION_GET_SKILL_LEVEL: i32 = 3104;
pub(crate) const SCRIPT_FUNCTION_KICK_PLAYER: i32 = 3201;
pub(crate) const SCRIPT_FUNCTION_BAN_PLAYER: i32 = 3202;
pub(crate) const SCRIPT_FUNCTION_SILENCE_PLAYER: i32 = 3203;
pub(crate) const SCRIPT_FUNCTION_CREATE_FACTION: i32 = 6001;
pub(crate) const SCRIPT_FUNCTION_APPLY_JOIN_FACTION: i32 = 6002;
pub(crate) const SCRIPT_FUNCTION_QUIT_JOIN_FACTION: i32 = 6003;
pub(crate) const SCRIPT_FUNCTION_OPERATOR_CITY_GATE: i32 = 6004;
pub(crate) const SCRIPT_FUNCTION_OBTAIN_TAX_PAYMENT: i32 = 6005;
pub(crate) const SCRIPT_FUNCTION_ADJUST_TAX_RATE: i32 = 6006;
pub(crate) const SCRIPT_FUNCTION_UPGRADE_FACTION: i32 = 6011;
pub(crate) const SCRIPT_FUNCTION_GET_FACTION_ID_BY_PLAYER_NAME: i32 = 6015;
pub(crate) const SCRIPT_FUNCTION_IS_FACTION_MASTER_BY_PLAYER_NAME: i32 = 6017;
pub(crate) const SCRIPT_FUNCTION_GET_CITY_GATE_STATE: i32 = 6019;
pub(crate) const SCRIPT_FUNCTION_OPERATE_CITY_GATE: i32 = 6020;
pub(crate) const SCRIPT_FUNCTION_FACTION_DECLARE_WAR: i32 = 6030;
pub(crate) const SCRIPT_FUNCTION_IS_ARRIVE_VILLAGE_APPLY_TIME: i32 = 6040;
pub(crate) const SCRIPT_FUNCTION_IS_ARRIVE_VILLAGE_WAR_TIME: i32 = 6041;
pub(crate) const SCRIPT_FUNCTION_APPLY_FOR_VILLAGE_WAR: i32 = 6042;
pub(crate) const SCRIPT_FUNCTION_ENTER_CONTEND_STATE: i32 = 6043;
pub(crate) const SCRIPT_FUNCTION_CITY_WAR_DECLARE: i32 = 6044;
pub(crate) const SCRIPT_FUNCTION_IS_CITY_WAR_DECLARE_TIME: i32 = 6045;
pub(crate) const SCRIPT_FUNCTION_IS_CITY_WAR_FIGHT_TIME: i32 = 6046;
pub(crate) const SCRIPT_FUNCTION_GET_OWNED_REGION_FACTION_ID: i32 = 6047;
pub(crate) const SCRIPT_FUNCTION_GET_TOTAL_TAX_PAYMENT: i32 = 6049;
pub(crate) const SCRIPT_FUNCTION_GET_TODAY_TAX_PAYMENT: i32 = 6050;
pub(crate) const SCRIPT_FUNCTION_REQUEST_PLAYER_RANKS: i32 = 6051;
pub(crate) const SCRIPT_FUNCTION_SET_TOTAL_TAX_PAYMENT: i32 = 6052;
pub(crate) const SCRIPT_FUNCTION_SET_TODAY_TAX_PAYMENT: i32 = 6053;
pub(crate) const SCRIPT_FUNCTION_GET_DAYS_HONOR_RANK: i32 = 2625;
pub(crate) const SCRIPT_FUNCTION_GET_WEEKS_HONOR_RANK: i32 = 2626;
pub(crate) const SCRIPT_FUNCTION_GET_MONTHS_HONOR_RANK: i32 = 2627;
pub(crate) const SCRIPT_FUNCTION_GET_TOTAL_HONOR_RANK: i32 = 2628;
pub(crate) const SCRIPT_FUNCTION_GET_ATTEMPT_APPELLATION_ID: i32 = 2630;
pub(crate) const SCRIPT_FUNCTION_GET_PLAYER_RANK: i32 = 2631;
pub(crate) const SCRIPT_FUNCTION_DELETE_EX_STATE_BY_TYPE: i32 = 2632;
pub(crate) const SCRIPT_FUNCTION_SEND_TOTAL_HONOR_RANKS: i32 = 2634;
pub(crate) const SCRIPT_FUNCTION_ADD_APPELLATION_STATE: i32 = 2635;
pub(crate) const SCRIPT_FUNCTION_DEL_APPELLATION_STATE: i32 = 2636;
pub(crate) const SCRIPT_FUNCTION_GET_APPELLATION_STATE: i32 = 2637;
pub(crate) const SCRIPT_FUNCTION_SET_THING_COUNT: i32 = 2650;
pub(crate) const SCRIPT_FUNCTION_GET_THING_COUNT: i32 = 2651;
pub(crate) const SCRIPT_FUNCTION_SET_JING_LI_DAN: i32 = 2652;
pub(crate) const SCRIPT_FUNCTION_GET_JING_LI_DAN: i32 = 2653;
pub(crate) const SCRIPT_FUNCTION_GET_WAR_REGION_STATE: i32 = 6054;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_OWNING_REGION: i32 = 6055;
pub(crate) const SCRIPT_FUNCTION_GET_WAR_START_TIME: i32 = 6056;
pub(crate) const SCRIPT_FUNCTION_CHANGE_REGION: i32 = 2304;
pub(crate) const SCRIPT_FUNCTION_ADD_GOODS: i32 = 2200;
pub(crate) const SCRIPT_FUNCTION_DELETE_GOODS: i32 = 2201;
pub(crate) const SCRIPT_FUNCTION_CHECK_GOODS: i32 = 2202;
pub(crate) const SCRIPT_FUNCTION_CHECK_SPACE: i32 = 2203;
pub(crate) const SCRIPT_FUNCTION_GET_GOODS_NUMBER: i32 = 2204;
pub(crate) const SCRIPT_FUNCTION_GET_FREE_SPACE: i32 = 2205;
pub(crate) const SCRIPT_FUNCTION_UPGRADE_SELECTED_EQUIPMENT: i32 = 2209;
pub(crate) const SCRIPT_FUNCTION_ADD_DEPOT_GOODS: i32 = 2210;
pub(crate) const SCRIPT_FUNCTION_DELETE_DEPOT_GOODS: i32 = 2211;
pub(crate) const SCRIPT_FUNCTION_CHECK_DEPOT_GOODS: i32 = 2212;
pub(crate) const SCRIPT_FUNCTION_CHECK_DEPOT_SPACE: i32 = 2213;
pub(crate) const SCRIPT_FUNCTION_GET_DEPOT_GOODS_NUMBER: i32 = 2214;
pub(crate) const SCRIPT_FUNCTION_GET_DEPOT_GOODS_FREE: i32 = 2215;
pub(crate) const SCRIPT_FUNCTION_DELETE_PLAYER_GOODS: i32 = 2217;
pub(crate) const SCRIPT_FUNCTION_GET_CONTAINER_ITEM_TYPE: i32 = 2218;
pub(crate) const SCRIPT_FUNCTION_OPEN_GOODS_CONTAINER: i32 = 2220;
pub(crate) const SCRIPT_FUNCTION_GET_GOODS_PROPERTY_1: i32 = 2223;
pub(crate) const SCRIPT_FUNCTION_GET_GOODS_PROPERTY_2: i32 = 2224;
pub(crate) const SCRIPT_FUNCTION_DELETE_SPLIT_GOODS: i32 = 2225;
pub(crate) const SCRIPT_FUNCTION_GET_GOODS_ORIGINAL_NAME: i32 = 2226;
pub(crate) const SCRIPT_FUNCTION_GET_GOODS_PRICE: i32 = 2227;
pub(crate) const SCRIPT_FUNCTION_SET_GOODS_PROPERTY_1: i32 = 2228;
pub(crate) const SCRIPT_FUNCTION_SET_GOODS_PROPERTY_2: i32 = 2229;
pub(crate) const SCRIPT_FUNCTION_RECREATE_GOODS_ADDON_PROPERTIES: i32 = 2230;
pub(crate) const SCRIPT_FUNCTION_GET_DIED_MONSTER_INDEX: i32 = 2240;
pub(crate) const SCRIPT_FUNCTION_GET_DIED_MONSTER_ORIGINAL_NAME: i32 = 2241;
pub(crate) const SCRIPT_FUNCTION_GET_DIED_MONSTER_LEVEL: i32 = 2242;
pub(crate) const SCRIPT_FUNCTION_OPEN_NPC_SHOP: i32 = 2300;
pub(crate) const SCRIPT_FUNCTION_OPEN_DEPOT: i32 = 2301;
pub(crate) const SCRIPT_FUNCTION_GET_TEAM_NUM: i32 = 2302;
pub(crate) const SCRIPT_FUNCTION_GET_TEAMER_NAME: i32 = 2303;
pub(crate) const SCRIPT_FUNCTION_ADD_INFO: i32 = 2305;
pub(crate) const SCRIPT_FUNCTION_GAME_MESSAGE: i32 = 2306;
pub(crate) const SCRIPT_FUNCTION_TALK_BOX: i32 = 2307;
pub(crate) const SCRIPT_FUNCTION_HELP: i32 = 2309;
pub(crate) const SCRIPT_FUNCTION_TALK_BOX_SMALL: i32 = 2324;
pub(crate) const SCRIPT_FUNCTION_ADD_GOODS_LOG: i32 = 2313;
pub(crate) const SCRIPT_FUNCTION_ADD_GEM_EXCHANGE_LOG: i32 = 2314;
pub(crate) const SCRIPT_FUNCTION_ADD_JEWELRY_MADE_LOG: i32 = 2315;
pub(crate) const SCRIPT_FUNCTION_OPEN_NEW_HELP_WINDOW: i32 = 2319;
pub(crate) const SCRIPT_FUNCTION_GET_GOODS_PROPERTY: i32 = 2321;
pub(crate) const SCRIPT_FUNCTION_SET_REGION_FOR_TEAM: i32 = 2310;
pub(crate) const SCRIPT_FUNCTION_SET_TEAM_REGION: i32 = 2311;
pub(crate) const SCRIPT_FUNCTION_IS_TEAMMATES_AROUND_ME: i32 = 2312;
pub(crate) const SCRIPT_FUNCTION_SCRIPT_IS_RUNNING: i32 = 2316;
pub(crate) const SCRIPT_FUNCTION_REMOVE_SCRIPT: i32 = 2317;
pub(crate) const SCRIPT_FUNCTION_ADD_FU_MO_PROPERTY: i32 = 2320;
pub(crate) const SCRIPT_FUNCTION_IS_TEAM_CAPTAIN: i32 = 2325;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY: i32 = 2500;
pub(crate) const SCRIPT_FUNCTION_CHANGE_COUNTRY: i32 = 2501;
pub(crate) const SCRIPT_FUNCTION_GET_CONTRIBUTION: i32 = 2502;
pub(crate) const SCRIPT_FUNCTION_SET_CONTRIBUTION: i32 = 2503;
pub(crate) const SCRIPT_FUNCTION_GET_YUAN_BAO: i32 = 2504;
pub(crate) const SCRIPT_FUNCTION_ADD_INCREMENT_LOG: i32 = 2570;
pub(crate) const SCRIPT_FUNCTION_LIST_ONLINE_GM: i32 = 5103;
pub(crate) const SCRIPT_FUNCTION_LIST_SILENCE_PLAYER: i32 = 5104;
pub(crate) const SCRIPT_FUNCTION_SAVE_ALL_PLAYERS: i32 = 5105;
pub(crate) const SCRIPT_FUNCTION_GET_ONLINE_PLAYERS: i32 = 5108;
pub(crate) const SCRIPT_FUNCTION_LIST_ONLINE_PLAYER: i32 = 5109;
pub(crate) const SCRIPT_FUNCTION_LIST_BANNED_PLAYER: i32 = 5106;
pub(crate) const SCRIPT_FUNCTION_KICK_ALL: i32 = 5301;
pub(crate) const SCRIPT_FUNCTION_KICK_MAP: i32 = 5302;
pub(crate) const SCRIPT_FUNCTION_GET_COPY_NUMBER: i32 = 9314;
pub(crate) const SCRIPT_FUNCTION_GET_LEVEL_EXPERIENCE: i32 = 5411;
pub(crate) const SCRIPT_FUNCTION_GET_MAXIMUM_LEVEL: i32 = 5412;
pub(crate) const SCRIPT_FUNCTION_GET_AREA_ID: i32 = 5413;
pub(crate) const SCRIPT_FUNCTION_GET_AREA_TYPE: i32 = 5414;
pub(crate) const SCRIPT_FUNCTION_GET_WORLD_SERVER_ID: i32 = 5420;
pub(crate) const SCRIPT_FUNCTION_PLAY_EFFECT: i32 = 5404;
pub(crate) const SCRIPT_FUNCTION_WEATHER: i32 = 5403;
pub(crate) const SCRIPT_FUNCTION_PLAY_ACTION: i32 = 5405;
pub(crate) const SCRIPT_FUNCTION_INVISIBLE: i32 = 5401;
pub(crate) const SCRIPT_FUNCTION_GOD_MODE: i32 = 5402;
pub(crate) const SCRIPT_FUNCTION_RESIDENT_MODE: i32 = 5406;
pub(crate) const SCRIPT_FUNCTION_GM_MODE: i32 = 5407;
pub(crate) const SCRIPT_FUNCTION_PLAY_SOUND: i32 = 5410;
pub(crate) const SCRIPT_FUNCTION_RELOAD: i32 = 5001;
pub(crate) const SCRIPT_FUNCTION_POST_PLAYER_INFO: i32 = 3316;
pub(crate) const SCRIPT_FUNCTION_IS_RIDER: i32 = 3317;
pub(crate) const SCRIPT_FUNCTION_DROP_GOODS: i32 = 3401;
pub(crate) const SCRIPT_FUNCTION_AUTO_MOVE: i32 = 3402;
pub(crate) const SCRIPT_FUNCTION_POST_REGION_INFO: i32 = 5201;
pub(crate) const SCRIPT_FUNCTION_POST_WORLD_INFO: i32 = 5202;
pub(crate) const SCRIPT_FUNCTION_POST_COUNTRY_INFO: i32 = 5203;
pub(crate) const SCRIPT_FUNCTION_ADD_QUEST: i32 = 6200;
pub(crate) const SCRIPT_FUNCTION_COMPLETE_QUEST: i32 = 6201;
pub(crate) const SCRIPT_FUNCTION_DISBAND_QUEST: i32 = 6202;
pub(crate) const SCRIPT_FUNCTION_GET_QUEST_STATE: i32 = 6203;
pub(crate) const SCRIPT_FUNCTION_UPDATE_QUEST_POSITION: i32 = 6207;
pub(crate) const SCRIPT_FUNCTION_NPC_TALK: i32 = 3301;
pub(crate) const SCRIPT_FUNCTION_CREATE_NPC: i32 = 3302;
pub(crate) const SCRIPT_FUNCTION_DELETE_NPC: i32 = 3303;
pub(crate) const SCRIPT_FUNCTION_MONSTER_TALK: i32 = 3304;
pub(crate) const SCRIPT_FUNCTION_CREATE_MONSTER: i32 = 3305;
pub(crate) const SCRIPT_FUNCTION_DELETE_MONSTER: i32 = 3306;
pub(crate) const SCRIPT_FUNCTION_KILL_MONSTER: i32 = 3307;
pub(crate) const SCRIPT_FUNCTION_PLAYER_MESSAGE: i32 = 3308;
pub(crate) const SCRIPT_FUNCTION_GET_MAP_INFO: i32 = 3309;
pub(crate) const SCRIPT_FUNCTION_OPEN_PLAYER_UI: i32 = 3310;
pub(crate) const SCRIPT_FUNCTION_CALL_MONSTER: i32 = 3311;
pub(crate) const SCRIPT_FUNCTION_ATTACK_PLAYER: i32 = 3312;
pub(crate) const SCRIPT_FUNCTION_DELETE_MONSTER_RECT: i32 = 3313;
pub(crate) const SCRIPT_FUNCTION_MOVE_PLAYER: i32 = 3314;
pub(crate) const SCRIPT_FUNCTION_DELETE_NPC_BY_NAME: i32 = 3315;
pub(crate) const SCRIPT_FUNCTION_REFRESH_BLOCK: i32 = 8000;
pub(crate) const SCRIPT_FUNCTION_GET_REGION_RANDOM_POSITION: i32 = 8003;
pub(crate) const SCRIPT_FUNCTION_OPEN_CHANGE_PLAYER_NAME: i32 = 8100;
pub(crate) const SCRIPT_FUNCTION_GET_MONSTER_REFRESH_TIME: i32 = 8101;
pub(crate) const SCRIPT_FUNCTION_IS_QUEST_ENABLED: i32 = 3500;
pub(crate) const SCRIPT_FUNCTION_SET_QUEST_ENABLED: i32 = 3501;
pub(crate) const SCRIPT_FUNCTION_QUEST_TIME_BEGIN: i32 = 3502;
pub(crate) const SCRIPT_FUNCTION_QUEST_TIME_CLEAR: i32 = 3503;
pub(crate) const SCRIPT_FUNCTION_ADD_CARRIAGE: i32 = 3504;
pub(crate) const SCRIPT_FUNCTION_DELETE_CARRIAGE: i32 = 3505;
pub(crate) const SCRIPT_FUNCTION_GET_CARRIAGE_DISTANCE: i32 = 3506;
pub(crate) const SCRIPT_FUNCTION_GET_QUEST_TIME: i32 = 3507;
pub(crate) const SCRIPT_FUNCTION_GET_CARRIAGE_INDEX: i32 = 3508;
pub(crate) const SCRIPT_FUNCTION_OPEN_SYNTHESIS: i32 = 3510;
pub(crate) const SCRIPT_FUNCTION_ACTIVITY_LOG: i32 = 9510;
pub(crate) const SCRIPT_FUNCTION_SET_COUNTRY_POWER: i32 = 9001;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_POWER: i32 = 9000;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_TECH_LEVEL: i32 = 9002;
pub(crate) const SCRIPT_FUNCTION_SET_COUNTRY_TECH_LEVEL: i32 = 9003;
pub(crate) const SCRIPT_FUNCTION_SET_COUNTRY_TREASURY: i32 = 9009;
pub(crate) const SCRIPT_FUNCTION_SET_COUNTRY_MATERIAL: i32 = 9011;
pub(crate) const SCRIPT_FUNCTION_SET_COUNTRY_TECH: i32 = 9013;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_CI: i32 = 9004;
pub(crate) const SCRIPT_FUNCTION_SET_COUNTRY_CI: i32 = 9005;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_KING_ID: i32 = 9006;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_TREASURY: i32 = 9008;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_MATERIAL: i32 = 9010;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_TECH: i32 = 9012;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_OCCUPATION: i32 = 2633;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_IDENTITY: i32 = 9020;
pub(crate) const SCRIPT_FUNCTION_GET_QUEST_SWITCH: i32 = 9018;
pub(crate) const SCRIPT_FUNCTION_SET_QUEST_SWITCH: i32 = 9019;
pub(crate) const SCRIPT_FUNCTION_EXILE_TIME: i32 = 9021;
pub(crate) const SCRIPT_FUNCTION_ADD_KING_POINT: i32 = 9317;
pub(crate) const SCRIPT_FUNCTION_GET_PLAYER_SZL: i32 = 11124;
pub(crate) const SCRIPT_FUNCTION_CHANGE_PLAYER_SZL: i32 = 11128;
pub(crate) const SCRIPT_FUNCTION_DECLARE_COUNTRY_WAR: i32 = 9100;
pub(crate) const SCRIPT_FUNCTION_IS_COUNTRY_WAR_DECLARE: i32 = 9101;
pub(crate) const SCRIPT_FUNCTION_IS_COUNTRY_DECLARED: i32 = 9102;
pub(crate) const SCRIPT_FUNCTION_IS_COUNTRY_WAR_PREPARE: i32 = 9103;
pub(crate) const SCRIPT_FUNCTION_IS_COUNTRY_WAR: i32 = 9104;
pub(crate) const SCRIPT_FUNCTION_ENTER_COUNTRY_CONTEND: i32 = 9105;
pub(crate) const SCRIPT_FUNCTION_IS_COUNTRY_WIN_SYMBOL: i32 = 9106;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_WAR_REGION: i32 = 9107;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_WAR_CAMP: i32 = 9108;
pub(crate) const SCRIPT_FUNCTION_GET_OTHER_WAR_COUNTRY: i32 = 9109;
pub(crate) const SCRIPT_FUNCTION_COUNTRY_WAR_VICTORY: i32 = 9110;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_WAR_RESULT: i32 = 9111;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_SEND_PLAYER_ID: i32 = 9304;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_CARRIAGE_BACK_TOWN: i32 = 9305;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_GET_FLAG_STATUS: i32 = 9306;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_GET_TIME: i32 = 9307;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_ENTER_CONTEND: i32 = 9308;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_COUNTRY_SIGN_UP: i32 = 9309;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_CLEAR_PLAYER_TIME: i32 = 9310;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_GET_NATION_STATUS: i32 = 9311;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_SET_PLAYER_TIME: i32 = 9312;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_IS_PLAYER_WEAK: i32 = 9313;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_GET_MORALE: i32 = 9315;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_CLEAR_MORALE: i32 = 9316;
pub(crate) const SCRIPT_FUNCTION_ADD_BATTLE_FAIRY_SKILL: i32 = 9400;
pub(crate) const SCRIPT_FUNCTION_GET_FETCH_POWER: i32 = 9401;
pub(crate) const SCRIPT_FUNCTION_SET_BATTLE_FAIRY_ATTRIBUTE: i32 = 9402;
pub(crate) const SCRIPT_FUNCTION_ALLOCATE_BATTLE_FAIRY_SKILL: i32 = 9403;
pub(crate) const SCRIPT_FUNCTION_ALLOCATE_BATTLE_FAIRY_SPECIAL_SKILL: i32 = 9404;
pub(crate) const SCRIPT_FUNCTION_REVIVE_BATTLE_FAIRY: i32 = 9406;
pub(crate) const SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_ID: i32 = 9407;
pub(crate) const SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_LEVEL: i32 = 9408;
pub(crate) const SCRIPT_FUNCTION_ADD_BATTLE_FAIRY_EXPERIENCE: i32 = 9409;
pub(crate) const SCRIPT_FUNCTION_GET_BATTLE_FAIRY_ATTRIBUTE: i32 = 9410;
pub(crate) const SCRIPT_FUNCTION_RECREATE_BATTLE_FAIRY_ATTRIBUTES: i32 = 9411;
const SCRIPT_INT_PARAMETER_ERROR: i32 = 0x09ff_fff9;
pub(crate) const SCRIPT_FUNCTION_ARGUMENT_CAPACITY: usize = 12;
const MAXIMUM_SCRIPT_SPAWN_COUNT: i32 = 4096;
const SCRIPT_PLAYER_TYPE: i32 = 400;
const SCRIPT_NPC_TYPE: i32 = 500;

pub(crate) trait ScriptAwardAuthenticationContext {
    /// Внешняя UniBill/Bsip граница exact `AwardAuthenByPatchID`: реализация
    /// создаёт order IDs, удерживает pending bill record до callback-а и
    /// возвращает immediate vendor result. `DrawAwards` исторически его
    /// игнорирует и сообщает лишь факт принятия запроса.
    fn submit_script_award_authentication(
        &mut self,
        player: &CPlayer,
        patch_id: i32,
        information_type: i32,
        color: u32,
        background: u32,
    ) -> i32;
}

pub(crate) trait ScriptFunctionRuntime:
    GameClockContext
    + NationCombatContext
    + EquipmentDaKongContext
    + GameContainerMessageRuntime
    + BattleFairyDeathContext
    + BattleFairySkillResetContext
    + ScriptRegionChangeContext
    + CityGateRuntimeContext
    + GodsBattleDeathContext
    + RealmAppellationScriptContext
    + ScriptAwardAuthenticationContext
    + MoveShapeCommandContext
    + ServerRegionMonsterContext
    + PlayerReliveContext
    + MonsterDeathContext
{
}

impl<T> ScriptFunctionRuntime for T where
    T: GameClockContext
        + NationCombatContext
        + EquipmentDaKongContext
        + GameContainerMessageRuntime
        + BattleFairyDeathContext
        + BattleFairySkillResetContext
        + ScriptRegionChangeContext
        + CityGateRuntimeContext
        + GodsBattleDeathContext
        + RealmAppellationScriptContext
        + ScriptAwardAuthenticationContext
        + MoveShapeCommandContext
        + ServerRegionMonsterContext
        + PlayerReliveContext
        + MonsterDeathContext
{
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WarContendEffect {
    PlayerState {
        player_id: i32,
        state: bool,
        changed: bool,
    },
    Time {
        player_id: i32,
        percentage: i32,
        delivery: i32,
    },
    Notice {
        player_id: Option<i32>,
        string_id: &'static str,
        delivery: i32,
    },
    FirstFactionNotice {
        country: u8,
        faction_name: String,
        symbol_name: String,
        delivery: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WarContendScriptDisposition {
    CallerMissing,
    CallerShapeMissing,
    TooFar {
        distance: i32,
        maximum: i32,
        delivery: i32,
    },
    ArgumentMissing {
        argument: usize,
    },
    RegionMissing {
        region_id: Option<i32>,
    },
    RegionNotWar {
        region_id: i32,
    },
    NpcMissing {
        region_id: i32,
        npc_id: i32,
    },
    PlayerWithoutFaction,
    ContendInvoked {
        region_id: i32,
        player_id: i32,
        symbol_id: i32,
        duration_ms: i32,
        required_goods: [String; 4],
        result: Result<(), AttackCityMembershipBlock>,
        effects: Vec<WarContendEffect>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WarContendScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        legacy_return: i32,
        disposition: WarContendScriptDisposition,
    },
}

#[derive(Clone, Copy)]
enum ContendEntrySchedule {
    City,
    Village,
}

struct GameWarContendEntryContext<'a, Runtime> {
    game: &'a mut CGame,
    runtime: &'a mut Runtime,
    region: CServerRegion,
    war_number: i32,
    owner_faction_id: i32,
    schedule: ContendEntrySchedule,
    needed_goods: Vec<String>,
    effects: Vec<WarContendEffect>,
}

impl<Runtime> WarRegionContext for GameWarContendEntryContext<'_, Runtime> {
    type MembershipError = AttackCityMembershipBlock;

    fn player_faction_id(&mut self, player_id: i32) -> Option<i32> {
        self.game
            .find_player(player_id)
            .map(|player| player.faction_id())
    }

    fn is_apply_war_faction(&mut self, faction_id: i32) -> Result<bool, Self::MembershipError> {
        match self.schedule {
            ContendEntrySchedule::City => self
                .game
                .attack_city_sys()
                .is_already_declar_for_war(self.war_number, faction_id),
            ContendEntrySchedule::Village => Ok(self
                .game
                .village_war_sys()
                .is_already_declar_for_war(self.war_number, faction_id)),
        }
    }

    fn send_contend_time(&mut self, player_id: i32, time: i32) {
        let mut message = CMessage::new(0x000b_ff29);
        message.base_mut().add_long(time);
        let delivery = message.send_to_player(self.game.net_server(), player_id);
        self.effects.push(WarContendEffect::Time {
            player_id,
            percentage: time,
            delivery,
        });
    }

    fn set_global_player_contend_state(&mut self, player_id: i32, state: bool) {
        let changed = self
            .game
            .publish_war_player_contend_state(&self.region, player_id, state)
            .is_some();
        self.effects.push(WarContendEffect::PlayerState {
            player_id,
            state,
            changed,
        });
    }

    fn set_region_player_contend_state(&mut self, region_id: i32, player_id: i32, state: bool) {
        if self
            .game
            .find_player(player_id)
            .is_some_and(|player| player.server_region_id() == Some(region_id))
        {
            self.set_global_player_contend_state(player_id, state);
        }
    }
}

impl<Runtime: GameClockContext> WarContendEntryContext for GameWarContendEntryContext<'_, Runtime> {
    fn now_millis(&mut self) -> u32 {
        self.runtime.now_milliseconds()
    }

    fn is_owner(&mut self, faction_id: i32) -> bool {
        faction_id != 0 && faction_id == self.owner_faction_id
    }

    fn player_has_good(&mut self, player_id: i32, good_name: &str) -> bool {
        let goods_index = self
            .game
            .goods_factory()
            .query_goods_id_by_original_name(Some(good_name.as_bytes()));
        self.game.find_player(player_id).is_some_and(|player| {
            player.check_item_in_packet(goods_index) != 0
                || player
                    .equipment()
                    .traversing_goods()
                    .into_iter()
                    .any(|(_, goods)| goods.base_properties_index() == goods_index)
        })
    }

    fn set_known_player_contend_state(&mut self, player_id: i32, state: bool) {
        self.set_global_player_contend_state(player_id, state);
    }

    fn notify_player(&mut self, player_id: i32, string_id: &'static str) {
        let delivery = send_country_war_script_notice(self.game, player_id, string_id.as_bytes());
        self.effects.push(WarContendEffect::Notice {
            player_id: Some(player_id),
            string_id,
            delivery,
        });
    }

    fn register_needed_good(&mut self, good_name: &str) {
        if !good_name.is_empty() {
            self.needed_goods.push(good_name.to_owned());
        }
    }

    fn send_first_faction_contender_notice(
        &mut self,
        country: u8,
        faction_name: &str,
        symbol_name: &str,
    ) {
        let country_name = self
            .game
            .globe_setup()
            .country_name(country)
            .unwrap_or_default();
        let text = format_legacy_text_fields(
            self.game.get_string_by_id(b"GS0246"),
            &[
                country_name,
                faction_name.as_bytes(),
                symbol_name.as_bytes(),
            ],
            0xff,
        );
        let delivery = colored_player_notice_message(0xffff_ffff, 0xffff_0000, &text)
            .send_to_region(Some(&self.region), None, self.game);
        self.effects.push(WarContendEffect::FirstFactionNotice {
            country,
            faction_name: faction_name.to_owned(),
            symbol_name: symbol_name.to_owned(),
            delivery,
        });
    }
}

pub(crate) fn run_war_contend_script_function<Runtime: GameClockContext>(
    game: &mut CGame,
    runtime: &mut Runtime,
    script_player_id: Option<i32>,
    script_npc_id: Option<i32>,
    script_region_id: Option<i32>,
    function_id: i32,
    integer_arguments: [Option<i32>; 2],
    string_arguments: [Option<&[u8]>; 4],
) -> WarContendScriptFunctionOutcome {
    if function_id != SCRIPT_FUNCTION_ENTER_CONTEND_STATE {
        return WarContendScriptFunctionOutcome::DifferentFunction;
    }
    let handled = |disposition| WarContendScriptFunctionOutcome::Handled {
        legacy_return: 0,
        disposition,
    };
    let (Some(player_id), Some(npc_id)) = (script_player_id, script_npc_id) else {
        return handled(WarContendScriptDisposition::CallerMissing);
    };
    let player_shape = game.resolve_shape(ShapeIdentity {
        object_type: SCRIPT_PLAYER_TYPE,
        id: player_id,
        ex_id: CGuid::GUID_INVALID,
    });
    let npc_shape = game.resolve_shape(ShapeIdentity {
        object_type: SCRIPT_NPC_TYPE,
        id: npc_id,
        ex_id: CGuid::GUID_INVALID,
    });
    let (Some(player_shape), Some(npc_shape)) = (player_shape, npc_shape) else {
        return handled(WarContendScriptDisposition::CallerShapeMissing);
    };
    let distance = npc_shape.distance(player_shape);
    if distance > 2 {
        let delivery = send_country_war_script_notice(game, player_id, b"GS0208");
        return handled(WarContendScriptDisposition::TooFar {
            distance,
            maximum: 2,
            delivery,
        });
    }
    let mut values = [0; 2];
    for (index, argument) in integer_arguments.into_iter().enumerate() {
        let Some(value) = argument.filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR) else {
            return handled(WarContendScriptDisposition::ArgumentMissing { argument: index });
        };
        values[index] = value;
    }
    let mut required_goods: [String; 4] = std::array::from_fn(|_| String::new());
    for (index, argument) in string_arguments.into_iter().enumerate() {
        let Some(value) = argument else {
            return handled(WarContendScriptDisposition::ArgumentMissing {
                argument: index + 2,
            });
        };
        required_goods[index] = String::from_utf8_lossy(value).into_owned();
    }
    let Some(region_id) = script_region_id else {
        return handled(WarContendScriptDisposition::RegionMissing { region_id: None });
    };
    let Some(player) = game.find_player(player_id) else {
        return handled(WarContendScriptDisposition::CallerMissing);
    };
    if player.faction_id() <= 0 {
        return handled(WarContendScriptDisposition::PlayerWithoutFaction);
    };
    let player = ContendPlayerState {
        player_id,
        faction_id: player.faction_id(),
        union_id: player.union_id(),
        country: player.country(),
        faction_name: String::from_utf8_lossy(player.faction_name()).into_owned(),
        shape_type: i32::from(player.shape().get_action()),
        is_dead: player.is_dead(),
    };
    let Some(owner) = game.find_region(region_id) else {
        return handled(WarContendScriptDisposition::RegionMissing {
            region_id: Some(region_id),
        });
    };
    let (war_number, owner_faction_id, schedule) = match owner {
        ServerRegionOwner::City(city) => (
            city.war.base.get_war_number(),
            city.war.base.owned_city_faction(),
            ContendEntrySchedule::City,
        ),
        ServerRegionOwner::Village(village) => {
            let war_number = village.war.base.get_war_number();
            let village_id = game
                .village_war_sys()
                .get_village_region_id_by_time(war_number);
            let owner_faction_id = game
                .find_region(village_id)
                .map(|region| region.base().owned_city_faction())
                .or_else(|| {
                    game.find_proxy_region(village_id)
                        .map(|region| region.owned_city_org().0)
                })
                .unwrap_or(0);
            (war_number, owner_faction_id, ContendEntrySchedule::Village)
        }
        _ => return handled(WarContendScriptDisposition::RegionNotWar { region_id }),
    };
    let Some(mut owner) = game.take_region_owner(region_id) else {
        return handled(WarContendScriptDisposition::RegionMissing {
            region_id: Some(region_id),
        });
    };
    let war = match &mut owner {
        ServerRegionOwner::City(region) => &mut region.war,
        ServerRegionOwner::Village(region) => &mut region.war,
        _ => unreachable!("war owner проверен до take"),
    };
    let Some(symbol_name) = war
        .base
        .find_npc_by_id(npc_id)
        .map(|npc| String::from_utf8_lossy(npc.name()).into_owned())
    else {
        game.restore_region_owner(owner);
        return handled(WarContendScriptDisposition::NpcMissing { region_id, npc_id });
    };
    let duration_ms = values[1].wrapping_mul(1_000);
    let mut context = GameWarContendEntryContext {
        game,
        runtime,
        region: war.base.clone(),
        war_number,
        owner_faction_id,
        schedule,
        needed_goods: Vec::new(),
        effects: Vec::new(),
    };
    let result = war.on_enter_contend(
        Some(&player),
        values[0],
        &symbol_name,
        duration_ms,
        std::array::from_fn(|index| required_goods[index].as_str()),
        &mut context,
    );
    let needed_goods = std::mem::take(&mut context.needed_goods);
    let effects = std::mem::take(&mut context.effects);
    drop(context);
    if let ServerRegionOwner::Village(region) = &mut owner {
        for good_name in &needed_goods {
            region.add_need_good(good_name);
        }
    }
    game.restore_region_owner(owner);
    handled(WarContendScriptDisposition::ContendInvoked {
        region_id,
        player_id,
        symbol_id: values[0],
        duration_ms,
        required_goods,
        result,
        effects,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleScalarScriptKind {
    AreaId,
    PlayerSzl,
    ChangePlayerSzl,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleScalarScriptDisposition {
    Scalar {
        player_id: Option<i32>,
        value: i32,
    },
    ArgumentMissing,
    Changed {
        player_id: Option<i32>,
        requested: i32,
        update: Option<GodsBattleSzlPlayerUpdate>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleScalarScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        kind: GodsBattleScalarScriptKind,
        legacy_return: i32,
        disposition: GodsBattleScalarScriptDisposition,
    },
}

pub(crate) fn run_gods_battle_scalar_script_function<Runtime: GodsBattleDeathContext>(
    game: &mut CGame,
    runtime: &mut Runtime,
    script_player_id: Option<i32>,
    function_id: i32,
    evaluated_argument: Option<i32>,
) -> GodsBattleScalarScriptFunctionOutcome {
    let kind = match function_id {
        SCRIPT_FUNCTION_GET_AREA_ID => GodsBattleScalarScriptKind::AreaId,
        SCRIPT_FUNCTION_GET_PLAYER_SZL => GodsBattleScalarScriptKind::PlayerSzl,
        SCRIPT_FUNCTION_CHANGE_PLAYER_SZL => GodsBattleScalarScriptKind::ChangePlayerSzl,
        _ => return GodsBattleScalarScriptFunctionOutcome::DifferentFunction,
    };
    let handled = |legacy_return, disposition| GodsBattleScalarScriptFunctionOutcome::Handled {
        kind,
        legacy_return,
        disposition,
    };
    match kind {
        GodsBattleScalarScriptKind::AreaId => {
            let value = game.area_id();
            handled(
                value,
                GodsBattleScalarScriptDisposition::Scalar {
                    player_id: script_player_id,
                    value,
                },
            )
        }
        GodsBattleScalarScriptKind::PlayerSzl => {
            let value = script_player_id
                .and_then(|player_id| game.find_player(player_id))
                .map_or(0, |player| player.szl() as i32);
            handled(
                value,
                GodsBattleScalarScriptDisposition::Scalar {
                    player_id: script_player_id,
                    value,
                },
            )
        }
        GodsBattleScalarScriptKind::ChangePlayerSzl => {
            let Some(requested) =
                evaluated_argument.filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return handled(0, GodsBattleScalarScriptDisposition::ArgumentMissing);
            };
            let update = script_player_id
                .and_then(|player_id| game.script_change_player_szl(player_id, requested, runtime));
            handled(
                0,
                GodsBattleScalarScriptDisposition::Changed {
                    player_id: script_player_id,
                    requested,
                    update,
                },
            )
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarActionKind {
    EnterContend,
    PublishVictory,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarContendEffect {
    PlayerState {
        player_id: i32,
        state: bool,
        changed: bool,
        around_delivery: Option<Result<i32, ShapeCoordinateBlock>>,
    },
    Time {
        player_id: i32,
        percentage: i32,
        delivery: i32,
    },
    Notice {
        player_id: i32,
        string_id: &'static str,
        delivery: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarActionScriptDisposition {
    CallerMissing,
    CallerShapeMissing,
    WarClosed {
        delivery: i32,
    },
    TooFar {
        distance: i32,
        maximum: i32,
        delivery: i32,
    },
    ArgumentMissing {
        argument: usize,
    },
    RegionMissing {
        region_id: Option<i32>,
    },
    RegionNotCountry {
        region_id: i32,
    },
    NpcMissing {
        region_id: i32,
        npc_id: i32,
    },
    ContendInvoked {
        region_id: i32,
        player_id: i32,
        symbol_id: i32,
        duration_ms: i32,
        result: Result<(), CountryNullPlayerCancelBlock>,
        effects: Vec<CountryWarContendEffect>,
    },
    VictoryNotPublished,
    VictoryPublished {
        country: u8,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarActionScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        function_id: i32,
        kind: CountryWarActionKind,
        legacy_return: i32,
        disposition: CountryWarActionScriptDisposition,
    },
}

struct GameCountryContendEntryContext<'a, Runtime> {
    game: &'a mut CGame,
    runtime: &'a mut Runtime,
    region: CServerRegion,
    effects: Vec<CountryWarContendEffect>,
}

impl<Runtime: GameClockContext> CountryContendEntryContext
    for GameCountryContendEntryContext<'_, Runtime>
{
    fn now_millis(&mut self) -> u32 {
        self.runtime.now_milliseconds()
    }

    fn send_contend_time(&mut self, player_id: i32, time: i32) {
        let mut message = CMessage::new(0x000b_ff29);
        message.base_mut().add_long(time);
        let delivery = message.send_to_player(self.game.net_server(), player_id);
        self.effects.push(CountryWarContendEffect::Time {
            player_id,
            percentage: time,
            delivery,
        });
    }

    fn set_known_player_contend_state(&mut self, player_id: i32, state: bool) {
        let around_delivery =
            self.game
                .publish_war_player_contend_state(&self.region, player_id, state);
        self.effects.push(CountryWarContendEffect::PlayerState {
            player_id,
            state,
            changed: around_delivery.is_some(),
            around_delivery,
        });
    }

    fn notify_player(&mut self, player_id: i32, string_id: &'static str) {
        let delivery = send_country_war_script_notice(self.game, player_id, string_id.as_bytes());
        self.effects.push(CountryWarContendEffect::Notice {
            player_id,
            string_id,
            delivery,
        });
    }
}

pub(crate) fn run_country_war_action_script_function<Runtime: GameClockContext>(
    game: &mut CGame,
    runtime: &mut Runtime,
    script_player_id: Option<i32>,
    script_npc_id: Option<i32>,
    script_region_id: Option<i32>,
    function_id: i32,
    evaluated_arguments: [Option<i32>; 2],
) -> CountryWarActionScriptFunctionOutcome {
    if function_id == SCRIPT_FUNCTION_COUNTRY_WAR_VICTORY {
        let country = evaluated_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
        if country == SCRIPT_INT_PARAMETER_ERROR {
            return country_war_action_handled(
                function_id,
                CountryWarActionKind::PublishVictory,
                -1,
                CountryWarActionScriptDisposition::ArgumentMissing { argument: 0 },
            );
        }
        if country == 0 {
            return country_war_action_handled(
                function_id,
                CountryWarActionKind::PublishVictory,
                0,
                CountryWarActionScriptDisposition::VictoryNotPublished,
            );
        }
        let country = country as u8;
        let mut request = CMessage::new(0x0006_0318);
        request.base_mut().add_byte(country);
        let delivery = request.send(game, false);
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::PublishVictory,
            0,
            CountryWarActionScriptDisposition::VictoryPublished { country, delivery },
        );
    }
    if function_id != SCRIPT_FUNCTION_ENTER_COUNTRY_CONTEND {
        return CountryWarActionScriptFunctionOutcome::DifferentFunction;
    }

    let (Some(player_id), Some(npc_id)) = (script_player_id, script_npc_id) else {
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::CallerMissing,
        );
    };
    let player_shape = game.resolve_shape(ShapeIdentity {
        object_type: SCRIPT_PLAYER_TYPE,
        id: player_id,
        ex_id: CGuid::GUID_INVALID,
    });
    let npc_shape = game.resolve_shape(ShapeIdentity {
        object_type: SCRIPT_NPC_TYPE,
        id: npc_id,
        ex_id: CGuid::GUID_INVALID,
    });
    let (Some(player_shape), Some(npc_shape)) = (player_shape, npc_shape) else {
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::CallerShapeMissing,
        );
    };
    if !game.country_war_sys().state_war {
        let delivery = send_country_war_script_notice(game, player_id, b"GS0219");
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::WarClosed { delivery },
        );
    }
    let distance = npc_shape.distance(player_shape);
    if distance > 2 {
        let delivery = send_country_war_script_notice(game, player_id, b"GS0208");
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::TooFar {
                distance,
                maximum: 2,
                delivery,
            },
        );
    }
    let symbol_id = evaluated_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    if symbol_id == SCRIPT_INT_PARAMETER_ERROR {
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::ArgumentMissing { argument: 0 },
        );
    }
    let duration = evaluated_arguments[1].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    if duration == SCRIPT_INT_PARAMETER_ERROR {
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::ArgumentMissing { argument: 1 },
        );
    }
    let Some(region_id) = script_region_id else {
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::RegionMissing { region_id: None },
        );
    };
    let Some(owner) = game.take_region_owner(region_id) else {
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::RegionMissing {
                region_id: Some(region_id),
            },
        );
    };
    let ServerRegionOwner::Country(mut region) = owner else {
        game.restore_region_owner(owner);
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::RegionNotCountry { region_id },
        );
    };
    let Some(symbol_name) = region
        .base
        .find_npc_by_id(npc_id)
        .map(|npc| String::from_utf8_lossy(npc.name()).into_owned())
    else {
        game.restore_region_owner(ServerRegionOwner::Country(region));
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::NpcMissing { region_id, npc_id },
        );
    };
    let Some(player) = game
        .find_player(player_id)
        .map(|player| CountryContendPlayer {
            player_id,
            faction_id: player.faction_id(),
            country: player.country(),
            shape_type: player_shape.identity.object_type,
            is_dead: player.is_dead(),
        })
    else {
        game.restore_region_owner(ServerRegionOwner::Country(region));
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::CallerMissing,
        );
    };
    let duration_ms = duration.wrapping_mul(1_000);
    let region_projection = region.base.clone();
    let mut context = GameCountryContendEntryContext {
        game,
        runtime,
        region: region_projection,
        effects: Vec::new(),
    };
    let result = region.on_enter_contend(
        Some(&player),
        symbol_id,
        &symbol_name,
        duration_ms,
        &mut context,
    );
    let effects = std::mem::take(&mut context.effects);
    drop(context);
    game.restore_region_owner(ServerRegionOwner::Country(region));
    country_war_action_handled(
        function_id,
        CountryWarActionKind::EnterContend,
        0,
        CountryWarActionScriptDisposition::ContendInvoked {
            region_id,
            player_id,
            symbol_id,
            duration_ms,
            result,
            effects,
        },
    )
}

fn country_war_action_handled(
    function_id: i32,
    kind: CountryWarActionKind,
    legacy_return: i32,
    disposition: CountryWarActionScriptDisposition,
) -> CountryWarActionScriptFunctionOutcome {
    CountryWarActionScriptFunctionOutcome::Handled {
        function_id,
        kind,
        legacy_return,
        disposition,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NationWarScriptKind {
    SendPlayerId,
    CarriageBackTown,
    GetFlagStatus,
    GetTime,
    EnterContend,
    CountrySignUp,
    ClearPlayerTime,
    GetNationStatus,
    SetPlayerTime,
    IsPlayerWeak,
    GetMorale,
    ClearMorale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NationWarScriptDisposition {
    ArgumentMissing {
        argument: usize,
    },
    PlayerMissing,
    TimingStart {
        target_player_id: i32,
        started: bool,
    },
    CarriageBackTown {
        enter_debug: Vec<u8>,
        report: Option<NationCarriageReturnReport>,
        completed_debug: Option<Vec<u8>>,
    },
    Scalar {
        region_id: Option<i32>,
        player_id: Option<i32>,
        value: i32,
    },
    PlayerWarTime {
        player_id: i32,
        enter_debug: Vec<u8>,
        value_debug: Vec<u8>,
        value: i32,
    },
    Contend {
        player_id: i32,
        duration_ms: u32,
        report: Option<NationContendEnterReport>,
    },
    SignUpSkipped,
    SignUpRequested {
        country: i32,
        delivery: Result<i32, SendMessageError>,
    },
    PlayerTimeCleared {
        player_id: i32,
        previous_ms: Option<u32>,
    },
    PlayerTimeSet {
        player_id: i32,
        time_ms: u32,
        previous_ms: Option<u32>,
    },
    MoraleCleared {
        previous: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NationWarScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        function_id: i32,
        kind: NationWarScriptKind,
        legacy_return: i32,
        disposition: NationWarScriptDisposition,
    },
}

pub(crate) fn run_nation_war_script_function<Runtime: NationCombatContext>(
    game: &mut CGame,
    runtime: &mut Runtime,
    script_player_id: Option<i32>,
    function_id: i32,
    evaluated_arguments: [Option<i32>; 2],
) -> NationWarScriptFunctionOutcome {
    let kind = match function_id {
        SCRIPT_FUNCTION_NATION_WAR_SEND_PLAYER_ID => NationWarScriptKind::SendPlayerId,
        SCRIPT_FUNCTION_NATION_WAR_CARRIAGE_BACK_TOWN => NationWarScriptKind::CarriageBackTown,
        SCRIPT_FUNCTION_NATION_WAR_GET_FLAG_STATUS => NationWarScriptKind::GetFlagStatus,
        SCRIPT_FUNCTION_NATION_WAR_GET_TIME => NationWarScriptKind::GetTime,
        SCRIPT_FUNCTION_NATION_WAR_ENTER_CONTEND => NationWarScriptKind::EnterContend,
        SCRIPT_FUNCTION_NATION_WAR_COUNTRY_SIGN_UP => NationWarScriptKind::CountrySignUp,
        SCRIPT_FUNCTION_NATION_WAR_CLEAR_PLAYER_TIME => NationWarScriptKind::ClearPlayerTime,
        SCRIPT_FUNCTION_NATION_WAR_GET_NATION_STATUS => NationWarScriptKind::GetNationStatus,
        SCRIPT_FUNCTION_NATION_WAR_SET_PLAYER_TIME => NationWarScriptKind::SetPlayerTime,
        SCRIPT_FUNCTION_NATION_WAR_IS_PLAYER_WEAK => NationWarScriptKind::IsPlayerWeak,
        SCRIPT_FUNCTION_NATION_WAR_GET_MORALE => NationWarScriptKind::GetMorale,
        SCRIPT_FUNCTION_NATION_WAR_CLEAR_MORALE => NationWarScriptKind::ClearMorale,
        _ => return NationWarScriptFunctionOutcome::DifferentFunction,
    };
    let argument = |index: usize| {
        evaluated_arguments[index]
            .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            .ok_or(NationWarScriptDisposition::ArgumentMissing { argument: index })
    };

    match kind {
        NationWarScriptKind::SendPlayerId => {
            let target_player_id = match argument(0) {
                Ok(value) => value,
                Err(disposition) => {
                    return nation_war_script_handled(function_id, kind, 0, disposition);
                }
            };
            let Some(script_player_id) = script_player_id else {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::PlayerMissing,
                );
            };
            let started =
                game.script_nation_war_send_player_id(script_player_id, target_player_id, || {
                    runtime.now_milliseconds()
                });
            nation_war_script_handled(
                function_id,
                kind,
                0,
                NationWarScriptDisposition::TimingStart {
                    target_player_id,
                    started,
                },
            )
        }
        NationWarScriptKind::CarriageBackTown => {
            let enter_debug = nation_war_script_debug(game, b"GS1053");
            let Some((region_id, country)) = script_player_id.and_then(|player_id| {
                let player = game.find_player(player_id)?;
                Some((player.server_region_id()?, i32::from(player.country())))
            }) else {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::CarriageBackTown {
                        enter_debug,
                        report: None,
                        completed_debug: None,
                    },
                );
            };
            let report = game.script_nation_carriage_back_town(region_id, country);
            let completed_debug = report
                .is_some()
                .then(|| nation_war_script_debug(game, b"GS1054"));
            nation_war_script_handled(
                function_id,
                kind,
                0,
                NationWarScriptDisposition::CarriageBackTown {
                    enter_debug,
                    report,
                    completed_debug,
                },
            )
        }
        NationWarScriptKind::GetFlagStatus => {
            let region_id = nation_script_player_region_id(game, script_player_id);
            let value = region_id
                .and_then(|region_id| match game.find_region(region_id) {
                    Some(ServerRegionOwner::Nation(region)) => Some(region.flag_belong_to_id()),
                    _ => None,
                })
                .unwrap_or(0);
            nation_war_script_handled(
                function_id,
                kind,
                value,
                NationWarScriptDisposition::Scalar {
                    region_id,
                    player_id: script_player_id,
                    value,
                },
            )
        }
        NationWarScriptKind::GetTime => {
            let enter_debug = nation_war_script_debug(game, b"GS1055");
            let Some(player_id) = script_player_id else {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::PlayerMissing,
                );
            };
            let debug_time = game
                .four_nation_war_sys()
                .player_war_time_seconds(player_id) as i32;
            let Some(player_name) = game
                .find_player(player_id)
                .map(|player| player.player_name().to_vec())
            else {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::PlayerMissing,
                );
            };
            let value_debug = format_nation_war_time_debug(
                game.get_string_by_id(b"GS1056"),
                &player_name,
                debug_time,
            );
            put_debug_string(&value_debug);
            let value = game
                .four_nation_war_sys()
                .player_war_time_seconds(player_id) as i32;
            nation_war_script_handled(
                function_id,
                kind,
                value,
                NationWarScriptDisposition::PlayerWarTime {
                    player_id,
                    enter_debug,
                    value_debug,
                    value,
                },
            )
        }
        NationWarScriptKind::EnterContend => {
            let seconds = match argument(0) {
                Ok(value) => value,
                Err(disposition) => {
                    return nation_war_script_handled(function_id, kind, 0, disposition);
                }
            };
            let Some(player_id) = script_player_id else {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::PlayerMissing,
                );
            };
            if game.find_player(player_id).is_none() {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::PlayerMissing,
                );
            }
            let duration_ms = (seconds as u32).wrapping_mul(1_000);
            let report =
                nation_script_player_region_id(game, Some(player_id)).and_then(|region_id| {
                    game.nation_enter_contend(region_id, player_id, duration_ms, runtime)
                });
            nation_war_script_handled(
                function_id,
                kind,
                0,
                NationWarScriptDisposition::Contend {
                    player_id,
                    duration_ms,
                    report,
                },
            )
        }
        NationWarScriptKind::CountrySignUp => {
            let operation = match argument(0) {
                Ok(value) => value,
                Err(disposition) => {
                    return nation_war_script_handled(function_id, kind, 0, disposition);
                }
            };
            if operation == 0 {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::SignUpSkipped,
                );
            }
            let Some(country) = script_player_id
                .and_then(|player_id| game.find_player(player_id))
                .map(|player| i32::from(player.country()))
            else {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::PlayerMissing,
                );
            };
            let mut request = CMessage::new(0x0006_031b);
            request.base_mut().add_long(country);
            let delivery = request.send(game, false);
            let legacy_return = match delivery {
                Ok(value) => value,
                Err(_) => 0,
            };
            nation_war_script_handled(
                function_id,
                kind,
                legacy_return,
                NationWarScriptDisposition::SignUpRequested { country, delivery },
            )
        }
        NationWarScriptKind::ClearPlayerTime => {
            let player_id = match argument(0) {
                Ok(value) => value,
                Err(disposition) => {
                    return nation_war_script_handled(function_id, kind, 0, disposition);
                }
            };
            let previous_ms = game
                .four_nation_war_sys_mut()
                .clear_one_player_war_time(player_id);
            nation_war_script_handled(
                function_id,
                kind,
                0,
                NationWarScriptDisposition::PlayerTimeCleared {
                    player_id,
                    previous_ms,
                },
            )
        }
        NationWarScriptKind::GetNationStatus => {
            let region_id = nation_script_player_region_id(game, script_player_id);
            let country = script_player_id
                .and_then(|player_id| game.find_player(player_id))
                .map(|player| i32::from(player.country()));
            let value = match (region_id, country) {
                (Some(region_id), Some(country)) => match game.find_region(region_id) {
                    Some(ServerRegionOwner::Nation(region)) if region.is_nation_fail(country) => 1,
                    _ => 0,
                },
                _ => 0,
            };
            nation_war_script_handled(
                function_id,
                kind,
                value,
                NationWarScriptDisposition::Scalar {
                    region_id,
                    player_id: script_player_id,
                    value,
                },
            )
        }
        NationWarScriptKind::SetPlayerTime => {
            // Исходный диспетчер вычисляет оба выражения до любой проверки
            // граничного значения.
            let player_id = evaluated_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let time_ms = evaluated_arguments[1].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            if player_id == SCRIPT_INT_PARAMETER_ERROR {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::ArgumentMissing { argument: 0 },
                );
            }
            if time_ms == SCRIPT_INT_PARAMETER_ERROR {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::ArgumentMissing { argument: 1 },
                );
            }
            let time_ms = time_ms as u32;
            let previous_ms = game
                .four_nation_war_sys_mut()
                .set_one_player_war_time(player_id, time_ms);
            nation_war_script_handled(
                function_id,
                kind,
                0,
                NationWarScriptDisposition::PlayerTimeSet {
                    player_id,
                    time_ms,
                    previous_ms,
                },
            )
        }
        NationWarScriptKind::IsPlayerWeak => {
            let value = script_player_id
                .and_then(|player_id| game.find_player(player_id))
                .map(|player| i32::from(player.is_nation_war_player_weak()))
                .unwrap_or(0);
            nation_war_script_handled(
                function_id,
                kind,
                value,
                NationWarScriptDisposition::Scalar {
                    region_id: None,
                    player_id: script_player_id,
                    value,
                },
            )
        }
        NationWarScriptKind::GetMorale => {
            let value = game.four_nation_war_sys().morale();
            nation_war_script_handled(
                function_id,
                kind,
                value,
                NationWarScriptDisposition::Scalar {
                    region_id: None,
                    player_id: None,
                    value,
                },
            )
        }
        NationWarScriptKind::ClearMorale => {
            let previous = game.four_nation_war_sys_mut().clear_morale_value();
            nation_war_script_handled(
                function_id,
                kind,
                0,
                NationWarScriptDisposition::MoraleCleared { previous },
            )
        }
    }
}

fn nation_script_player_region_id(game: &CGame, player_id: Option<i32>) -> Option<i32> {
    player_id
        .and_then(|player_id| game.find_player(player_id))
        .and_then(|player| player.server_region_id())
}

fn nation_war_script_debug(game: &CGame, string_id: &[u8]) -> Vec<u8> {
    let text = game.get_string_by_id(string_id).to_vec();
    put_debug_string(&text);
    text
}

fn format_nation_war_time_debug(template: &[u8], player_name: &[u8], time: i32) -> Vec<u8> {
    enum Argument<'a> {
        Bytes(&'a [u8]),
        Signed(i32),
    }
    let arguments = [Argument::Bytes(player_name), Argument::Signed(time)];
    let template = template.split(|byte| *byte == 0).next().unwrap_or_default();
    let mut output = Vec::new();
    let mut argument = 0usize;
    let mut offset = 0usize;
    // Безопасная замена обнулённого `char[64] + _snprintf(..., 64, ...)`:
    // один байт остаётся для завершающего нуля, поэтому при полном усечении
    // не воспроизводится неопределённое поведение старого буфера без нуля.
    while offset < template.len() && output.len() < 63 {
        if template[offset] != b'%' {
            output.push(template[offset]);
            offset += 1;
            continue;
        }
        if template.get(offset + 1) == Some(&b'%') {
            output.push(b'%');
            offset += 2;
            continue;
        }
        let rendered = match (template.get(offset + 1).copied(), arguments.get(argument)) {
            (Some(b's'), Some(Argument::Bytes(value))) => Some(
                value
                    .split(|byte| *byte == 0)
                    .next()
                    .unwrap_or_default()
                    .to_vec(),
            ),
            (Some(b'd' | b'i'), Some(Argument::Signed(value))) => {
                Some(value.to_string().into_bytes())
            }
            _ => None,
        };
        let Some(rendered) = rendered else {
            output.push(b'%');
            offset += 1;
            continue;
        };
        let remaining = 63usize.saturating_sub(output.len());
        output.extend_from_slice(&rendered[..rendered.len().min(remaining)]);
        argument += 1;
        offset += 2;
    }
    output
}

fn nation_war_script_handled(
    function_id: i32,
    kind: NationWarScriptKind,
    legacy_return: i32,
    disposition: NationWarScriptDisposition,
) -> NationWarScriptFunctionOutcome {
    NationWarScriptFunctionOutcome::Handled {
        function_id,
        kind,
        legacy_return,
        disposition,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarQueryKind {
    DeclarationOpen,
    CountryDeclared,
    PreparationOpen,
    WarOpen,
    WinSymbol,
    Region,
    Camp,
    OtherCountry,
    Result,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarQueryScriptDisposition {
    CallerMissing,
    CallerShapeMissing,
    TooFar {
        distance: i32,
        maximum: i32,
    },
    ArgumentMissing {
        argument: usize,
    },
    RegionMissing {
        region_id: i32,
    },
    CountryMissing {
        country: u8,
    },
    Completed {
        kind: CountryWarQueryKind,
        country: Option<i32>,
        value: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarQueryScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        function_id: i32,
        legacy_return: i32,
        disposition: CountryWarQueryScriptDisposition,
    },
}

pub(crate) fn run_country_war_query_script_function(
    game: &CGame,
    script_player_id: Option<i32>,
    script_npc_id: Option<i32>,
    function_id: i32,
    evaluated_arguments: [Option<i32>; 3],
) -> CountryWarQueryScriptFunctionOutcome {
    let kind = match function_id {
        SCRIPT_FUNCTION_IS_COUNTRY_WAR_DECLARE => CountryWarQueryKind::DeclarationOpen,
        SCRIPT_FUNCTION_IS_COUNTRY_DECLARED => CountryWarQueryKind::CountryDeclared,
        SCRIPT_FUNCTION_IS_COUNTRY_WAR_PREPARE => CountryWarQueryKind::PreparationOpen,
        SCRIPT_FUNCTION_IS_COUNTRY_WAR => CountryWarQueryKind::WarOpen,
        SCRIPT_FUNCTION_IS_COUNTRY_WIN_SYMBOL => CountryWarQueryKind::WinSymbol,
        SCRIPT_FUNCTION_GET_COUNTRY_WAR_REGION => CountryWarQueryKind::Region,
        SCRIPT_FUNCTION_GET_COUNTRY_WAR_CAMP => CountryWarQueryKind::Camp,
        SCRIPT_FUNCTION_GET_OTHER_WAR_COUNTRY => CountryWarQueryKind::OtherCountry,
        SCRIPT_FUNCTION_GET_COUNTRY_WAR_RESULT => CountryWarQueryKind::Result,
        _ => return CountryWarQueryScriptFunctionOutcome::DifferentFunction,
    };
    if matches!(
        kind,
        CountryWarQueryKind::DeclarationOpen
            | CountryWarQueryKind::CountryDeclared
            | CountryWarQueryKind::PreparationOpen
            | CountryWarQueryKind::WarOpen
    ) {
        let gate = country_war_script_caller_gate(game, script_player_id, script_npc_id);
        if let Err(disposition) = gate {
            return country_war_query_handled(function_id, 0, disposition);
        }
    }
    let default_country = || {
        script_player_id
            .and_then(|player_id| game.find_player(player_id))
            .map(|player| i32::from(player.country()))
    };
    let optional_country = || {
        let raw = evaluated_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
        if raw == SCRIPT_INT_PARAMETER_ERROR {
            default_country()
        } else {
            Some(raw)
        }
    };
    match kind {
        CountryWarQueryKind::DeclarationOpen => country_war_query_completed(
            function_id,
            kind,
            None,
            i32::from(game.country_war_sys().state_declare),
        ),
        CountryWarQueryKind::CountryDeclared => {
            let Some(country) = optional_country() else {
                return country_war_query_handled(
                    function_id,
                    0,
                    CountryWarQueryScriptDisposition::CallerMissing,
                );
            };
            country_war_query_completed(
                function_id,
                kind,
                Some(country),
                i32::from(game.country_war_sys().is_already_declar(country)),
            )
        }
        CountryWarQueryKind::PreparationOpen => country_war_query_completed(
            function_id,
            kind,
            None,
            i32::from(game.country_war_sys().state_prepare),
        ),
        CountryWarQueryKind::WarOpen => country_war_query_completed(
            function_id,
            kind,
            None,
            i32::from(game.country_war_sys().state_war),
        ),
        CountryWarQueryKind::WinSymbol => {
            let mut values = [0; 3];
            for (index, argument) in evaluated_arguments.into_iter().enumerate() {
                let Some(value) = argument.filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                else {
                    return country_war_query_handled(
                        function_id,
                        0,
                        CountryWarQueryScriptDisposition::ArgumentMissing { argument: index },
                    );
                };
                values[index] = value;
            }
            let Some(ServerRegionOwner::Country(region)) = game.find_region(values[0]) else {
                return country_war_query_handled(
                    function_id,
                    0,
                    CountryWarQueryScriptDisposition::RegionMissing {
                        region_id: values[0],
                    },
                );
            };
            country_war_query_completed(
                function_id,
                kind,
                None,
                i32::from(region.is_win_symbol(values[1], values[2])),
            )
        }
        CountryWarQueryKind::Region
        | CountryWarQueryKind::Camp
        | CountryWarQueryKind::OtherCountry => {
            let Some(country) = optional_country() else {
                return country_war_query_handled(
                    function_id,
                    0,
                    CountryWarQueryScriptDisposition::CallerMissing,
                );
            };
            let value = match kind {
                CountryWarQueryKind::Region => {
                    game.country_war_sys().get_war_region_for_country(country)
                }
                CountryWarQueryKind::Camp => game.country_war_sys().get_war_camp(country),
                CountryWarQueryKind::OtherCountry => {
                    game.country_war_sys().get_other_country(country)
                }
                _ => unreachable!(),
            };
            country_war_query_completed(function_id, kind, Some(country), value)
        }
        CountryWarQueryKind::Result => {
            let Some(country) = optional_country() else {
                return country_war_query_handled(
                    function_id,
                    0,
                    CountryWarQueryScriptDisposition::CallerMissing,
                );
            };
            let country = country as u8;
            let Some(owner) = game.country_handler().country(country) else {
                return country_war_query_handled(
                    function_id,
                    0,
                    CountryWarQueryScriptDisposition::CountryMissing { country },
                );
            };
            country_war_query_completed(
                function_id,
                kind,
                Some(i32::from(country)),
                owner.country_war_result,
            )
        }
    }
}

fn country_war_script_caller_gate(
    game: &CGame,
    script_player_id: Option<i32>,
    script_npc_id: Option<i32>,
) -> Result<(i32, i32), CountryWarQueryScriptDisposition> {
    let (Some(player_id), Some(npc_id)) = (script_player_id, script_npc_id) else {
        return Err(CountryWarQueryScriptDisposition::CallerMissing);
    };
    let player_shape = game.resolve_shape(ShapeIdentity {
        object_type: SCRIPT_PLAYER_TYPE,
        id: player_id,
        ex_id: CGuid::GUID_INVALID,
    });
    let npc_shape = game.resolve_shape(ShapeIdentity {
        object_type: SCRIPT_NPC_TYPE,
        id: npc_id,
        ex_id: CGuid::GUID_INVALID,
    });
    let (Some(player_shape), Some(npc_shape)) = (player_shape, npc_shape) else {
        return Err(CountryWarQueryScriptDisposition::CallerShapeMissing);
    };
    let distance = npc_shape.distance(player_shape);
    let maximum = game
        .globe_setup()
        .area_width()
        .max(game.globe_setup().area_height())
        / 2;
    if maximum < distance {
        return Err(CountryWarQueryScriptDisposition::TooFar { distance, maximum });
    }
    Ok((distance, maximum))
}

/// `CScript::RunFunction` вызывает этот gate до вычисления выражений только
/// для тех village-menu selectors, где исходный owner сначала проверяет живой
/// разговор. Dispatcher повторяет проверку перед gameplay side effects.
pub(crate) fn village_war_script_caller_is_live(
    game: &CGame,
    script_player_id: Option<i32>,
    script_npc_id: Option<i32>,
) -> bool {
    country_war_script_caller_gate(game, script_player_id, script_npc_id).is_ok()
}

pub(crate) fn owned_region_script_caller_is_live(
    game: &CGame,
    script_player_id: Option<i32>,
    script_region_id: Option<i32>,
) -> bool {
    script_player_id.is_some_and(|player_id| game.find_player(player_id).is_some())
        && script_region_id.is_some_and(|region_id| game.find_region(region_id).is_some())
}

pub(crate) fn script_player_npc_caller_exists(
    game: &CGame,
    script_player_id: Option<i32>,
    script_npc_id: Option<i32>,
) -> bool {
    let (Some(player_id), Some(npc_id)) = (script_player_id, script_npc_id) else {
        return false;
    };
    game.resolve_shape(ShapeIdentity {
        object_type: SCRIPT_PLAYER_TYPE,
        id: player_id,
        ex_id: CGuid::GUID_INVALID,
    })
    .is_some()
        && game
            .resolve_shape(ShapeIdentity {
                object_type: SCRIPT_NPC_TYPE,
                id: npc_id,
                ex_id: CGuid::GUID_INVALID,
            })
            .is_some()
}

fn country_war_query_completed(
    function_id: i32,
    kind: CountryWarQueryKind,
    country: Option<i32>,
    value: i32,
) -> CountryWarQueryScriptFunctionOutcome {
    country_war_query_handled(
        function_id,
        value,
        CountryWarQueryScriptDisposition::Completed {
            kind,
            country,
            value,
        },
    )
}

fn country_war_query_handled(
    function_id: i32,
    legacy_return: i32,
    disposition: CountryWarQueryScriptDisposition,
) -> CountryWarQueryScriptFunctionOutcome {
    CountryWarQueryScriptFunctionOutcome::Handled {
        function_id,
        legacy_return,
        disposition,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarDeclarationScriptDisposition {
    CallerMissing,
    CallerShapeMissing,
    TooFar {
        distance: i32,
        maximum: i32,
    },
    DeclarationClosed {
        delivery: i32,
    },
    TargetCountryMissing,
    SameCountry {
        country: u8,
        delivery: i32,
    },
    KingRequired {
        country: u8,
        recorded_king_id: i32,
        delivery: i32,
    },
    AttackerAlreadyDeclared {
        country: u8,
        delivery: i32,
    },
    TargetAlreadyDeclared {
        country: i32,
        delivery: i32,
    },
    IdleRegionMissing {
        delivery: i32,
    },
    Requested {
        player_id: i32,
        target_country: i32,
        idle_region_id: i32,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarDeclarationScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        legacy_return: i32,
        disposition: CountryWarDeclarationScriptDisposition,
    },
}

pub(crate) fn run_country_war_declaration_script_function(
    game: &mut CGame,
    script_player_id: Option<i32>,
    script_npc_id: Option<i32>,
    function_id: i32,
    evaluated_target_country: Option<i32>,
) -> CountryWarDeclarationScriptFunctionOutcome {
    if function_id != SCRIPT_FUNCTION_DECLARE_COUNTRY_WAR {
        return CountryWarDeclarationScriptFunctionOutcome::DifferentFunction;
    }
    let (Some(player_id), Some(npc_id)) = (script_player_id, script_npc_id) else {
        return country_war_declaration_handled(
            1,
            CountryWarDeclarationScriptDisposition::CallerMissing,
        );
    };
    let player_shape = game.resolve_shape(ShapeIdentity {
        object_type: SCRIPT_PLAYER_TYPE,
        id: player_id,
        ex_id: CGuid::GUID_INVALID,
    });
    let npc_shape = game.resolve_shape(ShapeIdentity {
        object_type: SCRIPT_NPC_TYPE,
        id: npc_id,
        ex_id: CGuid::GUID_INVALID,
    });
    let (Some(player_shape), Some(npc_shape)) = (player_shape, npc_shape) else {
        return country_war_declaration_handled(
            1,
            CountryWarDeclarationScriptDisposition::CallerShapeMissing,
        );
    };
    let distance = npc_shape.distance(player_shape);
    let maximum = game
        .globe_setup()
        .area_width()
        .max(game.globe_setup().area_height())
        / 2;
    if maximum < distance {
        return country_war_declaration_handled(
            2,
            CountryWarDeclarationScriptDisposition::TooFar { distance, maximum },
        );
    }
    if !game.country_war_sys().state_declare {
        let delivery = send_country_war_script_notice(game, player_id, b"GS0213");
        return country_war_declaration_handled(
            3,
            CountryWarDeclarationScriptDisposition::DeclarationClosed { delivery },
        );
    }
    let target_country = evaluated_target_country.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    if target_country == SCRIPT_INT_PARAMETER_ERROR {
        return country_war_declaration_handled(
            3,
            CountryWarDeclarationScriptDisposition::TargetCountryMissing,
        );
    }
    let Some(player_country) = game.find_player(player_id).map(|player| player.country()) else {
        return country_war_declaration_handled(
            1,
            CountryWarDeclarationScriptDisposition::CallerMissing,
        );
    };
    if i32::from(player_country) == target_country {
        let delivery = send_country_war_script_notice(game, player_id, b"GS0214");
        return country_war_declaration_handled(
            3,
            CountryWarDeclarationScriptDisposition::SameCountry {
                country: player_country,
                delivery,
            },
        );
    }
    let recorded_king_id = game
        .country_handler_mut()
        .country_mut(player_country)
        .map(|country| country.country_information(1))
        .unwrap_or(0);
    if recorded_king_id != player_id {
        let delivery = send_country_war_script_notice(game, player_id, b"GS0215");
        return country_war_declaration_handled(
            4,
            CountryWarDeclarationScriptDisposition::KingRequired {
                country: player_country,
                recorded_king_id,
                delivery,
            },
        );
    }
    if game
        .country_war_sys()
        .is_already_declar(i32::from(player_country))
    {
        let delivery = send_country_war_script_notice(game, player_id, b"GS0216");
        return country_war_declaration_handled(
            4,
            CountryWarDeclarationScriptDisposition::AttackerAlreadyDeclared {
                country: player_country,
                delivery,
            },
        );
    }
    if game.country_war_sys().is_already_declar(target_country) {
        let delivery = send_country_war_script_notice(game, player_id, b"GS0217");
        return country_war_declaration_handled(
            4,
            CountryWarDeclarationScriptDisposition::TargetAlreadyDeclared {
                country: target_country,
                delivery,
            },
        );
    }
    let idle_region_id = game.country_war_sys().get_idle_war_region();
    if idle_region_id == 0 {
        let delivery = send_country_war_script_notice(game, player_id, b"GS0218");
        return country_war_declaration_handled(
            4,
            CountryWarDeclarationScriptDisposition::IdleRegionMissing { delivery },
        );
    }
    let mut request = CMessage::new(0x0006_0317);
    request.base_mut().add_long(player_id);
    request.base_mut().add_long(target_country);
    let delivery = request.send(game, false);
    country_war_declaration_handled(
        0,
        CountryWarDeclarationScriptDisposition::Requested {
            player_id,
            target_country,
            idle_region_id,
            delivery,
        },
    )
}

fn send_country_war_script_notice(game: &CGame, player_id: i32, string_id: &[u8]) -> i32 {
    colored_player_notice_message(0xffff_ffff, 0xffff_0000, game.get_string_by_id(string_id))
        .send_to_player(game.net_server(), player_id)
}

fn country_war_declaration_handled(
    legacy_return: i32,
    disposition: CountryWarDeclarationScriptDisposition,
) -> CountryWarDeclarationScriptFunctionOutcome {
    CountryWarDeclarationScriptFunctionOutcome::Handled {
        legacy_return,
        disposition,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryScalarQueryField {
    Power,
    TechnologyLevel,
    Treasury,
    Material,
    TechnologyExperience,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryScalarQueryDisposition {
    ScriptPlayerMissing,
    CountryMissing {
        country: u8,
    },
    Completed {
        country: u8,
        field: CountryScalarQueryField,
        value: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryScalarQueryScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        function_id: i32,
        legacy_return: i32,
        disposition: CountryScalarQueryDisposition,
    },
}

pub(crate) fn run_country_scalar_query_script_function(
    game: &CGame,
    script_player_id: Option<i32>,
    function_id: i32,
    evaluated_country: Option<i32>,
) -> CountryScalarQueryScriptFunctionOutcome {
    let field = match function_id {
        SCRIPT_FUNCTION_GET_COUNTRY_POWER => CountryScalarQueryField::Power,
        SCRIPT_FUNCTION_GET_COUNTRY_TECH_LEVEL => CountryScalarQueryField::TechnologyLevel,
        SCRIPT_FUNCTION_GET_COUNTRY_TREASURY => CountryScalarQueryField::Treasury,
        SCRIPT_FUNCTION_GET_COUNTRY_MATERIAL => CountryScalarQueryField::Material,
        SCRIPT_FUNCTION_GET_COUNTRY_TECH => CountryScalarQueryField::TechnologyExperience,
        _ => return CountryScalarQueryScriptFunctionOutcome::DifferentFunction,
    };
    let raw_country = evaluated_country.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    let country = if raw_country == SCRIPT_INT_PARAMETER_ERROR {
        let Some(player) = script_player_id.and_then(|player_id| game.find_player(player_id))
        else {
            return CountryScalarQueryScriptFunctionOutcome::Handled {
                function_id,
                legacy_return: -1,
                disposition: CountryScalarQueryDisposition::ScriptPlayerMissing,
            };
        };
        player.country()
    } else {
        raw_country as u8
    };
    let Some(country_owner) = game.country_handler().country(country) else {
        return CountryScalarQueryScriptFunctionOutcome::Handled {
            function_id,
            legacy_return: -1,
            disposition: CountryScalarQueryDisposition::CountryMissing { country },
        };
    };
    let value = match field {
        CountryScalarQueryField::Power => country_owner.power,
        CountryScalarQueryField::TechnologyLevel => country_owner.tech_level,
        CountryScalarQueryField::Treasury => country_owner.treasury,
        CountryScalarQueryField::Material => country_owner.material_point,
        CountryScalarQueryField::TechnologyExperience => country_owner.tech_current_exp,
    };
    CountryScalarQueryScriptFunctionOutcome::Handled {
        function_id,
        legacy_return: value,
        disposition: CountryScalarQueryDisposition::Completed {
            country,
            field,
            value,
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryIdentityScriptDisposition {
    ArgumentMissing {
        argument: usize,
    },
    ScriptPlayerMissing,
    TargetPlayerMissing {
        player_id: i32,
    },
    IdentityOutOfRange {
        identity: i32,
    },
    CountryMissing {
        country: u8,
    },
    CountryInformationRead {
        country: u8,
        identity: u8,
        player_id: i32,
    },
    AssignmentRequested {
        country: u8,
        identity: u8,
        player_id: i32,
        delivery: Result<i32, SendMessageError>,
    },
    KingIdRead {
        country: u8,
        king_id: i32,
    },
    TargetIdRejected {
        player_id: i32,
    },
    PlayerMissing {
        player_id: Option<i32>,
    },
    PlayerIdentityRead {
        player_id: i32,
        identity: u8,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryIdentityScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        function_id: i32,
        legacy_return: i32,
        disposition: CountryIdentityScriptDisposition,
    },
}

pub(crate) fn run_country_identity_script_function(
    game: &mut CGame,
    script_player_id: Option<i32>,
    function_id: i32,
    evaluated_first: Option<i32>,
    evaluated_second: Option<i32>,
) -> CountryIdentityScriptFunctionOutcome {
    if !matches!(
        function_id,
        SCRIPT_FUNCTION_GET_COUNTRY_CI
            | SCRIPT_FUNCTION_SET_COUNTRY_CI
            | SCRIPT_FUNCTION_GET_COUNTRY_KING_ID
            | SCRIPT_FUNCTION_GET_COUNTRY_OCCUPATION
            | SCRIPT_FUNCTION_GET_COUNTRY_IDENTITY
    ) {
        return CountryIdentityScriptFunctionOutcome::DifferentFunction;
    }
    if matches!(
        function_id,
        SCRIPT_FUNCTION_GET_COUNTRY_OCCUPATION | SCRIPT_FUNCTION_GET_COUNTRY_IDENTITY
    ) {
        let raw_player_id = evaluated_first.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
        let player_id = if raw_player_id == SCRIPT_INT_PARAMETER_ERROR {
            let Some(player_id) = script_player_id else {
                return country_identity_handled(
                    function_id,
                    -1,
                    CountryIdentityScriptDisposition::PlayerMissing { player_id: None },
                );
            };
            player_id
        } else if raw_player_id <= 0 {
            return country_identity_handled(
                function_id,
                -1,
                CountryIdentityScriptDisposition::TargetIdRejected {
                    player_id: raw_player_id,
                },
            );
        } else {
            raw_player_id
        };
        if game.find_player(player_id).is_none() {
            return country_identity_handled(
                function_id,
                -1,
                CountryIdentityScriptDisposition::PlayerMissing {
                    player_id: Some(player_id),
                },
            );
        }
        let identity = game.player_country_identity(player_id);
        return country_identity_handled(
            function_id,
            i32::from(identity),
            CountryIdentityScriptDisposition::PlayerIdentityRead {
                player_id,
                identity,
            },
        );
    }
    if function_id == SCRIPT_FUNCTION_SET_COUNTRY_CI {
        let identity = evaluated_first.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
        if identity == SCRIPT_INT_PARAMETER_ERROR {
            return country_identity_handled(
                function_id,
                -1,
                CountryIdentityScriptDisposition::ArgumentMissing { argument: 0 },
            );
        }
        let raw_target = evaluated_second.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
        let target_id = if raw_target == SCRIPT_INT_PARAMETER_ERROR {
            let Some(player_id) = script_player_id else {
                return country_identity_handled(
                    function_id,
                    -1,
                    CountryIdentityScriptDisposition::ScriptPlayerMissing,
                );
            };
            player_id
        } else {
            raw_target
        };
        let Some(target) = game.find_player(target_id) else {
            return country_identity_handled(
                function_id,
                -1,
                CountryIdentityScriptDisposition::TargetPlayerMissing {
                    player_id: target_id,
                },
            );
        };
        let country = target.country();
        if !(0..=8).contains(&identity) {
            return country_identity_handled(
                function_id,
                -1,
                CountryIdentityScriptDisposition::IdentityOutOfRange { identity },
            );
        }
        let mut request = CMessage::new(0x0006_0304);
        request.base_mut().add_byte(country);
        request.base_mut().add_long(target_id);
        request.base_mut().add_byte(identity as u8);
        return country_identity_handled(
            function_id,
            0,
            CountryIdentityScriptDisposition::AssignmentRequested {
                country,
                identity: identity as u8,
                player_id: target_id,
                delivery: request.send(game, false),
            },
        );
    }

    let raw_country = evaluated_first.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    let country_was_defaulted = raw_country == SCRIPT_INT_PARAMETER_ERROR;
    let country = if raw_country == SCRIPT_INT_PARAMETER_ERROR {
        let Some(player) = script_player_id.and_then(|player_id| game.find_player(player_id))
        else {
            return country_identity_handled(
                function_id,
                -1,
                CountryIdentityScriptDisposition::ScriptPlayerMissing,
            );
        };
        player.country()
    } else {
        raw_country as u8
    };
    if function_id == SCRIPT_FUNCTION_GET_COUNTRY_KING_ID {
        let Some(country_owner) = game.country_handler().country(country) else {
            return country_identity_handled(
                function_id,
                -1,
                CountryIdentityScriptDisposition::CountryMissing { country },
            );
        };
        let king_id = country_owner.king_id();
        return country_identity_handled(
            function_id,
            king_id,
            CountryIdentityScriptDisposition::KingIdRead { country, king_id },
        );
    }

    let identity = if country_was_defaulted {
        1
    } else {
        evaluated_second
            .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            .unwrap_or(1) as u8
    };
    let Some(country_owner) = game.country_handler_mut().country_mut(country) else {
        return country_identity_handled(
            function_id,
            -1,
            CountryIdentityScriptDisposition::CountryMissing { country },
        );
    };
    let player_id = country_owner.country_information(identity);
    country_identity_handled(
        function_id,
        player_id,
        CountryIdentityScriptDisposition::CountryInformationRead {
            country,
            identity,
            player_id,
        },
    )
}

fn country_identity_handled(
    function_id: i32,
    legacy_return: i32,
    disposition: CountryIdentityScriptDisposition,
) -> CountryIdentityScriptFunctionOutcome {
    CountryIdentityScriptFunctionOutcome::Handled {
        function_id,
        legacy_return,
        disposition,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryControlPointScriptDisposition {
    ArgumentMissing {
        argument: usize,
    },
    CountryMissing {
        country: u8,
    },
    Applied {
        country: u8,
        delta: i32,
        mutation: CountryScalarMutationReport,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryControlPointScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        legacy_return: i32,
        disposition: CountryControlPointScriptDisposition,
    },
}

pub(crate) fn run_country_control_point_script_function(
    game: &mut CGame,
    function_id: i32,
    evaluated_delta: Option<i32>,
    evaluated_country: Option<i32>,
) -> CountryControlPointScriptFunctionOutcome {
    if function_id != SCRIPT_FUNCTION_ADD_KING_POINT {
        return CountryControlPointScriptFunctionOutcome::DifferentFunction;
    }
    let delta = evaluated_delta.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    if delta == SCRIPT_INT_PARAMETER_ERROR {
        return CountryControlPointScriptFunctionOutcome::Handled {
            legacy_return: 0,
            disposition: CountryControlPointScriptDisposition::ArgumentMissing { argument: 0 },
        };
    }
    let raw_country = evaluated_country.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    if raw_country == SCRIPT_INT_PARAMETER_ERROR {
        return CountryControlPointScriptFunctionOutcome::Handled {
            legacy_return: 0,
            disposition: CountryControlPointScriptDisposition::ArgumentMissing { argument: 1 },
        };
    }
    let country = raw_country as u8;
    let Some(country_owner) = game.country_handler().country(country) else {
        return CountryControlPointScriptFunctionOutcome::Handled {
            legacy_return: 0,
            disposition: CountryControlPointScriptDisposition::CountryMissing { country },
        };
    };
    let applied = country_owner.control_point.wrapping_add(delta);
    let (mutation, message) = game
        .country_handler_mut()
        .country_mut(country)
        .expect("country owner жив до control-point mutation")
        .set_script_scalar(5, applied);
    let delivery = message.send(game, false);
    CountryControlPointScriptFunctionOutcome::Handled {
        legacy_return: 0,
        disposition: CountryControlPointScriptDisposition::Applied {
            country,
            delta,
            mutation,
            delivery,
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryExileTimeScriptDisposition {
    ScriptPlayerMissing,
    CountryMissing {
        country: u8,
    },
    ParameterUnavailable {
        field: &'static str,
        sampled_at_ms: u32,
    },
    Completed(CountryExileRestTimeReport),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryExileTimeScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        player_id: i32,
        legacy_return: i32,
        disposition: CountryExileTimeScriptDisposition,
    },
}

pub(crate) fn run_country_exile_time_script_function<Context: GameClockContext>(
    game: &CGame,
    script_player_id: Option<i32>,
    function_id: i32,
    evaluated_player_id: Option<i32>,
    context: &mut Context,
) -> CountryExileTimeScriptFunctionOutcome {
    if function_id != SCRIPT_FUNCTION_EXILE_TIME {
        return CountryExileTimeScriptFunctionOutcome::DifferentFunction;
    }
    let Some(script_player) = script_player_id.and_then(|player_id| game.find_player(player_id))
    else {
        return CountryExileTimeScriptFunctionOutcome::Handled {
            player_id: evaluated_player_id.unwrap_or(SCRIPT_INT_PARAMETER_ERROR),
            legacy_return: -1,
            disposition: CountryExileTimeScriptDisposition::ScriptPlayerMissing,
        };
    };
    let player_id = match evaluated_player_id {
        Some(value) if value != SCRIPT_INT_PARAMETER_ERROR => value,
        _ => script_player_id.expect("live script player имеет ID"),
    };
    let country = script_player.country();
    let Some(country_owner) = game.country_handler().country(country) else {
        return CountryExileTimeScriptFunctionOutcome::Handled {
            player_id,
            legacy_return: -1,
            disposition: CountryExileTimeScriptDisposition::CountryMissing { country },
        };
    };
    let sampled_at_ms = context.now_milliseconds();
    match country_owner.exile_rest_time(player_id, sampled_at_ms, game.country_param().exile_time())
    {
        Ok(report) => CountryExileTimeScriptFunctionOutcome::Handled {
            player_id,
            legacy_return: report.remaining_seconds,
            disposition: CountryExileTimeScriptDisposition::Completed(report),
        },
        Err(field) => CountryExileTimeScriptFunctionOutcome::Handled {
            player_id,
            legacy_return: -1,
            disposition: CountryExileTimeScriptDisposition::ParameterUnavailable {
                field,
                sampled_at_ms,
            },
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryQuestSwitchScriptDisposition {
    IdentityMissing,
    PlayerMissing,
    CountryMissing {
        country: u8,
    },
    Read {
        country: u8,
        enabled: bool,
    },
    Written {
        country: u8,
        raw_switch: i32,
        mutation: crate::gameserver::appserver::country::country::CountryQuestSwitchMutationReport,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryQuestSwitchScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        function_id: i32,
        identity: i32,
        legacy_return: i32,
        disposition: CountryQuestSwitchScriptDisposition,
    },
}

pub(crate) fn run_country_quest_switch_script_function(
    game: &mut CGame,
    script_player_id: Option<i32>,
    function_id: i32,
    evaluated_identity: Option<i32>,
    evaluated_switch: Option<i32>,
    evaluated_country: Option<i32>,
) -> CountryQuestSwitchScriptFunctionOutcome {
    if !matches!(
        function_id,
        SCRIPT_FUNCTION_GET_QUEST_SWITCH | SCRIPT_FUNCTION_SET_QUEST_SWITCH
    ) {
        return CountryQuestSwitchScriptFunctionOutcome::DifferentFunction;
    }
    let identity = evaluated_identity.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    if identity == SCRIPT_INT_PARAMETER_ERROR {
        return CountryQuestSwitchScriptFunctionOutcome::Handled {
            function_id,
            identity,
            legacy_return: -1,
            disposition: CountryQuestSwitchScriptDisposition::IdentityMissing,
        };
    }
    let raw_switch = evaluated_switch.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    let raw_country = evaluated_country.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    let needs_player_country = if function_id == SCRIPT_FUNCTION_GET_QUEST_SWITCH {
        raw_country == SCRIPT_INT_PARAMETER_ERROR
    } else {
        raw_switch == SCRIPT_INT_PARAMETER_ERROR || raw_country == SCRIPT_INT_PARAMETER_ERROR
    };
    let country = if needs_player_country {
        let Some(player) = script_player_id.and_then(|player_id| game.find_player(player_id))
        else {
            return CountryQuestSwitchScriptFunctionOutcome::Handled {
                function_id,
                identity,
                legacy_return: -1,
                disposition: CountryQuestSwitchScriptDisposition::PlayerMissing,
            };
        };
        player.country()
    } else {
        raw_country as u8
    };
    let Some(country_owner) = game.country_handler().country(country) else {
        return CountryQuestSwitchScriptFunctionOutcome::Handled {
            function_id,
            identity,
            legacy_return: -1,
            disposition: CountryQuestSwitchScriptDisposition::CountryMissing { country },
        };
    };
    if function_id == SCRIPT_FUNCTION_GET_QUEST_SWITCH {
        let enabled = country_owner.quest_switch(identity as u8);
        return CountryQuestSwitchScriptFunctionOutcome::Handled {
            function_id,
            identity,
            legacy_return: i32::from(enabled),
            disposition: CountryQuestSwitchScriptDisposition::Read { country, enabled },
        };
    }

    let message = country_owner.quest_switch_message(identity as u8, true);
    let delivery = message.send(game, false);
    let mutation = game
        .country_handler_mut()
        .country_mut(country)
        .expect("country owner жив после quest-switch World enqueue")
        .apply_quest_switch(identity as u8, true);
    CountryQuestSwitchScriptFunctionOutcome::Handled {
        function_id,
        identity,
        legacy_return: if identity as u8 == 0 { -1 } else { identity },
        disposition: CountryQuestSwitchScriptDisposition::Written {
            country,
            raw_switch,
            mutation,
            delivery,
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryScalarScriptDisposition {
    ValueMissing,
    CountryMissing {
        country: u8,
    },
    ParameterUnavailable {
        field: &'static str,
    },
    TechnologyLevelMissing {
        level: i32,
    },
    Applied {
        country: u8,
        requested: i32,
        applied: i32,
        mutation: CountryScalarMutationReport,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryScalarScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        function_id: i32,
        legacy_return: i32,
        disposition: CountryScalarScriptDisposition,
    },
}

pub(crate) fn run_country_scalar_script_function(
    game: &mut CGame,
    function_id: i32,
    evaluated_country: Option<i32>,
    evaluated_value: Option<i32>,
) -> CountryScalarScriptFunctionOutcome {
    let (selector, default_return) = match function_id {
        SCRIPT_FUNCTION_SET_COUNTRY_POWER => (2, -1),
        SCRIPT_FUNCTION_SET_COUNTRY_TECH_LEVEL => (4, -1),
        SCRIPT_FUNCTION_SET_COUNTRY_TREASURY => (1, 0),
        SCRIPT_FUNCTION_SET_COUNTRY_MATERIAL => (6, -1),
        SCRIPT_FUNCTION_SET_COUNTRY_TECH => (3, -1),
        _ => return CountryScalarScriptFunctionOutcome::DifferentFunction,
    };
    let country = evaluated_country.unwrap_or(SCRIPT_INT_PARAMETER_ERROR) as u8;
    let Some(requested) = evaluated_value.filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
    else {
        return CountryScalarScriptFunctionOutcome::Handled {
            function_id,
            legacy_return: default_return,
            disposition: CountryScalarScriptDisposition::ValueMissing,
        };
    };
    if game.country_handler().country(country).is_none() {
        return CountryScalarScriptFunctionOutcome::Handled {
            function_id,
            legacy_return: default_return,
            disposition: CountryScalarScriptDisposition::CountryMissing { country },
        };
    }

    let applied = match function_id {
        SCRIPT_FUNCTION_SET_COUNTRY_POWER => {
            let Some(maximum) = game.country_param().max_country_power() else {
                return scalar_parameter_unavailable(
                    function_id,
                    default_return,
                    "_max_country_power",
                );
            };
            if requested < 0 {
                0
            } else {
                requested.min(maximum)
            }
        }
        SCRIPT_FUNCTION_SET_COUNTRY_TECH_LEVEL => {
            if requested <= 0 {
                1
            } else {
                requested.min(game.country_param().country_tech_level_count())
            }
        }
        SCRIPT_FUNCTION_SET_COUNTRY_TREASURY => {
            let Some(maximum) = game.country_param().max_country_treasury() else {
                return scalar_parameter_unavailable(
                    function_id,
                    default_return,
                    "_max_country_treasury",
                );
            };
            if requested < 0 {
                0
            } else {
                requested.min(maximum)
            }
        }
        SCRIPT_FUNCTION_SET_COUNTRY_MATERIAL => {
            let Some(maximum) = game.country_param().max_king_material_point() else {
                return scalar_parameter_unavailable(
                    function_id,
                    default_return,
                    "_max_king_material_point",
                );
            };
            if requested < 0 {
                0
            } else {
                requested.min(maximum)
            }
        }
        SCRIPT_FUNCTION_SET_COUNTRY_TECH => {
            let next_level = game
                .country_handler()
                .country(country)
                .expect("country owner проверен перед tech-level lookup")
                .tech_level
                .wrapping_add(1);
            let Some(level) = game.country_param().country_tech_level(next_level) else {
                return CountryScalarScriptFunctionOutcome::Handled {
                    function_id,
                    legacy_return: default_return,
                    disposition: CountryScalarScriptDisposition::TechnologyLevelMissing {
                        level: next_level,
                    },
                };
            };
            if requested > level.country_tech_exp {
                level.country_tech_exp
            } else if requested < 0 {
                0
            } else {
                requested
            }
        }
        _ => unreachable!("country scalar function ID проверен перед clamp"),
    };
    let (mutation, message) = game
        .country_handler_mut()
        .country_mut(country)
        .expect("country owner жив до scalar mutation")
        .set_script_scalar(selector, applied);
    let delivery = message.send(game, false);
    CountryScalarScriptFunctionOutcome::Handled {
        function_id,
        legacy_return: applied,
        disposition: CountryScalarScriptDisposition::Applied {
            country,
            requested,
            applied,
            mutation,
            delivery,
        },
    }
}

fn scalar_parameter_unavailable(
    function_id: i32,
    legacy_return: i32,
    field: &'static str,
) -> CountryScalarScriptFunctionOutcome {
    CountryScalarScriptFunctionOutcome::Handled {
        function_id,
        legacy_return,
        disposition: CountryScalarScriptDisposition::ParameterUnavailable { field },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentSessionScriptFunctionOutcome {
    DifferentFunction,
    Opened(EquipmentSessionOpenReport),
}

pub(crate) fn run_equipment_session_script_function(
    game: &mut CGame,
    player_id: i32,
    function_id: i32,
) -> EquipmentSessionScriptFunctionOutcome {
    let kind = match function_id {
        SCRIPT_FUNCTION_OPEN_DA_KONG => EquipmentSessionPlugKind::DaKong,
        SCRIPT_FUNCTION_OPEN_EQUIPMENT_COMPOSE => EquipmentSessionPlugKind::Compose,
        SCRIPT_FUNCTION_OPEN_EQUIPMENT_UPGRADE => EquipmentSessionPlugKind::Upgrade,
        _ => return EquipmentSessionScriptFunctionOutcome::DifferentFunction,
    };
    EquipmentSessionScriptFunctionOutcome::Opened(game.open_equipment_session(player_id, kind))
}

#[must_use = "script dispatch отличает чужой ID от handled no-op и выполненного gameplay"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentDaKongScriptFunctionOutcome {
    DifferentFunction,
    HandledWithoutCall,
    Refreshed(EquipmentDaKongExternalRefreshReport),
}

pub(crate) fn run_equipment_da_kong_script_function<Context: EquipmentDaKongContext>(
    game: &mut CGame,
    player_id: i32,
    function_id: i32,
    evaluated_first_string: Option<&[u8]>,
    context: &mut Context,
) -> EquipmentDaKongScriptFunctionOutcome {
    if function_id != SCRIPT_FUNCTION_REFLUSH_EXTERN_PROPERTY {
        return EquipmentDaKongScriptFunctionOutcome::DifferentFunction;
    }
    let Some(cost_original_name) = evaluated_first_string.filter(|value| !value.is_empty()) else {
        return EquipmentDaKongScriptFunctionOutcome::HandledWithoutCall;
    };
    EquipmentDaKongScriptFunctionOutcome::Refreshed(
        game.reflush_equipment_da_kong_external_property(player_id, cost_original_name, context),
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScriptFunctionDispatchOutcome {
    DifferentFunction,
    Invalid,
    Handled { legacy_return: i32 },
    Yielded { legacy_return: i32 },
    Terminated { legacy_return: i32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ScriptStringFunctionDispatchOutcome {
    DifferentFunction,
    Invalid,
    Handled(Vec<u8>),
}

/// Строковая половина `CScript::RunFunction`. Исторические `GetName` и
/// `GetStringByID` возвращали заимствованный `char *`; среда исполнения Rust
/// копирует те же байты до следующего шага вычисления, не превращая адрес в
/// число.
pub(crate) fn dispatch_script_string_function(
    game: &CGame,
    script_player_id: Option<i32>,
    died_monster_index: Option<u32>,
    function_id: i32,
    evaluated_integer: Option<i32>,
    evaluated_string: Option<&[u8]>,
) -> ScriptStringFunctionDispatchOutcome {
    if function_id == SCRIPT_FUNCTION_GET_STRING_BY_ID {
        return evaluated_string.map_or(ScriptStringFunctionDispatchOutcome::Invalid, |id| {
            ScriptStringFunctionDispatchOutcome::Handled(game.get_string_by_id(id).to_vec())
        });
    }
    if function_id == SCRIPT_FUNCTION_GET_GOODS_ORIGINAL_NAME {
        let value = script_player_id
            .and_then(|player_id| script_selected_goods(game, player_id))
            .and_then(|goods| {
                game.goods_factory()
                    .query_goods_original_name(goods.base_properties_index())
            })
            .unwrap_or_default();
        return ScriptStringFunctionDispatchOutcome::Handled(value.to_vec());
    }
    if function_id == SCRIPT_FUNCTION_GET_DIED_MONSTER_ORIGINAL_NAME {
        let value = died_monster_index
            .filter(|index| *index != 0)
            .and_then(|index| game.find_monster_property_by_origin_index(index))
            .map_or(&[][..], |properties| properties.original_name.as_slice());
        return ScriptStringFunctionDispatchOutcome::Handled(value.to_vec());
    }
    if function_id == SCRIPT_FUNCTION_GET_TEAMER_NAME {
        let position = evaluated_integer.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
        let value = script_player_id
            .filter(|_| position != SCRIPT_INT_PARAMETER_ERROR)
            .map(|player_id| game.script_team_member_name(player_id, position))
            .unwrap_or_default();
        return ScriptStringFunctionDispatchOutcome::Handled(value);
    }
    if function_id == SCRIPT_FUNCTION_GET_NAME {
        let requested = evaluated_integer.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
        let player_id = if requested == SCRIPT_INT_PARAMETER_ERROR {
            let Some(player_id) = script_player_id else {
                return ScriptStringFunctionDispatchOutcome::Invalid;
            };
            player_id
        } else {
            requested
        };
        return game.find_player(player_id).map_or(
            ScriptStringFunctionDispatchOutcome::Invalid,
            |player| {
                ScriptStringFunctionDispatchOutcome::Handled(
                    player.shape().base_object().get_name().to_vec(),
                )
            },
        );
    }
    ScriptStringFunctionDispatchOutcome::DifferentFunction
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScriptFunctionParameterKind {
    Integer,
    String,
    Unused,
}

/// Маршрутизация `GetStringParam`/`GetIntParam` исторического плотного
/// диспетчера. `CScript` использует таблицу до вычисления выражения и сохраняет
/// позиционные строковые аргументы вместо прежнего особого случая только для
/// `9351`.
pub(crate) fn script_function_parameter_kind(
    function_id: i32,
    index: usize,
) -> ScriptFunctionParameterKind {
    use ScriptFunctionParameterKind::{Integer, String, Unused};
    match function_id {
        SCRIPT_FUNCTION_GET_STRING_BY_ID | SCRIPT_FUNCTION_GET_ME => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CHANGE_ME | SCRIPT_FUNCTION_SET_ME => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_ENERGY | SCRIPT_FUNCTION_SET_MAXIMUM_ENERGY => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CHECK_LEVEL
        | SCRIPT_FUNCTION_GET_ENERGY
        | SCRIPT_FUNCTION_GET_MAXIMUM_ENERGY => Unused,
        SCRIPT_FUNCTION_OPEN_NPC_SHOP => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_OPEN_DEPOT => Unused,
        SCRIPT_FUNCTION_GET_NAME | SCRIPT_FUNCTION_GET_TEAMER_NAME => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_TEAM_NUM | SCRIPT_FUNCTION_IS_TEAM_CAPTAIN => Unused,
        SCRIPT_FUNCTION_IS_TEAMMATES_AROUND_ME => match index {
            0 | 1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_REGION_FOR_TEAM | SCRIPT_FUNCTION_SET_TEAM_REGION => match index {
            0..=5 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_FU_MO_PROPERTY => match index {
            0 | 1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_PLAYER_TALK => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_RE_LIVE => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_STATES_NUMBER => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_STATE => match index {
            0..=2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_PLAYER_LEVEL => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CHANGE_PLAYER | SCRIPT_FUNCTION_SET_PLAYER => match index {
            0 | 1 => String,
            2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_PLAYER => match index {
            0 | 1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_IS_PLAYER_ONLINE | SCRIPT_FUNCTION_GET_PLAYER_ID => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_REGION_ID => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_PLAYER_REGION => match index {
            0 => String,
            1..=5 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_PLAYER_REGION_EX => match index {
            0 => String,
            1..=3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_KICK_PLAYER_EX => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_PLAYER_ALL_PROPERTIES => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_FORCE_MOVE => match index {
            0 => String,
            1..=3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CHANGE_MONEY_BY_NAME | SCRIPT_FUNCTION_SET_MONEY_BY_NAME => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_MONEY_BY_NAME
        | SCRIPT_FUNCTION_GET_FACTION_ID_BY_PLAYER_NAME
        | SCRIPT_FUNCTION_IS_FACTION_MASTER_BY_PLAYER_NAME => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CHANGE_MONEY_BY_ID | SCRIPT_FUNCTION_SET_MONEY_BY_ID => match index {
            0..=1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_MONEY_BY_ID => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_PLAYER_ALL_VARIABLES => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_KICK_PLAYER => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_BAN_PLAYER | SCRIPT_FUNCTION_SILENCE_PLAYER => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CREATE_FACTION => match index {
            0 | 2 | 3 => Integer,
            1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_APPLY_JOIN_FACTION => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_OPERATOR_CITY_GATE | SCRIPT_FUNCTION_OPERATE_CITY_GATE => match index {
            0..=2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_IS_ARRIVE_VILLAGE_APPLY_TIME
        | SCRIPT_FUNCTION_IS_ARRIVE_VILLAGE_WAR_TIME
        | SCRIPT_FUNCTION_IS_CITY_WAR_DECLARE_TIME
        | SCRIPT_FUNCTION_IS_CITY_WAR_FIGHT_TIME
        | SCRIPT_FUNCTION_GET_OWNED_REGION_FACTION_ID
        | SCRIPT_FUNCTION_GET_WAR_REGION_STATE
        | SCRIPT_FUNCTION_GET_COUNTRY_OWNING_REGION
        | SCRIPT_FUNCTION_GET_WAR_START_TIME => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_APPLY_FOR_VILLAGE_WAR | SCRIPT_FUNCTION_CITY_WAR_DECLARE => match index {
            0..=1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ENTER_CONTEND_STATE => match index {
            0 | 1 => Integer,
            2..=5 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_CITY_GATE_STATE => match index {
            0..=1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DELETE_SKILL => match index {
            0 | 1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_SKILL_LEVEL => match index {
            0 | 1 => String,
            2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_SKILL => match index {
            0 | 1 => String,
            2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_SKILL_LEVEL => match index {
            0 | 1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CHANGE_REGION => match index {
            0..=6 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SCRIPT_IS_RUNNING | SCRIPT_FUNCTION_REMOVE_SCRIPT => match index {
            0 => Integer,
            1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_YEAR
        | SCRIPT_FUNCTION_MONTH
        | SCRIPT_FUNCTION_DAY
        | SCRIPT_FUNCTION_HOUR
        | SCRIPT_FUNCTION_MINUTE
        | SCRIPT_FUNCTION_DAY_OF_WEEK => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_HOUR_DIFF | SCRIPT_FUNCTION_MINUTE_DIFF => match index {
            0 | 1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_REGION_RANDOM_POSITION => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_CHARGED => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_CHANGE_BODY_STATE
        | SCRIPT_FUNCTION_DELETE_CHANGE_BODY_STATE
        | SCRIPT_FUNCTION_GET_CHANGE_BODY_STATE => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_EX_STATE
        | SCRIPT_FUNCTION_DELETE_EX_STATE
        | SCRIPT_FUNCTION_GET_EX_STATE
        | SCRIPT_FUNCTION_ADD_EX_STATE_NEW
        | SCRIPT_FUNCTION_DELETE_EX_STATE_NEW
        | SCRIPT_FUNCTION_GET_EX_STATE_NEW
        | SCRIPT_FUNCTION_ADD_UNDEAD_STATE
        | SCRIPT_FUNCTION_DELETE_UNDEAD_STATE
        | SCRIPT_FUNCTION_GET_UNDEAD_STATE => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_HOTKEY => match index {
            0..=2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_WALK_STEP
        | SCRIPT_FUNCTION_RUN_STEP
        | SCRIPT_FUNCTION_SET_PLAYER_DIRECTION => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_PLAYER_POSITION => match index {
            0 | 1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_EQUIPMENT_ID_BY_POSITION => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_UPGRADE_EQUIPMENT => match index {
            0 => String,
            1 | 2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_IS_CHARGED
        | SCRIPT_FUNCTION_CHANGE_BODY_CHECK
        | SCRIPT_FUNCTION_CHECK_MODE
        | SCRIPT_FUNCTION_GET_PROGRESS
        | SCRIPT_FUNCTION_IS_COMBAT_STATE
        | SCRIPT_FUNCTION_IS_RIDER => Unused,
        SCRIPT_FUNCTION_OPEN_PLAYER_UI | SCRIPT_FUNCTION_CALL_MONSTER => Unused,
        SCRIPT_FUNCTION_CREATE_NPC => match index {
            0 | 8 => String,
            1..=7 | 9..=11 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_NPC_TALK | SCRIPT_FUNCTION_MONSTER_TALK => match index {
            0..=1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ATTACK_PLAYER => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_MOVE_PLAYER => match index {
            0..=9 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CREATE_MONSTER => match index {
            0 | 6 => String,
            1..=5 | 7 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DELETE_NPC
        | SCRIPT_FUNCTION_DELETE_MONSTER
        | SCRIPT_FUNCTION_KILL_MONSTER => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_PLAYER_MESSAGE => match index {
            0..=1 => String,
            2..=3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DELETE_MONSTER_RECT => match index {
            0..=4 => Integer,
            5 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DROP_GOODS => match index {
            0 | 1 | 3 => Integer,
            2 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_AUTO_MOVE => match index {
            0 | 1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DELETE_NPC_BY_NAME => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_MAP_INFO => match index {
            0..=1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_MONSTER_REFRESH_TIME => match index {
            0..=1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_REFRESH_BLOCK => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_COPY_NUMBER => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_LEVEL_EXPERIENCE => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_TIME
        | SCRIPT_FUNCTION_SECOND
        | SCRIPT_FUNCTION_GET_COUNTRY
        | SCRIPT_FUNCTION_LIST_ONLINE_GM
        | SCRIPT_FUNCTION_LIST_SILENCE_PLAYER
        | SCRIPT_FUNCTION_SAVE_ALL_PLAYERS
        | SCRIPT_FUNCTION_GET_ONLINE_PLAYERS
        | SCRIPT_FUNCTION_LIST_ONLINE_PLAYER
        | SCRIPT_FUNCTION_LIST_BANNED_PLAYER
        | SCRIPT_FUNCTION_KICK_ALL
        | SCRIPT_FUNCTION_GET_MAXIMUM_LEVEL
        | SCRIPT_FUNCTION_GET_AREA_ID
        | SCRIPT_FUNCTION_GET_AREA_TYPE
        | SCRIPT_FUNCTION_GET_WORLD_SERVER_ID
        | SCRIPT_FUNCTION_GET_PLAYER_SZL
        | SCRIPT_FUNCTION_OPEN_CHANGE_PLAYER_NAME => Unused,
        SCRIPT_FUNCTION_KICK_MAP => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_INVISIBLE
        | SCRIPT_FUNCTION_GOD_MODE
        | SCRIPT_FUNCTION_RESIDENT_MODE
        | SCRIPT_FUNCTION_GM_MODE => Unused,
        SCRIPT_FUNCTION_CHANGE_PLAYER_SZL => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_PLAY_EFFECT => match index {
            0..=2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_WEATHER | SCRIPT_FUNCTION_PLAY_ACTION => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_PLAY_SOUND => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_OPEN_CI_QING_PAGE => Unused,
        SCRIPT_FUNCTION_PUSH_ITEM_TO_CI_QING => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_REQUEST_PLAYER_RANKS => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_DAYS_HONOR_RANK
        | SCRIPT_FUNCTION_GET_WEEKS_HONOR_RANK
        | SCRIPT_FUNCTION_GET_MONTHS_HONOR_RANK
        | SCRIPT_FUNCTION_GET_TOTAL_HONOR_RANK
        | SCRIPT_FUNCTION_GET_ATTEMPT_APPELLATION_ID
        | SCRIPT_FUNCTION_SEND_TOTAL_HONOR_RANKS
        | SCRIPT_FUNCTION_GET_PLAYER_RANK
        | SCRIPT_FUNCTION_GET_JING_LI_DAN => Unused,
        SCRIPT_FUNCTION_ADD_APPELLATION_STATE
        | SCRIPT_FUNCTION_DEL_APPELLATION_STATE
        | SCRIPT_FUNCTION_GET_APPELLATION_STATE => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_THING_COUNT => match index {
            0 | 1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_THING_COUNT => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DELETE_EX_STATE_BY_TYPE | SCRIPT_FUNCTION_SET_JING_LI_DAN => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_JING_JIE_BUFF => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_RELOAD => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_POST_PLAYER_INFO
        | SCRIPT_FUNCTION_POST_REGION_INFO
        | SCRIPT_FUNCTION_POST_WORLD_INFO => match index {
            0 => String,
            1 | 2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_POST_COUNTRY_INFO => match index {
            0 => String,
            1..=3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_INCREMENT_LOG => match index {
            0 | 3 => String,
            1 | 2 | 4 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_LOG => match index {
            0 => Integer,
            1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DRAW_AWARDS => match index {
            0..=3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_GOODS
        | SCRIPT_FUNCTION_DELETE_GOODS
        | SCRIPT_FUNCTION_CHECK_GOODS
        | SCRIPT_FUNCTION_ADD_INFO
        | SCRIPT_FUNCTION_TALK_BOX
        | SCRIPT_FUNCTION_HELP
        | SCRIPT_FUNCTION_TALK_BOX_SMALL
        | SCRIPT_FUNCTION_ADD_GOODS_LOG => match index {
            0 => String,
            1..=3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GAME_MESSAGE => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_GEM_EXCHANGE_LOG => match index {
            0..=1 => String,
            2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_JEWELRY_MADE_LOG => match index {
            0..=2 => String,
            3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_GOODS_PROPERTY => match index {
            0 => String,
            1..=2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_RANDOM => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_RGB | SCRIPT_FUNCTION_CHECK_SPACE => match index {
            0..=2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_FREE_SPACE | SCRIPT_FUNCTION_UPGRADE_SELECTED_EQUIPMENT => {
            match index {
                0 => Integer,
                _ => Unused,
            }
        }
        SCRIPT_FUNCTION_ADD_DEPOT_GOODS | SCRIPT_FUNCTION_DELETE_DEPOT_GOODS => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CHECK_DEPOT_GOODS | SCRIPT_FUNCTION_CHECK_DEPOT_SPACE => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DELETE_PLAYER_GOODS => match index {
            0..=1 => String,
            2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_OPEN_GOODS_CONTAINER => match index {
            0..=1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_GOODS_NUMBER
        | SCRIPT_FUNCTION_GET_DEPOT_GOODS_NUMBER
        | SCRIPT_FUNCTION_GET_DEPOT_GOODS_FREE
        | SCRIPT_FUNCTION_GET_CONTAINER_ITEM_TYPE => Unused,
        SCRIPT_FUNCTION_OPEN_NEW_HELP_WINDOW => Unused,
        SCRIPT_FUNCTION_CHANGE_COUNTRY | SCRIPT_FUNCTION_SET_CONTRIBUTION => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_CONTRIBUTION | SCRIPT_FUNCTION_GET_YUAN_BAO => Unused,
        SCRIPT_FUNCTION_GET_GOODS_PROPERTY_1 | SCRIPT_FUNCTION_GET_GOODS_PROPERTY_2 => {
            match index {
                0 => Integer,
                _ => Unused,
            }
        }
        SCRIPT_FUNCTION_SET_GOODS_PROPERTY_1 | SCRIPT_FUNCTION_SET_GOODS_PROPERTY_2 => {
            match index {
                0..=1 => Integer,
                _ => Unused,
            }
        }
        SCRIPT_FUNCTION_DELETE_SPLIT_GOODS
        | SCRIPT_FUNCTION_GET_GOODS_ORIGINAL_NAME
        | SCRIPT_FUNCTION_GET_GOODS_PRICE
        | SCRIPT_FUNCTION_RECREATE_GOODS_ADDON_PROPERTIES
        | SCRIPT_FUNCTION_GET_DIED_MONSTER_INDEX
        | SCRIPT_FUNCTION_GET_DIED_MONSTER_ORIGINAL_NAME
        | SCRIPT_FUNCTION_GET_DIED_MONSTER_LEVEL => Unused,
        SCRIPT_FUNCTION_ADD_QUEST
        | SCRIPT_FUNCTION_COMPLETE_QUEST
        | SCRIPT_FUNCTION_DISBAND_QUEST
        | SCRIPT_FUNCTION_GET_QUEST_STATE => match index {
            0 | 1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_UPDATE_QUEST_POSITION => match index {
            0..=4 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_IS_QUEST_ENABLED
        | SCRIPT_FUNCTION_QUEST_TIME_CLEAR
        | SCRIPT_FUNCTION_GET_QUEST_TIME
        | SCRIPT_FUNCTION_DELETE_CARRIAGE
        | SCRIPT_FUNCTION_GET_CARRIAGE_DISTANCE
        | SCRIPT_FUNCTION_GET_CARRIAGE_INDEX
        | SCRIPT_FUNCTION_OPEN_SYNTHESIS => Unused,
        SCRIPT_FUNCTION_SET_QUEST_ENABLED | SCRIPT_FUNCTION_QUEST_TIME_BEGIN => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_CARRIAGE => match index {
            0 | 1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ACTIVITY_LOG => Unused,
        SCRIPT_FUNCTION_DELETE_USED_GOODS
        | SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_1
        | SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_2
        | SCRIPT_FUNCTION_SET_CURRENT_DURABILITY
        | SCRIPT_FUNCTION_SET_SELECTED_DURABILITY => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_1 | SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_2 => {
            match index {
                0 | 1 => Integer,
                _ => Unused,
            }
        }
        SCRIPT_FUNCTION_CHECK_USED_GOODS
        | SCRIPT_FUNCTION_GET_CURRENT_DURABILITY
        | SCRIPT_FUNCTION_GET_SELECTED_DURABILITY => Unused,
        SCRIPT_FUNCTION_FAIRY_EXP_UP => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_REFLUSH_EXTERN_PROPERTY => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_OPEN_PRECIOUS_BOX => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_PRECIOUS_ITEM => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_BATTLE_FAIRY_SKILL => match index {
            0 | 1 => String,
            2 | 3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_FETCH_POWER
        | SCRIPT_FUNCTION_ALLOCATE_BATTLE_FAIRY_SPECIAL_SKILL
        | SCRIPT_FUNCTION_REVIVE_BATTLE_FAIRY => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_BATTLE_FAIRY_ATTRIBUTE => match index {
            0 => String,
            1 | 2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ALLOCATE_BATTLE_FAIRY_SKILL
        | SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_ID
        | SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_LEVEL
        | SCRIPT_FUNCTION_ADD_BATTLE_FAIRY_EXPERIENCE
        | SCRIPT_FUNCTION_GET_BATTLE_FAIRY_ATTRIBUTE => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_RECREATE_BATTLE_FAIRY_ATTRIBUTES => match index {
            0 => String,
            1..=3 => Integer,
            _ => Unused,
        },
        _ if index < 3 => Integer,
        _ => Unused,
    }
}

fn run_battle_fairy_script_function<Runtime: ScriptFunctionRuntime>(
    game: &mut CGame,
    runtime: &mut Runtime,
    script_player_id: Option<i32>,
    function_id: i32,
    integer_arguments: [Option<i32>; 4],
    string_arguments: [Option<&[u8]>; 2],
) -> Option<i32> {
    let string = |index: usize| string_arguments[index].map(<[u8]>::to_vec);
    let integer = |index: usize| integer_arguments[index].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    let action = match function_id {
        SCRIPT_FUNCTION_ADD_BATTLE_FAIRY_SKILL => {
            let (Some(player_name), Some(skill_name)) = (string(0), string(1)) else {
                return Some(-1);
            };
            BattleFairyScriptAction::AddSkill {
                player_name,
                skill_name,
                skill_level: integer(2),
                position: integer_arguments[3],
            }
        }
        SCRIPT_FUNCTION_GET_FETCH_POWER => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            BattleFairyScriptAction::GetFetchPower { player_name }
        }
        SCRIPT_FUNCTION_SET_BATTLE_FAIRY_ATTRIBUTE => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            let (attribute, value) = (integer(1), integer(2));
            if attribute == SCRIPT_INT_PARAMETER_ERROR || value == SCRIPT_INT_PARAMETER_ERROR {
                return Some(0);
            }
            BattleFairyScriptAction::SetAttribute {
                player_name,
                attribute,
                value,
            }
        }
        SCRIPT_FUNCTION_ALLOCATE_BATTLE_FAIRY_SKILL => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            let position = integer(1);
            if !(3..=5).contains(&position) {
                return Some(0);
            }
            BattleFairyScriptAction::ResetSkill {
                player_name,
                position,
            }
        }
        SCRIPT_FUNCTION_ALLOCATE_BATTLE_FAIRY_SPECIAL_SKILL => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            BattleFairyScriptAction::ResetSkill {
                player_name,
                position: 6,
            }
        }
        SCRIPT_FUNCTION_REVIVE_BATTLE_FAIRY => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            if script_player_id.is_none() {
                return Some(0);
            }
            BattleFairyScriptAction::Revive { player_name }
        }
        SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_ID
        | SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_LEVEL => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            let position = integer(1);
            let valid = if function_id == SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_ID {
                (3..=6).contains(&position)
            } else {
                (0..=6).contains(&position)
            };
            if !valid {
                return Some(0);
            }
            BattleFairyScriptAction::GetSkillValue {
                player_name,
                position,
                value_id: if function_id == SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_ID {
                    2
                } else {
                    1
                },
            }
        }
        SCRIPT_FUNCTION_ADD_BATTLE_FAIRY_EXPERIENCE => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            let experience = integer(1);
            if experience <= 0 || experience == SCRIPT_INT_PARAMETER_ERROR {
                return Some(0);
            }
            BattleFairyScriptAction::AddExperience {
                player_name,
                experience,
            }
        }
        SCRIPT_FUNCTION_GET_BATTLE_FAIRY_ATTRIBUTE => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            let attribute = integer(1);
            if attribute == SCRIPT_INT_PARAMETER_ERROR {
                return Some(0);
            }
            BattleFairyScriptAction::GetAttribute {
                player_name,
                attribute,
            }
        }
        SCRIPT_FUNCTION_RECREATE_BATTLE_FAIRY_ATTRIBUTES => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            let (mode, minimum, maximum) = (integer(1), integer(2), integer(3));
            if script_player_id.is_none()
                || !matches!(mode, 0 | 1)
                || minimum == SCRIPT_INT_PARAMETER_ERROR
                || maximum == SCRIPT_INT_PARAMETER_ERROR
            {
                return Some(0);
            }
            BattleFairyScriptAction::RecreateAttributes {
                player_name,
                mode,
                minimum,
                maximum,
            }
        }
        _ => return None,
    };
    Some(game.run_battle_fairy_script_action(script_player_id, action, runtime))
}

fn run_fairy_script_function<Runtime: ScriptFunctionRuntime>(
    game: &mut CGame,
    runtime: &mut Runtime,
    script_player_id: Option<i32>,
    function_id: i32,
    evaluated_experience: Option<i32>,
) -> Option<i32> {
    if function_id != SCRIPT_FUNCTION_FAIRY_EXP_UP {
        return None;
    }
    let Some(player_id) = script_player_id else {
        return Some(0);
    };
    let experience = evaluated_experience.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    if experience == SCRIPT_INT_PARAMETER_ERROR {
        return Some(-1);
    }
    if experience <= 0 {
        return Some(0);
    }
    Some(i32::from(game.fairy_exp_up_selected_goods(
        player_id,
        experience as u32,
        runtime,
    )))
}

enum GoodsItemScriptFunctionOutcome {
    DifferentFunction,
    Invalid,
    Handled(i32),
}

fn script_used_goods(game: &CGame, player_id: i32, goods_id: CGuid) -> Option<&CGoods> {
    game.find_player(player_id)
        .and_then(|player| player.packet().base().find(goods_id))
}

fn script_selected_goods(game: &CGame, player_id: i32) -> Option<&CGoods> {
    let player = game.find_player(player_id)?;
    let goods_id = player.enhancement_selected_goods_id()?;
    player.get_goods_by_id(goods_id)
}

fn run_goods_item_script_function<Runtime: ScriptFunctionRuntime>(
    game: &mut CGame,
    runtime: &mut Runtime,
    script_player_id: Option<i32>,
    used_item_id: Option<CGuid>,
    function_id: i32,
    argument_count: usize,
    integer_arguments: [Option<i32>; 2],
) -> GoodsItemScriptFunctionOutcome {
    let used_identity = || script_player_id.zip(used_item_id);
    match function_id {
        SCRIPT_FUNCTION_DELETE_USED_GOODS => {
            let requested = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            if argument_count != 1 || requested <= 0 || requested == SCRIPT_INT_PARAMETER_ERROR {
                return GoodsItemScriptFunctionOutcome::Handled(0);
            }
            let Some((player_id, goods_id)) = used_identity() else {
                return GoodsItemScriptFunctionOutcome::Handled(0);
            };
            if script_used_goods(game, player_id, goods_id).is_none() {
                return GoodsItemScriptFunctionOutcome::Handled(0);
            }
            let Some(consumption) = game
                .find_player_mut(player_id)
                .and_then(|player| player.remove_packet_goods_by_id(goods_id, requested as u32))
            else {
                return GoodsItemScriptFunctionOutcome::Handled(0);
            };
            let removed = consumption
                .previous_amount
                .wrapping_sub(consumption.remaining_amount);
            let _ = game.send_player_packet_consumption(&consumption);
            GoodsItemScriptFunctionOutcome::Handled(removed as i32)
        }
        SCRIPT_FUNCTION_CHECK_USED_GOODS => {
            if argument_count != 0 {
                return GoodsItemScriptFunctionOutcome::Invalid;
            }
            let amount = used_identity()
                .and_then(|(player_id, goods_id)| script_used_goods(game, player_id, goods_id))
                .map_or(0, |goods| goods.amount() as i32);
            GoodsItemScriptFunctionOutcome::Handled(amount)
        }
        SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_1 | SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_2 => {
            if argument_count != 1 {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            }
            let Some((player_id, goods_id)) = used_identity() else {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            };
            let Some(goods) = script_used_goods(game, player_id, goods_id) else {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            };
            let property = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let value_id = if function_id == SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_1 {
                1
            } else {
                2
            };
            let value = goods.addon_property_value(game.goods_factory(), property, value_id);
            GoodsItemScriptFunctionOutcome::Handled(
                if value != 0 || goods.query_attribute(property) {
                    value
                } else {
                    -1
                },
            )
        }
        SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_1 | SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_2 => {
            if argument_count != 2 {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            }
            let Some((player_id, goods_id)) = used_identity() else {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            };
            let property = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let modifier = integer_arguments[1].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let value_id = if function_id == SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_1 {
                1
            } else {
                2
            };
            let update = game.find_player_mut(player_id).and_then(|player| {
                let goods = player.packet_mut().base_mut().find_mut(goods_id)?;
                goods
                    .set_addon_property_modifier_core(property, value_id, modifier)
                    .then(|| (goods.identity(), runtime.encode_goods_for_old_client(goods)))
            });
            if let Some((goods, payload)) = update {
                send_script_goods_update(game, player_id, goods, &payload);
                GoodsItemScriptFunctionOutcome::Handled(1)
            } else {
                GoodsItemScriptFunctionOutcome::Handled(0)
            }
        }
        SCRIPT_FUNCTION_GET_CURRENT_DURABILITY | SCRIPT_FUNCTION_GET_SELECTED_DURABILITY => {
            let Some(player_id) = script_player_id else {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            };
            let goods = if function_id == SCRIPT_FUNCTION_GET_SELECTED_DURABILITY {
                script_selected_goods(game, player_id)
            } else {
                used_item_id.and_then(|goods_id| script_used_goods(game, player_id, goods_id))
            };
            GoodsItemScriptFunctionOutcome::Handled(
                goods.map_or(-1, |goods| goods.current_durability()),
            )
        }
        SCRIPT_FUNCTION_SET_CURRENT_DURABILITY | SCRIPT_FUNCTION_SET_SELECTED_DURABILITY => {
            let requested = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            if requested == SCRIPT_INT_PARAMETER_ERROR {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            }
            let Some(player_id) = script_player_id else {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            };
            let goods_id = if function_id == SCRIPT_FUNCTION_SET_SELECTED_DURABILITY {
                game.find_player(player_id)
                    .and_then(|player| player.enhancement_selected_goods_id())
            } else {
                used_item_id
                    .filter(|goods_id| script_used_goods(game, player_id, *goods_id).is_some())
            };
            let Some(goods_id) = goods_id else {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            };
            let (updated, update) = game
                .find_player_mut(player_id)
                .and_then(|player| player.get_goods_by_id_mut(goods_id))
                .map_or((-1, None), |goods| {
                    let updated = goods.set_current_durability(requested);
                    let update = (updated != -1)
                        .then(|| (goods.identity(), runtime.encode_goods_for_old_client(goods)));
                    (updated, update)
                });
            if let Some((goods, payload)) = update {
                send_script_goods_update(game, player_id, goods, &payload);
            }
            GoodsItemScriptFunctionOutcome::Handled(updated)
        }
        _ => GoodsItemScriptFunctionOutcome::DifferentFunction,
    }
}

fn send_script_goods_update(game: &CGame, player_id: i32, goods: ShapeIdentity, payload: &[u8]) {
    let mut message = CMessage::new(0x0b_f918);
    message.add_long(player_id);
    message.base_mut().add_guid(goods.ex_id);
    message.add_ulong(payload.len() as u32);
    message.base_mut().add(payload);
    let _ = message.send_to_player(game.net_server(), player_id);
}

/// Упакованный результат `time()` для сценарного селектора 13. Это не время
/// Unix: исходник хранит поля CRT `tm_year/tm_mon` и завершает значение тремя
/// битами дня недели.
fn pack_script_local_time(time: TagTime) -> i32 {
    i32::from(time.year)
        .wrapping_sub(1900)
        .wrapping_shl(4)
        .wrapping_add(i32::from(time.month).wrapping_sub(1))
        .wrapping_shl(5)
        .wrapping_add(i32::from(time.day))
        .wrapping_shl(5)
        .wrapping_add(i32::from(time.hour))
        .wrapping_shl(6)
        .wrapping_add(i32::from(time.minute))
        .wrapping_shl(3)
        .wrapping_add(i32::from(time.day_of_week))
}

fn script_packed_time_component(function_id: i32, packed: i32) -> i32 {
    match function_id {
        SCRIPT_FUNCTION_YEAR => (packed >> 23).wrapping_add(1900),
        SCRIPT_FUNCTION_MONTH => ((packed >> 19) & 0x0f).wrapping_add(1),
        SCRIPT_FUNCTION_DAY => (packed >> 14) & 0x1f,
        SCRIPT_FUNCTION_HOUR => (packed >> 9) & 0x1f,
        SCRIPT_FUNCTION_MINUTE => (packed >> 3) & 0x3f,
        SCRIPT_FUNCTION_DAY_OF_WEEK => packed & 0x07,
        _ => unreachable!("calendar component вызывается только для 14..19"),
    }
}

fn decode_script_packed_local_time(packed: i32) -> Option<libc::time_t> {
    let mut local = libc::tm {
        tm_sec: 0,
        tm_min: (packed >> 3) & 0x3f,
        tm_hour: (packed >> 9) & 0x1f,
        tm_mday: (packed >> 14) & 0x1f,
        tm_mon: (packed >> 19) & 0x0f,
        tm_year: packed >> 23,
        tm_wday: packed & 0x07,
        tm_yday: 0,
        tm_isdst: -1,
        ..unsafe { std::mem::zeroed() }
    };
    let value = unsafe { libc::mktime(&raw mut local) };
    (value != -1).then_some(value)
}

fn script_packed_time_difference(function_id: i32, first: i32, second: Option<i32>) -> Option<i32> {
    let first = decode_script_packed_local_time(first)?;
    let second = match second {
        Some(second) => decode_script_packed_local_time(second)?,
        None => {
            let now = unsafe { libc::time(std::ptr::null_mut()) };
            if now == -1 {
                return None;
            }
            now
        }
    };
    let divisor = if function_id == SCRIPT_FUNCTION_HOUR_DIFF {
        3_600
    } else {
        60
    };
    i32::try_from(second.checked_sub(first)? / divisor).ok()
}

#[derive(Clone, Debug)]
struct ScriptWarRegionSnapshot {
    name: Vec<u8>,
    country: u8,
    war_region_type: i32,
    war_number: i32,
    city_state: i32,
    owned_faction_id: i32,
    owned_union_id: i32,
}

fn script_war_region_snapshot(
    game: &CGame,
    region_id: i32,
    proxy_allowed: bool,
) -> Option<ScriptWarRegionSnapshot> {
    if let Some(region) = game.find_region(region_id) {
        let base = region.base();
        return Some(ScriptWarRegionSnapshot {
            name: region.name().to_vec(),
            country: base.country,
            war_region_type: base.war_region_type,
            war_number: base.get_war_number(),
            city_state: base.get_city_state(),
            owned_faction_id: base.owned_city_faction(),
            owned_union_id: base.owned_city_union(),
        });
    }
    let region = proxy_allowed
        .then(|| game.find_proxy_region(region_id))
        .flatten()?;
    let (war_number, city_state) = region.war_state();
    let (owned_faction_id, owned_union_id) = region.owned_city_org();
    Some(ScriptWarRegionSnapshot {
        name: region.get_name().to_vec(),
        country: region.country(),
        war_region_type: region.war_region_type(),
        war_number,
        city_state,
        owned_faction_id,
        owned_union_id,
    })
}

fn send_village_war_script_notice(
    game: &CGame,
    player_id: i32,
    string_id: &[u8],
    argument: Option<&[u8]>,
) -> i32 {
    let text = argument.map_or_else(
        || game.get_string_by_id(string_id).to_vec(),
        |argument| format_legacy_text_fields(game.get_string_by_id(string_id), &[argument], 0xff),
    );
    colored_player_notice_message(0xffff_ffff, 0xffff_0000, &text)
        .send_to_player(game.net_server(), player_id)
}

fn run_village_war_menu_script_function(
    game: &CGame,
    script_player_id: Option<i32>,
    script_npc_id: Option<i32>,
    script_region_id: Option<i32>,
    function_id: i32,
    integer_arguments: [Option<i32>; 2],
) -> Option<ScriptFunctionDispatchOutcome> {
    let handled = |legacy_return| ScriptFunctionDispatchOutcome::Handled { legacy_return };
    match function_id {
        SCRIPT_FUNCTION_IS_ARRIVE_VILLAGE_APPLY_TIME
        | SCRIPT_FUNCTION_IS_ARRIVE_VILLAGE_WAR_TIME
        | SCRIPT_FUNCTION_IS_CITY_WAR_DECLARE_TIME
        | SCRIPT_FUNCTION_IS_CITY_WAR_FIGHT_TIME => {
            if !village_war_script_caller_is_live(game, script_player_id, script_npc_id) {
                return Some(handled(0));
            }
            let Some(region_id) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(handled(0));
            };
            let village_query = matches!(
                function_id,
                SCRIPT_FUNCTION_IS_ARRIVE_VILLAGE_APPLY_TIME
                    | SCRIPT_FUNCTION_IS_ARRIVE_VILLAGE_WAR_TIME
            );
            let state = script_war_region_snapshot(game, region_id, !village_query)
                .map_or(0, |region| region.city_state);
            Some(handled(i32::from(
                if matches!(
                    function_id,
                    SCRIPT_FUNCTION_IS_ARRIVE_VILLAGE_WAR_TIME
                        | SCRIPT_FUNCTION_IS_CITY_WAR_FIGHT_TIME
                ) {
                    state == 3
                } else {
                    state != 0
                },
            )))
        }
        SCRIPT_FUNCTION_GET_OWNED_REGION_FACTION_ID => {
            if !owned_region_script_caller_is_live(game, script_player_id, script_region_id) {
                return Some(handled(0));
            }
            let Some(mut region_id) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(handled(0));
            };
            if region_id == 0 {
                region_id = script_region_id.unwrap_or_default();
            }
            Some(handled(
                script_war_region_snapshot(game, region_id, true)
                    .map_or(0, |region| region.owned_faction_id),
            ))
        }
        SCRIPT_FUNCTION_GET_WAR_REGION_STATE => {
            let Some(region_id) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(handled(-1));
            };
            Some(handled(
                script_war_region_snapshot(game, region_id, true)
                    .map_or(-1, |region| region.city_state),
            ))
        }
        SCRIPT_FUNCTION_GET_COUNTRY_OWNING_REGION => {
            let Some(region_id) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(handled(0));
            };
            Some(handled(
                script_war_region_snapshot(game, region_id, true)
                    .map_or(0, |region| i32::from(region.country)),
            ))
        }
        SCRIPT_FUNCTION_GET_WAR_START_TIME => {
            let Some(region_id) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(handled(0));
            };
            let value =
                script_war_region_snapshot(game, region_id, true).map_or(0, |region| match region
                    .war_region_type
                {
                    1 => game.village_war_sys().get_war_start_time(region_id),
                    2 => game.attack_city_sys().get_war_start_time(region_id),
                    _ => 0,
                });
            Some(handled(value))
        }
        SCRIPT_FUNCTION_CITY_WAR_DECLARE => {
            if !village_war_script_caller_is_live(game, script_player_id, script_npc_id) {
                return Some(handled(0));
            }
            let (Some(player_id), Some(region_id), Some(money)) = (
                script_player_id,
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[1].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(handled(0));
            };
            let Some(player) = game.find_player(player_id) else {
                return Some(handled(0));
            };
            if !player.is_faction_master() {
                return Some(handled(0));
            }
            let faction_id = player.faction_id();
            let union_id = player.union_id();
            let country = player.country();
            let current_progress = player.current_progress();
            let Some(region) = script_war_region_snapshot(game, region_id, true) else {
                return Some(handled(0));
            };
            if country == region.country {
                let _ =
                    send_village_war_script_notice(game, player_id, b"GS0209", Some(&region.name));
                return Some(handled(0));
            }
            if faction_id == region.owned_faction_id {
                let _ =
                    send_village_war_script_notice(game, player_id, b"GS0202", Some(&region.name));
                return Some(handled(0));
            }
            if union_id != 0 && union_id == region.owned_union_id {
                let _ = send_village_war_script_notice(game, player_id, b"GS0210", None);
                return Some(handled(0));
            }
            if region.city_state == 0 {
                let _ = send_village_war_script_notice(game, player_id, b"GS0211", None);
                return Some(handled(0));
            }
            if current_progress != PlayerProgress::None {
                let _ = send_village_war_script_notice(game, player_id, b"GS0204", None);
                return Some(handled(0));
            }
            if game.attack_city_sys().attacks.values().any(|setup| {
                setup.region_state != 0 && setup.declaring_factions.contains(&faction_id)
            }) {
                let (notice, argument) = if (1..=4).contains(&country) {
                    (
                        b"GS0206".as_slice(),
                        game.globe_setup().country_name(country),
                    )
                } else {
                    (b"GS0205".as_slice(), None)
                };
                let _ = send_village_war_script_notice(game, player_id, notice, argument);
                return Some(handled(0));
            }
            let village_war_region =
                game.village_war_sys()
                    .village_wars
                    .values()
                    .find_map(|setup| {
                        (setup.region_state != 0 && setup.declaring_factions.contains(&faction_id))
                            .then_some(setup.war_region_id)
                    });
            if let Some(other_region_id) = village_war_region {
                if let Some(other_region) = script_war_region_snapshot(game, other_region_id, true)
                {
                    let _ = send_village_war_script_notice(
                        game,
                        player_id,
                        b"GS0207",
                        Some(&other_region.name),
                    );
                }
                return Some(handled(0));
            }
            let mut request = CMessage::new(0x0006_0137);
            request.add_long(player_id);
            request.add_long(region.war_number);
            request.add_long(money);
            let _ = request.send(game, false);
            Some(handled(0))
        }
        SCRIPT_FUNCTION_APPLY_FOR_VILLAGE_WAR => {
            if !village_war_script_caller_is_live(game, script_player_id, script_npc_id) {
                return Some(handled(0));
            }
            let (Some(player_id), Some(region_id), Some(money)) = (
                script_player_id,
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[1].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(handled(0));
            };
            let Some(player) = game.find_player(player_id) else {
                return Some(handled(0));
            };
            let faction_id = player.faction_id();
            if faction_id <= 0 {
                return Some(handled(0));
            }
            let union_id = player.union_id();
            let country = player.country();
            let current_progress = player.current_progress();
            let Some(war_region) = script_war_region_snapshot(game, region_id, true) else {
                return Some(handled(0));
            };
            if war_region.city_state == 0 {
                let _ = send_village_war_script_notice(game, player_id, b"GS0201", None);
                return Some(handled(0));
            }
            let village_region_id = game
                .village_war_sys()
                .get_village_region_id_by_time(war_region.war_number);
            let Some(village_region) = script_war_region_snapshot(game, village_region_id, true)
            else {
                return Some(handled(0));
            };
            if faction_id == village_region.owned_faction_id {
                let _ = send_village_war_script_notice(
                    game,
                    player_id,
                    b"GS0202",
                    Some(&village_region.name),
                );
                return Some(handled(0));
            }
            if union_id != 0 && union_id == village_region.owned_union_id {
                let _ = send_village_war_script_notice(game, player_id, b"GS0203", None);
                return Some(handled(0));
            }
            if current_progress != PlayerProgress::None {
                let _ = send_village_war_script_notice(game, player_id, b"GS0204", None);
                return Some(handled(0));
            }
            let attack_city_region = game.attack_city_sys().attacks.values().find_map(|setup| {
                (setup.region_state != 0 && setup.declaring_factions.contains(&faction_id))
                    .then_some(setup.city_region_id)
            });
            if attack_city_region.is_some() {
                let (notice, argument) = if (1..=4).contains(&country) {
                    (
                        b"GS0206".as_slice(),
                        game.globe_setup().country_name(country),
                    )
                } else {
                    (b"GS0205".as_slice(), None)
                };
                let _ = send_village_war_script_notice(game, player_id, notice, argument);
                return Some(handled(0));
            }
            let village_war_region =
                game.village_war_sys()
                    .village_wars
                    .values()
                    .find_map(|setup| {
                        (setup.region_state != 0 && setup.declaring_factions.contains(&faction_id))
                            .then_some(setup.war_region_id)
                    });
            if let Some(other_region_id) = village_war_region {
                if let Some(other_region) = script_war_region_snapshot(game, other_region_id, true)
                {
                    let _ = send_village_war_script_notice(
                        game,
                        player_id,
                        b"GS0207",
                        Some(&other_region.name),
                    );
                }
                return Some(handled(0));
            }
            let mut request = CMessage::new(0x0006_0135);
            request.add_long(player_id);
            request.add_long(war_region.war_number);
            request.add_long(money);
            let _ = request.send(game, false);
            Some(handled(0))
        }
        _ => None,
    }
}

fn lei_ting_local_time(timestamp: i32) -> Option<LeiTingLocalTime> {
    let timestamp = libc::time_t::from(timestamp);
    let mut local = std::mem::MaybeUninit::<libc::tm>::uninit();
    // `localtime_r` — потокобезопасная системная замена MSVC `_localtime`;
    // non-null результат полностью инициализирует `tm` до копирования.
    let result = unsafe { libc::localtime_r(&timestamp, local.as_mut_ptr()) };
    if result.is_null() {
        return None;
    }
    let local = unsafe { local.assume_init() };
    Some(LeiTingLocalTime {
        second: local.tm_sec,
        minute: local.tm_min,
        hour: local.tm_hour,
        month_day: local.tm_mday,
        month: local.tm_mon,
        year_since_1900: local.tm_year,
        week_day: local.tm_wday,
        year_day: local.tm_yday,
        daylight_saving: local.tm_isdst,
    })
}

fn next_lei_ting_daily_stamp_if_same_local_day(current_stamp: u32) -> Option<u32> {
    let now = chrono::Local::now().timestamp() as i32;
    let current_local = lei_ting_local_time(now)?;
    let stored_local = lei_ting_local_time(current_stamp as i32)?;
    if current_local.year_since_1900 != stored_local.year_since_1900
        || current_local.year_day != stored_local.year_day
    {
        return None;
    }

    let mut next_local = lei_ting_local_time(now.wrapping_add(86_400))?;
    CThingSetup::set_daily_update_stamp(&mut next_local);
    let mut native = libc::tm {
        tm_sec: next_local.second,
        tm_min: next_local.minute,
        tm_hour: next_local.hour,
        tm_mday: next_local.month_day,
        tm_mon: next_local.month,
        tm_year: next_local.year_since_1900,
        tm_wday: next_local.week_day,
        tm_yday: next_local.year_day,
        tm_isdst: next_local.daylight_saving,
        ..unsafe { std::mem::zeroed() }
    };
    // `mktime` сохраняет local-time/DST нормализацию CRT owner-а; legacy
    // записывал даже `-1` простым DWORD cast.
    Some(unsafe { libc::mktime(&mut native) } as i32 as u32)
}

fn script_notice_arguments<'a>(
    argument_count: usize,
    integer_arguments: &[Option<i32>; SCRIPT_FUNCTION_ARGUMENT_CAPACITY],
    string_arguments: &[Option<&'a [u8]>; SCRIPT_FUNCTION_ARGUMENT_CAPACITY],
    default_background: i32,
) -> Option<(&'a [u8], u32, u32)> {
    let text = string_arguments[0].filter(|text| text.len() <= 0xff && !text.contains(&0))?;
    if argument_count > 3 {
        return None;
    }
    let color = match integer_arguments[1] {
        Some(SCRIPT_INT_PARAMETER_ERROR) => return None,
        Some(color) => color,
        None => -1,
    };
    let background = match integer_arguments[2] {
        Some(SCRIPT_INT_PARAMETER_ERROR) => return None,
        Some(background) => background,
        None => default_background,
    };
    Some((text, color as u32, background as u32))
}

fn publish_script_lei_ting_update(game: &CGame, player_id: i32) {
    let payload = game
        .find_player(player_id)
        .expect("LeiTing mutation сохраняет player owner")
        .encode_lei_ting();
    let mut client = CMessage::new(0x000b_f73e);
    client.base_mut().add(&payload);
    let _ = client.send_to_player(game.net_server(), player_id);

    let mut world = CMessage::new(0x0005_fd10);
    world.add_long(player_id);
    world.base_mut().add(&payload);
    let _ = world.send(game, false);
}

fn format_script_variable_line(name: &[u8], index: Option<usize>, value: i32) -> Vec<u8> {
    let mut line = name.to_vec();
    if let Some(index) = index {
        line.push(b'[');
        line.extend_from_slice(index.to_string().as_bytes());
        line.push(b']');
    }
    line.extend_from_slice(b" = ");
    line.extend_from_slice(value.to_string().as_bytes());
    // EXE использует стековый буфер размером `0x19000` байт и оставляет
    // последний байт для завершающего нуля.
    line.truncate(0x18fff);
    line
}

#[derive(Clone, Copy)]
enum ScriptDiagnosticFormatArgument<'a> {
    Bytes(&'a [u8]),
    Word(u32),
}

fn format_script_diagnostic(
    template: &[u8],
    arguments: &[ScriptDiagnosticFormatArgument<'_>],
) -> Vec<u8> {
    let template = template.split(|byte| *byte == 0).next().unwrap_or_default();
    let mut output = Vec::with_capacity(template.len());
    let mut argument_index = 0usize;
    let mut offset = 0usize;
    while offset < template.len() && output.len() < 0x18fff {
        if template[offset] != b'%' {
            output.push(template[offset]);
            offset += 1;
            continue;
        }
        let Some(conversion) = template.get(offset + 1).copied() else {
            output.push(b'%');
            break;
        };
        if conversion == b'%' {
            output.push(b'%');
            offset += 2;
            continue;
        }
        let Some(argument) = arguments.get(argument_index).copied() else {
            output.extend_from_slice(&template[offset..]);
            break;
        };
        let rendered = match (conversion, argument) {
            (b's', ScriptDiagnosticFormatArgument::Bytes(value)) => value.to_vec(),
            (b'd', ScriptDiagnosticFormatArgument::Word(value)) => {
                (value as i32).to_string().into_bytes()
            }
            (b'u', ScriptDiagnosticFormatArgument::Word(value)) => value.to_string().into_bytes(),
            _ => {
                output.push(b'%');
                offset += 1;
                continue;
            }
        };
        let remaining = 0x18fffusize.saturating_sub(output.len());
        output.extend_from_slice(&rendered[..rendered.len().min(remaining)]);
        argument_index += 1;
        offset += 2;
    }
    output.truncate(0x18fff);
    output
}

fn run_core_player_script_function<Runtime: ScriptFunctionRuntime>(
    game: &mut CGame,
    runtime: &mut Runtime,
    script_player_id: Option<i32>,
    script_npc_id: Option<i32>,
    script_region_id: Option<i32>,
    drop_goods_position: Option<(i32, i32)>,
    script_id: i32,
    script_path: &[u8],
    function_id: i32,
    argument_count: usize,
    integer_arguments: [Option<i32>; SCRIPT_FUNCTION_ARGUMENT_CAPACITY],
    string_arguments: [Option<&[u8]>; SCRIPT_FUNCTION_ARGUMENT_CAPACITY],
) -> Option<ScriptFunctionDispatchOutcome> {
    let player_id = script_player_id.unwrap_or_default();
    match function_id {
        SCRIPT_FUNCTION_RANDOM => {
            if argument_count != 1 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let Some(maximum) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.script_random(maximum),
            })
        }
        SCRIPT_FUNCTION_TIME => {
            let legacy_return = pack_script_local_time(TagTime::local_now());
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_YEAR
        | SCRIPT_FUNCTION_MONTH
        | SCRIPT_FUNCTION_DAY
        | SCRIPT_FUNCTION_HOUR
        | SCRIPT_FUNCTION_MINUTE
        | SCRIPT_FUNCTION_DAY_OF_WEEK => {
            let packed = integer_arguments[0]
                .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                .unwrap_or_else(|| pack_script_local_time(TagTime::local_now()));
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: script_packed_time_component(function_id, packed),
            })
        }
        SCRIPT_FUNCTION_SECOND => Some(ScriptFunctionDispatchOutcome::Handled {
            legacy_return: i32::from(TagTime::local_now().second),
        }),
        SCRIPT_FUNCTION_HOUR_DIFF | SCRIPT_FUNCTION_MINUTE_DIFF => {
            if !(1..=2).contains(&argument_count) {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let Some(first) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let second = match integer_arguments[1] {
                Some(SCRIPT_INT_PARAMETER_ERROR) => {
                    return Some(ScriptFunctionDispatchOutcome::Invalid);
                }
                value => value,
            };
            Some(
                script_packed_time_difference(function_id, first, second)
                    .map_or(ScriptFunctionDispatchOutcome::Invalid, |legacy_return| {
                        ScriptFunctionDispatchOutcome::Handled { legacy_return }
                    }),
            )
        }
        SCRIPT_FUNCTION_GET_REGION_RANDOM_POSITION => {
            if argument_count > 1 {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            }
            let Some(player_id) =
                script_player_id.filter(|player_id| game.find_player(*player_id).is_some())
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let player_region_id = game
                .find_player(player_id)
                .and_then(|player| player.server_region_id())
                .unwrap_or_default();
            let region_id = match integer_arguments[0] {
                Some(SCRIPT_INT_PARAMETER_ERROR) => {
                    return Some(ScriptFunctionDispatchOutcome::Invalid);
                }
                Some(0) | None => player_region_id,
                Some(region_id) => region_id,
            };
            let Some(position) = game
                .find_region(region_id)
                .and_then(|owner| owner.base().region.get_random_pos(runtime).ok())
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let player = game
                .find_player_mut(player_id)
                .expect("script-player сохранён между random position и $m_Temp mutation");
            let _ = player.set_integer_variable(b"$m_Temp", 0, position.x);
            let _ = player.set_integer_variable(b"$m_Temp", 1, position.y);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 1 })
        }
        SCRIPT_FUNCTION_IS_CHARGED => Some(ScriptFunctionDispatchOutcome::Handled {
            legacy_return: i32::from(
                game.find_player(player_id)
                    .is_some_and(|player| player.is_charged()),
            ),
        }),
        SCRIPT_FUNCTION_SET_CHARGED => {
            let requested = integer_arguments[0]
                .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                .unwrap_or_default();
            let legacy_return = game.find_player_mut(player_id).map_or(0, |player| {
                player.set_charged(requested != 0);
                i32::from(player.is_charged())
            });
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_CHANGE_BODY_CHECK => {
            let Some(player_id) = script_player_id else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.script_change_body_check(player_id, true),
            })
        }
        SCRIPT_FUNCTION_CHECK_MODE => Some(ScriptFunctionDispatchOutcome::Handled {
            // Точный селектор читает `m_eProgress` напрямую. Исходная ветвь
            // без игрока разыменовывала null; типизированная среда не повторяет
            // неопределённое поведение и оставляет начальный `PROGRESS_NONE`.
            legacy_return: game
                .find_player(player_id)
                .map_or(0, |player| player.current_progress() as i32),
        }),
        SCRIPT_FUNCTION_GET_PROGRESS => Some(ScriptFunctionDispatchOutcome::Handled {
            // Поздний alias идёт через `GetCurrentProgress` и в отличие от
            // CheckMode явно возвращает -1 при отсутствии script-player.
            legacy_return: script_player_id
                .and_then(|player_id| game.find_player(player_id))
                .map_or(-1, |player| player.current_progress() as i32),
        }),
        SCRIPT_FUNCTION_IS_COMBAT_STATE => Some(ScriptFunctionDispatchOutcome::Handled {
            // Точный виртуальный вызов `CShape::GetAction +0x74`: только
            // действие `1` считается боевым, а отсутствие сценарного игрока
            // возвращает `-1`.
            legacy_return: game
                .find_player(player_id)
                .map_or(-1, |player| i32::from(player.shape().get_action() == 1)),
        }),
        SCRIPT_FUNCTION_DRAW_AWARDS => {
            let Some(patch_id) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let information_type = integer_arguments[1].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let (color, background) = if information_type == 1 {
                (
                    integer_arguments[2]
                        .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                        .map_or(0xffff_ff00, |value| value as u32),
                    integer_arguments[3]
                        .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                        .map_or(0xffff_0000, |value| value as u32),
                )
            } else {
                (0, 0)
            };
            let Some(player) = game.find_player(player_id) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: -1 });
            };
            if player.is_dead() || player.in_changing_server() || player.in_changing_region() {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: -1 });
            }
            if player.current_progress() != PlayerProgress::None {
                let _ =
                    colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0175"))
                        .send_to_player(game.net_server(), player_id);
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: -1 });
            }
            if player.packet().is_full(game.goods_factory()) {
                let _ =
                    colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0176"))
                        .send_to_player(game.net_server(), player_id);
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: -1 });
            }
            let _ = runtime.submit_script_award_authentication(
                player,
                patch_id,
                information_type,
                color,
                background,
            );
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 1 })
        }
        SCRIPT_FUNCTION_ADD_UNDEAD_STATE
        | SCRIPT_FUNCTION_DELETE_UNDEAD_STATE
        | SCRIPT_FUNCTION_GET_UNDEAD_STATE => {
            let Some(state_id) = integer_arguments[0]
                .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                .map(|value| value as u32)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let Some(player_id) = script_player_id else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let now_ms = runtime.now_milliseconds();
            let legacy_return = match function_id {
                SCRIPT_FUNCTION_ADD_UNDEAD_STATE => {
                    game.add_script_appellation_state(player_id, state_id, now_ms, runtime)
                }
                SCRIPT_FUNCTION_DELETE_UNDEAD_STATE => {
                    game.delete_script_appellation_state(player_id, state_id, now_ms, runtime)
                }
                SCRIPT_FUNCTION_GET_UNDEAD_STATE => game
                    .find_player(player_id)
                    .map_or(0, |player| player.get_appellation_state(state_id)),
                _ => unreachable!("undead-state selector проверен"),
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: legacy_return as i32,
            })
        }
        SCRIPT_FUNCTION_SET_HOTKEY => {
            let position = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let hotkey_type = integer_arguments[1].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let index = integer_arguments[2].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            if !(0..=23).contains(&position) {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            }
            let value = (index as u32) | if hotkey_type == 1 { 0x8000_0000 } else { 0 };
            game.set_script_player_hotkey(player_id, position as u8, value);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_ADD_EX_STATE
        | SCRIPT_FUNCTION_DELETE_EX_STATE
        | SCRIPT_FUNCTION_GET_EX_STATE
        | SCRIPT_FUNCTION_ADD_EX_STATE_NEW
        | SCRIPT_FUNCTION_DELETE_EX_STATE_NEW
        | SCRIPT_FUNCTION_GET_EX_STATE_NEW => {
            let Some(state_id) = integer_arguments[0]
                .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                .map(|value| value as u32)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let Some(player_id) = script_player_id else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let kind = if matches!(
                function_id,
                SCRIPT_FUNCTION_ADD_EX_STATE_NEW
                    | SCRIPT_FUNCTION_DELETE_EX_STATE_NEW
                    | SCRIPT_FUNCTION_GET_EX_STATE_NEW
            ) {
                ExtendedStateKind::New
            } else {
                ExtendedStateKind::Original
            };
            let legacy_return = match function_id {
                SCRIPT_FUNCTION_ADD_EX_STATE | SCRIPT_FUNCTION_ADD_EX_STATE_NEW => {
                    let now_ms = runtime.now_milliseconds();
                    game.add_script_extended_state(player_id, state_id, kind, now_ms, runtime)
                }
                SCRIPT_FUNCTION_DELETE_EX_STATE | SCRIPT_FUNCTION_DELETE_EX_STATE_NEW => {
                    let now_ms = runtime.now_milliseconds();
                    game.delete_script_extended_state(player_id, state_id, kind, now_ms, runtime)
                }
                SCRIPT_FUNCTION_GET_EX_STATE | SCRIPT_FUNCTION_GET_EX_STATE_NEW => game
                    .find_player(player_id)
                    .map_or(0, |player| player.get_extended_state(kind, state_id)),
                _ => unreachable!("extended-state selector проверен"),
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: legacy_return as i32,
            })
        }
        SCRIPT_FUNCTION_ADD_CHANGE_BODY_STATE
        | SCRIPT_FUNCTION_DELETE_CHANGE_BODY_STATE
        | SCRIPT_FUNCTION_GET_CHANGE_BODY_STATE => {
            let Some(state_id) = integer_arguments[0]
                .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                .map(|value| value as u32)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let Some(player_id) = script_player_id else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let legacy_return = match function_id {
                SCRIPT_FUNCTION_ADD_CHANGE_BODY_STATE => {
                    let now_ms = runtime.now_milliseconds();
                    game.add_script_change_body_state(player_id, state_id, now_ms, runtime)
                }
                SCRIPT_FUNCTION_DELETE_CHANGE_BODY_STATE => {
                    game.delete_script_change_body_state(player_id, state_id, runtime)
                }
                SCRIPT_FUNCTION_GET_CHANGE_BODY_STATE => game
                    .find_player(player_id)
                    .map_or(0, |player| player.get_change_body_state(state_id)),
                _ => unreachable!("ChangeBody selector проверен"),
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: legacy_return as i32,
            })
        }
        SCRIPT_FUNCTION_CREATE_NPC => {
            if argument_count < 8 || argument_count > 12 || game.find_player(player_id).is_none() {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let Some(mut name) = string_arguments[0]
                .filter(|name| name.len() <= 0xff && !name.contains(&0))
                .map(<[u8]>::to_vec)
            else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let mut values = [0_i32; 7];
            for (index, destination) in values.iter_mut().enumerate() {
                let Some(value) = integer_arguments[index + 1]
                    .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                else {
                    return Some(ScriptFunctionDispatchOutcome::Invalid);
                };
                *destination = value;
            }
            if !(0..=MAXIMUM_SCRIPT_SPAWN_COUNT).contains(&values[1])
                || values[4] < values[2]
                || values[5] < values[3]
                || i32::try_from(i64::from(values[4]) - i64::from(values[2])).is_err()
                || i32::try_from(i64::from(values[5]) - i64::from(values[3])).is_err()
            {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            if name.is_empty() {
                name.extend_from_slice(b"No Name");
            }
            let mut script = match (argument_count > 8, string_arguments[8]) {
                (true, Some(script)) if script.len() <= 0xff && !script.contains(&0) => {
                    script.to_vec()
                }
                (true, _) => return Some(ScriptFunctionDispatchOutcome::Invalid),
                (false, _) => Vec::new(),
            };
            if script.is_empty() {
                script.extend_from_slice(b"scripts/Npc/test.script");
            }
            let player_region_id = game
                .find_player(player_id)
                .and_then(|player| player.server_region_id())
                .unwrap_or_default();
            let mut region_id = script_region_id.unwrap_or(player_region_id);
            if let Some(requested_region_id) = integer_arguments[9] {
                if requested_region_id == SCRIPT_INT_PARAMETER_ERROR {
                    return Some(ScriptFunctionDispatchOutcome::Invalid);
                }
                region_id = if requested_region_id == 0 {
                    script_region_id.unwrap_or_default()
                } else {
                    requested_region_id
                };
            }
            if region_id <= 0 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let show_list = match integer_arguments[10] {
                Some(SCRIPT_INT_PARAMETER_ERROR) => {
                    return Some(ScriptFunctionDispatchOutcome::Invalid);
                }
                Some(show_list) => show_list != 0,
                None => true,
            };
            let lifetime = match integer_arguments[11] {
                Some(SCRIPT_INT_PARAMETER_ERROR) => {
                    return Some(ScriptFunctionDispatchOutcome::Invalid);
                }
                Some(lifetime) => lifetime,
                None => 0,
            };
            let setup = ServerRegionNpcSetup {
                show_list,
                picture_id: values[0],
                count: values[1],
                left: values[2],
                top: values[3],
                right: values[4],
                bottom: values[5],
                direction: values[6],
                time: lifetime,
                name,
                script,
            };
            let Some(mut owner) = game.take_region_owner(region_id) else {
                let mut request = CMessage::new(0x0005_fa0b);
                request.add_byte(1);
                request.add_long(region_id);
                request.base_mut().add(&setup.name);
                request.add_byte(0);
                request.add_long(setup.picture_id);
                request.add_long(setup.count);
                request.add_long(setup.left);
                request.add_long(setup.top);
                request.add_long(setup.right);
                request.add_long(setup.bottom);
                request.add_long(setup.direction);
                let has_script = setup.script.len() > 1 && setup.script != b"0";
                request.add_byte(u8::from(has_script));
                if has_script {
                    request.base_mut().add(&setup.script);
                    request.add_byte(0);
                }
                request.add_long(setup.time);
                let _ = request.send(game, false);
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let (area_width, area_height) = game.area_dimensions();
            let spawn = owner.base_mut().add_npc_with_clock(
                &setup,
                false,
                true,
                area_width,
                area_height,
                runtime,
                |runtime| runtime.now_milliseconds(),
            );
            game.restore_region_owner(owner);
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: spawn
                    .ok()
                    .and_then(|spawn| spawn.created_ids.first().copied())
                    .unwrap_or_default(),
            })
        }
        SCRIPT_FUNCTION_NPC_TALK => {
            let (Some(region_id), Some(npc_id), Some(name), Some(text)) = (
                script_region_id,
                script_npc_id,
                string_arguments[0].filter(|value| {
                    !value.is_empty() && value.len() <= 1023 && !value.contains(&0)
                }),
                string_arguments[1].filter(|value| {
                    !value.is_empty() && value.len() <= 1023 && !value.contains(&0)
                }),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            if argument_count != 2 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let _ = game.script_npc_talk(region_id, npc_id, name, text);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_MONSTER_TALK => {
            if let (Some(player_id), Some(name), Some(text)) =
                (script_player_id, string_arguments[0], string_arguments[1])
            {
                let _ = game.script_monsters_talk(player_id, name, text);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_OPEN_PLAYER_UI => {
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_CALL_MONSTER => {
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 1 })
        }
        SCRIPT_FUNCTION_ATTACK_PLAYER => {
            if let (Some(player_id), Some(target_name)) = (
                script_player_id,
                string_arguments[0].filter(|name| !name.is_empty()),
            ) {
                let _ = game.script_monsters_attack_player(player_id, target_name);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_MOVE_PLAYER => {
            let arguments = std::array::from_fn(|index| {
                integer_arguments[index].unwrap_or(SCRIPT_INT_PARAMETER_ERROR)
            });
            let _ = game.move_script_players_in_rectangles(arguments, runtime);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_CREATE_MONSTER => {
            if !(6..=8).contains(&argument_count) || game.find_player(player_id).is_none() {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let Some(original_name) = string_arguments[0]
                .filter(|name| !name.is_empty() && name.len() <= u8::MAX as usize)
            else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let mut values = [0_i32; 5];
            for (index, destination) in values.iter_mut().enumerate() {
                let Some(value) = integer_arguments[index + 1]
                    .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                else {
                    return Some(ScriptFunctionDispatchOutcome::Invalid);
                };
                *destination = value;
            }
            let [count, left, top, right, bottom] = values;
            if !(1..=MAXIMUM_SCRIPT_SPAWN_COUNT).contains(&count)
                || right < left
                || bottom < top
                || i32::try_from(i64::from(right) - i64::from(left)).is_err()
                || i32::try_from(i64::from(bottom) - i64::from(top)).is_err()
            {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let script_file = match string_arguments[6] {
                Some(script) if script.len() <= u8::MAX as usize && !script.contains(&0) => script,
                Some(_) => return Some(ScriptFunctionDispatchOutcome::Invalid),
                None => &[],
            };
            let player_region_id = game
                .find_player(player_id)
                .and_then(|player| player.server_region_id())
                .unwrap_or_default();
            let region_id = match integer_arguments[7] {
                Some(SCRIPT_INT_PARAMETER_ERROR) => {
                    return Some(ScriptFunctionDispatchOutcome::Invalid);
                }
                Some(region_id) => region_id,
                None => player_region_id,
            };
            if region_id <= 0 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let Some(mut owner) = game.take_region_owner(region_id) else {
                let mut request = CMessage::new(0x0005_fa0b);
                request.add_byte(0);
                request.add_long(region_id);
                request.base_mut().add(original_name);
                request.add_byte(0);
                request.add_long(count);
                request.add_long(left);
                request.add_long(top);
                request.add_long(right);
                request.add_long(bottom);
                let has_script = !script_file.is_empty() && script_file != b"0";
                request.add_byte(u8::from(has_script));
                if has_script {
                    request.base_mut().add(script_file);
                    request.add_byte(0);
                }
                let _ = request.send(game, false);
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let Some(property) = game
                .find_monster_property_by_origin_name(original_name)
                .cloned()
            else {
                game.restore_region_owner(owner);
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };

            let (area_width, area_height) = game.area_dimensions();
            let width = right.wrapping_sub(left);
            let height = bottom.wrapping_sub(top);
            let mut first_monster_id = 0;
            for _ in 0..count {
                let position = owner
                    .base()
                    .region
                    .get_random_pos_in_range(left, top, width, height, runtime);
                let Ok(position) = position else {
                    continue;
                };
                let spawn = owner.base_mut().add_monster(
                    &property,
                    position.x,
                    position.y,
                    -1,
                    true,
                    false,
                    runtime.now_milliseconds(),
                    area_width,
                    area_height,
                    runtime,
                );
                let Ok(monster_id) = spawn else {
                    continue;
                };
                if first_monster_id == 0 {
                    first_monster_id = monster_id;
                }
                if !script_file.is_empty()
                    && script_file != b"0"
                    && let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id)
                {
                    monster.set_script_file(script_file);
                }
            }
            game.restore_region_owner(owner);
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: first_monster_id,
            })
        }
        SCRIPT_FUNCTION_DELETE_NPC | SCRIPT_FUNCTION_DELETE_MONSTER => {
            if argument_count != 1 || script_player_id.is_none() {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let Some(target_id) = integer_arguments[0]
                .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR && *value > 0)
            else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            if function_id == SCRIPT_FUNCTION_DELETE_NPC {
                let _ = game.delete_script_npc(player_id, target_id);
            } else {
                let _ = game.delete_script_monster(player_id, target_id);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_KILL_MONSTER => {
            if argument_count != 1 || script_player_id.is_none() {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let Some(target_id) = integer_arguments[0]
                .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR && *value > 0)
            else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let _ = game.kill_script_monster(player_id, target_id, runtime);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_DROP_GOODS => {
            if !(3..=4).contains(&argument_count) {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let (Some(player_id), Some(region_id), Some(tile_x), Some(tile_y), Some(name)) = (
                script_player_id.filter(|player_id| game.find_player(*player_id).is_some()),
                script_region_id.filter(|region_id| game.find_region(*region_id).is_some()),
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[1].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                string_arguments[2].filter(|name| !name.is_empty()),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let amount = integer_arguments[3]
                .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                .unwrap_or(1);
            if !(1..=MAXIMUM_SCRIPT_SPAWN_COUNT).contains(&amount) {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let (tile_x, tile_y) = drop_goods_position.unwrap_or((tile_x, tile_y));
            let _ = game.drop_script_goods(
                player_id,
                region_id,
                name,
                amount as u32,
                tile_x,
                tile_y,
                runtime,
            );
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_AUTO_MOVE => {
            let (Some(player_id), Some(tile_x), Some(tile_y)) = (
                script_player_id.filter(|player_id| game.find_player(*player_id).is_some()),
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[1].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let mut movement = CMessage::new(0x000b_f723);
            movement.add_long(tile_x);
            movement.add_long(tile_y);
            let _ = movement.send_to_player(game.net_server(), player_id);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_DELETE_MONSTER_RECT => {
            let region_id = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let Some((width, height)) = game
                .find_region(region_id)
                .map(|owner| (owner.base().region.width, owner.base().region.height))
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let (maximum_x, maximum_y) = (width.wrapping_sub(1), height.wrapping_sub(1));
            if maximum_x < 0 || maximum_y < 0 {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            }
            let mut rectangle = [
                integer_arguments[1].unwrap_or(SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[2].unwrap_or(SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[3].unwrap_or(SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[4].unwrap_or(SCRIPT_INT_PARAMETER_ERROR),
            ];
            if rectangle
                .iter()
                .all(|value| *value == SCRIPT_INT_PARAMETER_ERROR)
            {
                rectangle = [0, 0, maximum_x, maximum_y];
            } else {
                rectangle[0] = rectangle[0].clamp(0, maximum_x);
                rectangle[1] = rectangle[1].clamp(0, maximum_y);
                rectangle[2] = rectangle[2].clamp(0, maximum_x);
                rectangle[3] = rectangle[3].clamp(0, maximum_y);
            }
            let _ = game.delete_script_monsters_in_rect(region_id, rectangle, string_arguments[5]);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_DELETE_NPC_BY_NAME => {
            let Some(name) = string_arguments[0].filter(|name| !name.is_empty()) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let player_region_id = game
                .find_player(player_id)
                .and_then(|player| player.server_region_id())
                .unwrap_or_default();
            let source_region_id = script_region_id.unwrap_or(player_region_id);
            let region_id = match integer_arguments[1] {
                Some(SCRIPT_INT_PARAMETER_ERROR) => {
                    return Some(ScriptFunctionDispatchOutcome::Invalid);
                }
                Some(0) => script_region_id.unwrap_or_default(),
                Some(region_id) => region_id,
                None => source_region_id,
            };
            let _ = game.delete_script_npc_by_name(region_id, name);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_PLAYER_MESSAGE => {
            let (Some(target_name), Some(text)) = (
                string_arguments[0].filter(|name| !name.is_empty() && name.len() <= 23),
                string_arguments[1].filter(|text| text.len() <= 1023),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let color_argument =
                integer_arguments[2].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR);
            let color = color_argument.unwrap_or(-1);
            let message_type = if color_argument.is_some() {
                integer_arguments[3]
                    .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                    .unwrap_or_default()
            } else {
                0
            };
            if let Some(target_id) = game
                .find_player_by_name(target_name)
                .map(CPlayer::player_id)
            {
                let _ = colored_player_notice_message(color as u32, 0, text)
                    .send_to_player(game.net_server(), target_id);
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            }
            let Some(source_player_id) = script_player_id.filter(|player_id| *player_id > 0) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let mut request = CMessage::new(0x0005_fa04);
            request.base_mut().add(target_name);
            request.add_byte(0);
            request.base_mut().add(text);
            request.add_byte(0);
            request.add_long(color);
            request.add_long(message_type);
            request.add_long(source_player_id);
            let _ = request.send(game, false);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_SET_THING_COUNT => {
            let (Some(player_id), Some(thing_id), Some(requested_count)) = (
                script_player_id.filter(|player_id| game.find_player(*player_id).is_some()),
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[1].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let outcome = game
                .find_player_mut(player_id)
                .expect("script-player проверен до LeiTing mutation")
                .set_lei_ting_thing_count(
                    thing_id,
                    requested_count,
                    next_lei_ting_daily_stamp_if_same_local_day,
                );
            if matches!(outcome, PlayerLeiTingThingCountOutcome::Updated { .. }) {
                publish_script_lei_ting_update(game, player_id);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: -1 })
        }
        SCRIPT_FUNCTION_GET_THING_COUNT => {
            let (Some(player_id), Some(thing_id)) = (
                script_player_id.filter(|player_id| game.find_player(*player_id).is_some()),
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game
                    .find_player(player_id)
                    .and_then(|player| player.lei_ting_thing_count(thing_id))
                    .map(i32::from)
                    .unwrap_or(-1),
            })
        }
        SCRIPT_FUNCTION_RELOAD => {
            let Some(profile) = string_arguments[0] else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            if game.find_player(player_id).is_some() {
                let mut request = CMessage::new(0x0005_ff06);
                request.add_long(player_id);
                request.base_mut().add(profile);
                request.add_byte(0);
                let _ = request.send(game, false);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_GET_ME => {
            if argument_count != 1 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let Some(property) = string_arguments[0] else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let Some(player) = game.find_player(player_id) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let legacy_return = player.script_value(property).unwrap_or(0);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_CHANGE_ME => {
            if argument_count != 2 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let (Some(property), Some(delta)) = (string_arguments[0], integer_arguments[1]) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            Some(
                game.change_script_player_property(player_id, property, delta, runtime)
                    .map_or(ScriptFunctionDispatchOutcome::Invalid, |legacy_return| {
                        ScriptFunctionDispatchOutcome::Handled { legacy_return }
                    }),
            )
        }
        SCRIPT_FUNCTION_SET_ME => {
            if argument_count != 2 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let (Some(property), Some(value)) = (string_arguments[0], integer_arguments[1]) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            if !property.eq_ignore_ascii_case(b"dwVigour")
                && !property.eq_ignore_ascii_case(b"dwExp")
                && !property.eq_ignore_ascii_case(b"dwAppellationID")
                && !property.eq_ignore_ascii_case(b"dwRankOfNobilityID")
            {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            Some(
                game.set_script_player_property(player_id, property, value, runtime)
                    .map_or(ScriptFunctionDispatchOutcome::Invalid, |legacy_return| {
                        ScriptFunctionDispatchOutcome::Handled { legacy_return }
                    }),
            )
        }
        SCRIPT_FUNCTION_CHECK_LEVEL => {
            if argument_count != 0 || game.find_player(player_id).is_none() {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.check_script_player_level(player_id, runtime),
            })
        }
        SCRIPT_FUNCTION_SET_ENERGY | SCRIPT_FUNCTION_SET_MAXIMUM_ENERGY => {
            if argument_count != 1 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let Some(energy) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let maximum = function_id == SCRIPT_FUNCTION_SET_MAXIMUM_ENERGY;
            Some(
                game.set_script_player_energy(player_id, energy, maximum)
                    .map_or(ScriptFunctionDispatchOutcome::Invalid, |_| {
                        ScriptFunctionDispatchOutcome::Handled { legacy_return: 1 }
                    }),
            )
        }
        SCRIPT_FUNCTION_GET_ENERGY | SCRIPT_FUNCTION_GET_MAXIMUM_ENERGY => {
            if argument_count != 0 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let Some(player) = game.find_player(player_id) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let legacy_return = if function_id == SCRIPT_FUNCTION_GET_ENERGY {
                player.energy() as i32
            } else {
                player.maximum_energy() as i32
            };
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_WALK_STEP | SCRIPT_FUNCTION_RUN_STEP => {
            let Some(direction) = integer_arguments[0]
                .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                .filter(|value| (0..=7).contains(value))
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let _ = game.move_script_player_step(
                player_id,
                direction as usize,
                i32::from(function_id == SCRIPT_FUNCTION_RUN_STEP),
            );
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_SET_PLAYER_POSITION => {
            let tile_x = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let tile_y = integer_arguments[1].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let _ = game.set_script_player_position(player_id, tile_x, tile_y);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_SET_PLAYER_DIRECTION => {
            let direction = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let _ = game.set_script_player_direction(player_id, direction);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_GET_EQUIPMENT_ID_BY_POSITION => {
            let (Some(player_name), Some(position)) = (
                string_arguments[0],
                integer_arguments[1].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let legacy_return =
                game.script_equipment_base_index(script_player_id, player_name, position);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_UPGRADE_EQUIPMENT => {
            let (Some(player_name), Some(position), Some(level_delta)) = (
                string_arguments[0],
                integer_arguments[1].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[2].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let legacy_return = game.upgrade_script_player_equipment(
                script_player_id,
                player_name,
                position,
                level_delta,
                runtime,
            );
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_SET_PLAYER_LEVEL => {
            let (Some(target_name), Some(level)) = (string_arguments[0], integer_arguments[1])
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.set_script_player_level(player_id, target_name, level as u8),
            })
        }
        SCRIPT_FUNCTION_CHANGE_PLAYER => {
            let (Some(target_name), Some(property), Some(delta)) = (
                string_arguments[0],
                string_arguments[1],
                integer_arguments[2],
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: -1 });
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game
                    .change_named_script_player_property(target_name, property, delta, runtime)
                    .unwrap_or(-1),
            })
        }
        SCRIPT_FUNCTION_SET_PLAYER => {
            let (Some(target_name), Some(property)) = (string_arguments[0], string_arguments[1])
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: -1 });
            };
            let requested = integer_arguments[2].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game
                    .set_named_script_player_property(target_name, property, requested, runtime)
                    .unwrap_or(-1),
            })
        }
        SCRIPT_FUNCTION_GET_PLAYER => {
            let (Some(target_name), Some(property)) = (string_arguments[0], string_arguments[1])
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            if let Some(target) = game.find_player_by_name(target_name) {
                return Some(ScriptFunctionDispatchOutcome::Handled {
                    legacy_return: target.script_value(property).unwrap_or(0),
                });
            }
            let mut request = CMessage::new(0x0005_ff02);
            request.add_long(player_id);
            request.base_mut().add(target_name);
            request.add_byte(0);
            request.base_mut().add(property);
            request.add_byte(0);
            request.add_long(script_id);
            let _ = request.send(game, false);
            Some(ScriptFunctionDispatchOutcome::Yielded { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_IS_PLAYER_ONLINE => {
            let Some(target_name) = string_arguments[0]
                .filter(|name| !name.is_empty() && name.len() <= u8::MAX as usize)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            if let Some(target) = game.find_player_by_name(target_name) {
                return Some(ScriptFunctionDispatchOutcome::Handled {
                    legacy_return: target.player_id(),
                });
            }
            if script_player_id.is_none() {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            }
            let mut request = CMessage::new(0x0005_ff05);
            request.add_long(player_id);
            request.base_mut().add(target_name);
            request.add_byte(0);
            request.add_long(script_id);
            match request.send(game, false) {
                Ok(1) => Some(ScriptFunctionDispatchOutcome::Yielded { legacy_return: 0 }),
                _ => Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 }),
            }
        }
        SCRIPT_FUNCTION_GET_PLAYER_ID => Some(ScriptFunctionDispatchOutcome::Handled {
            legacy_return: string_arguments[0]
                .and_then(|target_name| game.find_player_by_name(target_name))
                .map_or(0, CPlayer::player_id),
        }),
        SCRIPT_FUNCTION_GET_REGION_ID => {
            let Some(region_name) = string_arguments[0]
                .filter(|name| !name.is_empty() && name.len() <= u8::MAX as usize)
            else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            if let Some(region) = game.find_region_by_name(region_name) {
                return Some(ScriptFunctionDispatchOutcome::Handled {
                    legacy_return: region.region_id(),
                });
            }
            if script_player_id.is_none() {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let mut request = CMessage::new(0x0005_ff04);
            request.add_long(player_id);
            request.base_mut().add(region_name);
            request.add_byte(0);
            request.add_long(script_id);
            match request.send(game, false) {
                Ok(1) => Some(ScriptFunctionDispatchOutcome::Yielded { legacy_return: 0 }),
                _ => Some(ScriptFunctionDispatchOutcome::Invalid),
            }
        }
        SCRIPT_FUNCTION_SET_PLAYER_REGION => {
            let Some(caller) = game.find_player(player_id) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let target_name = string_arguments[0]
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| caller.player_name())
                .to_vec();
            let Some(target_region_id) = integer_arguments[1].filter(|value| *value > 0) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            if target_name.is_empty() || target_name.len() > u8::MAX as usize {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let mut tile_x = integer_arguments[2].unwrap_or(-1);
            let mut tile_y = integer_arguments[3].unwrap_or(-1);
            if tile_x == 0 && tile_y == 0 {
                tile_x = -1;
                tile_y = -1;
            }
            if !((tile_x == -1 && tile_y == -1) || (tile_x >= 0 && tile_y >= 0)) {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let direction = integer_arguments[4].unwrap_or_else(|| {
                game.find_player_by_name(&target_name)
                    .map_or(-1, |target| target.shape().get_direction())
            });
            let range = integer_arguments[5].unwrap_or(2);
            if !(0..=(i32::MAX - 1) / 2).contains(&range) {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            if let Some(target_id) = game
                .find_player_by_name(&target_name)
                .map(CPlayer::player_id)
            {
                let _ = game.change_player_region(
                    target_id,
                    target_region_id,
                    tile_x,
                    tile_y,
                    direction,
                    0,
                    range,
                    0,
                    runtime,
                );
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            }
            let mut request = CMessage::new(0x0005_ff11);
            request.add_long(player_id);
            request.base_mut().add(&target_name);
            request.add_byte(0);
            request.add_long(target_region_id);
            request.add_long(tile_x);
            request.add_long(tile_y);
            match request.send(game, false) {
                Ok(1) => Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 }),
                _ => Some(ScriptFunctionDispatchOutcome::Invalid),
            }
        }
        SCRIPT_FUNCTION_SET_PLAYER_REGION_EX => {
            let (Some(target_name), Some(target_region_id), Some(tile_x), Some(tile_y)) = (
                string_arguments[0].filter(|name| !name.is_empty()),
                integer_arguments[1],
                integer_arguments[2],
                integer_arguments[3],
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let player_ids = game.script_players_around_name(target_name);
            for target_id in player_ids {
                let direction = game
                    .find_player(target_id)
                    .map_or(0, |player| player.shape().get_direction());
                let _ = game.change_player_region(
                    target_id,
                    target_region_id,
                    tile_x,
                    tile_y,
                    direction,
                    0,
                    0,
                    0,
                    runtime,
                );
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_KICK_PLAYER_EX => {
            let Some(target_name) = string_arguments[0]
                .filter(|name| !name.is_empty() && name.len() <= u8::MAX as usize)
            else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            if script_player_id.is_none() {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let traversal = game.kick_players_around_name(target_name);
            match traversal.outcome {
                GameKickAroundOutcome::TargetMissing => {
                    let mut request = CMessage::new(0x0005_ff08);
                    request.add_long(player_id);
                    request.base_mut().add(target_name);
                    request.add_byte(0);
                    match request.send(game, false) {
                        Ok(1) => Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 }),
                        _ => Some(ScriptFunctionDispatchOutcome::Invalid),
                    }
                }
                GameKickAroundOutcome::Completed => {
                    let count = traversal.matched_player_ids.len() as i32;
                    let count_text = count.to_string().into_bytes();
                    let text = format_legacy_text_fields(
                        game.get_string_by_id(b"GS0030"),
                        &[&count_text],
                        0xff,
                    );
                    let mut response = CMessage::new(0x000b_f806);
                    response.add_long(-1);
                    response.add_long(0);
                    response.base_mut().add(&text);
                    response.add_byte(0);
                    let _ = response.send_to_player(game.net_server(), player_id);
                    Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
                }
                _ => Some(ScriptFunctionDispatchOutcome::Invalid),
            }
        }
        SCRIPT_FUNCTION_CREATE_FACTION => {
            let (Some(required_level), Some(required_goods), Some(required_money), Some(country)) = (
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                string_arguments[1].filter(|value| !value.is_empty()),
                integer_arguments[2].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[3].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            if country as u8 == 0 {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            }
            if let Ok((_, _)) =
                country_war_script_caller_gate(game, script_player_id, script_npc_id)
            {
                game.start_script_faction_creation(
                    player_id,
                    required_level,
                    required_goods,
                    required_money,
                    country as u8,
                    runtime.now_milliseconds(),
                );
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_APPLY_JOIN_FACTION => {
            let Some(required_level) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            if let Ok((_, _)) =
                country_war_script_caller_gate(game, script_player_id, script_npc_id)
            {
                game.start_script_faction_application(
                    player_id,
                    required_level,
                    runtime.now_milliseconds(),
                );
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_QUIT_JOIN_FACTION => {
            if country_war_script_caller_gate(game, script_player_id, script_npc_id).is_ok() {
                let mut request = CMessage::new(0x0006_0109);
                request.add_long(player_id);
                let _ = request.send(game, false);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_OBTAIN_TAX_PAYMENT | SCRIPT_FUNCTION_ADJUST_TAX_RATE => {
            if let Ok((_, _)) =
                country_war_script_caller_gate(game, script_player_id, script_npc_id)
                && let Some(region_id) = game
                    .find_player(player_id)
                    .and_then(CPlayer::server_region_id)
            {
                let kind = if function_id == SCRIPT_FUNCTION_OBTAIN_TAX_PAYMENT {
                    RegionTaxSessionKind::ObtainPayment
                } else {
                    RegionTaxSessionKind::AdjustRate
                };
                let _ = game.request_script_region_tax_operation(player_id, region_id, kind);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_GET_TOTAL_TAX_PAYMENT | SCRIPT_FUNCTION_GET_TODAY_TAX_PAYMENT => {
            let value = script_region_id
                .and_then(|region_id| {
                    game.script_region_tax_value(
                        region_id,
                        function_id == SCRIPT_FUNCTION_GET_TODAY_TAX_PAYMENT,
                    )
                })
                .unwrap_or_default();
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: value as i32,
            })
        }
        SCRIPT_FUNCTION_SET_TOTAL_TAX_PAYMENT | SCRIPT_FUNCTION_SET_TODAY_TAX_PAYMENT => {
            if let (Some(region_id), Some(value)) = (
                script_region_id,
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) {
                let _ = game.set_script_region_tax_value(
                    region_id,
                    function_id == SCRIPT_FUNCTION_SET_TODAY_TAX_PAYMENT,
                    value as u32,
                );
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_UPGRADE_FACTION => {
            if country_war_script_caller_gate(game, script_player_id, script_npc_id).is_ok() {
                let faction_id = game
                    .find_player(player_id)
                    .map(|player| player.faction_id())
                    .unwrap_or_default();
                if faction_id > 0 {
                    let mut snapshot = Vec::new();
                    if game.find_player(player_id).is_some_and(|player| {
                        game.encode_player_game_save(player, &mut snapshot, runtime)
                    }) {
                        let mut request = CMessage::new(0x0006_0126);
                        request.add_long(faction_id);
                        request.add_long(player_id);
                        request.base_mut().add(&snapshot);
                        let _ = request.send(game, false);
                    }
                }
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_GET_CITY_GATE_STATE => {
            let (Some(region_id), Some(gate_id)) = (
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[1].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: -1 });
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.script_city_gate_state(region_id, gate_id),
            })
        }
        SCRIPT_FUNCTION_OPERATOR_CITY_GATE | SCRIPT_FUNCTION_OPERATE_CITY_GATE => {
            let (Some(region_id), Some(gate_id), Some(operation)) = (
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[1].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[2].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            if game.find_player(player_id).is_none() {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            }
            let current_state = game.script_city_gate_state(region_id, gate_id);
            if current_state < 0 {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            }
            let notice_id =
                if function_id == SCRIPT_FUNCTION_OPERATOR_CITY_GATE && current_state == 2 {
                    Some(b"GS0197".as_slice())
                } else if operation == 1 && current_state == 1 {
                    Some(b"GS0198".as_slice())
                } else if operation == 0 && current_state == 0 {
                    Some(b"GS0199".as_slice())
                } else if operation == 1
                    && !game.script_city_gate_can_close(region_id, gate_id, runtime)
                {
                    Some(b"GS0200".as_slice())
                } else {
                    None
                };
            if let Some(notice_id) = notice_id {
                let text = game.get_string_by_id(notice_id);
                let _ = colored_player_notice_message(0xffff_ffff, 0xffff_0000, text)
                    .send_to_player(game.net_server(), player_id);
            } else if function_id == SCRIPT_FUNCTION_OPERATE_CITY_GATE {
                let _ = game.operate_script_city_gate(region_id, gate_id, operation, runtime);
            } else {
                let mut request = CMessage::new(0x0006_012f);
                request.add_long(player_id);
                request.add_long(region_id);
                request.add_long(gate_id);
                request.add_long(operation);
                let _ = request.send(game, false);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_FACTION_DECLARE_WAR => {
            if country_war_script_caller_gate(game, script_player_id, script_npc_id).is_ok()
                && game
                    .find_player(player_id)
                    .is_some_and(|player| player.faction_id() > 0)
            {
                game.start_script_faction_war_declaration(player_id, runtime.now_milliseconds());
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_GET_PLAYER_ALL_PROPERTIES => {
            if let (Some(requester_id), Some(target_name)) = (script_player_id, string_arguments[0])
                && let Some(snapshot) = game
                    .find_player_by_name(target_name)
                    .map(CPlayer::all_properties_diagnostic_snapshot)
            {
                let mut summary_arguments = Vec::with_capacity(16);
                summary_arguments.push(ScriptDiagnosticFormatArgument::Bytes(&snapshot.name));
                summary_arguments.extend(
                    snapshot
                        .summary_words
                        .iter()
                        .copied()
                        .map(ScriptDiagnosticFormatArgument::Word),
                );
                let base_arguments: Vec<_> = snapshot
                    .base_combat_words
                    .iter()
                    .copied()
                    .map(ScriptDiagnosticFormatArgument::Word)
                    .collect();
                let current_arguments: Vec<_> = snapshot
                    .current_combat_words
                    .iter()
                    .copied()
                    .map(ScriptDiagnosticFormatArgument::Word)
                    .collect();
                for (string_id, color, arguments) in [
                    (b"GS0186".as_slice(), 0xffff_ff00, summary_arguments),
                    (b"GS0187".as_slice(), 0xffff_ffff, base_arguments),
                    (b"GS0188".as_slice(), 0xffff_00ff, current_arguments),
                ] {
                    let text =
                        format_script_diagnostic(game.get_string_by_id(string_id), &arguments);
                    let _ = colored_player_notice_message(color, 0, &text)
                        .send_to_player(game.net_server(), requester_id);
                }
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_KICK_PLAYER => {
            let Some(requester_id) = script_player_id else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let Some(target_name) =
                string_arguments[0].filter(|name| !name.is_empty() && name.len() <= 23)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            if game.kick_player_by_name(target_name).is_some() {
                let text = format_legacy_text_fields(
                    game.get_string_by_id(b"GS0025"),
                    &[target_name],
                    0xff,
                );
                let _ = colored_player_notice_message(0xffff_ffff, 0, &text)
                    .send_to_player(game.net_server(), requester_id);
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 1 });
            }

            let mut request = CMessage::new(0x0005_ff07);
            request.add_long(requester_id);
            request.base_mut().add(target_name);
            request.add_byte(0);
            let _ = request.send(game, false);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_BAN_PLAYER => {
            let (Some(requester_id), Some(target_name), Some(minutes)) = (
                script_player_id,
                string_arguments[0]
                    .filter(|name| !name.is_empty() && name.len() <= 49 && !name.contains(&b'\'')),
                integer_arguments[1]
                    .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR && *value != 0),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let mut request = CMessage::new(0x0005_ff12);
            request.add_long(requester_id);
            request.base_mut().add(target_name);
            request.add_byte(0);
            request.add_long(minutes);
            let _ = request.send(game, false);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_SILENCE_PLAYER => {
            let Some(requester_id) = script_player_id.filter(|player_id| *player_id > 0) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let Some(target_name) = string_arguments[0]
                .filter(|name| !name.is_empty() && name.len() <= u8::MAX as usize)
            else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let minutes = integer_arguments[1]
                .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                .unwrap_or(1);
            if minutes < 0 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            if game
                .silence_player_by_name(target_name, minutes, || runtime.now_milliseconds())
                .is_some()
            {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            }

            let mut request = CMessage::new(0x0005_ff0c);
            request.add_long(requester_id);
            request.base_mut().add(target_name);
            request.add_byte(0);
            request.add_long(minutes);
            let _ = request.send(game, false);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_GET_MAP_INFO => {
            let (Some(region_id), Some(tile_x), Some(tile_y)) = (
                script_region_id,
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[1].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: -1 });
            };
            let legacy_return = game
                .find_region(region_id)
                .and_then(|owner| owner.base().region.get_cell(tile_x, tile_y).ok().flatten())
                .map_or(-1, |cell| {
                    if cell.city_war_marker() == 1 {
                        2
                    } else {
                        match cell.security().value() {
                            2 => 3,
                            1 => 1,
                            _ => 0,
                        }
                    }
                });
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_FORCE_MOVE => {
            let target_name = string_arguments[0].filter(|name| name.len() < 24);
            let integer = |index: usize| {
                integer_arguments[index].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            };
            if let (Some(target_name), Some(x), Some(y), Some(duration_ms)) =
                (target_name, integer(1), integer(2), integer(3))
                && let Some(target_id) = game
                    .find_player_by_name(target_name)
                    .map(CPlayer::player_id)
            {
                let _ = game.force_move_script_player(target_id, x, y, duration_ms as u32, runtime);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_CHANGE_MONEY_BY_NAME | SCRIPT_FUNCTION_SET_MONEY_BY_NAME => {
            let (Some(target_name), Some(value)) = (
                string_arguments[0],
                integer_arguments[1].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let target_id = if target_name.is_empty() {
                game.find_player(player_id).map(CPlayer::player_id)
            } else {
                game.find_player_by_name(target_name)
                    .map(CPlayer::player_id)
            };
            let legacy_return = target_id.map_or(0, |target_id| {
                let current = game
                    .find_player(target_id)
                    .expect("money target найден непосредственно перед mutation")
                    .money();
                let requested = if function_id == SCRIPT_FUNCTION_CHANGE_MONEY_BY_NAME {
                    let changed = current.wrapping_add(value as u32);
                    if (changed as i32) < 0 { 0 } else { changed }
                } else {
                    value.max(0) as u32
                };
                let _ = game.set_script_player_money(target_id, requested, runtime);
                1
            });
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_GET_MONEY_BY_NAME
        | SCRIPT_FUNCTION_GET_FACTION_ID_BY_PLAYER_NAME
        | SCRIPT_FUNCTION_IS_FACTION_MASTER_BY_PLAYER_NAME => {
            let Some(target_name) = string_arguments[0] else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let target = if target_name.is_empty() {
                game.find_player(player_id)
            } else {
                game.find_player_by_name(target_name)
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: target.map_or(0, |player| match function_id {
                    SCRIPT_FUNCTION_GET_MONEY_BY_NAME => player.money() as i32,
                    SCRIPT_FUNCTION_GET_FACTION_ID_BY_PLAYER_NAME => player.faction_id(),
                    SCRIPT_FUNCTION_IS_FACTION_MASTER_BY_PLAYER_NAME => {
                        i32::from(player.is_faction_master())
                    }
                    _ => unreachable!("faction identity selector отфильтрован match-arm"),
                }),
            })
        }
        SCRIPT_FUNCTION_CHANGE_MONEY_BY_ID | SCRIPT_FUNCTION_SET_MONEY_BY_ID => {
            let (Some(requested_player_id), Some(value)) = (
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[1].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let target_id = if requested_player_id == 0 {
                game.find_player(player_id).map(CPlayer::player_id)
            } else {
                game.find_player(requested_player_id)
                    .map(CPlayer::player_id)
            };
            let legacy_return = target_id.map_or(0, |target_id| {
                let current = game
                    .find_player(target_id)
                    .expect("money target найден непосредственно перед mutation")
                    .money();
                let requested = if function_id == SCRIPT_FUNCTION_CHANGE_MONEY_BY_ID {
                    let changed = current.wrapping_add(value as u32);
                    if (changed as i32) < 0 { 0 } else { changed }
                } else {
                    value.max(0) as u32
                };
                let _ = game.set_script_player_money(target_id, requested, runtime);
                1
            });
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_GET_MONEY_BY_ID => {
            let Some(requested_player_id) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let target = if requested_player_id == 0 {
                game.find_player(player_id)
            } else {
                game.find_player(requested_player_id)
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: target.map_or(0, |player| player.money() as i32),
            })
        }
        SCRIPT_FUNCTION_GET_PLAYER_ALL_VARIABLES => {
            if let (Some(requester_id), Some(target_name)) = (script_player_id, string_arguments[0])
            {
                let lines = game
                    .find_player_by_name(target_name)
                    .map(|target| {
                        let mut lines = Vec::new();
                        for variable in target.variable_list().variables() {
                            match &variable.value {
                                GameVariableValue::Integer(value) => lines.push(
                                    format_script_variable_line(&variable.name, None, *value),
                                ),
                                GameVariableValue::String(value) => {
                                    let mut line = variable.name.clone();
                                    line.extend_from_slice(b" = \"");
                                    line.extend_from_slice(value);
                                    line.push(b'"');
                                    line.truncate(0x18fff);
                                    lines.push(line);
                                }
                                GameVariableValue::IntegerArray(values) => {
                                    lines.extend(values.iter().enumerate().map(
                                        |(index, value)| {
                                            format_script_variable_line(
                                                &variable.name,
                                                Some(index),
                                                *value,
                                            )
                                        },
                                    ));
                                }
                            }
                        }
                        lines
                    })
                    .unwrap_or_default();
                for line in lines {
                    let _ = colored_player_notice_message(0xff00_ff00, 0, &line)
                        .send_to_player(game.net_server(), requester_id);
                }
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_DELETE_SKILL => {
            let (Some(target_name), Some(skill_name)) = (string_arguments[0], string_arguments[1])
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: -1 });
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.delete_script_player_skill(player_id, target_name, skill_name),
            })
        }
        SCRIPT_FUNCTION_SET_SKILL_LEVEL => {
            let (Some(target_name), Some(skill_name), Some(level)) = (
                string_arguments[0],
                string_arguments[1],
                integer_arguments[2].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: -1 });
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.set_script_player_skill_level(
                    player_id,
                    target_name,
                    skill_name,
                    level,
                ),
            })
        }
        SCRIPT_FUNCTION_GET_SKILL_LEVEL => {
            if argument_count != 2 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let (Some(target_name), Some(skill_name)) = (string_arguments[0], string_arguments[1])
            else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            if target_name.len() > 49 || skill_name.is_empty() || skill_name.len() > 255 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.script_player_skill_level(player_id, target_name, skill_name),
            })
        }
        SCRIPT_FUNCTION_ADD_SKILL => {
            if !(2..=3).contains(&argument_count) {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let (Some(target_name), Some(skill_name)) = (string_arguments[0], string_arguments[1])
            else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let level = if argument_count == 2 {
                1
            } else {
                let Some(level) =
                    integer_arguments[2].filter(|level| *level != SCRIPT_INT_PARAMETER_ERROR)
                else {
                    return Some(ScriptFunctionDispatchOutcome::Invalid);
                };
                level
            };
            if target_name.len() > 49
                || skill_name.is_empty()
                || skill_name.len() > 255
                || !(1..=i16::MAX as i32).contains(&level)
            {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.add_script_player_skill(
                    player_id,
                    target_name,
                    skill_name,
                    level,
                    runtime,
                ),
            })
        }
        SCRIPT_FUNCTION_OPEN_NPC_SHOP => {
            if argument_count != 1
                || !script_player_npc_caller_exists(game, script_player_id, script_npc_id)
            {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let Some(trade_name) = string_arguments[0].filter(|name| !name.is_empty()) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let report = game.open_script_npc_shop(
                player_id,
                script_npc_id.expect("живой NPC проверен до открытия магазина"),
                trade_name,
            );
            Some(if report.outcome == ScriptNpcShopOpenOutcome::Opened {
                ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 }
            } else {
                ScriptFunctionDispatchOutcome::Invalid
            })
        }
        SCRIPT_FUNCTION_OPEN_DEPOT => {
            if argument_count != 0 || script_player_id.is_none() {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let report = game.open_script_depot(player_id);
            Some(if report.outcome == ScriptDepotOpenOutcome::Opened {
                ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 }
            } else {
                ScriptFunctionDispatchOutcome::Invalid
            })
        }
        SCRIPT_FUNCTION_GET_TEAM_NUM => {
            if argument_count != 0 || script_player_id.is_none() {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.script_team_member_count(player_id),
            })
        }
        SCRIPT_FUNCTION_IS_TEAM_CAPTAIN => Some(ScriptFunctionDispatchOutcome::Handled {
            legacy_return: script_player_id
                .map_or(-1, |player_id| game.script_is_team_captain(player_id)),
        }),
        SCRIPT_FUNCTION_IS_TEAMMATES_AROUND_ME => {
            let check_type = integer_arguments[0]
                .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                .unwrap_or_default();
            let radius = integer_arguments[1]
                .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                .unwrap_or(3);
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: script_player_id.map_or(0, |player_id| {
                    i32::from(game.script_are_teammates_around(player_id, check_type, radius))
                }),
            })
        }
        SCRIPT_FUNCTION_SET_REGION_FOR_TEAM => {
            let integer =
                |index: usize| integer_arguments[index].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.script_set_region_for_team(
                    player_id,
                    integer(0),
                    integer(1),
                    integer(2),
                    integer(3),
                    integer(4),
                    integer(5),
                    runtime,
                ),
            })
        }
        SCRIPT_FUNCTION_SET_TEAM_REGION => {
            let integer =
                |index: usize| integer_arguments[index].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            if integer(0) == SCRIPT_INT_PARAMETER_ERROR {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            }
            let Some(player_id) = script_player_id else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            game.script_set_team_region(
                player_id,
                integer(0),
                integer(1),
                integer(2),
                integer(3),
                integer(4),
                integer(5),
                runtime,
            );
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_ADD_FU_MO_PROPERTY => {
            let property_type = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let value = integer_arguments[1].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: if value == 0 {
                    0
                } else {
                    script_player_id.map_or(-1, |player_id| {
                        game.add_script_selected_fu_mo_property(player_id, property_type, value)
                    })
                },
            })
        }
        SCRIPT_FUNCTION_CHANGE_REGION => {
            let Some(target_region_id) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                // Исходный диспетчер считает отсутствующий или невычисленный
                // первый аргумент успешным пустым действием и продолжает
                // вызывающий сценарий.
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let current_direction = game
                .find_player(player_id)
                .map(|player| player.shape().get_direction());
            let Some(current_direction) = current_direction else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let integer = |index: usize| {
                integer_arguments[index].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            };
            let (tile_x, tile_y) = match (integer(1), integer(2)) {
                (Some(tile_x), Some(tile_y)) => (tile_x, tile_y),
                _ => (-1, -1),
            };
            let direction = integer(3).unwrap_or(current_direction);
            let use_goods = integer(4).unwrap_or_default();
            let range = integer(5).unwrap_or(2);
            let carriage_distance = integer(6).unwrap_or_default();
            let _ = game.change_player_region(
                player_id,
                target_region_id,
                tile_x,
                tile_y,
                direction,
                use_goods,
                range,
                carriage_distance,
                runtime,
            );
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_RE_LIVE => {
            let Some(relive_type) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                // Исходный диспетчер не вызывает `OnRelive`, если первый
                // аргумент не вычислен, но всё равно возвращает сценарный ноль.
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            if let Some(player_id) = script_player_id {
                let _ = game.relive_player(player_id, relive_type, runtime);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_ADD_STATE => {
            let (Some(state_id), Some(value1), Some(value2)) = (
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[1].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[2].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let legacy_return = script_player_id.map_or(0, |player_id| {
                game.add_script_move_state(player_id, state_id, value1, value2)
            });
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_GET_STATES_NUMBER => {
            let Some(state_id) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: script_player_id.map_or(0, |player_id| {
                    game.script_move_state_count(player_id, state_id)
                }),
            })
        }
        SCRIPT_FUNCTION_GET_COUNTRY => {
            if argument_count != 0 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let Some(player) = game.find_player(player_id) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: i32::from(player.country()),
            })
        }
        SCRIPT_FUNCTION_GET_ONLINE_PLAYERS => {
            if argument_count != 0 || game.find_player(player_id).is_none() {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let local_count = game.player_count().min(i32::MAX as u32) as i32;
            let mut request = CMessage::new(0x0005_ff01);
            request.add_long(player_id);
            request.add_long(script_id);
            let _ = request.send(game, false);
            Some(ScriptFunctionDispatchOutcome::Yielded {
                legacy_return: local_count,
            })
        }
        SCRIPT_FUNCTION_LIST_ONLINE_GM => {
            if argument_count != 0 || game.find_player(player_id).is_none() {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let _ = publish_local_online_gm_list(game, player_id);
            let mut request = CMessage::new(0x0005_ff14);
            request.add_long(player_id);
            let _ = request.send(game, false);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_LIST_SILENCE_PLAYER => {
            if argument_count != 0 || game.find_player(player_id).is_none() {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let mut request = CMessage::new(0x0005_ff0f);
            request.add_long(player_id);
            let _ = request.send(game, false);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_SAVE_ALL_PLAYERS => {
            if argument_count != 0 || game.find_player(player_id).is_none() {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let mut request = CMessage::new(0x0005_ff13);
            request.add_long(player_id);
            let _ = request.send(game, false);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_LIST_ONLINE_PLAYER => {
            if argument_count != 0 || game.find_player(player_id).is_none() {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            for name in game.script_region_player_names(player_id) {
                let mut response = CMessage::new(0x000b_f806);
                response.add_long(-1);
                response.add_long(0);
                response.base_mut().add(&name);
                response.add_byte(0);
                let _ = response.send_to_player(game.net_server(), player_id);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_LIST_BANNED_PLAYER => {
            if argument_count != 0 || game.find_player(player_id).is_none() || script_id <= 0 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let mut request = CMessage::new(0x0005_ff17);
            request.add_long(player_id);
            request.add_long(script_id);
            Some(match request.send(game, false) {
                Ok(1) => ScriptFunctionDispatchOutcome::Yielded { legacy_return: -1 },
                _ => ScriptFunctionDispatchOutcome::Handled { legacy_return: -1 },
            })
        }
        SCRIPT_FUNCTION_KICK_ALL => {
            if argument_count != 0 || game.find_player(player_id).is_none() {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let mut request = CMessage::new(0x0005_ff0a);
            request.add_long(player_id);
            let _ = request.send(game, false);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_KICK_MAP => {
            let Some(region_id) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            if game.find_region(region_id).is_some() {
                let _ = game.kick_players_in_region_except(region_id, player_id);
            } else if game.find_player(player_id).is_some() {
                let mut request = CMessage::new(0x0005_ff0b);
                request.add_long(player_id);
                request.add_long(region_id);
                let _ = request.send(game, false);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_OPEN_CI_QING_PAGE => {
            if argument_count != 0 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let _ = game.open_script_ci_qing_page(player_id);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_PUSH_ITEM_TO_CI_QING => {
            if let Some(original_name) = string_arguments[0].filter(|value| !value.is_empty()) {
                let _ = game.push_script_ci_qing_item(player_id, original_name, runtime);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_GET_COPY_NUMBER => {
            let Some(increment_flag) = integer_arguments[0].filter(|value| matches!(value, 0 | 1))
            else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            if argument_count < 1 || game.find_player(player_id).is_none() || script_id <= 0 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let mut request = CMessage::new(0x0005_fd0b);
            request.add_long(player_id);
            request.add_long(script_id);
            request.add_long(increment_flag);
            match request.send(game, false) {
                Ok(1) => Some(ScriptFunctionDispatchOutcome::Yielded { legacy_return: 0 }),
                _ => Some(ScriptFunctionDispatchOutcome::Invalid),
            }
        }
        SCRIPT_FUNCTION_POST_PLAYER_INFO
        | SCRIPT_FUNCTION_POST_REGION_INFO
        | SCRIPT_FUNCTION_POST_WORLD_INFO => {
            let default_background = if function_id == SCRIPT_FUNCTION_POST_PLAYER_INFO {
                -16_777_216
            } else {
                0
            };
            let Some((text, color, background)) = script_notice_arguments(
                argument_count,
                &integer_arguments,
                &string_arguments,
                default_background,
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            if player_id <= 0 || game.find_player(player_id).is_none() {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            }

            if function_id == SCRIPT_FUNCTION_POST_PLAYER_INFO {
                return Some(
                    if colored_text_message(0x000b_f811, color, background, text)
                        .send_to_player(game.net_server(), player_id)
                        != 0
                    {
                        ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 }
                    } else {
                        ScriptFunctionDispatchOutcome::Invalid
                    },
                );
            }
            let Some(region_id) = game
                .find_player(player_id)
                .and_then(|player| player.server_region_id())
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            if function_id == SCRIPT_FUNCTION_POST_REGION_INFO {
                let Some(region) = game.find_region(region_id) else {
                    return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
                };
                return Some(
                    if colored_player_notice_message(color, background, text).send_to_region(
                        Some(region.base()),
                        None,
                        game,
                    ) != 0
                    {
                        ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 }
                    } else {
                        ScriptFunctionDispatchOutcome::Invalid
                    },
                );
            }

            let mut request = CMessage::new(0x0005_ff0e);
            request.add_long(player_id);
            request.base_mut().add(text);
            request.add_byte(0);
            request.add_long(color as i32);
            request.add_long(background as i32);
            request.add_long(i32::from(color as i32 == -2));
            request.add_long(0);
            match request.send(game, false) {
                Ok(delivery) if delivery != 0 => {
                    Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
                }
                Ok(_) | Err(_) => Some(ScriptFunctionDispatchOutcome::Invalid),
            }
        }
        SCRIPT_FUNCTION_SCRIPT_IS_RUNNING | SCRIPT_FUNCTION_REMOVE_SCRIPT => {
            if function_id == SCRIPT_FUNCTION_SCRIPT_IS_RUNNING && argument_count != 2 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let requested_player_id = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            if requested_player_id == SCRIPT_INT_PARAMETER_ERROR {
                return Some(if function_id == SCRIPT_FUNCTION_SCRIPT_IS_RUNNING {
                    ScriptFunctionDispatchOutcome::Invalid
                } else {
                    ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 }
                });
            }
            let Some(path) = string_arguments[1].filter(|path| !path.is_empty()) else {
                return Some(if function_id == SCRIPT_FUNCTION_SCRIPT_IS_RUNNING {
                    ScriptFunctionDispatchOutcome::Invalid
                } else {
                    ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 }
                });
            };
            let target_player_id = if requested_player_id > 0 {
                requested_player_id
            } else {
                player_id
            };
            let (current_script_id, current_script_path) = if target_player_id == player_id {
                (script_id, script_path)
            } else {
                (0, &[][..])
            };
            if function_id == SCRIPT_FUNCTION_SCRIPT_IS_RUNNING {
                let legacy_return = i32::from(game.player_script_is_running(
                    target_player_id,
                    path,
                    current_script_id,
                    current_script_path,
                ));
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return });
            }
            let current_removed = game.remove_player_scripts(
                target_player_id,
                path,
                current_script_id,
                current_script_path,
            );
            return Some(if current_removed {
                ScriptFunctionDispatchOutcome::Terminated { legacy_return: 0 }
            } else {
                ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 }
            });
        }
        SCRIPT_FUNCTION_RGB => {
            let red = integer_arguments[0].unwrap_or_default() as u32 & 0xff;
            let green = integer_arguments[1].unwrap_or_default() as u32 & 0xff;
            let blue = integer_arguments[2].unwrap_or_default() as u32 & 0xff;
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: (red | green << 8 | blue << 16) as i32,
            })
        }
        SCRIPT_FUNCTION_ACTIVITY_LOG => {
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_ADD_QUEST
        | SCRIPT_FUNCTION_COMPLETE_QUEST
        | SCRIPT_FUNCTION_DISBAND_QUEST => {
            let Some(target_player_id) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let quest_id = integer_arguments[1].unwrap_or(SCRIPT_INT_PARAMETER_ERROR) as u16;
            let target_player_id = if target_player_id == 0 {
                player_id
            } else {
                target_player_id
            };
            if function_id == SCRIPT_FUNCTION_ADD_QUEST {
                game.add_script_player_quest(target_player_id, quest_id);
            } else if function_id == SCRIPT_FUNCTION_COMPLETE_QUEST {
                game.complete_script_player_quest(target_player_id, quest_id);
            } else {
                game.remove_script_player_quest(target_player_id, quest_id);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 1 })
        }
        SCRIPT_FUNCTION_GET_QUEST_STATE => {
            let Some(target_player_id) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: -1 });
            };
            let target_player_id = if target_player_id == 0 {
                player_id
            } else {
                target_player_id
            };
            let quest_id = integer_arguments[1].unwrap_or(SCRIPT_INT_PARAMETER_ERROR) as u16;
            let legacy_return = game
                .find_player(target_player_id)
                .map(|player| player.quest_state(quest_id))
                .unwrap_or(-1);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_UPDATE_QUEST_POSITION => {
            let Some(target_player_id) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let (Some(region_id), Some(tile_x), Some(tile_y)) = (
                integer_arguments[2].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[3].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[4].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let target_player_id = if target_player_id == 0 {
                player_id
            } else {
                target_player_id
            };
            let quest_id = integer_arguments[1].unwrap_or(SCRIPT_INT_PARAMETER_ERROR) as u16;
            game.update_script_player_quest_position(
                target_player_id,
                quest_id,
                region_id,
                tile_x,
                tile_y,
            );
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_IS_QUEST_ENABLED => {
            let legacy_return = game
                .find_player(player_id)
                .map(|player| i32::from(player.quest_enabled()))
                .unwrap_or(0);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_SET_QUEST_ENABLED => {
            let Some(enabled) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            if game.find_player(player_id).is_some() {
                game.set_script_player_quest_enabled(player_id, enabled != 0);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_QUEST_TIME_BEGIN => {
            let Some(time_limit) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            if game.find_player(player_id).is_none() {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            }
            game.begin_script_player_quest_time(
                player_id,
                chrono::Local::now().timestamp() as i32,
                time_limit,
            );
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 1 })
        }
        SCRIPT_FUNCTION_QUEST_TIME_CLEAR => {
            if game.find_player(player_id).is_some() {
                game.clear_script_player_quest_time(player_id);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_GET_QUEST_TIME => {
            let now_seconds = chrono::Local::now().timestamp() as i32;
            let legacy_return = game
                .find_player(player_id)
                .map(|player| player.quest_time_remaining(now_seconds))
                .unwrap_or(0);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_ADD_CARRIAGE => {
            let Some(original_name) = string_arguments[0] else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let legacy_return = game.add_script_player_carriage(
                player_id,
                original_name,
                string_arguments[1],
                runtime,
            );
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_DELETE_CARRIAGE => {
            let legacy_return = game.delete_script_player_carriage(player_id);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_GET_CARRIAGE_DISTANCE => {
            let legacy_return = game.script_player_carriage_distance(player_id);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_GET_CARRIAGE_INDEX => {
            let legacy_return = game.script_player_carriage_index(player_id);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_OPEN_SYNTHESIS => {
            if let Some(player_id) = script_player_id {
                let _ = game.open_script_synthesis_page(player_id);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_CHECK_GOODS => {
            let Some(name) = string_arguments[0].filter(|name| !name.is_empty()) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let goods_index = game
                .goods_factory()
                .query_goods_id_by_original_name(Some(name));
            let amount = game
                .find_player(player_id)
                .map(|player| {
                    player
                        .packet()
                        .base()
                        .traversing_goods()
                        .filter(|goods| goods.base_properties_index() == goods_index)
                        .fold(0u32, |total, goods| total.wrapping_add(goods.amount()))
                })
                .unwrap_or_default();
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: amount.min(i32::MAX as u32) as i32,
            })
        }
        SCRIPT_FUNCTION_GET_GOODS_NUMBER => Some(ScriptFunctionDispatchOutcome::Handled {
            legacy_return: game.script_packet_goods_number(player_id),
        }),
        SCRIPT_FUNCTION_GET_FREE_SPACE => {
            let package = match integer_arguments[0] {
                Some(SCRIPT_INT_PARAMETER_ERROR) => {
                    return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
                }
                Some(package) => package,
                None => 0,
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.script_container_free_space(player_id, package == 1),
            })
        }
        SCRIPT_FUNCTION_CHECK_DEPOT_GOODS => {
            let Some(name) = string_arguments[0].filter(|name| !name.is_empty()) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.script_depot_goods_amount(player_id, name),
            })
        }
        SCRIPT_FUNCTION_ADD_DEPOT_GOODS => {
            let Some(name) = string_arguments[0].filter(|name| !name.is_empty()) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let amount = match integer_arguments[1] {
                Some(SCRIPT_INT_PARAMETER_ERROR) => 1,
                Some(amount) if 0 < amount => amount as u32,
                Some(_) => {
                    return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
                }
                None => 1,
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.add_script_depot_goods(player_id, name, amount, runtime),
            })
        }
        SCRIPT_FUNCTION_DELETE_DEPOT_GOODS => {
            let (Some(name), Some(amount)) = (
                string_arguments[0].filter(|name| !name.is_empty()),
                integer_arguments[1]
                    .filter(|amount| *amount != SCRIPT_INT_PARAMETER_ERROR && 0 < *amount),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.delete_script_depot_goods(player_id, name, amount as u32),
            })
        }
        SCRIPT_FUNCTION_CHECK_DEPOT_SPACE => {
            let Some(name) = string_arguments[0].filter(|name| !name.is_empty()) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.script_depot_space_for_goods(player_id, name),
            })
        }
        SCRIPT_FUNCTION_GET_DEPOT_GOODS_NUMBER => Some(ScriptFunctionDispatchOutcome::Handled {
            legacy_return: game.script_depot_goods_number(player_id),
        }),
        SCRIPT_FUNCTION_GET_DEPOT_GOODS_FREE => {
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_DELETE_PLAYER_GOODS => {
            let (Some(player_name), Some(goods_name), Some(requested)) = (
                string_arguments[0].filter(|name| !name.is_empty() && name.len() <= 49),
                string_arguments[1].filter(|name| !name.is_empty() && name.len() <= 255),
                integer_arguments[2].filter(|value| *value > 0),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.delete_named_player_script_goods(
                    player_name,
                    goods_name,
                    requested as u32,
                    runtime,
                ),
            })
        }
        SCRIPT_FUNCTION_GET_CONTAINER_ITEM_TYPE => Some(ScriptFunctionDispatchOutcome::Handled {
            legacy_return: game.script_selected_container_item_type(player_id),
        }),
        SCRIPT_FUNCTION_UPGRADE_SELECTED_EQUIPMENT => {
            let Some(level_delta) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.upgrade_script_selected_equipment(
                    player_id,
                    level_delta,
                    runtime,
                ),
            })
        }
        SCRIPT_FUNCTION_GET_GOODS_PROPERTY_1 | SCRIPT_FUNCTION_GET_GOODS_PROPERTY_2 => {
            let Some(property) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: -1 });
            };
            let value_id = if function_id == SCRIPT_FUNCTION_GET_GOODS_PROPERTY_1 {
                1
            } else {
                2
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.script_selected_goods_property(player_id, property, value_id),
            })
        }
        SCRIPT_FUNCTION_GET_GOODS_PRICE => Some(ScriptFunctionDispatchOutcome::Handled {
            legacy_return: game.script_selected_goods_price(player_id),
        }),
        SCRIPT_FUNCTION_SET_GOODS_PROPERTY_1 | SCRIPT_FUNCTION_SET_GOODS_PROPERTY_2 => {
            let (Some(property), Some(modifier)) = (
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[1].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let value_id = if function_id == SCRIPT_FUNCTION_SET_GOODS_PROPERTY_1 {
                1
            } else {
                2
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.set_script_selected_goods_property(
                    player_id, property, value_id, modifier, runtime,
                ),
            })
        }
        SCRIPT_FUNCTION_RECREATE_GOODS_ADDON_PROPERTIES => {
            game.recreate_script_selected_goods_addons(player_id, runtime);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_DELETE_SPLIT_GOODS => {
            game.delete_script_selected_goods(player_id, runtime);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_OPEN_GOODS_CONTAINER => {
            if game.find_player(player_id).is_none() {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            if let Some(npc_id) = script_npc_id {
                let player = game.resolve_shape(ShapeIdentity {
                    object_type: SCRIPT_PLAYER_TYPE,
                    id: player_id,
                    ex_id: CGuid::GUID_INVALID,
                });
                let npc = game.resolve_shape(ShapeIdentity {
                    object_type: SCRIPT_NPC_TYPE,
                    id: npc_id,
                    ex_id: CGuid::GUID_INVALID,
                });
                if player
                    .zip(npc)
                    .is_some_and(|(player, npc)| npc.distance(player) > 8)
                {
                    let _ = colored_player_notice_message(
                        0xffff_ffff,
                        0,
                        game.get_string_by_id(b"GS0178"),
                    )
                    .send_to_player(game.net_server(), player_id);
                    return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
                }
            }
            let _ = game.open_script_goods_container(
                player_id,
                string_arguments[0].unwrap_or_default(),
                string_arguments[1].unwrap_or_default(),
            );
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_CHECK_SPACE => {
            let requested = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let package = integer_arguments[1].unwrap_or_default();
            let legacy_return = u32::try_from(requested)
                .ok()
                .and_then(|requested| {
                    game.find_player(player_id)
                        .map(|player| (player, requested))
                })
                .map_or(0, |(player, requested)| {
                    i32::from(if package == 1 {
                        player.depot().base().check_space(requested)
                    } else {
                        player.packet().check_space(requested)
                    })
                });
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_ADD_GOODS => {
            if argument_count > 4 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let Some(name) = string_arguments[0].filter(|name| !name.is_empty()) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let amount = integer_arguments[1].unwrap_or(1);
            if amount < 1 || game.find_player(player_id).is_none() {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            }
            let mut upgrade_level = integer_arguments[2].unwrap_or_default();
            if !(0..=100).contains(&upgrade_level) {
                upgrade_level = 0;
            }
            let particular_attribute = integer_arguments[3].unwrap_or_default();
            let goods_index = game
                .goods_factory()
                .query_goods_id_by_original_name(Some(name));
            let created = game.create_script_goods_batch(
                goods_index,
                amount as u32,
                upgrade_level,
                particular_attribute,
            );
            let mut encode = |goods: &CGoods| runtime.encode_goods_for_old_client(goods);
            if let Some((additions, _rejected)) =
                game.add_goods_to_player_packet(player_id, created, &mut encode)
            {
                for addition in &additions {
                    let _ = game.send_player_packet_addition(addition);
                }
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 1 })
        }
        SCRIPT_FUNCTION_DELETE_GOODS => {
            let Some(name) = string_arguments[0].filter(|name| !name.is_empty()) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let requested = integer_arguments[1].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let Ok(requested) = u32::try_from(requested) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            if requested == 0 {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            }
            let goods_index = game
                .goods_factory()
                .query_goods_id_by_original_name(Some(name));
            let legacy_return =
                game.delete_script_goods(player_id, goods_index, requested, runtime) as i32;
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return })
        }
        SCRIPT_FUNCTION_ADD_INFO => {
            let Some(text) = string_arguments[0] else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let color = integer_arguments[1].unwrap_or(-1) as u32;
            let background = integer_arguments[2].unwrap_or_default() as u32;
            if game.find_player(player_id).is_some() {
                let _ = colored_player_notice_message(color, background, text)
                    .send_to_player(game.net_server(), player_id);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_GAME_MESSAGE => {
            if let Some(text) = string_arguments[0]
                && game.find_player(player_id).is_some()
            {
                let mut message = CMessage::new(0x000b_f808);
                message.base_mut().add(text);
                message.add_byte(0);
                message.add_long(integer_arguments[1].unwrap_or(SCRIPT_INT_PARAMETER_ERROR));
                message.add_long(script_id);
                let _ = message.send_to_player(game.net_server(), player_id);
            }
            Some(ScriptFunctionDispatchOutcome::Yielded { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_OPEN_NEW_HELP_WINDOW => {
            if game.find_player(player_id).is_some() {
                let _ = CMessage::new(0x000b_f812).send_to_player(game.net_server(), player_id);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_GET_GOODS_PROPERTY => {
            let (Some(name), Some(property), Some(value_id)) = (
                string_arguments[0],
                integer_arguments[1].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
                integer_arguments[2].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            if game.find_player(player_id).is_none() {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            }
            let goods_index = game
                .goods_factory()
                .query_goods_id_by_original_name(Some(name));
            let Some(goods) = game.create_goods_core(goods_index) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: goods.addon_property_value(
                    game.goods_factory(),
                    property,
                    value_id as u32,
                ),
            })
        }
        SCRIPT_FUNCTION_CHANGE_COUNTRY => {
            let Some(country) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            if game.find_player(player_id).is_some() {
                let mut request = CMessage::new(0x0006_0301);
                request.add_long(player_id);
                request.add_byte(country as u8);
                let _ = request.send(game, false);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_GET_CONTRIBUTION => Some(ScriptFunctionDispatchOutcome::Handled {
            legacy_return: game.find_player(player_id).map_or(0, CPlayer::contribution),
        }),
        SCRIPT_FUNCTION_SET_CONTRIBUTION => {
            let Some(requested) =
                integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let Some((stored, fetch_power)) = game.find_player_mut(player_id).map(|player| {
                player.set_contribution(requested);
                (player.contribution(), player.fetch_power())
            }) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let mut message = CMessage::new(0x000b_f724);
            message.add_long(requested);
            message.add_ulong(fetch_power);
            let _ = message.send_to_player(game.net_server(), player_id);
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: stored,
            })
        }
        SCRIPT_FUNCTION_GET_YUAN_BAO => Some(ScriptFunctionDispatchOutcome::Handled {
            legacy_return: game
                .find_player(player_id)
                .map_or(0, |player| player.yuan_bao() as i32),
        }),
        SCRIPT_FUNCTION_POST_COUNTRY_INFO => {
            let (Some(text), Some(country_id)) = (
                string_arguments[0],
                integer_arguments[1].filter(|value| (1..=4).contains(value)),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            if argument_count > 4 || text.len() > 255 || text.contains(&0) {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let Some(player_id) = script_player_id.filter(|player_id| *player_id > 0) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            let color = match integer_arguments[2] {
                Some(SCRIPT_INT_PARAMETER_ERROR) => {
                    return Some(ScriptFunctionDispatchOutcome::Invalid);
                }
                Some(color) => color,
                None => -1,
            };
            let background = match integer_arguments[3] {
                Some(SCRIPT_INT_PARAMETER_ERROR) => {
                    return Some(ScriptFunctionDispatchOutcome::Invalid);
                }
                Some(background) => background,
                None => 0,
            };
            let mut request = CMessage::new(0x0005_ff16);
            request.add_long(player_id);
            request.base_mut().add(text);
            request.add_byte(0);
            request.add_long(country_id);
            request.add_long(color);
            request.add_long(background);
            match request.send(game, false) {
                Ok(delivery) if delivery != 0 => {
                    Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
                }
                _ => Some(ScriptFunctionDispatchOutcome::Invalid),
            }
        }
        SCRIPT_FUNCTION_TALK_BOX | SCRIPT_FUNCTION_HELP | SCRIPT_FUNCTION_TALK_BOX_SMALL => {
            if argument_count != 1 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let Some(text) = string_arguments[0].filter(|text| text.len() < 0x5000) else {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            };
            if game.find_player(player_id).is_none() {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            let mut message = CMessage::new(if function_id == SCRIPT_FUNCTION_TALK_BOX {
                0x000b_f805
            } else {
                0x000b_f71c
            });
            message.add_long(script_id);
            message.base_mut().add(text);
            message.add_byte(0);
            if function_id == SCRIPT_FUNCTION_TALK_BOX {
                message.add_byte(1);
            }
            if message.send_to_player(game.net_server(), player_id) == 0 {
                return Some(ScriptFunctionDispatchOutcome::Invalid);
            }
            Some(ScriptFunctionDispatchOutcome::Yielded { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_ADD_GOODS_LOG => {
            let Some(name) = string_arguments[0] else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let log_type = integer_arguments[1].unwrap_or(14);
            let requested_amount = integer_arguments[2].unwrap_or(1);
            let goods_index = game
                .goods_factory()
                .query_goods_id_by_original_name(Some(name));
            let facts = game.find_player(player_id).and_then(|player| {
                let goods = player
                    .packet()
                    .base()
                    .traversing_goods()
                    .find(|goods| goods.base_properties_index() == goods_index)?;
                Some((
                    player.pk_count(),
                    player.money(),
                    player.depot_money(),
                    goods.identity(),
                    goods.price(),
                    goods.name().to_vec(),
                    player.server_region_id().unwrap_or_default(),
                    player.shape().get_tile_x().unwrap_or_default(),
                    player.shape().get_tile_y().unwrap_or_default(),
                    player.client_ip(),
                ))
            });
            let Some((
                pk_count,
                money,
                depot_money,
                goods,
                price,
                actual_name,
                region_id,
                x,
                y,
                ip,
            )) = facts
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            if log_type == 13 && game.log_system().equipment_compose_enabled() {
                let mut message = CMessage::new(0x0006_0202);
                message.add_byte(log_type as u8);
                message.add_long(player_id);
                message.base_mut().add_word(pk_count);
                message.add_ulong(money);
                message.add_ulong(depot_money);
                message.base_mut().add_guid(goods.ex_id);
                message.add_ulong(price);
                message.base_mut().add(&actual_name);
                message.add_byte(0);
                message.add_long(requested_amount);
                message.add_long(region_id);
                message.add_long(x);
                message.add_long(y);
                message.add_ulong(ip);
                let _ = message.send(game, false);
            }
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 1 })
        }
        SCRIPT_FUNCTION_ADD_GEM_EXCHANGE_LOG => {
            let (Some(destination_name), Some(source_name), Some(source_amount)) = (
                string_arguments[0],
                string_arguments[1],
                integer_arguments[2].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let destination_index = game
                .goods_factory()
                .query_goods_id_by_original_name(Some(destination_name));
            let source_index = game
                .goods_factory()
                .query_goods_id_by_original_name(Some(source_name));
            let Some(destination) = game.create_goods_core(destination_index) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let Some(source) = game.create_goods_core(source_index) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let Some((region_id, tile_x, tile_y)) = game.find_player(player_id).map(|player| {
                (
                    player.server_region_id().unwrap_or_default(),
                    player.shape().get_tile_x().unwrap_or_default(),
                    player.shape().get_tile_y().unwrap_or_default(),
                )
            }) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let mut message = CMessage::new(0x0006_0204);
            message.add_long(player_id);
            message.base_mut().add_guid(destination.identity().ex_id);
            message.base_mut().add(destination.name());
            message.add_byte(0);
            message.base_mut().add_guid(source.identity().ex_id);
            message.base_mut().add(source.name());
            message.add_byte(0);
            message.add_long(source_amount);
            message.add_long(region_id);
            message.add_long(tile_x);
            message.add_long(tile_y);
            let _ = message.send(game, false);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_ADD_JEWELRY_MADE_LOG => {
            let (Some(goods_name), Some(material_name), Some(jade_name)) = (
                string_arguments[0],
                string_arguments[1],
                string_arguments[2],
            ) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let goods_index = game
                .goods_factory()
                .query_goods_id_by_original_name(Some(goods_name));
            let material_index = game
                .goods_factory()
                .query_goods_id_by_original_name(Some(material_name));
            let jade_index = game
                .goods_factory()
                .query_goods_id_by_original_name(Some(jade_name));
            let Some(goods) = game.create_goods_core(goods_index) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let material = game.create_goods_core(material_index);
            let Some(jade) = game.create_goods_core(jade_index) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let jade_amount = integer_arguments[3]
                .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                .unwrap_or(1);
            let Some((region_id, tile_x, tile_y)) = game.find_player(player_id).map(|player| {
                (
                    player.server_region_id().unwrap_or_default(),
                    player.shape().get_tile_x().unwrap_or_default(),
                    player.shape().get_tile_y().unwrap_or_default(),
                )
            }) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let mut message = CMessage::new(0x0006_0205);
            message.add_long(player_id);
            message.base_mut().add_guid(goods.identity().ex_id);
            message.base_mut().add(goods.name());
            message.add_byte(0);
            if let Some(material) = material {
                message.base_mut().add_guid(material.identity().ex_id);
                message.base_mut().add(material.name());
                message.add_byte(0);
            } else {
                message.base_mut().add_guid(CGuid::GUID_INVALID);
                message.add_byte(0);
            }
            message.base_mut().add_guid(jade.identity().ex_id);
            message.base_mut().add(jade.name());
            message.add_byte(0);
            message.add_long(jade_amount);
            message.add_long(region_id);
            message.add_long(tile_x);
            message.add_long(tile_y);
            let _ = message.send(game, false);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_ADD_INCREMENT_LOG => {
            let Some(description) = string_arguments[0].filter(|value| value.len() <= 0xff) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let Some(amount) =
                integer_arguments[1].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let log_type = integer_arguments[2]
                .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                .unwrap_or(1);
            let (item_name, item_amount) = if log_type == 0 {
                let Some(item_name) = string_arguments[3].filter(|value| value.len() <= 32) else {
                    return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
                };
                (
                    item_name,
                    integer_arguments[4]
                        .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                        .unwrap_or(0),
                )
            } else {
                (&[][..], 0)
            };
            let Some((player_id, client_ip)) = game
                .find_player(player_id)
                .map(|player| (player.player_id(), player.client_ip()))
            else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let mut message = CMessage::new(0x0006_020d);
            message.add_byte(log_type as u8);
            message.add_byte(0);
            message.add_long(amount);
            message.base_mut().add(description);
            message.add_byte(0);
            message.add_long(player_id);
            if log_type == 0 {
                message.base_mut().add(item_name);
                message.add_byte(0);
                message.add_long(item_amount);
            }
            message.add_ulong(client_ip);
            let _ = message.send(game, false);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_ADD_LOG => {
            let Some(player) = game.find_player(player_id) else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let log_type = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let Some(content) = string_arguments[1] else {
                return Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 });
            };
            let mut message = CMessage::new(0x0006_020f);
            message.add_long(player.player_id());
            message.add_long(log_type);
            message.base_mut().add(content);
            message.add_byte(0);
            let _ = message.send(game, false);
            Some(ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 })
        }
        SCRIPT_FUNCTION_GET_PLAYER_RANK => Some(ScriptFunctionDispatchOutcome::Handled {
            legacy_return: game
                .find_player(player_id)
                .and_then(|player| {
                    game.player_ranks().map(|ranks| {
                        ranks.get_specify_player_rank(player.player_id() as u32) as i32
                    })
                })
                .unwrap_or(0),
        }),
        SCRIPT_FUNCTION_DELETE_EX_STATE_BY_TYPE => {
            let state_type = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR) as u16;
            let now_ms = runtime.now_milliseconds();
            Some(ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game
                    .delete_script_extended_state_by_type(player_id, state_type, now_ms, runtime)
                    as i32,
            })
        }
        SCRIPT_FUNCTION_SET_JING_LI_DAN => Some(ScriptFunctionDispatchOutcome::Handled {
            legacy_return: game.set_script_jing_li_dan_count(
                player_id,
                integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR),
            ),
        }),
        SCRIPT_FUNCTION_GET_JING_LI_DAN => Some(ScriptFunctionDispatchOutcome::Handled {
            legacy_return: game
                .find_player(player_id)
                .map_or(0, |player| i32::from(player.remain_jing_li_dan_count())),
        }),
        _ => None,
    }
}

/// Единый reached tail `CScript::RunFunction`: selector уже разрешён через
/// загруженный FunctionList, а аргументы вычислены тем же экземпляром CScript.
/// Порядок family-вызовов не наблюдаем сценарием, потому что каждый owner
/// обязан вернуть `DifferentFunction` до любых side effects для чужого ID.
pub(crate) fn dispatch_script_function<Runtime: ScriptFunctionRuntime>(
    game: &mut CGame,
    runtime: &mut Runtime,
    script_player_id: Option<i32>,
    script_npc_id: Option<i32>,
    script_region_id: Option<i32>,
    used_item_id: Option<CGuid>,
    died_monster_index: Option<u32>,
    drop_goods_position: Option<(i32, i32)>,
    script_id: i32,
    script_path: &[u8],
    function_id: i32,
    argument_count: usize,
    integer_arguments: [Option<i32>; SCRIPT_FUNCTION_ARGUMENT_CAPACITY],
    string_arguments: [Option<&[u8]>; SCRIPT_FUNCTION_ARGUMENT_CAPACITY],
) -> ScriptFunctionDispatchOutcome {
    match function_id {
        SCRIPT_FUNCTION_GET_DIED_MONSTER_INDEX => {
            return ScriptFunctionDispatchOutcome::Handled {
                legacy_return: died_monster_index.unwrap_or_default() as i32,
            };
        }
        SCRIPT_FUNCTION_GET_DIED_MONSTER_LEVEL => {
            let legacy_return = died_monster_index
                .filter(|index| *index != 0)
                .and_then(|index| game.find_monster_property_by_origin_index(index))
                .map_or(0, |properties| properties.level as i32);
            return ScriptFunctionDispatchOutcome::Handled { legacy_return };
        }
        SCRIPT_FUNCTION_GET_LEVEL_EXPERIENCE => {
            let Some(player) = script_player_id.and_then(|player_id| game.find_player(player_id))
            else {
                return ScriptFunctionDispatchOutcome::Invalid;
            };
            let level = integer_arguments[0]
                .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                .unwrap_or(i32::from(player.level())) as u8;
            let legacy_return = game
                .player_list()
                .level_experience(level)
                .wrapping_sub(player.experience()) as i32;
            return ScriptFunctionDispatchOutcome::Handled { legacy_return };
        }
        SCRIPT_FUNCTION_GET_MAXIMUM_LEVEL => {
            return ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.player_list().level_count() as i32,
            };
        }
        SCRIPT_FUNCTION_GET_AREA_TYPE => {
            return ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 };
        }
        SCRIPT_FUNCTION_GET_WORLD_SERVER_ID => {
            return ScriptFunctionDispatchOutcome::Handled {
                legacy_return: game.server_ids().1,
            };
        }
        _ => {}
    }
    if function_id == SCRIPT_FUNCTION_GET_MONSTER_REFRESH_TIME {
        let Some(region_id) = integer_arguments[0]
            .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            .filter(|_| integer_arguments[1] != Some(0))
        else {
            return ScriptFunctionDispatchOutcome::Handled { legacy_return: -1 };
        };
        let legacy_return = game.find_region(region_id).map_or(-1, |region| {
            region.base().monster_refresh_remaining_seconds(
                integer_arguments[1].unwrap_or(SCRIPT_INT_PARAMETER_ERROR),
                runtime.now_milliseconds(),
            )
        });
        return ScriptFunctionDispatchOutcome::Handled { legacy_return };
    }
    if function_id == SCRIPT_FUNCTION_REFRESH_BLOCK {
        if let Some(region_id) =
            integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
        {
            let _ = game.refresh_script_region_blocks(region_id);
        }
        return ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 };
    }
    match run_buff_skill_script_function(
        game,
        runtime,
        script_player_id,
        function_id,
        argument_count,
        integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
    ) {
        BuffSkillScriptFunctionOutcome::Handled { legacy_return } => {
            return ScriptFunctionDispatchOutcome::Handled { legacy_return };
        }
        BuffSkillScriptFunctionOutcome::Invalid => return ScriptFunctionDispatchOutcome::Invalid,
        BuffSkillScriptFunctionOutcome::DifferentFunction => {}
    }
    if matches!(
        function_id,
        SCRIPT_FUNCTION_GET_DAYS_HONOR_RANK
            | SCRIPT_FUNCTION_GET_WEEKS_HONOR_RANK
            | SCRIPT_FUNCTION_GET_MONTHS_HONOR_RANK
            | SCRIPT_FUNCTION_GET_TOTAL_HONOR_RANK
    ) {
        let legacy_return = script_player_id.map_or(0, |player_id| {
            game.script_honor_rank_position(
                player_id,
                function_id - SCRIPT_FUNCTION_GET_DAYS_HONOR_RANK,
            )
        });
        return ScriptFunctionDispatchOutcome::Handled { legacy_return };
    }
    if function_id == SCRIPT_FUNCTION_SEND_TOTAL_HONOR_RANKS {
        let legacy_return =
            script_player_id.map_or(0, |player_id| game.send_script_total_honor_ranks(player_id));
        return ScriptFunctionDispatchOutcome::Handled { legacy_return };
    }
    if function_id == SCRIPT_FUNCTION_GET_ATTEMPT_APPELLATION_ID {
        if argument_count != 0 {
            return ScriptFunctionDispatchOutcome::Invalid;
        }
        let Some(legacy_return) = script_player_id
            .and_then(|player_id| game.script_attempt_appellation_id(player_id))
            .map(|value| value as i32)
        else {
            return ScriptFunctionDispatchOutcome::Invalid;
        };
        return ScriptFunctionDispatchOutcome::Handled { legacy_return };
    }
    if matches!(
        function_id,
        SCRIPT_FUNCTION_ADD_APPELLATION_STATE
            | SCRIPT_FUNCTION_DEL_APPELLATION_STATE
            | SCRIPT_FUNCTION_GET_APPELLATION_STATE
    ) {
        if argument_count != 1 {
            return ScriptFunctionDispatchOutcome::Invalid;
        }
        let (Some(player_id), Some(state_id)) = (
            script_player_id.filter(|player_id| game.find_player(*player_id).is_some()),
            integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
        ) else {
            return ScriptFunctionDispatchOutcome::Invalid;
        };
        let now_ms = runtime.now_milliseconds();
        let legacy_return = match function_id {
            SCRIPT_FUNCTION_ADD_APPELLATION_STATE => {
                game.add_script_appellation_state(player_id, state_id as u32, now_ms, runtime)
            }
            SCRIPT_FUNCTION_DEL_APPELLATION_STATE => {
                game.delete_script_appellation_state(player_id, state_id as u32, now_ms, runtime)
            }
            SCRIPT_FUNCTION_GET_APPELLATION_STATE => game
                .get_script_appellation_state(player_id, state_id as u32)
                .unwrap_or(0),
            _ => unreachable!("appellation selector уже проверен"),
        };
        return ScriptFunctionDispatchOutcome::Handled {
            legacy_return: legacy_return as i32,
        };
    }
    if function_id == SCRIPT_FUNCTION_REQUEST_PLAYER_RANKS {
        if !script_player_npc_caller_exists(game, script_player_id, script_npc_id) {
            return ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 };
        }
        if let (Some(player_id), Some(maximum_rank_count)) = (
            script_player_id,
            integer_arguments[0].filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR),
        ) {
            let _ = game.request_script_player_ranks(
                player_id,
                maximum_rank_count,
                runtime.now_milliseconds(),
            );
        }
        return ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 };
    }
    if function_id == SCRIPT_FUNCTION_PLAY_EFFECT {
        let (Some(player_id), Some(region_id)) = (script_player_id, script_region_id) else {
            return ScriptFunctionDispatchOutcome::Invalid;
        };
        let effect_id = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
        let coordinates = match (integer_arguments[1], integer_arguments[2]) {
            (Some(tile_x), Some(tile_y))
                if tile_x != SCRIPT_INT_PARAMETER_ERROR && tile_y != SCRIPT_INT_PARAMETER_ERROR =>
            {
                Some((tile_x, tile_y))
            }
            _ => None,
        };
        return game
            .script_play_region_effect(player_id, region_id, effect_id, coordinates)
            .map_or(ScriptFunctionDispatchOutcome::Invalid, |_| {
                ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 }
            });
    }
    if function_id == SCRIPT_FUNCTION_WEATHER {
        let Some(player_id) = script_player_id else {
            return ScriptFunctionDispatchOutcome::Invalid;
        };
        let weather_index = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
        let _ = game.script_change_weather(player_id, weather_index);
        return ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 };
    }
    if function_id == SCRIPT_FUNCTION_PLAY_ACTION {
        if let Some(player_id) = script_player_id {
            let action = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let _ = game.script_play_action(player_id, action);
        }
        return ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 };
    }
    if function_id == SCRIPT_FUNCTION_PLAY_SOUND {
        let (Some(player_id), Some(region_id), Some(sound_file)) =
            (script_player_id, script_region_id, string_arguments[0])
        else {
            return ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 };
        };
        let send_around = integer_arguments[1]
            .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            .unwrap_or_default()
            != 0;
        let _ = game.script_play_sound(player_id, region_id, sound_file, send_around);
        return ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 };
    }
    if function_id == SCRIPT_FUNCTION_INVISIBLE {
        return ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 };
    }
    if matches!(
        function_id,
        SCRIPT_FUNCTION_GOD_MODE | SCRIPT_FUNCTION_RESIDENT_MODE
    ) {
        if let Some(player_id) = script_player_id {
            let _ =
                game.set_script_player_god_mode(player_id, function_id == SCRIPT_FUNCTION_GOD_MODE);
        }
        return ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 };
    }
    if function_id == SCRIPT_FUNCTION_GM_MODE {
        if let Some(player_id) = script_player_id {
            let _ = game.send_script_player_gm_mode(player_id);
        }
        return ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 };
    }
    if function_id == SCRIPT_FUNCTION_PLAYER_TALK {
        if let (Some(player_id), Some(text)) = (script_player_id, string_arguments[0]) {
            let _ = game.script_player_talk(player_id, text);
        }
        return ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 };
    }
    if function_id == SCRIPT_FUNCTION_OPEN_CHANGE_PLAYER_NAME {
        if let Some(player_id) = script_player_id {
            let _ = game.open_script_player_rename(player_id);
        }
        return ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 };
    }
    if function_id == SCRIPT_FUNCTION_IS_RIDER {
        return ScriptFunctionDispatchOutcome::Handled {
            legacy_return: script_player_id
                .and_then(|player_id| game.find_player(player_id))
                .is_some_and(CPlayer::is_rider) as i32,
        };
    }
    if let Some(outcome) = run_village_war_menu_script_function(
        game,
        script_player_id,
        script_npc_id,
        script_region_id,
        function_id,
        [integer_arguments[0], integer_arguments[1]],
    ) {
        return outcome;
    }
    if let Some(outcome) = run_core_player_script_function(
        game,
        runtime,
        script_player_id,
        script_npc_id,
        script_region_id,
        drop_goods_position,
        script_id,
        script_path,
        function_id,
        argument_count,
        integer_arguments,
        string_arguments,
    ) {
        return outcome;
    }
    match function_id {
        SCRIPT_FUNCTION_OPEN_PRECIOUS_BOX => {
            let legacy_return = script_player_id.map_or(0, |player_id| {
                game.open_precious_box(player_id, string_arguments[0].unwrap_or_default())
            });
            return ScriptFunctionDispatchOutcome::Handled { legacy_return };
        }
        SCRIPT_FUNCTION_GET_PRECIOUS_ITEM => {
            let legacy_return = script_player_id.map_or(-1, |player_id| {
                game.get_precious_box_item(
                    player_id,
                    integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR),
                    runtime,
                )
            });
            return ScriptFunctionDispatchOutcome::Handled { legacy_return };
        }
        SCRIPT_FUNCTION_CLOSE_PRECIOUS_BOX => {
            if let Some(player_id) = script_player_id {
                game.close_precious_box(player_id);
            }
            return ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 };
        }
        _ => {}
    }
    macro_rules! handled {
        ($outcome:expr, $pattern:path) => {
            match $outcome {
                $pattern { legacy_return, .. } => {
                    return ScriptFunctionDispatchOutcome::Handled { legacy_return }
                }
                _ => {}
            }
        };
    }

    match run_goods_item_script_function(
        game,
        runtime,
        script_player_id,
        used_item_id,
        function_id,
        argument_count,
        [integer_arguments[0], integer_arguments[1]],
    ) {
        GoodsItemScriptFunctionOutcome::Handled(legacy_return) => {
            return ScriptFunctionDispatchOutcome::Handled { legacy_return };
        }
        GoodsItemScriptFunctionOutcome::Invalid => return ScriptFunctionDispatchOutcome::Invalid,
        GoodsItemScriptFunctionOutcome::DifferentFunction => {}
    }

    handled!(
        run_gods_battle_scalar_script_function(
            game,
            runtime,
            script_player_id,
            function_id,
            integer_arguments[0],
        ),
        GodsBattleScalarScriptFunctionOutcome::Handled
    );
    handled!(
        run_war_contend_script_function(
            game,
            runtime,
            script_player_id,
            script_npc_id,
            script_region_id,
            function_id,
            [integer_arguments[0], integer_arguments[1]],
            [
                string_arguments[2],
                string_arguments[3],
                string_arguments[4],
                string_arguments[5],
            ],
        ),
        WarContendScriptFunctionOutcome::Handled
    );
    handled!(
        run_country_war_action_script_function(
            game,
            runtime,
            script_player_id,
            script_npc_id,
            script_region_id,
            function_id,
            [integer_arguments[0], integer_arguments[1]],
        ),
        CountryWarActionScriptFunctionOutcome::Handled
    );
    handled!(
        run_nation_war_script_function(
            game,
            runtime,
            script_player_id,
            function_id,
            [integer_arguments[0], integer_arguments[1]],
        ),
        NationWarScriptFunctionOutcome::Handled
    );
    handled!(
        run_country_war_query_script_function(
            game,
            script_player_id,
            script_npc_id,
            function_id,
            [
                integer_arguments[0],
                integer_arguments[1],
                integer_arguments[2]
            ],
        ),
        CountryWarQueryScriptFunctionOutcome::Handled
    );
    handled!(
        run_country_war_declaration_script_function(
            game,
            script_player_id,
            script_npc_id,
            function_id,
            integer_arguments[0],
        ),
        CountryWarDeclarationScriptFunctionOutcome::Handled
    );

    if let Some(legacy_return) = run_fairy_script_function(
        game,
        runtime,
        script_player_id,
        function_id,
        integer_arguments[0],
    ) {
        return ScriptFunctionDispatchOutcome::Handled { legacy_return };
    }

    if let Some(legacy_return) = run_battle_fairy_script_function(
        game,
        runtime,
        script_player_id,
        function_id,
        [
            integer_arguments[0],
            integer_arguments[1],
            integer_arguments[2],
            integer_arguments[3],
        ],
        [string_arguments[0], string_arguments[1]],
    ) {
        return ScriptFunctionDispatchOutcome::Handled { legacy_return };
    }
    handled!(
        run_country_scalar_query_script_function(
            game,
            script_player_id,
            function_id,
            integer_arguments[0],
        ),
        CountryScalarQueryScriptFunctionOutcome::Handled
    );
    handled!(
        run_country_identity_script_function(
            game,
            script_player_id,
            function_id,
            integer_arguments[0],
            integer_arguments[1],
        ),
        CountryIdentityScriptFunctionOutcome::Handled
    );
    handled!(
        run_country_control_point_script_function(
            game,
            function_id,
            integer_arguments[0],
            integer_arguments[1],
        ),
        CountryControlPointScriptFunctionOutcome::Handled
    );
    handled!(
        run_country_exile_time_script_function(
            game,
            script_player_id,
            function_id,
            integer_arguments[0],
            runtime,
        ),
        CountryExileTimeScriptFunctionOutcome::Handled
    );
    handled!(
        run_country_quest_switch_script_function(
            game,
            script_player_id,
            function_id,
            integer_arguments[0],
            integer_arguments[1],
            integer_arguments[2],
        ),
        CountryQuestSwitchScriptFunctionOutcome::Handled
    );
    handled!(
        run_country_scalar_script_function(
            game,
            function_id,
            integer_arguments[0],
            integer_arguments[1],
        ),
        CountryScalarScriptFunctionOutcome::Handled
    );

    if let EquipmentSessionScriptFunctionOutcome::Opened(_) = run_equipment_session_script_function(
        game,
        script_player_id.unwrap_or_default(),
        function_id,
    ) {
        return ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 };
    }
    match run_equipment_da_kong_script_function(
        game,
        script_player_id.unwrap_or_default(),
        function_id,
        string_arguments[0],
        runtime,
    ) {
        EquipmentDaKongScriptFunctionOutcome::DifferentFunction => {
            ScriptFunctionDispatchOutcome::DifferentFunction
        }
        EquipmentDaKongScriptFunctionOutcome::HandledWithoutCall
        | EquipmentDaKongScriptFunctionOutcome::Refreshed(_) => {
            ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 }
        }
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2508::CreateFaction::DoAsyncCall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6131
// RVA: 0x000AEB90
// ADDRESS: 004aeb90
// PROTOTYPE: void __thiscall DoAsyncCall(__int64 param_1, long param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2527::ApplyJoinFaction::DoAsyncCall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6248
// RVA: 0x000AEC30
// ADDRESS: 004aec30
// PROTOTYPE: void __thiscall DoAsyncCall(__int64 param_1, long param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2712::DeclareFactionWar::DoAsyncCall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6716
// RVA: 0x000AECE0
// ADDRESS: 004aece0
// PROTOTYPE: void __thiscall DoAsyncCall(__int64 param_1, long param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2527::ApplyJoinFaction::ApplyJoinFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6242
// RVA: 0x000AEDB0
// ADDRESS: 004aedb0
// PROTOTYPE: undefined __thiscall ApplyJoinFaction(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2712::DeclareFactionWar::DeclareFactionWar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6710
// RVA: 0x000AEDE0
// ADDRESS: 004aede0
// PROTOTYPE: undefined __thiscall DeclareFactionWar(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2508::CreateFaction::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6128
// RVA: 0x000AEE90
// ADDRESS: 004aee90
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2508::CreateFaction::CreateFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6125
// RVA: 0x000AEEB0
// ADDRESS: 004aeeb0
// PROTOTYPE: undefined __thiscall CreateFaction(long param_1, char * param_2, long param_3, uchar param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2508::CreateFaction::OnAsyncCallback
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6144
// RVA: 0x000AF250
// ADDRESS: 004af250
// PROTOTYPE: void __thiscall OnAsyncCallback(tagAsyncResult * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004af3ee
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6181
// RVA: 0x000AF3EE
// ADDRESS: 004af3ee
// PROTOTYPE: undefined Catch@004af3ee()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004af43b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6185
// RVA: 0x000AF43B
// ADDRESS: 004af43b
// PROTOTYPE: undefined FUN_004af43b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2527::ApplyJoinFaction::OnAsyncCallback
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6263
// RVA: 0x000AF460
// ADDRESS: 004af460
// PROTOTYPE: void __thiscall OnAsyncCallback(tagAsyncResult * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2712::DeclareFactionWar::OnAsyncCallback
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6731
// RVA: 0x000AF660
// ADDRESS: 004af660
// PROTOTYPE: void __thiscall OnAsyncCallback(tagAsyncResult * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::CheckFunctionRunning
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:11947
// RVA: 0x000AF990
// ADDRESS: 004af990
// PROTOTYPE: SCRIPTRETURN __thiscall CheckFunctionRunning(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::RunFunction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:90
// RVA: 0x000AFAF0
// ADDRESS: 004afaf0
// PROTOTYPE: long __thiscall RunFunction(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004c3f4b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:137
// RVA: 0x000C3F4B
// ADDRESS: 004c3f4b
// PROTOTYPE: undefined Catch@004c3f4b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
