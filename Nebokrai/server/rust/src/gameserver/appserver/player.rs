//! Достигнутая send-family проекция `CPlayer` исторического GameServer.
//! UpdateProperty (0x004593E0, GameServer.exe/GameServer.pdb, player.cpp)
//! оставляет формулы экипировки у player, а +0x24 каждого состояния выполняет
//! общий живой CMoveShape-проход. Промежуточные tagProperty видны следующему
//! callback; копии списка, фильтр «последний CHBY» и очередь visuals сняты.
//! Чистые Ex/Undead/Ride-формулы читают owning payload по ссылке без callbacks.
//! CHBY Begin/End не дублируются в Player mutation-wrapper: их mode/hotkeys
//! и общий AddSkill/DelSkill выполняет единственный state-owner через CGame.
//! Mount0x00444E20 представлен CGame::begin_player_ride: C-string name перед
//! constructor/Begin/append, fight-state GS0154 с native return1 без установки.
//! Технический отказ allocator не эмулируется; время и actual participants
//! принадлежат этому Begin, а внешний Update остаётся в ветке UseItem.
//! Undead/Appellation также устанавливается и завершается через общий
//! lifecycle CMoveShape; Player не хранит промежуточную пачку копий состояний.
//! RestoreHp/RestoreMp (0x00444C80/0x00444D50) координирует CGame:
//! после двух cooldown-часов выполняются constructor, Begin(self, self)
//! с отдельным clock и общая регистрация. Нулевой срок не означает мгновенное
//! лечение; повторные состояния сохраняются, внешнего UpdateProperty нет.
//! HP/MP-mutation wrappers устранены: AI обращается к тому же живому payload.
//! Particular: OnObjectAdded (0x004451A0) вызывает общий Begin синхронно
//! после container commit и GoodsAI, до следующего товара; собственный equipment-listener
//! (0x004EF6C0) сначала публикует skills/properties/BF720/PackExpand.
//! OnEnterRegion (0x0045A410) собирает только ordered unique additional-значения;
//! packet-вектор отбрасывается, GUID-listener дополняется экипировкой.
//! Packet Add (0x004DE6E0) регистрирует GoodsAI до player-listener;
//! ComputeTicket (0x0043E820) читает wall-clock только после life/ticket/type/start gates.
//! Packet Swap проходит тот же синхронный Add, включая восстановление displaced
//! при отказе; storage-алгоритм и его конечный garbage-collect остаются общими.
//! Ride AI (0x004F9110, other states/ridestate.cpp) проверяет packet actual
//! Sufferer через существующий property-listener: сначала GAP_MOUNT_TYPE != 0,
//! затем GUID lookup и сравнение type/level. Проверка read-only, без cached GUID,
//! локального state-key и часов; поиск по goods-name оставлен только формуле.
//! OnLost (0x0044183B..0x00441894, player.cpp:1780) проходит живые позиции:
//! для очередного CHBY ставит has_changed_region=false/online=true и при
//! !restore_online сразу отправляет исходный BF806 и вызывает End. Следующий
//! индекс/размер читается после callback, без предварительной пачки ключей,
//! принудительного destructor, дополнительных часов или UpdateProperty.
//! SelfTarget — объектная перегрузка Attack с самим игроком: обычный запрос
//! вызывает virtual +0x78 в 0x00488E20, item — в 0x00489109/0x00489547,
//! WarSoul — в 0x0048953D/0x00489547. Он имеет ту же цель type=400/id игрока,
//! что явный Object на себя; Point использует другую перегрузку +0x74.
//! Сравнение ожидающего запроса не равно равенству исполнения:
//! Attack point (0x0050A349..0x0050A367) сравнивает ID/x/y, object
//! (0x0050A13A..0x0050A15C) — ID/type/target ID. Снимок уровня и GUID
//! не заменяют ожидающую команду; полный Eq dispatch остаётся для lifecycle.
//! Обычная object-очередь повторяет ID/type/target ID guard в
//! 0x0050A1B5..0x0050A1D7. Обе типизированные формы используют одно правило
//! проекции запроса; контейнеры и полное равенство исполнений независимы.
//! OnChangeSkill (0x00508E6A..0x00508E7E) выбирает GetDefaultAttackSkillID
//! через обычный SetCurrentSkill. Выбранный ID сохраняется после End;
//! живое исполнение отдельно принадлежит CPlayerAI, дополнительного idle-ID нет.
//! Exact-key AutoProtect End (0x005D44E0) снимает auto_protected перед
//! RemoveState; визуальный эффект и Player/GM gate принадлежат общему CGame
//! direct-End координатору, а не повторяются в тонком state-wrapper.
//!
//! PDB `GameServer/GameServer.pdb` подтверждает base `CMoveShape +0x0` и signed
//! `m_lTeamID +0xB20`, а также unsigned byte `m_btCountry +0xA5C`. Exact
//! `CMessage::SendToAround` RVA `0x00014420` и
//! `SendToRegionContryPlayer` RVA `0x00014760` читают inherited
//! `CBaseObject::m_lID +0x8` как numeric map/player identity, team ID и country
//! после RTTI `CShape/CMoveShape -> CPlayer`. Эти достигнутые поля имеют статус
//! `IMPLEMENTED, VERIFIED_DISASSEMBLY`; исходники
//! `server/gameserver/appserver/player.h/.cpp`.
//! Расход MP атрибутных навыков Po/Yu использует непосредственно equipment[10]
//! (CPojia::AI 0x0052a77f), без повторного GetWarSoulGoods. Это отдельный
//! адаптер к общему списанию; проверка типа товара у CWangsheng сохраняется.
//! GameSave сохраняет предмет в руке перед экипировкой: CPlayer::AddToByteArray
//! (0x00440dc0) вызывает m_cHand::Serialize, а World читает этот контейнер
//! в том же порядке. Используется готовый codec CAmountLimitGoodsContainer,
//! сохраняющий количество и порядок предметов, известных фабрике.
//! Общий `AddSkillsToByteArray` (0x00432b80) пишет навыки в порядке
//! Attack→Defense→Summon→State, внутри категории — в порядке регистрации.
//! GameSave и initial client используют один обход; `GetNumSkills`
//! (0x00432aa0) тем же фильтром исключает только Defense с ID 10.
//! Имя skill-снимка читается из текущих свойств ID/уровня, как GetSkillName
//! (0x004D86E0). Some хранит прочитанные байты, включая пустое имя; None
//! требует при публикации локализованный GS0318 либо пустую строку.
//! LoadBFDefualtProperty (0x00502BC0) после AddSkill отдельно проверяет
//! GetSkill: успех регистрации не гарантирует разрешение metadata-категории.
//! Два player-входа продолжают инициализацию без skill-пакета при null,
//! вместо паники или отката уже добавленного экземпляра.
//! Organizing identity `m_lFactionID/m_lFacMasterID` обновляется из полного
//! World `0x7FE06` wire; `IsFactionMaster` сохраняет exact positive-faction и
//! player-ID equality contract.
//! Полный World→Game `0x7F901` handoff теперь декодирует единый
//! `CPlayer::DecordFromByteArray(..., true)` layout: shape/base/combat,
//! skills/states, containers, variables, timers, companions, quests, country,
//! organization и session. Обратный `AddGameSaveToByteArray` использует те же
//! owned поля и live pet/carriage snapshot; container codec failure прекращает
//! wire строго на первом false, как исходная цепочка. После чтения base-wire
//! `bBFSummon` намеренно снова выводится из локального `m_dwWarSoulState`, а не
//! принимается как независимый persisted fact.
//! Quest-map, skill-list, friend-list и три organization-list decoder-а
//! сохраняют signed legacy count: отрицательное значение очищает коллекцию и
//! не отклоняет остальной handoff. CiQing/pet loops исходника на отрицательном
//! count патологически обходили бы весь `u32`; безопасная Rust-граница такие
//! данные отклоняет. Organization force/contribute читаются как `i32`, но
//! нормализуются в `0/1`, поскольку persisted owner хранит их как `bool`.
//! Decoder virtual +0x9C — `UpdateProperty`, не InitSkills. Последний
//! вызывается `OnLogMessage` после login-script (0x0049FC0A, slot +0x14C):
//! добавляет отсутствующие intrinsic skills и выбирает default посредством
//! SetCurrentSkill, не восстанавливая HP. CGame сохраняет этот owning tail.
//! CPlayer ctor (0x004590DC/0x00459365) задаёт InChangingRegion=true и
//! last-enter timestamp=0: первый 8F801 проходит обычный EnterTime/flag gate.
//! Initial-login tail обходит GoodsAI candidates в exact positional order:
//! equipment, packet, hand, auction и depot. Для equipment-state `2` нулевая
//! packed date прерывает только текущий container; просроченное состояние
//! становится `3` до old-client `0xBF928`, как в `OnLogMessage`.
//! `CMessage::Run` RVA `0x000149D0` дополнительно читает inherited father
//! `+0x40` как текущий `CServerRegion*`; удалённый raw pointer выражен
//! `Option<i32>` region identity в assembly-проекции.
//!
//! Материализована также подтверждённая setter-family: боевые scalar-ы
//! насыщаются до `INT_MAX`, contribution — до `±2_000_000_000`, а fetch power
//! сравнивается с unsigned-представлением setup limit. Это минимальный owned
//! player state для будущих equipment/battle-fairy side effects, но не замена
//! полного constructor-а, property recalc или runtime player lifecycle.
//! World kill confirmation `0x7F806` materializes `wPkCount`, `dwKillCount` и
//! murderer timestamp: PK насыщается до `0xFFFF`, kills wrapping-инкрементятся,
//! а clock читается только при первом ненулевом murderer state.
//! FourNation reward `0x7FE46` добавляет owned `dwExploit`: advertised client
//! value сохраняет wrapping addition, `SetExploit` отдельно применяет exact
//! unsigned CountryParam maximum, а virtual `UpdateProperty` остаётся
//! обязательным caller-runtime effect после мутации.
//! Reached script property catalog отделён от gameplay setter-ов: generic
//! `SetValue/ChangeValue` сохраняет narrowing/wrapping storage, включая
//! shipped `Experience` alias и достигнутые honor
//! `dwAppellationID/dwRankOfNobilityID`; bool fairy enable pair нормализует
//! ненулевой write в persisted player state. `GetValue` читает также credit,
//! SZL и contribution из canonical storage. Пересчёт и
//! `0xBF721` остаются у вызывающего `CGame`.
//! Total honor-rank startup материализует days/weeks/months counters и
//! nobility rank: reset меняет owned state и возвращает точный признак
//! `AdjustHonorRank`, который `CGame` связывает с общим script scheduler.
//! Silence-timeout, как и оригинал, проверяется лениво при query по
//! инъецируемому wrapping `timeGetTime`-значению; reached `OnExit` отдельно
//! сохраняет исходные один либо три clock-read и пересчитывает остаток перед
//! GameSave. GM `0x7FC0B/0x7FC0E`
//! замыкают name lookup, mutation, двухпроходный ordered query и World
//! responses, поэтому отдельный scheduler не требуется.
//! Тот же `OnExit` после around-публикации восстанавливает умершего и
//! сохраняет выбранные `GetReturnPoint` region/tile/direction до GameSave;
//! team membership при `OnLost` намеренно не очищается.
//! Client allocation `0x8FA01` владеет sex/occupation, remaining point,
//! четырьмя base stat и base HP/MP maxima. Сохранены общий STR gate для всех
//! `Add*`, безусловный расход очка и отдельный 0x9c-byte `m_Property` wire:
//! reached recompute заменяет только подтверждённые поля, не обнуляя хвост.
//! PDB-layout `tagBaseProperty/tagProperty` задаёт отдельные `wBaseMaxYp +0xB8`,
//! `wBaseBurden +0xD6`, hit/attack-speed slots и fairy-флаги
//! `+0x90..+0x92`; полный `UpdateProperty` пересобирает их из base owner-а,
//! а не сохраняет ошибочно сдвинутые байты прежнего combat snapshot.
//! PvP preferences `0x8FA05` хранят пять live permission flags, которые
//! downstream player/skill AI читает при выборе обычных, team, union,
//! criminal и country целей; unknown selector только потребляет вход.
//! `CPlayer::IsAttackAble` материализован двумя достигнутыми направлениями:
//! player-attacker проходит общую PvP/security политику, monster-attacker —
//! точные tame, city/country guard и criminal ветви. Их нельзя объединять с
//! одноимённой monster-side проверкой обратного направления.
//! Межсерверная прогрессия `0x7FA08/09/0B` использует собственную карту
//! навыков и поля уровня с опытом: перегрузки по имени делегируют фабрике
//! поиск ID, а `SetLevel` возвращает фракционное последствие вызывающему коду
//! до точного клиентского сообщения прогрессии. `0x7FA0A` относится к
//! контейнерному удалению предметов и не проходит через карту навыков.
//! Тот же persisted level/exp/vigour/base-stat owner теперь обслуживает reached
//! auto-inc `CheckLevel`; multi-level scripts, property recompute и network
//! результаты остаются у `CGame`, чтобы helper-ы не образовывали shadow path.
//! Метки времени общих и государственных разговоров принадлежат тому же
//! состоянию игрока: время восстановления с переполнением обновляется до
//! проверки и списания стоимости канала.
//! Текущие HP/MP имеют собственные setter-и с clamp к текущим max-свойствам;
//! RP/YP сохраняют соседние WORD offsets `0xAC/0xAE` base-wire. Изменение
//! самих max не выполняет этот clamp без конкретного caller-а.
//! Достигнутый damage runtime использует те же maximum HP и `reank` в
//! унаследованном `CMoveShape::Stiffen`, не создавая отдельный combat snapshot.
//! `CPlayer::OnBeenHurted` RVA `0x00041D80` связан целиком: после износа брони
//! пассивные свободные питомцы принимают identity нападавшего, а атака игрока
//! другой страны вне block `1` через region-owned wrapping cooldown публикует
//! World `0x5FD09/GS0133` с именем региона и координатами жертвы.
//! Соседний `OnDied` использует тот же `m_lNotify`, но отдельный kill timestamp
//! и строгий gate `last + interval < now`; `GS0140..GS0142` также запрещены
//! именно block `1`, а не одноимённым значением security.
//! `OnBeenMurdered` honor-eliminate `0x5FD0D` сравнивает union master identity,
//! проверяет тот же block `1` и передаёт World все четыре текущих счётчика;
//! общий PK policy ниже по цепочке по-прежнему использует security клетки.
//! `UseItem` материализует точные коды требований, принадлежащее игроку
//! изучение навыков, расход предметов в рюкзаке и четыре заменяемых боевых
//! `tagExpendableEffect`. Эффекты 0x4A..4D читают часы внутри своего case,
//! не в общем UseItem.
//! Новая запись: value1 для owner → clock → value2 → отдельное value1 для
//! прибавления свойства → append. Замена: снять old → value1/прибавить →
//! clock/timestamp → value2/duration → повторное value1/owner.value.
//! Эти чтения не объединяются; WORD/DWORD wrapping остаётся исходным.
//! Проверки и состояния ездового животного и
//! `ChangeBody` замкнуты на владельцах игрока и игры. Возврат предметами
//! сохраняет порядок рюкзак → экипировка → рука и передаёт `CGame` только
//! последовательное удаление предметов и смену региона; временные `CState`
//! и неподдержанные селекторы виртуальной машины сценариев остаются внешней
//! границей времени исполнения.
//! Как в связном `RefreshContainerOwners`, достигнутые equipment,
//! ordinary-fairy и battle-fairy containers принадлежат player type `400` с
//! его numeric ID. Ordinary fairy получает exact volume 14 и persisted
//! enable/vigour/experience; persisted battle-fairy enable декодируется из
//! World base-property offset `0x128`, а `CanMountEquip` использует оба enable
//! flag-а, headgear addon и live requirements без внешнего result snapshot.
//! `MountEquip` cases `0x75..0x78` применяют четыре ordinary-fairy addon-а к
//! player combat state через setup scales `+0x8AC..+0x8B8`, затем повторно
//! используют occupation-derived STR/DEX/INT формулы; signed pass и clamp
//! совпадают с остальными equipment addon-ами.
//! Общие прямые поля MountEquip0x00442610 и MountEquipRide0x0043C5E0
//! сохраняют DWORD wrapping до negative gate/INT_MAX и WORD wrapping
//! до отрицательного clamp; скорость атаки остаётся WORD без этого clamp.
//! Отдельные FuMo и производные формулы не подменяются этим прямым адаптером.
//! Battle-fairy cases `0x9B/0x9C/0x9E..0xA1` исполняются после slot-10
//! prelude: base fallback мутирует canonical goods, живая BF HP разрешает
//! масштабированный `0.0001` вклад в player properties, а BF HP/MP зажимаются
//! к обновлённым максимумам до общего state pass-а.
//! Durability gate в `MountAllEquip` ограничивает только Flash/TaoZhuang scan:
//! последующие `MountEquip/MountCiQingEquip` применяют addon-ы всех занятых
//! слотов, включая предметы с нулевой прочностью.
//! `MountCiQingEquip` cases `0x80..0x84` отдельно мутируют persisted
//! `m_BaseProperty +0x12C..+0x13C`: два signed pass-а сохраняют wrapping
//! сложение и нулевой clamp отрицательного результата до общего пересчёта.
//! `GAP_EQUIP_ACTIVE (0x69)` использует пересобранную из живой экипировки
//! anima-bind карту уровней и в storage order применяет процентный
//! `ActiveEquip`; ездовой owner этого case не имеет. `ActiveEquip`, обычные
//! addon-ы и `MountFuMoProperty` сохраняют x87 truncate полной производной
//! суммы, включая вложенное преобразование ordinary-fairy характеристик.
//! Periodic hatcher caller замкнут через `CGame`;
//! Hotkey owner хранит exact 24 DWORD и связывает назначение с возвратом
//! consumable из hand в packet/hand/wallet/YuanBao; equipment destination
//! проходит исходный remove→failed add→hand rollback без потери ownership.
//! Enhancement/precious-box confirm хранит server-trusted container-script
//! path у игрока; отмена очищает только shadow selection без переноса goods.
//! Remote equipment inspection использует owned persisted head/face/mode и
//! тот же live equipment container, не отдельный display snapshot.
//! Depot-password vertical дополнительно материализует `m_eProgress`, оба
//! changing-guard-а, password byte-string и owned `CBank/CDepot`; открытый
//! bank участвует в реальном wallet↔bank `0x90301` ownership pass. Numeric
//! значения внутреннего `eProgress` не выходят в wire и потому заменены typed
//! enum без выдуманного `repr`.
//! Exact `GetWarSoulGoods` читает headgear cell 10 и признаёт её боевой феей
//! только при addon `GAP_BF_BATTLE_FAIRY` value-id 1, равном единице.
//! `ReplacePlayerData/RestorePlayerData` выражены временной typed-проекцией
//! только для defense-pass навыков боевого духа: blast/level берутся из
//! headgear, три setup scale действуют во время защиты, а затем прежние scale
//! возвращаются с исходным усечением к нулю и minimum clamp.
//! `BatllteFairyCombine` соединяет container inputs, global BattleFairy gate,
//! fetch power, shared Game RNG/factory, `CMoveShape::AddSkill` и ordered
//! адресные object/skill/goods/audit effects. `BTreeMap` skill storage в
//! `CMoveShape` заменяет четыре pointer-vector-а только для общего confirmed
//! identity/level/type/name state; выполнение concrete skill owners не
//! перенесено сюда. Результат combine кладётся в обычную ячейку `Battle`, а
//! не в gear-ячейку, поэтому исходный owner доказательно не вызывает здесь
//! `BFPropertyAdd`, equipment mutation или `PropertiesChanged`. Account для
//! audit принадлежит player snapshot и пока заполняется отдельным caller-ом
//! при восстановлении player identity.
//! Script revive боевой феи восстанавливает HP/MP из maxima и атомарно меняет
//! recall/died/summon/WarSoul state; goods и properties wire публикует CGame.
//! Поэтому `from_send_state` остаётся явной assembly-границей уже
//! восстановленного runtime. Figure передаётся как доказанный derived virtual
//! fact; владение spatial state остаётся у `CMoveShape`.
//! `SummonBF` RVA `0x00101CB0` материализован единым Player→CGame→region
//! проходом: guards, summon/recall state, ordered around effects и area-map
//! action. Active pets теперь хранят exact movement-shape refs и восстанавливаются
//! из GameSave в region-owned monsters; codec и goods-message decoder
//! остаются явной границей и report не подменяет исторические packet bytes.
//! `BFPropertyAdd` соединяет восемь gear-ячеек с headgear battle fairy,
//! `GlobeSetup` occupation coefficients и player combat state. Сохранены
//! ранний effect до результата Add, post-remove `-1`, clamp текущих HP/MP,
//! двойное применение MaxHP/Str/Int/Dex, x87 truncate каждой дробной дельты
//! и двойной `0xBF918` в Remove.
//! Goods-message `0x8FC2A` материализован до ordered potential mutation:
//! aggregate guard остаётся в клиентских единицах, отдельные allocation
//! умножаются на `10000`, одинаковые property keys имеют `std::map` first-win,
//! а каждый вызов и outer caller публикуют собственный `0xBF918`.
//! Gear add/remove теперь через `CGame` действительно исполняет ordered
//! `0xBF721/0xBF918`; remove сохраняет две одинаково обязательные публикации
//! old-client payload после успешного `BFPropertyAdd(-1)`.
//! Улучшение `0x8FC28` замыкает проверки, принадлежащие кошельку предметы,
//! общий генератор случайных чисел, изменение уровня и роста на фабрике,
//! результат ошибки цели, позиционный расход камней и упорядоченные клиентские
//! и контрольные последствия. Снимки цели, камней и игрока сохраняют контрольную
//! запись мира после необратимого удаления, а `CGame` публикует точные
//! `0xC0101/0xC0102`, `0xBF918` и `0x60202/0x60203` в исходном порядке.
//! `ResetPotential` использует принадлежащий игроку рюкзак `CVolumeLimitGoodsContainer` 8×12:
//! первый `ZHQLS01` расходуется до изменения дополнительных свойств и игрока,
//! семь учтённых вкладов возвращаются в общий потенциал, после чего публикуется
//! один итоговый `0xBF918`.
//! `ResetSkill` связывает головной предмет экипировки, необязательный предмет
//! сброса из рюкзака, общий генератор случайных чисел игры, точные несовместимые
//! пары, полное снятие и установку девяти навыков боевой феи и подтверждения
//! `0xBF71D/0xBF918`.
//! Сообщение предмета `0x8FC29` использует отдельный сценарный сброс: владелец
//! игрока сохраняет исходное снятие и установку девяти навыков дополнительных
//! свойств вокруг живого сценария, не подменяя его внутренней случайной ветвью
//! `ResetSkill`.
//! `GetGoodsById` сохраняет точный поиск по руке, рюкзаку, экипировке и аукциону;
//! рука и аукцион являются владеющими контейнерами и участвуют в обновлении.
//! Однослотовый `m_cEnhancementContainer` хранит теневые данные выбранного
//! исходного предмета и даёт сценарию 9351 тот же живой предмет без копии.
//! Входящий `0x90301` проверяет контейнер, позицию, `GUID`, количество и
//! возможность складывания, затем записывает тень без смены владельца исходного
//! предмета и сохраняет исходный источник последней операции.
//! Сценарий `2249` использует того же живого владельца; созревшая замена
//! добавляется прямо в рюкзак, даже пока выполняется другой сценарий.
//! Двусторонний путь аукционного объявления использует те же рюкзак и
//! экипировку: обратный ход считает точную нагрузку экипировки, рюкзака и руки,
//! сохраняет последнюю операцию только после успешного добавления в назначение
//! и не включает временные предметы аукциона, феи и сессий в сумму веса.
//! Набор открытых базовых индексов `CiQing` хранится в упорядоченном `BTreeSet`;
//! запрос не создаёт постоянные предметы, а передаёт снимок владельцу фабрики
//! `CGame`; создание считает и удаляет стопки рюкзака в порядке контейнера и
//! сохраняет владение новым предметом или стопкой. `CGame` публикует точные
//! `0xC0101/02` и контрольную запись мира `0x60218`. Владеющие контейнеры
//! `CiQing` имеют точные объёмы `8/3`; ячейки сборки удаляются по позиции.
//! Основное удаление `CiQing` сохраняет семантику частичного количества
//! `DeleteGoods`. Установка в руку читает точные дополнительные свойства
//! `243/244`; расход из руки также сохраняет частичное количество и не выдаёт
//! полученную ссылку на `CGoods` за полную копию.
//! Владелец свойств `CiQing` хранит упорядоченные обычные карты, карты
//! `TaoZhuang` и идентификатор набора. `UpdateCiQingProperty` сопоставляет
//! равные по размеру упорядоченные снимки и насыщает отрицательную разницу
//! нулём; объединение для клиента сохраняет беззнаковое сложение с
//! переполнением. Снимок другого игрока читает это состояние и те же
//! восемь принадлежащих `CiQing` ячеек без копий. `MountAllEquip` вычисляет
//! два снимка одной формулой оборудования — до и после CiQing — затем
//! сохраняет насыщенную разницу; единый результат проводит обязательный
//! `SendResultToClient` через обычный, equipment и специальный CiQing caller.
//! TaoZhuang теперь сохраняет constructor flags, unique original-name set,
//! ordered set counts/threshold-prefix, max-level skills и раздельные обычные/
//! CiQing property maps. `CGame` исполняет полный `DoneTaoZhuang`, поэтому эти
//! player-методы не являются отдельным недостижимым adapter-слоем. Каждый
//! полный `UpdateProperty` снова выставляет equipment/TaoZhuang pending-флаг;
//! выбор немедленного либо AI-tail завершения остаётся у setup-gate caller-а.
//! `skillmessage 0x90001` сохраняет learned-skill authorization, contend
//! notice, безусловное обнуление emotion state, self/point/object target и
//! socket reject; `0x90005` добавляет feature/HP guards и странный fallback
//! `546/547`. Concrete `CPlayerAI`, region symbol rule и полный region object
//! registry передаются как explicit facts.
//! Item-skill `0x90004` использует тот же player route с client-provided level
//! и добавляет ID в owned ordered `CMoveShape` vector только перед AI dispatch.
//! Shape commands сохраняют owned direction и emotion index/timestamp:
//! ClearEmotion всегда обнуляет оба поля, PerformEmotion делает это до guards
//! и запоминает repeated ID/time только при разрешённом живом AI owner-е.
//! Client relocation использует общие movement facts и `CShape` owner через
//! `CServerRegion`; caller сохраняет исходный `BF603 -> SetTileXY -> GS0163`
//! порядок и contend/symbol predicate, поэтому замещённый RAW удалён.
//! Quest movement также использует concrete `OnCannotMove` wire с текущими
//! tile coordinates; player-AI caller очищает emotion перед постановкой шага.
//! Friend owner хранит исходный ordered список до 40 byte-exact имён и online
//! flag; message caller замыкает reciprocal mutation, World persistence и
//! addressed client result, поэтому `AddFriend/DelFriend` RAW удалён.
//! Public identity owner хранит headpiece/appellation/honor state; country job
//! вычисляется concrete `CCountry`, а change request записывает attempt ID до
//! вызова server-trusted script owner-а.
//! Client timing owner хранит quest countdown и heartbeat acknowledgement:
//! остаток сохраняет signed 32-bit arithmetic исходного `time_t`, а wall/local
//! clock остаются внешними runtime-фактами message caller-а.
//! Player quest lifecycle хранит persisted `ushort → complete byte`: accept,
//! complete и disband публикуют `0xBFF2C/2D/2E`, а `0xBFF2F` position остаётся
//! transient client hint и не создаёт второго авторитетного quest state.
//! LeiTing owner хранит пять scalar-полей и ordered `tagThing` list; codec
//! совпадает с WorldServer `Add/DecodeByteArrayLeiTing`, а reward-флаг
//! выставляется только после exact energy/count threshold. Script `2650/2651`
//! работает с тем же списком: успешное увеличение добавляет `point * delta`
//! к энергии, применяет суточный порог `60` и публикуется единым snapshot.
//! Полный `AddToByteArray_ForClient(true)` теперь отделён от GameSave: он
//! сохраняет category-order навыков, old-client goods projection, четыре
//! currency GUID, organization/quest tails и CiQing completion side effect;
//! `CGame::OnLogMessage` вкладывает результат непосредственно в `0xBF401`.
//! Goods-session `0x8FC25` использует полный typed `eProgress` owner и
//! сбрасывает его в `None`, одновременно снимая один nesting moveable-запрет;
//! полиморфные session End/plug Exit принадлежат caller runtime-у.
//! Nation-war player lifecycle связывает exact `SetContendState`,
//! `OnDied`/`OnRelive` и millisecond-tail `PeriodicalUpdate`: owned state
//! хранит три PDB-поля `+0xBA5/+0xBA8/+0xBAC`, а конкретные self/around
//! маршруты сообщений остаются у `CGame`, владеющего network/session runtime.
//! Периодический `ComputeWarSoulXY` сохраняет вещественное состояние
//! следования, точные пороги смерти и мгновенного переноса, общий хвост карты
//! областей и последующий `0xBF605`; готовность конкретного навыка остаётся
//! входным фактом. Повреждённое нечисловое состояние блокируется
//! типизированным результатом до прежнего целочисленного преобразования x87.
//! `SetWarSoulXY/DelWarSoul` (player.cpp:13071/13109,
//! 0x0042DF50/0x0042E0A0) завершают выбранный незаконченный навык через
//! End(int,0), не bool-перегрузку и не удаление AI-команды. Этот callback
//! выполняет `CGame`: Set только при найденных region/target area, Delete
//! после проверки equipment[10] GAP_BF_BATTLE_FAIRY==1 до region gate.
//! `CServerRegion` меняет карту с signed /15; player point обновляется
//! только при найденной target area для Set либо прежней area для Delete,
//! независимо от результата CArea::AddWarSoul/DelWarSoul.
//! Periodic HP-death prefix `CPlayer::AI` повторно нормализует summon/state и
//! recall/died флаги нулевой по HP equipped fairy, затем вызывает
//! `PropertiesChanged`; `CGame` собирает exact `0xBF721` целиком из owned
//! combat/base wire, включая add-element-attack, RP/max-RP, max-vigour и exalt.
//! Оригинал в этой ветви не чистит stale area-map entry и не посылает status
//! broadcast; оба отсутствующих side effect сохранены.
//! `CEquipmentContainer::OnObjectRemoved` player-tail связывает снятие
//! headgear с exact `SetWarSoulStaus(0)`, девятью skill detach, пересчётом
//! свойств при уже отсутствующем slot-е, HP/MP clamp и `0xBF720`; полный
//! virtual property owner остаётся injected callback-границей.
//! Monster-death caller восстанавливает transient continuous-kill clock/count,
//! persisted `wHitTopLog`, milestone EXP и exact `0xBF706/0xBF707` wire.
//! Потеря цели возвращает ended-навык к exact occupation/equipment-dependent
//! default `1/2/3`; active execution при этом остаётся отдельной AI-проекцией.
//! GodsBattle player snapshot теперь также хранит persisted faction/SZL;
//! faction membership появляется только в concrete region AddObject-tail и
//! удаляется его RemoveObject/DelObj-tail, не при восстановлении snapshot-а.
//! `UpdateSZL` проходит через `CGame`: player property/notice предшествуют
//! decrease-only appellation check и script-effect-у `RequestChangeAppellation`.
//! `IncreaseRp` восстановлен в reached combat path: профессия/уровневые
//! пороги, fixed attack gain, шесть damage/max-HP tiers, два последовательных
//! clamp-а и каждый исходный `PropertiesChanged` сохраняются.
//! Симметричный `OnObjectAdded` создаёт particular state только при ненулевом
//! inherited `m_pFather`; затем сохраняет late-block partial mutations, после
//! commit добавляет девять war-soul skills, пересчитывает свойства, публикует
//! `0xBF720` с исключением owner-а и отражает даже zero-delta `PackExpand` log.

use super::ai::playerai::CPlayerAI;
use super::area::WarSoulPoint;
use super::container::camountlimitgoodscontainer::{
    AmountLimitGoodsAdded, AmountLimitGoodsCodecError, AmountLimitGoodsRemoved,
    AmountLimitGoodsTaken, CAmountLimitGoodsContainer,
};
use super::container::camountlimitgoodsshadowcontainer::{
    AmountShadowAdded, CAmountLimitGoodsShadowContainer,
};
use super::container::cbank::{BankGoodsAddOutcome, CBank};
use super::container::cbattlefairycontainer::{
    BattleFairyCell, BattleFairyCombineCheck, BattleFairyContainerAddOutcome,
    BattleFairyDefaultGoodsUpdate, BattleFairyDefaultSkill,
    BattleFairyPropertyAddEffect, BattleFairyUpgradeConsumedGem, CBattleFairyContainer,
};
use super::container::ccontainer::ContainerListenerHandle;
use super::container::ccontainer::PreviousContainer;
use super::container::cdepot::CDepot;
use super::container::cequipmentcontainer::{
    CEquipmentContainer, EquipmentAddOutcome, EquipmentAddRuntimeFacts, EquipmentAroundUpdate,
    EquipmentColumn, EquipmentContainerCodecError, EquipmentOwnerPlayerFacts,
    EquipmentRemoveOutcome, EquipmentRemoveRuntimeFacts, EquipmentUnserializedEntry,
};
use super::container::cfairycontainer::{CFairyContainer, FairyContainerCodecError};
use super::container::cgoodscontainer::GoodsStackMergeOutcome;
use super::container::cgoodsshadowcontainer::{PlacedShadowGoods, ShadowRecordBlock};
use super::container::cjifen::CJiFen;
use super::container::cvolumelimitgoodscontainer::{
    CVolumeLimitGoodsContainer, VolumeGoodsAddOutcome, VolumeGoodsCodecError,
    VolumeGoodsRemoveOutcome, VolumeGoodsSwapOutcome,
};
use super::container::cwallet::{
    CWallet, CurrencyCodecError, CurrencyDecreaseOutcome, CurrencyGoodsAddOutcome,
    CurrencyGoodsTaken, CurrencyIncreaseOutcome,
};
use super::container::cyuanbao::CYuanBao;
use super::gameeffectjournal::{GameEffect, GameEffectJournal};
use super::goods::cbattlefairyproperty::BattleFairyCompose;
use super::goods::cgoods::CGoods;
use super::goods::cgoodsbaseproperties::{
    EQUIP_PLACE_HEADGEAR, GAP_AGILITY_CORRECTION, GAP_ANIMA_BIND, GAP_ARMOR_CORRECTION, GAP_ATTACK_AVOID,
    GAP_ATTACK_SPEED_CORRECTION, GAP_BF_ABRAVE_ADDON, GAP_BF_AGILITY, GAP_BF_AGILITY_ADDON,
    GAP_BF_AGILITY_BASE, GAP_BF_AGILITY_POTENTIAL, GAP_BF_ALL_SKILL, GAP_BF_ATTACK,
    GAP_BF_ATTACK_ADDON,
    GAP_BF_ATTACK_BASE, GAP_BF_ATTACK_POTENTIAL, GAP_BF_BATTLE_FAIRY, GAP_BF_BLAST,
    GAP_BF_BLAST_ADDON, GAP_BF_BLAST_POTENTIAL, GAP_BF_BRAVE, GAP_BF_BRAVE_BASE,
    GAP_BF_BRAVE_POTENTIAL, GAP_BF_CUT_HURT_ADDON,
    GAP_BF_CUT_HURT_SCALE, GAP_BF_EARTH, GAP_BF_EARTH_SKILL, GAP_BF_HP, GAP_BF_HUOXIESHU_SKILL,
    GAP_BF_LEVEL, GAP_BF_LIFE_ADDON, GAP_BF_LINGZHISHU_SKILL, GAP_BF_MAN, GAP_BF_MAN_SKILL,
    GAP_BF_MAX_HP, GAP_BF_MAX_MP, GAP_BF_MP, GAP_BF_MP_ADDON, GAP_BF_POTENTIAL,
    GAP_BF_PULLULATERATE, GAP_BF_SKY, GAP_BF_SKY_SKILL, GAP_BF_SPRITE, GAP_BF_SPRITE_ADDON,
    GAP_BF_SPRITE_BASE, GAP_BF_SPRITE_POTENTIAL, GAP_BF_SPRITUALISE_ADDON,
    GAP_BF_SPRITUALISM, GAP_BF_SPRITUALISM_BASE, GAP_BF_SPRITUALISM_POTENTIAL,
    GAP_BF_STRENGH, GAP_BF_STRENGH_ADDON, GAP_BF_STRENGH_BASE, GAP_BF_STRENGH_POTENTIAL,
    GAP_BF_WEAPON_LEVEL, GAP_BLAST_ATTACK, GAP_BLAST_ELEMENT_ATTACK,
    GAP_BREAK_ARMOUR, GAP_BREAK_BOUND, GAP_BREAK_ELEMENT, GAP_BURDEN_UPPER_LIMIT_CORRECTION,
    GAP_CIQING_PROPERTY1, GAP_CIQING_PROPERTY2,
    GAP_CONSTITUTION_CORRECTION, GAP_DODGE_CORRECTION, GAP_ELEMENT_ATTACK_CORRECTION,
    GAP_ELEMENT_AVOID, GAP_ELEMENT_RESISTANCE_CORRECTION, GAP_FATAL_BLOW_RATE_CORRECTION,
    GAP_EQUIP_ACTIVE, GAP_EXCEPTION_STATE, GAP_FULL_MISS, GAP_FUMO_PROPERTY, GAP_GEM_LEVEL,
    GAP_GOODS_BIND,
    GAP_FAIRY_AGILITY, GAP_FAIRY_HP, GAP_FAIRY_STRENGTH, GAP_FAIRY_WAKAN,
    GAP_GOODS_EQUIMENT_FLASH, GAP_GOODS_LIFE_TYPE, GAP_GOODS_MAXIMUM_DURABILITY,
    GAP_GOODS_PACKAGE_EXTENTION,
    GAP_GOLD_POWER, GAP_HIT_RATE_CORRECTION, GAP_HP_RESTORE_SPEED_CORRECTION,
    GAP_HP_UPPER_LIMIT_CORRECTION,
    GAP_MAXIMUM_ATTACK_CORRECTION, GAP_MINIMUM_ATTACK_CORRECTION, GAP_MOUNT_LEVEL, GAP_MOUNT_TYPE,
    GAP_MP_RESTORE_SPEED_CORRECTION, GAP_MP_UPPER_LIMIT_CORRECTION, GAP_PARTICULAR_ATTRIBUTE,
    GAP_REQUIRE_GENDER, GAP_REQUIRE_OCCUPATION, GAP_ROLE_MINIMUM_AGILITY_LIMIT,
    GAP_ROLE_MINIMUM_CONSTITUTION_LIMIT, GAP_ROLE_MINIMUM_LEVEL_LIMIT,
    GAP_ROLE_MINIMUM_STRENGTH_LIMIT, GAP_ROLE_MINIMUM_WAKAN_LIMIT,
    GAP_PUNCTURE, GAP_STIFFEN_PROBABILITY_CORRECTION, GAP_STRENGTH_CORRECTION,
    GAP_WAKAN_CORRECTION,
    GAP_WEAPON_CATEGORY, GAP_WEAPON_LEVEL, GOODS_TYPE_CONSUMABLE, GOODS_TYPE_EQUIPMENT,
};
use super::goods::cgoodsfactory::CGoodsFactory;
use super::legacycodec::{LegacyReader, LegacyWriter};
use super::listener::cgoodsparticularpropertylistener::GoodsParticularPropertyListener;
use super::moveshape::{
    CMoveShape, MoveShapeCommandBlock, MoveShapePositionFacts,
    MoveShapeSkill, SKILL_BASE_DEFENSE,
};
use super::script::variablelist::{
    CVariableList, GameVariableMutationOutcome, GameVariableSnapshotError,
};
use super::serverregion::CServerRegion;
use super::shape::{
    CShape, ShapeCoordinateBlock, ShapeDecodeError, ShapeFigure, ShapeIdentity, ShapeView,
};
use super::skills::archery::ARCHERY_SKILL_ID;
use super::skills::baseattack::BASE_ATTACK_SKILL_ID;
use super::skills::basemagic::BASE_MAGIC_SKILL_ID;
use super::skills::skillfactory::{CSkillFactory, SkillCategory, UNKNOWN_SKILL_ID};
use super::states::automaticrestore::AutomaticRestoreMutation;
use super::teamstate::CTeamState;
use crate::nets::netserver::message::GameServerAroundRuntime;
use crate::public::auctionnode::CGoodsNode;
use crate::public::guid::CGuid;
use crate::public::taozhuangsetup::CTaoZhuangSetup;
use crate::setup::globesetup::{GlobePlayerPropertyCoefficients, GlobeSetupSnapshot};
use crate::setup::hitlevelsetup::HitLevelEntry;
use crate::setup::questsystem::CQuestSystem;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use bitflags::bitflags;
use thiserror::Error;
use tracing::trace;

const PLAYER_TYPE: i32 = 400;
const PLAYER_BASE_PROPERTY_WIRE_SIZE: usize = 0x194;
const BASE_LEVEL_OFFSET: usize = 0x04;
const BASE_EXPERIENCE_OFFSET: usize = 0x08;
const BASE_HEAD_PICTURE_OFFSET: usize = 0x0c;
const BASE_FACE_PICTURE_OFFSET: usize = 0x0d;
const BASE_OCCUPATION_OFFSET: usize = 0x0e;
const BASE_SEX_OFFSET: usize = 0x0f;
const BASE_PK_COUNT_OFFSET: usize = 0x1c;
const BASE_KILL_COUNT_OFFSET: usize = 0x20;
const BASE_HIT_TOP_LOG_OFFSET: usize = 0x24;
const BASE_CHARGED_OFFSET: usize = 0x38;
const BASE_REMAIN_POINT_OFFSET: usize = 0x3a;
const BASE_HOTKEY_OFFSET: usize = 0x3c;
const BASE_PK_NORMAL_OFFSET: usize = 0x9c;
const BASE_PK_TEAM_OFFSET: usize = 0x9d;
const BASE_PK_UNION_OFFSET: usize = 0x9e;
const BASE_PK_BADMAN_OFFSET: usize = 0x9f;
const BASE_PK_COUNTRY_OFFSET: usize = 0xa0;
const BASE_HEALTH_OFFSET: usize = 0xa4;
const BASE_MANA_OFFSET: usize = 0xa8;
const BASE_RP_OFFSET: usize = 0xac;
const BASE_YP_OFFSET: usize = 0xae;
const BASE_MAXIMUM_HP_OFFSET: usize = 0xb0;
const BASE_MAXIMUM_MP_OFFSET: usize = 0xb4;
const BASE_MAXIMUM_YP_OFFSET: usize = 0xb8;
const BASE_MAXIMUM_RP_OFFSET: usize = 0xba;
const BASE_STRENGTH_OFFSET: usize = 0xbc;
const BASE_DEXTERITY_OFFSET: usize = 0xc0;
const BASE_CONSTITUTION_OFFSET: usize = 0xc4;
const BASE_INTELLIGENCE_OFFSET: usize = 0xc8;
const BASE_MINIMUM_ATTACK_OFFSET: usize = 0xcc;
const BASE_MAXIMUM_ATTACK_OFFSET: usize = 0xd0;
const BASE_HIT_OFFSET: usize = 0xd4;
const BASE_BURDEN_OFFSET: usize = 0xd6;
const BASE_CCH_OFFSET: usize = 0xd8;
const BASE_DEFENSE_OFFSET: usize = 0xdc;
const BASE_DODGE_OFFSET: usize = 0xe0;
const BASE_ATTACK_SPEED_OFFSET: usize = 0xe2;
const BASE_ELEMENT_RESISTANCE_OFFSET: usize = 0xe4;
const BASE_HP_RECOVERY_OFFSET: usize = 0xe8;
const BASE_MP_RECOVERY_OFFSET: usize = 0xea;
const BASE_VIGOUR_OFFSET: usize = 0xec;
const BASE_MAXIMUM_VIGOUR_OFFSET: usize = 0xf0;
const BASE_ENERGY_OFFSET: usize = 0xf4;
const BASE_MAXIMUM_ENERGY_OFFSET: usize = 0xf8;
const BASE_CREDIT_OFFSET: usize = 0xfc;
const BASE_EXALT_OFFSET: usize = 0x100;
const BASE_DISPLAY_HEAD_PIECE_OFFSET: usize = 0x104;
const BASE_QUEST_TIME_BEGIN_OFFSET: usize = 0x108;
const BASE_QUEST_TIME_LIMIT_OFFSET: usize = 0x10c;
const BASE_QUEST_ENABLED_OFFSET: usize = 0x110;
const BASE_EXPLOIT_OFFSET: usize = 0x114;
const BASE_FAIRY_CONTAINER_ENABLED_OFFSET: usize = 0x11c;
const BASE_BATTLE_FAIRY_ENABLED_OFFSET: usize = 0x128;
const BASE_BREAK_ARMOUR_OFFSET: usize = 0x12c;
const BASE_PUNCTURE_OFFSET: usize = 0x130;
const BASE_BREAK_ELEMENT_OFFSET: usize = 0x134;
const BASE_BREAK_BOUND_OFFSET: usize = 0x138;
const BASE_POWER_OF_GOLD_OFFSET: usize = 0x13c;
const BASE_DAYS_HONOR_OFFSET: usize = 0x140;
const BASE_WEEKS_HONOR_OFFSET: usize = 0x144;
const BASE_MONTHS_HONOR_OFFSET: usize = 0x148;
const BASE_TOTAL_HONOR_OFFSET: usize = 0x14c;
const BASE_RANK_OF_NOBILITY_OFFSET: usize = 0x150;
const BASE_APPELLATION_OFFSET: usize = 0x154;
const BASE_MODE_OFFSET: usize = 0x158;
const BASE_FETCH_POWER_OFFSET: usize = 0x164;
const BASE_BATTLE_FAIRY_SUMMONED_OFFSET: usize = 0x16c;
const BASE_BATTLE_FAIRY_RECALL_OFFSET: usize = 0x16d;
const BASE_BATTLE_FAIRY_DIED_OFFSET: usize = 0x16e;
const BASE_AUCTION_SPACE_OFFSET: usize = 0x170;
const BASE_JJC_LEVEL_OFFSET: usize = 0x174;
const BASE_JJC_SCORE_OFFSET: usize = 0x178;
const BASE_FY_ENERGY_OFFSET: usize = 0x17c;
const BASE_FY_ENABLE_FLAGS_OFFSET: usize = 0x180;
const BASE_LT_UP_60_COUNT_OFFSET: usize = 0x184;
const BASE_REMAIN_JING_LI_DAN_COUNT_OFFSET: usize = 0x186;
const BASE_LT_60_STAMP_OFFSET: usize = 0x188;
const BASE_SZL_OFFSET: usize = 0x18c;
const BASE_GODS_BATTLE_FACTION_OFFSET: usize = 0x190;
const LEGACY_COMBAT_MAXIMUM: u32 = i32::MAX as u32;
const CONTRIBUTION_MINIMUM: i32 = -2_000_000_000;
const CONTRIBUTION_MAXIMUM: i32 = 2_000_000_000;
const BATTLE_FAIRY_SKILL_ADDED_MESSAGE_TYPE: u32 = 0x0b_f71d;
const BATTLE_FAIRY_FETCH_POWER_MESSAGE_TYPE: u32 = 0x0b_f80c;
const BATTLE_FAIRY_CONTAINER_EXTEND_ID: u32 = 0x0c;
const MONSTER_TAMING_SKILL_ID: u32 = 0xd4;
const BATTLE_FAIRY_MOVE_MESSAGE_TYPE: u32 = 0x0b_f605;
const BATTLE_FAIRY_STATUS_MESSAGE_TYPE: u32 = 0x0b_f930;
const BATTLE_FAIRY_SUMMON_MESSAGE_TYPE: u32 = 0x0b_f92e;
const BATTLE_FAIRY_SKILL_REMOVED_MESSAGE_TYPE: u32 = 0x0b_f71e;
const BATTLE_FAIRY_SKILL_RESET_ITEM_MISSING: &str = "ZHGS0022";
const SKILL_EFFECT_MESSAGE_TYPE: u32 = 0x0b_fe01;
const SKILL_REJECT_REASON: u32 = 0;
const SKILL_REJECT_WAR_SOUL_REASON: u32 = 4;
const SKILL_REJECT_CODE: u8 = 0x0c;
const SKILL_POJIA: u32 = 530;
const SKILL_LEIMING: u32 = 543;
const SKILL_ID_MASK: u32 = i32::MAX as u32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyObjectMoveOperation {
    Delete,
    New,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyObjectMove {
    pub(crate) operation: BattleFairyObjectMoveOperation,
    pub(crate) player_id: i32,
    pub(crate) container_extend_id: u32,
    pub(crate) goods: super::shape::ShapeIdentity,
    pub(crate) position: u32,
    pub(crate) amount: u32,
    pub(crate) old_client_payload: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySkillAdded {
    pub(crate) message_type: u32,
    pub(crate) player_id: i32,
    pub(crate) skill_id: u32,
    pub(crate) skill_level: i32,
    pub(crate) skill_type: u32,
    pub(crate) skill_name: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyAuditLog {
    pub(crate) string_id: &'static str,
    pub(crate) account: Vec<u8>,
    pub(crate) goods_name: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyCombineEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    FetchPowerChanged {
        message_type: u32,
        player_id: i32,
        subject_id: i32,
        property_name: &'static str,
        value: u32,
    },
    ObjectMove(BattleFairyObjectMove),
    SkillAdded(BattleFairySkillAdded),
    GoodsUpdated(BattleFairyDefaultGoodsUpdate),
    Audit(BattleFairyAuditLog),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyCombineOutcome {
    FeatureDisabled,
    Rejected,
    InsufficientFetchPower,
    InputRemovalStopped,
    Failed,
    CreationFailed,
    CreationRejected,
    Created,
}

#[must_use = "combine report содержит последовательность адресных packet/log effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyCombineReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyCombineOutcome,
    pub(crate) effects: GameEffectJournal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyWarSoulAction {
    SetPosition {
        previous: WarSoulPoint,
        target: WarSoulPoint,
    },
    Delete {
        previous: WarSoulPoint,
        player_position: WarSoulPoint,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WarSoulHitOutcome {
    pub(crate) broken: bool,
    pub(crate) broadcast_previous_status: bool,
    pub(crate) update: super::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySummonOutcome {
    FeatureDisabled,
    AlreadySummoned,
    AlreadyRecalled,
    MissingHeadgear,
    InvalidHeadgear,
    NoHitPoints,
    ActivePet,
    MonsterTamingActive,
    CoordinateBlocked(ShapeCoordinateBlock),
    Summoned,
    Recalled,
    IgnoredMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySummonEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    AroundMessage {
        message_type: u32,
        player_id: i32,
        values: Vec<i32>,
    },
    PropertiesChanged {
        player_id: i32,
    },
}

#[must_use = "summon report хранит точный порядок адресных broadcast и property effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySummonReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairySummonOutcome,
    pub(crate) region_id: Option<i32>,
    pub(crate) spatial_action: Option<BattleFairyWarSoulAction>,
    pub(crate) effects: GameEffectJournal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyFollowOutcome {
    ActiveSkill,
    NotSummoned,
    Dead,
    CoordinateBlocked(ShapeCoordinateBlock),
    NonFiniteVisualState,
    InsideDeadZone,
    Moved,
    Snapped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyFollowEffect {
    AroundMove {
        message_type: u32,
        player_id: i32,
        object_type: i32,
        x: u32,
        y: u32,
    },
}

#[must_use = "план следования содержит пространственное действие и обязательную рассылку движения"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyFollowPlan {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyFollowOutcome,
    pub(crate) region_id: Option<i32>,
    pub(crate) spatial_action: Option<BattleFairyWarSoulAction>,
    pub(crate) effects: GameEffectJournal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyDeathOutcome {
    MissingHeadgear,
    NotBattleFairy,
    Alive,
    Died,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEquipmentRemoveRuntimeFacts {
    pub(crate) pack_add_enabled: bool,
    pub(crate) player_goods_package_extension: Option<u32>,
    pub(crate) active_war_soul_blocks_headgear: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerEquipmentRemoveEffect {
    WarSoulStatusAround {
        message_type: u32,
        player_id: i32,
        values: [i32; 2],
    },
    WarSoulSkillDetached {
        skill_id: u32,
    },
    SkillRemoved(BattleFairySkillRemoved),
    PropertiesChangedWithoutRemovedSlot {
        column: EquipmentColumn,
        combat_properties: PlayerCombatProperties,
        ci_qing_result_values: BTreeMap<u32, u32>,
    },
    VitalsClamped {
        previous_health: u32,
        current_health: u32,
        previous_mana: u32,
        current_mana: u32,
    },
    AroundUpdate(EquipmentAroundUpdate),
}

#[must_use = "equipment remove report сохраняет container ownership и player/network tail"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEquipmentRemoveReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: EquipmentRemoveOutcome,
    pub(crate) effects: GameEffectJournal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEquipmentAddRuntimeFacts {
    pub(crate) pack_add_enabled: bool,
    pub(crate) now: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerEquipmentAddEffect {
    WarSoulSkillAttached {
        skill_id: u32,
        level: i32,
    },
    SkillAdded(BattleFairySkillAdded),
    PropertiesChanged {
        combat_properties: PlayerCombatProperties,
        ci_qing_result_values: BTreeMap<u32, u32>,
    },
    AroundUpdate(EquipmentAroundUpdate),
    PackageExtensionLogged {
        category: &'static str,
        string_id: &'static str,
        expanded_package_num: u32,
    },
}

#[must_use = "equipment add report сохраняет исход частичных изменений контейнера"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEquipmentAddReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: EquipmentAddOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyEquipmentMutationOutcome {
    Added(BattleFairyContainerAddOutcome),
    Removed(VolumeGoodsRemoveOutcome),
    MissingGoods,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyEquipmentMutationEffect {
    PropertiesChanged { player_id: i32 },
    BattleFairyUpdated(BattleFairyDefaultGoodsUpdate),
}

#[must_use = "equipment report сохраняет container ownership и ранние property effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyEquipmentMutationReport {
    pub(crate) player_id: i32,
    pub(crate) cell: Option<BattleFairyCell>,
    pub(crate) property_applied: bool,
    pub(crate) outcome: BattleFairyEquipmentMutationOutcome,
    pub(crate) effects: GameEffectJournal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyPotentialAllocationOutcome {
    MissingHeadgear,
    InvalidHeadgear,
    AggregateInsufficient,
    Processed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyPotentialAllocationEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    PropertiesChanged {
        player_id: i32,
    },
    GoodsUpdated(BattleFairyDefaultGoodsUpdate),
}

#[must_use = "allocation report сохраняет ordered player и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyPotentialAllocationReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyPotentialAllocationOutcome,
    pub(crate) effects: GameEffectJournal,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct BattleFairyUpgradeLogGates {
    pub(crate) success: bool,
    pub(crate) failure: bool,
    pub(crate) lost_target: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyUpgradeOutcome {
    MissingRegion,
    InsufficientMoney,
    InvalidEquipment,
    MissingBaseGem,
    GemLevelMismatch,
    MaximumLevel,
    Succeeded,
    FailedKept,
    FailedDowngraded,
    FailedReset,
    FailedDestroyed,
    ConsumptionStopped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyUpgradeGoodsSnapshot {
    pub(crate) identity: super::shape::ShapeIdentity,
    pub(crate) name: Vec<u8>,
    pub(crate) price: u32,
    pub(crate) amount: u32,
}

impl BattleFairyUpgradeGoodsSnapshot {
    fn capture(goods: &CGoods) -> Self {
        Self {
            identity: goods.identity(),
            name: goods.name().to_vec(),
            price: goods.price(),
            amount: goods.amount(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyUpgradePlayerSnapshot {
    pub(crate) pk_count: u16,
    pub(crate) money: u32,
    pub(crate) depot_money: u32,
    pub(crate) region_id: i32,
    pub(crate) tile_x: i32,
    pub(crate) tile_y: i32,
    pub(crate) client_ip: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyUpgradeEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
        format_value: Option<u32>,
    },
    MoneyChanged {
        player_id: i32,
        previous: u32,
        current: u32,
        outcome: CurrencyDecreaseOutcome,
    },
    GoodsUpdated(BattleFairyDefaultGoodsUpdate),
    GemConsumed {
        player_id: i32,
        consumed: BattleFairyUpgradeConsumedGem,
    },
    TargetDeleted {
        player_id: i32,
        goods: BattleFairyUpgradeGoodsSnapshot,
        position: u32,
        removal: VolumeGoodsRemoveOutcome,
    },
    Audit {
        message_type: u32,
        event: u8,
        player_id: i32,
        player: BattleFairyUpgradePlayerSnapshot,
        target: BattleFairyUpgradeGoodsSnapshot,
        gems: [Option<BattleFairyUpgradeGoodsSnapshot>; 4],
    },
}

#[must_use = "upgrade report содержит wallet, ownership и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyUpgradeReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyUpgradeOutcome,
    pub(crate) effects: GameEffectJournal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyPotentialResetOutcome {
    FeatureDisabled,
    MissingHeadgear,
    InvalidHeadgear,
    MissingResetItem,
    Reset,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyPotentialResetEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    PacketItemConsumed {
        player_id: i32,
        goods: super::shape::ShapeIdentity,
        position: Option<u32>,
        previous_amount: u32,
        remaining_amount: u32,
        consumed: bool,
        removal: Option<VolumeGoodsRemoveOutcome>,
    },
    PropertiesChanged {
        player_id: i32,
    },
    GoodsUpdated(BattleFairyDefaultGoodsUpdate),
}

#[must_use = "reset report содержит packet ownership и player/network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyPotentialResetReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyPotentialResetOutcome,
    pub(crate) effects: GameEffectJournal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySkillResetOutcome {
    FeatureDisabled,
    MissingHeadgear,
    InvalidHeadgear,
    MissingResetItem,
    InvalidPosition,
    SelectedSkillUnavailable,
    Reset,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySkillRemoved {
    pub(crate) message_type: u32,
    pub(crate) player_id: i32,
    pub(crate) skill_id: u32,
    pub(crate) skill_name: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySkillResetEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    PacketItemConsumed {
        player_id: i32,
        goods: super::shape::ShapeIdentity,
        position: Option<u32>,
        previous_amount: u32,
        remaining_amount: u32,
        consumed: bool,
        removal: Option<VolumeGoodsRemoveOutcome>,
    },
    SkillRemoved(BattleFairySkillRemoved),
    SkillAdded(BattleFairySkillAdded),
    SelectedSkillLearned(BattleFairySkillAdded),
    GoodsUpdated(BattleFairyDefaultGoodsUpdate),
}

#[must_use = "skill reset report содержит packet, skill-state и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySkillResetReport {
    pub(crate) player_id: i32,
    pub(crate) position: i32,
    pub(crate) outcome: BattleFairySkillResetOutcome,
    pub(crate) effects: GameEffectJournal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerSkillRequest {
    pub(crate) raw_skill_id: i32,
    pub(crate) target_type: i32,
    pub(crate) target_id: i32,
    pub(crate) target_x: i32,
    pub(crate) target_y: i32,
}

impl PlayerSkillRequest {
    pub(crate) const fn skill_id(self) -> u32 {
        self.raw_skill_id as u32 & SKILL_ID_MASK
    }
}

/// Facts reached `CGame` owner-а: region virtual и canonical AI разрешаются
/// непосредственно, unknown polymorphic target остаётся process-границей.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerSkillRequestFacts {
    pub(crate) symbol_attackable: bool,
    pub(crate) player_ai_available: bool,
    pub(crate) object_target_available: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerSkillDispatch {
    SelfTarget {
        skill_id: u32,
        player_id: i32,
    },
    Point {
        skill_id: u32,
        x: i32,
        y: i32,
    },
    Object {
        skill_id: u32,
        target: super::shape::ShapeIdentity,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySkillRequest {
    pub(crate) raw_skill_id: i32,
    pub(crate) target_type: i32,
    pub(crate) target_id: i32,
    pub(crate) property_offset: i32,
    pub(crate) target_x: i32,
    pub(crate) target_y: i32,
}

impl BattleFairySkillRequest {
    pub(crate) const fn skill_id(self) -> u32 {
        self.raw_skill_id as u32 & SKILL_ID_MASK
    }
}

/// Facts reached `CGame` owner-а: region virtual и canonical AI разрешаются
/// непосредственно, unknown polymorphic target остаётся process-границей.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct BattleFairySkillRequestFacts {
    pub(crate) symbol_attackable: bool,
    pub(crate) player_ai_available: bool,
    pub(crate) object_target_available: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySkillDispatch {
    SelfTarget {
        skill_id: u32,
        skill_level: i32,
        player_id: i32,
    },
    Point {
        skill_id: u32,
        skill_level: i32,
        x: i32,
        y: i32,
    },
    Object {
        skill_id: u32,
        skill_level: i32,
        target: super::shape::ShapeIdentity,
    },
}

macro_rules! skill_dispatch_request {
    ($($dispatch:ty),+ $(,)?) => {$(
        impl $dispatch {
            const fn pending_request_key(self) -> (u32, u8, i32, i32) {
                match self {
                    Self::SelfTarget { skill_id, player_id, .. } => (skill_id, 2, PLAYER_TYPE, player_id),
                    Self::Point { skill_id, x, y, .. } => (skill_id, 1, x, y),
                    Self::Object { skill_id, target, .. } =>
                        (skill_id, 2, target.object_type, target.id),
                }
            }

            pub(crate) const fn skill_id(self) -> u32 {
                self.pending_request_key().0
            }

            pub(crate) fn same_pending_request(self, other: Self) -> bool {
                self.pending_request_key() == other.pending_request_key()
            }

            pub(crate) const fn object_target(self) -> Option<ShapeIdentity> {
                match self {
                    Self::SelfTarget { player_id, .. } => Some(ShapeIdentity {
                        object_type: PLAYER_TYPE,
                        id: player_id,
                        ex_id: CGuid::GUID_INVALID,
                    }),
                    Self::Object { target, .. } => Some(target),
                    Self::Point { .. } => None,
                }
            }

            /// CBaseAI::HasTarget (0x004C7DD0): знак важен для type/id,
            /// координаты проверяются только на ноль, не на границы региона.
            pub(crate) const fn has_target(self) -> bool {
                match self {
                    Self::Point { x, y, .. } => x != 0 && y != 0,
                    _ => match self.object_target() {
                        Some(target) => target.object_type > 0 && target.id > 0,
                        None => false,
                    },
                }
            }
        }
    )+};
}

skill_dispatch_request!(PlayerSkillDispatch, BattleFairySkillDispatch);

impl PlayerSkillDispatch {
    /// При отсутствии текущего CSkill OnSchedule выбирает default owner,
    /// но сохраняет уже извлечённую цель; ожидающий FIFO не меняется.
    pub(crate) const fn with_skill_id(mut self, selected: u32) -> Self {
        match &mut self {
            Self::SelfTarget { skill_id, .. }
            | Self::Point { skill_id, .. }
            | Self::Object { skill_id, .. } => *skill_id = selected,
        }
        self
    }
}

impl BattleFairySkillDispatch {
    pub(crate) const fn skill_level(self) -> i32 {
        match self {
            Self::SelfTarget { skill_level, .. }
            | Self::Point { skill_level, .. }
            | Self::Object { skill_level, .. } => skill_level,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyManaSpendOutcome {
    MissingEquipment,
    SpentWithoutWarSoul,
    Spent {
        update: Option<BattleFairyDefaultGoodsUpdate>,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct BattleFairyGearAddons {
    attack: i32,
    sprite: i32,
    strength: i32,
    brave: i32,
    agility: i32,
    spiritualism: i32,
    blast: i32,
    cut_hurt: i32,
    life: i32,
    mana: i32,
}

impl BattleFairyGearAddons {
    fn read(goods: &CGoods, factory: &CGoodsFactory) -> Self {
        let value = |property_type| goods.addon_property_value(factory, property_type, 1);
        Self {
            attack: value(GAP_BF_ATTACK_ADDON),
            sprite: value(GAP_BF_SPRITE_ADDON),
            strength: value(GAP_BF_STRENGH_ADDON),
            brave: value(GAP_BF_ABRAVE_ADDON),
            agility: value(GAP_BF_AGILITY_ADDON),
            spiritualism: value(GAP_BF_SPRITUALISE_ADDON),
            blast: value(GAP_BF_BLAST_ADDON),
            cut_hurt: value(GAP_BF_CUT_HURT_ADDON),
            life: value(GAP_BF_LIFE_ADDON),
            mana: value(GAP_BF_MP_ADDON),
        }
    }
}

bitflags! {
    /// Подтверждённые LeiTing/FY-флаги в `u32` legacy-формата игрока.
    ///
    /// Неизвестные биты сохраняются через `from_bits_retain` и возвращаются в
    /// сетевой/DB формат без усечения; известные `0..=8` соответствуют порогам
    /// энергии и числу суточных подъёмов выше 60.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub(crate) struct LeiTingEnableFlags: u32 {
        const ENERGY_20 = 1 << 0;
        const ENERGY_60 = 1 << 1;
        const ENERGY_80 = 1 << 2;
        const ENERGY_100 = 1 << 3;
        const LT_UP_60_4 = 1 << 4;
        const LT_UP_60_10 = 1 << 5;
        const LT_UP_60_16 = 1 << 6;
        const LT_UP_60_22 = 1 << 7;
        const LT_UP_60_28 = 1 << 8;
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerBaseProperties {
    pub(crate) level: u8,
    pub(crate) occupation: u8,
    pub(crate) sex: u8,
    pub(crate) remain_point: u16,
    pub(crate) base_maximum_hp: u32,
    pub(crate) base_maximum_mp: u32,
    pub(crate) base_burden: u16,
    pub(crate) base_strength: u32,
    pub(crate) base_dexterity: u32,
    pub(crate) base_constitution: u32,
    pub(crate) base_intelligence: u32,
    pub(crate) pk_normal: bool,
    pub(crate) pk_team: bool,
    pub(crate) pk_union: bool,
    pub(crate) pk_badman: bool,
    pub(crate) pk_country: bool,
    pub(crate) pk_count: u16,
    pub(crate) kill_count: u32,
    pub(crate) hit_top_log: u16,
    pub(crate) experience: u32,
    pub(crate) vigour: u32,
    pub(crate) credit: u32,
    pub(crate) charged: bool,
    pub(crate) fairy_container_enabled: bool,
    pub(crate) battle_fairy_enabled: bool,
    pub(crate) break_armour: u32,
    pub(crate) puncture: u32,
    pub(crate) break_element: u32,
    pub(crate) break_bound: u32,
    pub(crate) power_of_gold: u32,
    pub(crate) hotkeys: [u32; 24],
    pub(crate) mode: u32,
    pub(crate) display_head_piece: bool,
    pub(crate) quest_time_begin: i32,
    pub(crate) quest_time_limit: i32,
    pub(crate) quest_enabled: bool,
    pub(crate) fy_enable_flags: LeiTingEnableFlags,
    pub(crate) fy_energy: u32,
    pub(crate) lt_60_stamp: u32,
    pub(crate) lt_up_60_count: u16,
    pub(crate) remain_jing_li_dan_count: u16,
    pub(crate) appellation_id: u32,
    pub(crate) head_picture: i32,
    pub(crate) face_picture: i32,
    pub(crate) health: u32,
    pub(crate) mana: u32,
    pub(crate) rp: u16,
    pub(crate) yp: u16,
    pub(crate) maximum_yp: u16,
    pub(crate) maximum_rp: u16,
    pub(crate) maximum_vigour: u32,
    pub(crate) energy: u32,
    pub(crate) maximum_energy: u32,
    pub(crate) exalt: u32,
    pub(crate) fetch_power: u32,
    pub(crate) battle_fairy_recall: bool,
    pub(crate) battle_fairy_died: bool,
    pub(crate) auction_space: u32,
    pub(crate) jjc_level: u32,
    pub(crate) jjc_score: u32,
    pub(crate) days_honor_eliminate: u32,
    pub(crate) weeks_honor_eliminate: u32,
    pub(crate) months_honor_eliminate: u32,
    pub(crate) total_honor_eliminate: u32,
    pub(crate) rank_of_nobility_id: u32,
    pub(crate) exploit: u32,
    pub(crate) gods_battle_faction: i32,
    pub(crate) szl: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerHonorSnapshot {
    pub(crate) rank_of_nobility_id: u32,
    pub(crate) appellation_id: u32,
    pub(crate) days_eliminate: u32,
    pub(crate) weeks_eliminate: u32,
    pub(crate) months_eliminate: u32,
    pub(crate) total_eliminate: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerFriend {
    pub(crate) name: Vec<u8>,
    pub(crate) online: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerLeiTingThing {
    pub(crate) thing_id: u16,
    pub(crate) count: u16,
    pub(crate) max_count: u16,
    pub(crate) point: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerLeiTingThingCountOutcome {
    Missing,
    Rejected {
        current: u16,
        requested: i32,
        maximum: u16,
    },
    Updated {
        previous_count: u16,
        current_count: u16,
        previous_energy: u32,
        current_energy: u32,
        daily_count_incremented: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerUncreatedPet {
    pub(crate) original_name: Vec<u8>,
    pub(crate) health: u32,
    pub(crate) level: u32,
    pub(crate) experience: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerUncreatedCarriage {
    pub(crate) original_name: Vec<u8>,
    pub(crate) script: Vec<u8>,
    pub(crate) health: u32,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("LeiTing snapshot обрывается на {field} в {offset}: нужно {needed}, доступно {available}")]
pub(crate) struct PlayerLeiTingDecodeBlock {
    pub(crate) field: &'static str,
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum PlayerGameSaveCodecError {
    #[error(transparent)]
    Shape(#[from] ShapeDecodeError),
    #[error(transparent)]
    Goods(#[from] AmountLimitGoodsCodecError),
    #[error(transparent)]
    Volume(#[from] VolumeGoodsCodecError),
    #[error(transparent)]
    Equipment(EquipmentContainerCodecError),
    #[error(transparent)]
    Fairy(#[from] FairyContainerCodecError),
    #[error(transparent)]
    Currency(#[from] CurrencyCodecError),
    #[error(transparent)]
    Variables(#[from] GameVariableSnapshotError),
    #[error(transparent)]
    LeiTing(#[from] PlayerLeiTingDecodeBlock),
    #[error("player save обрывается на {field} в {offset}: нужно {needed}, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    #[error("player save содержит отрицательное count {count} в {field}")]
    NegativeCount { field: &'static str, count: i32 },
    #[error("player save string {field} имеет длину {length} при максимуме {maximum}")]
    StringTooLong {
        field: &'static str,
        length: usize,
        maximum: usize,
    },
    #[error("player save collection {field} длиной {length} не представима")]
    CollectionTooLarge { field: &'static str, length: usize },
    #[error("player save содержит object type {object_type} вместо player")]
    WrongObjectType { object_type: i32 },
    #[error("player save отклонил equipment position {position}")]
    EquipmentRejected { position: u32 },
    #[error("player save codec {field} вернул false")]
    CodecReturnedFalse { field: &'static str },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerLoginGoodsLocation {
    Equipment,
    Packet,
    Hand,
    Auction,
    Depot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerFriendAddOutcome {
    Added,
    AlreadyPresent,
    LimitReached,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerDeathGoodsCandidate {
    pub(crate) location: PlayerGoodsAiLocation,
    pub(crate) goods_id: CGuid,
    pub(crate) amount: u32,
    pub(crate) price: u32,
    pub(crate) name: Vec<u8>,
    pub(crate) particular_on_death: bool,
    pub(crate) table_drop_allowed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerCriminalStateEndReason {
    Timeout,
    PkThresholdExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerCriminalStateEnd {
    pub(crate) player_id: i32,
    pub(crate) previous_timestamp_ms: u32,
    pub(crate) checked_at_ms: u32,
    pub(crate) pk_count: u16,
    pub(crate) reason: PlayerCriminalStateEndReason,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerMurdererSignDecrease {
    pub(crate) player_id: i32,
    pub(crate) pk_count: u16,
    pub(crate) kill_count: u32,
    pub(crate) checked_at_ms: u32,
    pub(crate) next_timestamp_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerLostDelayStarted {
    pub(crate) player_id: i32,
    pub(crate) fight_state_count: i32,
    pub(crate) timestamp_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerExitSilenceUpdate {
    pub(crate) previous_minutes: i32,
    pub(crate) previous_timestamp_minutes: u32,
    pub(crate) sampled_minutes: [Option<u32>; 3],
    pub(crate) remaining_minutes: i32,
    pub(crate) timestamp_minutes: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerFightStateTransition {
    pub(crate) player_id: i32,
    pub(crate) previous_count: i32,
    pub(crate) current_count: i32,
    pub(crate) entered_peace: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerRemoteSkillMutation {
    pub(crate) skill_id: u32,
    pub(crate) skill_level: i32,
    pub(crate) legacy_result: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerRemoteLevelMutation {
    pub(crate) player_id: i32,
    pub(crate) faction_id: i32,
    pub(crate) previous_level: u8,
    pub(crate) level: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HotkeyHandTransferOutcome {
    MissingHandGoods,
    NotConsumable,
    UnsupportedSource,
    Moved,
    RolledBack,
    GarbageCollected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HotkeyHandOwnershipEvent {
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) position: Option<u32>,
    pub(crate) amount: u32,
    pub(crate) listeners: Vec<ContainerListenerHandle>,
}

#[must_use = "hand transfer report сохраняет ownership, fallback add и object-move исход"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HotkeyHandTransferReport {
    pub(crate) source_container_extend_id: u32,
    pub(crate) source_position: u32,
    pub(crate) goods: Option<ShapeIdentity>,
    pub(crate) hand_removal: Option<HotkeyHandOwnershipEvent>,
    pub(crate) packet_adds: Vec<PlayerPacketAddOutcome>,
    pub(crate) currency_adds: Vec<CurrencyGoodsAddOutcome>,
    pub(crate) hand_rollback: Option<AmountLimitGoodsAdded>,
    pub(crate) outcome: HotkeyHandTransferOutcome,
}

#[must_use = "packet add сохраняет исход передачи предмета контейнеру"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerPacketAddOutcome {
    pub(crate) outcome: VolumeGoodsAddOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerHonorEliminateMutation {
    pub(crate) player_id: i32,
    pub(crate) previous: [u32; 4],
    pub(crate) current: [u32; 4],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerCombatProperties {
    pub(crate) maximum_hp: u32,
    pub(crate) maximum_mp: u32,
    pub(crate) maximum_yp: u16,
    pub(crate) maximum_rp: u16,
    pub(crate) strength: u32,
    pub(crate) dexterity: u32,
    pub(crate) constitution: u32,
    pub(crate) intelligence: u32,
    pub(crate) minimum_attack: u32,
    pub(crate) maximum_attack: u32,
    pub(crate) attack_speed: u16,
    pub(crate) hit: u16,
    pub(crate) dodge: u16,
    pub(crate) cch: u16,
    pub(crate) defense: u32,
    pub(crate) element_resistance: u32,
    pub(crate) add_element_attack: u32,
    pub(crate) hp_recovery: u16,
    pub(crate) mp_recovery: u16,
    pub(crate) burden: u16,
    pub(crate) reank: u16,
    pub(crate) attack_avoid: u16,
    pub(crate) element_avoid: u16,
    pub(crate) full_miss: u16,
    pub(crate) element_modify: i32,
    pub(crate) blast_attack: u16,
    pub(crate) blast_element_attack: u16,
    pub(crate) soul_resistance: u16,
    pub(crate) add_soul_attack: u16,
    pub(crate) blast_attack_scale_bits: u32,
    pub(crate) blast_defense_scale_bits: u32,
    pub(crate) element_blast_attack_scale_bits: u32,
    pub(crate) element_blast_defense_scale_bits: u32,
    pub(crate) full_miss_scale_bits: u32,
    pub(crate) critical_rate_bits: u32,
    pub(crate) resume_hp_peace: i32,
    pub(crate) resume_mp_peace: i32,
    pub(crate) resume_hp_fight: i32,
    pub(crate) resume_mp_fight: i32,
    pub(crate) restored_hp_peace: i32,
    pub(crate) restored_mp_peace: i32,
    pub(crate) restored_hp_fight: i32,
    pub(crate) restored_mp_fight: i32,
    pub(crate) battle_fairy_summoned: bool,
    pub(crate) battle_fairy_recall: bool,
    pub(crate) battle_fairy_died: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerPropertyRecompute {
    pub(crate) properties: PlayerCombatProperties,
    pub(crate) ci_qing_result_values: BTreeMap<u32, u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TaoZhuangSetEvaluation {
    pub(crate) set_id: u32,
    pub(crate) collected_all: bool,
    pub(crate) completion_script: Vec<u8>,
}

/// Exact `GetPlayerAllProperties` diagnostic projection. Числа хранят raw
/// DWORD vararg bits: конкретный `%d`/`%u` шаблона определяет их signed view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerAllPropertiesDiagnosticSnapshot {
    pub(crate) name: Vec<u8>,
    pub(crate) summary_words: [u32; 15],
    pub(crate) base_combat_words: [u32; 15],
    pub(crate) current_combat_words: [u32; 20],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerExpendableEffect {
    pub(crate) property_type: i32,
    pub(crate) value: i32,
    pub(crate) start_time_ms: u32,
    pub(crate) effect_time_ms: u32,
}

pub(crate) const PLAYER_COMBAT_PROPERTY_WIRE_SIZE: usize = 0x9c;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerStatAllocationState {
    pub(crate) sex: u8,
    pub(crate) occupation: u8,
    pub(crate) remain_point: u16,
    pub(crate) base_maximum_hp: u32,
    pub(crate) base_maximum_mp: u32,
    pub(crate) base_strength: u32,
    pub(crate) base_dexterity: u32,
    pub(crate) base_constitution: u32,
    pub(crate) base_intelligence: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerStatAllocationMutation {
    pub(crate) player_id: i32,
    pub(crate) selector: u8,
    pub(crate) stat_changed: bool,
    pub(crate) previous: PlayerStatAllocationState,
    pub(crate) current: PlayerStatAllocationState,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerPkPermissions {
    pub(crate) player: bool,
    pub(crate) teammate: bool,
    pub(crate) guild_member: bool,
    pub(crate) criminal: bool,
    pub(crate) country: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerPkPermissionMutation {
    pub(crate) player_id: i32,
    pub(crate) selector: i8,
    pub(crate) requested: bool,
    pub(crate) recognized: bool,
    pub(crate) changed: bool,
    pub(crate) previous: PlayerPkPermissions,
    pub(crate) current: PlayerPkPermissions,
}

impl PlayerCombatProperties {
    pub(crate) const fn blast_attack_scale(self) -> f32 {
        f32::from_bits(self.blast_attack_scale_bits)
    }

    pub(crate) const fn blast_defense_scale(self) -> f32 {
        f32::from_bits(self.blast_defense_scale_bits)
    }

    pub(crate) const fn full_miss_scale(self) -> f32 {
        f32::from_bits(self.full_miss_scale_bits)
    }

    pub(crate) const fn element_blast_attack_scale(self) -> f32 {
        f32::from_bits(self.element_blast_attack_scale_bits)
    }

    pub(crate) const fn element_blast_defense_scale(self) -> f32 {
        f32::from_bits(self.element_blast_defense_scale_bits)
    }

    pub(crate) const fn critical_rate(self) -> f32 {
        f32::from_bits(self.critical_rate_bits)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum PlayerProgress {
    #[default]
    None,
    Banking,
    Trading,
    Shopping,
    OpenStall,
    Increment,
    Upgrade,
    Synthesis,
    Mailing,
    DaKong,
    Compose,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoodsSessionPlayerRelease {
    pub(crate) previous_progress: PlayerProgress,
    pub(crate) previous_moveable_count: i32,
    pub(crate) resulting_moveable_count: i32,
    pub(crate) moveable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuctionSelfGoodsRefresh {
    Throttled {
        sampled_tick_ms: u32,
        previous_tick_ms: u32,
    },
    Requested {
        sampled_tick_ms: u32,
        recorded_tick_ms: u32,
        goods_space: u32,
        wallet_space: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingPacketConsumption {
    pub(crate) player_id: i32,
    pub(crate) goods: super::shape::ShapeIdentity,
    pub(crate) position: u32,
    pub(crate) previous_amount: u32,
    pub(crate) remaining_amount: u32,
    pub(crate) removal: Option<VolumeGoodsRemoveOutcome>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingPacketAddition {
    pub(crate) player_id: i32,
    pub(crate) source: super::shape::ShapeIdentity,
    pub(crate) position: Option<u32>,
    pub(crate) outcome: VolumeGoodsAddOutcome,
    pub(crate) old_client_payload: Option<Vec<u8>>,
    pub(crate) resulting_amount: Option<u32>,
}

#[must_use = "изменение YuanBao содержит обязательный container/client effect"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerYuanBaoChange {
    pub(crate) player_id: i32,
    pub(crate) previous: u32,
    pub(crate) current: u32,
    pub(crate) outcome: PlayerYuanBaoChangeOutcome,
}

#[must_use = "списание денег содержит wallet outcome для обязательного client effect"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerMoneyDecrease {
    pub(crate) previous: u32,
    pub(crate) current: u32,
    pub(crate) outcome: CurrencyDecreaseOutcome,
}

#[must_use = "результат bank transfer определяет ownership и lock/add side effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerBankCurrencyAddOutcome {
    Wallet(CurrencyGoodsAddOutcome),
    Bank(BankGoodsAddOutcome),
    AuctionWallet(CurrencyGoodsAddOutcome),
}

#[must_use = "изменение аукционных денег содержит wallet outcome для client effect"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerAuctionMoneyChange {
    pub(crate) player_id: i32,
    pub(crate) previous: u32,
    pub(crate) current: u32,
    pub(crate) outcome: PlayerAuctionMoneyChangeOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerAuctionMoneyChangeOutcome {
    Unchanged,
    Increased(CurrencyIncreaseOutcome),
    Decreased(CurrencyDecreaseOutcome),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AuctionMoneyMoveCapacity {
    pub(crate) wallet_amount: u32,
    pub(crate) auction_amount: u32,
    pub(crate) maximum: u32,
    pub(crate) allowed: bool,
}

#[must_use = "возврат с аукциона содержит container, bind и ownership outcome"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerAuctionGoodsReturn {
    pub(crate) player_id: i32,
    pub(crate) position: u32,
    pub(crate) source: ShapeIdentity,
    pub(crate) outcome: VolumeGoodsAddOutcome,
    pub(crate) resulting_goods: Option<ShapeIdentity>,
    pub(crate) resulting_amount: Option<u32>,
    pub(crate) bind_stored: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuctionBuyGate {
    Throttled {
        sampled_tick_ms: u32,
        previous_tick_ms: u32,
    },
    Ready {
        sampled_tick_ms: u32,
        recorded_tick_ms: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuctionListingGate {
    Throttled {
        sampled_tick_ms: u32,
        previous_tick_ms: u32,
    },
    Ready {
        sampled_tick_ms: u32,
        recorded_tick_ms: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerYuanBaoChangeOutcome {
    Unchanged,
    Increased(CurrencyIncreaseOutcome),
    Decreased(CurrencyDecreaseOutcome),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingContainerAddition {
    pub(crate) player_id: i32,
    pub(crate) container_extend_id: u32,
    pub(crate) position: u32,
    pub(crate) source: super::shape::ShapeIdentity,
    pub(crate) outcome: VolumeGoodsAddOutcome,
    pub(crate) old_client_payload: Option<Vec<u8>>,
    pub(crate) resulting_amount: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingContainerConsumption {
    pub(crate) player_id: i32,
    pub(crate) container_extend_id: u32,
    pub(crate) position: u32,
    pub(crate) goods: super::shape::ShapeIdentity,
    pub(crate) previous_amount: u32,
    pub(crate) remaining_amount: u32,
    pub(crate) removal: Option<VolumeGoodsRemoveOutcome>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingHandConsumption {
    pub(crate) player_id: i32,
    pub(crate) goods: super::shape::ShapeIdentity,
    pub(crate) previous_amount: u32,
    pub(crate) remaining_amount: u32,
    pub(crate) removal: Option<AmountLimitGoodsRemoved>,
}

#[must_use = "уничтожение hand goods содержит ownership и listener-эффекты удаления"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GoodsDestroyHandConsumption {
    pub(crate) player_id: i32,
    pub(crate) goods: super::shape::ShapeIdentity,
    pub(crate) previous_amount: u32,
    pub(crate) removed_amount: u32,
    pub(crate) remaining_amount: u32,
    pub(crate) removal: Option<AmountLimitGoodsRemoved>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EnhancementSelectionBlock {
    MissingGoods,
    UnsupportedSourceContainer,
    GoodsIdentityMismatch,
    GoodsAmountMismatch,
    StackableGoods,
    MissingBaseProperties,
    Shadow(ShadowRecordBlock),
}

#[must_use = "selection report связывает source container и AddShadow effect"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnhancementSelectionReport {
    pub(crate) goods: ShapeIdentity,
    pub(crate) source: PreviousContainer,
    pub(crate) shadow: AmountShadowAdded,
    pub(crate) previous_last_operated: (u32, u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EnhancementDeselectionBlock {
    MissingShadow,
    GoodsIdentityMismatch,
    GoodsAmountMismatch,
    MissingSourceGoods,
}

#[must_use = "deselection report сохраняет original source и RemoveShadow effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnhancementDeselectionReport {
    pub(crate) goods: ShapeIdentity,
    pub(crate) source: PreviousContainer,
    pub(crate) removed: super::container::cgoodsshadowcontainer::ShadowRemovedReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerTalkChannel {
    Normal,
    Area,
    Country,
    World,
    Private,
    Team,
    Union,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CPlayer {
    move_shape: CMoveShape,
    player_ai: CPlayerAI,
    figure: ShapeFigure,
    faction_id: i32,
    faction_logo_id: i32,
    faction_level: u16,
    faction_experience: i32,
    faction_force: i32,
    faction_contribute: u32,
    faction_master_id: i32,
    faction_name: Vec<u8>,
    faction_title: Vec<u8>,
    enemy_factions: BTreeSet<i32>,
    city_war_enemy_factions: BTreeSet<i32>,
    faction_owned_regions: Vec<[u8; 8]>,
    union_id: i32,
    union_master_id: i32,
    team_id: i32,
    team_captain: bool,
    country: u8,
    server_region_id: Option<i32>,
    in_changing_server: bool,
    in_changing_region: bool,
    last_enter_region_tick_ms: u32,
    entered_region: bool,
    state_before_server_region_change: u16,
    current_progress: PlayerProgress,
    personal_shop_session_id: i32,
    personal_shop_plug_id: i32,
    war_soul_state: u32,
    war_soul_point: WarSoulPoint,
    war_soul_visual_x_bits: u32,
    war_soul_visual_y_bits: u32,
    current_ticket: u32,
    goods_ai_tree: BTreeMap<u32, BTreeSet<CGuid>>,
    goods_ai_delete_queue: VecDeque<BTreeSet<CGuid>>,
    flash_previous: [u32; 17],
    flash_current: [u32; 17],
    flash_changed: bool,
    battle_fairy_summoned: bool,
    recreate_carriage: bool,
    active_carriage_id: i32,
    create_faction_operator: bool,
    apply_join_faction_operator: bool,
    faction_declare_operator: bool,
    active_pet_count: u32,
    attempt_appellation_id: u32,
    realm_appellation_skill_id: u32,
    realm_appellation_skill_level: i32,
    heart_request_sent: i32,
    heart_received: bool,
    friends: Vec<PlayerFriend>,
    quest_states: BTreeMap<u16, u8>,
    lei_ting_things: VecDeque<PlayerLeiTingThing>,
    uncreated_pets: Vec<PlayerUncreatedPet>,
    uncreated_carriage: PlayerUncreatedCarriage,
    login: bool,
    session_id: Vec<u8>,
    title: Vec<u8>,
    base_property_wire: [u8; PLAYER_BASE_PROPERTY_WIRE_SIZE],
    jjc_data: [u8; 0x10],
    jjc_pk_state: bool,
    fight_state_count: i32,
    auto_protected: bool,
    lost_time_stamp_ms: u32,
    continuous_kill_amount: u32,
    continuous_kill_timestamp_ms: u32,
    base_properties: PlayerBaseProperties,
    combat_properties: PlayerCombatProperties,
    combat_property_wire: [u8; PLAYER_COMBAT_PROPERTY_WIRE_SIZE],
    expendable_effects: BTreeMap<i32, PlayerExpendableEffect>,
    last_skill_item_use_ms: BTreeMap<u32, u32>,
    ci_qing_open: bool,
    ci_qing_list: BTreeSet<u32>,
    ci_qing_add_values: BTreeMap<u32, u32>,
    ci_qing_tao_zhuang_add_values: BTreeMap<u32, u32>,
    tao_zhuang_id: u32,
    equipment_changed: bool,
    tao_zhuang_setup_pending: bool,
    tao_zhuang_items: BTreeMap<u32, u32>,
    tao_zhuang_original_names: BTreeSet<Vec<u8>>,
    tao_zhuang_properties: BTreeMap<u32, u32>,
    ci_qing_tao_zhuang_properties: BTreeMap<u32, u32>,
    tao_zhuang_skills: BTreeMap<u32, u32>,
    contend_state: bool,
    emotion_index: i32,
    emotion_timestamp_ms: u32,
    city_war_died_state: bool,
    city_war_died_state_time_ms: i32,
    died_state_start_time_ms: u32,
    criminal_state_timestamp_ms: u32,
    murderer_time_stamp_ms: u32,
    ping_time: i32,
    last_ping_time_ms: u32,
    contribution: i32,
    silence_minutes: i32,
    silence_timestamp_minutes: u32,
    normal_talk_timestamp_ms: u32,
    area_talk_timestamp_ms: u32,
    world_talk_timestamp_ms: u32,
    country_talk_timestamp_ms: u32,
    private_talk_timestamp_ms: u32,
    team_talk_timestamp_ms: u32,
    union_talk_timestamp_ms: u32,
    money: u32,
    client_ip: u32,
    account: Vec<u8>,
    depot_password: Vec<u8>,
    last_container_script: Vec<u8>,
    variable_list: CVariableList,
    bank: CBank,
    depot: CDepot,
    hand: CAmountLimitGoodsContainer,
    enhancement: CAmountLimitGoodsShadowContainer,
    last_operated_container: u32,
    last_operated_goods_position: u32,
    packet: CVolumeLimitGoodsContainer,
    wallet: CWallet,
    yuan_bao: CYuanBao,
    ji_fen: CJiFen,
    equipment: CEquipmentContainer,
    auction_listing: CVolumeLimitGoodsContainer,
    auction_goods: CVolumeLimitGoodsContainer,
    auction_wallet: CWallet,
    auction_open: bool,
    auction_search_name: Vec<u8>,
    auction_search_lower_level: i32,
    auction_search_upper_level: i32,
    auction_search_use_self: i32,
    auction_search_money_type: i32,
    auction_search_weapon_type: i32,
    auction_current_page: i32,
    last_auction_limit_tick_ms: u32,
    last_auction_option_tick_ms: u32,
    current_auction_node: Option<CGoodsNode>,
    auction_listing_fee: u32,
    current_auction_buy_node: Option<CGoodsNode>,
    ci_qing: CVolumeLimitGoodsContainer,
    ci_qing_compose: CVolumeLimitGoodsContainer,
    fairy_container: CFairyContainer,
    battle_fairy_container: CBattleFairyContainer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerGoodsAiLocation {
    pub(crate) extend_id: i32,
    pub(crate) position: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerParticularGoodsDrop {
    pub(crate) location: PlayerGoodsAiLocation,
    pub(crate) goods_id: CGuid,
    pub(crate) amount: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerGoodsAiDeletion {
    pub(crate) location: PlayerGoodsAiLocation,
    pub(crate) goods: CGoods,
    pub(crate) previous_amount: u32,
    pub(crate) removed_amount: u32,
    pub(crate) remaining_amount: u32,
    pub(crate) listeners: Vec<ContainerListenerHandle>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerReliveMutation {
    pub(crate) player_id: i32,
    pub(crate) previous_x: i32,
    pub(crate) previous_y: i32,
    pub(crate) direction: i32,
    pub(crate) health: u32,
    pub(crate) mana: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerReliveOwnedPrelude {
    pub(crate) cleared_uncreated_pets: usize,
    pub(crate) cleared_uncreated_carriage: bool,
    pub(crate) previous_moveable_count: i32,
    pub(crate) resulting_moveable_count: i32,
    pub(crate) moveable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerContinuousKillUpdate {
    pub(crate) amount: u32,
    pub(crate) new_top_log: Option<u16>,
    pub(crate) bonus_experience: u32,
}

fn apply_equipment_goods_properties(
    properties: &mut PlayerCombatProperties,
    goods: &CGoods,
    factory: &CGoodsFactory,
    coefficients: GlobePlayerPropertyCoefficients,
    occupation: usize,
    include_fairy_properties: bool,
    active_level: Option<u8>,
) {
    fn add_u32(target: &mut u32, delta: i32) {
        *target = (i64::from(*target) + i64::from(delta)).clamp(0, i64::from(i32::MAX)) as u32;
    }
    fn add_u16(target: &mut u16, delta: i32) {
        let value = i32::from(*target).wrapping_add(delta);
        *target = if value < 0 { 0 } else { value as u16 };
    }
    fn add_mount_u32(target: &mut u32, delta: i32) {
        let value = target.wrapping_add(delta as u32);
        *target = if delta < 0 && (value as i32) < 0 {
            0
        } else {
            value.min(i32::MAX as u32)
        };
    }
    fn add_mount_u16(target: &mut u16, delta: i32) {
        let value = i32::from(*target).wrapping_add(delta);
        *target = if delta < 0 && value < 0 { 0 } else { value as u16 };
    }
    // `AddPreItemToPlayer`, `MountFuMoProperty` и `ActiveEquip` перед FISTP
    // выставляют x87 RC=truncate; производные поля усекают полную сумму.
    fn scaled_delta(value: i32, coefficient: f32) -> i32 {
        ((value as f32) * coefficient).trunc() as i32
    }
    fn add_derived_u32(target: &mut u32, delta: i32, coefficient: f32) {
        let value = (*target as f32 + delta as f32 * coefficient).trunc() as i64;
        *target = value.clamp(0, i64::from(i32::MAX)) as u32;
    }
    fn add_derived_i32(target: &mut i32, delta: i32, coefficient: f32) {
        let value = (*target as f32 + delta as f32 * coefficient).trunc() as i32;
        *target = if delta < 0 && value < 0 { 0 } else { value };
    }
    fn add_derived_u16(target: &mut u16, delta: i32, coefficient: f32) {
        let value = (*target as f32 + delta as f32 * coefficient).trunc() as i32;
        *target = if value < 0 { 0 } else { value as u16 };
    }
    fn active_u32(current: u32, addition: i32, scale: f64) -> u32 {
        let value = (f64::from(current) + f64::from(addition) * scale).trunc() as i64;
        value.clamp(0, i64::from(i32::MAX)) as u32
    }
    fn active_signed(current: i32, addition: i32, scale: f64) -> i32 {
        let value = (f64::from(current) + f64::from(addition) * scale).trunc() as i64 as u32;
        if (value as i32) < 0 { 0 } else { value as i32 }
    }
    fn active_u16(current: u16, addition: i32, scale: f64) -> u16 {
        let value = (f64::from(current) + f64::from(addition) * scale).trunc() as i64;
        if value < 0 { 0 } else { value as u16 }
    }

    fn apply_active_equip(
        properties: &mut PlayerCombatProperties,
        goods: &CGoods,
        factory: &CGoodsFactory,
        selector: i32,
        percentage: i32,
    ) {
        let scale = f64::from(percentage) * 0.01_f64;
        match selector {
            GAP_MINIMUM_ATTACK_CORRECTION
            | GAP_MAXIMUM_ATTACK_CORRECTION
            | GAP_ELEMENT_ATTACK_CORRECTION => {
                properties.minimum_attack = active_u32(
                    properties.minimum_attack,
                    goods.addon_property_value(factory, GAP_MINIMUM_ATTACK_CORRECTION, 1),
                    scale,
                );
                properties.maximum_attack = active_u32(
                    properties.maximum_attack,
                    goods.addon_property_value(factory, GAP_MAXIMUM_ATTACK_CORRECTION, 1),
                    scale,
                );
                properties.element_modify = active_signed(
                    properties.element_modify,
                    goods.addon_property_value(factory, GAP_ELEMENT_ATTACK_CORRECTION, 1),
                    scale,
                );
            }
            GAP_ARMOR_CORRECTION | GAP_ELEMENT_RESISTANCE_CORRECTION => {
                properties.defense = active_u32(
                    properties.defense,
                    goods.addon_property_value(factory, GAP_ARMOR_CORRECTION, 1),
                    scale,
                );
                properties.element_resistance = active_u32(
                    properties.element_resistance,
                    goods.addon_property_value(factory, GAP_ELEMENT_RESISTANCE_CORRECTION, 1),
                    scale,
                );
            }
            GAP_HP_UPPER_LIMIT_CORRECTION => {
                properties.maximum_hp = active_u32(
                    properties.maximum_hp,
                    goods.addon_property_value(factory, GAP_HP_UPPER_LIMIT_CORRECTION, 1),
                    scale,
                );
            }
            GAP_ATTACK_AVOID | GAP_ELEMENT_AVOID => {
                properties.attack_avoid = active_u16(
                    properties.attack_avoid,
                    goods.addon_property_value(factory, GAP_ATTACK_AVOID, 1),
                    scale,
                );
                properties.element_avoid = active_u16(
                    properties.element_avoid,
                    goods.addon_property_value(factory, GAP_ELEMENT_AVOID, 1),
                    scale,
                );
            }
            _ => {}
        }
    }

    let enabled = goods.enabled_addon_properties(factory);
    // Native equipment/ride owners вызывают positive, затем negative pass:
    // первый pass принимает неотрицательные addon-ы, второй — отрицательные.
    for positive_pass in [true, false] {
        for &stored_type in &enabled {
            if stored_type == GAP_EQUIP_ACTIVE {
                if positive_pass
                    && goods.has_addon_property_values(factory, GAP_ANIMA_BIND)
                    && goods.addon_property_value(factory, GAP_ANIMA_BIND, 1) != 0
                    && active_level.is_some_and(|level| {
                        goods.addon_property_value(factory, GAP_ROLE_MINIMUM_LEVEL_LIMIT, 1)
                            <= i32::from(level)
                    })
                {
                    apply_active_equip(
                        properties,
                        goods,
                        factory,
                        goods.addon_property_value(factory, GAP_EQUIP_ACTIVE, 1),
                        goods.addon_property_value(factory, GAP_EQUIP_ACTIVE, 2),
                    );
                }
                continue;
            }
            let fumo = stored_type == GAP_FUMO_PROPERTY;
            let (property_type, delta) = if fumo {
                (
                    goods.addon_property_value(factory, stored_type, 1),
                    goods.addon_property_value(factory, stored_type, 2),
                )
            } else {
                (
                    stored_type,
                    goods.addon_property_value(factory, stored_type, 1),
                )
            };
            // GAP_FUMO_PROPERTY native-ветка существует только в первом
            // `MountEquipRide(true)` pass и уже внутри принимает signed delta.
            if (fumo && !positive_pass) || (!fumo && (delta >= 0) != positive_pass) {
                continue;
            }
            // MountEquip0x00442610 и MountEquipRide0x0043C5E0 складывают
            // low32 до signed negative gate. Отдельный MountFuMoProperty
            // здесь сохраняет прежний адаптер, не наследуя этот контракт.
            let add_direct_u32 = if fumo { add_u32 } else { add_mount_u32 };
            let add_direct_u16 = if fumo { add_u16 } else { add_mount_u16 };
            match property_type {
                GAP_MINIMUM_ATTACK_CORRECTION => add_direct_u32(&mut properties.minimum_attack, delta),
                GAP_MAXIMUM_ATTACK_CORRECTION => add_direct_u32(&mut properties.maximum_attack, delta),
                GAP_ELEMENT_ATTACK_CORRECTION => {
                    let value = properties.element_modify.wrapping_add(delta);
                    properties.element_modify = if delta < 0 && value < 0 { 0 } else { value };
                }
                GAP_ARMOR_CORRECTION => add_direct_u32(&mut properties.defense, delta),
                GAP_ATTACK_SPEED_CORRECTION => {
                    if fumo {
                        add_u16(&mut properties.attack_speed, delta);
                    } else {
                        properties.attack_speed = properties.attack_speed.wrapping_add(delta as u16);
                    }
                }
                GAP_HIT_RATE_CORRECTION => add_direct_u16(&mut properties.hit, delta),
                GAP_FATAL_BLOW_RATE_CORRECTION => add_direct_u16(&mut properties.cch, delta),
                GAP_DODGE_CORRECTION => add_direct_u16(&mut properties.dodge, delta),
                GAP_ELEMENT_RESISTANCE_CORRECTION => {
                    add_direct_u32(&mut properties.element_resistance, delta)
                }
                GAP_HP_RESTORE_SPEED_CORRECTION => add_direct_u16(&mut properties.hp_recovery, delta),
                GAP_MP_RESTORE_SPEED_CORRECTION => add_direct_u16(&mut properties.mp_recovery, delta),
                GAP_STRENGTH_CORRECTION => {
                    add_direct_u32(&mut properties.strength, delta);
                    add_derived_u32(
                        &mut properties.maximum_attack,
                        delta,
                        coefficients.str_to_max_attack[occupation],
                    );
                    add_derived_u16(
                        &mut properties.burden,
                        delta,
                        coefficients.str_to_burden[occupation],
                    );
                }
                GAP_AGILITY_CORRECTION => {
                    add_direct_u32(&mut properties.dexterity, delta);
                    add_derived_u32(
                        &mut properties.minimum_attack,
                        delta,
                        coefficients.dex_to_min_attack[occupation],
                    );
                    add_derived_u16(
                        &mut properties.reank,
                        delta,
                        coefficients.dex_to_stiff[occupation],
                    );
                }
                GAP_CONSTITUTION_CORRECTION => {
                    add_direct_u32(&mut properties.constitution, delta);
                    add_derived_u32(
                        &mut properties.maximum_hp,
                        delta,
                        coefficients.con_to_max_hp[occupation],
                    );
                    add_derived_u32(
                        &mut properties.defense,
                        delta,
                        coefficients.con_to_defense[occupation],
                    );
                }
                GAP_WAKAN_CORRECTION => {
                    add_direct_u32(&mut properties.intelligence, delta);
                    add_derived_i32(
                        &mut properties.element_modify,
                        delta,
                        coefficients.int_to_element[occupation],
                    );
                    add_derived_u32(
                        &mut properties.maximum_mp,
                        delta,
                        coefficients.int_to_max_mp[occupation],
                    );
                    add_derived_u32(
                        &mut properties.element_resistance,
                        delta,
                        coefficients.int_to_resistant[occupation],
                    );
                }
                GAP_HP_UPPER_LIMIT_CORRECTION => add_direct_u32(&mut properties.maximum_hp, delta),
                GAP_MP_UPPER_LIMIT_CORRECTION => add_direct_u32(&mut properties.maximum_mp, delta),
                GAP_STIFFEN_PROBABILITY_CORRECTION => add_direct_u16(&mut properties.reank, delta),
                GAP_BURDEN_UPPER_LIMIT_CORRECTION => add_direct_u16(&mut properties.burden, delta),
                GAP_ATTACK_AVOID => add_direct_u16(&mut properties.attack_avoid, delta),
                GAP_ELEMENT_AVOID => add_direct_u16(&mut properties.element_avoid, delta),
                GAP_FULL_MISS => add_direct_u16(&mut properties.full_miss, delta),
                GAP_BLAST_ATTACK => add_direct_u16(&mut properties.blast_attack, delta),
                GAP_BLAST_ELEMENT_ATTACK => {
                    // Legacy case 96 берёт base из wBlastAttack, не из target.
                    let mut value = properties.blast_attack;
                    add_direct_u16(&mut value, delta);
                    properties.blast_element_attack = value;
                }
                GAP_FAIRY_STRENGTH if include_fairy_properties => {
                    let player_delta = scaled_delta(delta, coefficients.fairy_strength_to_player);
                    add_u32(&mut properties.strength, player_delta);
                    add_derived_u32(
                        &mut properties.maximum_attack,
                        player_delta,
                        coefficients.str_to_max_attack[occupation],
                    );
                    add_derived_u16(
                        &mut properties.burden,
                        player_delta,
                        coefficients.str_to_burden[occupation],
                    );
                }
                GAP_FAIRY_AGILITY if include_fairy_properties => {
                    let player_delta = scaled_delta(delta, coefficients.fairy_agility_to_player);
                    add_u32(&mut properties.dexterity, player_delta);
                    add_derived_u32(
                        &mut properties.minimum_attack,
                        player_delta,
                        coefficients.dex_to_min_attack[occupation],
                    );
                    add_derived_u16(
                        &mut properties.reank,
                        player_delta,
                        coefficients.dex_to_stiff[occupation],
                    );
                }
                GAP_FAIRY_WAKAN if include_fairy_properties => {
                    let player_delta = scaled_delta(delta, coefficients.fairy_wakan_to_player);
                    add_u32(&mut properties.intelligence, player_delta);
                    add_derived_i32(
                        &mut properties.element_modify,
                        player_delta,
                        coefficients.int_to_element[occupation],
                    );
                    add_derived_u32(
                        &mut properties.maximum_mp,
                        player_delta,
                        coefficients.int_to_max_mp[occupation],
                    );
                    add_derived_u32(
                        &mut properties.element_resistance,
                        player_delta,
                        coefficients.int_to_resistant[occupation],
                    );
                }
                GAP_FAIRY_HP if include_fairy_properties => add_u32(
                    &mut properties.maximum_hp,
                    scaled_delta(delta, coefficients.fairy_hp_to_player),
                ),
                _ => {}
            }
        }
    }
}

impl CPlayer {
    /// Собирает только достигнутый send-family state уже созданного игрока;
    /// identity другого object type отвергается до регистрации.
    pub(crate) fn from_send_state(
        move_shape: CMoveShape,
        figure: ShapeFigure,
        team_id: i32,
        country: u8,
        server_region_id: Option<i32>,
    ) -> Option<Self> {
        if move_shape.shape().identity().object_type != PLAYER_TYPE {
            return None;
        }
        let owner_id = move_shape.shape().identity().id;
        let realm_appellation_bonus = move_shape
            .skills()
            .filter(|skill| {
                super::skills::realmappellation::is_bonus_skill(skill.id())
                    && (1..=4).contains(&skill.level())
            })
            .min_by_key(|skill| skill.id())
            .map(|skill| (skill.id(), skill.level()));
        let mut packet = CVolumeLimitGoodsContainer::new();
        let _empty_release = packet.set_container_dimensions(8, 12);
        let mut enhancement = CAmountLimitGoodsShadowContainer::new();
        enhancement.set_goods_amount_limit(1);
        enhancement.base_mut().set_container_extend_id(10);
        let mut ci_qing = CVolumeLimitGoodsContainer::new();
        let _empty_release = ci_qing.set_container_volume(8);
        let mut ci_qing_compose = CVolumeLimitGoodsContainer::new();
        let _empty_release = ci_qing_compose.set_container_volume(3);
        let mut fairy_container = CFairyContainer::new();
        let _empty_release = fairy_container.base_mut().set_container_volume(14);
        let mut auction_goods = CVolumeLimitGoodsContainer::new();
        let _empty_release = auction_goods.set_container_volume(0x12);
        let mut auction_listing = CVolumeLimitGoodsContainer::new();
        let _empty_release = auction_listing.set_container_volume(2);
        let mut depot = CDepot::new();
        let _empty_release = depot.base_mut().set_container_dimensions(8, 12);
        let mut player = Self {
            move_shape,
            player_ai: CPlayerAI::default(),
            figure,
            faction_id: 0,
            faction_logo_id: 0,
            faction_level: 0,
            faction_experience: 0,
            faction_force: 0,
            faction_contribute: 0,
            faction_master_id: 0,
            faction_name: Vec::new(),
            faction_title: Vec::new(),
            enemy_factions: BTreeSet::new(),
            city_war_enemy_factions: BTreeSet::new(),
            faction_owned_regions: Vec::new(),
            union_id: 0,
            union_master_id: 0,
            team_id,
            team_captain: false,
            country,
            server_region_id,
            in_changing_server: false,
            in_changing_region: true,
            last_enter_region_tick_ms: 0,
            entered_region: false,
            state_before_server_region_change: 0,
            current_progress: PlayerProgress::None,
            personal_shop_session_id: 0,
            personal_shop_plug_id: 0,
            war_soul_state: 0,
            war_soul_point: WarSoulPoint::default(),
            war_soul_visual_x_bits: 0.0f32.to_bits(),
            war_soul_visual_y_bits: 0.0f32.to_bits(),
            current_ticket: 0,
            goods_ai_tree: BTreeMap::new(),
            goods_ai_delete_queue: VecDeque::new(),
            flash_previous: [0; 17],
            flash_current: [0; 17],
            flash_changed: false,
            battle_fairy_summoned: false,
            recreate_carriage: false,
            active_carriage_id: 0,
            create_faction_operator: false,
            apply_join_faction_operator: false,
            faction_declare_operator: false,
            active_pet_count: 0,
            attempt_appellation_id: 0,
            realm_appellation_skill_id: realm_appellation_bonus
                .map_or(UNKNOWN_SKILL_ID, |identity| identity.0),
            realm_appellation_skill_level: realm_appellation_bonus.map_or(0, |identity| identity.1),
            heart_request_sent: 0,
            heart_received: false,
            friends: Vec::new(),
            quest_states: BTreeMap::new(),
            lei_ting_things: VecDeque::new(),
            uncreated_pets: Vec::new(),
            uncreated_carriage: PlayerUncreatedCarriage::default(),
            login: false,
            session_id: Vec::new(),
            title: Vec::new(),
            base_property_wire: [0; PLAYER_BASE_PROPERTY_WIRE_SIZE],
            jjc_data: [0; 0x10],
            jjc_pk_state: false,
            fight_state_count: 0,
            auto_protected: false,
            lost_time_stamp_ms: 0,
            continuous_kill_amount: 0,
            continuous_kill_timestamp_ms: 0,
            base_properties: PlayerBaseProperties::default(),
            combat_properties: PlayerCombatProperties::default(),
            combat_property_wire: [0; PLAYER_COMBAT_PROPERTY_WIRE_SIZE],
            expendable_effects: BTreeMap::new(),
            last_skill_item_use_ms: BTreeMap::new(),
            ci_qing_open: false,
            ci_qing_list: BTreeSet::new(),
            ci_qing_add_values: BTreeMap::new(),
            ci_qing_tao_zhuang_add_values: BTreeMap::new(),
            tao_zhuang_id: 0,
            equipment_changed: false,
            tao_zhuang_setup_pending: true,
            tao_zhuang_items: BTreeMap::new(),
            tao_zhuang_original_names: BTreeSet::new(),
            tao_zhuang_properties: BTreeMap::new(),
            ci_qing_tao_zhuang_properties: BTreeMap::new(),
            tao_zhuang_skills: BTreeMap::new(),
            contend_state: false,
            emotion_index: 0,
            emotion_timestamp_ms: 0,
            city_war_died_state: false,
            city_war_died_state_time_ms: 0,
            died_state_start_time_ms: 0,
            criminal_state_timestamp_ms: 0,
            murderer_time_stamp_ms: 0,
            ping_time: 0,
            last_ping_time_ms: 0,
            contribution: 0,
            silence_minutes: 0,
            silence_timestamp_minutes: 0,
            normal_talk_timestamp_ms: 0,
            area_talk_timestamp_ms: 0,
            world_talk_timestamp_ms: 0,
            country_talk_timestamp_ms: 0,
            private_talk_timestamp_ms: 0,
            team_talk_timestamp_ms: 0,
            union_talk_timestamp_ms: 0,
            money: 0,
            client_ip: 0,
            account: Vec::new(),
            depot_password: Vec::new(),
            last_container_script: Vec::new(),
            variable_list: CVariableList::default(),
            bank: CBank::new(),
            depot,
            hand: CAmountLimitGoodsContainer::new(),
            enhancement,
            last_operated_container: 0,
            last_operated_goods_position: 0,
            packet,
            wallet: CWallet::new(),
            yuan_bao: CYuanBao::new(),
            ji_fen: CJiFen::new(),
            equipment: CEquipmentContainer::new(),
            auction_listing,
            auction_goods,
            auction_wallet: CWallet::new(),
            auction_open: false,
            auction_search_name: Vec::new(),
            auction_search_lower_level: 0,
            auction_search_upper_level: 0,
            auction_search_use_self: 0,
            auction_search_money_type: 0,
            auction_search_weapon_type: 0,
            auction_current_page: 0,
            last_auction_limit_tick_ms: 0,
            last_auction_option_tick_ms: 0,
            current_auction_node: None,
            auction_listing_fee: 0,
            current_auction_buy_node: None,
            ci_qing,
            ci_qing_compose,
            fairy_container,
            battle_fairy_container: CBattleFairyContainer::new(),
        };
        player.refresh_reached_container_owners(owner_id);
        Some(player)
    }

    /// Exact World→Game player handoff. Decoder восстанавливает единый
    /// `CPlayer::DecordFromByteArray(..., true)` блок, а не отдельные игровые
    /// фрагменты. Native GoodsAI tail намеренно отложен до успешной регистрации
    /// player-а в map/region и исполняется `CGame::complete_world_player_login`
    /// для достигнутых equipment и packet owners.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn decode_game_save(
        source: &[u8],
        cursor: &mut usize,
        goods_factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
        variable_definitions: Option<&[u8]>,
        now_ms: u32,
        one_pk_count_time_ms: u32,
        state_now: &mut dyn FnMut() -> u32,
        pack_add_enabled: bool,
        ordinary_threshold: &mut dyn FnMut(u32, u32) -> u32,
        battle_threshold: &mut dyn FnMut(u32, u32) -> u32,
    ) -> Result<Self, PlayerGameSaveCodecError> {
        let start = *cursor;
        let mut move_shape = CMoveShape::default();
        move_shape
            .shape_mut()
            .decode_from_byte_array(source, cursor, true)?;
        let object_type = move_shape.shape().identity().object_type;
        if object_type != PLAYER_TYPE {
            return Err(PlayerGameSaveCodecError::WrongObjectType { object_type });
        }
        let server_region_id = move_shape.shape().get_region_id();

        let mut player = Self::from_send_state(
            move_shape,
            ShapeFigure::default(),
            0,
            0,
            Some(server_region_id),
        )
        .expect("player object type проверен до создания CPlayer");
        player.base_property_wire =
            read_player_game_save_array(source, cursor, "m_BaseProperty[0x194]")?;
        player.apply_base_property_wire();
        player.battle_fairy_summoned = player.war_soul_state != 0;
        player.account = read_player_game_save_string(source, cursor, "strAccount", 0x100)?;
        player.title = read_player_game_save_string(source, cursor, "strTitle", 0x100)?;
        player.combat_property_wire =
            read_player_game_save_array(source, cursor, "m_Property[0x9c]")?;
        player.apply_combat_property_wire();
        player.team_id = read_player_game_save_i32(source, cursor, "m_lTeamID")?;

        player.ci_qing_list.clear();
        let ci_qing_count = read_player_game_save_count(source, cursor, "m_setCiQingList")?;
        for _ in 0..ci_qing_count {
            player.ci_qing_list.insert(read_player_game_save_u32(
                source,
                cursor,
                "m_setCiQingList entry",
            )?);
        }

        player.move_shape.clear_persisted_runtime_state();
        player.auto_protected = false;
        let skill_count = read_player_game_save_i32(source, cursor, "skill count")?;
        for _ in 0..skill_count.max(0) {
            let packed = read_player_game_save_u32(source, cursor, "tagSkillID")?;
            let skill_id = packed & 0xffff;
            let level = (packed >> 16) as i32;
            let loaded = player.move_shape.add_skill(skill_id, level, skill_factory);
            if loaded
                && super::skills::realmappellation::is_bonus_skill(skill_id)
                && (1..=4).contains(&level)
            {
                player.realm_appellation_skill_id = skill_id;
                player.realm_appellation_skill_level = level;
            }
        }
        let ex_state_length = read_player_game_save_count(source, cursor, "m_vExStates length")?;
        player.move_shape.replace_ex_states(
            read_player_game_save_slice(source, cursor, "m_vExStates", ex_state_length)?.to_vec(),
            skill_factory,
            state_now,
        );

        player.friends.clear();
        let friend_count = read_player_game_save_i32(source, cursor, "m_listFriend")?;
        for _ in 0..friend_count.max(0) {
            player.friends.push(PlayerFriend {
                name: read_player_game_save_string(source, cursor, "tagFriend.strName", 0x94)?,
                online: read_player_game_save_u8(source, cursor, "tagFriend.bOnline")? != 0,
            });
        }
        player.decode_lei_ting(source, cursor)?;

        let _cleared_hand = player.hand.clear_goods();
        player.hand.set_goods_amount_limit(1);
        player.hand.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;

        let base_properties = player.base_properties;
        let combat_properties = player.combat_properties;
        let equipment = player.equipment.unserialize_with(
            source,
            cursor,
            goods_factory,
            true,
            |source, cursor| {
                let mut goods = CGoods::default();
                goods.unserialize(
                    source,
                    cursor,
                    true,
                    goods_factory,
                    &mut *ordinary_threshold,
                    &mut *battle_threshold,
                )?;
                Ok(Some(goods))
            },
            |goods| EquipmentAddRuntimeFacts {
                owner_player: Some(EquipmentOwnerPlayerFacts {
                    can_mount_result: Self::can_mount_equip_from_properties(
                        base_properties,
                        combat_properties,
                        goods,
                        goods_factory,
                    ),
                }),
                pack_add_enabled: false,
                now: u64::from(now_ms),
            },
            &mut |_| {},
            &mut |_, _, _| {},
        );
        let equipment =
            equipment.map_err(|failure| PlayerGameSaveCodecError::Equipment(failure.error))?;
        if let Some(position) = equipment.entries.iter().find_map(|entry| match entry {
            EquipmentUnserializedEntry::Rejected { position, .. }
            | EquipmentUnserializedEntry::DecoderReturnedNull { position } => Some(*position),
            EquipmentUnserializedEntry::Added { .. } => None,
        }) {
            return Err(PlayerGameSaveCodecError::EquipmentRejected { position });
        }

        let _released = player.packet.set_container_dimensions(8, 12);
        player.packet.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        player
            .packet
            .apply_player_expansion_limit(player.equipment.expanded_package_num());

        let _released = player.auction_goods.set_container_volume(0x12);
        player.auction_goods.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        player.auction_goods.set_all_inactive();
        if pack_add_enabled {
            let inactive = player
                .auction_goods
                .size()
                .wrapping_sub(player.base_properties.auction_space);
            player.auction_goods.apply_player_expansion_limit(inactive);
        }
        let _released = player.auction_listing.set_container_volume(2);
        player.auction_listing.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        player.wallet.unserialize(
            source,
            cursor,
            "m_cWallet marker",
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        player.auction_wallet.unserialize(
            source,
            cursor,
            "m_cAuctionWallet marker",
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        player.yuan_bao.unserialize(
            source,
            cursor,
            "m_cYuanBao marker",
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        player.ji_fen.unserialize(
            source,
            cursor,
            "m_cJiFen marker",
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        player.money = player.wallet.currency_amount();

        player.depot_password =
            read_player_game_save_string(source, cursor, "m_strDepotPassword", 0x6c)?;
        player.bank.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        player.depot.unserialize(
            source,
            cursor,
            goods_factory,
            pack_add_enabled,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        let _released = player.fairy_container.base_mut().set_container_volume(0x0e);
        player.fairy_container.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        let _released = player
            .battle_fairy_container
            .base_mut()
            .set_container_volume(0x11);
        player.battle_fairy_container.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        let _released = player.ci_qing.set_container_volume(8);
        player.ci_qing.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        let _released = player.ci_qing_compose.set_container_volume(3);
        player.ci_qing_compose.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;

        player
            .variable_list
            .decode_world_snapshot(variable_definitions, source, cursor)?;
        player.silence_minutes = read_player_game_save_i32(source, cursor, "m_lSilenceTime")?;
        let murderer_state = read_player_game_save_u8(source, cursor, "murderer state")? != 0;
        let murderer_remain = read_player_game_save_u32(source, cursor, "murderer remain time")?;
        player.restore_murderer_timestamp(
            murderer_state,
            murderer_remain,
            now_ms,
            one_pk_count_time_ms,
        );
        player.fight_state_count = read_player_game_save_i32(source, cursor, "m_lFightStateCount")?;

        player.uncreated_pets.clear();
        let pet_count = read_player_game_save_count(source, cursor, "m_vUncreatedPets")?;
        for _ in 0..pet_count {
            player.uncreated_pets.push(PlayerUncreatedPet {
                original_name: read_player_game_save_string(
                    source,
                    cursor,
                    "tagPetInformation.strOriginalName",
                    0x94,
                )?,
                health: read_player_game_save_u32(source, cursor, "tagPetInformation.dwHp")?,
                level: read_player_game_save_u32(source, cursor, "tagPetInformation.dwLevel")?,
                experience: read_player_game_save_u32(
                    source,
                    cursor,
                    "tagPetInformation.dwExperience",
                )?,
            });
        }
        player.uncreated_carriage = PlayerUncreatedCarriage {
            original_name: read_player_game_save_string(
                source,
                cursor,
                "tagCarriageInfo.strOriginalName",
                0x94,
            )?,
            script: read_player_game_save_string(
                source,
                cursor,
                "tagCarriageInfo.strCarriageScript",
                0x94,
            )?,
            health: read_player_game_save_u32(source, cursor, "tagCarriageInfo.dwHp")?,
        };
        player.recreate_carriage =
            read_player_game_save_u8(source, cursor, "m_bReCreateCarriage")? != 0;
        player.login = read_player_game_save_u8(source, cursor, "m_bLogin")? != 0;
        player.city_war_died_state_time_ms =
            read_player_game_save_i32(source, cursor, "m_lCityWarDiedStateTime")?;
        player.died_state_start_time_ms =
            u32::from(player.city_war_died_state_time_ms > 0).wrapping_mul(now_ms);
        player.city_war_died_state =
            player.city_war_died_state_time_ms > 0 && player.base_properties.occupation != 6;

        player.quest_states.clear();
        let quest_count = read_player_game_save_i32(source, cursor, "m_PlayerQuests")?;
        for _ in 0..quest_count.max(0) {
            let quest_id = read_player_game_save_u16(source, cursor, "tagPlayerQuest.wQuestID")?;
            let state = read_player_game_save_u8(source, cursor, "tagPlayerQuest.byComplete")?;
            player.quest_states.insert(quest_id, state);
        }
        player.country = read_player_game_save_u8(source, cursor, "m_btCountry")?;
        player.contribution = read_player_game_save_i32(source, cursor, "m_lContribute")?;
        player.jjc_data = read_player_game_save_array(source, cursor, "m_jjcdata[0x10]")?;
        player.jjc_pk_state = read_player_game_save_u8(source, cursor, "bJJcPkState")? != 0;
        player.decode_organizing_snapshot(source, cursor)?;
        player.session_id = read_player_game_save_string(source, cursor, "m_strSessionID", 0x40)?;
        player.refresh_reached_container_owners(player.player_id());

        let consumed_bytes = cursor.saturating_sub(start);
        tracing::trace!(
            player_id = player.player_id(),
            consumed_bytes,
            "сохранение игрока декодировано"
        );
        Ok(player)
    }

    /// `AddGameSaveToByteArray`: тот же persisted layout без organization
    /// snapshot (World обновляет его самостоятельно перед следующим handoff).
    pub(crate) fn encode_game_save(
        &mut self,
        destination: &mut Vec<u8>,
        goods_factory: &CGoodsFactory,
        now_ms: u32,
        timed_state_now_milliseconds: impl FnMut() -> u32,
        one_pk_count_time_ms: u32,
        pets: &[PlayerUncreatedPet],
        carriage: &PlayerUncreatedCarriage,
        recreate_carriage: bool,
    ) -> Result<bool, PlayerGameSaveCodecError> {
        if !self.shape().add_to_byte_array(destination, true) {
            return Err(PlayerGameSaveCodecError::CodecReturnedFalse { field: "CShape" });
        }
        destination.extend_from_slice(&self.synchronized_base_property_wire());
        append_player_game_save_string(destination, "strAccount", &self.account, 0x100)?;
        append_player_game_save_string(destination, "strTitle", &self.title, 0x100)?;
        destination.extend_from_slice(&self.combat_property_wire);
        LegacyWriter::new(destination).write_i32(self.team_id);

        append_player_game_save_count(destination, "m_setCiQingList", self.ci_qing_list.len())?;
        for base_index in &self.ci_qing_list {
            LegacyWriter::new(destination).write_u32(*base_index);
        }
        let skills: Vec<_> = self.serializable_skills().collect();
        append_player_game_save_count(destination, "skill count", skills.len())?;
        for skill in skills {
            let packed = (skill.id() & 0xffff) | ((skill.level() as u32 & 0xffff) << 16);
            LegacyWriter::new(destination).write_u32(packed);
        }
        let ex_states = self
            .move_shape
            .serialize_ex_states_for_save(now_ms, timed_state_now_milliseconds);
        append_player_game_save_count(destination, "m_vExStates length", ex_states.len())?;
        destination.extend_from_slice(&ex_states);
        append_player_game_save_count(destination, "m_listFriend", self.friends.len())?;
        for friend in &self.friends {
            append_player_game_save_string(destination, "tagFriend.strName", &friend.name, 0x94)?;
            destination.push(u8::from(friend.online));
        }
        destination.extend_from_slice(&self.encode_lei_ting());

        if !self.hand.serialize(destination, goods_factory) {
            return Err(PlayerGameSaveCodecError::CodecReturnedFalse { field: "m_cHand" });
        }
        let mut equipment_serialized = true;
        self.equipment.serialize_with(
            destination,
            goods_factory,
            true,
            |goods, include_child, destination| {
                equipment_serialized &= goods.serialize(destination, include_child);
            },
        );
        if !equipment_serialized {
            return Err(PlayerGameSaveCodecError::CodecReturnedFalse {
                field: "m_cEquipment",
            });
        }
        macro_rules! serialize_container {
            ($field:literal, $serialized:expr) => {
                if !$serialized {
                    return Err(PlayerGameSaveCodecError::CodecReturnedFalse { field: $field });
                }
            };
        }
        serialize_container!(
            "m_cPacket",
            self.packet.serialize(destination, goods_factory)
        );
        serialize_container!(
            "m_cAuctionGoodsContainer",
            self.auction_goods.serialize(destination, goods_factory)
        );
        serialize_container!(
            "m_cAuctionContainer",
            self.auction_listing.serialize(destination, goods_factory)
        );
        serialize_container!("m_cWallet", self.wallet.serialize(destination));
        serialize_container!(
            "m_cAuctionWallet",
            self.auction_wallet.serialize(destination)
        );
        serialize_container!("m_cYuanBao", self.yuan_bao.serialize(destination));
        serialize_container!("m_cJiFen", self.ji_fen.serialize(destination));
        append_player_game_save_string(
            destination,
            "m_strDepotPassword",
            &self.depot_password,
            0x6c,
        )?;
        serialize_container!("m_cBank", self.bank.serialize(destination));
        serialize_container!("m_cDepot", self.depot.serialize(destination, goods_factory));
        serialize_container!(
            "m_cFairy",
            self.fairy_container.serialize(destination, goods_factory)
        );
        serialize_container!(
            "m_cBF",
            self.battle_fairy_container
                .serialize(destination, goods_factory)
        );
        serialize_container!(
            "m_cCiQing",
            self.ci_qing.serialize(destination, goods_factory)
        );
        serialize_container!(
            "m_cComposeCiQing",
            self.ci_qing_compose.serialize(destination, goods_factory)
        );
        if !self.variable_list.encode_world_snapshot(destination) {
            return Err(PlayerGameSaveCodecError::CodecReturnedFalse {
                field: "m_pVariableList",
            });
        }
        LegacyWriter::new(destination).write_i32(self.silence_minutes);
        let murderer_state = self.base_properties.pk_count != 0 && self.murderer_time_stamp_ms != 0;
        LegacyWriter::new(destination).write_u8(u8::from(murderer_state));
        let murderer_remain = if self.murderer_time_stamp_ms == 0 {
            0
        } else {
            self.murderer_time_stamp_ms
                .wrapping_add(one_pk_count_time_ms)
                .wrapping_sub(now_ms)
                .min(one_pk_count_time_ms)
        };
        LegacyWriter::new(destination).write_u32(murderer_remain);
        LegacyWriter::new(destination).write_i32(self.fight_state_count);
        append_player_game_save_count(destination, "m_vUncreatedPets", pets.len())?;
        for pet in pets {
            append_player_game_save_string(
                destination,
                "tagPetInformation.strOriginalName",
                &pet.original_name,
                0x94,
            )?;
            let mut writer = LegacyWriter::new(destination);
            writer.write_u32(pet.health);
            writer.write_u32(pet.level);
            writer.write_u32(pet.experience);
        }
        append_player_game_save_string(
            destination,
            "tagCarriageInfo.strOriginalName",
            &carriage.original_name,
            0x94,
        )?;
        append_player_game_save_string(
            destination,
            "tagCarriageInfo.strCarriageScript",
            &carriage.script,
            0x94,
        )?;
        let mut writer = LegacyWriter::new(destination);
        writer.write_u32(carriage.health);
        writer.write_u8(u8::from(recreate_carriage));
        writer.write_u8(u8::from(self.login));
        writer.write_i32(self.city_war_died_state_time_ms);
        append_player_game_save_count(destination, "m_PlayerQuests", self.quest_states.len())?;
        for (quest_id, state) in &self.quest_states {
            let mut writer = LegacyWriter::new(destination);
            writer.write_u16(*quest_id);
            writer.write_u8(*state);
        }
        LegacyWriter::new(destination).write_u8(self.country);
        LegacyWriter::new(destination).write_i32(self.contribution);
        destination.extend_from_slice(&self.jjc_data);
        destination.push(u8::from(self.jjc_pk_state));
        append_player_game_save_string(destination, "m_strSessionID", &self.session_id, 0x40)?;
        Ok(true)
    }

    fn synchronized_base_property_wire(&self) -> [u8; PLAYER_BASE_PROPERTY_WIRE_SIZE] {
        let mut wire = self.base_property_wire;
        wire[BASE_LEVEL_OFFSET] = self.base_properties.level;
        write_player_wire_u32(
            &mut wire,
            BASE_EXPERIENCE_OFFSET,
            self.base_properties.experience,
        );
        wire[BASE_HEAD_PICTURE_OFFSET] = self.base_properties.head_picture as u8;
        wire[BASE_FACE_PICTURE_OFFSET] = self.base_properties.face_picture as u8;
        wire[BASE_OCCUPATION_OFFSET] = self.base_properties.occupation;
        wire[BASE_SEX_OFFSET] = self.base_properties.sex;
        write_player_wire_u16(
            &mut wire,
            BASE_PK_COUNT_OFFSET,
            self.base_properties.pk_count,
        );
        write_player_wire_u32(
            &mut wire,
            BASE_KILL_COUNT_OFFSET,
            self.base_properties.kill_count,
        );
        write_player_wire_u16(
            &mut wire,
            BASE_HIT_TOP_LOG_OFFSET,
            self.base_properties.hit_top_log,
        );
        write_player_wire_u16(
            &mut wire,
            BASE_REMAIN_POINT_OFFSET,
            self.base_properties.remain_point,
        );
        wire[BASE_CHARGED_OFFSET] = u8::from(self.base_properties.charged);
        for (index, hotkey) in self.base_properties.hotkeys.iter().copied().enumerate() {
            write_player_wire_u32(&mut wire, BASE_HOTKEY_OFFSET + index * 4, hotkey);
        }
        for (offset, value) in [
            (BASE_PK_NORMAL_OFFSET, self.base_properties.pk_normal),
            (BASE_PK_TEAM_OFFSET, self.base_properties.pk_team),
            (BASE_PK_UNION_OFFSET, self.base_properties.pk_union),
            (BASE_PK_BADMAN_OFFSET, self.base_properties.pk_badman),
            (BASE_PK_COUNTRY_OFFSET, self.base_properties.pk_country),
            (
                BASE_FAIRY_CONTAINER_ENABLED_OFFSET,
                self.base_properties.fairy_container_enabled,
            ),
            (
                BASE_BATTLE_FAIRY_ENABLED_OFFSET,
                self.base_properties.battle_fairy_enabled,
            ),
            (
                BASE_DISPLAY_HEAD_PIECE_OFFSET,
                self.base_properties.display_head_piece,
            ),
            (
                BASE_BATTLE_FAIRY_SUMMONED_OFFSET,
                self.battle_fairy_summoned,
            ),
            (
                BASE_BATTLE_FAIRY_RECALL_OFFSET,
                self.base_properties.battle_fairy_recall,
            ),
            (
                BASE_BATTLE_FAIRY_DIED_OFFSET,
                self.base_properties.battle_fairy_died,
            ),
            (
                BASE_QUEST_ENABLED_OFFSET,
                self.base_properties.quest_enabled,
            ),
        ] {
            wire[offset] = u8::from(value);
        }
        for (offset, value) in [
            (BASE_HEALTH_OFFSET, self.base_properties.health),
            (BASE_MANA_OFFSET, self.base_properties.mana),
            (BASE_MAXIMUM_HP_OFFSET, self.base_properties.base_maximum_hp),
            (BASE_MAXIMUM_MP_OFFSET, self.base_properties.base_maximum_mp),
            (BASE_STRENGTH_OFFSET, self.base_properties.base_strength),
            (BASE_DEXTERITY_OFFSET, self.base_properties.base_dexterity),
            (
                BASE_CONSTITUTION_OFFSET,
                self.base_properties.base_constitution,
            ),
            (
                BASE_INTELLIGENCE_OFFSET,
                self.base_properties.base_intelligence,
            ),
            (BASE_VIGOUR_OFFSET, self.base_properties.vigour),
            (
                BASE_MAXIMUM_VIGOUR_OFFSET,
                self.base_properties.maximum_vigour,
            ),
            (BASE_ENERGY_OFFSET, self.base_properties.energy),
            (
                BASE_MAXIMUM_ENERGY_OFFSET,
                self.base_properties.maximum_energy,
            ),
            (BASE_CREDIT_OFFSET, self.base_properties.credit),
            (BASE_EXALT_OFFSET, self.base_properties.exalt),
            (
                BASE_QUEST_TIME_BEGIN_OFFSET,
                self.base_properties.quest_time_begin as u32,
            ),
            (
                BASE_QUEST_TIME_LIMIT_OFFSET,
                self.base_properties.quest_time_limit as u32,
            ),
            (BASE_EXPLOIT_OFFSET, self.base_properties.exploit),
            (BASE_BREAK_ARMOUR_OFFSET, self.base_properties.break_armour),
            (BASE_PUNCTURE_OFFSET, self.base_properties.puncture),
            (BASE_BREAK_ELEMENT_OFFSET, self.base_properties.break_element),
            (BASE_BREAK_BOUND_OFFSET, self.base_properties.break_bound),
            (BASE_POWER_OF_GOLD_OFFSET, self.base_properties.power_of_gold),
            (
                BASE_DAYS_HONOR_OFFSET,
                self.base_properties.days_honor_eliminate,
            ),
            (
                BASE_WEEKS_HONOR_OFFSET,
                self.base_properties.weeks_honor_eliminate,
            ),
            (
                BASE_MONTHS_HONOR_OFFSET,
                self.base_properties.months_honor_eliminate,
            ),
            (
                BASE_TOTAL_HONOR_OFFSET,
                self.base_properties.total_honor_eliminate,
            ),
            (
                BASE_RANK_OF_NOBILITY_OFFSET,
                self.base_properties.rank_of_nobility_id,
            ),
            (BASE_APPELLATION_OFFSET, self.base_properties.appellation_id),
            (BASE_MODE_OFFSET, self.base_properties.mode),
            (BASE_FETCH_POWER_OFFSET, self.base_properties.fetch_power),
            (
                BASE_AUCTION_SPACE_OFFSET,
                self.base_properties.auction_space,
            ),
            (BASE_JJC_LEVEL_OFFSET, self.base_properties.jjc_level),
            (BASE_JJC_SCORE_OFFSET, self.base_properties.jjc_score),
            (BASE_FY_ENERGY_OFFSET, self.base_properties.fy_energy),
            (
                BASE_FY_ENABLE_FLAGS_OFFSET,
                self.base_properties.fy_enable_flags.bits(),
            ),
            (BASE_LT_60_STAMP_OFFSET, self.base_properties.lt_60_stamp),
            (BASE_SZL_OFFSET, self.base_properties.szl),
            (
                BASE_GODS_BATTLE_FACTION_OFFSET,
                self.base_properties.gods_battle_faction as u32,
            ),
        ] {
            write_player_wire_u32(&mut wire, offset, value);
        }
        write_player_wire_u16(
            &mut wire,
            BASE_LT_UP_60_COUNT_OFFSET,
            self.base_properties.lt_up_60_count,
        );
        write_player_wire_u16(&mut wire, BASE_RP_OFFSET, self.base_properties.rp);
        write_player_wire_u16(&mut wire, BASE_YP_OFFSET, self.base_properties.yp);
        write_player_wire_u16(
            &mut wire,
            BASE_MAXIMUM_YP_OFFSET,
            self.base_properties.maximum_yp,
        );
        write_player_wire_u16(
            &mut wire,
            BASE_BURDEN_OFFSET,
            self.base_properties.base_burden,
        );
        write_player_wire_u16(
            &mut wire,
            BASE_MAXIMUM_RP_OFFSET,
            self.base_properties.maximum_rp,
        );
        write_player_wire_u16(
            &mut wire,
            BASE_REMAIN_JING_LI_DAN_COUNT_OFFSET,
            self.base_properties.remain_jing_li_dan_count,
        );
        wire
    }

    fn restore_murderer_timestamp(
        &mut self,
        murderer_state: bool,
        murderer_remain: u32,
        now_ms: u32,
        one_pk_count_time_ms: u32,
    ) {
        if self.base_properties.pk_count == 0 {
            self.murderer_time_stamp_ms = 0;
            return;
        }
        if murderer_state || self.murderer_time_stamp_ms == 0 {
            self.murderer_time_stamp_ms = now_ms;
        }
        if murderer_remain != 0 {
            let remain = murderer_remain.min(one_pk_count_time_ms);
            self.murderer_time_stamp_ms =
                now_ms.wrapping_sub(one_pk_count_time_ms.wrapping_sub(remain));
        }
    }

    fn apply_base_property_wire(&mut self) {
        let wire = &self.base_property_wire;
        self.base_properties.level = wire[BASE_LEVEL_OFFSET];
        self.base_properties.experience = read_player_wire_u32(wire, BASE_EXPERIENCE_OFFSET);
        self.base_properties.head_picture = i32::from(wire[BASE_HEAD_PICTURE_OFFSET]);
        self.base_properties.face_picture = i32::from(wire[BASE_FACE_PICTURE_OFFSET]);
        self.base_properties.occupation = wire[BASE_OCCUPATION_OFFSET];
        self.base_properties.sex = wire[BASE_SEX_OFFSET];
        self.base_properties.pk_count = read_player_wire_u16(wire, BASE_PK_COUNT_OFFSET);
        self.base_properties.kill_count = read_player_wire_u32(wire, BASE_KILL_COUNT_OFFSET);
        self.base_properties.hit_top_log = read_player_wire_u16(wire, BASE_HIT_TOP_LOG_OFFSET);
        self.base_properties.charged = wire[BASE_CHARGED_OFFSET] != 0;
        self.base_properties.remain_point = read_player_wire_u16(wire, BASE_REMAIN_POINT_OFFSET);
        for (index, hotkey) in self.base_properties.hotkeys.iter_mut().enumerate() {
            *hotkey = read_player_wire_u32(wire, BASE_HOTKEY_OFFSET + index * 4);
        }
        self.base_properties.pk_normal = wire[BASE_PK_NORMAL_OFFSET] != 0;
        self.base_properties.pk_team = wire[BASE_PK_TEAM_OFFSET] != 0;
        self.base_properties.pk_union = wire[BASE_PK_UNION_OFFSET] != 0;
        self.base_properties.pk_badman = wire[BASE_PK_BADMAN_OFFSET] != 0;
        self.base_properties.pk_country = wire[BASE_PK_COUNTRY_OFFSET] != 0;
        self.base_properties.health = read_player_wire_u32(wire, BASE_HEALTH_OFFSET);
        self.base_properties.mana = read_player_wire_u32(wire, BASE_MANA_OFFSET);
        self.base_properties.rp = read_player_wire_u16(wire, BASE_RP_OFFSET);
        self.base_properties.yp = read_player_wire_u16(wire, BASE_YP_OFFSET);
        self.base_properties.maximum_yp = read_player_wire_u16(wire, BASE_MAXIMUM_YP_OFFSET);
        self.base_properties.maximum_rp = read_player_wire_u16(wire, BASE_MAXIMUM_RP_OFFSET);
        self.base_properties.base_maximum_hp = read_player_wire_u32(wire, BASE_MAXIMUM_HP_OFFSET);
        self.base_properties.base_maximum_mp = read_player_wire_u32(wire, BASE_MAXIMUM_MP_OFFSET);
        self.base_properties.base_burden = read_player_wire_u16(wire, BASE_BURDEN_OFFSET);
        self.base_properties.base_strength = read_player_wire_u32(wire, BASE_STRENGTH_OFFSET);
        self.base_properties.base_dexterity = read_player_wire_u32(wire, BASE_DEXTERITY_OFFSET);
        self.base_properties.base_constitution =
            read_player_wire_u32(wire, BASE_CONSTITUTION_OFFSET);
        self.base_properties.base_intelligence =
            read_player_wire_u32(wire, BASE_INTELLIGENCE_OFFSET);
        self.base_properties.vigour = read_player_wire_u32(wire, BASE_VIGOUR_OFFSET);
        self.base_properties.maximum_vigour =
            read_player_wire_u32(wire, BASE_MAXIMUM_VIGOUR_OFFSET);
        self.base_properties.energy = read_player_wire_u32(wire, BASE_ENERGY_OFFSET);
        self.base_properties.maximum_energy =
            read_player_wire_u32(wire, BASE_MAXIMUM_ENERGY_OFFSET);
        self.base_properties.credit = read_player_wire_u32(wire, BASE_CREDIT_OFFSET);
        self.base_properties.exalt = read_player_wire_u32(wire, BASE_EXALT_OFFSET);
        self.base_properties.display_head_piece = wire[BASE_DISPLAY_HEAD_PIECE_OFFSET] != 0;
        self.base_properties.quest_time_begin =
            read_player_wire_u32(wire, BASE_QUEST_TIME_BEGIN_OFFSET) as i32;
        self.base_properties.quest_time_limit =
            read_player_wire_u32(wire, BASE_QUEST_TIME_LIMIT_OFFSET) as i32;
        self.base_properties.quest_enabled = wire[BASE_QUEST_ENABLED_OFFSET] != 0;
        self.base_properties.exploit = read_player_wire_u32(wire, BASE_EXPLOIT_OFFSET);
        self.base_properties.fairy_container_enabled =
            wire[BASE_FAIRY_CONTAINER_ENABLED_OFFSET] != 0;
        self.base_properties.battle_fairy_enabled = wire[BASE_BATTLE_FAIRY_ENABLED_OFFSET] != 0;
        self.base_properties.break_armour = read_player_wire_u32(wire, BASE_BREAK_ARMOUR_OFFSET);
        self.base_properties.puncture = read_player_wire_u32(wire, BASE_PUNCTURE_OFFSET);
        self.base_properties.break_element = read_player_wire_u32(wire, BASE_BREAK_ELEMENT_OFFSET);
        self.base_properties.break_bound = read_player_wire_u32(wire, BASE_BREAK_BOUND_OFFSET);
        self.base_properties.power_of_gold = read_player_wire_u32(wire, BASE_POWER_OF_GOLD_OFFSET);
        self.base_properties.days_honor_eliminate =
            read_player_wire_u32(wire, BASE_DAYS_HONOR_OFFSET);
        self.base_properties.weeks_honor_eliminate =
            read_player_wire_u32(wire, BASE_WEEKS_HONOR_OFFSET);
        self.base_properties.months_honor_eliminate =
            read_player_wire_u32(wire, BASE_MONTHS_HONOR_OFFSET);
        self.base_properties.total_honor_eliminate =
            read_player_wire_u32(wire, BASE_TOTAL_HONOR_OFFSET);
        self.base_properties.rank_of_nobility_id =
            read_player_wire_u32(wire, BASE_RANK_OF_NOBILITY_OFFSET);
        self.base_properties.appellation_id = read_player_wire_u32(wire, BASE_APPELLATION_OFFSET);
        self.base_properties.mode = read_player_wire_u32(wire, BASE_MODE_OFFSET);
        self.base_properties.fetch_power = read_player_wire_u32(wire, BASE_FETCH_POWER_OFFSET);
        self.battle_fairy_summoned = wire[BASE_BATTLE_FAIRY_SUMMONED_OFFSET] != 0;
        self.base_properties.battle_fairy_recall = wire[BASE_BATTLE_FAIRY_RECALL_OFFSET] != 0;
        self.base_properties.battle_fairy_died = wire[BASE_BATTLE_FAIRY_DIED_OFFSET] != 0;
        self.base_properties.auction_space = read_player_wire_u32(wire, BASE_AUCTION_SPACE_OFFSET);
        self.base_properties.jjc_level = read_player_wire_u32(wire, BASE_JJC_LEVEL_OFFSET);
        self.base_properties.jjc_score = read_player_wire_u32(wire, BASE_JJC_SCORE_OFFSET);
        self.base_properties.fy_energy = read_player_wire_u32(wire, BASE_FY_ENERGY_OFFSET);
        self.base_properties.fy_enable_flags = LeiTingEnableFlags::from_bits_retain(
            read_player_wire_u32(wire, BASE_FY_ENABLE_FLAGS_OFFSET),
        );
        self.base_properties.lt_up_60_count =
            read_player_wire_u16(wire, BASE_LT_UP_60_COUNT_OFFSET);
        self.base_properties.remain_jing_li_dan_count =
            read_player_wire_u16(wire, BASE_REMAIN_JING_LI_DAN_COUNT_OFFSET);
        self.base_properties.lt_60_stamp = read_player_wire_u32(wire, BASE_LT_60_STAMP_OFFSET);
        self.base_properties.szl = read_player_wire_u32(wire, BASE_SZL_OFFSET);
        self.base_properties.gods_battle_faction =
            read_player_wire_u32(wire, BASE_GODS_BATTLE_FACTION_OFFSET) as i32;
    }

    fn apply_combat_property_wire(&mut self) {
        let wire = &self.combat_property_wire;
        self.combat_properties = PlayerCombatProperties {
            maximum_hp: read_player_wire_u32(wire, 0x00),
            maximum_mp: read_player_wire_u32(wire, 0x04),
            maximum_yp: read_player_wire_u16(wire, 0x08),
            maximum_rp: read_player_wire_u16(wire, 0x0a),
            strength: read_player_wire_u32(wire, 0x0c),
            dexterity: read_player_wire_u32(wire, 0x10),
            constitution: read_player_wire_u32(wire, 0x14),
            intelligence: read_player_wire_u32(wire, 0x18),
            minimum_attack: read_player_wire_u32(wire, 0x1c),
            maximum_attack: read_player_wire_u32(wire, 0x20),
            attack_speed: read_player_wire_u16(wire, 0x32),
            hit: read_player_wire_u16(wire, 0x24),
            dodge: read_player_wire_u16(wire, 0x30),
            cch: read_player_wire_u16(wire, 0x28),
            burden: read_player_wire_u16(wire, 0x26),
            defense: read_player_wire_u32(wire, 0x2c),
            element_resistance: read_player_wire_u32(wire, 0x34),
            add_element_attack: read_player_wire_u32(wire, 0x40),
            hp_recovery: read_player_wire_u16(wire, 0x38),
            mp_recovery: read_player_wire_u16(wire, 0x3a),
            element_modify: read_player_wire_u32(wire, 0x48) as i32,
            reank: read_player_wire_u16(wire, 0x4c),
            attack_avoid: read_player_wire_u16(wire, 0x4e),
            element_avoid: read_player_wire_u16(wire, 0x50),
            full_miss: read_player_wire_u16(wire, 0x52),
            blast_attack: read_player_wire_u16(wire, 0x54),
            blast_element_attack: read_player_wire_u16(wire, 0x56),
            soul_resistance: read_player_wire_u16(wire, 0x3c),
            add_soul_attack: read_player_wire_u16(wire, 0x44),
            blast_attack_scale_bits: read_player_wire_u32(wire, 0x58),
            blast_defense_scale_bits: read_player_wire_u32(wire, 0x5c),
            element_blast_attack_scale_bits: read_player_wire_u32(wire, 0x60),
            element_blast_defense_scale_bits: read_player_wire_u32(wire, 0x64),
            full_miss_scale_bits: read_player_wire_u32(wire, 0x68),
            critical_rate_bits: read_player_wire_u32(wire, 0x6c),
            resume_hp_peace: read_player_wire_u32(wire, 0x70) as i32,
            resume_mp_peace: read_player_wire_u32(wire, 0x74) as i32,
            resume_hp_fight: read_player_wire_u32(wire, 0x78) as i32,
            resume_mp_fight: read_player_wire_u32(wire, 0x7c) as i32,
            restored_hp_peace: read_player_wire_u32(wire, 0x80) as i32,
            restored_mp_peace: read_player_wire_u32(wire, 0x84) as i32,
            restored_hp_fight: read_player_wire_u32(wire, 0x88) as i32,
            restored_mp_fight: read_player_wire_u32(wire, 0x8c) as i32,
            battle_fairy_summoned: wire[0x90] != 0,
            battle_fairy_recall: wire[0x91] != 0,
            battle_fairy_died: wire[0x92] != 0,
        };
    }

    fn decode_organizing_snapshot(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), PlayerGameSaveCodecError> {
        self.faction_id = read_player_game_save_i32(source, cursor, "m_lFactionID")?;
        if self.faction_id > 0 {
            self.faction_logo_id =
                read_player_game_save_i32(source, cursor, "m_lFactionLogoID")?;
            self.faction_level = read_player_game_save_u16(source, cursor, "m_wFactionLevel")?;
            self.faction_experience =
                read_player_game_save_i32(source, cursor, "m_lFactionExperience")?;
            self.faction_force = i32::from(
                read_player_game_save_i32(source, cursor, "m_lForce")? != 0,
            );
            self.faction_contribute = u32::from(
                read_player_game_save_i32(source, cursor, "m_bFactionContribute")? != 0,
            );
            self.faction_name =
                read_player_game_save_string(source, cursor, "m_strFactionName", 0x100)?;
            self.faction_title =
                read_player_game_save_string(source, cursor, "m_strFactionTitle", 0x100)?;
            self.faction_master_id =
                read_player_game_save_i32(source, cursor, "m_lFactionMasterID")?;
            self.union_id = read_player_game_save_i32(source, cursor, "m_lUnionID")?;
            self.union_master_id =
                read_player_game_save_i32(source, cursor, "m_lUnionMasterID")?;
            for (field, destination) in [
                ("m_EnemyFactions", &mut self.enemy_factions),
                ("m_CityWarEnemyFactions", &mut self.city_war_enemy_factions),
            ] {
                let count = read_player_game_save_i32(source, cursor, field)?;
                destination.clear();
                for _ in 0..count.max(0) {
                    destination.insert(read_player_game_save_i32(source, cursor, field)?);
                }
            }
            let count = read_player_game_save_i32(source, cursor, "m_OwnedRegions")?;
            self.faction_owned_regions.clear();
            for _ in 0..count.max(0) {
                let wire = read_player_game_save_slice(source, cursor, "m_OwnedRegions", 8)?;
                self.faction_owned_regions.push(wire.try_into().expect("размер проверен"));
            }
        } else {
            self.faction_logo_id = 0;
            self.faction_level = 0;
            self.faction_experience = 0;
            self.faction_force = 0;
            self.faction_contribute = 0;
            self.faction_name.clear();
            self.faction_title.clear();
            self.enemy_factions.clear();
            self.city_war_enemy_factions.clear();
            self.faction_owned_regions.clear();
            self.faction_master_id = 0;
            self.union_id = 0;
            self.union_master_id = 0;
        }
        Ok(())
    }

    fn encode_organizing_snapshot(&self) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        let mut writer = LegacyWriter::new(&mut payload);
        writer.write_i32(self.faction_id);
        if self.faction_id <= 0 {
            return Some(payload);
        }
        writer.write_i32(self.faction_logo_id);
        writer.write_u16(self.faction_level);
        writer.write_i32(self.faction_experience);
        writer.write_i32(self.faction_force);
        writer.write_u32(self.faction_contribute);
        writer.write_c_string(&self.faction_name);
        writer.write_c_string(&self.faction_title);
        writer.write_i32(self.faction_master_id);
        writer.write_i32(self.union_id);
        writer.write_i32(self.union_master_id);
        writer.write_i32(i32::try_from(self.enemy_factions.len()).ok()?);
        for faction_id in &self.enemy_factions {
            writer.write_i32(*faction_id);
        }
        writer.write_i32(i32::try_from(self.city_war_enemy_factions.len()).ok()?);
        for faction_id in &self.city_war_enemy_factions {
            writer.write_i32(*faction_id);
        }
        writer.write_i32(i32::try_from(self.faction_owned_regions.len()).ok()?);
        for region in &self.faction_owned_regions {
            writer.write_bytes(region);
        }
        Some(payload)
    }

    pub(crate) const fn shape(&self) -> &CShape {
        self.move_shape.shape()
    }

    /// Точный short-вариант `CPlayer::AddToByteArray_ForClient(false)` для
    /// area/query публикаций. Полный login/save вариант остаётся у отдельного
    /// GameSave codec; здесь нет container payload-ов и account-данных.
    pub(crate) fn encode_client_shape_snapshot(
        &self,
        goods_factory: &CGoodsFactory,
        country_identity: u8,
        personal_shop: Option<(i32, i32, &[u8])>,
        team_member_count: usize,
        mut now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        const VISIBLE_EQUIPMENT: [u32; 11] = [0, 1, 2, 3, 4, 9, 10, 12, 13, 14, 15];

        let mut payload = self.move_shape.encode_client_snapshot_with_team_count(
            false,
            self.is_dead(),
            team_member_count,
            &mut now_milliseconds,
        )?;
        let mut writer = LegacyWriter::new(&mut payload);
        writer.write_u8(self.base_properties.head_picture as u8);
        writer.write_u8(u8::from(self.base_properties.display_head_piece));
        for position in VISIBLE_EQUIPMENT {
            writer.write_u32(
                self.equipment
                    .get_goods(position)
                    .map_or(0, CGoods::base_properties_index),
            );
        }
        for position in VISIBLE_EQUIPMENT {
            writer.write_u8(self.equipment.get_goods(position).map_or(0, |goods| {
                goods.addon_property_value(goods_factory, GAP_WEAPON_LEVEL, 1) as u8
            }));
        }
        writer.write_u32(self.base_properties.health);
        writer.write_u32(self.combat_properties.maximum_hp);
        writer.write_u16(self.base_properties.pk_count);
        writer.write_u8(u8::from(self.murderer_time_stamp_ms != 0));
        writer.write_bytes(&self.encode_organizing_snapshot()?);
        writer.write_u8(u8::from(self.contend_state));
        writer.write_u8(u8::from(self.city_war_died_state));
        writer.write_u8(self.base_properties.occupation as u8);
        writer.write_u8(self.base_properties.sex as u8);
        writer.write_u32(self.base_properties.mode);
        if let Some((session_id, plug_id, shop_name)) = personal_shop {
            writer.write_i32(session_id);
            writer.write_i32(plug_id);
            writer.write_c_string(shop_name);
        } else {
            writer.write_i32(0);
            writer.write_i32(0);
        }
        writer.write_i32(self.emotion_index);
        writer.write_u32(now_milliseconds().wrapping_sub(self.emotion_timestamp_ms));
        writer.write_u8(self.country);
        writer.write_u8(self.base_properties.face_picture as u8);
        writer.write_u8(self.base_properties.level);
        writer.write_u32(self.base_properties.credit);
        writer.write_u8(country_identity);
        writer.write_u32(self.base_properties.appellation_id);
        writer.write_u32(self.war_soul_state);
        writer.write_i32(self.base_properties.gods_battle_faction);
        Some(payload)
    }

    /// Точный полный вариант `CPlayer::AddToByteArray_ForClient(true)`,
    /// который `OnLogMessage` вкладывает в успешный `0xBF401`. Persisted
    /// GameSave здесь неприменим: client wire иначе упорядочивает навыки,
    /// контейнеры, валюты, задания и завершающие country/CiQing поля.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode_initial_client_snapshot(
        &mut self,
        goods_factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
        quest_system: &CQuestSystem,
        level_experience: u32,
        country_identity: u8,
        team_member_count: usize,
        loan_time_limit: u32,
        ci_qing_quest_id: u32,
        timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        const SKILL_USAGE_MP_COST: u32 = 2;
        const SKILL_USAGE_MIN_DISTANCE: u32 = 5_002;
        const SKILL_USAGE_MAX_DISTANCE: u32 = 5_003;
        const SKILL_USAGE_DELAY_TIME: u32 = 10_001;

        self.battle_fairy_summoned = self.war_soul_state != 0;
        let mut payload = self.move_shape.encode_client_snapshot_with_team_count(
            true,
            self.is_dead(),
            team_member_count,
            timed_state_now_milliseconds,
        )?;
        payload.extend_from_slice(&self.synchronized_base_property_wire());
        {
            let mut writer = LegacyWriter::new(&mut payload);
            writer.write_c_string(&self.account);
            writer.write_c_string(&self.title);
            writer.write_bytes(&self.combat_property_wire);
            writer.write_u32(level_experience);

            let skills: Vec<_> = self.serializable_skills().collect();
            writer.write_i32(i32::try_from(skills.len()).ok()?);
            for skill in skills {
                let properties = skill_factory
                    .query_skill_base_properties(skill.id(), skill.level())?;
                writer.write_u32(
                    (skill.id() & 0xffff) | ((skill.level() as u32 & 0xffff) << 16),
                );
                writer.write_u32(properties.query_property(SKILL_USAGE_DELAY_TIME));
                let maximum = properties.query_property(SKILL_USAGE_MAX_DISTANCE);
                writer.write_u16(if maximum == 0 { 1 } else { maximum } as u16);
                writer.write_u16(properties.query_property(SKILL_USAGE_MP_COST) as u16);
                writer.write_u16(properties.query_property(SKILL_USAGE_MIN_DISTANCE) as u16);
            }

            writer.write_i32(i32::try_from(self.friends.len()).ok()?);
            for friend in &self.friends {
                writer.write_c_string(&friend.name);
                writer.write_u8(u8::from(friend.online));
            }
        }
        payload.extend_from_slice(&self.encode_lei_ting());

        if let Some(goods) = self.hand.get_goods(0)
            && let Some(base) = goods_factory.query_goods_base_properties(goods.base_properties_index())
        {
            let mut writer = LegacyWriter::new(&mut payload);
            writer.write_u8(1);
            writer.write_u8(u8::from(base.goods_type() == GOODS_TYPE_EQUIPMENT));
            writer.write_u16(goods.amount() as u16);
            writer.write_u8(0);
            goods.serialize_for_old_client(&mut payload, goods_factory, true).then_some(())?;
        } else {
            payload.push(0);
        }

        let equipment = self.equipment.traversing_goods();
        LegacyWriter::new(&mut payload).write_i32(i32::try_from(equipment.len()).ok()?);
        for (column, goods) in equipment {
            goods.serialize_for_old_client(&mut payload, goods_factory, true).then_some(())?;
            LegacyWriter::new(&mut payload).write_u32(column.position());
        }
        append_old_client_volume(&mut payload, &self.auction_goods, goods_factory)?;
        append_old_client_volume(&mut payload, &self.packet, goods_factory)?;
        append_old_client_volume(&mut payload, &self.auction_listing, goods_factory)?;
        append_old_client_volume(&mut payload, self.fairy_container.base(), goods_factory)?;

        for (amount, goods) in [
            (self.wallet.currency_amount(), self.wallet.goods()),
            (self.auction_wallet.currency_amount(), self.auction_wallet.goods()),
            (self.yuan_bao.currency_amount(), self.yuan_bao.goods()),
            (self.ji_fen.currency_amount(), self.ji_fen.goods()),
        ] {
            let mut writer = LegacyWriter::new(&mut payload);
            writer.write_u32(amount);
            let guid = goods.map_or(CGuid::GUID_INVALID, |goods| goods.identity().ex_id);
            if guid.is_invalid() {
                writer.write_u8(0);
            } else {
                writer.write_u8(16);
                writer.write_bytes(guid.as_legacy_bytes());
            }
        }

        payload.extend_from_slice(&self.encode_organizing_snapshot()?);
        {
            let mut writer = LegacyWriter::new(&mut payload);
            writer.write_u8(u8::from(self.contend_state));
            writer.write_i32(i32::from(self.city_war_died_state));
        }
        self.append_client_quest_snapshot(&mut payload, quest_system)?;
        {
            let mut writer = LegacyWriter::new(&mut payload);
            writer.write_u8(self.country);
            writer.write_i32(self.contribution);
            writer.write_u8(country_identity);
            writer.write_u32(loan_time_limit);
        }
        append_old_client_volume(
            &mut payload,
            self.battle_fairy_container.base(),
            goods_factory,
        )?;
        append_old_client_volume(&mut payload, &self.ci_qing_compose, goods_factory)?;
        append_old_client_volume(&mut payload, &self.ci_qing, goods_factory)?;
        {
            let quest_state = self.quest_states.get(&(ci_qing_quest_id as u16)).copied();
            if quest_state == Some(1) {
                self.ci_qing_open = true;
            }
            let mut writer = LegacyWriter::new(&mut payload);
            writer.write_u32(self.war_soul_state);
            writer.write_u32(quest_state.map_or(2, u32::from));
        }
        Some(payload)
    }

    fn append_client_quest_snapshot(
        &self,
        destination: &mut Vec<u8>,
        quest_system: &CQuestSystem,
    ) -> Option<()> {
        let active: Vec<_> = self
            .quest_states
            .iter()
            .filter(|(_, state)| **state != 1)
            .filter_map(|(quest_id, _)| {
                quest_system.quest_data_by_id(*quest_id).map(|quest| (*quest_id, quest))
            })
            .collect();
        let mut writer = LegacyWriter::new(destination);
        writer.write_i32(quest_system.max_quest_count);
        writer.write_i32(i32::try_from(active.len()).ok()?);
        for (quest_id, quest) in active {
            writer.write_u16(quest_id);
            writer.write_u32(quest.old);
            writer.write_u32(quest.quest_type);
            writer.write_u32(quest.level);
            writer.write_u32(quest.difficulty);
            writer.write_u32(quest.track);
            writer.write_c_string(&quest.short_description);
            writer.write_c_string(&quest.name);
            writer.write_c_string(&quest.description);
            writer.write_u8(u8::from(quest.display));
            writer.write_i32(quest.region_id);
            writer.write_i32(quest.tile_x);
            writer.write_i32(quest.tile_y);
            writer.write_i32(quest.effect_id);
        }
        Some(())
    }

    pub(crate) const fn player_ai(&self) -> &CPlayerAI {
        &self.player_ai
    }

    pub(crate) const fn player_ai_mut(&mut self) -> &mut CPlayerAI {
        &mut self.player_ai
    }

    pub(crate) fn take_player_ai(&mut self) -> CPlayerAI {
        std::mem::take(&mut self.player_ai)
    }

    pub(crate) fn restore_player_ai(&mut self, player_ai: CPlayerAI) {
        self.player_ai = player_ai;
    }

    pub(crate) const fn restore_login_team(&mut self, captain: bool, team_id: i32) {
        self.team_captain = captain;
        self.team_id = team_id;
    }

    /// `CTeam::OnPlugInserted/OnPlugEnded` меняют canonical player team ID.
    pub(crate) const fn set_team_membership(&mut self, team_id: i32) {
        self.team_id = team_id;
        if team_id == 0 {
            self.team_captain = false;
        }
    }

    pub(crate) const fn set_team_captain(&mut self, captain: bool) {
        self.team_captain = captain;
    }

    pub(crate) const fn is_team_captain(&self) -> bool {
        self.team_captain
    }

    pub(crate) const fn mark_login_script_started(&mut self) -> bool {
        let first_login = !self.login;
        self.login = true;
        first_login
    }

    pub(crate) const fn player_id(&self) -> i32 {
        self.shape().identity().id
    }

    pub(crate) fn player_name(&self) -> &[u8] {
        self.shape().base_object().get_name()
    }

    pub(crate) fn friends(&self) -> &[PlayerFriend] {
        &self.friends
    }

    pub(crate) fn has_team_recruitment_state(&self) -> bool {
        self.move_shape.team_recruitment_states().next().is_some()
    }

    pub(crate) fn team_recruitment_state_count(&self) -> usize {
        self.move_shape.team_recruitment_states().count()
    }

    pub(crate) fn first_team_recruitment_state(&self) -> Option<&CTeamState> {
        self.move_shape.team_recruitment_states().next()
    }



    /// Exact `GetQuestState`: отсутствующий ushort ID имеет state `2`,
    /// существующий возвращает persisted byte без дополнительной проверки.
    pub(crate) fn quest_state(&self, quest_id: u16) -> i32 {
        self.quest_states
            .get(&quest_id)
            .copied()
            .map_or(2, i32::from)
    }

    pub(crate) fn set_quest_state_snapshot(&mut self, quest_id: u16, state: u8) {
        self.quest_states.insert(quest_id, state);
    }

    pub(crate) fn accept_script_quest(&mut self, quest_id: u16) -> bool {
        if self.quest_states.get(&quest_id).copied() == Some(0) {
            return false;
        }
        self.quest_states.insert(quest_id, 0);
        true
    }

    pub(crate) fn complete_script_quest(&mut self, quest_id: u16) -> bool {
        let Some(state) = self.quest_states.get_mut(&quest_id) else {
            return false;
        };
        *state = 1;
        true
    }

    pub(crate) fn remove_script_quest(&mut self, quest_id: u16) -> bool {
        self.quest_states.remove(&quest_id).is_some()
    }

    pub(crate) fn has_script_quest(&self, quest_id: u16) -> bool {
        self.quest_states.contains_key(&quest_id)
    }

    pub(crate) fn valid_script_quest_count(
        &self,
        mut is_displayed_quest: impl FnMut(u16) -> bool,
    ) -> i32 {
        self.quest_states
            .iter()
            .filter(|(quest_id, state)| **state != 1 && is_displayed_quest(**quest_id))
            .count()
            .try_into()
            .unwrap_or(i32::MAX)
    }

    pub(crate) fn add_friend_state(&mut self, name: &[u8]) -> PlayerFriendAddOutcome {
        let name = name.split(|byte| *byte == 0).next().unwrap_or_default();
        if self.friends.len() >= 0x28 {
            return PlayerFriendAddOutcome::LimitReached;
        }
        if self.friends.iter().any(|friend| friend.name == name) {
            return PlayerFriendAddOutcome::AlreadyPresent;
        }
        self.friends.push(PlayerFriend {
            name: name.to_vec(),
            online: true,
        });
        PlayerFriendAddOutcome::Added
    }

    pub(crate) fn delete_friend_state(&mut self, name: &[u8]) -> bool {
        let name = name.split(|byte| *byte == 0).next().unwrap_or_default();
        let Some(index) = self.friends.iter().position(|friend| friend.name == name) else {
            return false;
        };
        self.friends.remove(index);
        true
    }

    pub(crate) fn has_friend(&self, name: &[u8]) -> bool {
        let name = name.split(|byte| *byte == 0).next().unwrap_or_default();
        self.friends.iter().any(|friend| friend.name == name)
    }

    pub(crate) const fn team_id(&self) -> i32 {
        self.team_id
    }

    pub(crate) const fn is_charged(&self) -> bool {
        self.base_properties.charged
    }

    pub(crate) const fn set_charged(&mut self, charged: bool) {
        self.base_properties.charged = charged;
    }

    pub(crate) const fn faction_id(&self) -> i32 {
        self.faction_id
    }

    pub(crate) const fn faction_level(&self) -> u16 {
        self.faction_level
    }

    pub(crate) const fn faction_experience(&self) -> i32 {
        self.faction_experience
    }

    pub(crate) const fn is_faction_master(&self) -> bool {
        self.faction_id > 0 && self.faction_master_id == self.player_id()
    }

    pub(crate) fn faction_name(&self) -> &[u8] {
        &self.faction_name
    }

    pub(crate) const fn union_id(&self) -> i32 {
        self.union_id
    }

    pub(crate) const fn union_master_id(&self) -> i32 {
        self.union_master_id
    }

    pub(crate) const fn is_union_master(&self) -> bool {
        self.union_id > 0 && self.union_master_id == self.player_id()
    }

    pub(crate) const fn create_faction_operator(&self) -> bool {
        self.create_faction_operator
    }

    pub(crate) const fn set_create_faction_operator(&mut self, value: bool) {
        self.create_faction_operator = value;
    }

    pub(crate) const fn apply_join_faction_operator(&self) -> bool {
        self.apply_join_faction_operator
    }

    pub(crate) const fn set_apply_join_faction_operator(&mut self, value: bool) {
        self.apply_join_faction_operator = value;
    }

    pub(crate) const fn faction_declare_operator(&self) -> bool {
        self.faction_declare_operator
    }

    pub(crate) const fn set_faction_declare_operator(&mut self, value: bool) {
        self.faction_declare_operator = value;
    }

    pub(crate) fn restore_faction_identity(
        &mut self,
        faction_id: i32,
        faction_logo_id: i32,
        faction_level: u16,
        faction_experience: i32,
        faction_force: i32,
        faction_contribute: u32,
        faction_master_id: i32,
        faction_name: &[u8],
        faction_title: &[u8],
        union_id: i32,
        union_master_id: i32,
        enemy_factions: BTreeSet<i32>,
        city_war_enemy_factions: BTreeSet<i32>,
        faction_owned_regions: Vec<[u8; 8]>,
    ) {
        self.faction_id = faction_id;
        self.faction_logo_id = faction_logo_id;
        self.faction_level = faction_level;
        self.faction_experience = faction_experience;
        self.faction_force = i32::from(faction_force != 0);
        self.faction_contribute = u32::from(faction_contribute != 0);
        self.faction_master_id = faction_master_id;
        self.faction_name.clear();
        self.faction_name.extend_from_slice(faction_name);
        self.faction_title.clear();
        self.faction_title.extend_from_slice(faction_title);
        self.union_id = union_id;
        self.union_master_id = union_master_id;
        self.enemy_factions = enemy_factions;
        self.city_war_enemy_factions = city_war_enemy_factions;
        self.faction_owned_regions = faction_owned_regions;
    }

    pub(crate) fn is_enemy_faction_member(&self, faction_id: i32) -> bool {
        self.faction_id > 0 && self.enemy_factions.contains(&faction_id)
    }

    pub(crate) fn is_city_war_enemy_faction_member(&self, faction_id: i32) -> bool {
        self.faction_id > 0 && self.city_war_enemy_factions.contains(&faction_id)
    }

    pub(crate) const fn country(&self) -> u8 {
        self.country
    }

    /// `CPlayer::SetScriptValue("btCountry")` сужает signed script value до
    /// исходного byte storage без country-range validation.
    pub(crate) const fn set_script_country(&mut self, requested: i32) -> i32 {
        self.country = requested as u8;
        self.country as i32
    }

    /// Ответ World `0x7FF01` меняет страну только для signed диапазона `1..4`;
    /// невалидное значение всё равно публикуется caller-ом в `0xC0301`.
    pub(crate) fn apply_world_country(&mut self, requested: i32) {
        let previous = self.country;
        if (1..5).contains(&requested) {
            self.country = requested as u8;
        }
        tracing::trace!(
            player_id = self.player_id(),
            previous,
            requested,
            applied = self.country,
            changed = self.country != previous,
            "страна игрока изменена ответом World"
        );
    }

    /// Сбрасывает подтверждённые счётчики чести и возвращает exact условие
    /// `AdjustHonorRank`: ненулевой nobility rank требует запуска сценария.
    pub(crate) fn reset_total_honor_eliminate(&mut self, reset_mask: u32) -> bool {
        let player_id = self.player_id();
        let previous_days = self.base_properties.days_honor_eliminate;
        let previous_weeks = self.base_properties.weeks_honor_eliminate;
        let previous_months = self.base_properties.months_honor_eliminate;
        let adjust_honor_rank = self.base_properties.rank_of_nobility_id != 0;
        self.base_properties.days_honor_eliminate = 0;
        if reset_mask & 2 != 0 {
            self.base_properties.weeks_honor_eliminate = 0;
        }
        if reset_mask & 4 != 0 {
            self.base_properties.months_honor_eliminate = 0;
        }
        tracing::trace!(
            player_id,
            reset_mask,
            previous_days,
            previous_weeks,
            previous_months,
            adjust_honor_rank,
            "счётчики чести игрока сброшены"
        );
        adjust_honor_rank
    }

    pub(crate) const fn server_region_id(&self) -> Option<i32> {
        self.server_region_id
    }

    pub(crate) const fn in_changing_server(&self) -> bool {
        self.in_changing_server
    }

    pub(crate) const fn in_changing_region(&self) -> bool {
        self.in_changing_region
    }

    /// Exact anti-repeat prefix `8F801`: timestamp записывается после
    /// успешного EnterTime gate, но ещё до проверки `m_bInChangingRegion`.
    pub(crate) fn accept_region_entry_ack(&mut self, now_ms: u32, enter_time_ms: u32) -> bool {
        if now_ms.wrapping_sub(self.last_enter_region_tick_ms) < enter_time_ms {
            return false;
        }
        self.last_enter_region_tick_ms = now_ms;
        self.in_changing_region
    }

    pub(crate) const fn begin_region_entry_states(&mut self) {
        self.move_shape.reset_region_entry_control();
    }

    pub(crate) const fn mark_entered_region(&mut self) {
        self.entered_region = true;
    }

    pub(crate) const fn set_changing_state_snapshot(
        &mut self,
        in_changing_server: bool,
        in_changing_region: bool,
    ) {
        self.in_changing_server = in_changing_server;
        self.in_changing_region = in_changing_region;
    }

    /// Player-owned scalar tail локального `ChangeRegion`; фактическое
    /// membership перемещение остаётся deferred у `CServerRegion::AI`.
    pub(crate) fn stage_local_region_change(
        &mut self,
        region_id: i32,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
    ) {
        self.in_changing_server = false;
        self.in_changing_region = true;
        self.entered_region = false;
        self.movement_shape_mut()
            .stage_region_change(region_id, tile_x, tile_y, direction);
    }

    pub(crate) fn begin_cross_region_companion_change(&mut self) {
        self.recreate_carriage = false;
        self.uncreated_carriage = PlayerUncreatedCarriage::default();
    }

    /// Снимки companion-ов принадлежат игроку между выходом из исходного
    /// region owner-а и созданием новых monster-owner-ов в назначении.
    pub(crate) fn store_uncreated_region_pets(&mut self, pets: Vec<PlayerUncreatedPet>) {
        self.uncreated_pets = pets;
    }

    pub(crate) fn store_uncreated_region_carriage(&mut self, carriage: PlayerUncreatedCarriage) {
        self.uncreated_carriage = carriage;
        self.recreate_carriage = true;
    }

    /// Same-region `ChangeRegion` не пересоздаёт повозку: возможный перенос
    /// живого monster-owner выполняет `CGame`, а persisted snapshot сбрасывает
    /// player-owner до его поиска, как исходный `m_bReCreateCarriage = false`.
    pub(crate) const fn begin_same_region_change(&mut self) {
        self.recreate_carriage = false;
    }

    pub(crate) fn begin_server_region_change(&mut self) {
        self.state_before_server_region_change = self.shape().get_state();
        self.movement_shape_mut().set_state(0);
        self.in_changing_server = true;
        self.in_changing_region = true;
        self.entered_region = false;
        self.recreate_carriage = false;
    }

    pub(crate) fn cancel_server_region_change(&mut self) {
        if self.in_changing_server {
            let state = self.state_before_server_region_change;
            self.movement_shape_mut().set_state(state);
        }
        self.in_changing_server = false;
        self.in_changing_region = false;
    }

    pub(crate) fn apply_staged_local_region_change(&mut self) -> (i32, i32, i32, i32) {
        let destination = self.movement_shape_mut().apply_staged_region_change();
        self.server_region_id = Some(destination.0);
        destination
    }

    pub(crate) const fn current_progress(&self) -> PlayerProgress {
        self.current_progress
    }

    pub(crate) const fn set_current_progress_snapshot(&mut self, progress: PlayerProgress) {
        self.current_progress = progress;
    }

    pub(crate) const fn set_personal_shop_flag(&mut self, session_id: i32, plug_id: i32) {
        self.personal_shop_session_id = session_id;
        self.personal_shop_plug_id = plug_id;
    }

    pub(crate) const fn personal_shop_flag(&self) -> Option<(i32, i32)> {
        if self.personal_shop_session_id == 0 || self.personal_shop_plug_id == 0 {
            None
        } else {
            Some((self.personal_shop_session_id, self.personal_shop_plug_id))
        }
    }

    pub(crate) fn begin_equipment_session(
        &mut self,
        progress: PlayerProgress,
        lock_movement: bool,
    ) -> GoodsSessionPlayerRelease {
        let previous_progress = self.current_progress;
        let previous_moveable_count = self.move_shape.moveable_count();
        self.current_progress = progress;
        if lock_movement {
            self.move_shape.set_moveable(false);
        }
        GoodsSessionPlayerRelease {
            previous_progress,
            previous_moveable_count,
            resulting_moveable_count: self.move_shape.moveable_count(),
            moveable: self.move_shape.is_moveable(),
        }
    }

    pub(crate) fn attach_equipment_session_listener(&mut self, plug_id: i32) -> [bool; 2] {
        let listener = usize::try_from(plug_id)
            .ok()
            .and_then(ContainerListenerHandle::from_legacy_identity);
        let packet = self
            .packet
            .base_mut()
            .base_mut()
            .base_mut()
            .add_listener(listener);
        let equipment = self.equipment.base_mut().base_mut().add_listener(listener);
        [packet, equipment]
    }

    pub(crate) fn detach_equipment_session_listener(&mut self, plug_id: i32) -> [bool; 2] {
        let listener = usize::try_from(plug_id)
            .ok()
            .and_then(ContainerListenerHandle::from_legacy_identity);
        let packet = self
            .packet
            .base_mut()
            .base_mut()
            .base_mut()
            .remove_listener(listener);
        let equipment = self
            .equipment
            .base_mut()
            .base_mut()
            .remove_listener(listener);
        [packet, equipment]
    }

    pub(crate) const fn is_dead(&self) -> bool {
        CMoveShape::is_died(self.base_properties.health)
    }

    pub(crate) const fn set_god_mode(&mut self, enabled: bool) {
        self.move_shape.set_god(enabled);
    }

    pub(crate) const fn is_god_mode(&self) -> bool {
        self.move_shape.is_god()
    }

    pub(crate) fn release_goods_session_state(&mut self) -> GoodsSessionPlayerRelease {
        let previous_progress = self.current_progress;
        let previous_moveable_count = self.move_shape.moveable_count();
        self.current_progress = PlayerProgress::None;
        self.move_shape.set_moveable(true);
        GoodsSessionPlayerRelease {
            previous_progress,
            previous_moveable_count,
            resulting_moveable_count: self.move_shape.moveable_count(),
            moveable: self.move_shape.is_moveable(),
        }
    }

    pub(crate) fn begin_synthesis(&mut self) -> GoodsSessionPlayerRelease {
        let previous_progress = self.current_progress;
        let previous_moveable_count = self.move_shape.moveable_count();
        self.current_progress = PlayerProgress::Synthesis;
        self.move_shape.set_moveable(false);
        GoodsSessionPlayerRelease {
            previous_progress,
            previous_moveable_count,
            resulting_moveable_count: self.move_shape.moveable_count(),
            moveable: self.move_shape.is_moveable(),
        }
    }

    pub(crate) fn close_synthesis(&mut self) -> Option<GoodsSessionPlayerRelease> {
        (self.current_progress == PlayerProgress::Synthesis)
            .then(|| self.release_goods_session_state())
    }

    pub(crate) fn depot_password(&self) -> &[u8] {
        &self.depot_password
    }

    /// Exact `SetDepotPassword`: nullable C-string уже разрешена caller-ом;
    /// сохраняются только bytes до первого NUL.
    pub(crate) fn set_depot_password(&mut self, password: &[u8]) {
        let length = password
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(password.len());
        self.depot_password.clear();
        self.depot_password.extend_from_slice(&password[..length]);
    }

    pub(crate) fn unlock_depot_storage(&mut self) {
        let _legacy_bank_result = self.bank.unlock_if_authenticated(true);
        let _legacy_depot_result = self.depot.unlock_if_authenticated(true);
    }

    pub(crate) fn close_depot_storage(&mut self) {
        let _legacy_bank_result = self.bank.lock();
        let _legacy_depot_result = self.depot.lock();
        self.current_progress = PlayerProgress::None;
    }

    pub(crate) fn prepare_depot_storage(&mut self, password_required: bool) {
        if password_required {
            let _ = self.bank.lock();
            let _ = self.depot.lock();
        } else {
            let _ = self.bank.unlock_if_authenticated(true);
            let _ = self.depot.unlock_if_authenticated(true);
        }
    }

    pub(crate) fn bank_snapshot_goods(&self, position: u32) -> Option<&CGoods> {
        self.bank.snapshot_goods(position)
    }

    pub(crate) const fn bank_locked(&self) -> bool {
        self.bank.is_locked()
    }

    pub(crate) const fn depot_locked(&self) -> bool {
        self.depot.is_locked()
    }

    pub(crate) const fn base_properties(&self) -> PlayerBaseProperties {
        self.base_properties
    }

    pub(crate) const fn set_display_head_piece(&mut self, display: bool) {
        self.base_properties.display_head_piece = display;
    }

    pub(crate) const fn display_head_piece(&self) -> bool {
        self.base_properties.display_head_piece
    }

    pub(crate) const fn quest_enabled(&self) -> bool {
        self.base_properties.quest_enabled
    }

    pub(crate) const fn set_quest_enabled(&mut self, enabled: bool) {
        self.base_properties.quest_enabled = enabled;
    }

    pub(crate) const fn begin_quest_time(&mut self, now_seconds: i32, time_limit: i32) {
        self.base_properties.quest_time_begin = now_seconds;
        self.base_properties.quest_time_limit = time_limit;
    }

    pub(crate) const fn clear_quest_time(&mut self) {
        self.base_properties.quest_time_begin = 0;
        self.base_properties.quest_time_limit = 0;
    }

    pub(crate) const fn quest_time_remaining(&self, now_seconds: i32) -> i32 {
        if self.base_properties.quest_time_begin == 0 || self.base_properties.quest_time_limit == 0
        {
            return 0;
        }
        let remaining = self
            .base_properties
            .quest_time_begin
            .wrapping_add(self.base_properties.quest_time_limit)
            .wrapping_sub(now_seconds);
        if remaining < 0 { 0 } else { remaining }
    }

    pub(crate) const fn acknowledge_heartbeat(&mut self) {
        self.heart_request_sent = 0;
        self.heart_received = true;
    }

    pub(crate) fn encode_lei_ting(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(20 + self.lei_ting_things.len() * 8);
        let mut writer = LegacyWriter::new(&mut payload);
        writer.write_u32(self.base_properties.fy_enable_flags.bits());
        writer.write_u32(self.base_properties.fy_energy);
        writer.write_u32(self.base_properties.lt_60_stamp);
        writer.write_u16(self.base_properties.lt_up_60_count);
        writer.write_u16(self.base_properties.remain_jing_li_dan_count);
        writer.write_u32(self.lei_ting_things.len() as u32);
        for thing in &self.lei_ting_things {
            writer.write_u16(thing.thing_id);
            writer.write_u16(thing.count);
            writer.write_u16(thing.max_count);
            writer.write_u16(thing.point);
        }
        payload
    }

    pub(crate) fn decode_lei_ting(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), PlayerLeiTingDecodeBlock> {
        fn read_u32(
            source: &[u8],
            cursor: &mut usize,
            field: &'static str,
        ) -> Result<u32, PlayerLeiTingDecodeBlock> {
            let offset = *cursor;
            let available = source.len().saturating_sub(offset);
            let mut reader =
                LegacyReader::at(source, offset).map_err(|_| PlayerLeiTingDecodeBlock {
                    field,
                    offset,
                    needed: 4,
                    available,
                })?;
            let value = reader.read_u32().map_err(|_| PlayerLeiTingDecodeBlock {
                field,
                offset,
                needed: 4,
                available,
            })?;
            *cursor = reader.position();
            Ok(value)
        }
        fn read_u16(
            source: &[u8],
            cursor: &mut usize,
            field: &'static str,
        ) -> Result<u16, PlayerLeiTingDecodeBlock> {
            let offset = *cursor;
            let available = source.len().saturating_sub(offset);
            let mut reader =
                LegacyReader::at(source, offset).map_err(|_| PlayerLeiTingDecodeBlock {
                    field,
                    offset,
                    needed: 2,
                    available,
                })?;
            let value = reader.read_u16().map_err(|_| PlayerLeiTingDecodeBlock {
                field,
                offset,
                needed: 2,
                available,
            })?;
            *cursor = reader.position();
            Ok(value)
        }

        self.base_properties.fy_enable_flags =
            LeiTingEnableFlags::from_bits_retain(read_u32(source, cursor, "dwfyenFlag")?);
        self.base_properties.fy_energy = read_u32(source, cursor, "dwfyEnergy")?;
        self.base_properties.lt_60_stamp = read_u32(source, cursor, "dwLT60Stamp")?;
        self.base_properties.lt_up_60_count = read_u16(source, cursor, "wLTUp60Cnt")?;
        self.base_properties.remain_jing_li_dan_count =
            read_u16(source, cursor, "wRemainJingLiDanCnt")?;
        self.lei_ting_things.clear();
        let count = read_u32(source, cursor, "m_listThing count")?;
        for _ in 0..count {
            self.lei_ting_things.push_back(PlayerLeiTingThing {
                thing_id: read_u16(source, cursor, "tagThing.wTID")?,
                count: read_u16(source, cursor, "tagThing.wCnt")?,
                max_count: read_u16(source, cursor, "tagThing.wMaxCnt")?,
                point: read_u16(source, cursor, "tagThing.wPoint")?,
            });
        }
        Ok(())
    }

    pub(crate) const fn change_fy_energy_flag(&mut self, index: u16) -> bool {
        let threshold_reached = match index {
            0 => self.base_properties.fy_energy >= 20,
            1 => self.base_properties.fy_energy >= 60,
            2 => self.base_properties.fy_energy >= 80,
            3 => self.base_properties.fy_energy >= 100,
            4 => self.base_properties.lt_up_60_count >= 4,
            5 => self.base_properties.lt_up_60_count >= 10,
            6 => self.base_properties.lt_up_60_count >= 16,
            7 => self.base_properties.lt_up_60_count >= 22,
            8 => self.base_properties.lt_up_60_count >= 28,
            _ => return false,
        };
        let flag = LeiTingEnableFlags::from_bits_retain(1u32 << index);
        if !threshold_reached || self.base_properties.fy_enable_flags.contains(flag) {
            return false;
        }
        self.base_properties.fy_enable_flags = LeiTingEnableFlags::from_bits_retain(
            self.base_properties.fy_enable_flags.bits() | flag.bits(),
        );
        true
    }

    /// `GetOneThing((ushort)id)`: lookup намеренно сохраняет narrowing без
    /// последующей проверки исходного signed ID, как ветвь `GetThingCnt`.
    pub(crate) fn lei_ting_thing_count(&self, thing_id: i32) -> Option<u16> {
        let narrowed = thing_id as u16;
        self.lei_ting_things
            .iter()
            .find(|thing| thing.thing_id == narrowed)
            .map(|thing| thing.count)
    }

    /// Полный reached `AddThingCnt(id, count, false)`. В отличие от getter-а,
    /// setter после ushort lookup сравнивает сохранённый ID с исходным signed
    /// аргументом. Разрешено только строго положительное увеличение; энергия
    /// и суточные поля сохраняют wrapping-арифметику x86 owner-а. Closure —
    /// только CRT/local-time граница `AddLTUp60Cnt`; wire исполняет script
    /// caller после успешной mutation.
    pub(crate) fn set_lei_ting_thing_count(
        &mut self,
        thing_id: i32,
        requested_count: i32,
        next_daily_stamp_if_same_local_day: impl FnOnce(u32) -> Option<u32>,
    ) -> PlayerLeiTingThingCountOutcome {
        let narrowed = thing_id as u16;
        let Some(index) = self
            .lei_ting_things
            .iter()
            .position(|thing| thing.thing_id == narrowed)
        else {
            return PlayerLeiTingThingCountOutcome::Missing;
        };
        let current = self.lei_ting_things[index];
        if u32::from(current.thing_id) != thing_id as u32 {
            return PlayerLeiTingThingCountOutcome::Missing;
        }
        let difference = requested_count.wrapping_sub(i32::from(current.count));
        if difference < 1
            || difference > i32::from(current.max_count)
            || requested_count > i32::from(current.max_count)
        {
            return PlayerLeiTingThingCountOutcome::Rejected {
                current: current.count,
                requested: requested_count,
                maximum: current.max_count,
            };
        }

        self.lei_ting_things[index].count = requested_count as u16;
        let previous_energy = self.base_properties.fy_energy;
        self.base_properties.fy_energy =
            previous_energy.wrapping_add(u32::from(current.point).wrapping_mul(difference as u32));
        let daily_count_incremented = if self.base_properties.fy_energy >= 60 {
            if let Some(next_stamp) =
                next_daily_stamp_if_same_local_day(self.base_properties.lt_60_stamp)
            {
                self.base_properties.lt_up_60_count =
                    self.base_properties.lt_up_60_count.wrapping_add(1);
                self.base_properties.lt_60_stamp = next_stamp;
                true
            } else {
                false
            }
        } else {
            false
        };
        PlayerLeiTingThingCountOutcome::Updated {
            previous_count: current.count,
            current_count: requested_count as u16,
            previous_energy,
            current_energy: self.base_properties.fy_energy,
            daily_count_incremented,
        }
    }

    pub(crate) const fn honor_snapshot(&self) -> PlayerHonorSnapshot {
        PlayerHonorSnapshot {
            rank_of_nobility_id: self.base_properties.rank_of_nobility_id,
            appellation_id: self.base_properties.appellation_id,
            days_eliminate: self.base_properties.days_honor_eliminate,
            weeks_eliminate: self.base_properties.weeks_honor_eliminate,
            months_eliminate: self.base_properties.months_honor_eliminate,
            total_eliminate: self.base_properties.total_honor_eliminate,
        }
    }

    /// World acknowledgement `0x7FA16` подтверждает уже принятую honor-пару:
    /// все четыре счётчика увеличиваются независимо с DWORD wrapping.
    pub(crate) const fn acknowledge_honor_eliminate(&mut self) -> PlayerHonorEliminateMutation {
        let previous = [
            self.base_properties.days_honor_eliminate,
            self.base_properties.weeks_honor_eliminate,
            self.base_properties.months_honor_eliminate,
            self.base_properties.total_honor_eliminate,
        ];
        self.base_properties.days_honor_eliminate = previous[0].wrapping_add(1);
        self.base_properties.weeks_honor_eliminate = previous[1].wrapping_add(1);
        self.base_properties.months_honor_eliminate = previous[2].wrapping_add(1);
        self.base_properties.total_honor_eliminate = previous[3].wrapping_add(1);
        PlayerHonorEliminateMutation {
            player_id: self.player_id(),
            previous,
            current: [
                self.base_properties.days_honor_eliminate,
                self.base_properties.weeks_honor_eliminate,
                self.base_properties.months_honor_eliminate,
                self.base_properties.total_honor_eliminate,
            ],
        }
    }

    pub(crate) const fn request_change_appellation_state(&mut self, appellation_id: u32) {
        self.attempt_appellation_id = appellation_id;
    }


    pub(crate) const fn jjc_pk_state(&self) -> bool {
        self.jjc_pk_state
    }

    pub(crate) const fn set_jjc_pk_state(&mut self, active: bool) {
        self.jjc_pk_state = active;
    }

    pub(crate) const fn jjc_level(&self) -> u32 {
        self.base_properties.jjc_level
    }

    pub(crate) const fn jjc_score(&self) -> u32 {
        self.base_properties.jjc_score
    }

    pub(crate) const fn jjc_data(&self) -> &[u8; 0x10] {
        &self.jjc_data
    }

    /// Восемь `unsigned short` из точного `tagJJcData` лежат подряд в
    /// GameSave/wire-порядке: четыре недельных, затем четыре сезонных счётчика.
    pub(crate) fn jjc_counter(&self, selector: i32) -> Option<u16> {
        let index = usize::try_from(selector.checked_sub(1)?).ok()?;
        let offset = index.checked_mul(2)?;
        let bytes = self.jjc_data.get(offset..offset.checked_add(2)?)?;
        Some(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub(crate) fn set_jjc_counter(&mut self, selector: i32, value: u16) -> bool {
        let Some(offset) = selector
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| index.checked_mul(2))
        else {
            return false;
        };
        let Some(bytes) = self
            .jjc_data
            .get_mut(offset..offset.saturating_add(2))
        else {
            return false;
        };
        bytes.copy_from_slice(&value.to_le_bytes());
        true
    }

    /// Exact `JJcWeekClear`: первые четыре WORD — weekly counters; при
    /// пятнадцати участиях score получает level-dependent award с x87
    /// усечением к нулю и cap 1500.
    pub(crate) fn clear_jjc_week(&mut self) {
        let joined = LegacyReader::new(&self.jjc_data)
            .read_u16()
            .expect("JJC data содержит флаг участия");
        if joined >= 15 {
            let factor = if self.base_properties.jjc_level < 1001 {
                100.0
            } else {
                300.0
            };
            let award = (f64::from(self.base_properties.jjc_level) * 0.001 * factor)
                .trunc()
                .clamp(0.0, 1500.0) as u32;
            self.base_properties.jjc_score = self.base_properties.jjc_score.wrapping_add(award);
        }
        self.jjc_data[..8].fill(0);
    }

    pub(crate) fn clear_jjc_season(&mut self, default_level: i32) {
        self.jjc_data.fill(0);
        self.base_properties.jjc_level = default_level as u32;
        self.base_properties.jjc_score = 0;
    }

    pub(crate) fn get_appellation_state(&self, state_id: u32) -> u32 {
        self.move_shape.get_undead_state(state_id)
    }



    pub(crate) fn change_body_check(&self) -> bool {
        self.base_properties.mode == 0
            && !matches!(
                self.current_progress,
                PlayerProgress::Trading | PlayerProgress::OpenStall
            )
            && self.team_id == 0
            && !self.is_rider()
            && !self.has_pet()
            && self.uncreated_carriage.original_name.is_empty()
    }

    pub(crate) fn has_change_body_state(&self) -> bool {
        self.move_shape.active_change_body_state().is_some()
    }


    pub(crate) fn get_change_body_state(&self, state_id: u32) -> u32 {
        self.move_shape.get_change_body_state(state_id)
    }

    pub(crate) fn first_change_body_state_id(&self) -> Option<u32> {
        self.move_shape.first_change_body_state_id()
    }





    pub(crate) fn is_rider(&self) -> bool {
        self.move_shape.has_ride_state()
    }

    pub(crate) fn callosity_state(
        &self,
    ) -> Option<super::skills::callositystate::CallosityFamilyState> {
        self.move_shape.callosity_state()
    }

    pub(crate) fn take_callosity_state(
        &mut self,
        skill_id: u32,
    ) -> Option<super::skills::callositystate::CallosityFamilyState> {
        self.move_shape.take_callosity_state(skill_id)
    }

    pub(crate) fn begin_callosity_state(
        &mut self,
        state: super::skills::callositystate::CallosityFamilyState,
    ) {
        self.move_shape.begin_callosity_state(state);
    }


    pub(crate) fn replace_swordship_state(
        &mut self,
        state: super::skills::swordshipstate::SwordshipState,
    ) -> Option<super::skills::swordshipstate::SwordshipState> {
        self.move_shape.replace_swordship_state(state)
    }

    pub(crate) fn replace_wuxing_state(
        &mut self,
        state: super::skills::wuxingstate::WuXingState,
    ) -> Option<super::skills::wuxingstate::WuXingState> {
        self.move_shape.replace_wuxing_state(state)
    }

    pub(crate) fn agility_state(
        &self,
        skill_id: u32,
    ) -> Option<super::skills::agilitystate::AgilityState> {
        self.move_shape.agility_state(skill_id)
    }

    pub(crate) fn take_agility_state(
        &mut self,
        skill_id: u32,
    ) -> Option<super::skills::agilitystate::AgilityState> {
        self.move_shape.take_agility_state(skill_id)
    }

    pub(crate) fn begin_agility_state(
        &mut self,
        state: super::skills::agilitystate::AgilityState,
    ) {
        self.move_shape.begin_agility_state(state);
    }

    pub(crate) fn agility_state_2(
        &self,
    ) -> Option<super::skills::agilitystate2::AgilityState2> {
        self.move_shape.agility_state_2()
    }

    pub(crate) fn take_agility_state_2(
        &mut self,
    ) -> Option<super::skills::agilitystate2::AgilityState2> {
        self.move_shape.take_agility_state_2()
    }

    pub(crate) fn begin_agility_state_2(
        &mut self,
        state: super::skills::agilitystate2::AgilityState2,
    ) {
        self.move_shape.begin_agility_state_2(state);
    }

    pub(crate) fn take_persistent_agility_family_state(
        &mut self,
    ) -> Option<super::skills::agilitystate::PersistentAgilityFamilyState> {
        self.move_shape.take_persistent_agility_family_state()
    }

    pub(crate) fn begin_persistent_agility_family_state(
        &mut self,
        state: super::skills::agilitystate::PersistentAgilityFamilyState,
    ) {
        self.move_shape.begin_persistent_agility_family_state(state);
    }



    pub(crate) fn replace_taiji_state(
        &mut self,
        state: super::skills::taijistate::TaiJiState,
    ) -> Option<super::skills::taijistate::TaiJiState> {
        self.move_shape.replace_taiji_state(state)
    }

    pub(crate) fn replace_enlarge_max_hp_state(
        &mut self,
        state: super::skills::enlargemaxhpstate::EnlargeMaxHpState,
    ) -> Option<super::skills::enlargemaxhpstate::EnlargeMaxHpState> {
        self.move_shape.replace_enlarge_max_hp_state(state)
    }

    pub(crate) fn replace_enlarge_full_miss_state(
        &mut self,
        state: super::skills::enlargefullmissstate::EnlargeFullMissState,
    ) -> Option<super::skills::enlargefullmissstate::EnlargeFullMissState> {
        self.move_shape.replace_enlarge_full_miss_state(state)
    }

    pub(crate) fn replace_enlarge_max_mp_state(
        &mut self,
        state: super::skills::enlargemaxmpstate::EnlargeMaxMpState,
    ) -> Option<super::skills::enlargemaxmpstate::EnlargeMaxMpState> {
        self.move_shape.replace_enlarge_max_mp_state(state)
    }

    pub(crate) fn replace_origin_state(
        &mut self,
        state: super::skills::originstate::OriginState,
    ) -> Option<super::skills::originstate::OriginState> {
        self.move_shape.replace_origin_state(state)
    }

    pub(crate) fn replace_hearten_state(
        &mut self,
        state: super::skills::heartenstate::HeartenState,
    ) -> Option<super::skills::heartenstate::HeartenState> {
        self.move_shape.replace_hearten_state(state)
    }

    pub(crate) fn begin_promotion_state(
        &mut self,
        state: super::skills::promotionstate::PromotionState,
    ) -> bool {
        self.move_shape.begin_promotion_state(state)
    }

    pub(crate) fn promotion_heal_recover_factor(&self) -> Option<u16> {
        self.move_shape.promotion_heal_recover_factor()
    }



    pub(crate) fn replace_heal_state(
        &mut self,
        removed_skill_id: u32,
        state: super::skills::healstate::HealState,
    ) -> Option<super::skills::healstate::HealState> {
        self.move_shape
            .replace_heal_state(removed_skill_id, state)
    }

    pub(crate) fn rage_break_state(&self) -> Option<super::skills::ragebreakstate::RageBreakState> {
        self.move_shape.rage_break_state()
    }

    pub(crate) fn replace_rage_break_state(
        &mut self,
        state: super::skills::ragebreakstate::RageBreakState,
    ) -> Option<super::skills::ragebreakstate::RageBreakState> {
        self.move_shape.replace_rage_break_state(state)
    }


    pub(crate) fn take_rage_break_state(&mut self) -> Option<super::skills::ragebreakstate::RageBreakState> {
        self.move_shape.take_rage_break_state()
    }

    pub(crate) fn restart_rage_break_state(&mut self, now_ms: u32) -> bool {
        self.move_shape.restart_rage_break_state(now_ms)
    }



    pub(crate) fn push_fury_state(
        &mut self,
        state: super::skills::furystate::FuryState,
    ) {
        self.move_shape.push_fury_state(state);
    }


    pub(crate) fn fury_states(&self) -> impl Iterator<Item = &super::skills::furystate::FuryState> {
        self.move_shape.fury_states()
    }

    pub(crate) fn remove_fury_state(&mut self, position: usize) -> Option<super::skills::furystate::FuryState> {
        self.move_shape.remove_fury_state(position)
    }

    pub(crate) fn remove_serialized_heal_states(&mut self, skill_ids: &[u32]) {
        self.move_shape.remove_serialized_heal_states(skill_ids);
    }

    pub(crate) fn remove_serialized_heal_state(&mut self, skill_id: u32, occurrence: usize) {
        self.move_shape.remove_serialized_heal_state(skill_id, occurrence);
    }


    pub(crate) fn replace_boss_blue_quake_state(
        &mut self,
        state: super::skills::bossbluequakestate::BossBlueQuakeState,
    ) -> Option<super::skills::bossbluequakestate::BossBlueQuakeState> {
        self.move_shape.replace_boss_blue_quake_state(state)
    }




    pub(crate) fn take_boss_blue_quake_state(
        &mut self,
    ) -> Option<super::skills::bossbluequakestate::BossBlueQuakeState> {
        self.move_shape.take_boss_blue_quake_state()
    }

    /// Базовая и equipment/CiQing половина `CPlayer::UpdateProperty` до
    /// виртуального `CMoveShape::UpdateProperty`. Два signed addon-pass-а
    /// сохраняют slot order `MountAllEquip`. Восемь recovery scalar-ов
    /// каждый раз восстанавливаются из `CGlobeSetup::tagSetup` до применения
    /// equipment, CiQing и state addon-ов, как остальные базовые свойства.
    pub(crate) fn recompute_base_and_equipment_properties(
        &self,
        coefficients: GlobePlayerPropertyCoefficients,
        base_combat_scales: [f32; 5],
        critical_rate: f32,
        goods_factory: &CGoodsFactory,
    ) -> PlayerCombatProperties {
        self.recompute_base_and_mounted_properties(
            coefficients,
            base_combat_scales,
            critical_rate,
            goods_factory,
            true,
        )
    }

    /// Базовые cases `0x80..0x84` исходного `MountCiQingEquip`. Они не входят
    /// в combat snapshot: owner меняет сохранённый `tagBaseProperty` в порядке
    /// восьми ячеек, сперва для неотрицательных, затем для отрицательных
    /// addon-ов каждого предмета. Durability здесь намеренно не проверяется.
    pub(crate) fn apply_ci_qing_base_properties(&mut self, factory: &CGoodsFactory) {
        fn add_property(target: &mut u32, delta: i32) {
            let next = target.wrapping_add(delta as u32);
            *target = if delta < 0 && (next as i32) < 0 {
                0
            } else {
                next
            };
        }

        for position in 0..self.ci_qing.size() {
            let additions = {
                let Some(goods) = self.ci_qing.get_goods(position) else {
                    continue;
                };
                goods
                    .enabled_addon_properties(factory)
                    .into_iter()
                    .map(|property_type| {
                        (
                            property_type,
                            goods.addon_property_value(factory, property_type, 1),
                        )
                    })
                    .collect::<Vec<_>>()
            };
            for positive_pass in [true, false] {
                for &(property_type, delta) in &additions {
                    if (delta >= 0) != positive_pass {
                        continue;
                    }
                    match property_type {
                        GAP_BREAK_ARMOUR => {
                            add_property(&mut self.base_properties.break_armour, delta)
                        }
                        GAP_PUNCTURE => add_property(&mut self.base_properties.puncture, delta),
                        GAP_BREAK_ELEMENT => {
                            add_property(&mut self.base_properties.break_element, delta)
                        }
                        GAP_BREAK_BOUND => {
                            add_property(&mut self.base_properties.break_bound, delta)
                        }
                        GAP_GOLD_POWER => {
                            add_property(&mut self.base_properties.power_of_gold, delta)
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Первый снимок `MountAllEquip`: обычная экипировка уже применена, а
    /// восемь CiQing-ячеек ещё нет. Он нужен для exact насыщенной разницы
    /// `UpdateCiQingProperty` и не создаёт теневого player state.
    pub(crate) fn recompute_without_ci_qing_properties(
        &self,
        coefficients: GlobePlayerPropertyCoefficients,
        base_combat_scales: [f32; 5],
        critical_rate: f32,
        goods_factory: &CGoodsFactory,
    ) -> PlayerCombatProperties {
        self.recompute_base_and_mounted_properties(
            coefficients,
            base_combat_scales,
            critical_rate,
            goods_factory,
            false,
        )
    }

    fn recompute_base_and_mounted_properties(
        &self,
        coefficients: GlobePlayerPropertyCoefficients,
        base_combat_scales: [f32; 5],
        critical_rate: f32,
        goods_factory: &CGoodsFactory,
        include_ci_qing: bool,
    ) -> PlayerCombatProperties {
        let occupation = usize::from(self.base_properties.occupation).min(2);
        let base_u16 = |offset| read_player_wire_u16(&self.base_property_wire, offset);
        let base_u32 = |offset| read_player_wire_u32(&self.base_property_wire, offset);
        let derived = |value: u32, coefficient: f32| {
            ((value as f32) * coefficient).trunc() as u32
        };
        let mut properties = PlayerCombatProperties {
            maximum_hp: self
                .base_properties
                .base_maximum_hp
                .wrapping_add(derived(
                    self.base_properties.base_constitution,
                    coefficients.con_to_max_hp[occupation],
                )),
            maximum_mp: self
                .base_properties
                .base_maximum_mp
                .wrapping_add(derived(
                    self.base_properties.base_intelligence,
                    coefficients.int_to_max_mp[occupation],
                )),
            maximum_yp: self.base_properties.maximum_yp,
            maximum_rp: self.base_properties.maximum_rp,
            strength: self.base_properties.base_strength,
            dexterity: self.base_properties.base_dexterity,
            constitution: self.base_properties.base_constitution,
            intelligence: self.base_properties.base_intelligence,
            minimum_attack: base_u32(BASE_MINIMUM_ATTACK_OFFSET).wrapping_add(derived(
                self.base_properties.base_dexterity,
                coefficients.dex_to_min_attack[occupation],
            )),
            maximum_attack: base_u32(BASE_MAXIMUM_ATTACK_OFFSET).wrapping_add(derived(
                self.base_properties.base_strength,
                coefficients.str_to_max_attack[occupation],
            )),
            attack_speed: base_u16(BASE_ATTACK_SPEED_OFFSET),
            hit: base_u16(BASE_HIT_OFFSET),
            dodge: base_u16(BASE_DODGE_OFFSET),
            cch: base_u16(BASE_CCH_OFFSET),
            defense: base_u32(BASE_DEFENSE_OFFSET).wrapping_add(derived(
                self.base_properties.base_constitution,
                coefficients.con_to_defense[occupation],
            )),
            element_resistance: base_u32(BASE_ELEMENT_RESISTANCE_OFFSET).wrapping_add(derived(
                self.base_properties.base_intelligence,
                coefficients.int_to_resistant[occupation],
            )),
            hp_recovery: base_u16(BASE_HP_RECOVERY_OFFSET),
            mp_recovery: base_u16(BASE_MP_RECOVERY_OFFSET),
            burden: self.base_properties.base_burden.wrapping_add(
                derived(
                    self.base_properties.base_strength,
                    coefficients.str_to_burden[occupation],
                ) as u16,
            ),
            reank: derived(
                self.base_properties.base_dexterity,
                coefficients.dex_to_stiff[occupation],
            ) as u16,
            element_modify: derived(
                self.base_properties.base_intelligence,
                coefficients.int_to_element[occupation],
            ) as i32,
            blast_attack_scale_bits: base_combat_scales[0].to_bits(),
            blast_defense_scale_bits: base_combat_scales[1].to_bits(),
            element_blast_attack_scale_bits: base_combat_scales[2].to_bits(),
            element_blast_defense_scale_bits: base_combat_scales[3].to_bits(),
            full_miss_scale_bits: base_combat_scales[4].to_bits(),
            critical_rate_bits: critical_rate.to_bits(),
            resume_hp_peace: coefficients.resume_hp_peace,
            resume_mp_peace: coefficients.resume_mp_peace,
            resume_hp_fight: coefficients.resume_hp_fight,
            resume_mp_fight: coefficients.resume_mp_fight,
            restored_hp_peace: coefficients.restored_hp_peace,
            restored_mp_peace: coefficients.restored_mp_peace,
            restored_hp_fight: coefficients.restored_hp_fight,
            restored_mp_fight: coefficients.restored_mp_fight,
            battle_fairy_summoned: self.battle_fairy_summoned,
            battle_fairy_recall: self.base_properties.battle_fairy_recall,
            battle_fairy_died: self.base_properties.battle_fairy_died,
            ..PlayerCombatProperties::default()
        };
        let mut active_levels = BTreeMap::<i32, u8>::new();
        for (_, goods) in self.equipment.traversing_goods() {
            if goods.query_attribute(GAP_GOODS_MAXIMUM_DURABILITY)
                && goods.addon_property_value(goods_factory, GAP_GOODS_MAXIMUM_DURABILITY, 2) < 1
            {
                continue;
            }
            if !goods.has_addon_property_values(goods_factory, GAP_ANIMA_BIND)
                || goods.addon_property_value(goods_factory, GAP_ANIMA_BIND, 1) == 0
            {
                continue;
            }
            let key = goods.addon_property_value(goods_factory, GAP_ANIMA_BIND, 2);
            let mut level = goods
                .addon_property_value(goods_factory, GAP_ROLE_MINIMUM_LEVEL_LIMIT, 1)
                as u8;
            if level > 99 {
                level = level.wrapping_add(10);
            }
            active_levels
                .entry(key)
                .and_modify(|current| *current = (*current).max(level))
                .or_insert(level);
        }
        for (column, goods) in self.equipment.traversing_goods() {
            apply_equipment_goods_properties(
                &mut properties,
                goods,
                goods_factory,
                coefficients,
                occupation,
                true,
                active_levels.get(&(column.position() as i32)).copied(),
            );
        }
        if include_ci_qing {
            for position in 0..self.ci_qing.size() {
                let Some(goods) = self.ci_qing.get_goods(position) else {
                    continue;
                };
                apply_equipment_goods_properties(
                    &mut properties,
                    goods,
                    goods_factory,
                    coefficients,
                    occupation,
                    true,
                    active_levels.get(&(position as i32)).copied(),
                );
            }
        }
        properties
    }

    /// Мутирующая prelude исходного `CPlayer::UpdateProperty`: slot 10
    /// пересчитывает производные battle-fairy addon-ы до `MountAllEquip`.
    /// Значения `2` остаются instance-modifier-ами, а текущие HP/MP здесь
    /// намеренно не зажимаются — native owner обновляет только максимумы.
    /// Шесть FISTP-конверсий усекают полную сумму к нулю.
    pub(crate) fn refresh_battle_fairy_equipment_properties(
        &mut self,
        factory: &CGoodsFactory,
    ) {
        let Some(goods) = self.equipment.get_goods_mut(10) else {
            return;
        };
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) == 0 {
            return;
        }
        let level = goods.addon_property_value(factory, GAP_BF_LEVEL, 1);
        let pullulate = goods.addon_property_value(factory, GAP_BF_PULLULATERATE, 1) as f32;
        let growth = (level.wrapping_sub(1) as f32) * pullulate * 0.0001_f32 + 1.0_f32;
        for (value_property, base_property, potential_property) in [
            (GAP_BF_BRAVE, GAP_BF_BRAVE_BASE, GAP_BF_BRAVE_POTENTIAL),
            (
                GAP_BF_AGILITY,
                GAP_BF_AGILITY_BASE,
                GAP_BF_AGILITY_POTENTIAL,
            ),
            (
                GAP_BF_SPRITUALISM,
                GAP_BF_SPRITUALISM_BASE,
                GAP_BF_SPRITUALISM_POTENTIAL,
            ),
            (
                GAP_BF_STRENGH,
                GAP_BF_STRENGH_BASE,
                GAP_BF_STRENGH_POTENTIAL,
            ),
            (
                GAP_BF_ATTACK,
                GAP_BF_ATTACK_BASE,
                GAP_BF_ATTACK_POTENTIAL,
            ),
            (
                GAP_BF_SPRITE,
                GAP_BF_SPRITE_BASE,
                GAP_BF_SPRITE_POTENTIAL,
            ),
        ] {
            let base = goods.addon_property_value(factory, base_property, 1) as f32;
            let potential = goods.addon_property_value(factory, potential_property, 1) as f32;
            let modifier = goods.addon_property_value(factory, value_property, 2) as f32;
            let value = (modifier + potential + base * growth).trunc() as i32;
            let _ = goods.set_addon_property_value_core(value_property, 1, value);
        }
        let blast = goods
            .addon_property_value(factory, GAP_BF_BLAST_POTENTIAL, 1)
            .wrapping_add(goods.addon_property_value(factory, GAP_BF_BLAST, 2));
        let _ = goods.set_addon_property_value_core(GAP_BF_BLAST, 1, blast);
        let maximum_hp = goods.addon_property_value(factory, GAP_BF_STRENGH, 1);
        let maximum_mp = goods.addon_property_value(factory, GAP_BF_SPRITUALISM, 1);
        let _ = goods.set_addon_property_value_core(GAP_BF_MAX_HP, 1, maximum_hp);
        let _ = goods.set_addon_property_value_core(GAP_BF_MAX_MP, 1, maximum_mp);
        let _ = goods.set_instance_addon_modifier(GAP_BF_CUT_HURT_SCALE, 1, 0);
    }

    /// `MountEquip` cases `0x9B/0x9C/0x9E..0xA1` после battle-fairy prelude.
    /// Текущие BF-атрибуты хранятся в масштабе 1/10000; нулевое производное
    /// значение восстанавливает base addon-ы, как native positive pass. Все
    /// FISTP-преобразования используют truncate полной суммы с live property.
    pub(crate) fn apply_battle_fairy_equipment_properties(
        &mut self,
        mut properties: PlayerCombatProperties,
        coefficients: GlobePlayerPropertyCoefficients,
        factory: &CGoodsFactory,
    ) -> PlayerCombatProperties {
        fn add_u32(target: &mut u32, delta: i64) {
            *target = (i64::from(*target) + delta).clamp(0, i64::from(i32::MAX)) as u32;
        }
        fn add_scaled_u32(target: &mut u32, delta: f64) {
            let value = (f64::from(*target) + delta).trunc() as i64;
            *target = value.clamp(0, i64::from(i32::MAX)) as u32;
        }
        fn add_scaled_i32(target: &mut i32, delta: f64) {
            let value = (f64::from(*target) + delta).trunc() as i64;
            *target = value.clamp(0, i64::from(i32::MAX)) as i32;
        }
        fn add_scaled_u16(target: &mut u16, delta: f64) {
            let value = (f64::from(*target) + delta).trunc() as i64;
            *target = if value < 0 { 0 } else { value as u16 };
        }
        fn truncated_product(value: i32, coefficient: f32) -> i32 {
            (f64::from(value) * f64::from(coefficient)).trunc() as i32
        }
        fn scaled(value: i32) -> f64 {
            f64::from(value) * 0.0001_f64
        }

        let occupation = usize::from(self.base_properties.occupation).min(2);
        let Some(goods) = self.equipment.get_goods_mut(10) else {
            return properties;
        };
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) == 0 {
            return properties;
        }
        let enabled = goods.enabled_addon_properties(factory);
        for property in enabled {
            match property {
                GAP_BF_ATTACK => {
                    if goods.addon_property_value(factory, GAP_BF_ATTACK, 1) == 0 {
                        let base = goods.addon_property_value(factory, GAP_BF_ATTACK_BASE, 1);
                        let _ = goods.set_addon_property_value_core(GAP_BF_ATTACK, 1, base);
                    }
                }
                GAP_BF_SPRITE => {
                    if goods.addon_property_value(factory, GAP_BF_SPRITE, 1) == 0 {
                        let base = goods.addon_property_value(factory, GAP_BF_SPRITE_BASE, 1);
                        let _ = goods.set_addon_property_value_core(GAP_BF_SPRITE, 1, base);
                    }
                }
                GAP_BF_BRAVE => {
                    let base = goods.addon_property_value(factory, GAP_BF_BRAVE_BASE, 1);
                    let current = goods.addon_property_value(factory, GAP_BF_BRAVE, 1);
                    let base_effect =
                        truncated_product(base, coefficients.battle_fairy_brave_to_player);
                    let current_effect =
                        truncated_product(current, coefficients.battle_fairy_brave_to_player);
                    if current_effect == 0 {
                        let _ = goods.set_addon_property_value_core(GAP_BF_BRAVE, 1, base);
                        add_u32(&mut properties.strength, i64::from(base_effect));
                        continue;
                    }
                    if goods.addon_property_value(factory, GAP_BF_HP, 1) == 0 {
                        continue;
                    }
                    let effect = scaled(current_effect);
                    add_scaled_u32(&mut properties.strength, effect);
                    add_scaled_u32(
                        &mut properties.maximum_attack,
                        effect * f64::from(coefficients.str_to_max_attack[occupation]),
                    );
                    add_scaled_u16(
                        &mut properties.burden,
                        effect * f64::from(coefficients.str_to_burden[occupation]),
                    );
                }
                GAP_BF_AGILITY => {
                    let base = goods.addon_property_value(factory, GAP_BF_AGILITY_BASE, 1);
                    let current = goods.addon_property_value(factory, GAP_BF_AGILITY, 1);
                    let base_effect =
                        truncated_product(base, coefficients.battle_fairy_agility_to_player);
                    let current_effect =
                        truncated_product(current, coefficients.battle_fairy_agility_to_player);
                    if current_effect == 0 {
                        let _ = goods.set_addon_property_value_core(GAP_BF_AGILITY, 1, base);
                        add_u32(&mut properties.dexterity, i64::from(base_effect));
                        continue;
                    }
                    if goods.addon_property_value(factory, GAP_BF_HP, 1) == 0 {
                        continue;
                    }
                    let effect = scaled(current_effect);
                    add_scaled_u32(&mut properties.dexterity, effect);
                    add_scaled_u32(
                        &mut properties.minimum_attack,
                        effect * f64::from(coefficients.dex_to_min_attack[occupation]),
                    );
                    add_scaled_u16(
                        &mut properties.reank,
                        effect * f64::from(coefficients.dex_to_stiff[occupation]),
                    );
                }
                GAP_BF_SPRITUALISM => {
                    let base = goods.addon_property_value(factory, GAP_BF_SPRITUALISM_BASE, 1);
                    let current = goods.addon_property_value(factory, GAP_BF_SPRITUALISM, 1);
                    let base_effect =
                        truncated_product(base, coefficients.battle_fairy_spiritualism_to_player);
                    let current_effect =
                        truncated_product(current, coefficients.battle_fairy_spiritualism_to_player);
                    if current_effect == 0 {
                        let _ = goods.set_addon_property_value_core(GAP_BF_SPRITUALISM, 1, base);
                        let _ = goods.set_addon_property_value_core(GAP_BF_MAX_MP, 1, base);
                        let _ = goods.set_addon_property_value_core(GAP_BF_MP, 1, base);
                        add_u32(&mut properties.intelligence, i64::from(base_effect));
                        continue;
                    }
                    if goods.addon_property_value(factory, GAP_BF_HP, 1) == 0 {
                        continue;
                    }
                    let effect = scaled(current_effect);
                    add_scaled_u32(&mut properties.intelligence, effect);
                    add_scaled_i32(
                        &mut properties.element_modify,
                        effect * f64::from(coefficients.int_to_element[occupation]),
                    );
                    add_scaled_u32(
                        &mut properties.maximum_mp,
                        effect * f64::from(coefficients.int_to_max_mp[occupation]),
                    );
                    add_scaled_u32(
                        &mut properties.element_resistance,
                        effect * f64::from(coefficients.int_to_resistant[occupation]),
                    );
                    clamp_battle_fairy_current(goods, factory, GAP_BF_MP, GAP_BF_MAX_MP);
                }
                GAP_BF_STRENGH => {
                    let base = goods.addon_property_value(factory, GAP_BF_STRENGH_BASE, 1);
                    let current = goods.addon_property_value(factory, GAP_BF_STRENGH, 1);
                    let base_effect =
                        truncated_product(base, coefficients.battle_fairy_strength_to_hp);
                    let current_effect =
                        truncated_product(current, coefficients.battle_fairy_strength_to_hp);
                    if current_effect == 0 {
                        let _ = goods.set_addon_property_value_core(GAP_BF_STRENGH, 1, base);
                        let _ = goods.set_addon_property_value_core(GAP_BF_MAX_HP, 1, base);
                        let _ = goods.set_addon_property_value_core(GAP_BF_HP, 1, base);
                        add_u32(&mut properties.maximum_hp, i64::from(base_effect));
                    } else if goods.addon_property_value(factory, GAP_BF_HP, 1) != 0 {
                        add_scaled_u32(&mut properties.maximum_hp, scaled(current_effect));
                    } else {
                        continue;
                    }
                    clamp_battle_fairy_current(goods, factory, GAP_BF_HP, GAP_BF_MAX_HP);
                }
                _ => {}
            }
        }
        properties
    }

    /// Два снимка `UpdateCiQingProperty` должны видеть один и тот же
    /// pre-Mount BF goods state. Первый property-pass выполняется на clone,
    /// второй оставляет canonical zero-init/clamp mutations в slot 10.
    pub(crate) fn apply_battle_fairy_equipment_property_pair(
        &mut self,
        previous: PlayerCombatProperties,
        current: PlayerCombatProperties,
        coefficients: GlobePlayerPropertyCoefficients,
        factory: &CGoodsFactory,
    ) -> (PlayerCombatProperties, PlayerCombatProperties) {
        let saved = self.equipment.get_goods(10).cloned();
        let previous = self.apply_battle_fairy_equipment_properties(
            previous,
            coefficients,
            factory,
        );
        if let (Some(saved), Some(goods)) = (saved, self.equipment.get_goods_mut(10)) {
            *goods = saved;
        }
        let current = self.apply_battle_fairy_equipment_properties(
            current,
            coefficients,
            factory,
        );
        (previous, current)
    }

    /// Полный `MountAllEquip` строит два снимка из одного BF goods state,
    /// заменяет `m_mapCiQingAddValue` разностью и передаёт caller-у итог для
    /// обязательного `SendResultToClient` до `OnChangeProperties`.
    pub(crate) fn recompute_update_property(
        &mut self,
        coefficients: GlobePlayerPropertyCoefficients,
        base_combat_scales: [f32; 5],
        critical_rate: f32,
        factory: &CGoodsFactory,
    ) -> PlayerPropertyRecompute {
        self.equipment_changed = true;
        self.refresh_battle_fairy_equipment_properties(factory);
        self.apply_ci_qing_base_properties(factory);
        let previous = self.recompute_without_ci_qing_properties(
            coefficients,
            base_combat_scales,
            critical_rate,
            factory,
        );
        let current = self.recompute_base_and_equipment_properties(
            coefficients,
            base_combat_scales,
            critical_rate,
            factory,
        );
        let (previous, current) = self.apply_battle_fairy_equipment_property_pair(
            previous,
            current,
            coefficients,
            factory,
        );
        if let Some(values) = Self::update_ci_qing_property_difference(
            &Self::combat_type_values_from(previous),
            &Self::combat_type_values_from(current),
        ) {
            self.ci_qing_add_values = values;
        }
        PlayerPropertyRecompute {
            properties: current,
            ci_qing_result_values: self.ci_qing_property_result(),
        }
    }


    pub(crate) fn replace_mana_shield_state(
        &mut self,
        state: super::skills::manashieldstate::ManaShieldState,
    ) -> Option<super::skills::manashieldstate::ManaShieldState> {
        self.move_shape.replace_mana_shield_state(state)
    }

    pub(crate) fn replace_machine_shield_state(
        &mut self,
        state: super::skills::machineshieldstate::MachineShieldState,
    ) -> Option<super::skills::machineshieldstate::MachineShieldState> {
        self.move_shape.replace_machine_shield_state(state)
    }

    pub(crate) fn replace_life_shield_state(
        &mut self,
        state: super::skills::lifeshieldstate::LifeShieldState,
    ) -> Option<super::skills::lifeshieldstate::LifeShieldState> {
        self.move_shape.replace_life_shield_state(state)
    }

    pub(crate) fn defense_shields(
        &self,
    ) -> impl Iterator<Item = &super::skills::shieldstate::DefenseShieldState> {
        self.move_shape.defense_shields()
    }

    pub(crate) fn defense_shield_key(&self, skill_id: u32) -> Option<super::moveshape::StateKey> {
        self.move_shape.defense_shield_key(skill_id)
    }

    pub(crate) fn defense_shield(
        &self,
        key: super::moveshape::StateKey,
    ) -> Option<&super::skills::shieldstate::DefenseShieldState> {
        self.move_shape.defense_shield(key)
    }

    pub(crate) fn remove_defense_shield_key(
        &mut self,
        key: super::moveshape::StateKey,
    ) -> Option<super::skills::shieldstate::DefenseShieldState> {
        self.move_shape.remove_defense_shield_key(key)
    }

    pub(crate) fn remove_defense_shield(
        &mut self,
        skill_id: u32,
    ) -> Option<super::skills::shieldstate::DefenseShieldState> {
        self.move_shape.remove_defense_shield(skill_id)
    }


    pub(crate) fn take_defense_shields(
        &mut self,
    ) -> super::moveshape::StateBatch<super::skills::shieldstate::DefenseShieldState> {
        self.move_shape.take_defense_shields()
    }

    pub(crate) fn restore_defense_shields(
        &mut self,
        states: super::moveshape::StateBatch<super::skills::shieldstate::DefenseShieldState>,
    ) {
        self.move_shape.restore_defense_shields(states);
    }

    pub(crate) fn cure_state(&self) -> Option<super::skills::curestate::CureState> {
        self.move_shape.cure_state()
    }

    pub(crate) fn take_cure_state(&mut self) -> Option<super::skills::curestate::CureState> {
        self.move_shape.take_cure_state()
    }


    pub(crate) fn push_cure_state(&mut self, state: super::skills::curestate::CureState) {
        self.move_shape.push_cure_state(state);
    }

    pub(crate) fn replace_daub_poison_state(
        &mut self,
        state: super::skills::daubpoisonstate::DaubPoisonState,
    ) -> Option<super::skills::daubpoisonstate::DaubPoisonState> {
        self.move_shape.replace_daub_poison_state(state)
    }


    pub(crate) fn curable_state_ids(&self) -> Vec<u32> {
        self.move_shape.curable_state_ids()
    }

    pub(crate) fn replace_poison_arrow_state(
        &mut self,
        state: super::skills::poisonarrowstate::PoisonArrowState,
    ) -> Option<super::skills::poisonarrowstate::PoisonArrowState> {
        self.move_shape.replace_poison_arrow_state(state)
    }

    pub(crate) fn take_expired_poison_fog_state(&mut self, key: super::moveshape::StateKey, now_ms: u32) -> Option<super::skills::poisonfogstate::PoisonFogState> { self.move_shape.take_expired_poison_fog_state(key, now_ms) }
    pub(crate) fn take_poison_fog_state(&mut self) -> Option<super::skills::poisonfogstate::PoisonFogState> { self.move_shape.take_poison_fog_state() }
    pub(crate) fn meteor_arrow_state(&self) -> Option<super::skills::meteorarrowstate::MeteorArrowState> { self.move_shape.meteor_arrow_state() }
    pub(crate) fn add_meteor_arrows(&mut self, maximum: u32, amount: u32) -> Option<super::skills::meteorarrowstate::MeteorArrowState> { self.move_shape.add_meteor_arrows(maximum, amount) }
    pub(crate) fn take_meteor_arrow_state(&mut self) -> Option<super::skills::meteorarrowstate::MeteorArrowState> { self.move_shape.take_meteor_arrow_state() }






    pub(crate) fn replace_spider_poison_state(
        &mut self,
        state: super::skills::spiderpoisonstate::SpiderPoisonState,
    ) -> Option<super::skills::spiderpoisonstate::SpiderPoisonState> {
        self.move_shape.replace_spider_poison_state(state)
    }





    pub(crate) fn take_spider_poison_state(
        &mut self,
    ) -> Option<super::skills::spiderpoisonstate::SpiderPoisonState> {
        self.move_shape.take_spider_poison_state()
    }

    pub(crate) fn replace_sprite_burn_state(
        &mut self,
        state: super::skills::spriteburnstate::SpriteBurnState,
    ) -> Option<super::skills::spriteburnstate::SpriteBurnState> {
        self.move_shape.replace_sprite_burn_state(state)
    }





    pub(crate) fn take_sprite_burn_state(
        &mut self,
    ) -> Option<super::skills::spriteburnstate::SpriteBurnState> {
        self.move_shape.take_sprite_burn_state()
    }

    pub(crate) fn replace_spider_web_state(
        &mut self,
        state: super::skills::spiderwebstate::SpiderWebState,
    ) -> Option<super::skills::spiderwebstate::SpiderWebState> {
        self.move_shape.replace_spider_web_state(state)
    }


    pub(crate) fn replace_weak_state(
        &mut self,
        state: super::skills::weakstate::WeakState,
    ) -> Option<super::skills::weakstate::WeakState> {
        self.move_shape.replace_weak_state(state)
    }


    pub(crate) fn replace_roar_state(&mut self, state: super::skills::roarstate::RoarState) -> Option<super::skills::roarstate::RoarState> { self.move_shape.replace_roar_state(state) }

    pub(crate) fn energy_holding_state(&self) -> Option<super::skills::energyholdingstate::EnergyHoldingState> { self.move_shape.energy_holding_state() }
    pub(crate) fn energy_holding_state_mut(&mut self) -> Option<&mut super::skills::energyholdingstate::EnergyHoldingState> { self.move_shape.energy_holding_state_mut() }
    pub(crate) fn begin_energy_holding_state(&mut self, state: super::skills::energyholdingstate::EnergyHoldingState) { self.move_shape.begin_energy_holding_state(state); }
    pub(crate) fn take_energy_holding_state(&mut self) -> Option<super::skills::energyholdingstate::EnergyHoldingState> { self.move_shape.take_energy_holding_state() }

    pub(crate) fn take_boss_blue_fury_state(
        &mut self,
    ) -> Option<super::skills::bossbluefurystate::BossBlueFuryState> {
        self.move_shape.take_boss_blue_fury_state()
    }

    pub(crate) fn begin_boss_blue_fury_state(
        &mut self,
        state: super::skills::bossbluefurystate::BossBlueFuryState,
    ) {
        self.move_shape.begin_boss_blue_fury_state(state);
    }




    pub(crate) fn soul_collect_state(&self) -> Option<super::skills::soulcollectstate::SoulCollectState> {
        self.move_shape.soul_collect_state()
    }

    pub(crate) fn soul_collect_state_mut(&mut self) -> Option<&mut super::skills::soulcollectstate::SoulCollectState> {
        self.move_shape.soul_collect_state_mut()
    }

    pub(crate) fn begin_soul_collect_state(&mut self, state: super::skills::soulcollectstate::SoulCollectState) {
        self.move_shape.begin_soul_collect_state(state);
    }

    pub(crate) fn take_soul_collect_state(&mut self) -> Option<super::skills::soulcollectstate::SoulCollectState> {
        self.move_shape.take_soul_collect_state()
    }

    pub(crate) fn take_weak_state(&mut self) -> Option<super::skills::weakstate::WeakState> {
        self.move_shape.take_weak_state()
    }

    pub(crate) fn take_weak_state_outside(
        &mut self,
        tile_x: i32,
        tile_y: i32,
    ) -> Option<super::skills::weakstate::WeakState> {
        self.move_shape.take_weak_state_outside(tile_x, tile_y)
    }


    pub(crate) fn take_spider_web_state(
        &mut self,
    ) -> Option<super::skills::spiderwebstate::SpiderWebState> {
        self.move_shape.take_spider_web_state()
    }

    pub(crate) fn replace_knock_out_state(
        &mut self,
        state: super::skills::knockoutstate::KnockOutState,
    ) -> Option<super::skills::knockoutstate::KnockOutState> {
        self.move_shape.replace_knock_out_state(state)
    }







    pub(crate) fn take_blind_state(
        &mut self,
    ) -> Option<super::skills::blindstate::BlindState> {
        self.move_shape.take_blind_state()
    }

    pub(crate) fn replace_boa_lock_state(&mut self, state: super::skills::boalockstate::BoaLockState) -> Option<super::skills::boalockstate::BoaLockState> { self.move_shape.replace_boa_lock_state(state) }
    pub(crate) fn take_expired_boa_lock_state(&mut self, key: super::moveshape::StateKey, now_ms: u32) -> Option<super::skills::boalockstate::BoaLockState> { self.move_shape.take_expired_boa_lock_state(key, now_ms) }
    pub(crate) fn take_boa_lock_state(&mut self) -> Option<super::skills::boalockstate::BoaLockState> { self.move_shape.take_boa_lock_state() }

    pub(crate) fn pillar_state(&self) -> Option<super::skills::pillarstate::PillarState> {
        self.move_shape.pillar_state()
    }

    pub(crate) fn replace_rush_state(
        &mut self,
        state: super::skills::rushstate::RushState,
    ) -> Option<super::skills::rushstate::RushState> {
        self.move_shape.replace_rush_state(state)
    }


    pub(crate) fn take_expired_rush_state(
        &mut self,
        key: super::moveshape::StateKey,
        now_ms: u32,
    ) -> Option<super::skills::rushstate::RushState> {
        self.move_shape.take_expired_rush_state(key, now_ms)
    }

    pub(crate) fn take_rush_state(&mut self) -> Option<super::skills::rushstate::RushState> {
        self.move_shape.take_rush_state()
    }

    pub(crate) fn replace_rush_2_state(
        &mut self,
        state: super::skills::rushstate2::Rush2State,
    ) -> Option<super::skills::rushstate2::Rush2State> {
        self.move_shape.replace_rush_2_state(state)
    }


    pub(crate) fn take_expired_rush_2_state(
        &mut self,
        key: super::moveshape::StateKey,
        now_ms: u32,
    ) -> Option<super::skills::rushstate2::Rush2State> {
        self.move_shape.take_expired_rush_2_state(key, now_ms)
    }

    pub(crate) fn take_rush_2_state(&mut self) -> Option<super::skills::rushstate2::Rush2State> {
        self.move_shape.take_rush_2_state()
    }

    pub(crate) fn replace_pillar_state(
        &mut self, state: super::skills::pillarstate::PillarState,
    ) -> Option<super::skills::pillarstate::PillarState> {
        self.move_shape.replace_pillar_state(state)
    }




    pub(crate) fn take_pillar_state(&mut self) -> Option<super::skills::pillarstate::PillarState> {
        self.move_shape.take_pillar_state()
    }


    pub(crate) fn take_knock_out_state(
        &mut self,
    ) -> Option<super::skills::knockoutstate::KnockOutState> {
        self.move_shape.take_knock_out_state()
    }

    pub(crate) fn replace_knight_cut_state(
        &mut self,
        state: super::skills::knightcutstate::KnightCutState,
    ) -> Option<super::skills::knightcutstate::KnightCutState> {
        self.move_shape.replace_knight_cut_state(state)
    }


    pub(crate) fn take_expired_knight_cut_state(
        &mut self,
        key: super::moveshape::StateKey,
        now_ms: u32,
    ) -> Option<super::skills::knightcutstate::KnightCutState> {
        self.move_shape.take_expired_knight_cut_state(key, now_ms)
    }

    pub(crate) fn take_knight_cut_state(
        &mut self,
    ) -> Option<super::skills::knightcutstate::KnightCutState> {
        self.move_shape.take_knight_cut_state()
    }

    pub(crate) fn blind_state_order(&self) -> Vec<u32> {
        self.move_shape.blind_state_order()
    }

    pub(crate) fn replace_blood_loss_state(
        &mut self,
        state: super::skills::bloodlossstate::BloodLossState,
    ) -> Option<super::skills::bloodlossstate::BloodLossState> {
        self.move_shape.replace_blood_loss_state(state)
    }

    pub(crate) fn replace_leaf_cut_state(
        &mut self,
        state: super::skills::leafcutstate::LeafCutState,
        now_ms: u32,
    ) -> Option<super::skills::leafcutstate::LeafCutState> {
        self.move_shape.replace_leaf_cut_state(state, now_ms)
    }








    pub(crate) fn replace_leaf_cut_2_state(
        &mut self,
        state: super::skills::leafcutstate2::LeafCutState2,
    ) -> Option<super::skills::leafcutstate2::LeafCutState2> {
        self.move_shape.replace_leaf_cut_2_state(state)
    }











    pub(crate) fn replace_leaf_cut_3_state(
        &mut self,
        state: super::skills::leafcutstate3::LeafCutState3,
        now_ms: u32,
    ) -> Option<super::skills::leafcutstate3::LeafCutState3> {
        self.move_shape.replace_leaf_cut_3_state(state, now_ms)
    }







    pub(crate) fn replace_kerosene_state(&mut self, state: super::skills::kerosenestate::KeroseneState, now_ms: u32) -> Option<super::skills::kerosenestate::KeroseneState> { self.move_shape.replace_kerosene_state(state, now_ms) }


    pub(crate) fn take_kerosene_state(&mut self) -> Option<super::skills::kerosenestate::KeroseneState> { self.move_shape.take_kerosene_state() }



    pub(crate) fn take_expired_battle_fairy_attribute_state(&mut self, key: crate::gameserver::appserver::moveshape::StateKey, now_ms: u32) -> Option<super::skills::battlefairyattributestate::BattleFairyAttributeState> {
        self.move_shape.take_expired_battle_fairy_attribute_state(key, now_ms)
    }








    pub(crate) fn script_move_state_count(&self, state_id: i32) -> u32 {
        self.move_shape.state_count_by_state_id(state_id)
    }








    pub(crate) const fn is_auto_protected(&self) -> bool {
        self.auto_protected
    }

    pub(crate) const fn set_auto_protected(&mut self, value: bool) {
        self.auto_protected = value;
    }

    pub(crate) fn improve_experience_multiplier(&self) -> f64 {
        self.move_shape
            .script_states()
            .fold(1.0_f64, |multiplier, state| {
                multiplier + state.experience_multiplier_delta()
            })
    }

    pub(crate) const fn fight_state_count(&self) -> i32 {
        self.fight_state_count
    }

    /// Exact `EnterPeaceState` state half: virtual `CShape::SetState(0)` and
    /// fight countdown reset precede the caller-owned around publication.
    pub(crate) fn enter_peace_state(&mut self) -> PlayerFightStateTransition {
        let previous_count = self.fight_state_count;
        self.movement_shape_mut().set_state(0);
        self.fight_state_count = 0;
        PlayerFightStateTransition {
            player_id: self.player_id(),
            previous_count,
            current_count: 0,
            entered_peace: true,
        }
    }

    /// Exact `OnBeginSkill -> EnterCombatState`: base defense — единственное
    /// исключение; concrete skill caller уже отфильтровал его. Countdown
    /// хранится в simulation frames (`g_ms == 80`), around `0xBF607` остаётся
    /// у CGame рядом с live region transport.
    pub(crate) fn enter_combat_state(
        &mut self,
        fight_state_timer_ms: i32,
    ) -> PlayerFightStateTransition {
        let previous_count = self.fight_state_count;
        self.movement_shape_mut().set_state(1);
        self.fight_state_count = fight_state_timer_ms / 80;
        PlayerFightStateTransition {
            player_id: self.player_id(),
            previous_count,
            current_count: self.fight_state_count,
            entered_peace: false,
        }
    }

    pub(crate) const fn can_fight(&self) -> bool {
        self.move_shape.can_fight()
    }

    /// Reached `UpdateCurrentState` combat half. Только положительный counter
    /// декрементируется; переход через zero исполняет exact peace mutation.
    pub(crate) fn update_fight_state(&mut self) -> Option<PlayerFightStateTransition> {
        if self.fight_state_count <= 0 {
            return None;
        }
        let previous_count = self.fight_state_count;
        self.fight_state_count = self.fight_state_count.wrapping_sub(1);
        if self.fight_state_count < 1 {
            self.movement_shape_mut().set_state(0);
            self.fight_state_count = 0;
            return Some(PlayerFightStateTransition {
                player_id: self.player_id(),
                previous_count,
                current_count: 0,
                entered_peace: true,
            });
        }
        Some(PlayerFightStateTransition {
            player_id: self.player_id(),
            previous_count,
            current_count: self.fight_state_count,
            entered_peace: false,
        })
    }

    /// Exact delayed `OnLost` timestamp. Native formula deliberately keeps
    /// signed/wrapping intermediate arithmetic with fixed process `g_ms=80`.
    pub(crate) fn begin_lost_delay(
        &mut self,
        now_ms: u32,
        fight_state_timer_ms: i32,
    ) -> Option<PlayerLostDelayStarted> {
        if self.fight_state_count <= 0 {
            return None;
        }
        let remaining = self
            .fight_state_count
            .wrapping_mul(80)
            .wrapping_sub(fight_state_timer_ms);
        self.lost_time_stamp_ms = now_ms.wrapping_add(remaining as u32);
        Some(PlayerLostDelayStarted {
            player_id: self.player_id(),
            fight_state_count: self.fight_state_count,
            timestamp_ms: self.lost_time_stamp_ms,
        })
    }

    pub(crate) const fn lost_delay_due(&self, now_ms: u32, fight_state_timer_ms: i32) -> bool {
        self.lost_time_stamp_ms != 0
            && self
                .lost_time_stamp_ms
                .wrapping_add(fight_state_timer_ms as u32)
                <= now_ms
    }

    pub(crate) const fn has_lost_delay(&self) -> bool {
        self.lost_time_stamp_ms != 0
    }

    /// CRideState::AI (0x004F9110): listener сначала обходит весь packet
    /// с GAP_MOUNT_TYPE != 0, затем первый совпавший type/level завершает поиск.
    /// Имя, role limit, предыдущий GUID и локальный StateKey здесь не участвуют.
    pub(crate) fn has_ride_goods(
        &self,
        mount_type: u32,
        level: u32,
        factory: &CGoodsFactory,
    ) -> bool {
        let packet = self.packet.base();
        let mut listener = GoodsParticularPropertyListener::new(GAP_MOUNT_TYPE);
        for goods in packet.traversing_goods() {
            listener.visit(factory, goods);
        }
        listener.goods_ids().iter().any(|goods_id| {
            packet.find(*goods_id).is_some_and(|goods| {
                goods.addon_property_value(factory, GAP_MOUNT_TYPE, 1) as u32 == mount_type
                    && goods.addon_property_value(factory, GAP_MOUNT_LEVEL, 1) as u32 == level
            })
        })
    }

    fn ride_goods(
        &self,
        state: &super::ridestate::RideState,
        factory: &CGoodsFactory,
    ) -> Option<&CGoods> {
        let base_index = factory.query_goods_id_by_original_name(Some(state.goods_name()));
        self.packet
            .base()
            .traversing_goods()
            .find(|goods| goods.base_properties_index() == base_index)
    }

    /// `CNotDisappearAfterDead::OnUpdateProperties` применяет прямые поля до
    /// базовых характеристик, а отрицательные STR/DEX/CON/INT проецирует как
    /// разность производных старого и нового значения. IMUL сохраняет low32
    /// до unsigned /100; каждый native setter ограничивает DWORD INT_MAX.
    /// X87-преобразования усекают дробную часть; в отрицательных HP/MP/DEF-ветках сохраняется
    /// исходное знаковое WORD-сужение. Поле usage `20_001` соответствует
    /// `tagProperty.wHit +0x24`, а не соседнему `wAtcSpeed +0x32`.
    pub(crate) fn apply_undead_state_properties(
        &self,
        mut properties: PlayerCombatProperties,
        coefficients: GlobePlayerPropertyCoefficients,
        state: &super::moveshape::UndeadState,
    ) -> PlayerCombatProperties {
        fn clamp_wrapped_min_one(value: u32) -> u32 {
            if (value as i32) < 1 { 1 } else { value }
        }

        fn set_property(value: u32) -> u32 {
            value.min(i32::MAX as u32)
        }

        fn trunc_product(value: f64, coefficient: f32) -> i32 {
            super::skills::fightdefense::truncate_original(
                (value * f64::from(coefficient)).trunc(),
            )
        }

        fn ftol_word_product(value: f64, coefficient: f32) -> i16 {
            let value = (value * f64::from(coefficient)).trunc();
            // Только отрицательная INT→MP ветвь вызывает __ftol2 (FISTP64),
            // а затем использует AX. Integer-indefinite поэтому даёт WORD 0.
            if !value.is_finite()
                || value < -9_223_372_036_854_775_808.0
                || value >= 9_223_372_036_854_775_808.0
            {
                0
            } else {
                value as i64 as i16
            }
        }

        fn direct_value(current: u32, value: i16, percentage: bool) -> u32 {
            if value < 0 {
                let decrement = if percentage {
                    (current.wrapping_mul(value.wrapping_neg() as i32 as u32) / 100) as i16
                } else {
                    value.wrapping_neg()
                };
                let narrowed = (current as i16).wrapping_sub(decrement);
                if narrowed < 1 { 1 } else { narrowed as u32 }
            } else if value > 0 {
                let increment = if percentage {
                    current.wrapping_mul(value as u32) / 100
                } else {
                    value as u32
                };
                set_property(current.wrapping_add(increment))
            } else {
                current
            }
        }

        fn changed_stat(current: u32, value: i32, percentage: bool) -> (u32, u32, bool) {
            if value < 0 {
                let decrement = if percentage {
                    current.wrapping_mul(value.wrapping_neg() as u32) / 100
                } else {
                    value.wrapping_neg() as u32
                };
                let new = current.wrapping_sub(decrement);
                (clamp_wrapped_min_one(new), decrement, false)
            } else {
                let increment = if percentage {
                    current.wrapping_mul(value as u32) / 100
                } else {
                    value as u32
                };
                (set_property(current.wrapping_add(increment)), increment, true)
            }
        }

        fn add_positive_derived(current: u32, delta: f64, coefficient: f32) -> u32 {
            set_property(current.wrapping_add(trunc_product(delta, coefficient) as u32))
        }

        fn add_derived_difference(current: u32, old: f64, new: f64, coefficient: f32) -> u32 {
            let difference = trunc_product(new, coefficient)
                .wrapping_sub(trunc_product(old, coefficient));
            clamp_wrapped_min_one(current.wrapping_add(difference as u32))
        }

        fn add_short_derived_difference(
            current: u32,
            old: f64,
            new: f64,
            coefficient: f32,
        ) -> u32 {
            let difference = (trunc_product(new, coefficient) as i16)
                .wrapping_sub(trunc_product(old, coefficient) as i16);
            let narrowed = (current as i16).wrapping_add(difference);
            if narrowed < 1 { 1 } else { narrowed as u32 }
        }

        let occupation = usize::from(self.base_properties.occupation).min(2);
        properties.maximum_hp =
            direct_value(properties.maximum_hp, state.maximum_hp, state.percentage);
        properties.maximum_mp =
            direct_value(properties.maximum_mp, state.maximum_mp, state.percentage);
        properties.defense = direct_value(properties.defense, state.defense, state.percentage);
        properties.element_resistance = direct_value(
            properties.element_resistance,
            state.element_resistance,
            state.percentage,
        );

        if state.strength != 0 {
            let old = properties.strength;
            let (new, delta, positive) = changed_stat(old, state.strength, state.percentage);
            properties.strength = new;
            properties.maximum_attack = if positive {
                add_positive_derived(
                    properties.maximum_attack,
                    f64::from(delta),
                    coefficients.str_to_max_attack[occupation],
                )
            } else {
                add_derived_difference(
                    properties.maximum_attack,
                    f64::from(old),
                    f64::from(new),
                    coefficients.str_to_max_attack[occupation],
                )
            };
        }

        if state.dexterity != 0 {
            let old = properties.dexterity;
            let (new, delta, positive) = changed_stat(old, state.dexterity, state.percentage);
            properties.dexterity = new;
            properties.minimum_attack = if positive {
                add_positive_derived(
                    properties.minimum_attack,
                    f64::from(delta),
                    coefficients.dex_to_min_attack[occupation],
                )
            } else {
                add_derived_difference(
                    properties.minimum_attack,
                    f64::from(old),
                    f64::from(new),
                    coefficients.dex_to_min_attack[occupation],
                )
            };
        }

        if state.constitution != 0 {
            let old = properties.constitution;
            let (new, delta, positive) =
                changed_stat(old, state.constitution, state.percentage);
            properties.constitution = new;
            if positive {
                let projected = if state.percentage { f64::from(delta as f32) } else { f64::from(delta) };
                properties.maximum_hp = add_positive_derived(
                    properties.maximum_hp,
                    projected,
                    coefficients.con_to_max_hp[occupation],
                );
                properties.defense = add_positive_derived(
                    properties.defense,
                    projected,
                    coefficients.con_to_defense[occupation],
                );
            } else {
                properties.maximum_hp = add_short_derived_difference(
                    properties.maximum_hp,
                    f64::from(old),
                    f64::from(new),
                    coefficients.con_to_max_hp[occupation],
                );
                properties.defense = add_short_derived_difference(
                    properties.defense,
                    f64::from(old as f32),
                    f64::from(new as f32),
                    coefficients.con_to_defense[occupation],
                );
            }
        }

        if state.intelligence != 0 {
            let old = properties.intelligence;
            let (new, delta, positive) =
                changed_stat(old, state.intelligence, state.percentage);
            properties.intelligence = new;
            if positive {
                // В абсолютной ветке оригинал повторно проецирует уже новое
                // полное INT; процентная ветка проецирует только приращение.
                let projected = if state.percentage { f64::from(delta as f32) } else { f64::from(new) };
                properties.maximum_mp = add_positive_derived(
                    properties.maximum_mp,
                    projected,
                    coefficients.int_to_max_mp[occupation],
                );
                properties.element_resistance = add_positive_derived(
                    properties.element_resistance,
                    projected,
                    coefficients.int_to_resistant[occupation],
                );
                properties.element_modify = properties.element_modify.wrapping_add(
                    trunc_product(projected, coefficients.int_to_element[occupation]),
                );
            } else {
                let old_mp = ftol_word_product(f64::from(old), coefficients.int_to_max_mp[occupation]);
                let new_mp = ftol_word_product(f64::from(new as f32), coefficients.int_to_max_mp[occupation]);
                let maximum_mp = (properties.maximum_mp as i16)
                    .wrapping_add(new_mp.wrapping_sub(old_mp));
                properties.maximum_mp = maximum_mp.max(1) as u32;
                properties.element_resistance = add_derived_difference(
                    properties.element_resistance,
                    f64::from(old as f32),
                    f64::from(new as f32),
                    coefficients.int_to_resistant[occupation],
                );
                let difference = trunc_product(f64::from(new as f32), coefficients.int_to_element[occupation])
                    .wrapping_sub(trunc_product(
                        f64::from(old as f32),
                        coefficients.int_to_element[occupation],
                    ));
                properties.element_modify = properties.element_modify.wrapping_add(difference);
                if properties.element_modify < 1 {
                    properties.element_modify = 1;
                }
            }
        }

        properties.minimum_attack =
            direct_value(properties.minimum_attack, state.minimum_attack, false);
        properties.maximum_attack =
            direct_value(properties.maximum_attack, state.maximum_attack, false);
        if state.element_modify < 0 {
            let narrowed = (properties.element_modify as i16).wrapping_add(state.element_modify);
            properties.element_modify = if narrowed < 1 { 1 } else { i32::from(narrowed) };
        } else {
            properties.element_modify = properties
                .element_modify
                .wrapping_add(i32::from(state.element_modify));
        }
        properties.blast_attack = properties
            .blast_attack
            .wrapping_add(state.blast_attack as u16);
        properties.blast_element_attack = properties
            .blast_element_attack
            .wrapping_add(state.blast_element_attack as u16);
        properties.cch = properties.cch.wrapping_add(state.cch as u16);
        properties.full_miss = properties.full_miss.wrapping_add(state.full_miss as u16);
        properties.attack_avoid = properties
            .attack_avoid
            .wrapping_add(state.attack_avoid as u16);
        properties.element_avoid = properties
            .element_avoid
            .wrapping_add(state.element_avoid as u16);
        properties.hit = properties.hit.wrapping_add(state.hit as u16);
        properties.dodge = properties.dodge.wrapping_add(state.dodge as u16);
        properties
    }

    pub(crate) fn apply_extended_state_properties(
        mut properties: PlayerCombatProperties,
        state: &super::exstate::ExtendedState,
    ) -> PlayerCombatProperties {
        let add = |target: &mut u32, value: u16| {
            if value != 0 {
                *target = target.wrapping_add(u32::from(value)).min(i32::MAX as u32);
            }
        };
        add(&mut properties.maximum_hp, state.maximum_hp);
        add(&mut properties.maximum_mp, state.maximum_mp);
        add(&mut properties.minimum_attack, state.minimum_attack);
        add(&mut properties.maximum_attack, state.maximum_attack);
        add(&mut properties.defense, state.defense);
        add(&mut properties.element_resistance, state.element_resistance);
        properties.element_modify = properties
            .element_modify
            .wrapping_add(i32::from(state.element_modify));
        properties.cch = properties.cch.wrapping_add(state.cch);
        properties.full_miss = properties.full_miss.wrapping_add(state.full_miss);
        properties.attack_avoid = properties.attack_avoid.wrapping_add(state.attack_avoid);
        properties.element_avoid = properties.element_avoid.wrapping_add(state.element_avoid);
        properties.hit = properties.hit.wrapping_add(state.hit);
        properties.dodge = properties.dodge.wrapping_add(state.dodge);
        properties
    }

    pub(crate) fn apply_active_change_body_state_properties(
        mut properties: PlayerCombatProperties,
        state: &super::chbystate::ChangeBodyState,
    ) -> PlayerCombatProperties {
        let add = |target: &mut u32, value: u32| {
            if value != 0 {
                *target = target.wrapping_add(value).min(i32::MAX as u32);
            }
        };
        add(&mut properties.maximum_hp, state.maximum_hp);
        add(&mut properties.maximum_mp, state.maximum_mp);
        add(&mut properties.minimum_attack, state.minimum_attack);
        add(&mut properties.maximum_attack, state.maximum_attack);
        add(&mut properties.defense, state.defense);
        add(&mut properties.element_resistance, state.element_resistance);
        if state.cch != 0 {
            properties.cch = properties.cch.wrapping_add(state.cch);
        }
        if state.blast_attack != 0 {
            properties.blast_attack = properties.blast_attack.wrapping_add(state.blast_attack);
        }
        if state.blast_element_attack != 0 {
            properties.blast_element_attack = properties
                .blast_element_attack
                .wrapping_add(state.blast_element_attack);
        }
        properties
    }

    /// CRideState::OnUpdateProperties0x004F9000 выбирает первый packet goods
    /// по имени именно этого экземпляра, не по AI-cache. Общий расчёт делает
    /// MountEquipRide(true), затем(false), не копируя state или CGoods.
    pub(crate) fn apply_ride_state_properties(
        &self,
        mut properties: PlayerCombatProperties,
        state: &super::ridestate::RideState,
        coefficients: GlobePlayerPropertyCoefficients,
        goods_factory: &CGoodsFactory,
    ) -> PlayerCombatProperties {
        if let Some(goods) = self.ride_goods(state, goods_factory) {
            apply_equipment_goods_properties(
                &mut properties,
                goods,
                goods_factory,
                coefficients,
                usize::from(self.base_properties.occupation).min(2),
                false,
                None,
            );
        }
        properties
    }


    pub(crate) const fn remain_jing_li_dan_count(&self) -> u16 {
        self.base_properties.remain_jing_li_dan_count
    }

    pub(crate) const fn set_remain_jing_li_dan_count(&mut self, count: u16) {
        self.base_properties.remain_jing_li_dan_count = count;
    }

    pub(crate) fn get_extended_state(
        &self,
        kind: super::exstate::ExtendedStateKind,
        state_id: u32,
    ) -> u32 {
        self.move_shape.get_extended_state(kind, state_id)
    }



    pub(crate) fn realm_appellation_bonus_identity(
        &self,
    ) -> Option<super::skills::realmappellation::RealmBonusIdentity> {
        (self.realm_appellation_skill_id != UNKNOWN_SKILL_ID
            && (1..=4).contains(&self.realm_appellation_skill_level))
        .then_some(super::skills::realmappellation::RealmBonusIdentity {
            skill_id: self.realm_appellation_skill_id,
            level: self.realm_appellation_skill_level,
        })
    }

    pub(crate) const fn set_realm_appellation_bonus_identity(&mut self, skill_id: u32, level: i32) {
        self.realm_appellation_skill_id = skill_id;
        self.realm_appellation_skill_level = level;
    }

    pub(crate) fn realm_appellation_entitled(&self, appellation_id: u32, factory: &CSkillFactory) -> bool {
        super::skills::realmappellation::is_title(appellation_id)
            && self
                .move_shape
                .skill(appellation_id, factory)
                .is_some_and(|skill| skill.level() > 0)
    }

    /// Достигнутая часть единого `m_mapNameValue/GetScriptValue` catalog.
    /// DWORD возвращаются теми же битами в signed script integer; неизвестное
    /// имя остаётся `None`, а сам GetMe преобразует его в legacy zero.
    pub(crate) fn script_value(&self, property: &[u8]) -> Option<i32> {
        if property.eq_ignore_ascii_case(b"lRegionID") {
            Some(self.server_region_id().unwrap_or_default())
        } else if property.eq_ignore_ascii_case(b"lID") {
            Some(self.player_id())
        } else if property.eq_ignore_ascii_case(b"lTileX") {
            Some(self.shape().get_tile_x().unwrap_or_default())
        } else if property.eq_ignore_ascii_case(b"lTileY") {
            Some(self.shape().get_tile_y().unwrap_or_default())
        } else if property.eq_ignore_ascii_case(b"lDir") {
            Some(self.shape().get_direction())
        } else if property.eq_ignore_ascii_case(b"wState") {
            Some(i32::from(self.shape().get_state()))
        } else if property.eq_ignore_ascii_case(b"wAction") {
            Some(i32::from(self.shape().get_action()))
        } else if property.eq_ignore_ascii_case(b"btCountry") {
            Some(i32::from(self.country()))
        } else if property.eq_ignore_ascii_case(b"lPos") {
            Some(self.shape().get_position())
        } else if property.eq_ignore_ascii_case(b"dwVigour") {
            Some(self.vigour() as i32)
        } else if property.eq_ignore_ascii_case(b"lLevel") {
            Some(i32::from(self.level()))
        } else if property.eq_ignore_ascii_case(b"lSex") {
            Some(i32::from(self.base_properties.sex))
        } else if property.eq_ignore_ascii_case(b"dwExp") {
            Some(self.experience() as i32)
        } else if property.eq_ignore_ascii_case(b"wPkCount") {
            Some(i32::from(self.pk_count()))
        } else if property.eq_ignore_ascii_case(b"lOccupation") {
            Some(i32::from(self.occupation()))
        } else if property.eq_ignore_ascii_case(b"dwAppellationID") {
            Some(self.base_properties.appellation_id as i32)
        } else if property.eq_ignore_ascii_case(b"dwRankOfNobilityID") {
            Some(self.base_properties.rank_of_nobility_id as i32)
        } else if property.eq_ignore_ascii_case(b"dwCredit") {
            Some(self.base_properties.credit as i32)
        } else if property.eq_ignore_ascii_case(b"dwSZL") {
            Some(self.base_properties.szl as i32)
        } else if property.eq_ignore_ascii_case(b"lContribute") {
            Some(self.contribution)
        } else if property.eq_ignore_ascii_case(b"bFairyContainerEnabled") {
            Some(i32::from(self.base_properties.fairy_container_enabled))
        } else if property.eq_ignore_ascii_case(b"bBattleFairyEnabled") {
            Some(i32::from(self.base_properties.battle_fairy_enabled))
        } else {
            None
        }
    }

    /// Достигнутая writable-часть того же `m_mapNameValue/SetValue` catalog.
    /// Узкие поля сохраняют исходное integer narrowing, DWORD — все биты.
    pub(crate) fn set_script_value(&mut self, property: &[u8], value: i32) -> Option<i32> {
        if property.eq_ignore_ascii_case(b"btCountry") {
            Some(self.set_script_country(value))
        } else if property.eq_ignore_ascii_case(b"dwVigour") {
            Some(self.set_script_vigour(value))
        } else if property.eq_ignore_ascii_case(b"lLevel") {
            self.base_properties.level = value as u8;
            Some(i32::from(self.base_properties.level))
        } else if property.eq_ignore_ascii_case(b"lSex") {
            self.base_properties.sex = value as u8;
            Some(i32::from(self.base_properties.sex))
        } else if property.eq_ignore_ascii_case(b"dwExp") {
            Some(self.set_script_experience(value))
        } else if property.eq_ignore_ascii_case(b"lOccupation") {
            self.base_properties.occupation = value as u8;
            Some(i32::from(self.base_properties.occupation))
        } else if property.eq_ignore_ascii_case(b"dwAppellationID") {
            self.base_properties.appellation_id = value as u32;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"dwRankOfNobilityID") {
            self.base_properties.rank_of_nobility_id = value as u32;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"dwCredit") {
            self.base_properties.credit = value as u32;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"dwSZL") {
            self.base_properties.szl = value as u32;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"lContribute") {
            self.contribution = value;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"bFairyContainerEnabled") {
            self.base_properties.fairy_container_enabled = value != 0;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"bBattleFairyEnabled") {
            self.base_properties.battle_fairy_enabled = value != 0;
            Some(value)
        } else {
            None
        }
    }

    /// `ChangeValue` применяет wrapping arithmetic ширины фактического поля.
    /// Shipped GM-script использует исторический алиас `Experience` для
    /// canonical `dwExp`.
    pub(crate) fn change_script_value(&mut self, property: &[u8], delta: i32) -> Option<i32> {
        let canonical = if property.eq_ignore_ascii_case(b"Experience") {
            b"dwExp".as_slice()
        } else {
            property
        };
        let current = self.script_value(canonical)?;
        if canonical.eq_ignore_ascii_case(b"bFairyContainerEnabled")
            || canonical.eq_ignore_ascii_case(b"bBattleFairyEnabled")
        {
            let changed = i32::from(current.wrapping_add(delta) != 0);
            let _ = self.set_script_value(canonical, changed)?;
            return Some(changed);
        }
        self.set_script_value(canonical, current.wrapping_add(delta))
    }

    pub(crate) const fn gods_battle_faction(&self) -> i32 {
        self.base_properties.gods_battle_faction
    }

    pub(crate) const fn level(&self) -> u8 {
        self.base_properties.level
    }

    pub(crate) const fn set_level(&mut self, level: u8) {
        self.base_properties.level = level;
    }

    pub(crate) fn apply_level_property_upgrade(
        &mut self,
        upgrade: &crate::setup::playerlist::PlayerPropertiesUpgrade,
    ) {
        self.base_properties.base_maximum_hp = upgrade.base_maximum_hp;
        self.base_properties.base_dexterity = upgrade.base_dexterity;
        self.base_properties.base_maximum_mp = upgrade.base_maximum_mp;
        self.base_properties.base_strength = upgrade.base_strength;
        self.base_properties.base_burden = upgrade.base_burden;
        self.base_properties.base_constitution = upgrade.base_constitution;
        self.base_properties.base_intelligence = upgrade.base_intelligence;
    }

    pub(crate) const fn set_base_maximum_rp(&mut self, value: u16) {
        self.base_properties.maximum_rp = value;
    }

    pub(crate) const fn level_wire_properties(&self) -> (u32, u32, u16, u16) {
        (
            self.base_properties.base_maximum_hp,
            self.base_properties.base_maximum_mp,
            self.base_properties.base_burden,
            self.base_properties.maximum_rp,
        )
    }

    pub(crate) const fn energy(&self) -> u32 {
        self.base_properties.energy
    }

    pub(crate) const fn maximum_energy(&self) -> u32 {
        self.base_properties.maximum_energy
    }

    /// Exact `CPlayer::SetEnergy`: unsigned caller arithmetic сохраняется,
    /// затем значение ограничивается текущим `dwMaxEnergy`.
    pub(crate) fn set_energy(&mut self, energy: u32) {
        self.base_properties.energy = energy.min(self.base_properties.maximum_energy);
    }

    /// `CPlayer::SetMaxEnergy` сохраняет новый максимум и сразу ограничивает
    /// им текущее значение энергии.
    pub(crate) fn set_maximum_energy(&mut self, energy: u32) {
        self.base_properties.maximum_energy = energy;
        self.base_properties.energy = self.base_properties.energy.min(energy);
    }

    pub(crate) const fn occupation(&self) -> u8 {
        self.base_properties.occupation
    }

    /// Cross-Game level relay сохраняет `SetLevel` mutation и отдельный
    /// caller-side faction publication; experience сбрасывается после неё.
    pub(crate) fn apply_remote_level(&mut self, level: u8) -> PlayerRemoteLevelMutation {
        let mutation = PlayerRemoteLevelMutation {
            player_id: self.player_id(),
            faction_id: self.faction_id,
            previous_level: self.base_properties.level,
            level,
        };
        self.base_properties.level = level;
        self.base_properties.experience = 0;
        mutation
    }

    pub(crate) const fn szl(&self) -> u32 {
        self.base_properties.szl
    }

    pub(crate) const fn set_szl(&mut self, value: u32) {
        self.base_properties.szl = value;
    }

    pub(crate) const fn attempt_appellation_id(&self) -> u32 {
        self.attempt_appellation_id
    }

    pub(crate) const fn set_gods_battle_faction(&mut self, faction: i32) {
        self.base_properties.gods_battle_faction = faction;
    }

    /// Assembly/load boundary для persisted player tail; faction membership
    /// сам region восстанавливает только после фактического `AddObject`.
    pub(crate) const fn restore_gods_battle_state(&mut self, faction: i32, szl: u32) {
        self.base_properties.gods_battle_faction = faction;
        self.base_properties.szl = szl;
    }

    pub(crate) const fn restore_level_and_attempt_appellation(
        &mut self,
        level: u8,
        attempt_appellation_id: u32,
    ) {
        self.base_properties.level = level;
        self.attempt_appellation_id = attempt_appellation_id;
    }

    /// Exact `SetExploit`: signed CountryParam storage сравнивается как
    /// `unsigned long`, затем значение зажимается только сверху.
    pub(crate) const fn exploit(&self) -> u32 {
        self.base_properties.exploit
    }

    pub(crate) fn set_exploit(&mut self, requested: u32, maximum: i32) -> u32 {
        let previous = self.base_properties.exploit;
        let applied = requested.min(maximum as u32);
        self.base_properties.exploit = applied;
        tracing::trace!(
            player_id = self.player_id(),
            previous,
            requested,
            applied,
            "подвиг игрока изменён"
        );
        applied
    }

    /// Exact `SetValue("dwExploit", value)` из region reward path:
    /// generic property map пишет `DWORD` напрямую и не вызывает `SetExploit` clamp.
    pub(crate) fn set_exploit_property_value(&mut self, requested: u32) {
        let previous = self.base_properties.exploit;
        self.base_properties.exploit = requested;
        tracing::trace!(
            player_id = self.player_id(),
            previous,
            requested,
            "свойство подвига игрока изменено напрямую"
        );
    }

    /// `OnPlayerTimgingStart` отсеивает action `ACT_DIED == 6`
    /// отдельно от health-based `CMoveShape::IsDied`.
    pub(crate) fn can_start_nation_war_timing(&self) -> bool {
        self.shape().get_action() != 6 && !CMoveShape::is_died(self.base_properties.health)
    }

    /// Тот же double guard использует `ServerNationRegion::OnMonsterDamage`.
    pub(crate) fn can_attack_nation_monster(&self) -> bool {
        self.shape().get_action() != 6 && !CMoveShape::is_died(self.base_properties.health)
    }

    /// Focused same-region branch `ChangeRegion`, которую вызывает
    /// `ServerNationRegion::KickOutAllPlayerToReturnPoint`.
    pub(crate) fn prepare_nation_relive(&mut self) {
        self.current_progress = PlayerProgress::None;
        self.recreate_carriage = false;
    }

    pub(crate) const fn movement_position_facts(
        &self,
        area_width: i32,
        area_height: i32,
    ) -> MoveShapePositionFacts {
        MoveShapePositionFacts {
            current_hit_points: self.base_properties.health,
            figure: self.figure,
            current_area: None,
            area_width,
            area_height,
        }
    }

    pub(crate) const fn movement_shape_mut(&mut self) -> &mut CShape {
        self.move_shape.shape_mut()
    }

    pub(crate) const fn move_shape(&self) -> &CMoveShape {
        &self.move_shape
    }

    pub(crate) const fn move_shape_mut(&mut self) -> &mut CMoveShape {
        &mut self.move_shape
    }

    pub(crate) fn back_stage_skill_id(&self, index: usize) -> Option<u32> {
        self.player_ai.base_ai().back_stage_skill_id(index)
    }

    pub(crate) fn begin_pending_back_stage_skill_ids(&mut self, factory: &CSkillFactory) -> Vec<u32> {
        self.player_ai.base_ai_mut().begin_pending_back_stage_skill_ids()
            .into_iter()
            .filter(|skill_id| self.move_shape.skill(*skill_id, factory).is_some()
                && !self.move_shape.immediate_skill_ended(*skill_id, factory))
            .collect()
    }

    pub(crate) fn auto_start_passive_skills(&mut self) -> usize {
        self.move_shape.auto_start_passive_skills(self.player_ai.base_ai_mut())
    }

    pub(crate) const fn can_process_ai_destination(&self) -> bool {
        !CMoveShape::is_died(self.base_properties.health)
    }

    pub(crate) const fn is_movement_allowed(&self) -> bool {
        self.move_shape.is_moveable()
    }

    pub(crate) fn movement_speed(&self) -> f32 {
        self.shape().get_speed()
    }

    pub(crate) const fn set_skill_moveable(&mut self, moveable: bool) {
        self.move_shape.set_moveable(moveable);
    }

    pub(crate) const fn set_skill_fightable(&mut self, fightable: bool) {
        self.move_shape.set_fightable(fightable);
    }

    pub(crate) fn has_state_by_skill_id(&self, state_id: u32) -> bool {
        self.move_shape.has_state_by_skill_id(state_id)
    }

    pub(crate) fn force_move(
        &mut self,
        server_region: &mut CServerRegion,
        destination_x: i32,
        destination_y: i32,
        duration_ms: u32,
        area_width: i32,
        area_height: i32,
        around: &GameServerAroundRuntime<'_>,
        now_ms: impl FnOnce() -> u32,
    ) -> Result<bool, MoveShapeCommandBlock> {
        let facts = self.movement_position_facts(area_width, area_height);
        let moved = self.move_shape.force_move(
            Some(server_region),
            destination_x,
            destination_y,
            duration_ms,
            facts,
            around,
        )?;
        if moved {
            self.player_ai.begin_forced_stand(duration_ms, now_ms());
        }
        Ok(moved)
    }

    /// Клиентский ИИ и сценарные `WalkStep`/`RunStep` используют обычного
    /// владельца движения `CMoveShape`, поэтому сетевой маршрут `0xBF605` и
    /// перестановка в регионе остаются единым действием.
    pub(crate) fn move_step(
        &mut self,
        server_region: &mut CServerRegion,
        destination_x: i32,
        destination_y: i32,
        run: i32,
        area_width: i32,
        area_height: i32,
        around: &GameServerAroundRuntime<'_>,
    ) -> Result<(), MoveShapeCommandBlock> {
        let facts = self.movement_position_facts(area_width, area_height);
        self.move_shape.on_move(
            Some(server_region),
            destination_x,
            destination_y,
            run,
            facts,
            around,
        )
    }

    pub(crate) const fn figure(&self) -> ShapeFigure {
        self.figure
    }

    pub(crate) const fn nation_relive_position_facts(
        &self,
        area_width: i32,
        area_height: i32,
    ) -> MoveShapePositionFacts {
        self.movement_position_facts(area_width, area_height)
    }

    pub(crate) const fn nation_relive_shape_mut(&mut self) -> &mut CShape {
        self.movement_shape_mut()
    }

    pub(crate) const fn combat_properties(&self) -> PlayerCombatProperties {
        self.combat_properties
    }

    /// Один concrete OnUpdateProperties меняет живой tagProperty.
    /// Синхронизация wire — чистая проекция полей, без equipment callbacks,
    /// пересчёта остальных состояний или отложенной публикации visual.
    pub(crate) fn update_state_combat_properties(
        &mut self,
        update: impl FnOnce(PlayerCombatProperties) -> PlayerCombatProperties,
    ) {
        self.combat_properties = update(self.combat_properties);
        self.sync_combat_property_wire();
    }

    /// Exact scalar checks `CanUseItem`; catalog/instance addon fallback
    /// остаётся у `CGoods`, а result-коды являются частью `0xBF709` wire.
    pub(crate) fn can_use_item(&self, goods: &CGoods, factory: &CGoodsFactory) -> i32 {
        Self::can_use_item_from_properties(
            self.base_properties,
            self.combat_properties,
            goods,
            factory,
        )
    }

    /// Exact `CPlayer::CanMountEquip`: для headgear сначала согласует persisted
    /// ordinary/battle-fairy enable flags с `GAP_BF_BATTLE_FAIRY`, затем
    /// возвращает те же requirement-коды `1..7` или magic success `9`.
    pub(crate) fn can_mount_equip(&self, goods: &CGoods, factory: &CGoodsFactory) -> i32 {
        Self::can_mount_equip_from_properties(
            self.base_properties,
            self.combat_properties,
            goods,
            factory,
        )
    }

    fn can_mount_equip_from_properties(
        base_properties: PlayerBaseProperties,
        combat_properties: PlayerCombatProperties,
        goods: &CGoods,
        factory: &CGoodsFactory,
    ) -> i32 {
        if factory
            .query_goods_base_properties(goods.base_properties_index())
            .is_some_and(|properties| properties.equip_place() == EQUIP_PLACE_HEADGEAR)
        {
            let battle_fairy_headgear =
                goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) == 1;
            if (!base_properties.fairy_container_enabled
                && (!base_properties.battle_fairy_enabled || !battle_fairy_headgear))
                || (base_properties.fairy_container_enabled
                    && !base_properties.battle_fairy_enabled
                    && battle_fairy_headgear)
            {
                return 0;
            }
        }
        Self::can_use_item_from_properties(base_properties, combat_properties, goods, factory)
    }

    fn can_use_item_from_properties(
        base_properties: PlayerBaseProperties,
        combat_properties: PlayerCombatProperties,
        goods: &CGoods,
        factory: &CGoodsFactory,
    ) -> i32 {
        let required = |property| goods.addon_property_value(factory, property, 1) as u32;
        let level = required(GAP_ROLE_MINIMUM_LEVEL_LIMIT);
        if level != 0 && u32::from(base_properties.level) < level {
            return 1;
        }
        for (property, actual, result) in [
            (
                GAP_ROLE_MINIMUM_STRENGTH_LIMIT,
                combat_properties.strength,
                2,
            ),
            (
                GAP_ROLE_MINIMUM_AGILITY_LIMIT,
                combat_properties.dexterity,
                3,
            ),
            (
                GAP_ROLE_MINIMUM_CONSTITUTION_LIMIT,
                combat_properties.constitution,
                4,
            ),
            (
                GAP_ROLE_MINIMUM_WAKAN_LIMIT,
                combat_properties.intelligence,
                5,
            ),
        ] {
            let minimum = required(property);
            if minimum != 0 && actual < minimum {
                return result;
            }
        }
        let occupation = required(GAP_REQUIRE_OCCUPATION);
        if occupation != 0 && occupation != u32::from(base_properties.occupation) + 1 {
            return 6;
        }
        let gender = required(GAP_REQUIRE_GENDER);
        if gender != 0 && gender != u32::from(base_properties.sex) {
            return 7;
        }
        9
    }

    pub(crate) fn item_skill_level(&self, skill_id: u32, factory: &CSkillFactory) -> i32 {
        self.move_shape
            .skill(skill_id, factory)
            .map_or(0, MoveShapeSkill::level)
    }

    pub(crate) fn learn_item_skill(
        &mut self,
        skill_id: u32,
        level: i32,
        factory: &CSkillFactory,
    ) -> bool {
        self.move_shape.add_skill(skill_id, level, factory)
    }

    pub(crate) fn set_item_skill_position(&mut self, skill_id: u32, position: i32, factory: &CSkillFactory) -> bool {
        self.move_shape.set_item_skill_position(skill_id, position, factory)
    }

    pub(crate) fn item_skill_position(&self, skill_id: u32, factory: &CSkillFactory) -> Option<i32> {
        self.move_shape.skill(skill_id, factory).map(MoveShapeSkill::item_position)
    }

    /// `CPlayer::ReUseSkillItem` различает отсутствующий map-ключ и сохранённый
    /// нулевой timestamp после переполнения `timeGetTime`.
    pub(crate) fn last_skill_item_use_ms(&self, item_index: u32) -> Option<u32> {
        self.last_skill_item_use_ms.get(&item_index).copied()
    }

    pub(crate) fn mark_skill_item_used(&mut self, item_index: u32, now_ms: u32) {
        self.last_skill_item_use_ms.insert(item_index, now_ms);
    }

    /// `DeleteSkillItem` работает только с сохранённой ячейкой и не ищет
    /// подходящий stack в остальных ячейках пакета.
    pub(crate) fn consume_skill_item_at(
        &mut self,
        position: u32,
        item_index: u32,
        amount: u32,
    ) -> Option<CiQingPacketConsumption> {
        let goods = self.packet.get_goods(position)?;
        if goods.base_properties_index() != item_index || goods.amount() < amount || amount == 0 {
            return None;
        }
        self.remove_packet_goods_by_id(goods.identity().ex_id, amount)
    }

    /// UseItem, tagExpendableEffect 0x4A..4D: часы новой записи находятся в
    /// 0x4545F4/45473B/454887/4549CE, замены — 0x4546BC/454808/45494F/454A8D.
    /// Два value(1) обслуживают разные native reads: payload и live property.
    pub(crate) fn apply_expendable_item_effect(
        &mut self,
        property_type: i32,
        mut value: impl FnMut(u32) -> i32,
        mut now: impl FnMut() -> u32,
    ) -> i32 {
        if !matches!(property_type, 0x4a..=0x4d) {
            return 0;
        }
        let previous = self.expendable_effects.get(&property_type).copied();
        let mut adjust_property = |amount: i32, subtract: bool| {
            match property_type {
                0x4a => {
                    let current = self.combat_properties.maximum_attack;
                    self.combat_properties.maximum_attack = if subtract {
                        current.wrapping_sub(amount as u32)
                    } else {
                        current.wrapping_add(amount as u32)
                    };
                }
                0x4b => {
                    let current = self.combat_properties.attack_speed;
                    self.combat_properties.attack_speed = if subtract {
                        current.wrapping_sub(amount as u16)
                    } else {
                        current.wrapping_add(amount as u16)
                    };
                }
                0x4c => {
                    let current = self.combat_properties.defense;
                    self.combat_properties.defense = if subtract {
                        current.wrapping_sub(amount as u32)
                    } else {
                        current.wrapping_add(amount as u32)
                    };
                }
                0x4d => {
                    let current = self.combat_properties.element_modify;
                    self.combat_properties.element_modify = if subtract {
                        current.wrapping_sub(amount)
                    } else {
                        current.wrapping_add(amount)
                    };
                }
                _ => unreachable!("поддержанный expendable property проверен до чтения значений"),
            }
        };
        if let Some(previous) = previous {
            adjust_property(previous.value, true);
            adjust_property(value(1), false);
            let effect = self.expendable_effects.get_mut(&property_type)
                .expect("замена сохраняет найденный expendable effect");
            effect.start_time_ms = now();
            effect.effect_time_ms = value(2) as u32;
            effect.value = value(1);
        } else {
            let stored_value = value(1);
            let start_time_ms = now();
            let effect_time_ms = value(2) as u32;
            adjust_property(value(1), false);
            self.expendable_effects.insert(property_type, PlayerExpendableEffect {
                property_type,
                value: stored_value,
                start_time_ms,
                effect_time_ms,
            });
        }
        match property_type {
            0x4a => LegacyWriter::write_u32_at(
                &mut self.combat_property_wire,
                0x20,
                self.combat_properties.maximum_attack,
            )
            .expect("combat wire содержит maximum attack"),
            0x4b => LegacyWriter::write_u16_at(
                &mut self.combat_property_wire,
                0x32,
                self.combat_properties.attack_speed,
            )
            .expect("combat wire содержит attack speed"),
            0x4c => LegacyWriter::write_u32_at(
                &mut self.combat_property_wire,
                0x2c,
                self.combat_properties.defense,
            )
            .expect("combat wire содержит defense"),
            0x4d => LegacyWriter::write_i32_at(
                &mut self.combat_property_wire,
                0x48,
                self.combat_properties.element_modify,
            )
            .expect("combat wire содержит element modify"),
            _ => {}
        }
        match property_type {
            0x4a => self.combat_properties.maximum_attack as i32,
            0x4b => i32::from(self.combat_properties.attack_speed),
            0x4c => self.combat_properties.defense as i32,
            0x4d => self.combat_properties.element_modify,
            _ => 0,
        }
    }

    pub(crate) const fn combat_property_wire(&self) -> &[u8; PLAYER_COMBAT_PROPERTY_WIRE_SIZE] {
        &self.combat_property_wire
    }

    /// Selector `3009` читает не производную Rust-модель, а те же concrete
    /// `tagBaseProperty`/`tagProperty` slots, которые использует EXE. Поэтому
    /// неизвестные, но загруженные и пересчитанные поля сохраняются в выводе.
    pub(crate) fn all_properties_diagnostic_snapshot(
        &self,
    ) -> PlayerAllPropertiesDiagnosticSnapshot {
        let base = self.synchronized_base_property_wire();
        let current = &self.combat_property_wire;
        let base_u16 = |offset| u32::from(read_player_wire_u16(&base, offset));
        let base_u32 = |offset| read_player_wire_u32(&base, offset);
        let current_u16 = |offset| u32::from(read_player_wire_u16(current, offset));
        let current_u32 = |offset| read_player_wire_u32(current, offset);
        PlayerAllPropertiesDiagnosticSnapshot {
            name: self.shape().base_object().get_name().to_vec(),
            summary_words: [
                u32::from(self.occupation()),
                u32::from(self.level()),
                self.experience(),
                base_u32(BASE_HEALTH_OFFSET),
                current_u32(0x00),
                base_u32(BASE_MANA_OFFSET),
                current_u32(0x04),
                base_u16(0xac),
                current_u16(0x0a),
                base_u32(BASE_MAXIMUM_HP_OFFSET),
                base_u32(BASE_MAXIMUM_MP_OFFSET),
                base_u16(0xba),
                base_u16(BASE_PK_COUNT_OFFSET),
                base_u32(BASE_KILL_COUNT_OFFSET),
                self.money(),
            ],
            base_combat_words: [
                base_u32(BASE_STRENGTH_OFFSET),
                base_u32(BASE_DEXTERITY_OFFSET),
                base_u32(BASE_CONSTITUTION_OFFSET),
                base_u32(BASE_INTELLIGENCE_OFFSET),
                base_u32(0xcc),
                base_u32(0xd0),
                base_u16(0xd4),
                base_u16(0xd6),
                base_u16(0xd8),
                base_u32(0xdc),
                base_u16(0xe0),
                base_u16(0xe2),
                base_u32(0xe4),
                base_u16(0xe8),
                base_u16(0xea),
            ],
            current_combat_words: [
                current_u32(0x0c),
                current_u32(0x10),
                current_u32(0x14),
                current_u32(0x18),
                current_u32(0x1c),
                current_u32(0x20),
                current_u16(0x24),
                current_u16(0x26),
                current_u16(0x28),
                current_u32(0x2c),
                current_u16(0x30),
                i32::from(read_player_wire_u16(current, 0x32) as i16) as u32,
                current_u32(0x34),
                current_u16(0x38),
                current_u16(0x3a),
                current_u16(0x3c),
                current_u32(0x40),
                current_u16(0x44),
                current_u32(0x48),
                current_u16(0x4c),
            ],
        }
    }

    /// Граница восстановления exact `m_Property` из persisted player state.
    /// Последующие reached-пересчёты заменяют только известные поля layout.
    pub(crate) const fn restore_combat_property_wire(
        &mut self,
        wire: [u8; PLAYER_COMBAT_PROPERTY_WIRE_SIZE],
    ) {
        self.combat_property_wire = wire;
    }

    /// Применяет результат виртуального `UpdateProperty` и синхронизирует
    /// подтверждённые поля 0x9c-byte `tagProperty`, сохраняя неизвестные байты.
    pub(crate) fn apply_recomputed_combat_properties(
        &mut self,
        properties: PlayerCombatProperties,
        goods_factory: &CGoodsFactory,
    ) {
        self.combat_properties = properties;
        self.sync_combat_property_wire();
        self.refresh_equipment_flash(goods_factory);
    }

    fn sync_combat_property_wire(&mut self) {
        let properties = self.combat_properties;
        let write_u16 = |wire: &mut [u8], offset: usize, value: u16| {
            LegacyWriter::write_u16_at(wire, offset, value)
                .expect("combat wire offset проверен layout-константой");
        };
        let write_u32 = |wire: &mut [u8], offset: usize, value: u32| {
            LegacyWriter::write_u32_at(wire, offset, value)
                .expect("combat wire offset проверен layout-константой");
        };
        write_u32(&mut self.combat_property_wire, 0x00, properties.maximum_hp);
        write_u32(&mut self.combat_property_wire, 0x04, properties.maximum_mp);
        write_u16(&mut self.combat_property_wire, 0x08, properties.maximum_yp);
        write_u16(&mut self.combat_property_wire, 0x0a, properties.maximum_rp);
        write_u32(&mut self.combat_property_wire, 0x0c, properties.strength);
        write_u32(&mut self.combat_property_wire, 0x10, properties.dexterity);
        write_u32(
            &mut self.combat_property_wire,
            0x14,
            properties.constitution,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x18,
            properties.intelligence,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x1c,
            properties.minimum_attack,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x20,
            properties.maximum_attack,
        );
        write_u16(
            &mut self.combat_property_wire,
            0x32,
            properties.attack_speed,
        );
        write_u16(&mut self.combat_property_wire, 0x24, properties.hit);
        write_u16(&mut self.combat_property_wire, 0x30, properties.dodge);
        write_u16(&mut self.combat_property_wire, 0x28, properties.cch);
        write_u16(&mut self.combat_property_wire, 0x26, properties.burden);
        write_u32(&mut self.combat_property_wire, 0x2c, properties.defense);
        write_u32(
            &mut self.combat_property_wire,
            0x34,
            properties.element_resistance,
        );
        write_u16(
            &mut self.combat_property_wire,
            0x3c,
            properties.soul_resistance,
        );
        write_u16(
            &mut self.combat_property_wire,
            0x44,
            properties.add_soul_attack,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x58,
            properties.blast_attack_scale_bits,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x40,
            properties.add_element_attack,
        );
        write_u16(&mut self.combat_property_wire, 0x38, properties.hp_recovery);
        write_u16(&mut self.combat_property_wire, 0x3a, properties.mp_recovery);
        write_u32(
            &mut self.combat_property_wire,
            0x48,
            properties.element_modify as u32,
        );
        write_u16(&mut self.combat_property_wire, 0x4c, properties.reank);
        write_u16(
            &mut self.combat_property_wire,
            0x4e,
            properties.attack_avoid,
        );
        write_u16(
            &mut self.combat_property_wire,
            0x50,
            properties.element_avoid,
        );
        write_u16(&mut self.combat_property_wire, 0x52, properties.full_miss);
        write_u16(
            &mut self.combat_property_wire,
            0x54,
            properties.blast_attack,
        );
        write_u16(
            &mut self.combat_property_wire,
            0x56,
            properties.blast_element_attack,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x5c,
            properties.blast_defense_scale_bits,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x60,
            properties.element_blast_attack_scale_bits,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x64,
            properties.element_blast_defense_scale_bits,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x68,
            properties.full_miss_scale_bits,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x6c,
            properties.critical_rate_bits,
        );
        write_u32(&mut self.combat_property_wire, 0x70, properties.resume_hp_peace as u32);
        write_u32(&mut self.combat_property_wire, 0x74, properties.resume_mp_peace as u32);
        write_u32(&mut self.combat_property_wire, 0x78, properties.resume_hp_fight as u32);
        write_u32(&mut self.combat_property_wire, 0x7c, properties.resume_mp_fight as u32);
        write_u32(&mut self.combat_property_wire, 0x80, properties.restored_hp_peace as u32);
        write_u32(&mut self.combat_property_wire, 0x84, properties.restored_mp_peace as u32);
        write_u32(&mut self.combat_property_wire, 0x88, properties.restored_hp_fight as u32);
        write_u32(&mut self.combat_property_wire, 0x8c, properties.restored_mp_fight as u32);
        self.combat_property_wire[0x90] = u8::from(properties.battle_fairy_summoned);
        self.combat_property_wire[0x91] = u8::from(properties.battle_fairy_recall);
        self.combat_property_wire[0x92] = u8::from(properties.battle_fairy_died);
    }

    /// `MountAllEquip -> SetCurFlash`: пересобирает 17 flash-ячеек после
    /// каждого полного property commit. Исторический system clock заменён
    /// `SystemTime`; damaged equipment намеренно сохраняет прежнее значение,
    /// потому что original loop пропускает `SetCurFlash` для нулевой прочности.
    fn refresh_equipment_flash(&mut self, factory: &CGoodsFactory) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs() as u32);
        for position in 0..17usize {
            let Some(goods) = self.equipment.get_goods(position as u32) else {
                self.flash_current[position] = 0;
                self.flash_changed = true;
                continue;
            };
            if goods.query_attribute(GAP_GOODS_MAXIMUM_DURABILITY)
                && goods.addon_property_value(factory, GAP_GOODS_MAXIMUM_DURABILITY, 2) < 1
            {
                continue;
            }
            let flash = if goods.query_attribute(GAP_GOODS_LIFE_TYPE) {
                match goods.addon_property_value(factory, GAP_GOODS_EQUIMENT_FLASH, 2) {
                    1 => {
                        let expiry = goods
                            .start_point(factory)
                            .wrapping_add(u64::from(goods.goods_lifetime(factory)));
                        if (expiry >> 32) as i32 == 0 && expiry as u32 <= now {
                            0
                        } else {
                            goods.addon_property_value(factory, GAP_GOODS_EQUIMENT_FLASH, 1) as u32
                        }
                    }
                    2 => goods.addon_property_value(factory, GAP_GOODS_EQUIMENT_FLASH, 1) as u32,
                    _ => continue,
                }
            } else {
                goods.addon_property_value(factory, GAP_GOODS_EQUIMENT_FLASH, 1) as u32
            };
            self.flash_current[position] = flash;
            self.flash_changed = true;
        }
    }

    /// `DoneFlash` забирает один pending snapshot и одновременно продвигает
    /// previous-table. Повторный tick без нового `MountAllEquip` ничего не шлёт.
    pub(crate) fn take_flash_update(&mut self) -> Option<[(u32, u32); 17]> {
        if !self.flash_changed {
            return None;
        }
        let pairs = std::array::from_fn(|position| {
            (self.flash_previous[position], self.flash_current[position])
        });
        self.flash_previous = self.flash_current;
        self.flash_changed = false;
        Some(pairs)
    }

    pub(crate) const fn stat_allocation_state(&self) -> PlayerStatAllocationState {
        PlayerStatAllocationState {
            sex: self.base_properties.sex,
            occupation: self.base_properties.occupation,
            remain_point: self.base_properties.remain_point,
            base_maximum_hp: self.base_properties.base_maximum_hp,
            base_maximum_mp: self.base_properties.base_maximum_mp,
            base_strength: self.base_properties.base_strength,
            base_dexterity: self.base_properties.base_dexterity,
            base_constitution: self.base_properties.base_constitution,
            base_intelligence: self.base_properties.base_intelligence,
        }
    }

    /// Восстанавливает owned поля `m_BaseProperty`, участвующие в client
    /// allocation `0x8FA01`; полный decoder игрока остаётся отдельным owner-ом.
    pub(crate) const fn restore_stat_allocation_state(&mut self, state: PlayerStatAllocationState) {
        self.base_properties.sex = state.sex;
        self.base_properties.occupation = state.occupation;
        self.base_properties.remain_point = state.remain_point;
        self.base_properties.base_maximum_hp = state.base_maximum_hp;
        self.base_properties.base_maximum_mp = state.base_maximum_mp;
        self.base_properties.base_strength = state.base_strength;
        self.base_properties.base_dexterity = state.base_dexterity;
        self.base_properties.base_constitution = state.base_constitution;
        self.base_properties.base_intelligence = state.base_intelligence;
    }

    /// Exact mutation-tail `0x8FA01`: DEX/CON/INT используют legacy STR gate;
    /// неизвестный selector всё равно расходует одно очко и ведёт к recompute.
    pub(crate) fn allocate_stat_point(
        &mut self,
        selector: u8,
        constitution_hp: u16,
        intelligence_mp: u16,
    ) -> Option<PlayerStatAllocationMutation> {
        if self.base_properties.remain_point == 0 {
            return None;
        }
        let previous = self.stat_allocation_state();
        let strength_gate = self.base_properties.base_strength < i32::MAX as u32;
        let stat_changed = match selector {
            0 if strength_gate => {
                self.base_properties.base_strength =
                    self.base_properties.base_strength.wrapping_add(1);
                true
            }
            1 if strength_gate => {
                self.base_properties.base_dexterity =
                    self.base_properties.base_dexterity.wrapping_add(1);
                true
            }
            2 => {
                if strength_gate {
                    self.base_properties.base_constitution =
                        self.base_properties.base_constitution.wrapping_add(1);
                }
                self.base_properties.base_maximum_hp = self
                    .base_properties
                    .base_maximum_hp
                    .wrapping_add(u32::from(constitution_hp));
                strength_gate
            }
            3 => {
                if strength_gate {
                    self.base_properties.base_intelligence =
                        self.base_properties.base_intelligence.wrapping_add(1);
                }
                self.base_properties.base_maximum_mp = self
                    .base_properties
                    .base_maximum_mp
                    .wrapping_add(u32::from(intelligence_mp));
                strength_gate
            }
            _ => false,
        };
        self.base_properties.remain_point = self.base_properties.remain_point.wrapping_sub(1);
        Some(PlayerStatAllocationMutation {
            player_id: self.player_id(),
            selector,
            stat_changed,
            previous,
            current: self.stat_allocation_state(),
        })
    }

    pub(crate) const fn pk_permissions(&self) -> PlayerPkPermissions {
        PlayerPkPermissions {
            player: self.base_properties.pk_normal,
            teammate: self.base_properties.pk_team,
            guild_member: self.base_properties.pk_union,
            criminal: self.base_properties.pk_badman,
            country: self.base_properties.pk_country,
        }
    }

    /// Граница восстановления пяти persisted `bPk_*` перед skill/AI use.
    pub(crate) const fn restore_pk_permissions(&mut self, permissions: PlayerPkPermissions) {
        self.base_properties.pk_normal = permissions.player;
        self.base_properties.pk_team = permissions.teammate;
        self.base_properties.pk_union = permissions.guild_member;
        self.base_properties.pk_badman = permissions.criminal;
        self.base_properties.pk_country = permissions.country;
    }

    /// Exact selector `0x8FA05`; неизвестное signed-char значение не меняет
    /// state, но caller уже прочитал оба входных байта.
    pub(crate) fn set_pk_permission(
        &mut self,
        selector: i8,
        requested: bool,
    ) -> PlayerPkPermissionMutation {
        let previous = self.pk_permissions();
        let recognized = match selector {
            0 => {
                self.base_properties.pk_normal = requested;
                true
            }
            1 => {
                self.base_properties.pk_team = requested;
                true
            }
            2 => {
                self.base_properties.pk_union = requested;
                true
            }
            3 => {
                self.base_properties.pk_badman = requested;
                true
            }
            4 => {
                self.base_properties.pk_country = requested;
                true
            }
            _ => false,
        };
        let current = self.pk_permissions();
        PlayerPkPermissionMutation {
            player_id: self.player_id(),
            selector,
            requested,
            recognized,
            changed: previous != current,
            previous,
            current,
        }
    }

    /// Exact `GetCurBurden`: только equipment, packet и hand, в исходном
    /// wrapping-порядке. Временные auction/fairy/session containers не входят.
    pub(crate) fn current_burden(&self, factory: &CGoodsFactory) -> u32 {
        self.equipment
            .contents_weight(factory)
            .wrapping_add(self.packet.base().contents_weight(factory))
            .wrapping_add(self.hand.contents_weight(factory))
    }

    pub(crate) const fn ci_qing_open(&self) -> bool {
        self.ci_qing_open
    }

    pub(crate) fn ci_qing_list(&self) -> impl ExactSizeIterator<Item = u32> + '_ {
        self.ci_qing_list.iter().copied()
    }

    pub(crate) fn restore_ci_qing_entry(&mut self, base_index: u32) -> bool {
        self.ci_qing_list.insert(base_index)
    }

    pub(crate) fn ci_qing_property_snapshot(
        &self,
    ) -> (&BTreeMap<u32, u32>, &BTreeMap<u32, u32>, u32) {
        (
            &self.ci_qing_add_values,
            &self.ci_qing_tao_zhuang_add_values,
            self.tao_zhuang_id,
        )
    }

    pub(crate) fn apply_ci_qing_property_snapshot(
        &mut self,
        add_values: BTreeMap<u32, u32>,
        tao_zhuang_add_values: BTreeMap<u32, u32>,
        tao_zhuang_id: u32,
    ) {
        self.ci_qing_add_values = add_values;
        self.ci_qing_tao_zhuang_add_values = tao_zhuang_add_values;
        self.tao_zhuang_id = tao_zhuang_id;
    }

    pub(crate) fn ci_qing_property_result(&self) -> BTreeMap<u32, u32> {
        let mut result = self.ci_qing_add_values.clone();
        for (&property, &value) in &self.ci_qing_tao_zhuang_add_values {
            let current = result.entry(property).or_default();
            *current = current.wrapping_add(value);
        }
        result
    }

    pub(crate) fn update_ci_qing_property_difference(
        previous: &BTreeMap<u32, u32>,
        current: &BTreeMap<u32, u32>,
    ) -> Option<BTreeMap<u32, u32>> {
        if previous.len() != current.len() {
            return None;
        }
        let mut destination = BTreeMap::new();
        for ((_, &previous), (&property, &current)) in previous.iter().zip(current) {
            destination.insert(property, current.saturating_sub(previous));
        }
        Some(destination)
    }

    pub(crate) const fn tao_zhuang_is_pending(&self) -> bool {
        self.equipment_changed
    }

    pub(crate) const fn mark_tao_zhuang_pending(&mut self) {
        self.equipment_changed = true;
    }

    pub(crate) const fn tao_zhuang_setup_is_pending(&self) -> bool {
        self.tao_zhuang_setup_pending
    }

    pub(crate) const fn mark_tao_zhuang_setup_sent(&mut self) {
        self.tao_zhuang_setup_pending = false;
    }

    /// `MountAllEquip` собирает только уникальные original-name активных
    /// equipment/CiQing goods; одинаковое имя в двух слотах считается один раз.
    pub(crate) fn rebuild_tao_zhuang_items(
        &mut self,
        setup: &CTaoZhuangSetup,
        factory: &CGoodsFactory,
    ) {
        self.tao_zhuang_items.clear();
        self.tao_zhuang_original_names.clear();
        let mut names = Vec::new();
        for (_, goods) in self.equipment.traversing_goods() {
            if goods.query_attribute(GAP_GOODS_MAXIMUM_DURABILITY)
                && goods.addon_property_value(factory, GAP_GOODS_MAXIMUM_DURABILITY, 2) < 1
            {
                continue;
            }
            if let Some(name) = factory.query_goods_original_name(goods.base_properties_index()) {
                names.push(name.to_vec());
            }
        }
        for position in 0..self.ci_qing.size() {
            let Some(goods) = self.ci_qing.get_goods(position) else {
                continue;
            };
            if goods.query_attribute(GAP_GOODS_MAXIMUM_DURABILITY)
                && goods.addon_property_value(factory, GAP_GOODS_MAXIMUM_DURABILITY, 2) < 1
            {
                continue;
            }
            if let Some(name) = factory.query_goods_original_name(goods.base_properties_index()) {
                names.push(name.to_vec());
            }
        }
        for name in names {
            if self.tao_zhuang_original_names.insert(name.clone())
                && let Some(set_id) = setup.query_id_by_equipment_name(&name)
            {
                let count = self.tao_zhuang_items.entry(set_id).or_default();
                *count = count.wrapping_add(1);
            }
        }
    }

    pub(crate) fn tao_zhuang_skills_for_removal(
        &self,
        setup: &CTaoZhuangSetup,
        factory: &CSkillFactory,
    ) -> Vec<BattleFairySkillRemoved> {
        setup
            .skill_ids()
            .iter()
            .filter_map(|&skill_id| {
                self.move_shape
                    .skill(skill_id, factory)
                    .map(|skill| BattleFairySkillRemoved {
                        message_type: BATTLE_FAIRY_SKILL_REMOVED_MESSAGE_TYPE,
                        player_id: self.player_id(),
                        skill_id,
                        skill_name: skill.name(factory).map(<[u8]>::to_vec),
                    })
            })
            .collect()
    }

    pub(crate) fn delete_tao_zhuang_skill(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> bool {
        self.move_shape.delete_skill(skill_id, factory)
    }

    /// `ComputerAddValue`: каждый достигнутый threshold применяется, а первый
    /// threshold выше collected count немедленно завершает set-prefix.
    pub(crate) fn compute_tao_zhuang_bonuses(
        &mut self,
        setup: &CTaoZhuangSetup,
    ) -> Vec<TaoZhuangSetEvaluation> {
        self.tao_zhuang_properties.clear();
        self.ci_qing_tao_zhuang_properties.clear();
        self.tao_zhuang_skills.clear();
        let mut evaluations = Vec::new();
        for (&set_id, &item_count) in &self.tao_zhuang_items {
            let Some(item) = setup.item(set_id) else {
                continue;
            };
            for addition in item.additions().values() {
                if item_count < addition.number {
                    break;
                }
                for (&skill_id, &level) in &addition.skills {
                    self.tao_zhuang_skills
                        .entry(skill_id)
                        .and_modify(|current| *current = (*current).max(level))
                        .or_insert(level);
                }
                let destination = if set_id < 100 {
                    &mut self.tao_zhuang_properties
                } else {
                    &mut self.ci_qing_tao_zhuang_properties
                };
                for (&property, &value) in &addition.properties {
                    let current = destination.entry(property).or_default();
                    *current = current.wrapping_add(value);
                }
            }
            let collected_all = item.declared_equipment_count == item_count;
            evaluations.push(TaoZhuangSetEvaluation {
                set_id,
                collected_all,
                completion_script: if collected_all {
                    item.script.clone()
                } else {
                    Vec::new()
                },
            });
        }
        evaluations
    }

    pub(crate) const fn set_tao_zhuang_id(&mut self, set_id: u32) {
        self.tao_zhuang_id = set_id;
    }

    pub(crate) fn replace_ci_qing_tao_zhuang_add_values(&mut self, values: BTreeMap<u32, u32>) {
        self.ci_qing_tao_zhuang_add_values = values;
    }

    pub(crate) fn combat_type_values(&self) -> BTreeMap<u32, u32> {
        Self::combat_type_values_from(self.combat_properties)
    }

    pub(crate) fn combat_type_values_from(
        properties: PlayerCombatProperties,
    ) -> BTreeMap<u32, u32> {
        BTreeMap::from([
            (0x0e, properties.minimum_attack),
            (0x0f, properties.maximum_attack),
            (0x10, properties.element_modify as u32),
            (0x11, properties.defense),
            (0x12, u32::from(properties.attack_speed)),
            (0x13, u32::from(properties.hit)),
            (0x14, u32::from(properties.cch)),
            (0x15, u32::from(properties.dodge)),
            (0x17, properties.element_resistance),
            (0x19, u32::from(properties.hp_recovery)),
            (0x1a, u32::from(properties.mp_recovery)),
            (0x1b, properties.strength),
            (0x1c, properties.dexterity),
            (0x1d, properties.constitution),
            (0x1e, properties.intelligence),
            (0x1f, properties.maximum_hp),
            (0x20, properties.maximum_mp),
            (0x33, u32::from(properties.reank)),
            (0x34, u32::from(properties.burden)),
            (0x5b, u32::from(properties.attack_avoid)),
            (0x5c, u32::from(properties.element_avoid)),
            (0x5d, u32::from(properties.full_miss)),
            (0x5f, u32::from(properties.blast_attack)),
            (0x60, u32::from(properties.blast_element_attack)),
        ])
    }

    pub(crate) fn apply_tao_zhuang_properties(
        &mut self,
        ci_qing: bool,
        coefficients: GlobePlayerPropertyCoefficients,
    ) {
        let properties = if ci_qing {
            self.ci_qing_tao_zhuang_properties.clone()
        } else {
            self.tao_zhuang_properties.clone()
        };
        let occupation = usize::from(self.base_properties.occupation).min(2);
        // Оба TaoZhuang owner-а вызывают точный `AddPreItemToPlayer`: x87
        // усекает полную сумму live property и производной signed delta.
        let derived_u32 = |current: u32, value: u32, coefficient: f32| {
            (current as f32 + value as i32 as f32 * coefficient).trunc() as i32 as u32
        };
        let derived_i32 = |current: i32, value: u32, coefficient: f32| {
            (current as f32 + value as i32 as f32 * coefficient).trunc() as i32
        };
        let derived_u16 = |current: u16, value: u32, coefficient: f32| {
            (current as f32 + value as i32 as f32 * coefficient).trunc() as i32 as u16
        };
        for (property, value) in properties {
            match property {
                0x0e => {
                    self.combat_properties.minimum_attack =
                        self.combat_properties.minimum_attack.wrapping_add(value)
                }
                0x0f => {
                    self.combat_properties.maximum_attack =
                        self.combat_properties.maximum_attack.wrapping_add(value)
                }
                0x10 => {
                    self.combat_properties.element_modify = self
                        .combat_properties
                        .element_modify
                        .wrapping_add(value as i32)
                }
                0x11 => {
                    self.combat_properties.defense =
                        self.combat_properties.defense.wrapping_add(value)
                }
                0x12 => {
                    self.combat_properties.attack_speed = self
                        .combat_properties
                        .attack_speed
                        .wrapping_add(value as u16)
                }
                0x13 => {
                    self.combat_properties.hit =
                        self.combat_properties.hit.wrapping_add(value as u16)
                }
                0x14 => {
                    self.combat_properties.cch =
                        self.combat_properties.cch.wrapping_add(value as u16)
                }
                0x15 => {
                    self.combat_properties.dodge =
                        self.combat_properties.dodge.wrapping_add(value as u16)
                }
                0x17 => {
                    self.combat_properties.element_resistance = self
                        .combat_properties
                        .element_resistance
                        .wrapping_add(value)
                }
                0x19 => {
                    self.combat_properties.hp_recovery = self
                        .combat_properties
                        .hp_recovery
                        .wrapping_add(value as u16)
                }
                0x1a => {
                    self.combat_properties.mp_recovery = self
                        .combat_properties
                        .mp_recovery
                        .wrapping_add(value as u16)
                }
                0x1b => {
                    self.combat_properties.strength =
                        self.combat_properties.strength.wrapping_add(value);
                    self.combat_properties.maximum_attack = derived_u32(
                        self.combat_properties.maximum_attack,
                        value,
                        coefficients.str_to_max_attack[occupation],
                    );
                    self.combat_properties.burden = derived_u16(
                        self.combat_properties.burden,
                        value,
                        coefficients.str_to_burden[occupation],
                    );
                }
                0x1c => {
                    self.combat_properties.dexterity =
                        self.combat_properties.dexterity.wrapping_add(value);
                    self.combat_properties.minimum_attack = derived_u32(
                        self.combat_properties.minimum_attack,
                        value,
                        coefficients.dex_to_min_attack[occupation],
                    );
                    self.combat_properties.reank = derived_u16(
                        self.combat_properties.reank,
                        value,
                        coefficients.dex_to_stiff[occupation],
                    );
                }
                0x1d => {
                    self.combat_properties.constitution =
                        self.combat_properties.constitution.wrapping_add(value);
                    self.combat_properties.maximum_hp = derived_u32(
                        self.combat_properties.maximum_hp,
                        value,
                        coefficients.con_to_max_hp[occupation],
                    );
                    self.combat_properties.defense = derived_u32(
                        self.combat_properties.defense,
                        value,
                        coefficients.con_to_defense[occupation],
                    );
                }
                0x1e => {
                    self.combat_properties.intelligence =
                        self.combat_properties.intelligence.wrapping_add(value);
                    self.combat_properties.element_modify = derived_i32(
                        self.combat_properties.element_modify,
                        value,
                        coefficients.int_to_element[occupation],
                    );
                    self.combat_properties.maximum_mp = derived_u32(
                        self.combat_properties.maximum_mp,
                        value,
                        coefficients.int_to_max_mp[occupation],
                    );
                    self.combat_properties.element_resistance = derived_u32(
                        self.combat_properties.element_resistance,
                        value,
                        coefficients.int_to_resistant[occupation],
                    );
                }
                0x1f => {
                    self.combat_properties.maximum_hp =
                        self.combat_properties.maximum_hp.wrapping_add(value)
                }
                0x20 => {
                    self.combat_properties.maximum_mp =
                        self.combat_properties.maximum_mp.wrapping_add(value)
                }
                0x33 => {
                    self.combat_properties.reank =
                        self.combat_properties.reank.wrapping_add(value as u16)
                }
                0x34 => {
                    self.combat_properties.burden =
                        self.combat_properties.burden.wrapping_add(value as u16)
                }
                0x5b => {
                    self.combat_properties.attack_avoid = self
                        .combat_properties
                        .attack_avoid
                        .wrapping_add(value as u16)
                }
                0x5c => {
                    self.combat_properties.element_avoid = self
                        .combat_properties
                        .element_avoid
                        .wrapping_add(value as u16)
                }
                0x5d => {
                    self.combat_properties.full_miss =
                        self.combat_properties.full_miss.wrapping_add(value as u16)
                }
                0x5f => {
                    self.combat_properties.blast_attack = self
                        .combat_properties
                        .blast_attack
                        .wrapping_add(value as u16)
                }
                0x60 => {
                    self.combat_properties.blast_element_attack = self
                        .combat_properties
                        .blast_element_attack
                        .wrapping_add(value as u16)
                }
                _ => {}
            }
        }
        self.sync_combat_property_wire();
    }

    pub(crate) fn add_tao_zhuang_skills(
        &mut self,
        factory: &CSkillFactory,
    ) -> Vec<BattleFairySkillAdded> {
        let player_id = self.player_id();
        let skills = self.tao_zhuang_skills.clone();
        let mut added = Vec::new();
        for (skill_id, level) in skills {
            if self.move_shape.add_skill(skill_id, level as i32, factory)
                && let Some(skill) = self.move_shape.skill(skill_id, factory)
            {
                added.push(battle_fairy_skill_snapshot(player_id, skill, factory));
            }
        }
        added
    }

    pub(crate) fn finish_tao_zhuang_update(&mut self) {
        self.equipment_changed = false;
        self.tao_zhuang_items.clear();
        self.tao_zhuang_original_names.clear();
        self.tao_zhuang_properties.clear();
        self.ci_qing_tao_zhuang_properties.clear();
        self.tao_zhuang_skills.clear();
    }

    pub(crate) fn check_item_in_packet(&self, base_index: u32) -> u32 {
        if base_index == 0 {
            return 0;
        }
        self.packet
            .base()
            .get_goods_by_base_properties(base_index)
            .into_iter()
            .fold(0u32, |total, goods| total.wrapping_add(goods.amount()))
    }

    /// Exact insertion-order `remove_item_in_packet`: каждый stack проходит
    /// через семантику `DeleteGoods(PEI_PACKET, ..., remaining, false)`.
    pub(crate) fn remove_item_in_packet(
        &mut self,
        base_index: u32,
        requested: u32,
    ) -> Vec<CiQingPacketConsumption> {
        if base_index == 0 || requested == 0 {
            return Vec::new();
        }
        let candidates: Vec<_> = self
            .packet
            .base()
            .get_goods_by_base_properties(base_index)
            .into_iter()
            .map(|goods| (goods.identity(), goods.amount()))
            .collect();
        let player_id = self.player_id();
        let mut removed_amount = 0u32;
        let mut consumptions = Vec::new();
        for (identity, previous_amount) in candidates {
            let remaining_request = requested.wrapping_sub(removed_amount);
            if remaining_request == 0 {
                break;
            }
            let position = self
                .packet
                .query_goods_position(identity.ex_id)
                .unwrap_or_default();
            let consumed = previous_amount.min(remaining_request);
            if consumed == 0 {
                continue;
            }
            let remaining_amount = previous_amount.wrapping_sub(consumed);
            let removal = if remaining_amount == 0 {
                self.packet.remove_goods(identity.ex_id)
            } else {
                let position = self.packet.query_goods_position(identity.ex_id);
                if let Some(goods) =
                    position.and_then(|position| self.packet.get_goods_mut(position))
                {
                    goods.set_amount(remaining_amount);
                }
                None
            };
            removed_amount = removed_amount.wrapping_add(consumed);
            consumptions.push(CiQingPacketConsumption {
                player_id,
                goods: identity,
                position,
                previous_amount,
                remaining_amount,
                removal,
            });
        }
        consumptions
    }

    pub(crate) fn remove_packet_goods_by_id(
        &mut self,
        goods_id: CGuid,
        requested: u32,
    ) -> Option<CiQingPacketConsumption> {
        if requested == 0 {
            return None;
        }
        let goods = self.packet.base().find(goods_id)?;
        let identity = goods.identity();
        let position = self.packet.query_goods_position(goods_id)?;
        let previous_amount = goods.amount();
        let consumed = previous_amount.min(requested);
        let remaining_amount = previous_amount.wrapping_sub(consumed);
        let removal = if remaining_amount == 0 {
            self.packet.remove_goods(goods_id)
        } else {
            let position = self.packet.query_goods_position(goods_id)?;
            self.packet
                .get_goods_mut(position)?
                .set_amount(remaining_amount);
            None
        };
        Some(CiQingPacketConsumption {
            player_id: self.player_id(),
            goods: identity,
            position,
            previous_amount,
            remaining_amount,
            removal,
        })
    }

    pub(crate) fn add_packet_goods_at(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> PlayerPacketAddOutcome {
        let outcome =
            self.packet
                .add_goods_at(position, incoming, factory, owner_progress_allows);
        self.notify_packet_goods_added(&outcome, factory, on_goods_added);
        PlayerPacketAddOutcome { outcome }
    }

    pub(crate) fn add_packet_goods(
        &mut self,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> PlayerPacketAddOutcome {
        let outcome = self
            .packet
            .add_goods(incoming, factory, owner_progress_allows);
        self.notify_packet_goods_added(&outcome, factory, on_goods_added);
        PlayerPacketAddOutcome { outcome }
    }

    pub(crate) fn swap_packet_goods(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> Option<VolumeGoodsSwapOutcome> {
        CVolumeLimitGoodsContainer::swap_goods_with_owner(
            self,
            position,
            incoming,
            owner_progress_allows,
            |player| &mut player.packet,
            |player, position, incoming, owner_progress_allows| {
                player
                    .add_packet_goods_at(
                        position,
                        incoming,
                        factory,
                        owner_progress_allows,
                        on_goods_added,
                    )
                    .outcome
            },
        )
    }

    fn notify_packet_goods_added(
        &mut self,
        outcome: &VolumeGoodsAddOutcome,
        factory: &CGoodsFactory,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) {
        let VolumeGoodsAddOutcome::Added(added) = outcome else {
            return;
        };
        let current_ticket = self.current_ticket;
        let registration = self
            .packet
            .base_mut()
            .find_mut(added.identity.ex_id)
            .and_then(|goods| {
                Self::prepare_goods_ai_registration_with_clock(
                    current_ticket,
                    goods,
                    factory,
                    &mut crate::gameserver::gameserver::game::game_wall_time_seconds,
                )
            });
        if let Some((ticket, goods_id)) = registration {
            self.record_goods_ai_registration(ticket, goods_id);
        }
        if let Some(goods) = self.packet.base().find(added.identity.ex_id) {
            let additional = goods.addon_property_value(factory, GAP_EXCEPTION_STATE, 1) as u32;
            on_goods_added(self, additional);
        }
    }

    /// Player-side `AddGoodsToPacket`: успешный add забирает ownership из
    /// входного vector, rejected/несовместимый stack остаётся у caller-а.
    pub(crate) fn add_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        let owner_progress_allows = self.current_progress == PlayerProgress::None;
        self.add_goods_to_packet_with_progress(
            goods,
            factory,
            encode_old_client,
            owner_progress_allows,
            on_goods_added,
        )
    }

    /// Billing Increment response приходит при `PROGRESS_INCREMENT`, но
    /// исходный `BillOfIncShop` добавляет batch напрямую в packet и не
    /// применяет progress-lock к stack merge.
    pub(crate) fn add_increment_shop_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        self.add_goods_to_packet_with_progress(
            goods, factory, encode_old_client, true, on_goods_added,
        )
    }

    /// Script `2249` выполняется при занятом script progress, но native owner
    /// после созревания добавляет replacement напрямую в packet.
    pub(crate) fn add_script_fairy_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        self.add_goods_to_packet_with_progress(
            goods, factory, encode_old_client, true, on_goods_added,
        )
    }

    /// `GetPreciousItem` исполняется внутри script progress, но native owner
    /// также добавляет награду напрямую и не применяет ordinary progress-lock.
    pub(crate) fn add_precious_box_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        self.add_goods_to_packet_with_progress(
            goods, factory, encode_old_client, true, on_goods_added,
        )
    }

    /// `CTrader::Trade` добавляет contrary goods при
    /// `PROGRESS_TRADING`; этот owner намеренно обходит общий progress-lock,
    /// как прямой packet `Add` исходной функции.
    pub(crate) fn add_traded_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        self.add_goods_to_packet_with_progress(
            goods, factory, encode_old_client, true, on_goods_added,
        )
    }

    /// NPC shop добавляет batch напрямую при `PROGRESS_SHOPPING`.
    pub(crate) fn add_shop_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        self.add_goods_to_packet_with_progress(
            goods, factory, encode_old_client, true, on_goods_added,
        )
    }

    /// Обратная половина `CTrader::RollBack`: отменяет уже выполненный
    /// contrary packet add, включая direct stack merge, и возвращает client
    /// consumption fact. Сам исходный goods caller хранит отдельно до commit.
    pub(crate) fn rollback_traded_packet_addition(
        &mut self,
        addition: &CiQingPacketAddition,
        original_amount: u32,
    ) -> Option<CiQingPacketConsumption> {
        let position = addition.position?;
        match &addition.outcome {
            VolumeGoodsAddOutcome::Added(added) => {
                let removed = self.packet.remove_goods(added.identity.ex_id)?;
                let taken = match removed {
                    VolumeGoodsRemoveOutcome::Removed(taken)
                    | VolumeGoodsRemoveOutcome::RemovedButCellMissing(taken) => taken,
                };
                let removed = match taken {
                    AmountLimitGoodsTaken::Removed(removed) => removed,
                    AmountLimitGoodsTaken::Split(_) => return None,
                };
                Some(CiQingPacketConsumption {
                    player_id: self.player_id(),
                    goods: removed.goods.identity(),
                    position,
                    previous_amount: removed.amount,
                    remaining_amount: 0,
                    removal: None,
                })
            }
            VolumeGoodsAddOutcome::Stack(
                super::container::cgoodscontainer::GoodsStackMergeOutcome::Merged {
                    target,
                    amount,
                },
            ) if *amount == original_amount => {
                let goods = self.packet.get_goods_mut(position)?;
                if goods.identity() != *target || goods.amount() < original_amount {
                    return None;
                }
                let previous_amount = goods.amount();
                let remaining_amount = previous_amount.wrapping_sub(original_amount);
                goods.set_amount(remaining_amount);
                Some(CiQingPacketConsumption {
                    player_id: self.player_id(),
                    goods: *target,
                    position,
                    previous_amount,
                    remaining_amount,
                    removal: None,
                })
            }
            _ => None,
        }
    }

    fn add_goods_to_packet_with_progress(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
        owner_progress_allows: bool,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        let player_id = self.player_id();
        let mut additions = Vec::new();
        let mut remaining = Vec::new();
        for goods in goods {
            let source = goods.identity();
            let mut incoming = Some(goods);
            let packet_add = self.add_packet_goods(
                &mut incoming, factory, owner_progress_allows, on_goods_added,
            );
            let outcome = packet_add.outcome;
            let (old_client_payload, resulting_amount) = match &outcome {
                VolumeGoodsAddOutcome::Added(added) => {
                    let stored = self
                        .packet
                        .base()
                        .find(added.identity.ex_id)
                        .expect("успешный packet add сохранил новый goods");
                    let payload = Some(encode_old_client(stored));
                    let amount = Some(stored.amount());
                    (payload, amount)
                }
                VolumeGoodsAddOutcome::Stack(stack) => {
                    let target = match stack {
                        super::container::cgoodscontainer::GoodsStackMergeOutcome::Merged {
                            target,
                            ..
                        } => self.packet.base().find(target.ex_id),
                        _ => None,
                    };
                    (None, target.map(CGoods::amount))
                }
                VolumeGoodsAddOutcome::Rejected(_) => (None, None),
            };
            let position = match &outcome {
                VolumeGoodsAddOutcome::Added(added) => added.position,
                VolumeGoodsAddOutcome::Stack(
                    super::container::cgoodscontainer::GoodsStackMergeOutcome::Merged {
                        target,
                        ..
                    },
                ) => self.packet.query_goods_position(target.ex_id),
                _ => None,
            };
            additions.push(CiQingPacketAddition {
                player_id,
                source,
                position,
                outcome,
                old_client_payload,
                resulting_amount,
            });
            if let Some(goods) = incoming {
                remaining.push(goods);
            }
        }
        (additions, remaining)
    }

    pub(crate) fn ci_qing_compose_goods(&self, position: u32) -> Option<&CGoods> {
        self.ci_qing_compose.get_goods(position)
    }

    pub(crate) fn ci_qing_goods(&self, position: u32) -> Option<&CGoods> {
        self.ci_qing.get_goods(position)
    }

    pub(crate) fn ci_qing_goods_amount(&self, factory: &CGoodsFactory) -> u32 {
        self.ci_qing.goods_amount(factory)
    }

    pub(crate) fn add_goods_to_ci_qing(
        &mut self,
        goods: CGoods,
        position: u32,
        compose_container: bool,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> (CiQingContainerAddition, Option<CGoods>) {
        let player_id = self.player_id();
        let source = goods.identity();
        let container_extend_id = if compose_container { 17 } else { 16 };
        let container = if compose_container {
            &mut self.ci_qing_compose
        } else {
            &mut self.ci_qing
        };
        let mut incoming = Some(goods);
        let outcome = container.add_goods_at(
            position,
            &mut incoming,
            factory,
            self.current_progress == PlayerProgress::None,
        );
        let (old_client_payload, resulting_amount) = match &outcome {
            VolumeGoodsAddOutcome::Added(added) => {
                let stored = container
                    .base()
                    .find(added.identity.ex_id)
                    .expect("успешный CiQing add сохранил goods");
                (Some(encode_old_client(stored)), Some(stored.amount()))
            }
            VolumeGoodsAddOutcome::Stack(stack) => {
                let target = match stack {
                    super::container::cgoodscontainer::GoodsStackMergeOutcome::Merged {
                        target,
                        ..
                    } => container.base().find(target.ex_id),
                    _ => None,
                };
                (None, target.map(CGoods::amount))
            }
            VolumeGoodsAddOutcome::Rejected(_) => (None, None),
        };
        (
            CiQingContainerAddition {
                player_id,
                container_extend_id,
                position,
                source,
                outcome,
                old_client_payload,
                resulting_amount,
            },
            incoming,
        )
    }

    /// Ownership-ветвь generic `CC2SContainerObjectMove` для временного
    /// compose-контейнера. В отличие от CiQing gameplay packets здесь не
    /// нужен old-client object stream: итоговый `0xC0101` несёт только GUID
    /// и amount, но positional/automatic Add и listener state остаются у
    /// concrete container owner-а.
    pub(crate) fn add_ci_qing_compose_transfer_goods(
        &mut self,
        incoming: &mut Option<CGoods>,
        position: u32,
        factory: &CGoodsFactory,
    ) -> CiQingContainerAddition {
        let player_id = self.player_id();
        let source = incoming
            .as_ref()
            .expect("CiQing compose transfer add получает detached goods")
            .identity();
        let outcome = if position == u32::MAX {
            self.ci_qing_compose.add_goods(
                incoming,
                factory,
                self.current_progress == PlayerProgress::None,
            )
        } else {
            self.ci_qing_compose.add_goods_at(
                position,
                incoming,
                factory,
                self.current_progress == PlayerProgress::None,
            )
        };
        let (actual_position, resulting_amount) = match &outcome {
            VolumeGoodsAddOutcome::Added(added) => {
                let actual_position = added.position.unwrap_or(position);
                let amount = self
                    .ci_qing_compose
                    .get_goods(actual_position)
                    .map(CGoods::amount);
                (actual_position, amount)
            }
            VolumeGoodsAddOutcome::Stack(
                super::container::cgoodscontainer::GoodsStackMergeOutcome::Merged {
                    target, ..
                },
            ) => {
                let actual_position = self
                    .ci_qing_compose
                    .query_goods_position(target.ex_id)
                    .unwrap_or(position);
                let amount = self
                    .ci_qing_compose
                    .get_goods(actual_position)
                    .map(CGoods::amount);
                (actual_position, amount)
            }
            _ => (position, None),
        };
        CiQingContainerAddition {
            player_id,
            container_extend_id: 17,
            position: actual_position,
            source,
            outcome,
            old_client_payload: None,
            resulting_amount,
        }
    }

    pub(crate) fn take_ci_qing_compose_transfer_goods<Create>(
        &mut self,
        position: u32,
        requested_amount: u32,
        factory: &CGoodsFactory,
        create_goods: Create,
    ) -> Option<VolumeGoodsRemoveOutcome>
    where
        Create: FnMut(u32) -> Option<CGoods>,
    {
        self.ci_qing_compose
            .take_goods(position, requested_amount, factory, create_goods)
    }

    pub(crate) fn remove_ci_qing_compose_goods(
        &mut self,
        position: u32,
    ) -> Option<CiQingContainerConsumption> {
        let goods = self.ci_qing_compose.get_goods(position)?;
        let identity = goods.identity();
        let amount = goods.amount();
        let removal = self.ci_qing_compose.remove_goods(identity.ex_id)?;
        Some(CiQingContainerConsumption {
            player_id: self.player_id(),
            container_extend_id: 17,
            position,
            goods: identity,
            previous_amount: amount,
            remaining_amount: 0,
            removal: Some(removal),
        })
    }

    pub(crate) fn remove_ci_qing_goods(
        &mut self,
        position: u32,
        requested: u32,
    ) -> Option<CiQingContainerConsumption> {
        let goods = self.ci_qing.get_goods(position)?;
        let identity = goods.identity();
        let previous_amount = goods.amount();
        let consumed = previous_amount.min(requested);
        if consumed == 0 {
            return None;
        }
        let remaining_amount = previous_amount.wrapping_sub(consumed);
        let removal = if remaining_amount == 0 {
            self.ci_qing.remove_goods(identity.ex_id)
        } else {
            self.ci_qing
                .get_goods_mut(position)
                .map(|goods| goods.set_amount(remaining_amount));
            None
        };
        Some(CiQingContainerConsumption {
            player_id: self.player_id(),
            container_extend_id: 16,
            position,
            goods: identity,
            previous_amount,
            remaining_amount,
            removal,
        })
    }

    pub(crate) fn ci_qing_hand_goods(&self) -> Option<&CGoods> {
        self.hand.get_goods(0)
    }

    pub(crate) fn hotkey(&self, slot: u8) -> Option<u32> {
        self.base_properties.hotkeys.get(usize::from(slot)).copied()
    }

    pub(crate) fn set_hotkey(&mut self, slot: u8, value: u32) -> bool {
        let Some(hotkey) = self.base_properties.hotkeys.get_mut(usize::from(slot)) else {
            return false;
        };
        *hotkey = value;
        true
    }

    pub(crate) const fn last_operated_goods(&self) -> (u32, u32) {
        (
            self.last_operated_container,
            self.last_operated_goods_position,
        )
    }

    pub(crate) fn record_last_operated_goods(
        &mut self,
        source_extend_id: i32,
        source_position: u32,
    ) -> (u32, u32) {
        let previous = self.last_operated_goods();
        self.last_operated_container = source_extend_id as u32;
        self.last_operated_goods_position = source_position;
        previous
    }

    /// Storage core назначения hotkey из hand. Packet/hand/wallet/YuanBao
    /// замкнуты на owned containers; equipment для consumable доказательно
    /// отвергается до mutation.
    pub(crate) fn return_hotkey_hand_goods(
        &mut self,
        factory: &CGoodsFactory,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> HotkeyHandTransferReport {
        let (source_container_extend_id, source_position) = self.last_operated_goods();
        let mut report = HotkeyHandTransferReport {
            source_container_extend_id,
            source_position,
            goods: None,
            hand_removal: None,
            packet_adds: Vec::new(),
            currency_adds: Vec::new(),
            hand_rollback: None,
            outcome: HotkeyHandTransferOutcome::MissingHandGoods,
        };
        let Some(hand_goods) = self.hand.get_goods(0) else {
            return report;
        };
        report.goods = Some(hand_goods.identity());
        let Some(properties) =
            factory.query_goods_base_properties(hand_goods.base_properties_index())
        else {
            report.outcome = HotkeyHandTransferOutcome::NotConsumable;
            return report;
        };
        if properties.goods_type() != GOODS_TYPE_CONSUMABLE {
            report.outcome = HotkeyHandTransferOutcome::NotConsumable;
            return report;
        }
        if !(1..=5).contains(&source_container_extend_id) {
            report.outcome = HotkeyHandTransferOutcome::UnsupportedSource;
            return report;
        }

        let removed = self
            .hand
            .remove_goods(hand_goods.identity().ex_id)
            .expect("unlocked hand goods проверен перед synchronous remove");
        report.hand_removal = Some(HotkeyHandOwnershipEvent {
            owner_type: removed.owner_type,
            owner_id: removed.owner_id,
            position: removed.position,
            amount: removed.amount,
            listeners: removed.listeners.clone(),
        });
        let mut incoming = Some(removed.goods);
        if source_container_extend_id == 1 {
            let owner_progress_allows = self.current_progress == PlayerProgress::None;
            report.packet_adds.push(self.add_packet_goods_at(
                source_position,
                &mut incoming,
                factory,
                owner_progress_allows,
                on_goods_added,
            ));
            if incoming.is_some() {
                report.packet_adds.push(self.add_packet_goods(
                    &mut incoming,
                    factory,
                    owner_progress_allows,
                    on_goods_added,
                ));
            }
        } else if source_container_extend_id == 3 {
            let goods = incoming.take().expect("removed hand goods остаётся owned");
            match self.hand.add_goods(goods, factory) {
                Ok(added) => report.hand_rollback = Some(added),
                Err(goods) => incoming = Some(goods),
            }
        } else if source_container_extend_id == 4 {
            let owner_progress_allows = self.current_progress == PlayerProgress::None;
            report.currency_adds.push(self.wallet.add_goods(
                source_position,
                &mut incoming,
                factory,
                owner_progress_allows,
            ));
            if incoming.is_some() {
                report.currency_adds.push(self.wallet.add_goods(
                    0,
                    &mut incoming,
                    factory,
                    owner_progress_allows,
                ));
            }
        } else if source_container_extend_id == 5 {
            let owner_progress_allows = self.current_progress == PlayerProgress::None;
            report.currency_adds.push(self.yuan_bao.add_goods(
                source_position,
                &mut incoming,
                factory,
                owner_progress_allows,
            ));
            if incoming.is_some() {
                report.currency_adds.push(self.yuan_bao.add_goods(
                    0,
                    &mut incoming,
                    factory,
                    owner_progress_allows,
                ));
            }
        }

        if incoming.is_none() {
            if source_container_extend_id == 4 {
                self.money = self.wallet.currency_amount();
            }
            report.outcome = HotkeyHandTransferOutcome::Moved;
            return report;
        }
        let goods = incoming
            .take()
            .expect("failed destination сохраняет incoming");
        match self.hand.add_goods(goods, factory) {
            Ok(added) => {
                report.hand_rollback = Some(added);
                report.outcome = HotkeyHandTransferOutcome::RolledBack;
            }
            Err(goods) => {
                report.goods = Some(goods.identity());
                drop(goods);
                report.outcome = HotkeyHandTransferOutcome::GarbageCollected;
            }
        }
        report
    }

    pub(crate) fn destroy_hand_goods(
        &mut self,
        goods_id: CGuid,
        requested: u32,
    ) -> Option<GoodsDestroyHandConsumption> {
        if requested == 0 {
            return None;
        }
        let goods = self.hand.find(goods_id)?;
        let identity = goods.identity();
        let previous_amount = goods.amount();
        let removed_amount = previous_amount.min(requested);
        let remaining_amount = previous_amount.wrapping_sub(removed_amount);
        let removal = if remaining_amount == 0 {
            self.hand.remove_goods(goods_id)
        } else {
            self.hand.find_mut(goods_id)?.set_amount(remaining_amount);
            None
        };
        Some(GoodsDestroyHandConsumption {
            player_id: self.player_id(),
            goods: identity,
            previous_amount,
            removed_amount,
            remaining_amount,
            removal,
        })
    }

    pub(crate) fn remove_ci_qing_hand_goods(&mut self) -> Option<CiQingHandConsumption> {
        let goods = self.hand.get_goods(0)?;
        let identity = goods.identity();
        let previous_amount = goods.amount();
        if previous_amount == 0 {
            return None;
        }
        let remaining_amount = previous_amount.wrapping_sub(1);
        let removal = if remaining_amount == 0 {
            self.hand.remove_goods(identity.ex_id)
        } else {
            self.hand.find_mut(identity.ex_id).map(|goods| {
                goods.set_amount(remaining_amount);
            });
            None
        };
        Some(CiQingHandConsumption {
            player_id: self.player_id(),
            goods: identity,
            previous_amount,
            remaining_amount,
            removal,
        })
    }

    pub(crate) fn ci_qing_mount_facts(&self, factory: &CGoodsFactory) -> Option<(u32, u32, u32)> {
        let goods = self.ci_qing_hand_goods()?;
        if goods.addon_property_value(factory, GAP_ROLE_MINIMUM_LEVEL_LIMIT, 1)
            > i32::from(self.level())
        {
            return None;
        }
        Some((
            goods.addon_property_value(factory, GAP_CIQING_PROPERTY1, 1) as u32,
            goods.addon_property_value(factory, GAP_CIQING_PROPERTY1, 2) as u32,
            goods.addon_property_value(factory, GAP_CIQING_PROPERTY2, 1) as u32,
        ))
    }

    pub(crate) const fn contend_state(&self) -> bool {
        self.contend_state
    }

    pub(crate) fn apply_client_direction(&mut self, direction: u8) -> i32 {
        self.move_shape
            .shape_mut()
            .set_direction(i32::from(direction));
        self.move_shape.shape().get_direction()
    }

    pub(crate) fn clear_emotion_state(&mut self) {
        self.emotion_index = 0;
        self.emotion_timestamp_ms = 0;
    }

    /// Exact reached `PerformEmotion` state: state очищается до guards;
    /// repeated emotion запоминается, но around publication выполняется для
    /// любого разрешённого AI/жизни вызова.
    pub(crate) fn perform_emotion_state(
        &mut self,
        emotion_id: i32,
        repeated: bool,
        now_ms: u32,
        ai_available: bool,
        ai_has_target: bool,
    ) -> bool {
        self.clear_emotion_state();
        if self.is_dead() || !ai_available || ai_has_target {
            return false;
        }
        if repeated {
            self.emotion_index = emotion_id;
            self.emotion_timestamp_ms = now_ms;
        }
        true
    }

    /// State-часть exact `SetContendState`: unchanged setter не публикуется.
    /// `0xBFF28` собирает и маршрутизирует caller после успешной мутации.
    pub(crate) const fn set_contend_state(&mut self, contend_state: bool) -> bool {
        if self.contend_state == contend_state {
            return false;
        }
        self.contend_state = contend_state;
        true
    }

    pub(crate) const fn city_war_died_state_time_ms(&self) -> i32 {
        self.city_war_died_state_time_ms
    }

    pub(crate) const fn city_war_died_state(&self) -> bool {
        self.city_war_died_state
    }

    /// Script `9313` читает этот byte напрямую без вычисления аргументов.
    pub(crate) const fn is_nation_war_player_weak(&self) -> bool {
        self.city_war_died_state
    }

    pub(crate) const fn died_state_start_time_ms(&self) -> u32 {
        self.died_state_start_time_ms
    }

    /// Direct assignment из `OnDied`: original не вызывает setter и поэтому
    /// не посылает `0xBFF2B`; clock стартует только для positive duration.
    pub(crate) const fn begin_city_war_death_countdown(&mut self, duration_ms: i32, now_ms: u32) {
        self.city_war_died_state_time_ms = duration_ms;
        if duration_ms > 0 {
            self.died_state_start_time_ms = now_ms;
        }
    }

    /// Достигнутый decode-tail восстановления player: persisted duration
    /// запускает новый local clock, а action `ACT_DIED == 6` оставляет state
    /// выключенным до `OnRelive`.
    pub(crate) fn restore_city_war_death_countdown(&mut self, duration_ms: i32, now_ms: u32) {
        self.city_war_died_state_time_ms = duration_ms;
        if duration_ms > 0 {
            self.died_state_start_time_ms = now_ms;
            if self.shape().get_action() != 6 {
                self.city_war_died_state = true;
            }
        }
    }

    /// State-часть exact `SetCityWarDiedStateTime`; caller использует return
    /// как gate `0xBFF2B`, который допустим лишь при active died state.
    pub(crate) const fn set_city_war_died_state_time_ms(&mut self, time_ms: i32) -> bool {
        if self.city_war_died_state_time_ms == time_ms {
            return false;
        }
        self.city_war_died_state_time_ms = time_ms;
        self.city_war_died_state
    }

    /// Exact `SetCityWarDiedState` всегда пишет state и всегда публикует обе
    /// копии `0xBFF2A`, даже если значение не изменилось.
    pub(crate) const fn set_city_war_died_state(&mut self, died_state: bool) {
        self.city_war_died_state = died_state;
    }

    pub(crate) const fn restart_died_state_clock(&mut self, now_ms: u32) {
        self.died_state_start_time_ms = now_ms;
    }

    /// Exact counter-prefix `CPlayer::PeriodicalUpdate`: исторический `long`
    /// увеличивается до строгого порога `250`; caller только после этого
    /// снимает `timeGetTime`, обнуляет counter и публикует `0xBF809`.
    pub(crate) const fn advance_periodical_ping(&mut self) -> i32 {
        self.ping_time = self.ping_time.wrapping_add(1);
        self.ping_time
    }

    pub(crate) const fn complete_periodical_ping(&mut self, now_ms: u32) {
        self.ping_time = 0;
        self.last_ping_time_ms = now_ms;
    }

    pub(crate) const fn contribution(&self) -> i32 {
        self.contribution
    }

    pub(crate) const fn money(&self) -> u32 {
        self.money
    }

    pub(crate) fn bank_transfer_currency_goods(&self, extend_id: i32) -> Option<&CGoods> {
        match extend_id {
            4 => self.wallet.get_goods(0),
            8 => self.bank.get_goods(0),
            15 => self.auction_wallet.get_goods(0),
            _ => None,
        }
    }

    pub(crate) fn take_bank_transfer_currency_goods<Create>(
        &mut self,
        extend_id: i32,
        requested: u32,
        factory: &CGoodsFactory,
        mut create_goods: Create,
    ) -> Option<CurrencyGoodsTaken>
    where
        Create: FnMut(u32) -> Option<CGoods>,
    {
        let taken = match extend_id {
            4 => self
                .wallet
                .take_goods(0, requested, factory, &mut create_goods),
            8 => self
                .bank
                .take_goods(0, requested, factory, &mut create_goods),
            15 => self
                .auction_wallet
                .take_goods(0, requested, factory, &mut create_goods),
            _ => None,
        };
        if extend_id == 4 {
            self.money = self.wallet.currency_amount();
        }
        taken
    }

    pub(crate) fn add_bank_transfer_currency_goods(
        &mut self,
        extend_id: i32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
    ) -> Option<PlayerBankCurrencyAddOutcome> {
        let owner_progress_allows = !matches!(
            self.current_progress,
            PlayerProgress::OpenStall | PlayerProgress::Trading | PlayerProgress::Upgrade
        );
        let outcome = match extend_id {
            4 => PlayerBankCurrencyAddOutcome::Wallet(self.wallet.add_goods(
                0,
                incoming,
                factory,
                owner_progress_allows,
            )),
            8 => PlayerBankCurrencyAddOutcome::Bank(self.bank.add_goods(
                0,
                incoming,
                factory,
                owner_progress_allows,
            )),
            15 => PlayerBankCurrencyAddOutcome::AuctionWallet(self.auction_wallet.add_goods(
                0,
                incoming,
                factory,
                owner_progress_allows,
            )),
            _ => return None,
        };
        if extend_id == 4 {
            self.money = self.wallet.currency_amount();
        }
        Some(outcome)
    }

    pub(crate) fn ground_currency_goods(&self, extend_id: i32) -> Option<&CGoods> {
        match extend_id {
            4 => self.wallet.get_goods(0),
            5 => self.yuan_bao.get_goods(0),
            _ => None,
        }
    }

    pub(crate) fn take_ground_currency_goods<Create>(
        &mut self,
        extend_id: i32,
        requested: u32,
        factory: &CGoodsFactory,
        mut create_goods: Create,
    ) -> Option<CurrencyGoodsTaken>
    where
        Create: FnMut(u32) -> Option<CGoods>,
    {
        let taken = match extend_id {
            4 => self
                .wallet
                .take_goods(0, requested, factory, &mut create_goods),
            5 => self
                .yuan_bao
                .take_goods(0, requested, factory, &mut create_goods),
            _ => None,
        };
        if extend_id == 4 {
            self.money = self.wallet.currency_amount();
        }
        taken
    }

    pub(crate) fn add_ground_currency_goods(
        &mut self,
        extend_id: i32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> Option<CurrencyGoodsAddOutcome> {
        let outcome = match extend_id {
            4 => self
                .wallet
                .add_goods(0, incoming, factory, owner_progress_allows),
            5 => self
                .yuan_bao
                .add_goods(0, incoming, factory, owner_progress_allows),
            _ => return None,
        };
        if extend_id == 4 {
            self.money = self.wallet.currency_amount();
        }
        Some(outcome)
    }

    pub(crate) fn decrease_money(
        &mut self,
        requested: u32,
        factory: &CGoodsFactory,
    ) -> PlayerMoneyDecrease {
        let previous = self.wallet.currency_amount();
        let outcome = self.wallet.decrease_currency(requested, factory);
        self.money = self.wallet.currency_amount();
        PlayerMoneyDecrease {
            previous,
            current: self.money,
            outcome,
        }
    }

    pub(crate) fn increase_money(
        &mut self,
        requested: u32,
        factory: &CGoodsFactory,
        created_currency: Vec<CGoods>,
    ) -> super::container::cwallet::CurrencyIncreaseOutcome {
        let mut created_currency = Some(created_currency);
        let outcome = self
            .wallet
            .increase_currency(requested, factory, move |_, _| {
                created_currency.take().unwrap_or_default()
            });
        self.money = self.wallet.currency_amount();
        outcome
    }

    pub(crate) fn yuan_bao(&self) -> u32 {
        self.yuan_bao.currency_amount()
    }

    /// State-owner exact `SetYuanBao`: однослотовый currency container
    /// сохраняет create/increase/decrease/delete outcome для сетевого caller-а.
    pub(crate) fn set_yuan_bao(
        &mut self,
        current: u32,
        factory: &CGoodsFactory,
        created_currency: Vec<CGoods>,
    ) -> PlayerYuanBaoChange {
        let previous = self.yuan_bao.currency_amount();
        let outcome = if previous < current {
            let mut created_currency = Some(created_currency);
            PlayerYuanBaoChangeOutcome::Increased(self.yuan_bao.increase_currency(
                current.wrapping_sub(previous),
                factory,
                move |_, _| created_currency.take().unwrap_or_default(),
            ))
        } else if current < previous {
            PlayerYuanBaoChangeOutcome::Decreased(
                self.yuan_bao
                    .decrease_currency(previous.wrapping_sub(current), factory),
            )
        } else {
            PlayerYuanBaoChangeOutcome::Unchanged
        };
        PlayerYuanBaoChange {
            player_id: self.player_id(),
            previous,
            current: self.yuan_bao.currency_amount(),
            outcome,
        }
    }

    pub(crate) const fn client_ip(&self) -> u32 {
        self.client_ip
    }

    pub(crate) const fn set_client_ip_snapshot(&mut self, client_ip: u32) {
        self.client_ip = client_ip;
    }

    pub(crate) fn depot_money(&self) -> u32 {
        self.bank.gold_coins_amount()
    }

    pub(crate) const fn pk_count(&self) -> u16 {
        self.base_properties.pk_count
    }

    pub(crate) const fn kill_count(&self) -> u32 {
        self.base_properties.kill_count
    }

    pub(crate) const fn criminal_state_timestamp_ms(&self) -> u32 {
        self.criminal_state_timestamp_ms
    }

    pub(crate) const fn add_murder_kill_count(&mut self, amount: u32) -> u32 {
        self.base_properties.kill_count = self.base_properties.kill_count.wrapping_add(amount);
        self.base_properties.kill_count
    }

    /// `CPKSys::ReportMurderer` добавляет только PK count; kill count уже
    /// увеличен `OnBeenMurdered` перед вызовом policy.
    pub(crate) fn report_murderer(
        &mut self,
        pk_count_per_kill: u32,
        now_ms: impl FnOnce() -> u32,
    ) -> u16 {
        self.base_properties.pk_count = u32::from(self.base_properties.pk_count)
            .saturating_add(pk_count_per_kill)
            .min(u32::from(u16::MAX)) as u16;
        let murderer_timestamp_started =
            self.base_properties.pk_count != 0 && self.murderer_time_stamp_ms == 0;
        if murderer_timestamp_started {
            self.murderer_time_stamp_ms = now_ms();
        }
        tracing::trace!(
            player_id = self.player_id(),
            pk_count = self.base_properties.pk_count,
            kill_count = self.base_properties.kill_count,
            murderer_timestamp_started,
            "убийца зарегистрирован"
        );
        self.base_properties.pk_count
    }

    pub(crate) fn is_badman(&self, pk_count_per_kill: u32) -> bool {
        u32::from(self.base_properties.pk_count) > pk_count_per_kill
            || self.criminal_state_timestamp_ms != 0
    }

    /// Exact scalar часть `EnterCriminalState`: PK threshold проверяется до
    /// clock; повторный вход обновляет timestamp, но не требует around-wire.
    pub(crate) fn enter_criminal_state(
        &mut self,
        pk_count_per_kill: u32,
        now_ms: impl FnOnce() -> u32,
    ) -> Option<bool> {
        if u32::from(self.base_properties.pk_count) > pk_count_per_kill {
            return None;
        }
        let started = self.criminal_state_timestamp_ms == 0;
        self.criminal_state_timestamp_ms = now_ms();
        Some(started)
    }

    pub(crate) fn reset_murder_counters(&mut self) {
        let previous_pk_count = self.base_properties.pk_count;
        let previous_kill_count = self.base_properties.kill_count;
        self.base_properties.pk_count = 0;
        self.base_properties.kill_count = 0;
        tracing::trace!(
            player_id = self.player_id(),
            previous_pk_count,
            previous_kill_count,
            "счётчики убийств игрока сброшены"
        );
    }

    /// World kill confirmation tail: unsigned saturation, wrapping kill count
    /// и `OnUpdateMurdererSign` с единственным clock sample при старте timer-а.
    pub(crate) fn apply_confirmed_kill(
        &mut self,
        pk_count_per_kill: u32,
        now_ms: impl FnOnce() -> u32,
    ) -> (u16, u32) {
        self.base_properties.pk_count = u32::from(self.base_properties.pk_count)
            .saturating_add(pk_count_per_kill)
            .min(u32::from(u16::MAX)) as u16;
        self.base_properties.kill_count = self.base_properties.kill_count.wrapping_add(1);
        let murderer_timestamp_started =
            self.base_properties.pk_count != 0 && self.murderer_time_stamp_ms == 0;
        if self.base_properties.pk_count == 0 {
            self.murderer_time_stamp_ms = 0;
        } else if murderer_timestamp_started {
            self.murderer_time_stamp_ms = now_ms();
        }
        tracing::trace!(
            player_id = self.player_id(),
            pk_count = self.base_properties.pk_count,
            kill_count = self.base_properties.kill_count,
            murderer_timestamp_started,
            "подтверждённое убийство применено"
        );
        (
            self.base_properties.pk_count,
            self.base_properties.kill_count,
        )
    }

    /// Exact `OnDecreaseMurdererSign`: timer идёт только у живого murderer-а,
    /// сравнение сохраняет DWORD addition/order, а при оставшемся PK исходник
    /// повторно читает `timeGetTime` для начала следующего интервала.
    pub(crate) fn decrease_murderer_sign(
        &mut self,
        one_pk_count_time_ms: u32,
        mut now_ms: impl FnMut() -> u32,
    ) -> Option<PlayerMurdererSignDecrease> {
        if self.is_dead() || self.base_properties.pk_count == 0 {
            return None;
        }
        let checked_at_ms = now_ms();
        if self
            .murderer_time_stamp_ms
            .wrapping_add(one_pk_count_time_ms)
            > checked_at_ms
        {
            return None;
        }
        self.base_properties.pk_count -= 1;
        self.murderer_time_stamp_ms = if self.base_properties.pk_count == 0 {
            0
        } else {
            now_ms()
        };
        Some(PlayerMurdererSignDecrease {
            player_id: self.player_id(),
            pk_count: self.base_properties.pk_count,
            kill_count: self.base_properties.kill_count,
            checked_at_ms,
            next_timestamp_ms: self.murderer_time_stamp_ms,
        })
    }

    pub(crate) const fn set_money_snapshot(&mut self, money: u32) {
        self.money = money;
    }

    pub(crate) const fn silence_minutes(&self) -> i32 {
        self.silence_minutes
    }

    /// Exact chat cooldown: unsigned wrapping elapsed сравнивается до
    /// mutation. Caller сохраняет исходный порядок последующих проверок.
    pub(crate) fn begin_talk(
        &mut self,
        channel: PlayerTalkChannel,
        now_ms: u32,
        interval_ms: u32,
    ) -> bool {
        let timestamp = match channel {
            PlayerTalkChannel::Normal => &mut self.normal_talk_timestamp_ms,
            PlayerTalkChannel::Area => &mut self.area_talk_timestamp_ms,
            PlayerTalkChannel::Country => &mut self.country_talk_timestamp_ms,
            PlayerTalkChannel::World => &mut self.world_talk_timestamp_ms,
            PlayerTalkChannel::Private => &mut self.private_talk_timestamp_ms,
            PlayerTalkChannel::Team => &mut self.team_talk_timestamp_ms,
            PlayerTalkChannel::Union => &mut self.union_talk_timestamp_ms,
        };
        if now_ms.wrapping_sub(*timestamp) < interval_ms {
            return false;
        }
        *timestamp = now_ms;
        true
    }

    pub(crate) const fn equipment(&self) -> &CEquipmentContainer {
        &self.equipment
    }

    pub(crate) fn weapon_damage_level(&self, factory: &CGoodsFactory) -> i32 {
        self.equipment.get_goods(2).map_or(0, |goods| {
            goods.addon_property_value(
                factory,
                crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_DAMAGE_LEVEL,
                1,
            )
        })
    }

    /// Точный `CPlayer::GetWeaponModifier`: отрицательная разница уровней
    /// обнуляется, затем результат последовательно ограничивается сверху и
    /// снизу обычными float-сравнениями. Отдельной защиты от нулевого divisor
    /// в EXE нет, поэтому `0 / 0` намеренно остаётся `NaN`.
    pub(crate) fn weapon_modifier(
        &self,
        factory: &CGoodsFactory,
        target_level: i32,
        divisor: f32,
        minimum: f32,
    ) -> f32 {
        let delta = self
            .weapon_damage_level(factory)
            .wrapping_sub(target_level)
            .max(0);
        let mut modifier = delta as f32 / divisor;
        if modifier > 1.0 {
            modifier = 1.0;
        }
        if modifier < minimum {
            modifier = minimum;
        }
        modifier
    }

    /// Точный обход GoodsAI при первом входе: позиционные equipment/packet,
    /// одиночный hand, позиционные auction и depot. Возврат `false` повторяет
    /// исходный `break` только внутри текущего контейнера; следующий владелец
    /// всё равно обрабатывается. Закрытый depot читается через собственное
    /// базовое хранилище без временной смены lock-флага — безопасная замена
    /// исходного `Unlock(saved password) → traversal → Lock`, не меняющая
    /// наблюдаемое итоговое состояние блокировки.
    pub(crate) fn visit_login_goods_mut(
        &mut self,
        mut visit: impl FnMut(PlayerLoginGoodsLocation, &mut CGoods) -> bool,
    ) {
        for position in 0..17 {
            if let Some(goods) = self.equipment.get_goods_mut(position)
                && !visit(PlayerLoginGoodsLocation::Equipment, goods)
            {
                break;
            }
        }
        for position in 0..self.packet.size() {
            if let Some(goods) = self.packet.get_goods_mut(position)
                && !visit(PlayerLoginGoodsLocation::Packet, goods)
            {
                break;
            }
        }
        let hand_id = self
            .hand
            .traversing_goods()
            .next()
            .map(|goods| goods.identity().ex_id);
        if let Some(goods_id) = hand_id
            && let Some(goods) = self.hand.find_mut(goods_id)
        {
            let _ = visit(PlayerLoginGoodsLocation::Hand, goods);
        }
        for position in 0..self.auction_listing.size() {
            if let Some(goods) = self.auction_listing.get_goods_mut(position)
                && !visit(PlayerLoginGoodsLocation::Auction, goods)
            {
                break;
            }
        }
        for position in 0..self.depot.base().size() {
            if let Some(goods) = self.depot.base_mut().get_goods_mut(position)
                && !visit(PlayerLoginGoodsLocation::Depot, goods)
            {
                break;
            }
        }
    }

    pub(crate) const fn packet(&self) -> &CVolumeLimitGoodsContainer {
        &self.packet
    }

    pub(crate) const fn depot(&self) -> &CDepot {
        &self.depot
    }

    pub(crate) const fn depot_mut(&mut self) -> &mut CDepot {
        &mut self.depot
    }

    pub(crate) fn trade_source_goods(
        &self,
        extend_id: i32,
        position: u32,
        goods_id: CGuid,
    ) -> Option<&CGoods> {
        let goods = match extend_id {
            1 => self.packet.get_goods(position),
            2 => self.equipment.get_goods(position),
            4 => self.wallet.get_goods(position),
            5 => self.yuan_bao.get_goods(position),
            _ => None,
        }?;
        (goods.identity().ex_id == goods_id).then_some(goods)
    }

    pub(crate) fn enhancement_selected_goods_id(&self) -> Option<CGuid> {
        self.enhancement.base().goods_id_at(0)
    }

    /// Контейнер улучшения хранит только теневые метаданные; сценарии
    /// `9409/9411` каждый раз разрешают выбранный товар обратно в его живого
    /// владельца.
    pub(crate) fn enhancement_selected_goods_mut(&mut self) -> Option<&mut CGoods> {
        let goods_id = self.enhancement_selected_goods_id()?;
        self.get_goods_by_id_mut(goods_id)
    }

    /// Владелец сценарной функции записывает доверенный серверный путь; клиент
    /// `0x8FC11/12` никогда не передаёт имя исполняемого файла.
    pub(crate) fn set_last_container_script(&mut self, script: impl AsRef<[u8]>) {
        self.last_container_script.clear();
        self.last_container_script
            .extend_from_slice(script.as_ref());
    }

    pub(crate) fn last_container_script(&self) -> &[u8] {
        &self.last_container_script
    }

    pub(crate) const fn variable_list(&self) -> &CVariableList {
        &self.variable_list
    }

    pub(crate) fn initialize_variable_list(&mut self, definitions: Option<&[u8]>) {
        if self.variable_list.variables().is_empty() {
            self.variable_list = CVariableList::from_definitions(definitions);
        }
    }

    pub(crate) fn set_string_variable(
        &mut self,
        name: &[u8],
        value: &[u8],
    ) -> GameVariableMutationOutcome {
        self.variable_list.set_string(name, value)
    }

    pub(crate) fn set_integer_variable(
        &mut self,
        name: &[u8],
        element_index: usize,
        value: i32,
    ) -> GameVariableMutationOutcome {
        self.variable_list.set_integer(name, element_index, value)
    }

    pub(crate) fn add_integer_variable(
        &mut self,
        name: &[u8],
        value: i32,
    ) -> GameVariableMutationOutcome {
        self.variable_list.add_integer(name, value)
    }

    pub(crate) fn add_string_variable(
        &mut self,
        name: &[u8],
        value: &[u8],
    ) -> GameVariableMutationOutcome {
        self.variable_list.add_string(name, value)
    }

    pub(crate) fn clear_all_enhancement_selection(&mut self) -> usize {
        self.enhancement.clear()
    }

    pub(crate) fn record_enhancement_selection(
        &mut self,
        goods_id: CGuid,
        previous: PreviousContainer,
        placed_position: u32,
    ) -> Result<AmountShadowAdded, EnhancementSelectionBlock> {
        let goods = self
            .get_goods_by_id(goods_id)
            .ok_or(EnhancementSelectionBlock::MissingGoods)?;
        let placed = PlacedShadowGoods {
            identity: goods_id,
            position: placed_position,
            base_properties_index: goods.base_properties_index(),
            amount: goods.amount(),
        };
        self.enhancement
            .record_placed_goods(previous, placed)
            .map_err(EnhancementSelectionBlock::Shadow)
    }

    /// Exact player→enhancement часть `CC2SContainerObjectMove`: shadow не
    /// владеет goods, поэтому успешный native remove→source add безопасно
    /// свёрнут в проверку live source и атомарную запись metadata.
    pub(crate) fn select_enhancement_goods(
        &mut self,
        source_extend_id: i32,
        source_position: u32,
        goods_id: CGuid,
        amount: u32,
        factory: &CGoodsFactory,
    ) -> Result<EnhancementSelectionReport, EnhancementSelectionBlock> {
        let goods = match source_extend_id {
            1 => self.packet.get_goods(source_position),
            2 => self.equipment.get_goods(source_position),
            _ => return Err(EnhancementSelectionBlock::UnsupportedSourceContainer),
        }
        .ok_or(EnhancementSelectionBlock::MissingGoods)?;
        if goods.identity().ex_id != goods_id {
            return Err(EnhancementSelectionBlock::GoodsIdentityMismatch);
        }
        if goods.amount() != amount {
            return Err(EnhancementSelectionBlock::GoodsAmountMismatch);
        }
        match goods.can_stack(factory) {
            Ok(true) => return Err(EnhancementSelectionBlock::StackableGoods),
            Ok(false) => {}
            Err(_) => return Err(EnhancementSelectionBlock::MissingBaseProperties),
        }

        let goods = goods.identity();
        let source = PreviousContainer {
            container_type: PLAYER_TYPE,
            container_id: self.player_id(),
            container_extend_id: source_extend_id,
            goods_position: source_position,
        };
        let shadow = self.record_enhancement_selection(goods_id, source, source_position)?;
        let previous_last_operated =
            self.record_last_operated_goods(source_extend_id, source_position);
        Ok(EnhancementSelectionReport {
            goods,
            source,
            shadow,
            previous_last_operated,
        })
    }

    pub(crate) fn enhancement_original_container(
        &self,
        shadow_position: u32,
        goods_id: CGuid,
    ) -> Option<PreviousContainer> {
        (self.enhancement.base().goods_id_at(shadow_position) == Some(goods_id))
            .then(|| {
                self.enhancement
                    .base()
                    .original_container_information(goods_id)
            })
            .flatten()
    }

    pub(crate) fn enhancement_remove_shadow(
        &mut self,
        goods_id: CGuid,
    ) -> Option<super::container::cgoodsshadowcontainer::ShadowRemovedReport> {
        self.enhancement.base_mut().remove_shadow(goods_id)
    }

    /// Same-original-slot ветвь native shadow Remove: underlying goods после
    /// remove→add остаётся у прежнего owner-а, а здесь удаляется только shadow
    /// metadata и формируется обязательный `OT_DELETE_OBJECT` report.
    pub(crate) fn clear_enhancement_selection(
        &mut self,
        shadow_position: u32,
        goods_id: CGuid,
        amount: u32,
    ) -> Result<EnhancementDeselectionReport, EnhancementDeselectionBlock> {
        let actual_id = self
            .enhancement
            .base()
            .goods_id_at(shadow_position)
            .ok_or(EnhancementDeselectionBlock::MissingShadow)?;
        if actual_id != goods_id {
            return Err(EnhancementDeselectionBlock::GoodsIdentityMismatch);
        }
        let source = self
            .enhancement
            .base()
            .original_container_information(goods_id)
            .ok_or(EnhancementDeselectionBlock::MissingShadow)?;
        let goods = match source.container_extend_id {
            1 => self.packet.get_goods(source.goods_position),
            2 => self.equipment.get_goods(source.goods_position),
            _ => None,
        }
        .filter(|goods| goods.identity().ex_id == goods_id)
        .ok_or(EnhancementDeselectionBlock::MissingSourceGoods)?;
        if goods.amount() != amount {
            return Err(EnhancementDeselectionBlock::GoodsAmountMismatch);
        }
        let goods = goods.identity();
        let removed = self
            .enhancement
            .base_mut()
            .remove_shadow(goods_id)
            .ok_or(EnhancementDeselectionBlock::MissingShadow)?;
        Ok(EnhancementDeselectionReport {
            goods,
            source,
            removed,
        })
    }

    pub(crate) const fn packet_mut(&mut self) -> &mut CVolumeLimitGoodsContainer {
        &mut self.packet
    }

    pub(crate) const fn equipment_mut(&mut self) -> &mut CEquipmentContainer {
        &mut self.equipment
    }

    pub(crate) const fn battle_fairy_container(&self) -> &CBattleFairyContainer {
        &self.battle_fairy_container
    }

    pub(crate) const fn fairy_container(&self) -> &CFairyContainer {
        &self.fairy_container
    }

    pub(crate) const fn fairy_container_mut(&mut self) -> &mut CFairyContainer {
        &mut self.fairy_container
    }

    pub(crate) const fn battle_fairy_container_mut(&mut self) -> &mut CBattleFairyContainer {
        &mut self.battle_fairy_container
    }

    /// Выполняет player-часть `LoadBFDefualtProperty` для ещё не добавленного
    /// сценарного предмета: изменяет сам предмет и регистрирует три начальных
    /// навыка в каноническом `CMoveShape` игрока.
    pub(crate) fn initialize_script_battle_fairy_goods(
        &mut self,
        goods: &mut CGoods,
        factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Option<(BattleFairyDefaultGoodsUpdate, Vec<BattleFairySkillAdded>)> {
        let player_id = self.player_id();
        let mut skills = Vec::with_capacity(3);
        let mut register_skill = |skill: BattleFairyDefaultSkill| {
            let registered = self
                .move_shape
                .add_skill(skill.id, skill.level, skill_factory);
            if let Some(stored) = self.move_shape.skill(skill.id, skill_factory) {
                skills.push(battle_fairy_skill_snapshot(player_id, stored, skill_factory));
            }
            registered
        };
        let report = CBattleFairyContainer::load_default_properties(
            Some(player_id),
            goods,
            factory,
            &mut register_skill,
            encode_old_client,
        )?;
        Some((report, skills))
    }

    /// Учётная запись принадлежит снимку игрока и используется точным журналом
    /// объединения; до загрузки она остаётся пустой строкой.
    pub(crate) fn set_account(&mut self, account: impl AsRef<[u8]>) {
        self.account.clear();
        self.account.extend_from_slice(account.as_ref());
    }

    pub(crate) fn account(&self) -> &[u8] {
        &self.account
    }

    pub(crate) fn billing_session_id(&self) -> &[u8] {
        &self.session_id
    }

    /// Exact `GetWarSoulGoods`: боевой дух — только headgear в позиции 10,
    /// чьё первое значение `GAP_BF_BATTLE_FAIRY` равно единице.
    pub(crate) fn war_soul_goods(&self, factory: &CGoodsFactory) -> Option<&CGoods> {
        self.equipment
            .get_goods(10)
            .filter(|goods| goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) == 1)
    }

    pub(crate) fn war_soul_mana(&self, factory: &CGoodsFactory) -> Option<i32> {
        self.war_soul_goods(factory)
            .map(|goods| goods.addon_property_value(factory, GAP_BF_MP, 1))
    }

    pub(crate) fn war_soul_attack(&self, factory: &CGoodsFactory) -> Option<i32> {
        self.war_soul_goods(factory)
            .map(|goods| goods.addon_property_value(factory, GAP_BF_ATTACK, 1))
    }

    /// Временная проекция `ReplacePlayerData` для единственного вызова
    /// base-defense. Возвращаемые scale — уже точный результат последующего
    /// `RestorePlayerData`, включая legacy truncate через signed DWORD.
    pub(crate) fn war_soul_defense_projection(
        &self,
        factory: &CGoodsFactory,
        full_miss_scale: f32,
        critical_rate: f32,
        blast_defense_scale: f32,
    ) -> Option<(PlayerCombatProperties, u8, [f32; 3])> {
        let goods = self.war_soul_goods(factory)?;
        let restored_scale = |value: f32, minimum: f32| {
            let truncated = value.trunc() as i32 as u32;
            (truncated as f32).max(minimum)
        };
        let restored = [
            restored_scale(self.combat_properties.full_miss_scale(), 0.01),
            restored_scale(self.combat_properties.critical_rate(), 1.0),
            restored_scale(self.combat_properties.blast_defense_scale(), 0.01),
        ];
        let mut properties = self.combat_properties;
        properties.full_miss_scale_bits = full_miss_scale.max(0.01).to_bits();
        properties.critical_rate_bits = critical_rate.max(1.0).to_bits();
        properties.blast_defense_scale_bits = blast_defense_scale.max(0.01).to_bits();
        properties.blast_attack = goods.addon_property_value(factory, GAP_BF_BLAST, 1) as u16;
        let level = goods.addon_property_value(factory, GAP_BF_LEVEL, 1) as u8;
        Some((properties, level, restored))
    }

    pub(crate) fn restore_war_soul_defense_projection(&mut self, restored: [f32; 3]) {
        self.combat_properties.full_miss_scale_bits = restored[0].to_bits();
        self.combat_properties.critical_rate_bits = restored[1].to_bits();
        self.combat_properties.blast_defense_scale_bits = restored[2].to_bits();
    }

    /// Специальная ветвь `OnBeenAttacked(..., true)` сферы хаоса: урон
    /// сначала уменьшает `GAP_BF_HP` с точной wrapping-арифметикой, разрушение
    /// сбрасывает состояние, после чего формируется полный снимок старого клиента.
    pub(crate) fn apply_war_soul_hit(
        &mut self,
        damage: i32,
        factory: &CGoodsFactory,
        da_kong_key: bool,
    ) -> Option<WarSoulHitOutcome> {
        if self.war_soul_state == 0 { return None }
        let player_id = self.player_id();
        let goods = self.equipment_mut().get_goods_mut(10)?;
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 { return None }
        let cut_hurt = goods.addon_property_value(factory, GAP_BF_CUT_HURT_SCALE, 1);
        let health = goods.addon_property_value(factory, GAP_BF_HP, 1);
        let next = health.wrapping_sub(10_000i32.wrapping_sub(cut_hurt).wrapping_mul(damage));
        let broken = next < 1;
        let stored = if broken { 0 } else { next };
        let _ = goods.set_addon_property_value_core(GAP_BF_HP, 1, stored);
        let identity = goods.identity();
        let mut old_client_payload = Vec::new();
        if !goods.serialize_for_old_client(&mut old_client_payload, factory, da_kong_key) { return None }
        let broadcast_previous_status = broken && self.set_war_soul_status(0);
        Some(WarSoulHitOutcome {
            broken,
            broadcast_previous_status,
            update: super::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate {
                message_type: 0x000b_f918,
                player_id,
                goods: identity,
                old_client_payload,
            },
        })
    }

    pub(crate) fn equipped_battle_fairy_mana(&self, factory: &CGoodsFactory) -> Option<i32> {
        self.equipment
            .get_goods(10)
            .map(|goods| goods.addon_property_value(factory, GAP_BF_MP, 1))
    }

    /// `CWangsheng::AI` сначала необратимо списывает MP у slot 10 и лишь
    /// затем проверяет `GetWarSoulGoods`; отдельный исход сохраняет эту
    /// malformed-item границу без повторного описания уже выполненного расхода.
    pub(crate) fn spend_equipped_battle_fairy_mana(
        &mut self,
        amount: u32,
        factory: &CGoodsFactory,
        da_kong_key: bool,
    ) -> BattleFairyManaSpendOutcome {
        self.spend_equipped_battle_fairy_mana_inner(amount, factory, da_kong_key, true)
    }

    pub(crate) fn spend_attribute_skill_mana(
        &mut self,
        amount: u32,
        factory: &CGoodsFactory,
        da_kong_key: bool,
    ) -> BattleFairyManaSpendOutcome {
        self.spend_equipped_battle_fairy_mana_inner(amount, factory, da_kong_key, false)
    }

    fn spend_equipped_battle_fairy_mana_inner(
        &mut self,
        amount: u32,
        factory: &CGoodsFactory,
        da_kong_key: bool,
        require_war_soul: bool,
    ) -> BattleFairyManaSpendOutcome {
        let player_id = self.player_id();
        let Some(goods) = self.equipment_mut().get_goods_mut(10) else {
            return BattleFairyManaSpendOutcome::MissingEquipment;
        };
        let current = goods.addon_property_value(factory, GAP_BF_MP, 1);
        let _ = goods.set_addon_property_value_core(
            GAP_BF_MP,
            1,
            current.wrapping_sub(amount as i32),
        );
        if require_war_soul && goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            return BattleFairyManaSpendOutcome::SpentWithoutWarSoul;
        }
        let identity = goods.identity();
        let mut old_client_payload = Vec::new();
        let update = goods
            .serialize_for_old_client(&mut old_client_payload, factory, da_kong_key)
            .then_some(BattleFairyDefaultGoodsUpdate {
                message_type: 0x0b_f918,
                player_id,
                goods: identity,
                old_client_payload,
            });
        BattleFairyManaSpendOutcome::Spent { update }
    }

    pub(crate) fn spend_war_soul_mana(
        &mut self,
        amount: u32,
        factory: &CGoodsFactory,
        da_kong_key: bool,
    ) -> Option<super::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate> {
        let current = self.war_soul_mana(factory)?;
        let next = current.wrapping_sub(amount as i32);
        let player_id = self.player_id();
        let goods = self.equipment_mut().get_goods_mut(10)?;
        let _ = goods.set_addon_property_value_core(GAP_BF_MP, 1, next);
        let identity = goods.identity();
        let mut old_client_payload = Vec::new();
        if !goods.serialize_for_old_client(&mut old_client_payload, factory, da_kong_key) {
            return None;
        }
        Some(
            super::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate {
                message_type: 0x0b_f918,
                player_id,
                goods: identity,
                old_client_payload,
            },
        )
    }

    /// Изменяет `GAP_BF_HP` owned боевого духа и формирует тот же полный
    /// old-client payload, который исходный навык отправляет после лечения.
    pub(crate) fn restore_war_soul_health(
        &mut self,
        amount: u32,
        factory: &CGoodsFactory,
        da_kong_key: bool,
    ) -> Option<super::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate> {
        self.restore_war_soul_property(
            amount,
            GAP_BF_HP,
            GAP_BF_MAX_HP,
            factory,
            da_kong_key,
        )
    }

    pub(crate) fn restore_war_soul_mana(
        &mut self,
        amount: u32,
        factory: &CGoodsFactory,
        da_kong_key: bool,
    ) -> Option<super::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate> {
        self.restore_war_soul_property(
            amount,
            GAP_BF_MP,
            GAP_BF_MAX_MP,
            factory,
            da_kong_key,
        )
    }

    fn restore_war_soul_property(
        &mut self,
        amount: u32,
        property: i32,
        maximum_property: i32,
        factory: &CGoodsFactory,
        da_kong_key: bool,
    ) -> Option<super::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate> {
        let player_id = self.player_id();
        let goods = self.equipment_mut().get_goods_mut(10)?;
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            return None;
        }
        let current = goods.addon_property_value(factory, property, 1);
        let maximum = goods.addon_property_value(factory, maximum_property, 1);
        let restored = (current as u32).wrapping_add(amount).min(maximum as u32) as i32;
        let _ = goods.set_addon_property_value_core(property, 1, restored);
        let identity = goods.identity();
        let mut old_client_payload = Vec::new();
        if !goods.serialize_for_old_client(&mut old_client_payload, factory, da_kong_key) {
            return None;
        }
        Some(
            super::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate {
                message_type: 0x0b_f918,
                player_id,
                goods: identity,
                old_client_payload,
            },
        )
    }

    /// Exact `GetGoodsById` lookup order для goods-message `0x8FC2E`.
    /// Locked hand/packet/auction goods скрываются container `find`, equipment
    /// использует собственный positional storage.
    pub(crate) fn get_goods_by_id(&self, goods_id: CGuid) -> Option<&CGoods> {
        self.hand
            .find(goods_id)
            .or_else(|| self.packet.base().find(goods_id))
            .or_else(|| self.equipment.find(goods_id))
            .or_else(|| self.auction_listing.base().find(goods_id))
            .or_else(|| self.auction_goods.base().find(goods_id))
    }

    pub(crate) fn get_goods_by_id_mut(&mut self, goods_id: CGuid) -> Option<&mut CGoods> {
        if self.hand.find(goods_id).is_some() {
            return self.hand.find_mut(goods_id);
        }
        if self.packet.base().find(goods_id).is_some() {
            return self.packet.base_mut().find_mut(goods_id);
        }
        if self.equipment.find(goods_id).is_some() {
            return self.equipment.find_mut(goods_id);
        }
        if self.auction_listing.base().find(goods_id).is_some() {
            return self.auction_listing.base_mut().find_mut(goods_id);
        }
        self.auction_goods.base_mut().find_mut(goods_id)
    }

    pub(crate) const fn hand_mut(&mut self) -> &mut CAmountLimitGoodsContainer {
        &mut self.hand
    }

    pub(crate) const fn hand(&self) -> &CAmountLimitGoodsContainer {
        &self.hand
    }

    pub(crate) const fn auction_goods_mut(&mut self) -> &mut CVolumeLimitGoodsContainer {
        &mut self.auction_goods
    }

    pub(crate) const fn auction_goods(&self) -> &CVolumeLimitGoodsContainer {
        &self.auction_goods
    }

    /// Точная часть состояния `CPlayer::ModifyAuctionSpace`: значение должно
    /// находиться между текущим числом свободных расширенных ячеек и полным
    /// объёмом контейнера. Исходные методы контейнера работают только с
    /// расширенной областью, начинающейся с позиции 48.
    pub(crate) fn modify_auction_space(
        &mut self,
        requested: u32,
        pack_add_enabled: bool,
    ) -> Option<u32> {
        if requested > self.auction_goods.size() || requested < self.base_properties.auction_space {
            return None;
        }
        self.auction_goods.set_all_inactive();
        if pack_add_enabled {
            self.auction_goods
                .apply_player_expansion_limit(self.auction_goods.size().wrapping_sub(requested));
        }
        let current = self.auction_goods.expansion_available_space();
        self.base_properties.auction_space = current;
        Some(current)
    }

    pub(crate) const fn auction_listing(&self) -> &CVolumeLimitGoodsContainer {
        &self.auction_listing
    }

    pub(crate) const fn auction_listing_mut(&mut self) -> &mut CVolumeLimitGoodsContainer {
        &mut self.auction_listing
    }

    pub(crate) fn auction_listing_extension_bonus(&self, factory: &CGoodsFactory) -> i32 {
        let Some(goods) = self.auction_listing.get_goods(1) else {
            return 0;
        };
        if goods.addon_property_value(factory, GAP_GOODS_PACKAGE_EXTENTION, 1) != 3 {
            return 0;
        }
        goods.addon_property_value(factory, GAP_GOODS_PACKAGE_EXTENTION, 2)
    }

    pub(crate) fn take_auction_listing_goods(&mut self) -> Option<CGoods> {
        let goods_id = self.auction_listing.get_goods(0)?.identity().ex_id;
        let outcome = self.auction_listing.remove_goods(goods_id)?;
        let taken = match outcome {
            VolumeGoodsRemoveOutcome::Removed(taken)
            | VolumeGoodsRemoveOutcome::RemovedButCellMissing(taken) => taken,
        };
        Some(match taken {
            crate::gameserver::appserver::container::camountlimitgoodscontainer::AmountLimitGoodsTaken::Removed(removed) => removed.goods,
            crate::gameserver::appserver::container::camountlimitgoodscontainer::AmountLimitGoodsTaken::Split(split) => split.goods,
        })
    }

    /// Exact state-owner возврата `0x80404`: позиция выбирается до Add,
    /// stack merge использует обычный player-progress gate, а bind value-id 2
    /// записывается уже в итоговый stored goods.
    pub(crate) fn return_auction_goods(
        &mut self,
        goods: CGoods,
        bind_type: i32,
        factory: &CGoodsFactory,
    ) -> Option<PlayerAuctionGoodsReturn> {
        let position = self
            .auction_goods
            .find_position_for_goods(&goods, factory)?;
        let source = goods.identity();
        let owner_progress_allows = self.current_progress == PlayerProgress::None;
        let mut incoming = Some(goods);
        let outcome = self.auction_goods.add_goods_at(
            position,
            &mut incoming,
            factory,
            owner_progress_allows,
        );
        let successful = matches!(
            &outcome,
            VolumeGoodsAddOutcome::Added(_)
                | VolumeGoodsAddOutcome::Stack(GoodsStackMergeOutcome::Merged { .. })
        );
        let (resulting_goods, resulting_amount, bind_stored) = if successful {
            let stored = self
                .auction_goods
                .get_goods_mut(position)
                .expect("успешный auction Add обязан оставить stored goods");
            let bind_stored = stored.set_addon_property_value_core(GAP_GOODS_BIND, 2, bind_type);
            (Some(stored.identity()), Some(stored.amount()), bind_stored)
        } else {
            (None, None, false)
        };
        Some(PlayerAuctionGoodsReturn {
            player_id: self.player_id(),
            position,
            source,
            outcome,
            resulting_goods,
            resulting_amount,
            bind_stored,
        })
    }

    pub(crate) fn auction_money(&self) -> u32 {
        self.auction_wallet.currency_amount()
    }

    /// Exact state-часть `CheckAuctionMoneyMove`: checked unsigned sum
    /// основного и auction wallet сравнивается с max stack основного wallet.
    /// Уведомление `GPM015` остаётся у message runtime caller-а.
    pub(crate) fn auction_money_move_capacity(
        &self,
        factory: &CGoodsFactory,
    ) -> AuctionMoneyMoveCapacity {
        let wallet_amount = self.wallet.currency_amount();
        let auction_amount = self.auction_wallet.currency_amount();
        let maximum = self.wallet.max_stack_number(factory);
        let allowed = wallet_amount
            .checked_add(auction_amount)
            .is_some_and(|total| total <= maximum);
        AuctionMoneyMoveCapacity {
            wallet_amount,
            auction_amount,
            maximum,
            allowed,
        }
    }

    /// Exact `AuctionLimit`: listing slot `0` принимает только предмет без
    /// particular-флагов `0x20/0x04` и без life-type addon. Player state в
    /// формуле не участвует; owner остаётся здесь из-за исходного dispatch.
    pub(crate) fn auction_listing_goods_allowed(goods: &CGoods, factory: &CGoodsFactory) -> bool {
        let particular = goods.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1) as u32;
        particular & 0x20 == 0
            && particular & 0x04 == 0
            && !goods.query_attribute(GAP_GOODS_LIFE_TYPE)
    }

    pub(crate) fn auction_money_goods(&self) -> Option<&CGoods> {
        self.auction_wallet.get_goods(0)
    }

    /// State-часть exact `SetAuctionMoney`; аргумент является новым абсолютным
    /// балансом, а caller создаёт недостающий MONEY и публикует extend-id 15.
    pub(crate) fn set_auction_money(
        &mut self,
        current: u32,
        factory: &CGoodsFactory,
        created_currency: Vec<CGoods>,
    ) -> PlayerAuctionMoneyChange {
        let previous = self.auction_wallet.currency_amount();
        let outcome = if previous < current {
            let mut created_currency = Some(created_currency);
            PlayerAuctionMoneyChangeOutcome::Increased(
                self.auction_wallet.increase_currency(
                    current.wrapping_sub(previous),
                    factory,
                    move |_, _| created_currency.take().unwrap_or_default(),
                ),
            )
        } else if current < previous {
            PlayerAuctionMoneyChangeOutcome::Decreased(
                self.auction_wallet
                    .decrease_currency(previous.wrapping_sub(current), factory),
            )
        } else {
            PlayerAuctionMoneyChangeOutcome::Unchanged
        };
        PlayerAuctionMoneyChange {
            player_id: self.player_id(),
            previous,
            current: self.auction_wallet.currency_amount(),
            outcome,
        }
    }

    pub(crate) const fn set_auction_open(&mut self, open: bool) {
        self.auction_open = open;
    }

    /// State/container часть exact `TellClientScale`; закрытый аукцион не
    /// создаёт client-effect, открытый уменьшает положительные scale-счётчики
    /// и сохраняет container traversal order.
    pub(crate) fn auction_scale_goods_ids(
        &mut self,
        factory: &CGoodsFactory,
    ) -> Option<Vec<CGuid>> {
        self.auction_open
            .then(|| self.auction_goods.get_scale_goods(factory))
    }

    pub(crate) fn auction_goods_identity_at(&self, position: u32) -> Option<ShapeIdentity> {
        self.auction_goods.get_goods(position).map(CGoods::identity)
    }

    /// Exact `BuyItemFromAauction` clock gate: strict wrapping threshold и
    /// отдельный второй sample записываются до GUID decode/query.
    pub(crate) fn begin_auction_buy(&mut self, mut tick_ms: impl FnMut() -> u32) -> AuctionBuyGate {
        let sampled_tick_ms = tick_ms();
        let previous_tick_ms = self.last_auction_option_tick_ms;
        if previous_tick_ms.wrapping_add(5_000) >= sampled_tick_ms {
            return AuctionBuyGate::Throttled {
                sampled_tick_ms,
                previous_tick_ms,
            };
        }
        let recorded_tick_ms = tick_ms();
        self.last_auction_option_tick_ms = recorded_tick_ms;
        AuctionBuyGate::Ready {
            sampled_tick_ms,
            recorded_tick_ms,
        }
    }

    /// Exact `MakeCurAucNode` 5-second gate с отдельным вторым tick sample.
    pub(crate) fn begin_auction_listing(
        &mut self,
        mut tick_ms: impl FnMut() -> u32,
    ) -> AuctionListingGate {
        let sampled_tick_ms = tick_ms();
        let previous_tick_ms = self.last_auction_option_tick_ms;
        if previous_tick_ms.wrapping_add(5_000) >= sampled_tick_ms {
            return AuctionListingGate::Throttled {
                sampled_tick_ms,
                previous_tick_ms,
            };
        }
        let recorded_tick_ms = tick_ms();
        self.last_auction_option_tick_ms = recorded_tick_ms;
        AuctionListingGate::Ready {
            sampled_tick_ms,
            recorded_tick_ms,
        }
    }

    /// Exact `IsAollowAuction` 1-second gate: timestamp обновляется до limit
    /// queries, а failed limit также поглощает текущую попытку.
    pub(crate) fn begin_auction_limit_check(
        &mut self,
        tick_ms: u32,
        owner_goods_count: usize,
        global_goods_count: usize,
        player_maximum: f32,
        global_maximum: f32,
        extension_bonus: i32,
    ) -> bool {
        if tick_ms.wrapping_sub(self.last_auction_limit_tick_ms) <= 1_000 {
            return false;
        }
        self.last_auction_limit_tick_ms = tick_ms;
        (owner_goods_count as f32) < extension_bonus as f32 + player_maximum
            && (global_goods_count as f32) < global_maximum
    }

    pub(crate) fn current_auction_node(&self) -> Option<&CGoodsNode> {
        self.current_auction_node.as_ref()
    }

    pub(crate) fn set_current_auction_node(&mut self, node: CGoodsNode) -> bool {
        if self.current_auction_node.is_some() {
            return false;
        }
        self.current_auction_node = Some(node);
        true
    }

    /// Точная запись `AutoAddAuctionGoods`: каждый созданный предмет целиком
    /// заменяет предыдущий `m_CurrentAucNode` без проверки занятости узла.
    pub(crate) fn replace_current_auction_node(&mut self, node: CGoodsNode) {
        self.current_auction_node = Some(node);
    }

    pub(crate) fn take_current_auction_node(&mut self) -> Option<CGoodsNode> {
        self.current_auction_node.take()
    }

    pub(crate) const fn auction_listing_fee(&self) -> u32 {
        self.auction_listing_fee
    }

    pub(crate) const fn set_auction_listing_fee(&mut self, fee: u32) {
        self.auction_listing_fee = fee;
    }

    pub(crate) fn current_auction_buy_node(&self) -> Option<&CGoodsNode> {
        self.current_auction_buy_node.as_ref()
    }

    pub(crate) fn set_current_auction_buy_node(&mut self, node: CGoodsNode) -> bool {
        if self.current_auction_buy_node.is_some() {
            return false;
        }
        self.current_auction_buy_node = Some(node);
        true
    }

    pub(crate) fn take_current_auction_buy_node(&mut self) -> Option<CGoodsNode> {
        self.current_auction_buy_node.take()
    }

    pub(crate) fn client_ip_text(&self) -> Vec<u8> {
        let ip = self.client_ip;
        format!(
            "{}.{}.{}.{}",
            ip & 0xff,
            (ip >> 8) & 0xff,
            (ip >> 16) & 0xff,
            ip >> 24
        )
        .into_bytes()
    }

    pub(crate) fn begin_auction_search(
        &mut self,
        name: &[u8],
        lower_level: i32,
        upper_level: i32,
        use_self: i32,
        money_type: i32,
        weapon_type: i32,
    ) {
        self.auction_search_name.clear();
        self.auction_search_name.extend_from_slice(name);
        self.auction_search_lower_level = lower_level;
        self.auction_search_upper_level = upper_level;
        self.auction_search_use_self = use_self;
        self.auction_search_money_type = money_type;
        self.auction_search_weapon_type = weapon_type;
        self.auction_current_page = 0;
    }

    /// Exact `ReFlushSelfGoods`: strict wrapping `last + 5000 < first sample`,
    /// затем отдельный второй `timeGetTime` sample записывается до World send.
    pub(crate) fn refresh_auction_self_goods(
        &mut self,
        factory: &CGoodsFactory,
        mut tick_ms: impl FnMut() -> u32,
    ) -> AuctionSelfGoodsRefresh {
        let sampled_tick_ms = tick_ms();
        let previous_tick_ms = self.last_auction_option_tick_ms;
        if previous_tick_ms.wrapping_add(5_000) >= sampled_tick_ms {
            return AuctionSelfGoodsRefresh::Throttled {
                sampled_tick_ms,
                previous_tick_ms,
            };
        }
        let recorded_tick_ms = tick_ms();
        self.last_auction_option_tick_ms = recorded_tick_ms;
        let wallet_amount = self.auction_wallet.currency_amount();
        let wallet_maximum = self.auction_wallet.max_stack_number(factory);
        AuctionSelfGoodsRefresh::Requested {
            sampled_tick_ms,
            recorded_tick_ms,
            goods_space: self.auction_goods.space(),
            wallet_space: wallet_maximum.wrapping_sub(wallet_amount),
        }
    }

    /// Exact derived `bHasPet`: отдельный pet owner materializes list later;
    /// этому caller-у нужен только подтверждённый факт её непустоты.
    pub(crate) const fn set_active_pet_count(&mut self, count: u32) {
        self.active_pet_count = count;
    }

    pub(crate) fn take_uncreated_pets(&mut self) -> Vec<PlayerUncreatedPet> {
        std::mem::take(&mut self.uncreated_pets)
    }

    pub(crate) fn login_carriage(&self) -> (&PlayerUncreatedCarriage, bool) {
        (&self.uncreated_carriage, self.recreate_carriage)
    }

    pub(crate) fn finish_login_carriage_recreation(&mut self, carriage_id: i32) {
        self.active_carriage_id = carriage_id;
        self.recreate_carriage = false;
        self.uncreated_carriage = PlayerUncreatedCarriage::default();
    }

    pub(crate) fn finish_empty_login_carriage_recreation(&mut self) {
        self.finish_login_carriage_recreation(0);
    }

    pub(crate) const fn bind_active_carriage(&mut self, carriage_id: i32) {
        self.active_carriage_id = carriage_id;
    }

    pub(crate) const fn active_carriage_id(&self) -> i32 {
        self.active_carriage_id
    }

    pub(crate) const fn clear_active_carriage(&mut self, carriage_id: i32) -> bool {
        if self.active_carriage_id != carriage_id {
            return false;
        }
        self.active_carriage_id = 0;
        true
    }

    pub(crate) const fn current_pets_mode(&self) -> i32 {
        self.move_shape.current_pets_mode()
    }

    pub(crate) fn set_current_pets_mode(&mut self, mode: i32) -> bool {
        self.move_shape.set_current_pets_mode(mode)
    }

    pub(crate) fn add_active_pet(&mut self, object_type: i32, id: i32, figure: i32) {
        self.move_shape.add_pet(object_type, id, figure);
        self.active_pet_count = self.move_shape.pets().len() as u32;
    }

    pub(crate) fn remove_active_pet(&mut self, object_type: i32, id: i32) -> bool {
        let removed = self.move_shape.remove_pet(object_type, id);
        self.active_pet_count = self.move_shape.pets().len() as u32;
        removed
    }

    pub(crate) fn active_pets(&self) -> &[super::moveshape::MoveShapePet] {
        self.move_shape.pets()
    }

    pub(crate) fn learned_skill_level(&self, skill_id: u32, factory: &CSkillFactory) -> i32 {
        self.move_shape.skill_level(skill_id, factory)
    }

    pub(crate) fn learned_skill_level_if_present(&self, skill_id: u32, factory: &CSkillFactory) -> Option<i32> {
        self.move_shape.skill(skill_id, factory).map(|skill| skill.level())
    }

    fn serializable_skills(&self) -> impl Iterator<Item = &MoveShapeSkill> {
        [
            SkillCategory::Attack,
            SkillCategory::Defense,
            SkillCategory::Summon,
            SkillCategory::State,
        ]
        .into_iter()
        .flat_map(|category| {
            self.move_shape.skills_in_category(category).filter(move |skill| {
                category != SkillCategory::Defense || skill.id() != SKILL_BASE_DEFENSE
            })
        })
    }

    /// Создание intrinsic skills из `CPlayer::InitSkills` (0x00440C30):
    /// отсутствующая базовая защита добавляется первой,
    /// затем профессии `0/1/2` получают соответственно обычную атаку,
    /// атаку со стрельбой либо базовую магию. Уже загруженные записи не
    /// заменяются. CGame вызывает это после login-script; последующий
    /// SetCurrentSkill(GetDefaultAttackSkillID) остаётся у опубликованного owner-а.
    pub(crate) fn initialize_intrinsic_skills(&mut self, factory: &CSkillFactory) {
        if self.move_shape.skill(SKILL_BASE_DEFENSE, factory).is_none() {
            self.move_shape.add_base_defense_skill(factory);
        }
        let intrinsic = match self.occupation() {
            0 => &[BASE_ATTACK_SKILL_ID][..],
            1 => &[BASE_ATTACK_SKILL_ID, ARCHERY_SKILL_ID][..],
            2 => &[BASE_MAGIC_SKILL_ID][..],
            _ => &[],
        };
        for &skill_id in intrinsic {
            if self.move_shape.skill(skill_id, factory).is_none() {
                let _ = self.move_shape.add_skill(skill_id, 1, factory);
            }
        }
    }

    pub(crate) const fn has_pet(&self) -> bool {
        self.active_pet_count != 0
    }

    /// Snapshot/skill caller передаёт только current ID, достаточный для
    /// `SummonBF` запрета `SKILL_MONSTER_TAMING`; concrete skill execution не
    /// становится частью player owner-а.
    pub(crate) const fn set_current_skill_id(&mut self, skill_id: Option<u32>) {
        self.move_shape.set_current_skill_id(skill_id);
    }

    pub(crate) const fn current_skill_id(&self) -> Option<u32> {
        self.move_shape.current_skill_id()
    }

    /// Exact `CPlayer::GetDefaultAttackSkillID`: лучник использует базовую
    /// стрельбу только с луком, арбалетом либо weapon-category `8`; маг всегда
    /// возвращается к базовой магии, остальные варианты — к обычной атаке.
    pub(crate) fn default_attack_skill_id(&self, factory: &CGoodsFactory) -> u32 {
        match self.occupation() {
            1 if self.equipment.get_goods(2).is_some_and(|weapon| {
                matches!(
                    weapon.addon_property_value(factory, GAP_WEAPON_CATEGORY, 1),
                    3 | 4 | 8
                )
            }) => ARCHERY_SKILL_ID,
            2 => BASE_MAGIC_SKILL_ID,
            _ => BASE_ATTACK_SKILL_ID,
        }
    }

    /// Native `OnChangeSkill` и `OnLoseTarget` назначают вычисленный после
    /// concrete `End` default skill. Наличие выбранного ID не означает
    /// незавершённое исполнение: оно хранится отдельно в CPlayerAI.
    pub(crate) const fn restore_default_attack_skill_after_end(
        &mut self,
        default_attack_skill_id: u32,
    ) {
        self.move_shape.set_current_skill_id(Some(default_attack_skill_id));
    }

    pub(crate) const fn war_soul_state(&self) -> u32 {
        self.war_soul_state
    }

    pub(crate) const fn war_soul_point(&self) -> WarSoulPoint {
        self.war_soul_point
    }

    pub(crate) const fn battle_fairy_summoned(&self) -> bool {
        self.battle_fairy_summoned
    }

    /// Exact `SetWarSoulStaus`: around status публикуется по прежнему state,
    /// затем любое значение кроме единицы нормализуется к нулю.
    pub(crate) const fn set_war_soul_status(&mut self, value: u32) -> bool {
        let broadcast_previous = self.war_soul_state == 1;
        if value == 1 {
            self.battle_fairy_summoned = true;
            self.war_soul_state = 1;
        } else {
            self.battle_fairy_summoned = false;
            self.war_soul_state = 0;
        }
        broadcast_previous
    }

    /// Исполняет player-часть `CBattleFairyContainer::SummonBF`. Spatial map
    /// принадлежит `CServerRegion`, поэтому действие возвращается явным
    /// tail-ом для `CGame`; ordered notify/broadcast/property effects там
    /// сериализуются concrete wire после spatial mutation.
    pub(crate) fn summon_battle_fairy(
        &mut self,
        battle_fairy_enabled: bool,
        mode: i32,
        factory: &CGoodsFactory,
    ) -> BattleFairySummonReport {
        let player_id = self.player_id();
        let mut report = BattleFairySummonReport {
            player_id,
            outcome: BattleFairySummonOutcome::IgnoredMode,
            region_id: self.server_region_id,
            spatial_action: None,
            effects: GameEffectJournal::default(),
        };
        if !battle_fairy_enabled {
            report.outcome = BattleFairySummonOutcome::FeatureDisabled;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0023", 0xffff_ffff);
            return report;
        }
        if mode == 1 && self.war_soul_state == 1 {
            report.outcome = BattleFairySummonOutcome::AlreadySummoned;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0024", 0xffff_ffff);
            return report;
        }
        if mode == -1 && self.base_properties.battle_fairy_recall {
            report.outcome = BattleFairySummonOutcome::AlreadyRecalled;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0025", 0xffff_ffff);
            return report;
        }
        let Some(goods) = self.equipment.get_goods(10) else {
            report.outcome = BattleFairySummonOutcome::MissingHeadgear;
            return report;
        };
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            report.outcome = BattleFairySummonOutcome::InvalidHeadgear;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0009", 0xffff_ffff);
            return report;
        }
        if goods.addon_property_value(factory, GAP_BF_HP, 1) < 1 {
            report.outcome = BattleFairySummonOutcome::NoHitPoints;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0026", 0xffff_0000);
            return report;
        }
        if self.has_pet() {
            report.outcome = BattleFairySummonOutcome::ActivePet;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0027", 0xffff_ffff);
            return report;
        }
        if self.move_shape.current_skill_id() == Some(MONSTER_TAMING_SKILL_ID) {
            report.outcome = BattleFairySummonOutcome::MonsterTamingActive;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0028", 0xffff_ffff);
            return report;
        }
        let player_position = match (self.shape().get_tile_x(), self.shape().get_tile_y()) {
            (Ok(x), Ok(y)) => WarSoulPoint { x, y },
            (Err(error), _) | (_, Err(error)) => {
                report.outcome = BattleFairySummonOutcome::CoordinateBlocked(error);
                return report;
            }
        };

        match mode {
            1 => {
                self.battle_fairy_summoned = true;
                self.war_soul_state = 1;
                self.base_properties.battle_fairy_recall = false;
                self.base_properties.battle_fairy_died = false;
                self.war_soul_visual_x_bits = (player_position.x as f32).to_bits();
                self.war_soul_visual_y_bits = (player_position.y as f32).to_bits();
                report.outcome = BattleFairySummonOutcome::Summoned;
                report.spatial_action = Some(BattleFairyWarSoulAction::SetPosition {
                    previous: self.war_soul_point,
                    target: player_position,
                });
                report.effects.push(BattleFairySummonEffect::AroundMessage {
                    message_type: BATTLE_FAIRY_MOVE_MESSAGE_TYPE,
                    player_id,
                    values: vec![player_id, 700, player_position.x, player_position.y],
                });
                // `SetWarSoulStaus(1)` наблюдает уже записанный state `1` и
                // поэтому публикует exact `0xbf930 {400, player_id}`.
                let _broadcast_previous = self.set_war_soul_status(1);
                report.effects.push(BattleFairySummonEffect::AroundMessage {
                    message_type: BATTLE_FAIRY_STATUS_MESSAGE_TYPE,
                    player_id,
                    values: vec![400, player_id],
                });
                report.effects.push(BattleFairySummonEffect::AroundMessage {
                    message_type: BATTLE_FAIRY_SUMMON_MESSAGE_TYPE,
                    player_id,
                    values: vec![400, 1],
                });
            }
            -1 => {
                self.battle_fairy_summoned = false;
                self.war_soul_state = 0;
                self.base_properties.battle_fairy_recall = true;
                self.base_properties.battle_fairy_died = false;
                self.war_soul_visual_x_bits = (-1.0f32).to_bits();
                self.war_soul_visual_y_bits = (-1.0f32).to_bits();
                report.outcome = BattleFairySummonOutcome::Recalled;
                report.spatial_action = Some(BattleFairyWarSoulAction::Delete {
                    previous: self.war_soul_point,
                    player_position,
                });
                report.effects.push(BattleFairySummonEffect::AroundMessage {
                    message_type: BATTLE_FAIRY_STATUS_MESSAGE_TYPE,
                    player_id,
                    values: vec![400, -1],
                });
            }
            _ => {}
        }
        report
            .effects
            .push(BattleFairySummonEffect::PropertiesChanged { player_id });
        report
    }

    /// Полный player-tail успешного `CEquipmentContainer::Remove`: container
    /// mutation предшествует callback-ам, поэтому removed slot уже отсутствует
    /// во время injected результата virtual `PropertiesChanged`.
    pub(crate) fn remove_equipment_goods(
        &mut self,
        ex_id: CGuid,
        factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
        runtime: PlayerEquipmentRemoveRuntimeFacts,
        recompute_properties: &mut dyn FnMut(&mut CPlayer) -> PlayerPropertyRecompute,
    ) -> PlayerEquipmentRemoveReport {
        let player_id = self.player_id();
        let mut outcome = self.equipment.remove(
            ex_id,
            factory,
            EquipmentRemoveRuntimeFacts {
                owner_player_present: true,
                pack_add_enabled: runtime.pack_add_enabled,
                player_goods_package_extension: runtime.player_goods_package_extension,
                active_war_soul_blocks_headgear: runtime.active_war_soul_blocks_headgear,
            },
        );
        let mut effects = Vec::new();
        if let EquipmentRemoveOutcome::Removed(removed) = &mut outcome {
            self.equipment_changed = true;
            let now_seconds = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            self.unregister_goods_ai(&mut removed.goods, factory, now_seconds);
        }
        if let EquipmentRemoveOutcome::Removed(removed) = &outcome
            && let Some(player_effects) = removed.event.player_effects
        {
            if player_effects.clear_war_soul_status && self.set_war_soul_status(0) {
                effects.push(PlayerEquipmentRemoveEffect::WarSoulStatusAround {
                    message_type: BATTLE_FAIRY_STATUS_MESSAGE_TYPE,
                    player_id,
                    values: [400, player_id],
                });
            }
            if player_effects.delete_war_soul_skill {
                for (skill_id, _) in war_soul_skill_entries_from_goods(&removed.goods, factory) {
                    let _deleted = self.move_shape.delete_skill(skill_id, skill_factory);
                    effects.push(PlayerEquipmentRemoveEffect::WarSoulSkillDetached { skill_id });
                    if let Some(skill) = self.move_shape.skill(skill_id, skill_factory) {
                        effects.push(PlayerEquipmentRemoveEffect::SkillRemoved(
                            BattleFairySkillRemoved {
                                message_type: BATTLE_FAIRY_SKILL_REMOVED_MESSAGE_TYPE,
                                player_id,
                                skill_id,
                                skill_name: skill.name(skill_factory).map(<[u8]>::to_vec),
                            },
                        ));
                    }
                }
            }
            if player_effects.recompute_without_removed_slot {
                let recompute = recompute_properties(self);
                self.apply_recomputed_combat_properties(recompute.properties, factory);
                effects.push(
                    PlayerEquipmentRemoveEffect::PropertiesChangedWithoutRemovedSlot {
                        column: removed.event.column,
                        combat_properties: self.combat_properties,
                        ci_qing_result_values: recompute.ci_qing_result_values,
                    },
                );
            }
            if player_effects.clamp_hp_and_mp {
                let previous_health = self.health();
                let previous_mana = self.mana();
                self.set_health(previous_health);
                self.set_mana(previous_mana);
                effects.push(PlayerEquipmentRemoveEffect::VitalsClamped {
                    previous_health,
                    current_health: self.health(),
                    previous_mana,
                    current_mana: self.mana(),
                });
            }
            effects.push(PlayerEquipmentRemoveEffect::AroundUpdate(
                player_effects.around_update,
            ));
        }
        PlayerEquipmentRemoveReport {
            player_id,
            outcome,
            effects: effects.into_iter().collect(),
        }
    }

    /// Полный player-tail positional `CEquipmentContainer::Add`. Timed и
    /// goods-AI partial effects остаются наблюдаемы даже при late block; skill,
    /// properties, around и package-log выполняются только после commit.
    pub(crate) fn add_equipment_goods(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
        runtime: PlayerEquipmentAddRuntimeFacts,
        recompute_properties: &mut dyn FnMut(&mut CPlayer) -> PlayerPropertyRecompute,
        publish_effect: &mut dyn FnMut(&CPlayer, PlayerEquipmentAddEffect),
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> PlayerEquipmentAddReport {
        let player_id = self.player_id();
        let previous_expanded_package_num = self.equipment.expanded_package_num();
        let can_mount_result = incoming
            .as_ref()
            .map_or(0, |goods| self.can_mount_equip(goods, factory));
        let container_runtime = EquipmentAddRuntimeFacts {
            owner_player: Some(EquipmentOwnerPlayerFacts { can_mount_result }),
            pack_add_enabled: runtime.pack_add_enabled,
            now: runtime.now,
        };
        let outcome = {
            let current_ticket = self.current_ticket;
            let goods_ai_tree = &mut self.goods_ai_tree;
            let goods_ai_delete_queue = &mut self.goods_ai_delete_queue;
            let mut register_with_goods_ai = |goods: &mut CGoods| {
                if let Some((ticket, goods_id)) = Self::prepare_goods_ai_registration_with_clock(
                    current_ticket,
                    goods,
                    factory,
                    &mut crate::gameserver::gameserver::game::game_wall_time_seconds,
                ) {
                    Self::record_goods_ai_registration_in(
                        current_ticket,
                        goods_ai_tree,
                        goods_ai_delete_queue,
                        ticket,
                        goods_id,
                    );
                }
            };
            if position == u32::MAX {
                self.equipment.add_preferred(
                    incoming,
                    factory,
                    container_runtime,
                    &mut register_with_goods_ai,
                )
            } else {
                self.equipment.add_at(
                    position,
                    incoming,
                    factory,
                    container_runtime,
                    &mut register_with_goods_ai,
                )
            }
        };
        if matches!(&outcome, EquipmentAddOutcome::Added(_)) {
            self.equipment_changed = true;
        }
        if let EquipmentAddOutcome::Added(added) = &outcome
            && let Some(player_effects) = added.player_effects
        {
            if added.package_extension_applied {
                self.equipment
                    .set_expanded_package_num_snapshot(previous_expanded_package_num);
            }
            if player_effects.add_war_soul_skill
                && let Some(goods) = self.equipment.get_goods(added.column.position())
            {
                for (skill_id, level) in war_soul_skill_entries_from_goods(goods, factory) {
                    let _added = self.move_shape.add_skill(skill_id, level, skill_factory);
                    publish_effect(
                        self,
                        PlayerEquipmentAddEffect::WarSoulSkillAttached { skill_id, level },
                    );
                    if let Some(skill) = self.move_shape.skill(skill_id, skill_factory) {
                        publish_effect(
                            self,
                            PlayerEquipmentAddEffect::SkillAdded(
                                battle_fairy_skill_snapshot(player_id, skill, skill_factory),
                            ),
                        );
                    }
                }
            }
            if player_effects.recompute_properties {
                let recompute = recompute_properties(self);
                self.apply_recomputed_combat_properties(recompute.properties, factory);
                publish_effect(
                    self,
                    PlayerEquipmentAddEffect::PropertiesChanged {
                        combat_properties: self.combat_properties,
                        ci_qing_result_values: recompute.ci_qing_result_values,
                    },
                );
            }
            publish_effect(
                self,
                PlayerEquipmentAddEffect::AroundUpdate(player_effects.around_update),
            );
            if added.package_extension_applied {
                self.equipment.set_expanded_package_num_snapshot(
                    previous_expanded_package_num.wrapping_add(added.package_extension_delta),
                );
                publish_effect(
                    self,
                    PlayerEquipmentAddEffect::PackageExtensionLogged {
                        category: "PackExpand",
                        string_id: "KR002",
                        expanded_package_num: self.equipment.expanded_package_num(),
                    },
                );
            }
        }
        if let EquipmentAddOutcome::Added(added) = &outcome
            && let Some(goods) = self.equipment.get_goods(added.column.position())
        {
            let additional = goods.addon_property_value(factory, GAP_EXCEPTION_STATE, 1) as u32;
            on_goods_added(self, additional);
        }
        PlayerEquipmentAddReport { player_id, outcome }
    }

    /// Завершает принадлежащий `CGame` хвост области: `spatial_applied`
    /// означает найденную нужную area, а не изменение её map entry.
    pub(crate) const fn apply_war_soul_action(
        &mut self,
        action: BattleFairyWarSoulAction,
        spatial_applied: bool,
    ) {
        match action {
            BattleFairyWarSoulAction::SetPosition { target, .. } if spatial_applied => {
                self.war_soul_point = target;
            }
            BattleFairyWarSoulAction::Delete {
                player_position, ..
            } if spatial_applied => {
                self.war_soul_point = player_position;
            }
            BattleFairyWarSoulAction::SetPosition { .. }
            | BattleFairyWarSoulAction::Delete { .. } => {}
        }
    }

    /// Active WarSoul tail `CPlayer::OnEnterRegion`: visual float координаты
    /// возвращаются к клетке хозяина; spatial point применяет координатор
    /// после target-area gate и End(int,0) выбранного навыка.
    pub(crate) fn prepare_war_soul_region_entry(
        &mut self,
    ) -> Option<(BattleFairyWarSoulAction, u32, u32)> {
        if self.war_soul_state != 1 {
            return None;
        }
        let target = WarSoulPoint {
            x: self.shape().get_tile_x().ok()?,
            y: self.shape().get_tile_y().ok()?,
        };
        self.war_soul_visual_x_bits = (target.x as f32).to_bits();
        self.war_soul_visual_y_bits = (target.y as f32).to_bits();
        Some((
            BattleFairyWarSoulAction::SetPosition {
                previous: self.war_soul_point,
                target,
            },
            self.war_soul_visual_x_bits,
            self.war_soul_visual_y_bits,
        ))
    }

    /// Один проход живой ветви `ComputeWarSoulXY`. `Some(false)` означает
    /// найденный текущий навык боевой феи с `IsRestored()==0`; `None` точно
    /// соответствует отсутствующему навыку и не блокирует следование.
    pub(crate) fn compute_war_soul_xy(
        &mut self,
        current_war_soul_skill_restored: Option<bool>,
    ) -> BattleFairyFollowPlan {
        let player_id = self.player_id();
        let mut plan = BattleFairyFollowPlan {
            player_id,
            outcome: BattleFairyFollowOutcome::NotSummoned,
            region_id: self.server_region_id,
            spatial_action: None,
            effects: GameEffectJournal::default(),
        };
        if current_war_soul_skill_restored == Some(false) {
            plan.outcome = BattleFairyFollowOutcome::ActiveSkill;
            return plan;
        }
        if self.war_soul_state != 1 {
            return plan;
        }
        let (tile_x, tile_y) = match (self.shape().get_tile_x(), self.shape().get_tile_y()) {
            (Ok(x), Ok(y)) => (x, y),
            (Err(error), _) | (_, Err(error)) => {
                plan.outcome = BattleFairyFollowOutcome::CoordinateBlocked(error);
                return plan;
            }
        };
        let current_x = tile_x as f32;
        let current_y = tile_y as f32;
        let mut visual_x = f32::from_bits(self.war_soul_visual_x_bits);
        let mut visual_y = f32::from_bits(self.war_soul_visual_y_bits);
        let delta_x = current_x - visual_x;
        let delta_y = current_y - visual_y;
        let distance = (delta_x * delta_x + delta_y * delta_y).sqrt().abs();
        if !distance.is_finite() {
            plan.outcome = BattleFairyFollowOutcome::NonFiniteVisualState;
            return plan;
        }
        if distance < 0.5 {
            plan.outcome = BattleFairyFollowOutcome::InsideDeadZone;
            return plan;
        }

        let (target, outcome) = if distance <= 5.0 {
            let coefficient = if distance > 3.75 {
                0.265f32
            } else if distance > 0.75 {
                0.065f32
            } else {
                0.045f32
            };
            let step = distance * (coefficient + coefficient);
            if (current_x - visual_x).abs() > 0.1 {
                visual_x = if current_x <= visual_x {
                    visual_x - step
                } else {
                    visual_x + step
                };
            }
            if (current_y - visual_y).abs() > 0.1 {
                visual_y = if current_y <= visual_y {
                    visual_y - step
                } else {
                    visual_y + step
                };
            }
            (
                WarSoulPoint {
                    // EXE временно ставит x87 RC=truncate перед обоими fistp.
                    x: visual_x.trunc() as i32,
                    y: visual_y.trunc() as i32,
                },
                BattleFairyFollowOutcome::Moved,
            )
        } else {
            visual_x = current_x;
            visual_y = current_y;
            (
                WarSoulPoint {
                    x: tile_x,
                    y: tile_y,
                },
                BattleFairyFollowOutcome::Snapped,
            )
        };
        self.war_soul_visual_x_bits = visual_x.to_bits();
        self.war_soul_visual_y_bits = visual_y.to_bits();
        plan.outcome = outcome;
        plan.spatial_action = Some(BattleFairyWarSoulAction::SetPosition {
            previous: self.war_soul_point,
            target,
        });
        plan.effects.push(BattleFairyFollowEffect::AroundMove {
            message_type: BATTLE_FAIRY_MOVE_MESSAGE_TYPE,
            player_id,
            object_type: 700,
            x: visual_x.to_bits(),
            y: visual_y.to_bits(),
        });
        plan
    }

    /// Мёртвая ветвь сразу после `CMoveShape::AI`: пространственная позиция
    /// получает точное `(-1,-1)`, обе визуальные координаты `float` становятся
    /// `-1.0`, но исходник не публикует пакет движения вокруг.
    pub(crate) fn clear_dead_war_soul_xy(&mut self) -> BattleFairyFollowPlan {
        let target = WarSoulPoint { x: -1, y: -1 };
        self.war_soul_visual_x_bits = (-1.0f32).to_bits();
        self.war_soul_visual_y_bits = (-1.0f32).to_bits();
        BattleFairyFollowPlan {
            player_id: self.player_id(),
            outcome: BattleFairyFollowOutcome::Dead,
            region_id: self.server_region_id,
            spatial_action: Some(BattleFairyWarSoulAction::SetPosition {
                previous: self.war_soul_point,
                target,
            }),
            effects: GameEffectJournal::default(),
        }
    }

    /// Владеющая state/container середина оставшегося `CPlayer::AI` tail.
    /// GoodsAI/delete выполняется соседним CGame caller-ом до ticket mutation;
    /// packet expansion принадлежит самому player-у. Flash и TaoZhuang затем
    /// завершаются concrete CGame owner-ом.
    pub(crate) fn advance_ai_ticket_and_packet(
        &mut self,
        pack_add_enabled: bool,
    ) -> (u32, Option<u32>) {
        self.current_ticket = self.current_ticket.wrapping_add(1);
        let expanded_package_num = pack_add_enabled.then(|| {
            let expanded = self.equipment.expanded_package_num();
            self.packet.set_all_inactive();
            self.packet.apply_player_expansion_limit(expanded);
            expanded
        });
        (self.current_ticket, expanded_package_num)
    }

    pub(crate) const fn current_ticket(&self) -> u32 {
        self.current_ticket
    }

    /// Exact `CheckAddGoods -> ComputeTicket`: remaining lifetime переводится
    /// в AI tickets множителем 12.5. Пара возвращается наружу, чтобы container
    /// borrow завершился до mutation player map-а.
    pub(crate) fn prepare_goods_ai_registration(
        current_ticket: u32,
        goods: &mut CGoods,
        factory: &CGoodsFactory,
        now_seconds: u64,
    ) -> Option<(u32, CGuid)> {
        Self::prepare_goods_ai_registration_with_clock(
            current_ticket, goods, factory, &mut || now_seconds,
        )
    }

    fn prepare_goods_ai_registration_with_clock(
        current_ticket: u32,
        goods: &mut CGoods,
        factory: &CGoodsFactory,
        now_seconds: &mut dyn FnMut() -> u64,
    ) -> Option<(u32, CGuid)> {
        if !goods.query_attribute(GAP_GOODS_LIFE_TYPE) || goods.add_ticket() != 0 {
            return None;
        }
        let time_type = goods.goods_time_type(factory);
        let mut start = goods.start_point(factory);
        if !matches!(time_type, 1 | 3) && (!matches!(time_type, 2 | 4) || start == 0) {
            return None;
        }
        let now_seconds = now_seconds();
        if start == 0 {
            goods.set_start_point(now_seconds);
            start = now_seconds;
        }
        let elapsed = if start < now_seconds {
            (now_seconds as u32).wrapping_sub(start as u32)
        } else {
            0
        };
        let lifetime = goods.goods_lifetime(factory);
        if elapsed.wrapping_add(5) >= lifetime {
            return Some((current_ticket, goods.identity().ex_id));
        }
        let delta = ((u64::from(lifetime.wrapping_sub(elapsed)) * 25) / 2) as u32;
        if delta == 0 {
            return None;
        }
        let ticket = current_ticket.wrapping_add(delta);
        goods.set_add_ticket(ticket);
        (goods.add_ticket() != 0).then_some((ticket, goods.identity().ex_id))
    }

    pub(crate) fn record_goods_ai_registration(&mut self, ticket: u32, goods_id: CGuid) {
        Self::record_goods_ai_registration_in(
            self.current_ticket,
            &mut self.goods_ai_tree,
            &mut self.goods_ai_delete_queue,
            ticket,
            goods_id,
        );
    }

    fn record_goods_ai_registration_in(
        current_ticket: u32,
        goods_ai_tree: &mut BTreeMap<u32, BTreeSet<CGuid>>,
        goods_ai_delete_queue: &mut VecDeque<BTreeSet<CGuid>>,
        ticket: u32,
        goods_id: CGuid,
    ) {
        if ticket <= current_ticket {
            goods_ai_delete_queue.push_back(BTreeSet::from([goods_id]));
        } else {
            goods_ai_tree.entry(ticket).or_default().insert(goods_id);
        }
    }

    fn unregister_goods_ai(
        &mut self,
        goods: &mut CGoods,
        factory: &CGoodsFactory,
        now_seconds: u64,
    ) {
        let ticket = goods.add_ticket();
        if ticket == 0 {
            return;
        }
        let goods_id = goods.identity().ex_id;
        let mut registered = false;
        if let Some(bucket) = self.goods_ai_tree.get_mut(&ticket) {
            registered = bucket.remove(&goods_id);
            if bucket.is_empty() {
                self.goods_ai_tree.remove(&ticket);
            }
        }
        if !registered {
            return;
        }
        let start = goods.start_point(factory);
        let elapsed = if start < now_seconds {
            (now_seconds as u32).wrapping_sub(start as u32)
        } else {
            0
        };
        let lifetime = goods.goods_lifetime(factory);
        if elapsed.wrapping_add(5) >= lifetime {
            let _ = goods.set_addon_property_value_core(GAP_GOODS_LIFE_TYPE, 1, 0);
            self.goods_ai_delete_queue
                .push_back(BTreeSet::from([goods_id]));
            return;
        }
        let _ = goods.set_addon_property_value_core(
            GAP_GOODS_LIFE_TYPE,
            1,
            lifetime.wrapping_sub(elapsed) as i32,
        );
        goods.set_start_point(0);
        goods.set_add_ticket(0);
    }

    pub(crate) fn register_goods_ai_by_id(
        &mut self,
        goods_id: CGuid,
        factory: &CGoodsFactory,
        now_seconds: u64,
    ) -> bool {
        let current_ticket = self.current_ticket;
        let registration = self.get_goods_ai_by_id_mut(goods_id).and_then(|goods| {
            Self::prepare_goods_ai_registration(current_ticket, goods, factory, now_seconds)
        });
        let Some((ticket, goods_id)) = registration else {
            return false;
        };
        self.record_goods_ai_registration(ticket, goods_id);
        true
    }

    /// Выполняет `CPlayer::DelItemFromGoodsAiTree` для предмета, который
    /// остаётся в принадлежащем игроку контейнере. Это позволяет сценарному
    /// владельцу изменить временные поля между удалением и повторной
    /// регистрацией, не перемещая сам предмет.
    pub(crate) fn unregister_goods_ai_by_id(
        &mut self,
        goods_id: CGuid,
        factory: &CGoodsFactory,
        now_seconds: u64,
    ) -> bool {
        let Some(ticket) = self.get_goods_by_id(goods_id).map(CGoods::add_ticket) else {
            return false;
        };
        if ticket == 0 {
            return false;
        }
        let Some(bucket) = self.goods_ai_tree.get_mut(&ticket) else {
            return false;
        };
        let _ = bucket.remove(&goods_id);
        if bucket.is_empty() {
            self.goods_ai_tree.remove(&ticket);
        }

        let Some(goods) = self.get_goods_by_id_mut(goods_id) else {
            return false;
        };
        let start = goods.start_point(factory);
        let elapsed = if start < now_seconds {
            (now_seconds as u32).wrapping_sub(start as u32)
        } else {
            0
        };
        let lifetime = goods.goods_lifetime(factory);
        if elapsed.wrapping_add(5) >= lifetime {
            goods.set_goods_lifetime(0);
            self.goods_ai_delete_queue
                .push_back(BTreeSet::from([goods_id]));
            return true;
        }
        goods.set_goods_lifetime(lifetime.wrapping_sub(elapsed));
        goods.set_start_point(0);
        goods.set_add_ticket(0);
        true
    }

    /// Завершает удаление предмета со склада через тот же реестр `GoodsAI`,
    /// который обслуживает перемещения между контейнерами игрока.
    pub(crate) fn unregister_depot_goods_ai(
        &mut self,
        goods: &mut CGoods,
        factory: &CGoodsFactory,
        now_seconds: u64,
    ) {
        self.unregister_goods_ai(goods, factory, now_seconds);
    }

    fn get_goods_ai_by_id_mut(&mut self, goods_id: CGuid) -> Option<&mut CGoods> {
        if self.hand.find(goods_id).is_some() {
            return self.hand.find_mut(goods_id);
        }
        if self.packet.base().find(goods_id).is_some() {
            return self.packet.base_mut().find_mut(goods_id);
        }
        if self.equipment.find(goods_id).is_some() {
            return self.equipment.find_mut(goods_id);
        }
        if self.depot.base().base().find(goods_id).is_some() {
            return self.depot.base_mut().base_mut().find_mut(goods_id);
        }
        if self.fairy_container.base().base().find(goods_id).is_some() {
            return self
                .fairy_container
                .base_mut()
                .base_mut()
                .find_mut(goods_id);
        }
        if self
            .battle_fairy_container
            .base()
            .base()
            .find(goods_id)
            .is_some()
        {
            return self
                .battle_fairy_container
                .base_mut()
                .base_mut()
                .find_mut(goods_id);
        }
        if self.auction_listing.base().find(goods_id).is_some() {
            return self.auction_listing.base_mut().find_mut(goods_id);
        }
        if self.auction_goods.base().find(goods_id).is_some() {
            return self.auction_goods.base_mut().find_mut(goods_id);
        }
        if self.ci_qing.base().find(goods_id).is_some() {
            return self.ci_qing.base_mut().find_mut(goods_id);
        }
        self.ci_qing_compose.base_mut().find_mut(goods_id)
    }

    pub(crate) fn done_goods_ai_tree(&mut self) -> usize {
        let due_tickets: Vec<_> = self
            .goods_ai_tree
            .range(..=self.current_ticket)
            .map(|(&ticket, _)| ticket)
            .collect();
        let mut moved = 0;
        for ticket in due_tickets {
            if let Some(due) = self.goods_ai_tree.remove(&ticket)
                && !due.is_empty()
            {
                moved += due.len();
                self.goods_ai_delete_queue.push_back(due);
            }
        }
        moved
    }

    /// Exact `DoneDelList`: только front bucket и максимум четыре GUID за AI.
    pub(crate) fn take_goods_ai_deletions(&mut self) -> Vec<CGuid> {
        while self
            .goods_ai_delete_queue
            .front()
            .is_some_and(BTreeSet::is_empty)
        {
            self.goods_ai_delete_queue.pop_front();
        }
        let Some(front) = self.goods_ai_delete_queue.front_mut() else {
            return Vec::new();
        };
        let result: Vec<_> = front.iter().copied().take(4).collect();
        for goods_id in &result {
            front.remove(goods_id);
        }
        if front.is_empty() {
            self.goods_ai_delete_queue.pop_front();
        }
        result
    }

    pub(crate) fn goods_ai_location(&self, goods_id: CGuid) -> Option<PlayerGoodsAiLocation> {
        let located = |extend_id, position| PlayerGoodsAiLocation {
            extend_id,
            position,
        };
        self.packet
            .query_goods_position(goods_id)
            .map(|position| located(1, position))
            .or_else(|| {
                self.equipment
                    .query_goods_position_by_id(goods_id)
                    .map(|column| located(2, column.position()))
            })
            .or_else(|| {
                self.hand
                    .query_goods_position(goods_id)
                    .map(|p| located(3, p))
            })
            .or_else(|| {
                self.depot
                    .base()
                    .query_goods_position(goods_id)
                    .map(|p| located(9, p))
            })
            .or_else(|| {
                self.fairy_container
                    .base()
                    .query_goods_position(goods_id)
                    .map(|p| located(11, p))
            })
            .or_else(|| {
                self.battle_fairy_container
                    .base()
                    .query_goods_position(goods_id)
                    .map(|p| located(12, p))
            })
            .or_else(|| {
                self.auction_listing
                    .query_goods_position(goods_id)
                    .map(|p| located(13, p))
            })
            .or_else(|| {
                self.auction_goods
                    .query_goods_position(goods_id)
                    .map(|p| located(14, p))
            })
            .or_else(|| {
                self.ci_qing
                    .query_goods_position(goods_id)
                    .map(|p| located(16, p))
            })
            .or_else(|| {
                self.ci_qing_compose
                    .query_goods_position(goods_id)
                    .map(|p| located(17, p))
            })
    }

    /// Exact `DropParticularGoodsWhenLost` snapshot: packet, equipment, затем
    /// hand; выбирается бит `0x04`, а actual `DropGoods` mutation выполняется
    /// caller-ом после завершения обхода, чтобы container erase не сбивал его.
    pub(crate) fn particular_goods_drops(
        &self,
        factory: &CGoodsFactory,
    ) -> Vec<PlayerParticularGoodsDrop> {
        let mut drops = Vec::new();
        let mut push = |extend_id: i32, position: u32, goods: &CGoods| {
            if goods.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1) & 0x04 != 0 {
                drops.push(PlayerParticularGoodsDrop {
                    location: PlayerGoodsAiLocation {
                        extend_id,
                        position,
                    },
                    goods_id: goods.identity().ex_id,
                    amount: goods.amount(),
                });
            }
        };
        for position in 0..self.packet.size() {
            if let Some(goods) = self.packet.get_goods(position) {
                push(1, position, goods);
            }
        }
        for position in 0..17 {
            if let Some(goods) = self.equipment.get_goods(position) {
                push(2, position, goods);
            }
        }
        for goods in self.hand.traversing_goods() {
            if let Some(position) = self.hand.query_goods_position(goods.identity().ex_id) {
                push(3, position, goods);
            }
        }
        drops
    }

    /// Точный снимок `DropParticularGoodsWhenRecall`: рюкзак, экипировка,
    /// затем рука; выбирается бит `0x80`. Фактическое перемещение на землю
    /// выполняет вызывающий владелец после обхода, чтобы удаления не меняли
    /// порядок кандидатов.
    pub(crate) fn particular_goods_recall_drops(
        &self,
        factory: &CGoodsFactory,
    ) -> Vec<PlayerParticularGoodsDrop> {
        let mut drops = Vec::new();
        let mut push = |extend_id: i32, position: u32, goods: &CGoods| {
            if goods.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1) & 0x80 != 0 {
                drops.push(PlayerParticularGoodsDrop {
                    location: PlayerGoodsAiLocation { extend_id, position },
                    goods_id: goods.identity().ex_id,
                    amount: goods.amount(),
                });
            }
        };
        for position in 0..self.packet.size() {
            if let Some(goods) = self.packet.get_goods(position) {
                push(1, position, goods);
            }
        }
        for position in 0..17 {
            if let Some(goods) = self.equipment.get_goods(position) {
                push(2, position, goods);
            }
        }
        for goods in self.hand.traversing_goods() {
            if let Some(position) = self.hand.query_goods_position(goods.identity().ex_id) {
                push(3, position, goods);
            }
        }
        drops
    }

    pub(crate) fn script_equipment_base_index(&self, position: u32) -> Option<u32> {
        self.equipment
            .get_goods(position)
            .map(CGoods::base_properties_index)
    }

    pub(crate) fn upgrade_script_equipment(
        &mut self,
        position: u32,
        level_delta: i32,
        factory: &CGoodsFactory,
        mut random_below: impl FnMut(i32) -> i32,
    ) -> Option<ShapeIdentity> {
        let goods = self.equipment.get_goods_mut(position)?;
        let current_level = goods.addon_property_value(factory, GAP_WEAPON_LEVEL, 1);
        let target_level = (current_level as u32).wrapping_add(level_delta as u32) as i32;
        let _ = factory.upgrade_equipment(goods, target_level, &mut random_below);
        Some(goods.identity())
    }

    /// Stable snapshot контейнеров для `CPlayer::OnDied`: caller может
    /// удалять предметы, не инвалидируя исходный обход. Бит `0x08` задаёт
    /// unconditional death drop, бит `0x02` запрещает table-driven drop.
    pub(crate) fn death_goods_candidates(
        &self,
        factory: &CGoodsFactory,
    ) -> Vec<PlayerDeathGoodsCandidate> {
        let mut candidates = Vec::new();
        let mut push = |extend_id: i32, position: u32, goods: &CGoods| {
            let particular = goods.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1);
            candidates.push(PlayerDeathGoodsCandidate {
                location: PlayerGoodsAiLocation {
                    extend_id,
                    position,
                },
                goods_id: goods.identity().ex_id,
                amount: goods.amount(),
                price: goods.price(),
                name: goods.name().to_vec(),
                particular_on_death: particular & 0x08 != 0,
                table_drop_allowed: particular & 0x02 == 0,
            });
        };
        for position in 0..self.packet.size() {
            if let Some(goods) = self.packet.get_goods(position) {
                push(1, position, goods);
            }
        }
        for position in 0..17 {
            if let Some(goods) = self.equipment.get_goods(position) {
                push(2, position, goods);
            }
        }
        for goods in self.hand.traversing_goods() {
            if let Some(position) = self.hand.query_goods_position(goods.identity().ex_id) {
                push(3, position, goods);
            }
        }
        candidates
    }

    pub(crate) fn owned_goods_location(
        &self,
        extend_id: i32,
        goods_id: CGuid,
    ) -> Option<PlayerGoodsAiLocation> {
        let location = |position| PlayerGoodsAiLocation {
            extend_id,
            position,
        };
        match extend_id {
            1 | 2 | 3 | 9 | 11 | 12 | 13 | 14 | 16 | 17 => self
                .goods_ai_location(goods_id)
                .filter(|actual| actual.extend_id == extend_id),
            4 => self
                .wallet
                .get_goods(0)
                .filter(|goods| goods.identity().ex_id == goods_id)
                .map(|_| location(0)),
            5 => self
                .yuan_bao
                .get_goods(0)
                .filter(|goods| goods.identity().ex_id == goods_id)
                .map(|_| location(0)),
            6 => self
                .ji_fen
                .get_goods(0)
                .filter(|goods| goods.identity().ex_id == goods_id)
                .map(|_| location(0)),
            8 => self
                .bank
                .get_goods(0)
                .filter(|goods| goods.identity().ex_id == goods_id)
                .map(|_| location(0)),
            10 => (self.enhancement_selected_goods_id() == Some(goods_id))
                .then(|| self.goods_ai_location(goods_id))
                .flatten(),
            15 => self
                .auction_wallet
                .get_goods(0)
                .filter(|goods| goods.identity().ex_id == goods_id)
                .map(|_| location(0)),
            _ => None,
        }
    }

    /// Полный non-equipment `DeleteGoods(..., 1, true)` storage core. Client
    /// wire и World audit формирует CGame после сохранения удалённого snapshot.
    pub(crate) fn delete_owned_goods(
        &mut self,
        location: PlayerGoodsAiLocation,
        goods_id: CGuid,
        requested_amount: u32,
        factory: &CGoodsFactory,
    ) -> Option<PlayerGoodsAiDeletion> {
        macro_rules! delete_volume {
            ($container:expr) => {{
                let container = $container;
                let goods = container.get_goods(location.position)?.clone();
                (goods.identity().ex_id == goods_id).then_some(())?;
                let previous_amount = goods.amount();
                let removed_amount = requested_amount.min(previous_amount);
                let remaining_amount = previous_amount.wrapping_sub(removed_amount);
                let listeners = if remaining_amount != 0 {
                    container
                        .get_goods_mut(location.position)?
                        .set_amount(remaining_amount);
                    Vec::new()
                } else {
                    let removed = container.remove_goods(goods_id)?;
                    let taken = match removed {
                        VolumeGoodsRemoveOutcome::Removed(taken)
                        | VolumeGoodsRemoveOutcome::RemovedButCellMissing(taken) => taken,
                    };
                    match taken {
                        AmountLimitGoodsTaken::Removed(removed) => removed.listeners,
                        AmountLimitGoodsTaken::Split(_) => unreachable!("full GoodsAI delete"),
                    }
                };
                Some(PlayerGoodsAiDeletion {
                    location,
                    goods,
                    previous_amount,
                    removed_amount,
                    remaining_amount,
                    listeners,
                })
            }};
        }
        match location.extend_id {
            1 => delete_volume!(&mut self.packet),
            2 => {
                let goods = self.equipment.get_goods(location.position)?.clone();
                (goods.identity().ex_id == goods_id).then_some(())?;
                let previous_amount = goods.amount();
                (requested_amount < previous_amount).then_some(())?;
                self.equipment
                    .find_mut(goods_id)?
                    .set_amount(previous_amount.wrapping_sub(requested_amount));
                Some(PlayerGoodsAiDeletion {
                    location,
                    goods,
                    previous_amount,
                    removed_amount: requested_amount,
                    remaining_amount: previous_amount.wrapping_sub(requested_amount),
                    listeners: Vec::new(),
                })
            }
            3 => {
                let goods = self.hand.get_goods(location.position)?.clone();
                (goods.identity().ex_id == goods_id).then_some(())?;
                let previous_amount = goods.amount();
                let removed_amount = requested_amount.min(previous_amount);
                let remaining_amount = previous_amount.wrapping_sub(removed_amount);
                let listeners = if remaining_amount != 0 {
                    self.hand.find_mut(goods_id)?.set_amount(remaining_amount);
                    Vec::new()
                } else {
                    self.hand.remove_goods(goods_id)?.listeners
                };
                Some(PlayerGoodsAiDeletion {
                    location,
                    goods,
                    previous_amount,
                    removed_amount,
                    remaining_amount,
                    listeners,
                })
            }
            4 | 5 | 6 | 8 | 15 => {
                macro_rules! delete_currency {
                    ($container:expr) => {{
                        let container = $container;
                        let goods = container.get_goods(location.position)?.clone();
                        (goods.identity().ex_id == goods_id).then_some(())?;
                        let previous_amount = goods.amount();
                        let removed_amount = requested_amount.min(previous_amount);
                        let remaining_amount = previous_amount.wrapping_sub(removed_amount);
                        let listeners = if removed_amount == 0 {
                            Vec::new()
                        } else {
                            let template = goods.clone();
                            let taken = container.take_goods(
                                location.position,
                                removed_amount,
                                factory,
                                |_| Some(template.clone()),
                            )?;
                            match taken {
                                CurrencyGoodsTaken::Removed(removed) => removed.listeners,
                                CurrencyGoodsTaken::Split(_) => Vec::new(),
                            }
                        };
                        Some(PlayerGoodsAiDeletion {
                            location,
                            goods,
                            previous_amount,
                            removed_amount,
                            remaining_amount,
                            listeners,
                        })
                    }};
                }
                match location.extend_id {
                    4 => {
                        let deletion = delete_currency!(&mut self.wallet);
                        if deletion.is_some() {
                            self.money = self.wallet.currency_amount();
                        }
                        deletion
                    }
                    5 => delete_currency!(&mut self.yuan_bao),
                    6 => delete_currency!(&mut self.ji_fen),
                    8 => delete_currency!(&mut self.bank),
                    15 => delete_currency!(&mut self.auction_wallet),
                    _ => unreachable!(),
                }
            }
            9 => delete_volume!(self.depot.base_mut()),
            11 => delete_volume!(self.fairy_container.base_mut()),
            12 => delete_volume!(self.battle_fairy_container.base_mut()),
            13 => delete_volume!(&mut self.auction_listing),
            14 => delete_volume!(&mut self.auction_goods),
            16 => delete_volume!(&mut self.ci_qing),
            17 => delete_volume!(&mut self.ci_qing_compose),
            _ => None,
        }
    }

    /// Периодический префикс `CPlayer::AI`: нулевой HP надетой боевой феи при
    /// каждом проходе повторно нормализует четыре поля состояния и вызывает
    /// `PropertiesChanged`. Исходник не удаляет устаревшую запись карты области
    /// и не рассылает состояние.
    pub(crate) fn refresh_battle_fairy_death(
        &mut self,
        factory: &CGoodsFactory,
    ) -> BattleFairyDeathOutcome {
        let Some(goods) = self.equipment.get_goods(10) else {
            return BattleFairyDeathOutcome::MissingHeadgear;
        };
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            return BattleFairyDeathOutcome::NotBattleFairy;
        }
        if goods.addon_property_value(factory, GAP_BF_HP, 1) != 0 {
            return BattleFairyDeathOutcome::Alive;
        }
        self.battle_fairy_summoned = false;
        self.war_soul_state = 0;
        self.set_battle_fairy_recall(true);
        self.set_battle_fairy_died(true);
        BattleFairyDeathOutcome::Died
    }

    fn apply_battle_fairy_property(
        &mut self,
        cell: BattleFairyCell,
        addons: BattleFairyGearAddons,
        delta: i32,
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Option<BattleFairyDefaultGoodsUpdate> {
        if delta == 0 || !is_battle_fairy_property_cell(cell) {
            return None;
        }
        let occupation = usize::from(self.base_properties.occupation).min(2);
        let player_id = self.player_id();
        let battle_fairy = self.equipment.get_goods_mut(10)?;
        if battle_fairy.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            return None;
        }

        for (source, target) in [
            (addons.attack, GAP_BF_ATTACK),
            (addons.sprite, GAP_BF_SPRITE),
            (addons.strength, GAP_BF_STRENGH),
            (addons.brave, GAP_BF_BRAVE),
            (addons.agility, GAP_BF_AGILITY),
            (addons.spiritualism, GAP_BF_SPRITUALISM),
            (addons.blast, GAP_BF_BLAST),
            (addons.cut_hurt, GAP_BF_CUT_HURT_SCALE),
        ] {
            add_battle_fairy_addon(battle_fairy, factory, target, source.wrapping_mul(delta));
        }
        add_battle_fairy_addon(
            battle_fairy,
            factory,
            GAP_BF_MAX_HP,
            addons
                .strength
                .wrapping_add(addons.life)
                .wrapping_mul(delta),
        );
        add_battle_fairy_addon(
            battle_fairy,
            factory,
            GAP_BF_MAX_MP,
            addons
                .spiritualism
                .wrapping_add(addons.mana)
                .wrapping_mul(delta),
        );
        clamp_battle_fairy_current(battle_fairy, factory, GAP_BF_HP, GAP_BF_MAX_HP);
        clamp_battle_fairy_current(battle_fairy, factory, GAP_BF_MP, GAP_BF_MAX_MP);

        let strength = f64::from(addons.strength) * f64::from(delta) * 0.00001;
        let brave = f64::from(addons.brave) * f64::from(delta) * 0.00001;
        let agility = f64::from(addons.agility) * f64::from(delta) * 0.00001;
        let spiritualism = f64::from(addons.spiritualism) * f64::from(delta) * 0.00001;
        let combat = &mut self.combat_properties;
        combat.maximum_hp = add_battle_fairy_u32(combat.maximum_hp, strength);
        combat.strength = add_battle_fairy_u32(combat.strength, brave);
        combat.maximum_attack = add_battle_fairy_u32(
            combat.maximum_attack,
            brave * f64::from(coefficients.str_to_max_attack[occupation]),
        );
        combat.burden = add_battle_fairy_u16(
            combat.burden,
            brave * f64::from(coefficients.str_to_burden[occupation]),
        );
        combat.dexterity = add_battle_fairy_u32(combat.dexterity, agility);
        combat.minimum_attack = add_battle_fairy_u32(
            combat.minimum_attack,
            agility * f64::from(coefficients.dex_to_min_attack[occupation]),
        );
        combat.reank = add_battle_fairy_u16(
            combat.reank,
            agility * f64::from(coefficients.dex_to_stiff[occupation]),
        );
        combat.intelligence = add_battle_fairy_u32(combat.intelligence, spiritualism);
        combat.element_modify = add_battle_fairy_i32(
            combat.element_modify,
            spiritualism * f64::from(coefficients.int_to_element[occupation]),
        );
        combat.maximum_mp = add_battle_fairy_u32(
            combat.maximum_mp,
            spiritualism * f64::from(coefficients.int_to_max_mp[occupation]),
        );
        combat.element_resistance = add_battle_fairy_u32(
            combat.element_resistance,
            spiritualism * f64::from(coefficients.int_to_resistant[occupation]),
        );

        // Подтверждённый RU quirk: четыре основных значения применяются
        // повторно после производных коэффициентов.
        combat.maximum_hp = add_battle_fairy_u32(combat.maximum_hp, strength);
        combat.strength = add_battle_fairy_u32(combat.strength, brave);
        combat.intelligence = add_battle_fairy_u32(combat.intelligence, spiritualism);
        combat.dexterity = add_battle_fairy_u32(combat.dexterity, agility);

        Some(BattleFairyDefaultGoodsUpdate {
            message_type: 0x0b_f918,
            player_id,
            goods: battle_fairy.identity(),
            old_client_payload: encode_old_client(battle_fairy),
        })
    }

    /// Exact positional `CBattleFairyContainer::Add`: для gear-ячеек
    /// `BFPropertyAdd(+1)` является ранним partial effect и сохраняется даже
    /// если base storage затем отвергнет товар.
    pub(crate) fn add_battle_fairy_goods(
        &mut self,
        cell: BattleFairyCell,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        owner_progress_allows: bool,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyEquipmentMutationReport {
        let player_id = self.player_id();
        let early_property = incoming.as_ref().and_then(|goods| {
            self.battle_fairy_container
                .property_effect_before_add(cell, goods, factory)
                .map(|effect| (effect, BattleFairyGearAddons::read(goods, factory)))
        });
        let mut property_applied = false;
        let mut effects = Vec::new();
        if let Some((BattleFairyPropertyAddEffect { cell, delta }, addons)) = early_property
            && let Some(update) = self.apply_battle_fairy_property(
                cell,
                addons,
                delta,
                factory,
                coefficients,
                encode_old_client,
            )
        {
            property_applied = true;
            effects.push(BattleFairyEquipmentMutationEffect::PropertiesChanged { player_id });
            effects.push(BattleFairyEquipmentMutationEffect::BattleFairyUpdated(
                update,
            ));
        }
        let outcome =
            self.battle_fairy_container
                .add_at(cell, incoming, factory, owner_progress_allows);
        BattleFairyEquipmentMutationReport {
            player_id,
            cell: Some(cell),
            property_applied,
            outcome: BattleFairyEquipmentMutationOutcome::Added(outcome),
            effects: effects.into_iter().collect(),
        }
    }

    /// Безпозиционный overload сначала читает catalog BF equip-place. Только
    /// валидная колонка достигает player property-tail; все typed reject-и
    /// остаются у container owner-а без выдуманного размещения.
    pub(crate) fn add_battle_fairy_goods_auto(
        &mut self,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        owner_progress_allows: bool,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyEquipmentMutationReport {
        let cell = incoming
            .as_ref()
            .and_then(|goods| factory.query_goods_base_properties(goods.base_properties_index()))
            .and_then(|properties| properties.battle_fairy_equip_place())
            .and_then(|position| BattleFairyCell::from_position(position as u32));
        if let Some(cell) = cell {
            return self.add_battle_fairy_goods(
                cell,
                incoming,
                factory,
                coefficients,
                owner_progress_allows,
                encode_old_client,
            );
        }
        let player_id = self.player_id();
        let outcome = self
            .battle_fairy_container
            .add(incoming, factory, owner_progress_allows);
        BattleFairyEquipmentMutationReport {
            player_id,
            cell: None,
            property_applied: false,
            outcome: BattleFairyEquipmentMutationOutcome::Added(outcome),
            effects: GameEffectJournal::default(),
        }
    }

    /// Exact `Remove`: base container отделяет goods до `BFPropertyAdd(-1)`;
    /// успешный property path сериализует battle fairy дважды — один раз в
    /// `BFPropertyAdd`, затем ещё раз в override `Remove`.
    pub(crate) fn remove_battle_fairy_goods(
        &mut self,
        ex_id: CGuid,
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyEquipmentMutationReport {
        let player_id = self.player_id();
        let position = self
            .battle_fairy_container
            .base()
            .query_goods_position(ex_id);
        let cell = position.and_then(BattleFairyCell::from_position);
        let addons = position
            .and_then(|position| self.battle_fairy_container.base().get_goods(position))
            .map(|goods| BattleFairyGearAddons::read(goods, factory));
        let Some(outcome) = self.battle_fairy_container.base_mut().remove_goods(ex_id) else {
            return BattleFairyEquipmentMutationReport {
                player_id,
                cell,
                property_applied: false,
                outcome: BattleFairyEquipmentMutationOutcome::MissingGoods,
                effects: GameEffectJournal::default(),
            };
        };
        let mut report = BattleFairyEquipmentMutationReport {
            player_id,
            cell,
            property_applied: false,
            outcome: BattleFairyEquipmentMutationOutcome::Removed(outcome),
            effects: GameEffectJournal::default(),
        };
        if let (Some(cell), Some(addons)) = (cell, addons)
            && let Some(first_update) = self.apply_battle_fairy_property(
                cell,
                addons,
                -1,
                factory,
                coefficients,
                encode_old_client,
            )
        {
            report.property_applied = true;
            report
                .effects
                .push(BattleFairyEquipmentMutationEffect::PropertiesChanged { player_id });
            report
                .effects
                .push(BattleFairyEquipmentMutationEffect::BattleFairyUpdated(
                    first_update,
                ));
            if let Some(battle_fairy) = self.war_soul_goods(factory) {
                report
                    .effects
                    .push(BattleFairyEquipmentMutationEffect::BattleFairyUpdated(
                        BattleFairyDefaultGoodsUpdate {
                            message_type: 0x0b_f918,
                            player_id,
                            goods: battle_fairy.identity(),
                            old_client_payload: encode_old_client(battle_fairy),
                        },
                    ));
            }
        }
        report
    }

    /// Positional `Remove(position, amount)` использует полный player-tail
    /// для whole goods. Partial stack remove относится к material/gem cells и
    /// не запускает `BFPropertyAdd(-1)`, пока исходный slot остаётся занят.
    pub(crate) fn take_battle_fairy_goods<Create>(
        &mut self,
        cell: BattleFairyCell,
        amount: u32,
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        create_goods: Create,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyEquipmentMutationReport
    where
        Create: FnMut(u32) -> Option<CGoods>,
    {
        let player_id = self.player_id();
        let Some(goods) = self
            .battle_fairy_container
            .base()
            .get_goods(cell.position())
        else {
            return BattleFairyEquipmentMutationReport {
                player_id,
                cell: Some(cell),
                property_applied: false,
                outcome: BattleFairyEquipmentMutationOutcome::MissingGoods,
                effects: GameEffectJournal::default(),
            };
        };
        if goods.amount() == amount {
            return self.remove_battle_fairy_goods(
                goods.identity().ex_id,
                factory,
                coefficients,
                encode_old_client,
            );
        }
        let outcome = self.battle_fairy_container.base_mut().take_goods(
            cell.position(),
            amount,
            factory,
            create_goods,
        );
        BattleFairyEquipmentMutationReport {
            player_id,
            cell: Some(cell),
            property_applied: false,
            outcome: outcome.map_or(
                BattleFairyEquipmentMutationOutcome::MissingGoods,
                BattleFairyEquipmentMutationOutcome::Removed,
            ),
            effects: GameEffectJournal::default(),
        }
    }

    /// Полный player-side opcode `0x8FC2A`. `allocations` содержат пары
    /// property/client-points прямо из packet-а: legacy outer caller суммирует
    /// unscaled points, но передаёт каждому `AllocatePotential` wrapping
    /// `points * 10000`. `std::map::insert` сохраняет первую запись ключа.
    pub(crate) fn allocate_battle_fairy_potential(
        &mut self,
        battle_fairy_enabled: bool,
        allocations: &[(i32, i32)],
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyPotentialAllocationReport {
        let player_id = self.player_id();
        let aggregate_client_points = allocations
            .iter()
            .fold(0i32, |total, (_, points)| total.wrapping_add(*points));
        let mut report = BattleFairyPotentialAllocationReport {
            player_id,
            outcome: BattleFairyPotentialAllocationOutcome::MissingHeadgear,
            effects: GameEffectJournal::default(),
        };
        let Some(goods) = self.equipment.get_goods(10) else {
            return report;
        };
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            report.outcome = BattleFairyPotentialAllocationOutcome::InvalidHeadgear;
            report
                .effects
                .push(BattleFairyPotentialAllocationEffect::Notification {
                    player_id,
                    string_id: "ZHGS0009",
                    color: 0xffff_ffff,
                });
            return report;
        }
        if goods
            .addon_property_value(factory, GAP_BF_POTENTIAL, 1)
            .wrapping_sub(aggregate_client_points)
            < 0
        {
            report.outcome = BattleFairyPotentialAllocationOutcome::AggregateInsufficient;
            return report;
        }

        let mut ordered = BTreeMap::new();
        for &(property, points) in allocations {
            ordered.entry(property).or_insert(points);
        }
        for (property, points) in ordered {
            if !battle_fairy_enabled {
                report
                    .effects
                    .push(BattleFairyPotentialAllocationEffect::Notification {
                        player_id,
                        string_id: "ZHGS0008",
                        color: 0xffff_0000,
                    });
                continue;
            }
            let amount = points.wrapping_mul(10_000);
            self.allocate_one_battle_fairy_potential(property, amount, factory, coefficients);
            tracing::trace!(
                player_id,
                property,
                points,
                "свойство потенциала боевой феи обработано"
            );
            report
                .effects
                .push(BattleFairyPotentialAllocationEffect::PropertiesChanged { player_id });
            if let Some(goods) = self.war_soul_goods(factory) {
                report
                    .effects
                    .push(BattleFairyPotentialAllocationEffect::GoodsUpdated(
                        BattleFairyDefaultGoodsUpdate {
                            message_type: 0x0b_f918,
                            player_id,
                            goods: goods.identity(),
                            old_client_payload: encode_old_client(goods),
                        },
                    ));
            }
        }

        // Outer goods-message сериализует headgear ещё раз независимо от
        // feature-disabled/unknown-property результата внутренних вызовов.
        if let Some(goods) = self.war_soul_goods(factory) {
            report
                .effects
                .push(BattleFairyPotentialAllocationEffect::GoodsUpdated(
                    BattleFairyDefaultGoodsUpdate {
                        message_type: 0x0b_f918,
                        player_id,
                        goods: goods.identity(),
                        old_client_payload: encode_old_client(goods),
                    },
                ));
        }
        report.outcome = BattleFairyPotentialAllocationOutcome::Processed;
        report
    }

    fn allocate_one_battle_fairy_potential(
        &mut self,
        property: i32,
        amount: i32,
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
    ) {
        let occupation = usize::from(self.base_properties.occupation).min(2);
        let mut player_delta = None;
        {
            let Some(goods) = self.equipment.get_goods_mut(10) else {
                return;
            };
            let potential = goods.addon_property_value(factory, GAP_BF_POTENTIAL, 1);
            if potential.wrapping_sub(amount) < 0 {
                return;
            }
            let (tracked_property, applied_amount) = match property {
                GAP_BF_ATTACK => (
                    GAP_BF_ATTACK_POTENTIAL,
                    (f64::from(amount) * 1.5).trunc() as i32,
                ),
                GAP_BF_SPRITE => (
                    GAP_BF_SPRITE_POTENTIAL,
                    (f64::from(amount) * 1.5).trunc() as i32,
                ),
                GAP_BF_BLAST => (GAP_BF_BLAST_POTENTIAL, amount),
                GAP_BF_BRAVE => (GAP_BF_BRAVE_POTENTIAL, amount),
                GAP_BF_AGILITY => (GAP_BF_AGILITY_POTENTIAL, amount),
                GAP_BF_SPRITUALISM => (GAP_BF_SPRITUALISM_POTENTIAL, amount),
                GAP_BF_STRENGH => (GAP_BF_STRENGH_POTENTIAL, amount),
                _ => return,
            };
            add_battle_fairy_addon(goods, factory, property, applied_amount);
            add_battle_fairy_addon(goods, factory, tracked_property, applied_amount);
            let _stored = goods.set_addon_property_value_core(
                GAP_BF_POTENTIAL,
                1,
                potential.wrapping_sub(amount),
            );
            if property == GAP_BF_SPRITUALISM {
                add_battle_fairy_addon(goods, factory, GAP_BF_MAX_MP, amount);
            } else if property == GAP_BF_STRENGH {
                add_battle_fairy_addon(goods, factory, GAP_BF_MAX_HP, amount);
            }
            if matches!(
                property,
                GAP_BF_BRAVE | GAP_BF_AGILITY | GAP_BF_SPRITUALISM | GAP_BF_STRENGH
            ) {
                player_delta = Some((property, f64::from(amount) * 0.00001));
            }
        }

        let Some((property, delta)) = player_delta else {
            return;
        };
        let combat = &mut self.combat_properties;
        match property {
            GAP_BF_BRAVE => {
                combat.strength = add_battle_fairy_u32(combat.strength, delta);
                combat.maximum_attack = add_battle_fairy_u32(
                    combat.maximum_attack,
                    delta * f64::from(coefficients.str_to_max_attack[occupation]),
                );
                combat.burden = add_battle_fairy_u16(
                    combat.burden,
                    delta * f64::from(coefficients.str_to_burden[occupation]),
                );
            }
            GAP_BF_AGILITY => {
                combat.dexterity = add_battle_fairy_u32(combat.dexterity, delta);
                combat.minimum_attack = add_battle_fairy_u32(
                    combat.minimum_attack,
                    delta * f64::from(coefficients.dex_to_min_attack[occupation]),
                );
                combat.reank = add_battle_fairy_u16(
                    combat.reank,
                    delta * f64::from(coefficients.dex_to_stiff[occupation]),
                );
            }
            GAP_BF_SPRITUALISM => {
                combat.intelligence = add_battle_fairy_u32(combat.intelligence, delta);
                combat.element_modify = add_battle_fairy_i32(
                    combat.element_modify,
                    delta * f64::from(coefficients.int_to_element[occupation]),
                );
                combat.maximum_mp = add_battle_fairy_u32(
                    combat.maximum_mp,
                    delta * f64::from(coefficients.int_to_max_mp[occupation]),
                );
                combat.element_resistance = add_battle_fairy_u32(
                    combat.element_resistance,
                    delta * f64::from(coefficients.int_to_resistant[occupation]),
                );
            }
            GAP_BF_STRENGH => {
                combat.maximum_hp = add_battle_fairy_u32(combat.maximum_hp, delta);
            }
            _ => {}
        }
    }

    pub(crate) fn upgrade_battle_fairy_equipment(
        &mut self,
        factory: &CGoodsFactory,
        log_gates: BattleFairyUpgradeLogGates,
        random: &mut dyn FnMut(i32) -> i32,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyUpgradeReport {
        let player_id = self.player_id();
        let price = self.battle_fairy_container.upgrade_price(factory);
        let mut report = BattleFairyUpgradeReport {
            player_id,
            outcome: BattleFairyUpgradeOutcome::MissingRegion,
            effects: GameEffectJournal::default(),
        };
        if self.server_region_id.is_none() {
            return report;
        }
        if self.wallet.currency_amount() < price {
            report.outcome = BattleFairyUpgradeOutcome::InsufficientMoney;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0015", Some(price));
            return report;
        }
        let Some(equipment) = self
            .battle_fairy_container
            .base()
            .get_goods(BattleFairyCell::Equipment.position())
        else {
            report.outcome = BattleFairyUpgradeOutcome::InvalidEquipment;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0014", None);
            return report;
        };
        if !equipment.can_battle_fairy_equipment_upgrade(factory) {
            report.outcome = BattleFairyUpgradeOutcome::InvalidEquipment;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0014", None);
            return report;
        }
        let current_level = equipment.addon_property_value(factory, GAP_BF_WEAPON_LEVEL, 1);
        let target = BattleFairyUpgradeGoodsSnapshot::capture(equipment);
        let Some(base_gem) = self
            .battle_fairy_container
            .base()
            .get_goods(BattleFairyCell::GemBase.position())
        else {
            report.outcome = BattleFairyUpgradeOutcome::MissingBaseGem;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0013", None);
            return report;
        };
        let minimum = base_gem.addon_property_value(factory, GAP_GEM_LEVEL, 1);
        let maximum = base_gem
            .addon_property_value(factory, GAP_GEM_LEVEL, 2)
            .max(minimum);
        if current_level < minimum || maximum < current_level {
            report.outcome = BattleFairyUpgradeOutcome::GemLevelMismatch;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0012", None);
            return report;
        }
        if 98 < current_level as u32 {
            report.outcome = BattleFairyUpgradeOutcome::MaximumLevel;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0021", None);
            return report;
        }
        let probability = self.battle_fairy_container.probability(factory);
        tracing::trace!(
            player_id,
            price,
            probability,
            current_level,
            "параметры улучшения боевой феи рассчитаны"
        );
        if self.wallet.currency_amount() < price {
            report.outcome = BattleFairyUpgradeOutcome::InsufficientMoney;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0020", None);
            return report;
        }
        let gems = [
            BattleFairyCell::GemBase,
            BattleFairyCell::GemOne,
            BattleFairyCell::GemTwo,
            BattleFairyCell::GemThree,
        ]
        .map(|cell| {
            self.battle_fairy_container
                .base()
                .get_goods(cell.position())
                .map(BattleFairyUpgradeGoodsSnapshot::capture)
        });
        let money = self.decrease_money(price, factory);
        report.effects.push(BattleFairyUpgradeEffect::MoneyChanged {
            player_id,
            previous: money.previous,
            current: money.current,
            outcome: money.outcome,
        });

        let audit_player = BattleFairyUpgradePlayerSnapshot {
            pk_count: self.base_properties.pk_count,
            money: self.money,
            depot_money: self.depot_money(),
            region_id: self.server_region_id.unwrap_or_default(),
            tile_x: self.shape().get_tile_x().unwrap_or_default(),
            tile_y: self.shape().get_tile_y().unwrap_or_default(),
            client_ip: self.client_ip,
        };

        let success = (random(100) as u32).wrapping_add(1) <= probability;
        let mut target_present = true;
        if success {
            let increase = self.battle_fairy_container.success_result(factory, random);
            let target_level = (current_level as u32).wrapping_add(increase).min(99) as i32;
            if let Some(goods) = self
                .battle_fairy_container
                .base_mut()
                .get_goods_mut(BattleFairyCell::Equipment.position())
            {
                let _upgraded = factory.upgrade_battle_fairy_equipment(goods, target_level);
            }
            report.outcome = BattleFairyUpgradeOutcome::Succeeded;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0002", None);
            if log_gates.success {
                report.effects.push(BattleFairyUpgradeEffect::Audit {
                    message_type: 0x0006_0203,
                    event: 1,
                    player_id,
                    player: audit_player,
                    target: target.clone(),
                    gems: gems.clone(),
                });
            }
        } else {
            if log_gates.failure {
                report.effects.push(BattleFairyUpgradeEffect::Audit {
                    message_type: 0x0006_0203,
                    event: 2,
                    player_id,
                    player: audit_player,
                    target: target.clone(),
                    gems: gems.clone(),
                });
            }
            match self.battle_fairy_container.fail_result(factory) {
                1 => {
                    report.outcome = BattleFairyUpgradeOutcome::FailedKept;
                    push_battle_fairy_upgrade_notification(&mut report, "ZHGS0016", None);
                }
                2 => {
                    report.outcome = BattleFairyUpgradeOutcome::FailedDowngraded;
                    push_battle_fairy_upgrade_notification(&mut report, "ZHGS0017", None);
                    if current_level != 0
                        && let Some(goods) = self
                            .battle_fairy_container
                            .base_mut()
                            .get_goods_mut(BattleFairyCell::Equipment.position())
                    {
                        let _upgraded = factory
                            .upgrade_battle_fairy_equipment(goods, current_level.wrapping_sub(1));
                    }
                }
                3 => {
                    report.outcome = BattleFairyUpgradeOutcome::FailedReset;
                    push_battle_fairy_upgrade_notification(&mut report, "ZHGS0018", None);
                    if let Some(goods) = self
                        .battle_fairy_container
                        .base_mut()
                        .get_goods_mut(BattleFairyCell::Equipment.position())
                    {
                        let _upgraded = factory.upgrade_battle_fairy_equipment(goods, 0);
                    }
                }
                4 => {
                    report.outcome = BattleFairyUpgradeOutcome::FailedDestroyed;
                    push_battle_fairy_upgrade_notification(&mut report, "ZHGS0019", None);
                    if log_gates.lost_target {
                        report.effects.push(BattleFairyUpgradeEffect::Audit {
                            message_type: 0x0006_0202,
                            event: 5,
                            player_id,
                            player: audit_player,
                            target: target.clone(),
                            gems: gems.clone(),
                        });
                    }
                    if let Some((_goods, removal)) =
                        self.battle_fairy_container.delete_upgrade_target()
                    {
                        target_present = false;
                        report
                            .effects
                            .push(BattleFairyUpgradeEffect::TargetDeleted {
                                player_id,
                                goods: target.clone(),
                                position: BattleFairyCell::Equipment.position(),
                                removal,
                            });
                    }
                }
                _ => {
                    report.outcome = BattleFairyUpgradeOutcome::FailedKept;
                }
            }
        }
        if target_present
            && let Some(goods) = self
                .battle_fairy_container
                .base()
                .get_goods(BattleFairyCell::Equipment.position())
        {
            report.effects.push(BattleFairyUpgradeEffect::GoodsUpdated(
                BattleFairyDefaultGoodsUpdate {
                    message_type: 0x0b_f918,
                    player_id,
                    goods: goods.identity(),
                    old_client_payload: encode_old_client(goods),
                },
            ));
        }

        for cell in [
            BattleFairyCell::GemBase,
            BattleFairyCell::GemOne,
            BattleFairyCell::GemTwo,
            BattleFairyCell::GemThree,
        ] {
            let was_present = self
                .battle_fairy_container
                .base()
                .get_goods(cell.position())
                .is_some();
            let Some(consumed) = self.battle_fairy_container.consume_upgrade_gem(cell) else {
                if was_present || cell == BattleFairyCell::GemBase {
                    report.outcome = BattleFairyUpgradeOutcome::ConsumptionStopped;
                    break;
                }
                continue;
            };
            tracing::trace!(player_id, cell = ?cell, removed = consumed.removed, previous_amount = consumed.previous_amount, remaining_amount = consumed.remaining_amount, "камень улучшения боевой феи израсходован");
            report.effects.push(BattleFairyUpgradeEffect::GemConsumed {
                player_id,
                consumed: consumed.clone(),
            });
            if !consumed.removed
                && let Some(goods) = self
                    .battle_fairy_container
                    .base()
                    .get_goods(cell.position())
            {
                report.effects.push(BattleFairyUpgradeEffect::GoodsUpdated(
                    BattleFairyDefaultGoodsUpdate {
                        message_type: 0x0b_f918,
                        player_id,
                        goods: goods.identity(),
                        old_client_payload: encode_old_client(goods),
                    },
                ));
            }
        }
        report
    }

    pub(crate) fn reset_battle_fairy_potential(
        &mut self,
        battle_fairy_enabled: bool,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyPotentialResetReport {
        let player_id = self.player_id();
        let mut report = BattleFairyPotentialResetReport {
            player_id,
            outcome: BattleFairyPotentialResetOutcome::MissingHeadgear,
            effects: GameEffectJournal::default(),
        };
        if !battle_fairy_enabled {
            report.outcome = BattleFairyPotentialResetOutcome::FeatureDisabled;
            report
                .effects
                .push(BattleFairyPotentialResetEffect::Notification {
                    player_id,
                    string_id: "ZHGS0008",
                    color: 0xffff_0000,
                });
            return report;
        }
        let Some(headgear) = self.equipment.get_goods(10) else {
            return report;
        };
        if headgear.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            report.outcome = BattleFairyPotentialResetOutcome::InvalidHeadgear;
            report
                .effects
                .push(BattleFairyPotentialResetEffect::Notification {
                    player_id,
                    string_id: "ZHGS0009",
                    color: 0xffff_ffff,
                });
            return report;
        }

        let reset_index = factory.query_goods_id_by_original_name(Some(b"ZHQLS01"));
        let reset_item = self
            .packet
            .base()
            .traversing_goods()
            .find(|goods| goods.base_properties_index() == reset_index)
            .map(|goods| (goods.identity(), goods.amount()));
        let Some((reset_identity, reset_amount)) = reset_item else {
            report.outcome = BattleFairyPotentialResetOutcome::MissingResetItem;
            report
                .effects
                .push(BattleFairyPotentialResetEffect::Notification {
                    player_id,
                    string_id: "ZHGS0010",
                    color: 0xffff_ffff,
                });
            return report;
        };
        let reset_position = self.packet.query_goods_position(reset_identity.ex_id);
        let (remaining_amount, consumed, removal) = if reset_amount == 0 {
            (0, false, None)
        } else if reset_amount == 1 {
            let removal = self.packet.remove_goods(reset_identity.ex_id);
            (
                if removal.is_some() { 0 } else { reset_amount },
                removal.is_some(),
                removal,
            )
        } else {
            let remaining = reset_amount.wrapping_sub(1);
            let mut consumed = false;
            if let Some(position) = reset_position
                && let Some(goods) = self.packet.get_goods_mut(position)
            {
                goods.set_amount(remaining);
                consumed = true;
            }
            (
                if consumed { remaining } else { reset_amount },
                consumed,
                None,
            )
        };
        report
            .effects
            .push(BattleFairyPotentialResetEffect::PacketItemConsumed {
                player_id,
                goods: reset_identity,
                position: reset_position,
                previous_amount: reset_amount,
                remaining_amount,
                consumed,
                removal,
            });

        let recovered = {
            let goods = self
                .equipment
                .get_goods_mut(10)
                .expect("headgear проверен до packet consumption");
            let mut take = |tracked, property| {
                let value = goods.addon_property_value(factory, tracked, 1);
                let _tracked_stored = goods.set_addon_property_value_core(tracked, 1, 0);
                let current = goods.addon_property_value(factory, property, 1);
                let _property_stored =
                    goods.set_addon_property_value_core(property, 1, current.wrapping_sub(value));
                value
            };
            let attack = take(GAP_BF_ATTACK_POTENTIAL, GAP_BF_ATTACK);
            let sprite = take(GAP_BF_SPRITE_POTENTIAL, GAP_BF_SPRITE);
            let blast = take(GAP_BF_BLAST_POTENTIAL, GAP_BF_BLAST);
            let brave = take(GAP_BF_BRAVE_POTENTIAL, GAP_BF_BRAVE);
            let agility = take(GAP_BF_AGILITY_POTENTIAL, GAP_BF_AGILITY);
            let spiritualism = take(GAP_BF_SPRITUALISM_POTENTIAL, GAP_BF_SPRITUALISM);
            let strength = take(GAP_BF_STRENGH_POTENTIAL, GAP_BF_STRENGH);
            let recovered = ((f64::from(sprite) + f64::from(attack)) * (2.0 / 3.0)
                + f64::from(blast)
                + f64::from(brave)
                + f64::from(agility)
                + f64::from(spiritualism)
                + f64::from(strength))
            .trunc() as i32;
            let potential = goods.addon_property_value(factory, GAP_BF_POTENTIAL, 1);
            let _stored = goods.set_addon_property_value_core(
                GAP_BF_POTENTIAL,
                1,
                potential.wrapping_add(recovered),
            );
            (recovered, brave, agility, spiritualism, strength)
        };
        tracing::trace!(
            player_id,
            recovered_potential = recovered.0,
            "потенциал боевой феи восстановлен"
        );
        self.set_strength(
            self.combat_properties
                .strength
                .wrapping_sub((f64::from(recovered.1) * 0.00001).trunc() as u32),
        );
        self.set_dexterity(
            self.combat_properties
                .dexterity
                .wrapping_sub((f64::from(recovered.2) * 0.00001).trunc() as u32),
        );
        self.set_maximum_hp(
            self.combat_properties
                .maximum_hp
                .wrapping_sub((f64::from(recovered.4) * 0.00001).trunc() as u32),
        );
        self.set_intelligence(
            self.combat_properties
                .intelligence
                .wrapping_sub((f64::from(recovered.3) * 0.00001).trunc() as u32),
        );
        report
            .effects
            .push(BattleFairyPotentialResetEffect::PropertiesChanged { player_id });
        let headgear = self
            .equipment
            .get_goods(10)
            .expect("reset не отделяет equipped headgear");
        report
            .effects
            .push(BattleFairyPotentialResetEffect::GoodsUpdated(
                BattleFairyDefaultGoodsUpdate {
                    message_type: 0x0b_f918,
                    player_id,
                    goods: headgear.identity(),
                    old_client_payload: encode_old_client(headgear),
                },
            ));
        report.outcome = BattleFairyPotentialResetOutcome::Reset;
        report
    }

    /// Полный player-side `CBattleFairyContainer::ResetSkill`. `consume_item`
    /// соответствует третьему native аргументу: script allocation передаёт
    /// ноль, прямой gameplay caller может потребовать `ZHJNS01/02`.
    pub(crate) fn reset_battle_fairy_skill(
        &mut self,
        battle_fairy_enabled: bool,
        position: i32,
        consume_item: bool,
        factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
        random: &mut dyn FnMut(i32) -> i32,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairySkillResetReport {
        let player_id = self.player_id();
        let mut report = BattleFairySkillResetReport {
            player_id,
            position,
            outcome: BattleFairySkillResetOutcome::MissingHeadgear,
            effects: GameEffectJournal::default(),
        };
        if !battle_fairy_enabled {
            report.outcome = BattleFairySkillResetOutcome::FeatureDisabled;
            report
                .effects
                .push(BattleFairySkillResetEffect::Notification {
                    player_id,
                    string_id: "ZHGS0008",
                    color: 0xffff_0000,
                });
            return report;
        }
        let Some(headgear) = self.equipment.get_goods(10) else {
            return report;
        };
        if headgear.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            report.outcome = BattleFairySkillResetOutcome::InvalidHeadgear;
            return report;
        }

        if consume_item {
            let reset_name = match position {
                3..=5 => Some(b"ZHJNS01".as_slice()),
                6 => Some(b"ZHJNS02".as_slice()),
                _ => None,
            };
            if let Some(reset_name) = reset_name {
                let reset_index = factory.query_goods_id_by_original_name(Some(reset_name));
                let reset_item = self
                    .packet
                    .base()
                    .traversing_goods()
                    .find(|goods| goods.base_properties_index() == reset_index)
                    .map(|goods| (goods.identity(), goods.amount()));
                let Some((reset_identity, reset_amount)) = reset_item else {
                    report.outcome = BattleFairySkillResetOutcome::MissingResetItem;
                    report
                        .effects
                        .push(BattleFairySkillResetEffect::Notification {
                            player_id,
                            string_id: BATTLE_FAIRY_SKILL_RESET_ITEM_MISSING,
                            color: 0xffff_ffff,
                        });
                    return report;
                };
                let reset_position = self.packet.query_goods_position(reset_identity.ex_id);
                let (remaining_amount, consumed, removal) = if reset_amount == 0 {
                    (0, false, None)
                } else if reset_amount == 1 {
                    let removal = self.packet.remove_goods(reset_identity.ex_id);
                    (
                        if removal.is_some() { 0 } else { reset_amount },
                        removal.is_some(),
                        removal,
                    )
                } else {
                    let remaining = reset_amount.wrapping_sub(1);
                    let mut consumed = false;
                    if let Some(reset_position) = reset_position
                        && let Some(goods) = self.packet.get_goods_mut(reset_position)
                    {
                        goods.set_amount(remaining);
                        consumed = true;
                    }
                    (
                        if consumed { remaining } else { reset_amount },
                        consumed,
                        None,
                    )
                };
                report
                    .effects
                    .push(BattleFairySkillResetEffect::PacketItemConsumed {
                        player_id,
                        goods: reset_identity,
                        position: reset_position,
                        previous_amount: reset_amount,
                        remaining_amount,
                        consumed,
                        removal,
                    });
            }
        }

        let (current_skills, current_all_skill) = {
            let goods = self
                .equipment
                .get_goods(10)
                .expect("headgear остаётся equipped после reset-item consumption");
            (
                [
                    goods.addon_property_value(factory, GAP_BF_SKY_SKILL, 2) as u32,
                    goods.addon_property_value(factory, GAP_BF_EARTH_SKILL, 2) as u32,
                    goods.addon_property_value(factory, GAP_BF_MAN_SKILL, 2) as u32,
                ],
                goods.addon_property_value(factory, GAP_BF_ALL_SKILL, 2) as u32,
            )
        };
        let (property, previous_skill, replaced) = match position {
            3..=5 => {
                let replaced = (position - 3) as usize;
                (
                    GAP_BF_SKY_SKILL + replaced as i32,
                    current_skills[replaced],
                    Some(replaced),
                )
            }
            6 => (GAP_BF_ALL_SKILL, current_all_skill, None),
            _ => {
                report.outcome = BattleFairySkillResetOutcome::InvalidPosition;
                return report;
            }
        };
        tracing::trace!(
            player_id,
            position,
            previous_skill,
            "прежний навык боевой феи выбран для сброса"
        );

        // В каждом native switch-case полный detach расположен перед первым
        // random(), а не только перед addon mutation.
        let old_entries = self.war_soul_skill_entries(factory);
        for (skill_id, _) in old_entries {
            if skill_id == 0 {
                continue;
            }
            let _deleted = self.move_shape.delete_skill(skill_id, skill_factory);
            tracing::trace!(
                player_id,
                skill_id,
                "навык боевой феи отсоединён при сбросе"
            );
            // Native `DelWarSoulSkillInPlayer` вызывает TellClient после
            // DelSkill. Поэтому packet удаления существует лишь если skill
            // пережил отказ category lookup.
            if let Some(skill) = self.move_shape.skill(skill_id, skill_factory) {
                report
                    .effects
                    .push(BattleFairySkillResetEffect::SkillRemoved(
                        BattleFairySkillRemoved {
                            message_type: BATTLE_FAIRY_SKILL_REMOVED_MESSAGE_TYPE,
                            player_id,
                            skill_id,
                            skill_name: skill.name(skill_factory).map(<[u8]>::to_vec),
                        },
                    ));
            }
        }

        let selected_skill = match replaced {
            Some(replaced) => loop {
                let candidate = SKILL_POJIA.wrapping_add(random(13) as u32);
                if current_skills.contains(&candidate) {
                    continue;
                }
                let conflicts = unpaired_battle_fairy_skill(candidate).is_some_and(|paired| {
                    current_skills
                        .iter()
                        .enumerate()
                        .any(|(index, &skill)| index != replaced && skill == paired)
                });
                if !conflicts {
                    break candidate;
                }
            },
            None => loop {
                let candidate = SKILL_LEIMING.wrapping_add(random(3) as u32);
                if candidate != current_all_skill {
                    break candidate;
                }
            },
        };
        tracing::trace!(
            player_id,
            selected_skill,
            "новый навык боевой феи выбран при сбросе"
        );

        {
            let goods = self
                .equipment
                .get_goods_mut(10)
                .expect("skill detach не отделяет equipped headgear");
            let _level_cleared = goods.set_addon_property_value_core(property, 1, 0);
            let _skill_cleared = goods.set_addon_property_value_core(property, 2, 0);
            let _level_stored = goods.set_addon_property_value_core(property, 1, 1);
            let _skill_stored =
                goods.set_addon_property_value_core(property, 2, selected_skill as i32);
        }

        let new_entries = self.war_soul_skill_entries(factory);
        for (skill_id, level) in new_entries {
            let _added = self.move_shape.add_skill(skill_id, level, skill_factory);
            if let Some(skill) = self.move_shape.skill(skill_id, skill_factory) {
                tracing::trace!(
                    player_id,
                    skill_id,
                    "навык боевой феи присоединён после сброса"
                );
                report.effects.push(BattleFairySkillResetEffect::SkillAdded(
                    battle_fairy_skill_snapshot(player_id, skill, skill_factory),
                ));
            }
        }

        let Some(selected) = self.move_shape.skill(selected_skill, skill_factory) else {
            report.outcome = BattleFairySkillResetOutcome::SelectedSkillUnavailable;
            return report;
        };
        report
            .effects
            .push(BattleFairySkillResetEffect::SelectedSkillLearned(
                battle_fairy_skill_snapshot(player_id, selected, skill_factory),
            ));
        let headgear = self
            .equipment
            .get_goods(10)
            .expect("ResetSkill не отделяет equipped headgear");
        report
            .effects
            .push(BattleFairySkillResetEffect::GoodsUpdated(
                BattleFairyDefaultGoodsUpdate {
                    message_type: 0x0b_f918,
                    player_id,
                    goods: headgear.identity(),
                    old_client_payload: encode_old_client(headgear),
                },
            ));
        report.outcome = BattleFairySkillResetOutcome::Reset;
        report
    }

    /// Player-owned `DelWarSoulSkillInPlayer` перед запуском reset-script.
    /// Native `TellClient(false)` уже после `DelSkill` не находит удалённый
    /// skill, поэтому наблюдаемым результатом остаётся ordered detach state.
    pub(crate) fn detach_battle_fairy_script_skills(
        &mut self,
        factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
    ) -> Vec<u32> {
        let mut detached = Vec::new();
        for (skill_id, _) in self.war_soul_skill_entries(factory) {
            if skill_id == 0 {
                continue;
            }
            let _deleted = self.move_shape.delete_skill(skill_id, skill_factory);
            detached.push(skill_id);
        }
        detached
    }

    /// Player-owned `AddWarSoulSkillToPalyer` после reset-script: addon state
    /// перечитывается из того же equipped headgear, затем каждый достигнутый
    /// skill публикуется через обычный `TellClient(true)` snapshot.
    pub(crate) fn attach_battle_fairy_script_skills(
        &mut self,
        factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
    ) -> Vec<BattleFairySkillAdded> {
        let player_id = self.player_id();
        let mut attached = Vec::new();
        for (skill_id, level) in self.war_soul_skill_entries(factory) {
            if skill_id == 0 {
                continue;
            }
            let _added = self.move_shape.add_skill(skill_id, level, skill_factory);
            if let Some(skill) = self.move_shape.skill(skill_id, skill_factory) {
                attached.push(battle_fairy_skill_snapshot(player_id, skill, skill_factory));
            }
        }
        attached
    }

    fn war_soul_skill_entries(&self, factory: &CGoodsFactory) -> [(u32, i32); 9] {
        let Some(goods) = self.equipment.get_goods(10) else {
            return [(0, 0); 9];
        };
        war_soul_skill_entries_from_goods(goods, factory)
    }

    /// Полный player-side `skillmessage 0x90001` после успешного decoder-а.
    /// Contend notification не блокирует запрос; `ClearEmotion` всегда
    /// предшествует authorization и AI dispatch.
    pub(crate) fn request_player_skill(
        &mut self,
        socket_id: i32,
        request: PlayerSkillRequest,
        facts: PlayerSkillRequestFacts,
        skill_factory: &CSkillFactory,
    ) -> GameEffectJournal {
        self.request_player_skill_core(socket_id, request, facts, None, "GS0090", skill_factory)
    }

    /// Item-skill `0x90004` использует переданный client level, а успешная
    /// ветвь добавляет ID в native ordered item-skill vector перед AI effect.
    pub(crate) fn request_item_skill(
        &mut self,
        socket_id: i32,
        request: PlayerSkillRequest,
        skill_level: i32,
        facts: PlayerSkillRequestFacts,
        skill_factory: &CSkillFactory,
    ) -> GameEffectJournal {
        self.request_player_skill_core(
            socket_id,
            request,
            facts,
            Some(skill_level),
            "GS1039",
            skill_factory,
        )
    }

    fn request_player_skill_core(
        &mut self,
        socket_id: i32,
        request: PlayerSkillRequest,
        facts: PlayerSkillRequestFacts,
        item_skill_level: Option<i32>,
        contend_string_id: &'static str,
        skill_factory: &CSkillFactory,
    ) -> GameEffectJournal {
        let player_id = self.player_id();
        let skill_id = request.skill_id();
        let mut journal = GameEffectJournal::default();
        let mut target_type = request.target_type;
        let mut target_id = request.target_id;
        let mut target_x = request.target_x;
        let mut target_y = request.target_y;
        if self.contend_state && facts.symbol_attackable {
            journal.push(GameEffect::SkillNotification {
                player_id,
                string_id: contend_string_id,
                color: 0xffff_ffff,
                message_type: 0xffff_0000,
            });
        }

        self.emotion_index = 0;
        self.emotion_timestamp_ms = 0;
        journal.push(GameEffect::ClearPlayerEmotion {
            player_id,
            region_id: self.server_region_id,
        });

        let skill_level = item_skill_level.unwrap_or_else(|| {
            self.move_shape
                .skill(skill_id, skill_factory)
                .map_or(0, MoveShapeSkill::level)
        });
        if skill_level == 0 {
            push_player_skill_reject(&mut journal, socket_id);
            trace!(
                player_id,
                skill_id, "Запрос навыка отклонён: навык не разрешён"
            );
            return journal;
        }
        if skill_factory
            .query_skill_base_properties(skill_id, skill_level)
            .is_some_and(|properties| properties.is_target_self() != 0)
        {
            let (resolved_x, resolved_y) =
                match (self.shape().get_tile_x(), self.shape().get_tile_y()) {
                    (Ok(x), Ok(y)) => (x, y),
                    (Err(error), _) | (_, Err(error)) => {
                        trace!(
                            player_id,
                            skill_id,
                            ?error,
                            "Запрос навыка отклонён координатной границей"
                        );
                        return journal;
                    }
                };
            target_type = self.shape().identity().object_type;
            target_id = player_id;
            target_x = resolved_x;
            target_y = resolved_y;
        }
        if !facts.player_ai_available {
            trace!(
                player_id,
                skill_id, "Запрос навыка отклонён: отсутствует AI игрока"
            );
            return journal;
        }

        let dispatch = if target_type == 0 || target_id == 0 {
            if target_x == 0 || target_y == 0 {
                PlayerSkillDispatch::SelfTarget {
                    skill_id,
                    player_id,
                }
            } else {
                PlayerSkillDispatch::Point {
                    skill_id,
                    x: target_x,
                    y: target_y,
                }
            }
        } else {
            if self.server_region_id.is_none() {
                trace!(
                    player_id,
                    skill_id, "Запрос навыка отклонён: отсутствует регион"
                );
                return journal;
            }
            let target = ShapeIdentity {
                object_type: target_type,
                id: target_id,
                ex_id: CGuid::GUID_INVALID,
            };
            if !facts.object_target_available {
                push_player_skill_reject(&mut journal, socket_id);
                trace!(
                    player_id,
                    skill_id, target_type, target_id, "Запрос навыка отклонён: цель отсутствует"
                );
                return journal;
            }
            PlayerSkillDispatch::Object { skill_id, target }
        };
        if item_skill_level.is_some() {
            self.move_shape.set_item_skill(skill_id);
        }
        journal.push(GameEffect::QueuePlayerSkill {
            player_id,
            dispatch,
        });
        trace!(
            player_id,
            skill_id,
            skill_level,
            ?dispatch,
            "Запрос навыка передан AI"
        );
        journal
    }

    /// Полный player-side `skillmessage` opcode `0x90005` после успешного
    /// packet decode. Contend notification намеренно не блокирует запрос.
    pub(crate) fn request_battle_fairy_skill(
        &self,
        battle_fairy_enabled: bool,
        socket_id: i32,
        request: BattleFairySkillRequest,
        facts: BattleFairySkillRequestFacts,
        goods_factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
    ) -> GameEffectJournal {
        let player_id = self.player_id();
        let skill_id = request.skill_id();
        let mut journal = GameEffectJournal::default();
        let mut target_type = request.target_type;
        let mut target_id = request.target_id;
        let mut target_x = request.target_x;
        let mut target_y = request.target_y;
        if !battle_fairy_enabled {
            journal.push(GameEffect::SkillNotification {
                player_id,
                string_id: "ZHGS0037",
                color: 0xffff_0000,
                message_type: 0,
            });
            trace!(
                player_id,
                skill_id, "Запрос навыка боевой феи отклонён: подсистема выключена"
            );
            return journal;
        }
        let Some(goods) = self.equipment.get_goods(10) else {
            trace!(
                player_id,
                skill_id, "Запрос навыка боевой феи отклонён: отсутствует головной предмет"
            );
            return journal;
        };
        if goods.addon_property_value(goods_factory, GAP_BF_HP, 1) == 0 {
            trace!(
                player_id,
                skill_id, "Запрос навыка боевой феи отклонён: нет здоровья"
            );
            return journal;
        }
        if self.contend_state && facts.symbol_attackable {
            journal.push(GameEffect::SkillNotification {
                player_id,
                string_id: "ZHGS0038",
                color: 0xffff_ffff,
                message_type: 0xffff_0000,
            });
        }

        let skill_level =
            check_battle_fairy_skill(goods, goods_factory, request.property_offset, skill_id);
        if skill_level == 0 {
            push_battle_fairy_skill_reject(&mut journal, socket_id);
            trace!(
                player_id,
                skill_id, "Запрос навыка боевой феи отклонён: навык не разрешён"
            );
            return journal;
        }

        if skill_factory
            .query_skill_base_properties(skill_id, skill_level)
            .is_some_and(|properties| properties.is_target_self() != 0)
        {
            let (resolved_x, resolved_y) =
                match (self.shape().get_tile_x(), self.shape().get_tile_y()) {
                    (Ok(x), Ok(y)) => (x, y),
                    (Err(error), _) | (_, Err(error)) => {
                        trace!(
                            player_id,
                            skill_id,
                            ?error,
                            "Запрос навыка боевой феи отклонён координатной границей"
                        );
                        return journal;
                    }
                };
            target_type = self.shape().identity().object_type;
            target_id = player_id;
            target_x = resolved_x;
            target_y = resolved_y;
        }
        if !facts.player_ai_available {
            trace!(
                player_id,
                skill_id, "Запрос навыка боевой феи отклонён: отсутствует AI игрока"
            );
            return journal;
        }

        let dispatch = if target_type == 0 || target_id == 0 {
            if target_x == 0 || target_y == 0 {
                BattleFairySkillDispatch::SelfTarget {
                    skill_id,
                    skill_level,
                    player_id,
                }
            } else {
                BattleFairySkillDispatch::Point {
                    skill_id,
                    skill_level,
                    x: target_x,
                    y: target_y,
                }
            }
        } else {
            if self.server_region_id.is_none() {
                trace!(
                    player_id,
                    skill_id, "Запрос навыка боевой феи отклонён: отсутствует регион"
                );
                return journal;
            }
            let target = ShapeIdentity {
                object_type: target_type,
                id: target_id,
                ex_id: CGuid::GUID_INVALID,
            };
            if !facts.object_target_available {
                push_battle_fairy_skill_reject(&mut journal, socket_id);
                trace!(
                    player_id,
                    skill_id,
                    target_type,
                    target_id,
                    "Запрос навыка боевой феи отклонён: цель отсутствует"
                );
                return journal;
            }
            BattleFairySkillDispatch::Object {
                skill_id,
                skill_level,
                target,
            }
        };
        journal.push(GameEffect::QueueBattleFairySkill {
            player_id,
            dispatch,
        });
        trace!(
            player_id,
            skill_id,
            skill_level,
            ?dispatch,
            "Запрос навыка боевой феи передан AI"
        );
        journal
    }

    /// Достигнутая часть exact `RefreshContainerOwners`: owner ID должен быть
    /// перепривязан после создания player identity или его восстановления.
    pub(crate) const fn refresh_reached_container_owners(&mut self, player_id: i32) {
        self.bank.base_mut().set_owner(PLAYER_TYPE, player_id);
        self.depot
            .base_mut()
            .base_mut()
            .base_mut()
            .set_owner(PLAYER_TYPE, player_id);
        self.hand.set_owner(PLAYER_TYPE, player_id);
        self.enhancement
            .base_mut()
            .base_mut()
            .set_owner(PLAYER_TYPE, player_id);
        self.packet.base_mut().set_owner(PLAYER_TYPE, player_id);
        self.wallet.set_owner(PLAYER_TYPE, player_id);
        self.yuan_bao.set_owner(PLAYER_TYPE, player_id);
        self.ji_fen.set_owner(PLAYER_TYPE, player_id);
        self.equipment.base_mut().set_owner(PLAYER_TYPE, player_id);
        self.auction_listing
            .base_mut()
            .set_owner(PLAYER_TYPE, player_id);
        self.auction_goods
            .base_mut()
            .set_owner(PLAYER_TYPE, player_id);
        self.auction_wallet.set_owner(PLAYER_TYPE, player_id);
        self.ci_qing.base_mut().set_owner(PLAYER_TYPE, player_id);
        self.ci_qing_compose
            .base_mut()
            .set_owner(PLAYER_TYPE, player_id);
        self.fairy_container
            .base_mut()
            .base_mut()
            .set_owner(PLAYER_TYPE, player_id);
        self.battle_fairy_container
            .base_mut()
            .base_mut()
            .base_mut()
            .set_owner(PLAYER_TYPE, player_id);
    }

    pub(crate) const fn set_pk_count(&mut self, value: u16) {
        self.base_properties.pk_count = value;
    }

    /// `OnUpdateMurdererSign` сбрасывает часы при нулевом PK и запускает их
    /// только при первом переходе к ненулевому значению.
    pub(crate) fn update_murderer_sign(&mut self, now_ms: impl FnOnce() -> u32) {
        if self.base_properties.pk_count == 0 {
            self.murderer_time_stamp_ms = 0;
        } else if self.murderer_time_stamp_ms == 0 {
            self.murderer_time_stamp_ms = now_ms();
        }
    }

    pub(crate) const fn set_occupation(&mut self, occupation: u8) {
        self.base_properties.occupation = occupation;
    }

    pub(crate) const fn set_ci_qing_open(&mut self, value: bool) {
        self.ci_qing_open = value;
    }

    pub(crate) const fn set_experience(&mut self, value: u32) {
        self.base_properties.experience = value;
    }

    pub(crate) const fn experience(&self) -> u32 {
        self.base_properties.experience
    }

    /// Exact `CPlayer::IncreaseContinuousKill`: первый hit после истёкшего
    /// окна сбрасывает счётчик в ноль, а milestone меняет persisted
    /// `wHitTopLog` до начисления его bonus experience и client publications.
    pub(crate) fn increase_continuous_kill(
        &mut self,
        now_ms: u32,
        hit_time_ms: u32,
        hit_levels: &[HitLevelEntry],
    ) -> PlayerContinuousKillUpdate {
        let mut new_top_log = None;
        let mut bonus_experience = 0;
        if now_ms.wrapping_sub(self.continuous_kill_timestamp_ms) < hit_time_ms {
            self.continuous_kill_amount = self.continuous_kill_amount.wrapping_add(1);
            if u32::from(self.base_properties.hit_top_log) < self.continuous_kill_amount {
                if let Some(level) = hit_levels
                    .iter()
                    .find(|level| level.hit == self.continuous_kill_amount)
                {
                    self.base_properties.hit_top_log = level.hit as u16;
                    new_top_log = Some(self.base_properties.hit_top_log);
                    bonus_experience = level.experience;
                }
            }
        } else {
            self.continuous_kill_amount = 0;
        }
        self.continuous_kill_timestamp_ms = now_ms;
        PlayerContinuousKillUpdate {
            amount: self.continuous_kill_amount,
            new_top_log,
            bonus_experience,
        }
    }

    pub(crate) const fn continuous_kill_amount(&self) -> u32 {
        self.continuous_kill_amount
    }

    pub(crate) const fn vigour(&self) -> u32 {
        self.base_properties.vigour
    }

    /// Exact `CPlayer::SetVigour`: вход сначала записывается в base property,
    /// затем ограничивается текущим `dwMaxVigour` того же owner-а.
    pub(crate) const fn set_vigour(&mut self, value: u32) {
        self.base_properties.vigour = if self.base_properties.maximum_vigour < value {
            self.base_properties.maximum_vigour
        } else {
            value
        };
    }

    pub(crate) const fn set_script_vigour(&mut self, value: i32) -> i32 {
        self.base_properties.vigour = value as u32;
        value
    }

    pub(crate) const fn set_script_experience(&mut self, value: i32) -> i32 {
        self.base_properties.experience = value as u32;
        value
    }

    pub(crate) const fn fairy_container_enabled(&self) -> bool {
        self.base_properties.fairy_container_enabled
    }

    /// Граница восстановления `m_BaseProperty.bFairyContainerEnabled` из
    /// persisted player snapshot; default остаётся выключенным до decode.
    pub(crate) const fn set_fairy_container_enabled(&mut self, value: bool) {
        self.base_properties.fairy_container_enabled = value;
    }

    pub(crate) const fn restore_appearance_and_mode(
        &mut self,
        head_picture: i32,
        face_picture: i32,
        mode: u32,
    ) {
        self.base_properties.head_picture = head_picture;
        self.base_properties.face_picture = face_picture;
        self.base_properties.mode = mode;
    }

    pub(crate) const fn appearance_and_mode(&self) -> (i32, i32, u32) {
        (
            self.base_properties.head_picture,
            self.base_properties.face_picture,
            self.base_properties.mode,
        )
    }

    pub(crate) const fn health(&self) -> u32 {
        self.base_properties.health
    }

    pub(crate) const fn maximum_health(&self) -> u32 {
        self.combat_properties.maximum_hp
    }

    pub(crate) fn roll_stiffen(
        &mut self,
        damage: u32,
        setup: crate::setup::globesetup::GlobeStiffenSetup,
        now_ms: impl FnMut() -> u32,
        random: impl FnMut(i32) -> i32,
    ) -> u32 {
        let maximum_hp = self.combat_properties.maximum_hp;
        let reank = self.combat_properties.reank;
        self.move_shape
            .stiffen(damage as u16, maximum_hp, reank, setup, now_ms, random)
    }

    pub(crate) const fn maximum_mana(&self) -> u32 {
        self.combat_properties.maximum_mp
    }

    /// Собственный скалярный хвост `OnRelive` после внешних вызовов
    /// пассивных навыков, входа в регион и пересчёта свойств.
    pub(crate) fn apply_relive_scalars(
        &mut self,
    ) -> Result<PlayerReliveMutation, ShapeCoordinateBlock> {
        let previous_x = self.shape().get_tile_x()?;
        let previous_y = self.shape().get_tile_y()?;
        let direction = self.shape().get_direction();
        self.set_health(self.maximum_health());
        self.set_mana(self.maximum_mana());
        self.move_shape.shape_mut().set_action(0);
        self.move_shape.shape_mut().set_position(0);
        Ok(PlayerReliveMutation {
            player_id: self.player_id(),
            previous_x,
            previous_y,
            direction,
            health: self.health(),
            mana: self.mana(),
        })
    }

    /// Собственные неполиморфные изменения достигнутого `OnRelive`: временные
    /// снимки спутников очищаются до `OnEnterRegion/UpdateProperty`, а один
    /// уровень блокировки движения снимается после них.
    pub(crate) fn clear_relive_uncreated_companions(&mut self) -> (usize, bool) {
        let cleared_uncreated_pets = self.uncreated_pets.len();
        self.uncreated_pets.clear();
        let cleared_uncreated_carriage = !self.uncreated_carriage.original_name.is_empty()
            || self.uncreated_carriage.health != 0;
        self.uncreated_carriage.original_name.clear();
        self.uncreated_carriage.health = 0;
        (cleared_uncreated_pets, cleared_uncreated_carriage)
    }

    pub(crate) fn unlock_movement_after_relive(
        &mut self,
        cleared_uncreated_pets: usize,
        cleared_uncreated_carriage: bool,
    ) -> PlayerReliveOwnedPrelude {
        let previous_moveable_count = self.move_shape.moveable_count();
        self.move_shape.set_moveable(true);
        PlayerReliveOwnedPrelude {
            cleared_uncreated_pets,
            cleared_uncreated_carriage,
            previous_moveable_count,
            resulting_moveable_count: self.move_shape.moveable_count(),
            moveable: self.move_shape.is_moveable(),
        }
    }

    /// Точное скалярное изменение `EnterResidentState`; рассылкой соседям
    /// владеет `CGame`, где доступны действующие регион и сессия.
    pub(crate) fn enter_resident_state(&mut self) -> u32 {
        let previous = self.criminal_state_timestamp_ms;
        self.criminal_state_timestamp_ms = 0;
        previous
    }

    pub(crate) const fn criminal_state_active(&self) -> bool {
        self.criminal_state_timestamp_ms != 0
    }

    /// Exact criminal tail `UpdateCurrentState`: clock уже sampled caller-ом
    /// только при active timestamp; timeout использует wrapping DWORD sum,
    /// threshold сравнивает promoted `wPkCount` строго через `<`.
    pub(crate) fn criminal_state_end_due(
        &self,
        checked_at_ms: u32,
        criminal_time_ms: u32,
        pk_count_per_kill: u32,
    ) -> Option<PlayerCriminalStateEnd> {
        let previous_timestamp_ms = self.criminal_state_timestamp_ms;
        if previous_timestamp_ms == 0 {
            return None;
        }
        let reason = if previous_timestamp_ms.wrapping_add(criminal_time_ms) <= checked_at_ms {
            PlayerCriminalStateEndReason::Timeout
        } else if pk_count_per_kill < u32::from(self.base_properties.pk_count) {
            PlayerCriminalStateEndReason::PkThresholdExceeded
        } else {
            return None;
        };
        Some(PlayerCriminalStateEnd {
            player_id: self.player_id(),
            previous_timestamp_ms,
            checked_at_ms,
            pk_count: self.base_properties.pk_count,
            reason,
        })
    }

    pub(crate) const fn mana(&self) -> u32 {
        self.base_properties.mana
    }

    /// Добавляет четыре состояния из актуальных свойств после общего
    /// End-обхода Particular и AutomaticRestore у вызывающего владельца.
    pub(crate) fn append_automatic_hp_mp_states(&mut self) {
        self.move_shape
            .append_automatic_hp_mp_states(self.combat_properties);
    }

    pub(crate) fn particular_state_goods_present(
        &self,
        additional_data: u32,
        factory: &CGoodsFactory,
    ) -> bool {
        let mut listener = GoodsParticularPropertyListener::new(GAP_EXCEPTION_STATE);
        for goods in self.packet.base().traversing_goods() {
            listener.visit(factory, goods);
        }
        if listener.goods_ids().iter().any(|goods_id| {
            self.packet.base().find(*goods_id).is_some_and(|goods| {
                goods.addon_property_value(factory, GAP_EXCEPTION_STATE, 1) as u32
                    == additional_data
            })
        }) {
            return true;
        }
        for (_, goods) in self.equipment.traversing_goods() {
            listener.visit(factory, goods);
        }
        listener.goods_ids().iter().any(|goods_id| {
            self.equipment.find(*goods_id).is_some_and(|goods| {
                goods.addon_property_value(factory, GAP_EXCEPTION_STATE, 1) as u32
                    == additional_data
            })
        })
    }

    /// OnEnterRegion отбрасывает packet-значения, но дополняет тот же GUID-listener
    /// экипировкой. Уникальность относится к значениям, не к живым состояниям.
    pub(crate) fn equipment_particular_state_values(
        &self,
        factory: &CGoodsFactory,
    ) -> Vec<u32> {
        let mut listener = GoodsParticularPropertyListener::new(GAP_EXCEPTION_STATE);
        for goods in self.packet.base().traversing_goods() {
            listener.visit(factory, goods);
        }
        let mut values = Vec::new();
        for goods_id in listener.goods_ids() {
            if let Some(goods) = self.packet.base().find(*goods_id) {
                let additional = goods.addon_property_value(factory, GAP_EXCEPTION_STATE, 1) as u32;
                if additional != 0 && !values.contains(&additional) {
                    values.push(additional);
                }
            }
        }
        values.clear();
        for (_, goods) in self.equipment.traversing_goods() {
            listener.visit(factory, goods);
        }
        for goods_id in listener.goods_ids() {
            if let Some(goods) = self.equipment.find(*goods_id) {
                let additional = goods.addon_property_value(factory, GAP_EXCEPTION_STATE, 1) as u32;
                if additional != 0 && !values.contains(&additional) {
                    values.push(additional);
                }
            }
        }
        values
    }

    pub(crate) fn automatic_restore_needs_clock(&self, key: crate::gameserver::appserver::moveshape::StateKey) -> bool {
        self.move_shape
            .automatic_restore_state(key)
            .is_some_and(|state| {
                state.should_check(
                    self.is_dead(),
                    self.shape().get_state(),
                    self.health(),
                    self.maximum_health(),
                    self.mana(),
                    self.maximum_mana(),
                )
            })
    }

    pub(crate) fn automatic_restore_due(&self, key: crate::gameserver::appserver::moveshape::StateKey, checked_at_ms: u32) -> bool {
        self.move_shape
            .automatic_restore_state(key)
            .is_some_and(|state| state.due(checked_at_ms))
    }

    /// Фиксирует второе чтение часов даже при нулевом объёме. `true` означает,
    /// что исходный виртуальный `OnChangeStates` обязан быть вызван немедленно.
    pub(crate) fn apply_automatic_restore(
        &mut self,
        key: crate::gameserver::appserver::moveshape::StateKey,
        recorded_at_ms: u32,
    ) -> bool {
        let properties = self.combat_properties;
        let health = self.health();
        let maximum_health = self.maximum_health();
        let mana = self.mana();
        let maximum_mana = self.maximum_mana();
        let mutation = self
            .move_shape
            .automatic_restore_state_mut(key)
            .and_then(|state| {
                state.apply(
                    recorded_at_ms,
                    properties,
                    health,
                    maximum_health,
                    mana,
                    maximum_mana,
                )
            });
        match mutation {
            Some(AutomaticRestoreMutation::Health(value)) => self.set_health(value),
            Some(AutomaticRestoreMutation::Mana(value)) => self.set_mana(value),
            None => return false,
        }
        true
    }

    pub(crate) const fn rp(&self) -> u16 {
        self.base_properties.rp
    }

    pub(crate) const fn maximum_rp(&self) -> u16 {
        self.base_properties.maximum_rp
    }

    pub(crate) const fn yp(&self) -> u16 {
        self.base_properties.yp
    }

    pub(crate) const fn set_maximum_hp(&mut self, value: u32) {
        self.combat_properties.maximum_hp = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_maximum_mp(&mut self, value: u32) {
        self.combat_properties.maximum_mp = clamp_combat_scalar(value);
    }

    /// Exact `SetHP` сначала записывает вход, затем перечитывает виртуальный
    /// `GetMaxHP`; в typed owner-е это текущее combat поле.
    pub(crate) const fn set_health(&mut self, value: u32) {
        self.base_properties.health = if self.combat_properties.maximum_hp < value {
            self.combat_properties.maximum_hp
        } else {
            value
        };
    }

    pub(crate) const fn set_mana(&mut self, value: u32) {
        self.base_properties.mana = if self.combat_properties.maximum_mp < value {
            self.combat_properties.maximum_mp
        } else {
            value
        };
    }

    pub(crate) const fn set_rp(&mut self, value: u16) {
        self.base_properties.rp = if self.base_properties.maximum_rp < value {
            self.base_properties.maximum_rp
        } else {
            value
        };
    }

    /// Exact `CPlayer::IncreaseRp`: только профессия 0 и достигший первого
    /// setup-порога игрок получают RP. За атаку прибавляется фиксированное
    /// значение, а защитная ветвь идёт по шести порогам доли снятого HP с
    /// конца массива. `true` означает исходный вызов `PropertiesChanged` даже
    /// при нулевой прибавке или уже достигнутом пределе.
    pub(crate) fn increase_rp(
        &mut self,
        attacking: bool,
        damage: u16,
        globe_setup: &GlobeSetupSnapshot,
    ) -> bool {
        if self.occupation() != 0 {
            return false;
        }
        let Some(policy) = globe_setup.player_rp_gain_policy(self.level()) else {
            return false;
        };
        let gain = if attacking {
            policy.attack_gain
        } else {
            // Native сначала материализует ushort damage как f32, а деление
            // выполняет в x87 перед FSTP dword. f64 сохраняет точный u32
            // знаменатель до финального округления к тому же f32-result.
            let ratio = (damage as f64 / self.maximum_health() as f64) as f32;
            let mut gain = 0;
            for index in (0..policy.damage_factors.len()).rev() {
                if !(ratio <= policy.damage_factors[index]) {
                    break;
                }
                gain = policy.damage_gains[index];
            }
            gain
        };
        let value = u32::from(self.rp())
            .wrapping_add(u32::from(gain))
            .min(u32::from(policy.maximum)) as u16;
        self.set_rp(value);
        true
    }

    /// Player-owned scalar части `OnExit` return-point tail. Восстановление
    /// смерти выполняется до virtual `GetReturnPoint`, а destination location
    /// записывается только после успешного выбора точки.
    pub(crate) fn prepare_exit_return(&mut self, died: bool) {
        if died {
            self.set_health(self.maximum_health());
            self.move_shape.shape_mut().set_action(0);
            self.move_shape.shape_mut().set_position(0);
        }
    }

    pub(crate) fn apply_exit_return_location(
        &mut self,
        region_id: i32,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
    ) {
        let shape = self.move_shape.shape_mut();
        shape.set_region_id(region_id);
        shape.set_direction(direction);
        shape.set_pos_xy_base(tile_x as f32 + 0.5, tile_y as f32 + 0.5);
    }

    pub(crate) const fn set_strength(&mut self, value: u32) {
        self.combat_properties.strength = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_dexterity(&mut self, value: u32) {
        self.combat_properties.dexterity = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_constitution(&mut self, value: u32) {
        self.combat_properties.constitution = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_intelligence(&mut self, value: u32) {
        self.combat_properties.intelligence = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_minimum_attack(&mut self, value: u32) {
        self.combat_properties.minimum_attack = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_maximum_attack(&mut self, value: u32) {
        self.combat_properties.maximum_attack = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_defense(&mut self, value: u32) {
        self.combat_properties.defense = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_element_resistance(&mut self, value: u32) {
        self.combat_properties.element_resistance = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_blast_defense_scale(&mut self, value: f32) {
        self.combat_properties.blast_defense_scale_bits =
            (if value < 0.01 { 0.01 } else { value }).to_bits();
    }

    pub(crate) const fn set_full_miss_scale(&mut self, value: f32) {
        self.combat_properties.full_miss_scale_bits =
            (if value < 0.01 { 0.01 } else { value }).to_bits();
    }

    pub(crate) const fn set_critical_rate(&mut self, value: f32) {
        self.combat_properties.critical_rate_bits =
            (if value < 1.0 { 1.0 } else { value }).to_bits();
    }

    pub(crate) const fn set_contribution(&mut self, value: i32) {
        self.contribution = if value < CONTRIBUTION_MINIMUM {
            CONTRIBUTION_MINIMUM
        } else if value > CONTRIBUTION_MAXIMUM {
            CONTRIBUTION_MAXIMUM
        } else {
            value
        };
    }

    /// `lMaxFetchPower` в exact сравнивался после unsigned cast, поэтому
    /// отрицательный setup limit становится большим unsigned пределом.
    pub(crate) const fn set_fetch_power(&mut self, value: u32, setup_maximum: i32) {
        let maximum = setup_maximum as u32;
        self.base_properties.fetch_power = if maximum < value { maximum } else { value };
    }

    pub(crate) const fn fetch_power(&self) -> u32 {
        self.base_properties.fetch_power
    }

    /// Player-owned mutation `ReviveBattleFairy`; client goods/state wire
    /// остаётся у вызывающего `CGame`, уже после изменения всех полей.
    pub(crate) fn revive_battle_fairy(&mut self, factory: &CGoodsFactory) -> bool {
        let Some(goods) = self.equipment.get_goods_mut(10) else {
            return false;
        };
        if goods.addon_property_value(factory, GAP_BF_HP, 1) > 0 {
            return false;
        }
        let maximum_hp = goods.addon_property_value(factory, GAP_BF_MAX_HP, 1);
        let maximum_mp = goods.addon_property_value(factory, GAP_BF_MAX_MP, 1);
        let _ = goods.set_addon_property_value_core(GAP_BF_HP, 1, maximum_hp);
        let _ = goods.set_addon_property_value_core(GAP_BF_MP, 1, maximum_mp);
        self.base_properties.battle_fairy_recall = true;
        self.base_properties.battle_fairy_died = false;
        self.battle_fairy_summoned = false;
        self.war_soul_state = 0;
        true
    }

    pub(crate) const fn set_battle_fairy_recall(&mut self, value: bool) {
        self.base_properties.battle_fairy_recall = value;
    }

    pub(crate) const fn set_battle_fairy_died(&mut self, value: bool) {
        self.base_properties.battle_fairy_died = value;
    }

    /// Scalar tail `ApplyDeathFinalWarSoulReset`. В отличие от гибели самой
    /// боевой феи смерть хозяина снимает summon/state, разрешает recall и
    /// очищает `bBFDied`.
    pub(crate) const fn reset_war_soul_after_player_death(&mut self) {
        self.battle_fairy_summoned = false;
        self.war_soul_state = 0;
        self.base_properties.battle_fairy_recall = true;
        self.base_properties.battle_fairy_died = false;
    }

    /// Exact `SetSilence`: начало хранится в минутах `timeGetTime`, а
    /// не абсолютным deadline в миллисекундах.
    pub(crate) const fn set_silence(&mut self, minutes: i32, now_milliseconds: u32) {
        if minutes > 0 {
            self.silence_minutes = minutes;
            self.silence_timestamp_minutes = now_milliseconds / 60_000;
        } else {
            self.silence_minutes = 0;
            self.silence_timestamp_minutes = 0;
        }
    }

    /// Exact `IsInSilence`: равенство deadline ещё считается silence; после
    /// первой просроченной проверки оба legacy поля обнуляются.
    pub(crate) const fn is_in_silence(&mut self, now_milliseconds: u32) -> bool {
        if self.silence_minutes == 0 {
            return false;
        }
        let deadline =
            (self.silence_timestamp_minutes as i32).wrapping_add(self.silence_minutes) as u32;
        if now_milliseconds / 60_000 <= deadline {
            return true;
        }
        self.silence_minutes = 0;
        self.silence_timestamp_minutes = 0;
        false
    }

    /// Точная мутация `CPlayer::OnExit`: legacy owner отдельно считывает
    /// `timeGetTime` перед deadline, перед уменьшением остатка и перед новой
    /// отметкой. Поэтому caller передаёт часы как callback, а не один snapshot.
    pub(crate) fn update_silence_on_exit(
        &mut self,
        mut now_milliseconds: impl FnMut() -> u32,
    ) -> PlayerExitSilenceUpdate {
        let previous_minutes = self.silence_minutes;
        let previous_timestamp_minutes = self.silence_timestamp_minutes;
        let mut sampled_minutes = [None; 3];
        if self.silence_minutes != 0 {
            let first = now_milliseconds() / 60_000;
            sampled_minutes[0] = Some(first);
            let deadline =
                (self.silence_timestamp_minutes as i32).wrapping_add(self.silence_minutes) as u32;
            if deadline < first {
                self.silence_timestamp_minutes = 0;
                self.silence_minutes = 0;
            } else {
                let second = now_milliseconds() / 60_000;
                sampled_minutes[1] = Some(second);
                self.silence_minutes = self.silence_minutes.wrapping_add(
                    (self.silence_timestamp_minutes as i32).wrapping_sub(second as i32),
                );
                let third = now_milliseconds() / 60_000;
                sampled_minutes[2] = Some(third);
                self.silence_timestamp_minutes = third;
            }
        }
        PlayerExitSilenceUpdate {
            previous_minutes,
            previous_timestamp_minutes,
            sampled_minutes,
            remaining_minutes: self.silence_minutes,
            timestamp_minutes: self.silence_timestamp_minutes,
        }
    }

    /// Player caller `CheckBattleFairyCombine` всегда передаёт собственный ID
    /// в исходный owner; global compose configuration остаётся явным входом.
    pub(crate) fn check_battle_fairy_combine(
        &self,
        factory: &CGoodsFactory,
        compose: &[BattleFairyCompose],
    ) -> BattleFairyCombineCheck {
        self.battle_fairy_container.check_battle_fairy_combine(
            Some(self.player_id()),
            factory,
            compose,
        )
    }

    /// Полный player-side `BatllteFairyCombine`: gate, validation, exact
    /// random/deplete/remove order, creation, skill-state и адресные effects.
    /// Battle cell не является gear slot, поэтому этот caller намеренно не
    /// запускает `BFPropertyAdd` и общий player property recalc. Transport
    /// получает уже ordered report, не подменяя неизвестные поля исторических
    /// packet-encoder-ов выдуманными нулями.
    pub(crate) fn combine_battle_fairy<Create>(
        &mut self,
        battle_fairy_enabled: bool,
        setup_maximum_fetch_power: i32,
        factory: &CGoodsFactory,
        compose: &[BattleFairyCompose],
        skill_factory: &CSkillFactory,
        random: &mut dyn FnMut(i32) -> i32,
        create_goods: &mut Create,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyCombineReport
    where
        Create: FnMut(u32, &mut dyn FnMut(i32) -> i32) -> Option<CGoods>,
    {
        let player_id = self.player_id();
        let mut report = BattleFairyCombineReport {
            player_id,
            outcome: BattleFairyCombineOutcome::Rejected,
            effects: GameEffectJournal::default(),
        };
        if !battle_fairy_enabled {
            report.outcome = BattleFairyCombineOutcome::FeatureDisabled;
            report.effects.push(BattleFairyCombineEffect::Notification {
                player_id,
                string_id: "ZHGS0008",
                color: 0xffff_0000,
            });
            return report;
        }

        let recipe = match self
            .battle_fairy_container
            .battle_fairy_combine_recipe(factory, compose)
        {
            Ok(recipe) => recipe,
            Err(notification) => {
                report.effects.push(BattleFairyCombineEffect::Notification {
                    player_id,
                    string_id: notification.string_id(),
                    color: 0xffff_ffff,
                });
                return report;
            }
        };
        let fetch_power = self.base_properties.fetch_power;
        if fetch_power < recipe.deplete_fetch {
            report.outcome = BattleFairyCombineOutcome::InsufficientFetchPower;
            report.effects.push(BattleFairyCombineEffect::Notification {
                player_id,
                string_id: "ZHGS0061",
                color: 0xffff_ffff,
            });
            return report;
        }

        let success = (random(100) as f32) < recipe.success_rate;
        if !success {
            report.effects.push(BattleFairyCombineEffect::Notification {
                player_id,
                string_id: "ZHGS0006",
                color: 0xffff_ffff,
            });
        }
        self.set_fetch_power(
            fetch_power.wrapping_sub(recipe.deplete_fetch),
            setup_maximum_fetch_power,
        );
        report
            .effects
            .push(BattleFairyCombineEffect::FetchPowerChanged {
                message_type: BATTLE_FAIRY_FETCH_POWER_MESSAGE_TYPE,
                player_id,
                subject_id: player_id,
                property_name: "dwFetchPower",
                value: self.base_properties.fetch_power,
            });

        for cell in [
            super::container::cbattlefairycontainer::BattleFairyCell::FetchBody,
            super::container::cbattlefairycontainer::BattleFairyCell::FetchStone,
            super::container::cbattlefairycontainer::BattleFairyCell::Material,
        ] {
            let Some(removed) = self
                .battle_fairy_container
                .remove_battle_fairy_combine_input(cell)
            else {
                report.outcome = BattleFairyCombineOutcome::InputRemovalStopped;
                return report;
            };
            report.effects.push(BattleFairyCombineEffect::ObjectMove(
                BattleFairyObjectMove {
                    operation: BattleFairyObjectMoveOperation::Delete,
                    player_id,
                    container_extend_id: BATTLE_FAIRY_CONTAINER_EXTEND_ID,
                    goods: removed.goods,
                    position: removed.cell.position(),
                    amount: removed.amount,
                    old_client_payload: None,
                },
            ));
            tracing::trace!(player_id, cell = ?removed.cell, goods = ?removed.goods, amount = removed.amount, "материал соединения боевой феи удалён");
        }

        if !success {
            report.outcome = BattleFairyCombineOutcome::Failed;
            report
                .effects
                .push(BattleFairyCombineEffect::Audit(BattleFairyAuditLog {
                    string_id: "ZHGS0007",
                    account: self.account.clone(),
                    goods_name: Vec::new(),
                }));
            return report;
        }

        let Some(created) = create_goods(recipe.index, random) else {
            report.outcome = BattleFairyCombineOutcome::CreationFailed;
            report
                .effects
                .push(BattleFairyCombineEffect::Audit(BattleFairyAuditLog {
                    string_id: "ZHGS0003",
                    account: self.account.clone(),
                    goods_name: Vec::new(),
                }));
            return report;
        };
        let created_identity = created.identity();
        let created_amount = created.amount();
        let mut incoming = Some(created);
        let stored = matches!(
            self.battle_fairy_container.add_at(
                super::container::cbattlefairycontainer::BattleFairyCell::Battle,
                &mut incoming,
                factory,
                true,
            ),
            BattleFairyContainerAddOutcome::Stored {
                base: VolumeGoodsAddOutcome::Added(_),
                ..
            }
        );
        if !stored {
            report.outcome = BattleFairyCombineOutcome::CreationRejected;
            return report;
        }

        let old_client_payload = {
            let goods = self
                .battle_fairy_container
                .base()
                .get_goods(
                    super::container::cbattlefairycontainer::BattleFairyCell::Battle.position(),
                )
                .expect("успешный add боевой феи сохранил goods в Battle cell");
            encode_old_client(goods)
        };
        report.effects.push(BattleFairyCombineEffect::ObjectMove(
            BattleFairyObjectMove {
                operation: BattleFairyObjectMoveOperation::New,
                player_id,
                container_extend_id: BATTLE_FAIRY_CONTAINER_EXTEND_ID,
                goods: created_identity,
                position: BattleFairyCell::Battle.position(),
                amount: created_amount,
                old_client_payload: Some(old_client_payload),
            },
        ));
        report.effects.push(BattleFairyCombineEffect::Notification {
            player_id,
            string_id: "ZHGS0004",
            color: 0xffff_ffff,
        });

        let mut skill_effects = Vec::with_capacity(3);
        let default_properties = {
            let (move_shape, container) = (&mut self.move_shape, &mut self.battle_fairy_container);
            let goods = container
                .base_mut()
                .get_goods_mut(
                    super::container::cbattlefairycontainer::BattleFairyCell::Battle.position(),
                )
                .expect("успешный add боевой феи оставляет Battle cell доступной");
            let mut register_skill = |skill: BattleFairyDefaultSkill| {
                let registered = move_shape.add_skill(skill.id, skill.level, skill_factory);
                if let Some(stored) = move_shape.skill(skill.id, skill_factory) {
                    skill_effects.push(BattleFairyCombineEffect::SkillAdded(
                        battle_fairy_skill_snapshot(player_id, stored, skill_factory),
                    ));
                }
                registered
            };
            CBattleFairyContainer::load_default_properties(
                Some(player_id),
                goods,
                factory,
                &mut register_skill,
                encode_old_client,
            )
            .expect("existing player ID разрешает LoadBFDefualtProperty")
        };
        report.effects.extend(skill_effects);
        report.effects.push(BattleFairyCombineEffect::GoodsUpdated(
            default_properties,
        ));
        let goods_name = self
            .battle_fairy_container
            .base()
            .get_goods(super::container::cbattlefairycontainer::BattleFairyCell::Battle.position())
            .expect("созданная боевая фея остаётся в Battle cell")
            .name()
            .to_vec();
        report
            .effects
            .push(BattleFairyCombineEffect::Audit(BattleFairyAuditLog {
                string_id: "ZHGS0005",
                account: self.account.clone(),
                goods_name,
            }));
        report.outcome = BattleFairyCombineOutcome::Created;
        report
    }

    pub(crate) fn shape_view(&self) -> Option<ShapeView> {
        let identity = self.shape().identity();
        Some(ShapeView {
            identity,
            tile_x: self.shape().get_tile_x().ok()?,
            tile_y: self.shape().get_tile_y().ok()?,
            pos_x_bits: self.shape().get_pos_x().to_bits(),
            pos_y_bits: self.shape().get_pos_y().to_bits(),
            figure: self.figure,
        })
    }
}

fn push_battle_fairy_summon_notification(
    report: &mut BattleFairySummonReport,
    string_id: &'static str,
    color: u32,
) {
    report.effects.push(BattleFairySummonEffect::Notification {
        player_id: report.player_id,
        string_id,
        color,
    });
}

fn push_battle_fairy_upgrade_notification(
    report: &mut BattleFairyUpgradeReport,
    string_id: &'static str,
    format_value: Option<u32>,
) {
    report.effects.push(BattleFairyUpgradeEffect::Notification {
        player_id: report.player_id,
        string_id,
        color: 0xffff_ffff,
        format_value,
    });
}

fn battle_fairy_skill_snapshot(
    player_id: i32,
    skill: &MoveShapeSkill,
    factory: &CSkillFactory,
) -> BattleFairySkillAdded {
    BattleFairySkillAdded {
        message_type: BATTLE_FAIRY_SKILL_ADDED_MESSAGE_TYPE,
        player_id,
        skill_id: skill.id(),
        skill_level: skill.level(),
        skill_type: skill.skill_type(),
        skill_name: skill.name(factory).map(<[u8]>::to_vec),
    }
}

fn war_soul_skill_entries_from_goods(goods: &CGoods, factory: &CGoodsFactory) -> [(u32, i32); 9] {
    let entry = |property| {
        (
            goods.addon_property_value(factory, property, 2) as u32,
            goods.addon_property_value(factory, property, 1),
        )
    };
    let mut entries = [
        entry(GAP_BF_SKY),
        entry(GAP_BF_EARTH),
        entry(GAP_BF_MAN),
        entry(GAP_BF_SKY_SKILL),
        entry(GAP_BF_EARTH_SKILL),
        entry(GAP_BF_MAN_SKILL),
        entry(GAP_BF_ALL_SKILL),
        entry(GAP_BF_HUOXIESHU_SKILL),
        entry(GAP_BF_LINGZHISHU_SKILL),
    ];
    entries[7].1 = 1;
    entries[8].1 = 1;
    entries
}

fn check_battle_fairy_skill(
    goods: &CGoods,
    factory: &CGoodsFactory,
    property_offset: i32,
    requested_skill: u32,
) -> i32 {
    if property_offset == 0 {
        return 1;
    }
    let property = GAP_BF_MAN.wrapping_add(property_offset);
    if goods.addon_property_value(factory, property, 2) as u32 == requested_skill {
        return goods.addon_property_value(factory, property, 1);
    }
    if goods.addon_property_value(factory, GAP_BF_HUOXIESHU_SKILL, 2) == 0x222
        || goods.addon_property_value(factory, GAP_BF_LINGZHISHU_SKILL, 2) == 0x223
    {
        return 1;
    }
    0
}

fn push_battle_fairy_skill_reject(journal: &mut GameEffectJournal, socket_id: i32) {
    journal.push(GameEffect::SkillSocketReject {
        socket_id,
        message_type: SKILL_EFFECT_MESSAGE_TYPE,
        reason: SKILL_REJECT_WAR_SOUL_REASON,
        code: SKILL_REJECT_CODE,
    });
}

fn push_player_skill_reject(journal: &mut GameEffectJournal, socket_id: i32) {
    journal.push(GameEffect::SkillSocketReject {
        socket_id,
        message_type: SKILL_EFFECT_MESSAGE_TYPE,
        reason: SKILL_REJECT_REASON,
        code: SKILL_REJECT_CODE,
    });
}

/// Exact constructor map `m_UnPairSkills`, подтверждённый immediate-ами
/// `gameserver.exe` по адресу `0x00504052..0x00504149`.
const fn unpaired_battle_fairy_skill(skill_id: u32) -> Option<u32> {
    Some(match skill_id {
        530 => 534,
        531 => 535,
        532 => 536,
        533 => 537,
        534 => 530,
        535 => 531,
        536 => 532,
        537 => 533,
        _ => return None,
    })
}

const fn is_battle_fairy_property_cell(cell: BattleFairyCell) -> bool {
    matches!(
        cell,
        BattleFairyCell::Weapon
            | BattleFairyCell::Body
            | BattleFairyCell::Huxinjing
            | BattleFairyCell::Jewelry
            | BattleFairyCell::Glove
            | BattleFairyCell::Pifeng
            | BattleFairyCell::Yaodai
            | BattleFairyCell::Xiezi
    )
}

fn add_battle_fairy_addon(
    goods: &mut CGoods,
    factory: &CGoodsFactory,
    property_type: i32,
    delta: i32,
) {
    let value = goods
        .addon_property_value(factory, property_type, 1)
        .wrapping_add(delta);
    let _stored = goods.set_addon_property_value_core(property_type, 1, value);
}

fn clamp_battle_fairy_current(
    goods: &mut CGoods,
    factory: &CGoodsFactory,
    current_property: i32,
    maximum_property: i32,
) {
    let maximum = goods.addon_property_value(factory, maximum_property, 1);
    if maximum < goods.addon_property_value(factory, current_property, 1) {
        let _stored = goods.set_addon_property_value_core(current_property, 1, maximum);
    }
}

fn add_battle_fairy_u32(current: u32, delta: f64) -> u32 {
    // `BFPropertyAdd/AllocatePotential` сначала FISTP-усекают delta, затем
    // выполняют целочисленное сложение/вычитание с текущим свойством.
    let next = i64::from(current) + delta.trunc() as i64;
    if next < 0 {
        0
    } else {
        clamp_combat_scalar(next as u32)
    }
}

fn add_battle_fairy_u16(current: u16, delta: f64) -> u16 {
    let next = i64::from(current) + delta.trunc() as i64;
    if next < 0 { 0 } else { next as u16 }
}

fn add_battle_fairy_i32(current: i32, delta: f64) -> i32 {
    let next = i64::from(current) + delta.trunc() as i64;
    if next < 0 { 0 } else { next as i32 }
}

const fn clamp_combat_scalar(value: u32) -> u32 {
    if LEGACY_COMBAT_MAXIMUM < value {
        LEGACY_COMBAT_MAXIMUM
    } else {
        value
    }
}

fn read_player_game_save_slice<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    field: &'static str,
    needed: usize,
) -> Result<&'a [u8], PlayerGameSaveCodecError> {
    let mut reader = player_save_reader(source, *cursor, field, needed)?;
    let bytes = reader
        .read_bytes(needed)
        .map_err(|block| player_save_read_error(field, block))?;
    *cursor = reader.position();
    Ok(bytes)
}

fn read_player_game_save_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], PlayerGameSaveCodecError> {
    Ok(read_player_game_save_slice(source, cursor, field, N)?
        .try_into()
        .expect("player wire slice имеет запрошенную длину"))
}

fn read_player_game_save_u8(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u8, PlayerGameSaveCodecError> {
    let mut reader = player_save_reader(source, *cursor, field, 1)?;
    let value = reader
        .read_u8()
        .map_err(|block| player_save_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_player_game_save_u16(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u16, PlayerGameSaveCodecError> {
    let mut reader = player_save_reader(source, *cursor, field, 2)?;
    let value = reader
        .read_u16()
        .map_err(|block| player_save_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_player_game_save_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, PlayerGameSaveCodecError> {
    let mut reader = player_save_reader(source, *cursor, field, 4)?;
    let value = reader
        .read_u32()
        .map_err(|block| player_save_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_player_game_save_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, PlayerGameSaveCodecError> {
    let mut reader = player_save_reader(source, *cursor, field, 4)?;
    let value = reader
        .read_i32()
        .map_err(|block| player_save_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_player_game_save_count(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<usize, PlayerGameSaveCodecError> {
    let count = read_player_game_save_i32(source, cursor, field)?;
    usize::try_from(count).map_err(|_| PlayerGameSaveCodecError::NegativeCount { field, count })
}

fn read_player_game_save_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
    maximum: usize,
) -> Result<Vec<u8>, PlayerGameSaveCodecError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let mut reader = player_save_reader(source, offset, field, 1)?;
    let bytes = reader
        .read_c_string(available)
        .map_err(|block| player_save_read_error(field, block))?;
    let length = bytes.len();
    if length >= maximum {
        return Err(PlayerGameSaveCodecError::StringTooLong {
            field,
            length,
            maximum,
        });
    }
    *cursor = reader.position();
    Ok(bytes.to_vec())
}

fn append_player_game_save_count(
    destination: &mut Vec<u8>,
    field: &'static str,
    length: usize,
) -> Result<(), PlayerGameSaveCodecError> {
    let count = i32::try_from(length)
        .map_err(|_| PlayerGameSaveCodecError::CollectionTooLarge { field, length })?;
    LegacyWriter::new(destination).write_i32(count);
    Ok(())
}

fn append_old_client_volume(
    destination: &mut Vec<u8>,
    container: &CVolumeLimitGoodsContainer,
    goods_factory: &CGoodsFactory,
) -> Option<()> {
    let goods: Vec<_> = (0..container.size())
        .filter_map(|position| container.get_goods(position))
        .collect();
    LegacyWriter::new(destination).write_i32(i32::try_from(goods.len()).ok()?);
    for goods in goods {
        goods
            .serialize_for_old_client(destination, goods_factory, true)
            .then_some(())?;
    }
    Some(())
}

fn append_player_game_save_string(
    destination: &mut Vec<u8>,
    field: &'static str,
    value: &[u8],
    maximum: usize,
) -> Result<(), PlayerGameSaveCodecError> {
    let length = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    if length >= maximum {
        return Err(PlayerGameSaveCodecError::StringTooLong {
            field,
            length,
            maximum,
        });
    }
    LegacyWriter::new(destination).write_c_string(&value[..length]);
    Ok(())
}

fn player_save_reader<'source>(
    source: &'source [u8],
    cursor: usize,
    field: &'static str,
    needed: usize,
) -> Result<LegacyReader<'source>, PlayerGameSaveCodecError> {
    LegacyReader::at(source, cursor).map_err(|block| PlayerGameSaveCodecError::UnexpectedEnd {
        field,
        offset: block.offset,
        needed,
        available: block.available,
    })
}

fn player_save_read_error(
    field: &'static str,
    block: super::legacycodec::LegacyReadBlock,
) -> PlayerGameSaveCodecError {
    PlayerGameSaveCodecError::UnexpectedEnd {
        field,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}

fn read_player_wire_u16(wire: &[u8], offset: usize) -> u16 {
    LegacyReader::at(wire, offset)
        .and_then(|mut reader| reader.read_u16())
        .expect("base/property wire offset проверен layout-константой")
}

fn read_player_wire_u32(wire: &[u8], offset: usize) -> u32 {
    LegacyReader::at(wire, offset)
        .and_then(|mut reader| reader.read_u32())
        .expect("base/property wire offset проверен layout-константой")
}

fn write_player_wire_u16(wire: &mut [u8], offset: usize, value: u16) {
    LegacyWriter::write_u16_at(wire, offset, value)
        .expect("base/property wire offset проверен layout-константой");
}

fn write_player_wire_u32(wire: &mut [u8], offset: usize, value: u32) {
    LegacyWriter::write_u32_at(wire, offset, value)
        .expect("base/property wire offset проверен layout-константой");
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp

// ============================================================================
// FUNCTION: CPlayer::GetAccount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:371
// RVA: 0x00002860
// ADDRESS: 00402860
// PROTOTYPE: char * __thiscall GetAccount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetPkCount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:304
// RVA: 0x0001E580
// ADDRESS: 0041e580
// PROTOTYPE: void __thiscall SetPkCount(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCiQingOpenFun
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:250
// RVA: 0x0002ACD0
// ADDRESS: 0042acd0
// PROTOTYPE: void __thiscall SetCiQingOpenFun(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:289
// RVA: 0x0002ACE0
// ADDRESS: 0042ace0
// PROTOTYPE: void __thiscall SetExp(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMaxHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:390
// RVA: 0x0002ACF0
// ADDRESS: 0042acf0
// PROTOTYPE: void __thiscall SetMaxHP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMaxMP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:392
// RVA: 0x0002AD10
// ADDRESS: 0042ad10
// PROTOTYPE: void __thiscall SetMaxMP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetStr
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:398
// RVA: 0x0002AD30
// ADDRESS: 0042ad30
// PROTOTYPE: void __thiscall SetStr(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetDex
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:400
// RVA: 0x0002AD50
// ADDRESS: 0042ad50
// PROTOTYPE: void __thiscall SetDex(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCon
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:402
// RVA: 0x0002AD70
// ADDRESS: 0042ad70
// PROTOTYPE: void __thiscall SetCon(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetInt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:404
// RVA: 0x0002AD90
// ADDRESS: 0042ad90
// PROTOTYPE: void __thiscall SetInt(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMinAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:406
// RVA: 0x0002ADB0
// ADDRESS: 0042adb0
// PROTOTYPE: void __thiscall SetMinAtk(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMaxAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:408
// RVA: 0x0002ADD0
// ADDRESS: 0042add0
// PROTOTYPE: void __thiscall SetMaxAtk(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetDef
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:416
// RVA: 0x0002ADF0
// ADDRESS: 0042adf0
// PROTOTYPE: void __thiscall SetDef(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetElementResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:422
// RVA: 0x0002AE10
// ADDRESS: 0042ae10
// PROTOTYPE: void __thiscall SetElementResistant(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetBlastDefendScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:460
// RVA: 0x0002AE30
// ADDRESS: 0042ae30
// PROTOTYPE: void __thiscall SetBlastDefendScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetFullMissScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:466
// RVA: 0x0002AE60
// ADDRESS: 0042ae60
// PROTOTYPE: void __thiscall SetFullMissScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCriticalRate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:471
// RVA: 0x0002AE90
// ADDRESS: 0042ae90
// PROTOTYPE: void __thiscall SetCriticalRate(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetContribute
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:546
// RVA: 0x0002AEC0
// ADDRESS: 0042aec0
// PROTOTYPE: void __thiscall SetContribute(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CPlayer::SetFetchPower
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:680
// RVA: 0x0002AF20
// ADDRESS: 0042af20
// PROTOTYPE: void __thiscall SetFetchPower(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// MATERIALIZED: personal-shop flag storage lives above. Non-zero assignment is
// reached only after CSessionFactory seller ownership validation in the message caller;
// `(0, 0)` remains the unconditional terminal reset. Exact
// `GetDefaultAttackSkillID` materialized by the player owner above and reached
// from both reciprocal and death `OnLoseTarget` paths.

// IMPLEMENTED: `CPlayer::OnDecreaseMurdererSign` входит в reached
// `CGame::AI -> PeriodicalUpdate` pass через `decrease_murderer_sign`.

// IMPLEMENTED: `CPlayer::OnUpdateMurdererSign` входит в `apply_confirmed_kill` выше.

// ============================================================================
// FUNCTION: CPlayer::IsBadman
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:3089
// RVA: 0x0002B160
// ADDRESS: 0042b160
// PROTOTYPE: bool __thiscall IsBadman(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsInArea
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5078
// RVA: 0x0002B190
// ADDRESS: 0042b190
// PROTOTYPE: bool __thiscall IsInArea(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsInRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5123
// RVA: 0x0002B230
// ADDRESS: 0042b230
// PROTOTYPE: bool __thiscall IsInRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CanMountEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8582
// RVA: 0x0002C310
// ADDRESS: 0042c310
// PROTOTYPE: long __thiscall CanMountEquip(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED, VERIFIED_PSEUDOCODE: `DecodeSkillsFromByteArray` сохраняет
// signed count и достигнут единым GameSave decoder-ом выше; покрытый RAW удалён.

// ============================================================================
// FUNCTION: CPlayer::OnChangeProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9195
// RVA: 0x0002C620
// ADDRESS: 0042c620
// PROTOTYPE: void __thiscall OnChangeProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED, VERIFIED_DISASSEMBLY: `SetSilence/IsInSilence`
// RVA `0x0002C8A0/0x0002C8F0` материализованы выше и достигнуты GM
// `0x7FC0B/0x7FC0E`; покрытый raw удалён.
// IMPLEMENTED, VERIFIED_DISASSEMBLY: `UpdateCurrentState` combat/criminal
// halves and оба caller-а принадлежат `CGame`; покрытый raw удалён.

// ============================================================================
// FUNCTION: CPlayer::EnterCriminalState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9336
// RVA: 0x0002C9A0
// ADDRESS: 0042c9a0
// PROTOTYPE: void __thiscall EnterCriminalState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED, VERIFIED_DISASSEMBLY: `EnterResidentState` scalar хранит
// `CPlayer`, exact around wire публикует `CGame`; покрытый raw удалён.

// ============================================================================
// FUNCTION: CPlayer::EnterCombatState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9373
// RVA: 0x0002CB50
// ADDRESS: 0042cb50
// PROTOTYPE: void __thiscall EnterCombatState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetGoodsById_FromPackage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9501
// RVA: 0x0002CC70
// ADDRESS: 0042cc70
// PROTOTYPE: CGoods * __thiscall GetGoodsById_FromPackage(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnBeginSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9513
// RVA: 0x0002CC90
// ADDRESS: 0042cc90
// PROTOTYPE: int __thiscall OnBeginSkill(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendNotifyMessageA
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9558
// RVA: 0x0002CCD0
// ADDRESS: 0042ccd0
// PROTOTYPE: void __thiscall SendNotifyMessageA(char * param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendSystemInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9570
// RVA: 0x0002CD70
// ADDRESS: 0042cd70
// PROTOTYPE: void __thiscall SendSystemInfo(char * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendOtherInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9581
// RVA: 0x0002CE00
// ADDRESS: 0042ce00
// PROTOTYPE: void __thiscall SendOtherInfo(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CanMove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9644
// RVA: 0x0002CE80
// ADDRESS: 0042ce80
// PROTOTYPE: int __thiscall CanMove(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10030
// RVA: 0x0002CF40
// ADDRESS: 0042cf40
// PROTOTYPE: ulong __thiscall GetMoney(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetYuanBao
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10035
// RVA: 0x0002CF50
// ADDRESS: 0042cf50
// PROTOTYPE: ulong __thiscall GetYuanBao(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetDepotMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10045
// RVA: 0x0002CF60
// ADDRESS: 0042cf60
// PROTOTYPE: ulong __thiscall GetDepotMoney(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10050
// RVA: 0x0002CF70
// ADDRESS: 0042cf70
// PROTOTYPE: int __thiscall SetMoney(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetYuanBao
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10094
// RVA: 0x0002D120
// ADDRESS: 0042d120
// PROTOTYPE: int __thiscall SetYuanBao(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CPacketListener::CPacketListener
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10224
// RVA: 0x0002D2D0
// ADDRESS: 0042d2d0
// PROTOTYPE: undefined __thiscall CPacketListener(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CPacketListener::~CPacketListener
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10229
// RVA: 0x0002D2E0
// ADDRESS: 0042d2e0
// PROTOTYPE: void __thiscall ~CPacketListener(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CPacketListener::OnTraversingContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10234
// RVA: 0x0002D2F0
// ADDRESS: 0042d2f0
// PROTOTYPE: int __thiscall OnTraversingContainer(CContainer * param_1, CBaseObject * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAtcInterval
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10285
// RVA: 0x0002D3F0
// ADDRESS: 0042d3f0
// PROTOTYPE: ushort __thiscall GetAtcInterval(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetStrikeOutTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10309
// RVA: 0x0002D430
// ADDRESS: 0042d430
// PROTOTYPE: ulong __thiscall GetStrikeOutTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RejectUseSkillRequest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10327
// RVA: 0x0002D450
// ADDRESS: 0042d450
// PROTOTYPE: void __thiscall RejectUseSkillRequest(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CPlayer::PerformEmotion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11059
// RVA: 0x0002D590
// ADDRESS: 0042d590
// PROTOTYPE: void __thiscall PerformEmotion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CContainerListener::OnObjectRemoved
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11134
// RVA: 0x0002D720
// ADDRESS: 0042d720
// PROTOTYPE: int __thiscall OnObjectRemoved(CContainer * param_1, CBaseObject * param_2, ulong param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsFactionMaster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11200
// RVA: 0x0002D930
// ADDRESS: 0042d930
// PROTOTYPE: bool __thiscall IsFactionMaster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsUnionMaster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11211
// RVA: 0x0002D950
// ADDRESS: 0042d950
// PROTOTYPE: bool __thiscall IsUnionMaster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetWeaponModifier
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11410
// RVA: 0x0002D980
// ADDRESS: 0042d980
// PROTOTYPE: float __thiscall GetWeaponModifier(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetWeaponDamageLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11439
// RVA: 0x0002D9F0
// ADDRESS: 0042d9f0
// PROTOTYPE: ulong __thiscall GetWeaponDamageLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetSessionID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12173
// RVA: 0x0002DCC0
// ADDRESS: 0042dcc0
// PROTOTYPE: char * __thiscall GetSessionID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetIpAddress
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12179
// RVA: 0x0002DCD0
// ADDRESS: 0042dcd0
// PROTOTYPE: char * __thiscall GetIpAddress(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteSkillItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12861
// RVA: 0x0002DD20
// ADDRESS: 0042dd20
// PROTOTYPE: int __thiscall DeleteSkillItem(ulong param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED, VERIFIED_PSEUDOCODE: парный `ReplacePlayerData/RestorePlayerData`
// достигнут defense caller-ами выше; покрытый RAW удалён.

// IMPLEMENTED, VERIFIED_PSEUDOCODE: `TellClientMove/TellClient` достигнуты
// через follow journal и общий skill-message builder; покрытый RAW удалён.

// ============================================================================
// FUNCTION: CPlayer::RejectUseSkillRequestWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13358
// RVA: 0x0002E720
// ADDRESS: 0042e720
// PROTOTYPE: void __thiscall RejectUseSkillRequestWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAuctionMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14782
// RVA: 0x0002EEF0
// ADDRESS: 0042eef0
// PROTOTYPE: ulong __thiscall GetAuctionMoney(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendCutLog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15115
// RVA: 0x0002F0A0
// ADDRESS: 0042f0a0
// PROTOTYPE: void __thiscall SendCutLog(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCurFlash
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17480
// RVA: 0x0002FB50
// ADDRESS: 0042fb50
// PROTOTYPE: void __thiscall SetCurFlash(CGoods * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneFlash
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17528
// RVA: 0x0002FC50
// ADDRESS: 0042fc50
// PROTOTYPE: void __thiscall DoneFlash(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:368
// RVA: 0x000300D0
// ADDRESS: 004300d0
// PROTOTYPE: void __thiscall SetMP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetRP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:370
// RVA: 0x000300F0
// ADDRESS: 004300f0
// PROTOTYPE: void __thiscall SetRP(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetVigour
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:518
// RVA: 0x00030120
// ADDRESS: 00430120
// PROTOTYPE: void __thiscall SetVigour(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::PeriodicalUpdate
// STATUS: PARTIALLY_IMPLEMENTED_DEATH_STATE_TAIL
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2348
// RVA: 0x00030140
// ADDRESS: 00430140
// PROTOTYPE: void __thiscall PeriodicalUpdate(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::WriteGoodsDelLog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12633
// RVA: 0x00030430
// ADDRESS: 00430430
// PROTOTYPE: void __thiscall WriteGoodsDelLog(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetAuctionMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14737
// RVA: 0x00030E60
// ADDRESS: 00430e60
// PROTOTYPE: bool __thiscall SetAuctionMoney(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsGM
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5044
// RVA: 0x00031430
// ADDRESS: 00431430
// PROTOTYPE: bool __thiscall IsGM(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetGMLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5052
// RVA: 0x00031480
// ADDRESS: 00431480
// PROTOTYPE: long __thiscall GetGMLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10594
// RVA: 0x000314E0
// ADDRESS: 004314e0
// PROTOTYPE: ulong __thiscall DeleteGoods(PLAYER_EXTEND_ID param_1, CGUID * param_2, ulong param_3, bool param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoodsbyGuid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12330
// RVA: 0x00031800
// ADDRESS: 00431800
// PROTOTYPE: int __thiscall DeleteGoodsbyGuid(CGUID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddTaoZhuangSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15294
// RVA: 0x00031DE0
// ADDRESS: 00431de0
// PROTOTYPE: void __thiscall AddTaoZhuangSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddTaoZhuangPre
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15351
// RVA: 0x00031E70
// ADDRESS: 00431e70
// PROTOTYPE: void __thiscall AddTaoZhuangPre(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddCiQingTaoZhuangPre
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15362
// RVA: 0x00031EE0
// ADDRESS: 00431ee0
// PROTOTYPE: void __thiscall AddCiQingTaoZhuangPre(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DropGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:272
// RVA: 0x00032630
// ADDRESS: 00432630
// PROTOTYPE: int __thiscall DropGoods(PLAYER_EXTEND_ID param_1, CGUID * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetNumSkills
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8968
// RVA: 0x00032AA0
// ADDRESS: 00432aa0
// PROTOTYPE: long __thiscall GetNumSkills(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddSkillsToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8998
// RVA: 0x00032B80
// ADDRESS: 00432b80
// PROTOTYPE: void __thiscall AddSkillsToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnChangeStates
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9234
// RVA: 0x00033080
// ADDRESS: 00433080
// PROTOTYPE: void __thiscall OnChangeStates(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsEnemyFactionMember
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9397
// RVA: 0x00033210
// ADDRESS: 00433210
// PROTOTYPE: long __thiscall IsEnemyFactionMember(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsCityWarEneymyFactionMemeber
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9407
// RVA: 0x00033240
// ADDRESS: 00433240
// PROTOTYPE: long __thiscall IsCityWarEneymyFactionMemeber(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoodsInPacket
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10857
// RVA: 0x00033270
// ADDRESS: 00433270
// PROTOTYPE: void __thiscall DeleteGoodsInPacket(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddQuestDataByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11226
// RVA: 0x00033310
// ADDRESS: 00433310
// PROTOTYPE: bool __thiscall AddQuestDataByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ReUseSkillItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12832
// RVA: 0x00033610
// ADDRESS: 00433610
// PROTOTYPE: int __thiscall ReUseSkillItem(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::JudgeZhaoMuStatus
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13478
// RVA: 0x000336B0
// ADDRESS: 004336b0
// PROTOTYPE: bool __thiscall JudgeZhaoMuStatus(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CleanPreAndSkillList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15265
// RVA: 0x00033700
// ADDRESS: 00433700
// PROTOTYPE: void __thiscall CleanPreAndSkillList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelTaoZhuangSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15273
// RVA: 0x000337A0
// ADDRESS: 004337a0
// PROTOTYPE: void __thiscall DelTaoZhuangSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddByteCiQing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15841
// RVA: 0x00033840
// ADDRESS: 00433840
// PROTOTYPE: void __thiscall AddByteCiQing(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddOrgSysToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1518
// RVA: 0x00033C30
// ADDRESS: 00433c30
// PROTOTYPE: bool __thiscall AddOrgSysToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsAttackAble
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// MATERIALIZED: player/player ветвь находится в `CGame::player_base_attackable`
// и level gate, player/monster — в `CGame::player_attackable_by_monster`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10335
// RVA: 0x00034300
// ADDRESS: 00434300
// PROTOTYPE: bool __thiscall IsAttackAble(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::do_coutribute
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11514
// RVA: 0x00034A00
// ADDRESS: 00434a00
// PROTOTYPE: void __thiscall do_coutribute(CServerRegion * param_1, CPlayer * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddExploitToMurdererInCountryWar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12038
// RVA: 0x00035DF0
// ADDRESS: 00435df0
// PROTOTYPE: void __thiscall AddExploitToMurdererInCountryWar(CServerRegion * param_1, CPlayer * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DrawAwards
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12205
// RVA: 0x00035F70
// ADDRESS: 00435f70
// PROTOTYPE: long __thiscall DrawAwards(long param_1, int param_2, ulong param_3, ulong param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ChangeBodyCheck
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12918
// RVA: 0x00036080
// ADDRESS: 00436080
// PROTOTYPE: int __thiscall ChangeBodyCheck(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeByteCiQing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15853
// RVA: 0x00038EA0
// ADDRESS: 00438ea0
// PROTOTYPE: void __thiscall DeByteCiQing(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CheckGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:357
// RVA: 0x0003A020
// ADDRESS: 0043a020
// PROTOTYPE: int __thiscall CheckGoods(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10566
// RVA: 0x0003A300
// ADDRESS: 0043a300
// PROTOTYPE: CGUID __thiscall DeleteGoods(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DropParticularGoodsWhenDead
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10712
// RVA: 0x0003A4D0
// ADDRESS: 0043a4d0
// PROTOTYPE: void __thiscall DropParticularGoodsWhenDead(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToTaoZhuangSkillList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15230
// RVA: 0x0003AD30
// ADDRESS: 0043ad30
// PROTOTYPE: void __thiscall AddItemToTaoZhuangSkillList(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToTaoZhuangPreList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15243
// RVA: 0x0003AD70
// ADDRESS: 0043ad70
// PROTOTYPE: void __thiscall AddItemToTaoZhuangPreList(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToCiQingTaoZhuangPreList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15254
// RVA: 0x0003ADB0
// ADDRESS: 0043adb0
// PROTOTYPE: void __thiscall AddItemToCiQingTaoZhuangPreList(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetCurrentTypeValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16080
// RVA: 0x0003ADF0
// ADDRESS: 0043adf0
// PROTOTYPE: void __thiscall GetCurrentTypeValue(map<unsigned_long,unsigned_long,std::less<unsigned_long>,std::allocator<std::pair<unsigned_long_const_,unsigned_long>_>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED, VERIFIED_PSEUDOCODE: `DecordOrgSysFromByteArray` достигнут
// единым GameSave decoder-ом выше; покрытый RAW удалён.

// ============================================================================
// FUNCTION: CPlayer::MountEquipRide
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:6516
// RVA: 0x0003C5E0
// ADDRESS: 0043c5e0
// PROTOTYPE: void __thiscall MountEquipRide(CGoods * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnExitRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9661
// RVA: 0x0003DB50
// ADDRESS: 0043db50
// PROTOTYPE: void __thiscall OnExitRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IncExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10910
// RVA: 0x0003DB60
// ADDRESS: 0043db60
// PROTOTYPE: ulong __thiscall IncExp(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddQuestDataByteArray_ForClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11240
// RVA: 0x0003E1C0
// ADDRESS: 0043e1c0
// PROTOTYPE: bool __thiscall AddQuestDataByteArray_ForClient(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToDelList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12365
// RVA: 0x0003E3F0
// ADDRESS: 0043e3f0
// PROTOTYPE: bool __thiscall AddItemToDelList(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelAllItemInDelList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12401
// RVA: 0x0003E4E0
// ADDRESS: 0043e4e0
// PROTOTYPE: bool __thiscall DelAllItemInDelList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneDelList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12427
// RVA: 0x0003E670
// ADDRESS: 0043e670
// PROTOTYPE: void __thiscall DoneDelList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ComputeTicket
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12459
// RVA: 0x0003E820
// ADDRESS: 0043e820
// PROTOTYPE: ulong __thiscall ComputeTicket(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CheckAddGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12497
// RVA: 0x0003E920
// ADDRESS: 0043e920
// PROTOTYPE: ulong __thiscall CheckAddGoods(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToMap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12518
// RVA: 0x0003E980
// ADDRESS: 0043e980
// PROTOTYPE: bool __thiscall AddItemToMap(ulong param_1, CGoods * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToGoodsAiTree
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12543
// RVA: 0x0003EA60
// ADDRESS: 0043ea60
// PROTOTYPE: bool __thiscall AddItemToGoodsAiTree(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelItemFromGoodsAiTree
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12567
// RVA: 0x0003EAC0
// ADDRESS: 0043eac0
// PROTOTYPE: bool __thiscall DelItemFromGoodsAiTree(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneGoodsAiTree
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12616
// RVA: 0x0003EBD0
// ADDRESS: 0043ebd0
// PROTOTYPE: void __thiscall DoneGoodsAiTree(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CPlayer::UpdateGoodsGS2C` связан с goods/factory/game owner-ами; покрытый
// raw-блок удалён.

// ============================================================================
// FUNCTION: CPlayer::SetLastUseSkillItemTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12826
// RVA: 0x0003ED30
// ADDRESS: 0043ed30
// PROTOTYPE: void __thiscall SetLastUseSkillItemTime(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OutputBinaryStream
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15063
// RVA: 0x0003F890
// ADDRESS: 0043f890
// PROTOTYPE: void __thiscall OutputBinaryStream(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendTaoZhuangSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15214
// RVA: 0x0003FA20
// ADDRESS: 0043fa20
// PROTOTYPE: void __thiscall SendTaoZhuangSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CleanTaoZhuangItemList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15636
// RVA: 0x0003FAF0
// ADDRESS: 0043faf0
// PROTOTYPE: void __thiscall CleanTaoZhuangItemList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToTaoZhuangItemList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15642
// RVA: 0x0003FB60
// ADDRESS: 0043fb60
// PROTOTYPE: void __thiscall AddItemToTaoZhuangItemList(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendCiQingGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16023
// RVA: 0x0003FE90
// ADDRESS: 0043fe90
// PROTOTYPE: void __thiscall SendCiQingGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::InitSkills
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:530
// RVA: 0x00040C30
// ADDRESS: 00440c30
// PROTOTYPE: void __thiscall InitSkills(void)
//
// Реализовано `initialize_intrinsic_skills` и SetCurrentSkill-префиксом
// CGame::complete_world_player_login. Decode +0x9c — UpdateProperty,
// а InitSkills вызывается OnLogMessage после login-script через +0x14c.

// ============================================================================
// FUNCTION: CPlayer::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:742
// RVA: 0x00040DC0
// ADDRESS: 00440dc0
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0044158b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1715
// RVA: 0x0004158B
// ADDRESS: 0044158b
// PROTOTYPE: undefined FUN_0044158b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnLost
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1780
// RVA: 0x000417A0
// ADDRESS: 004417a0
// PROTOTYPE: void __thiscall OnLost(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CPlayer::OnEquipmentWaste` связан с goods/factory/game owner-ами; покрытый
// raw-блок удалён.

// IMPLEMENTED: `CPlayer::OnArmorDamaged` связан с goods/factory/game owner-ами; покрытый
// raw-блок удалён.

// ============================================================================
// FUNCTION: CPlayer::OnBeenMurdered
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:3111
// RVA: 0x00042040
// ADDRESS: 00442040
// PROTOTYPE: void __thiscall OnBeenMurdered(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5239
// RVA: 0x00042610
// ADDRESS: 00442610
// PROTOTYPE: void __thiscall MountEquip(ulong param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00443c5b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:6069
// RVA: 0x00043C5B
// ADDRESS: 00443c5b
// PROTOTYPE: undefined Catch@00443c5b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// IMPLEMENTED: `CPlayer::OnObjectAdded` связан с packet/equipment add и
// particular-state owner-ом; покрытый raw-блок удалён.

// IMPLEMENTED: `CPlayer::DecordQuestDataFromByteArray` входит в полный
// GameSave decoder; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CPlayer::AutoAddAuctionGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14862
// RVA: 0x000466A0
// ADDRESS: 004466a0
// PROTOTYPE: void __thiscall AutoAddAuctionGoods(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountCiQingEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16353
// RVA: 0x00047000
// ADDRESS: 00447000
// PROTOTYPE: void __thiscall MountCiQingEquip(ulong param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00448346
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17020
// RVA: 0x00048346
// ADDRESS: 00448346
// PROTOTYPE: undefined Catch@00448346()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::~CPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:256
// RVA: 0x00049860
// ADDRESS: 00449860
// PROTOTYPE: void __thiscall ~CPlayer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:286
// RVA: 0x0004A2D0
// ADDRESS: 0044a2d0
// PROTOTYPE: uchar __thiscall GetLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:288
// RVA: 0x0004A2E0
// ADDRESS: 0044a2e0
// PROTOTYPE: ulong __thiscall GetExp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:365
// RVA: 0x0004A2F0
// ADDRESS: 0044a2f0
// PROTOTYPE: ulong __thiscall GetHP(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:366
// RVA: 0x0004A300
// ADDRESS: 0044a300
// PROTOTYPE: void __thiscall SetHP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMaxHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:389
// RVA: 0x0004A340
// ADDRESS: 0044a340
// PROTOTYPE: ulong __thiscall GetMaxHP(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMinAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:405
// RVA: 0x0004A350
// ADDRESS: 0044a350
// PROTOTYPE: ulong __thiscall GetMinAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMaxAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:407
// RVA: 0x0004A360
// ADDRESS: 0044a360
// PROTOTYPE: ulong __thiscall GetMaxAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetHit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:409
// RVA: 0x0004A370
// ADDRESS: 0044a370
// PROTOTYPE: ushort __thiscall GetHit(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetCCH
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:413
// RVA: 0x0004A380
// ADDRESS: 0044a380
// PROTOTYPE: ushort __thiscall GetCCH(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetDef
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:415
// RVA: 0x0004A390
// ADDRESS: 0044a390
// PROTOTYPE: ulong __thiscall GetDef(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetDodge
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:417
// RVA: 0x0004A3A0
// ADDRESS: 0044a3a0
// PROTOTYPE: ushort __thiscall GetDodge(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAtcSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:419
// RVA: 0x0004A3B0
// ADDRESS: 0044a3b0
// PROTOTYPE: short __thiscall GetAtcSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetElementResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:421
// RVA: 0x0004A3C0
// ADDRESS: 0044a3c0
// PROTOTYPE: ulong __thiscall GetElementResistant(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetHpRecoverSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:423
// RVA: 0x0004A3D0
// ADDRESS: 0044a3d0
// PROTOTYPE: ushort __thiscall GetHpRecoverSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMpRecoverSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:425
// RVA: 0x0004A3E0
// ADDRESS: 0044a3e0
// PROTOTYPE: ushort __thiscall GetMpRecoverSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetExalt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:428
// RVA: 0x0004A3F0
// ADDRESS: 0044a3f0
// PROTOTYPE: ulong __thiscall GetExalt(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetExalt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:429
// RVA: 0x0004A400
// ADDRESS: 0044a400
// PROTOTYPE: void __thiscall SetExalt(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetSoulResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:432
// RVA: 0x0004A410
// ADDRESS: 0044a410
// PROTOTYPE: ushort __thiscall GetSoulResistant(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAddElementAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:434
// RVA: 0x0004A420
// ADDRESS: 0044a420
// PROTOTYPE: ulong __thiscall GetAddElementAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAddSoulAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:436
// RVA: 0x0004A430
// ADDRESS: 0044a430
// PROTOTYPE: ushort __thiscall GetAddSoulAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetReAnk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:441
// RVA: 0x0004A440
// ADDRESS: 0044a440
// PROTOTYPE: ushort __thiscall GetReAnk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAttackAvoid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:444
// RVA: 0x0004A450
// ADDRESS: 0044a450
// PROTOTYPE: ushort __thiscall GetAttackAvoid(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetElementAvoid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:446
// RVA: 0x0004A460
// ADDRESS: 0044a460
// PROTOTYPE: ushort __thiscall GetElementAvoid(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetFullMiss
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:448
// RVA: 0x0004A470
// ADDRESS: 0044a470
// PROTOTYPE: ushort __thiscall GetFullMiss(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddToByteArray_ForClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:941
// RVA: 0x0004A480
// ADDRESS: 0044a480
// PROTOTYPE: bool __thiscall AddToByteArray_ForClient(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1252
// RVA: 0x0004BA80
// ADDRESS: 0044ba80
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ChangeRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1867
// RVA: 0x0004C400
// ADDRESS: 0044c400
// PROTOTYPE: bool __thiscall ChangeRegion(long param_1, long param_2, long param_3, long param_4, long param_5, long param_6, long param_7)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnStandOnSwitchPoint
// STATUS: IMPLEMENTED
// MATERIALIZED: `CGame::on_player_stand_on_switch_point` сохраняет сценарную
// точку, ограничения, возврат, уведомление и общий `ChangeRegion`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2596
// RVA: 0x0004D420
// ADDRESS: 0044d420
// PROTOTYPE: int __thiscall OnStandOnSwitchPoint(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnDied
// STATUS: PARTIALLY_IMPLEMENTED_NATION_AND_GODS_BATTLE
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:3306
// RVA: 0x0004D850
// ADDRESS: 0044d850
// PROTOTYPE: void __thiscall OnDied(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CheckLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:4930
// RVA: 0x00052DD0
// ADDRESS: 00452dd0
// PROTOTYPE: long __thiscall CheckLevel(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountAllEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5132
// RVA: 0x00053480
// ADDRESS: 00453480
// PROTOTYPE: void __thiscall MountAllEquip(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004535ee
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5183
// RVA: 0x000535EE
// ADDRESS: 004535ee
// PROTOTYPE: undefined Catch@004535ee()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::InitNameValueMap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8693
// RVA: 0x00055000
// ADDRESS: 00455000
// PROTOTYPE: void __thiscall InitNameValueMap(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8858
// RVA: 0x000577B0
// ADDRESS: 004577b0
// PROTOTYPE: ulong __thiscall GetValue(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8889
// RVA: 0x00057AF0
// ADDRESS: 00457af0
// PROTOTYPE: ulong __thiscall SetValue(char * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ChangeValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8927
// RVA: 0x00057E60
// ADDRESS: 00457e60
// PROTOTYPE: ulong __thiscall ChangeValue(char * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IncreaseContinuousKill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9593
// RVA: 0x00058390
// ADDRESS: 00458390
// PROTOTYPE: void __thiscall IncreaseContinuousKill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ComputerAddValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15311
// RVA: 0x000585D0
// ADDRESS: 004585d0
// PROTOTYPE: void __thiscall ComputerAddValue(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneTaoZhuang
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15678
// RVA: 0x00058810
// ADDRESS: 00458810
// PROTOTYPE: void __thiscall DoneTaoZhuang(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RunQuestCompleteScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17798
// RVA: 0x00058940
// ADDRESS: 00458940
// PROTOTYPE: long __thiscall RunQuestCompleteScript(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:89
// RVA: 0x000589B0
// ADDRESS: 004589b0
// PROTOTYPE: undefined __thiscall CPlayer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnEnterRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9681
// RVA: 0x0005A170
// ADDRESS: 0045a170
// PROTOTYPE: void __thiscall OnEnterRegion(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetNetExID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1037
// RVA: 0x0007BD10
// ADDRESS: 0047bd10
// PROTOTYPE: long __thiscall GetNetExID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetLastContainerScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1173
// RVA: 0x00093440
// ADDRESS: 00493440
// PROTOTYPE: char * __thiscall GetLastContainerScript(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCharged
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:317
// RVA: 0x000AEB30
// ADDRESS: 004aeb30
// PROTOTYPE: void __thiscall SetCharged(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMaxEnergy
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:526
// RVA: 0x000AEB40
// ADDRESS: 004aeb40
// PROTOTYPE: void __thiscall SetMaxEnergy(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCreateFactionOperator
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1023
// RVA: 0x000AEB60
// ADDRESS: 004aeb60
// PROTOTYPE: void __thiscall SetCreateFactionOperator(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetApplyJoinOperator
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1026
// RVA: 0x000AEB70
// ADDRESS: 004aeb70
// PROTOTYPE: void __thiscall SetApplyJoinOperator(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetFactionDeclareWarOperator
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1029
// RVA: 0x000AEB80
// ADDRESS: 004aeb80
// PROTOTYPE: void __thiscall SetFactionDeclareWarOperator(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetEnergy
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:524
// RVA: 0x000AED90
// ADDRESS: 004aed90
// PROTOTYPE: void __thiscall SetEnergy(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetLastContainerScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1172
// RVA: 0x000AEFC0
// ADDRESS: 004aefc0
// PROTOTYPE: void __thiscall SetLastContainerScript(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::PushItemToCiQingList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:252
// RVA: 0x000AF1E0
// ADDRESS: 004af1e0
// PROTOTYPE: void __thiscall PushItemToCiQingList(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetBlastAttackScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:458
// RVA: 0x001DFD60
// ADDRESS: 005dfd60
// PROTOTYPE: void __thiscall SetBlastAttackScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetElementBlastAttackScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:462
// RVA: 0x001DFD90
// ADDRESS: 005dfd90
// PROTOTYPE: void __thiscall SetElementBlastAttackScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetElementBlastDefendScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:464
// RVA: 0x001DFDC0
// ADDRESS: 005dfdc0
// PROTOTYPE: void __thiscall SetElementBlastDefendScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
