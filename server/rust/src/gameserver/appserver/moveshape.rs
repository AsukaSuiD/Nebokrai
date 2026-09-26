//! Реализованная часть `CMoveShape` исторического GameServer.
//! Постоянные данные CArchery/CBaseMagic/CFireBolt принадлежат экземпляру
//! навыка: att_time обнуляется конструктором, но не Begin/End или удалением
//! исполнения команды. Игрок и монстр используют одно и то же хранение.
//! EnergyBolt/SnakeBolt/ZombieClaw сохраняют там область первого успешного
//! Begin; End очищает путь и параметры полёта, не заменяя эту область.
//! ChuckStone/SkeletonArchery хранят полёт в том же экземпляре; их скаляры
//! сбрасываются перед возвратом движения. GetMinDistance общий для клиента
//! и ИИ, с конкретным usage владельца и signed-положительной границей 1.
//! UpdateProperty (0x004CFB60, moveshape.cpp:93) реализован общим живым
//! dispatcher-ом states/state.rs. Zone-агрегат хранит одну PDB-структуру
//! tagProperties (+0x84, 25 signed LONG); её читают native monster getters.
//! Снимки PlayerPropertyState/MonsterPropertyState больше не дублируют арену.
//! Первичная замена GodBless/Fog/BF использует общий поиск живой позиции и
//! регистрацию записи с поколенческим ключом/DB-span. Предикат выбора и
//! наличие destructor после End остаются у native caller-а, а не у storage.
//! AddExState/AddExStateNew также используют этот общий lifecycle: прежняя
//! пачка removed/added и копия владельца для отложенных visual устранены.
//! Кодек extended-state пишет запись по span общей арены; первичный payload
//! не получает фиктивный offset, а сохранение не зависит от способа установки.
//! Обновление remaining сохраняет исходные padding-байты загруженного Ex,
//! не пересоздавая всю запись из полей, которых нет в игровом контракте.
//! AddCHBYState (0x004D2590) также использует общую арену: параметры снимаются
//! до End/destructor всех совпавших уровней, object Begin выполняется до append.
//! DelCHBYState (0x004CEBD0) завершает лишь первое совпадение и отдельно
//! вызывает UpdateProperty; GetCHBYState (0x004CEC40) не вызывает callbacks.
//! CHBY-кодек пишет актуальные mode/hotkeys/flags/remaining по общему span,
//! сохраняя исходный padding; отдельный owning ChangeBodyMutation устранён.
//! AddUndeadState (0x004D1780) снимает параметры до End/destructor всех
//! совпавших type/inner ID; проверка ID0 идёт после этих завершений. Новый
//! object Begin выполняется до append, без заранее снятых часов и пачки
//! removed/added. DelUndeadState (0x004CDE80) завершает первое совпадение,
//! отдельно пересчитывает свойства и возвращает запрошенный ID.
//! Ride также проходит Begin до append: CMoveShape больше не создаёт owning
//! копию результата и не завершает payload отдельно от общего End/RemoveState.
//! Variable DB-span нового Ride задаётся общей регистрацией; loaded offset
//! остаётся лишь fallback-кодеком, а не условием удаления живого экземпляра.
//! Particular также использует общий Begin/append/End; дополнительный код
//! предмета проверяет native caller, storage не подавляет повторные записи.
//! AddState (0x004D1560) делегирует семь вариантов owner-у scriptstate:
//! успешный Begin → общая регистрация/span → virtual UpdateProperty.
//! Список и копия результата для сценарных состояний отдельно не создаются.
//! Клиентский AddToByteArray_ForClient (0x004CDD30, moveshape.cpp:1779)
//! перенесён в Zone `skills::state::snapshot` (волна Z-M3): он считает
//! непустые позиции общей арены, затем пишет ID/+30/+38 и Team-name
//! в том же порядке. DB Serialize и его offsets здесь не используются;
//! клиентские getters заданы в едином каталоге states/state.rs. IsEnded
//! не фильтрует запись; повторные ID и пустые позиции не меняют идентичность.
//! Общая цепочка client snapshot не снимает предварительный timestamp:
//! каждый конкретный getter читает динамические часы на своём месте.
//! Для ещё не загруженного opaque owner отказ от снимка остаётся явным
//! безопасным отличием: неизвестный клиентский контракт не заменяется пустым.
//! Общий Save для Ex/CHBY/Undead вызывает remaining-getter ровно один раз
//! на экземпляр: один остаток записывается по общему DB-span и сразу в живой
//! keep до перехода к следующей позиции. Начальные timestamp не сбрасываются;
//! отдельные проходы по типам больше не меняют порядок часов и мутаций.
//! Немедленный background-owner сохраняет признак End у навыка до следующего
//! OnExecuteBackStageSkills (0x004C88E0): сначала проверка IsEnded, затем AI.
//! Запись не извлекается перед callback; следующий проход ставит SKILL_UNKNOW,
//! ещё следующий удаляет пометку. Неуспешное исполнение не фиксирует End.
//! Повторные ID видят общий End зарегистрированного навыка; AutoStart сбрасывает
//! его перед Begin. Очереди команд игрока остаются у CPlayerAI.
//! End немедленного навыка отмечает сам owner независимо от active/background;
//! координатор фонового обхода не выводит завершение из общего bool результата.
//! Новый Begin сбрасывает этот же признак в AutoStart и допущенном active-пути;
//! сброс не затрагивает очередь, выбранный ID и время восстановления.
//! Begin немедленного навыка хранится отдельно от IsEnded: CState constructor
//! (0x005DBCA0) задаёт ended=true. Все 14 immediate-конструкторов сохраняют
//! это значение и обнуляют собственный флаг +0x4c. До Begin эффекта нет
//! (например EnlargeFullMiss::AI 0x0051673b), End снова снимает этот флаг.
//! Constructor/Begin/End принадлежат одному lifecycle экземпляра;
//! новый и завершённый immediate-навыки оба удовлетворяют IsEnded.
//! Достигнутый immediate End снимает derived-флаг и очищает ту же базу
//! CSkill; отметка не оставляет активный kernel с устаревшим IsEnded=false.
//! GetDefaultAttackSkillID (RVA 0x000CE240, moveshape.cpp:2464) выбирает
//! ID 2 только из attack-категории, иначе ID 3 из summon, иначе ID 1.
//! Прямые проходы intrinsic-категорий не зависят от QuerySkillType.
//! Выбранный ID не доказывает наличие навыка: GetCurrentSkill (RVA 0x000CDC10,
//! moveshape.cpp:1720) разрешает его через реестр. OnIdle монстра/питомца и
//! OnMoving питомца проверяют эту проекцию; отсутствующий default ID не
//! подавляет ChangeSkill/SearchEnemy и самопроизвольно не сбрасывается.
//!
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! исходные владельцы `appserver/moveshape.h/.cpp`. Сохранены точный порядок
//! смены пространственной принадлежности, двоичные форматы `0xBF603/604/605`,
//! счётчики запрета движения и боя, а также подтверждённая странность
//! `ForceMove`, где верхняя граница Y записывает `width - 1`.
//! Общая основа wire-команд `0xBF603/604/605` (порция 2 волны moveshape)
//! перенесена в Zone `regions/moveshape.rs`: методы ниже собирают её решение
//! типизированным результатом, а around-доставку и запись членства
//! (`set_move_shape_tile_position`) по-прежнему выполняет эта обвязка; span и
//! запрет клетки читаются через `RegionSpanView` у владельца хранилищ.
//! Его BF604 строится общим способом; CGame сохраняет виртуальный SetTileXY
//! игрока с отменой захвата до свежего AI Stand. Монстр и постройка используют
//! пространственную базу напрямую, не создавая второго способа переноса игрока.
//! `SetKilledMeAttackInfo` (0x004CCE50) сохраняет данные убийцы в общей
//! базе после пакета смерти 0xBF60B. Единственный KillingAttackIdentity содержит
//! только потребляемую OnDied-проекцию: тип, ID и guild ID атакующего.
//! Это не восстановление полного native-layout: остальные скопированные
//! модификаторы и флаги, пока не имеющие перенесённых потребителей, остаются
//! в локальном исследовательском корпусе. Ни конкретная форма, ни отложенный удар не дублируют эту запись.
//!
//! Все достигнутые применённые состояния принадлежат одному `CanonicalStateStorage`.
//! Сырой `ex_states` скрыт внутри `LegacyStateCodec` и служит только для
//! сохранения точного порядка, неизвестных записей и обратного двоичного кодека;
//! игровое поведение читает типизированные состояния. Native input и Serialize-
//! cache разделены: Tian потребляет 10 байт, а его cache-span занимает 12.
//! Границы загруженных записей связаны с StateKey; opaque tail и остаточный
//! declared count никогда не участвуют в typed append/update/remove.
//! Save дописывает этот tail без изменения; это сохранение неизвестных байтов,
//! а не гарантия native round-trip спорной записи. Полный сброс snapshot
//! удаляет и tail, выборочный End — только доказанный cache-span.
//! Общий UpdateProperty обнуляет единственный набор `property_modifiers`, затем
//! вызывает живые property-state в исходном порядке, включая повторные ID.
//! PDB `tagProperties` (type 0x6D94, fieldlist 0x6D93) задаёт 25 signed long,
//! размер 0x64 и поле CMoveShape +0x84. Имена и порядок полей сохранены
//! типизированной структурой; это не wire-layout и не копия свойств монстра.
//! Monster-getters читают этот результат, а не запускают состояния повторно.
//! Добавление, замена,
//! таймеры и удаление обновляют типизированную модель и её кодек в одной
//! операции с прежними смещениями и порядком.
//! Типизированные экземпляры используют общую арену
//! `AppliedStateEntries`: поколенческий ключ задаёт экземпляр, отдельный список
//! сохраняет исходную позицию и пустые места после удаления. Общий factory-проход
//! загружает каждую известную запись, не схлопывая повторные ID. Общий
//! UpdateAbnormality в states/state.rs вызывает AI текущего экземпляра в порядке
//! массива, перечитывая его после callback. Там же единый список virtual AI/End
//! обслуживает ClearAllStates (0x004CF090) и CastCure: End → перечитать позицию
//! → удалить оставшийся объект. Death-фильтры, отсутствие уплотнения и отдельные
//! UpdateProperty сохранены. Активные навыки не дублируются
//! ссылками в m_vStates: проверка ID 0x198 в CastCure не доказывает AddState.
//! Exact CSpiderMist наследует CSummonSkill и не регистрирует себя состоянием.
//! Save кодирует записи одним обходом арены, включая порядок чтения часов,
//! и выдаёт runtime-порядок только при полном сопоставлении с DB-кодеком.
//! Неизвестный хвост, нематериализованная запись или неоднозначное соответствие
//! сохраняют исходную раскладку вне подтверждённых обновлений. Padding не
//! восстанавливается из потерявшего его typed-поля. Клиентский snapshot пока
//! сохраняет wire-порядок и исходные offsets. Активные cast не добавляются
//! в DB-кодек без доказанного AddState и соответствующего формата записи.
//! Замена Cure сохраняет runtime-позицию и отдельный offset DB-записи:
//! между End и установкой нового экземпляра runtime-слот остаётся пустым.
//! Уплотнение выполняется в начале UpdateAbnormality и mutable GameSave,
//! как в 0x004CFD00/0x004D10F0. Клиентский snapshot не меняет позиции:
//! он может выполняться внутри End, пока Cure ещё хранит место замены.
//! Vec::splice возвращает DB-запись на место, сдвигая сохранённые смещения
//! соседей без их повторной загрузки и без сброса runtime-таймеров.
//! У повторных защитных щитов удаление, DB-сериализация и клиентский life
//! выбирают экземпляр по порядковому номеру среди того же ID, а не первый ID.
//! Сбор душ хранится здесь без таймера; его DB-запись кодирует тот же payload. Порядок
//! очищаемых и ослепляющих состояний проецируется из общей
//! арены, без отдельных ID-наборов, теряющих повторные экземпляры.
//! Рыцарский удар хранит здесь единственную каноническую блокировку движения
//! и боя; замена, истечение и снятие очищением меняют те же счётчики.
//! Подготовка яростного удара также имеет здесь единственный типизированный
//! экземпляр: замена, истечение и потребление `Flash` не касаются
//! скрытой устаревшей двоичной записи.
//! `PillarState` хранится здесь же: проверки рывков, строгий таймер и поздний
//! коэффициент защиты читают один экземпляр без параллельной сырой записи.
//! Blind/Rush/Rush2/Strike используют общий timed payload с отдельными ID и сроками
//! каждого экземпляра; их запреты и DB-записи принадлежат той же арене.
//! `CNotDisappearAfterDead` использует точный client-time override
//! `CExStateNew::GetRemainedTime`: нулевой срок и достигнутый wrapping deadline
//! дают `0`, иначе публикуется оставшийся DWORD.
//! Расходуемые и автоматические восстановления HP/MP также входят в общий
//! DB-кодек. 16-байтные расходуемые записи материализуются при загрузке,
//! активируются при входе и удаляются из wire вместе с живым состоянием, не
//! обрывая следующий record. RestoreHpMp (0x004455D0) завершает четыре
//! автоматических типа и Particular одним обходом исходных позиций, затем
//! добавляет четыре 12-байтные записи из актуальных свойств игрока.
//! RestoreHp/RestoreMp (0x00444C80/0x00444D50) сначала проверяют отдельный
//! cooldown по clock1 и записывают clock2 до создания состояния. Подготовка
//! здесь не выполняет Begin и append: owner читает clock3 в Begin, после чего
//! CGame регистрирует единственный экземпляр и DB-span в общей арене.
//! Эти cooldown-поля обнуляются конструктором игрока (0x00458F3D/0x00458F43),
//! но не очисткой состояний и не DecordFromByteArray (0x0044BA80).
//! Доступ к старому кодеку с порядком байтов от младшего к старшему выполняют
//! общие `LegacyReader` и `LegacyWriter`; доказанные границы записей теперь
//! предоставляет достигнутый `CStateFactory`, а применение состояний остаётся
//! у этого владельца.
//!
//! `AddSkill`, `DelSkill`, `ClearSkills` сохраняют общий реестр навыков.
//! Сам реестр (четыре категории, current/item-список и скалярная identity
//! записи) перенесён в Zone `regions/skillregistry.rs` (порция 4 волны
//! moveshape): переходный агрегат ниже хранит `SkillRegistry<MoveShapeSkill>`
//! и делегирует ему поведение без изменения сигнатур своих методов; execution
//! kernel и retained данные полёта вместе с полной записью перенесены в Zone
//! `skills/execution::RegisteredSkillRecord` (порция 5), `MoveShapeSkill` —
//! её специализация монстровым payload; вместе с ним alias перенесён туда же
//! волной Z-M4 (`skills/execution/monster.rs` + alias в `mod.rs`) и здесь лишь
//! реэкспортирован прежним именем. Exact
//! `Stiffen` (RVA 0x000CD2F0) идёт общей операцией Zone `regions/moveshape.rs`
//! над скалярами Zone-агрегата с setup value-формой.
//! AutoStartPassiveSkill (0x004CDBB0) обходит state-категорию в порядке
//! вставки, получает concrete GetAI и только при его наличии вызывает
//! Begin(self, self), затем WhenAddBackStageSkill этого же AI. Сам background-
//! список принадлежит CBaseAI. Удаление навыка не очищает списки других
//! владельцев: очередной OnExecute помечает отсутствующий ID как UNKNOWN.
//! Четыре независимые категории сохраняют экземпляры, их порядок и
//! повторные ID: native AddSkill (0x004D1C70) допускает
//! повторный ID, когда уровень первого найденного экземпляра равен нулю.
//! В каждой категории SlotMap владеет навыками, а Vec ключей задаёт только
//! native-порядок. Поколенческий SkillSlot переживает сдвиги этого Vec и
//! не разрешает вложенному callback завершить новую одноимённую регистрацию.
//! Порядок самого SlotMap не используется; удаление и очистка инвалидируют
//! ключи, а не пересоздают хранилище с прежними поколениями. Это техническая
//! замена указателей экземпляров, не дополнительный каталог ID либо owners.
//! Категорию вставки задаёт concrete constructor; GetSkill (0x004CE2D0)/DelSkill выбирают
//! её отдельно через актуальный QuerySkillType(ID, 1). Явная &CSkillFactory
//! сохраняет изменения reload, включая частично декодированный snapshot,
//! без копии категорий в форме. Повышение ненулевого уровня удаляет первое
//! совпадение и добавляет экземпляр в хвост. DelSkill отвергает UNKNOWN до
//! current cleanup, но ID 0 проходит cleanup и только потом category lookup.
//! AutoStart обращается прямо к каждому state-экземпляру, не разрешая заново
//! его ID; смена metadata-категории не подменяет объект этого обхода.
//! CSkill::GetSkillName (0x004D86E0) читает актуальные свойства по ID/уровню,
//! а не имя времени регистрации. None в name означает отсутствие записи;
//! локализованный GS0318 и пустой fallback разрешаются владельцем публикации.
//! Исполнение игрока, боевого духа либо монстра и принадлежащие навыку ресурсы
//! хранятся в единственной типизированной ячейке зарегистрированного экземпляра,
//! вместе с общим для этого экземпляра reuse timestamp. Единственная база
//! lifecycle хранится в Inactive до concrete Begin, затем перемещается внутрь
//! kernel игрока, боевого духа либо монстра. Установка concrete-данных сохраняет
//! эту базу, в том числе уже записанные общим Begin source/target и время.
//! Неуспешный Begin сам по себе не удаляет прежние concrete-данные.
//! Удаление только исполнения возвращает ту же базу в Inactive без Begin,
//! End и callback; завершение базы вызывается владельцем отдельно до удаления.
//! Общая очистка Player/BattleFairy требует точного typed dispatch того же
//! экземпляра. Проверочный enum не хранит исполнение и не объединяет их AI.
//! Monster-подготовка End и освобождение путей используют hooks того же
//! registered payload; порядок возврата движения остаётся у общей политики.
//! Это не удаление исполнения: фаза и ресурсы очищаются, но kernel с исходной
//! базой остаётся доступен до общего End и следующего active-прохода AI.
//! Изменяемый доступ не создаёт фиктивного monster-kernel и не подменяет
//! чужой вариант. Общий IsEnded не выводится из наличия concrete-исполнения;
//! отдельный immediate-флаг +0x4c не подменяет базовый lifecycle.
//! Визуальный ресурс CState принадлежит самому экземпляру отдельно от
//! копируемой скалярной базы. Общий сброс сначала очищает source/target/time,
//! затем удаляет Option<SkillVisualEffect> и только потом выставляет ended.
//! Общий visual-ресурс с concrete видом живёт отдельно от execution payload:
//! Rage/KnightCut создают эффект до cast-проверок, поэтому он принадлежит навыку
//! и при failed Begin без payload. Update заимствует тот же Option, не извлекает
//! ресурс для публикации и не дублирует source/ID/level общего экземпляра.
//! Владеющие формы и регионы не клонируются: временным рассылкам достаточно
//! упорядоченного снимка адресатов, случайной позиции — заимствования CRegion.
//! CSkill constructor (0x004D8120) задаёт timestamp +0x40 равным нулю;
//! новая регистрация не наследует его от удалённого экземпляра того же ID.
//! Доступ к этим полям использует тот же первый GetSkill по текущей metadata,
//! без дополнительного реестра и без поиска по intrinsic-категории. Отсутствие
//! kernel не означает отсутствия самого registered owner-а. Общий End
//! в states/skill.rs уже обслуживает Rage/KnightCut, в том числе без payload,
//! и терминальный BattleFairy с owned visual. Остальные
//! concrete callers ещё не все используют эту границу.
//! StopAllSkills (0x004CDF50) вызывает End(0) каждого экземпляра в порядке
//! attack → defense → summon → state, не очищая AI target/FIFO/background.
//! Общий обход в states/skill.rs подключён перед приручением монстра:
//! Attack/Defense/Summon перечитывают длину, State фиксирует исходную длину,
//! каждый индекс снова берётся из реестра. Ключ захватывается только на End.
//! DelSkill для remote/script, realm и item-reuse синхронно завершает current,
//! затем storage удаляет первый экземпляр без дополнительного End. Вложенные
//! equipment/war-soul callers ещё используют локальную границу без общего End;
//! полный StopAll перед смертью требует опубликованного регионального owner-а.
//! В Luvinia Application/MoveShape.cpp AddSkill/DelSkill остались пустыми,
//! а StopAllSkills работает с другой active-module map и удаляет её записи.
//! Старый GetSkill сохранился в отключённом OtherMessage; world factory
//! хранит конфигурации. Эти формы не заменяют четыре owner-вектора Miracle.
//! Доказательства этих и остальных недостигнутых методов сохранены ниже.
//!
//! Волна Z-M5 перенесла сам агрегат `CMoveShape` (пространственную базу,
//! реестр навыков, каноническую арену состояний и скалярные колонки) в Zone
//! `regions/moveshape::MoveShapeState` вместе с `Default`, Deref-швом в арену
//! и accessor-ами чтения/записи колонок (moveable/can_fight/stiffen/
//! killed_by/pets, god, property modifiers, сброс входа в регион). Здесь
//! остаётся тонкая оболочка старого пакета: делегаты навыков и типизированных
//! состояний, клиентские snapshot-фасады, `auto_start_passive_skills` и
//! dispatcher-оркестрация wire-команд `0xBF603/604/605` (её перенос
//! зарезервирован волной Z-M6). `Deref` оболочки ведёт в `MoveShapeState`,
//! тот — в `CanonicalStateStorage`: прежний двойной шов потребителей,
//! включая прямой доступ к полям `ex_states`/`state_entries`, не меняется.

// Арена состояний, её enum-каталог, codec/интервалы, читающие проекции
// семейств (`accessors`), мутирующие операции и RAW-записи (`mutations`),
// DB Save/Load-кодек (`serialization`) и клиентский писатель снимка
// (`snapshot`) перенесены
// в Zone `skills::state`; путь `super::moveshape` сохраняет прежние имена.
pub(crate) use nebokrai_zone::skills::state::{
    AppliedState, StateBatch, StateData, StateKey,
};
use nebokrai_zone::skills::state::read_u32;

use std::ops::{Deref, DerefMut};

use super::ai::baseai::CBaseAI;
use super::chbystate::ChangeBodyState;
use super::exstate::{ExtendedState, ExtendedStateKind};
use super::particularstate::ParticularState;
use super::region::{CRegion, RegionCellAccessBlock};
use super::ridestate::RideState;
use super::restorestate::ConsumableRestoreState;
use super::scriptstate::ScriptMoveState;
use super::serverregion::CServerRegion;
use super::skills::kernel::{BattleFairyExecution, PlayerSkillExecution, SkillLifecycle, SkillTermination};
use super::states::visualeffect::SkillVisualEffect;
use super::teamstate::CTeamState;
use super::shape::{CShape, ShapeAreaCoordinates, ShapeFigure, ShapeIdentity, ShapeResolver};
use crate::gameserver::appserver::skills::immediatestate::is_immediate_state_skill;
use crate::gameserver::appserver::skills::curestate::{CureState, CURE_STATE_BYTES};
use crate::gameserver::appserver::skills::enlargefullmissstate::EnlargeFullMissState;
use crate::gameserver::appserver::skills::enlargemaxhpstate::EnlargeMaxHpState;
use crate::gameserver::appserver::skills::enlargemaxmpstate::EnlargeMaxMpState;
use crate::gameserver::appserver::skills::energyholdingstate::{
    EnergyHoldingState,
};
use crate::gameserver::appserver::skills::originstate::OriginState;
use crate::gameserver::appserver::skills::pillarstate::{
    PillarState,
};
use crate::gameserver::appserver::skills::spiderwebstate::SpiderWebState;
use crate::gameserver::appserver::skills::swordshipstate::SwordshipState;
use crate::gameserver::appserver::skills::battlefairyattributestate::BattleFairyAttributeState;
use crate::gameserver::appserver::skills::bossbluefurystate::BossBlueFuryState;
use crate::gameserver::appserver::skills::bossbluequakestate::BossBlueQuakeState;
use crate::gameserver::appserver::skills::skillfactory::{CSkillFactory, SkillCategory};
use crate::gameserver::appserver::skills::statefactory::known_state_record_offsets;
use crate::gameserver::appserver::skills::shieldstate::DefenseShieldState;
use crate::gameserver::appserver::skills::taijistate::TaiJiState;
use crate::gameserver::appserver::skills::tianshenxiafanstate::{
    TianShenXiaFanState,
};
use crate::gameserver::appserver::skills::wangshengstate::{
    WangshengState,
};
use crate::gameserver::appserver::skills::wuxingstate::WuXingState;
use crate::gameserver::appserver::states::automaticrestore::AutomaticRestoreState;
use crate::nets::netserver::message::{CMessage, GameServerAroundRuntime};
use crate::nets::netserver::message::GameMessageDomainOps;
use nebokrai_zone::regions::moveshape::{
    MoveShapeSetPositionOutcome, MoveShapeState, RegionSpanView, force_move_wire, on_move_wire,
    on_set_position_wire, set_pos_xy_core,
};
pub(crate) use nebokrai_zone::regions::moveshape::{
    KillingAttackIdentity, MoveShapeCommandBlock, MoveShapePet, MoveShapePositionBlock,
    MoveShapePositionFacts, MoveShapePropertyModifiers,
};
pub(crate) use nebokrai_zone::regions::skillregistry::{SKILL_BASE_DEFENSE, SkillSlot};
pub(crate) use nebokrai_zone::skills::execution::RegisteredSkillDispatch;

const SKILL_NOT_DISAPPEAR_AFTER_DEAD: u32 = 56;
const SKILL_USAGE_CONST: u32 = 20_010;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;

/// Полная запись зарегистрированного навыка: скалярная база `SkillIdentity`
/// (Zone `regions/skillregistry`), execution kernel и retained данные полёта.
/// Запись перенесена в Zone `skills/execution::RegisteredSkillRecord` (порция 5
/// волны moveshape, тела фасадов — буквально); payload исполнения монстра,
/// его сварка `MonsterSkillExecutionAccess` и каталог impl-ов
/// `MonsterSkillProgressState<M>` растворены там же волной Z-M4
/// (`skills/execution/monster.rs`), а alias `MoveShapeSkill` живёт в
/// `skills/execution/mod.rs` рядом с обоими операндами специализации.
/// Ниже сохранён re-export прежнего имени.
pub(crate) use nebokrai_zone::skills::execution::MoveShapeSkill;


pub(crate) use nebokrai_zone::effects::UndeadState;

/// Тонкая оболочка переходного Game: фабрика навыков остаётся у старого
/// владельца, а данные и 76-байтный кодек — в Zone `effects/undead.rs`.
pub(crate) fn undead_state_from_factory(
    state_id: u32,
    factory: &CSkillFactory,
) -> Option<UndeadState> {
    factory
        .query_skill_base_properties(SKILL_NOT_DISAPPEAR_AFTER_DEAD, state_id as i32)
        .map(|properties| UndeadState::from_properties(state_id, |usage| {
            properties.query_property(usage)
        }))
}



/// Шов Zone-основы wire-команд `0xBF603/604/605` к переходному владельцу
/// хранилищ: основе достаточно span и запрета клетки, а вся запись позиции
/// (`set_move_shape_tile_position`) и around-доставка остаются этой обвязке.
impl RegionSpanView for CServerRegion {
    fn width(&self) -> i32 {
        self.region.width
    }

    fn height(&self) -> i32 {
        self.region.height
    }

    fn get_block(&self, x: i32, y: i32) -> Result<u8, RegionCellAccessBlock> {
        self.region.get_block(x, y)
    }
}

pub(crate) trait MoveShapeResolver: ShapeResolver {
    /// `Some` означает успешный RTTI `CShape -> CMoveShape`; значение хранит
    /// exact `!IsDied`, полученный у concrete derived owner-а.
    fn move_shape_is_alive(&self, identity: ShapeIdentity) -> Option<bool>;
}

/// Тонкая оболочка старого пакета над Zone-агрегатом
/// `regions/moveshape::MoveShapeState` (волна Z-M5): колонки и их accessor-ы
/// перенесены туда, а здесь остаются делегаты навыков и типизированных
/// состояний, клиентские snapshot-фасады, `auto_start_passive_skills` (её
/// фоновый список принадлежит hub-владельцу `CBaseAI`, и оба caller-а
/// находятся в старом пакете) и dispatcher-оркестрация wire-команд
/// `0xBF603/604/605` — её перенос зарезервирован волной Z-M6.
/// `Deref` ведёт в Zone-агрегат, который сам разворачивает
/// `CanonicalStateStorage` — прежний двойной шов потребителей не меняется.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CMoveShape {
    inner: MoveShapeState,
}

impl Deref for CMoveShape {
    type Target = MoveShapeState;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for CMoveShape {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Default for CMoveShape {
    fn default() -> Self {
        Self {
            inner: MoveShapeState::default(),
        }
    }
}

impl CMoveShape {
    pub(crate) const fn property_modifiers(&self) -> &MoveShapePropertyModifiers {
        self.inner.property_modifiers()
    }

    pub(crate) const fn property_modifiers_mut(&mut self) -> &mut MoveShapePropertyModifiers {
        self.inner.property_modifiers_mut()
    }

    pub(crate) fn reset_property_modifiers(&mut self) {
        self.inner.reset_property_modifiers();
    }

    pub(crate) fn set_killed_by(&mut self, attack: KillingAttackIdentity) {
        self.inner.set_killed_by(attack);
    }

    pub(crate) const fn killed_by(&self) -> Option<KillingAttackIdentity> {
        self.inner.killed_by()
    }

    /// Exact `CMoveShape::Stiffen` (`RVA 0x000CD2F0`) идёт общей операцией Zone
    /// `regions/moveshape`: скаляры окна/live-limitа остаются полями Zone-
    /// агрегата, setup передаётся туда value-формой (`GlobeStiffenSetup` —
    /// POD проекция Shared resources, а не ссылка на setup-владельца).
    pub(crate) fn stiffen(
        &mut self,
        damage: u16,
        maximum_hp: u32,
        reank: u16,
        setup: crate::setup::globesetup::GlobeStiffenSetup,
        now_ms: impl FnMut() -> u32,
        random: impl FnMut(i32) -> i32,
    ) -> u32 {
        self.inner.stiffen(damage, maximum_hp, reank, setup, now_ms, random)
    }

    pub(crate) const fn current_pets_mode(&self) -> i32 {
        self.inner.current_pets_mode()
    }

    pub(crate) fn set_current_pets_mode(&mut self, mode: i32) -> bool {
        self.inner.set_current_pets_mode(mode)
    }

    pub(crate) fn add_pet(&mut self, object_type: i32, id: i32, figure: i32) {
        self.inner.add_pet(object_type, id, figure);
    }

    pub(crate) fn remove_pet(&mut self, object_type: i32, id: i32) -> bool {
        self.inner.remove_pet(object_type, id)
    }

    pub(crate) fn pets(&self) -> &[MoveShapePet] {
        self.inner.pets()
    }

    pub(crate) const fn shape(&self) -> &CShape {
        self.inner.shape()
    }

    pub(crate) const fn shape_mut(&mut self) -> &mut CShape {
        self.inner.shape_mut()
    }

    pub(crate) fn skill_level(&self, skill_id: u32, factory: &CSkillFactory) -> i32 {
        self.skill(skill_id, factory).map_or(0, MoveShapeSkill::level)
    }

    /// Точный fresh-object prefix `CMoveShape::AddToByteArray_ForClient`
    /// (died-byte и нулевой count состояний) формирует общий писатель Zone
    /// `skills::state::snapshot` (волна Z-M3) над этими же `CShape`.
    pub(crate) fn encode_fresh_client_snapshot(
        &self,
        include_child: bool,
        is_dead: bool,
    ) -> Option<Vec<u8>> {
        nebokrai_zone::skills::state::encode_fresh_client_snapshot(
            &self.shape,
            include_child,
            is_dead,
        )
    }

    /// Общий CMoveShape::AddToByteArray_ForClient идёт тем же писателем Zone
    /// `skills::state::snapshot` из живых экземпляров общей арены. Незагруженный
    /// opaque owner по-прежнему блокирует snapshot.
    pub(crate) fn encode_client_snapshot(
        &self,
        include_child: bool,
        is_dead: bool,
        timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        nebokrai_zone::skills::state::encode_client_snapshot(
            &self.shape,
            &self.state_storage,
            include_child,
            is_dead,
            timed_state_now_milliseconds,
        )
    }

    /// CMoveShape::AddToByteArray_ForClient (0x004CDD30): два прохода
    /// живого m_vStates — писатель Zone `skills::state::snapshot`.
    /// Player-owner по-прежнему передаёт канонический размер CTeam.
    pub(crate) fn encode_client_snapshot_with_team_count(
        &self,
        include_child: bool,
        is_dead: bool,
        team_member_count: usize,
        timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        nebokrai_zone::skills::state::encode_client_snapshot_with_team_count(
            &self.shape,
            &self.state_storage,
            include_child,
            is_dead,
            team_member_count,
            timed_state_now_milliseconds,
        )
    }

    /// Exact inline `CMoveShape::God`: runtime-only invulnerability flag не
    /// сериализуется и проверяется ordinary `OnBeenAttacked` owner-ом.
    pub(crate) const fn set_god(&mut self, enabled: bool) {
        self.inner.set_god(enabled);
    }

    pub(crate) const fn is_god(&self) -> bool {
        self.inner.is_god()
    }

    /// Счётчик запретов боя идёт общей операцией Zone moveshape.
    pub(crate) const fn set_fightable(&mut self, fightable: bool) {
        self.inner.set_fightable(fightable);
    }

    pub(crate) const fn can_fight(&self) -> bool {
        self.inner.can_fight()
    }

    pub(crate) fn skills(&self) -> impl Iterator<Item = &MoveShapeSkill> {
        self.skills.skills()
    }

    pub(crate) fn skills_in_category(&self, category: SkillCategory) -> impl ExactSizeIterator<Item = &MoveShapeSkill> {
        self.skills.skills_in_category(category)
    }

    /// `AutoStartPassiveSkill`: state-вектор обходится в порядке
    /// вставки, а каждый `IsAutoStart != 0` добавляется в background-очередь.
    /// Self-target `Begin(this, this)` в Rust задаётся самим владельцем.
    pub(crate) fn auto_start_passive_skills(&mut self, ai: &mut CBaseAI) -> usize {
        let mut count = 0;
        for skill in self.skills.skills_in_category(SkillCategory::State) {
            if is_immediate_state_skill(skill.id()) {
                ai.add_pending_back_stage_skill(skill.id());
                count += 1;
            }
        }
        count
    }

    pub(crate) fn undead_states(&self) -> impl Iterator<Item = &UndeadState> {
        self.state_entries.iter::<UndeadState>()
    }

    pub(crate) fn serialize_ex_states_for_save(
        &mut self,
        now_ms: u32,
        timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Vec<u8> {
        let storage = &mut self.inner.state_storage;
        nebokrai_zone::skills::state::serialize_ex_states_for_save(
            &mut storage.ex_states,
            &mut storage.state_entries,
            now_ms,
            timed_state_now_milliseconds,
        )
    }

    pub(crate) fn serialized_ex_states(
        &self,
        now_ms: u32,
        timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Vec<u8> {
        nebokrai_zone::skills::state::serialized_ex_states(
            &self.state_storage.ex_states,
            &self.state_storage.state_entries,
            now_ms,
            timed_state_now_milliseconds,
        )
    }

    pub(crate) fn replace_ex_states(&mut self, states: Vec<u8>, skill_factory: &CSkillFactory, now: &mut dyn FnMut() -> u32) {
        let state_owner = self.shape.identity();
        let storage = &mut self.inner.state_storage;
        nebokrai_zone::skills::state::replace_ex_states(
            &mut storage.ex_states,
            &mut storage.state_entries,
            state_owner,
            states,
            skill_factory,
            now,
        );
    }

    pub(crate) fn clear_persisted_runtime_state(&mut self) {
        let state = &mut self.inner;
        nebokrai_zone::skills::state::clear_persisted_runtime_state(
            &mut state.skills,
            &mut state.state_storage.ex_states,
            &mut state.state_storage.state_entries,
            &mut state.can_fight_count,
            &mut state.can_fight,
        );
    }

    pub(crate) fn ride_state(&self) -> Option<&RideState> {
        nebokrai_zone::skills::state::ride_state(&self.state_storage)
    }

    /// Scalar-prefix сброса при входе в регион идёт общей операцией Zone
    /// moveshape; конкретные Begin заново устанавливают свои запреты после
    /// этого сброса в общем живом проходе.
    pub(crate) const fn reset_region_entry_control(&mut self) {
        self.inner.reset_region_entry_control();
    }

    pub(crate) fn has_ride_state(&self) -> bool {
        nebokrai_zone::skills::state::has_ride_state(&self.state_storage)
    }

    /// Хвост RestoreHpMp (0x00445643..0x00445901): четыре новых экземпляра
    /// после общего End-обхода. Begin(null, holder) не читает часы; старые
    /// состояния здесь повторно не удаляются и UpdateProperty не вызывается.
    pub(crate) fn append_automatic_hp_mp_states(
        &mut self,
        properties: super::player::PlayerCombatProperties,
    ) {
        let region_id = self.shape.get_region_id();
        nebokrai_zone::skills::state::append_automatic_hp_mp_states(
            &mut self.inner.state_storage,
            properties,
            region_id,
        );
    }

    /// Общий NULL-user Begin свежего restore: без clock и пакета, visual loop=1.
    pub(crate) fn append_automatic_restore_state(&mut self, state: AutomaticRestoreState) -> StateKey {
        let region_id = self.shape.get_region_id();
        nebokrai_zone::skills::state::append_automatic_restore_state(
            &mut self.inner.state_storage,
            state,
            region_id,
        )
    }



    /// Cooldown и constructor до отдельного Begin: отказ читает только clock1,
    /// допуск записывает clock2; timestamp нового payload пока остаётся нулём.
    pub(crate) fn prepare_consumable_restore(
        &mut self,
        health: bool,
        amount: u32,
        time_to_keep_ms: u32,
        frequency_ms: u32,
        interval_ms: u32,
        now: &mut dyn FnMut() -> u32,
    ) -> Option<ConsumableRestoreState> {
        nebokrai_zone::skills::state::prepare_consumable_restore(
            &mut self.state_storage,
            health,
            amount,
            time_to_keep_ms,
            frequency_ms,
            interval_ms,
            now,
        )
    }

    pub(crate) fn particular_states(&self) -> impl Iterator<Item = &ParticularState> {
        nebokrai_zone::skills::state::particular_states(&self.state_storage)
    }

    pub(crate) fn team_recruitment_states(&self) -> impl Iterator<Item = &CTeamState> {
        nebokrai_zone::skills::state::team_recruitment_states(&self.state_storage)
    }


    pub(crate) fn automatic_restore_state(&self, key: StateKey) -> Option<AutomaticRestoreState> {
        nebokrai_zone::skills::state::automatic_restore_state(&self.state_storage, key)
    }

    pub(crate) fn automatic_restore_state_mut(
        &mut self,
        key: StateKey,
    ) -> Option<&mut AutomaticRestoreState> {
        nebokrai_zone::skills::state::automatic_restore_state_mut(&mut self.state_storage, key)
    }

    /// Точный `GetStateNumByStateID`: считает все живые экземпляры с данным
    /// базовым `CState::m_lID`, независимо от concrete owner-а состояния.
    pub(crate) fn state_count_by_state_id(&self, state_id: i32) -> u32 {
        nebokrai_zone::skills::state::state_count_by_state_id(&self.state_storage, state_id)
    }

    /// `GetStateBySkillID` просматривает канонические типизированные состояния
    /// по фактическому идентификатору навыка, а не по классу сетевой записи.
    pub(crate) fn has_state_by_skill_id(&self, state_id: u32) -> bool {
        nebokrai_zone::skills::state::has_state_by_skill_id(&self.state_storage, state_id)
    }


    pub(crate) fn swordship_states(&self) -> impl Iterator<Item = &SwordshipState> {
        nebokrai_zone::skills::state::swordship_states(&self.state_storage)
    }

    pub(crate) fn wuxing_states(&self) -> impl Iterator<Item = &WuXingState> {
        nebokrai_zone::skills::state::wuxing_states(&self.state_storage)
    }

    pub(crate) fn taiji_state(&self) -> Option<TaiJiState> {
        nebokrai_zone::skills::state::taiji_state(&self.state_storage)
    }

    pub(crate) fn enlarge_full_miss_state(&self) -> Option<EnlargeFullMissState> {
        nebokrai_zone::skills::state::enlarge_full_miss_state(&self.state_storage)
    }

    pub(crate) fn enlarge_max_hp_state(&self) -> Option<EnlargeMaxHpState> {
        nebokrai_zone::skills::state::enlarge_max_hp_state(&self.state_storage)
    }

    pub(crate) fn enlarge_max_mp_state(&self) -> Option<EnlargeMaxMpState> {
        nebokrai_zone::skills::state::enlarge_max_mp_state(&self.state_storage)
    }

    pub(crate) fn origin_state(&self) -> Option<OriginState> {
        nebokrai_zone::skills::state::origin_state(&self.state_storage)
    }








    pub(crate) fn take_boss_blue_fury_state(&mut self) -> Option<BossBlueFuryState> {
        nebokrai_zone::skills::state::take_boss_blue_fury_state(&mut self.state_storage)
    }

    pub(crate) fn begin_boss_blue_fury_state(&mut self, state: BossBlueFuryState) {
        nebokrai_zone::skills::state::begin_boss_blue_fury_state(&mut self.state_storage, state);
    }

    pub(crate) fn boss_blue_fury_state(&self) -> Option<BossBlueFuryState> {
        nebokrai_zone::skills::state::boss_blue_fury_state(&self.state_storage)
    }





    pub(crate) fn replace_boss_blue_quake_state(
        &mut self,
        state: BossBlueQuakeState,
    ) -> Option<BossBlueQuakeState> {
        nebokrai_zone::skills::state::replace_boss_blue_quake_state(&mut self.state_storage, state)
    }





    pub(crate) fn take_boss_blue_quake_state(&mut self) -> Option<BossBlueQuakeState> {
        nebokrai_zone::skills::state::take_boss_blue_quake_state(&mut self.state_storage)
    }

    pub(crate) fn promotion_magic_attack_factor(&self) -> Option<u16> {
        nebokrai_zone::skills::state::promotion_magic_attack_factor(&self.state_storage)
    }


    pub(crate) fn defense_shields(&self) -> impl Iterator<Item = &DefenseShieldState> {
        nebokrai_zone::skills::state::defense_shields(&self.state_storage)
    }

    pub(crate) fn defense_shield_keys(&self) -> Vec<StateKey> {
        nebokrai_zone::skills::state::defense_shield_keys(&self.state_storage)
    }

    pub(crate) fn defense_shield_key(&self, skill_id: u32) -> Option<StateKey> {
        nebokrai_zone::skills::state::defense_shield_key(&self.state_storage, skill_id)
    }

    pub(crate) fn defense_shield(&self, key: StateKey) -> Option<&DefenseShieldState> {
        nebokrai_zone::skills::state::defense_shield(&self.state_storage, key)
    }

    pub(crate) fn remove_defense_shield(&mut self, skill_id: u32) -> Option<DefenseShieldState> {
        nebokrai_zone::skills::state::remove_defense_shield(&mut self.state_storage, skill_id)
    }

    pub(crate) fn remove_defense_shield_key(&mut self, key: StateKey) -> Option<DefenseShieldState> {
        nebokrai_zone::skills::state::remove_defense_shield_key(&mut self.state_storage, key)
    }



    /// Общий push_back уже успешно начатого concrete state. DB-cache здесь
    /// технический: record подготовлен owner-ом без вызова игрового Serialize
    /// и без повторных часов. Новый span принадлежит тому же поколенческому
    /// ключу, поэтому дубли ID удаляются независимо. End/Update остаются caller-у.
    pub(crate) fn append_applied_state_record<T: AppliedState>(
        &mut self, state: T, record: &[u8],
    ) -> StateKey {
        nebokrai_zone::skills::state::append_applied_state_record(&mut self.state_storage, state, record)
    }

    /// Первый живой слот исходного m_vStates. Предикат задаёт игровой выбор
    /// caller-а; метод не копирует payload, не уплотняет и не вызывает End.
    /// После callback caller перечитывает эту позицию, если это требует EXE.
    pub(crate) fn find_state_position(
        &self, matches: impl FnMut(&StateData) -> bool,
    ) -> Option<(usize, StateKey)> {
        nebokrai_zone::skills::state::find_state_position(&self.state_storage, matches)
    }

    pub(crate) fn take_defense_shields(&mut self) -> StateBatch<DefenseShieldState> {
        nebokrai_zone::skills::state::take_defense_shields(&mut self.state_storage)
    }

    pub(crate) fn restore_defense_shields(&mut self, states: StateBatch<DefenseShieldState>) {
        nebokrai_zone::skills::state::restore_defense_shields(&mut self.state_storage, states);
    }

    /// Позиция замены и техническое место DB-записи. Caller сохраняет их
    /// до End: Begin новой записи не обязан добавлять её в конец m_vStates.
    pub(crate) fn applied_state_replacement_location(&self, key: StateKey) -> Option<(usize, usize)> {
        let state_id = self.applied_state_data(key)?.state_id();
        let position = self.state_entries.index_of(key)?;
        if let Some((offset, _)) = self.state_entries.serialized_span(key) {
            return Some((position, offset));
        }
        let ordinal = self.state_entries.entries().filter(|(_, state)| state.state_id() == state_id)
            .position(|(entry, _)| entry == key)?;
        let offset = known_state_record_offsets(&self.ex_states).into_iter()
            .filter(|offset| read_u32(&self.ex_states, *offset) == Some(state_id))
            .nth(ordinal)?;
        Some((position, offset))
    }

    /// Регистрация уже начатого состояния в освобождённой caller-ом позиции.
    /// Сериализованный cache и все spans сдвигаются один раз для любого owner-а;
    /// здесь нет End, игровых часов, Serialize или UpdateProperty.
    pub(crate) fn insert_replacement_state_record<T: AppliedState>(
        &mut self, state: T, record: &[u8], location: (usize, usize),
    ) -> Option<StateKey> {
        nebokrai_zone::skills::state::insert_replacement_state_record(
            &mut self.state_storage,
            state,
            record,
            location,
        )
    }



    pub(crate) fn state_slot_count(&self) -> usize {
        self.state_entries.len()
    }

    pub(crate) fn mark_applied_state_ended(&mut self, key: StateKey) -> bool {
        self.state_entries.mark_ended(key)
    }

    pub(crate) fn mark_applied_state_begun(&mut self, key: StateKey) -> bool {
        let region_id = self.shape.get_region_id();
        self.state_entries.mark_begun(key, region_id)
    }

    pub(crate) fn applied_state_user(&self, key: StateKey) -> Option<(i32, ShapeIdentity)> {
        self.state_entries.user(key, self.shape.get_region_id(), self.shape.identity())
    }

    pub(crate) fn applied_state_sufferer(&self, key: StateKey) -> Option<(i32, ShapeIdentity)> {
        self.state_entries.sufferer(key, self.shape.get_region_id(), self.shape.identity())
    }

    pub(crate) fn set_applied_state_user(&mut self, key: StateKey, user: Option<(i32, ShapeIdentity)>) -> bool {
        self.state_entries.set_user(key, user)
    }

    pub(crate) fn set_applied_state_sufferer(&mut self, key: StateKey, sufferer: Option<(i32, ShapeIdentity)>) -> bool {
        self.state_entries.set_sufferer(key, sufferer)
    }

    pub(crate) fn set_applied_state_user_region(&mut self, key: StateKey, region_id: i32) -> bool {
        self.state_entries.set_user_region(key, region_id)
    }

    pub(crate) fn set_applied_state_sufferer_region(&mut self, key: StateKey, region_id: i32) -> bool {
        self.state_entries.set_sufferer_region(key, region_id)
    }

    pub(crate) fn applied_state_has_visual(&self, key: StateKey) -> bool {
        self.state_entries.has_visual(key)
    }

    pub(crate) fn applied_state_visual_ended(&self, key: StateKey) -> Option<bool> {
        self.state_entries.visual_ended(key)
    }

    pub(crate) fn begin_applied_state_visual(&mut self, key: StateKey, loop_value: i32) -> bool {
        self.state_entries.begin_visual(key, loop_value)
    }

    pub(crate) fn update_applied_state_visual_base(&mut self, key: StateKey) -> bool {
        self.state_entries.update_visual_base(key)
    }

    pub(crate) fn applied_state_key<T: AppliedState>(&self) -> Option<StateKey> {
        self.state_entries.first_key::<T>()
    }

    pub(crate) fn applied_state_keys<T: AppliedState>(&self) -> Vec<StateKey> {
        self.state_entries.keys::<T>()
    }

    pub(crate) fn applied_state<T: AppliedState>(&self, key: StateKey) -> Option<&T> {
        T::as_data_ref(self.state_entries.get(key)?)
    }

    pub(crate) fn applied_state_mut<T: AppliedState>(&mut self, key: StateKey) -> Option<&mut T> {
        T::as_data_mut(self.state_entries.get_mut(key)?)
    }

    pub(crate) fn applied_state_data(&self, key: StateKey) -> Option<&StateData> {
        self.state_entries.get(key)
    }

    pub(crate) fn compact_state_slots(&mut self) -> bool {
        self.state_entries.compact()
    }

    /// Только финальный сброс контейнера после игрового ClearAllStates(false).
    /// SlotMap сохраняет поколения; это не повторный End оставшихся объектов.
    pub(crate) fn release_state_slots(&mut self) {
        self.state_entries.clear();
        self.ex_states.replace(0u32.to_le_bytes().to_vec());
    }

    pub(crate) fn state_at(&self, position: usize) -> Option<(StateKey, &StateData)> {
        let key = self.state_entries.address(position)?;
        Some((key, self.state_entries.get(key)?))
    }

    pub(crate) fn cure_state_key(&self) -> Option<StateKey> {
        self.state_entries.first_key::<CureState>()
    }

    pub(crate) fn cure_state_by_key(&self, key: StateKey) -> Option<CureState> {
        CureState::as_data_ref(self.state_entries.get(key)?).copied()
    }

    pub(crate) fn cure_state(&self) -> Option<CureState> {
        self.state_entries.first::<CureState>().copied()
    }

    pub(crate) fn take_cure_state(&mut self) -> Option<CureState> {
        self.remove_cure_state_by_key(self.cure_state_key()?)
    }

    pub(crate) fn remove_cure_state_by_key(&mut self, key: StateKey) -> Option<CureState> {
        self.remove_applied_state_record::<CureState>(key, CURE_STATE_BYTES)
    }

    pub(crate) fn remove_applied_state_record<T: AppliedState>(
        &mut self,
        key: StateKey,
        amount: usize,
    ) -> Option<T> {
        nebokrai_zone::skills::state::remove_applied_state_record(
            &mut self.state_storage,
            key,
            amount,
        )
    }

    /// Только удаление точного payload и его wire-записи; игровой End с
    /// visual, счётчиками и UpdateProperty выполняется владельцем снаружи.
    pub(crate) fn remove_applied_state_data(
        &mut self,
        key: StateKey,
        amount: usize,
    ) -> Option<StateData> {
        nebokrai_zone::skills::state::remove_applied_state_data(&mut self.state_storage, key, amount)
    }

    /// Destructor-only хвост ClearAllStates: wire-размер берётся из того же
    /// decoder-а, а не из второго каталога типов или выдуманного базового размера.
    pub(crate) fn remove_applied_state(&mut self, key: StateKey) -> Option<StateData> {
        nebokrai_zone::skills::state::remove_applied_state(&mut self.state_storage, key)
    }

























    pub(crate) fn replace_spider_web_state(
        &mut self,
        state: SpiderWebState,
    ) -> Option<SpiderWebState> {
        nebokrai_zone::skills::state::replace_spider_web_state(&mut self.state_storage, state)
    }






    pub(crate) fn energy_holding_states(&self) -> impl Iterator<Item = &EnergyHoldingState> {
        nebokrai_zone::skills::state::energy_holding_states(&self.state_storage)
    }

    pub(crate) fn take_spider_web_state(&mut self) -> Option<SpiderWebState> {
        nebokrai_zone::skills::state::take_spider_web_state(&mut self.state_storage)
    }











    pub(crate) fn pillar_state(&self) -> Option<PillarState> {
        nebokrai_zone::skills::state::pillar_state(&self.state_storage)
    }



    pub(crate) fn curable_state_ids(&self) -> Vec<u32> {
        nebokrai_zone::skills::state::curable_state_ids(&self.state_storage)
    }

    pub(crate) fn blind_state_order(&self) -> Vec<u32> {
        nebokrai_zone::skills::state::blind_state_order(&self.state_storage)
    }

    pub(crate) fn blind_state_instances(&self) -> Vec<(StateKey, u32)> {
        nebokrai_zone::skills::state::blind_state_instances(&self.state_storage)
    }


    pub(crate) fn battle_fairy_attribute_states(&self) -> impl Iterator<Item = &BattleFairyAttributeState> {
        nebokrai_zone::skills::state::battle_fairy_attribute_states(&self.state_storage)
    }


    pub(crate) fn take_expired_battle_fairy_attribute_state(
        &mut self,
        key: StateKey,
        now_ms: u32,
    ) -> Option<BattleFairyAttributeState> {
        nebokrai_zone::skills::state::take_expired_battle_fairy_attribute_state(
            &mut self.state_storage,
            key,
            now_ms,
        )
    }












    pub(crate) fn script_states(&self) -> impl Iterator<Item = &ScriptMoveState> {
        nebokrai_zone::skills::state::script_states(&self.state_storage)
    }





    pub(crate) fn tian_shen_xia_fan_state(&self) -> Option<TianShenXiaFanState> {
        nebokrai_zone::skills::state::tian_shen_xia_fan_state(&self.state_storage)
    }





    pub(crate) fn wangsheng_state(&self) -> Option<WangshengState> {
        nebokrai_zone::skills::state::wangsheng_state(&self.state_storage)
    }








    pub(crate) fn get_undead_state(&self, state_id: u32) -> u32 {
        nebokrai_zone::skills::state::get_undead_state(&self.state_storage, state_id)
    }




    pub(crate) fn undead_state_tick(
        &mut self,
        key: StateKey,
        now_milliseconds: impl FnMut() -> u32,
    ) -> (bool, Option<(u32, u32)>) {
        nebokrai_zone::skills::state::undead_state_tick(
            &mut self.state_storage,
            key,
            now_milliseconds,
        )
    }


    pub(crate) fn get_extended_state(&self, kind: ExtendedStateKind, state_id: u32) -> u32 {
        nebokrai_zone::skills::state::get_extended_state(&self.state_storage, kind, state_id)
    }

    pub(crate) fn extended_states(&self) -> impl Iterator<Item = &ExtendedState> {
        nebokrai_zone::skills::state::extended_states(&self.state_storage)
    }



    pub(crate) fn extended_state_tick(
        &mut self,
        key: StateKey,
        now_milliseconds: impl FnMut() -> u32,
    ) -> (bool, Option<(u32, u32)>) {
        nebokrai_zone::skills::state::extended_state_tick(
            &mut self.state_storage,
            key,
            now_milliseconds,
        )
    }


    pub(crate) fn get_change_body_state(&self, state_id: u32) -> u32 {
        nebokrai_zone::skills::state::get_change_body_state(&self.state_storage, state_id)
    }


    pub(crate) fn active_change_body_state(&self) -> Option<&ChangeBodyState> {
        nebokrai_zone::skills::state::active_change_body_state(&self.state_storage)
    }

    pub(crate) fn first_change_body_state_id(&self) -> Option<u32> {
        nebokrai_zone::skills::state::first_change_body_state_id(&self.state_storage)
    }




    pub(crate) fn skill(&self, skill_id: u32, factory: &CSkillFactory) -> Option<&MoveShapeSkill> {
        self.skills.skill(skill_id, factory)
    }

    fn skill_mut(&mut self, skill_id: u32, factory: &CSkillFactory) -> Option<&mut MoveShapeSkill> {
        self.skills.skill_mut(skill_id, factory)
    }

    pub(crate) fn skill_slot(&self, skill_id: u32, factory: &CSkillFactory) -> Option<SkillSlot> {
        self.skills.skill_slot(skill_id, factory)
    }

    /// Прямой native-обход берёт текущий индекс категории, без QuerySkillType.
    /// Возвращённый ключ сохраняет идентичность через последующие callbacks.
    pub(crate) fn skill_slot_at(&self, category: SkillCategory, index: usize) -> Option<SkillSlot> {
        self.skills.skill_slot_at(category, index)
    }

    pub(crate) fn skill_count_in_category(&self, category: SkillCategory) -> usize {
        self.skills.skill_count_in_category(category)
    }

    pub(crate) fn skill_at(&self, slot: SkillSlot) -> Option<&MoveShapeSkill> {
        self.skills.skill_at(slot)
    }

    pub(crate) fn skill_at_mut(&mut self, slot: SkillSlot) -> Option<&mut MoveShapeSkill> {
        self.skills.skill_at_mut(slot)
    }

    pub(crate) fn skill_lifecycle(
        &self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&SkillLifecycle> {
        Some(self.skill(skill_id, factory)?.lifecycle())
    }

    pub(crate) fn skill_lifecycle_mut(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&mut SkillLifecycle> {
        Some(self.skill_mut(skill_id, factory)?.lifecycle_mut())
    }

    pub(crate) fn skill_visual_effect_mut(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&mut SkillVisualEffect> {
        self.skills.skill_visual_effect_mut(skill_id, factory)
    }

    pub(crate) fn replace_skill_visual_effect(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
        effect: SkillVisualEffect,
    ) -> bool {
        self.skills.replace_skill_visual_effect(skill_id, factory, effect)
    }

    pub(crate) fn finish_skill_base(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
        termination: SkillTermination,
    ) -> bool {
        let Some(skill) = self.skill_mut(skill_id, factory) else {
            return false;
        };
        skill.finish_base(termination);
        true
    }

    pub(crate) fn monster_skill_execution(
        &self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&super::monster::MonsterSkillExecution> {
        self.skill(skill_id, factory)?.monster_payload()
    }

    pub(crate) fn monster_skill_execution_mut(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&mut super::monster::MonsterSkillExecution> {
        self.skill_mut(skill_id, factory)?.monster_payload_mut()
    }

    pub(crate) fn install_monster_execution(
        &mut self,
        execution: super::monster::MonsterSkillExecution,
        factory: &CSkillFactory,
    ) -> bool {
        let skill_id = execution.kernel.dispatch().skill_id;
        let Some(skill) = self.skill_mut(skill_id, factory) else { return false };
        skill.install_monster_execution(execution)
    }

    pub(crate) fn clear_monster_execution(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> bool {
        let Some(skill) = self.skill_mut(skill_id, factory) else { return false };
        skill.clear_monster_execution()
    }

    pub(crate) fn player_execution(
        &self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&PlayerSkillExecution> {
        self.skill(skill_id, factory)?.player_execution()
    }

    pub(crate) fn player_execution_mut(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&mut PlayerSkillExecution> {
        self.skill_mut(skill_id, factory)?.player_execution_mut()
    }

    pub(crate) fn install_player_execution(
        &mut self,
        execution: PlayerSkillExecution,
        factory: &CSkillFactory,
    ) -> bool {
        let skill_id = execution.kernel().dispatch().skill_id();
        let Some(skill) = self.skill_mut(skill_id, factory) else { return false };
        skill.install_player_execution(execution)
    }

    pub(crate) fn battle_fairy_execution(
        &self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&BattleFairyExecution> {
        self.skill(skill_id, factory)?.battle_fairy_execution_state()
    }

    pub(crate) fn skill_last_used_ms(&self, skill_id: u32, factory: &CSkillFactory) -> u32 {
        self.skills.skill_last_used_ms(skill_id, factory)
    }

    pub(crate) fn mark_skill_used(&mut self, skill_id: u32, now_ms: u32, factory: &CSkillFactory) {
        self.skills.mark_skill_used(skill_id, now_ms, factory);
    }

    /// Удаление реестра сохраняет неразрешённый current ID. Полный concrete
    /// End перед удалением ещё требует подключения lifecycle владельца.
    pub(crate) fn clear_skills(&mut self, factory: &CSkillFactory) {
        self.skills.clear_skills(factory);
    }

    /// `CSkillFactory::QuerySkill(SKILL_BASE_DEFENSE, 1)` создавал
    /// `CFightDefense` отдельной ветвью даже без reloadable properties.
    /// Reloadable properties не могут отменить intrinsic defense или
    /// изменить его категорию; имя читается только при обращении к экземпляру.
    /// Категория и concrete owner выбираются реестром Zone, execution/retained
    /// достраивает запись этого владельца.
    pub(crate) fn add_base_defense_skill(&mut self, _factory: &CSkillFactory) {
        self.skills.add_base_defense_skill(|owner| {
            MoveShapeSkill::registered(SKILL_BASE_DEFENSE, 1, owner)
        });
    }

    pub(crate) fn set_item_skill_position(&mut self, skill_id: u32, position: i32, factory: &CSkillFactory) -> bool {
        self.skills.set_item_skill_position(skill_id, position, factory)
    }

    /// Выбранный ID независимо от наличия зарегистрированного навыка.
    pub(crate) const fn current_skill_id(&self) -> Option<u32> {
        self.inner.skills.current_skill_id()
    }

    /// Проекция GetCurrentSkill в реестр; execution и End остаются у skill-owner.
    pub(crate) fn current_skill(&self, factory: &CSkillFactory) -> Option<&MoveShapeSkill> {
        self.skills.current_skill(factory)
    }

    /// GetDefaultAttackSkillID (0x004CE240): порядок категорий важнее порядка ID.
    pub(crate) fn default_attack_skill_id(&self) -> u32 {
        self.skills.default_attack_skill_id()
    }

    /// Typed boundary для snapshot/skill caller-а. Полное semantic действие
    /// `SetCurrentSkill` (завершение прежнего concrete skill) не подменяется
    /// записью ID и остаётся у соответствующего owner-а.
    pub(crate) const fn set_current_skill_id(&mut self, skill_id: Option<u32>) {
        self.inner.skills.set_current_skill_id(skill_id);
    }

    /// Exact `SetItemSkill`: native owner только добавляет ID в ordered vector
    /// непосредственно перед передачей item-skill в `CPlayerAI`.
    pub(crate) fn set_item_skill(&mut self, skill_id: u32) {
        self.skills.set_item_skill(skill_id);
    }

    pub(crate) const fn is_moveable(&self) -> bool {
        self.inner.is_moveable()
    }

    pub(crate) const fn moveable_count(&self) -> i32 {
        self.inner.moveable_count()
    }

    /// Счётчик запретов движения идёт общей операцией Zone moveshape.
    pub(crate) const fn set_moveable(&mut self, moveable: bool) {
        self.inner.set_moveable(moveable);
    }

    /// AddSkill (0x004D1C70): ненулевой уровень не понижается, повышение
    /// удаляет первое совпадение и добавляет новый экземпляр в хвост.
    /// Нулевой уровень прежнего экземпляра допускает повторный ID.
    pub(crate) fn add_skill(&mut self, skill_id: u32, level: i32, factory: &CSkillFactory) -> bool {
        self.skills.add_skill(skill_id, level, factory, |owner| {
            MoveShapeSkill::registered(skill_id, level, owner)
        })
    }

    pub(crate) fn insert_new_skill(&mut self, skill_id: u32, level: i32) -> bool {
        self.skills.insert_new_skill(skill_id, |owner| {
            MoveShapeSkill::registered(skill_id, level, owner)
        })
    }

    /// DelSkill (0x004CF320) удаляет только первый найденный экземпляр.
    /// До category lookup обрабатывается разрешённый current, даже если
    /// удаляется другой ID. UNKNOWN отвергается до этого, а ID 0 — после.
    pub(crate) fn delete_skill(&mut self, skill_id: u32, factory: &CSkillFactory) -> bool {
        self.skills.delete_skill(skill_id, factory)
    }

    pub(crate) fn delete_skill_in_category(&mut self, skill_id: u32, category: SkillCategory) {
        self.skills.delete_skill_in_category(skill_id, category);
    }

    pub(crate) fn set_pos_xy(
        &mut self,
        region: Option<&mut CRegion>,
        x: f32,
        y: f32,
        facts: MoveShapePositionFacts,
    ) -> Result<(), MoveShapePositionBlock> {
        set_pos_xy_core(region, &mut self.shape, x, y, facts)
    }

    pub(crate) const fn is_died(current_hit_points: u32) -> bool {
        nebokrai_zone::regions::moveshape::is_died(current_hit_points)
    }

    /// Геометрия ближайшей клетки footprint принадлежит общей операции Zone
    /// moveshape.
    pub(crate) fn nearest_figure_attack_point(
        tile_x: i32,
        tile_y: i32,
        figure: ShapeFigure,
        attacker_x: i32,
        attacker_y: i32,
    ) -> (i32, i32) {
        nebokrai_zone::regions::moveshape::nearest_figure_attack_point(
            tile_x,
            tile_y,
            figure,
            attacker_x,
            attacker_y,
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "literal ForceMove сохраняет исходные аргументы и две достигнутые owner-границы"
    )]
    pub(crate) fn force_move(
        &mut self,
        server_region: Option<&mut CServerRegion>,
        destination_x: i32,
        destination_y: i32,
        duration_ms: u32,
        facts: MoveShapePositionFacts,
        around: &GameServerAroundRuntime<'_>,
    ) -> Result<bool, MoveShapeCommandBlock> {
        let Some(server_region) = server_region else {
            return Ok(false);
        };
        let (destination, message) = self.force_move_message(
            server_region, destination_x, destination_y, duration_ms,
        )?;
        let _ = message
            .send_to_around(Some(&*server_region), &self.shape, None, around)
            .map_err(MoveShapeCommandBlock::Coordinate)?;

        server_region
            .set_move_shape_tile_position(&mut self.shape, destination.x, destination.y, facts)
            .map_err(MoveShapeCommandBlock::Position)?;
        Ok(true)
    }

    /// Общий BF604 и clamp принадлежат Zone-основе `regions/moveshape`;
    /// виртуальный SetTileXY и последующий AI Stand принадлежат конкретному
    /// владельцу движения.
    pub(crate) fn force_move_message(
        &self,
        server_region: &CServerRegion,
        destination_x: i32,
        destination_y: i32,
        duration_ms: u32,
    ) -> Result<(ShapeAreaCoordinates, CMessage), MoveShapeCommandBlock> {
        let outcome = force_move_wire(
            &self.shape,
            server_region,
            destination_x,
            destination_y,
            duration_ms,
        )
        .map_err(MoveShapeCommandBlock::Coordinate)?;
        Ok((outcome.destination, outcome.message))
    }

    pub(crate) fn on_move(
        &mut self,
        server_region: Option<&mut CServerRegion>,
        destination_x: i32,
        destination_y: i32,
        run: i32,
        facts: MoveShapePositionFacts,
        around: &GameServerAroundRuntime<'_>,
    ) -> Result<(), MoveShapeCommandBlock> {
        let message = on_move_wire(&mut self.shape, destination_x, destination_y, run)
            .map_err(MoveShapeCommandBlock::Coordinate)?;
        let _ = message
            .send_to_around(server_region.as_deref(), &self.shape, None, around)
            .map_err(MoveShapeCommandBlock::Coordinate)?;

        if let Some(server_region) = server_region {
            server_region
                .set_move_shape_tile_position(&mut self.shape, destination_x, destination_y, facts)
                .map_err(MoveShapeCommandBlock::Position)
        } else {
            set_pos_xy_core(
                None,
                &mut self.shape,
                (destination_x as f32) + 0.5,
                (destination_y as f32) + 0.5,
                facts,
            )
            .map_err(MoveShapeCommandBlock::DetachedPosition)
        }
    }

    pub(crate) fn on_set_position(
        &mut self,
        server_region: Option<&mut CServerRegion>,
        destination_x: i32,
        destination_y: i32,
        facts: MoveShapePositionFacts,
        around: &GameServerAroundRuntime<'_>,
    ) -> Result<bool, MoveShapeCommandBlock> {
        let Some(server_region) = server_region else {
            return Ok(false);
        };
        let MoveShapeSetPositionOutcome::Accepted(message) =
            on_set_position_wire(&self.shape, server_region, destination_x, destination_y)
                .map_err(MoveShapeCommandBlock::RegionCell)?
        else {
            return Ok(false);
        };
        let _ = message
            .send_to_around(Some(&*server_region), &self.shape, None, around)
            .map_err(MoveShapeCommandBlock::Coordinate)?;

        server_region
            .set_move_shape_tile_position(&mut self.shape, destination_x, destination_y, facts)
            .map_err(MoveShapeCommandBlock::Position)?;
        Ok(true)
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h

// ============================================================================
// GetDefaultAttackSkillID материализован в default_attack_skill_id.

// ============================================================================

// ============================================================================
// CMoveShape::ClearAllStates (0x004CF090, moveshape.cpp:2132)
// реализован общим states/state.rs::clear_move_shape_states; End и последующий
// destructor-only проход сохраняют позиции, death-фильтры и UpdateProperty.

// ============================================================================
// ============================================================================
// ============================================================================
// CMoveShape::prison_check (0x004D2360, moveshape.cpp:3404)
// реализован CGame::check_move_shape_prison: fresh victim-region, tamed master,
// PrisonConf::operator[], GS0126 и существующий ChangeRegion после очистки.


// COMPONENT_VARIANT_END: GameServer
