//! Реализованная часть `CMoveShape` исторического GameServer.
//! Постоянные данные CArchery/CBaseMagic/CFireBolt принадлежат экземпляру
//! навыка: att_time обнуляется конструктором, но не Begin/End или удалением
//! исполнения команды. Игрок и монстр используют одно и то же хранение.
//! EnergyBolt/SnakeBolt/ZombieClaw сохраняют там область первого успешного
//! Begin; End очищает путь и параметры полёта, не заменяя эту область.
//! UpdateProperty (0x004CFB60, moveshape.cpp:93) реализован общим живым
//! dispatcher-ом states/state.rs. Здесь хранится одна PDB-структура
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
//! считает непустые позиции общей арены, затем пишет ID/+30/+38 и Team-name
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
//! Его BF604 строится общим способом; CGame сохраняет виртуальный SetTileXY
//! игрока с отменой захвата до свежего AI Stand. Монстр и постройка используют
//! пространственную базу напрямую, не создавая второго способа переноса игрока.
//! `SetKilledMeAttackInfo` (0x004CCE50) сохраняет данные убийцы в общей
//! базе после пакета смерти 0xBF60B. Единственный KillingAttackIdentity содержит
//! только потребляемую OnDied-проекцию: тип, ID и guild ID атакующего.
//! Это не восстановление полного native-layout: остальные скопированные
//! модификаторы и флаги, пока не имеющие перенесённых потребителей, остаются
//! в RAW ниже. Ни конкретная форма, ни отложенный удар не дублируют эту запись.
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

mod state_storage;
pub(crate) use state_storage::{AppliedState, AppliedStateEntries, StateBatch, StateData, StateKey};
use state_storage::StateSerialization;

use std::ops::{Deref, DerefMut};
use slotmap::{SlotMap, new_key_type};

use super::ai::baseai::CBaseAI;
use super::chbystate::{CHANGE_BODY_STATE_ID, ChangeBodyState};
use super::exstate::{ExtendedState, ExtendedStateKind};
use super::legacycodec::{LegacyReader, LegacyWriter};
use super::particularstate::ParticularState;
use super::region::{CRegion, RegionCellAccessBlock};
use super::ridestate::{RIDE_STATE_ID, RideState};
use super::restorestate::{ConsumableRestoreIntervals, ConsumableRestoreState};
use super::restorehpstate::RestoreHpState;
use super::restorempstate::RestoreMpState;
use super::scriptstate::ScriptMoveState;
use super::serverregion::{CServerRegion, RegionMembershipBlock};
use super::skills::kernel::{BattleFairyExecution, PlayerSkillExecution, SkillLifecycle, SkillTermination};
use super::states::visualeffect::SkillVisualEffect;
use super::teamstate::CTeamState;
use super::shape::{
    CShape, SHAPE_CHANGE_AREA, SHAPE_CHANGE_NONE, ShapeAreaCoordinates, ShapeBlockError,
    ShapeCoordinateBlock, ShapeFigure, ShapeIdentity, ShapePositionDispatch, ShapeResolver,
};
use crate::gameserver::appserver::skills::agilitystate::PersistentAgilityFamilyState;
use crate::gameserver::appserver::skills::immediatestate::is_immediate_state_skill;
use crate::gameserver::appserver::skills::agilitystate2::AgilityState2;
use crate::gameserver::appserver::skills::callositystate::CallosityFamilyState;
use crate::gameserver::appserver::skills::curestate::{CureState, CURE_STATE_BYTES};
use crate::gameserver::appserver::skills::daubpoisonstate::DaubPoisonState;
use crate::gameserver::appserver::skills::enlargefullmissstate::EnlargeFullMissState;
use crate::gameserver::appserver::skills::enlargemaxhpstate::EnlargeMaxHpState;
use crate::gameserver::appserver::skills::enlargemaxmpstate::EnlargeMaxMpState;
use crate::gameserver::appserver::skills::heartenstate::HeartenState;
use crate::gameserver::appserver::skills::healstate::HealState;
use crate::gameserver::appserver::skills::furystate::FuryState;
use crate::gameserver::appserver::skills::ragebreakstate::RageBreakState;
use crate::gameserver::appserver::skills::rushstate::RushState;
use crate::gameserver::appserver::skills::rushstate2::Rush2State;
use crate::gameserver::appserver::skills::roarstate::{
    RoarState,
};
use crate::gameserver::appserver::skills::energyholdingstate::{
    EnergyHoldingState,
};
use crate::gameserver::appserver::skills::soulcollectstate::SoulCollectState;
use crate::gameserver::appserver::skills::lifeshieldstate::LIFE_SHIELD_STATE_BYTES;
use crate::gameserver::appserver::skills::machineshieldstate::MACHINE_SHIELD_STATE_BYTES;
use crate::gameserver::appserver::skills::manashieldstate::MANA_SHIELD_STATE_BYTES;
use crate::gameserver::appserver::skills::promotionstate::PROMOTION_STATE_BYTES;
use crate::gameserver::appserver::skills::knockoutstate::KnockOutState;
use crate::gameserver::appserver::skills::boalockstate::BoaLockState;
use crate::gameserver::appserver::skills::blindstate::BlindState;
use crate::gameserver::appserver::skills::knightcutstate::KnightCutState;
use crate::gameserver::appserver::skills::originstate::OriginState;
use crate::gameserver::appserver::skills::pillarstate::{
    PillarState,
};
use crate::gameserver::appserver::skills::poisonarrowstate::PoisonArrowState;
use crate::gameserver::appserver::skills::poisonfogstate::PoisonFogState;
use crate::gameserver::appserver::skills::meteorarrowstate::MeteorArrowState;
use crate::gameserver::appserver::skills::spiderpoisonstate::SpiderPoisonState;
use crate::gameserver::appserver::skills::spriteburnstate::SpriteBurnState;
use crate::gameserver::appserver::skills::spiderwebstate::{
    SPIDER_WEB_STATE_BYTES, SpiderWebState,
};
use crate::gameserver::appserver::skills::sealstate::SealState;
use crate::gameserver::appserver::skills::swordshipstate::SwordshipState;
use crate::gameserver::appserver::skills::strikestate::StrikeState;
use crate::gameserver::appserver::skills::bloodlossstate::BloodLossState;
use crate::gameserver::appserver::skills::kerosenestate::KeroseneState;
use crate::gameserver::appserver::skills::leafcutstate::LeafCutState;
use crate::gameserver::appserver::skills::leafcutstate2::LeafCutState2;
use crate::gameserver::appserver::skills::leafcutstate3::LeafCutState3;
use crate::gameserver::appserver::skills::battlefairyattributestate::{BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES, BattleFairyAttributeState};
use crate::gameserver::appserver::skills::bossbluefurystate::{
    BossBlueFuryState, BOSS_BLUE_FURY_STATE_BYTES,
};
use crate::gameserver::appserver::skills::bossbluequakestate::{
    BossBlueQuakeState, BOSS_BLUE_QUAKE_STATE_BYTES,
};
use crate::gameserver::appserver::skills::skillfactory::{CSkillFactory, SkillCategory, SkillOwner};
use crate::gameserver::appserver::skills::statefactory::{decode_state_record_into_cache, known_state_record_offsets, known_state_record_spans};
use crate::gameserver::appserver::skills::shieldstate::DefenseShieldState;
use crate::gameserver::appserver::skills::taijistate::TaiJiState;
use crate::gameserver::appserver::skills::tianshenxiafanstate::{
    TianShenXiaFanState,
};
use crate::gameserver::appserver::skills::wangshengstate::{
    WangshengState,
};
use crate::gameserver::appserver::skills::weakstate::WeakState;
use crate::gameserver::appserver::skills::wuxingstate::WuXingState;
use crate::gameserver::appserver::skills::godblessstate::{
    GodBlessState,
};
use crate::gameserver::appserver::states::automaticrestore::{
    AutomaticRestoreState, AUTOMATIC_RESTORE_STATE_BYTES,
};
use crate::nets::netserver::message::{CMessage, GameServerAroundRuntime};
use crate::public::tools::get_line_direction;

const NPC_TYPE: i32 = 500;
const SET_POSITION_MESSAGE: i32 = 0xBF603;
const FORCE_MOVE_MESSAGE: i32 = 0xBF604;
const MOVE_MESSAGE: i32 = 0xBF605;
pub(crate) const SKILL_BASE_DEFENSE: u32 = 10;
const SKILL_NOT_DISAPPEAR_AFTER_DEAD: u32 = 56;
const SKILL_USAGE_CONST: u32 = 20_010;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const UNDEAD_STATE_ID: u32 = 0x38;
const UNDEAD_STATE_PARAMETER_BYTES: usize = 72;

#[derive(Clone, Debug, Eq, PartialEq)]
enum RegisteredSkillExecution {
    Inactive(SkillLifecycle),
    Player(PlayerSkillExecution),
    BattleFairy(BattleFairyExecution),
    Monster(super::monster::MonsterSkillExecution),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RegisteredSkillDispatch {
    Player(super::player::PlayerSkillDispatch),
    BattleFairy(super::player::BattleFairySkillDispatch),
}

impl RegisteredSkillExecution {
    fn lifecycle(&self) -> &SkillLifecycle {
        match self {
            Self::Inactive(lifecycle) => lifecycle,
            Self::Player(execution) => execution.lifecycle(),
            Self::BattleFairy(execution) => execution.lifecycle(),
            Self::Monster(execution) => execution.kernel.lifecycle(),
        }
    }

    fn lifecycle_mut(&mut self) -> &mut SkillLifecycle {
        match self {
            Self::Inactive(lifecycle) => lifecycle,
            Self::Player(execution) => execution.lifecycle_mut(),
            Self::BattleFairy(execution) => execution.lifecycle_mut(),
            Self::Monster(execution) => execution.kernel.lifecycle_mut(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum SkillRetainedData {
    None,
    BaseProjectile(super::skills::baseprojectilecast::BaseProjectileProgress),
    PathProjectile(super::skills::energybolt::PathProjectileProgress),
}

impl SkillRetainedData {
    fn for_owner(owner: SkillOwner) -> Self {
        match owner {
            SkillOwner::CArchery | SkillOwner::CBaseMagic | SkillOwner::CFireBolt =>
                Self::BaseProjectile(Default::default()),
            SkillOwner::CEnergyBolt | SkillOwner::CSnakeBolt | SkillOwner::CZombieClaw =>
                Self::PathProjectile(Default::default()),
            _ => Self::None,
        }
    }
}

/// Достигнутая common-проекция `CSkill`: identity, level и concrete owner.
/// Алгоритмы concrete attack/defense/state/summon остаются у skill owners;
/// исполнение, его ресурсы и reuse принадлежат каждому экземпляру.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct MoveShapeSkill {
    id: u32,
    level: i32,
    owner: SkillOwner,
    item_position: i32,
    execution: RegisteredSkillExecution,
    retained_data: SkillRetainedData,
    current_visual_effect: Option<SkillVisualEffect>,
    last_used_ms: u32,
}

new_key_type! {
    struct SkillEntity;
}

/// Адрес конкретного экземпляра в одной категории данного CMoveShape.
/// После удаления ключ не разрешается в новую запись с тем же ID или индексом.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SkillSlot {
    category: SkillCategory,
    entity: SkillEntity,
}

#[derive(Debug, Default)]
struct SkillCollection {
    instances: SlotMap<SkillEntity, MoveShapeSkill>,
    order: Vec<SkillEntity>,
}

impl PartialEq for SkillCollection {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl Eq for SkillCollection {}

impl Drop for SkillCollection {
    fn drop(&mut self) {
        self.clear();
    }
}

impl SkillCollection {
    fn iter(&self) -> impl ExactSizeIterator<Item = &MoveShapeSkill> {
        self.order.iter().map(|entity| {
            self.instances.get(*entity)
                .expect("порядок категории содержит только живые экземпляры навыков")
        })
    }

    fn push(&mut self, skill: MoveShapeSkill) {
        self.order.push(self.instances.insert(skill));
    }

    fn remove(&mut self, index: usize) -> MoveShapeSkill {
        let entity = self.order.remove(index);
        self.instances.remove(entity)
            .expect("удаляемый индекс категории принадлежит живому экземпляру навыка")
    }

    fn clear(&mut self) {
        for entity in self.order.drain(..) {
            drop(self.instances.remove(entity));
        }
    }
}

/// Достигнутый wire/lifecycle owner `CNotDisappearAfterDead`.
/// Serialize (0x005d64f0) сохраняет остаток в живом keeptime без смены старта;
/// AI (0x005d7c80) использует строгие абсолютные wrapping сроки.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UndeadState {
    state_id: u32,
    state_type: u16,
    keep_time_ms: u32,
    started_ms: u32,
    last_item_tick_ms: u32,
    // ClearAllStates сравнивает сохранённый байт строго с 1, не с нулём.
    pub(crate) disappear_after_dead: u8,
    pub(crate) percentage: bool,
    pub(crate) maximum_hp: i16,
    pub(crate) maximum_mp: i16,
    pub(crate) minimum_attack: i16,
    pub(crate) maximum_attack: i16,
    pub(crate) element_modify: i16,
    pub(crate) defense: i16,
    pub(crate) element_resistance: i16,
    pub(crate) blast_attack: i16,
    pub(crate) blast_element_attack: i16,
    pub(crate) strength: i32,
    pub(crate) dexterity: i32,
    pub(crate) constitution: i32,
    pub(crate) intelligence: i32,
    pub(crate) cch: i16,
    pub(crate) full_miss: i16,
    pub(crate) attack_avoid: i16,
    pub(crate) element_avoid: i16,
    pub(crate) hit: i16,
    pub(crate) dodge: i16,
    item_index: u32,
    item_amount: u32,
    frequency_ms: u32,
    serialized_offset: Option<usize>,
}

impl UndeadState {
    pub(crate) const SERIALIZED_BYTES: usize = 4 + UNDEAD_STATE_PARAMETER_BYTES;

    pub(crate) const fn state_id(&self) -> u32 {
        self.state_id
    }

    pub(crate) const fn state_type(&self) -> u16 {
        self.state_type
    }

    pub(crate) const fn keep_time_ms(&self) -> u32 {
        self.keep_time_ms
    }

    pub(crate) const fn started_ms(&self) -> u32 {
        self.started_ms
    }

    pub(crate) fn begin_primary_at(&mut self, now_ms: u32) {
        self.started_ms = now_ms;
    }

    pub(crate) fn client_state_time(&self, now: &mut dyn FnMut() -> u32) -> u32 {
        crate::gameserver::appserver::states::state::extended_client_time(
            self.started_ms, self.keep_time_ms, now,
        )
    }

    pub(crate) fn commit_serialized_time(&mut self, remaining: u32) {
        self.keep_time_ms = remaining;
    }

    pub(crate) fn from_factory(state_id: u32, factory: &CSkillFactory) -> Option<Self> {
        let properties =
            factory.query_skill_base_properties(SKILL_NOT_DISAPPEAR_AFTER_DEAD, state_id as i32)?;
        let p = |usage| properties.query_property(usage);
        Some(Self {
            state_id,
            state_type: p(SKILL_USAGE_CONST) as u16,
            keep_time_ms: p(SKILL_USAGE_STATE_PERSIST_TIME),
            started_ms: 0,
            last_item_tick_ms: 0,
            disappear_after_dead: u8::from(p(80_001) != 0),
            percentage: p(80_002) != 0,
            maximum_hp: p(118) as i16,
            maximum_mp: p(119) as i16,
            minimum_attack: p(116) as i16,
            maximum_attack: p(117) as i16,
            element_modify: p(115) as i16,
            defense: p(109) as i16,
            element_resistance: p(112) as i16,
            blast_attack: p(125) as i16,
            blast_element_attack: p(126) as i16,
            strength: p(101) as i32,
            dexterity: p(102) as i32,
            constitution: p(103) as i32,
            intelligence: p(104) as i32,
            cch: p(108) as i16,
            full_miss: p(127) as i16,
            attack_avoid: p(128) as i16,
            element_avoid: p(129) as i16,
            hit: p(20_001) as i16,
            dodge: p(110) as i16,
            item_index: p(50_001),
            item_amount: p(50_002),
            frequency_ms: p(6_001),
            serialized_offset: None,
        })
    }

    pub(crate) fn decode_at(payload: &[u8], offset: usize, now_ms: u32) -> Option<Self> {
        if read_u32(payload, offset) != Some(UNDEAD_STATE_ID) {
            return None;
        }
        let base = offset.checked_add(4)?;
        let _ = payload.get(base..base.checked_add(UNDEAD_STATE_PARAMETER_BYTES)?)?;
        let state_id = read_u32(payload, base + 4)?;
        Some(Self {
                state_id,
                state_type: read_u16(payload, base).unwrap_or_default(),
                keep_time_ms: read_u32(payload, base + 8).unwrap_or_default(),
                started_ms: now_ms,
                last_item_tick_ms: now_ms,
                disappear_after_dead: payload[base + 12],
                percentage: payload[base + 13] != 0,
                maximum_hp: read_i16(payload, base + 14).unwrap_or_default(),
                maximum_mp: read_i16(payload, base + 16).unwrap_or_default(),
                minimum_attack: read_i16(payload, base + 18).unwrap_or_default(),
                maximum_attack: read_i16(payload, base + 20).unwrap_or_default(),
                element_modify: read_i16(payload, base + 22).unwrap_or_default(),
                defense: read_i16(payload, base + 24).unwrap_or_default(),
                element_resistance: read_i16(payload, base + 26).unwrap_or_default(),
                blast_attack: read_i16(payload, base + 28).unwrap_or_default(),
                blast_element_attack: read_i16(payload, base + 30).unwrap_or_default(),
                strength: read_i32(payload, base + 32).unwrap_or_default(),
                dexterity: read_i32(payload, base + 36).unwrap_or_default(),
                constitution: read_i32(payload, base + 40).unwrap_or_default(),
                intelligence: read_i32(payload, base + 44).unwrap_or_default(),
                cch: read_i16(payload, base + 48).unwrap_or_default(),
                full_miss: read_i16(payload, base + 50).unwrap_or_default(),
                attack_avoid: read_i16(payload, base + 52).unwrap_or_default(),
                element_avoid: read_i16(payload, base + 54).unwrap_or_default(),
                hit: read_i16(payload, base + 56).unwrap_or_default(),
                dodge: read_i16(payload, base + 58).unwrap_or_default(),
                item_index: read_u32(payload, base + 60).unwrap_or_default(),
                item_amount: read_u32(payload, base + 64).unwrap_or_default(),
                frequency_ms: read_u32(payload, base + 68).unwrap_or_default(),
                serialized_offset: Some(offset),
        })
    }

    pub(crate) fn encoded_for_install(&self) -> [u8; Self::SERIALIZED_BYTES] {
        let mut record = [0; Self::SERIALIZED_BYTES];
        self.update_serialized_record(&mut record, 0, self.keep_time_ms);
        record
    }

    pub(crate) fn update_serialized_record(
        &self, payload: &mut [u8], offset: usize, remaining: u32,
    ) {
        if offset.checked_add(Self::SERIALIZED_BYTES).is_none_or(|end| end > payload.len()) {
            return;
        }
        let base = offset + 4;
        write_u32(payload, offset, UNDEAD_STATE_ID);
        write_u16(payload, base, self.state_type);
        write_u32(payload, base + 4, self.state_id);
        write_u32(payload, base + 8, remaining);
        payload[base + 12] = self.disappear_after_dead;
        // Native Serialize копирует исходный BOOL-байт, а не нормализует его.
        if (payload[base + 13] != 0) != self.percentage {
            payload[base + 13] = u8::from(self.percentage);
        }
        for (position, value) in [
            (14, self.maximum_hp),
            (16, self.maximum_mp),
            (18, self.minimum_attack),
            (20, self.maximum_attack),
            (22, self.element_modify),
            (24, self.defense),
            (26, self.element_resistance),
            (28, self.blast_attack),
            (30, self.blast_element_attack),
            (48, self.cch),
            (50, self.full_miss),
            (52, self.attack_avoid),
            (54, self.element_avoid),
            (56, self.hit),
            (58, self.dodge),
        ] {
            write_i16(payload, base + position, value);
        }
        for (position, value) in [
            (32, self.strength),
            (36, self.dexterity),
            (40, self.constitution),
            (44, self.intelligence),
        ] {
            write_i32(payload, base + position, value);
        }
        write_u32(payload, base + 60, self.item_index);
        write_u32(payload, base + 64, self.item_amount);
        write_u32(payload, base + 68, self.frequency_ms);
    }

    fn serialized_span(&self) -> Option<(usize, usize)> {
        self.serialized_offset
            .map(|offset| (offset, 4 + UNDEAD_STATE_PARAMETER_BYTES))
    }

    fn shift_serialized_offset_for_insert(&mut self, inserted_offset: usize, amount: usize) {
        if let Some(offset) = &mut self.serialized_offset {
            if *offset >= inserted_offset {
                *offset += amount;
            }
        }
    }

    fn shift_serialized_offset_after(&mut self, removed_offset: usize, amount: usize) {
        if self
            .serialized_offset
            .is_some_and(|offset| removed_offset < offset)
        {
            self.serialized_offset = self.serialized_offset.map(|offset| offset - amount);
        }
    }

    fn expired(&self, now_ms: u32) -> bool {
        self.keep_time_ms != 0 && self.started_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    fn item_due(&mut self, now_ms: u32) -> bool {
        if self.last_item_tick_ms == 0 {
            self.last_item_tick_ms = self.started_ms;
        }
        self.frequency_ms != 0
            && self.item_index != 0
            && self.item_amount != 0
            && self.last_item_tick_ms.wrapping_add(self.frequency_ms) < now_ms
    }
}


impl MoveShapeSkill {
    pub(crate) const fn owner(&self) -> SkillOwner {
        self.owner
    }

    pub(crate) fn lifecycle(&self) -> &SkillLifecycle {
        self.execution.lifecycle()
    }

    pub(crate) fn lifecycle_mut(&mut self) -> &mut SkillLifecycle {
        self.execution.lifecycle_mut()
    }

    pub(crate) fn player_state<State: super::skills::kernel::PlayerSkillState>(&self) -> Option<&State> {
        match &self.execution {
            RegisteredSkillExecution::Player(execution) => State::from_execution(execution),
            _ => None,
        }
    }

    pub(crate) fn player_state_mut<State: super::skills::kernel::PlayerSkillState>(&mut self) -> Option<&mut State> {
        match &mut self.execution {
            RegisteredSkillExecution::Player(execution) => State::from_execution_mut(execution),
            _ => None,
        }
    }

    /// Один payload полёта находится в активной ветви исполнения владельца.
    /// Player и Monster не копируют его друг у друга при callbacks.
    pub(crate) fn targeted_projectile_progress(&self) -> Option<&super::skills::targetedprojectile::TargetedProjectileProgress> {
        match &self.execution {
            RegisteredSkillExecution::Player(PlayerSkillExecution::TargetedProjectile(state)) => Some(state.progress()),
            RegisteredSkillExecution::Monster(execution) => match execution.progress.as_ref()? {
                super::monster::MonsterSkillProgress::TargetedProjectile(state) => Some(state),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn targeted_projectile_progress_mut(&mut self) -> Option<&mut super::skills::targetedprojectile::TargetedProjectileProgress> {
        match &mut self.execution {
            RegisteredSkillExecution::Player(PlayerSkillExecution::TargetedProjectile(state)) => Some(state.progress_mut()),
            RegisteredSkillExecution::Monster(execution) => match execution.progress.as_mut()? {
                super::monster::MonsterSkillProgress::TargetedProjectile(state) => Some(state),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn path_projectile_progress(&self) -> Option<&super::skills::energybolt::PathProjectileProgress> {
        match &self.retained_data {
            SkillRetainedData::PathProjectile(progress) => Some(progress),
            _ => None,
        }
    }

    pub(crate) fn path_projectile_progress_mut(&mut self) -> Option<&mut super::skills::energybolt::PathProjectileProgress> {
        match &mut self.retained_data {
            SkillRetainedData::PathProjectile(progress) => Some(progress),
            _ => None,
        }
    }

    pub(crate) fn lightning_progress(&self) -> Option<&super::skills::lightning::LightningProgress> {
        match &self.execution {
            RegisteredSkillExecution::Player(PlayerSkillExecution::Lightning(state)) => Some(state.progress()),
            RegisteredSkillExecution::Monster(execution) => match execution.progress.as_ref()? {
                super::monster::MonsterSkillProgress::Lightning(state) => Some(state),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn lightning_progress_mut(&mut self) -> Option<&mut super::skills::lightning::LightningProgress> {
        match &mut self.execution {
            RegisteredSkillExecution::Player(PlayerSkillExecution::Lightning(state)) => Some(state.progress_mut()),
            RegisteredSkillExecution::Monster(execution) => match execution.progress.as_mut()? {
                super::monster::MonsterSkillProgress::Lightning(state) => Some(state),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn chain_lightning_progress(&self) -> Option<&super::skills::chainlightning::ChainLightningProgress> {
        match &self.execution {
            RegisteredSkillExecution::Player(PlayerSkillExecution::ChainLightning(state)) => Some(state.progress()),
            RegisteredSkillExecution::Monster(execution) => match execution.progress.as_ref()? {
                super::monster::MonsterSkillProgress::ChainLightning(state) => Some(state),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn chain_lightning_progress_mut(&mut self) -> Option<&mut super::skills::chainlightning::ChainLightningProgress> {
        match &mut self.execution {
            RegisteredSkillExecution::Player(PlayerSkillExecution::ChainLightning(state)) => Some(state.progress_mut()),
            RegisteredSkillExecution::Monster(execution) => match execution.progress.as_mut()? {
                super::monster::MonsterSkillProgress::ChainLightning(state) => Some(state),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn execution_stage(&self) -> Option<super::skills::kernel::SkillStage> {
        match &self.execution {
            RegisteredSkillExecution::Player(execution) => Some(execution.kernel().stage()),
            RegisteredSkillExecution::Monster(execution) => Some(execution.kernel.stage()),
            RegisteredSkillExecution::BattleFairy(execution) => Some(execution.kernel().stage()),
            RegisteredSkillExecution::Inactive(_) => None,
        }
    }

    pub(crate) fn base_projectile_progress(&self) -> Option<&super::skills::baseprojectilecast::BaseProjectileProgress> {
        match &self.retained_data {
            SkillRetainedData::BaseProjectile(progress) => Some(progress),
            _ => None,
        }
    }

    pub(crate) fn base_projectile_progress_mut(&mut self) -> Option<&mut super::skills::baseprojectilecast::BaseProjectileProgress> {
        match &mut self.retained_data {
            SkillRetainedData::BaseProjectile(progress) => Some(progress),
            _ => None,
        }
    }

    pub(crate) fn advance_execution(
        &mut self,
        from: super::skills::kernel::SkillStage,
        to: super::skills::kernel::SkillStage,
    ) -> bool {
        match &mut self.execution {
            RegisteredSkillExecution::Player(execution) => execution.kernel_mut().advance(from, to),
            RegisteredSkillExecution::Monster(execution) => execution.kernel.advance(from, to),
            RegisteredSkillExecution::BattleFairy(execution) => execution.kernel_mut().advance(from, to),
            RegisteredSkillExecution::Inactive(_) => false,
        }
    }

    pub(crate) fn monster_kernel(&self) -> Option<&super::monster::MonsterBaseAttackCast> {
        match &self.execution {
            RegisteredSkillExecution::Monster(execution) => Some(&execution.kernel),
            _ => None,
        }
    }

    pub(crate) fn monster_kernel_mut(&mut self) -> Option<&mut super::monster::MonsterBaseAttackCast> {
        match &mut self.execution {
            RegisteredSkillExecution::Monster(execution) => Some(&mut execution.kernel),
            _ => None,
        }
    }

    pub(crate) fn monster_progress<State: super::monster::MonsterSkillProgressState>(&self) -> Option<&State> {
        match &self.execution {
            RegisteredSkillExecution::Monster(execution) => State::from_progress(execution.progress.as_ref()?),
            _ => None,
        }
    }

    pub(crate) fn set_monster_progress(&mut self, progress: impl Into<super::monster::MonsterSkillProgress>) {
        if let RegisteredSkillExecution::Monster(execution) = &mut self.execution {
            execution.progress = Some(progress.into());
        }
    }

    pub(crate) fn execution_dispatch(&self) -> Option<RegisteredSkillDispatch> {
        match &self.execution {
            RegisteredSkillExecution::Player(execution) =>
                Some(RegisteredSkillDispatch::Player(execution.kernel().dispatch())),
            RegisteredSkillExecution::BattleFairy(execution) =>
                Some(RegisteredSkillDispatch::BattleFairy(execution.kernel().dispatch())),
            RegisteredSkillExecution::Inactive(_) | RegisteredSkillExecution::Monster(_) => None,
        }
    }

    pub(crate) fn player_dispatch(&self) -> Option<super::player::PlayerSkillDispatch> {
        match self.execution_dispatch() {
            Some(RegisteredSkillDispatch::Player(dispatch)) => Some(dispatch),
            _ => None,
        }
    }

    pub(crate) fn battle_fairy_dispatch(&self) -> Option<super::player::BattleFairySkillDispatch> {
        match self.execution_dispatch() {
            Some(RegisteredSkillDispatch::BattleFairy(dispatch)) => Some(dispatch),
            _ => None,
        }
    }

    pub(crate) fn battle_fairy_execution_state(&self) -> Option<&BattleFairyExecution> {
        match &self.execution {
            RegisteredSkillExecution::BattleFairy(execution) => Some(execution),
            _ => None,
        }
    }

    pub(crate) fn battle_fairy_execution_state_mut(&mut self) -> Option<&mut BattleFairyExecution> {
        match &mut self.execution {
            RegisteredSkillExecution::BattleFairy(execution) => Some(execution),
            _ => None,
        }
    }

    /// Материализация сохраняет уже начатую базу именно этого экземпляра.
    pub(crate) fn install_player_execution(&mut self, mut execution: PlayerSkillExecution) -> bool {
        if execution.kernel().dispatch().skill_id() != self.id
            || !matches!(self.execution, RegisteredSkillExecution::Inactive(_) | RegisteredSkillExecution::Player(_))
        {
            return false;
        }
        let lifecycle = std::mem::take(self.execution.lifecycle_mut());
        execution.kernel_mut().replace_lifecycle(lifecycle);
        self.execution = RegisteredSkillExecution::Player(execution);
        true
    }

    /// Материализация сохраняет уже начатую базу именно этого экземпляра.
    pub(crate) fn install_battle_fairy_execution(&mut self, mut execution: BattleFairyExecution) -> bool {
        if execution.kernel().dispatch().skill_id() != self.id
            || !matches!(self.execution, RegisteredSkillExecution::Inactive(_) | RegisteredSkillExecution::BattleFairy(_))
        {
            return false;
        }
        let lifecycle = std::mem::take(self.execution.lifecycle_mut());
        execution.kernel_mut().replace_lifecycle(lifecycle);
        self.execution = RegisteredSkillExecution::BattleFairy(execution);
        true
    }

    /// Убирает только payload этого экземпляра, без повторного поиска по ID.
    /// Общая база, visual и reuse не получают дополнительных End-переходов.
    pub(crate) fn clear_execution(&mut self, expected: RegisteredSkillDispatch) -> bool {
        if self.execution_dispatch() != Some(expected) {
            return false;
        }
        let lifecycle = std::mem::take(self.execution.lifecycle_mut());
        self.execution = RegisteredSkillExecution::Inactive(lifecycle);
        true
    }

    pub(crate) fn is_execution_inactive(&self) -> bool {
        matches!(self.execution, RegisteredSkillExecution::Inactive(_))
    }

    pub(crate) fn prepare_derived_end(&mut self, argument: i32) -> bool {
        match &mut self.execution {
            RegisteredSkillExecution::Player(execution) => {
                if self.owner.end_policy().reset_phase {
                    execution.kernel_mut().clear_phase_for_end();
                }
                if !execution.prepare_derived_end(argument) {
                    return false;
                }
            }
            RegisteredSkillExecution::Monster(execution) => execution.prepare_derived_end(),
            RegisteredSkillExecution::Inactive(_) | RegisteredSkillExecution::BattleFairy(_) => {}
        }
        true
    }

    pub(crate) fn clear_end_paths(&mut self) {
        if let SkillRetainedData::PathProjectile(progress) = &mut self.retained_data {
            progress.clear_end_paths();
        }
        match &mut self.execution {
            RegisteredSkillExecution::Player(execution) => execution.clear_end_paths(),
            RegisteredSkillExecution::Monster(execution) => execution.clear_end_paths(),
            RegisteredSkillExecution::Inactive(_) | RegisteredSkillExecution::BattleFairy(_) => {}
        }
    }

    pub(crate) fn mark_used(&mut self, now_ms: u32) {
        self.last_used_ms = now_ms;
    }

    pub(crate) const fn last_used_ms(&self) -> u32 {
        self.last_used_ms
    }

    pub(crate) fn replace_visual_effect(&mut self, effect: SkillVisualEffect) {
        self.current_visual_effect = Some(effect);
    }

    pub(crate) fn visual_effect_mut(&mut self) -> Option<&mut SkillVisualEffect> {
        self.current_visual_effect.as_mut()
    }

    pub(crate) fn visual_effect(&self) -> Option<&SkillVisualEffect> {
        self.current_visual_effect.as_ref()
    }

    /// Общий хвост CSkill::End после concrete cleanup и OnEndSkill.
    /// Владеющий visual не входит в копируемый снимок скалярного lifecycle.
    pub(crate) fn finish_base(&mut self, termination: SkillTermination) {
        self.clear_base_end_context();
        self.finish_cleared_base_end(termination);
    }

    pub(crate) fn clear_base_end_context(&mut self) {
        self.execution.lifecycle_mut().clear_end_context();
    }

    /// Visual и IsEnded следуют после отдельного native reuse-clock.
    pub(crate) fn finish_cleared_base_end(&mut self, termination: SkillTermination) {
        let visual = &mut self.current_visual_effect;
        self.execution
            .lifecycle_mut()
            .finish_end(termination, || drop(visual.take()));
    }

    pub(crate) const fn id(&self) -> u32 {
        self.id
    }

    pub(crate) const fn level(&self) -> i32 {
        self.level
    }

    pub(crate) const fn skill_type(&self) -> u32 {
        self.owner.category() as u32
    }

    pub(crate) fn name<'a>(&self, factory: &'a CSkillFactory) -> Option<&'a [u8]> {
        factory.query_skill_base_properties(self.id, self.level)
            .map(|properties| properties.skill_name())
    }

    pub(crate) const fn item_position(&self) -> i32 {
        self.item_position
    }

    pub(crate) const fn set_item_position(&mut self, position: i32) {
        self.item_position = position;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MoveShapePositionBlock {
    Coordinate(ShapeCoordinateBlock),
    ShapeBlock(ShapeBlockError),
    InvalidAreaSpan { width: i32, height: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MoveShapePositionFacts {
    pub(crate) current_hit_points: u32,
    pub(crate) figure: ShapeFigure,
    pub(crate) current_area: Option<ShapeAreaCoordinates>,
    pub(crate) area_width: i32,
    pub(crate) area_height: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MoveShapePet {
    pub(crate) object_type: i32,
    pub(crate) id: i32,
    pub(crate) figure: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KillingAttackIdentity {
    pub(crate) attacker_type: i32,
    pub(crate) attacker_id: i32,
    pub(crate) attacker_faction_id: i32,
}

impl From<&super::states::attackpower::AttackInformation> for KillingAttackIdentity {
    fn from(attack: &super::states::attackpower::AttackInformation) -> Self {
        Self {
            attacker_type: attack.attacker_type,
            attacker_id: attack.attacker_id,
            attacker_faction_id: attack.attacker_faction_id,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MoveShapeCommandBlock {
    Coordinate(ShapeCoordinateBlock),
    RegionCell(RegionCellAccessBlock),
    Position(RegionMembershipBlock),
    DetachedPosition(MoveShapePositionBlock),
}

pub(crate) trait MoveShapeResolver: ShapeResolver {
    /// `Some` означает успешный RTTI `CShape -> CMoveShape`; значение хранит
    /// exact `!IsDied`, полученный у concrete derived owner-а.
    fn move_shape_is_alive(&self, identity: ShapeIdentity) -> Option<bool>;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct MoveShapePropertyModifiers {
    pub(crate) maximum_hp: i32,
    pub(crate) maximum_mp: i32,
    pub(crate) maximum_yp: i32,
    pub(crate) maximum_rp: i32,
    pub(crate) strength: i32,
    pub(crate) dexterity: i32,
    pub(crate) constitution: i32,
    pub(crate) intelligence: i32,
    pub(crate) minimum_attack: i32,
    pub(crate) maximum_attack: i32,
    pub(crate) hit: i32,
    pub(crate) burden: i32,
    pub(crate) critical_hit: i32,
    pub(crate) defense: i32,
    pub(crate) dodge: i32,
    pub(crate) attack_speed: i32,
    pub(crate) element_resistance: i32,
    pub(crate) hp_recovery_speed: i32,
    pub(crate) mp_recovery_speed: i32,
    pub(crate) soul_resistance: i32,
    pub(crate) additional_element_attack: i32,
    pub(crate) additional_soul_attack: i32,
    pub(crate) element_modify: i32,
    pub(crate) attack_avoid: i32,
    pub(crate) element_avoid: i32,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CMoveShape {
    shape: CShape,
    skills: [SkillCollection; 4],
    current_skill_id: Option<u32>,
    item_skill_ids: Vec<u32>,
    state_storage: CanonicalStateStorage,
    property_modifiers: MoveShapePropertyModifiers,
    moveable_count: i32,
    moveable: bool,
    can_fight_count: i32,
    can_fight: bool,
    is_god: bool,
    pets: Vec<MoveShapePet>,
    current_pets_mode: i32,
    stiffen_started_ms: u32,
    stiffen_count: i32,
    killed_by: Option<KillingAttackIdentity>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct LegacyStateCodec {
    payload: Vec<u8>,
    opaque_tail: Vec<u8>,
    opaque_count: u32,
    header_was_present: bool,
}

impl LegacyStateCodec {
    fn replace(&mut self, payload: Vec<u8>) {
        self.header_was_present = payload.len() >= 4;
        self.payload = payload;
        self.opaque_tail.clear();
        self.opaque_count = 0;
    }

    fn clear(&mut self) {
        self.replace(Vec::new());
    }

    fn with_opaque_tail(&self, mut payload: Vec<u8>) -> Vec<u8> {
        if !self.header_was_present && read_u32(&payload, 0).unwrap_or(0) == 0 {
            return self.opaque_tail.clone();
        }
        if self.opaque_count == 0 && self.opaque_tail.is_empty() {
            return payload;
        }
        if payload.len() < 4 {
            payload = 0u32.to_le_bytes().to_vec();
        }
        let count = read_u32(&payload, 0).unwrap_or(0);
        write_u32(&mut payload, 0, count.wrapping_add(self.opaque_count));
        payload.extend_from_slice(&self.opaque_tail);
        payload
    }
}

impl Deref for LegacyStateCodec {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

impl DerefMut for LegacyStateCodec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.payload
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct CanonicalStateStorage {
    state_entries: AppliedStateEntries,
    consumable_restore_intervals: ConsumableRestoreIntervals,
    ex_states: LegacyStateCodec,
}

impl Deref for CMoveShape {
    type Target = CanonicalStateStorage;

    fn deref(&self) -> &Self::Target {
        &self.state_storage
    }
}

impl DerefMut for CMoveShape {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state_storage
    }
}


impl Default for CMoveShape {
    fn default() -> Self {
        Self {
            shape: CShape::default(),
            skills: Default::default(),
            current_skill_id: None,
            item_skill_ids: Vec::new(),
            state_storage: CanonicalStateStorage::default(),
            property_modifiers: MoveShapePropertyModifiers::default(),
            moveable_count: 0,
            moveable: true,
            can_fight_count: 0,
            can_fight: true,
            is_god: false,
            pets: Vec::new(),
            current_pets_mode: 1,
            stiffen_started_ms: 0,
            stiffen_count: 0,
            killed_by: None,
        }
    }
}

impl CMoveShape {
    pub(crate) const fn property_modifiers(&self) -> &MoveShapePropertyModifiers {
        &self.property_modifiers
    }

    pub(crate) const fn property_modifiers_mut(&mut self) -> &mut MoveShapePropertyModifiers {
        &mut self.property_modifiers
    }

    pub(crate) fn reset_property_modifiers(&mut self) {
        self.property_modifiers = MoveShapePropertyModifiers::default();
    }

    pub(crate) fn set_killed_by(&mut self, attack: KillingAttackIdentity) {
        self.killed_by = Some(attack);
    }

    pub(crate) const fn killed_by(&self) -> Option<KillingAttackIdentity> {
        self.killed_by
    }

    /// Exact `CMoveShape::Stiffen` (`RVA 0x000CD2F0`): окно и limit проверяются
    /// до RNG, просроченное окно делает второй замер часов, а вероятность
    /// уменьшается на `GetReAnk` перед signed-сравнением с `random(100)`.
    pub(crate) fn stiffen(
        &mut self,
        damage: u16,
        maximum_hp: u32,
        reank: u16,
        setup: crate::setup::globesetup::GlobeStiffenSetup,
        mut now_ms: impl FnMut() -> u32,
        mut random: impl FnMut(i32) -> i32,
    ) -> u32 {
        let now = now_ms();
        if self
            .stiffen_started_ms
            .wrapping_add(setup.bound_time_ms)
            < now
        {
            self.stiffen_started_ms = now_ms();
            self.stiffen_count = 0;
        } else if self.stiffen_count >= setup.limit {
            return 0;
        }

        let damage_ratio = f32::from(damage) / maximum_hp as f32;
        let probability = setup
            .damage_thresholds
            .iter()
            .zip(setup.probabilities)
            .take(usize::from(setup.count).min(4))
            .find_map(|(threshold, probability)| {
                (damage_ratio >= *threshold).then_some(probability)
            })
            .unwrap_or_default();
        let chance = i32::from(probability) - i32::from(reank);
        if random(100) > chance {
            return 0;
        }
        self.stiffen_count = self.stiffen_count.wrapping_add(1);
        setup.delay_ms
    }

    pub(crate) const fn current_pets_mode(&self) -> i32 {
        self.current_pets_mode
    }

    pub(crate) fn set_current_pets_mode(&mut self, mode: i32) -> bool {
        if self.current_pets_mode == mode {
            return false;
        }
        self.current_pets_mode = mode;
        true
    }

    pub(crate) fn add_pet(&mut self, object_type: i32, id: i32, figure: i32) {
        self.pets.push(MoveShapePet {
            object_type,
            id,
            figure,
        });
    }

    pub(crate) fn remove_pet(&mut self, object_type: i32, id: i32) -> bool {
        let Some(index) = self
            .pets
            .iter()
            .position(|pet| pet.object_type == object_type && pet.id == id)
        else {
            return false;
        };
        self.pets.remove(index);
        true
    }

    pub(crate) fn pets(&self) -> &[MoveShapePet] {
        &self.pets
    }

    pub(crate) fn skill_level(&self, skill_id: u32, factory: &CSkillFactory) -> i32 {
        self.skill(skill_id, factory).map_or(0, MoveShapeSkill::level)
    }

    pub(crate) const fn shape(&self) -> &CShape {
        &self.shape
    }

    pub(crate) const fn shape_mut(&mut self) -> &mut CShape {
        &mut self.shape
    }

    /// Материализует точный fresh-object prefix
    /// `CMoveShape::AddToByteArray_ForClient`: после `CShape` идут died-byte и
    /// нулевой count состояний. Метод намеренно не изображает общий state
    /// serializer и применяется до установки первого состояния.
    pub(crate) fn encode_fresh_client_snapshot(
        &self,
        include_child: bool,
        is_dead: bool,
    ) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        self.shape
            .add_to_byte_array(&mut payload, include_child)
            .then_some(())?;
        let mut writer = LegacyWriter::new(&mut payload);
        writer.write_u8(u8::from(is_dead));
        writer.write_i32(0);
        Some(payload)
    }

    /// Материализует общий CMoveShape::AddToByteArray_ForClient из живых
    /// экземпляров, а не их DB-записей. Незагруженный opaque owner блокирует
    /// snapshot: его клиентский контракт пока не подтверждён.
    pub(crate) fn encode_client_snapshot(
        &self,
        include_child: bool,
        is_dead: bool,
        timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        self.encode_client_snapshot_with_team_count(
            include_child,
            is_dead,
            1,
            timed_state_now_milliseconds,
        )
    }

    /// CMoveShape::AddToByteArray_ForClient (0x004CDD30): два прохода
    /// живого m_vStates, без DB Serialize и без уплотнения пустых позиций.
    /// Player-owner передаёт канонический размер CTeam для GetAdditionalData.
    pub(crate) fn encode_client_snapshot_with_team_count(
        &self,
        include_child: bool,
        is_dead: bool,
        team_member_count: usize,
        mut timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        // Непрозрачный незагруженный owner не выдаётся за пустое состояние.
        if self.ex_states.opaque_count != 0
            || (!self.ex_states.header_was_present && !self.ex_states.opaque_tail.is_empty())
        {
            return None;
        }
        let mut total_count = 0i32;
        for index in 0..self.state_entries.len() {
            if let Some(key) = self.state_entries.address(index) {
                self.state_entries.get(key)?;
                total_count = total_count.checked_add(1)?;
            }
        }
        let mut payload = Vec::new();
        self.shape.add_to_byte_array(&mut payload, include_child).then_some(())?;
        let mut writer = LegacyWriter::new(&mut payload);
        writer.write_u8(u8::from(is_dead));
        writer.write_i32(total_count);
        for index in 0..self.state_entries.len() {
            let Some(key) = self.state_entries.address(index) else { continue };
            let state = self.state_entries.get(key)?;
            writer.write_u32(state.state_id());
            let record = crate::gameserver::appserver::states::state::state_client_record(
                state, team_member_count, &mut timed_state_now_milliseconds,
            );
            writer.write_i32(record.time);
            writer.write_u32(record.additional);
            if let Some(name) = record.team_name {
                writer.write_c_string(name);
            }
        }
        Some(payload)
    }

    /// Exact inline `CMoveShape::God`: runtime-only invulnerability flag не
    /// сериализуется и проверяется ordinary `OnBeenAttacked` owner-ом.
    pub(crate) const fn set_god(&mut self, enabled: bool) {
        self.is_god = enabled;
    }

    pub(crate) const fn is_god(&self) -> bool {
        self.is_god
    }

    /// Exact nesting contract `SetFightable`: false добавляет запрет, true
    /// снимает один; отрицательный legacy count нормализуется только перед
    /// добавлением нового запрета.
    pub(crate) const fn set_fightable(&mut self, fightable: bool) {
        if !fightable {
            if self.can_fight_count < 0 {
                self.can_fight_count = 0;
            }
            self.can_fight_count = self.can_fight_count.wrapping_add(1);
        } else {
            self.can_fight_count = self.can_fight_count.wrapping_sub(1);
        }
        self.can_fight = self.can_fight_count < 1;
    }

    pub(crate) const fn can_fight(&self) -> bool {
        self.can_fight
    }

    pub(crate) fn skills(&self) -> impl Iterator<Item = &MoveShapeSkill> {
        self.skills.iter().flat_map(|category| category.iter())
    }

    pub(crate) fn skills_in_category(&self, category: SkillCategory) -> impl ExactSizeIterator<Item = &MoveShapeSkill> {
        self.skills[category as usize].iter()
    }

    /// `AutoStartPassiveSkill`: state-вектор обходится в порядке
    /// вставки, а каждый `IsAutoStart != 0` добавляется в background-очередь.
    /// Self-target `Begin(this, this)` в Rust задаётся самим владельцем.
    pub(crate) fn auto_start_passive_skills(&mut self, ai: &mut CBaseAI) -> usize {
        let mut count = 0;
        let state_skills = &self.skills[SkillCategory::State as usize];
        for entity in &state_skills.order {
            let skill = state_skills.instances.get(*entity)
                .expect("порядок state-категории содержит живые экземпляры навыков");
            if is_immediate_state_skill(skill.id) {
                ai.add_pending_back_stage_skill(skill.id);
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
        let _ = self.compact_state_slots();
        let payload = Self::serialize_state_records_with_entries(
            self.ex_states.to_vec(),
            StateSerialization::Save(&mut self.state_entries),
            now_ms,
            timed_state_now_milliseconds,
            true,
        );
        self.ex_states.with_opaque_tail(payload)
    }

    pub(crate) fn serialized_ex_states(
        &self,
        now_ms: u32,
        timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Vec<u8> {
        self.ex_states.with_opaque_tail(
            self.serialize_state_records(now_ms, timed_state_now_milliseconds, false),
        )
    }

    fn serialize_state_records(
        &self,
        now_ms: u32,
        timed_state_now_milliseconds: impl FnMut() -> u32,
        canonical_order: bool,
    ) -> Vec<u8> {
        Self::serialize_state_records_with_entries(
            self.ex_states.to_vec(),
            StateSerialization::ReadOnly(&self.state_entries),
            now_ms,
            timed_state_now_milliseconds,
            canonical_order,
        )
    }

    fn serialize_state_records_with_entries(
        mut payload: Vec<u8>,
        mut states: StateSerialization<'_>,
        now_ms: u32,
        mut timed_state_now_milliseconds: impl FnMut() -> u32,
        canonical_order: bool,
    ) -> Vec<u8> {
        let spans = known_state_record_spans(&payload);
        let declared_count = read_u32(&payload, 0).map(|count| count as usize);
        let parsed_end = spans.last().map_or(4, |(offset, amount)| offset + amount);
        let mut complete = declared_count == Some(spans.len()) && parsed_end == payload.len();
        let mut used = vec![false; spans.len()];
        let mut ordered_records = Vec::with_capacity(spans.len());

        for index in 0..states.entries().len() {
            let Some(key) = states.entries().address(index) else { continue };
            let Some(state) = states.entries().get(key) else {
                complete = false;
                continue;
            };
            let state_id = state.state_id();
            let exact_span = states.entries().serialized_span(key).map(Some).or_else(|| match state {
                StateData::ChangeBody(state) => Some(state.serialized_span()),
                StateData::Extended(state) => Some(state.serialized_span()),
                StateData::Undead(state) => Some(state.serialized_span()),
                StateData::Ride(state) => Some(state.serialized_span()),
                _ => None,
            });
            let record_index = if let Some(span) = exact_span {
                span.and_then(|span| spans.iter().enumerate().position(|(index, candidate)| {
                    !used[index] && *candidate == span
                        && read_u32(&payload, candidate.0) == Some(state_id)
                }))
            } else if let StateData::Swordship(state) = state {
                // Replace сохраняет runtime-позицию, но DB remove+append может
                // поменять порядок повторных ID. Все поля этой записи известны.
                let record = state.encoded();
                spans.iter().enumerate().position(|(index, (offset, amount))| {
                    !used[index] && payload.get(*offset..offset + amount) == Some(record.as_slice())
                })
            } else {
                let runtime_count = states.entries().iter_data()
                    .filter(|state| state.state_id() == state_id).count();
                let wire_count = spans.iter()
                    .filter(|(offset, _)| read_u32(&payload, *offset) == Some(state_id)).count();
                (runtime_count == wire_count).then(|| {
                    spans.iter().enumerate().position(|(index, (offset, _))| {
                        !used[index] && read_u32(&payload, *offset) == Some(state_id)
                    })
                }).flatten()
            };
            if let Some(record_index) = record_index {
                used[record_index] = true;
            } else {
                complete = false;
            }

            // Часы вызываются здесь, в едином порядке m_vStates. Отсутствие
            // однозначной DB-пары запрещает запись, но не добавляет type-pass.
            let mut commit_time: Option<(u32, fn(&mut StateData, u32))> = None;
            let encoded = match state {
                StateData::ChangeBody(state) => {
                    let remaining = state.client_state_time(&mut timed_state_now_milliseconds);
                    if let Some(record_index) = record_index {
                        state.update_serialized_record(
                            &mut payload, spans[record_index].0, remaining,
                        );
                    }
                    commit_time = Some((remaining, |data, remaining| {
                        if let Some(state) = ChangeBodyState::as_data_mut(data) {
                            state.commit_serialized_time(remaining);
                        }
                    }));
                    None
                }
                StateData::Extended(state) => {
                    let remaining = state.client_state_time(&mut timed_state_now_milliseconds);
                    if let Some(record_index) = record_index {
                        state.update_serialized_record(&mut payload, spans[record_index].0, remaining);
                    }
                    commit_time = Some((remaining, |data, remaining| {
                        if let Some(state) = ExtendedState::as_data_mut(data) {
                            state.commit_serialized_time(remaining);
                        }
                    }));
                    None
                }
                StateData::Undead(state) => {
                    let remaining = state.client_state_time(&mut timed_state_now_milliseconds);
                    if let Some(record_index) = record_index {
                        state.update_serialized_record(&mut payload, spans[record_index].0, remaining);
                    }
                    commit_time = Some((remaining, |data, remaining| {
                        if let Some(state) = UndeadState::as_data_mut(data) {
                            state.commit_serialized_time(remaining);
                        }
                    }));
                    None
                }
                StateData::LeafCut(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::LeafCut3(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::Kerosene(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::PoisonFog(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::MeteorArrow(state) => Some(state.encoded().to_vec()),
                StateData::Script(state) => Some(state.encoded(&mut timed_state_now_milliseconds)),
                StateData::ConsumableRestore(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::Blind(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::Seal(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::Strike(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::KnockOut(state) => Some([
                    state_id.to_le_bytes(),
                    (state.client_time(&mut timed_state_now_milliseconds) as u32).to_le_bytes(),
                ].concat()),
                StateData::SpiderWeb(state) => Some([
                    state_id.to_le_bytes(),
                    (state.client_time(&mut timed_state_now_milliseconds) as u32).to_le_bytes(),
                ].concat()),
                StateData::GodBless(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::Cure(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::Weak(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::SoulCollect(state) => Some(state.encoded().to_vec()),
                StateData::SpriteBurn(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::SpiderPoison(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::DaubPoison(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::BossBlueQuake(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::KnightCut(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::BoaLock(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::Rush(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::Roar(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::Pillar(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::RageBreak(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::Hearten(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::Heal(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::Fury(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::TianShenXiaFan(state) => Some(state.encoded().to_vec()),
                StateData::Wangsheng(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::DefenseShield(state) => Some(match state {
                    DefenseShieldState::Mana(state) => state.encoded(&mut timed_state_now_milliseconds).to_vec(),
                    DefenseShieldState::Machine(state) => state.encoded(&mut timed_state_now_milliseconds).to_vec(),
                    DefenseShieldState::Life(state) => state.encoded(&mut timed_state_now_milliseconds).to_vec(),
                    DefenseShieldState::Promotion(state) => state.encoded(&mut timed_state_now_milliseconds).to_vec(),
                }),
                StateData::LeafCut2(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::Rush2(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::Agility2(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::BloodLoss(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::EnergyHolding(state) => Some(state.encoded().to_vec()),
                StateData::Callosity(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::BossBlueFury(state) => Some(state.encoded(now_ms).to_vec()),
                StateData::PoisonArrow(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                StateData::BattleFairyAttribute(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
                // Эти неизменяемые записи уже синхронизированы при установке.
                // В частности, не обнуляем сохранённый padding tagWuXingState.
                StateData::PersistentAgility(_) | StateData::TaiJi(_)
                | StateData::EnlargeFullMiss(_) | StateData::EnlargeMaxHp(_)
                | StateData::EnlargeMaxMp(_) | StateData::Origin(_)
                | StateData::Swordship(_) | StateData::WuXing(_)
                | StateData::AutomaticRestore(_) | StateData::Particular(_)
                | StateData::Team(_) | StateData::Ride(_) => None,
            };
            if let Some((remaining, commit)) = commit_time
                && let Some(state) = states.get_mut(key)
            {
                // Serialize сохраняет тот же остаток без повторного getter/clock
                // и без сброса started/item timestamp. ReadOnly не даёт &mut.
                commit(state, remaining);
            }
            if let Some(record_index) = record_index {
                let (offset, amount) = spans[record_index];
                if let Some(record) = encoded {
                    if record.len() == amount {
                        payload[offset..offset + amount].copy_from_slice(&record);
                    } else {
                        complete = false;
                    }
                }
                ordered_records.push(record_index);
            }
        }

        if !canonical_order || !complete || used.iter().any(|used| !used) {
            return payload;
        }
        let mut ordered = Vec::with_capacity(payload.len());
        ordered.extend_from_slice(&payload[..4]);
        for record_index in ordered_records {
            let (offset, amount) = spans[record_index];
            ordered.extend_from_slice(&payload[offset..offset + amount]);
        }
        ordered
    }

    pub(crate) fn replace_ex_states(&mut self, states: Vec<u8>, skill_factory: &CSkillFactory, now: &mut dyn FnMut() -> u32) {
        let state_owner = self.shape.identity();
        self.state_entries.clear();
        let declared_count = read_u32(&states, 0);
        let mut payload = 0u32.to_le_bytes().to_vec();
        let mut cursor = if declared_count.is_some() { 4 } else { 0 };
        let mut decoded_count = 0u32;
        for _ in 0..declared_count.unwrap_or(0) {
            let cache_offset = payload.len();
            let Some((state, consumed)) = decode_state_record_into_cache(
                &states, cursor, &mut payload, state_owner, skill_factory, now,
            ) else { break };
            let cache_size = payload.len() - cache_offset;
            self.state_entries.append_loaded_data(state, (cache_offset, cache_size));
            cursor += consumed;
            decoded_count += 1;
        }
        write_u32(&mut payload, 0, decoded_count);
        self.ex_states = LegacyStateCodec {
            payload,
            opaque_tail: states[cursor..].to_vec(),
            opaque_count: declared_count.unwrap_or(0) - decoded_count,
            header_was_present: declared_count.is_some(),
        };
    }

    pub(crate) fn clear_persisted_runtime_state(&mut self) {
        self.skills.iter_mut().for_each(SkillCollection::clear);
        self.current_skill_id = None;
        self.item_skill_ids.clear();
        self.ex_states.clear();
        self.state_entries.clear();
        self.can_fight_count = 0;
        self.can_fight = true;
    }

    pub(crate) fn ride_state(&self) -> Option<&RideState> {
        self.state_entries.first::<RideState>()
    }

    /// Scalar-prefix CMoveShape::OnEnterRegion; конкретные Begin заново
    /// устанавливают свои запреты после этого сброса в общем живом проходе.
    pub(crate) const fn reset_region_entry_control(&mut self) {
        self.moveable = true;
        self.can_fight = true;
        self.moveable_count = 0;
        self.can_fight_count = 0;
    }

    pub(crate) fn has_ride_state(&self) -> bool {
        self.state_entries.first::<RideState>().is_some()
    }

    /// Хвост RestoreHpMp (0x00445643..0x00445901): четыре новых экземпляра
    /// после общего End-обхода. Begin(null, holder) не читает часы; старые
    /// состояния здесь повторно не удаляются и UpdateProperty не вызывается.
    pub(crate) fn append_automatic_hp_mp_states(
        &mut self,
        properties: super::player::PlayerCombatProperties,
    ) {
        for state in AutomaticRestoreState::restored(properties) {
            self.append_automatic_restore_state(state);
        }
    }

    /// Общий NULL-user Begin свежего restore: без clock и пакета, visual loop=1.
    pub(crate) fn append_automatic_restore_state(&mut self, state: AutomaticRestoreState) -> StateKey {
        self.append_serialized_state_record(&state.encoded_for_install());
        let offset = self.ex_states.len() - AUTOMATIC_RESTORE_STATE_BYTES;
        let key = self.state_entries.append(state);
        self.state_entries.set_serialized_span(key, (offset, AUTOMATIC_RESTORE_STATE_BYTES));
        self.mark_applied_state_begun(key);
        self.state_entries.begin_visual(key, 1);
        key
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
        if !self.consumable_restore_intervals.try_begin(health, interval_ms, &mut *now) {
            return None;
        }
        Some(if health {
            ConsumableRestoreState::Health(RestoreHpState::new(
                time_to_keep_ms, frequency_ms, amount,
            ))
        } else {
            ConsumableRestoreState::Mana(RestoreMpState::new(
                time_to_keep_ms, frequency_ms, amount,
            ))
        })
    }

    pub(crate) fn particular_states(&self) -> impl Iterator<Item = &ParticularState> {
        self.state_entries.iter::<ParticularState>()
    }

    pub(crate) fn team_recruitment_states(&self) -> impl Iterator<Item = &CTeamState> {
        self.state_entries.iter::<CTeamState>()
    }


    pub(crate) fn automatic_restore_state(&self, key: StateKey) -> Option<AutomaticRestoreState> {
        self.applied_state::<AutomaticRestoreState>(key).copied()
    }

    pub(crate) fn automatic_restore_state_mut(
        &mut self,
        key: StateKey,
    ) -> Option<&mut AutomaticRestoreState> {
        self.applied_state_mut::<AutomaticRestoreState>(key)
    }

    /// Точный `GetStateNumByStateID`: считает все живые экземпляры с данным
    /// базовым `CState::m_lID`, независимо от concrete owner-а состояния.
    pub(crate) fn state_count_by_state_id(&self, state_id: i32) -> u32 {
        (0..self.state_entries.len())
            .filter(|index| self.state_id_at(*index) == Some(state_id as u32))
            .count().min(u32::MAX as usize) as u32
    }

    /// `GetStateBySkillID` просматривает канонические типизированные состояния
    /// по фактическому идентификатору навыка, а не по классу сетевой записи.
    pub(crate) fn has_state_by_skill_id(&self, state_id: u32) -> bool {
        (0..self.state_entries.len()).any(|index| self.state_id_at(index) == Some(state_id))
    }


    pub(crate) fn swordship_states(&self) -> impl Iterator<Item = &SwordshipState> {
        self.state_entries.iter::<SwordshipState>()
    }

    pub(crate) fn wuxing_states(&self) -> impl Iterator<Item = &WuXingState> {
        self.state_entries.iter::<WuXingState>()
    }

    pub(crate) fn taiji_state(&self) -> Option<TaiJiState> {
        self.state_entries.first::<TaiJiState>().copied()
    }

    pub(crate) fn enlarge_full_miss_state(&self) -> Option<EnlargeFullMissState> {
        self.state_entries.first::<EnlargeFullMissState>().copied()
    }

    pub(crate) fn enlarge_max_hp_state(&self) -> Option<EnlargeMaxHpState> {
        self.state_entries.first::<EnlargeMaxHpState>().copied()
    }

    pub(crate) fn enlarge_max_mp_state(&self) -> Option<EnlargeMaxMpState> {
        self.state_entries.first::<EnlargeMaxMpState>().copied()
    }

    pub(crate) fn origin_state(&self) -> Option<OriginState> {
        self.state_entries.first::<OriginState>().copied()
    }








    pub(crate) fn take_boss_blue_fury_state(&mut self) -> Option<BossBlueFuryState> {
        let state = self.state_entries.take_first::<BossBlueFuryState>()?;
        self.remove_serialized_state_record(state.skill_id(), BOSS_BLUE_FURY_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn begin_boss_blue_fury_state(&mut self, state: BossBlueFuryState) {
        self.append_serialized_state_record(&state.encoded_for_install());
        self.state_entries.append(state);
    }

    pub(crate) fn boss_blue_fury_state(&self) -> Option<BossBlueFuryState> {
        self.state_entries.first::<BossBlueFuryState>().copied()
    }





    pub(crate) fn replace_boss_blue_quake_state(
        &mut self,
        state: BossBlueQuakeState,
    ) -> Option<BossBlueQuakeState> {
        self.remove_serialized_state_record(state.skill_id(), BOSS_BLUE_QUAKE_STATE_BYTES);
        self.append_serialized_state_record(&state.encoded_for_install());
        {
            let previous = self.state_entries.take_first::<BossBlueQuakeState>();
            self.state_entries.append(state);
            previous
        }
    }





    pub(crate) fn take_boss_blue_quake_state(&mut self) -> Option<BossBlueQuakeState> {
        let state = self.state_entries.take_first::<BossBlueQuakeState>()?;
        self.remove_serialized_state_record(state.skill_id(), BOSS_BLUE_QUAKE_STATE_BYTES);
        Some(state)
    }

    pub(crate) fn promotion_magic_attack_factor(&self) -> Option<u16> {
        self.state_entries.iter::<DefenseShieldState>().find_map(|state| match state {
            DefenseShieldState::Promotion(state) => Some(state.magic_attack_factor()),
            _ => None,
        })
    }


    pub(crate) fn defense_shields(&self) -> impl Iterator<Item = &DefenseShieldState> {
        self.state_entries.iter::<DefenseShieldState>()
    }

    pub(crate) fn defense_shield_keys(&self) -> Vec<StateKey> {
        self.state_entries.keys::<DefenseShieldState>()
    }

    pub(crate) fn defense_shield_key(&self, skill_id: u32) -> Option<StateKey> {
        self.defense_shield_keys().into_iter().find(|key| {
            self.defense_shield(*key).is_some_and(|state| state.skill_id() == skill_id)
        })
    }

    pub(crate) fn defense_shield(&self, key: StateKey) -> Option<&DefenseShieldState> {
        match self.state_entries.get(key)? {
            StateData::DefenseShield(state) => Some(state),
            _ => None,
        }
    }

    pub(crate) fn remove_defense_shield(&mut self, skill_id: u32) -> Option<DefenseShieldState> {
        self.remove_defense_shield_key(self.defense_shield_key(skill_id)?)
    }

    pub(crate) fn remove_defense_shield_key(&mut self, key: StateKey) -> Option<DefenseShieldState> {
        let state_id = self.defense_shield(key)?.skill_id();
        let occurrence = self.defense_shield_keys().into_iter()
            .filter(|candidate| {
                self.defense_shield(*candidate).is_some_and(|state| state.skill_id() == state_id)
            })
            .position(|candidate| candidate == key)?;
        let offset = known_state_record_offsets(&self.ex_states)
            .into_iter()
            .filter(|offset| read_u32(&self.ex_states, *offset) == Some(state_id))
            .nth(occurrence);
        let state = self.state_entries.take::<DefenseShieldState>(key)?;
        let bytes = match state {
            DefenseShieldState::Mana(_) => MANA_SHIELD_STATE_BYTES,
            DefenseShieldState::Machine(_) => MACHINE_SHIELD_STATE_BYTES,
            DefenseShieldState::Life(_) => LIFE_SHIELD_STATE_BYTES,
            DefenseShieldState::Promotion(_) => PROMOTION_STATE_BYTES,
        };
        if let Some(offset) = offset {
            self.remove_serialized_state_record_at(offset, bytes);
        }
        Some(state)
    }



    fn append_serialized_state_record(&mut self, record: &[u8]) {
        if self.ex_states.len() < 4 {
            self.ex_states.clear();
            LegacyWriter::new(&mut self.ex_states).write_u32(0);
        }
        let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
        write_u32(&mut self.ex_states, 0, count.wrapping_add(1));
        self.ex_states.extend_from_slice(record);
    }

    /// Общий push_back уже успешно начатого concrete state. DB-cache здесь
    /// технический: record подготовлен owner-ом без вызова игрового Serialize
    /// и без повторных часов. Новый span принадлежит тому же поколенческому
    /// ключу, поэтому дубли ID удаляются независимо. End/Update остаются caller-у.
    pub(crate) fn append_applied_state_record<T: AppliedState>(
        &mut self, state: T, record: &[u8],
    ) -> StateKey {
        self.append_serialized_state_record(record);
        let span = (self.ex_states.len() - record.len(), record.len());
        let key = self.state_entries.append(state);
        self.state_entries.set_serialized_span(key, span);
        key
    }

    /// Первый живой слот исходного m_vStates. Предикат задаёт игровой выбор
    /// caller-а; метод не копирует payload, не уплотняет и не вызывает End.
    /// После callback caller перечитывает эту позицию, если это требует EXE.
    pub(crate) fn find_state_position(
        &self, mut matches: impl FnMut(&StateData) -> bool,
    ) -> Option<(usize, StateKey)> {
        (0..self.state_slot_count()).find_map(|index| {
            let (key, state) = self.state_at(index)?;
            matches(state).then_some((index, key))
        })
    }

    fn remove_serialized_state_record(&mut self, state_id: u32, amount: usize) -> bool {
        let Some(offset) = known_state_record_offsets(&self.ex_states)
            .into_iter()
            .find(|offset| read_u32(&self.ex_states, *offset) == Some(state_id))
        else {
            return false;
        };
        self.remove_serialized_state_record_at(offset, amount)
    }

    fn remove_serialized_state_record_at(&mut self, offset: usize, amount: usize) -> bool {
        let Some(end) = offset.checked_add(amount).filter(|end| *end <= self.ex_states.len()) else {
            return false;
        };
        self.ex_states.drain(offset..end);
        let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
        write_u32(&mut self.ex_states, 0, count.saturating_sub(1));
        self.shift_serialized_state_offsets_after(offset, amount);
        true
    }

    fn shift_serialized_state_offsets_after(&mut self, offset: usize, amount: usize) {
        self.state_entries.shift_serialized_spans_after_remove(offset, amount);
        self.state_entries.for_each_mut::<ExtendedState>(|state| state.shift_serialized_offset_after(offset, amount));
        self.state_entries.for_each_mut::<ChangeBodyState>(|state| state.shift_serialized_offset_after(offset, amount));
        self.state_entries.for_each_mut::<UndeadState>(|state| state.shift_serialized_offset_after(offset, amount));
        self.state_entries.for_each_mut::<RideState>(|state| state.shift_serialized_offset_after(offset, amount));
    }

    pub(crate) fn take_defense_shields(&mut self) -> StateBatch<DefenseShieldState> {
        self.state_entries.take_batch::<DefenseShieldState>()
    }

    pub(crate) fn restore_defense_shields(&mut self, states: StateBatch<DefenseShieldState>) {
        self.state_entries.restore_batch(states);
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
        let (position, offset) = location;
        if position >= self.state_entries.len() || offset < 4 || offset > self.ex_states.len() {
            return None;
        }
        let amount = record.len();
        self.ex_states.splice(offset..offset, record.iter().copied());
        self.state_entries.shift_serialized_spans_for_insert(offset, amount);
        let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
        write_u32(&mut self.ex_states, 0, count.wrapping_add(1));
        self.state_entries.for_each_mut::<ExtendedState>(|known| { known.shift_serialized_offset_for_insert(offset, amount); });
        self.state_entries.for_each_mut::<ChangeBodyState>(|known| { known.shift_serialized_offset_for_insert(offset, amount); });
        self.state_entries.for_each_mut::<UndeadState>(|known| { known.shift_serialized_offset_for_insert(offset, amount); });
        self.state_entries.for_each_mut::<RideState>(|known| { known.shift_serialized_offset_for_insert(offset, amount); });
        let _ = self.state_entries.replace_at(position, state);
        let key = self.state_entries.address(position)?;
        self.state_entries.set_serialized_span(key, (offset, amount));
        Some(key)
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

    fn state_id_at(&self, index: usize) -> Option<u32> {
        Some(self.state_entries.get(self.state_entries.address(index)?)?.state_id())
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
        T::as_data_ref(self.state_entries.get(key)?)?;
        self.remove_applied_state_data(key, amount).and_then(T::from_data)
    }

    /// Только удаление точного payload и его wire-записи; игровой End с
    /// visual, счётчиками и UpdateProperty выполняется владельцем снаружи.
    pub(crate) fn remove_applied_state_data(
        &mut self,
        key: StateKey,
        amount: usize,
    ) -> Option<StateData> {
        self.remove_applied_state_data_inner(key, Some(amount))
    }

    /// Destructor-only хвост ClearAllStates: wire-размер берётся из того же
    /// decoder-а, а не из второго каталога типов или выдуманного базового размера.
    pub(crate) fn remove_applied_state(&mut self, key: StateKey) -> Option<StateData> {
        self.remove_applied_state_data_inner(key, None)
    }

    fn remove_applied_state_data_inner(
        &mut self,
        key: StateKey,
        _amount: Option<usize>,
    ) -> Option<StateData> {
        let state_id = self.state_entries.get(key)?.state_id();
        let occurrence = self.state_entries.entries()
            .filter(|(_, state)| state.state_id() == state_id)
            .position(|(candidate, _)| candidate == key)?;
        let records: Vec<_> = known_state_record_spans(&self.ex_states).into_iter()
            .filter(|(offset, _)| read_u32(&self.ex_states, *offset) == Some(state_id))
            .collect();
        let runtime_count = self.state_entries.entries()
            .filter(|(_, state)| state.state_id() == state_id).count();
        // Известная длина ещё не гарантирует успешную материализацию записи.
        // Как в save, неоднозначный ordinal не разрешает удалять чужие байты.
        let record = (runtime_count == records.len()).then(|| records[occurrence]);
        let span = self.state_entries.serialized_span(key).or_else(|| match self.state_entries.get(key)? {
            StateData::Swordship(state) => {
                // Как в save: Replace оставляет runtime-позицию, но переносит
                // DB-запись в хвост. Ordinal повторного ID уже не задаёт экземпляр.
                let encoded = state.encoded();
                known_state_record_spans(&self.ex_states).into_iter().find(|(offset, size)| {
                    self.ex_states.get(*offset..*offset + *size) == Some(encoded.as_slice())
                })
            }
            StateData::EnergyHolding(state) => {
                // Factory может пропустить неизвестный level, сохранив его
                // wire-запись. Level неизменен; mutable charge для identity
                // непригоден, потому что DB-проекция обновляется при save.
                let level = state.skill_level();
                let same_level = self.state_entries.entries().filter(|(_, entry)| {
                    matches!(entry, StateData::EnergyHolding(entry) if entry.skill_level() == level)
                }).position(|(candidate, _)| candidate == key)?;
                records.into_iter().filter(|(offset, _)| {
                    read_u32(&self.ex_states, offset + 4) == Some(level)
                }).nth(same_level)
            }
            _ => record,
        });
        let position = self.state_entries.index_of(key)?;
        let state = self.state_entries.remove_at(position)?;
        if let Some((offset, amount)) = span {
            self.remove_serialized_state_record_at(offset, amount);
        }
        Some(state)
    }

























    pub(crate) fn replace_spider_web_state(
        &mut self,
        state: SpiderWebState,
    ) -> Option<SpiderWebState> {
        let previous = self.state_entries.first_key::<SpiderWebState>()
            .and_then(|key| self.remove_applied_state_record::<SpiderWebState>(key, SPIDER_WEB_STATE_BYTES));
        self.append_serialized_state_record(&state.encoded_for_install());
        self.state_entries.append(state);
        previous
    }






    pub(crate) fn energy_holding_states(&self) -> impl Iterator<Item = &EnergyHoldingState> {
        self.state_entries.iter::<EnergyHoldingState>()
    }

    pub(crate) fn take_spider_web_state(&mut self) -> Option<SpiderWebState> {
        let key = self.state_entries.first_key::<SpiderWebState>()?;
        let state = self.remove_applied_state_record::<SpiderWebState>(key, SPIDER_WEB_STATE_BYTES)?;
        Some(state)
    }











    pub(crate) fn pillar_state(&self) -> Option<PillarState> { self.state_entries.first::<PillarState>().copied() }



    pub(crate) fn curable_state_ids(&self) -> Vec<u32> {
        self.state_entries.iter_data().filter(|state| state.is_curable())
            .map(StateData::state_id).collect()
    }

    pub(crate) fn blind_state_order(&self) -> Vec<u32> {
        self.state_entries.iter_data().filter(|state| state.is_blind())
            .map(StateData::state_id).collect()
    }

    pub(crate) fn blind_state_instances(&self) -> Vec<(StateKey, u32)> {
        self.state_entries.entries().filter(|(_, state)| state.is_blind())
            .map(|(key, state)| (key, state.state_id())).collect()
    }


    pub(crate) fn battle_fairy_attribute_states(&self) -> impl Iterator<Item = &BattleFairyAttributeState> {
        self.state_entries.iter::<BattleFairyAttributeState>()
    }


    pub(crate) fn take_expired_battle_fairy_attribute_state(
        &mut self,
        key: StateKey,
        now_ms: u32,
    ) -> Option<BattleFairyAttributeState> {
        if !self.applied_state::<BattleFairyAttributeState>(key)?.expired(now_ms) {
            return None;
        }
        self.remove_applied_state_record::<BattleFairyAttributeState>(key, BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES)
    }












    pub(crate) fn script_states(&self) -> impl Iterator<Item = &ScriptMoveState> {
        self.state_entries.iter::<ScriptMoveState>()
    }





    pub(crate) fn tian_shen_xia_fan_state(&self) -> Option<TianShenXiaFanState> {
        self.state_entries.first::<TianShenXiaFanState>().copied()
    }





    pub(crate) fn wangsheng_state(&self) -> Option<WangshengState> {
        self.state_entries.first::<WangshengState>().copied()
    }








    pub(crate) fn get_undead_state(&self, state_id: u32) -> u32 {
        self.state_entries.iter::<UndeadState>()
            .any(|state| state.state_id == state_id)
            .then_some(state_id)
            .unwrap_or(0)
    }




    pub(crate) fn undead_state_tick(
        &mut self,
        key: StateKey,
        mut now_milliseconds: impl FnMut() -> u32,
    ) -> (bool, Option<(u32, u32)>) {
        let Some(state) = self.applied_state_mut::<UndeadState>(key) else {
            return (false, None);
        };
        if state.keep_time_ms != 0 && state.expired(now_milliseconds()) {
            return (true, None);
        }
        if state.last_item_tick_ms == 0 {
            state.last_item_tick_ms = state.started_ms;
        }
        if state.frequency_ms != 0 && state.item_index != 0 && state.item_amount != 0
            && state.item_due(now_milliseconds())
        {
            state.last_item_tick_ms = now_milliseconds();
            return (false, Some((state.item_index, state.item_amount)));
        }
        (false, None)
    }


    pub(crate) fn get_extended_state(&self, kind: ExtendedStateKind, state_id: u32) -> u32 {
        self.state_entries.iter::<ExtendedState>()
            .any(|state| state.kind == kind && state.level == state_id)
            .then_some(state_id)
            .unwrap_or(0)
    }

    pub(crate) fn extended_states(&self) -> impl Iterator<Item = &ExtendedState> {
        self.state_entries.iter::<ExtendedState>()
    }



    pub(crate) fn extended_state_tick(
        &mut self,
        key: StateKey,
        mut now_milliseconds: impl FnMut() -> u32,
    ) -> (bool, Option<(u32, u32)>) {
        let Some(state) = self.applied_state_mut::<ExtendedState>(key) else {
            return (false, None);
        };
        if state.keep_time_ms != 0 && state.expired(now_milliseconds()) {
            return (true, None);
        }
        if state.kind == ExtendedStateKind::New {
            if state.last_item_tick_ms == 0 {
                state.last_item_tick_ms = state.started_ms;
            }
            if state.frequency_ms != 0 && state.item_index != 0 && state.item_amount != 0
                && state.item_due(now_milliseconds())
            {
                state.restart_item_clock(now_milliseconds());
                return (false, Some((state.item_index, state.item_amount)));
            }
        }
        (false, None)
    }


    pub(crate) fn get_change_body_state(&self, state_id: u32) -> u32 {
        self.state_entries.iter::<ChangeBodyState>()
            .any(|state| state.level == state_id)
            .then_some(state_id)
            .unwrap_or_default()
    }


    pub(crate) fn active_change_body_state(&self) -> Option<&ChangeBodyState> {
        self.state_entries.iter::<ChangeBodyState>().last()
    }

    pub(crate) fn first_change_body_state_id(&self) -> Option<u32> {
        self.state_entries.first::<ChangeBodyState>().map(|state| state.level)
    }





    pub(crate) fn skill(&self, skill_id: u32, factory: &CSkillFactory) -> Option<&MoveShapeSkill> {
        self.skill_at(self.skill_slot(skill_id, factory)?)
    }

    fn skill_mut(&mut self, skill_id: u32, factory: &CSkillFactory) -> Option<&mut MoveShapeSkill> {
        self.skill_at_mut(self.skill_slot(skill_id, factory)?)
    }

    pub(crate) fn skill_slot(&self, skill_id: u32, factory: &CSkillFactory) -> Option<SkillSlot> {
        let category = SkillCategory::from_raw(factory.query_skill_type(skill_id, 1))?;
        let index = self.skills[category as usize].iter().position(|skill| skill.id == skill_id)?;
        self.skill_slot_at(category, index)
    }

    /// Прямой native-обход берёт текущий индекс категории, без QuerySkillType.
    /// Возвращённый ключ сохраняет идентичность через последующие callbacks.
    pub(crate) fn skill_slot_at(&self, category: SkillCategory, index: usize) -> Option<SkillSlot> {
        let entity = *self.skills[category as usize].order.get(index)?;
        Some(SkillSlot { category, entity })
    }

    pub(crate) fn skill_count_in_category(&self, category: SkillCategory) -> usize {
        self.skills[category as usize].order.len()
    }

    pub(crate) fn skill_at(&self, slot: SkillSlot) -> Option<&MoveShapeSkill> {
        self.skills[slot.category as usize].instances.get(slot.entity)
    }

    pub(crate) fn skill_at_mut(&mut self, slot: SkillSlot) -> Option<&mut MoveShapeSkill> {
        self.skills[slot.category as usize].instances.get_mut(slot.entity)
    }

    pub(crate) fn skill_lifecycle(
        &self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&SkillLifecycle> {
        Some(self.skill(skill_id, factory)?.execution.lifecycle())
    }

    pub(crate) fn skill_lifecycle_mut(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&mut SkillLifecycle> {
        Some(self.skill_mut(skill_id, factory)?.execution.lifecycle_mut())
    }

    pub(crate) fn skill_visual_effect_mut(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&mut SkillVisualEffect> {
        self.skill_mut(skill_id, factory)?.current_visual_effect.as_mut()
    }

    pub(crate) fn replace_skill_visual_effect(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
        effect: SkillVisualEffect,
    ) -> bool {
        let Some(skill) = self.skill_mut(skill_id, factory) else {
            return false;
        };
        skill.replace_visual_effect(effect);
        true
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
        match &self.skill(skill_id, factory)?.execution {
            RegisteredSkillExecution::Monster(execution) => Some(execution),
            _ => None,
        }
    }

    pub(crate) fn monster_skill_execution_mut(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&mut super::monster::MonsterSkillExecution> {
        match &mut self.skill_mut(skill_id, factory)?.execution {
            RegisteredSkillExecution::Monster(execution) => Some(execution),
            _ => None,
        }
    }

    pub(crate) fn install_monster_execution(
        &mut self,
        mut execution: super::monster::MonsterSkillExecution,
        factory: &CSkillFactory,
    ) -> bool {
        let skill_id = execution.kernel.dispatch().skill_id;
        let Some(skill) = self.skill_mut(skill_id, factory) else { return false };
        if !matches!(skill.execution, RegisteredSkillExecution::Inactive(_) | RegisteredSkillExecution::Monster(_)) {
            return false;
        }
        let lifecycle = std::mem::take(skill.execution.lifecycle_mut());
        execution.kernel.replace_lifecycle(lifecycle);
        skill.execution = RegisteredSkillExecution::Monster(execution);
        true
    }

    pub(crate) fn clear_monster_execution(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> bool {
        let Some(skill) = self.skill_mut(skill_id, factory) else { return false };
        if !matches!(skill.execution, RegisteredSkillExecution::Monster(_)) {
            return false;
        }
        let lifecycle = std::mem::take(skill.execution.lifecycle_mut());
        skill.execution = RegisteredSkillExecution::Inactive(lifecycle);
        true
    }

    pub(crate) fn player_execution(
        &self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&PlayerSkillExecution> {
        match &self.skill(skill_id, factory)?.execution {
            RegisteredSkillExecution::Player(execution) => Some(execution),
            _ => None,
        }
    }

    pub(crate) fn player_execution_mut(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&mut PlayerSkillExecution> {
        match &mut self.skill_mut(skill_id, factory)?.execution {
            RegisteredSkillExecution::Player(execution) => Some(execution),
            _ => None,
        }
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
        self.skill(skill_id, factory).map_or(0, |skill| skill.last_used_ms)
    }

    pub(crate) fn mark_skill_used(&mut self, skill_id: u32, now_ms: u32, factory: &CSkillFactory) {
        if let Some(skill) = self.skill_mut(skill_id, factory) {
            skill.last_used_ms = now_ms;
        }
    }

    /// Удаление реестра сохраняет неразрешённый current ID. Полный concrete
    /// End перед удалением ещё требует подключения lifecycle владельца.
    pub(crate) fn clear_skills(&mut self, factory: &CSkillFactory) {
        if self.current_skill(factory).is_some() {
            self.current_skill_id = None;
        }
        for category in [SkillCategory::Attack, SkillCategory::Defense, SkillCategory::Summon, SkillCategory::State] {
            self.skills[category as usize].clear();
        }
    }

    /// `CSkillFactory::QuerySkill(SKILL_BASE_DEFENSE, 1)` создавал
    /// `CFightDefense` отдельной ветвью даже без reloadable properties.
    /// Reloadable properties не могут отменить intrinsic defense или
    /// изменить его категорию; имя читается только при обращении к экземпляру.
    pub(crate) fn add_base_defense_skill(&mut self, _factory: &CSkillFactory) {
        let owner = CSkillFactory::factory_owner(SKILL_BASE_DEFENSE)
            .expect("CFightDefense входит в native factory");
        self.skills[SkillCategory::Defense as usize].push(MoveShapeSkill {
            id: SKILL_BASE_DEFENSE,
            level: 1,
            owner,
            item_position: -1,
            execution: RegisteredSkillExecution::Inactive(SkillLifecycle::default()),
            retained_data: SkillRetainedData::for_owner(owner),
            current_visual_effect: None,
            last_used_ms: 0,
        });
    }

    pub(crate) fn set_item_skill_position(&mut self, skill_id: u32, position: i32, factory: &CSkillFactory) -> bool {
        let Some(skill) = self.skill_mut(skill_id, factory) else { return false };
        skill.set_item_position(position);
        true
    }

    /// Выбранный ID независимо от наличия зарегистрированного навыка.
    pub(crate) const fn current_skill_id(&self) -> Option<u32> {
        self.current_skill_id
    }

    /// Проекция GetCurrentSkill в реестр; execution и End остаются у skill-owner.
    pub(crate) fn current_skill(&self, factory: &CSkillFactory) -> Option<&MoveShapeSkill> {
        self.current_skill_id.and_then(|skill_id| self.skill(skill_id, factory))
    }

    /// GetDefaultAttackSkillID (0x004CE240): порядок категорий важнее порядка ID.
    pub(crate) fn default_attack_skill_id(&self) -> u32 {
        if self.skills_in_category(SkillCategory::Attack).any(|skill| skill.id == 2) {
            2
        } else if self.skills_in_category(SkillCategory::Summon).any(|skill| skill.id == 3) {
            3
        } else {
            1
        }
    }

    /// Typed boundary для snapshot/skill caller-а. Полное semantic действие
    /// `SetCurrentSkill` (завершение прежнего concrete skill) не подменяется
    /// записью ID и остаётся у соответствующего owner-а.
    pub(crate) const fn set_current_skill_id(&mut self, skill_id: Option<u32>) {
        self.current_skill_id = skill_id;
    }

    /// Exact `SetItemSkill`: native owner только добавляет ID в ordered vector
    /// непосредственно перед передачей item-skill в `CPlayerAI`.
    pub(crate) fn set_item_skill(&mut self, skill_id: u32) {
        self.item_skill_ids.push(skill_id);
    }

    pub(crate) const fn is_moveable(&self) -> bool {
        self.moveable
    }

    pub(crate) const fn moveable_count(&self) -> i32 {
        self.moveable_count
    }

    /// Exact counter semantics `SetMoveable`: `false` ставит новый запрет,
    /// `true` снимает один; отрицательный счётчик не нормализуется в ветви
    /// снятия и потому сохраняется как наблюдаемая legacy-семантика.
    pub(crate) const fn set_moveable(&mut self, moveable: bool) {
        if !moveable {
            if self.moveable_count < 0 {
                self.moveable_count = 0;
            }
            self.moveable_count = self.moveable_count.wrapping_add(1);
        } else {
            self.moveable_count = self.moveable_count.wrapping_sub(1);
        }
        self.moveable = self.moveable_count < 1;
    }

    /// AddSkill (0x004D1C70): ненулевой уровень не понижается, повышение
    /// удаляет первое совпадение и добавляет новый экземпляр в хвост.
    /// Нулевой уровень прежнего экземпляра допускает повторный ID.
    pub(crate) fn add_skill(&mut self, skill_id: u32, level: i32, factory: &CSkillFactory) -> bool {
        if let Some(existing) = self.skill(skill_id, factory) {
            if existing.level != 0 {
                if level <= existing.level {
                    return true;
                }
                self.delete_skill(skill_id, factory);
            }
        }
        self.insert_new_skill(skill_id, level)
    }

    pub(crate) fn insert_new_skill(&mut self, skill_id: u32, level: i32) -> bool {
        let Some(owner) = CSkillFactory::factory_owner(skill_id) else {
            return false;
        };
        self.skills[owner.category() as usize].push(MoveShapeSkill {
            id: skill_id,
            level,
            owner,
            item_position: -1,
            execution: RegisteredSkillExecution::Inactive(SkillLifecycle::default()),
            retained_data: SkillRetainedData::for_owner(owner),
            current_visual_effect: None,
            last_used_ms: 0,
        });
        true
    }

    /// DelSkill (0x004CF320) удаляет только первый найденный экземпляр.
    /// До category lookup обрабатывается разрешённый current, даже если
    /// удаляется другой ID. UNKNOWN отвергается до этого, а ID 0 — после.
    pub(crate) fn delete_skill(&mut self, skill_id: u32, factory: &CSkillFactory) -> bool {
        if skill_id == super::skills::skillfactory::UNKNOWN_SKILL_ID {
            return false;
        }
        if self.current_skill(factory).is_some() {
            self.current_skill_id = None;
        }
        let Some(category) = SkillCategory::from_raw(factory.query_skill_type(skill_id, 1)) else {
            return false;
        };
        self.delete_skill_in_category(skill_id, category);
        true
    }

    pub(crate) fn delete_skill_in_category(&mut self, skill_id: u32, category: SkillCategory) {
        let skills = &self.skills[category as usize];
        let index = skills.iter().position(|skill| skill.id == skill_id);
        if let Some(index) = index {
            self.skills[category as usize].remove(index);
        }
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
        current_hit_points == 0
    }

    pub(crate) fn get_dest_direction(
        source_x: i32,
        source_y: i32,
        destination_x: i32,
        destination_y: i32,
    ) -> i32 {
        let delta_x = source_x.wrapping_sub(destination_x);
        let delta_y = source_y.wrapping_sub(destination_y);
        // Подтверждённая странность GameServer RVA 0x000CCF60: совпавшие
        // точки возвращают DIR_DOWN `4`, а не отдельный sentinel.
        match (delta_x.signum(), delta_y.signum()) {
            (1, 1) => 7,
            (1, 0) => 6,
            (1, -1) => 5,
            (-1, 1) => 1,
            (-1, 0) => 2,
            (-1, -1) => 3,
            (0, 1) => 0,
            (0, 0 | -1) => 4,
            _ => unreachable!("signum возвращает только -1/0/1"),
        }
    }

    /// Общая геометрия exact overrides `CBuild/CMonster::GetBeAttackedPoint`:
    /// ближайшая клетка footprint с предпочтением прямого направления при
    /// равной Chebyshev-дистанции.
    pub(crate) fn nearest_figure_attack_point(
        tile_x: i32,
        tile_y: i32,
        figure: ShapeFigure,
        attacker_x: i32,
        attacker_y: i32,
    ) -> (i32, i32) {
        let horizontal = i32::from(figure.get(2));
        let vertical = i32::from(figure.get(0));
        let mut best_point = (tile_x, tile_y);
        let mut best_distance = 10_000_000;
        let mut best_direction: i32 = 0;

        for offset_x in -horizontal..=horizontal {
            let candidate_x = tile_x.wrapping_add(offset_x);
            for offset_y in -vertical..=vertical {
                let candidate_y = tile_y.wrapping_add(offset_y);
                let distance_x = candidate_x.wrapping_sub(attacker_x).unsigned_abs() as i32;
                let distance_y = candidate_y.wrapping_sub(attacker_y).unsigned_abs() as i32;
                let distance = distance_x.max(distance_y);
                let direction = Self::get_dest_direction(
                    attacker_x,
                    attacker_y,
                    candidate_x,
                    candidate_y,
                );
                if distance < best_distance
                    || (distance == best_distance
                        && best_direction.rem_euclid(2) == 1
                        && direction.rem_euclid(2) == 0)
                {
                    best_point = (candidate_x, candidate_y);
                    best_distance = distance;
                    best_direction = direction;
                }
            }
        }
        best_point
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

    /// Общий BF604 и clamp; виртуальный SetTileXY и последующий AI Stand
    /// принадлежат конкретному владельцу движения.
    pub(crate) fn force_move_message(
        &self,
        server_region: &CServerRegion,
        destination_x: i32,
        destination_y: i32,
        duration_ms: u32,
    ) -> Result<(ShapeAreaCoordinates, CMessage), MoveShapeCommandBlock> {
        let width = server_region.region.width;
        let height = server_region.region.height;
        let clamped_x = clamp_force_x(destination_x, width);
        let clamped_y = clamp_force_y(destination_y, width, height);
        let old_x = self
            .shape
            .get_tile_x()
            .map_err(MoveShapeCommandBlock::Coordinate)?;
        let old_y = self
            .shape
            .get_tile_y()
            .map_err(MoveShapeCommandBlock::Coordinate)?;
        let identity = self.shape.identity();

        let mut message = CMessage::new(FORCE_MOVE_MESSAGE);
        message.add_long(identity.id);
        message.add_long(identity.object_type);
        message.add_long(old_x);
        message.add_long(old_y);
        message.add_long(clamped_x);
        message.add_long(clamped_y);
        message.add_ulong(duration_ms);
        message.add_long(0);
        Ok((ShapeAreaCoordinates { x: clamped_x, y: clamped_y }, message))
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
        let old_x = self
            .shape
            .get_tile_x()
            .map_err(MoveShapeCommandBlock::Coordinate)?;
        let old_y = self
            .shape
            .get_tile_y()
            .map_err(MoveShapeCommandBlock::Coordinate)?;
        self.shape.set_direction(get_line_direction(
            old_x,
            old_y,
            destination_x,
            destination_y,
        ));
        let identity = self.shape.identity();

        let mut message = CMessage::new(MOVE_MESSAGE);
        message.add_long(identity.id);
        message.add_long(identity.object_type);
        message.add_long(old_x);
        message.add_long(old_y);
        message.add_byte(1);
        message.add_byte(2 + u8::from(run != 0));
        message.add_long(destination_x);
        message.add_long(destination_y);
        message.add_long(destination_x);
        message.add_long(destination_y);
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
        let region = &server_region.region;
        if destination_x < 0
            || destination_x >= region.width
            || destination_y < 0
            || destination_y >= region.height
        {
            return Ok(false);
        }
        if region
            .get_block(destination_x, destination_y)
            .map_err(MoveShapeCommandBlock::RegionCell)?
            != 0
        {
            return Ok(false);
        }

        let identity = self.shape.identity();
        let mut message = CMessage::new(SET_POSITION_MESSAGE);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.add_long(destination_x);
        message.add_long(destination_y);
        message.add_long(0);
        let _ = message
            .send_to_around(Some(&*server_region), &self.shape, None, around)
            .map_err(MoveShapeCommandBlock::Coordinate)?;

        server_region
            .set_move_shape_tile_position(&mut self.shape, destination_x, destination_y, facts)
            .map_err(MoveShapeCommandBlock::Position)?;
        Ok(true)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MoveShapePositionDispatch {
    pub(crate) facts: MoveShapePositionFacts,
}

impl ShapePositionDispatch for MoveShapePositionDispatch {
    type Error = MoveShapePositionBlock;

    fn set_pos_xy(
        &mut self,
        region: &mut CRegion,
        shape: &mut CShape,
        x: f32,
        y: f32,
    ) -> Result<(), Self::Error> {
        set_pos_xy_core(Some(region), shape, x, y, self.facts)
    }
}

fn set_pos_xy_core(
    region: Option<&mut CRegion>,
    shape: &mut CShape,
    x: f32,
    y: f32,
    facts: MoveShapePositionFacts,
) -> Result<(), MoveShapePositionBlock> {
    if let Some(region) =
        region.filter(|region| shape.is_assigned_to_server_region() && region.width != 0)
    {
        let old_y = shape
            .get_tile_y()
            .map_err(MoveShapePositionBlock::Coordinate)?;
        let old_x = shape
            .get_tile_x()
            .map_err(MoveShapePositionBlock::Coordinate)?;
        shape
            .set_block(region, old_x, old_y, 0, facts.figure)
            .map_err(MoveShapePositionBlock::ShapeBlock)?;

        if facts.current_hit_points != 0 || shape.identity().object_type == NPC_TYPE {
            let new_y = CShape::tile_from_value(y).map_err(MoveShapePositionBlock::Coordinate)?;
            let new_x = CShape::tile_from_value(x).map_err(MoveShapePositionBlock::Coordinate)?;
            shape
                .set_block(region, new_x, new_y, 3, facts.figure)
                .map_err(MoveShapePositionBlock::ShapeBlock)?;
        }
    }

    shape.set_pos_xy_move_order(x, y);
    let tile_y = CShape::tile_from_value(y).map_err(MoveShapePositionBlock::Coordinate)?;
    let tile_x = CShape::tile_from_value(x).map_err(MoveShapePositionBlock::Coordinate)?;
    if facts.area_width <= 0 || facts.area_height <= 0 {
        return Err(MoveShapePositionBlock::InvalidAreaSpan {
            width: facts.area_width,
            height: facts.area_height,
        });
    }

    let next_area = ShapeAreaCoordinates {
        x: tile_x / facts.area_width,
        y: tile_y / facts.area_height,
    };
    if facts
        .current_area
        .is_some_and(|current| current != next_area)
    {
        shape.set_next_area_coordinates(next_area);
        shape.set_change_state(SHAPE_CHANGE_AREA);
    } else {
        shape.set_change_state(SHAPE_CHANGE_NONE);
    }
    Ok(())
}

fn clamp_force_x(destination: i32, width: i32) -> i32 {
    if destination < 0 {
        0
    } else if destination >= width {
        width.wrapping_sub(1)
    } else {
        destination
    }
}

fn clamp_force_y(destination: i32, width: i32, height: i32) -> i32 {
    if destination < 0 {
        0
    } else if destination >= height {
        // Подтверждённая странность GameServer RVA 0x000CD1A0:
        // `if (height <= lDestY) lDestY = width - 1;`.
        width.wrapping_sub(1)
    } else {
        destination
    }
}

fn update_known_state_record(payload: &mut [u8], state_id: u32, record: &[u8]) {
    update_nth_known_state_record(payload, state_id, 0, record);
}

fn update_nth_known_state_record(payload: &mut [u8], state_id: u32, occurrence: usize, record: &[u8]) {
    if let Some(offset) = known_state_record_offsets(payload)
        .into_iter()
        .filter(|offset| read_u32(payload, *offset) == Some(state_id))
        .nth(occurrence)
        && let Some(destination) = payload.get_mut(offset..offset + record.len())
    {
        destination.copy_from_slice(record);
    }
}

fn read_u16(source: &[u8], offset: usize) -> Option<u16> {
    LegacyReader::at(source, offset).ok()?.read_u16().ok()
}

fn read_i16(source: &[u8], offset: usize) -> Option<i16> {
    LegacyReader::at(source, offset).ok()?.read_i16().ok()
}

fn read_u32(source: &[u8], offset: usize) -> Option<u32> {
    LegacyReader::at(source, offset).ok()?.read_u32().ok()
}

fn read_i32(source: &[u8], offset: usize) -> Option<i32> {
    LegacyReader::at(source, offset).ok()?.read_i32().ok()
}

fn write_u16(destination: &mut [u8], offset: usize, value: u16) {
    LegacyWriter::write_u16_at(destination, offset, value).expect("проверенное поле состояния");
}

fn write_i16(destination: &mut [u8], offset: usize, value: i16) {
    LegacyWriter::write_i16_at(destination, offset, value).expect("проверенное поле состояния");
}

fn write_u32(destination: &mut [u8], offset: usize, value: u32) {
    LegacyWriter::write_u32_at(destination, offset, value).expect("проверенное поле состояния");
}

fn write_i32(destination: &mut [u8], offset: usize, value: i32) {
    LegacyWriter::write_i32_at(destination, offset, value).expect("проверенное поле состояния");
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h

// ============================================================================
// FUNCTION: CMoveShape::God
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:151
// RVA: 0x0002ACB0
// ADDRESS: 0042acb0
// PROTOTYPE: void __thiscall God(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::CanMove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:155
// RVA: 0x0002ACC0
// ADDRESS: 0042acc0
// PROTOTYPE: int __thiscall CanMove(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetBeAttackedPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:76
// RVA: 0x0004A250
// ADDRESS: 0044a250
// PROTOTYPE: void __thiscall GetBeAttackedPoint(long param_1, long param_2, long * param_3, long * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetAttackerDir
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:80
// RVA: 0x0004A270
// ADDRESS: 0044a270
// PROTOTYPE: long __thiscall GetAttackerDir(long param_1, long param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetAI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:90
// RVA: 0x0004A280
// ADDRESS: 0044a280
// PROTOTYPE: CBaseAI * __thiscall GetAI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetAlertRange
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:142
// RVA: 0x0004A290
// ADDRESS: 0044a290
// PROTOTYPE: long __thiscall GetAlertRange(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetTrackRange
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:143
// RVA: 0x0004A2A0
// ADDRESS: 0044a2a0
// PROTOTYPE: long __thiscall GetTrackRange(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::SetAttackAble
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:332
// RVA: 0x0004A2B0
// ADDRESS: 0044a2b0
// PROTOTYPE: void __thiscall SetAttackAble(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetAttackAble
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:333
// RVA: 0x0004A2C0
// ADDRESS: 0044a2c0
// PROTOTYPE: bool __thiscall GetAttackAble(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::SetFightable
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:119
// RVA: 0x000CCE10
// ADDRESS: 004cce10
// PROTOTYPE: void __thiscall SetFightable(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::SetKilledMeAttackInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:139
// RVA: 0x000CCE50
// ADDRESS: 004cce50
// PROTOTYPE: void __thiscall SetKilledMeAttackInfo(tagAttackInformation * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetAtcInterval
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:367
// RVA: 0x000CCF40
// ADDRESS: 004ccf40
// PROTOTYPE: ushort __thiscall GetAtcInterval(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetStrikeOutTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:373
// RVA: 0x000CCF50
// ADDRESS: 004ccf50
// PROTOTYPE: ulong __thiscall GetStrikeOutTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CMoveShape::GetDestDir` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CMoveShape::GetCurrentPetsMode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2947
// RVA: 0x000CCFC0
// ADDRESS: 004ccfc0
// PROTOTYPE: PET_SEARCH_ENEMY_MODE __thiscall GetCurrentPetsMode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::FindPositionForCarriage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3015
// RVA: 0x000CCFD0
// ADDRESS: 004ccfd0
// PROTOTYPE: bool __thiscall FindPositionForCarriage(CMoveShape * param_1, long * param_2, long * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CMoveShape::SetPosXY` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CMoveShape::OnChangeStates
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2359
// RVA: 0x000CD3E0
// ADDRESS: 004cd3e0
// PROTOTYPE: void __thiscall OnChangeStates(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CMoveShape::OnMove` материализован выше; покрытый raw-блок
// удалён.

// IMPLEMENTED: `CMoveShape::OnSetPosition` материализован выше; покрытый
// raw-блок удалён.

// ============================================================================
// FUNCTION: CMoveShape::GetPetsAmount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2823
// RVA: 0x000CD6D0
// ADDRESS: 004cd6d0
// PROTOTYPE: ulong __thiscall GetPetsAmount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::Evanish
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2977
// RVA: 0x000CD700
// ADDRESS: 004cd700
// PROTOTYPE: void __thiscall Evanish(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::DelCarriage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3109
// RVA: 0x000CD7C0
// ADDRESS: 004cd7c0
// PROTOTYPE: void __thiscall DelCarriage(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::DoesStateExist
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:634
// RVA: 0x000CD9C0
// ADDRESS: 004cd9c0
// PROTOTYPE: int __thiscall DoesStateExist(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetStateBySkillID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:648
// RVA: 0x000CDA10
// ADDRESS: 004cda10
// PROTOTYPE: CState * __thiscall GetStateBySkillID(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetStateNumByStateID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:662
// RVA: 0x000CDA60
// ADDRESS: 004cda60
// PROTOTYPE: uint __thiscall GetStateNumByStateID(tagStateID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// ============================================================================
// FUNCTION: CMoveShape::GetCurrentSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1720
// RVA: 0x000CDC10
// ADDRESS: 004cdc10
// PROTOTYPE: CSkill * __thiscall GetCurrentSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// StartAllStates 0x004CE050 (moveshape.cpp:2234) перенесён в общий
// states/state.rs: SetRegion → перечитать позицию → Begin(NULL, holder),
// включая changing-region исключения и отдельный CHBY Begin(0, holder, true).

// ============================================================================
// FUNCTION: CMoveShape::OnAction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2303
// RVA: 0x000CE1E0
// ADDRESS: 004ce1e0
// PROTOTYPE: void __thiscall OnAction(tagAction param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// GetDefaultAttackSkillID материализован в default_attack_skill_id.

// ============================================================================

// ============================================================================
// FUNCTION: CMoveShape::FindPositionForPet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2771
// RVA: 0x000CE450
// ADDRESS: 004ce450
// PROTOTYPE: int __thiscall FindPositionForPet(CMoveShape * param_1, long * param_2, long * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// ============================================================================
// FUNCTION: CMoveShape::GetExState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3365
// RVA: 0x000CEB10
// ADDRESS: 004ceb10
// PROTOTYPE: uint __thiscall GetExState(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetExStateNew
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3384
// RVA: 0x000CEB70
// ADDRESS: 004ceb70
// PROTOTYPE: uint __thiscall GetExStateNew(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// ============================================================================
// FUNCTION: CMoveShape::SetCurrentSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1704
// RVA: 0x000CEEE0
// ADDRESS: 004ceee0
// PROTOTYPE: void __thiscall SetCurrentSkill(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::OnEnterRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2043
// RVA: 0x000CEF40
// ADDRESS: 004cef40
// PROTOTYPE: void __thiscall OnEnterRegion(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// CMoveShape::ClearAllStates (0x004CF090, moveshape.cpp:2132)
// реализован общим states/state.rs::clear_move_shape_states; End и последующий
// destructor-only проход сохраняют позиции, death-фильтры и UpdateProperty.

// ============================================================================
// FUNCTION: CMoveShape::CheckSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2664
// RVA: 0x000CF540
// ADDRESS: 004cf540
// PROTOTYPE: long __thiscall CheckSkill(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CMoveShape::CheckSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2652
// RVA: 0x000CF590
// ADDRESS: 004cf590
// PROTOTYPE: long __thiscall CheckSkill(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::RemovePet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2753
// RVA: 0x000CF5D0
// ADDRESS: 004cf5d0
// PROTOTYPE: int __thiscall RemovePet(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetValidPetsAmount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2828
// RVA: 0x000CF650
// ADDRESS: 004cf650
// PROTOTYPE: ulong __thiscall GetValidPetsAmount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::~CMoveShape
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:82
// RVA: 0x000CF950
// ADDRESS: 004cf950
// PROTOTYPE: void __thiscall ~CMoveShape(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::OnBeginSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:283
// RVA: 0x000CFB40
// ADDRESS: 004cfb40
// PROTOTYPE: int __thiscall OnBeginSkill(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetWeaponModifier
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:358
// RVA: 0x000CFB50
// ADDRESS: 004cfb50
// PROTOTYPE: float __thiscall GetWeaponModifier(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: Catch@004cfbff
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:107
// RVA: 0x000CFBFF
// ADDRESS: 004cfbff
// PROTOTYPE: undefined Catch@004cfbff()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: Catch@004d002e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:301
// RVA: 0x000D002E
// ADDRESS: 004d002e
// PROTOTYPE: undefined Catch@004d002e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::AddCarriage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3036
// RVA: 0x000D0140
// ADDRESS: 004d0140
// PROTOTYPE: bool __thiscall AddCarriage(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:230
// RVA: 0x000D0530
// ADDRESS: 004d0530
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::CMoveShape
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:50
// RVA: 0x000D0C60
// ADDRESS: 004d0c60
// PROTOTYPE: undefined __thiscall CMoveShape(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: CMoveShape::AddExStatesToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1814
// RVA: 0x000D10F0
// ADDRESS: 004d10f0
// PROTOTYPE: bool __thiscall AddExStatesToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetAllPets
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2931
// RVA: 0x000D1270
// ADDRESS: 004d1270
// PROTOTYPE: void __thiscall GetAllPets(vector<CMonster*,std::allocator<CMonster*>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::InitSkills
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:182
// RVA: 0x000D1460
// ADDRESS: 004d1460
// PROTOTYPE: void __thiscall InitSkills(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// ============================================================================
// FUNCTION: CMoveShape::DecodeExStatesFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1993
// RVA: 0x000D1A80
// ADDRESS: 004d1a80
// PROTOTYPE: void __thiscall DecodeExStatesFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CMoveShape::AddPet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2742
// RVA: 0x000D1E00
// ADDRESS: 004d1e00
// PROTOTYPE: int __thiscall AddPet(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// ============================================================================
// CMoveShape::prison_check (0x004D2360, moveshape.cpp:3404)
// реализован CGame::check_move_shape_prison: fresh victim-region, tamed master,
// PrisonConf::operator[], GS0126 и существующий ChangeRegion после очистки.


// ============================================================================
// FUNCTION: CMoveShape::OnBeenAttacked
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:762
// RVA: 0x000D2890
// ADDRESS: 004d2890
// PROTOTYPE: void __thiscall OnBeenAttacked(tagAttackInformation * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
